"""Seeded, stratified casting for S26 (step-26 §7.4a items 4-5) — the minimal deterministic version.

Reads the population profiles (characters/populations/*.yaml), the casting inputs
(characters/cast/casting_inputs.json: what each biography fixes, and the director's outfit) and the style
template (characters/style/template.yaml), and writes, per person, characters/cast/<key>/brief.yaml,
reference_prompt.txt and portrait_prompt.txt, plus characters/cast/CASTING.md (the distinctness table and the
realised distribution against the profile).

Determinism: every draw comes from random.Random(sha256(seed | attempt | world | attribute | key)); strata
are allocated by largest remainder over the world's whole cast, fixed members included; the remaining
multiset is matched to people in a seeded order, respecting what the biography allows. If the cast fails
the §7.3 distinctness rule, the attempt counter is raised and everything sampled is re-drawn; the attempt
used is recorded. Re-running with the same inputs reproduces every file byte for byte (X-24).

Never reads a surname. Stdlib only; no network; no image or Meshy call.

    python3 tools/character_pipeline/casting/cast_director.py [--check]

--check re-generates in memory and fails if any committed file differs.
"""

from __future__ import annotations

import hashlib
import json
import random
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CHARS = ROOT / "presentation/mineworld-default/3D/characters"
CAST = CHARS / "cast"
BANDS = ["20s", "30s", "40s", "50s", "60s", "70s", "80_plus"]
BAND_YEARS = {"20s": (20, 29), "30s": (30, 39), "40s": (40, 49), "50s": (50, 59), "60s": (60, 69),
              "70s": (70, 79), "80_plus": (80, 88)}
ANCESTRY_WORDS = {
    "hispanic_or_latino": "Hispanic or Latino",
    "nh_white": "non-Hispanic White",
    "nh_asian": "non-Hispanic Asian",
    "nh_black": "non-Hispanic Black",
    "nh_multiracial_or_other": "non-Hispanic multiracial or other",
}
MST_WORDS = {1: "very fair", 2: "fair", 3: "light", 4: "light-medium", 5: "medium", 6: "medium-tan",
             7: "tan-brown", 8: "brown", 9: "deep brown", 10: "very deep brown"}
BUILD_WORDS = {"slim": "slim", "average": "average", "solid": "solid, sturdy", "heavy": "heavy-set, full-figured"}
# Authored casting conventions (operator-gated at the casting page), not data.
HAIR_STYLES = {
    "female": ["chin-length bob", "pixie cut", "low tied-back bun", "low ponytail tied at the nape",
               "short natural curls", "braided crown updo", "close-cropped natural hair", "high top knot"],
    "male": ["short crop", "buzz cut", "short side part", "short curly top", "shaved head",
             "close-cropped natural hair", "short swept-back hair", "short wavy hair"],
}
TIED = {"low tied-back bun", "low ponytail tied at the nape", "braided crown updo", "high top knot",
        "pixie cut", "close-cropped natural hair"}
FACIAL = [("none", 0.55), ("stubble", 0.15), ("short trimmed beard", 0.2), ("moustache", 0.1)]


# ---------------------------------------------------------------- tiny readers (no YAML library)

def _scalar(text: str):
    text = text.strip().strip("'\"")
    try:
        return float(text) if "." in text else int(text)
    except ValueError:
        return text


def _split_top(text: str) -> list[str]:
    parts, depth, cur = [], 0, ""
    for ch in text:
        if ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    return parts


def _flow(text: str):
    text = text.strip()
    if text.startswith("["):
        return [_flow(p) for p in _split_top(text[1:-1])]
    if text.startswith("{"):
        out = {}
        for p in _split_top(text[1:-1]):
            k, v = p.split(":", 1)
            out[k.strip()] = _flow(v)
        return out
    return _scalar(text)


def read_sections(path: Path, names: tuple[str, ...]) -> dict:
    """Parse top-level sections `names` of a flat profile YAML: nested `key: value` maps, flow values."""
    out: dict = {}
    stack: list[tuple[int, dict]] = []
    for raw in path.read_text().splitlines():
        line = re.sub(r"\s+#.*$", "", raw).rstrip()
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip())
        key, _, value = line.strip().partition(":")
        if indent == 0:
            stack = [(0, out.setdefault(key, {}))] if key in names else []
            if key not in names and value.strip():
                out.setdefault("_top", {})[key] = _scalar(value)
            continue
        if not stack:
            continue
        while stack and stack[-1][0] >= indent and len(stack) > 1:
            stack.pop()
        parent = stack[-1][1]
        if value.strip():
            parent[key] = _flow(value)
        else:
            child: dict = {}
            parent[key] = child
            stack.append((indent, child))
    return out


def read_block(path: Path, section: str, key: str) -> str:
    """The `key: |` block scalar under top-level `section` of the template."""
    lines = path.read_text().splitlines()
    in_section, grab, body, base = False, False, [], None
    for line in lines:
        if not line.startswith(" ") and line.strip():
            if grab:
                break
            in_section = line.startswith(section + ":")
            continue
        if in_section and line.strip() == f"{key}: |":
            grab = True
            continue
        if grab:
            if line.strip() and base is None:
                base = len(line) - len(line.lstrip())
            if line.strip() and len(line) - len(line.lstrip()) < (base or 0):
                break
            body.append(line[base:] if base and len(line) >= base else line.strip())
    return "\n".join(body).rstrip() + "\n"


def load_profile(name: str) -> dict:
    path = CHARS / "populations" / f"{name}.yaml"
    if not path.exists():
        sys.exit(f"cast_director: population profile `{name}` not found at {path}")
    data = read_sections(path, ("sampler", "appearance_conventions"))
    of = data.get("_top", {}).get("profile_of")
    if of:
        return load_profile(of) | {"_via": name}
    if "sampler" not in data or "appearance_conventions" not in data:
        sys.exit(f"cast_director: profile `{name}` lacks a `sampler` or `appearance_conventions` section")
    return data | {"_name": name}


# ---------------------------------------------------------------- the sampler

def rng(seed: str, *parts) -> random.Random:
    digest = hashlib.sha256("|".join([seed, *map(str, parts)]).encode()).digest()
    return random.Random(int.from_bytes(digest[:8], "big"))


def largest_remainder(shares: dict[str, float], n: int) -> dict[str, int]:
    total = sum(shares.values())
    quotas = {k: v / total * n for k, v in shares.items()}
    counts = {k: int(q) for k, q in quotas.items()}
    order = sorted(quotas, key=lambda k: (-(quotas[k] - counts[k]), k))
    for k in order[: n - sum(counts.values())]:
        counts[k] += 1
    return counts


def match(multiset: list[str], people: list[dict], allowed, r: random.Random) -> dict[str, str] | None:
    """Assign the multiset's values to people, each within `allowed(person)`; seeded, backtracking."""
    pool = sorted(multiset)
    r.shuffle(pool)
    order = sorted(people, key=lambda p: (len([v for v in set(pool) if v in allowed(p)]), p["key"]))
    result: dict[str, str] = {}

    def go(i: int, left: list[str]) -> bool:
        if i == len(order):
            return True
        p = order[i]
        tried = set()
        for j, v in enumerate(left):
            if v in tried or v not in allowed(p):
                continue
            tried.add(v)
            result[p["key"]] = v
            if go(i + 1, left[:j] + left[j + 1:]):
                return True
        return False

    return result if go(0, pool) else None


def fixed(person: dict, attr: str):
    v = person.get(attr)
    return v["value"] if isinstance(v, dict) and "value" in v else v


def cast_world(world: str, spec: dict, people: list[dict], inputs: dict, attempt: int) -> list[dict]:
    prof = load_profile(spec["profile"])
    s, conv, seed = prof["sampler"], prof["appearance_conventions"], inputs["seed"]
    members = spec["fixed_members"] + people
    n = len(members)
    out = {p["key"]: {"sources": {}} for p in people}

    def stratify(attr: str, shares: dict, group: list[dict], allowed):
        counts = largest_remainder(shares, len(group))
        for m in group:
            v = fixed(m, attr)
            if v is not None:
                counts[v] -= 1
        free = [p for p in group if fixed(p, attr) is None and p in people]
        multiset = [k for k, c in counts.items() for _ in range(max(c, 0))]
        got = match(multiset, free, allowed, rng(seed, attempt, world, attr))
        if got is None:
            sys.exit(f"cast_director: {world}: no assignment of `{attr}` satisfies the biographies")
        for p in group:
            if p not in people:
                continue
            if fixed(p, attr) is not None:
                out[p["key"]][attr] = fixed(p, attr)
                out[p["key"]]["sources"][attr] = "biography: " + p[attr]["why"]
            else:
                out[p["key"]][attr] = got[p["key"]]
                out[p["key"]]["sources"][attr] = "sampler (stratified, largest remainder)"

    for p in people:
        out[p["key"]]["sex"] = fixed(p, "sex")
        out[p["key"]]["sources"]["sex"] = "biography: " + p["sex"]["why"]

    def age_ok(p):
        a = p.get("age_allowed")
        if not a:
            return set(BANDS)
        return set(inputs["work_age_bands"] if a["value"] == "work" else a["value"])

    stratify("age_band", s["age_band"], members, age_ok)
    stratify("ancestry", s["ancestry"], members, lambda p: set(ANCESTRY_WORDS))
    for sex in ("female", "male"):
        group = [m for m in members if fixed(m, "sex") == sex]
        stratify("build", s["build"][sex], group,
                 lambda p: set(fixed(p, "build_allowed") or BUILD_WORDS))
    for key in out:
        if "age_allowed" in next(p for p in people if p["key"] == key) and \
                out[key]["sources"]["age_band"].startswith("sampler"):
            why = next(p for p in people if p["key"] == key)["age_allowed"]["why"]
            out[key]["sources"]["age_band"] += f", constrained by the biography ({why})"

    for p in people:
        o, k = out[p["key"]], p["key"]
        lo, hi = BAND_YEARS[o["age_band"]]
        o["age"] = rng(seed, attempt, world, "age", k).randint(lo, hi)
        o["sources"]["apparent_age"] = "sampler (uniform within the band)"
        if fixed(p, "stature_m"):
            o["stature_m"] = fixed(p, "stature_m")
            o["sources"]["stature_m"] = "biography: " + p["stature_m"]["why"]
        else:
            mean, p5, p95 = s["stature_cm"][o["sex"]][o["age_band"]]
            sd = (p95 - p5) / 3.29
            draw = rng(seed, attempt, world, "stature", k).gauss(mean, sd)
            o["stature_m"] = round(min(max(draw, mean - 2 * sd), mean + 2 * sd) / 100, 2)
            o["sources"]["stature_m"] = (f"sampler (NHANES {o['sex']} {o['age_band']}: mean {mean} cm, "
                                         f"SD {sd:.1f} cm, clipped to +/- 2 SD)")
        if fixed(p, "skin_mst"):
            o["skin_mst"] = fixed(p, "skin_mst")
            o["sources"]["skin"] = "biography: " + p["skin_mst"]["why"]
        else:
            lo_m, hi_m = conv["skin_mst_range"][o["ancestry"]]
            o["skin_mst"] = rng(seed, attempt, world, "skin", k).randint(lo_m, hi_m)
            o["sources"]["skin"] = "sampler (MST range of the ancestry stratum)"
        if fixed(p, "hair"):
            o["hair"] = fixed(p, "hair")
            o["sources"]["hair"] = "biography: " + p["hair"]["why"]
        else:
            r = rng(seed, attempt, world, "hair", k)
            styles = HAIR_STYLES[o["sex"]]
            if fixed(p, "hair_tied"):
                styles = [h for h in styles if h in TIED]
            colour = r.choice(conv["hair_colour_pool"][o["ancestry"]])
            grey = conv["grey_by_age"].get(o["age_band"], 0)
            if r.random() < grey:
                colour = "grey" if o["age_band"] in ("50s", "60s") else r.choice(["silver-grey", "white"])
            o["hair"] = {"category": r.choice(styles), "colour": colour}
            o["sources"]["hair"] = "sampler (style list by sex" + (", tied back by the biography" if
                                   fixed(p, "hair_tied") else "") + "; colour pool of the stratum; grey by age)"
        if o["sex"] == "male":
            if fixed(p, "facial_hair"):
                o["facial_hair"] = fixed(p, "facial_hair")
                o["sources"]["facial_hair"] = "biography: " + p["facial_hair"]["why"]
            else:
                r, acc = rng(seed, attempt, world, "facial", k), 0.0
                x = r.random()
                for name, w in FACIAL:
                    acc += w
                    if x < acc:
                        o["facial_hair"] = name
                        break
                o["sources"]["facial_hair"] = "sampler (authored weights)"
        o["outer"] = f"{p['outfit']['outer_colour']} {p['outfit']['outer']}"
    return [out[p["key"]] | {"person": p} for p in people]


# ---------------------------------------------------------------- distinctness (§7.3)

def differences(a: dict, b: dict) -> list[str]:
    d = []
    if a["age_band"] != b["age_band"]:
        d.append("age")
    if abs(a["stature_m"] - b["stature_m"]) >= 0.05 - 1e-9:
        d.append("stature")
    if a["build"] != b["build"]:
        d.append("build")
    if abs(a["skin_mst"] - b["skin_mst"]) >= 2:
        d.append("skin")
    if a["hair"]["category"] != b["hair"]["category"] or a["hair"]["colour"] != b["hair"]["colour"]:
        d.append("hair")
    if a["outer"] != b["outer"]:
        d.append("outer")
    return d


def fixed_row(m: dict) -> dict:
    return {"key": m["key"], "sex": m["sex"], "age_band": m["age_band"], "stature_m": m["stature_m"],
            "build": m["build"], "skin_mst": m["skin_mst"], "hair": m["hair"], "outer": m["outer"],
            "ancestry": m["ancestry"]}


def lineup(world_spec: dict, cast: list[dict]) -> list[dict]:
    return [fixed_row(m) for m in world_spec["fixed_members"]] + \
           [c | {"key": c["person"]["key"]} for c in cast]


def failing_pairs(rows: list[dict]) -> list[tuple]:
    bad = []
    for i, a in enumerate(rows):
        for b in rows[i + 1:]:
            d = differences(a, b)
            if len(d) < 3:
                bad.append((a["key"], b["key"], d))
    return bad


FINDINGS = """## Findings of the casting director (2026-10-15)

- **F-CAST-1 — Vera has no full-body reference.** Her only approved image, and the input of her accepted
  Meshy run, is the chest-up street illustration `references/04_character_closeup.png`; its prompt was never
  recorded. Template v1 uses it as the style anchor (`../style/anchor.png`, a byte copy) and takes the
  full-body framing, A-pose, lighting and backdrop from route D+'s Meshy input needs instead. Option for the
  operator: generate a full-body A-pose Vera from the anchor (one OpenAI call) and make it anchor v2 before
  the pilot's casting gate.
- **F-CAST-2 — Lakeside declares no setting.** It is cast from the San Diego County profile
  (`../populations/lakeside.yaml` gives the reasons: E-2 extras on the San Diego street; the only declared
  setting; never from surnames). The operator may name another setting; only sampled attributes change.
- **F-CAST-3 — Seven Lakeside person files state no pronoun** (bruno, celia, dario, elin, finn, greta, hugo).
  Their sex is taken from the given name and, where present, the tag (fisherman, ferryman). Confirm, or have
  the world's files state it.
- **F-CAST-4 — Build follows national NHANES 2015-2018** (BMI >= 30 for about 42 % of adults), because NHANES
  has no county figures. San Diego County is probably leaner (CHIS, **verify**); with a county figure the
  heavy class would shrink by about one person per world.
- **F-CAST-5 — Sampled statures outside §7.2's literal ranges.** ivan-petrov 1.59 m (a man of 71) and
  greta-novak 1.54 m (a woman of 70) lie inside NHANES's mean +/- 2 SD for their sex and age band, which
  §7.4a's real-world rule asks for, but outside §7.2's default windows (men 1.66-1.92, women 1.55-1.78) that
  G-4 cites. Proposed bounded correction: G-4 checks the profile's band +/- 2 SD (the ARC-95 dated note).
- **F-CAST-6 — Vera's 1.767 m is above the 95th percentile** of US women in their twenties (172.8 cm).
  Informational: she is accepted and fixed; the sampler counts her as she is.
- **F-CAST-7 — Ancestry is sampled, never read from a name**, so several people do not match what their
  surname might suggest (for example grace-liu and hana-sato are sampled Hispanic or Latino). This is §7.4a
  item 5 working as specified; the operator sees it at the casting gate. The prompts state the census
  category in words, with an explicit no-caricature line; the operator may prefer skin and hair only.
""".splitlines()


# ---------------------------------------------------------------- outputs

def person_paragraph(c: dict, portrait: bool) -> str:
    p, o = c["person"], c["person"]["outfit"]
    noun = "woman" if c["sex"] == "female" else "man"
    poss = "her" if c["sex"] == "female" else "his"
    third = "early" if c["age"] % 10 <= 3 else ("mid" if c["age"] % 10 <= 6 else "late")
    age = f"in {poss} {third} {c['age'] // 10 * 10}s" if c["age"] < 80 else f"in {poss} eighties"
    stoop = " with a slight forward stoop of age" if c["age"] >= 75 else ""
    lines = [
        f"THE PERSON: a {noun} {age}, apparent age about {c['age']}, "
        f"{c['stature_m']:.2f} m tall, {'an' if c['build'] == 'average' else 'a'} {BUILD_WORDS[c['build']]} build{stoop}.",
        f"Ancestry, as the US census would record it: {ANCESTRY_WORDS[c['ancestry']]}. Natural, individual "
        "facial features; an ordinary real face, never a caricature or a stereotype.",
        f"Skin: {MST_WORDS[c['skin_mst']]} skin tone (Monk Skin Tone {c['skin_mst']} of 10), natural and even.",
        f"Hair: {c['hair']['colour']} hair, {c['hair']['category']}, kept clear of the shoulders.",
    ]
    if c.get("facial_hair") and c["facial_hair"] != "none":
        lines.append(f"Facial hair: {c['facial_hair']}" + (", grey" if "grey" in c["hair"]["colour"] or
                                                           "white" in c["hair"]["colour"] else "") + ".")
    elif c["sex"] == "male":
        lines.append("Clean-shaven.")
    if p.get("headwear"):
        lines.append(f"Headwear: {p['headwear']}.")
    wear = f"Wearing: {o['outer_colour']} {o['outer']}"
    if o["under"] not in ("none", "none visible"):
        wear += f" over a {o['under']}"
    if portrait:
        lines.append(wear + ".")
    else:
        lines.append(wear + f"; {o['legs']}; {o['feet']}.")
    head = p.get("headwear", "")
    acc = [a for a in p["accessories"] if a != "apron" and a.split()[-1] not in head]
    if acc and not portrait:
        acc = [re.sub(r"\s*\(G-\d+\)", "", a) for a in acc]
        lines.append("Carrying or wearing: " + "; ".join(acc) + " — worn on the body, hands still empty.")
    lines.append(f"Manner, shown only through a calm neutral face and posture: {p['manner']}.")
    lines.append("AVOID: " + "; ".join(p["avoid"]) + "; no text or logos on clothing.")
    return "\n".join(lines) + "\n"


def q(v) -> str:
    s = str(v)
    return json.dumps(s, ensure_ascii=False) if re.search(r"[:#,\[\]{}()&*!|>'\"%@`]|^\s|\s$", s) or s == "" else s


def brief_yaml(c: dict, world: str, worlds: list[str], inputs: dict, attempt: int, profile: str) -> str:
    p, o = c["person"], c["person"]["outfit"]
    files = [f"worlds/{w}/people/{p['file']}" for w in worlds]
    lines = [
        "# Casting brief (step-26 §7.1), generated by tools/character_pipeline/casting/cast_director.py.",
        "# Do not edit by hand: change casting_inputs.json or the profile and re-run. `sources` records which",
        "# attributes came from the biography and which from the sampler (§7.4a item 5).",
        f"person: {p['key']}",
        "from: [" + ", ".join(files) + "]",
        f"role: {q(p['role'])}",
        f"sex: {c['sex']}",
        f"apparent_age: {c['age']}",
        f"age_band: {c['age_band']}",
        f"stature_m: {c['stature_m']:.2f}            # sole to crown, hair excluded (HUMANOID_PROFILE.md §2)",
        f"build: {c['build']}",
        f"ancestry: {c['ancestry']}           # ACS race and Hispanic-origin stratum; gated at casting (Q-S26-7, ARC-95)",
        f"skin: {{ mst: {c['skin_mst']}, words: {q(MST_WORDS[c['skin_mst']])} }}",
        f"hair: {{ category: {q(c['hair']['category'])}, colour: {q(c['hair']['colour'])} }}",
    ]
    if c.get("facial_hair"):
        lines.append(f"facial_hair: {q(c['facial_hair'])}")
    if p.get("headwear"):
        lines.append(f"headwear: {q(p['headwear'])}")
    lines += [
        f"outfit: {{ outer: {q(o['outer_colour'] + ' ' + o['outer'])}, under: {q(o['under'])}, "
        f"legs: {q(o['legs'])}, feet: {q(o['feet'])} }}",
        "accessories: [" + ", ".join(q(a) for a in p["accessories"]) + "]",
        f"manner: {q(p['manner'])}",
        "avoid: [" + ", ".join(q(a) for a in p["avoid"]) + "]   # §7.4",
        "sits: true                 # every cast member may sit (A-4 decor, E-2 extras): G-13 applies",
    ]
    if p.get("identity_source"):
        lines.append(f"identity_source: {q(p['identity_source'])}")
    lines += [
        "casting:",
        f"  world: {world}",
        f"  profile: {profile}",
        f"  template_version: {inputs['template_version']}",
        f"  seed: {inputs['seed']}",
        f"  attempt: {attempt}",
        "  outfit_source: director (from role, tags and note; San Diego climate; §7.4 avoid list)",
        "sources:",
    ]
    for k, v in c["sources"].items():
        lines.append(f"  {k}: {q(v)}")
    return "\n".join(lines) + "\n"


def casting_md(results: dict, inputs: dict) -> str:
    out = ["# Casting — S26 cast director output",
           "",
           "Generated by `tools/character_pipeline/casting/cast_director.py` from `casting_inputs.json`, the",
           "population profiles in `../populations/` and the style template `../style/template.yaml`. Do not edit",
           "by hand. Specification: `.structured-coding/plans/mvp0/step-26-cast.md` §§7.1-7.4a.",
           "",
           f"Seed `{inputs['seed']}`, template v{inputs['template_version']}.", ""]
    for world, (spec, cast, attempt, profile) in results.items():
        rows = lineup(spec, cast)
        out += [f"## {world} (worlds: {', '.join(spec['worlds'])}; profile `{profile}`; attempt {attempt})", "",
                "### Cast", "",
                "| Key | Sex | Age | Stature | Build | Ancestry stratum | Skin (MST) | Hair | Outer |",
                "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
        for r in rows:
            age = r.get("age", r["age_band"])
            out.append(f"| {r['key']} | {r['sex']} | {age} | {r['stature_m']:.2f} | {r['build']} | "
                       f"{r['ancestry']} | {r['skin_mst']} | {r['hair']['colour']}, {r['hair']['category']} | "
                       f"{r['outer']} |")
        out += ["", "### Distinctness (§7.3: every pair differs in at least 3 of age band, stature ≥ 5 cm, "
                "build, skin ≥ 2 MST steps, hair, outer garment)", "",
                "| Pair | Differs in | Count | Verdict |", "| --- | --- | --- | --- |"]
        for i, a in enumerate(rows):
            for b in rows[i + 1:]:
                d = differences(a, b)
                out.append(f"| {a['key']} / {b['key']} | {', '.join(d)} | {len(d)} | "
                           f"{'PASS' if len(d) >= 3 else 'FAIL'} |")
        prof = load_profile(profile)["sampler"]
        out += ["", "### Realised distribution against the profile", "",
                "| Attribute | Stratum | Profile share | Expected of N | Realised |", "| --- | --- | --- | --- | --- |"]
        n = len(rows)
        tallies = [("sex", {"female": prof["sex"]["female"], "male": prof["sex"]["male"]}, "sex"),
                   ("age band", prof["age_band"], "age_band"),
                   ("ancestry", prof["ancestry"], "ancestry")]
        for label, shares, attr in tallies:
            for k, v in shares.items():
                real = sum(1 for r in rows if r[attr] == k)
                out.append(f"| {label} | {k} | {v:.3f} | {v * n:.2f} | {real} |")
        for sex in ("female", "male"):
            group = [r for r in rows if r["sex"] == sex]
            for k, v in prof["build"][sex].items():
                real = sum(1 for r in group if r["build"] == k)
                out.append(f"| build ({sex}, N={len(group)}) | {k} | {v:.3f} | {v * len(group):.2f} | {real} |")
        out.append("")
    return "\n".join(out + FINDINGS)


def main() -> int:
    check = "--check" in sys.argv
    inputs = json.loads((CAST / "casting_inputs.json").read_text())
    template = CHARS / "style" / "template.yaml"
    pre_full = read_block(template, "full_body", "preamble")
    pre_port = read_block(template, "portrait", "preamble")
    files: dict[Path, str] = {}
    results = {}
    for world, spec in inputs["worlds"].items():
        people = [p for p in inputs["people"] if p["world"] == world]
        for attempt in range(200):
            cast = cast_world(world, spec, people, inputs, attempt)
            if not failing_pairs(lineup(spec, cast)):
                break
        else:
            sys.exit(f"cast_director: {world}: no attempt under 200 satisfies §7.3")
        profile = spec["profile"]
        results[world] = (spec, cast, attempt, profile)
        for c in cast:
            d = CAST / c["person"]["key"]
            files[d / "brief.yaml"] = brief_yaml(c, world, spec["worlds"], inputs, attempt, profile)
            files[d / "reference_prompt.txt"] = pre_full + "\n" + person_paragraph(c, portrait=False)
            files[d / "portrait_prompt.txt"] = pre_port + "\n" + person_paragraph(c, portrait=True)
    files[CAST / "CASTING.md"] = casting_md(results, inputs)
    drift = []
    for path, text in sorted(files.items()):
        if check:
            if not path.exists() or path.read_text() != text:
                drift.append(str(path.relative_to(ROOT)))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    if drift:
        print("cast_director --check: differs from a fresh run:\n  " + "\n  ".join(drift))
        return 1
    print(f"cast_director: {'checked' if check else 'wrote'} {len(files)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())

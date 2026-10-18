"""Seeded, stratified casting (step-26 §7.4a items 4-6) — the minimal deterministic version.

Generic: the tool knows no attribute, no population and no cast. Everything comes from a casting config
(`--config`, e.g. presentation/mineworld-default/3D/characters/casting.yaml), which names

  * the seed, the style template, the casting inputs and the output directory;
  * per world, the population profile — whose `attributes:` define every trait, its kind (categorical,
    integer_range, normal), its dependencies (`given`), whether it is stratified over the cast, and its
    prompt words (`labels`); the schema is documented at the top of the default profile;
  * the distinctness categories (§7.3) and the CASTING.md table columns.

The template supplies the prompt preambles and the person paragraphs with {attr} / {attr|label.key}
placeholders. The inputs file gives, per person, the attributes the biography fixes or restricts and the
director's fields (outfit and prompt-ready lines). Never reads a surname.

Determinism: every draw comes from random.Random(sha256(seed | attempt | world | attribute | key)); strata
are allocated by largest remainder over the world's whole cast (fixed members included) within each `given`
group, then matched to people in a seeded order respecting `allowed`. If the cast fails distinctness, the
attempt counter rises and everything sampled is re-drawn; the attempt used is recorded. The same inputs
reproduce every file byte for byte (X-24); --check verifies that without writing.

    python3 tools/character_pipeline/casting/cast_director.py --config <casting.yaml> [--check]

Stdlib only; no network; no image or Meshy call. A missing or malformed profile, template, inputs file or
config fails by name.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class CastError(SystemExit):
    def __init__(self, msg: str):
        super().__init__(f"cast_director: {msg}")


# ---------------------------------------------------------------- a small YAML subset reader (no library)
# Supports block maps, flow maps and lists, quoted scalars, `|` block scalars and comments; enough for the
# config, profile and template files, whose headers state that they keep to this subset.


def _split_top(text: str) -> list[str]:
    parts, depth, cur, quote = [], 0, "", None
    for ch in text:
        if quote:
            quote = None if ch == quote else quote
        elif ch in "'\"":
            quote = ch
        elif ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
        if ch == "," and depth == 0 and not quote:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    return parts


def _scalar(text: str):
    text = text.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "'\"":
        return text[1:-1]
    if re.fullmatch(r"-?\d+", text):
        return int(text)
    if re.fullmatch(r"-?\d+\.\d*", text):
        return float(text)
    return {"true": True, "false": False}.get(text, text)


def _flow(text: str):
    text = text.strip()
    if text.startswith("[") and text.endswith("]"):
        return [_flow(p) for p in _split_top(text[1:-1])]
    if text.startswith("{") and text.endswith("}"):
        out = {}
        for p in _split_top(text[1:-1]):
            k, sep, v = p.partition(":")
            if not sep:
                raise ValueError(f"flow map entry without ':': {p.strip()!r}")
            out[str(_scalar(k))] = _flow(v)
        return out
    return _scalar(text)


def _strip_comment(line: str) -> str:
    quote = None
    for i, ch in enumerate(line):
        if quote:
            quote = None if ch == quote else quote
        elif ch in "'\"":
            quote = ch
        elif ch == "#" and (i == 0 or line[i - 1] in " \t"):
            return line[:i].rstrip()
    return line.rstrip()


def read_yaml(path: Path, what: str) -> dict:
    if not path.is_file():
        raise CastError(f"{what} not found: {path}")
    lines = path.read_text().splitlines()
    root: dict = {}
    stack: list[tuple[int, dict]] = [(-1, root)]
    last: tuple[dict, str, int] | None = None  # the last plain scalar, for multi-line continuation
    i = 0
    try:
        while i < len(lines):
            raw = lines[i]
            line = _strip_comment(raw)
            i += 1
            if not line.strip():
                continue
            indent = len(line) - len(line.lstrip())
            if line.strip().startswith("- "):
                last = None
                continue  # block lists are not used by these files; flow lists are
            if last and indent > last[2] and isinstance(last[0][last[1]], str):
                last[0][last[1]] += " " + line.strip()  # a plain scalar continued on the next line
                continue
            key, sep, value = line.strip().partition(":")
            if not sep:
                raise ValueError(f"line {i}: expected `key: value`")
            while stack[-1][0] >= indent:
                stack.pop()
            parent = stack[-1][1]
            value = value.strip()
            last = None
            if value in ("|", ">", ">-", "|-"):
                body, base = [], None
                while i < len(lines):
                    nxt = lines[i]
                    if nxt.strip() and len(nxt) - len(nxt.lstrip()) <= indent:
                        break
                    if nxt.strip() and base is None:
                        base = len(nxt) - len(nxt.lstrip())
                    body.append(nxt[base:] if base is not None and nxt.strip() else "")
                    i += 1
                text = "\n".join(body).rstrip() + "\n"
                parent[key] = text if value.startswith("|") else " ".join(text.split())
            elif value:
                parent[key] = _flow(value)
                last = (parent, key, indent)
            else:
                child: dict = {}
                parent[key] = child
                stack.append((indent, child))
    except ValueError as e:
        raise CastError(f"{what} {path} is malformed: {e}") from None
    return root


def rel(path: str) -> Path:
    return ROOT / path


# ---------------------------------------------------------------- profile

KINDS = ("categorical", "integer_range", "normal")


def load_profile(path: Path) -> dict:
    data = read_yaml(path, "population profile")
    if "profile_of" in data:
        return load_profile(path.parent / data["profile_of"]) | {
            "_path": str(path.relative_to(ROOT)),
            "_name": data.get("profile", path.stem),
        }
    attrs = data.get("attributes")
    if not isinstance(attrs, dict) or not attrs:
        raise CastError(f"population profile {path} has no `attributes:` map")
    for name, spec in attrs.items():
        if not isinstance(spec, dict) or spec.get("kind") not in KINDS:
            raise CastError(f"population profile {path}: attribute `{name}` needs `kind:` one of {KINDS}")
        for g in given_of(spec):
            if g not in attrs or list(attrs).index(g) > list(attrs).index(name):
                raise CastError(
                    f"population profile {path}: `{name}` is given `{g}`, which is not an earlier attribute"
                )
    return data | {"_path": str(path.relative_to(ROOT)), "_name": data.get("profile", path.stem)}


def given_of(spec: dict) -> list[str]:
    g = spec.get("given", [])
    return [g] if isinstance(g, str) else list(g)


def lookup(table, spec: dict, values: dict, attr: str):
    for g in given_of(spec):
        key = str(values[g])
        if not isinstance(table, dict) or key not in table:
            raise CastError(f"profile attribute `{attr}` has no entry for {g} = {key}")
        table = table[key]
    return table


def weights_of(spec: dict, values: dict, attr: str) -> dict:
    if "weights" in spec:
        w = lookup(spec["weights"], spec, values, attr)
    elif "options" in spec:
        w = {str(o): 1.0 for o in lookup(spec["options"], spec, values, attr)}
    else:
        raise CastError(f"categorical attribute `{attr}` needs `weights` or `options`")
    return {str(k): float(v) for k, v in w.items()}


def label(profile: dict, attr: str, value, key: str | None):
    labels = profile["attributes"].get(attr, {}).get("labels", {})
    entry = labels.get(str(value), "" if attr in profile["attributes"] else value)
    if key:
        return entry.get(key, "") if isinstance(entry, dict) else ""
    return entry


# ---------------------------------------------------------------- the sampler


def rng(*parts) -> random.Random:
    digest = hashlib.sha256("|".join(map(str, parts)).encode()).digest()
    return random.Random(int.from_bytes(digest[:8], "big"))


def largest_remainder(shares: dict[str, float], n: int) -> dict[str, int]:
    total = sum(shares.values())
    quotas = {k: v / total * n for k, v in shares.items()}
    counts = {k: int(q) for k, q in quotas.items()}
    for k in sorted(quotas, key=lambda k: (-(quotas[k] - counts[k]), k))[: n - sum(counts.values())]:
        counts[k] += 1
    return counts


def match(multiset: list[str], people: list[dict], allowed, r: random.Random) -> dict[str, str] | None:
    pool = sorted(multiset)
    r.shuffle(pool)
    order = sorted(people, key=lambda p: (len([v for v in set(pool) if v in allowed(p)]), p["key"]))
    result: dict[str, str] = {}

    def go(i: int, left: list[str]) -> bool:
        if i == len(order):
            return True
        tried = set()
        for j, v in enumerate(left):
            if v in tried or v not in allowed(order[i]):
                continue
            tried.add(v)
            result[order[i]["key"]] = v
            if go(i + 1, left[:j] + left[j + 1 :]):
                return True
        return False

    return result if go(0, pool) else None


def weighted(r: random.Random, w: dict[str, float], allowed: set | None) -> str:
    items = [(k, v) for k, v in w.items() if allowed is None or k in allowed]
    if not items:
        raise CastError(f"no value of {sorted(w)} is allowed")
    x, acc = r.random() * sum(v for _, v in items), 0.0
    for k, v in items:
        acc += v
        if x < acc:
            return k
    return items[-1][0]


def cast_world(
    world: str, profile: dict, fixed_members: list[dict], people: list[dict], seed: str, attempt: int
) -> list[dict]:
    attrs = profile["attributes"]
    values = {m["key"]: dict(m["values"]) for m in fixed_members}
    sources: dict[str, dict] = {p["key"]: {} for p in people}
    for p in people:
        values[p["key"]] = {}
        for a, f in p["fixed"].items():
            values[p["key"]][a] = f["value"]
            sources[p["key"]][a] = "biography: " + f["why"]
    everyone = [{"key": k} for k in values]

    def allowed_of(p_key: str, attr: str):
        p = next((q for q in people if q["key"] == p_key), None)
        a = p and p["allowed"].get(attr)
        return set(map(str, a["value"])) if a else None

    for attr, spec in attrs.items():
        free = [p for p in people if attr not in p["fixed"]]
        if spec["kind"] == "categorical" and spec.get("stratify"):
            groups: dict[tuple, list] = {}
            for m in everyone:
                groups.setdefault(tuple(str(values[m["key"]][g]) for g in given_of(spec)), []).append(m)
            for _gkey, group in sorted(groups.items()):
                shares = weights_of(spec, values[group[0]["key"]], attr)
                counts = largest_remainder(shares, len(group))
                for m in group:
                    if attr in values[m["key"]]:
                        counts[str(values[m["key"]][attr])] = counts.get(str(values[m["key"]][attr]), 0) - 1
                todo = [p for p in free if any(p["key"] == m["key"] for m in group)]
                multiset = [k for k, c in counts.items() for _ in range(max(c, 0))]
                got = match(
                    multiset,
                    todo,
                    lambda p, attr=attr, shares=shares: allowed_of(p["key"], attr) or set(shares),
                    rng(seed, attempt, world, attr),
                )
                if got is None:
                    raise CastError(f"{world}: no assignment of `{attr}` satisfies the biographies")
                for k, v in got.items():
                    values[k][attr] = v
                    why = people[[p["key"] for p in people].index(k)]["allowed"].get(attr)
                    sources[k][attr] = spec.get("source", "sampler") + (
                        f"; constrained by the biography ({why['why']})" if why else ""
                    )
            continue
        for p in free:
            k, v = p["key"], values[p["key"]]
            r = rng(seed, attempt, world, attr, k)
            if spec["kind"] == "categorical":
                val = weighted(r, weights_of(spec, v, attr), allowed_of(k, attr))
            elif spec["kind"] == "integer_range":
                lo, hi = lookup(spec["ranges"], spec, v, attr)
                val = r.randint(int(lo), int(hi))
            else:
                mean, p5, p95 = lookup(spec["params"], spec, v, attr)
                sd, clip = (p95 - p5) / 3.29, float(spec.get("clip_sd", 2))
                draw = min(max(r.gauss(mean, sd), mean - clip * sd), mean + clip * sd)
                val = round(draw * float(spec.get("scale", 1)), int(spec.get("round", 2)))
            rep = spec.get("replace")
            if rep and r.random() < float(rep["probability"].get(str(v[rep["given"]]), 0)):
                val = r.choice(rep["options"][str(v[rep["given"]])])
            v[attr] = val
            sources[k][attr] = spec.get("source", "sampler")
    return [
        {"key": p["key"], "values": values[p["key"]], "sources": sources[p["key"]], "person": p}
        for p in people
    ]


# ---------------------------------------------------------------- distinctness


def differences(a: dict, b: dict, cats: dict) -> list[str]:
    out = []
    for name, c in cats.items():
        if "min_diff" in c:
            if any(abs(float(a[x]) - float(b[x])) >= float(c["min_diff"]) - 1e-9 for x in c["attrs"]):
                out.append(name)
        elif any(str(a[x]) != str(b[x]) for x in c["attrs"]):
            out.append(name)
    return out


def lineup(fixed_members: list[dict], cast: list[dict]) -> list[dict]:
    return [{"key": m["key"]} | m["values"] for m in fixed_members] + [
        {"key": c["key"]} | c["values"] | c["person"]["fields"] for c in cast
    ]


def failing(rows: list[dict], dist: dict) -> list:
    cats, need = dist["categories"], int(dist["min_categories"])
    return [
        (a["key"], b["key"])
        for i, a in enumerate(rows)
        for b in rows[i + 1 :]
        if len(differences(a, b, cats)) < need
    ]


# ---------------------------------------------------------------- outputs


def fill(template: str, data: dict, profile: dict) -> str:
    def one(m: re.Match) -> str:
        path, _, lab = m.group(1).partition("|")
        if path not in data:
            raise CastError(f"template placeholder `{{{m.group(1)}}}` names no attribute or field")
        if lab:
            _, _, key = lab.partition(".")
            return str(label(profile, path, data[path], key or None))
        val = data[path]
        return f"{val:.2f}" if isinstance(val, float) else str(val)

    lines = []
    for line in template.splitlines():
        out = re.sub(r"\{([a-z_]+(?:\|label(?:\.[a-z_]+)?)?)\}", one, line)
        if out.strip():
            lines.append(out)
    return "\n".join(lines) + "\n"


def y(v) -> str:
    if isinstance(v, list):
        return "[" + ", ".join(y(x) for x in v) + "]"
    if isinstance(v, float):
        return f"{v:.2f}"
    s = str(v)
    return json.dumps(s, ensure_ascii=False) if re.search(r"[:#,\[\]{}()&*!|>'\"%@`]|^\s|\s$|^$", s) else s


def brief_yaml(c: dict, cfg: dict, world: str, attempt: int, profile: dict, tversion) -> str:
    p, f = c["person"], c["person"]["fields"]
    files = [f"worlds/{w}/people/{p['file']}" for w in cfg["worlds"][world]["worlds"]]
    lines = [
        "# Casting brief (step-26 §7.1), generated by tools/character_pipeline/casting/cast_director.py.",
        "# Do not edit by hand: change casting_inputs.json, the profile or casting.yaml and re-run.",
        "# `sources` records which attributes came from the biography and which from the sampler.",
        f"person: {p['key']}",
        f"from: {y(files)}",
        f"role: {y(f['role'])}",
    ]
    for attr in profile["attributes"]:
        lines.append(f"{attr}: {y(c['values'][attr])}")
    o = f["outfit"]
    lines.append(
        f"outfit: {{ outer: {y(f['outer'])}, under: {y(o['under'])}, legs: {y(o['legs'])}, "
        f"feet: {y(o['feet'])} }}"
    )
    if f.get("headwear"):
        lines.append(f"headwear: {y(f['headwear'])}")
    lines += [
        f"accessories: {y(f['accessories'])}",
        f"manner: {y(f['manner'])}",
        f"avoid: {y(f['avoid'])}   # §7.4",
        "sits: true                 # every cast member may sit (A-4 decor, E-2 extras): G-13 applies",
    ]
    if f.get("identity_source"):
        lines.append(f"identity_source: {y(f['identity_source'])}")
    lines += [
        "casting:",
        f"  world: {world}",
        f"  profile: {profile['_path']}",
        f"  template_version: {tversion}",
        f"  seed: {cfg['seed']}",
        f"  attempt: {attempt}",
        "  outfit_source: director (from role, tags and note; the setting's climate; §7.4 avoid list)",
        "sources:",
    ]
    lines += [f"  {k}: {y(v)}" for k, v in c["sources"].items()]
    return "\n".join(lines) + "\n"


def casting_md(results: dict, cfg: dict, notes: str) -> str:
    cols, dist = cfg["table"], cfg["distinctness"]
    out = [
        "# Casting — generated by `tools/character_pipeline/casting/cast_director.py`",
        "",
        f"Config `{cfg['_path']}`; seed `{cfg['seed']}`. Do not edit by hand. Specification:",
        "`.structured-coding/plans/mvp0/step-26-cast.md` §§7.1-7.4a. The profile schema is documented at the",
        "top of `populations/san-diego-county.yaml`.",
        "",
    ]
    for world, (cast, attempt, profile, fixed) in results.items():
        rows = lineup(fixed, cast)
        out += [
            f"## {world} (profile `{profile['_path']}`; attempt {attempt})",
            "",
            "### Cast",
            "",
            "| key | " + " | ".join(cols) + " |",
            "|" + " --- |" * (len(cols) + 1),
        ]
        for r in rows:
            out.append(f"| {r['key']} | " + " | ".join(y(r.get(c, "")).strip('"') for c in cols) + " |")
        out += [
            "",
            f"### Distinctness (§7.3: every pair differs in at least {dist['min_categories']} of "
            + ", ".join(dist["categories"])
            + ")",
            "",
            "| Pair | Differs in | Count | Verdict |",
            "| --- | --- | --- | --- |",
        ]
        for i, a in enumerate(rows):
            for b in rows[i + 1 :]:
                d = differences(a, b, dist["categories"])
                ok = len(d) >= int(dist["min_categories"])
                verdict = "PASS" if ok else "FAIL"
                out.append(f"| {a['key']} / {b['key']} | {', '.join(d)} | {len(d)} | {verdict} |")
        out += [
            "",
            "### Realised distribution against the profile (stratified attributes)",
            "",
            "| Attribute | Group | Value | Profile share | Expected | Realised |",
            "| --- | --- | --- | --- | --- | --- |",
        ]
        for attr, spec in profile["attributes"].items():
            if not spec.get("stratify"):
                continue
            groups: dict[tuple, list] = {}
            for r in rows:
                groups.setdefault(tuple(str(r[g]) for g in given_of(spec)), []).append(r)
            for gkey, group in sorted(groups.items()):
                for v, s in weights_of(spec, group[0], attr).items():
                    real = sum(1 for r in group if str(r[attr]) == v)
                    out.append(
                        f"| {attr} | {'/'.join(gkey) or 'all'} (N={len(group)}) | {v} | {s:.3f} | "
                        f"{s * len(group):.2f} | {real} |"
                    )
        out.append("")
    return "\n".join(out) + "\n" + notes


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--config", required=True, type=Path, help="casting.yaml of a Presentation Pack")
    ap.add_argument("--check", action="store_true", help="verify the committed outputs; write nothing")
    args = ap.parse_args(argv)
    cfg_path = args.config.resolve()
    cfg = read_yaml(cfg_path, "casting config") | {"_path": str(cfg_path.relative_to(ROOT))}
    for k in ("seed", "template", "inputs", "output_dir", "worlds", "distinctness", "table"):
        if k not in cfg:
            raise CastError(f"casting config {cfg_path} lacks `{k}`")
    template = read_yaml(rel(cfg["template"]), "style template")
    for k in ("full_body", "portrait", "person_full", "person_portrait"):
        if k not in template:
            raise CastError(f"style template {cfg['template']} lacks `{k}`")
    if not rel(cfg["inputs"]).is_file():
        raise CastError(f"casting inputs not found: {cfg['inputs']}")
    try:
        inputs = json.loads(rel(cfg["inputs"]).read_text())
    except json.JSONDecodeError as e:
        raise CastError(f"casting inputs {cfg['inputs']} are malformed: {e}") from None
    if str(inputs.get("template_version")) != str(template.get("version")):
        raise CastError(
            f"casting inputs name template v{inputs.get('template_version')}, "
            f"the template is v{template.get('version')} (X-22)"
        )
    out_dir = rel(cfg["output_dir"])
    files: dict[Path, str] = {}
    results = {}
    for world, wspec in cfg["worlds"].items():
        profile = load_profile(rel(wspec["profile"]))
        people = [p for p in inputs["people"] if p["world"] == world]
        fixed = inputs.get("fixed_members", {}).get(world, [])
        for attempt in range(int(cfg.get("max_attempts", 200))):
            cast = cast_world(world, profile, fixed, people, cfg["seed"], attempt)
            if not failing(lineup(fixed, cast), cfg["distinctness"]):
                break
        else:
            raise CastError(f"{world}: no attempt satisfies the distinctness rule")
        results[world] = (cast, attempt, profile, fixed)
        for c in cast:
            data = c["values"] | c["person"]["fields"]
            d = out_dir / c["key"]
            files[d / "brief.yaml"] = brief_yaml(c, cfg, world, attempt, profile, template["version"])
            files[d / "reference_prompt.txt"] = (
                template["full_body"]["preamble"] + "\n" + fill(template["person_full"], data, profile)
            )
            files[d / "portrait_prompt.txt"] = (
                template["portrait"]["preamble"] + "\n" + fill(template["person_portrait"], data, profile)
            )
    notes_path = out_dir / "CASTING_NOTES.md"
    notes = notes_path.read_text() if notes_path.is_file() else ""
    files[out_dir / "CASTING.md"] = casting_md(results, cfg, notes)
    drift = []
    for path, text in sorted(files.items()):
        if args.check:
            if not path.is_file() or path.read_text() != text:
                drift.append(str(path.relative_to(ROOT)))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    if drift:
        print("cast_director --check: differs from a fresh run:\n  " + "\n  ".join(drift))
        return 1
    print(f"cast_director: {'checked' if args.check else 'wrote'} {len(files)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())

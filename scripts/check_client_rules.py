#!/usr/bin/env python3
"""The executable half of "no rule in the client" for the 2D reference client (ARC-47).

Three checks, each reporting by file and line and exiting non-zero on any finding:

  python3 scripts/check_client_rules.py                  R1-R6 over clients/2d/**/*.gd
  python3 scripts/check_client_rules.py --scope BASE     every path the branch changed since its
                                                         merge base with BASE is in the 2D client's
                                                         allowed set (step-13 I-4, AC-W9, AC-I10)
  python3 scripts/check_client_rules.py --check-pack DIR a Presentation Pack's two files are
                                                         well formed, every file they name exists,
                                                         and its i18n/*.po wording parses
                                                         (clients/2d/PRESENTATION.md §7)

The rules, on code with comments removed:

  R1  only scripts/intents.gd calls submit( or submit_affordance(
  R2  no .gd file but intents.gd holds a string literal equal to an action type that intents.gd
      composes (its `const COMPOSED := [...]`, read from the file; absent is a failure), except a
      (file, literal) pair ADMITTED names with its reason; an admission that admits nothing fails
  R3  in intents.gd, menu.gd and hud/**, no function that submits or calls `intents.` reads may(,
      "available", unavailable_reason or requirement(  (R3', step-13 13b)
  R4  distance_to(, .length() and the word `within` appear only in walker.gd, projection.gd,
      town.gd, scripts/scene/** and the test harness scripts/harness/**
  R5  only scripts/link.gd constructs MineWorldClient or calls connect_to_world(
  R6  no .gd file but hud/words.gd and the harness passes a literal of two or more words to
      `text =`, add_item(, note( or toast( — user-visible text is a wording key (ARC-70); the
      harness prints evidence, not UI

  R7  (S35 N-e, I-Ne-1) over clients/2d/**, clients/3d-spike/scripts/** and
      clients/protocol/mineworld/needs.gd: (a) no code names a need or a need word (NEED_VOCABULARY);
      (b) needs.gd compares nothing (no <, >, <=, >=; the `->` of a return type is not a comparison).
      A client shows the words the packs disclose; it never names a need or compares a level.

  R8  (PR 13e, ARC-343) no .gd under clients/2d/scripts/ holds a string literal equal to an event
      type, or a payload field, that the default pack's renderer/godot.yaml `events.types` names,
      except a (file, literal) pair R8_ADMITTED names with its reason; an admission that admits
      nothing fails. Which facts are shown, and how, is the pack's.

Symlinked directories are not walked (clients/2d/mineworld is the shared module), so the verdict does
not depend on the Python version's rglob. Standard library only. Fail-closed: a missing directory, git
failure or unknown base is a failure.

  python3 scripts/check_client_rules.py --self-test      R7's own cases: planted violations are found,
                                                         clean lines and comments are not
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLIENT = ROOT / "clients" / "2d"
SCRIPTS = CLIENT / "scripts"
INTENTS = SCRIPTS / "intents.gd"

DISTANCE_ALLOWED = ("walker.gd", "projection.gd", "town.gd")
DISTANCE_ALLOWED_DIRS = ("scene", "harness")

# R2's admissions: a literal equal to a composed action type, written outside intents.gd, for a reason
# that is not naming the action. Each must admit something.
ADMITTED = {
    ("scripts/app.gd", "invite"): "the --invite command-line option, the join credential handed to "
    "link.gd (PROTOCOL.md §4.1); it shares its spelling with group-activity's action type",
    ("scripts/harness/interact.gd", "sleep"): "the scripted check's step that waits a number of seconds "
    "(`{\"sleep\": seconds}`, step-13 §15.3); it is not the action, which the harness only chooses from the menu",
}

# R3' extends R3 to these: the menu and the HUD, where a verdict is shown and an entry chosen.
VERDICT_SCOPED_FILES = ("intents.gd", "menu.gd")
VERDICT_SCOPED_DIRS = ("hud",)
VERDICT = re.compile(r"\bmay\s*\(|\"available\"|\bunavailable_reason\b|\brequirement\s*\(")

# R6: where user-visible text is set, and the files allowed to hold sentences.
UI_SINK = re.compile(r"\.text\s*=(?!=)|\badd_item\s*\(|\bnote\s*\(|\btoast\s*\(")
WORDING_FILES = ("hud/words.gd",)
WORDING_DIRS = ("harness",)

SCOPE_PREFIXES = (
    "clients/2d/",
    "presentation/mineworld-default/2D/",
    "presentation/mineworld-default/LICENSES/",
    "tools/cli/tests/godot2d/",
    # PR 13e (I-13e-4): its test, the shared module's M-8, and the module's adoption notes.
    "tools/cli/tests/client_2d_many.rs",
    "clients/protocol/mineworld/world_client.gd",
    "clients/protocol/ADOPTION.md",
)
SCOPE_FILES = (
    "mineworld-2d",
    "scripts/check_client_rules.py",
    "tools/cli/tests/client_2d.rs",
    "tools/cli/tests/client_2d_interact.rs",
    "tools/cli/tests/client_2d_interact_stub.rs",
    # S14's structural scan (merged after 13a froze) admits each client's action literals by entry;
    # 13a adds its two (step-13 §14.10 D-14), 13b its six composed types (QS13b-7). No other line of
    # that file is a client PR's.
    "tests/acceptance/tests/client_rules.rs",
)


def strip_comment(line: str) -> str:
    """The line without its comment: a `#` outside a string literal starts one."""
    quote = ""
    i = 0
    while i < len(line):
        c = line[i]
        if quote:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                quote = ""
        elif c in "\"'":
            quote = c
        elif c == "#":
            return line[:i]
        i += 1
    return line


def code_lines(path: Path) -> list[tuple[int, str]]:
    text = path.read_text(encoding="utf-8")
    return [(n, strip_comment(line)) for n, line in enumerate(text.splitlines(), start=1)]


def literals(code: str) -> list[str]:
    return [a or b for a, b in re.findall(r'"((?:[^"\\]|\\.)*)"|\'((?:[^\'\\]|\\.)*)\'', code)]


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def composed_types(findings: list[str]) -> list[str]:
    if not INTENTS.exists():
        findings.append(f"{rel(INTENTS)}: missing — the one file allowed to submit must exist")
        return []
    for n, code in code_lines(INTENTS):
        m = re.match(r"\s*const\s+COMPOSED\s*(?::=|=|:\s*Array\s*=)\s*\[(.*)\]", code)
        if m:
            return literals(m.group(1))
    findings.append(f"{rel(INTENTS)}: no `const COMPOSED := [...]` naming what it composes")
    return []


def functions(lines: list[tuple[int, str]]) -> list[list[tuple[int, str]]]:
    """Top-level functions, each as its lines (a function ends at the next unindented line)."""
    out: list[list[tuple[int, str]]] = []
    current: list[tuple[int, str]] | None = None
    for n, code in lines:
        if re.match(r"(static\s+)?func\s", code):
            current = [(n, code)]
            out.append(current)
        elif current is not None and code.strip() and not code[0].isspace():
            current = None
        elif current is not None:
            current.append((n, code))
    return out


def scripts_of_client() -> list[Path]:
    """Every .gd under the client, not descending a symlinked directory or `.godot`."""
    found: list[Path] = []
    for directory, subdirectories, files in os.walk(CLIENT, followlinks=False):
        subdirectories[:] = sorted(
            d for d in subdirectories
            if d != ".godot" and not (Path(directory) / d).is_symlink()
        )
        found.extend(Path(directory) / f for f in sorted(files) if f.endswith(".gd"))
    return found


def words(literal: str) -> int:
    """Whitespace-separated words with at least two letters: a sentence has two or more; a key
    (`ui.result.unknown`) and a format (`%s  ·  %s`) have none or one."""
    return sum(1 for token in literal.split() if re.search(r"[A-Za-z]{2,}", token))


def check_rules() -> list[str]:
    findings: list[str] = []
    if not SCRIPTS.is_dir():
        return [f"{rel(SCRIPTS)}: missing"]
    composed = composed_types(findings)
    admitted_used = {key: False for key in ADMITTED}
    for path in scripts_of_client():
        name = rel(path)
        lines = code_lines(path)
        is_intents = path == INTENTS
        in_scripts = SCRIPTS in path.parents
        sub = path.relative_to(SCRIPTS).parts if in_scripts else ()
        local = "/".join(sub)
        distance_ok = in_scripts and (
            (len(sub) == 1 and sub[0] in DISTANCE_ALLOWED)
            or (len(sub) > 1 and sub[0] in DISTANCE_ALLOWED_DIRS)
        )
        verdict_scoped = in_scripts and (
            (len(sub) == 1 and sub[0] in VERDICT_SCOPED_FILES)
            or (len(sub) > 1 and sub[0] in VERDICT_SCOPED_DIRS)
        )
        wording_ok = in_scripts and (local in WORDING_FILES or (len(sub) > 1 and sub[0] in WORDING_DIRS))
        is_link = path == SCRIPTS / "link.gd"
        for n, code in lines:
            if not is_intents and re.search(r"\bsubmit(_affordance)?\s*\(", code):
                findings.append(f"{name}:{n}: R1 submits — only scripts/intents.gd may")
            if not is_intents:
                for lit in literals(code):
                    if lit not in composed:
                        continue
                    key = (f"scripts/{local}", lit)
                    if key in ADMITTED:
                        admitted_used[key] = True
                    else:
                        findings.append(
                            f"{name}:{n}: R2 names the action type \"{lit}\" — only intents.gd may"
                        )
            if not distance_ok and re.search(
                r"\bdistance_to\s*\(|\.length\s*\(\s*\)|\bwithin\b", code
            ):
                findings.append(
                    f"{name}:{n}: R4 computes a distance — only walker, projection, town, scene/ may"
                )
            if not is_link and re.search(r"\bMineWorldClient\.new\s*\(|\bconnect_to_world\s*\(", code):
                findings.append(f"{name}:{n}: R5 opens a connection — only scripts/link.gd may")
            if not wording_ok and UI_SINK.search(code):
                for lit in literals(code):
                    if words(lit) >= 2:
                        findings.append(
                            f"{name}:{n}: R6 shows the sentence \"{lit}\" — UI text is a wording key "
                            "(ARC-70)"
                        )
        if verdict_scoped:
            for fn in functions(lines):
                body = "\n".join(code for _, code in fn)
                if not re.search(r"\bsubmit(_affordance)?\s*\(|\bintents\.", body):
                    continue
                for n, code in fn:
                    if VERDICT.search(code):
                        findings.append(
                            f"{name}:{n}: R3 reads the server's verdict in a function that submits "
                            "or chooses"
                        )
    for (file, literal), used in admitted_used.items():
        if not used:
            findings.append(f"clients/2d/{file}: R2 admits \"{literal}\", which it no longer holds; "
                            "remove the admission")
    findings.extend(check_r7())
    findings.extend(check_r8())
    return findings


# R8 (PR 13e, ARC-343): the facts a client shows are named by the pack's `events` table, not by code.
DEFAULT_RENDERER = ROOT / "presentation" / "mineworld-default" / "2D" / "renderer" / "godot.yaml"
R8_ADMITTED = {
    ("scripts/intents.gd", "utterance"): "the talk request's payload field, composed by the only "
    "submitter from the typed line (step-13 §15.4 D-b-5); it shares its spelling with the field "
    "conversation's `spoke` fact carries, and no fact is read through it",
    ("scripts/hud/readers.gd", "utterance"): "the conversation-history component's row field, read for "
    "the history panel (13b, a component the world discloses, not a fact): the panel's wording argument "
    "and the component field share the spelling",
    ("scripts/hud/readers.gd", "speaker"): "the conversation-history component's row field naming who "
    "spoke, read for the history panel (13b, a component, not a fact)",
}


def event_vocabulary(renderer: dict) -> set[str]:
    """Every event type and payload field a renderer's `events.types` table names."""
    types = renderer.get("events", {}).get("types", {})
    names: set[str] = set()
    if isinstance(types, dict):
        for event_type, how in types.items():
            names.add(event_type)
            for field in ("text", "who"):
                if isinstance(how, dict) and isinstance(how.get(field), str):
                    names.add(how[field])
    return names


def r8(name: str, lines: list[tuple[int, str]], vocabulary: set[str], admitted: dict) -> list[str]:
    """R8 on one file's code lines (comments removed); `admitted` maps a literal to whether it was used."""
    found: list[str] = []
    for n, code in lines:
        for lit in literals(code):
            if lit not in vocabulary:
                continue
            if lit in admitted:
                admitted[lit] = True
            else:
                found.append(f"{name}:{n}: R8 names the event vocabulary \"{lit}\" — which facts are "
                             "shown is the pack's events table (ARC-343)")
    return found


def check_r8() -> list[str]:
    try:
        renderer = json.loads(DEFAULT_RENDERER.read_text(encoding="utf-8"))
    except (OSError, ValueError) as failure:
        return [f"{rel(DEFAULT_RENDERER)}: R8 cannot read the default pack's events table: {failure}"]
    vocabulary = event_vocabulary(renderer)
    if not vocabulary:
        return [f"{rel(DEFAULT_RENDERER)}: R8 found no events.types — the scan would be empty"]
    findings: list[str] = []
    used = {key: False for key in R8_ADMITTED}
    for path in scripts_of_client():
        if SCRIPTS not in path.parents:
            continue
        local = "scripts/" + "/".join(path.relative_to(SCRIPTS).parts)
        admitted = {lit: used[(file, lit)] for (file, lit) in R8_ADMITTED if file == local}
        findings.extend(r8(rel(path), code_lines(path), vocabulary, admitted))
        for lit, was in admitted.items():
            used[(local, lit)] = used[(local, lit)] or was
    for (file, literal), was in used.items():
        if not was:
            findings.append(f"clients/2d/{file}: R8 admits \"{literal}\", which it no longer holds; "
                            "remove the admission")
    return findings


# R7 (S35 N-e, I-Ne-1): the need vocabulary is the packs' to disclose, never a client's to name.
NEED_VOCABULARY = ("hunger", "tiredness", "peckish", "hungry", "famished", "sleepy", "tired", "exhausted")
NEEDS_FILE = ROOT / "clients" / "protocol" / "mineworld" / "needs.gd"
THREE_D_SCRIPTS = ROOT / "clients" / "3d-spike" / "scripts"
TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
COMPARISON = re.compile(r"<|>")
STRING_LITERAL = re.compile(r'"(?:[^"\\]|\\.)*"')


def r7(name: str, lines: list[tuple[int, str]], holds_levels: bool) -> list[str]:
    """R7 on one file's code lines (comments removed). `holds_levels` applies (b): no comparison."""
    found: list[str] = []
    for n, code in lines:
        for token in TOKEN.findall(code):
            if token in NEED_VOCABULARY:
                found.append(f"{name}:{n}: R7 names the need or its word \"{token}\" — a client shows the "
                             "words a pack discloses (I-Ne-1)")
        if holds_levels:
            bare = STRING_LITERAL.sub('""', code).replace("->", "")
            if COMPARISON.search(bare):
                found.append(f"{name}:{n}: R7 compares a level — the reader shows the disclosed number and "
                             "compares nothing (I-Ne-1)")
    return found


def gd_files(directory: Path) -> list[Path]:
    """Every .gd under `directory`, not descending a symlinked directory or `.godot`."""
    found: list[Path] = []
    for current, subdirectories, files in os.walk(directory, followlinks=False):
        subdirectories[:] = sorted(
            d for d in subdirectories
            if d != ".godot" and not (Path(current) / d).is_symlink()
        )
        found.extend(Path(current) / f for f in sorted(files) if f.endswith(".gd"))
    return found


def check_r7() -> list[str]:
    findings: list[str] = []
    if not NEEDS_FILE.is_file():
        return [f"{rel(NEEDS_FILE)}: missing — the shared reader R7 covers must exist"]
    scanned = scripts_of_client() + gd_files(THREE_D_SCRIPTS) + [NEEDS_FILE]
    for path in scanned:
        findings.extend(r7(rel(path), code_lines(path), holds_levels=path == NEEDS_FILE))
    return findings


def self_test() -> list[str]:
    """R7's own cases (the planted violations it must find, and the clean code it must pass)."""
    cases = [
        ("a comparison planted in needs.gd is found", "if level >= 600:", True, True),
        ("a comparison in a return type is not", "static func own_needs(obs) -> Array:", True, False),
        ("a need word planted in a 2D script is found", 'var word := "hungry"', False, True),
        ("a need name planted as an identifier is found", "var hunger := 3", False, True),
        ("a need word in a comment is not", "var count := 3  # hungry", False, False),
        ("a comparison in a 2D script is not R7's (only needs.gd holds levels)", "if n > 2:", False, False),
        ("a string holding a comparison in needs.gd is not", 'var sign := "a > b"', True, False),
    ]
    failures: list[str] = []
    for claim, text, holds, expect in cases:
        found = r7("self-test", [(1, strip_comment(text))], holds_levels=holds)
        if bool(found) != expect:
            failures.append(f"R7 self-test FAIL: {claim}: {found}")
        else:
            print(f"R7 self-test PASS: {claim}")
    vocabulary = event_vocabulary({"events": {"types": {"spoke": {"show": "bubble", "text": "utterance"},
                                                        "group-activity-started": {"show": "notice"}}}})
    r8_cases = [
        ("an event type planted in a script is found (AD-13e-1)", 'if t == "spoke":', {}, True),
        ("a payload field the table names is found", 'var s = p["utterance"]', {}, True),
        ("an admitted literal is not", 'var s = p["utterance"]', {"utterance": False}, False),
        ("an event type in a comment is not", "var n := 1  # spoke", {}, False),
        ("a key built from data is not", 'var key := "ui.event." + type', {}, False),
    ]
    for claim, text, admitted, expect in r8_cases:
        found = r8("self-test", [(1, strip_comment(text))], vocabulary, admitted)
        if bool(found) != expect:
            failures.append(f"R8 self-test FAIL: {claim}: {found}")
        else:
            print(f"R8 self-test PASS: {claim}")
    pack_cases = [
        ("a bubble without its text field is found", {"spoke": {"show": "bubble"}}, True),
        ("an unknown way to show is found", {"spoke": {"show": "shout"}}, True),
        ("a well-formed table passes", {"spoke": {"show": "bubble", "text": "utterance"}}, False),
    ]
    for claim, types, expect in pack_cases:
        found = check_events({"events": {"bubble_seconds": 6, "notice_seconds": 5, "types": types}})
        if bool(found) != expect:
            failures.append(f"events self-test FAIL: {claim}: {found}")
        else:
            print(f"events self-test PASS: {claim}")
    return failures


def git(*args: str) -> str:
    done = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {done.stderr.strip()}")
    return done.stdout


def check_scope(base: str) -> list[str]:
    try:
        merge_base = git("merge-base", base, "HEAD").strip()
        changed = set(git("diff", "--name-only", "--no-renames", merge_base).split())
        changed |= set(git("ls-files", "--others", "--exclude-standard").split())
    except RuntimeError as failure:
        return [f"scope: {failure}"]
    findings = []
    for path in sorted(changed):
        if path.endswith(".md"):
            continue
        if path in SCOPE_FILES or path.startswith(SCOPE_PREFIXES):
            continue
        findings.append(f"{path}: outside the 2D client's allowed paths (I-4)")
    return findings


def check_pack(directory: str) -> list[str]:
    pack = (ROOT / directory) if not Path(directory).is_absolute() else Path(directory)
    findings: list[str] = []
    docs = {}
    for name in ("assets/asset_bindings.yaml", "renderer/godot.yaml"):
        path = pack / name
        try:
            docs[name] = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError) as failure:
            findings.append(f"{path}: not readable as YAML's JSON subset: {failure}")
    if findings:
        return findings
    bindings = docs["assets/asset_bindings.yaml"]
    renderer = docs["renderer/godot.yaml"]
    sprites = bindings.get("sprites", {})
    sets = bindings.get("sets", {})
    named: set[str] = set()
    for sid, sprite in sprites.items():
        file = sprite.get("file", "")
        named.add(file)
        if not (pack / file).is_file():
            findings.append(f"asset_bindings.yaml: sprite {sid}: {file} does not exist")
    if bindings.get("default_set") not in sets:
        findings.append(f"asset_bindings.yaml: default_set {bindings.get('default_set')!r} is not a set")

    def chain(name: str) -> dict:
        roles: dict = {}
        seen = set()
        while name:
            if name in seen or name not in sets:
                findings.append(f"asset_bindings.yaml: set {name!r} is missing or extends in a cycle")
                return roles
            seen.add(name)
            roles = {**sets[name].get("roles", {}), **roles}
            name = sets[name].get("extends", "")
        return roles

    def directional(target: str) -> bool:
        return target in sprites or f"{target}_front" in sprites

    resolved = {}
    for set_name in sets:
        roles = chain(set_name)
        resolved[set_name] = roles
        for role, target in roles.items():
            if not directional(target):
                findings.append(f"asset_bindings.yaml: set {set_name}: {role} -> {target} is no sprite")
    asked = set(re.findall(r'"((?:prop|texture):[a-z_]+)"', json.dumps(renderer)))
    default = resolved.get(bindings.get("default_set"), {})
    for role in sorted(asked):
        if role not in default:
            findings.append(f"renderer/godot.yaml names {role}, which the default set does not bind")
    for key in ("shader",):
        for effect in renderer.get("effects", {}).values():
            if key in effect and not (pack / effect[key]).is_file():
                findings.append(f"renderer/godot.yaml: {effect[key]} does not exist")
    for path in sorted(pack.rglob("*")):
        if path.suffix == ".import":
            findings.append(f"{path}: a Godot import sidecar has no place in a runtime pack")
    art = pack / "art"
    if art.is_dir():
        for path in sorted(art.rglob("*")):
            r = str(path.relative_to(pack))
            if path.is_file() and r not in named and not r.startswith("art/provenance/") and path.suffix != ".md":
                findings.append(f"{r}: carried but bound by no sprite")
    wording = pack / "i18n"
    if wording.is_dir():
        for path in sorted(wording.glob("*.po")):
            findings.extend(check_po(path, path.relative_to(pack).as_posix()))
    findings.extend(check_daylight(pack / "daylight.json"))
    findings.extend(check_events(renderer))
    return findings


def check_events(renderer: dict) -> list[str]:
    """PR 13e's `events` table (PRESENTATION.md §4.1): optional; when present, positive durations and
    each type shown as `bubble` (naming its `text` field) or `notice`."""
    events = renderer.get("events")
    if events is None:
        return []
    if not isinstance(events, dict) or not isinstance(events.get("types", {}), dict):
        return ["renderer/godot.yaml: events must be an object whose types is an object"]
    findings = []
    for key in ("bubble_seconds", "notice_seconds"):
        value = events.get(key, 1)
        if not isinstance(value, (int, float)) or value <= 0:
            findings.append(f"renderer/godot.yaml: events.{key} must be a positive number")
    for event_type, how in events.get("types", {}).items():
        show = how.get("show") if isinstance(how, dict) else None
        if show not in ("bubble", "notice"):
            findings.append(f"renderer/godot.yaml: events.types.{event_type}: show must be bubble or notice")
        elif show == "bubble" and not (isinstance(how.get("text"), str) and how["text"]):
            findings.append(f"renderer/godot.yaml: events.types.{event_type}: a bubble names its text field")
        elif "who" in how and not (isinstance(how["who"], str) and how["who"]):
            findings.append(f"renderer/godot.yaml: events.types.{event_type}: who names a payload field")
    return findings


def check_daylight(path: Path) -> list[str]:
    """TW-e's tint table (`clients/shared/WORLD_TIME.md` §6): optional; when present, ascending
    elevations, as many tints, each a `#rrggbb` colour."""
    if not path.is_file():
        return []
    try:
        table = json.loads(path.read_text(encoding="utf-8"))
        elevations = [float(e) for e in table["elevation_deg"]]
        tints = list(table["tint"])
    except (OSError, ValueError, KeyError, TypeError) as failure:
        return [f"daylight.json: not a daylight table: {failure}"]
    findings = []
    if not elevations or len(elevations) != len(tints):
        findings.append("daylight.json: elevation_deg and tint must be non-empty and of equal length")
    if any(b <= a for a, b in zip(elevations, elevations[1:])):
        findings.append("daylight.json: elevation_deg must ascend")
    for tint in tints:
        if not (isinstance(tint, str) and re.fullmatch(r"#[0-9a-fA-F]{6}", tint)):
            findings.append(f"daylight.json: {tint!r} is not a #rrggbb colour")
    return findings


PO_STRING = re.compile(r'"((?:[^"\\]|\\.)*)"\s*$')
ACTION_KEY = re.compile(r"action\.[a-z0-9]+(?:-[a-z0-9]+)*(?:\.done)?")
REASON_KEY = re.compile(r"reason\.[a-z0-9]+(?:[_-][a-z0-9]+)*")


def check_po(path: Path, name: str) -> list[str]:
    """A gettext file the client loads as wording (PRESENTATION.md §7): msgid/msgstr pairs of quoted
    strings, UTF-8, a header naming the `Language:`, no key twice, well-formed action/reason keys."""
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as failure:
        return [f"{name}: not readable as UTF-8: {failure}"]
    findings: list[str] = []
    entries: list[tuple[int, str, str]] = []  # (line of msgid, msgid, msgstr)
    keyword = ""
    msgid: tuple[int, str] | None = None
    msgstr: str | None = None

    def close(at: int) -> None:
        nonlocal msgid, msgstr
        if msgid is not None:
            if msgstr is None:
                findings.append(f"{name}:{msgid[0]}: msgid \"{msgid[1]}\" has no msgstr")
            else:
                entries.append((msgid[0], msgid[1], msgstr))
        msgid, msgstr = None, None

    for n, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        m = re.match(r"(msgid|msgstr)\s+(.*)$", line)
        quoted = m.group(2) if m else line
        if not quoted.startswith('"'):
            findings.append(f"{name}:{n}: not a gettext line: {line}")
            continue
        s = PO_STRING.fullmatch(quoted)
        if s is None:
            findings.append(f"{name}:{n}: unterminated string: {line}")
            continue
        value = s.group(1)
        if m and m.group(1) == "msgid":
            close(n)
            msgid, keyword = (n, value), "msgid"
        elif m:
            if msgid is None or msgstr is not None:
                findings.append(f"{name}:{n}: msgstr without its msgid")
                continue
            msgstr, keyword = value, "msgstr"
        elif keyword == "msgid" and msgid is not None and msgstr is None:
            msgid = (msgid[0], msgid[1] + value)
        elif keyword == "msgstr" and msgstr is not None:
            msgstr += value
        else:
            findings.append(f"{name}:{n}: a continuation string with nothing to continue")
    close(0)
    header = [e for e in entries if e[1] == ""]
    if not header or not re.search(r"(^|\\n)Language: *[A-Za-z]", header[0][2]):
        findings.append(f"{name}: no header entry (msgid \"\") naming its Language:")
    seen: dict[str, int] = {}
    for n, key, _ in entries:
        if key in seen:
            findings.append(f"{name}:{n}: \"{key}\" again (first at line {seen[key]})")
        seen.setdefault(key, n)
        if key.startswith("action.") and not ACTION_KEY.fullmatch(key):
            findings.append(f"{name}:{n}: \"{key}\" is not action.<type> or action.<type>.done")
        if key.startswith("reason.") and not REASON_KEY.fullmatch(key):
            findings.append(f"{name}:{n}: \"{key}\" is not reason.<code>")
    return findings


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        findings, what = self_test(), "self-test"
    elif len(argv) >= 2 and argv[0] == "--scope":
        findings, what = check_scope(argv[1]), "scope"
    elif len(argv) >= 2 and argv[0] == "--check-pack":
        findings, what = check_pack(argv[1]), "pack"
    elif not argv:
        findings, what = check_rules(), "rules"
    else:
        print(__doc__)
        return 2
    for finding in findings:
        print(finding)
    print(f"check_client_rules {what}: {'FAIL' if findings else 'PASS'} ({len(findings)} finding(s))")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

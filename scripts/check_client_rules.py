#!/usr/bin/env python3
"""The executable half of "no rule in the client" for the 2D reference client (ARC-47).

Three checks, each reporting by file and line and exiting non-zero on any finding:

  python3 scripts/check_client_rules.py                  R1-R5 over clients/2d/**/*.gd
  python3 scripts/check_client_rules.py --scope BASE     every path the branch changed since its
                                                         merge base with BASE is in the 2D client's
                                                         allowed set (step-13 I-4, AC-W9)
  python3 scripts/check_client_rules.py --check-pack DIR a Presentation Pack's two files are
                                                         well formed and every file they name exists
                                                         (clients/2d/PRESENTATION.md)

The rules, on code with comments removed:

  R1  only scripts/intents.gd calls submit( or submit_affordance(
  R2  no .gd file but intents.gd holds a string literal equal to an action type that intents.gd
      composes (its `const COMPOSED := [...]`, read from the file; absent is a failure)
  R3  in intents.gd, no function that submits reads may(, "available", unavailable_reason or
      requirement(
  R4  distance_to(, .length() and the word `within` appear only in walker.gd, projection.gd,
      town.gd, scripts/scene/** and the test harness scripts/harness/**
  R5  only scripts/link.gd constructs MineWorldClient or calls connect_to_world(

Standard library only. Fail-closed: a missing directory, git failure or unknown base is a failure.
"""

from __future__ import annotations

import json
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

SCOPE_PREFIXES = (
    "clients/2d/",
    "presentation/mineworld-default/2D/",
    "presentation/mineworld-default/LICENSES/",
    "tools/cli/tests/client_2d/",
)
SCOPE_FILES = (
    "mineworld-2d",
    "scripts/check_client_rules.py",
    "tools/cli/tests/client_2d.rs",
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


def check_rules() -> list[str]:
    findings: list[str] = []
    if not SCRIPTS.is_dir():
        return [f"{rel(SCRIPTS)}: missing"]
    composed = composed_types(findings)
    for path in sorted(CLIENT.rglob("*.gd")):
        if ".godot" in path.parts:
            continue
        name = rel(path)
        lines = code_lines(path)
        is_intents = path == INTENTS
        in_scripts = SCRIPTS in path.parents
        sub = path.relative_to(SCRIPTS).parts if in_scripts else ()
        distance_ok = in_scripts and (
            (len(sub) == 1 and sub[0] in DISTANCE_ALLOWED)
            or (len(sub) > 1 and sub[0] in DISTANCE_ALLOWED_DIRS)
        )
        is_link = path == SCRIPTS / "link.gd"
        for n, code in lines:
            if not is_intents and re.search(r"\bsubmit(_affordance)?\s*\(", code):
                findings.append(f"{name}:{n}: R1 submits — only scripts/intents.gd may")
            if not is_intents:
                for lit in literals(code):
                    if lit in composed:
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
        if is_intents:
            for fn in functions(lines):
                body = "\n".join(code for _, code in fn)
                if not re.search(r"\bsubmit(_affordance)?\s*\(", body):
                    continue
                for n, code in fn:
                    if re.search(r"\bmay\s*\(|\"available\"|\bunavailable_reason\b|\brequirement\s*\(", code):
                        findings.append(
                            f"{name}:{n}: R3 reads the server's verdict in a function that submits"
                        )
    return findings


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
    return findings


def main(argv: list[str]) -> int:
    if len(argv) >= 2 and argv[0] == "--scope":
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

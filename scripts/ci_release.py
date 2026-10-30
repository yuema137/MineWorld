#!/usr/bin/env python3
"""Release-path checks: the publishing gates of `publish.yml` and the citation's version (S23 R-d).

Commands:
    python3 scripts/ci_release.py publish-gate --event E --tag T --prerelease P
                                                      decide whether a run publishes (key=value on stdout)
    python3 scripts/ci_release.py verify-assets DIR --version V
                                                      SHA256SUMS names exactly DIR's assets; digests agree
    python3 scripts/ci_release.py itch-channels V     the target -> itch channel table for version V
    python3 scripts/ci_release.py check-secrets       no workflow outside publish.yml reads a secret or an
                                                      Environment; publish.yml has two triggers only
    python3 scripts/ci_release.py check-switch        publish.yml's live steps sit behind MINEWORLD_PUBLISH_LIVE
    python3 scripts/ci_release.py citation            CITATION.cff's `version` equals the workspace version
    python3 scripts/ci_release.py --self-test         every rule above on planted inputs, no network

`citation` is also called by R-b's `verify-tag` on a tag run, so a tag cannot ship with a stale citation
(design `pr-s23-rd-publish.md` sections 4.1 and 3). R-b's commands (`verify-tag`, `notes`, `sums`, `repro`)
are added beside these by its own design (`pr-s23-rb-release-workflow.md` section 4.2).

Exits non-zero and names every problem. The workflow parsers are line-based: this script reads only the
shapes `publish.yml` is written in (section 3), and the self-test plants each shape it must catch.
"""

from __future__ import annotations

import hashlib
import re
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PUBLISH_WORKFLOW = "publish.yml"
LIVE_SWITCH = "vars.MINEWORLD_PUBLISH_LIVE == 'true'"

# The release tag that publishes: vMAJOR.MINOR.PATCH, or a release candidate vMAJOR.MINOR.PATCH-rc.N.
TAG_SHAPE = re.compile(r"^v(?P<version>\d+\.\d+\.\d+)(?:-rc\.(?P<rc>\d+))?$")
# Target -> (itch channel for a release, archive extension). Candidates add `-rc` to the channel name.
TARGETS: dict[str, tuple[str, str]] = {
    "windows-x86_64": ("windows", "zip"),
    "macos-universal": ("mac", "zip"),
    "linux-x86_64": ("linux", "tar.gz"),
    "linux-arm64": ("linux-arm64", "tar.gz"),
}
CITATION_VERSION = re.compile(r"^version:\s*['\"]?(?P<version>[^'\"\s#]+)['\"]?\s*(?:#.*)?$", re.M)
SUMS_LINE = re.compile(r"^(?P<digest>[0-9a-f]{64})  (?P<name>\S+)$")


# ---------- workspace and citation ----------


def workspace_version(cargo_toml: str) -> str | None:
    """The version every crate inherits: `[workspace.package]` in the root `Cargo.toml`."""
    package = tomllib.loads(cargo_toml).get("workspace", {}).get("package", {})
    version = package.get("version")
    return version if isinstance(version, str) else None


def citation_version(citation: str) -> str | None:
    """The top-level `version:` of a CFF file; nested keys are indented, so they never match."""
    found = CITATION_VERSION.search(citation)
    return found.group("version") if found else None


def citation_problems(citation: str, cargo_toml: str) -> list[str]:
    expected = workspace_version(cargo_toml)
    if expected is None:
        return ["Cargo.toml: [workspace.package] has no version"]
    actual = citation_version(citation)
    if actual is None:
        return ["CITATION.cff: no top-level version"]
    if actual != expected:
        return [f"CITATION.cff: version {actual} ≠ {expected}"]
    return []


# ---------- publish-gate ----------


def publish_gate(event: str, tag: str, prerelease: str, workspace: str) -> tuple[dict[str, str], list[str]]:
    """The run's decision. Returns (outputs, problems); a problem refuses the run."""
    outputs = {"publish": "false", "tag": tag, "version": workspace, "rc": ""}
    if event == "workflow_dispatch":
        return outputs, []  # a dispatch verifies a named draft and publishes nothing
    if event != "release":
        return outputs, [f"event {event!r} is not one this workflow handles"]
    found = TAG_SHAPE.match(tag)
    if found is None:
        return outputs, []  # promo-v* and other releases: this workflow publishes nothing for them (F-Rd-2)
    version, rc = found.group("version"), found.group("rc")
    if prerelease not in ("true", "false"):
        return outputs, [f"prerelease must be true or false, got {prerelease!r}"]
    if (rc is not None) != (prerelease == "true"):
        return outputs, [
            f"tag {tag} with prerelease={prerelease}: a release candidate (-rc.N) must be a pre-release, "
            "and a version tag must not be"
        ]
    if version != workspace:
        return outputs, [f"tag {tag}: version {version} ≠ workspace version {workspace}"]
    outputs.update(publish="true", version=version, rc=rc or "")
    return outputs, []


# ---------- verify-assets ----------


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def expected_names(version: str) -> list[str]:
    """The four archives and the SBOM that `release.yml` names (R-b section 3.7 and 3.8)."""
    names = [f"MineWorld-{version}-{target}.{ext}" for target, (_, ext) in TARGETS.items()]
    return names + [f"MineWorld-{version}-sbom.spdx.json"]


def asset_problems(folder: Path, version: str) -> list[str]:
    sums = folder / "SHA256SUMS"
    if not sums.is_file():
        return ["SHA256SUMS is missing"]
    listed: dict[str, str] = {}
    problems = []
    for number, line in enumerate(sums.read_text(encoding="utf-8").splitlines(), start=1):
        found = SUMS_LINE.match(line)
        if found is None:
            problems.append(f"SHA256SUMS:{number}: not a `digest  name` line")
        else:
            listed[found.group("name")] = found.group("digest")
    present = {path.name for path in folder.iterdir() if path.name != "SHA256SUMS"}
    for name in sorted(set(expected_names(version)) - set(listed)):
        problems.append(f"{name}: expected by version {version} but not listed in SHA256SUMS")
    for name in sorted(present - set(listed)):
        problems.append(f"{name}: an asset not listed in SHA256SUMS")
    for name in sorted(set(listed) - present):
        problems.append(f"{name}: listed in SHA256SUMS but missing")
    for name in sorted(set(listed) & present):
        if sha256_of(folder / name) != listed[name]:
            problems.append(f"{name}: sha256 differs from SHA256SUMS")
    return problems


# ---------- itch-channels ----------


def channel_for(target: str, version: str) -> str:
    """The itch channel of a target: `windows`, `mac`, `linux`, `linux-arm64`, with `-rc` for a candidate."""
    if target not in TARGETS:
        raise KeyError(f"unknown target {target!r}; the table has {', '.join(TARGETS)}")
    base, _ = TARGETS[target]
    return base + ("-rc" if "-rc." in version else "")


def itch_rows(version: str) -> list[tuple[str, str, str, str, str]]:
    """Rows of `target archive channel userversion hidden`. A candidate's channels are hidden (Q-Rd-7)."""
    rows = []
    for target, (_, ext) in TARGETS.items():
        candidate = "-rc." in version
        rows.append(
            (target, f"MineWorld-{version}-{target}.{ext}", channel_for(target, version), version,
             "yes" if candidate else "no")
        )
    return rows


# ---------- check-secrets and check-switch (workflow text) ----------


def jobs_of(workflow: str) -> dict[str, list[tuple[int, str]]]:
    """Each job's name with its (line number, line) pairs, from the `jobs:` section. Comments are dropped."""
    jobs: dict[str, list[tuple[int, str]]] = {}
    current: str | None = None
    inside = False
    for number, line in enumerate(workflow.splitlines(), start=1):
        if re.match(r"^jobs:\s*$", line):
            inside = True
            continue
        if inside and re.match(r"^\S", line):
            break
        if not inside or line.lstrip().startswith("#"):
            continue
        key = re.match(r"^  (?P<job>[A-Za-z0-9_-]+):\s*$", line)
        if key:
            current = key.group("job")
            jobs[current] = []
        elif current is not None:
            jobs[current].append((number, line))
    return jobs


def triggers_of(workflow: str) -> set[str]:
    """The keys under the top-level `on:` (the two-space keys of its block)."""
    found: set[str] = set()
    inside = False
    for line in workflow.splitlines():
        if re.match(r"^on:\s*$", line):
            inside = True
        elif inside and re.match(r"^\S", line):
            break
        elif inside:
            key = re.match(r"^  (?P<trigger>[a-z_]+):", line)
            if key:
                found.add(key.group("trigger"))
    return found


def secret_names(text: str) -> set[str]:
    return set(re.findall(r"\bsecrets\.(\w+)", text))


def check_secrets(workflows: dict[str, str]) -> list[str]:
    """`workflows` maps a file name to its text. Only publish.yml may read a secret or name an Environment."""
    problems = []
    for name, text in sorted(workflows.items()):
        if "pull_request_target" in text:
            problems.append(f"{name}: pull_request_target is refused (a fork's code would run with secrets)")
        for number, line in enumerate(text.splitlines(), start=1):
            if re.match(r"^\s*environment\s*:", line) and name != PUBLISH_WORKFLOW:
                problems.append(f"{name}:{number}: names an Environment; only {PUBLISH_WORKFLOW} may")
        if name != PUBLISH_WORKFLOW:
            for secret in sorted(secret_names(text) - {"GITHUB_TOKEN"}):
                problems.append(f"{name}: reads secrets.{secret}; only {PUBLISH_WORKFLOW} may read a secret")
    publish = workflows.get(PUBLISH_WORKFLOW)
    if publish is None:
        problems.append(f"{PUBLISH_WORKFLOW} is missing")
    else:
        extra = triggers_of(publish) - {"release", "workflow_dispatch"}
        if extra:
            problems.append(f"{PUBLISH_WORKFLOW}: triggers {sorted(extra)} are refused (release and dispatch only)")
    return problems


def check_switch(publish: str) -> list[str]:
    """Every job that runs butler or holds the key is gated on the switch; the key stays in `push`."""
    problems = []
    jobs = jobs_of(publish)
    if "verify" in jobs and not any("live publishing is off" in line for _, line in jobs["verify"]):
        problems.append(f"{PUBLISH_WORKFLOW}: job verify does not log 'live publishing is off' while the switch is off")
    for job, lines in jobs.items():
        body = "\n".join(line for _, line in lines)
        touches_butler = bool(re.search(r"\bbutler\b|BUTLER_API_KEY", body))
        if re.search(r"BUTLER_API_KEY", body) and job != "push":
            problems.append(f"{PUBLISH_WORKFLOW}: job {job} holds BUTLER_API_KEY; only job push may")
        if re.search(r"^\s*environment\s*:", body, re.M) and job != "push":
            problems.append(f"{PUBLISH_WORKFLOW}: job {job} names an Environment; only job push may")
        if not touches_butler:
            continue
        condition = next((line for _, line in lines if re.match(r"^    if:", line)), "")
        if LIVE_SWITCH not in condition:
            problems.append(f"{PUBLISH_WORKFLOW}: job {job} runs butler without `{LIVE_SWITCH}` in its if:")
        if "needs.gate.outputs.publish == 'true'" not in condition:
            problems.append(f"{PUBLISH_WORKFLOW}: job {job} runs butler without the gate's publish output in its if:")
    push = jobs.get("push", [])
    if not any("environment: itch" in line for _, line in push):
        problems.append(f"{PUBLISH_WORKFLOW}: job push must run in Environment itch")
    return problems


def check_workflows(root: Path) -> list[str]:
    directory = root / ".github" / "workflows"
    workflows = {path.name: path.read_text(encoding="utf-8") for path in sorted(directory.glob("*.y*ml"))}
    problems = check_secrets(workflows)
    if PUBLISH_WORKFLOW in workflows:
        problems += check_switch(workflows[PUBLISH_WORKFLOW])
    return problems


# ---------- self-test ----------

CARGO = '[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "0.2.0"\nedition = "2024"\n'

GOOD_PUBLISH = """name: publish
on:
  release:
    types: [published]
  workflow_dispatch:
    inputs:
      tag:
        type: string
jobs:
  gate:
    runs-on: ubuntu-24.04
    steps:
      - run: echo gate
  verify:
    needs: gate
    if: needs.gate.result == 'success' && (github.event_name == 'workflow_dispatch' || needs.gate.outputs.publish == 'true')
    runs-on: ubuntu-24.04
    steps:
      - name: Say whether live publishing is on
        run: echo "live publishing is off"
  push:
    needs: [gate, verify]
    if: github.event_name == 'release' && needs.gate.outputs.publish == 'true' && vars.MINEWORLD_PUBLISH_LIVE == 'true'
    runs-on: ubuntu-24.04
    environment: itch
    steps:
      - name: Push
        env:
          BUTLER_API_KEY: ${{ secrets.BUTLER_API_KEY }}
        run: butler push folder "$ITCH_TARGET:linux" --userversion 0.2.0
"""


def self_test() -> list[str]:
    failures: list[str] = []

    def expect(label: str, got: object, want: object) -> None:
        if got != want:
            failures.append(f"{label}: expected {want!r}, got {got!r}")

    # citation
    expect("citation equal", citation_problems("version: 0.2.0\n", CARGO), [])
    expect("citation stale", citation_problems("cff-version: 1.2.0\nversion: 0.1.0\n", CARGO),
           ["CITATION.cff: version 0.1.0 ≠ 0.2.0"])
    expect("citation quoted", citation_problems("version: '0.2.0'\n", CARGO), [])
    expect("citation nested only", citation_problems("authors:\n  - version: 0.2.0\n", CARGO),
           ["CITATION.cff: no top-level version"])
    expect("citation no workspace version", citation_problems("version: 0.2.0\n", "[workspace]\n"),
           ["Cargo.toml: [workspace.package] has no version"])

    # publish-gate
    expect("gate v0.2.0", publish_gate("release", "v0.2.0", "false", "0.2.0")[0]["publish"], "true")
    rc = publish_gate("release", "v0.2.0-rc.1", "true", "0.2.0")
    expect("gate rc pre-release", (rc[0]["publish"], rc[0]["rc"], rc[1]), ("true", "1", []))
    expect("gate rc not pre-release refused", publish_gate("release", "v0.2.0-rc.1", "false", "0.2.0")[1] != [], True)
    expect("gate version tag pre-release refused", publish_gate("release", "v0.2.0", "true", "0.2.0")[1] != [], True)
    promo = publish_gate("release", "promo-v2.3", "true", "0.2.0")
    expect("gate promo publishes nothing", (promo[0]["publish"], promo[1]), ("false", []))
    expect("gate mismatched version refused", publish_gate("release", "v9.9.9", "false", "0.2.0")[1] != [], True)
    dispatch = publish_gate("workflow_dispatch", "test-42", "", "0.2.0")
    expect("gate dispatch publishes nothing", (dispatch[0]["publish"], dispatch[1]), ("false", []))

    # verify-assets, on a planted folder
    with tempfile.TemporaryDirectory() as scratch:
        folder = Path(scratch)
        names = expected_names("0.2.0")
        for name in names:
            (folder / name).write_bytes(f"planted {name}".encode())
        sums = "".join(f"{sha256_of(folder / name)}  {name}\n" for name in names)
        (folder / "SHA256SUMS").write_text(sums, encoding="utf-8")
        expect("assets good", asset_problems(folder, "0.2.0"), [])
        (folder / names[0]).write_bytes(b"modified")
        expect("assets modified named", asset_problems(folder, "0.2.0"),
               [f"{names[0]}: sha256 differs from SHA256SUMS"])
        (folder / names[0]).write_bytes(f"planted {names[0]}".encode())
        (folder / "extra.bin").write_bytes(b"x")
        expect("assets extra named", asset_problems(folder, "0.2.0"),
               ["extra.bin: an asset not listed in SHA256SUMS"])
        (folder / "extra.bin").unlink()
        (folder / names[1]).unlink()
        expect("assets missing named", asset_problems(folder, "0.2.0"),
               [f"{names[1]}: listed in SHA256SUMS but missing"])

    # itch-channels
    expect("channels release", [row[2] for row in itch_rows("0.2.0")],
           ["windows", "mac", "linux", "linux-arm64"])
    expect("channels candidate", [(row[2], row[3], row[4]) for row in itch_rows("0.2.0-rc.2")][0],
           ("windows-rc", "0.2.0-rc.2", "yes"))
    expect("channels release hidden", {row[4] for row in itch_rows("0.2.0")}, {"no"})
    try:
        channel_for("solaris-sparc", "0.2.0")
        failures.append("channels: an unknown target must fail")
    except KeyError:
        pass

    # check-secrets
    expect("secrets clean", check_secrets({"ci.yml": "run: echo ${{ secrets.GITHUB_TOKEN }}\n",
                                           PUBLISH_WORKFLOW: GOOD_PUBLISH}), [])
    expect("secrets read outside publish", check_secrets({"ci.yml": "x: ${{ secrets.FOO }}\n",
                                                          PUBLISH_WORKFLOW: GOOD_PUBLISH}),
           ["ci.yml: reads secrets.FOO; only publish.yml may read a secret"])
    expect("environment outside publish", check_secrets({"ci.yml": "  env:\n    environment: itch\n",
                                                         PUBLISH_WORKFLOW: GOOD_PUBLISH}),
           ["ci.yml:2: names an Environment; only publish.yml may"])
    expect("pull_request_target refused", check_secrets({"ci.yml": "on: pull_request_target\n",
                                                         PUBLISH_WORKFLOW: GOOD_PUBLISH}),
           ["ci.yml: pull_request_target is refused (a fork's code would run with secrets)"])
    expect("publish push trigger refused", check_secrets({PUBLISH_WORKFLOW: GOOD_PUBLISH.replace(
        "  workflow_dispatch:", "  push:\n  workflow_dispatch:")}),
           [f"{PUBLISH_WORKFLOW}: triggers ['push'] are refused (release and dispatch only)"])

    # check-switch
    expect("switch good", check_switch(GOOD_PUBLISH), [])
    ungated = GOOD_PUBLISH.replace("vars.MINEWORLD_PUBLISH_LIVE == 'true'", "true")
    expect("switch ungated push", check_switch(ungated),
           [f"{PUBLISH_WORKFLOW}: job push runs butler without `{LIVE_SWITCH}` in its if:"])
    moved = GOOD_PUBLISH.replace('echo "live publishing is off"', "butler push folder x")
    expect("butler moved to verify", any("job verify runs butler" in p for p in check_switch(moved)), True)
    unlogged = GOOD_PUBLISH.replace("live publishing is off", "nothing")
    expect("verify must log the switch", check_switch(unlogged),
           [f"{PUBLISH_WORKFLOW}: job verify does not log 'live publishing is off' while the switch is off"])
    leaked = GOOD_PUBLISH.replace("echo gate", "echo ${{ env.BUTLER_API_KEY }}")
    expect("key outside push", any("job gate holds BUTLER_API_KEY" in p for p in check_switch(leaked)), True)
    return failures


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        failures = self_test()
        for failure in failures:
            print(f"ci_release self-test: FAIL: {failure}")
        print(f"ci_release self-test: {'FAILED' if failures else 'passed'}")
        return 1 if failures else 0
    if arguments == ["citation"]:
        problems = citation_problems(
            (ROOT / "CITATION.cff").read_text(encoding="utf-8"), (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for problem in problems:
            print(f"citation: {problem}", file=sys.stderr)
        if not problems:
            print("citation: CITATION.cff version matches the workspace version")
        return 1 if problems else 0
    if arguments and arguments[0] == "publish-gate":
        options = dict(zip(arguments[1::2], arguments[2::2]))
        workspace = workspace_version((ROOT / "Cargo.toml").read_text(encoding="utf-8")) or ""
        outputs, problems = publish_gate(
            options.get("--event", ""), options.get("--tag", ""), options.get("--prerelease", ""), workspace)
        for problem in problems:
            print(f"publish-gate: refused: {problem}", file=sys.stderr)
        if not problems and outputs["publish"] == "false":
            print("publish-gate: live publishing is off for this run; it verifies or publishes nothing",
                  file=sys.stderr)
        for key, value in outputs.items():
            print(f"{key}={value}")
        return 1 if problems else 0
    if len(arguments) == 4 and arguments[0] == "verify-assets" and arguments[2] == "--version":
        problems = asset_problems(Path(arguments[1]), arguments[3])
        for problem in problems:
            print(f"verify-assets: {problem}", file=sys.stderr)
        if not problems:
            print(f"verify-assets: {len(expected_names(arguments[3])) + 1} assets match SHA256SUMS")
        return 1 if problems else 0
    if len(arguments) == 2 and arguments[0] == "itch-channels":
        for row in itch_rows(arguments[1]):
            print(" ".join(row))
        return 0
    if arguments == ["check-secrets"]:
        problems = check_workflows(ROOT)
        for problem in problems:
            print(f"check-secrets: {problem}", file=sys.stderr)
        return 1 if problems else 0
    if arguments == ["check-switch"]:
        publish = ROOT / ".github" / "workflows" / PUBLISH_WORKFLOW
        problems = check_switch(publish.read_text(encoding="utf-8")) if publish.is_file() else ["publish.yml missing"]
        for problem in problems:
            print(f"check-switch: {problem}", file=sys.stderr)
        return 1 if problems else 0
    print("usage: ci_release.py publish-gate|verify-assets|itch-channels|check-secrets|check-switch|citation|--self-test",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

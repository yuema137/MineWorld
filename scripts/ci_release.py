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

R-b's commands (`release.yml`, `pr-s23-rb-release-workflow.md` §3; the workflow only installs tools and
calls these):
    python3 scripts/ci_release.py verify-tag --tag T --commit C [--dry] [--repo OWNER/NAME] [--output FILE]
        A tag run: T is "v" + Cargo.toml's [workspace.package] version (package.py's parser), optionally
        "-rc.N"; C is an ancestor of origin/main; C's latest `fast` and `test` check runs concluded
        `success`. A dry run (`--dry`, a dispatch or a branch push): the would-be tag "v<version>" is
        checked by the same rule; ancestry and check runs are not read. One line per check; exits 1 naming
        the first failure. `--output` appends tag, version, dry, prerelease and epoch (C's committer time,
        the archives' SOURCE_DATE_EPOCH) in GITHUB_OUTPUT's `key=value` form.
    python3 scripts/ci_release.py sums --out FILE PATH [PATH ...]
        `sha256sum`'s format, sorted by file name: one line per PATH, named by its base name.
    python3 scripts/ci_release.py sums --check FILE [FILE ...]
        Every line of each FILE against the file of that name beside it; exits 1 naming each mismatch.
    python3 scripts/ci_release.py notes --version V --sha S --run-url U --assets DIR [--tag-run]
                                        [--changelog FILE] [--template FILE] [--out FILE]
        The release notes from packaging/release-notes.txt.in (step-23 §10.3). "What's new" and "Known
        issues" come from CHANGELOG.md's section for V; a dry run without one reads "See the commit
        history"; a tag run without one fails.
    python3 scripts/ci_release.py reference-root DIR
        The AC-8 reference's world tree: the bundle's worlds and Presentation Pack parts, copied from this
        checkout by package.py's own lists into DIR, laid out like a bundle's runtime/ (R-b §3.5).
    python3 scripts/ci_release.py repro A B
        Two bundles (folders, or .zip / .tar.gz archives) compared file by file by SHA-256; lists every
        file that differs or exists in one only (step-23 §5.3: a measurement, not a gate; exits 0).

`citation` is also called by R-b's `verify-tag` on a tag run, so a tag cannot ship with a stale citation
(design `pr-s23-rd-publish.md` sections 4.1 and 3). R-b's commands (`verify-tag`, `notes`, `sums`, `repro`)
are added beside these by its own design (`pr-s23-rb-release-workflow.md` section 4.2).

Exits non-zero and names every problem. The workflow parsers are line-based: this script reads only the
shapes `publish.yml` is written in (section 3), and the self-test plants each shape it must catch.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import package  # noqa: E402  (R-b: the bundle's version parser and lists are package.py's, never copied)

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


# ================================== R-b: the release workflow's rules ==================================
# `release.yml` and `pr-s23-rb-release-workflow.md` §3. One tag rule for both PRs: TAG_SHAPE above.

VERSION_SHAPE = re.compile(r"^\d+\.\d+\.\d+$")
REQUIRED_CHECKS = ("fast", "test")  # ci.yml's required checks by name (ARC-48)
NOTES_TEMPLATE = ROOT / "packaging" / "release-notes.txt.in"
CHANGELOG = ROOT / "CHANGELOG.md"
NO_CHANGELOG = "See the commit history."
ALWAYS_KNOWN = "Saves may not load in a later 0.x version."
RB_COMMANDS = ("verify-tag", "sums", "notes", "reference-root", "repro")


class Refused(Exception):
    """A rule did not hold; the message names it."""


# ---------- verify-tag ----------


def check_tag(tag: str, version: str) -> bool:
    """The tag against the workspace version; returns whether it is a release candidate."""
    if not VERSION_SHAPE.match(version):
        raise Refused(f"Cargo.toml [workspace.package] version {version!r} is not MAJOR.MINOR.PATCH")
    match = TAG_SHAPE.match(tag)
    if not match:
        raise Refused(f"{tag} is not v<MAJOR.MINOR.PATCH> or v<MAJOR.MINOR.PATCH>-rc.<N>")
    if match.group("version") != version:
        raise Refused(f"{tag} is not v{version} (Cargo.toml [workspace.package] version)")
    return match.group("rc") is not None


def check_runs(reply: dict, name: str) -> str:
    """The verdict on one required check from `GET …/commits/C/check-runs?check_name=…&filter=latest`."""
    runs = [run for run in reply.get("check_runs", []) if run.get("name") == name]
    if not runs:
        raise Refused(f"check {name}: no check run on this commit")
    latest = max(runs, key=lambda run: run.get("id", 0))
    if latest.get("status") != "completed" or latest.get("conclusion") != "success":
        raise Refused(f"check {name}: {latest.get('status')}, conclusion {latest.get('conclusion')}, not success")
    return f"check {name}: success (check run {latest.get('id')})"


def fetch_check_runs(repo: str, commit: str, name: str) -> dict:
    api = f"repos/{repo}/commits/{commit}/check-runs?check_name={name}&filter=latest&per_page=100"
    try:
        out = subprocess.run(["gh", "api", api], capture_output=True, text=True)
    except OSError as error:
        raise Refused(f"check {name}: could not start gh: {error}") from error
    if out.returncode != 0:
        raise Refused(f"check {name}: gh api failed ({out.stderr.strip() or out.returncode}); never passed on an unknown")
    try:
        return json.loads(out.stdout)
    except ValueError as error:
        raise Refused(f"check {name}: gh api returned no JSON ({error})") from error


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)


def verify_tag(tag: str, commit: str, dry: bool, repo: str, fetch=fetch_check_runs, ancestor=None) -> dict:
    """Every rule of R-b §3.3, in order; returns the outputs. Raises Refused on the first failure."""
    try:
        version = package.workspace_version()
    except package.Failure as failure:
        raise Refused(str(failure)) from failure
    if dry:
        tag = f"v{version}"
    prerelease = check_tag(tag, version)
    print(f"verify-tag: {tag} is v{version} (Cargo.toml [workspace.package] version){' — dry run' if dry else ''}")
    if not dry:
        is_ancestor = ancestor(commit) if ancestor else git("merge-base", "--is-ancestor", commit, "origin/main").returncode == 0
        if not is_ancestor:
            raise Refused(f"{commit} is not an ancestor of origin/main")
        print(f"verify-tag: {commit} is on origin/main")
        for name in REQUIRED_CHECKS:
            print("verify-tag: " + check_runs(fetch(repo, commit, name), name))
    else:
        print("verify-tag: dry run, so the ancestry and the check runs are not read")
    return {"tag": tag, "version": version, "dry": "true" if dry else "false",
            "prerelease": "true" if prerelease else "false"}


def commit_epoch(commit: str) -> str:
    out = git("show", "-s", "--format=%ct", commit)
    if out.returncode != 0 or not out.stdout.strip().isdigit():
        raise Refused(f"{commit}: no committer time ({out.stderr.strip()})")
    return out.stdout.strip()


# ---------- sums ----------


def sums_text(paths: list[Path]) -> str:
    names = [p.name for p in paths]
    if len(set(names)) != len(names):
        raise Refused("sums: two files share a name: " + ", ".join(sorted(names)))
    return "".join(f"{sha256_of(p)}  {p.name}\n" for p in sorted(paths, key=lambda p: p.name))


def check_sums(sums: Path) -> list[str]:
    """Each line of `sums` against the file of that name beside it."""
    problems = []
    lines = re.findall(r"^([0-9a-fA-F]{64}) [ *](.+?)\r?$", sums.read_text(encoding="utf-8"), re.M)
    if not lines:
        problems.append(f"{sums}: no `<sha256>  <name>` line")
    for digest, name in lines:
        path = sums.parent / name
        if not path.is_file():
            problems.append(f"{sums}: {name} is missing")
        elif sha256_of(path) != digest.lower():
            problems.append(f"{sums}: {name} does not match its SHA-256")
    return problems


# ---------- notes ----------


def changelog_section(text: str, version: str) -> str | None:
    """The body of `## [V]` (or `## V`) up to the next `## ` heading; None when absent."""
    match = re.search(rf"^## \[?{re.escape(version)}\]?(?:\s.*)?$", text, re.M)
    if not match:
        return None
    rest = text[match.end():]
    end = re.search(r"^## ", rest, re.M)
    return (rest[:end.start()] if end else rest).strip("\n")


def split_known(section: str) -> tuple[str, str]:
    """(what's new, known issues): the `### Known issues` subsection is cut out of the rest."""
    match = re.search(r"^### Known issues\s*$", section, re.M)
    if not match:
        return section.strip(), ""
    rest = section[match.end():]
    end = re.search(r"^### ", rest, re.M)
    known = rest[:end.start()] if end else rest
    new = section[:match.start()] + (rest[end.start():] if end else "")
    return new.strip(), known.strip()


def downloads(assets: Path) -> str:
    lines = []
    for target in package.TARGETS.values():
        found = sorted(assets.glob(f"MineWorld-*-{target.name}.{target.archive}"))
        if len(found) != 1:
            raise Refused(f"notes: expected one {target.name} archive in {assets}, found {len(found)}")
        size = found[0].stat().st_size / package.MB
        lines.append(f"- {target.name} — `{found[0].name}`, {size:.1f} MB")
    return "\n".join(lines)


def render_notes(template: str, version: str, sha: str, run_url: str, download: str,
                 changelog: str | None, tag_run: bool) -> str:
    section = changelog_section(changelog, version) if changelog is not None else None
    if section is None:
        if tag_run:
            raise Refused(f"notes: CHANGELOG.md has no section {version}" if changelog is not None
                          else "notes: no CHANGELOG.md, which a tag run requires (R-d writes it)")
        new, known = NO_CHANGELOG, ""
    else:
        new, known = split_known(section)
    known = "\n".join(line for line in (known, f"- {ALWAYS_KNOWN}") if line)
    return template.format_map({"version": version, "sha": sha, "run_url": run_url, "downloads": download,
                                "whats_new": new or NO_CHANGELOG, "known_issues": known})


# ---------- reference-root and repro ----------


def reference_root(into: Path) -> list[str]:
    """The worlds and Presentation Pack parts a bundle's runtime/ holds, by package.py's lists (R-b §3.5)."""
    if into.exists() and any(into.iterdir()):
        raise Refused(f"reference-root: {into} is not empty")
    into.mkdir(parents=True, exist_ok=True)
    prefixes = [f"worlds/{w}" for w in package.WORLDS]
    prefixes += [f"{package.PACK_2D}/{p}" for p in package.PACK_2D_PARTS]
    prefixes += [f"{package.PACK_3D}/{p}" for p in package.PACK_3D_PARTS]
    for prefix in prefixes:
        package.copy_tracked(prefix, into)
    return prefixes


def file_digests(source: Path) -> dict[str, str]:
    """Every file of a bundle by its path below the bundle's top folder."""
    with tempfile.TemporaryDirectory(prefix="mineworld-repro-") as tmp:
        top = source if source.is_dir() else package.unpack(source, Path(tmp) / "unpacked")
        return {p.relative_to(top).as_posix(): sha256_of(p) for p in sorted(top.rglob("*")) if p.is_file()}


def repro(a: dict[str, str], b: dict[str, str]) -> list[str]:
    lines = [f"only in A: {name}" for name in sorted(set(a) - set(b))]
    lines += [f"only in B: {name}" for name in sorted(set(b) - set(a))]
    lines += [f"differs: {name}" for name in sorted(set(a) & set(b)) if a[name] != b[name]]
    return lines


# ---------- R-b self-test ----------


def run_json(name: str, status: str, conclusion: str | None, run_id: int) -> dict:
    """One check run in the shape GitHub's API returns it (recorded from this repository, 2026-10-27)."""
    return {"id": run_id, "name": name, "status": status, "conclusion": conclusion}


def rb_self_test() -> list[str]:
    failures: list[str] = []

    def judged(name: str, good: bool, got: object = "") -> None:
        if not good:
            failures.append(f"{name}: got {got}")

    def refusal(call) -> str:
        try:
            call()
        except Refused as refused:
            return str(refused)
        return ""

    judged("v0.1.0 matches 0.1.0, not a release candidate", check_tag("v0.1.0", "0.1.0") is False)
    judged("v0.1.0-rc.2 matches 0.1.0, a release candidate", check_tag("v0.1.0-rc.2", "0.1.0") is True)
    for tag, expected in (("v9.9.9", "v9.9.9 is not v0.1.0"), ("v0.1", "is not v<MAJOR"),
                          ("0.1.0", "is not v<MAJOR"), ("v0.1.0-beta", "is not v<MAJOR"),
                          ("v0.1.0-rc.", "is not v<MAJOR"), ("v0.1.0-rc.1x", "is not v<MAJOR")):
        why = refusal(lambda tag=tag: check_tag(tag, "0.1.0"))
        judged(f"{tag} is refused", expected in why, why or "accepted")
    why = refusal(lambda: check_tag("v0.1.0", "0.1"))
    judged("a workspace version that is not MAJOR.MINOR.PATCH is refused", "is not MAJOR.MINOR.PATCH" in why, why)

    green = {"total_count": 1, "check_runs": [run_json("fast", "completed", "success", 7)]}
    judged("a successful check passes", check_runs(green, "fast").startswith("check fast: success"))
    for label, reply, expected in (
        ("a missing check", {"total_count": 0, "check_runs": []}, "no check run"),
        ("a failed check", {"check_runs": [run_json("test", "completed", "failure", 9)]}, "conclusion failure"),
        ("a skipped check (main's docs-only pushes)", {"check_runs": [run_json("test", "completed", "skipped", 9)]},
         "conclusion skipped"),
        ("a check still running", {"check_runs": [run_json("test", "in_progress", None, 9)]}, "in_progress"),
        ("an older success under a newer failure",
         {"check_runs": [run_json("test", "completed", "success", 8), run_json("test", "completed", "failure", 9)]},
         "conclusion failure"),
    ):
        why = refusal(lambda reply=reply: check_runs(reply, "test"))
        judged(f"{label} is refused", expected in why, why or "accepted")

    version = package.workspace_version()
    replies = {"fast": green, "test": {"check_runs": [run_json("test", "completed", "success", 8)]}}
    outputs = verify_tag(f"v{version}", "c0ffee", False, "o/r", fetch=lambda r, c, n: replies[n], ancestor=lambda c: True)
    judged("a tag run on main with fast and test green passes", outputs["dry"] == "false" and outputs["tag"] == f"v{version}",
           outputs)
    why = refusal(lambda: verify_tag(f"v{version}", "c0ffee", False, "o/r", fetch=lambda r, c, n: replies[n],
                                     ancestor=lambda c: False))
    judged("a tag on a commit not on main is refused", "not an ancestor of origin/main" in why, why)
    failing = dict(replies, test={"check_runs": [run_json("test", "completed", "failure", 9)]})
    why = refusal(lambda: verify_tag(f"v{version}", "c0ffee", False, "o/r", fetch=lambda r, c, n: failing[n],
                                     ancestor=lambda c: True))
    judged("a tag run with a red test is refused", "check test" in why, why)
    why = refusal(lambda: verify_tag("v9.9.9", "c0ffee", False, "o/r", fetch=lambda r, c, n: replies[n],
                                     ancestor=lambda c: True))
    judged("A-R2: v9.9.9 is refused before anything else is read", f"v9.9.9 is not v{version}" in why, why)

    def no_network(*_):
        raise AssertionError("a dry run read the check runs")
    dry = verify_tag("v9.9.9", "c0ffee", True, "o/r", fetch=no_network, ancestor=no_network)
    judged("a dry run checks the would-be tag and reads neither ancestry nor checks",
           dry["tag"] == f"v{version}" and dry["dry"] == "true", dry)

    with tempfile.TemporaryDirectory(prefix="mineworld-ci-release-") as tmp:
        folder = Path(tmp)
        (folder / "b.zip").write_bytes(b"bee")
        (folder / "a.tar.gz").write_bytes(b"ay")
        text = sums_text([folder / "b.zip", folder / "a.tar.gz"])
        expected = (f"{hashlib.sha256(b'ay').hexdigest()}  a.tar.gz\n"
                    f"{hashlib.sha256(b'bee').hexdigest()}  b.zip\n")
        judged("sums: sha256sum's format, sorted by name", text == expected, text)
        (folder / "SUMS").write_text(text, encoding="utf-8")
        judged("sums --check passes on untouched files", check_sums(folder / "SUMS") == [], check_sums(folder / "SUMS"))
        (folder / "b.zip").write_bytes(b"bee!")
        judged("sums --check names a changed file", any("b.zip does not match" in p for p in check_sums(folder / "SUMS")),
               check_sums(folder / "SUMS"))
        (folder / "a.tar.gz").unlink()
        judged("sums --check names a missing file", any("a.tar.gz is missing" in p for p in check_sums(folder / "SUMS")),
               check_sums(folder / "SUMS"))

        for target in package.TARGETS.values():
            (folder / f"MineWorld-0.1.0-{target.name}.{target.archive}").write_bytes(b"x" * 1_500_000)
        download = downloads(folder)
        judged("notes: one download line per target, with its size",
               download.count("\n") == 3 and "linux-arm64 — `MineWorld-0.1.0-linux-arm64.tar.gz`, 1.5 MB" in download, download)
        template = NOTES_TEMPLATE.read_text(encoding="utf-8")
        changelog = ("# Changelog\n\n## [0.2.0] - later\n\n### Added\n- not this one\n\n"
                     "## [0.1.0] - 2026-11-01\n\n### Added\n- the demo (#1)\n\n### Known issues\n- a known one (#2)\n\n"
                     "### Fixed\n- a fix (#3)\n\n## [0.0.1]\n- older\n")
        notes = render_notes(template, "0.1.0", "abc123", "https://run", download, changelog, True)
        judged("notes: this version's section only, known issues cut out and the 0.x line always there",
               "- the demo (#1)" in notes and "- a fix (#3)" in notes and "not this one" not in notes
               and "- older" not in notes and "- a known one (#2)" in notes and ALWAYS_KNOWN in notes
               and notes.index("a known one") > notes.index("a fix"), notes)
        judged("notes: every placeholder is filled", "{" not in notes and "abc123" in notes and "https://run" in notes, notes)
        judged("notes: the template carries step-23 §10.3's eight headings",
               all(h in template for h in ("What this is", "Download", "First open", "Verify (optional)",
                                           "What's new", "Known issues", "Licences", "Built from")), template)
        dry_notes = render_notes(template, "0.1.0", "abc", "u", download, None, False)
        judged("notes: a dry run without CHANGELOG.md says so and keeps the 0.x line",
               NO_CHANGELOG in dry_notes and ALWAYS_KNOWN in dry_notes, dry_notes)
        why = refusal(lambda: render_notes(template, "0.1.0", "abc", "u", download, None, True))
        judged("notes: a tag run without CHANGELOG.md fails", "no CHANGELOG.md" in why, why)
        why = refusal(lambda: render_notes(template, "0.3.0", "abc", "u", download, changelog, True))
        judged("notes: a tag run without its version's section fails", "CHANGELOG.md has no section 0.3.0" in why, why)
        (folder / "MineWorld-0.1.0-linux-arm64.tar.gz").unlink()
        why = refusal(lambda: downloads(folder))
        judged("notes: a missing archive fails by target", "linux-arm64 archive" in why, why)

        a = {"runtime/mineworld": "1", "runtime/clients/x.pck": "2", "PLAY.txt": "3"}
        b = {"runtime/mineworld": "1", "runtime/clients/x.pck": "9", "NOTICE": "4"}
        judged("repro lists each differing and one-sided file",
               repro(a, b) == ["only in A: PLAY.txt", "only in B: NOTICE", "differs: runtime/clients/x.pck"], repro(a, b))
        judged("repro of equal bundles is empty", repro(a, a) == [], repro(a, a))
    return failures


# ---------- R-b command line ----------


def write_outputs(path: str | None, outputs: dict[str, str]) -> None:
    lines = "".join(f"{key}={value}\n" for key, value in outputs.items())
    print(lines, end="")
    if path:
        with open(path, "a", encoding="utf-8", newline="\n") as out:
            out.write(lines)


def rb_main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="ci_release.py")
    sub = parser.add_subparsers(dest="command", required=True)
    v = sub.add_parser("verify-tag")
    v.add_argument("--tag", default="")
    v.add_argument("--commit", required=True)
    v.add_argument("--dry", action="store_true")
    v.add_argument("--repo", default="yuema137/MineWorld")
    v.add_argument("--output")
    s = sub.add_parser("sums")
    s.add_argument("--out")
    s.add_argument("--check", action="store_true")
    s.add_argument("paths", nargs="+")
    n = sub.add_parser("notes")
    n.add_argument("--version", required=True)
    n.add_argument("--sha", required=True)
    n.add_argument("--run-url", required=True)
    n.add_argument("--assets", required=True)
    n.add_argument("--tag-run", action="store_true")
    n.add_argument("--changelog", default=str(CHANGELOG))
    n.add_argument("--template", default=str(NOTES_TEMPLATE))
    n.add_argument("--out")
    r = sub.add_parser("reference-root")
    r.add_argument("dir")
    p = sub.add_parser("repro")
    p.add_argument("a")
    p.add_argument("b")
    args = parser.parse_args(argv)
    try:
        return rb_dispatch(args)
    except Refused as refused:
        print(f"{args.command}: {refused}", file=sys.stderr)
        return 1
    except (package.Failure, OSError, zipfile.BadZipFile, tarfile.TarError) as error:
        print(f"{args.command}: {error}", file=sys.stderr)
        return 1


def rb_dispatch(args: argparse.Namespace) -> int:
    if args.command == "verify-tag":
        outputs = verify_tag(args.tag, args.commit, args.dry, args.repo)
        outputs["epoch"] = commit_epoch(args.commit)
        write_outputs(args.output, outputs)
        return 0
    if args.command == "sums":
        paths = [Path(p) for p in args.paths]
        if args.check == bool(args.out):
            raise Refused("give exactly one of --out FILE or --check")
        if args.check:
            problems = [problem for path in paths for problem in check_sums(path)]
            for problem in problems:
                print(f"sums: {problem}", file=sys.stderr)
            if not problems:
                print("sums: " + ", ".join(p.name for p in paths) + ": every file matches")
            return 1 if problems else 0
        Path(args.out).write_text(sums_text(paths), encoding="utf-8", newline="\n")
        print(Path(args.out).read_text(encoding="utf-8"), end="")
        return 0
    if args.command == "notes":
        changelog = Path(args.changelog)
        text = render_notes(Path(args.template).read_text(encoding="utf-8"), args.version, args.sha, args.run_url,
                            downloads(Path(args.assets)),
                            changelog.read_text(encoding="utf-8") if changelog.is_file() else None, args.tag_run)
        if args.out:
            Path(args.out).write_text(text, encoding="utf-8", newline="\n")
        print(text, end="")
        return 0
    if args.command == "reference-root":
        prefixes = reference_root(Path(args.dir))
        print(f"reference-root: {len(prefixes)} tracked trees copied into {args.dir}")
        return 0
    lines = repro(file_digests(Path(args.a)), file_digests(Path(args.b)))
    for line in lines or ["every file is byte-identical"]:
        print(f"repro: {line}")
    return 0


# ---------- main ----------


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        failures = self_test() + rb_self_test()
        for failure in failures:
            print(f"ci_release self-test: FAIL: {failure}")
        print(f"ci_release self-test: {'FAILED' if failures else 'passed'}")
        return 1 if failures else 0
    if arguments and arguments[0] in RB_COMMANDS:
        return rb_main(arguments)
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
    print("usage: ci_release.py publish-gate|verify-assets|itch-channels|check-secrets|check-switch|citation|"
          "verify-tag|sums|notes|reference-root|repro|--self-test",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

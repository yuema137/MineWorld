#!/usr/bin/env python3
"""The nightly workflow's gate, its per-job verdicts and its report (`docs/DECISIONS.md` ARC-83).

`.github/workflows/nightly.yml` names these subcommands; what each group runs is `scripts/ci_layer.py`'s.

    python3 scripts/ci_nightly.py gate --github-output FILE
        Whether tonight runs, and which groups. A scheduled night runs iff `main`'s head differs from the
        `head_sha` of the last nightly run on `main` that concluded success or failure (none → run); a
        dispatch on `main` with force=false decides the same way; force=true, a dispatch elsewhere and a
        scratch push always run; any `gh api` error runs the night, with the error as the reason. Groups
        come from the dispatch input or, on `scratch/<name>[-<group>][-issue]-nightly`, from the branch
        name; an unknown group is an error (exit 1), never "none".
        Environment: EVENT, REF, SHA, REPO, RUN_ID, FORCE, GROUPS, GH_TOKEN.
    python3 scripts/ci_nightly.py verdict --job NAME --outcome STATUS [--layer L ...] [--step NAME=OUTCOME ...]
        One job's verdict, PASS only on positive evidence (I-13c-3), written to
        artifacts/nightly/verdict-<job>.txt: a layer that never started, a layer that started and never
        finished, a step that was skipped or cancelled, a cancelled job and a probe without its summary
        are INCONCLUSIVE; a layer that failed at a command, a failed step and a failed probe are FAIL.
    python3 scripts/ci_nightly.py report --artifacts DIR
        The night: one row per expected job, `main`'s push run for the commit, flake candidates, baseline
        drift, a benchmark; then the issue (label `nightly` on main, `nightly-scratch` on a scratch
        `-issue-nightly` branch). Exits 1 iff the night is red.
        Environment: NEEDS (toJSON(needs)), GATE_RUN, GATE_GROUPS, GATE_REASON, GATE_ISSUE, SHA, REF, REPO,
        RUN_ID, SERVER_URL, GH_TOKEN, GITHUB_STEP_SUMMARY.
    python3 scripts/ci_nightly.py --self-test
        The gate's decisions on recorded `gh api` answers (MN-6) and the verdict and report classifications.

Every value from a branch name, an input or a job output arrives through the environment, never through
an expression interpolated into a script. Standard library only, plus `gh`, which every hosted runner has.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

from ci_repeat import Sample, Transcript, classify
from ci_repeat import parse as parse_repeat

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "artifacts" / "nightly"
MAIN = "refs/heads/main"
WORKFLOW = "nightly.yml"
GROUPS = ("parity-long", "stability", "clients", "repeat")
# Every job between the gate and the report, with its group: a job missing from `needs` is a red row,
# never an absent one.
JOBS = (
    ("parity-long-linux", "parity-long"), ("parity-long-arm", "parity-long"), ("parity-long-mac", "parity-long"),
    ("parity-long-windows", "parity-long"), ("parity-long", "parity-long"),
    ("stability-linux", "stability"), ("stability-mac", "stability"), ("stability-windows", "stability"),
    ("clients-linux", "clients"), ("clients-mac", "clients"), ("clients-windows", "clients"),
    ("repeat-linux", "repeat"), ("repeat-mac", "repeat"), ("repeat-windows", "repeat"),
)
# The flake samples of one OS: the nightly's repeat job, and `main`'s push run's job of the same suite.
REPEAT_OS = (("repeat-linux", "test"), ("repeat-mac", "test-macos"), ("repeat-windows", "test-windows"))
TREND_NIGHTS = 14
ISSUE_TITLE = "Nightly CI is red on main"
VERDICTS = ("PASS", "FAIL", "INCONCLUSIVE")


class Unmet(Exception):
    """A step could not be done; the message names it."""


def gh(*arguments: str) -> str:
    result = subprocess.run(["gh", *arguments], cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        raise Unmet(f"gh {' '.join(arguments[:3])}: exit {result.returncode}: {result.stderr.strip()[-400:]}")
    return result.stdout


# ------------------------------------------------------------------------------------------------ gate


@dataclass
class Gate:
    run: bool
    groups: list[str]
    reason: str
    issue: str = ""
    last: str = ""


def groups_of(event: str, ref: str, requested: str) -> tuple[list[str], bool]:
    """(groups, issue mode). A dispatch names groups; a scratch branch may name one before `-nightly`."""
    if event == "workflow_dispatch":
        names = [name.strip() for name in (requested or "all").split(",") if name.strip()]
        if names == ["all"]:
            return list(GROUPS), False
        unknown = [name for name in names if name not in GROUPS]
        if unknown or not names:
            raise Unmet(f"unknown group(s) {', '.join(unknown) or '(none)'}; groups are all, {', '.join(GROUPS)}")
        return [name for name in GROUPS if name in names], False
    branch = ref.removeprefix("refs/heads/")
    stem = branch.removesuffix("-nightly") if branch.startswith("scratch/") and branch.endswith("-nightly") else ""
    issue = stem.endswith("-issue")
    stem = stem.removesuffix("-issue")
    named = [name for name in GROUPS if stem.endswith(f"-{name}")]
    return (named[-1:] if named else list(GROUPS)), issue


def decide(event: str, ref: str, sha: str, run_id: str, force: str, requested: str,
           runs: list[dict[str, object]] | Unmet) -> Gate:
    """The gate's decision from the event and the earlier nightly runs on main (or the error reading them)."""
    groups, issue = groups_of(event, ref, requested)
    label = "nightly" if ref == MAIN else ("nightly-scratch" if issue else "")
    keyed = event == "schedule" or (event == "workflow_dispatch" and ref == MAIN and force == "false")
    if not keyed:
        why = {"workflow_dispatch": f"dispatched on {ref}" + (" with force" if force == "true" else ""),
               "push": f"pushed to {ref}"}.get(event, f"event {event}")
        return Gate(True, groups, why, label)
    if isinstance(runs, Unmet):
        return Gate(True, groups, f"run: the earlier nightly runs could not be read ({runs})", label)
    earlier = [run for run in runs if str(run.get("id")) != run_id and run.get("conclusion") in ("success", "failure")]
    if not earlier:
        return Gate(True, groups, "run: no earlier nightly run on main concluded", label)
    last = earlier[0]
    previous = f"run {last.get('id')} on {str(last.get('head_sha'))[:12]}"
    if last.get("head_sha") == sha:
        return Gate(False, groups, f"skipped: main unmoved since {previous}", label, previous)
    return Gate(True, groups, f"run: main moved since {previous} to {sha[:12]}", label, previous)


def nightly_runs(repo: str) -> list[dict[str, object]] | Unmet:
    """The completed nightly runs on main, newest first (GitHub's order)."""
    try:
        answer = gh("api", f"repos/{repo}/actions/workflows/{WORKFLOW}/runs?branch=main&status=completed&per_page=30")
        return list(json.loads(answer)["workflow_runs"])
    except (Unmet, ValueError, KeyError) as error:
        return error if isinstance(error, Unmet) else Unmet(f"unreadable answer: {error}")


def gate(output: str) -> int:
    env = os.environ
    event, ref = env.get("EVENT", ""), env.get("REF", "")
    keyed = event == "schedule" or (event == "workflow_dispatch" and ref == MAIN and env.get("FORCE") == "false")
    runs = nightly_runs(env.get("REPO", "")) if keyed else []
    decided = decide(event, ref, env.get("SHA", ""), env.get("RUN_ID", ""), env.get("FORCE", ""),
                     env.get("GROUPS", ""), runs)
    print(f"[gate] {decided.reason}; groups {', '.join(decided.groups)}; issue label {decided.issue or 'none'}")
    with open(output, "a", encoding="utf-8") as out:
        out.write(f"run={'true' if decided.run else 'false'}\n")
        out.write(f"groups=,{','.join(decided.groups)},\n")
        out.write(f"reason={decided.reason}\n")
        out.write(f"issue={decided.issue}\n")
        out.write(f"last={decided.last}\n")
    return 0


# --------------------------------------------------------------------------------------------- verdict


def judge_job(outcome: str, layers: dict[str, list[str] | None], steps: dict[str, str],
              probes: list[list[str]], failed_tests: list[str]) -> tuple[str, list[str]]:
    """One job's verdict from what its layers, steps and probes left behind. PASS needs positive evidence."""
    fails: list[str] = []
    unknown: list[str] = []
    for layer, lines in layers.items():
        if lines is None:
            unknown.append(f"layer {layer} never started (a setup step failed before it)")
        elif any(line.startswith("failed at: ") for line in lines):
            fails += [f"layer {layer} {line}" for line in lines if line.startswith("failed at: ")]
        elif not any(line.startswith("passed") for line in lines):
            unknown.append(f"layer {layer} started and never finished")
    for step, result in steps.items():
        if result == "failure":
            fails.append(f"step {step} failed")
        elif result != "success":
            unknown.append(f"step {step} did not run ({result or 'no outcome'})")
    for probe in probes:
        found = probe[0].removeprefix("verdict ") if probe else "INCONCLUSIVE"
        if found == "FAIL":
            fails += probe[1:]
        elif found != "PASS":
            unknown += probe[1:] or ["a probe file without a verdict"]
    fails += [f"failed test {test}" for test in failed_tests]
    if outcome == "cancelled":
        unknown.append("the job was cancelled or timed out")
    if not layers and not steps:
        unknown.append("no layer or step was named, so nothing was evidenced")
    if fails:
        return "FAIL", fails + unknown
    if unknown or outcome != "success":
        return "INCONCLUSIVE", unknown or [f"the job's status is {outcome} outside its layers"]
    return "PASS", []


def verdict(arguments: list[str]) -> int:
    pairs = list(zip(arguments[::2], arguments[1::2]))
    job = next((value for key, value in pairs if key == "--job"), "")
    outcome = next((value for key, value in pairs if key == "--outcome"), "")
    if len(arguments) % 2 or not job or any(key not in ("--job", "--outcome", "--layer", "--step") for key, _ in pairs):
        print("usage: ci_nightly.py verdict --job NAME --outcome STATUS [--layer L ...] [--step NAME=OUTCOME ...]",
              file=sys.stderr)
        return 2
    layers = {}
    for layer in (value for key, value in pairs if key == "--layer"):
        path = OUT / f"layer-{layer}.txt"
        layers[layer] = path.read_text(encoding="utf-8").splitlines() if path.is_file() else None
    steps = dict(value.partition("=")[::2] for key, value in pairs if key == "--step")
    probes = [path.read_text(encoding="utf-8").splitlines() for path in sorted(OUT.glob("probe-*.txt"))]
    failed = sorted({f"{sample.name}: {test}" for path in sorted(OUT.glob("*.txt"))
                     if path.name == "repeat.txt" or path.name.endswith("-tests.txt")
                     for sample in parse_repeat(path.read_text(encoding="utf-8")) for test in sample.failed})
    decided, detail = judge_job(outcome, layers, steps, probes, failed)
    # Notes inform and never decide: a test skipped on this leg by name (QC-5), with its reason.
    notes = OUT / "notes.txt"
    detail += [f"note: {line}" for line in notes.read_text(encoding="utf-8").splitlines()] if notes.is_file() else []
    OUT.mkdir(parents=True, exist_ok=True)
    lines = [f"job {job}", f"commit {os.environ.get('GITHUB_SHA', '?')}", f"verdict {decided}",
             *[f"detail {line}" for line in detail]]
    (OUT / f"verdict-{job}.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(f"[verdict] {line}" for line in lines), flush=True)
    return 0


# ---------------------------------------------------------------------------------------------- report


@dataclass
class Row:
    job: str
    group: str
    verdict: str            # PASS | FAIL | INCONCLUSIVE | skipped
    detail: list[str] = field(default_factory=list)
    wall: str = ""


def read_verdict(path: Path) -> tuple[str, list[str]] | None:
    if not path.is_file():
        return None
    lines = path.read_text(encoding="utf-8").splitlines()
    found = next((line.removeprefix("verdict ") for line in lines if line.startswith("verdict ")), "")
    detail = [line.removeprefix("detail ") for line in lines if line.startswith("detail ")]
    return (found if found in VERDICTS else "INCONCLUSIVE"), detail


def classify_jobs(needs: dict[str, dict[str, object]], groups: list[str], artifacts: Path) -> list[Row]:
    rows = []
    for job, group in JOBS:
        if group not in groups:
            rows.append(Row(job, group, "skipped", ["not in tonight's groups"]))
            continue
        result = str(needs.get(job, {}).get("result", "absent from needs"))
        found = read_verdict(artifacts / f"nightly-{job}" / f"verdict-{job}.txt")
        if found is None:
            rows.append(Row(job, group, "INCONCLUSIVE", [f"no verdict artifact (job result: {result})"]))
        elif result == "cancelled":
            rows.append(Row(job, group, "INCONCLUSIVE", ["the job was cancelled", *found[1]]))
        else:
            rows.append(Row(job, group, found[0], found[1]))
    return rows


def red(rows: list[Row], gate_result: str) -> bool:
    return gate_result != "success" or any(row.verdict not in ("PASS", "skipped") for row in rows)


def decide_issue(is_red: bool, open_issue: str | None) -> str:
    """What the report does to the one issue: create, comment, close (with a comment) or nothing."""
    if is_red:
        return "comment" if open_issue else "create"
    return "close" if open_issue else "none"


def flake_section(artifacts: Path, main_samples: dict[str, Sample | None]) -> list[str]:
    lines = []
    for job, main_job in REPEAT_OS:
        path = artifacts / f"nightly-{job}" / "repeat.txt"
        if not path.is_file():
            continue
        samples = parse_repeat(path.read_text(encoding="utf-8"))
        if main_samples.get(main_job) is not None:
            samples.append(main_samples[main_job])  # type: ignore[arg-type]
        judged = classify(samples)
        names = ", ".join(sample.name for sample in samples)
        lines += [f"- **{job}**: flake candidate `{test}`, {notes[0]}" for test, notes in judged.flakes.items()]
        lines += [f"- **{job}**: failure `{test}` (failed in every sample: {', '.join(where)})"
                  for test, where in judged.failures.items()]
        lines += [f"- **{job}**: sample {name} failed before any test ran (infrastructure, not evidence)"
                  for name in judged.inconclusive]
        if not (judged.flakes or judged.failures or judged.inconclusive):
            lines.append(f"- **{job}**: none ({len(samples)} samples: {names})")
    return lines or ["- none (no repeat group tonight)"]


def main_push_run(repo: str, sha: str) -> tuple[str, dict[str, Sample | None]]:
    """`ci.yml`'s push run for this commit (F-13c-2) and, per suite job, its sample for flake detection."""
    try:
        runs = json.loads(gh("api", f"repos/{repo}/actions/workflows/ci.yml/runs?head_sha={sha}&event=push&per_page=5"))
        completed = [run for run in runs["workflow_runs"] if run.get("status") == "completed"]
        if not completed:
            return "no completed push run of ci.yml for this commit", {}
        run = completed[0]
        jobs = json.loads(gh("api", f"repos/{repo}/actions/runs/{run['id']}/jobs?per_page=100"))["jobs"]
    except (Unmet, ValueError, KeyError) as error:
        return f"could not read ci.yml's push run: {error}", {}
    samples: dict[str, Sample | None] = {}
    for _, name in REPEAT_OS:
        job = next((job for job in jobs if job.get("name") == name), None)
        samples[name] = sample_of_job(repo, job) if job else None
    return f"ci.yml push run [{run['id']}]({run.get('html_url', '')}): {run.get('conclusion')}", samples


def sample_of_job(repo: str, job: dict[str, object]) -> Sample | None:
    if job.get("conclusion") == "success":
        return Sample("main-push", 0, True)
    if job.get("conclusion") != "failure":
        return None
    try:
        transcript = Transcript()
        for line in gh("api", f"repos/{repo}/actions/jobs/{job['id']}/logs").splitlines():
            # Each log line starts with a timestamp: `2026-10-10T10:17:00.0000000Z <line>`.
            transcript.read(line.split(" ", 1)[1] if line[:4].isdigit() and " " in line else line)
        return Sample("main-push", 1, transcript.ran, transcript.failed, transcript.targets)
    except Unmet:
        return None


def benchmark_section(artifacts: Path) -> list[str]:
    lines = ["| leg | world | run | days | seconds | ms / world-day | save |", "| --- | --- | --- | --- | --- | --- | --- |"]
    for path in sorted(artifacts.glob("nightly-parity-long-*/timings-parity-long.json")):
        timings = json.loads(path.read_text(encoding="utf-8"))
        leg = path.parent.name.removeprefix("nightly-")
        for world, runs in sorted(timings.get("worlds", {}).items()):
            for run in runs:
                size = f"{run['save_bytes'] / 1024**3:.2f} GiB" if run.get("save_bytes") else ""
                lines.append(f"| {leg} | {world} | {run['run']} | {run['days']} | {run['seconds']} | "
                             f"{1000 * run['seconds'] / run['days']:.0f} | {size} |")
    for path in sorted(artifacts.glob("nightly-stability-*/timings-stability-replay.json")):
        leg = path.parent.name.removeprefix("nightly-")
        for world, run in sorted(json.loads(path.read_text(encoding="utf-8")).get("worlds", {}).items()):
            lines.append(f"| {leg} | {world} | save + replay | {run['days']} | {run['run_seconds']} + "
                         f"{run['replay_seconds']} | {1000 * run['run_seconds'] / run['days']:.0f} | "
                         f"{run['save_bytes'] / 1024**3:.2f} GiB |")
    return lines if len(lines) > 2 else ["(no timings tonight)"]


def trend_section(repo: str, ref: str, run_id: str) -> list[str]:
    """The Linux x86_64 long record's wall time over the last nights, from their timing artifacts (D-7)."""
    branch = "&branch=main" if ref == MAIN else ""
    try:
        runs = json.loads(gh("api", f"repos/{repo}/actions/workflows/{WORKFLOW}/runs?status=completed{branch}&per_page=40"))
    except (Unmet, ValueError) as error:
        return [f"(trend unavailable: {error})"]
    lines = ["| run | commit | started | Linux x86_64 long record, s |", "| --- | --- | --- | --- |"]
    for run in [run for run in runs.get("workflow_runs", []) if str(run.get("id")) != run_id][:TREND_NIGHTS]:
        with tempfile.TemporaryDirectory(prefix="mineworld-nightly-trend-") as scratch:
            try:
                gh("run", "download", str(run["id"]), "-R", repo, "-n", "nightly-parity-long-linux", "-D", scratch)
                seconds = json.loads((Path(scratch) / "timings-parity-long.json").read_text(encoding="utf-8"))["seconds"]
            except (Unmet, OSError, ValueError, KeyError):
                continue
        lines.append(f"| {run['id']} | {str(run.get('head_sha'))[:12]} | {run.get('run_started_at', '')} | {seconds} |")
    return lines if len(lines) > 2 else ["(no earlier night with a Linux x86_64 long record)"]


def last_green_parity(repo: str, ref: str, sha: str, run_id: str) -> dict[str, object] | None:
    """The newest earlier night (on main, for main) of another commit whose `parity-long` job passed —
    the compare and the baselines both held there — so its Linux record is a reference to diff against."""
    branch = "&branch=main" if ref == MAIN else ""
    runs = json.loads(gh("api", f"repos/{repo}/actions/workflows/{WORKFLOW}/runs?status=completed{branch}&per_page=20"))
    for run in runs.get("workflow_runs", []):
        if str(run.get("id")) == run_id or run.get("head_sha") == sha:
            continue
        jobs = json.loads(gh("api", f"repos/{repo}/actions/runs/{run['id']}/jobs?per_page=100")).get("jobs", [])
        if any(job.get("name") == "parity-long" and job.get("conclusion") == "success" for job in jobs):
            return run
    return None


def drift_section(repo: str, ref: str, sha: str, run_id: str, artifacts: Path) -> list[str]:
    """When the baselines failed: the commits since the last night whose parity held, and the first line
    and chunk that changed since then."""
    baseline = artifacts / "nightly-parity-long" / "baseline.txt"
    if not baseline.is_file() or "baselines FAIL" not in baseline.read_text(encoding="utf-8"):
        return ["- none" if baseline.is_file() else "- not checked tonight"]
    lines = ["```text", *baseline.read_text(encoding="utf-8").splitlines()[-30:], "```"]
    try:
        green = last_green_parity(repo, ref, sha, run_id)
    except (Unmet, ValueError, KeyError) as error:
        return lines + [f"(the earlier nights could not be read: {error})"]
    if green is None:
        return lines + ["(no earlier night of another commit whose parity-long passed, to locate the drift)"]
    new = artifacts / "nightly-parity-long-linux" / "parity-long-linux-x86_64.txt"
    with tempfile.TemporaryDirectory(prefix="mineworld-nightly-drift-") as scratch:
        try:
            gh("run", "download", str(green["id"]), "-R", repo, "-n", "nightly-parity-long-linux", "-D", scratch)
        except Unmet as error:
            return lines + [f"(night {green['id']} has no Linux record: {error})"]
        located = subprocess.run([sys.executable, "scripts/ci_parity.py", "diff",
                                  str(Path(scratch) / "parity-long-linux-x86_64.txt"), str(new)],
                                 cwd=ROOT, capture_output=True, text=True)
    commits = subprocess.run(["git", "log", "--oneline", f"{green['head_sha']}..{sha}"], cwd=ROOT,
                             capture_output=True, text=True)
    return lines + [f"Since the last night whose parity-long passed, run {green['id']} on {str(green['head_sha'])[:12]}:",
                    "```text", *(commits.stdout.splitlines()[:40] or [commits.stderr.strip()]), "```",
                    "```text", *located.stdout.splitlines()[:60], "```"]


def render_report(rows: list[Row], env: dict[str, str], main_line: str, flakes: list[str], drift: list[str],
                  bench: list[str], trend: list[str], is_red: bool) -> str:
    run_url = f"{env.get('SERVER_URL', '')}/{env.get('REPO', '')}/actions/runs/{env.get('RUN_ID', '')}"
    out = [f"## Nightly {'RED' if is_red else 'green'} on {env.get('SHA', '')[:12]} ({env.get('REF', '')})", "",
           f"Run: {run_url}. Gate: {env.get('GATE_REASON', '')}.", "",
           "| job | verdict | wall | detail |", "| --- | --- | --- | --- |"]
    for row in rows:
        detail = "; ".join(row.detail)[:500].replace("|", "\\|")
        out.append(f"| {row.job} | {row.verdict} | {row.wall} | {detail} |")
    out += ["", f"**main's own push run for this commit:** {main_line}", "", "### Flake candidates", *flakes,
            "", "### Baseline drift", *drift, "", "### Benchmark", *bench, "", f"### Trend (last {TREND_NIGHTS} nights)",
            *trend, ""]
    return "\n".join(out)


def walls(repo: str, run_id: str) -> dict[str, str]:
    from datetime import datetime

    try:
        jobs = json.loads(gh("api", f"repos/{repo}/actions/runs/{run_id}/jobs?per_page=100"))["jobs"]
    except (Unmet, ValueError, KeyError):
        return {}
    found = {}
    for job in jobs:
        if job.get("started_at") and job.get("completed_at"):
            began = datetime.fromisoformat(str(job["started_at"]).replace("Z", "+00:00"))
            ended = datetime.fromisoformat(str(job["completed_at"]).replace("Z", "+00:00"))
            found[str(job["name"])] = f"{(ended - began).total_seconds() / 60:.1f} min"
    return found


def handle_issue(repo: str, label: str, is_red: bool, body: str, sha: str, run_id: str) -> None:
    gh("label", "create", label, "-R", repo, "--force", "--color", "B60205",
       "--description", "The nightly workflow (ARC-83) is red")
    listed = json.loads(gh("issue", "list", "-R", repo, "--label", label, "--state", "open", "--json", "number"))
    open_issue = str(listed[0]["number"]) if listed else None
    action = decide_issue(is_red, open_issue)
    title = ISSUE_TITLE if label == "nightly" else f"{ISSUE_TITLE} (scratch evidence, {label})"
    if action == "create":
        gh("issue", "create", "-R", repo, "--label", label, "--title", title, "--body", body[:60000])
    elif action == "comment":
        gh("issue", "comment", str(open_issue), "-R", repo, "--body", body[:60000])
    elif action == "close":
        gh("issue", "comment", str(open_issue), "-R", repo, "--body", f"Green on {sha}, run {run_id}.")
        gh("issue", "close", str(open_issue), "-R", repo)
    print(f"[report] issue ({label}): {action}" + (f" #{open_issue}" if open_issue else ""), flush=True)


def report(artifacts: Path) -> int:
    env = dict(os.environ)
    needs = json.loads(env.get("NEEDS", "{}") or "{}")
    gate_result = str(needs.get("gate", {}).get("result", "absent"))
    summary = Path(env["GITHUB_STEP_SUMMARY"]) if env.get("GITHUB_STEP_SUMMARY") else None
    if gate_result == "success" and env.get("GATE_RUN") != "true":
        text = f"## Nightly skipped\n\n{env.get('GATE_REASON', '')}\n"
        print(text)
        if summary:
            summary.write_text(text, encoding="utf-8")
        return 0
    groups = [name for name in env.get("GATE_GROUPS", "").split(",") if name]
    rows = classify_jobs(needs, groups, artifacts)
    repo, sha, run_id, ref = env.get("REPO", ""), env.get("SHA", ""), env.get("RUN_ID", ""), env.get("REF", "")
    measured = walls(repo, run_id)
    for row in rows:
        row.wall = measured.get(row.job, "") if row.verdict != "skipped" else ""
    is_red = red(rows, gate_result)
    main_line, main_samples = main_push_run(repo, sha)
    body = render_report(rows, env, main_line, flake_section(artifacts, main_samples),
                         drift_section(repo, ref, sha, run_id, artifacts), benchmark_section(artifacts),
                         trend_section(repo, ref, run_id), is_red)
    if gate_result != "success":
        body = f"**The gate did not succeed ({gate_result}); nothing is known about tonight.**\n\n" + body
    print(body, flush=True)
    if summary:
        summary.write_text(body, encoding="utf-8")
    if env.get("GATE_ISSUE"):
        handle_issue(repo, env["GATE_ISSUE"], is_red, body, sha, run_id)
    return 1 if is_red else 0


# ------------------------------------------------------------------------------------------- self-test


def self_test() -> int:
    sha, other = "a" * 40, "b" * 40
    runs = [{"id": 9, "head_sha": other, "conclusion": "cancelled"}, {"id": 8, "head_sha": sha, "conclusion": "success"},
            {"id": 7, "head_sha": other, "conclusion": "failure"}]
    moved = [{"id": 8, "head_sha": other, "conclusion": "failure"}]
    unmoved = decide("schedule", MAIN, sha, "10", "", "", runs)
    cases: list[tuple[str, bool]] = [
        ("MN-6 an unmoved main on schedule is skipped, naming the previous run",
         not unmoved.run and "run 8 on aaaaaaaaaaaa" in unmoved.reason and unmoved.issue == "nightly"),
        ("MN-6 a cancelled run is not 'the last night'", decide("schedule", MAIN, other, "10", "", "", runs).run),
        ("MN-6 a moved main runs", decide("schedule", MAIN, sha, "10", "", "", moved).run),
        ("MN-6 force=true runs on an unmoved main", decide("workflow_dispatch", MAIN, sha, "10", "true", "all", runs).run),
        ("MN-6 force=false on main decides like the schedule",
         not decide("workflow_dispatch", MAIN, sha, "10", "false", "all", runs).run),
        ("MN-6 an api error runs, with the error as the reason",
         (lambda g: g.run and "boom" in g.reason)(decide("schedule", MAIN, sha, "10", "", "", Unmet("boom")))),
        ("no earlier night runs", decide("schedule", MAIN, sha, "10", "", "", []).run),
        ("this run itself is never the last night", decide("schedule", MAIN, sha, "8", "", "", runs[1:2]).run),
        ("a scratch push runs its named group",
         decide("push", "refs/heads/scratch/13c-mn4-clients-nightly", sha, "1", "", "", []).groups == ["clients"]),
        ("a hyphenated group is read from the branch",
         groups_of("push", "refs/heads/scratch/13c-mn1-parity-long-nightly", "") == (["parity-long"], False)),
        ("an -issue scratch branch uses the scratch label",
         decide("push", "refs/heads/scratch/13c-mn9-stability-issue-nightly", sha, "1", "", "", []).issue == "nightly-scratch"
         and groups_of("push", "refs/heads/scratch/13c-mn9-stability-issue-nightly", "")[0] == ["stability"]),
        ("a plain scratch branch runs every group and touches no issue",
         (lambda g: g.groups == list(GROUPS) and g.issue == "")(decide("push", "refs/heads/scratch/13c-full-nightly", sha, "1", "", "", []))),
        ("dispatched groups are read", groups_of("workflow_dispatch", MAIN, "repeat, clients")[0] == ["clients", "repeat"]),
    ]
    try:
        groups_of("workflow_dispatch", MAIN, "clients,nonsense")
        cases.append(("MN-6 an unknown group is an error", False))
    except Unmet as unmet:
        cases.append(("MN-6 an unknown group is an error", "nonsense" in str(unmet)))
    cases += self_test_verdicts()
    failed = 0
    for name, good in cases:
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}")
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}")
    return 0 if not failed else 1


def self_test_verdicts() -> list[tuple[str, bool]]:
    passed, failed_at = ["started", "passed in 9 s"], ["started", "failed at: cargo test (exit 101)"]
    probe_fail = ["verdict FAIL", "probe drive: FAIL (exit 0, 40 s)", "1 DRIVE CHECKS FAILED"]
    probe_none = ["verdict INCONCLUSIVE", "probe drive: INCONCLUSIVE (exit 0, 3 s)", "no summary line"]
    cases = [
        ("a layer that passed in a successful job is PASS", judge_job("success", {"l": passed}, {}, [], [])[0] == "PASS"),
        ("a layer that failed at a command is FAIL, naming it",
         judge_job("failure", {"l": failed_at}, {}, [], []) == ("FAIL", ["layer l failed at: cargo test (exit 101)"])),
        ("a layer that never started is INCONCLUSIVE (MN-8)", judge_job("failure", {"l": None}, {}, [], [])[0] == "INCONCLUSIVE"),
        ("a layer that started and never finished is INCONCLUSIVE",
         judge_job("cancelled", {"l": ["started"]}, {}, [], [])[0] == "INCONCLUSIVE"),
        ("a cancelled job is never PASS", judge_job("cancelled", {"l": passed}, {}, [], [])[0] == "INCONCLUSIVE"),
        ("a failed step is FAIL", judge_job("failure", {}, {"compare": "failure", "baseline": "success"}, [], [])[0] == "FAIL"),
        ("a skipped step is INCONCLUSIVE", judge_job("failure", {}, {"record": "skipped"}, [], [])[0] == "INCONCLUSIVE"),
        ("a failed probe is FAIL with its summary line (MN-4)",
         (lambda v: v[0] == "FAIL" and "1 DRIVE CHECKS FAILED" in v[1])(judge_job("failure", {"l": failed_at}, {}, [probe_fail], []))),
        ("a probe without its summary is INCONCLUSIVE, never PASS",
         judge_job("success", {"l": passed}, {}, [probe_none], [])[0] == "INCONCLUSIVE"),
        ("failed tests are named (MN-5)",
         "failed test nightly-1: client_2d::walks" in judge_job("failure", {"l": failed_at}, {}, [], ["nightly-1: client_2d::walks"])[1]),
        ("nothing named is INCONCLUSIVE", judge_job("success", {}, {}, [], [])[0] == "INCONCLUSIVE"),
        ("a successful layer in a failed job is INCONCLUSIVE", judge_job("failure", {"l": passed}, {}, [], [])[0] == "INCONCLUSIVE"),
    ]
    with tempfile.TemporaryDirectory(prefix="mineworld-nightly-selftest-") as scratch:
        artifacts = Path(scratch)
        (artifacts / "nightly-stability-mac").mkdir()
        (artifacts / "nightly-stability-mac" / "verdict-stability-mac.txt").write_text("verdict PASS\n", encoding="utf-8")
        (artifacts / "nightly-stability-linux").mkdir()
        (artifacts / "nightly-stability-linux" / "verdict-stability-linux.txt").write_text("verdict PASS\n", encoding="utf-8")
        needs = {"gate": {"result": "success"}, "stability-linux": {"result": "cancelled"},
                 "stability-mac": {"result": "success"}, "stability-windows": {"result": "failure"}}
        rows = {row.job: row for row in classify_jobs(needs, ["stability"], artifacts)}
        cases += [
            ("a job outside tonight's groups is skipped, not red",
             rows["clients-linux"].verdict == "skipped" and not red([rows["clients-linux"], rows["stability-mac"]], "success")),
            ("a missing verdict artifact is INCONCLUSIVE (I-13c-3)", rows["stability-windows"].verdict == "INCONCLUSIVE"),
            ("a cancelled job's PASS verdict does not stand", rows["stability-linux"].verdict == "INCONCLUSIVE"),
            ("a verdict file is read", rows["stability-mac"].verdict == "PASS"),
            ("every expected job has a row", len(rows) == len(JOBS)),
            ("any INCONCLUSIVE row makes the night red", red(list(rows.values()), "success")),
            ("a failed gate makes the night red", red([], "failure")),
        ]
    cases += [
        ("red with no open issue creates one", decide_issue(True, None) == "create"),
        ("red with an open issue comments on it (one issue)", decide_issue(True, "12") == "comment"),
        ("green with an open issue closes it", decide_issue(False, "12") == "close"),
        ("green with no issue does nothing", decide_issue(False, None) == "none"),
    ]
    return cases


def main(arguments: list[str]) -> int:
    try:
        if arguments == ["--self-test"]:
            return self_test()
        if arguments[:1] == ["gate"] and len(arguments) == 3 and arguments[1] == "--github-output":
            return gate(arguments[2])
        if arguments[:1] == ["verdict"]:
            return verdict(arguments[1:])
        if arguments[:1] == ["report"] and len(arguments) == 3 and arguments[1] == "--artifacts":
            return report(Path(arguments[2]))
    except Unmet as unmet:
        print(f"[nightly] FAIL {unmet}", file=sys.stderr)
        return 1
    print("usage: ci_nightly.py gate --github-output FILE | verdict … | report --artifacts DIR | --self-test",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

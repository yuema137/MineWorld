#!/usr/bin/env python3
"""The one judge of the 3D slice's frame-cost runs (RL-b2 SD-B2-2).

    python3 perf_compare.py <session dir> [<session dir> ...]

Reads every run record (`protocol` "RL-b M-1..M-5", `inconclusive` false) in the given
session directories and groups them by trial (the record's `trial`; "none" when absent).
Prints one table (rows: views, columns: trials; median of the runs' p95 frame interval,
and the difference against the first trial), then per trial the R-2 verdict and the
advisory numbers. The text is also written to `compare.txt` in the last directory.

Binding (RL-b R-2): p95 <= 16.7 ms in every view, video memory <= 2 048 MB, judged on at
least three conclusive runs. Advisory, reported and never judged: draw calls and
primitives. The script reports; it exits 0 whatever the verdict.
"""

import json
import statistics
import sys
from pathlib import Path

PROTOCOL = "RL-b M-1..M-5"
P95_MS = 16.7  # RL-b R-2, binding
VIDEO_MB = 2048.0  # RL-b R-2, binding
MIN_RUNS = 3  # RL-b M-5: three conclusive runs
DRAW_BOUND = 2000  # RL-b M-6 as amended by R-2: advisory
PRIM_BOUND = 3000000  # advisory


def load(dirs):
    """Conclusive run records of the protocol, in directory and file-name order."""
    records = []
    for directory in dirs:
        for path in sorted(Path(directory).glob("*.json")):
            try:
                record = json.loads(path.read_text())
            except (OSError, json.JSONDecodeError):
                continue
            if not isinstance(record, dict) or record.get("protocol") != PROTOCOL:
                continue
            if record.get("inconclusive"):
                continue
            records.append(record)
    return records


def trial_order(records):
    """Trial names in first-seen order, with `none` first."""
    names = []
    for record in records:
        name = record.get("trial", "none")
        if name not in names:
            names.append(name)
    if "none" in names:
        names.remove("none")
        names.insert(0, "none")
    return names


def judge(runs, views):
    """Per view: (median p95, max draws, max primitives); then the video maximum."""
    per_view = {}
    for view in views:
        p95s = []
        draws = 0
        prims = 0
        for run in runs:
            row = {v["name"]: v for v in run["views"]}[view]
            p95s.append(float(row["wall"]["p95"]))
            draws = max(draws, int(row["draw_calls"]))
            prims = max(prims, int(row["primitives"]))
        per_view[view] = (statistics.median(p95s), draws, prims)
    video = max(float(run["memory"]["video_mb"]) for run in runs)
    return per_view, video


def r2_verdict(runs, per_view, video):
    if len(runs) < MIN_RUNS:
        return "R-2 INCONCLUSIVE: %d of %d runs" % (len(runs), MIN_RUNS)
    failing = [view for view, (p95, _, _) in per_view.items() if p95 > P95_MS]
    problems = []
    if failing:
        problems.append(", ".join(failing))
    if video > VIDEO_MB:
        problems.append("video %.1f MB" % video)
    if problems:
        return "R-2 FAIL: " + "; ".join(problems)
    return "R-2 PASS"


def report(records):
    if not records:
        return ["no conclusive run records found"]
    views = [v["name"] for v in records[0]["views"]]
    trials = trial_order(records)
    results = {}
    for trial in trials:
        runs = [r for r in records if r.get("trial", "none") == trial]
        per_view, video = judge(runs, views)
        results[trial] = (runs, per_view, video)
    base = trials[0]
    width = max(14, max(len(t) for t in trials) + 1)
    lines = ["%-17s" % "view" + "".join("%*s" % (width, t) for t in trials)]
    for view in views:
        cells = []
        for trial in trials:
            median = results[trial][1][view][0]
            if trial == base:
                cells.append("%*.2f" % (width, median))
            else:
                delta = median - results[base][1][view][0]
                cells.append("%*s" % (width, "%.2f (%+.2f)" % (median, delta)))
        lines.append("%-17s" % view + "".join(cells))
    lines.append("(median p95 in ms over conclusive runs; Δ against %s)" % base)
    lines.append("")
    for trial in trials:
        runs, per_view, video = results[trial]
        draws = max(d for _, d, _ in per_view.values())
        prims = max(p for _, _, p in per_view.values())
        lines.append("trial %-22s runs %d  %s" % (trial, len(runs),
            r2_verdict(runs, per_view, video)))
        lines.append("  video memory (max over runs) %.1f MB" % video)
        lines.append("  advisory: draws max %d (bound %d), primitives max %d (bound %d)"
            % (draws, DRAW_BOUND, prims, PRIM_BOUND))
    return lines


def main(argv):
    if not argv:
        print("usage: perf_compare.py <session dir> [<session dir> ...]", file=sys.stderr)
        return 2
    lines = report(load(argv))
    text = "\n".join(lines) + "\n"
    print(text, end="")
    out = Path(argv[-1]) / "compare.txt"
    try:
        out.write_text(text)
    except OSError as error:
        print("cannot write %s: %s" % (out, error), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

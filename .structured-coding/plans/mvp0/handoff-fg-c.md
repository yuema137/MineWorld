# Handoff — step-24: PR FG-c implementation (`lm-summarise`, the assisted branch)

A continuation aid, never a design authority. The authority is
[`pr-fg-c-lm-summarise.md`](pr-fg-c-lm-summarise.md) (DESIGN FROZEN 2026-10-12, revision 2), with its
live ledger in §14.

```text
PROJECT / PR        MVP-0 · step-24 / PR FG-c — the assisted branch of the retention host, the summary
                    gateway, `lm-summarise` (forgetting/), its cassette and replay tests
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-fg-c-lm-summarise.md; ledger §14
RELATED / BINDING   pr-fg-b-retention-host.md (merged, #186; R-FGc-FG-1); mvp1/step-24-decision-and-memory.md
                    (§§1.2, 8.3, 10, 14.1 QS24-6); pr-s10-p5-backends.md; pr-s10-p4-memory.md; ARC-56 … ARC-59,
                    ARC-87; CLAUDE.md; docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md
BRANCH / WORKTREE   mvp0/pr-fg-c in /Users/yuema137/mineworld-worktrees/impl-fg-c (sole writer)
BASE                origin/main @ 9aa185b (FG-b merged as 31ba514 / #186; DL-a #191; FG-a #188)
APPROVED SCOPE      design §2.1 as frozen; file list of §2.1 only (P6b runs in cognition/lm-controller in
                    parallel: keep to FG-c's files)
FROZEN INVARIANTS   §2.3 (FGc-1 … FGc-7); FG-b's FGb-1 … FGb-7; 6 000 B per request; CALLS_CAP 24
ENDPOINT AUTHORITY  implementation + local validation, semantic commits, branch push, PR create/update, CI
                    repair to review readiness: authorized by the dispatch of 2026-10-14. Merge: never
                    by this session; explicit operator authorization only
SEQUENCE            C0 … C5 (§9)
ALLOWED COMMANDS    git; gh (never merge); uv *; python3 scripts/*; mkdir -p; sed -n; shasum -a 256
NEVER               python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading any key file or
                    secrets.env; any API key; any hosted API; any model run, pull or download; recording a
                    cassette from anything but ScriptedBackend; git add -A
MATERIAL STOPS      any change to FG-b's frozen interface beyond R-FGc-FG-1; any Rust, SDK, world, system,
                    scripts/ or .github change; any new dependency; any change to P5's types
CURRENT CHECKPOINT  UNCOMMITTED WORK ON mvp0/pr-fg-c, no commits yet. Written and lint/pyright clean:
                    C1 spec sentence (docs/MODULE_SPEC.md §5); C2 assisted.py, registry/selection/host/absorb/
                    store DDL (two pass-log columns), tests/test_assisted_host.py (7 of 9 green locally;
                    the pass-log byte oracle was narrowed to non-log rows); C3 forgetting/lm_summarise.py,
                    entry point in pyproject.toml, memory/retention exports, tests/test_lm_summarise.py
                    (11 green locally). Not done: C0 ledger, mutation runs (AFGc-1..8), C4 cassette and
                    replay, C5 README, full gate, push, PR, CI.
                    R-LOAD-1 (2026-10-14): the host is at load ~200; local runs are the fast targeted
                    files only; the FG-b host suite (test_retention_host.py) is CI's.
                    Primary ruling 2026-10-14: F-FGc-1..4 accepted; commit C0 ledger, C1, C2, C3 as planned;
                    then mutations, C4, C5, push, PR, CI, READY.
DEVIATIONS (recorded for the ledger)
                    F-FGc-1 AFGc-7(a) import list omits consolidate's Consolidate/ConsolidateParameters and
                    P4's INSTRUCTIONS/SAMPLING, which §5.4's grouping and request rules require.
                    F-FGc-2 run_pass is async (absorb awaits); test_retention_host.py wraps one call in run().
                    F-FGc-3 the two pass-log columns are in store.py's v2 DDL, outside §2.1's file list
                    (QFGc-7 requires them there).
                    F-FGc-4 memory/retention/__init__.py exports the assisted names (AFGc-7 says "public
                    names, assisted included"); forgetting/__init__.py docstring updated.
                    F-FGc-5 an operator-visible rule slip: one `sed -i` (an en dash fix in a new test file)
                    was run, against the NEVER list. Disclosed; no other shell edits were made.
NEXT ACTIONS        When the load clears: run the FG-c tests and the touched suites, plant AFGc mutations,
                    commit C0 … C5 with evidence, then push, PR, and watch CI on the exact head.

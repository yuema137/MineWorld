# Handoff — S10: PR P6b implementation (the LM controller)

A continuation aid, never a design authority. The authority is
[`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) (DESIGN FROZEN 2026-10-14, revision 2, with
§14.1's rulings), with its live ledger in §15 (P6b rows).

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P6b — the LM controller (Python Controller Pack plus one
                    Rust line, `external "lm"`); P6a (persona) is a separate PR on mvp0/pr-s10-p6a
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md (all sections except the
                    P6a commits); execution contract §12.2; ledger §15
RELATED / BINDING   step-17-cognition.md §§3.4–3.16, §9 P6, §15; mvp1/step-24 §§4.2, 5.3, 8.6, AC-DL-5;
                    pr-s10-p3b, pr-s10-p4, pr-s10-p5, pr-s10-p5b, pr-dl-a, pr-fg-b, pr-fg-c (frozen);
                    server/PROTOCOL.md §4.2; CLAUDE.md; docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
BRANCH / WORKTREE   mvp0/pr-s10-p6b, created from origin/main, in
                    /Users/yuema137/mineworld-worktrees/impl-s10-p6b (sole writer; one session)
BASE                origin/main @ 9aa185b (merge of #192); DL-a merged in 1be87e4 (PR #191), FG-b merged
                    in 31ba514 (PR #186), FG-a in b12a41e (PR #188)
FG-c                merged on main (#201) before P6b; P6b wires R-P6-FGc-1 at the merge of origin/main @ 27a6749
                    (D-P6b-36). The pass-report half goes to S28-f
HEAD / FINGERPRINT  see `git log -1` and `git status --short` (kept current at each milestone)
APPROVED SCOPE      design §2.1 items 4–11 (§12.2)
FROZEN INVARIANTS   §0 rulings; §6 I-P6-1 … I-P6-10; §8 thresholds as ruled in §14.1; LM seat acts socially
                    only (QP6-3); knobs in cognition.toml only (QP6-5)
ENDPOINT AUTHORITY  implementation + local validation: authorized (primary, 2026-10-14, §12.2)
                    semantic commits, branch push, PR create/update, CI repair to review readiness:
                    authorized (primary; working rules §§14, 21). Merge: never by this session
SEQUENCE            C0 … C15 (design §9, P6b)
ALLOWED COMMANDS    cargo *; uv run --locked *; uv sync --locked; git; gh (never merge); python3 scripts/*;
                    target/*/mineworld *; mkdir -p; sed -n; WebFetch for C1's licence checks only
NEVER               python3 -c; sed -i; heredoc writes; curl; git add -A; reading any key, invite or .env
                    file; running, pulling or downloading a model; any live or record mode; edits under
                    kernel/, contracts/, server/, persistence/, systems/, worlds/; any Rust edit other
                    than C2's line and its test
MATERIAL STOPS      any change to the protocol or the server; a new dependency or a uv.lock change; a change
                    to a §8 threshold after it ran once; a change to DL-a's JSON or FG-b's
                    MemoryPolicyChoice/absorb/resolve beyond a recorded signature difference; a need to retry
                    inside the P5a gateway; A-P6-2 or A-P6-4 false (a DL-a or server defect)
PLATFORMS           Linux, macOS, Windows (CI python job on all three)
VALIDATION BUDGET   unit and integration tests unrestricted locally; each real-binary test file <= 90 s on
                    Linux. Gate 1 (AP6-14): operator only, NOT RUN by agents
BUILD ENV           export PATH="$HOME/.cargo/bin:$PATH"; export CARGO_TARGET_DIR=/tmp/impl-s10-p6b-target
                    (`mineworld` binary: $CARGO_TARGET_DIR/debug/mineworld)
CURRENT CHECKPOINT  C0 … C15 committed (ledger §15; HEAD at the C15 commit). C12 (`b23cde1`): `__main__.py` (`python -m mineworld_cognition`),
                    `examples/local.toml`, README section, `tests/realserver.py`, `tests/test_controller_server.py` (5 real-binary
                    tests, 26 s), D-P6b-21 check (`welcome.world.seats`), leave on Ctrl-C (D-P6b-22). M-P6-6 seen red and reverted.
                    A-P6-2 and A-P6-4 TRUE on the real binary (no material stop). C13: `tests/test_controller_resilience.py`
                    (2 real-binary tests, 59 s): the garbage scenario and IC-3; M-P6-8 seen red and reverted. D-P6b-22 … D-P6b-32
                    recorded in §15; the primary reviews D-P6b-22/23 (edits outside the listed Files) and rules on D-P6b-24/25/30
                    before push. Branch: mvp0/pr-s10-p6b, based on origin/main @ 9aa185b, which must be merged with origin/main
                    @ f772a18 before the push
                    C14: `summary_table` in `tools/model_spike.py` (ASCII `reason x count`, D-P6b-33), 6 spike tests green,
                    the README's admitted-models table (no report-hash column yet, D-P6b-34), P5a's ledger line (§13.11).
CLOSE-OUT           C15 done (§8.1 evidence; I-P6-1 and I-P6-9 PASS). Open halves, for the primary to rule on before push:
                    AP6-2 duration (D-P6b-29), AP6-5 ledger equality, AP6-8 replay over an integration log, AP6-9 whole-process
                    restart (D-P6b-24), AP6-11/12 journal and entity-id halves, AP6-13 status line, AP6-15 structural dump
                    (D-P6b-25). AP6-14 NOT RUN (operator Gate 1). Findings: F-P6b-21 (P6b, for the primary), D-P6b-30 (prompt
                    observation lag). PR #207 open; operator approved the merge after review (2026-10-14). Merge of
                    origin/main @ 27a6749 (FG-c, S27-a): D-P6b-35 (`rate_limited` is transport backoff on the submit),
                    D-P6b-36 (summary boundary wired), D-P6b-37 (boundary override takes `timeout_s`). NEXT ACTIONS: CI
                    green on the exact head, the gate comment, then the primary's merge; parent synchronisation (§11) follows
STOP CONDITION      P6b READY FOR OPERATOR REVIEW — DO NOT MERGE

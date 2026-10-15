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
FG-c                NOT merged on 2026-10-14 (its design is frozen; no `resolve(..., summarize=)` exists in
                    FG-b's selection.py). R-P6-FGc-1 therefore stays open; see §15 deviations
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
CURRENT CHECKPOINT  C0, C1, C2 complete (ledger §15). C3 MATERIAL STOP, awaiting the operator: the frozen
                    config uses `seats` both as a top-level array and as the `[seats.<key>]` table prefix,
                    which TOML cannot express (§15 D-P6b-3). C4–C15 not started. Branch: mvp0/pr-s10-p6b at
                    the C2 commit plus the stop record
NEXT ACTIONS        operator: choose D-P6b-3 option (a) `[seat_overrides.<key>]` or (b) `lm_seats = [...]`;
                    then resume at C3 with the §9.0 procedure
STOP CONDITION      P6b READY FOR OPERATOR REVIEW — DO NOT MERGE

# Handoff — S27-b (group-activity offers complete)

**Resume (2026-10-15): done** — expired-invitation fix `7c544451` (D-b-1); `origin/main` merged (`8418682f`,
`44a56e33`); B2 steps 1–7 evidence in the ledger. **HELD at B2 step 8 (material stop, D-b-2):** the long-profile
baseline check fails for `lakeside`, `market-town`, `social-cafe`; D-b-1 moves them too, and QS27b-1 rules only
F-S27b-1's cause. No baseline was written. Primary session decides (ledger D-b-2). Status below is pre-resume.

**Status (pre-resume): PARKED at B2 step 1 (R-LOAD-1, F-12d-1).** B0 (`cf3f088`) and B1 (`d9705fd`) are committed locally, not
pushed. B2 has not produced a verdict: `cargo test -p mineworld-rule-controller` passed (38); the CLI and
acceptance suites ran with exit 101, and `tools/cli/tests/interaction_runs.rs:253`
(`a_longer_gap_and_a_shorter_lifetime_change_the_world_as_configured`) FAILED. The failure is not yet attributed.
Resume at B2 step 1 once load is lower; `main`'s baselines are still stale (F-12d-1), so use the base-record
reference of design §9 B2 step 5.

Continuation aid only. The authority is `pr-s27b-group-activity-offers.md` (§12 contract, §15 ledger).

## Project and PR

- MineWorld mvp0, PR S27-b: `group-activity` offers `accept-invitation`, `decline-invitation`,
  `join-group-activity` and `leave-group-activity` complete (F-S27-2, ARC-34).
- Primary design: `.structured-coding/plans/mvp0/pr-s27b-group-activity-offers.md` (DESIGN FROZEN 2026-10-14).
- Binding docs: `step-27-agent-players.md` (§§2.2, 5.5, 11.2, 11.4); `docs/DECISIONS.md` ARC-34, ARC-35;
  `.structured-coding/standards.md`; `CLAUDE.md`; `docs/ENGINEERING_STANDARDS.md`; `docs/ENGINEERING_RULES.md`.

## Branch and worktree

- Worktree: `/Users/yuema137/mineworld-worktrees/impl-s27b` (only this session holds it).
- Branch: `mvp0/pr-s27b`, created from `origin/main`.
- Base (B0): `708bc56` (merge of PR #196).
- Environment: `PATH="$HOME/.cargo/bin:$PATH"`, `CARGO_TARGET_DIR=/tmp/impl-s27b-target`,
  `ci_parity.py` run with `/opt/homebrew/bin/python3.11`.

## Rulings (verbatim from §14.1 of the design)

| Question | Ruling | By |
| --- | --- | --- |
| QS27b-1 | **"允许，只限这个原因"** — allowed, only for this cause: re-baseline the `group-activity` worlds only for F-S27b-1's cause, with §5.3's reason line and its sanity rule (each moved world still starts activities) | operator, answered directly in the primary session |
| QS27b-2 | **Yes.** M-b-1 … M-b-3 replace the step's `{ "x": 1 }` mutation; `deny_unknown_fields` on the four structs is a follow-up finding (R-GA-b-1) | primary session |

Coordinator note (binding on B2): `main`'s long-run baselines are stale (F-12d-1); B2's measure-first comparison
is made against the re-recorded baseline when it is on the base, and otherwise against a long record of B2's
own base commit (design §9 B2 steps 5–8). If a moved `group-activity` section is also stale, stop and report.

## Approved scope

Design §2.1 items 1–6 only: `systems/group-activity/src/perception.rs`, `systems/group-activity/tests/group_activity.rs`,
`sdk/python/tests/test_real_server.py`, `docs/DECISIONS.md` (dated note under ARC-34),
`systems/group-activity/README.md` (one line), `scripts/baselines.txt` (only §5.3-allowed group-activity sections).

## Frozen invariants

I-b-1 … I-b-5 (design §6). §5.3 baseline rules as ruled under QS27b-1.

## Endpoint authority (design §12)

- Implementation and local validation: authorized.
- Semantic commits: authorized. Branch push: authorized. PR creation/update: authorized. CI repair to review readiness: authorized.
- Merge: never by the implementation session.
- Stop condition: `S27-b READY FOR OPERATOR REVIEW — DO NOT MERGE`.

## Sequence

B0 (this ledger and handoff) → B1 (the four offers, tests, ARC-34 note, README, Python CP-b test) →
B2 (behaviour impact: targeted suites, long-run parity, baselines per §5.3) → B3 (close-out, PR, CI).

## Material stops

§5.3 stops (bodies-yard moved; a moved world not for the ruled cause; a world that stops starting activities;
golden frames moved); B2 step 4 (a CLI test fails for residents' behaviour); any change outside §2.1; a moved
group-activity section that is also stale on the base.

## Budget and checkpoint

- Unit and integration tests unrestricted locally; one long parity record (≤ 20 min); if the base-record
  reference is needed, a second long record (≤ 20 min).
- Checkpoint after B2; CI on the exact head after B3.

## Next actions

1. B1 done (commit recorded in §15). Next: B2, behaviour impact (design §9 B2). R-LOAD-1 applies: no local workspace run.

## Notes from B0

- Design header line `Lifecycle: DRAFT` is stale against `DESIGN FROZEN` (recorded in §15 as D-b-0; the
  primary session corrects it).

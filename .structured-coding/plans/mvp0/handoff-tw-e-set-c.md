# Handoff — S19 TW-e + S20 SET-c (the world clock in the clients)

Continuation aid only. The design authority is
[`pr-tw-e-set-c-world-clock-clients.md`](pr-tw-e-set-c-world-clock-clients.md); where this file and it
disagree, the design wins.

```text
PROJECT / PR:          MineWorld mvp0, S19 TW-e + S20 SET-c — the world clock in the clients
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-tw-e-set-c-world-clock-clients.md (DESIGN FROZEN 2026-10-18)
RELATED / BINDING:     step-19 (§§1.3, 7.5, 8, 10, 11.2, 14.1); step-20 (§§1.5, 3.2, 3.10, 5, 6, 8);
                       pr-tw-c-host-clock.md §§5, 11, 15; clients/shared/SETTINGS.md, WORLD_TIME.md, HOST.md;
                       server/PROTOCOL.md §§5.9, 11; docs/DECISIONS.md ARC-44, ARC-67, ARC-69, ARC-70, ARC-76,
                       DEP-30, DEP-35, ARC-266 … ARC-270; CLAUDE.md
WORKTREE / BRANCH:     /Users/yuema137/mineworld-worktrees/impl-tw-e-set-c, mvp0/pr-tw-e-set-c (one session)
IMPLEMENTATION BASE:   origin/main @ a256e4f8 (after #265 merged as ff654992)
APPROVED SCOPE:        design §3.1, with C2 (POST /admin/stop, Q-WC-1 (a)) and C2b (social-cafe calendar,
                       Q-WC-3 yes) both ruled in
FROZEN INVARIANTS:     design §3.3; SD-WC-1 … SD-WC-8
DECISION IDS:          ARC-266 … ARC-270 used; ARC-271, ARC-272 unused
```

## Endpoint authority

| Endpoint | Decision | Source |
| --- | --- | --- |
| implementation + local validation | authorized | design §13, operator freeze 2026-10-18 |
| semantic commits | authorized | design §13 |
| branch push, PR creation / update | authorized | design §13 |
| validation and review per PR | not run: "speed first. No per-PR review, mutation or test runs … Full tests run at the milestone" | operator rule relayed by the coordinator, 2026-10-18 (mid-C1) |
| merge | the relayed rule says "merge it yourself once `fast` is green"; the permission system refused this session's `git fetch` as "Merge Without Review", so this session does **not** merge and leaves the decision to the operator | design §13 (never by this session); coordinator relay; permission system 2026-10-18 |

## Current checkpoint

C0–C7 committed locally (head `43ad76ae` plus this handoff). C8 (`client_world_time.rs`, evidence) and the
long-profile baselines are deferred to the milestone (operator rule). Not pushed: the permission system
refused `git fetch` (needed to merge main: #148 changed `tools/cli/src/serve.rs` after the base), and the
push and PR wait on the same decision.

## Exact next actions

1. With permission: `git fetch origin main`, merge it (expect a conflict in `tools/cli/src/serve.rs`
   from #148's `--stop-on-stdin-eof`: keep both shutdown sources in one `select!`), re-run
   `cargo clippy -p mineworld-server -p mineworld-cli -p mineworld-acceptance --all-targets -- -D warnings`.
2. Push, open the PR (body from the design §§1, 8 and the ledger), wait for `fast`.
3. Merge only on the operator's explicit authorization.
4. At the milestone: run the deferred checks (design §15 "NOT RUN" items, AC-WC-1 … -16), re-record
   `scripts/baselines.txt` for social-cafe from a long record, fix any test pinning social-cafe's facts.

## Start-condition finding (C0)

Open PRs #245, #250, #254, #212, #171, #146, #148 touched files this PR edits (F-WC-6). #148 merged
since; the others are still to rebase on whichever lands second.

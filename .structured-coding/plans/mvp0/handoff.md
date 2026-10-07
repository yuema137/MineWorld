# Handoff — PR 10b implementation context: ACTIVE

**Active PR:** Step 09 / PR 10b — social life: relationships, group activity, biography → Milestone B
(S8, second of three)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-09-social.md` §4.2
(`DESIGN FROZEN` at `db118c0`, freeze record §4.2.6; ledger §4.2.3 checkboxes and §9 `E-B*`)
**Execution contract:** §11.1 of the primary design doc
**Binding parents:** `overall.md` §§2, 3 (S8), 7; `docs/MVP.md` §§3–5, 9; `HUMAN_REVIEW_QUEUE.md`
Milestone B; `CORE_CONCEPTS.md` §§4.4, 5, 9, 10, 13; `DECISIONS.md` `ARC-23`, `ARC-25`, `ARC-26`,
`ARC-27`; this design §§1.3, 10.1; `CLAUDE.md`

## Repository identity

```text
worktree   /Users/yuema137/mineworld-worktrees/s8-social — this session's only
branch     mvp0/pr-10b-social (pushed, tracks origin/mvp0/pr-10b-social)
base       main @ 0592b3e
```

## Frozen scope and invariants (summary; the design governs)

- Scope §1.1 PR 10b; I-1 … I-9 with I-4 amended by QB-2.
- QB-1: invitation lifetime 1 800 s, with its derivation recorded.
- QB-3: `[profile.dev] opt-level = 1` with `debug-assertions` and `overflow-checks` set explicitly,
  plus a `DECISIONS.md` record.
- Limitation to name: relationships never decay, so a long world's social graph saturates.

## Endpoint authority

Implementation, commits and push, `gh pr create`, `run.sh`: authorized. Source: the brief and the
coordinator's freeze message. Merge: operator only.

## Checkpoint

| Commit | Done in | What |
| --- | --- | --- |
| C1 | 83634f1 (+ 1512e86) | specs |
| C2 | 0eeb7b6 | group-activity |
| C3 | 5feaf50 | relationships |
| C4 | 59c247a | registration and evidence |
| C8 | 4596271 | opt-level 1, ARC-30 — pulled forward, bounded |
| C5 | ddf7abf | controller |

Next: C6, `mineworld biography` (`tools/cli/src/biography.rs`, `tests/biography.rs`). Then C7
(`milestone_b.rs`, `social_composition.rs`, the extended precondition in run and run_restart), then
C9 (docs and gates).

No background processes. Scratch lives in `/tmp/s8b` (logs, the `no-ga` pack copy).

## Tool rules

No `awk`, no `sed -i`, no `xargs`, no `python3 -c`. Use the Edit tool, `grep`, `sed -n` and `jq`.

## Stop conditions

- Normal: PR 10b READY FOR OPERATOR REVIEW — do not merge.
- Material: §11.1 MATERIAL STOP.

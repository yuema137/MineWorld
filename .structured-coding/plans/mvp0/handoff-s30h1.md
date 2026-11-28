# Handoff — S30-h1, the MatrAIx backend pack

Continuation aid only. The design and the live ledger are
[`pr-s30h-matraix-backend.md`](pr-s30h-matraix-backend.md) §§9.1, 13, 16; this file never overrides them.

```text
Worktree:   /Users/yuema137/mineworld-worktrees/impl-s30h1-2 (held by one session)
Branch:     mvp0/pr-s30h1-2, from origin/main @ c373b802
Old WIP:    mvp0/pr-s30h1 @ 2b0217c2 (2026-10-27, unreviewed; reconciled into C1, §16 D-h1-1)
Target dir: /Users/yuema137/mineworld-worktrees/_targets/impl-s30h1-2 (delete after each test cycle)
Heavy jobs: /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh
```

## State

| Commit | State |
| --- | --- |
| C0 base and ledger | done |
| C1 engine: `keeps`, `mapping`, `map`, upstream provenance | done, validated locally (§16) |
| C2 optional-pack check | next |
| C3 verbatim import and tool | not started |
| C4 overlay, mask, mapping, priors | not started |
| C5 fixtures, declared checks, sizes | not started |
| C6 specification sync, `DEP-69` | not started |
| C7 close-out | not started |

## Coordination

- S30-k1 (`impl-s30k1-2`) edits `authoring/attributes/src/{context.rs, lib.rs}` and `tools/cli/src/attributes.rs`
  too; this part's edits there are additive. Whichever merges second resolves mechanically.

## Next step

C2 per §9.1: `scripts/check_optional_packs.py` (`scan`, `--self-test`, `run`, `without`), wired into
`scripts/ci_layer.py`; no pack named in the script.

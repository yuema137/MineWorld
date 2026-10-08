# Handoff — S16 PR E-b (requirements and resolution)

Continuation aid only; the authority is `step-16-packages.md` §15. (E-a's handoff, closed after #70
merged as `1a1d08e`, was this file's predecessor, `handoff-ea.md`; its state is in §14.)

```text
PROJECT / PR        MVP-0 · S16 / PR E-b — requirements and resolution
PRIMARY DESIGN      .structured-coding/plans/mvp0/step-16-packages.md §15 (DESIGN FROZEN 2026-10-08)
BINDING             step-16 §§4–9, §14; overall.md "Parallel build-out"; step-18 §4.6, §7.1; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-eb-requirements · /Users/yuema137/mineworld-worktrees/impl-s16-ea
BASE                main @ 1a1d08e
SCOPE               §15.1–15.5 with the §15.0 rulings (FQ-b2 changed: default list + typed policy)
ENDPOINTS           implement, commit, push, open PR READY FOR OPERATOR REVIEW: authorized (freeze
                    message); merge: NOT authorized
SEQUENCE            Eb-C1 → C2 → C3 → C4 → C5
STOPS               editing load.rs; kernel/contract/persistence change; any digest change; any change
                    to the three worlds' validate output
```

## Current checkpoint

Freeze recorded (`15b5907`). Base binary for EB-6 building into `target/eb-base`, evidence in
`target/eb-evidence/base.txt`. Next: Eb-C1 (specs).

## Environment note

`cargo` is not on the non-interactive shell's PATH here: prefix `export PATH=$HOME/.cargo/bin:$PATH`.
No `sed -i`, no `python3 -c`, no heredoc writes.

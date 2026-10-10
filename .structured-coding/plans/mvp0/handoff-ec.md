# Handoff — S16 PR E-c (a System Pack from outside the repository)

Continuation aid only; the authority is `step-16-packages.md` §16 (with §16.12). E-b's handoff
(`handoff-eb.md`) is closed; its state is in §15.

```text
PROJECT / PR        MVP-0 · S16 / PR E-c — a System Pack from outside the repository (#99, draft)
PRIMARY DESIGN      .structured-coding/plans/mvp0/step-16-packages.md §16 (DESIGN FROZEN 2026-10-08)
BINDING             step-16 §§3–9, §14, §15; step-14 §13.0.3; overall.md "Parallel build-out"; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-ec-third-party · /Users/yuema137/mineworld-worktrees/impl-ec
EXTERNAL            yuema137/mineworld-pack-fishing main @ b40e71f, tag v0.1.0; clone at
                    /Users/yuema137/mineworld-worktrees/ext-fishing
BASE                main @ 6ca763d, merged with origin/main @ ec38570 (f5e39f0)
ENDPOINTS           implement, commit, push, PR, CI repair: authorized; pack repo main + tags: authorized;
                    merge: NOT authorized
STOPS               any edit outside §16's planned paths (kickoff); see F-Ec3 below
```

## Current checkpoint — STOPPED AT A MATERIAL DECISION (F-Ec3)

Ec-C1 … C8 implemented and committed. Evidence in §16.8 E-Ec0 … E-Ec8. The PR (#99) is a **draft**.

`fast` and `platforms` (macos-26, windows-2025) are green on the branch; **Linux `test` fails** because CI's
container layers put `CARGO_HOME` inside the checkout (`.github/actions/layer/action.yml`), so the git
checkout of the pack is classified bundled (ARC-54 point 2) and `package_sources.rs` EC-4 fails, correctly.
The fix (A) is ready on `scratch/ec-cargo-home` (the layer action's `CARGO_HOME` →
`$HOME/.mineworld-ci-cargo`); it is outside §16's paths, so it waits for the operator.

## Exact next actions after the operator rules

1. If (A) is approved for this PR: cherry-pick the layer-action change from `scratch/ec-cargo-home`
   (`f1482c6`) onto `mvp0/pr-ec-third-party`, record it in §16.8, push, wait for `fast`, `test`,
   `platforms` green on the exact head, mark the PR ready (READY FOR OPERATOR REVIEW). Do not merge.
2. If S13 lands it instead: merge origin/main, re-run, same.
3. Delete scratch branches `scratch/ec-platforms`, `scratch/ec-cargo-home`, `scratch/ec-mc9` and the
   worktrees `/tmp/ec-ch`, `/tmp/ec-m9` when no longer needed; local leftovers:
   `/Users/yuema137/mineworld-worktrees/ext-fishing-local.toml`, `ext-fishing-target/`, `/tmp/ec-ev`.

## Environment note

`cargo` is not on the non-interactive shell's PATH: prefix `export PATH=$HOME/.cargo/bin:$PATH`.
No `sed -i`, no `python3 -c`, no heredoc writes, no `awk`/`xargs`/`curl`.

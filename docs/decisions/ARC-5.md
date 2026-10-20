## ARC-5 — The repository is remote, and `main` is protected

**Date** 2026-09-26 · **Supersedes** the local-only half of effort decision `D-9`

`git@github.com:yuema137/MineWorld.git` exists. `main` and the in-flight branch are pushed, and
branch protection is being enabled, so from here every change reaches `main` through a pull
request.

**What changes.** Branch push and PR creation are authorized for implementation sessions. Remote
CI becomes possible before S13, which had assumed it would create the repository.

**What does not.** Merging remains the operator's decision. This is the one authority the
autonomous-execution grant never included, and protection now enforces it mechanically instead of
relying on an agent honouring a line in a contract — which is a strictly better place for it to
live.

**One command from the operator's setup was deliberately not run.** The given sequence ended
`git branch -M main`, which assumes a fresh repository whose current branch needs renaming. Here
`main` already existed and the checked-out branch was a feature branch, so that command would have
renamed the feature branch to `main` and clobbered the merged history. Recorded because the next
person setting up a mirror will meet the same instructions.

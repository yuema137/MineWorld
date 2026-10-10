# Handoff — S23 R-c (continuation aid; the authority is step-23-release.md §17.2 and §19)

- **PROJECT / PR:** MineWorld mvp0 · S23 R-c — mineworld-launch, server --stop-on-stdin-eof, smoke tests
- **PRIMARY DESIGN DOC:** `.structured-coding/plans/mvp0/step-23-release.md` §§1–16, §17.2 (contract), §19
  (commit plan and ledger)
- **RELATED / BINDING DOCS:** as listed in §17.2
- **Branch / worktree:** `mvp0/pr-s23-rc`, `/Users/yuema137/mineworld-worktrees/impl-s23-rc` (this session
  only)
- **Implementation base:** `origin/main @ bb62edf`
- **Current HEAD:** see `git log -1`; working tree clean at each commit
- **Approved scope / frozen invariants / sequence / budget / stop conditions:** §17.2, unmodified
- **Endpoint authority:** implementation, commits, push, PR, CI repair — authorized (§17.2, sources there;
  plus the primary session's dispatch of 2026-10-10, D-RC-0). Merge: operator only.
- **Current checkpoint:** C0 written.
- **Exact next actions:** C1 specs → C2 server flag → C3 launcher core → C4 per-OS → push, PR, CI. C5–C6
  wait for R-a's merge.
- **Cargo:** `export PATH="$HOME/.cargo/bin:$PATH"`, `CARGO_TARGET_DIR=/tmp/impl-s23-rc-target`.
- **Background processes:** none.

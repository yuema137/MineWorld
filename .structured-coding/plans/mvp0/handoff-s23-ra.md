# Handoff — S23 R-a (export presets, client path fixes, local packaging script)

Continuation aid only; the authority is `step-23-release.md` §17.1 (contract) and §18 (commit plan and
ledger).

- **PROJECT / PR:** MineWorld mvp0 · S23 R-a.
- **PRIMARY DESIGN DOC:** `.structured-coding/plans/mvp0/step-23-release.md` §§1–17, §18.
- **RELATED / BINDING DOCS:** as listed in §17.1.
- **Branch / base:** `mvp0/pr-s23-ra` from `origin/main @ bb62edf`; worktree
  `/Users/yuema137/mineworld-worktrees/impl-s23-ra`, held by this session only.
- **Approved scope, frozen invariants, endpoint authority, validation budget, stop conditions:** §17.1,
  unmodified.
- **Current checkpoint:** C0 written (§18) and committed; **STOPPED at material stops 1 and 2** (§18.4,
  MS-Ra-1 and MS-Ra-2). Nothing beyond Markdown is written. The C0 commit plan is not frozen.
- **Outside the repository:** Godot 4.7.2 export templates installed in
  `~/Library/Application Support/Godot/export_templates/4.7.2.stable/` (SHA-512 checked); scratch probes and
  measured exports in `/Users/yuema137/mineworld-worktrees/scratch-s23-ra/` (removable; nothing there is
  evidence that is not also copied into §18).
- **Background processes:** none.
- **Exact next actions, after the primary session's ruling:** freeze §18 with D-Ra-1 … D-Ra-4; resolve
  MS-Ra-1 (option A or B, and who owns it) and MS-Ra-2 (option C, D or E); then C1 → C6 as §18.5 orders them.
  Tell R-c about D-Ra-1 (the launcher starts exported clients; `--main-pack` does not exist in release
  templates) before its C3.

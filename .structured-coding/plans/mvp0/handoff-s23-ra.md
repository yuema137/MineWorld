# Handoff — S23 R-a (export presets, client path fixes, local packaging script)

Continuation aid only; the authority is `step-23-release.md` §17.1 (contract) and §18 (commit plan,
probes, rulings and ledger).

- **PROJECT / PR:** MineWorld mvp0 · S23 R-a · draft PR #146.
- **PRIMARY DESIGN DOC:** `.structured-coding/plans/mvp0/step-23-release.md` §§1–17, §18.
- **Branch / base:** `mvp0/pr-s23-ra` from `origin/main @ bb62edf`; worktree
  `/Users/yuema137/mineworld-worktrees/impl-s23-ra`, held by this session only.
- **Approved scope, frozen invariants, endpoint authority, validation budget, stop conditions:** §17.1, as
  revised by the C0 freeze (D-Ra-1 … D-Ra-4) and the primary rulings of 2026-10-10 (§18 status line).
- **Current checkpoint:** C0 frozen; C1 and C3 committed; C5 (package.py, packaging/, ci_layer line) and the
  2D half of C4 (2D presets, D-Ra-3 for 2D) in progress. C4's 3D half and C6 wait for RL-b (PR #131) to merge.
- **MS-Ra-2:** not resolved by RL-b (3d.pck 259.5 MB gzipped on RL-b's head, §18.7). Ruled 2026-10-10:
  512 px textures for small and distant props, in a separate 3D-lane PR after RL-b; R-a's 3D budget check
  runs against the tree with both merged. **R-a is paused (draft) until RL-b and that texture PR merge.**
- **Outside the repository:** Godot 4.7.2 templates in `~/Library/Application Support/Godot/export_templates/
  4.7.2.stable/`; cargo-about 0.9.2 in `~/.cargo/bin`; scratch in `/Users/yuema137/mineworld-worktrees/
  scratch-s23-ra/` including a detached worktree `rlb` at RL-b's `8f0853c` (remove with `git worktree
  remove` when done).
- **Exact next actions:** finish C5 validation (assemble macos-universal with `--clients 2d`, probe from a
  folder "MineWorld 世界"); commit C4-2D + C5; push; PR checks. After RL-b merges: merge origin/main, add the
  3D presets (exclude `tools/blender/*, tools/texture_art/*`, not `tools/*`), D-Ra-3 for 3D, then C6.

# Handoff — PR S33-c (3D world selection and the per-world depiction seam)

A continuation aid only; the authority is [`pr-s33c-world-seam.md`](pr-s33c-world-seam.md) §15.

- Worktree `/Users/yuema137/mineworld-worktrees/impl-s33c`, branch `mvp0/pr-s33c`, base `origin/main @ 5a8c6f28`.
- Depictions live under `clients/3d-spike/scripts/depictions/` (not `scripts/worlds/`: `client_rules.rs` check
  (c) refuses any literal path with a `worlds/` component; §15 D-S33c-1).
- Validation is local and targeted (SPEED FIRST charter): Social Café parity by `frame_diff.gd` and text
  comparison against the base captures, `client_rules`, the Lakeside link probe, the refusal test.
- Merge is the coordinator's; the PR stops at READY FOR OPERATOR REVIEW.

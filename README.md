# RL-b2 review frames (images only, not merged)

Before/after stills for PR RL-b2's 512 px textures (`./mineworld-slice --shots`, 1600x900).

- `base/` before: `main @ 9aa185b` with the PR's `slice_probe.gd` (for the three new frames)
- `c2/` after: the PR's texture commit
- `pairs/` the seven views to judge, `-1-before` and `-2-after` side by side
- `heat/` changed pixels (over 8/255) in red

Checklist: `docs/HUMAN_REVIEW_QUEUE.md`, "RL-b2". Design:
`.structured-coding/plans/mvp1/pr-rl-b2-two-views-and-textures.md`.

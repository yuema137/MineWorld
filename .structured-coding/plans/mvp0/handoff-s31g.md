# Handoff — S31-g, the agent (MCP) view of trade and notices

A continuation aid, never a second design authority: the design and its live ledger are
[`pr-s31g-agent-view.md`](pr-s31g-agent-view.md) §16.

- **Worktree / branch:** `/Users/yuema137/mineworld-worktrees/impl-s31g`, `mvp0/pr-s31g`, base `origin/main @ fec906f2`.
- **Push rule (dispatch):** commit locally only; no push, no PR, until the lane lead says so.
- **State:** C0 committed (base, shapes pinned, rulings recorded). Next: C1, the `ARC-153` note.
- **Validation budget:** only the files a commit writes or changes; cargo only through `cast-slots/with-slot.sh`;
  never a workspace test run. Acceptance criteria and mutations run at the milestone.

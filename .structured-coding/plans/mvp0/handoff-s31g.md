# Handoff — S31-g, the agent (MCP) view of trade and notices

A continuation aid, never a second design authority: the design and its live ledger are
[`pr-s31g-agent-view.md`](pr-s31g-agent-view.md) §16.

- **Worktree / branch:** `/Users/yuema137/mineworld-worktrees/impl-s31g`, `mvp0/pr-s31g`, base `origin/main @ fec906f2`.
- **Push rule (dispatch):** commit locally only; no push, no PR, until the lane lead says so.
- **State:** C0 … C8 committed locally. Unit, MCP-in-memory and renderer tests pass; static checks clean.
- **Left for the milestone:** run `clients/agent/tests/test_play_market_mcp.py` (written, never run: D-g-9) and the
  real-binary files whose `Amount` reads changed (`test_play_mcp.py`, `test_play_market_town.py`,
  `test_play_station_mcp.py`, `test_http_endpoint.py`); plant X-9 and M-g-1 … M-g-6; re-record the agent fixtures
  (D-g-5).
- **Validation budget:** only the files a commit writes or changes; cargo only through `cast-slots/with-slot.sh`;
  never a workspace test run.

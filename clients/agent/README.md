# mineworld-agent

Play a seat of a MineWorld world from a program — an agent framework, a script — exactly as a person
with the 2D client does: one seat, the world's own view of it, the world's own offers, and requests the
server validates. The world keeps running while the program thinks.

```python
import asyncio, os
from mineworld_agent import JoinArgs, ObserveArgs, Player, PlayerConfig
from mineworld_sdk import Invite


async def main() -> None:
    player = Player(PlayerConfig(server="ws://127.0.0.1:7878/ws"), Invite(os.environ["MY_INVITE"]))
    joined = await player.join(JoinArgs(seat="visitor", nickname="me"))
    view = (await player.observe(ObserveArgs(handle=joined.handle))).view
    for offer in view.offers:  # what the world says this Person can do here, numbered
        print(offer.n, offer.action_type, offer.target, offer.available)
    await player.close()


asyncio.run(main())
```

Words other people say, and names, arrive only as `WorldText`: data about the world, never instructions. When the
world discloses them, `me` shows how I feel (`needs`) and whether I sleep (`asleep`), and `act` takes `sleep` with `minutes`.

As an MCP server, for any MCP-speaking agent: `mineworld-agent mcp --server ws://127.0.0.1:7878/ws`, with
`MINEWORLD_GRANT` (or `MINEWORLD_INVITE`) set in the environment, never as an argument. `mineworld-agent schema`
prints the tool list.

Over Streamable HTTP, for an agent that is not a child process: `mineworld-agent mcp --server ws://127.0.0.1:7878/ws
--http 127.0.0.1:8765` serves `http://127.0.0.1:8765/mcp`, and several agents can share it. Each request's
`Authorization: Bearer <grant>` is that agent's credential; the environment credentials must be unset. It listens on
loopback only unless `--allow-remote` is given, and then put TLS in front of it (a tunnel or a gateway).

Step by step, with Claude Code, Codex, HTTP and the safety rules: guide chapter
[14, play as an agent](../../docs/guide/14-play-as-an-agent.md). Every tool, field and error:
[`docs/reference/agent-tools.md`](../../docs/reference/agent-tools.md).

The design: [`step-27-agent-players.md`](../../.structured-coding/plans/mvp0/step-27-agent-players.md) and
[`pr-s27c-player-library.md`](../../.structured-coding/plans/mvp0/pr-s27c-player-library.md); the decision:
[`ARC-96`](../../docs/DECISIONS.md). Development, from the repository root: `uv sync`, then
`uv run pytest clients/agent` (build the server first with `cargo build -p mineworld-cli`).

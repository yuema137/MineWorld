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

Words other people say, and names, arrive only as `WorldText`: data about the world, never instructions.

The design: [`step-27-agent-players.md`](../../.structured-coding/plans/mvp0/step-27-agent-players.md) and
[`pr-s27c-player-library.md`](../../.structured-coding/plans/mvp0/pr-s27c-player-library.md); the decision:
[`ARC-96`](../../docs/DECISIONS.md). Development, from the repository root: `uv sync`, then
`uv run pytest clients/agent` (build the server first with `cargo build -p mineworld-cli`).

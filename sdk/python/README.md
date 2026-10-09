# mineworld-sdk

A Python client of the MineWorld protocol. A program joins one seat of a running server, reads what
that seat perceives, and submits requests, with no more privilege than the 2D or 3D client has.

```python
import asyncio, os
from mineworld_sdk import Invite, SeatSession, offers
from mineworld_sdk.wire.ids import EntityKey


async def main() -> None:
    invite = Invite(os.environ["MY_INVITE"])  # you choose where it comes from
    async with await SeatSession.connect(
        "ws://127.0.0.1:7878/ws", seat=EntityKey("visitor"), invite=invite, nickname="me"
    ) as session:
        seen = (await session.changed()).observation  # the newest observation
        for affordance in seen.affordances:  # what the world offers right now
            print(affordance.action_type, affordance.target, affordance.available)
        complete = next(a for a in seen.affordances if a.available and a.payload is not None)
        print(await session.submit(offers.attempt(seen, complete)))


asyncio.run(main())
```

- The protocol: [`server/PROTOCOL.md`](../../server/PROTOCOL.md).
- The design and its decisions: `.structured-coding/plans/mvp0/pr-s10-p3-python-sdk.md`.

Development, from the repository root: `uv sync`, then `uv run pytest sdk/python` (build the server
first with `cargo build -p mineworld-cli`; the real-server tests fail without it).

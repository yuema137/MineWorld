# mineworld-sdk

A Python client of the MineWorld protocol. A program joins one seat of a running server, reads what
that seat perceives, and submits requests, with no more privilege than the 2D or 3D client has.

```python
import asyncio, os
from mineworld_sdk import CursorCell, Invite, ResumingSeat, offers
from mineworld_sdk.wire.ids import EntityKey


async def main() -> None:
    invite = Invite(os.environ["MY_INVITE"])  # you choose where it comes from
    cursor = CursorCell()  # or your own durable store: anything with .cursor()
    async with await ResumingSeat.connect(  # survives dropped sockets, lagged, restarts
        "ws://127.0.0.1:7878/ws",
        seat=EntityKey("visitor"),
        invite=invite,
        nickname="me",
        cursor=cursor,
    ) as seat:
        seen = (await seat.changed()).frame.observation  # the newest observation
        complete = next(a for a in seen.affordances if a.available and a.payload is not None)
        print(await seat.submit(offers.attempt(seen, complete)))
        async for batch in seat.perceived():  # every fact this Person learned, once, in order
            print([fact.event_type for fact in batch.events])
            cursor.commit(batch)  # after storing the facts, never before


asyncio.run(main())
```

`SeatSession` is the same for one connection only. The protocol:
[`server/PROTOCOL.md`](../../server/PROTOCOL.md). The designs: `.structured-coding/plans/mvp0/`
`pr-s10-p3-python-sdk.md` and `pr-s10-p3b-perceived.md`.

Development, from the repository root: `uv sync`, then `uv run pytest sdk/python` (build the server
first with `cargo build -p mineworld-cli`; the real-server tests fail without it).

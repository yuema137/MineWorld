"""Play a seat of a MineWorld world from a program, as a client (`docs/DECISIONS.md` `ARC-96`).

`Player` holds one seat per handle, each over its own `mineworld_sdk.ResumingSeat`, and renders that seat's
own observation compactly: short references, numbered offers, pages and a hard size cap. It adds no world
rule, evaluates no requirement, and carries world-authored text only as typed `WorldText`. The MCP adapter
(S27-d) is one module of this package, `mcp_server.py`, over this library; nothing else here names MCP.

```python
async with Player(PlayerConfig(server="ws://127.0.0.1:7878/ws"), Invite(invite)) as player:
    joined = await player.join(JoinArgs(seat="visitor", nickname="Ada"))
    seen = await player.observe(ObserveArgs(handle=joined.handle))
```
"""

from mineworld_agent.models import (
    ActResult,
    AttemptArgs,
    ClockArgs,
    ClockResult,
    ErrorCode,
    EventsArgs,
    EventsResult,
    InboxStatus,
    InspectArgs,
    InspectResult,
    JoinArgs,
    JoinRefused,
    JoinResult,
    LeaveArgs,
    LeaveResult,
    ListForSaleArgs,
    MoveArgs,
    ObserveArgs,
    ObserveResult,
    PlayerRefusal,
    PublishNoticeArgs,
    SayArgs,
    SeatLostRefusal,
    SeatsArgs,
    SeatsResult,
    SetPriceArgs,
    StopArgs,
    SubscribeArgs,
    ToolError,
    UnsubscribeArgs,
    View,
    ViewDelta,
    WaitArgs,
    WaitResult,
    WalkToArgs,
    WithdrawArgs,
    WorldText,
    render_text,
)
from mineworld_agent.player import Player, PlayerConfig
from mineworld_agent.roles import ROLE_TAGS, RoleTag

__all__ = [
    "ROLE_TAGS",
    "ActResult",
    "AttemptArgs",
    "ClockArgs",
    "ClockResult",
    "ErrorCode",
    "EventsArgs",
    "EventsResult",
    "InboxStatus",
    "InspectArgs",
    "InspectResult",
    "JoinArgs",
    "JoinRefused",
    "JoinResult",
    "LeaveArgs",
    "LeaveResult",
    "ListForSaleArgs",
    "MoveArgs",
    "ObserveArgs",
    "ObserveResult",
    "Player",
    "PlayerConfig",
    "PlayerRefusal",
    "PublishNoticeArgs",
    "RoleTag",
    "SayArgs",
    "SeatLostRefusal",
    "SeatsArgs",
    "SeatsResult",
    "SetPriceArgs",
    "StopArgs",
    "SubscribeArgs",
    "ToolError",
    "UnsubscribeArgs",
    "View",
    "ViewDelta",
    "WaitArgs",
    "WaitResult",
    "WalkToArgs",
    "WithdrawArgs",
    "WorldText",
    "render_text",
]

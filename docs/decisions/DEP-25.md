## DEP-25 — `websockets` for the Python client's WebSocket

**Date** 2026-10-08 · **Status** adopted by S10 PR P3 (`websockets>=17,<18`; locked 17.2) · **Approved
by** the primary session at P3's freeze · **Licence** BSD-3-Clause · **Relates to** `ARC-56` · **Design**
step-17 §4.6, §4.7 ("WebSocket client"); P3 R-P3-6 · **Placeholder** `DEP-S10-c`

**Problem.** One asyncio WebSocket connection per seat, text frames only, to a local or remote server.

**Options considered:**

```text
(a) websockets (BSD-3-Clause; asyncio-native; maintained; requires Python >= 3.10)
(b) aiohttp (a whole HTTP client and server framework)
(c) httpx-ws (not verified)
(d) our own client over asyncio streams (RFC 6455 framing, masking, close handshake)
```

**Choice: (a)**, its `websockets.asyncio.client` implementation only, pinned to one major version
because the library has replaced its asyncio API before (R-P3-6).
- **(b) declined:** a framework for one socket.
- **(c) declined:** maturity not verified.
- **(d) declined:** commodity infrastructure (`REUSE_POLICY.md` §12).

**Isolating interface.** `mineworld_sdk.session.SeatSession` is the only module that imports
`websockets`; nothing else in the SDK, and no caller, names a `websockets` type.

**Revisit** at the next major version, or if a transport other than WebSocket is added.

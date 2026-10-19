## ARC-40 — Who controls a Person is host state: a binding change moves no revision

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-B · **Approved by** the primary
session at S11-B's design freeze (step-12 §16, QS11B-1 … QS11B-6 as recommended) · **Relates to**
`ARC-25`, `ARC-27`, `ARC-42`, `INV-1`, [`NETWORKING.md`](NETWORKING.md) §§9–10 · **Design**
`.structured-coding/plans/mvp0/step-12-server.md` §§4.2–4.4, 16 · **Specification**
[`server/PROTOCOL.md`](../server/PROTOCOL.md) §4.2

**Problem.** `INV-1` says a Person exists independently of whatever decides its actions, and that
control may change at runtime without altering the Person; `AC-5` asks that a human take over an
existing NPC with biography, relationships, inventory and employment intact. Until S11-B a seat was not
exclusive: a human joining `alice` while `--agent alice` drove her made two controllers on one Person,
and nothing in the server knew who drove a seat.

**Choice.** Who drives a seat is **host state**, kept by the world thread's `SeatTable`
(`server/src/seats.rs`), its only writer: each seat is `free`, `hosted` (an in-server controller),
`connected` (one connection) or `held` (its connection dropped; nobody drives it until the hold ends or
the connection's resume secret returns). A binding change — a join, a takeover, a `leave`, a drop and its
hold, a resume, a hold's expiry, a release to the seat's default — is **never journaled, states no fact
and does not move the world's revision**. Seat bindings, sessions, holds, resume secrets and nicknames
appear in no save. A seat returns to its default with a controller built afresh from its configuration,
never one resumed from memory.

**Why this is how `AC-5` is measured.** If control changes and the journal does not, the change cannot
have altered the Person: everything the Person is — biography, `knows` edges, holdings, wallet, job — is
a projection of the fact log (`ARC-25`, `ARC-29`), and the log moved only by requests somebody submitted
through the one authority path. S11-B's acceptance reads the save's revision across every binding change
and requires every fact added while a human held the Person to name an `ActionId` that human's session
was answered with.

**Options considered.** (a) Bindings as host state on the world thread — chosen: the binding must live
where the world lives (step-12 A-9), and nothing about it is the world's business. (b) Bindings as a
component or a fact (`controlled-by`) — rejected: control would become world state, a replay would
re-bind controllers, and `INV-1` would be false by construction. (c) Bindings in the transport, per
session — rejected: two sessions could each believe they held one seat (the A-1 defect), and an
in-server controller is not a session. The pattern of hold-and-resume follows Colyseus's reconnection
(a grace window, a token refreshed on every connection, the stale connection superseded) as a reference,
not a dependency (step-12 §7.4).

**Limitations accepted (MVP-0).**
- **One level of trust.** Any holder of the server's invite may take any seat with `take_over: true`,
  including a dropped player's held seat. MVP-0 has one shared invite and no per-player identity
  (`NETWORKING.md` §9: "matchmaking, global accounts" are non-goals), so the server cannot tell the
  dropped player from a friend on the same invite; a finer policy — for example, only the operator may
  take a held seat — needs accounts. This is LAN-friends trust, stated rather than implied (QS11B-3).
- **A hold and a pace are both wall time.** `--hold` counts wall seconds (a hold is about a network).
  `--pace` counts wall seconds too — **amended 2026-10-08 by operator ruling QTW-13**
  ([`step-19-time-weather.md`](../.structured-coding/plans/mvp0/step-19-time-weather.md) §4.4), which
  reverses QS11B-4's "a pace is about a life": a hosted Person walks one stride per consult and walking is
  embodied, so the time scale must never make it walk or talk faster. An adapter states its cadence to
  the seam as `cadence × time_scale` world seconds (the reactive controller's one wall second likewise).
  With `--time-scale 60` a 30-second hold is half an hour of the world, and a paced seat is consulted
  every 5 wall seconds, 300 world seconds. Headless `run` is unaffected: it has no wall clock and keeps
  its 900-second world pace (`ARC-27`).
- **Nothing survives a restart.** Holds and resume secrets are host state and are not persisted; after a
  restart a player joins again and finds the seat free or hosted.
- **"Missed the moment."** A controller bound at an instant treats every line heard at or before that
  instant as not its to answer (`RuleController::since`, `F-13`), so a line said in the same world second
  a controller is bound is not answered — the limitation `ARC-27` already accepts for consults.

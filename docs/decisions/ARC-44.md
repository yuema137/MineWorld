## ARC-44 — The admin surface is HTTP behind a bearer token, mounted only when configured, and changes no world state

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-D · **Approved by** the primary
session at S11-D's design freeze (step-12 §18, QS11D-1 … QS11D-8; QS11D-3 by the operator) · **Relates
to** `ARC-40`, `ARC-41`, `DEP-14`, `INV-9`, [`NETWORKING.md`](NETWORKING.md) §§3, 5, 9 · **Design**
`.structured-coding/plans/mvp0/step-12-server.md` §§4.9, 7.6, 18;
[`step-19-time-weather.md`](../.structured-coding/plans/mvp0/step-19-time-weather.md) §§7.2–7.4, 15.3 ·
**Specification** [`server/PROTOCOL.md`](../server/PROTOCOL.md) §§5.9, 11

**Problem.** An operator could see a hosted world only through `/status`: not who is connected, not who
drives which seat, and not how to remove a misbehaving player or pause the town. S19 needs a host-only
pause, and a `clock` announcement so clients re-anchor their time display. None of this may give a
client a new way to change the world, and none of it may make the invite a host credential.

**Choice.**
- **HTTP on the control plane, never a socket frame.** `server/src/admin.rs` serves `GET
  /admin/sessions`, `GET /admin/seats`, `POST /admin/sessions/{session}/kick`, `POST
  /admin/seats/{seat}/release`, `GET /admin/clock` and `POST /admin/clock` (`{ "paused": bool }`), each
  body typed and denying unknown fields. The client vocabulary stays `join`, `submit`, `leave` (`INV-9`):
  a socket frame `pause`, `clock`, `kick` or `release` is `unknown_frame`.
- **Mounted only when configured.** With `--admin-token` / `MINEWORLD_ADMIN_TOKEN` the routes exist;
  without it every `/admin` path is `404`. One bearer token, compared in constant time with `DEP-14`'s
  `subtle`; every failure answers `401` no sooner than 500 ms after the request, in that request's task.
  The token follows the invite's rules, is never printed, is in no save, frame or `/status`, and must
  differ from the invite.
- **No world state.** Kick, release, pause and resume are host state (`ARC-40`): never journaled, no fact,
  no revision. A kick or release returns the seat to its default with no hold, and the displaced
  connection is told `closing { kicked }`. **Pause is host pacing**: the host clock is a segment that
  stops where it stands and continues from there on resume (monotonic and jump-free); while paused the
  world is not advanced, hosted controllers are not consulted and every `submit` is refused `paused`
  before an `ActionId` is allocated; observers stay connected. This is S19's clarification of I-4: a
  route may change the pacing of the clock, never its value — **no route and no field names an
  instant**.
- **Telling clients.** `WorldSummary.paused`, and a `clock { at, time_scale, paused }` frame after
  `welcome` and on every change, carried by a `tokio::sync::watch` channel the world thread writes —
  newest wins, never dropped, never waited on.
- **The live scale change is TW-c's.** `POST /admin/clock` carrying `time_scale` answers `409
  time_scale_fixed` until TW-c lands it with the hosted-cadence rescheduling and the host journal it
  needs. S19's record of pause and scale as host commands (its placeholder ARC-69) remains TW-c's.

**Options considered.** (a) HTTP routes on the existing axum router — chosen: the control plane
`NETWORKING.md` §3 names, testable with any HTTP client, no new dependency. (b) Admin frames on the
WebSocket — rejected: it widens the client vocabulary and puts host commands one field away from player
commands. (c) A separate gRPC admin service (`tonic`) — rejected: a second protocol stack and listener for
six routes. (d) `axum-extra`'s `TypedHeader` for the bearer header — rejected: two crates to read one
header, which is about fifteen lines with `HeaderMap`.

**Limitations accepted (MVP-0).**
- **One admin token, one level of host trust.** Whoever holds it is the host (the operator, or a
  single-player launcher that generated one for its own client).
- **No ban list** (QS11D-3, operator): a kicked player may rejoin at once with the invite, because there
  is no per-player identity. To remove someone for good, rotate the invite (restart with a new
  `--invite`).
- **A pause is not persisted** until TW-c's host journal: a server restarted while paused runs. Holds are
  wall time and still end during a pause.
- **Unbounded parallel wrong-token requests** each wait 500 ms but are not counted; rate limiting is the
  adopt route for public hosting (`DEP-14`).

**Note, 2026-10-14 (S19, PR TW-c; `ARC-69`).** `time_scale` in `POST /admin/clock` is now accepted: a live
change is host pacing and is recorded in the save's host journal, never as a fact. `time_scale_fixed` (409) is
retired (`ARC-41`: may omit, never redefine); `fast_forward_solo_only` (409) answers a scale above 24 that the
server was not started `--solo` to allow. The sentence "a pause is not persisted until TW-c's host journal" is
superseded in part: pauses, resumes, rescales and the start and stop of a persisted world are journaled as host
records, and a restart still runs (`ARC-69` items 6 and 8).

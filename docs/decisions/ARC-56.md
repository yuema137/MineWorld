## ARC-56 — Cognition is a client: a Python process joins a seat over the public protocol

**Date** 2026-10-08 · **Status** accepted; implemented first by S10 PR P3 (`mineworld-sdk`) · **Approved
by** the primary session at P3's design freeze (step-17 S10 freeze; P3 §11.1) · **Relates to** `ARC-41`,
`ARC-34`, `INV-1`, `INV-9`, `INV-13`, effort decision `D-4` · **Design**
`.structured-coding/plans/mvp0/step-17-cognition.md` §§3.2, 4.6;
`.structured-coding/plans/mvp0/pr-s10-p3-python-sdk.md` · **Placeholder in the step design**
`ARC-S10-a` (S10's range `ARC-56 … ARC-60`, `overall.md` ruling 6)

**Problem.** Cognition is Python (`ENGINEERING_STANDARDS.md` §3) and the world is Rust, and the server
must never block on a model (`ARCHITECTURE.md` §9). Something carries what a seat perceives out to the
cognition process and its requests back in.

**Options considered** (step-17 §4.6, with licences and maturity):

```text
(a) an LM controller in Rust, inside the server   a model stall shares the world's process; Rust's model
                                                  clients are thinner; against §3's Python rule
(b) Python embedded in the server (PyO3)          a cognition crash or GIL stall is a world crash or
                                                  stall; the server crate would depend on a controller
(c) a Python sidecar behind a Rust driver          a second protocol (gRPC or stdio JSON-RPC) beside the
    (gRPC or stdio JSON-RPC)                       client one; one controller split over two processes
(d) a Python process that is a client of the      chosen
    public protocol, one WebSocket per seat
```

**Choice: (d).** A model-driven Person is a client. Its process joins a seat with the invite and a
nickname, receives exactly what that seat is entitled to, and sends the same three frames a 2D or 3D
client sends (`join`, `submit`, `leave`). The Python side is `sdk/python` (`mineworld-sdk`): typed
mirrors of the revision-2 frames, held to `server/tests/frames/`, and one seat session.

- **`INV-9` and `INV-1` by construction.** The server cannot tell a model from a person, and has no path
  a model could use that a player lacks.
- **One boundary.** The JSON client protocol is already proven from Godot; Python is its next consumer,
  checked by the same golden frames (`ARC-41`). gRPC is declined: a second protocol beside the client
  one, with `.proto` mirrors of every contract, for an encoding gain the protocol's rates do not need.
- **The server never waits.** A model call happens in another process.

**Consequences.** A seat's cognition has no more privilege than a player. Whatever cognition needs from
the world (reliable perceived events, a resumable cursor, seat exclusivity) is a protocol feature every
client gets (step-17 §11, S11-B, S11-C). A Python SDK names no System Pack's vocabulary
(`mineworld-sdk` builds requests generically from what an observation offers); pack-specific request
builders live with the controller that uses them (P6).

**Revisit** if a measured latency or bandwidth need cannot be met by the JSON protocol — the same
trigger as `ARC-41`'s encoding choice.

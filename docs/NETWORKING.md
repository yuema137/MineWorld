# MineWorld — Networking and Hosting

**Status:** frozen networking decisions for v0.1 planning
**Audience:** coding agents. Vocabulary: [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md). Layering:
[`ARCHITECTURE.md`](ARCHITECTURE.md).

---

# 1. The decision that must never be revisited casually

> **MineWorld is always server-authoritative.**

There is no "single-player architecture" and no "multiplayer architecture". There is one
architecture, deployed differently. Single-player is a client connected to a local server:

```text
MineWorld Client
      │
   localhost
      │
MineWorld Server
```

Private multiplayer:

```text
Player A ─┐
Player B ─┼──►  MineWorld Server on your PC
Player C ─┘
```

Public world:

```text
Internet
   │
MineWorld cloud server
   │
persistent world
   ├── humans
   └── agents
```

The reason is structural: two code paths for the same semantics would diverge, and every
system would eventually need to know which mode it was running in.

---

# 2. Authority model

```text
                    ┌──────────────┐
                    │ World Server │
                    │ authoritative│
                    └───────┬──────┘
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
      Human Client     AI Controller    Admin Client
        (Godot)          Worker          / Editor
```

Clients and controllers may send:

```text
input
ActionIntent
chat
interaction requests
```

They may never send, and the server must never accept:

```text
"my money is now $5000"
"Bob moved here"
"this item belongs to me"
```

The server validates every world change (INV-5, INV-6, INV-9). A message that asserts state
rather than requesting an action is a protocol violation and is rejected, not tolerated.

---

# 3. Transport

For V0 and V1:

```text
HTTP        control plane, world lifecycle, admin, authoring
WebSocket   live observation streams, state deltas, action submission
```

This is a life simulation, not a competitive FPS. There is no requirement for 60 Hz
authoritative replication.

Future transports — QUIC, WebRTC, custom UDP — must be addable **without changing simulation
semantics** (INV-14). A transport that requires a change to an interaction system is wired at
the wrong layer.

---

# 4. Rate separation

These rates are independent and must never be conflated in code:

```text
Rendering:            60+ FPS                  client concern only
Local physics:        renderer-dependent       never authoritative
World simulation:     discrete-event / semantic ticks
Network replication:  state-delta based
Agent cognition:      event triggered, asynchronous
```

The server never blocks on a controller decision, and never blocks on a model call
([`ARCHITECTURE.md`](ARCHITECTURE.md) §9). An intent that returns late is revalidated against
current state and may fail.

---

# 5. Message classes

```text
Client → Server
    authenticate
    subscribe / unsubscribe observation scope
    submit ActionIntent
    chat / interaction request
    admin command                    (authorized clients only)

Server → Client
    world snapshot / delta
    observation update
    action result                    accepted | rejected | ActionUnavailable
    event notification               subject to Visibility
    error
```

Observation scope is enforced server-side. A client cannot widen its own perception by asking
for more (INV-13).

---

# 6. Local hosting

```bash
mineworld server ./worlds/lakewood --listen 0.0.0.0:7878
```

The local player connects to `localhost:7878`. Friends on the same network connect to the
machine's LAN address.

Internet hosting from a personal machine initially relies on existing mechanisms:

```text
port forwarding
VPN mesh
tunnel
```

A relay / NAT-traversal service may be offered later as an optional convenience. **It must
never be required by the open-source framework.** A world must remain fully playable with no
project-operated infrastructure in the path.

---

# 7. Cloud hosting

The same server binary, in a container:

```text
Internet
   │
Gateway / TLS
   │
World Server
   │
   ├── Persistence
   ├── Cognition Workers
   └── Asset Storage
```

Initial deployment:

```text
single VPS  +  Docker  +  persistent volume
```

Later, when load justifies it:

```text
gateway  +  many independent World Workers  +  Postgres  +  object storage
```

Do not build Kubernetes-level distributed infrastructure for the MVP. Scale complexity is
earned by measured need, not anticipated.

---

# 8. Local / cloud parity

The same World Pack must run on a laptop and inside a cloud container with **no semantic
differences**. This is an MVP acceptance criterion ([`MVP.md`](MVP.md)), not an aspiration.
A behavior that appears only in one deployment is a defect in that deployment path.

---

# 9. Authentication and identity

MVP scope:

```text
server invite token  +  player nickname
```

Explicitly out of MVP scope:

```text
matchmaking
global account service
server browser
```

Later, a public-world deployment will need durable player identity, permissions, moderation
tooling, and server discovery. Each is specified when it is built; none of them may alter the
authority model in §2.

---

# 10. Persistence across sessions

Restarting the server preserves the world exactly ([`MVP.md`](MVP.md) acceptance). The world
continues while nobody is connected; a player connects to an already-running society and
disconnects without stopping it.

Networking therefore never owns world state. It transports observations and intents, and it
must be removable from the picture entirely: the simulation runs headless with no client
connected ([`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §22).

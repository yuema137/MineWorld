# Step 05 — Vertical slice: three windows, one Alice

**Effort:** `mvp0` · parent: [`overall.md`](overall.md) · **Lifecycle:** `DESIGN FROZEN`
**Base:** `main` @ 9b29795 · S1, S2, S3a, S3b and PR 04 are merged; 144 tests green

Binding: [`docs/MVP.md`](../../../docs/MVP.md) §9 `AC-15` ·
[`docs/ENGINEERING_RULES.md`](../../../docs/ENGINEERING_RULES.md) ·
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `DEP-2`, `DEP-3`, `ARC-6`

---

# 1. What this step is for

The numbered roadmap reaches three live windows only at S14. The reorder recorded in
`overall.md` moved it here, because `AC-15`'s minimal form does not need the scheduler, durable
persistence, the economy or any model. It needs dispatch (merged), a thin conversation system,
enough perception to see who is present, a server, and two clients.

The target, exactly:

```text
Terminal 1:  mineworld server worlds/social-cafe
Terminal 2:  mineworld-2d
Terminal 3:  mineworld-3d
```

alive at once, and **there is only one Alice**: speak to her in 2D, walk up to her in 3D, and
she knows it happened.

Everything after this thickens a system that already runs end to end.

## 1.1 Scope, deliberately thin

```text
systems/conversation/     a ConversationSystem: talk action, ConversationStarted event,
                          a conversation-history component it owns
systems/presence/         perception: who and what is here, and the affordances for them
server/                   the runtime: hosts a world, HTTP + WebSocket, observation stream,
                          intent submission, multiple simultaneous clients
worlds/social-cafe/       a World Pack: one place, Alice, Bob, a player-controllable Person
tools/cli/                mineworld server <world> — enough of the CLI to launch it
clients/2d, clients/3d    real clients, promoted from the spikes
```

## 1.2 Non-goals

```text
scheduler and processes        → S4, and the slice must not need them
durable persistence            → S5; this slice may hold the world in memory
inventory, economy, employment → S9
LM controllers, memory, biography → MVP-1
travel between places          → later; one place is enough to prove AC-15
authentication beyond a nickname
```

## 1.3 Frozen invariants

- `AC-15` evidence names identity, never appearance: same world instance, same Alice
  `EntityId`, same authoritative event sequence. "Both clients showed the same thing" is not
  evidence, and the false success it would admit — two clients each holding their own Alice,
  synchronised well enough to look alike — means the opposite of the claim.
- Alice remembers through a **projection**, not a memory system: the objective event log →
  a conversation-history component → her controller's context. *"Player A spoke to me three
  minutes ago"* is enough (`MVP.md` §9.2).
- No world rule in a client (`ENGINEERING_RULES.md` §8). Distance, availability, permission and
  system presence are all decided server-side.
- The wire carries 64-bit ids as decimal strings — already true in the contracts since PR 04, so
  the transport must **not** re-implement it (`DEP-3`).
- One server binary for localhost and network alike (`NETWORKING.md` §1).

---

# 2. The PRs

Four, in dependency order. The first two are independent of each other and may run in parallel.

### PR 05a — ConversationSystem and PresenceSystem

The first real System Packs. `ConversationSystem` provides `talk`, emits `ConversationStarted`
and `Spoke`, and owns a `ConversationHistory` component — which is the whole of Alice's memory
for this slice. `PresenceSystem` owns location and produces `Observation`s: who is here, where,
and what the observer may do, with affordances computed server-side through
`SpatialRequirement::evaluate`.

Acceptance: two systems installed in a headless world; a `talk` request from one Person to
another is accepted, emits its event, and appears in the target's history; the same request from
too far away is refused `TooFarAway`; disabling `ConversationSystem` makes `talk` `Unavailable`
with nothing else changing.

### PR 05b — The server

`mineworld server <world>`: hosts one world, serves HTTP for control and WebSocket for
observations and intents, streams a per-observer `Observation`, accepts `ActionRequest`s and
allocates their identity. Built on tokio and axum (`DEP-3`). Multiple simultaneous clients are a
requirement of this PR, not a later one — `AC-15` needs two at once.

Acceptance: two WebSocket clients connect at once; each receives observations scoped to its own
observer; an intent from one produces an event both are entitled to see; killing a client leaves
the world running.

### PR 05c — World Pack loading and the CLI

`worlds/social-cafe/` as YAML — `world.yaml`, `people/`, `places/` — loaded and validated into a
running world, with `mineworld server worlds/social-cafe` as the command. Enough CLI to launch;
`create`, `validate` and `inspect` follow in S7.

Acceptance: the pack loads, entity keys resolve to ids deterministically, a malformed pack is
refused by name rather than panicking, and the same pack produces the same ids twice.

### PR 05d — The clients, and `AC-15`

Promote both spikes from mock data to the real protocol. They keep their scene graphs, their art
and their camera work; what changes is where the world comes from. Then the acceptance test
itself.

Acceptance, and this is the one that matters: with the server and both clients running at once,
a `talk` in the 2D client is followed by walking up to Alice in the 3D client and finding that
she knows — with the evidence recording the same world instance, the same Alice `EntityId`, and
the same event sequence observed from both.

---

# 3. Review and approval

```text
CHECKED  AC-15's minimal form needs nothing from S4, S5 or S10 — the reorder holds
CHECKED  Alice's memory is a projection off the event log, not a cognition architecture
CHECKED  the evidence requirement names identity rather than appearance
CHECKED  no client-side rule; affordances are server-computed
CHECKED  the transport does not re-implement id encoding, which PR 04 moved into the contract
CHECKED  scope excludes the scheduler, durable persistence and the economy, all of which
         thicken this slice later rather than gating it
```

APPROVED. 05a and 05b may be implemented in parallel; 05c depends on 05a; 05d depends on all.

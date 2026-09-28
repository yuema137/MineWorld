# MVP-0 status

The point of this file is to stop MVP-0 being declared complete while a real path is untested.

`✅` means **actually run and inspected**, never inferred from a passing test suite or a
successful compile. `🚧` means in progress. `❌` means not started.

**Updated:** 2026-09-27 (PR 05d). Subjective questions are queued in
[`HUMAN_REVIEW_QUEUE.md`](HUMAN_REVIEW_QUEUE.md); a demo parked there does not block engineering.

## Capability matrix

| Capability | Headless | 2D | 3D |
| --- | --- | --- | --- |
| World boot | 🚧 composed from declarations; no runtime loop yet | ❌ | ❌ |
| Person | 🚧 contracts + storage | ❌ | ❌ |
| Spatial state | 🚧 contracts only | ❌ | ❌ |
| Movement | ❌ | ❌ | ❌ |
| Place | 🚧 contracts + storage | ❌ | ❌ |
| Conversation | ✅ `talk` end to end: request, refusal for distance, event, history | 🚧 a client can speak and read what it was told; the 2D client's own adoption is pending | 🚧 the same, and the 3D client's adoption is pending |
| Object interaction | ❌ | ❌ | ❌ |
| Persistence | ❌ | — | — |
| Networking | ✅ two clients and an agent on one server, from the real binary | 🚧 protocol module runs against the real server; adoption pending | 🚧 the same |

## Shippable artefacts

What a person can actually run. None of these exists yet.

| Artefact | Command | State |
| --- | --- | --- |
| World server | `mineworld server worlds/social-cafe --agent alice` | ✅ loads the pack, serves two clients and drives Alice with a rule controller |
| 2D client | `mineworld-2d` | 🚧 presentation spike runnable, awaiting style decision ([queue](HUMAN_REVIEW_QUEUE.md)) |
| 3D client | `mineworld-3d` | 🚧 presentation spike runnable with three camera modes; awaiting feel review ([queue](HUMAN_REVIEW_QUEUE.md)) |
| Developer CLI | `mineworld create / validate / run / inspect` | 🚧 `server` and `validate` exist; `create`, `run` and `inspect` are S7 |
| `worlds/social-cafe` | `mineworld validate worlds/social-cafe` | ✅ one café, four people, three seats, loaded and hosted |
| `worlds/market-town` | — | ❌ |

## Independent axes

```text
Social Café composition:                 🚧 presence + conversation; the economy set is S9
Market Town composition:                 ❌
Cross-renderer semantic equivalence:     ✅ AC-13 against the real contracts, from frames the real
                                            Godot client submitted, compared by the server's own
                                            definition
One world, many windows (AC-15):         ✅ two clients and an agent-driven Person on one live
                                            server; evidence names the world instance, Alice's
                                            EntityId and one event sequence
```

## Current critical-path blocker

None on the slice. Step 05 closes with `AC-15` demonstrated, so the world runs end to end: a pack
loads, two clients and an agent inhabit it, and a conversation in one window is carried forward by
the NPC met in the other.

What the slice deliberately does not have, and what each is waiting for:

```text
the scheduler and processes   S4 — nothing in the slice defers work, and the server counts it if
                              anything does
durable persistence           S5 — the world is held in memory, so AC-15's fourth evidence line
                              (same persisted state revision) has no referent yet
travel between places         later — one place is enough to prove AC-15
the 2D and 3D clients         vis/2d-generated-assets and vis/3d-human-pipeline, which adopt
                              clients/protocol/mineworld/ rather than reimplementing it
```

## Stage progress

| Stage | State |
| --- | --- |
| S1 entity/component contracts | ✅ merged `e85c889` |
| S2 action/event/observation/spatial | ✅ merged `5df2c84`; review added event payload versioning |
| S3a kernel world state | ✅ merged `4c35a57`; single-writer enforced at compile time |
| S3b systems, registry, dispatch | ✅ merged; dispatch routes, validates, resolves and reduces |
| S4 clock, scheduler, process | design frozen |
| S5 persistence and event log | medium scope; owes the erased hook at `declare` |
| S6 first systems: time, places, movement | medium scope |
| S7 world pack loading, rule controller, headless run | 🚧 pack loading and a `RuleController` landed early with the step-05 slice; `create`, `run --headless --days` and `inspect` remain |
| S8 Social Café systems | medium scope |
| S9 Market Town + AC-1 proof | medium scope |
| S10 cognition | reduced: controllers + perception only; LM half deferred to MVP-1 with AC-4 and AC-10 |
| S11 server and networking | 🚧 the server, the protocol and multi-client sessions landed with the step-05 slice; the id encoding is in the contracts (PR 04). Authentication, admin frames and deltas remain |
| S12 Demo A, 2D client | spike running; the protocol layer it adopts is merged (`clients/protocol/`) |
| S13 deployment parity and CI | medium scope; the repository now exists, so this is workflows and containers |
| S14 Demo B, 3D walking world | spike running; the protocol layer it adopts is merged (`clients/protocol/`) |

## Evidence banked so far

| Claim | How it was established |
| --- | --- |
| A renderer can be run and inspected automatically | Godot 4.7.2 windowed run through Metal, `save_png`, image read back and looked at |
| Embodied 3D interaction is feasible | spike walked a collidable room, raycast-targeted an NPC by entity id, rendered a server-supplied affordance |
| Godot ↔ Rust transport works | `ActionIntent` → axum WebSocket → `ActionResult` round trip |
| Godot corrupts 64-bit ids in JSON | server sent `9001`, client read `9001.0`; fix belongs at the protocol boundary, not in the contracts |
| Single-writer is structural | a component cannot compile without naming its owning system (E0046) |
| The write capability cannot be forged | the reproduced A8 bypass no longer compiles once the issuer is sealed (E0624 ×2) |
| `AC-13` holds against the real contracts | the two `talk` requests the real Godot module submitted — one reporting a position, one not — have an identical semantic core and are resolved to the same result. *Byte identity was the old wording and was wrong*: `MVP.md` §9's correction names `actor_location` as the permitted difference, and the comparison now lives in the server |
| `AC-15` holds | two clients and an agent-driven Person on one `mineworld server worlds/social-cafe --agent alice`; the evidence names one world instance, one Alice `EntityId` and one event sequence, and Alice tells the second window what the first one said (`tools/cli/tests/ac15_one_alice.rs`, `clients/protocol/evidence/`) |
| A controller and a client are indistinguishable to the world | the rule controller occupies a seat, is answered by the actor check and is given a server-allocated `ActionId`, exactly as a socket client is (`INV-1`) |

## Non-blocking follow-ups

```text
presentation: an interior at room scale, and a night or overcast reference
contracts:    RelationTypeId::from_static, which would make edge ownership compile-time
tooling:      one standards.py approve so the helper can run the cargo checks
```

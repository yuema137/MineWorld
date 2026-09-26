# MVP-0 status

The point of this file is to stop MVP-0 being declared complete while a real path is untested.

`✅` means **actually run and inspected**, never inferred from a passing test suite or a
successful compile. `🚧` means in progress. `❌` means not started.

**Updated:** 2026-09-26

## Capability matrix

| Capability | Headless | 2D | 3D |
| --- | --- | --- | --- |
| World boot | 🚧 composed from declarations; no runtime loop yet | ❌ | ❌ |
| Person | 🚧 contracts + storage | ❌ | ❌ |
| Spatial state | 🚧 contracts only | ❌ | ❌ |
| Movement | ❌ | ❌ | ❌ |
| Place | 🚧 contracts + storage | ❌ | ❌ |
| Conversation | ❌ | ❌ | ❌ |
| Object interaction | ❌ | ❌ | ❌ |
| Persistence | ❌ | — | — |
| Networking | ❌ | ❌ | ❌ |

## Independent axes

```text
Social Café composition:                 ❌
Market Town composition:                 ❌
Cross-renderer semantic equivalence:     🚧 harness proven on throwaway clients;
                                            real version in the renderer spike
```

## Current critical-path blocker

PR 03b's dispatch pipeline. Until an `ActionIntent` can be routed, validated, resolved into
events and reduced, nothing above `Person` can move. Everything else on the critical path —
scheduler, persistence, the first systems, the world loader — sits behind it.

Running in parallel, deliberately not behind that blocker: the renderer-integration spike, which
consumes the merged contracts directly to find out whether they survive contact with a real 2D
and 3D client while changing them is still cheap.

## Stage progress

| Stage | State |
| --- | --- |
| S1 entity/component contracts | ✅ merged `e85c889` |
| S2 action/event/observation/spatial | ✅ merged `5df2c84`; review added event payload versioning |
| S3a kernel world state | ✅ merged `4c35a57`; single-writer enforced at compile time |
| S3b systems, registry, dispatch | 🚧 C1–C3 committed, C4 dispatch in progress |
| S4 clock, scheduler, process | design frozen |
| S5 persistence and event log | medium scope; owes the erased hook at `declare` |
| S6 first systems: time, places, movement | medium scope |
| S7 world pack loading, rule controller, headless run | medium scope |
| S8 Social Café systems | medium scope |
| S9 Market Town + AC-1 proof | medium scope |
| S10 cognition | reduced: controllers + perception only; LM half deferred to MVP-1 with AC-4 and AC-10 |
| S11 server and networking | medium scope; owes a wire encoding where 64-bit ids are strings |
| S12 Demo A, 2D client | spike running |
| S13 deployment parity and CI | medium scope; the repository now exists, so this is workflows and containers |
| S14 Demo B, 3D walking world | spike running |

## Evidence banked so far

| Claim | How it was established |
| --- | --- |
| A renderer can be run and inspected automatically | Godot 4.7.2 windowed run through Metal, `save_png`, image read back and looked at |
| Embodied 3D interaction is feasible | spike walked a collidable room, raycast-targeted an NPC by entity id, rendered a server-supplied affordance |
| Godot ↔ Rust transport works | `ActionIntent` → axum WebSocket → `ActionResult` round trip |
| Godot corrupts 64-bit ids in JSON | server sent `9001`, client read `9001.0`; fix belongs at the protocol boundary, not in the contracts |
| Single-writer is structural | a component cannot compile without naming its owning system (E0046) |
| The write capability cannot be forged | the reproduced A8 bypass no longer compiles once the issuer is sealed (E0624 ×2) |
| `AC-13` is mechanically testable | the same `talk` intent arrived from a 2D click and a 3D walk-up-look-press, byte identical |

## Non-blocking follow-ups

```text
presentation: an interior at room scale, and a night or overcast reference
contracts:    RelationTypeId::from_static, which would make edge ownership compile-time
tooling:      one standards.py approve so the helper can run the cargo checks
```

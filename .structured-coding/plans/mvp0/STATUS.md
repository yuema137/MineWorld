# MVP-0 status

Updated as stages land. Exists to stop MVP-0 being declared complete while a real path is
untested. `demonstrated` means it was actually run and inspected, never inferred from a test
suite or a successful compile.

**Last updated:** 2026-09-25, during S2 implementation

## Capability matrix

| Capability | Headless | 2D | 3D |
| --- | --- | --- | --- |
| World load | not started | not started | not started |
| Person | contracts only (S1) | not started | not started |
| Movement | not started | not started | not started |
| Place | contracts only (S1) | not started | not started |
| Talk | not started | not started | not started |
| Object interaction | not started | not started | not started |
| Persistence | not started | n/a | n/a |
| Multiplayer / server path | not started | not started | not started |

"contracts only" means the type exists and is tested, with no runtime behind it.

## Independent axes

| Axis | Status |
| --- | --- |
| Social Café composition | not started |
| Market Town composition | not started |
| Cross-renderer `ActionIntent` equivalence | harness proven on probe clients; awaits the real ones |

## Stage progress

| Stage | State |
| --- | --- |
| S1 entity/component contracts | **merged** `e85c889`, 32 tests, single-writer proven at compile time |
| S2 action/event/observation/spatial | design frozen, implementing |
| S3 system interface and registry | medium scope |
| S4 clock, scheduler, process | medium scope, D-6 open |
| S5 persistence and event log | medium scope |
| S6 first systems: time, places, movement | medium scope |
| S7 world pack loading, rule controller, headless run | medium scope |
| S8 Social Café systems | medium scope |
| S9 Market Town + AC-1 proof | medium scope |
| S10 cognition | medium scope |
| S11 server and networking | medium scope |
| S12 Demo A, 2D client | medium scope |
| S13 deployment parity and CI layers | medium scope |
| S14 Demo B, 3D walking world | medium scope |

## Evidence already banked

| Claim | Evidence |
| --- | --- |
| A renderer can be run and inspected automatically | Godot 4.7.2 windowed run, Metal, `save_png`, image read back |
| Embodied 3D interaction is feasible | spike walked a collidable room, raycast-targeted an NPC by entity id, rendered a server-supplied affordance |
| Godot ↔ Rust transport works | `ActionIntent` → axum WebSocket → `ActionResult` round trip |
| Single-writer is structural | falsification probe: a component cannot compile without naming its owner (E0046) |
| A 2D client can render a world and submit an intent from a click | probe client drew the café Place, Alice inside it, Bob outside; click → `talk` intent → server accepted; screenshot inspected |
| **`AC-13` is mechanically testable** | the same `talk` intent arrived from a 2D click and from a 3D walk-up-look-press, byte identical, with the server deciding the outcome alone |

## Non-blocking follow-ups

```text
presentation: interior room-scale reference, night and overcast references
tooling:      one standards.py approve so the helper can run the cargo checks (P-2)
```

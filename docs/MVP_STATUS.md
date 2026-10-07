# MVP-0 status

The point of this file is to stop MVP-0 being declared complete while a real path is untested.

`✅` means **actually run and inspected**, never inferred from a passing test suite or a
successful compile. `🚧` means in progress. `❌` means not started.

**Updated:** 2026-10-06 (PR 09; world boot, CLI, S7 and evidence rows — PR 08's rows stand). Subjective questions are queued in
[`HUMAN_REVIEW_QUEUE.md`](HUMAN_REVIEW_QUEUE.md); a demo parked there does not block engineering.

## Capability matrix

| Capability | Headless | 2D | 3D |
| --- | --- | --- | --- |
| World boot | ✅ `mineworld run worlds/social-cafe --headless --seed 7 --days 300`: 300 simulated days, every seat moving and talking throughout, the same world byte for byte from the same seed (PR 09) | ❌ | ❌ |
| Person | 🚧 contracts + storage | ❌ | ❌ |
| Spatial state | ✅ `Presence` and `present-in`, owned by `presence` and changed only by reducing `arrived`; persisted and rebuilt identically (PR 08) | — | — |
| Movement | ✅ `move` end to end: strides of at most 2 m decided by `movement`, `too_far_away` from the server, a doorway crossing recorded as `person-entered-place`, persisted and resumed; disabling `movement` answers `move` `unavailable` and changes nothing else (`AC-2`, PR 08) | 🚧 the Godot protocol demo walks in `move` strides against the real server (2D flavour); the 2D client's own adoption is pending | 🚧 the same, 3D flavour; the 3D client's adoption is pending |
| Place | 🚧 two places joined by a doorway (`passages`, owned by `movement`) in `worlds/social-cafe`; no interiors, doors or capacity | ❌ | ❌ |
| Conversation | ✅ `talk` end to end: request, refusal for distance, event, history | 🚧 a client can speak and read what it was told; the 2D client's own adoption is pending | 🚧 the same, and the 3D client's adoption is pending |
| Object interaction | ❌ | ❌ | ❌ |
| Persistence | ✅ `mineworld server --save` killed with SIGKILL and restarted: same instance, revision, people and conversation; `mineworld replay` re-executes the save (PR 07) | — | — |
| Networking | ✅ two clients and an agent on one server, from the real binary | 🚧 protocol module runs against the real server; adoption pending | 🚧 the same |

## Shippable artefacts

What a person can actually run. None of these exists yet.

| Artefact | Command | State |
| --- | --- | --- |
| World server | `mineworld server worlds/social-cafe --agent alice` | ✅ loads the pack, serves two clients and drives Alice with a rule controller |
| 2D client | `mineworld-2d` | 🚧 presentation spike runnable, awaiting style decision ([queue](HUMAN_REVIEW_QUEUE.md)) |
| 3D client | `mineworld-3d` | 🚧 presentation spike runnable with three camera modes; awaiting feel review ([queue](HUMAN_REVIEW_QUEUE.md)) |
| Developer CLI | `mineworld create / validate / run / inspect` | ✅ all four, plus `server` and `replay` (PR 09; `docs/MODULE_SPEC.md` §8.1) |
| `worlds/social-cafe` | `mineworld validate worlds/social-cafe` | ✅ a café and the street outside it, joined by a doorway; four people, three seats, loaded and hosted |
| `worlds/market-town` | — | ❌ |

## Independent axes

```text
Social Café composition:                 🚧 presence + movement + conversation; the economy set is S9
Market Town composition:                 ❌
Cross-renderer semantic equivalence:     ✅ AC-13 against the real contracts, from frames the real
                                            Godot client submitted, compared by the server's own
                                            definition
One world, many windows (AC-15):         ✅ two clients and an agent-driven Person on one live
                                            server; evidence names the world instance, Alice's
                                            EntityId, one event sequence and — since PR 07 — one
                                            persisted state revision, held by the save on disk
```

## Current critical-path blocker

None on the slice. Step 05 closes with `AC-15` demonstrated, so the world runs end to end: a pack
loads, two clients and an agent inhabit it, and a conversation in one window is carried forward by
the NPC met in the other.

What the slice deliberately does not have, and what each is waiting for:

```text
the scheduler and processes   S4 — nothing in the slice defers work, and the server counts it if
                              anything does
a travel Process              later — walking through a doorway between adjoining places exists
                              (PR 08); travel that takes simulated time does not
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
| S4 clock, scheduler, process | ✅ merged `1241cab` |
| S5 persistence and event log | ✅ merged `41d4ab1`: journal + fact log + snapshots in one SQLite file, verified re-execution (`ARC-25`) |
| S6 first systems: time, places, movement | 🚧 PR 08 ready for review: `movement` decides, `presence` owns (`ARC-26`); `arrive` retired; `passages` in the World Pack format; the street in social-cafe |
| S7 world pack loading, rule controller, headless run | 🚧 PR 09 ready for review: `run --headless --seed --days`, `inspect`, `create`; a seeded paced rule controller (`ARC-27`); `clap` (`DEP-11`) |
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
| A world survives the death of its process (`AC-6`) | a child process SIGKILLed mid-run and a new one resuming its SQLite file produce a save byte-identical to an uninterrupted run (`persistence/tests/kill_and_resume.rs`); the real `mineworld server --save`, SIGKILLed and restarted, is the same world, and Alice's conversation continues (`tools/cli/tests/restart.rs`) |
| A capability is removable (`AC-2`, first real evidence) | with `movement` disabled, `move` is `unavailable`, records nothing and is not offered, and the world is otherwise identical to one that never had it; the same comparison against a world where movement stays reachable reports 7 violations, so the test cannot pass by accident (`systems/movement/tests/movement.rs`) |
| A per-request stride bound does not punish an honest client | a client jogging at 2.6 m/s that reports before travelling 2 m since its last accepted position is never refused (29 accepted, 0 refused); one reporting once a second is refused every time (`systems/movement/tests/movement.rs`, the reporting rule of `server/PROTOCOL.md` §6.2) |
| Hundreds of days, headless, with the real systems (`AC-11`) | 300 simulated days of social-cafe with presence, movement and conversation, no renderer, no model: every seat has accepted moves and talks in every 30-day bucket, the street is entered, no fault — 141 043 facts (`tools/cli/tests/run.rs`) |
| The same seed is the same world (`AC-12`) | two saved 300-day runs of one seed are byte-identical in facts, journal and snapshots; another seed differs; the instance identity is the one excluded field (`ARC-27`, `tools/cli/tests/run.rs`) |
| A killed run finishes the same world (`AC-6` with controllers) | `run --save` SIGKILLed at days 5, 15 and 25 and run again, and a run stopped and continued, each byte-identical to an uninterrupted run; no line is ever answered twice, so step-06's `F-13` does not arise in `run` (`tools/cli/tests/run_restart.rs`) |
| Every fact has a cause (`AC-9`) | `mineworld inspect` resolves every action and event cause in a month-long save, and fails by name on a forged one (`tools/cli/tests/inspect.rs`) |
| A controller and a client are indistinguishable to the world | the rule controller occupies a seat, is answered by the actor check and is given a server-allocated `ActionId`, exactly as a socket client is (`INV-1`) |

## Non-blocking follow-ups

```text
presentation: an interior at room scale, and a night or overcast reference
contracts:    RelationTypeId::from_static, which would make edge ownership compile-time
tooling:      one standards.py approve so the helper can run the cargo checks
```

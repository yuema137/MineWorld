# MVP-0 status

The point of this file is to stop MVP-0 being declared complete while a real path is untested.

`✅` means **actually run and inspected**, never inferred from a passing test suite or a
successful compile. `🚧` means in progress. `❌` means not started.

**Updated:** 2026-10-07 (S9 complete: PR 11f merged as `fea2516`; AC-1 demonstrated as ARC-35 measures it, within ARC-33's static-linking boundary, operator acceptance pending; Milestone C demonstrated, awaiting the operator's review; next is S15's PR 12a; earlier rows stand). Subjective questions are queued in
[`HUMAN_REVIEW_QUEUE.md`](HUMAN_REVIEW_QUEUE.md); a demo parked there does not block engineering.

## Capability matrix

| Capability | Headless | 2D | 3D |
| --- | --- | --- | --- |
| World boot | ✅ `mineworld run worlds/social-cafe --headless --seed 7 --days 300`: 300 simulated days, every seat moving and talking throughout, the same world byte for byte from the same seed (PR 09) | ❌ | ❌ |
| Person | 🚧 contracts + storage | ❌ | ❌ |
| Spatial state | ✅ `Presence` and `present-in`, owned by `presence` and changed only by reducing `arrived`; persisted and rebuilt identically (PR 08) | — | — |
| Movement | ✅ `move` end to end: strides of at most 2 m decided by `movement`, `too_far_away` from the server, a doorway crossing recorded as `person-entered-place`, persisted and resumed; disabling `movement` answers `move` `unavailable` and changes nothing else (`AC-2`, PR 08) | 🚧 the Godot protocol demo walks in `move` strides against the real server (2D flavour); the 2D client's own adoption is pending | 🚧 the same, 3D flavour; the 3D client's adoption is pending |
| Place | 🚧 the MVP town in `worlds/social-cafe`: apartments, café, park, store and office, each with one doorway onto a street (`passages`, owned by `movement`), every place entered in a 300-day headless run (PR 10a). No interiors beyond positions, no doors that close, no capacity | ❌ | 🚧 the café's positions match the 3D slice, so a person at the counter can `talk` to Alice; the other places have no drawn geometry |
| Conversation | ✅ `talk` end to end: request, refusal for distance, event, history | 🚧 a client can speak and read what it was told; the 2D client's own adoption is pending | 🚧 the same, and the 3D client's adoption is pending |
| Complete affordances | 🚧 an offer may carry the exact request its pack would accept (`Affordance.payload`, `Offer::complete`), and the headless controller attempts one it was never compiled against, at a fixed rate frozen before the market exists — shown with a synthetic pack in `tests/acceptance` (PR 11c, `ARC-34`). No existing pack offers one yet | 🚧 the field is in every observation frame and `ADOPTION.md` says how to submit it; no client uses it before S12 | 🚧 the same |
| Social life | ✅ `group-activity`: invite, accept, decline, join, leave, with an activity that is a `Process` its owner ends. `relationships`: `knows` edges and values changed only by reacting to those facts and to `spoke`. A 300-day run forms ~6 300 activities. Known gap: relationships never decay and saturate after ~60 days (PR 10b, `ARC-28`) | 🚧 the protocol demo shows `invite` affordances and the observer's own `acquaintances`; no client UI for them yet | 🚧 the same |
| Biography | ✅ `mineworld biography <world> --save DIR --person KEY`: derived from the fact log, never stored, checked complete and sound against the facts' typed payloads (PR 10b, `ARC-29`); shows people by name as well as key (PR 10c) | — | — |
| Names | ✅ `naming`: each person file's `name:` becomes a `display-name` disclosed to whoever perceives the person; the rule controllers name people by it ("Earlier, Vera Lindgren said …"), never by id; removable (`AC-2`) (PR 10c, `ARC-31`) | 🚧 the protocol demo labels people by name, through `MineWorldObservation.display_name`, against the real server; the 2D client's own adoption is pending | 🚧 the module reads it; the 3D slice's binding is the vis track's |
| Routines | ✅ `schedule`: each person file's `routine:` becomes a day kept by a `Process` that wakes at every boundary and moves nobody; the headless controller follows it and every seat reaches ≥ 90 % of its day's parts in 30 days; removable (`AC-2`) (PR 10c, `ARC-32`). Known gap: people walk ~8 m/h at the headless pace, so each part of a day lasts ≥ 4 h | 🚧 a seat is disclosed its own `agenda`; no client shows it yet | 🚧 the same |
| Authored content owned by packs | ✅ a System Pack owns a section of a person or place file, validates it with its own type (refusals keep line and column) and seeds its own facts; the loader never learns what it means (PR 10c, `ARC-31`) | — | — |
| Items and organizations as World Pack content | 🚧 `world.yaml`'s `items:` and `organizations:` name files in `items/` and `organizations/`, carrying tags, a note and sections. An authored Item is an item kind, so stacked items only. Their ids come after people's, so no existing id or genesis fact moves, and keys are one namespace across the four lists. Bad files are refused by name and file. No installed pack owns a section on them yet (PR 11b, `ARC-36`) | — | — |
| Independently installable System Packs (MVP-0 form) | 🚧 a pack declares itself (`SystemPack`, `sdk/rust`); installing one is a directory, two lines in `systems/installed` and a rebuild, and edits nothing in the loader, the CLI or the root manifest — shown with a canary pack on a scratch branch, never merged (PR 11a, `ARC-33`). Without a rebuild is `ARC-8`, outside MVP-0 | — | — |
| Owning and giving things | 🚧 `item` declares kinds from item files' `item:` sections; `inventory` owns what people and organizations hold (`holdings:`), is the only writer of it, and refuses any fact it may not take; a person carries at most six; `item-transfer` offers a complete `give` per kind held to each other person within 3 m, and the unchanged headless controller gives. `worlds/market-town` is Social Café plus these three packs and their content (PR 11d, `ARC-37`). Money, work, shops and eating: the next row (PR 11e) | — | — |
| Work, money, shops and eating | 🚧 `economy` owns wallets and shops (`economy:` on person and organization files), offers a complete `buy` per priced kind in a shop and is the only mover of money, in integer minor units; `employment` owns two jobs (`job:`), a shift `Process` that pays for the time a person is at the workplace and restocks the employer, and states `wage-due`, which economy pays or records `wage-unpaid`; `consumption` offers `eat` and `drink` for the food and drink a person carries; `inventory` gained `items-produced` and `items-consumed`, still the only writer of holdings. The unchanged headless controller buys, eats and drinks; two people work by following the routines they already had. People without a job live on an endowment sized for 300 days — a bounded horizon, not a closed economy (L-13) (PR 11e, `ARC-38`) | 🚧 a seat is disclosed its wallet and, in a shop, the listing; no client shows them yet (S12) | 🚧 the same |
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
| Developer CLI | `mineworld create / validate / run / inspect / biography` | ✅ all five, plus `server` and `replay` (PR 09, `biography` PR 10b; `docs/MODULE_SPEC.md` §8.1) |
| `worlds/social-cafe` | `mineworld validate worlds/social-cafe` | ✅ the MVP town: six places joined by a street, twelve people, eleven seats; the café laid out as the 3D slice draws it; loaded, hosted and run headless (PR 10a) |
| `worlds/market-town` | `mineworld run worlds/market-town --headless --seed 7 --days 300` | ✅ Social Café plus six installed packs and their content: twenty item kinds that people hold and give (PR 11d); two organizations running the café and the store, wallets for everyone, two jobs, and people buying, eating and drinking for 300 days with no wallet drained (PR 11e). Proven by PR 11f: AC-1, CP-4 at 300 days, AC-2 per market pack, Milestone C (evidence rows below) |

## Independent axes

```text
Social Café composition:                 ✅ presence + movement + conversation + group-activity +
                                            relationships + naming + schedule (PR 10c, awaiting
                                            review); the economy set is S9's Market Town
Market Town composition:                 ✅ Social Café + item + inventory + item-transfer +
                                            economy + employment + consumption (PRs 11d, 11e);
                                            AC-1 demonstrated as ARC-35 measures it, within
                                            ARC-33's static-linking boundary (PR 11f,
                                            tests/acceptance/tests/ac1_composability.rs)
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
| S8 Social Café systems | 🚧 three PRs (`step-09-social.md`). 10a merged (`2f24eef`): the MVP town and the café re-authored to the 3D slice. 10b merged (`85451c7`): `group-activity`, `relationships`, `mineworld biography`, the paced controller's social initiative, `[profile.dev] opt-level = 1` (`ARC-30`), and Milestone B. PR 10c ready for review: the content seam (`ARC-31`), `naming`, `schedule` (`ARC-32`), names in replies, biography and the Godot module, the controller following its day |
| S9 Market Town + AC-1 proof | ✅ complete 2026-10-07: six PRs (`step-10-market.md`); how `AC-1` is measured is `ARC-35`. The three precursors are merged: 11a (`c472636`) installable System Packs (`ARC-33`, `DEP-12`) and the I-2 vocabulary scan; 11b (`ae1a315`) items and organizations as World Pack content (`ARC-36`); 11c (`c5dc51c`) complete affordances (`ARC-34`). The measured transformation's first half is merged: 11d (`70e532f`) `item`, `inventory`, `item-transfer` and `worlds/market-town` (`ARC-37`), its merge diff touching only `systems/`, `worlds/`, `Cargo.lock` and Markdown. The second half is merged: 11e (`2dddda8`) `economy`, `employment` and `consumption`, with `inventory`'s production and consumption facts (`ARC-38`), its merge diff likewise touching only `systems/`, `worlds/`, `Cargo.lock` and Markdown. The measured transformation is complete. The proof is merged: 11f (`fea2516`) — `ac1_composability` (ARC-35's three checks, passing 13/13 on `main` itself with full history), CP-4 as a committed 300-day test, AC-2 at world level for the six market packs, and Milestone C through the real server. AC-1 is demonstrated as ARC-35 measures it, within ARC-33's static-linking boundary; the operator's acceptance is pending (QS-65). Milestone C is demonstrated, awaiting the operator's review. Known limits carried: F-47, F-48, F-41, L-12, L-13, relationship saturation, QS-10 (`sleep`, needs), DP-5 (Alice's routine and shift both start at 05:30, so she is always late — content, for a later step), F-58, and S13's CI must fetch full history (`fetch-depth: 0`) |
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
| Hundreds of days, headless, with the real systems (`AC-11`) | 300 simulated days of social-cafe with presence, movement and conversation, no renderer and no model. Since PR 10a this is the twelve-person town: all eleven seats have accepted moves and talks in every 30-day bucket, every one of the six places is entered (counted per place), and there are no faults and no refused strides — 327 540 facts (`tools/cli/tests/run.rs`) |
| A person at the café counter can talk to Alice | walked there from the visitor's seat in accepted strides, `talk` is accepted; from the doorway it is refused `too_far_away`. On the pre-S8 café the same walk was refused at the counter, which is the 3D client's old failure (`worldpack/tests/social_cafe.rs::a_person_at_the_counter_can_talk_to_alice`, PR 10a) |
| The same seed is the same world (`AC-12`) | two saved 300-day runs of one seed are byte-identical in facts, journal and snapshots; another seed differs; the instance identity is the one excluded field (`ARC-27`, `tools/cli/tests/run.rs`) |
| A killed run finishes the same world (`AC-6` with controllers) | `run --save` SIGKILLed at days 5, 15 and 25 and run again, and a run stopped and continued, each byte-identical to an uninterrupted run; no line is ever answered twice, so step-06's `F-13` does not arise in `run` (`tools/cli/tests/run_restart.rs`) |
| Every fact has a cause (`AC-9`) | `mineworld inspect` resolves every action and event cause in a month-long save, and fails by name on a forged one (`tools/cli/tests/inspect.rs`) |
| Milestone B: Alice and Bob know each other, share an activity, and survive a restart with their history | `run --save` SIGKILLed after the located history, which is shown already on disk, then run again to a world byte-identical to the control. The history: both became acquainted, a level crossed, an activity both took part in. Both biographies, read by fresh processes, are unchanged. The save is then hosted by `mineworld server`, SIGKILLed and restarted, and Alice and Bob read their own relationship values unchanged (Close, 26 activities shared) (`tools/cli/tests/milestone_b.rs`, PR 10b) |
| A biography matches the log that produced it | Alice's biography (428 entries, all six biographical types) equals the set of facts whose own typed payload names her. Dropping a type, ignoring participants, or ignoring the person each fails the test (`tools/cli/tests/biography.rs`, PR 10b) |
| Relationships and group activity are removable (`AC-2`) | without `relationships`, every other system's 33 486 facts in 30 days are identical in order and content to the full run's. Without `group-activity`, relationships stays enabled, the world runs, and acquaintance comes from speech alone. The comparison is shown to see seed 7 against seed 8 (`tools/cli/tests/social_composition.rs`, PR 10b) |
| A person's day is kept by a Process, and people follow it (CP-4) | in a 30-day run every agenda change after genesis is its person's routine process waking on a boundary of their authored day; every seat reaches 96.7–100 % of its day's parts (≥ 90 % required, fixed before measuring); with the controller's agenda band off, 39–51 %; Otto, whom nobody drives, never moves while his day goes by (`tools/cli/tests/routines.rs`, PR 10c) |
| People are named, and removing names changes nothing else (`AC-2`) | Alice tells the 3D window "Earlier, Vera Lindgren said …", the name read from the observation, through the real server and the real Godot module (`ac15_one_alice.rs`, `clients/protocol/evidence/`); without `naming`, 33 951 facts in 30 days are the same row for row, and only replies' words differ, naming nobody (`social_composition.rs`, PR 10c) |
| Item kinds and organizations change nothing a world does not use (`ARC-36`) | social-cafe copied with two item kinds and one organization: `validate` lists them as ids 19–21 after social-cafe's unchanged 1–18, with the same 53 genesis facts. A 10-day seed-7 run's fact table is byte-identical to social-cafe's own, and the world resumed at day 5 equals its uninterrupted run in facts, journal and snapshots (`tools/cli/tests/content_kinds.rs`, PR 11b) |
| Installing a System Pack touches only `systems/` and `Cargo.lock` (F-1) | a canary pack (`wave`, `waved`) installed on a scratch branch changed only `systems/canary/**`, two lines of `systems/installed/` and `Cargo.lock` (one new path package, one dependency line), plus the scratch world enabling it; `mineworld validate` listed it and a day ran with no fault; the loader's suite passed unchanged. Removing a line from the installed list, or naming a pack in `worldpack`, fails a named guard (PR 11a, `step-10-market.md` §9 E-A5) |
| A controller and a client are indistinguishable to the world | the rule controller occupies a seat, is answered by the actor check and is given a server-allocated `ActionId`, exactly as a socket client is (`INV-1`) |
| A headless person uses a pack the controller has never seen (CP-3, F-3) | a test-only pack, `chimes`, offers a complete `ring` per bell at its belfry; driven on `mineworld run`'s schedule, the two people there ring every day for ten days, the two elsewhere never ask, every `rang` is caused by the request that asked for it, two runs are byte-identical, and with `chimes` disabled nobody asks; removing the band's call fails it (`tests/acceptance/tests/complete_affordances.rs`). The band's rate (20 of 100) was fixed on a scratch install offering a ring to everyone everywhere for 300 days: every seat still moved and talked, and rang, in every 30-day bucket. Social Café's 300-day run is unchanged byte for byte (PR 11c, `step-10-market.md` §9.3 E-C6) |
| People in Market Town own things and keep giving them for 300 days | `mineworld run worlds/market-town --headless --seed 7 --days 300 --save`: no fault, every seat moved and talked in every 30-day bucket, and — read from the save — every seat gave in every bucket (at least 114 times; 17 639 gives in all), nobody ever held more than six, and the 30 authored items were all still there. Only then: two 30-day runs identical, and a run stopped at day 15 and resumed equals the uninterrupted one fact for fact; 32.5 s with `--save`. With the capacity removed the unseated Otto absorbs all 30 items and giving stops in the second month. Installing the three packs left Social Café's 300-day run byte-identical (PR 11d, `step-10-market.md` §9.4 E-D6). True of 11d's merge; superseded as the current Market Town evidence by the next row, because after 11e items are produced, bought and consumed and are no longer conserved |
| Market Town works, buys, eats and gives for 300 days, and no wallet drains | `mineworld run worlds/market-town --headless --seed 7 --days 300 --save`: no fault, every seat moved and talked in every 30-day bucket, and — read from the save, against conditions stated before the first run — every bucket has purchases (247–349), a wage paid to each of the two job holders (28–30 each), production (194–203), meals (251–358) and gives (1 253–1 382); zero `wage-unpaid` in 590 wages due; no wallet ever below the cheapest price (lowest: Felix 2 313); money conserved (2 360 000); nobody above six. Only then: two 30-day runs identical, and a run stopped at day 15 and resumed equals the uninterrupted one fact for fact; 33.5 s with `--save`. With `consumption` removed, purchases and gives stop after the first month and the café cannot pay Alice (257 wages unpaid). Installing the three packs left Social Café's 300-day run byte-identical (PR 11e, `step-10-market.md` §9.5 E-E7) |
| AC-1: Market Town is Social Café plus installed packs and configuration | AC-1 demonstrated as ARC-35 measures it, within ARC-33's static-linking boundary. Three independent checks in a committed test: the two transformation merges (11d `70e532f`, 11e `2dddda8`), found by GitHub subject and pinned by id, change only `systems/`, `worlds/`, `Cargo.lock` (six path packages under `systems/`, nothing external) and Markdown; from `cargo metadata`, only `systems/` depends on a market pack, no normal or build path leads to one from kernel, contracts, persistence, server, authoring, sdk or the rule controller, and no code outside `systems/`, `worlds/`, `tests/acceptance/` names one; Market Town's files are Social Café's, unchanged, plus sections the six packs own. Each check is shown to fail by name under planted violations, and check 1 fails rather than skips on a shallow clone. Installing means a directory, two lines in `systems/installed` and a rebuild; without a rebuild is `ARC-8`, outside MVP-0 (`tests/acceptance/tests/ac1_composability.rs`, PR 11f, `step-10-market.md` §9.6 E-P2 … E-P4) |
| The market lives for 300 days, as a committed test (CP-4) | `tools/cli/tests/market_town.rs`, in the default suite: from a 300-day seed-7 save, every 30-day bucket has purchases (247–349), a wage paid to each of the two job holders, production, meals and a give from every seat; zero `wage-unpaid`; no wallet below the cheapest price; nobody above six; and the wallets and holdings in the newest snapshot equal the replay of the facts, total money 2 360 000 = genesis. Only then are runs compared: twin 30-day saves byte-identical, a run SIGKILLed after day 15 and re-run equal to the control, seed 8 different. Fails by name with `consumption` removed, with the payer never debited, and with `buy` offered incomplete (PR 11f, §9.6 E-P5) |
| Each market pack is removable (`AC-2`) | `tools/cli/tests/market_composition.rs`: a copy of Market Town without `item` or `inventory` is refused by `validate`, naming the dependency; without `item-transfer`, `economy`, `employment` or `consumption` it runs 30 days with no fault and every seat active, that pack's interactions absent and the rest of the market present (PR 11f, §9.6 E-P6) |
| Milestone C: work → earn → buy → holdings change → another client sees it → it survives a restart | `tools/cli/tests/milestone_c.rs`: a 2-day save locates Alice hired, arriving at the café during her shift, the shift worked, the wage due and paid. `mineworld server` hosts the save; Alice and Bob walk into the café through the pack's doorways; Alice submits an offered complete `buy` unchanged; her wallet falls by the listed price and she holds one more; Bob sees the café's stock one lower and none of Alice's private state. SIGKILL and restart: same instance, revision, wallet, holdings and listing; `inspect` resolves every cause. Demonstrated, awaiting the operator's review (PR 11f, §9.6 E-P7) |

## Non-blocking follow-ups

```text
presentation: an interior at room scale, and a night or overcast reference
contracts:    RelationTypeId::from_static, which would make edge ownership compile-time
tooling:      one standards.py approve so the helper can run the cargo checks
social life:  relationship values never decay, so a long world's social graph saturates within ~60
              days (ARC-28, step-09 QB-2) — a living-world gap for a later step
```

# Step 21 — S21: Regions, travel and maps (one world, many regions)

**Lifecycle:** `DRAFT, awaiting primary review`. Step-level design only. Nothing here is frozen and
nothing here authorizes implementation. Every decision id below is a placeholder (`ARC-RT-a` …,
`DEP-RT-a` …) to be numbered by the primary session.
**Effort:** `mvp1` (MVP-1's main line, operator ruling 1). This is the first document in
`.structured-coding/plans/mvp1/`. There is no `mvp1/overall.md` yet; until there is, the parents
are `mvp0/overall.md` and the operator rulings quoted in §1.
**Author:** the S21 planning session, 2026-10-09. Worktree
`/Users/yuema137/mineworld-worktrees/plan-s21`, branch `plan/s21-regions-travel`, from
`origin/main @ f80bbb7`.
**Binding parents:** `CLAUDE.md` §§2–4 (especially §4 rules 1, 2, 3, 14, 15, 16);
`docs/CORE_CONCEPTS.md` §§6, 9, 10, 15, 17; `docs/ENGINEERING_RULES.md` §§5–9, 11–12;
`docs/ARCHITECTURE.md` §§5–6, 10; `docs/MODULE_SPEC.md` §§1, 4, 6; `docs/DECISIONS.md` `ARC-26`,
`ARC-31`, `ARC-32`, `ARC-38`, `ARC-39`, `ARC-41`, `ARC-45`, `ARC-46`, `ARC-54`, `ARC-55`, `ARC-61`,
`ARC-62`, `ARC-67`, `DEP-4`, `DEP-8`; `mvp0/overall.md` "Framework, not demo", "One world, two
views", "The World Interaction List"; `mvp0/step-19-time-weather.md` §4 and QTW-13;
`mvp0/step-11-bodies.md` §21 (12n, PR #100, frozen on its branch); `mvp0/step-12-server.md`
(S11-B seats, S11-C audience); open PR #108 (`overall.md` section "One world of many regions,
linked by transport").
**Scope of this file.** It is the only file this session writes. Edits to `CORE_CONCEPTS.md`,
`MODULE_SPEC.md`, `DECISIONS.md` and other lanes' documents are *proposed* here and applied by
the primary session or the owning lane.

---

# 1. The requirement

## 1.1 The operator's words (2026-10-09, binding)

> "下一个我们需要解决的问题，就是怎么样才能让不同的小世界连起来，产生了一个有大世界的感觉。但是当然我们不可能一次把所有的资源，整个世界都加载……我们的这些素材比Minecraft都要细节很多，没有办法学习他们的实时渲染……我们应该把不同的小世界用交通工具给联系起来，然后交通工具一律用火车或者汽车。乘坐交通工具的时候，就按照当前地点和要去的地点的风景特征之类的安插一小段。车窗动画等等……到了一个新的小世界，就可以再加载那一边的素材等等，不需要一次性加载一个大世界。然后我们肯定是需要有一个地图的功能"

In English: link the small worlds with transport — trains or cars only — so that the whole feels
like one big world without loading everything at once. Our assets are far more detailed than
Minecraft's, so its real-time approach cannot be copied. During a ride, show a short sequence
(window animation and the like) chosen from the landscape features of the origin and the
destination. On arrival in a new small world, load that side's assets. A map is required.

## 1.2 Operator rulings (binding)

1. **Design now; implement after MVP-0 as MVP-1's main line.** Nothing in MVP-0 may preclude it
   (§10 is the audit).
2. **While riding, the player can move about the carriage and talk to NPCs and players on board,
   with scenery animating outside the windows. The carriage is itself a small place.**
3. **One world, many regions.** One server, one timeline, every region simulated all the time. NPCs
   may live across regions. A world author composes several region packs into one world.
4. **Cars (2026-10-09, relayed by the coordinator):**

   > "汽车可以选公交也可以选出租车。自己开车的选项只在一个区域内部可以，区域之间通行默认不能自己开。但是我们之后可以额外设置road trip模式（只有对特定的区域之间才可以），不过这个是长期计划，不是现阶段要处理的。"

   That is: cars come as buses and as taxis. Self-driving is allowed only within a region; travel
   between regions cannot be self-driven by default. A later "road trip" mode may allow
   self-driving between specific region pairs; that is a long-term plan, not part of this step.
   Design consequence (§5.4): buses are road lines with timetables; taxis are on demand;
   in-region self-driving is a design-only placeholder with no PR in this split; the data model
   keeps a per-link `self_drive` flag (default `forbidden`) so road trips are additive later.

## 1.3 The primary session's framing (binding)

- **Simulation is cheap; presentation is heavy.** The whole world's simulation stays loaded on the
  server; clients stream assets per region. The claim is measured in §2.8.
- **Rule 14:** `MoveIntent`, the travel `Process`, authoritative spatial state and rendered movement
  stay four distinct things (§5.10).
- **Ownership.** Transport is an optional **System Pack**. The kernel stays ignorant. Whether
  "region" needs an ontology extension is decided with evidence (§5.1).
- **Time domains (S19 §4, QTW-13):** a trip's duration is calendar time; walking in the carriage is
  embodied; the scenery's length follows the trip in wall time and doubles as the destination's
  loading screen; pause applies.
- **Maps:** a local region map and a world map, served by the server and drawn by both clients. No
  rule lives in a client.
- **Also:** all three platforms; real-world defaults with sources; framework, not demo (authors
  write their own regions, lines and scenery tags); coordinate with 12n; multiplayer across regions
  through S11-C's audience and S11-B's seats.

## 1.4 The answer in brief

```text
World Pack  ──regions:──►  region directories (or required region packs), each a set of places,
                           people and items in its own namespace, plus a region.yaml
                           (map placement, landscape tags)
Region      =  a Place with tag `region` at the top of a `contains` hierarchy (no new primitive)
transport   =  System Pack: lines, stops, vehicles (each vehicle's interior is a Place),
               a vehicle run is a Process in calendar time, `board` / `alight` / `travel-to`
atlas       =  System Pack: discloses the region map and the world map as records
fares       =  optional System Pack: tickets are inventory item kinds sold by economy's shops
clients     =  load one region's presentation bundle at a time; while riding, play a scenery
               sequence chosen from the run's disclosed landscape tags, sized to the remaining
               calendar time divided by the time scale, and load the destination in the background
```

The kernel, the contracts and `presence`'s audience rule are not edited.

---

# 2. Audit (`origin/main @ f80bbb7`, 2026-10-09; open PRs read on their branches)

Every finding was read in this session or by an audit agent of this session from the file named.

## 2.1 Places, frames and hierarchy

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-1 | `Location { place, local?, facing? }`; `LocalPosition` is `i32` mm **from the place's own origin**. No world frame exists anywhere. | `contracts/src/spatial.rs:114–124, 267–278`; `CORE_CONCEPTS.md` §§6.1–6.2 | A carriage interior has its own frame that does not move with the train: walking in a moving carriage is well defined today, with no new contract. Regions need no shared frame. |
| A-2 | "Hierarchy is a Relation, not a field." `CORE_CONCEPTS.md` §6 already shows `World → Lakewood → Downtown → Cafe → rooms`. The relation machinery exists (`connected_to`); `contains` appears only in tests. No pack declares a place-to-place relation. | `CORE_CONCEPTS.md` §6, §6.2 item 3, §9; `contracts/src/relation.rs:95,150`; `contracts/tests/observation.rs:447–518` | A region is expressible as a Place at the top of a `contains` hierarchy without a new primitive (§5.1). Some pack must own the relation. |
| A-3 | `SpatialRequirement::evaluate`: wrong place → `TooFarAway`, "the difference between the wrong room and the wrong town is only how far they must travel". Distance is only compared within one place. | `contracts/src/spatial.rs:470–516` | Every existing action already refuses correctly across regions. |
| A-4 | `EntityKey` is `[a-z0-9-_]{1,64}`; no `.`. Keys are one namespace across places, people, items and organizations. | `contracts/src/ids.rs:110,396`; `worldpack/src/read.rs:325`; `MODULE_SPEC.md` §4.1 | Two regions cannot both have `cafe`. Region composition needs a namespacing rule (§5.2). |
| A-5 | `EntityType` is Person, Place, Item, Organization. | `contracts/src/ids.rs:693–707` | A vehicle needs no new entity type: its interior is a Place, and `CORE_CONCEPTS.md` §1 already says `Vehicle = Entity + TransportComponent + …`. |

## 2.2 Passages and movement

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-6 | `Passage { to, here?, there? }` owned by movement; stated **at genesis only**, always open, one per place pair; `reachable` requires being within `MAX_STRIDE` (2 000 mm) of the doorway on both sides; anything else is `TooFarAway` — "that is travel". | `systems/movement/src/{component.rs:16–87, event.rs:19–53, system.rs:228–254}` | A transport link must **not** be a `Passage`: it would be always open, one per pair, and (A-11) glue two regions' frames together. Boarding is transport's own action. |
| A-7 | `ARC-26` "Room for travel": a future travel system depends on presence and states `Arrived` when its process ends; `move` refuses exactly what it would accept. | `DECISIONS.md:1806–1809` | `transport` states presence's `arrived` for board and alight, under `ARC-26`. Already agreed. |
| A-8 | `ARC-39` / `ARC-62`: presence's `ArrivalResolver` catalog — bodies decides where an arriving person actually ends up. | `DECISIONS.md` `ARC-39`, `ARC-62`; `systems/README.md` | Boarding into a crowded carriage is resolved by bodies like any arrival; transport does not compute collisions. |
| A-9 | Kernel `Process { id, process_type, owner, participants, place?, started, expected_end, phase, interruptibility, state: Vec<u8> }`; woken in calendar time. | `kernel/src/process.rs:99–112`; `kernel/src/view.rs:332–368` | A vehicle run is a Process owned by transport, its route in `state`. No kernel change. |
| A-10 | **12n (PR #100, frozen on its branch):** `walk-to { Place | Person }` breadth-first over `Passages`; the walk is movement's `Walking` component, not a Process, and takes no calendar time; strides are `walk-step` requests at one per wall second. §21.1 explicitly leaves "travel between places that do not adjoin by a passage chain" to the later travel system, and §21.3 rejects a `wayfinding` pack "now; revisit with long-distance travel". | `origin/plan/s15-12n:…/step-11-bodies.md` §§21.1–21.5 | The two scales split cleanly: 12n is "walk to X within a passage-connected region"; S21 is "travel to region Y", which composes `walk-to` legs with rides (§5.6). |

## 2.3 Geometry and the clients' layout

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-11 | The 2D client stitches "one town" from disclosed passages by translation; an unconnected place is put at `_far_root()`, 200 m east. The 3D slice hard-codes `cafe` and `street`. | `ARC-45`; `clients/2d/scripts/town.gd:148–172`; `clients/3d-spike/scripts/slice/{slice_world.gd:18–23, slice_link.gd:63–77}` | A carriage is an unconnected place: the 2D client would draw it 200 m off. Clients must scope their stitched layout per region and treat a vehicle interior as its own scene (§10, N-3). |
| A-12 | Bodies' `PlaceShape` (≤ 64 axis-aligned solids) per place, `COORDINATE_BOUND` ±100 m per place. | `systems/bodies/src/{component.rs:114–117, geometry.rs:67}` | A carriage (≈ 20 × 3 m) fits; "a region is one big place" does not, which confirms that a region is a hierarchy of places. |
| A-13 | Movement discloses `Passages` and bodies discloses `PlaceShape` **only for the observer's current place**. No map message exists anywhere. | `systems/movement/src/system.rs:188–219`; `systems/bodies/src/system.rs:295–325` | A map needs a new disclosure, owned by a pack (§5.8). |

## 2.4 World Pack composition

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-14 | Flat `places/<key>.yaml`, no sub-directories; ids allocated places → people → items → organizations, each in key order; genesis order passages, locations, configuration, sections. | `worldpack/src/{read.rs:325,574,685, load.rs:1–35,300–345}` | Region composition is a loader change (`worldpack`, not kernel). Deterministic id order must be preserved (§5.2). |
| A-15 | **A world cannot require a World Pack:** `PackType::WorldPack => "a world is not a part of another world"`. Entity Packs are refused until S16 E-d. | `packages/src/resolve.rs:217–222` | Region packs need either a relaxed rule or a pack role (§5.2, QRT-2). |
| A-16 | `calendar` has one `utc_offset`, latitude and longitude per world. S19's QTW-12 already defers regional weather: "additive later, one `climate` Process per region". Weather and calendar disclose world-level state "as records about the place the observer is in". | `systems/calendar/src/configuration.rs`; `step-19-time-weather.md` §5.5, §6.6, QTW-12 | Per-region weather is additive; nothing blocks it (§5.11). |
| A-17 | No two-region world exists. `social-cafe` and `market-town` are two separate worlds of six places each, arranged as a star around `street`. | `worlds/*/world.yaml` | The S21 fixture world composes both towns as regions (§8, RT-d). That is the step's AC-1-style proof. |

## 2.5 Schedule, economy and controllers

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-18 | `schedule` is an agenda kept by a Process and never moves anyone (`ARC-32`). Accepted limitation: "Routes are found only in a star town." | `DECISIONS.md` `ARC-32`; `systems/README.md` | A routine entry may name a place in another region. Getting there is the controller's choice, served by `travel-to` (§5.6). Schedule is unchanged. |
| A-19 | The paced controller's `head_for` takes the doorway whose `to` is the agenda place, else a seeded door. It is stateless (`ARC-27`). | `cognition/rule-controller/src/paced.rs:244–262` | With 12n it asks `walk-to`. For another region it must ask `travel-to` and then follow the disclosed itinerary. A stateless controller stays stateless (§5.6). |
| A-20 | `economy` is the only mover of money. `MoneyTransferred` and `pay()` are `pub(crate)`; only `admit_payment` and `balance` are public. `buy { item }` works only in a Place carrying a `Shop` (one shop per place) and emits `money-transferred` plus inventory's `items-transferred`. A cross-pack charge follows the `WageDue` pattern: economy subscribes to the requester's fact and answers. `inventory` is the only writer of holdings; `consumption` states inventory's fact and owns nothing. | `systems/economy/src/{money.rs:12–63, event.rs:85–119, system.rs:59–226}`; `DECISIONS.md` `ARC-28`, `ARC-37`, `ARC-38` | **A ticket is an item kind bought at a station shop with the existing `buy`: no economy change.** Charging money *at boarding* would need economy to subscribe to a transport fact (an economy edit), so it is not the design (§5.7). A taxi fare is the one place money moves at a ride: it uses the `WageDue` shape (§5.4). |
| A-20b | `schedule`: one uninterruptible `RoutineProcess` per person; segments `{from, place, label}`, 2–24 per day; every day the same; `TimeOfDay = at mod 86 400`; shifts may not cross midnight. | `systems/schedule/src/{component.rs:13–39, process.rs:11–16, segment.rs:11–22, time.rs:9–39}`; `step-10-market.md` SD-23 | A segment's place may be in another region; nothing else changes. A commuter's departure is content (an earlier segment), as S19's FX-24 remedy already says. |
| A-21 | QPL-11: no runtime entity creation by a System, because it needs a kernel change. | `mvp0/overall.md` "Physics list" item 4 | Vehicles are a **fixed, authored fleet**. Transport never spawns a carriage (§5.4). |

## 2.6 Time

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-22 | `ARC-67`: calendar time is `WorldTime`; embodied time is not a world quantity; time scale and pause are host pacing, recorded in the host journal, never read by a pack. A `clock { at, time_scale, paused }` frame tells clients. | `CORE_CONCEPTS.md` §17; `step-19-time-weather.md` §§4, 7.4 | A trip is calendar time (a Process). The client converts the remaining calendar time to wall time from the `clock` frame. Pause freezes the scenery. No pack reads the scale. |
| A-23 | 12n revision 1 withdrew a walk Process precisely because a calendar-time walk runs ×12 on screen. | step-11 §21.2 F-N6 | Walking inside the carriage stays embodied (`move`, `walk-to`/`walk-step`). Only the *vehicle* moves in calendar time, and nothing on screen is "walked" by it. |

## 2.7 Network, seats and perception

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-24 | Perception is strictly the observer's current place (`present_with`). S11-C's audience (presence, `audience.rs`): `Public` always; `Place(p)` iff the observer's whereabouts after the fact is `p`; participants and named entities otherwise. | `systems/presence/src/observe.rs:71–108`; `step-12-server.md` SD-C1 (PR #95) | Region boundaries are already perception boundaries, because a place belongs to exactly one region. Riders in one carriage perceive one another and nobody on the platform they are passing. No audience change. |
| A-25 | S11-B: seats are Persons a client may connect as; `Free`, `Hosted`, `Connected`, `Held`. Control is host state. | `step-12-server.md` §4.2 | A seat's region is wherever its Person is. Two players in different regions are two observers of one world. No seat change. |
| A-26 | Protocol revision 2: closed client vocabulary (`join`, `submit`, `leave`); no subscribe frame; unknown fields are `malformed_frame`; "specified whole, landed incrementally" (`ARC-41`). | `server/PROTOCOL.md` §§2, 5, 9 | A map is not a client request frame. It arrives as disclosure in observations (§5.8), so the closed vocabulary stays closed. |

## 2.8 Measured cost: is "simulate every region all the time" cheap?

Measured numbers in the plans (one Apple-silicon machine):

| Measurement | Value | Source |
| --- | --- | --- |
| 300 days, no bodies | 13–18.6 s CPU per town | step-11 §19 (line 6454); 12n F-N12 |
| 300 days, with bodies | 45–49 s CPU (×3.0 market-town, ×3.6 social-cafe); 12d WIP up to ≈ 105 s under load | step-11 §20 E-Z7; 12n F-N12 |
| 300-day log | 365 330 (social-cafe) / 374 857 (market-town with calendar) facts | step-19 lines 1046–1047, 1248 |
| Hosted tick | p50 0.3 ms, p99 2.3 ms, max 5 ms (bound p99 ≤ 50 ms) | step-12 CP-B4, line 2449 |

```text
per region per world day     ≈ 0.05 s CPU without bodies, ≈ 0.16 s with bodies (worst seen 0.35 s)
one world day at 12×         = 7 200 wall s (3 600 at 24×)
CPU share per region         ≈ 0.16 / 7 200 ≈ 0.002 % of one core (worst 0.01 % at 24×)
20 regions                   ≈ 0.05–0.2 % of one core; ≈ 25 000 facts per world day
```

These are headless `run` numbers at the notional pace. A hosted world adds embodied requests, which
scale with *connected players*, not with regions. Conclusion: simulating every region all the time
is cheap by two to three orders of magnitude; the binding cost is client presentation memory (§5.12).

**Two scaling caveats found by the audit** (never measured beyond 12 people in 6 places):
`present_with` scans every `Presence` per observation, O(world population)
(`systems/presence/src/observe.rs:100–108`), and S11-C's audience fold is one `BTreeMap` over
everyone. With 20 regions × 12 people that is 240 entries per observation — still small, but
quadratic in total population across all observers. RT-X1 (RT-d's checkpoint) measures a 20-region
fixture; if the hosted tick p99 exceeds 10 ms, presence gains a place index (N-13).

---

# 3. Research (read 2026-10-09)

All sources below were read on 2026-10-09 by this session's research agent. Secondary or
community sources are labelled; claims that could not be confirmed are listed at the end of the
section and are not used by the design.

## 3.1 Reference games: connecting zones with transport

| Game | How zones connect | What feels good or bad | Lesson for S21 | Source |
| --- | --- | --- | --- | --- |
| Animal Crossing: New Horizons | Dodo Airlines flights; every arrival or departure plays an unskippable plane cut-scene for **every** player on the island, and others cannot join until it ends. | Widely criticized as disguised, blocking loading (in a party of 8 it is watched 7 times). | A transition must be **local to the traveller** and never block other players (INV-RT-11). | [Kotaku](https://kotaku.com/nintendo-has-made-animal-crossing-visits-as-annoying-as-1842886629), [Vice](https://vice.com/en/article/4ag84j/animal-crossing-has-an-unskippable-cutscene-that-is-making-me-hate-my-visitors), [TheGamer](https://www.thegamer.com/animal-crossing-multiplayer-problems/) |
| Pokémon Gold/Silver/HGSS | Walkable routes inside a region; the Magnet Train links Johto and Kanto and needs a Pass; Fly goes only to towns already visited. | Routes give distance; the train gives a sense of *region*; Fly's "visited only" rule is the canonical fast-travel gate. | Two scales: walk within a region (12n), ride between regions (S21). | [Bulbapedia](https://bulbapedia.bulbagarden.net/wiki/Magnet_Train); [Fast travel](https://en.wikipedia.org/wiki/Fast_travel) (general) |
| Stardew Valley | A bus to Calico Desert: repaired first, 500g return ticket, Pam drives 10:00–17:00. | Price, schedule and gating are diegetic and make the desert feel far. | Timetables and tickets make distance felt at almost no cost (§5.5, §5.7). | [Stardew wiki](https://stardewvalleywiki.com/Bus) |
| Red Dead Redemption 2 | Train tickets from a clerk, $5–15 by distance, only to stations already visited; some routes need transfers. | Fast, but gated by place and money. | Fare by distance; itineraries with transfers (§5.6). | [Red Bull](https://redbull.com/ca-en/red-dead-redemption-2-guide-how-to-fast-travel), [GTABase](https://gtabase.com/red-dead-redemption-2/vehicles/train) |
| The Last Express (1997) | The whole game is a walkable train on a clock at ≈ 6× real time; NPCs move between cars whether or not the player is there. | Proves a train interior can carry a whole story; criticized as confined. | Ruling 2's walkable carriage is a proven form; keep rides short and the carriage populated. | [Wikipedia](https://en.wikipedia.org/wiki/The_Last_Express), [Automaton](https://automaton-media.com/en/reviews/20231208-23849/) |
| Euro Truck Simulator 2 | Continuous roads at ≈ 1:19 map scale (community consensus from SCS posts) and in-game time compression; an atlas map. | Compression keeps drives playable in an evening. | Authors may compress distance; state the scale (QRT-6). | [racinggames.gg](https://racinggames.gg/article/euro-truck-simulator-2-is-roughly-119-and-thats-why-it-works), [fandom](https://truck-simulator.fandom.com/wiki/Time_Compression) (community) |
| Mini Metro | Abstract map "in the classic abstract subway style of Harry Beck". | Readable at a glance. | The world map is a transit diagram (§5.8). | [Dinosaur Polo Club press kit](https://dinopoloclub.com/?p=29) |
| Persona 5 | The loading screen changes with travel mode: subway trips show commuter silhouettes. | Commuting reads as daily life. | The ride is a place where life continues (ruling 2). | [Twinfinite](https://twinfinite.net/2016/06/e3-2016-persona-5-details/) (pre-release) |
| Final Fantasy XIV | Discrete zones; teleport shows a plain black screen, which players asked to theme. | Predictable but immersion-breaking. | A carriage is a better loading screen than a black screen. | [consolegameswiki](https://ffxiv.consolegameswiki.com/wiki/Travelling), [forum](https://forum.square-enix.com/ffxiv/threads/338165-Suggestion-Have-an-Aetherial-Loading-Screen-when-Teleporting.?p=4323315) |
| World of Warcraft | Boats and zeppelins between continents; the crossing hides the continent load. Bug reports describe boats "sailing off into the loading screen". | The ride hides most of the load. | Ride + background load + a hold if loading is not done. | [GameBanshee](https://gamebanshee.com/kwywh), [Blizzard forum](https://us.forums.blizzard.com/en/wow/t/zeppelins-bug/1213627) (old, secondary) |
| Elevators that mask loading (Mass Effect et al.) | Fixed-length rides hiding loads. | Fails when the ride ends before loading (stutter after arrival) or when its length varies with load time. | **End the ride on load-complete, or loop a segment until then** (§5.9). | [Giant Bomb](https://giantbomb.com/wiki/Concepts/Elevators_That_Mask_Loading_Times), [Wayline](https://www.wayline.io/blog/transforming-loading-screens-immersive-experiences) |
| Breath of the Wild, GTA V | Seamless streaming open worlds; Rockstar calls fitting the streamed world into console memory the big achievement. | Seamless, but needs engine-level streaming designed in from the start. | Rejected for MineWorld: the operator rules it out, and Godot's texture streaming is only in a 4.8 dev snapshot (§3.2). | [BotW](https://en.wikipedia.org/wiki/The_Legend_of_Zelda:_Breath_of_the_Wild), [TechRadar](https://www.techradar.com/news/gaming/the-tech-that-built-an-empire-how-rockstar-created-the-world-of-gta-5-1181281) |
| The Sims 4 | Loading screens between lots; players report 5–10 minute loads. | Long loads with no diegetic cover are the worst case. | Budget per region bundle (§5.12). | [EA forum](https://forums.ea.com/discussions/the-sims-4-technical-issues-pc-en/long-loading-times/11739472) (anecdotal) |
| Final Fantasy VII Remake | The opening train ride runs seamlessly into action on arrival. | Arrival as a continuous moment. | Arrival is a dwell in the carriage, then `alight`, not a cut. | [Tom's Guide](https://www.tomsguide.com/uk/us/final-fantasy-vii-remake-e3,news-30355.html) |

**Lessons the design adopts:** (1) mask loading with diegetic travel, but let the ride end on
load-complete, not on a fixed timer; (2) never block other players; (3) make repeated rides
fast-forwardable where the host allows (QRT-7); (4) gate network access diegetically (tickets);
(5) compress distance with a stated scale; (6) "visited only" is the established fast-travel
gate (QRT-11).

**Not confirmed, not used:** the Hogwarts Legacy train intro, Night in the Woods, Mother 3,
Unrailed, Train Sim World specifics, and FFXIV's current airship/ferry presentation. A Short Hike
was not researched.

## 3.2 Godot 4.7 asset streaming (`DEP-4`: Godot 4.7.2)

- Stable documentation is Godot 4.7; 4.8 is in development (dev 5 published 2026-09-10).
- `load_threaded_request(path, type_hint = "", use_sub_threads = false, cache_mode = CACHE_MODE_REUSE)
  -> Error`; `load_threaded_get_status(path, progress)` fills `progress[0]` with 0.0–1.0 and is to be
  polled across frames; `load_threaded_get(path)` **blocks like `load()`** if called before
  `THREAD_LOAD_LOADED`. Status values `INVALID_RESOURCE`, `IN_PROGRESS`, `FAILED`, `LOADED`.
  ([class reference](https://docs.godotengine.org/en/stable/classes/class_resourceloader.html);
  [Background loading](https://docs.godotengine.org/en/stable/tutorials/io/background_loading.html))
- Practical caveats: instancing and `add_child` still run on the main thread and are the usual
  source of stutter, so spread them over frames and pre-warm shaders
  ([itch.io devlog](https://wiebow.itch.io/scramble-rnd-revisited/devlog/222308/update-3-technical-fixes-and-next-area));
  `use_sub_threads` has been linked to intermittent crashes
  ([godot#84012](https://github.com/godotengine/godot/issues/84012)), so the design keeps it `false`.
- **Texture streaming:** none in 4.7. **Godot 4.8 dev 5 adds mip-level texture streaming** (GH-113429,
  a `TextureStreaming` singleton and a "Texture2D Streamed" import type), pre-release
  ([dev snapshot](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-5/)). Mesh streaming is a
  proposal only ([proposals#6109](https://github.com/godotengine/godot-proposals/issues/6109)). The
  design does not depend on either; if 4.8 ships it, it lowers the per-region texture cost inside
  the same model (QRT-14).
- **Memory budget instruments:** `Performance` monitors `RENDER_VIDEO_MEM_USED`,
  `RENDER_TEXTURE_MEM_USED`, `RENDER_BUFFER_MEM_USED`, `OBJECT_RESOURCE_COUNT`; `MEMORY_STATIC` is
  **not available in release builds** and some monitors return 0 in release exports;
  `add_custom_monitor` is available
  ([Performance](https://docs.godotengine.org/en/stable/classes/class_performance.html)). RT-g measures
  with the render-memory monitors in a debug export.
- **Audit:** no client uses threaded loading today. The 2D client reads its Presentation Pack at
  runtime from disk with `FileAccess` and `Image.load_*`, lazily, into a cache
  (`clients/2d/scripts/presentation.gd:30–140`). The 3D spike uses synchronous `load()` of imported
  `res://` assets and has no runtime Presentation Pack loader yet (16f).

**Model chosen (§6):** region bundle = a Godot scene (3D) or a set of runtime-read images (2D) per
region, requested with `load_threaded_request` (3D) or a worker-thread read (2D) when a ride
towards that region departs; released when the player's region changes and a grace period
passes.

## 3.3 Window scenery

| Option | What | For | Against | Verdict |
| --- | --- | --- | --- | --- |
| A. Parallax layers | 2D layers scrolling at different speeds. Godot 4.3+ `Parallax2D` has `autoscroll` (px/s with no camera motion — exactly a window view), `repeat_size` for infinite loops, `scroll_scale` ([class](https://docs.godotengine.org/en/stable/classes/class_parallax2d.html), [tutorial](https://docs.godotengine.org/en/stable/tutorials/2d/2d_parallax.html)). In 3D: textured quads at depths outside the windows. | Cheap, art-directable, works in both clients, any length by looping. | Flat; needs authored layers per landscape. | **Chosen for 2D and as the 3D baseline.** |
| B. Tagged tile library | Short 3D or 2D segments ("coast", "fields", "tunnel", "station-approach") selected by tags and chained along the run. | Variety from a small library; tags come from the world; authors add tiles. | Seams between tiles need a convention (fade or tunnel). | **Chosen as the sequencing model** over A's layers. |
| C. Shader skybox/panorama | A scrolling panorama shader on a sky dome; Sky3D (`DEP-8` approved, MIT) for sun and weather. | Sky consistent with the world's clock and weather. | Distance only; no foreground. | **Chosen for the sky layer** in 3D. |
| D. Pre-rendered video | A video per segment. | Highest fidelity. | Large, not tag-composable, not time-of-day aware. | Rejected. |
| E. Real streaming of the line's geometry | Render the actual terrain between regions. | True continuity. | Exactly what the operator ruled out. | Rejected. |

Assets under `DEP-8`'s relicensing test:

| Source | Licence (read 2026-10-09) | Verdict |
| --- | --- | --- |
| [Kenney Train Kit](https://kenney.nl/assets/train-kit) (≈ 50 low-poly trains, spline track pieces; v1.1) | CC0 ([Kenney support](https://kenney.nl/support)) | Admissible as placeholder (DEP-8: Kenney is blockout only) |
| [Poly Haven](https://polyhaven.com/license) HDRIs, textures, models | CC0 | Admissible: sky panoramas and distant layers |
| [ambientCG](https://docs.ambientcg.com/license/) | CC0 1.0 | Admissible: materials |
| [Sky3D](https://github.com/TokisanGames/Sky3D) v2.1, Godot 4.3+ | MIT | Already approved by DEP-8: the sky layer |
| [Quaternius](https://quaternius.com/license.html) (Modular Train Pack, Public Transport Pack, Cars Pack, …) | **Quaternius Asset License v1.0: no redistribution of the assets themselves** | **Excluded** from the repository (fails the relicensing test); older packs marked CC0 on OpenGameArt are admissible per pack |
| OpenGameArt parallax backgrounds | per asset | CC0 admissible after per-asset check; CC-BY-SA and GPL excluded |
| Generated layers (paid Meshy, OpenAI images) | per DEP-8 | Admissible with `ARC-9` provenance |

## 3.4 Maps

| Option | For | Against | Verdict |
| --- | --- | --- | --- |
| A. Server-served, generated from region geometry: places' passages and `PlaceShape` footprints composed by translation (the `ARC-45` rule) into a region layout; world map from authored region placement and lines | One source of truth; matches "One world, two views"; works for authors without drawing anything | Needs an authored placement for places not connected by passages (rare) | **Chosen** |
| B. An authored map image per region in the Presentation Pack | Beautiful | A second source of layout that can drift from the world; violates "the server is the single source of layout" | Rejected as source; allowed as *decoration* keyed by region |
| C. A client renders a minimap with a `SubViewport` and an orthographic camera over its loaded scene ([SubViewport](https://docs.godotengine.org/en/4.1/classes/class_subviewport.html)) | Cheap in Godot | Only the loaded region; 2D and 3D would differ; no world map | Rejected as source; fine as a 3D presentation of A |
| D. Godot addons: NAP Map Generator (MIT, scans a 3D scene into a top-down image), DynamicMinimap (MIT, Godot 4.6), Mini Map (MIT, 2D) ([asset library](https://godotengine.org/asset-library/asset/edit/16477), [21840](https://godotengine.org/asset-library/asset/edit/21840), [21808](https://godotengine.org/asset-library/asset/edit/21808)) | MIT; ready-made | Each takes the client's scene tree as the source of layout — a second authority, and a different one per client | Not adopted as source; DynamicMinimap may be evaluated in RT-g as a *renderer* of atlas records |
| E. Transit-diagram layout: Nöllenburg–Wolff octilinear MIP ([paper](https://i11www.iti.kit.edu/extra/publications/nw-dlhqm-10.pdf)); LOOM (Freiburg, reportedly GPL-3.0, [paper](https://ad-publications.cs.uni-freiburg.de/Large-Scale_Generation_of_Transit_Maps_from_OpenStreetMap_Data.pdf)) | Optimal, proven | Built for hundreds of stations; MIP solvers or GPL code | Rejected: a world has a handful of lines; authored region positions plus octilinear snapping in the client is enough (this is a judgement, not a sourced claim) |

---

# 4. Terms

New terms proposed (`CLAUDE.md` §2.1 rule 3: they are defined once and are not synonyms of existing
ones):

```text
region            a Place tagged `region` at the top of a `contains` hierarchy of Places. Not a
                  new primitive. Every Place of a region-composed world lies in exactly one region.
line              transport's: an ordered list of stops with segment lengths and a mode
stop              a Place a line serves (a platform, a bus stop, a car park)
vehicle           an authored Item (or Organization-owned Item) whose interior is a Place; the
                  "TransportComponent" of CORE_CONCEPTS §1 is transport's `Vehicle` component
run               one journey of a vehicle along a line from one stop to the next: a Process
itinerary         a traveller's plan of legs (walk, ride), owned by transport, disclosed to them
scenery sequence  presentation only: what a client shows outside the windows during a run
region bundle     presentation only: the assets a client loads for one region
```

---

# 5. Design

## 5.1 Ontology: is "region" a new concept? (`ARC-RT-a`)

| Option | What | Verdict |
| --- | --- | --- |
| O-1. New core primitive `Region` | `CORE_CONCEPTS.md` §1 gains a concept | **Rejected.** §6 already says Places form an optional hierarchy and gives a town as a Place. A primitive would duplicate Place. |
| **O-2. A region is a Place at the top of a `contains` hierarchy, tagged `region`** | §6 gains a short subsection defining the term; the relation `contains` (Place ↔ Place) is declared by an owning System Pack | **Chosen.** No kernel change: relations and tags exist. |
| O-3. A region is only a World Pack directory, invisible at runtime | Loader-only | Rejected: clients must know which region they are in (asset loading), and the map must name regions. |

**Who owns `contains`?** The `atlas` pack (§5.8). It is seeded at genesis from the composition
(each place in a region's directory is `contains`-ed by the region's place) and never changes in
MVP-1. Presence, movement and transport do not need it: perception is per place (A-24), and
transport works on stops.

**Is a region place enterable?** No. It is an organizational Place with no `Presence` in it. A
region's people stand in its sub-places.

Proposed `CORE_CONCEPTS.md` amendment (applied by the primary session as a reviewed change): add
"§6.4 Region" with the definition above and the rule "a region is the unit in which a client loads
presentation and in which a map is drawn; it is not a unit of simulation, of perception or of time".

## 5.2 Composition: region packs into one world (`ARC-RT-b`)

```yaml
# worlds/two-towns/world.yaml
world: { id: two-towns, name: Two Towns, version: 0.1.0, license: MIT }
systems: [presence, movement, bodies, conversation, …, transport, atlas, fares]
regions:                     # new; each key is a region
  harbour:  { path: regions/harbour }                 # inside this World Pack
  uptown:   { pack: market-town, version: "^0.1" }    # a World Pack used as a region (QRT-2)
lines: configure/transport.yaml                       # lines are transport's configuration
```

```yaml
# regions/harbour/region.yaml (or a world used as a region: read from its world.yaml + atlas section)
region: { name: Harbour }
atlas:                        # atlas's section
  at: { x: 0, y: 0 }          # world-map placement, metres
  landscape: [coast, harbour, hills]
```

Rules:

1. **Namespacing.** In a region-composed world, every key of a region's content is qualified as
   `<region>-<key>` at load time (`harbour-cafe`), because `EntityKey` forbids `.` (A-4). The
   *authored* files keep their unqualified keys, so a town authored as a standalone world composes
   unchanged. References inside a region (passages, routines, locations) resolve within that region
   first. A reference to another region is written qualified (`uptown-workplace`).
2. **Determinism.** Ids are allocated region by region in key order, then within each region by the
   existing order (places → people → items → organizations). One region-free world loads
   byte-identically to today (INV-RT-1).
3. **Systems** are the composing world's `systems:` list. A region's own `systems:` must be a subset,
   or the world is refused by name.
4. **Configuration** (`configure/`) is the composing world's. A region's `configure/` files are
   ignored with a load-time note when the world provides its own, and refused when two regions'
   files disagree and the world provides none (QRT-3).
5. **Seats** are the union of the regions' seats, qualified.
6. **A World Pack used as a region** (QRT-2): `packages/src/resolve.rs:217` refuses a World Pack
   in `requires:` today. The recommendation is a narrow relaxation: a World Pack may be named under
   `regions:` (never under `requires:`), is resolved by the same rules as `ARC-54`, and contributes
   content only. It cannot itself have `regions:` (no nesting in MVP-1).

Owner: `worldpack` (loader) and `packages` (resolution). Not the kernel.

## 5.3 The `transport` System Pack (`ARC-RT-c`)

**Depends on:** `presence` (states `arrived`, `ARC-26`), `movement` (reads `Passages` for itineraries
and asks `walk-to` legs through the 12n catalog). **Optional reader of** `schedule`: none, because
controllers follow agendas.

**Owns:**

```text
Line        (configuration, not a component): id, mode (rail | road), stops [StopId…],
            segments [{ length_m, landscape: [tag…] }], service: Timetable | OnDemand
Vehicle     on the vehicle Item: line, interior PlaceId, doors [{ here: LocalPosition }],
            state: Docked { stop, until } | Running { from, to, run: ProcessId }
StopInfo    on a stop Place: lines served, the platform's door anchors
Itinerary   on a traveller Person: legs [Walk { to } | Ride { line, from, to }], current leg
```

**Facts (owned):** `vehicle-departed { vehicle, from, to, arrives_at }`,
`vehicle-arrived { vehicle, stop }`, `boarded { person, vehicle }`, `alighted { person, vehicle,
stop }`, `itinerary-set { person, destination }`, `itinerary-ended { person, outcome }`. Visibility:
departures and arrivals are `Place(stop)` and `Place(interior)`; boarding is `Place(interior)`.

**Configuration** (`configure/transport.yaml`, `ARC-61`): lines, timetables, dwell times, default
speeds. All integers (seconds, metres, metres per hour).

## 5.4 Vehicles, runs, boarding (`ARC-RT-d`)

- **The fleet is authored** (A-21): each vehicle is an Item file with a `transport:` section naming
  its line and interior place, and the interior is an ordinary Place file with a `body:` section
  (a carriage ≈ 20 × 2.8 m, seats as solids) and **no passages**.
- **A run is a Process** owned by transport: started at departure, `expected_end = departed +
  segment_length / speed + 0`; woken at its end; the wake states `vehicle-arrived` and opens the
  dwell. Calendar time throughout (A-22).
- **`board`** (actor, target vehicle): `SpatialRequirement { place: SamePlaceAsActor-of-stop,
  within_range: 3 000 mm of a door anchor }`; refused `Unavailable` with transport's code
  `not-docked` when the vehicle is running, `PermissionDenied` when a `BoardingCheck` catalog entry
  refuses (fares, §5.7). Resolve: states presence's `arrived` into the interior at the door's
  `here` (bodies resolves the exact spot, A-8), plus `boarded`.
- **`alight`**: actor in the interior, vehicle `Docked`; states `arrived` at the stop's platform
  anchor, plus `alighted`.
- **Inside the carriage** everything is ordinary: `move`, 12n's `walk-to`/`walk-step`, `talk`,
  `give`, `buy` from a trolley shop. Nothing in those packs knows it is moving (INV-RT-4).
- **A person still aboard at the terminus** stays aboard and rides back. An NPC whose itinerary says
  "alight here" alights at the first dwell consult.
- **Cars (operator ruling 4).** Three kinds, one model:

  | Kind | Service | Between regions? | In S21's split? |
  | --- | --- | --- | --- |
  | **Bus** | a road line with a timetable, exactly a train's model with `mode: road` | yes (and within a region) | yes, RT-c |
  | **Taxi** | `OnDemand`: a fleet of taxis docked at taxi stands (stops); a passenger aboard asks `hire { to: stop }`; the run starts at once and follows the road graph; the driver is a Person with a hosted or rule controller, or no driver at all in a world that does not model one | yes | yes, RT-c |
  | **Self-driven car** | a car the driver owns; `drive-to { stop }` along road links **of the current region only** | **no** by default | **no**: design-only placeholder (§5.4.1) |

  A taxi fare is the one ride that charges money at the ride. It uses the `WageDue` shape: `fares`
  states its own `fare-due { passenger, operator, amount }` at the end of the run, and economy — which
  already answers `wage-due` — answers it with `money-transferred` or `fare-unpaid`. That is one
  subscription added to economy, the same kind of edit `wage-due` was (`ARC-28`), and it lands in
  RT-c with economy's owner reviewing it (QRT-10). Bus and train fares stay tickets (§5.7).

### 5.4.1 Self-driving and road trips (design only; no PR in this split)

- **Road links** are transport configuration: `{ from: stop, to: stop, length_m, landscape, self_drive:
  forbidden | allowed }`. A link inside one region defaults to `allowed`; a link that crosses regions
  defaults to `forbidden` (ruling 4).
- `drive-to` (future) is offered only to the person in a car's driver seat, and validated by transport
  against the links: every link of the route must be `allowed`, else `PermissionDenied` with code
  `self-drive-forbidden`. A **road trip** mode is then only world configuration setting
  `self_drive: allowed` on chosen inter-region links — no new code, no new pack (INV-RT-12).
- Whether the driver steers (embodied driving inside a region) is a later design that must answer
  rule 14 again: a car's in-region motion would be embodied requests like `walk-step`, not a
  calendar-time run. S21 records the question and builds nothing.

## 5.5 Timetables and defaults with real-world sources (`DEP-RT-a` for any data)

Sources read 2026-10-09:

| Quantity | Default | Basis |
| --- | --- | --- |
| Regional train commercial speed (including stops) | 60 km/h | Poland 2024: all trains 55.2 km/h, PKP Intercity 80.4 km/h, urban rail 36.1 km/h ([polishtrains.eu](https://www.polishtrains.eu/railways/at-what-speed-does-the-intercity-train-travel)) |
| Intercity / high-speed (optional line classes) | 80 / 220 km/h | PKP Intercity as above; Nozomi Tokyo–Shin-Osaka 515 km in 2 h 21 m ≈ 219 km/h ([JR Central](https://global.jr-central.co.jp/en/company/ir/annualreport/_pdf/annualreport2025-13.pdf)) |
| Dwell at a stop | 60 world s | A design default; no source sought (a world-configurable value) |
| Headway | 20 world min | Munich S-Bahn every 20 min, 10 at peaks ([Wikipedia](https://en.wikipedia.org/wiki/Munich_S-Bahn)) |
| Bus speed inside a region / taxi speed between regions | 25 km/h / 80 km/h | EU limits: urban 50 km/h, rural mostly 80–90 km/h ([EU road safety](https://road-safety.transport.ec.europa.eu/eu-road-safety-policy/priorities/safe-road-use/safe-speed/archive/speeding/speed-limits_en)); bus average below the limit because of stops (a judgement) |
| Default inter-region distance | 21 km | Christaller's k = 3 spacing 7 / 12 / 21 / 36 km, theoretical ([arXiv](https://arxiv.org/pdf/1105.0589)) |
| Resulting ride | 21 km at 60 km/h = **21 world min** | Close to the mean one-way commute: EU 25 min (Eurostat 2019, [news](https://ec.europa.eu/eurostat/de/web/products-eurostat-news/-/ddn-20201021-2)); US 26.8 min (ACS 2023 via [CommercialCafe](https://www.commercialcafe.com/blog/?p=39951)) |
| Wall time of that ride | 105 s at 12×, 53 s at 24×, 210 s at 6× | `ARC-67` |
| Fare | 3 % of a day's wage per ride (demo content) | Relative scale; Deutschlandticket €63/month from 2026-01-01 ([DB](https://int.bahn.de/en/faq/deutschlandticket-cost-new)); RDR2 $5–15 by distance |

The operator's example — a 40-world-minute trip at 12× takes about 3.3 wall minutes — is a 40 km
regional line. Authors may compress lengths (QRT-6). All values are transport configuration
(`ARC-61`), integers in metres, metres per hour and seconds.

## 5.6 Two scales of one spatial model: `travel-to` and the itinerary (`ARC-RT-e`)

```text
walk-to X   (12n)     X in a passage-connected set of places; a Walking component; strides are
                      embodied walk-step requests; no calendar time
travel-to Y (S21)     Y anywhere; transport plans an Itinerary over the line graph + passages:
                      [Walk to stop A] [Ride line L from A to B] [Walk to Y]
                      Walk legs are 12n walks the traveller asks for; ride legs wait for a docked
                      vehicle, `board`, ride (calendar time), `alight`
```

- The itinerary planner is transport's, deterministic: Dijkstra over stops (edge cost = scheduled
  ride time + expected wait at the departure instant), with places mapped to their nearest stop via
  movement's passage graph (breadth-first, as 12n). `pathfinding` (12n's `DEP-P`) is reused.
- `travel-to` is offered incomplete, once, against nobody, like `walk-to` (12n SD-N7).
- The itinerary is disclosed to the traveller only: `itinerary { next: Leg, legs_left }`. A stateless
  controller follows it: if the next leg is Walk, ask `walk-to`; if Ride and a vehicle of that line
  is docked here, `board`; if aboard and docked at the leg's end, `alight`. That is a pure function of
  the observation (`ARC-27` holds).
- **Clients compose nothing:** a map click on a place in another region is `travel-to`; the itinerary
  is drawn from disclosure.

## 5.7 Tickets and fares (`ARC-RT-f`)

- **Tickets are inventory item kinds** (`ticket-<line>` or a zone ticket), sold at a station shop with
  economy's existing `buy`. Economy and inventory are unchanged.
- **`fares` is a separate optional pack** (depends on transport and inventory). It implements a
  transport-owned catalog `BoardingCheck` (`ARC-62`; pure, inert where it has no state): "does this
  person hold a valid ticket for this line?" → refusal `PermissionDenied` with code `no-ticket`. It
  then reacts to `boarded` by stating inventory's consumption fact (the `consumption` pattern).
- A world without `fares` rides free. A world without `economy` can still hand out tickets as items.
- An honest limitation: the check and the consumption are two steps at one instant; nothing can
  intervene between them in a single-threaded world (`DEP-6`), and RT-c's adversarial test proves it.

## 5.8 Maps: the `atlas` System Pack (`ARC-RT-g`)

- **Owns** `contains` (region → place), `RegionInfo { name, at, landscape }` on region places, and
  computes two records from authored, genesis-only state:
  - **region map**: places of the observer's region, each with its footprint (bodies' `PlaceShape`
    floor, when bodies is enabled) and its offset in the region frame (composed by translation along
    passages, the `ARC-45` rule, now done once on the server); passages; stops.
  - **world map**: regions with `at`, and lines with stops and segment lengths (read from transport
    through an atlas-owned `MapLayer` catalog that transport implements, so atlas never names
    transport).
- **Disclosure:** to every observer, `region-map` for the region of their current place and
  `world-map` once. Both carry a content hash; a delta repeats nothing that did not change (S11-C
  deltas). This is a disclosure about the observer's place, the S19 pattern (`step-19` §5.5).
- **Is a map omniscience?** No: it is authored geography, not state. No person, no item and no
  dynamic fact is on it (INV-RT-6). A world that wants hidden places tags them `unmapped`.
- **Places not connected by passages** inside a region need an authored offset (`atlas: { at }` on the
  place file); otherwise they are drawn in an "elsewhere" inset, never at an invented position.
- **Clients** draw both maps from the records: 2D as an overlay; 3D as a full-screen panel. The world
  map is a transit diagram (§3.1, Mini Metro). Neither computes a route: a click is `travel-to`.

## 5.9 Scenery sequences (presentation; `ARC-RT-h`)

- **Inputs, all disclosed:** the run's `from`, `to`, `departed_at`, `arrives_at`, the segment's
  `landscape` tags and the two regions' `landscape` tags (`vehicle` disclosure to riders), the
  `clock` frame (scale, pause) and calendar/weather records (time of day, rain).
- **Selection** is a Presentation Pack rule: `scenery/library.yaml` lists tiles, each with tags,
  a nominal wall length and a loop flag. The client picks tiles whose tags intersect the segment's,
  starting with the origin region's tags and ending with the destination's, seeded by the run's
  `ProcessId` so every client shows the same sequence for the same run (both clients, multiplayer).
- **Length:** `remaining_wall = (arrives_at − now) / time_scale`, re-computed on every `clock` frame;
  paused → frozen. Tiles are stretched by looping; never by changing speed in a way that implies a
  different world speed.
- **It is the loading screen, and it ends on load-complete** (§3.1 lesson 1). The destination's region
  bundle is requested at departure (prefetch). On arrival, if loading is not done, the window loops
  the station-approach tile and the HUD says "arriving"; the person is still inside the carriage
  (they have not alighted), so nothing in the world waits and no other player is affected
  (INV-RT-11). The client may hold its own `alight` request until loaded; that holds a request, it
  evaluates no rule. If the dwell ends first, the person rides on and the itinerary re-plans — an
  honest consequence, measured in RT-g (P95 load time must be below the default dwell).
- Authors add tiles and tags without code. The default library ships CC0 or MIT assets (§3.3).

## 5.10 Rule 14, item by item

```text
MoveIntent          move / walk-to / walk-step inside the carriage and on platforms; board, alight,
                    travel-to — requests, refusable
Travel Process      the vehicle's run, owned by transport, in calendar time
Spatial state       presence (the person is in the interior place) + transport (the vehicle is
                    Running { from, to } or Docked { stop })
Rendered movement   the embodied walk inside the carriage + the scenery sequence outside it
```

## 5.11 Time domains

- Calendar: runs, dwells, timetables, itineraries' waits, shifts the commuter is late for.
- Embodied: walking and talking in the carriage, scenery frame rate, loading.
- Pause: no run advances, no request is accepted (`paused`, S19 §7.2), the scenery freezes.
- Scale change mid-run: the scenery's remaining wall length is recomputed from the `clock` frame; no
  pack is told.
- One calendar for the world. Per-region weather is S19's QTW-12 (one `climate` Process per region,
  additive). Per-region time zones are out of scope (QRT-8).

## 5.12 Client streaming model (`ARC-RT-i`)

```text
State per client      current region R (from the atlas record of self_location's place)
On join              load R's bundle (blocking, with a loading screen), the vehicle interior scene
                     if the observer is aboard
On departure         request bundle(to-region) in the background; keep R loaded
On alight            switch; R stays loaded for a grace period (60 wall s) then is released
Budget               at most two region bundles + one vehicle interior + the scenery library
```

- **Presentation Pack layout:** `regions/<region-key>/…` bundles keyed by region key, plus shared
  assets (characters, vehicle interiors, scenery). A pack with no region bundles falls back to its
  shared assets (today's behaviour).
- **2D:** the stitched town (`ARC-45`) is per region, from atlas's `region-map`, not re-derived from
  passages walked so far.
- **3D:** 16f builds places from disclosure; S21 adds threaded loading of the region's dressing.

## 5.13 Multiplayer

- Each client observes its own observer's place. Two players in different regions receive disjoint
  observations except `Public` facts (A-24). Each loads only its own region.
- Players in one carriage perceive each other and talk (ruling 2). They see the same scenery because
  selection is seeded by the run (§5.9).
- The host's scale and pause apply to everyone; a single player's "doze through the ride" is a host
  pacing control, available only to the admin-token holder (QRT-7).

## 5.14 Persistence

One save, one fact log. Vehicles' states, runs and itineraries are components and Processes like any
other; a restart mid-run resumes the run at its stored `expected_end`. No per-region partitioning
(QRT-9). Saves from a region-free world load unchanged.

---

# 6. Invariants (proposed)

| Id | Invariant |
| --- | --- |
| INV-RT-1 | **Byte identity without regions.** A world with no `regions:` and none of `transport`, `atlas`, `fares` produces byte-identical `run` output, saves and fact logs before and after every S21 PR. |
| INV-RT-2 | **Kernel and contracts unchanged.** No diff in `kernel/` or `contracts/`. A needed edit is a material stop. |
| INV-RT-3 | **Presence's audience rule unchanged.** Region boundaries are perception boundaries only because places are. |
| INV-RT-4 | **Packs do not know they are moving.** No pack other than transport, fares and atlas names a vehicle, a line or a region. |
| INV-RT-5 | **No rule reads pacing.** No pack reads the time scale or pause (`ARC-67`); the scenery's wall length is computed only in clients. |
| INV-RT-6 | **Maps carry geography only.** No person, item, or dynamic state is on a map record. |
| INV-RT-7 | **Clients compose no route.** A map click is `travel-to`; the itinerary is the server's. |
| INV-RT-8 | **One region at a time on screen.** A client never needs assets of a region its observer is not in or travelling to. |
| INV-RT-9 | **Every region runs.** No region is paused, frozen or simulated at a lower fidelity because no player is in it. |
| INV-RT-10 | **Fixed fleet.** No vehicle or interior is created at runtime (QPL-11). |
| INV-RT-11 | **A transition never blocks another player.** Loading, scenery and arrival holds are per client; nothing in the world or on another client waits for them. |
| INV-RT-12 | **No self-driving between regions unless configured.** A cross-region road link is `self_drive: forbidden` by default; a road trip is configuration only. |

---

# 7. Reuse summary (`REUSE_POLICY.md`; both directions)

| Need | Adopt | Rejected | Build |
| --- | --- | --- | --- |
| Itinerary search | `pathfinding` (already adopted by 12n, `DEP-P`) | a GTFS router (OpenTripPlanner, R5: JVM services, far beyond a handful of lines) | the line graph |
| Timetable format | GTFS's *concepts* (stops, trips, stop_times) as vocabulary only; nothing is copied from the specification text | GTFS files as the format: CSV of strings and floats, wrong for typed integer configuration | `configure/transport.yaml` |
| Background loading | Godot `ResourceLoader` threaded loading | an addon | none |
| Scenery | `Parallax2D`, Sky3D (`DEP-8`) | video | the tag selector |
| Map drawing | Godot `Control` / `Line2D` / `SubViewport` | minimap addons | the transit-diagram layout |

---

# 8. PR split (medium scope; each frozen one at a time)

| PR | Scope | Depends on | Integration checkpoint | Adversarial criteria |
| --- | --- | --- | --- | --- |
| **RT-0** | Docs only: `CORE_CONCEPTS.md` §6.4 Region; `MODULE_SPEC.md` §4 `regions:`; `ARC-RT-a`, `-b`; `PROTOCOL.md` records `region-map`, `world-map`, `vehicle`, `itinerary` (specified whole) | operator agreement on §11 | Doc checks pass; terms reviewed | A term used as a synonym of an existing one fails review |
| **RT-a** | `worldpack` + `packages`: `regions:`, namespacing, World Pack as region | MVP-0 S16 merged | `two-towns` (social-cafe + market-town, unedited) loads; `run` 300 d passes; ids deterministic on all three platforms | Renaming a key in one region changes no id in the other; a world without `regions:` is byte-identical (INV-RT-1); two regions with the same key both load |
| **RT-b** | `atlas` pack: `contains`, `RegionInfo`, region-map and world-map disclosure, `MapLayer` catalog | RT-a; S11-C | A headless observer in each region receives its region map and the world map; hashes stable across restart | Removing `atlas` removes only maps (AC-2 style); a map record never contains a person (mutation adds one → test fails) |
| **RT-c** | `transport` pack: lines (rail, bus), taxis (`hire`), vehicles, runs, `board`, `alight`, road links with `self_drive`; `fares` pack with `BoardingCheck` and taxi `fare-due`; economy's one subscription | RT-a; 12n merged | Headless: a seeded person boards a train at harbour, rides 21 km, alights at uptown; another hires a taxi back; facts in order; restart mid-run resumes | Board while running → `not-docked`; no ticket → `PermissionDenied/no-ticket`; a pack other than transport/fares naming a vehicle fails a scan (INV-RT-4); `move` across regions still `TooFarAway`; a cross-region link flipped to `allowed` in configuration changes nothing until `drive-to` exists (no hidden path) |
| **RT-d** | `travel-to`, itineraries; paced controller follows itineraries; `two-towns` with commuters living in one region and working in the other | RT-c; 12n-2 | 7 world days at seed 1: ≥ 90 % of cross-region shifts start with the employee present (FX-24's form); RT-X1 cost re-measured | Removing `transport` → commuters stay home, nothing panics; a controller holding state fails the stateless test |
| **RT-e** | Server and protocol: disclosure records land; `clock` frame consumed for runs; parity extension | RT-b, RT-c; S19 TW-c | Both clients' parity reports agree on region, vehicle state, itinerary | A client computing an itinerary fails `check_client_rules.py` (new rule) |
| **RT-f** | 2D client: region-scoped stitching, carriage scene, maps, scenery (parallax) | RT-e; S12 | Operator plays: walk to station, buy ticket, board, talk on board, ride, alight | Disconnect mid-ride and resume; pause mid-ride freezes scenery |
| **RT-g** | 3D client: threaded region bundles, carriage interior, scenery tiles + Sky3D, maps | RT-e; S14 16f | Operator plays the same path in 3D; memory ≤ budget measured | Arrival before load completes: person stays aboard, no world wait; scale change mid-ride re-times scenery |
| **RT-h** | Proof: `two-towns` default demo; coverage of every new interaction; multiplayer two players in two regions plus one carriage | all | Milestone acceptance list for the operator | Kernel/contracts diff gate across RT-a…RT-h is empty (INV-RT-2) |

RT-f and RT-g can run in parallel after RT-e. RT-b and RT-c can run in parallel after RT-a.

---

# 9. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-1 | Loading takes longer than short rides on slow machines | Prefetch at departure; arrival hold inside the carriage; minimum ride wall length as an author note, not a rule |
| R-2 | Namespacing breaks references that cross regions silently | Qualified-only cross-region references; loader refuses ambiguous keys by name |
| R-3 | Rides feel like waiting | Ride is a place with people (ruling 2); host "doze" control (QRT-7); authored compression |
| R-4 | Commuting NPCs miss shifts at 24× | FX-24-style criterion in RT-d; remedy is content (earlier departures), never a controller reading the scale |
| R-5 | Atlas becomes a God pack | Atlas owns geography only; lines come through `MapLayer`; no state beyond genesis |
| R-6 | Maps and the 2D stitched town disagree | 2D draws from atlas's region map only (one composition, on the server) |
| R-7 | World Pack as region conflicts with S16's package rules | QRT-2 decided before RT-a freezes |
| R-8 | Observation and audience cost grow with total population across regions (§2.8 caveats) | RT-X1 measures a 20-region fixture; a place index in presence if p99 > 10 ms (N-13) |
| R-9 | Some research is community-sourced (ETS2 scale, WoW, Persona) | The design depends on none of those numbers; they are cited as lessons only |

---

# 10. MVP-0 non-preclusion audit (recommendations to lanes)

| Id | Lane / PR | Would it block S21? | Recommendation |
| --- | --- | --- | --- |
| N-1 | **12n** (PR #100) | No. `walk-to` BFS over passages stays within a region because no passage crosses regions. | Keep the `Destination` enum open for a later arm; keep `walk-to` to an unreachable place refusing with a stable code (`TooFarAway` is acceptable; a pack code `no-route` is better for a controller to fall back to `travel-to`). Keep the `Walking` component free of any assumption that the place it is in stays still. |
| N-2 | **S11-C** (PR #95) | No. The audience is per place. | Ensure an observer changing place to one with no passages (a carriage) gets a full keyframe; ensure deltas tolerate a place with no passages and no neighbours. |
| N-3 | **S12 2D** (13b, PR #103; `ARC-45`) | Partly: an unconnected place is drawn 200 m east (`_far_root`). | Treat a place not reachable by passages as its own scene, not a far-off part of the town; keep the stitched layout in one object that can be reset when the region changes. |
| N-4 | **S14 3D** (16d PR #105, 16f) | Partly: places are hard-coded (`cafe`, `street`). | 16f's "places from disclosure" must key dressing by place role or tag, not by world key, and load dressing through one function that can later be made threaded. |
| N-5 | **S16 packages** (E-c PR #99, E-d PR #101) | The refusal "a world is not a part of another world" (`resolve.rs:217`). | Keep the refusal for `requires:`; do not encode it anywhere else, so RT-a can admit World Packs under `regions:` only (QRT-2). |
| N-6 | **worldpack loader** | Flat namespace; id order. | Keep id allocation a single function of ordered keys so a region prefix composes; do not introduce sub-directories under `places/` for another purpose. |
| N-7 | **S19** (TW-b/TW-d PR #107, TW-c) | No. The `clock` frame is exactly what scenery needs. | Keep the `clock` frame sent on every pause/resume/scale change (S19 §7.4); keep calendar/weather disclosed per observer place (QTW-12 regional weather stays additive). |
| N-8 | **Paced controller** (12n-2, S10) | `head_for` with an agenda place in another region. | Add a test that an agenda naming an unreachable place makes the controller wander or wait, never panic. |
| N-9 | **IL-b** (PR #102) | No. | Interaction classes for `board`/`alight` arrive with transport; nothing to change. |
| N-10 | **Protocol** (`ARC-41`) | No. | Clients should ignore unknown record kinds inside an observation (they already must for packs they do not draw). State this in `ADOPTION.md` if it is not there. |
| N-11 | **S20 settings** | No. | Leave room for a "map" key binding and a "doze" host control in the settings layout. |
| N-12 | **QPL-11** (no runtime entity creation) | No, by design (INV-RT-10). | None. |
| N-13 | **presence** (`observe.rs` `present_with`; S11-C `Whereabouts`) | Not today; a cost risk at many regions. | Keep `present_with` behind one function so a place index can replace the scan without changing its result; S11-C's fold likewise. No change requested now. |
| N-14 | **calendar** (TW-a, merged) and **weather** (TW-b) | No: disclosure is keyed to the observer's Presence place, and riders keep a Presence in the carriage. | Keep disclosure keyed to the observer's place (not to "the world"), so per-region weather (QRT-15) stays additive. |
| N-15 | **economy** | No. | Keep the `wage-due` subscription shape the pattern for one more requester (`fare-due`); do not make `money-transferred`'s constructor public. |

---

# 11. Questions

`[OM]` = operator-material.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QRT-1 [OM]** | Is "a region is a Place at the top of a `contains` hierarchy" acceptable rather than a new core concept? | Yes (§5.1). It uses §6's existing hierarchy. |
| **QRT-2 [OM]** | May an existing World Pack (e.g. `market-town`) be used unchanged as a region of another world? | Yes, under `regions:` only, no nesting. It makes "compose two towns into one world with no edits" the step's proof. |
| QRT-3 | Configuration when regions disagree. | The composing world's `configure/` wins; disagreement without it is refused by name. |
| QRT-4 | Are bus and taxi interiors walkable places? | A bus is a walkable place like a carriage (short aisle). A taxi is a place with seats only: sit and talk, no walking. Both use the same Place + `PlaceShape` model. |
| *(QRT-5)* | *Cars as buses or taxis; self-driving.* | **Answered by the operator (ruling 4, §1.2):** buses and taxis; self-driving only within a region, design-only in S21; road trips later. |
| **QRT-6 [OM]** | Real or compressed distances by default? | Real speeds; authors compress distances (`length_m`). The default demo uses 21 km (21 world min, ≈ 105 wall s at 12×). |
| **QRT-7 [OM]** | **Fast-forward during a trip.** May a player speed the clock up while riding? | **Single-player: yes**, as a host pacing control ("doze") available to the admin-token holder, which raises the time scale until arrival and restores it; it is a host journal record, never a world rule (`ARC-67`, INV-RT-5). **Multiplayer: no** by default, because the clock is shared and a ride must never change other players' time (INV-RT-11); a host may still change the scale for everyone through S11-D/TW-c. |
| QRT-8 | Per-region time zones and calendars. | Out of scope; one calendar per world. |
| QRT-9 | Partitioned persistence per region. | No; one save (§2.8 numbers). |
| QRT-10 | Fares in the demo, and economy's one new subscription for taxi `fare-due`. | Yes to both: a ticket shop at each station; taxi fares through the `wage-due` shape, reviewed by economy's owner in RT-c. |
| **QRT-11 [OM]** | **Unvisited places on the map.** Does the map show places and regions the player has never been to? | **Yes, all of them**: a map is public geography, like a real town map or a transit map, and `travel-to` is a journey, not a teleport, so the "visited only" gate that games use for *fast travel* (Pokémon Fly, RDR2) does not apply. An author can hide a place with the `unmapped` tag. A per-player fog of war would be per-person knowledge — a later pack, not atlas. |
| QRT-12 | Who sees the world map? | Every observer of a world with `atlas`. |
| QRT-13 | Which directory holds the default scenery library? | `presentation/mineworld-default/{2D,3D}/scenery/`, CC0/MIT only (Quaternius excluded, §3.3). |
| QRT-14 | Wait for Godot 4.8's texture streaming? | No. The design works on 4.7.2 (`DEP-4`); adopt 4.8 streaming later as an optimization inside the same region-bundle model. |
| **QRT-15 [OM]** | **Weather per region.** One weather for the whole world, or each region its own? | **Per region, in MVP-1, but after RT-h**, as S19's QTW-12 already planned: one `climate` Process per region in the `weather` pack, configured per region (a coastal and an inland region differ). Disclosure is already keyed to the observer's place (N-14), so nothing in S21 blocks it. RT-h ships with one world weather; the scenery reads whatever weather the destination discloses. |

---

# 12. Proposed records (placeholders)

`ARC-RT-a` region is a Place; `ARC-RT-b` regions composition; `ARC-RT-c` transport pack;
`ARC-RT-d` vehicles, runs, boarding; `ARC-RT-e` travel-to and itineraries; `ARC-RT-f` fares;
`ARC-RT-g` atlas and maps; `ARC-RT-h` scenery sequences; `ARC-RT-i` client streaming;
`DEP-RT-a` real-world default sources; `DEP-RT-b` Godot threaded loading and scenery assets.

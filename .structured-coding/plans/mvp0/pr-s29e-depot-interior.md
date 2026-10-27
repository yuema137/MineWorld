# PR S29-e — The 3D depot's interior, boards, ticket window and platform, and the shared station state

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended)

```text
Design revision:        revision 1 (2026-10-19), DRAFT; reconciled 2026-10-19 with S29-a revision 4, which
                        writes R-S29a-d-1's bay and platform frames (counter y 0 … 600; platform 11 × 60 m,
                        piers x 2 200 … 2 800) as this design reads them; nothing here changes
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29de, branch docs/pr-design-s29de,
                        planning base origin/main @ 44f050e3 (#171, 12d, merged; #258 and #261 merged).
                        Written with pr-s29d-depot-exterior.md (same branch) and against the S33-e draft
                        (design-s33de @ 54c9d02e, pr-s33e-pier-halt.md), the frozen S33-c design and PR #254
                        (12e) head e8d481f4
Approved by / evidence: none yet. Step-29's rulings Q-S29-1 … Q-S29-10 and S29-a's Q-S29a-1 … Q-S29a-7 bind and
                        are not re-asked; S33's Q-S33-10 (S29-e's pieces are written for a village-scale
                        building) binds
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S29-d, S29-a,
                        S29-b, S29-f, S33-e2 and 12e
Start condition:        (1) Milestone F's checklist handed to the operator (step-29 §11.2); (2) S29-d merged
                        (step-29 §11.1: "S29-d"); (3) this design frozen; (4) the lane lead's dispatch.
                        S29-b (the newsstand) is not required: without it Market Town's kiosk case is recorded
                        N/A. NONE of (1)–(3) holds at 44f050e3
Execution contract:     §12
Implementer:            Haiku-class from this commit plan, plus an Opus frame review of C4 and C5 (step-29
                        §11.1, the S29-e row: "Haiku with a commit plan for the fit-out and station_state;
                        Opus review of frames")
Lifecycle:              DESIGN FROZEN 2026-10-19 —
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-29-station-landmark.md`](step-29-station-landmark.md)
(APPROVED 2026-10-15) §§1 (outcomes 1, 4, 6), 5 (all), 6.2 (benches, ticket bay, newsstand, boards, clock, arcade
and platform rows), 6.5, 6.6, 7.3, 9.1 (`st_5`–`st_8`), 9.5 (X-5, X-7), 10 (I-S29-3, I-S29-5, I-S29-6,
I-S29-8), 11.1 (the S29-e row), 11.2 (the Lakeside note, superseded by S33's Q-S33-10) ·
[`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§4.4, 4.5, 16.2 (the note to S29-e) · **Parent:**
[`overall.md`](overall.md) · **Consumes:** [`pr-s29d-depot-exterior.md`](pr-s29d-depot-exterior.md) (the kit,
the volumes and doors, `station.*` categories, viewpoints), [`pr-s29a-town-contract.md`](pr-s29a-town-contract.md)
§5.5 (as revised per R-S29a-d-1), 12e (`--slice-geometry`) · **Followed by:** S29-f (2D; it uses
`station_state.gd`), S29-g (agent view), S33-e2 (Lakeside's halt with this PR's pieces) · **Binding parents:**
`CLAUDE.md` §§1, 2, 4 (items 3, 5, 7, 8, 10, 11, 13, 14, 15); `docs/ENGINEERING_RULES.md` §§2–3, 4, 7–9;
`docs/VISUAL_SLICE.md` §§3.4, 6, 7.1; `docs/decisions` `ARC-67`, `ARC-105`, `ARC-106`, `ARC-108`, `ARC-192`,
`INV-S27-2`.

**Decision ids.** No new record. This PR adds a dated note under **`ARC-108`** (the shared module's place and
interface, "last disclosed state per place", the clock rule, the provisional `departures` shape) and one under
**`ARC-192`** (the pieces contract: functions of size and style, reused by S33-e). `ARC-193` … `ARC-195` and
`DEP-102` … `DEP-104` are not used.

No hosted model, image model or paid API is called by this design or by anything it specifies.

---

## 0. Binding inputs, exactly restated

- **Scope (step-29 §11.1, S29-e row).** "hall fit-out (beams, wainscot, benches as a MultiMesh, kiosk, counter,
  boards, clock); `station-tickets` bay; platform, fence and track; `station_state.gd` (shared); `st_5`–`st_8`;
  threshold measurement."
- **Integration checkpoint (same row).** "link probe: spawn → hall → tickets → platform, 0 refusals, the board
  reads `no_service`; perf of the new views."
- **Adversarial criteria (same row).** "X-5, X-7."
- **Honest by construction (step-29 §5, `ARC-108`).** The board lists lines only when a `departures` record is
  disclosed, else "No service yet"; the window is open only when its place discloses a `shop` listing a
  `ticket`-tagged kind; the kiosk is open only when the hall discloses a `shop`; the clock shows the disclosed
  world time, or nothing. No timetable, no train, no clerk figure, no decorative travellers (§§5.2, 7.3).
- **Ruled sizes (Q-S29a-5 … 7, operator 2026-10-18).** The towns' ticket bay 6.0 × 4.6 m; Lakeside's 4.6 × 4.6 m;
  the track on the platform's east edge at Lakeside.
- **Reuse (step-33 §16.2, Q-S33-10).** "S29-e: the halt pieces (window bay, board, bench row) are written so that a
  second world's building can use them at village scale." S33-e asks the same (R-S29e-e-1: "functions of size, not
  of the depot") and may land `station_state.gd` first (its Q-S33e-1).
- **Operator principles (2026-10-18).** Generic and data-driven; every criterion names a second world or variant;
  realistic dimensions with cited sources; nothing faked before trains exist (ARC-108).
- **SPEED FIRST.** Per PR: the build and the tests the commit writes, plus `fast`. Rendered and connected
  acceptance is run at the milestone. Endpoint MERGE-READY; the coordinator merges.

---

## 1. Goal

Furnish the depot so that it is a waiting room and not a shell, and make every fixture tell the truth. A player
who walks in sees the long oak benches in rows under redwood beams, the tiled wainscot, a shut newsstand (Social
Café) or an open one with today's papers (Market Town), a board that reads "No service yet", and a clock that
shows the world's time where the world has a calendar and no hands where it does not. They step into the ticket
bay and find the shutters down behind two brass grilles and a "Closed" plate. They go out through the trackside
doors under the arcade onto the platform, along two empty tracks, with fences and gates at both ends. Nobody waits
for a train.

The rule that decides what each fixture shows is written **once**, in `clients/shared/station/station_state.gd`, a
pure function of what the world discloses. Both clients use it, and the agent view renders the same Observation.
The furniture is built by **pieces** — bench rows, a kiosk, a window bay, a board, a clock, a platform, a track, a
fence — each a function of its size and a style, so that S33-e builds Lakeside's timber halt with the same pieces at
village scale.

---

## 2. Scope

### 2.1 In scope

1. **Records (C1).** Notes under `ARC-108` and `ARC-192`; `clients/3d-spike/PRESENTATION.md` §5 gains the kit's
   `fitout` and `track` groups and §6 "Station pieces"; the locale keys `station.no_service`, `station.window_closed`,
   `station.departures` in `clients/shared/settings/locale/{messages.pot, en.po, zh_Hans.po}` (unless S29-f or
   S33-e landed them first: then reused, C0).
2. **The shared state (C2).** `clients/shared/station/station_state.gd` (`MineWorldStationState`, pure);
   the symlinks `clients/3d-spike/mineworld_station` and `clients/2d/mineworld_station` → `../shared/station` (the
   `mineworld_environment` precedent); the headless check `clients/shared/checks/station_state_check.gd`; the
   ignored Rust test `tools/cli/tests/client_station.rs` and its entry in `scripts/ci_godot.py`'s `GODOT_TESTS`.
   If S29-f or S33-e2 merged the module first, C2 adopts it and adds only what §5.2 lacks.
3. **The pieces (C3).** `clients/3d-spike/scripts/station/pieces.gd` (`StationPieces`: bench rows, kiosk, window
   bay, board, clock) and `clients/3d-spike/scripts/station/track.gd` (`StationTrack`: platform slab and coping,
   track, fence and gate); `--slice-station-kit` builds them at depot scale and at village scale.
4. **The depot's fit-out (C4).** `clients/3d-spike/scripts/station/depot_interior.gd` (`StationDepotInterior`):
   the hall's beams, purlins and nine arches, the wainscot, plaster, chandeliers and lights, the benches, kiosk,
   boards and clock; the ticket bay's counter screen; `clients/3d-spike/scripts/station/fixtures.gd`
   (`StationFixtures`: the per-place state cache and the refresh of every drawn fixture);
   `clients/3d-spike/scripts/slice/slice_link.gd` (one call per observation); the kit's `fitout` group in
   `presentation/mineworld-default/3D/stations/depot.yaml`.
5. **The platform (C5).** The platform slab, coping and detectable-warning strip, the platform board, two tracks,
   fences and gates at both ends, the line leaving through cuttings; the kit's `track` group.
6. **Probes and shots (C6).** `--slice-measure` rows for the fit-out; 12e's `--slice-geometry` over the bay and the
   platform; the `--link` walk extended to the bay and the platform; `--slice-threshold` at the arch; perf counts of
   the `station hall wide` and `station platform` views; `st_5`–`st_8`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The depot's exterior, the kit reader, the tile shader, the landmark check, `st_1`–`st_4` | S29-d |
| The 2D station and its sprites | S29-f (it calls `station_state.gd`) |
| `PlaceLine.roles` and the agent scenario | S29-g |
| Lakeside's halt building | S33-e2 (with this PR's pieces) |
| A train, a carriage, a timetable, a clerk, waiting figures | never before S21 (step-29 §5.2) |
| People drawn in the hall | S26's cast binding, whenever it lands (step-29 §7.3); no decorative figure |
| Any path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/`, `cognition/` | never (I-e-1) |

### 2.3 Dependencies

- **S29-d**, merged: `StationKit`, `StationDepot`, the volumes `station.hall`, `station.tickets`,
  `station.platform`, the three doors, the arcade, `station.*` categories kept through batching, the depiction's
  viewpoints and `station_shots`.
- **S29-a**, merged with R-S29a-d-1 (via S29-d's start condition): the hall's 35 solids, the bay's counter at
  y 0 … 600 with its door at (3 000, 4 200), the platform 11 × 60 m with ten pier solids and two benches. Every
  piece is drawn on those numbers.
- **12e (#254)**: when merged, the geometry rows run; when not, `N/A (12e not merged)` and the milestone runs them.
- **S29-b**: when merged, Market Town's hall discloses `station-news`' shop, so the kiosk-open case is a real
  world's disclosure; when not, that row is recorded N/A and the table test covers the rule.
- **Shared module race.** S29-f and S33-e2 may land `station_state.gd` first (S33-e's Q-S33e-1; step-29 §11.1's
  S29-f row: "or lands it first"). Whoever is first writes §5.2's interface exactly; the others adopt it.

---

## 3. Audit (`origin/main @ 44f050e3`; 12e @ `e8d481f4`; 2026-10-19)

### 3.1 Source anchors

| # | Fact | Where | Consequence |
| --- | --- | --- | --- |
| A-1 | `shop` is economy's owned component on a place: a `Listing` of `listed: [{item, price, in_stock}]`, disclosed to whoever perceives the place | `systems/economy/src/component.rs:96-130` | The window and kiosk rules read `shop.listed` |
| A-2 | `calendar-day` and `calendar-light` are calendar's records "on the place an observer is in" | `systems/calendar/src/component.rs:1-35` | The clock rule reads `calendar-day` on the observer's place |
| A-3 | The protocol client's Observation: `at()` (world time), `place()`, `entity(id)` (tags), `component(id, type)`, `component_value(id, type)` | `clients/protocol/mineworld/observation.gd:56, 69, 91, 127, 137` | The adapter collects one place's components and the tags of the items its shop lists |
| A-4 | The 2D HUD already reads `shop` on the observer's place | `clients/2d/scripts/hud/readers.gd:90-92, 120-123` | S29-f's 2D fixtures read the same component through the same module |
| A-5 | Shared GDScript modules reach both clients through symlinks (`mineworld_environment -> ../shared/environment`, `mineworld_settings -> ../shared/settings`) | `ls -la clients/3d-spike clients/2d` | `mineworld_station -> ../shared/station` in both |
| A-6 | Shared headless checks: `clients/shared/checks/*_check.gd` print `[PASS]`/`[FAIL]` lines and `<name>: PASS|FAIL`, run by an ignored Rust test through a `check(project, script, args)` helper | `clients/shared/checks/store_check.gd:1-12`; `tools/cli/tests/client_settings.rs:117-135` | `station_state_check.gd` follows it; `client_station.rs` registers in `scripts/ci_godot.py:78` `GODOT_TESTS` |
| A-7 | Localized words live in shared catalogs (`messages.pot`, `en.po`, `zh_Hans.po`); `MineWorldText.text(key)` | `clients/shared/settings/locale/`; `clients/shared/settings/clock_format.gd:1-30` | The board and plate words are catalog keys, never code |
| A-8 | Interiors without GI use pendant `OmniLight3D`s (shadowed), one shadowless fill, and a `ReflectionProbe` `UPDATE_ONCE` with `AMBIENT_COLOR`; the one `VoxelGI` covers the café only | `clients/3d-spike/scripts/slice/shop_interior.gd:324-370`; `slice_main.gd:389-450` | The hall is lit the shop's way, not with a 52 m VoxelGI |
| A-9 | No `MultiMesh` anywhere in the 3D client's scripts | `grep -rln MultiMesh clients/3d-spike/scripts` (empty) | The bench rows and the track's ties are the first `MultiMeshInstance3D`s; `SliceBatch` does not touch them (not a primitive mesh, `batch.gd:42`) |
| A-10 | VISUAL_SLICE §3.4: counter 1.00–1.10 m; seat 0.42–0.47 m; §6.2: objects at three sizes; §6.3: walking width ≥ 0.80 m beside counters and tables; step-29 §6.5: aisles between bench rows ≥ 1.2 m | `docs/VISUAL_SLICE.md:116-127, 241-265`; step-29 §6.5 | The measure rows |
| A-11 | `--slice-threshold`: the §7.1 measurement at four points, reported separately | `clients/3d-spike/scripts/slice/slice_probe.gd:9, 571-602` | Gains the arch as a target |
| A-12 | S29-a's hall solids: bench rows 6 000 × 600 at y 6 000 + 2 400 k (k 0 … 16) in banks x 1 200 … 7 200 and 9 600 … 15 600, height 450; the kiosk x 1 200 … 3 200, y 1 500 … 3 500, height 2 400 | `pr-s29a-town-contract.md` §5.5 | Drawn exactly there (12e's 150 mm rule) |
| A-13 | S33-e (draft) needs the window bay, board, clock and bench row as "functions of size, not of the depot" (R-S29e-e-1); its halt's clock shows no hands (Lakeside has no calendar, F-S33e-5); its track ends at a buffer stop and leaves north through a cutting (F-S33e-4) | `design-s33de@54c9d02e:.structured-coding/plans/mvp0/pr-s33e-pier-halt.md` §§2.2, 3.2, 5.2, 10 | The pieces take sizes and a style; the track takes an end kind |
| A-14 | The NRHP nomination's interior: "nine arched elements and natural redwood beam and purlin ceilings"; tile wainscot "with plaster above 8'"; "arched clerestory windows, near the ceiling"; "original clay tile flooring in two sizes and colors"; "Long oak benches"; "chandeliers of bronze and glass" | NRHP 72000248 §7 (`pr-s29d-depot-exterior.md` §3.2) | The fit-out's content and its sources |

### 3.2 Real values and their sources

| Element | Value | Source / status |
| --- | --- | --- |
| Wainscot height | 2.59 m (8½ ft) | Wikipedia (step-29 §2.2); the NRHP says "above 8'" (2.44 m); S29-d's `MEASUREMENTS.md` decides from a HABS photograph and records both |
| Bench seat | 0.45 m | VISUAL_SLICE §3.4 range; S29-a's solids |
| Counter | 1.05 m | VISUAL_SLICE §3.4; S29-a |
| Standard gauge | 1 435 mm (4 ft 8½ in) | the standard gauge of North American railways (AAR) |
| Crossties | 2 590 × 230 × 180 mm (8 ft 6 in × 9 in × 7 in) at ≈ 495 mm (19½ in) centres | typical US main-line practice (AREMA); C1 cites the chapter or labels the values approximate |
| Platform edge above rail head | 0.9 m | step-29 §6.2 (design); C1 records a cited norm or keeps the label "design" |
| Platform edge to the near track's centreline | 1.65 m | approximate; C1 cites or labels it |
| Detectable warning at the platform edge | 610 mm (24 in) wide, truncated domes, contrasting colour | 2010 ADA Standards §§705, 810.5.2; C1 quotes the clauses |
| Track centres | 4.5 m | approximate (design) |
| Platform length | 60 m (the real ≈ 198 m compressed, step-29 §6.2) | S29-a; stated compression |

### 3.3 Findings

- **F-S29e-1. A fixture outside the observer's place cannot be read live.** The observer perceives only its own
  place (step-29 F-S29-3). From the hall, the bay's window and the platform's board are visible but their places'
  disclosures are not. The client keeps, per place, the state from the **last observation made in that place**, and
  shows a fixture's default (shut, "No service yet", no hands) for a place never observed. A remembered state is
  still a disclosed state; an invented one is not. → **Q-S29e-2**.
- **F-S29e-2. The `departures` record does not exist yet** (S21 RT-c, not designed). The board needs a shape to be
  tested now. The provisional shape is the smallest that names lines and times: `{"lines": [{"line": <entity
  ref>, "next": [<world seconds>]}]}`. It is a non-preclusion item for RT-c, and only the adapter and the table
  rows change if RT-c chooses another (step-29 R-S29-6). → **Q-S29e-3**.
- **F-S29e-3. The window's rule needs the listed items' tags.** `shop.listed[].item` is an entity reference; the
  adapter reads that entity's tags from the Observation. If an item's entity is not in the Observation, its tags
  are unknown and the item does not count as a ticket: the window stays shut. That is the honest default.
- **F-S29e-4. The tracks would end at the boundary.** With `reach_m` 100, the near track (≈ x −90 … −95 scene)
  meets the boundary ≈ 33 … 40 m north and south of the axis, just past the platform's ends. A track that stops at
  the near ground's edge reads as a cliff. S33-e's halt has the same problem northward and answers it with a
  shallow cutting closed by trees (F-S33e-4). → **Q-S29e-1**.
- **F-S29e-5. The clock is a fixture too.** `ARC-108` says it "shows the disclosed world time, or nothing". The
  disclosure that the world keeps a calendar is `calendar-day` on the observer's place (A-2). Social Café and
  Lakeside have no calendar, so their clocks have no hands; Market Town's show `at()`'s time of day. The host's wall
  time is never shown.
- **F-S29e-6. The bench backs rise above the world's solid height.** S29-a's bench solids are 450 mm (the seat).
  The drawn bench has a back to ≈ 0.85 m inside the same 600 mm footprint. 12e's probe compares footprints, not
  heights, so it passes; nothing in `bodies` uses the solid's height for walking. Recorded, not changed.
- **F-S29e-7. Every acceptance criterion needs a second world.** Here: **Market Town** joined with the Social
  Café depiction (`pr-s29d-depot-exterior.md` F-S29d-12): its hall discloses a shop (after S29-b) and a calendar,
  so the kiosk opens and the clock has hands — the other branch of every rule; the **renamed-key copy** of Social
  Café; and the **village-scale build** of the pieces (S33-e's halt sizes) in `--slice-station-kit`.

### 3.4 What does not exist

No `clients/shared/station/`, no station piece, no `MultiMesh` in the 3D client, no fixture state, no
`station.*` locale key, no platform or track geometry.

---

## 4. Reuse, both ways

| Need | Decision | Why |
| --- | --- | --- |
| Repeated benches and ties | **Godot's `MultiMeshInstance3D`** | the engine's instancing; one draw per material for 34 bench rows and ≈ 320 ties. Declined: `SliceBatch` merging (it would bake 34 copies into vertices; fine for draws but heavier in memory and invisible to per-row colliders) |
| Furniture, kiosk, counter screen, boards, clock, fence | **Build ours** with `Build` and `Mats` (the café's furniture pattern, step-29 §6.4) | one method; Poly Haven's bench is a park bench |
| Hand-scale props (newspapers, leaflets, an umbrella, a bin) | shipped Poly Haven props where one fits, else procedural boxes and cards | zero new bytes |
| Text on boards and plates | `Label3D` with the shared fonts and catalog words | the slice's signage pattern; CJK covered by the shipped fonts |
| The state rule | **our own pure GDScript**, shared | it is a few lines of domain logic; no library applies |
| Lighting | the shop interior's pattern (A-8) | proven in the slice; a VoxelGI over 52 m would be too coarse |

No new dependency, no new texture.

---

## 5. Design

### 5.1 The flow

```text
server Observation ──► SliceLink.learn(obs)
                          │ StationFixtures.learn(obs)  : cache[place id] = MineWorldStationState.from_observation(obs, obs.place())
                          ▼
                       StationFixtures.refresh(layout)  : for each drawn fixture (piece + the kit place it stands in),
                                                          place = layout.place_of_volume(volume); state = cache.get(place, DEFAULT)
                                                          piece.show(state)   (board text, shutter up/down, kiosk open/shut, clock hands)
```

### 5.2 `MineWorldStationState` (`clients/shared/station/station_state.gd`; `ARC-108`)

A pure function of plain data, so both clients and the headless check call the same code.

```text
class_name MineWorldStationState extends RefCounted
const NONE := "none"; const LINES := "lines"; const OPEN := "open"; const SHUT := "shut"
const DEFAULT := {"board": {"state": NONE}, "window": SHUT, "kiosk": SHUT, "clock": -1}

# components: { component_type: payload } disclosed on ONE place; item_tags: { item id: [tags] }; at: world seconds
static func of(components: Dictionary, item_tags: Dictionary, at: int) -> Dictionary
    board  = {"state": LINES, "lines": [...]}  iff components.departures.lines is a non-empty Array of
             {line, next: Array[int]};  else {"state": NONE}
    window = OPEN iff components.shop.listed has an entry whose item_tags[item] contains "ticket";  else SHUT
    kiosk  = OPEN iff components.shop is a Dictionary with a non-empty `listed`;  else SHUT
    clock  = posmod(at, 86400) iff components has "calendar-day";  else -1
# The adapter, for a client holding a MineWorldObservation:
static func from_observation(obs, place: String) -> Dictionary
```

- A malformed record yields the default for that fixture and a `push_warning` naming the record: never an
  invented value.
- The function knows no place key and no role tag: the caller decides which fixture reads which field.
- **Who draws what.** The board reads `board`, the window `window`, the kiosk `kiosk`, the clock `clock`, each
  from the state of the place the fixture stands in.
- The words: `station.no_service` ("No service yet"), `station.window_closed` ("Closed"), `station.departures`
  ("Departures"); zh_Hans "暂无班次", "暂停售票", "发车信息" (C1; the operator may reword in review).

### 5.3 The pieces (`StationPieces`, `StationTrack`; `ARC-192` note)

Every piece is a static function of **sizes in metres** and a **style** — `{wood, metal, plaster, glass, accent}`
materials chosen by the caller — and returns a `Node3D` whose `show(state)` updates it. No piece reads a kit, a
place or a world; the caller passes numbers. S33-e's halt passes timber materials and village sizes.

| Piece | Signature (sizes in m) | What it draws |
| --- | --- | --- |
| `bench_rows(parent, rows: Array[AABB], seat_h, faces: Vector3, style)` | one entry per row footprint | one `MultiMeshInstance3D` per material (oak seat and back, iron armrests every 1.5 m); one `StaticBody3D` box per row (the footprint, seat height) |
| `kiosk(parent, footprint: AABB, height, style)` | — | a timber kiosk with a counter flap, an awning and a roll shutter; `show`: shutter down (SHUT) or up with stacked papers and a rack (OPEN) |
| `window_bay(parent, width, counter_h, counter_depth, grilles, style)` | — | a counter at `counter_h`, a carved screen to 2.4 m with `grilles` brass grilles, a shutter behind each, a plate; `show`: shutters down with `station.window_closed` on the plate (SHUT), or up (OPEN; no clerk figure — a clerk is a disclosed person, drawn by S26) |
| `board(parent, width, height, style)` | — | a framed board with the heading `station.departures`; `show`: `station.no_service` (NONE) or one row per line (LINES: the line's display name and its next times, formatted by `MineWorldClockFormat`) |
| `clock(parent, diameter, style)` | — | a round wall clock; `show`: hands at the time of day (≥ 0) or a face with no hands (−1) |
| `StationTrack.platform(parent, rect: Rect2, edge: "west"/"east", top_y, style)` | — | the slab, a stone coping, the 610 mm detectable-warning strip along the edge, an invisible air-wall collider on the edge (12e's probe needs a collider on the disclosed edge) |
| `StationTrack.track(parent, from: Vector3, to: Vector3, gauge, rail_y, end_kind, style)` | `end_kind`: `"through"` (into a cutting) or `"buffer"` | rails (two long boxes), ties as one `MultiMeshInstance3D`, ballast; a cutting of planted banks, or a buffer stop |
| `StationTrack.fence(parent, from, to, height, gate_at, style)` | — | painted iron palings with a closed gate |

Each piece's node carries `mw_category` `station.fitout` (pieces of the platform: `station.platform`), so
`SliceBatch` keeps it and the landmark check hides it with the rest of the station.

### 5.4 The depot's fit-out (`StationDepotInterior.build(depot_node, kit, site)`)

All positions from the kit's `fitout` group (hall-frame mm → scene via S29-d's one mapping function), each group
with a `source`:

1. **Ceiling.** Nine transverse arches (the NRHP's "nine arched elements") dividing the hall into ten bays of
   5.18 m, as laminated redwood arches springing from shallow pilasters (≤ 120 mm proud of the wall, inside 12e's
   150 mm); longitudinal beams and purlins between them (`brown_planks_09`, tinted redwood). Replaces S29-d's plain
   ceiling plane.
2. **Walls.** The wainscot in `glazed_tile` `frieze` to the kit's height, a ziggurat frieze course at its top
   (NRHP); plaster above. The clerestories are S29-d's.
3. **Light.** Nine bronze-and-glass chandeliers, one per arch bay centre line, lowest point ≥ 2.6 m (above the
   1.90–2.30 m pendant range, because the hall is 9 m to the eaves: design); emissive glass; four shadowed
   `OmniLight3D`s and one shadowless fill; one `ReflectionProbe` `UPDATE_ONCE`, `AMBIENT_COLOR` (A-8).
4. **Benches.** `bench_rows` over the 34 footprints (A-12), seat 0.45 m, all facing north (design: the board
   hangs on the north end wall); aisles 1.2 m at the side walls and 2.4 m in the centre, as S29-a writes them.
5. **Kiosk.** `kiosk` on the kiosk footprint (A-12); hand scale when open: newspapers on the counter, a leaflet
   rack.
6. **Boards and clock.** The hall board on the north end wall at 3.0 m (centre), 3.6 × 1.8 m; the clock above the
   trackside door, 0.9 m diameter. The platform board under the arcade beside the trackside door (C5).
7. **Hand scale (always, VISUAL_SLICE §6.2).** A timetable-leaflet rack by the trackside door (empty slots before
   S21: a rack is furniture, leaflets would be a timetable), a litter bin, and one lost umbrella on a bench.
8. **The ticket bay.** `window_bay` on the bay's counter (bay frame y 0 … 600, the far south wall), 6.0 m wide, two
   grilles; the bay's floor in `quarry`; one pendant light.
9. **Doors.** S29-d's leaves; nothing changes the openings.

### 5.5 The platform (`StationTrack` over the kit's `places.platform` and `track` groups)

- The slab over the platform's floor (11 × 60 m), top at the hall floor's level; the edge on the **west**
  (R-S29a-d-1); the detectable-warning strip along it.
- The arcade is S29-d's; two benches under it on S29-a's bench solids; the platform board beside the door.
- Two tracks: the near centreline 1.65 m beyond the edge, the second 4.5 m beyond it; rail head 0.9 m below the
  platform's top; standard gauge; ties at 495 mm.
- At both platform ends a fence across the platform's width with a closed gate (1.2 m, design); beyond the ends the
  tracks run on to the boundary and into a cutting of planted banks that closes the view of the line (Q-S29e-1);
  `end_kind` `"through"`.
- No train, no carriage, no signal aspect that implies a movement (step-29 §5.2).

### 5.6 Fixture state in the 3D client (`StationFixtures`)

- `learn(obs)`: `cache[obs.place()] = MineWorldStationState.from_observation(obs, obs.place())`.
- `refresh(layout)`: for each registered fixture `{piece, volume}`, `place = layout.place_of_volume(volume)`;
  `piece.show(cache.get(place, MineWorldStationState.DEFAULT))`. Called after `learn` and when a binding changes.
- Offline (no server): every fixture shows `DEFAULT`.
- The cache holds only what was disclosed; it is never written from anything else (Q-S29e-2).
- `SliceLink.learn` (16c, `slice_link.gd` ≈ :244) gains one call; nothing else in the link changes.

### 5.7 Shots and views

| Name | Pose | Must show |
| --- | --- | --- |
| `st_5_hall_wide` | S29-d's `station hall wide` pose | benches in rows, the arches, the wainscot, the board |
| `st_6_ticket_window` | in the bay, 2.2 m from the counter, eye 1.6 m, facing the counter | shutters down, "Closed" legible |
| `st_7_board` | in the hall, 5 m from the board, facing it | "No service yet" legible |
| `st_8_platform` | on the platform at its north end, 1.7 m, looking south along the edge | the edge, the strip, both tracks, the far fence and gate |

All committed to `clients/3d-spike/shots/social-cafe/station/`; the depiction's `station_shots` gains them.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-e-1 (I-S29-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/`, `cognition/` changes | C7 diff scan |
| I-e-2 (I-S29-3) | Every fixture shows only disclosed state: `station_state.gd` is the one rule; no fixture text is a literal; no train | table test; X-7; AC-e-9 |
| I-e-3 | The state rule is pure: no I/O, no clock, no intent; it never reads a place key or a role tag | review; client-rules scan |
| I-e-4 | Pieces are functions of sizes and a style; none reads a kit, a world or a place | review; the village-scale build (AC-e-2) |
| I-e-5 | Every drawn collider on walkable floor has a disclosed solid within 150 mm; every disclosed edge has a collider | 12e's `--slice-geometry` |
| I-e-6 (I-S29-6) | Every fit-out number has a source in the kit | `STATION_KIT_UNCITED` |
| I-e-7 (I-S29-8) | The new views meet the absolute gates; existing views stay within step-29 §6.6's caps | AC-e-6 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | client-rules scan (the module and pieces); doc scripts | `fast`; local |
| Headless Godot, per commit (through `with-slot.sh`) | `station_state_check.gd` (the table, X-7); `--slice-station-kit` at depot and village scale with measure rows | `client_station.rs` (ignored; local run recorded) |
| Rendered / connected (**milestone**) | `--link` walk to bay and platform; `--slice-geometry`; `--slice-threshold`; perf counts; X-5 on a planted copy | `client_far_field.rs` / `client_station.rs` (ignored), `clients-probes` |
| Gate 1 / Gate 2 | NOT REQUIRED / the connected walk (milestone) | — |

---

## 8. Acceptance criteria (fixed before measuring)

**M** = run at the milestone; **P** = in the PR.

| ID | Criterion | Second world / variant | When | Step id |
| --- | --- | --- | --- | --- |
| AC-e-1 | `station_state_check.gd` passes every row: no `departures` → NONE; a planted `departures` with two lines → LINES with both; `shop` listing only an untagged kind → window SHUT, kiosk OPEN; listing a `ticket`-tagged kind → window OPEN; an item whose entity is absent → SHUT; no `calendar-day` → clock −1; with it → `posmod(at, 86400)`; a malformed record → default and a warning | rows built from each reference world's disclosure shape: Social Café (no shop, no calendar), Market Town (the newsstand's shop, a calendar), Lakeside (neither) | P | AC-S29-5; X-7 |
| AC-e-2 | `--slice-station-kit=depot` prints the fit-out rows: every bench seat 0.42–0.47 m; the counter 1.00–1.10 m; aisles ≥ 1.2 m; 34 bench instances; the board's and plate's text keys | the **village-scale build** (waiting room 8 × 5 m, bay 4.6 × 4.6 m, platform 4 × 40 m, one track, `buffer` end; S33-e's sizes): the same rows hold, nothing depot-specific appears | P | §6.1, step-33 §16.2 |
| AC-e-3 | Connected (`--world --link`): spawn → hall → ticket bay → hall → platform, each crossing a server place change, 0 refusals; the board's label is `station.no_service`; the window SHUT; the kiosk SHUT; the clock has no hands | (a) the renamed-key copy: identical; (b) **Market Town** joined: the same walk binds with 0 misses, the kiosk OPEN (after S29-b; else N/A recorded), the clock shows Market Town's time, the window SHUT | M | AC-S29-3, AC-S29-5 |
| AC-e-4 | `--slice-geometry` over the hall, bay and platform: benches, kiosk, counter, piers and edges within 150 mm, both directions | Market Town joined | M (N/A if 12e unmerged) | §6.5 |
| AC-e-5 | **X-5**: on a planted copy of Market Town with a shop at the ticket office selling `newspaper` (not `ticket`-tagged), the window stays SHUT on the link walk; with a kind tagged `ticket` added, it opens | the planted copy is itself the second world | M | X-5 |
| AC-e-6 | Counts in `station hall wide` and `station platform` under the absolute gates (16.7 ms p95 by the operator; ≤ 2 048 MB); the existing M-1 views within step-29 §6.6's caps; `3d.pck` still ≤ 6 MB above its pre-S29-d size | Market Town joined: `station hall wide` with the kiosk open (more props) recorded | M | AC-S29-6 |
| AC-e-7 | `--slice-threshold` at the arch: four points, reported separately (VISUAL_SLICE §7.1) | the bay door (a second threshold) reported the same way | M | step-29 §6.5 |
| AC-e-8 | `st_5`–`st_8` committed; text legible in `st_6` and `st_7` (Opus frame review); the operator's look queued | — | P (commit), M (look) | §9.1 |
| AC-e-9 | Nothing pretends: no node in the station named or categorized as a train or carriage; every `Label3D` text in the station comes from a catalog key; `grep -rn "No service" clients/` matches only the catalogs | the village-scale build | P | I-S29-3 |

### 8.1 Planted mutations

| Id | Plant | Must fail with | When |
| --- | --- | --- | --- |
| **X-5** | as AC-e-5 | the window stays SHUT with the newspaper-only shop (a pass), opens with the ticket kind; planting "open iff any shop" in `window` fails AC-e-1's untagged row by name | M (walk); P (table) |
| **X-7** | `board` returns NONE whatever is disclosed | `[FAIL] departures: two lines` in `station_state_check.gd` | P |
| M-e-1 | the clock reads the host's time (`Time.get_time_dict_from_system`) | `[FAIL] no calendar → no hands` | P |
| M-e-2 | `StationFixtures.refresh` shows OPEN for a never-observed place | AC-e-3's window row from the hall | M |
| M-e-3 | a bench row 300 mm off its solid | `--slice-geometry` names the row | M (12e) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; `N/A` needs the audited reason. Haiku-class implementer: follow the steps in
order and do not improvise; anything not covered is a stop (§12).

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s29e`. Never `cd` elsewhere.

1. Re-read this commit's section, the design sections it cites, and every file you will edit, in full.
2. Make the numbered steps in order.
3. Godot and cargo only through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`. Record per
   command: what ran, rows, PASS / FAIL / INCONCLUSIVE.
4. Static: `python3 scripts/check_decision_ids.py`; `python3 scripts/check_doc_headings.py`;
   `cargo test -p mineworld-acceptance --test client_rules` where GDScript changed.
5. For each listed mutation: copy the file aside, plant, run the named check, record the red line, restore from the
   copy (`cmp` clean), run green.
6. `git status --short`, `git diff --stat`: only the commit's **Files**.
7. Update §15 and `handoff-s29e.md` in the same commit.
8. Stage by explicit path; commit with the line `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`
   (the implementing model's own attribution line if the session gives another). No heredoc writes, no `sed -i`,
   no `python3 -c`, no `curl`.
9. A material stop is reported, not improvised around.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15); `.structured-coding/plans/mvp0/handoff-s29e.md` (new).
- **Steps:**
  1. Record the base commit; quote the dispatch (Milestone F handed over; else stop).
  2. Record the merge state of S29-d (required), S29-b, S29-f, S33-e2, 12e.
  3. `ls clients/shared/station/` — if `station_state.gd` exists, read it and record which of §5.2's functions it
     has (C2 adopts it). `grep -n "station\." clients/shared/settings/locale/en.po` — record which keys exist.
  4. Re-read A-1 … A-14 on the base; read S29-d's merged `depot.yaml`, `depot.gd`, the depiction's viewpoints, and
     the bay's and platform's merged place files (R-S29a-d-1's numbers); any difference from §5 is recorded (bounded)
     or a stop (a frame that cannot be drawn by translation).
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: the start condition holds; the module race resolved.
- **Commit message:** `docs(plan): S29-e — open the ledger at the base`

#### C1 — Records and words

- **Files:** `docs/decisions/ARC-108.md`, `docs/decisions/ARC-192.md` (notes), `docs/decisions/INDEX.md`
  (regenerated if headings changed), `clients/3d-spike/PRESENTATION.md`, `clients/shared/settings/locale/{messages.pot,
  en.po, zh_Hans.po}`.
- **Steps:**
  1. `## ARC-108 note — 2026-10-…: the shared module, remembered state, the clock, departures` stating §5.2, Q-S29e-2's
     and Q-S29e-3's answers, F-S29e-5.
  2. `## ARC-192 note — 2026-10-…: station pieces` stating §5.3 (functions of size and style; S33-e reuses them).
  3. `PRESENTATION.md` §5: the `fitout` and `track` groups (each key, its unit, its source rule); §6 "Station pieces":
     the table of §5.3, and "a piece decides nothing a world owns".
  4. The three keys (skip those C0 found), English and Chinese per §5.2.
- **Validation:** `check_decision_ids.py`; `check_doc_headings.py`; the 3D and 2D projects import with no catalog
  error (headless `--import`).
- [ ] Implementation. [ ] Validation. [ ] Review: the words say only what is true before trains ("No service yet",
  not "No trains today").
- **Commit message:** `docs: station fixtures and pieces — notes on ARC-108 and ARC-192; the station words (S29-e)`

#### C2 — The shared station state

- **Files:** `clients/shared/station/station_state.gd` (+ `.uid`), `clients/3d-spike/mineworld_station`,
  `clients/2d/mineworld_station` (symlinks), `clients/shared/checks/station_state_check.gd` (+ `.uid`),
  `tools/cli/tests/client_station.rs`, `scripts/ci_godot.py` (`GODOT_TESTS` gains `client_station`).
- **Steps:**
  1. `MineWorldStationState` exactly per §5.2 (or the missing functions only, if C0 found the file).
  2. The symlinks, made with `ln -s ../shared/station <client>/mineworld_station` (relative, as the existing ones).
  3. The check: one row per AC-e-1 case, `[PASS]`/`[FAIL] <row>`, then `station_state_check: PASS|FAIL`.
  4. `client_station.rs`: `#[ignore]` test `station_state_table` running the check in `clients/3d-spike` and in
     `clients/2d` (both must pass; the helper copied from `client_settings.rs:117-135`).
- **Validation:** the check headless in both projects (through `with-slot.sh`): every row PASS. **X-7** and
  **M-e-1** planted and reverted.
- [ ] Implementation. [ ] Validation. [ ] Review: no place key, no role tag, no time source but `at`.
- **Commit message:** `feat(clients): the shared station state — board, window, kiosk and clock from disclosure (S29-e)`

#### C3 — The pieces

- **Files:** `clients/3d-spike/scripts/station/pieces.gd`, `clients/3d-spike/scripts/station/track.gd` (+ `.uid`s),
  `clients/3d-spike/scripts/slice/slice_probe.gd` (`station-kit` builds pieces and prints their rows),
  `presentation/mineworld-default/3D/stations/depot.yaml` (`fitout` and `track` groups with sources).
- **Steps:**
  1. Each function of §5.3 table, in that order, each with `show(state)`.
  2. `--slice-station-kit=depot`: after S29-d's build, build the pieces from the kit and print AC-e-2's rows.
  3. `--slice-station-kit=village` (a built-in parameter set, not a kit file: S33-e owns Lakeside's kit): the pieces
     at S33-e's sizes and a timber style; the same rows.
- **Validation:** both builds headless; AC-e-2's rows PASS.
- [ ] Implementation. [ ] Validation. [ ] Review: no piece reads the kit, a place or a world (I-e-4).
- **Commit message:** `feat(3d): station pieces — benches, kiosk, window bay, board, clock, platform, track, fence (S29-e)`

#### C4 — The depot's fit-out and the fixture state

- **Files:** `clients/3d-spike/scripts/station/depot_interior.gd`, `clients/3d-spike/scripts/station/fixtures.gd`
  (+ `.uid`s), `clients/3d-spike/scripts/slice/slice_link.gd`, the Social Café depiction (one call to
  `StationDepotInterior.build` after `StationDepot.build`).
- **Steps:** §5.4 items 1–9; §5.6.
- **Validation:** offline build; `--slice-station-kit=depot` rows still PASS; offline frames at `st_5`–`st_7`'s poses
  written to `shots/social-cafe/station/review/` (not committed).
- [ ] Implementation. [ ] Validation. [ ] Review (**Opus frame review**): the hall reads as the depot's waiting room
  against the HABS interior photograph; three object sizes present; text legible; no light leaks.
- **Commit message:** `feat(3d): the depot's waiting room — arches, wainscot, benches, kiosk, board, clock, ticket window (S29-e)`

#### C5 — The platform

- **Files:** `clients/3d-spike/scripts/station/depot_interior.gd` (the platform section) or a sibling
  `depot_platform.gd` (≤ 500 lines per file), the kit's `track` group.
- **Steps:** §5.5.
- **Validation:** offline frame at `st_8`'s pose to `review/`; `--slice-station-kit=depot` prints the platform rows
  (edge height, gauge, tie pitch, strip width).
- [ ] Implementation. [ ] Validation. [ ] Review (**Opus frame review**): the line leaves through the cuttings without
  a visible end; nothing suggests a train is due.
- **Commit message:** `feat(3d): the depot's platform — edge, two tracks, fences and gates (S29-e)`

#### C6 — Probes, views and shots

- **Files:** `slice_probe.gd` (`measure` fit-out rows; `threshold` at the arch and the bay door), 12e's
  `slice_probe_geometry.gd` (only if 12e merged), the depiction (`probe_route` to the bay and platform by tags;
  `station_shots` += `st_5`–`st_8`), `tools/cli/tests/client_station.rs`
  (`three_d_station_walk`: the walk on Social Café, the renamed-key copy and Market Town; `x5_ticket_window`:
  the planted copy), `clients/3d-spike/shots/social-cafe/station/st_5…st_8`.
- **Validation:** one local connected walk on Social Café through `with-slot.sh` (AC-e-3's first case); the shots
  written; perf counts for the two views. Everything else marked M.
- [ ] Implementation. [ ] Validation. [ ] Review: each M row has its exact command recorded in §15.
- **Commit message:** `test(3d): the station walk, threshold, geometry and shots st_5–st_8 (S29-e)`

#### C7 — Close-out

- **Files:** this file (§15), `docs/HUMAN_REVIEW_QUEUE.md` (a row for `st_5`–`st_8`), `handoff-s29e.md`.
- **Steps:** I-e-1's diff scan; push; open the PR; `fast` green on the exact head; report MERGE-READY with the PR
  number and head SHA.
- [ ] Implementation. [ ] Validation: `fast` green. [ ] Review: every §8 row evidenced or marked M with its command.
- **Commit message:** `docs(plan): S29-e — close-out`

---

## 10. Requirements this PR places on other PRs

- **R-S21-e-1 (on S21 RT-c; Q-S29e-3).** Disclose `departures` to observers in `station`-tagged places (step-29
  §14.6) in the shape `{lines: [{line, next: [world seconds]}]}`, or change `station_state.gd`'s adapter and table
  rows in the same PR.
- **R-S29f-e-1 (on S29-f).** Call `MineWorldStationState` through `mineworld_station`; keep the 2D fixtures' state
  per place as §5.6 does (Q-S29e-2); reuse the C1 words.
- **R-S33e-e-1 (on S33-e2).** Build the halt with `StationPieces` / `StationTrack` (village sizes, timber style,
  `end_kind` `"buffer"`); register its fixtures with `StationFixtures`; if S33-e2 lands `station_state.gd` first,
  write §5.2 exactly.
- **R-S29g-e-1 (on S29-g).** The agent view renders the Observation; it does not call `station_state.gd` and infers
  no state (INV-S27-2), so the two agree by construction.

## 11. Edits to parent documents (owner: the lane lead / primary session)

| Document | Edit |
| --- | --- |
| `step-29-station-landmark.md` §5.1 | the remembered state per place (Q-S29e-2); the clock rule (F-S29e-5) |
| `step-29-station-landmark.md` §14.6 | the provisional `departures` shape (Q-S29e-3) |
| `step-29-station-landmark.md` §6.2 | the tracks leave through cuttings (Q-S29e-1); the cited track values (§3.2) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S29-e — the 3D depot's interior, ticket window, platform and track, the
                         shared station state, the reusable station pieces
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s29e-depot-interior.md
RELATED / BINDING DOCS:  step-29-station-landmark.md (§§5, 6.2, 6.5, 6.6, 7.3, 9.1, 9.5, 10, 11);
                         step-33-lakeside-3d.md (§§4.4, 4.5, 16.2); pr-s29d-depot-exterior.md;
                         pr-s29a-town-contract.md §5.5 (as revised per R-S29a-d-1); docs/VISUAL_SLICE.md §§3.4,
                         6, 7.1; clients/3d-spike/PRESENTATION.md; docs/decisions ARC-67, ARC-105, ARC-106,
                         ARC-108, ARC-192; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s29e, its own, held by one session
BRANCH:                  mvp0/pr-s29e, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator AND S29-d merged AND this design frozen
                         AND the lane lead's dispatch                             (step-29 §§11.1, 11.2)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–6, as §14's rulings shape them
FROZEN INVARIANTS:       step-29 I-S29-1, I-S29-3, I-S29-5, I-S29-6, I-S29-8; §6 I-e-1 … I-e-7; §8 as written
SEQUENCE:                C0 … C7 (§9)
IMPLEMENTER:             Haiku-class from this plan; Opus frame review at C4 and C5
ALLOWED COMMANDS:        Godot and cargo only through cast-slots/with-slot.sh: ./mineworld-slice and its
                         --slice-* modes, godot --headless (--import, --script), cargo build, cargo test -p
                         <crate> --test <file> [-- --ignored <name>]; python3 scripts/check_decision_ids.py,
                         check_doc_headings.py, ci_layer.py --list; ln -s (C2's two links only); git; gh (pr
                         create/view/checks, run view); mkdir -p; cp; cmp; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or a whole
                         member suite locally (R-LOAD-1); reading any key or .env; any model or image call;
                         editing a path I-e-1 forbids; a new texture; drawing a train or a waiting figure;
                         merging
MATERIAL STOPS:          a start condition missing; a merged place frame that translation cannot draw; a needed
                         change under I-e-1's paths; S29-f or S33-e2 merged a station_state.gd whose interface
                         differs from §5.2 in a way that changes a rule (not just a name); an absolute perf gate
                         failed in a new view at the milestone; any change to a frozen step invariant or
                         acceptance
PLATFORMS:               Linux, macOS, Windows (the clients and farfield layers' nightly; per PR `fast` only)
VALIDATION BUDGET:       per commit: the build, the headless checks and one local Godot run per mode touched,
                         through with-slot.sh; the rows marked M in §8 at the milestone, in one batch. Gate 1:
                         NOT REQUIRED. Gate 2: the connected walk (milestone). No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s29e.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to `fast` green: authorized           (working rules §21; charter SPEED FIRST)
                         endpoint: MERGE-READY = READY FOR OPERATOR REVIEW: pushed, `fast` green on the exact
                         head, reported with the PR number and head SHA (charter SPEED FIRST)
                         merge: NOT by the implementation session — the coordinator merges
POST-MERGE SYNC OWNER:   lane lead: step-29, overall.md, the progress lines; implementation session: this
                         ledger, the merge identity, evidence, deviations
STOP CONDITION:          S29-e MERGE-READY (READY FOR OPERATOR REVIEW) — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | The lit hall with 34 bench rows breaks 16.7 ms (R-S29-1) | `MultiMesh` benches; four shadowed lights only; `ReflectionProbe` once; counts in C6; the reduced hall is the operator's lever |
| R-e-2 | Three PRs race to write `station_state.gd` | §5.2 is the one interface; C0's adoption branch; a rule difference is a material stop |
| R-e-3 | S21 chooses another `departures` shape | Q-S29e-3; only the adapter and its rows change (R-S21-e-1) |
| R-e-4 | A Haiku implementer drifts on art | Opus frame review at C4 and C5 against the HABS interior and the NRHP's list |
| R-e-5 | Board text unreadable in Chinese at distance | shared CJK fonts; `st_7` reviewed in both locales (the probe's `--locale`) |
| R-e-6 | Market Town's kiosk case is missing because S29-b is late | the table test holds the rule; the connected case is recorded N/A, then run at the milestone |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29e-1 [operator — where the tracks go, F-S29e-4]** | With the walkable area ending 100 m from the origin, the tracks would stop just past the platform's ends, like a cliff. Let them run on to the boundary and into a shallow cutting of planted banks that hides the line's end (the same answer S33-e gives for the halt), as a stated exception to "nothing beyond the boundary suggests a path"? | **Yes.** A railway leaving a through station is what a station is; the server never lets anyone onto the track (the platform's edge is a wall), and the cutting closes the view as RL-e0 requires. *中文：轨道越过站台端头延伸到边界，进入两侧有绿化坡的浅路堑把尽头挡住（和 S33-e 小站一样），作为"边界外不暗示可走路径"的说明性例外。同意吗？* |
| **Q-S29e-2 [operator — fixtures seen from another place, F-S29e-1]** | From the hall you see the ticket window, but the world discloses the window's state only to someone standing at it. Show the state from the last time the player stood there (and "shut" if never), or always "shut" until they step in? | **The last state seen there, "shut" if never.** It is still something the world disclosed, and it stops the window flipping open as the player steps through the door once S21 opens it. *中文：从大厅看售票窗时，显示玩家上次站在那里时世界披露的状态（从未去过则显示关闭）。同意吗？* |
| **Q-S29e-3 [operator / S21 — the departures record, F-S29e-2]** | The board needs a `departures` record that S21 has not designed. Fix the provisional shape now as `{lines: [{line, next: [world seconds]}]}`, which S21's RT-c either adopts or changes together with the board's adapter and tests? | **Yes.** It lets the board's rule be tested now (X-7) and costs RT-c nothing it would not decide anyway. *中文：先把 S21 的 `departures` 暂定为"线路 + 下几班时刻"的最小结构，S21 采用或连同看板适配一起改。同意吗？* |

### 14.1 Rulings

Operator, 2026-10-19 (relayed by the lane lead with the freeze): **Q-S29e-1** yes, the tracks run on into a
planted cutting; **Q-S29e-2** the last window state seen, "shut" if none was seen; **Q-S29e-3** the provisional
`departures` shape `{lines: [{line, next: [s]}]}`.

---

## 15. Ledger (live during implementation)

### Commit ledger

Executed under the lane charter's SPEED FIRST (2026-10-18): the commit plan C0 … C7 is folded into four
semantic commits; no mutation runs and no independent reviewer (charter); the Opus frame review of C4/C5 is
the implementing session's own look at the offline frames (recorded below); `fast` is the gate.

| Commit | What | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| 1 | records, words, the kit's `fitout` and `track` groups (C0, C1) | [x] | [x] decision ids, doc headings | [x] words say only what is true before trains |
| 2 | the shared station state (C2): S29-f's module adopted, the clock added | [x] | [x] both shared checks headless: PASS | [x] no place key, no time source but `at` |
| 3 | pieces, fit-out, platform, fixtures, furnishings binding (C3–C5) | [x] | [x] `--slice-station-kit` (depot and reduced-hall variant): all station kit and piece checks pass | [x] frames below |
| 4 | shots `st_5`–`st_8`, ledger (C6, C7) | [x] | [x] `--slice-station-shots` | [x] `fast` on the exact head (PR body) |

### Evidence

- **Base.** Branch `mvp0/pr-s29e` stacked on S29-d (`origin/mvp0/pr-s29d` @ `6a099d7c`, PR #403, not merged: a
  merge freeze holds main; `origin/main` @ `423c301b`). S29-b: not merged (Market Town's kiosk-open connected
  case is N/A until it is; the table covers the rule). S29-f (#366): open, held for the operator. 12e: merged
  (`slice_probe_geometry.gd` is on main) — AC-e-4 runs at the milestone.
- **Shared checks** (`godot --headless --path clients/shared --script res://checks/…`, 2026-10-20, through
  `with-slot.sh`): `station_state_check.gd` → `all station state checks pass` (S29-f's 13 cases);
  `station_clock_check.gd` → `all station clock checks pass` (7 cases: no calendar → −1, calendar → time of day,
  from an observation frame). PASS. `client_station.rs` runs both (`#[ignore]`, `GODOT_TESTS` += `client_station`).
- **AC-e-2** (`./mineworld-slice --station-kit`, 2026-10-20): every `KIT` row ok; `PIECE` rows at depot scale and
  at village scale (S33-e's sizes, timber style, one track with a `buffer` end): bench seat 0.447 m (0.42–0.47),
  counter 1.050 m (1.00–1.10), window bay, board and clock within their sizes, warning strip 0.610 m, gauge
  1.435 m, rail head 0.900 m under the platform, tie pitch 0.495 m; the plate's and the heading's text keys
  `station.window_closed`, `station.departures`; the buffer stop drawn only at village scale. The depot's fixtures
  offline: board "No service yet" (hall and platform), clock no hands, window shut. `all station piece checks
  pass`, `all station kit checks pass`. The same on `tests/station_kits/variant-reduced-hall.yaml` (12 × 30 m hall):
  PASS. The 34 bench instances are drawn from the world's furnishings (below), not from the kit.
- **Static.** `cargo fmt --check`; `cargo clippy -p mineworld-cli --tests -D warnings`; `cargo test -p
  mineworld-acceptance --test client_rules` 5/5; `check_decision_ids.py`; `check_doc_headings.py`. PASS.
  `cargo test -p mineworld-cli --test furnishings`: 4/5 — `every_kind_every_world_names_is_bound_in_each_default_pack`
  fails on **2D only** (`bench.station`, `kiosk.news` have no 2D role); the 3D half now passes (F-S29e-impl-3).
- **Frames** (`./mineworld-slice --station-shots`, standalone, committed `shots/social-cafe/station/st_5`–`st_8`;
  looked at by the implementing session): `st_5` the hall reads as a waiting room — rows of oak benches facing
  north, eight redwood arches springing from pilasters between the clerestories, purlins, boarded ceiling, tiled
  wainscot under a frieze course, nine chandeliers, the board on the north wall; `st_6` two brass grilles,
  shutters down, "Closed" legible; `st_7` "Departures / No service yet" legible from 5 m; `st_8` the platform's
  coping and yellow strip, two tracks on ballast in the sunken trackbed, the overbridge closing the line at the
  far end. No train, no figure. The operator's look is queued at the milestone (AC-e-8).

### Milestone rows (M) and their commands

| Row | Command |
| --- | --- |
| AC-e-3 | `./mineworld-slice --world --station-walk` (Social Café; `--world=market-town`); the walk to the bay and the platform and the `FIXTURE` rows are to be added to the walk at the milestone (F-S29e-impl-5) |
| AC-e-4 | `./mineworld-slice --world --geometry` (12e) over the hall, bay and platform |
| AC-e-5 | X-5 on a planted Market Town copy (a newspaper shop at the ticket office), then a `ticket`-tagged kind |
| AC-e-6 | `./mineworld-slice --perf` (views `station hall wide`, `station platform`) |
| AC-e-7 | `./mineworld-slice --threshold` at the arch and the bay door |

### Mutations

Not run (charter SPEED FIRST). X-7 and M-e-1 are pinned by named rows the checks print
(`STATION of_observation lines2 open shut`, `CLOCK no calendar -> no hands -1`) and asserted in `client_station.rs`.

### Deviations and findings

F-S29e-1 … F-S29e-7 (§3.3), recorded at design time.

- **F-S29e-impl-1 (module race; bounded).** S29-f wrote `MineWorldStationState` first, on its branch (#366, not
  merged). Its interface differs from §5.2 by name and shape (`of(observation)`, `evaluate(place_tags, departures,
  shop, item_tags)`, `nothing_disclosed()`; no `clock`), and its window/kiosk rule reads the place's
  `ticket-office` role tag (§5.2 said the rule reads none). As the dispatch instructed, S29-e adopts S29-f's file
  and check **byte for byte** (so either PR merges cleanly over the other) and adds the clock as its own pure
  module, `station_clock.gd` (`MineWorldStationClock`), instead of a `clock` key. No rule changes; I-e-3's "no role
  tag" is relaxed to S29-f's use of one role tag to route a field (recorded in ARC-108's note).
- **F-S29e-impl-2 (furnishings).** Per the dispatch, the hall's benches and kiosk and the platform's benches are
  the world's `body.furnishings` (already on main, S29-a rev. 4), drawn through `furniture/furnishings.yaml`: the
  binding gains `"piece"` (a `StationPieces` builder sized to the disclosed solids) beside `"model"`. Standalone,
  they come from recorded station observations written by the new `station_furnishings_fixture.rs` from bodies'
  own `place-shaped` facts (the café's Q-FD-5 rule). Each piece is merged by `SliceBatch` (its toggled parts are
  surface-material meshes, which the batch leaves alone); the MultiMesh of §5.3 is used for the ties only.
- **F-S29e-impl-3 (2D roles; for S29-f).** `bench.station` and `kiosk.news` have no 2D role in
  `2D/renderer/godot.yaml` on main or on S29-f's branch, so `furnishings.rs`'s every-kind test fails on its 2D half
  (deferred layer). The 2D binding is S29-f's.
- **F-S29e-impl-4 (the near ground).** The far field's near ground (y −0.14) covered a trackbed 0.9 m below the
  platform. `SliceFarFieldView.cut_near_ground(holes)` re-meshes the near-ground disc without the trackbed's
  rectangle (Godot `Geometry2D.clip_polygons` + `triangulate_polygon`); the collider is unchanged. The far-field
  rendered check runs at the milestone.
- **F-S29e-impl-5 (scope deferred to the milestone).** The `--link` walk's extension to the bay and the platform,
  `--slice-threshold` at the arch, perf counts and the X-5 planted copy are milestone rows (SPEED FIRST); their
  commands are above.
- **F-S29e-impl-6 (arches).** NRHP's "nine arched elements" are read as S29-d reads the clerestories: nine bays,
  so the kit's `fitout.arches` is 8, standing between the windows, with one chandelier per bay (§5.4 said nine
  arches and ten bays, which would put ribs across the clerestories).
- **F-S29e-impl-7 (fit-out numbers).** `fitout` gains `counter_mm` (1 050, VISUAL_SLICE §3.4) and the board's
  centre is 3.7 m (not 3.0 m) so the board clears the 2.59 m wainscot and its frieze; both cited in the kit.
- **F-S29e-impl-8 (client rules).** Adding `clients/3d-spike/mineworld_station` brings S29-f's check under the 3D
  place-key scan; its literal `station` tag lists are admitted in `client_rules.rs` with the reason.

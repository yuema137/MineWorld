# Step 29 — S29: every town's train station, and its signature landmark

**Role:** step document for **S29**, a new step that the primary session adds to [`overall.md`](overall.md).
It records the operator requirement. It audits what exists: places, far field, travel, clients, packs and the
agent view. It then designs:

- the World Pack contract for a town's station and landmark;
- the station's honest state before trains exist;
- the 3D building after San Diego's Santa Fe Depot, and its 2D counterpart;
- the reasons people have to go there;
- the checks.

It ends with a PR split, the invariants, the risks and the questions only the operator can answer.

**Effort:** `mvp0`. Timing is ruled: **after Milestone F, before the first public release** (S23).
**Parent:** [`overall.md`](overall.md).

**Lifecycle:** `APPROVED 2026-10-15 (step plan; PR designs to follow)`. The operator answered
Q-S29-1…10 directly in the primary session on 2026-10-15, every one as recommended (§13.1).
Amended 2026-10-18 by two S33 rulings (§14.9): §4.3, each with a dated note in place.

- Approval fixes this step's scope, invariants (§10) and step acceptance (§11.3). Progress, evidence,
  findings and bounded corrections stay writable.
- Each PR is detailed to the commit in its own PR design and frozen in turn (`CLAUDE.md` §3.1). Only a PR
  design carries `DESIGN FROZEN`.
- **Start condition (operator, 2026-10-15):** nothing starts until Milestone F's checklist has been handed
  to the operator.
- Nothing in this file authorizes implementation, an image-model call or a merge.
- Superseded lifecycle: `STEP DESIGN (DRAFT)`, PR #218 revision 1.

**Author:** S29 design session (Opus), 2026-10-15.

- Worktree: `/Users/yuema137/mineworld-worktrees/design-s29`.
- Branch: `docs/s29-station`.
- Audit base: `origin/main @ 2d07c6c0` (RL-e0 frozen design merged, #215).

**Ids.** Assigned 2026-10-15, after checking that each was still free on `origin/main`: `ARC-105`,
`ARC-106`, `ARC-107` and `ARC-108` (they replace the placeholders ARC-S29-a…d), and `DEP-60` (it replaces
DEP-S29-a). The five entries are in `docs/DECISIONS.md` (§14.5).

Local ids in this file:

| Prefix | Meaning |
| --- | --- |
| `SD-S29-n` | step decisions |
| `A-n` | audit anchors |
| `F-S29-n` | findings |
| `AC-S29-n` | step acceptance |
| `X-n` | planted mutations |
| `I-S29-n` | invariants |
| `R-S29-n` | risks |
| `Q-S29-n` | operator questions |

**Parallel-safety.** This file is the only file this session writes. The edits it needs elsewhere are
**proposed** in §14 and made by the PRs that own those files:

- `overall.md`;
- `docs/MODULE_SPEC.md`, `docs/CORE_CONCEPTS.md`, `docs/VISUAL_SLICE.md`, `docs/ART_DIRECTION.md`,
  `docs/DECISIONS.md`;
- the RL-e0 design;
- step-21.

---

## 0. Binding inputs

### 0.1 Operator requirements, 2026-10-15 (verbatim)

> "而且一定要有火车站，还要有town的特色！作为一个town的特色建筑"

> "再小的世界，再小的城镇，都要有远景…只有这样才能做到沉浸式体验"

In English, for the specification documents (`CLAUDE.md` §2.1 rule 5: English is authoritative, and the
Chinese is quoted as the source):

> *"And there must be a train station, and it must carry the town's character! As the town's signature
> building."*
>
> *"However small the world, however small the town, it must have a far field … only that makes the
> experience immersive."*

### 0.2 Rulings already made (relayed with this task, 2026-10-15)

| Ruling | Effect here |
| --- | --- |
| **Timing:** after Milestone F, before the first public release | S29 must not delay F's checklist. It must land before S23's release candidate. Its PRs start only after F is handed to the operator (§11.2) |
| **The landmark is modelled on San Diego's Santa Fe Depot** (1915, Mission Revival, twin tiled towers). **The station itself is the landmark** | The Market Town and Social Café station *is* the landmark: one building, one Place group, two roles (SD-S29-3). RL-e0's Q-FF-8 ruled the same |
| RL-e0 (frozen) reserves `skyline.slots.station` at 277°, 790 m and leaves `landmark` `unplaced`. The station PR adds a `filled` state "(an additive change, not a format bump)" | S29 adds `filled` (§5.4) |

### 0.3 Standing rules that bind this step

| Rule | Consequence |
| --- | --- |
| Framework, not demo | A station is a **World Pack contract** that any town meets with its own content. Market Town's depot is one default composition, never a hard-coded scene (§3) |
| Realistic defaults (cite the reference) | Every building dimension cites a primary source: HABS CA-1965 or the NRHP nomination 72000248. An approximate value says so and names who replaces it (§6.2). The one deliberate departure from the real geography, the distance (Q-S29-1), is stated as such |
| NPCs to the same standard as the hero | Anyone drawn in the station is a cast person (S26), never a placeholder figure (§7.3) |
| All platforms | Every rendered check runs on the Linux, macOS and Windows nightly runners, as RL-e0's does (§9) |
| Agents are first-class players | The agent (MCP) view shows the station as a Place with its role, through the same Observation as everyone else (§9.4) |
| Local load: CI is the gate (R-LOAD-1) | Local timings are INCONCLUSIVE. Counts are recorded locally, and timings come from the operator or CI (§6.6) |
| Design on Opus, implement on Haiku | The PR split marks the implementer for each PR (§11) |
| One art style, one precision, one method (S26 §7.4a, operator 2026-10-15) | Read across to buildings: the depot uses the slice's material library, scale contract, palette and lighting. It may not have a bespoke look or a second toolchain (§6.1) |

---

## 1. The requirement, restated as checkable outcomes

1. **Every town has a train station.** The station is a real Place group the player can walk into. It has:
   - a concourse (the waiting hall);
   - a ticket window;
   - a platform with an anchor.

   It uses S21's term `stop` for the platform.
2. **Every town has a signature landmark.** In Market Town and Social Café the landmark is the station,
   built after the Santa Fe Depot.
3. **A town without either is refused by name.**
4. **Before trains exist, nothing pretends.**
   - The board says "No service yet".
   - The ticket window is shut.
   - Nobody waits for a train that cannot come.
5. **The station sits in the view and on foot.**
   - It closes the street's westward view.
   - It stands where the far field reserved its slot (bearing), at street level.
   - It is reachable on foot from spawn.
6. **Both reference clients draw it.** Each draws it in its own pipeline and art style.
7. **People have a reason to go there**, without a language model.
8. **The checks hold it in place.** They are rendered regression shots, validation tests and the agent view.

---

## 2. Audit (`origin/main @ 2d07c6c0`, 2026-10-15)

### 2.1 Anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | No world has a station. `social-cafe` and `market-town` have six places each (apartments, cafe, park, store, street, workplace). `lakeside` has seven (bakery, boathouse, cottages, lane, pier, shore, square). `bodies-yard` is a test yard | `worlds/*/world.yaml` | Three worlds need a station group. The fourth needs an explicit "not a town" |
| A-2 | A place file has only `tags`, `note`, `passages` and the owned sections (`body:` from 12d). Tags are an open vocabulary and reach clients in an Observation. Passages disclose `to_tags` | `docs/MODULE_SPEC.md` §4.1 (`places/<key>.yaml`), "What a place discloses" (`ARC-82`) | A role tag on a plain Place is enough to disclose "this is a station" with no kernel change. The loader validates tags as strings only |
| A-3 | Places have no rooms of their own. Hierarchy is a Relation, not a field (`CORE_CONCEPTS.md` §6.2 rule 3), and S21 gives `contains` to its `atlas` pack. Interior granularity is a simulation choice (§6) | `docs/CORE_CONCEPTS.md` §§6, 6.2 | Before `atlas`, the station's rooms are **sibling Places joined by passages**. They are tied together by a shared tag, not by a parent field (SD-S29-2) |
| A-4 | Place ids are allocated in **key order** after nothing, then people, items and organizations (`world.yaml` comments; S21 rule 2) | `worlds/social-cafe/world.yaml` `places:` comment | A new key such as `station` sorts between `park` and `store`, so ids shift. Every replay digest and golden fixture of the three towns is re-baselined once (Q-S29-7) |
| A-5 | `overall.md` §5 says: "Both towns' replay digests are re-baselined **once**, in S15's 12d" | `overall.md` "Digests" | S29 needs a second, recorded re-baseline, which is an operator decision (Q-S29-7) |
| A-6 | S21 (MVP-1, step design) defines a **stop** as "a Place a line serves (a platform, a bus stop, a car park)". `StopInfo` sits "on a stop Place: lines served, the platform's door anchors" and is transport's. `board` requires being within 3 000 mm of a door anchor. Tickets are inventory item kinds sold "at a station shop with economy's existing `buy`" (QRT-10, accepted). The carriage is a walkable Place with no passages | `.structured-coding/plans/mvp1/step-21-regions-travel.md` §§4, 5.3, 5.4, 5.7, 11.1 | S29 uses `stop` for the platform and leaves `StopInfo` to transport. The ticket window must be **its own Place**, because economy keeps one shop per place and the hall may hold the newsstand (SD-S29-4) |
| A-7 | Economy discloses a place's `shop` listing (operator, prices, stock) to whoever perceives the place. `buy` works only in a Place carrying a `Shop`. A shop is declared on an organization: `economy.shop.at: <place>` | `systems/economy/src/{component.rs:96-130, system.rs:236-262}`; `worlds/market-town/organizations/corner-store.yaml` | A ticket window is open **if and only if** its place discloses a shop. This gives the client a rule to follow, not a hard-coded state (SD-S29-5) |
| A-8 | `newspaper` is an item kind in Market Town, and **no shop sells it** | `worlds/market-town/items/newspaper.yaml`; `organizations/*.yaml` | A station newsstand gives an existing item a seller (§7.2) |
| A-9 | Routines are `{ from, place, label }`. A label is 1–48 of `a-z0-9-`. Every boundary lies within 05:00–23:00, a fixture dependency of the restart tests | `systems/schedule/src/label.rs`; `worlds/market-town/people/bob.yaml` | Station visits are routine segments. No schedule change is needed (§7.1) |
| A-10 | RL-e0 (frozen) has these properties: <ul><li>`far_field/<world>.yaml` is required per Presentation Pack.</li><li>A `Slot` is `unplaced` or `reserved`; a reserved slot's `distance_m` must exceed `boundary.reach_m`.</li><li>`FAR_FIELD_SLOT` refuses any other state.</li><li>Social Café's 3D reach is 60 m, the boundary is natural and the ground beyond is a `drop`.</li><li>The station is reserved at 277°, 790 m with a 120 m footprint.</li><li>Lakeside's slots are both `unplaced`.</li><li>`ff_6_west_street_end` shows the bay and Point Loma beyond the parapet.</li></ul> | `pr-rl-e0-far-field-mandatory.md` §§5.1.2, 5.1.6, 7, App. B | S29 adds a `filled` state, lets a filled slot lie inside the boundary, grows `reach_m`, and re-shoots `ff_6`. This is an intended far-field change under RL-e0's V-gate rule (§5.4, §9.1) |
| A-11 | The slice street runs x = −34 … +34 m (scene frame: +x east, −z north). At the west end a stone parapet stands at `X_MIN − 1`, with a setts slab to x ≈ −44. Spawn is (−11.5, 0.45, −5.55), facing east | `clients/3d-spike/scripts/slice/street.gd:1-40, 122-144`; `slice_world.gd:22-28` | The west end is where a station can close the street's view at street level, about 23 m from spawn (§6.3) |
| A-12 | The 3D client builds every building **procedurally in GDScript**: `Build.box`, `Mats.pbr` over Poly Haven CC0 1k textures, and `Props` over Poly Haven CC0 glTF. Each building is one file, for example `cafe.gd` (456 lines), `cafe_interior.gd` (631), `shop_interior.gd` (392). The textures already include `painted_plaster_wall`, `clay_roof_tiles_02`, `wood_floor_worn`, `brown_planks_09`, `long_white_tiles` and `sandstone_blocks_05` | `clients/3d-spike/scripts/slice/`; `clients/3d-spike/ASSETS.md`; `assets/textures/` | The depot is built the same way. The Mission Revival palette of stucco, clay tile and dark timber is already in the library. **Reuse comes before any new asset** (§6.4) |
| A-13 | The slice binds volumes to Place ids (`SliceWorld.CAFE_PLACE`, `SliceLink.PLACE_KEY`). S21 A-11 says the 3D slice hard-codes `cafe` and `street`. 16c binds every doorway | `slice_world.gd:17-23`; step-21 A-11; step-15 §20 | The station's 3D binding comes after 16c and binds by **role tag**, not by key (SD-S29-8) |
| A-14 | Budgets: <ul><li>p95 ≤ 16.7 ms and video memory ≤ 2 048 MB are binding; draw calls and primitives are advisory (RL-b R-2).</li><li>RL-e0 caps its own increment at ≤ +1.0 ms, ≤ +24 draws, ≤ +100 k primitives and ≤ +32 MB per view.</li><li>The `3d.pck` hard limit is 250 MB, and S26's cast already forecasts ≈ 245–260 MB under O-1 (ruled: O-1 + O-3 + O-5, else O-4 raises the limit to ≈ 350 MB).</li></ul> | `pr-rl-b-3d-budget.md` §0; `pr-rl-e0…` §8; `step-26-cast.md` §10.3 | The station's byte budget is tight. **Its target is ≤ 6 MB of new pack data** (§6.6) |
| A-15 | 2D façades are bound by tag (`facade:<tag>` in `presentation/mineworld-default/2D/assets/asset_bindings.yaml`). The building sprites (`sib_building_*`) are this project's own generated images: OpenAI image edits conditioned on the pack's references, normalized to 64 px per metre, each with a provenance sidecar. Interiors come from an `interiors` spec chosen by tag | `presentation/mineworld-default/2D/art/PROVENANCE.md`; `clients/2d/scripts/scene/places.gd:199-310, 427-437` | The 2D station is a `facade:station` binding to a new `sib_building_station` sprite from the same pipeline, plus an `interiors` entry (§8) |
| A-16 | S27 (agent players): the View's `me.place` and `exits[].leads_to` are `PlaceLine`s. `PerceivedEntity` carries `tags`. The passage disclosure carries `to_tags` | `step-27-agent-players.md` §5.3; `pr-s27c-player-library.md` C-6; `MODULE_SPEC.md` §4.1 | The station's role reaches an agent through tags it already receives. S29 verifies this end to end and adds at most a rendering of the role tags in `PlaceLine` (§9.4) |
| A-17 | `VISUAL_SLICE.md` §4 lists as **out of scope** "a third enterable building … and any building the player can see but never reach". §4 terminates the street "at both ends" | `docs/VISUAL_SLICE.md:125-150` | The depot is a third enterable building, so the specification is amended first (§14). A depot seen at 790 m but unreachable would break the second clause (F-S29-1) |
| A-18 | `world.yaml` `version`/`license` are "optional to the loader; required by `packs validate`" (`ARC-53`) | `MODULE_SPEC.md` §4.1 | This is the precedent for an **authoring-gate requirement** that does not break every test fixture world (SD-S29-1) |

### 2.2 Facts about the real building (read 2026-10-15)

| Fact | Value | Source |
| --- | --- | --- |
| Name, address | Santa Fe Depot (Santa Fe Railroad Station), 1050 Kettner Blvd, San Diego | NRHP 72000248; HABS CA-1965 ([LoC](https://www.loc.gov/item/ca0609)) |
| Built, architects, style | 1914–1915 (opened 8 March 1915); Bakewell and Brown, San Francisco; Mission Revival / Spanish Colonial Revival | [Wikipedia](https://en.wikipedia.org/wiki/Santa_Fe_Depot_(San_Diego)); HABS CA-1965 |
| Coordinates | 32°43′00″ N 117°10′10″ W (32.7167, −117.1694), which agrees with RL-e0 A-25 | Wikipedia (infobox) |
| Front | A main arch flanked by **two bell towers**, each topped by a **tile-covered dome**. The tiles are blue, green, yellow and white in a zigzag, and the Santa Fe cross is on all four sides. The building terminates the view down Broadway | Wikipedia; [Gaslamp Foundation](https://gaslampfoundation.org/?p=9876); [SOHO tile tour](https://www.sohosandiego.org/tours/tile/santafedepot.htm) |
| Main waiting room | **170 × 55 ft (51.8 × 16.8 m)** | Wikipedia; Gaslamp Foundation |
| Interior | <ul><li>Natural redwood beam ceilings.</li><li>A glazed tile wainscot **8½ ft (2.59 m)** high in green, yellow, blue, white and black, with Moorish patterns and a ziggurat frieze (California China Products Co.).</li><li>Original oak benches.</li></ul> | Wikipedia; Gaslamp Foundation |
| Arcade / concourse | Arcade 27 × 650 ft (8.2 × 198 m). Covered concourse ≈ 650 × 106 ft | Wikipedia |
| Tracks | 6 tracks, 2 island and 2 side platforms | Wikipedia |
| Tower height | **Not found in the sources read.** HABS CA-1965 has 15 photographs and 11 data pages, and the measurement must be taken from them | HABS CA-1965 (public domain, a federal work) |

### 2.3 Findings

- **F-S29-1. The operator's two placement words cannot both hold literally.** "At the far-field slot" means
  790 m. "Reachable on foot from spawn, at street level" means inside the walkable area, whose radius is
  60 m (A-10). A depot visible at 790 m but not reachable is exactly the building `VISUAL_SLICE.md` §4
  forbids (A-17). The step therefore keeps the slot's **bearing** and brings the building to the street's
  end. This is the main operator question (Q-S29-1).
- **F-S29-2. A station needs at least three Places today.** Economy keeps one shop per place (A-7), and
  QRT-10 sells tickets at a station shop. If the ticket window and the newsstand shared the hall, the hall
  could never hold both. The platform must be its own Place, because S21's `board` and `alight` resolve
  against the stop Place (A-6).
- **F-S29-3. "No service yet" must come from the world, not from a constant.** S21's `StopInfo` sits on
  the stop. An observer in the hall perceives only the hall (S21 A-24), so a board in the hall could never
  read the platform's `StopInfo`. S21 must disclose departures to observers in station places. That is a
  non-preclusion item for S21 (§14.6), and before S21 the board reads "the absence of any departures
  record".
- **F-S29-4. A second digest re-baseline is unavoidable** for any added place, person, organization or
  routine (A-4, A-5).
- **F-S29-5. The 3D package has no room to spare** (A-14). The depot has to be built mostly from materials
  the slice already ships, plus shader-generated tile patterns.

---

## 3. Terms

No new primitive is introduced. These terms are added to `CORE_CONCEPTS.md` §6 as **role tags** with fixed
meanings (§14.1). They are not synonyms of existing terms.

```text
town            a World Pack whose world.yaml declares `settlement: { kind: town, … }` (SD-S29-1)
station         the Place group of a town where its railway meets it: every Place tagged `station`
                that is joined, through passages between `station`-tagged Places, to the hall the
                world declares
station hall    the declared entrance Place of the station group (the concourse, the waiting room)
stop            S21's term, unchanged: a Place a line serves. In a station, the platform Place,
                tagged `stop` and `platform`
ticket office   the station Place, tagged `ticket-office`, where tickets are sold once a shop is there
landmark        the Place a town declares as its signature building, tagged `landmark`; it may be the
                station hall
platform anchor the origin of a `stop` Place: a point on the platform edge, mid-length (SD-S29-6)
```

"Station" here is a tag-defined group, not a new Entity type and not a System. When S21's `transport`
pack exists, it attaches `StopInfo` to the same `stop` Place. When `atlas` exists, it may seed `contains`
from the same group. Nothing here needs to change for either.

---

## 4. The World Pack contract (`ARC-105`)

### 4.1 How a world declares that it is a town (SD-S29-1)

`world.yaml` gains one framework key, `settlement:`, owned by the World Pack loader (`worldpack/`). It is
not owned by any System and not by the kernel.

```yaml
settlement:
  kind: town            # town | none
  station: station      # kind town: required. A key of `places`: the station hall
  landmark: station     # kind town: required. A key of `places`; may equal `station`
```

```yaml
settlement:
  kind: none            # this world is not a settlement (a test yard, a lone cabin, a void arena)
```

**Required at authoring, validated whenever present.** The key follows the `ARC-53` precedent (A-18):

- **Optional to the loader.** A fixture world written by a test still loads.
- **Required by `mineworld validate` and `mineworld packs validate`.** A World Pack without it is refused
  `SETTLEMENT_UNDECLARED`, naming the file.
- **Validated in full whenever present**, by the loader and therefore by `run`, `server` and `validate`
  alike.

An undeclared world is therefore refused by name at the gate every author runs, and CI runs that gate over
every `worlds/*` directory (`AC-S29-1`).

*Alternatives rejected:*

| Option | Why rejected |
| --- | --- |
| (a) Infer "town" from a tag on some place | A world that forgot the tag would silently stop being a town, which is the failure the operator named |
| (b) Make every world a town, with no opt-out | `bodies-yard` and the test yards are not settlements, so a station there would be fake content |
| (c) Make the key required by the loader | Every inline test fixture world would change, which costs churn and adds no safety beyond (b)'s gate. Raised as part of Q-S29-2 |

**Is a village a town?** Yes. "However small the town" (§0.1) puts Lakeside in. A world that is
genuinely not a settlement declares `kind: none`. `kind` has two values; any finer classification (city,
village, hamlet) would be a later, additive value and changes nothing here.

### 4.2 What a town must contain (SD-S29-2)

With `kind: town`, the loader checks the following. Each failure is a named refusal with the file and the
key.

| Code | Refused when |
| --- | --- |
| `SETTLEMENT_UNDECLARED` | (`validate` / `packs validate` only) `world.yaml` has no `settlement:` |
| `SETTLEMENT_MALFORMED` | <ul><li>An unknown key.</li><li>A `kind` other than `town` or `none`.</li><li>`station` or `landmark` given with `kind: none`.</li><li>Either one missing with `kind: town`.</li></ul> |
| `TOWN_STATION_UNKNOWN` / `TOWN_LANDMARK_UNKNOWN` | the key is not one of `places` |
| `TOWN_STATION_UNTAGGED` | the station hall lacks the tags `station` and `public` |
| `TOWN_LANDMARK_UNTAGGED` | the landmark place lacks the tag `landmark` |
| `TOWN_STATION_NO_STOP` | the station group contains no Place tagged both `stop` and `platform` |
| `TOWN_STATION_NO_TICKET_OFFICE` | the station group contains no Place tagged `ticket-office` |
| `TOWN_STATION_SCATTERED` | a Place tagged `station` is not in the declared hall's group. Two stations need S21's regions, not two groups in one town |
| `TOWN_STATION_UNREACHABLE` | (`movement` enabled) some seat's starting place has no passage chain to the hall. This makes "reachable on foot from spawn" a load-time fact |
| `TOWN_LANDMARK_UNREACHABLE` | (`movement` enabled; added 2026-10-15, Q-S29a-2) some seat's starting place has no passage chain to the landmark, when the landmark is not the hall |
| ~~`TOWN_PLATFORM_ANCHOR`~~ | ~~(`bodies` enabled) the stop's origin (0, 0) does not lie on **exactly one** edge of its `body.floor` (SD-S29-6)~~ **Moved 2026-10-15 (Q-S29a-1):** not a loader check, because `body:` is opaque to the loader (`ARC-31`). The rule stays the meaning of a stop's origin (`CORE_CONCEPTS` §6.4); S29-a's C6 holds it for the reference towns with a content test over `bodies`' `place-shaped` genesis fact; S21's RT-c refuses an off-edge anchor when it attaches `StopInfo` |

The group is found by breadth-first search over passages between `station`-tagged Places, starting at the
hall. This is the same graph `movement` records. The loader reads the pack's own passage list and needs no
System. Without `movement`, a world has no passages: the group is then every `station`-tagged Place, and
the reachability and scattering checks are skipped. The skip is stated in the refusal table's notes, not
hidden.

**No kernel change, no contract change, no System change** (`I-S29-1`). The key is loader data. The tags
are strings the loader already accepts. The checks read the pack's own files.

### 4.3 The station group in the reference worlds (SD-S29-3, SD-S29-4)

| Place key | Tags | Passages | Role |
| --- | --- | --- | --- |
| `station` | `station`, `public`, `landmark` (Market Town and Social Café) | `street` (the forecourt doorway under the arch); `station-tickets`; `station-platform` | hall, waiting room, meeting place, newsstand (Market Town) |
| `station-tickets` | `station`, `ticket-office`, `public` | `station` (the counter bay off the hall) | the ticket window. **Shut**: no shop is declared there before S21's `fares` (§5.2) |
| `station-platform` | `station`, `stop`, `platform`, `public`, `outdoor` | `station` (the trackside doors) | S21's stop. Its origin is the platform anchor |

- **Keys.** These keys sort as `station` < `station-platform` < `station-tickets`. Their place in the
  key order (A-4) is the re-baseline of Q-S29-7.
- **Bodies.** Each place gets a `body:` section (12d's format) at real dimensions: the hall 51 800 ×
  16 800 mm (§2.2), the counter bay about 6 000 × 3 000 mm, the platform about 60 000 × 6 000 mm. All lie
  within bodies' ±100 m bound (S21 A-12).
  *(Dated note, 2026-10-18, operator ruling in the primary session, recorded in `pr-s33b-lakeside-bodies.md`
  F-S33b-5 and §11; applied by S29-a revision 3, §14.9.)* The ticket room is **at least 4.6 × 4.0 m**, sized so
  that `bodies`' capacity rule (4·(population − 1) + 1 points of the 650 mm lattice, with the counter in place)
  holds the town's people. "About 6 000 × 3 000 mm" above is superseded and kept for the record: it admits 27
  points where the towns' 12 people need 45. S29-a revision 3 writes the towns' bay 6 000 × 4 600 mm, pending
  its Q-S29a-5.
  *(Dated note, 2026-10-19, S29-a revision 4, §14.10.)* The bay lies south of the hall, its counter on the south
  wall; the platform runs north–south along the hall's west wall, about 11 000 × 60 000 mm (the real 8.2 m
  arcade), pending S29-a's Q-S29a-8.
- **The street.** Its `body.floor` grows west to take in the forecourt (§6.3).

**Lakeside** (Q-S29-3) has a **small halt**: the same three places at village scale.
*(Dated note, 2026-10-15, S33 approval; non-preclusion item for S29-a.)* S33 §4.0 asks S29-a to place the
halt at the **east end of `lane`**, with a north–south platform ending at a buffer stop near the shore (a
terminus), unless S29-a's audit shows a reason against. S33-b adds `body:` sections to Lakeside's other
places and depends on the halt's position being fixed here. The hall is a waiting
room of about 8 × 5 m, the ticket office a window bay, and the platform about 40 × 4 m. They are joined to
`lane`. Its landmark is a separate place, `pier` (ruled, Q-S29-3), tagged `landmark`.
*(Dated note, 2026-10-18, operator ruling Q-S33a-1 in the primary session, recorded in
`pr-s33a-setting-far-field.md` §14; applied by S29-a revision 3, §14.9.)* Ruled: the halt stands **at the east
end of `lane`**, with a **north–south platform whose end is against the shore**. The proposal above is
therefore no longer conditional. Its ticket window bay is at least 4.6 × 4.0 m (the capacity ruling, above;
4.6 × 4.6 m in S29-a revision 3, pending its Q-S29a-6). Which edge of the platform the track runs along is S29-a's
Q-S29a-7.

**`bodies-yard`** declares `settlement: { kind: none }`.

### 4.4 Specification text (summary; the full text is written by S29-a's first commit)

- `MODULE_SPEC.md` §4.1: the `settlement:` key, its two shapes, and the "required by validate" note.
- `MODULE_SPEC.md` §4.3 (new): "Towns: station and landmark", with the refusal table above.
- `CORE_CONCEPTS.md` §6: the role tags of §3, with the rule "a role tag is authored content that the
  framework validates. It grants no behaviour; behaviour comes only from Systems". The proposed text is in
  §14.1.

---

## 5. Before trains exist: honest by construction (`ARC-108`)

### 5.1 The principle (SD-S29-5)

**A station fixture shows the state the world discloses, and nothing else.** Each of the three fixtures
has a rule that a client evaluates from the Observation:

| Fixture | Shown open or live when | Shown otherwise |
| --- | --- | --- |
| **Departures board** (hall, and a smaller one on the platform) | the observer's place discloses a `departures` record (S21, §14.6): the board lists its lines and next times | "No service yet", localized (`station.no_service`) |
| **Ticket window** (`station-tickets`) | that place discloses a `shop` listing at least one kind tagged `ticket` | the shutter is down with a "Closed" plate (`station.window_closed`); no clerk figure |
| **Newsstand** (in the hall) | the hall discloses a `shop` (Market Town, §7.2) | the kiosk shutter is down (Social Café, Lakeside) |

The rule is the same in both clients and in the agent view. It is written once in
`clients/shared/station/station_state.gd`, a pure function from the disclosed components to
`{board: none | lines[…], window: open | shut, kiosk: open | shut}`. This follows the RL-e0 shared-module
pattern. The function never reaches an intent; the client-rules scan holds it to that.

### 5.2 What "no fake behaviour" excludes, explicitly

- **No invented timetable, no departure times, no "delayed" text.** The board shows nothing that the world
  did not disclose.
- **No `buy ticket` offer.** No shop exists in `station-tickets` until S21's `fares` composes one, so
  economy offers nothing there. An agent standing at the window sees **no offers**, which is the truth.
- **No routine label of `wait-for-train` or similar.** Nobody waits for a train before `transport` is
  installed (§7.1).
- **No train in the 3D or 2D scene.**
  - The track and platform edge are drawn; a train is not.
  - The S21 carriage is a Place and arrives with RT-c/RT-g, not as decoration.
  - A rendered train before then would be "a building the player can see but never reach" (A-17), on
    wheels.

### 5.3 What changes when S21 lands (non-preclusion, no S29 rework)

- RT-c's `transport` attaches `StopInfo` to the existing `station-platform`. Its door anchors are
  positions in the platform's frame, along the anchor edge (SD-S29-6).
- RT-c discloses `departures` to observers in `station`-tagged places (§14.6). The board then lists lines,
  with no client change.
- RT-c's `fares` (QRT-10) adds a shop at `station-tickets`, run by the line's operator organization, listing
  `ticket-<line>` kinds. The window opens with no client change.

---

## 6. 3D: the depot (`ARC-106`)

### 6.1 Art direction (SD-S29-7)

- **One art style.** The depot uses the slice's material library (`Mats.pbr` over the Poly Haven CC0
  set), `SlicePalette`, the scale contract (`VISUAL_SLICE.md` §3), the grade and the light. The town has
  one toolchain: procedural GDScript geometry. No second one is introduced: no DCC-modelled hero mesh, no
  photogrammetry, no generated 3D model (a Meshy building would be a second method; S26 §7.4a read across).
- **One precision.**
  - Every element follows §3's tolerances, and a measurement verifies it (`--slice-measure` gains the
    station's rows).
  - Doors are 0.85–1.00 m wide and 2.05–2.20 m tall.
  - Benches have a seat height of 0.42–0.47 m.
  - The counter is 1.00–1.10 m high.
  - The tile wainscot is 2.59 m high (§2.2).
- **The sanctioned exception.** The depot is the town's **one** Mission Revival building beside a street
  whose architecture is governed by the lakeside references (QRL-2/3). A landmark is meant to stand out by
  form, not by rendering. The materials it needs are already in the street: `painted_plaster_wall` for
  stucco, `clay_roof_tiles_02` for roofs, and the wood sets for the beams and benches. The operator confirms
  this in Q-S29-5.
- **References.** These are the HABS CA-1965 photographs and data pages. They are federal works in the
  public domain, so they pass DEP-8's relicensing test. They are committed under
  `presentation/mineworld-default/3D/references/station/`, with a provenance file (URL, LoC id, SHA-256).
  CC BY-SA photographs (Wikimedia Commons) fail DEP-8 and are **linked, never committed** (Q-S29-8).

### 6.2 The building, element by element (real dimensions; approximations named)

| Element | Specification | Source / status |
| --- | --- | --- |
| Waiting hall | 51.8 × 16.8 m inside, long axis north–south (scene z), the east façade facing the street | §2.2 (real) |
| Hall height | 10–12 m to the ridge, under exposed redwood-coloured beams (`brown_planks_09`, tinted) | **approximate**, measured from HABS photos in S29-d's first commit |
| Main arch | Centred on the street axis. Clear opening ≈ 6 m wide and ≈ 9 m tall, with a recessed entrance and real door leaves at §3.2 dimensions | approximate; HABS |
| Twin towers | Square shafts flanking the arch, stucco, with belfry openings and **tile-covered domes**. Height to the top of the dome is **to be measured** (HABS). Until then, 30 m is a placeholder that the rendered check does not depend on | placeholder, replaced in S29-d C1 |
| Dome tiles | A **shader-generated** zigzag of blue, green, yellow and white glazed tile (`shaders/glazed_tile.gdshader`, our own, MIT). It adds no bytes. The cross emblem is a decal from an SVG we author | own work |
| Wainscot | 2.59 m of glazed tile with a frieze, the **same shader** with a geometric pattern | §2.2 (real height); own work |
| Roofs | `clay_roof_tiles_02` (already shipped) | reuse |
| Walls | `painted_plaster_wall` (already shipped), warm white | reuse |
| Benches | Long oak benches, procedural from the wood sets, in rows across the hall; seat at 0.45 m | §2.2 (oak benches); §3.4 |
| Ticket window bay | A carved timber counter screen with two grilles. The shutter is drawn per §5.1 | approximate |
| Newsstand | A timber kiosk in the hall with a shutter per §5.1 | design content |
| Boards | A hall board and a platform board; their text per §5.1, through `Label3D` and the localized words | design content |
| Clock | A wall clock that shows **the world clock's disclosed time** (calendar, `ARC-67`). It shows no time when the world has no calendar, rather than the host's wall time | honest by §5.1's rule |
| Arcade and platform | An arcade 8.2 m deep (real) along the west side. Platform 60 m long (compressed from 198 m), platform edge 0.9 m above the rail head, two tracks, a fence and gates at both ends | arcade real; the length is compressed (§6.3) |

### 6.3 Where it stands (SD-S29-8; Q-S29-1)

**Ruled (Q-S29-1, 2026-10-15): true bearing family, compressed distance, real building.**

- **Placement.** The forecourt replaces the west parapet (`street.gd` `_surrounds`). The street opens into
  a setts forecourt about 25 m deep. The arch stands on the street's axis, with the east façade at about
  x = −62 m.
  - This is bearing 270° from the origin.
  - The real depot's centroid is at 277° (A-10). The real relationship, that Broadway's axis ends at the
    depot's front, is what the scene keeps.
  - The hall's centroid falls at about 277° once the hall's 16.8 m depth and its southward offset are
    included. S29-d C1 records the measured bearing.
- **Distance.** The building is about 790 m from the origin in reality and about 62–79 m in the scene. This
  is stated in the far field's `source` as `"Santa Fe Depot, compressed from 790 m to the street's end so
  that it is reachable on foot (Q-S29-1)"`.
- **The street view.** Seen from spawn turning west, and from the east end looking down the street, the
  towers close the westward view: the "street view closed by a landmark" of the requirement.
- **Spawn and walk.** Spawn stays at x = −11.5, about 50 m from the arch, about 37 s at 1.34 m/s.
- **The bay.** The bay (170°–300°, 1 km) stays visible past the hall's south end at 200°–250°. The ground
  beyond the platform stays a `drop` toward the water.
- **The far field.** `boundary.reach_m` for Social Café grows from 60 to **100** m to contain the platform
  and its fence. The massing (`from_m` 250) and the silhouettes are untouched.

| Option | What | Verdict |
| --- | --- | --- |
| **P-1. At the street's end (ruled, Q-S29-1)** | Described above | It meets "reachable from spawn", "street level" and "closes a street view". It keeps the slot's bearing. It costs one honest compression, recorded with its source |
| P-2. At the real 790 m | A walkable approach of ≈ 700 m (about 9 min at 1.34 m/s) | It needs ≈ 700 m of frontage, or "buildings the player can see but never reach" along the way (A-17). The slice's scope and the perf budget would both be re-planned. This is S21's region scale, not a street |
| P-3. Silhouette at 790 m plus a separate near station | Two buildings for one role | Rejected. One Place group would be drawn twice, and the far one would be an unreachable landmark |

### 6.4 Reuse, in both directions (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Building geometry | **Build ours**, procedurally, in the slice's own builder (`Build`, `Mats`, `SliceTerrace` patterns) | This is the town's one method (§6.1). A third-party "Spanish mission" model would be a second style and fails DEP-8's per-asset check more often than not |
| Stucco, clay tile, timber, setts | **Reuse** the shipped Poly Haven CC0 sets | The bytes are already paid for (A-12) |
| Glazed tile patterns (domes, wainscot) | **Our own shader**, `glazed_tile.gdshader` (≈ 80 lines). One parameter set per pattern | Zero bytes. No CC0 azulejo set matches the depot's zigzag, and a photo texture of the real tiles is someone's copyright |
| Benches, kiosk, counter, boards, clock | **Build ours**, procedurally (the café's furniture pattern) | Same reason as the geometry. Poly Haven's `painted_wooden_bench` is the wrong type (a park bench) |
| Small props (a bin, a potted palm, a lamp) | **Reuse** the shipped Poly Haven props | Already shipped |
| References | **HABS CA-1965** (public domain) | DEP-8 |
| A new CC0 texture | **At most two**, only if a reviewed frame shows the shipped set cannot read as the material: Poly Haven 1k, ≤ 3 MB each, one DEP-8 row each | The byte budget (A-14) |

No new dependency is adopted, so no `DEP` record is needed unless one is triggered. `DEP-60` is
reserved for that case and is withdrawn at freeze if unused.

### 6.5 Interior and threshold

- **Structure.** The hall follows `VISUAL_SLICE.md` §6.1's structure rules: four walls with thickness, a
  walked floor, a treated ceiling, glazing shared with the exterior, and the arch plus the trackside doors
  as openings.
- **Fit-out.** It reads at §6.2's three object sizes:
  - room scale: beams, wainscot, boards;
  - furniture scale: benches, kiosk, counter;
  - hand scale: the newspapers on the kiosk when it is open, timetable leaflets in a rack, a lost umbrella
    on a bench.
- **Threshold light.** It is measured at the arch as in §7.1, at four points, and reported separately.
- **Bodies.**
  - The 12d `body:` solids, at most 64 (S21 A-12), cover the bench rows, the kiosk, the counter and the
    columns.
  - The bench rows leave aisles of at least 1.2 m.
  - The routes are checked by `bodies`' route planner in S29-a's integration test, with no stop caused
    by geometry: NW-3's bound of ≤ 10 % holds on the station's walks.

### 6.6 Performance and size (against RL-b, RL-b2 and RL-e0)

**Increment caps in every existing RL-b M-1 view** in which the depot is visible (the street views; the
west end): ≤ +0.8 ms p95, ≤ +40 draw calls, ≤ +150 000 primitives and ≤ +48 MB video memory.

**New views,** added to `SlicePerf`'s list: `station forecourt`, `station hall wide`, `station platform`.
The **absolute** gates apply: p95 ≤ 16.7 ms and video memory ≤ 2 048 MB. Draw calls and primitives are
reported, as advisory counts.

**Levers built in from the start:**

- the hall's geometry goes through `SliceBatch` (24 m cells);
- the tile shader is one material for all tile surfaces;
- the bench rows are one `MultiMesh`;
- the towers and the hall are occluders for the frontage behind them (`SliceOccluders`);
- the interior is culled from street views by the existing visibility pattern (the café's).

**Size.** **≤ 6 MB** of new data in `3d.pck`, with zero new textures as the target (§6.4). It is measured
by S23 R-a's export and recorded.

**Measurement.** Locally: counts only (R-LOAD-1). The operator runs RL-b's three-run protocol (R-6, display
awake) on the S29-d head.

**Stops.**

- A count cap exceeded is a **material stop**.
- A p95 increment above the cap, or an absolute failure in a new view, is a **material stop** with the
  levers measured. The hall size is the reserved lever (Q-S29-4).

### 6.7 The far-field `filled` slot (`ARC-107`; an additive amendment to RL-e0's format 1)

```text
Slot
  {"state": "unplaced"}
  | {"state": "reserved", … as RL-e0}
  | {"state": "filled", "place": PlaceKey, "bearing_deg": f64, "distance_m": f64 > 0,
     "footprint_m": f64 > 0, "source": string}
```

- **A filled slot names near content.** Its `distance_m` **may** be ≤ `boundary.reach_m`. This is the one
  far-field entry allowed inside the boundary, because it records where a reachable building stands rather
  than drawing it.
- **The massing keeps clear of its footprint.** The rule is the same as for `reserved`.
- **Town cross-check.** In a 3D far field of a world with `settlement.kind: town`, both slots must be
  `filled`. `station.place` must equal `settlement.station`, and `landmark.place` must equal
  `settlement.landmark`. Otherwise the file is refused `FAR_FIELD_SLOT_TOWN { pack, world, slot }`, by
  `mineworld validate` through `packages::resolve` and at client load.
- **Not a town.** In a world with `kind: none`, `filled` is refused (`FAR_FIELD_SLOT`), and the slots stay
  `unplaced` or `reserved`.
- **Default content.**
  - Social Café 3D: `station` and `landmark` both `filled`, place `station`, 270°, 70 m, footprint 60 m.
    Each `source` cites §2.2 and the compression.
  - Lakeside 3D: `station` is filled with the halt's design position. `landmark` is filled with `pier`'s.
    Every source reads "design content; lakeside declares no real location", as RL-e0's do. No client draws
    3D Lakeside before 16f.
    *(Dated note, 2026-10-15, S33 Q-S33-1/2.)* Lakeside now has a real setting, Lake Cuyamaca, and the halt
    is recorded as a stated departure from history. S33-a replaces this design content with cited positions;
    the design content stands only until S33-a lands. Lakeside is drawn in 3D by S33, not by 16f.
- **Both validators.** The Rust and GDScript validators gain the state and the code together, and RL-e0's
  AC-FF-3 corpus gains one file per new refusal.

---

## 7. Interaction: reasons to go there

### 7.1 Routines (owner: `schedule`; no System changes)

Each town gives **at least three people** one station segment in their existing routines, using labels
that are true before trains exist.

| Label | Meaning |
| --- | --- |
| `newspaper` | Market Town only: buy the paper at the kiosk |
| `meet` | the hall as the town's meeting point. Two people whose `meet` segments overlap are co-present, so `conversation` and `group-activity` work there as they do in the café |
| `sit` | a quiet half-hour on the oak benches |

- **Overlap.** At least one pair overlaps each day, so the station is never only empty scenery.
- **Boundaries.** Every boundary stays within 05:00–23:00 (A-9).
- **Labels after S21.** Waiting for a train becomes a label only when `transport` exists, through S21's
  itinerary (S21 §5.6), and never as a schedule segment.

### 7.2 The newsstand (Market Town; owners: `economy`, `employment`, `inventory`)

- **New organization** `station-news`.
  - It is declared like `corner-store`.
  - Its shop is at `station`, selling `newspaper`; the price is a realistic default with a cited source
    (a US daily single copy), scaled to Market Town's money unit.
  - Its holdings and wallet are sized so that the 300-day E-9 run never drains them.
- **Staffing.** **One existing person** gets a morning `job` there (`employer: station-news`,
  `workplace: station`, `produces: { newspaper: n }`). The person is one without a job today (ruled,
  Q-S29-6); S29-b names them. Using an existing person adds no character (S26 size) and no seat.
- **Single ownership holds.**
  - Only `economy` moves money (`buy`, `wage-due`).
  - Only `inventory` moves papers.
  - Only `employment` states the wage due.
  - Nothing new owns anything.
- **Social Café and Lakeside** have no `economy`. Their kiosk is drawn **shut** (§5.1).

### 7.3 Who is drawn there

- **People.** Only Persons the world discloses are drawn in the station, through S26's cast binding.
- **Figures.** No decorative figure is placed in the hall. A crowd of placeholder travellers waiting for
  no train would be fake twice over.
- **Lakeside.** The halt follows the same rule, under S26's B2 or its fallback.

---

## 8. 2D (`clients/2d`; the default 2D pack)

### 8.1 The sprites

- **Footprint.** The plan is the hall's real plan, 51.8 × 16.8 m, with its arch on the doorway and the
  platform strip behind it. It comes from an `interiors`/`outdoor` spec bound by tag (A-15): a new spec
  `station` with the footprint size, the door position along the long side, and the fittings (bench rows,
  kiosk, boards).
- **Façade sprite.** `facade:station` binds `sib_building_station`. It is generated by **the same pipeline
  as the other `sib_*` buildings**:
  - OpenAI image edits conditioned on the 2D pack's references, plus one HABS photograph as a form
    reference;
  - normalized at 64 px per metre;
  - one provenance sidecar per file.

  Recognisability is the twin domed towers and the arch. The style is the pack's: illustrated, moderate
  stylization, warm and calm.
- **More sprites.** Further `facade:` or `prop:` bindings:
  - `facade:ticket-office` (the counter bay with its two shutter states);
  - `prop:departures-board` (two states: blank with the "No service yet" label drawn by the client, and
    a lined board);
  - `prop:kiosk` (open and shut);
  - `prop:station-bench`.
- **Lakeside's halt.** `facade:station` is the same binding. The lakeside world selects a halt-sized
  sprite through a **second binding keyed by an extra tag** (`halt`), so the pack does not draw a 52 m
  depot at a village halt. The default binding stays the depot.
- **Pier landmark.** If Q-S29-3 rules for the pier, the `landmark` tag on `pier` adds a pier-head pavilion
  prop. This is presentation only: the pier's simulation is unchanged.
- **The approval gate.** Image generation is an external API call. It runs under the standing permission
  (API keys: call, never read), in S29-f, with the operator's look at a contact sheet before the sprites
  are committed.

### 8.2 Behaviour

- **Rules.** The state rules of §5.1 are rendered through the shared `station_state.gd`.
- **Words.** The 2D i18n gains `place.station`, `place.ticket-office`, `place.platform`,
  `station.no_service` and `station.window_closed` in every shipped locale.
- **Far field.** RL-e0's 2D surround is unchanged; the hub extent simply grows. The 2D far-field check
  (`--drive=farfield`) re-runs on the new extent.

---

## 9. Checks

### 9.1 Rendered regression shots

| Client | Path (committed) | Views |
| --- | --- | --- |
| 3D | `clients/3d-spike/shots/social-cafe/station/` | <ul><li>`st_1_spawn_west`: at spawn, yaw 270 (west), pitch 4. The towers close the street.</li><li>`st_2_east_end_west`: at (30, 1.8, 0), looking down the whole street. The landmark terminates the view.</li><li>`st_3_forecourt`: the arch from 15 m.</li><li>`st_4_arch_threshold`: standing in the arch, both sides visible.</li><li>`st_5_hall_wide`.</li><li>`st_6_ticket_window`: shut, "Closed" legible.</li><li>`st_7_board`: "No service yet" legible.</li><li>`st_8_platform`: along the edge, the fence and track ends in frame.</li></ul> |
| 3D, far field | RL-e0's `clients/3d-spike/shots/social-cafe/farfield/` | **All seven `ff_*` re-shot** as an **intended** far-field change. Both sets go to the operator (RL-e0 §7's V-gate rule). `ff_5_west_aerial` and `ff_6_west_street_end` must show the depot. `ff_6` still shows the bay to the south-west |
| 2D | `clients/2d/shots/<world>/station/` for `market-town`, `social-cafe`, `lakeside` | `st2d_approach`, `st2d_hall`, `st2d_tickets`, `st2d_platform` |

### 9.2 The landmark check (rendered, like RL-e0's)

`--slice-landmark` (3D) uses RL-e0's S/H method:

- **S** is everything shown. **H** is every node with `mw_category = "station.*"` hidden.
- It renders from `st_1` and `st_2`, and from `ff_5`.
- **LM-1 (closes the view).** In the 20° of azimuth centred on 270°, between the street's eaves line and
  the horizon, at least **60 %** of the pixels that are open in H (sky or far field) differ between S and H.
- **LM-2 (seen from the far field).** In `ff_5`, the station's projected footprint differs from H in at
  least 60 % of its pixels.

The verdict lines take RL-e0's shapes (`all landmark checks pass` / `<n> LANDMARK CHECKS FAILED`). It runs
in the nightly `clients` layer on all three OSes (RL-e0 §6.5's harness) and in RL-e0's per-PR Linux job.

### 9.3 Validation and integration tests

| Test | Layer | Asserts |
| --- | --- | --- |
| `worldpack` table test | crate | one planted world per refusal code of §4.2, each asserting the code and the named key |
| `every_reference_world_validates` | CLI | `mineworld validate` passes on every `worlds/*`, so each one declares `settlement:` |
| `packages` far-field tests | crate | `filled` accepted; `FAR_FIELD_SLOT_TOWN` for a mismatch; `filled` refused under `kind: none`; the two validators agree (RL-e0 AC-FF-3 corpus) |
| `station_visit` scenario | integration (headless) | `market-town`, 30 days, seeded: <ul><li>≥ 3 people arrive at `station` each day;</li><li>≥ 1 conversation or group activity there each day;</li><li>`newspaper` is bought there;</li><li>`buy` is never offered in `station-tickets`;</li><li>a save, restart and resume gives the same digest.</li></ul> `social-cafe` and `lakeside`: arrivals and the meeting only |
| `e9_economy_300d` | existing | still holds with `station-news` |
| 3D link probe | Godot (nightly) | `--link`: <ul><li>walk from spawn → the arch → `station` (the server's place changes);</li><li>→ `station-tickets` and back;</li><li>→ `station-platform`;</li><li>0 refusals;</li><li>the board reads `station.no_service`.</li></ul> |
| 2D drive | Godot (nightly) | `--drive=station`: the same walk; the façade is the bound sprite and the window is shut |

### 9.4 The agent (MCP) view (S27)

- The `PlaceLine` for `me.place` and for each `exits[].leads_to` carries the place's tags. It renders the
  **role tags** (`station`, `stop`, `platform`, `ticket-office`, `landmark`) as a `roles` list ahead of
  the other tags.
- This is a rendering of the Observation and nothing more. The adapter infers no service state
  (INV-S27-2). If S27-c's `PlaceLine` already carries tags, the change is the ordering only.
- **Test** (`clients/agent` scenario, after S27-c/d): an agent seat in `market-town`
  1. observes at `street`: one exit's `leads_to.roles` contains `station` and `landmark`;
  2. calls `walk_to` station: arrives;
  3. observes: offers include `buy newspaper`;
  4. moves to `station-tickets` and observes: **no offers**, `roles` contains `ticket-office`, and no
     `shop` is disclosed;
  5. moves to `station-platform`: `roles` contains `stop` and `platform`.

  The same run on `social-cafe` shows no offers in the hall.

### 9.5 Planted mutations (each must fail by name, then be reverted)

| Id | Plant | Must fail with |
| --- | --- | --- |
| X-1 | delete `settlement:` from `worlds/market-town/world.yaml` | `mineworld validate` → `SETTLEMENT_UNDECLARED` |
| X-2 | remove the `stop` tag from `station-platform` | `TOWN_STATION_NO_STOP` |
| X-3 | remove the street↔station passage | `TOWN_STATION_UNREACHABLE` naming a seat |
| X-4 | set Social Café's 3D `station` slot back to `reserved` | `FAR_FIELD_SLOT_TOWN { slot: station }` at validate and at slice load |
| X-5 | add a shop at `station-tickets` selling `newspaper` (not tagged `ticket`) | the window stays shut (link probe), and a shop selling a `ticket`-tagged kind opens it. This proves the rule reads the world |
| X-6 | hide the towers in `station.gd` | `FAIL landmark st_2 LM-1` |
| X-7 | a hard-coded `"No service yet"` that ignores a planted `departures` fixture | the shared `station_state` table test fails |

---

## 10. Invariants (proposed)

| Id | Invariant |
| --- | --- |
| I-S29-1 | **No kernel, contract, protocol or System change.** The station is Places, tags, passages, bodies, and the existing economy, employment and schedule. The diff in `kernel/` and `contracts/` is empty, and in `systems/` it is empty. The one exception is a content-only fixture in a System's test, if one is needed |
| I-S29-2 | Every town declares a station group (hall, ticket office, stop) and a landmark. A world that does not is refused by name at authoring |
| I-S29-3 | Nothing at the station shows a state the world did not disclose: board, window, kiosk and clock follow §5.1 in both clients and in the agent view |
| I-S29-4 | The landmark is reachable on foot from every seat's starting place (a load-time check where `movement` is enabled: `TOWN_STATION_UNREACHABLE`, and `TOWN_LANDMARK_UNREACHABLE` when the landmark is not the hall, Q-S29a-2) |
| I-S29-5 | One art style and one method: the depot is built with the slice's own builder and shipped materials. Every new external file passes DEP-8 |
| I-S29-6 | Real dimensions with citations. The one compression (distance; platform length) is stated in the far field's `source` and in this file |
| I-S29-7 | The far-field rules hold: RL-e0's rendered check stays green, and a `filled` slot is the only far-field entry inside the boundary |
| I-S29-8 | §6.6's performance and size caps hold |

---

## 11. PR split

### 11.1 The PRs

```text
                RL-e0 merged ─────────────┐
12d merged ─► S29-a  town contract ──┬────┼─► S29-c  far-field `filled` ─┐
              (spec, loader, content)│    │                              │
                                     ├─► S29-b  newsstand (Market Town)  │
                                     │                                   ▼
16c merged ──────────────────────────┼────────────────────────► S29-d  3D depot exterior + forecourt
RL-b2 merged (perf baseline) ────────┘                                   │
                                                                         ▼
                                                                S29-e  3D interior, boards, window, platform
                                     S29-a ─► S29-f  2D sprites + rules (image pipeline; operator sheet)
                         S29-a + S27-c/d ─► S29-g  agent view roles + scenario
S26-c2 (cast figures) ─ soft: people in the hall are drawn by S26's binding whenever it lands; not a gate
```

| PR | Scope | Depends on | Integration checkpoint | Adversarial criteria | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S29-a** town contract | <ul><li>**C1, specification first:** MODULE_SPEC §4.1 and §4.3, CORE_CONCEPTS §6 role tags, VISUAL_SLICE §4 amendment, `ARC-105`.</li><li>**C2:** the `worldpack` `settlement:` key and the §4.2 checks.</li><li>**C3:** `mineworld validate` requires the key.</li><li>**C4:** the three station groups in three worlds, with bodies, passages and routines; `bodies-yard` `kind: none`.</li><li>**C5:** the one recorded re-baseline of digests and golden fixtures (Q-S29-7).</li></ul> | **12d merged** (bodies sections, its own re-baseline first) | `mineworld run worlds/market-town` 30 d: `station_visit` (social half) passes; save, restart and resume are identical; `every_reference_world_validates` | X-1, X-2, X-3; a `station` tag on a place not joined to the hall → `TOWN_STATION_SCATTERED` | **Haiku**, from a commit-level plan (the `naming`/`ARC-53` pattern) |
| **S29-b** newsstand | `station-news` organization, shop and stock; one person's `job`; price with source; E-9 300-day run | S29-a | `station_visit` buys a newspaper; E-9 holds | removing `economy` from the world refuses `station-news`'s section by name (the existing rule) | **Haiku** |
| **S29-c** far-field `filled` | RL-e0 schema `filled` in Rust and GDScript; `FAR_FIELD_SLOT_TOWN`; Social Café 3D and Lakeside 3D content; Social Café `reach_m` 60 → 100; corpus files; MODULE_SPEC §6.1.1 amendment; `ARC-107` | **RL-e0 merged**; S29-a | `mineworld validate worlds/lakeside --packs …` passes; the slice loads; RL-e0's rendered check is still green (the footprint is clear of massing) | X-4; a `filled` slot under `kind: none` is refused | **Haiku** (additive schema on a frozen pattern) |
| **S29-d** 3D depot exterior | `station.gd` (towers, arch, hall shell, arcade, roofs); `glazed_tile.gdshader`; forecourt in `street.gd`; binding by role tag; `--slice-measure` rows; HABS references and provenance; perf views; `st_1`–`st_4`; `ff_*` re-shot; `--slice-landmark` | **16c merged**; RL-b2 merged; S29-a; S29-c | `./mineworld-slice --world`: walk from spawn into the hall; the server's place changes to `station`; LM-1 and LM-2 pass on three OSes; counts within §6.6 | X-6; the RL-e0 rendered check must still pass; a tile shader that writes to an intent fails the client-rules scan | **Opus** (art judgement, perf levers); mechanical parts delegable |
| **S29-e** 3D interior | hall fit-out (beams, wainscot, benches as a MultiMesh, kiosk, counter, boards, clock); `station-tickets` bay; platform, fence and track; `station_state.gd` (shared); `st_5`–`st_8`; threshold measurement | S29-d | link probe: spawn → hall → tickets → platform, 0 refusals, the board reads `no_service`; perf of the new views | X-5, X-7 | **Haiku** with a commit plan for the fit-out and `station_state`; **Opus** review of frames |
| **S29-f** 2D | the `station` spec; `sib_building_station`, the halt sprite, kiosk, board and window sprites by the existing image pipeline; i18n words; the shared state rule; 2D shots; RL-e0's 2D far-field check re-run | S29-a (S29-e's `station_state.gd`, or lands it first) | `./mineworld-2d` on all three towns: walk to the station, the window is shut, the board reads "No service yet" | the halt binding: Lakeside does not draw the 52 m depot; 2D far field has zero clear-colour pixels | **Haiku** (pipeline run); the operator's contact-sheet gate |
| **S29-g** agent view | `PlaceLine.roles`; the §9.4 scenario in `clients/agent` | S29-a; **S27-c merged** (S27-d for MCP transport) | the §9.4 scenario passes over MCP | the adapter must not invent a service state: a planted `departures` fixture is shown only as disclosed | **Haiku** |

### 11.2 Start conditions and order

- **Start.** Nothing starts before Milestone F's checklist is handed to the operator. That is the timing
  ruling.
- **First wave.** S29-a goes first, once 12d has merged.
- **Second wave.** S29-b, S29-c, S29-f and S29-g follow in parallel worktrees, one each (§3.1).
- **3D.** S29-d waits for 16c and RL-b2, and S29-e follows S29-d.
- **Release.** All seven are in before S23's first public release candidate. S29-d and S29-e are the
  critical path.
- **Lakeside in 3D.** Lakeside's 3D halt and pier pavilion are **not** in this split. They come with 16f,
  the general 3D pack loader that draws Lakeside in 3D (MVP-1). S29-c gives them their far-field slots now.
  *(Superseded 2026-10-15 by the operator's re-ruling, S33 Q-S33-10.)* They come with **S33 (MVP-0,
  release-blocking)**: S33-e builds them with S29-e's pieces, which S29-e writes so a village-scale building
  can reuse them ([step-33-lakeside-3d.md](step-33-lakeside-3d.md) §4.4). 16f is MVP-0 too (`overall.md`).

### 11.3 Step acceptance (fixed before measuring)

| Id | Criterion |
| --- | --- |
| AC-S29-1 | `mineworld validate` passes on every `worlds/*`. X-1 … X-4 each fail by name |
| AC-S29-2 | `station_visit` passes on the three towns, including save, restart and resume |
| AC-S29-3 | In the 3D slice: from spawn, the player walks into the hall, the ticket bay and the platform, with server place changes and 0 refusals. `st_1`–`st_8` are committed |
| AC-S29-4 | LM-1 and LM-2, and RL-e0's far-field check, pass on Linux, macOS and Windows |
| AC-S29-5 | The window is shut, the kiosk follows the shop rule, and the board reads "No service yet" in 2D, in 3D and in the agent view. X-5 and X-7 behave as specified |
| AC-S29-6 | §6.6's counts are within caps on the M5. The operator's three-run timings are recorded. `3d.pck` grows by ≤ 6 MB |
| AC-S29-7 | 2D shows a recognisable depot and the Lakeside halt from the same pipeline, with provenance sidecars. The operator's contact-sheet verdict is recorded |
| AC-S29-8 | The operator walks it: an item in the release's hands-on checklist ("walk from spawn to the station; look back down the street; enter; read the board") |

---

## 12. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S29-1 | The 52 × 17 m hall at real size breaks 16.7 ms in its own view | Batching, a MultiMesh for the benches, one tile material and occluders are built in from the start (§6.6). The reserved lever is a reduced hall (Q-S29-4), which is a material stop |
| R-S29-2 | The re-baseline collides with S26-a's people-file edits and with in-flight PRs | S29-a's C5 is a single commit, rebased last. The primary schedules it in a quiet window |
| R-S29-3 | HABS photographs lack a scale, so the tower height stays a placeholder | Measure against the documented 8½ ft wainscot and the door leaves in the same photograph. Record the method and the error |
| R-S29-4 | The Mission Revival building clashes with the lakeside-referenced street | It is sanctioned explicitly (Q-S29-5) and uses the same palette and materials. The operator judges `st_1` and `st_2` |
| R-S29-5 | The byte budget is already contested by S26 | Zero new textures is the target. The tile is a shader. ≤ 6 MB is a gate, not a hope |
| R-S29-6 | S21 takes another shape for departures, so the board rule is rewritten | §14.6 hands S21 the non-preclusion item now. The board reads a record by name, and only that record's shape can change |

---

## 13. Questions for the operator (each with a recommendation; all ruled 2026-10-15)

| Id | Question | Recommendation | Ruling |
| --- | --- | --- | --- |
| **Q-S29-1** | **Where the station stands.** At the far-field slot's real distance (790 m, with a ≈ 700 m walkable approach), or at the west end of the slice street (≈ 62–79 m, the same bearing family, real building size), reachable from spawn in ≈ 40 s? | **The street's west end (P-1).** It is the only placement that meets "reachable on foot from spawn", "street level" and "closes a street view" together. It keeps the slot's bearing and mirrors the real Broadway–depot axis. The compression is recorded in the far field's `source`. The far-field slot becomes `filled` (§6.3, §6.7) | **Ruled 2026-10-15** (operator answered directly in the primary session): P-1: the west end of the street (≈ 62–79 m), real building size. The compression is recorded in the far field's `source`, and the slot becomes `filled`. |
| **Q-S29-2** | **How a world says it is a town**, and where the requirement is enforced | **`settlement: { kind: town \| none, station, landmark }` in `world.yaml`,** required by `mineworld validate` / `packs validate` (the `ARC-53` precedent) and fully validated by the loader whenever present. Lakeside counts as a town and `bodies-yard` declares `none` | **Ruled 2026-10-15** (operator answered directly in the primary session): the `settlement:` key, enforced by `mineworld validate` / `packs validate` (as with `ARC-53`). |
| **Q-S29-3** | **Lakeside: its station and its landmark.** Does the village get a station, and which place is its landmark? | **Yes, a small halt** (a waiting room of ≈ 8 × 5 m, a ticket window and a 40 m platform) off `lane`. **The landmark is the pier**, with a pier-head pavilion (presentation only). The pier is public, outdoor, seen from the whole shore, and already a destination (`fishing`). The boathouse is people's home (`residential`), and a landmark should be a place anyone may go | **Ruled 2026-10-15** (operator answered directly in the primary session): Lakeside gets a small halt, and the pier is its landmark. |
| **Q-S29-4** | **Hall size.** Real (51.8 × 16.8 m), or reduced from the start? | **Real.** A reduced hall (≈ 30 × 12 m) is the reserved lever, used only if §6.6's material stop is reached, with numbers | **Ruled 2026-10-15** (operator answered directly in the primary session): the real hall size, 51.8 × 16.8 m. |
| **Q-S29-5** | **Style.** Is one Mission Revival building, the depot, acceptable as the sanctioned exception beside the lakeside-referenced street, rendered with the same materials, palette and light? | **Yes.** That is what "the town's signature building" means. The same materials, palette and precision keep it one art style | **Ruled 2026-10-15** (operator answered directly in the primary session): the depot is the one sanctioned Mission Revival exception. |
| **Q-S29-6** | **The newsstand's seller** in Market Town: an existing person given a morning job, or a new person? | **An existing person** without a job today. That means no new character (S26 bytes), no new seat and fewer moving digests | **Ruled 2026-10-15** (operator answered directly in the primary session): an existing person with no job runs the newsstand. |
| **Q-S29-7** | **A second digest re-baseline.** Adding the station's places, the organization and the routines shifts ids in all three towns, against `overall.md`'s "re-baselined once, in 12d" | **Accept one recorded re-baseline in S29-a's C5,** after 12d and S26-a have merged, in one commit | **Ruled 2026-10-15** (operator answered directly in the primary session): one recorded re-baseline, in S29-a's C5. |
| **Q-S29-8** | **References.** May HABS CA-1965's public-domain photographs be committed to the 3D pack's `references/`? | **Yes.** They are federal works in the public domain and pass DEP-8. CC BY-SA photographs are linked only | **Ruled 2026-10-15** (operator answered directly in the primary session): the HABS CA-1965 photographs may be committed. |
| **Q-S29-9** | **Social Café too?** It is the 3D slice's world and has no economy | **Yes, all three towns.** Social Café's kiosk is drawn shut, which is honest. The slice needs its station there because the slice *is* Social Café | **Ruled 2026-10-15** (operator answered directly in the primary session): all three towns get a station. |
| **Q-S29-10** | **Byte budget.** May S29 add up to 6 MB to `3d.pck`, inside S26's ruled envelope (O-1, else O-4)? | **Yes, ≤ 6 MB, with a target of 0 new textures** | **Ruled 2026-10-15** (operator answered directly in the primary session): the byte budget is ≤ 6 MB, with a target of 0 new textures. |

### 13.1 Rulings, 2026-10-15

The operator answered Q-S29-1…10 directly in the primary session on 2026-10-15: every question as
recommended. The coordinator relayed the rulings to this session. The operator also confirmed the start
condition: nothing in S29 starts until Milestone F's checklist has been handed over (§11.2).

---

## 14. Proposed edits elsewhere (made by the PRs named)

### 14.1 `docs/CORE_CONCEPTS.md` §6 (S29-a C1)

Add "§6.4 Role tags", or the next free number if S21's §6.4 Region lands first:

> A **role tag** is a tag whose meaning the framework fixes, so that validation and every client read it
> alike. Role tags are authored content. They grant no behaviour; behaviour comes only from Systems. The
> role tags are: `station`, `stop`, `platform`, `ticket-office`, `landmark`.
>
> - A **station** is the group of Places tagged `station` joined by passages to the hall a town declares.
> - A **stop** is S21's: a Place a line serves.
> - The origin of a `stop` Place is its **platform anchor**: a point on the platform edge, mid-length, at
>   platform height. The track lies beyond that edge.
> - A **landmark** is the Place a town declares as its signature building.

### 14.2 `docs/MODULE_SPEC.md` (S29-a C1; S29-c)

- §4.1: the `settlement:` key, as §4.1 of this file.
- §4.3 (new): the refusal table of §4.2.
- §6.1.1 (RL-e0's): the `filled` state, the inside-boundary exception and `FAR_FIELD_SLOT_TOWN`.

### 14.3 `docs/VISUAL_SLICE.md` (S29-a C1, dated amendment)

- §4: "a third enterable building" moves from out of scope to **required: the town's station**, as in
  §4.1's amendment pattern.
- The west termination becomes "the station forecourt closing the view".
- §12: add view 11, "the station from the street and in the hall".

### 14.4 `docs/ART_DIRECTION.md` §4 (S29-d)

Add "the landmark exception": one signature building per town may follow its own vernacular, within the
pack's materials, palette, light and precision.

### 14.5 `docs/DECISIONS.md` (ids assigned 2026-10-15; entries added with the step approval)

| Record | Content |
| --- | --- |
| `ARC-105` | towns declare `settlement:`; the station group and landmark; role tags; authoring-gate enforcement |
| `ARC-106` | the depot: compressed distance, real building, one method; the HABS references |
| `ARC-107` | the far-field `filled` slot and its inside-boundary exception (dated note on `ARC-102`) |
| `ARC-108` | fixtures show only disclosed state (board, window, kiosk, clock) |
| `DEP-60` | reserved for a new CC0 texture if §6.4's last row triggers; withdrawn otherwise |

### 14.6 Step-21 (non-preclusion item for RT-c)

`transport` discloses a `departures` record (lines and next departures from the group's stop) to observers
in any Place tagged `station` whose group contains a `stop` it serves. A board in the hall can then read it
(F-S29-3). `fares`' ticket shop is declared at the town's `ticket-office` Place. `StopInfo`'s door anchors
lie along the platform-anchor edge. **Added 2026-10-15 (Q-S29a-1):** when it attaches `StopInfo`, `transport`
refuses by name a stop whose origin does not lie on exactly one edge of its `body.floor`.

### 14.7 `overall.md`

- Add S29 to §3's steps, after Milestone F and before S23's release candidate, with this file linked.
- Record Q-S29-7's re-baseline next to the 12d "Digests" rule once ruled.

### 14.8 Rulings from the S29-a PR design (2026-10-15)

Ruled 2026-10-15 (operator answered directly in the primary session), every one as
[`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) §14 recommended:

| Id | Ruling | Effect here |
| --- | --- | --- |
| Q-S29a-1 | The platform-anchor rule moves to its consumer: a content test in S29-a's C6, and a refusal in S21's RT-c when `transport` attaches `StopInfo` | §4.2's `TOWN_PLATFORM_ANCHOR` row struck and annotated; §14.6 gains the RT-c refusal |
| Q-S29a-2 | Add `TOWN_LANDMARK_UNREACHABLE` | §4.2 row added; I-S29-4 annotated |
| Q-S29a-3 | The `mineworld create` template declares `settlement: { kind: none }`, with a comment pointing to `MODULE_SPEC.md` §4.3 | — |
| Q-S29a-4 | Social Café's station routines land in S29-a | — |

Also recorded from the PR design's audit: `mineworld validate` did not enforce `ARC-53` (A-18 is corrected: the
gate is new for that command, F-S29a-2), and Q-S29-7's "one re-baseline" is read as the one id shift; S29-b
records its newsstand sales as a behaviour change (F-S29a-7).

### 14.9 Rulings from the S33 PR designs (2026-10-18), applied by S29-a revision 3

Ruled 2026-10-18 by the operator in the primary session and recorded in the S33-a and S33-b PR designs (frozen,
merged as #255). S29-a revision 3 ([`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) §14.2) carries them.

| Ruling | Source | Effect here |
| --- | --- | --- |
| Q-S33a-1: the halt at the lane's east end, a north–south platform with its end against the shore | `pr-s33a-setting-far-field.md` §14; R-S29a-a-1 | §4.3's Lakeside paragraph: dated note; the proposal is now ruled |
| The ticket room is at least 4.6 × 4.0 m, by `bodies`' capacity rule; "S29 must change to carry it" | `pr-s33b-lakeside-bodies.md` header, F-S33b-5, §11 | §4.3's "Bodies" bullet: dated note; "about 6 000 × 3 000 mm" superseded |

No invariant (§10), step acceptance (§11.3) or PR boundary (§11.1) changes. The details the rulings leave open —
the towns' bay size, Lakeside's bay depth with its counter, and the track's edge — are S29-a's Q-S29a-5 …
Q-S29a-7. *(Ruled 2026-10-18 as recommended: 6.0 × 4.6 m, 4.6 × 4.6 m, the east edge; S29-a §14.4.)*

### 14.10 Frames the clients can draw, applied by S29-a revision 4 (2026-10-19)

A client places a Place by moving its frame, never by turning it, so each station place's frame must already face
the way it is drawn. The S29-d and S33-e designs found three frames that do not. S29-a revision 4
([`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) §14.5) carries the fixes; the ones marked *open* wait
for its Q-S29a-8 … Q-S29a-11.

| Change | Source | Effect here |
| --- | --- | --- |
| Lakeside's ticket bay north of the waiting room | Q-S33e-3, ruled (#285, 2026-10-19) | §4.3's Lakeside paragraph: the bay's place is fixed |
| The towns' bay south of the hall (door on its north edge, counter on its south wall); the platform north–south along the hall's west wall, about 11 000 × 60 000 mm, the real 8.2 m arcade with its piers as solids | S29-d R-S29a-d-1; *open* (Q-S29a-8) | §4.3's "Bodies" bullet: "the platform about 60 000 × 6 000 mm" becomes 11 000 × 60 000 mm, running north–south |
| No `halt` tag: a client tells the depot from the halt by `landmark` | S29-f Q-S29f-1; *open* (Q-S29a-9) | §8.1's "second binding keyed by an extra tag (`halt`)" is superseded |
| A role tag may choose a drawing or order lines in a client; it never grants behaviour | S29-g Q-S29g-2; *open* (Q-S29a-10) | I-S29-1's reading; §14.1's text gains the sentence |
| Role tags are `CORE_CONCEPTS.md` §6.5 (§6.4 is now "Sit point") | `main` | §14.1's heading |

No invariant (§10), step acceptance (§11.3) or PR boundary (§11.1) changes.

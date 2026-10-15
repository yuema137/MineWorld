# Step 33 — S33: the Lakeside world in 3D

**Role:** step document for **S33**, a new step that the primary session adds to [`overall.md`](overall.md).
The document contains:

- the operator's re-ruling and the earlier rulings it supersedes;
- an audit of the 3D client and of Lakeside's semantic world;
- a proposed real-world setting for Lakeside, with a far field built from cited data;
- the 3D world: terrain, water, the pier landmark, the halt, vegetation, interiors, sit points,
  navigation and collisions;
- people, 2D parity, agents, budgets and packaging;
- acceptance and adversarial criteria, the PR split with implementers, risks, records and operator
  questions.

**Effort:** `mvp0`, **release-blocking** (operator, 2026-10-15; §0.1). **Parent:** [`overall.md`](overall.md).

**Lifecycle:** `APPROVED 2026-10-15 (step plan; PR designs to follow)`. The operator answered Q-S33-1…11
directly in the primary session on 2026-10-15, every one as recommended (§14.1).

- Approval fixes this step's scope, invariants (§10) and step acceptance (§11). Progress, evidence,
  findings and bounded corrections stay writable.
- Each PR is detailed to the commit in its own PR design and frozen in turn (`CLAUDE.md` §3.1). Only a PR
  design carries `DESIGN FROZEN`.
- **Start condition (Q-S33-10):** nothing starts until Milestone F's checklist has been handed to the
  operator.
- Nothing in this file authorizes implementation, a Meshy call, an OpenAI call, a purchase or a merge. S33
  spends **0 Meshy credits**; S33-h's OpenAI cap is set at its freeze; nothing is bought (Q-S33-7).
- Superseded lifecycle: `STEP DESIGN (DRAFT)`, PR #228 revision 1 (`e56634ba`).

**Author:** S33 design session (Opus), 2026-10-15.

- Worktree: `/Users/yuema137/mineworld-worktrees/design-s33`.
- Branch: `docs/s33-lakeside-3d`.
- Audit base: `origin/main @ 5d901e85` (S32 approved and merged, #226).

**Ids.** Assigned 2026-10-15 from the range the primary session reserved for this step (`ARC-143…ARC-148`,
`DEP-77…DEP-80`), bottom up. They replace the placeholders ARC-S33-a…d and DEP-S33-a…d:
`ARC-143…ARC-146` and `DEP-77…DEP-79` are used; `ARC-147` is reserved for the `posture` System step that
Q-S33-6 makes a required follow-up (§14.2); `ARC-148` is unused and returned; `DEP-80` (a paid asset) is
withdrawn because Q-S33-7(c) ruled that nothing is bought (§15). Each record is written into
`docs/DECISIONS.md` by the PR named in §15.
Local ids in this file:

| Prefix | Meaning |
| --- | --- |
| `SD-S33-n` | step decisions |
| `A-n` | audit anchors |
| `F-S33-n` | findings |
| `AC-S33-n` | step acceptance |
| `X-n` | planted mutations |
| `I-S33-n` | invariants |
| `R-S33-n` | risks |
| `Q-S33-n` | operator questions |
| `VP-L-n` | 3D viewpoints in Lakeside |

**Parallel-safety.** Revision 1 wrote only this file. At approval the coordinator directed this session to
apply the plan-file edits of §16 in the same PR (#228), as dated notes:

- `overall.md` (S26–S33 entries, 16f's placement);
- `step-26-cast.md`, `step-29-station-landmark.md`;
- the RL-e0 design (`../mvp1/pr-rl-e0-far-field-mandatory.md`).

`docs/MODULE_SPEC.md`, `docs/VISUAL_SLICE.md`, `docs/ART_DIRECTION.md` and `docs/DECISIONS.md` are edited by
the S33 PRs named in §§15–16.

---

## 0. Binding inputs

### 0.1 Operator re-ruling, 2026-10-15 (relayed by the coordinator)

- **Lakeside's own 3D world ships before the public release.** Before this ruling it was MVP-1 (Q-S26-1),
  and the ten Lakeside people were only extras on the café street (E-2).
- The governing principle, in the operator's words:

  > "我们要发布就得把这些功能全部做好"

  In English, for the specification (`CLAUDE.md` §2.1 rule 5): *"If we are going to release, every one of
  these features has to be done well."* A problem that looks hard is researched, not dropped (the same
  principle S32 §0.1 records).

### 0.2 Earlier rulings this re-ruling supersedes or touches

| Ruling | Where | What changes |
| --- | --- | --- |
| Q-S26-1: "Lakeside's 3D cast is MVP-1, except as café-street extras under E-2" | `step-26-cast.md` §16.1, header | Lakeside's cast is drawn **in its own world** in MVP-0. E-2 stands in both directions: the café town's cast becomes Lakeside's extras (S26 §8 already says "a future 3D lakeside's extras are the café town's") |
| S29 §11.2: "Lakeside's 3D halt and pier pavilion are **not** in this split. They come with 16f … (MVP-1)" | `step-29-station-landmark.md` §11.2 | They come with S33, in MVP-0 |
| S29 §6.7: Lakeside's 3D far-field slots are filled with "design content; lakeside declares no real location" | `step-29…` §6.7; `ARC-107` | S33-a replaces the design content with a cited real setting (Q-S33-1) |
| RL-e0 App. B.2: Lakeside's 3D far field is "design content; no real location", the lake to the **north** (300°→60°) | `../mvp1/pr-rl-e0-far-field-mandatory.md` App. B.2, B.3 | Replaced by S33-a. The lake's bearing is also wrong against the world's own data (F-S33-2) |
| QRL-2 (2026-10-09): "the alpine lake moves to a later S21 region. The lakeside references keep their authority over architecture, materials, light and mood" | `../mvp1/step-22-realism.md` §13.1 | Unchanged. S33's setting is inside San Diego County (the casting profile), not an alpine region. `06_lake_trail.png` and `01_lakeside_promenade.png` keep authority over architecture, materials, light and mood, **not** over the snow peak in their backgrounds (§3.3) |

### 0.3 Standing rules that bind this step

| Rule | Consequence here |
| --- | --- |
| Every world has a far field; air walls are fine, an empty horizon is not (RL-e0, `ARC-102`) | Lakeside's 3D far field passes RL-e0's rendered check from every Lakeside viewpoint (§9, AC-S33-1) |
| Every town has a train station and a signature landmark (S29, `ARC-105`) | The halt and the pier are drawn in 3D, reachable on foot, and the pier passes a landmark check (§4.3, §4.4) |
| Every NPC meets the protagonist's standard (S26 §1, S32) | Lakeside's ten people are their own modular characters (S32) at Vera's tier; no placeholder figure is drawn (§5) |
| Humans and agents are both first-class players (S27) | An MCP agent plays Lakeside through the same server path, and a 3D client sees it (§7) |
| Framework, not demo | Lakeside's 3D look is presentation data and builders selected by world and by tag. Nothing in the kernel, contracts or Systems knows Lakeside (§4.8, I-S33-1) |
| All platforms | Every rendered check runs on the Linux, macOS and Windows nightly runners (§9.6) |
| Realistic defaults, cited | Every dimension cites a source, or says it is approximate and names who replaces it (§3, §4) |
| One art style, one precision, one method (S26 §7.4a; S29 §6.1) | Lakeside is built with the slice's procedural builder, its material library and its palette. No second toolchain, no generated building (§4.9) |
| Reuse before building (`REUSE_POLICY.md`) | Water, terrain and vegetation options compared both ways (§4.9) |
| Local load: CI is the gate (R-LOAD-1) | Local timings are INCONCLUSIVE. Counts are recorded locally; timings come from the operator or CI (§8) |
| Design on Opus, implement on Haiku | Each PR names its implementer (§12) |
| Anything that costs money is an operator question | §14 |

---

## 1. The requirement, restated as checkable outcomes

1. **Lakeside is a 3D world you walk in.** `./mineworld-slice --world=lakeside` hosts `worlds/lakeside` and
   puts the player in their seat's person, in the village, on the same server-authoritative path as the
   café street.
2. **It has a real setting and a far field.** The setting is a cited real place. The far field closes every
   view from every reachable point (RL-e0's check).
3. **The landmark and the station are there.** The pier is the landmark: it is seen from the whole shore and
   walked to its head. The halt (waiting room, ticket window, 40 m platform) is entered and walked.
4. **The places are real.** Interiors exist where the world has an indoor place. Seats can be sat on, with
   seat contact.
5. **Its own people live there.** The ten Lakeside people are their own characters in their own outfits. The
   café town's people are the extras.
6. **Both clients and agents agree.** 2D and 3D show the same places, doorways and people. An MCP agent plays
   it like a human.
7. **It fits.** The perf views hold 16.7 ms p95. The release size holds the ruled budget.
8. **It is removable.** A world without the Lakeside 3D content still runs, and the café street is
   unchanged.

---

## 2. Audit (`origin/main @ 5d901e85`, 2026-10-15)

### 2.1 What 3D exists today

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | The current 3D client is the slice: `scenes/slice.tscn` → `SliceMain`, Godot 4.7 (CI pins 4.7.2), Forward+, Jolt, occlusion culling on. `./mineworld-slice --world` **always** hosts `worlds/social-cafe`. There is no world-id switch | `clients/3d-spike/project.godot`; `scripts/slice/slice_main.gd`; RL-e0 SD-FF-7 | S33 must add 3D world selection (§4.8) |
| A-2 | The 3D client loads **no Presentation Pack** ("S14 16f"). Places are bound to geometry by hand: `SliceWorld` constants (`CAFE_PLACE`, `STREET_PLACE`, `FLORIST_PLACE`), `_place_volumes()` `Area3D`s with `place_id`, and `SliceLink.KEYS = ["cafe", "street"]`, bound by doorway translation (`door_point`, `origin`, `to_scene`/`to_world`) | `slice_world.gd`; `slice_link.gd`; step-15 D-16c-3, SC-4 | The Lakeside binding needs the per-world part of 16f (F-S33-1) |
| A-3 | Everything is built in GDScript at load: `SliceStreet`, `SliceTerrace`, `SliceCafe`, `SliceCafeInterior`, `SliceShopInterior`, `SliceStreetscape`; then `SliceBatch.merge` (24 m cells), `SliceBudget.apply` (screen-size visibility), `SliceOccluders.build`. Materials go through `Mats.pbr(slug, metres, tint, rough)` over 22 Poly Haven CC0 1k sets; `SlicePalette` is the named library | `scripts/slice/*.gd`; `scripts/mats.gd`; `ASSETS.md` | Lakeside is built the same way, from the same library (§4.9) |
| A-4 | **The earlier promenade spike already built a lakeside**: `scripts/town.gd` (`Town`) has `_lake` (a 1 400 × 1 200 m plane at y −1.15 with `shaders/water.gdshader`, 41 lines, noise normals), quay walls, `_lake_trail`, `_far_shore`, `_mountains` (a FastNoise heightfield), a jetty, a rowing boat and boulders. `scripts/props.gd` and `scripts/procgen.gd` make **broadleaf and conifer trees with alpha-cut card canopies** and leaf, needle, flower, grass and treeline textures procedurally | `scripts/town.gd`; `scripts/props.gd`; `scripts/procgen.gd`; `ASSETS.md:220-223`; `screenshots/05_lake_scenic.jpg`, `08_lake_trail.jpg` | The jetty, boat, boulders, tree cards and water shader are **reused** into the slice's builder pattern, not rebuilt (§4.9) |
| A-5 | Sky and light: one HDRI (`qwantani_puresky_2k`), AgX, a fixed sun (elevation −19.3°, azimuth −48°), depth fog 70→1 500 m, switchable GI (default VoxelGI over the café), two reflection probes. **No time of day, no weather** (TW-e) | `slice_main.gd` | Lakeside uses the same rig, with its own GI volume and probes (§4.2). Time of day stays TW-e's |
| A-6 | The slice terrain is flat: a 900 × 900 m `leafy_grass` apron at y −0.14, plus box banks and parapets. RL-e0 replaces the apron with a near-ground disc and a far-field foreground | `street.gd`; RL-e0 §5.4 | Lakeside needs a graded near terrain (a shore that falls to the water) (§4.1) |
| A-7 | **RL-e0 is frozen, not merged** (PR #220 open). No `far_field/` file exists. It specifies `far_field/<world>.yaml`, the 3D view (`SliceFarFieldView`: camera-anchored silhouette ribbons at 900/1 100/1 300 m, a foreground annulus to 1 450 m, a water surface and horizon ribbon at 1 400 m, seeded massing), the rendered check `--slice-farfield` (S/H/E frames, 8 azimuths, FF-BELOW ≥ 90 %, FF-ABOVE ≥ 25 %) and no fallback (SD-FF-8) | `../mvp1/pr-rl-e0-far-field-mandatory.md` §§5–6 | S33 depends on RL-e0 merged and re-uses its view and check unchanged, with Lakeside's viewpoints (§9.1) |
| A-8 | **No navmesh** anywhere. Layers: `LAYER_WORLD` 1, `LAYER_BODIES` 2. The player capsule is r 0.30, h 1.72; walk 1.45 m/s, jog 3.10 m/s, floor limit 52°; steps and kerbs are hidden wedge ramps. A `move` is sent every 0.5 m or 20° of turn; **height is never reported** | `build.gd`; `player.gd`; `camera_rig.gd` | Terrain height is presentation; the server stays planar (§4.6) |
| A-9 | 12d (towns get bodies; PR #171) and 12e (client colliders from disclosure, `client_bodies.gd` `DisclosedGeometry`, SD-E4) are frozen and unmerged. **12d covers `social-cafe` and `market-town` only** | `step-11-bodies.md` §§19, 22; PR #171 | Lakeside has no `body:` sections and no `bodies` System (F-S33-3) |
| A-10 | Seats: there is no sit-point system. `SliceStreetscape._chair_seat()` derives one seat transform from a glTF chair for one seated decor NPC; the spike uses `CAFE_SEAT_Y` 0.45 and `BENCH_SEAT_Y` 0.44. The simulation has no posture | `streetscape.gd`; `town.gd`; `systems/` (no posture component) | Sit points are designed here as presentation data (§4.5, Q-S33-6) |
| A-11 | Characters today: CharMorph `vitruvian.glb` for townspeople, `meshy_d.glb` for the player, Quaternius UAL clips. The cast binding (`cast.yaml`, looks, extras, default) is S26-a/S26-c2; modular characters are S32-c | `human.gd`; `npc.gd`; step-26 §4.3; step-32 §11 | Lakeside people come through the same binding (§5) |
| A-12 | Perf: `SlicePerf` (`--perf`), 8 fixed views, 1920 × 1080, gi=voxel, settle 6 / warm 120 / measure 300 frames. Binding: p95 ≤ 16.7 ms, video memory ≤ 2 048 MB. Advisory: draw calls 2 000, primitives 3 000 000. Last operator run (2026-10-14, M5, Metal): 6 views pass, **cafe frontage 17.11 ms and doorway 18.59 ms fail** (RL-b2's to fix); video memory 1 608 MB | `slice_perf.gd`; `pr-rl-b2…` line 31 | Lakeside adds its own views, judged on the absolute gates (§8.1) |
| A-13 | `3d.pck`: hard limit 250 MB, goal 150 MB; R-a measured ≈ 259.5 MB compressed (over); RL-b2 estimate ≈ 177 MB; S26/S32 cast forecast over the limit, ruled O-1 + O-3 + O-5, else O-4 raises the limit to ≈ 350 MB (Q-S26-5, Q-S32-8); S29 adds ≤ 6 MB | step-23 §7; step-26 §10.3; step-32 §14.2; step-29 §6.6 | Lakeside's increment must be small, and its packaging is an operator question (§8.2, Q-S33-3) |
| A-14 | Tests: probe modes (`shots`, `drive`, `measure`, `threshold`, `perf`, …; `link`, `conversation`, `target`, …); verdict lines `all <mode> checks pass` / `<n> <MODE> CHECKS FAILED`; nightly `clients-probes` (Linux, macOS) and `clients` (Rust `#[ignore]` Godot tests); `tools/frame_diff.gd` (8/255). No Godot run per PR yet (RL-e0 adds a Linux per-PR job) | `slice_probe.gd`; `slice_probe_world.gd`; `scripts/ci_layer.py`; `scripts/ci_godot.py` | Lakeside's checks are new probe modes on the same harness (§9) |
| A-15 | References: `presentation/mineworld-default/3D/references/01_lakeside_promenade.png` and `06_lake_trail.png`. Read in pixels for this design (ACCEPTANCE §3.1): **06** shows a packed-earth trail, a slatted timber bench, a fingerpost ("Lakeside Trail / Pine Ridge / Town Center"), granite boulders, wildflowers (daisies, lupins), conifers and broadleaf trees, a **timber jetty on round posts** with a lantern on a crate at its head and two moored rowing boats, lily pads and ducks, a footbridge, a church spire and a waterfall across the lake, a snow-capped peak, low sun to the right. **01** shows a setts promenade with a low railing to the water, lamp posts, planters, benches, sailboats and a far shore with a spire under wooded mountains | the PNGs | The pier and shore follow **06**; the lane's materials follow **01**/**05**. The snow peak and the waterfall are not San Diego content and are not copied (§3.3) |

### 2.2 What Lakeside's semantic world contains

| Id | Fact | Source |
| --- | --- | --- |
| A-16 | `world.yaml`: id `lakeside`, version 0.1.0, MIT. Requires `acme-fishing`, `modern-goods`, `mineworld-default-2d`, `mineworld-default-3d` (declared only). Systems: presence, movement, conversation, group-activity, relationships, naming, schedule, item, inventory, item-transfer, consumption, fishing. **No** `bodies`, `economy`, `appearance`, `attributes`, `wardrobe`, no `settlement:`, no `configure/` (no calendar, no location) | `worlds/lakeside/world.yaml` |
| A-17 | **Seven places, a star around `lane`**, each joined by one passage. Positions are millimetres from each place's south-west origin, x east, y north: <ul><li>`lane` (street, public, outdoor): about 30 × 6 m, "a cobbled lane along the lake"; houses, bakery and square on its **north** side, shore and pier on its **south** side.</li><li>`cottages` (residential): four cottages round a yard with a pump; door at lane (3 000, 5 800).</li><li>`boathouse` (residential): "rooms above the boats. The fishermen live here"; door at lane (9 000, 5 800).</li><li>`bakery` (indoor): a long table by the window; door at lane (15 000, 5 800).</li><li>`square` (public, outdoor): an old well, benches under a lime tree; door at lane (21 000, 5 800).</li><li>`shore` (public, outdoor, water): fishing 60 min; door at lane (12 000, 200), at the shore's (2 000, 9 800), so the shore is at least 10 m deep.</li><li>`pier` (public, outdoor, water): about 4 × 20 m, running **south** from the lane, its foot on its north edge; a bench at its end; fishing 90 min; door at lane (18 000, 200).</li></ul> The lane's comment: "No place here has drawn geometry yet; positions are semantic" | `worlds/lakeside/places/*.yaml` |
| A-18 | **Ten people, all seats**: Alba Ferreira (baker), Bruno Lindqvist (carpenter), Celia Okafor (teacher), Greta Novak (retired), Ines Duarte (nurse), Elin Berg (painter, visitor) live in `cottages`; Dario Russo and Nils Aaltonen (fishermen), Finn Halvorsen (boatbuilder), Hugo Marchetti (ferryman, "rows people across the lake") live in the `boathouse`. Nils starts on the pier (Milestone E M-8). Routines run 05:00–19:00 | `worlds/lakeside/people/*.yaml` |
| A-19 | Items: `fish` (food); everything else from `modern-goods`. No money | `worlds/lakeside/items/`; README |
| A-20 | 2D: no Lakeside layout file. The 2D client lays places out at runtime from disclosed passages (`town.gd`, `ARC-45`). Façades and footprints come by tag (`places.gd`, `asset_bindings.yaml`). **No Lakeside tag has a façade binding** (boathouse falls to `facade:residential` → an apartment block; bakery, cottages and others to `facade:unknown:n`), and `PLACE_NAME_KEYS` (`places.gd:79-80`) knows only cafe, park, store, apartments, workplace and street | `clients/2d/scripts/{town.gd, scene/places.gd}`; `presentation/mineworld-default/2D/assets/asset_bindings.yaml:77-81` |
| A-21 | Planned and approved for Lakeside elsewhere: S26-a adds `appearance` lines; S29-a adds the halt (`station`, `station-tickets`, `station-platform`, joined to `lane`), `settlement: { kind: town, station: station, landmark: pier }` and the `landmark` tag on `pier`; S29-c fills the far-field slots; S29-f draws the halt in 2D; S30-g adds attributes; S32-f adds starting wardrobes; S31 leaves Lakeside's baselines byte-identical (no economy) | step-26 §12; step-29 §§4.3, 6.7, 8.1, 11; step-30; step-31 AC-S31-9; step-32 §16 |
| A-22 | Tests that pin Lakeside: `tools/cli/tests/milestone_e.rs`, `tools/cli/tests/lakeside/{mod,history}.rs`, `scripts/baselines.txt` | grep |

### 2.3 Findings

- **F-S33-1. S33 needs the per-world part of 16f, and 16f is undesigned and inconsistently placed.**
  `overall.md` (QPL-17, "Physics list" decision 3) puts **16f in MVP-0**: "the 3D client reads a
  Presentation Pack and builds places from the server's disclosure". Step-29 §11.2 and RL-e0 SD-FF-7 call
  16f **MVP-1**. No 16f design exists. Lakeside in 3D needs exactly one slice of it: selecting the world, and
  binding a world's places to their drawn depictions without a hard-coded key list. S33-c designs that
  slice as the seed of 16f (SD-S33-6, Q-S33-4). The contradiction is recorded for the primary session to
  resolve in `overall.md` (`CLAUDE.md` §2.1 rule 4).
- **F-S33-2. RL-e0's Lakeside far field puts the lake on the wrong side.** App. B.2/B.3 put the lake at
  300°→60° (north). The world says the shore and the pier are on the lane's **south** side and the pier
  "runs south" (A-17). The 2D client lays the pier out south of the lane from those passages. A lake to the
  north would put the pier on dry land in 3D and contradict 2D. S33-a moves the lake south in both packs,
  and RL-e0's cross-pack water check (AC-FF-4) holds them together.
- **F-S33-3. Lakeside has no bodies.** 12d gives walls to the two towns only, and `bodies` is not in
  Lakeside's `systems:`. Without bodies the server lets a person walk through the bakery wall, off the pier
  or into the lake; only the 3D scene would stop them. That is a rule held in a renderer
  (`CLAUDE.md` §4 rule 15). S33-b adds `bodies` and a `body:` section to every Lakeside place.
- **F-S33-4. Height is presentation; the shore is a slope.** The server is planar (A-8). A lane above the
  water, a shore falling to it and a pier deck above it are client-side heights over one planar footprint.
  That is correct (`CLAUDE.md` §4 rule 14), but the body floors must still end at the water line, so the
  server, not the slope, stops a walk into the lake (§4.6).
- **F-S33-5. The cast bytes are already counted.** E-2 already ships Lakeside's ten people as the café
  street's extras, and the café town's people as the cast. A 3D Lakeside adds **no new character**: its
  extras are the café people already shipped. Its byte increment is environment only (§8.2).
- **F-S33-6. Hugo is a ferryman without a ferry.** No System ferries anyone. A boat rowing across the lake
  would show a behaviour the world does not have. Boats are drawn **moored** (S29 §5.2's rule: nothing
  pretends) (I-S33-5).
- **F-S33-7. Lakeside's 2D is generic.** It runs (A-20), but its façades and place names fall back. "Lakeside
  already exists in 2D" is true of the layout, not of the look (§6, Q-S33-7).
- **F-S33-8. People sitting is not a simulation fact.** The world has no posture (A-10). Seats in 3D must
  therefore be a presentation rule shared by both clients, or a new System (Q-S33-6).

---

## 3. Location and far field (`ARC-143`)

### 3.1 The constraint

Lakeside declares no location. Casting uses the San Diego profile (S26 §7.4a; S30 F-CAST-2: "the café
town, market town and lakeside share San Diego County today"). The realistic-defaults rule wants a real,
cited place whose terrain, water, vegetation and light the far field and the near field can be built from.
QRL-2 keeps an alpine lake out of San Diego (§0.2).

### 3.2 Options (facts read 2026-10-15)

| Option | Facts (source) | For | Against |
| --- | --- | --- | --- |
| **L-1. Lake Cuyamaca (recommended)** | Cuyamaca Reservoir, San Diego County: 32°59′11″ N 116°34′50″ W; surface elevation **4 613 ft (1 406 m)**; **110 acres (45 ha)**; dam completed **1888**, "second-oldest in California still in use"; stocked with trout year-round, also bass, catfish, crappie, bluegill, sturgeon; boat rentals, store, restaurant and tackle shop on site; SR-79 to Julian (north) ([Wikipedia](https://en.wikipedia.org/wiki/Lake_Cuyamaca)). **Cuyamaca Peak 6 512 ft (1 985 m)**, 32°56′48″ N 116°36′24″ W; forest of Jeffrey, Coulter and sugar pine, white fir, incense cedar and black oak, much of it lost in the 2003 Cedar Fire and regrowing ([Wikipedia](https://en.wikipedia.org/wiki/Cuyamaca_Peak)) | <ul><li>A fishing lake with boats, which is what Lakeside's people do (`fishing`, fishermen, a boatbuilder, a ferryman).</li><li>Conifers, oaks, granite and a peak across the water: **06**'s mood, inside San Diego County, without an alpine snowfield.</li><li>The peak is a strong, real far-field form: about 5.0 km at ≈ 209° from the lake's named coordinate, ≈ 580 m above the water, ≈ 6.6° above the horizon (computed from the two coordinates above; S33-a re-derives it from USGS 3DEP for the village's site).</li><li>Real elevation gives real air (1.4 km) and real light.</li></ul> | <ul><li>**No railway ever reached it.** The San Diego, Cuyamaca and Eastern ran from San Diego through Lakeside to Foster from 1889 and never got farther; floods cut it in 1916 ([Foster, Wikipedia](https://en.wikipedia.org/wiki/Foster,_San_Diego_County,_California); [La Mesa Depot Museum](https://psrm.org/visit/la-mesa-depot/)). The halt is a stated counterfactual (Q-S33-2).</li><li>No village stands on the shore today; the village is design content on a real site.</li></ul> |
| L-2. Lindo Lake, Lakeside, CA | Lakeside, CA: 32°50′58″ N 116°54′20″ W, **413 ft (126 m)**; founded 1886 around the natural lake; railway from 1889; El Cajon Mountain ("El Capitan") dominates the north-east ([Wikipedia](https://en.wikipedia.org/wiki/Lakeside,_California)) | <ul><li>The real town named Lakeside, with a real railway from 1889: the halt is historical.</li></ul> | <ul><li>A small suburban park lake today, chaparral, eucalyptus and palms: not **06**'s conifers, granite and deep water, and a weak fishing village.</li><li>Lake dimensions not found in the source read; the pier's "deep water" is doubtful.</li></ul> |
| L-3. Lake Hodges (Escondido) | a reservoir in chaparral (not read in detail for this draft) | real, in the county | coastal sage and chaparral, no conifers; the far field is low hills. It reads as the café town's hinterland, not as its own place |
| L-4. Keep "design content" | RL-e0 B.2 | no work | fails realistic defaults; and F-S33-2 already forces a rewrite |

**Recommendation: L-1, Lake Cuyamaca,** with the village on the lake's **north shore** so that the lake lies
south, as the world's data require (F-S33-2). The halt is the San Diego, Cuyamaca and Eastern's line carried
to the lake, recorded in the far field's `source` as a stated departure from history, the same kind of honest
compression S29 recorded for the depot (Q-S29-1). This is Q-S33-1 and Q-S33-2.

### 3.3 What the references keep and what the setting replaces

| Element | Governed by |
| --- | --- |
| Architecture, materials, light and mood of the village, the pier and the shore | **06** and **01** (QRL-2 unchanged) |
| Terrain, peaks, skyline, water extent, vegetation species, elevation and air | the real setting (L-1), cited |
| The snow peak, the waterfall and the church spire across the water in **06** | **Not copied.** No snowfield and no waterfall exist at Cuyamaca. A spire would be design content on a far shore that has none |

### 3.4 Far field content for L-1 (3D; replaces RL-e0 App. B.2)

Values marked *derive* are computed in S33-a from the cited datasets by RL-b C8's method (the summit table,
earth curvature and refraction k = 0.13) and recorded with their source line. Nothing below is invented
relief.

```text
scope           {"world": "lakeside"}
boundary        natural; reach_m 80 (RL-e0's value; S33-a confirms it contains the halt's
                platform end and the pier head, §4.7); beyond "water" in the lake sector, else
                "planting" (the woods)
datum_m         1408   = lake surface 1406 m (Wikipedia, Lake Cuyamaca) + 2 m (the lane above full pool;
                approximate, S33-b's body floors and S33-d's grading fix it)
sky             panorama "qwantani_puresky_2k" (reused); visibility_m derive: a cited mountain-air range
                for the Cuyamacas (S33-a names the source; RL-e0 used 80 km for the coast)
foreground      "dry_grass" in the open sectors, "planting" ring at the boundary (woods); drop_m small
                (≈ 2–10 m, derive from 3DEP), run_m derive
water           cuyamaca  kind "lake"  sector derive (south of the village, ≈ 120°→240° expected)
                shore_m ≥ reach_m (the far shore and the lake's ends beyond the near field)
                level_m 1406  horizon "closed"  colours: deep/shallow re-picked against 06 in pixels
                source "Lake Cuyamaca (Cuyamaca Reservoir), 45 ha, surface 1 406 m; outline from USGS NHD"
terrain_silhouettes  (USGS 3DEP 1/3 arc-second, read for the village's site)
                near: the lake's far shore and the wooded rises round it
                mid:  Cuyamaca Peak (1 985 m, ≈ 209°, ≈ 5 km), Middle and North Peak (directly north of
                      Cuyamaca Peak; heights derive), Stonewall Peak (derive)
                far:  the Laguna Mountains to the east and the Volcan Mountains to the north
                      (derive; named only if the 3DEP profile shows them above the near terrain)
                every point cited; 360° covered (FAR_FIELD_GAP must not fire)
skyline         sectors: a sparse settlement massing only where real buildings stand within view
                (the lake's own store and restaurant site; Julian is ≈ 10 km north, likely below the
                ridge — derive); density low; seed fixed
                slots: station  filled (place station, the halt's position, §4.4)
                       landmark filled (place pier, the pier's position, §4.3)
                sources cite §3.2 and Q-S33-2's ruling
```

**2D (`presentation/mineworld-default/2D/far_field/lakeside.yaml`).** The same lake sector, level, colours and
source (AC-FF-4), base `land`, haze re-picked. The surround keeps RL-e0's extent.

**The spec change.** No schema change. RL-e0's format 1 with S29-c's `filled` slot carries all of it. The
content change is an intended far-field change under RL-e0's V-gate rule: the Lakeside shots go to the
operator.

### 3.5 Does the world itself declare the location?

Market Town declares a NOAA station for weather (`configure/weather.yaml`). Lakeside has no `configure/`.
Declaring Lake Cuyamaca in world data (a calendar location, a weather station) would change the simulation
and its digests and is not needed to draw the world. **Recommendation: presentation only in S33** (the far
field's sources name the place); a world-data location is a separate, later change owned by the time and
weather packs (Q-S33-11).

---

## 4. The 3D world

### 4.0 Scene frame and plan (SD-S33-1)

The slice's convention: metres, +x east, +z south, y up, y = 0 the lane's surface. The lane's centre is the
origin. Semantic positions convert by the 16c rule (doorway translation, `ARC-45`), so the plan below is
where the world's passages already put things; S33-b's bodies make it exact.

```text
                    N (−z)
      cottages yard     boathouse      bakery        square            halt (S29-a; proposed east end)
   x≈−12               x≈−6           x≈0           x≈+6        ┌─ waiting room 8×5 ─┐ platform 40×4,
  ─────┬──────────────────┬──────────────┬────────────┬─────────┤                     │ track N–S,
       │              lane (30 × 6 m, cobbles), z −3 … +3       │                     │ buffer stop at
  ─────┴───────────┬──────┴─────────┬─────┴──────────────────────┴─────────────────────┘ the south end
              shore (x≈−3, ≥10 m deep,      pier foot x≈+3
              grading to the water line)    deck 4 × 20 m running south over the water,
                                            pavilion and bench at its head (z ≈ +23)
                    ~~~~~~~~~~~~~~~ Lake Cuyamaca (south) ~~~~~~~~~~~~~~~
                                   Cuyamaca Peak ≈ 209°, ≈ 5 km
```

- **The halt's position is S29-a's to decide** (it writes the halt's places, bodies and passages). S33
  proposes the **lane's east end** with a north–south track ending at a buffer stop by the shore: a
  terminus, which is what a branch line's end at a lake is. This is a non-preclusion item for S29-a's PR
  design (§16.2).
- **Spawn** is the observer's disclosed position (Elin, the visitor, starts in `cottages`). There is no
  scene spawn constant for Lakeside.

### 4.1 Terrain and water (SD-S33-2; `DEP-77`, `DEP-78`)

| Element | Specification | Source / status |
| --- | --- | --- |
| Near terrain | One heightfield over the near field (≈ 200 × 200 m, 1 m cells), built by our builder from a small **authored height grid** (graded lane, yard, square, the shore's fall to the water, wooded rises behind the village), shaded with the slice's `leafy_grass`/`forest ground`/`cobblestone_floor_08` by a splat of at most four layers | own work; Godot `ArrayMesh` + `HeightMapShape3D` (DEP-77) |
| Water line | Full pool at y = −2.0 m (the lane 2 m above the lake, §3.4) | approximate; S33-b's floors and S33-d's grading fix it |
| Near lake surface | A world-anchored surface from the shore line to `reach_m`, continuing into RL-e0's far-field water sector at the same level and colours. Shader: `water.gdshader` (RL-e0's version) **extended** with a depth fade at the shore and round the pier posts (from the depth texture), and lily pads as a decal layer near the shore (06). No flow, no foam simulation | own shader, MIT (DEP-78) |
| Reflections | (a) a reflection probe over the near lake and the sky radiance (static, cheap); (b) SSR as a measured option for the pier views. **Planar reflection** (a second camera) is rejected: it doubles the scene's draw cost (§8.1) | measured in S33-d |
| Shore | Granite boulders (the spike's and Poly Haven's `boulder_01`, CC0), a packed-earth trail (06), reeds at the water line (procedural cards) | reuse |
| Lane | `cobblestone_floor_08` setts (01), a low stone kerb, a timber railing where the lane is above the shore | reuse |

### 4.2 Vegetation and the far-field ring (SD-S33-3; `DEP-79`)

- **Species follow the setting** (L-1): Jeffrey pine and Coulter pine, incense cedar, black oak on the slopes
  (Cuyamaca Peak, Wikipedia). The square's **lime tree** stays, because the world says so; it reads as a
  planted village tree.
- **Method:** the spike's procedural tree builder (`props.gd`, `procgen.gd`: card canopies, needle and leaf
  textures generated at load, zero texture bytes), moved into the slice's builder and tuned per species. One
  `MultiMesh` per species and LOD band, so the woods cost a handful of draws.
- **The woods ring is the boundary.** Dense planting from about 60 m to `reach_m` closes the land sectors
  (`beyond: planting`), with air walls behind the first rank (allowed, §0.3). Beyond `reach_m`, RL-e0's
  foreground and silhouettes take over. Nothing past the boundary suggests a path.
- **Ground cover:** wildflowers and grass as cards (06's daisies and lupins), in clumps along the trail and
  the shore, budgeted per view.

### 4.3 The pier: the landmark (SD-S33-4)

| Element | Specification | Source / status |
| --- | --- | --- |
| Deck | 4 m wide, 20 m long, south from the lane (A-17); timber planks (`weathered_brown_planks`), round posts in pairs at ≈ 3 m, cross-bracing below the deck | world data; reference 06 |
| Deck height | ≈ 0.6 m above full pool | approximate; S33-e cites a fixed-dock design source or replaces it |
| Rails | Fishing-pier rails **no higher than 34 in (865 mm)** so that a seated angler can fish over them, with edge protection along the deck | 2010 ADA Standards §1005 (fishing piers); S33-e quotes the clause |
| Pavilion at the head | A small timber shelter (≈ 4 × 4 m, gabled, shingled), the world's **bench at the end** inside it, a lantern on a post (06) | S29 Q-S29-3 (presentation only) |
| Boats | Two or three rowing boats moored at the head and at the boathouse slip, **never moving** (F-S33-6) | reuse: the spike's rowing boat |
| The landmark check | LM-1 and LM-2 (S29 §9.2's method, `mw_category = "landmark.*"`): the pier and pavilion close the southward view from the lane, and are seen from the shore and the square | §9.2 |

### 4.4 The halt (SD-S33-5)

The halt is S29's design at village scale (S29 §4.3: waiting room ≈ 8 × 5 m, a ticket window bay, a
platform ≈ 40 × 4 m). S33-e builds it in 3D with S29-e's pieces:

- the shared `clients/shared/station/station_state.gd` (S29 §5.1): the board reads "No service yet", the
  window is shut, the kiosk shut (Lakeside has no economy);
- the building: a small timber-framed waiting room in the village's idiom (board-and-batten walls, a
  shingled roof with a deep canopy over the platform side), **not** the Santa Fe Depot: the depot is
  Market Town's and Social Café's landmark; Lakeside's landmark is the pier;
- the platform with a track and **a buffer stop** (a terminus), no train (S29 §5.2);
- benches in the waiting room and on the platform (sit points, §4.5).

### 4.5 Interiors, seats and sit points (`ARC-145`; Q-S33-5, Q-S33-6)

**Interiors.** An interior is drawn where the world has an indoor Place a person enters:

| Place | Interior | Fit-out (room, furniture, hand scale; VISUAL_SLICE §6.2) |
| --- | --- | --- |
| `bakery` | **yes** | oven wall, counter, bread racks, the world's **long table by the window**; loaves and baskets |
| `boathouse` | **the ground-floor boat shed** | a boat on the slip, workbench, oars, nets; the stair to the rooms above is drawn and **gated** (the world has no upstairs place) |
| `station` (halt) | **yes** | per §4.4 |
| `station-tickets` | **the window bay** | per S29 §5 |
| `cottages` | **no** | the place is the yard with its pump. The cottage doors are closed; the world has no cottage interior place. Closed doors on a reachable yard are not "a building the player can see but never reach" |

**Sit points (presentation).** A sit point is pack data: `{position, facing, seat_height_m, approach}` on a
named seat (bench, chair, the pier bench). Seat heights follow `VISUAL_SLICE.md` §3.4 (0.42–0.47 m) unless the
object's source says otherwise.

**Who sits (SD-S33-7).** The world has no posture (F-S33-8). The rule is written **once**, in the shared
client module (`clients/shared/`), and both clients apply it:

```text
a disclosed person is drawn seated at sit point s  iff
  the person's disclosed position lies within s.approach (≤ 0.40 m of the seat's front edge),
  the person has not moved for ≥ 2 s of world time,
  and s is not already drawn occupied (ties broken by entity id, so every client agrees)
```

The rule chooses a **pose**; it decides nothing the world owns. Two clients and the agent view therefore agree
on where a person *is*; whether they look seated is presentation, and both reference clients compute it the
same way. A future `posture` System that discloses sitting overrides the rule (non-preclusion). The player sits
the same way: standing still at a sit point.

**Seat contact.** G-13 (S26 §6.1) and its modular restatement (S32 §7.1) gate each character's sit pose at
± 20 mm of the seat height. S33 adds the **scene** side: every Lakeside sit point, with every Lakeside cast
body and every extra, seats the pelvis within ± 20 mm of `seat_height_m` (AC-S33-3).

### 4.6 Navigation and collisions (12d/12e conventions)

- **The server decides where a person can be.** S33-b gives every Lakeside place a 12d `body:` section at real
  dimensions: the lane, yard, square, shore and pier floors; the bakery, boathouse and halt walls and
  furniture as solids (≤ 64 per place); the pier floor ending at its deck edges; the shore floor ending at the
  water line. A stride into the lake or off the pier is stopped by `bodies`, not by the scene.
- **The client mirrors disclosure.** 12e's `DisclosedGeometry` (invisible) is the collision truth for walls;
  the drawn scene adds only presentation colliders (steps, ramps, the terrain's height, the rails) and the air
  walls at the woods. A check compares drawn walls with disclosed walls (X-8; R-B4).
- **Height is presentation** (A-8, F-S33-4). People stand on the terrain the client draws, found by a ray to
  `LAYER_WORLD`; the server never sees a height. The step from lane to pier deck is a ramp ≤ 0.25 m per step.
- **No navmesh.** NPCs move by the server's routes (`movement`, `bodies`' route planner); NW-3's bound (≤ 10 %
  of walks stopped by geometry) is checked on Lakeside's walks in S33-b.

### 4.7 Boundary and reach

`reach_m` 80 contains: the lane (±15 m), the yard, square and bakery (≤ 25 m north), the pier head (≈ 23 m
south), the halt (≈ 15–35 m east, platform ≈ 40 m along z). S33-a confirms the number against S29-a's halt
position, and raises it only if the platform's far end lies outside. Air walls stand at the woods; the lake is
closed by the bodies floors and a low rail where the lane is above the water.

### 4.8 One server-authoritative path; semantic places apart from geometry (`ARC-144`)

- **Same path.** `./mineworld-slice --world=lakeside` hosts `mineworld server worlds/lakeside` exactly as
  `--world` hosts `social-cafe`; `--server=host:port --invite=…` joins any server and requires
  `--world-key=lakeside` (RL-e0 SD-FF-7's argument, now in 3D). The client sends only `ActionIntent`s.
- **The per-world seam (SD-S33-6).** The client gains a registry from **world key** to a **world depiction**:
  a builder module with one interface, which `SliceWorld` (social-cafe) already almost is:

  ```text
  build(root) · place_volumes() · depictions(): [{tag, volume, door}]   (what is drawn as which tagged place)
  viewpoints(): far-field, landmark, perf and probe viewpoints          (§9)
  sit_points(): [SitPoint]                                              (§4.5)
  ```

  Binding is by **tag** and doorway coincidence (16c's D-16c-3 rule generalised): the Lakeside depiction
  declares that its bakery room depicts the place tagged `bakery`, and so on. The client keeps **no list of
  the world's place keys** (16c SC-4); a key scan holds it to that (X-6).
- **16f.** This seam is the Lakeside-needed part of 16f. 16f, when designed, generalises it to depictions
  loaded from a Presentation Pack and walls built from disclosure; S33-c's interface is written to be that
  seam's first implementation (Q-S33-4).

### 4.9 Reuse, both ways (`REUSE_POLICY.md`; `DEP-77…DEP-79`)

| Need | Options compared | Decision | Why |
| --- | --- | --- | --- |
| Near terrain | **Terrain3D** (TokisanGames; MIT; C++ GDExtension; up to 32 textures, 10 LODs, sculpting) · **HTerrain** (Zylann `godot_heightmap_plugin`; GDScript; "Godot 4.6+"; maintained for bug fixes only) · **Godot built-ins** (`ArrayMesh` from a height grid, `HeightMapShape3D`) | **Built-ins, our builder** (`DEP-77` records both rejections) | The near field is ≈ 0.04 km². Terrain3D's strengths (large worlds, painting tools) do not apply, and a GDExtension adds per-platform binaries to three export targets. HTerrain is in maintenance mode. Neither changes the far field, which RL-e0 draws as silhouettes. Revisit at S21's region scale. Facts to re-verify at S33-d: Terrain3D's platform list and Godot 4.7 support; HTerrain's licence file |
| Water | **Our `water.gdshader`** (exists; RL-e0 extends it) · community Godot 4 water shaders (ocean FFT and stylised lake shaders on the Godot Shaders site and GitHub) | **Ours, extended** (`DEP-78` records the comparison) | A still lake needs normals, a depth fade, Fresnel and colour, which ours has or gains in ≈ 40 lines. FFT oceans solve waves we do not have. S33-d records each candidate's licence and Godot 4.7 status, and adopts one only if a reviewed frame shows ours cannot read as 06's water |
| Trees, ground cover | **The spike's procedural cards** (`props.gd`, `procgen.gd`) · Poly Haven CC0 tree models (`tree_small_02`, `jacaranda_tree`: ASSETS.md notes the look is right but the cost is high for thirty trees) · CC0 low-poly nature packs (a second style) · commercial tree packs and generators (paid) | **The spike's cards, reused** (`DEP-79`); Poly Haven models only for one or two hero trees if a reviewed frame needs them | Zero texture bytes, one method, already the project's. Paid packs are an operator question with no recommendation to buy (Q-S33-7) |
| Buildings | **Our builder** (S29 §6.1: one method) | build ours | the town's one method |
| Rocks, boats, jetty | the spike's and Poly Haven's CC0 | reuse | already shipped |
| New CC0 textures | at most four Poly Haven 1k sets (bark, forest ground, granite, shingle) only if a reviewed frame shows the shipped sets cannot read as the material; each with a DEP-8 row; ≤ 3 MB each | `DEP-79` holds them | the byte budget (§8.2) |
| Generated 3D (Meshy) | — | **not used** for environment | a second method (S29 §6.1) |

---

## 5. People (S26, S32; E-2)

- **The cast.** Lakeside's ten people are drawn as their own characters: each person's `look` (S26-a) binds
  their modular body (S32-c), wearing their starting wardrobe (S32-f). The binding, fallback and extras rule are
  S26 §4.4's, unchanged.
- **The extras are the café town's people** (E-2 in the other direction, S26 §8). No Lakeside person ever
  appears as an extra in Lakeside (the pools are disjoint). Decor figures in Lakeside are few (≤ 6: walkers on
  the shore trail, a person on the square's bench) and sit only if they passed G-13.
- **No placeholder figure.** A disclosed person with no bound look draws an extra; never CharMorph, never a
  capsule (S26 X-16).
- **Dependency on the S32 wardrobe pilot.** S32-p decides whether route B reaches the Vera look. Lakeside's
  bodies and garments come in S32-d's batches. If S32-p's route is not adopted, S32 researches it (S32 §5.2)
  and S33-g waits; the environment PRs do not (§12.2). Lakeside's people at the release are exactly as good as
  the café town's: one standard.
- **Activities shown, not invented.** A person fishing is drawn at the water's edge facing the water, with a
  rod prop, **only while** the world discloses their fishing Process (the `fishing` pack's Process; S33-g
  audits the exact disclosure). No one is shown rowing (F-S33-6).

---

## 6. 2D parity

Lakeside already runs in 2D (A-20). These must stay in sync with 3D:

| Item | Kept in sync by |
| --- | --- |
| Places, passages, doorways | one World Pack; both clients bind by disclosure. The `SHOWN` report (13a/16c) of each client on Lakeside lists the same places and doorways (AC-S33-7) |
| The lake's bearing, level and colours | the two far-field files, held together by RL-e0's AC-FF-4 (F-S33-2) |
| The halt and the pier | S29-f's halt binding and pier pavilion prop in 2D; S33-e in 3D; one `station_state.gd` |
| People and outfits | S26-e/S32-e layers in 2D; S32-c in 3D; one look and one outfit per person |
| Sit rule | the shared module (§4.5). The 2D client applies the same rule; it may draw seated as its sprites allow |
| Place names | the shared locale layer gains `place.bakery`, `place.boathouse`, `place.cottages`, `place.square`, `place.shore`, `place.pier`, `place.lane` in every shipped locale; `PLACE_NAME_KEYS` reads them |
| **Façades** | **Gap today** (F-S33-7): `facade:bakery`, `facade:boathouse`, `facade:cottages`, `prop:well`, `prop:lime-tree` bindings to new `sib_*` images from the existing 2D pipeline (OpenAI image edits, 64 px per metre, provenance sidecars), with the operator's contact sheet. This costs OpenAI images (Q-S33-7) |

---

## 7. Agents

- An agent plays Lakeside over MCP (S27-d stdio) through the same server and the same Observations. Nothing in
  S33 is agent-specific; the check is that it works on this world.
- **Scenario** (`clients/agent`, after S27-d and S29-g): an agent seat (Nils) observes on the pier
  (`roles` contains `landmark`), calls `walk_to` the halt, observes `roles` `station`, sees **no offers** at the
  ticket window, and walks back. A 3D client connected to the same server draws Nils's person moving along the
  lane, and the server's place changes match in both.

---

## 8. Budgets

### 8.1 Performance (RL-b R-2; RL-b's three-run protocol on the operator's M5)

**New views** in `SlicePerf` for `--world-key=lakeside` (absolute gates: p95 ≤ 16.7 ms, video memory
≤ 2 048 MB binding; draw calls and primitives reported, advisory):

| View | Why |
| --- | --- |
| `lane wide` | the village frontage, the lake, the woods |
| `pier head looking north` | the whole village across the water: the most geometry in one frame |
| `shore across the lake` | water, reflections and the far field |
| `square` | the lime tree, benches, people |
| `bakery interior` | interior and threshold |
| `boathouse shed` | interior |
| `halt platform` | the platform, track, the halt's canopy |
| `lakeside crowd` | the ten people and the decor figures, half in frame at 3–15 m (S26 §11's crowd view) |

**Estimate (stated as such):** the village is ≈ 10 buildings, the woods a few `MultiMesh`es, the water two
surfaces, the far field RL-e0's ≈ 10 draws. Expected well under the café street's 614–4 215 draws. The risks are
water fill rate with SSR, card overdraw in the woods, and the crowd.

**Levers, built in:** `SliceBatch`, `SliceBudget` and `SliceOccluders` over the village; `MultiMesh` per
species and LOD band; visibility ranges on ground cover; interiors culled from exterior views (the café's
pattern); SSR off unless measured affordable. Stops: a p95 above 16.7 ms in a new view is a **material stop**
with the levers measured.

**Measurement:** locally counts only (R-LOAD-1); the operator's three-run protocol on the S33-d, S33-f and
S33-g heads.

### 8.2 Release size and packaging (`ARC-146`; Q-S33-3)

**Increment.** No new characters (F-S33-5). New data: builder scripts, the height grid, sit points, at most
four CC0 texture sets (§4.9). **Target ≤ 12 MB, gate ≤ 16 MB**, measured by R-a's export.

**How a second 3D world ships:**

| Option | What | For | Against |
| --- | --- | --- | --- |
| PK-1 | one Presentation Pack, one `3d.pck` holding both worlds | simplest; no loader change | `3d.pck` is already over its 250 MB limit (A-13); "a world without Lakeside's 3D content" is not testable as a missing file |
| **PK-2 (recommended)** | one Presentation Pack (`mineworld-default/3D`, which already holds Lakeside's far field), **two export files**: `3d.pck` (the client, shared assets, the cast, social-cafe) and `3d-lakeside.pck` (Lakeside's builders, height grid, sit points and its own textures), loaded at start with `ProjectSettings.load_resource_pack` when `--world-key=lakeside` | keeps each file inside a budget; makes removal a real, testable fact (X-5); the pattern every later world follows | R-a's `scripts/package.py` learns per-world exports (small) |
| PK-3 | a separate Presentation Pack, `mineworld-lakeside-3d`, depending on `mineworld-default-3d` | the purest "framework, not demo" form | needs multi-pack loading in the 3D client (16f) and an amendment to RL-e0's rule that the far field lives in the required pack; larger than S33 needs now. It is where 16f leads |

With PK-2, a client asked to present `lakeside` without `3d-lakeside.pck` refuses by name
(`PRESENTATION_WORLD_MISSING { world: lakeside }`, exit 3 in a scripted mode), as RL-e0 refuses a missing far
field. There is no fallback depiction.

---

## 9. Checks

### 9.1 Viewpoints (scene metres; S33-a fixes the final values from S33-b's bodies)

| Id | Where | Why |
| --- | --- | --- |
| VP-L-1 spawn | Elin's disclosed start in `cottages`, eye height | where a player begins |
| VP-L-2 lane | (0, 1.66, 0) | the lane's centre |
| VP-L-3 pier head | (3, deck + 1.66, 22) | the farthest reachable point over the water |
| VP-L-4 shore | the shore's water line | the lake at its edge |
| VP-L-5 square | the well | the north side |
| VP-L-6 halt | the platform's south end | the reach's east limit |
| VP-L-7 high | (0, 40, 0) | RL-e0's elevated case |

### 9.2 Rendered checks (same harness as RL-e0 and S29)

- **Far field (AC-S33-1):** RL-e0's `--slice-farfield` with `--world-key=lakeside` at VP-L-1…7, **8 azimuths**
  each, S/H/E frames, FF-BELOW ≥ 90 % and FF-ABOVE ≥ 25 % (the lake sector is `closed`, so FF-ABOVE applies
  everywhere). Verdict lines unchanged.
- **Landmark (AC-S33-6):** S29's `--slice-landmark` with H hiding `mw_category = "landmark.*"`: from VP-L-2
  looking south (LM-1: the pier and pavilion close the view) and from VP-L-4 and VP-L-7 (LM-2: the footprint
  differs from H in ≥ 60 % of its pixels).
- **Shots:** `clients/3d-spike/shots/lakeside/{farfield,landmark,review}/`, committed, to the operator under
  `VIS-3D-LAKESIDE`.

### 9.3 Walk, seat and removal probes

| Probe | Asserts |
| --- | --- |
| `--world=lakeside --link` route | spawn → lane → pier foot → **pier head** (place `pier`) → lane → **halt** (`station`) → `station-tickets` → `station-platform` → back; the server's place changes at each doorway; **0 refusals**; the board reads `station.no_service`; NW-3 ≤ 10 % geometry stops |
| `--world=lakeside --seat` | for each sit point and each body in the Lakeside cast and extras pool: the pelvis within ± 20 mm of `seat_height_m`; the rule's tie-break is identical in two clients |
| removal | §9.4's AC-S33-4 rows |
| `SHOWN` parity | the 2D and 3D `SHOWN` reports on Lakeside list the same places and doorways |
| disclosed vs drawn walls | every drawn wall of a Lakeside room lies within 50 mm of a disclosed solid face (12e geometry); no drawn wall without a disclosed one |

### 9.4 Planted mutations (each must fail by name, then be reverted)

| Id | Plant | Must fail with |
| --- | --- | --- |
| X-1 | delete the lake's water sector from the 3D far field | `FAR_FIELD_GAP` at validate, or `FF-BELOW` in the south sectors at VP-L-3 |
| X-2 | move the 3D lake to the north only | RL-e0's AC-FF-4 cross-pack water check |
| X-3 | remove the pier ↔ lane passage | S29's landmark reachability check (I-S29-4) at load, and the link route |
| X-4 | raise one sit point by 50 mm | `--seat` names the point and the body |
| X-5 | delete `3d-lakeside.pck` | `social-cafe` still runs and passes its probes; `--world-key=lakeside` refuses `PRESENTATION_WORLD_MISSING`; headless `lakeside` runs with an unchanged digest |
| X-6 | add the literal `"bakery"` as a key in the client's binding | the client key scan (16c SC-4) |
| X-7 | animate a rowing boat across the lake | the honesty scan: no moving prop without a disclosed Process |
| X-8 | add a drawn wall in the bakery with no disclosed solid | the disclosed-vs-drawn wall check |
| X-9 | hide the pavilion and pier | `FAIL landmark … LM-1` |

### 9.5 The agent scenario

§7's scenario, over MCP, in the `clients/agent` suite.

### 9.6 Where it runs

The rendered checks and probes run in the nightly `clients` and `clients-probes` layers on **ubuntu-24.04,
macos-26 and windows-2025** (RL-e0 §6.5's harness), and in RL-e0's per-PR Linux job, path-filtered.

---

## 10. Invariants (proposed)

| Id | Invariant |
| --- | --- |
| I-S33-1 | **No kernel, contract, protocol or System change.** Lakeside gains the existing `bodies` System in its `systems:` and `body:` sections in its place files; everything else is presentation. The diff in `kernel/`, `contracts/` and `systems/` is empty |
| I-S33-2 | **Social Café is unchanged** by S33: its shots are pixel-equal within frame_diff's tolerance and its probes pass |
| I-S33-3 | The 3D client keeps no list of any world's place keys; binding is by tag and doorway |
| I-S33-4 | Every Lakeside viewpoint closes on a far field (RL-e0's rule) |
| I-S33-5 | Nothing pretends: no train, no rowing ferry, no open ticket window, no seated pose without a person at the sit point |
| I-S33-6 | One art style, one method; every new external file passes DEP-8 |
| I-S33-7 | Real dimensions with citations; every departure (the halt's history, approximate heights) is stated where it is used |
| I-S33-8 | §8's performance and size caps hold |
| I-S33-9 | Walls are the server's: a walk the server refuses is not allowed by the scene, and the reverse |

---

## 11. Step acceptance (fixed before measuring)

| Id | Criterion |
| --- | --- |
| AC-S33-1 | Lakeside's far-field check passes at VP-L-1…7 × 8 azimuths on Linux, macOS and Windows |
| AC-S33-2 | The walk route spawn → pier head → halt → platform passes with 0 refusals and server place changes (§9.3) |
| AC-S33-3 | Seat contact within ± 20 mm at every sit point for every Lakeside cast body and every extra |
| AC-S33-4 | **A world without the Lakeside 3D pack still runs**: headless `lakeside` digest unchanged with and without the file; 2D `lakeside` runs; 3D `social-cafe` runs and passes; 3D `lakeside` refuses by name (X-5) |
| AC-S33-5 | Every Lakeside perf view p95 ≤ 16.7 ms and video memory ≤ 2 048 MB on the operator's M5 (three runs); counts recorded |
| AC-S33-6 | The pier passes LM-1 and LM-2 |
| AC-S33-7 | 2D and 3D `SHOWN` reports agree on Lakeside; place names and façades bound in 2D |
| AC-S33-8 | The MCP agent scenario passes, with a 3D client observing |
| AC-S33-9 | `3d-lakeside.pck` (or the ruled packaging) ≤ 16 MB; the release within the ruled `3d.pck` limit |
| AC-S33-10 | X-1…X-9 each fail by name |
| AC-S33-11 | The operator walks it (`VIS-3D-LAKESIDE`): enter at Elin's seat, walk to the pier head, look back at the village, walk to the halt and read the board, enter the bakery, sit on the pier bench |

---

## 12. PR split (MVP-0, release-blocking)

### 12.1 The PRs

```text
RL-e0 merged ─┐
S29-a merged ─┼─► S33-a setting + far field (data) ─────────────────────┐
S29-c merged ─┘                                                         │
12d merged ──► S33-b Lakeside bodies (sim content, one re-baseline) ────┤
16c merged ──► S33-c 3D world selection + per-world seam (16f seed) ────┼─► S33-d terrain, water, village exterior, woods ─┐
                                                                        │                                                  ├─► S33-e pier landmark + halt (3D)
12e merged ─────────────────────────────────────────────────────────────┘          S29-e merged (station_state, halt pieces) ┘        │
                                                                                                                                        ▼
                                                                                        S33-f interiors + sit points (shared rule) ─────┤
S32-c, S32-d (Lakeside batch), S26-c2 ─────────────────────────────────────────────────► S33-g people (cast, extras, activities) ───────┤
S29-f, S33-a, S33-c ───────────────────────────────────────────────────────────────────► S33-h 2D parity (façades, names, far field) ──┤
S27-d, S29-g ──────────────────────────────────────────────────────────────────────────► S33-i agents (MCP scenario) ────────────────┤
R-a, S32-h ────────────────────────────────────────────────────────────────────────────► S33-j packaging, perf session, checklist ◄──┘
```

| PR | Scope | Depends on | Integration checkpoint | Adversarial criteria | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S33-a** setting and far field | C1 spec: `ARC-143`; a dated amendment note on RL-e0 App. B.2/B.3 and S29 §6.7. C2: 3DEP and NHD reads by RL-b C8's method, the profile table with sources. C3: `far_field/lakeside.yaml` in 3D and 2D, lake south, slots filled | RL-e0 merged; S29-a (halt place); S29-c (`filled`); Q-S33-1/2 ruled | `mineworld validate worlds/lakeside --packs …` passes; the far-field check runs on a stub Lakeside depiction (S33-c) or on RL-e0's harness | X-1, X-2 | **Haiku** (data by a fixed method); **Opus** reviews the profile against the map |
| **S33-b** Lakeside bodies | `bodies` in `systems:`; `body:` on all Lakeside places at real dimensions (the halt's from S29-a); NW-3 walks; one recorded re-baseline of Lakeside's digests and Milestone E expectations | **12d merged**; S29-a | `mineworld run worlds/lakeside 30d` seeded: no person in the lake or off the pier; save, restart and resume identical; `milestone_e` green with recorded new values | a stride off the pier deck is stopped by `bodies`; removing `bodies` from `systems:` restores the old digests | **Haiku** from a commit plan (12d's pattern) |
| **S33-c** 3D world selection and the per-world seam (`ARC-144`) | `--world=lakeside`, `--world-key` in 3D; the world registry; `SliceWorld` refactored behind the interface with **no visual change**; a Lakeside depiction stub (flat ground, boxes at the places' footprints) bound by tag; `SHOWN` on Lakeside; probe/perf viewpoints per world | **16c merged**; RL-e0 merged | social-cafe shots pixel-equal (I-S33-2); `--world=lakeside --link` walks the stub with server place changes | X-6; a world key with no depiction refuses `PRESENTATION_WORLD_MISSING` | **Opus** (architecture; 16f's seed) |
| **S33-d** terrain, water, exterior | the height grid and terrain mesh; the near lake and shader extension; shore, lane, trail, boulders; cottages, bakery and boathouse exteriors; the woods (`MultiMesh` cards); GI volume and probes; far-field views; `farfield` shots; perf views | S33-a, S33-b, S33-c; **12e merged** (disclosed walls) | AC-S33-1 on three OSes; counts within §8; first `VIS-3D-LAKESIDE` preview to the operator (ARC-24) | X-1 holds; X-8 | **Opus** (art judgement); mechanical parts to **Haiku** |
| **S33-e** pier and halt | pier, rails, pavilion, moored boats; the halt building, window bay, platform, track and buffer stop with S29-e's shared pieces; `--slice-landmark` for Lakeside | S33-d; **S29-e merged** | LM-1/LM-2 pass; the route reaches the pier head and the platform; the board reads `no_service` | X-3, X-7, X-9 | **Opus** for the pier; **Haiku** for the halt (S29-e pattern) |
| **S33-f** interiors and sit points (`ARC-145`) | bakery and boathouse-shed interiors; sit points in all seats; the shared sit rule in `clients/shared/`; `--seat` probe; threshold light at the bakery door | S33-d; S26-c1/S32-c bodies for the probe | AC-S33-3 on the bodies available; interiors walkable, server place changes | X-4; two clients agree on a contested seat | **Haiku** from a commit plan; **Opus** frame review |
| **S33-g** people | Lakeside's cast bound in its own world; extras = café cast; ≤ 6 decor figures; the fishing pose tied to the disclosed Process; the `lakeside crowd` perf view | **S32-c merged**; S32-d's Lakeside batch accepted; S26-c2 | every Lakeside person is their own character in their outfit; an unknown person is an extra | S26 X-16 on Lakeside; no Lakeside person drawn as an extra | **Haiku** |
| **S33-h** 2D parity | façade and prop bindings (new `sib_*` by the existing pipeline, operator contact sheet); place-name keys in every locale; the 2D far field's lake; `SHOWN` parity test | S29-f; S33-a; S33-c; Q-S33-7 (images) | `./mineworld-2d` on Lakeside: named places, bound façades, the halt and pavilion; parity test green | the 2D far-field clear-colour check; a missing binding falls back, never crashes | **Haiku** (pipeline run) |
| **S33-i** agents | the §7 scenario in `clients/agent` over MCP with a 3D observer | S27-d; S29-g; S33-c | AC-S33-8 | an agent away from the halt sees no `roles: station` exit leading there | **Haiku** |
| **S33-j** release and perf | packaging per Q-S33-3 (PK-2: per-world export in `scripts/package.py`); the size gate; the operator's perf session; the hands-on checklist item | R-a merged; S32-h; S33-d…g | AC-S33-4, -5, -9; X-5 | X-5 | **Haiku** |

### 12.2 Start conditions and order

- **Start.** As S29: nothing starts before Milestone F's checklist is handed to the operator, and after the
  step is approved.
- **First wave (parallel worktrees):** S33-a (once RL-e0, S29-a and S29-c merge), S33-b (once 12d and S29-a
  merge), S33-c (once 16c and RL-e0 merge).
- **Environment line:** S33-d → S33-e → S33-f. These do not wait for people.
- **People:** S33-g waits for S32's route and Lakeside batch, and is the **critical path** with S32.
- **Release:** all ten before S23's first public release candidate (MVP-0, release-blocking).

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S33-1 | RL-e0, 12d, 12e or 16c slip; S33's first wave waits | S33-c's refactor and S33-a's data reads can be prepared on the frozen designs; nothing merges before its parent |
| R-S33-2 | Water with SSR breaks 16.7 ms at the pier views | probe + radiance first; SSR only if measured affordable; planar reflection rejected |
| R-S33-3 | Card trees read as cheap beside 06 | the operator sees the woods early (S33-d's preview, ARC-24); the lever is one or two Poly Haven hero trees and one CC0 bark set, not a second method |
| R-S33-4 | S32's route B fails the pilot; Lakeside's people are late | S33-g is the only PR that waits; the environment lands; the release waits for people, as the operator's principle requires |
| R-S33-5 | The halt's departure from history is judged wrong | Q-S33-2 asks it now; L-2 (Lindo Lake, a real railway town) is the alternative |
| R-S33-6 | 16f, when designed, takes another shape than S33-c's seam | S33-c is written as 16f's seed, reviewed by the primary as such (Q-S33-4) |
| R-S33-7 | `3d.pck` is over its limit before Lakeside | PK-2 keeps Lakeside's bytes in their own file; the limit question stays Q-S26-5's |
| R-S33-8 | The 3DEP profile shows the peak hidden by near ridges from the village site | the site on the north shore is chosen in S33-a with the peak's visibility checked first; the far field shows what is really visible |
| R-S33-9 | S33-b's re-baseline collides with S29-a's, S30-g's and S32-f's Lakeside changes | one recorded re-baseline per PR, rebased last, scheduled by the primary (Q-S33-8) |

---

## 14. Questions for the operator (each with a recommendation; all ruled 2026-10-15)

| Id | Question | Recommendation | Cost | Ruling |
| --- | --- | --- | --- | --- |
| **Q-S33-1** | **Where is Lakeside?** Lake Cuyamaca (L-1), Lindo Lake in Lakeside, CA (L-2), Lake Hodges (L-3), or keep design content (L-4)? | **L-1, Lake Cuyamaca**, village on the north shore, the lake to the south. It has the fishing, boats, conifers, granite and a peak across the water that 06 shows, inside San Diego County, with real elevation (1 406 m) and cited terrain | none (public USGS data) | **Ruled 2026-10-15** (operator answered directly in the primary session): L-1, Lake Cuyamaca; the village on the north shore, the lake to the south. |
| **Q-S33-2** | **The halt's history.** No railway ever reached Lake Cuyamaca (the San Diego, Cuyamaca and Eastern stopped at Foster, 1889–1916). Record the halt as that line carried to the lake, a stated departure in the far field's `source`? | **Yes**, stated as a departure, like the depot's compressed distance (Q-S29-1). If a historical halt matters more than the setting, choose L-2 instead | none | **Ruled 2026-10-15** (operator answered directly in the primary session): the halt is a stated departure from history, recorded in the far field's `source`. |
| **Q-S33-3** | **How does a second 3D world ship?** One `3d.pck` (PK-1), one pack with a per-world export file (PK-2), or a separate Presentation Pack (PK-3)? | **PK-2**: `3d-lakeside.pck` beside `3d.pck`. It keeps each file inside a budget, makes "a world without the Lakeside 3D pack" a real test, and is the pattern later worlds follow. PK-3 comes with 16f | R-a's packaging script grows slightly | **Ruled 2026-10-15** (operator answered directly in the primary session): PK-2: `3d-lakeside.pck` ships beside `3d.pck`. |
| **Q-S33-4** | **16f.** `overall.md` has 16f in MVP-0 (QPL-17); S29 and RL-e0 say MVP-1; no design exists. May S33-c build the per-world seam (world selection, depiction by tag) as 16f's seed now, and the primary resolve 16f's placement in `overall.md`? | **Yes.** S33-c delivers only what Lakeside needs and is written as 16f's first implementation | none | **Ruled 2026-10-15** (operator answered directly in the primary session): yes: S33-c builds the per-world seam as 16f's seed; the primary session fixes 16f's placement in `overall.md` (MVP-0). |
| **Q-S33-5** | **Interiors:** the bakery, the boathouse's boat shed and the halt; the cottages' doors closed (the world has no cottage interior)? | **Yes.** An interior exists wherever the world has an indoor place. Adding cottage interiors would need new places, a simulation change | none | **Ruled 2026-10-15** (operator answered directly in the primary session): interiors only where the world has an indoor place (bakery, boathouse shed, halt); cottage doors closed. |
| **Q-S33-6** | **Sitting.** A shared presentation rule (a still person at a sit point is drawn seated, both clients alike), or a new `posture` System Pack with a `sit` action that every client and agent sees? | **The shared rule now; a `posture` System as its own later step** for all worlds (café, station, Lakeside), not inside S33. The rule is written so that a disclosed posture overrides it | none now | **Ruled 2026-10-15** (operator answered directly in the primary session): the shared sit rule now. A `posture` System Pack is its own later step, and it is a **required follow-up before the public release**, not optional (operator principle "我们要发布就得把这些功能全部做好"; §14.2). |
| **Q-S33-7** | **Spending.** (a) Meshy: S33 needs **0 credits** (no new character, no generated building). (b) OpenAI images for Lakeside's 2D façades and props (≈ 5 bindings × 2–4 candidates × possibly one redo ≈ 20–40 images), at the per-image price read when S33-h is frozen? (c) Paid terrain, tree or water assets? | (a) **0.** (b) **Approve a cap at S33-h's freeze**, stated in dollars from the current price list, under the standing "call, never read" rule. (c) **None.** The free options are sufficient; nothing is bought | (a) 0; (b) price read at freeze (**verify**); (c) 0 | **Ruled 2026-10-15** (operator answered directly in the primary session): (a) 0 Meshy credits; (b) the OpenAI cap for 2D façades is set at S33-h's freeze; (c) nothing is bought. |
| **Q-S33-8** | **Re-baselines.** S33-b adds `bodies` to Lakeside, which moves its digests and Milestone E's expected values once | **Accept one recorded re-baseline in S33-b**, rebased last, after S29-a's | none | **Ruled 2026-10-15** (operator answered directly in the primary session): one recorded re-baseline in S33-b. |
| **Q-S33-9** | **The boundary:** keep `reach_m` 80 with air walls behind the first rank of woods? | **Yes**, raised only if the halt's platform end lies outside | none | **Ruled 2026-10-15** (operator answered directly in the primary session): `reach_m` 80 with air walls. |
| **Q-S33-10** | **Placement:** S33 in MVP-0, release-blocking, starting after Milestone F's checklist, superseding Q-S26-1's MVP-1 placement and S29 §11.2? | **Yes**, as the re-ruling says | none | **Ruled 2026-10-15** (operator answered directly in the primary session): MVP-0, release-blocking, after Milestone F's checklist. |
| **Q-S33-11** | **Location in world data:** should `worlds/lakeside` itself declare Lake Cuyamaca (calendar location, weather station), or only the presentation? | **Presentation only in S33.** A world-data location changes the simulation and belongs to the time and weather packs' own change | none | **Ruled 2026-10-15** (operator answered directly in the primary session): the location is used in presentation only. |

### 14.1 Rulings, 2026-10-15

The operator answered Q-S33-1…11 directly in the primary session on 2026-10-15, every question as
recommended. The coordinator relayed the rulings to this session. The step is approved as a step plan; PR
designs follow, each frozen in turn.

### 14.2 Required follow-up before release: the `posture` System Pack (Q-S33-6)

The ruling keeps S33's sitting a shared presentation rule (§4.5, `ARC-145`) and makes a **`posture` System
Pack its own step**. Under the operator's principle "我们要发布就得把这些功能全部做好" that step is
**release-blocking**: it is a required follow-up before the public release, not an optional one.

- **Owner:** the primary session adds the step to `overall.md` (§16.1) and assigns its designer. Its record is
  `ARC-147` (reserved here).
- **What it must deliver** (the requirement, not the design): a `posture` System that owns whether a person is
  standing or seated at a seat, with a `sit`/`stand` action that a human, an NPC controller and an MCP agent
  all use; seat exclusivity decided by the server; posture disclosed to every observer, so both clients and the
  agent view show it from the world rather than from a client rule; all worlds (the café, the station halls,
  Lakeside).
- **What S33 guarantees for it:** the shared sit rule yields to a disclosed posture (§4.5); sit points are pack
  data, which the System's seats can be bound to by tag; no S33 code needs to change when it lands except the
  rule's override branch.

---

## 15. Records (ids assigned 2026-10-15; written into `docs/DECISIONS.md` by the PR named)

| Id | Content |
| --- | --- |
| `ARC-143` | Lakeside's real setting, Lake Cuyamaca (Q-S33-1/2), and its far field; the lake south; the halt's stated departure from history. Written by S33-a C1 |
| `ARC-144` | 3D world selection and the per-world depiction seam, bound by tag; 16f's seed (Q-S33-4). Written by S33-c |
| `ARC-145` | Sit points as presentation data and the shared sit rule; a disclosed posture overrides it (Q-S33-6). Written by S33-f |
| `ARC-146` | How a second 3D world ships: one Presentation Pack, per-world export files, `3d-lakeside.pck` (Q-S33-3, PK-2). Written by S33-c (the refusal) and S33-j (the export) |
| `ARC-147` | **Reserved** for the `posture` System Pack step, the required follow-up before release (§14.2). Written by that step |
| `ARC-148` | Unused; returned to the primary session |
| `DEP-77` | Terrain: Godot built-ins over Terrain3D and HTerrain (rejections recorded). Written by S33-d |
| `DEP-78` | Water: our shader over community water shaders (comparison recorded). Written by S33-d |
| `DEP-79` | Vegetation: the spike's procedural cards; at most four CC0 texture sets if a reviewed frame needs them. Written by S33-d |
| `DEP-80` | **Withdrawn**: it was reserved for a paid asset, and Q-S33-7(c) ruled that nothing is bought |

---

## 16. Edits elsewhere

§§16.1–16.4's plan-file edits were **applied in this PR** (#228, 2026-10-15) on the coordinator's instruction;
the `MODULE_SPEC.md`, `VISUAL_SLICE.md` and `ART_DIRECTION.md` edits stay with the PRs named.

### 16.1 `overall.md` (applied)

- Add S33 after S29 and S32, MVP-0, release-blocking, with this file linked.
- Record that Q-S26-1's Lakeside placement and S29 §11.2's "Lakeside in 3D with 16f (MVP-1)" are superseded
  by the 2026-10-15 re-ruling.
- 16f placed in MVP-0 (QPL-17 stands), with S33-c as its seed (Q-S33-4; F-S33-1 resolved).
- The `posture` System step listed as a required follow-up before release (§14.2).

### 16.2 `step-29-station-landmark.md` (applied as a dated note; non-preclusion items for S29-a, S29-c, S29-e)

- S29-a: place Lakeside's halt at the lane's **east end**, with a north–south platform ending at a buffer stop
  near the shore (§4.0), unless S29-a's audit shows a reason against.
- S29-c: Lakeside's 3D slots carry S33-a's positions (Lake Cuyamaca, ruled); S29-c's design content stands only until S33-a lands.
- S29-e: the halt pieces (window bay, board, bench row) are written so that a second world's building can use
  them at village scale.
- §11.2: "Lakeside in 3D … come with 16f (MVP-1)" → "come with S33 (MVP-0)".

### 16.3 `step-26-cast.md` (applied)

- A dated note in the header: Q-S26-1's "Lakeside's 3D cast is MVP-1" is superseded; Lakeside's cast is drawn
  in its own world in MVP-0 (S33-g); E-2 runs both ways.

### 16.4 RL-e0 design (applied) and `MODULE_SPEC.md` §6 (S33-c)

- A dated note on App. B.2/B.3: replaced by S33-a; the lake moves south (F-S33-2).
- With PK-2 (Q-S33-3): `MODULE_SPEC.md` §6 gains a sentence on per-world export files of one Presentation Pack
  and the `PRESENTATION_WORLD_MISSING` refusal.

### 16.5 `docs/VISUAL_SLICE.md` and `docs/ART_DIRECTION.md`

- `VISUAL_SLICE.md`: a dated note that the slice's scale contract, structure rules and review views apply to
  every 3D world the default pack presents, with Lakeside's views added.
- `ART_DIRECTION.md` §4: "lakeside towns" gains the setting rule of §3.3 (references govern architecture,
  materials, light and mood; the real setting governs terrain, water, vegetation and the far field).

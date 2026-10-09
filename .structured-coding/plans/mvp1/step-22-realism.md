# Step 22 — S22: Realism — approximate 1:1 classical mechanics, and an environment that looks real

**Lifecycle:** `DRAFT, awaiting primary review`. Nothing here is frozen, and nothing here authorizes
implementation. Proposed edits to `overall.md`, `docs/DECISIONS.md` and the specifications are stated in
§14 and are not applied.
**Effort:** `mvp1` (MVP-0's non-goals exclude "photorealistic fidelity, art production", `mvp0/overall.md`
§1). §11 lists what can improve *now* inside existing MVP-0 lanes without breaking that rule.
**Author:** the S22 planning session, 2026-10-09. Worktree `/Users/yuema137/mineworld-worktrees/plan-s22`,
branch `plan/s22-realism`, from `origin/main @ f80bbb7`. One writer.
**Binding parents:** `CLAUDE.md` §§1.1, 2–4 (rules 3, 9, 10, 14, 15, 16); `mvp0/overall.md` "Framework, not
demo", "One world, two views", "The World Interaction List", and (on `origin/docs/overall-2026-10-09`)
"Operator requirements and rulings, 2026-10-08 to 2026-10-09" — *Every platform*, *Realistic defaults*,
*Classical-mechanics realism*, *One world of many regions*; `docs/ENGINEERING_RULES.md`;
`docs/ART_DIRECTION.md`; `docs/VISUAL_FIDELITY.md`; `docs/REUSE_POLICY.md`; `docs/DECISIONS.md` DEP-7, DEP-8,
ARC-9, ARC-13, ARC-20, ARC-24, ARC-46, ARC-47, DEP-13, DEP-20, ARC-55, ARC-61, ARC-62;
`mvp0/step-11-bodies.md`; `mvp0/step-18-interaction-list.md` (IL-c onward) and `step-18-physics-list.md`
§4.3–4.4 (incorporated by it); `mvp0/step-19-time-weather.md` §§6, 8 and §17 (TW-b, frozen on
`origin/plan/s19-tw-bd @ c52fa54`); `mvp0/step-15-demo-3d.md` §20 (16c), 12e and 16d as frozen on their
plan branches; `mvp1/step-21-regions-travel.md` (S21, in planning in a parallel lane; not read by this
session, referenced by path only).
**Placeholder ids** (the primary session assigns numbers): decisions `ARC-RL-a … c`, `DEP-RL-a … d`; PRs
`RL-a … RL-k`; invariants `INV-RL-n`; risks `R-RL-n`; questions `QRL-n`; findings `F-RL-n`; visual
milestones `VIS-REAL-n`.

---

# 1. The requirement

## 1.1 The operator's words, 2026-10-09 (verbatim)

> "我们最终希望我们的这个小世界，在经典力学范围内是可以1:1复刻真实世界的，当然精度不需要那么高，但是要满足基本的真实，比如说水流，比如说，风吹的树叶动之类的，这些都需要比较真实。然后我觉得现在。商店之类的物品还比较粗糙，不像真实世界，而且远处的山和河都非常非常的假，一点都不像真的"

In English:
1. The world should eventually reproduce the real world 1:1 within classical mechanics. Precision need not
   be high, but the basics must be realistic: flowing water, leaves moving in the wind.
2. The shop goods are crude and do not look like the real world.
3. The distant mountains and rivers look very fake.

## 1.2 Standing rules that bind this step

- **Realistic defaults** (operator, 2026-10-08): *"我们应该跟真实世界靠拢"* — every physical or
  behavioural default states its real-world reference beside it. First application: `PERSON_RADIUS`
  300 → 250 mm in 12d (`origin/mvp0/pr-12d-towns`, `systems/bodies/src/geometry.rs:15`).
- **Every platform** (2026-10-08): macOS, Linux and Windows. Every design names its behaviour on each.
- **Framework, not demo** (2026-10-08): realism is *content* and *Presentation Packs* a world author can
  swap. Nothing here may make realism a property of the kernel or of a reference client's core.
- **The boundary** (indexed in `overall.md` on `origin/docs/overall-2026-10-09`, "Classical-mechanics
  realism"): authoritative simulation is server-side and deterministic with real-world defaults that become
  World Interaction List content; presentation-only physics may run in the client but is driven by disclosed
  server state and decides nothing. §4 states it precisely.
- **One world of many regions** (2026-10-09, MVP-1 main line, S21): terrain, water and foliage are per
  region; clients load and release presentation assets per region. §12 checks non-preclusion.
- **DEP-8** (relicensing, not distribution, is the test; nothing Epic authored; the Tencent Hunyuan family
  excluded) and **ARC-55** (`CC-BY-4.0` is not in the default allow-list until attribution is tracked).

## 1.3 The answer in brief

```text
SERVER (authoritative, deterministic, integers in / integers out)
  bodies    gravity, Coulomb friction, restitution, mass       real defaults, cited     IL-d content
            buoyancy, water drag, wading, drift in a stream     new, needs a water pack  RL-f, RL-g
            air drag (only where it changes an outcome)         optional                 RL-c
  water     a System Pack: water bodies, surface level, flow    disclosed                RL-f
  weather   wind speed and direction per hour                   TW-b (exists in design)  —
  calendar  the sun                                             TW-a (merged)            —

CLIENT (presentation-only, decides nothing, driven by the disclosure above)
  shared    wind field = disclosed hour wind + deterministic gusts (same in 2D, 3D, every player)
  3D        foliage sway, water surface flow and foam, far-field terrain from real elevation data,
            atmosphere, item-kind models on shelves, quality tiers per OS
  2D        sprite sway, animated water polygons, the same wind numbers
```

The visual work is mostly **content and pipeline**, not engine invention: real elevation data (USGS 3DEP,
public domain), CC0 libraries (Poly Haven, ambientCG), owned generated or photographed props, and three small
shaders we write ourselves (wind, river flow, far-field terrain). The first engineering step is **budget
recovery** (§6.6): the slice already costs 15–20 ms per frame at 1600×900 on an Apple M5 with 2.4 GB of video
memory, before any realism is added.

---

# 2. Audit (`origin/main @ f80bbb7`, 2026-10-09)

Read from source by this session or by a read-only audit sub-agent whose report cites file and line; frames
captured by this session with `./mineworld-slice --shots` and `--perf` (Godot `4.7.2.stable.official.ed1daf0bf`,
Apple M5, 1600×900, `--gi=voxel`). Captured frames are in `clients/3d-spike/shots/slice/` (ignored by Git).

## 2.1 Two scenes, and which one the operator may have judged

| Scene | Launcher | Status |
| --- | --- | --- |
| **Slice** (`scenes/slice.tscn`, `scripts/slice/*`) | `./mineworld-slice` | The current client. `VIS-3D-GODOT-2` ACCEPTED (2026-10-07) on screenshots. Built from script at load. |
| **Promenade spike** (`scenes/main.tscn`, `scripts/{main,town,props,procgen}.gd`) | `./mineworld-3d` | The earlier spike, kept untouched because it is the artefact behind an accepted movement and camera decision (`mineworld-slice` header comment). |

**F-RL-1. The slice has no river and no lake.** Its only water item is a `water_manhole_cover` prop. The
lake and the faceted alpine mountains are in the promenade spike (`clients/3d-spike/screenshots/05_lake_scenic.jpg`,
`08_lake_trail.jpg`). The operator's "远处的山和河" therefore most likely describes the spike's frames, the
slice's far ridges, or both. QRL-1 asks; the design covers both.

## 2.2 Distant terrain and mountains — how they are made, and why they look fake

**Slice** (`scripts/slice/street.gd:147 backdrop()`, `:157 _ridge`):
- three ridge strips built by `SurfaceTool` from a 56-column **sine sum plus seeded RNG**; no heightmap, no
  noise field, no real data;
- at 320, 520 and 760 m, heights 62, 138 and 214 m;
- each a **flat `StandardMaterial3D` colour** (green, blue-grey, paler), roughness 1, specular off, casting
  no shadow;
- ground: a 900 × 900 m `leafy_grass` PBR plane (`:124`).

**Promenade** (`scripts/town.gd:97 _mountains`): FastNoiseLite simplex (seed 7, frequency 0.0022, five
octaves) on a **96 × 34 grid**, peaks about 430 m, vertex colours forest → rock → snow (`:155`); the far shore
is `treeline_card` billboards (`procgen.gd:175`) in front of white boxes for houses.

**Why they read as fake** (each is a fact, largest first):

| # | Cause | Evidence |
| --- | --- | --- |
| 1 | **Wrong angular size.** A 214 m ridge at 760 m subtends 15.7° above the horizon. The real San Diego skyline the world claims (TW-a: latitude 32.7157, longitude −117.1611) is low: Cuyamaca Peak (≈ 1 986 m) at roughly 60 km subtends about 1.9°, Cowles Mountain (≈ 486 m) at roughly 16 km about 1.7°. Mountains drawn close and tall read as stage flats. *Distances approximate; RL-e measures them from the DEM.* | `street.gd:157–185` |
| 2 | **No geomorphology.** Sine sums and low-octave simplex have no drainage, ridgelines, talus or erosion; real terrain is dendritic. The eye recognises this instantly. | both generators |
| 3 | **No surface detail or lighting response.** Flat colour with specular off, or vertex colour on a 96 × 34 grid: no texture, no slope-dependent material, visible facets (promenade), no self-shadowing. | `street.gd`, `town.gd:155` |
| 4 | **Atmosphere is uniform.** Depth fog 70 → 1 500 m with aerial perspective 0.60 (`slice_main.gd:159–168`) treats a 760 m ridge as "far"; real aerial perspective at 10–60 km desaturates and blue-shifts much more, and grades within a ridge with distance. | `slice_main.gd` |
| 5 | **The sky does not match the land.** One HDRI (`qwantani_puresky_2k.hdr`, Poly Haven, CC0) at a fixed sun (elevation −19.3°, azimuth −48°, ARC-13), and the key light is not driven by the calendar (TW-e will). Sky colour near the horizon does not match the fog colour, so ridges have a visible seam. | `slice_main.gd:108–129, 189` |
| 6 | **Billboards and boxes in the mid-field** (promenade): `treeline_card` rows and white boxes are legible as cards at 500 m. | `05_lake_scenic.jpg` |

## 2.3 Water

The promenade lake (`town.gd:72 _lake`) is **one 1 400 × 1 200 m `PlaneMesh`, 1 × 1 subdivisions**, with
`shaders/water.gdshader` (41 lines): three octaves of procedural value-noise normals scrolled by `TIME`;
colour by distance; roughness 0.035–0.085; reflection from the sky radiance only.

Why it reads as fake: (1) it reflects only the sky, so the mountains and town are not mirrored, which is the
single strongest cue of real still water; (2) no refraction, depth colour or shore transition, so the
shoreline is a hard line; (3) the normal noise is isotropic and not wind-aligned, so it reads as "noise" not
"wind on water"; (4) no flow: a river cannot be represented at all; (5) stair-step dark artefacts at the jetty
(`05_lake_scenic.jpg`) — shadow or depth aliasing on a single huge quad.

## 2.4 Foliage and wind

- Trees are procedural (`props.gd:109 broadleaf`, `:162 conifer`; about 23 alpha leaf cards per tree; card
  textures generated in GDScript, `procgen.gd:54`). Glb foliage: Poly Haven `shrub_02`, `shrub_03`,
  `calathea_orbifolia_01`, `potted_plant_02` (CC0).
- **There is no wind, sway or foliage animation anywhere** in either client. Every `.gdshader` in the repository:
  `clients/3d-spike/shaders/water.gdshader`, `hair_card.gdshader`, and the 2D `grade.gdshader`, `ink.gdshader`.

## 2.5 Shop goods

- Café goods are Poly Haven glTF (CC0) at real scale (`cafe_interior.gd`: `croissant`, `carrot_cake`,
  `food_apple_01`, `wine_bottles_01`, tea set, jugs, bowls, crates, baskets), placed by `SliceProps`
  (`dressing.gd`), never rescaled. These read well.
- **What reads as crude** (frame `17_street_from_east.png`, `21_florist_interior.png`):
  - Lakeside Deli's window shelves are **rows of coloured boxes**;
  - the café's back-shelf jars, canisters and bottles are procedural (`cafe_interior.gd` ~:161–178), the
    till is procedural (`:418`, Poly Haven `CashRegister_01` excluded for its banknote texture);
  - The Flower Room is mostly `Build.box`/`Build.cyl` primitives (`shop_interior.gd:193–276`); bucket
    flowers are procedural leaf cards that read as noisy blobs at 2 m;
  - nothing on a shelf corresponds to what the world sells: Market Town's twenty item kinds
    (`worlds/market-town/items/*.yaml`: apple, book, bread, cake, candle, coffee, croissant, flowers,
    juice, milk, mug, newspaper, notebook, pen, sandwich, scarf, soap, soup, tea, umbrella) have no
    presentation binding.
- 16d (frozen, `origin/plan/s14-16d @ 72087c3`) discloses a shop's listing as `listed[{item, price,
  in_stock}]` to both clients (A16d-8) and adds names on things. QS14-13 ruled "CC0 props where one fits the
  shape, else primitives" for loose objects (12e).

## 2.6 Lighting, renderer, budgets (measured)

`project.godot`: Godot 4.7, **Forward+**, Jolt for the client body (DEP-20). Sun fixed (ARC-13), SSAO on,
VoxelGI baked over the café, two `ReflectionProbe`s `UPDATE_ONCE`, SSR off, volumetric fog off, AgX tonemap,
fixed exposure.

**E-RL-1, `./mineworld-slice --perf`, 2026-10-09, Apple M5, 1600×900, gi=voxel:**

```text
view               mean ms    p50 ms  worst ms  draw calls   primitives
street wide          18.45     17.16     49.62        4155      8779705
cafe frontage        19.54     19.64     20.28        4420     10409613
interior             14.69     14.68     15.08        3899      9621580
doorway              18.06     18.02     18.30        3409      8140963
objects in frame  3468      video memory  2436.2 MB
batch  5894 primitive instances -> 663 meshes, 79068 triangles
```

**F-RL-2. The slice is already over any modest-hardware budget** on a machine much faster than modest: 4 000
draw calls, 8–10 M primitives per frame, 2.4 GB of video memory. On an integrated GPU sharing 8 GB of
system memory (Intel Iris Xe, Apple M1 base) this does not fit. Realism added on top of it would fail §6.6's
budgets; recovery comes first (RL-b).

## 2.7 Server physics today

- Rapier `=0.36.0`, `enhanced-determinism`, isolated in `systems/bodies/src/rapier.rs` (DEP-13). Integers in,
  integers out; floats only inside the adapter; no momentum between resolutions; one dynamic body per flight
  (kick, throw), rotations locked, CCD on.
- Constants: `DT = 1/60` s; `FRICTION 0.5`; `RESTITUTION 0.1`; `GRAVITY_Z −9.81` (`rapier.rs:48–56`);
  `GRAVITY 9 810` mm/s², `KICK_SPEED 5 000` mm/s (`geometry.rs:102, 145`); `PERSON_RADIUS 300` on main,
  250 on 12d's branch.
- One material for every object; no mass, no density, no drag, no buoyancy, no water, no wind.
- IL-c (after 12d) moves 26 constants into bodies' section of the World Interaction List with `default`
  reproducing today byte for byte; IL-d adds classes, materials, regions and `extends`
  (`step-18-physics-list.md` §4.3–4.4).

## 2.8 Weather and calendar reaching clients

- `calendar` (TW-a, merged #94) discloses `calendar-day` (the sun every 15 minutes, integer millidegrees)
  and `calendar-light {phase}` on the observer's place.
- `weather` (TW-b, `DESIGN FROZEN 2026-10-09`, §17 on `origin/plan/s19-tw-bd`) will disclose
  `weather-today` (24 `WeatherHour`s, each with `wind_dms` in 0.1 m/s and `wind_from_deg`, 0–359, the
  direction the wind comes from) and `weather-now`. **In TW-b the wind is the month's constant, with no
  noise** (SD-TW-b-5 (5)); TW-d's record gives the station's daily AWND.
- **No client reads any of it yet.** step-19 §8.3 places sun and weather interpretation in a shared client
  module `clients/shared/world_time/` (TW-e/TW-f).
- The world frame (`docs/CORE_CONCEPTS.md` §6.1): `+x` east, `+y` north, `+z` up; yaw from `+y` toward `+x`.

## 2.9 Licences already in force

DEP-8's approved table (Poly Haven CC0, ambientCG CC0, Sky3D MIT, Kenney CC0 for blockout only, MakeHuman
meshes CC0, CMU mocap) and its generated-asset rulings (Meshy **paid** plan passes; free plan fails; OpenAI
images pass). ARC-55's default allow-list has no `CC-BY-4.0`.

---

# 3. What "classical mechanics, roughly 1:1" means

## 3.1 Scope

**In:** rigid bodies under gravity; Coulomb contact friction; restitution; mass and inertia; buoyancy and
drag in water; wading; objects carried by a current; aerodynamic drag where it changes an outcome; wind
moving light things (later). The *appearance* of fluids, foliage, cloth and hair responding to wind and flow.

**Out:** computational fluid dynamics of the water itself (water is a **kinematic field**: a surface level
and a flow velocity, not a solved Navier–Stokes field); deformable solids and fracture as mechanics (IL-i's
"fragile" is a rule); thermodynamics beyond what `weather` already models; relativistic or quantum anything.

## 3.2 Fidelity targets (testable; "precision need not be high")

| Quantity | Target | How it is tested |
| --- | --- | --- |
| Free fall | time to fall height `h` within 2 % of `√(2h/g)` | a server test drops an object from 1.0 m and 2.0 m |
| Sliding | stopping distance on a flat surface within 15 % of `v²/(2μg)` for a given class pair | kick on two materials |
| Bounce | rebound height ratio within 15 % of `e²` | drop on the floor |
| Floating | draft (submerged depth) within 10 % of Archimedes' `ρ_obj/ρ_water × height` for a box | RL-g |
| Drift | an object in a current of speed `u` approaches `u` within 20 % after `≥ 3 m/u` seconds | RL-g |
| Wading | walking speed in water follows the cited table (§5.1) | RL-g |

These are **physical-truth tests**: they compare the engine's answer to the closed-form classical answer, so
a mis-scaled unit, a wrong sign or a dropped term is caught by physics, not by a pinned digest.

---

# 4. The simulation / presentation boundary

## 4.1 The rule

```text
A quantity is AUTHORITATIVE if any world state, any fact, any action's outcome, any controller's
decision or any other player's view can depend on it. It is computed on the server, deterministically,
from integers, and disclosed.

A quantity is PRESENTATION if nothing above can depend on it. It may be computed in a client, every
frame, in floats, but only as a pure function of
    (disclosed state, the client's world-clock estimate, presentation-pack content, a fixed seed)
so that every client, in 2D or 3D, for every player, computes the same value at the same world instant,
up to the clock estimate's error.
```

Corollaries, each binding:
1. **No presentation quantity flows back.** No client value reaches an intent, a fact or a controller
   (`ENGINEERING_RULES.md`; ARC-47's "no rule in the client, made executable" scan extends to S22's names,
   §8 INV-RL-2).
2. **Inputs are disclosed records, never client guesses.** Wind comes from `weather-today`/`weather-now`;
   the sun from `calendar-day`; water from the `water` pack's disclosure (§5.3). A client without those
   packs draws a declared *calm default* (no wind, still water, ARC-13's fixed sun), never invented weather.
3. **Anything a body can reach is authoritative geometry.** A river a person can wade into, a bank they can
   stand on, a slope they can walk up: these are world content the server knows. Terrain, water and foliage
   **beyond the reachable boundary** of the world's places are decoration (the "One world, two views" rule
   1 allowance) and must look unreachable (no walkable approach drawn).
4. **Decoration may not contradict the world.** A decorative river may not flow uphill relative to disclosed
   water it joins; decorative weather does not exist; distant terrain is chosen by the world's location or
   region (§6.1), not by the client.
5. **Client prediction mirrors, it does not define.** Where the 3D client predicts with Jolt (DEP-20), any
   physical constant it uses that the server also uses (gravity for a jump, wading speed) is either read from
   disclosure or held equal to the server's by a test. It never becomes a second source.

## 4.2 Classification of every effect the requirement names

| Effect | Class | Driven by | Owner |
| --- | --- | --- | --- |
| an object falls, slides, bounces, is kicked or thrown | authoritative | bodies' list parameters | `bodies` |
| an object floats, sinks, drifts downstream | authoritative | `water` disclosure + bodies' list | `bodies` (reads `water`'s facts through a catalog, ARC-62) |
| a person wades (slower in deeper water) | authoritative | `water` + bodies' list | `bodies` |
| a newspaper blown along the street | authoritative (later; QRL-9) | `weather`'s Public facts | `bodies` through a catalog |
| leaves, branches, grass sway in the wind | presentation | disclosed hour wind + shared gust field | shared client module + Presentation Pack shaders |
| awnings, flags, umbrellas, hair, cloth flutter | presentation | the same | the same |
| water surface ripples, flow, foam, wind chop | presentation | `water` disclosure (flow) + disclosed wind | the same |
| sea waves at the coast | presentation | disclosed wind (and later a disclosed sea state) | the same |
| rain, fog, cloud | presentation | `weather` | TW-e / TW-f (step-19), not S22 |
| sun and sky colour | presentation | `calendar` | TW-e, not S22 |
| far mountains, distant river | presentation (decoration) | region / location content | Presentation Pack (RL-e) |

## 4.3 Why the gust field is shared and deterministic

If each client drew its own random gusts, two players standing side by side would see different trees move,
and the 2D and 3D views would disagree — exactly the divergence "One world, two views" forbids, in a
presentation quantity. Making the gust field a pure function of `(disclosed hour, world time, position,
fixed seed)` gives agreement for free, with no extra traffic, and keeps the server's wind integer and hourly.

---

# 5. Physical parameters with real-world defaults

## 5.1 Parameter table

Units are integers, per `step-18-physics-list.md` §4.2 rule 2. **Where** says whether the value is a World
Interaction List parameter (`IL`, bodies' section, configurable per world, from IL-d) or an engine constant
(`E`, code; changing it bumps bodies' `VERSION`). The **source** column states how the reference was obtained:
*read 2026-10-09* when this session or its research sub-agents read it at source; *standard reference,
re-read at the PR's freeze* when it is a well-known published value this session did not fetch today.

| Parameter | Today | Proposed default | Real-world reference | Source | Where |
| --- | --- | --- | --- | --- | --- |
| `launch.gravity` | 9 810 mm/s² | **9 807** | standard gravity g_n = 9.80665 m/s² (3rd CGPM, 1901) | BIPM SI Brochure, 9th ed., Appendix; NIST SP 330 — standard reference, re-read at freeze | IL |
| local gravity (market-town) | — | 9 795 (optional override) | WGS 84 normal gravity at φ = 32.7157°: 9.7954 m/s² (Somigliana formula, computed in this document) | NGA.STND.0036 (WGS 84), eq. 4-1 — standard reference, re-read at freeze | IL (world `configure/bodies.yaml`) |
| `PERSON_RADIUS` | 300 mm (main) | **250** (12d) | adult shoulder (bideltoid) breadth ≈ 0.41–0.47 m plus clothing | ANSUR II (2012) and ISO 7250-1 summaries — re-read at freeze; ruled by the operator 2026-10-08 | E |
| `PERSON_HEIGHT` | as 12d | unchanged; QRL-8 | US adult mean stature: men 175.4 cm, women 161.5 cm (2015–2018) | Fryar et al., *NHSR* 160 (2021), CDC NHANES — re-read at freeze | E |
| person mass (new, `classes.person.mass`) | — | **80 000 g** | US adult means 90.6 kg (men), 77.5 kg (women); world-wide adult mean ≈ 62 kg | Fryar et al. 2021; Walpole et al., *BMC Public Health* 12:439 (2012) — re-read at freeze; QRL-8 chooses | IL |
| walking speed (route pacing, 12n) | as 12n | 1 300 mm/s | comfortable adult gait 1.27–1.46 m/s (ages 20–59) | Bohannon, *Age and Ageing* 26:15–19 (1997) — re-read at freeze | IL (12n's section) |
| `classes.*.material.friction` | 500 ‰ (one value) | per material, below | Rapier has one Coulomb coefficient per collider (no static/kinetic split): use the kinetic value | Rapier docs (`ColliderBuilder::friction`) | IL |
| — wood on wood | | 400 | 0.25–0.5 (dry) | CRC Handbook of Chemistry and Physics, "Coefficients of friction"; Engineering ToolBox — re-read at freeze | IL |
| — rubber/shoe sole on concrete or stone | | 700 | 0.6–0.85 dry; slip-resistance practice treats ≥ 0.5 as safe | same; ANSI A326.3 / ANSI A137.1 DCOF ≥ 0.42 wet — re-read at freeze | IL |
| — glass or glazed ceramic on wood | | 300 | 0.2–0.4 | same | IL |
| — fruit or bread on wood | | 500 | no standard; soft-body food on wood ≈ 0.4–0.6 (agricultural engineering handbooks, e.g. Mohsenin, *Physical Properties of Plant and Animal Materials*, 1986) — re-read at freeze | IL |
| — anything on ice | | 30 | 0.02–0.05 | CRC — re-read at freeze; IL-i's rink | IL |
| `combine` | `average` | `average` | Rapier default; Unity's | DEP-13, step-18 §4.3 | IL |
| `classes.*.material.restitution` | 100 ‰ | per class, below | `e = √(h_rebound / h_drop)` | — | IL |
| — ball (tennis-like) | | 740 | ITF Rules of Tennis App. I: dropped 254 cm, rebound 135–147 cm → e 0.73–0.76 | ITF — re-read at freeze | IL |
| — hard ceramic or glass (mug) on wood | | 350 | 0.3–0.5 before breakage | no single standard; IL-i owns breaking | IL |
| — book, bread, fruit, cloth | | 100 | 0.05–0.2 (deadening) | typical; measured in RL-c by a drop test of the engine against the closed form | IL |
| item mass (`classes.<kind>.mass`, g) | — | per kind, below | typical retail and USDA portion weights | USDA FoodData Central portion weights (foods); manufacturer typicals (others) — **all re-read at RL-c's freeze** | IL |
| — apple 182, croissant 57, bread loaf 454, cake (whole) 900, sandwich 250, soup can 305, milk 1 L 1 030, juice 1 L 1 050, coffee (cup, full) 400, tea box 100 | | | medium apple 182 g; croissant 57 g; 1 lb loaf; 10.75 oz can | | |
| — mug 330, book 500, notebook 200, newspaper 250, pen 10, candle 300, soap 113, scarf 150, umbrella 450, flowers (bouquet) 500 | | | | | |
| density for buoyancy (`classes.*.density`, kg/m³) | — | apple 800, bread 250, wood 500–700, ceramic 2 400, glass 2 500, paper 700 (dry), plastic bottle (sealed) effective < 1 000 | apples float (≈ 25 % air by volume) | Mohsenin 1986; CRC densities — re-read at freeze | IL |
| water density | — | fresh 998, sea 1 025 kg/m³ | fresh water at 20 °C 998.2; mean surface seawater ≈ 1 025 | CRC; UNESCO/TEOS-10 — re-read at freeze | `water` pack content |
| air density | — | 1 225 g/m³ | ISA sea level, 15 °C: 1.225 kg/m³ | ISO 2533:1975 / ICAO Doc 7488 — re-read at freeze | IL (RL-c) |
| drag coefficient `classes.*.drag` (‰) | — | sphere 470, cube 1 050, flat sheet broadside 1 170 | Re ≈ 10⁴–10⁵ | Hoerner, *Fluid-Dynamic Drag* (1965) — re-read at freeze | IL (RL-c) |
| kick launch speed | 5 000 mm/s | 5 000 (unchanged) — a nudge-kick of a household object, not a football strike (≈ 20–30 m/s) | sports-biomechanics surveys of instep kicks | Lees & Nolan, *J Sports Sci* 16:211 (1998) — re-read at freeze | IL |
| throw: unaimed range | 3 000 mm | 3 000 (unchanged) — an underarm toss | underarm toss 3–6 m/s release | — | IL |
| wading speed factor by water depth | — | ankle 0.9, knee 0.6, waist 0.35, chest: refused (`Unavailable`, swim not offered) | walking speed falls sharply with immersion depth; values to be taken from published aquatic-gait studies at RL-g's freeze (e.g. Barela et al., *Clin Biomech* 21:274 (2006)) — **not yet read; placeholders** | IL (RL-g) |
| `DT`, sub-steps, rest rule, lattice | as DEP-13 | unchanged | numerical method, not physics | DEP-13 | E |

**F-RL-3.** Rapier's single friction coefficient cannot express a static/kinetic difference. That is
acceptable at the §3.2 targets (sliding distance uses the kinetic value), and is stated so no list author
looks for a static coefficient.

**F-RL-4.** Air drag is negligible for every MVP launch: a 182 g apple (A ≈ 0.0055 m², Cd 0.47) at 5 m/s feels
0.5 × 1.225 × 0.47 × 0.0055 × 25 ≈ 0.04 N against a weight of 1.79 N (2 %). So air drag is **off by
default** and RL-c adds it only for classes whose terminal velocity is low (paper, cloth, umbrellas), where it
changes where things land.

## 5.2 What IL must grow to carry this

Bodies' list (IL-d's classes and materials) gains, per class: `mass` (g, already drafted), `density` (kg/m³),
`drag` (‰), and per material its friction/restitution. The world gains `fluids: { air: {density}, … }`. Each
is bounded like every other parameter (`step-18-physics-list.md` §4.2 rule 4), and the `default` list keeps
today's values so IL-c's byte-identity holds; **real-world values arrive as a new reference list
`realistic`** (RL-c) that Market Town and Social Café select, so the change is visible, versioned and
revertible by content.

## 5.3 Server-side water (`water` System Pack, RL-f)

- **Owns** water bodies: a `water-body` component on a place (or a region, after S21), with its outline in
  the place frame (integer mm polygon), surface level `z` (mm), kind `still | river | sea`, density (kg/m³),
  and for a river a flow polyline with per-vertex velocity (mm/s, direction from the polyline). Content,
  in the place file's `water:` section (ARC-31 pattern) or `configure/water.yaml`.
- **Tide** (sea, later): a pure function of world time from a harmonic table (NOAA CO-OPS station 9410170,
  San Diego, predictions — US government work; licence to be confirmed at freeze), integer.
- **Discloses** each water body on the observer's place: outline, level, kind and flow. That is the input
  for every water visual (§7.2) and for bodies' buoyancy and wading.
- **Bodies reads it through an extension catalog** (ARC-62), never by naming the pack's internals; with no
  `water` pack enabled, nothing floats and nothing wades (calm default).

## 5.4 Server-side wind (not in S22's first PRs)

Wind already exists authoritatively in `weather` (hourly). S22 adds **no** authoritative wind effect in its
first PRs. QRL-9 asks whether light objects should ever be moved by wind; if yes, bodies reads the hour's
wind through a catalog when it resolves a flight, never a client's gusts.

---

# 6. Visual research (licences and dates read)

Every row below was read at source on **2026-10-09** by this session or its research sub-agents, unless marked
*unverified*. An unverified row may not enter a decision record until it is re-read at that PR's freeze.

## 6.1 Terrain and distant mountains

| Candidate | Licence | State (2026-10-09) | Verdict |
| --- | --- | --- | --- |
| **USGS 3DEP** DEMs (1/3 arc-second ≈ 10 m seamless for the contiguous US; 1 m lidar where flown; 1 arc-second ≈ 30 m) | **US public domain** ("USGS-authored or produced data … considered to be in the U.S. Public Domain"); credit requested: "Credit: U.S. Geological Survey" (usgs.gov copyrights-and-credits) | The National Map Downloader (apps.nationalmap.gov/downloader). San Diego in UTM 11N, NAVD88 heights | **Adopt** as the default elevation source for US regions |
| NASADEM / SRTM (30 m, 60°N–56°S) | LP DAAC: "no restrictions on reuse, sale, or redistribution"; citation requested; not formally called public domain | active | **Adopt for non-US regions** where 3DEP does not exist, with the citation |
| Copernicus DEM GLO-30 | free licence, redistribution allowed; **mandatory** copyright notice ("© DLR e.V. 2010-2014 and © Airbus …"), "produced using Copernicus WorldDEM-30", and a liability disclaimer | active | **Second choice** outside the US; the mandatory notices fail ARC-55's typed policy today and need the attribution tracking of DEP-RL-c |
| **Terrain3D** (TokisanGames) | MIT | v1.0.2-stable, 2026-05-19; active; C++ GDExtension; Windows, Linux, macOS | **Adopt later, for walkable relief only (RL-j)**, after a prototype proves it on Godot 4.7 (it claims 4.4–4.6) |
| HTerrain (Zylann) | MIT | maintenance mode, bug fixes only; master targets 4.6+ | fallback only |
| godot-imposter (zhangjt93) — octahedral impostors | MIT | small (23 commits), verified on 4.5 beta | **Prototype**; vendor if adopted (REUSE_POLICY §9) |
| Octahedral Impostors (wojtekpil) | MIT | Godot 3.2 only, unmaintained | reject |
| Godot built-in fog: depth + height fog, **Aerial Perspective**, Sun Scatter (all renderers); volumetric fog (Forward+ only) | MIT | 4.7 docs | **Adopt**; volumetric fog only in the High tier |
| `PhysicalSkyMaterial` (Rayleigh/Mie) | MIT | built-in | **Adopt** as the sky under TW-e; matches fog to sky by construction |
| Sky3D (TokisanGames) | MIT; the bundled star map requires credit | active; 4.3+; all renderers | already DEP-8-approved; TW-e's choice |
| Poly Haven HDRIs | CC0 ("Our assets are all licensed as CC0"); scraping needs permission, use the API | active | **Keep** for reflections and interiors; not as the outdoor sky once the sun moves (an HDRI has a baked sun) |
| USDA **NAIP** orthoimagery (0.6–1 m) and USGS **NLCD** land cover (30 m) | believed US public domain — *unverified today* | — | **Investigate** at RL-e's freeze: NLCD drives far-field material and tree density; NAIP only as a far-field albedo hint (it bakes in one date's shadows) |

**Recommendation.** For the far field, do **not** put a runtime terrain addon in the client. Bake: a small
offline tool (`tools/terrain-bake`, Rust, `tiff` crate — MIT — to read GeoTIFF; DEP-RL-a) turns a 3DEP
tile set into **concentric glTF rings** (DEP-7) around the region's origin — 30 m spacing to 5 km, 90 m to
20 km, 270 m to 80 km — with skirts, normals and a material index from slope, altitude and (later) NLCD.
One terrain shader (ours) splats ambientCG CC0 materials (dry grass, chaparral, rock, sand) by that index
and fades to an aerial-perspective colour. Godot's visibility ranges give the rings HLOD for free. This is
renderer-agnostic (works on Mobile and Compatibility), deterministic, small (a 20 km × 20 km area at 30 m is
≈ 0.9 MB of 16-bit heights), and the same rings render S21's window scenery. Terrain3D enters only when the
**walkable** ground stops being flat, which is an authoritative change (RL-j).

## 6.2 Water and rivers

| Candidate | Licence | State | Verdict |
| --- | --- | --- | --- |
| Waterways (Arnklit): Bézier rivers, baked flow and foam maps | MIT (© 2021 Kasper Arnklit Frandsen) | Godot 3 only, inactive | **Reference to port the technique from**, credited; not a dependency |
| waterways-net (Tshmofen) | MIT (credits the original) | 4.2+, **C#/.NET only** | reject (needs the .NET editor build; the slice is GDScript) |
| **GodotOceanWaves** (2Retr0): FFT ocean, Jacobian foam | MIT; bundled OTFFT MIT, HDRI CC0 | 18 commits, no releases; RenderingDevice compute (Forward+/Mobile, not Compatibility) | **Prototype** for the coast in the High tier; vendor if adopted |
| Boujie Water Shader: Gerstner, LOD ocean, shore foam | MIT | v1.0.1 (2023-09-22), stale | **Reference** for the Low-tier sea |
| GodotSSRWater (marcelb): SSR for transparent water | MIT | branches for 4.3/4.4 | **Reference**: Godot's built-in SSR does not apply to transparent surfaces |
| godotshaders.com | **per shader**: CC0, MIT or GPL-3.0, author's choice, no default | — | only CC0/MIT shaders, recorded per file; **GPL-3.0 excluded** |

**Technique (Godot 4.7):** screen-texture refraction (`hint_screen_texture`), depth absorption and shore foam
(`hint_depth_texture`), two-phase flow-map scrolling along a spline-generated ribbon mesh, normal maps (CC0
or generated) **aligned to wind and flow**, Fresnel, and reflections from a planar or screen-space pass where
the tier allows, else a `ReflectionProbe` that includes terrain (not the sky alone, §2.3 cause 1).

**Recommendation.** One small MineWorld-owned water shader family (river, lake, sea-low) built on the
standard flow-map technique, ported from Waterways' approach with credit; GodotOceanWaves prototyped for the
High-tier coast. Every input — outline, level, flow, wind — comes from disclosure (§4) or, for decoration,
from the Presentation Pack.

## 6.3 Foliage and wind

| Candidate | Licence | State | Verdict |
| --- | --- | --- | --- |
| **Our own wind vertex shader** — hierarchical main bending + branch + leaf flutter, per-instance phase via `INSTANCE_CUSTOM` | ours (MIT) | technique published: Sousa, "Vegetation Procedural Animation and Shading in Crysis", *GPU Gems 3* ch. 16 (2007) | **Build**: ~150 lines; copying a third-party shader would import licence ambiguity for no gain |
| MultiMesh instancing | engine | "millions" of instances per draw; **no per-instance culling** — chunk per area | **Adopt** for grass and shrubs |
| ProtonScatter (HungryProton) | MIT; **demo textures from Textures.com are not redistributable** | rewritten for Godot 4 | **Optional editor tool**; strip its textures; the slice is built from script, so it may not be needed |
| Spatial Gardener (dreadpon) | MIT | ≥ 4.2 | same as above |
| **ez-tree** (dgreenheck) — procedural tree generator, GLB with LODs | MIT repository; output terms not stated (generated by MIT code) | active | **Adopt as a tool** for broadleaf trees; LICENSE text to be read at freeze |
| Blender Sapling Tree Gen | GPL-3.0-or-later (the add-on) | Blender 4.4+, "limited support" | generated meshes are ours; the add-on is never vendored |
| Tree It (Evolved) | "Free"; FAQ allows selling models "so long as its your own work"; no formal licence | Windows only | weak; not recommended |
| SpeedTree | EULA not readable today (redirect, 403); reports say meshes are restricted from redistribution and possibly engine-bound | — | **Exclude** (fails DEP-8's relicensing test as far as can be read) |
| The Grove | proprietary; EULA not readable (404) | — | **Exclude** until its EULA is read |
| Poly Haven models (Nature: plants, boulders, debris) | CC0 | active | **Adopt** for shrubs, rocks, ground debris |
| Quaternius, Kenney | CC0 | — | style is low-poly; blockout only (DEP-8) |

**San Diego vegetation** (if QRL-2 chooses San Diego): Mexican fan palm (*Washingtonia robusta*), Canary
Island date palm, jacaranda, coast live oak, eucalyptus, Torrey pine; coastal sage scrub and chaparral on the
far hills. Palms are not what ez-tree generates; they need a dedicated model (Poly Haven if one exists,
else a Meshy paid-plan generation under ARC-9, else hand-modelled). That gap is recorded, not assumed away.

## 6.4 Props and shop goods

| Source | Licence | Redistributable in an MIT repository? | Verdict |
| --- | --- | --- | --- |
| **Poly Haven models** (15 categories incl. Food & Kitchen, Containers, Decor) | CC0 | yes | **First choice**; Food is thin (bananas, carrot cake, croissant, apple, pots, plates) |
| ambientCG | CC0 ("You can include the raw files in your project") | yes | materials for packaging, shelves, labels |
| Smithsonian 3D (items labelled CC0) | CC0 per item (*unverified today*: page returned 403) | yes, CC0 items only | museum objects, not groceries; heavy, decimate |
| **Meshy, paid plan** | owned output ("customers on a paid Meshy plan own their Customer Output", Terms of Use last updated 2026-09-19, read 2026-10-09) | yes (DEP-8 ruling) | **Second choice** for goods Poly Haven lacks; ARC-9 provenance; AI identifiers kept |
| Meshy, free plan | CC-BY 4.0 with credit to Meshy; Meshy owns the output | not under ARC-55's default | **Excluded** (DEP-8: the free plan fails) |
| **Our own photogrammetry** (Meshroom MPL-2.0 or COLMAP BSD-3-Clause, both run as tools, never vendored) | output owned by the photographer | yes | **Third choice**, the most realistic for groceries; **generic, unbranded** items only (packaging art and logos are someone else's copyright and trademarks) |
| TripoSR / TripoSG | MIT code and weights (DEP-8, with BiRefNet, not the bundled non-commercial removers) | yes | alternative to Meshy |
| Fab — Standard License, and Quixel Megascans on any tier | Epic's grant is non-sublicensable; Fab summary: no standalone redistribution (*the Fab EULA page returned 403 today*) | **no** | **Excluded** by DEP-8's 2026-09-27 amendment ("nothing Epic authored enters it") |
| Fab — third-party listings marked CC-BY or open source | per listing | per listing, with attribution | only after DEP-RL-c (attribution tracking) and a per-asset check (DEP-8 §2(c) door) |
| Sketchfab | CC0 and CC-BY usable with attribution; **CC-BY-NC, any ND, CC-BY-SA, Editorial and Sketchfab Standard excluded**; authors can change a licence later, so snapshot it at download | CC0 yes; CC-BY after DEP-RL-c | per-asset only, recorded in the pack's `LICENSES/` |
| Objaverse | per-object licences (CC-BY ≈ 721 k, NC and SA variants, CC0 ≈ 3.5 k); provenance disputed | — | **Avoid** as a source |
| Tencent Hunyuan3D and family | Tencent community licences | **no** | **Excluded** (DEP-8, operator 2026-10-08) |

**Why the goods look crude is not a sourcing problem alone.** The coherence procedure of DEP-8 (scale →
material authority → lighting rig → texel density ≈ 512 px/m) applies to goods at a shopper's distance
(0.5–2 m), where texel density must be higher: **≈ 1 024–2 048 px/m for hand-held goods**, which a 1k texture
on a 10 cm apple already exceeds. The crude reads come from primitives and procedural cards, from **no
repetition variety** (identical boxes in rows), and from **no merchandising logic** (real shelves have
facings, labels, price tags, depth and slight disorder). RL-h addresses all three.

## 6.5 Rendering features and cost (Godot 4.7 docs, read 2026-10-09)

- Base cost: Compatibility < Mobile < Forward+. **Forward+ only:** SDFGI, VoxelGI, SSIL, volumetric fog, SSR,
  TAA/FSR2, SSS, PCSS. Mobile lacks SSAO. Since 4.4 a failed Forward+/Mobile start falls back to
  Compatibility.
- Measured by others on an Apple M1: a volumetric-fog stress scene at ≈ 80–95 fps against 768 without; the
  Metal backend regressed against MoltenVK in 4.4; **SDFGI produced no indirect light on Metal** in a reported
  thread (godot PR #108028, issue #103723). Iris Xe ≈ half an M1's 3DMark score; SDFGI and VoxelGI reported
  broken on some Intel drivers (anecdotal).
- Mesh LOD is generated at import (meshoptimizer); `visibility_range` with `visibility_parent` gives HLOD.

## 6.6 Performance budgets, per tier and per OS (proposed; QRL-6)

| Tier | Reference hardware (each OS) | Resolution / target | Frame budget | Video memory | Renderer and features |
| --- | --- | --- | --- | --- | --- |
| **Low** | macOS: Apple M1 (7/8-core GPU). Windows 11 and Ubuntu 24.04: Intel Iris Xe (i5-1135G7 class) | 1920×1080 output, 0.67 render scale (FSR 1), 30 fps | 33.3 ms | ≤ 1.5 GB | Mobile renderer; no SSAO/SSR/GI; Gerstner sea; wind on trees only; far rings 2 |
| **Default** | macOS: M1 Pro / M2. Windows and Linux: GTX 1650 / RX 6500 XT class | 1920×1080, 60 fps | 16.7 ms | ≤ 2.0 GB | Forward+; SSAO; SSIL or VoxelGI; wind on trees, shrubs, grass; flow-map water; far rings 3 |
| **High** | macOS: M3 Pro+. Windows and Linux: RTX 3060 class | 2560×1440, 60 fps | 16.7 ms | ≤ 4 GB | adds volumetric fog, SSR, FFT coast, denser grass |

**Draw-call and geometry ceilings at Default:** ≤ 2 000 draw calls, ≤ 3 M primitives per frame.
**Increments S22 may spend at Default** (measured against RL-b's recovered baseline, never against today's
over-budget slice): wind foliage +1.0 ms; far terrain +1.0 ms; water +1.5 ms; sky and atmosphere +0.5 ms;
goods +0.5 ms; total ≤ +4.5 ms.

**Measurement protocol.** `./mineworld-slice --perf` gains `--tier=<low|default|high>` and prints the four
views plus a fixed far-field view and a water view. It runs by hand on each named reference machine, because
CI is a headless Linux container with no GPU (DEP-17). Each run records OS, GPU, driver, Godot build,
renderer and tier. A tier passes on an OS when every view's **p95** is within its frame budget and video
memory is within the ceiling. Which physical machines exist for Windows and Linux is QRL-6
(operator-material).

**Settings.** The tier is a presentation setting in the shared client settings module (`overall.md`
"Framework, not demo" item 4; S20), defaulting from a one-time probe of the adapter name and video memory,
never reaching the server.

---

# 7. Design

## 7.1 The shared environment module (both clients)

`clients/shared/environment/` beside step-19's `clients/shared/world_time/` (one GDScript module; neither is
the protocol module):

- **`wind.gd`** — `wind_at(world_pos_mm: Vector3i, at_est: int) -> Vector3` (m/s, world frame) and
  `gust_phase(...)`. Built from:
  - the disclosed hour (`weather-today.hours[h].wind_dms`, `.wind_from_deg`), blended linearly across the
    hour boundary over 600 world seconds so an hourly change is not a step;
  - the mean flow toward `wind_from_deg + 180°` in the world frame (`+x` east, `+y` north);
  - a **height profile**: the log law `u(z) = u₁₀ · ln(z/z₀)/ln(10/z₀)`, with `z₀` from the Presentation
    Pack (suburban 0.5 m by default; Wieringa 1992 / WMO CIMO Guide roughness classes — re-read at freeze);
  - **gusts**: turbulence intensity `I = 1/ln(z/z₀)` (EN 1991-1-4 §4.4 with k_I = 1 — re-read at freeze),
    applied as `u · (1 + I · n)`, where `n` is a band-limited value-noise field in `(position − ū·t, t)` —
    Taylor's frozen turbulence, so gust fronts travel downwind and neighbouring trees sway in sequence —
    seeded by a fixed constant of the Presentation Pack and the hour's `day_start + 3600 h`.
  - With no `weather` record disclosed: calm (`u = 0`), stated on the HUD's debug line, not invented.
- **`beaufort.gd`** — the WMO Beaufort table as presentation content: force 2 (1.6–3.3 m/s) "leaves rustle",
  3 (3.4–5.4) "leaves and small twigs in constant motion", 4 (5.5–7.9) "small branches are moved",
  5 (8.0–10.7) "small trees in leaf begin to sway", 6 (10.8–13.8) "large branches in motion" (WMO-No. 306 /
  Met Office Beaufort scale — re-read at freeze). Foliage amplitudes are **calibrated to this table**, which
  is the real-world reference for "how much should leaves move at this wind".
- **`water_view.gd`** — reads the `water` pack's disclosure (§5.3) into presentation records: outlines, level,
  flow polylines; and for the sea a wind-derived sea state: significant wave height and peak period from the
  Pierson–Moskowitz relations for a fully developed sea (`H_s ≈ 0.21 U₁₉.₅²/g`, `T_p ≈ 7.14 U₁₉.₅/g` —
  Pierson & Moskowitz, *JGR* 69:5181 (1964), re-read at freeze), capped by a fetch limit from the
  Presentation Pack. Swell (San Diego's long-period Pacific swell) is presentation content until a pack
  discloses a sea state (QRL-10).
- **Parity:** both clients print `ENV {"wind": [vx, vy, vz], "beaufort": n, "gust": g, "water": [...]}` at
  fixed probe frames; the "One world, two views" parity test (16e/13f) admits `ENV` lines and compares them
  within the clock-estimate tolerance. A client that bypasses the module fails it.

## 7.2 3D (`clients/3d-spike`)

- **Wind on foliage (RL-d).** One spatial shader with three layers (Sousa 2007): main bending of the whole
  plant (stiffness by height²), branch sway (per-vertex colour channel = branch weight and phase), leaf
  flutter (UV-space, high frequency, amplitude ∝ gust). Uniforms come from `wind.gd` per chunk; per-instance
  phase from `INSTANCE_CUSTOM`. Applied to the procedural trees (`props.gd`), Poly Haven shrubs, planters,
  grass MultiMesh chunks, awnings and flags (a cloth-like bend, no simulation). Shadow pass uses the same
  vertex function so shadows sway with leaves.
- **Far field (RL-e).** Baked rings (§6.1) behind the town, oriented by the world's real north; the slice's
  sine ridges are retired. Aerial perspective, fog colour and sky are tuned together and driven by TW-e's sun.
- **Water (RL-f presentation half).** River ribbon from the disclosed flow polyline (or decorative content),
  flow-map shader, depth colour and shore foam, wind-aligned normals scaled by Beaufort; lake: same shader,
  flow zero; sea: Gerstner (Low/Default) or FFT (High). Reflections: a `ReflectionProbe` capturing terrain and
  town at Low/Default, SSR-for-water at High.
- **Goods (RL-h).** A **role → art binding for item kinds** in the Presentation Pack (ARC-46): each `ItemKind`
  key maps to a model, a facing size and a shelf arrangement; a **merchandiser** places goods on a shop's
  shelves from the disclosed listing (`listed[{item, in_stock}]`, 16d): kinds offered are shown, `in_stock = 0`
  shows an empty gap with a price tag, counts are capped at the shelf's facing capacity. Goods with no binding
  fall back to a neutral packaged box, never to an invented kind. This is decoration that *follows* the world
  (`"One world, two views"` rule 1), and it reports nothing to the parity test beyond the listing it read.

## 7.3 2D (`clients/2d`)

- Tree and shrub sprites sway with a canvas vertex shader driven by the same `wind.gd` (skew about the
  sprite's base, amplitude by Beaufort).
- Disclosed water polygons draw with an animated flow texture along the flow polyline and a shoreline foam
  line. Decorative water only where the 2D presentation pack draws scenery.
- No far-field terrain in the isometric view (a hillshade vignette at most, presentation pack's choice).
- Goods: the same item-kind binding table carries a 2D icon per kind; shop panels and shelves use it.

## 7.4 Presentation Pack layout (the default; a world author swaps it)

```text
presentation/mineworld-default/
  3D/
    environment/   wind.yaml (z0, gust band, Beaufort amplitudes), water.yaml (fetch limit, colours)
    terrain/<region>/rings/*.glb, NOTICE   (baked from 3DEP; credit "U.S. Geological Survey")
    items/<kind>.yaml → model, facing, arrangement
  2D/
    items/<kind>.png …
```

Large binaries (terrain rings, goods, textures) live in **Asset Packs** (ARC-7), one per region where it
makes sense, so S21's per-region streaming loads and releases them. Where those packs are stored (Git LFS,
a separate repository, release artefacts) is QRL-7 (operator-material).

---

# 8. Invariants (proposed; frozen only by the primary session or the operator)

| Id | Invariant |
| --- | --- |
| INV-RL-1 | Every authoritative physical quantity is computed on the server, from integers, deterministically; floats exist only inside `systems/bodies/src/rapier.rs` (DEP-13's isolation unchanged). |
| INV-RL-2 | No presentation quantity reaches an intent, fact, controller or another client. The client-rules scan (`tests/acceptance/tests/client_rules.rs`, `scripts/check_client_rules.py`) is extended with S22's module names and passes. |
| INV-RL-3 | Every presentation-physics input is a disclosed record, the client clock estimate, presentation-pack content or a fixed seed. With a pack absent, the calm default is drawn. |
| INV-RL-4 | Two clients (2D and 3D, or two players) given the same observation stream compute the same `ENV` values at the same world instant within the clock-estimate tolerance. |
| INV-RL-5 | Anything a person can reach is disclosed world geometry; decoration is drawn unreachable. |
| INV-RL-6 | Every physical default states its real-world reference and source beside it, in the list file and in §5.1's successor spec. A default with no source is a review FAIL. |
| INV-RL-7 | `default` reference lists stay byte-identical to their predecessors; real-world values arrive as a new named list (`realistic`) a world selects, with a bodies `VERSION` bump if any engine constant moves. |
| INV-RL-8 | Every committed asset passes DEP-8's relicensing test and ARC-55's policy, with provenance (`ARC-9` for generated, `NOTICE`/`LICENSES/` for data and third-party). |
| INV-RL-9 | `VIS-3D-GODOT-1` and `VIS-3D-GODOT-2` do not regress except where the operator accepts a named change (frame-diff guard as 16c §20.6, per view). |
| INV-RL-10 | Each tier meets its budget on each OS on its reference machine before a realism PR is READY; a PR that cannot is a material stop. |
| INV-RL-11 | No kernel, contract or persistence change: realism is System Packs, list content and Presentation Packs (`CLAUDE.md` §4 rule 5). |

---

# 9. PR split, integration checkpoints and adversarial criteria

## 9.1 Order

```text
                        ┌─► RL-d wind (needs TW-b merged; TW-e for the sun) ─┐
RL-a spec + module ─────┤                                                     ├─► VIS-REAL-1 (foliage + sky)
                        │  RL-b budget recovery ─► RL-e far field ───────────┤─► VIS-REAL-2 (distance)
                        │                       └► RL-h goods ───────────────┤─► VIS-REAL-3 (goods)
IL-d (S18) ─► RL-c realistic list ─► RL-f water pack + water visuals ─► RL-g buoyancy/wading/drift ─► VIS-REAL-4
RL-i quality tiers (with S20 settings) — parallel to all
RL-j walkable relief (Terrain3D + bodies heightfield) — outlined, MVP-1 late or MVP-2; with S21
RL-k CC-BY attribution tracking (DEP-RL-c) — parallel; unblocks CC-BY sources
```

## 9.2 The PRs

| PR | Scope | Depends on | Integration checkpoint (real execution) | Adversarial criterion (fixed before measuring) |
| --- | --- | --- | --- | --- |
| **RL-a** Spec, boundary, shared module skeleton | `docs/` spec of §4 and §5.1 (ARC-RL-a), the `ENV` report format, `clients/shared/environment/{wind,beaufort}.gd` with unit-free tests, the scan extension | TW-a (merged); TW-b's record shapes (frozen) | both clients connected to one Market Town with `weather`: each prints `ENV` at three world instants; values equal within tolerance | feeding one client an observation stream with the wind from 270° and the other from 90° makes the parity check FAIL naming `wind`; a client file that writes a wind value into an intent fails the scan |
| **RL-b** Budget recovery (prerequisite) | merge static meshes per block, LOD/HLOD via visibility ranges, texture compression (VRAM-compressed import, 1k cap for distant props), shadow-distance and cascade tuning, `--perf --tier` | none (presentation only) | `--perf` on the M5 and on every reference machine available: Default ≤ 2 000 draw calls, ≤ 3 M primitives, VRAM ≤ 2.0 GB, p95 ≤ 16.7 ms | V-1 frame diff against today within the base's own noise + 0.5 points per view (16c §20.6); a deliberately unbatched block raises draw calls above the ceiling and the perf check FAILs |
| **RL-c** The `realistic` bodies list | per-class mass, density, drag, materials; air drag for low-terminal-velocity classes; Market Town and Social Café select `realistic`; physical-truth tests of §3.2 | IL-d (classes, materials) | headless `mineworld run` of both towns with kicks and throws; digests re-baselined by design; a 3D session kicks an apple on stone and on wood | each §3.2 test also runs against a mutated list (μ halved; g = 0) and must FAIL; `default` stays byte-identical (IL-I1) |
| **RL-d** Wind in both clients | the foliage shader (3D), sprite sway (2D), awnings and flags, Beaufort calibration | RL-a; TW-b merged (TW-e for the sun if it lands first) | `--world` with weather: the operator walks the street at Beaufort 1, 3 and 5 (a test world pins the hour's wind) | with `weather` disabled the trees are still (calm default); swapping wind direction in the disclosure reverses the sway direction in both clients' `ENV` and in a frame pair; frame cost increment ≤ +1.0 ms at Default |
| **RL-e** Far-field terrain and atmosphere | `tools/terrain-bake` (DEP-RL-a), San Diego rings from 3DEP 1/3″ (DEP-RL-b), terrain shader with ambientCG splats, fog and aerial perspective tuned with TW-e's sky; retire `street.gd` ridges | RL-b; QRL-2; TW-e preferred | `./mineworld-slice --shots` adds four far-field views (east to the mountains, west to the sea, north, south) at three sun elevations | the baked skyline's angular profile, sampled at 1° azimuth steps from the town origin, matches a profile computed directly from the source DEM within 0.1°; a ring with its north rotated 90° FAILs that check; increment ≤ +1.0 ms |
| **RL-f** The `water` pack and water visuals | System Pack `water` (ARC-RL-b): water bodies, disclosure; Market Town content (QRL-3); both clients draw disclosed water with flow; sea from wind | RL-a; RL-c for densities | headless run discloses water; both clients draw it; `ENV.water` equal in both | a river polyline reversed in content reverses the drawn flow in both clients; a water body removed from content disappears from both (no client draws water the world lacks); increment ≤ +1.5 ms |
| **RL-g** Buoyancy, wading, drift | bodies reads `water` through a catalog: buoyancy and water drag in a flight, wading speed factor, a `drift` Process carrying floating objects downstream in bounded steps | RL-f; 12n (walk-to) | a thrown apple lands in the river and drifts; a person wades knee-deep at the cited speed; save, restart, resume: identical | §3.2 floating and drift tests, each with a mutated density that must FAIL; without `water` nothing floats; arm64 = x86_64 digests (DEP-13's practice) |
| **RL-h** Goods and merchandising | item-kind binding table (3D models, 2D icons), merchandiser from the disclosed listing, sourcing per §6.4, the coherence procedure, texel density ≥ 1 024 px/m for hand-held goods | 16d merged (listing and names); RL-b | Market Town connected: every shop shows only kinds it lists; buy one until `in_stock = 0`, the facing empties | a listing with a kind that has no binding shows the neutral fallback, never another kind; removing a kind from the listing removes it from the shelf; increment ≤ +0.5 ms |
| **RL-i** Quality tiers | tier selection in the settings module, renderer per tier, feature toggles; per-OS runs | S20 settings module; RL-b | each tier on each available OS reaches its budget | forcing Low on the M5 changes renderer and features as declared (asserted from the running viewport, not the settings file) |
| **RL-j** Walkable relief *(outlined only)* | non-flat floors: bodies gains a Rapier height-field collider from integer heights; places carry relief; Terrain3D for the near field | S21 regions; IL-d; a separate step design | — | — |
| **RL-k** CC-BY attribution tracking | DEP-RL-c: an attribution manifest per pack, checked; ARC-55's allow-list gains `CC-BY-4.0` for asset packs | S16 E-b's policy | `mineworld packs validate` refuses a CC-BY asset without its attribution entry | — |

## 9.3 Visual acceptance items the operator judges by hand

Each is a named milestone under ARC-20, previewed early under ARC-24, with the package of
`VISUAL_FIDELITY.md` §10: launch command, runtime frames at the reference's framing, the reference beside
them, known misses largest first, specific questions. The agent's §5 hard-fail check runs first.

| Id | What the operator does | What they should see |
| --- | --- | --- |
| **VIS-REAL-1** Wind | `./mineworld-slice --world` with a pinned windy hour; stand under the street trees; then a calm hour | leaves flutter and small branches move at Beaufort 3–4; gusts visibly travel down the street; calm means still; 2D and 3D agree in direction |
| **VIS-REAL-2** Distance | look east, west, north, south from the street and from the café terrace at golden hour and midday | mountains low and far, as the real place's skyline; layered blue-grey with distance; no seam between land and sky; no facets or cards visible |
| **VIS-REAL-3** Goods | walk into each shop, stand at the shelf, pick up and buy | goods read as real objects at 0.5 m; shelves look stocked by a person (facings, tags, slight disorder); only what the shop sells |
| **VIS-REAL-4** Water | stand at the river (or lake / coast per QRL-3); throw an apple in; wade | surface flows the right way with foam at obstacles and banks; reflections of the banks and town; the apple floats and drifts; wading slows the walk |

---

# 10. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-RL-1 | The slice is already over budget (F-RL-2); realism makes it worse on modest machines | RL-b first; every RL PR measured against RL-b's baseline; INV-RL-10 |
| R-RL-2 | No Windows or Linux GPU machine to measure on (CI has no GPU) | QRL-6; until then a tier is "measured on macOS only", stated, never claimed for the others |
| R-RL-3 | SDFGI is broken or empty on Metal and some Intel drivers | no tier depends on SDFGI; VoxelGI/SSIL only at Default and High |
| R-RL-4 | Terrain3D on 4.7 unproven; impostor addon small | far field is baked glTF with no runtime addon; Terrain3D only in RL-j after a prototype |
| R-RL-5 | Repository size explodes with terrain, goods and textures (today ≈ 150 MB under `clients/3d-spike/assets`) | Asset Packs per region outside the main history (QRL-7); per-pack size budgets |
| R-RL-6 | The default reference images (alpine lakeside, snow peaks) contradict a San Diego world (TW-a, TW-b) | QRL-2 |
| R-RL-7 | A presentation effect drifts into a rule (e.g. the client slows the player in water it draws) | INV-RL-2 scan; RL-g puts wading on the server; client prediction mirrors (§4.1 corollary 5) |
| R-RL-8 | Gust fields diverge between clients because clock estimates differ | the tolerance is the estimate's error by definition; the parity test measures it |
| R-RL-9 | Physical references cited from memory turn out wrong | every row marked "re-read at freeze" is re-read before its PR freezes; physical-truth tests check the engine against physics, not against the citation |
| R-RL-10 | Photogrammetry or generated goods carry brands or text | generic, unbranded goods only; the §5 hard-fail check includes visible logos and readable third-party text, as the excluded `CashRegister_01` banknote precedent |
| R-RL-11 | Realism blurs readability (ART_DIRECTION §4 "gameplay readability remains more important") | VIS-REAL items ask about readability explicitly; interactive objects keep their affordance highlights |
| R-RL-12 | Rapier version bumps change physical-truth results | the pin is exact (DEP-13); the physical-truth tests are tolerant by design, so an upgrade that changes digests but not physics passes them |

---

# 11. MVP-0 non-preclusion and quick wins

MVP-0 excludes "photorealistic fidelity, art production". Nothing below adds scope to a frozen MVP-0 PR on
its own authority; each line is a **proposal to that lane's owner** (the primary session rules), sized to fit.

## 11.1 Non-preclusion checks (already true, or one sentence to keep true)

| Lane | What must not be precluded | Status |
| --- | --- | --- |
| IL-c / IL-d (S18) | per-class `mass`, `density`, `drag`; a named reference list besides `default`; `fluids` | IL-d's drafted schema already has `mass` and named lists with `extends`; ask IL-d to reserve `density` and `drag` as "only value in this step: absent" fields (step-18 §4.4's pattern) |
| TW-b / TW-d / TW-e (S19) | wind per hour with direction; sun per 15 min | already true (SD-TW-b-5, -8, -9). Ask TW-e to put its sky/weather mapping in `clients/shared/world_time/` so RL-a's `environment/` sits beside it, not inside a client |
| 12e (S15, frozen) | loose objects drawn by kind | 12e draws objects from disclosed shapes (QS14-13: CC0 where one fits). RL-h's binding table replaces the choice later without changing 12e's colliders |
| 16c / 16d / 16g (S14) | shelves dressed from the listing; the florist as the store | 16d discloses `listed[{item, price, in_stock}]` to both clients — the merchandiser's only input. Nothing to change |
| 12n / 12d (S15) | walking speed and wading as route pacing factors | 12n's `walk-step` pacing should take its speed from the list, so RL-g can scale it by water depth without a new mechanism |
| S20 (settings) | a graphics quality tier | ask S20 to reserve a `display.quality` entry (`low | default | high`) |
| S21 (regions) | terrain, water and foliage per region; window scenery | RL-e's rings are per region and can render the ride's window scenery from landscape tags |

## 11.2 Quick wins now (bounded; each a small, reviewable change)

1. **Wind input is free.** When TW-b merges, `weather-today` already carries the hour's wind: no server work is
   needed for RL-d. Recommendation: start RL-a/RL-d as a **visual-track preview** (ARC-24) right after TW-b,
   without waiting for MVP-0 to close, because it touches only `clients/` and `presentation/`.
2. **Stop showing the promenade spike as "the world".** Its faceted mountains and flat lake (`05_lake_scenic.jpg`)
   are the least realistic frames in the repository. Label `./mineworld-3d` as the movement/camera spike in
   its header and README line (16c's SC-7 does the same for standalone). One-line change, no visual change.
3. **Fix the angular size of the slice's ridges** (a constants change in `street.gd`): move them to 5–20 km
   and scale heights to the real skyline's 1–2°, with fog colour matched to the sky's horizon colour. A
   cheap step toward VIS-REAL-2 before RL-e's real data, previewed and judged by the operator (it changes
   accepted frames, so it is the operator's call — QRL-11).
4. **12e's loose objects:** use the CC0 models already in the tree (`food_apple_01`, `croissant`,
   `carrot_cake`, `wine_bottles_01`) for kinds that exist in Market Town's items, per QS14-13's own ruling.
5. **Lakeside Deli's coloured boxes:** re-dress with the CC0 goods already committed (café goods, tea set,
   jars from Poly Haven), as a bounded item in 16g or the visual track. No new asset, no new licence.
6. **IL-d's `default` stays, Market Town's numbers move:** the first real-world values (gravity 9 807,
   per-material friction) can land as Market Town's own `configure/bodies.yaml` once IL-d merges, before
   RL-c's full list — content only.

---

# 12. Cross-lane impacts

- **S18 (IL-d, IL-i):** reserves `density`, `drag`; `realistic` reference list (RL-c) is a list, not a fork.
- **S19:** RL-a's module sits beside `world_time/`; TW-e owns sky, rain, fog, cloud; RL-d owns wind effects.
  The two meet in one Presentation Pack table (`step-19` §8.4's intent mapping gains a `wind` row read from
  `wind.gd`, not computed twice).
- **S14/S15 clients:** RL-b touches the slice's build (batching, LOD); it must land when no 16x/12e PR holds
  the same files (`slice_world.gd`, `batch.gd`, `streetscape.gd`); coordinate by the one-working-tree rule.
- **S16 (packages):** Asset Packs per region (ARC-7); RL-k extends ARC-55.
- **S20 (settings):** `display.quality`.
- **S21 (regions):** per-region terrain rings, water, foliage; window scenery; asset streaming.
- **AC-1:** `water` joins the generic-pack allow-list when Market Town enables it (ARC-35 note, as TW-a/TW-d).

---

# 13. Questions (QRL-n). **[OPERATOR]** marks operator-material ones; the rest the primary session may rule.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QRL-1 [OPERATOR]** | Which "mountains and river" did you judge: the slice's far ridges (`./mineworld-slice`), the older promenade spike's mountains and lake (`./mineworld-3d`), or both? | Both are addressed; the answer orders RL-e before or after RL-f. Recommend treating both as failing and retiring the spike from demos (§11.2 item 2) |
| **QRL-2 [OPERATOR]** | **Where is the default world?** TW-a and TW-b put Market Town in San Diego (its sun, its weather). The default style's references (`01`, `06`) show an alpine lake with snow peaks. Realism needs one: (a) San Diego's real terrain, coast, palms and chaparral for Market Town, and the alpine lakeside later as a second region (S21) with its own real DEM; (b) keep the alpine look and move the world's location to a real alpine lake town; (c) a fictional landscape. | **(a).** The world already *is* San Diego in time and weather; a fake alpine backdrop over San Diego's sun and rain contradicts the realism being asked for. The lakeside references stay authoritative for architecture, materials, light and mood (ARC-4), not for the skyline |
| **QRL-3 [OPERATOR]** | What water should Market Town have? (a) a river through or beside town (the San Diego River is real but not downtown), (b) a bay/coast edge (Mission Bay or San Diego Bay), (c) only distant decoration | **(b)** if QRL-2 is (a): a bay promenade fits both the references' waterfront and the real city; a small river or canal can follow |
| **QRL-4 [OPERATOR]** | Should items float, drift and should people wade (RL-g) — real authoritative water — or is water visual only for MVP-1? | **Authoritative**, after RL-f: "water you can wade through" is the operator's own example of classical mechanics |
| **QRL-5** | Gravity default: standard 9 807 or keep 9 810, and should Market Town override with the local 9 795? | **9 807** in `realistic`; no local override (0.13 % is imperceptible; one fewer content knob) |
| **QRL-6 [OPERATOR]** | Reference machines for the tiers on Windows and Linux: which physical machines can measure? And are the proposed tiers (Low = M1 / Iris Xe at 30 fps) right? | Accept the tiers; name at least one Windows laptop with Iris Xe and one Linux desktop with a GTX 1650-class GPU, or accept "macOS-measured only" in RL-i's evidence until they exist |
| **QRL-7 [OPERATOR]** | Where do large assets live — in this repository, Git LFS, or separate per-region Asset Pack repositories / release artefacts? | **Separate Asset Packs** (ARC-7), versioned and checksummed, fetched by a tool; keeps clone size small and fits S21's per-region streaming |
| **QRL-8 [OPERATOR]** | Default person mass and stature: US means (NHANES), world means, or per-person authored with a default? | **Per-person authored with a default of 70 kg and 1.70 m** (between the world and US means), each cited; a world chooses |
| **QRL-9 [OPERATOR]** | Should wind ever move objects authoritatively (a newspaper, an umbrella)? | **Not in S22.** Revisit after RL-g; if yes, bodies reads `weather`'s Public facts, never client gusts |
| **QRL-10** | Swell at the coast: presentation content only, or a disclosed sea state from data (CDIP buoys, Scripps)? | **Presentation content** until a coast is authoritative water with waves that move bodies |
| **QRL-11 [OPERATOR]** | May quick win 3 (re-scaling the slice's ridges) change `VIS-3D-GODOT-2`'s accepted frames before RL-e? | **Yes, as a preview** the operator judges; revert is one constant change |
| **QRL-12** | `tools/terrain-bake` in Rust (`tiff` crate) or Python (GDAL)? | **Rust** (`CLAUDE.md` §5 language choice; a developer tool like `tools/weather-fetch`); GDAL is a large native dependency on three OSes |
| **QRL-13** | Default renderer for the shipped 3D client: Forward+ (today) or Mobile with Forward+ as the High tier? | **Tier-selected** (RL-i): Mobile for Low, Forward+ for Default and High; a probe picks once, the user can change it |
| **QRL-14 [OPERATOR]** | Is CC-BY content (Sketchfab, Fab third-party CC-BY) wanted, given it needs attribution tracking (RL-k, ARC-55)? | **Not yet.** CC0 + owned output (Meshy paid, own photogrammetry, TripoSG) covers the goods; add RL-k only if a needed asset exists only under CC-BY |

---

# 14. Proposed records and amendments (text for the primary session; not applied)

## 14.1 `docs/DECISIONS.md` (placeholder ids)

- **ARC-RL-a — Realism has two halves: authoritative mechanics on the server, presentation physics driven
  by disclosure.** §4.1's rule and corollaries; §4.3's reason; INV-RL-1 … 5.
- **ARC-RL-b — Water is a System Pack; bodies reads it through a catalog.** §5.3; calm default without it.
- **ARC-RL-c — Physical defaults cite their reference; real-world values arrive as a named list.**
  §5.1–5.2, INV-RL-6, INV-RL-7.
- **DEP-RL-a — `tools/terrain-bake`: our own baker on the `tiff` crate**, rejecting a runtime terrain addon
  for the far field (Terrain3D kept for RL-j) — both directions of REUSE_POLICY §§11–12.
- **DEP-RL-b — Elevation data: USGS 3DEP (public domain), NASADEM outside the US**, Copernicus only with
  attribution tracking; a `NOTICE` per region.
- **DEP-RL-c — CC-BY attribution tracking** (only if QRL-14 says yes).
- **DEP-RL-d — Water and wind shaders are ours**, techniques credited (Sousa 2007; Waterways' flow maps,
  MIT, credited); GodotOceanWaves (MIT) for the High-tier coast if its prototype passes.
- **DEP-8 amendment** rows: excluded SpeedTree and The Grove (EULA not readable as redistributable);
  godotshaders.com per-shader check, GPL-3.0 excluded; ProtonScatter's demo textures excluded; generic
  unbranded goods only.

## 14.2 `overall.md` (MVP-1)

The S22 entry: output, dependencies (TW-b, IL-d, 16d, S20, S21), acceptance (VIS-REAL-1 … 4 accepted by the
operator; §3.2's physical-truth tests green; tiers met per OS), and the "Classical-mechanics realism" index
entry's pointer updated from "in planning" to this file's status.

## 14.3 `docs/ART_DIRECTION.md`

§3 "Realism Level" says "realistic, but intentionally not maximally realistic". The operator's 2026-10-09
requirement raises the bar for **environment and physics** (believable at distance, moving in the wind, real
goods), not for characters' faces (ARC-19 governs those). Proposed amendment: a dated note in §3 that the
environment target is "visually credible against the real place the world claims", with §21's environment
criterion extended to distance, motion and goods. If QRL-2 is (a), §22's sentence "the four approved
lakeside-town reference images are the canonical visual references" is scoped to architecture, materials,
light and mood.

---

# 15. Ledger

```text
E-RL-1  2026-10-09  ./mineworld-slice --perf on origin/main @ f80bbb7, Apple M5, Godot 4.7.2: §2.6
E-RL-2  2026-10-09  ./mineworld-slice --shots, 26 views, clients/3d-spike/shots/slice/ (ignored);
                    frames read: 17_street_from_east, 21_florist_interior, 14_west_frontage;
                    promenade spike frame 05_lake_scenic.jpg (committed)
E-RL-3  2026-10-09  research by three read-only sub-agents (audit; terrain/sky/water; foliage/props/
                    renderer); licences and dates as §6 states; rows marked unverified are not decisions
```

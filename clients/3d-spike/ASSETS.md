# Assets used by the 3D presentation spike

Every third-party file in this directory is **CC0 1.0** (public domain
dedication), from two libraries. Nothing here carries an attribution
requirement, a non-commercial clause, or a redistribution restriction.

Everything not listed as third-party is authored in-engine, from primitives,
in `scripts/`.

Re-fetch the third-party files with `tools/fetch_assets.sh` (idempotent).

---

## Third-party

### Poly Haven — https://polyhaven.com — CC0 1.0

Poly Haven states all its assets are CC0 and explicitly permits
redistribution. Verified against <https://polyhaven.com/license>.

**Textures** — `assets/textures/<slug>/`, 1k JPG, `diff` + `nor_gl` + `arm`:

| Slug | Used for |
| --- | --- |
| `concrete_pavement` | promenade and plaza pavers |
| `pavement_02` | shopfront sidewalk band |
| `cobblestone_floor_08` | side-street setts, shore trail |
| `asphalt_01` | roadway (material defined, currently unused) |
| `red_brick_03` | brick shopfronts (also retinted to a tan brick) |
| `beige_wall_001` | stucco / rendered walls |
| `stone_brick_wall_001` | plinths, kerbs, planter walls, quay, boulders |
| `weathered_brown_planks` | jetty, decking, bins, boat |
| `roof_slates_03` | roofs |
| `leafy_grass` | lawn and verges |

**Models** — `assets/models/<slug>/`, 1k glTF, loaded at runtime through
`GLTFDocument`:

| Slug | Used for |
| --- | --- |
| `painted_wooden_bench` | promenade and plaza benches |
| `outdoor_table_chair_set_01` | café pavement seating |
| `street_lamp_01` | promenade lamps |
| `street_lamp_02` | side-street lamps |
| `potted_plant_02` | shopfront planting |
| `celandine_01` | *(fetched, not yet placed)* |
| `grass_medium_01` | *(fetched, not yet placed)* |

**HDRI** — `assets/hdri/kloofendal_48d_partly_cloudy_puresky_2k.hdr`. The one
sky and the only ambient/reflection source in the scene.

### CharMorph "Vitruvian" — CC0 — `assets/characters/vitruvian/`

The rigged human. Baked out of a [VitruvianGodot](https://github.com/ibrews/VitruvianGodot)
clone by `tools/character_bake.py`; the clone itself is **not** in this tree.

**Read [`docs/CHARACTER_ASSET_AUDIT.md`](../../docs/CHARACTER_ASSET_AUDIT.md) before
touching this.** Upstream ships six Mixamo-derived animations baked into the body GLB
and six Mixamo source FBX beside it. None of them are here: the bake drops the
animation array and the clone stays outside the repository. What is here is the
CharMorph character — mesh, skeleton and skin weights — which `config.yaml` in
CharMorph-Vitruvian declares `license: CC0`, credits Sean Buckley and Olaf
Delgado-Friedrichs for, and which derives from the CC0 *Antonia Polygon*. The
`mixamorig:` bone names it carries are CharMorph's own "Mixamo (Game-Ready)"
compatibility preset, not Mixamo output; the audit shows the evidence.

The one gap, stated: the CharMorph-Vitruvian repositories carry no `LICENSE` file,
so CC0 rests on a machine-readable field in the shipped data plus a documented
relicensing permission plus a CC0 upstream.

### Quaternius Universal Animation Library (Standard) — CC0 — `assets/characters/`

`quaternius_ual.glb`, trimmed by `tools/animation_trim.py` to the four clips used
(`Idle`, `Walk`, `Jog_Fwd`, `A_TPose`) with the mannequin removed. The pack's own
`License.txt` ships alongside as `LICENSE.quaternius-ual.txt`.

Two deliberate deviations from the sourcing advice:

- **1k tier, not 2k.** These are all small props. 1k across a 1.8 m bench is
  ~570 px/m, already past the ~512 px/m target for a walkable street, and it
  roughly halves the repository.
- **Every imported prop is retinted** (`Props.retint`). Poly Haven's street
  furniture comes from their `hidden_alley` collection and arrives grimy and
  back-alley grey. Albedo is multiplied toward the promenade palette and
  roughness clamped to 0.30–0.80. Taking material authority back from the asset
  author is what stops borrowed props announcing that they were borrowed.

---

## Authored in-engine

No open licence covers these at a semi-realistic fidelity, so they are built
from primitives in `scripts/`:

| Thing | Where |
| --- | --- |
| Walking controller and the three cameras | `scripts/player.gd`, `scripts/camera_rig.gd` |
| The player character | `scripts/player.gd` -- it *is* `scripts/npc.gd`'s mannequin, built by the same `NPC.make()` from the same primitives and the same clothing palette, with a fixed seed and its gait driven by the controller's measured ground speed instead of by a path. No second character, and no new third-party file. |
| Buildings, shopfronts, doors, windows, fascias, awnings | `scripts/town.gd` |
| Broadleaf trees and conifers (alpha-cut card canopies) | `scripts/props.gd` |
| Leaf / needle / flower / grass / treeline cut-out textures | `scripts/procgen.gd` |
| Planters, bollards, bins, sandwich boards, hanging signs, wayfinders, railings, parasols, bicycles | `scripts/props.gd` |
| Fountain, jetty, rowing boat, boulders | `scripts/town.gd`, `scripts/props.gd` |
| Mountain ridge line (noise heightfield), far shore | `scripts/town.gd` |
| NPCs and the dog | `scripts/npc.gd`, `scripts/town.gd` |
| Lake surface shader | `shaders/water.gdshader` |

---

## Considered and rejected

- **Quality Godot First Person Controller v2** (MIT,
  <https://github.com/ColormaticStudios/quality-godot-first-person-2>) — a
  legitimate option, but it targets Godot 4.2–4.3 and its defaults (head bob,
  FOV kick, jump, sprint) are tuned for an FPS. Getting it to a walking-sim
  feel on 4.7 is comparable work to the ~120 lines in `player.gd`, and those
  120 lines are tuned to the brief: real human speeds, no jump, minimal bob.
- **Poly Haven `tree_small_02` / `jacaranda_tree`** (CC0) — correct look, but
  95 MB and 200 MB of mesh respectively at the 1k tier. Unusable when the scene
  needs thirty trees.
- **Poly Haven `boulder_01`, `metal_trash_can`, `fire_hydrant`** (CC0) — fine
  assets, dropped purely on repository size; equivalents are built from
  primitives with the same stone and metal materials, which is also more
  coherent.
- **Mixamo** — excluded on licence. Their terms permit use inside a project but
  not redistribution of the character and animation files as content, which is
  exactly what committing an FBX to a public repository does. No Mixamo file is
  in this tree.
- **Synty, Unity/Unreal marketplace packs, CGTrader/TurboSquid "free",
  Renderpeople samples, SMPL/AMASS/100STYLE** — excluded on licence.
- **A semi-realistic modular building kit** — does not exist under an open
  licence. Every CC0 kit is low-poly or PSX-era. Box geometry at correct
  proportions dressed in Poly Haven brick, stucco, painted wood and roofing is
  the intended path, not a fallback.
- ~~**Clothed, rigged, realistically-proportioned modern people** — likewise does
  not exist CC0.~~ **This was wrong, and it is the main correction this branch
  makes.** One does: the CharMorph "Vitruvian" character, listed above. The capsule
  mannequin that used to stand in for it is gone. `DEP-8` records the same gap and
  should be amended with it. **Mixamo, Synty, marketplace packs and the SMPL/AMASS
  family remain excluded on licence, and nothing from them is in this tree** — see
  the audit for how the Mixamo-derived parts of the upstream project were separated
  out.

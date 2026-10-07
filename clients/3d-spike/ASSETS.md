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

**HDRI** — `assets/hdri/qwantani_puresky_2k.hdr`. The one sky and the only
ambient/reflection source in the scene. It replaced
`kloofendal_48d_partly_cloudy_puresky`, which was a bright overcast sky: with it
every frame read grey and desaturated whatever the geometry did, and the plates
are warm, saturated and lit by a low sun. Changing the sky and dropping the sun
to 17 degrees was a larger step toward the reference look than any amount of
geometry would have been.

### Poly Haven, slice set (`VIS-3D-GODOT-2`) — CC0 1.0

Fetched by `tools/fetch_slice_assets.sh`, which names every file, and checked **one asset at a
time** by `tools/check_slice_provenance.py` and by eye. Checked 2026-09-30.

**The licence test (`DEP-8` as amended 2026-09-27) is whether we may relicense, not whether we
may redistribute.** CC0 1.0 is a waiver of all copyright and related rights to the extent the
law allows, with a fallback licence where it does not, so a CC0 file may be placed under MIT.
Poly Haven, at <https://polyhaven.com/license>, verbatim: *"Our assets are all licensed as CC0"*;
*"All assets (HDRIs, textures and 3D models) on this site are the original work of Poly Haven
staff, or artists who willingly and directly donate/sell their work to Poly Haven."*; *"You can
redistribute them, share them around, include them when sharing your own work, or even in a
product you sell."* Poly Haven's API terms (fetched from
`github.com/Poly-Haven/Public-API/blob/master/ToS.md`) state that the assets stay CC0 when
fetched through the API (*"CC0 assets carry no attribution requirement whatsoever, now or
ever"*) and ask API calls to carry an identifying User-Agent, which both tools send.

**What CC0 can and cannot cover.** A CC0 dedication waives the rights *its authors* held. It
cannot waive someone else's copyright reproduced inside the asset, such as a signed painting, a
banknote or a logo. That is not visible in the catalogue, so every model atlas was inspected as
pixels. Two failed, and are **excluded: not fetched, not on disk, not in this repository**:

| Slug | What the atlas contains | Why it fails |
| --- | --- | --- |
| `CashRegister_01` | a reproduction of a Bank of Canada $1 note (portrait, `CANADA` legend) in `CashRegister_01_diff_1k.jpg`; also `I ♥ LAS VEGAS` / `I ♥ QUEBEC` badges | banknote designs are the issuing bank's copyright; Poly Haven's authors (credited: Joe Seabuhr) could not dedicate it. The till in the café is authored in-engine instead (`cafe_interior.gd` `_till`) |
| `hanging_picture_frame_02` | a painting of a palace façade signed **"Celine F"** in `hanging_picture_frame_02_artwork_diff_1k.jpg` | the credited author is James Ray Cock; nothing establishes that the painter dedicated the painting to CC0. Removed from the café wall rather than replaced |

Atlases that contain imagery or lettering, inspected and **passed**:

| Slug | What was seen | Why it passes |
| --- | --- | --- |
| `fancy_picture_frame_01` | an unsigned Dutch-manner oil landscape (stepped-gable mill, bridge, riders) | an antique painting photographed by the credited authors (Rob Tuytel, Rico Cilliers); no signature, no modern artist named. **Residual:** the painting's date is not documented. Its style and craquelure point to pre-20th-century, and that is a judgement from the pixels, not a fact |
| `wine_bottles_01` | four labels: *Ftero Vinea*, *Clearwater Crystals*, *Intertwine* | invented brands. The Intertwine label's own copy names "the Haven Vineyards", which is Poly Haven's in-joke |
| `wall_clock` | dial numerals and a small `URBAN quartz` wordmark | numerals and a bare wordmark carry no copyright. A wordmark is a trademark question at most, and that does not bear on relicensing the file |
| `tea_set_01` | floral transfer decoration | generic period decoration on a scanned object, by the credited authors |

**Added 2026-10-06:** `cobblestone_floor_08` (Rob Tuytel), the grey setts of both pavements.
Already on disk for the promenade spike (`tools/fetch_assets.sh`); now also named in the slice
script so the slice's own check covers it. Its diffuse atlas was inspected at 1024 px: a
photograph of granite setts, soil joints and weeds, with no lettering, logo or artwork.

`check_slice_provenance.py` result for the set that remains: 13 textures and 33 models, each
found in Poly Haven's own catalogue as the asset type it is fetched as, each with credited
authors, every glTF dependency present locally, none external, nothing unreferenced, no
`asset.copyright` field in any glTF. **0 failed.**

**Textures** — `assets/textures/<slug>/`, 1k JPG, `diff` + `nor_gl` + `arm`:
`cobblestone_floor_08` · `rectangular_paving` · `concrete` · `worn_asphalt` · `sandstone_blocks_05` · `red_bricks_04` ·
`yellow_bricks` · `clay_roof_tiles_02` · `wood_floor_worn` · `brown_planks_09` ·
`wood_table_worn` · `long_white_tiles` · `painted_plaster_wall`. What each is for is in the
fetch script, beside the slug.

**Models** — `assets/models/<slug>/`, 1k glTF:
`wooden_bookshelf_worn` · `wooden_display_shelves_01` · `steel_frame_shelves_02` · `Shelf_01` ·
`round_wooden_table_02` · `dining_chair_02` · `painted_wooden_chair_01` · `bar_chair_round_01` ·
`wooden_crate_01` · `wicker_basket_01` · `hanging_industrial_lamp` · `modern_ceiling_lamp_01` ·
`industrial_pipe_lamp` · `croissant` · `carrot_cake` · `tea_set_01` · `wine_bottles_01` ·
`jug_01` · `wooden_bowl_01` · `food_apple_01` · `ceramic_vase_02` · `antique_ceramic_vase_01` ·
`brass_pot_01` · `pot_enamel_01` · `fancy_picture_frame_01` · `wall_clock` ·
`calathea_orbifolia_01` · `standing_chalkboard_01` · `planter_box_01` · `planter_pot_clay` ·
`shrub_02` · `shrub_03` · `water_manhole_cover`.

**One derived file per card-foliage model.** `shrub_02` and `shrub_03` draw their leaves on
cards whose outline is an alpha cut, but Poly Haven's glTF ships the base colour as a JPG, which
has no alpha. The fetch script therefore also downloads that asset's published `Alpha` map and
composes it with the `Diffuse` PNG into `textures/<slug>_diff_alpha_1k.png`. It is the same
asset's own data, so it is equally CC0. `scripts/slice/dressing.gd` swaps it in at load.

Also fetched, inspected and rejected **on appearance**, not on licence: `granite_tile_02`,
`white_plaster_rough_02`, `yellow_plaster_02` and `grey_cartago_02`. The reasons are in the
fetch script.

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

### Meshy route D+ candidate — AI-generated, owned (paid Meshy plan) — `assets/characters/meshy_d/`

The default character's own body slot (`Human.Body.REFERENCE`; only the player uses it).
One Meshy image-to-3D generation from the canonical reference, decimated to 95,000
triangles, base colour re-baked, rigged on the CharMorph skeleton above so the same bone
map and clips apply. **A candidate, not an accepted asset** (`ARC-9`): it is under
review as `VIS-3D-GODOT-1`. Provenance is kept in the GLB itself
(`scenes[0].extras.mineworld_provenance`, `"ai_generated": true`) and must survive any
re-export. Licence and the AI-identifier record:
[`presentation/mineworld-default/LICENSES/MESHY_ROUTE_D_CHARACTER.txt`](../../presentation/mineworld-default/LICENSES/MESHY_ROUTE_D_CHARACTER.txt);
method: [`docs/references/CHARACTER_ROUTE_D_PLUS.md`](../../docs/references/CHARACTER_ROUTE_D_PLUS.md).
`meshy_d_base_color.png` is Godot's extraction of the GLB's embedded image; Godot does not
overwrite it on reimport, so replace it together with the GLB.

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

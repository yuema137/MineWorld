# Step 26 — The cast: every person built with the protagonist's pipeline and standard

**Role:** step document for **S26 — the cast** (new step; to be added to [`overall.md`](overall.md) by the
primary session). It records the operator requirement, an audit of the real 3D and 2D cast, the route D+
tooling and the size and performance budgets, the architecture of per-person appearance, the pipeline
brought into the repository, the per-person standard written as checkable gates, the review protocol, the
budget, a PR split with integration checkpoints and adversarial criteria, risks, findings, proposed
decision records and the questions only the operator can answer.
**Effort:** `mvp0` for S26-a, S26-b1, S26-b2 (pilot), S26-cast, S26-c1, S26-c2, S26-d B1, S26-e and
S26-f; lakeside's 3D cast (B2) is MVP-1, except as café-street extras under E-2 (ruled, Q-S26-1) · parent:
[`overall.md`](overall.md).
**Lifecycle:** `STEP DESIGN FROZEN 2026-10-14 (operator Q-S26-1…12)`. Rulings in §16.1. Scope, invariants,
the gates and acceptance are frozen at step level; progress, evidence, findings and bounded corrections stay
writable. The step freeze does not authorize implementation, a Meshy call or an OpenAI call: each PR is
detailed to the commit and frozen in turn (`CLAUDE.md` §3.1), and starts under §12.1's start condition, in
a fresh session. Superseded lifecycle: `STEP DESIGN (DRAFT) — awaiting primary-session and operator review`.
**Author:** design session, 2026-10-14, worktree `/Users/yuema137/mineworld-worktrees/design-s26`, branch
`docs/s26-cast`. Audit base: `origin/main @ adf5d2c`.
**Ids** (assigned by the primary session at freeze, 2026-10-14):
`ARC-91…ARC-95` (architecture records), `DEP-51…DEP-55` (dependency records), all ten used (§15); local:
`SD-S26-n` (step decisions),
`G-n` (per-character gates), `AC-S26-n` (step acceptance), `X-n` (planted mutations), `Q-S26-n` (operator
questions), `R-S26-n` (risks), `F-S26-n` (findings), `B0…B3` (generation batches).
**Parallel-safety:** this file is the only file this session writes. Edits it needs elsewhere
(`overall.md`, `docs/DECISIONS.md`, `docs/HUMANOID_PROFILE.md`, `clients/2d/PRESENTATION.md`,
`docs/HUMAN_REVIEW_QUEUE.md`, `docs/MODULE_SPEC.md`) are **proposed** in §15 and made by the PRs that own
them.

---

## 1. The requirement

**Operator, 2026-10-14, verbatim:**

> 我发现现在除了主角，其他人物都很粗糙。请你保证每一个npc都是跟主角一样的流程和标准制作的

*Gloss:* "Apart from the protagonist, every other character is crude. Make sure every NPC is made with the
same process and the same standard as the protagonist."

**Operator clarification on keys, 2026-10-14:** agents may call the Meshy and OpenAI APIs by running
`source ~/.config/mineworld/secrets.env` inside the calling command. They never read, print, log, copy or
commit a key. The secret path never becomes a hard runtime dependency of MineWorld.

Read precisely, the requirement has three parts, and each is binding:

1. **Same process.** Every person is made by route D+ — a reference image, one Meshy image-to-3D
   generation, the game-ready mesh, our 52-joint armature, the weight fixes, the export with provenance —
   not by a cheaper second process ([`CHARACTER_ROUTE_D_PLUS.md`](../../../docs/references/CHARACTER_ROUTE_D_PLUS.md)).
2. **Same standard.** Every person passes what `VIS-3D-GODOT-1` passed when the operator accepted route D+
   as the interim standard on 2026-10-07 ([`HUMAN_REVIEW_QUEUE.md`](../../../docs/HUMAN_REVIEW_QUEUE.md)):
   the budget, the skeleton, the clips, deformation in motion, the hand and pack checks, the head-steadiness
   bound, the scorecard, and the operator's own look. No person is held to a lower one, and a cheaper
   variant exists only as an LOD of the same character, shown to the operator (§10.3).
3. **Every NPC.** That includes the decorative figures, the fallback for a person no pack knows, and the 2D
   cast (§§8, 9). A "crowd class" of cruder bodies does not survive this step.

## 2. Audit (verified on `origin/main @ adf5d2c`)

Every row was read in source or measured with `du`/`ls`. Where the brief's established facts differ from
source, the source wins and the difference is stated.

| # | Fact | Evidence |
| --- | --- | --- |
| A-1 | **Only the player uses the route D+ body.** `Human.Body { TOWN, REFERENCE }`; `REFERENCE` loads `res://assets/characters/meshy_d/meshy_d.glb`, `TOWN` loads `vitruvian/vitruvian.glb` | `clients/3d-spike/scripts/human.gd:27-41,269-271` |
| A-2 | `NPC.make(..., reference = true)` is the only route to `REFERENCE`, and `CharacterSlot._occupy` is its only caller with `true` | `npc.gd:148-158`; `slice/character_slot.gd:47-50` |
| A-3 | **Every other 3D person** is the CharMorph Vitruvian body with one CC0 skin albedo tinted per instance from six skin and six hair tints, with tops and legs from fixed palettes, and the garment geometry of the default character (hoodie and pack hidden by alpha 0) | `npc.gd:44-79,159-166`; `human.gd:266-345`; `ARC-22` |
| A-4 | Three creators of townspeople, not one: **(a)** the connected slice, `SliceLink._figure`: RNG seeded with `hash(id)`, fixed **1.72 m**, `Pose.STAND`; **(b)** the promenade spike, `Town._people`: RNG seed 4242, height `randf_range(1.62, 1.83)`, 6 walkers, 2 seated at café tables, 1 on a bench, 3 standing = **12**; **(c)** the slice's street, `Streetscape._people`: seed 40711, **4** figures at 1.74, 1.68, 1.72 (seated), 1.76 m | `slice/slice_link.gd:413-420`; `town.gd:585-627`; `slice/streetscape.gd:343-360` |
| A-5 | The comment at `_figure` says "appearance is chosen here from tags"; the code chooses it from `hash(id)` and ignores tags | `slice/slice_link.gd:410-415` |
| A-6 | **Per-body tuning lives in code constants, keyed by the enum:** `GRIP_*_D`, `GAZE_*_D` (`human.gd:244-268,333-340`); `FINGER_CURL_D` 62°, `ARM_OUT_D` 4°, `SPINE_DAMP_D` 0.6, `HEAD_STEADY_D` 0.9, `HEAD_TAU_D` 0.18 s (`posture.gd:25-37,130-160`); the Meshy material response (roughness 0.9, specular 0.25, `human.gd` `_meshy`) | source |
| A-7 | `meshy_d.glb` 10,120,872 B and `meshy_d_base_color.png` 6,142,149 B: **the same pixels twice** (embedded in the GLB, and extracted by Godot), per the licence file | `ls -la`; `presentation/mineworld-default/LICENSES/MESHY_ROUTE_D_CHARACTER.txt` |
| A-8 | The extracted base colour is imported with `compress/mode=0` (lossless), `mipmaps/generate=true`, `size_limit=0` — **not** the BC7 that step-23 §7.2 item 2 assumes for slice textures | `meshy_d_base_color.png.import` |
| A-9 | The GLB import has `meshes/generate_lods=true` and `create_shadow_meshes=true`, the committed bone map, and the six retarget options of `HUMANOID_PROFILE.md` §3 | `meshy_d.glb.import` |
| A-10 | Route D+ numbers: 95,000 triangles, one material `MW_Meshy`, 2048² base colour, no normal map, 52 joints; three Meshy A-pose generations at 30 credits each, **one of three kept every recognition cue** | `CHARACTER_ROUTE_D_PLUS.md` §§3, 8.1, 8.5 |
| A-11 | **Hand work per character in D+:** joint picks by hand on an orthographic render (`rig_meshy.py` `P` dict, `gameready_mesh.py` `EYE_Z_MESHY`); spine fitting corrected twice by drawing bones; rigid head region; rigid pack core with a width found by tearing; finger re-weight; pack side panels re-weighted twice (`ARMZERO`, `SMOOTH`); grip solved and re-solved; `ARM_OUT_D` set, reversed, set again; spine damping and head-steady added after the operator's finding | `CHARACTER_ROUTE_D_PLUS.md` §§8.2-8.9; `rig_meshy.py:31-46,181-200` |
| A-12 | The D+ scripts are **not versioned**: `/Users/yuema137/mineworld-demos/i23d/` (`meshy_i23d.py` 101 lines, `prep_inputs.py` 64, `gameready_mesh.py` 157, `rig_meshy.py` 264, `reweight_fingers.py` 75, `reweight_pack.py` 91, `rebake_head.py` 143, `fix_flecks.py` 107). Constants from one generation are hard-coded (`S, SOLE_M, SOLE_Z, CY`, the picks, the Meshy task id in the provenance) and the reference path points into another worktree | `wc -l`; source |
| A-13 | `meshy_i23d.py` reads `MESHY_API_KEY` from the environment only, never prints it, keeps `request.json` (without image bytes), `task.json`, the downloads and the balance before and after | source |
| A-14 | **No person file carries appearance data.** A person file has `name`, `routine`, `tags`, `note`, `location`, and per world `holdings`, `economy`, `job` | `worlds/*/people/*.yaml` |
| A-15 | **A world key never reaches a client.** Entity ids are allocated by a monotonic counter in creation order (places, people, items, organizations); the welcome frame carries `instance`, not the World Pack's id | `worldpack/src/load.rs:14-25`; `worlds/market-town/people/alice.yaml:3-5`; `server/PROTOCOL.md`, the welcome frame's `instance` |
| A-16 | What a client can read about a person: its id string, its tags, every component disclosed to it (`component_value`), and `display-name` from the `naming` pack, which discloses names publicly to every perceiver | `clients/protocol/mineworld/observation.gd:77-166`; `systems/naming/src/lib.rs:1-18` |
| A-17 | `ARC-46`: "The client never asks for a role by a world key (`alice`, `cafe`), only by tag, index or hash." 2D roles `person:<n>` are chosen by `posmod(id.hash(), cast_size)` | `docs/DECISIONS.md` ARC-46; `clients/2d/PRESENTATION.md` §3.1; `clients/2d/scripts/scene/people.gd` `_cast_index` |
| A-18 | **2D cast:** set `people` binds `person:0…8` to `gen_a`…`gen_j` (no `gen_i`) — **9 sprites for 12 people**, so hash collisions put two people in one look; `player` → `gen_player` | `presentation/mineworld-default/2D/assets/asset_bindings.yaml:15-29` |
| A-19 | **The 2D player is not the 3D protagonist.** `gen_player`'s prompt: "a warm amber-orange jacket over a cream shirt, teal-blue trousers, dark brown boots, short reddish-brown hair — he stands slightly taller than everyone else". The 3D protagonist is a young woman in a burgundy hoodie with a backpack | `presentation/mineworld-default/2D/art/provenance/gen_player_back.json` |
| A-20 | 2D sprites come from `tools/asset_generation` (`mineworld_gen`, OpenAI Images `/v1/images/edits`, `gpt-image-2.5-sunburst`, 1024×1536, high, transparent), conditioned on 2D reference crops, back view conditioned on the front; normalized so pixel height = metres × 64; `ARC-9` sidecar per file; credentials from the environment, else `MINEWORLD_SECRETS_FILE`, else the documented default file — a convention, not a dependency | `tools/asset_generation/README.md`; provenance JSON |
| A-21 | **Named identities.** `market-town` and `social-cafe` share the same 12 names and notes (diffed): Alice Moreau, Bob Achterberg, Carol Mensah, Dev Raman, Erin Walsh, Felix Okafor, Grace Liu, Hana Sato, Ivan Petrov, Otto Brandt, **Vera Lindgren (`visitor`, "The player")**, Wes Calloway (`wanderer`). `lakeside` has 10: Alba Ferreira, Bruno Lindqvist, Celia Okafor, Dario Russo, Elin Berg, Finn Halvorsen, Greta Novak, Hugo Marchetti, Ines Duarte, Nils Aaltonen. `bodies-yard` has 12 test keys (ada … lea), tag `walker`, no names | `worlds/*/people/*.yaml`, `diff` |
| A-22 | **The 3D client hosts one world.** `./mineworld-slice --world` starts `worlds/social-cafe`; the slice draws one café street. No 3D scene of lakeside exists | `mineworld-slice:18,111-139`; `slice/slice_world.gd:17` |
| A-23 | **The 3D client loads no Presentation Pack at runtime.** Its assets are Godot-imported resources under `clients/3d-spike/assets/`; `presentation/mineworld-default/3D/` holds references, manifest, identity contract and candidate evidence, and no `assets/` directory. The editor importer's BoneMap retarget (A-9) is an import-time feature | `ls`; `grep presentation clients/3d-spike/scripts` (comments only) |
| A-24 | **Size budget** (step-23 §7.1, frozen): `3d.pck` hard 250 MB compressed, goal 150; each bundle hard 400, goal 250. R-a measured `3d.pck` at 259.5 MB `gzip -6` with RL-b's textures; RL-b2 (frozen 2026-10-14, implementing in `impl-rl-b2`) targets **≈ 177 MB** (estimate), gate ≤ 250 MB (Q-B2-5) | `step-23-release.md` §7; `mvp1/pr-rl-b2-two-views-and-textures.md` §0, §0.1 |
| A-25 | **Perf budget** (RL-b R-2, binding): p95 ≤ 16.7 ms per view and video memory ≤ 2,048 MB on the operator's Apple M5 at 1920×1080, gi=voxel; draw calls ≤ 2,000 and primitives ≤ 3 M advisory. RL-b measured cafe frontage 17.11 ms and doorway 18.59 ms (over), video memory 1,608 MB; the frame is GPU-bound | `pr-rl-b2-two-views-and-textures.md` §0 |
| A-26 | D+ frame time: the Meshy body and the CharMorph body are indistinguishable at the display's 120 Hz cap (`INCONCLUSIVE` as a cost figure) | `CHARACTER_ROUTE_D_PLUS.md` §8.4 |
| A-27 | Repository: `.git` is 654 MB; no Git LFS (`.gitattributes` marks `*.glb` binary only) | `du -sh`; `.gitattributes` |
| A-28 | **16c** (the living street in 3D: walking figures, interpolation) is frozen and unmerged, waiting for 12d; it rewrites the figure path of `SliceLink`. **R-a** (export presets, packaging, size gate) is frozen and unmerged | `step-15-demo-3d.md` header, F-S14-4; `overall.md` Milestone F table; `git branch -r` |
| A-29 | A new System Pack is one crate, one line in `systems/installed`, and an entry in each world's `systems:` list; `naming` (369 lines) is the closest template: one section, one public component, no dependencies | `systems/installed/src/lib.rs:36-45`; `systems/naming/src/*` |
| A-30 | `HUMANOID_PROFILE.md` §4: clip ground speeds are **measured on the character** (`tools/measure_stride.gd`), because stride is leg rotation applied to that body's limb lengths; §2: instance scale = height / authored height | `docs/HUMANOID_PROFILE.md` |

## 3. Goals and non-goals

**Goals.**

- G-a. Every person the 3D client draws — named, unknown or decorative — is a route D+ character that has
  passed §6's gates, and the CharMorph tint crowd is no longer reachable from any person path.
- G-b. Each named person in `market-town`/`social-cafe` and `lakeside` is visibly a different person,
  cast from their biography and role (§7).
- G-c. Route D+ is versioned, reproducible tooling in the repository, with the hand fixes of A-11 replaced
  by automatic steps wherever the evidence allows and by recorded per-character parameters where it does
  not (§5).
- G-d. One 2D sprite set per person, made the way `gen_player` was made, and the same person in both
  clients (§9).
- G-e. The release stays inside a size budget the operator has ruled on, and the slice stays inside RL-b's
  16.7 ms (§§10, 11).

**Non-goals.** Blink, gaze, mouth movement and expressions (route D+ gave them up, §6 of the D+ record;
the cast inherits that limitation, and it is a later step — Q-S26-12). Cloth or hair simulation.
Hand-held props. A 3D scene of `lakeside` or `bodies-yard`. Runtime-loadable 3D Presentation Packs
(F-S26-3). Any change to the kernel, the protocol, or a System's rules.

## 4. Architecture: where appearance lives

### 4.1 The binding needs a key the client can read

The person → character binding lives in the Presentation Pack, as the 2D `asset_bindings` already does.
But a binding is keyed by something, and A-15/A-16/A-17 leave only three things a client can read: the
entity id, the display name, the tags. None of them identifies a person's appearance correctly:

| | Key | Why it is wrong |
| --- | --- | --- |
| K-1 | `hash(entity id)` mod pool (today's 2D and 3D rule) | distinct only by luck; 9 sprites for 12 people already collide (A-18); cannot say "Alice looks like this" |
| K-2 | the entity id itself, listed in the pack | ids are creation-order counters (A-15): adding a place renumbers every person, silently re-casting the world; a pack of opaque numbers is unreviewable |
| K-3 | the display name (`person:named:Alice Moreau`) | absent in worlds without `naming` (`bodies-yard`); two people may share a name; a renamed person changes face; and `ARC-31` records acquaintance-gated names as a later option, under which a stranger's face would change the moment they are introduced. **Appearance must not depend on what the observer knows** |
| K-4 | a tag (`tags: [barista, look:alice]`) | tags are semantic, read by Systems and by controllers; a presentation handle in them pollutes every consumer and has no owner |
| K-5 | **a disclosed `look` component, owned by a System Pack** | chosen (SD-S26-1) |

### 4.2 Decision: appearance is a perceivable trait of a Person, owned by an `appearance` System Pack (`ARC-91`)

```text
section     appearance             a person file's  appearance: { look: alice-moreau }
emits       appeared               at genesis, from the section; public
owns        Look                   `look`, reduced from `appeared` by this pack alone
discloses   Look                   to every observer who perceives the person — what someone looks like
                                   is visible to anyone who sees them
depends on  nothing
```

- **What a `look` is.** An opaque key — a lowercase slug, `[a-z0-9-]{1,48}` — naming the person's
  appearance. It is to appearance what `display-name` is to identity in speech. It is **not** a mesh, a
  file, a texture, a colour, a height or any other renderer concept: the world states *that* Alice has the
  appearance called `alice-moreau`, never what it is made of. A Presentation Pack resolves it; a pack
  that does not know it uses its fallback (§4.4).
- **Why a System Pack and not the kernel.** `CORE_CONCEPTS.md` §4.2 places slow-changing characteristics
  in Traits, "changed only through an explicit system"; kernel ignorance (`CLAUDE.md` §4 rule 2) keeps the
  concept out of the kernel. A later clothing, ageing or disguise System changes a look by emitting an
  event the `appearance` pack owns — the single-ownership rule holds.
- **Why it is world data at all.** Appearance is perceivable: the two reference clients, a third-party
  client and a controller all perceive *the same person* looking the same way. Keeping it out of the world
  forces every client to invent its own person identity (K-1…K-3), which is how 9 sprites came to serve 12
  people. The `naming` pack (`ARC-31`) is the precedent: "Nobody owned a name, so none reached a client."
- **Change amplification** (`CLAUDE.md` §4 rule 5): one crate (A-29), one installed line, one appended
  entry in each world's `systems:` list, one `appearance:` line per person file. No edit to `Person`, the
  kernel, the protocol, the loader or any other System.
- **Descriptors deliberately not added.** Apparent age, stature, build, skin and hair are not world facts
  in this step: no System consumes them (`CLAUDE.md` §4 rule 11, no premature abstraction). They live in
  the casting brief (§7.1), in the Presentation Pack. If a System later needs stature (reach, sightlines)
  it becomes a `bodies` section and presentation reads it then; recorded as F-S26-6.
- **What controllers see.** `look` is disclosed like any component. The cognition perceived-entity
  formatter must not present the slug as a name (S26-a checks `cognition/` for generic component
  rendering and either omits `look` or labels it as an appearance key). Look keys in the default cast are
  name slugs for review readability; if acquaintance-gated names ever land, the keys become opaque codes
  in one data migration (R-S26-9).
- **A world without the pack** is unchanged: people have no `look`, and every client uses its fallback.
  A world that enables the pack but omits a person's section gives that person no `look` — legitimate, the
  fallback draws them.

### 4.3 The cast binding, in the Presentation Pack (`ARC-92`)

**3D** — `presentation/mineworld-default/3D/characters/cast.yaml` (the `characters/` directory
`MODULE_SPEC.md` §6.1 already lists), written in YAML's JSON-compatible subset like `ARC-46`'s files:

```json
{
  "format": 1,
  "looks":    { "vera-lindgren": "vera-lindgren", "alice-moreau": "alice-moreau", "...": "..." },
  "extras":   [ "alba-ferreira", "bruno-lindqvist", "..." ],
  "default":  "vera-lindgren"
}
```

- `looks` maps a disclosed `look` to a **character id**. Several looks may map to one character only
  where the operator has accepted it (not in the default pack).
- `extras` is the fallback and decorative pool (§§4.4, 8). Every entry is a character that passed §6.
- `default` is the character the standalone slice and the `--portrait` captures use when no world is
  connected.
- A character id resolves to `characters/cast/<id>/character.yaml` (§4.5) plus its provenance, its
  casting brief and its approved reference images, all in the pack. The GLB and texture are imported
  resources of the 3D client at `clients/3d-spike/assets/characters/cast/<id>/` (F-S26-3: the 3D client
  cannot load a skinned, retargeted GLB from a runtime pack today; the binding is data in the pack, the
  meshes stay where the importer can apply the bone map). One location per kind of file, no copies.

**2D** — the client asks for `look:<key>` first, then `person:<n>` by hash, then draws plainly.
`asset_bindings.yaml` gains `look:<key>` roles in the `people` set. This **extends `ARC-46`'s role list**:
`look:<key>` is a disclosed component value, the same class of input as the tag in `facade:<tag>`, never a
world key — so `ARC-46`'s sentence "never by a world key" stands, and its list of role kinds gains one.

### 4.4 Resolution and fallback, identical in both clients (`SD-S26-2`)

```text
person with a `look` the pack binds        → that character / sprite
person with no `look`, or one the pack     → extras[ posmod(hash(id string), len(extras)) ]
  does not bind
pack binds no extras (3D) / no person:<n>  → the pack's `default` (3D) / plain drawing (2D, ARC-46 §1)
no pack at all (--presentation=none, 2D)   → plain drawing (unchanged)
```

**The fallback is never a cruder model class.** Every `extras` entry is a cast member or an extra made by
the same pipeline and passing the same gates. The CharMorph tint body leaves every person path; it stays in
the repository only as the rig source the pipeline fits from (§5.2 stage 7) and behind the existing
`--town-body` diagnostic until S26-f removes it from the release (§10.3).

### 4.5 The per-character manifest replaces `Human.Body` (`ARC-93`)

Everything A-6 keys by `Body.REFERENCE` becomes data in `character.yaml`, written by the pipeline and
checked by the gates:

```json
{
  "format": 1,
  "id": "alice-moreau",
  "scene": "res://assets/characters/cast/alice-moreau/alice-moreau.glb",
  "material": "MW_Cast",
  "stature_m": 1.66,
  "authored_height_m": 1.66,
  "clips": { "walk_mps": 1.04, "jog_mps": 2.61 },
  "posture": { "arm_out_deg": -4.0, "finger_curl_deg": 62.0, "spine_damp": 0.6,
               "head_steady": 0.9, "head_tau_s": 0.18 },
  "grip": null,
  "gaze": null,
  "surface": { "roughness": 0.9, "specular": 0.25 },
  "accessories": [ "apron" ]
}
```

- `human.gd` loads by character id; the enum, `SRC_D`, `GRIP_*_D`, `GAZE_*_D` and the `_D` posture
  constants disappear. The protagonist becomes `vera-lindgren` with her constants moved verbatim into her
  manifest — behaviour byte-identical, proved by AC-S26-6.
- **Authored at true stature, instance scale 1.0** (`SD-S26-3`). The pipeline scales each body to its
  brief's stature, so `HUMANOID_PROFILE.md` §2's instance scale is 1 and §4's clip speeds are measured on
  that very body (A-30). The client's `height` argument stops scaling a cast body; a request to draw a
  person at a different height than their character is a defect to report, not to stretch.
- `grip` and `gaze` are optional, per character, because only a strap-holder needs a grip and only a
  posed portrait needs a gaze. `accessories` drives which gates apply (G-12, G-13).

### 4.6 The two spatial gates (`CLAUDE.md` §4) and the reuse question

- *Can it support a Minecraft-like embodied 3D client without redesigning the kernel?* Yes: the kernel is
  untouched; a third client resolves `look` through its own pack.
- *Can both clients use the capability without duplicating game logic?* Yes: there is no game logic in it.
  Both clients run the same four-line resolution (§4.4) over their own pack.
- *Are we reinventing a wheel?* glTF, the Godot importer, its BoneMap and auto-LOD, Blender, Meshy,
  OpenAI Images and BiRefNet are reused; §5.4 compares the rigging and weighting options both ways.
- *Are we forcing a wheel?* Meshy's own rig and Mixamo's would force their skeleton and animation model
  on us; §5.4 records why they are not adopted as the rig, and what of them may still be used.

## 5. Route D+ in the repository

### 5.1 Where it lives and what it may touch

`tools/character_pipeline/` — production tooling under the same rules as `tools/asset_generation`
(`REUSE_POLICY.md` §6, `ARC-9`): no server, client, kernel, System or contract imports it; it is never a
runtime dependency; it runs on an operator's machine, on demand; it writes **candidates**, and only a
person's acceptance makes a candidate an asset. Python 3.12 driver; Blender (headless, ≥ the version D+
used, **verify** the exact version in S26-b1) for mesh stages; Godot 4.7.2 headless for the runtime gates.

```text
tools/character_pipeline/
  README.md                 human orientation (short)
  PIPELINE.md               the specification: stages, inputs, outputs, gates, provenance, secrets
  mwcast/                   the Python package: one module per stage below
  blender/                  the Blender scripts, ported from i23d/ and parameterized
  godot/                    headless gate runners (import check, A_TPose, stride, sweep, headtrace)
  recipes/<batch>.yaml      the batch: which character ids, which briefs, which retry budget
```

Per character, in the pack: `presentation/mineworld-default/3D/characters/cast/<id>/` holds `brief.yaml`
(§7.1), `reference_full.png`, `reference_portrait.png` (once approved), `character.yaml`, `fit.yaml`
(§5.3), `generation_log.yaml` (every run: task id, credits, verdict, reason), `scorecard.md`, and the
review frames. Out of the repository, per run, in a scratch root given by `MINEWORLD_CAST_SCRATCH`: the
unmodified Meshy download, `request.json`, `task.json` — kept because **a generation is not reproducible**
(A-13), never committed (size, §10.3; and nothing is ever stripped from them, `DEP-8`'s Meshy clause).

### 5.2 Stages

| # | Stage | From D+ | Automation in S26 | Output |
| --- | --- | --- | --- | --- |
| 1 | **Casting brief** from biography and role | — (the protagonist had a reference) | written by an agent from the person file, reviewed by the operator at the casting gate (§6.4) | `brief.yaml` |
| 2 | **Reference images** (OpenAI Images, `mineworld_gen`'s provider reused, not a second client): a full-body front image in a neutral stance with arms clear of the body (the Meshy input), then a chest-up portrait conditioned on the full-body image (the scorecard's framing). Conditioned on `04_character_closeup.png` and the 3D style references for **quality tier and style**, never for identity | — | prompt from the brief; 2–4 candidates per person; operator selects | `reference_full.png`, `reference_portrait.png` with `ARC-9` sidecars |
| 3 | **Mask** (BiRefNet, MIT, via `rembg`'s ONNX session; the non-commercial removers excluded by `DEP-8`) | `prep_inputs.py` | automatic; crop box from the alpha bounding box instead of hand-typed boxes | masked input |
| 4 | **Meshy image-to-3D** REST (`ai_model` pinned, `pose_mode` `a-pose`, no remesh, 2k texture, `remove_lighting`) | `meshy_i23d.py` | automatic; balance read before and after; credits logged; hard stop at the batch's credit cap | download in scratch |
| 5 | **Raw gate** on the multi-view thumbnails: hard-fail categories (`VISUAL_FIDELITY.md` §5) against the brief and reference | §3 identity check | agent judgement, recorded per run in `generation_log.yaml` (§6.5) | keep or reject |
| 6 | **Game-ready mesh**: scale to stature (`SD-S26-3`), weld, decimate to ≤ 95,000 triangles keeping UVs, re-bake base colour from the original at 2048² | `gameready_mesh.py` | automatic; the eye line and sole read from stage 7's landmarks, not typed; every number printed | `low.blend` |
| 7 | **Fit our armature**: the 52 `mixamorig:*` bones of the CharMorph export, names, parents and rolls kept, joint positions moved | `rig_meshy.py` (hand picks) | **landmarks detected** on the orthographic front and side renders (§5.4 L) plus the measured centre line; the spine kept at CharMorph's curve relative to the measured hips (D+ fix 1); every joint printed and drawn over the body for the gate | fitted armature |
| 8 | **Bind and fix weights** | heat weights + `set_rigid` + `reweight_fingers.py` + `reweight_pack.py` | heat weights (or weight transfer, §5.4 W); head rigid above the detected chin; per-finger inverse-distance weights (already automatic); pack core rigid and side panels `ARMZERO` + smoothed — run only when `accessories` names a pack, with the region found from the brief's accessory and the geometry behind the back; limit 4, normalize | bound mesh |
| 9 | **Bake fixes**: hairline fleck repair | `fix_flecks.py`, `rebake_head.py` | `fix_flecks.py` automatic and logged; `rebake_head.py` dropped (D+ §8.4 step 7 showed it had no effect) | texture |
| 10 | **Export** through `character_model.py`'s glTF settings, provenance in `scenes[0].extras.mineworld_provenance`; **texture stored once** (external PNG referenced by the GLB, so the bytes of A-7 are not committed twice — **verify** in S26-b1 that Godot's importer keeps the external reference) | `rig_meshy.py` export | automatic | GLB, PNG, `character.yaml` draft |
| 11 | **Runtime tuning, measured**: clip speeds (`measure_stride.gd`), `arm_out_deg` (the smallest outward angle at which no sleeve vertex enters the torso or pack in the walk and jog sweep), head-steady verified by `--headtrace` | `ARM_OUT_D`, §8.8 | automatic Godot headless runners | `character.yaml` final |
| 12 | **Gates** (§6) and the review frames | §§7-8 of the D+ record | automatic where measurable; agent scorecard; operator review | evidence |

What is reproducible: stages 3 and 6–12 are deterministic functions of the Meshy download and `fit.yaml`
(re-running them must give the same triangle count, joint positions within 1 mm and the same weight
groups — AC-S26-3). Stages 2 and 4 are samples; their inputs and outputs are kept, never re-run to
"reproduce".

### 5.3 Per-character parameters that cannot be automated

Some per-character knowledge is genuinely per character — a long coat's hem, an apron's straps, a hood.
It goes into `fit.yaml`, reviewed with the character, never into code: landmark overrides (with the
reason), extra rigid regions, the pack region's bounds, grip targets. A `fit.yaml` with more than three
overrides is a finding on the automation, reported in the batch's PR (the evidence for S26-b2's next
iteration), not silently accumulated.

### 5.4 Reuse, both ways

**R — rigging (who decides the skeleton).**

| Option | What it is | Verdict |
| --- | --- | --- |
| Our armature, fitted (D+) | the 52-bone CharMorph export; bone map, import settings and clip library unchanged | **kept as the rig** — it is the humanoid profile's runtime contract (`ARC-19`, `HUMANOID_PROFILE.md` §1) |
| Meshy's auto-rigging API | Meshy rigs a Meshy model and returns a rigged GLB/FBX with basic animations; its skeleton is Meshy's own (**verify** bone count and whether it has finger bones; **verify** the credit price); output is Customer Output under the paid plan (D+ §4) | **not adopted as the rig** (another skeleton means another bone map and another clip retarget, the forcing case); **evaluated in S26-b2 as a joint-position source**: its joint heads, mapped by name to ours, may replace landmark detection for the limbs |
| Mixamo auto-rigger | Adobe web service, Mixamo skeleton (the same `mixamorig:` naming family as ours) | **declined**: no API for an automated pipeline, and `DEP-8` commits no Mixamo file — a mesh rigged by Mixamo is one |
| AccuRIG (Reallusion) | free desktop auto-rigger | **declined for the pipeline**: GUI only (**verify** whether a CLI exists), so not reproducible by an agent |
| UniRig, RigNet (research auto-riggers) | learned skeleton and skin prediction | **declined for now**: need a CUDA GPU this Mac does not have (UniRig), or are older and copyleft (**verify** RigNet's licence); revisit if the pipeline moves to a CUDA machine |

**W — skin weights.**

| Option | Verdict |
| --- | --- |
| Blender heat weights + scripted fixes (D+) | the baseline; it worked for one character with five hand iterations |
| **Robust skin weight transfer** (Abdrashitov et al., SIGGRAPH Asia 2023 Technical Communications; reference implementation on libigl — **verify** the code's licence and libigl's Python binding licence before use): transfer CharMorph's production weights, fingers included, from the CharMorph body onto the fitted Meshy body by closest point with normal agreement, and inpaint where no match is confident (hair, pack) | **prototyped in S26-b2** against the baseline on the protagonist and the pilot; adopted only if it passes the same gates with fewer `fit.yaml` overrides (`DEP-53`) |
| Meshy rig's weights | evaluated with R above; weights for a different skeleton must be remapped bone by bone, and it lacks our finger chain if it has no finger bones (**verify**) |

**L — landmarks (replacing the hand picks of A-11).**

| Option | Verdict |
| --- | --- |
| MediaPipe Pose and Hand Landmarker (Google, Apache-2.0, **verify**) on orthographic renders of the A-pose body | **prototyped in S26-b2**; it reads shoulders, elbows, wrists, hips, knees, ankles, eye and finger tips from a rendered human |
| Geometric analysis only (cross-sections, limb principal axes — D+ already measures the centre line this way) | the fallback, and the cross-check: a landmark that disagrees with the geometry by more than 3 cm is printed and fails G-6 |
| Meshy rig joints | as R |

**M — decimation.** Blender Decimate (D+) is kept; Meshy's in-task remesh is excluded (D+ §3: it shredded
the hair). meshoptimizer is already in Godot's importer for LODs (A-9) and is not a second decimator here.

**I — reference images.** `mineworld_gen`'s OpenAI provider is reused (A-20); no second image client.

### 5.5 Secrets

- Keys are read from the environment only (`MESHY_API_KEY`, `OPENAI_API_KEY`), as `meshy_i23d.py` and
  `mineworld_gen` already do; an agent provides them by running `source ~/.config/mineworld/secrets.env`
  inside the calling command and never reads, prints, logs, copies or commits a key.
- No pipeline output contains a header, a key, or a response body that echoes one; error bodies are
  truncated as `meshy_i23d.py` does.
- **Not a runtime dependency**: nothing under `clients/`, `server/`, `systems/`, `kernel/`, `contracts/`,
  `worldpack/`, `persistence/`, `cognition/` or `sdk/` names the secrets path; the pipeline works with the
  variables exported by any means. Both are checked by a scan in CI (AC-S26-9).

## 6. The per-character standard (`ARC-94`)

The standard is `VIS-3D-GODOT-1`'s, as accepted on 2026-10-07, written as gates. **A gate's bound is a
fixed literal taken from the protagonist's accepted record or from a stated real-world reference, never
from the candidate being measured** (`ARC-23` rule 2). Every automatic gate first prints *where* it
looked (the region, the extrema, the vertex counts) before its verdict (`ARC-23` rule 1).

### 6.1 Automatic gates (scripted; CI re-runs the static ones on every committed character)

| Gate | Requirement | Bound's source | Instrument |
| --- | --- | --- | --- |
| G-1 Triangles | ≤ 100,000 in one skinned mesh | D+ §5.2 budget | GLB reader |
| G-2 Materials | exactly one material, `MW_Cast`, one base-colour texture; no other surface | D+ §8.1 | GLB reader; `human.gd` warns on any unbound surface |
| G-3 Texture | base colour exactly 2048 × 2048, sRGB; mipmaps on; the import settings the release policy (Q-S26-5) rules | D+ §5.2; A-8 | `.import` reader |
| G-4 Stature | sole-to-eye height = brief stature × 0.9366 ± 10 mm; soles at the profile's sole height ± 5 mm; the brief's stature itself within §7.2's realistic range | 0.9366 is the protagonist's measured ratio (1.655 / 1.767, `gameready_mesh.py`, `HUMANOID_PROFILE.md` §2) | Blender print; Godot bounds |
| G-5 Skeleton | the 52 bone names, parents and rolls of the CharMorph export, unchanged; bone map validates (`make_bonemaps.gd`); 0 unweighted vertices; ≤ 4 influences, normalized | `HUMANOID_PROFILE.md` §1 | Blender dump; Godot import |
| G-6 Joints | every fitted joint inside the body's surface and within 30 mm of the geometric cross-check; limb segment ratios within ±12 % of Winter's anthropometric segment lengths (upper arm 0.186 H, forearm 0.146 H, thigh 0.245 H, shank 0.246 H — D. A. Winter, *Biomechanics and Motor Control of Human Movement*; **verify** the edition's table) | realistic defaults; Winter | Blender print + bone overlay render |
| G-7 Import | `./mineworld-slice` loads the character with no import error and no unbound-surface warning; `A_TPose` renders a clean T-pose (arms level within 5°, legs straight, feet flat) | `HUMANOID_PROFILE.md` §3 | Godot headless + one frame |
| G-8 Clips | `Idle`, `Walk`, `Jog_Fwd`, `Sit`, the stand clips play; clip speeds measured on this body and written to `character.yaml`; `--drive`-style walk: planted-foot drift ≤ 25 % of body speed, contact ≥ 50 % of sampled frames | CharMorph measured 21 % / 61 % (`HUMANOID_PROFILE.md` §4) | `measure_stride.gd`; drive runner |
| G-9 Deformation | in the stand, idle loop, walk and jog sweeps (eight phases each, D+ `--sweep`) and the sit pose: no triangle stretched beyond 3× its rest edge length outside the shoulders and hips' known ranges, located and printed; no sleeve, hand or leg vertex inside the torso or pack volume beyond 5 mm | D+ §§8.7-8.9 (no tear, no clipping) | Godot sweep runner |
| G-10 Head | steady walk and jog: head yaw, pitch and roll in the body frame each ≤ 6° peak-to-peak | D+ §8.8 bound | `--headtrace` |
| G-11 Hands | every finger bone carries ≥ 10 vertices (each thumb bone ≥ 5) weighted > 0.5 to it; the curl closes the hand (fingertip-to-palm distance shrinks by ≥ 40 % at the curl angle) | D+ §8.6 counts (fewest outside the thumb: Pinky 14; Thumb 6 on one bone — so ≥ 10 per finger bone, ≥ 5 per thumb bone) | Blender weights; Godot pose |
| G-12 Pack / bag (if `accessories` names one) | pack vertices carry no upper-arm or forearm share; no slit or ridge in the jog sweep at 2×; sleeve and side panel never interpenetrate in the walk sweep | D+ §§8.7 step 6, 8.9 | weights + sweep |
| G-13 Seated (if the person sits in any world or decor) | the `Sit` pose: thighs, hips and garment hem without tear; seat contact within ± 20 mm of the seat height | A-4 (people sit at tables, benches, a counter stool) | sit runner |
| G-14 Provenance | `scenes[0].extras.mineworld_provenance` with `source_type: generated`, `ai_generated: true`, generator and model, Meshy task id, the reference images by path and SHA-256, the OpenAI model, the post-processing; a `LICENSES/` entry per batch; an `ASSETS.md` row; no key-shaped string | `ARC-9`; `DEP-8`; D+ §4 | GLB reader; scan |

### 6.2 The scorecard (agent, then operator)

Per character, against **that person's approved reference** (`ARC-17`: the reference is the identity source
of truth), at the reference's own framing (`VISUAL_FIDELITY.md` §8), from runtime frames in the client's own
lighting (§10 of that document: front or three-quarter gameplay view, rear, the portrait framing, a
side-by-side sheet). Rows, each with the reference fact, the candidate fact, the frame, and `PASS` /
`PARTIAL` / `FAIL`:

```text
Face identity        face shape, eyes, nose, mouth, apparent age, distinguishing marks
Hair silhouette      the brief's hairstyle category and outline
Hair colour          major colour, highlights or grey
Outer garment        category and defining structures
Under layer          colour, layering, prints
Legwear, footwear    category and colour
Accessories          every accessory the brief names, and how it sits
Material quality     skin, hair, cloth — at the protagonist's tier, judged beside her (§6.3)
Overall identity     VISUAL_FIDELITY.md §9's question, answered yes or no
Overall vibe         the brief's one-line manner
```

The banned comparative language stays banned; the largest miss is listed first in a viewer's words.
**To reach the operator a character needs**: no `FAIL` row, no hard-fail category (`VISUAL_FIDELITY.md`
§5, including the quality tier), overall identity `PASS`, face identity `PASS` or `PARTIAL` — exactly the
state in which the protagonist was previewed and then accepted. Only the operator accepts (`ARC-11`).

### 6.3 Quality-tier parity

Every review sheet carries **Vera at the same framing in the same lighting**, as the control. The tier
question is asked of the pair: "is this person at the protagonist's quality tier?" A character that reads
cruder than Vera beside her fails the quality-tier hard-fail category, whatever its own scorecard says.

### 6.4 Operator review: two gates per batch (`SD-S26-4`)

About 21–29 characters (§8) cannot be reviewed one conversation each. Two gates per batch, both before
the expensive or irreversible step they protect:

1. **Casting gate — before any Meshy credit is spent.** One page per batch: per person, the brief
   (role, apparent age, stature, build, palette, outfit, accessories, one-line manner) and 2–4 reference
   candidates side by side, plus the batch's distinctness lineup (§7.3). The operator picks one candidate
   per person, or asks for a redo with a reason. Expected time: under a minute per person.
2. **Runtime gate — after §§6.1–6.3. Parked by operator rule (2026-10-14, Q-S26-6):** every hands-on
   and visual check is batched into one big milestone, so this page is prepared when a batch is ready and
   **delivered with the next big milestone's single checklist**, not asked on its own. One page per batch: an orientation index (all characters at street
   distance with Vera, a contact sheet — for orientation only, never a verdict, `VISUAL_FIDELITY.md` §8),
   then **one full-resolution side-by-side sheet per person** (reference portrait | runtime three-quarter
   portrait | front | full body | rear | a walk frame at 2×), each carrying the scorecard and the known
   misses, and one accept / redo / reject per person.

**Ruled (Q-S26-6):** two review pages per batch. The casting page is a pre-spend decision and is asked
as soon as it is ready. Each page is a private HTML review page recording a per-person verdict, mirrored
into `HUMAN_REVIEW_QUEUE.md`. Until the runtime verdict arrives, a batch's characters may be merged and
used as **candidates** (`ARC-9`), and later batches proceed; nothing is called accepted before the
operator says so. The verdicts are recorded in `HUMAN_REVIEW_QUEUE.md` under a
new `VIS-3D-CAST-<batch>` milestone, and an accepted character's `ASSETS.md` row changes from candidate to
asset only then (`ARC-9`).

**Pose sheet per person (operator, 2026-10-15; binding).** The operator's words: "npc人物形象设计好之后，你需要截图给我看，
每个npc都要换几个不同动作截一下图，而且最好有坐在椅子上等等不同姿势". In English: once each NPC's look is done, the operator
wants screenshots of every NPC in several different actions and poses, including sitting on a chair.

Every person's runtime sheet therefore adds a **pose row**: in-client screenshots from the 3D client at
the reference framing and lighting, using the same clips and rig as Vera. The row has these frames:

| # | Pose / action | Framing |
| --- | --- | --- |
| P-1 | idle stand (A-pose relaxed) | front, three-quarter, rear |
| P-2 | walk, mid-stride (contact and passing frames) | side |
| P-3 | run or brisk walk, mid-stride | side |
| P-4 | **seated on a café chair** at a table (G-13), hands on the table | three-quarter |
| P-5 | seated on a bench or counter stool, whichever the person's world has | three-quarter |
| P-6 | talking / gesturing to another person (the conversation pose) | two-shot with Vera |
| P-7 | reaching or handing an item over the counter (the buy/give interaction) | three-quarter |
| P-8 | face close-up at conversation distance (2.5 m) | front |

Rules for the pose row:
- Every frame is rendered by a scripted, deterministic probe (`--cast-sheet <look>`), not hand-posed, so
  it can be re-shot after any change.
- Every frame is checked for clipping and tearing by the same detectors as G-9 and G-13 before it reaches
  the sheet. A frame that fails is shown, red-flagged, never hidden.
- A person whose world has no seat (none at present) still gets P-4, because seats are common decor.
- The pose sheet is part of the runtime gate, so it is batched into the big milestone like the rest of
  that gate. The pilot's three people (S26-b2) get their pose sheets first.
- Acceptance additions:
  - X-26: a planted missing seat-contact fails P-4 and is flagged red on the sheet.
  - X-27: the sheet for a person is byte-identical when re-shot at the same commit.

### 6.5 A failed generation (a run is a sample)

```text
run k of a person (k ≤ the batch's retry budget, default 3)
  │
  ├─ stage-5 raw gate fails a hard-fail category ──→ logged with the reason; next run
  ├─ stage 6–12 gate fails on a defect the pipeline can repair (bake, weights, fit) ──→ repair, re-gate;
  │      repair is the agent's to finish (ARC-24), never shown as a preview
  ├─ scorecard below §6.2's bar ──→ logged; next run
  └─ passes ──→ runtime gate
after the budget for one person is spent without a pass:
  1. diagnose: a defect of the reference (occluded accessory, ambiguous hair, arms against the body)
     → regenerate the reference image and take it back through the casting gate (OpenAI cost only);
     or of the parameters (e.g. multi-view input) → one documented parameter change;
  2. one more budget of runs;
  3. still failing → the person is BLOCKED, reported to the operator with every run's thumbnails and
     reasons and a proposal. Never substituted by a cruder body, never shipped as "close".
```

Every run is logged in `generation_log.yaml` (task id, credits, verdict, reason) and its download kept in
scratch, so the batch's PR states the real retention rate and the real cost per accepted character.

## 7. Distinctness

### 7.1 The casting brief (`brief.yaml`, Presentation Pack)

```yaml
person: alice-moreau            # the look key
from: [worlds/market-town/people/alice.yaml, worlds/social-cafe/people/alice.yaml]
role: barista                   # from tags and note
apparent_age: 30-35
stature_m: 1.66                 # sole to crown, hair excluded (HUMANOID_PROFILE.md §2)
build: slim-average
skin: <set from the biography, gated by the operator at casting (Q-S26-7, ARC-95)>
hair: { category: tied-back bun, colour: dark brown }
outfit: { outer: bib apron, under: rolled-sleeve oatmeal shirt, legs: dark trousers, feet: clogs }
accessories: [apron]
manner: brisk and friendly
avoid: [glasses, hand-held props]   # §7.4
```

The brief is the person's identity contract until a reference is approved; then the approved reference
is (`ARC-17`), and the brief records why each choice follows from the biography.

### 7.2 Realistic defaults

Body proportions and statures follow real-world values (the standing rule: realistic defaults, with the
reference cited):

- **Stature.** Adult means by country range from about 160 to 183 cm for men and 149 to 170 cm for women
  (NCD Risk Factor Collaboration, "A century of trends in adult human height", *eLife* 2016;5:e13410 —
  **verify** the exact extremes); within a population the standard deviation is about 7 cm (**verify**).
  The default cast stays within mean ± 2 SD of a European population for its sex unless the brief says
  why: women 1.55–1.78 m, men 1.66–1.92 m.
- **Age.** Stature falls with age, by roughly 1–3 cm by the seventies and more after (longitudinal data,
  e.g. Sorkin, Muller & Andres 1999, Baltimore Longitudinal Study of Aging — **verify**); the oldest cast
  members are briefed shorter accordingly, and slightly stooped where the biography says so.
- **Segments.** G-6's Winter ratios. Route D+'s body measured 0.56 m shoulder to wrist at 1.767 m
  (D+ §8.2), close to Winter's 0.332 H = 0.587 m; CharMorph's 0.48 m was the short one.

### 7.3 The distinctness rule

- Any two people **in the same world** differ in at least **three** of: apparent age band (decade), stature
  (≥ 5 cm), build, skin tone, hair (category or major colour), outer-garment category or dominant colour.
  Checked on the briefs at the casting gate (a table the tooling prints) and again on the runtime lineup.
- **The lineup**: the world's whole cast and Vera side by side at street distance (10 m) and at
  conversation distance (2.5 m), front and rear, in the client's lighting — the frame in which "they all
  look the same" is visible or not.

### 7.4 What the brief avoids

Features route D+ cannot carry are kept out of briefs rather than discovered as failures: transparent
glasses lenses (Meshy fuses them), hand-held props (no attachment system, and a welded hand), loose long
hair over the shoulders (rigid to the head, D+ §8.2), long skirts or coats for people who sit (one fused
surface with the legs; G-13). Where the biography wants one of these, the brief says what replaces it.

### 7.4a One style, one precision, one method; real-world diversity (operator, 2026-10-15; binding)

The operator, verbatim: "我们一定要保证所有人物画风一致，但是特征有多样性（高矮胖瘦，性别 族裔，符合真实世界分布），
然后精度一致，建模方法和工具一致". In English: every character must share one art style, while their features are
diverse (tall/short, slim/heavy, sex, ethnicity) **following real-world distributions**, with one precision and
one modelling method and toolset. This section makes that checkable. Where it is stricter than §7.2 or §7.3, it
governs. It amends §7.2's default population (`ARC-95` dated note, written by S26-cast).

1. **One art style.**
   - Every reference image is generated from **one style template**: a fixed prompt preamble covering
     rendering style, A-pose, neutral expression, camera height, focal length, framing, lighting rig,
     background and output size. Vera's approved reference image is passed as the **style anchor** to
     the image model for every person.
   - The template and the anchor are versioned files in the Presentation Pack
     (`characters/style/template.yaml`, `characters/style/anchor.png`). Changing either is a new
     template version, and every character records the version it was cast with.
   - Every Meshy run uses the same model version and settings as Vera's (route D+, `DEP-51`).
   - The casting page and the runtime lineup show each person beside Vera at the same framing. The
     operator's verdict on style is recorded per person (`ARC-94` review protocol).
2. **One precision.**
   - The gates G-1…G-14 apply unchanged to every person.
   - A batch also fails if its people diverge more than Vera's own record allows, on any of:
     - triangle count: 85 000–100 000;
     - texture size: 2048²;
     - texel density on the body: within ±15 % of Vera's;
     - material count: 1;
     - skeleton: the same 52 joints.
   - These numbers are fixed from Vera's record before any generation, and never from a candidate.
3. **One method and toolset.**
   - Every character in a batch carries the same pipeline version and the same tool versions in its
     provenance (G-14): `mwcast`, Blender, Godot importer, Meshy model, image model.
   - A batch with mixed versions is refused by `cast.py check-characters` with a named error.
   - Re-casting one person under a newer pipeline re-runs the whole batch's gates, never that person alone.
4. **Real-world diversity, by data.**
   - A world's cast is drawn from a **population profile**: a versioned data file per setting, with
     every figure cited. Examples: `characters/populations/san-diego-county.yaml` for the café town and
     market town, whose weather already comes from San Diego (TW-b).
   - The profile states:
     - the sex ratio;
     - the adult age distribution;
     - the ethnicity / ancestry mix: for the United States, the census race and Hispanic-origin
       categories, for example ACS 5-year estimates for the county;
     - stature by sex and age band, mean and SD: for example NHANES anthropometric reference data;
     - a build (BMI) distribution by sex: for example NHANES.
   - Every figure is marked **verify** until read at its source by S26-cast.
5. **A seeded, stratified sampler.** It assigns the attributes a person's biography does not fix, so
   that even a 12-person cast approximates the profile:
   - it is deterministic from the world seed and the look key;
   - its strata are allocated by largest remainder;
   - it never stereotypes from a surname, and the biography wins where it specifies.
   - Each brief records which attributes came from the biography and which came from the sampler.
   - The casting page prints the cast's realised distribution against the profile.
6. **Infra, not a fixed cast.**
   - The profile, the template, the anchor and the sampler seed are parameters of the Presentation Pack.
   - A user's world may name a different profile, for example another city or a fantasy setting with
     its own cited or authored figures, without code changes.
   - A missing or malformed profile fails by name.

`S26-cast` owns items 4–5 and the profile file. `S26-b2` owns items 1–3 for the pilot, and `S26-d` owns
them for each batch. The acceptance tests are X-22 to X-25:

- **X-22.** A batch with one person cast under a different template version is refused.
- **X-23.** A candidate with 70 000 triangles fails the precision band.
- **X-24.** The sampler, re-run with the same seed, reproduces the briefs byte for byte. A different
  seed changes only sampled attributes.
- **X-25.** For a synthetic 1 000-person cast, the realised distribution is within 2 percentage points
  of the profile on every stratum.

### 7.5 A first casting sketch (for the casting PR to refine; skin and hair set per person at casting, Q-S26-7)

| Look | Role (source) | Age | Stature | Build | Outfit direction |
| --- | --- | --- | --- | --- | --- |
| vera-lindgren | the player (`visitor`) | young adult | as accepted | as accepted | **exists**, the protagonist |
| alice-moreau | barista | 30s | 1.66 | slim-average | apron, shirt, hair tied back |
| bob-achterberg | counter regular | 60s | 1.82 | heavy | cardigan, flat cap |
| carol-mensah | resident who knows everyone | 50s | 1.63 | average | long cardigan, trousers |
| dev-raman | runner | late 20s | 1.76 | lean athletic | running top and tights, trainers |
| erin-walsh | reads on a bench "whenever it is dry" | 40s | 1.69 | average | light rain jacket, scarf |
| felix-okafor | neighbour who forgets things | 30s | 1.79 | average | casual jacket, jeans |
| grace-liu | office commuter | 30s | 1.62 | slim | blazer, trousers, shoulder bag (G-12) |
| hana-sato | office commuter | 20s | 1.58 | slim | shirt, knee-length skirt, bob cut |
| ivan-petrov | talkative passer-by | 70s | 1.72 | stocky | overcoat, scarf |
| otto-brandt | top-floor resident, rarely down | 80s | 1.69 | thin, slight stoop | cardigan, slacks |
| wes-calloway | second player seat, "passing through" | 20s–30s | 1.86 | tall | amber jacket, travel pack (G-12) — from the 2D `gen_player` (ruled, Q-S26-9) |
| alba-ferreira | baker | 40s | 1.60 | sturdy | apron, rolled sleeves |
| bruno-lindqvist | carpenter | 50s | 1.84 | broad | work trousers, flannel shirt |
| celia-okafor | schoolteacher | 40s | 1.70 | average | cardigan, trousers |
| dario-russo | fisherman | 40s | 1.74 | wiry | waterproof jacket, knit cap, boots |
| elin-berg | summer painter | 20s | 1.69 | slim | paint-marked linen shirt, sun hat |
| finn-halvorsen | boatbuilder | 30s | 1.88 | strong | dungarees |
| greta-novak | the village's oldest | 80s | 1.56 | small, stooped | headscarf, cardigan, rubber boots |
| hugo-marchetti | ferryman | 60s | 1.76 | solid | knit sailor's jumper, cap |
| ines-duarte | village nurse | 30s | 1.64 | average | off-duty tunic, light jacket |
| nils-aaltonen | fisherman who sleeps little | 50s | 1.79 | lean | fishing vest, beanie |

## 8. Decorative figures and the fallback pool

The 12 promenade figures and the 4 slice figures (A-4) and every person without a bound look draw from
`extras` (§4.3). Two ways to fill it, both on the same pipeline and standard; **ruled E-2** (Q-S26-8):

- **E-1 — dedicated extras.** About 8 more characters made for the pool. Clean semantics (an extra is
  nobody), but +8 generations and +8 characters in the release.
- **E-2 — the other world's cast** (ruled). The café street's extras are the lakeside people, and a
  future 3D lakeside's extras are the café town's. No named person of the present world ever appears twice
  (the pools are disjoint from the world's looks), no extra generation, and the lakeside 3D cast is used
  from the first day instead of waiting for a 3D lakeside (A-22). Its limit: a world that is neither needs
  its own pool — that is the moment E-1 is built.

Decor that sits (A-4: café tables, bench, terrace chair) draws only from extras that passed G-13.
`bodies-yard` (12 unnamed test persons, no `appearance` section) exercises the fallback path with real
characters (ruled, Q-S26-4).

## 9. 2D parity

- **One sprite set per person** — four images (`_front`, `_front_b`, `_back`, `_back_b`), made exactly as
  `gen_player` was: `mineworld_gen` with `gpt-image-2.5-sunburst` `/v1/images/edits` at 1024×1536 high,
  conditioned on the 2D style references; back views conditioned on the front; normalized so pixel height =
  stature × 64 (the brief's stature, not a default); `ARC-9` sidecar per file (A-20).
- **The same person in both clients.** Each 2D prompt is written from the person's brief and conditioned
  on their approved 3D reference portrait for identity (hair, colours, garments), drawn in the 2D style.
  Different art, same person (`ARC-3`: the packs share meaning, not art).
- **The player follows the Person, not the client** (INV-1): the observer is drawn with the look of the
  Person it occupies — Vera for `visitor`, Wes for `wanderer` — in 2D and 3D. The `player` role stays
  only as the fallback for an observer with no bound look. Consequence: today's `gen_player` (a man in an
  amber jacket, A-19) stops being Vera's 2D image and becomes Wes's 2D look and the identity source of his 3D brief (ruled, Q-S26-9).
- Bindings: `look:<key>` roles in the `people` set (§4.3); `person:<n>` stays for unbound persons.
  The 2D bundle stays far inside its 40 MB limit (sprites are ~110 px tall after normalization).

## 10. Budget

### 10.1 Meshy credits (estimates; the pilot measures the real numbers)

| | Basis | Credits |
| --- | --- | --- |
| Per run | 30 (D+ §3, image-to-3D, no remesh, 2k texture) | 30 |
| Runs per accepted character | D+ retained 1 of 3 → geometric mean 3 runs (**the pilot replaces this with its own rate**) | ≈ 90 |
| Pilot B0, 3 characters | 3 × 90, plus S26-b2's rigging comparison (Meshy rig on ≤ 3 models, **verify** its price) | ≈ 300 |
| Named cast after the pilot: 21 new − 3 = 18 | 18 × 90 | ≈ 1,620 |
| E-1 extras (not built: E-2 ruled, Q-S26-8) | 8 × 90 | ≈ 720 |
| **Total** E-2 / E-1 | | **≈ 1,920 / ≈ 2,640** |
| **Ruled cap** (Q-S26-3) | hard cap for the whole step; reported at 50 % and 100 %; the pilot's measured rate re-estimates B1 and B2 before they start | **3,000** |

The monetary price per credit depends on the operator's Meshy plan (**verify**; not assumed here).

### 10.2 OpenAI images (estimates)

Per person: 2–4 full-body candidates + 1–2 portraits (3D reference) + 4 sprites (2D), each possibly
redone once → about 10–20 images per person at 1024×1536 high. At a per-image price the S26-b1 PR reads
from OpenAI's current price list (**verify**; not recalled here), 22–30 people × 20 images is the
ceiling to budget. **Ruled (Q-S26-3):** S26-b1 reads the current per-image price and states a dollar cap in its PR for the primary session to confirm; no image is generated before it is confirmed.

### 10.3 Release size against the 250 MB `3d.pck` (estimates, flagged)

| Item | Per character | Basis |
| --- | --- | --- |
| Base colour, 2048² | ≈ 5.6 MB as BC7 with mips (4 194 304 B × 4/3), barely compressible (R-a §18.7); ≈ 4–6 MB lossless as imported today (A-8) | arithmetic; A-8 |
| Mesh, ~47 k vertices, 95 k triangles, with LODs and shadow mesh | ≈ 2–3 MB raw, less compressed | estimate from Godot's compressed vertex format; **measure** in B0 |
| **Per character** | **≈ 7.5–8.5 MB** | sum |

| Scenario | Characters shipped | Added | `3d.pck` estimate (RL-b2's ≈ 177 MB, − CharMorph ≈ 11 MB) |
| --- | --- | --- | --- |
| S-1: café town only (the one 3D world), E-2 extras = lakeside | 11 + 10 = 21 | ≈ 160–180 MB | **≈ 330–345 MB — over the 250 MB hard limit** |
| S-2: café town only, E-1 with 4 extras | 15 | ≈ 115–130 MB | ≈ 280–295 MB — over |
| S-3: café town 11, no extras (fallback = café cast, duplicates allowed) | 11 | ≈ 85–95 MB | ≈ 250–260 MB — at the limit; and duplicates break §8 |

**So the full standard does not fit the current budget, and nothing below lowers the standard silently.**
The options, each shown to the operator with numbers measured on the pilot (Q-S26-5):

- **O-1. Basis Universal (UASTC + RDO) for cast textures.** Godot 4.7.2's importer exposes
  `compress/uastc_level` and `compress/rdo_quality_loss` (A-8's `.import` fields). On disk a fraction of
  BC7's 8 bits per pixel (**measure**; ~2–3 bpp is the range to expect), transcoded to a GPU format at load.
  A visual change at the texel level: before/after portraits at the reference framing go to the operator.
  If it holds: ≈ 4 MB saved per character → S-1 ≈ 245–260 MB.
- **O-2. LOD of the same character.** Godot's import LODs (on, A-9) cut distant triangles at runtime; they
  save frame time, not disk. A 1k texture for distance would add bytes, not save them, because the 2k is
  still needed close up — so it is not a size lever and is not proposed for size.
- **O-3. Ship only what a 3D world uses.** Lakeside's 3D cast enters the release only if it is the café
  street's extras (E-2) or when a 3D lakeside exists.
- **O-4. Raise the limit.** `3d.pck` hard limit to ~350 MB and the bundle goal accordingly; a demo download
  of ~450 MB. The operator's call (step-23 §7.1 is frozen).
- **O-5. Exclude CharMorph from the release** (≈ 11 MB) — S26-f does this regardless; the rig source
  stays in the repository.

**Ruled (Q-S26-5):** **O-1 + O-3 + O-5**. Basis Universal UASTC for cast textures, subject to the
operator's before/after portraits at the reference framing (delivered with the big milestone's checklist,
§6.4); if they are rejected, **O-4: the `3d.pck` hard limit rises to about 350 MB**. The triangle budget and
the 2048² texture stay the standard.

**Repository growth.** An accepted character commits ≈ 10 MB once (texture stored once, A-7 fixed); 21
characters ≈ 210 MB of history, against `.git`'s 654 MB today (A-27). Rejected runs never enter the
repository. **Ruled (Q-S26-10):** accepted characters only, the texture stored once, no Git LFS.

## 11. Performance

**Expectation, stated as such.** Today each townsperson is a ~106 k-triangle, ~15-material CharMorph
instance (D+ §5.2); a cast character is ≤ 100 k triangles and **one** material, so draws per person fall
and triangles stay level. Skinning is per instance either way. New costs: distinct textures (≈ 5.6 MB of
video memory per character present, against today's one shared set) and no sharing of mesh buffers between
people.

**Measurement plan** (RL-b's protocol unchanged: the operator's M5, 1920×1080, gi=voxel, three fresh runs,
median p95, `tools/perf_compare.py`'s verdict per R-2):

1. Baseline: RL-b's eight views on the head S26-c starts from (RL-b2 merged).
2. Same eight views with the cast in place (S26-c), plus a new **crowd** view: the connected café with all
   twelve social-cafe people disclosed, half of them in frame at 3–15 m, and the four street decor figures.
3. The crowd view with LODs forced off, to show the LODs' share (§10.3 O-2) — evidence, not a shipped mode.
4. Video memory with the café town cast and extras resident.

**Pass:** every view p95 ≤ 16.7 ms and video memory ≤ 2,048 MB (binding); draws and primitives reported
(advisory). If the crowd view fails, the levers in order: shadow casting off for people beyond a
distance, LOD bias, fewer simultaneous decor figures — each a visual change shown to the operator. Lower
triangle counts or textures for a person are **not** a lever here (§1 part 2).

## 12. PR split

```text
S26-a  appearance System Pack + cast binding contract ─────────────┐
S26-b1 pipeline import (port, parameterize, gates)  ─┐             │
S26-b2 automation spike + pilot B0 (3 people)       ─┴→ S26-cast ──┼→ S26-d(B1, B2) ─┐
S26-c1 human.gd data-driven characters (no slice_link) ────────────┤                 ├→ S26-f release
S26-c2 slice figures through the binding  (after 16c) ─────────────┘                 │
S26-e  2D sprites per person (after S26-a and each batch's casting gate) ────────────┘
```

| PR | Scope (files) | Depends on | Integration checkpoint | Adversarial criteria | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S26-a** binding contract | `systems/appearance/` (crate on the `naming` template: section, `Look` component, `appeared` event, public disclosure); `systems/installed`; `appearance:` lines and `systems:` entries in `market-town`, `social-cafe`, `lakeside`; golden fixtures that move; `presentation/mineworld-default/3D/characters/cast.yaml` format (spec + empty-but-valid file); `clients/2d/PRESENTATION.md` §3.1 `look:<key>` role; cognition check (§4.2); records `ARC-91/92` | none (no RL-b2, R-a or 16c dependency) | `mineworld server worlds/social-cafe` and a scripted client observation shows `look` on every named person; `bodies-yard` shows none; a save/restart keeps it | X-1 an invalid slug is refused at the section's line; X-2 a world with the section but without the pack in `systems:` is refused by name (the `naming` precedent); X-3 a world without the pack loads with unchanged digests; X-4 `look` is not rendered as a name by the perceived formatter | **Haiku**, from a commit-level plan mirroring `naming` |
| **S26-b1** pipeline import | `tools/character_pipeline/` (§5.1): the eight scripts ported, constants into `fit.yaml`, landmarks still hand-supplied; stages 3, 6–12; static gates G-1…G-6, G-14 as a CI check on committed characters; secret handling (§5.5); `PIPELINE.md` | none | **the pipeline re-derives the protagonist** from her kept download (`out/meshy/apose_v3`) and her `fit.yaml`: G-1…G-14 pass, joints within 1 mm of `meshy_d.glb`'s, weights equal after rounding | X-5 a 100,001-triangle mesh fails G-1; X-6 a second material fails G-2; X-7 a renamed bone fails G-5; X-8 a key-shaped string in a sidecar fails the scan; X-9 a bound computed from the candidate is rejected in review (`ARC-23` rule 2) | **Haiku** for the port; Opus reviews |
| **S26-b2** automation spike + pilot B0 | landmark detection (§5.4 L), weight transfer vs heat weights (W), Meshy rig as a joint source (R); runtime runners G-7…G-13 automated; the pilot: three briefs, references through a casting gate, Meshy runs, gates, one runtime review | S26-b1; S26-c1 for in-client gates; operator casting gate | three pilot characters through every gate with ≤ 3 `fit.yaml` overrides each; **measured** retention rate, credits per accepted character, bytes per character in an exported pack — the numbers §10 replaces its estimates with | X-10 a landmark moved 5 cm fails G-6 by location; X-11 a sleeve forced into the torso fails G-9 and prints where; X-12 the head-steady disabled fails G-10 | **Opus** (research and vision judgement) |
| **S26-cast** casting | every brief of §7 for the chosen worlds; the distinctness table; reference images per batch; casting-gate pages | S26-a (look keys); ruled Q-S26-7 (widen, gated at casting) | operator casting verdicts recorded | X-13 two briefs that differ in two categories only are flagged by the table | **Opus** (briefs), images by tooling |
| **S26-c1** 3D characters as data | `human.gd`, `posture.gd`, `npc.gd`: `character.yaml` loading, `Human.Body` removed, Vera's constants moved verbatim; `town.gd` and `streetscape.gd` draw from `extras`; `--perf` crowd view | RL-b2 merged (perf baseline, shared files) | **the protagonist is unchanged**: `--portrait` frames pixel-equal to `main`'s within the frame-diff tolerance, `--headtrace` within 0.5° of `main`'s; promenade and street figures are pilot characters | X-14 a manifest missing `stature_m` fails at load naming the field; X-15 `vitruvian.glb` is loaded on no person path (scan, only `--town-body`) | **Haiku** with a commit plan; Opus review |
| **S26-c2** connected figures | `slice_link.gd` figure creation resolves `look` → `cast.yaml` → character, fallback §4.4; the player's body follows the observer's look | **16c merged** (it rewrites this path); S26-a; S26-c1 | `./mineworld-slice --world`: Alice, Bob and every disclosed person is their own character; an unknown person is an extra | X-16 a person whose `look` the pack does not bind is drawn as an extra, never CharMorph; X-17 two connected players see each other with their own looks | **Haiku** after 16c's shape is known; else Opus |
| **S26-d** batches | B1 café town (the 8 not in the pilot), B2 lakeside (10): generation, gates, runtime review pages, accepted assets, `HUMAN_REVIEW_QUEUE.md` `VIS-3D-CAST-B1/B2` | S26-b2, S26-cast, S26-c1 | runtime verdicts (parked for the big milestone, §6.4); meanwhile every committed candidate passes G-1…G-14 in CI | X-18 a character marked accepted without an operator verdict fails the review-queue check | **Opus** for gates and scorecards; mechanical stages may be delegated |
| **S26-e** 2D sprites | `mineworld_gen` recipe `cast_2d.yaml`; sprites, sidecars, normalization; `look:<key>` bindings; the 2D client's role lookup | S26-a; each batch's casting gate | `./mineworld-2d` on `social-cafe` and `lakeside`: every named person their own sprite, Vera as the visitor | X-19 a 13th person in a world draws a `person:<n>` sprite, not a crash; X-20 a sprite whose normalized height disagrees with its brief fails AC-W12's height check | **Haiku** |
| **S26-f** release packing | R-a's export filters for cast characters per the size ruling (Q-S26-5); texture import policy (Q-S26-5); CharMorph out of the release; final perf session (§11) | **R-a merged**, RL-b2 merged, S26-c2, S26-d | exported `3d.pck` within the ruled limit; perf PASS on the operator's machine | X-21 a planted extra character over budget turns the size gate red | **Haiku** (size ruled, Q-S26-5) |

**Integration checkpoint of the step** (AC-S26 below): `./mineworld-slice --world` with every social-cafe
person their own accepted character, the decor and an unknown person drawn from extras, both clients
showing the same people, the exported release inside the ruled budget and the perf views inside 16.7 ms.

### 12.1 Start conditions and implementers (frozen with the step)

Every PR is detailed to the commit and frozen by the primary session before it starts, and starts in a
fresh session in its own worktree. "Merged" means merged on `main`. No PR spends a Meshy credit or makes an
OpenAI call outside the caps of Q-S26-3, and every call follows §5.5.

| PR | MVP | Starts when | Implementer |
| --- | --- | --- | --- |
| S26-a | MVP-0 | its PR design is frozen; nothing else (no RL-b2, R-a or 16c dependency) | Haiku, from a commit-level plan mirroring `naming`; Opus review |
| S26-b1 | MVP-0 | its PR design is frozen; nothing else. Its PR states the OpenAI dollar cap for the primary to confirm (Q-S26-3) | Haiku for the port; Opus review |
| S26-b2 + pilot B0 | MVP-0 | S26-b1 merged; S26-c1 merged (in-client gates); the OpenAI cap confirmed; the pilot's three casting pages ruled by the operator before any Meshy run (pre-spend decision, asked when ready) | Opus |
| S26-cast | MVP-0 | S26-a merged (look keys). Café town (B1) first; lakeside's briefs for E-2's extras next | Opus (briefs); images by the tooling |
| S26-c1 | MVP-0 | RL-b2 merged (perf baseline; shared files) | Haiku, from a commit plan; Opus review |
| S26-c2 | MVP-0 | 16c merged; S26-a and S26-c1 merged | Haiku once 16c's figure path is known; Opus if 16c leaves it unsettled |
| S26-d B1 (café town, 8 after the pilot) | MVP-0 | S26-b2 merged with the pilot's re-estimate of B1/B2 credits inside the 3,000 cap; S26-c1 merged; B1's casting pages ruled | Opus for gates and scorecards; mechanical stages may be delegated to Haiku |
| S26-d B2 (lakeside, 10) | MVP-0 **as café-street extras only** (E-2); their own world's 3D use is MVP-1 (Q-S26-1) | B1's generation finished; B2's casting pages ruled; the re-estimate still inside the cap | Opus, as B1 |
| S26-e | MVP-0 | S26-a merged; per batch, that batch's casting gate ruled (the 2D sprite is conditioned on the approved reference) | Haiku |
| S26-f | MVP-0 | R-a merged; RL-b2 merged; S26-c2 and S26-d B1 merged; the UASTC before/after portraits ruled (else the ~350 MB limit applies, Q-S26-5) | Haiku |

Runtime review pages (§6.4 gate 2) never hold a PR's start: they are parked for the next big milestone's
single checklist, and the characters they cover stay candidates until then (`ARC-9`).

### 12.2 Step acceptance (adversarial)

| | Criterion | How it is shown to fail when broken |
| --- | --- | --- |
| AC-S26-1 | Every person path in the 3D client resolves to a cast or extras character | scan for `vitruvian.glb` loads outside `--town-body`; X-15, X-16 |
| AC-S26-2 | Every shipped character passes G-1…G-14 | CI check over `characters/cast/*`; X-5…X-8 |
| AC-S26-3 | The pipeline's deterministic stages reproduce their outputs | re-run on a committed character's kept inputs; joint drift > 1 mm fails |
| AC-S26-4 | Each world's cast satisfies §7.3 | the distinctness table; X-13 |
| AC-S26-5 | Every accepted character has an operator verdict | review-queue check; X-18 |
| AC-S26-6 | The protagonist is unchanged by the refactor | frame diff and head trace against `main` (S26-c1) |
| AC-S26-7 | Same person in both clients | per-person sheet: 2D sprite beside 3D portrait, reviewed at the runtime gate |
| AC-S26-8 | Release size and perf within the ruled budgets | R-a's size gate; X-21; §11 |
| AC-S26-9 | No key in the repository; no runtime path names the secrets file | CI scan; X-8 |
| AC-S26-10 | A world without `appearance` behaves exactly as before | X-3: digests unchanged |

## 13. Risks

| | Risk | Mitigation |
| --- | --- | --- |
| R-S26-1 | Retention worse than 1 in 3 for some people (older faces, unusual garments) — credits burn | the pilot measures it; per-person retry budget; the reference is fixed before more runs (§6.5); hard cap |
| R-S26-2 | Automation does not replace the hand fixes; each character needs D+'s days of iteration | S26-b2 is a measured spike; `fit.yaml` overrides counted and reported; if > 3 per character persist, the operator is told the real per-character cost before B1 |
| R-S26-3 | Release size (§10.3) | ruled (Q-S26-5): UASTC subject to stills, else ~350 MB limit; pilot measures bytes |
| R-S26-4 | 16c reshapes `SliceLink`'s figures | S26-c split into c1 (independent) and c2 (after 16c) |
| R-S26-5 | Golden digests move when the `appearance` section adds genesis facts | S26-a updates them in one reviewed commit; X-3 proves worlds without the pack are unchanged |
| R-S26-6 | Two generated reference images of one person disagree (full body vs portrait) | the portrait is conditioned on the approved full body; the casting gate sees both |
| R-S26-7 | Casting from surnames stereotypes people | briefs are chosen for each person and reviewed by the operator at casting (ruled, Q-S26-7) |
| R-S26-8 | Seated poses tear fused garments | G-13; briefs avoid long hems for people who sit (§7.4) |
| R-S26-9 | Look keys leak names if names are later gated by acquaintance | recorded; a one-file-per-world migration to opaque keys |
| R-S26-10 | Video memory with many distinct textures | §11 step 4; only resident worlds' characters load |
| R-S26-11 | Meshy changes its model, terms or prices mid-step | `ai_model` pinned per batch and recorded; terms re-read at each batch's start (D+ §4 practice); a model change is a new pilot, not a silent swap |
| R-S26-12 | Repository growth (§10.3) | accepted-only commits, texture once (ruled, Q-S26-10) |

## 14. Findings

- **F-S26-1** The brief said the slice's NPCs are "tinted by `hash(id)` at a fixed 1.72 m": true for the
  connected slice; the promenade (seed 4242, 1.62–1.83 m random) and the street (seed 40711, fixed heights)
  use other rules (A-4).
- **F-S26-2** `SliceLink._figure`'s comment says appearance comes from tags; the code ignores tags (A-5).
- **F-S26-3** The 3D Presentation Pack is not runtime-loadable: the importer applies the bone map at
  import time (A-23). The 3D binding is data in the pack; the meshes remain client resources. Making 3D
  packs loadable from any directory (`ARC-46` point 4's 3D equivalent) is its own later design (runtime
  glTF load plus retarget), not S26.
- **F-S26-4** The protagonist's texture is imported lossless, not BC7, so step-23 §7.2's texture rule does
  not cover characters today (A-8). S26-f states the character import policy explicitly.
- **F-S26-5** `meshy_d`'s base colour is committed twice (A-7, ≈ 6 MB); the cast stores it once.
- **F-S26-6** No System owns stature; `height_mm` lived only in the spike server (`HUMANOID_PROFILE.md` §2).
  Stature is presentation in S26 (§4.2).
- **F-S26-7** The 2D player and the 3D protagonist are different people (A-19).
- **F-S26-8** 9 sprites serve 12 people in 2D (A-18): the 2D cast already fails "each person distinct".
- **F-S26-9** `ARC-22`'s premise — one albedo — does not hold for per-person generated characters; its own
  text says widening then "is a palette change and needs no new decision", and also that widening is the
  operator's. Ruled 2026-10-14: widen it (Q-S26-7, ARC-95).

## 15. Records and edits (ids assigned 2026-10-14; all ten used)

| Placeholder | Content | Written by |
| --- | --- | --- |
| **ARC-91** | Appearance is a perceivable trait owned by the `appearance` System Pack; `look` is an opaque disclosed key; no renderer concept, no descriptors in the world (§4.2) | S26-a |
| **ARC-92** | The cast binding lives in the Presentation Pack: 3D `characters/cast.yaml`, 2D `look:<key>` roles (amends `ARC-46`'s role list); one resolution order; the fallback is the extras pool, never a cruder class (§§4.3–4.4) | S26-a |
| **ARC-93** | A character is data (`character.yaml`), authored at true stature, instance scale 1; `Human.Body` removed (§4.5) | S26-c1 |
| **ARC-94** | The per-character standard (G-1…G-14, §6.2 scorecard, quality-tier parity), the two-gate review, the failed-generation policy (§6) | S26-b1 |
| **ARC-95** | `ARC-22` amended: the default cast's range follows per-person briefs once each person has their own generated texture, set from each biography and gated by the operator at casting (ruled, Q-S26-7) | S26-cast |
| **DEP-51** | Meshy image-to-3D as the cast's mesh source at scale; the paid-plan ownership record extended; model pinned per batch | S26-b1 |
| **DEP-52** | Blender, headless, as the pipeline's mesh runtime (a tool, never distributed; its outputs are ours) | S26-b1 |
| **DEP-53** | Skin weights: robust weight transfer adopted or declined against heat weights, with the measured result | S26-b2 |
| **DEP-54** | Landmarks: MediaPipe adopted or declined against geometric fitting, with the measured result | S26-b2 |
| **DEP-55** | Rigging: our fitted armature kept; Meshy rigging, Mixamo, AccuRIG, UniRig/RigNet declined as the rig, with reasons (§5.4) | S26-b1 |

Document edits: `HUMANOID_PROFILE.md` §5 ("What a second character costs") and §6 rewritten for the cast
pipeline; `VISUAL_FIDELITY.md` gains the cast's per-person reference rule (§6.2) as a pointer to
`ARC-94`; `MODULE_SPEC.md` §6.1 names `characters/cast.yaml`; `clients/3d-spike/ASSETS.md` one row per
character; `HUMAN_REVIEW_QUEUE.md` `VIS-3D-CAST-*`; `overall.md` gains S26.

## 16. Questions for the operator

As asked, each with its recommendation. **All twelve were accepted as recommended on 2026-10-14**; the
rulings, with the operator's qualifications, are in §16.1 and govern where this list and they differ.

1. **Q-S26-1 — MVP-0 or MVP-1?** *Recommendation:* **MVP-0** for S26-a, S26-b1/b2 with the pilot, S26-c1,
   the café-town cast (B1) and its 2D sprites — the crude crowd is in every 3D frame of the release, and the
   first public tag already waits on the 3D visuals settling. Lakeside's 3D cast in MVP-0 only if it is the
   café street's extras (Q-S26-8, E-2); otherwise MVP-1, when a 3D lakeside exists.
2. **Q-S26-2 — Which worlds first?** *Recommendation:* the café town (`social-cafe` = `market-town`'s
   twelve; Vera exists, 11 new), piloting three of them chosen to stress the pipeline — an older person
   (Otto or Ivan), a person in an apron (Alice), a person with a bag (Grace or Wes); then lakeside's ten.
3. **Q-S26-3 — A credit budget cap?** *Recommendation:* **3,000 Meshy credits** for the whole step
   (estimate ≈ 1,920 with E-2), reported at 50 % and 100 %, hard stop at the cap; the pilot's measured rate
   re-estimates B1 and B2 before they start. OpenAI: a dollar cap you set once S26-b1 has read the current
   per-image price.
4. **Q-S26-4 — The `bodies-yard` test fixture?** *Recommendation:* no `appearance` section and no
   characters of its own; its twelve unnamed walkers exercise the fallback path (extras by hash) with real
   cast characters.
5. **Q-S26-5 — LOD and texture policy for the release size?** The full standard for 21 characters is
   estimated at ≈ 330–345 MB of `3d.pck` against the 250 MB hard limit (§10.3). *Recommendation:* keep
   ≤ 100 k triangles and 2048² as the standard; Godot's import LODs for distance (the same character);
   **Basis Universal UASTC for cast textures, if you accept its before/after portraits at the reference
   framing**; ship only characters a 3D world uses; remove CharMorph from the release. If you reject
   UASTC's stills: raise the `3d.pck` limit to about 350 MB. A 1k distance texture is not proposed — it adds
   bytes rather than saving them.
6. **Q-S26-6 — The review format?** *Recommendation:* two gates per batch — a **casting page** (briefs and
   2–4 reference candidates per person, plus the distinctness lineup) before any Meshy credit is spent, and a
   **runtime page** (an orientation index, then one full-resolution side-by-side sheet per person with Vera
   as the control, scorecard and known misses, and accept / redo / reject per person). Delivered as one
   private HTML review page per gate that records your per-person verdicts, mirrored into
   `HUMAN_REVIEW_QUEUE.md` — or, if you prefer, Markdown rows in the queue only.
7. **Q-S26-7 — The cast's range of appearance (`ARC-22`).** With a generated texture per person, the
   one-albedo limit no longer applies. *Recommendation:* widen it: choose each person's skin, hair and
   features in their brief from their biography, with every brief reviewed by you at the casting gate — never
   inferred mechanically from a surname.
8. **Q-S26-8 — Decorative figures and the fallback pool?** *Recommendation:* **E-2** — the café street's
   extras are the lakeside cast (and the reverse later), no dedicated extras, no person of the present world
   ever duplicated; E-1 (about 8 dedicated extras, ≈ +720 credits and ≈ +60 MB) only when a world has no
   other world's cast to borrow.
9. **Q-S26-9 — The player's look.** *Recommendation:* appearance follows the Person (`INV-1`): Vera for the
   `visitor` seat, Wes for `wanderer`, in both clients, so two players see each other consistently. Vera
   gets a new 2D sprite matching her 3D identity; today's 2D `gen_player` (a man in an amber jacket) becomes
   Wes's 2D look and the identity source for his 3D brief, if you like it, or is retired.
10. **Q-S26-10 — Repository growth.** *Recommendation:* commit accepted characters only (≈ 10 MB each,
    ≈ 210 MB for 21), the texture stored once, rejected runs kept outside the repository; no Git LFS.
11. **Q-S26-11 — Where a person's look key lives.** *Recommendation:* a small `appearance` System Pack and
    one `appearance: { look: … }` line per person file (§4.2), rather than binding by display name, which
    would change a face when a name changes or is hidden.
12. **Q-S26-12 — Blink and expression.** The cast inherits route D+'s limits: painted eyes, no blink, no
    mouth movement while speaking. *Recommendation:* accept for S26 — it is the protagonist's standard —
    and plan faces as a later step for the whole cast at once.

### 16.1 Rulings (operator, 2026-10-14; binding)

Relayed by the coordinator on 2026-10-14: the operator accepted every recommendation of §16, with the
qualifications below. Recorded as relayed.

| Question | Ruling |
| --- | --- |
| Q-S26-1 | MVP-0 for S26-a, S26-b1/b2 with the 3-person pilot, S26-c1, the café-town cast (B1) and its 2D sprites. Lakeside's 3D cast is MVP-1, except as café-street extras under E-2 |
| Q-S26-2 | Café town first; the pilot of three chosen to stress the pipeline as recommended |
| Q-S26-3 | A hard cap of 3,000 Meshy credits for the whole step, reported at 50 % and 100 %. The pilot re-estimates B1 and B2. OpenAI: S26-b1 reads the current per-image price and states a dollar cap in its PR for the primary to confirm |
| Q-S26-4 | As recommended: no `appearance` section and no characters for `bodies-yard`; it exercises the fallback |
| Q-S26-5 | Basis UASTC for cast textures, subject to the operator's before/after portraits. If rejected, raise the `3d.pck` limit to about 350 MB |
| Q-S26-6 | Two review pages per batch. By a new operator rule, all hands-on and visual checks are batched into one big milestone: the casting page is a pre-spend decision and may be asked when ready; the runtime review page is parked and delivered with the next big milestone's single checklist |
| Q-S26-7 | Widen the range, set from each person's biography, with the operator gating it at casting (`ARC-95`) |
| Q-S26-8 | E-2: the café street's extras are the lakeside cast |
| Q-S26-9 | Appearance follows the Person: Vera is the visitor, Wes the wanderer |
| Q-S26-10 | Accepted characters only, no LFS |
| Q-S26-11 | The `appearance` System Pack, with one look line per person file (`ARC-91`) |
| Q-S26-12 | Accept route D+'s face limits |

**Ids assigned** (primary session, 2026-10-14), in order: `ARC-91` appearance System Pack and `look`;
`ARC-92` cast binding in the Presentation Pack and the extras fallback; `ARC-93` the per-character
manifest; `ARC-94` the per-character standard and review protocol; `ARC-95` `ARC-22` amended for
per-person generated casts; `DEP-51` Meshy image-to-3D at cast scale; `DEP-52` headless Blender;
`DEP-53` skin weights; `DEP-54` landmarks; `DEP-55` rigging. All ten are used; each is written into
`docs/DECISIONS.md` by the PR §15 names. `DEP-53` and `DEP-54` record S26-b2's measured choice.

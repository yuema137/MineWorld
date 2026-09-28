# MineWorld humanoid profile

**Date** 2026-09-27 · **Status** describes a working implementation, not a plan
· **Implemented by** `clients/3d-spike/` — `scripts/human.gd`, `tools/character_bake.py`,
`tools/make_bonemaps.gd`, `tools/measure_stride.gd`

Every number here was measured from the running thing, by a tool in this repository
that you can re-run. Where something is asserted without a measurement behind it, it
says so.

---

## 1. The standard

**Godot's `SkeletonProfileHumanoid`.** Not because Godot is the engine, but because it
is the convention the candidate sources already converge on: it is the same bone set as
VRM 1.0 and Unity Mecanim, and both rigs MineWorld actually uses map onto it 1:1.

Read from the engine (`tools/make_bonemaps.gd` prints it): **56 bones**,
`root_bone = "Root"`, `scale_base_bone = "Hips"`, **18 flagged required**.

Two things the engine says that prose usually gets wrong, both confirmed by dumping the
profile rather than by reading about it:

- **`Neck` is `required = false`.**
- **`Chest` and `UpperChest` are optional, yet `UpperChest` is the declared parent of
  `Neck` and of both shoulders.** A rig without it needs its shoulders reparented at map
  time.

**MineWorld therefore requires, above Godot's 18:** `Chest`, `UpperChest`, `Neck`,
`LeftToes`, `RightToes`. Both rigs in the tree have all five; requiring them keeps every
consumer simpler, and toes are what stop feet reading as planks.

`Root` is the documented exception: the profile declares it, the CharMorph rig roots at
`Hips`, and nothing needs it. `LeftEye`, `RightEye` and `Jaw` are unmapped on both rigs.

### The two committed BoneMaps

| | file | mapped | unmapped | skeleton bones unused |
| --- | --- | --- | --- | --- |
| character | `assets/characters/vitruvian/vitruvian_bonemap.tres` | 52/56 | `Root`, `LeftEye`, `RightEye`, `Jaw` | **0 of 52** |
| animation | `assets/characters/quaternius_ual_bonemap.tres` | 53/56 | `LeftEye`, `RightEye`, `Jaw` | **0 of 53** |

Both are **generated and validated** by `tools/make_bonemaps.gd` from one table, which
checks that every mapped bone exists in the skeleton it claims to map and that no bone
is mapped twice. A typo fails at generation instead of appearing later as one limb that
does not animate.

Zero unused bones on both sides is the evidence for the convergence claim: the CharMorph
"mixamo" preset and Quaternius's Rigify-`DEF` rig are the same skeleton under two naming
schemes.

**The thumb ambiguity, decided.** The profile has
`ThumbMetacarpal → ThumbProximal → ThumbDistal`; both source rigs give three thumb joints
starting one level further out. MineWorld maps `thumb1/2/3` →
`Metacarpal/Proximal/Distal`. The other reading is equally defensible; what matters is
that both maps make the same choice, and they do.

---

## 2. Conventions, measured

| Property | Value | How it was established |
| --- | --- | --- |
| Units | metres, 1 glTF unit = 1 m | glTF 2.0; the baked character measures 1.7687 m |
| Up axis | **+Y** | glTF 2.0 / Godot |
| Model forward, node space | **−Z** | Godot `Node3D` convention |
| **Reference-pose forward** | **+Z** | the profile's own convention, and it is what the character does — `--drive` prints `mesh face . player forward = +1.0000` |
| Bone axis | **+Y from parent to child** | enforced by the importer's *Overwrite Axis* |
| Node transform on the skeleton | none | enforced by *Apply Node Transform* |
| Rest pose | T-pose | see §3 |
| **Canonical authored height** | **1.7687 m** | printed by `tools/character_bake.py` from the baked GLB's own bounding box (y −0.0101 … 1.7586) |
| Skeleton `motion_scale` | 0.9956 | Godot sets it from hip height under *Normalize Position Tracks* |

### Height rule

Author once, scale the instance:

```
instance.scale = height_m / 1.7687
```

`spike/server/src/world.rs` already declares `height_mm` per person (1800 / 1680 / 1750),
and `npc.gd` passes it straight through, so those numbers are now visible in the frame.
Anything derived from speed must divide by that scale — a shorter body covers less ground
per stride, and §4 depends on it.

---

## 3. Import settings — the whole retarget contract

Written into the `.import` files by `tools/patch_imports.py` so they are reviewable as a
diff. **All six must match across the character and the animation library.**

| Option | Character | Clips | Why |
| --- | --- | --- | --- |
| `bone_map` | vitruvian | quaternius | the committed profile mapping |
| `rename_bones` | on | on | rewrites both rigs to profile names; this is what makes one clip set portable |
| `apply_node_transforms` | on | on | Blender bakes a −90° X quaternion on its glTF roots. Quaternius's rig carries it, ours does not. **Set it on one and not the other and the character animates lying on its back with nothing in the log.** |
| `overwrite_axis` | on | on | rewrites each bone's rest onto the profile axis |
| `fix_silhouette` | **on** | off | the character is A-posed, the clips are authored on a T-pose |
| `normalize_position_tracks` | on | on | stores hip height as `motion_scale` |

`fix_silhouette` filters `LeftFoot`, `RightFoot`, `LeftToes`, `RightToes`: silhouette
fixing rotates a foot to point at its toe, which tips the whole foot forward and leaves
the character on tiptoe.

### The check that clears all of this at once

Play Quaternius's **`A_TPose`** on the character. If the result is a clean T-pose — arms
level, legs straight, feet flat — then the BoneMap, the axis overwrite, the silhouette
fix and the node-transform trap are all correct simultaneously. It is one render instead
of four separate investigations, and it is why `A_TPose` is kept in the trimmed clip set.
`Human.debug_clip("A_TPose")` plays it.

---

## 4. Animation naming and the cadence rule

Clips keep their Quaternius names with Godot's `_Loop` suffix stripped at import:
**`Idle`, `Walk`, `Jog_Fwd`, `A_TPose`**. A character exposes one verb,
`Human.set_gait(speed_mps)`; nothing outside `human.gd` names a clip.

### The rule

**Animation phase advances with distance travelled, never with wall-clock time.**
Standing still advances no phase. Walking twice as fast takes steps twice as often, of
the same length. Driving an `AnimationTree` from a timer and tuning until it looks right
is what produces skating, and it drifts the moment anything changes.

### Measured clip speeds

`tools/measure_stride.gd` samples a foot relative to the hips across one cycle, takes the
peak-to-peak travel as one stride, and counts two strides per cycle.

| clip | cycle | stride | ground speed |
| --- | --- | --- | --- |
| `Walk` | 1.333 s | 0.705 m | **1.058 m/s** |
| `Jog_Fwd` | 0.933 s | 1.235 m | **2.647 m/s** |

**These must be measured on the character, not on the animation rig** — on Quaternius's
own skeleton the same clips give 1.021 and 2.503 m/s. Stride is leg *rotation* applied to
*our* limb lengths, which is also exactly why `Normalize Position Tracks` does not fix
foot sliding: it rescales position tracks, and stride is not in them.

### How the rule is implemented

- Each clip sits at its measured speed in the blend space, so between two points the
  blended stride interpolates as the blend position does.
- Below walking pace the blend is idle→walk, which scales stride amplitude and speed
  together, so the clip plays at rate 1.0.
- Above it the band is **pinned to a single clip** and the playback rate carries the
  speed. Cross-fading walk into jog blends two cadences and shortens the stride.
- `sync` is **on**. Without it a 1.333 s clip and a 0.933 s clip run on their own clocks
  and blending them averages two out-of-phase footfall patterns.

### How it is verified

`./mineworld-3d --drive`, current output:

```
standing still:   0.00 m travelled, 0.00 gait cycles
walk 4.0 s:       4.00 cycles over 5.19 m, 0.771 cycles/m, planted foot drifts 24% of body speed
walk 2.2 s:       1.50 cycles over 2.18 m, 0.687 cycles/m
jog  4.0 s:       5.50 cycles over 11.84 m, 0.465 cycles/m, planted foot drifts 82% of body speed
```

Cycles are counted from the foot's own swing, between interpolated first and last
crossings, so the acceleration ramp is not folded in. Drift is the smaller per-tick
movement of the two feet, which needs no stance detection.

**Reading the numbers honestly.** Walk and jog legitimately differ in cycles/m — a jog
has a longer stride. The jog drift figure is *not* comparable to the walk's, because a
run has a flight phase where neither foot is planted. The 24% walk residual is ankle roll
through stance plus sampling noise; it is not gross skating, but it is not zero either
and a foot-lock IK pass is where it would go next.

---

## 5. What a second character costs

The pipeline is `tools/character_bake.py` plus a `BoneMap`. For a new character that is
already a rigged glTF humanoid:

| step | cost |
| --- | --- |
| bake (strip animation, merge head/hair, decimate, retarget attributes) | minutes, scripted |
| BoneMap: add a column to the table in `make_bonemaps.gd`, regenerate, validate | ~30 min if the rig is a known family |
| `.import` settings | copy, change two paths |
| materials | ~1 h, and only if its texture set differs |
| animation | **free** — the clips already address profile bone names |

**The real cost is not the rig, it is the appearance.** The CC0 asset ships exactly one
body and face texture set, so a crowd built from it is one person in different clothes.
`human.gd` tints the albedo per instance to buy back some range, and that is a mitigation
rather than a fix. A genuinely different-looking person needs a second texture set, and
that is where generation belongs: `ARC-9` prefers generated *appearance* layered onto a
standard rig over generated meshes, and this profile is the standard rig that makes it
possible.

---

## 6. Limitations, stated

- **One face, one body texture.** See above. The largest gap.
- **The head is rigid.** Upstream ships it as a separate unskinned mesh; the bake
  rigid-binds it to the `Head` joint, so the neck does not deform with it. Invisible at
  conversation distance, visible if a character ever looks sharply sideways.
- **FACS blendshapes are stripped.** 26 of them exist in the source. `ARC-4` scopes
  facial fidelity down, and they cost ~8 MB. Re-bake without the strip if expressions are
  ever wanted.
- **Hair is decimated card geometry**, cropped by height to a short cut. It has no
  physics; upstream's spring-bone rig was dropped with its skin.
- **The hoodie is a shell**, offset from the t-shirt and the bare arms. No hood, no zip,
  no pockets, no slack.
- **Godot 4.7.2** is what this was built and measured on.

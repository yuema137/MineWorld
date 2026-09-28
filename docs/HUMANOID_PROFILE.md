# MineWorld humanoid profile

**Date** 2026-09-27 · **Status** describes a working implementation, not a plan
· **Implemented by** `clients/3d-spike/` — `scripts/human.gd`,
`tools/character_model.py`, `tools/garments.py`, `tools/hair.py`,
`tools/model_lib.py`, `tools/character_textures.py`, `tools/patch_imports.py`,
`tools/make_bonemaps.gd`, `tools/measure_stride.gd`, `tools/material_probe.gd`

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
| Units | metres, 1 glTF unit = 1 m | glTF 2.0; the baked character measures 1.7670 m |
| Up axis | **+Y** | glTF 2.0 / Godot |
| Model forward, node space | **−Z** | Godot `Node3D` convention |
| **Reference-pose forward** | **+Z** | the profile's own convention, and it is what the character does — `--drive` prints `mesh face . player forward = +1.0000` |
| Bone axis | **+Y from parent to child** | enforced by the importer's *Overwrite Axis* |
| Node transform on the skeleton | none | enforced by *Apply Node Transform* |
| Rest pose | T-pose | see §3 |
| **Canonical authored height** | **1.7670 m** | printed by `tools/character_model.py`: sole to crown, z −0.0201 … 1.7469. **Hair is excluded on purpose** — the gathered updo reaches 1.8026, and `height_mm` in the world means how tall the person is, not how tall her hair is. |
| Skeleton `motion_scale` | 1.0048 | Godot sets it from hip height under *Normalize Position Tracks*; it moved from 0.9956 when the body was re-baked through the Ultra Feminine morph |

### Height rule

Author once, scale the instance:

```
instance.scale = height_m / 1.7670
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
| `Walk` | 1.333 s | 0.708 m | **1.063 m/s** |
| `Jog_Fwd` | 0.933 s | 1.241 m | **2.660 m/s** |

**These must be measured on the character, not on the animation rig** — on Quaternius's
own skeleton the same clips give 1.021 and 2.503 m/s, and on the previous
occlusion-deleted body they gave 1.058 and 2.647. Stride is leg *rotation* applied to
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
walk 4.0 s:       4.00 cycles over 5.05 m, 0.793 cycles/m, planted foot drifts 21% of body speed
walk 2.2 s:       2.00 cycles over 2.77 m, 0.721 cycles/m
jog  4.0 s:       5.50 cycles over 11.18 m, 0.492 cycles/m, planted foot drifts 79% of body speed
```

Cycles are counted from the foot's own swing, between interpolated first and last
crossings, so the acceleration ramp is not folded in. Drift is the smaller per-tick
movement of the two feet, which needs no stance detection.

### Ground contact is a separate property, separately measured

Drift asks whether a planted foot slides. It does not ask whether anything is
planted: a body hovering with its legs cycling below it would pass. So
`--drive` also reports contact, calibrated against the character standing
still, where the sole is on the ground by construction:

```
lower foot vs its standing height: min -0.006 m, max +0.044 m
in contact (within 15 mm of the ground) on 55% of sampled frames
```

There is a support phase. 6 mm of sole clipping at the low point is the
remaining artefact. This measurement exists because a screenshot appeared to
show both feet off the ground, and the measurement showed the screenshot was
taken mid-deceleration rather than the gait being wrong.

**Reading the numbers honestly.** Walk and jog legitimately differ in cycles/m — a jog
has a longer stride. The jog drift figure is *not* comparable to the walk's, because a
run has a flight phase where neither foot is planted. The 21% walk residual is ankle roll
through stance plus sampling noise; it is not gross skating, but it is not zero either
and a foot-lock IK pass is where it would go next.

---

## 5. What a second character costs

The pipeline is `tools/character_model.py` plus a `BoneMap`. For a new character that is
already a rigged glTF humanoid:

| step | cost |
| --- | --- |
| model (cut the garments out of the body, build hair and shoes, trim, export) | minutes, scripted, needs Blender |
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

- **One face, one body texture.** See above. The largest gap for a *crowd*; the
  default character now carries generated freckles on top of the CC0 face map,
  which is per-character appearance rather than a second set.
- **The head is no longer rigid.** It was, when the head arrived as a separate
  unskinned mesh rigid-bound to the `Head` joint. The body is now exported whole
  from CharMorph, so head, neck and torso are one skinned surface and the neck
  deforms with it.
- **FACS blendshapes are stripped.** 26 of them exist in the source; the export
  does not carry them. That is what makes the character's expression fixed: she
  cannot smile, and `CHARACTER_IDENTITY.md` §3 names a slight closed-mouth smile
  as an identity feature. Re-exporting with `export_morph` on is the route.
- **Hair is modelled card geometry** (`tools/hair.py`): an opaque cap, a swept
  mass in clumps, a twist at the crown, a hairline layer and named loose
  strands. It has no physics.
- **The garments are cut out of the body surface** and offset, which is what
  makes them fit and inherit skin weights. Real tailoring — panels with seam
  allowance, a lining, cloth simulation — is not what this is.
- **The shoes are cut from the feet**, inflated and given a flattened sole. No
  laces, tongue or tread.
- **Texture import flags are owned by `tools/patch_imports.py`.** They have to
  be: every character map was imported with `mipmaps/generate=false`, and at
  portrait distance a 2048 face albedo without a mip chain aliases into a
  featureless smear while the hair opacity mask speckles. It looked like an art
  problem for one review round and was a checkbox.
- **Godot 4.7.2** is what this was built and measured on.

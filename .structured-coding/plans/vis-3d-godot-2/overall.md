# Overall — `VIS-3D-GODOT-2`, the Godot environment vertical slice

**Effort** `vis-3d-godot-2` · **Branch** `vis/3d-godot-2-environment` · **Base** `main` at `5b99840`
**Milestone** `VIS-3D-GODOT-2` ([`docs/HUMAN_REVIEW_QUEUE.md`](../../../docs/HUMAN_REVIEW_QUEUE.md),
[`DECISIONS.md`](../../../docs/DECISIONS.md) `ARC-20`)

---

## 1. The question this effort answers

> How close can a properly executed Godot Forward+ presentation get to the intended default
> visual quality?

`ARC-21` returned the 3D effort to Godot and `ARC-18` §2 keeps the engine question open. `ARC-20`
§2 requires that Godot be **pushed properly before it is judged**, because the current gap traces
to wrong assets, materials and lighting and not to a demonstrated rendering ceiling.

**No conclusion about Godot's visual ceiling may be drawn from this effort until the slice is
seriously implemented.** An early verdict from a half-built scene would answer the wrong question
and would make `VIS-3D-AB-1` undecidable if the Unreal track restarts.

## 2. Observable outcome

A launchable Godot client in which a person walks a short town street at golden hour, approaches a
café, crosses its threshold on foot, moves around a furnished interior, inspects its furnishings,
and walks back out — with real geometry, real collision, real materials, a real lighting pipeline,
verified scale, working cameras, recorded asset provenance, and a live connection to the
authoritative MineWorld world.

Scope is frozen by [`docs/VISUAL_SLICE.md`](../../../docs/VISUAL_SLICE.md), which is written
engine-neutral so that `VIS-3D-UE5-1` can be built to the same target (`ARC-20` §2).

## 3. Non-goals

```text
expanding the town beyond VISUAL_SLICE.md §4
a second enterable building · a square · a market · vehicles · weather · a day/night cycle
the reference character itself — that is VIS-3D-GODOT-1, on vis/3d-human-pipeline
any kernel, contract, System, World Pack, persistence or networking change (ARC-21 §1)
retiring or rewriting the existing clients/3d-spike scene
```

**The character branch is untouched.** `presentation/mineworld-default/3D/CHARACTER_IDENTITY.md`,
`clients/3d-spike/assets/characters/**`, `clients/3d-spike/scripts/human.gd`,
`clients/3d-spike/scripts/posture.gd` and `clients/3d-spike/tools/character_bake.py` belong to
`vis/3d-human-pipeline`. This effort consumes them through one adapter and edits none of them.

## 4. Approach

The existing `clients/3d-spike` Godot project is **reused, not forked**: it already holds the CC0
asset set, the material library, the geometry helpers, the movement controller, the three-camera
rig, the golden-hour lighting rig (`ARC-13`) and the `--drive` / `--shots` harness. Rebuilding
those would be the reinvention `REUSE_POLICY.md` forbids.

What is new is a **second scene** in that project — the slice — with its own construction code,
its own launcher and its own capture set. The existing promenade scene is left working and
untouched, because it is the artefact behind an accepted movement and camera decision and
deleting it would destroy evidence.

## 5. Steps

| | Step | Output | Checkpoint |
| --- | --- | --- | --- |
| **01** | Slice specification | `docs/VISUAL_SLICE.md`, engine-neutral | it contains no engine-specific technique |
| **02** | Scene skeleton and launcher | slice scene, ground, massing, launcher, capture set | launches from a clean checkout; the player walks the full street |
| **03** | Café exterior architecture | profiled shopfront, signage, masonry, openings with depth | measured against §3.2 and compared to `03_cafe_frontage.png` as pixels |
| **04** | Street: secondary façades, ground, furniture, planting | the rest of §4 | no repeated unit; the street reads from both ends |
| **05** | Material and lighting pipeline | material authority, GI decision with evidence, probes, sky, exposure | the GI choice is recorded with measurements, not asserted |
| **06** | Café interior | §6 in full, with collision and navigability | the walk-in loop passes; three object sizes present |
| **07** | Threshold verification | §7.1 measured at four sample points | numbers, not impressions |
| **08** | Runtime integration and character slot | protocol module adopted, `Place` identity, slot | an intent round-trips against the real server |
| **09** | Review package | §12 views, licences, limitations, queue entry | `ARC-20` §3 satisfied |

Steps 02–08 are one PR's worth of work executed as a sequence of semantic commits, each pushed on
completion, per the operator's standing instruction never to accumulate a large uncommitted
visual branch.

## 6. Risks

| Risk | Handling |
| --- | --- |
| **Concurrent character branch** touches the same Godot project | new files only; the one shared file touched is the root launcher list. Conflicts resolved in favour of the character branch |
| **Baked GI cannot be produced from script** | evaluated in step 05 with evidence; if a bake cannot be produced reproducibly from a clean checkout, the fallback is recorded as a decision with its measurements, not as a preference |
| **"Push Godot properly" is unfalsifiable** | every claim in steps 05 and 07 is a measurement against `VISUAL_SLICE.md` §7.1, and every fidelity claim is a pixel comparison at the reference's own framing (`ARC-17` §6) |
| **Scope creep into a town** | `VISUAL_SLICE.md` §4 lists what is out of scope by name |
| **An asset that may be redistributed but not relicensed** | `DEP-8` as amended 2026-09-27 is applied per asset, and the question asked is relicensing |

## 7. What this effort does not settle

The engine choice (`ARC-18` §2, `ARC-20` §5) and the default look (`ARC-11`). Both are the
operator's. This effort produces the evidence, names its limitations, and stops.

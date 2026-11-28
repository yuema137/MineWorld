# PROPOSED B: consolidated engine-verified drawing-anchor packet

For coordinator/operator decision only. Implementation remains immutable clean `4ce72437c8206826a5d5b5eba9fc3471024032c6`; the C5 packet at `9c41ba69` and earlier measurement report are unchanged. No format/API/world/production change has been implemented. This append closes engine-placement measurement, not S34-e acceptance.

## Proposed decision

Approve bounded presentation-only B: add an explicit place-local east/north drawing displacement for the two Café depiction files, using the measured central seat drawing anchors below. Preserve semantic world sit points, all existing scalar outputs, and S34-f ownership of body fit, armrests, actual contact, approach, and traversal. The proposed 10 mm per-axis/height tolerance is coordinate/height reproducibility only; it is not an anatomical or clearance threshold. A decision on this concrete scope and these anchors is still required before implementation.

A universal furnishing-origin oracle is incorrect. A scalar alone cannot reach the selected visual seat patches for table-2.2 and table-3.1 in either Café (four rows total, earlier independently reviewed gaps approximately 280/271 mm). Longitudinal-only A would leave that gap; C would defer completion. World-point/content corrections are excluded from B and would require separately scoped authoring review. Recommend B for this bounded presentation problem.

## Engine path and parity

`engine.gd` uses unchanged `Props.gltf`, `SliceFurnishings.load_pack/draw_parsed/build`, `SliceProps.put/aabb_of/true_box`, and `SliceSettle.stand/settle`. It builds the current social-cafe source's complete Café furnishing list in its actual order under the normal dedicated furnishings node, then measures the nine bound seat instances. Market-town's furnishing list (including positions/facings/order) is exactly equal, asserted in the helper. Their solids differ, but all these Café bindings are ordinary model bindings with no repeat/form/piece usage, so solids do not enter their placement or support path. This is one shared geometry case, not a second separately instantiated world.

The furnishing parent contains earlier furniture/dressing meshes as in production. The building floor is outside that dedicated node; settling's collection stays within this parent. Floor mounting names the place origin's floor Y. Measured at place origin zero, all nine seats have exactly 0 vertical translation beyond `stand`; one X rounding difference is 0.000238 mm. All three models declare floor mounting, root scale (1,1,1), root origin (0,0,0), and binding front_md=180000. Source node transforms, including the stool's approximately 90-degree Y rotation, agree with the imported engine geometry.

Godot 4.7.2 imported-cache measurement: 11 unique Café/dressing models actually loaded, zero runtime glTF fallbacks; three seat model types and nine seats measured. Twelve existing relevant cache sets were staged (one unused dressing variant) from the reviewed local base only after exact source-file equality checks. No importer, new content, broad scene/body campaign or rendering capture ran. Cache provenance records the actual imported resources. The cached resource UID table was not copied: Godot resolved texture UID warnings by their matching text paths. Final run exited 0 and wrote all measurements; one `Parameter material is null` teardown diagnostic after the measurement remains in the log. This is measurement evidence, not a claim of a warning-free renderer test.

Mesh AABB vs true vertices matters: imported mesh faces are quantized relative to source floats. Maximum transformed vertex-bound difference is below 0.049 mm; AABB/source differences are below 0.00004 mm. Engine AABB-center vs engine true-center horizontal components differ by at most 0.025 mm (all components below 0.027 mm). The engine's `stand` uses its true lowest vertex rather than AABB min. After this adjustment, selected anchor ray differences from offline height are at most 0.066 mm (more precise than the preliminary commentary's rounded 0.05 mm statement). None approaches the proposed 10 mm reproducibility tolerance. Exact model-specific numbers are in `engine-targets.json`.

For each central drawing anchor, the offline selected patch centroid X/Z is transformed by the actual placed engine node. A vertical ray through that X/Z against actual imported model mesh triangles gives its seat height. All nine rays hit within 0.024 mm of the transformed offline surface. The ray starts 15 mm above the selected seat surface, so this measures the seat rather than overhead arms/back. It is intentionally not a whole-body clearance ray. The proposed anchor is a drawing convention, not a human pelvis fit conclusion.

## Proposed authoring values

Both worlds use the following displacement in metres from the selected semantic sit-point position. Heights are measured upper seat-ray heights above the floor. Six decimal places bound authoring rounding below 0.001 mm. `engine-targets.json` retains full precision and world/point/prop linkage.

| Point | east (m) | north (m) | seat_height_m |
|---|---|---|---|
| window-stools.1 | +0.000012 | -0.477133 | 0.689300 |
| window-stools.2 | -0.000674 | -0.498166 | 0.689300 |
| window-stools.3 | -0.001352 | -0.516267 | 0.689300 |
| table-1.1 | +0.652133 | +0.013499 | 0.453183 |
| table-1.2 | -0.652133 | -0.013499 | 0.453183 |
| table-2.1 | +0.593954 | -0.009468 | 0.447000 |
| table-2.2 | -0.047954 | -0.498532 | 0.447000 |
| table-3.1 | +0.100832 | +0.486464 | 0.453183 |
| table-3.2 | -0.640832 | -0.006464 | 0.453183 |

The support evidence remains the selected original visual triangles, their mesh hashes, ray intersections and `candidate-surfaces.json` outer envelopes, with `seat-surfaces.png` for interpretation. The envelopes are not continuous-support proofs. Proposed acceptance requires an actual selected-triangle hit at the anchor, not only hull containment. No minimum body-sized support margin is claimed or invented. Curved surfaces do not require a fictitious single horizontal seat plane.

## Exact proposed contract

- New optional `seat_displacement_m: {east: number, north: number}` uses place-local horizontal metres, independent of facing. `pose_of` uses `selected_at_mm + 1000*(east,north)`. Facing continues to control drawing direction/yaw. The existing `pelvis(at,facing,offset)` API and `pose_of` output shape remain unchanged.
- Existing files remain format 1. Only the two new Café files use format 2. The loader accepts 1/2; vectors in format 1 are invalid. Unknown formats remain rejected. Format 2 may contain scalar rows with unchanged scalar behavior.
- Vector and scalar fields are mutually exclusive. A vector must be a dictionary with exactly east/north, each finite numeric and non-boolean. Reject null, strings, missing/extra axes, mixed fields and parser overflow/nonfinite numbers. Report file and point; reject the affected new-format file atomically. Every new Café row explicitly authors the measured height above. Do not broaden unrelated legacy scalar validation.
- Disclosed at/facing override fallback exactly as existing pose selection; absent disclosure uses fallback, and absence of both positions yields empty. New Café data need no invented fallback world positions. Matching remains scoped by world, place tags and point key. Reach, decide, chaining and interaction semantics do not change.
- If the vector is absent, execute the existing scalar arithmetic, defaults (offset/height 0.45), matching, fallback, and 3D undepicted-empty behavior exactly. No migration of any Lakeside row. Mirror cases negate east and transform facing to (360000-facing)%360000; north is unchanged.

## Proposed oracle and legacy preservation

Replace the universal-origin part of §5.7/AC-e-9 only after approval. For the 18 new Café rows, compare drawing coordinates in both axes and height to independently recorded mesh-derived anchors with 10 mm maximum per component; also require selected visual triangle support at the target. Freeze target data and source hashes independently of authored displacement values: the oracle must never compute expected target by reading the displacement under test.

For all 29 legacy Lakeside rows, freeze existing executable disclosed/fallback outputs before edits and require exact output parity. Their existing calibrated drawing anchors remain the reference; the previous source-mesh/primitive audit finds all 29 over seat surfaces with height differences at most 3.71 mm. The five bakery samples retain the intentional 60 mm tableward calibration; multi-seat benches retain separate point stations. No new centering or engine authoring correction is applied to them. This closure did not instantiate Lakeside; its existing source-geometry evidence and exact-output requirement remain explicitly separate from Café engine parity. Armrest clearance remains S34-f.

## Bounded deliverables if approved

1. `clients/shared/sitting/sit_rule.gd`: versioned loading/validation and vector branch of pose resolution only; scalar API and behavior preserved.
2. `clients/shared/README.md`, `clients/3d-spike/PRESENTATION.md`, this PR's owned ledger: the precise schema, drawing-anchor convention and approved oracle wording. Coordinator reconciles binding parent freeze/dependency text; implementer does not edit shared root plans.
3. `presentation/mineworld-default/3D/sit_points/social-cafe.yaml` and `market-town.yaml`: nine measured vector rows each, scoped by world/place tags/point, with existing prop keys and explicit heights.
4. Focused existing shared pose/loader checks and cross-client actual drawing comparator; `tools/cli/tests/sit_point_depictions.rs` plus independent `tools/cli/tests/fixtures/seat-surfaces.json` containing provenance and target/support measurements. No generated sprites, world/server/kernel edits, F-RC-7 changes or S34-f probe changes.

Bounded validation: freeze/compare 29 legacy disclosed and fallback results exactly; vector east/north and non-cardinal cases; disclosed precedence, absent position and malformed/mixed format cases; renamed/mirrored fixture cases; actual 2D/3D placement for new Café targets; independent all-row depiction support/coordinate checks. Drop/reverse each vector axis must turn its target/placement test red. Revise C4's currently undepicted Café expectation only with the approved new data. Run affected checks in existing shared slots, not a broad body/asset campaign; report any required budget increase to the coordinator before running it. Existing input/stand/cancel logic is unchanged and gets no unrelated expansion.

## Reproduction and remaining decision

```sh
python3 /tmp/mineworld-s34e-seat-measure/stage-engine.py
/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh /opt/homebrew/bin/godot --headless --path /tmp/mineworld-recover-s34e/clients/3d-spike --script /tmp/mineworld-s34e-seat-measure/engine.gd
python3 /tmp/mineworld-s34e-seat-measure/close-engine.py
```

Preserved `engine-world-inputs.json` is the exact Ruby YAML projection of the current world files; its source hashes are in the provenance. Helpers, engine log/results, input snapshot, imported-cache hashes and target numbers are recorded in `engine-closure-provenance.json`.

Measurement parity and support of the chosen central drawing anchors are established at the inspected inputs. The remaining decision is to approve this bounded schema/oracle amendment and adopt the measured central drawing convention, not to infer anatomy. Human identification of the selected seat bands remains interpretive and documented. Future asset/import/source changes must remeasure the independent evidence. No claim of full S34-e acceptance, shipping approval, or prerequisite merge follows from this packet.

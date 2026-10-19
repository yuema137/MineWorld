# PR S36-P1 — `mwcast/contact.py`: one contact module, tolerance data, static body, garment and item checks

**Lifecycle:** `DESIGN FROZEN — 2026-10-19` (operator approved in the primary session: freeze with Q-S36-1 … Q-S36-9 answered as recommended). No implementation before a `DESIGN FROZEN`
header and a filled contract (§9).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.1. **Records:** `DEP-142` (libigl),
`DEP-143` (trimesh + python-fcl), `DEP-145` (rejections). **Audit base:** `origin/main @ b843d546`.

---

## 1. Goal

Give every later clipping check one place to ask "how deep is A inside B?" and "which of these objects touch?", backed
by libigl and trimesh/python-fcl, with the bounds as data. Use it at once for the checks that need no animation:

| Gate | Question | Input |
| --- | --- | --- |
| **C-1** | At rest, does a limb go into the trunk? | a fused body GLB (bind pose) |
| **C-2** | At rest, does the body poke through a covering garment, or an inner layer through an outer one? | an assembled outfit GLB (`blender/mpfb/assemble.py`) and its manifest |
| **C-3** | Does a floor-standing item rest on the floor? | a prop glTF |
| **C-4** | Do two parts of one item interpenetrate? | a prop glTF with several nodes |

Example. `outdoor_table_chair_set_01.gltf` is one file with a table and chairs as separate nodes. C-3 asks whether
the lowest point of the set is at `y = 0` within 3 mm. C-4 loads every node with its transform into one
`CollisionManager` and asks which pairs touch, and how deep.

## 2. Audit

| Fact | Where |
| --- | --- |
| The pipeline is stdlib only. Its tests run in CI's `python` layer from the root uv environment (`uv run --locked pytest tools/character_pipeline/tests`) on ubuntu-24.04 and windows-2025, and on macOS in `test-macos (b)`; ruff and pyright (strict) run in `fast` | `scripts/ci_layer.py` `PIPELINE`, `PYTHON_STATIC`, `LAYERS["python"]`; `pyproject.toml` `[tool.pyright]`, `executionEnvironments` |
| The root has one dev dependency group, `dev = ["pyyaml>=6.0"]`; `uv.lock` holds no numpy | `pyproject.toml` `[dependency-groups]` |
| The operator's tooling venv pins `numpy==2.5.3` among others | `tools/character_pipeline/requirements.txt` |
| A gate returns `GateResult(gate, verdict, looked_at, reasons)`; a gate that cannot look fails with the reason; bounds come from `standard.py` | `mwcast/gates.py` (`GateId`, `_result`) |
| GLB reading: `read_glb`, `accessor`, `positions` (the one skinned primitive), `vertex_weights`, `skin_joint_names`; no index (triangle) reader for an arbitrary primitive | `mwcast/glb.py` |
| `primitive_positions(glb, mesh, primitive)` reads any primitive's positions | `mwcast/wardrobe/fixtures.py` |
| An assembled outfit: node `Body` and one skinned node `part_<id>` per part; the manifest names each part's class, layer and covered regions | `mwcast/wardrobe/wgates.py` module docstring, `parts()`; `pose_sheet.gd` `_layer`, `_covers` |
| G-9's vertex filters: dominant bone groups (`_group`), trunk share and limb share ≤ `MIX_W` 0.05, limb segment radii `LIMB_R`, proximal cut-offs | `godot/pose_sheet.gd` `_group`, `_checks` |
| Committed bodies on `main`: `clients/3d-spike/assets/characters/meshy_d/meshy_d.glb` (fused), `vitruvian/vitruvian.glb` (layered garments); the cast GLBs arrive with S26-c1 (Git LFS) | `git ls-files` |
| Props: 40 directories `clients/3d-spike/assets/models/<slug>/<slug>.gltf`, loaded by `props.gd`; some hang from the ceiling or a wall (`hanging_industrial_lamp`, `wall_clock`, `fancy_picture_frame_01`) | `clients/3d-spike/assets/models/`, `scripts/slice/props.gd` |
| No gate id `C-n` exists | `git grep` |

**Consequence of the hanging props.** C-3 cannot assume every prop stands on the floor. Guessing from the name would
be world-specific hard-coding. So each prop declares its mount in data (§4, commit 4).

## 3. Scope

**In:** root dev dependencies and `uv.lock`; `requirements.txt` pins; `mwcast/contact.py`; `mwcast/tolerances.py`;
`tools/character_pipeline/contact_tolerances.json`; `mwcast/glb.py` readers; gates C-1…C-4 in a new
`mwcast/contact_gates.py`; `cast.py contact` and `cast.py contact-items`; the props' mount file; tests; PIPELINE.md §18.

**Out (non-goals):** any animated check (P2); any change to `pose_sheet.gd`; any push-out (P3); the client (P4); place
files (P5). G-4, G-6, G-9, W-4 and G-13 keep working unchanged in this PR.

## 4. Commits

### Commit 1 — the dependencies and the contact module

**Goal.** One module wraps libigl and trimesh/python-fcl; nothing else imports them; GPL is impossible to import by
accident.

**Scope.** `pyproject.toml` (`[dependency-groups] dev` gains `numpy`, `libigl==2.6.3`, `trimesh==5.1.1`,
`python-fcl==0.7.0.11`, each with a comment naming its record), `uv.lock`,
`tools/character_pipeline/requirements.txt` (the same pins), `tools/character_pipeline/mwcast/contact.py` (new),
`tools/character_pipeline/tests/test_contact.py` (new), `tools/character_pipeline/tests/test_contact_licence.py` (new).

**Implementation.**

- [ ] Add the four packages; `uv lock`; confirm the resolved wheels exist for macOS arm64 and x86_64,
      manylinux x86_64 and win_amd64 at Python 3.12 (read from `uv.lock`), and record libigl's own declared
      dependencies (numpy; scipy if its metadata names it).
- [ ] `contact.py` types (frozen dataclasses over numpy arrays): `TriMesh(vertices: (n,3) float64, faces: (m,3)
      int64)`, validated on construction (finite, faces in range, non-empty); `Inside(indices, depths_mm)`;
      `Overlap(a: str, b: str, depth_mm: float)`; `RigidPart(name, mesh, transform: (4,4))`.
- [ ] `inside(outer: TriMesh, points) -> Inside`: `igl.fast_winding_number(V, F, points) > 0.5`; the depth of each
      inside point is `sqrt(igl.point_mesh_squared_distance(points, V, F))` in mm.
- [ ] `lowest(points) -> float` and `clearance(points, floor_y) -> float` (numpy only; the floor is a plane).
- [ ] `overlaps(parts: Sequence[RigidPart]) -> list[Overlap]`: `trimesh.collision.CollisionManager`, every part
      added with its transform, `in_collision_internal(return_names=True, return_data=True)`; depth per pair is the
      largest `ContactData.depth`, in mm.
- [ ] The libraries are imported inside `contact.py` only. A missing library raises `ContactUnavailable(name)`;
      a gate turns that into FAIL "cannot look: <library> is not installed".
- [ ] Typed boundary: the libraries have no stubs, so each call result is checked (`numpy.asarray`, shape, dtype)
      and cast once at the boundary; pyright strict passes without blanket ignores (one narrow
      `# pyright: ignore[reportMissingTypeStubs]` per import line at most).
- [ ] `test_contact_licence.py`: (a) an AST scan of every `.py` under the repository (excluding `target/` and venvs)
      finds no `import igl.copyleft`, no `from igl import copyleft`, no `from igl.copyleft …`, and no
      `importlib.import_module("igl.copyleft…")` string; (b) after importing `mwcast.contact` and calling each
      function once, no module named `igl.copyleft*` is in `sys.modules`; (c) no file other than `contact.py`
      imports `igl`, `trimesh` or `fcl` (I-S36-1).

**Validation.**

- Integration over synthetic meshes built in the test (no fixture files): a closed unit cube; an **open** hemisphere
  shell (the generalized winding number must still call its inside inside, where ray parity would not); two boxes
  overlapping by 4 mm and two 1 mm apart.
- Known answers: a point 8 mm inside the cube's face → inside, depth 8 ± 0.01 mm; a point 1 mm outside → not inside;
  a point under the open shell's dome → inside.
- Negative: a face index out of range, a NaN vertex and an empty mesh are refused with `ValueError`; a missing
  library gives `ContactUnavailable` (simulated by a monkeypatched import).
- Mutation: change the 0.5 threshold to 1.5 → the hemisphere test fails.

**Acceptance.** The four libraries are in `uv.lock` and `requirements.txt`; `uv run --locked pytest
tools/character_pipeline/tests/test_contact.py tools/character_pipeline/tests/test_contact_licence.py` passes on the
author's machine; a planted `from igl.copyleft import cgal` in a scratch test file makes the licence test fail (shown,
then removed).

**Failure cases.** A library without a wheel for a platform: stop and report (a material dependency question), do
not fall back silently.

**Commit boundary.** Dependencies, the module and its tests only. No gate yet.

### Commit 2 — the tolerance file

**Goal.** Every bound is data with its rationale (I-S36-2).

**Scope.** `tools/character_pipeline/contact_tolerances.json` (new), `mwcast/tolerances.py` (new),
`tests/test_tolerances.py` (new).

**Implementation.**

- [ ] The file: `{"format": 1, "regions": {"<key>": {"bound_mm": n, "rationale": "...", "source": "...",
      "status": "cited|inherited|inconclusive"}}}`, with the keys of the step's §3.3 table.
- [ ] `tolerances.py`: `Tolerances.load(path) -> Tolerances`, `bound(key) -> float`, `sha256`. Unknown keys,
      missing fields, a non-positive bound and an unknown status are refused by name.
- [ ] `Tolerances.tightened_by(pack_file)`: a pack file may only lower a bound; a higher value is refused naming the
      key (Q-S36-4).
- [ ] Read the sources the table cites (Winter 1992; ASTM D1777 figures for cloth; a footwear compression source)
      and set each `status` from what was actually read.

**Validation.** Load the shipped file; refuse a file with a missing `source`; refuse a pack file that loosens
`floor.sink.foot` to 8 mm; accept one that tightens it to 4 mm.

**Acceptance.** Every key used by C-1…C-4 and by P2's planned gates is present; each has a non-empty source.

**Commit boundary.** Data and loader only.

### Commit 3 — C-1 and C-2: bodies and garments at rest

**Goal.** The rest-pose clipping questions, answered by `contact.py`.

**Scope.** `mwcast/glb.py` (`triangles(glb, mesh, primitive) -> list[tuple[int,int,int]]`; move
`primitive_positions` here from `wardrobe/fixtures.py`, which then imports it), `mwcast/contact_gates.py` (new;
`GateId` gains `C1 … C4` in `gates.py`), `mwcast/cli.py` (`contact <asset> [--outfit <manifest>] [--tolerances
<file>]`), `tests/test_contact_gates.py` (new).

**Implementation.**

- [ ] **Trunk patch and limb queries (C-1).** The trunk patch is the faces whose three vertices are trunk-dominant
      (Spine, Chest, UpperChest; `mixamorig:` names in a GLB, mapped through `standard.py`'s joint list) with limb
      share ≤ 0.05. The queries are hand, forearm, distal upper-arm, distal thigh and shin vertices, filtered as G-9
      does (dominant bone group, trunk share ≤ 0.05, within the limb's segment radius). The vertex filters are
      ported from `pose_sheet.gd` `_group` / `_checks` into one Python function, so P2 uses the same filter on
      dumped frames.
- [ ] C-1 fails when an `inside` query point is deeper than `self.limb_in_trunk` (or `self.hand_in_trunk` for hand
      vertices); it prints the worst depth per group.
- [ ] **Layer order (C-2).** For an assembled outfit: for each part in layer order (from the manifest), for each
      region it covers, the body's vertices in that region (the body's region index, as the assembly writes it in
      `TEXCOORD_1`) must be inside the part's mesh, or hidden (R-11's hidden set); a body vertex outside the
      covering part by more than `garment.poke.body` fails. Likewise each inner part's vertices in a region covered
      by an outer part must be inside the outer part, by `garment.poke.layer`. Never "anything inside anything".
- [ ] `looked_at` lists every pair tested, its vertex count and worst margin.

**Validation.**

- Real: `meshy_d.glb` (fused) passes C-1 at rest, margins printed. `vitruvian.glb` (layered) is run through C-2 by
  its own layer order; the result is recorded whatever it is (today's G-9 flags it, PIPELINE.md §15).
- Generated variant (built in the test from boxes and an open trunk shell): a hand box sunk 8 mm → C-1 FAIL, 8 mm;
  sunk 4 mm → PASS. An outfit of three nested shells, the middle one pushed 3 mm through the outer one in one
  region → C-2 FAIL naming the two parts and the region; at 1 mm → PASS.
- Negative: an outfit manifest naming a part that is not in the GLB → FAIL "cannot look".

**Acceptance.** C-1 and C-2 report PASS/FAIL with margins on the two committed bodies and on the generated variants
exactly as listed.

**Commit boundary.** Body and garment gates, the GLB readers, and their tests.

### Commit 4 — C-3 and C-4: items

**Goal.** Every prop in the library is checked, and a new prop cannot skip the check.

**Scope.** `clients/3d-spike/assets/models/contact.json` (new: per slug, `{"mount": "floor|surface|wall|ceiling",
"resting": [["<node>", "<node>"], …]}`), `mwcast/contact_gates.py` (C-3, C-4), `mwcast/cli.py`
(`contact-items [--root] [--tolerances]`), `scripts/ci_layer.py` (one command in `LAYERS["python"]`:
`cast.py contact-items`), `tests/test_contact_items.py` (new).

**Implementation.**

- [ ] Load each `<slug>/<slug>.gltf` with trimesh as a scene (node transforms applied; geometry in metres).
- [ ] The set of slugs is the directory listing. A directory without a `contact.json` entry fails C-3 "no mount
      declared"; an entry without a directory fails too.
- [ ] C-3, `mount: floor` only: the scene's lowest vertex `y` within `item.rest.floor` of 0. Other mounts are listed
      as "not floor-standing" (not judged here; their placement is P5's).
- [ ] C-4: every pair of mesh nodes in one prop. A pair deeper than `item.overlap.parts` fails, unless declared in
      `resting`, which allows contact but still fails beyond the bound.

**Validation.**

- Real: all 40 props are classified and run; the numbers are recorded in the ledger. Any prop that fails is
  recorded as a finding with its depth (fixing an asset is not this PR's scope; the finding goes to the asset lane).
- Generated variant: a temporary model directory written by the test, with a box raised 6 mm (FAIL), raised 2 mm
  (PASS), and two boxes overlapping by 4 mm (FAIL) or declared resting and touching (PASS).
- Negative: an unknown mount word; a `resting` pair naming a missing node.

**Acceptance.** `cast.py contact-items` exits 0 on the library or names every failing prop; the generated variants
behave as listed; the command runs in CI's `python` layer.

**Commit boundary.** Item gates, the mount data, the CI line and tests.

### Commit 5 — the specification and the ledger

- [ ] `tools/character_pipeline/PIPELINE.md` §18 "Contact checks": the module boundary, the gates, the tolerance
      file, the mount file, the commands.
- [ ] This ledger: evidence per commit, findings.

## 5. Acceptance criteria (PR level)

1. `contact.py` is the only importer of `igl`, `trimesh`, `fcl`; the licence test fails on a planted copyleft import.
2. The tolerance file holds every bound C-1…C-4 use; no gate holds a literal bound.
3. C-1…C-4 pass or fail exactly on the generated variants of commits 3 and 4 (each planted depth fails; the depth
   minus 1 mm below the bound passes).
4. Inputs are enumerated: bodies by path argument, props by directory listing; a second prop library (the test's
   temporary directory) is checked by the same command with `--root`.
5. `fast` is green on the final head.

## 6. Risks

- pyright strict over untyped libraries: handled at the boundary (commit 1).
- trimesh's glTF loader and Godot's import may differ (axis, scale); C-3 compares with Godot's view on one prop
  (`round_wooden_table_02`, whose top is measured at 0.746 m in PIPELINE.md §15) and records the difference.
- CI time: measured on the first green run and recorded.

## 7. Questions

None beyond the step's Q-S36-4 and Q-S36-5.

## 8. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] | [x] | [x] | §8.1 |
| 2 | [x] | [x] | [x] | §8.2 |
| 3 | [x] | [x] | [x] | §8.3 |
| 4 | [ ] | [ ] | [ ] | — |
| 5 | [ ] | N/A (docs) | [ ] | — |

### 8.1 Commit 1 evidence (2026-10-19)

- Implementation: `pyproject.toml` dev group gains `numpy==2.5.3` (the tooling venv's pin), `libigl==2.6.3`,
  `trimesh==5.1.1`, `python-fcl==0.7.0.11`; `uv lock` added those plus `scipy 1.18.1` (libigl's declared
  dependency) and `cython 3.3.0` (python-fcl's). `requirements.txt` carries the same pins. `mwcast/contact.py`:
  `TriMesh`, `Inside`, `Outside`, `Overlap`, `RigidPart`, `inside`, `outside`, `lowest`, `clearance`, `overlaps`,
  `ContactUnavailable`. DEP-142, DEP-143 and DEP-145 flipped to accepted (the step's freeze).
- Wheels read from `uv.lock` at cp312: libigl abi3 for macOS x86_64 + arm64, manylinux x86_64 + aarch64,
  win_amd64; python-fcl cp312 for the same five; numpy and scipy likewise; trimesh is pure Python. No platform gap.
- Validation: `uv run --locked pytest tests/test_contact.py tests/test_contact_licence.py` → 12 passed (PASS).
  Mutation: `INSIDE_THRESHOLD = 1.5` → the cube and open-dome tests fail (2 failed); restored. Planted
  `from igl.copyleft import cgal` in a scratch test file → the licence scan fails naming the file; removed.
  ruff check/format and pyright strict over the new files: 0 errors, with no ignore comment at all.
- Bounded deviation (recorded): `Outside` added beside `Inside`, because C-2 asks how far a body vertex is
  *outside* its covering part. Reason: the design's C-2 rule needs the outside distance; impact: one more type.
- Bounded deviation: FCL reports a face-to-face touch as a contact of depth 0 and a 1 mm gap as no contact, so
  "resting" pairs show up as depth ≈ 0 (used by C-4).
- Review: the libraries are loaded only by `_load` (`importlib`), so a missing one is `ContactUnavailable`, and
  every result is re-wrapped with `numpy.asarray` and length-checked; numpy itself is imported at the top of
  `contact.py`, so `cli.py` imports the contact gates lazily (the stdlib-only `check-characters` keeps working
  without numpy).

### 8.2 Commit 2 evidence (2026-10-19)

- Implementation: `contact_tolerances.json` (format 1, the 11 keys of step §3.3), `mwcast/tolerances.py`
  (`Tolerances.load`, `bound`, `sha256`, `tightened_by`; stdlib only), `tests/test_tolerances.py`.
- Sources actually read (web search, 2026-10-19), and the status set from them:
  Winter 1992 — the citation (Physical Therapy 72(1):45–53) is confirmed, but the ~13 mm minimum toe clearance
  was found only second-hand (a Victoria University thesis attributing it to Winter), so `floor.float.planted`
  is `inconclusive` rather than `cited`; P2 reads the paper. ASTM D1777 — its scope was read; it is the test
  method and states no per-fabric thickness, so the garment keys stay `inherited` (W-4). No footwear compression
  source and no ANSUR II table was read: `floor.sink.foot` stays `inherited`, `scene.mesh_over_solid`
  `inconclusive`. `item.rest.floor` and `item.overlap.parts` are new bounds without an external source:
  `inconclusive` (still enforced).
- Bounded deviation: the pack-tightened result's `sha256` hashes the base digest plus the pack file, so a
  measurement under a pack is distinguishable from one under the base file.
- Validation: `pytest tests/test_tolerances.py` → 7 passed: the shipped file loads with every key; a missing
  `source` is refused naming `item.rest.floor`; unknown key, missing key, zero bound, unknown status refused; a
  pack at 4 mm tightens `floor.sink.foot`, one at 8 mm is refused naming the key. pyright strict 0 errors.
- Review: `KEYS` is the closed set, so a gate asking for a mistyped key fails loudly (`bound` raises).

### 8.3 Commit 3 evidence (2026-10-19)

- Implementation: `glb.py` gains `primitive_attribute`, `primitive_positions` (moved from
  `wardrobe/fixtures.py`, which now imports it), `triangles`, `primitive_counts`, `node_meshes`,
  `skinned_primitive_index`; `gates.py` `GateId.C1…C4`; `contact.py` `nearest` (libigl point-to-point);
  `mwcast/contact_gates.py`: `humanoid` (mixamorig → Godot humanoid names, as `*_bonemap.tres` maps them),
  `bone_group` (port of `_group`), `Skinning`, `limb_queries` (port of `_checks`' filters: dominant group,
  trunk share ≤ `MIX_W`, proximal cut-off, segment radius `LIMB_R`, Spine…Neck − 5 cm range), `trunk_patch`,
  `limb_in_trunk` / `gate_c1`, `OutfitManifest`, `gate_c2`; `cli.py contact <asset> [--outfit] [--tolerances]`,
  importing the contact gates lazily (a bare interpreter still runs `check-characters`; `contact` then
  reports FAIL "cannot look: numpy is not installed", I-S36-4).
- Real runs: `cast.py contact meshy_d.glb` → **C-1 PASS**: trunk patch 6 555 of 95 000 faces; Left sleeve 334,
  Left upper_arm 389, Right sleeve 418, Right upper_arm 412 query vertices, none inside, margin 5.0 mm each (no
  hand or leg vertex lies in the trunk's height range at the bind pose). `vitruvian.glb` with a hand-written
  manifest of its own layers (Tee/Jeans/Shoes 1, Hoodie 2, Pack 3, Hair) → **C-1 FAIL "cannot look: no
  trunk-dominant face"** (its body mesh carries no pure-trunk face; it is a layered asset, not a fused body) and
  **C-2 FAIL "cannot look: mesh 0 has no TEXCOORD_1"** (it predates the assembly's region channel). Recorded,
  as the design asks, whatever the result.
- Generated variants (`tests/test_contact_gates.py`, 5 passed): hand box sunk 8 mm into an open trunk shell →
  C-1 FAIL "Left hand is 8.0 mm inside the trunk"; 4 mm → PASS (worst 4.0 mm). Three nested shells over a
  two-region body, the middle pushed 3 mm through the outer at the top → C-2 FAIL "part_mid pokes 3.0 mm
  through part_outer in torso"; 1 mm → PASS, with all six layer-order pairs listed. A manifest naming
  `part_scarf` → FAIL cannot look; a body without TEXCOORD_1 → FAIL cannot look.
- Mutation: inverting the layer comparison (`<` → `>`) in `_layer_order` → the 3 mm variant passes and the
  test fails; restored. Whole pipeline suite: 158 passed. ruff, pyright strict: 0 errors.
- Bounded deviations (recorded):
  1. C-2 asks only about vertices the assembly flags as under an outer layer (TEXCOORD_1 `v`), as W-4 does;
     skin at an open neckline or below a hem is not covered and must not fail.
  2. A part vertex's region is its nearest body vertex's region (the assembly writes regions on the body
     only); this is what "an inner part's vertices in a region covered by an outer part" needs.
  3. "Or hidden (R-11)": C-2 reads the drawn `Body` node, from which the mask has removed the hidden set, so a
     hidden vertex is never asked about. `BodyCheck` (W-4's unmasked reference) is not used by C-2.
  4. Parts of class `hair`, and parts with no covered region (a bag, shoes), are never an outer shell.
  5. A C-2 that tests no vertex at all fails ("cannot look"), as W-4 refuses a check that tested nothing.
- Review: the G-9 filter constants (`MIX_W`, `LIMB_R`, cut-offs) choose query vertices; they are not bounds, so
  I-S36-2 holds (every bound is read with `Tolerances.bound`). No name of a body or garment decides anything:
  the body node name defaults to the assembly's `Body`, parts come from the manifest.

## 9. Execution contract (filled at freeze)

```text
PROJECT / PR:          S36-P1 — mwcast/contact.py and static contact gates
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-s36-p1-contact-module.md
RELATED / BINDING:     step-36-interpenetration.md; DEP-142, DEP-143, DEP-145; tools/character_pipeline/PIPELINE.md
IMPLEMENTATION BASE:   origin/main @ 8052bc06 (merge of #314, the freeze)
BRANCH / WORKTREE:     mvp1/pr-s36-p1 · /Users/yuema137/mineworld-worktrees/impl-s36-p1
PUBLICATION:           commit, push, open the PR; the coordinator merges (charter)
APPROVED SCOPE:        §3
FROZEN INVARIANTS:     I-S36-1, I-S36-2, I-S36-4, I-S36-5 (step §5)
SEQUENCE:              commits 1-5
VALIDATION BUDGET:     local pytest of the new tests only (charter: speed first); no Godot, no money
LIVE DOCUMENTATION:    this file, §8
```

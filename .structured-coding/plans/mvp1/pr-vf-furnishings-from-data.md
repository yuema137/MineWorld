# PR vf-furnishings — the furniture both clients draw comes from the world's data

**Lifecycle:** `DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered as recommended)`. No implementation before a `DESIGN FROZEN` header and a filled
contract (§11). The questions in §9 need the operator's ruling first.

**Parent:** [`pr-vf-furniture-solids.md`](pr-vf-furniture-solids.md) §4 (Q-VF-F1, recommended option (b)).
**Coordinates with:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.5 (S36-P5, O-7, Q-S36-9: P5
needs a link from a drawn prop to its `Solid`), [`../mvp0/pr-s33b-lakeside-bodies.md`](../mvp0/pr-s33b-lakeside-bodies.md)
§5.3 and [`../mvp0/pr-s33f-interiors-sit-points.md`](../mvp0/pr-s33f-interiors-sit-points.md) (Lakeside's solids and
interiors, frozen, not yet implemented). **Records:** `ARC-307` (id confirmed by the coordinator 2026-10-19), amending
`ARC-39` and completing `ARC-306`. **Audit base:** `origin/main @ 5a8c6f28` (2026-10-19, after #292 and #254).
**Branch (design):** `docs/pr-design-vf-furniture-data`.

No model is called by this design, and it spends no money.

---

## 0. Binding inputs

- **Operator ruling (2026-10-19, relayed by the coordinator).** "3D furniture must come from world data, not
  hand-placed in client scripts, as its own PR after 12e." 12e (#254) and the furniture-solids fix (#292) have
  merged, so this PR can start once frozen.
- **`ARC-306`.** Everything drawn that a body could bump into is a solid. "Visual only" is kept for what a body
  cannot meet: things on a table or a counter, on a wall, overhead, rugs.
- **No world-specific hard-coding** (charter, 2026-10-18). The mechanism reads the world's data. A fix that only
  makes the café town pass is rejected. Every acceptance criterion names a second world or a generated variant.
- **The server decides where a body is** (`ENGINEERING_RULES.md` §8, `DEP-20`). The client draws; it never decides
  what blocks.
- **Semantic space is not render geometry** (`CLAUDE.md` §4 rule 14). The world may say "a chair stands here,
  facing east". It may not name a mesh, a texture or a Godot node.
- **S36-P5 (Q-S36-9, ruled "yes").** P5 decides the drawn-prop ↔ `Solid` link together with Q-VF-F1. This design
  supplies that link as data, so P5 only has to check it (§3.7). P5 is designed in parallel; this document was
  written read-only against step 36 and changes nothing in it.

## 1. Goal

Make one data source hold both what the server resolves against and what the clients draw:

```text
place file  body.furnishings   key, kind, at, facing      "a round table stands here"
            body.solids        min, max, height, furnishing   "and this box is what a body meets"
                    │
                    ▼ disclosed with the place (place-shape, unchanged component)
3D client   furnishing kind → a model from the 3D Presentation Pack, at `at`, turned to `facing`
2D client   furnishing kind → a role from the 2D Presentation Pack, or the solids' footprints, plainly
P5 check    furnishing key → the drawn mesh and the solids that name it
```

Example. The café's east table today is written twice: once in GDScript (`cafe_interior.gd:477`, scene
`(4.86, −3.50)`, yaw 0.15) and once, by hand, in `places/cafe.yaml` as the box `(4 443, 2 743)–(5 277, 3 577)`.
After this PR it is written once:

```yaml
body:
  furnishings:
    - { key: table.east, kind: table.round, at: { x: 4860, y: 3160 }, facing: <from the slice's yaw, C2> }
  solids:
    - { min: { x: 4443, y: 2743 }, max: { x: 5277, y: 3577 }, height: 716, furnishing: table.east }
```

The server reads the box, as today. The 3D client reads the furnishing and draws the pack's model for
`table.round` there. A world whose café has its table one metre further west draws it one metre further west,
with no client change.

## 2. Audit (`origin/main @ 5a8c6f28`)

### 2.1 What exists

| Concern | Today | Where |
| --- | --- | --- |
| Server solids | `Solid { min, max: Corner, height }`, integer mm, no id, no kind; at most 64 per place (`SOLIDS_MAX`) | `systems/bodies/src/component.rs:15, 79–108` |
| The place form | `body: { floor, solids? }`; the type is its validation (`PlaceShape::checked`), errors at the loader's line | `component.rs:111–200`; `section.rs:1–60` |
| Disclosure | `place-shape` (schema 1) is disclosed for the observer's **own place only**: "a person learns the walls and the objects of the room they stand in and of no other" | `systems/bodies/src/system.rs:300–333` |
| Consumers of `place-shape` | 3D `client_bodies.gd` (colliders from every disclosed solid), `slice_probe_geometry.gd` (`--world --geometry`), `slice_probe_bodies.gd`; rule controller reads only `floor` (`cognition/rule-controller/src/needs.rs:780–792`); the 2D client draws nothing of it | listed files |
| The café's furniture | drawn from GDScript constants: three tables and six chairs (`cafe_interior.gd:467–497`), three stools (`:458–462`), the display shelves and `Shelf_01` (`:558–561`), two potted plants (`:554–557`) | `clients/3d-spike/scripts/slice/cafe_interior.gd` |
| Collider rule | `put_solid`: the model's glTF bounds, yawed, × 0.92, centred on the placed position | `slice/dressing.gd` `put_solid`, `put` |
| Both towns | `market-town`'s café is the same room (`town_bodies.rs` TD-3) and carries the same solids | `worlds/market-town/places/cafe.yaml` |
| Street furniture | 62 solids in `street.yaml`; the terrace table sets are **two crossed boxes each** (`street.yaml:80–86`); drawn by `streetscape.gd` (`Props.place` + `Build.box_blocker`) | `worlds/social-cafe/places/street.yaml`; `slice/streetscape.gd:59–376` |
| Lakeside | no `body:` yet; S33-b (frozen, unimplemented) authors its solids; S33-f (frozen) draws its interiors in client code at those solids, and says its chairs are "drawn props, not solids" (F-S33f-5) | `pr-s33b-lakeside-bodies.md` §5.3; `pr-s33f-interiors-sit-points.md:145–146, 205` |
| Sit points | `group`: "optional slug naming the furniture the point belongs to" — no furniture key exists for it to name | `CORE_CONCEPTS.md` §6.4; `MODULE_SPEC.md:889` |
| Pack bindings | the 3D pack binds data in `characters/cast.yaml` (look → character); the 2D pack binds roles in `assets/asset_bindings.yaml`; "the core asks for roles and never for files" (`ARC-46`) | `clients/3d-spike/PRESENTATION.md` §2; `clients/2d/scripts/presentation.gd` |
| Prop mount data | S36-P1 records each prop's mount (`floor`, `surface`, `wall`, `ceiling`) | `clients/3d-spike/assets/models/contact.json` (P1, #318) |
| Recorded observations | precedent for a checked-in observation fixture | `tests/fixtures/observations/needs/market-town-night.json` |

### 2.2 Findings

- **F-FD-1. Positions are written twice.** Each café table, chair and stool lives in GDScript and, by hand
  transcription, in two YAML files. #292 closed one gap by adding the missing copies; the next differing world
  reopens it.
- **F-FD-2. `ARC-306` residue in the café.** Five floor-standing pieces a body can meet are drawn with no solid,
  in either town: the display shelves on the west wall (`cafe_interior.gd:558`), `Shelf_01` on the east wall
  (`:560`), and the two potted plants (`:554`, `:556`). A person can walk into them today.
- **F-FD-3. One piece can need several boxes.** A terrace table set is held by two crossed boxes. So a link from a
  drawn piece to "its solid" must allow several solids per piece. A `kind` field on each solid (Q-VF-F1's literal
  option (b)) cannot express that, and gives P5 no stable key.
- **F-FD-4. A client sees only the room it stands in.** Through the café's window the street is visible, and
  through the florist's window its shop. A piece in a place the client has never stood in is not disclosed. This
  matters for street furniture and the shop; it does not matter for the café room, because the 3D layout binds
  only after the observer has stood in the café (`slice_layout.gd` `anchor`).
- **F-FD-5. S33-f conflicts with both rulings.** Its fit-out is placed in client code at S33-b's footprints, and
  its table chairs are props without solids. Both were frozen before `ARC-306` and before this ruling.
- **F-FD-6. The 2D client draws no furniture.** It has nothing to keep in parity with today; after this PR it can
  draw what the world discloses.

## 3. Design

### 3.1 The term: a furnishing

A **furnishing** is a named piece that stands on a place's floor and that a body can meet: a table, a chair, a
shelf unit, a bench, a lamp post. It is semantic place data owned by the `bodies` System Pack. It is not an Entity
and not an Item: it never moves, is never owned or carried, and has no state. It is what the world says stands
there; its solids are what a body meets; its look belongs to a Presentation Pack.

- "Fixture" is not used, because the repository already uses it for test fixtures and sample worlds.
- A furnishing that moves, is bought or is carried is out of scope. That would be an Item with a body
  (`ARC-36`), Q-VF-F1's option (c).
- `CORE_CONCEPTS.md` gains §6.5 "Furnishing" beside §6.4 "Sit point", and `MODULE_SPEC.md`'s `body:` section
  gains the fields (C1).

### 3.2 The place form

```yaml
body:
  floor: { min: { x: 0, y: 0 }, max: { x: 8320, y: 10320 } }
  furnishings:                 # optional, at most 64 per place
    - key: table.east          # slug, 1–48 bytes of a–z, 0–9, '-', '.'; unique in the place
      kind: table.round        # slug, same grammar; what it is, never what it looks like
      at: { x: 4860, y: 3160 } # mm, place frame: the centre of its drawn footprint
      facing: 0                # millidegrees, 0 north, 90 000 east (as SitPoint.facing); C2 converts the slice's yaws
  solids:
    - { min: { x: 4443, y: 2743 }, max: { x: 5277, y: 3577 }, height: 716, furnishing: table.east }
    - { min: { x: 3860, y: 6570 }, max: { x: 8320, y: 7350 }, height: 1060 }   # the counter: shell, no furnishing
```

Rules, all checked by `PlaceShape::checked` (so a bad value cannot be constructed from YAML, a fact or a
snapshot):

1. Every furnishing is named by at least one solid's `furnishing`, and every `furnishing` on a solid names a
   furnishing of the same place. So a furnishing with nothing a body can meet cannot be written (`ARC-306` by
   construction), and P5's link is total.
2. `at` lies inside the bounding rectangle of its own solids. A typo that puts the drawn chair 3 m from its box is
   refused at the loader's line.
3. `at` is within the floor and within ±100 000 mm; `facing` is 0 … 359 999.
4. Keys are unique within the place; at most 64 furnishings; the 64-solid cap is unchanged and counts every solid.
5. Unknown fields are refused, as today.

The `kind` grammar allows `.` so that a world can be specific and still fall back: a client that has no binding
for `chair.painted` uses `chair` (§3.5). A kind is not a closed list; it is a label the world's author chooses, as
a tag is.

**Why the solids stay a flat list (not nested inside each furnishing).** The resolver, `client_bodies.gd`, the
geometry probe and the rule controller read `solids` today. With a flat list and an optional `furnishing` field,
all four keep working unchanged, the resolver's insertion order is the authored order as before, and the crossed
terrace boxes simply name the same furnishing. Nesting would make every reader learn a second place to find
solids.

### 3.3 The Rust types (`systems/bodies`)

```rust
pub struct FurnishingKey(..);     // slug, validated on construction
pub struct FurnishingKind(..);    // slug, validated on construction
pub struct Furnishing { key: FurnishingKey, kind: FurnishingKind, at: Corner, facing: Millidegrees }
pub struct Solid { min, max, height, furnishing: Option<FurnishingKey> }   // serde: skip if None
pub struct PlaceShape { floor, solids, furnishings: Vec<Furnishing> }      // serde: default, skip if empty
```

- `PlaceShape::room()` is unchanged: the resolver ignores keys and kinds.
- `place-shape` stays **schema 1**. Both fields are optional with defaults, so every saved `place-shaped` fact and
  snapshot decodes to the same value it did. `Replay` of an existing save stays byte-identical. (If the operator
  prefers a schema bump, Q-FD-6.)
- `Millidegrees` comes from `mineworld-contracts` (`contracts/src/spatial.rs:91`), as posture's `facing` does.
- The schemars description of each field feeds the generated reference (`docs/reference/sections.md`); its drift
  check is re-run (25a).

### 3.4 Disclosure

Nothing new is disclosed through a new path. The furnishings travel inside `place-shape`, to whoever perceives the
place, which is the observer's own place (F-FD-4). A client keeps what it has learned about a place for the
session, as `SliceLayout` keeps passages and tags, so a room stays furnished after the observer steps out of it.

That is enough for this PR's scope (the café room). Widening perception to neighbouring places is a presence and
networking change, asked in Q-FD-3 and left to a later PR.

### 3.5 The 3D client

**New: `clients/3d-spike/scripts/slice/furnishings.gd` (`SliceFurnishings`).** Built like `client_bodies.gd`:

- `observe(obs, layout, world_root)`: for the observer's place, when its disclosed `place-shape` changed, (re)build
  that place's furnishings under one node `Furnishings/<place id>`. Built places are kept when the observer leaves.
- For each furnishing: look up the binding for `kind`, then for each shorter `.`-prefix (`chair.painted` →
  `chair`). Place the bound model with `SliceProps.put` at `layout.to_scene(place, at)` on the floor, turned to
  `facing`. Scene yaw = `front − facing` in radians, where `front` is the binding's yaw that turns the model to face
  north (the world's frame is fixed, so the layout only translates; `slice_layout.gd:5`).
- No binding at all: a plain box over each of its solids, in the palette's neutral timber, and a warning row in a
  `FURNISHED` report (like 16c's `SHOWN`). A disclosed piece is never left undrawn, because an undrawn solid is an
  invisible wall.
- It builds **no colliders**. `client_bodies.gd` already builds one from every disclosed solid, including these.
- It reads no place key, no world id and no kind literal (I-FD-2).

**New: the 3D pack binding `furniture/furnishings.yaml`** (JSON-compatible YAML, `DEP-16`, specified in
`clients/3d-spike/PRESENTATION.md` §5, new):

```json
{
  "format": 1,
  "kinds": {
    "table.round":  { "model": "round_wooden_table_02", "front_md": 0, "tint": [1.04, 0.96, 0.86],
                      "dressing": [[{ "model": "tea_set_01", "parts": ["…"], "at": [0.05, -0.02], "yaw_md": 30000 }]] },
    "chair.dining": { "model": "dining_chair_02", "front_md": 90000, "tint": [0.98, 0.92, 0.84] }
  }
}
```

- `model` is a prop slug of the client's model library whose S36-P1 mount is `floor`; a check refuses any other
  mount (C5).
- `parts` selects members of a variant set, as `SliceProps.keep` does.
- `dressing` is the hand-scale dressing that follows the piece (cups on a table): a list of sets, each a list of
  props in the piece's own frame (metres, on its top surface). The set is chosen by a stable hash
  (FNV-1a 32) of `place id + "/" + furnishing key`, so the same table always carries the same cups and a moved
  table carries them with it. Hand-scale dressing never has a solid (`ARC-306` 1).

**Removed from `cafe_interior.gd`:** the tables, chairs and stools of `_loose_seating` and `_window_bench`, the
display shelves, `Shelf_01` and the two floor plants (with what stands on them moved into their bindings'
`dressing`). What stays is the room's **shell**: walls, linings, the counter and back bar, the window bench and its
drinks ledge, the west dresser, the door, the back room, wall and ceiling pieces, lighting. Each shell solid stays
checked by `--world --geometry` within 150 mm, as today (Q-FD-4).

**The unconnected slice (no server).** Standalone shots and perf runs have no disclosure. They read a recorded
observation of the café, checked into `tests/fixtures/observations/furnishings/social-cafe-cafe.json`, written by
a CLI test and failing on drift from the world files (C3). The standalone café therefore draws the same pieces as
the connected one, from the same data (Q-FD-5).

### 3.6 The 2D client

The 2D client draws each disclosed furnishing of the place it shows: the pack's role `furnishing.<kind>` (with the
same `.`-prefix fallback) at `at`, turned to `facing`; with no binding, or with no pack, the footprints of its
solids as plain filled shapes. It draws plain solids without a furnishing (the shell) as outlines, so the 2D view
shows every box a body meets. Same rule as 3D: nothing it was not told, and nothing it was told left out.

### 3.7 What S36-P5 gets

1. **The link, as data.** For every place: furnishing key → its kind, `at`, `facing`, and the indices of the
   disclosed solids that name it. Total by rule 1 of §3.2.
2. **A dump.** `slice_main --furnishings-dump=<dir>` (headless, connected or from the recorded observation) writes
   one `furnishings.json` per place:

   ```json
   { "format": 1, "world": "<world id>", "place": "<place id>",
     "furnishings": [ { "key": "table.east", "kind": "table.round", "bound": "table.round",
                        "model": "round_wooden_table_02", "parts": [],
                        "gltf": "assets/models/round_wooden_table_02/round_wooden_table_02.gltf",
                        "to_place": [16 floats, row-major: model-local metres → place frame metres, x east, y north, z up],
                        "solids": [4], "dressing": [ { "model": "...", "gltf": "...", "to_place": [...] } ] } ],
     "shell": [ { "solid": 0 } ] }
   ```

   P5 loads each glTF with trimesh, applies `to_place`, and checks (a) the floor footprint inside the union of its
   solids within `scene.mesh_over_solid`, (b) piece–piece overlap, (c) rest on the floor. P5 needs no Godot export
   format and no knowledge of any scene.
3. **The generated variant P5 asked for** (a prop drawn 60 mm wider) is a binding override in a scratch pack, and
   "one solid moved to overlap another" is a scratch copy of a place file. Both are inputs, not code.

How P5 reports and gates is P5's design; this PR does not implement any of P5's checks.

### 3.8 Shell or furnishing

A piece is a **furnishing** when it stands free on the floor and could stand elsewhere in another layout of the
room: tables, chairs, stools, shelf units, plants in pots, benches, lamp posts, planters, A-boards. It is **shell**
when it is part of the building: walls, a fitted counter, a built-in bench, a dresser built into the wall, a door
leaf, a stair. Shell stays drawn by the place's depiction (16c, S33-c) with plain solids and is checked by
`--geometry`. The test is "would a different world's version of this room move it?", not its size.

### 3.9 Lakeside and Market Town

- **Market Town.** Its café is the same room; C2 writes the same furnishings there. Its street is not in scope
  (§4).
- **Lakeside.** It has no `body:` on main. S33-b (frozen) will author its solids and S33-f (frozen) will draw its
  interiors. Under the operator's ruling and `ARC-306`, both need a bounded revision, asked in Q-FD-2:
  S33-b writes its free-standing pieces (the pump, the long table and its chairs, the bread racks, the workbench,
  the boat on its slip, the well, the benches, the boulders, the pier bench) as furnishings with solids; S33-f
  binds their kinds in the 3D pack and draws only the shells in code; the long table's chairs become solids
  (superseding F-S33f-5's "chairs are props"). This PR proves the mechanism on Lakeside before S33-b lands with a
  generated variant: a scratch copy of `worlds/lakeside` whose bakery carries S33-b §5.3's floor and solids plus
  furnishings (AC-FD-4).

## 4. Scope

**In.** `systems/bodies` (types, validation, schemars docs, tests); both towns' `places/cafe.yaml`; the five
`ARC-306` residue solids (F-FD-2) in both towns; `town_bodies.rs`; the 30-day decisions fixture re-captured; the
3D `SliceFurnishings`, the 3D pack binding file and its spec; `cafe_interior.gd` reduced to the shell; the recorded
observation and its drift test; the 2D drawing and its pack roles; the binding check (C5); the P5 dump;
`CORE_CONCEPTS.md` §6.5, `MODULE_SPEC.md` `body:`, `ARC-307`.

**Out.** The street's furniture and the florist/store interior (both are seen before they are entered, F-FD-4;
they follow in PR vf-furnishings-b after Q-FD-3); Lakeside's real data (S33-b rev. 2); any of P5's checks; seated
heights from a furnishing's binding (`town.gd` `CAFE_SEAT_Y` stays; noted for S34); validating a sit point's
`group` against a furnishing key (posture's seam, noted for S34-b); any change to the resolver.

## 5. Commits

### C1 — the contract

**Scope.** `systems/bodies/src/component.rs` (`Furnishing`, `FurnishingKey`, `FurnishingKind`, `Solid.furnishing`,
`PlaceShape.furnishings`, the §3.2 rules in `checked`), `section.rs` (place form doc and example),
`systems/bodies/tests/genesis.rs` (or a new `tests/furnishings.rs`), `docs/CORE_CONCEPTS.md` §6.5,
`docs/MODULE_SPEC.md` `body:`, `docs/decisions/ARC-307.md` + `scripts/check_decision_ids.py --write-index`, the
generated reference.

**Validation.** Load tests, one per rule: an orphan furnishing, a solid naming an unknown key, `at` outside its
solids, a duplicate key, a 65th furnishing, `facing` 360 000 — each refused at its line with its message. A place
with furnishings discloses them inside `place-shape`, and an old fact without them decodes unchanged (the existing
replay test passes untouched). Mutation: drop rule 1 → the orphan test turns red.

### C2 — the café's data, both towns

**Scope.** `worlds/social-cafe/places/cafe.yaml`, `worlds/market-town/places/cafe.yaml`: the twelve existing
furniture solids gain `furnishing`; twelve furnishings (3 `table.round`, 4 `chair.dining`, 2 `chair.painted`,
3 `stool.bar`) at the slice's positions and yaws; five new furnishings with solids for F-FD-2 (`shelves.display`,
`shelves.wall`, 2 `plant.potted`), sized by the `put_solid` rule from `--slice-measure`. `tools/cli/tests/town_bodies.rs`:
a stride into the display shelves stops at them in both towns (fails on the base). The 30-day fixture re-captured.

**Failure case.** A new solid breaks a doorway's or a start's clearance at genesis: move the drawn piece within the
room (bounded, recorded); never shrink a solid below its mesh.

### C3 — the 3D client

**Scope.** `slice/furnishings.gd` (new), its call from `slice_link.gd` beside `client_bodies.observe`; the default
3D pack's `furniture/furnishings.yaml`; `PRESENTATION.md` §5; `cafe_interior.gd` reduced to the shell; the
recorded observation and its CLI drift test; the `FURNISHED` report.

**Validation.** Headless `--world --link` in social-cafe: `FURNISHED` lists 17 pieces, 0 unbound; each drawn
piece's placed AABB centre within 5 mm of its `at`. The same run on market-town. A generated variant (social-cafe
copied to the scratchpad with `table.east` 1 m west and a chair turned 90°) draws both where the data says, and a
stride into the moved chair stops there on the server. Shots re-baselined once; the reviewer compares them with the
base (furniture in the same places, cups may differ). `--world --geometry` still passes.

### C4 — the 2D client

**Scope.** The 2D scene draws furnishings and shell solids (§3.6); default 2D pack roles `furnishing.<kind>` for
the café's kinds (plain shapes are acceptable art for this PR). Validation: a headless capture of social-cafe's
café shows the pieces at the 3D positions; the generated variant's moved table moves.

### C5 — checks and the P5 dump

**Scope.** A Rust test (`tools/cli/tests/furnishings.rs`): for every world under `worlds/` (listed, not named),
every furnishing kind resolves to a binding in each default pack (2D and 3D) through the `.`-prefix rule, and every
3D binding's model has mount `floor` in `contact.json`. A key scan (I-FD-2) over `furnishings.gd` and the 2D
drawing script finds no place key, world id or kind literal. `--furnishings-dump` (§3.7) on social-cafe and on the
Lakeside variant.

### C6 — documentation and the ledger

`clients/3d-spike/README.md` row, `step-11` §19.3.1 note ("solids may name a furnishing, `ARC-307`"), this
document's ledger.

## 6. Acceptance criteria

| Id | Criterion | Worlds |
| --- | --- | --- |
| AC-FD-1 | No café furniture position or yaw is written in client code; a scan finds no furniture slug in `cafe_interior.gd` | social-cafe, market-town |
| AC-FD-2 | Every drawn café piece stands where its world's furnishing says (≤ 5 mm, ≤ 0.5°), connected and standalone | social-cafe, market-town |
| AC-FD-3 | A generated variant with a moved table and a turned chair is drawn moved in both clients, and the server stops a stride at the moved chair | social-cafe variant |
| AC-FD-4 | A Lakeside variant's bakery (S33-b §5.3 solids + furnishings) validates, discloses its furnishings, draws in 2D, and dumps for P5 | lakeside variant |
| AC-FD-5 | The five F-FD-2 pieces are solids; a stride into each stops at it | social-cafe, market-town |
| AC-FD-6 | Every kind used by every shipped world is bound in each default pack; an unbound kind is drawn as plain boxes and reported, never dropped | all worlds; a scratch world with an unknown kind |
| AC-FD-7 | Old saves replay byte-identical (`mineworld replay` on a save made on the base) | social-cafe |

## 7. Invariants

- **I-FD-1.** Every furnishing has at least one solid, and every solid's `furnishing` names one (by type).
- **I-FD-2.** No client script that draws furnishings names a place, a world or a kind.
- **I-FD-3.** The client builds colliders only from disclosed solids; furnishing drawing adds none.
- **I-FD-4.** The resolver's behaviour is unchanged for a place without furnishings (the existing route and stride
  tests pass untouched).
- **I-FD-5.** A disclosed furnishing is always drawn: by its binding, by a prefix's binding, or as plain boxes.

## 8. Risks

| Risk | Effect | Mitigation |
| --- | --- | --- |
| Yaw conventions differ between the world (clockwise from north) and Godot (counter-clockwise about +Y) | chairs face the wrong way | one conversion in `furnishings.gd`, `front_md` per binding, checked by AC-FD-2 with a turned-chair variant |
| The F-FD-2 solids break a start or doorway clearance | genesis refuses the town | C2 failure case; bounded move of the drawn piece |
| Shots change (cup arrangement) | screenshot diffs | one recorded re-baseline in C3, reviewed against the base |
| Standalone recorded observation drifts | standalone draws stale furniture | the drift test fails on any difference |
| S33-b/S33-f implemented before their revision | Lakeside furniture hand-placed again | Q-FD-2 ruled before S33-b starts |
| Shop and street still hand-placed | the ruling only half met | named out of scope with its reason; PR vf-furnishings-b follows Q-FD-3 |

## 9. Questions for the operator, each with a recommendation

**Ruled 2026-10-19 (operator, primary session):** every question below is answered as recommended.

| Id | Question | Recommendation | 中文 |
| --- | --- | --- | --- |
| Q-FD-1 | Data shape: (a) `kind`/`yaw` on each solid (Q-VF-F1 (b) as written); (b) a `furnishings` list (key, kind, at, facing) with solids naming their furnishing; (c) furniture as Items | **(b).** (a) cannot hold one piece with two boxes (F-FD-3) and gives P5 no key; (c) is heavy for things that never move | 家具数据：在 `body:` 里加 `furnishings` 列表（key、kind、位置、朝向），每个 solid 写明属于哪件家具？推荐 |
| Q-FD-2 | Revise S33-b and S33-f (frozen, not started): Lakeside's free-standing pieces become furnishings with solids, its table chairs become solids, S33-f draws only shells in code | **Yes**, as bounded revisions applied before S33-b starts | 湖边 S33-b/S33-f 改为用 furnishings 数据，椅子也变成 solid？推荐同意 |
| Q-FD-3 | A client sees only its own room's furnishings. For the street and the shop (seen through windows before entering): (a) keep own-place disclosure and convert them later; (b) a follow-up step widens disclosure of static place data to places one passage away | **(a) now, (b) as a separate design** before PR vf-furnishings-b | 客户端只知道自己所在房间的家具。街道和花店先不改，另开设计扩大可见范围？推荐 |
| Q-FD-4 | Built-in pieces (counter, window bench and ledge, dresser, door leaf, stair) stay shell drawn by the depiction, checked by `--geometry`; only free-standing pieces become furnishings | **Yes** | 柜台、固定长凳等建筑部分仍由场景画，只有可移动摆放的家具来自数据？推荐 |
| Q-FD-5 | The standalone slice (no server) reads a recorded, drift-checked observation | **Yes** | 不连服务器时，用录制好、CI 校验的数据画家具？推荐 |
| Q-FD-6 | Keep `place-shape` at schema 1 (new fields optional, old saves decode unchanged) | **Yes** | `place-shape` 版本保持 1（新字段可选，旧存档不变）？推荐 |
| Q-FD-7 | Fix F-FD-2 here: the display shelves, `Shelf_01` and two floor plants become solids in both towns (decisions fixture re-captured once) | **Yes** | 咖啡馆里还有 5 件能撞到却没有 solid 的家具，本 PR 一并修？推荐 |
| Q-FD-8 | Decision id `ARC-307`; term "furnishing" (not "fixture", which means test fixture here) | **Yes** | 记录编号 ARC-307，术语用 furnishing？推荐 |

## 10. Ledger

- 2026-10-19 — DRAFT written (design session, worktree `/Users/yuema137/mineworld-worktrees/design-vf-furniture-data`).
  Audit base `origin/main @ 5a8c6f28`. No implementation.

## 11. Execution contract (filled at freeze)

```text
PROJECT / PR:          vf-furnishings — furniture both clients draw comes from the world's data
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-vf-furnishings-from-data.md
RELATED / BINDING:     pr-vf-furniture-solids.md; step-36 §4.5; pr-s33b, pr-s33f (as revised by Q-FD-2); ARC-39, ARC-306, ARC-307
IMPLEMENTATION BASE:   origin/main at freeze
BRANCH / WORKTREE:     mvp1/pr-vf-furnishings · /Users/yuema137/mineworld-worktrees/impl-vf-furnishings
PUBLICATION:           commit, push, open the PR; the coordinator merges (charter)
APPROVED SCOPE:        §4
FROZEN INVARIANTS:     I-FD-1 … I-FD-5
SEQUENCE:              C1 → C2 → (C3 ∥ C4) → C5 → C6
VALIDATION BUDGET:     the new tests and one headless slice run per world, via with-slot.sh (charter: speed first); no money
LIVE DOCUMENTATION:    this file, §10
```

## 12. Execution ledger

- 2026-10-19 — Implemented on `mvp1/pr-vf-furnishings` from `origin/main @ 1308db6d` (session
  `impl-vf-furnishings`). Decision id `ARC-307` free on every remote branch; record written, index
  regenerated.
- C1 `systems/bodies`: `furnishing.rs` (`FurnishingKey`, `FurnishingKind`, `Furnishing`), `Solid.furnishing`,
  `PlaceShape.furnishings` (serialized only when non-empty: schema 1, old payloads unchanged), the §3.2 rules
  in `PlaceShape::checked`; `tests/furnishings.rs` (one refusal per rule, disclosure inside `place-shape`).
  `CORE_CONCEPTS.md` §6.5, `MODULE_SPEC.md` `body:`, `ARC-307`, the generated reference.
- C2 both `places/cafe.yaml`: 16 furnishings, the 12 furniture solids name theirs, 4 new solids;
  `town_bodies.rs` stride into the display shelves (both towns); `rooms_of` skips the furnishings list;
  the 30-day decisions golden re-captured.
- C3 `slice/furnishings.gd`, `slice_link.gd` hook and `FURNISHED` line, standalone draw from
  `tests/fixtures/observations/furnishings/social-cafe-cafe.json`, `cafe_interior.gd` reduced to the shell,
  3D binding `presentation/mineworld-default/3D/furniture/furnishings.yaml`, `PRESENTATION.md` §5,
  `--furnishings-dump`; the geometry probe checks shell solids only.
- C4 `clients/2d/scripts/scene/furnishings.gd`, roles `furnishing.{table,chair,stool,shelves,plant}`, two
  plain SVGs (chair, stool), 2D `PRESENTATION.md`.
- C5 `tools/cli/tests/furnishings.rs`: kinds of every world bound in both packs with mounts; unbound kind
  resolves to nothing; I-FD-2 literal scan; recorded-observation drift against the real server; the Lakeside
  variant (AC-FD-4) validates.

**Bounded deviations (recorded, no frozen invariant touched).**

- D-FD-1. F-FD-2 says five pieces; the audit finds four (display shelves, `Shelf_01`, two potted plants), so
  the café has 16 furnishings, not 17.
- D-FD-2. The display shelves' model, as drawn today, reaches 156 mm into the west wall; its solid is cut
  at the floor's edge (x 0). Nothing a body can reach is lost.
- D-FD-3. The potted plants were placed by their origin (`Props.place`); as furnishings they are centred on
  `at` like every other piece (`SliceProps.put`), a shift of a few centimetres.
- D-FD-4. The 2D pack's café `fittings` placed three tables and two plants by hand; they are removed, since
  the world now discloses them. Chairs and stools have no 2D sprite, so two plain SVGs bind them.
- D-FD-5. The P5 dump gives each dressing prop's placed point (`at_place`) rather than a full 4×4.

**Validation (speed first).** Rust: fmt; one slot run of build, `reference --out`, the 30-day golden, the
bodies and CLI furnishings tests, the town_bodies stride/geometry tests, the reference test and clippy
(results below). Godot runs (AC-FD-2/3 headless slice, 2D capture, generated variant drawn) are not run
locally (slot queue); INCONCLUSIVE until the milestone run.

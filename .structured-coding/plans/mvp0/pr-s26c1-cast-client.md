# PR S26-c1/c2 — The café-town cast drawn from the server's look, in both clients

## Status

```text
Authority:   operator rulings of 2026-10-19, relayed by the coordinator session: wire the ten generated
             café-town characters into BOTH clients so each Person's look comes from the server, with no
             client-side identity guessing; commit the characters via Git LFS now, compress the release
             later; 2D uses realistic sprites rendered from the same 3D characters. Speed first, but it
             must really work in the running client.
Frozen by:   the operator ruling above (scope, invariants and acceptance below). This is the PR ledger;
             step-26 §§4.3-4.5, 9, 12 are the governing design.
Branch:      mvp0/pr-s26c1-cast-client, on PR #287 (fix/vf-person-identity) + b805ddaf (the ten characters,
             Git LFS).
State:       READY FOR OPERATOR REVIEW (not pushed, not merged).
```

## 1. Scope

In: step-26 S26-c1's generic, data-driven cast body (`character.yaml` → `Human.build_character`) and
S26-c2's connected figures and player through the cast binding; the ten looks bound in `cast.yaml`; the
2D client's `look:<key>` roles bound to sprites rendered from the same characters, Vera included; LFS
pointer degradation; provenance; validation in both running clients.

Out (left to the PRs step-26 §12 names): removing `Human.Body`, `SRC_D` and Vera's `_D` constants into
her own manifest (AC-S26-6's pixel-equality proof belongs with that move); `extras` and the promenade /
street figures (`town.gd`, `streetscape.gd`); Wes Calloway's character (unbound in both clients, so he
falls back per #287's rule: the town figure in 3D, the `gen_player` stand-in in 2D); the lakeside cast;
release packing (S26-f).

## 2. Invariants

- I-1 (I-PID-1) A Person's body is chosen from the disclosed `look` only, through the pack's binding; no
  client code names a person.
- I-2 No per-person code: every cast body is built by one function from its manifest.
- I-3 Vera (the reference body) and the town figure build exactly as before: same scenes, same constants,
  same clip-library cache entries (keyed by `Human.Body`), same RNG draws.
- I-4 A character whose files are Git LFS pointers, missing, unimported or whose manifest is incomplete
  is drawn as the town figure with one warning naming why; nothing fails.
- I-5 A cast body is drawn at its authored stature, instance scale 1 (`SD-S26-3`).

## 3. Decisions

- **D-1 Where the manifest lives: the Presentation Pack**, `presentation/mineworld-default/3D/characters/cast/<id>/character.yaml`,
  as step-26 §4.5 and PRESENTATION.md §2 already name that directory as the character's home (brief,
  provenance, references). Not the client index `assets/characters/cast/index.json`: the index says which
  bodies this client has imported; the manifest says how the pack draws a character, and a second 3D client
  would read the same manifest. The index keeps only `vera-lindgren: reference` until her constants move
  (§1 Out), and that entry wins over a manifest. Specified in `clients/3d-spike/PRESENTATION.md` §2.1;
  `scripts/check_cast.py` gains CC-7 (every named character has its directory) and CC-8 (manifest shape,
  scene is a file of the client) with a self-test of X-14's shape.
- **D-2 One generic builder beside the old one.** `Human.build_character(manifest)`: its own scene (cached
  by path), scale 1, its `MW_Cast` material duplicated with the manifest's surface (roughness 0.9,
  specular 0.25: `_meshy`'s values), `Posture.natural_stance()`, and the clips placed at the manifest's
  speeds through two instance fields (`_walk_mps`, `_jog_mps`) that default to the old constants, so the
  town and reference bodies are unchanged (I-3). `NPC.make_body(rng, pose, height, spec)` dispatches on the
  spec `SliceCast.body_for` returns (`{kind: cast, manifest}` / `{kind: reference}` / `{}`); `NPC.make` is
  unchanged for every other caller. `CharacterSlot` tracks a `body_key` (character id, `reference`,
  `town`) instead of a boolean, so the player's body follows the observer's look to any cast member.
- **D-3 Git LFS is a material dependency → `DEP-141`** (next free DEP id after `DEP-140` on this base; if
  another branch takes 141 first, renumber per `docs/DECISIONS.md` §3). CI does not fetch LFS objects.
  Measured: Godot 4.7.2 `--import` of a glTF whose `.bin` and `.png` are pointers prints importer errors
  and **exits 0** (scratch project, 2026-10-19), so the composite action's import step
  (`.github/actions/godot/action.yml`) and `tools/cli/tests/client_far_field.rs` `import()` — the only
  CI paths that import the 3D project (jobs `farfield` in `ci.yml`; the Godot jobs of `nightly.yml`) and
  judge only the exit status — are unaffected; no test greps import logs for `ERROR`. At run time the
  client never loads a failed import: `SliceCast.asset_problem` reads the first bytes of the scene, its
  `.bin` and `_base_color.png` and returns "is a Git LFS pointer" before any `load()`. Proven locally by
  swapping Alice's `.bin` for its pointer: one `WARNING: cast: character 'alice-moreau' drawn as the town
  figure: … is a Git LFS pointer`, no error. No CI file is changed.
- **D-4 Clip speeds measured, not estimated.** `tools/measure_stride.gd` gains `-- --character <id>`
  (builds the body exactly as the slice does). It reproduces the town body's 1.063 / 2.660 m/s, then
  measured the ten (walk / jog m/s): Alice 1.016 / 2.573, Bob 1.100 / 2.758, Carol 0.983 / 2.479, Dev
  1.041 / 2.649, Erin 0.986 / 2.545, Felix 1.097 / 2.771, Grace 0.965 / 2.435, Hana 1.068 / 2.685, Ivan
  1.015 / 2.563, Otto 1.045 / 2.652. Each manifest records `measured_by`.
- **D-5 2D renders (step-26 §9 method amended by the operator, 2026-10-19).** The 2D sprites are rendered
  in Blender from the same 3D characters instead of generated with `gpt-image` from the portrait. The
  renderer is committed as `tools/character_pipeline/blender/render_sprites.py`. Vera's set was rendered by
  it from `meshy_d.glb` at 1.767 m (113 px). `look:<key>` roles for the eleven Persons point at
  `cast_<id>`; Wes and the lakeside looks keep their #287 stand-ins. Sidecars carry `source_type: rendered`,
  recorded as an `ARC-9` note. Known visual mismatch (PROPOSAL.md decision 2): realistic renders beside
  storybook `gen_*` stand-ins; accepted by the ruling.
- **D-6 Import settings.** Each `<id>.gltf.import` is `meshy_d.glb.import`'s parameters (bone map
  `vitruvian_bonemap.tres` on `PATH:mixamo_vitruvian/Skeleton3D`, rest fixer, silhouette fix) — the
  settings the pose sheet uses; every cast glTF names its armature `mixamo_vitruvian`. The generated
  `_base_color.png.import` files are committed (plain text, not LFS).
- **D-7 Evidence modes.** `./mineworld-slice --world --cast` (`SliceProbeWorld._cast_frames`) frames each
  perceived person and the room and prints look → character → scene; `./mineworld-2d --drive=cast
  --capture` takes a wide and a close still where the seat starts.

## 4. Acceptance and evidence

| Item | Result | Evidence |
| --- | --- | --- |
| `clients/shared/checks/identity.sh` (social-cafe, market-town) | PASS | "PASS social-cafe: 23 identity lines agree … PASS market-town: 23 …" |
| `scripts/check_cast.py`, `--self-test` | PASS | "11 looks, 0 extras, default vera-lindgren: valid"; "14 cases, all as expected" |
| 3D connected, seat `visitor` | PASS | Alice → `alice-moreau.gltf`, Bob → `bob-achterberg.gltf`, Wes → town figure, player wears `meshy_d.glb`; `mineworld-demos/casting/client-shots/3d_seat-visitor_*.png` |
| 3D connected, seat `bob` (player as a cast body) | PASS | "wearing bob-achterberg", Vera drawn as NPC on `meshy_d.glb`; `3d_seat-bob_*.png` |
| 2D connected, seats `bob` and `visitor` | PASS (`drive complete: PASS`) | `2d_seat-*_cast_{wide,close}.png`: Vera, Bob, Alice as renders, Wes as stand-in |
| LFS pointer degrades | PASS | D-3 |
| `ruff check` / `format --check` on `tools/character_pipeline` | PASS | "All checks passed!", "42 files already formatted" |
| `check_decision_ids.py --write-index` | PASS | "178 decision ids, all distinct; index current" |
| `check_client_rules.py` | FAIL, pre-existing | its one finding is `clients/2d/scripts/harness/identity_check.gd:54` (R5), added by #287, untouched here |
| Vera unchanged (I-3) | by construction; not pixel-diffed | §3 D-2; AC-S26-6's `--portrait` diff belongs to the PR that moves her constants |

Not run locally (R-LOAD-1: CI on the exact head is the gate): the full Rust workspace and the Godot test
suites.

## 5. Findings

- F-1 The slice draws only the café block; at start only Alice, Bob, Wes and the visitor are in it, so the
  3D evidence shows three cast characters at once (two NPCs plus the player). The rest are bound and resolve
  (identity check), and are drawn when they enter.
- F-2 `./mineworld-2d --drive=walk` fails on `social-cafe` (one street door; the scenario wants a second)
  — a scenario limit, not this PR; `--drive=cast` was added instead.
- F-3 The labels stand at 2.05 m for every figure; on the shorter cast (Grace 1.54 m) the gap is larger.
  Cosmetic; left.

# PR S33-c — 3D world selection and the per-world depiction seam, bound by tag (16f's seed)

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        revision 1 (2026-10-17), FROZEN 2026-10-18
Author:                 S33-a/b/c PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s33abc, branch docs/pr-designs-s33abc,
                        planning base origin/main @ da80316f. Designed against PR #245 (16c) head 36325625,
                        PR #220 (RL-e0) head e765df47 and PR #254 (12e, stacked on 16c) head ed0aa6bb
Approved by / evidence: Q-S33-3 (PK-2), Q-S33-4 (S33-c is 16f's seed) and Q-S33-10 bind and are not
                        re-asked; §14's Q-S33c-1 ruled 2026-10-18 by the operator in the primary session, as
                        recommended
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of 16c, RL-e0,
                        12e, S33-a, S33-b, S29-a
Start condition:        (1) Milestone F's checklist handed to the operator (Q-S33-10); (2) 16c merged (#245:
                        OPEN at 2026-10-18); (3) RL-e0 merged (met: #220, merge 621ea0e2); (4) Q-S33c-1 ruled
                        (met: 2026-10-18); (5) this design frozen (met: 2026-10-18); (6) the primary session's
                        dispatch. 12e (#254) is not required: whichever of 12e and S33-c merges second adapts
                        (R-12e-c-1)
Execution contract:     §12
Implementer:            Opus-class (step-33 §12.1: "Opus (architecture; 16f's seed)"); reviewer the primary
                        session, which reviews the seam as 16f's first implementation (R-S33-6)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) (APPROVED
2026-10-15) §§1 (outcomes 1, 6, 8), 2.1 (A-1, A-2, A-7, A-14), 2.3 (F-S33-1), 4.8 (`ARC-144`, SD-S33-6), 8.2
(PK-2, the refusal), 9.1, 9.3 (`SHOWN` parity), 9.4 (X-6), 10 (I-S33-1, I-S33-2, I-S33-3), 12.1 (the S33-c row),
14 (Q-S33-3, Q-S33-4), 15 (`ARC-144`, `ARC-146`), 16.4 · **Parent:** [`overall.md`](overall.md) (16f placed in
MVP-0 with S33-c as its seed) · **Consumes:** 16c (`step-15-demo-3d.md`'s 16c design; PR #245: `SliceLayout`,
binding by doorway coincidence, `SHOWN`), RL-e0 (the far-field loader and view; SD-FF-7, SD-FF-8),
[`pr-s33b-lakeside-bodies.md`](pr-s33b-lakeside-bodies.md) (the doorway points and footprints the stub draws) ·
**Followed by:** S33-d…g (Lakeside's real depiction behind this seam), S33-j (the per-world export file), 16f
(depictions from a Presentation Pack, walls from disclosure) · **Binding parents:** `CLAUDE.md` §§1, 2, 3, 4
(items 3, 5, 6, 7, 11, 13, 14, 15); `docs/ENGINEERING_RULES.md` §§2–3, 5–6, 10–12; `docs/MODULE_SPEC.md` §6;
`docs/DECISIONS.md` `ARC-45`, `ARC-102`…`ARC-104`.

**Decision ids.** `ARC-144` (the seam) and `ARC-146` (its first half: the per-world export file's name and the
`PRESENTATION_WORLD_MISSING` refusal; S33-j adds the export), written by C1. No other id.

No model is called by this design.

---

## 0. Binding inputs, exactly restated

- **Scope (step-33 §12.1, S33-c row).** "`--world=lakeside`, `--world-key` in 3D; the world registry; `SliceWorld`
  refactored behind the interface with **no visual change**; a Lakeside depiction stub (flat ground, boxes at the
  places' footprints) bound by tag; `SHOWN` on Lakeside; probe/perf viewpoints per world." Depends on **16c
  merged**; RL-e0 merged.
- **Integration checkpoint.** "social-cafe shots pixel-equal (I-S33-2); `--world=lakeside --link` walks the stub
  with server place changes."
- **Adversarial.** "X-6; a world key with no depiction refuses `PRESENTATION_WORLD_MISSING`."
- **The seam (step §4.8, SD-S33-6).** "The client gains a registry from **world key** to a **world depiction**: a
  builder module with one interface … `build(root) · place_volumes() · depictions(): [{tag, volume, door}] ·
  viewpoints() · sit_points()`. Binding is by **tag** and doorway coincidence (16c's D-16c-3 rule generalised) …
  The client keeps **no list of the world's place keys** (16c SC-4); a key scan holds it to that (X-6)."
- **Same path (step §4.8).** "`./mineworld-slice --world=lakeside` hosts `mineworld server worlds/lakeside` exactly
  as `--world` hosts `social-cafe`; `--server=host:port --invite=…` joins any server and requires
  `--world-key=lakeside`. The client sends only `ActionIntent`s."
- **PK-2 (Q-S33-3, ruled).** "`3d-lakeside.pck` … loaded at start with `ProjectSettings.load_resource_pack` when
  `--world-key=lakeside`"; "a client asked to present `lakeside` without `3d-lakeside.pck` refuses by name
  (`PRESENTATION_WORLD_MISSING { world: lakeside }`, exit 3 in a scripted mode) … There is no fallback
  depiction."
- **Q-S33-4 (ruled).** "S33-c builds the per-world seam as 16f's seed."
- **R-LOAD-1.** Locally: Godot probes one at a time (`./mineworld-slice … --link`, `--layout`, `--shots`), the
  acceptance scan (`cargo test -p mineworld-acceptance --test client_rules`), `cargo build -p mineworld-cli`.
  Never `cargo test --workspace`. Local timings INCONCLUSIVE; CI on the exact head is the gate.

---

## 1. Goal

The 3D client presents **any** world it has a depiction for, through one seam: `./mineworld-slice --world=<key>`
hosts `worlds/<key>` on the same server path, `--world-key=<key>` selects what the client draws, and a registry
maps the key to a **world depiction** — a builder with one interface that says what it draws, which tagged
places each drawn room or street depicts, where its viewpoints and seats are. Social Café becomes the first
depiction with no visual or behavioural change; a Lakeside **stub** depiction (flat ground and boxes at the
places' footprints) becomes the second, bound purely by tag and doorway coincidence. A key without a depiction is
refused by name. The client names no place key anywhere, and a scan proves it. This is 16f's first
implementation: 16f later loads depictions from a Presentation Pack and builds walls from disclosure.

---

## 2. Scope

### 2.1 In scope

1. `clients/3d-spike/scripts/worlds/depiction.gd` (new, `class_name SliceDepiction`): the interface (§5.2) and
   `static func tag(name: String) -> String` (the one place a tag literal is written, §5.6).
2. `clients/3d-spike/scripts/worlds/registry.gd` (new, `class_name SliceWorlds`): the key from the arguments, the
   lookup, the per-world resource pack, the refusal (§5.3).
3. `clients/3d-spike/scripts/worlds/social-cafe/depiction.gd` (new): Social Café behind the interface, wrapping
   `SliceWorld` and carrying its spawn, its viewpoints and its door table (§5.4).
4. `clients/3d-spike/scripts/worlds/lakeside/depiction.gd` (new): the Lakeside stub (§5.5).
5. Edits: `slice_main.gd` (key, registry, depiction instead of `SliceWorld.build` and `WORLD_KEY`),
   `slice_layout.gd` (anchor and door table from the depiction, binding by tag set), `slice_world.gd`
   (`SPAWN`, `ANCHOR_TAG` move to the Social Café depiction), `slice_link.gd` (the world key; nothing else),
   `slice_perf.gd` and `slice_far_field_check.gd` (views and viewpoints from the depiction; shots under
   `res://shots/<key>/…`), `slice_probe_world.gd` / `slice_probe_town.gd` (the link route from the depiction's
   probe data; `SHOWN` for any world).
6. `mineworld-slice` (the launcher): `--world=<key>`, `--world-key` forwarding, the per-world hosting line (§5.7).
7. `tests/acceptance/tests/client_rules.rs`: check (d), place-key literals (§5.6).
8. `scripts/ci_layer.py`: `clients-probes` gains `slice --world=lakeside --link`.
9. `docs/MODULE_SPEC.md` §6: per-world export files of one Presentation Pack and the refusal (step §16.4).
   `docs/DECISIONS.md`: `ARC-144`, `ARC-146`.

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| Lakeside's real terrain, water, buildings, woods, perf views | S33-d |
| The pier and the halt in 3D; the full route to the platform | S33-e |
| Sit points' values and the shared sit rule | S33-f (`sit_points()` exists and returns `[]` here) |
| Splitting the export into `3d.pck` and `3d-lakeside.pck` (`scripts/package.py` is not on `main`) | S33-j (on R-a) |
| Depictions read from a Presentation Pack; walls built from disclosure; multi-pack loading | 16f |
| The 2D client's place-name keys (`places.gd:79–80`) | S33-h (the scan covers the 3D client and `clients/shared/` only) |
| Any server, protocol, kernel, contract or System change | never (I-S33-1) |

### 2.3 Dependencies

**Required before C0:** 16c merged (`SliceLayout` and the door registry this PR generalises), RL-e0 merged (the
far-field loader the key now selects). Not required, coordinated: 12e (#254, stacked on 16c; it adds
`client_bodies.gd` and two probes to the chain — R-12e-c-1); S33-b (the stub's doors follow its points;
R-S33c-b-1); S29-a (the halt's entries bind only if its places exist; otherwise they are reported as misses,
never invented).

---

## 3. Audit (`origin/main @ da80316f`; 16c `@ 36325625`; RL-e0 `@ e765df47`; 12e `@ ed0aa6bb`)

### 3.1 Source anchors (paths under `clients/3d-spike/` unless stated)

| # | Fact | Where |
| --- | --- | --- |
| A-1 | No GDScript reads `--world` or `--world-key` on any ref; `--world` is a launcher flag only | `scripts/slice/*.gd` (all refs) |
| A-2 | `_ready` order on main: settings, GI, environment, sun, `world = SliceWorld.build(self)` (:62), GI, reflections, `_spawn_player`, HUD, `_link`; the probe chosen at :70; 16c: `SliceWorld.build(self, SliceLink.address_from_args() != "")` (:63) and the probe chain Town → World → base (:71–76); RL-e0: `if not _load_far_field(): return` (:59), `SliceWorld.build(self, far_field)` (:64) | `scripts/slice/slice_main.gd` |
| A-3 | RL-e0 hard-codes the key: `const WORLD_KEY := "social-cafe"` (:80), `const PACK_DIR := "../../presentation/mineworld-default/3D"` (:83); `_load_far_field()` (:91–113) calls `MineWorldFarField.load_for(dir, WORLD_KEY, "3d")`, prints `far_field.refusal_line("slice")`, shows a full-screen label and in a scripted run `quit(3)` (:112). SD-FF-7: "The 3D slice **always** uses `social-cafe`" | RL-e0 `slice_main.gd`; RL-e0 Doc:545–556 |
| A-4 | `MineWorldFarField.load_for(pack_dir, world_key, dimension)` (:50–62) refuses an empty key `FAR_FIELD_WORLD_UNKNOWN` ("start the client with --world-key=<key>", :53); `refusal_line(client)` (:77–78) → `"[mineworld-%s] %s: %s: %s"` | RL-e0 `clients/shared/environment/far_field.gd` |
| A-5 | Spawn: `player.place(SliceWorld.SPAWN, SliceWorld.SPAWN_YAW, -2.0)` (main :455; 16c :461; RL-e0 :503); `SPAWN := Vector3(-11.5, 0.45, -5.55)`, `SPAWN_YAW := -78.0` | `slice_world.gd:27–28` (main) |
| A-6 | `SliceWorld` (16c): `CAFE_PLACE`, `STREET_PLACE`, `FLORIST_PLACE` (:22, :23, :25), `ANCHOR_TAG := "cafe"` (:29), `build(parent, connected)` (:40), `_place_volumes` (:201), `_place` (:213), `place_at` (:232) | 16c `slice_world.gd` |
| A-7 | `SliceLayout` (16c, new): `ROOM`/`DOOR`/`STREET` (:35–37), `DOOR_INSET := 0.40` (:44, "the authoring rule 12d adopted for every doorway"), `DOOR_MATCH := 0.15` (:49); state `passages`, `tags`, `origins`, `drawn_as`, `volumes`, `doors`, `misses`, `anchor`, `street` (:53–70); `learn` (:75) anchors when `tags[place].has(SliceWorld.ANCHOR_TAG)` (:82); `is_drawn` :89, `place_of_volume` :94, `to_scene` :104, `to_world` :111, `out_point` :118, `in_point` :124, `shown` :142–161, `_bind_anchor` :184, `_bind_street_doorways` :201, `_bind` :236, `_door_of_volume` :246 | 16c `scripts/slice/slice_layout.gd` |
| A-8 | The drawn-door registry is `static var doors` in `SliceTerrace` (`terrace.gd:37`), appended at `terrace.gd:159, 246` and `cafe.gd:255`; each door carries a `volume` (`terrace.gd:65`) | 16c |
| A-9 | `SliceLink` (16c) drops `KEYS`, `PLACE_KEY`, `door_point`, `origin` (comment :43–50), owns `layout := SliceLayout.new()` (:74), learns per observation (:244), gates people on `layout.is_drawn` (:264), delegates `to_scene`/`to_world` (:339–346), maps the player's volume through `layout.place_of_volume` (:357–358), `shown()` (:492–504). `--server=`, `--seat=` (default "visitor"), `--invite=`, `--nickname=` parsers (:99–127) | 16c `slice_link.gd` |
| A-10 | `SHOWN` in 3D: `slice_probe_world.gd:976–981` `_print_shown` → `print("SHOWN ", JSON.stringify(report))`; `slice_probe_town.gd` L-3 `_check_shown` (:256–261), S-5 (:670–691), `SHOWN_MM := 50` (:31). In 2D: `clients/2d/scripts/harness/drive.gd:440–457` | 16c; main |
| A-11 | Viewpoints are social-cafe constants: `SlicePerf.VIEWS` (main :33–41, eight `[name, pos, yaw, pitch]`), `BREAKDOWN_VIEWS` (:45); `SliceFarFieldCheck.VIEWPOINTS` (RL-e0 :37–45, seven), `AZIMUTHS` (:34), `SHOTS_DIR "res://shots/social-cafe/farfield"` (:48), `SHOTS` (:50–58) | main `slice_perf.gd`; RL-e0 `slice_far_field_check.gd` |
| A-12 | Probe modes: `SliceProbe.MODES` (main :112–113) + RL-e0's `farfield`, `farfield-shots` (:112–113); `SliceProbeWorld.WORLD_MODES` (`link`, `conversation`, `target`, `settings`, `controls`; :19); 16c's `SliceProbeTown.TOWN_MODES` (`layout`, `street`; :21); verdicts "all <mode> checks pass" / "<n> <MODE> CHECKS FAILED"; `ci_godot.py judge_probe` (main :203–211) needs exactly one pass line and no `FAIL` | main; 16c; RL-e0 |
| A-13 | The launcher `mineworld-slice` (main): `--world` sets `world=1` and is not forwarded (:92–105); hosting `"$here/target/debug/mineworld" server "$here/worlds/social-cafe" --listen 127.0.0.1:0 --agent alice` (:121–122), then `--server=$address --invite=$invite` appended (:138); 16c adds `--street`, `--layout`, `--town`, `--save` (:107–146, server still `worlds/social-cafe` :146); RL-e0 adds `--farfield` → `--slice-farfield` with `--gi=none` (:109), server at :136 | `mineworld-slice` |
| A-14 | **No place-key scan exists.** 16c's SC-4 ("the client keeps no list of the world's place keys") is checked by review (16c-C2). The only scanner, `tests/acceptance/tests/client_rules.rs`, scans every `clients/**/*.gd` (except `clients/*/tools/`, `protocol/demo`, `protocol/checks`; :17–22) for action-name literals (`ACTION_LITERALS` :40–107), rule-named constants (`RULE_WORDS` :113) and `worlds/`/`systems/` path literals (:8–15) | main = 16c |
| A-15 | **No resource-pack loading exists** (`load_resource_pack`, `.pck`) on main, 16c, RL-e0 or 12e; `scripts/package.py` exists only on the paused `origin/mvp0/pr-s23-ra` (its 3D export is Godot's `--export-release`, `runtime/clients/mineworld-3d.pck`) | all refs; `pr-s23-ra:scripts/package.py:96–97, 255–277` |
| A-16 | 12e (stacked on 16c): `client_bodies.gd` (`SliceClientBodies`; `observe(obs, layout, world_root)` :42, `static build(where, shape, passages, layout)` :67), used by `slice_link.gd` (12e :94, :281, :284); probe chain extended (12e `slice_main.gd:71–77`) | 12e |
| A-17 | CI: `clients-probes` (main `ci_layer.py:174–189`) runs `slice --drive`, `slice --world --link`, `slice --world --target` and the protocol runs; RL-e0 adds the per-PR Linux `farfield` job (`ci.yml:316–337`, layer `farfield` = `client_far_field` ignored tests, path filter `clients/`, `presentation/`) | main; RL-e0 |
| A-18 | `project.godot`'s main scene is `res://scenes/main.tscn` (the promenade); the slice is `res://scenes/slice.tscn` (one Node3D with `slice_main.gd`); no autoloads | `project.godot:10`; `scenes/slice.tscn:1–6` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33c-1** | 16c's binding is café-shaped: it anchors only on a place tagged `cafe` "once the observer has stood in it" (A-7 :82) and binds the rest through the café door's street. Lakeside's observer may start in any of seven places (Elin in `cottages`, Nils on `pier`) | The anchor generalises (§5.2): any depiction entry may anchor, either by its door's in-point or by a declared frame origin; the street binds from whichever anchored place opens onto it; later places by doorway coincidence, unchanged |
| **F-S33c-2** | SC-4 has no mechanical check (A-14). X-6 needs one | Check (d) in `client_rules.rs` (§5.6), with the one exemption `SliceDepiction.tag("…")` and a reviewed allow-list (like `ACTION_LITERALS`) for literals that are not place keys in use |
| **F-S33c-3** | Nothing loads a resource pack, and the per-world export does not exist yet (A-15). The refusal must still be real now, and must not need changing when S33-j splits the export | The registry looks up a depiction by convention (`res://scripts/worlds/<key>/depiction.gd`); if absent, it loads `3d-<key>.pck` from beside the executable when that file exists, then looks again; if still absent, it refuses. In the editor and the dev tree both depictions are present; S33-j only moves Lakeside's files into its own export (§5.3) |
| **F-S33c-4** | The launcher's hosting line names `--agent alice` (A-13), a Social Café person; Lakeside has none, and its player seat for the hands-on walk is Elin (AC-S33-11) | §5.7: the bare `--world` keeps today's line byte-for-byte; `--world=<key>` hosts without `--agent` and passes `--seat=` from the user or, for Lakeside, `elin`. C0 confirms what `--agent` does and records it |
| **F-S33c-5** | The viewpoints, the far-field shots directory, the spawn and the probe route are Social Café constants spread over five files (A-5, A-11) | They move into the Social Café depiction, values unchanged (§5.4); each consumer reads the current depiction |
| **F-S33c-6** | RL-e0's 3D far-field check and its per-PR CI job run Social Café only (A-17); Lakeside's AC-S33-1 needs the real near field (S33-d) | S33-c makes `--world-key=lakeside --farfield` runnable on the stub and records one run if S33-a has merged; it adds no CI case (S33-d adds Lakeside's to `client_far_field.rs`) |

### 3.3 What does not exist

No world registry, no depiction interface, no 3D `--world-key`, no resource-pack loading, no place-key scan, no
3D depiction of any world but Social Café.

---

## 4. Reuse, both ways

| Need | Decision | Why |
| --- | --- | --- |
| Loading a world's assets at start | **Godot's `ProjectSettings.load_resource_pack`** (engine built-in) | the engine's mechanism for exactly this; declined: GDExtension plugins or a custom archive format |
| Selecting a builder by key | a path convention plus `ResourceLoader.exists` / `load` | no registry file to keep in sync; a world is present iff its depiction script is; declined: a hand-written key → script table (it is a list of world keys in the client, the seed of the place-key list SC-4 forbids, and it would need editing per world) |
| The interface | a GDScript base class with documented virtual methods | Godot's idiom; no plugin framework |
| The key scan | an addition to the existing `client_rules.rs` scanner | one scanner, one allow-list format |

---

## 5. Design

### 5.1 The seam in one picture

```text
mineworld-slice --world=<key>          host worlds/<key>; pass --world-key=<key> and --server/--invite
        │
slice_main.gd ── SliceWorlds.key()     --world-key=…, or "social-cafe" offline with no --server
        │        SliceWorlds.open(key) → SliceDepiction  | refuse PRESENTATION_WORLD_MISSING (exit 3)
        │        MineWorldFarField.load_for(pack, key, "3d")                 (RL-e0, unchanged)
        │        depiction.build(root, far_field, connected) → the world node
        │        depiction.spawn(), .viewpoints(), .sit_points()
        ▼
SliceLink ── SliceLayout(depiction)    binds disclosed places to depiction entries by tag set + doorway
                                        coincidence; never by key
```

### 5.2 The interface (`SliceDepiction`, `scripts/worlds/depiction.gd`)

```text
class_name SliceDepiction extends RefCounted

# Built once. `connected` is 16c's flag (a hosted or joined server is present).
func build(root: Node3D, far_field: MineWorldFarField, connected: bool) -> Node3D
# Every scene volume (Area3D with meta place_id = the volume's name) this depiction draws.
func place_volumes() -> Array[Area3D]
# What is drawn as which tagged place:
#   { tags: PackedStringArray  (all must be among the place's disclosed tags; written with tag())
#     volume: String           (the Area3D's place_id; "" for a door-only place)
#     drawn_as: ROOM | DOOR | STREET
#     door: int                (index into doors(), or -1)
#     origin: Variant          (null, or the scene position of the place frame's (0, 0) — for an anchor
#                               that has no door, such as a street) }
func depictions() -> Array[Dictionary]
# The drawn doors, in 16c's door format (SliceTerrace.doors' entries).
func doors() -> Array
# Offline spawn: { position: Vector3, yaw: float }. Connected runs spawn by disclosure (16c, unchanged).
func spawn() -> Dictionary
# { farfield: [[name, pos, yaw]], farfield_shots: [...], perf: [[name, pos, yaw, pitch]],
#   perf_breakdown: [...], landmark: [...], probe_route: [ PackedStringArray of tags, … ] }
func viewpoints() -> Dictionary
# Sit points (presentation data, step §4.5); [] until S33-f.
func sit_points() -> Array
# The one place a tag literal is written (§5.6).
static func tag(name: String) -> String: return name
```

**Binding rule (generalises 16c's D-16c-3).** For a disclosed place P with tags T:

1. The candidate entries are those whose `tags ⊆ T`. The one with the most tags wins; a tie is a miss
   (`why: "ambiguous: <entries>"`), never a guess.
2. **Anchor.** While nothing is bound, the first observed place with a candidate entry anchors the scene: its
   origin is the entry's `origin` if given, else `in_point(door) − to_3d(P's passage here)` (16c's
   `_bind_anchor`). 
3. **Street.** The place an anchored place's door opens onto is bound to the entry drawn as `STREET` whose tags
   it carries, its origin from the passage's `there` (16c's rule).
4. **The rest.** Every passage from a bound place binds its target to the candidate entry whose door's out-point
   lies within `DOOR_MATCH` (0.15 m) of the passage's scene point (16c's `_bind_street_doorways`/`_bind`,
   unchanged). No match → a miss, reported in `SHOWN` (unchanged).

Social Café's entries reproduce 16c's results exactly (C2's equality gate).

### 5.3 The registry (`SliceWorlds`, `scripts/worlds/registry.gd`)

- `static func key() -> String`: `--world-key=<k>` if given; else, with no `--server=`, `"social-cafe"` (the slice
  offline, as today); else `""`.
- `static func open(key) -> SliceDepiction`: an empty key → refuse `FAR_FIELD_WORLD_UNKNOWN` through RL-e0's
  loader (unchanged message). Else `path = "res://scripts/worlds/%s/depiction.gd" % key`; if
  `not ResourceLoader.exists(path)`: `pck = OS.get_executable_path().get_base_dir().path_join("3d-%s.pck" %
  key)`; if `FileAccess.file_exists(pck)`: `ProjectSettings.load_resource_pack(pck)`; look again. Still absent →
  **refuse** `PRESENTATION_WORLD_MISSING`: `[mineworld-slice] PRESENTATION_WORLD_MISSING: { world: <key> }:
  this client has no depiction for the world; install its 3D presentation file 3d-<key>.pck`, the full-screen
  label, `quit(3)` in a scripted run (RL-e0's pattern, A-3). There is no fallback depiction.
- The depiction is checked **before** the far field, so a missing depiction is named as such.

### 5.4 Social Café behind the interface (no visual change)

`scripts/worlds/social-cafe/depiction.gd` delegates `build` to `SliceWorld.build(root, connected)` (with RL-e0's
far-field argument as merged), `place_volumes` to its volumes, `doors()` to `SliceTerrace.doors`, and returns
`spawn()` = A-5's values, `viewpoints()` = A-11's tables **moved verbatim** (perf views, breakdown views,
far-field viewpoints and shots, the 16c probe route), and `depictions()` = the café room (`tag("cafe")`, ROOM,
the café door), the florist (DOOR or ROOM as 16c draws it), the street (`tag("street")`, STREET) and the
remaining 16c door-bound entries, each written so §5.2's rule yields 16c's bindings. `ANCHOR_TAG` and `SPAWN`
leave `SliceWorld`.

### 5.5 The Lakeside stub (`scripts/worlds/lakeside/depiction.gd`)

- **Ground:** RL-e0's far-field view for `lakeside` (`SliceFarFieldView.build`), plus a flat 200 × 200 m disc at
  y 0 in `Mats.pbr("leafy_grass", …)`. No terrain, no water mesh, no buildings beyond boxes: S33-d replaces all of
  it.
- **Boxes:** one box per place at S33-b's lane-frame footprint (`pr-s33b-lakeside-bodies.md` §5.3, translated to
  the scene by step §4.0's frame: lane (x, y) mm → scene (x/1000 − 15, 0, 3 − y/1000)), walls 2.4 m high, 0.2 m
  thick, a gap at each door; one `Area3D` volume per box; colliders on `LAYER_WORLD` for the walls (12e's
  disclosed geometry, if merged, adds its invisible walls as for Social Café). One door table (`DOORS`), in 16c's
  door format, at S33-b's doorway points (R-S33c-b-1).
- **Entries:** `tag("street")` STREET with `origin` = the lane frame's (0, 0) at scene (−15, 0, 3); `cottages`,
  `boathouse`, `bakery`, `square`, `shore`, `pier` (ROOM or DOOR boxes, each with its door); the halt's three as
  `[tag("station")]`, `[tag("station"), tag("ticket-office")]`, `[tag("station"), tag("platform")]` when S29-a's
  places exist, else omitted (they appear as misses, never invented).
- **Viewpoints:** far field VP-L-1…7 at step §9.1's provisional values (S33-a and S33-d fix them); perf `[]`
  (S33-d's eight views); probe route `lane → bakery → lane → pier → lane` by tags; `spawn()` = the cottages yard's
  centre (offline only).
- A HUD line names it a stub (Q-S33c-1).

### 5.6 The place-key scan (X-6; `client_rules.rs` check (d))

- **Keys:** the stems of every `worlds/*/places/*.yaml` in the checkout.
- **Files:** `clients/3d-spike/scripts/**/*.gd` and `clients/shared/**/*.gd` (A-14's exclusions kept).
- **Flagged:** a string literal (`"…"` or `'…'`, after stripping comments) whose content equals a key.
- **Exempt:** the argument of `SliceDepiction.tag(…)` (or `tag(…)` inside a depiction file); and an allow-list
  `PLACE_KEY_LITERALS: [(file, literal, reason)]` in the test, each reviewed — for a literal that is not used as a
  place key (C0 fills it from a report-mode run; e.g. 16c's `const STREET := "street"`, the `drawn_as`
  vocabulary of D-16c-7). An allow-list entry for a literal compared with a place's key is forbidden (review).
- **Message:** the file, line, literal and "the 3D client keeps no list of a world's place keys (SC-4,
  I-S33-3): bind by tag with `SliceDepiction.tag()`".

### 5.7 The launcher (`mineworld-slice`)

| Flag | Behaviour |
| --- | --- |
| `--world` (bare) | unchanged, byte for byte: hosts `worlds/social-cafe … --agent alice`; adds `--world-key=social-cafe` to the Godot arguments |
| `--world=<key>` | hosts `"$here/target/debug/mineworld" server "$here/worlds/<key>" --listen 127.0.0.1:0 --packs "$here/entities" --packs "$here/presentation/mineworld-default"` (the roots every Lakeside command names, `world.yaml:15`); appends `--server`, `--invite`, `--world-key=<key>`, and `--seat=elin` for `lakeside` unless `--seat=` was given (F-S33c-4); refuses a key with no `worlds/<key>/world.yaml` by name before starting anything |
| `--world-key=<key>` | forwarded verbatim (a join with `--server=`) |
| 16c's `--street`, `--layout`, `--save`, `--town`; RL-e0's `--farfield` | unchanged; they apply to the selected world |

### 5.8 Records and spec text

- `ARC-144`: the 3D client selects a world by key and draws it through one depiction interface (§5.2), bound by
  tag set and doorway coincidence; the client keeps no place key (check (d)); 16f generalises the seam to
  depictions from a Presentation Pack (Q-S33-4).
- `ARC-146` (this PR's half): one Presentation Pack may ship per-world export files named `3d-<world-key>.pck`;
  the client loads one at start when it lacks that world's depiction, and refuses `PRESENTATION_WORLD_MISSING
  { world }` (exit 3 scripted) when none is found; there is no fallback depiction (Q-S33-3). S33-j adds the export.
- `MODULE_SPEC.md` §6 (after §6.1's directory list): two sentences stating the per-world export file and the
  refusal, linking `ARC-146`.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-c-1 (I-S33-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/` changes | C7 diff scan |
| I-c-2 (I-S33-2) | Social Café's shots are pixel-equal within `frame_diff.gd`'s 8/255; its `SHOWN`, `--layout` and probe verdicts are equal to the base's | AC-c-1, AC-c-2 |
| I-c-3 (I-S33-3) | No place-key literal in the 3D client or `clients/shared/` outside `tag()` and the reviewed allow-list | check (d) |
| I-c-4 | A key with no depiction is refused by name; nothing draws a fallback | AC-c-5 |
| I-c-5 | The client sends only `ActionIntent`s; connected Lakeside uses the same server path as Social Café | AC-c-4 (the server's place changes) |
| I-c-6 | Binding never guesses: an ambiguous or unmatched place is a reported miss | AC-c-6 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static scan | I-c-3 (check (d)) and the existing client rules | `cargo test -p mineworld-acceptance --test client_rules` (CI `core`) |
| Rendered parity | I-c-2: Social Café shots before/after by `frame_diff.gd`; far-field shots | local Godot, base captured at C1; CI nightly `clients` |
| Probe (Godot, real server) | `--world --link` and `--world --target` (Social Café, unchanged verdicts); `--world=lakeside --link` (the stub: spawn by disclosure, walk the probe route, the server's `place` changes at each doorway, `SHOWN` printed and every disclosed village place bound); `--layout` (16c's L-checks, Social Café) | CI nightly `clients-probes` (Linux, macOS; Windows per RL-e0's ruling); local one at a time |
| Refusal | `--world-key=nowhere` scripted → `PRESENTATION_WORLD_MISSING`, exit 3; `--server=… ` with no key → `FAR_FIELD_WORLD_UNKNOWN`, exit 3 | a `#[ignore]` Godot test in `tools/cli/tests/client_far_field.rs` (RL-e0's harness): `a_world_without_a_depiction_is_refused_by_name` |
| Gate 1 / 2 | NOT REQUIRED | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-c-1 | Every Social Café shot (`--shots`, `--farfield-shots`) at the final head is within `frame_diff.gd`'s 8/255 of the C1 base capture | I-S33-2 |
| AC-c-2 | Social Café's `SHOWN` reports (`--world --link`) and `--layout` verdicts are text-equal to C1's; `--world --target`, `--drive` pass | I-S33-2 |
| AC-c-3 | `./mineworld-slice --world=lakeside --link` prints `all link checks pass`: the server's `place` changes at each doorway of the probe route, 0 refusals, `SHOWN` lists every disclosed village place bound (the halt's too if S29-a has merged) | §12.1 checkpoint |
| AC-c-4 | The same run's client sent only `ActionIntent`s (the link probe's existing check) | I-c-5 |
| AC-c-5 | X-6: planting `"bakery"` as a key in `slice_layout.gd` turns check (d) red naming the file and line; `--world-key=nowhere` refuses `PRESENTATION_WORLD_MISSING { world: nowhere }`, exit 3 | §12.1 adversarial |
| AC-c-6 | A Lakeside entry made ambiguous (two entries with the same tag set) is reported as a miss in `SHOWN`, not bound | I-c-6 |
| AC-c-7 | If S33-a has merged: `./mineworld-slice --world-key=lakeside --farfield` runs on the stub and its verdict and shots are recorded (not gated) | F-S33c-6 |
| AC-c-8 | The diff touches only §2.1's paths | I-c-1 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and evidence; inapplicable → `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s33c`. Never `cd` elsewhere. Steps 1–10 of
[`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) §9.0 apply, with these checks: `cargo test -p
mineworld-acceptance --test client_rules`; Godot probes one at a time through `./mineworld-slice`; `godot
--headless --path clients/3d-spike --script res://tools/frame_diff.gd …` for shot comparisons; doc scripts.
R-LOAD-1: never `cargo test --workspace`; no local perf timing is evidence.

#### C0 — Base, prerequisites, ledger, scan report (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s33c.md` (new).
- Steps:
  1. Record the base; confirm 16c (A-6…A-10) and RL-e0 (A-3, A-4, A-11) are on `main`; record 12e's, S33-a's,
     S33-b's and S29-a's states. 16c or RL-e0 missing → material stop.
  2. Re-read A-1…A-18 on the base; record moved lines (bounded) or changed behaviour (stop).
  3. Run §5.6's scan in report mode (a scratch copy of the test, not committed) and record every hit with the
     disposition planned for it (`tag()`, move into a depiction, or allow-list with reason).
  4. Record what the launcher's `--agent alice` does (F-S33c-4).
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: prerequisites with evidence; every scan hit has a
  disposition.
- **Commit message:** `docs(plan): S33-c — open the ledger at the base`

#### C1 — Records, spec, and the base captures

- **Files:** `docs/DECISIONS.md` (`ARC-144`, `ARC-146`), `docs/MODULE_SPEC.md` §6, this file.
- Steps: §5.8. Capture on the unmodified base, into the session scratchpad (not committed), with each command and
  the base SHA in §15: `--shots`, `--farfield-shots`, the `SHOWN` lines of `--world --link`, the `--layout`
  output.
- Validation: doc scripts; the captures exist and are listed with sizes.
- [ ] Implementation. [ ] Validation. [ ] Review: `ARC-144` says nothing 16f would have to undo (R-S33-6).
- **Commit message:** `docs: ARC-144 and ARC-146 — one depiction seam per world, refused by name when absent (S33-c)`

#### C2 — The interface and Social Café behind it (no visual change)

- **Files:** `scripts/worlds/depiction.gd`, `scripts/worlds/social-cafe/depiction.gd`,
  `scripts/slice/{slice_main.gd, slice_world.gd, slice_layout.gd, slice_link.gd, slice_perf.gd,
  slice_far_field_check.gd, slice_probe_world.gd, slice_probe_town.gd}`, this file.
- Steps:
  1. `SliceDepiction` (§5.2).
  2. The Social Café depiction (§5.4): A-5's and A-11's values moved verbatim; `depictions()` per §5.4.
  3. `SliceLayout`: takes the depiction; §5.2's binding rule replaces `ANCHOR_TAG` (A-7 :82) and the
     `SliceTerrace.doors` reads; `DOOR_INSET`, `DOOR_MATCH`, `shown` unchanged.
  4. `slice_main.gd`: still Social Café only in this commit (the depiction constructed directly); spawn,
     far-field key and the viewpoint consumers read the depiction.
- Validation: AC-c-1 and AC-c-2 against C1's captures (each shot's `frame_diff` result recorded); `--world --target`;
  `--drive`. Mutation **M-c-1** (the Social Café entry for the café room loses its door) → `--layout` fails L-1
  by name.
- [ ] Implementation. [ ] Validation. [ ] Review: every moved constant is byte-identical (diff of the values);
  no binding behaviour beyond §5.2 was added.
- **Commit message:** `refactor(3d): Social Café is the first world depiction, behind one interface (S33-c)`

#### C3 — The registry, the key and the refusal

- **Files:** `scripts/worlds/registry.gd`, `scripts/slice/slice_main.gd`, `scripts/slice/slice_link.gd`,
  `tools/cli/tests/client_far_field.rs`, this file.
- Steps: §5.3; `slice_main.gd` uses `SliceWorlds.key()` / `open()` and passes the key to RL-e0's loader (A-3's
  `WORLD_KEY` deleted); the `#[ignore]` refusal test of §7.
- Validation: the refusal test run once locally (`cargo test -p mineworld-cli --test client_far_field -- --ignored
  a_world_without_a_depiction_is_refused_by_name`); AC-c-1/AC-c-2 re-run (the default key path).
  Mutation **M-c-2** (`open` returns the Social Café depiction for an unknown key) → the refusal test red.
- [ ] Implementation. [ ] Validation. [ ] Review: the depiction is checked before the far field; no fallback path.
- **Commit message:** `feat(3d): select the world by key; a world without a depiction is refused by name (S33-c)`

#### C4 — The launcher

- **Files:** `mineworld-slice`, `scripts/ci_layer.py`, this file.
- Steps: §5.7; `clients-probes` gains `slice --world=lakeside --link`.
- Validation: `bash -n mineworld-slice`; `./mineworld-slice --world --link` (unchanged verdict, the hosting line
  printed equal to the base's); `python3 scripts/ci_layer.py --list clients-probes`.
- [ ] Implementation. [ ] Validation. [ ] Review: the bare `--world` path is byte-identical in what it runs.
- **Commit message:** `feat(launcher): --world=<key> hosts any world on the same server path (S33-c)`

#### C5 — The Lakeside stub

- **Files:** `scripts/worlds/lakeside/depiction.gd`, this file.
- Steps: §5.5, the boxes and doors from S33-b's tables as merged (or, if S33-b is unmerged, from its frozen
  design's §5.2–§5.3, recorded as R-S33c-b-1's pending update).
- Validation: AC-c-3, AC-c-4, AC-c-6 (a planted duplicate entry, reverted), AC-c-7 if S33-a has merged. Mutation
  **M-c-3** (the bakery door moved 0.3 m along the lane) → `SHOWN` reports bakery as a miss and the link probe
  fails at the bakery doorway.
- [ ] Implementation. [ ] Validation. [ ] Review: the stub contains no place key (check (d) is C6's, run here in
  report mode); nothing in it pretends to be the finished village.
- **Commit message:** `feat(3d): a Lakeside stub depiction, bound by tag (S33-c)`

#### C6 — The place-key scan (X-6)

- **Files:** `tests/acceptance/tests/client_rules.rs`, any client file C0's report listed for `tag()`, this file.
- Steps: §5.6; C0's dispositions applied.
- Validation: `cargo test -p mineworld-acceptance --test client_rules`. Mutation **X-6** (`"bakery"` as a key in
  `slice_layout.gd`) → red naming the file and line; restored.
- [ ] Implementation. [ ] Validation. [ ] Review: every allow-list entry has a reason and none is a place key in
  use; AC-c-1 re-run if any client file changed.
- **Commit message:** `test(acceptance): the 3D client names no place key (SC-4, X-6; S33-c)`

#### C7 — Close-out

- **Files:** this file (§15), `handoff-s33c.md`.
- Steps: AC-c-8's diff scan; push `mvp0/pr-s33c`; `gh pr create`; CI on the exact head (the per-PR `farfield` job
  runs, `clients/` changed); repair to green; the Social Café parity evidence and the Lakeside stub shots in the
  PR body.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S33-c close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-12e-c-1 (on 12e or this PR, whichever merges second).** `SliceClientBodies.observe(obs, layout, …)` keeps
  working with the depiction-backed `SliceLayout`; 12e's probes join the chain after this PR's registry.
- **R-S33c-b-1 (with S33-b).** The stub's door table follows S33-b's doorway points (S33-b §10).
- **R-S33d-c-1 (on S33-d).** Lakeside's real depiction replaces the stub behind the same file and interface; it
  adds Lakeside to `client_far_field.rs`'s 3D check and fills `viewpoints().perf`.
- **R-S33j-c-1 (on S33-j).** The export puts `scripts/worlds/lakeside/` and its assets into `3d-lakeside.pck` and
  removes them from `3d.pck`; the registry needs no change.
- **R-16f-c-1 (on 16f).** 16f keeps `SliceDepiction` as the boundary and changes only where depictions come from.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §4.8 | the interface as §5.2 (doors and anchors made explicit); SC-4 now a mechanical check (F-S33c-2) |
| `step-15-demo-3d.md` (16c) | SC-4's check is `client_rules.rs` (d) |
| RL-e0 design SD-FF-7 | "the 3D slice always uses `social-cafe`" → the key is selected (S33-c) |
| `overall.md` | 16f's seed landed (S33-c); `ARC-144`, `ARC-146` |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-c — 3D world selection and the per-world depiction seam (16f seed)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33c-world-seam.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md (§§1, 2, 4.8, 8.2, 9, 10, 12, 14, 15, 16.4); step-15-demo-3d.md
                         (16c: D-16c-3, D-16c-5, D-16c-7, SC-4); ../mvp1/pr-rl-e0-far-field-mandatory.md (§§5.7,
                         6; SD-FF-7, SD-FF-8); pr-s33b-lakeside-bodies.md (§§5.2–5.3); docs/MODULE_SPEC.md §6;
                         docs/ENGINEERING_RULES.md; docs/DECISIONS.md ARC-45, ARC-102…104; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33c, its own, held by one session
BRANCH:                  mvp0/pr-s33c, created from origin/main
START CONDITION:         Milestone F's checklist handed over; 16c (NOT met at 2026-10-18: #245 open) and RL-e0
                         (met: #220, 621ea0e2) merged; Q-S33c-1 ruled (met: 2026-10-18); this design frozen
                         (met: 2026-10-18); the primary's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–9
FROZEN INVARIANTS:       step-33 I-S33-1, I-S33-2, I-S33-3; §6 I-c-1 … I-c-6; Q-S33-3, Q-S33-4 as ruled;
                         16c's DOOR_INSET and DOOR_MATCH unchanged
SEQUENCE:                C0 … C7 (§9)
ALLOWED COMMANDS:        cargo build -p mineworld-cli; cargo test -p mineworld-acceptance --test client_rules;
                         cargo test -p mineworld-cli --test client_far_field -- --ignored <one test>;
                         ./mineworld-slice probe runs, one at a time; godot --headless for frame_diff; python3
                         scripts/*; bash -n; git; gh (never merge); mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace; edits
                         under kernel/, contracts/, systems/, server/, worlds/, clients/protocol/; a fallback
                         depiction; a place-key list
MATERIAL STOPS:          16c or RL-e0 missing; Social Café parity (AC-c-1/2) unattainable without a visual or
                         binding change; a needed server or protocol change; the scan needing an allow-list
                         entry for a place key in use; 12e's merge forcing an interface change beyond R-12e-c-1
PLATFORMS:               the scan on Linux, macOS, Windows (CI core); probes per the nightly's OS matrix and
                         RL-e0's per-PR Linux job
VALIDATION BUDGET:       targeted local runs as listed; CI: the PR's runs. Gate 1/2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s33c.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                  (source: the operator's freeze of this design, 2026-10-18)
                         semantic commits: authorized                   (working rules §14)
                         branch push: authorized                        (working rules §21)
                         PR creation / update: authorized               (working rules §21)
                         CI repair to review readiness: authorized      (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (§22)
POST-MERGE SYNC OWNER:   primary: step-33, step-15, RL-e0 design, overall.md; implementation session: this ledger
STOP CONDITION:          S33-c READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-c-1 | The refactor shifts a Social Café pixel or binding | C1's captures; AC-c-1/2 after C2, C3 and C6; a difference is a stop, not a re-baseline |
| R-c-2 | 16f takes another shape (R-S33-6) | the interface is small and data-shaped; the primary reviews it as 16f's seed; R-16f-c-1 |
| R-c-3 | 12e and this PR both edit `slice_link.gd` and the probe chain | R-12e-c-1; whichever is second adapts; the interface is not widened for it |
| R-c-4 | The scan flags many innocent literals (`"street"` is also report vocabulary) | the reviewed allow-list, filled from C0's report; the exemption is never for a key in use |
| R-c-5 | The stub is mistaken for the finished village | Q-S33c-1; a HUD line; S33-d replaces it |
| R-c-6 | The stub's doors drift from S33-b's points | one door table; R-S33c-b-1; M-c-3 shows the probe catches a 0.3 m drift |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33c-1 [operator — what `main` shows between S33-c and S33-d]** | From S33-c's merge until S33-d's, `./mineworld-slice --world=lakeside` shows a stub: flat ground, grey boxes at the places, the real far field. Ship it on `main` like that, or keep it behind a developer flag? | **Ship it on `main`, labelled.** A HUD line reads "Lakeside — stub depiction (S33-d builds the village)". The nightly link probe then guards the seam every night, and the release cannot pass with it (AC-S33-1, -5, -11 need S33-d…g). A developer flag would hide the seam from the checks that keep it honest. **Ruled 2026-10-18 (operator): ship it on `main`, labelled, as recommended — the annotated placeholder 3D goes onto `main`.** |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet; C1's base captures and every `frame_diff` result go here)

### Mutations

(none yet; M-c-1 … M-c-3, X-6 as §9 lists them)

### Deviations and findings

F-S33c-1 … F-S33c-6 (§3.2), recorded at design time.

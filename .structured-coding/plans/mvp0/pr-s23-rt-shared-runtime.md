# PR S23-RT — One Godot runtime for both clients in the release bundle

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended) — awaiting the operator's freeze (direction approved 2026-10-20)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-20), DRAFT
Author:                 pack-size lane (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s23-rt,
                        branch docs/pr-design-s23-rt, planning base origin/main @ fec906f2
Approved by / evidence: operator ruling 2026-10-20, relayed by the coordinator: "one shared Godot runtime
                        for both clients in the S23 package"; direction "2D export as the single runtime that
                        loads the 3D pack when launched for 3D". §2 explains why this design uses a neutral
                        host project instead (Q-RT-1)
Implementation base:    the head of mvp0/pr-s23-ra (#146) at C0, or origin/main once #146 has merged;
                        tools/launch (S23 R-c, #148) is on main
Start condition:        this design frozen; #401 (DEP-160 … DEP-162) merged or stacked below
Execution contract:     §10
Implementer:            Opus-class (engine start-up and packaging; one prototype gate, P-RT-1)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-23-release.md`](step-23-release.md) §3.2 (option (a)), §17.1 (R-a), §17.2
(R-c), §18.2 (probe P-3), §18.4 (D-Ra-1), §19.5 (D-RC-4) · **Amends:** `ARC-80` (bundle layout), step-23 D-Ra-1
and D-RC-4, [`tools/launch/LAUNCHER.md`](../../../tools/launch/LAUNCHER.md) §§3–4 · **Relates to:** `DEP-41`
(official templates only), `DEP-160` … `DEP-162` (the 3D pack's size), `ARC-78` (launchers)

---

## 0. The problem in one paragraph

A release bundle carries two full Godot exports, one per client (D-Ra-1), so it carries the engine twice. On
macOS universal one engine is a 171 MB binary, about 60 MB compressed. After `DEP-162` the macOS universal
archive is 396.4 MB against a 400 MB hard limit: one more cast member breaks it. Step-23 §3.2 preferred one
engine with two packs started by `--main-pack`; probe P-3 showed that official release templates are built with
`disable_path_overrides` and abort on `--main-pack`, and `DEP-41` declined building templates from source. This
PR reaches "one engine, two packs" a different way: the engine starts its own small pack, which then loads the
client's pack at run time with `ProjectSettings.load_resource_pack()`, a call release templates allow.

## 1. How Godot behaves, and what must be proven first

- **The call.** `ProjectSettings.load_resource_pack(path, replace_files)` mounts a `.pck` into `res://`. When
  the project is already running it also refreshes the global script-class list ("this pack may have declared
  new global classes") and reloads the UID cache (Godot `core/config/project_settings.cpp`,
  `_load_resource_pack`). So a client's `class_name` types (`Human`, `SliceMain`, …) and `uid://` references
  resolve after the load.
- **What it does not do.** It does not re-read `project.godot`. Settings the engine reads once at start-up
  stay those of the pack the engine started with. §3 sorts every setting the two clients use into "the same
  at start-up" and "applied at run time".
- **Proof gate P-RT-1 (first implementation commit, before anything else).** In a Godot 4.7.2 *release
  template* export on macOS (universal), with the real 3D client's pack: the host starts, loads
  `mineworld-3d.pck`, changes to the slice scene, and the existing headless link probe
  (`--slice-link`, verdict `all link checks pass`) passes against a bundled server. The same for the 2D pack
  (`--drive`, `drive complete: PASS`). Any failure stops the PR and goes back to the coordinator with the
  output (stop condition S-1).

## 2. Which project hosts the engine (Q-RT-1)

| Option | What the engine starts with | Cost |
| --- | --- | --- |
| (a) The 2D client's export is the runtime; for 3D it loads `mineworld-3d.pck` over its own files | the 2D client's pack and settings | the 2D project must carry the 3D client's start-up settings (Jolt, the 3D input map); the 3D pack's files replace the 2D pack's at shared paths (`res://scripts/…`, `res://scenes/…`), and a 2D script already loaded stays in memory beside them |
| **(b) A neutral host project `clients/runtime/`** (recommended) | a pack holding only `res://runtime/`: one boot scene and one script; its `project.godot` holds the settings both clients share at start-up | one small new Godot project (four files) and its export presets |

**Recommended answer: (b).** It keeps each client's project free of the other client's settings (no 3D input
map or physics engine in the 2D project), no client file is ever shadowed by another client's file, and the two
clients are treated the same way, so a third client is a row in a table. Option (a) works too and is the
fallback if the operator prefers fewer projects; the rest of this design applies to either with "host" read as
"the 2D client".

## 3. Settings: shared at start-up, or applied at run time

Every key either client's `project.godot` sets today, and where it goes. The host's `project.godot` holds the
**start-up** rows; the host's boot script applies the **run-time** rows for the client it loads, from a
settings file the packager writes (§4.3). **Import-only** keys matter to the editor's import, never to a run.

| Key (section) | 2D | 3D | Class | How the host honours it |
| --- | --- | --- | --- | --- |
| `application/config/use_custom_user_dir`, `custom_user_dir_name` | true, MineWorld | true, MineWorld | start-up | same value in the host |
| `display/window/size/viewport_width`, `viewport_height` | 1600, 900 | 1600, 900 | start-up | same value in the host |
| `rendering/renderer/rendering_method` | forward_plus | forward_plus | start-up | same value in the host |
| `physics/3d/physics_engine` | — | Jolt Physics | start-up | set in the host (2D draws no 3D physics, so it is inert there) |
| `input/*` (7 actions) | — | move_*, camera_cycle, jump, jog | start-up | copied into the host's `[input]`; identical by check (§4.4) |
| `application/config/name` | MineWorld 2D | MineWorld 3D Spike | run time | `get_window().title` |
| `application/run/main_scene` (and `.mineworld_release`) | app.tscn | main.tscn / slice.tscn | run time | the scene the host changes to: `client_<k>_scene` in `BUNDLE.toml` |
| `application/config/quit_on_go_back` | true | — | run time | `get_tree().quit_on_go_back` |
| `display/window/stretch/mode`, `aspect` | canvas_items, expand | viewport | run time | `get_window().content_scale_mode`, `content_scale_aspect`, `content_scale_size` = the viewport size |
| `rendering/textures/canvas_textures/default_texture_filter` | 1 | — | run time | `get_viewport().canvas_item_default_texture_filter` |
| `rendering/anti_aliasing/quality/msaa_2d` | 2 | — | run time | `get_viewport().msaa_2d` |
| `rendering/anti_aliasing/quality/msaa_3d` | — | 0 | run time | `get_viewport().msaa_3d` |
| `rendering/anti_aliasing/quality/screen_space_aa` | — | 1 | run time | `get_viewport().screen_space_aa` |
| `rendering/occlusion_culling/use_occlusion_culling` | — | true | run time | `get_viewport().use_occlusion_culling` |
| `rendering/textures/default_filters/anisotropic_filtering_level` | — | 4 | run time | `get_viewport().anisotropic_filtering_level` (P-RT-1 confirms the property; if absent, it becomes a start-up row) |
| `rendering/lights_and_shadows/directional_shadow/size` | — | 2048 | run time | `RenderingServer.directional_shadow_atlas_set_size(2048, true)` |
| `…/directional_shadow/soft_shadow_filter_quality`, `…/positional_shadow/soft_shadow_filter_quality` | — | 2, 2 | run time | `RenderingServer.directional_soft_shadow_filter_set_quality`, `positional_soft_shadow_filter_set_quality` |
| `rendering/environment/defaults/default_clear_color` | set | — | run time | `RenderingServer.set_default_clear_color` |
| `rendering/textures/vram_compression/import_etc2_astc` | true | — (packager adds it, D-Ra-6) | import-only | ignored |
| `config_version`, `application/config/features` | | | metadata | ignored |

**Rules.** A client may not declare `[autoload]`, `[editor_plugins]` or a GDExtension: the host cannot register
them after start-up. Neither client does today. The table lives in one place, the host's boot script, as data
(key → applier); the packager reads the same list (§4.4).

## 4. Design

### 4.1 The host project `clients/runtime/`

```text
clients/runtime/
  project.godot          the start-up rows of §3; main scene res://runtime/boot.tscn; [input] = the 3D client's
  runtime/boot.tscn      one Node
  runtime/boot.gd        reads its arguments, loads the client's pack, applies its settings, changes scene
  export_presets.cfg     the four targets of step-23 (macOS universal, Windows x86_64, Linux x86_64/arm64)
  README.md              one paragraph: what this is, that it holds no game code
```

`boot.gd`, in order:
1. Read the user arguments `--client=<key>` and `--root=<runtime folder>` (`ARC-80`'s `--root`, already passed
   by the launcher and the probe). Missing either: print `[runtime] FAIL <what>` and quit with status 2.
2. Read `<root>/BUNDLE.toml` (its `key = "value"` lines, as `package.py` reads them) for `client_<key>_pack`
   and `client_<key>_scene`. Missing: FAIL, status 2.
3. `ProjectSettings.load_resource_pack(<root>/<pack>, true)`; false: FAIL, status 2.
4. Read `<root>/<pack without .pck>.settings.json` (§4.3) and apply each key with the applier of §3; an unknown
   key: FAIL, status 2 (a packager and a host out of step are an error, not a silent default).
5. `get_tree().change_scene_to_file.call_deferred(<scene>)`.

The clients' own argument parsers ignore `--client=` (the 2D `parse()` keeps unknown `--key=value` pairs; the 3D
client reads only the flags it names). The clients are unchanged: run from a checkout, each starts as today.

### 4.2 Exports (`scripts/package.py`, R-a's `export`)

- The host: `--export-release <preset> clients/<host>` as the clients are exported today (macOS: an ad-hoc-signed
  `MineWorld.app`; elsewhere `mineworld[.exe]` beside `mineworld.pck`). Its name on disk is `mineworld-client`
  (`.app`: `MineWorld.app`), so the server's `mineworld[.exe]` keeps its name.
- Each client: `--export-pack <preset> clients/mineworld-<2d|3d>.pck` with its own presets (filters, features),
  inside the existing `etc2_astc_for` context (D-Ra-6).
- The `CLIENTS` table gains `scene` (the release entry: 2D `res://scenes/app.tscn`, 3D `res://scenes/slice.tscn`).
  The 3D client's `run/main_scene.mineworld_release` override stays for exports made outside the packager.

### 4.3 What the bundle holds (amends `ARC-80`)

```text
runtime/
  clients/MineWorld.app                (macOS)  or  clients/mineworld-client[.exe] + mineworld-client.pck
  clients/mineworld-2d.pck   clients/mineworld-2d.settings.json
  clients/mineworld-3d.pck   clients/mineworld-3d.settings.json
  BUNDLE.toml                client_2d = client_3d = the host executable; client_<k>_pack, client_<k>_scene
```

`mineworld-<k>.settings.json` holds the client's run-time rows (§3) as `{"<key>": <value>}`, values converted
from `project.godot` text by the packager (integers, booleans, strings, `Color(r, g, b, a)` as four numbers).
The client packs sit outside `MineWorld.app`, so adding or replacing one never touches the app's signature.

### 4.4 The packager's settings check (new, in `assemble` before export)

For each client, every key of its `project.godot` must be either (a) a start-up row whose value equals the
host's, (b) a run-time row the host's appliers know, or (c) import-only or metadata. Anything else fails the
packaging with `<client>/project.godot: <key>: not supported by the shared runtime (step-23 PR S23-RT §3)`. So a
client that adds a start-up-only setting, an autoload or an input action the host lacks cannot ship silently.
The check reads the host's appliers from one data table shared by the boot script and the packager (a JSON file
`clients/runtime/runtime/appliers.json`, read by both).

### 4.5 The launcher (`tools/launch`, amends `LAUNCHER.md` §§3–4 and step-23 D-RC-4)

`bundle.rs` `client()` returns the host executable for both clients (macOS: the one file in
`clients/MineWorld.app/Contents/MacOS/`); `children.rs` puts `--client=<2d|3d>` first after `--`. The three
launcher programs and their names are unchanged.

### 4.6 Probe and budget

`package.py probe` starts `client_<k>` with `--client=<k>` added; the verdict lines are unchanged. `budget`
counts the host's pack with the 2D pack's limit line (it holds two scripts). Expected macOS universal archive:
about 396 − 60 ≈ 336 MB (measured in C4, not assumed).

## 5. Acceptance

| Id | Criterion | Evidence |
| --- | --- | --- |
| A-RT-1 | P-RT-1 passes (§1) | output in the ledger |
| A-RT-2 | A macOS universal bundle holds exactly one Godot engine (one `.app` under `runtime/clients/`) and the two client packs | `find`, in the ledger |
| A-RT-3 | `package.py probe` PASSes 2D on market-town and 3D on social-cafe from the assembled bundle, and 3D on a second world (market-town, the `client_3d_market` path), headless, on macOS | probe output |
| A-RT-4 | The archive and `3d.pck` are re-measured; the archive is below 400 MB by at least the removed engine | budget output |
| A-RT-5 | The settings check fails on a planted key (a scratch copy of a client `project.godot` with `[autoload]`, and one with an unknown rendering key) and passes on the real projects | `package.py --self-test` cases |
| A-RT-6 | `cargo test -p mineworld-launch` passes with the new client path; a test asserts `--client=<k>` is the first user argument | CI `fast` + targeted |
| A-RT-7 | Windows and Linux bundles assemble and probe in the release workflow (R-b), or the leg is classified with evidence | CI run |

## 6. Commit plan

- **C0** — this document's revision marker and the implementation base recorded (Markdown only).
- **C1 — P-RT-1.** `clients/runtime/` (§4.1), exported by hand on macOS with the real packs; record the output.
  Stop on failure (S-1).
- **C2 — packager.** `CLIENTS.scene`, host export, `--export-pack` for clients, settings JSON, the check (§4.4),
  `BUNDLE.toml.in` keys, probe argument, `--self-test` cases (A-RT-5).
- **C3 — launcher.** `bundle.rs`, `children.rs`, tests, `LAUNCHER.md`.
- **C4 — measure and gate.** Assemble macOS universal, probe (A-RT-2 … A-RT-4); `ARC-80` and step-23 D-Ra-1 /
  D-RC-4 amended in place with a dated note; ledger.

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| A setting read only at start-up that §3 missed changes how a client looks | the check (§4.4) makes the list exhaustive over today's keys; C4 compares 3D slice stills from the old and new bundles (same seed, same frame) |
| A resource the client loads by path before the pack is mounted | nothing of the client runs before step 5 of §4.1; the host loads no client path |
| macOS shows "MineWorld" instead of "MineWorld 2D/3D" as the process name | accepted in step-23 §3.2 (a); the launcher's own `.app` names are unchanged |
| R-a (#146) changes `export()` again before merging | implement on #146's head; read-only coordination with its worktree (`impl-s23-ra`); rebase on its merge |

## 8. Out of scope

The 3D pack's own size (`DEP-160` … `DEP-162`), signing and notarisation (step-23 §8, ruled unsigned), and
building engine templates from source (`DEP-41`).

## 9. Questions for the freeze

- **Q-RT-1.** Host project: (b) a neutral `clients/runtime/` (recommended, §2) or (a) the 2D client's export.
- **Q-RT-2.** Process name on macOS: one "MineWorld" for both clients (recommended; step-23 §3.2 (a) already
  accepted it) or keep per-client names (not possible with one engine binary without two `.app` wrappers).

## 10. Execution contract

- **Scope:** §4 and §6 only. **Frozen invariants:** the clients' code and behaviour unchanged; `--root`,
  `--headless` and the verdict lines unchanged; official templates only (`DEP-41`).
- **Autonomous:** implement, validate, commit, push, open the PR, repair CI.
- **Stop conditions:** S-1 P-RT-1 fails; S-2 a client needs a code change to run under the host; S-3 the change
  needs a new dependency. Each goes back to the coordinator with evidence.
- **Merge:** the coordinator, after `fast` is green (charter).
- **Ledger:** append below.

## 11. Ledger

| Date | Commit | Event |
| --- | --- | --- |
| 2026-10-20 | — | revision 1 drafted |

# PR SET-b — basic input options: the Controls tab (mouse sensitivity, invert Y, read-only key map)

**Status: DRAFT (2026-10-10).** Not `DESIGN FROZEN`. Implementation does not start until the operator
freezes this design (`CLAUDE.md` §3.1). This is the single design authority for SET-b. It is linked
from `step-20-client-settings.md` §6 (SET-b row); the step file is not edited by this PR.

**Parent documents (binding):** `CLAUDE.md` §§1.1, 2–4; `.structured-coding/standards.md`
(`review.conventions`); `step-20-client-settings.md` §§1.5 (QSET-7, all-platform requirement), §3.2,
§3.8, §3.9, §6 (SET-b row), §7 (AC-SET-1 … 16 that SET-b extends), §8, §12 (SET-a design and ledger,
merged as PR #117, merge commit `44ac762`); `clients/shared/SETTINGS.md` §§2–9 (the module as it
stands after SET-a); `docs/ENGINEERING_RULES.md` §§2–3, 10–12; `docs/ENGINEERING_STANDARDS.md`;
`docs/DECISIONS.md` `ARC-76` (the module), `ARC-70` (wording), `DEP-35` (engine APIs adopted),
`DEP-8` (no new dependency is proposed).

**Scope of this file.** It is the only file this session writes. It names no new decision number:
SET-b adds a dated note under `ARC-76` (§11), and the primary session applies it.

---

## 1. Identity, base, scope

- **Working name:** SET-b. PR number assigned when the PR opens.
- **Branch:** `docs/set-b-design` for this design PR. The implementation PR later takes
  `mvp0/pr-set-b` from `origin/main` **after this design is frozen**, in a fresh session, in its own
  worktree held by one session (`CLAUDE.md` §3.1).
- **Base:** `origin/main @ bb62edf`. SET-a (#117) is on it. The anchors in §2 were read on this commit.
- **Goal:** in the 3D client, the player sets mouse-look sensitivity and vertical-look inversion in the
  settings menu; both values apply on Apply, persist per user in `[input]`, and survive a restart. In
  both clients the Controls tab shows the current key map, read-only, in the player's language.
- **Non-goals:**
  - key rebinding (deferred by QSET-7; it would put the input map in the settings file, and the two
    clients build their maps differently today, step-20 §3.9);
  - gamepad, touch, and any other input device;
  - a mouse-look control in 2D (2D has no look; the capability set hides it, §4.3);
  - the World section and any HTTP (SET-c);
  - any change to the server, the protocol module, contracts, kernel or a pack;
  - any change to the promenade (`./mineworld-3d`), which runs on defaults (QSET-9);
  - a new decision number, a new dependency, or a new font.

## 2. Audit anchors (read on `origin/main @ bb62edf`; re-verify at freeze)

| Id | Anchor | What it shows |
| --- | --- | --- |
| A-SETb-1 | `clients/3d-spike/scripts/player.gd:31` | `const MOUSE_SENS := 0.0016`: the accepted look rate. Sensitivity 100 % in this design is this constant, so the default feel does not change. |
| A-SETb-2 | `player.gd:145–151` | `_unhandled_input` rotates yaw by `-relative.x * MOUSE_SENS` and sets `rig.pitch = clampf(rig.pitch - relative.y * MOUSE_SENS, ±PITCH_LIMIT)`. There is no inversion and no scale hook. |
| A-SETb-3 | `player.gd` header and `scripted_look` / `ignore_mouse_look` | The promenade and the slice share this file. A new hook must default to today's behaviour. |
| A-SETb-4 | `clients/3d-spike/project.godot` `[input]` | Physical-key bindings: `move_forward` W, `move_back` S, `move_left` A, `move_right` D, `camera_cycle` (V and F5), `jump` Space, `jog` Shift. `ui_cancel` (Esc) is Godot's built-in. This is the 3D key map's only source. |
| A-SETb-5 | `clients/2d/scripts/app.gd:345–356` (`_inputs`) | 2D's WASD and arrows are added to `InputMap` at runtime. |
| A-SETb-6 | `clients/2d/scripts/app.gd:206–224` (`_unhandled_input`) | 2D's other keys are literal `KEY_*` checks, not `InputMap` actions: Esc, E, Q, I, H. A key map read from `InputMap` alone would miss them. |
| A-SETb-7 | `clients/shared/settings/settings.gd` | `SECTIONS = ["general", "display"]`, `KEYS`, `read`, `write`, `differs_in`, `copy`, `equals`. `[input]` is not owned yet; SET-a's SETTINGS.md §3 reserves it. |
| A-SETb-8 | `clients/shared/settings/store.gd` | Per-section merge on save (SETTINGS.md §4). An `[input]` section written by another client is kept untouched by a display Apply. |
| A-SETb-9 | `clients/shared/settings/menu.gd` | Signals `opened`, `closed`, `quit_requested` (l. 10–14). `add_tab(key, control)` at l. 98. Capability check for render scale at l. 409 (`MineWorldDisplay.CAP_RENDER_SCALE`). No `applied` signal exists yet. |
| A-SETb-10 | `clients/3d-spike/scripts/slice/slice_main.gd:91–110` | The menu is built with `PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE])`; Esc is handled in `_input` before the player (l. 139–142). |
| A-SETb-11 | `clients/3d-spike/scripts/slice/slice_probe_world.gd:727–760` (`_menu_takes_input`) | The AC-SET-12 probe already injects `InputEventMouseMotion` and counts requests; the same pattern measures yaw per count. |
| A-SETb-12 | `clients/shared/settings/locale/en.po` | `ui.settings.*` keys exist: `tab.general`, `tab.display`, `percent_value`, `hint`, … No `ui.settings.tab.controls` or `ui.settings.controls.*` key exists yet. |
| A-SETb-13 | SET-a's C7 evidence (step-20 §12.5) | Two runs of the same 2D stills are not byte-identical (the world moves). A frame-capture oracle therefore cannot prove "no change". §4.5 uses the request transcript instead of captures. |

## 3. Design decisions (SD-SETb-n)

| Id | Decision |
| --- | --- |
| SD-SETb-1 | **Two settings, both in `[input]`:** `mouse_sensitivity` (integer percent) and `invert_y` (boolean). No other input setting is stored. A key binding is never in the file (SETTINGS.md §1; QSET-7). |
| SD-SETb-2 | **Sensitivity is a discrete choice, not a slider.** Allowed values 50, 75, 100, 125, 150, 200 (percent); default 100. The same pattern as `render_scale` (SD-SET-a: an allow-list read by `MineWorldSettings.read`). Recommendation and alternative: operator question Q-SETb-1. |
| SD-SETb-3 | **`MineWorldSettings` owns the new fields, with the existing rules.** `INPUT := "input"` joins `SECTIONS` and `KEYS["input"] = ["mouse_sensitivity", "invert_y"]`. `read`, `write`, `differs_in`, `copy`, `equals` gain the two fields. Unknown keys in `[input]` are now judged (one warning, dropped), as in `general` and `display`. Sections other than the owned three stay untouched. |
| SD-SETb-4 | **A pure conversion lives in the value, not in the client:** `MineWorldSettings.look_scale() -> float` returns `mouse_sensitivity / 100.0`. The 3D player receives a float and a bool; it never reads a `ConfigFile` or the store. |
| SD-SETb-5 | **The player gets a look hook that defaults to today's behaviour.** `player.gd` gains `var look_scale := 1.0` and `var invert_look_y := false`, set through `func set_look(scale: float, invert_y: bool) -> void`. The yaw term is `-relative.x * MOUSE_SENS * look_scale`; the pitch term is `-relative.y * MOUSE_SENS * look_scale * (-1 if invert_look_y else 1)`. `MOUSE_SENS` is not changed. The promenade never calls `set_look`, so its look is unchanged (AC-SETb-9). |
| SD-SETb-6 | **Apply, not Keep.** Sensitivity and invert apply on Apply and are saved on Apply, like language and clock. They get no 15-second confirmation, because they cannot strand a player (the reason the display changes have one). Operator question Q-SETb-5 asks for confirmation of this. |
| SD-SETb-7 | **The menu gains one signal and no tab of its own.** `signal applied(settings: MineWorldSettings)` is emitted at the end of `apply()`. The 3D composition root connects it to `player.set_look(...)`. This is one line in `menu.gd`; the Controls tab is added with `add_tab` (SET-a's API), so the existing tabs are not edited. |
| SD-SETb-8 | **The Controls tab is a module script, `settings/controls_tab.gd`** (`MineWorldControlsTab`, a `VBoxContainer`). It builds its rows in code, like the rest of the module. A client adds it: `menu.add_tab("ui.settings.tab.controls", MineWorldControlsTab.new(store, capabilities, key_rows))`. |
| SD-SETb-9 | **Capability gating, as render scale.** A new capability `MineWorldControlsTab.CAP_MOUSE_LOOK := "mouse_look"` shows the two controls. The 3D client passes it; the 2D client does not. The key map is shown in both clients (it has no capability). |
| SD-SETb-10 | **The key map is a list of typed rows declared by the client, not read by the module.** `MineWorldKeyRow` (a `RefCounted` with `action: String` (a catalog key), `keys: PackedInt32Array` (physical keycodes)). The module never reads `InputMap` or a client's scripts (the module knows no client, SETTINGS.md §2). Each client builds its rows from its own binding source (§4.4). |
| SD-SETb-11 | **Key names are Godot's, not catalog text.** A row's key labels come from `OS.get_keycode_string(...)` (or the layout label API the probe picks, Q-SETb-4). The row's action name is a catalog key `ui.settings.key.<action>`. Key labels are engine data and are admitted by an allow-list in `client_text.rs`, each with its reason (INV-SET-4). |
| SD-SETb-12 | **Catalog keys (new, SET-b):** `ui.settings.tab.controls`; `ui.settings.controls.sensitivity`; `ui.settings.controls.invert_y`; `ui.settings.controls.invert_y.on` / `.off`; `ui.settings.controls.keys` (the key map heading); `ui.settings.controls.keys.none`; `ui.settings.controls.read_only` ("Key bindings are fixed for now."); `ui.settings.key.<action>` for each row the 3D client and 2D client declare (for example `move_forward` "Walk forward", `camera_cycle` "Change camera", `jump` "Jump", `jog` "Hold to run", `talk` "Talk" in 2D's wording, as the client's rows need). Percentages reuse `ui.settings.percent_value`. Keys follow SETTINGS.md §6.1. Each new key is in both `en.po` and `zh_Hans.po` of the shared layer. |
| SD-SETb-13 | **Harness independence stays.** The new 3D probe mode is a harness flag like every probe mode (`slice-controls`); it runs on defaults unless it passes `--settings=<path>` (INV-SET-5). |
| SD-SETb-14 | **No platform branch (AC-SET-16 holds).** Mouse deltas and keycodes are Godot's on every OS. Per-OS differences are OS pointer acceleration, which sits before Godot and is the player's own setting, and the key labels, which Godot names. §10 lists the hand checks. |
| SD-SETb-15 | **2D key map from one table.** `clients/2d/scripts/controls.gd` (a `class_name` with one `const` table of the 2D bindings) is read by `app.gd` `_unhandled_input` and `_inputs()` and by the 2D composition root that builds the rows. No behaviour changes; the literal `KEY_*` checks become table lookups. Recommendation and alternative: Q-SETb-3. |

## 4. Settings keys, defaults and sources

### 4.1 The file

```ini
[input]
mouse_sensitivity=100      ; percent; one of 50, 75, 100, 125, 150, 200
invert_y=false             ; boolean
```

`[meta] version` stays 1. A file without `[input]` reads as the defaults below (SETTINGS.md §4,
"missing key → default, silently").

### 4.2 Defaults and sources

| Key | Type | Allowed | Default | Source of the default |
| --- | --- | --- | --- | --- |
| `input` / `mouse_sensitivity` | integer (percent) | 50, 75, 100, 125, 150, 200 | 100 | `player.gd:31`: `MOUSE_SENS := 0.0016` is the accepted look rate, so 100 % is today's feel exactly (A-SETb-1). |
| `input` / `invert_y` | boolean | true, false | false | Today's look: moving the mouse up looks up (A-SETb-2). Inversion is opt-in. |

Effective yaw per mouse count at 100 % is `0.0016` rad, unchanged. The allowed range is the
customary range of game sensitivities; the operator may widen it (Q-SETb-1). No value is derived from
a real-world reference, so the realistic-defaults rule does not apply.

### 4.3 Behaviour of the Controls tab, per client

| Row | 3D client | 2D client |
| --- | --- | --- |
| Mouse sensitivity (a choice: 50 %, 75 %, 100 %, 125 %, 150 %, 200 %; labels `ui.settings.percent_value`) | shown (`CAP_MOUSE_LOOK`) | hidden |
| Invert vertical look (On / Off) | shown | hidden |
| Key map (read-only, one line per row: action name, then its keys) | shown | shown |
| Note `ui.settings.controls.read_only` | shown | shown |

The tab's controls use the menu's Apply/Revert: a change is a draft until Apply (SD-SETb-6).
Revert restores the store's saved values. Close drops an unapplied draft.

### 4.4 Key-map rows, per client

- **3D:** the client builds its rows from `InputMap` in `slice_main.gd` for these actions, in this
  order: `move_forward`, `move_left`, `move_back`, `move_right`, `jump`, `jog`, `camera_cycle`, plus
  Esc as a row keyed `ui.settings.key.menu` (Esc is `ui_cancel`, which the slice handles in `_input`).
  The row list is built from the actions' physical keycodes at start; if an action has no event the
  row shows `ui.settings.controls.keys.none`.
- **2D:** the client builds its rows from `controls.gd`'s table (SD-SETb-15): WASD and arrows as
  walking; E, Q, I, H and Esc as the actions they run in `app.gd` today. Each row's action key is one
  of the 2D entries of `ui.settings.key.*`.

### 4.5 Storage rules that stay unchanged

- The file holds only the two keys above under `[input]` (INV-SET-2; AC-SETb-6).
- Key bindings, the invite, seat, nickname, server address and admin token are never written (INV-SET-2,
  INV-SET-3).
- `clients/shared/host/` does not exist yet and is not touched (SET-c).

## 5. Behaviour and flow

1. **Start (3D):** `slice_main._settings()` opens the store, then after `MineWorldDisplay.apply` calls
   `player.set_look(store.settings.look_scale(), store.settings.invert_y)`. A harness run passes
   `--settings=none` and gets 1.0 and false.
2. **Apply:** the menu saves `[input]` (only if it differs, SETTINGS.md §4 step 1) and emits
   `applied(settings)`. The 3D client calls `player.set_look(...)` again, so the next mouse event uses
   the new value, with no restart.
3. **Revert or Close:** the draft is dropped; `applied` is not emitted; `player` keeps the saved values.
4. **Menu open:** the existing rule holds (SETTINGS.md §7). The mouse is released, so mouse motion
   does not look (AC-SETb-11).
5. **2D:** the tab shows the key map and nothing else of SET-b. No `[input]` value changes 2D's behaviour.

## 6. Files touched

```text
new   clients/shared/settings/controls_tab.gd (+ .uid)          MineWorldControlsTab, MineWorldKeyRow
new   clients/2d/scripts/controls.gd (+ .uid)                  the 2D binding table (SD-SETb-15)
edit  clients/shared/settings/settings.gd                      [input] fields, read/write/differs/copy/equals, look_scale()
edit  clients/shared/settings/menu.gd                          signal applied; emitted in apply(); no other change
edit  clients/shared/settings/locale/en.po, zh_Hans.po         new ui.settings.* keys (SD-SETb-12)
edit  clients/shared/settings/locale/messages.pot              regenerated by the catalog check
edit  clients/shared/SETTINGS.md                               §3 table, §4 [input], §6 keys, §7 tab, §8 notes, §9 checks
edit  clients/shared/checks/store_check.gd                     [input] cases (AC-SETb-7)
edit  clients/shared/checks/menu_check.gd                      Controls tab walk, applied signal (AC-SETb-8)
edit  clients/3d-spike/scripts/player.gd                       set_look, look_scale, invert_look_y (SD-SETb-5)
edit  clients/3d-spike/scripts/slice/slice_main.gd             controls tab with CAP_MOUSE_LOOK; set_look at start and on applied; key rows from InputMap
edit  clients/3d-spike/scripts/slice/slice_probe_world.gd      mode `controls` (yaw/pitch per count, persistence, menu-open check)
edit  clients/3d-spike/scripts/slice/slice_probe.gd            only if the mode list needs the new name
edit  clients/2d/scripts/app.gd                                table-driven keys; controls tab with rows, no capability
edit  clients/2d/scripts/harness/settings.gd                   scenario `controls` (key rows equal the table; no sensitivity control)
edit  clients/2d/README.md, clients/3d-spike/README.md         one line each: Controls tab
edit  tests/acceptance/tests/client_text.rs                    AC-SET-3 scan covers controls_tab.gd; key-label allow-list with reasons
edit  tools/cli/tests/client_settings.rs                       new #[ignore] tests (§7.2)
edit  docs/DECISIONS.md                                        dated note under ARC-76 (§11), applied by the primary session
```

No file under `clients/protocol/`, `server/`, `contracts/`, `kernel/`, `presentation/` (pack
files) or `clients/shared/host/` is touched. No new dependency, no font, no autoload, no project setting.

## 7. Tests

### 7.1 Module checks (headless, through `client_settings.rs`)

- `store_check.gd`: `[input]` round trip of both keys; defaults when the section is absent; the value of
  `look_scale()` for each allowed percent; bad values (`mouse_sensitivity=999`, `mouse_sensitivity="fast"`,
  `invert_y="yes"`) take their defaults, each with one warning naming file and key; an unknown key in
  `[input]` is dropped with a warning; the file is byte-identical after load and `.bak` holds the old
  bytes after an Apply; a display Apply does not rewrite `[input]` written by another client.
- `menu_check.gd`: the Controls tab's texts turn Chinese at once and back exactly (the AC-SET-1 walk over
  this tab); under the marker catalog every Controls text carries `⟦`; the 2D capability set does not
  show the two controls; Apply emits `applied` with the new values; Revert and Close emit nothing.

### 7.2 Runtime tests (`tools/cli/tests/client_settings.rs`, `#[ignore]`, Godot)

- `three_d_controls_apply_live_and_persist` — probe `slice-controls` on `--world`: scripted mouse motion
  of 100 counts at 50 %, 100 % and 200 % gives a yaw change within 1 % of `-100 * 0.0016 * scale`; after
  Apply without restart, the new scale is in effect; a second run with `--settings=<file>` written with
  200 % and `invert_y=true` shows the new scale and the pitch sign flipped; the saved file parses into the
  keys of §4.1 only.
- `three_d_settings_never_reach_the_server` — the request transcript of a deterministic run is equal on
  defaults and on a file with 200 % and inversion (the transcript analogue of AC-SET-5; captures are not
  used, A-SETb-13).
- `two_d_controls_tab_shows_key_map_and_no_look_controls` — the 2D tab's key rows equal `controls.gd`'s
  table; the 2D tab contains no sensitivity or invert control; `harness/settings.gd` scenario `controls`
  reports both.
- `the_promenade_look_is_unchanged_by_a_planted_file` — `./mineworld-3d` runs its look probe with a
  planted `[input]` file (200 %, inverted) in the shared user directory; its yaw and pitch per count equal
  the defaults (the promenade reads no settings, QSET-9).
- `the_controls_module_checks_pass` — the module checks of §7.1, run from `client_settings.rs`.

### 7.3 Static checks (`cargo test`, in CI)

- `client_text.rs`: AC-SET-3 scans `controls_tab.gd` and the 3D changes for literals in text sinks;
  AC-SET-4 checks every new `ui.settings.*` key in both languages and that `messages.pot` is current;
  INV-SET-1 identifier scan holds for `settings/`; AC-SET-16 platform scan holds.
- `client_rules.rs` and `scripts/check_client_rules.py` (R4, R6): green with `controls.gd` present (its
  keys are not action-type literals; it uses `ui.settings.key.<action>` keys).

## 8. Mutations (each must turn a named test red; each is reverted and recorded)

| Id | Mutation | Must fail |
| --- | --- | --- |
| M-SETb-1 | `look_scale` applied squared (`scale * scale`) | the 50 %/200 % yaw test (§7.2, first test) |
| M-SETb-2 | `invert_y` ignored in the pitch term | the inversion test (§7.2, first test) |
| M-SETb-3 | `set_look` called only at start, not on `applied` | the live-apply part of the first test |
| M-SETb-4 | the 2D tab shows the sensitivity control (capability ignored) | `two_d_controls_tab_shows_key_map_and_no_look_controls` |
| M-SETb-5 | a planted key-binding row written to `[input]` (for example `key_jump=...`) | the file-contents assertion and `store_check` (unknown key) |
| M-SETb-6 | a literal `"Sensitivity"` in `controls_tab.gd` | `client_text.rs` AC-SET-3, naming the line |
| M-SETb-7 | a planted `OS.get_name()` branch in `controls_tab.gd` | `client_text.rs` AC-SET-16 |
| M-SETb-8 | the menu's Controls tab not re-rendered on `NOTIFICATION_TRANSLATION_CHANGED` | the AC-SET-1 walk over the tab (`menu_check.gd`) |
| M-SETb-9 | the promenade's `player.gd` default changed to 0.5 | `the_promenade_look_is_unchanged_by_a_planted_file` (and the defaults) |
| M-SETb-10 | 2D's `_unhandled_input` keeps its literals while the table says otherwise | `two_d_controls_tab_shows_key_map_and_no_look_controls` (rows equal the table, not the behaviour) — and the existing 13a/13b suites for 2D keys |

## 9. Acceptance (fixed before measuring)

| Id | Criterion | Oracle | Mutation that fails it |
| --- | --- | --- | --- |
| **AC-SETb-1** | Mouse sensitivity changes the look rate, live and after restart. At 50 %, 100 % and 200 % a scripted 100-count horizontal motion turns yaw by `-100 × 0.0016 × scale` within 1 %; a value applied with Apply takes effect without a restart. | `EVIDENCE` line from `slice-controls` | M-SETb-1, M-SETb-3 |
| **AC-SETb-2** | Invert Y flips the vertical look and nothing else. With inversion, a scripted vertical motion moves pitch in the opposite sign to the default run, with the same magnitude; with the default, pitch behaves exactly as on `main`. | `EVIDENCE` line | M-SETb-2 |
| **AC-SETb-3** | Input settings never reach the server. The request transcript of the same run is equal on defaults and on a file with 200 % and inversion. | request transcript | a planted look-related field in a request (the transcript differs) |
| **AC-SETb-4** | The key map shows the bindings the client uses. 3D rows equal the physical keycodes of the `InputMap` actions they name; 2D rows equal `controls.gd`'s table, which `app.gd` reads. The map is read-only: no control writes a binding, and the file holds no `key_*` key. | the rows printed by the probe, compared with the bindings | M-SETb-5, M-SETb-10 |
| **AC-SETb-5** | The 2D client offers no look control. The 3D client offers both. The capability set decides, and the menu decides nothing else per client. | the menu's item list in each client | M-SETb-4 |
| **AC-SETb-6** | The settings file holds only the keys of §4.1 under `[input]`, and no invite, token, seat, address or nickname (extends AC-SET-6). | file content after an Apply in each client | M-SETb-5 |
| **AC-SETb-7** | Bad `[input]` values never break a client (extends AC-SET-8). `mouse_sensitivity=999`, `mouse_sensitivity="fast"` and `invert_y="yes"` each take their default with one warning naming file and key, the file stays byte-identical until Apply, and an unknown key in `[input]` is dropped with a warning. | `store_check` and the runtime bad-file case | a store that writes on load |
| **AC-SETb-8** | Every Controls text is translated at once and reproduces exactly on switching back (extends AC-SET-1, -2, -3, -4). A new key missing from `zh_Hans.po` fails. | the tab walk; `client_text.rs` | M-SETb-6, M-SETb-8 |
| **AC-SETb-9** | Harnesses and the promenade are independent of the player's input settings (extends AC-SET-9). A `slice-controls` run without `--settings=` ignores a planted `[input]` file; the promenade's look per count equals the defaults with the same file planted. | `EVIDENCE` lines | M-SETb-9; a probe without the harness flag |
| **AC-SETb-10** | No gameplay input while the Controls tab is open (extends AC-SET-12): 2 s of W, mouse motion and E with the tab open produce no request and no yaw or pitch change. | request count; yaw; pitch | the menu not consuming mouse motion (existing AC-SET-12 mutation applies to this tab) |
| **AC-SETb-11** | The 3D sensitivity control and inversion survive a relaunch: the value saved in one 3D run is in effect in the next run's first frame, read through the same `EVIDENCE` path as AC-SET-7. | `EVIDENCE` line | a store that saves only on quit |

Criteria with no run are `INCONCLUSIVE`, never `PASS`. A static check passes only when its output is read.

## 10. Commit plan

Each commit tracks implementation, deterministic validation and LLM logic review as separate items,
`[x]` only with its evidence, `N/A` with the audited reason. All items below are open; this design
commit is C0.

### C0 — design (this file)

- [ ] Implementation: this file, `DRAFT`, at the design branch's head.
- [ ] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`
  pass on the branch (no new decision id is proposed; the dated `ARC-76` note is applied later by the
  primary session).
- [ ] Review (LLM): every SD traces to the step's §§3.2, 3.8, 3.9 and QSET-7; every AC traces to §7 of
  step-20 or extends one; the step's oracle for SET-b (a frame capture) is replaced by the transcript,
  §2 A-SETb-13.

### C1 — specs before code (Markdown only)

- Goal: a reviewer can check the code against `SETTINGS.md` alone.
- [ ] Implementation: `clients/shared/SETTINGS.md` §3 (the two fields), §4 (`[input]` file rules and
  the unknown-key rule), §6.1 (new keys), §7 (the Controls tab and the `applied` signal), §8 (INV-SET-2
  and -3 hold for `[input]`), §9 (checks); `docs/DECISIONS.md` dated note under `ARC-76` (applied by the
  primary session).
- [ ] Validation: the two heading and decision-id checks pass; `SETTINGS.md` names every key of §4.1.
- [ ] Review (LLM): `SETTINGS.md` uses the terms of `CORE_CONCEPTS.md` (Presentation, Observation,
  ActionIntent) unchanged; no defined term is added.
- Commit boundary: documentation only.

### C2 — the settings value and the store (no UI, no client)

- Goal: `[input]` loads, validates, saves and converts to a look scale, headless-safe.
- [ ] Implementation: `settings/settings.gd` (`INPUT`, `KEYS`, `SECTIONS`, the two fields,
  `look_scale()`, `read`/`write`/`differs_in`/`copy`/`equals`); `checks/store_check.gd` cases.
- [ ] Validation: `godot --headless --path clients/shared --script res://checks/store_check.gd` → PASS
  with the new `[input]` lines (`[PASS]` count and `0 [FAIL]` printed); the macOS run is the gate.
- [ ] Review (LLM): no `Dictionary` crosses the API; `write` writes only §4.1's keys (INV-SET-2); no
  network identifier added (INV-SET-1 scan); no platform branch.
- Commit boundary: the module's own project only.

### C3 — the Controls tab and its text

- Goal: the tab, the key-row type and the catalog keys, before any client adds it.
- [ ] Implementation: `settings/controls_tab.gd` (`MineWorldControlsTab`, `MineWorldKeyRow`,
  `CAP_MOUSE_LOOK`); `settings/menu.gd` (`applied` signal only); `en.po`, `zh_Hans.po` (SD-SETb-12),
  `messages.pot` regenerated; `checks/menu_check.gd` walk of the tab.
- [ ] Validation: `cargo test -p mineworld-acceptance --test client_text` → PASS (AC-SET-3, AC-SET-4,
  INV-SET-1, AC-SET-16 with the new file); `menu_check.gd` → PASS; a mutation planted in the tab (M-SETb-6)
  is named by the scan.
- [ ] Review (LLM): the tab names no client; keys are dotted and in the allow-listed families; the
  key-label allow-list entry has its reason.
- Commit boundary: module and tests only.

### C4 — the 3D client

- Goal: sensitivity and inversion act in the slice and persist; the promenade is unchanged.
- [ ] Implementation: `player.gd` (`set_look`, `look_scale`, `invert_look_y`; `MOUSE_SENS` unchanged);
  `slice_main.gd` (tab with `CAP_MOUSE_LOOK`, rows from `InputMap`, `set_look` at start and on
  `applied`); `slice_probe_world.gd` mode `controls`; the harness-flag list (via `slice_probe.gd` only if
  needed).
- [ ] Validation: `./mineworld-slice --world --settings=none` controls probe → PASS with the yaw and
  pitch numbers in `EVIDENCE`; `./mineworld-slice --drive` and `--world --link` still "all checks pass";
  the promenade's look probe with a planted file equals the defaults (macOS, the gate); the existing
  AC-SET-12 3D run unchanged.
- [ ] Review (LLM): `player.gd` gains no settings read (one `set_look` call only); the change to the
  promenade's behaviour is zero at defaults; no `ConfigFile` in the client.
- Commit boundary: the 3D client, its probe and its tests.

### C5 — the 2D client

- Goal: the 2D key map from one table, and the tab with no look controls.
- [ ] Implementation: `clients/2d/scripts/controls.gd` (the table); `app.gd` (`_unhandled_input` and
  `_inputs()` read the table; the Controls tab added with rows and no capability); `harness/settings.gd`
  scenario `controls`.
- [ ] Validation: the 13a/13b suites on this tree (`client_2d` 8/8, `client_2d_interact` 5/5,
  `client_2d_interact_stub` 3/3 at the SET-a head) remain green; `client_settings` 2D controls test
  PASS; `check_client_rules.py` → PASS (0).
- [ ] Review (LLM): no 2D binding changed; no literal `KEY_*` left in `app.gd`'s input path (or each
  left one is listed with a reason).
- Commit boundary: the 2D client, its launcher-free harness and its tests.

### C6 — cross-client gate and ledger

- [ ] Implementation: the ledger in this file, the evidence, `docs/MVP_STATUS.md` row for SET-b.
- [ ] Validation: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, `cargo test --workspace` (Godot tests `--ignored` run at the gate on macOS); CI's fast and
  test layers on the PR head.
- [ ] Review (LLM): every AC-SETb-n has a result, `PASS`, `FAIL` or `INCONCLUSIVE`, from evidence; the
  Windows and Linux items of §12 are listed for the operator.

## 11. Decision records (for the primary session to apply)

- **A dated note under `ARC-76` (no new number):** "Extended by S20 SET-b: `[input]` holds
  `mouse_sensitivity` (percent, allow-listed) and `invert_y`; the Controls tab is added by each client
  through `add_tab`, shows a key map declared by the client as `MineWorldKeyRow`s, and offers look
  controls only with `CAP_MOUSE_LOOK`. Key rebinding stays deferred (QSET-7)."
- **No DEP.** No dependency, font or engine API is added. `InputMap`, `OS.get_keycode_string` and the
  key-label API are Godot's (Q-SETb-4 confirms which one the probe uses).

## 12. Hand checks (step-20 §8 style; each names its platform line)

The automated column runs on macOS at the gate. Windows and Linux are the operator's checklist; no
headless CI check can stand in for them (step-20 §8).

| Id | What to look at | macOS | Windows (D3D12 and Vulkan) | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- | --- |
| H-11 (SET-b) | Controls tab in `en` and `zh_Hans`; key labels read correctly on the operator's keyboard layout | gate (US layout) and one non-US layout by eye | checklist | checklist (`fc-list` note as step-20 H-1) | checklist |
| H-12 (SET-b) | Feel at 50 %, 100 %, 200 % with the OS pointer speed at default | by eye | checklist | checklist | checklist (pointer capture under Wayland; relative motion) |
| H-13 (SET-b) | Invert Y: look up goes down when inverted; default unchanged | gate | checklist | checklist | checklist |

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-SETb-1 | 2D key drift: the displayed map and the bindings differ. | SD-SETb-15 (one table) and AC-SETb-4; M-SETb-10. |
| R-SETb-2 | `player.gd` is shared with the promenade; a default change changes the promenade. | SD-SETb-5 keeps defaults at 1.0 and false; AC-SETb-9; M-SETb-9. |
| R-SETb-3 | Key labels differ by keyboard layout, so the map can show a letter the player does not see on the keycap. | Q-SETb-4; H-11 on a non-US layout. |
| R-SETb-4 | The step's oracle (a frame capture equal across settings) cannot discriminate (A-SETb-13). | §7.2 uses the request transcript and measured yaw. |
| R-SETb-5 | OS pointer acceleration stacks with the in-game multiplier; the range may be too coarse or too fine on some systems. | Sensitivity is relative to today's feel; H-12 on each OS; the range is an operator question (Q-SETb-1). |
| R-SETb-6 | `[input]` becomes an owned section, so an existing file with stray keys in `[input]` now logs warnings. | No file exists with `[input]` today (SET-a writes none); AC-SETb-7 covers it. |
| R-SETb-7 | `menu.gd` is touched by a second PR (the `applied` signal). | One line; SET-c also needs the menu; the change is additive. |

## 14. Operator questions

These need an operator ruling before the design is frozen. Each has a recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-SETb-1** | Sensitivity as a choice (50, 75, 100, 125, 150, 200 %) or a slider with a free range? | **Choice**, as `render_scale` is: it is testable, matches the existing menu style, and avoids float equality in the file. Widen or narrow the allow-list if preferred. |
| **Q-SETb-2** | Invert Y in 3D only (the capability), as designed here, or in both clients? | **3D only.** 2D has no look; a control that does nothing is worse than none. Step-20 §3.9 read the pair as a 3D control already. |
| **Q-SETb-3** | The 2D key map: a single `controls.gd` table read by `app.gd` (touches 13b's input code, behaviour unchanged), or a display-only list written beside `_unhandled_input` (two sources, drift risk)? | **The single table** (SD-SETb-15), with the existing 2D suites as the regression proof. |
| **Q-SETb-4** | Key labels: Godot's name for the physical key (`OS.get_keycode_string`, shows "W" on every layout), or the label of the key on the player's layout (the keycap they see, for example "Z" on AZERTY for the W position)? | **The layout label**, because the player looks at the keycap. The probe in C3 confirms which Godot call returns it on macOS; Windows and Linux are H-11. If the layout label cannot be read on one OS, fall back to the physical name and say so in the tab. |
| **Q-SETb-5** | Apply without the 15-second confirmation, since neither value can strand a player? | **Yes**, as designed (SD-SETb-6). |
| **Q-SETb-6** | The step's oracle for SET-b (a frame capture equal under changed settings) is replaced by the request transcript and the measured yaw, because captures are not byte-identical between runs (A-SETb-13). | **Accept the replacement.** |
| **Q-SETb-7** | Should the 3D client's key map also list the mouse buttons (click to interact)? | **No** for SET-b: the mouse buttons are not bindings in `InputMap`; listing them would make the map describe something the client does not read. Revisit with rebinding. |

## 15. Execution contract (proposed; takes effect at freeze)

| Item | Contract |
| --- | --- |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-set-b` (held by one session), created at implementation. |
| Branch | `mvp0/pr-set-b` from `origin/main` after this design is frozen. |
| Platforms | The gate runs on macOS. Windows and Linux items of §12 are the operator's checklist and go into the PR body. |
| Allowed commands | `cargo *`; `git`; `gh` (PR create and view, never merge); `godot *`; `./mineworld-2d`, `./mineworld-slice`, `./mineworld-3d`; the CLI binary under `target/`. |
| Not allowed | `python3 -c`, `sed -i`, `awk`, `xargs`, `curl`, heredoc writes. File changes go through Read, Edit and Write. |
| Authority | Commit and push to the branch; open the PR; mark it READY FOR OPERATOR REVIEW; never merge. |
| Material stops | (1) any edit to `clients/protocol/`, `server/`, `contracts/`, `kernel/` or a pack; (2) a new runtime dependency, font or decision number; (3) an `en` text change beyond the new keys and the 2D rows of §4.4; (4) a change to any 13a, 13b or SET-a check result; (5) a 2D binding whose behaviour changes (Q-SETb-3's refactor must keep every key's action); (6) a platform-specific code path (AC-SET-16); (7) a settings key other than §4.1's two. |
| Gate | §10 C6. |

## 16. Ledger

| Item | Status | Evidence |
| --- | --- | --- |
| C0 (this design) | DRAFT; not frozen | this PR |
| C1 … C6 | not started | — |
| Operator questions Q-SETb-1 … 7 | open | §14 |
| H-11 … H-13 | open (operator checklist) | §12 |

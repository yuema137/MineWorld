# Step 20 — S20 (placeholder): in-game client settings, shared by the 2D and 3D clients

**Lifecycle:** the step plan has been reviewed and its questions are ruled (§1.5, 2026-10-08). **PR
SET-a (§12) is `DESIGN FROZEN 2026-10-08 (primary session)`**, and §12 alone authorizes implementation,
under its execution contract (§12.9), **starting only after S12 13b merges**. SET-b and SET-c are
scoped in §6 and are not frozen. Each PR is frozen on its own (`CLAUDE.md` §3.1).
**Author:** the settings planning session, 2026-10-08. Worktree
`/Users/yuema137/mineworld-worktrees/plan-settings`, branch `plan/client-settings`, from
`origin/main @ 9cf8f8e` (S12 13a merged as #82; S14 16a merged as #79).
**Binding parents:** `CLAUDE.md` §§1.1, 2–4; `overall.md` "Framework, not demo" item 4, "One world, two
views", "Parallel build-out" rulings 1, 4, 6, 8 and 9; `step-19-time-weather.md` §1.3, §7.3, §8.1, §8.2,
§14.1 (QTW-16) and §15.2; `step-13-client-2d.md` §4.1, §14; `step-15-demo-3d.md` §9, §10, §13;
`clients/protocol/ADOPTION.md` §1; `docs/DECISIONS.md` DEP-8 and ARC-55; `docs/REUSE_POLICY.md`;
`docs/ENGINEERING_RULES.md` §§2–3, 10–12.
**Scope of this file.** It is the only file this session writes. Edits to `overall.md`,
`docs/DECISIONS.md`, `step-19-time-weather.md` and the client step documents are proposed in §11, for
the primary session to apply. Decision records use placeholder ids (`ARC-SET-a`, `ARC-SET-b`,
`DEP-SET-a`, `DEP-SET-b`). The step number "S20" is also a placeholder; the primary session assigns
the real one.

---

# 1. The requirement

## 1.1 The operator's words, 2026-10-08 (`overall.md` "Framework, not demo" item 4)

> "增加设置功能，在游戏里可以修改设置。设置要包括基本的语言选择（目前支持英文和简体中文就行，英文是default），然后刷新率 分辨率之类的"

That is: settings that can be changed in game, including language (English, the default, and
Simplified Chinese), refresh rate, resolution and the like. `overall.md` records it as binding on both
reference clients:

- an in-game settings menu with language (`en` by default, plus `zh-Hans`, covering all client UI
  text), display (resolution, window or fullscreen, VSync, frame-rate cap) and basic input options;
- settings persist per user on the local machine;
- settings are presentation-only and never reach the server;
- the 2D and 3D clients share settings through one client-side settings module, which is not the
  protocol module;
- translations are standard translation files in the Presentation layer, so a user can add a language
  without code;
- world content (names, dialogue) is not translated under this requirement;
- options are to be compared (Godot `TranslationServer` with gettext `.po` against CSV; Godot's
  `DisplayServer` and project settings; existing settings-menu addons); the work runs after S12 13a and
  S14 16a land. **Both have landed** (#82, #79).

## 1.2 The operator's requirements as relayed for this planning session, 2026-10-08 (binding)

- **Language:** English (the default) and Simplified Chinese (`zh-Hans`). Every client UI string can
  be translated.
- **Display:** resolution, window mode (windowed, borderless, fullscreen), VSync, and frame-rate cap
  ("刷新率 分辨率之类的", refresh rate, resolution and the like).
- **Time display:** a 12-hour or 24-hour clock, chosen in settings. It is client-only.
- **Host-only commands, reached through the same settings menu:** pause and resume, and a day length
  of 4 h, 2 h or 1 h per real day (default 2 h). These are host commands, not settings. They are sent
  to the server's `/admin/clock` (S19 TW-c, S11-D) and are never stored in the settings file. Non-hosts
  see them read-only, or not at all. See `step-19-time-weather.md` §8.1 and §15.2.
- Settings never reach the server. They are per machine and persisted locally.

## 1.3 Inherited from S19 (`step-19-time-weather.md`)

- §1.3: "游戏里面还要显示日期和时间，时间可以用12hr也可以24hr，用户在设置里选择". The 12h/24h choice is a
  presentation preference in the shared client settings module, never sent to the server. Formats
  follow the language setting and are translation content, not code.
- §8.1's table: Display → Clock (12h/24h; until the user chooses, 12h for `en` and 24h for `zh-Hans`,
  ruled as QTW-16); a World section shown only with a host token (pause and resume, day length 4 h, 2 h
  or 1 h, sent as host commands through `/admin/clock`); a single-player launcher preference ("keep
  the town running when I close the game").
- §8.2's translation entries (`hud.date`, `hud.time.24h`, `hud.time.12h`, `hud.ampm.am`/`.pm`,
  `hud.datetime`, `hud.paused`, weekday and month names) are entries this step's catalogs must be able
  to carry. S19 left the choice of `.po` or CSV to this step.
- §15.2: the amendment to item 4 says pause and day length are host commands reachable from the menu.
  Whether non-hosts see them read-only or hidden "is a presentation choice of the settings-menu PR".

## 1.4 Inherited from S12 PR 13b (frozen 2026-10-08; relayed by the primary session during this planning)

The primary session's coordination note: "2D PR 13b (frozen) routes all 2D UI text through
translation keys now, with English in a gettext `.po` file at
`presentation/mineworld-default/2D/i18n/en.po`. … Your lane still owns the language switch, zh-Hans
and the final format. Design so that the same keys keep working. If you choose a different format,
include a migration of 13b's `.po` in your first PR."

13b's design (`origin/plan/s12-13b`, `step-13-client-2d.md` §15.4 D-b-9, ruling QS13b-1, decision
**ARC-70**): every user-visible 2D string is `tr(key)`, with arguments formatted afterwards. Its key
families are `action.<type>`, `action.<type>.done`, `reason.<code>`, `ui.*`, `panel.*` and
`suggest.invite-kind`. Keys are built from data (`"action." + type`). English lives in the pack's
`i18n/en.po` and is loaded at runtime from the pack directory. A key missing everywhere falls back to
a readable form of its last part (`too_far_away` → "too far away"). `--presentation=none` loads no
wording. The 2D wording helper is `clients/2d/scripts/hud/words.gd`. `check_client_rules.py
--check-pack` parses the `.po`, and R6 flags literal text outside `words.gd`.

**This design keeps 13b's format (gettext `.po`) and every one of its keys unchanged.** It adds the
language switch, `zh_Hans`, a shared catalog layer below the pack's, and a user layer above it (§3.6).
The one migration is a move, not a reformat. Entries that both clients use (`action.*`, `reason.*`)
move verbatim from the 2D pack's `en.po` into the shared catalog, so the 2D and 3D clients use one
translation. Their keys do not change, so 13b's `tr()` calls keep working untouched (SD-SET-a-17).

## 1.5 Rulings, and a new binding requirement, 2026-10-08 (relayed by the coordinator)

**By the operator:**

| Id | Ruling |
| --- | --- |
| QSET-1 | **Noto Sans SC Regular**, under a narrow DEP-8 exception for fonts only: OFL-1.1, bundled with the software and never sold on its own. The exception's text is in §4.4.1. The primary session adds it to `DECISIONS.md` when SET-a lands. |
| QSET-2 | **Yes:** Esc opens the menu in both clients, and Quit moves into the menu. |
| QSET-3 | **Accepted as designed.** "Resolution" is the window size in windowed mode and the 3D render scale in the two full-screen modes. "Refresh rate" is the frame-rate cap plus VSync, with the monitor's rate shown read-only. |
| QSET-7 | **Accepted as designed.** SET-b covers sensitivity, invert-Y and a read-only key map. Key rebinding comes later. |

**By the primary session:** QSET-4 to QSET-16 are accepted as recommended.

**New binding requirement (operator, 2026-10-08):**

> "我们要保证支持全平台，mac linux windows都可以"

That is: every platform must be supported, macOS, Linux and Windows. For SET-a this means:

- §3.5.1 states how each display mode, VSync and the frame cap behave on Windows (D3D12 and Vulkan),
  Linux (X11 and Wayland) and macOS.
- §3.4 states where the shared settings folder resolves on each OS.
- §3.7 states the font fallback on each OS.
- §8 names the OS each hand check must run on. CI runs no Godot (A-SET-12), so no headless CI check
  can stand in for a rendered one. The Windows and Linux hand checks are therefore checklist items
  for the operator, apart from what the platform-independent CI checks already cover (§8).

## 1.6 The answer in brief

```text
module      clients/shared/ is a small Godot project, like clients/protocol/. It holds module folders.
            settings/ is the settings-and-UI-text module: a typed settings value, a ConfigFile store,
            display application, the catalog loader, the clock formatter, the menu, its .po catalogs
            and a bundled CJK font. Each client symlinks it in as res://mineworld_settings, the way it
            already takes res://mineworld (ADOPTION.md §1). No autoload, no project setting.
language    gettext .po through Godot's own TranslationServer — 13b's format and keys (ARC-70), kept.
            Catalogs are layered per key: shared module locale/ (keys both clients use) → the
            Presentation Pack's i18n/ (13b's file; pack wording) → user://locale/ (user overrides).
            Languages are discovered at runtime from the .po files present (Godot loads .po at
            runtime without import, verified here and by 13b), so dropping a fr.po into any layer
            adds French with no code.
            Switching is live (NOTIFICATION_TRANSLATION_CHANGED). Logs, transcripts and evidence
            stay English. World content is never passed through tr().
display     window mode → DisplayServer modes; size presets in windowed mode; a 3D render scale in
            the two full-screen modes; VSync → window_set_vsync_mode; frame-rate cap → Engine.max_fps.
            The display's own refresh rate is shown, not changed (Godot cannot change it).
clock       `auto | 12h | 24h`; `auto` takes each catalog's own default (en 12h, zh_Hans 24h).
persist     one ConfigFile per user, shared by both clients (custom user dir "MineWorld"), written
            atomically. `--settings=<path>|none` overrides it; every harness mode uses `none`.
world       the World section (pause, day length) is a separate module folder that never touches the
            settings file; it sends host commands with a token the launcher hands over. It lands in
            SET-c, after S11-D and TW-c.
PRs         SET-a module + language + display + clock + menu in both clients + every check;
            SET-b basic input options; SET-c the World section (host commands).
```

---

# 2. Audit (`origin/main @ 9cf8f8e`, 2026-10-08; Godot 4.7.2.stable on this machine)

## 2.1 Repository facts

| Id | Finding | Where | Consequence |
| --- | --- | --- | --- |
| A-SET-1 | **The protocol module is shared by symlink.** `clients/2d/mineworld` and `clients/3d-spike/mineworld` are symlinks to `../protocol/mineworld`. ADOPTION.md §1 documents "copy" or "symlink" and says the module has "no dependencies, no autoloads, no project settings and no assets". `clients/protocol/project.godot` sits *above* the module folder and gives it a runnable home with `demo/` and `checks/`. | `find clients -type l`; `clients/protocol/ADOPTION.md` §1 | A shared settings module follows the same pattern: a sibling project `clients/shared/` with module folders under it, each symlinked into the clients. The `project.godot` must sit above the module folder, never inside it, because Godot treats a folder that holds a `project.godot` as a separate project. |
| A-SET-2 | **Step-19 already names the location.** §8.3: "one GDScript module beside the settings module, e.g. `clients/shared/world_time/`; not the protocol module". | `step-19-time-weather.md` §8.3 | `clients/shared/` is the home of shared non-protocol client modules. `settings/` is the first; `world_time/` (TW-e) and `host/` (SET-c) follow. |
| A-SET-3 | **Neither client has a menu.** 2D: `app.gd` `_unhandled_input` quits on `KEY_ESCAPE`. 3D: `player.gd:154` toggles mouse capture on `ui_cancel` (Esc). The 3D controls line says "Esc release mouse"; the 2D hint says "Esc quits". | `clients/2d/scripts/app.gd:115`; `clients/3d-spike/scripts/player.gd:154–159`; `controls_hud.gd:18` | Opening the menu needs a key. Esc changes an accepted control in both clients (QSET-2, operator-material). |
| A-SET-4 | **Every UI string is a hardcoded English literal.** 2D: `app.gd` (4 notes), `hud/status.gd` (hint, who/when lines, the `clock()` format), `scene/places.gd` (`"door to %s"`, `"outside"`). 3D: `controls_hud.gd` (`CONTROLS`, `MODE_TITLES`, `"camera: %s"`), `slice_main.gd` (`"place: %s"`, `"looking at: %s"`, `"world: …"`), `slice_link.gd` (16 `_say(...)` calls), `intents.gd` (verbs used in messages). No `tr()` appears anywhere in either client. | `grep` over `.text =`, `note(`, `toast(`, `add_line(`, `_say(` | Every one of these moves to a key. On the 2D side, 13b (A-SET-17) does most of this; SET-a converts whatever 2D text remains after 13b merges, and all of the 3D text. The 2D rejection note shows raw JSON (`JSON.stringify(result["rejected"])`), which is not text a player can read in any language. |
| A-SET-5 | **The 3D link prints and shows the same string.** `_say(shown, log_detail, notice)` prints `"[link] " + shown` and emits `said(shown)`. The probe modes and tests read those printed lines. | `slice_link.gd:209–211`; `slice_probe_world.gd` | Translation must split the two: the log line stays English and stable, and only the emitted line is translated (INV-SET-6). |
| A-SET-6 | **In-world signage is drawn as text.** `cafe.gd` `NAME := "The Daily Bean"`, `streetscape.gd` signs `["Lakeside", "Town Square", "Market", "Trails"]`, `palette.gd` font lists for signage. | `clients/3d-spike/scripts/slice/*.gd` | This is art on a façade, not client UI. It is out of scope (QSET-10) and is allow-listed by the scan with a reason. |
| A-SET-7 | **World content is shown through display labels.** 2D `people.gd:64` (name or id), `places.gd` (place tags as fallback names); 3D `slice_link.display_label`. | as named | These strings are world content and are never passed through `tr()`. The nodes that show them are marked as world text so the runtime checks can tell them apart (SD-SET-a-9). |
| A-SET-8 | **Nothing uses `user://` today.** `grep -rn "user://" clients/2d clients/3d-spike/scripts clients/protocol/mineworld` finds nothing. Each project's `user://` is separate: this machine has `app_userdata/MineWorld 2D`, `MineWorld 3D Spike` and `MineWorld client protocol`. | the grep; `~/Library/Application Support/Godot/app_userdata/` | One shared settings file needs both projects to name the same user directory (`application/config/use_custom_user_dir`, `custom_user_dir_name="MineWorld"`). That changes nothing else, because nothing else uses `user://`. |
| A-SET-9 | **The launchers fix the window size.** `mineworld-2d` passes `--resolution "$res"` (default 1600x900) in every windowed mode; `mineworld-slice` passes `--resolution "$res"` always. Captures depend on it. | `mineworld-2d` l. ~128–135; `mineworld-slice:106` | In play mode the saved window size must win, so the launchers stop passing `--resolution` unless `--res=` or a capture mode asks for it. Captures keep their fixed size and use `--settings=none`. |
| A-SET-10 | **Both clients build UI in code.** No `.tscn` for HUD or panels; `project.godot` of 2D says "nothing under res:// is an asset" (the art is loaded from disk at runtime). | `clients/2d/project.godot`; `controls_hud.gd`; `hud/status.gd` | The menu is built in code too. Catalogs and the font are loaded at runtime from the module folder, with no import step. |
| A-SET-11 | **"No rule in the client" scans read every `*.gd` under `clients/`.** `tests/acceptance/tests/client_rules.rs` (I-S14-1.1) excludes `clients/*/tools/`, `clients/protocol/demo/`, `clients/protocol/checks/`, does not follow symlinks, and refuses a string literal equal to any action type (`move`, `talk`, `invite`, `buy`, …) outside its allow-list. `scripts/check_client_rules.py` R4 refuses the word `within` outside named 2D files. It reads `clients/2d` with `rglob`. | `client_rules.rs` header and l. 260–275; `check_client_rules.py` | Shared module code must contain no action-type literal and must not trip R4 if `rglob` follows the new symlink. Translation keys are dotted (`ui.menu.quit`), so a key can never equal an action type. `clients/shared/checks/` needs the same exclusion `clients/protocol/checks/` has, *only* if its checks name actions; they do not. SET-a verifies R4 against the symlink (a bounded detail). |
| A-SET-12 | **CI runs no Godot.** `scripts/ci_layer.py` runs `check_doc_headings.py`, `check_decision_ids.py`, `check_ci_pins.py`, `check_scratch.py` and cargo. Godot tests in `tools/cli/tests/client_2d.rs` are `#[ignore]`d ("a test not run is not a pass"). | `scripts/ci_layer.py:39–52`; `client_2d.rs` header | The static half of the "no hardcoded UI string" check and the catalog consistency check must run in `cargo test` to block in CI. The runtime half is an `#[ignore]` Godot test, run at the gate. |
| A-SET-13 | **The 2D and 3D harnesses.** 2D `--drive[=scenario]` (`harness/drive.gd`, scenarios matched by name), driven by `client_2d.rs` against a real server or a stub that records frames. 3D probe modes in `slice_probe.gd` / `slice_probe_world.gd` (`--world --link`, `--world --target`, …). | as named | SET-a adds a `settings` scenario to 2D and a `--world --settings` probe to 3D. The stub's frame record is the oracle for "never reaches the server". |
| A-SET-14 | **No admin surface yet.** No `/admin` route in `server/src`; `server/PROTOCOL.md` has `time_scale` in `welcome` (S11-B) and no `clock` frame. | `grep` | The World section cannot be built before S11-D (admin routes) and TW-c (`/admin/clock`, the `clock` frame, `WorldSummary.paused`). It is SET-c and waits for them. |
| A-SET-15 | **Licence policy.** DEP-8: "ask whether we may relicense, not whether we may redistribute"; MineWorld is MIT. ARC-55's default pack allow-list is `MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, CC0-1.0, Unlicense`. No OFL font is in the repository (`git ls-files` finds no `.ttf/.otf/.woff`). | `docs/DECISIONS.md` l. 267ff, l. 4292ff | A bundled OFL-1.1 font cannot be relicensed under MIT. Shipping one needs an explicit DEP-8 carve-out (QSET-1, operator-material). |
| A-SET-17 | **13b (frozen, not yet merged) owns 2D wording by key.** `hud/words.gd`; `presentation/mineworld-default/2D/i18n/en.po`; key families of D-b-9; ARC-70; `check_client_rules.py --check-pack` and R6; B-10: `check_client_rules.py` must skip symlinks explicitly (Python 3.14's `rglob` does not descend them; older versions may). | `origin/plan/s12-13b`, `step-13-client-2d.md` §15 | SET-a starts after 13b merges and builds on `words.gd` and 13b's keys (§1.4). The 2D half of "no hardcoded string" is 13b's R6. SET-a's scan covers the 3D client and the shared module and does not re-scan `clients/2d`. |
| A-SET-16 | **The 3D promenade spike shares `ControlsHud`.** `mineworld-3d` runs `scenes/main.tscn` on the same project and the same `controls_hud.gd`. | `controls_hud.gd` header; `mineworld-3d` | Translating `ControlsHud` translates the promenade's HUD as a side effect. The promenade gets no menu (QSET-9). |

## 2.2 Engine probe, run in this session (scratch project `/tmp/mw-settings-probe`, Godot 4.7.2, headless)

| Id | Probe | Result |
| --- | --- | --- |
| P-1 | `load("res://i18n/zh_Hans.po")`, with no import step | a `Translation`; `locale` read from the header = `zh_Hans` |
| P-2 | `ResourceLoader.load("/tmp/…/zh_Hans.po")`, an absolute path outside `res://` | a `Translation`. A `.po` can be loaded at runtime from any folder. |
| P-3 | `ResourceLoader.load("res://i18n/ui.csv")` with no import | `No loader found for resource` → null. CSV must go through the editor's importer. |
| P-4 | `standardize_locale("zh-Hans")` | `zh_Hans` |
| P-5 | `set_locale` then `translate`, for `en`, `en_US`, `zh_Hans`, `zh_CN`, `zh_TW`, `fr` | en/en_US → English; zh_Hans/zh_CN/**zh_TW** → the zh_Hans catalog; fr → English (the fallback locale) |
| P-6 | `msgctxt` and `msgid_plural` | context and plural forms both resolve (`{n} person`/`{n} people`; Chinese one form) |
| P-7 | `"{ampm} {hour12}:{minute2}".format({...})` from the zh_Hans catalog | `下午 7:42`: named placeholders let word order differ per language |
| P-8 | `ConfigFile` save then load of `{language: "zh_Hans", max_fps: 60}` | round trip; the integer stays `TYPE_INT` |
| P-9 | `ThemeDB.fallback_font` | a `FontFile` with `allow_system_fallback = true`, and **no glyph for 设**: Chinese today renders only through whatever system font the OS supplies |
| P-10 | pseudolocalization setting, `Node.auto_translate_mode` | both exist |
| P-11 | `DisplayServer` under `--headless` | name `headless`, `screen_get_refresh_rate() = -1`: display application must be a no-op headless |

---

# 3. Design

## 3.1 Where the module lives and how a client takes it (ARC-SET-a)

```text
clients/shared/                 a Godot project, the modules' runnable home (like clients/protocol/)
  project.godot                 config/name "MineWorld shared client modules"; custom user dir "MineWorld"
  README.md                     human orientation
  SETTINGS.md                   the specification of the settings module (an agent document)
  settings/                     THE MODULE: what a client symlinks in as res://mineworld_settings
    settings.gd                 MineWorldSettings — the typed settings value and its validation
    store.gd                    MineWorldSettingsStore — locate, load, validate, save (atomic)
    display.gd                  MineWorldDisplay — apply window mode, size, render scale, VSync, cap
    text.gd                     MineWorldText — catalogs, languages, live switch, code families
    clock_format.gd             MineWorldClockFormat — time of day in the chosen 12h/24h form
    menu.gd                     MineWorldSettingsMenu — the menu (General, Display; later tabs)
    locale/en.po                the shared layer's source catalog: keys both clients use (§3.6)
    locale/zh_Hans.po           Simplified Chinese for those keys
    locale/messages.pot         the template, regenerated by a check from en.po
presentation/mineworld-default/2D/i18n/en.po        13b's pack layer (2D wording), kept
presentation/mineworld-default/2D/i18n/zh_Hans.po   added by SET-a
    fonts/NotoSansSC-Regular.otf  + OFL.txt   (QSET-1)
  checks/                       headless checks of the module (store, catalogs, glyph coverage, menu)
clients/2d/mineworld_settings      -> ../shared/settings   (symlink)
clients/3d-spike/mineworld_settings -> ../shared/settings  (symlink)
```

Rules, stated in `SETTINGS.md`:

1. The module has **no autoload, no project setting it requires, no import step**. A client's
   composition root calls `MineWorldSettingsStore.open(args)` and `apply(...)` first in `_ready`
   (§3.4), then builds its own nodes. The one project setting a client adds is the shared user
   directory (§3.4), and it is optional: without it the client keeps a file of its own.
2. **The settings module has no network code.** No `MineWorldClient`, `HTTPRequest`,
   `WebSocketPeer`, `StreamPeer*`, `submit` or `connect_to_world` appears under `settings/`. A static
   scan enforces it (INV-SET-1).
3. **The World section is not in this folder.** It is `clients/shared/host/` (SET-c), which contributes
   a tab to the menu through the menu's tab API and never reads or writes the settings file
   (INV-SET-3).
4. **Copy or symlink**, as ADOPTION.md §1 allows. The reference clients symlink.

Why not inside `clients/protocol/mineworld`: `overall.md` item 4 says "This is not the protocol
module", the protocol module is S11's and changes only in the order ruling 4 fixes, and settings have
nothing to do with the wire.

## 3.2 What is a setting, and what is not

| Thing | Kind | Stored in the settings file | Reaches the server | PR |
| --- | --- | --- | --- | --- |
| Language | setting | yes | **no** | SET-a |
| Clock 12h / 24h / auto | setting | yes | **no** | SET-a |
| Window mode, window size, render scale (3D), VSync, frame-rate cap | setting | yes | **no** | SET-a |
| Mouse sensitivity, invert Y (3D) | setting | yes | **no** | SET-b |
| Pause / resume, day length | **host command** (`/admin/clock`) | **never** | yes, as a host command | SET-c |
| Admin token | credential, handed over by the launcher | **never** | yes, as the bearer header of a host command | SET-c |
| Invite, seat, server address, nickname | join arguments | **never** | the join frame, as today | — |
| "Keep the town running when I close the game" | launcher preference (S19 §7.5) | yes, `[launcher]` section, read by the client on close | **no** | TW-e (uses the store) |

## 3.3 The settings value (typed, `settings.gd`)

`MineWorldSettings` is a `RefCounted` with typed fields and enums, never a `Dictionary` (`CLAUDE.md`
§4 rule 7):

```text
language: String              a locale the catalogs offer, standardized ("en", "zh_Hans"); default "en"
clock: ClockFormat            AUTO | H12 | H24; default AUTO
window_mode: WindowMode       WINDOWED | BORDERLESS | FULLSCREEN; default WINDOWED
window_size: Vector2i         default (1600, 900), the launchers' size today
render_scale: int             percent, one of 50, 67, 75, 85, 100; default 100; used by 3D only
vsync: VSync                  OFF | ON | ADAPTIVE; default ON (Godot's default)
max_fps: int                  0 (no cap) or one of 30, 60, 120, 144, 165, 240; default 0
                              plus "match display": stored as -1, applied as the display's refresh rate
```

`validate()` returns the value with every out-of-range field reset to its default, and the list of
`(section, key, raw value)` it reset. Nothing invalid survives a load.

## 3.4 Persistence (DEP-SET-a)

- **File:** `user://settings.cfg`, a Godot `ConfigFile`, sections `[meta] version=1`, `[general]`,
  `[display]`, later `[input]` and `[launcher]`. Both `clients/2d/project.godot` and
  `clients/3d-spike/project.godot` set `application/config/use_custom_user_dir=true` and
  `custom_user_dir_name="MineWorld"`, so **one file serves both clients** (QSET-5). Language chosen in
  2D is the language of 3D. Where the shared folder resolves, from Godot's "Data paths" page (custom
  directory with a custom name):

  | OS | `user://` (the shared settings folder) | The file |
  | --- | --- | --- |
  | Windows | `%APPDATA%\MineWorld\` (typically `C:\Users\<user>\AppData\Roaming\MineWorld\`) | `%APPDATA%\MineWorld\settings.cfg` |
  | macOS | `~/Library/Application Support/MineWorld/` | `…/MineWorld/settings.cfg` |
  | Linux | `~/.local/share/MineWorld/`, or `$XDG_DATA_HOME/MineWorld/` when `XDG_DATA_HOME` is set | `…/MineWorld/settings.cfg` |

  The user's extra catalogs (`user://locale/`) sit in the same folder. A check prints
  `OS.get_user_data_dir()` on the OS it runs on and asserts the path ends in `MineWorld` (AC-SET-15).
  It runs on macOS at the gate, and the operator runs it on Windows and Linux (H-10).
- **Override:** `--settings=<path>` uses another file; `--settings=none` uses defaults and never
  writes. Every harness mode (2D `--drive`, `--capture`; 3D `--shots`, `--drive`, `--measure`,
  `--threshold`, `--perf`, `--hud`, `--character`, every `--world` probe) behaves as `--settings=none`
  unless it passes `--settings=<path>` itself (INV-SET-5).
- **Load:** a missing file means defaults, silently. An unreadable or unparsable file means defaults,
  one warning naming the path and Godot's error, and the file is **left as it is** until the user
  applies a change, when it is replaced (the old content kept as `settings.cfg.bak`). Unknown keys are
  dropped with a warning naming them. A `[meta] version` above the one the module knows means
  defaults and a warning, and the file is not overwritten until the user applies.
- **Save:** only on Apply in the menu (and on the display-revert timer's confirmation, §3.8). It
  writes `settings.cfg.tmp` and renames it over `settings.cfg` (`DirAccess.rename_absolute`), so a
  crash leaves the old or the new file, never half of one. On Windows a rename onto an existing
  file is the case to verify. If `rename_absolute` refuses to replace the file there, the store
  removes `settings.cfg` and renames `.tmp` into place, after first keeping a copy as `.bak`. That is a
  bounded detail C2 decides from evidence and records; the observable rule stays the same. Two clients open at once: the last Apply
  wins, whole-file. Each client re-reads the file before writing and keeps the sections it did not
  change, so a 3D Apply of the display tab does not undo a 2D language change made a minute earlier.
- **Precedence at start:** `--settings` choice of file → the file → defaults. A launcher's explicit
  `--res=` (a developer asking for a size) wins over the saved size for that run and is not saved.

## 3.5 Display (applied by `display.gd`)

| Setting | Godot call | Notes |
| --- | --- | --- |
| Windowed | `DisplayServer.window_set_mode(WINDOW_MODE_WINDOWED)`, borderless flag off, then `window_set_size(window_size)` and centre on the current screen | the size list offers presets that fit the current screen's usable rect (1280x720, 1600x900, 1920x1080, 2560x1440, and the screen's own size), in physical pixels |
| Borderless | `WINDOW_MODE_FULLSCREEN` | Godot's non-exclusive full screen is a borderless window covering the screen; on macOS it uses its own Space. Hand check H-3. |
| Fullscreen | `WINDOW_MODE_EXCLUSIVE_FULLSCREEN` | Godot does not switch the monitor's resolution or refresh rate in either full-screen mode; the desktop's mode is kept |
| Render scale (3D only) | `get_viewport().scaling_3d_scale = render_scale / 100.0` (bilinear) | what "resolution" can mean when the window is the whole screen; offered only by a client that declares the 3D capability; the 2D client hides it |
| VSync | `DisplayServer.window_set_vsync_mode(VSYNC_DISABLED / ENABLED / ADAPTIVE)` | Mailbox is not offered (it is not uniformly supported and the difference is not a player-level choice) |
| Frame-rate cap | `Engine.max_fps = n` (0 = none; -1 → `round(screen_get_refresh_rate())`, or none when it reports -1) | "refresh rate" in the operator's words: the game cannot change the monitor's refresh rate (Godot has no API for it); it can cap its own frame rate, and the Display tab shows the monitor's rate read-only (QSET-3) |

### 3.5.1 Per platform (the operator's "全平台" requirement)

Godot changes the monitor's video mode on no platform, in either full-screen mode. The Godot 4.7
`DisplayServer` reference says so for both modes: "The display's video mode is not changed". The
quotations below are from that page.

| Setting | Windows (D3D12 or Vulkan) | macOS (Metal or Vulkan through MoltenVK) | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| Windowed | Decorated window at the saved size, centred on the current screen | same | same | Size honoured. **Position is the compositor's:** a client cannot place its window, so "centre" is a no-op. Recorded, not a defect. |
| Borderless (`WINDOW_MODE_FULLSCREEN`) | A borderless window covering the monitor. The desktop compositor (DWM) stays in the path; Alt-Tab is instant. | "A new desktop is used to display the running project" (its own Space) | Borderless full screen through the window manager; the compositor stays in the path | xdg full screen |
| Fullscreen (`WINDOW_MODE_EXCLUSIVE_FULLSCREEN`) | One window per screen, less overhead. "Depending on video driver, full screen transition might cause screens to go black for a moment." Alt-Tab triggers a transition. | Its own Space, and "prevents Dock and Menu from showing up when the mouse pointer is hovering the edge of the screen" | "bypasses compositor" | "**Equivalent to WINDOW_MODE_FULLSCREEN**." Both entries stay in the menu. On Wayland the Fullscreen entry's tooltip says it behaves as Borderless (key `ui.settings.display.wayland_same`). |
| VSync | Supported (`FEATURE_SWAP_BUFFERS` lists Windows). Adaptive needs driver support for relaxed FIFO and otherwise behaves as On. | Supported | Supported | Supported, but **the compositor may still synchronise presentation with VSync Off**. H-5 records what happens. |
| Frame-rate cap (`Engine.max_fps`) | All platforms. With VSync On or Adaptive the frame rate is also "limited by the monitor refresh rate", so the lower limit wins. With VSync Off the cap is the only limit. AC-SET-11 measures that the cap holds with VSync Off. | same | same | same |
| Monitor rate, read-only (`screen_get_refresh_rate`) | Implemented | Implemented | Implemented | Implemented |
| Render scale (3D) | Viewport scaling on every renderer; bilinear, no FSR | same | same | same |
| HiDPI | The window size is in physical pixels. Under Windows display scaling (125 %, 150 %) the size presets are filtered against the usable rect in physical pixels. | Retina: `screen_get_scale()` is 2.0, and sizes are physical, so 1600×900 is half as tall on screen as on a 1× display. H-4 judges whether presets should be logical; if so, that is a bounded change recorded in C2. | scale 1.0 | Fractional scales are reported rounded up for indirect screen queries; only the main window's screen is queried |

`display.gd` logs the rendering driver (`RenderingServer.get_current_rendering_driver_name()`) and
the display server (`DisplayServer.get_name()`) on start. Each hand check records them, so a result
always names its platform. SET-a adds no Windows- or Linux-specific code path, beyond the
Wayland tooltip text and the no-op centring that Godot itself performs.

Everything is a no-op under the headless display server (P-11), so a headless check can load and
apply settings safely. A window-mode or size change starts a 15-second "Keep these display settings?"
confirmation; no answer reverts to the previous mode and size (an unusable mode never strands a
player). The Display tab also shows the measured frame rate, for the hand checks.

## 3.6 Language and UI text (ARC-SET-b, DEP-SET-a)

**Format and keys: 13b's, unchanged (ARC-70).** gettext `.po`, one per language per layer. Keys are
symbolic, lowercase, dot-separated, with segments of `[a-z0-9_-]` so that 13b's data-built keys fit
(`action.join-group-activity`, `reason.too_far_away`, `suggest.invite-kind`). 13b's families stay
as they are: `action.<type>`, `action.<type>.done`, `reason.<code>`, `ui.*`, `panel.*`, `suggest.*`.
SET-a adds `ui.settings.*` (the menu), `hud.*`, `hint.*`, `camera.*`, `link.*` (3D connection
messages), `note.*`, `door.*`, `state.*`, `language.self_name` and `clock.default`. Every layer's
`en.po` is that layer's source catalog. `messages.pot` is generated from the shared `en.po` by the
catalog check (C3), so translators can use Poedit or `msgmerge`. Plurals use `msgid_plural` (P-6).
Placeholders are named (`{target}`, `{place}`) and filled with `String.format` (P-7), as 13b does.
Each language declares `language.self_name` ("English", "简体中文") and `clock.default` (`12h` or
`24h`) in the shared layer, so a language says how it is listed and which clock it prefers, with no
code.

**Layers, per key, last wins.** All three are the Presentation layer, and all use the same format:

| Layer | Folder | Holds | Owner |
| --- | --- | --- | --- |
| 1. shared | `clients/shared/settings/locale/` | keys both clients use: `action.*`, `reason.*` (moved from 13b's pack file, SD-SET-a-17), `ui.settings.*`, `hud.time.*`, `state.*`, `language.*`, `clock.*`; and the 3D client's own keys until the 3D client reads a Presentation Pack (S14 16f), when they move, keys unchanged, to `presentation/mineworld-default/3D/i18n/` | SET-a |
| 2. pack | `presentation/<pack>/<2D or 3D>/i18n/` | the pack's own wording: 13b's `ui.*`, `panel.*`, `suggest.*`, 2D `hud.*`/`note.*`, and any override of a shared key that a style wants | 13b (en), SET-a (zh_Hans) |
| 3. user | `user://locale/` | a user's additions or corrections | the user |

**Discovery.** At start `MineWorldText` loads every `*.po` of layer 1, then of layer 2 (the folder
the client's composition root passes; 13b's `words.gd` passes the pack directory it already reads),
then of layer 3 (P-1, P-2). It merges them per locale into one `Translation` per locale, where a
later layer's entry replaces an earlier one, and registers that with `TranslationServer`. The
languages offered are those that have a `language.self_name` in some layer. A user's
`user://locale/fr.po` therefore adds French. A missing key falls back first to English
(`internationalization/locale/fallback` stays `en`; P-5) and then, if English lacks it too, to 13b's
readable form of the key's last part. A raw key is never shown. `--presentation=none` (13b) still
removes layer 2. A new `--wording=none` removes every layer, for 13b's wording-removed check.

**Default.** `en` on first launch, whatever the OS locale (the operator: "英文是default"; QSET-8).

**Live switching.** `MineWorldText.set_language(locale)` calls `TranslationServer.set_locale`. Godot
then sends `NOTIFICATION_TRANSLATION_CHANGED` to every node. Labels whose whole text is one key use
Godot's auto-translation. Every label composed in code (a template plus data) sets
`auto_translate_mode = AUTO_TRANSLATE_MODE_DISABLED` and re-renders itself on that notification from
its last data. Nothing needs a restart.

**What is translated, and what is not.**

| Translated (client UI text) | Not translated |
| --- | --- |
| the menu; HUD lines and their templates; the 2D hint and the 3D controls line; camera titles; toasts and notes; door labels' template ("door to {place}"); connection states; the readable form of a refusal code or rejection reason the server sent | world content: names, place names and tags, dialogue, anything the observation carries (`overall.md` item 4); in-world signage art (A-SET-6, QSET-10); stdout logs, `[link]` lines, `drive:` and `EVIDENCE` lines, `push_warning` text (INV-SET-6) |

**Server codes.** A refusal code or rejection reason uses 13b's family: `reason.<code>`, built from
data as 13b does, through `MineWorldText.code("reason", code)`. When no layer has the key (a newer
server), it shows 13b's readable fallback. Keys built from data are confined to the declared families
(`action`, `reason`, `state`). The catalog check makes sure every refusal code and rejection reason
that `server/PROTOCOL.md` lists, and every action type a System Pack declares, has an entry in the
shared `en.po` and `zh_Hans.po`.

**Formatting the clock.** `MineWorldClockFormat.time_of_day(seconds_into_day, clock, locale)` returns
the time with `hud.time.12h` or `hud.time.24h` and `hud.ampm.am/pm` from the catalog. TW-e extends the
same file with the date entries of step-19 §8.2; SET-a does not add them.

## 3.7 The font (DEP-SET-b)

Godot's default font has no CJK glyphs (P-9). Today Chinese would render only through the operating
system's own font: PingFang on macOS, whatever is installed (or nothing, as tofu boxes) on Linux. The
output would differ by machine, and a capture could not be compared.

`MineWorldText` therefore loads `fonts/NotoSansSC-Regular.otf` at runtime
(`FontFile.load_dynamic_font`, no import) and appends it to the **fallbacks** of the default theme
font. Latin text keeps its current font, glyph for glyph, so `en` rendering does not change. Any glyph
the default font lacks comes from Noto Sans SC before the system is asked. That covers Chinese UI text
and Chinese world content alike. A check (AC-SET-13) proves that every character of every shipped
catalog is in the bundled font.

**Per platform.** The chain is the same on every OS: the default font, then the bundled Noto Sans SC,
then the system. Godot rasterizes with its own FreeType on all three, so a bundled glyph has the
same shape everywhere. Only the last step differs, and shipped text never reaches it (INV-SET-9):

| OS | System fallback (Godot: "the engine automatically uses system fonts as fallback fonts") | Reached for |
| --- | --- | --- |
| Windows | Windows' font set (e.g. Microsoft YaHei for hanzi) | only characters neither bundled font has (rare hanzi in world content, emoji) |
| macOS | CoreText's set (e.g. PingFang SC) | same |
| Linux | fontconfig. Godot: "the set of default fonts shipped on Linux depends on the distribution". A minimal install may have no CJK font. | same. On a bare Linux machine such a character is a tofu box, which is why the bundled font, not the system, carries every shipped string |

H-1 is checked on all three OSes. On Linux it is checked on a machine with no CJK system font
(`fc-list :lang=zh` empty), so that a pass shows the bundled font is doing the work.

## 3.8 The menu (`menu.gd`)

- **Opening.** Esc opens the menu in both clients and closes it again (QSET-2). In 3D, opening it
  releases the mouse; closing it leaves the mouse released until the next click, as today. In 2D,
  "Quit" moves into the menu. The hint and controls lines say "Esc menu".
- **Layout.** A centred panel with tabs: **General** (Language, Clock), **Display** (Window mode,
  Window size or Render scale, VSync, Frame-rate cap, the monitor's refresh rate read-only, the
  measured FPS), later **Controls** (SET-b) and **World** (SET-c). Buttons: Apply, Revert, Close;
  Quit at the bottom. A client passes a `Theme` (2D's warm panel, 3D's light-on-dark) and a capability
  set (`{ "render_scale": true }` for 3D); the menu decides nothing else per client.
- **Language and clock apply at once,** and are saved on Apply. Display changes apply on Apply,
  followed by the confirmation of §3.5.
- **The world keeps running.** The menu does not pause anything (pausing is a host command, SET-c).
  While the menu is open it consumes input: no walk, no move request, no camera look (AC-SET-12).
- **Tab API** for later PRs: `add_tab(key: String, control: Control)`. SET-b and SET-c use it; neither
  edits `menu.gd`'s existing tabs.

## 3.9 SET-b — basic input options (scoped, not designed to the commit)

`overall.md` item 4 lists "basic input options". Proposed (QSET-7, operator-material): a Controls tab
with **mouse-look sensitivity** and **invert vertical look** for the 3D client, persisted in
`[input]`, applied by `player.gd`'s look code through one read of `MineWorldSettings`. The tab also
shows the current key map read-only, in both clients, translated. Key rebinding is deferred, because
it would put the input map in the settings file and both clients build their input maps
differently today (2D at runtime in `app.gd._inputs`, 3D in `project.godot`).

## 3.10 SET-c — the World section (host commands; scoped, not designed to the commit)

- **Module:** `clients/shared/host/` (`host_clock.gd`, `world_tab.gd`), symlinked as
  `res://mineworld_host`. It is the only shared client code allowed to use `HTTPRequest`, and it never
  imports `store.gd`.
- **State shown** comes from the server only: `WorldSummary.paused` and `time_scale` from `welcome`,
  then each `clock { at, time_scale, paused }` frame (TW-c, read through the protocol module's reader
  that S11/TW-c adds). Day length is shown as 4 h / 2 h / 1 h for scales 6 / 12 / 24, and as
  "custom (n×)" for any other scale.
- **Commands:** `POST /admin/clock` with `{ "paused": bool }` or `{ "time_scale": 6 | 12 | 24 }`, and
  `Authorization: Bearer <token>`. The token is handed over by the launcher (`--admin-token=` or
  `MINEWORLD_ADMIN_TOKEN`, step-19 §7.3). It is held in memory only and is never written to the
  settings file, a log or a capture (INV-SET-3).
- **No optimism:** after a command the controls show "pending" until a `clock` frame confirms it. A
  401, 404 or timeout is shown, translated, and the controls return to the server's state.
- **Without a token:** the tab is shown **read-only**, with the current day length and a paused
  badge (QSET-4, recommended over hiding it, because a non-host needs to know why the world stopped).
- **Ownership:** this moves "the World section of the settings menu" out of TW-e's scope and into
  SET-c (QSET-6). TW-e keeps the HUD date and time, `world_time`, the sun and the launcher's
  close behaviour.

## 3.11 Modularity, layers, and the gating questions

- **Kernel, contracts, systems, server, protocol module:** untouched by SET-a and SET-b. SET-c touches
  no server code; it uses TW-c's routes and frame.
- **Change amplification:** adding a language is one `.po` file. Adding a setting is one field in
  `settings.gd`, one control in `menu.gd` and its keys in each catalog. Adding a client is one symlink
  and three calls in its composition root. None of these edits a client scene graph, the protocol
  module or a pack.
- **No layer leakage:** settings are presentation-only and never become an `ActionIntent` or a frame
  (INV-SET-1, AC-SET-4). Host commands go through the host's admin surface, never through the world's
  request path.
- **ENGINEERING_RULES §11** (can a Minecraft-like 3D client use it without kernel redesign?): yes.
  The module is client-local and the 3D client is one of its two users. **§12** (both clients without
  duplicated logic?): yes. Both clients call the same module; the only per-client inputs are a theme
  and a capability set.
- **Reuse question, both halves:** §4. Godot's own `TranslationServer`, `ConfigFile` and
  `DisplayServer` are adopted. Addons are rejected for fit, not because we would rather build. The
  font is adopted.

---

# 4. Reuse comparisons (the operator's standing rule)

## 4.1 Translation

| Option | Licence | Fit | Maturity | Verdict |
| --- | --- | --- | --- | --- |
| **Godot `TranslationServer` + gettext `.po`** | MIT (engine) | Runtime-loadable from any folder with no import (P-1, P-2; 13b's B-14), which is what "add a language without code" needs. Plurals and context (P-6). Standard tooling: Poedit, `msgmerge`, `msgfmt --check`. Diffs cleanly in Git (Godot docs: "gettext files work better with version control systems"). | Built into Godot since 3.x; documented for 4.7 | **Adopt. Already adopted for 2D wording by 13b (ARC-70); this step confirms it for both clients and keeps it as the final format, so 13b's file needs no reformatting.** |
| Godot `TranslationServer` + CSV | MIT (engine) | Plurals and context since 4.6 (Godot 4.7 docs). But a CSV **must be imported** by the editor's importer and listed in project settings; there is no runtime loader (P-3). A user cannot drop one in. One wide file for all languages means every translator edits the same file. | Built in | Reject: it fails the "without code" clause at runtime. |
| Godot `Translation` resources (`.translation`, `.tres`) built in code | MIT | Binary or engine-specific; no translator tooling | Built in | Reject. |
| Our own JSON/YAML catalog + lookup | — | Reinvents plural rules, fallback, locale matching and live notifications that the engine already has | — | Reject (`REUSE_POLICY.md`: commodity). |
| Third-party localisation addons (`marynate/godot.localization`, MIT, 10 stars, last pushed 2023; various editor-only translation editors) | MIT / none | Editor tooling or thin wrappers over `TranslationServer`; nothing our runtime needs | Small, stale | Reject; Poedit or any `.po` editor covers translators. |
| POT extraction from scripts | — | Godot's "Generate POT" is in the editor only; the CLI has no flag for it (`godot --help`, 4.7.2). With symbolic keys the template is `en.po` without its translations, so a check generates `messages.pot` from `en.po`, and a scan proves every key used in code is in `en.po`. | — | Generate from `en.po` in the catalog check; no external tool. |

## 4.2 Settings persistence

| Option | Licence | Fit | Maturity | Verdict |
| --- | --- | --- | --- | --- |
| **Godot `ConfigFile` in `user://`** | MIT | INI-like, typed `Variant` values survive a round trip (P-8). Human-readable. Per user by construction. Maaack's template uses the same approach (`PlayerConfig`, `user://player_config.cfg`). | Built in, widely used | **Adopt**, with our own atomic write and validation. |
| `ProjectSettings` overrides (`override.cfg`) | MIT | Read once at start from the project or executable folder. In a checkout that is `clients/2d/override.cfg`: per checkout, not per user, and inside the tree. It cannot change live. | Built in | Reject. |
| JSON through `FileAccess` | MIT | Loses integer types (numbers come back as floats) and needs our own schema code anyway | Built in | Reject. |
| A custom `Resource` saved with `ResourceSaver` (`.tres`) | MIT | Typed, but loading a `.tres` from a user-writable folder can instantiate embedded scripts | Built in | Reject on safety. |
| OS-native preferences (macOS `defaults`, the Windows registry) | — | Not reachable from GDScript without native code; three back ends | — | Reject. |

## 4.3 Settings-menu addons

| Option | Licence | Fit | Maturity | Verdict |
| --- | --- | --- | --- | --- |
| Maaack's Godot Menus Template / Game Template (`Maaack/Godot-Menus-Template`, MIT, 484 stars; `…-Game-Template`, MIT, 1,688 stars; both pushed 2026-09-10; targets 4.7) | MIT | Main, options and pause menus as scenes, an editor plugin with a setup wizard, its own scene loader. `app_settings.gd` offers fullscreen as a bool and VSync. It has no borderless/exclusive distinction, no frame cap, no render scale and no locale setting. Its keys are untyped `StringName`s. Our clients have no editor step and build their UI in code (A-SET-10). | Active, popular | **Reject adoption; borrow the pattern** (ConfigFile + static apply helpers), credited in DEP-SET-a. Adopting it would force its scene and plugin flow into two code-built clients. |
| GGS — Godot Game Settings (`zijcht/godot-game-settings`, MIT, 523 stars, 3.3.0 "updated for Godot 4.5") | MIT | Display, audio and input settings with UI components; plugin settings saved as a resource; editor-plugin oriented; no localisation | Active | Reject: editor-centric, saves resources (see 4.2), and it would still need our window-mode, cap and i18n work. |
| Build our own small module | — | About six short scripts over three engine APIs, typed, with no editor step | — | **Build**, because the engine supplies the substance and the addons supply mostly the flow we do not want. |

## 4.4 A font for Simplified Chinese (DEP-8)

| Option | Licence | Size | Fit | Verdict |
| --- | --- | --- | --- | --- |
| **Noto Sans SC Regular, static OTF** (`notofonts/noto-cjk`, `Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`) | **OFL-1.1** (the repository's `Sans/LICENSE`, verified) | **8,331,336 bytes** (GitHub API, 2026-10-08) | Designed as a UI face; pairs with Latin sans; full GB 18030 coverage of common hanzi, so Chinese world content renders too | **Recommended**, unmodified, with its `OFL.txt`, under an explicit DEP-8 carve-out (QSET-1). |
| Noto Sans SC, variable (`google/fonts`, `ofl/notosanssc/NotoSansSC[wght].ttf`) | OFL-1.1 | 17,772,300 bytes | Every weight in one file | Reject: twice the size for weights the UI does not use (bold can be synthesized with `FontVariation.variation_embolden`). |
| Noto Sans SC subset (fontTools `pyftsubset`, MIT, to GB 2312's 6,763 hanzi + Latin) | OFL-1.1 (a Modified Version; Noto declares no Reserved Font Name that the subset would use; to be re-verified) | ~2–3 MB, estimated, to be measured | Smaller; but a build step, and rare characters in world content become tofu | Alternative if the operator rejects the 8 MB. |
| Source Han Sans SC | OFL-1.1, Reserved Font Name "Source" | similar | Identical glyphs to Noto CJK | No advantage. |
| WenQuanYi Micro Hei | Apache-2.0 **or** GPL-3.0 with font exception (licence to be verified from the upstream file before adoption) | ~5 MB, to be measured | Passes ARC-55's list through its Apache branch; older design, one weight | **Fallback** if OFL is refused. |
| Droid Sans Fallback | Apache-2.0 (to be verified) | ~4 MB, to be measured | Passes ARC-55; dated, uneven at UI sizes | Second fallback. |
| The OS's own fonts (Godot's system fallback, P-9) | the OS's | 0 | Different on every machine; tofu on Linux without CJK fonts; captures cannot be compared | Reject as the mechanism; it stays the last resort after the bundled font. |
| LXGW WenKai, Sarasa Gothic | OFL-1.1 | various | Kai calligraphic style; a programmer's mono-width blend | Not UI faces for this style. |

Every OFL option fails DEP-8's relicensing test as written: OFL forbids distributing the font under
any other licence. The proposed carve-out is therefore narrow. A font file may enter the repository
under OFL-1.1 when it is unmodified (or a subset, if QSET-1 chooses that), it sits beside its
`OFL.txt`, `NOTICE` names it as not covered by MIT, and it is never embedded into a file that is under
MIT. This is the ordinary way OFL fonts are bundled with software (OFL §1: "may be bundled, embedded,
redistributed and/or sold with any software").

### 4.4.1 The DEP-8 font exception (ruled by the operator under QSET-1; text for `DECISIONS.md`)

The primary session adds this text, word for word, to `docs/DECISIONS.md` as an amendment to DEP-8
when SET-a lands:

> **Amended 2026-10-08 — the font exception (operator ruling QSET-1, S20).** DEP-8's binding test is
> whether MineWorld may relicense an asset under MIT. Fonts under the SIL Open Font License 1.1
> cannot pass it, because OFL-1.1 requires the font to stay under OFL. They are the only exception to
> the test, under every one of these conditions:
>
> 1. **Fonts only.** The exception covers font software (`.otf`, `.ttf`, `.otc`, `.woff2`) licensed
>    under `OFL-1.1` and nothing else: no texture, model, sound, code or data file, and no other
>    licence.
> 2. **Bundled, never sold on its own.** The font ships only inside MineWorld, as a file the clients
>    load, and is never offered, sold or distributed as a product on its own (OFL-1.1 §1).
> 3. **Unmodified and named as upstream.** The file is the upstream release, byte for byte. Its
>    source URL, version and SHA-256 are recorded beside it. A modified or subset font is not covered
>    without a new decision, which would also have to respect any Reserved Font Name (OFL-1.1 §3).
> 4. **Its licence travels with it.** `OFL.txt`, with the upstream copyright notice, sits in the same
>    folder (OFL-1.1 §2). `NOTICE` names the font, its copyright holder and its licence, and states
>    that MineWorld's MIT licence does not cover it.
> 5. **Never merged into MIT material.** The font is never embedded into, or concatenated with, a
>    file under MIT. Loading it at runtime is use, not merging.
> 6. **Listed.** Every font admitted under this exception is a row of DEP-8's table. At present
>    there is one: **Noto Sans SC Regular** (`notofonts/noto-cjk`,
>    `Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`, 8,331,336 bytes, OFL-1.1, copyright Adobe with
>    Reserved Font Name "Source"), the CJK fallback font of both reference clients' UI (S20 SET-a).
>
> ARC-55's default pack allow-list is unchanged. A **pack** that carries an OFL font still needs a
> world-level policy that allows `OFL-1.1` (ARC-55 point 5). This exception covers fonts bundled with
> MineWorld's own clients.

## 4.5 Display control

| Option | Fit | Verdict |
| --- | --- | --- |
| **`DisplayServer` + `Engine.max_fps` + `Viewport.scaling_3d_scale`** | Every requirement except changing the monitor's mode, which Godot does not expose | **Adopt.** |
| Project settings (`display/window/size/*`, `display/window/vsync/vsync_mode`, `application/run/max_fps`) | Read at start only; a change needs a restart and a write to `override.cfg` (4.2) | Reject as the mechanism. |
| Native code to change the monitor's resolution or refresh rate | GDExtension per OS; risky (leaves the desktop in the wrong mode on a crash) | Reject (QSET-3). |

---

# 5. Invariants (INV-SET-n)

1. **INV-SET-1 — Settings never reach the server.** No file under `clients/shared/settings/` names
   `MineWorldClient`, `HTTPRequest`, `WebSocketPeer`, `StreamPeer`, `PacketPeer`, `submit`,
   `submit_affordance` or `connect_to_world`. A client's frames to the server are the same with any
   settings as with the defaults.
2. **INV-SET-2 — The settings file holds only settings.** No invite, admin token, seat, server
   address, nickname, world id or save path is ever written to it.
3. **INV-SET-3 — Host commands are not settings.** Pause and day length are never stored in the
   settings file; `clients/shared/host/` never imports `store.gd`; the admin token lives in memory
   only.
4. **INV-SET-4 — Every client UI string comes from a catalog.** No user-visible string in either
   client's UI is a code literal, apart from an allow-list where each entry gives its reason and an
   entry that admits nothing fails. World content and signage art are not UI strings.
5. **INV-SET-5 — Harnesses are independent of the user's settings.** Every scripted or capture mode
   runs on defaults unless it passes its own `--settings=` file; a developer's saved settings never
   change a test, a capture or evidence.
6. **INV-SET-6 — Logs stay English.** Stdout, `[link]`, `drive:`, `EVIDENCE`, `[PASS]`/`[FAIL]`
   lines and warnings are identical whatever the language.
7. **INV-SET-7 — `en` is unchanged.** In `en`, every pre-existing UI string reads as it reads on
   `main`, apart from the changes SD-SET-a-12 lists by name. Latin glyphs come from the same font as
   before.
8. **INV-SET-8 — One module, two clients.** Both clients take `clients/shared/settings` by symlink,
   and neither has its own copy of a setting, a catalog entry or a display call.
9. **INV-SET-9 — Determinism of text rendering.** Every character of every shipped catalog has a
   glyph in a bundled font; the system font is never needed for shipped text.
10. **INV-SET-10 — World content is never translated.** No name, place, tag or dialogue from an
    observation passes through `tr()`.

---

# 6. PR split

```text
SET-a  ──▶  SET-b
  │
  └──────▶  SET-c   (also needs S11-D's admin surface and TW-c's /admin/clock + clock frame)
TW-e (S19) uses SET-a's clock formatter, catalogs and [launcher] section.
```

| PR | Scope | Depends on | Integration checkpoint | Adversarial criteria (fixed before measuring) |
| --- | --- | --- | --- | --- |
| **SET-a** — settings module, language, display, clock, menu, in both clients | §3.1–§3.8: `clients/shared/` project and `settings/` module; catalogs `en`, `zh_Hans`; Noto Sans SC; the menu with General and Display; both clients wired (every UI string through the catalogs, Esc opens the menu, the 2D HUD clock honours 12h/24h, the 3D HUD gains the same time line when connected); launchers stop forcing `--resolution` in play; static scan and catalog check in `cargo test`; runtime checks as `#[ignore]` Godot tests; DECISIONS ARC-SET-a/b, DEP-SET-a/b; `SETTINGS.md`. Full design §12. | **13b merged** (its `words.gd`, keys and pack `en.po` are SET-a's base; 13a #82 and 16a #79 already merged). Scheduled when no S12 or S14 PR is editing `app.gd`, `hud/status.gd`, `slice_main.gd`, `slice_link.gd` or `controls_hud.gd` (QSET-12). | CP-SET-a: `./mineworld-2d` hosting market-town: Esc → menu → 简体中文 → every HUD and menu string turns Chinese at once; clock 24h → `第1天 14:05`-style line; window → 1280×720 → Apply → keep. Quit. `./mineworld-slice --world`: opens in 简体中文 at 1280×720 with the same clock setting. Quit both, relaunch 2D: still Chinese. Run on macOS at the gate, and by the operator on Windows and Linux (§8). | AC-SET-1 … AC-SET-16 (§7). |
| **SET-b** — basic input options | §3.9: Controls tab (3D mouse sensitivity, invert Y; read-only key map in both clients) through the tab API; `[input]` section; `player.gd` reads it. | SET-a | CP-SET-b: in 3D, sensitivity 0.5× halves the yaw per mouse count, measured by the probe from a scripted mouse motion; invert Y flips pitch sign; settings survive restart. | (1) A frame capture with sensitivity changed equals the default run's frames, apart from the positions walked. (2) Mutation: apply sensitivity twice (squared) → the yaw-per-count measurement fails. (3) The 2D client offers no 3D-only control (capability set). |
| **SET-c** — the World section (host commands) | §3.10: `clients/shared/host/`, World tab, token hand-over from both launchers, `clock` frame reading, pending/confirmed states, translated errors. | SET-a; **S11-D** (admin surface); **TW-c** (`/admin/clock`, `clock` frame, `WorldSummary.paused`) | CP-SET-c: single-player 2D: World → Pause → HUD shows `Paused` only after the `clock` frame arrives; `/status` `at` frozen for 10 s; day length 1 h → `at` advances 240 ± 24 s in 10 wall s; a second client joined by invite without a token sees the tab read-only and the paused badge, and its buttons are absent. | (1) INV-SET-3: after a session that paused and rescaled, the settings file is byte-identical to before, and grepping it, the logs and the captures for the token finds nothing. (2) Without a token, no HTTP request is made (no `HTTPRequest` node created). (3) Mutation: show "paused" on click instead of on the frame → a stub that never confirms makes the test fail. (4) A 401 from the stub is shown translated, and the controls return to the server's state. |

---

# 7. Adversarial criteria, fixed before measuring (SET-a)

Each criterion states its oracle, and each mutation that must turn it red. A criterion with no run is
`INCONCLUSIVE`, never `PASS`.

| Id | Criterion | Oracle | Mutation that must fail it |
| --- | --- | --- | --- |
| **AC-SET-1** | **Switching language changes every visible string, live.** In each client, with the menu open over a connected HUD (2D: market-town; 3D: `--world`), a tree walk records the text of every visible text-bearing `Control` (`Label`, `Button`, `OptionButton` and its items, `CheckBox`, `TabBar` titles, `RichTextLabel`) not marked as world text. After `set_language("zh_Hans")` with no restart, every recorded text has changed and contains at least one CJK character, apart from an allow-list of strings identical in both languages, each with its reason (e.g. "VSync", "FPS"; an entry that matches nothing fails). Switching back to `en` reproduces every original text exactly. | the tree walk, from inside the client, printed as `EVIDENCE` | A label composed once in `_ready` with no `NOTIFICATION_TRANSLATION_CHANGED` handling. |
| **AC-SET-2** | **No hardcoded UI string survives at runtime.** Under a test-only marker catalog `xx` (generated by the check from `en.po`, each msgstr wrapped as `⟦…⟧`), every visible non-world text control's text contains `⟦`. | the same tree walk | A planted `label.text = "Hello"` in either client. |
| **AC-SET-3** | **No hardcoded UI string remains, by static scan.** For `clients/2d/`, the scan is 13b's R6 in `check_client_rules.py`, which SET-a keeps green and does not duplicate. For every `*.gd` under `clients/3d-spike/scripts/` (excluding `tools/`) and `clients/shared/`, it is `tests/acceptance/tests/client_text.rs`, which runs in CI: no string literal containing a letter is an argument of a text sink (`.text =`, `.tooltip_text =`, `add_item(`, `set_item_text(`, `add_tab(`, `note(`, `toast(`, `caption(`, `add_line(`, `set_tab_title(`, and a shown-text argument of `_say(`), unless it is a key inside `tr(`, `tr_n(` or a `MineWorldText` call, or is admitted by the allow-list with its reason. | the scan, naming file and line | A planted `status.note("Hello")`; a planted `_say("seated")` with no key. |
| **AC-SET-4** | **Catalogs are complete and consistent across layers** (same test file; reads the shared layer and every `presentation/*/*/i18n/` layer). Every literal key used in code (both clients, the shared module) exists in the `en.po` of the layer the client loads; in each layer, every `en.po` key exists in that layer's `zh_Hans.po` with a non-empty msgstr; the named placeholders of every entry are the same set in both languages; no `en.po` key is unused, apart from the data-built families (`action`, `reason`, `state`), whose members must cover every action type a System Pack declares and every refusal code and rejection reason `server/PROTOCOL.md` lists; no key is defined in two layers unless the later one marks it `#. override`; `messages.pot` equals the template generated from the shared `en.po`. | the test | Delete one `zh_Hans.po` entry; rename `{target}` to `{who}` in one translation; add a refusal code to `PROTOCOL.md`'s table without a catalog entry; duplicate an `action.*` key in the 2D pack without the override mark. |
| **AC-SET-5** | **Settings never reach the server.** 2D: the same `--drive=settings` scenario runs twice against the recording stub, once on defaults and once with `zh_Hans`, 24h, borderless, 30 fps written to its `--settings=` file. The two sequences of frames the stub received are equal after replacing request tokens. 3D: the same with `--world --settings` against a real server, comparing the requests the module's `submitted_request` signal reports. Static half: INV-SET-1's identifier scan over `clients/shared/settings/`. | the stub's record; the request transcript; the scan | A planted `client.submit("move", …)` whose payload carries the locale (the frames differ); a planted `HTTPRequest` in `store.gd` (the scan). |
| **AC-SET-6** | **The settings file holds only settings (INV-SET-2).** After a 2D and a 3D session started with a known invite `INVITE-PROBE-7f3a`, nickname `nick-probe`, and a seat, every one of those strings is absent from the settings file, which parses into exactly the keys of §3.3. | file content | The store writes the parsed command-line options into `[general]`. |
| **AC-SET-7** | **Persistence, across both clients.** Apply `zh_Hans` + 24h + 1280×720 in 2D; quit; launch the 3D client on the same user directory: language, clock and size are in effect **before its first frame is drawn**, recorded by `DisplayServer.window_get_size()` and `TranslationServer.get_locale()` in the first `_process`. | the probe's `EVIDENCE` line | Apply settings after the first frame; a per-project user directory. |
| **AC-SET-8** | **A bad file never breaks a client.** For each of: unparsable bytes; `window_mode = 7`; `language = "tlh"` (no catalog); `max_fps = "fast"`; `[meta] version = 99`: the client starts on defaults (per key for type and range errors), prints one warning naming the file and the key, and leaves the file byte-identical until an Apply. On Apply the old content goes to `settings.cfg.bak`. | file bytes before and after; stderr | Overwrite on load. |
| **AC-SET-9** | **Harnesses ignore the user's settings (INV-SET-5).** With a planted user file (`zh_Hans`, borderless, 30 fps, 24h) in the shared user directory: 2D `--drive=walk`, 3D `--world --link` and `--drive` produce the same `[PASS]`/`EVIDENCE` lines as without it, in English, and the 2D `--capture` stills are byte-identical. | evidence and still bytes | A harness mode that does not force `--settings=none`. |
| **AC-SET-10** | **Logs stay English (INV-SET-6).** `--world --link` under `--settings=<file with zh_Hans>` prints the same `[link]` lines as under `en`. | stdout diff | Translate inside `_say` before printing. |
| **AC-SET-11** | **Display settings take effect.** Windowed run, `--quit-after`: size 1280×720 → `window_get_size() == (1280, 720)`; VSync OFF and cap 30 → mean frame rate over 5 s ≤ 31; cap 0 and VSync OFF → mean > 31 on the reference scene; `window_get_vsync_mode()` equals the set mode; in 3D, render scale 67 → `scaling_3d_scale == 0.67`. Headless: applying every setting is a no-op without error. | `EVIDENCE` lines | Drop the `Engine.max_fps` call. |
| **AC-SET-12** | **An open menu takes gameplay input.** With the menu open, a scripted WASD hold of 2 s and a click on a walkable point send no request (2D stub record; 3D request transcript), and the 3D camera yaw does not change under scripted mouse motion. | stub record; transcript; yaw | Let `_unhandled_input` walk while the menu is open. |
| **AC-SET-13** | **Every shipped glyph is bundled (INV-SET-9).** A check loads the bundled font with `allow_system_fallback = false` and asserts `has_char` for every character of every msgstr in every shipped catalog, plus the digits and punctuation the formats use. | `clients/shared/checks/glyph_check.gd` | Add a catalog entry using a character outside the font (e.g. an emoji). |
| **AC-SET-14** | **A language can be added without code.** A scratch `user://locale/xx_test.po` with two entries and `language.self_name` is listed in the language selector; choosing it shows its two strings and English for every other key (never a raw key). | the menu's item list; the tree walk | Hardcode the language list. |
| **AC-SET-15** | **The shared settings folder resolves as §3.4 states, on the OS it runs on.** `clients/shared/checks/store_check.gd` prints `OS.get_name()`, `OS.get_user_data_dir()`, the rendering driver and the display server, and asserts that the folder's last component is `MineWorld`. It also asserts that a file saved by one project (`clients/2d`) is read by the other (`clients/3d-spike`). Run on macOS at the gate; the same command is H-10 on Windows and Linux. | the printed line | Drop `custom_user_dir_name` from one project. |
| **AC-SET-16** | **No platform branch in the module, apart from the declared one.** `client_text.rs` finds no `OS.get_name()`, `OS.has_feature("windows"/"macos"/"linuxbsd"/"x11"/"wayland")` or `DisplayServer.get_name()` comparison under `clients/shared/settings/`, apart from `display.gd`'s Wayland tooltip, admitted by name. Platform behaviour comes from Godot, not from our branches. | the scan | A planted `if OS.get_name() == "Windows":` in `store.gd`. |

---

# 8. Visual checks the operator must do by hand

These cannot be decided by a script. Each is a short look, judged by the operator, with the result
recorded in the PR together with the platform line `display.gd` prints (OS, rendering driver,
display server).

**What CI covers, and what it cannot.** CI is a Linux container without Godot (A-SET-12). The checks
that run there are platform-independent by construction: the static text scan (AC-SET-3), the
catalog consistency check (AC-SET-4) and the no-platform-branch scan (AC-SET-16). They hold on every
OS because they read files, not a running engine. Glyph coverage (AC-SET-13) is also
platform-independent, because it reads the bundled font with system fallback off. It runs under
Godot at the gate on macOS, and its verdict carries to the other OSes, since the font and the catalogs
are the same bytes everywhere. Everything rendered or windowed needs the real OS. **So every
Windows and Linux entry below is an operator checklist item**, and SET-a's gate runs the macOS
column.

**Prerequisites on Windows.** The clients take the shared modules by symlink. A Windows checkout
needs `git config core.symlinks true` with Developer Mode or administrator rights, or the module
folders appear as text files (R-SET-10). The launchers are bash scripts and run from Git Bash. If a
symlink is missing, both launchers stop with a message naming the fix (SD-SET-a-18).

| Id | What to look at | Pass | macOS | Windows (D3D12, and Vulkan with `--rendering-driver vulkan`) | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- | --- | --- |
| H-1 | `zh_Hans`, 2D and 3D: HUD, toasts, captions, door labels, the whole menu | crisp at the default size, no tofu, no mixed faces in one line | Retina and an external 1× display (gate) | at 100 % and 150 % display scaling (checklist) | on a machine with `fc-list :lang=zh` empty (checklist) | at a fractional scale such as 125 % (checklist) |
| H-2 | menu layout in both languages | nothing clipped or overflowing | gate | at 150 % scaling (checklist) | — (same font and layout as macOS) | — |
| H-3 | windowed → borderless → fullscreen → windowed, both clients | each mode as §3.5.1 says for that platform; back in windowed with the same size (and position, except on Wayland) | Cmd-Tab in each; no empty Space left behind (gate) | Alt-Tab in each; at most a brief black flash on exclusive (checklist) | exclusive bypasses the compositor; no stuck full screen after Alt-Tab (checklist) | Fullscreen behaves as Borderless and the tooltip says so (checklist) |
| H-4 | a window size change and the 15 s confirmation | the countdown shows; no answer reverts; an answer persists across a restart | gate; judge whether presets should be logical points on Retina (§3.5.1) | checklist | checklist | checklist |
| H-5 | 3D: VSync Off/On/Adaptive, cap 30/60/none, render scale 50–100 % | tearing with Off on a panning shot and none with On; 30 fps is capped (the Display tab's FPS confirms it); 50 % visibly softer while the HUD stays sharp | gate | D3D12 and Vulkan (checklist) | checklist | record whether VSync Off is honoured or the compositor syncs anyway (checklist) |
| H-6 | 12h and 24h × `en` and `zh_Hans`, both clients in one world | same time, same form (`7:42 PM` / `下午 7:42`; `19:42`) | gate | — (no platform dependency) | — | — |
| H-7 | Esc behaviour | Esc opens and closes the menu; 2D Quit in the menu; 3D mouse released while open and recaptured on the next click | gate | checklist | checklist | checklist (pointer capture under Wayland) |
| H-8 | 3D golden-hour slice in `en` | exactly as accepted (ARC-13) | gate (the accepted baseline) | checklist (a sanity look, not a pixel match) | — | — |
| H-9 | relaunch: language chosen in 2D | the 3D client opens in it | gate | checklist | checklist | — (same code path as X11) |
| H-10 | the shared settings folder (AC-SET-15's command) | resolves to §3.4's path, and 2D and 3D share the file | gate | `%APPDATA%\MineWorld\settings.cfg` (checklist) | `~/.local/share/MineWorld/settings.cfg`, and `$XDG_DATA_HOME/MineWorld/` when set (checklist) | — (same as X11) |

---

# 9. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-SET-1 | SET-a edits files the S12 and S14 lanes also edit (`app.gd`, `hud/status.gd`, `slice_main.gd`, `slice_link.gd`, `controls_hud.gd`). | Schedule in a window with no open PR on them (QSET-12). The text changes are mechanical, so a rebase is cheap. Fallback: split into SET-a1 (module + 2D) and SET-a2 (3D). |
| R-SET-2 | Tests and probes parse printed lines; translating them breaks the harness. | INV-SET-6 and AC-SET-10. `_say` splits log from shown text. |
| R-SET-3 | Godot's auto-translation turns a world name that equals a key into a translation. | Keys are dotted and lowercase, and world-text nodes set `auto_translate_mode = DISABLED` (SD-SET-a-9). |
| R-SET-4 | macOS full-screen semantics (Spaces, exclusive mode hiding the Dock and menu bar) surprise the player. | H-3; names in the menu describe what the player sees; the revert timer. |
| R-SET-5 | 8.3 MB font in Git history, and OFL in an MIT repository. | QSET-1 (operator). Subset or an Apache-licensed fallback are the alternatives. |
| R-SET-6 | Two clients writing one file at once. | Re-read before write, section-wise merge, atomic rename (§3.4). Last Apply wins, and that is documented. |
| R-SET-7 | `check_client_rules.py` R4 (`within`) or the action-literal scan trips on shared code reached through the new symlink. | Shared code avoids both words as literals. SET-a verifies the scans against the symlink; Python's `rglob` symlink behaviour is checked, not assumed. |
| R-SET-8 | Chinese strings are longer or shorter than English and break HUD layout. | Containers size to content; H-2. |
| R-SET-9 | P-5: a `zh_TW` system locale is shown Simplified Chinese. | Moot while the default is always `en` and the user picks explicitly (QSET-8). Recorded in `SETTINGS.md`. |
| R-SET-10 | Windows checkouts without `core.symlinks` see a text file instead of the folder. This is now in scope, because every platform must work (§1.5). | The same is already true of `mineworld/`. SET-a documents the Windows setup in both client READMEs and `SETTINGS.md`, and both launchers stop with a clear message when a module folder is not a directory (SD-SET-a-18). Replacing symlinks with a copy step, or packaging the clients for export, is a deployment question for S13, raised there and not solved in SET-a. |
| R-SET-13 | Platform differences appear only on the OS that has them, and CI cannot see them. | §3.5.1 states the expected behaviour per platform; the checklist in §8 has an entry for each; the platform line `display.gd` prints makes every report traceable. |
| R-SET-14 | The Windows atomic replace of `settings.cfg` behaves differently from POSIX rename. | §3.4: verified in C2; the remove-then-rename fallback keeps `.bak`. |
| R-SET-12 | 13b and SET-a disagree on wording mechanics: where English lives, the fallback, the scans. | §1.4 and §3.6 keep 13b's format, keys, fallback and R6. The only change to 13b's material is the verbatim move of `action.*`/`reason.*` into the shared layer (SD-SET-a-17), checked key for key. SET-a starts after 13b merges. |
| R-SET-11 | Pseudo-localization or the marker catalog leaks into a shipped build. | The marker catalog is generated into a scratch folder by the check and passed with `--extra-locale=`; it is never in `settings/locale/`. |

---

# 10. Questions (QSET-n). **[OPERATOR]** marks operator-material ones; the primary session may rule the rest.

**All ruled on 2026-10-08 (§1.5).** QSET-1, -2, -3 and -7 were ruled by the operator as recommended.
QSET-1 also fixed the exception's terms: fonts only, OFL-1.1, bundled and never sold on its own
(§4.4.1). QSET-4 … -6 and -8 … -16 were ruled by the primary session as recommended. The table keeps
the questions as they were asked.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QSET-1 [OPERATOR]** | **The Chinese font and its licence.** (a) Noto Sans SC Regular, 8.3 MB, OFL-1.1, unmodified, under a narrow DEP-8 carve-out for fonts (§4.4); (b) the same, subset to GB 2312 (~2–3 MB, a build step, rare characters missing); (c) WenQuanYi Micro Hei through its Apache-2.0 branch (passes ARC-55, older design); (d) no bundled font, relying on the OS (fails INV-SET-9, tofu on bare Linux). | **(a).** OFL is the normal licence for open fonts and the carve-out is narrow; (c) if OFL is refused. |
| **QSET-2 [OPERATOR]** | **Esc opens the menu in both clients.** That changes accepted controls: 2D's "Esc quits" (Quit moves into the menu) and 3D's "Esc releases the mouse" (opening the menu releases it). The alternative is another key (F10, or `` ` ``) plus an on-screen gear button, keeping Esc as it is. | **Esc**, the convention players expect; the menu holds Quit. |
| **QSET-3 [OPERATOR]** | **What "刷新率 分辨率" means in practice.** Godot cannot change the monitor's refresh rate or resolution. Proposed: refresh rate = a frame-rate cap (including "match display") plus VSync, with the monitor's rate shown read-only; resolution = the window size in windowed mode, and a 3D render scale in the two full-screen modes. | As proposed. Changing the monitor's mode needs native code and is rejected (§4.5). |
| QSET-4 | Non-hosts and the World section: read-only or hidden? (S19 §15.2 left it to this PR.) | **Read-only**, so every player sees why the world stopped. |
| QSET-5 | One settings file for both clients (shared user directory "MineWorld"), or one per client? | **One**: one player, one machine, one language. |
| QSET-6 | Move "the World section of the settings menu" from TW-e to this step's SET-c? | **Yes**: one owner for the menu. TW-e keeps the HUD date and time, the sun and the launcher's close behaviour. |
| **QSET-7 [OPERATOR]** | **"Basic input options"** (`overall.md` item 4): is SET-b's scope enough? 3D mouse sensitivity and invert Y, plus a read-only key map in both clients; key rebinding deferred. | Yes; rebinding after both clients share one input-map source. |
| QSET-8 | The default language: always `en`, or the OS language when a catalog exists? | **Always `en`**, the operator's words; the player switches once. |
| QSET-9 | Does the 3D promenade (`./mineworld-3d`) get the menu? | **No.** It is a preserved spike; its shared `ControlsHud` lines become translatable as a side effect, and it runs on defaults. |
| QSET-10 | In-world signage (the café's name, the street's signs) is art drawn as text. Translate it? | **No**: it is Presentation Pack art, like a texture; a pack may localize its signage later. It is allow-listed in the scan with this reason. |
| QSET-11 | Where languages come from: the three layers of §3.6 (shared module, the Presentation Pack's `i18n/` as 13b set it up, `user://locale/`)? | **Yes.** The 3D client gains its pack layer with S14 16f. |
| QSET-12 | SET-a as one PR over both clients, or SET-a1 (module + 2D) then SET-a2 (3D)? | **One**, because the checkpoint is "one module, two clients". Split only if a lane collision forces it (R-SET-1). |
| QSET-13 | Symbolic keys or English source text as msgid? | **Settled by 13b's ARC-70: symbolic keys**, which this step keeps; step-19 §8.2 uses them too. Recorded so it is not reopened. |
| QSET-16 | Move 13b's `action.*` and `reason.*` entries from the 2D pack's `en.po` into the shared layer, so both clients use one translation, or leave them in the 2D pack and give the 3D client its own copy? | **Move**, verbatim and with the same keys (SD-SET-a-17); a pack may still override any of them. Two copies would let the clients' wording for one action drift apart. |
| QSET-14 | The static scan and catalog check in Rust (`tests/acceptance`, runs in CI, reusing `client_rules.rs`'s GDScript lexer), or Python in `scripts/`? | **Rust**, so it blocks in CI. The lexer moves to a shared test module, unchanged. |
| QSET-15 | Show measured FPS in the Display tab? | Yes; it is what makes H-5 checkable. |

---

# 11. Proposed decision records and amendments (for the primary session to apply)

## 11.1 `docs/DECISIONS.md` (placeholder ids)

- **ARC-SET-a — Client settings are a shared, presentation-only client module.**
  `clients/shared/settings`, symlinked into each client as `res://mineworld_settings`; no network
  code; one per-user `ConfigFile`; settings never reach the server. Host commands (pause, day length)
  live in `clients/shared/host` and are never stored. Rejected: settings in the protocol module;
  settings as a server-side or world-side concept; an autoload.
- **ARC-SET-b — a dated note under ARC-70 (13b), not a new number, if the primary session prefers.**
  "Extended to both clients by S20: catalogs are layered per key (shared module → Presentation Pack →
  user), languages are discovered from the `.po` files present, the language is switched live, the
  fallback is English and then the readable key, and world content and logs are never translated.
  `action.*` and `reason.*` live in the shared layer."
- **DEP-SET-a — `ConfigFile` and `DisplayServer` adopted; `ProjectSettings` overrides, JSON, `.tres`,
  Maaack's template and GGS rejected,** with the comparisons of §4.2, §4.3 and §4.5, crediting Maaack's
  `PlayerConfig` pattern. The translation comparison of §4.1 confirms ARC-70's adoption and is
  attached to the ARC-70 note.
- **DEP-SET-b — Noto Sans SC Regular (OFL-1.1) bundled as a font fallback**, subject to QSET-1; the
  DEP-8 amendment below.
- **DEP-8 amendment (fonts).** "A font may be bundled under OFL-1.1, unmodified (or subset, if so
  decided), beside its `OFL.txt`, named in `NOTICE` as not covered by MIT, and never merged into an
  MIT-licensed file. This is the only exception to the relicensing test, and it is for fonts only."
  The DEP-8 table gains the row: "Noto Sans SC (notofonts/noto-cjk) — OFL-1.1 — the CJK fallback of
  both clients' UI".

## 11.2 `overall.md`

- "Parallel build-out": add **S20 — In-game client settings** with the document
  [`step-20-client-settings.md`](step-20-client-settings.md) and the PR line `SET-a → SET-b; SET-a +
  S11-D + TW-c → SET-c`. SET-a runs now, in its own worktree, when no S12 or S14 PR edits the client
  files of R-SET-1.
- "Framework, not demo" item 4: add, after the requirement: "**Designed in
  [`step-20-client-settings.md`](step-20-client-settings.md).** Pause and day length are host commands
  in the menu's World section (SET-c), never settings (S19 §15.2). 'Refresh rate' is a frame-rate cap
  plus VSync, and 'resolution' is the window size or, in full screen, a 3D render scale (QSET-3).
  'Basic input options' is SET-b."  Also apply step-19 §15.2's text, if the primary session has not
  yet.
- Decision numbers: S20 needs two ARC and two DEP numbers (ruling 6).
- **A new cross-cutting requirement (operator, 2026-10-08):** "我们要保证支持全平台，mac linux
  windows都可以" (every platform must be supported, macOS, Linux and Windows). Proposed record in
  `overall.md`, binding on every client and deployment step: designs state their per-OS behaviour;
  hand checks name their OS; whatever CI cannot run on an OS becomes an operator checklist item. S20
  applies it in §1.5, §3.4, §3.5.1, §3.7 and §8. The symlinked shared client modules on Windows
  (R-SET-10) are raised to S13 as a deployment question.

## 11.3 `step-19-time-weather.md`

- §11.2 TW-e row: remove "the World section of the settings menu (pause, day length) via
  `/admin/clock`" (moved to SET-c, QSET-6). Add: "uses SET-a's `MineWorldClockFormat`, catalogs and
  the store's `[launcher]` section".
- §8.2: "the settings planning lane chooses `.po` or CSV" → "`.po` with symbolic keys (S20
  ARC-SET-b)".
- §12, row "Settings-menu PR": → "S20 SET-a (and SET-c for the World section)".

## 11.4 `step-13-client-2d.md` and `step-15-demo-3d.md`

- A one-line note in each: "Every UI string goes through `clients/shared/settings` (S20); Esc opens
  the settings menu (QSET-2)". `step-13` §14's AC-W list is unaffected; the 2D README's "Esc quits"
  becomes "Esc menu".

## 11.5 `clients/protocol/ADOPTION.md`

- §1 gains one sentence: "Other shared client modules live in `clients/shared/` and are taken the same
  way; see `clients/shared/SETTINGS.md`." (Lands in SET-a's C1; it is a cross-reference, not an edit
  to the protocol module.)

---

# 12. PR SET-a — the settings module, language, display, clock and menu, in both clients

**`DESIGN FROZEN 2026-10-08 (primary session; operator rulings QSET-1, -2, -3, -7 and the all-platform
requirement of the same date, §1.5)`**

Frozen means the scope (§12.1), the design decisions (§12.3), the acceptance (§12.6, §7, §8) and the
execution contract (§12.9) are frozen. Progress, evidence, audit findings and bounded corrections stay
writable. This section is SET-a's single PR design authority and its ledger. **Implementation starts
only after S12 13b merges**, from the `main` that contains it, in a fresh session.

## 12.1 Identity, base, scope

- Working name **SET-a**; PR number assigned when the PR opens. Branch `mvp0/pr-set-a`, from `main`
  **after S12 13b merges** (§1.4), with the anchors of §12.2 re-verified against that `main`. Worktree `/Users/yuema137/mineworld-worktrees/impl-set-a`, held by one
  session.
- **Goal:** both reference clients open a settings menu, switch language live between `en` and
  `zh_Hans`, apply and persist display and clock settings, and do it through one shared module, with
  every check in §7 green.
- **Non-goals:** input options (SET-b); the World section and any HTTP (SET-c); the HUD date, the sun,
  weather and the launcher's close behaviour (TW-e); translating world content or signage art; key
  rebinding; audio (no audio exists); the promenade's menu.

## 12.2 Audit anchors (re-verify at freeze against `main`)

- 13b as merged: `clients/2d/scripts/hud/words.gd` (loading, fallback), `presentation/mineworld-default/2D/i18n/en.po`
  (its key list), `check_client_rules.py` R6, `--check-pack`, and the symlink skip (B-10).
- `clients/2d/scripts/app.gd` (`_ready` order, `_unhandled_input` Esc, the four `status.note`
  calls), `hud/status.gd` (hint, `show_state`, `clock()`), `scene/places.gd` `_door_text`,
  `scene/people.gd:64`.
- `clients/3d-spike/scripts/controls_hud.gd` (`CONTROLS`, `MODE_TITLES`, `add_line`, `toast`,
  `caption`), `slice/slice_main.gd` (`_hud`, `_process`, `_link`), `slice/slice_link.gd` (`_say` and
  its 16 calls, `display_label`, `_readable`), `slice/intents.gd` (`verb`), `player.gd:154` (Esc).
- Launchers `mineworld-2d` (`--resolution` in every windowed mode), `mineworld-slice:106`.
- Scans: `tests/acceptance/tests/client_rules.rs` (lexer, exclusions, symlink rule),
  `scripts/check_client_rules.py` (R4, `rglob`).
- Harnesses: `clients/2d/scripts/harness/drive.gd` (scenario match), `tools/cli/tests/client_2d.rs`
  and `godot2d/` (stub frame record), `slice_probe_world.gd` (`requested()`, modes).
- `server/PROTOCOL.md` refusal codes and rejection reasons (the code families of AC-SET-4).

## 12.3 Design decisions (SD-SET-a-n)

| Id | Decision |
| --- | --- |
| SD-SET-a-1 | `clients/shared/` project and `settings/` module as §3.1; class names prefixed `MineWorld` so they cannot collide with the 3D project's classes. Symlinks `clients/2d/mineworld_settings` and `clients/3d-spike/mineworld_settings`. |
| SD-SET-a-2 | Both clients' `project.godot` gain `application/config/use_custom_user_dir=true`, `custom_user_dir_name="MineWorld"` (§3.4). No other project setting changes. |
| SD-SET-a-3 | `MineWorldSettings` typed value and `validate()` (§3.3); `MineWorldSettingsStore.open(args: PackedStringArray) -> MineWorldSettingsStore` resolves `--settings=`, and harness flags force `none` (the list of harness flags is passed by each client's composition root, so the module knows no client's flags). |
| SD-SET-a-4 | `MineWorldDisplay.apply(settings, window, capabilities)` (§3.5), a no-op headless; the 15 s revert lives in the menu, not in `display.gd`. |
| SD-SET-a-5 | `MineWorldText`: `load_layers(pack_dir: String)` (shared → pack → user, merged per locale, §3.6), `languages() -> Array[MineWorldLanguage]` (locale, self_name), `set_language`, `code(family, code, args)` with 13b's readable fallback, `install_font_fallback()`; families declared in `const FAMILIES := ["action", "reason", "state"]`. 13b's `words.gd` keeps its API and delegates loading and fallback to `MineWorldText` (one implementation of the fallback). |
| SD-SET-a-6 | Keys: 13b's families unchanged (`action.*`, `action.*.done`, `reason.*`, `ui.*`, `panel.*`, `suggest.*`); SET-a adds `ui.settings.*`, `hud.*`, `hint.*`, `camera.*`, `link.*`, `note.*`, `door.*`, `state.*`, `language.self_name`, `clock.default`. Segments `[a-z0-9_-]`. |
| SD-SET-a-7 | `MineWorldClockFormat.time_of_day(seconds_into_day, clock, locale)`. 2D `status.gd` replaces its `clock()` with `hud.day_time` (`{day_label} {time}`) built from it. 3D adds one HUD line `hud.time` when connected, from the link's latest `at()`. |
| SD-SET-a-8 | Esc handling, subject to QSET-2: 2D `app.gd` toggles the menu instead of quitting; 3D `slice_main.gd` handles `ui_cancel` before `player.gd` and toggles the menu. `player.gd`'s release-mouse branch stays for the promenade, and the slice consumes the event first. |
| SD-SET-a-9 | Nodes showing world content (2D people and door names, 3D figure labels) join group `mineworld_world_text` and set `auto_translate_mode = DISABLED`. Composed labels re-render on `NOTIFICATION_TRANSLATION_CHANGED`. |
| SD-SET-a-10 | `slice_link._say(log_line, shown_key, args, notice)`: prints the English `log_line` unchanged (INV-SET-6) and emits `tr(shown_key).format(args)`. Every one of the 16 calls is converted. |
| SD-SET-a-11 | Launchers: `mineworld-2d` passes `--resolution` only with `--res=` or `--capture`; `mineworld-slice` only with `--res=` or a capture or probe mode. Both pass `--settings=none` to every scripted mode. |
| SD-SET-a-12 | Intentional `en` wording changes (INV-SET-7), the only ones: the 2D hint's "Esc quits" → "Esc menu"; the 3D controls line's "Esc release mouse" → "Esc menu"; the 2D rejection note's raw JSON → `reason.<code>` prose (unless 13b has already done it); the 2D clock line's `day N  hh:mm` keeps its form in 24h and gains `h:mm AM/PM` in 12h. Every other English string moves verbatim into `en.po`. |
| SD-SET-a-13 | `tests/acceptance/tests/client_text.rs` (AC-SET-3 for 3D and the shared module, since 2D is 13b's R6; AC-SET-4 over every layer; INV-SET-1's identifier scan); the GDScript lexer moves from `client_rules.rs` into `tests/acceptance/tests/support/gdscript.rs` without behaviour change (client_rules stays green). Its `.po` reader is about 60 lines, std-only, test-only, with a comment saying why no crate was added (a crate for one test is a heavier dependency than the parser). |
| SD-SET-a-14 | Runtime checks as `#[ignore]` tests in `tools/cli/tests/client_settings.rs` (AC-SET-1, -2, -5 … -12, -14), driving `./mineworld-2d --drive=settings` and `./mineworld-slice --world --settings`, plus the module's own headless checks under `clients/shared/checks/` (store, glyphs, menu) run by the same test file. |
| SD-SET-a-15 | Font as §3.7, subject to QSET-1: `fonts/NotoSansSC-Regular.otf` (sha256 recorded in `NOTICE` and `SETTINGS.md`), `fonts/OFL.txt`; the DEP-8 amendment of §11.1 lands in C1. |
| SD-SET-a-17 | **Migration of 13b's `.po` (a move, not a reformat).** In C3, every `action.*`, `action.*.done` and `reason.*` entry of `presentation/mineworld-default/2D/i18n/en.po` moves verbatim (msgid, msgstr, comments) into `clients/shared/settings/locale/en.po`. All other 13b entries stay in the pack. No key is renamed. The check asserts that the union of the two files after the move equals 13b's file before it, entry for entry. `check_client_rules.py --check-pack` stays green on the smaller pack file. In C5, `presentation/mineworld-default/2D/i18n/zh_Hans.po` is added for the entries that stay in the pack. |
| SD-SET-a-18 | **Every platform (§1.5).** No platform branch in the module beyond the Wayland tooltip (AC-SET-16). `display.gd` prints the platform line once on start. Both launchers check that `mineworld` and `mineworld_settings` are directories, and if not stop with "the shared client modules are symlinks; on Windows run `git config core.symlinks true` (Developer Mode) and check out again". The client READMEs and `SETTINGS.md` carry the per-OS facts of §3.4, §3.5.1 and §3.7. The DEP-8 font exception text of §4.4.1 goes into the C1 records. |
| SD-SET-a-16 | Decision records in C1: ARC-SET-a, ARC-SET-b, DEP-SET-a, DEP-SET-b with real numbers from the primary session, and the DEP-8 amendment. |

## 12.4 Files touched

```text
new   clients/shared/{project.godot, README.md, SETTINGS.md, .gitignore}
new   clients/shared/settings/{settings,store,display,text,clock_format,menu}.gd (+ .uid)
new   clients/shared/settings/locale/{en.po, zh_Hans.po, messages.pot}
new   clients/shared/settings/fonts/{NotoSansSC-Regular.otf, OFL.txt}
new   clients/shared/checks/{store_check,glyph_check,menu_check}.gd
new   clients/2d/mineworld_settings, clients/3d-spike/mineworld_settings          (symlinks)
edit  clients/2d/project.godot, scripts/app.gd, scripts/hud/status.gd, scripts/hud/words.gd (13b's;
      delegates to MineWorldText), scripts/scene/places.gd, scripts/scene/people.gd (group only),
      scripts/harness/drive.gd (scenario `settings`), README.md, PRESENTATION.md ("Wording": layers)
edit  presentation/mineworld-default/2D/i18n/en.po (action.*/reason.* moved out, SD-SET-a-17)
new   presentation/mineworld-default/2D/i18n/zh_Hans.po
edit  clients/3d-spike/project.godot, scripts/controls_hud.gd, scripts/slice/slice_main.gd,
      scripts/slice/slice_link.gd, scripts/slice/intents.gd, scripts/slice/slice_probe_world.gd
      (mode `--settings`), README.md
edit  mineworld-2d, mineworld-slice
new   tests/acceptance/tests/client_text.rs, tests/acceptance/tests/support/gdscript.rs
edit  tests/acceptance/tests/client_rules.rs (lexer moved out only)
new   tools/cli/tests/client_settings.rs
edit  docs/DECISIONS.md, NOTICE, clients/protocol/ADOPTION.md (§11.5 sentence)
edit  .structured-coding/plans/mvp0/step-20-client-settings.md (ledger)
```

## 12.5 Commit plan

Each commit tracks implementation, deterministic validation and LLM logic review separately. `[x]`
requires the work and the evidence. An item that does not apply is `N/A` with its audited reason.

### C0 — design (this section)

- Goal: the PR's design and acceptance, before code. Markdown only.
- [x] Implementation: this §12 is current at freeze; anchors re-verified against `main @ aee8290`
  on 2026-10-09 (§12.11 F-1 … F-12).
- [x] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py` —
  run with C1 (below).
- [x] Review: every SD traces to §3, and every AC to §7 (re-read at implementation start; F-1 … F-4
  are the bounded corrections the re-verification found).

### C1 — specs before code

- **Goal:** the contracts a reviewer checks the code against: `clients/shared/SETTINGS.md` (module
  layout, adoption, API, the schema of §3.3, the file of §3.4, key conventions, the code families,
  INV-SET-1 … 10), `clients/shared/README.md` (short), DECISIONS records (SD-SET-a-16), the DEP-8
  amendment, `NOTICE`, the ADOPTION.md sentence.
- **Scope:** Markdown only. **Non-goals:** any `.gd`.
- [x] Implementation: the files above — `clients/shared/SETTINGS.md`, `clients/shared/README.md`;
  `docs/DECISIONS.md`: DEP-8's font exception (§4.4.1, word for word) and its table row, a dated note
  under ARC-70 (ARC-SET-b), ARC-72 (ARC-SET-a), DEP-32 (DEP-SET-a), DEP-33 (DEP-SET-b) — numbers
  provisional (F-10); `NOTICE`; the `ADOPTION.md` §1 sentence.
- [x] Validation: `check_doc_headings.py` → "192 numbered sections across 26 documents, none
  duplicated"; `check_decision_ids.py` → "86 decision ids, all distinct". PASS. No new defined term:
  "setting" and "host command" are explained in `SETTINGS.md` §1, which says they are not ontology.
- [x] Review: `SETTINGS.md` uses `Presentation Pack`, `Observation`, `ActionIntent` as
  `CORE_CONCEPTS.md` defines them, with no synonym; the DEP-8 exception text is §4.4.1's verbatim, limited
  to fonts under OFL-1.1.
- **Acceptance:** a reader can implement C2–C6 from `SETTINGS.md` alone.
- **Commit boundary:** documentation only.

### C2 — the store and display application (no UI, no client)

- **Goal:** settings can be loaded, validated, saved atomically and applied, headless-safe.
- **Scope:** `clients/shared/project.godot`, `settings.gd`, `store.gd`, `display.gd`,
  `checks/store_check.gd`. **Non-goals:** catalogs, menu, clients.
- [x] Implementation: `clients/shared/{project.godot,.gitignore}`; `settings/settings.gd`
  (`MineWorldSettings`: enums, `read(config, problems)` in place of `validate()` — the same rule, read
  and validated in one pass — `write`, `differs_in`, `copy`, `equals`); `settings/store.gd`
  (`MineWorldSettingsStore.open(args, harness_flags)`, `--settings=<path>|none`, `--extra-locale=`,
  load: missing / unreadable / future version / bad key / unknown key / lone `.tmp` recovered (F-11),
  `settle_language(offered)`, `save`: re-read, per-section merge, `.bak` after a load problem, `.tmp` +
  `rename_absolute`); `settings/display.gd` (`MineWorldDisplay.apply` changes only what differs;
  `CAP_RENDER_SCALE`; `effective_fps`; `refresh_rate`; `size_pinned` — an engine `--resolution` wins;
  `size_presets`; `fullscreen_same_as_borderless` (the one Wayland branch); `print_platform_once`).
- [x] Validation: `godot --headless --path clients/shared --script res://checks/store_check.gd` →
  `store_check: PASS`, 49 `[PASS]`, 0 `[FAIL]` (macOS, Godot 4.7.2, driver metal, display headless).
  Covers AC-SET-8's five cases (unparsable; `window_mode=7`; `language="tlh"` through
  `settle_language`; `max_fps="fast"`; `version=99`): defaults per key, a warning naming file and key,
  file byte-identical after load, `.bak` equal to the old bytes after an apply; the round trip of every
  field; harness flags → `none`; atomic write (no `.tmp` left; a lone `.tmp` recovered); the two-store
  merge; a headless `apply` of every mode × vsync, render scale and caps; AC-SET-15's folder line
  (`…/Application Support/MineWorld`). First run: 2 FAIL — (1) after a key problem the save rebuilt the
  file from scratch and lost another module's `[launcher]` section: fixed, a parseable file is always
  merged and, after a load problem, every owned section is rewritten; (2) the check assumed a client
  window, but a bare `--script` root is 100×100: the check now starts from the default size. Mutations
  (each then reverted, recorded): skip the merge → 2 FAIL (the `[launcher]` keep and the two-client
  merge); drop the `Engine.max_fps` assignment → "the frame cap applies" FAIL. Note: for an unparsable
  file Godot itself also prints its `ConfigFile parse error` line before the store's one warning.
- [x] Review: `grep` for `MineWorldClient|HTTPRequest|WebSocketPeer|StreamPeer|PacketPeer|submit|connect_to_world`
  under `settings/` finds nothing (INV-SET-1; held in CI by `client_text.rs` from C3). No `Dictionary`
  crosses the API: capabilities are a `PackedStringArray` of `CAP_*`, problems a `PackedStringArray`.
  The store writes only `MineWorldSettings.write`'s keys (INV-SET-2).
- **Failure cases:** unwritable user directory → a warning; settings stay in memory, and the client runs.
- **Commit boundary:** the module's own project only.

### C3 — UI text: catalogs, discovery, live switching, the font, the scans

- **Goal:** the text layer and its CI checks, before any client uses it.
- **Scope:** `text.gd`, `clock_format.gd`, `locale/{en,zh_Hans}.po`, `messages.pot` (the keys the menu
  needs and the clock entries), `fonts/`, `checks/glyph_check.gd`, `tests/acceptance/tests/client_text.rs`,
  the lexer move to `support/gdscript.rs`.
- [x] Implementation: `settings/text.gd` (`MineWorldText`: `load_layers(pack_dir, extra_dirs, wording)`
  merging shared → pack → `user://locale` → `--extra-locale=` per locale into one registered
  `Translation` each; `locales`, `languages` (`MineWorldText.Language`), `set_language`, `has`, `text`
  with 13b's readable fallback, `code(family, …)` over `FAMILIES`, `readable`, `set_clock`/`clock`,
  `install_font_fallback`, `bundled_font`, `mark_world_text`, `visible_ui_texts`);
  `settings/clock_format.gd` (`time_of_day(seconds, clock)`, step-19 §8.2's placeholders
  `{hour24}`/`{hour12}`/`{minute2}`/`{ampm}` — the design's `locale` parameter is the current language,
  bounded); `locale/{en,zh_Hans}.po` + `messages.pot` + `.gdignore`; `fonts/NotoSansSC-Regular.otf`
  (fetched with `gh api … NotoSansSC-Regular.otf`, SHA-256 `faa6c9df…706d5ea9`, git blob `fc0fda93…`
  equal to upstream's), `fonts/OFL.txt` (upstream `Sans/LICENSE` verbatim, under the copyright line read
  from the font's name table) + `.gdignore`; the move (SD-SET-a-17, F-4) — every `action.*`/`reason.*`
  entry from the 2D pack's `en.po` into the shared layer, keys unchanged, `action.move` neutral
  ("Walk") in the shared layer and 13b's entry kept in the pack under `#. override`; the shared layer
  also carries every §5.5 refusal code (F-7) and `language.self_name`, `clock.default`, `hud.time.*`,
  `hud.ampm.*`; `checks/glyph_check.gd`, `checks/text_check.gd`;
  `tests/acceptance/tests/support/gdscript.rs` (the lexer, moved unchanged) and
  `tests/acceptance/tests/client_text.rs` (AC-SET-3, AC-SET-4, SD-SET-a-17's move check against
  `aee8290`'s file, INV-SET-1, AC-SET-16, a reader self-test). Marker catalog: generated by
  `menu_check.gd`'s `write_marker` (C4) into a scratch folder, locale `qaa`, loaded with
  `--extra-locale=`.
  **Brought forward from C5 (bounded, F-13):** the 2D symlink `clients/2d/mineworld_settings`,
  `words.gd`'s delegation (`load_pack` → `MineWorldText.load_layers`; `has`, `text`, `readable`,
  `reason` delegate) and `presentation/mineworld-default/2D/i18n/zh_Hans.po` — the moved entries live
  in the shared layer from this commit on, so the 2D client must load it here, and AC-SET-4's per-layer
  completeness needs the pack's `zh_Hans.po` as soon as the test exists.
- [x] Validation (macOS, Godot 4.7.2):
  - `cargo test -p mineworld-acceptance --test client_rules` → 3 passed (unchanged after the lexer
    move). On the way it caught `store_check.gd`'s probe key `invite` (an action type's spelling);
    renamed `server_address` (C2's committed check therefore trips `client_rules` at C2's head only).
  - `cargo test -p mineworld-acceptance --test client_text` → 4 passed, 1 failed by design: AC-SET-3's
    scan lists exactly the 3D client's present literals (`controls_hud.gd:170` "camera: %s",
    `slice_main.gd` "place: %s", "looking at: %s", "world: offline …", `slice_link.gd` "perceive %s",
    …), which C6 converts. That is the scan's discrimination on the real tree; it is red from C3 to C6.
    Two false positives found and fixed in the scan, each pinned by the self-test: `"\n"` (the lexer
    keeps an escape's character, so two letters are required) and placeholder names in
    `{"fps": n}` argument dictionaries (a literal followed by `:` is a key).
  - AC-SET-4's four mutations, applied together, each produce their own finding (then reverted):
    delete `reason.paused` from `zh_Hans.po` → "has no zh_Hans entry"; `{target}` → `{who}` in
    `action.talk` → "has placeholders {"who"}, en has {"target"}"; a `mutant_code` row in
    `PROTOCOL.md` §5.5 → "no "reason.mutant_code""; `action.talk` duplicated in the 2D pack → "also
    in the shared layer; mark it `#. override`".
  - `python3 scripts/check_client_rules.py` → PASS (0); `--check-pack presentation/mineworld-default/2D`
    → PASS (0) on the smaller pack file with `zh_Hans.po`.
  - `glyph_check.gd` → PASS: 4 catalogs, 263 distinct characters, 0 missing. Mutation: a scratch
    catalog with `☕🙂` → FAIL naming U+2615 and U+1F642.
  - `text_check.gd` → PASS: en/zh_Hans offered, en first; the round trip; `code` and the readable
    fallback (never raw, in both languages); a key only English has falls back to English; 00:00,
    12:00, 19:42, 23:59 × 12h/24h × en/zh_Hans (`12:00 AM`/`00:00` … `下午 11:59`/`23:59`); automatic
    12h in en and 24h in zh_Hans; a chosen 12h holds in zh_Hans.
  - 13b's and 13a's 2D suites on this tree: see the C3/C4 run below.
- [x] Review: keys built from data — `MineWorldText.code` and, in the 2D client, `words.gd`'s
  `action()` (13b's, whose own fallback names the action's arguments) and its `reason()`, now through
  `code`; 13b's `status.gd` builds `ui.state.<state>` (13b's key, unchanged; `client_text.rs` counts
  a literal prefix ending in `.` as asking for its keys). The fallback never shows a raw key
  (`text_check`).
- **Commit boundary:** module + one acceptance test file + the lexer extraction.

### C4 — the menu

- **Goal:** a working menu over the store, display and text.
- **Scope:** `menu.gd`, `checks/menu_check.gd`, catalog keys for the menu.
- [x] Implementation: `settings/menu.gd` (`MineWorldSettingsMenu`): General (language by
  `self_name`, clock) and Display (window mode with the Wayland tooltip, window size in windowed mode,
  render scale in the full-screen modes with `CAP_RENDER_SCALE`, VSync, frame cap, the monitor's rate
  and the measured frame rate, read-only); Apply / Revert / Close / Quit (`quit_requested`); the 15 s
  confirmation (`confirm_left`, `tick`, `keep_display`); `add_tab(key, control)`; the client's
  `Theme`; a full-screen control with `MOUSE_FILTER_STOP`. Every text from a key, rendered again on
  `NOTIFICATION_TRANSLATION_CHANGED`; a clock change propagates that notification so composed labels
  re-render. Close drops what was not applied (§7 of `SETTINGS.md`). The menu's keys
  (`ui.settings.*`) in both shared catalogs. Committed together with C3 (one commit, recorded).
- [x] Validation: `menu_check.gd` → PASS (39 texts over both tabs): every text turns Chinese at once
  and switching back reproduces every text exactly (AC-SET-1 on the menu alone; the language list is
  admitted — each language names itself); Apply saves language and clock with no confirmation; Revert
  restores; Close drops an unapplied language; a display change applies, is not saved, and reverts
  after `tick(16)`; Keep saves it; `add_tab` adds one tab and leaves the others; under the marker
  catalog (`qaa`, 100+ entries) every text carries `⟦` (AC-SET-2 on the menu alone). First run: 1 FAIL
  — the walk saw only the current tab (13 texts); the check now visits each tab. Mutations (reverted):
  the menu ignores `NOTIFICATION_TRANSLATION_CHANGED` → both walks FAIL naming the title label; a
  planted `Label` "Hello there" → both walks FAIL and `client_text.rs` names `menu.gd:267`.
- [x] Review: the menu decides nothing per client beyond `capabilities` and the `Theme`; it names no
  client, no network identifier and no platform (the Wayland question is `MineWorldDisplay`'s).
- **Commit boundary:** module only.

### C5 — the 2D client

- **Goal:** the 2D client uses the module for every UI string, the menu, the clock and display.
- **Scope:** symlink, `project.godot`, `app.gd`, `hud/status.gd`, `scene/places.gd`, `scene/people.gd`,
  `harness/drive.gd` (`settings` scenario), `README.md`, `mineworld-2d`, catalog keys.
- [x] Implementation: `project.godot` (SD-SET-a-2); `app.gd` `_settings()` first in `_ready` (store
  with `HARNESS_FLAGS = ["drive", "capture"]`, layers with 13b's `--no-wording` and
  `--presentation=none`, font, clock, language, platform line, display), the settings menu in `_hud()`
  with a warm `Theme`, the walker disabled while it is open, Esc closes 13b's menu or toggles the
  settings menu (Quit is in it), keys and clicks ignored while it is open, `_notification` re-rendering
  door labels, markers, 13b's menu and panels and the status line and clearing toasts;
  `hud/status.gd` (the hint and the time line re-rendered, the who-line marked world text),
  `hud/words.gd` (`day_time` → `MineWorldClockFormat.day_time`), `hud/talk_line.gd` (hint
  re-rendered), `hud/toasts.gd` (`clear`), `scene/people.gd` (names marked world text, markers not
  auto-translated), `scene/places.gd` (door labels not auto-translated); `harness/settings.gd` (the
  `settings` scenario: a two-stride walk, AC-SET-1's walk both ways with `EVIDENCE text` pairs,
  `--marker` (AC-SET-2), `--apply=` through the menu, AC-SET-12, then `leave_world`; the `display`
  scenario for AC-SET-11) and `drive.gd` (dispatch, and every scripted run prints
  `EVIDENCE {"settings": …}`); `ui.hint` "Esc: close, menu" in both 2D catalogs (SD-SET-a-12, in
  `client_text.rs`'s `NAMED_CHANGES`); `mineworld-2d` (symlink check, `--settings=none` for `--drive`,
  `--resolution` only with `--res=` or `--capture`, Bash 3.2-safe empty array); `README.md`,
  `PRESENTATION.md` §7 (layers, `--no-wording`); `tools/cli/tests/godot2d/mod.rs` (`StubLog.frames`,
  `Drive::start_with`); `tools/cli/tests/client_settings.rs`.
  **F-16 (bounded, recorded):** the clock setting governs the HUD's day and time through a new shared
  key `hud.day_time`; 13b's `format.clock` (the 24-hour schedule times of the agenda and employment
  rows) is unchanged. Following the setting there would have turned 13b's agenda row "08:00" into
  "8:00 AM" under QTW-16's English default and failed 13b's
  `the_panels_show_what_is_disclosed` (material stop 4); SD-SET-a-7 names only the status line.
- [x] Validation (macOS, Godot 4.7.2):
  - `check_client_rules.py` → PASS (0) with `clients/2d/mineworld_settings` present (F-5);
    `--check-pack presentation/mineworld-default/2D` → PASS (0); `client_text` → 5 passed (with C6).
  - 13a/13b suites on the C5 tree: `client_2d` 8/8, `client_2d_interact` 5/5,
    `client_2d_interact_stub` 3/3 (`/tmp/mw-seta-logs/c5-2d.log`).
  - `client_settings` 2D tests: AC-SET-1/-2/-12 on market-town (41 texts; "every UI text turns from
    en to zh_Hans at once", "switching back reproduces every text exactly", "under the marker catalog
    …", "an open menu takes gameplay input 0 requests while open"); AC-SET-5 (frames from `join` to
    `leave` equal on defaults and on a zh_Hans/24h/borderless/30 fps file; `drive:` lines and check
    names equal, INV-SET-6); AC-SET-6 (an Apply's file holds exactly the §3.3 keys, none of the invite,
    nickname, seat or address); AC-SET-8 at runtime (a `window_mode=7` file: the client runs, the
    problem names file and key, the file is byte-identical); AC-SET-9 2D half (`--drive=walk` with a
    planted zh_Hans/30 fps file in the real folder reports `file ""`, en, cap 0 and the same checks;
    the player's file restored); AC-SET-11 2D (window 1280×720, VSync 0; cap 30 → 30.01 fps; no cap →
    119.97 fps on a 120 Hz display). First run: 2 FAIL in the test's verdict parsing (names padded to
    30 columns then one space; borderless hides the size list, so text counts differ) — fixed in the
    test, the client unchanged.
  - Mutations (each reverted): walker left running while open → AC-SET-12 FAIL "7 requests while
    open"; hint not re-rendered → AC-SET-1 and AC-SET-2 FAIL naming the hint label; `seat="carol"`
    written by the store → AC-SET-6 FAIL "the settings file holds "carol""; the file saved on load →
    store_check FAIL (byte-identical, `.bak`, merge); the locale in the move payload → AC-SET-5 FAIL
    "the frames sent do not depend on the settings"; `drive` dropped from `HARNESS_FLAGS` → AC-SET-9
    FAIL ("a harness reads no settings file", `user://settings.cfg`).
- [x] Review: names, places, tags, dialogue and activity kinds are never passed through a catalog
  (people labels and the who-line are world text; door labels and markers put world content only in
  `{place}`/`{kind}`); 13a's behaviour is unchanged apart from Esc (13a/13b suites green).
- **Commit boundary:** the 2D client, its launcher and its tests.

### C6 — the 3D client

- **Goal:** the same for the 3D slice.
- **Scope:** symlink, `project.godot`, `controls_hud.gd`, `slice_main.gd`, `slice_link.gd`,
  `intents.gd`, `slice_probe_world.gd` (`--world --settings`), `README.md`, `mineworld-slice`,
  catalog keys.
- [x] Implementation: symlink `clients/3d-spike/mineworld_settings`; `project.godot` (shared user
  dir); `controls_hud.gd` (every line from keys, re-rendered on a language change, a showing toast
  dropped; `attach(…, esc_opens_menu)`: the slice says "Esc  menu", the promenade keeps "Esc  release
  mouse" through its own key `hint.controls`, and `attach` loads the shared layer when no client has —
  the promenade loads no settings, QSET-9; conversation captions marked world text);
  `slice/slice_main.gd` (`_settings()` first in `_ready`, `harness_flags()` = every probe mode, render
  scale capability, the menu with a dark `Theme`, Esc in `_input` before the player's own handler,
  walking/looking/E stopped while open, HUD lines from keys every frame, the world line kept as key +
  arguments and re-rendered, the time line `hud.day_time` when connected, `first_frame` for AC-SET-7);
  `slice/slice_link.gd` (`said(key, args, notice)`; `_say(log_line, key, args, notice, log_detail)` —
  all 16 calls converted, every log line's format string as on `main`; `shown_label`/`shown_text`
  for "You"/"Someone"; `_shown_reason` as a `reason.<code>` reference; figure labels world text;
  `_speak` names the speaker in English in the log); `slice/intents.gd` (`VERBS` → keys,
  `LOG_VERBS` for the log); `slice/slice_probe.gd` (`--hud`'s place line from its key);
  `slice/slice_probe_world.gd` (mode `settings`: first frame, render scale, two deterministic
  requests, AC-SET-1 walk, `--marker`, AC-SET-12 with play-mode input); `mineworld-slice` (symlink
  check, `--settings` mode, `--settings=none` for every scripted mode, `--resolution` only with
  `--res=` or a scripted mode); `README.md`. Shared: `MineWorldText.Ref`/`ref`/`code_ref` (an argument
  worded at render time, found by the 3D walk — see validation), `has_cjk`, the walk skips faded text,
  `MineWorldClockFormat.day_time`; the 3D keys in both shared catalogs, `en` reading exactly as on
  `main`.
- [x] Validation (macOS, Godot 4.7.2):
  - `cargo test -p mineworld-acceptance --test client_text --test client_rules` → 5 + 3 passed: the
    AC-SET-3 scan is green over the 3D client and the module (red since C3 by design). One finding on
    the way: a log detail literal on a `_say(` line — moved to its own line.
  - `./mineworld-slice --drive` → "all drive checks pass"; `--world --link` → "all link checks pass",
    every `[link]` line in `main`'s English format; `--world --target` → "all target checks pass";
    `--measure` → "1 SCALE CHECKS OUT OF RANGE", the stature 1.802 m line recorded as pre-existing
    in step-15 (§19 J-1: "the same lines as before") — unchanged; `--world --conversation` → first run
    INCONCLUSIVE (the connection closed (−1) right after connecting, before the probe began), re-run
    PASS (reply on screen 1.6 s after talk; caption 2 lines).
  - `--world --settings --marker` (windowed): first run 2 FAIL — (1) a faded toast was walked, then
    correctly dropped by the language change: the walk now skips text at alpha 0 and the probe waits
    for the toast's fade; (2) **a real defect:** the world line re-rendered its template in Chinese
    but kept its arguments (verb, reason) worded in English at arrival ("世界：无法与 Wes Calloway talk
    to：too far away") — fixed with `MineWorldText.Ref` arguments; then PASS (44 texts, "世界：无法与 Wes
    Calloway 交谈：太远了").
  - `client_settings` (9 tests) → 9 passed: the module's checks; AC-SET-15 over the shared, 2D and 3D
    projects, and a probe file written by the 2D project read by the 3D one; the 2D tests above; the 3D
    test (AC-SET-1/-2/-12 on social-cafe, AC-SET-5: the requests equal on defaults and on a
    zh_Hans/24h/67 % file, AC-SET-10: the `[link]` lines equal after masking per-run port and instance,
    AC-SET-11 3D: `scaling_3d_scale` 0.67 from the file, 1.0 by default); AC-SET-7 (2D Apply of
    zh_Hans/24h/1280×720 → the 3D first frame: zh_Hans, clock 2, window 1280×720; the player's file
    restored).
- [x] Review: `_say`'s 16 calls: each `log_line` is the same format string with the same English
  arguments as on `main` (`display_label`, `_readable`, `SliceIntents.log_verb`); `player.gd` is
  unchanged, its Esc branch still serves the promenade; the menu's Esc reaches the slice first
  (`_input`).
- **Commit boundary:** the 3D client, its launcher and its tests.

### C7 — cross-client evidence and gates

- **Goal:** the one-module-two-clients checkpoint and the full gate on the PR head.
- [x] Implementation: evidence and ledger; and three small code items found by the gate's runs —
  the menu's language selector follows the live language (a still showed "English" selected over a
  Chinese menu), stills hooks (2D `--capture`, 3D `--stills=`), AC-SET-14's check in
  `menu_check.gd`; `clients/shared/evidence/` (stills, `.gdignore`, README); `origin/main` merged
  (`72efd61`; one conflict in `docs/DECISIONS.md`, both sides appended — kept both; ARC-68 landed on
  main, no collision with ARC-72/DEP-32/DEP-33).
- [x] Validation — the full gate, on the merged tree (macOS 26, Godot 4.7.2, Metal, 120 Hz display):
  - `cargo fmt --all --check` PASS; `cargo clippy --workspace --all-targets --all-features -D warnings`
    PASS; `cargo test --workspace` (at `72efd61` + the stills commit's non-Rust changes) → 886 passed,
    0 failed, 29 ignored (the Godot tests) — PASS; `check_doc_headings.py` → 192 sections, none
    duplicated; `check_decision_ids.py` → 88 ids, all distinct; `check_scratch.py scan` → 185 test
    sources clean; `check_client_rules.py` → PASS (0); `--check-pack presentation/mineworld-default/2D`
    → PASS (0); `client_text` 5/5; `client_rules` 3/3.
  - Godot, on code head `d5921e7`: `client_2d` 8/8, `client_2d_interact` 5/5,
    `client_2d_interact_stub` 3/3, `client_settings` 9/9 (`/tmp/mw-seta-logs/c7-godot.log`); on
    `873612b` (AC-SET-14 added): `the_settings_module_checks_pass` and `the_settings_folder_is_shared`
    re-run → 2/2; the four module checks PASS (store 49, text, glyph 263 characters, menu incl.
    AC-SET-14).
  - 3D probes: `--drive`, `--world --link`, `--world --target` pass; `--measure` the same one pre-existing
    line; `--world --conversation` PASS (on re-run); `--world --settings --marker` PASS.
  - AC-SET-9, 3D half and stills: with a zh_Hans/24h/borderless/30 fps file planted in the real
    folder (then removed by `store_check --read`), `--drive` and `--world --link` pass with no Chinese
    in their output and the platform line unchanged. The 2D `--capture` stills are **not byte-identical
    even between two runs with no planted file** (the world's clock and people move), so that oracle
    cannot discriminate: INCONCLUSIVE as an oracle; the discriminating evidence is the settings line,
    identical with and without the planted file (`file ""`, en, cap 0, VSync on, windowed 1600×900),
    and the planted run's still shows the English 12-hour HUD.
  - AC-SET-14: PASS (`menu_check`): a scratch `user://locale/xx_test.po` is offered as "Testish";
    choosing it shows its two texts and English for the rest, never a raw key; removed afterwards.
  - CP-SET-a by hand on macOS (§6): see §12.12; the parts the harness covers are in the AC table.
  - Mutations (each reverted): the language list hardcoded → AC-SET-14 FAIL; an `HTTPRequest` and an
    `OS.get_name()` branch planted in `store.gd` → `client_text` FAIL naming
    `store.gd:61 names HTTPRequest (INV-SET-1)` and `store.gd:63 a platform branch (AC-SET-16)`.
- [x] Review: INV-SET-1 … 10 traced in §12.12.

## 12.12 Acceptance on the final head (macOS; Windows and Linux are the operator's, §8)

| Criterion | Result | Evidence (where) | Mutation that turned it red |
| --- | --- | --- | --- |
| AC-SET-1 | PASS | 2D market-town 41 texts, 3D social-cafe 44 texts, menu alone 39 — every text turns at once and back (`client_settings`, `menu_check`; `EVIDENCE text` pairs) | menu ignores `NOTIFICATION_TRANSLATION_CHANGED`; 2D hint not re-rendered |
| AC-SET-2 | PASS | marker `qaa` walks in 2D, 3D and the menu | planted `Label` "Hello there" |
| AC-SET-3 | PASS | `client_text` (3D client + module; 2D is 13b's R6, PASS) | planted label literal named at `menu.gd:267`; the pre-C6 3D tree itself (12 literals) |
| AC-SET-4 | PASS | `client_text` catalogs, families from `PROTOCOL.md` §5.5 and `Rejection`, `messages.pot`; the move vs `aee8290` | 4/4 design mutations |
| AC-SET-5 | PASS | 2D stub: frames `join`…`leave` equal; 3D: requests equal; static: INV-SET-1 scan | locale in the move payload; planted `HTTPRequest` |
| AC-SET-6 | PASS | an Apply's file holds exactly §3.3's keys, no invite/nickname/seat/address | the store writes `seat` |
| AC-SET-7 | PASS | 2D Apply zh_Hans/24h/1280×720 → 3D first frame zh_Hans, clock 2, 1280×720 | — (no planted mutation; the oracle is the first frame) |
| AC-SET-8 | PASS | `store_check` five cases; 2D runtime `window_mode=7` | the file saved on load |
| AC-SET-9 | PASS (stills byte-identity INCONCLUSIVE as an oracle, see C7) | 2D `--drive=walk`, 3D `--drive`, `--world --link` with a planted file | `drive` dropped from `HARNESS_FLAGS` |
| AC-SET-10 | PASS | 3D `[link]` lines equal across en/zh files; `--world --link` with a planted zh file all English; 2D `drive:` lines equal | — (by construction: `_say` prints `log_line`; review of all 16 calls) |
| AC-SET-11 | PASS | 2D window 1280×720, VSync 0, cap 30 → 30.01 fps, none → 119.97 fps; 3D render scale 0.67; headless no-op | `Engine.max_fps` assignment dropped (C2) |
| AC-SET-12 | PASS | 2D: W 2 s + a click → 0 requests; 3D: W, mouse motion, E → 0 requests, yaw and pitch unchanged | walker left running |
| AC-SET-13 | PASS | `glyph_check` 263 characters, 0 missing | ☕🙂 in a scratch catalog |
| AC-SET-14 | PASS | `menu_check` | the language list hardcoded |
| AC-SET-15 | PASS (macOS) | `…/Application Support/MineWorld` in the shared, 2D and 3D projects; a file written by the 2D project read by the 3D one | — (H-10 on Windows/Linux) |
| AC-SET-16 | PASS | `client_text` (the one Wayland line admitted) | planted `OS.get_name()` branch |

**INV-SET-1 … 10.** 1: `client_text` scan + AC-SET-5. 2: AC-SET-6. 3: N/A in SET-a (no host module;
SET-c). 4: AC-SET-1/-2/-3, R6. 5: AC-SET-9. 6: AC-SET-10 and the 2D `drive:` comparison. 7: the named
changes only (`client_text` move check: `ui.hint`; SD-SET-a-12; F-15, F-16) — the 3D `[link]` lines
and HUD English as on `main` (review, stills). 8: both clients symlink one module; INV-SET-8's one
duplicated formula removed (`MineWorldClockFormat.day_time`). 9: AC-SET-13. 10: review — names, places,
tags, dialogue and kinds are world text or plain arguments, never keys.

**CP-SET-a (§6), macOS.** The harness covers it end to end: Esc's menu open over the connected 2D HUD
→ 简体中文 → every HUD and menu string Chinese at once (AC-SET-1); the clock line in 24-hour form
("第1天  00:00"); Apply of zh_Hans/24h/1280×720 → the 3D client's first frame in Chinese, 24-hour, at
1280×720 (AC-SET-7). The windowed walk-through by hand (Esc key, the 15 s confirmation on a real window
change, relaunch) is H-3, H-4, H-7 and H-9 below, left to the operator.

## 12.13 The operator's hand checks (§8), per OS

The gate ran the automated column on macOS. Each check records the platform line the client prints
(`[settings] platform <OS>, driver <driver>, display <server>`).

| Id | macOS | Windows (D3D12; Vulkan with `--rendering-driver vulkan`) | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| H-1 Chinese crisp, no tofu, no mixed faces | stills in `clients/shared/evidence/` (Retina capture); external 1× display: operator | 100 % and 150 % scaling | on a machine with `fc-list :lang=zh` empty | at 125 % |
| H-2 menu layout in both languages | stills; operator | 150 % | — | — |
| H-3 windowed → borderless → fullscreen → windowed | operator (Cmd-Tab, no empty Space) | operator (Alt-Tab, brief black flash at most) | operator (no stuck full screen) | operator (Fullscreen = Borderless, tooltip says so) |
| H-4 size change, 15 s countdown, revert, keep across restart | operator; judge logical vs physical presets on Retina | operator | operator | operator |
| H-5 3D VSync Off/On/Adaptive, cap 30/60/none, render scale 50–100 % | operator (the Display tab's FPS: 30.01 with cap 30, 119.97 uncapped measured in 2D) | operator, D3D12 and Vulkan | operator | operator: is VSync Off honoured? |
| H-6 12h/24h × en/zh, both clients | automated (`text_check` table; stills "第1天  00:00", "day 1  12:00 AM"); operator in one world | — | — | — |
| H-7 Esc opens/closes; 2D Quit in the menu; 3D mouse released, recaptured on click | operator | operator | operator | operator (pointer capture) |
| H-8 3D golden-hour slice in `en` as accepted | operator | operator (sanity look) | — | — |
| H-9 relaunch: 2D language → 3D opens in it | automated (AC-SET-7); operator by hand | operator | operator | — |
| H-10 the shared folder | automated (AC-SET-15): `~/Library/Application Support/MineWorld` | operator: `%APPDATA%\MineWorld\settings.cfg` | operator: `~/.local/share/MineWorld/`, and `$XDG_DATA_HOME/MineWorld/` | — |

Windows prerequisite: `git config core.symlinks true` (Developer Mode) before checkout; the launchers run
from Git Bash and stop with that message if a module folder is a text file.
- **Commit boundary:** ledger and evidence only. Then mark **READY FOR OPERATOR REVIEW**, with the H-1 … H-10 macOS results and the Windows and Linux checklist listed for the operator.

## 12.6 Acceptance (fixed before measuring)

CP-SET-a (§6) and AC-SET-1 … AC-SET-16 (§7), all `PASS` on the PR's final head (AC-SET-15 on macOS),
and the hand checks H-1 … H-10 (§8). The macOS column is run at the gate; the Windows and Linux columns
are handed to the operator as a checklist, and the operator judges them.

## 12.7 Test ownership

| Test | Owner | Runs in CI |
| --- | --- | --- |
| `tests/acceptance/tests/client_text.rs` | SET-a (new) | yes |
| `tests/acceptance/tests/client_rules.rs` | S14 (I-S14-1); SET-a moves its lexer only | yes |
| `scripts/check_client_rules.py` | S12 | no (run at the gate) |
| `tools/cli/tests/client_settings.rs` | SET-a (new), `#[ignore]` | no, Godot; run at the gate |
| `tools/cli/tests/client_2d.rs` | S12; SET-a must leave it green | no, Godot; run at the gate |
| `clients/shared/checks/*.gd` | SET-a | through `client_settings.rs` |

## 12.8 Risks specific to SET-a

R-SET-1, -2, -3, -7 and -11 (§9). Also: the 2D stub's frame record cannot see a frame that a mutation
sends after the scenario ends. The scenario therefore holds the connection for 2 s after its last
step, and the test asserts that the stub saw the client's `leave`.

## 12.9 Execution contract (frozen 2026-10-08)

| Item | Contract |
| --- | --- |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-set-a`, held by one session only (`CLAUDE.md` §3.1). |
| Branch | `mvp0/pr-set-a`, created from `main` **after S12 13b merges**. Until then the PR does not start. |
| Platforms | The gate runs on macOS. The Windows and Linux columns of §8 go into the PR as an operator checklist, together with the platform line `display.gd` prints. The PR is READY FOR OPERATOR REVIEW with that checklist open; the operator closes it. |
| Allowed commands | `cargo *` (with `$HOME/.cargo/bin/cargo`); `git`; `gh` for PR create and view, never merge; `python3 scripts/*`; `mkdir -p`; `sed -n`; `godot *`; `ln -s` for the two symlinks; the launchers `./mineworld-2d`, `./mineworld-slice`; the CLI binary under `target/`; `gh api` (read-only) to fetch the font, with its sha256 recorded. |
| Not allowed | `python3 -c`, `sed -i`, `awk`, `xargs`, `curl`, heredoc writes. File changes go through Read, Edit and Write. |
| Authority | Commit and push to the branch; open the PR; mark it READY FOR OPERATOR REVIEW; never merge. |
| Material stops | (1) an edit to `clients/protocol/mineworld`, the server, contracts, kernel or any pack; (2) a new runtime dependency or a font other than QSET-1's ruling; (3) an `en` wording change beyond SD-SET-a-12; (4) a 13a, 13b or 16a check that changes result; (5) any harness output that changes under a planted user file (AC-SET-9); (6) a platform-specific code path beyond AC-SET-16's admitted one; (7) a change to any of 13b's keys (SD-SET-a-17 allows a move only). |
| Budget | Godot runs as the checks need; no long town runs. |
| Gate | §12.5 C7. |

## 12.10 Ledger

| Item | Status | Evidence |
| --- | --- | --- |
| C0 (this design) | done, frozen 2026-10-08; anchors re-verified 2026-10-09 against `main @ aee8290` (§12.11) | `plan/client-settings`, PR #91 |
| C1 | done | `3614584` |
| C2 | done | `ff82a50` |
| C3 + C4 | done (one commit) | `42ea4de` |
| C5 + C6 | done (one commit) | `b95e748` |
| merge `origin/main` | done | `72efd61` |
| C7 | done; final code head `873612b` | `ce0b4c1`, `d5921e7`, `873612b`; ledger commit after it |
| Lifecycle | **READY FOR OPERATOR REVIEW** (PR opened; CI on the exact head recorded in the PR) — DO NOT MERGE | — |
| Post-merge synchronization | this session: §12 lifecycle, merge identity and evidence; the primary session: step-20 §§1–11, `overall.md` (S20 row, decision numbers ARC-72/DEP-32/DEP-33 to confirm) | — |
| QSET-1 … QSET-16 | ruled 2026-10-08 (§1.5) | coordinator's relay |
| Windows / Linux checklist (§8) | open, for the operator | — |

Implementation session: 2026-10-09, worktree `/Users/yuema137/mineworld-worktrees/impl-set-a`, branch
`mvp0/pr-set-a` from `origin/main @ aee8290` (13b merged as #103; S11-D, 16a and IL-b on `main`).
Handoff: [`handoff-set-a.md`](handoff-set-a.md).

## 12.11 Findings at implementation (anchors re-verified against `main @ aee8290`)

| Id | Finding | Source | Decision (bounded unless marked) |
| --- | --- | --- | --- |
| F-1 | 13b's `words.gd` API is `load_pack(pack_dir, locale)`, `has`, `text`, `action`, `code`, `reason`, `money`, `clock`, `readable`; its wording-removed switch is `--no-wording` (`app.gd`), not a `--wording=none`. | `clients/2d/scripts/hud/words.gd`; `app.gd` `_ready` | `words.gd` keeps every function and delegates loading and the fallback to `MineWorldText`. §3.6's "new `--wording=none`" is **13b's existing `--no-wording`**, which now removes every layer; no new flag. |
| F-2 | 13b already words the 2D clock through `format.clock` = `day {day}  {hh}:{mm}`, used by the status line and the agenda rows. There is no `clock()` in `status.gd` any more. | `en.po`; `words.gd` `clock`; `readers.gd` | SD-SET-a-7's 2D half: `Words.clock` formats the time with `MineWorldClockFormat`, and `format.clock`'s msgstr becomes `day {day}  {time}` (key unchanged). In 24h the line reads exactly as before; in 12h it gains `h:mm AM/PM` — SD-SET-a-12's named change. |
| F-3 | 13b's hint `ui.hint` ends "Esc: close, quit"; the 2D rejection note's raw JSON is gone (13b words it through `reason.<code>`). | `en.po` `ui.hint`; `app.gd` `_on_resolved` | SD-SET-a-12's 2D items: "Esc: close, quit" → "Esc: close, menu"; the rejection prose item is already done by 13b. |
| F-4 | **A16d-11 / step-13 E-15:** `action.move` = "Walk (click where to go)" is 2D-specific. 16d plans (D-16d-6) a neutral shared entry and a 2D override. | `step-13-client-2d.md` E-15; `origin/plan/s14-16d` step-15 A16d-11, D-16d-6 | Done in SET-a's move: the shared `action.move` reads "Walk"; the 2D pack keeps 13b's entry verbatim under `#. override`. 2D's screen does not change; no key changes. The union check compares the **effective** wording (pack over shared) with 13b's file, entry for entry. |
| F-5 | `check_client_rules.py` walks `clients/2d` without descending symlinked directories (13b's B-10, `os.walk(followlinks=False)`), so the new `mineworld_settings` symlink is never scanned by R1–R6. | `scripts/check_client_rules.py` `scripts_of_client` | R-SET-7 closed for the Python scan by its own code; verified at C5. `client_rules.rs` also skips symlinks and reads `clients/shared/` once where it lives. |
| F-6 | The 2D Godot tests start Godot directly (`godot --headless --path clients/2d -- --drive=…`), not through the launcher. | `tools/cli/tests/godot2d/mod.rs` `Drive::start` | INV-SET-5 is held by the client, not the launcher: `drive` and `capture` are the 2D harness flags that force `--settings=none`. The launchers also pass `--settings=none` to scripted modes (belt and braces, SD-SET-a-11). |
| F-7 | `server/PROTOCOL.md` lists refusal codes in a table (§5.5) but lists no rejection reasons; the kernel's closed reasons are the `Rejection` enum in `contracts/src/action.rs`. | `PROTOCOL.md` §5.5; `contracts/src/action.rs` | AC-SET-4's code families are read from both: every §5.5 refusal code and every unit variant of `Rejection` (snake_case) needs a `reason.<code>` entry in the shared `en.po` and `zh_Hans.po`. |
| F-8 | The 2D client's own interaction menu is `scripts/menu.gd` (13b's `Menu`). | `clients/2d/scripts/menu.gd` | The settings menu is `MineWorldSettingsMenu`; the two never share a name. |
| F-9 | The 3D `SliceIntents.DEFAULT_UTTERANCE` ("Hello! A coffee, please.") is the player's words, sent as the `talk` payload. | `clients/3d-spike/scripts/slice/intents.gd` | Not UI text: translating it would change the request (AC-SET-5). It stays as written, admitted by `client_text.rs`'s allow-list with that reason. |
| F-10 | No decision numbers were allocated to S20 (`overall.md` "Decision numbers assigned since the parallel build-out table"). | `overall.md` §5 | Provisional: `ARC-SET-a` → **ARC-72**; `ARC-SET-b` → a dated note under `ARC-70` (§11.1 allows it); `DEP-SET-a` → **DEP-32**; `DEP-SET-b` → **DEP-33**. Recorded in each entry as provisional, for the primary session to confirm; `check_decision_ids.py` guards a collision. |
| F-11 | Godot 4.7.2's `DirAccessWindows::rename` removes an existing destination and then `MoveFileW`s the source; it does not refuse. | `godotengine/godot` tag `4.7.2-stable`, `drivers/windows/dir_access_windows.cpp` l. 339–346 (read 2026-10-09 with `gh api`) | §3.4's Windows fallback is already Godot's own behaviour. The store therefore never removes the file itself; it recovers a lone `settings.cfg.tmp` on load (the crash window between remove and move), keeps `.bak` before replacing a file that had a load problem, and states the rule in `SETTINGS.md` §4. No platform branch. |
| F-13 | The move puts `action.*`/`reason.*` in the shared layer, so from that commit the 2D client must load it, and AC-SET-4 needs the 2D pack's `zh_Hans.po` as soon as the test exists. | C3's validation | The 2D symlink, `words.gd`'s delegation and the pack's `zh_Hans.po` land in C3 rather than C5 (a reordering inside the frozen file list). |
| F-14 | **Session overlap, 2026-10-09 17:14–17:16.** The coordinator resumed this session twice; for a short while two copies may have run in this worktree. Audit after it: `ps` shows no other process on the worktree; the C2 commit `ff82a50` holds only this session's files and its checks passed on that content; the one foreign change in the tree was `clients/shared/SETTINGS.md` §4's `--settings=<path>` line, rewritten to "(an absolute path, or user://…)", which contradicts the code (relative paths resolve against the working directory). | `git status`, `git show --stat ff82a50`, `ps` | Reverted to the committed text (`git checkout`), and this session continues as the only writer. |
| F-15 | **3D reason wording.** §3.6 has the 3D client word a refusal code or rejection reason through `reason.<code>`, the family whose `en` text is 13b's. On `main` the 3D client shows the code made readable (`busy`, `permission denied`). For the six kernel reasons 13b worded differently (`busy` → "busy right now", `permission_denied` → "not allowed", `no_supported_interaction`, `target_unavailable`, `unavailable`, `precondition_failed`) the 3D toast's English therefore changes; INV-SET-7 lists no such change. | §3.6 "Server codes"; SD-SET-a-12; `slice_link.gd` `_readable` | Read as the frozen design's own consequence (§3.6 governs reason wording; SD-SET-a-17 makes 13b's wording the shared one), not as an extra change: the 3D shown text follows §3.6. To keep everything else unchanged, the refusal codes' `en` entries (new, SET-a's) read exactly as the 3D client showed them (`not joined`). `[link]` log lines keep the readable code (INV-SET-6). **Flagged for the operator** in the PR: if the 3D toast must keep `busy`, a 3D override layer can carry it once 16f gives 3D a pack. |
| F-17 | Main's `platforms` CI layer (Windows, macOS) and its note that Windows checkouts keep the runner's `core.autocrlf`: a CRLF `messages.pot` would fail `client_text`'s byte comparison for a Windows developer (the layer itself does not run `mineworld-acceptance`). | `.github/workflows/ci.yml` l. 60–84; `scripts/ci_layer.py` `platforms` | The comparison normalizes `\r\n`; the `.po` reader already uses `str::lines`. Godot reads `.po` line by line; CRLF catalogs on Windows are part of H-1 on Windows. |
| F-12 | Engine probe (Godot 4.7.2, headless, scratch project outside the repository): `standardize_locale("xx_test")` → `xx`; a `.po` loads from an absolute path inside a `.gdignore` folder; appending Noto Sans SC to `ThemeDB.fallback_font.fallbacks` makes `"设置 Settings"` shape with no tofu and two fonts; `screen_get_refresh_rate()` is `-1` headless. | scratch probe, recorded here | The marker catalog uses locale `qaa` (a private-use code) so it never collides with AC-SET-14's scratch `xx_test` (→ `xx`). Catalogs and the font sit in `.gdignore` folders and are loaded by absolute path, so no `.import` sidecar is ever written through the symlinks. |

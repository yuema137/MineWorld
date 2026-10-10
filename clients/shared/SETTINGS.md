# The MineWorld client settings module

**Audience:** whoever adopts, extends or reviews the shared settings module — the 2D reference client,
the 3D reference client, a third client, or a later PR (SET-b input options, SET-c the World section,
S19 TW-e's HUD date and time). This is a specification, not an introduction; [`README.md`](README.md)
is the orientation.
**Authority:** `docs/DECISIONS.md` `ARC-72` (the module), `ARC-70` and its 2026-10-09 note (wording and
its layers), `DEP-32` (engine APIs adopted), `DEP-33` and `DEP-8`'s font exception (the bundled font).
The design and its rulings are `.structured-coding/plans/mvp0/step-20-client-settings.md` §§3, 5, 12.
Where this document and a decision disagree, the decision governs and this document is the defect.

---

## 1. What a setting is, and what it is not

A **setting** is one person's presentation preference on one machine: the language the client's UI
text is shown in, the clock form, the window and display options. A setting is never part of the world:
it never becomes an `ActionIntent`, a frame or a request, and two players in one world may have
different settings.

A **host command** is a request to the server's administrative surface (`/admin/clock`, S11-D, S19
TW-c): pause, resume, day length. It changes the world for every player, it is sent with the host's
token, and it is **never** a setting. Host commands live in `clients/shared/host/` (SET-c), never in
this module, and are never stored in the settings file.

| Thing | Kind | In the settings file | Reaches the server |
| --- | --- | --- | --- |
| Language | setting | yes, `[general] language` | no |
| Clock 12h / 24h / auto | setting | yes, `[general] clock` | no |
| Window mode, window size, render scale, VSync, frame-rate cap | setting | yes, `[display]` | no |
| Mouse sensitivity, invert Y (SET-b) | setting | yes, `[input]` | no |
| Pause, resume, day length (SET-c) | host command | **never** | yes, as a host command |
| Admin token (SET-c) | credential, from the launcher | **never** | as the host command's bearer header |
| Invite, seat, server address, nickname, world | join arguments | **never** | the join frame, as before |
| "Keep the town running when I close the game" (TW-e) | launcher preference | yes, `[launcher]` | no |

No defined term of `docs/CORE_CONCEPTS.md` is extended by this module. "Setting" and "host command"
are explained here and are not part of the ontology.

## 2. Layout and adoption

```text
clients/shared/                  a Godot project: the shared modules' runnable home, with their checks
  project.godot                  "MineWorld shared client modules"; custom user dir "MineWorld"
  README.md                      human orientation
  SETTINGS.md                    this specification
  settings/                      THE MODULE — what a client takes, as res://mineworld_settings
    settings.gd                  MineWorldSettings       the typed settings value
    store.gd                     MineWorldSettingsStore  locate, load, validate, save (atomic)
    display.gd                   MineWorldDisplay        apply window mode, size, render scale, VSync, cap
    text.gd                      MineWorldText           catalogs, languages, live switch, font fallback
    clock_format.gd              MineWorldClockFormat    a time of day in the 12h or 24h form
    menu.gd                      MineWorldSettingsMenu   the in-game menu (General, Display, later tabs)
    locale/                      the shared catalog layer: en.po, zh_Hans.po, messages.pot, .gdignore
    fonts/                       NotoSansSC-Regular.otf, OFL.txt, .gdignore
  checks/                        headless checks of the module (store, glyphs, menu); not the module
```

**Taking the module.** Copy or symlink `clients/shared/settings` into the client project as
`mineworld_settings`, exactly as `clients/protocol/ADOPTION.md` §1 describes for the protocol module.
The reference clients symlink:

```text
clients/2d/mineworld_settings        -> ../shared/settings
clients/3d-spike/mineworld_settings  -> ../shared/settings
```

On Windows a checkout needs `git config core.symlinks true` (with Developer Mode, or as an
administrator) before cloning or re-checking out; otherwise the symlinks arrive as small text files and
the classes are missing. Both launchers (`./mineworld-2d`, `./mineworld-slice`) stop with a message
naming this fix when `mineworld` or `mineworld_settings` is not a directory. Packaging the clients
without symlinks is a deployment question for S13.

**The module requires:** no autoload, no import step, and no project setting. `locale/` and `fonts/`
each hold a `.gdignore`, so Godot never imports the catalogs or the font into `.godot/`; the module
loads them from disk at runtime by absolute path. The one project setting a client adds is optional:

```ini
[application]
config/use_custom_user_dir=true
config/custom_user_dir_name="MineWorld"
```

With it, every client that sets it shares one settings file per user (§4). Without it, a client keeps a
file of its own in its own user directory.

**The composition root's calls,** first in `_ready`, before it builds its own nodes:

```gdscript
var store := MineWorldSettingsStore.open(OS.get_cmdline_user_args(), HARNESS_FLAGS)
for problem in MineWorldText.load_layers(pack_dir, store.extra_locale_dirs, wording):
    push_warning(problem)
store.settle_language(MineWorldText.locales())
MineWorldText.install_font_fallback()
MineWorldText.set_clock(store.settings.clock)
MineWorldText.set_language(store.settings.language)
MineWorldDisplay.apply(store.settings, get_window(), CAPABILITIES)
```

and later, once its HUD exists, it adds the menu:

```gdscript
var menu := MineWorldSettingsMenu.new()
add_child(menu)
menu.setup(store, CAPABILITIES, theme)
menu.quit_requested.connect(func() -> void: get_tree().quit(0))
```

`HARNESS_FLAGS` is the client's own list of scripted and capture flags (2D: `drive`, `capture`; 3D:
every `--slice-*` probe mode). The module knows no client's flags. `CAPABILITIES` is a
`PackedStringArray` of `MineWorldDisplay.CAP_*` names: the 3D client passes `[CAP_RENDER_SCALE]`, the 2D
client passes nothing. These two inputs and the `Theme` are the only things that differ per client.

**No network code (INV-SET-1).** No file under `settings/` names `MineWorldClient`, `HTTPRequest`,
`WebSocketPeer`, `StreamPeer`, `PacketPeer`, `submit`, `submit_affordance` or `connect_to_world`.
`tests/acceptance/tests/client_text.rs` enforces it.

**No platform branch (AC-SET-16).** No file under `settings/` compares `OS.get_name()` or
`DisplayServer.get_name()`, or asks `OS.has_feature` for a platform, except the one declared case:
`display.gd`'s `fullscreen_same_as_borderless()`, which tells the menu to say, on Wayland, that
Fullscreen behaves as Borderless. Per-platform behaviour is Godot's (§5.1).

## 3. The settings value — `MineWorldSettings` (`settings.gd`)

A `RefCounted` with typed fields and enums, never a `Dictionary`.

| Field | Type | Values | Default | File: section / key / written as |
| --- | --- | --- | --- | --- |
| `language` | `String` | a standardized locale some catalog layer offers (`en`, `zh_Hans`, …) | `"en"` | `general` / `language` / the locale |
| `clock` | `ClockFormat` | `AUTO`, `H12`, `H24` | `AUTO` | `general` / `clock` / `"auto"`, `"12h"`, `"24h"` |
| `window_mode` | `WindowMode` | `WINDOWED`, `BORDERLESS`, `FULLSCREEN` | `WINDOWED` | `display` / `window_mode` / `"windowed"`, `"borderless"`, `"fullscreen"` |
| `window_size` | `Vector2i` | each side within 640×360 … 7680×4320 | `(1600, 900)` | `display` / `window_size` / `Vector2i` |
| `render_scale` | `int` | percent: 50, 67, 75, 85, 100 | `100` | `display` / `render_scale` / integer |
| `vsync` | `VSync` | `OFF`, `ON`, `ADAPTIVE` | `ON` | `display` / `vsync` / `"off"`, `"on"`, `"adaptive"` |
| `max_fps` | `int` | `0` (no cap), 30, 60, 120, 144, 165, 240, or `-1` ("match display") | `0` | `display` / `max_fps` / integer |

`MineWorldSettings.read(config, problems)` builds a value from a `ConfigFile`: every key of the table
that is missing takes its default silently; every key whose type or value is out of range takes its
default and appends one problem `"<section>/<key> = <raw value>: <why>"`; every key the table does not
know, in a section the table owns, appends one problem naming it and is dropped. Sections the table does
not own (`input`, `launcher`, anything later) are not read and not judged. Nothing invalid survives a
read. `write(config, sections)` writes the named sections of the table, replacing each section whole.
`differs_in(other, section)` says whether two values differ in one section.

## 4. The file — `MineWorldSettingsStore` (`store.gd`)

**Where.** `user://settings.cfg`, a Godot `ConfigFile`: `[meta] version=1`, `[general]`, `[display]`,
later `[input]` and `[launcher]`. With the custom user directory of §2, `user://` resolves per OS as
Godot's "Data paths" page states for a custom directory with a custom name:

| OS | `user://` (the shared settings folder) | The file |
| --- | --- | --- |
| Windows | `%APPDATA%\MineWorld\` (typically `C:\Users\<user>\AppData\Roaming\MineWorld\`) | `%APPDATA%\MineWorld\settings.cfg` |
| macOS | `~/Library/Application Support/MineWorld/` | `…/MineWorld/settings.cfg` |
| Linux | `~/.local/share/MineWorld/`, or `$XDG_DATA_HOME/MineWorld/` when `XDG_DATA_HOME` is set | `…/MineWorld/settings.cfg` |

A user's extra catalogs (`user://locale/`, §6) sit in the same folder.
`clients/shared/checks/store_check.gd` prints `OS.get_user_data_dir()` on the OS it runs on and asserts
that its last component is `MineWorld`.

**Choosing the file — `MineWorldSettingsStore.open(args, harness_flags)`.**

```text
--settings=<path>    use that file (absolute, or relative to the working directory)
--settings=none      defaults; never read, never written
a harness flag       any argument --<flag> or --<flag>=… with <flag> in harness_flags behaves as
                     --settings=none, unless --settings=<path> is also given (INV-SET-5)
otherwise            user://settings.cfg
--extra-locale=<dir> (repeatable) a catalog folder loaded after every other layer; used by checks for
                     the marker catalog (§6.5), never by a player
```

`store.settings` is the value in effect; `store.path` is the file (`""` for `none`);
`store.extra_locale_dirs` the `--extra-locale=` folders.

**Loading.**

- A missing file means defaults, silently. If the file is missing but `settings.cfg.tmp` exists and
  parses, a save was interrupted between removing the old file and renaming the new one (Windows, see
  Saving): the `.tmp` is read, with one warning, and the next save puts a file in place.
- An unreadable or unparsable file means defaults and one warning naming the path and Godot's error.
- `[meta] version` above 1 means defaults and one warning; a version that is not an integer is a
  problem as below. A file without `[meta]` is read as version 1.
- Every problem of `MineWorldSettings.read` is one warning naming the file and the key.
- In each of those cases the file is **left byte-identical** until the user applies a change; on that
  save the old content is first copied to `settings.cfg.bak`.
- `store.settle_language(offered)`: a `language` no catalog layer offers (`"tlh"`) is a problem of the
  same kind (one warning naming the file and the key, defaults to `"en"`, file left as it is).

Warnings are `push_warning` lines starting `[settings]`, in English (INV-SET-6).

**Saving — `store.save(value) -> Error`, only on Apply in the menu** (and when the display
confirmation is answered "Keep", §7). With `none` it only replaces `store.settings` in memory.

1. The file is **re-read** first. The sections `value` changes relative to the last value this store
   loaded or saved are written from `value`; every other section — including sections this module does
   not own — is kept exactly as the file now has it. Two clients open at once: the last Apply wins per
   section, so a 3D Apply of the display tab does not undo a 2D language change made a minute earlier.
2. If a load problem is pending, the current file is copied to `settings.cfg.bak`.
3. The result is written to `settings.cfg.tmp` and renamed over `settings.cfg`
   (`DirAccess.rename_absolute`). On macOS and Linux that rename replaces the file atomically. On
   Windows Godot's `rename` removes the existing file and then moves the new one into place (Godot
   4.7.2 `drivers/windows/dir_access_windows.cpp`, `DirAccessWindows::rename`), so a crash between the
   two leaves only the `.tmp`, which the next load recovers (Loading). Never half a file.
4. An unwritable folder or a failed rename is one warning; the value stays in memory for this run and
   the client keeps running.

The file holds settings only (INV-SET-2): `write` writes the keys of §3 and nothing from the command
line, so no invite, token, seat, address, nickname, world id or save path can reach it.

**Precedence at start:** the `--settings` choice → the file → defaults. An engine `--resolution`
argument (a launcher's `--res=`, or a capture mode) wins over the saved window size for that run and is
never saved.

## 5. Display — `MineWorldDisplay` (`display.gd`)

`MineWorldDisplay.apply(settings, window, capabilities)` changes only what differs from the window's
current state, so applying the defaults to a fresh window changes nothing:

| Setting | Godot call |
| --- | --- |
| Windowed | `window.mode = Window.MODE_WINDOWED`; then, unless an engine `--resolution` was given, `window.size = window_size` and the window centred in its screen's usable rect |
| Borderless | `window.mode = Window.MODE_FULLSCREEN` (Godot's non-exclusive full screen: a borderless window covering the screen) |
| Fullscreen | `window.mode = Window.MODE_EXCLUSIVE_FULLSCREEN` |
| Render scale | `window.scaling_3d_scale = render_scale / 100.0`, only with `CAP_RENDER_SCALE` |
| VSync | `DisplayServer.window_set_vsync_mode(DISABLED / ENABLED / ADAPTIVE, window id)`; Mailbox is not offered |
| Frame-rate cap | `Engine.max_fps = max_fps`; `-1` → `round(screen_get_refresh_rate())`, or no cap when that reports `-1` |

Godot changes the monitor's video mode on no platform, in either full-screen mode: "resolution" is the
window size in windowed mode and the 3D render scale in the two full-screen modes; "refresh rate" is the
frame-rate cap plus VSync, and the menu shows the monitor's refresh rate read-only (QSET-3).

Everything is a no-op under the headless display server, where `screen_get_refresh_rate()` is `-1`, so
a headless run can load and apply settings safely. `print_platform_once()` prints one line,
`[settings] platform <OS.get_name()>, driver <rendering driver>, display <DisplayServer.get_name()>`,
so every hand check names its platform.

`size_presets(window)` offers 1280×720, 1600×900, 1920×1080, 2560×1440 and the screen's own size, each
only if it fits the current screen's usable rect, in physical pixels.

### 5.1 Per platform

From the Godot 4.7 `DisplayServer` reference ("The display's video mode is not changed" for both
full-screen modes):

| Setting | Windows (D3D12 or Vulkan) | macOS (Metal, or Vulkan through MoltenVK) | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| Windowed | decorated window at the saved size, centred | same | same | size honoured; position is the compositor's, so centring is a no-op |
| Borderless | a borderless window covering the monitor; the compositor (DWM) stays in the path | its own Space | borderless full screen through the window manager | xdg full screen |
| Fullscreen | one window per screen; a brief black flash is possible on transition | its own Space; Dock and menu bar stay hidden at the screen edge | bypasses the compositor | **the same as Borderless**; the menu's tooltip says so (`ui.settings.display.wayland_same`) |
| VSync | supported; Adaptive needs relaxed-FIFO support, else behaves as On | supported | supported | supported; the compositor may still synchronise with VSync Off |
| Frame-rate cap | all platforms; with VSync On or Adaptive the monitor's rate also limits | same | same | same |
| Monitor rate (read-only) | implemented | implemented | implemented | implemented |
| Render scale | viewport scaling, bilinear | same | same | same |
| HiDPI | sizes are physical pixels; presets are filtered against the usable rect in physical pixels | Retina: sizes are physical, so 1600×900 is half as tall on screen as on a 1× display | scale 1.0 | fractional scales are reported rounded up |

## 6. UI text — `MineWorldText` (`text.gd`)

### 6.1 Format and keys

gettext `.po`, one file per language per layer, loaded at runtime by Godot's `TranslationServer`
(`ARC-70`). Keys are symbolic, lowercase, dot-separated, each segment `[a-z0-9_-]+`. The families:

```text
action.<type>, action.<type>.done   an action, built from data (13b)
reason.<code>                       a rejection reason or refusal code, built from data (13b)
state.<state>                       a connection state, built from data
ui.*  panel.*  suggest.*  format.*  13b's fixed interface text, panel titles, suggestions, formats
ui.settings.*                       the settings menu
hud.*  hint.*  camera.*  link.*     HUD lines and templates, the controls lines, camera titles,
note.*  door.*                      3D connection messages, notes, door labels
language.self_name                  how a language names itself ("English", "简体中文")
clock.default                       the clock form a language prefers: "12h" or "24h"
```

Keys built from data are confined to `MineWorldText.FAMILIES = ["action", "reason", "state"]` and are
built only by `MineWorldText.code(family, code, args)` (and, in the 2D client, by 13b's `words.gd`,
which delegates to it). Placeholders are named (`{target}`, `{place}`) and filled with `String.format`
after the lookup. Plural forms are not used by any shipped key; the merge of §6.2 carries singular
messages.

### 6.2 Layers

Per key, the later layer wins:

| Layer | Folder | Holds |
| --- | --- | --- |
| 1. shared | `settings/locale/` | keys both clients use: `action.*`, `reason.*` (moved verbatim from the 2D pack), `ui.settings.*`, `hud.time.*`, `hud.ampm.*`, `state.*`, `language.*`, `clock.*`; and the 3D client's own keys until the 3D client reads a Presentation Pack (S14 16f), when they move, keys unchanged, to `presentation/mineworld-default/3D/i18n/` |
| 2. pack | `presentation/<pack>/<2D or 3D>/i18n/` | the pack's own wording, and any override of a shared key, marked by an extracted comment `#. override` on the entry |
| 3. user | `user://locale/` | a user's additions or corrections |
| (checks) | each `--extra-locale=` folder | the marker catalog; never shipped |

`MineWorldText.load_layers(pack_dir, extra_dirs, wording)` loads every `*.po` of each layer in order,
merges them per locale into one `Translation` per locale and registers those with `TranslationServer`
(replacing what an earlier call registered). `pack_dir` `""` skips layer 2 (13b's
`--presentation=none`); `wording` `false` skips every layer (13b's `--no-wording`, the
language-independence check). It returns its problems as strings for the caller to warn with.

A **language is offered** when some layer gives its locale a `language.self_name`.
`MineWorldText.locales()` lists them, `languages()` lists `MineWorldText.Language` values
(`locale`, `self_name`), `en` first. Dropping `fr.po` with a `language.self_name` into any layer adds
French with no code.

**Fallback.** A key the selected language lacks is looked up in `en`
(`internationalization/locale/fallback` stays `en`), and a key no layer has is shown as 13b's readable
form of its last part (`reason.too_far_away` → "too far away"). A raw key is never shown.

**Default.** `en` on first launch, whatever the OS locale (QSET-8). A system locale is never consulted;
so Godot's mapping of `zh_TW` onto `zh_Hans` (P-5) never applies.

### 6.3 Live switching

`MineWorldText.set_language(locale)` calls `TranslationServer.set_locale`, and Godot sends
`NOTIFICATION_TRANSLATION_CHANGED` to every node. A label whose whole text is one key may use Godot's
auto-translation. Every label composed in code (a template plus data) sets
`auto_translate_mode = AUTO_TRANSLATE_MODE_DISABLED` and re-renders itself on that notification from
its last data. Nothing needs a restart.

### 6.4 What is translated, and what is not

| Translated (client UI text) | Never translated |
| --- | --- |
| the menu; HUD lines and their templates; the 2D hint and the 3D controls line; camera titles; toasts and notes; door labels' template; connection states; the readable form of a refusal code or rejection reason the server sent | world content: names, place names and tags, dialogue, activity kinds, anything an `Observation` carries (INV-SET-10); in-world signage art; what the player says (the 3D client's default utterance is the player's words and stays as written, so a request is the same in every language); stdout logs, `[link]` lines, `drive:`, `REQUEST`, `EVIDENCE`, `[PASS]`/`[FAIL]` lines and warnings (INV-SET-6) |

Nodes that show world content join the group `mineworld_world_text` and set
`auto_translate_mode = AUTO_TRANSLATE_MODE_DISABLED`.

### 6.5 The marker catalog and the evidence walk

`MineWorldText.visible_ui_texts(root)` walks the tree under `root` and returns `"<node path>\t<text>"`
for every visible, non-empty text of a `Label`, `Button` (and `OptionButton` items), `CheckBox`,
`TabContainer`/`TabBar` title and `RichTextLabel` that is not in (or under) the group
`mineworld_world_text`. The checks use it for AC-SET-1 and AC-SET-2. The marker catalog is generated by a
check from the shared and pack `en.po` files into a scratch folder, with every msgstr wrapped as
`⟦…⟧` and the locale `qaa`, and is passed with `--extra-locale=`; it is never in `settings/locale/`.

### 6.6 The clock — `MineWorldClockFormat` (`clock_format.gd`)

`MineWorldClockFormat.time_of_day(seconds_into_day, clock)` formats with `hud.time.24h`
(`{hour24}:{minute2}`) or `hud.time.12h` (`{hour12}:{minute2} {ampm}`, with `hud.ampm.am` / `.pm`), the
templates of step-19 §8.2, in the current language. `clock` is `H12` or `H24`; `AUTO` (the default)
takes the current language's `clock.default` (`en` 12h, `zh_Hans` 24h, QTW-16). `MineWorldText.set_clock`
holds the setting; `MineWorldText.clock()` answers the form in effect. TW-e adds the date entries of
step-19 §8.2 to the same file.

### 6.7 The font

Godot's default font has no CJK glyphs. `MineWorldText.install_font_fallback()` loads
`fonts/NotoSansSC-Regular.otf` at runtime (`FontFile.load_dynamic_font`) and appends it to the
fallbacks of `ThemeDB.fallback_font`. Latin text keeps its font glyph for glyph; any glyph the default
font lacks comes from Noto Sans SC before the system is asked. The chain is the same on every OS (the
default font → Noto Sans SC → the system), and Godot rasterizes with its own FreeType on all three, so a
bundled glyph has the same shape everywhere. The system fallback (Windows' font set, macOS CoreText,
Linux fontconfig — possibly no CJK font at all) is reached only by characters the bundled fonts lack,
never by shipped text (INV-SET-9, checked by `clients/shared/checks/glyph_check.gd`).

The font is the unmodified upstream file (`notofonts/noto-cjk`,
`Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`, commit `165c01b46ea533872e002e0785ff17e44f6d97d8`,
8,331,336 bytes, SHA-256 `faa6c9df652116dde789d351359f3d7e5d2285a2b2a1f04a2d7244df706d5ea9`), under
OFL-1.1 (`fonts/OFL.txt`), admitted by `DEP-8`'s font exception; `NOTICE` names it.

## 7. The menu — `MineWorldSettingsMenu` (`menu.gd`)

A `CanvasLayer` the client adds and sets up with `setup(store, capabilities, theme)`.

- **Opening.** The client decides the key: both reference clients open and close it with Esc
  (QSET-2). `open()`, `close()`, `toggle()`, `is_open()`; signals `opened`, `closed`, `quit_requested`.
- **Tabs.** **General**: Language (the offered languages, each by its `language.self_name`), Clock
  (Automatic, 12-hour, 24-hour). **Display**: Window mode, Window size (in windowed mode) or Render scale
  (in the full-screen modes, only with `CAP_RENDER_SCALE`), VSync, Frame-rate cap, the monitor's refresh
  rate read-only, the measured frame rate. Buttons: Apply, Revert, Close; Quit at the bottom.
- **Language and clock apply at once** and are saved on Apply. **Display changes apply on Apply**,
  followed by a 15-second "Keep these display settings?" confirmation; no answer, or Revert, re-applies
  the previous display settings. A kept change is saved; a reverted one is not. An unusable mode never
  strands a player.
- **Revert** returns every control and the live language and clock to the store's saved value.
- **The world keeps running.** The menu pauses nothing. While it is open it covers the screen with a
  control that stops the mouse, and the client stops gameplay input (2D: the walker; 3D: walking,
  looking and the talk key), so no request is sent from input while the menu is open.
- **Tab API** for later PRs: `add_tab(key, control)` adds a tab titled by the key's text; SET-b and
  SET-c use it and never edit the existing tabs.
- **Per client,** only the `Theme` and the capabilities differ; the menu decides nothing else per client.

## 8. Invariants

1. **INV-SET-1 — Settings never reach the server.** §2 "No network code"; a client's frames are the
   same with any settings as with the defaults.
2. **INV-SET-2 — The settings file holds only settings.** §4.
3. **INV-SET-3 — Host commands are not settings.** §1; `clients/shared/host/` never loads `store.gd`.
4. **INV-SET-4 — Every client UI string comes from a catalog,** apart from an allow-list in
   `tests/acceptance/tests/client_text.rs` where each entry gives its reason and an entry that admits
   nothing fails. World content and signage art are not UI strings.
5. **INV-SET-5 — Harnesses are independent of the user's settings.** §4 harness flags.
6. **INV-SET-6 — Logs stay English.** Stdout, `[link]`, `drive:`, `EVIDENCE`, `[PASS]`/`[FAIL]` lines
   and warnings are identical whatever the language.
7. **INV-SET-7 — `en` is unchanged,** apart from step-20 SD-SET-a-12's named changes ("Esc: close, quit"
   → "Esc: close, menu"; the 3D "Esc release mouse" → "Esc menu"; the clock line gains the 12-hour form).
   Latin glyphs come from the same font as before.
8. **INV-SET-8 — One module, two clients.** Neither client has its own copy of a setting, a catalog
   entry or a display call.
9. **INV-SET-9 — Every shipped character has a bundled glyph.** §6.7.
10. **INV-SET-10 — World content is never translated.** §6.4.

## 9. Checks

| Check | What it holds | Runs |
| --- | --- | --- |
| `tests/acceptance/tests/client_text.rs` | no hardcoded UI string in the 3D client and the shared module (AC-SET-3); catalogs complete and consistent across layers, and `messages.pot` current (AC-SET-4); no network identifier under `settings/` (INV-SET-1); no platform branch (AC-SET-16) | `cargo test`, in CI |
| `scripts/check_client_rules.py` R6 | no hardcoded UI string in the 2D client (13b) | at the gate |
| `clients/shared/checks/store_check.gd` | loading, validation, the bad-file cases, the atomic save, the per-section merge, the shared folder (AC-SET-8, AC-SET-15) | `godot --headless --path clients/shared --script res://checks/store_check.gd` |
| `clients/shared/checks/glyph_check.gd` | every shipped character is in the bundled font (AC-SET-13) | the same, `glyph_check.gd` |
| `clients/shared/checks/menu_check.gd` | the menu's language switch, Apply, Revert, the revert timer, `add_tab`, the marker catalog on the menu alone | the same, `menu_check.gd` |
| `tools/cli/tests/client_settings.rs` | the runtime criteria in both clients (AC-SET-1, -2, -5 … -12, -14, -15) | `cargo test -p mineworld-cli --test client_settings -- --ignored --test-threads=1` (needs Godot) |

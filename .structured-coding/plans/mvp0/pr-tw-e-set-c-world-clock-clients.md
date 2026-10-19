# PR S19 TW-e + S20 SET-c — The world clock in the clients: HUD date and time, the sun, and the World section

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: recommended answers; Q-WC-1 add `POST /admin/stop`)

```text
Design revision:        revision 1 (2026-10-18), this file as first committed on docs/pr-design-tw-e-set-c
Approved by / evidence: the operator, 2026-10-18, in the primary session: every §14 question as recommended; Q-WC-1 (a)
Implementation base:    origin/main at C0 of the implementation session; planned against
                        origin/main @ b85f2804 (#260; TW-c #199 merged as 96b41db0)
Execution contract:     §13 (filled at freeze, 2026-10-18)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Steps:** S19 [`step-19-time-weather.md`](step-19-time-weather.md) (§1.3, §7.5, §8,
§10 INV-TW-6, §11.2 the TW-e row, §14.1) and S20 [`step-20-client-settings.md`](step-20-client-settings.md)
(§3.2, §3.10, §6 the SET-c row, §5 INV-SET-1 … -3, §1.5 the all-platform requirement) · **Parent:**
[`overall.md`](overall.md) S19, S20, "Framework, not demo" item 4, "One world, two views" · **Builds on:**
[`pr-tw-c-host-clock.md`](pr-tw-c-host-clock.md) (merged #199; its §11 places requirements on this PR),
SET-a (step-20 §12, merged #117), [`pr-set-b-controls.md`](pr-set-b-controls.md) (merged), S12 13a (#82),
S14 16a (#79) · **Binding specs:** `CLAUDE.md` §§1.1, 2–4; `docs/ENGINEERING_RULES.md` §§2–3, 8–12;
`docs/ENGINEERING_STANDARDS.md`; `docs/REUSE_POLICY.md`; `docs/DECISIONS.md` `ARC-67`, `ARC-69`, `ARC-70`,
`ARC-76`, `DEP-30`, `DEP-35`; `server/PROTOCOL.md` §§5.7, 5.9, 11; `clients/shared/SETTINGS.md`;
`clients/protocol/ADOPTION.md` §1.

**Working name:** TW-e + SET-c (one PR). The PR number is assigned when the implementation PR opens.
**Planning worktree:** `/Users/yuema137/mineworld-worktrees/design-tw-e-set-c`, branch
`docs/pr-design-tw-e-set-c`, from `origin/main @ b85f2804`. This planning session writes this file and
two dated pointer notes in the step documents (§12). It implements nothing.

---

## 0. Why one PR, and the ownership ruling it makes

Step-19 §11.2's TW-e row (line 786) still lists "the World section of the settings menu (pause, day
length) via `/admin/clock`". Step-20 §3.10 and its SET-c row (line 639) claim the same section, and
QSET-6 (primary session, 2026-10-08) moved it to SET-c — but the amendment step-20 §11.3 proposed for the
TW-e row was never applied, so the two step documents contradict each other (`CLAUDE.md` §2.1 rule 4).

**This design settles it with one owner per artefact (SD-WC-1):**

| Artefact | Single owner | Module / file |
| --- | --- | --- |
| The World tab of the settings menu: pause/resume, day length 4 h · 2 h · 1 h, the read-only view and paused badge, pending/confirmed state, error words | **SET-c** | `clients/shared/host/` (`res://mineworld_host`) |
| The admin token in the client (receipt from the launcher, memory only, bearer header) | **SET-c** | `clients/shared/host/` |
| The world-time estimate, the day record reader, date roll-over, sun interpolation, light phase | **TW-e** | `clients/shared/world_time/` (`res://mineworld_world_time`) |
| The HUD date and time, `Paused` in the HUD, 12h/24h formatting of the date | **TW-e** | `clients/shared/settings/clock_format.gd` (extended; SETTINGS.md §6.6 already assigns it) |
| 3D key light, sky dimming and far-field light from the sun; 2D tint | **TW-e** | client files (§6) |
| "Keep the town running when I close the game" (launcher preference, `[launcher]`) and graceful close | **TW-e** | settings module (`[launcher]`, General tab row) + launchers |
| Token generation, `--solo`, hand-over in the launchers | **TW-e** (launcher) generates and passes; **SET-c** consumes | `mineworld-2d`, `mineworld-slice` |

The two halves are delivered in one PR because the checkpoint the operator asked for in step-19
(CP-TW-e: "pause freezes it and shows `Paused`; 1 h day length makes it advance 24 per minute") is only
observable through SET-c's buttons, and SET-c's own checkpoint ("HUD shows `Paused` only after the `clock`
frame") is only observable through TW-e's HUD. Splitting them would leave each PR with a checkpoint it
cannot run. The commits keep the halves separable (§10: C3 and C4–C5 are TW-e; C6 is SET-c), so a
reviewer reads each owner's code in isolation, and the dependency is one-way: `host/` reads nothing of
`world_time/`, and `world_time/` reads nothing of `host/`; both read the protocol module's `clock_changed`.

C1 applies the ruling to step-19 §11.2 (the TW-e row loses the World section and gains "uses SET-a's
`MineWorldClockFormat`, catalogs and the store's `[launcher]` section", exactly as step-20 §11.3 proposed).

---

## 1. Goal, in one paragraph

A single-player session started from either launcher shows the world's date and time in the HUD, in the
player's language and clock form, advancing live at the host's scale, holding still and saying `Paused`
when the host pauses, and changing rate the moment the host changes the day length. The 3D sun stands where
San Diego's sun stands at that world instant, disclosed by the server's `calendar` System Pack and only
interpolated in the client; the 2D town is tinted by the same sun. The host — and only a client holding the
host's token — pauses, resumes and chooses a day length of 4 h, 2 h or 1 h from the settings menu's World
tab; every other client sees the same state read-only. Closing the window stops the host gracefully, so
reopening resumes the same minute, unless the player chose to keep the town running. No client computes a
date or a sun, no setting reaches the server, the server stays the only authority for time, and the two
clients share one settings store and one world-time module on macOS, Linux and Windows.

---

## 2. Audit — what exists on `origin/main @ b85f2804`

| Id | Finding | Where | Consequence |
| --- | --- | --- | --- |
| A-1 | **The `clock` frame reader exists.** `signal clock_changed(at, time_scale, paused)`; `var paused`, `var time_scale` set from each frame. The protocol module is S11's and is not edited. | `clients/protocol/mineworld/world_client.gd:78–81, 184–188, 521–526` | TW-e and SET-c both subscribe to it; neither edits the module. |
| A-2 | **The `clock` frame is sent after `welcome`, on every pause, resume and rescale** (TW-c). `at` never moves on a rescale. | `server/PROTOCOL.md` §5.9 | The estimate re-anchors on every frame. |
| A-3 | **Stale protocol text after TW-c merged:** the frame table (§5, line 272) still says "on every pause and resume"; the landing table (line 832) still says TW-c's row is "specified; lands in TW-c's own commits". | `server/PROTOCOL.md:272, 832` | C1 fixes both (doc defect, `CLAUDE.md` §2.1 rule 4). |
| A-4 | **`/admin/clock` accepts `{paused}`, `{time_scale}`, or both**; 1…24 always, >24 only `--solo` (409 `fast_forward_solo_only`), 400 `malformed`, 401 `unauthorized` (≥ 500 ms), 503 `world_stopped`; admin surface exists only with `--admin-token` / `MINEWORLD_ADMIN_TOKEN`, else every `/admin` path is 404. Same listener as `/status`. | `server/PROTOCOL.md` §§11.1–11.2; `tools/cli/src/main.rs:119`; `tools/cli/src/serve.rs:47–204` | SET-c sends exactly 6, 12, 24 and handles every code (TW-c §11). |
| A-5 | **Graceful stop is Ctrl-C only on Unix.** `stop_requested()` waits on `ctrl_c()` (SIGINT) on Unix; on Windows also Ctrl-Break, console close, logoff, shutdown. SIGTERM is not handled. | `tools/cli/src/serve.rs:385–429` | Both launchers stop their server with `kill "$server_pid"` (SIGTERM) in an `EXIT` trap — **not graceful**: no checkpoint, no `Stopped` journal record, so "reopening resumes the same minute" (CP-TW-e) does not hold today, and from Git Bash on Windows a native process gets no console control event at all. §5.6, Q-WC-1. |
| A-6 | **No route stops the server.** No `/admin/stop`; no protocol frame for it. | `server/PROTOCOL.md` §11.2 | Graceful close needs either a new admin route (public contract) or launcher signals that do not exist on Windows. Q-WC-1. |
| A-7 | **Neither launcher passes `--admin-token` or `--solo`.** 2D: `server … --listen 127.0.0.1:0 --save "$save"`; 3D `--world`: `server worlds/social-cafe --listen 127.0.0.1:0 --agent alice`, **no `--save`**. | `mineworld-2d:130`; `mineworld-slice:136` | TW-c §11 requires the launcher to start `--solo` with a generated token. The 3D `--world` world is not persisted, so a 3D restart cannot resume the minute (Q-WC-4). |
| A-8 | **Only `market-town` enables `calendar` (and `weather`).** `social-cafe` (the 3D slice's world) and `lakeside` do not; `bodies-yard` neither. `market-town` and `social-cafe` set `hosting: time_scale: 12`. | `worlds/*/world.yaml`; `worlds/market-town/configure/calendar.yaml` | In 3D, today, there is no day record: the 3D client would show `Day N` and the fixed sun forever (INV-TW-6). CP-TW-e in 3D needs `social-cafe` to opt into `calendar` (content, new goldens) — Q-WC-3. |
| A-9 | **The calendar discloses two records on the observer's place only:** `calendar-day` (`date {year, month, day}`, `weekday` 0 = Monday, `day_start`, `events` {astronomical_dawn … astronomical_dusk, each seconds after day start or null}, `track`: 97 `{elevation, azimuth}` samples every 900 s in millidegrees) and `calendar-light` (`phase`: night, astronomical-twilight, civil-twilight, day). "An observer in no place gets none." | `systems/calendar/src/{lib,day,component,system}.rs` | The client reads both with `observation.component(observation.place(), "calendar-day")`. It must **keep the last record** while the observer is in no place (in transit), not fall back to `Day N` each time. |
| A-10 | **San Diego coordinates** in `market-town`: `latitude: 32.7157`, `longitude: -117.1611`, `utc_offset: -08:00` (fixed, QTW-9), epoch 2026-10-08. TW-a verified the sun against the NOAA GML Solar Calculator at exactly these coordinates (`https://gml.noaa.gov/grad/solcalc/table.php?lat=32.7157&lon=-117.1611&year=2026`, step-19 §16 ledger line 1065). The coordinates' own primary source is not recorded in the repository. | `worlds/market-town/configure/calendar.yaml`; step-19 l. 393, 1065 | The client takes the sun from the record and uses no coordinate. The provenance gap is recorded (F-WC-3, Q-WC-9). |
| A-11 | **The clock formatter exists:** `MineWorldClockFormat.day_time(seconds)` (`hud.day_time`, "day 1  7:42 PM") and `time_of_day(seconds_into_day, clock)` with `hud.time.12h/24h`, `hud.ampm.am/pm`. Its header says "TW-e adds the date here". No `hud.date`, `hud.datetime`, `hud.paused`, weekday or month entries exist. | `clients/shared/settings/clock_format.gd`; `locale/en.po:21–37`; `SETTINGS.md` §6.6 | C3 adds them, keys from step-19 §8.2. |
| A-12 | **Both HUDs format `latest.at()` once per observation** — 2D in `status.gd:58` (`ui.status.when`, refreshed in `show_state`), 3D in `slice_main.gd:526` (`_hud_time`, refreshed in `_process` from `link.client.latest.at()`). Neither ticks between frames, reads `clock_changed`, or shows paused. | as named | Both move to the shared estimate (C4, C5). |
| A-13 | **3D sun is fixed golden hour:** `SUN_ELEVATION := -19.3`, `SUN_AZIMUTH := -48.0`, `SUN_ENERGY := 4.4`, `SlicePalette.SUN_WARM`; a `SkyFill` light; `SKY_AMBIENT`; fixed exposure (auto exposure off). The sky is the far field's declared sky: a **golden-hour HDRI panorama** (`PanoramaSkyMaterial`, `energy_multiplier 0.80`) or a procedural fallback. | `clients/3d-spike/scripts/slice/slice_main.gd:27–35, 255–363`; `palette.gd:49` | A moving sun needs an energy and colour function whose golden-hour value is exactly ARC-13's, and a night that does not sit under a golden-hour panorama (§6.3, Q-WC-5). |
| A-14 | **The far field's haze reads the sun constants:** `_haze()` uses `SliceMain.SUN_ENERGY * sin(-SUN_ELEVATION) + SKY_AMBIENT * 0.8`. | `clients/3d-spike/scripts/slice/far_field_view.gd:293–298` | A night sun with golden-hour land radiance would leave a lit horizon under a dark sky ("always a far field" still holds, but it would look wrong). C5 drives it from the current light (§6.3). |
| A-15 | **Probes pin the sun:** `slice_probe.gd:729` asserts sun elevation 12…25°, `:813` reads `Sun`; `slice_far_field_check.gd`, `slice_perf.gd` read `DirectionalLight3D` shadows. | as named | Every scripted mode keeps the fixed golden-hour sun (§6.3: no `calendar` record ⇒ constants; harness modes pass `--sun=fixed`), so no capture or perf baseline moves. |
| A-16 | **`slice_main.gd` is 590 lines** (above the ~500 review threshold), `app.gd` 415, `slice_link.gd` 603. | `wc -l` | New 3D logic goes in a new `slice_sun.gd`, not into `slice_main.gd`; `slice_main.gd` gains calls only (§6.3). |
| A-17 | **Shared modules are symlinked** as `mineworld_<module>`: `mineworld`, `mineworld_settings`, `mineworld_environment` in both clients. Both launchers refuse to start when a module is not a folder (Windows `core.symlinks`). | `ls -la clients/2d clients/3d-spike`; `mineworld-2d:86–92`; `mineworld-slice:79` | Two more symlinks, `mineworld_world_time` and `mineworld_host`, and the launcher loops gain them. |
| A-18 | **The static scans name their folders explicitly:** `client_text.rs` scans `clients/3d-spike/scripts` and `clients/shared/settings` for literals (AC-SET-3) and `clients/shared/settings` alone for network identifiers (INV-SET-1); `client_rules.rs` refuses action-type literals outside an allow-list. | `tests/acceptance/tests/client_text.rs:59–73, 599, 798, 848` | C3 and C6 add `clients/shared/world_time` and `clients/shared/host` to the literal scan; `world_time/` joins the network-identifier scan; `host/` is the one declared exception for `HTTPRequest` and gets the inverse rule (no `store.gd`, no `MineWorldSettingsStore`, INV-SET-3). |
| A-19 | **No `[launcher]` section exists** in the store or settings value, though SETTINGS.md §1's table reserves it for TW-e. | `clients/shared/settings/{store,settings}.gd` | C7 adds it with one key (§5.5). |
| A-20 | **Test seams exist:** `tools/cli/tests/godot2d/mod.rs` `StubWorld` (a WebSocket stub that records the frames it receives and sends scripted ones); `client_settings.rs` (Godot runtime tests, `#[ignore]`); `host_clock.rs` (real-binary host clock); `clients/protocol/checks/admin_check.gd` (HTTP from GDScript with `HTTPRequest`, `run.sh admin`). | as named | The runtime tests reuse them (§8). The stub gains scripted `clock` frames and a tiny `/admin/clock` HTTP responder for the "never confirms" and 401 cases. |
| A-21 | **CI runs no Godot** (A-SET-12): Godot tests are `#[ignore]` and run at the gate; static scans run in `cargo test`. | step-20 §2.1 | Same split here (§7). |
| A-22 | **TW-c ledger findings that bind this PR.** F-TWc-1 (ledger): shifts start with nobody present — content, unrelated to clients; a client must not "fix" lateness by reading the scale. F-TWc-2 (ledger): `run` cannot tell scales apart, so every rate claim here is measured against a **hosted** server, never `run`. F-TWc-4: the 24× segment of AC-TWc-7 is the noisy one (six stride pairs, 0.04-point miss) — rate tolerances here are set from wall-clock estimates over ≥ 10 s, never over a handful of samples. F-TWc-5: the "> 24 needs `--solo`" check at start lives only in the CLI (`serve.rs`) and live in `admin.rs` — the launcher must start through the CLI with `--solo`, and SET-c must word `409 fast_forward_solo_only` though the menu never offers > 24. (Planning F-TWc-1/-2 — jitter-free consult instants and the straddling answer window — are server-side and resolved in TW-c; they guarantee that a rescale from the World tab does not change a hosted Person's wall cadence, which this PR relies on and does not re-test beyond AC-WC-8's rate check.) | `pr-tw-c-host-clock.md` §2, §15 | §§5–8. |

---

## 3. Scope

### 3.1 In scope

1. **`clients/shared/world_time/`** (TW-e): `MineWorldWorldTime` — anchor from `clock` frames and
   observations, a monotonic wall-clock estimate, the never-backwards display rule, the last day record,
   date roll-over at 24:00 until the next record, sun interpolation over the 97-sample track, the light
   phase from the disclosed record and day events. A headless check over a recorded stream.
2. **Date formatting** in `clock_format.gd` (TW-e): `hud.date`, `hud.datetime`, weekday and month names,
   `hud.paused`, `en` and `zh_Hans`; the glyph check covers them.
3. **2D client** (TW-e): HUD date/time from the estimate, `Paused`, a `CanvasModulate` tint indexed by sun
   elevation (table as 2D Presentation Pack content).
4. **3D client** (TW-e): `slice_sun.gd` — key light direction from the interpolated sun, energy and colour
   from elevation, sky and far-field light from it; HUD date/time and `Paused`; fixed golden hour when no
   record is disclosed or in any harness mode.
5. **`clients/shared/host/`** (SET-c): `MineWorldHostClock` (POST `/admin/clock`, pending/confirmed) and
   `MineWorldWorldTab` (the World tab, through `menu.add_tab`), token from the environment only.
6. **Launchers** (TW-e): generate an admin token, start the server `--solo` with it in the environment,
   hand it to the client by environment; persist the 3D `--world` save (Q-WC-4); graceful close and "keep
   running" (§5.6).
7. **`[launcher] keep_running`** in the settings value and store, one row in the General tab, shown only to
   a client its launcher hosts.
8. **Server, conditional on Q-WC-1:** `POST /admin/stop` (graceful shutdown with the admin token).
9. **World content, conditional on Q-WC-3:** `social-cafe` opts into `calendar` (San Diego), its own commit
   with new goldens.
10. Specifications first: `clients/shared/WORLD_TIME.md`, `clients/shared/HOST.md`, `SETTINGS.md` §§1,
    6.6, 7; `PROTOCOL.md` stale lines (A-3) and, if ruled, §11.2 `/admin/stop`; decision notes (§11);
    step-19 §11.2 ownership amendment.

### 3.2 Not in scope

| Item | Where |
| --- | --- |
| Weather visuals, condition → presentation intent mapping, Sky3D, rain, fog, GI re-check under a moving sun | TW-f (Q-WC-6: the intent mapping moves to TW-f, which is its first consumer) |
| Fast-forward ("doze") from the client | S21 (QRT-7); the World tab never offers > 24× |
| Moon | QTW-11 (later calendar PR) |
| A HUD that shows a world without `calendar` anything but `Day N, HH:MM` | INV-TW-6, unchanged |
| Any change to `kernel/`, `contracts/`, `systems/`, `cognition/`, `clients/protocol/mineworld/` | never |
| Multiplayer host UI beyond "token holder = host" (a LAN operator passes `MINEWORLD_ADMIN_TOKEN` to their own client by hand) | documented in `HOST.md`; a launcher for it is S23 |
| The promenade (`./mineworld-3d`) | QSET-9: runs on defaults, gets no World tab and no moving sun |

### 3.3 Invariants this PR must hold

| Id | Invariant |
| --- | --- |
| INV-TW-2 | No world rule reads pacing. Nothing in this PR adds a server-side reader of scale or pause. |
| INV-TW-3 | Kernel and contracts unchanged. |
| INV-TW-5 | Integers on the wire. The client converts millidegrees to floats locally only. |
| INV-TW-6 | **Clients only render.** Neither client computes the sun or the date from coordinates or the OS clock; both read disclosure through `world_time/`. Without a `calendar-day` record: today's fixed light and `Day N`. The wall clock is used only to advance the estimate between frames, and only through a monotonic source (`Time.get_ticks_usec()`). |
| INV-TW-8 | The displayed clock never runs backwards within a session (§5.2). |
| INV-SET-1 | Settings never reach the server. `world_time/` and `settings/` contain no network identifier; `keep_running` changes only what the client does on close. |
| INV-SET-2 | The settings file holds only settings: no token, invite, address, scale or pause state. |
| INV-SET-3 | Host commands are not settings. `host/` never loads `store.gd`; the token lives in memory only and appears in no file, log, evidence line or capture. |
| INV-SET-8 | One module, two clients: both take `world_time/` and `host/` by symlink; neither has its own estimate, formatter, sun interpolation or HTTP code. |
| INV-WC-1 (new) | **No optimism.** The HUD's `Paused`, the World tab's state and the displayed day length change only on a `clock` frame (or the `welcome` summary), never on a click or an HTTP 200. |
| INV-WC-2 (new) | **Harness independence of the sun.** Every scripted or capture mode of the 3D client renders ARC-13's golden-hour constants, bit for bit as on `main`, whatever the world discloses. |
| INV-WC-3 (new) | **One-way module dependency.** `world_time/` → protocol module (reader) only; `host/` → protocol module (signal) and `settings/` text only; `settings/` → neither. |
| ARC-25 / INV-TW-1 | A world that does not enable `calendar` produces byte-identical `run` output, saves and fact logs. If Q-WC-3 opts `social-cafe` in, that world's goldens change in that one content commit only, and `bodies-yard`, `lakeside`, `market-town` stay byte-identical (`scripts/ci_parity.py`). |

---

## 4. Reuse (`REUSE_POLICY.md`; both halves of the question)

| Need | Options compared | Verdict |
| --- | --- | --- |
| Sun position in the client | (a) server-disclosed track, interpolated (step-19 §3.1 chose this; `DEP-30`); (b) a GDScript solar library (e.g. a port of suncalc) in each client; (c) Sky3D's `TimeOfDay` astronomy | **(a)**, already decided. (b) and (c) are a second astronomy in the client — a parity defect by construction and a breach of INV-TW-6. Sky3D is adopted for *rendering* in TW-f with its astronomy off. |
| Date formatting / localisation | Godot `Time.get_datetime_string_from_unix_time` (English-only, no weekday names, uses Unix time not world time); ICU via GDExtension; our catalog templates (step-19 §8.2, ARC-70) | **Catalog templates through `MineWorldText`**: the date fields come from the record, not from a clock, so there is nothing for a date library to compute; ICU is a native dependency for four templates. Godot's helpers are rejected for fit (language, source of time), not preference. |
| Date roll-over past 24:00 before the next record | Godot `Time` (Unix-epoch helpers); our ~15-line civil successor (`next_date(y, m, d)`) mirroring the calendar pack's `days_in_month` | **Ours**, one function, shared, tested against the pack's own leap rule on 2028-02-28/29 and 2026-12-31. `Time.get_date_dict_from_unix_time` would work but reintroduces a Unix-time path a reviewer must prove never touches the OS clock. Recorded as a bounded "build" with this reason. |
| Monotonic wall clock | `Time.get_ticks_usec()` (monotonic, all platforms); `Time.get_unix_time_from_system()` (wall, can jump) | **`get_ticks_usec`**. |
| HTTP to `/admin/clock` | Godot `HTTPRequest` (already used by `admin_check.gd`); `HTTPClient`; a WebSocket admin channel | **`HTTPRequest`** — engine-native, all three platforms, loopback works; no new dependency. |
| Settings-menu tab | `MineWorldSettingsMenu.add_tab` (SET-a's tab API) | **Adopt**, no edit to `menu.gd`'s tabs. |
| Token generation in bash | `openssl rand -hex 16` (not on every minimal Linux); `od -An -tx1 -N16 /dev/urandom` (POSIX, present in Git Bash on Windows); `$RANDOM` (not cryptographic) | **`od … /dev/urandom`**, with a refusal if it yields fewer than 32 hex digits. |

No new dependency is proposed.

---

## 5. Design — the shared modules and the launcher contract

### 5.1 `clients/shared/world_time/` — `MineWorldWorldTime` (SD-WC-2)

```text
clients/shared/world_time/
  world_time.gd     MineWorldWorldTime   anchor, estimate, display rule, day record, roll-over, sun, phase
  civil.gd          MineWorldCivil       next_date(), days_in_month(), is_leap() — integers only
```

A `RefCounted` a client owns one of. **Inputs, all from the server:**

- `on_welcome(world_summary)` — `at`, `time_scale`, `paused`: the first anchor; resets the display rule
  (a new session may legitimately be earlier, e.g. a killed host; §5.2).
- `on_clock(at, time_scale, paused)` — from `MineWorldClient.clock_changed`: re-anchor.
- `on_observation(observation)` — re-anchor at `observation.at()` (with the current scale and pause
  state), and read `calendar-day` / `calendar-light` about `observation.place()`. A frame without them
  keeps the last record (A-9).

**Wall clock:** `now_usec` is a parameter of every query (`estimate(now_usec)`), supplied by the caller as
`Time.get_ticks_usec()`. Tests pass a scripted value. Nothing in the module reads any clock itself.

**Estimate:** `at_est = at_anchor + (paused ? 0 : floor((now_usec − anchor_usec) × time_scale / 1e6))`,
integers (`int` is 64-bit in GDScript).

**Queries (pure functions of state and `now_usec`):** `display_seconds(now_usec) -> int`;
`has_calendar() -> bool`; `date_at(seconds) -> {year, month, day, weekday}` (the record's date, rolled by
`MineWorldCivil.next_date` for each whole day past `day_start + 86 400` until a newer record arrives);
`seconds_into_day(seconds)`; `sun_at(seconds) -> {elevation_deg, azimuth_deg, valid}` (linear between the
two neighbouring samples, azimuth unwrapped across 0/360°; `valid = false` past the record's 24:00 sample,
when the client holds the last sample instead of extrapolating astronomy); `phase_at(seconds)` (the
record's `calendar-light` phase, advanced across the day record's own `events` times — reading disclosed
instants, not computing them); `paused()`; `time_scale()`.

### 5.2 The display rule — never backwards (SD-WC-3)

`display = max(last_display, at_est)` within a session. If a re-anchor puts `at_est` behind what was shown
(the estimate ran ahead of a late frame), the displayed minute holds until the world catches up; it is never
set back. Reset only by `on_welcome` (a new session), because TW-c's `Stopped` record makes a graceful
restart resume at or after the last instant shown, and a killed host may resume earlier — that one case
is allowed to step back once, at the new `welcome`, and is documented in `WORLD_TIME.md`.

### 5.3 Formatting (SD-WC-4)

`MineWorldClockFormat` gains `date(d: Dictionary) -> String` and `date_time(world_time, now_usec) ->
String`. With a record: `hud.datetime` = `{date}, {time}` (`en`) / `{date} {time}` (`zh_Hans`), `hud.date`
as step-19 §8.2, time in the form in effect (12h/24h/auto). Without one: the existing `hud.day_time`
(unchanged, INV-TW-6). Paused: `hud.paused` (`Paused` / `已暂停`) appended by the HUD as its own label, not
inside the time template, so the clock string a parity test compares stays one string. New keys (both
catalogs, symbolic, `msgctxt`-free):

```text
hud.date            {weekday_short} {day} {month_short} {year}   |  {year}年{month}月{day}日 {weekday_long}
hud.datetime        {date}, {time}                                |  {date} {time}
hud.paused          Paused                                        |  已暂停
hud.weekday.short.0 … .6   Mon … Sun (0 = Monday, as the pack)    |  周一 … 周日
hud.weekday.long.0 … .6    Monday … Sunday                        |  星期一 … 星期日
hud.month.short.1 … .12    Jan … Dec                              |  1月 … 12月 (unused by zh's template; kept complete)
```

Examples fixed now (8 October 2026 is a Thursday; weekday 3): `Thu 8 Oct 2026, 12:00 AM`;
`2026年10月8日 星期四 00:00` (zh, auto ⇒ 24h); `2026年10月8日 星期四 上午 12:00` (zh, 12h).

### 5.4 `clients/shared/host/` — `MineWorldHostClock` and `MineWorldWorldTab` (SD-WC-5, SET-c)

```text
clients/shared/host/
  host_clock.gd   MineWorldHostClock  token, base URL, POST /admin/clock, pending/confirmed, error codes
  world_tab.gd    MineWorldWorldTab   the World tab: state, Pause/Resume, Day length 4 h · 2 h · 1 h
```

- **Token:** read once from `OS.get_environment("MINEWORLD_ADMIN_TOKEN")` at `setup()`; never from argv
  (process lists show argv), never logged, never stored. If empty → **no token**: the tab is read-only and
  **no `HTTPRequest` node is ever created** (step-20 SET-c criterion 2).
- **Base URL:** `http://<the address the client joined>` — the admin surface is on the same listener
  (A-4). The module takes it from the composition root; it does not parse join arguments itself.
- **State shown** comes from the `clock` frame only (via `clock_changed`) and the `welcome` summary: paused
  badge; day length shown as `4 h` / `2 h` / `1 h` for 6 / 12 / 24 and `custom ({n}×)` otherwise (a solo
  world dozing in S21, or a `--time-scale 1` world).
- **Commands:** `{"paused": true|false}` and `{"time_scale": 6|12|24}`, `Authorization: Bearer <token>`,
  `Content-Type: application/json`, one request in flight; a second click while pending is ignored.
- **Pending:** after a 200 the control shows `ui.world.pending` until a `clock` frame whose fields match the
  request arrives, or 5 wall s pass, after which it shows `ui.world.unconfirmed` and returns to the last
  frame's state (INV-WC-1). A repeated value (200, no frame — `PROTOCOL.md` §11.2 "already in force") is
  confirmed by the current state already matching.
- **Errors**, each translated by code (`ui.world.error.<code>`): `unauthorized` (401), `malformed` (400),
  `fast_forward_solo_only` (409), `world_stopped` (503), `timeout` (no answer in 5 s — the 401 path takes
  ≥ 500 ms by design, so the timeout is 5 s, not 1 s), `unreachable` (connection failed). The controls
  return to the server's state.
- **Who sees what:** with a token, enabled controls; without, the same rows read-only plus the badge
  (QSET-4). The tab is added in both clients, always, so a non-host learns why the world stopped.
- **Isolation:** `host/` loads `MineWorldText` for words and nothing else of `settings/`; it never names
  `MineWorldSettingsStore` or `store.gd` (scan, INV-SET-3).

### 5.5 `[launcher] keep_running` (SD-WC-6, TW-e)

`MineWorldSettings.keep_running: bool`, default **false** (step-19 §7.5), file `[launcher] keep_running`,
validated like every other key. Shown as one row in the **General** tab only when the client was started by
its local launcher (`--hosted-here`, a join argument the launcher adds; never saved). It is a presentation-
side launcher preference: it changes what the client does on close (§5.6) and is never sent to the server.

### 5.6 Closing the game (SD-WC-7; depends on Q-WC-1)

Recommended design (Q-WC-1 (a)):

```text
window close / menu Quit, client started --hosted-here:
  keep_running = false  →  client POSTs /admin/stop with the token, waits ≤ 5 s for closing{server_stopping}
                           or the connection's end, then quits with exit code 0
  keep_running = true   →  client disconnects and quits with exit code 10 ("leave the host running")
launcher after Godot exits:
  exit 10               →  leaves the server running, prints its address, invite file and how to stop it
                           (Ctrl-C in its own console / the stop route), and removes its EXIT trap
  otherwise             →  waits ≤ 10 s for the server to exit by itself; if it is still running (client
                           crashed, or an old client) it sends SIGINT on Unix (graceful), then the existing
                           kill as a last resort (loses at most the time since the last checkpoint — S11's
                           guarantee), and says so
```

`POST /admin/stop`: body `{}` or absent; 202 `{ "stopping": true }`, then the same path as Ctrl-C (stop
accepting, `closing { reason: "server_stopping" }` to sessions, checkpoint, `Stopped` journal record,
statistics). 401/404 as every admin route. It changes no world state (`ARC-44`); it is a host command like
pause. This is the only path that is graceful on Windows from a Git Bash launcher (A-5).

Fallback if Q-WC-1 is ruled (b) (no new route): the launcher sends SIGINT on Unix; on Windows the close is
not graceful and CP-TW-e's "resumes the same minute" is recorded as unmet on Windows (an operator checklist
FAIL, not a pass).

### 5.7 The launcher contract (SD-WC-8, TW-e; TW-c §11)

Both launchers, when they host the world themselves:

1. Generate a 32-hex-digit token from `/dev/urandom`; refuse to start if shorter.
2. `export MINEWORLD_ADMIN_TOKEN` for the server and the client processes only (a subshell / `env` prefix,
   never echoed, never in a log line).
3. Start `mineworld server <world> --listen 127.0.0.1:0 --solo --save <save> …`, **no `--time-scale`**
   (the world's `hosting:` gives 12×, or the save's journal; TW-c §5.5).
4. Pass `--hosted-here` to the client.
5. Close per §5.6.

When joining a world hosted elsewhere (`--server … --invite …`), they pass nothing: no token unless the
user exported `MINEWORLD_ADMIN_TOKEN` themselves (a LAN host on their own machine), so the tab is
read-only.

### 5.8 Per platform (the operator's all-platform requirement)

| Concern | macOS | Linux (X11, Wayland) | Windows (Git Bash launchers) |
| --- | --- | --- | --- |
| Monotonic estimate (`Time.get_ticks_usec`) | mach absolute time | `CLOCK_MONOTONIC` | `QueryPerformanceCounter` — all monotonic; no platform branch in `world_time/` (scan) |
| `HTTPRequest` to `127.0.0.1` | yes | yes | yes; Windows Firewall does not prompt for loopback |
| Token from `/dev/urandom` via `od` | yes | yes | Git Bash provides `/dev/urandom` and `od`; checked by H-WC-4 |
| Environment hand-over | `export` | `export` | `export` in Git Bash reaches native `mineworld.exe` and `godot.exe` (MSYS converts the environment) |
| Graceful close | `/admin/stop`; SIGINT fallback | same | `/admin/stop` only (no console event from Git Bash to a background native process, A-5) |
| Exit code 10 from Godot (`get_tree().quit(10)`) | yes | yes | yes; `$?` in Git Bash |
| Symlinked modules | yes | yes | needs `core.symlinks` — the launcher loops refuse with the existing message (A-17) |
| Settings folder | as SET-a §3.4 | as SET-a §3.4 | as SET-a §3.4 |

---

## 6. Design — the clients

### 6.1 Both clients

- The composition root creates one `MineWorldWorldTime`, connects `client.clock_changed` and its
  observation callback to it, and on every `_process` asks `MineWorldClockFormat.date_time(wt,
  Time.get_ticks_usec())` for the HUD line, and `wt.paused()` for the `Paused` label.
- It adds the World tab: `menu.add_tab("ui.settings.tab.world", MineWorldWorldTab.new(menu, host_clock,
  client))`.
- On `NOTIFICATION_WM_CLOSE_REQUEST` (window close) and on the menu's `quit_requested`, it runs §5.6.

### 6.2 2D (`clients/2d`)

- `hud/status.gd` `ui.status.when` takes the clock string from `date_time(...)` and is refreshed every frame
  (today only per observation, A-12); its `Words.day_time` keeps working for the no-calendar case.
- Day and night: one `CanvasModulate` under the world's canvas layer (the HUD is on its own `CanvasLayer`
  and is not tinted), colour = the 2D Presentation Pack's table `presentation/mineworld-default/2D/daylight.json`
  `{ "elevation_deg": [...], "tint": ["#rrggbb", ...] }`, interpolated by `sun_at(display).elevation_deg`.
  No record ⇒ no `CanvasModulate` (today's look, INV-TW-6). The pack table is art content (`ART_DIRECTION.md`),
  validated by `check_client_rules.py --check-pack` (monotonic elevations, same lengths).

### 6.3 3D (`clients/3d-spike`, the slice)

New `scripts/slice/slice_sun.gd` (`SliceSun`), owned by `slice_main.gd`, which keeps its constants as the
golden-hour anchor and gains calls only:

- **Direction.** `rotation_degrees = Vector3(−elevation, azimuth_deg + SLICE_NORTH_YAW_OFFSET, 0)` with the
  sign convention of today's rig. `SLICE_NORTH_YAW_OFFSET` is presentation content: where the slice's map
  north points. It is chosen once so that San Diego's **afternoon** sun at elevation 19.3° on the world's
  epoch day lands on ARC-13's yaw −48° (C5 reads that day's record from a hosted `social-cafe`, records the
  azimuth and the offset, and a check asserts the golden-hour reproduction within 0.1°). Q-WC-7.
- **Energy, physically scaled (realistic defaults).** Above the horizon, direct-beam irradiance by elevation
  from the Kasten–Young air mass `AM = 1 / (sin h + 0.50572 (h + 6.07995°)^−1.6364)` (F. Kasten and A. T.
  Young, *Applied Optics* 28(22):4735–4738, 1989) and Meinel's clear-sky beam model `I = 1.353 kW m⁻² ×
  0.7^(AM^0.678)` (A. B. Meinel and M. P. Meinel, *Applied Solar Energy*, Addison-Wesley, 1976), normalised
  so that `energy(19.3°) = SUN_ENERGY = 4.4` exactly. This gives about 1.38 × 4.4 ≈ 6.1 at San Diego's
  October noon (elevation ≈ 51°) and fades to 0 at the sunrise horizon (−0.833°, the pack's standard
  refraction). Godot's `DirectionalLight3D.light_energy` is normal-incidence irradiance (the surface
  cosine is the renderer's), so the ratio applies directly.
- **Colour.** `SUN_WARM` at ≤ 19.3°, blending to neutral `Color(1, 0.96, 0.90)` by 40° (art content; direct
  sunlight's colour shift with air mass is the physical cause, the exact curve is ART_DIRECTION's).
- **Below the horizon** the key light is off; `SkyFill` energy, `SKY_AMBIENT` and the panorama's
  `energy_multiplier` follow a phase table (day 1.0, civil twilight 0.35, astronomical twilight 0.08,
  night 0.03 of today's values), so night is dark but legible at the fixed exposure. These are
  presentation values, not physical illuminance (a real 10⁵:10⁻³ lux range is not displayable at a fixed
  exposure); `WORLD_TIME.md` says so.
- **Sky.** The far field's declared panorama is golden-hour; TW-e only dims it by phase (Q-WC-5). Its baked
  sun disc stays where it is — recorded as a known mismatch for TW-f (Sky3D, driven externally).
- **Far field.** `far_field_view.gd` `_haze()` takes the current `light` (sun term + sky term) from
  `SliceSun` instead of the constants. If the land colours are baked at build time, the change is applied as
  one material colour multiplier updated when the light changes by > 2 %, not a rebuild (bounded detail,
  decided in C5 from the code and recorded).
- **Fixed mode (INV-WC-2).** `SliceSun` holds the constants whenever (a) no `calendar-day` has been
  disclosed in this session, (b) the client runs any scripted or capture mode (`_scripted()`), or (c)
  `--sun=fixed` is given. So `--shots`, `--perf`, `--farfield`, every `--world` probe and the far-field check
  render exactly what they render on `main`; `slice_probe.gd:729` stays green unedited.
- **GI.** `VOXEL` stays the default. Step-19 R-TW-6 says VoxelGI "does not follow a moving sun"; Godot 4's
  VoxelGI bakes geometry, not light, and is documented to support dynamic lights, so the claim is
  **unverified** (F-WC-4). TW-e records two captures (noon, golden hour) of the café interior with VOXEL
  for TW-f's GI measurement; it does not change the GI mode.
- HUD: `_hud_time` from `date_time(...)` plus a `Paused` label.

---

## 7. Test ownership

| Failure class | Owner | Runs |
| --- | --- | --- |
| Literal UI strings in `world_time/`, `host/`, 3D files; catalog completeness for the new keys; network identifiers in `world_time/` and `settings/`; `host/` loads no store; no platform branch in `world_time/` and `host/` | `tests/acceptance/tests/client_text.rs` (extended) | `cargo test`, CI, all three OSes |
| `/admin/stop` route: auth, 202, graceful path, `Stopped` record, 404 without token | `server/tests/admin.rs`, `tools/cli/tests/host_clock.rs` (extended) | `cargo test`, CI |
| `social-cafe` calendar opt-in (if ruled): goldens, INV-TW-1 elsewhere | existing world tests + `scripts/ci_parity.py diff` | CI + recorded once |
| Estimate, display rule, roll-over, sun interpolation, phase, formatting, both languages, both forms | `clients/shared/checks/world_time_check.gd` over a recorded stream fixture `clients/shared/checks/fixtures/clock_stream.json` | Godot headless at the gate (macOS); verdict platform-independent (pure integers + catalogs) |
| `host/` pending, confirm, errors, no-token-no-request | `clients/shared/checks/host_check.gd` against a scripted `HTTPRequest` seam and a stub server | Godot headless at the gate |
| Both clients end to end (CP-WC, AC-WC-*) | `tools/cli/tests/client_world_time.rs` (`#[ignore]`, Godot) against a real `mineworld server … --solo --admin-token` and the `StubWorld` | at the gate, macOS; Windows and Linux are operator checklist items (H-WC-*) |
| Real LLM | **NOT REQUIRED** — no model is involved | — |
| Rendered look (night legibility, golden hour equals ARC-13, tint) | hand checks H-WC-1 … H-WC-6 | operator |

---

## 8. Acceptance and adversarial criteria (fixed now, before anything is measured; `ARC-23`)

**CP-WC — the integration checkpoint.** From each launcher (`./mineworld-2d` on `market-town`;
`./mineworld-slice --world` on `social-cafe` once it has `calendar`, Q-WC-3), on a fresh save:
the HUD shows `Thu 8 Oct 2026, 12:00 AM` and advances 12 ± 1 world minutes per wall minute at the default
scale; World → Pause → `Paused` appears only after the `clock` frame and `/status` `at` is frozen for 10 wall
s; Resume; Day length 1 h → the HUD advances 24 ± 2 world minutes per wall minute and `/status` `at` by 240 ±
24 world s in 10 wall s; zh_Hans with clock 12h shows `2026年10月8日 星期四 上午 …`; close the window → the
server process exits with its `Stopped` record; relaunch → the HUD resumes at or after the last minute shown,
within one minute. In 3D the key light's elevation at world 12:00 equals the record's interpolated
elevation within 0.1°, and at the afternoon 19.3° instant the rig equals ARC-13's constants within 0.1°.
Run on macOS at the gate; Windows and Linux by the operator (H-WC-4).

| Id | Criterion | Oracle | Mutation that must fail it |
| --- | --- | --- | --- |
| **AC-WC-1** | **Parity.** Both clients, headless, fed the same recorded stream (welcome, clock frames incl. a pause and a rescale, observations with and without `calendar-day`, a day boundary) with the same scripted `now_usec` schedule, print identical `EVIDENCE hud_clock` lines and identical light phase at the same steps, in `en`/`zh_Hans` × 12h/24h. | both clients' evidence lines, diffed | A 3D-only formatter (bypass `clock_format.gd` in `slice_main.gd`). |
| **AC-WC-2** | **Never backwards.** In the stream, a clock frame arrives with `at` 120 s behind the running estimate: the displayed minute holds, then advances once the world passes it; at no step is a displayed value smaller than the previous one. A new `welcome` earlier than the last display is shown at once (the documented exception). | `world_time_check.gd` | Set `display = at_est` on re-anchor. |
| **AC-WC-3** | **No calendar ⇒ today's behaviour (INV-TW-6).** 2D on `social-cafe` (no calendar, until Q-WC-3) and 3D on `bodies-yard`-style stream without records: HUD is `hud.day_time` exactly as on `main`; no `CanvasModulate` exists; the 3D `Sun` node's rotation, energy and colour equal the constants bit for bit. | evidence + node walk | Default the date to the epoch when no record exists. |
| **AC-WC-4** | **The date comes from the record, never the OS clock.** The fixture's epoch is 2031-03-14 (≠ today on any machine): the HUD shows `Fri 14 Mar 2031, …`. A static scan finds no `Time.get_date*`, `get_datetime*`, `get_unix_time_from_system` or `OS.get_*time*` under `world_time/` and `clock_format.gd`. | check + scan | Compute the date with `Time.get_date_dict_from_system()`. |
| **AC-WC-5** | **Roll-over.** Estimate passes 24:00 with no new record: the date advances by `MineWorldCivil.next_date` — 2026-10-08 → 09, 2026-12-31 → 2027-01-01, 2028-02-28 → 29, 2027-02-28 → 03-01 — and the next record replaces it without a visible jump. | check | Drop the leap rule. |
| **AC-WC-6** | **Sun interpolation.** For three instants (a sample, the midpoint of two samples, a midpoint across azimuth 359.9°→0.2°) `sun_at` equals the linear interpolation of the disclosed millidegrees within 0.001°, azimuth unwrapped (the cross-north midpoint is ≈ 0.05°, not ≈ 180°). | check | Interpolate azimuth without unwrapping. |
| **AC-WC-7** | **No optimism (INV-WC-1).** Against the stub that answers 200 to `POST /admin/clock` and sends no `clock` frame: after Pause, the HUD never shows `Paused`; the tab shows pending, then `unconfirmed` after 5 s, then the frame's state. | `client_world_time.rs` stub run | Set `paused` on click / on the 200. |
| **AC-WC-8** | **Rate follows the server.** Real server, 2D: over 20 wall s at 12× and then 20 wall s after Day length 1 h, the displayed clock advances 240 ± 24 and 480 ± 48 world s, and each agrees with `/status` `at` sampled at the same moments within 2 world s × scale. Tolerances are over ≥ 20 s windows because of F-TWc-4. | test samples | Ignore `time_scale` from the `clock` frame (keep the welcome's). |
| **AC-WC-9** | **Settings never reach the server (INV-SET-1).** The 2D stub record of a session with `zh_Hans`, 24h and `keep_running = true` equals the default session's frames (after replacing request tokens); the 3D request transcript likewise. | stub record; transcript | Put the clock form in a `join` field. |
| **AC-WC-10** | **The token never leaks (INV-SET-2, -3).** After a session that paused, rescaled and closed, `grep -r` of the token over the settings file, both clients' logs, the server log, `shots/`, evidence directories and the launcher's output finds nothing; the settings file is byte-identical to before except `[launcher]` if the user changed it. | grep | Print the token in a `[link]` line; read it from argv. |
| **AC-WC-11** | **No token ⇒ no request.** A second 2D client joined by invite without the variable: the World tab is read-only, shows the paused badge after the host pauses, and its scene tree contains no `HTTPRequest` node at any time. | node walk | Create the `HTTPRequest` in `_ready` unconditionally. |
| **AC-WC-12** | **Errors are worded and revert.** Stub answers 401 (after 500 ms), 400, 409, 503, and no answer: each shows `ui.world.error.<code>` in the current language and the controls return to the last frame's state. | stub run | Show the raw body. |
| **AC-WC-13** | **Graceful close resumes the minute** (needs Q-WC-1 (a)). Real server via the 2D launcher path: after 3 wall min at 12×, close: the server log ends with the checkpoint line, the save's host journal ends with `Stopped`; relaunch: the first HUD value is ≥ the last shown and ≤ last shown + 60 s. With `keep_running = true`: the server is still running after the client exits, and a new client reconnects to it. | `host_clock.rs`-style test + save inspection | Launcher keeps `kill` (SIGTERM) as the first action. |
| **AC-WC-14** | **Harnesses keep golden hour (INV-WC-2).** With a calendar-enabled world, `--world --link`, `--shots` and `--farfield` render with the `Sun` node exactly at the constants, and `--perf` JSON shows the same light list as `main`. | node read in each probe | Let `SliceSun` move the light in scripted modes. |
| **AC-WC-15** | **One module, two clients (INV-SET-8, INV-WC-3).** Both clients have `mineworld_world_time` and `mineworld_host` as symlinks into `clients/shared/`; `client_text.rs` finds no `HTTPRequest` outside `host/` and `clients/protocol/checks/`, no `store.gd` reference in `host/`, and no network identifier in `world_time/`. | scan | Copy `world_time.gd` into `clients/2d/scripts/`. |
| **AC-WC-16** | **INV-TW-1 / ARC-25.** `ci_parity.py diff` base vs head: no world differs, except `social-cafe` if and only if Q-WC-3 opts it in, and then only in that content commit. | parity tool | — |

---

## 9. Hand checks (operator; each records the platform line `display.gd` prints)

| Id | What | Pass | macOS | Windows | Linux |
| --- | --- | --- | --- | --- | --- |
| H-WC-1 | 3D through one world day at 24× (1 h): dawn, noon, golden hour, dusk, night | golden hour looks as ARC-13's accepted capture; noon is not blown out; night is dark but the café and street are legible; the far field never glows under a dark sky | gate | checklist | checklist |
| H-WC-2 | 2D tint over the same day | night readable; HUD untinted | gate | checklist | checklist |
| H-WC-3 | World tab in both languages, host and non-host | layout, badge, pending, errors | gate | checklist (150 %) | checklist |
| H-WC-4 | CP-WC end to end from both launchers | as §8 | gate | checklist (Git Bash; token from `/dev/urandom`; close via `/admin/stop`) | checklist (X11 and Wayland) |
| H-WC-5 | Keep running on, close, reopen | server stayed up; reconnect | gate | checklist | checklist |
| H-WC-6 | VoxelGI under noon vs golden hour (F-WC-4) | two captures recorded for TW-f | gate | — | — |

---

## 10. Commit plan

Every commit lists implementation, validation and review as separate items. `[ ]` until done with
evidence; `N/A` with its reason.

### C0 — Freeze and contract (Markdown only)

1. **Goal:** record the freeze header, the base commit, the rulings of §14.
2. **Scope:** this file only.
3. **Implementation:** [ ] header to `DESIGN FROZEN` with the operator's approval reference; [ ] base recorded.
4. **Validation:** [ ] `python3 scripts/check_doc_headings.py` and `check_decision_ids.py` green.
5. **Acceptance:** the header names an approval the operator gave; base is `origin/main` at C0.
6. **Edge cases:** a ruling that changes scope sends the design back (no C1).
7. **Commands/evidence:** recorded in §15.
8. **Boundary:** Markdown only. [ ] Review: the header matches §14's rulings.

### C1 — Specifications before code

1. **Goal:** the rules exist before the code (`CLAUDE.md` §2.2).
2. **Scope:** new `clients/shared/WORLD_TIME.md`, `clients/shared/HOST.md`; `clients/shared/SETTINGS.md` §1
   (`[launcher]` row filled), §2 (two new modules in the layout), §6.6 (date entries), §7 (World tab,
   `keep_running` row); `clients/shared/README.md` one line each; `clients/protocol/ADOPTION.md` §1 one
   sentence; `server/PROTOCOL.md` lines 272 and 832 (A-3) and, if Q-WC-1 (a), §11.2 `/admin/stop` and its
   landing-table row; `docs/DECISIONS.md` dated notes (§11); step-19 §11.2 TW-e row amended (§0).
   Non-goals: no code.
3. **Implementation:** [ ] each file above.
4. **Validation:** [ ] doc heading and decision-id checks; [ ] `cargo test -p mineworld-acceptance --test
   client_text` still green (catalogs untouched yet).
5. **Acceptance:** every rule §§5–6 states is in a spec; no README carries a rule.
6. **Edge cases:** a PROTOCOL change beyond A-3 and `/admin/stop` is a material stop.
7. **Commands/evidence:** §15.
8. **Boundary:** docs only. [ ] Review: terminology (`System Pack`, `Observation`, "host command", "setting")
   matches CORE_CONCEPTS and SETTINGS.md; no synonym introduced.

### C2 — `server`: `POST /admin/stop` (only if Q-WC-1 (a); else `N/A` with the ruling)

1. **Goal:** a graceful stop reachable from a client on every platform.
2. **Scope:** `server/src/admin.rs` (route), `server/src/runtime/control.rs` or the app's shutdown signal
   (the same future `stop_requested` completes), `tools/cli/src/serve.rs` (select on it beside
   `stop_requested()`); tests in `server/tests/admin.rs`, `tools/cli/tests/host_clock.rs`. Non-goals: no
   change to pause/scale, no new world command, no fact.
3. **Implementation:** [ ] route behind the bearer check, 202; [ ] wired to the existing graceful path; [ ]
   404 without token, 401 wrong token (≥ 500 ms).
4. **Validation:** [ ] unit: auth matrix; [ ] binary: `--solo --admin-token T`, POST stop → process exits 0,
   save's journal ends `Stopped`, restart resumes at `Stopped.at`; [ ] mutation M-WC-1: route returns 202
   without triggering shutdown → binary test red.
5. **Acceptance:** AC-WC-13's server half.
6. **Edge cases:** stop while paused (journal `Paused` then `Stopped`; restart runs, `PROTOCOL.md` §11.3);
   two stops (second is 202, idempotent); stop during a checkpoint.
7. **Commands:** `cargo test -p mineworld-server --test admin`; `cargo test -p mineworld-cli --test
   host_clock`.
8. **Boundary:** server only. [ ] Review: `ARC-44` — no world state changed; revision and fact count
   unchanged by the call.

### C2b — `worlds: social-cafe has San Diego's calendar` (only if Q-WC-3 yes)

1. **Goal:** a 3D world with a day record.
2. **Scope:** `worlds/social-cafe/world.yaml` (append `calendar` and its `configure:` entry),
   `worlds/social-cafe/configure/calendar.yaml` (same content as `market-town`'s), regenerated goldens the
   world's tests pin. Non-goals: no `weather` (TW-f), no other world.
3. **Implementation:** [ ] opt-in; [ ] goldens.
4. **Validation:** [ ] `ci_parity.py diff`: only `social-cafe` differs; [ ] P7 (`test_milestone_d_100_days`)
   and social tests green in CI; [ ] `run social-cafe --days 2` shows two `day-began`.
5. **Acceptance:** AC-WC-16.
6. **Edge cases:** a test that pins social-cafe's fact count → updated with the reason "calendar facts".
7. **Commands:** `target/debug/mineworld run worlds/social-cafe --days 2 --headless`; `python3
   scripts/ci_parity.py record|diff`.
8. **Boundary:** content only. [ ] Review: no pack constant, no code.

### C3 — `clients/shared/world_time/` and the date formatter (TW-e)

1. **Goal:** one estimate, one date, one sun for both clients.
2. **Scope:** `clients/shared/world_time/{world_time.gd,civil.gd}`; `clients/shared/settings/clock_format.gd`
   (`date`, `date_time`); `settings/locale/{en,zh_Hans}.po`, `messages.pot`; `clients/shared/checks/
   world_time_check.gd` + `fixtures/clock_stream.json`; `client_text.rs` scopes (A-18) and the OS-clock
   scan (AC-WC-4); `glyph_check.gd` covers the new strings automatically. Non-goals: no client wiring.
3. **Implementation:** [ ] module per §5.1–5.2; [ ] civil; [ ] formatter per §5.3; [ ] catalogs; [ ] scans.
4. **Validation:** [ ] `world_time_check.gd`: AC-WC-2, -4, -5, -6 and the format table of §5.3 in both
   languages and forms; [ ] `client_text.rs` green; [ ] glyph check green; [ ] mutations M-WC-2 (no unwrap),
   M-WC-3 (`display = at_est`), M-WC-4 (OS date) each red by name.
5. **Acceptance:** the listed criteria pass from the check alone.
6. **Edge cases:** record with `null` events (polar); scale 1; paused at welcome; frame order clock-before-
   observation and after; `at` beyond the record's 24:00 sample (`valid = false`).
7. **Commands:** `godot --headless --path clients/shared --script res://checks/world_time_check.gd`;
   `godot … glyph_check.gd`; `cargo test -p mineworld-acceptance --test client_text`.
8. **Boundary:** shared modules only. [ ] Review: no clock read inside the module; integers until the last
   step; no client name in the module.

### C4 — 2D client: HUD date and time, Paused, tint (TW-e)

1. **Goal:** the 2D half of CP-WC except the World tab.
2. **Scope:** `clients/2d/mineworld_world_time` (symlink), `scripts/app.gd` (create and feed
   `MineWorldWorldTime`), `scripts/hud/status.gd` (per-frame clock, `Paused` label), a new
   `scripts/scene/daylight.gd` (`CanvasModulate` from the pack table),
   `presentation/mineworld-default/2D/daylight.json` and its `i18n` untouched; `mineworld-2d` module loop;
   `scripts/check_client_rules.py --check-pack` validates the table. Non-goals: no World tab yet.
3. **Implementation:** [ ] wiring; [ ] HUD; [ ] tint; [ ] launcher loop.
4. **Validation:** [ ] 2D half of AC-WC-1 and AC-WC-3 in `client_world_time.rs`; [ ] `check_client_rules.py`
   R6 green; [ ] existing `client_2d*.rs` and `client_settings.rs` green (`--ignored`, gate).
5. **Acceptance:** 2D prints the evidence lines AC-WC-1 compares.
6. **Edge cases:** observer in no place (keeps the record); `--presentation=none` (no tint table ⇒ no tint).
7. **Commands:** `cargo test -p mineworld-cli --test client_world_time -- --ignored --test-threads=1`.
8. **Boundary:** 2D only. [ ] Review: `app.gd` stays under 500 lines; no rule in the renderer.

### C5 — 3D client: the sun, sky and far-field light, HUD (TW-e)

1. **Goal:** the 3D half of CP-WC except the World tab.
2. **Scope:** `clients/3d-spike/mineworld_world_time` (symlink); new `scripts/slice/slice_sun.gd`;
   `slice_main.gd` (calls only; constants kept as the anchor); `far_field_view.gd` (`_haze` light from
   `SliceSun`); `slice_link.gd` (forward `clock_changed`/observations — or `slice_main.gd` subscribes
   directly; bounded); `mineworld-slice` module loop and `--sun=fixed`. Non-goals: no Sky3D, no GI change.
3. **Implementation:** [ ] `SliceSun` per §6.3; [ ] north offset measured and recorded; [ ] fixed mode; [ ]
   far field; [ ] HUD.
4. **Validation:** [ ] 3D half of AC-WC-1, AC-WC-3, AC-WC-14; [ ] the energy function: `energy(19.3) ==
   4.4` exactly, monotonic above the horizon, 0 below −0.833°; [ ] CP-WC's 0.1° rig checks; [ ] existing
   3D probes (`--world --link`, `--farfield`, `client_far_field.rs`, `client_settings.rs`) green; [ ] mutation
   M-WC-5 (move the light in scripted modes) → AC-WC-14 red.
5. **Acceptance:** as listed; H-WC-1, H-WC-6 captures recorded.
6. **Edge cases:** no record then a record mid-session (fade over 1 s, not a jump); polar record; the
   panorama missing (procedural fallback is dimmed too).
7. **Commands:** `./mineworld-slice --world --link`; `cargo test -p mineworld-cli --test client_world_time
   -- --ignored`.
8. **Boundary:** 3D only. [ ] Review: `slice_main.gd` does not grow past 600; no astronomy in GDScript
   (grep for `sin(` over coordinates; only the elevation→energy function evaluates trig, on the
   disclosed elevation).

### C6 — `clients/shared/host/` and the World tab in both clients (SET-c)

1. **Goal:** the host pauses, resumes and sets the day length from the menu; everyone sees the state.
2. **Scope:** `clients/shared/host/{host_clock.gd,world_tab.gd}`; `clients/shared/checks/host_check.gd`;
   catalog keys `ui.settings.tab.world`, `ui.world.*` (both languages); symlinks `mineworld_host` in both
   clients; one `add_tab` call in each composition root; `client_text.rs` (HTTPRequest confined to `host/`,
   no store in `host/`); `godot2d::StubWorld` scripted clock frames and a minimal HTTP responder. Non-goals:
   no fast-forward option; no change to `menu.gd`'s tabs.
3. **Implementation:** [ ] module per §5.4; [ ] tab; [ ] wiring; [ ] stub; [ ] scans.
4. **Validation:** [ ] `host_check.gd`; [ ] AC-WC-7, -11, -12 in `client_world_time.rs`; [ ] mutation M-WC-6
   (optimistic pause) → AC-WC-7 red; M-WC-7 (HTTPRequest unconditional) → AC-WC-11 red.
5. **Acceptance:** SET-c's step-20 criteria (1)–(4), restated as AC-WC-7, -10, -11, -12.
6. **Edge cases:** a `clock` frame arriving between click and 200; the world stopping (503); a non-6/12/24
   scale shown as custom; language switch while pending.
7. **Commands:** `godot --headless --path clients/shared --script res://checks/host_check.gd`; the test.
8. **Boundary:** SET-c's module and two `add_tab` lines. [ ] Review: INV-SET-3 by reading every load in
   `host/`.

### C7 — Launchers, `[launcher] keep_running`, graceful close (TW-e)

1. **Goal:** the single-player launch and close of §§5.5–5.7.
2. **Scope:** `mineworld-2d`, `mineworld-slice` (token, `--solo`, env hand-over, `--hosted-here`, exit-code
   10 handling, wait-then-SIGINT-then-kill, 3D `--world` save under `clients/3d-spike/.save/social-cafe`
   with `--fresh`, Q-WC-4); `clients/shared/settings/{settings,store}.gd` (`keep_running`);
   `menu.gd` General tab row (one control, behind a capability `CAP_HOSTED_HERE`); both composition roots'
   close handling; `store_check.gd` (`[launcher]` round trip, bad values). Non-goals: no packaging (S13/S23).
3. **Implementation:** [ ] each item.
4. **Validation:** [ ] AC-WC-9, -10, -13; [ ] `store_check.gd`; [ ] `menu_check.gd` (row hidden without the
   capability); [ ] mutation M-WC-8 (launcher kills first) → AC-WC-13 red.
5. **Acceptance:** CP-WC on macOS.
6. **Edge cases:** server fails to start (`--solo` refused on a non-loopback address — cannot happen with
   `127.0.0.1:0`, asserted); Godot crashes (launcher fallback path, message); `--server` join (no token, no
   `--hosted-here`); token generation fails (refuse to start, message).
7. **Commands:** the test; manual `./mineworld-2d` and `./mineworld-slice --world` runs, logs recorded.
8. **Boundary:** launchers, store field, one row. [ ] Review: the token appears in no `echo`, no log file, no
   `set -x` path; `trap` handling correct for exit 10.

### C8 — Cross-client gate and evidence

1. **Goal:** CP-WC and every AC on the final executable head.
2. **Scope:** `tools/cli/tests/client_world_time.rs` complete; evidence under `clients/shared/evidence/`
   (HUD stills both languages, World tab host and non-host, 3D noon/golden hour/night). No product code.
3. **Implementation:** [ ] evidence capture.
4. **Validation:** [ ] all `#[ignore]` Godot tests of this PR plus `client_settings.rs`,
   `client_far_field.rs`, `client_2d*.rs` once at the gate (macOS); [ ] CI on the exact head (fast, test,
   test-macos, test-windows, platforms, python).
5. **Acceptance:** §8 all PASS, or a recorded FAIL that stops (§13).
6. **Edge cases:** a flake re-run once and recorded; never averaged.
7. **Commands:** `cargo test -p mineworld-cli --test client_world_time -- --ignored --test-threads=1`.
8. **Boundary:** tests and evidence. [ ] Review: each AC's oracle actually ran.

### C9 — Close-out

[ ] Ledger, deviations, the H-WC checklist for the operator, sizes, parent-sync request (§12). [ ] Review.

---

## 11. Decision records (text for the primary session to apply; no new number proposed unless ruled)

- **Note under `ARC-76`** (the settings module), dated, TW-e + SET-c: "`clients/shared/world_time/` and
  `clients/shared/host/` are shared client modules beside `settings/`. `host/` is the only shared client code
  that sends HTTP; it never loads the store; the admin token reaches a client only through
  `MINEWORLD_ADMIN_TOKEN` and lives in memory. `[launcher] keep_running` is the one launcher preference in the
  settings file."
- **Note under `ARC-69`**, dated: "The reference launchers start a hosted single-player world `--solo` with a
  generated token and no `--time-scale`; the World tab sends 6, 12 or 24 only; state is shown from the `clock`
  frame only."
- **If Q-WC-1 (a): a note under `ARC-44`**: "`POST /admin/stop` is a host command: graceful shutdown, no world
  state."
- **Note under `DEP-30`**: the client-side date successor is built, not adopted (§4's reason).

---

## 12. Edits to planning documents

Applied now, by this planning session (pointer notes only; they change no frozen text):

- `step-19-time-weather.md` §11.2: a dated note under the table that TW-e is designed with SET-c in this file,
  that the World section's owner is SET-c (QSET-6, SD-WC-1), and that the TW-e row's World-section clause is
  superseded at this design's freeze.
- `step-20-client-settings.md` §6: a dated note that SET-c is designed in this file.

At freeze (C1): step-19 §11.2's TW-e row text amended as step-20 §11.3 proposed. Post-merge: the primary
session owns `overall.md` S19/S20 rows and the step documents' lifecycle lines.

---

## 13. Execution contract (filled at freeze, 2026-10-18)

```text
PROJECT / PR:            MineWorld mvp0, S19 TW-e + S20 SET-c — the world clock in the clients
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-tw-e-set-c-world-clock-clients.md
RELATED / BINDING DOCS:  step-19 (§§1.3, 7.5, 8, 10, 11.2, 14.1); step-20 (§§1.5, 3.2, 3.10, 5, 6, 8);
                         pr-tw-c-host-clock.md §§5, 11, 15; pr-set-b-controls.md; clients/shared/SETTINGS.md;
                         server/PROTOCOL.md §§5.9, 11; docs/DECISIONS.md ARC-44, ARC-67, ARC-69, ARC-70,
                         ARC-76, DEP-30, DEP-35; docs/ENGINEERING_STANDARDS.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-tw-e-set-c, its own, one session
BRANCH:                  mvp0/pr-tw-e-set-c, from origin/main
START CONDITION:         design frozen by the operator; no open PR editing app.gd, hud/status.gd,
                         slice_main.gd, slice_link.gd, far_field_view.gd, mineworld-2d, mineworld-slice
                         (S33 lakeside and S32 wardrobe lanes checked at C0)
IMPLEMENTATION BASE:     origin/main at C0
APPROVED SCOPE:          §3.1 as ruled (C2 and C2b conditional on Q-WC-1, Q-WC-3)
FROZEN INVARIANTS:       §3.3; SD-WC-1 … SD-WC-8
SEQUENCE:                C0 … C9
ALLOWED COMMANDS:        cargo *; git; gh (never merge); python3 scripts/*; target/*/mineworld *;
                         ./mineworld-2d, ./mineworld-slice; godot --headless (checks); mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; editing kernel/, contracts/, systems/,
                         cognition/, clients/protocol/mineworld/; printing the admin token
MATERIAL STOPS:          any AC FAIL after a fix attempt; a threshold change after a run; a PROTOCOL change
                         beyond A-3 and /admin/stop; a new dependency; a SAVE_FORMAT change; any world
                         other than social-cafe changing under ci_parity; the golden-hour rig not
                         reproducible within 0.1°
PLATFORMS:               macOS (gate), Linux, Windows (CI + operator checklist §9)
VALIDATION BUDGET:       local Godot gate runs once per AC set (R-LOAD-1: no local full-workspace cargo
                         test while other agents build; CI on the exact head is the workspace gate)
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-tw-e-set-c.md
ENDPOINT AUTHORITY:      implementation, local validation, semantic commits, branch push, PR
                         creation/update, CI repair: authorized; merge: never by the implementation
                         session — the PR merges only through a merge train assembled by the coordinator
                         (operator CI policy, 2026-10-18), under the operator's merge grant of 2026-10-16
DECISION IDS:            ARC-266 … ARC-272 (allocated by the primary session)
POST-MERGE SYNC OWNER:   primary session: overall.md, step-19 §11, step-20 §6; implementation session:
                         this document's ledger
STOP CONDITION:          TRAIN-READY (head SHA, local validation, review verdict, mutation evidence
                         reported to the coordinator) — DO NOT MERGE
```

---

## 14. Questions (Q-WC-n), with recommendations. **[OPERATOR]** marks operator-material ones.

Ruled by the operator on 2026-10-18 in the primary session: every question below is answered with its
recommendation; Q-WC-1 is answered (a), add `POST /admin/stop`.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-WC-1 [OPERATOR]** | **Graceful close.** Today's launchers `kill` (SIGTERM) their server, which is not graceful, and Windows has no signal path from Git Bash (A-5). (a) add `POST /admin/stop` (host command, admin token, same graceful path as Ctrl-C) and have the client call it on close; (b) launchers send SIGINT on Unix only, Windows close stays non-graceful; (c) the server handles SIGTERM too, and Windows stays non-graceful. | **(a)**: one small additive route, works on all three OSes, keeps "closing saves and pauses" true everywhere; (b) as the launcher's fallback after a client crash. |
| **Q-WC-2** | Ownership: the World tab, token handling and `host/` belong to SET-c; world time, HUD, sun, launcher close to TW-e; delivered as one PR with separable commits (§0). | **Yes**, as SD-WC-1; the step-19 TW-e row is amended at freeze. |
| **Q-WC-3 [OPERATOR]** | **3D needs a day record.** `social-cafe` has no `calendar` (A-8), so the 3D sun cannot move. Opt `social-cafe` into `calendar` (San Diego, same epoch) in its own content commit, changing that world's goldens (and P7's fact stream)? | **Yes**, without `weather` (TW-f). Alternative: TW-e ships 3D on fixed golden hour and the sun moves only when a 3D world opts in later — CP-WC would then be 2D-only for the sun. |
| **Q-WC-4** | `./mineworld-slice --world` keeps no save (A-7). Give it a persistent save `clients/3d-spike/.save/social-cafe` (with `--fresh`), as `./mineworld-2d` has, so 3D "reopen resumes the minute" can hold? Scripted probes keep scratch saves. | **Yes.** |
| **Q-WC-5** | **3D sky in TW-e.** The declared sky is a golden-hour HDRI. (a) dim it by phase only (cheap; its baked sun disc is wrong away from golden hour until TW-f); (b) switch to `PhysicalSkyMaterial` whenever a record is disclosed (follows the sun, but changes RL-e0's declared far-field sky). | **(a)**; Sky3D driven externally in TW-f fixes the disc. |
| **Q-WC-6** | Step-19's TW-e row puts "condition → intent" (weather) in `world_time/`. Move it to TW-f, its first consumer? | **Yes** (`CLAUDE.md` §4 rule 11, no premature abstraction). |
| **Q-WC-7** | Golden-hour mapping. ARC-13's sun (yaw −48°, elevation 19.3°) is an art choice, not San Diego's geometry. Choose the slice's map-north offset so the **afternoon** 19.3° sun lands exactly on ARC-13's rig (the approved look returns every afternoon), rather than keep an arbitrary north and lose that look? | **Yes**; the offset is presentation content, recorded with the measured azimuth. |
| **Q-WC-8** | Sun energy above the horizon follows Kasten–Young air mass with Meinel's clear-sky beam model, normalised to 4.4 at 19.3°; below the horizon, presentation tables (a real illuminance range cannot be shown at fixed exposure). Accept both sources as the realistic default? | **Yes.** |
| **Q-WC-9** | The San Diego coordinates (32.7157, −117.1611) have no recorded primary source (A-10), though TW-a checked the sun at exactly those values against NOAA's calculator. Record a primary source (USGS GNIS, "San Diego" populated place) in `market-town`'s `calendar.yaml` comment? Geocoders give ≈ 32.7153, −117.1573 — ≈ 0.004° of longitude, under 1 s of solar time, so no value changes. | **Yes, comment only, values unchanged** — but only if a comment edit does not change the configured fact (IL-a's drift check reads values, to be confirmed at C1); otherwise record it in `WORLD_TIME.md` and leave the file. |
| **Q-WC-10** | Who sees the "keep running" row: only a client started by its local launcher (`--hosted-here`)? | **Yes**; a joining client has no host to keep. |
| **Q-WC-11** | Pending timeout and HTTP timeout 5 s (the 401 path is ≥ 500 ms by design). | **Yes.** |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | Hash | Status |
| --- | --- | --- |
| C0 … C9 | — | not started (design DRAFT) |

### Evidence

- Planning audit read on `origin/main @ b85f2804` (2026-10-18): files and lines in §2.

### Mutations

| Id | Mutation | Expected red |
| --- | --- | --- |
| M-WC-1 | `/admin/stop` answers 202 and does not stop | C2 binary test |
| M-WC-2 | azimuth interpolated without unwrapping | AC-WC-6 |
| M-WC-3 | `display = at_est` on re-anchor | AC-WC-2 |
| M-WC-4 | date from `Time.get_date_dict_from_system()` | AC-WC-4 (check and scan) |
| M-WC-5 | `SliceSun` moves the light in scripted modes | AC-WC-14 |
| M-WC-6 | `paused` shown on click / on 200 | AC-WC-7 |
| M-WC-7 | `HTTPRequest` created without a token | AC-WC-11 |
| M-WC-8 | launcher `kill` before `/admin/stop` | AC-WC-13 |

### Findings recorded at planning

- **F-WC-1** — step-19 and step-20 both claimed the World section; QSET-6's amendment to step-19 was never
  applied. Resolved by SD-WC-1 (§0).
- **F-WC-2** — `server/PROTOCOL.md` lines 272 and 832 are stale after TW-c (A-3). Fixed in C1.
- **F-WC-3** — the San Diego coordinates' primary source is not recorded (A-10). Q-WC-9.
- **F-WC-4** — step-19 R-TW-6's "VoxelGI does not follow a moving sun" is unverified and probably wrong for
  Godot 4 (VoxelGI bakes geometry; lights are dynamic). Captures in C5 for TW-f.
- **F-WC-5** — today's launchers stop their server non-gracefully (SIGTERM; A-5), so CP-TW-e's "reopening
  resumes the same minute" cannot hold on `main`. Q-WC-1.

### Deviations

None yet.

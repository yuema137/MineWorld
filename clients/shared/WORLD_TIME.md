# The MineWorld world-time client module

**Audience:** whoever adopts, extends or reviews the shared world-time module — the 2D reference client,
the 3D reference client, a third client, or a later PR (TW-f weather visuals, S21 fast-forward). This is a
specification, not an introduction; [`README.md`](README.md) is the orientation.
**Authority:** `docs/DECISIONS.md` `ARC-266` (this module), `ARC-67` and `DEP-30` (the `calendar` System
Pack, which is the only source of a date and a sun), `ARC-69` (pacing is a host command), `ARC-70` (wording),
`ARC-270` (how the 3D client turns the disclosed sun into light). The design and its rulings are
`.structured-coding/plans/mvp0/pr-tw-e-set-c-world-clock-clients.md` §§3.3, 5.1–5.3, 6. Where this document
and a decision disagree, the decision governs and this document is the defect.

---

## 1. What this module is, and what it is not

A client shows the world's date and time and draws the world's light. Both come from the server: the time
from the `clock` frame and every observation (`server/PROTOCOL.md` §§5.2, 5.9), the date and the sun from
the `calendar` System Pack's two disclosed records, `calendar-day` and `calendar-light`
(`systems/calendar`). This module turns those into what a HUD and a light rig need, between frames, and
nothing else.

It is **presentation, never authority** (`INV-TW-6`). It computes no date and no sun from coordinates or
from the machine's clock. Its one use of wall time is to advance the estimate between frames, and the
caller supplies that wall time as a monotonic microsecond count (`Time.get_ticks_usec()`); the module
reads no clock of its own. A world that does not enable `calendar` discloses no record, and every client
then shows today's `Day N, HH:MM` and today's fixed light.

It sends nothing (`INV-SET-1`): no file under `world_time/` names a network identifier
(`tests/acceptance/tests/client_text.rs`).

## 2. Layout and adoption

```text
clients/shared/world_time/     THE MODULE — what a client takes, as res://mineworld_world_time
  world_time.gd                MineWorldWorldTime   anchor, estimate, display rule, day record, roll-over,
                               sun interpolation, light phase
  civil.gd                     MineWorldCivil       is_leap(), days_in_month(), next_date() — integers only
clients/shared/checks/
  world_time_check.gd          the module's headless check over a recorded stream
  fixtures/clock_stream.json   the recorded stream (§8)
```

The reference clients symlink it, as they do the protocol and settings modules:

```text
clients/2d/mineworld_world_time        -> ../shared/world_time
clients/3d-spike/mineworld_world_time  -> ../shared/world_time
```

Both launchers refuse to start when `mineworld_world_time` is not a directory (Windows without
`core.symlinks`), with the same message as for the other modules.

**Dependency direction (`INV-WC-3`).** `world_time/` reads the protocol module's `MineWorldObservation`
and nothing else. It never names the settings module, the host module, or a client's scripts. The date
formatter lives in the settings module (`MineWorldClockFormat`, `SETTINGS.md` §6.6) and takes plain values
(an instant and a date dictionary) from the composition root, so `settings/` never names this module
either.

## 3. `MineWorldWorldTime`

A `RefCounted`. A client owns exactly one and feeds it from its composition root.

### 3.1 Inputs — all from the server, each with the caller's `now_usec`

| Call | When | What it does |
| --- | --- | --- |
| `on_welcome(world: Dictionary, now_usec: int)` | each `welcome` (its `world` summary, `PROTOCOL.md` §5.7) | anchors at `world.at`, `world.time_scale` (default 1), `world.paused` (default false); **resets the display rule** (§3.3) — a new session may legitimately be earlier |
| `on_clock(at: int, time_scale: int, paused: bool, now_usec: int)` | each `MineWorldClient.clock_changed` | re-anchors at `at` with the new scale and pause state |
| `on_observation(observation: MineWorldObservation, now_usec: int)` | each observation | re-anchors at `observation.at()` with the scale and pause state in force; reads `calendar-day` and `calendar-light` about `observation.place()`. A frame without them **keeps the last record** (an observer in no place, in transit, is disclosed none) |

`now_usec` is never read inside the module. Tests pass a scripted schedule.

### 3.2 The estimate

With the anchor `(at_anchor, anchor_usec)`, the scale `s` and the pause state `p`, all integers:

```text
estimate(now_usec) = at_anchor                                   when p
                   = at_anchor + floor((now_usec − anchor_usec) × s / 1 000 000)   otherwise
```

`int` is 64-bit in GDScript, so the product never overflows for any session length. Before the first
anchor the estimate is 0 and `anchored()` is false.

### 3.3 The display rule — never backwards (`INV-TW-8`)

`display_seconds(now_usec) = max(last_display, estimate(now_usec))`, and `last_display` becomes that value.
When a re-anchor puts the estimate behind what was already shown (the estimate ran ahead of a late frame),
the shown minute holds until the world catches up; it is never set back. Only `on_welcome` resets
`last_display`: a graceful restart resumes at or after the last instant shown (the host journal's
`Stopped` record, `ARC-69` item 8), while a killed host may resume earlier — that one case steps back once,
at the new `welcome`, and is the documented exception.

### 3.4 Queries

Each is a function of the module's state and its arguments.

| Query | Answer |
| --- | --- |
| `anchored() -> bool` | an anchor exists |
| `paused() -> bool`, `time_scale() -> int` | from the newest anchor |
| `has_calendar() -> bool` | a `calendar-day` record has been read this session |
| `date_at(seconds: int) -> Dictionary` | `{year, month, day, weekday}` (weekday 0 = Monday, as the pack), or `{}` without a record. The record's date when `seconds` lies in the record's day; for each whole day past `day_start + 86 400` before a newer record arrives, the date advances by `MineWorldCivil.next_date` and the weekday by one. An instant before `day_start` takes the record's date (the record never names a day it does not start). |
| `seconds_into_day(seconds: int) -> int` | `posmod(seconds, 86 400)`: every local midnight is a multiple of 86 400 from instant 0 (`calendar`'s `DAY`, `INV-TW-4`) |
| `sun_at(seconds: int) -> Dictionary` | `{elevation_deg: float, azimuth_deg: float, valid: bool}` from the record's track (§3.5); `valid` false and the elevation `-90.0` without a record |
| `phase_at(seconds: int) -> String` | one of `night`, `astronomical-twilight`, `civil-twilight`, `day` (§3.6); `""` without a record |

### 3.5 Sun interpolation

The track is 97 samples `{elevation, azimuth}` in millidegrees, one every 900 s from the day's 00:00 to
24:00 inclusive (`calendar`'s `SAMPLE_INTERVAL`, `SAMPLES`). For `offset = seconds − day_start`:

- `0 ≤ offset ≤ 86 400`: `i = offset / 900`, `f = (offset mod 900) / 900`; the elevation is the linear
  interpolation of samples `i` and `i + 1` (sample 96 alone at `offset = 86 400`); the azimuth is
  interpolated **unwrapped**: `d = a[i+1] − a[i]`, minus 360 000 when `d > 180 000`, plus 360 000 when
  `d < −180 000`, and the result is taken modulo 360°. A midpoint across north (359.9° → 0.2°) is ≈ 0.05°,
  never ≈ 180°. `valid` is true.
- `offset > 86 400` (past the record's 24:00, before a newer record): the last sample, held, `valid` false.
  The client never extrapolates astronomy.
- `offset < 0`: the first sample, held, `valid` false.

Millidegrees become floats only in this last step (`INV-TW-5`: integers on the wire).

### 3.6 The light phase

The record's `calendar-light` phase holds at the instant of the observation that carried it. A later
instant of the same day advances it across the day record's own `events`, each the instant (seconds after
`day_start`) a phase begins, in the pack's order (`DayEvents::light_changes`): astronomical dawn →
`astronomical-twilight`, civil dawn → `civil-twilight`, sunrise → `day`, sunset → `civil-twilight`, civil
dusk → `astronomical-twilight`, astronomical dusk → `night`; an event that is `null` (polar day or night)
changes nothing. The module reads disclosed instants; it computes none. Past the record's day the last
phase holds until a newer record arrives.

## 4. `MineWorldCivil`

Integers only, mirroring `systems/calendar/src/civil.rs`:

- `is_leap(year)`: divisible by 4 and not by 100, or by 400.
- `days_in_month(year, month)`: 31, 28 or 29, 31, 30, …
- `next_date(date: Dictionary) -> Dictionary`: the civil successor of `{year, month, day}`.
  2026-10-08 → 09; 2026-12-31 → 2027-01-01; 2028-02-28 → 29; 2027-02-28 → 03-01.

It is built, not adopted (`ARC-266`): Godot's `Time.get_date_dict_from_unix_time` would answer the same
but reintroduces a Unix-time path that a reviewer would have to prove never reaches the machine's clock.

## 5. Formatting — in the settings module

`MineWorldClockFormat` (`SETTINGS.md` §6.6) gains:

- `date(d: Dictionary) -> String`: `hud.date` with `{weekday}` (`hud.weekday.<0…6>`), `{day}`,
  `{month}` (`hud.month.<1…12>`) and `{year}`. Every language's template takes the same four
  placeholders (`client_text.rs` holds placeholder sets equal across languages); each language names its
  weekdays and months as its template needs: `Thu`, `Oct` in `en`; `星期四`, `10月` in `zh_Hans`.
- `date_time(seconds: int, d: Dictionary) -> String`: with a date, `hud.datetime` of `{date}` and `{time}`
  (the time in the form in effect, `time_of_day`); with `{}`, the existing `hud.day_time` unchanged
  (`INV-TW-6`).

A composition root calls `date_time(wt.display_seconds(now), wt.date_at(that instant))`. `Paused`
(`hud.paused`) is the HUD's own label, never inside the clock string, so the string a parity check
compares stays one string.

```text
en       Thu 8 Oct 2026, 12:00 AM
zh_Hans  2026年10月8日 星期四 00:00          (automatic ⇒ 24-hour)
zh_Hans  2026年10月8日 星期四 上午 12:00     (12-hour)
```

## 6. How the clients use it

Each composition root:

1. creates one `MineWorldWorldTime`;
2. calls `on_welcome`, `on_clock` (from `MineWorldClient.clock_changed`) and `on_observation` with
   `Time.get_ticks_usec()`;
3. every frame sets its HUD clock line to `MineWorldClockFormat.date_time(…)` and shows `hud.paused`
   beside it while `paused()`;
4. draws light from `sun_at` and `phase_at` of the same displayed instant.

**2D (`clients/2d`).** A `CanvasModulate` under the world's canvas (the HUD is on its own `CanvasLayer`
and is not tinted), coloured from the 2D Presentation Pack's table
`presentation/mineworld-default/2D/daylight.json`, `{ "elevation_deg": [...], "tint": ["#rrggbb", ...] }`,
interpolated by the sun's elevation and clamped at both ends. Without a record, or without the table
(`--presentation=none`), no `CanvasModulate` exists: today's look. `scripts/check_client_rules.py
--check-pack` validates the table (ascending elevations, equal lengths, colours that parse).

**3D (`clients/3d-spike`).** `SliceSun` (`scripts/slice/slice_sun.gd`, `ARC-270`) owns the key light: its
direction from the interpolated sun and the slice's map-north offset, its energy from the elevation by a
clear-sky beam model normalised to ARC-13's 4.4 at 19.3°, its colour from ARC-13's warm sun to neutral by
40°, and below the horizon the key light off, the sky fill, ambient and panorama dimmed by a phase table.
These are presentation values, not physical illuminance: a real 10⁵ : 10⁻³ lux day cannot be shown at the
slice's fixed exposure. **Fixed mode (`INV-WC-2`).** The rig keeps ARC-13's constants, bit for bit, while
no record has been disclosed in the session, in every scripted or capture mode, and with `--sun=fixed`.

## 7. Per platform

`Time.get_ticks_usec()` is monotonic on macOS (mach absolute time), Linux (`CLOCK_MONOTONIC`) and Windows
(`QueryPerformanceCounter`). No file under `world_time/` asks which platform it runs on
(`client_text.rs`), and none names `Time.get_date*`, `Time.get_datetime*`, `Time.get_unix_time_from_system`
or `OS.get_*time*` (the OS-clock scan, `AC-WC-4`).

## 8. Checks

| Check | What it holds | Runs |
| --- | --- | --- |
| `clients/shared/checks/world_time_check.gd` over `fixtures/clock_stream.json` | the estimate, the display rule (`AC-WC-2`), the date from the record and never the OS clock (`AC-WC-4`, epoch 2031-03-14), roll-over (`AC-WC-5`), sun interpolation (`AC-WC-6`), the phase, the formats of §5 in both languages and both forms | `godot --headless --path clients/shared --script res://checks/world_time_check.gd` |
| `tests/acceptance/tests/client_text.rs` | no literal UI string, no network identifier, no platform branch and no OS clock under `world_time/`; catalog keys complete | `cargo test`, in CI |
| `tools/cli/tests/client_world_time.rs` | both clients end to end (`AC-WC-1`, `-3`, `-8`, `-14`) | `cargo test -p mineworld-cli --test client_world_time -- --ignored --test-threads=1` (needs Godot) |

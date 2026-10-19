## DEP-30 — Solar position: `solar-positioning` with `libm`, behind a private seam; civil dates built

**Date** 2026-10-08 · **Status** selected; one dependency added to `systems/calendar` only · **Approved
by** the primary session at PR TW-a's design freeze (SD-TW-a-1, SD-TW-a-6, SD-TW-a-7; §16.8 (a)) ·
**Relates to** `ARC-67`, `REUSE_POLICY.md` §§11–12 · **Design**
`.structured-coding/plans/mvp0/step-19-time-weather.md` §3.1, §5.4, §16 (S19, PR TW-a)

**Problem.** The `calendar` System Pack must say, for a configured latitude and longitude, where the
sun is at any instant and when the day's light events fall (astronomical and civil dawn, sunrise, solar
noon, sunset, civil and astronomical dusk), including polar days and nights. It must also turn an
instant into a civil date and weekday. Every result enters facts, so it must be the same integers on
every platform.

**Options considered** (read 2026-10-08; the full table is step-19 §3.1).

```text
solar position
  (a) solar-positioning 0.7.0 — MIT; NREL SPA and Grena3; azimuth, zenith, rise, set, transit, every
      twilight; numeric Julian-date API with no chrono and no heap; `libm` feature; pre-1.0
  (b) NOAA Solar Calculator equations (Meeus), ~100 lines of our own
  (c) NREL SPA C reference code — not freely redistributable
  (d) sunrise 3.0 — rise and set only, no elevation or azimuth; chrono
  (e) spa 0.5 — not NREL's SPA despite its name; stale
  (f) sun 0.3, astro 2.0 — low accuracy or abandoned
civil dates
  (g) chrono or time — mature, but heavier than the two functions the pack needs
  (h) H. Hinnant's public-domain integer algorithms days_from_civil / civil_from_days, ~25 lines
```

**Choice: (a) and (h).**
- `solar-positioning = { version = "=0.7.0", default-features = false, features = ["libm"] }`, an exact
  pin, in `systems/calendar/Cargo.toml` only. With `libm` and without `std` the crate's transcendental
  functions are the pure-Rust `libm`'s, not the platform's, so the same inputs give the same bits on
  every host. The lock gains `solar-positioning` alone (`libm` 0.2.16 is already locked).
- Civil dates are (h), in `systems/calendar/src/civil.rs`, citing their source.

**Why not the others.** (b) is the fallback, kept ready: it would be ours to maintain and test for no gain
in fit. (c) fails the licence rule. (d) cannot light a scene. (e) and (f) are less accurate, stale or
dead. (g) would add a dependency for two integer functions and a weekday; formatting a date is a
client's job.

**Isolating interface.** `systems/calendar/src/sun.rs` is the only file that names the crate:
`sun_at(…) -> SunSample` and `day_events(…) -> DayEvents`, both returning integers (millidegrees,
whole seconds; round half away from zero). If the pre-1.0 API churns or the crate is withdrawn, the same
two functions are re-implemented from (b) without touching anything else.

**Determinism and its guard.** Every output is quantized to integers before it reaches a fact, a
Process state or a disclosure. A test pins three exact `SunSample` integers (San Diego at 12:00 and
18:00 local on 2026-10-08, 78° N at winter-solstice noon). A change to any of them — a dependency bump,
a platform difference at a rounding boundary — fails the test and is a material stop, never a re-golden.

**Accepted limitations.** The library is pre-1.0 and its README warns the API may change: the exact pin
holds it. Delta T comes from the library's own estimate at the middle of the day's month, so it is fixed
per month, not observed. A day is searched for its first rise and first set of each horizon; at polar
latitudes on the few days with two crossings of one horizon, the second is not an event that day and
the next day's opening phase corrects the light.

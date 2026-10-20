## ARC-68 — Calendar and weather are System Packs; weather is a seeded WGEN-lite generator built from the published algorithm

**Date** 2026-10-09 · **Approved by** the primary session at PR TW-b's design freeze (step-19 §17.9.1;
QTWb-1 … 7 ruled as recommended) · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §17 ·
**Relates to** `ARC-26`, `ARC-28`, `ARC-33`, `ARC-35`, `ARC-61`, `ARC-67`, `DEP-30`, `DEP-31` (weather
data and the fetch tool, TW-d), `REUSE_POLICY.md` §§11–12 · **Design**
`.structured-coding/plans/mvp0/step-19-time-weather.md` §3.3, §5, §6, §15.1, §16, §17 (S19, PRs TW-a and
TW-b)

**Problem.** A world that wants a date, a sun and weather must have them without the kernel learning what
a day, a sunrise or rain is (kernel ignorance), without any client computing them (clients only render,
`INV-TW-6`), and without the time scale or pause reaching a rule (`ARC-67`, `INV-TW-2`). The weather must
be the same for every observer, reproducible from the world's facts, realistic by default, and
replaceable by a world's author.

**Choice.**

1. **Both are System Packs.** `calendar` (`systems/calendar`, PR TW-a) owns the civil date and the sun.
   `weather` (`systems/weather`, PR TW-b) owns the weather: condition, cloud, temperature, precipitation
   and wind, hour by hour, one climate per world. Each is installed by one line in `systems/installed`
   (`ARC-33`) and enabled by a world's `systems:` list.
2. **`weather` depends on `calendar`, never the reverse** (`INV-TW-10`). It declares the dependency, so a
   world that enables `weather` without `calendar` is refused at assembly by the kernel's existing
   dependency check. It reacts to calendar's Public `day-began` and reads the date and the day's light
   events from that fact's payload; it never recomputes a date or the sun. Removing `weather` leaves
   calendar's facts byte-identical.
3. **Both are configured through `ARC-61`'s seam**: `configure/calendar.yaml` and
   `configure/weather.yaml`, decoded by the owner's own types so a refusal names the file, its line and
   column, and the key. Each configuration becomes one SystemInternal fact with no subjects
   (`calendar-configured`, `weather-configured`). A pack that is enabled but not configured states
   nothing.
4. **Each keeps one world-level Process** (`calendar`, `climate`) whose state is the fold of the pack's
   own facts, so a snapshot and a replay agree and a resumed world continues the same weather chain.
5. **Both emit Public facts at changes** — `day-began`, `daylight-changed`, `weather-changed` — so other
   packs (an umbrella seller, a rainy-day routine) may react without either pack knowing them
   (`ARC-26`, `ARC-28`). The per-day weather record, `weather-day`, is SystemInternal.
6. **Both disclose on the observer's place** through presence's existing `PerceptionProvider::discloses`
   (`calendar-day`, `calendar-light`; `weather-today`, `weather-now`). An observer in no place gets
   none.
7. **Integers only.** Weather is fixed-point integers end to end (0.1 °C, 0.1 mm, 0.1 m/s, per-mille
   probabilities, oktas); no `f32` or `f64` appears in the weather pack's source, which a test scans
   for (`INV-TW-5`). Its randomness is counter-based SplitMix64 keyed by the configured seed, the world
   day and a fixed draw index, so a fixed configuration gives the same weather on every platform.
8. **Kernel, contracts and presence are unchanged** (`INV-TW-3`).

**Alternatives rejected.** A `discloses_at` on presence (a perception contract change for a need the
existing `discloses` meets); client-side astronomy or weather (every client would compute a world rule,
and two clients could disagree); a kernel clock that knows days (`ARC-67`).

**Reuse: the weather generator is built from the published algorithm** (`REUSE_POLICY.md`; step-19
§3.3, read 2026-10-08).

```text
(a) WGEN — Richardson 1981, Water Resources Research 17:182–190; Richardson & Wright 1984, USDA-ARS
    ARS-8. A published algorithm: a first-order two-state Markov chain for wet and dry days with monthly
    P(W|D) and P(W|W), gamma wet-day amounts, AR(1) temperature conditioned on wet or dry. A clean-room
    implementation has no licence issue.
(b) LARS-WG — academic, non-commercial licence only
(c) ClimGen (WSU) — Weibull amounts; not pursued
(d) plain monthly rule tables — trivially authored, but independent days with no rain spells
```

**Selected: (a), simplified to "WGEN-lite"** and written by us in `systems/weather`: the Markov chain
with monthly per-mille probabilities; wet-day amounts from a per-month table of five quintile amounts
instead of a gamma sampler (no floating point); temperature as the monthly mean plus an integer AR(1)
anomaly with bounded integer noise. (b) fails the licence rule. (c) offers no gain in fit for this use.
(d) is kept as the degenerate case of the same file: a table with `p_wet_after_dry == p_wet_after_wet`
*is* an independent table, so there is one schema, not two. No crate is adopted: no maintained Rust
weather generator exists, and the generator is about two hundred lines of integer arithmetic over a
mixer the tree already uses (the paced controller's SplitMix64 finalizer, restated in the pack and
pinned to the reference outputs). The station record, its CSV form and the fetch tool's HTTP client are
`DEP-31`'s (TW-d), not this decision's.

**Defaults are content with sources.** The pack's climate is `configure/weather.yaml`. Market Town's
table is derived from NOAA's U.S. Climate Normals 1991–2020 for San Diego Lindbergh Field (USW00023188)
by formulas written in the file's header; values the normals do not give are labelled provisional until
TW-d fits them from the station record. The daily-to-hourly constants are pack constants with cited
sources: the diurnal temperature shape of Parton & Logan (1981), the American Meteorological Society's
drizzle and heavy-rain intensity thresholds, and the WMO okta scale (step-19 QTWb-6).

**Accepted limitations.** One climate per world (regional weather is QTW-12). Wet-hour counts by daily
amount are a stated design default, not a measured climatology, until the hourly layer (TW-g). Wind has
no day-to-day noise in this version.

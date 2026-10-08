# Step 19 — S19: World time, two time domains, pause, day and night, weather

**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing here is frozen, and nothing in it
authorizes implementation. The first PR's design (§19) is likewise a draft.
**Author:** the S19 planning session, 2026-10-08. Worktree `/Users/yuema137/mineworld-worktrees/plan-s19`,
branch `plan/s19-time-weather`, from `main @ f842c52`.
**Binding parents:** `CLAUDE.md` §§2–4; `overall.md` "Parallel build-out", "Framework, not demo", "One world,
two views", "The World Interaction List", and S11, S12, S14; `docs/CORE_CONCEPTS.md`; `docs/ARCHITECTURE.md`;
`docs/ENGINEERING_RULES.md` §12; `docs/REUSE_POLICY.md`; `docs/DECISIONS.md` DEP-6, DEP-8, ARC-9, ARC-26,
ARC-28, ARC-61 to ARC-65; `step-12-server.md` §16 (S11-B, as frozen on its branch); `step-18-interaction-list.md`
§11 (IL-a, as frozen on its branch); `step-13-client-2d.md`; `step-15-demo-3d.md`.
**Scope of this file.** It is the only file this session writes. Edits to `overall.md` and
`docs/DECISIONS.md` are proposed in §18 and applied by the primary session. Decision numbers are placeholders
(`ARC-TW-a`, `DEP-TW-a`, …) until the primary session assigns them.

---

# 1. The requirement

## 1.1 The operator's words, 2026-10-08 (first statement)

> "我觉得我们应该实现小镇的暂停功能，就是如果玩家在游戏中选择，就可以暂停。玩家如果愿意在后台运行让小镇一直运行也可以，但是关闭游戏之后就肯定是存档暂停的。然后白天和夜晚我觉得我们就用现实世界的24小时线性对照到2小时，然后日出日落时间还有光照需要按照纬度设定来计算。然后天气也需要设定，可以用现实中的城市历史记录天气，也可以自己设定规则，这个自由度留给玩家。default我希望设定加州圣地亚哥的天气"

In English:

- **Pause.** A player can pause the town in game. A player may also choose to keep the town running in
  the background. Closing the game always saves and pauses.
- **Day and night.** A real-world 24 hours maps linearly onto 2 real hours: game time runs at 12×.
- **Sun and light.** Sunrise, sunset and lighting are computed from the world's latitude.
- **Weather.** It is configurable. Either a real city's historical records drive it, or the player defines
  rules; that freedom belongs to the player. The default is San Diego, California.

## 1.2 The operator's words, 2026-10-08 (second statement: scale options and two time domains)

> "我觉得我们可以提供不同的选项，24小时map到4小时，2小时，1小时都可以，然后default是2小时。这些在游戏设置里面应该可以直接修改，而且要实时修改。这个时间应该影响天气变化和事件发生速率，但是不应该影响人物动作，物理规律，对话速度等等"

In English:

- **Scale options.** A day maps to 4 h, 2 h or 1 h of real time (6×, 12× or 24×). The default is 2 h (12×).
- **Live changes.** The scale is changed in the in-game settings, live, while the world runs. In
  multiplayer only the host or an admin may change it. It is a world setting, like pause.
- **What the scale affects.** Weather change and the rate of world events: the calendar, day and night,
  the sun, weather, schedules and shifts, wages, and other calendar-driven processes.
- **What it must not affect.** Character actions and movement speed, animation, physics, and dialogue
  pacing stay in real time.

The coordinator's brief derived six obligations from it, answered in §4: the calendar domain and how it
maps onto `WorldTime` (preferring no kernel change); the embodied domain and how real-time pacing is
enforced, including across a live scale change; what the scale means in headless `run`; persistence and
replay of scale changes, and who owns that record; feasibility of NPC agendas at 24×, as an acceptance
criterion fixed before measuring; and the settings UI.

## 1.3 The operator's words, 2026-10-08 (third statement: date and time display)

> "游戏里面还要显示日期和时间，时间可以用12hr也可以24hr，用户在设置里选择"

In English: the game shows the date and the time, and the user chooses a 12-hour or a 24-hour clock in
settings. The coordinator's brief adds:

1. The display uses the world's calendar as the server discloses it. A client never derives the date from
   its own clock.
2. The 12h/24h choice is a client-side presentation preference in the shared client settings module. It
   never reaches the server.
3. Formats follow the language setting (`en` default, `zh-Hans`), for example `Tue 8 Oct 2026, 7:42 PM`
   and `2026年10月8日 星期二 19:42`, with a 12-hour Chinese variant such as `下午 7:42`. Format strings are
   Presentation or translation content, not code.
4. The HUD clock updates live, reflects pause and scale changes, and is shown in both 2D and 3D.
5. Acceptance covers parity of the two clients' date and time (part of "One world, two views"), the
   12h/24h toggle, and both languages.

## 1.4 The primary session's framing (binding on this design)

- **Time, day and night, and weather are world state,** owned by the server. 2D, 3D and multiplayer see
  the same time and the same weather. Clients only render light and weather ("One world, two views";
  `CLAUDE.md` §4 rule 3).
- **Weather is a System Pack.** It is optional and removable, and it owns its facts. Other packs may react
  to it (ARC-26, ARC-28). The kernel stays ignorant.
- **Location and sources are content.** Latitude, longitude, the weather source and the time scale are
  configured through the `configure:` seam (ARC-61, IL-a) or world data. Defaults: San Diego, 12× hosted.
- **Determinism.** Headless `run` stays fast and deterministic. Weather from data comes from a committed,
  licensed data file; weather from rules comes from a seeded generator. No network fetch at run time;
  data is fetched offline by a tool.
- **Pause and time control.** Single-player: the local host pauses or resumes the world clock when the
  player asks, and on window close (save, then stop). Background running: the host keeps serving after
  the client exits (today's server mode). Multiplayer: only the host or an admin pauses, coordinated with
  S11-D's admin surface. S11-B adds `--time-scale`; play defaults to 12× while `run` keeps skipping idle
  time.

## 1.5 The answer in brief

```text
two time domains   WorldTime stays the one kernel clock and IS calendar time. Embodied time is never a
                   world quantity: it is the wall-clock cadence at which embodied inputs arrive (a human's
                   client; the host's consults of hosted controllers). No world rule reads the scale.
                   Scale and pause are host pacing; their changes are recorded as host records in the save.
calendar pack      `calendar` (System Pack): epoch date, UTC offset, latitude/longitude from configure/;
                   date and sun state computed in integers (sun via a float model pinned to `libm`, then
                   quantized) and disclosed on the observer's place; daylight-phase facts.
weather pack       `weather` (System Pack): a world-level `climate` Process; source = a station record
                   (default San Diego, NOAA ISD-Lite, public domain) or seeded rules (WGEN-lite Markov
                   chain); `weather-changed` Public facts at condition change-points; a `Weather`
                   component on every place; temperature disclosed from daily extremes.
host clock         pause / resume / live scale (6×, 12×, 24×) through an admin route owned by S11;
                   single-player close = graceful shutdown (checkpoint), background = keep serving.
clients            light, sky, rain, fog and the HUD date/time rendered from disclosure only; shared
                   presentation-side interpretation, formats as translation content, 12h/24h a client
                   preference.
kernel             unchanged. Contracts unchanged. One additive, defaulted method on presence's
                   PerceptionProvider; one additive table in persistence for host records.
```

---

# 2. Audit (`main @ f842c52`; S11-B at `mvp0/pr-s11b-seats @ 798cb28`; IL-a at `mvp0/pr-il-a-seam @ 2da43f5`)

## 2.1 The kernel clock and `WorldTime`

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `contracts/src/time.rs` `WorldTime(i64)`, `SimDuration(i64)` | Signed whole simulated seconds from the world's epoch. The module doc states "No calendar here": nothing in contracts knows a date, a weekday, a month or a season; there is deliberately no `chrono` in the contract layer. | The calendar is a System Pack's interpretation of seconds. No contract change is needed or wanted. |
| `kernel/src/clock.rs` `WorldClock` | Holds `now` and `started`. Never reads an OS clock; moves only by dispatch, advance or assembly. One rule: a started clock never moves backwards. "How fast a hosted world's seconds pass in real time is the host's decision, not this type's." | Scale and pause belong to the host, exactly as the kernel already says. |
| `kernel/src/advance.rs` | `advance_to(until)` fires every due instant in `(WorldTime, sequence)` order and skips idle time (`Advanced::instants` counts instants that ran). DEP-6's purpose-built queue. | The event queue needs no knowledge of scale: the host decides `until`; the queue fires what is due on the way. |
| `kernel/src/process.rs` `Process` | A record with owner-encoded state bytes, optional place, participants, start and expected end; persisted in snapshots; only the owner changes it. | A world-level `climate` Process can carry the weather source and cursor without inventing a world entity (§9.2). |
| `kernel/src/view.rs` `WorldRead` | Exposes entities, components, relations and processes; **no `now()`**. | A `discloses` implementation cannot see the instant it is answering for (§6.4). |
| Genesis instant | `tools/cli/src/run.rs` `Begun::new` begins every world at `WorldTime::EPOCH` (0); `server/src/host.rs` `HostConfig::default().epoch` is `EPOCH` for an ephemeral hosted world; a persisted world resumes at its saved `now`. | Every world today begins at instant 0. A calendar maps instant 0 to local midnight of its epoch date (§5). |

**Conclusion.** A world has no calendar and no epoch date today. Instant 0 is "midnight" only by
`schedule`'s convention (below).

## 2.2 How `schedule` and other packs use time

| File / symbol | Finding | Domain under §4 |
| --- | --- | --- |
| `systems/schedule/src/time.rs` `DAY = 86_400`, `TimeOfDay::of(at) = at.rem_euclid(DAY)` | Schedule's own convention: midnight is every multiple of 86 400 s from instant 0. Routines are `"HH:MM"` (`worlds/market-town/people/alice.yaml`: `05:30 cafe work`, `14:00 park walk`, `18:00 apartments home`). | calendar |
| `systems/employment` (`system.rs` `next`, `lib.rs`) | Uses `schedule`'s `TimeOfDay`; a `shift` Process per job; "work is attendance": `shift-started` records whether the employee is present, `shift-ended` the seconds worked, `wage-due` follows. | calendar (attendance is measured in world seconds) |
| `systems/group-activity` `ACTIVITY_LENGTH = 3600 s`, `INVITATION_LIFETIME = 1800 s` | An activity lasts one world hour; an invitation expires after 30 world minutes. | calendar (QTW-15 for the invitation's human response window) |
| `systems/conversation` `CONVERSATION_GAP = 300 s` | Silence longer than five world minutes makes the next exchange a new conversation (`ConversationStarted`). Only the conversation pack reacts to `ConversationStarted` (grep of `systems/*/src`, `cognition/*/src`). | calendar by §4's rule; QTW-15 |
| `systems/movement` `MAX_STRIDE = 2000 mm` | A bound on one request, explicitly "not a speed limit" (ARC-26, L-1). No duration. | neither: per request |
| `systems/bodies` | Resolves arrivals and kick/throw/shove at the instant of the request; no time-based Process (grep for `start_process`, `SimDuration`). | neither: instantaneous |
| L-12 (`step-09-social.md`, measured) | Headless: one stride per consult every 900 s, ~8 m per world hour; median journey 2 world hours, one in ten nearly 4; routines are authored in parts of at least 4 h. | headless time-lapse only |

## 2.3 The host: what the server does and discloses about time

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `server/src/runtime.rs` `HostClock` (main) | `now = epoch + started.elapsed().as_secs()`: 1 world second per wall second, whole seconds. | S11-B adds a scale (below). Pause and live changes need a rebase-able clock (§11.2). |
| S11-B SD-B4, SD-B9 (frozen on its branch) | `--time-scale N` (world seconds per wall second, integer ≥ 1, default 1), reported as `WorldSummary.time_scale` in `welcome.world` and `/status`. `--pace SECONDS` is in **world** seconds (default 5); `--hold` in wall seconds. QS11B-4 ruled "a hold is about a network, a pace about a life". | The second operator statement reverses QS11B-4 for consult cadence: walking is embodied, so cadence must be wall seconds (§4.5, QTW-13). |
| S11-B SD-B5/SD-B6 | `HostedController::next_consult(after: WorldTime) -> WorldTime`; consults run in `tick` before the sweep; a hosted request goes through `submit_at`. | The trait can stay; the adapter computes the next instant from a wall cadence and the current scale (§4.5). |
| `runtime.rs` `WorldRuntime::new` | A persisted world's host clock starts from the saved `now`. | Time does not pass while a server is down: "closing saves and pauses" already holds for a stopped host (§11.4). |
| `runtime.rs` `Command::Shutdown` → `checkpoint()` | A graceful stop writes a snapshot at the head. | Window close in single-player maps to a graceful stop (§11.4). |
| `server/src/protocol/summary.rs` `WorldSummary.at` | The world's clock as of a status answer. | Clients get time from `Observation.at` and `WorldSummary`; no date, no scale before S11-B, no pause. |
| `contracts/src/observation.rs` `Observation.at` | Every observation carries the instant. | The HUD clock and every interpolation start from it. |
| `step-12-server.md` §4.9 (S11-D) | Admin HTTP routes under `/admin`, bearer token; "No admin command touches world state … no route that … advances the clock" (I-4). | Pause and scale are host pacing, not world state, so a clock-control route is compatible with I-4 (§11.3). |

## 2.4 Perception: how a pack discloses state

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `systems/presence/src/interaction.rs` `PerceptionProvider::discloses(world, observer, subject)` | Defaulted to nothing; called for each perceived entity, **the observer's place included**; a record about another entity is dropped. | Calendar and weather disclose world-level state as records about the place the observer is in (§6.4, §9.5). |
| `systems/presence/src/observe.rs` `observe(world, observer, at, providers)`, `disclosed(…)` | `observe` has `at`; `disclosed` does not pass it on. A disclosed record survives only if its component type is declared by an enabled system (`owned_by_an_enabled_system`). | An additive, defaulted `discloses_at` carries `at` without changing any existing implementor (§6.4). A derived record's type must be declared by its pack. |
| `contracts/src/event.rs` `Visibility::Public` | "Anyone in the world could have learned of it — a public announcement, a change of season." | Weather changes and daylight changes are Public facts. |

## 2.5 The configuration seam (IL-a, in implementation)

| File / symbol (IL-a branch) | Finding | Consequence |
| --- | --- | --- |
| `authoring/src/configuration.rs` `PackConfiguration` | Owner-typed configuration from `configure/<id>.yaml`, listed in `world.yaml` `configure:`; `seed(&Seeding, &Configuration) -> Vec<Emission>`; `FACTS` filter the drift check; facts should be `SystemInternal`, no subjects. | `calendar` and `weather` are configured this way. Their configured fact is reduced into components on every place, the precedent IL's §4.5 sets for interaction sections. |
| `sdk/rust/src/pack.rs` `configures!()` | Defines `CONFIGURATION`, `CONFIGURATION_FACTS`, `decode_configuration`. A pack that declares nothing is never configured. | There is no "configuration required" flag: a pack enabled without its file seeds nothing (§5.5, QTW-8). |
| `authoring/src/section.rs` `Seeding` | Read access to the assembled world and the resolved keys. | `seed` can enumerate places. |
| IL-a SD-IA-5 | One YAML file per key; no data attachments. | A station record (tens to hundreds of KB) needs either an inline block or an attachment mechanism (§7.6, QTW-7). |

## 2.6 The clients

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `clients/3d-spike/scripts/slice/slice_main.gd` | One sky, one sun, one tonemap, one exposure (ARC-13's rig). The sun is **fixed golden hour**: `SUN_ELEVATION := -19.3`, `SUN_AZIMUTH := -48.0`, `SUN_ENERGY := 4.4`; constants chosen so the north pavement stays in sun and the beam reaches 4.8 m into the café. GI mode chosen by `--gi` (`NONE`, `SSIL`, `VOXEL`, `SDFGI`; default `VOXEL`). | A moving sun replaces constants with a function of disclosed sun state; ARC-13's art intent ("warm, low sun") becomes the *look at golden hour*, not the only hour (§10.2). Baked or semi-static GI must be re-checked under a moving sun (R-TW-6). |
| `clients/3d-spike/scripts/main.gd` | The promenade scene: `ProceduralSkyMaterial` or `PanoramaSkyMaterial` (HDR), `DirectionalLight3D` key at `(-17, 124, 0)`, a blue fill light, depth fog. | Built-in sky materials are already in use; the night sky has none. |
| DEP-8 table | **Sky3D (MIT) is already approved** as a source ("sky and daylight. Credit is required only if the bundled star map ships"). | Adopting it is a code-dependency decision, not a licence question (§10.3). |
| `clients/protocol/mineworld/{observation,space,world_client}.gd` | The shared protocol module; S11 owns it ("No other PR edits the module", ruling 4). | Presentation-side interpretation of sky and weather lives in a separate shared client module (§10.1). |
| `step-13-client-2d.md` A-19, item 8, R-S11-6 | The 2D HUD shows `Observation.at` formatted as day and time; R-S11-6 asked for the time scale "so the client can tick its clock display smoothly between frames". The 2D client (S12 13a) is isometric and not yet merged. | The HUD date and time (§10.5) refines item 8; day/night in 2D is a canvas tint (§10.4). |
| `overall.md` "Framework, not demo" item 4 | The in-game settings menu (language `en`/`zh-Hans`, display, input) is binding on S12 and S14, presentation-only, in one shared client settings module; to be planned after S12 13a and S14 16a. | 12h/24h joins that module. Pause, scale and "keep running in background" are **not** presentation settings: they go to the host (§11.5). |

## 2.7 Randomness and floating point already in the tree

- Seeded randomness is counter-based SplitMix64 (`cognition/rule-controller/src/paced.rs` `mix`, the bodies
  long-run tests). No `rand` dependency in any System Pack.
- No pack uses `chrono`, `time` or `libm` today. Bodies uses Rapier floats with determinism handled under
  DEP-13; positions reach facts as integer millimetres.

---

# 3. Reuse comparisons (`REUSE_POLICY.md`; the operator's standing rule)

All sources below were read on **2026-10-08** by this session's research agents. Nothing was downloaded; sizes
marked *estimate* are computed from the record formats because the NCEI directory listings could not be read
(the GHCN-Daily `by_station/` index exceeds 10 MB; the ISD-Lite 2023 index was truncated before `722900`).

## 3.1 Solar position (server side)

| Option | Licence | Fit and accuracy | Maturity (crates.io API) | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| NOAA Solar Calculator equations (Meeus), <https://gml.noaa.gov/grad/solcalc/calcdetails.html> | No licence statement on the page; US-government work and published mathematics, so a re-implementation is safe (inference, recorded as such) | Rise/set "theoretically accurate to within a minute" within ±72° latitude; valid 1901–2099; elevation, azimuth, twilight all derivable | The page says the calculator is no longer actively supported; the equations are stable | ~100 lines of `f64` of our own, plus golden tests | **Fallback.** Well within game accuracy; ours to maintain. |
| NREL SPA (Reda & Andreas 2004) | Paper public; the **C reference code is not freely redistributable** (pvlib: "Due to license restrictions, the C code must be downloaded separately", <https://pvlib-python.readthedocs.io/en/stable/reference/generated/pvlib.solarposition.spa_c.html>; `midcdmz.nrel.gov` did not resolve) | ±0.0003°, years −2000…6000 — far beyond need | Reference implementation | Vendoring is a licence problem | **Reject the C code.** Clean-room ports of the paper are fine (next row). |
| `solar-positioning` 0.7.0 (2026-09-26), <https://github.com/klausbrunner/solarpositioning-rs> | MIT | Full SPA plus Grena3; azimuth, zenith, sunrise, sunset, transit, civil/nautical/astronomical twilight, custom horizons; >1000 reference test points | 65k downloads total, 56k recent; repo pushed 2026-10-05; **pre-1.0, README warns the API may change** | One dependency; `default-features = false, features = ["libm"]`; `chrono` optional; numeric `JulianDate` API; event searches need no heap | **Adopt, behind a MineWorld seam.** Best fit. |
| `sunrise` 3.0.0 (2026-01-01), <https://github.com/nathan-osman/rust-sunrise> | MIT | Rise/set/dawn times only (Wikipedia "Sunrise equation"); **no elevation or azimuth** | 4.68M / 251k | Uses `chrono::NaiveDate` | **Reject:** too narrow — lighting needs elevation and azimuth. |
| `spa` 0.5.1 (2024-02-11), <https://github.com/frehberg/spa-rs> | Apache-2.0 | Not the NREL SPA despite its name: PSA "sunpos" (<0.5′ from 1999), sunrise "within a few minutes" | 4.54M / 197k; stale | `FloatOps` trait, caller supplies `libm` | **Reject:** unmaintained, misleading name, less accurate. |
| `sun` 0.3.1 (2024-10-18), <https://github.com/flosse/rust-sun> (moved to Codeberg) | MIT | Port of JS suncalc | 178k / 16k | `no_std` not stated | **Reject:** low accuracy and activity. |
| `astro` 2.0.0 (2016), <https://github.com/saurvs/astro-rust> | MIT | Broad Meeus | 189k / 44k; abandoned since 2019 | — | **Reject:** dead. |

**Floating point and determinism.** Every option evaluates `f64` transcendental functions. `std`'s
`sin`/`cos`/`atan2` are not guaranteed bit-identical across platforms; `libm` is a pure-Rust software
implementation and is. The pack therefore builds the crate with `libm` only, quantizes every output to
integers before it reaches a fact, a component or a disclosure (millidegrees for angles, whole world seconds
for event times), and pins golden values for San Diego and for a high-latitude case in a regression test.
Quantization absorbs ulp-level differences except exactly at a rounding boundary; the golden test is what
would catch a dependency upgrade that moves a boundary (R-TW-2).

**Recommendation (DEP-TW-a).** Adopt `solar-positioning` (MIT, `libm`, no `chrono`) inside the `calendar`
pack behind one private function `sun_at(latitude, longitude, instant_utc) -> SunState` and one
`day_events(...) -> DayEvents`; if the pre-1.0 API churns or the dependency is rejected at review, the
same two functions are re-implemented from the NOAA equations (~100 lines) without touching anything else.

**Who computes the sun: the server, once.** Options: (a) the server computes and discloses integer sun state;
(b) the server discloses latitude, longitude and the calendar, and each client computes. (b) puts the same
astronomy in two GDScript clients plus every controller that wants to know whether it is dark, and lets them
disagree (a parity defect by construction). **Chosen: (a).** The server is the only place that runs the model;
clients interpolate between disclosed samples with one shared GDScript helper (§8.3). Sky3D's own astronomy
(below) is switched off.

## 3.2 Historical weather data

| Source | Licence and terms | Fit for San Diego | Size | Verdict |
| --- | --- | --- | --- | --- |
| **NOAA GHCN-Daily**, station `USW00023188` San Diego International Airport (Lindbergh Field), <https://www.ncei.noaa.gov/cdo-web/datasets/GHCND/stations/GHCND:USW00023188/detail> | NOAA open data, **CC0-1.0** ("no restrictions on the use of the data"; <https://registry.opendata.aws/noaa-ghcnh/>, <https://catalog.data.gov/dataset/global-historical-climatology-network-daily-ghcn-daily-version-32>); US federal work. NOAA asks for attribution, no implied endorsement, and that modified data not be presented as original NOAA data. | Period of record 1939-07-01 → 2026-10-03 (updating). Elements (<https://www.ncei.noaa.gov/pub/data/ghcn/daily/readme.txt>): TMAX, TMIN (0.1 °C), PRCP (0.1 mm), AWND (0.1 m/s), weather types WT01 fog, WT02 heavy fog, WT03 thunder, WT08 smoke/haze, WT13 mist, WT14 drizzle, WT16 rain, WT21 ground fog. **Which WT flags are populated for this station, and in which years, is unverified** (checked by the fetch tool, §6.4). | *Estimate:* one wide row per day, 10 years ≈ 3,650 rows ≈ **120–150 KB** uncompressed. | **Adopt as the default daily layer.** |
| NOAA ISD / ISD-Lite, station `722900-23188`, <https://www.ncei.noaa.gov/pub/data/noaa/isd-lite/isd-lite-format.txt> | Same NOAA terms (CC0 / public domain). | Hourly: temperature, dewpoint, sea-level pressure, wind direction and speed, **sky total coverage code (0–19)**, precipitation 1 h / 6 h. **No visibility or fog field.** **Superseded**: no updates after ~2025-08-24; NCEI's FTP/HTTPS ISD service retired 2026-07-31 (notice 2026-06-23); still served through NODD (<https://www.nesdis.noaa.gov/news/global-historical-climate-network-hourly-integrated-surface-data-global-hourly>, <https://forum.cmascenter.org/t/transition-isd-to-ghcnh/5985>). | *Estimate:* ~0.5 MB/year raw, ~80–120 KB gzipped; 10 years ≈ 5 MB raw, 1 MB gz. | **Reject for new work** in favour of GHCNh. |
| **NOAA GHCNh** (GHCN-hourly), S3 bucket `noaa-ghcnh-pds`, <https://registry.opendata.aws/noaa-ghcnh/> | CC0. | ISD's successor: one file per station for the whole record, updated daily, **includes visibility and present weather** (fog). | Per-station file size not read; derived hourly sky-cover summary is small (§6.4). | **Adopt as the optional hourly layer** (sky cover, fog) — second PR of the data plan (QTW-6). |
| Meteostat, <https://dev.meteostat.net/license>, <https://dev.meteostat.net/terms.html> | **Now CC BY 4.0** ("for any purpose, even commercially", credit Meteostat and its providers) — *not* the CC BY-NC the brief expected; the change from older terms is recorded with this date. | Repackages NOAA for this station; adds nothing. | — | **Reject:** a redundant intermediary with an attribution duty NOAA's own data does not carry. |
| Open-Meteo historical, <https://open-meteo.com/en/licence>, <https://open-meteo.com/en/terms> | Data CC BY 4.0 with visible attribution ("Weather data by Open-Meteo.com"); code AGPLv3; **the free API is non-commercial only** ("You may only use the free API services for non-commercial purposes"). Historical API believed to be ERA5-based (unverified). | Gridded reanalysis; convenient API. | — | **Reject:** the free access route fails DEP-8's commercial-use rule even though the data licence passes. |
| ERA5 / Copernicus | **CC BY 4.0 since 2025-07-02** (<https://forum.ecmwf.int/t/cc-by-licence-to-replace-licence-to-use-copernicus-products-on-02-july-2025/13464>); attribution "Generated using or contains modified Copernicus Climate Change Service information [year]". | ~31 km grid smooths San Diego's coastal marine layer ("May Gray / June Gloom"); needs a CDS account and NetCDF/GRIB extraction. | — | **Fallback only**, for a world placed where no station exists (QTW-10). |

**San Diego default plan** — §6.4.

## 3.3 Rule-based weather

| Option | Licence | Fit | Verdict |
| --- | --- | --- | --- |
| **WGEN** (Richardson 1981, *Water Resources Research* 17:182–190; Richardson & Wright 1984, USDA-ARS ARS-8; <https://modeling.bsyse.wsu.edu/ClimGen/documentation/description.pdf>, <https://int-res.com/articles/cr1998/10/c010p095.pdf>) | Published algorithm; a clean-room implementation has no licence issue | First-order two-state Markov chain for wet/dry days with seasonally varying P(W\|D), P(W\|W); wet-day amounts from a two-parameter gamma; temperature as AR(1) conditioned on wet/dry. Known weaknesses: spell lengths, extremes. | **Adopt the structure, simplified ("WGEN-lite")**: Markov wet/dry with monthly integer probabilities (per-mille), amounts from a per-month integer table of quantiles instead of a gamma sampler (no floats), temperature as monthly mean ± integer-bounded noise with an integer AR(1) coefficient. All in `i64`, driven by counter-based SplitMix64 (the tree's existing generator, §2.7). |
| LARS-WG, <https://sites.google.com/view/lars-wg/> | Academic, non-commercial only | Stronger spell modelling | **Reject:** licence. |
| ClimGen (WSU) | Not checked | Weibull amounts | Not pursued. |
| Plain rule tables (per month: chance of each condition) | Ours | Independent days; no persistence of rain spells; trivially authored | **Kept as the degenerate case of the same file**: a rules file with P(W\|W) = P(W\|D) *is* an independent table. One schema, not two. |

The rules are content: `configure/weather.yaml` carries the monthly table. A tool (`tools/weather-fetch`,
§6.4) can **fit** the table from the committed station file, so "San Diego rules" and "San Diego record"
describe the same climate; a player may also author a table by hand (e.g. "always rainy town").

## 3.4 Godot sky, light and weather visuals (Presentation Pack content)

| Option | Licence | Fit | Verdict |
| --- | --- | --- | --- |
| **Sky3D** v2.1.0 (2026-05-19), <https://github.com/TokisanGames/Sky3D> | MIT (Cory Petkovsek 2023–25; J. Cuéllar 2021); star-map assets carry their own licences — **already approved in DEP-8's table**, credit required only if the bundled star map ships | Godot 4.3+, Forward+/Mobile/Compatibility; sun, moon with phases, stars, clouds, fog; no rain. Computes the sun itself (`TimeOfDay.gd`: lat/long/UTC offset/date, SIMPLE or REALISTIC), **but can be driven externally**: `SkyDome.gd` exports `sun_azimuth`, `sun_altitude`, `moon_altitude`; turn off `game_time_enabled`/`editor_time_enabled`. ~906 stars. | **Adopt for the 3D client**, driven by disclosed sun state; its clock and astronomy off (otherwise it duplicates the server's authority). Moon: QTW-11. |
| Built-in `PhysicalSkyMaterial` / `ProceduralSkyMaterial`, <https://docs.godotengine.org/en/stable/classes/class_physicalskymaterial.html> | Godot (MIT) | Sun from the first `DirectionalLight3D`; Rayleigh/Mie, turbidity, `night_sky` texture; no clouds, moon or stars | **Fallback path** and the first PR's renderer: rotate the existing key light from disclosed state; zero dependency. |
| Time of Day (J. Cuéllar) forks, e.g. `Boyquotes/jc.time-of-day` | MIT | Original repository gone; Sky3D is its maintained continuation | **Reject** in favour of Sky3D. |
| SunshineClouds2 (Bonkahe), Godot asset library 17397 | MIT, ~609 stars, pushed 2026-09-27, Godot 4.4 | Volumetric clouds only | **Later, optional** (overcast visuals); not in S19's PRs. |
| godotshaders.com sky/rain shaders | Per-shader (CC0, MIT, GPL-3, Shadertoy ports) | Mixed | Only individually audited CC0/MIT items; none needed now. |
| Rain: `GPUParticles3D` + `GPUParticlesCollisionHeightField3D`, <https://docs.godotengine.org/en/stable/tutorials/3d/particles/collision.html> | Godot | Camera-following height field, "When Moved"/"Always" update; SDF collision absent in Compatibility; no maintained MIT 3D rain addon found (WeatherSystem2D and a screen-space rain shader are 2D/overlay, MIT) | **Build in-house**: a camera-following box emitter with streak quads and a height-field collider — small commodity work. |

**Shared versus renderer-only.** Shared (one GDScript module, used by both clients): reading the disclosed
calendar and weather records, the clock estimate between frames, interpolation of the sun track, date and time
formatting, and the mapping from weather condition to a *presentation intent* (`rain_intensity`, `fog_density`,
`cloud_cover`, `wetness`, all 0..1). Renderer-only: how the 3D client spends those intents (Sky3D, key light,
particles, fog volume, wet-surface roughness) and how the 2D client does (a canvas tint and a particle
overlay). Weather visuals, sky assets and the condition → intent table are Presentation Pack content.

<!-- §4 onward follows -->

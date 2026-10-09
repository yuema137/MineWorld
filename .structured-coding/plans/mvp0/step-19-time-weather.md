# Step 19 — S19: World time, two time domains, pause, day and night, weather

**Lifecycle:** step plan reviewed; the questions are ruled (§14.1, 2026-10-08). **TW-a (§16) is `DESIGN
FROZEN 2026-10-08`**, and §16 alone authorizes implementation, under its execution contract (§16.7). TW-b to
TW-g are scoped in §11 and are not frozen.
**Author:** the S19 planning session, 2026-10-08. Worktree `/Users/yuema137/mineworld-worktrees/plan-s19`,
branch `plan/s19-time-weather`, from `main @ f842c52`.
**Binding parents:** `CLAUDE.md` §§2–4; `overall.md` "Parallel build-out", "Framework, not demo", "One world,
two views", "The World Interaction List", and S11, S12, S14; `docs/CORE_CONCEPTS.md`; `docs/ARCHITECTURE.md`;
`docs/ENGINEERING_RULES.md` §12; `docs/REUSE_POLICY.md`; `docs/DECISIONS.md` DEP-6, DEP-8, ARC-9, ARC-26,
ARC-28, ARC-61 to ARC-65; `step-12-server.md` §16 (S11-B, as frozen on its branch); `step-18-interaction-list.md`
§11 (IL-a, as frozen on its branch); `step-13-client-2d.md`; `step-15-demo-3d.md`.
**Scope of this file.** It is the only file this session writes. Edits to `overall.md` and
`docs/DECISIONS.md` are proposed in §15 and applied by the primary session. Decision numbers were
assigned centrally on 2026-10-08: ARC-67 (two time domains), ARC-68 (calendar and weather are System Packs),
ARC-69 (pause and scale are host commands), DEP-30 (solar position and civil dates), DEP-31 (weather data
and the weather generator). ARC-66 belongs to E-c.

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
3. Formats follow the language setting (`en` default, `zh-Hans`), for example `Thu 8 Oct 2026, 7:42 PM`
   and `2026年10月8日 星期四 19:42` (8 October 2026 is a Thursday; the brief wrote `Tue`), with a 12-hour Chinese variant such as `下午 7:42`. Format strings are
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
calendar pack      `calendar` (System Pack, §5): epoch date, UTC offset, latitude/longitude from configure/;
                   `day-began` (with the date and a 15-minute integer sun track) and `daylight-changed`
                   Public facts; sun from `solar-positioning` built with `libm`, quantized to integers;
                   state folded into one `calendar` Process and disclosed on the observer's place.
weather pack       `weather` (System Pack, §6, depends on calendar): a world-level `climate` Process;
                   source = a station record (default San Diego, NOAA GHCN-Daily USW00023188, CC0,
                   2015–2024, ~130 KB) or seeded integer rules (WGEN-lite Markov chain); `weather-day`
                   and Public `weather-changed` facts; hourly state disclosed on the observer's place.
host clock         (§7) pause / resume / live scale (6×, 12×, 24×) via `/admin/clock` on S11-D's surface;
                   a `clock` frame; single-player close = graceful shutdown (checkpoint); background =
                   keep serving; hosted consult cadence in wall seconds; changes in a host journal.
clients            (§8) light, sky, rain, fog and the HUD date/time rendered from disclosure only through
                   one shared `world_time` client module; formats are translation content; 12h/24h a
                   client preference; pause and day length are host commands in the same menu.
kernel             unchanged. Contracts and presence unchanged. One additive table in persistence for
                   the host journal; one additive frame and refusal in the protocol.
```

---

# 2. Audit (`main @ f842c52`; S11-B at `mvp0/pr-s11b-seats @ 798cb28`; IL-a at `mvp0/pr-il-a-seam @ 2da43f5`)

## 2.1 The kernel clock and `WorldTime`

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `contracts/src/time.rs` `WorldTime(i64)`, `SimDuration(i64)` | Signed whole simulated seconds from the world's epoch. The module doc states "No calendar here": nothing in contracts knows a date, a weekday, a month or a season; there is deliberately no `chrono` in the contract layer. | The calendar is a System Pack's interpretation of seconds. No contract change is needed or wanted. |
| `kernel/src/clock.rs` `WorldClock` | Holds `now` and `started`. Never reads an OS clock; moves only by dispatch, advance or assembly. One rule: a started clock never moves backwards. "How fast a hosted world's seconds pass in real time is the host's decision, not this type's." | Scale and pause belong to the host, exactly as the kernel already says. |
| `kernel/src/advance.rs` | `advance_to(until)` fires every due instant in `(WorldTime, sequence)` order and skips idle time (`Advanced::instants` counts instants that ran). DEP-6's purpose-built queue. | The event queue needs no knowledge of scale: the host decides `until`; the queue fires what is due on the way. |
| `kernel/src/process.rs` `Process` | A record with owner-encoded state bytes, optional place, participants, start and expected end; persisted in snapshots; only the owner changes it. | A world-level `climate` Process can carry the weather source and cursor without inventing a world entity (§6.6); likewise a `calendar` Process (§5.3). |
| `kernel/src/view.rs` `WorldRead` | Exposes entities, components, relations and processes; **no `now()`**. | A `discloses` implementation cannot see the instant it is answering for; S19 therefore discloses state folded from its own facts (§5.3). |
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
| `server/src/runtime.rs` `HostClock` (main) | `now = epoch + started.elapsed().as_secs()`: 1 world second per wall second, whole seconds. | S11-B adds a scale (below). Pause and live changes need a rebase-able clock (§4.2). |
| S11-B SD-B4, SD-B9 (frozen on its branch) | `--time-scale N` (world seconds per wall second, integer ≥ 1, default 1), reported as `WorldSummary.time_scale` in `welcome.world` and `/status`. `--pace SECONDS` is in **world** seconds (default 5); `--hold` in wall seconds. QS11B-4 ruled "a hold is about a network, a pace about a life". | The second operator statement reverses QS11B-4 for consult cadence: walking is embodied, so cadence must be wall seconds (§4.4, QTW-13). |
| S11-B SD-B5/SD-B6 | `HostedController::next_consult(after: WorldTime) -> WorldTime`; consults run in `tick` before the sweep; a hosted request goes through `submit_at`. | The trait can stay; the adapter computes the next instant from a wall cadence and the current scale (§4.4). |
| `runtime.rs` `WorldRuntime::new` | A persisted world's host clock starts from the saved `now`. | Time does not pass while a server is down: "closing saves and pauses" already holds for a stopped host (§7.5). |
| `runtime.rs` `Command::Shutdown` → `checkpoint()` | A graceful stop writes a snapshot at the head. | Window close in single-player maps to a graceful stop (§7.5). |
| `server/src/protocol/summary.rs` `WorldSummary.at` | The world's clock as of a status answer. | Clients get time from `Observation.at` and `WorldSummary`; no date, no scale before S11-B, no pause. |
| `contracts/src/observation.rs` `Observation.at` | Every observation carries the instant. | The HUD clock and every interpolation start from it. |
| `step-12-server.md` §4.9 (S11-D) | Admin HTTP routes under `/admin`, bearer token; "No admin command touches world state … no route that … advances the clock" (I-4). | Pause and scale are host pacing, not world state, so a clock-control route is compatible with I-4, with a clarifying amendment (§7.3, §15.3). |

## 2.4 Perception: how a pack discloses state

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `systems/presence/src/interaction.rs` `PerceptionProvider::discloses(world, observer, subject)` | Defaulted to nothing; called for each perceived entity, **the observer's place included**; a record about another entity is dropped. | Calendar and weather disclose world-level state as records about the place the observer is in (§5.5, §6.6). |
| `systems/presence/src/observe.rs` `observe(world, observer, at, providers)`, `disclosed(…)` | `observe` has `at`; `disclosed` does not pass it on. A disclosed record survives only if its component type is declared by an enabled system (`owned_by_an_enabled_system`). | S19 does not need `at` at disclosure: calendar and weather disclose state folded from their own facts; an additive `discloses_at` was considered and rejected (§5.3). A derived record's type must be declared by its pack. |
| `contracts/src/event.rs` `Visibility::Public` | "Anyone in the world could have learned of it — a public announcement, a change of season." | Weather changes and daylight changes are Public facts. |

## 2.5 The configuration seam (IL-a, in implementation)

| File / symbol (IL-a branch) | Finding | Consequence |
| --- | --- | --- |
| `authoring/src/configuration.rs` `PackConfiguration` | Owner-typed configuration from `configure/<id>.yaml`, listed in `world.yaml` `configure:`; `seed(&Seeding, &Configuration) -> Vec<Emission>`; `FACTS` filter the drift check; facts should be `SystemInternal`, no subjects. | `calendar` and `weather` are configured this way. Their configured fact is reduced into the pack's own world-level Process state (§5.3, §6.6), not into components on every place. |
| `sdk/rust/src/pack.rs` `configures!()` | Defines `CONFIGURATION`, `CONFIGURATION_FACTS`, `decode_configuration`. A pack that declares nothing is never configured. | There is no "configuration required" flag: a pack enabled without its file seeds nothing (§5.5, QTW-8). |
| `authoring/src/section.rs` `Seeding` | Read access to the assembled world and the resolved keys. | `seed` can enumerate places. |
| IL-a SD-IA-5 | One YAML file per key; no data attachments. | A station record (tens to hundreds of KB) needs either an inline block or an attachment mechanism (§6.5, QTW-7). |

## 2.6 The clients

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `clients/3d-spike/scripts/slice/slice_main.gd` | One sky, one sun, one tonemap, one exposure (ARC-13's rig). The sun is **fixed golden hour**: `SUN_ELEVATION := -19.3`, `SUN_AZIMUTH := -48.0`, `SUN_ENERGY := 4.4`; constants chosen so the north pavement stays in sun and the beam reaches 4.8 m into the café. GI mode chosen by `--gi` (`NONE`, `SSIL`, `VOXEL`, `SDFGI`; default `VOXEL`). | A moving sun replaces constants with a function of disclosed sun state; ARC-13's art intent ("warm, low sun") becomes the *look at golden hour*, not the only hour (§8.3). Baked or semi-static GI must be re-checked under a moving sun (R-TW-6). |
| `clients/3d-spike/scripts/main.gd` | The promenade scene: `ProceduralSkyMaterial` or `PanoramaSkyMaterial` (HDR), `DirectionalLight3D` key at `(-17, 124, 0)`, a blue fill light, depth fog. | Built-in sky materials are already in use; the night sky has none. |
| DEP-8 table | **Sky3D (MIT) is already approved** as a source ("sky and daylight. Credit is required only if the bundled star map ships"). | Adopting it is a code-dependency decision, not a licence question (§3.4). |
| `clients/protocol/mineworld/{observation,space,world_client}.gd` | The shared protocol module; S11 owns it ("No other PR edits the module", ruling 4). | Presentation-side interpretation of sky and weather lives in a separate shared client module (§8.3). |
| `step-13-client-2d.md` A-19, item 8, R-S11-6 | The 2D HUD shows `Observation.at` formatted as day and time; R-S11-6 asked for the time scale "so the client can tick its clock display smoothly between frames". The 2D client (S12 13a) is isometric and not yet merged. | The HUD date and time (§8.2) refines item 8; day/night in 2D is a canvas tint (§8.3). |
| `overall.md` "Framework, not demo" item 4 | The in-game settings menu (language `en`/`zh-Hans`, display, input) is binding on S12 and S14, presentation-only, in one shared client settings module; to be planned after S12 13a and S14 16a. | 12h/24h joins that module. Pause, scale and "keep running in background" are **not** presentation settings: they go to the host (§7.3, §8.1). |

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

**Recommendation (DEP-30).** Adopt `solar-positioning` (MIT, `libm`, no `chrono`) inside the `calendar`
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

---

# 4. Design — two time domains

## 4.1 Definitions (proposed terms; added to `CORE_CONCEPTS.md` §time by ARC-67, not synonyms of existing ones)

- **Calendar time** is `WorldTime`: the world's one clock, whole simulated seconds from the epoch. Every fact,
  every Process wake-up and every `Observation.at` is stamped in it. The calendar, the sun, weather, routines,
  shifts, wages and every calendar-driven Process are functions of it. *Nothing about this changes.*
- **Embodied time** is real (wall-clock) time as experienced by a body: how fast a player walks, how long a
  line takes to type or to speak, how long an animation or a physics impulse plays. **Embodied time is not a
  world quantity.** No fact, component, Process or rule carries it. It exists only as the *cadence at which
  embodied inputs arrive at the server*: a human's client sends `MoveIntent`s and lines at human speed; the
  host consults a hosted controller every `cadence` wall seconds; a client animates at frame rate.
- **Time scale** `s` is a host pacing parameter: world seconds per wall second while the world runs
  (integer ≥ 1; play offers 6, 12, 24; default 12). **Paused** is a host pacing state in which world seconds
  do not pass. Neither is world state; no System Pack can read either.

## 4.2 How the scale maps onto `WorldTime` and the event queue

The kernel already says pacing is the host's (`kernel/src/clock.rs`: "How fast a hosted world's seconds pass
in real time is the host's decision"). The host computes the target instant from the wall clock and calls
`advance_to(target)`; the queue fires everything due on the way in `(WorldTime, sequence)` order. The scale
changes only *how fast the target moves*.

Alternatives compared:

| Option | What changes | Verdict |
| --- | --- | --- |
| **A. Host-paced `WorldTime` (chosen).** `HostClock` keeps a *segment*: `(wall_anchor, world_anchor, scale, paused)`; `now = world_anchor + floor((wall − wall_anchor) × scale)` when running, `world_anchor` when paused. A pause, resume or scale change closes the segment at the current `now` and opens a new one. | `server/src/runtime.rs` `HostClock` only (S11-B already adds a constant scale). | Kernel and contracts untouched; `run` untouched; the queue untouched. Monotonicity holds because every new segment starts at the old segment's `now`. |
| B. Two kernel clocks (calendar and embodied) with an explicit ratio in the kernel | `WorldClock`, `advance`, every Process start (which clock?), every fact stamp | **Reject.** Kernel learns a game concept (a "day" ratio), every Process must say which clock it uses, and persistence must reconcile two timelines. Violates kernel ignorance and change amplification. |
| C. Scale applied inside each System Pack (each pack multiplies its durations) | Every calendar-driven pack reads a shared scale | **Reject.** Packs would read host state; a rule depending on the host is a determinism and ownership defect; headless and hosted would diverge. |
| D. Embodied actions take world durations divided by the scale (e.g. a walk Process) | Movement, conversation, bodies | **Reject.** Embodied pacing is already not modelled in world time (ARC-26: `MAX_STRIDE` is "not a speed limit"); inventing durations just to cancel the scale adds the coupling the operator asked to avoid. |

**Consequence for embodied rules.** The rule "the scale must not affect actions, physics or dialogue speed" is
met *by construction* for anything that is per-request (movement strides, bodies impulses, a line spoken): the
server resolves each request at the instant it arrives, and the wall-clock rate of requests is the body's.
It is met *by the cadence rule* for hosted controllers (§4.4). The residue — world-time constants that in
practice measure a human's embodied response — is audited in §4.5.

## 4.3 Live scale changes and pause are recorded inputs

- **Owner.** The host (the `server` crate, S11's lane). Not a System Pack: no rule may depend on them.
- **Record.** Every segment boundary is appended to a **host journal** in the save: `HostRecord { at:
  WorldTime, revision: Revision, kind: Paused | Resumed | Scaled { to: u32 } | Started { scale, paused } |
  Stopped }`, written in the same transaction as the checkpoint that follows it or by itself. It is not a fact
  (I-4: admin touches no world state) and it never enters the fact log, revisions or drift checks.
- **Why it is needed, and what "exact replay" means.** The *world* replays exactly without it: every fact
  already carries its instant, and state is the fold of facts. What the journal adds is the *host's*
  timeline: for a hosted world it lets a tool reconstruct when each hosted controller was due (`cadence × s`)
  and therefore re-run the hosted controllers against the recorded facts and get the same requests (the
  hosted analogue of AC-12). The adversarial test is exactly that (TW-c, criterion 4).
- **Persistence.** One additive table `host_journal(seq INTEGER PRIMARY KEY, at INTEGER, revision INTEGER,
  kind TEXT, scale INTEGER)`; an older save without the table opens with an empty journal (no migration of
  existing rows). The `persistence` crate owns the table; the server owns its meaning.

Alternatives: (a) a SystemInternal fact from a `clock` pack — rejected, it would make pacing world state and
let rules react to the scale; (b) a sidecar file beside the save — rejected, a second persistence authority
that can drift from the checkpoint.

## 4.4 Hosted controllers: cadence in wall seconds

S11-B's `--pace SECONDS` is in **world** seconds (QS11B-4 ruled "a pace is about a life"). Under the second
operator statement that ruling inverts for *consult cadence*: a hosted Person walks one stride per consult,
and walking is embodied. Proposed (QTW-13, operator-material because it amends a ruling):

- `--pace` becomes **wall seconds** (`--cadence` is the clearer name; QTW-13). The adapter's
  `next_consult(after) = after + cadence × s_current`, computed when the consult is scheduled.
- **Live scale change.** The host recomputes every pending `next_consult` from the change instant: a seat due
  at `t` under scale `s₀` with `w` wall seconds still to wait is rescheduled to `now + w × s₁`. So a walker's
  wall-clock stride rate never jumps. **Pause** freezes them all (the clock does not move, nothing is due).
- **Headless `run` is unchanged**: its paced controller keeps `pace = 900` world seconds — a time-lapse in
  which embodied cadence is notionally `900 / s` wall seconds. `run` has no scale (§4.6).

## 4.5 World-time constants that are embodied in practice (audit result)

| Constant | Pack | At 12× it lasts (wall) | Domain ruling | Action |
| --- | --- | --- | --- | --- |
| `CONVERSATION_GAP = 300 s` | conversation | 25 s (12.5 s at 24×) | Calendar by the rule "no pack reads the scale"; but a human typing a long line can open a "new conversation" | QTW-15: make it configurable content through IL-b, raise the interactive World Packs' value; headless default unchanged |
| `INVITATION_LIFETIME = 1800 s` | group-activity | 150 s (75 s at 24×) | Calendar ("the café meetup at 3 pm") but answered by a human | QTW-15, same remedy |
| `ACTIVITY_LENGTH = 3600 s` | group-activity | 5 min | Calendar | none |
| Shift lengths, routine parts | schedule, employment | parts ≥ 4 h → ≥ 20 min (≥ 10 min at 24×) | Calendar | §9 feasibility |
| `MAX_STRIDE = 2000 mm` | movement | per request | Neither | none |
| bodies impulses | bodies | per request | Neither | none |

No pack constant is changed by S19. QTW-15 is a content question, routed to IL-b.

## 4.6 What the scale means in headless `run`

Nothing. `run` has no wall clock: it advances day by day with idle skipping, as fast as the machine allows
(`AC-11`). It neither accepts nor records a scale; its output for a world without `calendar`/`weather` is
byte-identical to today's (INV-TW-1). Calendar and weather packs behave identically in `run` and hosted,
because they read only `WorldTime`.

---

# 5. Design — the `calendar` System Pack

## 5.1 What it owns

The world's civil calendar and sun: which local date and weekday an instant is, where the sun is, and the
day's light events. It owns no other pack's facts. Removing it removes the date display's source, day and
night, and nothing else; a world without it renders today's fixed golden hour and an un-dated clock.

## 5.2 Configuration (`configure/calendar.yaml`, ARC-61 seam; content, not code)

```yaml
# configure/calendar.yaml — the market-town default
epoch: 2026-10-08          # local civil date at instant 0; instant 0 is that date's local midnight
utc_offset: -08:00         # fixed standard offset; no DST in v1 (QTW-9)
latitude: 32.7157          # San Diego; decimal degrees, stored as micro-degrees (i32)
longitude: -117.1611
```

- Instant 0 = local midnight of `epoch`, so the calendar agrees with `schedule`'s existing convention
  (`TimeOfDay::of(at) = at mod 86 400`) without touching `schedule` (INV-TW-4).
- Decoded into owner types (`CalendarDate`, `UtcOffsetSeconds(i32)`, `Latitude(MicroDegrees)`,
  `Longitude(MicroDegrees)`); a latitude outside ±90°, a longitude outside ±180°, an offset outside ±14 h, or an
  invalid date is a refusal at assembly (IL-a's `Rejection`), naming the file and key.
- Polar day and night are legal (no sunrise that day); the record says so (§5.4).

## 5.3 Facts (event types owned by `calendar`)

| Event | Visibility | When | Payload (integers) |
| --- | --- | --- | --- |
| `calendar-configured` | SystemInternal | genesis (IL-a seed) | the decoded configuration |
| `day-began` | Public | each local midnight | `date {y, m, d}`, `weekday` (0 = Monday), `day_start: WorldTime`, `events` (§5.4), `track` (§5.4) |
| `daylight-changed` | Public | at astronomical dawn, civil dawn, sunrise, sunset, civil dusk, astronomical dusk | `phase` ∈ {night, astronomical-twilight, civil-twilight, day} |

Reducers fold them into the state of one world-level `calendar` Process (started by the configured fact's
reduction, woken at the next due light event or midnight). The Process state is the current day record; it is
persisted in snapshots like every Process, so a resumed world needs no recomputation.

Why facts rather than a pure function evaluated at disclosure: `WorldRead` has no `now()` (§2.1), presence's
`discloses` has no `at` (§2.4), and other packs must be able to *react* to dawn and dusk (street lamps, a
sleep routine) through facts (ARC-26, ARC-28). Rejected alternative: an additive `discloses_at` on presence's
`PerceptionProvider` — it would let disclosure depend on the instant without an owned fact, and reactors would
still need facts. With facts, presence, kernel and contracts are untouched (INV-TW-3).

## 5.4 The day record

```text
CalendarDay {
  date: CalendarDate { year: i32, month: u8, day: u8 }, weekday: u8,
  day_start: WorldTime,                       // instant of local midnight
  events: DayEvents { astronomical_dawn, civil_dawn, sunrise, solar_noon, sunset, civil_dusk,
                      astronomical_dusk }     // each Option<u32> seconds after day_start; None in polar cases
  track: [SunSample; 97]                      // every 900 s from 00:00 to 24:00 inclusive
}
SunSample { elevation: i32 /* millidegrees */, azimuth: i32 /* millidegrees from north, clockwise */ }
```

- Date arithmetic: days-from-civil / civil-from-days (H. Hinnant's public-domain integer algorithms, ~25
  lines) in the pack. Compared with `chrono` and `time`: both are mature, but the pack needs only these two
  integer conversions and a weekday; formatting happens in clients. Recorded in DEP-30 as "build: two
  functions; a dependency for them would be heavier than the code".
- The 15-minute track (97 × 2 × `i32`) is the only sun the clients see; clients interpolate linearly by their
  clock estimate (§8.3). At 15 minutes the sun moves ≤ 3.75°; linear interpolation error is far under a degree.
- Leap years follow the proleptic Gregorian calendar; dates are valid for years 1901–2099 (NOAA's validity
  range, a refusal outside it).

## 5.5 Disclosure

`calendar` implements presence's `PerceptionProvider::discloses` for the observer's **place**: one record
`calendar.day` (the current `CalendarDay`, minus nothing) and one record `calendar.light` (`phase`). Both
change only at facts, so S11-C's delta stream sends them once per change. Every observation still carries
`at`; clients compute the time of day as `at − day_start` and the HUD from there (§8.2). The record types are
declared by `calendar`, so presence keeps them (`owned_by_an_enabled_system`).

A world that enables `calendar` without `configure/calendar.yaml` seeds nothing and discloses nothing
(IL-a's "a pack that declares nothing is never configured"); `mineworld check` warns (QTW-8).

---

# 6. Design — the `weather` System Pack

## 6.1 What it owns, and what it depends on

The world's weather: condition, cloud, temperature, precipitation and wind, hour by hour, for the whole
world (one climate per world in v1; regional weather is a later, additive change, QTW-12). It **depends on
`calendar`** (declared pack dependency): it reacts to `day-began` to roll its day and to the day's sunrise
for the temperature curve, so it never re-derives dates or the sun. Enabling `weather` without `calendar` is
an assembly refusal. Other packs may react to its Public facts (an umbrella seller, a sleepy rainy-day
routine, a slippery surface in bodies) without `weather` knowing them (ARC-26, ARC-28).

## 6.2 Configuration (`configure/weather.yaml`)

```yaml
# record-driven (the market-town default)
source: record
record:
  station: USW00023188                     # provenance only; the data is the attachment below
  data: data/weather/san-diego-usw00023188-2015-2024.csv
  first_year: 2015                         # the record year world year `calendar.epoch.year` maps to
fill: rules                                # gaps longer than 3 days are filled by the fitted rules below
rules: rules/san-diego.yaml                # also usable alone with `source: rules`
seed: 19                                   # for rule days and for hour placement; content, not --seed
```

```yaml
# rules/san-diego.yaml — WGEN-lite, fitted by tools/weather-fetch or authored by hand; all integers
months:                                    # 12 entries, January first
  - { p_wet_after_dry: 120, p_wet_after_wet: 380,          # per mille
      rain_tenth_mm: [10, 30, 80, 160, 400],               # quintile table of wet-day amounts
      tmax_dc: 196, tmin_dc: 98, t_noise_dc: 25, t_ar_permille: 600,
      fog_permille: 90, overcast_morning_permille: 150, wind_dms: 30 }
  # … eleven more
```

A plain rule table is the same file with `p_wet_after_dry == p_wet_after_wet`. The player's freedom
(requirement 1) is: choose `source: record` with any committed station file, or `source: rules` with any rules
file, per World Pack, through the settings of the world being created (S16 `mineworld create`), not by editing
code.

## 6.3 Determinism

- **Record days** are a pure function of `(data, first_year, world date)`; **rule days** of `(rules, seed,
  day index, previous day's wet state)`; **hour placement** (which hours of a wet day are wet, when fog lifts)
  of `(seed, day index)`. The generator is counter-based SplitMix64 keyed by `(seed, day index, draw index)`,
  the tree's existing idiom (§2.7). All arithmetic is integer; no float reaches weather.
- The Markov chain's state (yesterday wet or dry) lives in the `climate` Process state, folded from the
  `weather-day` fact; so a resumed world continues the same chain without replaying from day 0.
- **Mapping world dates to record dates.** `record_year = first_year + (world_year − epoch_year) mod N` where
  `N` is the number of whole years in the file; the record loops. **Leap days:** a world 29 February in a
  non-leap record year uses that record's 28 February; a record 29 February is used only by a world
  29 February. Month and day are always kept, so seasons stay aligned.
- **Missing values** (`-9999`, absent rows): a gap of ≤ 3 days copies the previous day's values; a longer gap
  takes rule days for its duration (`fill: rules`) or, with `fill: none`, is a refusal at assembly naming the
  dates. The fetch tool reports gaps when it writes the file, so the refusal is never a surprise.

## 6.4 The San Diego default data plan

| Item | Decision |
| --- | --- |
| Source | NOAA **GHCN-Daily**, station **USW00023188** (San Diego International Airport / Lindbergh Field), period of record 1939-07-01 → present (read 2026-10-08). |
| Years | **2015-01-01 → 2024-12-31** (10 whole years; includes the 2015–16 El Niño winter and two leap years, 2016 and 2020). QTW-5 offers 30 years. |
| Fields | TMAX, TMIN (0.1 °C), PRCP (0.1 mm), AWND (0.1 m/s), and the weather-type flags WT01, WT02 (fog), WT03 (thunder), WT13 (mist), WT14 (drizzle), WT16 (rain). The fetch tool reports per-flag population; an unpopulated flag is dropped and the fitted rules supply fog frequency instead. |
| Optional hourly layer (TW-f, QTW-6) | **GHCNh** for the same station and years, reduced to two integers per day: morning (06–11) and afternoon (12–18) mean sky cover in oktas, and fog hours. ISD-Lite rejected (superseded, no fog field). |
| Committed form | `worlds/market-town/data/weather/san-diego-usw00023188-2015-2024.csv`, one wide row per day: `date,tmax_dc,tmin_dc,prcp_tenth_mm,awnd_dms,fog,thunder,drizzle,rain` (+ `sky_am,sky_pm,fog_hours` after TW-f). *Estimate:* **~130 KB** (≈ 3,653 rows × ~36 B); the encoded configured fact ~40 KB (≈ 11 B/day). |
| Licence and provenance | CC0 / US public domain. A `NOTICE` beside the file: source URL, station, retrieval date, the fetch tool's version and command, "modified (reshaped and gap-reported) from NOAA GHCN-Daily; not endorsed by NOAA", and the requested citations: Menne, M.J., et al. (2012), *J. Atmos. Oceanic Technol.* 29, 897–910, doi:10.1175/JTECH-D-11-00103.1; and Menne et al. (2012), GHCN-Daily Version 3, NOAA NCDC, doi:10.7289/V5D21VHZ. DEP-8's table gains a row (DEP-31). |
| Fetch tool | `tools/weather-fetch` (Rust, offline, never run by `run`, CI or the server): downloads the station CSV from NCEI, reshapes, reports gaps and flag population, writes the CSV and NOTICE, and fits `rules/<name>.yaml` from it. Re-running it with the same arguments against the same upstream file is byte-identical. It is a developer tool. No HTTP client exists in the workspace today (audited: no `ureq`/`reqwest`); the tool's `--input FILE` mode reshapes a file the developer downloaded, and its fetch mode adds `ureq` to this tool crate only (DEP-31), never to a runtime crate (QTW-14). |
| Size limit | A station file over 1 MB is a `mineworld check` warning (keeps World Packs small; R-TW-4). |

## 6.5 How the attachment reaches the world (needs IL-a's seam to grow, QTW-7)

IL-a's SD-IA-5 is "one YAML file per key; no data attachments". Options:

| Option | Verdict |
| --- | --- |
| (a) Inline the 3,653 rows as a YAML list in `configure/weather.yaml` | Works with IL-a as frozen; a 200 KB YAML file is awkward to read and diff. Acceptable fallback. |
| **(b) A typed `data:` attachment**: a configuration key may name a file under the World Pack; the seam reads it, the pack's decoder parses it, and the **decoded, compact series is carried inside `weather-configured`** (so the world remains a fold of its facts and a changed CSV is caught by IL-a's drift check like any configuration change) | **Recommended.** A small, generic addition to the seam (IL-b or a TW PR touching `authoring`), reviewed by the IL lane. |
| (c) The pack reads the CSV at run time from disk | **Reject.** World behaviour would depend on a file outside the facts; a resumed world with a changed file would silently diverge. |

## 6.6 Facts, state and disclosure

| Event | Visibility | When | Payload |
| --- | --- | --- | --- |
| `weather-configured` | SystemInternal | genesis | source, decoded series or rules, seed, first_year |
| `weather-day` | SystemInternal | reacting to `day-began` | `WeatherDay { record_date: Option<date>, rule_day: bool, hours: [WeatherHour; 24] }` |
| `weather-changed` | Public | at each hour whose `condition` differs from the previous hour's | `condition`, `cloud_oktas`, `precipitation_tenth_mm_per_h` |

```text
WeatherHour { condition: Condition, cloud_oktas: u8, temperature_dc: i16, precipitation_tenth_mm: u16,
              wind_dms: u16, wind_from_deg: u16 }
Condition = clear | partly-cloudy | overcast | fog | drizzle | rain | heavy-rain | thunderstorm
```

- **Daily to hourly** (record and rule days alike, integer tables in the rules file): temperature follows a
  fixed 24-entry per-mille diurnal curve between TMIN at sunrise and TMAX at 15:00; a wet day's hours are
  placed as one or two seeded runs whose total length grows with PRCP (table); fog days fog from civil dawn to
  10:00; cloud from the hourly layer when present, else from the month's `overcast_morning_permille`.
- The `climate` Process wakes at each condition change hour and at midnight. A 30-day headless run adds ~30
  `weather-day` facts and typically a few `weather-changed` per day.
- **Disclosure** (presence `discloses`, on the observer's place): `weather.day` (the 24 hours) and
  `weather.now` (the current condition since the last change). Clients take temperature, wind and visual
  intensity from `weather.day` by their clock estimate. The record is the same in every place; clients draw
  precipitation only outdoors (a presentation decision, as indoors/outdoors is geometry).
- The Pack also declares `weather` for observing Persons' controllers: the paced controller may read it
  later (S10's lane); S19 changes no controller.

---

# 7. Design — the host clock: pause, scale, close, background, multiplayer

## 7.1 States

```text
running(scale)  ──pause──▶  paused  ──resume──▶  running(scale)
running(s₀)     ──scale s₁──▶ running(s₁)        paused ──scale s₁──▶ paused (applies on resume)
any             ──graceful stop──▶ (checkpoint, journal `Stopped`; the world does not age while down)
```

Every arrow is a host-journal record (§4.3) and closes a `HostClock` segment (§4.2).

## 7.2 What pause means (QTW-2, operator-material)

**Recommended: a full pause.** While paused: the clock does not move; no Process wakes; hosted controllers
are not consulted; client action requests are refused with a new refusal `paused` (nothing is half-done);
observers stay connected and keep receiving frames (nothing changes). Alternative: a "time stop" in which
players can still walk and talk at a frozen instant — rejected as the default because it lets many facts pile
up at one instant and contradicts "pause the town", but noted as a possible later host option.

## 7.3 Who may pause or change the scale

- **Control surface: S11-D's admin HTTP surface**, two routes added:
  `GET /admin/clock` → `{ at, time_scale, paused }`; `POST /admin/clock` with `{ "paused": bool }` and/or
  `{ "time_scale": 6 | 12 | 24 | n }` (integer 1…3600; the UI offers 6/12/24). Bearer token, constant-time,
  as the other admin routes.
- **I-4 is kept and clarified** (amendment text in §15): pausing and scaling change the host's *pacing* of the
  clock; they never set a component, emit a fact or move the clock to an instant. The adversarial check
  (S11-D criterion 1) extends: calling the clock routes leaves the save's revision and fact count unchanged
  (the journal is not a fact).
- **Single-player**: the local launcher starts the host with a freshly generated admin token and hands it to
  its own client (process argument or environment, never written to the save, I-5). The client's settings
  menu shows the time controls only when it holds a token.
- **Multiplayer**: only the host's operator or an admin-token holder sees enabled controls; every client sees
  the pause overlay and the current day length from the `clock` frame (below).
- **Earliest delivery.** If S11-D is not merged when TW-c is ready, TW-c lands the two routes with the bearer
  check as its own small PR in S11's lane, rebased by S11-D (QTW-3).

## 7.4 Telling clients: a `clock` server frame

`WorldSummary` gains `paused: bool` beside S11-B's `time_scale`. A new server frame `clock { at, time_scale,
paused }` is sent after `welcome` and on every pause, resume or scale change, so clients' HUD estimates (§8.2)
re-anchor at once rather than at the next observation. Added to `PROTOCOL.md` revision 2's landing table,
owned by TW-c (ARC-41: specified whole, landed incrementally). Refusal code `paused` likewise.

## 7.5 Close, background, defaults

- **Closing the game (single-player)**: the client asks the launcher to stop the host gracefully (the
  existing `Command::Shutdown` → checkpoint). Time does not pass while no host runs, so the town is saved
  and paused by construction (§2.3). A killed host loses at most the time since the last checkpoint
  (S11's existing guarantee).
- **"Keep the town running when I close the game"**: a launcher setting (default **off**). When on, closing
  the window disconnects the client and leaves the host serving with its hosted controllers — today's server
  mode. Re-opening the game reconnects (S11-B resume). This is a launcher preference, not a world setting.
- **Default scale** is content: `world.yaml` gains `hosting: { time_scale: 12 }` (read by the server, not by
  any pack; QTW-4). Precedence at start: `--time-scale` flag > the journal's last scale for a resumed world >
  `world.yaml hosting.time_scale` > 1. The S12 and S14 launchers pass nothing and get 12 from the World
  Pack; `--time-scale` stays for tests and operators.

---

# 8. Design — the clients (2D `clients/2d`, 3D `clients/3d-spike`; both Godot 4)

## 8.1 Settings menu (joins the shared client settings module of `overall.md` "Framework, not demo" item 4)

| Section | Item | Kind | Reaches the server? |
| --- | --- | --- | --- |
| Display → Clock | 12-hour / 24-hour (default: 12-hour for `en`, 24-hour for `zh-Hans`, until the user chooses; QTW-16) | presentation preference, persisted per user | **no** |
| World (shown only with a host token) | Pause / Resume; Day length: 4 h · 2 h · 1 h (6×, 12×, 24×) | **host commands** through `/admin/clock` (§7.3), not settings | yes, as a host command; never stored in the client's settings file |
| Launcher (single-player only) | Keep the town running when I close the game | launcher preference | no (it changes what the launcher does on close) |

Item 4 says settings "never reach the server". Pause and day length are therefore *host commands that the
operator wants reachable from the in-game settings menu* (2026-10-08), not settings; the amendment in §15.2
says so explicitly. The host (a client holding the host's or an admin token) sees them enabled. Every other
client sees the current day length and paused state read-only, or does not see the section at all.

## 8.2 The HUD date and time

- **Source.** The last `calendar.day` record and the last anchor `(at, time_scale, paused, wall_received)`
  from the latest observation or `clock` frame. Never the client's OS clock for the *date*; the wall clock is
  used only to advance the estimate between frames.
- **Estimate.** `at_est = at_anchor + (paused ? 0 : floor((wall_now − wall_received) × time_scale))`, shown at
  minute resolution. A newer frame re-anchors; if the estimate is ahead of the new `at`, the displayed minute
  holds until the world catches up (the HUD never runs backwards). The time of day is `at_est − day_start`;
  past 24:00 without a new `day-began` the client rolls the date itself with the same civil algorithm (shared
  GDScript, one function) until the record arrives.
- **Without `calendar`** the HUD shows `Day N, HH:MM` from `at` as `step-13-client-2d.md` item 8 already
  specifies (unchanged behaviour).
- **Formats are translation content** (the settings planning lane chooses `.po` or CSV; these are the
  entries, not code):

| Key | `en` | `zh-Hans` |
| --- | --- | --- |
| `hud.date` | `{weekday_short} {day} {month_short} {year}` | `{year}年{month}月{day}日 {weekday_long}` |
| `hud.time.24h` | `{hour24}:{minute2}` | `{hour24}:{minute2}` |
| `hud.time.12h` | `{hour12}:{minute2} {ampm}` | `{ampm} {hour12}:{minute2}` |
| `hud.ampm.am` / `.pm` | `AM` / `PM` | `上午` / `下午` |
| `hud.datetime` | `{date}, {time}` | `{date} {time}` |
| `hud.paused` | `Paused` | `已暂停` |

  Examples (8 October 2026 is a **Thursday**): `Thu 8 Oct 2026, 7:42 PM`;
  `2026年10月8日 星期四 19:42`; `2026年10月8日 星期四 下午 7:42`.
  Weekday and month names are translation entries too (`hud.weekday.short.0` …).
- **Live.** The HUD updates every frame from the estimate, shows `Paused` while paused, and changes rate the
  moment a `clock` frame reports a new scale.

## 8.3 Day and night

- **Shared** (one GDScript module beside the settings module, e.g. `clients/shared/world_time/`; not the
  protocol module, which S11 owns): the clock estimate, sun-track interpolation (linear between the 97
  samples, azimuth unwrapped across 360°), the light phase, and the weather intent mapping (§3.4).
- **3D** (`clients/3d-spike`): the slice's key light `rotation_degrees = (−elevation, azimuth_to_godot)` from
  the interpolated sun; light energy and colour from an elevation-indexed presentation table whose
  golden-hour band reproduces ARC-13's constants (`SUN_ELEVATION −19.3`, `SUN_AZIMUTH −48`, `SUN_ENERGY 4.4`,
  `SUN_WARM`), so the approved look is what golden hour looks like, not a lost look; night uses a moon/sky fill
  only. Sky3D in TW-e (§11), driven with its own clock and astronomy off; the first renderer PR keeps the
  existing `ProceduralSkyMaterial`/`PhysicalSkyMaterial`. **GI:** `VoxelGI` is baked and does not follow a
  moving sun; TW-e measures VOXEL against SDFGI under the moving sun and records the choice (R-TW-6).
- **2D** (`clients/2d`): a `CanvasModulate` tint from an elevation-indexed table; lit windows at night are a
  later art item.
- **Without `calendar`** both clients keep today's fixed lighting (INV-TW-6).

## 8.4 Weather visuals (Presentation Pack content)

- Shared mapping: `Condition` and the hour's numbers → `{ rain_intensity, fog_density, cloud_cover, wetness,
  wind }` (0..1 floats in the client only), a table in the Presentation Pack.
- 3D: camera-following `GPUParticles3D` rain with a height-field collider; fog through the environment's
  depth fog / volumetric fog density; cloud cover dims the sun's energy and, with Sky3D, sets its cloud
  coverage; wet surfaces lower roughness. Rain only outside interior volumes.
- 2D: a screen-space rain overlay, a fog overlay, and the tint.
- No weather visual feeds back to the server.

## 8.5 Parity

Extends the "One world, two views" parity test: both clients, run headless against one recorded observation
stream with the same settings, print the same HUD string at the same frames (both languages, both clock
modes), the same light phase, and the same weather intent. Because both use the shared module, a failure
means one client bypassed it.

---

# 9. Feasibility at 24× (acceptance criterion fixed before measuring)

## 9.1 Analysis from L-12

L-12 measured, headlessly at pace 900 s: ~8 m per world hour, i.e. ~2 m per consult (4 consults/hour),
median journey 2 world hours (~8 consults), one journey in ten nearly 4 hours (~16 consults). The paced
controller's choice per consult does not depend on its pace except for its answering window, so **a journey
costs a number of consults, not a number of seconds.** Under §4.4 a hosted consult happens every `c` wall
seconds, so a journey takes `k × c` wall seconds and `k × c × s` world seconds.

| Scale | Shortest routine part (4 world h) | P90 journey at `c = 5 s` (S11-B's default pace, read as wall s) | P90 / part |
| --- | --- | --- | --- |
| 6× | 40 wall min | 16 × 5 = 80 wall s = 8 world min | 3 % |
| 12× | 20 wall min | 80 wall s = 16 world min | 7 % |
| 24× | 10 wall min | 80 wall s = 32 world min | 13 % |

And for a 1-world-hour shift at 24× (150 wall s): a median employee (8 consults, 40 s = 16 world min) is
present at `shift-started` only if they set off ≥ 16 world minutes early. Routines in `market-town` start
travel at the routine part's start, so they arrive *during* the part. Hence the operator's concern is real
for parts or shifts shorter than ~2 world hours, not for today's ≥ 4 h parts.

## 9.2 The criterion (fixed now, measured in TW-c's checkpoint)

**FX-24.** In `market-town`, hosted at 24× with the default cadence, measured headlessly by the equivalent
`run` pace `c × s` (= 120 world s for `c = 5`, exact under §4.4) over 7 world days, seed 1:

1. ≥ 90 % of agenda journeys end at the agenda place within **25 %** of the agenda part they serve, and every
   journey within 50 %;
2. every scheduled shift's `shift-started` records the employee present for ≥ 80 % of shifts;
3. the same holds at 12× and 6× (it must, if it holds at 24×; checked to catch a cadence bug).

If FX-24 fails, the remedy is **content** (longer parts, earlier departure entries in routines, a shorter
town) or an **operator decision** (drop the 1 h option, or accept late arrivals at 24×) — never a controller
change that reads the scale (INV-TW-2). QTW-1 asks the operator now, before measuring.

A note for S10's lane, not a remedy: a `run --pace` flag (the measurement needs one) is added in TW-c as a
test-only knob on the CLI; `run`'s default pace stays 900.

---

# 10. Invariants (proposed; frozen only by the primary session or the operator)

| Id | Invariant |
| --- | --- |
| INV-TW-1 | **Byte identity.** A world that enables neither `calendar` nor `weather` produces byte-identical `run` output, saves and fact logs before and after every S19 PR (checked on `social-cafe`, `market-town` as it is today, `bodies-yard`). |
| INV-TW-2 | **No world rule reads pacing.** No System Pack, controller decision or fact depends on the time scale or pause. Only the host and clients see them. |
| INV-TW-3 | **Kernel and contracts unchanged.** `WorldTime`, `WorldClock`, `advance`, `Process`, `WorldRead`, `Observation` and presence's `PerceptionProvider` are not edited by S19. |
| INV-TW-4 | **One midnight.** `calendar`'s local midnight is every multiple of 86 400 s from instant 0, the same as `schedule`'s `TimeOfDay`. |
| INV-TW-5 | **Integers on the wire and in facts.** Sun angles in millidegrees, times in world seconds, weather in fixed-point integers; no float in a fact, a component, a disclosure or the protocol. |
| INV-TW-6 | **Clients only render.** Neither client computes the sun, the date or the weather; both read disclosure through the shared module. A client without `calendar` disclosure shows today's fixed lighting and `Day N` clock. |
| INV-TW-7 | **No runtime fetch.** No crate reachable from `run`, the server or a client opens a network connection for weather or sun data; the fetch tool is the only one. |
| INV-TW-8 | **Monotonic host clock.** Pause, resume and scale changes never move `now` backwards or jump it forwards; a resumed world starts at its saved `now`. |
| INV-TW-9 | **Pacing is recorded, not fact.** Every pacing change is in the host journal; none is in the fact log or the revision count. |
| INV-TW-10 | **Weather depends on calendar, never the reverse;** removing `weather` leaves calendar facts byte-identical. |

---

# 11. PR split

## 11.1 Order and parallelism

```text
IL-a (configure seam, in implementation) ──▶ TW-a calendar pack ──▶ TW-b weather pack (rules) ──▶ TW-d record data + fetch tool
S11-B (in implementation) ─────────────────▶ TW-c host clock: pause, scale, journal, admin routes, clock frame, FX-24
S12 13a + S14 16a + settings-menu PR + TW-a + TW-c ──▶ TW-e clients: HUD date/time, 12h/24h, sun, World controls
TW-b + TW-e ──▶ TW-f weather visuals, Sky3D, GI under a moving sun      (TW-g optional: GHCNh hourly layer, after TW-d)
```

TW-a and TW-c touch disjoint crates (`systems/calendar`, `worlds/*` vs `server`, `persistence`) and run in
parallel in separate worktrees. TW-e and TW-f are client PRs in the S12/S14 lanes' files and are scheduled with
those lanes, never in parallel with another PR editing the same client.

## 11.2 The PRs

| PR | Scope | Integration checkpoint | Adversarial criteria (fixed before measuring) |
| --- | --- | --- | --- |
| **TW-a** | `systems/calendar` (new System Pack): configuration through IL-a's seam, civil date, sun via `solar-positioning` (`libm`), `day-began` / `daylight-changed` facts, `calendar` Process, disclosure on the place; DEP-30; `market-town` opts in (own commit, new goldens). Full design §16. | CP-TW-a: `mineworld run worlds/market-town --headless --seed 1 --days 7 --save D` → facts show 7 `day-began` with dates 2026-10-08…14 and weekday Thu…Wed; San Diego sunrise and sunset on 2026-10-08 within ±2 min of the NOAA Solar Calculator's values, which the PR records (with the URL and date read) before running, converted to the fixed −08:00 offset; restart from `D` continues on day 8 identically to an uninterrupted 8-day run. | (1) INV-TW-1 on `social-cafe` and `bodies-yard` (and `market-town` before its opt-in commit). (2) Golden sun values at San Diego and at 78° N (polar night: `sunrise = None`) across a `libm` build on macOS and Linux CI — equal integers. (3) Mutation: drop the `libm` feature — the golden test must still pass on the CI host *or* fail loudly; recorded either way (it establishes the guard). (4) Mutation: shift midnight by one second — INV-TW-4's test must fail. (5) An invalid `configure/calendar.yaml` (lat 91) is refused at assembly naming file and key. |
| **TW-b** | `systems/weather` (new System Pack), `source: rules` only: WGEN-lite generator (integer, SplitMix64), daily → hourly derivation, `weather-day` / `weather-changed` facts, `climate` Process, disclosure; dependency on `calendar`; `rules/san-diego.yaml` hand-authored provisional table; `market-town` opts in. | CP-TW-b: `run market-town --days 30 --seed 1` twice → identical bytes; the 30 days contain wet and dry spells; a resumed run equals an uninterrupted one. | (1) INV-TW-1 and INV-TW-10 (calendar facts byte-identical with and without `weather`). (2) Over 3,650 rule days the wet-day frequency per month is within ±3 percentage points of the table's stationary probability `p_wd / (1 − p_ww + p_wd)` — fixed before running. (3) Mutation: make the chain ignore yesterday — criterion 2's spell-length check (mean wet spell ≥ 1/(1 − p_ww) − 0.2) must fail. (4) Enabling `weather` without `calendar` is refused at assembly. |
| **TW-c** | `server`: `HostClock` segments; pause/resume/scale; `paused` refusal; host journal (`persistence` additive table); `/admin/clock` routes (or on S11-D); `clock` frame and `WorldSummary.paused`; cadence in wall seconds with live rescheduling; `world.yaml hosting.time_scale`; `run --pace` test knob; FX-24 measurement. | CP-TW-c: through the binary, `market-town --town --save D --admin-token T`: pause → `/status` `at` frozen for 10 wall s and a client move refused `paused`; resume at 24× → `at` advances 240 ± 24 s in 10 wall s; scale 6× → 60 ± 6; graceful stop and restart → `at` resumes from the saved instant, scale from the journal. FX-24 (§9.2) measured and recorded. | (1) INV-TW-8: a property test over random pause/resume/scale sequences — `now` never decreases and never jumps. (2) INV-TW-9: the clock routes leave the save's revision and fact count unchanged (S11-D criterion 1 extended). (3) Replay: re-running the hosted paced controllers from the journal and the recorded facts reproduces the same requests in a scripted hosted session with two scale changes and a pause. (4) Mutation: compute `next_consult` with the old scale after a change — criterion 3 must fail. (5) Without a token the clock routes answer 404; wrong token 401 no sooner than 500 ms. (6) INV-TW-1 for `run` (no scale reaches it). |
| **TW-d** | `tools/weather-fetch` (`--input` and fetch modes, rules fitting); `worlds/market-town/data/weather/` CSV + NOTICE (2015–2024, GHCN-Daily USW00023188); the seam's `data:` attachment (QTW-7, reviewed by the IL lane); `source: record` in `weather`; DEP-31 and the DEP-8 row; `market-town` switches its default to the record. | CP-TW-d: `run market-town --days 365` shows San Diego's seasonality: winter months wetter than summer; ≥ 1 fog day in May–July if WT flags are populated; with `first_year: 2015`, the world's 2026-10-08 replays the record's 2015-10-08 and the world's 2027-10-08 the record's 2016-10-08 (§6.3) — exact TMAX/TMIN/PRCP asserted from those CSV rows. | (1) Re-running the tool on the same input is byte-identical. (2) Leap mapping: a world 2027-02-29 does not exist; a world 2028-02-29 against a non-leap record year uses that year's 02-28 — asserted. (3) Mutation: edit one CSV value after assembly — IL-a's drift check reports `weather` changed. (4) INV-TW-7: no runtime crate depends on the tool or on an HTTP client (`cargo tree` check). (5) The fitted rules reproduce the record's monthly wet-day frequency within ±3 points. |
| **TW-e** | Shared client module `world_time` (estimate, interpolation, formatting, condition → intent); HUD date/time in 2D and 3D; 12h/24h preference in the settings module; translation entries (`en`, `zh-Hans`); 3D key light from the sun, 2D tint; the World section of the settings menu (pause, day length) via `/admin/clock`; launcher "keep running" and graceful close. | CP-TW-e: a single-player session from each launcher: HUD shows `Thu 8 Oct 2026, 12:00 AM`… advancing ~12 world minutes per wall minute; pause freezes it and shows `Paused`; 1 h day length makes it advance 24 per minute; closing the window stops the host and reopening resumes the same minute; zh-Hans shows `2026年10月8日 星期四 上午 12:00`. | (1) Parity (§8.5): both clients' HUD strings and light phase identical over a recorded stream, both languages, both clock modes. (2) The HUD never runs backwards across a re-anchor where the estimate was ahead. (3) A client given no `calendar` disclosure renders today's fixed light and `Day N` (INV-TW-6). (4) The 12h/24h preference appears in no frame sent to the server (frame capture). (5) Mutation: compute the date from the OS clock — criterion 1 with an epoch ≠ today must fail. |
| **TW-f** | 3D: Sky3D driven externally (clock and astronomy off), night fill, rain particles with height-field collider, fog and cloud dimming, wet surfaces; GI measured VOXEL vs SDFGI under a moving sun; 2D: rain and fog overlays. Presentation Pack content only. | CP-TW-f: the 3D slice at 24× through one world day with a scripted rain hour: dawn, noon, golden hour (matching ARC-13's reference capture within the visual-fidelity tolerance), night, rain visible outdoors and absent indoors. | (1) No GDScript in TW-f reads lat/long or computes the sun (grep + review). (2) Frame time at the reference scene stays within VISUAL_FIDELITY's budget with rain on. (3) Golden-hour capture compared with ARC-13's reference — a regression is a FAIL, not a note. |
| TW-g (optional) | GHCNh hourly layer → `sky_am`, `sky_pm`, `fog_hours` columns. | — | the same as TW-d's 1, 3, 4. |

Each PR, when designed, gets its own `pr-TW-x-*.md` with a commit plan and execution contract; this table
fixes scope, checkpoints and adversarial criteria only.

---

# 12. Cross-lane impacts

| Lane | Impact | What S19 needs from it / gives it |
| --- | --- | --- |
| **S11-B** (in implementation) | `--time-scale` lands as a constant; `--pace` in world seconds. | TW-c builds on it: segments, live change, `--pace` → wall cadence (QTW-13). No change requested inside S11-B; its launchers' default comes from `world.yaml` later (TW-c). |
| **S11-D** (planned) | Admin surface. | Two `/admin/clock` routes and the I-4 clarification (§7.3, §15). Either S11-D includes them or TW-c lands them first (QTW-3). |
| **S11-C** | Delta stream. | Calendar and weather records change only at facts, so deltas stay small; the `clock` frame joins the landing table. |
| **IL-a** (in implementation) | `configure:` seam. | TW-a/TW-b are its first non-interaction users (configured facts, `Rejection`). No change requested in IL-a. |
| **IL-b** | The next IL PR. | The `data:` attachment (QTW-7) and making `CONVERSATION_GAP` / `INVITATION_LIFETIME` configurable (QTW-15). |
| **S12** (2D client) | HUD item 8, settings. | TW-e refines item 8 (date and time), adds the tint, uses the shared `world_time` module; scheduled after 13a. |
| **S14** (3D) | Slice lighting rig (ARC-13). | TW-e replaces fixed sun constants by a function whose golden-hour band reproduces them; TW-f adds Sky3D and re-checks GI. After 16a. |
| **Settings-menu PR** (after S12 13a and S14 16a) | One shared client settings module, translations. | 12h/24h preference and the HUD translation entries; the World section is host commands, an explicit amendment (§15). |
| **S16** (packages) | `mineworld create`, pack catalog. | `calendar` and `weather` are System Packs in the catalog; `create` asks for location and weather source (template prompts, later). |
| **S10** (NPC behaviour) | Paced controller, routines. | FX-24 (§9) — if it fails, a content change in routines; the paced controller is not changed. Later (not S19): controllers may read `weather.now` / `calendar.light` (e.g. go home at dusk) — the facts and records are there for it. |

---

# 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-TW-1 | FX-24 fails: NPCs are late at 24×. | Criterion and remedies fixed before measuring (§9.2); QTW-1 asks the operator now. |
| R-TW-2 | Float drift across platforms in the sun model moves a quantization boundary. | `libm` only, integers out, golden values on two OSes in CI; a moved boundary is a FAIL to investigate, not a re-golden. |
| R-TW-3 | `solar-positioning` is pre-1.0 and its API churns. | One private seam (§3.1); NOAA fallback ~100 lines; pin the exact version. |
| R-TW-4 | Station data bloats World Packs or the genesis fact. | 10 years ≈ 130 KB CSV / ~40 KB fact; a > 1 MB warning; 30 years only if QTW-5 says so. |
| R-TW-5 | GHCN-Daily fog/weather-type flags sparsely populated for USW00023188. | The tool reports population; fog falls back to the fitted rule frequency; GHCNh layer (TW-g). |
| R-TW-6 | `VoxelGI` (baked) looks wrong under a moving sun; SDFGI costs frame time. | TW-f measures both against VISUAL_FIDELITY's budget and records the choice; golden-hour reference capture guards ARC-13's look. |
| R-TW-7 | Humans at 12×–24× find conversation and invitation windows (world-time constants) too short. | QTW-15: configurable content in IL-b; headless defaults unchanged. |
| R-TW-8 | The settings-menu rule "never reaches the server" is read as forbidding the World section. | Explicit amendment (§15): host commands, not settings. |
| R-TW-9 | Fixed UTC offset (no DST) shows summer sunrise an hour off from a real San Diego clock. | QTW-9; DST rules are an additive calendar option later. |
| R-TW-10 | ISD/NCEI service changes move URLs again. | The data is committed; the tool records the URL and date; nothing at run time depends on NCEI. |

---

# 14. Questions (QTW-n). **[OPERATOR]** marks operator-material ones; the rest the primary session may rule.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QTW-1 [OPERATOR]** | At a 1 h day (24×), if FX-24 (§9.2) fails, which remedy? (a) content: longer routine parts and earlier departures; (b) drop the 1 h option; (c) accept late arrivals at 24× as the price of speed. Asked before measuring. | (a), with (c) as the fallback for shifts under 2 world hours. Never a controller hack. |
| **QTW-2 [OPERATOR]** | Pause semantics: full pause (nothing moves, requests refused `paused`) or "time stop" (players may still walk and talk at a frozen instant)? | Full pause (§7.2). |
| QTW-3 | Clock routes: inside S11-D, or a small TW-c route PR first in S11's lane? | Inside S11-D if it is designed before TW-c starts; otherwise TW-c lands them, S11-D rebases. |
| QTW-4 | Where the default scale lives: `world.yaml hosting.time_scale` (server reads it) vs a `configure/` key of `calendar` (a pack would hold host pacing). | `world.yaml hosting:` — pacing is not a pack's (INV-TW-2). |
| QTW-5 [OPERATOR] | How many record years ship by default: 10 (2015–2024, ~130 KB) or 30 (1995–2024, ~400 KB, the climatological normal)? | 10; a World Pack may ship more. |
| QTW-6 | Add the GHCNh hourly layer (sky cover, fog hours) now (TW-g) or later? | Later, after TW-d shows whether GHCN-Daily's WT flags are enough. |
| QTW-7 | How a data file reaches a pack: (b) a typed `data:` attachment on IL's seam, carried decoded in the configured fact; vs (a) inline YAML. | (b), in IL-b or TW-d with the IL lane reviewing. |
| QTW-8 | A world enabling `calendar`/`weather` without its configure file: silent (IL-a's rule) or a `mineworld check` warning? | Warning; never a refusal (keeps IL-a's rule). |
| QTW-9 [OPERATOR] | Daylight saving time: fixed standard offset (v1, simpler, summer sunrise one hour "early" against a real San Diego clock) or US DST rules (needs a rule table; `chrono-tz` or our own)? | Fixed offset in v1; DST as an additive calendar option later. |
| QTW-10 | Worlds placed where no station exists: allow ERA5 (CC BY 4.0) in the fetch tool? | Not now; rules suffice. Record ERA5 as the known path. |
| QTW-11 | Moon phase and moonlight: compute and disclose in `calendar`, or leave Sky3D's moon decorative? | Disclose moon elevation/phase in a later calendar PR; until then Sky3D's moon off (a client-computed moon would be a second astronomy). |
| QTW-12 | Regional weather (different weather per region of a large world)? | Not in S19; additive later (one `climate` Process per region). |
| **QTW-13 [OPERATOR]** | Hosted controllers' consult cadence in **wall** seconds (reverses QS11B-4's "a pace is about a life" for cadence; rename `--pace` → `--cadence`). | Yes: walking is embodied (requirement 2). With scale 1 nothing changes. |
| QTW-14 | `tools/weather-fetch` fetch mode adds `ureq` to the tool crate only, or `--input` mode only (developer downloads by hand)? | Both; `ureq` confined to the tool crate, recorded in DEP-31. |
| QTW-15 | Conversation gap (300 s) and invitation lifetime (1800 s) are world-time constants answered by humans; at 12–24× they are 12–150 wall s. Make them configurable content (IL-b) and set larger values in interactive World Packs? | Yes, via IL-b; headless defaults unchanged (INV-TW-1). |
| QTW-16 | Default clock mode before the user chooses: per language (12 h for `en`, 24 h for `zh-Hans`) or one global default? | Per language, as above. |
| QTW-17 | Epoch date for `market-town`: a fixed date (2026-10-08, the day of the requirement) or "today" at world creation (`mineworld create` writes it)? | Fixed in the shipped World Pack (determinism); `create` writes the creation date for new worlds. |

## 14.1 Rulings, 2026-10-08 (binding; relayed by the coordinator)

**By the operator:**

| Id | Ruling |
| --- | --- |
| QTW-1 | Content first: longer routine parts and earlier departures. Accepting late arrivals for short shifts is the fallback. Never a controller change that reads the scale. |
| QTW-2 | Full pause (§7.2): nothing moves, and client requests are refused `paused`. |
| QTW-5 | 10 record years (2015–2024). |
| QTW-9 | A fixed UTC offset, with no DST in v1. |
| QTW-13 | Hosted consult cadence is in **wall seconds**. This reverses QS11B-4 for consult cadence. **The S11-B lane must take this ruling**: `--pace` becomes a wall-second cadence (renaming it `--cadence` is part of the ruling as recommended), and the S11-B design is amended accordingly. |

**By the primary session, each as recommended above:** QTW-3, QTW-4, QTW-6, QTW-7, QTW-8, QTW-10, QTW-11,
QTW-12, QTW-14, QTW-15, QTW-16 and QTW-17.

No question in §14 remains open.

---

# 15. Proposed decision records and amendments (text for the primary session to apply)

## 15.1 `docs/DECISIONS.md` (placeholder ids)

- **ARC-67 — Two time domains.** `WorldTime` is calendar time and the only world clock. Embodied time is
  not a world quantity; it is the wall-clock cadence of embodied inputs. Time scale and pause are host pacing
  (`HostClock` segments), recorded in a host journal, never facts; no System Pack reads them. Alternatives
  B–D of §4.2 rejected with reasons. Consequence: hosted consult cadence is in wall seconds (QTW-13).
- **ARC-68 — Calendar and weather are System Packs.** `calendar` owns the civil date and sun; `weather`
  depends on it and owns the weather; both are configured through ARC-61's seam, emit Public facts at day,
  light and condition changes, and disclose on the observer's place. Kernel, contracts and presence
  unchanged. Rejected: `discloses_at` on presence; client-side astronomy.
- **ARC-69 — Pause and scale are host commands.** `/admin/clock`, the `clock` frame, `WorldSummary.paused`,
  refusal `paused`. Clarifies I-4.
- **DEP-30 — Solar position: adopt `solar-positioning` (MIT) with `libm`, behind a private seam; NOAA
  equations as the fallback. Civil date arithmetic: build (two public-domain integer functions) rather than
  adopt `chrono`/`time`.** Comparison: §3.1.
- **DEP-31 — Weather data: NOAA GHCN-Daily (CC0) as the default record; GHCNh as the optional hourly
  layer; Meteostat, Open-Meteo (free API non-commercial), ERA5 (fallback only), LARS-WG (non-commercial)
  rejected; WGEN-lite built from the published algorithm. `ureq` confined to `tools/weather-fetch`.**
  Comparison: §3.2–§3.3.
- **DEP-8 table rows**: "NOAA GHCN-Daily, station USW00023188 — CC0 / US public domain — attribution and
  citation in NOTICE, no implied endorsement, modified data so labelled"; Sky3D's existing row gains "driven
  externally; star map not shipped unless credited".

## 15.2 `overall.md`

- Add **S19** to "Parallel build-out" with §11's PR table and dependencies (IL-a, S11-B, S11-D, S12 13a,
  S14 16a, the settings-menu PR).
- "Framework, not demo" item 4 (settings): add — "**Pause and day length are host commands, reachable from
  the in-game settings menu** (operator, 2026-10-08). They are not settings. The menu sends them to the
  host's admin surface (`/admin/clock`), and they are never stored in the client's settings file, so the
  rule that settings never reach the server is unchanged. The menu shows them **enabled to the host**, that
  is, a client holding the host's or an admin token. To every other client it shows them **read-only**
  (current day length and paused state), or hides them; which of the two is a presentation choice of the
  settings-menu PR. The 12h/24h clock is an ordinary presentation setting."
- "One world, two views": add the HUD date and time, light phase and weather intent to the parity test.
- S11: note that `--pace` becomes a wall-second cadence (QTW-13, ruled by the operator 2026-10-08; it
  reverses QS11B-4, and the S11-B lane takes it), and that S11-D gains `/admin/clock` (QTW-3).
- §7 living-world gaps: add FX-24's result once measured; L-12 gains "hosted cadence is wall-time (S19)".

## 15.3 `step-12-server.md`

- §4.9 I-4: "There is no route that sets a component, moves a Person, emits a fact, enables a system or
  **moves the clock to an instant**; pausing and changing the time scale change the host's pacing and are
  recorded in the host journal, not in the fact log (S19 ARC-69)."

## 15.4 `docs/CORE_CONCEPTS.md`

- In the time section: "calendar time" (= `WorldTime`), "embodied time" (not a world quantity), "time scale"
  and "paused" (host pacing), as defined in §4.1.

---

# 16. TW-a — the `calendar` System Pack

**`DESIGN FROZEN 2026-10-08 (primary session; operator rulings QTW-1/2/5/9/13 of the same date)`**

Frozen means the scope (§16.1), the design decisions (§16.3), the acceptance (§16.5), the freeze review
amendments (§16.8) and the execution contract (§16.7) are frozen. Progress, evidence, audit findings and
bounded corrections stay writable. This section is TW-a's single PR design authority and its ledger. If the
execution session moves it into a separate `pr-TW-a-calendar.md`, it moves it whole and leaves a pointer
here; the two never exist side by side. Branch `mvp0/pr-tw-a-calendar`, from `main` after IL-a (#80) merges.

## 16.1 Goal and non-goals

Goal: a world can say where and when it is, and every observer sees the same date, day and sun. Non-goals:
weather (TW-b), pacing (TW-c), any client change (TW-e), DST (QTW-9), moon (QTW-11).

## 16.2 Audit anchors (to re-verify at freeze against `main`)

- Pack shape: `systems/employment` (`lib.rs` header table, `process.rs` `ProcessKind`, `system.rs`
  `react`/`wake`, `world.at()`, `start_process`, `reschedule_process`, `set_process_state` in
  `kernel/src/view.rs` l. 332–392), `PerceptionProvider::discloses` (`systems/presence/src/interaction.rs`
  l. 118).
- Registry: `systems/installed/src/lib.rs` (one line per pack).
- Configuration: IL-a's `PackConfiguration` (`authoring/src/configuration.rs`: `FACTS`, `seed`) and
  `configures!()` (`sdk/rust/src/pack.rs`).
- Midnight convention: `systems/schedule/src/time.rs` `DAY`, `TimeOfDay::of`.

## 16.3 Design decisions (SD-TW-a-n)

| Id | Decision |
| --- | --- |
| SD-TW-a-1 | New crate `systems/calendar` (`mineworld-calendar`), registered in `systems/installed` as `Calendar`. Depends on `mineworld-{contracts,kernel,authoring,presence,sdk}`, `serde`, `serde_json`, `thiserror`, and `solar-positioning = { version = "=0.7.0", default-features = false, features = ["libm"] }` (exact pin, R-TW-3). Not on `schedule`: it re-states the 86 400 s day as its own constant and INV-TW-4's test pins the agreement. |
| SD-TW-a-2 | Configuration `CalendarConfiguration { epoch: CalendarDate, utc_offset: UtcOffsetSeconds, latitude: MicroDegrees, longitude: MicroDegrees }` decoded from `configure/calendar.yaml` (§5.2) with refusals for out-of-range values. `FACTS = [calendar-configured]`. `seed` emits one `calendar-configured` (SystemInternal, no subjects) at genesis. |
| SD-TW-a-3 | `react(calendar-configured)`: computes day 0's `CalendarDay`, emits `day-began` for it, starts one `CalendarProcess` (`ProcessKind`, type `calendar`, no place, no participants, uninterruptible) with state `{ configuration, day, next: NextEvent }`, due at the first of the day's light events after `at` or the next midnight. |
| SD-TW-a-4 | `wake`: if the due event is midnight, compute the new `CalendarDay` (date + 1 via civil-from-days), set state, emit `day-began`; otherwise emit `daylight-changed { phase }`. Reschedule to the next event. Exactly one wake per event; a day with polar `None` events has fewer. |
| SD-TW-a-5 | `react(day-began)` / `react(daylight-changed)`: fold into the Process state via `set_process_state` (the state is the fold of facts, so a snapshot and a replay agree). |
| SD-TW-a-6 | `sun.rs`: the only file that imports `solar-positioning`. `fn sun_at(lat, lon, utc_seconds) -> SunSample` and `fn day_events(lat, lon, day_start_utc) -> DayEvents`, both returning integers (millidegrees, whole seconds, round half away from zero). UTC instant = `day_start + s − utc_offset`; the pack converts world date + offset to a Julian date numerically (no `chrono`). |
| SD-TW-a-7 | `civil.rs`: `days_from_civil`, `civil_from_days`, `weekday` (Hinnant, public domain), with a doc comment citing the source; years 1901–2099 enforced at configuration. |
| SD-TW-a-8 | Disclosure: for `subject` == the observer's current place (read from presence), two records `calendar.day` (`CalendarDay`) and `calendar.light` (`{ phase }`); nothing for any other subject. Component types declared by the pack so presence keeps them. |
| SD-TW-a-9 | Visibility: `day-began` and `daylight-changed` are `Public` (CORE_CONCEPTS: "a change of season"); `calendar-configured` SystemInternal. |
| SD-TW-a-10 | `worlds/market-town` opts in with `configure/calendar.yaml` (§5.2) and `calendar` in its systems list, in the PR's last commit, with regenerated goldens; `social-cafe` and `bodies-yard` unchanged (INV-TW-1). |
| SD-TW-a-11 | In the PR's first commit (C1, spec before code), `docs/DECISIONS.md` gains **ARC-67** (the two time domains, with the time terms of §15.4 added to `CORE_CONCEPTS.md`) and **DEP-30**. ARC-68, ARC-69 and DEP-31 land with the PRs they govern (ARC-68 with TW-b, ARC-69 with TW-c, DEP-31 with TW-b/TW-d). |

## 16.4 Commit plan

| # | Commit | Implementation | Deterministic validation | LLM logic review |
| --- | --- | --- | --- | --- |
| C1 | `docs: ARC-67 two time domains, DEP-30 solar position and civil dates; calendar pack spec` | DECISIONS entries ARC-67 and DEP-30; `CORE_CONCEPTS.md` time terms (§15.4); `systems/calendar/README.md` (short) and the pack's spec header | `check_doc_headings.py`, `check_decision_ids.py` | terminology against CORE_CONCEPTS; no synonym for `Process`/`Event` |
| C2 | `calendar: civil dates and the sun, in integers` | `civil.rs`, `sun.rs`, crate skeleton, dependency pin | golden tests: Hinnant round-trip over 1901–2099 (every day); weekday of 2026-10-08 = Thursday; the pinned-integer `SunSample` test of §16.8 (a); San Diego 2026-10-08 sunrise/sunset/noon vs NOAA values recorded in the test with URL and date; 78° N polar night; 0°/0° equinox | quantization rounding; Julian date conversion; azimuth convention (north = 0, clockwise) |
| C3 | `calendar: configuration, facts, the calendar process, disclosure` | `configuration.rs`, `event.rs`, `process.rs`, `system.rs`, registry line | integration: a test world with calendar, `run` 7 days → 7 `day-began`, light events in order, restart mid-day equals uninterrupted; refusal tests; INV-TW-4 test (midnight agrees with `schedule::TimeOfDay`) and its mutation; the no-place observer test of §16.8 (c) | single ownership; no read of host state; Process state is a fold of facts |
| C4 | `worlds: market-town lives in San Diego` | `configure/calendar.yaml`, systems list, goldens | CP-TW-a; INV-TW-1: the `social-cafe`, `bodies-yard` and `long_run` digests identical to `main`'s; the new `market-town` digest recorded in the ledger as the baseline (§16.8 (b)) | content only; no code |
| C5 | `docs(plan): TW-a evidence` | ledger | full gate: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` (workspace) | — |

## 16.5 Acceptance (fixed before measuring)

CP-TW-a and TW-a's adversarial criteria 1–5 (§11.2), plus: the observation of a Person in `market-town`'s
café at 12:00 on day 0 carries `calendar.day` with date 2026-10-08, weekday 3 (Monday = 0), and a sun track
whose 12:00 sample has elevation within ±0.5° of NOAA's value for that instant.

## 16.6 Risks specific to TW-a

- IL-a's final API differs from its branch (`seed` signature): C3 adapts; a change to IL-a's contract is
  not TW-a's to make (stop and report).
- `solar-positioning`'s `libm` build differs from its `std` build at a rounding boundary: the golden tests
  are written against the `libm` build only.

## 16.7 Execution contract (frozen 2026-10-08)

| Item | Contract |
| --- | --- |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-tw-a`, held by one session only (`CLAUDE.md` §3.1). |
| Branch | `mvp0/pr-tw-a-calendar`, created from `main` **after IL-a (#80) merges**. Until then the PR does not start. |
| Allowed commands | `cargo *` (including `$HOME/.cargo/bin/cargo`); `git`; `gh` for PR create and view, never merge; `python3 scripts/*`; `mkdir -p`; `sed -n`; the CLI binary under `target/`. |
| Not allowed | `python3 -c`, `sed -i`, `awk`, `xargs`, `curl`, heredoc writes. File changes go through Read, Edit and Write. |
| Authority | Commit and push to the branch; open the PR as **READY FOR OPERATOR REVIEW**; never merge. |
| Material stops | Stop the dependent action and report to the primary session with evidence when any of these happens: (1) an edit is needed to the kernel, contracts, presence, persistence, server, clients, `schedule` or any other pack; (2) a dependency is needed beyond `solar-positioning =0.7.0`; (3) IL-a's API must change; (4) any digest changes other than `market-town`'s; (5) a NOAA comparison falls outside ±0.5°. |
| Budget | Six 300-day town runs at most. |
| Gate | `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`, both doc checks. Each result is classified `PASS`, `FAIL` or `INCONCLUSIVE` from the output, never from the exit code alone. |

## 16.8 Freeze review amendments (binding, primary session 2026-10-08)

- **(a) Determinism of the sun.** `solar-positioning` is platform-sensitive only through `libm`, which is pure
  Rust and deterministic. C2 adds a test that pins **three exact `SunSample` integers**: San Diego at 2026-10-08
  12:00 local, San Diego at 2026-10-08 18:00 local, and 78° N at the winter solstice, noon. A dependency bump or
  a platform difference must therefore fail the test, not drift silently. The values are written once, from
  the first green run on the CI host, with a comment that a change to them is a material stop and not a
  re-golden.
- **(b) The market-town digest.** It changes by design in C4. The ledger records the new digest as the
  baseline, together with the commit that produced it. The `social-cafe`, `bodies-yard` and `long_run`
  digests must stay identical to `main`'s (INV-TW-1); any change to them is material stop (4).
- **(c) SD-TW-a-8 and an observer in no place.** An observer who is in no place (no presence location)
  gets **no** calendar records: the disclosure is keyed to the observer's place, and there is none to key it
  to. C3 tests this directly: an observer without a place receives an observation with neither
  `calendar.day` nor `calendar.light`, and receives both once placed. The HUD for such an observer falls back
  to `Day N, HH:MM` (INV-TW-6), which is acceptable because the 2D and 3D clients always place the player.

## 16.9 Ledger

Execution session: worktree `/Users/yuema137/mineworld-worktrees/impl-tw-a`, branch
`mvp0/pr-tw-a-calendar`, base `origin/main @ 543c80a` (IL-a #80 merged). Handoff:
[`handoff-tw-a.md`](handoff-tw-a.md).

**Start-of-session audit (2026-10-08).** §16.2's anchors re-verified on 543c80a: `systems/employment`
(`lib.rs` table, `process.rs` `ShiftProcess`, `system.rs` `react`/`wake`); `kernel/src/view.rs`
`start_process` l. 332, `reschedule_process` l. 368, `set_process_state` l. 392;
`systems/presence/src/interaction.rs` `discloses` l. 118; `systems/installed/src/lib.rs` (one line per
pack); `authoring/src/configuration.rs` `PackConfiguration { Configuration, FACTS, references,
requires, seed }`; `sdk/rust/src/pack.rs` `configures!()`; `worldpack/src/configure.rs` `read` / `seed` /
`compare`; `systems/schedule/src/time.rs` `DAY = 86_400`, `TimeOfDay::of = rem_euclid(DAY)`. IL-a's
final `seed(&Seeding, &Configuration) -> Result<Vec<Emission>, Rejection>` equals the branch's (§16.6's
first risk did not occur). No pack on main uses `configures!()` yet: `calendar` is its first.

### Commit ledger

| # | Implementation | Deterministic validation | LLM logic review |
| --- | --- | --- | --- |
| C1 | [x] ARC-67, DEP-30 appended to `docs/DECISIONS.md`; `CORE_CONCEPTS.md` §17 *Time* (calendar time, embodied time, time scale, paused; four rules); `systems/calendar/README.md`; the spec header as `src/lib.rs`'s module doc, with a doc-only `Cargo.toml` (TWa-D1) | [x] `check_doc_headings.py`: 191 numbered sections across 26 documents, none duplicated — PASS; `check_decision_ids.py`: 71 ids, all distinct — PASS; `cargo check -p mineworld-calendar` clean | [x] terms checked against CORE_CONCEPTS §§1, 10, 11: "calendar time", "embodied time", "time scale", "paused" are new terms, none a synonym of `Process`/`Event`/`WorldTime` (calendar time is *defined as* `WorldTime`, stated, not renamed); §17 is the "time section" §15.4 asked for (none existed) |
| C2 | [x] `civil.rs` (`CalendarDate` valid by construction; Hinnant `days`/`from_days`; `weekday` 0 = Monday); `day.rs` (`MicroDegrees`, `SunSample`, `DayEvents` + `light_changes`, `Phase`, `CalendarDay::compute`, `DAY`, `SAMPLE_INTERVAL` 900, `SAMPLES` 97); `sun.rs` (`sun_at`, `day_events`, `phase_at`; the only file naming the crate; ΔT = the library's estimate at mid-month of the instant's UTC month); dependency `solar-positioning =0.7.0`, no default features, `libm` | [x] `cargo test -p mineworld-calendar`: 8 passed (E-TWa-2): every day 1901–2099 round-trips (72 684 days); 2026-10-08 = day 20 734, Thursday; NOAA table ±150 s (rise −16 s, set −1 s, noon −8 s); elevation vs NOAA's model ≤ 0.002° at 09/12/15 h; three pinned samples (§16.8 a); 78° N polar night; equator equinox; azimuth convention. Clippy `-p mineworld-calendar --all-targets -D warnings` clean. Lock: + `solar-positioning 0.7.0` only (`libm 0.2.16` already locked). Mutations M-TWa-3a/3b below | [x] quantization: `f64::round` is half away from zero and exact on every IEEE host; azimuth 360 000 wraps to 0; event offsets kept only in 1 … 86 399 so an event never coincides with a midnight wake. Julian date `2 440 587.5 + utc/86 400` (|utc| < 2^53, exact division error ≈ 40 µs). Azimuth north = 0 clockwise is the library's convention and is tested. Phase uses unrefracted elevation against the same three horizons the event search uses, so a day's opening phase and its events agree; the track is refracted (apparent), as NOAA's displayed elevation is |
| C3 | [ ] | [ ] | [ ] |
| C4 | [ ] | [ ] | [ ] |
| C5 | [ ] | [ ] | — |

### Evidence

```text
E-TWa-0  2026-10-08, base capture on 543c80a (clean tree), dev profile, by target/tw-a/capture.sh base
         (IL-a's capture.sh method; "sha" = sha-256 of every output line but `wall`):
         social-cafe run 300 d seed 7: exit 0, faults 0, 365 330 facts, sha ad49c723…c64b, wall 13 s
         market-town run 300 d seed 7: exit 0, faults 0, 372 755 facts, sha 365b50e0…1d1d, wall 23 s
         bodies-yard 30 d seed 7: exit 0, faults 0, 62 385 facts, sha bd6a1002…80e6
         long_run second process: 4 019 632 bytes, sha 23f7fa76…5125
         long_run_objects second process: 612 428 bytes, sha c8358f8b…c5b4
         validate ×3: shas ebcd60a0…, 64f41086…, 7356b8f8… (= E-IA-0)
         Every value equals main's recorded reference (E-IA-8 / E-IA-13). Town runs used: 2 of 6.
E-TWa-1  NOAA references, read once on 2026-10-08 with the WebFetch tool (never at run time):
         (a) https://gml.noaa.gov/grad/solcalc/table.php?lat=32.7157&lon=-117.1611&year=2026 —
             "Time Zone Offset: America/Los_Angeles -7.0", local time with DST. 2026-10-08: sunrise
             06:48, sunset 18:24, solar noon 12:36:14 (PDT) → at the world's fixed −08:00: 05:48, 17:24,
             11:36:14. Cross-check: its 2026-12-21 row (standard time) 06:47 / 16:47 / 11:46:38. The
             same URL with `&tz=-8` prints the same (the parameter is ignored).
         (b) Elevation at an instant is computed in the browser (azel.html, the current calculator) and
             published as no table, so the elevation reference is NOAA's own spreadsheet model
             (NOAA_Solar_Calculations_day.xls, linked from
             https://gml.noaa.gov/grad/solcalc/calcdetails.html, read 2026-10-08; refraction formulas
             as that page prints them), transcribed in `src/sun/tests.rs` `noaa_elevation` — an
             independent model, not SPA. (TWa-D2.)
E-TWa-2  C2, 2026-10-08, `cargo test -p mineworld-calendar -- --nocapture` (macOS arm64, dev):
         8 passed. San Diego 2026-10-08 (−08:00): sunrise 20 864 s = 05:47:44 (NOAA 05:48, −16 s),
         sunset 62 639 s = 17:23:59 (NOAA 17:24, −1 s), solar noon 41 766 s = 11:36:06 (NOAA 11:36:14,
         −8 s) — all within CP-TW-a's ±2 min. Other events: astronomical dawn 15 956, civil dawn
         19 384, civil dusk 64 118, astronomical dusk 67 540. Elevation vs NOAA's model: 09:00 36.369°
         / 36.371°, 12:00 50.759° / 50.761°, 15:00 27.965° / 27.966° — max |Δ| 0.002° (bound 0.5°).
         Pinned SunSample (§16.8 a), millidegrees (elevation, azimuth):
           San Diego 2026-10-08 12:00 −08:00    ( 50 759, 189 418)
           San Diego 2026-10-08 18:00 −08:00    ( −8 391, 267 971)
           78° N 15° E 2026-12-21 12:00 +01:00  (−11 440, 180 458)
         78° N 2026-12-21: astronomical dawn 27 433, solar noon 43 083, astronomical dusk 58 731;
         sunrise, sunset, civil dawn and dusk None; phase at midnight Night. PASS.
M-TWa-3a Mutation (TW-a adversarial 3): `features = []` (neither libm nor std) → the crate does not
         compile ("cannot find module or crate `libm`", ×8 in solar-positioning). Fails loudly. Reverted.
M-TWa-3b Variant: `features = ["std"]` (the platform's math) → 8 passed on macOS: at millidegree
         quantization the platform's and libm's results agree on this host. Expected, and it is why the
         guard is the exact feature pin plus the pinned samples on two OSes (macOS here, Linux in CI),
         not a test that could tell the backends apart on one host. Reverted; `git diff` of
         Cargo.toml empty against the intended text.
```

### Deviations and findings

```text
TWa-D1  (bounded) C1 carries a doc-only crate skeleton (Cargo.toml with no dependencies, lib.rs = the
        spec header). Reason: the workspace's `systems/*` glob refuses a member directory without a
        manifest (`cargo metadata`: "failed to read systems/calendar/Cargo.toml"), so a README alone
        would break every build. Impact: none; the solar dependency still lands in C2.
TWa-F1  (MATERIAL, found at start, blocks C4 only) AC-1 check 3
        (tests/acceptance/tests/ac1_composability.rs `compare_systems`, `compare_manifests`,
        `world_delta_failures`; ARC-35 item 4) requires Market Town's systems to be Social Café's list
        plus exactly the six market packs, every other world.yaml key equal, and no top-level entry
        but items/ and organizations/ in one pack only. SD-TW-a-10 (market-town adds `calendar`,
        `configure: [calendar]` and configure/) fails all three. Not anticipated by §16; the frozen
        stop list does not name the acceptance tests, but the honest fixes each change a frozen
        acceptance measure (ARC-35) or a frozen scope item (SD-TW-a-10). C1–C3 are unaffected and
        proceed; C4 is reported to the primary session (see the C4 row).
TWa-D2  (bounded) The ±0.5° elevation check (§16.5) is against NOAA's published spreadsheet equations,
        not a published elevation value: NOAA publishes none for an instant (E-TWa-1 b). The rise,
        set and noon checks are against NOAA's published table. Impact: none on the acceptance bound.
TWa-D3  (bounded) Component types are `calendar-day` and `calendar-light`, not `calendar.day` /
        `calendar.light` (§5.5): an identifier may contain only a–z, 0–9, '-', '_'
        (contracts/src/ids.rs `is_legal_identifier_byte`).
TWa-D4  (bounded) `day-began` also carries `phase`, the light phase at day_start, and the Process state
        holds `{ configuration, day, phase }` rather than SD-TW-a-3's `next`: the next due event is a
        pure function of the day record and the instant, recomputed at each wake, and the phase is what
        `calendar-light` discloses. Without the opening phase a day that begins in twilight or polar day
        would have no fact saying so, and the fold could not be the state.
```

| Item | Status | Evidence |
| --- | --- | --- |
| `market-town` digest baseline (§16.8 b) | not recorded | — |

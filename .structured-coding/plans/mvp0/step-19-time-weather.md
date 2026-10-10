# Step 19 — S19: World time, two time domains, pause, day and night, weather

**Lifecycle:** step plan reviewed; the questions are ruled (§14.1, 2026-10-08). **TW-a (§16) is `DESIGN
FROZEN 2026-10-08`** and merged (#94, f80bbb7). **TW-b (§17) and TW-d (§18) are `DESIGN FROZEN
2026-10-09`** (primary session; rulings in §17.9.1 and §18.9.1). Each authorizes implementation under its own
execution contract (§17.10, §18.10). TW-d starts only after TW-b and IL-b have merged. TW-c, TW-e, TW-f and TW-g are scoped in §11
and are not frozen.
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
fixes scope, checkpoints and adversarial criteria only. (TW-a, TW-b and TW-d are designed in place, §16,
§17 and §18, to keep one authority. §17.6 and §18.6 restate the TW-b and TW-d rows with the corrections
their audits required, each with its reason. Where a row and its section differ, the section governs once
it is frozen.)

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
| C3 | [x] `configuration.rs` (`CalendarConfiguration`, decode-only, `deny_unknown_fields`; `Epoch` 1901–2099 `YYYY-MM-DD`, `UtcOffset` `±HH:MM` ≤ 14 h, `Latitude` ±90, `Longitude` ±180 → micro-degrees); `event.rs` (`CalendarConfigured`, `DayBegan { day, phase }`, `DaylightChanged { phase }`); `process.rs` (`CalendarProcess` "calendar", `CalendarState { configured, day, phase }`); `component.rs` (`calendar-day`, `calendar-light`, declared, carried by no entity); `system.rs` (`PackConfiguration` + `configures!()`; `react` starts the process from `calendar-configured` and folds the other two; `wake` states midnight or the light changes due; `discloses` on the observer's place only); registry line `Calendar => mineworld_calendar::CalendarSystem` + its Cargo line | [x] E-TWa-3: `cargo test -p mineworld-calendar` 8 + 5 + 2; `-p mineworld-installed-systems -p mineworld-worldpack` all ok; `-p mineworld-acceptance` all ok (ac1_composability 13, precursor_vocabulary, seam guards unedited); `-p mineworld-cli --test configure --test commands` ok; clippy `--all-targets --all-features -D warnings` on calendar and installed clean; fmt clean. M-TWa-4 killed | [x] single ownership: only this pack writes its Process; it reads presence's `Presence` (declared dependency) and nothing of host pacing — no time scale, no pause exists in its inputs (INV-TW-2); the Process state is written only in `react` from fact payloads, `wake` only reschedules and states facts, so a snapshot equals the fold (resume test byte-identical); the configured fact is SystemInternal with no subjects; Public facts have no subjects; disclosure is keyed to `Presence` of the observer, so §16.8 (c) holds by construction and is tested; an enabled-but-unconfigured calendar states and discloses nothing (QTW-8's warning is `mineworld check`'s, not in TW-a's scope) |
| C4 | [x] Unblocked by the operator's ruling of 2026-10-09 (option (i), TWa-R1). Two commits: 344d28b the ARC-35 note and AC-1 check 3's `GENERIC_PACKS = ["calendar"]` (`compare_systems`, new `compare_configure`, `configure/` entries); 4018434 the opt-in (`configure/calendar.yaml`, `calendar` after the six, `configure: [calendar]`), byte-identical to the parked cf91bbb | [x] E-TWa-8: AC-1 14/14 with C4; both ruled mutations red, naming the pack. E-TWa-9: market-town baseline `24a95d2a…d270` on 4018434; the other digests equal main. CP-TW-a: E-TWa-4 | [x] the market delta is still exact: the six are compared as a set right after Social Café's list; a generic pack is admitted only after them, once, and only if allow-listed; `configure` is admitted only for an enabled allow-listed pack and never in Social Café; check 3's section rule is unchanged, so a generic pack cannot add a section to a person or place file; checks 1 and 2 untouched |
| C5 | [x] ledger, handoff, PR #94 (C4 parked; not READY — TWa-F1) | [x] full gate E-TWa-6; CI E-TWa-7 | — |

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
E-TWa-3  C3, 2026-10-08 (macOS arm64, dev): `cargo test -p mineworld-calendar`: lib 8, tests/calendar 5,
         tests/configuration 2 — all pass. tests/calendar: 7 days → 7 `day-began` at 0, 86 400, …,
         dated 2026-10-08 … 14 with weekdays 3,4,5,6,0,1,2 (Thu … Wed), each at TimeOfDay 0
         (schedule's), each naming its own midnight, opening phase Night; 42 `daylight-changed`, six a
         day in the order AT, CT, Day, CT, AT, Night at exactly the instants the day record names; the
         process state equals the last day and last phase, next due 7 × 86 400. Facts: day-began and
         daylight-changed Public, calendar-configured SystemInternal, none with subjects. Unconfigured:
         no facts, no records. §16.8 (c): `bo` (no Presence) → no calendar record; `ada` → exactly
         [(square, calendar-day), (square, calendar-light)]; the noon record's sample 48 is the pinned
         (50 759, 189 418) and the light is Day; after `place-me` puts bo in the square, bo gets the
         same two. Restart: stopped at day 2 14:30, resumed, run to day 8 → 6 day-began (last
         2026-10-16), facts byte-identical to the uninterrupted world's. tests/configuration: through
         `WorldPack::read`, latitude 91 refused as "…/configure/calendar.yaml is not a valid
         configuration file: error: line 3 column 11: latitude is decimal degrees from -90 to 90, not
         91" (file, line/column, key, value; adversarial 5 PASS); 11 out-of-range values each refused
         naming their key; 9 bound values accepted; an unknown key `dst` refused.
M-TWa-4  Mutation (adversarial 4): `day_start = index × 86 400 + 1` in system.rs `day_at` → 3 of 5
         calendar tests fail, INV-TW-4's among them ("the record names its own midnight": WorldTime(1)
         vs WorldTime(0)); also the noon sample (azimuth 189 424 ≠ 189 418) and the restart count.
         Killed. Reverted; `git grep MUTATION -- '*.rs'` empty.
E-TWa-4  C4 applied, uncommitted, on fa78d36 (= the commit parked as cf91bbb on
         mvp0/pr-tw-a-c4-proposed), 2026-10-08, dev:
         AC-1 check 3: `cargo test -p mineworld-acceptance --test ac1_composability check_3` FAILS:
           "world.yaml: `configure` differs" · "the systems appended to Social Café's are [item,
           inventory, item-transfer, economy, employment, consumption, calendar], not the six market
           packs" · "configure: present in one pack only". TWa-F1 confirmed. → MATERIAL STOP for C4.
         CP-TW-a through the binary: `mineworld validate worlds/market-town` valid, 131 genesis facts.
           `run worlds/market-town --headless --seed 1 --days 7 --save D`: exit 0, faults 0, 9 046 facts;
           calendar-configured 1, day-began 8 at t0, t86400, … t604800 (TWa-D5), daylight-changed 42
           (six a day for days 0–6). Day 0's sunrise fact at t20864 = 05:47:44 and sunset at t62639 =
           17:23:59 (NOAA 05:48 / 17:24 at −08:00; −16 s / −1 s). Dates and weekdays of the facts are
           the pack's (E-TWa-3 asserts 2026-10-08 … 14, Thu … Wed on the same code path).
           Restart: D resumed to `--days 8` ("resumed … at revision 6753 (snapshot 6720 + 33
           re-executed)") vs an uninterrupted 8-day run E: both 10 360 facts, fingerprint
           ed1a964ca73864f9, `inspect --last 20000` identical but for the instance id; `replay` of each:
           "7748 revision(s) re-executed from genesis, 10360 fact(s) and 122 snapshot(s) reproduced byte
           for byte". PASS for CP-TW-a's content; its landing waits on C4.
E-TWa-5  INV-TW-1 / §16.8 (b), by capture.sh c4 on fa78d36 + the C4 opt-in (artifacts target/tw-a/c4-*):
         social-cafe 300 d seed 7: sha ad49c723…c64b, 365 330 facts (= main, E-TWa-0) — PASS
         bodies-yard 30 d: sha bd6a1002…80e6 (= main) — PASS
         long_run: 4 019 632 bytes, sha 23f7fa76…5125 (= main) — PASS
         long_run_objects: 612 428 bytes, sha c8358f8b…c5b4 (= main) — PASS
         validate social-cafe / bodies-yard: = main; market-town: 31fb85d4…0620 (changed by design)
         market-town 300 d seed 7 — CANDIDATE baseline (not recorded as the baseline: C4 has not
           landed): sha 24a95d2ae4e9d99b0e183de8df5f5d1d08eb5edb19127bccbd20f7532a66d270, faults 0,
           374 857 facts = main's 372 755 + 2 102 = 1 calendar-configured + 301 day-began + 1 800
           daylight-changed; every other fact count equal to main's; wall 20 s.
         Town runs used: 4 of 6 (E-TWa-0 ×2, E-TWa-5 ×2).
E-TWa-6  Full gate on e7fdee7 (C1–C3 + ledger, origin/main aa74b32 merged in — planning documents only,
         no code), clean tree, macOS arm64, by target/tw-a/gate.sh, 2026-10-08/09; host load average
         ≈ 270 (other sessions):
         cargo fmt --all --check                         exit 0                  PASS
         check_doc_headings.py                           191 sections, distinct  PASS
         check_decision_ids.py                           71 ids, distinct        PASS
         check_ci_pins.py                                exit 0                  PASS
         check_scratch.py scan                           159 sources, 2 exempt   PASS
         cargo clippy --workspace --all-targets --all-features -D warnings   exit 0   PASS
         cargo test --workspace --no-fail-fast           exit 101, 59 min: 173 test binaries,
           755 passed, 0 failed, 9 ignored, and one harness-less target panicked:
           persistence/tests/kill_and_resume.rs:682 "middle: the victim died of SIGKILL" — the
           victim reached revision 236 before a kill sent at 126 landed (scenario `clock`), i.e. it
           finished before the signal arrived. A timing race under a saturated host; TW-a changes
           nothing persistence runs (the test composes its own scenarios, not the installed set).
           Classified INCONCLUSIVE for TW-a, not FAIL: the same target passed in CI's `test` on the
           same head (E-TWa-7). Re-run alone: see E-TWa-6b.
         check_scratch.py left --target-dir target       exit 1: 4 entries, 0 B, all
           mineworld-kill-* in the system temp dir — the panicked kill test's own scratch. Follows
           from the line above; INCONCLUSIVE for the same reason.
E-TWa-6b `cargo test -p mineworld-persistence --test kill_and_resume` alone on e7fdee7, load ≈ 277:
         "[cafe] PASS", "[clock] PASS", exit 0 (899 s wall, 11 s user). The full-gate panic was the
         load race. `check_scratch.py left` still lists the 4 empty `mineworld-kill-*` directories the
         panicked run left in the system temp dir (0 B; removing them needs `rm`, outside this
         session's allowed commands — left for the operator; the re-run added none).
E-TWa-8  AC-1 amendment, 2026-10-09, on 1b4318a (origin/main 15b05a9 merged: S11-B and plans; the
         only change on the headless path is `RuleController::since`, a new constructor — `new()` is
         unchanged, confirmed by E-TWa-9) + 344d28b + the C4 files:
         `cargo test -p mineworld-acceptance --test ac1_composability`: 14 passed (13 existing + the new
         `only_allow_listed_generic_packs_may_follow_the_six_and_only_after_them`, which holds the
         allowed shape, both mutations and the `configure` rules on hand-written manifests).
         Mutations on the real worlds/market-town/world.yaml, each through `check_3`:
         M-TWa-A1 `- bodies` appended after `calendar` → FAILED: "world.yaml: `bodies` follows the six
                  market packs and is not an allow-listed generic pack [\"calendar\"]". Killed.
         M-TWa-A2 `- calendar` moved between economy and employment → FAILED: "world.yaml: `calendar` is
                  an allow-listed generic pack at position 11; it may only follow the six market
                  packs". Killed.
         Both reverted; world.yaml and configure/calendar.yaml diff-equal to cf91bbb; `git grep
         MUTATION` empty; check 3 green again. check_decision_ids: 73 ids distinct; headings PASS;
         clippy -p mineworld-acceptance --all-targets --all-features -D warnings clean.
E-TWa-9  By capture.sh head on 4018434 (clean tree), 2026-10-09, dev:
         social-cafe 300 d seed 7: sha ad49c723…c64b, 365 330 facts (= main) — PASS
         market-town 300 d seed 7: sha 24a95d2a…d270, 374 857 facts, faults 0 — the BASELINE (§16.8 b)
         bodies-yard 30 d: bd6a1002…80e6 (= main); long_run 4 019 632 B 23f7fa76…5125 (= main);
         long_run_objects 612 428 B c8358f8b…c5b4 (= main); validate ×3 as E-TWa-5 — PASS.
         Town runs used: 6 of 6 (E-TWa-0 ×2, E-TWa-5 ×2, E-TWa-9 ×2). Budget exhausted, not exceeded.
         The four empty `mineworld-kill-*` directories of E-TWa-6b were removed with
         `rm -rf "$TMPDIR"/mineworld-kill-*` (authorized by the coordinator, 2026-10-09).
E-TWa-7  CI on e7fdee7 (PR #94, pull_request run 37891509810): fast PASS (1 m 13 s), test PASS
         (13 m 38 s; its log shows `sun::tests::three_sun_samples_are_pinned_exactly ... ok` on
         Linux x86_64 — the three pinned integers equal macOS's, §16.8 (a) held on two OSes).
         https://github.com/yuema137/MineWorld/actions/runs/37891509810
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
TWa-D5  (bounded) `run --days 7` records 8 `day-began`, not 7: `run` advances to instant 7 × 86 400
        inclusive, and that instant is 2026-10-15's local midnight. The seven dates CP-TW-a names are
        all there (t0 … t518400); the eighth begins at the run's last instant. No code change: a day
        that begins at the final instant has begun.
TWa-F2  (cross-platform, operator requirement 2026-10-08: macOS, Linux, Windows) What TW-a adds holds no
        Unix-only assumption: tests build paths with `Path::join` (the refusal test checks
        `configure` + `calendar.yaml` joined by the platform's separator, not a literal '/'), use
        DEP-29's scratch (no `/tmp`), no signal and no shell helper; the sun is `libm`-only so the
        pinned integers are OS-independent. The one Unix-only artefact is this PR's evidence script
        target/tw-a/capture.sh (bash, shasum; untracked, under target/, never run by a test or CI) —
        recorded, not a product defect. No existing test was edited by TW-a.
        Encountered (not edited): persistence/tests/kill_and_resume.rs kills its victim with SIGKILL
        and keeps scratch in the system temp dir (`mineworld-kill-*`), both Unix-only assumptions,
        and is timing-sensitive under load (E-TWa-6). Owner lane: persistence (S6) with the
        test-hygiene / DEP-29 lane for the scratch location.
C4 decision requested (material stop, §16.7 spirit: a frozen acceptance measure). The smallest options:
  (i)  amend ARC-35 by a note and AC-1 check 3 to admit packs appended after the six market packs
       when each is a configured world-level pack with its `configure:` entry and `configure/` file
       (e.g. a fixed allow-list `["calendar"]`) — the market delta still measured exactly; owner: the
       AC-1 / S9 lane; then C4 lands as cf91bbb;
  (ii) keep market-town as AC-1's fixture and opt a different World Pack into the calendar (a new
       scope item; CP-TW-a re-targeted), leaving ARC-35 untouched;
  (iii) drop C4 from TW-a and land it with the client PR (TW-e) after (i) or (ii).
  Recommendation: (i) — AC-1 measures "Social Café + the market", and a calendar is neither; the guard
  should say which later packs are configuration, as ARC-35's notes have done before.
TWa-R1  RULING (operator, 2026-10-09, relayed by the coordinator): option (i). "AC-1 check 3 stays exact
        for the market delta (exactly the six market packs appended to Social Café); generic packs on
        an explicit allow-list may also appear after them, enabled through configuration." Implemented
        in TW-a: ARC-35 note (2026-10-09), `GENERIC_PACKS = ["calendar"]`, two mutations killed
        (E-TWa-8), C4 landed (4018434), market-town baseline recorded (E-TWa-9). Weather and later
        generic packs extend the allow-list in their own PRs.
```

| Item | Status | Evidence |
| --- | --- | --- |
| `market-town` digest baseline (§16.8 b) | **recorded** | 300 days, seed 7: sha `24a95d2ae4e9d99b0e183de8df5f5d1d08eb5edb19127bccbd20f7532a66d270`, 374 857 facts, faults 0; produced by commit 4018434 (E-TWa-9; the same value as the candidate in E-TWa-5) |

---

# 17. TW-b — the `weather` System Pack (seeded rules)

**`DESIGN FROZEN 2026-10-09 (primary session; rulings in §17.9.1)`**

Design revision: §17 as of PR #107 (drafted by the planning session on 2026-10-09 in worktree
`/Users/yuema137/mineworld-worktrees/plan-tw-bd`, branch `plan/s19-tw-bd`, from `origin/main @ f80bbb7`).
Approved by: the primary session, 2026-10-09, relayed by the coordinator; QTWb-1 … 7 ruled as recommended.
Implementation base: `origin/main` at the start of execution, at or after f80bbb7; IL-b need not be merged
(R-TWb-1). Execution contract: §17.10. Lifecycle: FROZEN.

Frozen means the scope (§17.1), the decisions (§17.3, §17.4), the acceptance and adversarial criteria
(§17.6) and the execution contract (§17.10) are frozen. Progress, evidence, audit findings and bounded
corrections stay writable.

This section is TW-b's single PR design authority and, once frozen, its ledger (§17.11). If the execution
session moves it into a separate `pr-TW-b-weather.md`, it moves it whole and leaves a pointer here; the two
never exist side by side.

## 17.1 Goal, scope, non-goals

**Goal.** A world that enables `calendar` and `weather` has weather hour by hour. Every observer sees the
same weather, and other packs can react to its changes. The weather comes from a seeded, integer-only
generator whose climate is content (`configure/weather.yaml`), and its default table is San Diego's.

**In scope.**
- A new crate `systems/weather` (`mineworld-weather`), registered in the installed set.
- The WGEN-lite daily generator (§3.3) and the daily-to-hourly derivation (§6.6), both integer-only and
  keyed by counter-based SplitMix64.
- `weather-configured`, `weather-day` and `weather-changed` facts; a world-level `climate` Process.
- Disclosure on the observer's place.
- A declared dependency on `calendar` (refusal when it is absent).
- Market Town opts in with a provisional San Diego rules table taken from NOAA's 1991–2020 normals.
- AC-1's `GENERIC_PACKS` gains `"weather"`.
- `docs/DECISIONS.md` **ARC-68**.

**Non-goals.**
- Record-driven weather, the station CSV, `data:` attachments, the fetch tool and DEP-31 (all TW-d).
- Any client (TW-e, TW-f) or host-pacing change (TW-c).
- Any controller reading weather (S10's lane).
- `mineworld check` warnings (QTWb-5).
- Regional weather (QTW-12).
- The GHCNh hourly layer (TW-g).
- Any change to the kernel, contracts, presence, persistence, the server, `calendar` or any other pack's
  source (INV-TW-3).

## 17.2 Audit anchors (`origin/main @ f80bbb7`, read 2026-10-09; re-verify at freeze)

| File / symbol | Finding | Consequence for TW-b |
| --- | --- | --- |
| `systems/calendar/src/system.rs` | Pattern to copy. `PackConfiguration` + `configures!()`. `react` starts the Process from the configured fact and folds the pack's own facts with `set_process_state`. `wake` only reschedules and states facts. `discloses` is keyed to the observer's `Presence` place. | `weather` follows the same shape, so the Process state is the fold of its facts (§16.9 C3 review). |
| `systems/calendar/src/event.rs` `DayBegan { day: CalendarDay, phase }`; `day.rs` `CalendarDay::{date, weekday, day_start, events, track}`, `DayEvents::{sunrise, civil_dawn, …}: Option<u32>` (seconds after `day_start`) | Public, no subjects. The date and the light events arrive in the payload. | `weather` subscribes to `day-began` and decodes it with calendar's published type. It needs the crate dependency `mineworld-calendar` and never recomputes a date or the sun (§6.1). |
| `kernel/src/dispatch.rs` `genesis` (l. 509–527) and `reduce` (l. 605–) | Genesis records every seeded fact first, then reduces **generation by generation** (breadth-first), with subscribers in registration order. | With `configure: [calendar, weather]`, `calendar-configured` and `weather-configured` are both in generation 0. The climate Process exists before day 0's `day-began` (generation 1) reaches `weather`. At each midnight: `day-began` (gen 1) → `weather-day` (gen 2) → fold and a possible `weather-changed` (gen 3). |
| `kernel/src/registry.rs` `check_dependencies` (l. 207–227) | A declared dependency that is not installed gives `SystemDependencyMissing`; one that is not enabled gives `SystemDependencyDisabled`. The world is refused at assembly. | `depending_on([PresenceSystem::ID, CalendarSystem::ID])` gives TW-b criterion 4 with no new mechanism. |
| `kernel/src/process.rs` `ProcessStart::new` (l. 192) / `ending_at` (l. 220) | Open-ended is the default, and `ending_at` must be strictly later than the start. | The climate Process is rescheduled to the next condition change, or left open-ended until the next `day-began` when no change is due. |
| `kernel/src/view.rs` `start_process` l. 332, `reschedule_process` l. 368, `set_process_state` l. 392 | Available on `WorldView` in `react` and in `wake`. | As in calendar. |
| `authoring/src/configuration.rs` (main) `PackConfiguration { Configuration, FACTS, references, requires, seed(&Seeding, &C) }` | IL-a's seam. On the IL-b branch, `seed` gains a third argument, `&ConfigurationContext` (`mvp0/pr-il-b-interactions @ 29d0b49`, SD-IB-3). | TW-b writes `seed` against whichever signature `main` has at its branch point. Whichever of TW-b and IL-b merges second adapts the other's implementors (step-18 §12.6 "Other lanes"). |
| `systems/installed/src/lib.rs`, `Cargo.toml` | One line in each per pack. `tests/installed.rs` holds them equal. | Two lines. |
| `tests/acceptance/tests/ac1_composability.rs` l. 1026–1030 `GENERIC_PACKS = ["calendar"]` | Check 3 admits allow-listed generic packs only after the six market packs, each once. `configure:` entries and `configure/<key>.yaml` are admitted only for allow-listed packs. The doc comment says "Each is added by the PR that brings its pack" (TWa-R1). | `["calendar", "weather"]`. No ARC-35 amendment is needed (TWa-R1 already covers later generic packs). |
| `cognition/rule-controller/src/paced.rs` l. 311–313 `mix(a, b)` | SplitMix64's finalizer over two words. It is the tree's idiom, and no System Pack depends on `rand`. | `weather` restates it as its own private function and does not depend on the controller (a pack never depends on cognition). A test pins its outputs against three published SplitMix64 reference values, so the restatement cannot drift silently. |
| `systems/presence/src/observe.rs` `owned_by_an_enabled_system` | A disclosed record survives only when its component type is declared by an enabled system. | `weather` declares `weather-today` and `weather-now` as owned component types that no entity carries (calendar's pattern, TWa-D3 naming rule: `[a-z0-9_-]`). |
| `persistence/src/world.rs` l. 13 `DEFAULT_SNAPSHOT_INTERVAL = 64`; `sqlite.rs` `checkpoint` `INSERT OR IGNORE` | A full world snapshot every 64 revisions, all retained. | Every byte of Process state is multiplied by the number of snapshots (F-TWbd-1). The climate state is kept small (§17.3 SD-TW-b-6). |
| `worlds/market-town/world.yaml` | `calendar` is last in `systems:`, and `configure: [calendar]`. | Append `weather` after `calendar`, and `configure: [calendar, weather]`. |
| `.github/workflows/ci.yml` | `ubuntu-24.04` only. There is no macOS or Windows CI job. | Cross-platform claims rest on integer-only arithmetic, a no-float source scan and the Linux CI run (§17.7 R-TWb-5). |

**Evidence read for realism** (WebFetch, 2026-10-09, never at run time):
- **E-TWb-pre-1.** NOAA NCEI U.S. Climate Normals 1991–2020, monthly, station USW00023188 (San Diego
  Lindbergh Field), <https://www.ncei.noaa.gov/data/normals-monthly/1991-2020/access/USW00023188.csv>.
  Columns `MLY-TMAX-NORMAL`, `MLY-TMIN-NORMAL` (°F) and `MLY-PRCP-AVGNDS-GE001HI` (mean days with
  ≥ 0.01 in).
  - TMAX, January to December: 66.4, 66.2, 67.0, 68.8, 69.5, 71.7, 75.3, 77.3, 77.2, 74.6, 70.7, 66.0.
  - TMIN, January to December: 50.3, 51.8, 54.5, 57.1, 60.0, 62.6, 66.1, 67.5, 66.2, 61.5, 54.8, 49.8.
  - Days with ≥ 0.01 in, January to December: 6.5, 7.1, 6.2, 3.8, 2.2, 0.7, 0.7, 0.3, 0.9, 2.4, 3.7, 5.8.
  - The implementing session re-reads this file in C4 and records the values it used.
- **E-TWb-pre-2.** The GHCN-Daily element definitions,
  <https://www.ncei.noaa.gov/pub/data/ghcn/daily/readme.txt>. PRCP is tenths of mm, TMAX and TMIN tenths
  of °C, AWND tenths of m/s, and the missing value is −9999. Used for units only.

## 17.3 Design decisions (SD-TW-b-n)

| Id | Decision |
| --- | --- |
| SD-TW-b-1 | **Crate.** `systems/weather` (`mineworld-weather`), registered as `Weather => mineworld_weather::WeatherSystem`. It depends on `mineworld-{contracts,kernel,authoring,presence,sdk,calendar}`, `serde`, `serde_json` and nothing else: no new external dependency, and nothing new in `Cargo.lock`. Its dev-dependencies are those of calendar's tests (`mineworld-persistence`, `mineworld-test-support`, `mineworld-worldpack`, `serde-saphyr`). |
| SD-TW-b-2 | **Declaration.** `depending_on([PresenceSystem::ID, CalendarSystem::ID])`. It owns `weather-today` and `weather-now`, emits its three facts, subscribes to its three facts and to calendar's `day-began`. It does **not** subscribe to `daylight-changed`: hours are placed from the day record's events. It provides no action. `VERSION = 1`. |
| SD-TW-b-3 | **Configuration** (`configure/weather.yaml`, `deny_unknown_fields`, decoding is validating; §17.4 has the exact schema). In TW-b `source` admits only `rules`. A `source: record` is refused while decoding with "source `record` needs the record data of TW-d; this build accepts `rules`", which names the key. The rules live **inline** under `rules:`, not in a separate file: IL-a's seam reads one YAML file per key, and IL-b's attachments live under `data/` and are bytes the pack would have to parse itself (QTWb-1). `FACTS = [weather-configured]`. `seed` emits one `weather-configured`, SystemInternal, with no subjects. |
| SD-TW-b-4 | **`weather-configured` is shaped for TW-d now.** The payload is `{ seed: u64, rules: Rules, record: Option<RecordRef> }`, and TW-b always writes `record: None`. TW-d fills it in without bumping the event schema version, so a TW-b-era save stays readable by TW-d's code. (Before a stable contract this is not compatibility debt: it is the payload's final shape stated once.) `RecordRef` in TW-b is a unit-less placeholder type that cannot be constructed (an empty enum), so no TW-b world can claim a record. |
| SD-TW-b-5 | **The generator** (`generate.rs`), WGEN-lite, per world day `d` (the index `day_start / 86 400`). (1) Wet or dry: `u₀ < (wet_yesterday ? p_wet_after_wet : p_wet_after_dry)`, both per-mille. (2) A wet day's amount: quintile `q = u₁ × 5 / 1000`, the amount is `rain_tenth_mm[q]`, and the result is at least 3 (0.3 mm, the first record value at or above NOAA's 0.01 in wet-day threshold). (3) Temperature anomalies as integer AR(1): `aₜ = aₜ₋₁ × t_ar_permille / 1000 + noise`, with `noise = u₂ × (2 × t_noise_dc + 1) / 1000 − t_noise_dc` (and likewise for tmin with `u₃`). Rounding is toward zero, as `i64` division does, which is stated and tested. TMAX = `tmax_dc + a_max + (wet ? wet_tmax_shift_dc : 0)` and TMIN = `tmin_dc + a_min`, with TMIN ≤ TMAX − 1 enforced by lowering TMIN. (4) Fog: `u₄ < fog_permille`. Thunder (wet days only): `u₅ < thunder_permille`. Morning overcast (dry days only): `u₆ < overcast_morning_permille`. (5) Wind: `wind_dms` and `wind_from_deg` of the month, with no noise in TW-b. Each `uₖ` is a per-mille draw `draw(seed, d, k)` (SD-TW-b-7). The month is the calendar date's month from `day-began`. |
| SD-TW-b-6 | **The Process state stays small.** The `climate` Process (`ProcessKind` type `climate`, no place, no participants, uninterruptible) holds `{ configured, today: WeatherDay, now: HourIndex, chain: Chain }`. The rules are 12 × 11 integers, about 2 KB as JSON. `Chain { wet: bool, tmax_anomaly_dc: i16, tmin_anomaly_dc: i16 }` is the generator's carry. Because every snapshot holds the whole state (F-TWbd-1), the encoded climate state has a test-pinned budget of **8 KB**. Exceeding it is a FAIL. |
| SD-TW-b-7 | **Draws.** `draw(seed, day, k) = permille(mix(mix(seed, day as u64), k))`, where `mix` is SplitMix64's finalizer restated in `draw.rs` (§17.2) and `permille(x) = ((x >> 32) × 1000) >> 32`, which is in `0 … 999` by a multiply-shift with no modulo bias. Draw indices are fixed constants: 0…6 for the daily draws and 16 + n for hour placement. A new draw takes a new index, so existing draws never shift. |
| SD-TW-b-8 | **Daily to hourly** (`hours.rs`, all integers). Three parts follow. (a) Temperature (SD-TW-b-8a): a 24-entry per-mille curve, pack constant `DIURNAL`, with 0 at the sunrise hour and 1000 at 15:00, rising on a half-sine and decaying exponentially after 15:00 to the next sunrise. This is the shape of Parton & Logan (1981), *Agricultural Meteorology* 23:205–216, precomputed once offline into integers and committed with the citation and the generating formula in a comment. The sunrise hour is `day.events().sunrise / 3600`, falling back to 6 when it is `None` (polar). `T(h) = TMIN + (TMAX − TMIN) × curve(h) / 1000`, so `max T(h) == TMAX` and `min T(h) == TMIN` exactly. (b) Wet hours (SD-TW-b-8b): pack constant `WET_HOURS` by daily amount (≤ 2.0 mm → 2 h, ≤ 10 mm → 4 h, ≤ 25 mm → 8 h, otherwise 12 h). This is a stated design default, not a measured climatology; TW-g's hourly layer replaces it. The hours form one run, or two runs when the amount exceeds 10 mm, placed by draws 16 and 17. The rate per wet hour is `prcp / wet_hours`, with the remainder added to the first wet hour so the 24 hours sum exactly to the day's amount. (c) Condition (SD-TW-b-8c): a wet hour is `drizzle` when its rate is ≤ 1.0 mm/h, which uses the AMS Glossary's "drizzle … seldom exceeds 1 mm per hour". It is `rain` when the rate is ≤ 7.6 mm/h and `heavy-rain` above that, which uses the AMS Glossary's heavy-rain threshold of more than 7.6 mm/h. A wet hour on a thunder day is `thunderstorm`. A fog day is `fog` from the civil-dawn hour (falling back to 5) to 09:59. A dry hour takes its cloud from the day: overcast-morning days have 8 oktas from 05:00 to 10:59, wet days 8 oktas in and next to wet hours, and otherwise 1 okta. Cloud then maps by the okta scale: 0–2 `clear`, 3–6 `partly-cloudy`, 7–8 `overcast`. Wind is the day's wind every hour. The AMS and WMO sources are cited in code with the URL and the date read. |
| SD-TW-b-9 | **Facts.** `weather-day` is SystemInternal: `WeatherDay { day_start, date, origin: Rule, summary: DailyWeather { tmax_dc, tmin_dc, prcp_tenth_mm, awnd_dms, wind_from_deg, fog, thunder, overcast_morning }, chain: Chain, hours: [WeatherHour; 24] }`, with `origin` an enum whose `Record` and `Filled` variants arrive with TW-d. `weather-changed` is **Public**: `{ hour: u8, condition, cloud_oktas, precipitation_tenth_mm }`, emitted when an hour's condition differs from the condition in force. The `summary` is in the fact (§11.2's TW-d checkpoint needs exact TMAX, TMIN and PRCP), and the `chain` is in the fact (the fold must equal the state). |
| SD-TW-b-10 | **Lifecycle.** (a) `react(weather-configured)` starts the climate Process with no `today` (state `Unstarted`), open-ended. It refuses `weather-configured-twice`. (b) `react(day-began)`: when the climate Process is absent, return nothing (weather is unconfigured); otherwise compute the day from the date, the events, the chain and the draws, and emit `weather-day`. (c) `react(weather-day)` folds `today`, `chain` and `now = 0`, and emits `weather-changed` for hour 0 at once if its condition differs from the condition in force (or if none is in force yet, on day 0). It reschedules the Process to the next hour whose condition differs, or leaves it open-ended. (d) `wake`: when the hour due differs from `now`, emit `weather-changed` and reschedule. (e) `react(weather-changed)` folds `now`. State is written only in `react` from fact payloads; `wake` only states facts and reschedules (calendar's discipline, §16.9 C3 review). |
| SD-TW-b-11 | **Disclosure.** For a subject equal to the observer's place: `weather-today` (the 24 hours and the summary) and `weather-now` (`{ hour, condition }`). Nothing for any other subject, and nothing for an observer in no place (as §16.8 (c)). |
| SD-TW-b-12 | **Market Town opts in** in C4. `configure/weather.yaml` takes the provisional San Diego rules (§17.4), derived from E-TWb-pre-1 by the formulas written in the file's header, so a reader can re-derive every number. `systems:` gains `weather` after `calendar`; `configure:` becomes `[calendar, weather]`. The market-town digest changes by design, and its new value is recorded as the baseline. |
| SD-TW-b-13 | **ARC-68** lands in C1, covering both packs (§15.1 text, extended). Its reuse paragraph records WGEN-lite as built from the published algorithm (Richardson 1981; Richardson & Wright 1984), LARS-WG rejected (non-commercial licence), ClimGen not pursued, and plain monthly tables kept as the degenerate case (§3.3). The data-source and fetch-tool records stay in DEP-31 (TW-d). |

## 17.4 Configuration schema (TW-b)

```yaml
# configure/weather.yaml — Market Town, TW-b (provisional; TW-d replaces the table with the fit and
# switches `source` to the record)
source: rules                 # TW-b: only `rules`
seed: 19                      # content, not --seed (§6.2); u64
rules:
  months:                     # exactly 12 entries, January first
    - p_wet_after_dry: 147    # per mille, 0 … 1000
      p_wet_after_wet: 447    # per mille, 0 … 1000
      rain_tenth_mm: [3, 20, 50, 110, 300]   # 5 ascending integers ≥ 3 (quintile amounts, 0.1 mm)
      tmax_dc: 191            # mean daily maximum, 0.1 °C, −900 … 600
      tmin_dc: 102            # mean daily minimum, 0.1 °C, < tmax_dc
      t_noise_dc: 20          # 0 … 200
      t_ar_permille: 600      # 0 … 999
      wet_tmax_shift_dc: -20  # −200 … 200
      fog_permille: 60        # 0 … 1000
      thunder_permille: 20    # 0 … 1000, applies on wet days
      overcast_morning_permille: 150   # 0 … 1000, applies on dry days
      wind_dms: 30            # 0.1 m/s, 0 … 1000
      wind_from_deg: 290      # 0 … 359, the direction the wind comes from
    # … eleven more
```

The provisional numbers are derived as follows. The header of the committed file states the formulas, and
C4 records the inputs.
- `tmax_dc` and `tmin_dc` are the normals converted with `round((°F − 32) × 50 / 9)`.
- The wet-day frequency is `π = days ≥ 0.01 in / days in month`.
- `p_wet_after_dry` and `p_wet_after_wet` assume a lag-1 persistence `r = 300‰`, so
  `p_wet_after_dry = π × (1000 − r)` and `p_wet_after_wet = p_wet_after_dry + r`.
- The other fields (amounts, noise, fog, thunder, overcast, wind) are provisional design values, stated as
  such, and TW-d's fit replaces them. Their only acceptance claims are their bounds and the frequency check
  (§17.6 criterion 2), which uses this table's own stationary probability.

Refusals arise while decoding and each names its key:
- not 12 months;
- a probability out of range;
- `rain_tenth_mm` not 5 ascending values ≥ 3;
- `tmin_dc ≥ tmax_dc`;
- any bound above;
- an unknown key;
- `source` other than `rules`.

## 17.5 Commit plan

### C1 — `docs: ARC-68 calendar and weather are System Packs; weather pack spec`

1. **Goal.** The decision record and the pack's specification exist before the code they govern
   (`CLAUDE.md` §2.2).
2. **Scope.** `docs/DECISIONS.md` gains ARC-68 (§15.1 text plus SD-TW-b-13's reuse paragraph). New:
   `systems/weather/README.md` (short, human), `systems/weather/Cargo.toml` (doc-only, no dependencies:
   TWa-D1's precedent, because the `systems/*` glob refuses a member without a manifest), and
   `systems/weather/src/lib.rs` (the spec header only: §17.3's table as a module doc). There is no code.
   ARC-67's "Relates to" already names ARC-68, so it is not edited.
3. **Implementation.**
   - [ ] ARC-68 appended after DEP-30, with date, approval, design pointer, choice, alternatives and
     reuse.
   - [ ] The README and the lib.rs spec header.
   - [ ] The doc-only manifest.
4. **Validation.**
   - [ ] `python3 scripts/check_doc_headings.py` passes.
   - [ ] `python3 scripts/check_decision_ids.py` passes, with the id count one higher than before.
   - [ ] `cargo check -p mineworld-weather` is clean.
5. **Acceptance.** Both checks print their success lines. ARC-68 is cited by the README and the lib.rs
   header.
6. **Failure cases.** An id collision means the coordinator re-assigns: stop and report.
7. **Review.**
   - [ ] Terminology is checked against CORE_CONCEPTS: `Process`, `Event` and `System Pack` are used as
     defined, and "climate" is a Process type name, not a new ontology term.
   - [ ] ARC-68 does not restate DEP-31's content.
8. **Boundary.** Docs and an empty crate only.

### C2 — `weather: the generator and the hours, in integers`

1. **Goal.** The climate model is pure, deterministic and integer-only, and it is tested on its own,
   before any world exists.
2. **Scope.** New files `src/{rules,draw,generate,hours,day}.rs`; the crate's real manifest
   (SD-TW-b-1); `src/lib.rs` exports.
   - `rules.rs`: `Rules`, `Month` and their `TryFrom` validation.
   - `draw.rs`: `mix`, `draw`, `permille`.
   - `generate.rs`: `fn day(rules, seed, day_index, date, chain) -> (DailyWeather, Chain)`.
   - `hours.rs`: `DIURNAL`, `WET_HOURS`, `fn hours(summary, events, seed, day_index) -> [WeatherHour; 24]`
     and the classification.
   - `day.rs`: `Condition`, `WeatherHour`, `DailyWeather`, `Chain`, `WeatherDay`.

   Nothing from world assembly is in this commit.
3. **Implementation.**
   - [ ] The types, with `Condition` as a closed enum (§6.6).
   - [ ] `mix` restated with the paced controller's constant, with a doc citation.
   - [ ] `permille` as a multiply-shift.
   - [ ] The generator, per SD-TW-b-5.
   - [ ] The hours, per SD-TW-b-8.
   - [ ] `DIURNAL` precomputed, with its formula and citation in the comment.
4. **Validation.** Unit tests in `src/*/tests.rs`, each owning one failure class:
   - [ ] (a) `mix` pinned to SplitMix64 reference outputs. These are the first three outputs from seed 0
     of the reference `splitmix64.c` (Vigna, <https://prng.di.unimi.it/splitmix64.c>), recorded with the
     URL. The test owns the failure "the restated mixer drifted".
   - [ ] (b) **Frequency, criterion 2.** Over 100 world years (36 524 days) with the provisional
     San Diego table and seed 19, each month's wet-day frequency is within ±3 points of its stationary
     `p_wd / (1000 − p_ww + p_wd)`.
   - [ ] (c) **Spell length, criterion 3.** With a constant table (every month `p_wd = 200`,
     `p_ww = 500`) over 100 years, the mean wet-spell length is within ±10 % of `1000 / (1000 − p_ww)`
     = 2.0. Mutation **M-TWb-3**: the chain ignores yesterday (it always uses `p_wd`). The mean must
     fall to about 1.25, so (c) fails.
   - [ ] (d) Determinism: the same `(rules, seed, day, chain)` gives the same bytes twice. Seed 19 and
     seed 20 give different 365-day sequences.
   - [ ] (e) Hours: over many generated days, `max T(h) == TMAX` and `min T(h) == TMIN`, and the hourly
     precipitation sums exactly to PRCP.
   - [ ] (f) The classification boundaries 10 / 11 and 76 / 77 tenths mm/h. A thunder wet hour is
     `thunderstorm`. A fog day is fog from civil dawn to 09:59. Polar `None` events fall back to the
     stated hours.
   - [ ] (g) Rules refusals: each bound in §17.4, through serde-saphyr, with line and column. LF and CRLF
     input give equal results.
   - [ ] (h) No float: a test scans `systems/weather/src/**/*.rs` for `f32` and `f64` and fails on any
     occurrence (INV-TW-5 for this pack).
5. **Acceptance.**
   - (b): each of the 12 months is within ±3.0 points.
   - (c): the mean is in [1.8, 2.2], and M-TWb-3 is observed red, by name.
   - (a): equal to the reference.
   - (e): exact equality over every day of the 100-year run.
6. **Failure and edge cases.**
   - `p_ww = 1000` gives an absorbing wet chain. This is legal content and is tested as such ("always
     rainy town", §3.3).
   - `p_wd = p_ww` gives independent days (the plain table).
   - TMIN ≥ TMAX after noise is repaired by lowering TMIN, and tested.
   - `i64` overflow is impossible within the stated bounds, which the review checks.
7. **Commands.**
   - `cargo test -p mineworld-weather --lib`.
   - `cargo clippy -p mineworld-weather --all-targets --all-features -- -D warnings`.
   - The mutation is recorded as M-TWb-3; afterwards `git grep MUTATION -- '*.rs'` is empty.
8. **Review.**
   - [ ] Rounding direction is stated everywhere.
   - [ ] Draw indices are never reused.
   - [ ] The quintile index is never 5.
   - [ ] The remainder in the wet-hour division is placed deterministically.
   - [ ] Sources are cited in code (Parton & Logan; AMS; WMO okta; Richardson).
   - [ ] Nothing reads the host, the scale or pause (INV-TW-2).
9. **Boundary.** A pure library: no `System` impl, no registry line.

### C3 — `weather: configuration, facts, the climate process, disclosure`

1. **Goal.** The pack runs in a world: configured at genesis, a day each midnight, changes at hours, and
   disclosed to observers.
2. **Scope.** `src/{configuration,event,process,component,system}.rs`; the registry line and the
   installed `Cargo.toml` line; tests `tests/{weather,configuration}.rs` and `tests/support/mod.rs`
   (calendar's test support pattern).
3. **Implementation.**
   - [ ] `WeatherConfiguration` (SD-TW-b-3) and `WeatherConfigured` (SD-TW-b-4).
   - [ ] `WeatherDay` and `WeatherChanged` events.
   - [ ] `ClimateProcess` and `ClimateState`.
   - [ ] `WeatherToday` and `WeatherNow` components.
   - [ ] `WeatherSystem`: `PackConfiguration` + `configures!()`, then `declaration`, `install`, `react`,
     `wake` and `discloses` per SD-TW-b-2, -10 and -11.
   - [ ] The registry line `Weather => mineworld_weather::WeatherSystem`.
4. **Validation.** Integration tests over a test world with presence, calendar and weather:
   - [ ] (a) Genesis order: `calendar-configured`, `weather-configured`, then `day-began` (day 0), then
     `weather-day` (day 0). The first `weather-changed` (hour 0) is at instant 0.
   - [ ] (b) 30 days: 31 `weather-day` (TWa-D5: the final instant is a midnight). Each `weather-changed`
     is at `day_start + 3600 h`, its condition differs from the one before it, and the sequence equals
     the one derived from the days' `hours`.
   - [ ] (c) **Restart** stopped at day 2 14:30, resumed and run to day 8, gives facts byte-identical to
     the uninterrupted world, and the resumed climate state equals the fold.
   - [ ] (d) **Criterion 4:** a World Pack enabling `weather` without `calendar` is refused at assembly
     with `SystemDependencyDisabled` or `SystemDependencyMissing`, naming `weather` and `calendar`
     (through `WorldPack::read` / assemble).
   - [ ] (e) Unconfigured: `weather` enabled with no `configure/weather.yaml` produces no weather fact and
     no record, and calendar's facts are unchanged.
   - [ ] (f) Disclosure: an observer in a place gets exactly `[(place, weather-today), (place,
     weather-now)]`. An observer in no place gets none, and gets both once placed.
   - [ ] (g) **INV-TW-10 (criterion 1b):** over 30 days, the sequence of calendar-owned facts' `(at,
     event_type, visibility, subjects, payload bytes)` is equal with and without `weather`. Event ids
     are excluded: they are a world counter shared by all packs, and their shift is expected.
   - [ ] (h) The climate state budget is ≤ 8 KB encoded (SD-TW-b-6).
   - [ ] (i) Configuration refusals through `WorldPack::read`, each naming file, line and column, and key.
     These are `source: record` and three bound violations.
   - [ ] (j) `cargo test -p mineworld-installed-systems` passes (the registry consistency test).
5. **Acceptance.** Every check above is observed with the named facts and their counts recorded in
   evidence, not inferred from the exit code.
6. **Failure cases.**
   - `day-began` before the climate Process exists. This is impossible by §17.2's generation order, and a
     test proves the order. If it ever happens, the day is skipped and nothing is refused, and the review
     confirms there is no path to it.
   - A `weather-changed` folded with no `today`: refused as `weather-unstarted`, an internal defect.
   - A `wake` with nothing due: it reschedules and emits nothing.
7. **Commands.**
   - `cargo test -p mineworld-weather`.
   - `cargo test -p mineworld-installed-systems -p mineworld-worldpack`.
   - `cargo test -p mineworld-acceptance` (AC-1 still 14/14 because no world has changed yet).
   - clippy on weather and installed.
8. **Review.**
   - [ ] Single ownership: only weather writes the climate state, and it reads calendar only through the
     `day-began` payload.
   - [ ] No host state is read.
   - [ ] The Process state is a fold of facts.
   - [ ] Public facts have no subjects.
   - [ ] The configured fact is SystemInternal with no subjects.
   - [ ] Dependency direction weather → calendar → presence, never the reverse (INV-TW-10).
9. **Boundary.** No world is changed. No acceptance test is edited.

### C4 — `worlds: market-town has San Diego's weather (rules)`

1. **Goal.** The default town has weather, and AC-1 still measures the market exactly.
2. **Scope.**
   - `worlds/market-town/configure/weather.yaml` (§17.4, with the derivation header).
   - `world.yaml`: `systems:` gains `weather` after `calendar`, with a comment in calendar's style;
     `configure: [calendar, weather]`.
   - `worlds/market-town/README.md`: one line, only if it lists the town's packs.
   - `tests/acceptance/tests/ac1_composability.rs`: `GENERIC_PACKS = ["calendar", "weather"]`, plus its
     unit test's allowed shape if that test enumerates the list.
3. **Implementation.**
   - [ ] Re-read E-TWb-pre-1 and record it.
   - [ ] Compute the table by the header's formulas.
   - [ ] Write the files.
4. **Validation.**
   - [ ] (a) **CP-TW-b** (§17.6).
   - [ ] (b) **INV-TW-1:** the social-cafe 300 d seed 7, bodies-yard 30 d, long_run and long_run_objects
     digests equal `main`'s (E-TWa-9 values, re-captured on the base first as E-TWb-0), and
     `validate` of social-cafe and bodies-yard is unchanged.
   - [ ] (c) The new market-town 300 d seed 7 digest is recorded as the baseline.
   - [ ] (d) **Weather moves nobody (criterion 5):** in market-town 300 d seed 7, the sequence of facts
     *not* owned by `weather`, compared by `(at, event_type, visibility, subjects, payload)`, equals the
     TW-a baseline world's. This is measured from `inspect` output of two saves, or by a test-only
     comparison, and every non-weather fact count equals E-TWa-9's.
   - [ ] (e) AC-1: 14/14. Two mutations on the real `world.yaml`: **M-TWb-A1** puts `weather` before
     `calendar` but still after the six (it must stay green: order among generic packs is free) and is
     recorded. **M-TWb-A2** removes `"weather"` from `GENERIC_PACKS`, and check 3 must fail naming
     `weather`.
5. **Acceptance.**
   - (b): all digests are byte-equal to E-TWb-0.
   - (d): equality holds. If it fails, a controller or rule reads weather records or is perturbed by
     weather facts. That is **material stop (6)**: report it with the first diverging fact, and do not
     re-baseline.
6. **Failure cases.**
   - A digest other than market-town's changes: material stop (4).
   - A 300-day run with zero wet days in a winter month is not a failure by itself: the statistical claims
     are C2's. CP-TW-b's spell claim is over 120 days and is fixed in §17.6.
7. **Commands.** `target/debug/mineworld run worlds/<w> --headless --seed 7 --days 300` per world, with
   TW-a's capture method (sha-256 of every output line but `wall`, by an untracked script under
   `target/tw-b/`), and `cargo test -p mineworld-acceptance --test ac1_composability`.
8. **Review.**
   - [ ] Content only plus the one allow-list word.
   - [ ] The derivation header lets a reader recompute every normals-derived number.
   - [ ] Provisional values are labelled as such.
9. **Boundary.** One world and one allow-list entry.

### C5 — `docs(plan): TW-b evidence`

- [ ] Ledger, handoff (`handoff-tw-b.md`), PR opened as **READY FOR OPERATOR REVIEW** after the full
  gate and CI on the exact head.
- [ ] Gate (classified from output):
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  - `cargo test --workspace --no-fail-fast`;
  - both doc checks;
  - `check_ci_pins.py`;
  - `check_scratch.py`.
- [ ] Review: N/A for the ledger itself, which is records only.

## 17.6 Acceptance and adversarial criteria (fixed before measuring)

**CP-TW-b**, revising §11.2. The 30-day spell claim moves to 120 days, because a provisional October–November
San Diego table can produce no wet day in 30 days with any given seed, so "the 30 days contain wet and dry
spells" could fail by chance rather than by defect.
- `mineworld run worlds/market-town --headless --seed 1 --days 120 --save D`, run twice into two
  directories, gives identical output lines (but `wall`) and identical fact sequences.
- The 121 `weather-day` facts are dated 2026-10-08 … 2027-02-05.
- At least one wet day and at least one dry run of ≥ 7 days occur.
- Every `weather-changed` matches its day's hours.
- D resumed from day 60 to day 120 equals the uninterrupted run (`replay` reproduces every fact and
  snapshot byte for byte).

**Adversarial criteria**, as §11.2 with the corrections stated:

| # | Criterion | Owner | Mutation |
| --- | --- | --- | --- |
| 1a | INV-TW-1: social-cafe, bodies-yard, long_run, long_run_objects digests and validate outputs equal `main`'s | C4 (b) | — |
| 1b | INV-TW-10: calendar facts identical with and without weather (ids excluded, §17.5 C3 (g)) | C3 (g) | **M-TWb-1**: compare *including* event ids. The comparison must then fail, which proves the test observes the interleaving of weather facts. The structural guard, that calendar subscribes to nothing of weather and does not depend on it, is a C3 review item |
| 2 | Monthly wet-day frequency within ±3 points of the stationary probability over **100 years** (36 524 days). §11.2 said 3 650 days, but at about 300 days per month with persistence the standard error is about 3 points, so ±3 would fail by chance about one month in three. At 100 years it is about 1 point, so ±3 is about 3σ. | C2 (b) | **M-TWb-2**: use `p_ww` for both branches. The frequencies move far outside ±3 and (b) fails |
| 3 | Mean wet-spell length within ±10 % of `1000/(1000 − p_ww)` on a constant table | C2 (c) | **M-TWb-3** (chain ignores yesterday) must fail |
| 4 | `weather` without `calendar` is refused at assembly | C3 (d) | **M-TWb-4**: drop `CalendarSystem::ID` from `depending_on`. (d) must fail, or the world must assemble and fail later. Recorded either way, because it establishes the guard |
| 5 (new) | Weather moves nobody: non-weather facts of market-town 300 d equal TW-a's baseline | C4 (d) | — (a failure is a material stop, not a mutation) |
| 6 (new) | Integers only: no `f32`/`f64` token in `systems/weather/src` | C2 (h) | **M-TWb-6**: insert `let _x: f64 = 0.0;`. The scan must fail |

## 17.7 Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-TWb-1 | IL-b merges during TW-b, so `seed` gains `&ConfigurationContext`. | Merge `main` and adapt `weather`'s `seed` (a mechanical, bounded change). If IL-b merges after TW-b, IL-b adapts `weather` as it does calendar (step-18 §12.6). |
| R-TWb-2 | Criterion 5 fails: the paced controller's choices depend on the content of observations (for example a hash over records), so adding records changes NPC behaviour. | Material stop. Calendar's evidence (E-TWa-5: every other fact count equal) suggests it holds, but counts are weaker than payloads. That is why the criterion is fixed now. |
| R-TWb-3 | The provisional table's non-normal fields (amounts, fog, overcast, wind) are unrealistic. | They are labelled provisional. TW-d replaces them with the fit from the record, and the fit's own criterion is TW-d's #5. |
| R-TWb-4 | Disclosure size: `weather-today` (24 hours) on every observation. | About 2 KB per observation, the same order as calendar's 97-sample track. S11-C's delta stream sends it once per day. Measured in C3's evidence. |
| R-TWb-5 | Platforms: no macOS or Windows CI. | Integer-only arithmetic (criterion 6), no paths beyond `Path::join` in tests, DEP-29 scratch, no signals. Local runs on macOS plus CI on Linux are recorded. Windows is argued from the integer-only design and the CRLF test (C2 (g)), and is not claimed as run. |
| R-TWb-6 | Snapshot growth (F-TWbd-1). | Climate state ≤ 8 KB, pinned by C3 (h). |

## 17.8 Findings recorded at planning

```text
F-TWbd-1  (pre-existing, persistence lane) Every snapshot is a full world state and all are retained
          (persistence/src/world.rs DEFAULT_SNAPSHOT_INTERVAL = 64; sqlite.rs INSERT OR IGNORE). Measured
          2026-10-09 on origin/main @ f80bbb7 (dev build, macOS arm64): `mineworld run worlds/market-town
          --headless --seed 1 --days 30 --save target/plan-tw-bd/s30` → 38 055 facts, world.sqlite
          266 342 400 bytes (about 0.5 MB per snapshot; estimated about 450 snapshots). A 300-day save is
          therefore of the order of 2.5 GB today, before S19. S19's packs add to every snapshot: calendar's
          day record (about 2 KB), weather's climate state (≤ 8 KB, SD-TW-b-6) and TW-d's record series
          (§18.3 SD-TW-d-5). §13 R-TW-4's "~40 KB fact" estimate counted the fact log only and missed
          the snapshot multiplier. Not S19's to fix (persistence owns retention); reported for the
          primary session to route to the S6 lane (QTWd-2).
F-TWbd-2  `mineworld check` does not exist on main (tools/cli/src/main.rs subcommands: server,
          validate, replay, create, install, add-system, inspect, run, biography, packs). QTW-8's warning
          (enabled without its configure file) and §6.4's 1 MB station-file warning have no host;
          IL-b's 4 MiB hard cap (ATTACHMENT_MAX_BYTES) is the only size guard. QTWb-5.
```

## 17.9 Questions (QTWb-n). **[OPERATOR]** marks operator-material ones.

| Id | Question | Recommendation |
| --- | --- | --- |
| QTWb-1 | Where the rules table lives: inline under `rules:` in `configure/weather.yaml` (IL-a's seam as it is), or a separate YAML file (§6.2 sketched `rules/san-diego.yaml`, which IL-a cannot read; as an IL-b attachment, the pack would parse YAML bytes itself and lose the loader's line and column)? | **Inline.** One file per key, positioned refusals, and drift checked through `weather-configured`. TW-d's tool writes the fitted block for this file (§18). |
| QTWb-2 | Criterion 2's sample: 100 years instead of §11.2's 3 650 days (statistical reason in §17.6). | **Yes.** It is a pure-function test and costs well under a second. |
| QTWb-3 | Market Town opts into rules-driven weather in TW-b with a provisional table (normals-derived temperatures and wet frequencies, other values provisional), rather than waiting for TW-d. | **Yes.** It gives the clients (TW-e, TW-f) weather to render earlier, and TW-d switches the source in one content commit. |
| QTWb-4 | ARC-68 carries WGEN-lite's reuse verdict (built from the published algorithm), and DEP-31 (TW-d) carries data, CSV and the fetch tool's HTTP client. Is that split acceptable, given §15.1 put the generator in DEP-31? | **Yes.** DEP-31 is assigned to TW-d's data. A build decision must be recorded before the code that embodies it (REUSE_POLICY), and TW-b embodies the generator. |
| QTWb-5 | QTW-8's `mineworld check` warning has no command to live in (F-TWbd-2). Defer it to S16's CLI work, or add a warning to `validate` in TW-b? | **Defer** to S16. TW-b does not touch the CLI. |
| QTWb-6 | The hour-derivation constants (`DIURNAL`, Parton & Logan; `WET_HOURS`, a design default; AMS intensity thresholds; okta mapping) are pack constants with citations, not content. Should they become content in the rules file now? | **Constants now.** They are physics-like defaults with sources, and the rules file stays the climate. Making them content is an additive change if a world needs it. |
| QTWb-7 | The provisional persistence `r = 300‰` used to split normals into `p_wd`/`p_ww` (§17.4). | **Accept as provisional.** TW-d's fit replaces it with measured transitions. |

None of TW-b's questions is operator-material: each stays within §14.1's rulings and ARC-61/ARC-35 as noted.

### 17.9.1 Rulings, 2026-10-09 (primary session; binding)

| Id | Ruling |
| --- | --- |
| QTWb-1 … QTWb-7 | Accepted as recommended. |
| IL-b's `seed` signature | Whichever of TW-b and IL-b merges second adapts the other's implementors (R-TWb-1). IL-b is close to merging, so TW-b should expect to write the three-argument `seed`. |
| F-TWbd-1 | Routed to the persistence lane as **F-SAVE-1**: snapshot retention, with saves growing about 2.5 GB per 300 days. The primary session opens a design for it. The climate state stays at or below 8 KB, as designed. |

## 17.10 Execution contract (frozen 2026-10-09)

```text
PROJECT / PR            S19 TW-b — the `weather` System Pack (seeded rules)
PRIMARY DESIGN DOC      .structured-coding/plans/mvp0/step-19-time-weather.md §17 (this section; live ledger §17.11)
RELATED / BINDING DOCS  CLAUDE.md; step-19 §§4–6, §10 (INV-TW-1 … 10), §11, §14.1; §16 (TW-a, merged);
                        docs/DECISIONS.md ARC-26, ARC-28, ARC-33, ARC-35 (+ 2026-10-09 note), ARC-61,
                        ARC-67, DEP-30; docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md;
                        docs/CORE_CONCEPTS.md
IMPLEMENTATION BASE     origin/main at freeze (≥ f80bbb7, TW-a merged). IL-b need not be merged (R-TWb-1).
WORKTREE                /Users/yuema137/mineworld-worktrees/impl-tw-b — held by one session only (CLAUDE.md §3.1)
BRANCH                  mvp0/pr-tw-b-weather, from the base above
APPROVED SCOPE          §17.1. Change set: systems/weather/** (new); systems/installed/{Cargo.toml,src/lib.rs};
                        worlds/market-town/{world.yaml,configure/weather.yaml,README.md};
                        tests/acceptance/tests/ac1_composability.rs (GENERIC_PACKS only); docs/DECISIONS.md
                        (ARC-68); Cargo.lock (workspace member only); this section; handoff-tw-b.md
FROZEN INVARIANTS       INV-TW-1, -2, -3, -5, -10; SD-TW-b-1 … 13; §17.6 criteria and mutations
SEQUENCE                C1 → C2 → C3 → C4 → C5 (§17.5)
ALLOWED COMMANDS        cargo * (incl. $HOME/.cargo/bin/cargo); git; gh (PR create/view/edit, never merge);
                        python3 scripts/*; mkdir -p; sed -n; the CLI binary under target/; WebFetch for NOAA
                        normals only
NOT ALLOWED             python3 -c, sed -i, awk, xargs, curl, heredoc writes; edits outside the change set;
                        .claude/settings*; other worktrees
AUTHORITY               commit and push to the branch; open the PR; mark READY FOR OPERATOR REVIEW; never merge
BUDGET                  ≤ 8 town runs of 300 days (E-TWb-0 ×2, C4 ×4, reserve ×2); unit statistics unrestricted;
                        full workspace test ≤ 2 runs
MATERIAL STOPS          (1) an edit needed to the kernel, contracts, presence, persistence, server, clients,
                        calendar or any other pack; (2) any new external dependency; (3) IL-a's/IL-b's API
                        must change; (4) any digest other than market-town's changes; (5) a criterion in
                        §17.6 fails after a correct implementation; (6) criterion 5 fails; (7) AC-1 needs
                        more than the allow-list word
GATE                    cargo fmt --all --check; cargo clippy --workspace --all-targets --all-features -- -D
                        warnings; cargo test --workspace --no-fail-fast; check_doc_headings.py;
                        check_decision_ids.py; check_ci_pins.py; check_scratch.py — each PASS / FAIL /
                        INCONCLUSIVE from output, never from the exit code alone; CI on the exact head
HANDOFF                 .structured-coding/plans/mvp0/handoff-tw-b.md
```

## 17.11 Ledger

Execution session: worktree `/Users/yuema137/mineworld-worktrees/impl-tw-b`, branch `mvp0/pr-tw-b-weather`,
base `origin/main @ a454e37` (#107 merged; TW-a on main). IL-b (#102) was open at the start. Handoff:
[`handoff-tw-b.md`](handoff-tw-b.md).

**Start-of-session audit (2026-10-09).** §17.2's anchors re-verified on a454e37: calendar's
`system.rs` shape (`PackConfiguration` + `configures!()`, `react` starts the Process from the configured
fact and folds its own facts, `wake` only reschedules and states facts, `discloses` keyed to `Presence`);
`DayBegan { day, phase }` with `CalendarDay::{date, weekday, day_start, events, track}` public and
`DayEvents` fields public `Option<u32>`; `kernel/src/registry.rs` `check_dependencies` l. 212–228
(`SystemDependencyMissing` / `SystemDependencyDisabled`); `authoring/src/configuration.rs`
`seed(&Seeding, &C)` (two arguments on main at the branch point); `systems/installed` one line each, with
`mineworld-calendar` a path dependency (not a workspace dependency); AC-1 `GENERIC_PACKS: [&str; 1] =
["calendar"]` at l. 1030; `cognition/rule-controller/src/paced.rs` `mix` at l. 312–318;
`worlds/market-town/world.yaml` `calendar` last, `configure: [calendar]`; `docs/DECISIONS.md` ends at
DEP-30 with 74 ids. All as recorded.

### Commit ledger

| # | Implementation | Deterministic validation | LLM logic review |
| --- | --- | --- | --- |
| C1 | [x] ARC-68 appended after DEP-30 (date, approval, design pointer, choice 1–8, alternatives, reuse table WGEN / LARS-WG / ClimGen / plain tables, defaults with sources, limitations); `systems/weather/README.md`; `src/lib.rs` = the spec header (§17.3 as a module doc); doc-only `Cargo.toml` (TWa-D1's precedent) | [x] `check_doc_headings.py`: 191 sections, none duplicated — PASS; `check_decision_ids.py`: 75 ids (74 + 1), all distinct — PASS; `cargo check -p mineworld-weather` clean; Cargo.lock gains the member only | [x] terms against CORE_CONCEPTS: `System Pack`, `Process`, `Event` (facts), `Component` used as defined; "climate" is a Process type name, "WGEN-lite" a generator name, neither an ontology term. ARC-68 names DEP-31 as TW-d's and does not restate data, CSV or fetch content |
| C2 | [x] `day.rs` (`Condition` closed, kebab-case; `WeatherHour`, `DailyWeather`, `Chain`, `Origin::Rule`, `WeatherDay`); `draw.rs` (`mix` restated with the paced controller's constants and Vigna's reference cited; `permille` multiply-shift; `draw`; fixed indices 0…6, 16, 17); `rules.rs` (`Rules`/`Month`, decode-is-validate, every refusal naming its key); `generate.rs` (`day` per SD-TW-b-5, `weather_day` composing a calendar day → `WeatherDay`); `hours.rs` (`DIURNAL`, `WET_HOURS`, AMS thresholds, okta mapping, `hours`); real manifest (SD-TW-b-1) | [x] E-TWb-2: `cargo test -p mineworld-weather` lib 15 + diurnal 1 + no_float 1, all pass; clippy `-p mineworld-weather --all-targets --all-features -D warnings` clean; fmt clean. Mutations M-TWb-2, -3, -6 killed (below); `git grep MUTATION -- '*.rs'` empty | [x] rounding stated (i64 division toward zero, documented in `generate.rs`; T(h) floor with range > 0, `hours.rs`); indices are distinct constants in one module, never reused; quintile = u·5/1000 ≤ 4 since u ≤ 999; the remainder goes to the first wet hour, deterministic; sources cited in code (Richardson; Parton & Logan; AMS drizzle / rain; WMO okta; Vigna); nothing reads host, scale or pause — the inputs are the rules, the seed, the day index, the month and the light events; overflow: every i64 product ≤ 10^4·10^3, every narrowing bounded by validation or `ANOMALY_LIMIT_DC` (TWb-D2) |
| C3 | [x] `configuration.rs` (`WeatherConfiguration { source, seed, rules }`, `deny_unknown_fields`; `source: record` refused with SD-TW-b-3's message); `event.rs` (`WeatherConfigured { seed, rules, record: Option<RecordRef> }` with `RecordRef` an empty enum; `WeatherDay` is itself the `weather-day` event; `WeatherChanged { hour, condition, cloud_oktas, precipitation_tenth_mm }`); `process.rs` (`ClimateProcess` "climate", `ClimateState { configured, today: Option, now, chain }`); `component.rs` (`weather-today`, `weather-now`); `system.rs` (`PackConfiguration` + `configures!()`; `depending_on([presence, calendar])`; `react` and `wake` per SD-TW-b-10; `discloses` per SD-TW-b-11); registry line `Weather => mineworld_weather::WeatherSystem` plus its Cargo line. Separate commit: the placeholder `weather` in two loader tests renamed (TWb-D9) | [x] E-TWb-3: `cargo test -p mineworld-weather` lib 15, configuration 2, diurnal 1, no_float 1, weather 8, all pass; `-p mineworld-installed-systems -p mineworld-worldpack` all pass (after TWb-D9); `-p mineworld-cli --test configure --test commands` pass; `-p mineworld-acceptance` E-TWb-3b; clippy on weather and installed `--all-targets --all-features -D warnings` clean; fmt clean. M-TWb-1 held as a permanent control; M-TWb-4 killed | [x] single ownership: only weather starts, reschedules or writes the climate Process (`ProcessKind::Owner`), and it reads calendar only through `day-began`'s payload (`DayBegan::day()`), never calendar's Process or components. No host state, scale or pause is read (inputs: facts, `world.at()`, Presence for disclosure). State is written only in `react` from fact payloads (`with_day`, `with_now`), and `wake` only reschedules and states facts, so a snapshot equals the fold (restart test byte-identical). Public `weather-changed` has no subjects; `weather-configured` and `weather-day` are SystemInternal with none. Dependency direction: weather → calendar → presence; calendar names nothing of weather (its crate does not depend on weather and its declaration subscribes to nothing of weather), so INV-TW-10 holds structurally as well as by C3 (g). The "day-began before the climate exists" path: genesis records both configured facts in generation 0 and `day-began` is generation 1 (C3 (a) observes the order), and later `day-began`s come from calendar's wake after genesis. If it ever happened, `react` returns nothing (treated as unconfigured), not a refusal. A `weather-changed` with no `today` is refused `weather-unstarted`; a `wake` with nothing due reschedules and states nothing |
| C4 | [x] 0728fa9: `configure/weather.yaml` (§17.4 table from E-TWb-1, derivation header with inputs and formulas, provisional values labelled); `world.yaml` `weather` after `calendar`, `configure: [calendar, weather]`; README one line; `GENERIC_PACKS = ["calendar", "weather"]`; the AC-1 unit test's foreign pack `weather` → `bodies`, plus the allowed shape for both generic packs in either order; ARC-35 note (2026-10-09, TW-b) as the kickoff asked; the weather unit tests read the world's own file (one copy of the table). 52a3c1c merges origin/main (IL-b #102, 13b #103, P3 #98); 0599b2b adapts `seed` to IL-b's third argument. The opt-in tests `tests/moves_nobody.rs` (criterion 5 and its diagnostic) and `tests/checkpoint.rs` (CP-TW-b) read saves | [x] E-TWb-0 (base), E-TWb-4 (INV-TW-1 and the new baseline), E-TWb-5 (CP-TW-b), E-TWb-6 (criterion 5: **FAIL as frozen**; material stop (6), TWb-F1). AC-1 14/14; M-TWb-A1, M-TWb-A2 as ruled. Town runs 6 of 8 | [x] content plus the allow-list word, the unit test's example and the ARC-35 note; the derivation header lets a reader recompute every normals-derived number (inputs printed, formulas stated, rounding stated); provisional values labelled. TWb-F2: the world must list calendar before weather (kernel install order), though AC-1 is order-free |
| C5 | [x] ledger, handoff, PR #113 opened **not READY** (TWb-F1 open) | [x] full gate E-TWb-7 PASS; CI on the final head is recorded in the PR and the handoff, not here (a commit cannot name its own CI) | — |

### Evidence

```text
E-TWb-1  E-TWb-pre-1 re-read 2026-10-09 with WebFetch:
         https://www.ncei.noaa.gov/data/normals-monthly/1991-2020/access/USW00023188.csv (San Diego Lindbergh
         Fld). Jan … Dec:
           MLY-TMAX-NORMAL °F         66.4 66.2 67.0 68.8 69.5 71.7 75.3 77.3 77.2 74.6 70.7 66.0
           MLY-TMIN-NORMAL °F         50.3 51.8 54.5 57.1 60.0 62.6 66.1 67.5 66.2 61.5 54.8 49.8
           MLY-PRCP-AVGNDS-GE001HI    6.5  7.1  6.2  3.8  2.2  0.7  0.7  0.3  0.9  2.4  3.7  5.8
           MLY-PRCP-NORMAL in         1.98 2.20 1.46 0.65 0.28 0.05 0.08 0.01 0.12 0.50 0.79 1.67
         The first three rows equal §17.2's E-TWb-pre-1. The fourth is also used, for the amounts
         (TWb-D4). The table they give, by §17.4's formulas (February 28 days, TWb-D8):
           tmax_dc  191 190 194 204 208 221 241 252 251 237 215 189
           tmin_dc  102 110 125 139 156 170 189 197 190 164 127  99
           p_wd     147 178 140  89  50  16  16   7  21  54  86 131   (p_ww = p_wd + 300)
E-TWb-2  C2, 2026-10-09, macOS arm64, dev, `cargo test -p mineworld-weather -- --nocapture`: 17 passed.
         (a) mix(0,1..3) = E220A8397B1DCDAF, 6E789E6AA1B965F4, 06C45D188009454F = the reference
             splitmix64.c's first three outputs from seed 0. PASS.
         (b) Criterion 2, 36 524 days, seed 19, the provisional San Diego table: wet ‰ against
             stationary ‰ — Jan 221/210, Feb 268/254, Mar 215/200, Apr 123/127, May 80/71, Jun 20/22,
             Jul 19/22, Aug 9/10, Sep 38/30, Oct 76/77, Nov 126/122, Dec 175/187. Max |Δ| = 15 ‰
             (Mar), bound 30 ‰. PASS.
         (c) Criterion 3, constant table p_wd 200 / p_ww 500: 10 435 wet days in 5 213 spells, mean
             2.00 days (bound 1.8 … 2.2). PASS.
         (d) Same seed gives equal bytes over 365 days; seeds 19 and 20 differ. PASS.
         (e) Over all 36 524 days: max T(h) = TMAX and min T(h) = TMIN exactly, Σ hourly precipitation =
             PRCP exactly, wet-hour count = WET_HOURS(PRCP); 1 000+ wet days exercised. PASS.
         (f) 20 → 2 h drizzle (10/h, boundary); 44 → 4 h rain (11/h); 912 → 12 h rain (76/h, boundary);
             924 → heavy rain (77/h); 913 → the remainder tips only the first hour to heavy rain; thunder
             day → thunderstorm; fog 05–09 (civil dawn 05:23), polar fog 05–09, civil dawn 07:00 →
             07–09; polar minimum at 06:00, maximum at 15:00; San Diego minimum at 05:00. PASS.
         (g) 18 refusals through serde-saphyr, each naming its key and "line 2 … column"; eleven months
             refused naming `months` and 11; unknown keys `gusts`, `seasons` refused; five bound values
             accepted; CRLF = LF; a fact copy with tmin ≥ tmax is refused on decode. PASS.
         (h) no_float: 0 tokens in 12 source files. PASS. diurnal: DIURNAL equals the formula
             recomputed in floating point (outside src/), every row 0 at sunrise and 1000 at 15:00.
         Edge cases: p_ww = 1000 absorbing after the first wet day; p_wd = p_ww → 300 ± 30 ‰ marginal
         and conditional (independent); TMIN repair exercised with tmin 190 / tmax 191, noise 200,
         ρ 999 — tmin < tmax on every day of 100 years.
E-TWb-3  C3, 2026-10-09, macOS arm64, dev, `cargo test -p mineworld-weather -- --nocapture`: 27 passed.
         The test world has presence, calendar (San Diego), weather (a changeable test climate: every
         month p_wd 250 / p_ww 550, fog 300 ‰, thunder 150 ‰, grey mornings 400 ‰) and a placer.
         (a) Genesis at instant 0, in this order: calendar-configured, weather-configured, day-began,
             weather-day, weather-changed (hour 0). PASS.
         (b) 30 days → 31 weather-day at 0, 86 400, … 30 × 86 400, dated 2026-10-08 … 2026-11-07; 83
             weather-changed, each at day_start + 3 600 h, each changing the condition, and the
             sequence equals the one the days' own hours imply (instant, hour, condition, oktas,
             amount). 11 wet days; conditions seen: clear, overcast, fog, drizzle, rain, thunderstorm.
             Process state = the fold (last day, chain, now). PASS.
         Visibility: weather-changed Public; weather-day and weather-configured SystemInternal; no
             subjects on any. PASS.
         (c) Restart stopped at day 2 14:30, resumed, run to day 8: 6 weather-day after the stop; the
             facts after the stop are byte-identical to the uninterrupted world's; the resumed state's
             today, chain and now equal the fold of the facts. PASS.
         (d) Criterion 4, through WorldPack::read + assemble: `systems: [presence, weather]` refused with
             "the world this pack describes cannot be composed: system 'weather' depends on 'calendar',
             which is not installed in this world". The same pack with calendar assembles and its
             genesis carries weather-configured. PASS.
         (e) weather enabled, no configure/weather.yaml: no weather fact, no climate Process, no weather
             record disclosed; calendar's facts (projection without ids) equal a world without weather
             installed. PASS.
         (f) bo (no place) → no weather record; ada → exactly [(square, weather-today), (square,
             weather-now)]; weather-today = day 0's weather-day; weather-now = the last change (hour,
             condition), which is the noon hour's condition; after `place-me` bo gets the same two.
             Disclosure size: weather-today 3 179 B, weather-now 31 B (R-TWb-4 estimated about 2 KB). PASS.
         (g) INV-TW-10: over 30 days the 1 + 31 + 180 calendar facts are equal with and without weather in
             (at, type, visibility, subjects, payload bytes). PASS. Control M-TWb-1 below.
         (h) Largest encoded climate state at 13:00 on each of 30 days: 6 701 bytes ≤ 8 192. PASS.
         (i) Through WorldPack::read: `source: record` → "…/configure/weather.yaml is not a valid
             configuration file: error: line 1 column 9: source `record` needs the record data of TW-d;
             this build accepts `rules`"; p_wet_after_wet 1447 → "line 5 column 7: p_wet_after_wet is per
             mille from 0 to 1000, not 1447"; tmin_dc 300 → "line 5 column 7: tmin_dc must be below
             tmax_dc, not 300 against 191"; rain_tenth_mm of 2 → "line 5 column 68: rain_tenth_mm is 5
             amounts in 0.1 mm, not 2". File (by the platform's separator), line, column and key in
             each. PASS.
         (j) `cargo test -p mineworld-installed-systems -p mineworld-worldpack`: all pass, the registry
             consistency test included. `-p mineworld-cli --test configure --test commands`: 4 + 2 pass.
E-TWb-3b `cargo test -p mineworld-acceptance` on the C3 tree (log target/tw-b/c3-acceptance.log): exit 0,
         ac1_composability 14 passed, and every other target passes; 10 s wall. No world has changed yet.
E-TWb-0  Base capture on a454e37 (clean; the branch's base), dev, by target/tw-b/capture.sh base save
         (untracked; TW-a's method: sha-256 of every `run` output line but `wall`; LONG-RUN payload bytes):
           social-cafe 300 d seed 7   ad49c723…c64b   365 330 facts, faults 0, 15.7 s
           market-town 300 d seed 7   24a95d2a…d270   374 857 facts, faults 0, 21.2 s
           bodies-yard 30 d seed 7    bd6a1002…80e6   62 385 facts
           validate social-cafe / bodies-yard / market-town   ebcd60a0… / 7356b8f8… / 31fb85d4…
           long_run          4 019 622 B   fdcf28d6…4445   (TW-a counted 4 019 632 with prefix and newline)
           long_run_objects    612 410 B   e018c85e…d2be
           market-town 300 d seed 7 --save base/market-save: 374 857 facts, 2 756 976 640 bytes, 39.2 s
         All equal TW-a's E-TWa-9 references. Town runs: 3.
E-TWb-4  Head capture on 0599b2b (C1–C4 + origin/main 551fb2c merged + the IL-b adaptation; clean), dev:
           social-cafe 300 d seed 7   ad49c723…c64b   365 330 facts   = E-TWb-0   PASS (INV-TW-1)
           bodies-yard 30 d           bd6a1002…80e6                   = E-TWb-0   PASS
           validate social-cafe, bodies-yard                          = E-TWb-0   PASS
           long_run, long_run_objects                                 = E-TWb-0   PASS
           market-town 300 d seed 7   90479fd8631a3f9c88ddc9c05720fbf8fd8fbc5b1573d6abd47e7351b9d1ae57,
             375 527 facts = 374 857 + 670 weather (1 weather-configured, 301 weather-day, 368
             weather-changed), faults 0, 27.0 s — the NEW MARKET-TOWN BASELINE (SD-TW-b-12)
           validate market-town 6368595a…0318 (changed by design: 134 genesis facts)
           market-town 300 d seed 7 --save head/market-save: 375 527 facts, 2 872 868 864 bytes, 45.8 s
         Main moved from a454e37 to 551fb2c during TW-b. The unchanged social-cafe, bodies-yard and long_run
         digests on the merged head show that the move did not change the headless path, and IL-b's own
         E-IB-15 records market-town at main equal to 24a95d2a. So E-TWb-0 stays the reference.
         The run summaries of base and head market-town (every line but `wall`) differ only in the
         per-day cumulative totals and the history line, both of which count the weather's own facts.
         Every per-type fact count, every request count and every person's activity line is equal.
         Town runs: 6 of 8.
E-TWb-5  CP-TW-b on 0599b2b + the checkpoint test, 2026-10-09: `run market-town --seed 1 --days 120 --save
         A` and again into B (17.5 s, 17.3 s), output equal but the save path in the header line and
         `wall`; R run to day 60 and resumed to day 120 ("resumed … at revision 58258 (snapshot 58240 + 18
         re-executed)"). tests/checkpoint.rs (opt-in): A, B and R hold the same fact rows byte for byte;
         121 weather-day at 0, 86 400, …, dated 2026-10-08 … 2027-02-05; 16 wet days, longest dry run 16
         days, 112.8 mm in all (normals for that span: about 19 wet days, about 127 mm); 134 weather-
         changed, each as the hours imply. `mineworld replay --save R worlds/market-town`: "116389
         revision(s) re-executed from genesis, 150616 fact(s) and 1819 snapshot(s) reproduced byte for
         byte". PASS.
E-TWb-6  Criterion 5 (weather moves nobody), tests/moves_nobody.rs (opt-in), E-TWb-0's base save vs
         E-TWb-4's head save:
         `weather_moves_nobody` — FAILS as frozen (payload bytes compared): the first diverging non-
           weather fact is #119, at t0, agenda-changed, Participants, [EntityId(7)]: payload
           `{… "label":"home","until":19800,"routine":"2"}` without weather, `"routine":"3"` with.
         `where_the_non_weather_facts_differ` (diagnostic, written after the failure) — over all 374 857
           non-weather facts: instant, type, visibility and subjects are equal for every one; 32 953
           payloads differ, and every differing leaf is a Process id shifted by exactly +1:
             agenda-changed.routine              11 712 of 11 712
             group-activity-started.activity      6 651 of  6 651
             group-activity-ended.activity        6 650 of  6 650
             left-group-activity.activity         4 481 of  4 481
             joined-group-activity.activity       3 459 of  3 459
         Cause: the climate Process is started in genesis generation 0 and takes Process id 2 from the
         world's one Process counter (calendar's is 1). Every routine and group activity started after it
         gets an id one higher. That is the same kind of shift as the event ids that §17.5 C3 (g) already
         excludes; it is not a controller or rule reading weather.
         → TWb-F1, MATERIAL STOP (6). Not re-baselined; the criterion is not edited.
E-TWb-7  Full gate on 5928db2 (clean), macOS arm64, by target/tw-b/gate.sh, 2026-10-09 17:27–17:38:
           cargo fmt --all --check                                         exit 0           PASS
           check_doc_headings.py   192 numbered sections, none duplicated                    PASS
           check_decision_ids.py   84 ids, all distinct                                      PASS
           check_ci_pins.py        toolchain pins agree                                      PASS
           check_scratch.py scan   180 test sources, none outside test-support (2 exempt)    PASS
           cargo clippy --workspace --all-targets --all-features -- -D warnings   exit 0   PASS
           cargo test --workspace --no-fail-fast   exit 0, 641 s: 194 result lines, all ok;
             864 passed, 0 failed, 20 ignored (the opt-in save checks among them)        PASS
           check_scratch.py left --target-dir target   no scratch left                       PASS
         The later commits change only the plan documents. Full workspace test runs: 1 of 2.
```

### Mutations

```text
M-TWb-3  generate.rs: the wet branch uses p_wet_after_dry (the chain ignores yesterday) →
         the_mean_wet_spell_is_the_chains FAILS: "7300 wet days in 5837 spells: mean 1.25 days … outside
         1.8 … 2.2" (as §17.5 C2 (c) predicted, 1.25), and criterion 2 and the absorbing-chain test fail
         too. Killed. Reverted.
M-TWb-2  generate.rs: p_wet_after_wet on both branches → criterion 2 FAILS at January: 1 379 wet of 3 100
         = 444 ‰ against 210 ‰. Killed. Reverted.
M-TWb-6  generate.rs: `let _x: f64 = 0.0;` → no_float FAILS naming generate.rs. Killed. Reverted.
         After all three: `git grep MUTATION -- '*.rs'` empty; 17 passed.
M-TWb-1  Kept as a permanent control in tests/weather.rs `calendars_facts_are_the_same_with_and_without_
         weather`: the same projection *including* event ids is asserted unequal between the two worlds,
         and it is (the weather's facts take ids from the shared counter). So the id-free comparison is
         known to see the interleaving and to exclude only the ids. Observed: PASS of the `assert_ne!`.
M-TWb-A1 world.yaml: `weather` moved before `calendar`, still after the six → AC-1 14/14, green as ruled
         (order among generic packs is free in check 3). But `mineworld validate worlds/market-town` then
         refuses the world: "system 'weather' depends on 'calendar', which is not installed in this world"
         — the kernel installs in `systems:` order and checks dependencies at install (TWb-F2). Reverted.
M-TWb-A2 ac1_composability.rs: `GENERIC_PACKS = ["calendar"]` → check_3_the_world_delta FAILS:
         "world.yaml: `weather` follows the six market packs and is not an allow-listed generic pack
         [\"calendar\"]", "world.yaml: `configure` names `weather`, not an allow-listed generic pack",
         "configure/weather.yaml: not the configuration of an allow-listed generic pack"; the unit test
         fails too (its allowed shape). Killed. Reverted; `git grep MUTATION` empty; 14/14.
M-TWb-4  system.rs: `depending_on([PresenceSystem::ID])` (calendar dropped) →
         weather_without_calendar_is_refused_at_assembly FAILS: "refused: ()" — the world without calendar
         assembled. So the guard is the declared dependency, and the test observes it. Killed. Reverted;
         `git grep MUTATION -- '*.rs'` empty.
```

### Deviations and findings

```text
TWb-D1  (bounded) `rain_tenth_mm` "5 ascending integers ≥ 3" is checked as non-decreasing, each 3 … 10 000.
        Reason: the normals-derived August quintiles are 3, 3, 6, 10, 19 (two amounts at the 0.3 mm
        floor), and a non-decreasing table still gives a well-defined quintile lookup. The upper bound
        keeps the amount in u16. Impact: none on the generator.
TWb-D2  (bounded) The temperature anomaly is clamped to ±400 (40 °C), `ANOMALY_LIMIT_DC`. Reason: at the
        stated bounds (ρ up to 999 ‰, noise up to 200) the AR(1) fixed point is ±200 000, which would
        overflow the i16 `Chain` and the temperatures. Realistic tables stay far inside the clamp: the
        stationary sd is about 0.72 × noise ≈ 2 °C. Impact: deterministic and stated. TMAX stays within
        ±1 200 and TMIN within ±1 300, so both fit i16.
TWb-D3  (bounded) `DIURNAL` is 13 rows of 24, one per sunrise hour 01 … 13, not one 24-entry curve. Reason:
        SD-TW-b-8a's curve must be 0 at the day's sunrise hour, which varies (San Diego 05 … 06), so one
        row cannot hold it. A sunrise outside 01 … 13 is clamped to the nearest (near polar day), and a
        missing sunrise uses 06 as designed. The table is precomputed from the formula in its comment
        (Parton & Logan's sine to 15:00, exponential decay b = 2.2 to the next sunrise), and
        tests/diurnal.rs recomputes it in floating point outside src/ and holds it equal.
TWb-D4  (bounded; realism, operator rule "realistic defaults with sources") The provisional wet-day
        amounts are not free design values. Each month's are derived from the same normals file:
        μ = MLY-PRCP-NORMAL × 254 / MLY-PRCP-AVGNDS-GE001HI (the mean wet-day amount, 0.1 mm), times the
        exponential distribution's quintile midpoints −ln(1 − p) at p = 0.1, 0.3, 0.5, 0.7, 0.9
        (0.105, 0.357, 0.693, 1.204, 2.303), rounded and floored at 3. The other provisional fields are
        labelled design values (C4's file header): t_noise 30 (stationary sd ≈ 2.2 °C), t_ar 600 (WGEN's
        typical lag-1 temperature persistence), wet_tmax_shift −20, and monthly fog, thunder, overcast
        and wind values. TW-d's fit replaces them all.
TWb-D5  (bounded) A fog hour carries 8 oktas (the sky is obscured: WMO's 9 is outside the 0 … 8 field).
        Precedence within an hour is wet, then fog, then cloud.
TWb-D6  (bounded) The no-float scan (C2 (h)) and the DIURNAL formula check live in systems/weather/tests/,
        outside src/: the scan reads src/ only, and the formula check must evaluate floating point.
TWb-D7  (bounded) `generate::day` takes the month (u8) rather than the whole date: the month is all it
        reads. `generate::weather_day` (calendar day + chain → `WeatherDay`) is added in C2 as the pure
        composition C3's `react` calls.
TWb-D8  (bounded) The wet-day frequency π uses 28 days for February (§17.4 does not state it). With 28.25
        February's p_wd would be 176 rather than 178.
TWb-D9  (OUTSIDE §17.10's CHANGE SET; flagged for the operator) Installing a pack named `weather` falsified
        the premise of two existing loader tests, which used `weather` as their example of "a name no
        system of this build provides": worldpack/tests/configuration.rs
        `a_key_that_is_no_system_of_this_build_is_refused_listing_the_systems` (it then failed with
        `ConfigurationOwnerNotEnabled { system: "weather" }`) and tools/cli/tests/configure.rs
        `validate_refuses_each_configuration_mistake_by_name`. Change: the placeholder name only,
        `weather` → `tides`, with a comment saying why. No assertion is weakened and no production code
        is touched. It is one separate commit, so it can be reviewed or dropped on its own. Why it was
        not treated as a stop: none of §17.10's material stops (1)–(7) names the loader's tests, the
        change is forced by the frozen scope item "registered in the installed set", and there is no
        alternative inside the change set (the pack's id `weather` is frozen by SD-TW-b-1 and
        §17.4). Kernel tests that define their own local `weather` stub system compose their own
        worlds, not the installed set, and are unaffected.
TWb-D10 (bounded) Small shape choices inside SD-TW-b-4/-9:
        - `WeatherDay` is itself the `weather-day` event type, rather than being wrapped.
        - `Condition` derives `Ord`, for deterministic sets in tests and clients.
        - `rain_tenth_mm` is decoded through a counting newtype, so a list of the wrong length names its
          key rather than serde's "invalid length".
        - `now` (and `weather-now.hour`) is the hour of *today* the condition holds from. At midnight it
          is folded to 0 when yesterday's condition carries over and no `weather-changed` is stated
          (SD-TW-b-10 (c)).
        - `react(weather-day)` takes the hour from `world.at()` rather than assuming 0, so a genesis that
          is not at a midnight would still be right. At every midnight, and so in every world today,
          it is 0.
TWb-D9  RULING (coordinator, 2026-10-09): accepted, as bounded and necessary.
TWb-D11 (process; reconciliation) Two copies of this execution session ran at once in this worktree after
        two resume messages (the coordinator's error, 2026-10-09). The first copy found unexplained edits
        and stopped under CLAUDE.md §3.1. The coordinator confirmed that both copies had stopped and that
        this session holds the tree alone, and ordered a reconciliation. The twin's uncommitted changes
        were reviewed file by file against §17's C3/C4:
          systems/weather/src/fixture.rs        kept — reads the world's own weather.yaml (one copy)
          systems/weather/src/generate/tests.rs kept — adapted to the block-style table
          systems/weather/src/hours/tests.rs    kept — adapted to `san_diego()`
          systems/weather/src/rules/tests.rs    kept — January's line range computed from the edited
                                                  text, so an inserted key still points inside January
          worlds/market-town/world.yaml         kept — `weather` after `calendar`, `configure` entry,
                                                  comment in calendar's style
          worlds/market-town/README.md          kept, rewrapped to the file's width
          worlds/market-town/configure/weather.yaml  kept — byte-for-byte the table derived in E-TWb-1,
                                                  confirmed by the unit statistics, which equal E-TWb-2's
        None was restored from 4b922da. All of it is in 0728fa9 and was validated after reconciliation
        (weather 27, AC-1 14/14, mutations above).
TWb-F1  MATERIAL STOP (6) — criterion 5 fails as frozen, and its stated premise does not hold. Evidence:
        E-TWb-6. §17.5 C4 (d) compares non-weather facts by payload bytes and says a failure means "a
        controller or rule reads weather records or is perturbed by weather facts". The failure is
        instead the world's shared Process-id counter: the climate Process takes id 2 at genesis, and
        every routine and group activity id after it is one higher. Instants, types, visibility, subjects,
        per-type counts, requests and every person's activity are identical over 300 days.
        This is the Process-id analogue of the event ids that C3 (g) already excludes. No correct
        implementation of SD-TW-b-6/-10 avoids it: the climate is a Process (frozen) and is started at
        genesis (frozen generation order), and the id counter is the kernel's (INV-TW-3).
        Not re-baselined, and the criterion is not edited. Smallest revision proposed for the operator:
        criterion 5 compares non-weather facts with event ids and Process ids excluded, where a Process id
        in a payload is compared through the bijection that maps each id to the id at the same start
        order. Equivalently, the diagnostic's rule: equal everywhere except Process-id leaves shifted by
        the number of climate Processes started before them. The diagnostic test already proves that
        form on the full 300 days, so accepting the revision changes no code. Until it is ruled the PR
        is not marked READY FOR OPERATOR REVIEW.
TWb-F2  (finding) AC-1 check 3 treats order among generic packs as free (M-TWb-A1 stays green), but the
        kernel installs `systems:` in order and refuses a dependency listed later. So a world must list
        `calendar` before `weather`, and Market Town does. No change: AC-1 measures the delta's
        structure, and assembly refuses the wrong order with a clear message.
```

---

# 18. TW-d — San Diego record data, `tools/weather-fetch`, and `source: record`

**`DESIGN FROZEN 2026-10-09 (primary session; rulings in §18.9.1)`**

Design revision: §18 as of PR #107 (planning session, 2026-10-09; same worktree and base as §17).
Approved by: the primary session, 2026-10-09, relayed by the coordinator. QTWd-1 and QTWd-2 are ruled
yes and accepted. QTWd-3 … 8 are ruled as recommended. Implementation base: `origin/main` **after both
TW-b (§17) and IL-b (`mvp0/pr-il-b-interactions`, `data:` attachments, QTW-7) have merged**; until then this
PR does not start. Execution contract: §18.10. Lifecycle: FROZEN.

Frozen means the scope (§18.1), the decisions (§18.3, §18.5), the acceptance and adversarial criteria
(§18.6) and the execution contract (§18.10) are frozen. Progress, evidence, audit findings and bounded
corrections stay writable.

This section is TW-d's single PR design authority and, once frozen, its ledger (§18.11).

## 18.1 Goal, scope, non-goals

**Goal.** Market Town's weather is San Diego's real weather of 2015–2024, replayed day by day and looped.
A player or a world author can switch any world to any station's committed record, or back to rules, by
editing content only. The data is committed, licensed, provenance-recorded and produced by a reproducible
offline tool. Nothing reaches the network at run time.

**In scope.**
- `source: record` in `weather`: the attachment-decoding CSV parser (LF, CRLF and BOM tolerant), gap rules,
  the packed series, the record-date mapping with leap rules, and a second, never-woken Process
  `weather-record` that holds the series.
- The new tool crate `tools/weather-fetch`:
  - a `reshape` mode from NOAA's `.dly` file;
  - `fit` of the rules block;
  - a `fetch` mode behind the cargo feature `fetch`, with `ureq` confined to the tool.
- `worlds/market-town/data/weather/san-diego-usw00023188-2015-2024.csv` and `NOTICE`.
- `.gitattributes` line endings for World Pack data.
- Market Town switches to the record, with the fitted rules for gap filling.
- AC-1 check 3 admits `data/` for allow-listed generic packs (QTWd-1).
- `docs/DECISIONS.md` **DEP-31** and the DEP-8 table row; a root `NOTICE` pointer.

**Non-goals.**
- The GHCNh hourly layer (TW-g).
- ERA5 (QTW-10).
- Any client.
- `mineworld check` (F-TWbd-2).
- Persistence snapshot retention (F-TWbd-1, QTWd-2).
- Changes to IL-b's seam (if it must change, material stop).
- More than 10 record years (QTW-5).

## 18.2 Audit anchors (re-verify at freeze against `main` after IL-b merges)

| File / symbol | Finding | Consequence for TW-d |
| --- | --- | --- |
| IL-b `authoring/src/attachment.rs` (`mvp0/pr-il-b-interactions @ 29d0b49`): `Attachment` (`/`-separated, first component `data`, no `\`, `:`, `.`, `..`, empty or absolute), `path_under(root)` builds the platform path by `join`; `Attached` (bytes by attachment); `ATTACHMENT_MAX_BYTES = 4 MiB` | Paths are platform-neutral by construction, and the loader reads bytes whole. | The pack never sees a path, only bytes, so line endings and a BOM are the pack's to normalize (SD-TW-d-3). The 130 KB file is far under the cap. |
| IL-b `authoring/src/configuration.rs`: `PackConfiguration::attachments(&C) -> Vec<&Attachment>`; `seed(&Seeding, &C, &ConfigurationContext)`; `ConfigurationContext::attached()` | The owner names its attachments and receives their bytes at seed. | `WeatherConfiguration::record.data: Attachment`, and `seed` decodes the bytes into the series it states in `weather-configured`. |
| IL-b SD-IB-5, worldpack `configure.rs` (`read_attachments`; refusals `AttachmentMissing`, `AttachmentOutside`, `AttachmentTooLarge`); drift: `check_configuration` re-reads attachments and compares the seeded facts (E-IB-3: `a_changed_attachment_is_drift_and_an_unchanged_one_is_not`) | A changed file is drift with no new mechanism. | Criterion 3 runs through the real drift check. Because decoding normalizes line endings, a CRLF checkout of an LF file is **not** drift (SD-TW-d-3), which is tested. |
| IL-b test pack `test-table` (E-IB-7: "rows decoded from the attachment (CRLF-tolerant)"); D-IB-9: "S19 TW-d's pack takes a plain configuration" | A plain configuration with an attachment key is the supported shape. | `weather` keeps a plain `PackConfiguration` (not `interactions!()`). |
| §17 SD-TW-b-4 `weather-configured { seed, rules, record: Option<RecordRef> }` | Shaped for this PR. | `RecordRef` becomes `RecordSeries { station, first_year, years, days: packed }`, with no event schema bump. |
| `persistence/src/world.rs` snapshot cadence (F-TWbd-1) | Process state is multiplied by the number of snapshots. | The series is packed, and it lives in its own never-rewritten Process so the frequently folded climate state stays ≤ 8 KB (SD-TW-d-5). |
| `tests/acceptance/tests/ac1_composability.rs` l. 1234–1280 `world_delta_failures` | Top-level entries present in one pack only must be `items`, `organizations` or `configure` (Market Town only). **`data/` in Market Town fails check 3** ("data: present in one pack only"). | TW-d needs QTWd-1's ruling: admit `data/` in Market Town when every file in it is an attachment named by an allow-listed generic pack's `configure/` file or that attachment's `NOTICE`. TWa-R1's wording ("generic packs on an explicit allow-list … enabled through configuration") arguably covers it, but it changes a frozen acceptance measure, so it is asked. |
| `Cargo.toml` `[workspace] members` lists `tools/cli` explicitly | A new tool crate is a members edit. | `"tools/weather-fetch"` is added to the root manifest. |
| `.gitattributes` | Only `*.bin` and `*.glb` binary; no `eol` rule. | `worlds/*/data/**/*.csv text eol=lf` is added, so every platform checks out the same bytes and the NOTICE's byte count holds everywhere (SD-TW-d-12). The parser stays CRLF-tolerant for user-authored files (SD-TW-d-3). |
| `NOTICE` (root) | Lists where bundled assets' records live. | One entry is added: `worlds/market-town/data/weather/NOTICE`, NOAA GHCN-Daily, CC0 / US public domain. |
| `docs/DECISIONS.md` DEP-8 table (l. ~303) | Source, licence and use rows. | One row (§15.1 text). |
| No HTTP client in the workspace (`Cargo.lock`: no `ureq`, `reqwest`); agents may not run `curl` (execution contracts) | The data must be fetched by some program. | The tool's `fetch` mode is how the implementing session obtains the file (QTWd-4). The feature is off by default. |

**Evidence read for the data plan** (WebFetch, 2026-10-09; never at run time):
- **E-TWd-pre-1.** <https://www.ncei.noaa.gov/pub/data/ghcn/daily/readme.txt>: PRCP is tenths of mm,
  TMAX and TMIN tenths of °C, AWND tenths of m/s, and missing is −9999. The WT codes are 01 fog (may
  include heavy fog), 02 heavy fog, 03 thunder, 08 smoke or haze, 13 mist, 14 drizzle, 16 rain, and 21
  ground fog. A Q-flag blank means "did not fail any quality assurance check".
- **E-TWd-pre-2.** <https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/USW00023188.dly> is plain fixed-width
  text of about 4.3 M characters, beginning `USW00023188193907TMAX …`. A read of the excerpt covering
  2016-03 to 2017-12 found:
  - present: TMAX, TMIN, PRCP, AWND, WDF2, WSF2, WT01, WT02, WT03 and WT08, with source flag `W`;
  - absent in that excerpt: WT13, WT14, WT16, WT21, ACSH, ACMH, PSUN and TSUN.

  Population over 2015–2024 as a whole is **unverified** and is the tool's report (C4).
- **E-TWd-pre-3.** <https://www.ncei.noaa.gov/pub/data/ghcn/daily/readme-by_station.txt>: the by-station
  CSV is a long format (`ID, YYYYMMDD, ELEMENT, VALUE, M, Q, S, OBS-TIME`) and is served gzipped. It was
  not chosen (QTWd-3).

## 18.3 Design decisions (SD-TW-d-n)

| Id | Decision |
| --- | --- |
| SD-TW-d-1 | **The committed CSV format** (the pack's input, owned by `weather`, documented in its README). UTF-8. The header is exactly `date,tmax_dc,tmin_dc,prcp_tenth_mm,awnd_dms,wdf2_deg,fog,thunder`. One row per day, `date` is `YYYY-MM-DD`, ascending, no duplicates, and covering whole calendar years from Jan 1 to Dec 31. The integer cells may be empty (missing). `fog` and `thunder` are `0` or `1`. Columns change from §6.4: drizzle and rain flags are dropped (WT14 and WT16 are unpopulated per E-TWd-pre-2, and precipitation comes from PRCP), and `wdf2_deg` is added (populated, it gives `wind_from_deg`). `fog = WT01 ∨ WT02 ∨ WT21`; `thunder = WT03`. The `sky_am`, `sky_pm` and `fog_hours` columns are TW-g's, additive. |
| SD-TW-d-2 | **Missing values.** In the tool, a `.dly` value of −9999, or one with a non-blank Q-flag, is written as an empty cell. A WT element absent on a day is 0 (WT elements record presence only). In the pack, a day is *missing* when TMAX, TMIN or PRCP is empty. A run of ≤ 3 missing days copies the previous complete day's values (origin `Filled`). A longer run takes rule days (origin `Rule`) when `fill: rules`, or is refused at seed when `fill: none`, naming the attachment and the first and last missing dates. An empty AWND or WDF2 alone takes the month's `wind_dms` or `wind_from_deg` from the rules and does not make the day missing. A leading missing run (no previous day) is treated as a long gap. |
| SD-TW-d-3 | **Cross-platform decoding.** The parser accepts and strips one leading UTF-8 BOM. It splits lines on `\n` and strips one trailing `\r` from each, so LF and CRLF files decode identically. A final newline is optional, and an empty final line is ignored. Any other `\r`, a tab, a quoted cell or a non-ASCII byte outside the BOM is refused with the 1-based line number. The parser never sees a path (IL-b gives it bytes). A test decodes one fixture in LF, CRLF, CRLF+BOM and no-final-newline forms and asserts byte-identical `weather-configured` payloads. |
| SD-TW-d-4 | **Record-date mapping** (§6.3, made exact for a `first_year` that is not the file's first year). `record_year = file_first + ((first_year − file_first) + (world_year − epoch_year)).rem_euclid(N)`. Here `file_first` is the file's first year, `N` the number of whole years in the file, and `epoch_year` the year of the first `day-began` the pack sees, stored in the climate state. With `first_year == file_first` this is §6.3's formula. Month and day are kept. A world Feb 29 on a non-leap record year uses that year's Feb 28. A record Feb 29 is used only by a world Feb 29. `first_year` must lie within the file's years, or the configuration is refused at seed. |
| SD-TW-d-5 | **Where the series lives.** `weather-configured.record = RecordSeries { station: String (≤ 32 ASCII), first_year: i32, years: u16, days: PackedDays }`. `PackedDays` is a fixed 9-byte record per day: `tmax_dc i16, tmin_dc i16, prcp u16, awnd u8, wdf2/2 u8, flags u8` (fog, thunder, origin 2 bits, and wind-missing). It is serialized as one lowercase hex string, so 3 653 × 18 characters is about 66 KB in JSON. Hex needs no new dependency. Base64 would save about a third and is rejected only to avoid a dependency or 30 lines of codec; this is recorded. `react(weather-configured)` starts a second Process `weather-record` (open-ended, never woken, never rewritten) holding `{ series }`. The climate Process stays ≤ 8 KB (SD-TW-b-6). `react(day-began)` reads the record Process once per day (read-only) to take that day's 9 bytes. Each snapshot therefore carries about 66 KB more, which is about +13 % on today's about 0.5 MB market-town snapshot (F-TWbd-1, QTWd-2). Alternatives: (a) the series in the climate state, rejected because it is re-encoded at every fold, several times a day; (b) re-reading the CSV at run time, rejected as §6.5 (c); (c) a kernel "static world data" store, rejected by INV-TW-3. |
| SD-TW-d-6 | **Record days in the generator.** A record or filled day sets `summary` from the record and `chain.wet = prcp ≥ 3`. It resets the anomalies to `tmax − month.tmax_dc` and `tmin − month.tmin_dc`, so a following rule-filled day continues the same climate. The hours are derived by §17's `hours()`, unchanged. `origin` is `Record { date }`, `Filled { date }` or `Rule`. |
| SD-TW-d-7 | **Configuration** (TW-d's schema; `deny_unknown_fields`). `source: rules \| record`; `seed`; `rules` (always required, used for fill and for wind gaps); `record:` required iff `source: record`, holding `{ station: <GHCN id, provenance only>, data: <Attachment>, first_year: <i32> }`; `fill: rules \| none` (default `rules`). `attachments()` returns `record.data` when present. |
| SD-TW-d-8 | **`tools/weather-fetch`** (crate `mineworld-weather-fetch`, binary `weather-fetch`, a workspace member). Its dependencies are `clap` (workspace), `mineworld-weather` (one CSV and rules authority: the tool writes what the pack decodes, and `fit` and the generator are the pack's types) and `serde-saphyr` (to read `--base`). It also has an optional `ureq` behind feature `fetch`, off by default, with the minimal TLS feature set; the version is pinned exactly at C3 and recorded in DEP-31. Its modes are listed in the three rows below. |
| SD-TW-d-8a | `fetch --station USW00023188 --out FILE`, available only with `--features fetch`, downloads `https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/<station>.dly` and writes it verbatim. It records the URL, the UTC retrieval date and the byte count to stdout for the ledger (SD-TW-d-12: no hash crate). |
| SD-TW-d-8b | `reshape --input FILE.dly --station ID --from YYYY --to YYYY --out-dir DIR` parses the `.dly` file by the readme's fixed columns, writes `<name>.csv` (LF, SD-TW-d-1) and `NOTICE` (§6.4's text plus the provenance fields of SD-TW-d-12), and prints a report: rows; missing TMAX/TMIN/PRCP by year; gap runs with their lengths; per-WT population by year; Q-flagged values dropped. |
| SD-TW-d-8c | `fit --input FILE.csv --base configure/weather.yaml --out FILE.yaml` estimates per month, from complete record days: `p_wet_after_dry` and `p_wet_after_wet` from counted transitions (wet = PRCP ≥ 3, NOAA's 0.01 in rounded up to the record's 0.1 mm); `rain_tenth_mm` as the 10th/30th/50th/70th/90th percentiles of wet-day amounts (nearest rank); `tmax_dc` and `tmin_dc` as means; `t_noise_dc` as the integer square root of the anomaly variance × 3 / 2 (because uniform noise on ±n has variance n²/3), clamped to bounds; `t_ar_permille` from the lag-1 autocovariance ratio; `wet_tmax_shift_dc` as the wet minus all-day TMAX mean; `fog_permille` and `thunder_permille` as frequencies; `wind_dms` as the mean AWND; `wind_from_deg` as the modal WDF2 in 10° bins. `overcast_morning_permille` cannot be estimated from GHCN-Daily and is copied from `--base`, with a comment saying so. It writes the full `configure/weather.yaml` text with the fitted `rules:` block and a header naming the command. All of its arithmetic is integer (`i64`/`i128`), so the fit is byte-reproducible. |
| SD-TW-d-9 | **Determinism of the tool.** Same input bytes and same arguments give byte-identical outputs: no timestamps in the CSV, the NOTICE's retrieval date passed as `--retrieved YYYY-MM-DD` rather than read from the clock, `BTreeMap` iteration only, and LF only. |
| SD-TW-d-10 | **Market Town** switches to `source: record`, `first_year: 2015`, `fill: rules`, with the `rules:` block replaced by `fit`'s output on the committed CSV and the `--base` being TW-b's file. A test (C5) re-runs `fit` in-process on the committed CSV and asserts the committed `configure/weather.yaml` is byte-equal to its output, so the data and its fitted rules cannot drift apart. |
| SD-TW-d-11 | **DEP-31** lands in C1, as §15.1 text, amended. It covers the data source (GHCN-Daily, the `.dly` input), GHCNh for TW-g, and the rejections of Meteostat, Open-Meteo (free API non-commercial), ERA5 (fallback only) and LARS-WG. It records that `ureq` is confined to `tools/weather-fetch` behind an off-by-default feature (QTW-14) and that the CSV is our own format, reshaped and labelled "modified". It adds the DEP-8 row. |

## 18.4 Commit plan

### C1 — `docs: DEP-31 weather data; the record format; weather-fetch spec`

1. **Goal.** The data, licence and dependency decisions, and the CSV format, are written before code.
2. **Scope.**
   - `docs/DECISIONS.md`: DEP-31 plus the DEP-8 row, and, if QTWd-1 is ruled, an ARC-35 note dated with
     the ruling.
   - `systems/weather/README.md`: the record format section.
   - New `tools/weather-fetch/{README.md,Cargo.toml,src/main.rs}`, as a doc-only skeleton.
   - The root `Cargo.toml` members list.
   - The root `NOTICE` entry.
3. **Implementation.**
   - [ ] DEP-31.
   - [ ] The DEP-8 row.
   - [ ] The ARC-35 note (only with the ruling).
   - [ ] The format section.
   - [ ] The tool skeleton and the members entry.
   - [ ] The NOTICE pointer.
4. **Validation.**
   - [ ] Both doc checks pass.
   - [ ] `cargo check -p mineworld-weather-fetch` is clean.
5. **Acceptance.** As for TW-b C1.
6. **Failure cases.** QTWd-1 unruled: C1 lands without the ARC-35 note, and C5 is parked (TW-a's C4
   precedent).
7. **Review.**
   - [ ] DEP-31 does not restate ARC-68.
   - [ ] The licence wording matches §3.2 and E-TWd-pre-1.
   - [ ] "Modified data so labelled" is present.
8. **Boundary.** Docs and skeletons only.

### C2 — `weather: record source — decoding, gaps, the record process, the date mapping`

1. **Goal.** The pack replays a committed record exactly, on every platform.
2. **Scope.** `weather/src/{record.rs (new: CSV decode, PackedDays, gaps), configuration.rs, event.rs
   (RecordSeries), process.rs (RecordProcess), system.rs (attachments, seed, react)}`; tests
   `tests/record.rs` with a fixture World Pack under `tests/fixtures/record-town/` holding a 2-year
   hand-written CSV (2015–2016, including a 2-day gap, a 5-day gap, Feb 29 2016 and a Q-dropped empty
   cell).
3. **Implementation.**
   - [ ] The decoder (SD-TW-d-1, -2, -3).
   - [ ] Packing (SD-TW-d-5).
   - [ ] `source: record` and `attachments()` (SD-TW-d-7).
   - [ ] `seed` decodes the bytes and refuses with `Rejection::System { code: "weather-record-invalid",
     detail: "<attachment>: line N: …" }`.
   - [ ] The record Process.
   - [ ] The mapping (SD-TW-d-4).
   - [ ] Record and filled days in the generator (SD-TW-d-6).
4. **Validation.**
   - [ ] (a) **LF, CRLF, BOM:** four encodings give byte-identical `weather-configured`.
   - [ ] (b) Refusals, each with its line: bad header, unsorted date, duplicate date, a partial year, a
     non-integer cell, a tab, a quoted cell, `first_year` outside the file, `fill: none` with a 5-day gap
     (naming both dates).
   - [ ] (c) **Criterion 2 (leap)**, with epoch 2026-10-08 and the fixture file 2015–2016 (`file_first`
     2015, N = 2), asserted on `weather-day.origin` dates. The expected values follow from SD-TW-d-4:
     - `first_year: 2015`:
       - world 2027-02-28 → record 2016-02-28. Record 2016-02-29 is skipped, because world 2027 has no
         Feb 29.
       - world 2027-03-01 → record 2016-03-01.
       - world 2028-02-29 → record 2015-02-28 (2015 is not a leap year).
       - world 2028-03-01 → record 2015-03-01.
     - `first_year: 2016`:
       - world 2026-10-08 → record 2016-10-08.
       - world 2028-02-29 → record 2016-02-29 (leap to leap).
   - [ ] (d) Gaps: the 2-day gap is `Filled` with the previous day's values, and the 5-day gap is 5 `Rule`
     days whose chain continues from the last record day.
   - [ ] (e) **Criterion 3 (drift):** through `WorldPack::read` and the real drift check, editing one CSV
     value after assembly reports `weather` changed, and converting the file to CRLF reports no drift.
   - [ ] (f) Climate state ≤ 8 KB (TW-b's pin still holds). The record Process holds the series, and its
     encoded size is recorded.
   - [ ] (g) A resume mid-year equals uninterrupted.
5. **Acceptance.** Every assertion above is on observed facts. (c)'s expected dates are written in the test
   before it is run.
6. **Failure cases.** As in (b). An attachment over 4 MiB is IL-b's refusal, which is not re-tested here.
7. **Commands.** `cargo test -p mineworld-weather`; clippy.
8. **Review.**
   - [ ] The fold equals the state.
   - [ ] The record Process is never rewritten (grep for `set_process_state::<RecordProcess>` gives none).
   - [ ] No path handling in the pack.
   - [ ] Leap logic is written once.
   - [ ] Integer overflow is impossible in packing.
9. **Boundary.** The pack only. No world, no tool.

### C3 — `weather-fetch: reshape, fit, and an optional fetch`

1. **Goal.** The data and its fitted rules are reproducible by a tool anyone can run offline.
2. **Scope.** `tools/weather-fetch/src/{main.rs, dly.rs (fixed-width parser), reshape.rs, notice.rs,
   fit.rs, fetch.rs (#[cfg(feature = "fetch")])}`, the manifest with `ureq` optional, and tests with a
   hand-made `.dly` fixture (three months, with −9999 values, a Q-flag, WT01 and WT03 lines, and a
   month with no WT lines).
3. **Implementation.**
   - [ ] The modes (SD-TW-d-8, SD-TW-d-8a to -8c).
   - [ ] Determinism (SD-TW-d-9).
   - [ ] The report.
4. **Validation.**
   - [ ] (a) **Criterion 1:** `reshape` run twice on the fixture gives byte-identical CSV and NOTICE,
     and the CSV decodes with the pack's decoder (round trip).
   - [ ] (b) Q-flagged values become empty cells. −9999 becomes empty. Missing WT lines become 0.
   - [ ] (c) `fit` on a synthetic CSV generated by the pack's own generator from a known table (100 years)
     recovers `p_wd` and `p_ww` within ±20‰ and the means within ±3 dC. This is the fit's own oracle.
   - [ ] (d) **Criterion 4 (INV-TW-7):** `cargo tree -e normal -i ureq --workspace --all-features` lists
     only `mineworld-weather-fetch` as the dependent. `cargo tree -p mineworld-cli -i ureq` and
     `cargo tree -p mineworld-server -i ureq` (default and all features) report no match. The commands
     and output are recorded verbatim.
   - [ ] (e) `cargo build -p mineworld-weather-fetch` (no features) does not compile `ureq` (from
     `cargo tree -p mineworld-weather-fetch`).
5. **Acceptance.** (a) is byte-equal; (c) is within bounds; (d) lists exactly one dependent.
6. **Failure cases.**
   - Malformed `.dly` lines are refused with a line number.
   - A station id mismatch between `--station` and the file is refused.
   - `--from` > `--to` is refused.
   - Years outside the file are refused.
   - `fetch` without network gives a clear error, and the tool never retries silently.
7. **Commands.** `cargo test -p mineworld-weather-fetch`; `cargo clippy -p mineworld-weather-fetch
   --all-targets --all-features -- -D warnings`; the `cargo tree` lines.
8. **Review.**
   - [ ] The tool owns no CSV format of its own (it calls the pack's writer and decoder types).
   - [ ] No float in `fit` (scan as in TW-b criterion 6).
   - [ ] The NOTICE text matches §6.4 and DEP-31.
   - [ ] `ureq`'s TLS backend needs no build tool beyond the CI image's (material stop (2) otherwise).
9. **Boundary.** The tool only. No data is committed yet.

### C4 — `data: San Diego, GHCN-Daily USW00023188, 2015–2024`

1. **Goal.** The real record is committed with provenance.
2. **Scope.** `worlds/market-town/data/weather/san-diego-usw00023188-2015-2024.csv` and `NOTICE`;
   `.gitattributes` (`worlds/*/data/**/*.csv text eol=lf`).
3. **Implementation.**
   - [ ] `cargo run -p mineworld-weather-fetch --features fetch -- fetch --station USW00023188 --out
     target/tw-d/USW00023188.dly` (once; the URL, date and byte count are recorded).
   - [ ] `reshape … --from 2015 --to 2024 --retrieved <date> --out-dir worlds/market-town/data/weather`.
   - [ ] The report goes into evidence.
4. **Validation.**
   - [ ] (a) 3 653 data rows (2015–2024, two leap years).
   - [ ] (b) File size recorded, expected about 110–130 KB, must be < 1 MiB.
   - [ ] (c) The report: missing-day runs; WT01, WT03 population per year. If WT01 is unpopulated in any
     year, that year's fog comes from no record day, and this is recorded as R-TW-5 occurring.
   - [ ] (d) Re-running `reshape` on the same `.dly` gives a byte-identical CSV and NOTICE (criterion 1
     on real data).
   - [ ] (e) The NOTICE's recorded byte and line counts (SD-TW-d-12) equal the committed CSV's.
5. **Acceptance.** (a), (b), (d) and (e) as stated, and (c) recorded.
6. **Failure cases.**
   - NOAA unreachable: retry once later. If it is still unreachable, the operator downloads the file by
     hand and the tool's `reshape` runs on it (provenance records who downloaded it).
   - A gap > 3 days in 2015–2024 is fine with `fill: rules` and is reported.
7. **Commands.** The tool lines above.
8. **Review.**
   - [ ] The NOTICE carries the attribution, both Menne et al. citations, "modified … not endorsed by
     NOAA", the URL and the retrieval date.
   - [ ] The licence per E-TWd-pre-1 and §3.2.
9. **Boundary.** Data and `.gitattributes` only.

### C5 — `worlds: market-town's weather is San Diego's record`

1. **Goal.** The default town replays the record, and AC-1 stays exact.
2. **Scope.**
   - `worlds/market-town/configure/weather.yaml` (`fit`'s output).
   - `tests/acceptance/tests/ac1_composability.rs`: QTWd-1's `data/` admission and its unit test.
   - `systems/weather/tests/market_town.rs` (or under the CLI tests, by the existing pattern): the
     fit-equality test (SD-TW-d-10).
3. **Implementation.**
   - [ ] Run `fit` and commit its output.
   - [ ] The AC-1 change (only after the QTWd-1 ruling; otherwise park the commit as TW-a's C4 did).
   - [ ] The fit-equality test.
4. **Validation.**
   - [ ] (a) **CP-TW-d** (§18.6).
   - [ ] (b) **Criterion 5:** over 100 world years with the fitted rules (`source: rules`, pure function),
     each month's wet-day frequency is within ±3 points of the record's 2015–2024 frequency for that
     month, and the pooled mean wet-spell length is within ±15 % of the record's. M-TWd-5 must turn
     both red.
   - [ ] (c) The fit-equality test is green. Mutation **M-TWd-10**: edit one fitted number, and the test
     must fail.
   - [ ] (d) INV-TW-1: the four other digests equal `main`'s. The new market-town baseline is recorded.
   - [ ] (e) Criterion 5 of TW-b (weather moves nobody) re-checked on the record world.
   - [ ] (f) AC-1 14/14 plus the new `data/` test. Mutations: **M-TWd-A1**, a stray
     `worlds/market-town/data/other.txt` that is not an attachment or a NOTICE, must fail naming it.
     **M-TWd-A2**, a `data/` directory in Social Café, must fail.
5. **Acceptance.** As written. (b)'s tolerance and sample are fixed now.
6. **Failure cases.**
   - (b) fails because the record's wet frequency is non-stationary within a month (a known WGEN
     weakness). Report it with the per-month numbers. It is not material unless the gap is more than
     5 points.
   - A digest moves: material stop (4).
7. **Commands.** As TW-b C4, with town runs within budget.
8. **Review.**
   - [ ] The AC-1 change admits nothing but attachments and their NOTICE.
   - [ ] The fitted file's header names the command.
   - [ ] The rules still bound-check.
9. **Boundary.** Content, one acceptance guard, one test.

### C6 — `docs(plan): TW-d evidence`

- [ ] Ledger, handoff (`handoff-tw-d.md`), full gate (as TW-b C5), CI on the exact head; then READY FOR
  OPERATOR REVIEW.

## 18.5 Provenance without a hash dependency (SD-TW-d-12)

`Cargo.lock` at `f80bbb7` contains no SHA-256 implementation (it has `sha1` and `digest` through the
WebSocket stack, and no `sha2`; audited 2026-10-09). The tool therefore adds no hash crate. The NOTICE
records the following, and the `fetch` mode prints the same counts for the `.dly` file:
- the source URL;
- the retrieval date (`--retrieved`);
- the exact command line;
- the tool's version;
- the `.dly` input's byte count;
- the CSV's byte count and line count.

The provenance claim is that the data came from NOAA's published file through this tool. Reproducibility is
owned by the reshape determinism test (criterion 1) and by `eol=lf` (QTWd-6), not by a digest in a text
file. Adding a hash crate for this alone would be material stop (2). §6.4's "the fetch tool's version and
command" is kept, and no digest is recorded (QTWd-7).

## 18.6 Acceptance and adversarial criteria (fixed before measuring)

**CP-TW-d**, revising §11.2 only in precision:

`mineworld run worlds/market-town --headless --seed 1 --days 365 --save D`, then:
1. **Seasonality.** Wet days with world dates in Dec–Feb exceed wet days in Jun–Aug.
2. **Fog.** At least one `weather-day` with `fog` in May–July, **if** the tool's report shows WT01/WT02
   populated in record year 2016. World 2027 maps to record 2016. If WT01/WT02 are unpopulated, the claim is
   recorded N/A with the report's evidence.
3. **Exact replay.** World 2026-10-08's `weather-day` has `origin = Record { 2015-10-08 }` and its summary
   TMAX, TMIN and PRCP equal that CSV row's cells exactly. The test reads the row from the committed file.
   World 2027-10-08 likewise equals record 2016-10-08. World 2027-10-08 is day 365, and its `day-began`
   is at instant 365 × 86 400, the run's last instant, which `run` includes (TWa-D5).
4. **Restart.** D resumed from day 200 to day 365 equals the uninterrupted run.

**Adversarial criteria:**

| # | Criterion | Owner | Mutation |
| --- | --- | --- | --- |
| 1 | The tool is byte-reproducible on the same input | C3 (a), C4 (d) | **M-TWd-1**: the NOTICE gains the wall-clock time of the run (seconds). (a) must fail |
| 2 | The leap mapping, exactly as SD-TW-d-4 | C2 (c) | **M-TWd-2**: map a world Feb 29 to the record's Mar 1. (c) must fail |
| 3 | A changed CSV value is drift; a CRLF conversion is not | C2 (e) | the edit itself is the mutation; **M-TWd-3b**: stop stripping `\r`, and the CRLF case must report drift |
| 4 | INV-TW-7: only `mineworld-weather-fetch` depends on an HTTP client | C3 (d) | **M-TWd-4**: add `mineworld-weather-fetch` as a dev-dependency of `mineworld-cli` with `fetch` enabled. The tree check must name `mineworld-cli` |
| 5 | Fitted rules reproduce, over 100 years, each month's record wet-day frequency within ±3 points, and the record's pooled mean wet-spell length (all months, 2015–2024) within ±15 % | C5 (b) | **M-TWd-5**: the fit writes `p_wet_after_wet := p_wet_after_dry`. The stationary frequency falls from `p_wd / (1000 − p_ww + p_wd)` to `p_wd`, and the spell length falls to about 1 day. Both halves must fail |
| 6 (new) | Cross-platform decode: LF, CRLF and BOM give identical facts | C2 (a) | M-TWd-3b |
| 7 (new) | The committed rules equal `fit` of the committed CSV | C5 (c) | M-TWd-10 |

## 18.7 Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-TWd-1 | IL-b's final API differs from `29d0b49` (`attachments`, the context, `Attached`). | Re-audit at freeze. An API change TW-d needs is material stop (3). |
| R-TWd-2 | AC-1 check 3 refuses `data/` (§18.2). | QTWd-1. Until it is ruled, C5 is parked, with the TW-a C4 precedent. |
| R-TWd-3 | Snapshot growth: about +66 KB per snapshot (F-TWbd-1). | Packed hex. Measured in C2 (f) and C5. QTWd-2 asks whether to accept it. The structural fix (retention) is the persistence lane's. |
| R-TWd-4 | `ureq`'s TLS backend needs native tooling (`aws-lc-rs` needs cmake and nasm on Windows; `ring` needs a C compiler and clang on aarch64-windows). | The `fetch` feature is off by default, so default builds and every runtime crate never compile it. `--all-features` clippy on Linux CI compiles it. A backend needing more than the CI image's C toolchain is material stop (2). The fallback is `--input` only (QTW-14's alternative). |
| R-TWd-5 | WT flags are sparsely populated (R-TW-5). | The tool reports it. Fog then comes from rule days only on filled days, and CP-TW-d 2 is N/A with evidence. TW-g adds GHCNh. |
| R-TWd-6 | NOAA moves the `.dly` path (R-TW-10). | The data is committed. The tool takes `--input` too, and the URL is one constant recorded in DEP-31. |
| R-TWd-7 | Platforms: there is no Windows CI, and a Windows checkout with `core.autocrlf=true` converts text files. | `eol=lf` in `.gitattributes`, plus a CRLF-tolerant parser, plus test (a). Windows is argued, not run (F-IB-16 records the Unix-only test files that block a Windows CI today). |
| R-TWd-8 | The fitted WGEN-lite misses San Diego's spell structure or seasonality within a month. | Criteria 5 and its spell pairing. The record is the default and the rules only fill gaps, so the default world's realism comes from the record. |

## 18.8 Findings recorded at planning

```text
F-TWd-1  E-TWd-pre-2 (excerpt 2016-03 … 2017-12): WT14 (drizzle) and WT16 (rain) absent, WT01/02/03/08
         present, WDF2 present. §6.4's CSV columns are corrected accordingly (SD-TW-d-1).
F-TWd-2  The GHCN-Daily `.dly` file is uncompressed fixed-width text (about 4.3 MB for the whole record);
         the by-station CSV is gzipped (E-TWd-pre-3). Choosing `.dly` avoids a gzip dependency (QTWd-3).
F-TWd-3  AC-1 check 3 refuses a `data/` directory in Market Town (§18.2).
F-TWd-4  The execution contracts forbid `curl`, and the workspace has no HTTP client: without the tool's
         `fetch` mode the implementing session cannot obtain the data itself (QTWd-4).
```

## 18.9 Questions (QTWd-n). **[OPERATOR]** marks operator-material ones.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QTWd-1 [OPERATOR]** | AC-1 check 3 (ARC-35): admit a `data/` directory in Market Town when every file in it is an attachment named by an allow-listed generic pack's `configure/` file, or that attachment's sibling `NOTICE`? TWa-R1 admitted "generic packs … enabled through configuration". Attachments are configuration (ARC-61 note), but the ruling did not name `data/`, so this is asked rather than assumed. | **Yes**, as an ARC-35 note dated with the ruling. The market delta stays exactly the six packs and their content, and two mutations (M-TWd-A1, -A2) guard the admission. |
| **QTWd-2 [OPERATOR]** | Accept that the record series adds about 66 KB to every world snapshot (+~13 % on Market Town's measured ~0.5 MB snapshots; F-TWbd-1), with snapshot retention routed to the persistence lane as a separate item? The alternative inside S19 is fewer record years, which reverses QTW-5. | **Accept**, and open a persistence item: a 30-day Market Town save is already 266 MB on `main` (measured), so retention is needed with or without weather. |
| QTWd-3 | Tool input: NOAA's `.dly` (plain fixed-width, the format readme.txt §III documents) rather than the gzipped by-station CSV? | **`.dly`.** No gzip dependency, and it is the canonical documented format. |
| QTWd-4 | Keep QTW-14's `fetch` mode (`ureq` in the tool only), behind an **off-by-default** cargo feature? | **Yes.** It is the only way an agent session can fetch the data under the no-`curl` rule, and the feature keeps it out of every default build. |
| QTWd-5 | CSV columns as SD-TW-d-1 (drop drizzle and rain flags, add `wdf2_deg`), revising §6.4. | **Yes**, per F-TWd-1. |
| QTWd-6 | `.gitattributes`: `worlds/*/data/**/*.csv text eol=lf`. | **Yes.** The same bytes on every platform. The parser still tolerates CRLF for user files. |
| QTWd-7 | NOTICE hash: no new dependency for a SHA-256 (§18.5). | **Yes.** Reproducibility is owned by criterion 1 and `eol=lf`. |
| QTWd-8 | The series encoding: hex (no dependency, about 66 KB) rather than base64 (about 44 KB, needs a dependency or a hand-written codec)? | **Hex** now. Revisit with QTWd-2's persistence item. |

### 18.9.1 Rulings, 2026-10-09 (primary session; binding)

| Id | Ruling |
| --- | --- |
| QTWd-1 | **Yes.** This applies the operator's AC-1 ruling of 2026-10-09 (TWa-R1: generic packs on the allow-list are admitted after the six market packs). Admitting their named `data/` attachments and the NOTICEs beside them is the same allowance. TW-d records it as an **ARC-35 note** in C1 and implements it in C5, with mutations M-TWd-A1 (a stray file under `data/` must fail, named) and M-TWd-A2 (a `data/` in Social Café must fail). |
| QTWd-2 | **Accepted.** Weather's per-snapshot addition (about 66 KB packed series, SD-TW-d-5) is kept as designed. Snapshot retention goes to the persistence lane as **F-SAVE-1** (saves growing about 2.5 GB per 300 days); the primary session opens its design. |
| QTWd-3 … QTWd-8 | Accepted as recommended. |

These rulings settle every "after QTWd-1" and "if QTWd-1 is unruled" clause in §18.4 and §18.10. C1 lands the ARC-35 note, and C5 is not parked.

## 18.10 Execution contract (frozen 2026-10-09)

```text
PROJECT / PR            S19 TW-d — San Diego record data, tools/weather-fetch, and source: record
PRIMARY DESIGN DOC      .structured-coding/plans/mvp0/step-19-time-weather.md §18 (live ledger §18.11)
RELATED / BINDING DOCS  as §17.10, plus §17 (TW-b, merged), step-18-interaction-list.md §12 (IL-b: SD-IB-5,
                        QIB-6, D-IB-9), docs/DECISIONS.md DEP-8, DEP-31 (this PR), ARC-55; docs/REUSE_POLICY.md
IMPLEMENTATION BASE     origin/main at freeze, after BOTH TW-b and IL-b have merged; re-audit §18.2 there
WORKTREE                /Users/yuema137/mineworld-worktrees/impl-tw-d — held by one session only
BRANCH                  mvp0/pr-tw-d-data, from the base above
APPROVED SCOPE          §18.1. Change set: systems/weather/** ; tools/weather-fetch/** (new); Cargo.toml
                        (members); Cargo.lock; .gitattributes (one line); NOTICE (one entry);
                        worlds/market-town/{configure/weather.yaml,data/weather/**,README.md};
                        tests/acceptance/tests/ac1_composability.rs (data/ admission, after QTWd-1);
                        docs/DECISIONS.md (DEP-31, DEP-8 row, ARC-35 note after QTWd-1); this section;
                        handoff-tw-d.md
FROZEN INVARIANTS       INV-TW-1, -2, -3, -5, -7, -10; SD-TW-d-1 … 11; §18.6 criteria and mutations
SEQUENCE                C1 → C2 → C3 → C4 → C5 → C6 (§18.4); QTWd-1 is ruled (§18.9.1): C1 carries the
                        ARC-35 note and C5 is not parked
ALLOWED COMMANDS        as §17.10, plus: the weather-fetch binary under target/ (including `fetch`, at most
                        3 network fetches of the one NOAA URL); WebFetch for NOAA metadata only
NOT ALLOWED             as §17.10
AUTHORITY               commit and push to the branch; open the PR; READY FOR OPERATOR REVIEW; never merge
BUDGET                  ≤ 8 town runs (300–366 days); ≤ 3 NOAA fetches; full workspace test ≤ 2 runs
MATERIAL STOPS          (1)–(4) as §17.10 with "any new external dependency" read as "any beyond `ureq`
                        (and its tree) in the tool"; (5) a TLS backend needing tools beyond the CI image's
                        C toolchain; (6) the data's licence or terms differ from E-TWd-pre-1 / §3.2 when
                        re-read; (7) a §18.6 criterion fails after a correct implementation (except the
                        bounded case of C5 (b) ≤ 5 points); (8) AC-1 needs more than QTWd-1's ruling
GATE                    as §17.10
HANDOFF                 .structured-coding/plans/mvp0/handoff-tw-d.md
```

## 18.11 Ledger

Execution session: worktree `/Users/yuema137/mineworld-worktrees/impl-tw-d`, branch `mvp0/pr-tw-d-data`, base
`origin/main @ 2c6d34c` (TW-b #113 = 712bb51 and IL-b #102 merged; also ED #101 and the S10 P5 plan #111).
Handoff: [`handoff-tw-d.md`](handoff-tw-d.md).

**Start-of-session audit (2026-10-09).** §18.2's anchors re-verified on 2c6d34c:
- `authoring/src/attachment.rs`: `Attachment` (`/`-separated, first component `data`, refusals as listed),
  `path_under`, `Attached::get`, `ATTACHMENT_MAX_BYTES = 4 MiB`. `authoring/src/configuration.rs`:
  `PackConfiguration::attachments(&C) -> Vec<&Attachment>`, `seed(&Seeding, &C, &ConfigurationContext)`,
  `ConfigurationContext::attached()`. `worldpack/src/configure.rs` `read_attachments` (l. 193;
  `AttachmentMissing`/`Outside`/`TooLarge`) and `check_configuration` (l. 524, re-assembles and compares the
  configuration facts). The test pack `test-table` (tests/acceptance/tests/configuration/mod.rs l. 425) is a
  plain configuration with one attachment. All as recorded; IL-b's API is as R-TWd-1 assumed.
- weather (TW-b, merged): `WeatherConfigured { seed, rules, record: Option<RecordRef> }` with `RecordRef` an
  empty enum; `Origin { Rule }`; `ClimateState { configured, today, now, chain }` — note: `configured` is the
  whole `WeatherConfigured`, so the series must not be stored there (TWd-D2).
- AC-1 `world_delta_failures` (l. 1235) refuses `data/` ("present in one pack only"); `GENERIC_PACKS =
  ["calendar", "weather"]` (l. 1030).
- `Cargo.toml` members list `tools/cli` explicitly; `Cargo.lock` has no `ureq`; `.gitattributes` has only
  `*.bin`/`*.glb binary`; root `NOTICE` lists asset records; DEP-8 table at l. 303; `DECISIONS.md` ends at
  ARC-68 with 85 ids.
- Calendar: `mineworld_calendar::civil::{is_leap, days_in_month}` and `CalendarDate::{new, days,
  from_days, next}` are public.
- Digest method (sha-256 of every `run` output line but `wall`): bodies-yard 30 d seed 7 on the base gives
  `bd6a1002…80e6` = E-TWb-0.

### Commit ledger

| # | Implementation | Deterministic validation | LLM logic review |
| --- | --- | --- | --- |
| C1 | [x] DEP-31 appended after ARC-68 (data source table, `.dly` input, our own CSV labelled modified, provenance without a digest, the tool, `ureq =3.4.2` with `rustls`/ring behind `fetch`, limitations); DEP-8 row (NOAA GHCN-Daily, CC0); ARC-35 note (QTWd-1 ruling, 2026-10-09); `systems/weather/README.md` "The record format"; `tools/weather-fetch/{Cargo.toml,README.md,src/main.rs}` skeleton (no dependency); root `Cargo.toml` member; root `NOTICE` pointer | [x] `check_doc_headings.py`: 192 sections, none duplicated — PASS; `check_decision_ids.py`: 86 ids (85 + 1), all distinct — PASS; `cargo check -p mineworld-weather-fetch` clean; `Cargo.lock` gains the member only | [x] DEP-31 does not restate ARC-68 (it points to it for the generator and LARS-WG); the licence wording matches §3.2 and E-TWd-1 (CC0, "no restrictions", attribution requested, no endorsement, modified data not presented as original); "modified (reshaped and gap-reported) … not endorsed by NOAA" present; terms (`System Pack`, `World Pack`) as defined |
| C2 | [x] `src/record.rs` (new): `HEADER`, `RecordRow`, `RecordFile`, `decode` (BOM, LF/CRLF, refusals with line), `encode` (the tool's writer), `PackedDay`/`PackedDays` (9 bytes, hex), `Fill`, `pack` (gaps), `Station`, `RecordSeries` (`record_date`, `day`: SD-TW-d-4, the only leap logic); `configuration.rs` (`source: rules \| record`, `record:` iff record, `fill:` only with record, `RecordConfiguration { station, data: Attachment, first_year }`); `event.rs` (`WeatherConfigured.record: Option<RecordSeries>`, `RecordRef` removed; `without_record`); `day.rs` (`Origin::{Record, Filled} { date }`); `generate.rs` (`record_day`, SD-TW-d-6); `process.rs` (`RecordProcess` "weather-record", `RecordState`; `ClimateState.epoch_year`, skipped when absent); `system.rs` (`attachments`, `seed` decoding into `weather-record-invalid`, the record Process started after the climate, `day-began` replays or draws, `weather-day` folds the epoch in record worlds); `lib.rs` exports (and `CalendarDate` re-exported); tests `src/record/tests.rs`, `tests/record.rs`, `tests/configuration.rs` (the TW-b `source: record` refusal case replaced, TWd-D3) | [x] E-TWd-2: `cargo test -p mineworld-weather` lib 20, configuration 3, record 6, weather 8, diurnal 1, no_float 1, all pass; `-p mineworld-installed-systems -p mineworld-worldpack` pass; clippy `-p mineworld-weather --all-targets --all-features -D warnings` clean; fmt clean. M-TWd-2, M-TWd-3b killed; `git grep MUTATION` empty | [x] the fold equals the state: the climate state is written only in `react` from fact payloads (`with_day`, `with_now`, `with_epoch` from the `weather-day`'s date), and (g) is byte-identical after a resume across a new year; the record Process is never rewritten (`grep 'RecordProcess>('` finds only `state_for`; (f) compares its state before and after 40 days); no path handling in the pack (no `std::fs`/`Path` in `src`); leap logic once (`RecordSeries::record_date`; `is_leap` otherwise only counts a year's days); packing cannot overflow: decode bounds tmax/tmin to −900 … 600 (i16), prcp ≤ 65 535 (u16), awnd ≤ 255 (u8), wdf2 ≤ 360 → half ≤ 180 (u8), and the hex decoder re-checks every bound; rules worlds unchanged (TWd-D2) |
| C3 | [x] `tools/weather-fetch`: manifest (lib + bin `weather-fetch`; `clap`, `mineworld-weather`, `serde-saphyr`; `ureq =3.4.2` optional, `default-features = false, features = ["rustls"]`, feature `fetch`); `src/lib.rs` (`reshape_bytes`, `write`, `date_argument`), `dly.rs` (readme §III columns; refusals by line), `reshape.rs` (rows by the pack's `RecordRow`, the report), `notice.rs` (§6.4 text, provenance, no clock), `fit.rs` (SD-TW-d-8c in `i128`), `fetch.rs` (`cfg(feature = "fetch")`, no retry), `main.rs` (clap; `fetch` without the feature explains how to get the file); tests `reshape.rs`, `fit.rs`, `confined.rs`; DEP-31 gains the feature tree's licences | [x] E-TWd-3: `cargo test -p mineworld-weather-fetch`: confined 1, fit 2, reshape 3, all pass; clippy `-p mineworld-weather-fetch --all-targets --all-features -D warnings` clean (compiles ring/rustls/ureq); fmt clean; criterion 4 commands verbatim in E-TWd-3. M-TWd-1, M-TWd-4 killed (TWd-F1) | [x] the tool owns no CSV format: rows are `mineworld_weather::RecordRow`, written by `record::encode` and re-read by `record::decode` before anything is written; the rules are decoded by `WeatherConfiguration` before `fit` returns. No float in the tool's source (the scan test). The NOTICE carries the attribution, both citations, CC0, "MODIFIED DATA … not endorsed by NOAA", the URL, the retrieval date and the provenance counts (§6.4, DEP-31, SD-TW-d-12). ring needs only a C compiler: the CI image is `rust:1.97.1-slim-trixie` (gcc), and `fast`'s clippy is the only `--all-features` build; the `platforms` layer (macOS, Windows) builds `mineworld-cli` and S16's tests, never the feature — no material stop (5) |
| C4 | [x] `target/debug/weather-fetch fetch --station USW00023188 --out target/tw-d/noaa/USW00023188.dly` (the tool built with `--features fetch`; 1 of ≤ 3 fetches); `reshape --input … --station USW00023188 --from 2015 --to 2024 --retrieved 2026-10-10 --out-dir worlds/market-town/data/weather --name san-diego-usw00023188-2015-2024`; `.gitattributes` `worlds/*/data/**/*.csv text eol=lf`; the report into E-TWd-4 | [x] E-TWd-4: (a) 3 653 data rows; (b) 116 705 bytes (< 1 MiB); (c) one gap (2018-07-06, 1 day; TMAX Q-flagged), WT01 88 … 149 and WT02 8 … 26 days every year (CP-TW-d 2 applies; R-TW-5 does not occur); (d) re-run with the same arguments → `git diff --exit-code` on the staged CSV and NOTICE clean, report identical; (e) NOTICE "116705 bytes, 3654 lines (LF)" = `wc -c -l` of the committed CSV; `git check-attr`: text set, eol lf | [x] the NOTICE carries the attribution, both Menne et al. 2012 citations with DOIs, "MODIFIED DATA … not endorsed by NOAA", the URL and "retrieved 2026-10-10 (UTC)"; the licence is CC0-1.0 as E-TWd-1 / §3.2 |
| C5 | [x] `worlds/market-town/configure/weather.yaml` = `weather-fetch fit --input <the CSV> --base <TW-b's file> --out … --station USW00023188 --data data/weather/san-diego-usw00023188-2015-2024.csv --first-year 2015` (`source: record`, `fill: rules`, seed 19, the fitted months); `worlds/market-town/README.md` one sentence; `ac1_composability.rs` (`files_under`, `data_failures`, `data/` admitted in Market Town for allow-listed packs' attachments and their NOTICE, refused in Social Café; unit test `data_admits_only_attachments_and_their_notice`); `tools/weather-fetch/tests/market_town.rs` (criterion 7 fit-equality; criterion 5); `systems/weather/tests/record_checkpoint.rs` (CP-TW-d, opt-in) and `moves_nobody.rs` gains `the_record_moves_nobody_but_process_ids` (opt-in); `src/rules/tests.rs` and `src/fixture.rs` no longer name the old table's values (TWd-D11) | [x] E-TWd-5 … E-TWd-8: CP-TW-d PASS; criterion 5 PASS (max \|Δ\| 14 ‰; spells 1.859 vs 1.844 days); criterion 7 PASS; INV-TW-1 PASS (four digests and the validate outputs = main's); the new market-town baseline `d5db8988…22ee`; TW-b criterion 5 re-checked (process ids +1 only, the TWb-F1 class); AC-1 15/15. M-TWd-5, M-TWd-10, M-TWd-A1, M-TWd-A2 killed. Town runs 8 of 8 (counting the 200-day leg) | [x] the AC-1 change admits nothing but the attachments the allow-listed packs' configurations name (read through the loader, `FoundConfiguration::configuration.attachments()`) and a `NOTICE` in the same directory; it is Market Town only, and `data/` in Social Café is named; checks 1 and 2 untouched. The fitted file's header names the tool, its version and the data file, and the formulas; the rules still bound-check (the pack decodes the file; `validate` accepts the world). A rules world's bytes are unchanged (E-TWd-5) |
| C6 | [x] merge of origin/main @ cf18713 (70db0b0; clean, no conflict; main's `.gitattributes` now has `* text=auto eol=lf`, and TW-d's narrower line stays as frozen); this ledger; `handoff-tw-d.md` | [x] E-TWd-9: the full gate on 70db0b0; CI on the exact final head is recorded in the PR and the handoff, not here | N/A — records only |

### Evidence

```text
E-TWd-1  Licence and format re-read 2026-10-09 with WebFetch (material stop (6) check):
         https://www.ncei.noaa.gov/pub/data/ghcn/daily/readme.txt — §III: ID 1–11, YEAR 12–15, MONTH 16–17,
         ELEMENT 18–21, then VALUEn/MFLAGn/QFLAGn/SFLAGn in 8 columns from 22 (VALUE31 262–269); PRCP tenths
         of mm, TMAX/TMIN tenths of °C, AWND tenths of m/s, WDF2 degrees; missing −9999; WT01 fog (may
         include heavy fog), WT02 heavy fog, WT03 thunder, WT08 smoke or haze, WT13 mist, WT14 drizzle,
         WT16 rain, WT21 ground fog; QFLAG blank = "did not fail any quality assurance check"; citations
         Menne et al. 2012 doi:10.1175/JTECH-D-11-00103.1 and doi:10.7289/V5D21VHZ. No licence text in the
         readme. https://registry.opendata.aws/noaa-ghcn/ — "made available under the Creative Commons 1.0
         Universal Public Domain Dedication (CC0-1.0)… There are no restrictions on the use of the data";
         "NOAA requests attribution"; "not permissible to state or imply endorsement"; "If you modify NOAA
         data, you may not state or imply that it is original, unaltered NOAA data". Equal to E-TWd-pre-1
         and §3.2 → no material stop (6). (The data.gov catalog URL of §3.2 now returns 404; the NODD
         registry page is cited instead.)
E-TWd-2  C2, 2026-10-09, macOS arm64, dev, `cargo test -p mineworld-weather -- --nocapture`:
         lib 20 (5 new: record), configuration 3, record 6, weather 8, diurnal 1, no_float 1 — all pass.
         (a) Criterion 6: the fixture as LF, CRLF, CRLF+BOM and without a final newline → the same
             `weather-configured`, 16 562 bytes each, byte-equal. Unit: the same four forms decode to
             equal rows. PASS.
         (b) Refusals, each "line N: …": header (line 1), a date before the line before, a repeated
             date, a skipped day, a file beginning on 2015-01-02 (line 2) or ending on 2015-12-30, a
             non-integer cell ('19.5'), awnd 300, fog 2, 7 cells, a tab, a quoted cell, a stray \r, a
             non-ASCII byte, 2015-02-30; empty file; header only. At assembly through WorldPack::read:
             `fill: none` → "weather-record-invalid … data/weather/test-2015-2016.csv: line 183:
             2015-07-01 … 2015-07-05 are missing, more than 3 days, and `fill` is `none`";
             first_year 2017 → "… first_year 2017 is outside the record's years 2015 … 2016". PASS.
         (c) Criterion 2 (epoch 2026-10-08, file 2015–2016), on `weather-day.origin`:
             first_year 2015: 2026-10-08 → Record 2015-10-08; 2027-02-28 → 2016-02-28; 2027-03-01 →
             2016-03-01; 2028-02-29 → 2015-02-28; 2028-03-01 → 2015-03-01; 2027-10-08 → 2016-10-08 (its
             empty wind cell → the month's 25); no day replays 2016-02-29. first_year 2016: 2026-10-08
             → 2016-10-08; 2028-02-29 → 2016-02-29. Each record day's TMAX, TMIN, PRCP, fog and thunder
             equal the fixture row's. PASS.
         (d) 2028-03-10/11 → Filled {2015-03-10/11} with 2015-03-09's values; 2028-07-01 … 05 → five
             Rule days, each equal to `weather_day(rules, seed, its day-began, the day before's
             chain)`, the first from record 2015-06-30's carry; 2028-07-06 → Record 2015-07-06. PASS.
         (e) Criterion 3 through WorldPack::read + check_configuration: the file rewritten with CRLF →
             Ok; one value changed (2015-01-01 tmax 140 → 141) → ConfigurationDrift { system: weather }.
             PASS.
         (f) Largest climate state over 40 days at 13:00: 6 734 bytes ≤ 8 192; the record Process
             state is 13 255 bytes for 731 days (about 18 bytes a day, as SD-TW-d-5 predicts), unchanged
             after 40 days; the climate state holds no record; epoch_year 2026. PASS.
         (g) Stopped at day 50 14:30, resumed through day 120 (crossing into 2027): 70 weather-day, the
             last replaying 2015-02-05 from the saved epoch; facts byte-identical to the uninterrupted
             world's. PASS.
E-TWd-3  C3, 2026-10-09, macOS arm64, dev, `cargo test -p mineworld-weather-fetch -- --nocapture`: 6 pass.
         (a) Criterion 1 on the hand-made .dly (station USW00099999, 2016; Jan–Mar elements, WT01 Jan/Feb,
             WT03 Feb, March without WT lines): reshape twice, the second 1.1 s later → CSV, NOTICE and
             report byte-equal; the CSV decodes with the pack's decoder (366 rows, LF only); the NOTICE
             names the URL, "retrieved 2026-10-09 (UTC)", MODIFIED DATA, "not endorsed by NOAA", both
             DOIs, CC0-1.0, the command, "input USW00099999.dly, 4860 bytes", "output test-2016.csv,
             8491 bytes, 367 lines (LF)". PASS.
         (b) 2016-01-05 (−9999) and 2016-01-10 (Q-flag 'X') → empty TMAX; WT01 on Jan 3–4 → fog; WT03 on
             Feb 15 → thunder; March (no WT line) → 0; April–December (no line) → empty; report: "TMAX 1"
             flagged, gap runs 2016-01-05 (1 d), 2016-01-10 (1 d), 2016-04-01 … 2016-12-31 (275 d). PASS.
             Failure cases: a 268-character line → "line 1: a .dly line is 269 …"; a second station →
             "line 19: station USW00011111, but the file began with USW00099999"; MONTH 13 refused;
             --station mismatch, --from after --to, years outside the file refused. PASS.
         (c) fit's oracle: 100 years (36 524 days, 2001–2100) drawn by the pack's generator from a known
             table (p_wd 40 … 260, p_ww 250 … 580, tmax 150 … 260, tmin 80 … 168, noise 30, ρ 600, no
             wet shift): p_wd within ±20 ‰ in every month (max |Δ| 14, Aug); tmax/tmin within ±3 (max
             |Δ| 2); p_ww within its tolerance (TWd-D8; max |Δ| 51 in May, tolerance 68); t_noise
             recovered as 30 in all 12 months, ρ 568 … 599; overcast copied; byte-identical twice. PASS.
             No f32/f64 in the tool's src (7 files). PASS.
         (d) Criterion 4 (INV-TW-7), verbatim (target/tw-d/criterion4.txt):
               $ cargo tree -e normal -i ureq --workspace --all-features
               ureq v3.4.2
               └── mineworld-weather-fetch v0.1.0 (…/tools/weather-fetch)
               $ cargo tree -p mineworld-cli -i ureq            → error: package ID specification `ureq`
                                                                  did not match any packages [exit 101]
               $ cargo tree -p mineworld-cli -i ureq --all-features      → the same
               $ cargo tree -p mineworld-server -i ureq                  → the same
               $ cargo tree -p mineworld-server -i ureq --all-features   → the same
             Exactly one dependent. PASS. tests/confined.rs: in Cargo.lock only mineworld-weather-fetch
             depends on ureq and nothing depends on mineworld-weather-fetch. PASS.
         (e) `cargo tree -p mineworld-weather-fetch -i ureq` (no features) → no match; `cargo tree -p
             mineworld-weather-fetch -e normal | grep -c ureq` → 0. The default build does not compile
             ureq. PASS. With `--features fetch` it builds ring 0.17.14, rustls 0.23.45, ureq 3.4.2 in
             7.6 s on macOS (Apple clang).
E-TWd-4  C4, the real record, 2026-10-09 18:29 PDT (macOS arm64):
         fetch (1 of ≤ 3; the only network access of this PR, through the tool's documented path):
           url        https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/USW00023188.dly
           retrieved  2026-10-10 (UTC)
           bytes      4300290        (15 927 lines; first line "USW00023188193907TMAX  244  0 …")
         The .dly file stays untracked under target/tw-d/noaa/ (provenance: fetched by this session
         with the tool, not by hand).
         reshape report (target/tw-d/reshape-report.txt):
           rows 3653 (2015-01-01 … 2024-12-31)
           missing (TMAX TMIN PRCP AWND WDF2; any of the first three): 0 every year but 2018: 1 0 0 0 0 1
           gap runs 1 (longer than 3 days: 0): 2018-07-06 … 2018-07-06 1 d
           weather types, days present by year:
             year  WT01 WT02 WT03 WT08 WT13 WT14 WT16 WT21
             2015    88    8    6   74    0    0    0    0
             2016    97   15    6   70    0    0    0    0
             2017   107   21    3   81    0    0    0    0
             2018    96   14    2   84    0    0    0    0
             2019   118    9    7   84    0    0    0    0
             2020   130   26    3  115    0    0    0    0
             2021   110   11    8  120    0    0    0    0
             2022   102   10    6   79    0    0    0    0
             2023   149   10    7  101    0    0    0    0
             2024   127   23    3  107    0    0    0    0
           fog days 1127, thunder days 51; quality-flagged values dropped: TMAX 1
         (a) 3 653 rows. PASS. (b) 116 705 bytes, 3 654 lines. PASS. (c) recorded: WT01/WT02 populated
         in every year, 2016 included, so CP-TW-d 2 applies; WT13/14/16/21 never (F-TWd-1 confirmed
         over the whole decade). (d) the same command again → CSV and NOTICE byte-identical (`git diff
         --exit-code` against the staged files) and the report identical (`cmp`). PASS. (e) the NOTICE's
         "116705 bytes, 3654 lines (LF)" = `wc -c -l`. PASS.
         Spot rows: 2015-10-08,278,194,0,17,310,1,0; 2016-10-08,300,167,0,20,300,0,0;
         2018-07-06,,200,0,30,330,0,0.
E-TWd-5  Worlds without weather data are byte-identical — on 38ce8c6 (C1–C4; Market Town still TW-b's
         rules), dev, macOS arm64, digest = sha-256 of every `run` output line but `wall` (TW-a's method):
           market-town 300 d seed 7 (no save)   90479fd8631a3f9c88ddc9c05720fbf8fd8fbc5b1573d6abd47e7351b9d1ae57
                                                = E-TWb-4's baseline, byte for byte; 375 527 facts. PASS.
           the same with --save target/tw-d/rules-save: 375 527 facts, history fingerprint 142354b7…615c
           (equal to the no-save run's), 2.7 GB. Town runs 1, 2.
E-TWd-6  INV-TW-1 and the new baseline, on the C5 tree (c5 code + fitted weather.yaml), dev:
           social-cafe 300 d seed 7     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
                                        = E-TWb-0/-4. PASS. 365 330 facts.               (town run 4)
           bodies-yard 30 d seed 7      bd6a1002…80e6 = E-TWb-0. PASS.
           validate social-cafe / bodies-yard / market-town   ebcd60a0… / 7356b8f8… / 6368595a…0318
                                        = E-TWb-0/-4 (validate prints no weather payload). PASS.
           long_run (bodies, second process payload)          4 019 622 B, fdcf28d6…4445 = E-TWb-0. PASS.
           long_run_objects                                   612 410 B, e018c85e…d2be = E-TWb-0. PASS.
           market-town 300 d seed 7 (record)
             d5db8988bb9d8c33ec8e1cf1ba906d58d1fbd49d2a4ad7bc2d2a69b0b0a922ee — THE NEW MARKET-TOWN
             BASELINE; 375 619 facts, faults 0, 27.0 s.                                    (town run 3)
           Against the rules run (E-TWd-5), every summary line but the day lines is equal except
           "weather-changed 368" → "460" and the history line: every non-weather per-type count,
           request count and activity line is the same.
E-TWd-7  TW-b's criterion 5 re-checked on the record world (C5 (e)), the record save (300 d seed 7,
         375 619 facts; town run 5) against E-TWd-5's rules save, `the_record_moves_nobody_but_process_ids`:
           374 857 non-weather facts in each; instant, type, visibility and subjects equal for every one;
           32 953 differ in payload, every differing leaf a Process id exactly one higher:
             agenda-changed.routine 11 712, group-activity-started.activity 6 651,
             group-activity-ended.activity 6 650, left-group-activity.activity 4 481,
             joined-group-activity.activity 3 459; every type's count equal.
         The record world's `weather-record` Process takes one id at genesis — the TWb-F1 class, the same
         32 953 leaves TW-b found for the climate Process. PASS in the form TWb-F1 proposed (and TW-b
         merged with); the frozen payload-bytes form would fail exactly as TW-b's did. TWd-F2.
E-TWd-8  CP-TW-d (§18.6) and criteria 5, 7, on the C5 tree:
         `run market-town --seed 1 --days 365 --save U` (90.3 s; 456 677 facts) and `--days 200 --save R`
         then `--days 365 --save R` ("resumed … at revision 193821 (snapshot 193792 + 29 re-executed)";
         456 677 facts) — town runs 6, 7, 8. tests/record_checkpoint.rs (opt-in):
           4. U and R hold the same fact rows byte for byte. PASS.
           1. wet days (≥ 0.3 mm) dated December–February 19, June–August 0. PASS.
           2. 16 fog days in May–July (world 2027, record 2016). PASS.
           3. world 2026-10-08 → Record {2015-10-08}: TMAX 278, TMIN 194, PRCP 0 = the CSV row; world
              2027-10-08 (day 365, instant 31 536 000, the run's last) → Record {2016-10-08}: 300, 167, 0 =
              the CSV row. PASS. All 366 days are record days (no gap falls in 2015-10-08 … 2016-10-08).
         Criterion 5 (tools/weather-fetch/tests/market_town.rs), wet ‰ record / 100 fitted years:
           Jan 229/224 Feb 194/200 Mar 251/243 Apr 106/104 May 116/130 Jun 23/27 Jul 12/18 Aug 16/15
           Sep 56/52 Oct 54/54 Nov 130/144 Dec 174/169 — max |Δ| 14 ‰ (May, Nov) ≤ 30. Pooled mean wet
           spell 1.844 days (225 spells) / 1.859 (2 262 spells), Δ 0.8 % ≤ 15 %. PASS.
         Criterion 7: `fit` of the committed CSV with the committed file as the base is byte-equal to it
           (and re-running the tool with its own output as the base reproduces it). PASS.
         AC-1: `cargo test -p mineworld-acceptance --test ac1_composability`: 15 passed (14 + the data/
           unit test). PASS.
E-TWd-9  Full gate on 70db0b0 (C1–C5 + origin/main cf18713 merged; clean), macOS arm64, 2026-10-09
         18:47–18:58 PDT (target/tw-d/gate.log, gate-test.log):
           cargo fmt --all --check                                          exit 0          PASS
           check_doc_headings.py   192 numbered sections, none duplicated                    PASS
           check_decision_ids.py   88 ids, all distinct                                      PASS
           check_ci_pins.py        toolchain pins agree                                      PASS
           check_scratch.py scan   188 test sources, none outside test-support (2 exempt)    PASS
           cargo clippy --workspace --all-targets --all-features -- -D warnings   exit 0   PASS
           cargo test --workspace --no-fail-fast   exit 0: 204 result lines, all ok;
             902 passed, 0 failed, 22 ignored (the opt-in save checks among them)        PASS
           check_scratch.py left --target-dir target   no scratch left                       PASS
         Later commits change only the plan documents. Full workspace test runs: 1 of 2.
```

### Mutations

```text
M-TWd-2  record.rs `record_date`: a world 29 February in a non-leap record year → 1 March →
         unit `the_record_date_keeps_month_and_day_and_loops` FAILS (2015-03-01 against 2015-02-28) and
         `world_dates_replay_record_dates_and_gaps_are_filled_or_drawn` FAILS "2028-02-29: Record
         {2015-03-01} against Record {2015-02-28}". Killed. Reverted.
M-TWd-3b record.rs `lines`: the trailing \r no longer stripped → the CRLF checkout is no longer the same
         record: `a_changed_value_is_drift_and_a_crlf_checkout_is_not` FAILS ("CRLF is the same record:
         … weather-record-invalid … line 1: a carriage return inside the line"), and both (a) tests
         fail. Killed — as a refusal rather than as drift, because the decoder refuses a stray \r
         (SD-TW-d-3) before any value could differ. Reverted; `git grep MUTATION -- '*.rs'` empty.
M-TWd-1  notice.rs: the NOTICE gains "written <unix seconds>" → `reshape_is_byte_reproducible_and_
         writes_the_packs_format` FAILS on "the NOTICE" (…1791595649 against …1791595650). Killed by the
         1.1 s gap between the two runs. Reverted.
M-TWd-4  tools/cli/Cargo.toml: `mineworld-weather-fetch` as a dev-dependency with `fetch` →
         `cargo tree -e normal -i ureq --workspace --all-features` STILL lists only the tool (a dev edge is
         not a normal edge: TWd-F1), but `cargo tree -p mineworld-cli -i ureq` names
         "[dev-dependencies] └── mineworld-cli", and tests/confined.rs FAILS: left ["mineworld-cli"],
         right []. Killed by the second command and the lock test. Reverted; `grep -rn MUTATION tools
         systems` empty.
M-TWd-5  fit.rs: p_wet_after_wet estimated as p_wet_after_dry; the town's file re-fitted with it →
         criterion 5 FAILS on both halves: Jan 142 against 229 ‰, Feb 122/194, Mar 161/251, Apr 57/106,
         Nov 77/130, Dec 114/174; mean wet spell 1.130 against 1.844 days. Killed. Reverted, re-fitted:
         the file is byte-identical to the committed fit again.
M-TWd-10 weather.yaml: January's p_wet_after_dry 143 → 144 → `the_committed_rules_are_the_fit_of_the_
         committed_record` FAILS ("configure/weather.yaml is not `fit` of the record"). Killed. Reverted
         (cmp equal).
M-TWd-A1 worlds/market-town/data/other.txt → check 3 FAILS: "data/other.txt: not an attachment of an
         allow-listed generic pack's configuration, nor its NOTICE". Killed. Removed (git clean).
M-TWd-A2 worlds/social-cafe/data/weather/NOTICE → check 3 FAILS: "data/: must be absent in Social Café".
         Killed. Removed. `grep -rn MUTATION` over the tree: none.
```

### Deviations and findings

```text
TWd-D1  (bounded) C2's fixture is not a committed `tests/fixtures/record-town/` directory: tests/record.rs
        writes a scratch World Pack (DEP-29) with a two-year CSV generated by a stated rule of the day's
        index, holding exactly the planned features (a 2-day gap, a 5-day gap, 29 February 2016, an
        empty quality-dropped cell). Reason: four line-ending variants and the drift edit each need
        their own copy anyway, a 731-row hand-typed file is unreadable, and a committed fixture outside
        `worlds/*/data` would be subject to a Windows checkout's autocrlf. The rows are written with the
        pack's own `encode`; decoding is tested separately on hand-edited text (unit (b)).
TWd-D2  (bounded; the "worlds without weather data are byte-identical" rule) The climate state keeps the
        configuration *without* the record (`WeatherConfigured::without_record`), because TW-b's
        `ClimateState.configured` held the whole fact and would otherwise carry the 66 KB series
        (SD-TW-d-5). `epoch_year` is `#[serde(skip_serializing_if = "Option::is_none")]` and folded only
        when a record Process exists, and `record: null` is unchanged, so a world of rules has the same
        facts and the same Process state bytes as under TW-b (TW-b's tests all pass unchanged but the one
        refusal case, TWd-D3; the market-town digest check before C5 measures it end to end).
TWd-D3  (bounded) The rules between keys of SD-TW-d-7 (`record:` iff `source: record`; `fill:` only with
        it) are decided in the configuration's `try_from`, and serde-saphyr reports no line and column
        for a refusal raised at the level of the whole file. These refusals name the file and the keys,
        not a position. TW-b's test case "source: record is refused" is replaced: `source: almanac`
        keeps the positional case, and a new test holds the five cross-key and record-block refusals.
TWd-D4  (bounded) `RecordSeries` carries both `file_first_year` and the configured `first_year`
        (SD-TW-d-5 listed one `first_year`; SD-TW-d-4 needs both, and the series alone must locate a
        day). `years` and the day count are checked against each other on decode.
TWd-D5  (bounded) The packed flags carry two "missing" bits, one for the wind speed and one for the
        direction (SD-TW-d-5 said one "wind-missing"), because SD-TW-d-2 takes each from the month
        independently. A direction is stored halved (0 … 180): an odd degree rounds down (GHCN-Daily's
        are multiples of ten); 360 is read back as 0 (north).
TWd-D6  (bounded; SD-TW-d-6 does not say) A record or filled day's grey morning, which GHCN-Daily cannot
        say, is drawn by the rules on a dry day with the same draw index as a rule day (`OVERCAST`), so
        the marine layer still appears on record days. A record minimum at or above its maximum is
        lowered to one below it, as the generator does, so the hours' range stays positive. Thunder is
        kept as recorded even on a dry day (it then affects no hour).
TWd-D7  (bounded) `mineworld-weather` re-exports `CalendarDate`, so the tool can build record rows
        without a direct dependency on calendar (SD-TW-d-8 lists the tool's dependencies).
TWd-D8  (bounded; a statistical correction of C3 (c), which is not a §18.6 criterion) "p_ww within ±20 ‰"
        over 100 years cannot hold by design in a dry month: January of the known table has 203 days
        after a wet day, so the binomial standard error of p_ww is about 30 ‰ and ±20 ‰ is under 1σ
        (first run: 227 against 250, FAIL by chance). The test keeps ±20 ‰ for p_wd (2 500+ dry
        yesterdays a month, 3σ ≈ 18 ‰) and the means, and for p_ww uses max(20 ‰, 3 standard errors from
        that month's own count). M-TWd-5 (p_ww := p_wd) is still far outside it (January 42 against 250,
        tolerance 92). Criterion 5 (C5 (b)) is unaffected: it compares frequencies and spell lengths.
TWd-D9  (bounded; SD-TW-d-8c's wording) t_noise_dc is ⌊√(3 · var · (1 − ρ²))⌋ with the fitted ρ, where
        SD-TW-d-8c wrote "the integer square root of the anomaly variance × 3 / 2". Its stated reason —
        uniform noise on ±n has variance n²/3 — gives n = √(3 · noise variance), and an AR(1) anomaly's
        variance is the noise's / (1 − ρ²); "/ 2" is that factor at ρ ≈ 0.7. With the fitted ρ the
        oracle recovers the generating noise exactly (30 in all 12 months; "× 3 / 2" would give 26).
TWd-D10 (bounded) The tool has a library target (`src/lib.rs`) besides the binary, so its tests call
        `reshape_bytes` and `fit` in process; `reshape` takes an optional `--name` (default
        `<station>-<from>-<to>`), and `fit` takes `--station/--data/--first-year/--fill` to write a
        `source: record` file from a rules base, or keeps the base's own record block. `fetch` exists
        in every build and, without the feature, says how to get the file instead of being an unknown
        subcommand.
TWd-F1  (finding) Criterion 4's first command, `cargo tree -e normal … --workspace`, cannot see a dev-
        dependency (M-TWd-4 survives it). The per-crate commands (default edges include dev) and the new
        `tests/confined.rs` over Cargo.lock catch it, and the lock test runs in CI.
TWd-D11 (bounded) TW-b's unit tests read Market Town's own table (one copy, TWb-D11), and two of them
        named its provisional January values literally (`rules/tests.rs`: "p_wet_after_dry: 147" …). With
        the fitted table they could not find them. They now set a key's value whatever the table holds;
        every refusal, bound and line assertion is unchanged. TW-b's statistics tests (criterion 2 and
        3, the hours) pass on the fitted table as they did on the provisional one.
TWd-D12 (bounded) The AC-1 `data/` rule reads the attachments through the World Pack loader (the
        configurations of the allow-listed packs, `attachments()`), not by parsing YAML in the test, so
        the admission is exactly what the loader reads.
TWd-F2  (finding; the TWb-F1 class) A record world starts a second weather Process at genesis
        (`weather-record`, SD-TW-d-5, frozen), so every later routine and group-activity Process id is one
        higher than in a rules world (E-TWd-7). Instants, types, visibility, subjects, counts, requests and
        every person's activity are unchanged. TW-b merged with TWb-F1 open in its ledger; the operator's
        acceptance of its proposed form is inferred from that merge, not read in a recorded ruling — this
        PR reports the re-check in both forms rather than assume it.
TWd-F4  MATERIAL STOP — the licence policy that landed on main after the freeze rejects the `fetch` tree.
        main @ 0ba037f (#99, S16 E-c) added `deny.toml` (DEP-22) with `[graph] all-features = true`, and
        `fast` now runs `cargo deny check licenses sources bans`. The PR run 38015301810 on the merge
        with main FAILS there: `webpki-roots v1.0.9` (license "CDLA-Permissive-2.0", Mozilla's CA list as
        data) ← `ureq 3.4.2` (feature `rustls`) ← `mineworld-weather-fetch`; nothing else in the graph
        fails (local `cargo deny check` on 376a538: "bans ok, licenses FAILED, sources ok", the one
        rejection). `test` passed on that merge. Audited alternatives:
          - `rustls-no-provider` + `_ring` + `platform-verifier` (tried locally, reverted): pulls
            `webpki-root-certs v1.0.9`, also CDLA-Permissive-2.0, through rustls-platform-verifier.
          - `native-tls`: needs OpenSSL headers in the CI image (material stop (5)).
          - Loading the OS roots ourselves through `rustls-native-certs`: a new dependency beyond
            ureq's tree (material stop (2)).
        Every route to working TLS with rustls carries a CDLA-Permissive-2.0 root list, and `deny.toml`
        is outside §18.10's change set and is the project's licence policy (DEP-22, ARC-55). Smallest
        revisions proposed for the operator:
          (A) one exception in deny.toml, scoped to the crate:
                [[licenses.exceptions]]
                crate = "webpki-roots"
                allow = ["CDLA-Permissive-2.0"]
              with a comment saying it is data (root certificates), reached only through the
              off-by-default `fetch` feature of a developer tool that no world build compiles, plus a
              DEP-31/DEP-22 note. CDLA-Permissive-2.0 is a permissive data licence with no copyleft
              and no attribution requirement for use.
          (B) drop the `fetch` mode and `ureq` (QTW-14's own alternative): `reshape --input` on a file
              downloaded by hand. The data is already committed and its NOTICE records how it was
              fetched. This reverses QTWd-4, so it is the operator's decision too.
        Not done: no edit to deny.toml, no change to the tool's TLS. The PR stays open, NOT READY.
TWd-F3  (process) Three forbidden-list slips by this session, all read-only or no-ops, recorded for
        honesty: an `awk` in a grep pipeline while auditing DECISIONS.md (it printed nothing), an empty
        heredoc to /dev/null, and an `awk` summing the gate's test counts (read-only). No file was
        written by any of them.
```

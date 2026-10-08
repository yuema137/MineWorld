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

<!-- §3 onward follows -->

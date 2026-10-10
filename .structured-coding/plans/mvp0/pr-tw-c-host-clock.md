# PR S19 TW-c — The host clock: live time scale, the host journal, restart pacing, wall cadence, FX-24

## DESIGN DRAFT — awaiting the primary session's review (not frozen)

```text
Design revision:        revision 1 (2026-10-10), first commit on docs/tw-c-design
Approved by / evidence: none yet. Not frozen. Implementation is NOT authorized by this document.
Implementation base:    main at the start of implementation (C0 records it); see Q-TWc-3 for the
                        ordering against 12n-2
Execution contract:     §13 (proposed; filled and confirmed only at the freeze)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** S19, [`step-19-time-weather.md`](step-19-time-weather.md) §4 (two time
domains), §7 (the host clock), §9 (FX-24), §10 (INV-TW-1, -2, -3, -8, -9), §11.2 (the TW-c row), §14.1
(rulings QTW-2, QTW-3, QTW-4, QTW-13) · **Parent:** [`overall.md`](overall.md) S19 and the decision-number
table (ARC-69 is TW-c's) · **Related:** [`step-12-server.md`](step-12-server.md) §16 (S11-B: SD-B5 … SD-B9,
D-SB6) and §18 (S11-D: SD-D6 … SD-D10, QS11D-1); [`step-11-bodies.md`](step-11-bodies.md) SD-N14 (12n-2:
hosted step consults); [`step-20-client-settings.md`](step-20-client-settings.md) §3.10 (SET-c, the consumer
of this PR's route); `step-21-regions-travel.md` (MVP-1) QRT-7 (the single-player "doze").

**Working name:** TW-c. The PR number is assigned when the PR opens.
**Planning base:** `origin/main @ bb62edf` (#140). Planning worktree
`/Users/yuema137/mineworld-worktrees/design-tw-c`, branch `docs/tw-c-design`. This planning session writes
this file only; edits to `overall.md`, `step-19-time-weather.md` and `docs/` are proposed (§12) and are
the primary session's, or this PR's own commits once frozen.

### Binding rulings this design is written under

| Source | Ruling |
| --- | --- |
| Operator, 2026-10-08 (step-19 §1.2) | A day maps to 4 h, 2 h or 1 h (6×, 12×, 24×); default 2 h (12×); changed **live**; in multiplayer only the host or an admin changes it. The scale changes the calendar's rate, never walking, physics or dialogue. |
| Operator, QTW-2 | Full pause: nothing moves; client requests are refused `paused`. (Landed by S11-D.) |
| Operator, QTW-13 | Hosted consult cadence is in **wall** seconds; `--pace` becomes a wall-second cadence, renamed `--cadence`. (Wall cadence landed by S11-B D-SB6; the rename did not.) |
| Operator, relayed with this task (2026-10-10) | **Time scale affects calendar time only**; walking, physics and dialogue run in wall time. **Only a single-player world may fast-forward ("doze"); a multiplayer world never does.** (Consistent with QRT-7 of step-21, ruled 2026-10-09.) |
| Primary session, QTW-3 | The clock routes inside S11-D. S11-D landed pause/resume and answered `time_scale` `409 time_scale_fixed`, leaving the live change to TW-c (QS11D-1). |
| Primary session, QTW-4 | The default scale is content in `world.yaml` `hosting:`, read by the server, never by a pack. |
| ARC-67 (merged, TW-a) | `WorldTime` is calendar time; embodied time is not a world quantity; scale and pause are host pacing whose changes are host records in a host journal, never facts (`ARC-69`, TW-c). |
| ARC-25, ARC-27 | A world's state is its journal re-executed, byte for byte; a headless run is a pace schedule over stateless seeded controllers. Neither may change for a world that is never hosted. |

---

## 1. Goal, in one paragraph

S11-B gave the host a constant `--time-scale`; S11-D made the host clock a pausable segment and put pause,
resume and the `clock` frame behind `/admin/clock`. What remains of S19's host clock is everything that
needs the scale to **change while the world runs**: re-segmenting the clock without a jump, rescheduling
every pending in-server consult so that no hosted Person's wall-clock cadence (strides, lines) jumps,
telling clients, recording each pacing change in a **host journal** in the save (never a fact), restoring
pacing across a restart from that journal, the default scale as World Pack content, the operator's rule that
**only a single-player world may fast-forward**, the `--cadence` rename, and the FX-24 measurement of whether
agendas stay feasible at 24×. Kernel, contracts, presence and every System Pack stay untouched; `run` stays
byte-identical.

---

## 2. Audit — what exists on `main @ bb62edf`, and what TW-c still owes

Step-19 §11.2's TW-c row, item by item. "S11-B" and "S11-D" are the merged PRs (#83 and the S11-D merge);
file and line references are to `main @ bb62edf`.

| # | TW-c row item (step-19 §11.2, §7) | Status | Evidence on `main` | Delta TW-c owes |
| --- | --- | --- | --- | --- |
| A-1 | `HostClock` segments `(world_anchor, wall_anchor, scale, paused)` | **Exists** (S11-D SD-D6) | `server/src/runtime/world.rs:34–91`: `new`, `now_at(Instant)`, `pause_at`, `resume_at`; property test `the_host_clock_is_monotonic_and_jump_free_across_pauses` (l. 244, DA-7) | A third transition, `rescale_at(wall, scale)`, folding the reading into the anchor; the property test extended to random rescales (§5.1) |
| A-2 | Pause / resume | **Exists** | `runtime/control.rs:98–110` `ControlCommand::Pause`; `runtime.rs:288–293` `submit` refused `paused`; `runtime.rs:385–403` `tick` skips consults and advance while paused | None, except journaling each change (A-8) |
| A-3 | Live scale change | **Missing**; explicitly refused | `server/src/admin.rs:229–254`: `ClockChange { paused, time_scale: Option<Value> }`, any `time_scale` → `409 time_scale_fixed`; `PROTOCOL.md` §10 l. 775, §11.2 l. 835 | Accept `time_scale` (§5.4); retire `time_scale_fixed` |
| A-4 | `paused` refusal | **Exists** | `RefusalCode::Paused`; `PROTOCOL.md` §5.5 l. 452 | None |
| A-5 | `clock` frame; `WorldSummary.paused` | **Exists** for pause/resume | `runtime.rs:96–98` `announced: watch::Sender<ClockState>`; `control.rs:107` `send_replace`; `runtime/status.rs:22` `paused`; `PROTOCOL.md` §5.9 l. 571–592 ("on every pause and every resume") | Also sent on every scale change (§5.4); §5.9's sentence widened |
| A-6 | `/admin/clock` routes, bearer check, 404 without token | **Exists** | `admin.rs:54–76`; `tools/cli/tests/admin.rs` DA-1 … DA-10 | None beyond A-3 |
| A-7 | Cadence in wall seconds (QTW-13) | **Exists for a constant scale** | `tools/cli/src/hosted.rs:102–194`: `ReactiveSeat` step = scale; `PacedSeat` lattice `genesis + k·s + m·pace·s`; guard `cadence_is_wall_time_whatever_the_scale` (l. 248) | Live rescheduling (§5.2): the adapters capture the scale at construction (`hosted.rs:112`, `:159`) and nothing can tell them it changed |
| A-8 | Host journal (persistence, additive table) | **Missing** | `persistence/src/sqlite.rs:46–67`: four tables, no `host_journal`; `PROTOCOL.md` §11.3 l. 838–848: "A pause is not persisted … none appears in a save" | New, §6 |
| A-9 | `world.yaml hosting.time_scale` | **Missing** | `worldpack/src/format.rs:43` `WorldManifest` (`deny_unknown_fields`) has no `hosting`; no world declares one | New key (§5.6); `market-town`, `social-cafe` opt in (Q-TWc-4) |
| A-10 | `--pace` → `--cadence` | **Missing** (only the unit changed) | `tools/cli/src/main.rs:124–126` `--pace` "in wall seconds"; `tools/cli/tests/hosted_town.rs:36`; `docs/MODULE_SPEC.md` §8.1 l. 1142, 1167 | Renamed cleanly, no alias (§5.7, Q-TWc-7) |
| A-11 | Start precedence: `--time-scale` > journal > `hosting.time_scale` > 1 | **Missing** | `main.rs:131–132` `--time-scale` has `default_value = "1"` (an explicit flag is indistinguishable from the default) | `--time-scale` becomes optional; resolution in the runtime (§5.5) |
| A-12 | Restart resumes "from the saved instant" | **Partly**: resumes at the last *revision's* instant | `runtime.rs:102–109`: a persisted world's epoch is `world.world().now()`; `runtime.rs:200–206` `checkpoint()` snapshots the head and does not advance; idle advances are never journaled (ARC-25) | A world idle for a while before a graceful stop resumes *earlier* than the last `at` its clients were shown: the HUD would run backwards across a restart. Fixed from the journal's `Stopped` record (§5.5) |
| A-13 | `run --pace` test knob | **Missing** | `tools/cli/src/run.rs:41` `pub const PACE = 900 s`; no flag | §5.8 |
| A-14 | FX-24 measured and recorded | **Missing** | No test or evidence references FX-24 (`grep -rn FX-24` hits step-19 and one S11-D scope line only) | §8 AC-TWc-9, C8 |
| A-15 | Single-player-only fast-forward (operator, 2026-10-10) | **Missing**, and nothing distinguishes single-player | `main.rs:27`: "There is no separate single-player mode"; any `--time-scale ≥ 1` accepted (`tools/cli/tests/walking.rs:699` uses 28 800) | §5.3, Q-TWc-1, Q-TWc-2 |
| A-16 | Hosted step consults (12n-2, SD-N14) rescheduled on a live change | **Not on `main`**: 12n-2 is unmerged | `mvp0/pr-12n2-walk @ 1bf18cc`: `PacedSeat { step: EMBODIED_STEP·scale, walking, last }` (`hosted.rs:136–245` on that branch); SD-N14: "S19's live-rescale pass covers step consults like any pending consult" | Covered by the same seam (§5.2) if 12n-2 merges first (Q-TWc-3) |
| A-17 | Client side: the `clock` reader | **Exists** | `clients/protocol/mineworld/world_client.gd:78–81` signal `clock_changed(at, time_scale, paused)`, `:521–526`; `checks/admin_check.gd` | Only the headless check gains a scale step (C7); the module is not edited |
| A-18 | The World section of the settings menu | **Not TW-c's** | Moved to SET-c by step-20 §3.10 (QSET-6) | None; SET-c consumes this PR's route and frame |

Two further findings the audit made, each resolved inside this design:

- **F-TWc-1 — reactive consult instants follow tick jitter.** `server/src/hosted.rs:81–89`
  `HostedSlot::decide` computes the next instant as `next_consult(now.max(self.next))`, where `now` is the
  tick's clock reading. For `PacedSeat` that is still the next lattice point; for `ReactiveSeat`
  (`after + scale`) it is `tick-now + scale`, so its consult instants depend on when ticks happened to run.
  A host journal cannot then determine what in-server controllers did. Fixed in §5.2 (SD-TWc-4).
- **F-TWc-2 — the answering window straddles a rescale.** `PacedRuleController` answers a line heard at `h`
  at consult `t` iff `t − P < h ≤ t`, `P` = its pace in world seconds
  (`cognition/rule-controller/src/paced.rs:208`). "Answer each line once" holds only while consecutive
  consults are exactly `P` apart (ARC-27 item 3). Across a live change from `s₀` to `s₁` the gap straddling
  the change is neither `pace·s₀` nor `pace·s₁`: scaling up answers some lines twice, scaling down never
  answers some. Fixed in the adapter (§5.2, SD-TWc-5), without touching the cognition crate.

---

## 3. Scope

### 3.1 In scope

1. `HostClock::rescale_at` and a live `time_scale` on `POST /admin/clock` (alone or with `paused`).
2. The in-server seam learns a rescale: `HostedController::rescaled`, and factories bound with the scale.
   Every pending consult rescheduled by one server rule; both CLI adapters re-anchored, including the
   straddling answer window; `HostedSlot`'s next instant made jitter-free.
3. The fast-forward rule: live and start-time scales above the day-length range (24×) only in a world
   started `--solo`; `--solo` requires a loopback listen address.
4. The host journal: typed records in `persistence`, an additive table created lazily, written by the
   server on start, every pause, resume and rescale, and on graceful stop; checked by `verify`.
5. Start resolution of scale and instant from the journal; a fast-forward scale never survives a restart.
6. `world.yaml` `hosting: { time_scale }` (1…24) read by `mineworld server`; `market-town` and
   `social-cafe` default to 12× (Q-TWc-4).
7. `--pace` renamed `--cadence`; `run --pace` added as a test knob (default 900, unchanged).
8. FX-24 measured, as an automated scenario test.
9. Specifications first: `ARC-69`; notes on `ARC-42` and `ARC-44`; `PROTOCOL.md` §§5.9, 10, 11;
   `MODULE_SPEC.md` §§4.1, 8.1.

### 3.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The settings menu's World section (pause, day length buttons), the read-only badge | SET-c (step-20 §3.10) |
| HUD date/time, `world_time` client module, light from the sun, launcher "keep running" and close behaviour | TW-e |
| A launcher that generates an admin token and starts the host `--solo` | TW-e / S23 R-c (they pass the flags this PR adds) |
| "Doze until arrival" automation (raise the scale, restore it on arrival) | S21 (QRT-7). TW-c supplies only the permission rule and the scale change; no route names an instant or a condition |
| Persisting the paused state across a restart | Not done: S11-D's rule stands, a restarted world runs (§5.5) |
| Changing any pack constant (`CONVERSATION_GAP`, `INVITATION_LIFETIME`) | QTW-15, IL-b |
| A remedy if FX-24 fails (content in routines) | A material stop (§13), ruled by QTW-1: content first; never a controller that reads the scale |
| Telling clients whether a world is `--solo` | Not needed by TW-c; a later PR adds it to `WorldSummary` if SET-c or S21 asks |

### 3.3 Invariants this PR must hold

| Id | Invariant | Checked by |
| --- | --- | --- |
| INV-TW-1 | A world that is never hosted is byte-identical: `run` output, `run` saves (rows **and** the SQLite schema) and fact logs unchanged for `social-cafe`, `market-town`, `bodies-yard` | AC-TWc-8 |
| INV-TW-2 | No System Pack, controller *decision rule* or fact reads the scale or pause. The adapters (host side, `tools/cli/src/hosted.rs`) may know the scale; the cognition crate is not edited | review; AC-TWc-10 diff scope |
| INV-TW-3 | `kernel/`, `contracts/`, `systems/` (presence included) are not edited | AC-TWc-10 |
| INV-TW-8 | The host clock never decreases and never jumps across pause, resume, rescale and restart | AC-TWc-1, AC-TWc-5 |
| INV-TW-9 | Every pacing change is a host-journal record; none is a fact, a journal (input) row, a snapshot, or a revision | AC-TWc-4 |
| INV-TWc-1 (new) | A hosted Person's wall-clock cadence (consults, strides, lines) is unchanged by a live rescale, to within one tick | AC-TWc-2, AC-TWc-7 |
| INV-TWc-2 (new) | What in-server controllers do is a function of the world, the seeds and the host journal — not of tick timing | AC-TWc-3 |
| INV-TWc-3 (new) | A world not started `--solo` never runs faster than 24× | AC-TWc-6 |
| ARC-25 | Re-execution of the input journal is unchanged: the host journal is never an input and never compared by replay | AC-TWc-4, existing restart tests |

---

## 4. Reuse

Nothing in this PR is commodity infrastructure that a library would supply better. The clock arithmetic is
four integer formulas over `std::time::Instant`; the journal is one table in the SQLite backend already
adopted (`DEP-2`), encoded with `serde_json` as every other row (`DEP-5`). Considered and declined:
`tokio::time::pause` for deterministic tests (the world thread is a blocking `std` thread outside the
runtime, so a paused tokio clock would not reach it; injected `Instant`s, as DA-7 already does, are
smaller); a time-series or event-store crate for the journal (a second persistence authority, step-19
§4.3 (b)). No new dependency; no `DEP-` record needed.

---

## 5. Design

### 5.1 The clock: rescale is the third segment transition (SD-TWc-1)

`HostClock::rescale_at(&mut self, wall: Instant, scale: NonZeroU32)`:

```text
running   world_anchor := now_at(wall); wall_anchor := wall; scale := s₁
paused    scale := s₁                      (applies from the resume; the anchor is already frozen)
same s    nothing
```

So `now` after the transition equals `now` before it (no jump), and from then on it reads
`world_anchor + ⌊elapsed_ms · s₁ / 1000⌋`. Monotonic by construction, like pause and resume.

### 5.2 Rescheduling in-server consults (SD-TWc-2 … SD-TWc-5)

**SD-TWc-2 — the seam.** `server/src/hosted.rs`:

```rust
pub trait HostedController: 'static {
    fn next_consult(&self, after: WorldTime) -> WorldTime;            // unchanged
    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest>; // unchanged
    fn answered(&mut self, _answer: &HostedAnswer) {}                  // unchanged
    /// The host changed the time scale. `pending` is the instant the server has moved this
    /// controller's pending consult to; from it on, `next_consult` answers at `scale`.
    fn rescaled(&mut self, pending: WorldTime, scale: NonZeroU32);     // new, required
}
pub struct HostedBinding { pub at: WorldTime, pub scale: NonZeroU32 }
pub type HostedFactory = Box<dyn Fn(HostedBinding) -> Box<dyn HostedController>>;
```

`rescaled` has no default: a controller that ignored it would keep the old cadence silently, which is
mutation M-TWc-1. Before a public stable contract, the seam is changed cleanly (`CLAUDE.md` §4 rule 12);
the implementors are the two CLI adapters and three test controllers (`server/src/seats/tests.rs:40`,
`server/tests/seats.rs:257`, and the new runtime test). The factory receives the current scale because a
seat released after a rescale must be rebound at the scale in force, not the one at start.

**SD-TWc-3 — one rescheduling rule, in the server.** On a rescale at world instant `T` from `s₀` to
`s₁`, for every `Hosted` seat with pending instant `p`:

```text
p' = p                                  if p ≤ T  (already due: consulted at the next tick, as now)
p' = T + ⌈(p − T) · s₁ / s₀⌉            otherwise (the same wall wait, in the new world seconds)
slot.next := p';  controller.rescaled(p', s₁)
```

`SeatTable::rescale(at, from, to)` applies it (seats.rs), called from `runtime/control.rs` in the same
command that re-segments the clock — one command on the world thread, so no tick can come between. While
paused the rule applies at the frozen `T`, which is exact: the remaining wall wait is frozen too.

**SD-TWc-4 — jitter-free next instants (F-TWc-1).** `HostedSlot::decide` computes
`next := next_consult(self.next)`, then while `next ≤ now`: `next := next_consult(next)` — the missed-consult
skip of S11-B kept, but the chain starts from the instant the consult was due, never from the tick's reading.
For `PacedSeat` this changes nothing at a constant scale (its lattice points are the same); for
`ReactiveSeat` consults become `bound + n·s` instead of `tick-now + s`.

**SD-TWc-5 — the adapters re-anchor (`tools/cli/src/hosted.rs`).** Both adapters keep their cadence as a
count of **wall seconds**, mapped to world instants through their current segment:

```text
segment       (origin: WorldTime, origin_tick: i64, scale: i64)
instant(n)    origin + (n − origin_tick) · scale         n = the wall-second tick count since binding
ReactiveSeat  every tick
PacedSeat     lattice ticks n ≡ k (mod pace); while walking (12n-2), every tick
rescaled(p', s₁)   origin := p'; origin_tick := the tick p' stood for; scale := s₁
```

At a constant scale and binding at the epoch this is exactly today's `genesis + k·s + m·pace·s` (the
existing unit tests `a_paced_seat_is_consulted_on_run_s_lattice` and `cadence_is_wall_time_whatever_the_scale`
stay green unedited). **The straddling answer window (F-TWc-2):** `PacedSeat` remembers the instant of its
last lattice consult; the first lattice consult after a rescale is decided by
`PacedRuleController::new(seed, t − last_lattice)` — a window exactly as long as the real gap — and every
later one by `PacedRuleController::new(seed, pace·s₁)`. The controller stays stateless; only the adapter
chooses the window, as it already chooses the pace.

**Restart and release are bindings (accepted limitation L-TWc-1).** A seat bound afresh — at server start,
or when a person gives it back — anchors its lattice at the epoch with the scale in force, as today
(`serve.rs:182–193`; `ARC-42`: a controller is built afresh, never resumed). After a world that was
rescaled mid-life restarts, the first window after the restart may overlap or miss the last window before
it, so one line may be answered twice or not at all. This is the same class as the existing release
behaviour and as F-13; recorded, not fixed (Q-TWc-8).

### 5.3 Fast-forward is single-player only (SD-TWc-6; operator rule of 2026-10-10)

```text
day-length range    1 … 24       (the menu offers 6, 12, 24; SET-c shows any other as "custom (n×)")
fast-forward        25 … 3600    (step-19 §7.3's ceiling: one world hour per wall second)
```

- `mineworld server --solo` declares a single-player world. It **requires a loopback listen address**
  (`127.0.0.0/8` or `::1`); `--solo` with any other address stops the server before it listens, naming the
  flag and the address. Loopback is what makes the declaration true: nobody on another machine can join.
  Inference from the connection count was rejected: a controller that must wait, a language model,
  connects as a session (`server/src/hosted.rs:18–20`, `ARCHITECTURE.md` §9), so "one connection" is not
  "one player".
- Without `--solo`, `POST /admin/clock {"time_scale": n}` with `n > 24` is answered
  **`409 fast_forward_solo_only`** and nothing changes; with `--solo`, `n ≤ 3600` is accepted. Above 3600,
  0, a non-integer or a negative number: `400 malformed`.
- The **start-time** scale follows the same rule (Q-TWc-2, recommended): `--time-scale n` with `n > 24`
  needs `--solo`, refused at start otherwise. Existing tests that use large scales on loopback
  (`tools/cli/tests/walking.rs:699` at 28 800, `server_command.rs:469` at 60) add `--solo`. The ceiling
  3600 is for the live route; the flag stays unbounded above with `--solo`, because tests compress days.
- `world.yaml hosting.time_scale` is limited to the day-length range: a World Pack cannot make a world
  fast-forward (refused at load, naming file and key).
- A fast-forward scale **never survives a restart** (§5.5).

### 5.4 The route and the frame (SD-TWc-7)

`POST /admin/clock` body: `{ "paused": bool }`, `{ "time_scale": n }`, or both (`deny_unknown_fields`
kept; `{}` stays `400 malformed`). Both: the scale is applied first, then the pause state, as one world
command; one `clock` frame and at most two journal records result. A repeated value is `200` and changes
and records nothing. The answer is `{ at, time_scale, paused }` as today. Every effective change writes
`ClockState` to the `watch` channel, so the `clock` frame (`PROTOCOL.md` §5.9) is sent on every pause,
resume and **rescale**. Error codes after TW-c: `unauthorized` 401, `malformed` 400,
`fast_forward_solo_only` 409, `world_stopped` 503; `time_scale_fixed` is removed (transitional since
S11-D; "may omit, never redefine", `ARC-41`).

### 5.5 Start: which scale, which instant, and paused or not (SD-TWc-8, SD-TWc-9)

**Scale** (`HostConfig.time_scale` becomes `TimeScaleChoice { flag: Option<NonZeroU32>, pack:
Option<NonZeroU32> }`, resolved in `WorldRuntime::new`):

```text
--time-scale given                           that scale (fast-forward needs --solo, §5.3)
else the journal's last day-length scale     the newest Started/Scaled record with scale ≤ 24
else world.yaml hosting.time_scale           the pack's default
else                                         1
```

**Instant.** A persisted world's host clock starts at `max(world.now(), last Stopped.at)`. `Stopped` is
written by the graceful shutdown (`Command::Shutdown`, after `checkpoint()`), with the host clock's reading
at that moment. Without it (a killed host) the start is `world.now()`, as today. The first tick advances
the world from its last revision's instant to the host's. Every tick before the stop had already advanced
the world to its own reading, and an advance that fires is journaled, so nothing can be due between the
last revision and the last pre-stop tick; whatever falls due in the last tick's interval (at most 100 ms of
wall time) fires at its own instants, exactly as the stopped host would have fired it. This makes CP-TW-c's "`at` resumes from the saved instant" exact and
keeps the HUD from running backwards across a restart (TW-e).

**Paused.** Unchanged from S11-D: a restarted world runs (`PROTOCOL.md` §11.3). The journal's `Started`
record says `paused: false`.

### 5.6 `world.yaml` `hosting:` (SD-TWc-10)

```yaml
# How a host paces this world when it serves it (step-19 QTW-4; never world state, never read by a pack).
hosting:
  time_scale: 12      # world seconds per wall second, 1 … 24; a day lasts 24 h / time_scale
```

`WorldManifest.hosting: Option<Hosting>`, `Hosting { time_scale: Option<DayLengthScale> }`, both
`deny_unknown_fields`; `DayLengthScale` a `NonZeroU32` newtype validated 1…24 at load. Not seeded, not a
`configure:` key, not part of the drift check, ignored by `run` (INV-TW-1). `mineworld validate` reports an
out-of-range value by file and key.

### 5.7 CLI (SD-TWc-11)

```text
mineworld server <world> … [--town [--seed N] [--cadence SECONDS]] [--time-scale N] [--solo] …
  --cadence SECONDS   how often each paced seat is consulted, in wall seconds (default 5); was --pace
  --time-scale N      world seconds per wall second; default: the save's journal, then world.yaml, then 1
  --solo              a single-player world: loopback only; may fast-forward above 24×
mineworld run <world> … [--pace SECONDS]
  --pace SECONDS      world seconds between a seat's consults (default 900); a measurement knob (FX-24)
```

`--pace` on `server` is removed, not aliased (Q-TWc-7). `run --pace` must be a multiple of 12n-2's
`RUN_STEP` once 12n-2 is on the base (refused by name otherwise). The startup line reports the resolved
scale and where it came from: `time scale 12 (from world.yaml)`, `(from the save)`, `(from --time-scale)`.

### 5.8 FX-24, the measurement (SD-TWc-12)

Step-19 §9.2, unchanged in its thresholds, measured in `run` at the equivalent pace `cadence × scale` with
cadence 5: `--pace 30` (6×), `60` (12×), `120` (24×), 7 world days, seed 1, `market-town`. Metrics reuse
`tools/cli/tests/routines.rs`'s journey extraction (agenda-changed → arrival at the agenda's place) and
employment's `shift-started` presence flag. **Note on exactness:** with 12n-2 on the base, `run` takes a
stride every `RUN_STEP` = 30 world s, while a hosted walker at 24× strides every 24 world s; `run` walks
slower in world time, so a PASS in `run` is conservative for hosting. The 6× and 12× legs are exact only
for lattice consults. Recorded with the result.

### 5.9 Size plan

`server/src/host.rs` is 499 lines and `tools/cli/src/main.rs` 526. The `HostedBinding`/factory change adds to
`host.rs`; `HostConfig` (l. 217–246) and its `Default` move to `server/src/host/config.rs` in a pure move
first (C3's first step) so `host.rs` stays under 500. `main.rs` gains three flags and loses one; its
`Server` arguments move into `tools/cli/src/serve.rs`'s `ServeRequest` construction if it grows past 530
(reviewed, §30 of the standards). `runtime.rs` (449) gains only the tick's call-through; rescale and journal
handling live in `runtime/control.rs` (126) and a new `runtime/pacing.rs`.

---

## 6. Persistence and replay

### 6.1 The record (SD-TWc-13)

`persistence/src/host.rs` (new):

```rust
pub struct HostRecord { pub at: WorldTime, pub revision: WorldRevision, pub change: PacingChange }
pub enum PacingChange {
    Started { scale: NonZeroU32, paused: bool },  // every start of a persisted world
    Paused,
    Resumed,
    Scaled { to: NonZeroU32 },
    Stopped,                                      // graceful shutdown only
}
```

`at` is the host clock's reading at the change; `revision` the save's head when it was written (orders the
record against inputs). Encoded with `format::encode` (`serde_json`, externally tagged, field order as
declared): `{"at":4112,"revision":37,"change":{"Scaled":{"to":24}}}`. A golden test pins the bytes of each
variant.

### 6.2 The table (SD-TWc-14)

```sql
CREATE TABLE IF NOT EXISTS host_journal (
    seq       INTEGER PRIMARY KEY,
    revision  INTEGER NOT NULL,
    at        INTEGER NOT NULL,
    record    BLOB    NOT NULL
);
```

- **Created lazily**, in the transaction of the first `append_host_record`, never by `SCHEMA`. A save that
  is never hosted — every `run` save — has exactly the four tables it has today, so INV-TW-1 holds for the
  file's schema as well as its rows. Reading a save without the table yields an empty journal.
- `PersistenceBackend` gains `append_host_record(&HostRow)` and `host_records() -> Vec<HostRow>`
  (`HostRow { revision, at, record: Vec<u8> }`, meaning-free like every backend row); `PersistentWorld`
  gains `record_host(change, at)` and `host_records() -> Vec<HostRecord>`.
- One record per transaction, `synchronous` as the save's durability (FULL for hosted worlds). Pacing
  changes are human-rate; the write is bounded and on the world thread like a commit (step-12 I-11).
- **No `SAVE_FORMAT` bump.** No existing row's encoding changes; the table's absence has a meaning (empty).
  A pre-TW-c binary resuming a TW-c save ignores the table; a TW-c binary resuming an older save sees an
  empty journal. Recorded in `ARC-69`.

### 6.3 Replay and verification, byte for byte

- `verify` (ARC-25) re-executes the **input** journal exactly as today; the host journal is never an input,
  never re-executed and never compared with regenerated bytes. `ReplayDiverged` cannot be caused by it.
- `verify` additionally checks the host journal's coherence and refuses a save that violates it, naming
  the record: records decode; `seq` order has non-decreasing `revision`; `at` never decreases across the
  whole journal, restarts included (INV-TW-8 over the save's lifetime); `revision ≤ head`.
- **What the journal adds (INV-TWc-2).** Facts and inputs already replay without it. The journal records
  the only remaining input to hosted behaviour: when the host changed pacing, and to what. With SD-TWc-4,
  in-server consult instants are a function of `(bindings, seeds, host journal)`; AC-TWc-3 shows it.
- **ARC-27.** `run` is untouched except the `--pace` flag (default 900): its saves, digests and the
  300-day seed-7 fingerprints stay byte-identical.

---

## 7. Test ownership

| Test | Owner | New / edited |
| --- | --- | --- |
| `server/src/runtime/world.rs` DA-7 property test | S11-D, extended here | edited: rescale steps added (AC-TWc-1) |
| `server/src/runtime/tests.rs` (new): AC-TWc-2, AC-TWc-3 over an injected wall clock | TW-c | new |
| `server/src/seats/tests.rs` `SeatTable::rescale` | TW-c | new cases |
| `server/tests/admin.rs` DA-8 table | S11-D | edited: the two `time_scale_fixed` rows become AC-TWc-6's rows |
| `persistence/tests` host journal: lazy table, golden bytes, verify coherence | TW-c | new |
| `tools/cli/src/hosted.rs` unit tests: re-anchor, straddle window | TW-c | new; existing two unedited |
| `tools/cli/tests/admin.rs` DA-3, DA-8 | S11-D | edited: scale added to DA-3's route list; DA-8's 409 row |
| `tools/cli/tests/host_clock.rs` (new): CP-TW-c, restart, fast-forward | TW-c | new |
| `tools/cli/tests/walking_pace.rs` (12n-2) — live variant AC-TWc-7 | TW-c, if 12n-2 is on the base | new test beside NW-9 |
| `tools/cli/tests/fx24.rs` (new) | TW-c | new |
| `tools/cli/tests/{hosted_town.rs, walking.rs, server_command.rs}` | S11-B / 12n-1 | edited: `--cadence`, `--solo`, explicit `--time-scale 1` where scale matters (listed in §15) |
| `clients/protocol/checks/admin_check.gd`, `run.sh admin`, evidence | S11-D | edited: a rescale step |

---

## 8. Acceptance and adversarial criteria (fixed now, before anything is measured; `ARC-23`)

Step-19 §11.2's TW-c row, restated where the audit required it, each restatement with its reason.

```text
AC-TWc-1  The clock never decreases and never jumps (INV-TW-8; row criterion 1). DA-7 extended: 10 000
          seeded steps over {pause, resume, rescale to a random s ∈ 1…3600, wall advance 0–5000 ms}:
          `now` never decreases; constant while paused; equal immediately before and after every
          transition; and between transitions it advances by exactly ⌊elapsed_ms·s/1000⌋ of the scale in
          force. [runtime/world.rs]
          M-TWc-3  rescale sets the scale without folding `now` into the anchor → the jump assertion fails.

AC-TWc-2  A live rescale keeps every hosted cadence (INV-TWc-1; row criterion 4). Runtime driven with
          injected wall instants, ticks every 100 ms: a paced test seat (k = 1, cadence 5) and a reactive one
          at 12×; at wall 7.3 s rescale to 24×, at 19.1 s to 6×. Asserted on the recorded consult instants:
          (a) gaps between consecutive paced consults are 60 world s before, 120 after the first change,
          30 after the second, except the straddling gap, which equals T − p_prev + ⌈(p − T)·s₁/s₀⌉;
          (b) the wall instants at which consults happen (mapped back through the segments) are 5 s apart
          ± one tick throughout; (c) the reactive seat is consulted once per wall second ± one tick.
          [server/src/runtime/tests.rs]
          M-TWc-1  a test controller whose `rescaled` keeps the old period → (a) fails.
          M-TWc-2  the server leaves the pending instant unchanged → the straddling-gap assertion fails.

AC-TWc-3  Hosting is a function of the host journal (INV-TWc-2). RESTATES row criterion 3, which asked
          to "re-run the hosted paced controllers from the journal and the recorded facts". Reason: the
          server names no controller crate, so a replayer would have to reimplement the consult loop
          outside it — a second implementation proving only that it agrees with itself — and SD-TWc-4
          shows the one thing the journal must determine is the consult schedule. The restated test:
          a persisted scripted session (three paced test seats that speak from their observation, two
          rescales and one pause at fixed wall instants) is run twice with different seeded tick jitter
          (ticks at 100 ms ± 0–40 ms) → identical host journals; byte-identical fact envelopes in EventId
          order; identical sequences of dispatched requests (ActionId, actor, request, at). A third run
          whose pacing changes happen at the *world instants* the first run's journal recorded (not at
          its wall instants) gives the same three results. [server/src/runtime/tests.rs]
          M-TWc-7  revert SD-TWc-4 (next from the tick's reading) → the reactive seat's instants differ
          between the jittered runs; the test fails naming the first differing request.

AC-TWc-4  Pacing is recorded, not fact (INV-TW-9; row criterion 2). Through the binary
          (`mineworld server worlds/social-cafe --save D --admin-token T`, no --town, inside the
          routine-free first minutes): every clock route, rescale included, twice → /status revision
          unchanged; after a kill, `mineworld inspect D` reports `validate`'s genesis fact count; the
          save's host_journal holds Started, then exactly the effective changes in order; the input
          journal and facts tables have exactly the rows a run without the calls has.
          [tools/cli/tests/admin.rs, DA-3 extended]
          M-TWc-4  journal a rescale as an input row → revision moves; fails.

AC-TWc-5  CP-TW-c through the binary (the row's checkpoint, unchanged in substance).
          `mineworld server worlds/market-town --town --save D --admin-token T --time-scale 12`:
          pause → three /status `at` readings over 10 wall s equal, a client submit refused `paused`;
          resume and rescale to 24 → `at` advances 240 ± 24 in 10 wall s and both clients receive
          clock {time_scale: 24} within 1 wall s; rescale to 6 → 60 ± 6 in 10 wall s; graceful stop
          (Ctrl-C equivalent, portable) and restart **without** --time-scale → the startup line says
          `time scale 6 (from the save)`, and the first /status `at` ≥ the last `at` read before the stop
          and ≤ it + 6·(wall seconds since start + 1). Then: restart with --time-scale 12 → 12 (flag wins);
          a fresh save of market-town with neither → 12 (from world.yaml); social-cafe's save after
          `world.yaml` is removed of hosting (a scratch copy) → 1. [tools/cli/tests/host_clock.rs]
          M-TWc-5  ignore Stopped.at on restart → in an idle world (no --town, 30 wall s idle before the
          stop) the restart `at` is below the last read `at`; fails.

AC-TWc-6  Multiplayer never fast-forwards (INV-TWc-3; operator rule 2026-10-10).
          Without --solo: POST {"time_scale": 25} → 409 fast_forward_solo_only, /status unchanged;
          {"time_scale": 24} → 200; {"time_scale": 0}, {"time_scale": 3601}, {"time_scale": "12"},
          {"time_scale": 12.5}, {"time_scale": -1} → 400; {"time_scale": 24, "paused": true} → 200 and
          one clock frame saying both. `--time-scale 25` without --solo → non-zero exit before listening,
          naming the flag. With --solo: {"time_scale": 3600} → 200; `--solo --listen 0.0.0.0:0` → non-zero
          exit naming --solo and the address. A world.yaml with hosting.time_scale 25 → `mineworld
          validate` refuses it naming the file and key. Restart of a --solo world last scaled to 3600 →
          resumes at its last day-length scale, not 3600. [server/tests/admin.rs, host_clock.rs,
          worldpack tests]
          M-TWc-6  drop the --solo check → the 409 row answers 200; fails.
          M-TWc-9  restore the newest scale regardless of range → the restart assertion fails.

AC-TWc-7  Embodied pace across a live rescale (operator rule; only if 12n-2 is on the base).
          NW-9's world, one server at --time-scale 6 --solo, the walker mid-walk: rescale to 24 after
          ~7 strides and to 12 after ~14. Median stride speed in each segment 1.34 m/s ± 15 %, the three
          medians within 10 % of each other, and world time per stride = the segment's scale ± 15 %.
          [tools/cli/tests/walking_pace.rs]
          M-TWc-8  PacedSeat::rescaled ignores the step period → the 24× segment walks ≥ 3× faster; fails.
          If 12n-2 is not on the base: N/A here, and the obligation passes to 12n-2 (§11).

AC-TWc-8  Nothing un-hosted moved (INV-TW-1; row criterion 6). Both 300-day seed-7 `run` digests equal
          the base's; a `run` save's sqlite_master lists exactly manifest, journal, facts, snapshots,
          facts_by_revision; `run --pace 900` output byte-identical to `run` without it; every existing
          test passes, edits limited to §15's list. [tools/cli/tests/run.rs, persistence tests]
          M-TWc-10  create host_journal in SCHEMA → the sqlite_master assertion fails.

AC-TWc-9  FX-24 (step-19 §9.2, thresholds unchanged). `run worlds/market-town --days 7 --seed 1`
          at --pace 120, 60 and 30: (1) ≥ 90 % of agenda journeys end at the agenda place within 25 % of
          the part they serve and every journey within 50 %; (2) ≥ 80 % of shift-started record the
          employee present; (3) holds at all three. Result recorded with the RUN_STEP note (§5.8).
          [tools/cli/tests/fx24.rs; in the default suite if ≤ 30 s debug on the CI host, else
          #[ignore] with the evidence recorded and the reason]
          A FAIL is a material stop (§13), never retuned.

AC-TWc-10 Scope and size. No diff under kernel/, contracts/, systems/, cognition/, sdk/, authoring/,
          clients/protocol/mineworld/; no new dependency (Cargo.lock unchanged except workspace
          crates' own entries); host.rs, runtime.rs, session.rs, protocol.rs, app.rs under 500; the
          server names no pack and no controller crate.

AC-TWc-11 The far side. `clients/protocol/run.sh admin`: the check sees clock_changed(time_scale 1),
          posts {"time_scale": 12}, sees clock_changed(time_scale 12) before its next observation's `at`
          is used, and is refused `paused` after a pause, as before. Linux in CI's existing protocol job;
          macOS and Windows where the platforms job runs it.

AC-TWc-12 The answering window across a rescale (F-TWc-2). Unit, real adapter: a PacedSeat at 12×,
          observations carrying lines heard at every world second over 20 minutes, rescaled to 24× and
          later to 6× between lattice consults: every line lies in exactly one lattice consult's window
          (no line answered twice, none skipped). [tools/cli/src/hosted.rs tests]
          M-TWc-11  the straddling consult uses pace·s₁ → a line is double-counted (scale up) or missed
          (scale down); fails.
```

**Platforms.** AC-TWc-1 … -4, -6, -8, -10, -12 run in the default suite on Linux, macOS and Windows (CI's
`test`, `test-macos`, `test-windows`). The wall-time bounds in AC-TWc-5 and -7 use ± tolerances sized for
Windows' 15.6 ms timer; their stops use the portable graceful path (stdin close or Ctrl-Break on Windows,
SIGINT elsewhere, as S11-D's tests do), and `--solo`'s loopback check covers IPv4 and IPv6 on all three.

---

## 9. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-TWc-1 | 12n-2 and TW-c both edit `tools/cli/src/hosted.rs` (`PacedSeat`). | Q-TWc-3: base TW-c on main after 12n-2 merges. Otherwise 12n-2 owns `rescaled` for its step consults (§11). |
| R-TWc-2 | Making 12× the default for the launcher worlds changes timing in existing binary tests. | C6 is its own commit; every test that assumes scale 1 passes `--time-scale 1` explicitly; the list is recorded (§15). |
| R-TWc-3 | FX-24 fails at 24×. | Ruled before measuring (QTW-1): content first; a material stop, never a controller change. |
| R-TWc-4 | A FULL-synchronous journal write on the world thread delays a tick. | Pacing changes are human-rate; measured in AC-TWc-5's run and reported against CP-B4's p99 ≤ 50 ms. |
| R-TWc-5 | `ceil` in SD-TWc-3 and the tick count in SD-TWc-5 disagree by one world second at an awkward ratio. | AC-TWc-2's straddling-gap assertion is exact; AC-TWc-12 covers the window. |
| R-TWc-6 | `--solo` is read as a "separate single-player mode", against `main.rs:27`. | It selects no code path: the same server, the same authority; it only widens one permission and narrows the listen address. Stated in `ARC-69`. |

---

## 10. Commit plan

Each commit tracks **implementation**, **validation** and **review** separately; `[x]` needs the work and
its evidence; an inapplicable item is `N/A` with the audited reason. Targeted validation per commit; the PR's
CI is the one full run (test rules §8). Commands from the worktree root.

### C0 — Freeze and contract (Markdown only)

- **Goal.** Start implementation against the frozen design.
- **Scope.** This document (ledger opened), `handoff-tw-c.md` (new).
- [ ] Implementation: verify the `DESIGN FROZEN` header and §13's authority lines; record the base commit
  and whether 12n-2 is on it (AC-TWc-7 applies or is N/A); initialize the handoff.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Review: no authority line widened; Q-TWc rulings copied verbatim.
- **Commit boundary.** Documentation only.

### C1 — Specifications before code

- **Goal.** The decisions and wire changes exist before the code (`CLAUDE.md` §2.2).
- **Scope.** `docs/DECISIONS.md`: **ARC-69** (pause and scale are host commands; the live rescale and the
  rescheduling rule; the host journal, lazy table, no format bump; restart resolution; fast-forward only
  `--solo`; I-4's clarification restated); a dated note on **ARC-42** (the seam's `rescaled` and
  `HostedBinding`; SD-TWc-4); a dated note on **ARC-44** (`time_scale_fixed` retired). `server/PROTOCOL.md`
  §5.9 (frame on rescale), §10 (landing row → landed by TW-c), §11.2 (body, codes), §11.3 (pacing changes
  are journaled as host records; still never facts). `docs/MODULE_SPEC.md` §4.1 (`hosting:`), §8.1
  (`--cadence`, `--time-scale` resolution, `--solo`, `run --pace`).
- [ ] Implementation: the edits above, with citations to this design.
- [ ] Validation: both doc checks; `grep -n time_scale_fixed server/PROTOCOL.md` shows only the history note.
- [ ] Review: ARC-69 names rejected alternatives (connection-count inference; a `clock` pack; a sidecar
  file; a replayer outside the server) and a revisit trigger (a dedicated-server multiplayer mode).
- **Commit boundary.** Documentation only.

### C2 — `persistence`: the host journal

- **Goal.** A save can hold host records without changing anything a never-hosted save contains.
- **Scope.** `persistence/src/host.rs` (new), `backend.rs`, `sqlite.rs`, `world.rs`, `replay.rs`
  (coherence check), `lib.rs`; tests in `persistence/tests/`.
- [ ] Implementation: SD-TWc-13, SD-TWc-14; `verify`'s coherence check (§6.3).
- [ ] Validation: `cargo test -p mineworld-persistence`; golden bytes of each `PacingChange`; a save
  without the table reads empty; append to an old save creates the table; `verify` refuses a journal whose
  `at` decreases (naming `seq`); M-TWc-10 run and reverted; `cargo clippy -p mineworld-persistence
  --all-targets -- -D warnings`.
- [ ] Review: the backend stays meaning-free (bytes in, bytes out); no `SAVE_FORMAT` change; ARC-25's
  replay path untouched (`git diff` of `replay.rs` limited to the new check).
- **Failure cases.** A damaged record → `PersistError::Damaged` naming the row, the world refused (never
  a guessed scale).

### C3 — `server`: rescale, the seam, jitter-free consults

- **Goal.** The host clock rescales live without moving any hosted cadence.
- **Scope.** Pure move first: `HostConfig` → `server/src/host/config.rs`. Then `runtime/world.rs`
  (`rescale_at`), `hosted.rs` (`rescaled`, `HostedBinding`, SD-TWc-4), `seats.rs` (`rescale`, binding
  with the scale), `runtime/control.rs` (`ControlCommand::Clock(ClockChange)`), `runtime.rs` (`tick_at`,
  `control_at` taking the wall instant; `tick`/`control` call them with `Instant::now()`), `admin.rs`
  (body, codes, the solo rule via `Access`), `app.rs`/`lib.rs` exports; `seats/tests.rs`,
  `server/tests/seats.rs` (seam updates), `server/src/runtime/tests.rs` (new), `server/tests/admin.rs`.
- [ ] Implementation: SD-TWc-1 … SD-TWc-4, SD-TWc-6 (server half), SD-TWc-7.
- [ ] Validation: AC-TWc-1, AC-TWc-2, AC-TWc-3 (without persistence: its journal half lands in C4),
  AC-TWc-6's route rows; mutations M-TWc-1, -2, -3, -6, -7 each run red and reverted;
  `cargo test -p mineworld-server`; clippy.
- [ ] Review: no `World` touched by an admin handler; the rescale and its reschedule are one world
  command; `runtime.rs` and `host.rs` under 500; the pure move is a separate diff hunk with no edit.
- **Failure cases.** Rescale to the current scale: 200, nothing recorded, no frame. Rescale while paused:
  applies on resume; pending instants rescheduled at the frozen `T`.

### C4 — `server`: journaling and start resolution

- **Goal.** Every pacing change is recorded, and a restart resumes the host's pacing from the record.
- **Scope.** `runtime/pacing.rs` (new: resolution, `Started`/`Stopped`, writes), `runtime.rs` (`new`,
  `Shutdown`), `host/config.rs` (`TimeScaleChoice`), `runtime/tests.rs`.
- [ ] Implementation: SD-TWc-8, SD-TWc-9; records written for persisted worlds only.
- [ ] Validation: AC-TWc-3's persisted half (identical host journals); resolution table unit-tested over
  every row of §5.5; M-TWc-5 and M-TWc-9 at unit level (the binary-level runs are C7's).
- [ ] Review: the journal write never precedes the clock change it records; a failed journal write stops
  the world like a failed commit (`Failure::Stopped`), never silently continues.

### C5 — CLI and World Pack format: `--cadence`, `--time-scale`, `--solo`, `hosting:`, adapters

- **Goal.** The composition root uses the seam and exposes the operator surface.
- **Scope.** `tools/cli/src/{main.rs, serve.rs, hosted.rs}`; `worldpack/src/format.rs` (+ its tests);
  `tools/cli/tests/{hosted_town.rs, walking.rs, server_command.rs}` (flag edits only).
- [ ] Implementation: SD-TWc-5 (both adapters, the straddle window), SD-TWc-6 (CLI half), SD-TWc-10,
  SD-TWc-11; the startup line names the scale's source.
- [ ] Validation: AC-TWc-12 and M-TWc-11; the two existing adapter tests unedited and green;
  `cargo test -p mineworld-cli --test server_command --test hosted_town --test admin`; worldpack tests for
  `hosting:` (range, unknown field).
- [ ] Review: the adapters are the only place that turns wall seconds into world seconds outside the
  server; `PacedRuleController` is constructed, never edited; no alias for `--pace`.

### C6 — Worlds: the launchers' worlds default to 12×

- **Goal.** A launcher that passes no scale gets the operator's default day (2 h).
- **Scope.** `worlds/market-town/world.yaml`, `worlds/social-cafe/world.yaml` (`hosting:` only; Q-TWc-4);
  tests that assume scale 1 gain `--time-scale 1` (each listed in §15 with the reason).
- [ ] Implementation: the two keys; the test edits.
- [ ] Validation: `mineworld validate` on both; AC-TWc-8's digests (run ignores `hosting:`); the edited
  binary tests.
- [ ] Review: no other line of either world changes; INV-TW-1's byte identity confirmed for `run`.

### C7 — Real-binary acceptance and the far side

- **Goal.** CP-TW-c and the operator rule shown through the real binary and a real client.
- **Scope.** `tools/cli/tests/host_clock.rs` (new), `tools/cli/tests/admin.rs` (DA-3, DA-8 rows),
  `tools/cli/tests/walking_pace.rs` (AC-TWc-7, if 12n-2 is on the base), `clients/protocol/checks/admin_check.gd`,
  `clients/protocol/run.sh`, `clients/protocol/evidence/*` (regenerated).
- [ ] Implementation: the tests; the check's rescale step.
- [ ] Validation: AC-TWc-4, -5, -6 (binary rows), -7, -11; M-TWc-4, -5, -6, -8, -9 at binary level, each
  red then reverted; record wall times and tolerances observed.
- [ ] Review: no sleep-based assertion without a bound derived from the cadence; stops portable.

### C8 — `run --pace` and FX-24

- **Goal.** The feasibility criterion measured and kept as a regression.
- **Scope.** `tools/cli/src/{main.rs, run.rs}` (`--pace`), `tools/cli/tests/fx24.rs` (new), the journey
  extraction shared from `tools/cli/tests/routines.rs` through `tests/social/mod.rs` if it is not already.
- [ ] Implementation: SD-TWc-12.
- [ ] Validation: AC-TWc-9 at three paces; AC-TWc-8's `--pace 900` identity; record the numbers.
- [ ] Review: thresholds equal step-19 §9.2's; the RUN_STEP note recorded; no content change.
- **Failure cases.** FX-24 FAIL → stop (§13), report the numbers and the content options of QTW-1.

### C9 — Close-out

- **Goal.** Review-ready.
- **Scope.** `server/README.md` (one line: live day length and `--solo`), this ledger, the handoff.
- [ ] Implementation: README line; ledger; deviations; `overall.md`/step-19 edits proposed in §12 handed
  to the primary session.
- [ ] Validation: the PR's CI green on Linux, macOS, Windows at the exact head; sizes (AC-TWc-10).
- [ ] Review: §8 walked item by item with evidence; every mutation recorded.

---

## 11. Requirements this PR places on other PRs

- **12n-2** (if it merges after TW-c): `PacedSeat`'s step consults implement `rescaled` per SD-TWc-5 and
  AC-TWc-7 moves into 12n-2's acceptance.
- **SET-c**: sends `{"time_scale": 6 | 12 | 24}`; shows any other scale as "custom (n×)"; handles
  `409 fast_forward_solo_only` (translated) though the menu never offers > 24.
- **TW-e / S23 R-c**: the single-player launcher starts the server with `--solo` and a generated
  `--admin-token`, passes no `--time-scale`, and gets 12× from `world.yaml`.
- **S21**: "doze" = `POST /admin/clock {"time_scale": n > 24}` on a `--solo` world, restored by the client;
  no route names an instant or an arrival.

---

## 12. Proposed edits to planning documents (for the primary session)

- `step-19-time-weather.md` §11.2 TW-c row: "full design in `pr-tw-c-host-clock.md`"; the header's
  lifecycle line.
- `overall.md`: S19 row — TW-c designed (DRAFT), then frozen; the operator rule of 2026-10-10 (fast-forward
  single-player only) recorded under S19 with QRT-7.
- `step-12-server.md` §18.15 (or its closeout): QS11D-1's 409 retired by TW-c.

---

## 13. Execution contract (proposed; confirmed only at the freeze)

```text
PROJECT / PR:            MineWorld mvp0, S19 PR TW-c — the host clock
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-tw-c-host-clock.md
RELATED / BINDING DOCS:  step-19-time-weather.md (§§4, 7, 9, 10, 11.2, 14.1); step-12-server.md (§§16, 18);
                         step-11-bodies.md SD-N14; step-20-client-settings.md §3.10; docs/DECISIONS.md
                         ARC-25, ARC-27, ARC-42, ARC-44, ARC-67; server/PROTOCOL.md; docs/MODULE_SPEC.md;
                         docs/ENGINEERING_STANDARDS.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-tw-c, its own, one session
BRANCH:                  mvp0/pr-tw-c-host-clock, from main
IMPLEMENTATION BASE:     origin/main at C0 (after 12n-2 if Q-TWc-3 is ruled so)
APPROVED SCOPE:          §3.1, as frozen
FROZEN INVARIANTS:       §3.3; SD-TWc-1 … SD-TWc-14 as ruled
SEQUENCE:                C0 … C9
ALLOWED COMMANDS:        cargo *; git; gh (never merge); python3 scripts/*; target/*/mineworld *;
                         clients/protocol/run.sh; godot --headless (the checks); mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; editing kernel/, contracts/, systems/,
                         cognition/, clients/protocol/mineworld/
MATERIAL STOPS:          FX-24 FAIL; any edit outside §3.1's paths; a new dependency; a change to a
                         threshold in §8 after it has run once; a SAVE_FORMAT change; a pack constant
PLATFORMS:               Linux, macOS, Windows
VALIDATION BUDGET:       unit, integration, binary tests: unrestricted locally; CI: the PR's runs
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-tw-c.md
ENDPOINT AUTHORITY:      to be filled at the freeze by the primary session
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Questions (Q-TWc-n). **[OPERATOR]** marks operator-material ones.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-TWc-1 [OPERATOR]** | How does the server know a world is single-player, so that only it may fast-forward? (a) an explicit `--solo` flag that requires a loopback listen address; (b) infer it from one connected session (fails: an LM-driven controller connects as a session); (c) TW-c only caps every world at 24× and leaves the single-player exception to S21. | **(a)**: a few lines, true by construction (nobody off the machine can join), and it lets S21 and the launcher use it without another server PR. (c) is acceptable if the operator prefers to ship fast-forward with S21. |
| **Q-TWc-2 [OPERATOR]** | Does the rule bind the start-time `--time-scale` too? A LAN server started `--time-scale 3600` is a multiplayer world fast-forwarding. | **Yes**: > 24 needs `--solo` at start as well; the loopback tests that compress days (`walking.rs` 28 800×, `server_command.rs` 60×) add `--solo`. |
| Q-TWc-3 | Base TW-c after 12n-2 merges? 12n-2 is held at M-2 (ruled) on `mvp0/pr-12n2-walk @ 1bf18cc`; both edit `PacedSeat`, and FX-24 should be measured on the walking town. | **Yes.** Otherwise 12n-2 takes `rescaled` for step consults and AC-TWc-7 (§11). |
| Q-TWc-4 | Which worlds get `hosting.time_scale: 12`? | `market-town` (the 2D launcher's) and `social-cafe` (the slice's); `bodies-yard` stays 1 (a physics yard watched in real time). |
| Q-TWc-5 | Accept AC-TWc-3's restatement of row criterion 3 (jitter-independence and journal-instant-driven determinism, instead of a replayer of hosted controllers)? | **Yes**, for §8's reason: a replayer outside the server would test a second implementation against itself. |
| Q-TWc-6 | Restore the host clock on restart from the `Stopped` record (`max(world.now, Stopped.at)`), while keeping S11-D's "a restarted world runs"? | **Yes**: it makes CP-TW-c's "resumes from the saved instant" exact and keeps the HUD from running backwards (TW-e). |
| Q-TWc-7 | Rename `--pace` to `--cadence` with no alias (QTW-13's ruling; `CLAUDE.md` §4 rule 12)? | **Yes.** Two in-repo users (`hosted_town.rs`, `MODULE_SPEC.md`); no launcher passes it. |
| Q-TWc-8 | Accept L-TWc-1: after a restart of a world rescaled mid-life, one answering window may overlap or miss the last one before the stop (a fresh binding, as at a release)? | **Yes**, recorded in ARC-69; exact continuity would need the controller's last lattice instant in the save, which is controller state in world persistence (ARC-27 (c) rejected that). |
| Q-TWc-9 | Decision ids: TW-c uses ARC-69 only and adds notes to ARC-42 and ARC-44; no DEP. Confirm none other is needed. | Confirm. |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | Hash | Status |
| --- | --- | --- |
| C0 … C9 | — | not started (design not frozen) |

### Evidence

None yet.

### Mutations

| Id | Mutation | Expected | Observed |
| --- | --- | --- | --- |
| M-TWc-1 … M-TWc-11 | §8 | red | — |

### Existing tests edited (filled by C3, C5, C6, C7)

None yet.

### Deviations and findings

- F-TWc-1, F-TWc-2: recorded at planning (§2), resolved by SD-TWc-4 and SD-TWc-5.

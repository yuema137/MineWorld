## ARC-69 — Pause and scale are host commands; a live rescale reschedules hosted consults; pacing is a host journal, never a fact

**Date** 2026-10-14 · **Status** accepted (design, S19 PR TW-c, frozen 2026-10-10 as revision 2; implemented by TW-c) · **Approved by** the operator's rulings of 2026-10-08 (step-19 §1.2: live change, host or admin only in multiplayer), QTW-2, QTW-13, the relayed rule of 2026-10-10 (only a single-player world may fast-forward), and Q-TWc-1 … Q-TWc-9 (2026-10-10; §14.1 of the design) · **Implements** `ARC-67` item 3 · **Relates to** `ARC-25`, `ARC-27`, `ARC-40`, `ARC-42` (note of this date), `ARC-44` (note of this date), `ARC-67`, `ARC-75`, `INV-TW-1`, `INV-TW-2`, `INV-TW-3`, `INV-TW-8`, `INV-TW-9`, `INV-TWc-1`, `INV-TWc-2`, `INV-TWc-3` · **Design** `.structured-coding/plans/mvp0/pr-tw-c-host-clock.md` §§5–6

**Problem.** `ARC-67` made the time scale and pause host pacing, not world state, and stated that their
changes are host records in a host journal. Three things were still open: a scale that changes while the
world runs must not make any hosted Person's wall-clock cadence (consults, strides, lines) jump; a restart
must resume the host's pacing rather than the last revision's instant; and a multiplayer world must never
fast-forward. None of this may reach a rule, a fact or the revision.

**Choice.**

1. **The clock's third transition is a rescale** (`HostClock::rescale_at`). A running clock folds its reading
   into its anchor and changes scale; a paused clock changes scale, applied from the resume. `now` does not
   jump and never decreases (`INV-TW-8`).
2. **One rescheduling rule, in the server.** On a rescale at world instant `T` from `s₀` to `s₁`, a pending
   consult at `p > T` moves to `T + ⌈(p − T)·s₁/s₀⌉` (the same wall wait in the new world seconds); a consult
   already due stays due. The server moves the pending instant and tells the controller, in one world
   command, through `HostedController::rescaled(pending, scale)` (required, no default). A factory is given
   the scale in force, so a seat bound after a rescale is built at it (`HostedBinding`).
3. **Consult instants are jitter-free.** A reactive consult's next instant is computed from the instant the
   consult was due, not from the tick's reading (`next := next_consult(next)` while `next ≤ now`). In-server
   controllers' instants are therefore a function of their bindings, seeds and the host journal, not of tick
   timing (`INV-TWc-2`).
4. **The CLI adapters re-anchor.** Each adapter keeps its cadence in wall seconds, mapped to world instants
   through its current segment. A paced seat whose answering window straddles a rescale answers its lines
   with a window exactly as long as the real gap, then with `pace × s₁` (`PacedRuleController` is constructed,
   never edited). Restart and release remain fresh bindings, as `ARC-42` already states; after a restart of a
   world rescaled mid-life, one answering window may overlap or miss the last one before the stop.
   This is accepted limitation **L-TWc-1**: exact continuity would need the controller's last lattice instant
   in the save, which is controller state in world persistence and was rejected by `ARC-27` (c).
5. **Fast-forward is single-player only.** The day-length range is 1 … 24 for every world. Above 24, a scale
   (start-time `--time-scale`, `POST /admin/clock`, `hosting:`) is permitted only in a world started with
   `--solo`, and the live route accepts at most 3600. `--solo` is an explicit flag, not an inference from the
   connection count: an LM-driven controller connects as a session, so one connection is not one player. It
   requires a loopback listen address, which makes the declaration true by construction. `--solo` selects no
   code path: the same server and the same authority; it only widens one permission and narrows the listen
   address. `main.rs`'s statement that there is no separate single-player mode stands.
6. **The host journal.** A new persistence record, `HostRecord { at, revision, change: PacingChange }` with
   `PacingChange::{Started { scale, paused }, Paused, Resumed, Scaled { to }, Stopped}`, in an additive table
   `host_journal`. The table is created lazily by the first append, never by the save's `SCHEMA`: a save that is
   never hosted keeps exactly the four tables it has, and reading a save without the table yields an empty
   journal. **No `SAVE_FORMAT` bump**: no existing row's encoding changes. A pre-TW-c binary resuming a TW-c save
   ignores the table; a TW-c binary resuming an older save sees an empty journal.
7. **The record is not an input.** `verify` re-executes the input journal exactly as `ARC-25` states and never
   re-executes or compares the host journal. It separately checks the host journal's coherence: records decode,
   `seq` order has non-decreasing `revision`, `at` never decreases across the save's lifetime, and
   `revision ≤ head`. A pacing change is never a fact, a journal row, a snapshot or a revision (`INV-TW-9`).
8. **Restart resolution.** The scale is `--time-scale` if given, else the journal's last day-length scale,
   else `world.yaml`'s `hosting.time_scale`, else 1. A fast-forward scale never survives a restart. A persisted
   world's host clock starts at `max(world.now(), last Stopped.at)`; `Stopped` is written by the graceful
   shutdown, after the checkpoint. A restarted world runs: a pause is not persisted (`ARC-44`, unchanged).
9. **`world.yaml hosting: { time_scale }`** is host pacing content, read by `mineworld server` and never by a
   pack, a controller or a fact. It is limited to 1 … 24 and refused at load otherwise. `market-town` and
   `social-cafe` declare 12; `bodies-yard` keeps 1 (Q-TWc-4). It is ignored by `run` (`INV-TW-1`).
10. **Wire.** `POST /admin/clock` takes `time_scale`, `paused`, or both; `time_scale_fixed` is retired
    (`ARC-41`: may omit, never redefine); `fast_forward_solo_only` is added. Every effective change sends a
    `clock` frame.
11. **`--pace` is renamed `--cadence`, with no alias** (Q-TWc-7). `run --pace` is a measurement knob for FX-24,
    default 900.

**Invariants.** `INV-TW-1` (a never-hosted world is byte-identical), `INV-TW-2` (no pack, rule or fact reads the
scale or pause; the adapters may know the scale, the cognition crate is not edited), `INV-TW-3` (kernel,
contracts and systems untouched), `INV-TW-8` (the host clock never decreases or jumps), `INV-TW-9` (pacing
changes are host records), `INV-TWc-1` (a hosted cadence is unchanged by a live rescale, to within one tick),
`INV-TWc-2` (in-server controllers' behaviour is a function of the world, the seeds and the host journal),
`INV-TWc-3` (a world not started `--solo` never runs faster than 24×).

**Alternatives rejected.**
- *Inferring a single-player world from one connected session.* An LM-driven controller connects as a session,
  so one connection is not one player (`ARCHITECTURE.md` §9).
- *A `clock` pack that owns the scale.* The scale is host pacing; a pack would read host state, the coupling
  `ARC-67` rejects.
- *A sidecar file for the journal.* A second persistence authority; the save is the one place a world's history
  lives (`ARC-25`).
- *A replayer of hosted controllers outside the server.* It would be a second implementation of the consult loop,
  proving only that it agrees with itself. The restated test (AC-TWc-3) checks the property that matters: the
  journal determines the schedule.
- *Rescheduling in each adapter.* The rule must hold for every hosted controller, so the server owns it; the
  adapters only re-anchor (SD-TWc-3, SD-TWc-5).

**Revisit trigger.** A dedicated-server multiplayer mode (a server that never runs for one player) would make
`--solo` a deployment property rather than a declaration, and would need this record reviewed.

**Consequences.** A launcher starts a single-player world with `--solo` and a generated admin token; S21's
"doze" is `POST /admin/clock {"time_scale": n}` with n > 24 on a `--solo` world, restored by the client
(no route names an instant or an arrival). `ARC-67`'s FX-24 remains the feasibility test; its remedy is content,
never a controller that reads the scale.

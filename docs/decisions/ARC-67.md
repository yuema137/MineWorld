## ARC-67 — Two time domains: `WorldTime` is calendar time; embodied time is not a world quantity

**Date** 2026-10-08 · **Approved by** the operator (requirements of 2026-10-08, rulings QTW-1, QTW-2,
QTW-5, QTW-9, QTW-13) and the primary session at PR TW-a's design freeze · **Implements**
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §17 · **Relates to** `ARC-26`, `ARC-27`, `ARC-28`, `ARC-61`,
`ARC-68` (calendar and weather packs, TW-b), `ARC-69` (pause and scale, TW-c), `DEP-6`, `DEP-30` ·
**Design** `.structured-coding/plans/mvp0/step-19-time-weather.md` §4, §15.1 (S19)

**Problem.** The operator asked for a town whose day passes faster than a real day — a real 24 hours
mapped onto 4, 2 or 1 real hours (6×, 12×, 24×; 12× by default), changeable live, and pausable — and
fixed what the speed may and may not touch: it changes the rate of weather and of world events (the
calendar, day and night, the sun, schedules, shifts, wages), and it must **not** change how fast a
character walks, animates, falls or talks. The kernel already has one clock, `WorldTime`, and states
that "how fast a hosted world's seconds pass in real time is the host's decision" (`kernel/src/clock.rs`).
The question is how the two kinds of time relate without the kernel learning what a day is.

**Choice.**

1. **Calendar time is `WorldTime`**, unchanged: the world's one clock, whole simulated seconds from the
   epoch. Every fact, Process wake-up and `Observation.at` is stamped in it, and the calendar, the sun,
   weather, routines, shifts, wages and every calendar-driven Process are functions of it.
2. **Embodied time is not a world quantity.** It is real (wall-clock) time as a body experiences it:
   how fast a player walks, how long a line takes to type, how long an animation or a physics impulse
   plays. No fact, component, Process or rule carries it. It exists only as the cadence at which
   embodied inputs arrive at the server: a human's client sends requests at human speed, the host
   consults a hosted controller every *cadence* wall seconds, and a client animates at frame rate. A
   per-request rule (a stride, an impulse, a spoken line) is resolved at the instant its request
   arrives, so the scale cannot reach it.
3. **Time scale and paused are host pacing.** The time scale is world seconds per wall second while a
   hosted world runs (an integer ≥ 1); paused is the host state in which world seconds do not pass. The
   host turns wall time into the instant it advances the world to; the event queue fires what is due on
   the way. Neither is world state: no System Pack, controller decision or fact reads either
   (`INV-TW-2`). Their changes are host records in a host journal, never facts (`ARC-69`, TW-c).
4. **Consequence: hosted consult cadence is in wall seconds** (QTW-13, ruled by the operator; it reverses
   QS11B-4 for consult cadence). Headless `run` has no scale and no wall clock: it is unchanged, and its
   output for a world without the time packs is byte-identical.

**Alternatives rejected** (step-19 §4.2).
- *Two kernel clocks with a ratio:* the kernel would learn a day, every Process would have to say which
  clock it uses, and persistence would reconcile two timelines — kernel ignorance and change
  amplification both fail.
- *Each pack multiplies its durations by the scale:* packs would read host state; a rule that depends on
  the host is a determinism and ownership defect, and headless and hosted runs would diverge.
- *Embodied actions take world durations divided by the scale:* movement is per request and not
  modelled in world time (`ARC-26`); inventing durations only to cancel the scale is the coupling the
  operator asked to avoid.

**Accepted limitations.** World-time constants that a human answers in practice — conversation's
300 s gap, group-activity's 1 800 s invitation lifetime — become short in wall time at 12–24×. They stay
calendar time by this decision's rule; making them configurable content is QTW-15 (IL-b). FX-24
(step-19 §9.2) measures whether agendas remain feasible at 24×; its remedies are content, never a
controller that reads the scale.

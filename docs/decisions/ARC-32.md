## ARC-32 — A schedule is an agenda that controllers follow, never a mover

**Date** 2026-10-07 · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §10 · **Relates to**
`INV-1`, `INV-3`, `INV-6`, `INV-12`, `INV-13`, `ARC-26`, `ARC-27`, `ARC-31` · **Design**
`.structured-coding/plans/mvp0/step-09-social.md` SD-13, §10.1 Q8, §4.3 (S8, PR 10c)

**Problem.** People in a living town keep a day: the café in the morning, work, the park, home. Someone
must own that day, and something must make it happen. If the owner also moved people, it would
override whoever controls them — a human included — and bypass movement's rules: a person teleported
to work never crossed a street.

**Options considered.** (a) The schedule states presence's `arrived` at each boundary: a mover,
rejected for the reasons above. (b) **The schedule owns an agenda, and controllers walk to it.**

**Choice: (b).**

1. `ScheduleSystem` owns, on each person:
   - `Routine`: the authored segments, each `{ from: time of day, place, label }`;
   - `Agenda`: the segment in force, with its start and end and the routine's process.
2. Each person's day is **one `Process` of kind `routine`**, started while schedule reduces the
   genesis `routine-assigned`. Its expected end is the next boundary. At each wake, schedule emits
   `agenda-changed` (`Causation::Process`) and reschedules the process. The process never ends.
3. **It moves no one.** It provides no action, depends on no system, and states no other pack's
   vocabulary, so the registry would refuse it if it tried. Following an agenda is a controller's
   choice:
   - `mineworld run`'s paced controller walks there through `move`, after being addressed and
     before taking any initiative;
   - a human may ignore it;
   - a person nobody drives keeps their place while their agenda changes.
4. **Time of day is world seconds since the epoch, modulo 86 400.** That is schedule's own
   convention, and the kernel does not know it (`INV-12`).
5. The agenda is disclosed to its holder only (`INV-13`). `agenda-changed` is biographical (`ARC-29`),
   and validation guarantees that every one is a real change.

**Accepted limitations.**
- A day is the same every day: no weekdays, no exceptions.
- Routes are found only in a star town.
- Hosted worlds have no paced controller, so their agendas change and nobody follows them unless a
  person chooses to.
- **Fixture dependency (step-09 §4.3.7).** `social-cafe`'s routines keep 00:00–05:00 free of
  boundaries. A world hosted from genesis or from a day-end save therefore commits nothing for five
  hours, and the restart tests' "revision unchanged" claims rely on that. Each of those tests checks
  the assumption first, and fails naming it.

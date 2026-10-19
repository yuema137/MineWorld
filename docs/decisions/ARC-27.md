## ARC-27 — A headless run is a pace schedule over stateless seeded controllers

**Date** 2026-10-06 · **Implements** [`MVP.md`](MVP.md) §9 `AC-6`, `AC-11`, `AC-12` ·
[`MODULE_SPEC.md`](MODULE_SPEC.md) §8.1 `run` · **Relates to** `INV-1`, `INV-13`, `ARC-23`, `ARC-25`,
`DEP-6` · **Design** `.structured-coding/plans/mvp0/step-08-headless.md` (S7, PR 09)

**Problem.** `AC-11` asks that *a seeded rule-based configuration* run hundreds of simulated days with
no renderer and no model, and `AC-12` that the same seed reproduce the run. Three facts found in
source made the obvious reading empty. The real System Packs (presence, movement, conversation) are
time-inert: none starts a process or defers a fact. The only controller, `RuleController`, answers
whoever spoke to its Person and otherwise does nothing. And with no client connected nothing else
submits a request. A headless café would therefore advance an idle clock for three hundred days and
record its five genesis facts — a history perfectly stable and perfectly reproducible, which is the
clean number from an instrument that cannot see that `ARC-23` forbids trusting.

A second problem: the server's agent path cannot be reused. A hosted world's time follows the wall
clock, observations are delivered with `try_send` and dropped under load, and the agent is a task.
Which observation a controller decides on is a matter of timing. And the reactive controller keeps
*which line it has answered* in memory, so a restarted one answers its last line again (step-06
`F-13`) — a killed-and-resumed run would then differ from an uninterrupted one.

**Options considered.**

```text
(a) drive the world through the server, as --agent does     wall-paced and timing-dependent
(b) a scripted input file                                    reproducible, but a replay of a script,
                                                             not a rule-based configuration
(c) a stateful seeded controller whose memory is persisted   controller bookkeeping becomes world
                                                             state, against INV-1's separation
(d) a stateless seeded controller on a fixed pace schedule   chosen
```

**Choice: (d).**

1. **The driver.** `mineworld run` steps the world itself, synchronously, on one thread, calling only
   what the server calls: `advance_to`, the pack's perception, `ActionIntent::allocate`, `dispatch`.
   A run's `ActionId`s are allocated in order from 1, and a resumed run's from one past the
   journal's highest.
2. **The pace.** Seat *k*, in the pack's seat order, is consulted at `genesis + k + m·P` simulated
   seconds, `m = 0, 1, …` (P = 600 s; refused if the pack has P seats or more). No two seats are ever
   consulted at one instant, so the order of decisions inside an instant never arises.
3. **The controller is stateless.** `PacedRuleController::decide(&self, &Observation)`: its decision
   is a pure function of its seed, its pace and the observation, which the compiler enforces. Every
   choice is a SplitMix64 mix of `(seed, observer, instant)`. It answers the newest line a speaker
   said to it *since its previous consult* — a line heard at `h` is answered at `t` only if
   `t − P < h ≤ t` — and otherwise takes seeded initiative: greet, approach, wander, use a doorway.
   Because only consults dispatch and no two share an instant, each line lies in exactly one of its
   listener's windows: *answer once* holds with no record of having answered, and a controller built
   after a restart decides exactly as the one that died.
4. **Age, not duration.** `--days N` runs the world until it is N days old, so the same command run
   again after a crash completes the same world.

**What `AC-12` covers, and what it excludes.** Covered: the facts, the journal and the snapshots of a
run, byte for byte, as a function of the pack, the seed, the age, the pace and the code. Excluded by
name: the world instance identity in a save's manifest (allocated from the wall clock and the
process, because two runs are two worlds — `server/PROTOCOL.md` §5), and the elapsed wall time `run`
prints on its own line. Equality is always shown by comparing bytes; the printed fingerprint is for
reading, never evidence.

**What it does not cover.** A hosted world's `--agent` keeps the reactive controller and keeps `F-13`:
it has no fixed consult schedule, receives many observations per simulated second, and needs its
memory. Its remedy is a perception or cognition change (S10).

**Accepted limitations.** A line whose window passes while `talk` is unavailable is never answered —
the listener missed the moment. The controller's speech is formulaic, as a rule's is; interpretation
is cognition's. Places have no extent, so wandering is bounded only by the pull of nearby people.

**Note, 2026-10-07 (S8, `step-09-social.md` C3).** Three changes, none to the decision itself:

1. **P is 900 s**, not 600. S8 made `social-cafe` a town of twelve people with eleven seats, and one
   300-day debug run took 77 s at 600 s. The step-08 rule is that the pace rises before the days fall,
   so it rose: 51–55 s. A pack with P seats or more is still refused.
2. **A door is a seeded choice kept for six simulated hours**, a draw over `(seed, observer, instant ÷
   21 600)`. That is still a pure function of the observation. Before, *leave* took the first disclosed
   passage, and on a street of five doors everybody walked into the lowest-numbered place and never
   came back. In a place with more than one door, which in practice is a street, walking on is the
   likelier draw.
3. **A proposed stride never exceeds `MAX_STRIDE`.** The stride arithmetic divided by the *floored*
   square root, so a stride came out a fraction of a millimetre over 2 000 mm and was refused
   `TooFarAway`. It now divides by the root rounded up. S7's `TooFarAway` refusals, read at the time as
   crossings at a rounded 2 000 mm, were this defect, and the S7 test could not see it because it
   measured strides with the same floored root (`ARC-23`). With the fix, a 300-day run has no refusals
   at all.

**Note, 2026-10-09 (S15, PR 12n-2; `step-11-bodies.md` §21 SD-N15, SD-N16; `ARC-75`).** People walk
by asking movement for a walk (`walk-to`) and then for its strides (`walk-step`), one per request. The
run paces those strides; nothing about the decision itself changes:

1. **Step consults.** As well as the lattice `genesis + k + m·P`, seat `k` is consulted at `genesis + k
   + n·RUN_STEP` (`RUN_STEP` = 30 world seconds, a divisor of P) whenever its person has a `Walking`
   at that instant. The run reads that from world state through movement's `is_walking`, so a resumed
   run schedules exactly the step consults the dead one would have, which is this decision's restart
   property. A seat that is not walking is not consulted, and the clock is not advanced for it: a
   `Walking` is only ever created by a request, so a seat that was not walking before the instant is
   not walking at it. `RUN_STEP` is notional (step-19 §4.6: `run` has no time scale). A 20 m walk is
   15 strides, 7½ notional minutes, rather than ten lattice consults.
2. **At a step consult the controller is asked `step`**, a separate pure function that returns only
   `walk-step`, takes no draw and never answers. `decide` is still asked only on the lattice, once per
   pace, so "answer each line once" holds. A lattice instant is always a step instant. The one consult
   there is `decide`, and when `decide` returns nothing for a walker, `step`, so that a walker who
   rolls "stand" keeps walking.
3. **A pack with `RUN_STEP` (30) seats or more is refused**, naming `RUN_STEP` (it was P). Residues
   mod 30 keep every instant to one seat. The largest pack has eleven seats.

**Note, 2026-10-13 (S24, PR DL-a; `pr-dl-a-decision-list.md` SD-DLa-1, SD-DLa-2, I-DLa-3; `ARC-85`).**
"Stateless seeded controllers" is generalised from the paced rule controller to **every in-process
decision model**. The headless run now consults each seat through the model its decision list resolves
(`ARC-84`); a model is a `DecisionModel` (`cognition/decision`) whose `decide` and `step` take `&self`,
its typed parameters and a `Consult { seed, pace, observation }`, so what it attempts is a pure function
of `(parameters, seed, pace, observation)` and a model rebuilt after a restart decides exactly as the one
that died. The pace travels in `Consult` because it is the answer window, host context a model cannot
know otherwise. Model crates forbid interior mutability by their own `clippy.toml`, and a template test
runs every installed in-process model twice and across a kill-and-resume against the save's bytes.
`Draw` and `mix` moved verbatim from `paced.rs` to `mineworld-decision`, salt included; the rule
controller, `rule`, is the first model and the default, and with no decision list every seat is `rule`
and every byte as before. The lattice, `RUN_STEP` and everything above are unchanged.

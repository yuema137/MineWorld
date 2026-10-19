## ARC-42 — In-server controllers run on the world thread behind a bounded synchronous seam

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-B · **Approved by** the primary
session at S11-B's design freeze (step-12 §16; QS11-2) · **Relates to** `ARC-27`, `ARC-40`,
[`ARCHITECTURE.md`](ARCHITECTURE.md) §9 · **Design** `.structured-coding/plans/mvp0/step-12-server.md`
§§4.5, 7.5, 16

**Problem.** A hosted world had no walkers: `mineworld server` ran only `--agent` seats, as tokio tasks
that joined a seat like a client and decided on whichever observation the 10 Hz `try_send` stream
happened to deliver. The paced controller that makes a town live (`ARC-27`) must be consulted at
instants it can name — once per pace, on a lattice — which only the thread that owns the clock can
guarantee.

**Choice.** `server/src/hosted.rs` declares `HostedController`: `next_consult(after)` names the next world
instant it wants, `decide(&observation)` returns at most one `ActionRequest` from its own Person's
observation, and `answered(&answer)` tells it what became of that request. It is **synchronous and
bounded**: `decide` must return in microseconds and never block, wait or call out. On every tick the world
thread consults each hosted seat whose instant is due — in instant order, then seat order — computing
that seat's observation through the same perception call a session gets and passing any request through
the same `submit_at` a session's request takes (actor check, server-allocated `ActionId`, journal before
answer). Consults due while a person drives the seat are skipped, never queued. The server names no
controller crate: `tools/cli` adapts the reactive and the paced rule controllers onto the trait and
registers a factory per seat (`HostedWorld::hosting`), which the server calls at start and at every
release. **An asynchronous controller — a language model — does not use this seam: it connects as a
session**, over the protocol, so the world never waits for it (`ARCHITECTURE.md` §9).

**Options considered** (step-12 §7.5).

```text
(a) own consult lattice on the world thread, ARC-27's formula   chosen: exact world instants, no
                                                                 dropped frames, no new concept
(b) one tokio task per controller (the old agent.rs)             decides on whichever frame arrived;
                                                                 cannot name its instants. Kept only as
                                                                 the shape of a future asynchronous
                                                                 controller, which is a session
(c) the kernel's scheduler, a controller as a Process            a controller would become world state
                                                                 (INV-1; ARC-27 option (c))
(d) job schedulers (tokio-cron-scheduler, apalis)                wall-clock cron and queues: the wrong
                                                                 clock and the wrong concept
```

**Why not theirs.** Nothing external schedules on a world's own clock against a seat binding the world
thread owns; the lattice is the formula `mineworld run` already uses, a few lines here.

**Measured, not assumed.** `decide` runs on the world thread, so its cost is the world's. The server
counts its ticks in a fixed histogram and keeps the longest, and prints them on a graceful stop
(`[world] ticks N, p50 A ms, p99 B ms, longest tick M ms`); the composition root prints each hosted
seat's consults and outcomes beside it. S11-B's CP-B4 bounds the **p99** tick of a hosted
`market-town --town --save` at 50 ms, half the observation cadence, and reports the maximum beside it
(operator ruling on D-SB12, 2026-10-08: one off-CPU stall the server cannot control — descheduling,
or the commit's I/O wait — must not decide the bound; step-12 E-SB9 located every outlier there).

**Limitations accepted.** A hosted world is not reproducible from a seed — its instants follow the wall
clock (`ARC-27` excludes it from `AC-12`); its save still replays byte for byte, because the journal holds
the requests. The paced controller's rates are tuned to a 900-second pace; at the hosted default of 5 s
it is chattier than a person (QS11-4), which is cognition's to tune (S10), not the server's.

**Note, 2026-10-09 (S15, PR 12n-2; `step-11-bodies.md` §21 SD-N14; `ARC-75` item 7).** A hosted paced
seat steps its walks at wall cadence. `PacedSeat` keeps its lattice `genesis + (k + m·pace)·scale`
for `decide`. After a consult whose observation disclosed the seat's own `walking` record, or whose
request was `walk-to` or `walk-step`, it adds **step consults** every `EMBODIED_STEP × scale` world
seconds, offset `k·scale` (QTW-13's form), each answered by the controller's pure `step`. It stops at
the first step consult that returns nothing. `EMBODIED_STEP` is one wall second, a constant of this
adapter: one `walk-step` of 1 340 mm per wall second is 1.34 m/s on screen at any time scale
(Weidmann 1993). No pack and no controller sees the scale. Idle seats get no step consults. A
consult is the lattice consult when a lattice instant lies between the seat's previous consult and
this one, because the runtime builds the observation at `due.max(world.now())`. There it is `decide`,
and `step` when `decide` returns nothing for a walker. The server's seam is unchanged. The reactive
seat does not walk.

**Note, 2026-10-14 (S19, PR TW-c; `ARC-69`; design `pr-tw-c-host-clock.md` §5.2, SD-TWc-2 … SD-TWc-5).** The
seam learns a live rescale. `HostedController::rescaled(pending, scale)` is required, with no default; the
factory is `Fn(HostedBinding { at, scale })`, so a seat bound after a rescale is built at the scale in force.
On a rescale the server moves each pending consult by the same wall wait in the new world seconds, in the same
world command that re-segments the clock. `HostedSlot::decide` computes the next instant from the instant the
consult was due, not from the tick's reading, so in-server consult instants do not follow tick jitter. The
adapters re-anchor their wall-second cadence to the new scale, and a paced seat's first lattice consult after a
rescale answers the lines heard since its last one, once. Restart and release remain fresh bindings (`L-TWc-1`,
`ARC-69` item 4).
**Note, 2026-10-15 (F-tick-1; operator ruling amending D-SB12, `step-12-server.md` §16.11).** The
wall-clock p99 bound failed on shared CI runners while the server's own work was unchanged: F-12n-CI1's
57.3 and 62.8 ms, PR #206's 70.7 ms and PR #198's 189.6 ms, every one with a wall p50 of 0.3–0.6 ms and
each passing on re-run. E-SB9 had already located every slow tick off-CPU (wall ≫ CPU). The p99 of
1 200 wall ticks is the twelfth-longest, so twelve preemptions decide it. The measured statement is
therefore split by what each clock can see:

- The server also records the world thread's **own CPU time** per tick, in a second fixed histogram
  (`DEP-59`), and the statistics line becomes `[world] ticks N, p50 A ms, p99 B ms, longest tick M ms,
  cpu p50 C ms, cpu p99 D ms`. The first three duration columns are wall time, as before.
- CP-B4 (and CA-13, which reuses its bound) asserts **CPU p99 ≤ 50 ms**, the tail of the work a tick
  does, which runner load cannot move, and **wall p50 ≤ 50 ms**, the typical tick as a client feels it,
  which catches a tick that waits without computing (a sleep, a blocking call). The wall p99 and the
  maximum are printed beside them and do not gate.
- **Accepted gap.** An off-CPU wait in fewer than half the ticks (the journal's fsync per hosted request,
  `Durability::PowerLoss`, on a slow disk) is visible only in the advisory columns. E-SB9 measured a
  commit at ≤ 5.1 ms even under eight fsync-heavy neighbouring servers, so the bound gives up nothing
  measured; a durability regression is R-S11-5's to watch, not this bound's.
- If the thread's CPU clock cannot be read, the tick's CPU time is recorded as its wall time, an upper
  bound: a missing clock can only make the bound harder to meet.

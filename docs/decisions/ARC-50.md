## ARC-50 — A predicting 3D client corrects its body to the server's position beyond 150 mm, once, and waits

**Date** 2026-10-16 · **Status** decided; in force from S15 PR 12e · **Approved by** the primary session
at 12e's design freeze (step-11 §22.0, 2026-10-09; the identifier was left for the primary session to
assign from S14's range `ARC-50` … `ARC-52`, `overall.md` ruling 6, and is taken here as that range's
first, recorded in step-11 §22.15) · **Relates to** `DEP-20`, `ARC-26`, `ARC-67`,
[`NETWORKING.md`](NETWORKING.md) §4, [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§5–9,
[`server/PROTOCOL.md`](../server/PROTOCOL.md) §6.2, [`clients/protocol/ADOPTION.md`](../clients/protocol/ADOPTION.md)
§4.1 · **Design** `.structured-coding/plans/mvp0/step-11-bodies.md` §22 SD-E5, SD-E8, SD-E11
(drafted in `step-15-demo-3d.md` §4.3 as `ARC-S14-a`)

**Problem.** The 3D client walks its own body ahead of the server and reports where it went
(`PROTOCOL.md` §6.2); the server decides every stride. Before S15 PR 12e the client took the server's
position only when a `move` was refused, so a stride the server accepted but stopped short, a nudge by
somebody else, a shove, or a jam at a box left the drawn body somewhere the world did not hold it, and
nothing ever closed the gap. Once the client also collides locally with people and objects, the drawn
body and the server's body part routinely and must be brought back together without jitter.

**Options considered** (step-15 §8.2: no library fits a non-Godot semantic server, so the pattern is
adopted and built).

```text
(a) refusal-only reconciliation (before 12e)          drifts: every accepted-but-different answer is lost
(b) snap to every observation                         the body the player drives shakes at every report
(c) a snapshot buffer / input replay for the player   latency on the one body the player drives; the
                                                      server's strides are not replayable client-side
(d) one tolerance, a correction, a suspension          chosen
```

**Choice.**

1. **The reference.** The point the client reported with the request the observation reflects. In the
   **exact form** the observation's `acted_through` (S11-C) names that request, and the client compares
   the server's position with the position that request reported — or, for a request that is not a
   `move` (a talk, a kick, a throw, a shove reports the body's position but asks for none), with the
   point of the stride answered before it. The **heuristic form**, for an observation whose
   `acted_through` names no request the client recorded: the point of the newest answered `move`, and
   every point still in flight.
2. **One number.** When the server's position differs from the reference by more than **150 mm** —
   above the server's 1 mm quantization and its controller's 10 mm gap, below a visible jump, and the
   same tolerance as 12e's geometry probe and 16c's `DOOR_MATCH` — and (heuristically) from every point
   in flight, the reference becomes the server's position. If the drawn body also lies more than 150 mm
   from it, the body is **corrected** to it: velocity zeroed, moved there.
3. **Suspension.** After a correction the client reports no `move` until every request sent before the
   correction has been answered and one further observation has arrived; it then applies the rule once
   more and resumes. A stride sent from the wrong position can still be accepted after the correction;
   without the suspension it would drag the body back, correction after correction.
4. **Drawing.** A correction up to 1 m glides over 120 ms of wall time — the order of the 100 ms entity
   interpolation of Valve's Source networking — moved kinematically toward a point the server already
   holds free, with input live; a larger one snaps. Every duration is wall time: the world's time scale
   is never read (`ARC-67`).
5. **Nothing else moves the body to a server position** (I-S14-4). The first view of a world places the
   body where the observation says; after that only this rule does.
6. **Press-through.** A body stopped locally at a person never asks the server to nudge them. When the
   slide meets a person or an object and the input points into it, the client reports the point the
   input wished for — the capsule cast along the input for the wished travel since the last report,
   against walls only — and the server decides what that stride achieves.

**Why.** One number covers a stride stopped short, a nudge, a shove, a jam and a refusal; the suspension
is what makes it converge; the glide keeps it readable. The client still decides nothing about the
world: it reports intent and draws the answer.

**Accepted limitations.** The heuristic form can correct late under latency (step-11 R-E3); the exact
form is used whenever `acted_through` names a recorded request. A pre-correction stride the server
accepts after a correction moves the server's body, not the drawn one; the next report brings the two
together again (step-11 §22.15). The person capsule is a client constant equal to the only value the
server can have until a world can configure it (step-11 QE-7, R-S15-3).

**Revisit** when a world can configure a person's shape (IL-c), or if the correction count on the
accepted routes (`--world --link`, a free walk) is ever non-zero.

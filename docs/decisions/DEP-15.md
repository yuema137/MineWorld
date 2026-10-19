## DEP-15 — Observation deltas: the typed `ObservationDelta`, chosen by measurement

**Date** 2026-10-09 · **Status** selected; shipped in S11 PR S11-C · **Approved by** the primary session
at S11-C's design freeze (SD-C10's rule and QS11C-5, frozen before measuring) · **Relates to** `ARC-41`,
`ARC-23`, `ARC-43`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §15 · **Design**
`.structured-coding/plans/mvp0/step-12-server.md` §§4.8, 7.3, 17 (CP-C1, E-SC8) · **Specification**
[`server/PROTOCOL.md`](../server/PROTOCOL.md) §5.3

**Problem.** A hosted world sends every connected client a whole observation ten times a second. On a
hosted market town that is about 87 KB per client per second, almost all of it unchanged from the
frame before: the people in the room, their components, the list of what may be attempted.

**Options considered** (both directions of the reuse question).

```text
(a) the typed ObservationDelta (ours, server/src/protocol/delta.rs)   chosen, by the frozen rule
    keyed by contract identity: entities upserted / removed by EntityId,
    relations and affordances replaced whole when changed, events always
(b) RFC 6902 JSON Patch (the json-patch crate 4.2.0, MIT/Apache-2.0)  measured; 13 % smaller than (a)
    a generic diff of the observation's JSON                          on this world; no GDScript
                                                                      applier exists
(c) whole observations only                                           the conforming fallback
(d) transport compression, permessage-deflate                         a dead end: Godot's
                                                                      WebSocketPeer cannot negotiate
                                                                      it (godot#103230, ARC-41)
(e) a binary encoding (MessagePack)                                   a different question: ARC-41
                                                                      keeps JSON; bytes are answered
                                                                      by deltas, not by encoding
```

**Measurement** (step-12 E-SC8; `tools/cli/tests/deltas.rs`, run explicitly). A hosted
`worlds/market-town --town`, four sessions (visitor, wanderer, and Alice and Bob taken over from their
in-server controllers), 60 wall seconds, 601 whole frames per client; on the same frames, per client
per second:

```text
client     whole       typed            json-patch
visitor    81 993 B/s  1 141 B/s        993 B/s
wanderer   93 733 B/s  1 161 B/s      1 012 B/s
alice      90 487 B/s  1 155 B/s      1 007 B/s
bob        81 552 B/s  1 140 B/s        992 B/s
mean       86 941 B/s  1 149 B/s (1.3 %)  1 001 B/s (1.2 %)
```

Every one of the 2 400 consecutive pairs reconstructs exactly: `apply(previous, diff(previous, next))`
equals `next` with its entities in id order (CA-9).

**Choice.** The rule frozen before measuring (step-12 SD-C10) reads: *typed* if it is at most half the
whole bytes and json-patch is not within 10 % of it; *json-patch* if within 10 % of typed and at most
half the whole bytes; *whole observations only* otherwise. Typed is 1.3 % of whole, and json-patch is
not within 10 % of it (it is 12.9 % smaller), so the rule selects **typed**, and it ships: `delta`
frames between keyframes (`PROTOCOL.md` §5.3), the first frame, every `--keyframe-every`-th frame
(default 50) and the first after a resume or a backfill being whole.

**Disagreement recorded.** step-12 §4.8, written before the rule, said the typed delta is kept "only
if it beats both". On these numbers it beats whole observations by a factor of 75 and loses to
json-patch by 148 B/s per client; read that way, neither non-whole branch of the frozen rule applies.
The freeze is the binding text and its rule was applied as written; the operator is told of the
tension in S11-C's handoff. What the rule's outcome keeps: an applier a client writes in a few lines
against `PROTOCOL.md` §5.3's table, with no RFC 6902 implementation in GDScript to adopt or write, and
deltas keyed by contract identity rather than by JSON paths into a list. What it gives up: about
150 B/s per client against json-patch, on a stream already 75 times smaller.

**Isolating interface.** `server/src/protocol/delta.rs` (`ObservationDelta`, `diff`, `apply`,
`canonical`); the Godot module's `mineworld/delta.gd`. `json-patch` is a dev-dependency of
`mineworld-cli` only, for this measurement, and is in no shipped artefact.

**Limitations accepted.** Measured on one town and one cadence; a world whose observations churn
differently may weigh the encodings differently, and the measurement is re-runnable as written. A
`delta` whose `base` is not the frame a client holds cannot occur on one WebSocket; a client that sees
one resumes, which yields a whole observation.

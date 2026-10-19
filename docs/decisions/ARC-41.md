## ARC-41 — Client protocol revision 2: specified whole, landed incrementally, and still JSON

**Date** 2026-10-08 · **Status** accepted; revision 2 begins on the wire in S11 PR S11-A · **Approved by**
the operator (QS11-1, QS11-15 as recommended, `overall.md` "Parallel build-out, 2026-10-08") and the
primary session at S11-A's design freeze · **Relates to** `DEP-3`, `ARC-25`, `ARC-34`, effort decision
`D-4` · **Design** `.structured-coding/plans/mvp0/step-12-server.md` §§5, 15 · **Specification**
[`server/PROTOCOL.md`](../server/PROTOCOL.md)

**Problem.** Step S11 changes the client protocol in five pull requests — authentication, seats and
hold, facts and deltas, admin, proof — while the 2D client (S12), the 3D client (S14) and the Python
cognition SDK (S10) are built against it at the same time. One revision number per pull request would
make three clients chase four revisions; one revision delivered at the end would block all three until
S11 finishes. Separately, effort decision `D-4` said Protobuf would be introduced "at the first real
cross-language boundary", and that boundary — the Godot client — has run on JSON since S5V.

**Choice, the revision rule.** Revision 2 is **specified whole** (step-12 §5, then `PROTOCOL.md`) and
**implemented incrementally**: `protocol` becomes `2` in the first pull request that changes the wire
(S11-A), and until the step completes a server may *omit* a frame or a field the specification marks as
arriving later, and never gives a frame or a field it sends a different meaning. `PROTOCOL.md` §10 is
the landing table. A client written against the whole specification is correct against every
intermediate `main`. Fields other steps asked for (the explicit takeover flag and time scale, S11-B;
`acted_through` and the cursor-resumable perceived stream, S11-C) are specified in `PROTOCOL.md` by the
pull request that lands them; until then a `join` carrying them is refused as malformed, because every
client frame denies unknown fields.

**Choice, the encoding.** **JSON text frames stay the wire encoding for MVP-0.** This supersedes
`D-4`'s *timing*, not its direction: the Rust types in `mineworld-contracts` remain the single source of
truth, and other languages mirror them — checked against golden frames the Rust tests keep in
`server/tests/frames/` — rather than through a parallel schema. Protobuf (`prost` with `godobuf`) was
declined for now because a mirror `.proto` of every contract type is the drift risk `R-3` names, the
GDScript generator has a single maintainer, and payloads owned by System Packs would be JSON inside
bytes anyway. MessagePack (`rmp-serde`) is the natural first step if a binary encoding is ever measured
to be needed, because it keeps `serde` as the one source; CBOR has no maintained GDScript decoder;
transport compression (`permessage-deflate`) cannot be negotiated by Godot's `WebSocketPeer`
(godot#103230). Bandwidth is answered by S11-C's measured deltas, not by the encoding.

**Disagreement recorded.** [`ARCHITECTURE.md`](ARCHITECTURE.md) §13.1 still states `D-4`'s timing. It is
stale; the edit belongs to S10's P3 (step-17 G-1), which introduces the second cross-language consumer,
and is recorded here so the contradiction is not silent (`CLAUDE.md` §2.1 rule 4).

**Limitation accepted.** Between S11's pull requests, two `main` commits that both say `protocol: 2`
differ in what they send. The landing table is the only place a client learns which; a client for a
deployed server built from an intermediate `main` reads that commit's `PROTOCOL.md`.

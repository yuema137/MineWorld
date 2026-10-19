## DEP-14 — Join secrets: `getrandom` to make them, `subtle` to compare them

**Date** 2026-10-08 · **Status** selected; dependencies added in S11 PR S11-A · **Approved by** the
primary session at S11-A's design freeze (step-12 §7.2, DEP-S11-a) · **Relates to** `DEP-3`, `ARC-41`,
[`NETWORKING.md`](NETWORKING.md) §9 · **Design** `.structured-coding/plans/mvp0/step-12-server.md`
§§4.1, 7.2, 7.7, 15

**Problem.** MVP authentication is a server invite token plus a player nickname (`NETWORKING.md` §9).
The server must make an unguessable invite when its operator gives none (and, from S11-B, a resume
secret per seat binding), and must compare an offered invite with the real one without the comparison's
timing telling an attacker how many leading characters were right.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions).

```text
credential carriage
(a) invite + nickname in the protocol's join frame, checked by
    server/src/admission.rs                                    chosen: transport-independent, works for
                                                               a browser client, never in a URL
(b) tower-http bearer validation on the /ws upgrade           a browser cannot set the header; it
                                                               authenticates the transport, not the
                                                               protocol
(c) JWT / PASETO session tokens (jsonwebtoken)                accounts and expiry nobody has yet
(d) axum-login / tower-sessions                               cookie sessions and a user model
                                                               MineWorld does not have
constant-time comparison
(e) subtle 2.6 (ConstantTimeEq)                               chosen
(f) constant_time_eq 0.6                                      acceptable; subtle preferred for its
                                                               review history
(g) hash both sides (sha2) and compare                         an indirect argument for no gain
(h) our own fold-and-or loop                                   the classic way to be undone by an
                                                               optimizer
secret generation
(i) getrandom 0.3 (the OS random source)                      chosen: already compiled
(j) rand 0.9                                                  a full RNG API for 16 bytes
(k) uuid v4                                                   an identifier, not a secret format
(l) WorldInstanceId::allocate's clock ⊕ pid ⊕ ordinal         guessable
join rate limiting
(m) one guess per connection + a fixed 500 ms delay (ours)    chosen for MVP-0, a LAN or a tunnel
(n) governor / tower_governor (GCRA per IP)                   deferred: the adopt route for public
                                                               hosting
```

**Choice.** `subtle = "2.6"` and `getrandom = "0.3"`, both in the root `[workspace.dependencies]` and
used by `mineworld-server` only.

**Facts, verified 2026-10-08** (`cargo info`, `cargo tree`):

```text
subtle            2.6.1, BSD-3-Clause, dalek-cryptography/subtle; no dependencies; default features
                  std, i128. New to Cargo.lock.
getrandom         0.3.4, MIT OR Apache-2.0, rust-random/getrandom. Already in Cargo.lock as a normal
                  dependency of mineworld-server (rand_core ← rand ← tungstenite ← tokio-tungstenite
                  ← axum), so this adds an edge, not a package. 0.4.3 exists; 0.3 avoids a second copy.
constant_time_eq  0.6.1, CC0-1.0 OR MIT-0 OR Apache-2.0 — the recorded alternative.
```

**Why not ourselves.** Both are commodity primitives (`REUSE_POLICY.md` §4); an own constant-time loop is
exactly what an optimizer can turn back into an early exit. **Why ours for the rate limit.** One guess
per connection and a fixed delay are a few lines and suffice for a 128-bit invite on a LAN; `governor`
needs per-IP keying that is wrong behind a proxy, and is the adopt route the day public hosting is in
scope.

**Isolating interface.** `server/src/admission.rs` is the only file that names either crate. It exposes
`InviteToken` (generated or operator-given; `Debug` redacted; no `Serialize`), `OfferedInvite`,
`Nickname`, `Admission::admit` and `UNAUTHORIZED_DELAY`. No secret type implements `Serialize`, so none
can reach a frame, a fact or a save by accident.

**Limitations accepted.** `subtle`'s slice comparison returns early on unequal lengths: an invite's
length is not secret (a generated one is always 32 characters). Secrets cross a LAN in clear over
`ws://`; TLS comes from a gateway or a tunnel (`NETWORKING.md` §7, QS11-14). An operator-given invite on
the command line is visible to other local users through the process list; `MINEWORLD_INVITE` avoids
that.

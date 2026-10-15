# PR S27-a — Seat grants and a per-seat request budget (server, protocol)

## DRAFT — PR DESIGN, NOT FROZEN

```text
Design revision:        revision 1 (2026-10-14), DRAFT
Author:                 S27-a/b/c PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s27abc, branch docs/s27abc-designs,
                        planning base origin/main @ df77b27
Approved by / evidence: — (not frozen; the primary session freezes, overall.md §5 ruling 9)
Implementation base:    origin/main at C0; C0 records the commit
Start condition:        now (step-27 §11.4: "No S27 dependency; S11-E not required"), on the primary
                        session's dispatch, after this design is frozen
Execution contract:     §12, filled at freeze
Implementer:            Opus-class (step-27 §11.4)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-27-agent-players.md`](step-27-agent-players.md) §§2.2 (F-S27-1, F-S27-3,
F-S27-4), 3 (INV-S27-1, -3, -4, -5, -11, -13), 6.3–6.5, 7.2, 8 (the `governor` row), 11.2 "PR S27-a", 11.4, 12
(`ARC-97`, `DEP-57`), 13.1, 16 (AC-S27-6, AC-S27-14) · **Parent:** [`overall.md`](overall.md) · **Siblings:**
[`pr-s27b-group-activity-offers.md`](pr-s27b-group-activity-offers.md),
[`pr-s27c-player-library.md`](pr-s27c-player-library.md) (S27-c uses the invite until this PR lands; the
grant variant of the play-test and the adapter's `rate_limited` wait are added by whichever lands second, §10)
· **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 3, 7, 8, 9, 10, 12, 16); `server/PROTOCOL.md` (revision 2,
whole); `docs/NETWORKING.md` §§2, 9; `docs/DECISIONS.md` `ARC-40` (`:3667`), `ARC-41`, `ARC-44` (`:4298`),
`ARC-67`, `ARC-75`, `DEP-14` (`:4007`); `docs/REUSE_POLICY.md` §§11–12, 15; `clients/protocol/ADOPTION.md`
§4.2.

**Decision records written here.** `ARC-97` (grants and the budget) and `DEP-57` (`governor` or our own
bucket), both in C0, after the prototype (step-27 §12). Numbers assigned by the primary session 2026-10-14.

---

## 0. Binding inputs, exactly restated

- **§6.4 (frozen).** `mineworld server WORLD --grant visitor --grant wanderer --grant alice:take-over …`.
  Each `--grant SEATS[:take-over]` (one seat, or several joined by `+`) makes the server generate a token from
  the OS random source and print it once: `[mineworld] grant visitor <token> — join with: <address>
  seat=visitor invite=<token>`. A grant token is never given on the command line, never persisted, never in a
  save, observation, fact or `/status`; a restart makes new ones. A `join` presents it in the existing
  `invite` field — no new field, no new frame. Check 2 admits it. A new check between "seat" and "perceived"
  refuses a seat the grant does not name with `not_granted` (the connection stays). Check 6 refuses
  `take_over: true` on a connected or held seat with `not_granted` unless the grant carries `take-over`.
  Resuming one's own held seat with `resume` is always allowed. A hosted seat is joined without a flag. The
  invite, the admin token and every grant token must be distinct; the server refuses to start otherwise.
  Grants are host state (`ARC-40`). `GET /admin/sessions` shows which credential kind admitted a session
  (`invite` or `grant`), never the token. Nothing here is agent-specific (INV-S27-13).
- **§7.2 (frozen).** Each seat has a token bucket of `R` requests per wall second sustained and `B` burst,
  kept by the seat table (keyed by seat, so a reconnect or a takeover does not refill it). A `submit` that
  finds it empty is refused `rate_limited` before an `ActionId` is allocated — the same place as `paused` —
  with `retry_after_ms`. It applies to every connection identically and not to in-server controllers.
  Default floor `R = 10`, `B = 20`; C0 measures the 2D walker and the 3D client's harness traces; the frozen
  rule is `R = max(10, 2 × measured p99)`, `B = 2R`, configurable by `--request-budget R/B` with `0` meaning
  "no budget".
- **§8 (frozen).** Adopt `governor` (0.10.4, MIT) for one direct limiter per seat with its fake clock for
  tests (**verify** clock injection and licence at C0); if the prototype shows its clock cannot be driven from
  the seat table's injected `Instant`, a ~40-line token bucket in `seats.rs` with the decline recorded
  (`DEP-57`).
- **F-S27-4 (frozen).** "S27-a's two codes must land in the SDK and the Godot module in the same PR, and in
  `PROTOCOL.md` §10's landing table." A rule "treat an unknown refusal code as a generic refusal" is recorded
  for the next protocol revision, not retrofitted.
- **CP-a and Adversarial (step-27 §11.2)**, restated as §8 below.
- **Not in scope (frozen).** Per-action embodied cadence (QS27-7); bans; durable identity; any change to the
  invite's meaning; a `:watch` grant (QS27-6).

---

## 1. Goal

Give a host two server rules that are the same for every connection, human or agent: **(a) a narrower
credential** — a seat grant that admits only the seats it names, with or without the right to take a seat
another player holds; **(b) a per-seat request budget** that refuses a flood of `submit`s cleanly, before the
world allocates anything. Both are host state: no fact, no journal entry, no revision, nothing in a save. The
two new refusal codes land in the protocol, the Python SDK and the Godot clients in the same PR, so no
existing client breaks.

## 2. Scope

### 2.1 In scope

1. `server/src/admission.rs`: `GrantToken`, `SeatGrant`, `SeatRights`, `Credential`/`CredentialKind`;
   `Admission::with_grants`; `admit` returning the credential; distinctness.
2. `server/src/app.rs`: `Access::new` checking the admin token against the invite **and** every grant.
3. `server/src/seats.rs` (+ `seats/tests.rs`): `JoinRequest.rights`; the take-over right in `decide`; rename
   the crate-private `Grant` (a granted join, `:120-125`) to `JoinGranted` (terminology, F-S27a-3); the
   per-seat budget (`spend`) keyed by seat.
4. `server/src/runtime.rs`: the seat-named check in `join` (`:219-226`); the budget check in `submit` after
   `paused` (`:288-292`).
5. `server/src/session/join.rs`, `server/src/session.rs`: the credential from `admit` carried into the
   `JoinRequest` (rights only, never the token) and into the registry.
6. `server/src/admin/registry.rs`, `server/src/admin.rs`: `Joined.credential`; `SessionView.credential`.
7. `server/src/protocol.rs`: `RefusalCode::{NotGranted, RateLimited}`; `ServerFrame::Refused.retry_after_ms`;
   `Refusal::retry_after`.
8. `server/src/host.rs`: `HostConfig.request_budget`.
9. `server/src/lib.rs`: re-exports.
10. `tools/cli/src/main.rs`, `tools/cli/src/serve.rs`, `tools/cli/src/invite.rs`: `--grant`,
    `--request-budget`, the grant lines, startup refusals.
11. Tests: `server/src/seats/tests.rs`, `server/src/admission.rs` tests, `server/tests/grants.rs` (new),
    `server/tests/request_budget.rs` (new), `server/tests/frames.rs` and two new goldens,
    `server/tests/admin.rs`, `server/tests/facts.rs` (one struct literal, `:307`), `tools/cli/tests/grants.rs`
    (new), `tools/cli/tests/request_rate_2d.rs` (new, `#[ignore]`, Godot), `server/tests/traces/` (new
    fixture).
12. SDK: `sdk/python/src/mineworld_sdk/wire/frames.py`, `session.py`, `reconnect.py`;
    `sdk/python/tests/test_golden_frames.py`.
13. Godot: `clients/2d/scripts/link.gd` (`FINAL`); `clients/shared/settings/locale/en.po`, `zh_Hans.po`,
    `messages.pot` (regenerated).
14. Specs: `server/PROTOCOL.md` §§4.1, 4.2, 5.5, 8, 10, 11.1, 11.2; `docs/NETWORKING.md` §9 (one paragraph);
    `clients/protocol/ADOPTION.md` §4.2 (one sentence); `server/README.md` (one line, if it lists flags);
    `docs/DECISIONS.md` `ARC-97`, `DEP-57`.
15. `Cargo.toml` / `Cargo.lock` / `server/Cargo.toml`: `governor`, only if `DEP-57` adopts it.

### 2.2 Not in scope

Per-action cadence (QS27-7); bans and durable identity; a `:watch` grant (QS27-6); per-address join rate
limiting (`DEP-14` option (n)); any change to the invite's semantics; any change to the client vocabulary
(`join`/`submit`/`leave`); any agent-specific branch; making the SDK or Godot module tolerant of unknown codes
(F-S27-4: next revision); `CORE_CONCEPTS.md` §14 and `MODULE_SPEC.md` (step-27 §13.1, the primary session's).

### 2.3 Dependencies

None unmerged. The SDK (P3, P3b), S11-A…D and the Godot settings layer (S20) are on `main`.

---

## 3. Audit (`origin/main @ df77b27`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-a-1 | `Admission { invite }`; `admit(&OfferedInvite) -> Result<(), Unauthorized>` compares in constant time; one invite per process | `server/src/admission.rs:287-315` |
| A-a-2 | `InviteToken::generate` (16 random bytes → 32 hex), `reveal` called once by the composition root; redacted `Debug`, no `Display`/`Serialize` | `server/src/admission.rs:55-92`, `:238-243` |
| A-a-3 | `AdminToken::is_invite` is the only distinctness check; `serve.rs::admin_access` refuses an admin token equal to the invite | `server/src/admission.rs:132-137`; `tools/cli/src/serve.rs:258-270` |
| A-a-4 | `Access { admission, admin }` with `From<Admission>` | `server/src/app.rs:55-76` |
| A-a-5 | The join checks in the connection task: protocol (`:43`), invite with the 500 ms delay (`:52-59`), nickname (`:60-65`), then a `JoinRequest { seat, take_over, resume, session }` to the world thread (`:68-81`), then the backfill | `server/src/session/join.rs:36-93` |
| A-a-6 | On the world thread: unknown seat (`:220-223`), `observer_of` → `seat_not_in_world` (`:224`), `perceived_start` → `cursor_unavailable` (`:226`), then `table.join` (control) | `server/src/runtime.rs:214-266` |
| A-a-7 | `decide`: resume first (held or live secret → `held`, else `invalid_resume`); free → `none`; hosted → `hosted`; connected/held with `take_over` → `connection`; else `seat_occupied` | `server/src/seats.rs:341-371` |
| A-a-8 | `JoinRequest` is `pub` with `pub` fields (`pub mod seats`); `JoinRequest::plain`; external literal in `server/tests/facts.rs:307` | `server/src/seats.rs:27-50`; `server/src/lib.rs:81`; `server/tests/facts.rs:30, 171, 281, 307, 755, 762` |
| A-a-9 | The crate-private `seats::Grant { took_over, displaced }` means "a granted join" — the same word as the new domain term | `server/src/seats.rs:120-125`, `:176`; `runtime.rs:230-240` |
| A-a-10 | `SeatTable` is owned by the world thread, time injected (`Instant` for holds, `WorldTime` for controllers); unit tests drive it with literal instants | `server/src/seats.rs:1-14`, `:127-186`; `server/src/seats/tests.rs:1-80` |
| A-a-11 | `WorldRuntime::submit(subscription: Option<SubscriptionId>, …)`: `paused` refused before allocation (`:288-293`); `submit_at` does the actor check and allocates the `ActionId` (`:313-348`). In-server controllers call `submit_at` directly from `consult` (`:429`); `WorldHost::submit` (library callers) passes `None` | `server/src/runtime.rs:276-348`, `:405-437`; `server/src/host.rs:400-430` |
| A-a-12 | `HostConfig` (epoch, cadence, backlogs, hold, time scale, keyframes) with `Default` | `server/src/host.rs:217-260` |
| A-a-13 | `ServerFrame::Refused { token, code, detail }`, both optional fields `skip_serializing_if`; `RefusalCode` is a closed enum of 17 codes; `Refusal` builder | `server/src/protocol.rs:271-281`, `:293-342`, `:344-395` |
| A-a-14 | `Joined { nickname, seat, observer, connected_at }` is listed by the registry; `SessionView` serializes it for `GET /admin/sessions` | `server/src/admin/registry.rs:26-34`; `server/src/admin.rs:121-160` |
| A-a-15 | CLI flags: `--invite`/`MINEWORLD_INVITE`, `--admin-token`/`MINEWORLD_ADMIN_TOKEN`, `--hold`, … ; `Invite::join_line`; `suggested_seat` | `tools/cli/src/main.rs:96-143`; `tools/cli/src/invite.rs:23-78`; `tools/cli/src/serve.rs:96-124`, `:219-231` |
| A-a-16 | Golden frames: one file per frame kind plus `refused-cursor_unavailable.json`, `refused-lagged.json`; `server/tests/frames.rs` checks each; the SDK's `test_every_golden_frame_has_a_model` fails for a golden file with no Python check | `server/tests/frames/`; `server/tests/frames.rs:30-80`, `:192-215`; `sdk/python/tests/test_golden_frames.py:38-43`, `:135-143` |
| A-a-17 | **SDK:** `RefusalCode` is a closed `Literal` (`:150-168`); `WireModel` is `extra="forbid"` (`contract.py:47-57`), so an unknown code **or** an unknown field on `refused` is a `MalformedFrame` that ends the session (`codec.py:28-36`; `session.py:381-390`). `RefusedRequest(code, detail)` (`session.py:115-124`); `route` builds it (`:444-447`). `reconnect.py`'s `SeatLossReason` and `_FINAL_REFUSALS` (`:33-67`); an unknown refusal on a join is `SeatLost("protocol_violation")` (`:137-143`) | `sdk/python/src/mineworld_sdk/…` |
| A-a-18 | **Godot module:** a refusal's `code` is a `String` passed on unchanged (`world_client.gd:65`, `:516-520`); nothing decodes codes as a closed set. A refusal while reconnecting other than `invalid_resume` disconnects (`:539-548`) | `clients/protocol/mineworld/world_client.gd` |
| A-a-19 | **2D client:** `link.gd` `FINAL` lists the refusals that end retrying for good (`unknown_seat`, `seat_not_in_world`, `already_joined`, `unauthorized`, `protocol_mismatch`, `invalid_nickname`, `left`); any other join refusal is retried with capped back-off forever | `clients/2d/scripts/link.gd:23-26`, `:84-107` |
| A-a-20 | **Text:** a refusal is worded through `reason.<code>`; a code with no catalog entry reads as the code made readable ("an unknown code reads, never raw"); `tests/acceptance/tests/client_text.rs` reads `PROTOCOL.md` §5.5's table and **fails** unless `en.po` and `zh_Hans.po` hold `reason.<code>` for every code; `messages.pot` is generated by `MINEWORLD_WRITE_POT=1 cargo test -p mineworld-acceptance --test client_text` | `clients/shared/settings/text.gd:141-152`, `:183-190`; `clients/shared/checks/text_check.gd:22-23`; `tests/acceptance/tests/client_text.rs:362-380`, `:564-592`; `clients/shared/settings/locale/messages.pot:1-3`; `en.po:417-430` |
| A-a-21 | **3D client:** shows a refusal through the same catalog and does not retry | `clients/3d-spike/scripts/slice/slice_link.gd:198-213`, `:577-591` |
| A-a-22 | `ADOPTION.md` §4.2: "the server neither knows nor checks how often you send" — false once a budget exists | `clients/protocol/ADOPTION.md:349-352` |
| A-a-23 | `PROTOCOL.md` §4.2 says "**Joining (check 5 of §4.1)**" but control is check 6 since S11-C added `perceived` | `server/PROTOCOL.md:191` vs `:132-147` |
| A-a-24 | The 2D walker: `STRIDE_M 1.9`, `WALK_SPEED 1.6` m/s, a `move` every `REPORT_M 0.5` m or `REPORT_TURN_DEG 20°`, **one request in flight** (`_token == ""`) | `clients/2d/scripts/walker.gd:19-24`, `:257-286` |
| A-a-25 | The 2D harness prints `RESULT {token, result, t_ms}` for every answer and `REQUEST {token, request}` for every submit; `godot2d::Drive::start(server, seat, args)`, `lines()`, `tagged(lines, prefix)`; Godot tests are `#[ignore]`, need `godot` on PATH; Godot 4.7 is on the planning machine (`/opt/homebrew/bin/godot`), not in CI | `clients/2d/scripts/harness/drive.gd:76-81`; `tools/cli/tests/godot2d/mod.rs:33-43`, `:84-201`; `.github/workflows/ci.yml` (no Godot) |
| A-a-26 | `deny.toml` allows MIT; `multiple-versions = "allow"` | `deny.toml:12-14`, `:52` |
| A-a-27 | `getrandom 0.3.4`, `subtle 2.6.1` are the server's secret dependencies (`DEP-14`); `Cargo.lock` has no `governor`, `parking_lot`, `spinning_top`, `portable-atomic`, `nonzero_ext`, `futures-timer`, `web-time` | `server/Cargo.toml:21-24`; `Cargo.toml:95-96`; `Cargo.lock` |

### 3.2 External facts (read 2026-10-14)

| Fact | Source |
| --- | --- |
| `governor` 0.10.4, created 2025-12-16, licence field `MIT`; repository `boinkor-net/governor`; root `LICENSE` is the MIT License ("Copyright (c) 2023 Andreas Fuchs") | `https://crates.io/api/v1/crates/governor`; `https://raw.githubusercontent.com/boinkor-net/governor/master/LICENSE` |
| `RateLimiter::direct_with_clock(quota, clock)`; `check() -> Result<_, NotUntil<P>>`; `NotUntil::wait_time_from(from: P) -> Duration`, `earliest_possible() -> P` | `https://docs.rs/governor/0.10.4/governor/struct.RateLimiter.html`, `…/struct.NotUntil.html` |
| `clock::Clock` (`type Instant: Reference; fn now(&self) -> Self::Instant`, `Clone`) and `clock::Reference` (`duration_since`, `saturating_sub`, `Add<Nanos>`) may be implemented by the user; `FakeRelativeClock` (`advance(&self, Duration)`, an `Arc`'d atomic); `MonotonicClock` is std `Instant`-backed | `https://docs.rs/governor/0.10.4/governor/clock/index.html`, `…/clock/struct.FakeRelativeClock.html` |
| Features: `default = ["std", "dashmap", "jitter", "quanta"]`; with `default-features = false, features = ["std"]` the dependencies are `nonzero_ext`, `parking_lot`, `spinning_top`, `portable-atomic`, `futures-timer`, `futures-util`, `futures-sink`, `cfg-if`, `smallvec`, `web-time`, `hashbrown 0.16` | `https://raw.githubusercontent.com/boinkor-net/governor/master/governor/Cargo.toml` |

**Verify at C0 (not confirmed from source here):** whether `Reference` is implemented for
`std::time::Instant` (so a custom `Clock` can return the seat table's injected `Instant`), and whether
`RateLimiter`'s `Clock` bound requires `Send + Sync` (the seat table lives on one thread). Both decide `DEP-57`.

### 3.3 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S27a-1 (F-S27-4 audited)** | The step says the Godot protocol module decodes codes as a closed set. It does not (A-a-18): codes are strings, and the text layer reads an unknown code readably (A-a-20). What does break: **(1)** the SDK, for an unknown code *and* for the new `retry_after_ms` field (A-a-17); **(2)** `client_text.rs`, which fails CI as soon as §5.5 lists a code with no catalog entry (A-a-20); **(3)** the 2D client's retry loop, which would retry a `not_granted` join forever (A-a-19). | §5.4: one PR lands the codes everywhere at once (C2), before any server path can send them (C3, C5). |
| **F-S27a-2** | `ADOPTION.md` §4.2 states the server never checks cadence (A-a-22). | Amended in C1: the server bounds requests per seat generically; walking cadence is still the sender's (QS27-7). |
| **F-S27a-3** | The seat table's `Grant` (a granted join, A-a-9) would collide with "seat grant" (`CLAUDE.md` §2.1(3)). | Renamed `JoinGranted` in C3 (crate-private, no public effect). |
| **F-S27a-4** | `PROTOCOL.md:191` names control "check 5" (A-a-23). | Corrected to "check 6" in C1 while §4.1 is being edited. |
| **F-S27a-5** | An old SDK fails closed on the new codes, never silently: `MalformedFrame` ends the session; on a join it becomes `SeatLost("protocol_violation")` (A-a-17). | Acceptable for the clients that cannot receive the codes (§5.4 (a)); recorded in `PROTOCOL.md` §10's row. |

---

## 4. Reuse, both ways (`REUSE_POLICY.md` §§11–12, 15)

| Component | Options | Decision |
| --- | --- | --- |
| Grant token generation and comparison | (a) `getrandom` + `subtle`, as the invite (`DEP-14`); (b) a JWT/PASETO library; (c) HMAC-derived tokens from the invite | **(a)**: the same secret class as the invite, already compiled. (b) adds expiry and claims nobody needs (`DEP-14` (c)); (c) makes every grant derivable from one secret and changes the invite's meaning |
| Per-seat token bucket | (a) `governor` direct limiter per seat, custom clock; (b) `governor` keyed limiter (`hashmap_with_clock`); (c) our own integer bucket; (d) `tower_governor` / `tower::limit` | **(a) if C0's prototype passes**, else **(c)** (`DEP-57`, the step's frozen rule). (b) keeps keys in a `HashMap` the seat table cannot iterate deterministically and duplicates the seat table's own key set; (d) limits HTTP requests, not WebSocket frames, and sits in the transport, where a per-seat key does not exist |

---

## 5. Design

### 5.1 Credentials (`admission.rs`, `app.rs`)

```text
GrantToken     16 OS-random bytes → 32 lowercase hex; generated only (never given, never parsed from argv
               or env); redacted Debug; no Display/Serialize; reveal() called once, for the startup line
SeatGrant      { seats: BTreeSet<EntityKey> (non-empty), take_over: bool, token: GrantToken }
SeatRights     Any                                   (the invite: every seat, take-over of any)
               Named { seats: BTreeSet<EntityKey>, take_over: bool }   (a grant; no token inside)
Credential     { kind: CredentialKind, rights: SeatRights }   CredentialKind = Invite | Grant (Serialize,
               snake_case); /admin/sessions serializes kind plus, for a grant, rights.seats and
               rights.take_over as `CredentialView` (QS27a-1), never a token
Admission      { invite: InviteToken, grants: Vec<SeatGrant> }
  new(invite)                         as today (no grants)
  with_grants(self, grants) -> Result<Self, AdmissionError>   refuses a grant whose token equals the invite
                                      or another grant's (AdmissionError::CredentialsNotDistinct)
  admit(&OfferedInvite) -> Result<Credential, Unauthorized>
                                      compares the offer with the invite and with every grant, each in
                                      constant time, all of them always (no early return: the result is
                                      folded), then answers which matched
Access::new(admission, admin) -> Result<Access, AdmissionError>
                                      refuses an admin token equal to the invite or to any grant
```

`From<Admission> for Access` stays (no admin). `serve.rs::admin_access` becomes a call to `Access::new`.
`Unauthorized` and the 500 ms delay are unchanged: a wrong credential, invite or grant, is one answer
(`unauthorized`), so the answer says nothing about which kinds exist.

### 5.2 Seat-named and take-over checks (`runtime.rs`, `seats.rs`, `session/join.rs`)

- `session/join.rs` keeps the order of §4.1. `admit` now returns the `Credential`; its `rights` go into the
  `JoinRequest` (a new `pub rights: SeatRights` field; `JoinRequest::plain` sets `SeatRights::Any`), and its
  `kind` into the registry's `Joined` (§5.5). The token never leaves the connection task (`ARC-40`, step-12
  I-5): the world thread receives seat names and a boolean.
- **Check 4a (new), `runtime.rs::join`, between `observer_of` (`:224`) and `perceived_start` (`:226`):** if
  `!request.rights.names(seat)` → `Refusal::new(RefusalCode::NotGranted).detail("this credential does not
  admit this seat")`. The connection stays (`Joining::Refused`), exactly as for `unknown_seat`.
- **Check 6, `seats.rs::decide` (`:360-370`):** the resume rule (`:349-359`) is unchanged and comes first, so
  resuming one's own hold works with any credential. In the `Connected`/`Held` arms with `take_over: true`,
  if `!request.rights.may_take_over()` → `NotGranted` ("this credential may not take a seat another player
  holds"). Without `take_over`, `seat_occupied` as today. `Free` and `Hosted` are granted whatever the flag.
- Order proof (the adversarial mutation, M-a-1): a `not_granted` from either check displaces nobody, because
  both return before `SeatTable::join` inserts the binding and before `runtime.join` releases a displaced
  connection (`:237-240`).

### 5.3 The per-seat budget (`seats.rs`, `runtime.rs`, `host.rs`)

```text
RequestBudget  { per_second: NonZeroU32, burst: NonZeroU32 }         HostConfig.request_budget: Option<…>
               Default: Some(R/B) with C0's frozen numbers (floor 10/20)
SeatTable      budgets: BTreeMap<EntityKey, Bucket>   one per seat, created on its first spend; never
               removed or reset by join, depart, kick, release, hold or expiry (keyed by seat, §7.2)
  spend(subscription, now: Instant) -> Result<(), Duration>
               finds the seat Connected to `subscription` (none: Ok — nothing seated can submit); takes one
               token or answers how long until one is available
WorldRuntime::submit  paused → (subscription is Some and a budget is set) spend → submit_at
               Err(wait) → Refusal::new(RefusalCode::RateLimited).retry_after(wait).detail(…)
```

- Before allocation: no `ActionId`, no journal entry, no `acted_through` change, no revision (§7.2; the same
  property as `paused`, A-a-11). A paused world takes no token (its check comes first).
- Not applied to `consult` (in-server controllers, `submit_at` directly) nor to `WorldHost::submit` with no
  subscription (library callers and tests), as §7.2 says.
- `retry_after_ms` is the bucket's wait rounded **up** to a whole millisecond, ≥ 1.
- **Bucket implementation (`DEP-57`).** If C0's prototype passes: one `governor::RateLimiter` per seat, built
  with `direct_with_clock(Quota::per_second(R).allow_burst(B), TableClock)`, where `TableClock` is a clock the
  seat table sets to its injected `now` before each `check` (the exact shape — a shared cell, or a
  `Reference` newtype — is what the prototype settles); `retry_after = not_until.wait_time_from(now)`.
  Otherwise our own: integer milli-tokens, `tokens = min(B·1000, tokens + elapsed_ms·R)`, a spend takes 1000,
  `wait_ms = ceil((1000 − tokens) / R)`; ~40 lines in `seats.rs`, with the same tests.

### 5.4 Landing the two codes without breaking a client (F-S27-4, F-S27a-1)

Decision, with reasons:

| | Rule | Why it is safe |
| --- | --- | --- |
| (a) | **The codes reach only a client that causes them.** `not_granted` answers only a `join` that presented a grant token — a credential that did not exist before this PR. `rate_limited` answers only a seat that exceeded `R` (≥ 2 × the measured p99 of the 2D and 3D clients, C0) | An unmodified client that keeps behaving as it does today can never receive either. `retry_after_ms` appears **only** on a `rate_limited` refusal, so no frame an old client can receive changes shape |
| (b) | **Protocol stays revision 2**; the codes and the field are rows of §10's landing table ("S27-a — landed"; "before it lands: never sent") | Raising `protocol` to 3 would refuse every existing client `protocol_mismatch` for an additive change; §10 already provides additive landing within revision 2 |
| (c) | **Every client in the repository learns them in this PR (C2)**, before any server code can emit them (C3, C5): the SDK (`RefusalCode` literals, `Refused.retry_after_ms`, `RefusedRequest.retry_after_ms`, `not_granted` a final join refusal, `SeatLossReason` `not_granted`); the 2D client (`not_granted` in `FINAL`); the shared catalog (`reason.not_granted`, `reason.rate_limited` in `en.po` and `zh_Hans.po`, `messages.pot` regenerated); the Godot module itself needs **no change** (A-a-18) and the 3D client none (A-a-21) | Each commit leaves every client correct; `client_text.rs` and the SDK's golden test make a missing piece red |
| (d) | **An old client fails closed, loudly** (F-S27a-5); the "unknown code is a generic refusal" rule is written into §10 as a requirement of the next revision, not adopted now | The frozen ruling F-S27-4 |
| (e) | `rate_limited` is **not** final for anyone: it is a submit refusal; the 2D client shows it as a toast like any refusal (`app.gd:322-327`) and the walker ends that stride (`walker.gd:210-213`); the SDK hands it to the caller as `RefusedRequest` | The next stride or click is a new request |

Wording (catalog; the English `msgstr` is the readable code where nothing better reads): `reason.not_granted` →
"not granted" / zh_Hans "未获授权"; `reason.rate_limited` → "too many requests" / zh_Hans "请求过于频繁".

### 5.5 Admin and startup

- `Joined.credential: Credential` (no token inside), serialized as `CredentialView` (ruled, QS27a-1); `SessionView` gains
  `"credential": { "kind": "invite" }` or `"credential": { "kind": "grant", "seats": ["visitor"], "take_over":
  false }` — the grant's seat list and take-over right, never the token, and no controller kind (INV-S27-4).
  `GET /admin/seats` is unchanged.
- `--grant SEATS[:take-over]` (repeatable; `SEATS` = `seat` or `seat+seat+…`; any other suffix, an empty
  seat, a seat not in the roster, or a seat named twice in one grant refuses to start with a message naming
  the flag's value, which is not secret). Grant tokens are generated after the roster is known; each is printed
  once, in roster order of its first seat, as
  `[mineworld] grant <seats joined by +>[ take-over] <token> — join with: <address> seat=<first seat> invite=<token>`.
  No environment variable for grants (a token is never given, §6.4).
- `--request-budget R/B` (`R`, `B` positive integers, `B ≥ R`; `0` alone = no budget; any other form refuses
  to start). The startup summary line gains `request budget R/s burst B per seat` or `no request budget`.

### 5.6 Specifications (C1)

`PROTOCOL.md`: §4.1 row `4a grant` between `4 seat` and `5 perceived` (`refused not_granted; the connection
stays`), and the paragraph "**Seat grants**" (the `--grant` line, presented in `invite`, never printed again,
not persisted, distinct from the invite and admin token, host state); §4.2 "check 6" correction and a rule-4
sentence ("… unless take_over is true **and the credential may take over** …; otherwise not_granted");
§5.5 two rows (`not_granted`; `rate_limited` with `retry_after_ms`, "refused before the server allocates an
`ActionId`; submit again after `retry_after_ms`") and the `refused` example field list; §8 a "request budget"
row; §10 two rows plus the next-revision requirement; §11.1 the distinctness sentence extended to grants;
§11.2 the `credential` field (kind, and for a grant its seat list and take-over right; ruled QS27a-1). `NETWORKING.md` §9 the step's paragraph (§13.1). `ADOPTION.md` §4.2 the corrected
sentence (F-S27a-2).

---

## 6. Invariants

| Id | Invariant | Maps to | Checked by |
| --- | --- | --- | --- |
| I-a-1 | A grant admits exactly the seats it names; a grant without `take-over` never displaces a connected or held seat; resume of one's own hold works with any credential | INV-S27-5, INV-S27-13 | `seats/tests.rs` table; `server/tests/grants.rs`; CP-a |
| I-a-2 | A `not_granted` displaces nobody and grants nothing; the connection may join again | INV-S27-5 | `grants.rs` (M-a-1) |
| I-a-3 | A `rate_limited` refusal allocates no `ActionId`, journals nothing, moves no revision and no `acted_through` | INV-S27-1 (one path), INV-S27-7 | `request_budget.rs`; CP-a's save check |
| I-a-4 | The budget is keyed by seat: reconnect, resume, takeover, kick and release do not refill it | §7.2 | `seats/tests.rs`; `request_budget.rs` (M-a-3) |
| I-a-5 | Grants and the budget apply identically to every connection; no code path, field or log distinguishes agent from human; the admin surface reports a credential kind, never a controller kind | INV-S27-4, INV-S27-13 | review of the diff (no `agent` identifier in `server/`); AC-S27-6 shape test in `grants.rs` |
| I-a-6 | A grant token appears only on its one startup line: never in a frame, a fact, a save, `/status`, `/admin/*`, a `Debug` string or another log line | INV-S27-11 (server side) | `admission.rs` unit test; `tools/cli/tests/grants.rs` canary scan |
| I-a-7 | Host state only: no grant, join, refusal or bucket state is journaled, states a fact or moves a revision | `ARC-40`, INV-S27-7 | `request_budget.rs` revision check; review |
| I-a-8 | The client vocabulary is unchanged: no new client frame or field; `join.invite` carries the grant | `PROTOCOL.md` §2, INV-S27-1 | `server/tests/frames.rs` (client goldens unchanged) |
| I-a-9 | Every client in the repository decodes and words both codes in the same PR; protocol stays 2 | F-S27-4 | SDK golden tests; `client_text.rs`; C2 |
| I-a-10 | The 2D walking trace replayed at its recorded pace is never refused `rate_limited` | §7.2, CP-a | `request_budget.rs` replay test |

---

## 7. Test ownership (test rules §26)

| Layer | Owns |
| --- | --- |
| Static | `cargo fmt`, `cargo clippy -p mineworld-server -p mineworld-cli --all-targets -D warnings`, `cargo deny check` (licences, if `governor` lands); ruff/pyright for the SDK; the doc checks |
| Unit (Rust, no socket) | `admission.rs`: grant tokens are fresh 32-hex, distinct, redacted; `with_grants` and `Access::new` refuse collisions; `admit` returns the right `Credential` for invite, each grant, and a near miss. `seats/tests.rs`: the decision table of §5.2 (every binding × rights × take_over × resume), budget transitions with injected `Instant` (burst, refill, `retry_after` exact, rounding up, keyed by seat across depart/rejoin/takeover/kick/release/expiry) |
| Codec | `server/tests/frames.rs`: `refused-not_granted.json`, `refused-rate_limited.json` (with `retry_after_ms`); SDK `test_golden_frames.py` checks both |
| Server integration (in-process server, real sockets) | `server/tests/grants.rs` (CP-a's grant half; AC-S27-6's identical answers for the same request shape); `server/tests/request_budget.rs` (50 in one second; nothing allocated; keyed by seat; the 2D trace replay) |
| Real binary | `tools/cli/tests/grants.rs`: `--grant` parsing and refusals, the printed lines, the token scan over stdout/stderr/save/`/status`/`/admin/sessions`, `--request-budget` parsing and the startup line, the save's revision vs answered requests |
| Godot / text | `client_text.rs` (catalog completeness, CI); `clients/shared/checks/text_check.gd` unchanged |
| Measurement (operator machine, not CI) | `tools/cli/tests/request_rate_2d.rs` (`#[ignore]`, Godot): the 2D walker's per-second submit counts and the recorded trace |
| Gate 1 (real LLM) | NOT REQUIRED: no LLM-facing change |
| CI impact | `server/**`, `tools/cli/**` → Rust full fallback; `sdk/python/**` → `python` job on three OSes; `clients/**` → `fast` (client rules) and the acceptance crate; `Cargo.lock` → full |

Mutations (planted in review, seen red by name, reverted):

| Id | Mutation | Must turn red |
| --- | --- | --- |
| M-a-1 | The seat-named check moved after control (after `table.join`) | `grants.rs::a_grant_refused_a_seat_displaces_nobody` (the connected human is sent `closing taken_over`) |
| M-a-2 | `decide` ignores `may_take_over()` | `seats/tests.rs` decision table and `grants.rs::a_grant_without_take_over_cannot_displace_a_player` |
| M-a-3 | The bucket is stored in `Connected` (refilled on rejoin) | `seats/tests.rs::the_budget_is_the_seats_across_a_rejoin` and `request_budget.rs::reconnecting_does_not_refill_the_budget` |
| M-a-4 | The budget check after `allocate()` in `submit_at` | `request_budget.rs::a_refused_request_allocates_nothing` (an `ActionId` gap / the journal count) |
| M-a-5 | `admit` returns at the first match (no fold) | none observable by timing in CI — **honestly not mutation-testable**; owned by review of `admit` (recorded) |
| M-a-6 | `reason.rate_limited` removed from `zh_Hans.po` | `client_text.rs::the_catalogs_are_complete_and_consistent` |
| M-a-7 | `"rate_limited"` removed from the SDK's `RefusalCode` | `test_golden_frames.py::test_golden_frame[refused-rate_limited]` |

---

## 8. Acceptance criteria (CP-a and the adversarial set, fixed before measuring)

| Id | Criterion |
| --- | --- |
| AA-1 | Against the real binary started with `--grant visitor`: a join with that grant to `visitor` is welcomed; the same grant asking for `alice` is refused `not_granted` and the same connection may then join `visitor`. |
| AA-2 | A grant without `take-over` cannot displace a connected player (`not_granted`, the player stays connected); a grant with it can (`took_over: "connection"`, the player is sent `closing taken_over`). |
| AA-3 | A connection that dropped resumes its own held seat with its `resume`, whatever credential it joined with; a grant without `take-over` cannot take a seat another player's drop left held. |
| AA-4 | A client submitting 50 requests within one wall second has every request beyond the budget refused `rate_limited` with `retry_after_ms ≥ 1`; no refused one has an `ActionId`; the save's journal holds exactly the answered ones. |
| AA-5 | Reconnecting (resume or fresh join), or being taken over, does not refill a seat's bucket. |
| AA-6 | The 2D client's recorded walking trace (C0), replayed at its recorded times, is never refused `rate_limited`. |
| AA-7 | A grant token equal to the invite, to the admin token, or to another grant refuses to construct the server's `Access`/`Admission` (library test; grant tokens are generated, so the CLI cannot be given one). |
| AA-8 | No grant token appears in any stdout/stderr line other than its own startup line, in the save's bytes, in `/status`, or in `/admin/sessions`; `/admin/sessions` shows `credential: { kind: "grant", seats, take_over }` for a grant session (its seat list, ruled QS27a-1) and `{ kind: "invite" }` for an invite session. |
| AA-9 | The same join and submit sequence presented with the same grant by two connections answers identically (AC-S27-6's shape, without the adapter). |
| AA-10 | `server/tests/frames/*.json` client goldens unchanged; the two new server goldens round-trip in Rust and Python; `client_text.rs` passes; the 2D `FINAL` list contains `not_granted`. |
| AA-11 | Mutations M-a-1 … M-a-4, M-a-6, M-a-7 each turned a named test red; M-a-5 is recorded as review-owned. |
| AA-12 | C0's measurement recorded: the 2D walker's per-second submit p99 (and the 3D client's, or INCONCLUSIVE with the reason), and `R`, `B` fixed by `R = max(10, 2 × p99)`, `B = 2R`. |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately;
`[x]` needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run (test rules §§8–9).

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s27a`. Re-read the commit's section and every
file before editing it; do only that commit's work; run its validation and record command, count, wall time
and PASS/FAIL/INCONCLUSIVE; plant and revert its mutations; static checks (`cargo fmt --all --check`; `cargo
clippy -p <crate> --all-targets -- -D warnings`; for Python `uv run --locked ruff check sdk/python`, `ruff format
--check`, `pyright sdk/python`, pytest one path at a time; for Markdown `python3 scripts/check_doc_headings.py`,
`python3 scripts/check_decision_ids.py`); answer the review questions in §15; check `git status --short` against
**Files**; update §15 and the handoff in the same commit; stage by explicit path; commit with the message and
the session's attribution line. No `git add -A`, no `sed -i`, no heredoc, no `python3 -c`.

#### C0 — Ledger, measurement, prototype, and the decision records

- **Goal.** Fix `R`/`B` from evidence and decide `DEP-57` before any code depends on them (`REUSE_POLICY.md`
  §15; step §7.2). **Files:** this file (§15), `handoff-s27a.md` (new), `tools/cli/tests/request_rate_2d.rs`
  (new), `server/tests/traces/walker-2d.json` (new), `docs/DECISIONS.md` (`ARC-97`, `DEP-57`).
- Steps:
  1. Record the base commit; create the handoff (the working rules' fields; sequence C0–C6).
  2. Write `tools/cli/tests/request_rate_2d.rs`, `#[ignore]`d with the module doc "operator machine with
     Godot 4.7; not in CI": start `mineworld server worlds/market-town --save <scratch>` with
     `support::Server`, run `godot2d::Drive::start(address, "visitor", &[<the harness's walking scenario
     arguments; read drive.gd's argument list, `:22-35`, and pick the longest walking scenario>])`, collect
     `RESULT` lines (`tagged(lines, "RESULT")`), bucket their `t_ms` into wall seconds, print the per-second
     counts, their p50/p99/max, and write `{ "source": <git rev-parse HEAD>, "world": "market-town", "seat":
     "visitor", "requests": [ { "t_ms": …, "request": <the REQUEST line's request> } … ] }` to
     `server/tests/traces/walker-2d.json` when `MINEWORLD_WRITE_TRACE=1`.
  3. Run it: `cargo test -p mineworld-cli --test request_rate_2d -- --ignored --nocapture` (with and then
     without `MINEWORLD_WRITE_TRACE=1`); record the counts. Add the analytic bound beside it: one request in
     flight; `1.6 m/s ÷ 0.5 m` = 3.2 moves/s walking (A-a-24).
  4. The 3D client: look for a runnable 3D harness (`clients/3d-spike/scripts/slice/`,
     `tools/cli/tests/*` naming the slice). If one prints per-request timestamps, measure it the same way;
     otherwise record **INCONCLUSIVE (no timestamped 3D harness)** (ruled, QS27a-2: the recorded 2D trace is
     the evidence for `R`) and use the 3D slice's documented cadence
     (the walker's "the 3D slice's cadence", `walker.gd:22`) as its bound.
  5. Fix `R = max(10, 2 × p99)`, `B = 2R` and record them in §15 (frozen from here; a later change is a
     material stop).
  6. Prototype `governor` **outside the repository build**: `cargo new --lib target/scratch-s27a/governor-proto`,
     `governor = { version = "=0.10.4", default-features = false, features = ["std"] }`; implement a clock
     whose `now()` returns an `Instant` the test sets (the seat table's model, A-a-10), build a direct limiter
     `Quota::per_second(R).allow_burst(B)`, and assert in a test with no sleep: B checks pass at `t0`, the next
     fails with `wait_time_from(t0) == 1/R s`, one passes at `t0 + 1/R`. Record: whether `Reference` is
     implemented for `std::time::Instant`; the `Clock` bounds; `cargo tree -e normal -p governor-proto` (count
     the new crates against `Cargo.lock`); `cargo deny` is not run on the scratch crate.
  7. Decide `DEP-57` by the frozen rule: the clock can be driven from the injected `Instant` → adopt;
     otherwise our own bucket. Write `DEP-57` (problem, options (a)–(d) of §4, choice, facts verified
     2026-10-14 with sources, the crate list) and `ARC-97` (step §12's text, with §5.4's landing decision and
     the `R`/`B` evidence) at the end of `docs/DECISIONS.md`, in the file's record format (model: `DEP-14`,
     `:4007`).
- [ ] Implementation: steps 1, 2, 5, 7.
- [ ] Validation: steps 3, 4, 6, recorded; the doc checks.
- [ ] Review: is the measured scenario a fair upper bound of a person walking (one in flight, keys)? does
  `DEP-57` state both halves of the reuse question?
- **Commit message:** `docs(plan): S27-a — measured request rate, ARC-97, DEP-57`

#### C1 — Specification before code

- **Files:** `server/PROTOCOL.md`, `docs/NETWORKING.md`, `clients/protocol/ADOPTION.md`, `server/README.md`
  (only if it lists server flags).
- Steps: §5.6, every bullet; mark each new row "S27-a — landed" in §10 only in C6 (until then "S27-a").
- [ ] Implementation. [ ] Validation: doc checks. [ ] Review: terms exact (`seat`, `seat grant`, `credential`);
  no rule only in a README; §5.5's table still parses for `client_text.rs` (`| \`code\` |` rows).
- **Commit message:** `docs(protocol): seat grants and the per-seat request budget (S27-a, ARC-97)`

#### C2 — The two codes on the wire and in every client

- **Goal.** §5.4 (c), before anything can send them. **Files:** `server/src/protocol.rs`,
  `server/tests/frames.rs`, `server/tests/frames/refused-not_granted.json`,
  `server/tests/frames/refused-rate_limited.json`, `sdk/python/src/mineworld_sdk/wire/frames.py`,
  `sdk/python/src/mineworld_sdk/session.py`, `sdk/python/src/mineworld_sdk/reconnect.py`,
  `sdk/python/tests/test_golden_frames.py`, `clients/2d/scripts/link.gd`,
  `clients/shared/settings/locale/{en.po, zh_Hans.po, messages.pot}`.
- Steps:
  1. `RefusalCode::NotGranted`, `RefusalCode::RateLimited` (doc comments as §5.5's rows);
     `ServerFrame::Refused { …, #[serde(default, skip_serializing_if = "Option::is_none")] retry_after_ms:
     Option<u64> }`; `Refusal { retry_after: Option<Duration> }` with `#[must_use] fn retry_after(self,
     Duration)` (rounded up to ≥ 1 ms in `into_frame`). Every existing `Refused` construction passes `None`.
  2. Two goldens and their `frames.rs` tests, in the style of `refused_lagged` (`:204-215`).
  3. SDK: the two literals; `Refused.retry_after_ms: U64 | None = Field(default=None, exclude_if=…)`;
     `RefusedRequest.retry_after_ms: int | None = None` filled in `route`; `"not_granted"` in
     `SeatLossReason` and `_FINAL_REFUSALS`; the module docstrings' landed lists; two `CHECKS` entries.
  4. `link.gd` `FINAL` gains `"not_granted"` with a comment ("a credential that does not admit this seat will
     not admit it on retry").
  5. Catalog: `reason.not_granted`, `reason.rate_limited` in `en.po` and `zh_Hans.po` (§5.4 wording), placed
     with the other `reason.*` entries; regenerate `messages.pot` with
     `MINEWORLD_WRITE_POT=1 cargo test -p mineworld-acceptance --test client_text`.
- Validation: `cargo test -p mineworld-server --test frames`; `cargo test -p mineworld-acceptance --test
  client_text --test client_rules`; `uv run --locked pytest sdk/python/tests/test_golden_frames.py`; `uv run
  --locked pytest sdk/python/tests/test_resuming.py` (reconnect table); mutations M-a-6, M-a-7.
- [ ] Implementation. [ ] Validation. [ ] Review: no existing golden changed; `retry_after_ms` absent from
  every non-`rate_limited` frame; `not_granted` is final in both the SDK and the 2D client.
- **Commit message:** `feat(protocol): not_granted and rate_limited in the server, the SDK and the clients (S27-a, F-S27-4)`

#### C3 — Seat grants in the server library

- **Files:** `server/src/admission.rs`, `server/src/app.rs`, `server/src/seats.rs`, `server/src/seats/tests.rs`,
  `server/src/runtime.rs`, `server/src/session.rs`, `server/src/session/join.rs`,
  `server/src/admin/registry.rs`, `server/src/admin.rs`, `server/src/lib.rs`, `server/tests/facts.rs`,
  `server/tests/admin.rs`, `server/tests/grants.rs` (new), `server/tests/support/mod.rs` (a grant helper).
- Steps: §5.1, §5.2, §5.5's admin field; rename `Grant` → `JoinGranted`; `JoinRequest.rights`
  (`plain` = `Any`); fix `facts.rs:307`'s literal; `server/tests/admin.rs` expects `credential: { "kind": "invite" }` (QS27a-1).
  `grants.rs` covers AA-1, AA-2, AA-3, AA-9 and M-a-1/M-a-2 over real sockets with `Admission::new(..)
  .with_grants(..)` (the support module's pattern, `support/mod.rs:61-70`).
- Validation: `cargo test -p mineworld-server` (lib + `grants`, `seats`, `handshake`, `admin`, `facts`);
  M-a-1, M-a-2; clippy.
- [ ] Implementation. [ ] Validation. [ ] Review: the token never crosses into `runtime/`; `admit` folds all
  comparisons (M-a-5, review-owned); a `not_granted` returns before any binding change.
- **Commit message:** `feat(server): seat grants — narrower credentials that admit named seats (S27-a, ARC-97)`

#### C4 — `--grant` in the CLI

- **Files:** `tools/cli/src/main.rs`, `tools/cli/src/serve.rs`, `tools/cli/src/invite.rs`,
  `tools/cli/tests/grants.rs` (new).
- Steps: §5.5's `--grant` parsing, generation after the roster, the lines, the refusals; `admin_access` →
  `Access::new`. Tests: AA-7's CLI half (a malformed `--grant` refuses with exit status ≠ 0 and names the
  value); AA-8 (start with `--grant visitor --admin-token …` and canary nicknames; parse the token from the
  grant line; join with it; act; stop; scan stdout minus that line, stderr, every file under the save, the
  `/status` and `/admin/sessions` bodies for the token).
- Validation: `cargo test -p mineworld-cli --test grants --test server_command --test admin`.
- [ ] Implementation. [ ] Validation. [ ] Review: no env var reads a grant; the grant line follows the invite
  line's grammar so a launcher can parse it.
- **Commit message:** `feat(cli): mineworld server --grant SEATS[:take-over] (S27-a)`

#### C5 — The per-seat request budget

- **Files:** `server/src/seats.rs`, `server/src/seats/tests.rs`, `server/src/runtime.rs`, `server/src/host.rs`,
  `server/src/lib.rs`, `server/Cargo.toml`, `Cargo.toml`, `Cargo.lock` (the last three only if `DEP-57` adopts
  `governor`), `server/tests/request_budget.rs` (new), `tools/cli/src/main.rs`, `tools/cli/src/serve.rs`,
  `tools/cli/tests/grants.rs` (the `--request-budget` cases) .
- Steps: §5.3; `--request-budget` (§5.5). `request_budget.rs`: AA-4 (a client sends 50 `move`s of 0 mm to
  its own position within one second; counts `result` vs `refused rate_limited`; `retry_after_ms ≥ 1`;
  persisted world: the journal count equals the `result` count, read through the server's history API the
  tests already use, or through `mineworld inspect` in the CLI test — record which), AA-5, AA-6 (replay
  `server/tests/traces/walker-2d.json`'s requests at their recorded offsets with the actor set to the
  observer; assert zero `rate_limited`), M-a-3, M-a-4.
- Validation: `cargo test -p mineworld-server`; `cargo test -p mineworld-cli --test grants`; `cargo deny
  check` if `governor` was added; clippy.
- [ ] Implementation. [ ] Validation. [ ] Review: no token taken while paused; in-server controllers and
  `WorldHost::submit(None)` untouched; the bucket survives every binding transition.
- **Commit message:** `feat(server): a per-seat request budget, refused rate_limited before allocation (S27-a, ARC-97)`

#### C6 — Close-out

- **Files:** this file (§15), `handoff-s27a.md`, `server/PROTOCOL.md` (§10 rows marked "landed").
- Steps: `git diff --stat origin/main...HEAD` against §2.1; grep the diff for `agent` under `server/` (I-a-5);
  fill §8's evidence; mark parent synchronisation pending (§11); push `mvp0/pr-s27a`; `gh pr create`; CI on the
  exact head on Linux, macOS, Windows.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S27-a close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S27c-a-1 / R-S27d-a-1 (on S27-c or S27-d, whichever lands after this PR).** Map `rate_limited` to the
  adapter's `refused { code: rate_limited }` and wait `retry_after_ms` before that handle's next submit
  (step-27 §6.5); map `not_granted` to the typed `join` refusal (§5.2 of the step).
- **R-S27d-a-2 (on S27-d).** The grant variant of the play-test's steps 2 and 9 (step-27 §11.1).
- **R-next-revision (on protocol revision 3).** "A client treats a refusal code it does not know as a generic
  refusal" (F-S27-4), recorded in `PROTOCOL.md` §10 by C1.
- **R-QS27-7 (on the per-action cadence design).** It builds on this PR's bucket in the seat table.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-27-agent-players.md` §2.2 F-S27-4 | "the Godot protocol module is the same" corrected by F-S27a-1 (codes are strings; the catalog and the 2D retry list were the real dependents) |
| `step-27-agent-players.md` §11.2 S27-a | `R`/`B` as fixed by C0 |
| `overall.md` | S27-a frozen / merged as …; `ARC-97`, `DEP-57` used |

---

## 12. Execution contract (filled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, S27-a — seat grants and a per-seat request budget
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s27a-seat-grants-budget.md
RELATED / BINDING DOCS:  step-27-agent-players.md (§§2.2, 3, 6.3–6.5, 7.2, 8, 11, 12, 13.1, 16);
                         server/PROTOCOL.md; docs/NETWORKING.md §§2, 9; docs/DECISIONS.md ARC-40, ARC-41,
                         ARC-44, ARC-67, ARC-75, DEP-14; docs/REUSE_POLICY.md; clients/protocol/ADOPTION.md;
                         step-12-server.md (S11's seat and admin design); CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s27a, its own, held by one session
BRANCH:                  mvp0/pr-s27a, created from origin/main
START CONDITION:         this design frozen, then the primary session's dispatch (step-27 §11.4)
IMPLEMENTATION BASE:     origin/main at C0; C0 records the commit
APPROVED SCOPE:          §2.1 items 1–15
FROZEN INVARIANTS:       §0's binding inputs; §6 I-a-1 … I-a-10; §8 AA-1 … AA-12; R and B as C0 fixes
                         them by the frozen rule
SEQUENCE:                C0 … C6 (§9)
ALLOWED COMMANDS:        cargo *; cargo deny; uv run --locked *; uv sync --locked; git; gh (never merge);
                         python3 scripts/*; target/*/mineworld *; godot (C0 only); mkdir -p; sed -n;
                         WebFetch for C0's crate facts only
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; edits outside APPROVED SCOPE;
                         printing or logging a grant token anywhere but its startup line; any agent-specific
                         branch in server/
MATERIAL STOPS:          a change to the client vocabulary or to the invite's meaning; raising `protocol`;
                         a need to persist any grant or bucket state; a new dependency other than governor
                         under DEP-57; R/B differing from C0's rule; a measured 2D or 3D p99 that makes R
                         exceed 50 (the budget would no longer bound abuse meaningfully — report instead)
PLATFORMS:               Linux, macOS, Windows (the PR's CI: Rust layers, the python job, the acceptance
                         crate on all three)
VALIDATION BUDGET:       unit and integration tests unrestricted locally; C0's Godot measurement ≤ 15 min;
                         CI: the PR's runs. Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s27a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized    (primary, at freeze — unresolved
                                                                           until then)
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         merge: never by the implementation session; only with the operator's explicit
                         authorization (working rules §22)
POST-MERGE SYNC OWNER:   primary session: step-27, overall.md, the decision-number table; implementation
                         session: this ledger
STOP CONDITION:          S27-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-a-1 | The budget refuses a real human at a burst (key-mashing, quick menu use) | `B = 2R ≥ 20`; measured basis (C0); the 2D trace replay test (AA-6); `rate_limited` is not final, and the next request goes through |
| R-a-2 | An old SDK in the wild meets a new code and ends its session | only reachable by a client that presents a grant or floods (§5.4 (a)); fails closed (F-S27a-5); the next revision's tolerance rule |
| R-a-3 | `governor`'s clock model fights the injected `Instant` | C0 prototype; own bucket fallback (`DEP-57`) |
| R-a-4 | A grant token leaks through a `Debug` derive on a new struct | `SeatGrant`/`GrantToken` redacted `Debug`; `SeatRights`/`Credential` carry no token; AA-8's canary scan |
| R-a-5 | The 2D measurement is a harness, not a person | the analytic bound alongside; `R` floor 10 ≈ 3 × the walking rate; QS27-7's later per-action cadence |
| R-a-6 | `client_text.rs` reads §5.5 by table rows; a reworded table breaks it | C1's review question; C2 runs it |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS27a-1 [primary — admin surface]** | `GET /admin/sessions` gains `credential: "invite" \| "grant"`. Should it also say **which** grant (e.g. its seat list, never the token), so a host who issued several can tell sessions apart? | **Yes, the seat list only**: `"credential": { "kind": "grant", "seats": ["visitor"], "take_over": false }`. It names no secret and is what a host needs to rotate the right grant; still no "agent"/"human" kind (INV-S27-4). **Ruled yes (§14.1).** |
| **QS27a-2 [primary — measurement]** | No CI runner has Godot, so C0's measurement runs on the implementer's (operator's) machine once, and CI replays the recorded trace (AA-6). Accept a one-time recorded trace as the evidence for `R`, with the 3D side INCONCLUSIVE if no timestamped 3D harness exists? | **Yes.** The trace is frozen real evidence (test rules §25); re-measure when the 3D client gains a request-timestamped harness **Ruled yes (§14.1).** |

### 14.1 Rulings, 2026-10-14 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| QS27a-1 | **Yes.** `/admin/sessions` shows the grant's seat list, never the token (§5.5 as amended) | primary session |
| QS27a-2 | **Yes.** A recorded 2D trace is the evidence for `R`; the 3D side is INCONCLUSIVE if no timestamped 3D harness exists | primary session |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | — | [ ] | [ ] | [ ] |
| C1 | — | [ ] | [ ] | [ ] |
| C2 | — | [ ] | [ ] | [ ] |
| C3 | — | [ ] | [ ] | [ ] |
| C4 | — | [ ] | [ ] | [ ] |
| C5 | — | [ ] | [ ] | [ ] |
| C6 | — | [ ] | [ ] | [ ] |

### Measurement (C0)

| Item | Value |
| --- | --- |
| 2D per-second submits p50 / p99 / max | — |
| 3D | — |
| R / B fixed | — |

### Evidence

(none yet)

### Assumptions to verify in implementation

| Id | Assumption | Verified in | If false |
| --- | --- | --- | --- |
| A-a-v1 | `governor`'s `Reference` is implemented for `std::time::Instant`, and its `Clock` bound allows a clock owned by the world thread | C0 step 6 | own bucket (`DEP-57`) |
| A-a-v2 | The 2D harness has a walking scenario long enough for a p99 (≥ 60 wall s of walking) | C0 step 2 | lengthen with the harness's `--strides=n`; record |
| A-a-v3 | The server tests can read a persisted world's journal count | C5 | use the CLI test with `mineworld inspect` |

### Mutations

(none run yet)

### Deviations and findings

(none yet; planning findings are §3.3)

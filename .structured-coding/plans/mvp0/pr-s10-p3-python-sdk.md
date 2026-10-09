# PR S10-P3 — The Python protocol SDK: typed revision-2 frames, the seat session, offers

## DESIGN FROZEN 2026-10-08 (primary session)

```text
Design revision:        §§1–9 and §11's rulings, as committed on plan/s10-next (PR #86) with this
                        header
Approved by / evidence: the primary session's freeze message of 2026-10-08, relayed by the
                        coordinator to the S10 planning session; its rulings are recorded in §11
Implementation base:    main at the start of implementation (exact commit recorded in C0)
Execution contract:     §10
Lifecycle:              FROZEN
```

**Amendment under the freeze, 2026-10-08 (operator requirement, relayed by the coordinator).** The
operator's requirement, verbatim:

> 我们要保证支持全平台，mac linux windows都可以

That is: MineWorld must support every platform, macOS, Linux and Windows. The amendment adds:

- D-P3-11 (platforms);
- AP-12;
- R-P3-9 and R-P3-10;
- platform items in C1, C3, C4 and C5;
- the matrix rule in C5;
- `astral-sh/setup-uv` in `DEP-S10-e`.

It also records two closures: the operator answered QS10-19, and the primary session answered QS10-18
(§11.1). The freeze stands, and the amendment is part of it.

Scope, invariants, decisions D-P3-1 … D-P3-11, the adversarial criteria AP-1 … AP-12 and the commit
plan are frozen. Progress, evidence, findings and bounded corrections stay writable (§12).
**Not frozen with it, and blocking nothing in P3:** QS10-18 and QS10-19 (operator-material, open), and
QS10-21 (forwarded to S11-C). See §11.
**Effort:** `mvp0` · **Step:** S10, [`step-17-cognition.md`](step-17-cognition.md) (P3 in §8 and §9;
the state of every S10 PR is in §15, "Audit, 2026-10-08") · **Parent:** [`overall.md`](overall.md) §3
(S10), coordination rulings 1, 2, 3, 6, 7 and 8 under "Parallel build-out, 2026-10-08".
**Working name:** P3. The PR number is assigned at freeze (ruling 7).
**Planning base:** `origin/main @ 9cf8f8e` (S12 13a merged; S11-A merged as #76). Branch of this design:
`plan/s10-next`.
**Decision identifiers:** placeholders (`ARC-S10-a`, `DEP-S10-b`, …). Ruling 6 gives S10 the range
`ARC-56 … ARC-60` and `DEP-24 … DEP-27`. The primary session maps the placeholders to that range at
freeze.

---

## 1. Goal, in one paragraph

A Python program can join a seat of a running MineWorld server as a client of the public protocol. It
reads typed frames and submits typed requests, with no more privilege than the 2D or 3D client has.
This PR adds `sdk/python` (`mineworld-sdk`). It holds the revision-2 frames as typed models, kept equal
to the Rust types by the golden frames S11-A committed. It holds one seat session: handshake,
newest-wins observations, submits paired with their results by token, refusals, closing and leave. It
holds the construction of requests only from what the newest observation offers. It also brings in the
repository's Python toolchain: `uv`, `ruff`, `pyright` and `pytest`, enabled in
`.structured-coding/standards.md` and run by CI. This is S10's first PR, and the only one that can
start now (step-17 §15). Every later S10 PR builds on it: the perceived stream (P3b), memory (P4),
backends (P5) and the `LMController` (P6).

## 2. Scope

### 2.1 In scope

- `sdk/python/`: `pyproject.toml`, `src/mineworld_sdk/**`, `tests/**`, a short `README.md`.
- The repository's Python workspace root and lockfile. The layout is decided in §5, D-P3-4.
- Typed models for every frame and field that `server/PROTOCOL.md` on the implementation base marks as
  landed. Optional fields follow the landing table (§10 there).
- `SeatSession`: connect, join with invite and nickname, `welcome`, observations, `submit` and
  `result`, `refused`, `closing`, `leave`.
- `offers`: request construction from the newest observation, for both a complete affordance
  (`ARC-34`) and a free-form request the observation offers by action type and target.
- Two oracles for the wire models:
  - the golden frames, `server/tests/frames/*.json`;
  - every frame a real `mineworld server` sends during the integration tests.
- A network guard: the Python suite cannot reach anything but localhost (I-11, IC-5's Python half).
- `.structured-coding/standards.md`: `ruff` and `pyright` enabled with commands, and `pytest` added.
  That file says this happens "in the same change that introduces the first Python module".
- CI: Python checks in `scripts/ci_layer.py`, `uv` in the toolchain image, and a separate `python`
  job (QP3-3, accepted with the condition that `fast` and `test` are neither renamed nor slowed; C5).
- Decision records:
  - `ARC-S10-a`, the process boundary (step-17 §3.2);
  - `DEP-S10-b` Pydantic;
  - `DEP-S10-c` `websockets`;
  - `DEP-S10-e` the Python toolchain: `uv`, `ruff`, `pyright`, `pytest`, the network guard, and for
    the non-Linux CI legs, `astral-sh/setup-uv` pinned by commit SHA.
- Every platform: Linux, macOS and Windows (operator, 2026-10-08; D-P3-11).
- The G-1 edit to `ARCHITECTURE.md` §13.1 (step-17 §12).

### 2.2 Not in scope, and where it goes

| Not here | Why | Where |
| --- | --- | --- |
| The `perceived` stream, cursor resume, `cursor_unavailable`, `lagged`; IC-1's live and resumed halves | Not on `main`. They land in S11-C (ruling 2) | **P3b**, after S11-C (step-17 §15) |
| Non-empty `observation.events`, `delta` frames, `acted_through` | S11-C. No golden frame exists to check a Python model against | S11-C, or P3b (R-S11-9, §9) |
| Reconnect with `resume` | S11-B (#83, open). P3 stores `welcome.resume` and does nothing with it | P3b, or C6 if S11-B lands during P3 |
| The `Controller` protocol and a runner that drives one | One implementation would exist (a test's echo controller). `CLAUDE.md` §4 rule 11 | P6, with `LMController` as the second (D-P3-1, QP3-1) |
| Request builders for `talk` and `move` by name | They name a System Pack's vocabulary. The SDK stays pack-agnostic, as presence's audience names no other pack (`ARC-28` point 3) | P6, in `mineworld_cognition` (D-P3-2, QP3-2) |
| Any model, backend, prompt, budget or cassette | — | P5, P6 |
| Any Rust change: server, contracts, kernel, systems, worlds | I-1. The protocol is S11's (ruling 1) | — |
| A JSON-Schema or code-generation pipeline | QS10-12: hand-mirrored now, generation when the vocabulary outgrows review | later |

### 2.3 Invariants this PR must hold

All are from step-17 §5 or follow from it.

```text
I-1    no change under kernel/, contracts/, persistence/, server/src/, systems/, worlds/
I-2    no Rust crate depends on Python; mineworld_sdk imports nothing outside itself and its declared
       dependencies
I-7    byte-identity: every observation frame, transcript and 300-day digest unchanged. Holds
       trivially because no Rust changes, and is held by the diff gate (AP-9)
I-10   no provider concept in mineworld_sdk: no provider or model name, no endpoint other than the
       MineWorld server's, no key
I-11   the Python suite passes with no network except localhost
INV-9  the SDK can say exactly join, submit and leave; it has no other frame to send
INV-13 a session delivers only frames for its own observer, and refuses to continue if a frame
       names another
```

---

## 3. Audit anchors (`origin/main @ 9cf8f8e`)

Each row was read in this planning session. The implementing session re-reads each one before
editing (working rules §3).

| Anchor | What it establishes for P3 |
| --- | --- |
| `server/src/protocol.rs` (`ClientFrame`, `ServerFrame`, `RefusalCode`, `PROTOCOL_VERSION = 2`) | The closed frame sets the models mirror. `ClientFrame` is internally tagged on `t` with `deny_unknown_fields`, so the SDK must never send an unknown field. `Refused.token` and `Refused.detail` are omitted when `None`; `Closing.detail` likewise. `Welcome.resume` is serialized as `null`, not omitted. |
| `server/src/protocol/{connection,request,summary}.rs` | `SessionId` and `TookOver` (`none`, plus later values); `ClosingReason`; `CorrelationToken` (1 to `MAX_TOKEN_LENGTH` bytes, no control characters); `WirePayload`; `WorldSummary` and `SystemSummary`; `WorldInstanceId` (32 lowercase hexadecimal characters). |
| `server/tests/frames/*.json`, eight files: `join`, `submit`, `leave`, `welcome`, `observation`, `result`, `refused`, `closing` | The reviewed oracle (SD-A12). `server/tests/frames.rs` checks server frames in both directions and client frames through `ClientFrame::decode`. Its module comment names "the Python cognition SDK" as a consumer. The observation example is minimal: every list is empty. |
| `server/PROTOCOL.md` revision 2: §2 (three client frames), §4.1 (handshake order), §5 (server frames), §7 (ids are decimal strings; every other number is an integer; a float is refused), §8 (newest observation wins), §10 (landing table) | What a conforming client must do, and which fields may be absent on today's `main`. §10: a client sends a later field (`take_over`, `perceived`) only to a server whose `PROTOCOL.md` lists it as landed. |
| `contracts/src/observation.rs` (`Observation`, `PerceivedEntity`, `PerceivedEvent`, `Affordance` with `payload`, `AffordanceFields`) | The shapes inside `observation`. An affordance's `available` must agree with `unavailable_reason`, enforced at decode: the SDK's model enforces the same agreement. `payload` is present only on a complete affordance. |
| `contracts/src/action.rs` (`ActionRequest`, `ActionRecord`, `ActionResult`, `Rejection`) | The `submit` and `result` shapes. `ActionResult` is externally tagged (`{"accepted":{"events":[…]}}`, `{"rejected":…}`, `"unavailable"`). `Rejection` is snake_case, plus `system { code, detail }`. A request's `action_type` must equal its payload record's (`ActionRequestFields`). |
| `contracts/src/spatial.rs`, `relation.rs`, `ids.rs`, `time.rs`, `component.rs` | `Location`, `LocalPosition` (`Millimetres`), `Orientation`, `SpatialRequirement`, `PlaceRequirement`, `Relation`, `ComponentRecord`, `WorldTime`, and the id newtypes. Implementing commit C2 reads each file for its exact serde attributes. |
| `server/src/app.rs` `bind` | The server binds the given address and prints the **bound** address, so `--listen 127.0.0.1:0` should report the real port. C4 verifies this before relying on it; the fallback is the Rust harness's free-port method. |
| `tools/cli/src/main.rs` `serve`; `tools/cli/src/invite.rs` | `mineworld server <world> --listen ADDR --invite TOKEN`, or `MINEWORLD_INVITE`. A given invite is never printed. Startup prints `[mineworld] listening on http://ADDR (ws://ADDR/ws), protocol 2`. |
| `tools/cli/tests/support/mod.rs` | How the Rust tests run the real binary: an ephemeral port, the invite by environment, killed on drop. The Python fixture follows the same pattern. |
| `worlds/social-cafe/world.yaml` `seats` | `visitor, wanderer, alice, bob, carol, dev`. CP-P3 uses `visitor` and `alice`. |
| `systems/conversation/src/component.rs` (`conversation-history`); golden `submit.json` (`talk` with `utterance`) | The vocabulary the **tests** use to hold a conversation. The SDK itself does not name it (D-P3-2). |
| `systems/economy/src/offer.rs`, `systems/consumption/src/offer.rs` | Where complete affordances (`payload` present) come from. They are in market-town, not social-cafe. AP-2 needs one. |
| `.structured-coding/standards.md` | `ruff` and `pyright` are declared and disabled until the first Python module. CI runs layers through `scripts/ci_layer.py` (`ARC-48`). |
| `scripts/ci_layer.py`; `Dockerfile` toolchain stage; `scripts/check_ci_pins.py` | The layers `fast` and `core`. The image installs `git`, `python3` and `ca-certificates` only: no `uv`, no Node. Every `FROM` is pinned by digest, and the pin check reads `FROM` lines only. |
| `.gitignore` | Already ignores `__pycache__/`, `*.py[cod]`, `.venv/`, `.env`, `*.env` and `secrets*`. Missing: `.pytest_cache/` and `.ruff_cache/`. |
| `tools/asset_generation/` | The only other Python in the tree: tooling, with `requirements.txt` and no `pyproject.toml`. It is not a workspace member and is not touched. |
| `docs/DECISIONS.md` `ARC-41` | JSON stays the wire encoding. `ARCHITECTURE.md` §13.1 still says Protobuf at "the first boundary Python cognition": the G-1 edit this PR makes. |
| PR #83 (S11-B, open, `7811e06`): `server/tests/frames/{join,welcome,closing}.json` | Once merged it adds `join.take_over`, `welcome.world.time_scale`, `took_over` values `hosted`/`held`/`connection`, a real `resume`, and `closing` reasons `superseded`/`taken_over`. P3 must absorb these whichever PR merges second (D-P3-5). |

**Not verified in planning; verified in C1** (an assumption that fails here goes to §7):

- `uv` supports a virtual workspace root (a `pyproject.toml` with only `[tool.uv.workspace]`).
- `uv`'s licence (dual MIT / Apache-2.0, as step-17 §4.7 states but did not verify).
- `pyright` from PyPI can run without a separately installed Node.js, through its `nodejs` extra.
- `pytest-socket` allows `127.0.0.1` and `::1` while asyncio's self-pipe works: its
  `--allow-unix-socket` option covers the Unix socketpair.
- The Python available locally (3.14.7, `overall.md` §7) and in the image (Debian trixie's 3.13)
  both satisfy `requires-python`.

---

## 4. Design

### 4.1 Package layout

```text
pyproject.toml                    the uv workspace root: no [project], members = ["sdk/python"]  (D-P3-4)
uv.lock                           one lock for every Python member, now and later (P4–P6 add theirs)
sdk/python/
  pyproject.toml                  mineworld-sdk; requires-python >= 3.12; deps: pydantic, websockets
  README.md                       human orientation, under 40 lines
  src/mineworld_sdk/
    __init__.py                   the public names, and nothing else
    py.typed
    wire/
      ids.py                      EntityId, EventId, ActionId, SessionId, EntityKey, WorldInstanceId,
                                  CorrelationToken: validated str NewTypes; JsonValue
      contract.py                 Location … Observation, Affordance, ActionRequest, ActionResult,
                                  Rejection: the contract shapes as they appear on the wire
      frames.py                   ClientFrame = Join | Submit | Leave; ServerFrame = Welcome |
                                  ObservationFrame | Result | Refused | Closing; RefusalCode,
                                  ClosingReason, TookOver, WorldSummary
      codec.py                    encode(ClientFrame) -> str; decode(str) -> ServerFrame; the
                                  per-field omission rules
    session.py                    SeatSession, Invite, JoinRefused, SessionClosed, Outcome
    offers.py                     attempt(observation, affordance); request(observation, …);
                                  NotOffered
  tests/
    conftest.py                   the network guard; the `server` fixture (real binary)
    test_golden_frames.py         AP-1, AP-3
    test_session_state.py         AP-4: the session's frame handling with frames given directly
    test_real_server.py           AP-2, AP-5, AP-6, AP-7: the real binary over real sockets
    test_network_guard.py         AP-8
```

No file is expected past about 300 lines. `contract.py` is the largest. If it passes about 400 lines,
it splits into `contract/observation.py` and `contract/action.py` along the Rust files' own lines.

### 4.2 Decisions (D-P3-1 … D-P3-10)

| ID | Decision | Reason |
| --- | --- | --- |
| **D-P3-1** | No `Controller` protocol, and no runner, in P3. They land in P6. | Step-17 §3.4.1 places `controller.py` in the SDK at P3. At P3 the only implementation would be a test's echo controller, which is an abstraction before a second implementation (`CLAUDE.md` §4 rule 11). The session itself is the seam: any controller is a loop over `observations()` and `submit()`. P6 introduces the protocol together with its second implementation. A bounded deviation from the step design, raised as QP3-1. |
| **D-P3-2** | The SDK names no System Pack's vocabulary: no `talk`, `move`, `utterance` or `conversation-history`. `offers.request(observation, action_type, target, payload)` is generic; the `Say` and `Step` builders live in `mineworld_cognition` (P6). | A generic SDK used by an RL or scripted controller must not know which packs a world installed. The Rust rule controller depends on pack crates by name; an SDK that did the same would be the reverse of `ARC-28` point 3. The tests, which are fixtures of social-cafe, may name the vocabulary. QP3-2. |
| **D-P3-3** | Models are strict. Server frames forbid unknown fields, as the Rust client frames do. Ids are accepted **only** as decimal strings, never JSON numbers. Every other integer is a strict `int`, so a float is refused. Enumerations are closed. | `PROTOCOL.md` §7 says a client keeps ids as strings, and §2 makes unknown fields loud on the server side. The SDK lives in the same repository and CI as the server, so drift fails a test rather than a user. Forward compatibility across repositories is not a requirement before a public stable contract exists (`CLAUDE.md` §4 rule 12). |
| **D-P3-4** | One uv workspace at the repository root (a virtual root: `[tool.uv.workspace]` only), with one `uv.lock`. `sdk/python` is its first member. | P4–P6 add `cognition/lm-controller` as a second member, which depends on `mineworld-sdk` through the workspace. One lock means one resolution and no drift between the two packages. The alternative, a lock per package with a path dependency, gives two resolutions of `pydantic`. QP3-4. |
| **D-P3-5** | Absorbing S11-B: whichever of P3 and S11-B (#83) merges second updates the SDK models and the golden test. If #83 is merged at P3's implementation base, C2 includes its fields and C6 is N/A. If it merges during P3, C6 absorbs it. If P3 merges first, #83's rebase updates `sdk/python` (R-S11-9, §9). | Rule from ruling 1: one owner per interface, and the far side is verified (`overall.md` R-9). |
| **D-P3-6** | `observation.events` is modelled as **empty-only** until S11-C. A non-empty list raises `UnsupportedFrame("observation.events: arrives with S11-C; this SDK predates it")`. | No golden frame and no real frame carries an event envelope on the wire today, so a Python model of `EventEnvelope` would be written against nothing (working rules §25). Failing loudly is honest. P3b, or S11-C under R-S11-9, replaces it with the real model and a golden file. |
| **D-P3-7** | `payload` values (component payloads, affordance payloads, request payloads) are typed `JsonValue`, a recursive alias of JSON's types. They are never `Any` or `dict[str, Any]`. | That is the payload-erasure boundary the Rust side chose itself: `WireObservation = Observation<serde_json::Value>`. The owning pack's JSON is interpreted only by code that knows the pack (`CORE_CONCEPTS.md` §15). |
| **D-P3-8** | `offers` refuses locally, before anything is sent, a request the newest observation does not offer as **available** for that action type and target. For a complete affordance, it resubmits the affordance's `payload` unchanged. It never computes a world rule. | Step-17 §3.6 layer 2, which reads the server's verdict and computes nothing. The world remains the authority: a raw `submit` of anything is still validated by the server, and AP-6 shows both halves. |
| **D-P3-9** | The invite is an `Invite` object whose `repr` and `str` are `Invite(<redacted>)`. It is built by the caller, from a literal in tests or from an environment variable the caller names. The SDK never reads the environment itself, never logs the invite, and never puts it in an exception message. | R-S11-5 and step-17 I-16: no secret in any artefact. The server does the same with `OfferedInvite`'s redacted `Debug`. |
| **D-P3-10** | The integration tests **fail**, they do not skip, when the server binary is missing. The binary is found from `MINEWORLD_BIN`, or else `target/debug/mineworld` under the repository root, with the platform's executable suffix (`mineworld.exe` on Windows). The failure message names the build command. A CI job that deliberately does not build the binary, such as a Windows smoke job (C5), deselects these tests by the `real_server` marker on its command line. That deselection is visible in the layer's command and is never a skip inside the test. | A skipped test that reads as green is a test that did not run (`CLAUDE.md` §3.1: "a test not run is not a pass"). |
| **D-P3-11** | **Every platform** (operator requirement, 2026-10-08): the SDK, the uv workspace, the static checks and the tests work on Linux, macOS and Windows. Concretely:<br>(a) Paths are built with `pathlib`, never with string separators. The executable suffix comes from the platform.<br>(b) The network guard runs `pytest-socket` with `--allow-hosts=127.0.0.1,::1` on every platform. `--disable-socket --allow-unix-socket` is added only where asyncio's self-pipe is a Unix socket. On Windows, asyncio's `socketpair()` is a loopback TCP pair, which `--disable-socket` would block. C3 settles the exact flags on all three platforms, and AP-8 holds on each.<br>(c) The server process is stopped with `Popen.kill()`, which is `SIGKILL` on POSIX and `TerminateProcess` on Windows. No test sends a POSIX signal by name.<br>(d) Files are read as UTF-8 explicitly. Golden frames are compared as parsed JSON, never as text, so CRLF checkouts are harmless.<br>(e) No shell-specific commands: tests start the binary with argument lists, never through a shell. | `CLAUDE.md` §1.1: the reference clients and the cognition process run on players' own machines. A Python SDK that only works on POSIX would make Windows players second-class. |

### 4.3 The session, precisely

```text
SeatSession.connect(url, seat, invite, nickname) -> SeatSession     (an async context manager)
  1. open the WebSocket (websockets' asyncio client)
  2. send join { protocol: 2, invite, nickname, seat, resume: null }
     + after S11-B lands: take_over (default false)
  3. await the first server frame:
       welcome   → protocol must be 2, or close and raise ProtocolMismatch;
                   keep observer, world (instance), resume, hold_seconds, took_over
       refused   → raise JoinRefused(code). For protocol_mismatch and unauthorized the server
                   follows with closing; for any other code the session closes the socket itself
       closing   → raise JoinRefused(reason)
  4. a reader task routes every later frame:
       observation → its observer must equal welcome.observer, or the session closes and raises
                     ForeignObserver (INV-13, AP-5). Otherwise it replaces `newest`, and waiters
                     on `changed()` are woken. Frames are never queued: §8, newest wins
       result      → resolves the pending submit with the same token; an unknown token is a
                     protocol error, and the session closes
       refused     → with a token: resolves that submit as Refused(code). Without one: a protocol
                     error, raised from the next await
       closing     → every pending submit fails with SessionClosed(reason); the session ends
submit(request) -> Outcome     a fresh token per session ("c1", "c2", …); sends submit; awaits the
                               paired answer. Outcome = Answered(action_id, ActionResult) | Refused(code)
leave()                        sends leave and awaits closing { left }
```

The session does no reconnecting and no retrying. A dropped socket ends it with
`SessionClosed(None)`. Reconnect belongs with `resume` (P3b, or C6).

---

## 5. Test ownership (test rules §26)

```text
STATIC     ruff (format, lint, imports); pyright strict (typed calls, NewType mixing such as passing an
           EventId where an EntityId is expected, protocol misuse). No unit test re-proves these
UNIT       codec against the golden files (AP-1); id fidelity above 2^53 (AP-3); the session's frame
           routing with frames given directly (AP-4); the affordance agreement rule; the empty-only
           events guard; the offers refusal (the local half of AP-6)
GATE 1     NOT REQUIRED. No language model is involved and no model-facing text exists
           (CLAUDE.md §5)
GATE 2     the real `mineworld` binary over real sockets: handshake, a two-seat conversation, every
           received frame round-tripped (AP-2, AP-5, the world half of AP-6, AP-7). These run in
           pytest against a process the fixture starts, so CI runs them
CI IMPACT  the new Python checks (C5); the Rust layers unchanged. The Rust suite is not re-run for
           evidence beyond the diff gate, because no Rust file changes (AP-9)
```

---

## 6. Adversarial criteria (fixed now, before anything is measured)

Bounds are literals taken from the requirement, never from the implementation (`ARC-23`). Each
criterion names the mutation that must turn it red. A criterion whose mutation stays green has tested
nothing.

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| **AP-1** Golden frames | For each of the eight files in `server/tests/frames/`, checked as the Rust test checks them. A server frame decodes and re-encodes to a JSON value equal to the file. A client frame, built from its model, encodes equal to the file. **Completeness:** every `*.json` in the directory has a model, so a ninth file fails the test by name. | (a) Rename `hold_seconds` in the `Welcome` model: `welcome.json` fails. (b) Encode `Refused.token` as `null` instead of omitting it: `refused.json` fails. (c) Delete the `Leave` model: completeness fails, naming `leave.json`. |
| **AP-2** Real-frame fidelity | During AP-5's run, and one market-town run, every frame received is kept raw. For each, `encode(decode(raw))` equals `raw` as a JSON value. Coverage is fixed now: at least 1 `welcome`, 50 `observation` and 2 `result` frames per session; at least one observation with an entity that has a component; one with an affordance and its `requirement`; one with a non-empty `relations`; and one with a **complete** affordance (`payload`). C4 picks, by reading the pack sources and `tools/cli/tests/market_town.rs`, a seat and position that offer one. It records the choice before running. | The `Observation` model drops `relations`: the round trip differs at `observation.relations`, and the test names that path. |
| **AP-3** Ids are strings | One observation holding entity ids `"9007199254740995"` and `"9007199254740997"` decodes to two distinct ids and re-encodes to the same two strings. A JSON **number** in an id position is refused at decode. | Coerce ids through `int` or `float`: the two ids collapse, or the number is accepted. |
| **AP-4** Token pairing | Unit. Three submits are pending, and their `result` frames are given to the session in reverse order: each caller receives its own `action_id`. A `refused` carrying the second token resolves only the second. A `closing` fails the remaining submits with `SessionClosed(reason)`. | Pair answers first-in-first-out instead of by token: callers receive one another's results. |
| **AP-5** Two seats, one conversation, real binary | `mineworld server worlds/social-cafe` (in memory), two `SeatSession`s: `visitor` and `alice`. The test's echo loop on `alice` answers any newly heard line through `offers.request` (`talk` to the speaker). `visitor` submits `talk` to Alice. Within **10 wall seconds**: `visitor`'s result is `accepted` with at least one event id; `visitor`'s own observation discloses Alice's reply; every frame either session received names its own observer. | Make the echo loop submit with `visitor`'s entity as actor: the server answers `actor_not_observer`, and the reply step fails by name. |
| **AP-6** Offers read the verdict; the world still decides | `offers.request(obs, "shoot", target=bob, payload={})` raises `NotOffered` and **no frame is sent**: the session's sent-frame count is unchanged. The same request sent raw with `session.submit` is answered `ActionResult.unavailable` by the world (`INV-10`, seen from the far side). | Remove the local check: the sent count rises, so the first half fails while the second still passes. That shows the world's check is the one that holds. |
| **AP-7** Secrets | A join with a wrong invite raises `JoinRefused("unauthorized")` no sooner than **500 ms** after the join was sent (the server's fixed delay, `PROTOCOL.md` §4.1). The invite string appears in no captured log record, in no exception's `str` or `repr`, and in no session's `repr`. Checked with a 32-character marker invite. | Put the offered invite in `JoinRefused`'s message: the scan finds it. |
| **AP-8** No network but localhost | The whole suite runs with the guard on. A test connecting to `192.0.2.1:80` (TEST-NET-1, RFC 5737) fails within **1 s** with the guard's own error type, not a timeout and not `OSError`. | Disable the guard: the test fails because the error type or the elapsed time is wrong. |
| **AP-9** Scope | `git diff --stat <base>..HEAD` touches only: `pyproject.toml`, `uv.lock`, `sdk/python/**`, `.gitignore`, `.structured-coding/**`, `docs/DECISIONS.md`, `docs/ARCHITECTURE.md`, and, with C5, `scripts/ci_layer.py`, `Dockerfile`, `scripts/check_ci_pins.py`, `.github/workflows/ci.yml` (one new job only). No `*.rs` file appears. | — (a diff gate; review plants nothing) |
| **AP-10** Static | `ruff check`, `ruff format --check` and `pyright` (strict, over `sdk/python`) report zero findings. | Introduce `def f(x): return x`: pyright strict reports a missing annotation. |
| **AP-11** Fail, never skip | With `MINEWORLD_BIN=/nonexistent`, the integration tests **error**, and the message names `cargo build -p mineworld-cli`. Their count in the pytest summary is "error", never "skipped". | Make the fixture `pytest.skip`: the summary shows "skipped". |
| **AP-12** Every platform | The full suite (AP-1 … AP-11) passes on Linux, locally and in CI's `python` job. On Windows and macOS it passes in CI, as the full suite if C5's matrix rule admits that platform, and otherwise as the smoke selection. The smoke selection is `-m "not real_server"` plus the static checks, so AP-1, AP-3, AP-4, AP-8 and AP-10 still hold there. AP-8's guard is shown red on Windows too, by the same mutation. | Hard-code `target/debug/mineworld` without the platform suffix: AP-11's message test fails on Windows. Or hard-code `--disable-socket` for every platform: asyncio cannot create its loop on Windows, and the Windows job is red. |

---

## 7. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-P3-1 | `pyright` needs Node.js. The toolchain image has none, and `pip install pyright` downloads Node at first run, which is network at check time. | Verified in C1: the `pyright[nodejs]` extra, which installs Node from a wheel and is locked by `uv.lock`. If it fails, record a deviation: either `basedpyright` (bundles Node through `nodejs-wheel`) with the same strict configuration, or Node pinned in the image. Either way, an entry in `DEP-S10-e`. |
| R-P3-2 | `uv` in the CI image: a new binary in the toolchain, and PyPI access during CI. | Copy `uv` from its official image pinned by **digest** (`COPY --from=ghcr.io/astral-sh/uv:<version>@sha256:…`). Extend `check_ci_pins.py` so a `COPY --from=<image>` also needs a digest. Run `uv sync --locked`, so the lock is the whole resolution. Set `UV_PYTHON_DOWNLOADS=never`, so the image's Python 3.13 is used and nothing else is downloaded. Python runs in a separate `python` job, so the required `fast` and `test` checks are not slowed (QP3-3's ruling). |
| R-P3-3 | S11-B (#83) and S11-C change golden frames while P3 is open. | D-P3-5 and R-S11-9. The completeness check (AP-1) makes the drift visible in whichever PR merges second. |
| R-P3-4 | The re-encoding of optional fields drifts from serde's. `Refused.token` is omitted when `None`; `Welcome.resume` is written as `null`; `Affordance.payload` is omitted when absent. | Per-field omission rules in `codec.py`, held by AP-1 and AP-2. A blanket `exclude_none` is wrong and AP-1 (b) catches it. |
| R-P3-5 | A two-seat test that depends on timing flakes. | Waits are bounded and event-driven, through `changed()`. The 10 s bound is a ceiling, not a sleep. The fixture's server is in memory, so there is no save I/O. |
| R-P3-6 | `websockets` changed its client API (the legacy and new asyncio implementations). | Pin a major version in `pyproject.toml` and use only `websockets.asyncio.client.connect`. DEP-S10-c records the version. |
| R-P3-7 | A Python package in a Rust repository rots: unpinned tools, unrun checks. | Lock, strict static checks, and CI (C5). Without C5, I-11 is a review promise (QS10-16). This is why QP3-3 recommends C5. |
| R-P3-8 | `requires-python` excludes a supported environment. | `>=3.12`. CI runs the image's 3.13; the operator's machine runs 3.14.7. Both are exercised before review: the local run and the CI run. |
| R-P3-9 | Windows-specific failures: asyncio's proactor loop, `pytest-socket`'s flags, process termination, path suffixes, and pyright's Node wheel on Windows. | D-P3-11, AP-12, and a Windows job in CI (C5). C1 verifies that `uv sync --locked` and `pyright[nodejs]` resolve for `win_amd64` and `macosx_arm64` as well as Linux: `uv.lock` is universal, so the lock itself shows it. |
| R-P3-10 | Windows and macOS runners cannot use the Linux toolchain container (`DEP-17`, `ARC-48`), so those jobs differ from Linux CI's environment. They are also slower, and they cost more minutes. | The non-Linux jobs pin what they install: the Rust toolchain from `rust-toolchain.toml` through `rustup`, `uv` through `astral-sh/setup-uv` pinned by commit SHA (`DEP-S10-e`), and Python from `requires-python` through `uv`. They still call only `ci_layer.py` layers. An `ARC-48` note records the departure from "inside the container" for these jobs only, because the operator's platform requirement needs it. C5's measured rule decides whether each runs the full suite or a smoke selection. |

---

## 8. Commit plan

Each commit tracks **implementation**, **validation** and **review** separately. `[x]` needs the work
and the evidence. An item that turns out not to apply is `N/A`, with the audited reason. Commands
assume the worktree root; `uv` and `cargo` are on `PATH` (`$HOME/.cargo/bin/cargo` otherwise).

### C0 — Freeze and contract (Markdown only)

- **Goal.** Record the primary session's freeze, and fill the execution contract (§10) with its
  sources.
- **Scope.** This document, and a new `handoff-s10-p3.md`. No code.
- [x] Implementation: the `DESIGN FROZEN` header with the operator or primary-session reference; the
  contract's endpoint authority lines with sources; the handoff initialized. (§12.1)
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  (E-P3-0)
- [x] Review: every endpoint line has a source, and none was narrowed or widened without one
  (working rules, contract block). (§12.1)
- **Commit boundary.** Documentation only.

### C1 — Specs and toolchain before code

- **Goal.** The decisions exist before the code they govern (`CLAUDE.md` §2.2), and the empty package
  builds, locks, lints and type-checks.
- **Scope.**
  - root `pyproject.toml` (virtual workspace) and `uv.lock`;
  - `sdk/python/pyproject.toml` with:
    - dependencies `pydantic` (2.x) and `websockets` (major version pinned);
    - dev group `pytest`, `pytest-socket`, `ruff`, `pyright[nodejs]`;
    - `[tool.ruff]` with the rule set chosen in C1 (at least `E`, `F`, `I`, `UP`, `B`, `ANN`);
    - `[tool.pyright]` with `typeCheckingMode = "strict"`;
  - `src/mineworld_sdk/__init__.py`, `py.typed`, `README.md`;
  - `.gitignore`: `.pytest_cache/` and `.ruff_cache/`;
  - `.structured-coding/standards.md`:
    - `ruff` enabled, with `command` `["uv","run","--locked","ruff","check","sdk/python"]`, plus a
      `ruff-format` entry;
    - `pyright` enabled, with `["uv","run","--locked","pyright","sdk/python"]`;
    - `pytest` added, with `["uv","run","--locked","pytest","sdk/python"]`;
    - the prose "Current state of the checks" updated;
  - `docs/DECISIONS.md`: `ARC-S10-a`, `DEP-S10-b`, `DEP-S10-c`, `DEP-S10-e`. Placeholders, unless the
    primary session has mapped them;
  - `docs/ARCHITECTURE.md` §13.1, the G-1 edit:
    - the first cross-language boundary was the JSON client protocol (Godot, then Python);
    - Python mirrors it as typed models held to the golden frames;
    - Protobuf and gRPC are declined until an encoding need is measured (`ARC-41`);
  - the §13 table row "Contracts" updated to match.
- **Non-goals.** No model and no session code. No CI change (C5).
- [x] Implementation: the files above. Verify and record the five "not verified in planning" items
  of §3; each failure is recorded with its fallback. (§12.1b)
- [x] Validation (E-P3-1):
  - `uv lock` then `uv sync --locked` succeed;
  - `uv run --locked ruff check sdk/python`, `ruff format --check sdk/python` and `pyright sdk/python`
    report zero findings (AP-10 baseline);
  - the planted `def f(x): return x` mutation is red under pyright, then removed (AP-10 mutation);
  - both doc checks pass;
  - `python3 .claude/skills/structured-coding/scripts/standards.py inspect --project .` lists the
    three new checks as enabled;
  - D-P3-11 / R-P3-9: `uv.lock` holds wheels for `win_amd64`, `macosx_*_arm64` and Linux for every
    package that ships wheels. This is read from the lock, and `pyright[nodejs]`'s Node wheel is
    checked for each of the three.
- [x] Review (§12.1b):
  - each DECISIONS entry states what it adopts, the alternatives from step-17 §4 with their
    verdicts, and a re-evaluation trigger (`REUSE_POLICY.md` §§11–12);
  - the §13.1 edit contradicts neither `ARC-41` nor D-4's direction;
  - no provider concept appears anywhere (I-10).
- **Failure cases.** `uv` cannot do a virtual root: fall back to `sdk/python` as the root, and record
  a deviation that names P4's migration. `pyright[nodejs]` does not work: R-P3-1.

### C2 — Wire models and codec, held to the golden frames

- **Goal.** Every landed revision-2 frame is a typed Python value, and its encoding is the Rust
  encoding.
- **Scope.**
  - `wire/ids.py`, `wire/contract.py`, `wire/frames.py`, `wire/codec.py`;
  - `tests/test_golden_frames.py`.
  - If S11-B has merged at the base, its fields too (D-P3-5).
- **Non-goals.** Event envelopes: empty-only (D-P3-6). `delta`. `perceived`.
- [x] Implementation (§12.1c):
  - read every contract file in §3 for its serde attributes;
  - id `NewType`s: decimal-string validation for ids; the contract's key rule for `EntityKey`;
    32-character lowercase hexadecimal for `WorldInstanceId`; the length and printable rule for
    `CorrelationToken`;
  - contract models, with the affordance agreement validator and the request `action_type`
    agreement validator;
  - frame unions, discriminated on `t`;
  - `codec.encode` and `codec.decode` with per-field omission (R-P3-4);
  - `UnsupportedFrame` for non-empty `events`.
- [x] Validation (E-P3-2; M-1 … M-6; M-3's owner moved to AP-2, §12.4):
  - AP-1 (the eight files and completeness), with the three mutations (a), (b) and (c) run and seen
    red, then reverted;
  - AP-3, with its mutation;
  - a unit test for each validator: the affordance agreement and the request `action_type`
    agreement. They are cross-field rules, not library behaviour;
  - the `UnsupportedFrame` guard;
  - ruff and pyright clean.
- [x] Review (§12.1c):
  - compare every model field with its Rust struct line by line, and list in the ledger any field
    whose optionality differs from serde's;
  - the landing table: no field from an unlanded PR is sent;
  - `JsonValue` appears only at payload positions (D-P3-7).
- **Commit boundary.** Wire layer only. No socket code.

### C3 — The seat session and offers

- **Goal.** One seat, driven from Python, with no privilege a client lacks.
- **Scope.**
  - `session.py`, `offers.py`;
  - `tests/test_session_state.py`;
  - `tests/conftest.py` (the network guard only).
- [ ] Implementation:
  - `SeatSession` as specified in §4.3: the handshake, the reader task, newest-wins `observation()`
    and `changed()`, token allocation, `submit`, `leave`, closing;
  - `Invite`, redacted (D-P3-9);
  - `offers.attempt` and `offers.request` with `NotOffered` (D-P3-8);
  - the guard: `pytest-socket` with allow-hosts `127.0.0.1,::1` on every platform, and
    `--disable-socket --allow-unix-socket` only where asyncio's self-pipe is a Unix socket. It is
    configured in `conftest.py` by platform, so no test can forget it (D-P3-11 (b));
  - the `real_server` marker registered in `pyproject.toml` (D-P3-10).
- [ ] Validation:
  - AP-4 with its FIFO mutation;
  - the local half of AP-6: the sent-frame count is unchanged on `NotOffered`;
  - AP-8 with its mutation;
  - the session's error paths, as units: `welcome.protocol` 3 gives `ProtocolMismatch`; a foreign
    observer gives `ForeignObserver`; an unknown result token is a protocol error.
- [ ] Review:
  - no frame other than join, submit and leave can be produced (INV-9): the encoder's input type is
    the closed union;
  - nothing in the session reads the environment or prints;
  - no busy loop and no unbounded queue.
- **Commit boundary.** No test touches a real server yet.

### C4 — The real binary: the integration checkpoint

- **Goal.** CP-P3: Python is a client of the real server, end to end.
- **Scope.**
  - `tests/conftest.py`: the `server` fixture, which starts `mineworld server <world> --listen
    127.0.0.1:0` with `MINEWORLD_INVITE` set to a test marker, reads the bound address from the
    `listening on` line, and kills the process at teardown (D-P3-10);
  - `tests/test_real_server.py`.
- [ ] Implementation:
  - verify that `--listen 127.0.0.1:0` reports the real port, or use the free-port fallback (§3);
  - record AP-2's market-town seat and position before the first run, chosen from the pack sources;
  - the fixture, and the tests for AP-2, AP-5, the world half of AP-6, AP-7 and AP-11.
- [ ] Validation:
  - `cargo build -p mineworld-cli`, then `uv run --locked pytest sdk/python`, with wall time and
    counts recorded;
  - AP-5's mutation (wrong actor), AP-7's mutation (the invite in the message) and AP-11's mutation
    (skip), each run and seen red, then reverted;
  - AP-2's coverage counts recorded;
  - the fixture's binary path and process stop follow D-P3-11 (a) and (c). They are proved on
    Windows and macOS by C5's jobs (AP-12).
- [ ] Review:
  - every wait is bounded;
  - the fixture removes nothing it did not create (an in-memory world writes no scratch);
  - the test vocabulary (`talk`, `conversation-history`) stays in `tests/`, never in `src/` (D-P3-2).
- **Failure cases.**
  - The server does not start within 30 s: the fixture fails with the server's stderr.
  - A port collision: retried a bounded number of times, as the Rust harness does.

### C5 — CI runs the Python checks (QP3-3 accepted, with the primary session's condition)

- **Goal.** I-11 and AP-10 become CI gates, not review promises (QS10-16, R-S13-1).
- **The primary session's condition (binding).** The required checks `fast` and `test` keep their
  names and do not get slower.
  - The Python **static** checks (`uv sync --locked`, `ruff check`, `ruff format --check`, `pyright`)
    go in `fast` only if they add **under about 60 s** to `fast`'s wall time, measured on a CI run.
    Otherwise they go in the separate job below.
  - The Python **tests** (`pytest`, which needs a built `mineworld` binary) never go in `core`.
    `core` is the `test` job, and adding them would slow it.
  - A separate job is not required by branch protection until the primary session adds it. C5 does
    not touch branch protection.
- **Scope.**
  - `Dockerfile` toolchain stage: `COPY --from=ghcr.io/astral-sh/uv:<version>@sha256:<digest>
    /uv /uvx /bin/`, and `ENV UV_PYTHON_DOWNLOADS=never`;
  - `scripts/check_ci_pins.py`: a `COPY --from=` image must carry a digest;
  - `scripts/ci_layer.py`:
    - a new layer `python`: `uv sync --locked`, the static checks (unless they moved to `fast`),
      `cargo build -p mineworld-cli`, `uv run --locked pytest sdk/python`, and the scratch check;
    - `fast` gains the static checks **only** under the 60 s condition above;
    - `ENVIRONMENT` gains `uv --version`, so the log records it;
    - a command that runs a repository script names the interpreter through `sys.executable`, not
      the literal `python3`, which Windows does not have. Existing layers are unchanged;
  - `.github/workflows/ci.yml`: one new job `python`, which, like the others, only names its layer
    (I-S13-9 holds: no command appears in YAML). The `fast` and `test` jobs are unchanged;
  - **every platform** (operator requirement, D-P3-11). The `python` job runs on `ubuntu-24.04`
    inside the toolchain container. It becomes a matrix over `ubuntu-24.04`, `windows-latest` (the
    exact image is pinned in C5) and `macos-latest`. A non-Linux leg runs without the container
    (R-P3-10) and installs:
    - the toolchain from `rust-toolchain.toml` through `rustup`;
    - `uv` through `astral-sh/setup-uv` pinned by commit SHA;
    - then calls `python scripts/ci_layer.py <layer>`, where `python` is the interpreter uv
      provides;
  - **the matrix rule** (coordinator, 2026-10-08):
    - a leg runs the **full** `python` layer if it adds **under about 3 minutes** of wall time over
      the Ubuntu leg, measured on a CI run;
    - otherwise only Windows gets a **smoke** leg: a layer `python-smoke`, which runs
      `uv sync --locked`, the static checks and `pytest -m "not real_server"`, with no Rust build;
    - the macOS leg is then dropped, with the measurement recorded;
  - an `ARC-48` note recording the new layer(s), their jobs, and the non-container legs.
- [ ] Implementation: as above. First measure, in one CI run:
  - the static checks' wall time in the `python` layer;
  - each platform leg's wall time.

  Then place the static checks by the 60 s rule and size the matrix by the 3-minute rule. Record
  every measurement and placement in the ledger before the decisive run.
- [ ] Validation:
  - `python3 scripts/check_ci_pins.py` passes, and fails on a planted digest-less `COPY --from`;
  - `python3 scripts/ci_layer.py --list fast`, `--list core` and `--list python` show the intended
    commands, and `core`'s list is unchanged;
  - push a scratch branch `scratch/s10-p3-mutation` with AP-8's guard disabled, see the `python`
    job red on every leg (Windows included), then green on the PR head (step-14's R-4 practice);
  - AP-12: each kept leg green on the PR head, with its selection (full or smoke) named in the
    ledger;
  - on the PR head, `fast` and `test` are green, and their wall times are compared with the base's.
    `fast` may grow by the static checks only within the 60 s rule; `test` must not grow beyond run
    noise.
- [ ] Review: the S13 owner's rules hold: the layers are the only command list, every image is
  pinned by digest, required checks keep their names, and nothing in CI can reach a model.

### C6 — Absorb S11-B (conditional on D-P3-5)

- **Goal.** The SDK speaks the revision-2 frames as landed on `main` when P3 merges.
- **Scope.**
  - `join.take_over` (default `false`, sent only once landed);
  - `WorldSummary.time_scale`;
  - `TookOver` values `hosted`, `held`, `connection`;
  - `ClosingReason` values `superseded`, `taken_over`;
  - the `seat_occupied` refusal;
  - a non-null `resume`, stored but not used.
- [ ] Implementation: models and the golden test follow #83's files. The session raises
  `JoinRefused("seat_occupied")`.
- [ ] Validation: AP-1 over the updated files; AP-2 and AP-5 re-run on the merged base.
- [ ] Review: no reconnect policy was added (it is P3b's).
- **N/A** when #83 is at the base (C2 covers it) or still unmerged at review. In the second case,
  R-S11-9 moves the obligation to #83.

### C7 — Close out

- [ ] Implementation:
  - `sdk/python/README.md`: what the SDK is, the one-screen example, and a link to `PROTOCOL.md`;
  - the ledger (§12) with all evidence;
  - the handoff closed.
- [ ] Validation:
  - the full Python suite and the static checks on the final head;
  - `cargo test --workspace` **NOT RUN for evidence**, because no Rust changed: AP-9 holds it, and CI
    runs it anyway;
  - both doc checks.
- [ ] Review: the whole diff against §2.3's invariants; every `[x]` has its evidence; deviations
  recorded.
- **Stop:** `READY FOR OPERATOR REVIEW — DO NOT MERGE`.

---

## 9. Requirements this PR places on other steps

- **R-S11-9 (new; on S11-B and S11-C), for the primary session.** Once P3 has merged, an S11 PR that
  adds or changes a file under `server/tests/frames/` updates `sdk/python`'s models in the same PR. The
  Python suite's AP-1 fails until it does. This applies the far-side rule (`overall.md` R-9) that
  already makes S11 PRs update the GDScript module. For S11-C it means its `perceived`, `delta` and
  event-bearing `observation` golden frames arrive with Python models. P3b then adds only the stream
  semantics (cursor, resume, `lagged`) and the session's use of them.
- **R-S11-10 (on S11-C), restating step-17 R-S11-7 for the new frames.** S11-C commits golden files for:
  - `perceived`;
  - `delta`, if CP-C1 keeps deltas;
  - an `observation` whose `events` is non-empty and holds at least one `Place`-visible and one
    `Participants`-visible envelope;
  - the `join` carrying `perceived`;
  - the refusals `cursor_unavailable` and `lagged`.

  Without them the Python event model would have no oracle (D-P3-6).

## 10. Execution contract (filled at freeze, 2026-10-08)

Source of every line marked "primary session": its freeze message of 2026-10-08, relayed by the
coordinator to the S10 planning session.

```text
PROJECT / PR:            MineWorld mvp0, S10 PR P3 — the Python protocol SDK
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p3-python-sdk.md
RELATED / BINDING DOCS:  step-17-cognition.md (§3.2, §3.4, §4, §5, §15); overall.md (S10; rulings
                         1–8); server/PROTOCOL.md rev 2; docs/ARCHITECTURE.md; docs/MODULE_SPEC.md §5;
                         docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s10-p3          (primary session)
BRANCH:                  mvp0/pr-s10-p3-sdk, created from main                     (primary session)
IMPLEMENTATION BASE:     origin/main at the start of implementation; C0 records the exact commit and
                         whether #83 (S11-B) is in it (D-P3-5)
APPROVED SCOPE:          §2.1, as frozen
FROZEN INVARIANTS:       §2.3; D-P3-1 … D-P3-11 (D-P3-11: every platform, operator 2026-10-08);
                         §11.1's rulings, including QP3-3's CI condition and C5's matrix rule
SEQUENCE:                C0 … C7 (C5 under QP3-3's condition; C6 conditional on D-P3-5)
ALLOWED COMMANDS:        cargo *; git; gh (never merge); uv * (local tool ~/.local/bin/uv);
                         python3 scripts/*; npx pyright* (only through the project's uv/npm
                         environment); mkdir -p; sed -n; target/*/mineworld *   (primary session)
NEVER:                   python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading
                         ~/.config/mineworld/secrets.env                            (primary session)
MATERIAL STOPS:          any Rust server or protocol change; any dependency beyond the DEP records
                         this design names (DEP-S10-b Pydantic, DEP-S10-c websockets, DEP-S10-e uv,
                         ruff, pyright, pytest, pytest-socket, and for CI astral-sh/setup-uv pinned
                         by SHA); any hosted-API use                                (primary session)
PLATFORMS:               Linux, macOS, Windows (operator, 2026-10-08; D-P3-11, AP-12)
VALIDATION BUDGET:       unit, static and local integration: unrestricted. Real LLM calls: none
                         (Gate 1 NOT REQUIRED). No paid API, no key, no model. CI: C5's measuring
                         run and its scratch mutation branch, once each
LIVE DOCUMENTATION:      this document (§12 ledger)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s10-p3.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized   (primary session freeze)
                         semantic commits: authorized                    (primary session freeze)
                         branch push: authorized                         (primary session freeze)
                         PR creation / update: authorized                (primary session freeze)
                         CI repair to review readiness: authorized       (primary session freeze)
                         merge: explicit operator authorization only
POST-MERGE SYNC OWNER:   the S10 planning session owns step-17 §15 and overall.md; the
                         implementation session owns this document, the merge identity, the
                         evidence and the deviations (the workflow's default)
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 11. Questions

Recommendations are given for each. **[operator]** marks a question that only the operator can
answer: provider choice, a paid API, scope, or a reversal of an operator ruling. **[primary]** marks one
the primary session decides.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QP3-1 [primary]** | Defer the `Controller` protocol and runner from P3 to P6 (D-P3-1), a deviation from step-17 §3.4.1? | **Yes.** At P3 it would have one implementation, a test's. P6 brings the second. The session is already the seam. |
| **QP3-2 [primary]** | Keep pack vocabulary (`talk`, `move`) out of `mineworld_sdk`, moving `Say` and `Step` to `mineworld_cognition` (D-P3-2)? | **Yes.** An SDK for any controller must not assume a world's packs. `offers.request` is generic and keeps the same local verdict check. |
| **QP3-3 [primary, with S13's owner]** | Does P3 wire Python into CI (C5: `uv` in the image, Python commands in the layers), or does S13 do it (R-S13-1, QS10-16)? | **P3 does, in C5.** `standards.md` ties enabling the checks to the first Python module. No S13 PR plans it (step-14 audit: no Python job). Without it I-11 and IC-5 are promises. The change follows S13's own rules: layers only, digest pins. |
| **QP3-4 [primary]** | A uv workspace at the repository root with one lock (D-P3-4), or a lock per package? | **Root workspace, one lock.** P4–P6 add `cognition/lm-controller` as a member. Two locks would resolve shared dependencies twice. |
| **QP3-5 [primary]** | Freeze and start P3 before S11-B (#83) merges? | **Yes.** P3 needs only S11-A. D-P3-5 and R-S11-9 make the order safe either way, and the absorb step is about six fields. |
| **QP3-6 [primary]** | The network guard: `pytest-socket`, or a guard of our own in `conftest.py`? | **`pytest-socket`** (MIT, small, purpose-built), recorded in `DEP-S10-e`. Our own guard would re-implement socket patching (`REUSE_POLICY.md` §12). C1 verifies it works with asyncio (§3). |
| **QP3-7 [primary]** | Pydantic for the wire models, or `msgspec` or dataclasses? | **Pydantic** (`DEP-S10-b`, step-17 §4.2). The same library generates P6's structured-output schema, and pyright understands it. `msgspec` is faster, but speed is not a constraint at 10 Hz per seat, and it would be a second schema library later. |

No P3 question is operator-material. The operator-material S10 questions belong to later PRs. They are
gathered in step-17 §15.6 so that the operator sees them in one place.

### 11.1 Rulings, 2026-10-08 (primary session, at freeze)

| ID | Ruling |
| --- | --- |
| QP3-1, QP3-2, QS10-20 | **Accepted as recommended.** The `Controller` protocol, its runner, and the `Say`/`Step` builders move to P6. The perceived stream moves to P3b. |
| QP3-3 | **Accepted, with a condition**: P3 adds the Python CI job (C5). The job must not rename or slow the required `fast` and `test` checks. The Python checks go in `fast` only if they add under about one minute; otherwise they run in a separate job. A separate job is not required by branch protection until the primary session adds it. C5 carries this condition. |
| QP3-4 | **Accepted:** a root uv workspace with one lock. |
| QP3-5 | **Accepted:** frozen before #83 merges. D-P3-5 governs the order. |
| QP3-6 | **Accepted:** `pytest-socket`. |
| QP3-7 | **Accepted:** Pydantic. |
| QS10-21 | **Forwarded** to the S11-C design, in progress in parallel, as a cross-lane request: R-S11-9, R-S11-10, and `mineworld perceived` in an early commit. It does not block P3. |
| QS10-19 | **Operator ruling, 2026-10-08: local models only.** No MVP-0 gate depends on a paid API. Test cassettes are recorded from a local model such as Ollama. A hosted API is an optional backend that users configure for themselves. Agents never use the operator's key. (This supersedes the first recording of the question as open.) |
| QS10-18 | **Primary-session ruling, 2026-10-08, as recommended.** Cost ceilings (calls and tokens) are keyed on wall time. The context bound stays keyed on simulated time. Binding on P5's design; nothing in P3. |
| Platforms | **Operator requirement, 2026-10-08:** "我们要保证支持全平台，mac linux windows都可以" (support every platform: Mac, Linux and Windows). P3's SDK, uv workspace, pyright and pytest-socket must work on Windows. The Python CI job runs on Ubuntu, and on Windows and macOS as a matrix if each adds under about 3 minutes; otherwise only a Windows smoke test is added. Recorded as D-P3-11, AP-12, R-P3-9, R-P3-10 and C5's matrix rule. |

## 12. Ledger (live during implementation)

```text
Status:            IN IMPLEMENTATION (fresh session, 2026-10-08), worktree
                   /Users/yuema137/mineworld-worktrees/impl-s10-p3, branch mvp0/pr-s10-p3-sdk
Implementation
base:              origin/main @ 827daf9 (#86, the freeze). #83 (S11-B) is open (head 39d02ea) and not
                   in the base: C2 models S11-A's frames only; C6 is live only if #83 merges during P3
Handoff:           handoff-s10-p3.md
```

### 12.1 C0 — freeze and contract

- [x] Implementation: the `DESIGN FROZEN` header and §10's contract were committed with the freeze (#86);
  every endpoint line there names its source (the primary session's freeze message). This session
  records the base above and initializes [`handoff-s10-p3.md`](handoff-s10-p3.md).
- [x] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`,
  both exit 0 (E-P3-0).
- [x] Review: every endpoint line in §10 has a source; none was narrowed or widened. The coordinator's
  note that `gh` is temporarily unauthenticated narrows no endpoint: a failed push is INCONCLUSIVE and
  retried.

**Anchor re-verification (working rules §3).** `git diff 9cf8f8e..827daf9` touches none of the §3
anchors except `tools/cli/src/main.rs` (save-configuration checks for `--save`; `serve`'s printed lines
and `app::bind` unchanged). Re-read in this session: `server/src/protocol.rs` and its three submodules,
the eight golden frames and `server/tests/frames.rs`, `contracts/src/{observation,action,spatial,
relation,ids,component,time,entity}.rs`, `server/src/app.rs` `bind` (prints `local_addr()`, so
`--listen 127.0.0.1:0` reports the real port), `tools/cli/src/main.rs` `serve`,
`tools/cli/tests/support/mod.rs`, `worlds/{social-cafe,market-town}/world.yaml` seats,
`systems/economy/src/offer.rs`, `scripts/ci_layer.py`, `Dockerfile`, `scripts/check_ci_pins.py`,
`.github/workflows/ci.yml`, `.github/actions/layer/action.yml`, `.structured-coding/standards.md`.

### 12.1b C1 — specs and toolchain

- [x] Implementation: root `pyproject.toml` (virtual workspace, pyright's strict configuration);
  `uv.lock`; `sdk/python/{pyproject.toml,README.md,src/mineworld_sdk/__init__.py,py.typed}`;
  `.gitignore` (`.pytest_cache/`, `.ruff_cache/`, and the `{"*` pattern rewritten as `[{]"*`, see
  DV-P3-3); `.structured-coding/standards.md` (four Python checks, prose); `docs/DECISIONS.md` ARC-56,
  DEP-24, DEP-25, DEP-26; `docs/ARCHITECTURE.md` §13 row "Contracts" and §13.1 (G-1).
- [x] Validation (E-P3-1): `uv lock`, `uv sync --locked` succeed; `ruff check`, `ruff format --check`,
  `pyright` (strict) report zero findings; the planted `def f(x): return x` gives 4 strict errors
  (`reportMissingParameterType`, `reportUnknownParameterType` ×2, `reportUnknownVariableType`), then
  removed; doc checks exit 0; `standards.py inspect` lists `ruff-lint`, `ruff-format`,
  `pyright-strict`, `pytest` enabled; the lock holds `win_amd64`, `macosx_11_0_arm64` and
  `manylinux2014_x86_64` wheels for `pydantic-core`, `websockets`, `nodejs-wheel-binaries`.
- [x] Review: each DECISIONS entry states what it adopts, the step-17 §4 alternatives with verdicts,
  and a revisit trigger; the §13.1 edit keeps D-4's direction (Rust types are the source) and agrees
  with ARC-41 (JSON stays; Protobuf declined until measured); no provider concept anywhere (I-10:
  `grep -ri 'openai\|ollama\|anthropic' sdk/python pyproject.toml` empty).

**§3's five "not verified in planning" items, verified:**

```text
virtual uv root          PASS  pyproject.toml with only [tool.uv.workspace] (+ [tool.pyright]); uv 0.12.5
                               locks and syncs it
uv licence               PASS  PyPI license_expression "MIT OR Apache-2.0" (pypi.org/pypi/uv/json,
                               read 2026-10-08)
pyright without Node     PASS  pyright[nodejs] → nodejs-wheel-binaries 24.19.0; with
                               PYRIGHT_PYTHON_GLOBAL_NODE=0 its debug log reads "Using nodejs_wheel
                               package". FINDING: by default the wrapper prefers a PATH node and queries
                               PyPI for its newest version on every run (pyright/_utils.py,
                               node.py) — CI sets both variables off (DEP-26)
pytest-socket + asyncio  PASS, with a correction (DV-P3-2): with --allow-hosts given, pytest-socket
                               ignores --disable-socket (pytest_socket/__init__.py,
                               pytest_runtest_setup: "socket_disabled and not hosts"), so the guard is
                               connect-only; asyncio.run works and 192.0.2.1 raises
                               SocketConnectBlockedError, with one flag set on every platform
Python versions          PASS  requires-python >=3.12; local CPython 3.14; the image's Debian trixie
                               python3 is 3.13 (checked again by the CI job, C5)
```

### 12.1c C2 — wire models and codec

- [x] Implementation (commit `73c2f74`): `src/mineworld_sdk/errors.py` (`MineWorldError`,
  `MalformedFrame`, `UnsupportedFrame` — a small module of its own so every later module shares one
  error base, bounded); `wire/ids.py` (14 `NewType`s, validated `*Field` aliases: canonical decimal
  ≤ u64 for opaque ids, `contracts/src/ids.rs`'s identifier rule, 32 lowercase hex for the instance,
  1–64 UTF-8 bytes and no `Cc` character for a token; `JsonValue` as a PEP 695 recursive alias);
  `wire/contract.py` (the contract shapes, `strict`/`frozen`/`extra="forbid"`, the affordance and
  request agreement validators, `Observation.events: tuple[()]` with the `UnsupportedFrame` guard);
  `wire/frames.py` (`Join`/`Submit`/`Leave`, `Welcome`/`ObservationFrame`/`Result`/`Refused`/`Closing`,
  `t`-discriminated unions, `Invite` — placed beside `Join`, its only reader, and re-exported by the
  session in C3); `wire/codec.py` (`encode`, `decode`, `to_json`); `tests/test_golden_frames.py`.
  S11-B is not at the base, so only S11-A's fields are modelled (C6 conditional).
- [x] Validation (E-P3-2): 14 passed. Mutations, each run and reverted (M-1 … M-6 in §12.4).
- [x] Review:
  - **Field-by-field against the Rust structs.** Every field name and JSON shape matches
    (`PlaceRequirement` and `Rejection` externally tagged; `ActionResult` `{"accepted":{"events":…}}` /
    `{"rejected":…}` / `"unavailable"`; `Relation.from` by alias). `TookOver`, `ClosingReason` and
    `RefusalCode` mirror the Rust enums **as they are on main**, which already hold `hosted`, `held`,
    `kicked`, `superseded`, `server_stopping` and `seat_occupied`; C6 therefore adds only #83's new
    values and fields. **Optionality that differs from serde's**, listed as C2 requires: serde reads a
    missing `Option` field as `None`; the SDK requires these present (the server always writes them, so
    the stricter decode loses nothing and the round trip is exact): `Orientation.pitch`,
    `SpatialRequirement.within_range`, `PerceivedEntity.location`, `Observation.self_location`,
    `Affordance.target`, `Affordance.unavailable_reason`, `SystemRejectionBody.detail`,
    `Welcome.resume`, `ObservationFrame.revision`, `WorldSummary.revision`. Fields a client also
    constructs default to `None` and encode `null`, as serde writes them: `Location.local`,
    `Location.facing`, `ActionRequest.target`, `ActionRequest.actor_location`, `Join.resume`. Fields
    serde omits when `None` default to `None` and are omitted: `Refused.token`, `Refused.detail`,
    `Closing.detail`, `Affordance.payload`. Integers carry the Rust width as bounds (`i32`, `u32`,
    `u64`, `i64`).
  - **Landing table.** `Join` sends exactly S11-A's fields (`protocol`, `invite`, `nickname`, `seat`,
    `resume`); no S11-B or S11-C field can be sent.
  - **`JsonValue` only at payload positions** (D-P3-7): `ActionRecord.payload`,
    `ComponentRecord.payload`, `Affordance.payload`. Nowhere else; no `Any` anywhere in `src/`.

### 12.2 Evidence

```text
E-P3-0  C0  check_doc_headings.py, check_decision_ids.py: exit 0 on the C0 tree
E-P3-1  C1  uv lock: 18 packages (pydantic 2.14.0, pydantic-core 2.50.0, websockets 17.2, pytest 9.1.1,
            pytest-socket 0.8.1, ruff 0.16.10, pyright 1.1.414, nodejs-wheel-binaries 24.19.0);
            ruff check "All checks passed!"; ruff format --check "2 files already formatted"; pyright
            "0 errors"; mutation: 4 errors; doc checks: 191 sections / 73 decision ids distinct
E-P3-2  C2  uv run --locked pytest sdk/python: 14 passed (8 golden frames, completeness, AP-3 ×2, the
            two cross-field validators, the events guard); ruff check / format --check clean; pyright
            strict 0 errors
```

### 12.4 Mutations

```text
M-1  AP-1 (a)  Welcome.hold_seconds → hold_second          RED   test_golden_frame[welcome]
M-2  AP-1 (b)  Closing.detail written null, not omitted     RED   test_golden_frame[closing]
M-3  AP-1 (b)  Refused.token written null, not omitted      SURVIVED AP-1: refused.json carries a token
               (as designed), so no golden file exercises the omission. Owner moved to AP-2: the real
               server's `unauthorized` refusal has no token, and C4 round-trips it (re-run there)
M-4  AP-1 (c)  the `leave` check removed from the table     RED   completeness names ['leave.json']
M-5  AP-3      ids coerced through float                    RED   submit golden (…996) and AP-3
M-6  AP-3      numbers coerced to strings (lax mode)        RED   test_an_id_written_as_a_number_is_refused
```

### 12.3 Deviations

```text
DV-P3-1  Decision ids mapped (bounded; ruling 6 gave S10 ARC-56…60, DEP-24…27, and the primary session
         had not mapped the placeholders): ARC-S10-a → ARC-56; DEP-S10-b → DEP-24; DEP-S10-c → DEP-25;
         DEP-S10-e → DEP-26. FINDING for the primary session: step-17 has six DEP placeholders (a…f)
         and S10's range holds four; after P3 only DEP-27 is left for DEP-S10-a, -d and -f.
DV-P3-2  D-P3-11 (b) simplified (bounded): `--allow-hosts=127.0.0.1,::1` alone, in pyproject addopts,
         on every platform, instead of adding `--disable-socket --allow-unix-socket` on POSIX: the
         plugin ignores --disable-socket once --allow-hosts is set (C1 evidence). The intent (only
         loopback reachable; asyncio works on all three platforms) is unchanged; AP-8 holds it.
DV-P3-3  `.gitignore` line `{"*` rewritten `[{]"*` (bounded; .gitignore is in AP-9's list): ruff
         honours .gitignore and refused the whole file (E902, "unclosed alternate group"). Git matches
         `[{]` as a literal `{`; checked by creating `{"x` and seeing `git status --ignored` list it.
DV-P3-4  The checks are named `ruff-lint`, `ruff-format`, `pyright-strict`, `pytest` rather than
         `ruff`/`pyright` with commands (bounded): the standards helper refuses a declared command for
         `ruff` and `pyright` and runs its own argv from PATH, outside the locked environment
         (references/standards.md, "Tools the skill does not ship argv for").
DV-P3-5  [tool.pyright] lives in the root pyproject.toml, not sdk/python's (bounded): pyright reads
         configuration from the directory it runs in, and every check runs from the root; it covers
         every future workspace member.
```


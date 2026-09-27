# PR 05c — World Pack loading, and the command that launches one

## DESIGN FROZEN

Design revision: `step-05-vertical-slice.md` §2 "PR 05c — World Pack loading and the CLI", plus
§1.3's frozen invariants. This file is the PR-level ledger for that frozen scope; it adds no scope
of its own.
Approved by / evidence: operator kickoff of this implementation session (2026-09-27), which states
the scope, the decision this PR exists to make, the acceptance list, the constraints and the
publication authority verbatim.
Implementation base: `main` @ `7f610ab`
Branch: `mvp0/pr-05c-world-packs`
Lifecycle: FROZEN

---

## 1. What this PR is

Three artefacts and one decision.

```text
worlds/social-cafe/    a real World Pack in YAML: world.yaml, people/, places/
worldpack/             mineworld-worldpack: read a pack, validate it, load it into a running World
tools/cli/             mineworld server <world> — the command ARC-6 makes the deliverable
```

The decision: **how a World Pack seeds component state.** PR 05b found that it cannot, without
inventing an `ActionId` no allocator issued (its §7 DISCOVERY). §4 below answers that, because the
answer changes what a world's event log says about where its initial state came from.

The step document's acceptance, restated because it is what this PR is judged on:

```text
A1  the pack loads, and the world it produces is the one the YAML describes
A2  entity keys resolve to ids deterministically — the same pack loaded twice gives the same ids
A3  a malformed pack is refused BY NAME, not by panic: a missing file, an unknown system, a
    person in a place that does not exist, a duplicate key, a bad field — each saying what is
    wrong and where
A4  `mineworld server worlds/social-cafe` starts, and a client can connect to it
```

## 2. Implementation contract

```text
PROJECT / PR:        MVP0 PR 05c — World Pack loading and the CLI
PRIMARY DESIGN DOC:  this file
RELATED / BINDING:   .structured-coding/plans/mvp0/step-05-vertical-slice.md §1.3, §2
                     .structured-coding/plans/mvp0/pr-05b-server.md §7 (the seeding DISCOVERY)
                     docs/MODULE_SPEC.md §4 (what a World Pack is, and the three constraints)
                     docs/MVP.md §9 AC-12 (determinism), AC-9 (causality)
                     docs/DECISIONS.md ARC-6 (artefacts are the deliverable), DEP-5, DEP-9
                     docs/REUSE_POLICY.md (the YAML dependency decision)
                     docs/ENGINEERING_RULES.md, docs/ENGINEERING_STANDARDS.md
                     contracts/src/{event,ids,entity,spatial}.rs — Causation::WorldGenesis
                     kernel/src/{world,dispatch,system,entities,registry}.rs
                     systems/presence/src/*, systems/conversation/src/*
                     server/src/{host,perception,protocol}.rs, server/PROTOCOL.md
                     server/tests/support/mod.rs (the workaround this PR removes)
IMPLEMENTATION BASE: main @ 7f610ab — 05a and 05b merged; 193 tests green at that HEAD
BRANCH:              mvp0/pr-05c-world-packs

APPROVED SCOPE:
  worlds/social-cafe/ (content), a new worldpack/ crate, a new tools/cli/ crate, the workspace
  manifest, the narrow kernel change §4 decides on, the emission constructor §5.2 adds to
  systems/presence, the removal of the workaround in server/tests/support/mod.rs, and the
  documentation those force (docs/MODULE_SPEC.md §4.1, docs/DECISIONS.md ARC-10 + DEP-10, the
  new crates' READMEs).

NON-GOALS:
  `mineworld create` / `inspect`                → S7
  a configuration schema per system             → S7 (conversation's own TODOs name it)
  Entity Packs, organizations, items, scenarios  → the pack format has the directories; MVP-0
                                                  authors none of them
  durable persistence of a loaded world          → S5
  the clients                                    → PR 05d
  a display-name component                       → §7.5: a World Pack cannot own a component

FROZEN INVARIANTS:
  - a World Pack declares capabilities and content, never secrets and never rules
    (MODULE_SPEC.md §4's three constraints)
  - entity keys resolve to ids deterministically: the same pack loaded twice produces the same
    ids, the same event ids and the same event sequence (AC-12 at this layer)
  - no floats anywhere; every ordered collection is a BTreeMap/BTreeSet/Vec, never a hash map,
    because load order reaches the event log
  - every component value a loaded world starts with traces to a recorded event (AC-9); no
    authored write bypasses the event log
  - a malformed pack is refused with a named error, never a panic
  - dependencies point one way: worldpack → systems → kernel → contracts. The server crate does
    NOT gain a dependency on a System Pack or on the loader
  - fmt, check, clippy -D warnings and cargo test --workspace clean at the final HEAD

ENDPOINT AUTHORITY:
  implementation + local validation  authorized   source: operator kickoff, this session
  semantic commits                   authorized   source: operator kickoff ("Commits and pushes
                                                   on your branch are authorized")
  branch push                        authorized   source: operator kickoff
                                                   ("git push -u origin mvp0/pr-05c-world-packs")
  PR creation / merge                FORBIDDEN    source: operator kickoff ("No merge, no PR")

TEST OWNERSHIP:
  STATIC       fmt, check, clippy -D warnings; clippy.toml's HashMap/HashSet ban is what keeps
               the determinism invariant from depending on a reviewer noticing.
  UNIT         nothing that only exercises the YAML parser. The parser is a dependency and
               ENGINEERING_STANDARDS §8 forbids a test whose only claim is that a library works.
  INTEGRATION  (this PR's real evidence) load worlds/social-cafe and assert the world it
               produced: the entities, their ids, their components, the genesis event sequence,
               the seats, and the composition. Every refusal case is an integration test over a
               real malformed pack directory on disk.
  MUTATION     determinism specifically: replace the ordered map that drives id assignment with
               a hash map and confirm the determinism test goes red. A determinism test that
               passes when determinism is broken is worthless.
  GATE 1 (real LLM):      NOT REQUIRED — nothing here is LLM-facing.
  GATE 2 (real lifecycle): `mineworld server worlds/social-cafe` really started, with a real
               WebSocket client really connecting to it. Recorded in §8 with the transcript.
               No Godot client runs here; that is PR 05d's acceptance and this PR does not
               claim it.

NORMAL STOP CONDITION:
  PR 05c implemented, validated, committed and pushed. NO PR. NO MERGE.
```

---

## 3. Source audit

Read in full before writing: `CLAUDE.md`; the `structured-coding` SKILL and its Execute-phase
resources; `step-05-vertical-slice.md`; `pr-05b-server.md` §7; `docs/MODULE_SPEC.md` §§1, 4;
`docs/DECISIONS.md`; `kernel/src/{lib,world,dispatch,system,entities}.rs`;
`contracts/src/{event,ids,entity,spatial,observation}.rs`; all of `systems/presence/src` and
`systems/conversation/src`; `server/src/{lib,main,host,runtime,perception,protocol}.rs`;
`server/tests/support/mod.rs`.

Five findings decided the shape of this PR.

### 3.1 The kernel already forbids the thing a loader wants to do

`kernel/src/system.rs`, on `System::install`:

> Only declarations belong here. There is no access to rows: **initial state is a World Pack's
> business (S7), and a system that wrote state while being installed would write facts no event
> explains.**

So the prohibition is not an oversight to route around — it is the rule, and it already names the
criterion the answer has to meet: *a fact an event explains*. Nothing in the kernel offered a path
that satisfies it, which is exactly the hole PR 05b fell into.

### 3.2 `Causation::WorldGenesis` is that path, and it was already written down

`contracts/src/event.rs`:

> `WorldGenesis` — The world coming into existence. **A loaded World Pack's initial facts are
> caused by this and by nothing else, so initial state is explained rather than uncaused.**

The contract layer has carried the answer since S2 (`DD-8`). What was missing was a kernel entry
point that records a fact with that causation. Nothing in the repository constructed
`Causation::WorldGenesis` outside `contracts/tests/event.rs` before this PR.

### 3.3 A payload can only be encoded by the pack that declared its event type

`Emission::new::<E>` requires `E`'s Rust type, and each System Pack owns its own codec
(`systems/presence/src/codec.rs` is `pub(crate)`, deliberately: "a payload's format is a contract
between the system that declares the event type and whoever reads it back"). A loader therefore
cannot build an `Arrived` payload, and must not be allowed to — so the pack exposes a constructor
for its own genesis fact (§5.2) rather than the loader learning its encoding.

### 3.4 The server crate must not learn what a System Pack is

`server/src/main.rs` says PR 05c fills its `WorldHost::spawn` closure "with nothing in this crate
changing". Filling it in place would make `mineworld-server` depend on `mineworld-worldpack` and
therefore on `mineworld-presence` and `mineworld-conversation` — a transport depending on domain
systems, which is the one-way dependency rule inverted. So the binary moves to `tools/cli`
instead, and the server crate becomes a library. This is a bounded deviation from 05b's forecast,
recorded in §7.1 with the reason.

### 3.5 Perception has to be adapted somewhere, and the composition root is the only legal place

`mineworld_presence::observe` returns `Observation<Vec<u8>>`; the transport carries
`WireObservation = Observation<serde_json::Value>` (`server/src/protocol.rs`). The adapter needs
`mineworld-server` **and** the System Packs, so it belongs to whoever depends on both. That is the
CLI, not the server and not a pack. `tools/cli/src/perceive.rs`.

---

## 4. THE DECISION — initial state is a recorded genesis fact

The question: how does a World Pack put Alice in the café, when component state is writable only
by its owning system, only through a `WorldView`, and only while resolving an action or reacting
to an event?

### 4.1 The three candidates, judged on what they do to the event log

| Candidate | What the log says afterwards | Verdict |
| --- | --- | --- |
| **Seeded allocator handed to the loader through configuration** (`HostConfig` starts the server's allocator above whatever assembly spent) | `Causation::Action(9_000_000)` — a request that no controller made, no client sent and no allocator issued. Replaying the log asks "what was action 9000000?" and the honest answer is *nothing; it was a number chosen to avoid a collision*. | **Rejected.** It invents history. It also leaks world-assembly detail into the transport's configuration, so a loader and a server would have to agree on a number to stay out of each other's way — a coupling with no owner. |
| **Assembly-time allocator owned by the pack loader** | The same fabricated `Causation::Action(n)`, with the fabrication moved one crate over and made tidier. | **Rejected**, for the same reason. Tidying the workaround is not answering it. |
| **A kernel authored-state path** | Depends entirely on whether the path records events. A path that writes components directly leaves state with no causal origin, breaks `AC-9` for the whole initial state, and makes a world unreconstructible from its log — the failure `kernel/src/system.rs` already refuses for `install`. A path that **records the facts with `Causation::WorldGenesis` and reduces them through their owners** leaves a log whose first entries say *the world came into existence, and here is what was true of it*. | **Chosen, in the second form.** |

### 4.2 What was implemented

`World::genesis(&mut self, at, facts: Vec<Emission>) -> Result<Vec<EventEnvelope>, KernelError>`:

```text
1  refuse if this world has already dispatched anything — genesis is world assembly, and a
   world that could state an uncaused fact mid-run would make AC-9 decidable only by reading
   the whole log
2  for each emission, in the order given:
     the emitter is E::OWNER, read off the event type the emission carries — not a name the
     caller supplies, so a caller cannot attribute a fact to a system that does not own it
     refuse if that system is not installed, or if its declaration does not list the event type
     allocate an EventId from the world's own counter — the same counter dispatch uses
     record with caused_by = Causation::WorldGenesis and Provenance::new(owner) with NO
       controller_decision: there is no request, so the field that names one stays empty
3  reduce every recorded fact through its subscribers, in registration order, with the same
   cascade limit dispatch uses
```

Three properties follow, and they are why this is the answer rather than a third workaround:

- **No `ActionId` is invented anywhere.** `Provenance::controller_decision` is `Option<ActionId>`
  and genesis leaves it `None`, which is what it already means: *this fact did not come from a
  request*. The server's allocator keeps starting at 1 and can never collide with assembly,
  because assembly allocates no request identities at all.
- **Every seeded component has a causal origin that survives replay.** "Where did Alice's initial
  position come from" answers: event 1, `arrived`, `caused_by: world_genesis`, emitted by
  `presence`, reduced into `Presence` by `presence` — the same reduction path a runtime `arrive`
  takes. State and log cannot disagree, because there is only one way in.
- **`AC-12` holds through the seeding.** Event ids come from the world's monotonic counter in the
  order the loader states the facts, and that order is fixed by §5.3. Two loads of one pack
  produce byte-identical event sequences.

### 4.3 Cost, stated honestly

This is a **kernel change**, which the kickoff asked to be flagged before doing: the decision
forces it, because no other layer may record an event. What it adds is 3 public items
(`World::genesis`, `Emission::owner`, one `KernelError` variant) and no domain knowledge —
`genesis` cannot name a component, an action or a system, and reads the emitting system off the
contract rather than from an argument. `Causation::WorldGenesis` becoming reachable is the
contract layer's own stated intent, so `contracts/` is unchanged.

The alternative to changing the kernel was to leave `WorldGenesis` unreachable forever and
fabricate action ids in every World Pack for the life of the project. That is not a smaller
change; it is the same change, paid for in the event log.

### 4.4 The workaround is gone

`server/tests/support/mod.rs`'s `ASSEMBLY_ACTION_ID = 9_000_000` and its four dispatched `place`
intents are replaced by one `World::genesis` call. Its stub `placement` pack now emits a `placed`
fact and reduces it, which is what a System Pack does — so the test world is a better example of
one than it was, and `grep -rn 9_000_000` finds nothing.

---

## 5. The pack, the loader, and the command

### 5.1 `worlds/social-cafe/`

```text
worlds/social-cafe/
├── world.yaml          the world's identity, the systems it enables, its population, its seats
├── people/
│   ├── alice.yaml      a barista, behind the counter
│   ├── bob.yaml        a regular, at a table
│   └── visitor.yaml    the player-controllable Person — the seat a client occupies
└── places/
    └── cafe.yaml       one place; travel is out of scope for this slice
```

Content thin, structure real: the directory layout is `MODULE_SPEC.md` §4's, and the fields the
loader reads are specified in the same document as §4.1 rather than only in code.

**A person's key is stated once.** `world.yaml`'s `population` lists the keys and the loader reads
`people/<key>.yaml`; the person file does not repeat its own key. Two copies of one fact in a pack
are two chances for them to disagree — the reason `EntityRegistry` derives its key index rather
than persisting it.

### 5.2 `worldpack/` — `mineworld-worldpack`

```text
format.rs    the authored shape: what a world.yaml, a person file and a place file may say
read.rs      directory → WorldPack, with every refusal named and located
load.rs      WorldPack → LoadedWorld: install, create, resolve, seed
catalog.rs   which system names this build provides, and how each is installed
error.rs     PackError: one variant per way a pack can be wrong
```

`WorldPack::read` is validation and nothing else; `WorldPack::load` is what produces a `World`.
Splitting them is what gives `mineworld validate` for free (§5.4) and what keeps a filesystem
error distinguishable from a composition error.

Contract types with validating `serde` are used directly in the authored shape — `EntityKey`,
`Tags`, `Millimetres` — so an illegal name is refused by the contract's own rule at the exact line
of the file that wrote it, rather than by a second implementation that could drift from it
(`contracts/src/ids.rs` is explicit that the rule lives in one place).

`presence` gains one public item: `mineworld_presence::arrival(person, location) -> Emission`,
used by both `PresenceSystem::resolve` and world assembly. The pack that declares `arrived` keeps
owning its encoding (§3.3).

### 5.3 Determinism, mechanically

```text
places first, in EntityKey order      cafe                      → 1
then people, in EntityKey order       alice, bob, visitor       → 2, 3, 4
then genesis facts in that same order, one `arrived` per person with a location
```

`BTreeMap<EntityKey, _>` is the only container in the path, so the order is the key order and not
the order the author happened to list them in, nor the order a directory happened to be read in.
Reordering `population` in `world.yaml` therefore changes nothing — and adding a person cannot
renumber a place.

The claim is pinned three ways, because the weak version of this test is the one that passes when
determinism is broken:

```text
exact ids          cafe=1, alice=2, bob=3, visitor=4 are asserted as literals
two loads agree    the full key→id map, the full genesis event sequence, and every component
                   value compare equal across two independent loads of the same directory
mutation           the driving BTreeMap replaced by a HashMap → the test must go red
```

### 5.4 `tools/cli/` — `mineworld`

```text
mineworld server <world> [--listen ADDRESS]   load the pack, host it, serve clients
mineworld validate <world>                    load it and report what it is, then exit
```

`create` and `inspect` are S7's and are absent rather than stubbed. `validate` is present because
it is `WorldPack::read` plus `load` with no new code — the kickoff's "if `validate` falls out of
the loader for free, take it".

Arguments are parsed by hand, as `server/src/main.rs` did: two subcommands and one option do not
justify a dependency, and `clap` becomes a decision when `create`/`inspect` arrive with real
option surfaces. Recorded so it is a decision rather than an omission.

---

## 6. Commit plan

| # | Commit | Impl | Validation | Review |
| --- | --- | --- | --- | --- |
| 1 | `feat(kernel)`: genesis — initial state is a recorded fact | `[x]` | `[x]` | `[x]` |
| 2 | `refactor(presence,server)`: seed through genesis, and delete the invented ActionId | `[x]` | `[x]` | `[x]` |
| 3 | `feat(worldpack)`: the pack, the loader and its refusals | `[x]` | `[x]` | `[x]` |
| 4 | `feat(cli)`: mineworld server \<world\> | `[x]` | `[x]` | `[x]` |
| 5 | `docs`: the format, the two decisions, and the ledger | `[ ]` | `[ ]` | `[ ]` |

### Commit 1 — the kernel genesis path

- [x] Implementation: `Emission` carries `owner: SystemId` read off `E::OWNER`; `World::genesis`
      (in `kernel/src/dispatch.rs`, beside `World::dispatch`, so the two share one `Dispatcher`,
      one recorder and one reducer); `World::{has_dispatched, note_dispatch}` and the `dispatched`
      flag in `kernel/src/world.rs`; `record`/`reduce` take `Option<ActionId>`;
      `KernelError::GenesisAfterTheWorldHasRun` and `KernelError::GenesisFactHasNoInstalledOwner`.
- [x] Validation: `kernel/tests/genesis.rs` — **9 tests, all PASS**. Causation is
      `WorldGenesis`; `provenance.controller_decision() == None`; ids 1,2 at genesis continue to
      3,4 on the first dispatch, which still names its `ActionId`; the owning system reduces the
      fact into the state it owns; a consequence of a genesis fact is `Causation::Event(parent)`;
      an uninstalled owner, an undeclared event type, and genesis after a dispatch are each
      refused by name and change nothing; two identically assembled worlds record identical
      `(id, event_type, causation, controller_decision)` sequences.
      **Mutation (the load-bearing one):** `genesis` changed to record
      `Causation::Action(ActionId::from_raw(9_000_000))` with a matching controller decision — the
      exact workaround this PR exists to delete. `no_action_id_is_invented_for_a_fact_nobody_requested`
      and `a_genesis_fact_is_recorded_as_caused_by_the_world_coming_into_existence` both went RED
      (`test result: FAILED. 7 passed; 2 failed`); reverted, 9/9 green again.
      Kernel suite after the change: 29 + 1 + 9 + 8 + 11 + 9 + 4 + 5 + 6 = **82 PASS**, 0 failed.
- [x] Review: every `record`/`reduce` caller inspected for the `Option<ActionId>` change — the
      only `Some(..)` sites are the two in `Dispatcher::dispatch`, so a dispatched fact still
      names its request, pinned by the pre-existing provenance assertions in
      `kernel/tests/dispatch.rs` (unchanged and green). Checked that `genesis` cannot name a
      component, an action or a system of its own, that it holds no domain vocabulary, and that
      the `Emission::owner` field is derived from the contract rather than from an argument.

### Commit 2 — presence's genesis fact, and the workaround's removal

- [x] Implementation: `mineworld_presence::arrival` in `systems/presence/src/event.rs`, used by
      `PresenceSystem::resolve` and by world assembly; `server/tests/support/mod.rs` seeds through
      `World::genesis`, its stub `placement` pack now emits and reduces a `placed` fact instead of
      providing an action nothing dispatched, and `ASSEMBLY_ACTION_ID` is gone.
- [x] Validation: `cargo test -p mineworld-presence -p mineworld-conversation` — 21 PASS;
      `cargo test -p mineworld-server` — 15 + 4 + 8 + 1 = **28 PASS** (the same assertions as
      before this PR, over real sockets), which is what says the seeding swap is
      behaviour-preserving rather than merely compiling. `grep -rn 9_000_000` over `*.rs` finds
      nothing outside prose in the two PR ledgers.
- [x] Review: the stub pack's `Room` is written in exactly one place (`react`), so it is still a
      projection; both server test binaries compile against the shared module; `arrival` is the
      only route by which an `Arrived` payload is encoded, so a loader cannot grow a second copy
      of presence's codec.

### Commit 3 — the pack, the loader, the refusals

- [x] Implementation: `worlds/social-cafe/` (`world.yaml`, `places/cafe.yaml`,
      `people/{alice,bob,visitor}.yaml`, `README.md`); `worldpack/` (`format`, `read`, `load`,
      `catalog`, `error`, `lib`, `README.md`); workspace member; `serde-saphyr` as the YAML reader.
- [x] Validation: **27 PASS** in this crate.
      `worldpack/tests/social_cafe.rs` — 9 tests, loading the real `worlds/social-cafe` off the
      disk: the pack's own statement of itself; the world (4 entities, both systems enabled, tags
      and authoring provenance as written); ids pinned as literals *and* compared across two loads;
      the genesis sequence compared across two loads down to the payload bytes; each authored
      position as an `arrived` fact with `WorldGenesis` and no controller decision, reduced into
      `Presence` and the `present-in` edge; nobody holding a `ConversationHistory` (a pack seeds
      only what it authored); a `talk` from Bob accepted and one from the visitor
      `Rejected(TooFarAway)` — the authored geometry deciding what the world permits, with no
      distance arithmetic in the test; an observation naming 4 entities and offering `talk` twice,
      unavailable both times.
      `worldpack/tests/refusals.rs` — 17 tests over real malformed pack directories under
      `CARGO_TARGET_TMPDIR`; the messages are in §8.2.
      **Mutation:** the `BTreeMap` that drives creation order replaced by a `HashMap`. Three runs:
      `FAILED. 8 passed; 1 failed` (history), `FAILED. 8 passed; 1 failed` (literal ids),
      `FAILED. 7 passed; 2 failed`. Red every time, and *which* test caught it varied — which is
      the argument for having both: run 2's two-load comparison agreed while the pinned ids did
      not. Reverted; 27/27 green.
- [x] Review: read the loader for a hash map (none; `clippy.toml` bans them and clippy is clean), a
      float (none — positions are `Millimetres`, an `i32` newtype), an insertion-ordered container
      in any path that reaches identity, and for any simulation rule (none: the loader decides no
      admissibility, evaluates no distance and writes no component). Checked the format against
      `MODULE_SPEC.md` §4's three constraints: no field can carry a secret, an endpoint, a renderer
      asset or a rule, and `deny_unknown_fields` means one cannot be smuggled in either.

### Commit 4 — the command

- [x] Implementation: `tools/cli/` — `main.rs` (`server`, `validate`, `--help`, and a named refusal
      for `create`/`inspect`/`run`), `perceive.rs` (the adapter); `server/src/main.rs` removed and
      the server crate made a library; `HostError::Build` so a caller's world builder can fail
      without the server learning what a World Pack is; `PerceptionContext` carries `&World`
      (§7.4).
- [x] Validation: **7 PASS** in this crate.
      `tools/cli/tests/server_command.rs` — 3 tests running the **real binary** against the **real
      pack** on an ephemeral port: `/status` reports 4 entities and `presence, conversation` in the
      pack's order with one seat; a real WebSocket client joins `visitor`, is welcomed as observer
      **4**, and its first observation names entities `[1, 2, 3, 4]` with Alice at `(1200, 2400)`
      millimetres and `talk` offered twice and unavailable both times; a join for `alice` — a person
      the pack does not offer as a seat — is refused.
      `tools/cli/tests/commands.rs` — 4 tests: `validate` succeeds and reports the world including
      `1  cafe` … `4  visitor`; a missing pack, a malformed pack and a command that does not exist
      each exit non-zero naming the problem and its position, with `panicked` absent from every
      message.
      Manual run, recorded in §8.3: `./target/debug/mineworld server worlds/social-cafe` with
      `curl /status`.
- [x] Review: `server/Cargo.toml` gained no System Pack and no loader dependency (checked by
      reading it and by `cargo tree`'s absence of one — `mineworld-server` compiles with only
      contracts, kernel, axum, tokio, futures-util, serde, serde_json, thiserror). The adapter
      computes no rule: it forwards to `observe` and re-parameterizes the payload. Checked that the
      CLI holds no world state and no clock of its own.

### Commit 5 — documentation

- [ ] Implementation: `MODULE_SPEC.md` §4.1 (the fields the loader reads); `DECISIONS.md` ARC-10
      (genesis) and DEP-10 (`serde-saphyr`); `worldpack/README.md`, `tools/cli/README.md`,
      `worlds/social-cafe/README.md`; `server/README.md` reconciled; this ledger.
- [ ] Validation: `cargo fmt --all --check`, `cargo check --workspace --all-targets`,
      `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
      `cargo test --workspace` — all four recorded in §8 at the final HEAD.
- [ ] Review: checked every document changed against the two-kinds-of-Markdown rule and against
      the terminology list in `CLAUDE.md` §2.1.

---

## 7. Discoveries and deviations

### 7.1 DEVIATION (bounded) — the binary moved to the CLI instead of filling `server/src/main.rs`

```text
Deviation:      PR 05b forecast that 05c would fill the WorldHost::spawn closure in
                server/src/main.rs with "nothing in this crate changing". Instead
                server/src/main.rs is deleted and the binary is tools/cli.
Reason:         filling it in place requires mineworld-server to depend on the loader and
                therefore on mineworld-presence and mineworld-conversation — the transport
                depending on domain System Packs, which inverts the one-way dependency rule
                (ENGINEERING_STANDARDS §4). Two binaries that both start a server would also
                contradict NETWORKING.md §1's one-binary rule.
Source:         server/Cargo.toml, worldpack/Cargo.toml's dependency set, NETWORKING.md §1.
Impact:         `mineworld-server` is now a library crate. The seam 05b built — HostedWorld,
                SeatRoster, Perception, WorldHost::spawn — is used exactly as designed and is
                unchanged. `cargo run -p mineworld-server` is replaced by
                `cargo run -p mineworld-cli -- server worlds/social-cafe`.
Validation:     tools/cli/tests/server_command.rs runs the real binary end to end.
```

### 7.2 DEVIATION (bounded) — the stub `placement` pack now emits a fact

Seeding through genesis requires an event to reduce, and 05b's stub pack wrote its `Room`
component directly in `resolve` while emitting nothing. It now emits `placed` and reduces it. This
is test-support code, the change makes it a more honest example of a System Pack, and every
existing server assertion passes unchanged.

### 7.3 FINDING — `Emission` had no notion of who owns the fact

Dispatch never needed one: the emitter is whichever system is currently running. Genesis has no
running system, so the fact has to say whose vocabulary it belongs to — and `Event::OWNER` already
says it, in the contract. Making `Emission` carry `E::OWNER` was therefore recovering a fact the
type already had access to, not adding a field with a new source of truth. Dispatch does not read
it, so its behaviour is untouched.

### 7.4 DEVIATION (bounded) — `PerceptionContext` had to carry the world, not only its stores

```text
Deviation:      server/src/perception.rs's PerceptionContext held a WorldRead. It now holds a
                &World, and offers `read()` for an implementation that wants only state.
Reason:         mineworld_presence::observe takes a `&World`, and its own documentation says why:
                an affordance depends on whether the world still PROVIDES the action, which is the
                system registry's answer and not the stores'. A WorldRead exposes no registry, so
                the seam PR 05b built could not be connected to the perception system PR 05a built
                at all. The alternatives were worse: a perception implementation keeping its own
                copy of the route map is the exact thing observe.rs refuses ("that check is the
                kernel's route map rather than a list kept here", because disabling a pack must
                remove its affordances with no edit anywhere — AC-2), and threading a route-check
                closure through the seam would be the same coupling with more machinery.
Source:         systems/presence/src/observe.rs's signature and its rationale; kernel/src/view.rs,
                where WorldRead has no registry accessor.
Impact:         `&World` is as read-only as `WorldRead` — World has no `&self` method that changes
                anything, every component write goes through a system's own write token, and the
                kernel has no interior mutability — so the seam's guarantee is unchanged. Two
                callers adjusted: server/src/runtime.rs passes `&self.world`, and the server's test
                perception reads `context.read()`. The server's own 28 assertions pass unchanged.
Validation:     cargo test -p mineworld-server (28 PASS) and the CLI's real-client acceptance,
                which is the first thing that ever exercised this seam end to end.
```

### 7.5 FINDING — a World Pack cannot give Alice a display name in MVP-0

`systems/presence/src/observe.rs` states it: "no system in this world owns a display name yet".
A name is component state and a World Pack cannot own a component, so a `name:` field would be a
field with no consumer and no owner. The pack therefore carries `tags` (an open vocabulary the
contract validates, already reaching clients through `PerceivedEntity::with_tags`) and an
authoring note in `Metadata`, and PR 05d gains the display name by installing a pack that owns
one. Recorded rather than guessed at.

---

## 8. Validation record

Filled during implementation; see the sections above for which commit owns each claim.

---

## 9. Limitations and follow-ups

Filled at review readiness.

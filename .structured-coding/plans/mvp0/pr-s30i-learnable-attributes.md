# PR S30-i — Learnable attributes: `tell-about-self`, the `told-about-self` fact, and what a listener remembers

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        draft 1 (2026-10-26)
Author:                 S30-i PR-design session (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s30i,
                        branch docs/pr-design-s30i, planning base origin/main @ 0b2faf2f (#382)
Approved by / evidence: none yet. Needs the operator's rulings on §15 and a freeze.
Implementation base:    main (origin/main at C0); C0 records the commit and the merged prerequisites
Start condition:        S30-f merged (yes: #373 at 97d43fe3) and S28-c merged (yes: #353 at 7fa73c5d), as step-30
                        §13.3 requires; S28-a, S28-b, S28-d … S28-g also merged; this design frozen; §15 ruled;
                        the coordinator's dispatch
Execution contract:     §13
Implementer:            Opus-class session (step-30 §13.1, the S30-i row); C6, C7 and C8 may be delegated to
                        Haiku-class sub-agents from this commit plan, each reviewed by the session
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) §§0 item 6, 4.2 item 2,
5.4 (visibility), 9.5 (memory), 11.1 (facts), 12 (I-S30-3, I-S30-8, I-S30-9, I-S30-10), 13.1 (the S30-i row), 13.3
(start conditions; release gate), 15.4 (step-28's `stable` fact) · **Memory step:**
[`step-28-shared-memory.md`](step-28-shared-memory.md) §§3 (I-S28-1, I-S28-2, I-S28-7, I-S28-8), 4.4 (the private
layer), 6.1 (ownership), 7 (bounds), 10 (no migration: rebuild) · **Builds on:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (MERGED), [`pr-s30f-attribute-consumers.md`](pr-s30f-attribute-consumers.md)
(MERGED #373; its R-S30i-f-1), [`pr-s28a-memory-home.md`](pr-s28a-memory-home.md),
[`pr-s28c-memory-scopes.md`](pr-s28c-memory-scopes.md), [`pr-s28d-anchored-scope.md`](pr-s28d-anchored-scope.md),
[`pr-s28e-memory-inspect.md`](pr-s28e-memory-inspect.md) (all MERGED) · **Parent:** [`overall.md`](overall.md) ·
**Binding parents:** `CLAUDE.md` §§1.1, 2, 4 (items 1, 2, 3, 5, 7, 8, 9, 10, 12, 13, 15); the lane-lead charter's
"no world-specific hard-coding" rule (2026-10-18); `docs/DECISIONS.md` `ARC-26`, `ARC-29`, `ARC-34`, `ARC-59`,
`ARC-100`, `ARC-101`, `ARC-109`, `ARC-120` … `ARC-126`, `ARC-153`.

**Decision ids.** One new record, `ARC-127` (telling about oneself; the told fact; learned attributes are memory).
Dated notes under `ARC-120` (item 3: `attributes` now provides one action), `ARC-122` (the disclosed block's
schema 3) and `ARC-109` (memory schema 7). `ARC-128`, `ARC-129` and `DEP-69` stay reserved and unused.

No model is called. No reference world's bytes move (no reference world enables `attributes` on the base; §11
R-S30g-i-1 covers the other order).

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-i row).** "`tell-about-self`; S28 `stable` fact ingestion".
- **Integration checkpoint (same row).** "Bob tells Alice his chronotype; Alice recalls it; Carol, absent, does not".
- **Adversarial criteria (same row).** "the fact never widens Alice's observation; a `private` attribute cannot be
  told".
- **Depends on (same row; §13.3).** "S30-i starts when S30-f and S28-c have merged." Both have.
- **Release gate (step-30 §13.3).** "the public release (S23) does not ship until S30-a…k have all merged." S30-i is
  release-blocking (RR-S30-2).
- **The visibility rule (step-30 §5.4).** `learnable`: "the holder's own observation: the value; another observer:
  nothing. Learnable only through conversation, and then stored in the learner's memory (§9.5)". "Disclosure is by
  perception only … It never depends on what the observer knows."
- **The memory rule (step-30 §9.5).** "A learned attribute is a memory (INV-4). … Alice's memory holds the utterance.
  With S30-i … it may also hold a structured `stable` fact in S28's private layer, `(subject: Bob, attribute:
  core.chronotype, value: night-owl, learned_from: EventId)`. That fact is subjective, may be false, may be
  forgotten, and **never** widens Alice's observation of Bob." "**S30-i's optional action `tell-about-self {
  attribute }`** is owned by `attributes`. It states a conversation-audience fact of the holder's true value, for a
  `learnable` attribute only. It makes 'learnable in conversation' structured and truthful without letting an LM's
  free speech become world state." "**The world records no knowledge state.** Who knows what about whom is
  cognition's."
- **Requirement placed on this PR by S30-f (R-S30i-f-1).** "A learned attribute enters the learner's memory and prompt
  through memory, never through `SelfAttributes` or `Person.description` (I-S30-9)."
- **Invariants that bind (step-30 §12).** I-S30-1 (one owner; only the owner writes), I-S30-3 (a private or hidden
  attribute never appears in another seat's observation, MCP view or client panel), I-S30-5 (no System reads `text`),
  I-S30-8 (kernel and contracts ignorant), I-S30-9 (what a person learned about another is memory, never world
  state), I-S30-10 (a world without `attributes` is byte-identical).
- **Invariants that bind (step-28 §3).** I-S28-1 (perception is the access right), I-S28-2 (sharing is transparent),
  I-S28-7 (truth is not memory: no scope reads the world log), I-S28-10 (no kernel, contract, server or protocol
  change).
- **`CLAUDE.md` §4 item 15.** An action declares its spatial requirement; a client reports intent; the server answers
  with the resolved action or a named refusal.
- **The charter's rule (2026-10-18).** Mechanisms are generic and data-driven. The world discloses facts; each
  controller kind decides by its own model (rule = configurable policy data, motivated = a scored option, LLM =
  decides from the disclosed facts); consequences are events owned by a System. Every acceptance criterion names a
  second world or a generated variant.
- **The brief (2026-10-26).** `tell-about-self` is owned by `attributes`; memory ingestion is owned by S28's owner
  (cognition's memory package); single ownership holds (events, not cross-system mutation). Consider how each
  controller kind (rule, motivated, LLM, MCP agent) chooses the action.
- **R-LOAD-1.** Locally only the targeted test files of a commit, `cargo clippy` on touched crates, `ruff` and `pyright`
  on touched Python packages; never `cargo test --workspace`; CI on the exact head is the gate.

---

## 1. Goal

Let a person **tell someone a true thing about themselves**, and let the listener **remember it**, without the world
learning who knows what.

The causal chain, in one example:

1. Bob stands next to Alice in the café. Bob's chronotype is `early-bird`, a `learnable` attribute. Alice cannot see
   it: her observation of Bob does not carry it (step-30 §5.4).
2. Bob's controller sends `tell-about-self { attribute: core.chronotype }` with target Alice.
3. The `attributes` System checks the request. It reads Bob's true value. It states one fact, `told-about-self`,
   heard by Bob and Alice only. The fact carries the true value and its phrase, "an early bird".
4. Nothing in the world changes. No component is written. Alice's observation of Bob is the same as before.
5. Alice's memory perceives the fact. It keeps the gist ("Bob told me: «an early bird»") and one structured row:
   `(subject Bob, attribute core.chronotype, "an early bird", learned from that fact)`.
6. Later Alice's prompt says, under "People I know": "Bob: … Told me: «an early bird»." Carol was in another place.
   She never perceived the fact, so her memory holds nothing about it.

What this PR adds, by layer:

1. **The action and the fact** (model crate, `attributes` pack): `tell-about-self`, its spatial requirement, its named
   refusals, its offer; `told-about-self`, its audience.
2. **A holder-only flag** (the disclosed block, schema 3): `tellable: true` on each of the holder's own entries the
   pack accepts in `tell-about-self`, so every controller and client knows what it may tell without knowing the rule.
3. **Memory** (cognition): a renderer for `told-about-self`; a private table `learned` (schema 7); the L3 line of a
   person shows what they told me; bounds.
4. **Every controller kind can choose it.** The LM controller (a `tell` act), the MCP adapter (a `tell` act kind and
   the told line in events), `rule` and `motivated` (policy data), and both clients (a menu entry).

---

## 2. Scope

### 2.1 In scope

1. **The model crate** (`authoring/attributes/`): the action type `TellAboutSelf` and the fact type `ToldAboutSelf`
   (`facts.rs`); the block's schema 3 with the holder-only `tellable` flag (`render.rs`); the typed reader
   (`disclosed.rs`) reading schema 3; the composition golden moved to schema 3.
2. **The `attributes` pack** (`systems/attributes/`): `providing::<TellAboutSelf>()`, `emitting::<ToldAboutSelf>()`;
   `validate`, `resolve`; `offers`; `discloses` passes the tellable rule to the renderer.
3. **The Python SDK reader** (`sdk/python/src/mineworld_sdk/attributes.py`) and the GDScript reader
   (`clients/protocol/mineworld/attributes.gd`): schema 3, `tellable`, and a `tellable(entries)` selection.
4. **Memory** (`cognition/lm-controller/src/mineworld_cognition/memory/`): the `told-about-self` renderer; `Rendered.learned`;
   the base private table `learned` and schema 7; ingestion; the L3 line; the citable set; the cap and the host bound
   step; the S28-b parity filter; `mineworld-memory inspect` shows the table.
5. **The LM controller** (`cognition/lm-controller/src/mineworld_cognition/controller/`): the tellable list and the
   `tell` act, offered only when the seat may tell.
6. **The MCP adapter** (`clients/agent/`): `TellAct`, `AttributeLine.tellable`, `EventLine.told`; the generated tool
   reference.
7. **`rule` and `motivated`** (`cognition/rule-controller/`, `cognition/motivated/`): one shared `tell` policy block.
8. **Both clients** (`clients/2d/`, `clients/3d-spike/`, the shared protocol module): a "Tell about myself" menu entry
   with a chooser of the own sheet's tellable lines.
9. **The scenario** (§5.10): two fixture worlds and one hand-written oracle; real-server tests.
10. **Specification sync**: `docs/MODULE_SPEC.md` §§4.1, 5.3; `docs/decisions/ARC-127.md` (new) and dated notes under
    `ARC-120`, `ARC-122`, `ARC-109`; the READMEs of `systems/attributes/`, `clients/agent/`, `sdk/python/` (one line each).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Telling an attribute another pack owns (`employment.occupation`, a future `economy` or `notices` one) | the owner's own PR, with the model crate's helper this PR exports (Q-S30i-3) |
| A world's Interaction List over `tell-about-self` (forbid a pair, another range) | later; `attributes` already uses its one configuration slot for `configure/attributes.yaml` (A-i-20) |
| Lying, guessing, gossip about a third person | not a world action: free speech is `talk`, and stays an utterance (step-30 §9.5) |
| Systems that act on what someone learned | never: knowledge is cognition's (I-S30-9) |
| Localised told phrases | S30-k (the fact carries the server's phrase; S30-k decides the locale it is rendered in) |
| Reference worlds enabling `attributes` | S30-g (§11 R-S30g-i-1) |
| Any change under `kernel/`, `contracts/`, `server/` | never |

### 2.3 Dependencies

- **Hard, merged:** S30-c (the pack, the model crate, `RunningCatalogue`), S30-f (schema 2, the three readers, the
  composition golden, `SelfAttributes`, `Person.description`, the agent's `AttributeLine`), S28-a … S28-g (the memory
  home, scopes, the scope-table host, inspect, the scale gates).
- **Concurrent, coordinate:** S31-b2 (`impl-s31b2`, `notices` adopts `AttributeOwner`) calls `disclose_owned`; this PR
  keeps that signature unchanged (§5.5). S30-g, S30-h, S30-j, S30-k are being designed in parallel; §11 states what
  each must take from here.

---

## 3. Audit (`origin/main @ 0b2faf2f`, 2026-10-26)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-i-1 | `attributes` declares no dependency, owns `Traits`, emits four facts, subscribes to four, provides **no action**; `VERSION = 1` | `systems/attributes/src/system.rs:88-108` | the action and the fact are added to this declaration (§5.1); C0 records whether `ARC-25` asks for a `VERSION` bump |
| A-i-2 | `ARC-120` item 3: "`attributes` … provides no action; it reacts to `attribute-change-proposed`" | `docs/decisions/ARC-120.md` | a dated note under `ARC-120`, and `ARC-127` (C9) |
| A-i-3 | `check` reads another pack's `Presence` component without declaring a dependency, to pick a perceivable change's audience | `systems/attributes/src/system.rs:162-205` | `validate` may read `Presence` the same way; no new dependency (D-i-3) |
| A-i-4 | `discloses` renders the holder's or another observer's view through `render::disclose(catalogue, &ATTRIBUTES, value_of, View::of(observer, subject))`; derived values computed at the read | `systems/attributes/src/system.rs:208-243` | the tellable flag is computed in the same call (§5.5) |
| A-i-5 | `Entry { attribute, kind, label, facets, rank?, value?, text? }`, `deny_unknown_fields`; the doc comment: "A later schema adds fields under a new `schema` number; a reader of this one refuses a field it does not know, so a block is read whole or not at all" | `authoring/attributes/src/render.rs:55-77`; `SCHEMA = 2` at line 29 | a new field is schema 3 (Q-S30i-1) |
| A-i-6 | `disclose_owned::<O>` calls `disclose` with the owner's id and `View::of` | `authoring/attributes/src/owner.rs:31-44` | its signature stays; owners' entries are never `tellable` (Q-S30i-3) |
| A-i-7 | The Python reader reads schema 2 only and skips any other block whole, counted; the GDScript reader likewise | `sdk/python/src/mineworld_sdk/attributes.py:32, 170-205`; `clients/protocol/mineworld/attributes.gd:13-62` | both move to schema 3 in the same commit as the server (C1) |
| A-i-8 | `talk` declares `same_place().within(range).requiring_target_available()`, evaluated by `SpatialRequirement::evaluate`; refuses `NoSupportedInteraction` for self or a non-person, `TargetUnavailable` for an unknown target, `PreconditionFailed` for an unlocated speaker, and `malformed-payload` for an unreadable payload | `systems/conversation/src/action.rs:60-90`; `system.rs:95-140` | `tell-about-self` follows the same order and the same contract evaluator (§5.3) |
| A-i-9 | `spoke`'s default audience is the place (overheard), narrowable to participants; `conversation-started`'s is participants | `systems/conversation/src/interactions.rs:94-110` | `told-about-self` takes participants (Q-S30i-2) |
| A-i-10 | Perception asks every provider's `offers` for no target and for each present entity, prices each offer with the one evaluator, and sends every offer with its verdict; a complete offer carries its payload | `systems/presence/src/observe.rs:221-300`; `interaction.rs:141-190` | a complete offer per (person, attribute) would multiply affordances; one incomplete offer per person (§5.4), as `notices` does (Q-S31a-3, `systems/notices/src/system.rs:269-302`) |
| A-i-11 | The LM controller's choice is `say`, `attempt` (a complete affordance by number), `go`, `sleep`, `nothing`; `sleep` and `go` join the schema only when offered, so a request without them is unchanged | `controller/choice.py:20-75`; `intent.py:1-80`; `situation.py:100-180`; `prompt.py:110-220` | a `tell` act joins the schema only when a tell is offered and something is tellable (§5.7.1) |
| A-i-12 | The MCP adapter's `act` is one tool over a discriminated union of act kinds; `subscribe`/`unsubscribe` were added as kinds by S31 | `clients/agent/src/mineworld_agent/models.py:1195-1295` | a `tell` kind (§5.7.2) |
| A-i-13 | `AttributeLine { attribute, label, kind, value?, text? }`, omitted fields excluded | `clients/agent/src/mineworld_agent/models.py:486-499` | `tellable` added, omitted when false |
| A-i-14 | `motivated` builds a fixed set of `Choice` variants from the observation; `OPTIONS` (13 names) is what a `base` entry may name; draws use fixed indices | `cognition/motivated/src/options.rs:40-75, 320-400`; `parameters.rs:50-54`; `model.rs:45-57` | a `Tell` choice, option `tell`, a new draw index (§5.7.4) |
| A-i-15 | `rule`'s parameters are three blocks, each defaulted and inert when absent; draw indices used: 0–15 | `cognition/rule-controller/src/model.rs:1-45`; `agenda.rs:30`, `offered.rs:39-42`, `social.rs:48-52` | a fourth block `tell` and draw index 16, independent of the others (§5.7.3) |
| A-i-16 | A memory renderer is a pure function `(fact, me, names) -> Rendered`; `Rendered.names` teaches the `NameBook`; unknown event types render generically | `memory/render/__init__.py:1-200`; `render/naming.py` | a `told-about-self` renderer; `Rendered.learned`, written at ingest like names (§5.6) |
| A-i-17 | Ingestion writes the seat's private rows from `Rendered`, in one transaction: names, the held record, episodes, stable facts | `memory/ingest.py:66-130` | `learned` rows are written there too |
| A-i-18 | The home schema is 6; a home of another version is refused by name and rebuilt from the perceived stream (QS28-8); S28-b and S28-d each bumped it; the S28-b parity view strips `meta.schema` and new tables | `memory/home/schema.py:22-30`; `tests/capture_s28b_parity.py:16-19, 121-126` | schema 7 for one new private table; the parity filter strips `learned` lines (§5.6.3) |
| A-i-19 | An optional scope's `index` receives the envelope (`FactRecord`) and the seat's `Reading` (gist, salience, mine, importance, anchors), **never the payload**; a scope re-enabled later is rebuilt from the stored `access` and `access_anchor` rows | `memory/scopes/interface.py:85-110`; `home/scope_tables.py:220-260, 330-360` | a scope could not rebuild learned rows, because the payload is not kept (F-S30i-4) |
| A-i-20 | `configures!` and `interactions!` both define a pack's one configuration slot; `attributes` uses `configures!` for `configure/attributes.yaml` | `sdk/rust/src/pack.rs:237-262`; `sdk/rust/src/interactions/mod.rs:210-235`; `systems/attributes/src/system.rs:40-50` | `tell-about-self` has no Interaction List section in this PR; its range is a compiled default (D-i-2) |
| A-i-21 | The L3 line is `"<name>: met on day N, last seen day M, met K times, E exchanges[, level]."`, citing first, last and level facts; the citable set includes stable keys | `compress/stable.py:52-84`; `memory/home/view.py:178-195` | learned rows extend the line and the citable set (§5.6.4) |
| A-i-22 | The host bound drops units in a fixed step order, the last step `stable-facts` | `memory/retention/bound.py:204-250` | a `learned` step before `stable-facts` (§5.6.5) |
| A-i-23 | The 2D menu lists one entry per affordance: complete (send), composable (ask for input, then compose), unsupported (listed disabled); `intents.gd` holds the per-type inputs | `clients/2d/scripts/menu.gd:1-30, 160-240`; `clients/2d/scripts/intents.gd:74-140` | an unknown incomplete offer already shows as "not supported by this client"; the entry becomes composable (§5.8) |
| A-i-24 | The 3D slice has an offer menu and its own intents | `clients/3d-spike/scripts/slice/offer_menu.gd`, `slice/intents.gd` | the same entry in 3D |
| A-i-25 | No world under `worlds/` enables `attributes` | `grep -n attributes worlds/*/world.yaml`: empty | no reference world's bytes move (I-i-1) |
| A-i-26 | `cargo test -p mineworld-cli --test decisions` fails on main for a reason S30-f recorded (a stale golden after S29-a) | `pr-s30f-attribute-consumers.md` §16 "Evidence" | the decisions parity check uses the case the test names, not the whole golden, and records that failure as pre-existing |

### 3.2 Findings

- **F-S30i-1. The holder cannot see which of its attributes it may tell.** The block carries no visibility
  (A-i-5). A controller would have to guess and be refused. Decided by Q-S30i-1's recommendation: the holder's own
  entries carry `tellable: true` where the pack accepts `tell-about-self` (schema 3). The server states the answer;
  no reader evaluates a rule (`CLAUDE.md` §4 item 15).
- **F-S30i-2. A complete offer per attribute is too many.** The built-in catalogue has about 60 `learnable`
  attributes. In a café of 12 people that would be about 660 affordances in one observation (A-i-10). One
  incomplete offer per person is 11. The tellable list travels once, in the holder's own block.
- **F-S30i-3. An owner's attribute cannot be read by `attributes`.** `employment.occupation` is `learnable`, but its
  value is `employment`'s projection, which `attributes` cannot call without depending on `employment` (A-i-6;
  `ARC-120` note 2026-10-18). Q-S30i-3 recommends: only attributes `attributes` owns are tellable in this PR; an owner
  may provide its own telling later with a model-crate helper.
- **F-S30i-4. A scope cannot hold learned rows.** A scope never sees the payload, and its rows must be rebuildable
  from held facts (A-i-19; I-S28-9). The told value lives only in the payload. So the learned row belongs in the
  person scope's private layer, written at ingest from the renderer, as names are (A-i-16, A-i-17). That is what
  step-30 §9.5 says ("S28's private layer"). Cost: schema 7 (Q-S30i-4).
- **F-S30i-5. The existing `stable` table is per counterpart, not per attribute.** Step-30 §9.5 calls the new row "a
  structured `stable` fact". The table `stable` is keyed `(seat, entity)` and holds meeting counts (A-i-21). A
  second key cannot be added without changing L3's meaning. Decided (D-i-5): a new table `learned`, keyed
  `(seat, subject, attribute)`, shown inside the subject's L3 line. The word "stable" in step-30 is read as "an
  L3-level fact"; a dated note records it (§12).
- **F-S30i-6. No Interaction List for this action** (A-i-20). A world cannot forbid telling for a pair or change the
  range in this PR. Recorded as a limitation (§2.2) and in `ARC-127`'s Revisit.

### 3.3 What does not exist

No action of `attributes`. No `told-about-self` fact. No `learned` table. No consumer of a told fact. No controller
that tells anything about itself, except by free `talk`.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

- **Adopted, already in the tree:** the contract layer's `SpatialRequirement` and its one evaluator; perception's
  offer pricing; the memory home, its transaction, its dump codecs and its host bound; `pydantic` for the renderer's
  payload model; the LM controller's conditional-schema pattern (`sleep`, `go`); the agent's act union; the 2D
  composed-entry pattern (`invite`'s suggestions).
- **Considered and declined:**
  - an epistemic-logic or belief-tracking library (for example a knowledge graph of "who knows what") — the world
    records no knowledge state by rule (step-30 §9.5), and memory already stores what a person perceived;
  - an optional memory scope for learned rows — it cannot rebuild them (F-S30i-4);
  - one complete offer per attribute — too many affordances (F-S30i-2).
- **Not forced:** nothing here adopts a framework's execution model.

---

## 5. Design

### 5.1 The action (model crate `facts.rs`; owned by `attributes`)

```rust
/// A person tells the intent's target the true value of one of their own attributes.
pub struct TellAboutSelf { pub attribute: QualifiedId }
impl Action for TellAboutSelf {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("tell-about-self");
    const OWNER: SystemId = ATTRIBUTES;
    const CATEGORY: ActionCategory = ActionCategory::Social;
}
pub const TELL_RANGE: Millimetres = Millimetres::new(3_000);
pub fn tell_requirement() -> SpatialRequirement;   // same_place().within(TELL_RANGE).requiring_target_available()
```

- The teller is the intent's actor; the listener is its target. The payload names the attribute only. The value is
  never in the request: the server reads the true value (step-30 §9.5 "truthful").
- **D-i-2. The range is a compiled default**, three metres, the same as `talk`'s compiled default
  (`INTERACTION_RANGE`). It is a pack default, not a world's hard-coding. A world cannot change it in this PR
  (F-S30i-6). If a world gives `talk` a longer range, telling still needs three metres; this is recorded in `ARC-127`.
- The type lives in the model crate, beside `AttributeChangeProposed`, so a controller can build the request with no
  dependency on the System Pack (`ARC-124`; `motivated` already depends on the model crate, A-i-14).

### 5.2 The fact (model crate `facts.rs`; emitted by `attributes` only)

```rust
pub struct ToldAboutSelf {
    pub teller: PersonId, pub listener: PersonId,
    pub attribute: QualifiedId,
    pub label: String,              // the declaration's label, after overrides
    pub value: AttributeValue,      // the true value at the instant, in the save's wire form
    pub text: String,               // the holder-view phrase of that value (non-empty)
}
// EVENT_TYPE "told-about-self", OWNER attributes, SCHEMA_VERSION 1
```

- **Audience: `Participants`** — the teller and the listener (Q-S30i-2). `about: [teller]`, `participants: [teller,
  listener]`, `at_place(place)`. A third person present does not perceive it. Words said aloud are `talk`'s and keep
  `spoke`'s audience.
- **Not biographical** (`ARC-29`): telling is not an event in a life.
- **Nobody reduces it.** `attributes` does not subscribe to it. No component changes. The world records that a
  statement was made, not that anyone knows anything (step-30 §9.5; I-S30-9).
- `label` and `text` are carried so that memory and the agent need no catalogue, and so that the Python side never
  renders a value (S30-f F-S30f-8: the renderer exists once, in Rust). They are world text (INV-S27-12).
- `value` is the typed value as `trait-changed` carries it. It may be stale later, when the trait changes; that is
  why the learned row is "subjective, may be false" (step-30 §9.5).

### 5.3 `validate` and `resolve` (`systems/attributes/src/tell.rs`, new)

`validate` reads state and writes nothing. It checks, in this order, the order a player experiences:

| # | Check | Refusal |
| --- | --- | --- |
| 1 | the payload decodes | `System { code: "malformed-payload" }` |
| 2 | a target is named | `PreconditionFailed` |
| 3 | the actor is a Person; the target is a Person; they differ | `NoSupportedInteraction` |
| 4 | the target exists | `TargetUnavailable` |
| 5 | the world has a running catalogue | `System { code: "attributes-uncomposed" }` |
| 6 | the attribute is declared **and is not `hidden`** | `System { code: "unknown-attribute" }` — the same answer for both, so a hidden attribute's existence never leaks |
| 7 | its owner is `attributes` | `System { code: "told-by-owner", detail: <owner id> }` (Q-S30i-3) |
| 8 | its visibility is `learnable` | `System { code: "not-learnable" }` — for `perceivable` and `private` |
| 9 | the teller has a value (stored, or derived at this instant) | `System { code: "no-value" }` |
| 10 | the value renders a non-empty phrase in the holder view | `System { code: "nothing-to-tell" }` (an `instant`, Q-S30f-4; an empty set) |
| 11 | the teller is located | `PreconditionFailed` |
| 12 | `tell_requirement()` evaluated against both `Presence` locations and the target's lifecycle | the evaluator's own: `TooFarAway`, `TargetUnavailable`, … |

- Steps 6–10 are one function in the model crate, `tellable(catalogue, declaration, value) -> Result<String,
  TellRefusal>`, used by `validate`, by `offers` (to decide whether the actor has anything to tell) and by
  `discloses` (the `tellable` flag). One rule, three readers in one crate; no reader outside it.
- `resolve` re-reads the value (the same instant), renders the phrase, and emits one `told-about-self`. It writes
  nothing.
- **Strong typing.** The refusal reasons are a closed enum `TellRefusal` in the model crate, mapped to the
  `RejectionCode`s above in one place.

### 5.4 The offer (`offers`)

- With a target that is a Person other than the observer, and the observer a Person with **at least one tellable
  entry** (§5.3 steps 6–10 over its own attributes), `attributes` offers one **incomplete** `tell-about-self` with
  `tell_requirement()` and the target's availability. Otherwise nothing.
- Incomplete, once per person, as `notices` offers `subscribe` (Q-S31a-3): no controller tells at random, and the
  observation grows by one affordance per perceived person, only in worlds with `attributes`.
- Perception prices it with the one evaluator; the verdict travels (A-i-10).

### 5.5 The holder's block, schema 3 (model crate `render.rs`; Q-S30i-1)

```json
{ "schema": 3,
  "entries": [
    { "attribute": "core.chronotype", "kind": "level", "label": "Chronotype", "facets": ["decision"],
      "value": { "level": "early-bird" }, "text": "an early bird", "tellable": true } ] }
```

- **One field added:** `tellable: true`, present only in the **holder's** view, only on entries §5.3 steps 6–10
  accept. Absent otherwise (`skip_serializing_if`). Another observer's view never carries it.
- `render::disclose` gains an argument `tells: Tells` (`Tells::Accepted(&dyn Fn(&Declaration, &AttributeValue) ->
  bool)` or `Tells::Never`). `attributes`' `discloses` passes its rule; `disclose_owned` passes `Tells::Never`, so its
  signature, and S31-b2's call, do not change (§2.3).
- `SCHEMA = 3`. The Rust reader (`Disclosed`), the Python reader and the GDScript reader read schema 3 only and skip
  any other block whole (S30-f's rule; no compatibility debt, `CLAUDE.md` §4 item 12). Each gains a selection
  `tellable(entries)`: the entries with `tellable` and `text`, in rank order.
- The composition golden (`authoring/attributes/tests/fixtures/composition.json`) moves to schema 3 by a reviewed,
  hand-made edit: the schema number, plus one new case with `tellable` entries and its hand-written expected selection.
- **Cost.** C1 measures the holder's own block on S30-f's 93-attribute fixture before and after; the expected growth is
  about 16 B per learnable entry, in the holder's own observation only. Recorded in §16.

### 5.6 Memory (cognition's memory package; owner of the private layer, step-28 §6.1)

#### 5.6.1 The renderer (`memory/render/attributes.py`, new; registered in `RENDERERS`)

```python
class ToldAboutSelf(Payload):
    teller: PersonRef; listener: PersonRef; attribute: str; label: str; text: str

def told_about_self(fact, me, names) -> Rendered: ...
```

- Gist: the listener reads `"<teller> told me: «<text>»"`; the teller reads `"I told <listener>: «<text>»"`; anyone
  else (none under §5.2's audience; kept for a world that widens it later) reads `"<teller> told <listener>:
  «<text>»"`. Text passes through `quote` (no `«`/`»` forgery) and names through the `NameBook` (no ids, I-10).
- Salience `notable`; importance `TOLD = 500` (as `named`); `mine` when I am the teller.
- `Rendered.learned: tuple[Learned, ...]` — one `Learned(subject=teller, attribute, label, text, value)` when I am
  not the teller; empty otherwise. `value` is kept as the payload's canonical JSON text.
- An undecodable payload falls back to the generic rendering, counted, with no learned row (A-i-16).

#### 5.6.2 The private table `learned` (schema 7)

```text
learned (seat, subject, attribute, label, text, value, source_key, told_at,
         PRIMARY KEY (seat, subject, attribute))
```

- Written by `ingest`, in the same transaction as the held record, from `Rendered.learned`: an upsert, so the newest
  telling of one attribute by one person wins. `source_key` is the told fact's `EventKey`.
- **Per seat, from the seat's own perception only.** A row exists only because this seat held the fact (I-S28-1).
  Another seat's rows are never read (I-S28-3). Sharing a home changes nothing (I-S28-2).
- `VIEW_TABLES` and `CODECS` gain the table (`source_key` a key column, written as a number). `SCHEMA_VERSION = 7`.
  A schema-6 home is refused by name and rebuilt from the perceived stream (QS28-8), as S28-b and S28-d did.
- `ARC-109` gets a dated note: "schema 7 since S30-i: the private table `learned`".

#### 5.6.3 Parity and dumps

- An empty `learned` table writes no dump line. A world without `attributes` therefore dumps the same lines as on
  `main`, except `meta.schema`. The S28-b parity view (the filter at `capture_s28b_parity.py:121-126`) already strips
  `meta.schema`; it also strips `learned` lines (one condition beside `access_anchor`'s), so
  `tests/golden/s28b_parity.json` is unchanged.

#### 5.6.4 Recall (the L3 line)

- `stable_text(fact, names, learned)` appends, when the subject has learned rows, `" Told me: «t1»; «t2»."` —
  at most `LEARNED_PER_LINE = 3` rows, newest `told_at` first, then by attribute id. The line is cut at 160 B with its
  citation, as every line is (A-i-21).
- The citation adds each shown row's `source_key`. The citable set (`memory/home/view.py`) includes `learned.source_key`,
  so the line stays citable after its L0 record is forgotten, as a stable fact's keys do.
- **Without learned rows the L3 line is byte-identical** (I-i-2).
- Nothing else reads `learned`. `SelfAttributes` and `Person.description` read the observation only (R-S30i-f-1).
- **D-i-6. Why L3 and not a new section.** The person scope's five sections and their byte split sum to the 6 000-byte
  ceiling (`ARC-59`); a sixth section would take bytes from another and move every request. A line extension moves
  nothing where nothing was told. Known limit: L3 lists the 16 most-met people; a person met rarely in a crowd may be
  cut. The told gist still reaches L1/L0 and the FTS search (R-i-4).

#### 5.6.5 Bounds and forgetting

- **A cap:** `LEARNED_MAX = 256` rows per seat. An insert past it deletes the row with the oldest `told_at`, then the
  lowest `(subject, attribute)`. Deterministic, inside the ingest transaction.
- **The host bound** gains a step `learned` before `stable-facts`: it drops learned rows oldest first (A-i-22), each
  counted like the other steps.
- **Retention policies** do not touch `learned`, as they keep stable facts and names (MODULE_SPEC §5.2's table). The
  row "may be forgotten" by the cap and the bound (step-30 §9.5).
- The seat view's size includes the rows (step-28 §7), so the S28-g scale gates measure them.

#### 5.6.6 Inspect

- `mineworld-memory inspect` lists `learned` rows with the seat's other private tables (S28-e reads view tables from
  `VIEW_TABLES`); C4 checks that one row appears with its subject's name and its citation.

### 5.7 Each controller kind chooses the action

Every kind reads the same two disclosed facts: **who I may tell** (the available `tell-about-self` affordances) and
**what I may tell** (my own entries with `tellable`). None evaluates a rule.

#### 5.7.1 The LM controller (`cognition/lm-controller`)

- `Situation.tell_to: tuple[str, ...]` (labels of people with an available `tell-about-self`) and
  `Situation.tellable: tuple[Tellable, ...]` (`number`, `attribute`, `label`, `text`; from the observer's own block
  through `mineworld_sdk.attributes.tellable`; cut at `TELLABLE_BYTES = 400` at an entry boundary, the composition
  rule of S30-f §5.4).
- The prompt, only when both are non-empty (else byte-identical, I-i-2):
  - user message, under `I can talk to`: `I can tell about myself to: Alice, Carol` and
    `Things about me I may tell (number):` then one line per tellable, `1. Chronotype: an early bird`;
  - the rules: `You may also use "tell" with "to" set to a person listed above and "attribute" set to a number from
    "Things about me I may tell"; the world decides whether you may.`
- `choice_schema(..., tell=True)` adds the act `tell` and the key `attribute` (integer). `SocialChoice.attribute: int |
  None = None`. Without `tell`, the schema is the base schema, unchanged (A-i-11).
- Parsing: a number not listed is the new content reason `unknown_attribute`; a `to` not in `tell_to` is
  `unknown_person`. `intent.build` sends `offers.request(observation, "tell-about-self", target=to, payload=
  {"attribute": <id>})`; a vanished offer is `not_offered` (A-i-11).
- Logged decision inputs gain the two fields with empty defaults, so an old log replays to its old request
  (S30-f A-f-8).
- Free speech stays free: the model may also `say` "I'm an early riser". That is an utterance, not a told fact
  (step-30 §9.5).

#### 5.7.2 The MCP adapter (`clients/agent`)

- `TellAct { kind: "tell", to: Ref, attribute: WorldText }` joins the `Act` union (`ARC-153`'s note: one typed kind
  per action sent by name). The adapter resolves `to`, sends `tell-about-self`, and returns the server's `Outcome`.
- `AttributeLine.tellable: bool = False`, excluded when false: `me.attributes` shows which lines may be told. A world
  without attributes renders byte-identical views.
- `EventLine.told: ToldLine | None` (`attribute`, `label`, `text`, each `WorldText`), as `said` is for `spoke`, so an
  agent reads what it was told from its events.
- The generated tool reference (`docs/reference/agent-tools.{json,md}`, S30-f D-impl-6) is regenerated; the diff is
  the new kind and the two fields only, reviewed in §16.
- An outside agent's own memory is its own. It perceives the told fact in its events like any player (INV-S27-2).

#### 5.7.3 `rule` (`cognition/rule-controller/src/tell.rs`, new; policy data)

```yaml
models:
  rule:
    tell:
      attributes: [core.chronotype, core.hometown]   # 0 … 16 qualified ids; default []: never tells
      per_mille: 200                                  # 0 … 1 000; default 0
      within: 300                                     # seconds since they last spoke to me; 1 … 3 600; default 300
```

- At a consult where `rule` would otherwise not act: take the people whose newest line in my disclosed
  `ConversationHistory` is within `within` seconds of now and to whom `tell-about-self` is **available**, lowest
  `EntityId` first; draw `TELL_DRAW = 16` (a new, independent index); if below `per_mille`, choose by the same draw
  one of `attributes` that my own block marks `tellable`; send `tell-about-self`.
- Ids are data, decoded and shape-checked with line and column, never literals in code (I-i-5). An id my block does
  not mark tellable is skipped, so one policy works in any world.
- Absent or empty: nothing happens, and no draw is taken, so every existing decision is unchanged (the `decisions`
  golden's affected case, A-i-26).

#### 5.7.4 `motivated` (`cognition/motivated`; a scored option)

- `motivated` reads the same `tell` block (`attributes`, `within`; `per_mille` is not used: scoring decides).
- `Choice::Tell(PersonId, QualifiedId)`: at most one per eligible person (§5.7.3's eligibility), the attribute chosen
  by `TELL_DRAW`. Added only when the block lists at least one attribute.
- `OPTIONS` gains `tell`; its `base` defaults to 300 per mille. S30-f's `attributes` consideration may name option
  `tell` (for example `core.talkativeness`), so how talkative a person is changes how often they tell, by data.
- With no `tell` block every score, draw and decision is unchanged.

#### 5.7.5 Both clients (human players)

- The shared protocol module gains `MineWorldAttributes.tellable(entries) -> Array` (the composition rule; S30-f §5.9).
- **2D:** `intents.gd` `INPUTS["tell-about-self"] = {"choose": "tellable"}`. The person's menu shows "Tell about
  myself…" as a composable entry; choosing it lists my tellable lines (`label: text`); choosing a line composes
  `{"attribute": id}` and submits. The verdict is the server's; an unavailable entry stays choosable (`R3'`).
- **3D:** the offer menu shows the same entry; the same chooser panel; the same payload.
- A told fact addressed to me appears in each client's event line through the words key `event.told-about-self`
  ("{teller} told you: {text}"). No client decides anything.

### 5.8 What never happens

- `attributes` never writes a component on a telling; no System subscribes to `told-about-self`.
- No observation, MCP view field other than `events`, or client panel changes because of a told fact.
  `Person.description`, `SelfAttributes`, `PersonLine.description` and the inspect panels read the observation only.
- No scope, consumer or test reads the world log to find what someone told (I-S28-7).

### 5.9 Decisions recorded here rather than asked

D-i-2 (range a compiled default), D-i-3 (no new dependency; `Presence` read as `check` reads it), D-i-5 (a new table
`learned`, not a column of `stable`), D-i-6 (shown in L3), D-i-7 (`hidden` and undeclared refused alike), D-i-8
(`VERSION`: C0 applies `ARC-25`'s rule — a new action and a new fact type are additions, not a changed payload; if the
rule asks for a bump, it is made and recorded).

### 5.10 The scenario: two worlds, one oracle

- **World A** `tools/cli/tests/fixtures/attributes-tell/`: two places, `cafe` and `street`; the systems of S30-f's
  `attributes-consumers` fixture (so `employment` is present for the owner case); requires `person-attributes: "^1"`;
  every value authored. Its own `attributes/local.yaml` declares two world attributes: `attributes-tell.secret`
  (`hidden`) and `attributes-tell.nickname` (`learnable` `text`, no default).
  - **Alice** (cafe): `core.chronotype: night-owl` (learnable), `core.religion` (private).
  - **Bob** (cafe, 1.2 m from Alice): `core.chronotype: early-bird`, `core.hometown: Leeds` (learnable text),
    `core.religion` (private), `core.stature` (perceivable), `attributes-tell.secret`, no `nickname`, and a job with
    `title: baker` (so `employment.occupation` is his, owned by `employment`).
  - **Carol** (street): `core.chronotype: neutral`.
  - **Dave** (cafe, 2 m from Bob, not the listener): proves the participants audience.
- **World B** `tools/cli/tests/fixtures/attributes-tell-harbour/`: one place, `quay`; requires `person-attributes`
  too, and adds its own vocabulary: a world-declared learnable attribute `attributes-tell-harbour.home-port`
  (category over `harbour-ports`), and an `overrides:` entry narrowing `core.chronotype` to `private`. People Mia and
  Noor. No line of code names anything in it.
- **The oracle** `tools/cli/tests/fixtures/attributes-tell.expected.json`, written by hand from the two worlds'
  declarations (test rules §25): the told phrases, the refusal codes, the expected memory lines, and the texts that
  must be absent.
- **Real-server tests:**

| Test | Drives | Checks |
| --- | --- | --- |
| `tools/cli/tests/attributes_tell.rs` (Rust, the real binary) | Bob tells Alice `core.chronotype`; Bob tries `core.religion`, `attributes-tell.secret`, `core.stature`, an undeclared id, Carol (another place); in B, Mia tells Noor `home-port`, tries `core.chronotype` | Alice's and Bob's perceived streams carry one `told-about-self` with `text` "an early bird"; Carol's and Dave's carry none; each refusal's code; Alice's observation of Bob before and after is identical; B's told phrase and refusal |
| `cognition/lm-controller/tests/test_tell_real.py` (`realserver.py`, the fake backend) | the same tell; Alice's and Carol's seats consult | Alice's logged request has "Told me: «an early bird»" in her L3 line for Bob; Carol's has no "early bird"; Alice's "People here" bracket for Bob (S30-f) has no chronotype text |
| `clients/agent/tests/test_tell_real.py` (`realserver.py`) | Bob's agent: `act {kind: tell}`; Alice's agent: `events` | `accepted`; Alice's `EventLine.told`; Bob's `me.attributes` marks `core.chronotype` tellable and `core.religion` not |
| `tools/cli/tests/client_2d_tell.rs` (`godot2d` harness) | Bob's 2D client chooses "Tell about myself…" → Chronotype | the server records the told fact |

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-i-1 (I-S30-10) | Every reference world reads, loads, runs and observes byte-identically | no reference world enables `attributes` (A-i-25); the existing world tests; one nightly `baseline check` on the final head |
| I-i-2 | Without attributes or told facts, every consumer is byte-identical: LM requests, cassettes, agent views, client panels, memory dumps (except `meta.schema`), L3 lines | `prompt_request.json`, the cassettes, `test_view.py` goldens, `s28b_parity.json` unchanged; M-i-6 |
| I-i-3 (I-S30-9; step-30 §9.5) | A told fact changes no world state: no component is written, no System reduces it | AC-i-8's state comparison; structural scan; M-i-3 |
| I-i-4 (I-S30-3) | Only `learnable` attributes `attributes` owns, with a value and a phrase, can be told; `private`, `hidden`, `perceivable`, undeclared and owners' are refused by name; `hidden` and undeclared alike | AC-i-3; M-i-1 |
| I-i-5 (charter) | No consumer or controller names an attribute id in code; ids come from data or from the block | AC-i-9's scan |
| I-i-6 (I-S28-1, I-S28-7) | A learned row exists only for a fact the seat perceived; nothing reads the world log or another seat's rows | the Carol and Dave cases; source scan; M-i-4 |
| I-i-7 (R-S30i-f-1) | A learned attribute never reaches `SelfAttributes`, `Person.description`, `PersonLine.description`, an inspect panel or an observation | AC-i-2; M-i-5 |
| I-i-8 | The value told is the teller's true value at the instant, never the request's | the payload has no value field; review |
| I-i-9 | Every rule of telling has one implementation (the model crate's `tellable`), read by `validate`, `offers` and `discloses` | review; the three paths tested against one oracle |
| I-i-10 (I-S30-8, I-S28-10) | No change under `kernel/`, `contracts/`, `server/` | `git diff --stat` in C10 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy `-D warnings` on `mineworld-attributes-model`, `mineworld-attributes`, `mineworld-rule-controller`, `mineworld-motivated`, `mineworld-cli`, `mineworld-acceptance`; `ruff` and `pyright` on `sdk/python`, `cognition/lm-controller`, `clients/agent`; Godot parse through the `godot` layer | per commit; CI `fast` |
| Unit (Rust) | `tellable`'s refusal order; schema 3's flag (holder only); the action's and fact's encoding; `validate` refusals; `rule`'s and `motivated`'s `tell` decoding and eligibility | `authoring/attributes/src/*/tests.rs`; `systems/attributes/tests/tell.rs` (new); `cognition/rule-controller/src/tell_tests.rs`; `cognition/motivated/src/tell_tests.rs` |
| Unit (Python) | the renderer; ingest writes and caps `learned`; the L3 line; the citable set; the host bound step; the schema-3 reader; the LM prompt lines and `tell` parsing; the agent's lines | `cognition/lm-controller/tests/test_memory_learned.py`, `test_controller_tell.py`; `sdk/python/tests/test_attributes.py`; `clients/agent/tests/test_view.py` |
| Unit (GDScript) | the composition golden at schema 3, `tellable` included | `clients/protocol/checks/attributes_check.gd` |
| Golden | the composition golden (hand-edited to schema 3, one new case); `prompt_request.json` unchanged; `prompt_request_tell.json` (new, hand-written); `s28b_parity.json` unchanged | as named |
| Integration (real binary) | the four tests of §5.10 | as §5.10 |
| Structural | AC-i-9's scans; AC-i-8's no-subscriber scan | `tests/acceptance/tests/attribute_consumers.rs` (extended) |
| Regression | I-i-1, I-i-2 | existing tests unchanged; one nightly dispatch |
| Gate 1 (real LLM) | NOT REQUIRED in this PR: the prompt and schema are deterministic text, compared byte for byte; whether a real model chooses `tell` sensibly is a milestone acceptance item (§11 R-MS-i-1) | — |
| Gate 2 (real lifecycle) | NOT REQUIRED beyond the real-binary tests and the memory kill/replay case (AC-i-10) | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-i-1 | **Checkpoint.** In world A, Bob tells Alice `core.chronotype`: Alice's and Bob's perceived streams each hold one `told-about-self` with `text` "an early bird"; Alice's next consult's logged request shows "Told me: «an early bird»" in her L3 line for Bob; Carol (street) perceives no such fact and her request contains no "early bird"; **Dave (same café, not addressed) perceives none either**. In world B, Mia tells Noor `home-port` and Noor recalls it, with no line of code naming `home-port` | S30-i row |
| AC-i-2 | **Never widens the observation.** Alice's observation entity for Bob, after the telling, carries the same `traits` entries as before (no `core.chronotype` id or text); her `Person.description` for Bob, Bob's `PersonLine.description` in Alice's MCP view and Alice's 2D glance are unchanged; her `SelfAttributes` is unchanged | S30-i row; R-S30i-f-1 |
| AC-i-3 | **What cannot be told.** Bob to Alice: `core.religion` → `not-learnable`; `core.stature` → `not-learnable`; `attributes-tell.secret` (hidden) and `core.no-such` → `unknown-attribute`, with identical refusal bytes apart from the request id; `employment.occupation` → `told-by-owner`; `attributes-tell.nickname` (no value) → `no-value`; Carol as target → the evaluator's place refusal; Alice 5 m away → `TooFarAway`; no fact is recorded for any of them. In world B, `core.chronotype` (narrowed to private) → `not-learnable` | S30-i row; I-i-4 |
| AC-i-4 | **The holder's flag.** Bob's own block marks exactly `core.chronotype` and `core.hometown` `tellable`; Alice's view of Bob has no `tellable` member anywhere; schema 3 is read by the Rust, Python and GDScript readers against one golden | Q-S30i-1 |
| AC-i-5 | **Every controller kind.** (a) `rule` with `tell: {attributes: [core.chronotype], per_mille: 1000}` tells Alice within one consult after she speaks to Bob; (b) `motivated` with `base: {tell: 1000}` chooses `Tell(Alice, core.chronotype)` and its explanation table names it; (c) the LM controller, given a fake answer `{"act":"tell","to":"Alice","attribute":1}`, sends `tell-about-self { attribute: core.chronotype }`, and a request in world B lists `Home port` as number 1 with no code change; (d) the MCP `tell` act is accepted; (e) the 2D client's chooser sends it | charter |
| AC-i-6 | **Nothing changes without attributes.** `prompt_request.json` equals the built request; every cassette replays; `test_view.py` goldens pass; `s28b_parity.json` is unchanged; the `decisions` golden's no-list case is unchanged with the `tell` block absent | I-i-2 |
| AC-i-7 | **Memory bounds.** 300 distinct tellings to one seat keep 256 `learned` rows, the oldest evicted; the host bound's `learned` step runs before `stable-facts`; after the told L0 record is forgotten the L3 line still cites `[#<told>]` and the host accepts the citation | §5.6.5 |
| AC-i-8 | **No knowledge state.** The world's component state digest after the telling equals the digest before it; `told-about-self` has no subscriber in any installed System's declaration (scan) | I-S30-9 |
| AC-i-9 | **Structural.** Over `cognition/*/src`, `sdk/python/src`, `clients/agent/src`, `clients/*/scripts`, `clients/*/mineworld`: no literal matching `core\.[a-z]`; over `kernel/`, `contracts/`, `server/`: no `tell-about-self`, `told-about-self`, `learned`, `tellable` | I-i-5, I-i-10 |
| AC-i-10 | **Replay.** A memory home rebuilt from Alice's perceived stream (kill, delete the home, resume) has the same `learned` rows and the same dump bytes | step-28 §6.5 |
| AC-i-11 | **Reference worlds.** The nightly `baseline check` on the final head passes with `scripts/baselines.txt` unchanged | I-i-1 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s30i`. Never `cd` elsewhere.

1. Re-read this commit's section and every file before editing it.
2. Make the steps in order; nothing of a later commit.
3. Run the commit's validation one `--test`/`--lib` or one `pytest` file at a time; record command, count, wall time and
   PASS / FAIL / INCONCLUSIVE. `cargo clippy -p <touched crates> --all-targets -- -D warnings`; `cargo fmt --all
   --check`; `uv run ruff check <touched package>`; `uv run pyright <touched package>`. **R-LOAD-1:** never `cargo test
   --workspace`, never a whole Python suite, never the nightly locally. Heavy jobs through
   `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
4. Mutations: copy the file, plant, run the named test red (record its name), restore from the copy, `cmp`, run green.
5. Answer the commit's review questions in §16 in one or two sentences each.
6. `git status --short`: every changed path is in the commit's **Files**; otherwise revert or stop and record why.
7. Update §16 and the handoff in the same commit; stage by path; commit with the message and the session's attribution
   line. No heredoc, no `sed -i`, no `python3 -c`, no `curl`.

#### C0 — Base and ledger

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30i.md` (new).
- **Goal:** start from recorded truth.
- [ ] Implementation: record the base; confirm S30-f and S28-c merged; re-read A-i-1 … A-i-26 and record every moved
  line; record whether S31-b2 and S30-g have merged (§11); apply D-i-8 (`VERSION`); confirm §15's rulings.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Review: the base satisfies §2.3; every ruling of §15 is applied to §§5, 8 before C1.
- **Commit message:** `docs(plan): S30-i — open the ledger at the base`

#### C1 — The action and fact types; schema 3 with `tellable`; three readers

- **Goal:** the vocabulary every later commit uses, and one wire shape read the same in three languages. One commit,
  because the server and the readers must move together or the real-server tests go red between commits.
- **Files:** `authoring/attributes/src/{facts.rs, render.rs, disclosed.rs, owner.rs, lib.rs, tell.rs (new)}` and their
  `tests.rs`; `authoring/attributes/tests/fixtures/composition.json`; `sdk/python/src/mineworld_sdk/attributes.py`,
  `sdk/python/tests/test_attributes.py`; `clients/protocol/mineworld/attributes.gd`,
  `clients/protocol/checks/attributes_check.gd`; `systems/attributes/tests/pack.rs` (expected payloads);
  `tools/cli/tests/{attributes.rs, attributes_consumers.rs}` and `tools/cli/tests/fixtures/attributes-consumers.expected.json`
  only where they spell `"schema": 2`.
- [ ] Implementation: §5.1 (`TellAboutSelf`, `TELL_RANGE`, `tell_requirement`), §5.2 (`ToldAboutSelf`), §5.3's
  `tellable` and `TellRefusal`; §5.5 (`Tells`, `SCHEMA = 3`, the flag in the holder view only); the three readers at
  schema 3 with `tellable(entries)`; the golden edit and its new case; measure the holder block (§5.5 Cost).
- [ ] Validation: `cargo test -p mineworld-attributes-model --lib`; `-p mineworld-attributes --test pack`;
  `-p mineworld-cli --test attributes`, `--test attributes_consumers`; `uv run pytest sdk/python/tests/test_attributes.py`;
  the GDScript check through `python3 scripts/ci_godot.py` if Godot is installed, else `NOT RUN` (CI's `godot` layer).
  Mutation **M-i-2** (`tellable` written in the observer view too) → the holder-only test red.
- [ ] Review: `disclose_owned`'s signature unchanged; an owner's entry is never tellable; `tellable`'s order matches
  §5.3 exactly; no reader evaluates visibility.
- **Commit message:** `feat(attributes-model): tell-about-self and told-about-self; the holder's block says what may be told (schema 3, ARC-127, S30-i)`

#### C2 — The `attributes` pack provides `tell-about-self`

- **Files:** `systems/attributes/src/{system.rs, tell.rs (new), lib.rs}`, `systems/attributes/tests/tell.rs` (new),
  `systems/attributes/README.md` (the declaration table), `Cargo.lock` if needed.
- [ ] Implementation: declaration (`providing`, `emitting`); `validate` and `resolve` (§5.3); `offers` (§5.4);
  `discloses` passes `Tells::Accepted`.
- [ ] Validation: `cargo test -p mineworld-attributes --test tell`, `--test pack`, `--lib`. Cases: each refusal of §5.3
  in order (one case per row, each from a state where only that row fails); the fact's audience is exactly
  `{teller, listener}`; no component changes (the world's component digest before and after); an observer with nothing
  tellable gets no offer. Mutations **M-i-1** (accept `private`) → the `not-learnable` case red; **M-i-3** (`react`
  writes `Traits` on `told-about-self`) → the no-state case red.
- [ ] Review: no new dependency (D-i-3); the refusal order is the table's; `hidden` and undeclared give identical
  bytes; `told-about-self` is not biographical.
- **Commit message:** `feat(attributes): tell-about-self — a person tells a listener the true value of a learnable attribute (ARC-127, S30-i)`

#### C3 — The worlds, the oracle and the server checkpoint

- **Files:** `tools/cli/tests/fixtures/attributes-tell/**`, `tools/cli/tests/fixtures/attributes-tell-harbour/**`,
  `tools/cli/tests/fixtures/attributes-tell.expected.json` (hand-written), `tools/cli/tests/attributes_tell.rs` (new).
- [ ] Implementation: §5.10's worlds and oracle; the Rust real-server test of §5.10's first row.
- [ ] Validation: `cargo test -p mineworld-cli --test attributes_tell`; `target/debug/mineworld attributes check` on
  both worlds (exit 0). Mutation **M-i-4** (audience `Place`) → the Dave case red.
- [ ] Review: the oracle is hand-written; world B shares no code path that names its attribute; AC-i-1 (server half),
  AC-i-2 (observation half), AC-i-3, AC-i-8 each have an assertion.
- **Commit message:** `test(attributes): Bob tells Alice his chronotype; Carol and Dave do not hear; nothing private is told (S30-i)`

#### C4 — Memory: the renderer, `learned`, schema 7, the L3 line

- **Files:** `cognition/lm-controller/src/mineworld_cognition/memory/{render/__init__.py, render/attributes.py (new),
  ingest.py, learned.py (new), home/schema.py, home/view.py, retention/bound.py}`,
  `cognition/lm-controller/src/mineworld_cognition/compress/stable.py`, `memory/retrieve.py`;
  `cognition/lm-controller/tests/{test_memory_learned.py (new), capture_s28b_parity.py (the filter only)}`.
- [ ] Implementation: §5.6.1 … §5.6.6.
- [ ] Validation: `uv run pytest cognition/lm-controller/tests/test_memory_learned.py`, then `test_home_store.py`,
  `test_home_golden.py`, `test_home_summaries.py` (the parity golden, unchanged), `test_home_bound.py`,
  `test_inspect_scopes.py`, `test_home_kill.py`, one at a time; ruff; pyright. Mutations **M-i-5** (the L3 line also
  written into `Person.description`) → AC-i-2's Python case red in C5; **M-i-6** (an empty `Told me:` suffix) →
  `test_home_golden.py` red; **M-i-7** (the cap evicts the newest) → the cap case red.
- [ ] Review: one transaction per ingest; no read of another seat; the citable set; the parity filter strips only
  `learned`; schema 7 noted.
- **Commit message:** `feat(memory): what someone told me about themselves — a private learned row, shown in their L3 line (schema 7, S30-i)`

#### C5 — The LM controller tells, and the cognition checkpoint

- **Files:** `cognition/lm-controller/src/mineworld_cognition/controller/{situation.py, prompt.py, choice.py,
  validate.py, intent.py, log.py}`; `cognition/lm-controller/tests/{test_controller_tell.py (new),
  fixtures/prompt_request_tell.json (new, hand-written), test_tell_real.py (new)}`.
- [ ] Implementation: §5.7.1; the real-server test of §5.10's second row.
- [ ] Validation: `uv run pytest cognition/lm-controller/tests/test_controller_tell.py`, `test_controller_choice.py`
  (the base golden unchanged), `test_record_replay.py` (cassettes), `test_tell_real.py`, one at a time; ruff;
  pyright. Mutation **M-i-8** (the `tell` act offered when nothing is tellable) → `prompt_request.json` red.
- [ ] Review: base schema unchanged without a tell offer; old logs replay; every text through `_quoted`.
- **Commit message:** `feat(cognition): an LM seat may tell someone about itself; Alice recalls what Bob told her (S30-i)`

#### C6 — The MCP adapter

- **Files:** `clients/agent/src/mineworld_agent/{models.py, player.py, view.py, mcp_server.py}`,
  `clients/agent/tests/{test_view.py, test_tell_real.py (new)}`, `docs/reference/agent-tools.{json,md}`
  (regenerated), `clients/agent/README.md` (one line).
- [ ] Implementation: §5.7.2.
- [ ] Validation: `uv run pytest clients/agent/tests/test_view.py`, `test_mcp_tools.py`, `test_tell_real.py`, one at a
  time; ruff; pyright.
- [ ] Review: every world string is `WorldText`; omitted when empty; the reference diff is the kind and two fields.
- **Commit message:** `feat(agent): tell as an act; my tellable attributes; what I was told in my events (S30-i)`

#### C7 — `rule` and `motivated`

- **Files:** `cognition/rule-controller/{Cargo.toml, src/tell.rs (new), src/tell_tests.rs (new), src/model.rs,
  src/paced.rs, src/lib.rs}`; `cognition/motivated/src/{parameters.rs, parameters_tests.rs, options.rs, model.rs,
  explain.rs, tell_tests.rs (new)}`; `Cargo.lock`.
- [ ] Implementation: §5.7.3, §5.7.4.
- [ ] Validation: `cargo test -p mineworld-rule-controller --lib`; `-p mineworld-motivated --lib`; `-p mineworld-cli
  --test decisions` (only the no-list case; the pre-existing failure recorded, A-i-26). Mutation **M-i-9** (the draw
  taken even when `tell` is absent) → the parity case red.
- [ ] Review: no attribute id literal in `src/`; absent block takes no draw; refusals keep line and column.
- **Commit message:** `feat(controllers): rule and motivated tell what their policy lists, when someone has spoken to them (S30-i)`

#### C8 — Both clients

- **Files:** `clients/protocol/mineworld/attributes.gd` (`tellable`); `clients/2d/scripts/{intents.gd, menu.gd,
  hud/words.gd}`; `clients/3d-spike/scripts/slice/{offer_menu.gd, intents.gd}`; the shared words file;
  `tools/cli/tests/client_2d_tell.rs` (new); the 2D harness step it needs.
- [ ] Implementation: §5.7.5.
- [ ] Validation: the GDScript check; `cargo test -p mineworld-cli --test client_2d_tell` (Godot on this machine, else
  `NOT RUN`, CI's `godot` layer); `cargo test -p mineworld-acceptance --test client_rules`.
- [ ] Review: no rule word or action decision in client code (`client_rules.rs`); the chooser lists only `tellable`
  lines; 2D and 3D build the same payload.
- **Commit message:** `feat(clients): tell about myself from the person menu, in 2D and 3D (S30-i)`

#### C9 — Specification sync

- **Files:** `docs/MODULE_SPEC.md`, `docs/decisions/ARC-127.md` (new), `docs/decisions/ARC-120.md`, `ARC-122.md`,
  `ARC-109.md` (dated notes), the index regenerated by `python3 scripts/check_decision_ids.py --write-index`,
  `sdk/python/README.md` (one line), `tests/acceptance/tests/attribute_consumers.rs` (AC-i-9's scans).
- [ ] Implementation: `MODULE_SPEC` §4.1 (the action, the fact, schema 3), §5.3 (the person scope's private layer
  lists `learned`); `ARC-127` with Problem, Options (§15's), Choice, Consequences, Revisit (F-S30i-6, D-i-2); notes.
- [ ] Validation: `python3 scripts/ci_layer.py docs`; `cargo test -p mineworld-acceptance --test attribute_consumers`.
- [ ] Review: every finding of §3.2 that changed specified text has its note (`CLAUDE.md` §2.1 rule 4).
- **Commit message:** `docs(spec): telling about oneself, schema 3 and the learned row as implemented (ARC-127, S30-i)`

#### C10 — Close-out

- **Files:** this file (§16), the handoff.
- [ ] Implementation: AC-i-9's scans; `git diff --stat origin/main...HEAD -- kernel contracts server` empty; push
  `mvp0/pr-s30i`; `gh pr create`; CI on the exact head green; one `nightly.yml` dispatch for AC-i-11.
- [ ] Validation: CI and the nightly verdict, classified PASS / FAIL / INCONCLUSIVE.
- [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-i close-out`
- **Then:** reported MERGE-READY to the coordinator — DO NOT MERGE.

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time; `cargo clippy` on the
touched crates; `ruff` and `pyright` on the touched Python packages. Never `cargo test --workspace`, never a whole
Python suite, never the nightly locally. The PR's CI on the exact head is the gate; one `nightly.yml` dispatch on the
final head is the only long run.

---

## 11. Requirements this PR places on other PRs

- **R-S30g-i-1 (S30-g).** A reference world that enables `attributes` gains one `tell-about-self` affordance per
  perceived person and schema-3 blocks. If S30-g merges before S30-i, S30-i re-baselines those worlds once with this
  reason; if after, S30-g's re-baseline includes them. S30-g may add `tell:` blocks to reference worlds' decision
  lists; nothing requires it.
- **R-S30k-i-1 (S30-k).** The fact carries `label` and `text` rendered by the server. S30-k decides the locale they are
  rendered in (the teller's world locale), and the memory line's fixed words (`told me`, `Told me:`) become label-pack
  words.
- **R-S30h-i-1 (S30-h).** MatrAIx dimensions marked `learnable` become tellable with no code; the demo may show it.
- **R-owner-i-1 (any owner, for example `employment`).** An owner that wants its learnable attribute told provides its
  own action, using the model crate's `tellable` rule and `ToldAboutSelf` fact type with its own `OWNER`, and passes
  `Tells::Accepted` from its own `discloses` (Q-S30i-3).
- **R-MS-i-1 (the milestone acceptance).** One bounded real-LLM check: an LM seat with something tellable, spoken to,
  chooses `tell` at least once in a short run, and never names an unlisted number.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §9.5 | "a structured `stable` fact" → the private row `learned`, shown in the subject's L3 line (F-S30i-5); the audience is the participants (Q-S30i-2); owners' attributes are told by their owners (Q-S30i-3) |
| `step-30-person-attributes.md` §13.1, S30-i row | scope adds "the holder's `tellable` flag (schema 3); every controller kind and both clients" |
| `step-30-person-attributes.md` §4.2 item 2 | "The pack provides no action" → "one action, `tell-about-self` (S30-i)" |
| `step-28-shared-memory.md` §4.4 | the private layer's tables gain `learned` (schema 7) |
| `pr-s30f-attribute-consumers.md` §5.1 | a forward note: schema 3 since S30-i |
| `overall.md` | S30-i drafted / frozen / merged |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-i — learnable attributes: tell-about-self, the told fact, and the
                         listener's learned row
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30i-learnable-attributes.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§5.4, 9.5, 11.1, 12, 13); step-28-shared-memory.md (§§3, 4.4,
                         6, 7, 10); pr-s30c-attributes-pack.md; pr-s30f-attribute-consumers.md; pr-s28a … pr-s28e;
                         docs/MODULE_SPEC.md §§4.1, 5.2, 5.3; docs/decisions/ARC-109, ARC-120 … ARC-126, ARC-153;
                         CLAUDE.md; docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md;
                         .structured-coding/standards.md; the lane-lead charter
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30i, its own, held by one session
BRANCH:                  mvp0/pr-s30i, created from origin/main
START CONDITION:         S30-f and S28-c merged (both are) AND this design frozen AND §15 ruled, then the
                         coordinator's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–10, as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-1, I-S30-3, I-S30-5, I-S30-8, I-S30-9, I-S30-10; step-28 I-S28-1, I-S28-2,
                         I-S28-7, I-S28-10; §6 I-i-1 … I-i-10; §8 as written
SEQUENCE:                C0 … C10 (§9)
IMPLEMENTER:             Opus-class session; C6, C7, C8 may be delegated to Haiku-class sub-agents from §9, each
                         reviewed by the session before its commit
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *;
                         uv run pytest <one file>; uv run ruff; uv run pyright; python3 scripts/ci_godot.py;
                         python3 scripts/check_doc_headings.py, check_decision_ids.py, ci_layer.py docs,
                         ci_parity.py baseline check; tools/reference regeneration; git; gh (pr create/view; workflow
                         run nightly.yml once in C10; run download; never merge); cp; cmp; mkdir -p; sed -n; grep;
                         heavy jobs through cast-slots/with-slot.sh
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace; a whole Python
                         suite; any edit under kernel/, contracts/, server/, worlds/, or systems/<pack> other than
                         attributes; any model call; a new external dependency
MATERIAL STOPS:          a reference world's bytes moving while no reference world enables attributes; the prompt
                         golden, a cassette or s28b_parity.json moving for a world without attributes; a needed change
                         to another System; a needed kernel, contracts, server or protocol change; S31-b2 merged with a
                         `disclose_owned` incompatible with §5.5 beyond a parameter name
PLATFORMS:               Linux, macOS, Windows (CI core and godot layers on all three)
VALIDATION BUDGET:       crate, Python and client tests unrestricted, one file at a time; one nightly dispatch (C10).
                         Gate 1 and Gate 2: NOT REQUIRED (§7)
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30i.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once frozen (source: the operator's freeze)
                         semantic commits: authorized      (working rules §14)
                         branch push: authorized           (working rules §21)
                         PR creation / update: authorized  (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: MERGE-READY, reported to the coordinator (charter, SPEED FIRST 2026-10-18)
                         merge: never by the implementation session — the coordinator merges
POST-MERGE SYNC OWNER:   primary: step-30, step-28, pr-s30f's forward note, overall.md; implementation session: this
                         ledger and the merge identity
STOP CONDITION:          S30-i READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-i-1 | Schema 3 collides with a concurrent PR that reads or writes blocks (S31-b2, S30-g, S30-h, S30-k) | `disclose_owned` keeps its signature; §11 names each PR's obligation; C0 records what merged |
| R-i-2 | Memory schema 7 forces every home to rebuild | the established rule (QS28-8); S28-b and S28-d did the same; the parity golden is unchanged by its filter |
| R-i-3 | A world's phrase forges a prompt or memory boundary | `quote` in memory, `_quoted` in the prompt; a test plants `»` and `<<<end>>>` in a phrase |
| R-i-4 | A rarely met person's told facts are cut from L3 in a crowd | the gist stays in L1/L0 and FTS; a later scope may add a "what people told me" section; recorded in `ARC-127` Revisit |
| R-i-5 | Controllers tell the same thing over and over | `rule` and `motivated` tell only after the listener spoke within `within`, by a draw; repeats are harmless (the row upserts) |
| R-i-6 | A three-metre compiled range disagrees with a world's longer `talk` range | recorded (D-i-2); an Interaction List for `attributes` is later work (F-S30i-6) |
| R-i-7 | Telling becomes a leak of private data through refusals | `hidden` and undeclared refuse identically; refusals go only to the teller, who holds the value |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30i-1 [the wire; `MODULE_SPEC` §4.1; S30-f's schema 2]** | A controller must know which of its own attributes it may tell. The block carries no visibility (F-S30i-1). How does it learn? | **Schema 3: the holder's own entries carry `tellable: true` where the pack accepts `tell-about-self`**; another observer's view never carries it. The server states the answer, so no client or controller evaluates visibility. Cost: one field, holder view only, three readers moved in one commit. Declined: (b) one complete offer per attribute (≈ 660 affordances in a café, F-S30i-2); (c) incomplete offers and no list, so controllers guess and are refused |
| **Q-S30i-2 [audience of `told-about-self`]** | Who perceives the told fact? `spoke` is overheard by the place; step-30 calls it "a conversation-audience fact". | **The participants only: the teller and the listener.** A told fact makes a durable memory row; only the person addressed should get one. Words said aloud stay `talk`'s and keep its audience. Alternative: the place, like `spoke` (a bystander remembers too), which would also need an Interaction List `attributes` cannot have yet (A-i-20) |
| **Q-S30i-3 [owners' attributes]** | `employment.occupation` is `learnable`, but `attributes` cannot read `employment`'s projection without depending on it (F-S30i-3). | **In this PR only attributes `attributes` owns are tellable; an owner's is refused `told-by-owner`.** The model crate exports the rule and the fact type, so an owner adds its own telling in its own PR (R-owner-i-1). Alternative: `attributes` depends on every owner (breaks single ownership and `ARC-120`'s note of 2026-10-18) |
| **Q-S30i-4 [where the learned row lives]** | The row must survive a rebuild, but an optional scope never sees the payload (F-S30i-4). | **A private table `learned` in the person scope, written at ingest like names, memory schema 7, shown inside the subject's L3 line.** Matches step-30 §9.5's "S28's private layer". Cost: every home rebuilds once (QS28-8); no golden moves. Declined: an optional scope (cannot rebuild); a new prompt section (takes bytes from the 6 000-byte split and moves every request) |
| **Q-S30i-5 [controllers and clients in this PR]** | The brief asks how each controller kind chooses the action. Doing all of them makes the PR larger. | **All four kinds and both clients in this PR** (§5.7), with C6–C8 delegable to Haiku. The charter wants each kind to decide by its own model, and humans are players too. Alternative: the LM controller and MCP now; `rule`, `motivated` and the clients in a follow-up S30-i2 (both clients then show the offer as "not supported by this client") |
| **Q-S30i-6 [order with S30-g]** | If S30-g enables `attributes` in reference worlds first, this PR moves their observations (offers, schema 3). | **Prefer S30-i before S30-g.** If S30-g lands first, S30-i re-baselines those worlds once, with this reason, and that is not a material stop (R-S30g-i-1) |

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base and ledger | | [ ] | [ ] | [ ] |
| C1 types, schema 3, readers | | [ ] | [ ] | [ ] |
| C2 the pack's action | | [ ] | [ ] | [ ] |
| C3 worlds, oracle, server checkpoint | | [ ] | [ ] | [ ] |
| C4 memory | | [ ] | [ ] | [ ] |
| C5 LM controller and cognition checkpoint | | [ ] | [ ] | [ ] |
| C6 MCP adapter | | [ ] | [ ] | [ ] |
| C7 `rule` and `motivated` | | [ ] | [ ] | [ ] |
| C8 both clients | | [ ] | [ ] | [ ] |
| C9 specification sync | | [ ] | [ ] | [ ] |
| C10 close-out | | [ ] | [ ] | [ ] |

### Evidence

None yet.

### Mutations

None planted yet. Planned: M-i-1 … M-i-9 (§9).

### Deviations and findings

F-S30i-1 … F-S30i-6 (§3.2), recorded at design time.

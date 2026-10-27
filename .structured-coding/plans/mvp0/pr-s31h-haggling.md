# PR S31-h — Haggling: a buyer offers a price, the seller accepts or declines, and acceptance settles at once

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 2 (2026-10-19, lane lead: the charter's SPEED FIRST block — the coordinator
                        merges; per-PR validation is the build and the commit's own tests; the property
                        test at 10 000 states, the 30-day AC-h-10 runs, the client shots and the planted
                        mutations run at the milestone), DRAFT; revision 1 (2026-10-18)
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31h, branch docs/pr-design-s31h,
                        planning base origin/main @ a1e05874 (#273; step-31 APPROVED, S31-a and S31-b
                        DESIGN FROZEN, neither implemented)
Approved by / evidence: none yet. The step's rulings (Q-S31-1 … Q-S31-8, step-31 §18) and S31-a's
                        Q-S31a-3 (accept and decline are offered incomplete) bind and are not re-asked.
                        §14 holds this PR's questions, each with a recommendation
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S31-a,
                        S31-b, S31-d, S31-e, S31-g and S27-d1 (#261)
Start condition:        step-31 §16.5: "Nothing starts before Milestone F's checklist has been handed to
                        the operator (the S29 timing rule). Each PR starts only once its own PR design is
                        frozen." Then S31-a merged (NOT met at 2026-10-18: frozen, not implemented), then
                        this design frozen (NOT met), then the primary session's dispatch. S31-h is in the
                        second wave, beside S31-c, S31-e and S31-g. It is not gated on S31-e, S31-g or
                        (Q-S31h-2) S31-d
Execution contract:     §12 (filled, endpoint authority included; ends at MERGE-READY; the coordinator merges)
Implementer:            Opus-class (step-31 §16.2, the S31-h row: "economy state with expiry, re-validation
                        at acceptance"); reviewer the lane lead plus an Opus logic review of C4
Lifecycle:              FROZEN 2026-10-20 (§14.1); IMPLEMENTING on mvp0/pr-s31h from origin/main @ fec906f2
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 and Q-S31-6 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§5.2 (last paragraph), 5.4, 5.5,
5.7, **5.9 in full**, 10.1, 11, 12, 13, 14 (I-S31-1 … I-S31-5, I-S31-9, I-S31-10, I-S31-12), 15 (AC-S31-19,
AC-S31-20; X-13, X-14), 16.2 (the S31-h row), 16.5, 19.4 (`ARC-118`) · **Parent:** [`overall.md`](overall.md) ·
**Builds on:** [`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN 2026-10-18; §5.7 settlement, the
property test, R-S31h-a-1) · **Coordinates with:** S31-d (the `pricing:` block, designed in parallel at
`/Users/yuema137/mineworld-worktrees/design-s31d`), S31-e (the sale panel, `design-s31e`), S31-g (the agent view,
`design-s31g`), [`pr-s31b-notices.md`](pr-s31b-notices.md) (FROZEN; the board record a haggling buyer reads),
`pr-s27d1-single-act.md` (FROZEN and merged in #261 after the planning base: one `act` tool, `ARC-153`) ·
**Followed by:** S31-f (Market Town's haggling sellers and buyers, AC-S31-21) · **Binding parents:** `CLAUDE.md`
§§2, 3, 4 (items 1, 2, 3, 7, 8, 9, 10, 12, 13, 15); `docs/CORE_CONCEPTS.md` §§12, 15.2; `docs/MODULE_SPEC.md`
§§4.1, 5.1; decision records `ARC-25`, `ARC-26`, `ARC-34`, `ARC-38`, `ARC-43`, `ARC-112` (S31-a), `ARC-153`.

**Decision ids.** This PR writes **`ARC-118`**, assigned by the step (step-31 §19.4: "Haggling settles at
acceptance without escrow"). It is free on `origin/main @ 94183574`: `docs/decisions/` holds no `ARC-118.md`
(`git ls-tree --name-only origin/main docs/decisions/`). #258 split the log into one file per record after this
design's planning base, so C1 writes `docs/decisions/ARC-118.md` and regenerates the index with
`python3 scripts/check_decision_ids.py --write-index`. **No new id is needed:** the proposal identity, the decline
cooldown and the composition-only re-baseline are all parts of the one haggling decision. `ARC-189` stays unused.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Operator principle (2026-10-18, relayed with this task):** "Any seller sets its own price, and bargaining
  (haggling) exists." S31-a gives the first half. This PR gives the second.
- **Operator release rule (2026-10-15, step-31 §18.1):** "我们要发布就得把这些功能全部做好" — *"If we are going
  to release, all of these features have to be done properly."* Haggling is in MVP-0 and blocks the release.
- **Scope (step-31 §16.2, the S31-h row, verbatim).** "§5.9: `Proposals`, the four actions, the events and codes,
  offers and disclosure; the `haggle` and `accept_at_least_permille` options of the `pricing:` block; the
  sale-panel rows (2D and 3D) and the four agent tools; `ARC-118`; no Market Town content (its haggling sellers
  and buyers are S31-f content, so the two-re-baseline ruling holds)." Depends on "S31-a; S31-e and S31-g for the
  client and agent rows (or they land in S31-h if those merge first)". Checkpoint "fixture world: AC-S31-19,
  AC-S31-20". Adversarial "X-13, X-14". Implementer **Opus**.
- **The core rule (step-31 §5.9).** "Haggling is a buyer's counter-offer that a seller accepts or declines.
  **Acceptance settles exactly like `buy`, at that instant, with every check re-run (§5.7).** Nothing is reserved
  at any time, so there is no escrow."
- **Q-S31a-3 (ruled 2026-10-15; step-31 §5.9 dated note).** "`accept-proposal` and `decline-proposal` are offered
  **incomplete**, so a generic controller never accepts or declines at random. S31-h's design applies this." The
  principle behind it: "A complete offer must be safe for a controller to take at random" (step-31 §5.6 note).
- **R-S31h-a-1 (S31-a §10).** "`accept-proposal`/`decline-proposal` are offered incomplete …; acceptance reuses
  `judge_sale` steps 5–9 and `settle`."
- **Order with the clients and the agent (step-31 §16.5).** "S31-h is not gated on S31-e or S31-g. If they merge
  first, S31-h adds its own sale-panel rows and agent tools. If S31-h merges first, S31-e and S31-g include them."
- **Two re-baselines (Q-S31-3, ruled).** Market Town is re-baselined twice in S31: S31-a (vocabulary) and S31-f
  (content). §16.5: "S31-h changes no reference world." §3.2 F-S31h-1 shows that this sentence does not hold for
  the journal digest, and Q-S31h-5 asks how to keep the ruling's intent.
- **Speed-first charter (operator, 2026-10-18).** Mechanisms are generic and data-driven, with no world-specific
  hard-coding. Every acceptance criterion names at least one second world or a generated variant (§8).

---

## 1. Goal

A buyer who thinks a price is too high can say so. Today the only answers are "buy at the asking price" or "walk
away". After this PR the buyer may **propose** a lower (or higher) price for one unit. The seller sees the
proposal and **accepts** or **declines** it. The buyer may **withdraw** it. A proposal lasts 15 world minutes.

Acceptance is a sale. At the instant the seller accepts, economy runs every check a `buy` runs, at the proposed
price, and states the sale's facts in one dispatch: the money and the item move together, or nothing moves.
Between the proposal and the acceptance nothing is reserved. The buyer may spend the money and the seller may give
the item away. Then acceptance is refused by name.

Example. Grace asks 300 for bread. Bob stands 2 m from her and proposes 250. Economy records the proposal and tells
Bob and Grace, nobody else. Five minutes later Grace accepts. Economy checks that Bob still has 250, that Grace
still holds bread, that Bob can still carry it, and that Bob is still within 3 m. All hold, so one revision holds
`proposal-accepted`, `sold { price: 250 }`, `money-transferred { 250 }` and `items-transferred`. Carol, standing
beside them, sees the sale (sales are seen by the place) but never saw the proposal.

Nothing changes in `kernel/` or `contracts/` (I-S31-1). No reference world gains content.

---

## 2. Scope

### 2.1 In scope

1. **`systems/economy`** (the only crate whose rules change):
   - `src/proposal.rs` (new): `ProposalId`, `Proposal`, `ProposalStatus`, the `Proposals` component, the
     constants `PROPOSAL_VALID_FOR` and `PROPOSALS_PER_SELLER`, and the `proposals` record (§5.3, §5.9).
   - `src/haggle.rs` (new): `ProposePrice`, `AcceptProposal`, `DeclineProposal`, `WithdrawProposal`; the judges
     `judge_proposal` and `judge_answer`; the facts each action states (§5.4 … §5.7).
   - `src/sale.rs` (S31-a's): `judge_sale` split into the seller-resolving half and `judge_exchange`, so that
     acceptance runs S31-a's steps 5–9 through the same code (§5.6). `settle` takes a `Sale` value that names the
     seller, the buyer, the item, the price and the place.
   - `src/event.rs`: `PriceProposed`, `ProposalDeclined`, `ProposalWithdrawn`, `ProposalAccepted`.
   - `src/offer.rs`, `src/system.rs`, `src/codec.rs`, `src/lib.rs`: declaration, `validate`, `resolve`, `react`,
     `offers`, `discloses` for the above; `SystemVersion` raised by one (§5.11).
   - `tests/`: new `haggle.rs`, `haggle_settlement.rs`, `haggle_persisted.rs`; `support/mod.rs` gains Grace and
     the second composition (§8); `Cargo.toml` gains `mineworld-persistence` and `mineworld-item-transfer` as
     dev-dependencies (workspace crates, the `inventory/tests/persisted.rs` precedent; no external crate).
   - `README.md`: the four actions in the pack table.
2. **`tools/cli/tests/market/mod.rs`**: the mirror learns the four new facts, and the I-S31-3 sale scan counts a
   sale caused by `accept-proposal` (§5.6). No content and no run changes.
3. **The NPC options** (conditional, Q-S31h-2): the `haggle` and `accept_at_least_permille` options of S31-d's
   `pricing:` block, and the two behaviours behind them in S31-d's shared `price` module (§5.12).
4. **The agent rows** (conditional on S31-g, step §16.5): `MeLine.proposals`, the four `act` kinds (Q-S31h-3),
   the four facts read by name in the inbox (§5.14).
5. **The client rows** (conditional on S31-e, step §16.5): the sale panel's *Offer a price…* row, the buyer's
   open-offer row with *Withdraw*, the seller's incoming-offer row with *Accept* and *Decline*, and the refusal
   words, in 2D and 3D (§5.13).
6. **`scripts/baselines.txt`**: the composition-only re-baseline of Market Town, if Q-S31h-5 is answered as
   recommended (§5.16).
7. **Documentation**: `docs/decisions/ARC-118.md` and the regenerated index; `docs/MODULE_SPEC.md` §4.1's economy
   row gains the four actions; the step's §5.9 amendments listed in §15 are proposed, not made here.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Market Town's haggling sellers and buyers, AC-S31-21 | S31-f |
| The `pricing:` block itself, its other policies, the reporter model | S31-d |
| The sale panel, the stall, the price field and its major-unit conversion, the shared formatter | S31-e (this PR adds rows to them) |
| `MeLine.wallet/asks/wealth`, the act kinds for S31-a's seller actions and S31-b's notices | S31-g (this PR adds its four kinds the same way) |
| An LM seat haggling | the PR that gives the LM controller attempts of incomplete offers (Q-S31h-4) |
| Remote haggling (a proposal to a seller who is not present) | out of scope, as remote sales are (step §5.4) |
| Haggling over more than one unit | out of scope: `buy` is one unit (S31-a §5.7, `count: 1`) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/inventory`, `systems/item-transfer` | never in this PR (I-h-1) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over (operator); S31-a merged. S31-h edits S31-a's
  `sale.rs`, `offer.rs` and `system.rs`, so it cannot start from S31-a's design alone.
- **Either order, rule "whichever merges second" (step §16.5):** S31-e and S31-g. C7 (agent) and C8 (clients)
  are done here only if that PR merged before C0; otherwise they are N/A here, and §10's R-S31e-h-1 and
  R-S31g-h-1 bind that PR.
- **Either order, the same rule extended to S31-d (Q-S31h-2):** C6 (NPC options) is done here only if S31-d
  merged before C0; otherwise R-S31d-h-1 binds S31-d.
- **No dependency on S31-b, S31-c or S30.** A haggling buyer reads a current suggestion from S31-b's board record
  when it exists, and the reference value otherwise (§5.12); without either it never haggles.

---

## 3. Audit (`origin/main @ a1e05874`; two anchors from `origin/main @ 94183574`, marked)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | Economy today: `validate` (buy in a shop), `resolve` (`pay` then `inventory::transfer`, a refusal is `FactRefusedByOwner`), `react` (the only writes of `Wallet` and `Shop`), `discloses` (wallet to the holder only) | `systems/economy/src/system.rs:88-109, 113-137, 145-198, 241-263` | S31-a rewrites these. S31-h adds beside S31-a's code, in new files, and edits `sale.rs` once (§5.6) |
| A-2 | A payment to two people is `Visibility::Participants` with `.about(…)` and `.with_participants(…)` | `systems/economy/src/system.rs:203-226` (`answer`) | The proposal facts use exactly this shape (§5.7) |
| A-3 | Presence's audience rule: `Participants` admits exactly the envelope's participants; `Place` admits the place's occupants **and** the participants and subjects | `systems/presence/src/audience.rs:104-117` | A proposal is `Participants` (X-14 plants `Place`). A third person in the place sees `sold` but never `price-proposed` |
| A-4 | Every request has an `ActionId`, allocated by the world, "the key its answer and its events refer back to"; `ActionId(u64)` is opaque, ordered, journaled | `contracts/src/action.rs:650-653`; `contracts/src/ids.rs:335-347` | The proposal's identity is the `ActionId` of the `propose-price` request: no counter, no new state (SD-S31h-1) |
| A-5 | `WorldRead::at()` and `WorldView::at()` give the world time at a read | `kernel/src/view.rs:94, 258` | Expiry is lazy: every judge and every disclosure compares with `at()` (§5.10) |
| A-6 | `Offer::new` is incomplete (no payload); `Offer::complete` carries one; `refused(reason)` is reported before the requirement; `offers(world, observer, target)` is asked per target | `systems/presence/src/interaction.rs:154-216` | Every haggling offer is `Offer::new`, targeted where the action has a target (§5.8) |
| A-7 | `SpatialRequirement::evaluate(actor, target, available)`: availability, then place, then range; `SamePlaceAsActor` and a range need a target, else `PreconditionFailed` | `contracts/src/spatial.rs:470-525` | `accept-proposal` takes the **buyer** as its target, so "the buyer walked away" is `TooFarAway` from the requirement itself (SD-S31h-3) |
| A-8 | The paced rule attempts only **complete** available affordances (20 % of consults); `motivated` lists only complete available ones as choices | `cognition/rule-controller/src/offered.rs:26-52`; `cognition/motivated/src/options.rs:285-289` | Incomplete haggling offers move no NPC in any world that does not configure haggling. Market Town's behaviour cannot move (AC-h-10) |
| A-9 | The LM controller builds a request only for `Say` (a `talk`) or `Attempt` (a complete affordance, unchanged) | `cognition/lm-controller/src/mineworld_cognition/controller/intent.py:27-56` | An LM seat cannot propose, accept or decline today: all four offers are incomplete (F-S31h-3, Q-S31h-4) |
| A-10 | **(94183574)** The MCP surface is nine tools; the acts are one `act` tool with five `kind`s; "every other action is still reached only through `attempt` on a complete offer"; QS27-9 still refuses a generic free-form act | `docs/decisions/ARC-153.md`; `clients/agent/src/mineworld_agent/mcp_server.py:143-160` | The step's "four agent tools" would be four new tools against `ARC-153`. They become four typed `act` kinds (F-S31h-4, Q-S31h-3) |
| A-11 | A save stores each system's declaration (`SystemDeclaration`: version, owned, emitted, subscribed) in its manifest and snapshots; the genesis journal row carries the composition, and a declaration change moves `journal-300s` while facts and summaries stay | `persistence/src/format.rs:25-48`; `scripts/baselines.txt:8` (FG-a's reason line) | Adding four facts, a component and a version to economy moves Market Town's `journal-300s` and `sealed-300s`, with no behaviour change (F-S31h-1, Q-S31h-5) |
| A-12 | The persisted-world test pattern: a real SQLite save, actions, drop, resume a freshly composed world, compare | `systems/inventory/tests/persisted.rs:1-60` | `haggle_persisted.rs` follows it (AC-h-8) |
| A-13 | Economy's dev-dependencies already include `mineworld-rule-controller` (the paced test) and the item pack | `systems/economy/Cargo.toml` `[dev-dependencies]` | The NPC fixture test runs the real rule model in economy's tests, as `paced.rs` does |
| A-14 | `mineworld-rule-controller` does not depend on economy today | `cognition/rule-controller/Cargo.toml` `[dependencies]` | S31-d adds that dependency for `set-price` (step §10.1, A-20 of the step). C6 builds proposal requests through economy's published types on top of it |
| A-15 | S31-a's `judge_sale` order: 1 buyer, 2 seller, 3 ask, 4 price equal (validate only), 5 SelfTrade, 6 funds, 7 stock, 8 carry, 9 the owners' own `admit_payment` and `admit_transfer`; then the requirement | `pr-s31a-currency-asks.md` §5.7 | Steps 5–9 become `judge_exchange`, shared by `buy`, `propose-price` and `accept-proposal` (§5.6) |
| A-16 | S31-a's `for: shop` authority: a keeper standing in a shop at its place; outside it, no offer and `NoSupportedInteraction` | `pr-s31a-currency-asks.md` §5.6 step 2, C4 validation | A keeper answers a shop's proposal only while standing in that shop, by the same rule (§5.5) |
| A-17 | S31-a's sale scan `sales_are_atomic(facts, journal)` counts sales caused by `buy` | `pr-s31a-currency-asks.md` §6 I-a-3, C5 step 7 | It must also count sales caused by `accept-proposal`, or S31-f's haggling would read as a defect (C5) |
| A-18 | The 2D menu lists incomplete affordances as disabled raw entries until a client gives them words | `clients/2d/scripts/menu.gd:141-180`; `pr-s31a-currency-asks.md` R-S31a-3 | If C8 waits for S31-e, the four offers appear disabled in 2D meanwhile; nothing is sent |

### 3.2 Findings

- **F-S31h-1 — S31-h does move a Market Town digest.** Step §16.5 says "S31-h changes no reference world", so that
  the two-re-baseline ruling (Q-S31-3) holds. Market Town enables economy. Its saves record economy's declaration
  (A-11), and this PR adds four emitted facts, one owned component and one version. So `journal-300s` (genesis
  row) and `sealed-300s` move. No fact and no summary moves, because no Market Town controller ever attempts an
  incomplete offer (A-8) and no proposal exists there. The nightly would go red after this PR merges alone. →
  **Q-S31h-5**.
- **F-S31h-2 — A complete `withdraw-proposal` would undo haggling at random.** The step offers it complete. Under
  A-8, a buyer NPC with one open proposal would withdraw it on about one consult in five. That breaks AC-S31-21
  (S31-f) and contradicts the principle ruled in Q-S31a-3. → **Q-S31h-1** (recommended: incomplete).
- **F-S31h-3 — An LM seat cannot haggle yet.** Step §5.9: "An LM decides in language from the same offers." The
  LM controller attempts only complete offers (A-9), and all four haggling offers are incomplete by ruling. S31-a's
  seller actions and S31-b's `publish-notice` meet the same gap, and S31-d's LM reporter must close it for
  `publish-notice`. → **Q-S31h-4**.
- **F-S31h-4 — "Four agent tools" conflicts with `ARC-153`.** After the planning base, #261 merged one `act` tool
  with a closed set of kinds (A-10). Four new tools would undo that ruling. Four new typed kinds keep it, and
  QS27-9 is not touched because each kind is typed, not a free-form action. S31-g meets the same question for six
  actions. → **Q-S31h-3**.
- **F-S31h-5 — A stateless buyer would propose forever.** A decision model is a pure function of its observation
  (`MODULE_SPEC.md` §5.1). If a declined proposal simply vanished, a `haggle` buyer would propose again at the
  next consult, be declined again, and never buy. The decline must stay visible to the buyer for a while. →
  **Q-S31h-6** (recommended: a declined proposal is kept, marked declined, until its own `valid_until`, and the
  same buyer cannot re-propose that kind to that seller before then).
- **F-S31h-6 — `SamePlaceAsActor` needs a target** (A-7). Step §5.9 gives `accept-proposal` the requirement
  "`SamePlaceAsActor` and within `TRADE_RANGE` of the buyer". With the payload `{ proposal }` alone there is no
  target. The request's target is the proposal's buyer (SD-S31h-3); the payload still names the proposal.
- **F-S31h-7 — The decline and withdraw facts need the seller to be reduced.** The step's payload
  `{ proposal }` would make economy search every seller's `Proposals` at reduction, and an inbox reader could not
  say who declined what. The four facts carry `{ proposal, seller, buyer, item, price }` (SD-S31h-6). This is
  the step's information-rich-observation rule (step §0.2, R-S27c-INFO).
- **F-S31h-8 — "An expired proposal … with no wake and no fact" leaves entries in state.** Lazy expiry (step §5.9)
  means an expired entry stays in the component until the next write on that seller. That is deterministic and
  bounded: every reduction on a seller first drops the entries whose `valid_until` has passed, and the count limit
  counts only live entries (§5.10).
- **F-S31h-9 — The step's `proposal-limit` covers two cases.** "More than 8 open on the seller, or a duplicate."
  Both are one code, as the step says; the `detail` names which (`seller-full` or `already-proposed`), so a client
  can word them differently with no new code.

### 3.3 What does not exist

No proposal, counter-offer or bargaining state, action, fact or offer anywhere (`git grep -n "propos" systems/
cognition/ clients/` finds only group-activity's invitations and the LM controller's own `Proposal` type, which
are unrelated). No sale panel and no `pricing:` block exist on the base either: they are S31-e's and S31-d's.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Negotiation protocols | FIPA Contract Net, agent-negotiation libraries (`negmas`, GENIUS) | **Not adopted.** They model multi-round, multi-issue negotiation between autonomous agents with their own state. MineWorld's haggle is one round, one issue (the unit price), decided by a System Pack at one instant under single ownership. Adopting one would move the rule out of the server-authoritative path (`CLAUDE.md` §4 rules 3, 16). Below the `DEP` threshold: nothing is adopted and no infrastructure is rebuilt |
| Escrow, two-phase commit | none | Declined by the step (§5.7 option (b)); nothing to adopt |
| Identity allocation | a counter in economy's state | **Not built**: the kernel's `ActionId` is already unique and journaled (A-4) |
| Property testing | `proptest` | Not adopted, as S31-a: the in-test SplitMix64 idiom (S31-a SD-S31a-10) |

---

## 5. Design

### 5.1 Module layout (`systems/economy/src/`, after S31-a)

```text
proposal.rs   ProposalId, Proposal, ProposalStatus, Proposals (component), PROPOSAL_VALID_FOR,
              PROPOSALS_PER_SELLER, live(), the `proposals` record                                  (new)
haggle.rs     ProposePrice, AcceptProposal, DeclineProposal, WithdrawProposal; judge_proposal,
              judge_answer, judge_withdrawal; their facts; the offers of §5.8                       (new)
sale.rs       resolve_seller (S31-a steps 1–4), judge_exchange (steps 5–9), judge_sale = both, Sale,
              settle(Sale)                                                                          (S31-a's, split)
event.rs      + PriceProposed, ProposalDeclined, ProposalWithdrawn, ProposalAccepted
system.rs     dispatch only: the four actions route to haggle.rs
```

`haggle.rs` is expected near 350 lines and `proposal.rs` near 200. `system.rs` stays dispatch only.

### 5.2 Terms, from plain to exact

| Term | Plain meaning | Exact meaning |
| --- | --- | --- |
| **proposal** | "I'll give you 250 for one bread." | One buyer's offer to buy one unit of one kind from one seller at one price, recorded by economy until it is answered or expires |
| **open** | still waiting for an answer | `status = Open` and `now < valid_until` |
| **declined** | the seller said no | `status = Declined` and `now < valid_until`; kept only so the buyer can see the "no" (Q-S31h-6) |
| **live** | still counts | open or declined, i.e. `now < valid_until`. Only live entries count toward the limit |
| **answer** | accept or decline | `accept-proposal` or `decline-proposal`, by the seller or, for a shop, a keeper standing in it |
| **acceptance** | the seller says yes and the sale happens | one dispatch of `accept-proposal` whose facts are `proposal-accepted` and a full sale |

### 5.3 State: `Proposals` on the seller

```text
ProposalId      = the ActionId of the propose-price request (u64, opaque; SD-S31h-1)
Proposal        { id: ProposalId, buyer: PersonId, item: ItemId, price: NonZeroU64,
                  place: Option<PlaceId>,            // Some(shop place) for a shop's proposal, None for a person's
                  made_at: WorldTime, valid_until: WorldTime, status: ProposalStatus }
ProposalStatus  = Open | Declined
Proposals       { entries: Vec<Proposal> }           // on the seller entity: a Person, or the Organization
                                                     // that operates the shop; sorted by id
PROPOSAL_VALID_FOR   = 900 s   (15 world minutes, step §5.9; published)
PROPOSALS_PER_SELLER = 8       (live entries, step §5.9; published)
```

- **Where it lives.** On the seller, as the step says: a Person for its own asks, the operating Organization for a
  shop. One Organization may run two shops, so a shop's proposal records its `place`.
- **One writer.** Only economy's reductions of its own four facts write `Proposals` (I-S31-2 extended, I-h-2).
- **Bounded.** At most 8 live entries per seller. At most one live entry per (buyer, seller, item): a second
  proposal while one is open or declined-and-live is `proposal-limit` (F-S31h-9, Q-S31h-6).
- **Nothing is reserved.** `Proposals` holds no money and no item. No other component changes when a proposal is
  made. A wallet, a holding and an ask stay free to change.
- **Accepted and withdrawn entries are removed** at reduction. A declined entry is kept, marked `Declined`, until
  its own `valid_until` (Q-S31h-6). After that it is dead and the next reduction on that seller drops it.

### 5.4 Actions, payloads, targets and requirements

| Action | Payload (JSON) | Target | Actor | Requirement (data, `CLAUDE.md` §4 rule 15) |
| --- | --- | --- | --- | --- |
| `propose-price` to a shop | `{ "item": <ItemId>, "price": <u64> }` | none | a living Person standing in a shop | `at_place(here)`, as `buy` from a shop (S31-a §5.5) |
| `propose-price` to a person | the same | the seller | a living Person | `same_place().within(TRADE_RANGE).requiring_target_available()`, `TRADE_RANGE = 3 000 mm`, as `buy` from a person |
| `accept-proposal` | `{ "proposal": <ProposalId> }` | the proposal's **buyer** (SD-S31h-3) | the seller Person, or a keeper standing in the proposal's shop | person seller: `same_place().within(TRADE_RANGE).requiring_target_available()`; shop: `same_place().requiring_target_available()` (both in the shop) |
| `decline-proposal` | `{ "proposal": <ProposalId> }` | none | the same | `NONE` (step §5.9); a shop's keeper must still stand in that shop (§5.5 step 3) |
| `withdraw-proposal` | `{ "proposal": <ProposalId> }` | none | the proposal's buyer | `NONE` |

- `price` is `u64` and checked in the judge (`price-out-of-range` for 0 or above `PRICE_MAX`), as S31-a's seller
  actions check theirs.
- A proposal may be above the ask (SD-S31h-4). A buyer who wants a scarce item may offer more. Nothing forbids it,
  and acceptance does not read the ask anyway (step §5.9).
- An `accept-proposal` whose target is not the proposal's buyer is `PreconditionFailed` (a malformed request).
- No compatibility layer: everything is new (`CLAUDE.md` §4 rule 12).

### 5.5 The judges and their order (`haggle.rs`; offers and `validate` call the same function)

**`judge_proposal(world, buyer, target, item, price)`**, for `propose-price`:

| # | Check | Refusal |
| --- | --- | --- |
| 1–3 | S31-a's `resolve_seller` steps 1–3: buyer a living Person; seller from the target (a living Person ≠ buyer with `Asks`) or the shop at the buyer's place; an ask for `item` | `NoSupportedInteraction` (NoSuchSeller), `not-for-sale` |
| 4 | `1 ≤ price ≤ PRICE_MAX` | `price-out-of-range` |
| 5 | S31-a step 5: buyer is the seller, or a keeper of the shop | `PreconditionFailed` (SelfTrade) |
| 6 | a live entry for (buyer, seller, item) exists, **or** the seller has 8 live entries | `proposal-limit` (`detail`: `already-proposed` or `seller-full`) |
| 7–9 | S31-a steps 6–9 at the **proposed** price: funds, stock, carry, the owners' own functions | `insufficient-funds`, `out-of-stock`, `cannot-carry`, the owner's reason |
| then | the requirement of §5.4 | `TooFarAway` (person seller only) |

Steps 7–9 at proposal time are an early, honest refusal (SD-S31h-5). A buyer cannot offer money it does not have.
They reserve nothing, and acceptance asks them again.

**`judge_answer(world, actor, target, proposal, kind)`**, for `accept-proposal` and `decline-proposal`:

| # | Check | Refusal |
| --- | --- | --- |
| 1 | actor a living Person | `NoSupportedInteraction` |
| 2 | some seller's `Proposals` holds `proposal` with `status = Open` and `now < valid_until` | `no-such-proposal` (unknown, answered, withdrawn, declined or expired) |
| 3 | authority. Person seller: actor = seller. Shop: a shop stands at the actor's place, its place = `proposal.place` and its operator = the seller, else `NoSupportedInteraction`; the actor is one of its keepers | `PermissionDenied` (NotYourProposal) |
| 4 | accept only: the request's target = `proposal.buyer` | `PreconditionFailed` |
| 5 | accept only: `judge_exchange(Sale { seller, buyer, item, price: proposal.price, place })`, i.e. S31-a steps 5–9 | `PreconditionFailed` (SelfTrade), `insufficient-funds`, `out-of-stock`, `cannot-carry`, the owner's reason |
| then | accept only: the requirement of §5.4 | `TooFarAway` (the buyer walked off, or left the shop) |

Acceptance is **not** checked against the current ask (step §5.9: "The seller is choosing a different price on
purpose"). An ask that was changed or withdrawn after the proposal does not block acceptance (SD-S31h-7).

**`judge_withdrawal(world, actor, proposal)`**, for `withdraw-proposal`: 1 actor a living Person
(`NoSupportedInteraction`); 2 the proposal open (`no-such-proposal`); 3 actor = `proposal.buyer`
(`PermissionDenied`).

The order is fixed and tested, so two reasons at once always report the same name. Every code is
`Rejection::System { code }` built with `RejectionCode::from_static` (compile-time checked), except the kernel
reasons named in the tables.

### 5.6 Settlement at acceptance: S31-a's settlement, reused

S31-a's `judge_sale` is split in place, with no behaviour change for `buy`:

```text
resolve_seller(world, buyer, target, item, price?) -> Result<Sale, Rejection>    S31-a steps 1–4
judge_exchange(world, &Sale)                       -> Result<(), Rejection>      S31-a steps 5–9
judge_sale = resolve_seller then judge_exchange                                  unchanged for buy
Sale { seller, buyer: PersonId, item: ItemId, price: NonZeroU64, place: PlaceId }
settle(world, &Sale) -> [sold, money-transferred, items-transferred]             S31-a's, now taking a Sale
```

**`accept-proposal`, resolve:**

```text
validate (read-only) ─────────────────────────────────────────────────────────────────────────────────
   judge_answer steps 1–5 (§5.5), then SpatialRequirement::evaluate(actor, buyer, buyer living)
resolve ───────────────────────────────────────────────────────────────────────────────────────────────
   [ proposal-accepted { proposal, seller, buyer, item, price },             Participants
     sold { seller, buyer, item, count: 1, price, place },                   Place(place)   ┐ settle(Sale),
     money-transferred { buyer → seller, price },                            Place(place)   │ S31-a's, built
     items-transferred { seller → buyer, item, 1 } ]                         Participants   ┘ through pay() and
                                                                                               inventory::transfer()
reduce (same generation, same instant) ──────────────────────────────────────────────────────────────
   economy: removes the entry; writes both Wallets      inventory: writes both Holdings   (each re-checks)
persist ─────────────────────────────────────────────────────────────────────────────────────────────
   one revision = the journal entry + all four facts, in one SQLite transaction
```

Why it is atomic is S31-a's three-reason argument, unchanged (step §5.7). The only addition is
`proposal-accepted`, which economy reduces into its own `Proposals` only. Reducing it changes nothing that
`admit_payment` or `admit_transfer` reads. So the agreement S31-a's property test proves still holds, and C4
extends that test to acceptance (AC-h-6).

`sold.place` is the place where the sale happens: the shop for a shop, the seller's place for a person seller
(both stand in it; the requirement says so).

The sale scan (I-S31-3) counts a sale caused by `accept-proposal` exactly as one caused by `buy`: one `sold`, one
matching `money-transferred` and one matching `items-transferred`, sharing the cause, plus one
`proposal-accepted` (I-h-3).

### 5.7 Facts, audiences and reductions (all owned by `economy`)

| Fact | Payload | Audience | Reduces to |
| --- | --- | --- | --- |
| `price-proposed` | `{ proposal, seller, buyer, item, price, place?, valid_until }` | `Participants`: the buyer, and the seller Person, or (a shop) the keepers standing in the shop at that instant; `about: [buyer, seller]` | drops the seller's dead entries, then inserts an `Open` entry. A duplicate live (buyer, item) entry, or a ninth live entry, is `FactRefusedByOwner` |
| `proposal-declined` | `{ proposal, seller, buyer, item, price }` | `Participants`: the same set, now | drops dead entries; the entry becomes `Declined` (kept until `valid_until`). Not open → `FactRefusedByOwner` |
| `proposal-withdrawn` | the same | the same | drops dead entries; removes the entry. Not open → `FactRefusedByOwner` |
| `proposal-accepted` | the same | the same | drops dead entries; removes the entry. Not open → `FactRefusedByOwner` |
| `sold`, `money-transferred`, `items-transferred` | S31-a's, unchanged | S31-a's | S31-a's |

- **"Now" at reduction** is the dispatch's world time (A-5), so a reduction and its validate read the same clock.
- **Who hears a proposal.** Nobody but the buyer and the people who may answer it (step §5.9, "Disclosure").
  Carol, beside them, hears none of the four proposal facts (AC-h-1, X-14). She does hear `sold`, because a sale
  in a place is seen by that place (S31-a; step §7.2).
- **Biographical:** none of the four, like every economy fact (step §5.3).
- The `place?` field is present only for a shop's proposal (`skip_serializing_if`).

### 5.8 Offers (all incomplete; Q-S31a-3 ruled, Q-S31h-1)

| To whom | Offer | Target | Available when | Refused (before the requirement) |
| --- | --- | --- | --- | --- |
| a person standing in a shop whose operator has asks | one `propose-price` | none | always | `PreconditionFailed` (SelfTrade: a keeper of this shop); `proposal-limit` (`seller-full`) |
| a person perceiving a living person seller with asks | one `propose-price` | the seller | always | `proposal-limit` (`seller-full`) |
| a seller, or a keeper standing in the proposal's shop, per buyer B with at least one open proposal it may answer | one `accept-proposal` | B | some open proposal of B passes `judge_answer` steps 1–5 | the code of B's lowest-id open proposal, when none passes |
| the same, when it may answer at least one open proposal | one `decline-proposal` | none | always | — |
| a buyer with at least one open proposal | one `withdraw-proposal` | none | always | — |

- **Why incomplete.** An incomplete offer carries no payload, so a generic controller never takes it (A-8). A
  proposal, an acceptance, a decline and a withdrawal are each a choice: the price is the buyer's, the yes or no
  is the seller's. Q-S31a-3 ruled the rule for accept and decline. Q-S31h-1 applies it to withdraw.
- **Per target, not per proposal.** An incomplete offer cannot name a proposal. The `proposals` record (§5.9) lists
  them; the offer says whether answering B is possible right now, and perception then checks the requirement
  (A-6). The dispatch decides each proposal exactly.
- A `propose-price` offer is item-independent. Its refusal covers only reasons that hold for every kind (self-trade,
  a full seller). Every other reason is the dispatch's answer.
- After `valid_until`, an expired proposal yields no `accept-proposal` and no `decline-proposal` offer (AC-S31-20).

### 5.9 Disclosure: the `proposals` record

`proposals { incoming: [Line], outgoing: [Line] }`, a declared, never-carried record disclosed **about the
observer itself** to a living Person, and absent when both lists are empty.

```text
Line { proposal, seller, buyer, item, price, ask: Option<u64>, valid_until, status: open | declined,
       for_shop: bool }
incoming  live entries on the observer's own Proposals, and, when the observer stands in a shop it keeps,
          the operator's live entries whose place is that shop (for_shop: true)
outgoing  live entries anywhere whose buyer is the observer
order     by proposal id
ask       the seller's current asking price for the item (after WE-a modifiers), or null if withdrawn
```

- A buyer sees its own proposals, including a decline, until `valid_until` (Q-S31h-6). A seller sees the ones it
  may answer. A keeper sees a shop's proposals only while standing in that shop.
- **Nobody else sees any proposal** (step §5.9; I-S31-9 extended, I-h-5). The record is about the observer only,
  so presence drops it for anyone else by its own rule (`interaction.rs:110-118`).
- `ask` is shown to the buyer as well. The buyer already perceived it when proposing (S31-a §5.6), and the line is
  easier to read with it. It reveals no holdings count.

### 5.10 Expiry is lazy, and the decline is a short memory

- **No wake, no fact.** Nothing happens at `valid_until`. Every judge, offer and disclosure compares
  `world.at()` with `valid_until`. At exactly `valid_until` the proposal is dead: `now < valid_until` is the rule,
  as for notices (S31-b §5.3).
- **Pruning.** Each reduction on a seller first drops its dead entries. So a seller's state holds at most 8 live
  entries plus the dead ones left since its last write, and those are dropped by its next proposal fact.
- **The decline memory (Q-S31h-6).** A declined entry stays until its own `valid_until`. The buyer sees it in
  `outgoing` with `status: declined`. A second proposal from the same buyer for the same kind to the same seller
  is `proposal-limit` (`already-proposed`) until then.
  - Example: Bob proposes 250 at 10:00, and Grace declines at 10:02. Until 10:15 Bob sees "declined" and cannot
    propose bread to Grace again. A `haggle` buyer then either buys at the ask or walks on. At 10:15 he may try
    again.
- A withdrawn proposal is removed at once. The buyer chose to stop, and may propose again (for example at a new
  price).

### 5.11 Persistence, determinism and version

- **Facts only.** `Proposals` is reduced from logged facts; replay reproduces it; kill-and-resume equals an
  uninterrupted run (`ARC-25`). A proposal open at a restart is still open after it, with the same `valid_until`,
  because world time is persisted (AC-h-8).
- **Determinism.** Integers only; no draws in economy; entries sorted by id; offers ordered by target id, then
  action type; the record's lists by id.
- **Version.** Economy's `SystemVersion` rises by one over S31-a's (2 → 3, or 3 → 4 if WE-a's landed between). A
  save written before is refused by name (`ARC-25`).

### 5.12 NPC behaviour: the `haggle` and `accept_at_least_permille` options (interface with S31-d)

S31-d designs the `pricing:` block, its parser and its `price` module in parallel. This PR adds two options and
two behaviours. The names below are the interface this design assumes; the lane lead reconciles them with
S31-d's design (§10 R-S31d-h-1).

```yaml
# decisions.yaml (World Pack), per seat or class; both options are absent (off) by default
bob:   { model: rule, parameters: { pricing: { haggle: { offer_permille: 850, deadband_permille: 50 } } } }
grace: { model: rule, parameters: { pricing: { accept_at_least_permille: 900 } } }
```

| Option | Bounds | Behaviour |
| --- | --- | --- |
| `haggle.offer_permille` | 1 … 999 | the proposed price is `max(1, floor(ask × offer_permille / 1000))` |
| `haggle.deadband_permille` | 0 … 1000 | haggle only when `ask × 1000 > basis × (1000 + deadband_permille)` |
| `accept_at_least_permille` | 1 … 1000 | accept a proposal with `price × 1000 ≥ ask × accept_at_least_permille`; decline one below it |

- **Basis** (step §5.9): the current suggested price for the kind, read from S31-b's `notice-board` record (a
  `prices` line; with several current lines, the newest `issued_at`, then the lowest publisher id); if there is
  none, the kind's reference value from S31-a's `currency` record; if there is neither, the buyer never haggles.
  The board lists only current notices, so an expired suggestion is never used (X-6's rule, step §10.1).
- **The buyer's hook.** Where the model has chosen a `buy` request (whichever band chose it: the paced rule's
  offered band, or S31-d's `buyer:` choice), `price::haggle_instead(observation, pricing, &buy)` returns the
  `propose-price` request instead when all of these hold: `haggle` is set; the rule above holds; the record shows
  no live outgoing proposal for that (seller, item); the `propose-price` offer for that seller is available. If a
  **declined** proposal is live, it returns the `buy` unchanged (the buyer pays the ask). If an **open** one is
  live, it returns nothing for that band, so the buyer waits. No new draw is taken; the frozen constants of
  `offered.rs` are unchanged (`ARC-35` point 6).
  - Example: ask 300, suggestion 260, deadband 50: 300 000 > 260 × 1 050 = 273 000, so Bob proposes
    floor(300 × 850 / 1 000) = 255.
- **The seller's band.** `price::answer_proposals(observation, pricing)` looks at `incoming` in id order and
  returns one request for the first open line: `accept-proposal` (target the buyer) when the threshold holds and
  the `accept-proposal` offer for that buyer is available; `decline-proposal` when the threshold fails or `ask` is
  null; nothing when the threshold holds but acceptance is unavailable (the proposal may become acceptable before
  it expires). The rule model runs this band before its offered band; `motivated` adds it as one choice. Exactly
  where each model calls it is S31-d's call site for pricing policies.
- **Requests are built through economy's published types** (`ProposePrice`, `AcceptProposal`, `DeclineProposal`),
  as the step's A-20 does for `set-price`. The controller names action types only inside `price`.
- **Default parity.** With neither option set, both functions return their input or nothing, and every existing
  world decides exactly as before (R-S31-5; AC-h-10).

### 5.13 The clients: sale-panel rows (2D and 3D; S31-e's panel model)

Both clients share S31-e's panel model and formatter, so the rows are written once (step §12).

| Row | Shown when | Action | Words |
| --- | --- | --- | --- |
| *Offer a price…* beside each buy row | the `propose-price` offer for that seller is present | opens S31-e's price field (major units, converted with the disclosed exponent); sends `propose-price { item, price }` with the seller as target for a person | `sale.offer_price` |
| "Your offer: bread 2.55 · valid until 10:15 · *Withdraw*" | an `outgoing` line | *Withdraw* sends `withdraw-proposal { proposal }` | `sale.your_offer`, `sale.withdraw` |
| "Declined · until 10:15" on the same row | `status: declined` | none | `sale.declined` |
| "Bob offers 2.55 for bread (ask 3.00) · *Accept* · *Decline*" | an `incoming` open line | *Accept* sends `accept-proposal { proposal }` targeted at the buyer; *Decline* sends `decline-proposal { proposal }` | `sale.incoming`, `sale.accept`, `sale.decline` |
| refusals | an outcome or a refused offer | — | `refusal.no-such-proposal`, `refusal.proposal-limit`, `refusal.proposal-limit.seller-full`, plus S31-e's existing codes |

- *Accept* is enabled exactly when the server's `accept-proposal` offer for that buyer is available, and shows the
  offer's refusal word otherwise. The client compares no price with any ask and no wallet with any price (I-S31-12;
  S31-e's client-rules patterns already fail CI on the latter).
- Words go into `en.po` and `zh_Hans.po` of the default 2D pack and the 3D slice's shared keys, as S31-e does.
- The 3D slice shows the same panel through its existing interaction prompt; no new geometry.

### 5.14 The agent view (S31-g's surfaces; Q-S31h-3)

| Surface | Change |
| --- | --- |
| `MeLine.proposals` | `{ incoming: [ProposalLine], outgoing: [ProposalLine] }` from the record, amounts as minor units and `display` (S31-g's rule) |
| `act` kinds | `propose_price { to: Ref \| null, item: Ref, price: int ≥ 1 }`, `accept_proposal { proposal: int ≥ 1 }`, `decline_proposal { proposal: int ≥ 1 }`, `withdraw_proposal { proposal: int ≥ 1 }`, each bound to a new `Player` operation that builds the request for its one action type (for `accept_proposal`, the target is the proposal's buyer, read from the seat's latest record) |
| inbox | `price-proposed`, `proposal-accepted`, `proposal-declined`, `proposal-withdrawn` read by name, like `spoke` |

- The kinds are typed and closed, so QS27-9 (no generic free-form act) is not reversed (A-10). The server path and
  the rules are the same as for a human (agents are first-class players).
- What an agent never sees: anyone else's proposals (I-h-5). AC-h-9 asserts it through the adapter.

### 5.15 An LM seat (Q-S31h-4)

This PR adds nothing to `cognition/lm-controller`. The four offers and the record are in every observation, so an
LM seat reads them. It can attempt them once the LM controller can build a request for an incomplete offer by
name. S31-d needs that for its LM reporter's `publish-notice`. Whichever PR adds it covers the four haggling
actions with no further change, because the payloads are small typed objects (§5.4).

### 5.16 Market Town and the other reference worlds

No content changes. No world configures `haggle` or `accept_at_least_permille`. Market Town's facts and summaries
cannot move (A-8, AC-h-10). Its `journal-300s` and `sealed-300s` move, because its saves record economy's new
declaration (F-S31h-1). If Q-S31h-5 is answered as recommended, C9 records a composition-only re-baseline of
`[world market-town]`, with a reason line stating that no fact and no summary moved. Social Café, Lakeside and
bodies-yard do not enable economy and do not move at all (AC-S31-9).

---

## 6. Invariants this PR holds

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-h-1 (I-S31-1) | No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/inventory/`, `systems/item-transfer/` changes | C10 diff scan |
| I-h-2 (I-S31-2) | Only economy writes `Proposals`, `Wallet`, `Asks`, `Shop`; only inventory writes `Holdings` | `declaration()` review; the reductions' tests |
| I-h-3 (I-S31-3) | Every accepted `accept-proposal` has exactly one `proposal-accepted`, one `sold`, one matching `money-transferred` and one matching `items-transferred`, in one revision; a refused one has none | `haggle_settlement.rs`; the extended sale scan |
| I-h-4 (I-S31-4, I-S31-5) | `money-transferred.amount = sold.price = proposal.price` for an acceptance; Σ wallets = Σ `funded` at every revision | AC-h-1; the property test |
| I-h-5 (I-S31-9) | A proposal (its facts and its record lines) reaches only its buyer and those who may answer it | AC-h-1, AC-h-9; X-14 |
| I-h-6 | Nothing is reserved: making, declining or withdrawing a proposal writes no `Wallet`, `Holdings` or `Asks` | AC-h-2's state-byte comparison |
| I-h-7 | Acceptance re-runs every `buy` check through `judge_exchange`, the same function `buy` uses | review; X-13 |
| I-h-8 | An expired proposal is never offered, accepted, declined or withdrawn, and no fact marks its expiry | AC-h-3 |
| I-h-9 (I-S31-10) | A world that configures no haggling decides exactly as before: no controller attempts a haggling action | AC-h-10 |
| I-h-10 (I-S31-12) | No client compares a proposal with an ask or a wallet | S31-e's client-rules scan; review of C8 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt, clippy `-D warnings`, the compile-time codes | CI `fast`, `core` |
| Integration (economy, real kernel dispatch over the hand-built world) | AC-h-1 … AC-h-5, AC-h-7; X-13, X-14; offers ≡ dispatch | `haggle.rs`, `haggle_settlement.rs` |
| Property (economy) | AC-h-6: acceptance admitted ⇒ four facts, no fault; offer verdict ≡ validate | `haggle_settlement.rs` |
| Persistence (real SQLite) | AC-h-8 | `haggle_persisted.rs` |
| Controller (real rule model in economy's fixture) | AC-h-11 (C6) | `systems/economy/tests/haggle_paced.rs`; unit cases in S31-d's `price` tests |
| Agent | AC-h-9 (C7) | `clients/agent/tests/` (the file S31-g names for its economy kinds) |
| Clients | AC-h-12 (C8) | S31-e's drive and shot harness |
| Long | AC-h-10, and the re-baseline (C9) | one `parity-long` nightly dispatch on the branch |
| Gate 1 | NOT REQUIRED: no LLM-facing prompt or tool description changes, except C7's kind descriptions, which are covered by the agent golden | — |

Oracles are literal prices, wallets and holdings written in the test, or the base build's record; never a value
economy reports about itself.

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

Two compositions are used, so no criterion rests on one world:

- **W1 (S31-a's fixture):** the corner store (shop, operator `corner-store`, keeper Felix), Grace (a person with
  `asks: { bread: 300, pen: 150, apple: 120 }`, holding 2 bread, 1 pen, 3 apple), Alice, Bob, Dave and Carol;
  currency USD, exponent 2.
- **W2 (generated variant):** a market square with a stall run by an Organization `guild` with two keepers,
  currency `{ crown, cr, 0, after }`, prices in whole crowns, and buyers drawn by SplitMix64 seed 41 (positions
  0 … 6 m, wallets 0 … 50).

| Id | Criterion | Step ref |
| --- | --- | --- |
| AC-h-1 | **Haggling settles atomically at the proposed price.** W1: Bob, 2 m from Grace, proposes 250 for bread against her ask of 300; Grace accepts. One revision holds `proposal-accepted`, `sold { price: 250 }`, `money-transferred { 250 }` and `items-transferred { bread, 1 }`; Bob's wallet −250, Grace's +250; I-S31-3 and I-S31-5 hold. Carol, 1 m away in the same place, perceives `sold` and none of `price-proposed` or `proposal-accepted` (presence's `perceived_by`). The same in W2 for a shop proposal accepted by a keeper: the other keeper, standing outside the stall, perceives nothing of it | AC-S31-19 |
| AC-h-2 | **Nothing is reserved.** After `propose-price` and after `decline-proposal`, the bytes of every `Wallet`, `Holdings` and `Asks` equal those before | AC-S31-20 (premise) |
| AC-h-3 | **Acceptance re-checks everything, and each refusal moves nothing** (state bytes equal before and after): Bob spends his money first → `insufficient-funds`; Grace gives her last bread to Dave first → `out-of-stock`; Bob walks to 3.1 m → `TooFarAway`; Bob already carries six → `cannot-carry`; at `made_at + 900 s` → `no-such-proposal`, and the accept and decline offers for Bob are gone; Alice accepts Grace's proposal → `PermissionDenied`; Felix answers a store proposal from the street → `NoSupportedInteraction`, from inside → accepted | AC-S31-20 |
| AC-h-4 | **The limits.** Three buyers propose three kinds each to Grace: the ninth live proposal is `proposal-limit` (`seller-full`). Bob proposing bread twice → `proposal-limit` (`already-proposed`). After Grace declines Bob's bread, Bob's new bread proposal → `proposal-limit` until `valid_until`, accepted after it (Q-S31h-6). After Bob withdraws, a new proposal is accepted at once | AC-S31-20 |
| AC-h-5 | **Each named refusal of `propose-price`** in §5.5's order (`NoSupportedInteraction`, `not-for-sale`, `price-out-of-range` for 0 and `PRICE_MAX + 1`, SelfTrade, `proposal-limit`, `insufficient-funds`, `out-of-stock`, `cannot-carry`, `TooFarAway`), and of `withdraw-proposal` (`no-such-proposal`, `PermissionDenied`), by dispatch and, where an offer exists, by the offer's refusal | §5.9 refusals |
| AC-h-6 | **Property.** Over 10 000 seeded states (W2's generator plus open proposals at random prices and ages 0 … 1 200 s), whenever `validate` admits `accept-proposal`, dispatch returns `Accepted` with exactly the four facts and no `KernelError`; and the `accept-proposal` offer's verdict for the buyer equals `validate`'s for that buyer's lowest-id open proposal whenever it has one | AC-S31-4's property, extended |
| AC-h-7 | **Offers.** In W1 and W2: the buyer gets one incomplete `propose-price` per seller; the seller gets one incomplete `accept-proposal` per buyer with an open proposal and one incomplete `decline-proposal`; the buyer one incomplete `withdraw-proposal`; no haggling offer is complete; a keeper outside its shop gets none for that shop | Q-S31a-3; Q-S31h-1 |
| AC-h-8 | **Across a restart.** W1 in a real SQLite save: Bob proposes at 10:00; the world is dropped and resumed from the file; Grace accepts at 10:05 → settled as AC-h-1; a second run resumed at 10:16 → `no-such-proposal`. Every resumed state equals an uninterrupted run's, byte for byte | `ARC-25`; I-S31-3 |
| AC-h-9 | **Agent view** (C7). Through the adapter: Bob's `MeLine.proposals.outgoing` shows his proposal with `display`; Grace's `incoming` shows it; Carol's view and `inspect` of Grace show neither; the four kinds reach their own `Player` operations | §5.14; AC-S31-8's privacy rule |
| AC-h-10 | **Nobody haggles unless configured.** Market Town, seed 7, 30 days, base record (C0) vs head: the journal's sequence of `(actor, action type, at, outcome kind)` is equal, every fact is equal in order and payload, and no haggling fact exists; with the long record, every summary and `facts-300s` equal `main`'s, and only `journal-300s` and `sealed-300s` of `[world market-town]` differ | I-h-9; F-S31h-1 |
| AC-h-11 | **NPCs haggle** (C6). W1 and W2 under the rule model for 2 world days. W1: Grace asks 300 for bread, `accept_at_least_permille: 900` (threshold 270), reference value 260; Bob `haggle { offer_permille: 850, deadband_permille: 50 }` proposes 255, Alice `haggle { offer_permille: 950, deadband_permille: 50 }` proposes 285. Bob's proposal is declined and Alice's is accepted, in the same run: at least one `proposal-declined` and one `proposal-accepted`; every accepted price ≥ ask × 0.9; no buyer re-proposes a kind to a seller while its decline is live. W2: the same options on the guild's keepers and three generated buyers give at least one of each fact | AC-S31-21 (its mechanism; the town run is S31-f's) |
| AC-h-12 | **Clients** (C8). 2D and 3D drives in W1's world pack form: Bob offers 2.50, Grace's panel shows the incoming row, *Accept* settles, both wallets change by 2.50; a refusal shows its words; shots on three OSes | AC-S31-15, haggling rows |

### 8.1 Planted mutations (each must turn its test red by name, then be reverted from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-13 | `judge_answer` skips `judge_exchange`'s funds step and the `admit_payment` call for acceptance | AC-h-3's `insufficient-funds` arm: the outcome becomes `FactRefusedByOwner { system: economy }` from `pay()` at resolve instead of `System { insufficient-funds }` |
| X-14 | `price-proposed` stated with `Visibility::Place(place)` | AC-h-1: Carol perceives `price-proposed` |
| M-h-1 | expiry compares `now <= valid_until` | AC-h-3: acceptance at exactly `made_at + 900 s` is admitted |
| M-h-2 | `decline-proposal` removes the entry instead of marking it declined | AC-h-4: Bob's second bread proposal is admitted before `valid_until` |
| M-h-3 | `accept-proposal` offered complete | AC-h-7 |
| M-h-4 | the reduction of `price-proposed` skips the limit | AC-h-6 (a ninth entry from a state built past `validate` is reduced) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31h`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full.
2. Make the numbered steps, in order. Do not do a later commit's work.
3. Run the commit's validation. Record per command: test count, wall time, PASS / FAIL / INCONCLUSIVE.
4. For each listed mutation: plant, run the named test, record the red (test name), revert from a byte copy
   (`cmp` clean), run green.
5. Static: `cargo fmt --check`; `cargo clippy -p <touched crate> --all-targets -- -D warnings`. Tests:
   `cargo test -p mineworld-economy --test <file>`, one file at a time. **R-LOAD-1:** never
   `cargo test --workspace`, never all of `tools/cli`. Heavy jobs only through
   `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
6. Review: answer the commit's review questions in one or two sentences each, in §16.
7. `git status --short` and `git diff --stat`: every changed path is in the commit's **Files**, or revert it, or
   stop and record why.
8. Update §16 (ledger row, evidence, findings) in the same commit.
9. Stage by explicit path (never `git add -A`); commit with the session's attribution line. No heredoc writes, no
   `sed -i`, no `python3 -c`, no `curl`.
10. Stop and report a material stop (§12) instead of improvising.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Goal:** record the base and choose the branch of §2.3 for each conditional commit.
- **Files:** this file (§16).
- [ ] Implementation: record the base commit; confirm Milestone F handed over and S31-a merged (either missing →
  material stop); record the merge state of S31-b, S31-d, S31-e, S31-g and the decision of Q-S31h-2/-3/-5; mark
  C6, C7, C8, C9 as "here" or "N/A, moved to <PR>"; re-read A-1 … A-18 on the base and record moved lines; build
  the base binary and write the base Market Town 30-day save for AC-h-10 (not committed; record its sha256).
- [ ] Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Review: §2.3 holds on the base; S31-a's `sale.rs` has the shape §5.6 splits (else a bounded adaptation,
  recorded).
- **Commit:** `docs(plan): S31-h — open the ledger at the base`

#### C1 — The decision first

- **Goal:** `ARC-118` exists before the code it governs (`CLAUDE.md` §2.2).
- **Files:** `docs/decisions/ARC-118.md`, the regenerated decision index, `docs/MODULE_SPEC.md` (§4.1 economy row).
- [ ] Implementation: `ARC-118` — problem, options (escrow declined; a counter declined for `ActionId`; complete
  offers declined by Q-S31a-3), the choice (§§5.3 … 5.11), the rulings of §14 as answered, consequences (the
  composition-only re-baseline). MODULE_SPEC: the economy row lists the four actions.
- [ ] Validation: `python3 scripts/check_decision_ids.py --write-index`, then without arguments (all distinct);
  `python3 scripts/check_doc_headings.py`; `python3 scripts/ci_layer.py docs`.
- [ ] Review: no new ontology term (`CLAUDE.md` §2.1 rule 3); every code of §5.5 appears in `ARC-118`.
- **Commit:** `docs: haggling settles at acceptance with nothing reserved — ARC-118 (S31-h)`

#### C2 — `Proposals` and the four facts

- **Goal:** the state and its only writes, before any action can produce them.
- **Files:** `systems/economy/src/{proposal.rs (new), event.rs, system.rs, codec.rs, lib.rs}`,
  `systems/economy/tests/{haggle.rs (new), support/mod.rs}`.
- [ ] Implementation: `ProposalId`, `Proposal`, `ProposalStatus`, `Proposals`, the two constants; the four facts;
  the reductions of §5.7 with pruning; `declaration()` owns `Proposals`, emits and subscribes to the four facts;
  `SystemVersion` + 1.
- [ ] Validation: `--test haggle` (reductions only, from facts stated directly through `Emission`: insert,
  decline-marks, withdraw and accept remove, pruning at `valid_until` exactly, each `FactRefusedByOwner` case);
  `--test buy`, `--test sale` (S31-a's, unchanged).
- [ ] Review: nothing but these reductions writes `Proposals`; no float, no draw; the component serializes in id
  order.
- **Commit:** `feat(economy): proposals on the seller, written only from economy's own facts (S31-h)`

#### C3 — Proposing and withdrawing

- **Goal:** a buyer can make and take back a proposal, with honest refusals, and sees its own proposals.
- **Files:** `systems/economy/src/{haggle.rs (new), sale.rs, offer.rs, system.rs, codec.rs, proposal.rs, lib.rs}`,
  `systems/economy/tests/{haggle.rs, support/mod.rs}`.
- [ ] Implementation: split `judge_sale` (§5.6) with no change to `buy`; `ProposePrice`, `WithdrawProposal`;
  `judge_proposal`, `judge_withdrawal`; resolve states the facts with §5.7's audiences; the buyer's offers
  (§5.8); the `proposals` record (`outgoing`, and `incoming` for a person seller).
- [ ] Validation: `--test haggle` (AC-h-2, AC-h-4's propose and withdraw arms, AC-h-5, AC-h-7's buyer half, the
  record's audiences); `--test sale` and `--test settlement` (S31-a's, unchanged: the split moved no `buy`
  outcome). Mutation M-h-1 (on the propose side: a proposal is offered withdraw at exactly `valid_until`).
- [ ] Review: `judge_sale`'s outcomes are identical (S31-a's property test is green unchanged); the record is
  about the observer only; `ReferenceValue` is not read.
- **Commit:** `feat(economy): a buyer proposes a price and may withdraw it; nothing is reserved (S31-h)`

#### C4 — Accepting and declining; settlement at acceptance

- **Goal:** the seller answers, and acceptance is a sale with every check re-run.
- **Files:** `systems/economy/src/{haggle.rs, sale.rs, offer.rs, system.rs, codec.rs, proposal.rs}`,
  `systems/economy/{Cargo.toml (dev-deps), tests/{haggle.rs, haggle_settlement.rs (new), haggle_persisted.rs
  (new), support/mod.rs}}`.
- [ ] Implementation: `AcceptProposal`, `DeclineProposal`; `judge_answer`; accept's resolve = `proposal-accepted`
  then `settle(Sale)`; the seller's and keeper's offers; `incoming` for keepers standing in their shop; W1's
  Grace and Dave and W2's generator in `support/mod.rs`.
- [ ] Validation: `--test haggle` (AC-h-3, AC-h-4's decline arm, AC-h-7); `--test haggle_settlement` (AC-h-1 in W1
  and W2, AC-h-6; budget ≤ 60 s on Linux CI); `--test haggle_persisted` (AC-h-8). Mutations X-13, X-14, M-h-1,
  M-h-2, M-h-3, M-h-4.
- [ ] Review (Opus logic review): the acceptance path reads no ask; every `buy` check runs through
  `judge_exchange`; the four facts are built only through `pay` and `inventory::transfer` plus economy's own;
  the audiences equal §5.7's table.
- **Commit:** `feat(economy): the seller accepts or declines; acceptance settles atomically, re-checked (S31-h)`

#### C5 — The market mirror and the sale scan

- **Goal:** S31-f's town run reads haggled sales as sales, not as defects (A-17).
- **Files:** `tools/cli/tests/market/mod.rs`.
- [ ] Implementation: mirror the four facts (schema 1, exact fields); `sales_are_atomic` accepts a sale whose
  cause is `accept-proposal` with its `proposal-accepted`, and flags one without it.
- [ ] Validation: CI only (R-LOAD-1: the files that use the mirror are 30- and 300-day runs); locally
  `cargo check -p mineworld-cli --tests`.
- [ ] Review: no change in what the scan accepts for `buy`.
- **Commit:** `test(market): the sale scan counts sales settled by an accepted proposal (S31-h)`

#### C6 — The NPC options (here only if S31-d merged at C0; else N/A, R-S31d-h-1)

- **Goal:** a configured NPC buyer haggles and a configured NPC seller answers.
- **Files:** S31-d's `cognition/rule-controller/src/price.rs` and its tests; the rule model's and `motivated`'s
  pricing call sites (as S31-d named them); `systems/economy/tests/haggle_paced.rs` (new).
- [ ] Implementation: the two options with their bounds in S31-d's `Pricing` type; `haggle_instead`;
  `answer_proposals`; the two call sites.
- [ ] Validation: S31-d's `price` unit tests gain the §5.12 cases (basis from the board, from the reference, none;
  the deadband edge; declined → buy; open → wait); `--test haggle_paced` (AC-h-11); `-p mineworld-rule-controller`
  template determinism test (every installed model unchanged with the options absent).
- [ ] Review: no new draw; `offered.rs` unchanged; options absent ⇒ identical requests.
- **Commit:** `feat(rule): pricing gains haggle and accept_at_least_permille (S31-h)`

#### C7 — The agent rows (here only if S31-g merged at C0; else N/A, R-S31g-h-1)

- **Files:** `clients/agent/src/mineworld_agent/{models.py, player.py, view.py, mcp_server.py, inbox.py}`,
  `clients/agent/tests/` (S31-g's economy test file; the golden `tests/golden/tools.json`).
- [ ] Implementation: §5.14, following S31-g's pattern for its economy kinds.
- [ ] Validation: `uv run --locked pytest` on the touched test files one at a time; the golden regenerated and
  byte-equal to `mineworld-agent schema`; AC-h-9.
- [ ] Review: no generic free-form act; no field echoes agent input in errors (`ARC-153` R-4).
- **Commit:** `feat(agent): propose, accept, decline and withdraw a price as act kinds (S31-h)`

#### C8 — The client rows (here only if S31-e merged at C0; else N/A, R-S31e-h-1)

- **Files:** S31-e's panel model and its 2D and 3D views; `en.po`, `zh_Hans.po`; S31-e's drive scripts.
- [ ] Implementation: §5.13.
- [ ] Validation: S31-e's client-rules scan; the drives and shots of AC-h-12 through
  `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, or CI's client jobs.
- [ ] Review: no rule evaluated in a client; both clients use one panel model.
- **Commit:** `feat(clients): offer a price, and answer offers, in the sale panel (S31-h)`

#### C9 — Composition-only re-baseline (rebased last; per Q-S31h-5)

- **Files:** `scripts/baselines.txt`.
- [ ] Implementation: push; dispatch the nightly on the branch with `groups=parity-long` once; download the long
  record; `python3 scripts/ci_parity.py baseline write <record> scripts/baselines.txt`; add `reason S31-h
  (ARC-118, Q-S31h-5): economy's declaration gained proposals and four facts; journal-300s and sealed-300s move at
  the genesis row only; no fact and no summary moved (AC-h-10)`.
- [ ] Validation: `baseline check` against the record; AC-h-10's comparison: only those two keys of
  `[world market-town]` differ from `main`.
- [ ] Review: no other world moved.
- **Commit:** `chore(baselines): Market Town's composition gained economy's haggling vocabulary (S31-h)`

#### C10 — Close-out

- **Files:** this file (§16), `systems/economy/README.md`.
- [ ] Implementation: the README table; I-h-1's diff scan; §8's evidence; open the PR `mvp0/pr-s31h`.
- [ ] Validation: CI green on the exact head (Linux, macOS, Windows).
- [ ] Review: every deviation recorded.
- **Commit:** `docs(plan): S31-h close-out` · **Then:** MERGE-READY: report the PR number and head SHA; the
  coordinator merges once `fast` is green.

---

## 10. Requirements this PR places on other PRs

- **R-S31d-h-1 (on S31-d).** If S31-h merges first, S31-d includes §5.12 as written: the two options in its
  `Pricing` type, `haggle_instead` and `answer_proposals` in `price`, at the same call sites as its policies, and
  AC-h-11 in its tests. If S31-d merges first, it leaves a call site where a chosen `buy` request passes through the
  `price` module, so C6 can attach without moving its policies. S31-d's names may differ from §5.12's; the lane
  lead reconciles them before either design freezes.
- **R-S31d-h-2 (on S31-d's LM half).** The mechanism that lets the LM seat attempt `publish-notice` by name is
  general over incomplete offers, so it also covers the four haggling actions (Q-S31h-4).
- **R-S31e-h-1 (on S31-e).** If S31-h merges first, S31-e includes §5.13's rows, words and AC-h-12. If S31-e merges
  first, its panel model has a row type for an incoming offer with two buttons and a price-entry row, so C8 adds
  rows and no new component.
- **R-S31g-h-1 (on S31-g).** If S31-h merges first, S31-g includes §5.14 and AC-h-9. Either way, S31-g's own
  economy actions and S31-h's four go the same way under `ARC-153` (Q-S31h-3), decided once.
- **R-S31f-h-1 (on S31-f).** Market Town's haggling sellers and buyers use §5.12's option names. S31-f's
  re-baseline reason line names S31-h if Q-S31h-5 is answered (b).

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31h-1 | `ProposalId` = the `ActionId` of the `propose-price` request | unique, world-allocated, journaled (A-4); no counter state to own |
| SD-S31h-2 | Proposals live on the seller; a shop's proposal records its place | step §5.9; one Organization may run two shops |
| SD-S31h-3 | `accept-proposal`'s target is the buyer | `SamePlaceAsActor` and a range need a target (A-7, F-S31h-6) |
| SD-S31h-4 | A proposal above the ask is allowed | a buyer may bid up a scarce item; acceptance never reads the ask |
| SD-S31h-5 | `propose-price` runs the funds, stock and carry checks at the proposed price | an early, honest refusal; nothing reserved; acceptance re-runs them |
| SD-S31h-6 | The four facts carry `{ proposal, seller, buyer, item, price }` | reduction finds the seller directly; an inbox can say who and what (F-S31h-7) |
| SD-S31h-7 | A changed or withdrawn ask does not block acceptance | step §5.9: acceptance is not checked against the current ask |
| SD-S31h-8 | `proposal-limit`'s `detail` is `already-proposed` or `seller-full` | one code as the step says; two words for clients (F-S31h-9) |
| SD-S31h-9 | Every reduction on a seller prunes its dead entries first | lazy expiry with bounded state (F-S31h-8) |
| SD-S31h-10 | Offers are per target (the seller to propose to, the buyer to answer) | an incomplete offer cannot name a proposal; the record does |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-h — haggling: propose, accept, decline, withdraw; settlement at
                         acceptance with nothing reserved
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31h-haggling.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§5.2, 5.4–5.7, 5.9, 10.1, 11–16, 18, 19.4);
                         pr-s31a-currency-asks.md (§§5.5–5.7, 8, 10, 11); pr-s31b-notices.md (§5.8, the board
                         record); S31-d's, S31-e's and S31-g's designs as frozen; docs/decisions ARC-25, ARC-26,
                         ARC-34, ARC-112, ARC-153; docs/CORE_CONCEPTS.md §§12, 15.2; docs/MODULE_SPEC.md §§4.1,
                         5.1; .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31h, its own, held by one session
BRANCH:                  mvp0/pr-s31h, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5) AND S31-a merged AND
                         this design frozen AND the lane lead's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7, as §14's rulings shape them; C6, C7, C8 only where C0 found the
                         dependency merged
FROZEN INVARIANTS:       step-31 I-S31-1 … I-S31-5, I-S31-9, I-S31-10, I-S31-12 (their S31-h parts); §6 I-h-1 …
                         I-h-10; §8 as written; S31-a's frozen settlement (§5.7 there), changed only by the
                         split of §5.6 here, with no change to any buy outcome
SEQUENCE:                C0 … C10 (§9)
ALLOWED COMMANDS:        cargo build / check / test -p <crate> --test <file> / clippy -p / fmt (heavy ones
                         through cast-slots/with-slot.sh); target/*/mineworld *; uv run --locked *; git; gh (pr
                         create/view/checks; workflow run nightly.yml on the branch with groups=parity-long, once
                         per final head; run download); python3 scripts/*; mkdir -p; cmp; shasum; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or all of
                         tools/cli locally (R-LOAD-1); reading any key or .env; calling any model; editing
                         kernel/, contracts/, persistence/, server/, systems/inventory/, systems/item-transfer/;
                         merging
MATERIAL STOPS:          Milestone F not handed over or S31-a not merged; a needed change under I-h-1's paths; a
                         needed external dependency; the §5.6 split changes a buy outcome; AC-h-6 finds a state
                         validate admits and a reduction refuses; AC-h-10 shows a fact or summary moved; S31-d's,
                         S31-e's or S31-g's merged shape cannot take §5.12, §5.13 or §5.14 without changing their
                         frozen design; any change to a frozen step invariant or acceptance
PLATFORMS:               Linux, macOS, Windows (CI core, python and client jobs on all three)
VALIDATION BUDGET:       economy test files unrestricted locally, one at a time; haggle_settlement.rs ≤ 60 s on
                         Linux CI; tools/cli market files in CI only; one parity-long dispatch per final head
                         (C9). Gate 1: NOT REQUIRED. Gate 2: the real SQLite save (AC-h-8) and the long record
                         (AC-h-10). No paid call
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31h.md, created at the first session boundary
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to review readiness: authorized       (working rules §21)
                         the nightly dispatch of C9: at the milestone, not per PR (charter SPEED FIRST)
                         endpoint: MERGE-READY — the PR pushed with `fast` green on the exact head, its
                         number and head SHA reported
                         merge: NOT by the implementation session and NOT by the lane lead. The coordinator
                         merges once `fast` is green (lane-lead charter, SPEED FIRST 2026-10-18)
POST-MERGE SYNC OWNER:   lane lead: step-31, overall.md, the progress lines; implementation session: this
                         ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-h MERGE-READY — the implementation session does not merge
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31h-1 | The split of `judge_sale` changes a `buy` outcome | S31-a's property test and buy tests run unchanged in C3; a change is a material stop |
| R-S31h-2 | S31-d, S31-e and S31-g name things differently from §§5.12–5.14 | §10's requirements; the lane lead reconciles before freezing; C0 chooses here or N/A |
| R-S31h-3 | Buyer and seller policies loop (propose, decline, propose) | the decline memory (Q-S31h-6); AC-h-11 asserts no re-proposal while a decline is live |
| R-S31h-4 | NPC sellers accept proposals they should not, draining stock below cost | `accept_at_least_permille` is relative to the seller's own ask; S31-f sizes it against E-9's criteria (AC-S31-21) |
| R-S31h-5 | Economy grows large | §5.1's two new files; `system.rs` dispatch only; size triggers reviewed in C4 |
| R-S31h-6 | `mineworld-rule-controller` depends on economy (S31-d) while economy's tests depend on the rule controller | Cargo allows the dev-dependency cycle; the paced test uses the controller only through observations and requests, so the two compiled copies of economy never meet in one type. If S31-d finds otherwise, the paced test moves to `tools/cli` |
| R-S31h-7 | The re-baseline is read as a behaviour change | C9's reason line and AC-h-10's comparison state that only the composition moved |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation | Reason, one line |
| --- | --- | --- | --- |
| **Q-S31h-1** | The step offers `withdraw-proposal` **complete**. Generic NPCs take any complete offer at random, so they would withdraw their own proposals about one consult in five (F-S31h-2). Offer it **incomplete**, like accept and decline? | **Yes, incomplete.** | A complete offer must be safe to take at random (Q-S31a-3's principle); withdrawing undoes the haggle |
| **Q-S31h-2** | The `pricing:` block is S31-d's, and S31-d comes after S31-h in the order. Apply the "whichever merges second" rule to S31-d too: if S31-h merges first, S31-d adds the two haggling options as specified here? | **Yes.** | S31-h then needs no block that does not exist yet, and nothing is designed twice |
| **Q-S31h-3** | The step names "four agent tools". Since #261 (`ARC-153`) the MCP surface is one `act` tool with closed kinds. Add `propose_price`, `accept_proposal`, `decline_proposal`, `withdraw_proposal` as four typed **act kinds** instead, the same way S31-g adds its economy kinds? | **Yes, four typed act kinds.** | It keeps the one-tool ruling and stays typed, so QS27-9's ban on a free-form act is untouched |
| **Q-S31h-4** | An LM seat can attempt only complete offers today, and all haggling offers are incomplete. Accept that LM haggling arrives with the general "attempt an incomplete offer by name" mechanism (needed by S31-d's LM reporter anyway), with no LM work or LM acceptance in S31-h? | **Yes.** | One general mechanism covers every incomplete action; building it here for four actions would duplicate it |
| **Q-S31h-5** | S31-h adds four facts and a version to economy. Market Town's save records economy's declaration, so its `journal-300s` and `sealed-300s` digests move, though no fact or summary moves (F-S31h-1). How to keep the two-re-baseline ruling? (a) S31-h records a **composition-only** re-baseline with a reason line, measured to move no fact or summary; (b) S31-h merges only in one train with S31-f, whose content re-baseline covers both | **(a).** | It keeps S31-h independently mergeable; the ruling's intent (two behaviour changes) holds, and the measurement proves it |
| **Q-S31h-6** | After a seller declines, a stateless NPC buyer would propose again at once, forever (F-S31h-5). Keep a declined proposal visible, marked declined, until its 15 minutes end, and refuse the same buyer's new proposal for that kind to that seller until then? | **Yes.** | NPC buyers then pay the ask or walk on, as people do after a "no"; the state stays bounded |

### 14.1 Rulings

The operator froze S31-c … S31-h on 2026-10-20 in the primary session, answering every question as
recommended (relayed to the implementation session by the lane lead):

| Id | Ruling |
| --- | --- |
| Q-S31h-1 | `withdraw-proposal` is offered **incomplete**, like accept and decline |
| Q-S31h-2 | "Whichever merges second" also applies to S31-d: if S31-h merges first, S31-d adds the two haggling options (R-S31d-h-1) |
| Q-S31h-3 | Four typed `act` kinds, not four tools (`ARC-153` kept) |
| Q-S31h-4 | No LM haggling in S31-h; it arrives with the general "attempt an incomplete offer by name" mechanism |
| Q-S31h-5 | Option (a): a composition-only re-baseline, measured to move no fact or summary (Q-S31f-6 confirms it) |
| Q-S31h-6 | A declined proposal stays visible, marked declined, and blocks the same buyer, kind and seller for its 15 minutes |

---

## 15. Proposed edits elsewhere (made by the PR named)

- **step-31 §5.9** (lane lead or the primary, at this design's freeze): dated notes for Q-S31h-1 (withdraw
  incomplete), Q-S31h-3 (act kinds, not tools), Q-S31h-4 (LM haggling with the general mechanism), Q-S31h-6 (the
  decline memory), and SD-S31h-3/-6 (the accept target; the facts' fields).
- **step-31 §16.2 and §16.5:** the S31-h row depends on S31-d under the same "whichever merges second" rule
  (Q-S31h-2); "S31-h changes no reference world" becomes "S31-h changes no reference world's facts or summaries;
  its composition-only re-baseline is recorded" (Q-S31h-5).
- **step-31 §11:** the tools row names act kinds (Q-S31h-3), for S31-g's six actions too.
- **S31-d, S31-e, S31-g designs:** §10's requirements.
- **`docs/decisions/ARC-153.md`:** a dated note listing the new act kinds, by whichever of S31-g and S31-h adds the
  first economy kind (Q-S31h-3).

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] base, merge states, conditional commits chosen | [x] `ci_layer.py docs` | [x] §2.3 holds; `sale.rs` has the shape §5.6 splits |
| C1 | `60de7b85` | [x] `ARC-118`, index, MODULE_SPEC §4.1 economy row | [x] `check_decision_ids.py` (215 distinct), `check_doc_headings.py`, `ci_layer.py docs` | [x] no new ontology term; every §5.5 code named in `ARC-118` |
| C2–C4 | `564eeaae` (one commit, F-S31h-10) | [x] `proposal.rs`, `haggle.rs`, the four facts, the `sale.rs` split, wiring, version 5, catalogs (F-S31h-13) | [x] fmt, clippy `-D warnings` (economy, all targets); `--test haggle` 7/7, `--test haggle_settlement` 3/3 (10 000 states, 1.5 s), `--test haggle_persisted` 1/1; S31-a's `--test sale` 6/6, `--test settlement` 1/1, `--test buy` 6/6 unchanged | [x] §16 review notes below |
| C5 | `1c165295` | [x] the mirror reads the four facts; the scan ties `proposal-accepted` to its sale | [x] `cargo clippy -p mineworld-cli --tests -D warnings` (compiles the mirror); the runs are CI/milestone (R-LOAD-1) | [x] the `buy` path is unchanged: a cause with no proposal fact is judged exactly as before |
| C6 | N/A | N/A: S31-d not merged at C0; R-S31d-h-1 binds S31-d | N/A | N/A |
| C7 | `715d04c1` (on merge `208c2266`) | [x] §5.14: `MeLine.proposals` (`ProposalsLine`/`ProposalLine`, amounts with `display`), four typed `act` kinds bound to `Player.propose_price`/`accept_proposal`/`decline_proposal`/`withdraw_proposal` (accept's target is the buyer read from the seat's newest `proposals` record), `ProposalReading` for the four facts in the inbox; golden `docs/reference/agent-tools.json` and `.md` regenerated | [x] `uv run --locked pytest` one file at a time, all PASS: `test_market_acts.py` 14/14 (AC-h-9), `test_market_inbox.py` 9/9, `test_mcp_tools.py` 15/15, `test_mcp_secrets.py` 2/2, `test_world_text.py` 5/5, `test_cli.py` 9/9 (golden byte-equal to `mineworld-agent schema`), `test_markdown_reference.py` 4/4, `test_descriptions.py` 3/3, `test_reference_pages.py` 3/3, `test_mcp_resources.py` 10/10, `test_guide_chapter.py` 7/7, `test_market_view.py` 20/20, `test_inbox.py` 7/7, `test_view.py` 22/22; ruff and pyright clean | [x] the kinds are typed and closed (no generic free-form act, QS27-9/A-10); no error echoes agent input; the record is read from my own entity only (I-h-5) |
| C8 | `da543c28` (on merge `907d6fbb`, S31-e #422 merged) | [x] §5.13 through S31-e's panel model (R-S31e-h-1, S31-e merged first): `MineWorldTrade.answer_rows` reads the `proposals` record (incoming open lines: the buyer-targeted accept, whose verdict the row shows, and the target-less decline; outgoing: withdraw while open, none once declined; every button fills `{proposal}`); both clients' `TRADE_INPUTS` gain the four types (propose-price beside buy rows via `sale_rows`); both `intents` compose `{item, price}` and `{proposal}` and send the offer's own target (the 3D `compose_trade` sent `null` before); words in the shared `en.po`/`zh_Hans.po` (F-S31h-17); `clients/shared/TRADE.md` §§5.3, 5.6; economy's proposal macro spells each type with `from_static` so the client-rules scan knows them (F-S31h-18) | [x] `trade_check.gd` PASS (new `_proposals` case); Godot `--import` + `--check-only` of every script: 2D 0 failing, 3D 0 failing; `client_rules` 11/11, `client_text` 6/6 (`messages.pot` regenerated); `-p mineworld-economy` PASS; clippy `-D warnings` clean. AC-h-12's drives and three-OS shots run at the milestone (revision 2, SPEED FIRST) | [x] no client compares a proposal with an ask or a wallet (check e holds with `MONEY_ADMITTED` empty); both clients draw one panel model; nothing hidden by a verdict |
| C9 | milestone | the nightly `parity-long` dispatch runs at the milestone (charter SPEED FIRST; §12 endpoint) | milestone | milestone |
| C10 | (this commit) | [x] README table; I-h-1 diff scan clean (no path under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/inventory/`, `systems/item-transfer/`); PR opened | [ ] `fast` CI green on the exact head (recorded in the PR) | [x] every deviation recorded (F-S31h-10 … 15) |

**Left for the milestone (SPEED FIRST):** C9's `parity-long` nightly dispatch and composition-only re-baseline of
`[world market-town]` (AC-h-10, Q-S31h-5); the planted mutations X-13, X-14, M-h-1 … M-h-4; the independent Opus
review of C4; the full economy and `tools/cli` market runs (R-LOAD-1); the remaining economy test files and the
acceptance `client_text` test (the catalogs gained the four keys).

### Evidence

**C0 (2026-10-20).** Base `origin/main @ fec906f2` (#412). Merge states on the base: S31-a merged (#380,
`16f19d3f`), S31-b merged (#349), WE-a merged (#189), S29-b merged, S27-d1 merged (#261); Milestone F's checklist
handed over (milestone F batches 1–2 are on main). S31-d, S31-e and S31-g are **not** merged, so C6, C7 and C8 are
N/A here and R-S31d-h-1, R-S31e-h-1 and R-S31g-h-1 bind those PRs (§2.3). S31-c is implemented in parallel; both
PRs may touch `docs/MODULE_SPEC.md`, and whichever lands second merges. Q-S31h-2, -3 and -5 are ruled as
recommended (§14.1). The base Market Town 30-day save for AC-h-10 and C9's re-baseline are milestone work
(SPEED FIRST), not built here.

Re-audit of S31-a as merged (A-1, A-15): `systems/economy/src/sale.rs` holds `judge_sale(world, buyer, target,
item, submitted)` in S31-a's order 1–9, a private `Sale { seller: Seller, item, price }` and `settle(world, buyer,
target, &Buy)`, so §5.6's split applies. Bounded changes S31-a recorded and this PR builds on: `Shop::adjust`
(WE-a) is applied to a shop's asking price by `offer::asking_price`; economy's `SystemVersion` is **4** on the
base, so this PR raises it to **5** (§5.11 "or 3 → 4 if WE-a's landed between": one more step, recorded);
`EconomyConfiguration` carries `price_modifiers` and `currency`, untouched here. `ActionIntent::action_id()`
(A-4) and `WorldRead::at()` (A-5) are as audited.

**C2–C4 review (implementing agent, 2026-10-20).** The acceptance path reads no ask: `judge_acceptance` calls
`judge_authority` then `judge_exchange` on a `Sale` built from the stored proposal's price. Every `buy` check runs
through `judge_exchange`, the function `judge_sale` calls (I-h-7). The facts are built only through `pay`,
`inventory::transfer` (inside the shared `settle`) and economy's own proposal constructors. Audiences match
§5.7: all four proposal facts are `Participants` = buyer + person seller, or buyer + keepers standing in the shop
(`haggle::heard_by`). Only `proposal::reduce` writes `Proposals`, each reduction pruning dead entries first and
refusing a broken limit, a repeated id or a non-open entry. No float and no draw. Sizes: `haggle.rs` ≈ 560 lines
and `proposal.rs` ≈ 500 (above §5.1's estimate; both under the 800-line warning). The independent Opus review of
C4 is milestone work (SPEED FIRST).

**C7 (2026-10-26).** S31-g merged to main (#418) after C0, so C7 came into scope by §2.3's "whichever merges
second": `origin/main` was merged into this branch first (conflicts in `systems/economy/src/system.rs`, both
`Proposals` and S35's `ShopStock` kept, and `docs/MODULE_SPEC.md`, the haggling text and the `price-survey` row
both kept; `docs/decisions/INDEX.md` regenerated). AC-h-9 is asserted at the unit layer S31-g's privacy tests
use (planted real observations, `test_market_acts.py`): Bob's `outgoing` and Grace's `incoming` show proposal 7 at
3.50 against an ask of 4.50; Carol's view and her `inspect` of Grace carry no `proposals`; a record planted on
someone else never reaches the view. The real-server leg rides the milestone runs (SPEED FIRST).

### Mutations

Milestone (charter SPEED FIRST): X-13, X-14, M-h-1 … M-h-4 as §8.1 lists them are planted at the milestone, not
per PR. Each has a named test written here to turn red: X-13 →
`haggle.rs::acceptance_rechecks_every_buy_check_and_a_refusal_moves_nothing` (the funds arm); X-14 →
`haggle_settlement.rs::an_accepted_proposal_settles_atomically_at_the_proposed_price` (Alice and Carol never
hear `price-proposed`) and `haggle.rs::a_proposal_is_heard_by_two_shows_in_their_records_and_reserves_nothing`;
M-h-1 → `haggle.rs::a_proposal_expires_at_its_valid_until_with_no_fact`; M-h-2 →
`haggle.rs::the_limits_hold_and_a_decline_blocks_the_kind_until_it_expires`; M-h-3 →
`haggle.rs::every_haggling_offer_is_incomplete_and_per_target`; M-h-4 → see F-S31h-12.

### Deviations and findings

F-S31h-1 … F-S31h-9 (§3.2) and SD-S31h-1 … 10 (§11), recorded at design time.

- **F-S31h-10 (bounded) — C2, C3 and C4 are one commit.** The state, the two buyer actions and the two seller
  actions share `haggle.rs`, `proposal.rs` and the `system.rs` wiring; splitting them would have produced two
  intermediate heads with half-wired dispatch. SPEED FIRST; the ledger rows C2–C4 share one SHA.
- **F-S31h-11 (bounded) — §5.6's split, as built.** `Sale` gained `buyer`; the step-5 keeper check reads the
  `Shop` at the sale's place rather than a flag carried from `resolve_seller`, so `judge_exchange(world, &Sale)`
  needs nothing else and acceptance runs it unchanged. `judge_exchange` is `judge_self_trade` then `judge_goods`;
  `propose-price` runs the limit check between the two (§5.5 step 6 sits between steps 5 and 7).
  `settle(world, &Sale)` is the shared settlement; `settle_buy` re-judges a `buy` and calls it. No `buy` outcome
  changes: the order of steps 1–9 and every code are as S31-a's; `sale.rs`, `settlement.rs` and `buy.rs` run
  unchanged at the milestone (R-S31h-1).
- **F-S31h-12 (bounded) — tests written against the fixture's own people.** The store fixture has no Dave and no
  1 m bystander: "Grace gives her last bread to Dave" is Alice buying Grace's last loaf; the bystander in W1 is
  Alice (2 m away, same place) plus Carol on the street; "Bob walks to 3.1 m" is a `move` to (3 500, 1 000), 3.2 m
  from Grace. W2 is the same streets re-authored (crowns, exponent 0, two keepers with one outside). AC-h-6's
  generator varies the buyer's wallet, the proposed price, the seller's stock, a spend, a sell-off, a walk-away,
  the buyer's load and the age (0 … 1 200 s), for a person seller and the shop. M-h-4's forged ninth entry needs
  a test-only stater past `validate`; it is a milestone item.
- **F-S31h-13 (bounded) — the client catalogs.** `tests/acceptance/tests/client_text.rs` (AC-SET-4) needs an
  `action.<type>` key for every declared action type, so the four actions get `action.*` and `.done` lines in
  `en.po`, `zh_Hans.po` and `messages.pot` (the a7d145f5 precedent). These are not C8's sale-panel rows, which
  stay R-S31e-h-1.
- **F-S31h-14 (bounded) — the `proposals` record's component type.** The record is disclosed under the
  `proposals` component type (the state's), as S31-a discloses `AsksView` under `asks`: a view about the observer
  itself, never the stored state. No second component type is declared.
- **F-S31h-15 (bounded) — the fixture support.** `tests/support/mod.rs` gained `Town::assemble` and
  `Town::compose` (the save test's world before genesis, and a fresh composition to resume into), a `log` of every
  fact for presence's `perceived_by`, and `Town::request` for any action.
- **F-S31h-16 (bounded) — C7 also touches `disclosed.py` and `__init__.py`.** S31-g's pattern reads every pack
  record in `disclosed.py` (the `proposals` parser and the buyer lookup for `accept_proposal`) and exports each
  `*Args` from the package; §5.14's file list named the other five. A proposal number is `ProposalNumber`
  (1 … 2^64−1, economy's `ProposalId`); an `accept_proposal` whose number is not in my newest `incoming` is refused
  `not_offered` before anything is sent, an existing adapter code.
- **F-S31h-17 (bounded) — C8's word keys follow S31-e's catalog families.** §5.13 named `sale.*` and `refusal.*`
  keys; the shared catalogs admit only the families of `tests/acceptance/tests/client_text.rs` (`KEY_FAMILIES`),
  where S31-e's panel words are `trade.*` and refusals `reason.*`. So the rows use `trade.offer-price`,
  `trade.incoming`, `trade.incoming.ask`, `trade.accept`, `trade.decline`, `trade.your-offer`, `trade.declined`,
  `trade.withdraw-offer`, and the refusals `reason.no-such-proposal`, `reason.proposal-limit`. The
  `proposal-limit` detail (`seller-full`, `already-proposed`) is not worded separately: the reason formatter
  words a code, not a detail, and one sentence covers both. `answer_rows` gains an optional `clock` (the bare
  world time without one), as `board_rows` takes one; existing callers are unchanged.
- **F-S31h-18 (bounded) — the scan learns the three proposal types.** `client_rules`' `action_types` reads
  `ActionTypeId::from_static("…")` literals; economy's `proposal_action!` macro built the call from a literal
  argument, so the scan knew only `propose-price` and the clients' admissions of the other three admitted
  nothing. The macro now takes the `ActionTypeId` expression, spelled at each call. No behaviour change.

# PR S27-g — Order offers by the category each System Pack declares for its actions

## DESIGN FROZEN 2026-10-20 (operator approved the approach in the primary session)

```text
Design revision:        revision 0 (2026-10-20), FROZEN 2026-10-20
Branch / worktree:      mvp0/pr-s27g-offer-order, /Users/yuema137/mineworld-worktrees/impl-offer-order
Planning base:          origin/main @ fec906f2
Approved by / evidence: operator ruling 2026-10-20, relayed by the primary session: "order the offers an agent
                        sees by the category each System Pack declares for its actions — social and interaction
                        before commerce — never by action name"
Execution contract:     this document (design and implementation land in one PR)
Lifecycle:              DESIGN FROZEN 2026-10-20
```

**Effort:** `mvp0` · **Step:** [`step-27-agent-players.md`](step-27-agent-players.md) · **Follows:** #408
(paging made obvious to agents).

## 1. Problem

An observation lists its affordances in the order perception produced them: target by target (no target first,
then each present entity), and inside one target in the order the World Pack composed its perception providers
(`systems/presence/src/observe.rs`, `affordances`). Nothing in that order says which offers matter to a person
walking into a place. When S31-a's economy actions (`buy`, `list-for-sale`, `set-price`, `withdraw`) reached the
default town, they filled the first window of the agent view's `offers` list, and `talk` moved to page 2. #408
made paging obvious; it did not fix the order.

Sorting by action name would be world-specific hard-coding in disguise: a new pack's action name says nothing
about what it is. The pack that defines an action knows what kind of act it is, so the pack declares it.

## 2. Design

### 2.1 Where a category is declared

The `Action` trait in `mineworld-contracts` (the kernel's action contract, which every action already implements
to declare `ACTION_TYPE` and `OWNER`) gains one associated constant with a default:

```rust
pub trait Action: Serialize + DeserializeOwned + Sized {
    const ACTION_TYPE: ActionTypeId;
    const OWNER: SystemId;
    /// What kind of act this is, which orders it among the offers an observer sees.
    const CATEGORY: ActionCategory = ActionCategory::Interaction;
}
```

`ActionCategory` is a closed enum in `mineworld-contracts` (`contracts/src/action.rs`), declared in rank order:

| Rank | Variant | Meaning | Examples in this workspace |
| --- | --- | --- | --- |
| 0 | `Social` | An act directed at another person as a person: speaking, inviting, answering an invitation, joining or leaving a shared activity | `talk`, `invite`, `accept-invitation`, `decline-invitation`, `join-group-activity`, `leave-group-activity` |
| 1 | `Interaction` | Any other act on the world, the actor's own body or a thing: sitting, eating, sleeping, giving, pushing, subscribing | `sit`, `eat`, `give`, `shove`, `subscribe` |
| 2 | `Commerce` | An act whose point is a trade or the terms of one: buying, listing, pricing, withdrawing stock | `buy`, `list-for-sale`, `set-price`, `withdraw` |

Why the trait and not a pack manifest field: the category belongs to the action, the trait is read off the type
(`Offer::new::<A>` already reads `A::ACTION_TYPE` this way), so an offer cannot carry another action's category,
and a pack declares it next to the action it describes. A manifest field would be a second, string-keyed table
that can drift from the actions it names.

Kernel ignorance holds: the kernel never reads the category and does not branch on it. The enum is a declared,
closed vocabulary for ordering what an observer is offered; it is not a rule about money or conversation. A world
needing a finer order is a later change to this table, reviewed as a contract change.

### 2.2 Default for packs that declare none

`Interaction`. An undeclared action is an act on the world; it ranks after social acts and before commerce, which
is the operator's order. No existing pack is forced to edit anything, and a new pack that forgets to declare is
never pushed behind commerce or ahead of conversation.

### 2.3 Where the order is applied, and the stable tiebreak

Once, server side, in presence's `observe` (`systems/presence/src/observe.rs`). `Offer` records `A::CATEGORY`
when it is built (`Offer::new`, and so `Offer::complete`). After every offer is priced, the list is sorted with a
**stable** sort by category rank. Inside one category the previous order is kept exactly: target by target (no
target first, then present entities in perception order), then provider composition order, then each provider's
own order. That order is already deterministic (it is what `AC-12` replay comparison relies on), so the sorted
order is deterministic too. The action name is never consulted.

The wire format does not change: an `Affordance` carries no category. The order of the list is the information.

### 2.4 Which clients use it

Every client takes the observation's order as given, so one sort serves all of them:

- the agent adapter (MCP and the library): `view.py` numbers offers "in the observation's order" and windows them,
  so the first page of `offers` now shows social acts first;
- the 2D client's interaction menu (`clients/2d/scripts/menu.gd`) lists "in the server's order";
- the 3D client reads the same observation through the shared protocol and inherits the same order.

The rule and motivated controllers search affordances by action type and are unaffected.

### 2.5 Agent reference

`View.offers` (and `ViewDelta.offers`) get a field description saying the list is ordered by the category each
pack declares — social, then interaction, then commerce — so an agent knows `talk` is near the top. The tool
reference (`docs/reference/agent-tools.json`, `docs/reference/agent-tools.md`) is regenerated with
`mineworld-agent schema`.

## 3. Changes

| File | Change |
| --- | --- |
| `contracts/src/action.rs`, `contracts/src/lib.rs` | `ActionCategory`; `Action::CATEGORY` with default `Interaction` |
| `systems/conversation/src/action.rs` | `talk` → `Social` |
| `systems/group-activity/src/action.rs` | the five invitation and activity actions → `Social` |
| `systems/economy/src/action.rs` | `buy`, `list-for-sale`, `set-price`, `withdraw` → `Commerce` |
| `systems/presence/src/interaction.rs` | `Offer` records the category; `Offer::category` |
| `systems/presence/src/observe.rs` | stable sort of the priced affordances by category |
| `systems/presence/tests/presence.rs` | integration test (§4) |
| `clients/agent/src/mineworld_agent/models.py`, `clients/agent/tests/test_descriptions.py` | `offers` descriptions |
| `docs/reference/agent-tools.json`, `docs/reference/agent-tools.md` | regenerated |

## 4. Acceptance

- **AC-S27g-1.** A world whose providers offer a `Commerce` action before a `Social` one, and an undeclared action
  between them, produces an observation listing social, then the undeclared (interaction), then commerce; offers
  of one category keep their composition order. Tested in `systems/presence/tests/presence.rs` with test-only
  actions and providers — a second, generated composition, not the default town.
- **AC-S27g-2.** No code compares action names to order offers (review of the diff).
- **AC-S27g-3.** The regenerated reference matches `mineworld-agent schema` output (the existing reference test).

## 5. Ledger

- [x] C1 implementation — contracts, packs, presence, agent descriptions, regenerated reference.
- [x] C1 deterministic validation — `cargo test -p mineworld-presence`, `cargo clippy` on the touched crates, the
      agent's description and reference tests (evidence in the PR body).
- [x] C1 LLM logic review — self-review against §2: stable sort, default rank, no name comparison.

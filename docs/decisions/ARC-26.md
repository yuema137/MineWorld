## ARC-26 — Movement decides, presence owns: an event type's owner is its vocabulary and its reducer

**Date** 2026-09-30 · **Implements** [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §6,
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§11, 13 · **Relates to** `INV-7`, `INV-9`, `INV-10`,
[`MVP.md`](MVP.md) §9 `AC-2`, `ARC-15`, `ARC-23`, `ARC-25` · **Design**
`.structured-coding/plans/mvp0/step-07-movement.md` (S6, PR 08)

**Problem.** S6 adds movement. `PresenceSystem` already owned where people are and provided `arrive`,
an action that put a person anywhere and checked only identity. Two questions followed. Where does
movement live, given that `ConversationSystem` depends on presence and the registry refuses to
disable a system another enabled system depends on? And how does a movement system change presence's
state, given that presence must not know movement exists and movement must read presence's state to
validate a move — a dependency in each direction is a Cargo cycle?

The audit also found an existing contradiction. `contracts/src/event.rs` documented `Event::OWNER` as
"the system that emits it", while `Dispatcher::record` (`kernel/src/dispatch.rs`) only ever checked
that the *running* system's own declaration lists the type — never that it is the type's owner. The
specification and the kernel already disagreed (`CLAUDE.md` §2.1(4)).

**Options considered.**

```text
(a) movement inside PresenceSystem      disabling movement disables presence; refused while conversation
                                        depends on it, so AC-2 for movement is unreachable
(b) MovementSystem owns location        presence and conversation would depend on movement; removing
                                        movement removes location and perception
(c) movement emits its own fact and     presence imports movement's event while movement imports
    presence reacts                     presence's component: a cycle, and presence names another pack
(d) movement keeps its own positions    two truths about one person's position
(e) movement decides; it states         movement → presence, one way; presence keeps the only state
    presence's own Arrived, built by    and never learns movement exists
    presence's checked constructor
```

**Choice: (e), and `arrive` is retired.**

1. **`Event::OWNER` is the event type's vocabulary owner**: the system that defines its schema and its
   public constructor, and **the only system that reduces it into owned state**. It is not merely the
   only system permitted to emit it. Provenance names the system that *stated* a fact; `OWNER` names
   whose vocabulary it is. They differ exactly when one system decided what another records.
2. **Another system may state a fact of that type only if it** declares the emission, **depends on the
   owner** (both refused at installation otherwise — `KernelError::EmittedEventOwnerNotADependency`),
   and builds the payload through the owner's public constructor.
3. **The owner still decides.** The owner's constructor checks the fact against the world and refuses
   one its state may not take; its reduction checks again, writes nothing, and fails with
   `KernelError::FactRefusedByOwner` if a fact built some other way reaches it. Movement decides
   whether a *move* is legal; presence decides whether its *state* may take the value.
4. **One movement path.** `arrive` is removed. A distance rule beside an unrestricted relocation is
   not a rule: `TooFarAway` would hold for the action and not for the world. Initial placement is a
   genesis fact (`ARC-15`), so nothing needs `arrive` to exist.

The resulting split:

```text
PresenceSystem   owns Presence and present-in; owns Arrived and PersonEnteredPlace; provides no action
MovementSystem   depends on presence; provides `move`; owns Passages (which places open onto which, and
                 where the doorway is) and PassageOpened; states presence's Arrived when a move is legal
```

`move` is decided entirely by `MovementSystem::validate` through `SpatialRequirement::evaluate`: a
stride of at most `MAX_STRIDE` (2 000 mm) inside a place; into another place only through a passage,
within a stride of the doorway on both sides; otherwise `TooFarAway`. `PersonEnteredPlace` is
presence's, emitted while it reduces an `Arrived` that changes a known place, because occupancy is
presence's state whoever caused the arrival.

**Why this does not weaken single ownership (`CLAUDE.md` §4 rule 1).** That rule is about mutable
state. `Presence` is still written by exactly one thing — presence's own reduction — and presence can
refuse the value. Exposing an owner's vocabulary to declared dependents is the smallest acyclic form of
"emit a fact and let the owner decide".

**Room for travel.** Travel between places that do not open onto each other is a `Process` that takes
simulated time (`ENGINEERING_RULES.md` §6). A future travel system depends on presence and states
`Arrived` when its process ends, under this same rule. `move` refuses exactly the requests such a
system would accept.

**The client reporting rule.** A per-request stride bound is correct only if clients report often
enough: a 3D client jogging at about 2.6 m/s that reports once a second would be refused for moving
legally. So clients **report before travelling `MAX_STRIDE` since their last accepted position**
(`server/PROTOCOL.md`, `clients/protocol/ADOPTION.md`).

**Accepted limitations.** The stride bounds one request, not requests per second: there is no speed
model, which needs time accounting finer than `WorldTime`'s one second. Walls inside a place are not
evaluated (line of access needs geometry no layer owns, `DD-7`). Passages are stated at genesis and are
always open; doors are a later system. `MAX_STRIDE` is a constant until world configuration exists
(S7). Saves made before this decision are refused by name (`SAVE_FORMAT` 2), not migrated.

**Note, 2026-10-06 (S7, step-08 §10.1 Q5).** S7 did not make `MAX_STRIDE` world configuration. No
pack yet needs another value, and a configuration mechanism built for one constant is the premature
abstraction `CLAUDE.md` §4 rule 11 forbids. It stays a published constant until the first pack that
needs a different stride, which introduces System Pack configuration (`MODULE_SPEC.md` §9) then.

**Note, 2026-10-09 (S15, PR 12n-1; step-11 §21, SD-N1, SD-N2) — walking is not the travel Process.**
`movement` gains a second scale of the same spatial model inside a place and across adjoining places:
`walk-to` (go to a destination) and `walk-step` (the next stride of my walk), with the walk held as
`movement`'s `Walking` component (`ARC-75`). It is not the "Room for travel" above: a walk takes no
calendar time and starts no `Process`; each stride is one request, at most `WALK_STRIDE` (1 340 mm),
checked by the same rule as `move` and stated through presence's `arrivals()`. Travel between places
that do not adjoin by a passage chain remains the later travel system's, a `Process` in calendar time.
`move` is unchanged and still one stride: a walker's own `move` ends their walk.

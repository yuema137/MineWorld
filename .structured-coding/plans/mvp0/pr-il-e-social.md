# PR IL-e — The World Interaction List for the social packs: `talk`, `invite`, `join`, `acquaint`

## DESIGN FROZEN 2026-10-10 (primary session)

```text
Design revision:        revision 2 (2026-10-10): revision 1 (8abea2e on docs/il-e-design) with §13's
                        rulings recorded and §12's contract filled, as committed with this header
Approved by / evidence: the operator accepted every recommendation, QIE-1 … QIE-12 (including the [OM]
                        questions QIE-6 and QIE-12), on 2026-10-10; QIE-9 ruled "dated notes on ARC-63
                        and ARC-65 only, no new ARC". Relayed by the primary session to this planning
                        session (§13 "Rulings")
Implementation base:    main at the start of implementation (exact commit recorded in IE-C1's evidence)
Execution contract:     §12 (filled at freeze)
Lifecycle:              FROZEN
```

Scope (§1), section shapes (§4), decisions SD-IE-1 … SD-IE-12, acceptance IE-1 … IE-13 and the commit plan
are frozen. Progress, evidence, findings and bounded corrections stay writable (§10, §14, §15). No IL-e
question remains open. Implementation starts in a fresh session (§12); this planning session does not
implement.

*Superseded header:* `DRAFT — PR design, awaiting the primary session's review (NOT frozen)`, revision 1.

**Effort:** `mvp0` · **Step:** S17, [`step-18-interaction-list.md`](step-18-interaction-list.md) (§2.2 rows
conversation / relationships / group-activity, §4.4–§4.8, §5 IL-I1 … IL-I11, §6.2 IL-e row, §12 IL-b and its
deviations D-IB-1 … D-IB-15, §12.15 IL-e row) · **Parent:** [`overall.md`](overall.md) "The World Interaction
List" (operator rulings QIL-3, QIL-7, QIL-8, QIL-10, QIL-17; primary rulings QIL-2 overruled, QIL-4 … QIL-6,
QIL-9, QIL-11 … QIL-16, QIL-18 … QIL-20).
**Predecessors (merged):** IL-a (#80, `543c80a`), IL-b (#102), S11-C (#95, `370bb38`: `audience::admits`,
`mineworld perceived`).
**Siblings in parallel:** IL-f (ownership packs), IL-g (movement, employment, schedule). Disjoint packs; the
shared lines are listed in §9.
**Planning base:** `origin/main @ bb62edf` (#140). Planning worktree
`/Users/yuema137/mineworld-worktrees/design-il-e`, branch `docs/il-e-design`, one writer.
**Decision numbers:** none (QIE-9, ruled 2026-10-10). IL-e implements `ARC-63` and `ARC-65` and adds dated
notes to them only.

---

## 1. Identity, goal, scope

```text
PR            IL-e — sections of the World's Interaction List for conversation, group-activity and
              relationships: rules, parameters, consequences and one knob. S17, a conversion PR
base          main after IL-b and S11-C (both merged at bb62edf). Re-audit §3 if any of these moved:
              systems/{conversation,group-activity,relationships,presence}/src, sdk/rust/src/interactions,
              tools/cli/src/{biography,perceived,interactions}.rs, tools/cli/tests/{interactions,inspect,
              social_composition,interaction_runs}.rs, worldpack/tests/interaction_sections.rs
branch        mvp0/pr-il-e from main, worktree /Users/yuema137/mineworld-worktrees/impl-il-e (named at
              freeze), held by the implementing session only
depends on    IL-b (sections for conversation and group-activity exist; Offer::refused; the biography
              projection reads sections); S11-C (the bystander proof, QIB-12)
merge         a merge commit, never a squash (ARC-5)
```

**Goal.** A world can say, by configuration alone, who may talk to whom, invite whom and join whom; whose
contacts form relationships; how far a voice and an invitation reach; how long an activity lasts; how much a
contact moves familiarity and regard; who overhears a line; whether a social fact enters a biography; and
whether a listener keeps a line in conversation's in-world history. A world that configures none of it runs
byte for byte as before.

Concretely, the operator's examples R-IL-2 that live in these packs become one list entry each:

```yaml
# configure/conversation.yaml — two classes do not talk; nobody overhears; a class's lines are not history
rules:
  - { action: talk, actor: noble, target: commoner, effect: forbid }
  - { action: talk, actor: commoner, target: noble, effect: forbid }
consequences:
  - { fact: spoke, biography: on }                      # MVP-0's compiled default is off (F-IE-9)
  - { fact: spoke, actor: servant, biography: off }
  - { fact: spoke, audience: participants }
```

**Approved scope (frozen 2026-10-10).** §6.2's IL-e row and §12.15's IL-e row, made concrete in §4–§5:

1. conversation: the `talk` rule; parameters `range` (was `INTERACTION_RANGE`) and `remembered` (was
   `REMEMBERED_AT_MOST`) beside IL-b's `gap`; consequences of `spoke` and `conversation-started`; the knob
   `remember`.
2. group-activity: rules for `invite`, `accept-invitation` and `join-group-activity` (QIE-2); parameters
   `invite_range` (was `INVITE_RANGE`) and `activity_length` (was `ACTIVITY_LENGTH`) beside IL-b's
   `invitation_lifetime`; consequences of its seven facts (QIE-3).
3. relationships: a new section; the `acquaint` rule (QIE-1); its five contact parameters (was
   `SPOKE_FAMILIARITY` … `ACTIVITY_REGARD`); consequences of `became-acquainted` and `relationship-changed`.
4. The proofs the IL-b freeze deferred to this PR: a configurable biography through the binary (QIB-11) and a
   narrowed fact not perceived by a bystander, through S11-C's unchanged `audience::admits` and
   `mineworld perceived` (QIB-12).

**Non-goals.**

- Any SDK, authoring, worldpack-source or tools-source change. The schema is IL-b's and is used as merged
  (R-IL-6). A need to reshape it is a material stop (§12), not a bounded fix.
- A hearing range or any perception refinement: overhearing is place-level for MVP-0 (operator, QS11C-6). The
  list only narrows `spoke` from `Place` to `Participants`.
- presence, naming and item sections (QIL-9). Ownership packs (IL-f). movement/employment/schedule (IL-g).
- Any world's content: no file under `worlds/` changes; the demonstration world is IL-h.
- Changing a compiled default. Every default equals today's constant, so IL-I1 holds pack by pack.
- Telling minds what to forget (QIL-10); attribute conditions (QIL-11); mutable classes (QIL-7).
- `UTTERANCE_MAX_BYTES`, `KIND_MAX_BYTES`, `FAMILIARITY`, `REGARD`: engine bounds (E), never configurable
  (step-18 §12.3).

---

## 2. Binding inputs

- `CLAUDE.md` §4 rules 1–3, 5, 7, 8–10, 15; kernel ignorance and single ownership: each pack decodes and
  enforces its own section; nothing social enters `kernel/`, `contracts/`, `persistence/`, the SDK or the
  loader.
- `ARC-63` items 1–10 (shape, levels, specificity, forbid-overrides, ambiguity refused at load, storage,
  lookups, enforcement in the owner, cannot grant), `ARC-64` (classes from tags, fixed during play), `ARC-65`
  (audience narrows only, biography where the owner allows, memory only through audience), `DEP-28`.
- `ARC-29` as amended by IL-b (the biography projection asks configured sections), `ARC-34` and its IL-b note
  (`Offer::refused`), `ARC-43` (S11-C's audience rule), `ARC-25` (a SystemVersion change refuses old saves).
- IL-b's deviations, which are now the schema's facts: D-IB-1 (reference lists in Rust), D-IB-2 (decode-time
  refusals are `Malformed` with line and column), D-IB-3 (`interactions!()` shape), D-IB-5
  (`PARAMETER_ROLES`), D-IB-7 (a pack derives `Clone`, `Debug`, `PartialEq`), D-IB-12 and D-IB-14 (version
  pins in CLI tests move with a version bump).
- Operator QS11C-6: overhearing is place-level in MVP-0.
- Operator requirement (2026-10-08): macOS, Linux and Windows. Every test this PR adds is platform-neutral
  (no `std::os::unix`, no `/tmp`, no SIGKILL); the SIGKILL proofs stay where 13w put them.

---

## 3. Source audit (`origin/main @ bb62edf`, 2026-10-10)

Every finding was read in source, not inferred from a plan.

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-IE-1** | conversation has a section since IL-b with one parameter (`gap: u32 = 300, 1 ..= 86_400`), `Knobs = ()`, `PARAMETER_ROLES = [Actor, Target, Place]`, no `ACTIONS`, no `FACTS`; VERSION 2; the pinned test `the_default_section_is_pinned_to_the_version` asserts VERSION 2. | `systems/conversation/src/interactions.rs:27–77`; `system.rs:62–64` | IL-e grows the same section. The pinned test moves to VERSION 3 (§5.4). |
| **F-IE-2** | `talk`'s validate order: payload → listener ≠ speaker → speaker is a Person → listener exists and is a Person → speaker located → `talk_requirement().evaluate`. Offers: one `Offer::new::<Talk>(talk_requirement())` per Person target, availability = target `Active`. No `permits` anywhere. | `system.rs:97–124`, `:230–253` | `permits` goes after the listener check and the speaker's location (it needs the place), before `evaluate` (QIL-13, `ARC-63` item 8); the offer calls the same `permits` and `Offer::refused` on `Err`. |
| **F-IE-3** | `INTERACTION_RANGE` (3 000 mm) is read only through `talk_requirement()`, which takes no argument. `talk_requirement()` is called outside the pack by five rule-controller test files and one conversation test, never by rule-controller production code; the controller reads the requirement from the affordance. | `action.rs:16, :76–81`; `git grep talk_requirement` (cognition/rule-controller/src/{agenda,offered,paced}_tests.rs, tests.rs; systems/conversation/tests); `invite_requirement` likewise in `social_tests.rs:14, :169` | Add `talk_requirement_within(range)`; `talk_requirement()` stays the compiled default's requirement (`within(INTERACTION_RANGE)`), so no cognition file changes (QIE-8). validate and offers call `_within` with the looked-up range. |
| **F-IE-4** | `spoke` is stated with `Visibility::Place(place)`, subjects `[listener]`, participants `[speaker, listener]`; `conversation-started` with `Visibility::Participants`, subjects and participants `[speaker, listener]`. conversation declares no biographical fact (`BIOGRAPHICAL` absent ⇒ empty). | `system.rs:157–186`, `:49–52` | `FactDecl`s in §4.1. `spoke`'s audience narrows `Place → Participants`; `conversation-started` is already `Participants`, so only its biography is configurable. |
| **F-IE-5** | `ConversationHistory::remember` drops the oldest beyond the constant `REMEMBERED_AT_MOST` (32); `react` reduces every `spoke` into the listener's history. `continues_a_conversation` reads **both** parties' histories to decide `conversation-started`. | `component.rs:18, :88–93`; `system.rs:194–217`, `:380–395` | A per-listener bound becomes `remember_within(heard, at_most)`. The `remember: off` knob skips the insert, which can make the next exchange from that speaker start a new conversation (§4.1, QIE-5). |
| **F-IE-6** | group-activity has a section since IL-b: `invitation_lifetime: u32 = 1_800, 1 ..= 86_400`, `Knobs = ()`, `PARAMETER_ROLES = [Actor, Target, Place]`; VERSION 2. Its codec for the section is `serde_json` inline. | `systems/group-activity/src/interactions.rs:24–73`; `system.rs:61–63` | Grown the same way; VERSION 3. |
| **F-IE-7** | group-activity's validate: actor located first (`here`), actor a Person; `leave` handled first; then target exists and is a Person; then per action: `invite` → `invite_requirement().evaluate`; `accept`/`decline` → open invitation, (accept) not Busy, evaluate; `join` → not Busy, evaluate. **Accepting an invitation joins the inviter's running activity** (`resolve`: `Some(activity) => join(..)`). Offers mirror this. | `system.rs:108–170`, `:207–230`; `perception.rs:35–95` | A `join` rule alone would leave a hole: a forbidden pair can still come together through `accept-invitation`. So the section governs `invite`, `accept-invitation` and `join-group-activity` (QIE-2). `decline` and `leave` stay ungoverned: saying no and leaving are always possible. |
| **F-IE-8** | group-activity facts: `invited`/`invitation-accepted`/`invitation-declined` are `Participants`, subjects `[invitee]`, participants `[inviter, invitee]`; `group-activity-started`/`-ended` are `Place(place)`, subjects = participants = the members list (variable length); `joined-`/`left-group-activity` are `Place(place)`, subjects = participants = `[person]`. `BIOGRAPHICAL` = started, joined, left, ended. | `event.rs:68–73, :146–152, :210–214`; `system.rs:50–55` | §4.2's `FactDecl` table. For started/ended only the `place` role is declared, because the members list has no fixed positions (F-IE-10). |
| **F-IE-9** | **No line is biographical in MVP-0**: `spoke` is in no `BIOGRAPHICAL` set, so `{ fact: spoke, actor: X, biography: off }` alone changes nothing. | F-IE-4; `sdk/rust/src/interactions/lookup.rs:218`, `biography.rs:129–156` | §6.2-IL-e's checkpoint "a `spoke` with `biography: off` absent from biographies" would pass vacuously. IE-6 turns `spoke` **on** for the world and **off** for one class, and asserts both halves. IL-h's manor list needs the same two entries (QIE-12 [OM]). |
| **F-IE-10** | `biography::selected` decides **per fact, not per reader**: the roles are filled from the envelope, and the answer is the same for every person the fact names. | `sdk/rust/src/interactions/biography.rs:119–157` | `{ fact: spoke, actor: servant, biography: off }` removes a servant's line from the servant's **and** the listener's biography. That is what "servants' lines are not history" means; it is recorded as an accepted limitation (QIE-6). A multi-member fact can only be scoped by `place`. |
| **F-IE-11** | relationships has no section, no actions, derives only `Default`, VERSION 1. It reacts to `spoke`, `invitation-accepted`, `invitation-declined`, `group-activity-ended`; each change is directed `(person, counterpart)`; the five increments are `pub const`s. Its facts are `Participants`, subjects `[person]`, participants `[person, counterpart]`, at the cause's place when it has one. `BIOGRAPHICAL` = both facts. `serde_json`, `mineworld-sdk` are already dependencies. | `systems/relationships/src/system.rs:21–98, :124–236`; `Cargo.toml` | Adding a section needs `Clone, Debug, PartialEq` derives (D-IB-7), `interactions!()`, `declare`/`install`/`reduce`, VERSION 2. No `Cargo.toml` or `Cargo.lock` change. |
| **F-IE-12** | The SDK's rules are keyed by `ActionTypeId` (`ActionDecl.action`, `permits(.., action: &ActionTypeId, ..)`). relationships provides no action: whether a pair forms a relationship is decided in `react`. | `sdk/rust/src/interactions/decl.rs:74–83`; `lookup.rs:165–182` | `acquaint` is a rule name relationships declares for a decision it makes in a reaction, never an action a client can send (QIE-1). It needs a dated note on `ARC-63` (its item 1 says "rule: an action and role selectors"). |
| **F-IE-13** | `permits` and `parameters` take a `PlaceId`; only `consequence` takes `Option<PlaceId>`. relationships' four causes are all stated `at_place` (`spoke`, the invitation facts, `group-activity-ended`). | `lookup.rs:165, :185, :211`; F-IE-4, F-IE-8 | relationships looks up at the cause's place. A cause without a place cannot occur for these four types; if one did, the compiled default applies (permit, default parameters), and a unit test pins that every subscribed cause carries a place. No SDK change. |
| **F-IE-14** | `Fields` knobs: any `Fields` type works; `parameters!` generates one (the partial twin, every field `Option`). serde-saphyr reads YAML 1.1 booleans (`on`/`off`/`yes`/`no`) unless `strict_booleans` is set; the loader does not set it. | `section.rs:24–66, :89–188`; serde-saphyr 1.3.0 `de/deserializer.rs:929–948`, `parse_scalars.rs:18–35`; `worldpack/src/configure.rs:158` | conversation's `Knobs` is the partial twin of a `parameters!` block with `remember: bool = true, false ..= true`; `remember: off` decodes. No SDK change. |
| **F-IE-15** | S11-C is merged: `mineworld_presence::audience::admits` reads only the envelope's Visibility, and `mineworld perceived <world> --save S --person KEY [--json]` exports a person's perceived facts. | `systems/presence/src/audience.rs:1–30, :104–124`; `tools/cli/src/perceived.rs`; `tools/cli/tests/perceived.rs` | The bystander proof QIB-12 deferred lands here (IE-7), with no change to presence or the CLI. |
| **F-IE-16** | Tests that pin what IL-e changes on purpose: `tools/cli/tests/inspect.rs:36–37` and `social_composition.rs:379–380, :419–420` pin "conversation v2, group-activity v2, relationships v1"; `tools/cli/tests/interactions.rs:205–208` pins "'whisper' is not an action 'conversation' declares (it declares: none)"; `interactions.rs:72–89, :167, :188` pin "default (compiled)" per section (relationships gains one). | as cited | Edited on purpose, claim by claim (§8). |
| **F-IE-17** | social-cafe's tags: residents `carol`, `dev`, `erin`, `otto` (otto is not a seat); commuters `grace`, `hana`; `alice` `barista`+`staff`; `bob` `regular`; `felix` `neighbour`; `ivan` `passerby`; `visitor`, `wanderer` `visitor`. Places: `cafe` (`cafe`, `public`), `park` (`park`, `public`, `outdoor`), `workplace` (`workplace`, `office`) … | `worlds/social-cafe/{people,places}/*.yaml` | The scratch classes of §7 use `resident` and `commuter`, both with seats on each side. |
| **F-IE-18** | The paced rule controller attempts only affordances the server marks available (`is_available()`), for `talk` and for every social action. | `cognition/rule-controller/src/offered.rs:44`, `social.rs:112–130`, `lib.rs:208` | A forbidden pair is never attempted by the controller when the offer is refused, so "every attempt refused" is proven by **zero** `PermissionDenied` rejections in the run plus a scripted dispatch (IE-4, M-IE1/M-IE2). |
| **F-IE-19** | `mineworld run`'s printed output is facts, request outcomes and the fingerprint; no line prints a component or a SystemVersion. Last recorded 300-day seed-7 values: social-cafe `ad49c723…c64b` (365 330 facts), market-town `365b50e0…1d1d`. 12d has not merged. | `tools/cli/src/run.rs:422–481`; step-19 E-TWb; step-18 E-IB-0 | E-IE-0 is captured on the base before any code; IL-I1 compares against it, never against a remembered value. |

---

## 4. Section shapes, per pack

Each table is what the pack declares through `InteractionSection` (IL-b's trait) and what an author may
write. "default" is the compiled value and equals today's constant. Roles in rules and consequences are
only those listed; anything else is refused at its line and column (D-IB-2).

### 4.1 `configure/conversation.yaml` (owner: conversation; VERSION 2 → 3)

```yaml
extends: default
default: permit
rules:
  - { action: talk, actor: noble, target: commoner, effect: forbid }
parameters:
  - { gap: 300, range: 3000, remembered: 32 }
  - { actor: guard, range: 6000 }
  - { target: child, remembered: 8 }
consequences:
  - { fact: spoke, biography: on }
  - { fact: spoke, actor: servant, biography: off }
  - { fact: spoke, audience: participants }
  - { fact: spoke, target: servant, remember: off }
  - { fact: conversation-started, biography: on }
regions:
  library:
    rules: [ { action: talk, effect: forbid } ]
    consequences: [ { fact: spoke, audience: participants } ]
```

**Actions** (`ACTIONS`):

| Action | Roles a rule may name | Regional | Enforced where |
| --- | --- | --- | --- |
| `talk` | `actor` (speaker), `target` (listener), `place` (speaker's place) | yes | `validate` after the listener is known to be a Person (active or not) and the speaker is located, before `talk_requirement_within(range).evaluate`; `offers` through the same call → `Offer::refused(PermissionDenied)` |

**Parameters** (`ConversationParameters`, `PARAMETER_ROLES = [actor, target, place]`):

| Field | Type, unit | Default | Bound (L0) | Was | Looked up |
| --- | --- | --- | --- | --- | --- |
| `gap` | `u32`, seconds | 300 | 1 ..= 86 400 | `CONVERSATION_GAP` (IL-b) | `resolve`, roles speaker/listener, speaker's place (unchanged) |
| `range` | `u32`, millimetres | 3 000 | 1 ..= 100 000 (QIE-4) | `INTERACTION_RANGE` | `validate` and `offers`, roles speaker/listener (offers: observer/target), the speaker's place |
| `remembered` | `u16`, entries | 32 | 1 ..= 64 (QIE-4) | `REMEMBERED_AT_MOST` | `react` on `spoke`, roles speaker/listener, the fact's place |

`INTERACTION_RANGE` and `REMEMBERED_AT_MOST` stay public as the defaults' values (as `CONVERSATION_GAP` did
in IL-b); the pinned test asserts each default equals its constant.

**Facts** (`FACTS`):

| Fact | Roles → envelope position | Owner default | Narrowest | Biography configurable | Compiled biographical |
| --- | --- | --- | --- | --- | --- |
| `spoke` | `actor` → `Participant(0)` (speaker), `target` → `Participant(1)` (listener), `place` → `Place` | `place` | `participants` | yes | no |
| `conversation-started` | `actor` → `Participant(0)`, `target` → `Participant(1)`, `place` → `Place` | `participants` | `participants` | yes | no |

**Knobs** (`ConversationKnobs`, the partial twin of a `parameters!` block):

| Knob | Type | Default (absent) | Meaning |
| --- | --- | --- | --- |
| `remember` | `bool` (`on`/`off`) | on | Whether the **listener's** `ConversationHistory` keeps this line. Read in `react` with the fact's roles and place. `off` writes nothing; the fact is still recorded, perceived and reduced by relationships. |

Consequences at emission (`resolve`): `consequence::<Self>(read, Some(place), spoke, roles, Visibility::Place(place))`
→ the envelope's Visibility; likewise `conversation-started` with `Visibility::Participants`. Unconfigured,
each returns `owner_default` unchanged (`lookup.rs:219–225`), so both emissions are byte-identical.

**What a `remember: off` world means** (QIE-5). The listener does not keep the line; `continues_a_conversation`
asks both histories, so a reply from the listener keeps the conversation going (the speaker's history holds
it), but repeated lines in one direction to a non-remembering listener each start a new conversation. That
is the honest consequence of "this person keeps no record" and is documented in conversation's README.

### 4.2 `configure/group-activity.yaml` (owner: group-activity; VERSION 2 → 3)

```yaml
rules:
  - { action: invite, actor: noble, target: commoner, effect: forbid }
  - { action: accept-invitation, actor: noble, target: commoner, effect: forbid }
  - { action: join-group-activity, actor: noble, target: commoner, effect: forbid }
parameters:
  - { invitation_lifetime: 1800, invite_range: 3000, activity_length: 3600 }
  - { place: park, activity_length: 7200 }
consequences:
  - { fact: group-activity-started, audience: participants }
  - { fact: joined-group-activity, actor: servant, biography: off }
  - { fact: invited, biography: on }
```

**Actions** (QIE-2):

| Action | Roles | Regional | Where |
| --- | --- | --- | --- |
| `invite` | `actor` (inviter), `target` (invitee), `place` | yes | validate after the payload and target checks, before `invite_requirement_within(invite_range)`; offers |
| `accept-invitation` | `actor` (the one accepting), `target` (the inviter), `place` | yes | validate after the payload is readable, before the open-invitation and Busy checks; offers |
| `join-group-activity` | `actor` (joiner), `target` (the member joined), `place` | yes | validate after the payload is readable, before Busy and the requirement; offers |

`decline-invitation` and `leave-group-activity` are **not** declared: a rule naming them is refused as an
undeclared action. A list can stop people coming together; it cannot trap anyone in an invitation or an
activity.

**Parameters** (`GroupActivityParameters`, `PARAMETER_ROLES = [actor, target, place]`):

| Field | Type, unit | Default | Bound | Was | Looked up |
| --- | --- | --- | --- | --- | --- |
| `invitation_lifetime` | `u32`, s | 1 800 | 1 ..= 86 400 | IL-b | `react` on `invited` (unchanged) |
| `invite_range` | `u32`, mm | 3 000 | 1 ..= 100 000 | `INVITE_RANGE` | validate and offers for `invite`, roles inviter/invitee, the inviter's place |
| `activity_length` | `u32`, s | 3 600 | 60 ..= 86 400 | `ACTIVITY_LENGTH` | `begin` (resolve of an accept that starts an activity), roles `actor` = inviter, `target` = invitee, the place |

**Facts** (QIE-3):

| Fact | Roles → position | Default | Narrowest | Bio. conf. | Compiled bio. |
| --- | --- | --- | --- | --- | --- |
| `invited` | `actor` → `Participant(0)` (inviter), `target` → `Participant(1)` (invitee), `place` | `participants` | `participants` | yes | no |
| `invitation-accepted` | `actor` → `Participant(1)` (invitee, who answered), `target` → `Participant(0)` (inviter), `place` | `participants` | `participants` | yes | no |
| `invitation-declined` | as `invitation-accepted` | `participants` | `participants` | yes | no |
| `group-activity-started` | `place` only (members list, F-IE-8) | `place` | `participants` | yes | yes |
| `group-activity-ended` | `place` only | `place` | `participants` | yes | yes |
| `joined-group-activity` | `actor` → `Participant(0)` (the person), `place` | `place` | `participants` | yes | yes |
| `left-group-activity` | `actor` → `Participant(0)`, `place` | `place` | `participants` | yes | yes |

`Knobs = ()`. The three emission helpers in `event.rs` take the chosen `Visibility` instead of naming it, so
the owner's default lives in one `FactDecl` and one call site; `wake` (an ending at the hour) looks up at
the process's place.

### 4.3 `configure/relationships.yaml` (owner: relationships; new section; VERSION 1 → 2)

```yaml
rules:
  - { action: acquaint, actor: noble, target: commoner, effect: forbid }
parameters:
  - { spoke_familiarity: 10, accepted_regard: 50, declined_regard: -30,
      activity_familiarity: 50, activity_regard: 20 }
  - { actor: grump, declined_regard: -100 }
  - { place: cafe, spoke_familiarity: 20 }
consequences:
  - { fact: became-acquainted, actor: servant, biography: off }
```

**The reaction rule `acquaint`** (QIE-1):

| Rule name | Roles | Regional | Where |
| --- | --- | --- | --- |
| `acquaint` | `actor` (the person who would come to know), `target` (the counterpart) | **no** | `react`, per directed `Change`, before the entry is read: forbidden ⇒ no `Acquaintances` entry, no `knows` edge, no `became-acquainted`, no `relationship-changed` for that direction |

It is directed like `knows`: forbidding `noble → commoner` leaves `commoner → noble` to the list. It is not
regional and has no `place` role: whether two kinds of people form relationships at all is about the pair,
and a regional forbid would make an existing relationship's later contacts ambiguous (count here, not
there). Classes are fixed during play (ARC-64) and the section is fixed at genesis, so a forbidden pair has
never formed a relationship; there is no "existing" case.

**Parameters** (`RelationshipParameters`, `PARAMETER_ROLES = [actor, target, place]`; actor = the person whose
values change, target = the counterpart, place = the cause's place):

| Field | Type | Default | Bound | Was |
| --- | --- | --- | --- | --- |
| `spoke_familiarity` | `i32` | 10 | 0 ..= 1 000 | `SPOKE_FAMILIARITY` |
| `accepted_regard` | `i32` | 50 | −1 000 ..= 1 000 | `ACCEPTED_REGARD` |
| `declined_regard` | `i32` | −30 | −1 000 ..= 1 000 | `DECLINED_REGARD` |
| `activity_familiarity` | `i32` | 50 | 0 ..= 1 000 | `ACTIVITY_FAMILIARITY` |
| `activity_regard` | `i32` | 20 | −1 000 ..= 1 000 | `ACTIVITY_REGARD` |

Bounds are the component's own ranges (`FAMILIARITY` 0 … 1 000, `REGARD` −1 000 … 1 000, E-class): one
contact can move a value at most across its whole range, and familiarity never decreases through contact.
The five constants stay public as the defaults' values.

**Facts:**

| Fact | Roles → position | Default | Narrowest | Bio. conf. | Compiled bio. |
| --- | --- | --- | --- | --- | --- |
| `became-acquainted` | `actor` → `Subject(0)` (holder), `target` → `Participant(1)`, `place` | `participants` | `participants` | yes | yes |
| `relationship-changed` | as above | `participants` | `participants` | yes | yes |

`Knobs = ()`. The facts' audience is already the narrowest, so only biography is configurable.

### 4.4 What each section cannot do (IL-I3, restated per pack)

- conversation: forbid `decline`/`leave` (not its actions); narrow `spoke` below `participants` (no such
  audience exists); widen `conversation-started` to `place` (refused as widening); set `range: 0` or
  `remembered: 0` (bounds); write into a listener's history what was not said (a knob only withholds).
- group-activity: govern `decline-invitation` or `leave-group-activity`; widen the invitation facts.
- relationships: put `acquaint` in a region; govern another pack's fact (a consequence for `spoke` in
  relationships' section is an undeclared fact there); create a relationship the pack would not form
  (`permit` is a filter).

---

## 5. Design decisions (SD-IE-1 … SD-IE-12)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-IE-1** | The three sections of §4, each declared by its owner through IL-b's `InteractionSection`, with no SDK, authoring or loader change. | R-IL-6; IL-I4/IL-I10; single ownership. |
| **SD-IE-2** | `permits` placement: after the payload is readable, the actor and target exist and are Persons and the actor is located (the lookup needs the place); before any pack precondition that depends on state (open invitation, Busy) and before the spatial requirement. The offer path calls the same function with the observer's place and refuses with the returned `Rejection`. An observer with no location gets no refusal (its dispatch is refused `PreconditionFailed` first, as today). | QIL-13, `ARC-63` item 8: dispatch and offer answer alike, in the same order. |
| **SD-IE-3** | `talk_requirement_within(Millimetres)` and `invite_requirement_within(Millimetres)` are added; `talk_requirement()` and `invite_requirement()` remain the default requirement. validate and offers use the `_within` form with the looked-up range. | F-IE-3: no cognition diff; the requirement shown is the requirement enforced. |
| **SD-IE-4** | `ConversationHistory::remember_within(heard, at_most)`; `remember(heard)` is deleted (no shim, pre-stable, `CLAUDE.md` §4 rule 12) and its test uses `remember_within(.., REMEMBERED_AT_MOST)`. | F-IE-5. |
| **SD-IE-5** | Emissions take the consequence's `Visibility`: conversation's two, group-activity's seven (through its emission helpers), relationships' two. A fact's owner default is written once, in its `FactDecl`, and the emission passes `FactDecl.default_audience`-equivalent `Visibility` as `owner_default`. A unit test per pack asserts each emission's unconfigured Visibility equals the `FactDecl`'s default. | One source for the owner default; IL-I1. |
| **SD-IE-6** | The knob `remember` is read in `react` on `spoke` (where the history is written), through `consequence::<ConversationSystem>(read, event.place(), spoke, roles, ..).knobs`. The fact's Visibility is not re-derived there. | The knob governs conversation's own state (`ARC-65` item 3, "in-world"). |
| **SD-IE-7** | relationships: `#[derive(Debug, Clone, Default, PartialEq)]`; `impl InteractionSection` with `CONFIGURED = "relationships-interactions-configured"`, `COMPONENT = "relationships-interactions"`, codec `serde_json` through its `codec.rs`; `interactions!()` in `impl SystemPack`; `declaration()` through `interactions::declare`, `install` through `interactions::install`, `react` calls `interactions::reduce` first. | IL-b's pattern (D-IB-3, D-IB-7). |
| **SD-IE-8** | `acquaint` is declared as `ActionDecl { action: ActionTypeId::from_static("acquaint"), roles: [Actor, Target], regional: false }`. A test asserts no installed pack provides an action type `acquaint` (so the rule name cannot be confused with a dispatchable action in this build). A dated note on `ARC-63` says: "a pack may declare a rule name for a decision it makes in a reaction; it names no action type a client can send". | QIE-1; F-IE-12. The SDK keys rules by `ActionTypeId`; no new type is introduced. |
| **SD-IE-9** | Version bumps: conversation 2 → 3, group-activity 2 → 3, relationships 1 → 2. Each pinned test holds `(VERSION, default parameters, CONFIGURED, COMPONENT)`. | A configured fact's payload shape grows (new parameter fields are not `#[serde(default)]`), so an IL-b save with a configured section would not decode; `ARC-25` refuses it by version instead (QIL-18). |
| **SD-IE-10** | Byte identity is by construction: with no section configured every lookup returns the compiled default without reading state (`lookup.rs:171, :190, :219`), every default equals today's constant, every `_within` call passes today's range, every emission's `owner_default` is today's Visibility, and `remember` defaults to on. | IL-I1; §6 IE-1 measures it. |
| **SD-IE-11** | Documentation: each pack's README gains (or extends) "Its section" with §4's tables; conversation's README states §4.1's `remember: off` semantics; `MODULE_SPEC.md` §4.2 gains one sentence on reaction rules; `DECISIONS.md` gains dated notes on `ARC-63` (SD-IE-8) and `ARC-65` (first configurable facts in installed packs; QIB-11 and QIB-12 closed by IE-6/IE-7; F-IE-10's per-fact limitation stated). | `CLAUDE.md` §2.2: specs before code. |
| **SD-IE-12** | No world changes; no pack's compiled default changes; `systems/presence`, the SDK, `authoring`, `worldpack/src`, `tools/cli/src`, `cognition/` are not edited. | Scope; IL-I4. |

---

## 6. Acceptance (decided before measuring, `ARC-23`)

Rules (IL-a's and IL-b's): every guarded criterion names its mutation; a mutation is applied in the working
tree, observed to fail by name, then reverted; `git status` and `git grep MUTATION -- '*.rs'` are recorded
afterwards. Expected values are literals from the test's own layout or E-IE-0's captured values.

```text
IE-1  Byte identity (IL-I1). E-IE-0 captured on the base before any code. On the final head:
        social-cafe and market-town `mineworld run <w> --headless --seed 7 --days 300`: output sha (all
        lines but `wall`), fact count, fingerprint, faults 0 equal E-IE-0;
        bodies-yard 30 days seed 7 (it installs conversation): sha equals E-IE-0;
        the three worlds' `mineworld validate` output: cmp-identical.
      M-IE1a: conversation's compiled `range` 3000 → 2999 → the 30-day social-cafe sha differs (the range
      is read in validate/offers, and the instrument sees it).
      M-IE1b: relationships' compiled `spoke_familiarity` 10 → 9 → the 30-day social-cafe sha differs.
      M-IE1c: group-activity's compiled `activity_length` 3600 → 3599 → the 30-day social-cafe sha differs.
IE-2  Explicit default (IL-I2). A scratch social-cafe with `configure: [conversation, group-activity,
      relationships]`, each file `extends: default` only, against the unconfigured copy, 30 days seed 7
      --save: facts equal except exactly three genesis configuration facts; every later event id and every
      id a fact refers to offset by exactly 3; request outcomes equal.
      M-IE2: relationships' reference list `default` built with `declined_regard` 0 → the first
      relationship-changed after a decline differs and the test fails there.
IE-3  Validate order and offers, scripted (worldpack/tests/social_sections.rs, World::dispatch and
      presence's observe on a scratch social-cafe with classes { resident, commuter }):
      (a) `talk` resident → commuter forbidden: dispatch refused PermissionDenied **even when out of range**
          (permits before the requirement); the observer's affordance is unavailable with reason
          PermissionDenied and its requirement still shown; commuter → resident (no rule) accepted;
      (b) the same for `invite` and `join-group-activity`; and for `accept-invitation` with a section that
          forbids only `accept-invitation` resident → commuter: a commuter invites a resident (permitted),
          the resident's accept is refused PermissionDenied and its offer unavailable for that reason;
      (c) `decline-invitation` in a section is refused at load as undeclared, at its line and column;
      (d) `range: 6000` for `actor: resident`: a talk at 4 m is accepted for a resident and TooFarAway for a
          commuter; the offered requirement carries 6 000 for the resident and 3 000 otherwise;
      (e) a region `cafe` forbidding `talk`: refused in the café, accepted in the park.
      M-IE3a: offers skip `permits` (validate keeps it) → (a)'s affordance case fails.
      M-IE3b: validate skips `permits` (offers keep it) → (a)'s dispatch case fails.
      M-IE3c: validate uses the constant range, offers the looked-up one → (d) fails.
      M-IE3d: `accept-invitation` not checked → (b)'s accept case fails.
IE-4  Forbidden talk, through the binary (tools/cli/tests/social_interactions.rs; scratch social-cafe,
      30 days seed 7). classes { resident: person tagged resident; commuter: person tagged commuter },
      rules forbid talk resident ↔ commuter both ways. Fixed before measuring:
      (a) no `spoke` whose speaker and listener are one resident and one commuter;
      (b) `requests talk rejected PermissionDenied` is 0 (the controller follows offers, F-IE-18);
      (c) activity holds: every seat has ≥ 1 accepted `talk` in every 10-day bucket; faults 0;
      (d) the unconfigured twin has ≥ 1 such `spoke` (else the case is INCONCLUSIVE, not PASS).
      M-IE1 (step §6.2 (1)): offers skip `permits` for `talk` only → the controller attempts forbidden
      talks → (b) is above 0 and the test fails.
IE-5  Acquaint (relationships), through the binary: rule forbid acquaint actor resident → target
      commuter. Over 30 days: no `became-acquainted` or `relationship-changed` with a resident holder and a
      commuter counterpart; the reverse direction does occur; the twin has ≥ 1 in the forbidden direction
      (else INCONCLUSIVE). A scripted reduction (relationships' pack tests) shows no `knows` edge and no
      `Acquaintances` entry for the forbidden direction.
      M-IE5: relationships ignores the rule → (first clause) fails.
      `acquaint` in a region → refused at load, line and column.
IE-6  Biography through the binary (QIB-11 closed). consequences: `{ fact: spoke, biography: on }`,
      `{ fact: spoke, actor: resident, biography: off }`, `{ fact: became-acquainted, actor: commuter,
      biography: off }`. 30 days, then `mineworld biography --person <commuter seat>`:
      (a) contains `spoke` entries said by non-residents to or by that person, and none said by a resident
          (F-IE-10: also none heard from a resident);
      (b) contains no `became-acquainted` held by that person; a resident's biography still does;
      (c) the fact log holds every one of the excluded facts.
      M-IE6: conversation's `FactDecl` maps `actor` to `Participant(1)` (the listener) → (a) fails.
IE-7  Perception (QIB-12 closed). consequences `{ fact: spoke, audience: participants }`. 30 days --save,
      then `mineworld perceived --person <seat> --json` for every seat: no perceived `spoke` names a
      person other than the seat among its participants; the twin's export has ≥ 1 overheard `spoke`
      for some seat (else INCONCLUSIVE). The envelope's Visibility is Participants on every `spoke`.
      M-IE7: conversation states `spoke` with the owner default regardless of the section → fails.
      `audience: public` on `spoke`, `audience: place` on `conversation-started`, `audience: nobody` →
      each refused at load, line and column (the last as an unknown variant).
IE-8  The `remember` knob, scripted: `{ fact: spoke, target: commuter, remember: off }`: after a resident
      talks to a commuter, the commuter's ConversationHistory is unchanged and a `spoke` fact exists; a
      commuter → resident line is remembered by the resident; two lines resident → commuter 60 s apart
      give two `conversation-started` (QIE-5 stated). `remembered: 2` for target class keeps exactly the
      last two.
      M-IE8: `react` ignores the knob → fails.
IE-9  Group-activity parameters and consequences, scripted: `activity_length: 600` in region `park` ends a
      park activity at +600 s and a café activity at +3 600 s; `{ fact: group-activity-started, audience:
      participants }` gives Visibility::Participants; `{ fact: joined-group-activity, actor: X,
      biography: off }` excludes it from `biography::selected` for X and keeps it for others.
      M-IE9: `begin` uses ACTIVITY_LENGTH → the park case fails.
IE-10 Pins and tools: each pack's pinned test (VERSION 3/3/2, defaults, names); `mineworld interactions`
      on social-cafe prints "default (compiled)" for relationships too; on the IE-4 copy it prints the
      resolved `talk` rules and each person's class; `inspect` shows "conversation v3, group-activity v3,
      relationships v2".
IE-11 Cost (R-IL-5), social-cafe 30 days seed 7, dev profile, sequential on one idle machine, median of 3
      walls: head unconfigured ≤ base × 1.05; the IE-6 configuration (three sections, two classes, a
      region) ≤ head unconfigured × 1.05. Base walls spreading > 5 % → INCONCLUSIVE: re-run once, then
      report rather than retry.
IE-12 Unchanged guards: no diff under kernel/, contracts/, persistence/, server/, clients/, worlds/,
      sdk/, authoring/, worldpack/src/, tools/cli/src/, cognition/, systems/presence/, any other pack;
      root Cargo.toml and Cargo.lock unchanged; ac1_composability, precursor_vocabulary, seam_vocabulary,
      configuration_vocabulary unedited and passing (git diff --stat recorded).
IE-13 Full gate on the final head: cargo fmt --check, check, clippy -D warnings, test (workspace), both doc
      scripts, check_scratch.py; CI green on the exact PR head (macOS, Linux, Windows jobs as CI defines).
```

---

## 7. Test data fixed now

- Classes (scratch only, never in `worlds/`): `{ class: resident, of: person, tag: resident }`,
  `{ class: commuter, of: person, tag: commuter }`. Residents with seats: carol, dev, erin; commuters: grace,
  hana (F-IE-17). For IE-3(b)'s place-class case, `{ class: open, of: place, tag: public }`.
- Scratch copies are made at test time by IL-b's helper pattern (`interaction_runs.rs::configured`); nothing
  is written under `worlds/`.
- Runs: 30 days, seed 7, `--save` where a save is read. Buckets: 10 days.

---

## 8. Test ownership

| Test | Owner | Edited by IL-e |
| --- | --- | --- |
| `systems/conversation/src/interactions.rs` pinned test | conversation | yes: VERSION 3, new defaults |
| `systems/conversation/tests/conversation_and_presence.rs` (`REMEMBERED_AT_MOST` at :742–752; component list by owner) | conversation | only if `remember` → `remember_within` touches it (it calls the reducer through dispatch; expected unedited) — recorded either way |
| `systems/group-activity/src/interactions.rs` pinned test; `tests/group_activity.rs` | group-activity | pinned test yes; `group_activity.rs` gains IE-9 cases |
| `systems/relationships/tests/relationships.rs` | relationships | gains IE-5's scripted half; existing asserts on the constants unchanged (defaults unchanged) |
| `tools/cli/tests/inspect.rs:36–37`, `social_composition.rs:379–380, :419–420` | cli | **yes, claim changed on purpose**: versions (D-IB-12 precedent) |
| `tools/cli/tests/interactions.rs:205–208` | cli (IL-b) | **yes**: "(it declares: none)" → "(it declares: talk)"; the claim (an undeclared action is refused) is kept |
| `tools/cli/tests/interactions.rs:72–89` | cli (IL-b) | no: "every section default (compiled)" holds with relationships included — verified, edited only if it lists sections by name |
| `tools/cli/tests/{interaction_runs,biography,perceived,run,restart}.rs`, `worldpack/tests/interaction_sections.rs` | IL-b / S11-C / cli | **no** |
| `cognition/rule-controller/src/*` | rule-controller | **no** (SD-IE-3) |
| `tests/acceptance/tests/{ac1_composability,precursor_vocabulary,seam_vocabulary,configuration_vocabulary,complete_affordances}.rs` | acceptance | **no** |
| new: `worldpack/tests/social_sections.rs` (IE-3, IE-8 scripted, IE-9), `tools/cli/tests/social_interactions.rs` (IE-2, IE-4 … IE-7) | IL-e | new |

Layers: static (fmt, clippy); unit (each pack's pinned test, the emission-visibility tests, relationships'
reduction); integration through `World::dispatch` and the real loader (worldpack); real lifecycle through the
binary (IE-1, IE-2, IE-4 … IE-7, IE-11). Real-LLM Gate: **N/A** — no language model is involved
(`CLAUDE.md` §5). CI: IE-13.

---

## 9. Shared files with parallel lanes

| File | IL-e | Other lane | Rule |
| --- | --- | --- | --- |
| `tools/cli/tests/inspect.rs:36–37`, `social_composition.rs:379–380, :419–420` | conversation v3, group-activity v3, relationships v2 | IL-g: movement, schedule (and employment in market-town tests) versions on the same lines | whichever merges second merges the line by hand; both versions kept |
| `docs/DECISIONS.md` `ARC-63`/`ARC-65` | dated notes | IL-f, IL-g: dated notes | appended blocks, keep all |
| `docs/MVP_STATUS.md` S17 row | its cell | IL-f/IL-g cells | per-lane wording, keep all |
| towns' digests | must be unchanged | 12d re-baselines; 12n-2 may move the paced controller | if main moves the references during implementation, merge main and re-capture E-IE-0 from main (IL-a E-IA-13 precedent); IE-1 compares with main at IL-e's merge |

---

## 10. Commit plan

Each commit lists implementation, validation and review separately; each `[x]` needs evidence in §14.

### IE-C0 — Design (this document), docs only

**Goal.** A reviewable design before code (`CLAUDE.md` §2.2). **Scope.** This file. **Boundary.** Markdown only.
- [x] Implementation: written from §3's audit (planning session, 2026-10-10).
- [x] Validation: `python3 scripts/check_doc_headings.py` → "193 numbered sections across 26 documents, none
  duplicated", exit 0; `python3 scripts/check_decision_ids.py` → "104 decision ids, all distinct", exit 0
  (E-IE-d).
- [x] Review (self, planning session): every finding cites a file and line; every guarded criterion names a
  mutation; no decision number is taken. The freeze is the primary session's.

### IE-C1 — Specs before code

**Goal.** SD-IE-11's documentation, so the code that follows has a spec to match.
**Scope.** `docs/DECISIONS.md` (dated notes on `ARC-63`, `ARC-65`); `docs/MODULE_SPEC.md` §4.2 (one sentence:
reaction rules); `systems/{conversation,group-activity,relationships}/README.md` ("Its section", §4's
tables); `docs/MVP_STATUS.md` (S17 row: IL-e in progress); `handoff-il-e.md` created.
**Non-goals.** No code. No new decision number (QIE-9, ruled).
- [x] Implementation: the notes and README sections, worded from §4/§5 (E-IE-1).
- [x] Validation: both doc scripts; every term used is `MODULE_SPEC.md` §4.2's vocabulary (no synonym)
  (E-IE-1).
- [x] Review: the README tables equal §4's (field, bound, default); `acquaint` is never called an action
  (E-IE-1).
**Failure cases.** A doc script failure is fixed in the commit; a term conflict with `CORE_CONCEPTS.md` is a
stop (terminology law).

### IE-C2 — conversation: `talk` rule, `range`, `remembered`, consequences, `remember`

**Goal.** §4.1, SD-IE-2 … SD-IE-6, SD-IE-9 for conversation. One commit: the section, its enforcement and its
version move together.
**Scope.** `systems/conversation/src/{interactions,action,component,system,lib}.rs`; its pinned test;
`tools/cli/tests/{inspect,social_composition,interactions}.rs` version and "(it declares: …)" pins.
**Dependencies.** IE-C1.
- [ ] Implementation: `ACTIONS` (`talk`), `FACTS` (`spoke`, `conversation-started`), `ConversationParameters`
  gains `range`, `remembered`; `ConversationKnobs` (`remember`); `talk_requirement_within`; validate calls
  `permits` then evaluates with the looked-up range; offers likewise with `Offer::refused`; `resolve` routes both
  emissions through `consequence`; `react` reads `remember` and `remembered`; `remember_within`; VERSION 3.
- [ ] Validation: conversation's tests; the emission-visibility unit test (unconfigured = `FactDecl` default);
  the pinned test; the three CLI pins; `cargo clippy -p mineworld-conversation -D warnings`.
- [ ] Review: no other use of `INTERACTION_RANGE` / `REMEMBERED_AT_MOST` remains except as defaults' values and
  docs (`git grep`); validate and offers call one `permits` with the same roles and place; no cognition diff.
**Acceptance.** Unconfigured, every emission and requirement is byte-equal to today's (unit-level); a forbidden
pair is refused with the reason in both paths (fully proven in IE-C5).
**Failure cases.** An observer with no location: no refusal, default range (SD-IE-2). A listener not a Person:
`NoSupportedInteraction` before `permits`, as today.

### IE-C3 — group-activity: `invite` / `accept-invitation` / `join-group-activity`, `invite_range`, `activity_length`, consequences

**Goal.** §4.2 for group-activity. **Scope.** `systems/group-activity/src/{interactions,action,event,
perception,process,system,lib}.rs`; pinned test; the version pins in the three CLI tests.
**Dependencies.** IE-C1 (independent of IE-C2).
- [ ] Implementation: `ACTIONS` (three), `FACTS` (seven), parameters `invite_range`, `activity_length`;
  `invite_requirement_within`; `permits` in validate per SD-IE-2 and in offers; emission helpers take a
  `Visibility`; `begin` reads `activity_length`; `wake`/`depart` look up the ending's consequence at the
  process place; VERSION 3.
- [ ] Validation: group-activity's tests (`group_activity.rs:100` unchanged); the emission-visibility unit
  test; IE-9's scripted cases; rule-controller tests unedited and passing.
- [ ] Review: `decline`/`leave` remain ungoverned; the accept path cannot bypass a `join` forbid when both
  rules are written (scripted case); no `ACTIVITY_LENGTH` use remains but the default's value.
**Failure cases.** `activity_length` lookup when the accept's place is unknown: impossible (validate requires
`here`); `begin` errs as today (`ActionNotResolvedBySystem`).

### IE-C4 — relationships: the section, `acquaint`, five parameters, consequences

**Goal.** §4.3, SD-IE-7, SD-IE-8. **Scope.** `systems/relationships/src/{interactions.rs NEW, system,lib}.rs`;
`tests/relationships.rs`; README already in IE-C1; version pins.
**Dependencies.** IE-C1.
- [ ] Implementation: derives; `InteractionSection`; `interactions!()`; `declare`/`install`/`reduce`;
  `changes` takes the looked-up parameters per directed change; `apply` checks `permits(acquaint)` first;
  `fact` takes the consequence's Visibility; VERSION 2.
- [ ] Validation: relationships' tests (defaults unchanged); a scripted forbid (no entry, no edge, no fact,
  reverse direction formed); a unit test that every subscribed cause type is stated with a place (F-IE-13);
  a test that no installed pack provides `acquaint` (SD-IE-8).
- [ ] Review: the parameters are looked up with actor = holder, target = counterpart, for each direction; a
  consequence entry naming `spoke` in this section is refused (another pack's fact).

### IE-C5 — Scripted proofs through the loader

**Scope.** NEW `worldpack/tests/social_sections.rs`: IE-3 (a)–(e), IE-8, IE-9's scripted halves, with
M-IE3a–d, M-IE8, M-IE9.
- [ ] Implementation: scratch copies (§7), scripted dispatch and observe.
- [ ] Validation: each case passes; each mutation observed failing by name and reverted (`git status`,
  `git grep MUTATION`).
- [ ] Review: each test reads offers **and** dispatch; expected values are literals from the layout.

### IE-C6 — Through the binary

**Scope.** NEW `tools/cli/tests/social_interactions.rs`: IE-2, IE-4, IE-5, IE-6, IE-7 with M-IE1, M-IE2,
M-IE5, M-IE6, M-IE7; IE-10's interactions/inspect cases.
- [ ] Implementation: scratch copies; one unconfigured twin shared by the cases.
- [ ] Validation: each criterion's literal; INCONCLUSIVE rules applied as written; mutations observed.
- [ ] Review: no assertion depends on a wall clock or a platform path.

### IE-C7 — Close: byte identity, cost, guards, gate, ledger

**Scope.** IE-1 (M-IE1a–c), IE-11, IE-12, IE-13; `MVP_STATUS.md`; the ledger (§14) and handoff.
- [ ] Implementation: as above.
- [ ] Validation: E-IE-0 vs head; cost medians; guard diffs; full gate; CI on the exact head.
- [ ] Review: every changed path is in §11's change set or recorded as a deviation (§15).

---

## 11. Change set and paths with no diff

```text
docs/DECISIONS.md                       dated notes on ARC-63 (reaction rules, SD-IE-8) and ARC-65
docs/MODULE_SPEC.md                     §4.2: one sentence (reaction rules)
docs/MVP_STATUS.md                      the S17 row
systems/conversation/src/{interactions,action,component,system,lib}.rs, README.md, tests/ (if needed)
systems/group-activity/src/{interactions,action,event,perception,process,system,lib}.rs, README.md, tests/
systems/relationships/src/{interactions.rs NEW,system,lib}.rs, README.md, tests/relationships.rs
tools/cli/tests/{inspect,social_composition,interactions}.rs   pins (F-IE-16)
tools/cli/tests/social_interactions.rs  NEW
worldpack/tests/social_sections.rs      NEW
.structured-coding/plans/mvp0/{pr-il-e-social,handoff-il-e}.md
```

No diff: `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `worlds/**`, `sdk/`, `authoring/`,
`worldpack/src/`, `tools/cli/src/`, `cognition/`, `systems/presence/` and every other pack; the root
`Cargo.toml` and `Cargo.lock`; `tests/acceptance/**`.

---

## 12. Execution contract for PR IL-e (filled at the freeze, 2026-10-10)

```text
PROJECT / PR        MVP-0 · S17 / PR IL-e — social sections of the World's Interaction List
FREEZE              DESIGN FROZEN 2026-10-10 (primary session); operator accepted QIE-1 … QIE-12
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-il-e-social.md; evidence §14 (E-IE<n>); deviations §15
RELATED / BINDING   step-18-interaction-list.md §§2.2, 4, 5, 6.2, 12 (IL-b as merged), 12.15; overall.md
                    "The World Interaction List"; step-12-server.md QS11C-6; DECISIONS ARC-5, ARC-23,
                    ARC-25, ARC-29, ARC-34, ARC-43, ARC-61, ARC-63, ARC-64, ARC-65, DEP-28; MODULE_SPEC §4.2;
                    CLAUDE.md §§2–4
IMPLEMENTATION BASE main at start (exact commit recorded in IE-C1's evidence; re-audit §3); branch
                    mvp0/pr-il-e from main; worktree /Users/yuema137/mineworld-worktrees/impl-il-e; one
                    worktree, one session, a fresh one
COMMANDS            cargo (fmt, check, clippy -D warnings, test), the built `mineworld` binary, git, gh (never
                    merge), python3 scripts/*, mkdir -p, sed -n; no python3 -c, sed -i, awk, xargs, curl or
                    heredoc writes; files edited with the editor tools; no other worktree; no .claude/settings*
APPROVED SCOPE      §1, §4, §5 as answered by §13; IE-C1 … IE-C7
FROZEN INVARIANTS   IL-I1 … IL-I11; §11's no-diff list; IE-1 equal to E-IE-0 (re-captured from main if main
                    moves); every compiled default equals today's constant; a list cannot grant
SEQUENCE            E-IE-0 before IE-C2; IE-C1 → IE-C7 (IE-C2/C3/C4 in any order), each committed and pushed
                    when coherent
VALIDATION BUDGET   FOUR 300-day town runs: E-IE-0 social-cafe ×1, market-town ×1 on the base; IE-1 ×1 each
                    on the head. 30-day runs uncounted. bodies-yard 30-day twice. One full workspace gate on
                    the final head. Real-model NOT REQUIRED
LIVE DOCUMENTATION  §10 checkboxes; §14 ledger; §15 deviations; handoff-il-e.md
ENDPOINT AUTHORITY  implementation + local validation: at the freeze message; semantic commits, branch push:
                    authorized; PR creation/update: authorized, marked READY FOR OPERATOR REVIEW; merge:
                    operator only, merge commit
NORMAL STOP         PR IL-e READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any edit in §11's no-diff list (above all sdk/, authoring/, worldpack/src, kernel,
                    contracts, persistence, worlds); any digest change with default content not explained by
                    main moving; the schema needing a change for a social pack (R-IB-1 becoming fact); an
                    answer to QIE-1 … QIE-12 other than the freeze's; any of R-IE-1 … R-IE-8 becoming fact
                    beyond its mitigation
```

---

## 13. Questions (QIE-1 …)

### Rulings (2026-10-10)

- **The operator accepted every recommendation below, QIE-1 … QIE-12**, including the operator-material
  QIE-6 (biography per fact, not per reader, for MVP-0) and QIE-12 (`spoke` stays non-biographical by
  default; IL-h's manor list carries both entries). Relayed by the primary session on 2026-10-10.
- **QIE-9:** dated notes on `ARC-63` and `ARC-65` only; no new ARC number.
- **QIE-12's consequence for IL-h** is recorded in `step-18-interaction-list.md` §12.15's IL-h row.

**[OM]** marks operator-material questions. Each has a recommendation, and each recommendation is now the
ruling.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QIE-1** | relationships decides "does this pair form a relationship" in a reaction, but the SDK keys rules by `ActionTypeId`. Options: (a) declare a rule name `acquaint` (no pack provides it), non-regional, roles actor/target, with a dated note on `ARC-63`; (b) a boolean parameter `forms: on/off` scoped by actor/target (ambiguity refused, no forbid-wins); (c) defer `acquaint`. | **(a).** It is the step's frozen shape (§6.2), keeps Cedar's forbid-overrides for the one yes/no question, and adds no SDK type. SD-IE-8's test keeps the name from colliding with a real action in this build. |
| **QIE-2** | group-activity governs `invite`, `accept-invitation` and `join-group-activity`; `decline` and `leave` are never governed. | **Yes.** `accept` joins the inviter's activity (F-IE-7), so a `join`-only rule would leak; leaving and declining must stay possible. |
| **QIE-3** | All seven group-activity facts are declared; `started`/`ended` by `place` only; the invitation facts' biography becomes configurable (compiled off). | **Yes.** |
| **QIE-4** | Bounds: `range`, `invite_range` 1 … 100 000 mm; `remembered` 1 … 64 (the component's "a projection, not a memory" argument caps it); `activity_length` 60 … 86 400 s; familiarity increments 0 … 1 000, regard −1 000 … 1 000. | **Yes.** |
| **QIE-5** | `remember: off` means the listener's history keeps nothing, so one-directional lines to that listener each start a new conversation (F-IE-5). Alternative: compute conversation continuity from a separate record — a new component, out of scope. | **Accept and document.** |
| **QIE-6 [OM]** | Biography is chosen per fact, not per reader (F-IE-10): "servants' lines are not history" removes them from the listener's biography too. Per-reader selection would change the SDK and `ARC-65`. | **Accept for MVP-0**, stated in `ARC-65`'s note and conversation's README. |
| **QIE-7** | conversation 2 → 3, group-activity 2 → 3, relationships 1 → 2; old saves refused by name; no digest moves (QIL-18). | **Accept.** |
| **QIE-8** | `talk_requirement()` and `invite_requirement()` stay as the default requirements, beside `_within(range)`, so no cognition test changes. | **Yes.** It is the requirement's default, not a compatibility shim. |
| **QIE-9** | No new decision number: dated notes on `ARC-63` and `ARC-65`. If the primary session wants its own record for reaction rules, it assigns ARC-81. | **Notes only.** |
| **QIE-10** | The bystander proof (QIB-12) and the binary biography proof (QIB-11) close in this PR (IE-6, IE-7). | **Yes.** |
| **QIE-11** | `spoke` keeps `Place` as its compiled audience (QS11C-6); the list may only narrow it to `participants`; no hearing range here. | **Yes.** |
| **QIE-12 [OM]** | No line is biographical by default in MVP-0 (F-IE-9). IL-h's manor must write `{ fact: spoke, biography: on }` and then `{ fact: spoke, actor: servant, biography: off }`; flipping the compiled default instead would move every town's biography (not its digest) and is not proposed. | **Keep the compiled default off;** IL-h's list carries both entries. Recorded for IL-h's design. |

---

## 16. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-IE-1 | A forbid starves the paced controller (a seat with nobody to talk to). | IE-4(c)'s activity criterion, fixed now; the remedy is content, never the controller (ARC-34). |
| R-IE-2 | A conversion changes an unconfigured result (an emission's Visibility, a requirement's range, a reduction order). | SD-IE-10; per-pack emission-visibility unit tests; IE-1 with three mutations proving the instrument sees each new read. |
| R-IE-3 | The schema needs reshaping for a social pack (R-IB-1). | Material stop; §3 audited every seam used (F-IE-12 … F-IE-14) and found none needed. |
| R-IE-4 | Version pins collide with IL-g on the same CLI test lines. | §9's rule. |
| R-IE-5 | 12n-2 or 12d moves the towns' references during implementation. | Re-capture from main (§9); IE-1 compares at merge. |
| R-IE-6 | Cost: `permits` on every offer of three packs for every target. | Unconfigured lookups return before reading state; IE-11 bounds both cases. |
| R-IE-7 | `remember: off` is mistaken for memory control. | `ARC-65` item 3's layers; conversation's README says it governs only the in-world history; IE-7 proves memory follows audience. |
| R-IE-8 | `acquaint` is read as a dispatchable action. | SD-IE-8's note and test; READMEs call it a rule name decided in a reaction. |

---

## 14. Evidence ledger (E-IE)

```text
E-IE-d  2026-10-10, design commit on docs/il-e-design from origin/main @ bb62edf: check_doc_headings →
        193 sections / 26 documents, none duplicated (exit 0); check_decision_ids → 104 ids, all distinct
        (exit 0).
E-IE-0  2026-10-10, implementation session, on origin/main @ c9832d3 (the freeze merge #147; §3's paths
        unchanged since bb62edf except plan documents — re-audited before IE-C2), dev profile, binary kept
        as /tmp/impl-il-e-base/base-mineworld; "sha" = sha-256 of every output line but `wall`
        (`grep -v '^wall' | shasum -a 256`); machine shared (load average ≈ 100):
        social-cafe `run --headless --seed 7 --days 300`: exit 0, faults 0, 365 330 facts, fingerprint
          59339a9c281829c9, sha ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
          (= F-IE-19's recorded value)
        market-town, same: exit 0, faults 0, 375 619 facts, fingerprint 27693f9e0c72bc9f, sha
          d5db8988bb9d8c33ec8e1cf1ba906d58d1fbd49d2a4ad7bc2d2a69b0b0a922ee. This differs from F-IE-19's
          remembered 365b50e0…1d1d: main moved market-town's references between IL-b and this base (12d
          and later merges); IE-1 compares against this capture, as §6 IE-1 and §9 require.
        (town runs used: 2 of 4)
        bodies-yard `--days 30`: exit 0, faults 0, sha
          bd6a10026f608dba1bb4d48f1399ccaa26e353c7c570b4190ef99039975c80e6 (bodies-yard runs: 1 of 2)
        validate (sha-256 of the whole output): social-cafe ebcd60a0…f56a8, market-town
          6368595ab6cea52d5677fd77517d88d0d6ab3596a5f39286a05552bdca910318, bodies-yard 7356b8f8…2063f
        social-cafe 30 days seed 7 (M-IE1a–c's reference): sha
          06e2d63c6e7ee369fe3d13d59624ee0c691a0050dd8fe93fa1a4eed5d5016fbe, 37 085 facts, fingerprint
          2f65cd4b5a2b540e; talk accepted 6 705. PASS (references captured).
E-IE-1  2026-10-10, IE-C1: DECISIONS.md dated notes on ARC-63 (first rules of real packs; the reaction
        rule `acquaint`, SD-IE-8) and ARC-65 (first configurable facts of installed packs; QIB-11/12
        closed by IE-6/IE-7; per-fact biography, QIE-6; QIE-12); MODULE_SPEC.md §4.2 one paragraph
        "A rule decided in a reaction"; READMEs of conversation, group-activity, relationships: "Its
        section" with §4's tables (field, unit, default, bound equal §4.1–§4.3; facts' audiences and
        compiled biography equal §4's FactDecl tables); conversation's README states QIE-5's
        `remember: off` semantics and QIE-6; MVP_STATUS.md S17 row: IL-e in progress; handoff-il-e.md
        created. check_doc_headings → 193 numbered sections across 26 documents, none duplicated (exit
        0); check_decision_ids → 104 decision ids, all distinct (exit 0; no new id). Review: terms used
        are §4.2's (section, rule, parameter, consequence, region, class, role); `acquaint` is called a
        "rule name" decided "in a reaction" everywhere and "not an action anybody sends". PASS.
```

## 15. Deviations

None yet.

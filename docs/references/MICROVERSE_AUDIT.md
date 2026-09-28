# Reference audit — Microverse (KsanaDock)

**Status:** informational. **This document is not authoritative architecture.** It records what an
existing project implemented, what is worth borrowing, and what must not be imported. It changes no
MineWorld contract, and nothing in it overrides
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md), [`ARCHITECTURE.md`](../ARCHITECTURE.md),
[`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md),
[`ENGINEERING_STANDARDS.md`](../ENGINEERING_STANDARDS.md) or [`DECISIONS.md`](../DECISIONS.md). Where
this document and a specification disagree, the specification governs and this file is the defect.

**Audience:** coding agents. Vocabulary is [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md)'s throughout:
`Entity`, `Person`, `Place`, `Item`, `Organization`, `Relation`, `Process`, `ActionIntent`, `Event`,
`System`, `Controller`, `World`, `Component`, `Observation`, and the six pack types of
[`MODULE_SPEC.md`](../MODULE_SPEC.md). Microverse's own terms are quoted when they are Microverse's
(`AIAgent`, `RoomManager`, `APIConfig`) and never used as MineWorld terms.

| | |
| --- | --- |
| **Subject** | <https://github.com/KsanaDock/Microverse> |
| **Audited at** | commit `7a061d41ed67825496588e39ca37b0b2b7465fb2` |
| **History** | 20 commits, `2025-10-09` (`202506初版源代码` — the June 2025 initial source) to `2026-04-10`; later commits are community PRs adding providers, a macOS resolution fix and README work |
| **Self-description** | the open-source version of the initial June 2025 demo of *Microverse In Box*, a commercial title on Steam. Godot 4.3+, GDScript, JSON local storage |
| **Code licence** | MIT (`LICENSE`, `Copyright (c) 2025 KsanaDock`) |
| **Asset licence** | **not established.** 4,300+ art files credited to LimeZu (itch.io) in the README with no `LICENSES/`, no per-asset terms and no licence file anywhere under `asset/`. See §2.2 |
| **Size** | 30 GDScript files, 282 KB of script; one playable scene (`scene/maps/Office.tscn`, 17,052 lines) |
| **Audited by** | reading the source at that commit, plus two behavioural probes against Godot 4.7.2 recorded in Appendix A |

---

# 0. How to read this document

## 0.1 What Microverse is to us

An **implementation reference**. Never a foundation, never a target architecture, never a fork
candidate. [`REUSE_POLICY.md`](../REUSE_POLICY.md) §8 is the governing rule: *existing AI/agent
projects are references, not foundations by default*, and §7 is the operating instruction — reuse the
useful part without adopting the architecture around it.

The purpose of the audit, stated once:

> learn from concrete implementation experience, reuse what is genuinely reusable, and explicitly
> identify architecture that must not be imported.

## 0.2 The abstraction levels differ by a layer, and that difference is the point

```text
Microverse answers    how does an AI character live in this game?
MineWorld answers     what is a world that AI and humans can live in,
                      and how does someone else generate and assemble one?
```

Microverse is a **game** with AI characters in it. MineWorld is **infrastructure** for building
worlds ([`VISION.md`](../VISION.md) §2.1). Almost every disagreement in §4 reduces to that one
sentence, and almost none of them is a mistake on Microverse's part: a game that ships does not need
a kernel, a pack format, a headless mode, replay determinism, or a second renderer. It needs to run
on one machine and be fun.

So this audit judges Microverse as what it says it is — a June 2025 demo of a commercial game,
Godot + GDScript + JSON, published for people to learn from — and it judges it fairly. It shipped
working features that MineWorld has not yet written a line of: a nine-provider model abstraction,
per-character model binding, an LM conversation loop with turn-taking and graceful exit, categorised
memory with importance-weighted prompt selection, LM task generation with deterministic fallbacks at
every decision point, a god-mode authoring surface, click-to-move navigation, and a world-bible layer
of soft social rules. Several of those are ahead of us. §9 says so plainly.

What must not be imported is its **state model**, and the reason is not taste: it is that the
semantic state of the world lives in Godot node metadata, is written by five different owners
including the UI, is keyed by node name, is timestamped on the wall clock, and is reachable only from
inside a running engine. Every one of those is a specific MineWorld invariant, and §6 maps each one.

## 0.3 A large file is not a finding

[`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md) §17's thresholds are review triggers. This audit
does not report size, naming, comment language, `print` debugging, or dead code as findings in their
own right. It reports a coupling when a unit of code **knows two things it must not know at once** —
`CoffeeMachine` and HTTP, a chair's sprite depth and whether the chair is taken — and it names the
rule the coupling would violate if imported. Where a defect is cited, it is cited as *evidence for a
structural claim*, never as a scorecard.

## 0.4 Classification vocabulary

| Class | Meaning |
| --- | --- |
| **REUSE** | adopt substantially as-is. Requires that the licence permits it **and** that the architecture permits clean adoption. In this audit, applied to nothing from Microverse's code — see §2 |
| **ADAPT** | take the idea, the shape, or the data, and re-implement it inside a MineWorld abstraction |
| **REFERENCE ONLY** | worth knowing they did it and how it went; do not port |
| **REJECT** | must not enter MineWorld in any form, including as a shape to imitate |

Where MineWorld already has the stronger abstraction, conceptual adaptation is preferred over copied
code even when the licence would allow the copy ([`REUSE_POLICY.md`](../REUSE_POLICY.md) §§5, 10).

---

# 1. What Microverse is, described accurately

```text
Godot 4 project, GL Compatibility renderer, one scene: Office.tscn
├── 8 Persons          CharacterBody2D scenes, personality as prose in a GDScript const
├── 9 "rooms"          Area2D + CollisionShape2D + two exported strings, in the scene
├── 7 autoloads        SettingsManager DialogManager CharacterManager APIManager
│                      GameSaveManager SaveLoadUIManager MemoryManager
├── AIAgent.gd         one 2,213-line Node per character: perception, prompts, HTTP,
│                      decisions, tasks, movement, conversation, memory writes
├── APIConfig.gd       9 model providers behind one description
├── GodUI.gd           1,100 lines: implant memory, inflict disease, grant money, set
│                      emotion between two people, add/complete tasks, edit world rules
└── JSON in user://    settings, per-character AI settings, chat history, saves, rules
```

The runtime loop, as built:

```text
Timer (60 s, per character)
   → AIAgent.make_decision()
   → generate_scene_description()      reads RoomManager, the scene tree, wall-clock hour
   → get_character_status_info()       reads node metadata: money, mood, health, relations
   → MemoryManager.get_formatted_memories_for_prompt()
   → get_character_task_info()         and refreshes daily tasks as a side effect
   → one string prompt
   → APIManager.generate_dialog()      HTTPRequest node created per call
   → APIConfig.parse_response()        → String
   → match on "1" | "2"                → adjust tasks | continue current task
   → on any failure: _execute_default_decision()   deterministic fallback
```

Two properties of that loop deserve credit before any criticism. It **never blocks the game** — every
model call is an async `HTTPRequest` with a per-character in-flight guard
(`AIAgent.gd:264` `waiting_responses`), which is the policy [`ARCHITECTURE.md`](../ARCHITECTURE.md)
§9 requires. And it **degrades rather than stalls** — every LM decision point has a deterministic
default beside it. That second property is the runtime half of
[`VISION.md`](../VISION.md) §1.1, and Microverse has a concrete shape for it while MineWorld has only
the requirement.

---

# 2. Licence, and what may physically enter the repository

## 2.1 The code is MIT, which settles permission and not fit

`LICENSE` is a verbatim MIT licence, `Copyright (c) 2025 KsanaDock`. MineWorld is MIT
([`DECISIONS.md`](../DECISIONS.md) `DEP-8`), so MIT-licensed GDScript could be copied in with
attribution and an origin record ([`REUSE_POLICY.md`](../REUSE_POLICY.md) §10). **This audit
recommends copying none of it**, for architectural reasons rather than legal ones:

1. Every substantial script depends on Godot autoload singletons, absolute node paths, node
   metadata, or `CharacterBody2D`. Ported into MineWorld, the dependency travels with the code.
2. MineWorld's authoritative layers are Rust (`DEP-9`); GDScript belongs only inside a Presentation
   Pack, and nothing in Microverse's scripts is only presentation.
3. Where the *idea* is good, MineWorld already has a stronger abstraction to express it in (§4), and
   [`REUSE_POLICY.md`](../REUSE_POLICY.md) §5 asks for a thin adapter over a MineWorld interface
   rather than a transplanted API.

Consequence: **no row in §8's index is classified `REUSE`.** That is a finding, not an oversight. The
reusable things Microverse reveals are reusable *upstream libraries and engine features* (§10), not
Microverse code.

## 2.2 The art cannot be cleared, so it does not come near us — REJECT

`asset/` holds 4,305 files across `ui/` (3,621), `singleObjects/` (339), `objects/` (314),
`maps/` (15) and `characters/` (16). The README credits *"Art Assets: LimeZu"* and links
`limezu.itch.io`. There is no licence file under `asset/`, no per-asset terms, and no `LICENSES/`
directory anywhere in the repository. File names (`Modern_UI_Style_1_32x32.png`,
`interiors/`, `exteriors/`) correspond to LimeZu's commercial itch.io packs.

`DEP-8`'s test is **"free to redistribute"**, established from a primary source, because MineWorld
ships what it commits. Nothing in this repository establishes that for the art, and the repository's
own MIT licence covers KsanaDock's work, not a third party's. Therefore:

> **No file under `asset/` may be copied into MineWorld, referenced from a MineWorld pack, or used as
> local scaffolding that could later be committed.** Classification: **REJECT**.

This is a statement about *MineWorld's* clearance bar, not an allegation about Microverse's
compliance. `asset/fonts/fusion-pixel-12px-proportional-zh_hans.otf` is an open pixel font and would
clear on its own terms if we ever needed a CJK pixel face, but it must be sourced upstream with its
own licence recorded, not taken from here.

The two `.tscn` files are also off limits as content: they embed those assets by path.

---

# 3. MineWorld's position, so this audit can be read on its own

Established by reading the tree at the audit's base commit, not by trusting a status document.

**What exists and is green.** `mineworld-contracts` (the typed vocabulary: `EntityId`, `PersonId`,
`PlaceId`, `Location`, `LocalPosition`, `Orientation`, `Millimetres`, `WorldTime`, `SimDuration`,
`Component`, `ComponentRecord`, `Action`, `ActionRequest`, `ActionIntent`, `ActionResult`,
`Rejection`, `RejectionCode`, `Event`, `EventEnvelope`, `Causation`, `Visibility`, `Relation`,
`Observation`, `PerceivedEntity`, `PerceivedEvent`, `Affordance`, `SpatialRequirement`).
`mineworld-kernel` (`World`, `EntityRegistry`, `ComponentStore`, `RelationStore`, `SystemRegistry`,
`System` with `install`/`validate`/`resolve`/`react`, `WorldRead`, `WorldView<S>`, `Declarations<S>`,
`SystemIdentity`, `OwnedBy<S>`, `WriteToken<S>`, `WriteAccess`, `Emission`, `Deferral`, dispatch,
genesis). Two System Packs: `mineworld-presence` (`PresenceSystem`, `Presence`, `Arrive`, `Arrived`,
`present_in`, `observe()`, `InteractionProvider`, `Offer`) and `mineworld-conversation`
(`ConversationSystem`, `Talk`, `Spoke`, `ConversationStarted`, `ConversationHistory`, `Heard`,
`Utterance`). `mineworld-server` (`WorldHost`, `HostedWorld`, `SeatRoster`, `Perception`,
`PerceptionContext`, `ClientFrame`, `ServerFrame`, `WireObservation`, `CorrelationToken`, `Refusal`).
`mineworld-worldpack` (`WorldPack`, `WorldManifest`, `AuthoredPerson`, `AuthoredPlace`, `Capability`,
`LoadedWorld`). Two throwaway Godot spike clients that proved the contracts survive a renderer.

**What does not exist yet.** `cognition/` is empty — there is no `Controller`, no `LMController`, no
`RuleController`, no model provider, no prompt assembly, no memory beyond `ConversationHistory`, no
cognition budget. No persistence, no event log on disk. No world clock or scheduler (the server
thread advances a provisional stand-in from wall time). No movement (`Arrive` records an arrival; a
`MoveIntent` and a travel `Process` do not exist). No `Item` interaction, no `Organization` system, no
`Process`. No real 2D or 3D client: no camera controller, no character controller, no UI, no settings,
no save/load screen. No `worlds/social-cafe`.

**Therefore the honest comparison is not "our implementation versus theirs".** In roughly half the
areas below, MineWorld has a stronger *abstraction* and no implementation, and Microverse has a
working implementation and no abstraction. Both halves of that sentence matter when deciding what to
borrow.

---

# 4. Area-by-area audit

Each area answers the same six questions: **what it does**, **the problem it solves**, **worth
borrowing**, **must not be copied**, **whether MineWorld already has a stronger abstraction**, and
**what mature Godot feature or library it reveals**.

## 4.1 Model-provider abstraction

**What it does.** `script/ai/APIConfig.gd` (8.5 KB) holds a static `Dictionary` of nine
`APIProvider` values — Ollama, OpenAI, DeepSeek, Doubao, Gemini, Claude, SiliconFlow, KIMI,
OpenAICompatible — each carrying `url`, `models`, `requires_api_key`, `headers_template`,
`request_format` and `response_parser`. Four functions do all the work: `build_request_data`,
`build_headers` (substituting `{api_key}`), `get_url` (substituting `{model}`) and `parse_response`.
`APIManager.generate_dialog(prompt, character_name)` resolves the character's settings, creates a
fresh `HTTPRequest` node, fires it, and returns the node so the caller can connect a completion
callback.

**The problem it solves.** Adding a model vendor without touching any call site, and letting a
per-character setting select one at runtime. It is a genuinely good pattern and MineWorld will need
it.

**Worth borrowing.** Four things.

- **One table, four fields, and the code is data-driven.** URL template, header template, request
  shape, response shape. The right shape.
- **`request_format` and `response_parser` are separate fields.** Two providers can share a request
  shape and differ in the response. Cheap, and correct.
- **An empirical datum about the provider landscape.** Six of their nine entries declare
  `request_format: "openai"` *and* `response_parser: "openai"`; only Ollama, Gemini and Claude
  differ. The provider space has collapsed onto OpenAI-compatible, which means MineWorld's table is
  small: one OpenAI-compatible adapter plus native Ollama, Anthropic and Gemini.
  [`ARCHITECTURE.md`](../ARCHITECTURE.md) §9.2 already lists exactly the four first-class backends,
  and three of them (OpenAI-compatible endpoint, vLLM, llama.cpp server) are the *same* adapter.
- **Per-`Person` provider binding with fallback to a world default.**
  `SettingsManager.get_character_ai_settings(name)` returns the character's override or the default
  (`script/ui/SettingsManager.gd:105`). [`ARCHITECTURE.md`](../ARCHITECTURE.md) §9.1 tiers cognition
  by *decision class*; Microverse tiers it by *character*. Both are wanted, and their axis is the one
  we have not specified.

**Must not be copied.** `parse_response()` returns `String`. Every caller then re-derives structure
from prose — `AIAgent.gd:591` matches the whole decision on the literals `"1"` and `"2"`, and the
task, movement, farewell and room-choice handlers each re-parse their own formats. That single design
choice is the origin of most of `AIAgent.gd`'s size and all of its ~14 fallback functions. Also not to
be copied: credentials interpolated into a header template with no redaction anywhere near a `print`;
no timeout, retry, rate limit, token accounting or cost ceiling; no `system` role, no multi-turn
`messages`, no temperature, no `max_tokens` except a hardcoded 1024 for Claude; `Claude`'s header
template is `Authorization: Bearer {api_key}` where the Anthropic API expects `x-api-key` — a
per-provider correctness detail that a table with no capability or auth *type* cannot express.

**Does MineWorld already have a stronger abstraction? No, and this is a real gap.** `cognition/`
contains a `.gitkeep`. [`ARCHITECTURE.md`](../ARCHITECTURE.md) §9.2 names the interface
(provider-neutral, credentials belong to the operator, a world declares capabilities not secrets) and
[`MODULE_SPEC.md`](../MODULE_SPEC.md) §5 assigns `model routing` and `cognition budgets` to a
Controller Pack — but nothing implements either. Microverse is ahead of us here, and §5.1 is the
design opinion this audit owes.

**Godot feature or library revealed.** `HTTPRequest` per call is the only HTTP client GDScript has,
and it is a `Node`, which is why their client leaks request nodes and needs a 1-second deferred
cleanup timer (`APIManager.gd:58-66`). This is an argument for MineWorld's existing split, not against
Godot: model calls belong in Python `cognition/` behind a `ModelProvider`, and the Godot client should
never make one. Upstream candidates to evaluate rather than hand-write the table: the `openai` Python
SDK against any OpenAI-compatible endpoint, `ollama`'s own client, `anthropic`, `google-genai`, or a
router such as LiteLLM — a `REUSE_POLICY.md` §1 investigation with a `DECISIONS.md` record either way.

## 4.2 Human / AI control switching

**What it does.** `AIAgent` is a child `Node` of the character body, created in
`CharacterController._ready()` (`CharacterController.gd:39`). `CharacterController.set_selected(bool)`
calls `ai_agent.toggle_player_control(enabled)`, which stops the decision `Timer` and sets
`current_state = State.IDLE`; deselecting restarts the timer (`AIAgent.gd:54`). While selected, the
same `_physics_process` reads `Input.get_axis(...)` and keyboard movement pre-empts any pathfinding
target. `CharacterManager.select_character()` toggles selection, moves the camera, and mirrors the
selection into `GodUI`.

**The problem it solves.** Taking over any NPC at runtime with one click, and giving it back, without
reloading anything.

**Worth borrowing.** Two points, both real.

- **The Person outlives the decider.** Their `Person` is the `CharacterBody2D`; the decider is a
  detachable child node that can be muted. Personality, memory, tasks and relations are untouched by
  the switch. That is `INV-1`'s *intent* honoured in a demo, and `AC-5` (a human takes control of an
  existing NPC without destroying that Person's biography) is satisfied by construction.
- **Control state is saved.** `GameSaveManager` records `is_player_controlled` and re-applies it via
  `toggle_player_control` on load (`GameSaveManager.gd:154`, `:241-242`). Control binding is treated as
  world configuration, not as session UI state. Correct instinct.

**Must not be copied.** Selection is a renderer concept (`is_selected`, a click, a camera) that
directly gates authoritative behaviour, and `is_player_controlled` lives on the agent node inside the
scene. There is no notion of *which client* holds the Person, so the design cannot extend to two
humans; and because the human path writes `velocity` on the body directly, human control and AI
control produce different kinds of change to the world — one writes physics, the other calls
`move_to`. In MineWorld both must produce the same `ActionRequest` (`AC-13`).

**Does MineWorld already have a stronger abstraction? Partly — stronger in principle, thinner in
practice.** `INV-1` is stated, and `SeatRoster`/`Seated` in `server/src/host.rs` already binds a
*connection* to an observer `EntityId` once and refuses a second `Join`
(`server/src/session.rs:handshake`), which is a cleaner authority model than a click-to-select. But
MineWorld has no `Controller` type at all: there is no `HumanController`, no `LMController`, and no
rebinding operation, so `AC-5` is currently unexercised. Microverse has demonstrated the *user
experience* of runtime rebinding; we have the invariant and the seat, and the mechanism is unbuilt.

**Godot feature or library revealed.** Nothing specific. Their input handling is plain
`Input.get_axis` with actions declared in `project.godot` — which is the right thing, and the reason
`DEP-9` hands input to Godot.

## 4.3 Conversation lifecycle

**What it does.** `DialogService` (`script/ai/DialogService.gd`) holds a dictionary of active
`ConversationManager` instances keyed by a generated `conversation_id`
(`"<speaker>_<listener>_<unixtime>"`). `try_start_conversation(speaker, listener)` refuses if either
is already talking or if `global_position.distance_to(...)` exceeds `max_dialog_distance = 100`.
`ConversationManager` is a `RefCounted` per-conversation object that builds the speaker's prompt,
calls the model, instantiates a `DialogBubble`, appends the line to **both** participants'
`ChatHistory` nodes, then swaps `speaker` and `listener` and recurses
(`ConversationManager.gd:249`). Exit is decided elsewhere: `AIAgent.make_conversation_decision()`
asks the model whether to continue, and on "end" generates a farewell line and calls
`dialog_service.end_character_conversations`.

**The problem it solves.** Several simultaneous two-party conversations, each with its own identity,
prompt context and lifecycle, with a graceful spoken exit rather than an abrupt stop.

**Worth borrowing.** Three things, and the third is the best idea in this area.

- **A conversation is an addressable instance with a lifetime**, not a global flag. `DialogService`
  keeps `is_in_conversation` only as an explicitly-labelled compatibility leftover
  (`DialogManager.gd:10`), and the real state is per-conversation. This is a `Process`
  ([`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §10) discovered by necessity.
- **Signals for lifecycle, not polling**: `conversation_started`, `conversation_ended`,
  `dialog_generated`, and `DialogService` reacting to a child conversation ending. Clean.
- **The graceful exit is a modelled act.** Ending a conversation *generates a farewell utterance* and
  then ends the conversation (`AIAgent.gd:708-811`). A conversation that stops without anybody saying
  goodbye reads as a bug to a player. MineWorld has no equivalent concept and should want one.

**Must not be copied.** The recursion has no turn budget: each reply schedules the next with no
bound, so a conversation runs until the per-character decision timer happens to intervene, and cost
is unbounded. The participants are two typed fields, `speaker` and `listener`, so three people at a
table is not representable — the exact demo assumption [`VISION.md`](../VISION.md) §3.1 lists as
disqualifying for infrastructure. Range is a renderer-side pixel distance evaluated in the service
(`DialogService.gd:113`), and a second, different range constant (`DIALOG_DISTANCE = 100` in
`CharacterManager.gd:5`, `max_dialog_distance` in `DialogService`, `max_dialog_distance = 100` again
in `DialogManager.gd:170`) exists in three places. And `_on_request_completed` does presentation
(instantiates a bubble into the scene root), persistence (writes two JSON files), and cognition (fires
the next model call) in one function.

**Does MineWorld already have a stronger abstraction? Yes, in the parts it has built — and it is
missing the part they built.** `ConversationSystem` owns `talk` and evaluates
`talk_requirement()` — `SpatialRequirement::same_place().within(INTERACTION_RANGE)
.requiring_target_available()` — through `SpatialRequirement::evaluate`, the single evaluator every
System shares, so an `Affordance` and a dispatch of the same request cannot disagree. Range is one
named constant, `INTERACTION_RANGE = Millimetres::new(3_000)`, in the owning pack. Two distinct facts
are emitted, `ConversationStarted` and `Spoke`, rather than one fact with a `started: bool`, and the
`CONVERSATION_GAP` of 300 simulated seconds decides which. What MineWorld does **not** have is a
conversation as a `Process` with participants, a turn structure, a budget or an ending — `talk` is a
single instantaneous exchange. Their `ConversationManager` is the concrete shape of the thing our
`Process` concept is reserved for, and §9.3 records that.

**Godot feature or library revealed.** `DialogBubble` is a hand-built `NinePatchRect` + `Label`
positioned every frame in `_process`, added to the scene root, hidden by a `Tween` and never freed —
one node accumulates per utterance for the life of the process. The native answers are a
`Control` under a `Node2D` with `top_level` handling, or simply `Label.visible_characters` with
`Tween`; and for the follow behaviour, `RemoteTransform2D` or parenting to the character. Presentation
lifecycle is something a Presentation Pack must own explicitly.

## 4.4 Memory representation and retrieval

**What it does.** `script/ai/memory/MemoryManager.gd` (an autoload) stores memories as an `Array` of
`Dictionary` under node metadata `character_data.memories`. Each entry is
`{content, timestamp, type, importance, created_at}` where `type` is one of five enum values
(`PERSONAL`, `INTERACTION`, `TASK`, `EMOTION`, `EVENT`), `importance` is one of four levels
(`LOW=1`, `NORMAL=3`, `HIGH=5`, `CRITICAL=10`), `timestamp` is a formatted wall-clock string and
`created_at` a wall-clock unix float. `add_memory` appends then calls `_cleanup_old_memories`, which
sorts by importance then recency and truncates to 50. `get_formatted_memories_for_prompt(character,
max_count = -1)` sorts the same way and renders `"[timestamp] content"` lines into the prompt;
`-1` means **all of them**. `get_recent_memories(hours = 24)` filters by wall-clock age.
`search_memories(keywords)` does case-insensitive substring matching and is never called.

**The problem it solves.** Giving a model a bounded, ranked slice of what a character knows, so
prompts stay useful as a session gets long — and doing it with no vector database, no embeddings and
no server.

**Worth borrowing.** The *heuristics*, in detail, are the most directly valuable thing in the
repository, and §5.3 dissects which of them survive MineWorld's `event log → observation → subjective
memory → compression` chain. In summary: the importance ladder, the importance-then-recency selection
order, the recency window, and the idea that pruning is a policy the memory layer owns.

**Must not be copied.** The representation. `Alice`'s memories **are** the world's record of what
happened to Alice: there is no event log, so a pruned memory is destroyed and no biography can be
regenerated from anything. Timestamps are wall-clock, so the same run replayed produces different
memories. The default prompt selection is *everything*, bounded only by the 50-entry prune — the
`AC-10` failure in its purest form. The five categories are written but **never read**: no retrieval
path filters by `type` anywhere in the repository, so the taxonomy is declared and unused.

And memory has **four writers with two schemas**. `MemoryManager.add_memory` writes the five-field
entry; `DialogManager._add_memory_to_character` (`DialogManager.gd:260`) and
`_add_memory_to_current_character` (`:282`) write a two-field `{content, timestamp}` entry with
`timestamp` as a float; `GodUI` writes through `MemoryManager` in most places but does
`character.set_meta("memories", [])` at `GodUI.gd:486`. The consequences are visible in the code
rather than hypothetical: `MemoryManager._get_memory_timestamp` reads `created_at`, which the
`DialogManager` entries lack, so those memories sort as epoch-zero and are pruned first;
`_format_memory_for_display` prints their float timestamp verbatim into the prompt; and
`DialogManager.get_character_status_info` reads a *different* metadata key entirely
(`character.get_meta("memories", [])`, `DialogManager.gd:61`) which nothing writes, so that prompt
path reports "no important memories" unconditionally.

**Does MineWorld already have a stronger abstraction? Yes, decisively, and it is deliberately
smaller.** `INV-4` separates `Memory ≠ Biography ≠ World Truth`; `INV-11` makes the event log the
single source of history; [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §5.4 specifies hierarchical
compression `L0→L3` that always retains the underlying Event IDs. The one thing built is
`ConversationHistory`, and its documentation is explicit that it is a **projection**: `Heard`
carries exactly `{speaker: PersonId, at: WorldTime, utterance: Utterance}` and *no* importance, no
sentiment, no summary and no embedding, because each of those is an interpretation and interpretation
is a Controller's business; `REMEMBERED_AT_MOST = 32` bounds it; it is written in exactly one place,
`ConversationSystem::react` reducing this pack's own `Spoke` fact, so a replay rebuilds it exactly.
Microverse's memory is primary and mutable; ours is derived and reproducible. That difference is the
whole of §5.3.

**Godot feature or library revealed.** None. The absence is the point: they needed retrieval and got
substring search, because a GDScript client has no vector index.
[`REUSE_POLICY.md`](../REUSE_POLICY.md) §1 already lists `embedding/vector search` as commodity
infrastructure for Python `cognition/` when a memory policy needs it.

## 4.5 Task handling

**What it does.** Tasks are `{description, priority 1-10, created_at, completed}` dictionaries in node
metadata. There is a full LM-driven task lifecycle inside `AIAgent.gd`: `_check_and_initialize_tasks`
→ `_generate_initial_tasks` (model produces a list) → the 60-second decision asks the model to choose
between *adjust tasks* and *continue current task* → `_adjust_tasks` / `_continue_current_task`
(model chooses among move, converse, think, complete) → `_execute_task_movement`,
`_execute_task_conversation`, `_execute_task_thinking`, `_complete_task`. When a task names a target
that cannot be found, `_handle_target_not_found` asks the model whether to search, `_choose_room_to_search`
asks which room, and `_reschedule_task` asks for a replacement task. `_refresh_daily_tasks` tops the
list up to three every 24 wall-clock hours, drawing from a hardcoded `tasks_pool` that varies by job
title substring (`"经理"`, `"销售"`, `"技术"`, `"人力"`). Every LM branch has a `_default` twin.

**The problem it solves.** Giving each character a persistent intention that survives across
decisions, so behaviour looks purposeful instead of random — and keeping it purposeful when the model
is unreachable.

**Worth borrowing.** Three ideas, all good.

- **A deterministic twin for every LM decision.** `_execute_default_decision`,
  `_generate_default_tasks`, `_adjust_tasks_default`, `_continue_current_task_default`,
  `_rearrange_task_priorities_default`, `_add_urgent_task_default`, `_execute_task_movement_default`,
  `_execute_task_conversation_default`. This is the most valuable structural idea in the repository
  and §9.1 promotes it to a proposal.
- **A single scalar priority the model may re-rank**, described to the model as "渴望程度" (degree of
  desire) rather than as a queue position. It reads as motivation instead of scheduling, which is the
  right register for an LM.
- **Explicit handling of the stale-intent case.** `_handle_target_not_found` →
  `_choose_room_to_search` → `_reschedule_task`, plus `_start_movement_tracking` /
  `_check_movement_progress`, which polls at 0.5 s to notice arrival or failure. This is a hand-rolled
  answer to *the world changed while the model was deciding*, and it is direct empirical support for
  [`ARCHITECTURE.md`](../ARCHITECTURE.md) §9's mandatory revalidation step: without a revalidation
  seam, every controller grows its own recovery code. Five functions of theirs exist because one
  contract of ours does not yet.

**Must not be copied.** Tasks live in untyped dictionaries in node metadata and are written by
`AIAgent` **and** by `GodUI`, under two different keys. Within `GodUI.gd` alone, `_on_add_task`
writes `meta["tasks"]` (`:641`) while `_on_complete_task` reads and writes
`meta["character_data"]["tasks"]` (`:674-694`) — adding a task and completing a task in the same
panel operate on two different stores. `AIAgent` uses `meta["tasks"]` throughout;
`ConversationManager.get_character_tasks` reads `meta["character_data"]["tasks"]`
(`ConversationManager.gd:316`), so the conversation prompt and the decision prompt see different task
lists. `_refresh_daily_tasks`, `_check_daily_task_refresh` and `_generate_random_task` are duplicated
almost verbatim in `AIAgent.gd` and `GodUI.gd` — the same rule in two places, which is exactly the
change amplification [`ENGINEERING_STANDARDS.md`](../ENGINEERING_STANDARDS.md) §8 tests for. And
`get_character_task_info()`, a *read* used to build a prompt, refreshes daily tasks as a side effect
(`AIAgent.gd:361`): a mutation triggered by observing.

**Does MineWorld already have a stronger abstraction? Partly, and the unbuilt part is the one that
matters.** [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §10 specifies `Process` with participants,
`interruptibility` and an `owning_system` that alone decides how it ends — a far better home for "what
this Person is doing over time" than a dictionary. And a *task* in the MineWorld sense is not one
concept but two: an authoritative `Process` owned by a System, and a private intention owned by a
Controller Pack. Microverse conflates them, which is why the LM ends up scheduling movement. But
neither of ours is implemented: no `Process`, no scheduler, no Controller Pack. On implementation,
they are ahead; on where the concept belongs, we are.

**Godot feature or library revealed.** Their `Timer`-per-concern pattern (decision timer, movement
tracking timer, z-order timer, cleanup timer) is idiomatic Godot for a game. It is also the reason
their simulation rate is wall time, which MineWorld forbids for the authoritative layer
([`ARCHITECTURE.md`](../ARCHITECTURE.md) §5). Nothing to reuse; a boundary to keep.

## 4.6 Environment and room perception

**What it does.** `AIAgent.generate_scene_description()` builds one prose string: the current room's
name and authored description, a hardcoded environment paragraph selected by
`get_tree().current_scene.name`, a time-of-day sentence derived from the wall-clock hour, the items in
the room (`get_room_objects`), the people in the room with their job titles and states, and then a
**whole-map summary** — every room's name, centre coordinate, bounding box, distance, compass
direction, and who is in it.

**The problem it solves.** Turning a 2D scene into something a language model can reason about
spatially, including navigation options it has not visited.

**Worth borrowing.** Two ideas.

- **Perception includes reachable elsewhere, not just here.** The map summary is what lets a character
  decide to go to the tea room. An `Observation` that describes only the current room cannot support
  "walk out of the café and along the promenade" — which is precisely finding `F3` of our own renderer
  spike, and why `Observation` now carries `relations`. Microverse arrived at the same requirement
  from the other direction and solved it with prose.
- **Authored place descriptions are load-bearing.** `room_desc` strings such as *"茶水间是一个长廊，
  西边连通HR办公室、东边连通员工办公室…"* carry adjacency, contents and character in one authored
  sentence, and they are visibly the highest-value-per-byte content in the project. MineWorld's
  `AuthoredPlace` should expect an authored description as first-class World Pack content, not as
  flavour.

**Must not be copied.** Perception reads the scene tree directly:
`get_tree().get_nodes_in_group("interactable")` for objects,
`get_tree().get_nodes_in_group("character")` for people, `room_manager.rooms` for places. Group names
are strings with no checker, and in the published snapshot this silently breaks perception: no node
anywhere joins `interactable` or `character` (the only `add_to_group` calls are `chairs`,
`controllable_characters` and `api_manager`, and the scene declares `characters`), so
`get_room_objects()` and `get_room_characters()` return empty arrays and the "items in the room" and
"people in the room" sections never appear in any prompt. The 40-line `get_object_info` table that
knows about `CoffeeMachine`, `Printer`, `Whiteboard` and `FileCabinet` is, in the shipped build, dead
code reachable only through a group nobody joins. Also not to be copied: pixel distances presented to
the model as metres (`"距离约" + str(int(distance)) + "米"`, `AIAgent.gd:119` and `:259` — a distance
in Godot 2D pixels labelled 米/metres); `PERCEPTION_RADIUS = 200` declared at `AIAgent.gd:6` and never
used, because perception is actually room-AABB membership; and observation scoping implemented as
*what we chose to concatenate* rather than as a boundary.

One nuance deserves genuine credit, though: `ConversationManager.build_dialog_prompt` **deliberately
omits the listener's memories and detailed state**, with the comment *"these are the other party's
private information"* (`ConversationManager.gd:166`). The instinct behind `INV-13` is present. It is
enforced by prompt-assembly discipline, which is the difference between an instinct and an invariant.

**Does MineWorld already have a stronger abstraction? Yes, and it is the clearest win in the audit.**
`Observation` is a value listing what was exposed: no component store, no registry, no world handle,
no query method, so there is nothing in it to widen, and `Observation::entity(id)` searches the list
and nothing but the list. `mineworld_presence::observe()` states its judgements explicitly — the place
you are in, everybody `Presence` knows to be in it, the `present_in` edges for those entities, the
affordances, and *no events, because deciding which recorded facts a person learned of needs a
per-observer position in the log and honouring `Visibility` over time, which is a perception system's
job*. Entities come back as `PerceivedEntity` with only the components the observer is entitled to,
ordered by `EntityId` so two runs agree (`AC-12`). A misspelled group name cannot empty it, because
there are no group names: `read.components::<Presence>()` is a typed query the compiler checks.

**Godot feature or library revealed.** Their `Area2D` + `CollisionShape2D` room volumes are the right
*Godot* primitive for the client side of place binding, and `body_entered`/`body_exited` (used by
`Chair.gd`) is the right way for a client to notice that the player entered a volume — as a trigger to
send an `ActionRequest`, never as the fact itself. Worth recording for the 2D reference client.

## 4.7 Movement and navigation

**What it does.** `CharacterController` extends `CharacterBody2D`. `move_to(target)` builds a
`NavigationPathQueryParameters2D` with `PATH_POSTPROCESSING_CORRIDORFUNNEL`, calls
`NavigationServer2D.query_path`, and stores the resulting point list. `_physics_process` follows it
with a speed ramp near the end and a dynamic arrival threshold scaled by current speed, falling back
to a straight line when the query returns nothing. On top of that sits ~120 lines of hand-written
avoidance: a forward raycast, left/right clearance raycasts, a chosen avoidance direction held stable
for 0.8 s to stop oscillation, a blend factor by proximity, stuck detection over a 2 s window, and up
to three path recalculations toward a jittered offset target before giving up on a straight line.

**The problem it solves.** Eight characters walking around a furnished office without wedging
themselves on desks, driven by coarse "go to this point" commands from a model.

**Worth borrowing.** Two implementation details are genuinely hard-won: the **speed ramp plus
speed-scaled arrival threshold** (a fixed threshold either overshoots at speed or stalls at low
speed), and **stuck detection with bounded retries and a final straight-line fallback** — a navigation
consumer must have a give-up path or characters freeze forever. Both are lessons for a client, not for
the kernel.

**Must not be copied.** The avoidance layer itself should never have been written (see below). And
movement here *is* the world state: position is `global_position` on the body, nothing records that a
movement happened, nothing can replay it, and arrival is detected by polling distance from a separate
timer in `AIAgent._check_movement_progress`.

**Does MineWorld already have a stronger abstraction? In design yes, in code almost nothing.**
[`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md) §6 keeps `MoveIntent`, the travel `Process`,
authoritative spatial state and rendered movement as four distinct things; `Location` with an optional
`LocalPosition` in `i32` millimetres and `Orientation` in millidegrees spans both scales in one type,
fixed-point precisely because positions reach the event log. But the only thing implemented is
`Arrive`, whose own documentation says it *is not movement*: it records an arrival that something else
decided. So MineWorld has the better decomposition and no movement system, and Microverse has working
2D locomotion. Their code is the reference for the *client* half only.

**Godot feature or library revealed — the most concrete reuse finding in the audit.**

- `NavigationAgent2D` / `NavigationAgent3D` already provide path following, `target_desired_distance`,
  `path_desired_distance`, and **RVO avoidance** via `avoidance_enabled`, `radius`,
  `max_speed`, `neighbor_distance` and the `velocity_computed` signal. Microverse's entire hand-rolled
  raycast avoidance layer, its oscillation damping and its stuck detection are what one has to write
  after choosing raw `NavigationServer2D.query_path` instead of the agent node. That is a
  [`REUSE_POLICY.md`](../REUSE_POLICY.md) §4 lesson with a measurable size: ~120 lines and a class of
  bugs, avoided by using the mature component. Record it before the 2D client is written.
- For the 3D client, `DEP-8` already selected *Quality Godot First Person Controller v2* (MIT); this
  audit adds no reason to revisit that.
- `NavigationRegion2D`/`3D` baked from the scene is where a client's walkable surface belongs — a
  Presentation Pack concern that the simulation must never learn about
  ([`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md) §12).

## 4.8 Camera and controls

**What it does.** `CameraController` extends `Camera2D` with three modes: follow the selected
character at zoom 1.5, manual drag (right mouse) with wheel zoom clamped to `[0.5, 2.0]`, and an
overview at zoom 0.8 centred on a hardcoded map centre `Vector2(576, 320)`. Space restores the
previous view. Every transition is a `lerp` in `_process`. Selection and movement are mouse-driven
through `CharacterManager._unhandled_input`, which converts the click with
`get_canvas_transform().affine_inverse()`, hit-tests characters with
`PhysicsPointQueryParameters2D`, then chairs by radius, then falls through to "walk here".

**The problem it solves.** A god-view sandbox: watch the whole office, drop into one character, walk
them around by clicking.

**Worth borrowing.** The **three-mode camera with a restore key** is a good pattern for the 2D
reference client — overview for architectural validation, follow for embodiment, free drag for
inspection — and the **click resolution order** (character → interactable → ground) is the right
precedence for a 2D client's targeting. Both are Presentation Pack material.

**Must not be copied.** Nothing here is dangerous; it is client code doing client work. Two things
should not be imitated: the hardcoded map centre (a Presentation Pack should read place bounds, not a
literal), and `zoom_camera()` writing back into the *exported default* (`character_zoom`,
`zoom_level`) so a runtime zoom silently redefines the mode's default.

**Does MineWorld already have a stronger abstraction?** Not applicable and deliberately so — camera is
Presentation ([`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md) §12; `DEP-9` hands camera to Godot).
MineWorld has no 2D camera controller; its renderer spikes are throwaways. This is straightforwardly
an area where a shipped project has something and we have nothing, and the right response is to write
our own small one in the 2D client, informed by their mode structure.

**Godot feature or library revealed.** `Camera2D` already provides `position_smoothing_enabled` /
`position_smoothing_speed` and limit rectangles; the hand-written `lerp` in `_process` is unnecessary.
`Camera2D.make_current()` and `get_viewport().get_camera_2d()` are how a client should locate its
camera — which Microverse does correctly.

## 4.9 Save / load

**What it does.** `GameSaveManager` (autoload) serialises a `Dictionary` to
`user://saves/<name>.json`: a version string, a wall-clock timestamp, the scene name, a `characters`
array, a `rooms` map, and a `global_state` holding the settings. Per character it intends to record
position, facing, sitting state, current chair by name, `is_player_controlled`, AI state and tasks.
Loading matches characters by node name and re-applies. A separate `ChatHistory` node per character
writes `user://chat_history/<node name>_history.json` on **every message**, and
`BackgroundStoryManager` writes its own `user://custom_social_rules.json`.

**The problem it solves.** Session continuity for a single-player game with no server and no
database.

**Worth borrowing.** One genuine design point: **control binding is saved world state**, not UI
state. And one operational point: separate files for separate concerns (save, chat history, settings,
per-character AI settings, custom rules) keeps a corrupt file from destroying everything — a modest
but real property that a single monolithic blob does not have.

**Must not be copied — and this area is the strongest single argument in the audit for MineWorld's
design.** Three compounding problems.

1. **Identity is the node name.** `find_character_by_name`, `find_chair_by_name`, chat history file
   names, memory keys, relation keys and `CharacterPersonality` lookups are all strings matched
   against `Node.name`. Renaming a node in the editor orphans that character's entire history.
   `INV`-wise this is the absence of a stable `EntityId`: [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §3
   requires an id that is *allocated, monotonic and never reused*, and
   `kernel/src/entities.rs` is the only allocator.
2. **The save is not the state.** Semantic state lives in node metadata (`money`, `mood`, `health`,
   `relations`, `character_data.memories`, `character_data.tasks`), and
   `collect_character_data` reads **none** of those keys. Memories, relations, money, mood and health
   are simply not in the save file. Rooms *are* written into the save
   (`GameSaveManager.gd:100-112`) and nothing ever reads them back, because rooms are re-derived from
   the scene on load — dead data in a save format.
3. **The save path cannot run as published.** `collect_character_data` and `apply_character_data` call
   `controller.has_property(...)` and `ai_agent.has_property(...)` at thirteen sites.
   `Object.has_property()` does not exist in Godot 4 — verified against Godot 4.7.2 in Appendix A,
   where the call raises `Invalid call. Nonexistent function 'has_property' in base 'Node'` and
   aborts the enclosing function. Since the first such call is unconditional once a controller is
   found, `collect_character_data` returns `null` for every character and `collect_game_data` skips
   each one, so a save written by this snapshot contains `"characters": []`.

That third item is a shipped-demo bug and this audit does not dwell on it as a quality judgement. It
is cited because of *why it went unnoticed*: there is no type that says what a saved character is, no
schema to validate against, no test that round-trips a world, and the state being saved is a bag of
`Variant`s on a scene node. [`ENGINEERING_STANDARDS.md`](../ENGINEERING_STANDARDS.md) §12 (strong
typing), §14 (invalid states should be difficult to represent), §§20–21 (integration-first, and
continuous real-world execution: create → act → persist → restart → verify) and `AC-6` exist to make this
class of failure impossible rather than unlucky.

**Does MineWorld already have a stronger abstraction? Yes in design; unimplemented.**
[`ARCHITECTURE.md`](../ARCHITECTURE.md) §§7-8 specify event sourcing with periodic snapshots,
`snapshot + events` reconstruction, and a `PersistenceBackend` (`DEP-2`: `rusqlite`) behind which the
database is invisible to world semantics (`INV-14`). `Component` carries an explicit
`schema_version`; `EntityRegistrySnapshot` and `RelationStoreSnapshot` exist; `World::genesis` records
a pack's initial facts with `Causation::WorldGenesis` so that even initial state is *explained*. None
of it is written yet — persistence is `❌` in `MVP_STATUS.md`. What Microverse's failure mode tells us
is which test to write first: a world loaded, acted upon, persisted, restarted and compared, with a
renamed entity somewhere in it.

**Godot feature or library revealed.** `FileAccess` + `JSON` is all GDScript has, and a per-message
full-file rewrite (`ChatHistory.add_message` → `save_history`) is the natural consequence. Nothing to
reuse; it reinforces `DEP-2` (SQLite, server side) and the rule that a client persists nothing
authoritative.

## 4.10 Character configuration

**What it does.** `script/CharacterPersonality.gd` is a `const PERSONALITY_CONFIG` dictionary keyed by
character name, with five prose fields each: `position`, `personality`, `speaking_style`,
`work_duties`, `work_habits`. The eight entries are long, specific and very well written — they are
the reason the demo has a voice. `get_personality(name)` returns the entry or a two-field default.
Each character is also a `.tscn` scene under `scene/characters/`, and its portrait and body sprites
are named after it (`AliceP32.png`, `Alicex32.png`).

**The problem it solves.** Making eight characters feel distinct with no authoring tool, in a form a
prompt can consume directly.

**Worth borrowing.** Two things, and the first is more valuable than it looks.

- **The five-field schema is a good schema.** `position` / `personality` / `speaking_style` /
  `work_duties` / `work_habits` separates *what they do*, *who they are*, *how they sound* and *how
  they behave in routine* — four axes that a model uses differently, and that a World Pack author can
  fill in without instruction. `speaking_style` as a field of its own is the single highest-leverage
  one for output quality, and MineWorld's `AuthoredPerson` has no equivalent.
- **A roster constraint in the prompt.** `get_company_employees_info()` appends *"only the employees
  listed above may be mentioned; do not invent new character names"*. Cheap, effective, and exactly the
  kind of thing a Controller Pack's prompt policy should own.

**Must not be copied.** Character configuration is a `const` in a compiled script, so adding a
character means editing source — `fork source → modify game code`, the anti-pattern
[`VISION.md`](../VISION.md) §3.1 names. The key is the node name, so configuration, sprite, save
file, chat history and memory are bound together by a string. Prose is the *storage* format for
traits, not a rendering of them, so nothing can be reasoned about: `_generate_random_task` has to
`to_lower().contains("经理")` on a job title to decide what tasks suit a manager — a substring match
on prose standing in for a typed role. The fallback returned by `get_personality` for an unknown name
omits `position`, `work_duties` and `work_habits`, while callers index `personality["position"]`
directly (`AIAgent.gd:518`), so an unlisted character is an error rather than a default. And
`relations` are `{type: String, strength: int}` keyed by name, with `strength` interpreted by an
if-ladder of magnitude words — structured enough to be useful, untyped enough to diverge.

**Does MineWorld already have a stronger abstraction? Yes.**
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §4 splits a Person into identity, traits, mutable state and
relationships, each field owned by whichever System provides it, so *a world that does not enable that
System simply does not have the field*. §4.4 is explicit that relationships are structured typed edges
and that narrative text may be **generated** from them for display or a prompt but is **never read
back as state** — which is precisely the inversion Microverse performs. `AuthoredPerson` in
`worldpack/src/format.rs` is authored YAML per Person, one file per key, with unknown fields refused
and no field for a secret. Adding a Person is adding a file, not editing a script. What we lack is the
*content* schema for traits, and their five fields are the best candidate to start from (§9.4).

**Godot feature or library revealed.** `Resource` subclasses with `@export` fields and `.tres` files
are the idiomatic Godot answer to data that a designer edits without touching code, and are what a
Presentation Pack should use for its own per-character presentation data. Not for simulation data,
which belongs in the World Pack.

## 4.11 UI and settings

**What it does.** `SettingsManager` (autoload) holds one `Dictionary` mixing model settings
(`api_type`, `model`, `api_key`), a display toggle (`show_ai_model_label`) and window settings
(`window_mode`, `screen_width`, `screen_height`), persisted to `user://settings.cfg` as JSON, with a
`settings_changed` signal for decoupling. Per-character overrides live in a second file.
`GlobalSettingsUI` and `CharacterAISettings` provide tabbed panels; `CharacterAISettings` auto-saves
on character switch. `AIModelLabel` is a `Label` under each character showing which provider and model
drives it, globally toggleable. `GodUI` (1,100 lines) is the god-mode panel: a character list, a detail
pane with tabs, and popups to implant a memory, inflict a disease with a severity slider, grant or
remove money with a stated reason, set a directed emotion between two characters with a strength
slider, add/complete/delete tasks, and edit the world's social rules.

**The problem it solves.** Configuring models without recompiling; seeing at a glance which model
drives whom; and — most interestingly — giving a person direct authorial control over the simulation
while it runs.

**Worth borrowing.** Three ideas, all of which MineWorld wants.

- **`AIModelLabel`.** A per-Person marker showing which Controller and which model decides for it,
  toggleable. In a world where `Alice → human`, `Bob → local Qwen`, `David → deterministic NPC`
  ([`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §14), being unable to see that binding while playing is a
  real handicap for development. Trivially cheap; high diagnostic value.
- **A signal-based settings singleton with per-subject override and fallback.** `get_character_ai_settings`
  falling back to the default is the exact shape a cognition profile with per-Person overrides needs.
- **The god-mode surface itself.** *Implant a memory that never happened* is not a hack; under
  `INV-4` it is a first-class capability, because memory is subjective belief and the world's truth is
  the event log. Microverse's most "game-like" debug feature is accidentally the best demonstration
  available of why `INV-4` is worth having. Inflicting a state change with a **stated reason** that is
  also written into the character's memory is likewise the right instinct: a cause travels with the
  effect (`CausedBy`, `INV-15`).

**Must not be copied.** `GodUI` writes authoritative state directly:
`character.set_meta("health", ...)` (`:366`), `set_meta("money", ...)` (`:424`),
`set_meta("relations", ...)` (`:482`), `set_meta("tasks", ...)` (`:641`),
`set_meta("character_data", ...)` (`:694`, `:721`, `:790`). No validation, no event, no owning System,
no traceability — the UI is a writer of world state. It also re-implements task rules that
`AIAgent.gd` already implements. API keys are stored in plaintext JSON and interpolated into a header
template in a module that also `print`s the URL. And one `Dictionary` conflating model credentials with
window size means the settings type cannot be validated as either.

One thing they got right and we must keep right: **the key is in the user's settings file, not in the
world content.** [`ARCHITECTURE.md`](../ARCHITECTURE.md) §9.2 requires exactly that — credentials
belong to the server operator and never appear in a World Pack — and
`worldpack/src/lib.rs` enforces it structurally by having no field for a secret and refusing unknown
fields.

**Does MineWorld already have a stronger abstraction? For the writing path, yes and emphatically.**
The only way to change state is an `ActionRequest` → `ActionIntent` → the owning System's
`validate`/`resolve` → `Event` → `react`, with `WriteToken<S>` making a cross-system write fail to
compile and `WriteAccess::new` crate-private so a token cannot be forged (pinned by
`kernel/tests/compile_fail/`). `ClientFrame` has exactly two variants, `Join` and `Submit`, so *"my
money is now 5000"* is not a frame that exists and is refused as a protocol violation. A `GodUI`
equivalent is therefore impossible to build the way theirs is built — which is the point, and which
also means we owe an answer for the legitimate need behind it (§9.5). For the settings and labelling
path, we have nothing at all.

**Godot feature or library revealed.** `Window.mode`, `content_scale_mode`,
`DisplayServer.screen_get_size` and the `CONTENT_SCALE_ASPECT_EXPAND` handling in
`apply_display_settings` are a tidy, correct reference for the display-settings part of both reference
clients. `Label.add_theme_font_size_override` / `font_outline_color` for a world-space nameplate is the
right primitive for `AIModelLabel`'s equivalent.

## 4.12 Scene and asset organisation

**What it does.** One scene per map (`scene/maps/Office.tscn`, 17,052 lines, 70 nodes) containing the
tile art, a `Characters` node with eight instanced character scenes, a `RoomArea` node with nine
`Area2D` children carrying `room_name` / `room_desc`, furniture instanced from `scene/prefab/`, a
`NavigationRegion2D`, the camera, and a `CanvasLayer` with the UI. Characters, UI panels and prefabs
are separate `.tscn` files. Assets are grouped by kind (`characters/body`, `characters/portraits`,
`maps/interiors`, `maps/exteriors`, `objects`, `singleObjects`, `ui/theme`, `ui/shaders`, `fonts`).

**The problem it solves.** Shipping one hand-authored playable space with reusable furniture.

**Worth borrowing.** The **prefab split** (`Chair.tscn`, `Desk.tscn`, `FrontChair.tscn`,
`BackChair.tscn`) and the asset-by-kind layout are both sound and match what a Presentation Pack needs
([`MODULE_SPEC.md`](../MODULE_SPEC.md) §6.1, `ARC-7`). The idea of **authoring rooms as volumes in the
editor** is genuinely good authoring ergonomics — see §5.2 for where it belongs in MineWorld.

**Must not be copied.** The scene is the world: the set of Places, their descriptions, who exists,
where they start, and the map's identity are all inside a `.tscn`, so a different renderer would have
to re-author all of it, and a headless run cannot obtain any of it. Three maps are configured in
`BackgroundStoryManager.BACKGROUND_CONFIGS` (`Office`, `School`, `Jail`) but only `Office.tscn`
exists, and the environment paragraph is selected by `match get_tree().current_scene.name` inside
`AIAgent.get_environment_info()` — world content dispatched on a scene node's name, inside the agent.

**Does MineWorld already have a stronger abstraction? Yes.** A World Pack is *semantic configuration
and population, not code*: `world.yaml` plus `people/<key>.yaml` plus `places/<key>.yaml`, read by
`mineworld-worldpack`, which refuses unknown fields, refuses a fact whose owning System is not
enabled (`catalog::Capability`), and loads in a deterministic order because that order reaches the
event log. A Presentation Pack then renders it, and swapping the pack changes nothing about the
simulation ([`MODULE_SPEC.md`](../MODULE_SPEC.md) §6). Microverse's arrangement is a scene that *is*
the world; ours is a world that a scene *binds to*, via `PerceivedEntity::id` as the join key
(`DD-14`).

**Godot feature or library revealed.** `Desk.gd` recomputes its `z_index` on a 0.1 s `Timer` by
scanning every character and chair in the scene — the native answer is `y_sort_enabled` on the
parent `Node2D`/`CanvasItem`, which Godot implements for exactly this problem. A small, concrete
reuse note for the 2D client, and one more instance of the pattern in §4.7: mature engine features
replaced by hand-written equivalents.

---

# 5. The three questions this audit was asked to answer directly

## 5.1 Should MineWorld abstract one layer above `APIConfig`? — Yes. A concrete opinion.

**The pattern to keep.** One declarative description per provider, four fields, data-driven request
and response handling, and a per-`Person` binding that falls back to a world default. Microverse got
that right and it is what MineWorld needs.

**The pattern to change.** `parse_response(api_type, response) -> String`. A provider that returns a
string has pushed every hard problem to its callers, and Microverse's source is the proof: a
2,213-line agent whose largest category of code is parsing model prose back into decisions, plus
~14 `_default` functions that exist to handle the case where the parse failed.

**The proposal.** A `ModelProvider` that *declares capabilities* and *returns typed results*. Stated
as a shape, not as a signature to be adopted verbatim:

```text
ModelProvider
    capabilities()  -> Capabilities
        structured_generation   can it be constrained to a schema, and how
        streaming               token streaming
        vision                  image input
        embeddings              vector output, and its dimensionality
        image_generation        for tools/, never for the runtime (ARC-10)
        tool_use                function calling
        context_window          tokens
        deterministic_seeding   whether a seed is honoured at all

    complete(CompletionRequest)            -> Completion { text, usage, finish_reason, ... }
    generate<T>(StructuredRequest<T>)      -> T                     requires structured_generation
    embed(EmbedRequest)                    -> Embeddings            requires embeddings
```

Four arguments, in order of weight.

1. **The frozen architecture already requires a capability layer.**
   [`ARCHITECTURE.md`](../ARCHITECTURE.md) §9.2 states that a world declares
   `requires: { structured_generation: true, context_window: ">=16k" }`. A provider that returns
   `String` **cannot answer that declaration**, so a capability query is not an extra layer we are
   proposing — it is the missing implementation of a decision already taken. Microverse's table has
   no field that could answer it.
2. **A typed result moves failure handling from every call site to one.** Their
   `match decision: "1" / "2" / _` with a default branch is repeated in shape at every decision point.
   A `generate<Decision>()` that either yields a `Decision` or fails leaves exactly one place to
   decide what an unusable answer means — and that place is where the deterministic fallback of §9.1
   belongs.
3. **The consumer's target type is already fixed.** A Controller must emit an `ActionRequest`
   naming an `ActionTypeId` with an encoded payload, and the world answers
   `Accepted`/`Rejected`/`Unavailable`. So the *useful* output of cognition is not prose but a
   candidate request, and the provider interface should be able to carry a schema-constrained value
   the Controller can turn into one. `INV-10` remains untouched: constraining generation to the
   actions a world provides is an optimisation, and the world's refusal is still the authority — a
   controller that asks to shoot in a world with no combat system gets `Unavailable`, however the
   generation was constrained.
4. **Capabilities are for admission, not for branching.** The failure mode of a capability enum is a
   `match` on capability at every call site — a second `APIConfig` in a different shape. The rule
   should be: capabilities are checked **once**, when a Controller Pack is bound to a provider, and a
   world whose declared requirements the provider cannot meet fails to compose, the same way
   `SystemRegistry` refuses a `ConversationSystem` with no `PresenceSystem` rather than discovering it
   at run time.

**Where it lives, and where it must not.** In Python `cognition/`, inside a Controller Pack
([`MODULE_SPEC.md`](../MODULE_SPEC.md) §5 assigns `model routing`, `prompting`, `memory policy` and
`cognition budgets` there), behind a MineWorld-owned interface
([`REUSE_POLICY.md`](../REUSE_POLICY.md) §5). **Never** in `mineworld-contracts` and never in the
kernel: `INV-12` (no domain semantics in the kernel), `INV-14` (semantics never depend on the model
provider), and `MODULE_SPEC.md` §5's hard constraint 5 (provider-specific details stay behind the
model-backend interface; cognition contracts stay provider-neutral).

**Reuse before building it.** Six of Microverse's nine providers are OpenAI-compatible, and three of
[`ARCHITECTURE.md`](../ARCHITECTURE.md) §9.2's four first-class backends (OpenAI-compatible endpoint,
vLLM, llama.cpp server) are the same adapter. So the table is small, and before hand-writing it,
[`REUSE_POLICY.md`](../REUSE_POLICY.md) §1 requires investigating the maintained clients
(`openai`, `anthropic`, `ollama`, `google-genai`) and the routers (LiteLLM and similar), with a
`DECISIONS.md` record for whichever way it goes — including for choosing our own.

**Classification.** `APIConfig`'s provider-table *pattern*: **ADAPT**. Its `-> String` contract:
**REJECT**. Its per-character binding: **ADAPT**. Its code: not imported.

## 5.2 `RoomManager` derives semantic rooms from the scene; MineWorld runs the arrow the other way

**Their direction.** `RoomManager._init_rooms()` walks `get_tree().get_nodes_in_group("room_area")`,
reads each `Area2D`'s exported `room_name` and `room_desc`, takes the `CollisionShape2D`'s
`RectangleShape2D.extents * 2` as the room's size and `area.global_position + collision_shape.position`
as its centre, and builds a flat `Dictionary` of `RoomData`. `get_current_room(rooms, position)`
linearly scans and returns the first room whose AABB contains the point.

```text
Microverse     scene geometry  ──derives──►  semantic room
MineWorld      semantic Place  ──bound by──►  rendered geometry
```

**What their direction buys — and it is not nothing.**

1. **One authority for space, so nothing can disagree.** A level designer draws a volume and types two
   strings; there is no second artefact to keep in step, and no possibility of a Place existing that
   the map does not show or vice versa. That is a real class of bug eliminated.
2. **Authoring cost near zero.** A new room is an `Area2D` with a shape and two exported strings. No
   schema, no id, no file, no loader. For a one-map game this is the correct engineering trade.
3. **Geometry-derived answers come free.** "Which room is this point in", "how far is that room",
   "which direction is it" are all immediate, because the semantic room *is* a rectangle. Their
   whole-map prompt summary (§4.6) exists because that information was already there.

**What it costs.**

1. **No headless world.** Deriving Places requires instantiating `Area2D` and `CollisionShape2D`
   inside a running engine. There is no configuration a server could read. This is fatal against
   `AC-11` (hundreds of simulated days with no renderer and no model), `AC-3` (the simulation
   continues while the client is disconnected) and `AC-8` (laptop/Docker parity), and it is `INV-14`
   directly: simulation semantics must not depend on the renderer. For Microverse this cost is
   invisible, because there is no server — the simulation *is* the Godot process.
2. **Two clients would derive two different worlds.** `AC-13` (2D/3D semantic parity) and `AC-15`
   (*there is only one Alice*) are unreachable by construction: a 3D scene would derive a different
   room set from different volumes, and neither derivation is authoritative.
3. **Only geometric Places are representable.** `rooms` is flat: no hierarchy, no adjacency. A café
   inside a town inside a world, a Place two hours' travel away, an `Organization`'s remote office, or
   the "place" a phone call happens in cannot be expressed. [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md)
   §6 makes hierarchy a `Relation` precisely so that it exists without geometry, and
   `mineworld-presence` writes `present_in` edges into the `Observation` for exactly this reason.
   Microverse's authored `room_desc` strings compensate by *describing* adjacency in prose ("西边连通
   HR办公室"), which a model can read and no code can traverse.
4. **The geometry's units become the semantic units.** Pixel distances are handed to the model as
   metres (`AIAgent.gd:119`, `:259`). Nobody decided that; it follows from the semantic layer having
   no units of its own. MineWorld's `LocalPosition` is `i32` millimetres with a stated frame
   (`+x` east, `+y` north, `+z` up, yaw from `+y` toward `+x`) because our own spike found that two
   independently written clients otherwise invent two conventions.
5. **Ambiguity resolved by scene-tree order.** Overlapping volumes are resolved by whichever `Area2D`
   the group iteration reaches first — a semantic fact decided by node ordering, and not reproducible
   across edits.
6. **Persistence has nothing to persist.** `GameSaveManager` writes room name, position and size into
   the save and nothing reads them back, because on load the rooms are re-derived. A save format
   carrying dead fields is the symptom of a world whose state is not the save's business.

**Why ours has to be ours.** Not because deriving is wrong, but because MineWorld's acceptance
criteria are *specifically* the ones a derived world cannot meet. `AC-15` requires one Alice seen
through a 2D client, a 3D client and an agent at the same instant; that requires an authoritative
`PlaceId` that neither renderer owns. `AC-11` and `AC-12` require the world to run and reproduce with
no renderer in the process. `INV-5` says presentation observes and never mutates — and a renderer that
*defines* the set of Places has mutated the world's ontology, which is a stronger form of ownership
than writing a component. So the arrow is forced: a semantic `Place` exists in the authoritative
world; a renderer binds to it by `EntityId`; the binding is the client's private business and the
contract says nothing about it.

**But their direction survives, in the one place it belongs.** Authoring in an editor is genuinely
better than authoring YAML by hand, and nothing stops a **tool** from reading a Godot scene and
*emitting* World Pack YAML — draft `places/<key>.yaml` entries with name, description and bounds
derived from `Area2D` volumes, reviewed by a human, committed as content. `ARC-10` already establishes
that generation machinery lives in `tools/`, is never a runtime dependency, and is never referenced by
a contract. That keeps their authoring ergonomics and inverts the arrow at build time rather than at
run time. Recorded as a proposal in §9.6, not as a decision.

**Classification.** Deriving Places from scene geometry at run time: **REJECT** (`INV-5`, `INV-14`,
`AC-3`, `AC-8`, `AC-11`, `AC-15`). Authoring Place volumes in an editor and exporting World Pack
content from them: **ADAPT**, as a `tools/` job. Authored per-Place prose descriptions as first-class
World Pack content: **ADAPT**.

## 5.3 Which memory heuristics survive `event log → observation → subjective memory → compression`?

Microverse's memory **is** the world's history: there is no event log, so `Alice.memories` is both what
Alice believes and the only record that anything happened. MineWorld's chain makes memory *derived* —
`INV-11` (the Event Log is the single source of truth for history) and `INV-4`
(`Memory ≠ Biography ≠ World Truth`). Some of their heuristics are unaffected by that change, some
change shape, and some exist only because memory is primary.

| # | Their heuristic | Verdict | What it becomes in MineWorld |
| --- | --- | --- | --- |
| 1 | Five categories: `PERSONAL`, `INTERACTION`, `TASK`, `EMOTION`, `EVENT` | **Survives, with a condition** | A tag on a derived memory, set by the deriving Controller Pack. The condition: adopt the axis only when a retrieval path filters on it. In their code nothing does — the taxonomy is written and never read, which is how an unused axis looks |
| 2 | Four importance levels `1 / 3 / 5 / 10` | **Survives, relocated** | A coarse ladder beats a continuous score for prompt selection. But theirs is assigned **by the writer at the call site** (player implant = `HIGH`, task refresh = `NORMAL`), which encodes author intent. In our chain importance must be **derived** at memory-construction time from properties of the `PerceivedEvent` and the observer, or it is authoring dressed as cognition |
| 3 | Select by importance, then recency, take top *N* | **Survives directly** | The single most reusable heuristic here. It is the selection policy for the `L0`/`L1` layers of [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §5.4, and it belongs to a Controller Pack's memory policy |
| 4 | Prune to 50, keeping highest importance then newest | **Changes meaning** | As *deletion of the only copy* it violates `INV-11` — their prune destroys history and no biography can be regenerated. As **forgetting** it is legitimate and wanted: a Person genuinely may forget. The mechanism becomes compression `L0→L1→L2`, and `AC-10` requires every summary to retain the underlying Event IDs, so the original facts are always recoverable through the log |
| 5 | Timestamps from `Time.get_unix_time_from_system()` | **Does not survive** | Must be `WorldTime`. `AC-12` requires the same inputs and seed to reproduce a run, and a wall clock guarantees they will not. `Heard::at()` is already `WorldTime`, and `ConversationHistory`'s documentation states the reason |
| 6 | Recency window of 24 hours (`get_recent_memories`) | **Survives, retyped** | A `SimDuration` window over `WorldTime`. The idea that recency is measured in *the world's* time, not the player's session, is what makes it meaningful across a save |
| 7 | Prose strings as the stored form, concatenated into the prompt | **Does not survive as storage; survives as rendering** | `Heard` stores `PersonId`, `WorldTime` and `Utterance`. Prose is *generated* for a prompt and never read back as state ([`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §4.4). Their `_generate_random_task` matching a substring of a job title is what happens when generated prose becomes the only representation |
| 8 | `search_memories` by keyword substring | **Unresolved; record and defer** | Dead code in their repository, so it carries no evidence. Retrieval strategy belongs to a Controller Pack's memory policy, and [`REUSE_POLICY.md`](../REUSE_POLICY.md) §1 lists embedding/vector search as commodity infrastructure to adopt rather than write when one is needed |
| 9 | Default `max_count = -1` — put **all** memories in the prompt | **Does not survive; it is the named failure** | Bounded only by the 50-entry prune. This is exactly the condition `AC-10` exists to forbid: *after 100 simulated days, character history must not require feeding all historical events to a model*. A memory policy must have a context budget, not a storage cap standing in for one |
| 10 | Memory written by four call sites, two schemas, two metadata keys | **Anti-pattern** | One owner derives subjective memory from `PerceivedEvent`s. `INV-7` and `WriteToken<S>` make the multi-writer version fail to compile. Their consequences are in §4.4 and are not hypothetical |
| 11 | Memory as the only record of what happened | **Rejected, and instructive** | `INV-4`, `INV-11`. What it demonstrates positively: their **implant a memory** feature is only coherent *because* memory and truth are different things. It is the best available argument for `INV-4` from a project that does not have it — they can implant a false belief, and they cannot tell afterwards that it was false, because there is nothing to compare it against |

**The one heuristic of theirs that our chain makes better rather than merely legal.** Importance +
recency selection, applied to memories that are *derived from an event log*, can be recomputed. If a
controller's context turns out to have been badly chosen, MineWorld can re-derive memory from the log
with a different policy and compare the two runs. Microverse cannot: its selection is applied to the
only copy, once, destructively. That is the practical payoff of `INV-11` and it is worth stating in
whatever design eventually implements this.

---

# 6. Architectural coupling register

Each row names a place where semantic game state is coupled to an engine, transport, UI or provider
concept, and the MineWorld rule that coupling would violate if imported. Rules referenced:
`INV-1` Person ≠ Controller · `INV-4` Memory ≠ Biography ≠ World Truth · `INV-5` renderer ownership ·
`INV-6` controllers never mutate · `INV-7` single writer · `INV-8` no entity modifies another ·
`INV-11` event log is history · `INV-12` kernel ignorance · `INV-13` observation scoping ·
`INV-14` semantics independent of renderer, transport, persistence and model provider ·
`INV-15` traceable mutation · **one-way**: World Pack → Systems → kernel contracts
([`ARCHITECTURE.md`](../ARCHITECTURE.md) §14).

| # | Coupling, with evidence | Rule violated |
| --- | --- | --- |
| C-1 | Authoritative semantic state is Godot node metadata: `money`, `mood`, `health`, `relations`, `character_data.memories`, `character_data.tasks` on a `CharacterBody2D` (`AIAgent.gd:273`, `ConversationManager.gd:282-285`, `MemoryManager.gd:26`) | `INV-5`, `INV-14` — world state lives inside the renderer's scene graph and cannot exist without it |
| C-2 | Five writers of that state: `AIAgent`, `DialogManager`, `MemoryManager`, `GodUI`, `GameSaveManager`; two schemas for memory; two keys for tasks (`GodUI.gd:641` writes `meta["tasks"]`, `GodUI.gd:674-694` reads and writes `meta["character_data"]["tasks"]`) | `INV-7` — no owning System, so the same fact has two stores that silently diverge |
| C-3 | The UI mutates world state directly: `set_meta("health")` `:366`, `set_meta("money")` `:424`, `set_meta("relations")` `:482`, `set_meta("tasks")` `:641` in `GodUI.gd` | `INV-5`, `INV-6`, `INV-15` — presentation writes; nothing produces an `Event`; no mutation is traceable |
| C-4 | One entity writes another's position: `Chair.sit_character()` assigns `character.global_position` (`Chair.gd:60`), `stand_up()` assigns it again (`:92`) | `INV-8`, `INV-5` — an `Item` moves a `Person` with no System mediating |
| C-5 | Semantic occupancy and sprite depth in one node: `Chair.occupied` beside `z_index` juggling in the same two methods (`Chair.gd:6, 56, 62-70`) | `INV-5`, `INV-14` — "this chair is taken" is a world fact stored on a render node whose other job is draw order |
| C-6 | `AIAgent` reaches the world by absolute node path and autoload singleton: `/root/DialogManager`, `/root/CharacterManager`, `/root/APIManager`, `/root/Office/RoomManager` (`AIAgent.gd:24-33`) — the last hardcodes the map's node name | `INV-14`, one-way dependency — a Controller depends on the scene's shape, so it cannot run without that scene and cannot run outside Godot |
| C-7 | Perception reads the SceneTree by group-name string: `get_nodes_in_group("interactable")` `:139`, `("character")` `:150`, `("characters")` `:1357`. Neither `interactable` nor `character` is ever joined, so those reads return empty in the shipped build | `INV-13`, `INV-5` — the observation boundary is a string convention over the render graph, and a typo empties it with no error |
| C-8 | Semantic `Place` derived from render geometry: `Area2D` + `RectangleShape2D.extents` → `RoomData` (`RoomManager.gd:12-31`) | `INV-5`, `INV-14` — see §5.2 |
| C-9 | Render units used as semantic units: pixel distance labelled 米/metres in the model's prompt (`AIAgent.gd:119`, `:259`) | `INV-14` — a renderer's coordinate scale became the world's measurement system |
| C-10 | World content dispatched on a scene node's name inside the agent: `match get_tree().current_scene.name` selecting the environment paragraph (`AIAgent.gd:187`) | `INV-14`, one-way — World Pack content decided by a render node's identifier |
| C-11 | HTTP, prompt text, room geometry and `CoffeeMachine` in one unit: `AIAgent.gd` contains `get_object_info`'s furniture table (`:220-261`), prompt assembly (`:516-543`), `HTTPRequest` callbacks (`:561`, `:674`, `:752`, `:961`, …) and provider response parsing | `INV-12` in spirit (domain semantics fused with transport), `ENGINEERING_STANDARDS.md` §9 (no God objects). This is the `CoffeeMachine`-and-HTTP case, and §7 treats it in full |
| C-12 | Prompt logic is a Controller concern executed inside the entity's own node, and reads other Persons' private state to decide what to omit (`ConversationManager.build_dialog_prompt` chooses not to include the listener's memories, `:166`) | `INV-13` — scoping by assembly discipline rather than by construction: the code *can* read everything and elects not to |
| C-13 | Memory is the world's history; a prune destroys it (`MemoryManager._cleanup_old_memories`, `:130-158`) | `INV-4`, `INV-11` — no event log, so belief and truth are one mutable list |
| C-14 | Time is wall-clock throughout: memory timestamps, task refresh (24 h since `Time.get_unix_time_from_system()`), time-of-day in the prompt, decision cadence (`Timer(60)`), conversation ids | `INV-14`, `AC-12` — simulation semantics depend on the host clock, so no run reproduces |
| C-15 | Identity is the node name: character lookup, chat-history filenames, memory and relation keys, personality lookup (`GameSaveManager.find_character_by_name`, `ChatHistory.get_history_file_path`, `CharacterPersonality.get_personality`) | `INV-11`, `AC-6` — renaming an editor node orphans a Person's whole history; no stable `EntityId` |
| C-16 | Interaction range evaluated in the renderer's units, in three places (`CharacterManager.gd:5`, `DialogService.gd:8`, `DialogManager.gd:170`) | `ENGINEERING_RULES.md` §§7-8 — a rule evaluated in a renderer is a defect, and a duplicated rule is two rules |
| C-17 | Model-provider concepts inside the entity's decision code: `APIConfig.parse_response(api_manager.current_settings.api_type, response)` called from `AIAgent` (`:579`) | `INV-14`, `MODULE_SPEC.md` §5 constraint 5 — provider identity leaks into the character's own logic |
| C-18 | Task rules duplicated between agent and UI: `_refresh_daily_tasks`, `_check_daily_task_refresh`, `_generate_random_task` in both `AIAgent.gd` and `GodUI.gd` | `ENGINEERING_STANDARDS.md` §8 change-amplification — one rule change requires two edits in two layers |
| C-19 | A read mutates: `get_character_task_info()`, used to build a prompt, calls `_check_and_refresh_daily_tasks` (`AIAgent.gd:361`) | `INV-15` — observing changes the world, and the change traces to nothing |
| C-20 | Presentation created and owned by cognition: `ConversationManager._on_request_completed` instantiates a `DialogBubble` into the scene root, writes two JSON files, and fires the next model call (`:188-253`) | `INV-5`, `INV-6` — one function spans cognition, presentation and persistence |

---

# 7. `AIAgent.gd` as a maintainability case study

`script/ai/AIAgent.gd` is 90,904 bytes / 2,213 lines / 63 functions, attached as a child `Node` to
each `CharacterBody2D`. Eight such instances exist at run time. Its responsibilities, read off its
own function list:

```text
perception          generate_scene_description, get_room_objects, get_room_characters,
                    get_environment_info, get_object_info, get_direction_description
world geometry      room bounds arithmetic, distance and compass computation
personality         CharacterPersonality lookups, prompt persona construction
memory              _add_memory, MemoryManager calls, recent-memory formatting
company/world lore  get_company_basic_info, get_company_employees_info
tasks               _check_and_initialize_tasks, _generate_initial_tasks, _adjust_tasks,
                    _continue_current_task, _complete_task, _refresh_daily_tasks,
                    _generate_random_task, _reschedule_task, _rearrange_task_priorities
prompts             ~10 distinct prompt templates inline
transport           ~9 HTTPRequest completion callbacks, JSON parsing, provider dispatch
decisions           make_decision, make_conversation_decision, ~14 _default fallbacks
movement            move_to_target, _choose_random_target, _execute_task_movement,
                    _start_movement_tracking, _check_movement_progress
social behaviour    initiate_conversation, _get_conversation_partner,
                    _generate_farewell_message, _send_farewell_and_end_conversation
time                wall-clock hour → prompt, 24 h task refresh, 60 s decision cadence
```

**The finding is not the size.** It is that one unit knows `CoffeeMachine`, room AABB arithmetic,
`Time.get_unix_time_from_system()`, `HTTPRequest.RESULT_SUCCESS`, `APIConfig`'s provider identity,
`CharacterPersonality`'s prose schema, `MemoryManager`'s dictionary schema, `RoomManager`'s node path,
`DialogManager`'s conversation state and `CharacterBody2D.move_to` — and that changing any one of them
is a change to this file. Adding an `Item` type means editing it. Adding a provider means editing it.
Changing the memory schema means editing it. Adding a new interaction means editing it. That is the
change-amplification failure [`ENGINEERING_STANDARDS.md`](../ENGINEERING_STANDARDS.md) §8 tests for,
and the God-object prohibition of §9.

It is also worth saying why it happened, because the cause is not carelessness: with no kernel, no
action dispatch, no typed `Observation` and no provider interface, **there is nowhere else for any of
this to go**. The file is the shape a competent developer arrives at when the architecture underneath
is a scene tree and an HTTP client. This is the clearest justification in the audit for MineWorld
paying the up-front cost of a kernel.

## 7.1 Which responsibilities MineWorld separates, through which mechanism, with our type names

| `AIAgent` responsibility | MineWorld's owner | The mechanism that makes convergence impossible |
| --- | --- | --- |
| Deciding what a character attempts | a **Controller**, outside the world | `INV-1`, `INV-6`. A Controller receives `Observation` and emits `ActionRequest`; `ActionIntent::allocate` is the only way an intent exists and only the server calls it, so a Controller cannot even name the identity of its own request |
| Producing what a character knows | a perception System — today `mineworld_presence::observe()` | `Observation` is a **value**: no component store, no registry, no world handle, no query method. `Observation::entity(id)` searches the list and nothing else, so `INV-13` holds by construction rather than by assembly discipline |
| Deciding whether an action is possible | the owning System's `validate`, plus `SpatialRequirement::evaluate` | `System::validate` is handed a `WorldRead<'_>` — reads only — and one shared evaluator answers every spatial requirement, so an `Affordance` and a dispatch of the same request cannot disagree |
| Telling a client what may be attempted | `Affordance`, computed by the server | `Affordance::available` / `Affordance::unavailable` cannot represent "available because too far away", and deserialization applies the same check. No client computes availability |
| Knowing what an action of another pack means | nothing — the question is asked, not answered | `InteractionProvider::offers()` returns `Offer`s; `mineworld_presence` prices them without knowing what any action is, and `tests/presence.rs` checks structurally that no other pack's action name appears in its sources |
| Changing where a Person is | `PresenceSystem` alone, writing `Presence` | `owned_component!` + `OwnedBy<PresenceSystem>` + `WriteToken<PresenceSystem>`: a write to another pack's component **cannot be named**, pinned by `kernel/tests/compile_fail/a_system_cannot_write_another_systems_component.rs` |
| Changing what a Person remembers | `ConversationSystem::react`, reducing its own `Spoke` fact | `ConversationHistory` has exactly one writer, so a replay rebuilds identical memory (`AC-12`). Its documentation forbids growing it into a memory system |
| Recording that something happened | the kernel, at `record` | Dispatch assigns identity, instant, `Causation` and `Provenance`; a System returns `Emission`s and never appends to a log. `Causation` is never absent, so `INV-15` holds for initial state too (`Causation::WorldGenesis`) |
| Reacting to something another pack did | `System::react`, in registration order | Cross-domain effects travel as `Event`s; `EmploymentSystem` emits `WageDue` and only `EconomySystem` moves money |
| Talking to a model | a Controller Pack in `cognition/`, behind a `ModelProvider` | `INV-14`; `MODULE_SPEC.md` §5 constraint 5. No `Event`, `Component`, `Action` or contract type may name a provider |
| Assembling a prompt | a Controller Pack's prompt policy | `MODULE_SPEC.md` §5 lists `prompting` as a Controller Pack concern. `Utterance` carries no language tag, because a language tag is presentation (`DD-13`) |
| Knowing what an `Item` is and what it affords | the System Pack that owns it | `INV-12`. `Talk` and `Arrive` are `ActionTypeId`s declared by their packs; `mineworld-contracts` names no action, and its only action names are in its own tests |
| Knowing the time | the world clock (S4), passed in | Dispatch is *told* the instant; deferrals come back in `Dispatched::deferred` rather than being scheduled, so the scheduler cannot re-plumb the pipeline |
| Moving a body | the client, from authoritative `Location` | `ENGINEERING_RULES.md` §6 keeps `MoveIntent`, the travel `Process`, authoritative spatial state and rendered movement as four things; `Location`'s `LocalPosition` is `i32` millimetres because positions reach the log |
| Speaking to a network | `mineworld-server` | `ClientFrame` has two variants, `Join` and `Submit`; there is no frame that asserts a fact, and the kernel never sees an HTTP or WebSocket type |
| Storing anything | the persistence backend (S5) | `INV-14`; a behaviour that appears only under one backend is a defect in that backend |

**The one-line test to apply in review.** If a MineWorld unit would need to change when a new `Item`
type, a new model provider, a new renderer, a new memory schema *or* a new interaction is added, it
has started to become `AIAgent.gd` and the design goes back.

---

# 8. Classification index

Every idea this audit extracted, with its class and the reason.

| Idea | Class | Note |
| --- | --- | --- |
| Provider description table (URL / headers / request shape / response shape) | **ADAPT** | §5.1. Shape yes, `-> String` no, code no |
| `request_format` and `response_parser` as separate fields | **ADAPT** | Correct factoring; keep it |
| Per-`Person` model binding with fallback to a world default | **ADAPT** | Axis we have not specified; complements `ARCHITECTURE.md` §9.1's decision tiers |
| Provider returning `String` | **REJECT** | Pushes every hard problem to callers; origin of `AIAgent.gd`'s size |
| Credentials in user settings rather than in world content | **ADAPT** | Already our rule (`ARCHITECTURE.md` §9.2); their placement confirms it. Plaintext storage and log exposure: **REJECT** |
| Deterministic fallback beside every LM decision | **ADAPT** | §9.1. Highest-value structural idea in the repository |
| In-flight guard: one outstanding model call per character | **ADAPT** | With `ARCHITECTURE.md` §9's revalidation, which they lack |
| Hand-rolled recovery for stale intents (`_handle_target_not_found`, movement tracking) | **REFERENCE ONLY** | Evidence *for* the revalidation seam; five of their functions replace one of our contracts |
| Conversation as an addressable instance with a lifecycle and signals | **ADAPT** | §9.3. The concrete shape of a `Process` |
| Modelled graceful exit (a generated farewell, then end) | **ADAPT** | §9.3 |
| Unbounded recursive turn alternation | **REJECT** | No turn budget; unbounded cost |
| Two-participant conversation as a typed pair | **REJECT** | `VISION.md` §3.1 names this as a disqualifying demo assumption |
| Memory categories (5) | **ADAPT** | §5.3 row 1 — only with a retrieval path that uses them |
| Memory importance ladder (4 levels) | **ADAPT** | §5.3 row 2 — derived, not assigned by the writer |
| Importance-then-recency selection, top *N* | **ADAPT** | §5.3 row 3 — the single most reusable heuristic here |
| Prune-to-*N* as destruction of the record | **REJECT** | `INV-11`; becomes compression retaining Event IDs |
| Wall-clock timestamps on memory | **REJECT** | `AC-12`; must be `WorldTime` |
| Recency window over world time | **ADAPT** | `SimDuration`, not hours of wall time |
| Prose as the stored form of memory and traits | **REJECT** | `CORE_CONCEPTS.md` §4.4 — generated for a prompt, never read back as state |
| All memories into the prompt by default | **REJECT** | The `AC-10` failure, named |
| Memory as the world's only history | **REJECT** | `INV-4`, `INV-11` |
| "Implant a memory" as a first-class capability | **ADAPT** | §9.5. Coherent only under `INV-4`, which is the point |
| Single scalar priority the model may re-rank ("degree of desire") | **ADAPT** | A Controller Pack's intention model, not authoritative state |
| Daily top-up of intentions from a pool | **REFERENCE ONLY** | Good for a workplace demo; the pool is hardcoded content |
| Tasks in untyped node metadata, two keys, two writers | **REJECT** | `INV-7`; §4.5 |
| Task rules duplicated in agent and UI | **REJECT** | Change amplification |
| Perception that includes reachable elsewhere, not only here | **ADAPT** | Independently confirms our spike finding `F3` |
| Authored per-Place prose descriptions as content | **ADAPT** | `AuthoredPlace` should carry one; highest value per byte in their repo |
| Deriving semantic Places from scene geometry at run time | **REJECT** | §5.2 |
| Authoring Place volumes in an editor and exporting World Pack YAML | **ADAPT** | §9.6, as a `tools/` job under `ARC-10` |
| Perception via SceneTree group-name strings | **REJECT** | `INV-13`; a typo silently empties it |
| Pixel distances presented as metres | **REJECT** | `INV-14` |
| Prompt scoping that omits the other party's private state | **ADAPT** | Right instinct; ours must be structural, not assembly discipline |
| Roster constraint in the prompt ("do not invent names") | **ADAPT** | A Controller Pack prompt-policy detail worth keeping |
| Five-field character schema (`position`, `personality`, `speaking_style`, `work_duties`, `work_habits`) | **ADAPT** | §9.4. Best candidate for `AuthoredPerson`'s trait content |
| Character configuration as a `const` in a compiled script | **REJECT** | `fork source → modify game code` (`VISION.md` §3.1) |
| Identity by node name | **REJECT** | `CORE_CONCEPTS.md` §3; `AC-6` |
| Speed ramp + speed-scaled arrival threshold | **ADAPT** | Client-side navigation detail worth keeping |
| Stuck detection with bounded retries and a give-up path | **ADAPT** | Client-side; any navigation consumer needs one |
| Hand-written raycast avoidance layer | **REJECT** | `NavigationAgent2D.avoidance_enabled` exists (§10) |
| Three-mode camera with a restore key | **ADAPT** | Presentation Pack; good pattern for the 2D client |
| Click resolution order: character → interactable → ground | **ADAPT** | Presentation Pack targeting precedence |
| `AIModelLabel` — a per-Person marker of which model decides | **ADAPT** | §9.7. Cheap, high diagnostic value in a mixed-control world |
| Signal-based settings singleton with per-subject override | **ADAPT** | Shape for a cognition profile |
| One settings dictionary conflating credentials and window size | **REJECT** | `ENGINEERING_RULES.md` §16 |
| God-mode authoring surface (the capability) | **ADAPT** | §9.5 — through the action pipeline, never as direct writes |
| God-mode implemented as direct `set_meta` writes | **REJECT** | `INV-5`, `INV-6`, `INV-7`, `INV-15` |
| Soft social norms as authored, player-extensible world context | **ADAPT** | §9.2 — the most interesting idea we have no concept for |
| Separate files per persistence concern | **REFERENCE ONLY** | Modest robustness property; `DEP-2` decides our storage |
| Control binding as saved world state | **ADAPT** | Correct instinct; belongs in the World Pack / save, not in UI state |
| `has_property` guards and untyped save dictionaries | **REJECT** | §4.9; the failure is the argument for typed persistence |
| Prefab split and asset-by-kind layout | **REFERENCE ONLY** | Matches `MODULE_SPEC.md` §6.1 and `ARC-7` already |
| Everything under `asset/` | **REJECT** | §2.2 — licence not established; `DEP-8` |
| Any Microverse source file, copied | **not imported** | §2.1 — permitted by MIT, refused on architecture |

---

# 9. What Microverse has solved that MineWorld has not

Each item is a **clearly-marked proposal to be evaluated against the specifications**, not a
conclusion and not a decision. None of them changes a frozen contract; each is a gap in something not
yet designed.

## 9.1 PROPOSAL — a deterministic twin for every LM decision point

**What they have.** Fourteen `_default` functions, one beside each LM decision:
`_execute_default_decision`, `_generate_default_tasks`, `_adjust_tasks_default`,
`_continue_current_task_default`, `_rearrange_task_priorities_default`, `_add_urgent_task_default`,
`_execute_task_movement_default`, `_execute_task_conversation_default`, and the rest. An unreachable
provider, a malformed response or an unparseable answer all land on the deterministic path, and the
character keeps behaving.

**Why it matters to us.** [`VISION.md`](../VISION.md) §1.1 and
[`ENGINEERING_STANDARDS.md`](../ENGINEERING_STANDARDS.md) §§22–24 require that a running world not
depend on a model, and `AC-11` requires hundreds of simulated days with no renderer and no model. We state
the requirement; we have no shape for it. The obvious reading — "use a `RuleController` instead" —
is a *different* world, not the same world degrading. Their shape is better: the same Controller, with
a deterministic answer available at every point where it would otherwise ask.

**To evaluate.** Whether a Controller Pack should be required to declare a deterministic policy
alongside its model policy, such that a Controller is constructible with the model half absent; and
whether that requirement belongs in [`MODULE_SPEC.md`](../MODULE_SPEC.md) §5 as a hard constraint
beside the existing five. Note the interaction with `AC-12`: a deterministic fallback is seeded and
reproducible, so a run that fell back is *more* reproducible than one that did not — which suggests
the fallback path is also the right substrate for recorded-cognition regression tests
([`ARCHITECTURE.md`](../ARCHITECTURE.md) §7).

## 9.2 PROPOSAL — soft norms: authored, player-extensible normative context for Controllers

**What they have.** `BackgroundStoryManager` holds, per map, an institution name, an environment
description, a time period, a cultural context, an economic situation and a list of **social rules**
(*"keep a professional attitude during working hours"*, *"meeting rooms must be booked in advance"*).
The player can add, delete and clear **custom rules** at run time through `GodUI`, persisted to
`user://custom_social_rules.json` and keyed by map. Every prompt gets
`generate_background_prompt()` prepended.

**Why it matters to us.** MineWorld has **hard** rules — a System provides an action or it does not
exist (`INV-10`), and a `SpatialRequirement` is met or the answer is `TooFarAway`. It has **no concept
of a norm**: something that is possible but inappropriate here. An `Observation` carries affordances
(*what may be attempted*) and nothing that says *what is done around here*. Yet a living world is
mostly norms, and this is a MineWorld-shaped gap: a `Place`, an `Organization` or a World Pack could
declare the register of behaviour expected in it, and a Controller Pack could consume it.

**To evaluate, with the boundaries stated up front.** A norm must be **Controller-layer input**, never
world state a System evaluates, or it becomes a rule in the kernel (`INV-12`) or a validity check
performed outside the owning System. Candidate homes: a World Pack field consumed by a Controller
Pack's prompt policy, or an `Organization`-scoped declaration. Their *player-editable* half is the
part most worth keeping and most dangerous: a runtime-editable norm list is a delightful sandbox
feature and would have to be recorded as what it is — an authoring change, traceable like any other
(§9.5).

## 9.3 PROPOSAL — conversation as a `Process` with participants, turns and an ending

**What they have.** A `ConversationManager` instance with an id, two participants, a turn alternation,
signals for started/ended/utterance-generated, several concurrent instances, and an ending that is
itself a spoken act.

**Why it matters to us.** `ConversationSystem` provides `talk`, one instantaneous exchange, and derives
"is this a new conversation" from a `CONVERSATION_GAP` over `ConversationHistory`. That is the right
MVP-0 scope and its documentation says so. But [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) §10 lists
`talking` as a `Process` example, and a `Process` has participants (plural), `interruptibility` and an
`owning_system` that alone decides how it ends — which is exactly what a conversation needs and what
`talk` cannot express: a third participant joining, an interruption request from another System, a
turn budget, or a conversation that a client can render as ongoing.

**To evaluate.** Whether `ConversationSystem` gains a `Conversation` `Process` when S4's scheduler
lands, and whether the graceful-exit act is a second action (`leave-conversation`) or a
Controller-level convention. Their unbounded recursion is the warning: a `Process` with no budget is
an unbounded cost in a world with paid inference, which is `ARCHITECTURE.md` §9.1's concern.

## 9.4 PROPOSAL — a trait content schema for `AuthoredPerson`

**What they have.** `position` / `personality` / `speaking_style` / `work_duties` / `work_habits`,
filled in richly for eight characters, and directly responsible for the demo having a voice.

**Why it matters to us.** `AuthoredPerson` carries identity and location; `CORE_CONCEPTS.md` §4.2
names traits (`personality`, `preferences`, `skills`, `values`, `habits`) as slow-changing
characteristics owned by whichever System provides them. What no document specifies is the *content
shape* a World Pack author fills in, and a creator staring at an empty `people/alice.yaml` needs one.
`speaking_style` as a separate field is the highest-leverage single idea, because it is the field that
most changes generated output and the one authors omit when not prompted for it.

**To evaluate.** Whether these axes belong in `AuthoredPerson` as typed fields or in a Controller
Pack's own character schema — and the answer is probably *both*, split: what a Person *is* is world
data; how a Person *sounds to a model* is a Controller Pack's. Their conflation of the two is exactly
what makes `_generate_random_task` substring-match a job title. Decide the split before writing the
fields.

## 9.5 PROPOSAL — a privileged authoring path, so the god-mode need is met without breaking `INV-5`

**What they have.** A panel that implants memories, inflicts illness, grants money, sets a directed
emotion between two Persons, and edits tasks — each with a stated reason that is also written into the
subject's memory.

**Why it matters to us.** The need is real: testing a living world requires forcing states that would
take simulated weeks to reach, and `ENGINEERING_STANDARDS.md` §21 requires features to be validated by
running realistic scenarios. `tools/` is named as the home of `world-validator`, `replay` and
`inspector`, but nothing specifies *writing*. Their implementation is the canonical `INV-5`/`INV-6`
violation (§6 C-3) and cannot be imitated. But the capability must exist somewhere.

**To evaluate.** A privileged operator path that goes **through** the pipeline rather than around it:
an `ActionRequest` submitted by an operator seat, dispatched to the owning System, validated (or
deliberately exempted by a System that offers an operator action), producing an `Event` with a
`Provenance` that names the operator and a `CausedBy` that explains it. That keeps `INV-15` and makes
an operator intervention *visible in the event log*, which is strictly better than their version:
theirs is untraceable, ours would be auditable and replayable. Note the pleasing consequence: an
implanted memory under `INV-4` is a legitimate divergence between belief and truth, and with a logged
cause it is a *reproducible* one.

## 9.6 PROPOSAL — derive draft World Pack content from an authored scene, in `tools/`

Covered in §5.2. Editor-authored Place volumes exported as reviewable `places/<key>.yaml`, under
`ARC-10`'s rule that generation is development tooling and never a runtime dependency. It is the way
to keep their authoring ergonomics with our arrow.

## 9.7 SMALL, CHEAP, DO IT WHEN THE CLIENTS EXIST

- **A per-Person controller marker in both reference clients** (`AIModelLabel`'s equivalent): which
  Controller, which model, toggleable. In a world with mixed control this is basic legibility.
- **A display-settings block** modelled on `apply_display_settings` — window mode, resolution, content
  scale — which is correct Godot and saves an hour.
- **`NavigationAgent2D` with `avoidance_enabled` from the start** in the 2D client, rather than raw
  `NavigationServer2D` queries (§10).

---

# 10. Mature Godot features and libraries this audit surfaced

`DEP-9` hands GPU, physics, animation, camera, input, navigation, audio and platform export to Godot,
and MineWorld writes none of it. Microverse is useful evidence about *which* Godot features get
re-implemented by hand when a project does not go looking for them.

| Feature | What Microverse hand-wrote instead | Where it matters for us |
| --- | --- | --- |
| `NavigationAgent2D` / `NavigationAgent3D`, with `avoidance_enabled`, `radius`, `max_speed`, `neighbor_distance`, `velocity_computed` | ~120 lines of raycast avoidance, oscillation damping via a 0.8 s direction hold, stuck detection, and jittered path recalculation, on top of raw `NavigationServer2D.query_path` | The 2D reference client, from its first commit. `REUSE_POLICY.md` §4 — basic navigation is commodity |
| `NavigationRegion2D` / `3D` baked in the scene | Used correctly (the scene has one) | Walkable surface is a Presentation Pack concern; `ENGINEERING_RULES.md` §12 keeps navmeshes out of contracts |
| `Node2D` / `CanvasItem` `y_sort_enabled` | `Desk.gd` recomputes `z_index` on a 0.1 s `Timer` by scanning every character and chair | 2D client depth sorting |
| `Camera2D.position_smoothing_enabled` / `position_smoothing_speed`, limit rect | Manual `lerp` in `_process` | 2D client camera |
| `RemoteTransform2D`, or a `Control` anchored to a body, for world-space UI | `DialogBubble` repositions itself every frame in `_process`, is parented to the scene root, and is never freed | Speech bubbles and nameplates in the 2D client |
| `Resource` + `@export` + `.tres` for designer-edited data | A `const Dictionary` in a script | Presentation Pack per-character presentation data — never simulation data |
| `Area2D` `body_entered` / `body_exited` | Used correctly in `Chair.gd` | The right way for a client to *notice* a volume entry and submit an `ActionRequest` — never to decide the fact |
| `Window.mode`, `content_scale_mode`, `DisplayServer.screen_get_size` | Used correctly in `SettingsManager.apply_display_settings` | Both reference clients' display settings |
| `Object.has_method` / the `in` operator for property checks | `Object.has_property`, which does not exist (Appendix A) | A reminder that GDScript has no compiler to catch this, which is why authoritative logic is Rust (`DEP-9`) |

Non-Godot libraries surfaced: maintained model clients (`openai`, `anthropic`, `ollama`,
`google-genai`) and provider routers for `cognition/` (§5.1); an embedding/vector index if and when a
memory policy needs retrieval (§5.3 row 8). Each requires a
[`REUSE_POLICY.md`](../REUSE_POLICY.md) §11 or §12 record in
[`DECISIONS.md`](../DECISIONS.md) before adoption or rejection.

---

# 11. The short list: what must never enter MineWorld

For a reviewer who reads only one section.

1. **Semantic state on renderer nodes.** No world fact in Godot node metadata, node properties, or
   anything else that requires an engine to exist. `INV-5`, `INV-14`.
2. **More than one writer of a fact.** One owning System, `WriteToken<S>`, and cross-system change by
   `Event`. `INV-7`.
3. **A UI or a Controller that writes state.** `ClientFrame` has `Join` and `Submit` and there is no
   third. `INV-5`, `INV-6`.
4. **Perception by scene traversal, group name, or any widenable handle.** `Observation` is a value.
   `INV-13`.
5. **Semantic `Place` derived from render geometry**, and render units used as semantic units.
   `INV-5`, `INV-14`, §5.2.
6. **Wall-clock time anywhere in world semantics.** `WorldTime` and `SimDuration`. `AC-12`.
7. **Identity by name.** `EntityId`, allocated, monotonic, never reused. `AC-6`.
8. **Memory as the world's history.** `INV-4`, `INV-11`.
9. **A provider, a prompt, or a transport named in a contract, a `Component`, an `Action` or an
   `Event`.** `INV-12`, `INV-14`.
10. **A unit that would change when an `Item` type, a provider, a renderer, a memory schema *or* an
    interaction is added.** §7's review test.
11. **Any file under `asset/`.** Licence not established; `DEP-8` §2.2.

---

# Appendix A — behavioural probes

Two claims in this audit are behavioural rather than textual, so both were checked rather than
asserted. Godot `4.7.2.stable.official.ed1daf0bf`, headless, macOS arm64. (Microverse targets Godot
4.3+; `Object.has_property` has never existed in the Godot 4 `Object` API, and 4.7 is the version
available here.)

**A.1 — `Object.has_property` does not exist.**

```text
script:  var n := Node.new(); print(n.has_method("has_property"))
output:  has_property is a method: false
```

**A.2 — calling it raises and aborts the enclosing function.**

```text
script:  print("before"); var r = n.has_property("name"); print("after: ", r)
output:  before
         SCRIPT ERROR: Invalid call. Nonexistent function 'has_property' in base 'Node'.
                   at: _init (res://probe2.gd:5)
```

`"after"` is never printed, so execution leaves the function at the failed call. Applied to
`GameSaveManager.collect_character_data`, whose first `has_property` call
(`GameSaveManager.gd:144`) is reached unconditionally once a controller node is found: the function
returns `null`, `collect_game_data`'s `if character_data:` skips it, and a save written by this
snapshot contains `"characters": []`. This is reported in §4.9 as evidence for typed persistence
boundaries, not as a quality judgement on a demo.

**A.3 — group membership, established by reading rather than running.** `add_to_group` appears three
times in the repository: `chairs` (`Chair.gd:11`), `controllable_characters`
(`CharacterController.gd:30`) and `api_manager` (`APIManager.gd:25`). The scene files declare
`characters` (2), `room_area` (9) and `godui` (1). The groups `interactable` (read at
`AIAgent.gd:139`), `character` (read at `AIAgent.gd:150`, `:1853` and three times in
`GameSaveManager.gd`) and `npc` (read at `DialogManager.gd:156`) are therefore never populated, and
those reads return empty arrays. §4.6 and §6 C-7 rely on this.

---

# Appendix B — one-line summary of each audited unit

| File | Bytes | What it is | Verdict |
| --- | --- | --- | --- |
| `script/ai/AIAgent.gd` | 90,904 | perception + prompts + HTTP + decisions + tasks + movement + social, per character | §7 case study. **REJECT** as a shape |
| `script/ui/GodUI.gd` | 39,242 | god-mode authoring panel that writes state directly | capability **ADAPT** (§9.5); implementation **REJECT** |
| `script/CharacterController.gd` | 17,765 | `CharacterBody2D`: nav path following, hand-rolled avoidance, sit, human/AI gate | navigation details **ADAPT**; avoidance **REJECT** (§10) |
| `script/ai/ConversationManager.gd` | 12,034 | one conversation: prompt, model call, bubble, history, turn swap | lifecycle **ADAPT** (§9.3); recursion and dyad **REJECT** |
| `script/ai/DialogManager.gd` | 11,829 | autoload: input, conversation entry points, a second memory writer | **REJECT** — third memory schema, `INV-7` |
| `script/GameSaveManager.gd` | 10,735 | JSON save/load keyed by node name | **REJECT**; §4.9 is the lesson |
| `script/ai/background_story/BackgroundStoryManager.gd` | 9,930 | per-map world bible + preset and custom social rules | **ADAPT** (§9.2) — the most interesting gap it reveals |
| `script/ai/APIConfig.gd` | 8,561 | nine providers behind one description | pattern **ADAPT** (§5.1); `-> String` **REJECT** |
| `script/ui/CharacterAISettings.gd` | 6,976 | per-character provider/model panel | per-subject override **ADAPT** |
| `script/ui/GlobalSettingsUI.gd` | 6,964 | tabbed settings | **REFERENCE ONLY** |
| `script/CharacterPersonality.gd` | 6,832 | eight characters as prose in a `const` | schema **ADAPT** (§9.4); storage **REJECT** |
| `script/ui/SettingsManager.gd` | 6,487 | autoload settings, signal-based, per-character overrides | shape **ADAPT**; conflated dictionary and plaintext keys **REJECT** |
| `script/CharacterManager.gd` | 5,676 | click selection, camera handoff, GodUI sync | click precedence **ADAPT** |
| `script/ai/memory/MemoryManager.gd` | 5,468 | categorised, importance-weighted memory with pruning | heuristics **ADAPT** (§5.3); representation **REJECT** |
| `script/ai/DialogService.gd` | 5,094 | registry of active conversations | instance-with-lifecycle **ADAPT** |
| `script/ChatHistory.gd` | 4,278 | per-character JSON transcript, rewritten per message | **REJECT**; identity by node name |
| `script/CameraController.gd` | 3,820 | three-mode 2D camera | **ADAPT** for the 2D client |
| `script/ui/AIModelLabel.gd` | 3,419 | per-character model nameplate | **ADAPT** (§9.7) |
| `script/ai/APIManager.gd` | 3,339 | autoload: builds and fires one `HTTPRequest` per call | **REFERENCE ONLY** |
| `script/Chair.gd` | 3,207 | occupancy + sit anchor + z-order, and it moves the Person | **REJECT**; §6 C-4, C-5 |
| `script/ui/DialogBubble.gd` | 2,200 | speech bubble following a target, never freed | **REFERENCE ONLY**; §10 |
| `script/RoomManager.gd` | 1,924 | derives rooms from `Area2D` volumes | **REJECT** at run time; **ADAPT** as tooling (§5.2) |
| `script/Desk.gd` | 1,314 | z-order by timer | **REJECT**; `y_sort_enabled` exists |
| `script/RoomData.gd` / `RoomArea.gd` | 457 | room value object and its authored strings | authored descriptions **ADAPT** |
| `scene/maps/Office.tscn` | 17,052 lines | the world, as a scene | **REJECT** as a world format; §4.12 |
| `asset/**` | 4,305 files | LimeZu art, licence not established | **REJECT**; §2.2 |

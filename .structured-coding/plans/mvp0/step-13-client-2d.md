# Step S12 — Demo A: the 2D reference client

**Effort:** `mvp0` · **Parent:** [`overall.md`](overall.md) §3 S12, §4, §7
**Lifecycle:** `DRAFT — awaiting the primary session's review`
**Planning base:** `main` @ `0fd0be3`, branch `plan/s12-2d`, worktree
`/Users/yuema137/mineworld-worktrees/plan-s12-2d`
**Planned in parallel with:** S11 (server and protocol), S13, S14, Milestone E. This document writes
no other file. Every proposed edit to `overall.md`, `MVP_STATUS.md`, `DECISIONS.md` or a specification
is in §13. New decision records carry placeholder identifiers (`ARC-S12-a`, `DEP-S12-a`); the primary
session assigns real numbers at freeze.
**Binding directive (operator, 2026-10-08, relayed during planning):** keep the code clean, modular and
pluggable; look for open-source solutions first and compare several before choosing; build our own only
when nothing existing serves the goal. §5 (reuse) and §4.2 (modularity) answer it.

Phase: **step planning**. Nothing here authorizes implementation. Each PR in §9 is detailed to the
commit and frozen in turn, in a fresh session, as the workflow requires.

---

## 1. The requirement and its acceptance, quoted

### 1.1 The step, from `overall.md` §3

> **S12 — Demo A: 2D reference client**
>
> - **Output:** the reference top-down 2D client, reading observations and submitting intents. It
>   is a permanent integration testbed, not a mock UI to be discarded once 3D exists
>   (`ENGINEERING_RULES.md` §2).
> - **Depends on:** S11.
> - **Acceptance checkpoint:** the client renders the world and drives a human-controlled Person;
>   killing the client leaves the simulation running (`AC-3` end to end); no simulation contract
>   changed to accommodate the renderer (`INV-5`, `INV-14`); it demonstrates the Demo A list in
>   `docs/MVP.md` §7.1 — multiple Persons, places, movement, conversation, relationships, basic
>   items, basic group activity, persistence, human and agent controllers, multiplayer.

### 1.2 Demo A, from `docs/MVP.md` §7.1 and `ENGINEERING_RULES.md` §10

> A simple Godot top-down client. Its purpose is speed: it is the cheapest place to validate world
> state, movement, interaction, multiplayer, persistence, agent behavior, and system composition.
> It is a permanent integration testbed, **not** a temporary mock UI to be discarded once 3D
> exists.
>
> Must demonstrate: multiple Persons, Places, movement, conversation, relationships, basic items,
> basic group activity, persistence, human and agent controllers, multiplayer connectivity.

`MVP.md` §7.4 names the artefact: `mineworld-2d`, "a real program you open and play: walk, see NPCs,
click a person, talk".

### 1.3 The acceptance criteria this step carries

| ID | Text (`docs/MVP.md` §9) | What S12 owes |
| --- | --- | --- |
| **AC-3** | The simulation continues correctly while the Godot client is disconnected, and a client may reconnect to the running world. | End to end, with the real 2D client killed and relaunched (overall §4: "S11, end-to-end in S12"). |
| **AC-13** | `Talk` initiated by clicking an NPC in Demo A and by approaching, looking at, and pressing interact in Demo B produce an **identical semantic core** — `actor`, `action_type`, `target`, payload — resolved by the same system to the same result. Neither client implements any validity rule. | S14 owns the test; S12 delivers the 2D half, and must record its submitted requests in the format `tools/cli/tests/ac13_semantic_parity.rs` already reads. |
| **AC-15** | A 2D client, a 3D client and an agent-driven Person are connected to **one running server** at the same time … | S14 owns it "against S11's server with S12's client also connected" (overall §4). S12's client must be that client. |

### 1.4 The invariants named by the checkpoint (`docs/CORE_CONCEPTS.md` §2)

> **INV-5** The renderer does not own world state. Presentation observes; it never mutates.
>
> **INV-14** Simulation semantics never depend on the renderer, the transport, the persistence
> backend, the deployment target, or the model provider.

And the rules the client is built under (`ENGINEERING_RULES.md`):

> §2 — [The 2D client] is an architectural and integration testbed, not merely a temporary mock UI.
>
> §8 — The client must not directly start or modify semantic world processes. The same rule applies
> to the 2D client.
>
> §9 — Both must ultimately produce the same semantic ActionIntent. … If the 2D and 3D clients
> require separate implementations of business rules, the architecture is wrong.
>
> §19 — For rendering-related work, actually run the relevant reference client.
>
> §22 — Can the same semantic capability be used by both 2D and 3D clients without duplicating game
> logic?

---

## 2. Audit (`main` @ `0fd0be3`, 2026-10-08)

Every finding was read in this session from the file named. Nothing about a 2D client is assumed.

| ID | Finding | Evidence |
| --- | --- | --- |
| **A-1** | **No 2D client exists on `main`.** `clients/` holds `3d-spike/` and `protocol/` only. The "2D client" is `clients/2d-spike/` on the unmerged branch `vis/2d-generated-assets`, cut from `cc40ad5` (2026-09-27), 23 commits ahead and 355 behind `main`. Its diff against the merge base is purely additive: 598 files, 18 778 insertions, 0 deletions — `clients/2d-spike/**`, the `mineworld-2d` launcher, `docs/DECISIONS.md` (`ARC-14`, +55 lines) and `.structured-coding/plans/vis-2d/handoff.md`. | `git diff --stat $(git merge-base origin/main origin/vis/2d-generated-assets) origin/vis/2d-generated-assets` |
| **A-2** | **The spike is presentation only, and says so.** Its README: "It requires no kernel changes, contains no semantics — no dialogue, no inventory, no networking, no `ActionIntent` — and invents no world rules." It does not use `clients/protocol`. It lays out one invented quayside square (four shopfronts, a quay, a park corner), five background walkers on fixed routes and a dog, and a player on WASD. | `clients/2d-spike/README.md`, `scripts/Main.gd` (branch) |
| **A-3** | **The spike's walkable space is a client rule.** `Main.gd` hard-codes `PLAZA_WALK`, `ROOM` and `DOOR` rectangles and slides the player along their edges. Acceptable in a spike; in the reference client it would be a world rule in a renderer (`ENGINEERING_RULES.md` §8). | `Main.gd` lines ~66–75 (branch) |
| **A-4** | **The spike's projection is 2:1 isometric, not top-down.** `Iso.gd`: `TILE_W 128`, `TILE_H 64`, 2 m per world unit, 32 px per metre; the manifest says `projection: isometric`. `MineWorldSpace.to_2d` is a top-down (north-up) conversion. The accepted look therefore needs one more presentation projection on top of the shared axis conversion. `MVP.md` §7.1 says "top-down"; the operator accepted an isometric look (`ARC-14`). | `scripts/Iso.gd` (branch); `presentation/mineworld-default/2D/manifest.yaml`; `clients/protocol/mineworld/space.gd` |
| **A-5** | **The four style variants** are `town` (accepted default, `ARC-14`), `full`, `people`, `procedural`, selected by `--variant=`. A variant is a role map: scene roles (`shop_cafe`, `npc_b`, `player`) resolve to sprites. `ARC-14` keeps all four "as the demonstration that the presentation layer swaps", with `--variant=` "the interface". | `Main.gd` `VARIANTS`, `_role`, `ROLE_PEOPLE`; `ARC-14` (branch `docs/DECISIONS.md:929`) |
| **A-6** | **The spike's art is 101 MB**, of which `art/candidates/` is 81.7 MB (rejected or unchosen candidates), `art/ground/` 7.7 MB, `art/generated/` 5.7 MB, `art/svg/` 2.4 MB, `art/style_refs/` 2.0 MB, `art/interior/` 1.1 MB; `screenshots/` another 8.9 MB. | `git ls-tree -r -l` on the branch |
| **A-7** | **The spike's motion is sound and measured.** `--drive` runs headless (~6 s) and asserts a per-frame step bound, a clamped simulation delta (`MAX_SIM_DT`), entry through the café door, and gait driven by distance (`STRIDE_M` 0.72 m) rather than by the clock. Capture is behind `--capture` because `frame_post_draw` never arrives headless. | `clients/2d-spike/README.md` "Motion" (branch) |
| **A-8** | **The spike's interior is a cut-away in one coordinate space**: the café room lies behind its façade, the façade fades out once the player crosses the threshold, and nothing loads. This matches the world model exactly: the café is a `Place` with its own frame, joined to the street by a passage. | README "Going inside"; `scripts/Interior.gd` (branch) |
| **A-9** | **How people are labelled.** The spike labels nobody (it has no world). The protocol demo labels people by `MineWorldObservation.display_name(id)` (naming's `display-name`, `ARC-31`) and falls back to the id, "never a key". The 3D slice does the same through `display_label`. | `clients/protocol/demo/demo.gd` `_called`; `clients/3d-spike/scripts/slice/slice_link.gd:221` |
| **A-10** | **The shared module.** `clients/protocol/mineworld/` = `world_client.gd` (310 lines: join, observe, submit, refuse, disconnect; `PROTOCOL := 1`), `observation.gd` (202: a reader; `may`, `affordance`, `unavailable_reason`, `requirement`, `component`, `own_component`, `display_name`, `events`), `space.gd` (150: the one axis conversion). The 3D client adopts it **by symlink** (`clients/3d-spike/mineworld -> ../protocol/mineworld`), so any edit is live in 3D at once. | `ls -la clients/3d-spike/mineworld`; the three files |
| **A-11** | **Complete affordances are carried and unused.** `observation.gd` has no accessor for `payload`; `affordance(type, target)` returns the first match, which is ambiguous when several complete affordances share a type and target — the normal case (QS-20 in step-10). `ADOPTION.md` §2 documents submitting one through `submit(type, target, payload)`; "the first client use is S12's". | `observation.gd:154`; `ADOPTION.md` §2; step-10 QS-20 |
| **A-12** | **`submit`'s payload is typed `Dictionary`.** Every payload today is a JSON object, so this holds; a complete affordance whose pack encodes a non-object payload could not be submitted unchanged. | `world_client.gd:155–175` |
| **A-13** | **The module drops the frame's `revision`.** `_observation` keeps `seq` and the inner `observation`; the frame-level `revision` (`PROTOCOL.md` §5, `AC-15`'s fourth evidence line) is discarded. `world["revision"]` is only the welcome's. | `world_client.gd:275–288` |
| **A-14** | **The module has no reconnect.** `ADOPTION.md` §6: "a dropped connection ends; retrying is the client's own policy". `connect_to_world` accepts a call from `CLOSED`, so a client can reconnect with the existing API. | `ADOPTION.md` §6; `world_client.gd:116–130` |
| **A-15** | **Evidence format for `AC-13`.** `demo.gd --flavour 2d --requests <file>` writes `[{token, flavour, request}]`; `tools/cli/tests/ac13_semantic_parity.rs` reads `clients/protocol/evidence/request-2d.json` and `request-3d.json`. The 2D flavour sends `actor_location: null`. | `demo.gd` `_on_submitted`, `_report`; `clients/protocol/README.md` |
| **A-16** | **Protocol rev 1 (`server/PROTOCOL.md`).** Two client frames (`join {seat}`, `submit`); server frames `welcome`, `observation {seq, revision, observation}`, `result`, `refused`. §9 "not in revision 1": authentication (seat name only), scoping, state deltas, admin commands, binary encodings. Observation cadence 10 Hz. | `server/PROTOCOL.md` §§2–9 |
| **A-17** | **Seats are not exclusive.** `WorldRuntime::join` checks only that the seat is in the roster and adds a subscriber; two connections — or a connection and an `--agent` — may hold one seat and both act. So reconnecting is trivial today, and `AC-5`'s takeover has no mechanism yet (S11's). | `server/src/runtime.rs:255–280` |
| **A-18** | **A hosted world's NPCs do not take initiative.** `mineworld server --agent SEAT` drives a seat with `RuleController`, which only answers. `PacedRuleController` (the one that walks, invites, buys) runs only in `mineworld run`. Connected to `mineworld server worlds/market-town`, a 2D client sees eleven people standing still unless addressed. | `tools/cli/src/agent.rs:40–80`; `cognition/rule-controller/src/{lib,paced}.rs`; overall §7 S7 ("no system acts alone and RuleController only answers") |
| **A-19** | **The hosted clock is wall-clock 1:1.** `HostClock::now` = epoch + elapsed real seconds; no time scale. A market-town day takes 24 real hours; shifts and routines are authored in ≥ 4 h parts (L-12). | `server/src/runtime.rs:64–81`; `HostConfig` in `host.rs:180–209` |
| **A-20** | **What an observation lists.** The observer's place, then everybody in it, with tags, location and the components packs chose to disclose; relations; affordances; `events` empty. Other places are not perceived. | `systems/presence/src/observe.rs` (module doc, `present_with`, `perceived`) |
| **A-21** | **What packs disclose today**, by subject: `passages` on the observer's place (movement — `leads_to [{to, here, there}]`); `place-shape` on the place (bodies, floor rectangle and solids, only where a place has `body:`); `shop` listing on a place with a shop (economy); `display-name` and `participation` about every present person (naming, group-activity); to the observer only: `conversation-history`, `acquaintances`, `invitations`, `holdings`, `wallet`, `employment`, `agenda`. | each pack's `discloses` in `systems/*/src/system.rs`, `group-activity/src/perception.rs` |
| **A-22** | **What packs offer today**, and whether completely: `move` (movement), `talk` (conversation, payload `{utterance}`), `invite` (`{kind}`, a free slug), `accept-invitation`, `decline-invitation`, `join-group-activity`, `leave-group-activity` (empty payloads) — none complete. **Complete:** `give {item, count: 1}` per kind held (item-transfer), `buy {item}` per priced kind (economy), `eat`/`drink {item}` per food/drink held (consumption). | `Offer::new` / `Offer::complete` in `systems/*/src/{offer,perception,system}.rs` |
| **A-23** | **A passage does not say what it leads to.** `leads_to[].to` is `{entity, entity_type}`; the destination place is not perceived, so its tags are unknown until one walks in. From the street a client sees five doorways and cannot tell which is the café. The 3D slice avoids this by binding only two places it already knows by key. | `systems/movement/src/component.rs` (`Passages`); `slice_link.gd` `KEYS`, `_learn_passages` |
| **A-24** | **Item kinds have no names (F-41).** Items are never perceived; holdings, listings and complete payloads carry item ids. Item files carry `tags` (e.g. `coffee.yaml`: `[drink, hot]`) and `item: {category}`, no name. Organizations likewise. Recorded "for S12". | step-10 F-41; `worlds/market-town/items/*.yaml`; overall §7 |
| **A-25** | **AC-1 check 3 constrains how F-41 can be fixed.** "Market Town's files are Social Café's plus sections the six packs own." A `name:` section on item files owned by `naming` (not one of the six) would fail it; a name field inside `item:` (owned by `item`, one of the six) would not. | `tests/acceptance/tests/ac1_composability.rs`; overall §7 S9 11f |
| **A-26** | **S15 12c (frozen, not merged)** gives objects and three actions: loose objects disclosed **on the place** as `loose-objects` `[{object, shape, at}]` (SD-O2); `kick {object}` and `throw {object, toward: null}` as **target-less complete affordances** per object within reach; `throw {object, toward: {x,y}}` as an aimed, non-complete request; `shove {}` as a complete affordance against a present person; facts `object-moved {…, path}` and `person-shoved`. Only `worlds/bodies-yard` installs bodies until **12d** gives the towns walls and objects. step-11 §7.3: "The 2D client (S12) needs no physics … sends `kick`, `throw` and `shove`". | step-11 §7.2–7.3, §18 SD-O2, SD-O11–SD-O15, F-O1, F-O12 |
| **A-27** | **Object motion arrives as an event.** `object-moved` carries a `path` for kicks and throws; step-11 §7.1 rule 5 asks clients to animate along it. With `events` empty in rev 1, a client sees only the next resting position. | step-11 §7.1; `PROTOCOL.md` §9; A-20 |
| **A-28** | **The Presentation Pack for 2D is a skeleton.** `presentation/mineworld-default/2D/` has `manifest.yaml`, `ART_DIRECTION.md`, `references/` and `candidates/`; no `assets/asset_bindings.yaml`, no `renderer/godot.yaml`. `MODULE_SPEC.md` §6.1 names those files and specifies none of their contents. | `ls presentation/mineworld-default/2D`; `MODULE_SPEC.md` §6, §6.1 |
| **A-29** | **The 3D slice's spatial binding is the precedent.** "World Packs author no geometry, but the world states its doorways … a binding may only translate. Each place is bound so that its disclosed doorway lands on this slice's café door." It submits regardless of `may()` and shows the server's verdict. | `slice_link.gd` header, `origin`, `talk_to_facing` |
| **A-30** | **Market Town's layout, as data.** Every place has one passage, onto the street. Street frame: north-side doorways at `y = 3 000` (café `x 0`, apartments `x −12 000`, store `x +12 000`), south side at `y = −13 000` (office `x −6 000`, park gate `x +8 000`). Café frame: origin at the room's inner south-west corner, 8.32 × 10.32 m, its door `here (1 610, 200)` / `there (0, 3 000)`. | `worlds/market-town/places/{street,cafe}.yaml` |

---

## 3. Demo A: what works and what is missing

"Works" means it runs today through the real server and the protocol; "client" means the reference 2D
client, which does not exist on `main` (A-1).

| Demo A item | Works today | Missing |
| --- | --- | --- |
| **Multiple Persons** | Observations list everybody in the observer's place with tags, position, facing, `display-name`, `participation` (A-20, A-21). The protocol demo draws them as dots. | The client. Distinct appearance per person chosen from world data (tags, id), labelled by name. |
| **Places** | Six places in market-town, joined by passages; `passages` disclosed; `place-shape` where bodies is installed. | The client's binding of place frames into one drawn town (§4.3). **A-23:** a doorway does not say where it leads, so façades cannot be chosen before entering — **R-PK-1**. Walls in the towns arrive with S15 12d. |
| **Movement** | `move` strides ≤ 2 m, doorway crossings, `too_far_away`, persisted (S6). demo.gd walks in strides. | Click-to-move and WASD in the client, routing through doorways, reconciliation (§4.5). |
| **Conversation** | `talk` with `{utterance}`; the observer's own `conversation-history` (what was said to me). `--agent alice` answers. | The client's talk input and history panel. **Conversations between NPCs are invisible**: `events` is empty in rev 1 — **R-S11-4**. NPCs only answer (A-18) — **R-S10-1**. |
| **Relationships** | `acquaintances` disclosed to the observer (ARC-28). | A panel. Nothing about others' relationships is disclosed, correctly (`INV-13`). |
| **Basic items** | market-town: `give`, `buy`, `eat`, `drink` as complete affordances; `holdings`, `wallet`, `shop` disclosed. After 12c/12d: loose objects, `kick`, `throw`, `shove`. | The client's generic complete-affordance menu (§4.4). **F-41: no names for item kinds** — **R-PK-2**, PR 13c. Objects need 12c (bodies-yard) and 12d (towns). |
| **Basic group activity** | `invite {kind}`, accept, decline, join, leave; `invitations` (own) and `participation` (everyone present) disclosed. | The client's menu and indicators. `invite`'s `kind` is free player input (QS12-6). |
| **Persistence** | `mineworld server --save` resumes the same instance and revision (S5); frames carry `revision`. | The module drops the frame's revision (A-13, **M-3**); a client restart/server restart scenario through the client (PR 13f). |
| **Human and agent controllers** | A human seat via the protocol; `--agent SEAT` via the same authority path (INV-1). | Hosted agents that take initiative (**R-S10-1**); human takeover of an agent-driven NPC with handover (**R-S11-3**, `AC-5`). |
| **Multiplayer connectivity** | Several connections to one world; `AC-15` holds with the protocol demo. | Authentication (invite token + nickname, **R-S11-1**); seat exclusivity (**R-S11-2**); the client's join screen and seat picker. |
| **`AC-3` end to end** | Server-side: the world runs with nobody connected; a client can reconnect (non-exclusive seats, A-17). | Kill the real 2D client and relaunch it, asserted (PR 13a); a reconnect policy in the client (§4.6). |
| **Living-world pace** | — | Hosted time runs 1:1 (A-19); a day is 24 h — **R-S11-6**. |

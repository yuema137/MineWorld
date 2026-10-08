# Step S12 — Demo A: the 2D reference client

**Effort:** `mvp0` · **Parent:** [`overall.md`](overall.md) §3 S12, §4, §7
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)` — frozen at step level by the primary session under the operator decisions and coordination rulings in `overall.md` "Parallel build-out, 2026-10-08", which bind and override this document where they differ (decision numbers, protocol ownership, event perception, the shared module, digests). Superseded wording below: `DRAFT — awaiting the primary session's review`
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

**Summary.** The server side of Demo A is largely there; the client is not. Of the ten items, every one
needs the client; four need something outside it — names for doorways and item kinds (packs), events
in observations, hosted agents with initiative, authentication with seat exclusivity (S11 / S10).
None needs a kernel or contract change.

---

## 4. Design

### 4.1 Client architecture

Three layers, each removable without editing the others, and one composition root.

```text
                    ┌──────────────────────────────────────────────────────────────┐
  shared with 3D    │ clients/protocol/mineworld/   world_client · observation ·   │
  (symlinked in)    │                               space   (§7 lists S12's edits)  │
                    └──────────────────────────────▲───────────────────────────────┘
                                                   │ observations in, requests out
                    ┌──────────────────────────────┴───────────────────────────────┐
  2D client core    │ clients/2d/scripts/                                          │
  (no art, no rule) │   link.gd      connection, seat, reconnect policy            │
                    │   town.gd      place frames glued at passages (§4.3)         │
                    │   projection.gd world metres ↔ screen (iso or plan)         │
                    │   scene/*.gd   reconcile people, places, objects, labels     │
                    │   walker.gd    click / WASD → move strides, reconciliation   │
                    │   intents.gd   THE ONLY file that names an action type       │
                    │   menu.gd      choices built from affordances               │
                    │   hud/*.gd     panels from the observer's own disclosures   │
                    │   harness/     scripted drive, evidence, capture             │
                    └──────────────────────────────▲───────────────────────────────┘
                                                   │ roles → art, wording, scale
                    ┌──────────────────────────────┴───────────────────────────────┐
  Presentation Pack │ presentation/mineworld-default/2D/                            │
  (swappable)       │   manifest.yaml · assets/asset_bindings.yaml ·               │
                    │   renderer/godot.yaml · art/ (the accepted `town` set)       │
                    └──────────────────────────────────────────────────────────────┘
  composition root  clients/2d/scripts/app.gd: arguments, wiring; ./mineworld-2d launches it
```

- **The core never names a sprite, a texture or an art file.** It asks the presentation for a *role*:
  `person`, `place:<tag>`, `facade:<tag>`, `solid`, `object:<shape>`, `ground:<tag>`. The pack
  resolves roles to art; a role it does not bind falls back to the core's plain drawing (shapes and
  text, as `demo.gd` draws). So the client runs, and is fully testable, with **no** pack.
- **The core never decides validity.** `intents.gd` holds, per action type the client knows how to
  *ask for*, a composer (what the payload is made of: a clicked point, a typed utterance, a chosen
  kind) and nothing else. Whether the action exists, is available, or is close enough is read from the
  affordance; whether it succeeded is read from the result (§4.4, §8).
- **Godot-native, GDScript only.** No GDExtension, no addon is required to run the core (§5).
- **Size.** The spike's `Main.gd` is 1 217 lines; the standards question a file past ~500. The split
  above keeps each file to one concept; the layout is the PR's to fix after audit, not this table's.

### 4.2 Modularity and pluggability

Terminology, fixed by `MODULE_SPEC.md` §6 and `MVP.md` §7.4: the **2D client** is an *application*
(the program `mineworld-2d`); the **Presentation Pack** `presentation/mineworld-default/2D` is the
*pack* that says how the world looks in it. The client is not itself a pack type, and this step does
not make it one. What the operator's directive asks — removable and pluggable — holds at both levels:

| Claim | How it holds | How it is checked |
| --- | --- | --- |
| **The client is removable.** Deleting `clients/2d/` changes no system, no contract, no world and no test outside the client's own. | Nothing under `kernel/`, `contracts/`, `systems/`, `server/`, `worldpack/`, `persistence/`, `cognition/` names it. Its tests live with it or in a client-scoped test file. | I-4 path scan (§8): the client PRs' merge diffs touch only `clients/2d/**`, `clients/protocol/**` (listed edits), `presentation/mineworld-default/2D/**`, `mineworld-2d`, client-scoped tests and Markdown. |
| **The Presentation Pack is swappable.** `--presentation <dir>` selects a pack; none means plain drawing. | The core addresses art only through roles (§4.1). | Run the scripted drive three ways — default pack, `--variant` bindings, no pack — and compare the **submitted request transcripts**: identical (I-5). |
| **The `ARC-14` variants survive as bindings, not code.** `town` is the default; `full`, `people`, `procedural` are alternative binding sets in the same pack (or sibling packs, QS12-4). | A variant is `asset_bindings.yaml` content. | A change that can only be made to `town` is a change in the wrong place (`ARC-14`): the drive passes for every variant. |
| **Packs the client was never written against still work.** A pack that offers a complete affordance appears in the menu and can be used. | §4.4: complete affordances are submitted unchanged by a generic path. | I-3 adversarial run with `tests/acceptance`'s synthetic `ring` pack (§8.2). |
| **Removing a System Pack removes its UI.** A world without `item-transfer` shows no `give`; without `economy`, no wallet panel. | Every menu entry is an affordance; every panel is a disclosed component; an absent one is not drawn. | Run against market-town with a pack removed (AC-2 seen through the client). |

**Shared with 3D, through `clients/protocol/mineworld/`** — and only through it: the connection and
its frames, the seat, the observation reader, complete-affordance submission, the axis conversion
(`MineWorldSpace`), the id-is-a-string and integer rules, revision tracking. Both clients submit the
same requests for the same intent (`AC-13`).

**2D-only** — and deliberately not shared, because each is acquisition or appearance: the isometric
projection, click picking and the floor click, click-to-move route planning, the cut-away interior,
the sprite and label layer, the menu and HUD layout, the Presentation Pack bindings for 2D. If a
piece of this turns out to be needed by 3D with identical meaning (the place-frame gluing of §4.3 is
the one candidate), it moves into the shared module by a listed change, never by copy (QS12-9).

### 4.3 Rendering semantic space

The world gives each place its own frame and authors no geometry beyond positions, passages and —
where `bodies` is installed — a floor and solids (A-20, A-21, A-30). The client turns that into one
drawn town without inventing a fact.

1. **Place frames are glued at their passages, by translation only** (`ARC-S12-a`, the 3D slice's rule,
   A-29). The street is the root frame of the drawing. A place `P` whose passage to the street has
   `here = h` (in `P`) and `there = t` (in the street) is drawn with its origin at `t − h` plus the
   pack's *doorstep offset* (the art's wall thickness, a presentation constant). The frame is never
   rotated or scaled: `+x` east, `+y` north everywhere (`CORE_CONCEPTS.md` §6.1). Gluing uses only
   disclosed passages; from the street all five are disclosed, from inside a place only its own.
2. **The layout is learned, not authored in the client.** The client keeps a per-world-instance cache
   of passages it has been shown (presentation memory, like a map the player has drawn). It holds no
   world truth: it is dropped when the instance changes, and nothing is submitted from it except
   positions the player chose.
3. **What a place looks like** is chosen by the presentation from the place's **tags** — world data —
   once known: `facade:cafe`, `ground:street`, `interior:cafe`. Before a neighbouring place's tags are
   known, it is drawn as a generic doorway (A-23). With **R-PK-1** the passage itself names the
   destination's tags and the town is drawn complete from the street.
4. **Extent.** Where a `place-shape` is disclosed, the floor rectangle and solids are drawn from the
   server's numbers, and they are the only blocking geometry the client draws or predicts. Where none
   is disclosed (market-town and social-cafe until 12d), the place's footprint is decoration from the
   pack, drawn but **not** enforced: the server accepts any stride there, so the client must not refuse
   one (A-3 is not carried over).
5. **People** are drawn where the newest observation puts them, smoothed toward it over the frame
   (prediction for display only), facing from `facing.yaw`. Appearance is chosen by the pack from
   tags and a stable hash of the `EntityId` string (never a number, `ADOPTION.md` §3.1). Labels are
   `display_name`, else the id. Participation in an activity is drawn as a marker from the disclosed
   `participation`. Nobody outside the observer's place is drawn, because nobody there is perceived:
   other buildings are façades, honestly empty.
6. **Inside.** When the observer's place is not the street, the cut-away of A-8 applies: that place's
   façade lifts, its interior and its people are drawn; the street is drawn dimmed with nobody in it
   (the observer does not perceive the street from inside).
7. **Objects** (after 12c): from the place's `loose-objects` listing, drawn by shape (`box`, `ball`)
   through `object:<shape>` roles; with events (**R-S11-4**) a moved object is animated along its
   `path`, otherwise it is moved to its new rest.
8. **The clock** shown is `Observation.at` formatted as the world's day and time; the revision and
   world instance are shown in a status line (they are `AC-15`'s evidence and persistence made visible).

### 4.4 Interaction through affordances, with no rule in the client

The menu is the observation's affordance list, filtered by *what was clicked*, and nothing else.

```text
click a person P      → affordances whose target is P
click myself          → target-less affordances (eat, drink, leave an activity, …)
click an object O     → target-less affordances whose payload names O (kick, throw), plus "throw here…"
click the floor       → move (a walk, §4.5); with an object selected, an aimed throw
press E / Space       → the first available affordance against the person under the cursor
```

Each entry is one of two kinds, and the difference is the whole design:

| Kind | Examples today | What the client does | What the client knows |
| --- | --- | --- | --- |
| **Complete** (`payload` present, ARC-34) | `give {item, count}`, `buy {item}`, `eat`/`drink {item}`; after 12c `kick {object}`, `throw {object, toward: null}`, `shove {}` | Shows one entry per affordance, in list order, labelled from the pack's wording table by action type, plus a name read from the payload's referenced ids where one is known (R-PK-2); submits it **unchanged** through `submit_affordance` (M-2). | Nothing about the action. An unknown action type is labelled by its id. |
| **Composed** (no `payload`) | `move`, `talk {utterance}`, `invite {kind}`, `accept-invitation`/`decline-invitation`/`join-group-activity`/`leave-group-activity {}`, aimed `throw {object, toward}` | Shown only when offered. Submitted with a payload `intents.gd` composes from player input (a point, a typed line, a chosen kind). | How to *ask*: the payload's shape. Not whether it exists or is allowed. An offered composed action the client has no composer for is shown greyed "not supported by this client", never guessed at. |

Rules, each enforced in §8:

- **Shown, not decided.** `available: false` is drawn greyed with the reason from `unavailable_reason`
  in the client's own words (`DD-13`) and, where declared, `requirement.within_range` as "needs 3 m"
  unevaluated (`ADOPTION.md` §3.3).
- **Submitted regardless.** Choosing a greyed entry submits it; the server answers. (The 3D slice does
  the same, A-29.) Refusing to submit because the client concluded it would fail is the defect.
- **The answer is the server's.** The result (`accepted`, `rejected` with reason, `unavailable`) is
  shown as a toast against the target; a `rejected too_far_away` on a person offers "walk closer",
  which is a `move` toward them — a new request the player chose, not a client verdict.
- **What was said.** Talk opens a text line; the history panel shows the observer's own
  `conversation-history`, names resolved through `display_name`. With **R-S11-4**, lines spoken
  between others at the observer's place appear as bubbles over the speakers.
- **Panels** read the observer's own disclosed components — wallet, holdings, acquaintances, agenda,
  employment, invitations — through `hud/readers.gd`, which knows each component's shape the way a
  client knows wording. An own component the client has no reader for is listed raw in a collapsible
  "other" panel rather than dropped (the step-08 C2 rule: carried and ignored, never an error).

`AC-13`: a click on Alice and E-at-Alice submit `talk` through the same composer, and the harness
records requests in demo.gd's format (A-15) so S14 can compare them.

### 4.5 Human control of a Person

- **Joining a seat** makes the client the controller of that Person (`INV-1`); nothing about the
  Person changes.
- **Two input modes, one request.** Click-to-move plans a route and walks it in strides of at most
  1.9 m (demo.gd's `STRIDE`, under `MAX_STRIDE`), each sent after the previous is answered. WASD moves
  the body continuously and reports a `move` every 0.5 m or 20° of turn (the 3D slice's cadence,
  A-29). Both obey the reporting rule (`PROTOCOL.md` §6.2).
- **Route planning is acquisition.** Within a place, the route avoids the disclosed solids where
  there are any (NavigationServer2D, §5.4); across places, it walks to the doorway's disclosed `here`
  point and submits the crossing to the destination's `there`. The server decides every stride; a
  refused stride ends the walk.
- **Reconciliation on difference** (step-11 §7.1 rule 4): when an observation places the observer
  more than 150 mm from the last reported position, the body snaps to the server's position. One rule
  covers a stride stopped short, a nudge, a shove and a refusal.
- **A stopped controller.** Closing the client leaves the Person where they stand; the world goes on
  (`AC-3`). Whether an agent resumes the seat is S11's handover (R-S11-3).

### 4.6 Reconnect

The module stays policy-free (`ADOPTION.md` §6); the client's `link.gd` owns the policy:

1. On `disconnected`, show "reconnecting", keep drawing the last observation greyed, and retry with
   capped exponential back-off (0.5 s doubling to 8 s, indefinitely while the window is open).
2. Re-join the **same seat** (and, after R-S11-1, the same token and nickname).
3. On `welcome`, compare `world.instance` with the previous one. **Same instance:** keep the learned
   layout and continue; the revision may be higher — the world went on. **Different instance:** the
   world was replaced; clear the layout cache and every drawn entity.
4. A `refused` on re-join is final for that attempt and shown by code (`unknown_seat`,
   `seat_not_in_world`, and S11's new ones, R-S11-2); `world_stopped` keeps retrying.
5. Any walk in progress is abandoned at disconnect; nothing is replayed after reconnecting. A request
   whose result never arrived is reported as "unknown — check the world", not retried: a retry could
   do a thing twice (the world allocates the identity, `INV-6`).

`AC-3` end to end is then: the scripted client is SIGKILLed mid-walk; the server's clock, revision and
the agent's facts advance; the client is relaunched, re-joins, is told the same instance and a higher
revision, and its observer stands where the server last put them (PR 13a's checkpoint).

### 4.7 Multiplayer

Each player runs their own `mineworld-2d` and joins a different seat on one server (`MVP.md` §6: up to
four humans). Other players are drawn as the Persons they are; the client does not mark them as human,
because the world does not disclose who controls a Person (`INV-1`), and inventing that would be a
client fact. A join screen asks for the address, the seat (listed from `GET /status`), and — after
R-S11-1 — the invite token and a nickname. Two 2D clients and the 3D client on one server is `AC-15`'s
shape, exercised by PR 13f's scripted two-client run and by S14.

### 4.8 A living town needs agents that take initiative

Demo A's "human and agent controllers" and the north star ("agent simulation exists to make that world
alive") are not met by NPCs that stand still until addressed (A-18). The 2D client cannot fix this and
must not try (a client animating NPCs would be a client inventing their lives). It is **R-S10-1**: the
hosted server drives the world's unoccupied seats with the paced controller in real time, and yields a
seat to a human who joins it. Until it lands, the client is demonstrated with `--agent` seats and is
honest about what it shows.

### 4.9 How the operator plays and judges it

**One command, as `./mineworld-slice --world` did for VIS-3D-GODOT-2:**

```text
./mineworld-2d                                   connect to 127.0.0.1:7878, join screen
./mineworld-2d --world worlds/market-town        start a local server for that world and join
./mineworld-2d --world worlds/market-town --seat visitor --presentation <dir> --variant=full
./mineworld-2d --drive                           headless scripted checks, exits non-zero on failure
./mineworld-2d --drive --capture                 the same, with stills (needs a window)
```

The launcher builds the Godot import cache on first run (the spike's launcher already does, A-1), and
builds `mineworld` if needed; a clean checkout must work (`ACCEPTANCE.md` §4.1).

**Two separate judgements, never mixed** (`ARC-11`, `ARC-20`):

- **Demo A (framework).** A checklist the operator runs (the milestone-handoff practice): walk out of
  the apartment, into the café; talk to Alice and read her reply; buy a coffee and see the wallet fall;
  give it to Bob; join a coffee with Bob; see an acquaintance appear; close the client mid-walk, reopen
  it, find yourself where you were; restart the server, find the same world; open a second client as
  another seat and see the first player. Each step is also in the scripted drive, so the checklist is
  a confirmation, not the only evidence.
- **VIS-2D-1 (taste).** A review package per `ACCEPTANCE.md` §6 for the connected client in the `town`
  style: launch command, runtime stills (street, café interior, a crowd), the references, the
  reference-framed still, known limitations, and the specific questions. Whether the connected,
  world-driven layout (market-town's street, A-30) may replace the spike's invented quayside square as
  the VIS-2D-1 candidate is the operator's (QS12-1, operator-material).

### 4.10 The gating questions (`ENGINEERING_RULES.md` §§11, 12, 9/22)

- **§11 — Does anything here constrain a Minecraft-like 3D client?** No. S12 adds no contract; every
  requirement it states (§6) is transport or pack disclosure that a 3D client benefits from equally
  (doorway destinations, item names, events, reconnect, auth).
- **§12 — Does a renderer concept enter a generic contract?** No. Projection, sprites, roles, routes
  and the cut-away live in `clients/2d/` and the Presentation Pack. R-PK-1 and R-PK-2 add tags and
  names — world data — to pack disclosures, not geometry.
- **§§9, 22 — Can 2D and 3D use each capability without duplicating game logic?** Yes, by construction:
  no game logic exists in either client (§8), and every action is the same request from both.

---

## 5. Reuse before reinvention

Per `REUSE_POLICY.md` and the operator's directive of 2026-10-08. Each non-trivial piece compares real
candidates and "build our own" on fit, licence, maturity and maintenance, and cost; the verdict is per
option. Facts about third-party projects were checked on 2026-10-08 (sources at the end of this
section); a PR design re-checks them before adopting anything.

The governing constraint from `REUSE_POLICY.md`: never force MineWorld inside a framework's execution
model — world state, authority and the protocol stay MineWorld's. That removes some candidates on fit
before cost is weighed.

### 5.1 Drawing the world (tilemap and rendering)

| Option | Fit | Licence | Maturity / maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Godot `Node2D` + `Sprite2D` with `y_sort_enabled`, `Camera2D`, `CanvasLayer` shaders** (what the spike uses) | Exact: free-placed sprites at world positions in millimetres; painter's sort by ground point; the accepted `town` look is built on it (A-7, A-8). | MIT (engine) | Core engine, maintained | Already written and measured in the spike | **Adopt.** |
| **Godot `TileMapLayer`** (built-in, isometric tile shapes) | Partial: good for ground (paving, grass) as tiles; poor for the world's continuous positions, and the accepted ground is drawn per slab with polygons, not tiles. | MIT | Core engine (4.3+), maintained | Re-authoring ground art as tilesets | **Adopt only if** a PR finds ground drawing too slow; not needed now. |
| **Tiled + YATI importer** | Poor: an authored map of the town would be a second, client-side source of layout, duplicating the world's passages and drifting from them; places would be bound twice. | Tiled GPL-2 (editor only, maps are data); YATI MIT | YATI actively released (2.2.7, 2026-03) | An authoring pipeline and a binding layer | **Reject** for layout. Revisit for decorative dressing per place if the pack grows large (it would still be decoration bound by tag). |
| **LDtk + importer** | Same objection as Tiled. | LDtk MIT; importer MIT | Importer maintained | Same | **Reject.** |
| **Build our own renderer** | — | — | — | High; no reason | **Reject.** |

### 5.2 UI widgets (menu, text input, panels, toasts)

| Option | Fit | Licence | Maturity / maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Godot `Control` nodes + `Theme`** (`PopupMenu`, `LineEdit`, `RichTextLabel`, `PanelContainer`) | Exact: menus built at runtime from a list; text entry; themable by the Presentation Pack. | MIT | Core engine | Low | **Adopt.** |
| **Dialogue Manager** (nathanhoad) | Wrong problem: authored, branching dialogue scripts that the game runs. MineWorld dialogue is world state — what is said is a `talk` request and a pack's history — so a dialogue runtime in the client would be a second source of conversation. | MIT | Production-ready, maintained (updated 2026-09) | Integration plus fighting its model | **Reject** (fit). |
| **Dialogic 2** | Same objection; also a heavier editor-driven model. | MIT | Less production-ready than Dialogue Manager by its peers' accounts | Higher | **Reject** (fit). |
| **Build our own widget toolkit** | — | — | — | High | **Reject.** |

### 5.3 The networking layer in Godot

| Option | Fit | Licence | Maturity / maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **`clients/protocol/mineworld` over Godot's `WebSocketPeer`** (exists, A-10) | Exact: speaks `PROTOCOL.md`, keeps ids as strings, proven against the real server from both clients. | MIT (engine) + ours | Merged, used by 3D | Zero | **Adopt** (already adopted). |
| **Godot high-level multiplayer (`MultiplayerAPI`, `WebSocketMultiplayerPeer`, RPCs, `MultiplayerSynchronizer`)** | Wrong model: Godot peers replicate scene-tree state with Godot authority; MineWorld's authority is the Rust server and the protocol has two client frames (`NETWORKING.md` §2). | MIT | Core engine | Would require a Godot-shaped server | **Reject** (fit; `REUSE_POLICY.md`'s "never force MineWorld inside a framework's execution model"). |
| **Nakama + its Godot client** | Wrong model: a game backend with its own authority, accounts, matchmaking — all non-goals (`MVP.md` §11). | Apache-2.0 | Mature | A second server | **Reject.** |
| **godot-rust (`gdext`) to reuse `mineworld-contracts` types inside the client** | Attractive in principle (one type definition, no JSON drift), but it makes the client a native GDExtension per platform, puts a Rust build in front of every client run, and breaks the "a client is GDScript that copies one folder" contract S5V proved. The protocol is JSON on purpose, so a non-Rust client is first-class (`R-9`). | MPL-2.0 | Usable, actively developed (2026-08) | Build matrix, export complexity | **Reject for S12**; record as a candidate if a typed Godot binding is ever wanted (DEP-S12-a). |
| **Build our own** | That is what the shared module is; nothing new to build. | — | — | — | — |

### 5.4 Pathfinding and click-to-move

| Option | Fit | Licence | Maturity / maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Straight strides only** (demo.gd) | Sufficient in places without solids (every town place until 12d) and for doorway routing (one waypoint). | — | Exists | Minimal | **Adopt** as the default route. |
| **Godot `NavigationServer2D` / `NavigationRegion2D` with a `NavigationPolygon` built at runtime from the disclosed `place-shape`** (floor minus solids, agent radius as margin) | Good: routes around counters and tables from the server's own numbers; the route is acquisition, the server still decides each stride. | MIT | Core engine, runtime baking from source geometry supported in Godot 4.x | Moderate; one builder function | **Adopt** when a place discloses solids (PR 13d). |
| **Godot `AStarGrid2D`** | Workable, but a grid is a second discretisation of a world that is continuous millimetres. | MIT | Core engine | Low | **Fallback** if navigation baking proves unreliable in tests. |
| **Third-party pathfinding addons** | No advantage over the engine's two built-ins for a single room. | various | various | Integration | **Reject.** |
| **Build our own A\*** | No reason. | — | — | Medium | **Reject.** |

### 5.5 Camera, input mapping, testing harness

Godot's `Camera2D` (smoothing, zoom), `InputMap`, and the spike's own headless `--drive` pattern
(scripted input, numeric assertions, capture behind `--capture`) cover these. Camera addons
(e.g. Phantom Camera, MIT) add nothing a 2D follow camera needs. Godot unit-test frameworks (GUT, gdUnit4,
both MIT) were considered for the harness and are **not adopted for now**: the evidence that matters is
a real client against a real server, which a scripted scene already produces; a framework would add a
dependency to assert the same things. Revisit if client-local logic grows (QS12-8).

### 5.6 Decision record proposed

`DEP-S12-a` — *The 2D reference client is built from Godot built-ins and the shared protocol module; no
addon is required.* It records §§5.1–5.5's comparison, the rejections on fit (Godot MultiplayerAPI,
Nakama, dialogue runtimes, Tiled/LDtk for layout) and the deferred candidate (`gdext`).

Sources checked 2026-10-08:
[Dialogue Manager](https://github.com/nathanhoad/godot_dialogue_manager) ·
[Dialogue Manager, Godot Asset Store](https://store.godotengine.org/asset/nathanhoad/dialogue-manager/) ·
[gdext](https://github.com/godot-rust/gdext) ·
[YATI](https://github.com/Kiamo2/YATI) ·
[LDtk importer](https://github.com/heygleeson/godot-ldtk-importer).

---

## 6. Requirements S12 places on other steps

S11 owns the wire protocol (`PROTOCOL.md`); S12 consumes it and designs none of it. Each requirement
below states the **need** and the **minimal shape the 2D client can consume**; the frame design is
S11's, and the primary session reconciles. "Blocks" names the S12 PR that waits on it.

### 6.1 On S11 (protocol and server)

| ID | Need | Minimal shape S12 can consume | Blocks |
| --- | --- | --- | --- |
| **R-S11-1** | **Authentication** (`NETWORKING.md` §9: invite token + nickname). The client's join screen must collect and send both. | The join frame carries the token and a nickname alongside `seat`; a wrong or missing token is a `refused` with a **distinct code** (e.g. `bad_token`) the client can branch on; the welcome echoes the nickname. The client must be able to list a world's seats **with the same credentials it joins with** (today `GET /status`), so the seat picker works on a protected server. A server started without a token keeps accepting a join without one (localhost play stays one command). | 13e |
| **R-S11-2** | **Seat occupancy**, so two players cannot drive one Person by accident, and a reconnecting player is not locked out by their own dead connection (A-17). | A join to a seat another live connection holds is `refused` with a distinct code (e.g. `seat_occupied`). A join presenting the **same credentials** as the holder replaces the holder (the old connection is closed with a reason the client can show). The seat listing says which seats are free. | 13e |
| **R-S11-3** | **Human takeover of an agent-driven Person** (`AC-5`), seen from the client. | Joining a seat an agent drives succeeds, and the agent stops acting for it while the human holds it; whether it resumes on leave is S11's. The client needs no new frame; an optional welcome field saying a controller was replaced lets it show "you have taken over Alice". | 13e (shown), 13f (asserted) |
| **R-S11-4** | **Events in observations**, so a player sees what happens around them: NPCs talking to each other, an activity starting, an object flying along its `path` (A-27), a shove. Rev 1 leaves `events` empty (A-20). | `observation.events` lists facts public at the observer's place (`Visibility`) that the observer has not yet been sent, each with its `EventId` **as a string**, its event type id, `at`, the place, the `ActionId` that caused it when there is one, and the owning pack's payload as JSON. At-least-once within a bounded window is enough: the client de-duplicates by id. A bound per frame is fine; a dropped event must not be silently presented as complete (a count of omitted events suffices). | 13d (animation), 13e (bubbles) |
| **R-S11-5** | **Revision discipline.** The shared module refuses an unknown `protocol` (A-10). | Additive fields stay within revision 1 (as `revision` and `payload` did); anything a rev-1 client would misread raises `protocol`, and the PR that raises it updates `clients/protocol/mineworld/` in the same change (M-5). | all |
| **R-S11-6** | **A time scale for hosted worlds**, so a demo shows a working day in minutes (A-19). | A server option setting world seconds per real second (default 1, so nothing changes unless asked), reported in the welcome/status summary so the client can tick its clock display smoothly between frames. | 13e, 13f |
| **R-S11-7** | **Deltas: none needed.** Whole observations at 10 Hz carry a dozen people comfortably. | If S11 introduces deltas, they are either opt-in at join or reassembled inside the shared module, so `MineWorldObservation` stays whole and `ADOPTION.md` §3.4 ("render the newest observation, an observation is exhaustive") stays true. | — |
| **R-S11-8** | **Admin frames: none needed.** The reference client is played from inside a Person; it is not a dashboard (`ENGINEERING_RULES.md` §1). | — | — |
| **R-S11-9** | **Stable refusal codes and the frame `revision`** stay as rev 1 has them. | — (already true; recorded so a redesign keeps them). | — |

### 6.2 On S10 (controllers), or S11 — the primary session assigns

| ID | Need | Minimal shape | Blocks |
| --- | --- | --- | --- |
| **R-S10-1** | **Hosted agents that take initiative** (A-18). Demo A's "agent controllers" and a living town. | `mineworld server <world>` can drive its unoccupied seats with the paced rule controller in real (scaled) time, through the same seat path (`INV-1`), and yields a seat a human joins (with R-S11-3). | 13e, 13f |

### 6.3 On System Packs — framework PRs, outside the client PRs

These are world semantics, not renderer accommodations: what a person standing in the street can see
on a sign, and what a thing is called. Neither touches `contracts/` or `kernel/` (`INV-14` holds).
PR 13c carries them (§9), unless the primary session places them elsewhere.

| ID | Need | Proposed route | Constraint |
| --- | --- | --- | --- |
| **R-PK-1** | **A doorway says where it leads** (A-23): the destination's tags. | `movement`'s `passages` disclosure joins each destination's tags into its `leads_to` entry, from current state at disclosure (F-O5's precedent). Additive to the disclosed JSON; no fact changes, so no digest changes. | The paced controller reads `passages` (`cognition/rule-controller/src/{agenda,paced}.rs`): its decoder must tolerate the added field, and the 300-day digests must be shown unchanged. |
| **R-PK-2** | **Item kinds have names** (F-41, A-24). | `item`'s own `item:` section gains a `name`; `item` discloses, on every perceived place, a catalogue of declared kinds `{item, category, name}`. A client resolves any item id it meets — in holdings, a listing, a payload, a loose object — through it. | Keeps AC-1 check 3 (A-25: `item:` is owned by one of the six). **Changes genesis facts, hence both towns' digests** — see QS12-3. Organization names are not needed by any Demo A interaction and stay out. |

---

## 7. Changes to the shared GDScript module `clients/protocol/mineworld`

Live in the 3D client the moment they merge (symlink, A-10). Every change below is **additive**; none
changes the meaning of an existing method. Each lands with the 3D slice's `--drive` and the protocol
demo's `run.sh evidence` re-run on the PR head, and is coordinated with S14 and S15's 12e, which also
edit `ADOPTION.md`.

| ID | Change | Why | PR | Coordination |
| --- | --- | --- | --- | --- |
| **M-1** | `MineWorldObservation.choices(action_type := "", target: Variant = null) -> Array` — every affordance matching, in list order, both kinds; `""` matches any type. `MineWorldObservation.is_complete(affordance) -> bool` (has `payload`). | A-11 / QS-20: `affordance()` returns only the first match. | 13b | S14 may use it; 12e unaffected. |
| **M-2** | `MineWorldClient.submit_affordance(affordance: Dictionary, actor_location: Variant = null) -> String` — submits `action_type`, `target` and `payload` exactly as offered. `submit`'s `payload` parameter widens from `Dictionary` to `Variant` (source-compatible for every existing caller). | ARC-34's "submit it unchanged" as one call that cannot be got wrong; A-12. | 13b | 12e's kick/shove can use it. |
| **M-3** | `MineWorldClient.revision: Variant` (an `int` or `null`), set from every observation frame's `revision`; `ADOPTION.md` §2 documents it. | A-13: persistence and `AC-15`'s fourth line made visible to a client. | 13a | S14's `AC-15` evidence. |
| **M-4** | `ADOPTION.md` §6 gains the reconnect pattern of §4.6 (re-join the seat, compare `instance`, never replay an unanswered request) as guidance; the module stays policy-free. | A-14. | 13a | S14 adopts the same pattern. |
| **M-5** | Join credentials: `connect_to_world(address, seat, credentials: Dictionary = {})`, with the fields S11 decides (R-S11-1), and any protocol-number change (R-S11-5). | R-S11-1. | 13e, or S11's own PR | **Shape owned by S11.** Whichever PR changes the join frame changes the module. |
| **M-6** | `MineWorldObservation.events()` already exists; if S11's events (R-S11-4) are not a plain array in `observation.events`, the reader follows S11's shape. De-duplication by id stays in the client. | R-S11-4. | S11's PR or 13d | Owned by S11's shape. |
| **M-7** | `ADOPTION.md` §2–§4: document M-1–M-3, the composed/complete distinction of §4.4, and that an offered action a client cannot compose is shown, not guessed. | Keep the specification of the module current. | 13b | 12e adds the 150 mm reconciliation rule to §4 in the same file — sequence the two edits. |

**Not changed:** `space.gd` (the isometric projection is the 2D client's, §4.2), `demo.gd` and
`run.sh` (they stay the minimal reference and the `AC-13` evidence source until S14 switches it to the
real clients, QS12-14).

---

## 8. Invariants and the adversarial checks

### 8.1 Invariants of this step

| ID | Invariant |
| --- | --- |
| **I-1** | **No world rule in the client.** It never decides whether an action exists, is available, is near enough, is permitted or would succeed; it never refuses to submit what the player chose. Distances it computes are request sizes (stride splitting), route planning and drawing — each named in one file. |
| **I-2** | **A client acts and never asserts** (`INV-5`, `INV-9`). It sends `join` and `submit` only, through the shared module; nothing it holds is world state, and its layout cache (§4.3) is presentation memory dropped with the instance. |
| **I-3** | **Packs it was never written against work.** A complete affordance from any pack is shown and submitted unchanged; an unknown own component is shown raw; an unknown composed action is shown greyed as unsupported, never guessed. |
| **I-4** | **The client is removable.** Client PRs (13a, 13b, 13d, 13e, 13f) change nothing under `kernel/`, `contracts/`, `systems/`, `server/`, `worldpack/`, `persistence/`, `authoring/`, `sdk/`, `cognition/`, `worlds/`. Framework needs go through §6, in their own PRs. |
| **I-5** | **Presentation independence.** The same scripted drive against the same world submits the same requests (by semantic core, `server/src/parity.rs`) with the default pack, with each `ARC-14` variant and with no pack. |
| **I-6** | **Identity is a string; every other number is an integer** (`ADOPTION.md` §§3.1–3.2). |
| **I-7** | **Only the perceived is drawn.** No person, object or state appears that the newest observation did not list; what it does not list is removed. |
| **I-8** | **Existing behaviour is unchanged by client PRs.** Both towns' 300-day digests, `ac1_composability`, the protocol demo evidence and the 3D slice's drive all still pass on each client PR's head. |
| **I-9** | **Headless-runnable.** The client's scripted drive runs with `--headless` and exits non-zero on failure; stills are behind `--capture` (A-7). |

### 8.2 The adversarial "no rule in the client" check, runnable

Five parts. Each is shown to **bite** on a planted violation before it is trusted (`ARC-23`).

1. **Static scan — `scripts/check_client_rules.py`** (new, standard library only, reports by file and
   line, exits non-zero):
   - only `clients/2d/scripts/intents.gd` calls `submit(` or `submit_affordance(`;
   - `intents.gd` and `menu.gd` never read `may(`, `"available"`, `unavailable_reason`,
     `requirement(`, `distance_to`, `length(`, `within` in the same function that submits;
   - only `walker.gd`, `projection.gd`, `town.gd` and `scene/` may compute distances, and none of them
     calls `submit`;
   - no `.gd` file outside `intents.gd` contains a string literal equal to an action type id the client
     composes.
   Planted violation: `if not obs.may("talk", target): return` before a submit → named failure.
2. **The lying server.** A test-only WebSocket stub (Rust, test code in a client-scoped test file,
   speaking rev 1) seats the scripted client and offers `talk` to a person as `available: false,
   too_far_away`; the scripted player chooses it; the stub asserts the `submit` arrived. Inverse: it
   offers `available: true` and answers `rejected too_far_away`; the client shows the rejection and its
   drawn state changes only with the next observation. Planted violation as above → the first case
   fails because no submit arrives.
3. **The teleport.** The stub's next observation moves the observer 5 m; the client snaps to it within
   one frame and does not submit a correcting `move` (reconciliation, not argument).
4. **A pack the client never heard of.** Against a world composed with `tests/acceptance`'s synthetic
   complete-affordance pack (the `ring` bell of `PROTOCOL.md` §5), the scripted player opens the menu,
   finds `ring` with each offered payload, submits one, and the server accepts — with no client edit.
5. **A pack removed.** Against market-town with `item-transfer` removed, the menu offers no `give` and
   nothing else changes (`AC-2` through the client).

Plus the path-scope check of I-4 on each client PR's actual merge diff, as `ac1_composability`'s check 1
reads merge diffs.

---

## 9. The PR split

Six PRs. Each is detailed to the commit and frozen in turn. "Now" means it can be designed and built
against `main` and protocol revision 1 today; "waits" names the merge it needs.

| PR | Scope | Starts | Integration checkpoint | Adversarial criteria |
| --- | --- | --- | --- | --- |
| **13a — A world you can walk** | `clients/2d/` project adopting the module by symlink; composition root; `link.gd` with the reconnect policy (§4.6); `town.gd` (place frames glued at passages, layout cache); `projection.gd` (the spike's `Iso.gd` over `MineWorldSpace`); people, places and labels; `walker.gd` (click-to-move strides, WASD reporting, doorway crossing, reconciliation on difference); the `town` art moved into `presentation/mineworld-default/2D/` with provenance and `asset_bindings.yaml` (QS12-4, QS12-10); plain fallback; `--drive`/`--capture`; the `mineworld-2d` launcher; M-3, M-4. | **Now.** | Against `mineworld server worlds/market-town --agent alice --save DIR`: the scripted client walks out of the apartments, along the street and into the café through two doorways, every stride server-accepted, and its observer's place changes twice; Alice is drawn at her server position, labelled by name. **`AC-3` end to end:** the client is SIGKILLed mid-walk; the server's revision and Alice's facts advance; the client is relaunched, re-joins, is told the same instance and a higher revision, and is drawn where the server last put it. Stills captured and inspected. | No-pack and `town` drives submit identical requests (I-5). A stride the server refuses ends the walk; the client never walks through it. Static scan parts (1) live and shown to bite. Path scope (I-4). 3D slice drive and protocol evidence unchanged (I-8). |
| **13b — Interaction through affordances** | `intents.gd` (composers for `move`, `talk`, `invite`, accept/decline/join/leave), `menu.gd` (both kinds, greyed with reasons, submit regardless), complete affordances generic (`give`, `buy`, `eat`, `drink`), results as toasts, talk input and history, HUD readers (wallet, holdings, shop listing, acquaintances, invitations, participation, agenda, employment, raw "other"); `AC-13` request transcript; M-1, M-2, M-7. | **Now.** | Market-town through the real server with `--agent alice` and `--agent bob`: the scripted player talks to Alice and shows her reply from its own history; buys a coffee (wallet falls by the price, holdings gain one); gives it to Bob; invites Bob for coffee and a second scripted 2D client seated as Bob accepts, after which both are drawn with the participation marker (the hosted `RuleController` answers talk only, so an agent cannot accept until R-S10-1); acquaintances panel shows Alice. | §8.2 parts 1–5 all pass and each bites. The `AC-13` transcript is in demo.gd's format and `ac13_semantic_parity`'s comparison accepts it against `request-3d.json`. |
| **13c — Names for things** (framework, packs only) | R-PK-1 in `systems/movement`; R-PK-2 in `systems/item` and `worlds/market-town/items/*.yaml`; the client reads both (a few lines in `town.gd` and `hud/readers.gd`, or in 13b/13d if they land later). | R-PK-1 **now**; R-PK-2 **waits** on QS12-3's decision about the digest re-baseline (S15 12d). | From the street, the client draws the café's façade before ever entering it; every buy, give, eat and drink entry and every holdings line shows the kind's name. | `ac1_composability` 13/13 (check 3 still holds); the paced controller's digest unchanged by R-PK-1; R-PK-2's re-baseline only as QS12-3 decides; removing `item` removes the catalogue and the client falls back to ids. |
| **13d — Bodies in 2D** | Walls and solids drawn from `place-shape`; NavigationServer2D routes around solids; loose objects drawn from `loose-objects`; `kick`, default `throw` and `shove` through the generic complete path; aimed `throw` composed from a floor click; object animation along `path` when events exist (R-S11-4), otherwise a move to rest. | **Waits** on S15 12c (bodies-yard) and 12d (towns). | In `worlds/bodies-yard`: the scripted player walks into a box and the drawn box moves where the server says; kicks a ball; throws it at a clicked point; is shoved by an `--agent` seat and is drawn where the server put it; is stopped by a wall the server resolved. | With the client's route planning disabled, the server still stops the player at the wall and the client reconciles. No collision rule in the client: the static scan finds none. |
| **13e — Many players, a living town** | Join screen (address, seat picker, token, nickname), occupancy and takeover messages, speech bubbles and activity notices from events, the clock at the server's time scale, M-5, M-6. | **Waits** on S11 (R-S11-1–4, R-S11-6) and R-S10-1. | Two 2D clients as two seats plus hosted agents on one persisted server: each sees the other walk; one speaks to Alice and the other sees the bubble; Alice, agent-driven, crosses the street on her own; one client takes over Bob and Bob's biography, relationships and holdings are as before (`AC-5` as seen). | A wrong token is refused by code and nothing is drawn; a second client asking for an occupied seat is refused; a reconnect with the same credentials replaces its own dead connection. |
| **13f — Demo A proof and VIS-2D-1 package** | The Demo A acceptance run (every item of `MVP.md` §7.1, scripted, two clients and agents, SIGKILL of a client and of the server, resume); the operator's runnable checklist; the VIS-2D-1 review package (`ACCEPTANCE.md` §6) for the primary session to queue. | **Waits** on 13a–13e. | The checklist of §4.9 run end to end by the harness and by hand; persisted revision identical before and after a server restart; `AC-15`'s evidence lines read from the 2D side. | The I-4 path scan over the whole step's client merges; §8.2 re-run on the final head. |

**Ordering.** 13a → 13b are the critical path and need nothing from S11; they make Demo A playable
against today's server for one or more players (non-exclusive seats). 13c is independent of the
client PRs. 13d follows S15. 13e follows S11. 13f closes the step.

---

## 10. Files touched

| Path | PRs | Kind |
| --- | --- | --- |
| `clients/2d/**` (new: `project.godot`, `scenes/`, `scripts/`, `scripts/harness/`, `run.sh`, `evidence/`, `README.md`, `.gitignore`, symlink `mineworld -> ../protocol/mineworld`) | 13a, 13b, 13d, 13e, 13f | new |
| `mineworld-2d` (repository root launcher) | 13a | new (adapted from the spike's) |
| `presentation/mineworld-default/2D/` — `assets/asset_bindings.yaml`, `renderer/godot.yaml`, `art/` (the `town` subset), `LICENSES/` or provenance files, `README` updates | 13a | new files in an existing pack |
| `clients/protocol/mineworld/world_client.gd`, `observation.gd` | 13a (M-3), 13b (M-1, M-2), 13e (M-5, M-6) | shared, additive |
| `clients/protocol/ADOPTION.md` | 13a (M-4), 13b (M-7) | shared spec |
| `scripts/check_client_rules.py` | 13a (introduced), 13b (extended) | new |
| Client-scoped Rust tests (e.g. `tools/cli/tests/client_2d_*.rs`, stub server, Godot-gated runs) | 13a, 13b, 13d, 13e, 13f | new |
| `systems/movement/src/*` (R-PK-1), `systems/item/src/*` and `worlds/market-town/items/*.yaml` (R-PK-2) | 13c | framework |
| Markdown: this step, its PR documents, `clients/2d/README.md`; proposals in §13 applied by the primary session | all | docs |

Not touched by any S12 PR: `kernel/`, `contracts/`, `server/`, `persistence/`, `worldpack/`,
`authoring/`, `sdk/`, `cognition/`, `clients/3d-spike/` (other than through the symlinked module),
`clients/protocol/demo/`, `space.gd`.

---

## 11. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **RK-1** | The spike's art is 101 MB (A-6); landing it bloats every clone. | Land only the `town` subset the bindings reference (≈ 17 MB: `generated`, `ground`, `interior`, `svg`), with provenance; candidates and screenshots stay on `vis/2d-generated-assets` (QS12-10). |
| **RK-2** | A module edit breaks the 3D client live (symlink). | Additive only (§7); 3D slice `--drive` and the protocol evidence re-run on every PR head that edits the module. |
| **RK-3** | No Godot in CI, so the client's evidence is not re-run automatically; a test not run is not a pass. | Godot-executing tests are marked and reported `NOT RUN` when Godot is absent, never green; S13 is asked to install headless Godot in a CI layer (QS12-8). |
| **RK-4** | S11's join, events and occupancy shapes differ from §6's assumptions, forcing rework in 13e. | 13a/13b depend on nothing new; 13e is designed only after S11 freezes; §6 states needs, not frames. |
| **RK-5** | R-PK-2 changes genesis facts, re-baselining both towns' digests, which S15 froze to happen "in 12d, and only there". | QS12-3: carry the names in 12d's re-baseline, or amend that invariant explicitly. |
| **RK-6** | The world-driven layout (market-town's street) looks less like the references than the spike's invented square, and VIS-2D-1 regresses in taste. | QS12-1 puts the choice to the operator; the pack may dress places by tag; the street's art is decoration bound to the disclosed doorways. |
| **RK-7** | Isometric picking is ambiguous near façades and doorways (a click may land in a building's sprite). | Pick against drawn footprints, not sprite rectangles; the drive asserts doorway crossings. |
| **RK-8** | The client drifts into a dashboard (north star). | The camera follows the player; nothing is drawn that the player's Person does not perceive (I-7); no map of the whole town's people. |
| **RK-9** | Without R-S10-1 the town looks dead and Demo A's "agent controllers" is hollow. | Stated as a dependency; 13a/13b use `--agent` seats and say so. |
| **RK-10** | Concurrent edits to `ADOPTION.md` by S12, S14 and S15's 12e conflict. | §7 lists each section touched; the primary session sequences them. |

---

## 12. Questions

Marked **[OPERATOR-MATERIAL]** where they are taste, scope, or a change to an operator decision; the
rest are for the primary session.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QS12-1** | **[OPERATOR-MATERIAL — visual taste]** Is VIS-2D-1 judged on the connected client, laid out from the world (market-town's street, A-30), in the `town` style — replacing the spike's invented quayside square as the candidate? | **Yes.** A playable 2D scene must be a world you can play in; the invented square has no people to meet and no café to buy in. Keep the spike branch as the visual reference, unmerged. |
| **QS12-2** | **[OPERATOR-MATERIAL — taste, and a spec wording change]** The accepted look is 2:1 isometric (`ARC-14`, A-4), while `MVP.md` §7.1 says "top-down". Keep isometric as the default, with a plain north-up plan as the no-pack mode? | **Yes**, and amend `MVP.md` §7.1's wording to "a simple Godot 2D client (isometric or top-down)". Nothing semantic depends on it. |
| **QS12-3** | **[OPERATOR-MATERIAL — changes a frozen S15 invariant]** F-41's route (R-PK-2: a name in `item:`, a kind catalogue disclosed by `item`) changes genesis facts. Either 12d carries it inside its one re-baseline, or 13c re-baselines again and S15's "only in 12d" is amended. | **Carry R-PK-2 in 12d's re-baseline** (12d is not yet frozen); if 12d's design cannot absorb it, amend the rule with evidence. |
| **QS12-4** | `ARC-14`'s variants: binding sets inside one Presentation Pack, or four sibling packs? | **Binding sets inside `mineworld-default/2D`** (`asset_bindings.yaml` `variants:`), `town` default, `--variant=` kept as the interface `ARC-14` names. A sibling pack remains possible for a truly different style. |
| **QS12-5** | Names: client at `clients/2d/`, launcher `./mineworld-2d`; the spike branch is not merged. | **Accept.** `clients/3d-spike` keeps its name until S14 decides its own. |
| **QS12-6** | `invite`'s `kind` is a free slug and the only list of kinds is the rule controller's (`coffee`, `chat`, `walk`). The human chooses how? | **Free player input with suggestions from the Presentation Pack's wording table**, like an utterance. Not a pack change: making invites complete would change the paced controller's behaviour and both digests. |
| **QS12-7** | R-PK-1: the destination's tags joined into `passages`, an additive disclosure in `movement`. | **Accept**, in 13c, with the paced controller's digests shown unchanged. Fallback if refused: learn tags by entering (§4.3 point 3). |
| **QS12-8** | Godot-executed tests: run how, and where? | Rust tests that spawn `godot --headless` are marked ignored-unless-Godot and documented with their exact command; evidence is committed; results are reported `NOT RUN` when Godot is absent. **Ask S13** for a CI layer with headless Godot 4.7.2. |
| **QS12-9** | Should place-frame gluing (§4.3) live in the shared module now, for S14? | **Not yet.** 2D-only in 13a; it moves into the module by a listed change when S14 needs it with the same meaning. |
| **QS12-10** | Art on `main`: only the `town` subset, plain Git, no LFS? | **Yes** (≈ 17 MB, RK-1), with per-asset provenance carried from the branch (`DEP-8`). |
| **QS12-11** | **[OPERATOR-MATERIAL — scope]** Hosted agents that take initiative (R-S10-1) are not named in S10's reduced scope or in S11's output. Are they in MVP-0, and whose step? | **In MVP-0, in S10 (reduced)** — "the controller abstraction with `HumanController` and `RuleController`" hosted for real; without it Demo A's agents only answer. |
| **QS12-12** | R-S11-6, a server time scale (default 1:1). | **Accept**; the demo needs a day in minutes, and world semantics are unchanged by how fast real time maps to world time. |
| **QS12-13** | Where the `AC-3` end-to-end test lives. | A client-scoped Godot-gated test under `tools/cli/tests/` beside `milestone_c.rs`, driving the real binary and the real client (13a). |
| **QS12-14** | `demo.gd` and `run.sh`: keep them as the minimal protocol reference and `AC-13`'s current evidence source? | **Keep, unchanged.** S14 decides when `AC-13` compares the two real clients. |
| **QS12-15** | Is 13d (bodies in 2D) required for S12 to complete, given S15 says it "feeds S12 (the same three actions)"? | **Yes for bodies-yard after 12c**; for the towns, after 12d. S12 does not wait on 12e. |

---

## 13. Proposed edits to other documents (applied by the primary session)

- **`overall.md` §3 S12:** "Depends on: S11" → "13a–13b on `main` today; 13d on S15 12c/12d; 13e on
  S11 and R-S10-1"; link this document; list PRs 13a–13f. **§4:** `AC-3` row unchanged; add "F-41 names
  for item kinds — S12 (13c)". **§7:** S12 entry pointing here.
- **`docs/MVP_STATUS.md`:** the "2D client" row and the S12 row point to this step.
- **`docs/DECISIONS.md`** (placeholders):
  - `ARC-S12-a` — *A 2D client draws one town by gluing place frames at their disclosed passages,
    translation only; the layout is learned presentation memory, never world state.*
  - `ARC-S12-b` — *A Presentation Pack binds roles to art (`assets/asset_bindings.yaml`); a client core
    names no asset; an unbound role is drawn plainly.* (First concrete content for `MODULE_SPEC.md`
    §6.1's named file.)
  - `ARC-S12-c` — *A client composes only the actions it knows how to ask for, submits complete
    affordances unchanged, and decides nothing; the adversarial check of §8.2 is the test.*
  - `DEP-S12-a` — §5's comparison and verdicts.
- **`docs/MVP.md` §7.1:** "top-down" wording, if QS12-2 is accepted.
- **`server/PROTOCOL.md`:** S11's, for R-S11-1 … R-S11-6.
- **`docs/HUMAN_REVIEW_QUEUE.md`:** the VIS-2D-1 package when 13f is ready (QS12-1).
- **`step-11-bodies.md`:** 12d's scope gains R-PK-2's re-baseline, if QS12-3 is accepted.

---

## 14. PR 13a — A world you can walk (full design)

### DESIGN FROZEN

```text
Lifecycle:            DESIGN FROZEN (2026-10-08), primary session
Design revision:      §14 as committed in 26c451d
Approved by:          the primary session's message of 2026-10-08, "FREEZE: S12 PR 13a is DESIGN FROZEN
                      (2026-10-08), primary session. §14 is accepted as written", covering AC-W1 … AC-W12
                      with their mutations, the preview gates and play commands, the ≈ 18.4 MB art subset
                      with sidecars loaded at runtime, the JSON-compatible YAML subset, C0–C7, ARC-45 …
                      ARC-47 and DEP-16, the verbatim ARC-14 port, and the S11-A coordination
Implementation base:  main @ 47c81d1
Execution contract:   §14.9
```

### 14.0 Identity, base, scope

```text
PR            13a — a world you can walk (S12, first of six; PR number assigned at freeze)
base          main @ 47c81d1 (re-audited: 0fd0be3..47c81d1 changes only Markdown, so §2's A-1 … A-30 hold)
branch        mvp0/pr-s12-13a-walkable-2d, worktree /Users/yuema137/mineworld-worktrees/impl-s12-13a,
              held by the 13a implementation session only
parents       overall.md "Parallel build-out, 2026-10-08" (binds); this step §§1–13; ARC-14 (spike branch)
scope         §9's 13a row, as amended by §14.1
```

### 14.1 What the 2026-10-08 rulings change in §9's 13a row

| §9 says | Ruling | 13a does |
| --- | --- | --- |
| "M-3" (frame `revision` in the module) | Ruling 4: only 16b and S11 edit `clients/protocol/mineworld/`. The coordinator, 2026-10-08: 16b is frozen and adds `MineWorldClient.revision`. | **Does not edit the module.** 13a reads the welcome's `world.revision` (`PROTOCOL.md` §5), which carries the revision at join — enough for `AC-3` ("same instance, a higher revision"). The status line shows the welcome's revision until 16b lands; if 16b is on `main` when 13a rebases, the status line reads `MineWorldClient.revision` instead (bounded; recorded). |
| "M-4" (`ADOPTION.md` §6 reconnect guidance) | The coordinator, 2026-10-08: "S12's M-4 (reconnect guidance in `ADOPTION.md` §6) stays with your 13a." | Edits **`clients/protocol/ADOPTION.md` §6 only** (the document, not the module). 16b edits §2 of the same file: whichever merges second rebases; neither touches the other's section. |
| Module names in this step (`choices`, M-1, M-2) | 16b's final names: `affordances(action_type, target)` (`null` target = any, `""` = target-less), `complete_affordances(...)`, `is_complete(aff)`, `affordances_about(id)`, `component_value(id, type)`, `submit_affordance(aff, actor_location)`, `revision`. | 13a calls **none** of them: it reads only `self_location`, `place`, `entities`, `entity`, `location_of`, `component`, `display_name`, and submits only `move`, all existing on `main`. 13b's design uses 16b's names (`choices` in §4.4/§7 is superseded). |
| "top-down" (`MVP.md` §7.1, `ART_DIRECTION.md` §20) | Operator: isometric, in the `town` style, laid out from market-town's real street (QS12-1, QS12-2). | C1 rewords both sentences; the no-pack mode is a north-up plan. |
| Decision placeholders | Ruling 6: S12 holds `ARC-45 … ARC-47`, `DEP-16`. | `ARC-45`, `ARC-46`, `ARC-47`, `DEP-16` (§14.4). Also ports `ARC-14` (already reserved for the spike's decision, `ARC-16` "identifier gap") verbatim onto `main`. |
| — (new since the step) | S11-A (in flight, `impl-s11a`) makes the join carry `invite` and `nickname` and requires an invite even on loopback (QS11-6), and updates "every in-repo caller". | 13a has exactly **one** `connect_to_world` call (in `link.gd`) and one place that starts a server (the launcher). If S11-A merges first, 13a rebases and passes the launcher-captured invite through that call — a consumer change, no module edit (bounded, recorded). If 13a merges first, S11-A's "every in-repo caller" includes it. |

### 14.2 Acceptance — decided before measuring (`ARC-23`)

Every bound below is a fixed literal from a requirement, never derived from the quantity under test.
"Godot-gated" tests are `#[ignore = "needs Godot 4.7 on PATH"]` in `tools/cli/tests/client_2d.rs`, run
with `cargo test -p mineworld-cli --test client_2d -- --ignored`; reported `NOT RUN` where Godot is
absent, never green (RK-3, QS12-8).

```text
AC-W1  WALK (integration checkpoint). Against `mineworld server worlds/market-town --agent alice --save DIR`
       (fresh DIR), the scripted 2D client seated as `carol` (genesis: apartments (3000, 2500)) walks out
       of the apartments, along the street, and into the café through two doorways.
       PASS iff: every `move` it submitted has an `accepted` result (transcript); the observation's
       `place()` changes exactly twice (apartments → street → café); and, independently of the client,
       the save's fact log (read by the Rust test after the server is stopped) holds exactly two
       `person-entered-place` facts for carol's id, to the street then to the café, and as many accepted
       move facts as the transcript has accepted moves.
       MUTATION: the walker submits a doorway crossing to the destination's `here` instead of its
       `there` → the crossing is rejected too_far_away → FAIL.
AC-W2  DRAWN WHERE THE SERVER SAYS. In the café, for every person the observation lists, the drawn node's
       ground point, inverse-projected, equals the observation's `location.local` within 1 mm after the
       smoothing settles (≤ 1 s); Alice's label equals `display_name(alice)`, which is non-empty and is
       not her id. No drawn person is absent from the newest observation (I-7).
       MUTATION: the label falls back to the id first → FAIL; a person dropped from the observation
       keeps its node → FAIL.
AC-W3  AC-3 END TO END (relaunch). During AC-W1's walk, after the first accepted street stride, the test
       SIGKILLs the Godot process. While it is dead: a Rust client seated as `wanderer` says something to
       Alice (the `--agent` answers), and a Rust client seated as `carol` (seats are not exclusive
       today, A-17) moves carol one stride and then leaves; `/status` shows `at` and `revision` higher
       than before the kill. The client is relaunched. PASS iff its welcome names the same `instance`,
       a `revision` ≥ the post-kill `/status` revision, and > the pre-kill one; and carol is drawn at
       the position the Rust client moved her to (1 mm), not where the dead client left her.
       MUTATION: the client restores a remembered position at startup → FAIL.
AC-W4  RECONNECT POLICY (§4.6), in-process. (a) The server is SIGKILLed while the client is seated and
       restarted on the same `--save` and port: the client shows "reconnecting", re-joins the same seat
       within 10 s, is told the same instance, keeps its learned layout, and abandons the walk in
       progress (no `move` is submitted between disconnect and the first new observation). (b) A server
       on a fresh save on the same port: a different instance; the layout cache is cleared (the harness
       reports the cache's passage count = the new observation's passage count).
       MUTATION: replay the unanswered stride after re-joining → (a) FAIL.
AC-W5  A REFUSED STRIDE ENDS THE WALK (stub). A test-only rev-1 WebSocket stub (Rust, test code) seats
       the client and answers its third stride `rejected too_far_away`. PASS iff no fourth stride is
       submitted and the body is drawn at the stub's next observed position within one frame.
       MUTATION: the walker ignores rejections → FAIL.
AC-W6  THE TELEPORT (stub, §8.2 part 3). The stub's next observation moves the observer 5 m. PASS iff the
       drawn body snaps there within one rendered frame and no `move` is submitted in response.
       MUTATION: the walker "corrects" back to its predicted position → FAIL.
AC-W7  SUBMITTED REGARDLESS (stub, §8.2 part 2 for `move`). The stub offers `move` as
       `available: false, too_far_away`. PASS iff the scripted click still submits the stride.
       MUTATION: `if not obs.may("move"): return` in the move composer → FAIL (and AC-W8 names it).
AC-W8  STATIC "NO RULE IN THE CLIENT" SCAN, `python3 scripts/check_client_rules.py` (stdlib only, file:line
       reports, non-zero on any finding), over `clients/2d/**/*.gd`, comments excluded:
         R1 only scripts/intents.gd calls `submit(` or `submit_affordance(`;
         R2 no .gd file but intents.gd contains a string literal equal to an action type intents.gd
            composes (the composer table's keys, read from intents.gd);
         R3 in intents.gd, no function that submits reads `may(`, `"available"`, `unavailable_reason`,
            `requirement(`;
         R4 `distance_to`, `.length()`, `within` appear only in walker.gd, projection.gd, town.gd and
            scripts/scene/**;
         R5 only scripts/link.gd constructs `MineWorldClient` or calls `connect_to_world`.
       PASS iff clean on the PR head AND each of five plants (one per rule) is reported by file and line.
AC-W9  PATH SCOPE (I-4), `python3 scripts/check_client_rules.py --scope <base>`: every path changed
       between the merge base and HEAD (plus untracked, unignored files) is under clients/2d/**,
       presentation/mineworld-default/2D/**, mineworld-2d, scripts/check_client_rules.py,
       tools/cli/tests/client_2d.rs, tools/cli/tests/client_2d/**, or is Markdown (which includes
       docs/DECISIONS.md, docs/MVP.md, docs/ART_DIRECTION.md, clients/protocol/ADOPTION.md).
       In particular nothing under clients/protocol/mineworld/. PASS on the head; a planted touch of
       server/src/lib.rs and of clients/protocol/mineworld/space.gd each FAIL by name. Fail-closed: no
       git or an unknown base is a failure.
AC-W10 PRESENTATION INDEPENDENCE (I-5). AC-W1's walk is driven five times, each on a fresh world: pack
       `town` (default), `--variant=full`, `--variant=people`, `--variant=procedural`,
       `--presentation=none`. PASS iff the five request transcripts have identical semantic cores, in
       order (`mineworld_server::semantic_core`). The drive scripts targets in world coordinates; the
       click path is covered separately: for each projection, picking the screen point where a world
       point is drawn returns that point within 1 mm.
       MUTATION: the route's doorway approach uses the pack's `doorstep_m` → transcripts differ → FAIL.
AC-W11 EXISTING BEHAVIOUR UNCHANGED (I-8), once on the final head: `cargo fmt --check`, `cargo clippy
       --workspace --all-targets -- -D warnings`, `cargo test --workspace` (13a adds no non-ignored Rust
       test, so the count equals main's); `clients/protocol/run.sh evidence` (AC-13/AC-15 evidence
       regenerates identically apart from instances and timestamps); `./mineworld-slice --drive` PASS.
AC-W12 STABLE PREVIEW (ARC-24, VISUAL_FIDELITY §9.1) — the objective half, all asserted by the drive
       or a capture run, on a clean checkout (`git clean -xfd clients/2d` first):
         - launches and imports with zero lines matching `ERROR|SCRIPT ERROR|Failed loading` on stderr;
         - every role the active binding set references resolves to a loaded texture (0 missing), for
           each of the four variants;
         - a 1.75 m person draws 56 px tall ± 2 px at zoom 1 (32 px/m, the ARC-14 scale);
         - every drawn façade's door anchor lies within 2 px of its disclosed doorway's projected point;
         - on entering the café the façade's alpha reaches 0 within 1.0 s, and the street's people are
           not drawn (I-7: they are not perceived);
         - painter's order: of two people, the one further south is drawn in front.
       Then stills (below) are captured windowed and inspected one at a time by the session.
```

**Stills captured from the connected client** (`./mineworld-2d --drive --capture`, windowed, in the
background), written to `clients/2d/shots/preview/` and committed:
`01_street_wide` (carol leaving the apartments, Ivan on the street), `02_cafe_door`,
`03_cafe_interior` (Alice labelled, Bob, the visitor and the wanderer), `04_ref_framing` (framed as
`presentation/mineworld-default/2D/references/02_cafe_street.png`), `05_plain` (no pack, same moment),
`06_variant_full`.

**What counts as a stable preview, and what the preview asks.** Stable = AC-W12 passes and no still
shows torn sprites, a missing texture, a person drawn outside the observation, or a façade detached
from its doorway — i.e. the next change is refinement, not repair. The preview package (`ARC-20`
item 3: launch command, stills, the reference beside `04_ref_framing`, known misses as facts largest
first, the questions) goes in the PR body. It is labelled **preview**; it can never end in
`ACCEPTED` (`ARC-24`). It asks the operator: *is the connected market-town street, in the `town`
style, the right direction for VIS-2D-1, and what is most wrong?* Known misses stated in advance: from
the street, a neighbouring place's façade is generic until visited (A-23, R-PK-1 is 13c); interiors
other than the café are dressed generically; nobody but the player moves (A-18, R-S10-1); the
interior's footprint is pack decoration, not enforced (§4.3 point 4). `VISUAL_FIDELITY.md` §5's
character categories do not apply (no character reference is claimed); the place category "massing of
a referenced building" does not apply either, because the references are style plates, not this town.
The reference-framed still is reported per §6 with the measured luminance distribution beside the
plate's (mean 0.517, p50 0.512, >0.7 24.2%, <0.2 4.4%) as facts, not as a gate.

### 14.3 How the operator plays it

```text
./mineworld-2d                              build `mineworld` if needed, host worlds/market-town locally
                                            (--agent alice, saved under clients/2d/.save/market-town),
                                            join seat `carol`, play: click to walk, WASD, Esc quits
./mineworld-2d --seat visitor               the same world, as somebody else
./mineworld-2d --world worlds/social-cafe   another world
./mineworld-2d --fresh                      start that world over (deletes its save)
./mineworld-2d --server ADDRESS --invite TOKEN [--seat carol]
                                            join a world someone else is hosting (revision 2, D-9)
./mineworld-2d --variant=full|people|procedural          another ARC-14 binding set
./mineworld-2d --presentation=<dir>|none                 another Presentation Pack, or plain drawing
./mineworld-2d --drive [--capture]                       the scripted checks, headless (stills need a window)
```

`AC-3` by hand: `mineworld server worlds/market-town --agent alice --save /tmp/town` in one terminal,
`./mineworld-2d --server <address> --invite <token>` in another, both copied from the server's
`[mineworld] invite …` line (D-9); close the window mid-walk; reopen; you stand where
the world left you. The launcher builds the Godot class cache on first run (`ACCEPTANCE.md` §4.1). The
operator's checklist for this PR is the one above plus "walk from the apartments into the café".

### 14.4 Design decisions (recorded in C1)

- **ARC-45 — One drawn town from disclosed passages.** §4.3 points 1–3, 5–6 as written, plus: dressing
  (terraces, planters, lamps, trees) anchors only to disclosed doorways or to the extent they span,
  never to a world key or an absolute coordinate; the layout cache is per instance, in memory, dropped
  on an instance change (a cross-launch cache on disk was considered and not chosen: it adds state for
  a convenience R-PK-1 provides properly). An unknown neighbouring place is drawn with a generic façade
  chosen by a stable hash of its id string.
- **ARC-46 — A Presentation Pack binds roles; the client names no asset.** The pack directory is loaded
  at runtime from any path (`--presentation`), never imported into the Godot project:
  `Image.load_from_file`, `Image.load_svg_from_buffer`, shaders from source text. Files:
  `assets/asset_bindings.yaml` (variants → role → file; `town` default; `full`, `people`, `procedural`)
  and `renderer/godot.yaml` (projection kind, px per metre, sprite scale rule, shader files, façade
  dressing offsets, interior footprints per tag). Both are written in **YAML's JSON-compatible subset**
  so Godot's built-in `JSON` reads them and any YAML tool reads them too; their schema is specified in
  `clients/2d/PRESENTATION.md`, and `MODULE_SPEC.md` §6.1 gains one pointer line. No GDScript lives in
  the pack. Role vocabulary: `person:<n>`, `player`, `facade:<tag>`, `facade:unknown:<n>`,
  `ground:<tag>`, `interior:<tag>:<fitting>`, `prop:<name>`, `effect:<name>`. An unbound role draws
  plainly.
- **ARC-47 — No rule in the client, executable.** §8.1 I-1 and §4.4's rules, with AC-W5 … AC-W8 as the
  test. 13b extends the same scan and stub; it adds no new principle.
- **DEP-16 — Godot built-ins and the shared module; no addon.** §5's verdicts, re-checked 2026-10-08,
  plus the pack-file reading decision: YAML addons exist — the GDExtensions
  [fimbul-works/godot-yaml](https://github.com/fimbul-works/godot-yaml) and
  [KoBeWi/Godot-YAML](https://github.com/KoBeWi/Godot-YAML) (both RapidYAML) and the pure-GDScript
  [YAML.gd](https://godotengine.org/asset-library/asset/4120) — but a GDExtension is native per platform
  (§5.3's objection to gdext), and a GDScript YAML parser is an unvetted dependency for two small files
  we author. The JSON subset costs only comments, which the spec file carries instead. Revisit if a
  pack author needs full YAML.

### 14.5 The art (QS12-10)

From `origin/vis/2d-generated-assets` @ `af5e236`, read only, never merged: `art/generated/` (5.96 MB,
the `gen_*` cast and `sib_*` normalized shared set), `art/ground/` (8.07 MB), `art/interior/` (1.17 MB),
`art/svg/` (2.53 MB), `art/paper_grain.png` (0.64 MB), `art/props.json`, `art/generated/generated.json`,
`art/grade.gdshader`, `art/ink.gdshader` → `presentation/mineworld-default/2D/art/` and `renderer/`.
About 18.4 MB; together they cover all four `ARC-14` variants. **Not carried:** `art/candidates/*.png`
(81.7 MB), `art/style_refs/`, `screenshots/`, Godot `.import` sidecars, the spike's scripts and tools.
Provenance (DEP-8, ARC-9): the 66 `art/candidates/*.json` sidecars (≈ 130 KB) are carried as
`art/provenance/`; `sib_*` derive from `2D/candidates/*.provenance.yaml` already on `main`; the paper
grain's CC0 terms go to `presentation/mineworld-default/LICENSES/`; `art/PROVENANCE.md` states each
group's origin, the tools that produced it, and the spike commit. ARC-22's cast range is unaffected
(no regeneration).

### 14.6 Commit plan

#### C0 — Design (this section) — Markdown only
- [x] Implementation: §14, from the audit in §2 re-verified at `47c81d1`, the spike at `af5e236`, the
  rulings and the coordinator's 16b names.
- [x] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py`.
- [x] Review: every module reference uses 16b's names or none; no edit to `clients/protocol/mineworld/`
  is planned; decision ids in ARC-45…47 / DEP-16 only.

#### C1 — Specs before code
**Goal.** The decisions and the pack format exist before code relies on them (`CLAUDE.md` §2.2).
**Scope.** `docs/DECISIONS.md`: `ARC-14` ported verbatim from `af5e236` with a dated note (2026-10-08:
laid out from market-town's street, QS12-1), `ARC-45`, `ARC-46`, `ARC-47`, `DEP-16`.
`docs/MVP.md` §7.1 and `docs/ART_DIRECTION.md` §20: "top-down" → "isometric (the default pack) or
top-down". `docs/MODULE_SPEC.md` §6.1: one pointer to `clients/2d/PRESENTATION.md`.
`clients/2d/PRESENTATION.md` (new spec): the two files' schema, the role vocabulary, runtime loading,
fallback. `clients/protocol/ADOPTION.md` §6: the reconnect pattern (M-4) as guidance.
- [x] Implementation: as scoped. ARC-14 inserted after ARC-13 (numeric adjacency, filling `ARC-16`'s
  gap); ARC-45, ARC-46, ARC-47, DEP-16 appended. `clients/2d/PRESENTATION.md` specifies both pack
  files, the role vocabulary, doorway-relative anchoring, interiors, outdoor places, edges, plain
  drawing. `ADOPTION.md` gains §6.1 and a pointer in §6's table.
- [x] Validation: `check_decision_ids` → 56 ids, all distinct (51 + 5); `check_doc_headings` → 176
  sections, none duplicated; each new heading present exactly once.
- [x] Review: ARC-14 extracted from `af5e236` and from this tree and `diff`ed → identical (the note
  follows it). No defined term redefined. `ADOPTION.md` touched in §6 only (16b edits §2).

#### C2 — The pack's art, bindings and renderer parameters
**Scope.** §14.5's files; `assets/asset_bindings.yaml` (four variants); `renderer/godot.yaml`;
`art/PROVENANCE.md`; LICENSES file. **Non-goals:** no client code.
- [x] Implementation: files extracted with `git archive af5e236 -- <paths> ':(exclude,glob)**/*.import'`
  (no merge, no cherry-pick). Sprite metrics converted by a one-off GDScript (`/tmp`, not committed)
  from the spike's `props.json`, `generated.json` and `Main.gd`'s `SCALE`/`VEG_M`/ink classes into
  `assets/asset_bindings.yaml` (120 sprites + 2 textures; four sets). `renderer/godot.yaml` (palette from
  the spike's `Ground.gd`/`Interior.gd`, effects, dressing, outdoor `park`, interiors `*`/`cafe`/
  `apartments`, edges). `art/PROVENANCE.md`; 54 sidecars in `art/provenance/`;
  `LICENSES/PUZZLEANDY_WATERCOLOR_TEXTURES.txt`. Door pixels of the four shared-set buildings read off
  the images (each viewed) into `door`.
- [x] Validation: `python3 scripts/check_client_rules.py --check-pack presentation/mineworld-default/2D`
  → PASS; mutation: `gen_a_front.png` renamed → FAIL, 2 findings (the missing file, the unbound one),
  restored → PASS. Size: `art/` 7.4 MB, `renderer/` 20 KB, `assets/` 24 KB (D-2). No `.import` file
  (the check refuses one).
- [x] Review: `town` = spike `ROLE_PEOPLE` + `ROLE_SHARED`, `full` = `ROLE_PEOPLE` + `ROLE_WORLD`, `people`
  = `ROLE_PEOPLE`, `procedural` = svg with `ROLE_PROC`'s five-person cast, compared role by role;
  `barrel` is the role `prop:bin`. New roles with no spike equivalent (the spike had no apartments):
  `facade:apartments`, `facade:residential`, `facade:unknown:0/1`, bound in `town` only; the other sets
  draw those plainly. Shopfronts with painted signs (bakery, books, bloom) are deliberately not bound:
  drawing a convenience store as "Bakery" would state a falsehood. Every carried `gen_*` image has its
  sidecar; `sib_*` map to `candidates/*.provenance.yaml` on `main`.

#### C3 — Client skeleton: connect, see, draw plainly and with the pack
**Scope.** `clients/2d/{project.godot, .gitignore, README.md, scenes/app.tscn}`, symlink
`mineworld -> ../protocol/mineworld`; `scripts/app.gd` (arguments, wiring), `scripts/link.gd` (join,
status; reconnect in C6), `scripts/presentation.gd` (loader, roles, fallback), `scripts/projection.gd`
(iso 2:1 at 32 px/m over `MineWorldSpace.to_2d`; plan for no pack; inverse for picking),
`scripts/scene/{people,places,plain}.gd` (current place only), `scripts/hud/status.gd`,
`scripts/harness/drive.gd`; the `mineworld-2d` launcher.
- [x] Implementation: as scoped (landed with C4 and C5's client code in one commit, D-6). Every file
  under 330 lines (largest `scene/places.gd` 324, `harness/drive.gd` 307). The plain shapes live in
  `scene/plain_shape.gd`; sprite mechanics (anchor, door, scale, ink) in `scene/sprites.gd`; the grade
  pass in `scene/grade.gd`; stills in `harness/capture.gd`. The launcher starts the server with
  `--listen 127.0.0.1:0` and reads the bound address from its first line, and kills only the PID it
  started (the coordinator's 16b finding 1: no fixed port, no `pkill` by name).
- [x] Validation (E-3, on the working tree at `38dfe93` + this commit's files): `./mineworld-2d
  --drive=seated` → PASS (carol and Otto drawn at 0.0000 m, labelled "Carol Mensah"/"Otto Brandt",
  heights 58.0 / 57.0 px against 58.2 / 57.0 intended, 48 roles resolve, no load error); the same with
  `--variant=full|people|procedural` → PASS (44, 44, 40 roles); `--presentation=none` walks → PASS.
- [x] Review: `grep -rn "art/\|\.png\|\.svg" clients/2d/scripts` → none (the core names no asset). Ids are
  kept as strings throughout (`town`, `people`, `walker`); every number sent comes from
  `town.local_in` (int) or `MineWorldSpace` (int) — the 16b D-2 float trap cannot arise for `move`.

#### C4 — One town: frames glued at passages, façades, interiors, people
**Scope.** `scripts/town.gd` (ARC-45; layout cache per instance), `scripts/scene/places.gd` (ground by
tag, façades at doorways, generic unknown façade, cut-away and dimming), `scripts/scene/people.gd`
(sprites by tag and id hash, labels, smoothing, distance-driven gait, painter's order).
- [x] Implementation: as scoped, plus `scene/ground.gd` (paving, lawns, cut-away rooms — the spike's
  `Ground.gd`/`Interior.gd` generalized to any rectangle and orientation). A doorway's "in" direction
  is the dominant axis from the hub extent's centre to the doorway; a far-side place is a façade that
  lifts, a near-side place a cut-away room (so nothing tall hides the street), an `outdoor` tag a lawn.
  The drive scenario is `walk` (the design's `look` merged into it). `SHOWN` lines report what is drawn
  as data (the coordinator's "one world, two views" rule, for 13f).
- [x] Validation (E-4): `./mineworld-2d --drive=walk` → PASS: façades on their doorways 0.00 px (2
  checked from the street: the apartments' and the unknown neighbour's); façade gone 0.28 s after the
  place changed (bound 1.0 s); in the café all five people drawn at 0.0000 m, labelled by name, heights
  within 0.2 px of intended; only perceived people drawn. Click path (`--drive=click`, both
  projections): miss 0.0000 m.
- [x] Review: no place is drawn by key or by an absolute coordinate (`places.gd` positions everything
  from `town.to_plan` of a disclosed doorway or the hub extent); nobody unperceived is drawn (the drive
  checks drawn ⊆ listed). Under the 2026-10-08 "one world, two views" rule: every façade, room and
  lawn stands at a disclosed doorway of a real place; dressing props carry no simulation meaning;
  nothing invents a place or a door.

#### C5 — Walking: intents, walker, the scan, the stub
**Scope.** `scripts/intents.gd` (the `move` composer; the only `submit`), `scripts/walker.gd` (click
route in ≤ 1.9 m strides, one in flight; doorway routing `here` → `there`; WASD reporting every 0.5 m
or 20°; reconciliation at > 150 mm; a rejection ends the walk); `scripts/check_client_rules.py`
(R1–R5, `--scope`, `--check-pack`); `tools/cli/tests/client_2d.rs` + `tools/cli/tests/client_2d/`
(Godot spawn helper, rev-1 stub).
- [x] Implementation: client side in `74b7c2c` (D-6); the scan in C2 (D-3); `tools/cli/tests/client_2d.rs`
  (seven `#[ignore]`d tests) and `tools/cli/tests/godot2d/mod.rs` (Godot run, parse guard F-4, a world
  restartable on one address, the revision-1 stub) in the C5/C6 commit. Two walker fixes found by these
  tests: F-2 (stride count) and F-5 (request facing must not depend on frame timing, below).
- [x] Validation (E-5, `cargo test -p mineworld-cli --test client_2d -- --ignored --test-threads=1`,
  7 passed, 132.7 s, Godot 4.7.2, macOS arm64):
  - AC-W1 `walks_from_the_apartments_into_the_cafe`: 14 moves, all accepted; the save holds two
    `person-entered-place` for carol (street, then café) and 14 action-caused `arrived` facts.
  - AC-W5/W6/W7 against the stub: 3 submits then none after the refusal, drawn at the stub's position,
    reconciled within ≤ 1 frame; the teleport followed with no request after it; an unavailable `move`
    submitted.
  - AC-W8 (rules): clean; five plants (R1 submit and R2 `"move"` in `walker.gd`, R3 `may(` in
    `intents.gd`, R4 `distance_to` and R5 `MineWorldClient.new()` in `app.gd`) → FAIL, 5 findings, each
    by file and line; reverted → PASS.
  - AC-W10: five transcripts (town, full, people, procedural, none) identical by
    `mineworld_server::differing_fields`; the click path in both projections, miss 0.0000 m.
  - Mutations, each run then reverted (grep for the markers finds none): W1 crossing to `here` → the
    drive fails "the world refused a stride" (red); W5 rejection treated as accepted → 6 submits, not 3
    (red); W6 a correcting move after reconciling → "no move answered the teleport" (red); W7
    `may("move")` guard → 0 submits (red; R3 also names it); W10 route offset by the projection's px/m
    → "--presentation=none request 0 differs" (red).
- [x] Review: every distance in `walker.gd` is a stride size (≤ 1.9 m), a route through a disclosed
  doorway, the 150 mm reconciliation, the 2 mm stale-frame match or drawing; `resolved()` never
  resubmits; a refusal or rejection ends the walk; `abandon()` drops the route and the step.

#### C6 — Reconnect and AC-3 end to end
**Scope.** `link.gd`: §4.6 points 1–5 (back-off 0.5 s doubling to 8 s; same seat; instance compare;
`refused` final except `world_stopped`; abandon the walk; unanswered request reported "unknown").
Tests AC-W3, AC-W4 in `client_2d.rs`.
- [x] Implementation: `link.gd` retries with back-off 0.5 s doubling to 8 s, the same seat;
  `unknown_seat`, `seat_not_in_world` and `already_joined` end it, `world_stopped` does not; `app.gd`
  abandons the walk, dims the world while disconnected, says when a request went unanswered, and
  resets the town only when the welcome names another instance. Drive scenarios `street` and `idle`.
- [x] Validation (E-5): AC-W3 `killed_mid_walk_and_relaunched` — SIGKILL after the first accepted street
  stride; meanwhile the wanderer walks up to Alice and talks (her agent answers) and a second
  connection moves carol one stride; `/status` revision rose; the relaunched client is told the same
  instance and a revision ≥ the post-kill one, draws carol exactly at the moved-to millimetres, and never
  reconciles. AC-W4 `reconnects_to_a_restarted_world_and_to_a_replaced_one` — server SIGKILLed mid-walk
  and restarted on its save and address: STATE reconnecting → seated, same instance, 6 learned
  doorways kept, no request after the drop; then a fresh world on that address: 1 doorway known.
  Mutations: W3 remembered position at startup → red only after the test was strengthened (F-3); W4
  replay of the unanswered stride, held until seated → "nothing was submitted after the drop" (red);
  without the hold the plant was neutralized because the module refuses to submit when not seated
  (two defences; recorded, not a weakness of the test).
- [x] Review: nothing is replayed (`abandon()`); the layout cache is cleared only in `_on_welcomed` on a
  different instance; a final refusal stops retrying; the module stays policy-free (`ADOPTION.md` §6.1).

#### C7 — Stills, preview package, final gates
- [ ] Implementation: capture mode; stills committed under `clients/2d/shots/preview/`; README;
  ledger and handoff.
- [ ] Validation: AC-W9, AC-W11 and AC-W12 on the final head; stills viewed one at a time and the
  findings recorded as facts.
- [ ] Review: the preview package complete per `ARC-20`; known misses stated largest first; no
  comparative without a fact (`ARC-17`).

### 14.7 Files touched

```text
new      clients/2d/** (project.godot, .gitignore, README.md, PRESENTATION.md, scenes/, scripts/,
         scripts/scene/, scripts/hud/, scripts/harness/, shots/preview/, symlink mineworld)
new      mineworld-2d
new      presentation/mineworld-default/2D/{assets/asset_bindings.yaml, renderer/, art/}
new      presentation/mineworld-default/LICENSES/<paper grain CC0>
new      scripts/check_client_rules.py
new      tools/cli/tests/client_2d.rs, tools/cli/tests/client_2d/
docs     docs/DECISIONS.md, docs/MVP.md §7.1, docs/ART_DIRECTION.md §20, docs/MODULE_SPEC.md §6.1 (one
         line), clients/protocol/ADOPTION.md §6, this file
NOT      clients/protocol/mineworld/**, server/, kernel/, contracts/, systems/, worldpack/, persistence/,
         cognition/, worlds/, sdk/, authoring/, clients/3d-spike/, clients/protocol/demo/, Cargo files
```

### 14.8 Test ownership

```text
STATIC      cargo fmt/clippy (the Rust test code); check_client_rules.py R1–R5, scope, pack files
UNIT        none new: the client's checks are against a real or stub server, not helpers
INTEGRATION Godot-gated: AC-W1 … AC-W7, AC-W10 (real server, real save, or the rev-1 stub)
REAL RUN    the windowed capture (stills inspected); AC-W11's protocol evidence and slice drive
GATE 1      NOT REQUIRED — no model
GATE 2      AC-W1, AC-W3, AC-W4 are the real-lifecycle evidence (real processes, SIGKILL, real save)
CI          none on main today; if S13's 13a lands first, its CI runs on the PR. Godot-gated tests are
            not run by CI (no Godot layer yet, QS12-8) and are reported NOT RUN there
```

### 14.9 Execution contract

```text
PROJECT / PR:            S12 PR 13a — a world you can walk
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/step-13-client-2d.md §14 (live ledger)
RELATED / BINDING DOCS:  overall.md "Parallel build-out, 2026-10-08"; this step §§1–13; CLAUDE.md;
                         ENGINEERING_RULES, ENGINEERING_STANDARDS, ART_DIRECTION, VISUAL_FIDELITY,
                         ACCEPTANCE; server/PROTOCOL.md; clients/protocol/ADOPTION.md; ARC-14 (af5e236)
IMPLEMENTATION BASE:     main @ 47c81d1; rebase onto main before the PR if S11-A or 16b merged
APPROVED SCOPE:          §14.0–§14.7
FROZEN INVARIANTS:       I-1 … I-9 (§8.1); no edit to clients/protocol/mineworld/**; no server, kernel,
                         contract, System Pack or World Pack change; only ARC-45…47 / DEP-16 (+ ARC-14
                         port); the art carried is §14.5's set only; the spike branch is not merged
APPROVED SEQUENCE:       C0 → C7 (§14.6)
VALIDATION BUDGET:       cargo and Godot runs unrestricted when bounded; each Godot run ≤ 10 min, anything
                         > ~2 min in the background; full cargo gate once on the final head; total
                         ≈ 1 h of validation wall time; no paid API, no model
LIVE DOCUMENTATION:      this section; CONTEXT HANDOFF: .structured-coding/plans/mvp0/handoff.md, 13a
                         block only (other efforts' blocks preserved)
ENDPOINT AUTHORITY:
  implementation + local validation   authorized after freeze — source: primary session's kickoff
                                      ("Phase 2 — after my freeze message")
  semantic commits                    authorized — source: same, and working rules §14
  branch push                         authorized — source: D-12; kickoff
  PR creation / update                authorized, marked READY FOR OPERATOR REVIEW — source: kickoff
  CI repair to review readiness       authorized — source: D-12
  merge                               explicit operator authorization only. Not by this session.
MATERIAL STOPS:          any edit to the shared module; any server, kernel, contract or pack (System or
                         World) change; a falsified AC above; S11-A's join shape needing more than a
                         consumer change
POST-MERGE SYNC OWNER:   this session: §14's lifecycle and evidence; the primary session: step header,
                         overall §7, MVP_STATUS, HUMAN_REVIEW_QUEUE VIS-2D-1
NORMAL STOP:             PR 13a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

### 14.10 Deviations and discoveries during implementation (13a session)

- **D-1 (bounded) — the handoff is its own file.** *Design:* a 13a block in `handoff.md`. *Reason:* six
  lanes run in parallel and each would append to that file, guaranteeing merge conflicts.
  *Resolution:* `handoff-s12-13a.md`. *Impact:* none on scope.
- **D-2 (bounded) — less art carried than §14.5 listed (7.4 MB, not ≈ 18.4 MB).** *Previous
  assumption:* `art/ground/` (8.07 MB) and all of `generated/` and `svg/` are part of the accepted look.
  *Audit evidence:* `grep -rn "ground_\|art/ground" clients/2d-spike/scripts/` at `af5e236` finds
  nothing — the spike draws its ground procedurally (`Ground.gd`) and no variant loads those textures
  (their sources are already on `main` as `2D/candidates/ground_*`). The dog, cat, seated people,
  `char_test_walk_e`, the signed shopfronts (bakery, books, bloom), the quay set (rail, jetty, boat,
  bird, fountain) and three planted props bind to nothing the connected client draws. *Corrected
  understanding:* RK-1's rule — carry only what the bindings reference — is the binding one, and
  §14.5's list over-counted. *Consequence:* 120 sprites + 2 textures; `--check-pack` refuses a carried
  file that no sprite names, so the rule is held mechanically. Ground is drawn procedurally from the
  palette, as in the spike.
- **D-3 (bounded) — `check_client_rules.py` lands in C2, not C5,** because its `--check-pack` mode is
  C2's validation. R1–R5 and `--scope` are exercised with their plants in C5.
- **D-4 (bounded) — façade door pixels.** A building sprite's anchor is its lowest base corner, not its
  door; aligning anchors would put doors up to 22 screen px off their doorways. The four shared-set
  buildings carry a measured `door` pixel and are placed by it; sprites without one are placed by the
  anchor (as the spike did). `PRESENTATION.md` §3 gains the optional field, and `height_m` (read only by
  the drive's size check).
- **D-5 (bounded) — the child sprite is not in the cast.** `gen_i` is 1.28 m tall. Assigned by id hash
  it drew Alice, an adult barista, as a child (seen in the first interior still): an age the world
  never stated. The cast is the other nine; `gen_i`'s images and sidecars are not carried.
- **D-6 (bounded) — C3, C4 and C5's client code land as one commit.** The composition root wires every
  part, so no subset runs; the evidence is recorded per acceptance item, and the scan, the Rust tests
  and reconnect remain separate commits.
- **D-7 (bounded) — 16b merged mid-PR (`e1ec5ff`, merged into this branch as `38dfe93`).** As §14.1
  anticipated, the status line now reads `MineWorldClient.revision`; `link.gd` keeps the welcome's
  revision for the AC-3 evidence. `ADOPTION.md` merged cleanly (16b's §2, this PR's §6.1).
- **F-1 (found by the click check, fixed) — a room with one door was taken for the hub.** `town.hub()`
  returned the only place with passages, the apartments, so its single passage was drawn as a "doorway
  out of the hub" with a footprint north of the door, and a click inside the apartments was routed out
  into the street (miss 1.63 m in both projections). A hub now needs at least two doorways; before one
  is known the observer's room is drawn from inside. Click miss after the fix: 0.0000 m.
- **D-8 (bounded) — the Rust support directory is `tools/cli/tests/godot2d/`, not `client_2d/`.** Rust
  refuses a module that has both `client_2d.rs` and `client_2d/mod.rs`. `--scope` allows the new path.
- **F-2 (found by the stub, fixed) — float noise added a stride.** `ceil(3.8 / 1.9)` is 3 in floats
  (3.8 / 1.9 = 2.0000000002), so a two-stride walk sent three requests; with a teleport after the
  second, the extra stride overwrote it and AC-W6 failed for the wrong reason. `_strides` subtracts
  1e-6 before `ceil`.
- **F-3 (found by mutation, test strengthened) — AC-W3's planted "remembered position" survived.** The
  plant restored the dead client's last accepted position at startup; the reconciliation rule then
  corrected it within one observation, so the final drawn position was right and the test passed. The
  property was held by a second mechanism, which is good architecture and a weak instrument
  (`ARC-23`). The test now also requires that the relaunched client never reconciles (a client placed
  from its first observation has nothing to correct); with the plant it fails on exactly that line.
- **F-4 (found, guarded) — a script that does not parse hangs a run.** A stray tab left by a reverted
  mutation made `walker.gd` unparseable; Godot then ran an empty scene until the 150 s run limit,
  printing nothing. The test support now parses `app.gd` (and through its preloads every script) with
  `--check-only` before the first run and fails at once, naming the error.
- **D-9 (bounded, anticipated by §14.1) — S11-A merged first (`f842c52`, merged here as `d6bd41d`).**
  The one `connect_to_world` call (`link.gd`) passes `invite` and `nickname` (revision 2); `link.gd`
  also treats `unauthorized`, `protocol_mismatch`, `invalid_nickname` and a `closing` of `left` as
  final. The launcher starts the server on port 0 with no `--invite`, reads its one join line
  (`[mineworld] invite <token> — join with: <address> …`) as `mineworld-slice` does, passes the invite
  to the client and never echoes it; `--server ADDRESS` needs `--invite TOKEN` (or `MINEWORLD_INVITE`).
  The Rust tests start every world with `support::INVITE` and the stub checks the client's revision-2
  join (protocol 2, the invite). No committed file holds an invite: the drive prints none and the
  server log stays in a scratch or git-ignored directory. `git grep connect_to_world` shows only the
  four-argument form. Merge conflicts: `DECISIONS.md` (both sides' records kept; the 12c note stays
  with DEP-13) and `ADOPTION.md` §6 (S11-A's wording plus this PR's §6.1 pointer).
- **D-10 (bounded) — the drive derives every waypoint from the disclosure (12d's QD-11).** 12d moves
  the café's doorway from (0, 3000) to (0, 2800). The drive no longer holds market-town coordinates:
  out through the starting place's doorway and 1.5 m toward the middle of the street, to the nearest
  other doorway of the street, 2.5 m in. AC-W1 now also checks that the place entered is tagged
  `cafe`, read from the observation.
- **F-6 (found in the `SHOWN` report, fixed) — the corner store was drawn as a near-side room.** A
  doorway's "in" direction was the dominant axis from the hub extent's centre; the store's doorway, at
  the extent's north-east corner, came out "east" (|dx| 12 m against |dy| 8 m) and was drawn as a
  cut-away room instead of a façade. Doorways line the long sides of a street, so the direction is now
  across the extent's long axis: all three north-side doorways open north, both south-side ones
  south. Still derived only from disclosed doorways (`ARC-45`).
- **D-11 (bounded) — `--always-on-top` for capture runs.** One capture stalled for ten minutes after its
  first still: a window the OS treats as hidden is not drawn, and `frame_post_draw` never arrives. The
  launcher keeps the capture window on top; stalled runs were killed by PID and re-run.
- **F-5 (found by AC-W10, fixed) — a request's facing depended on frame timing.** Transcripts differed
  in `yaw` by 1–8 millidegrees between runs: the facing was taken from the drawn body (which is within
  2 cm of its goal, at a frame-dependent point) and, at a crossing, from a near-zero vector (`here` and
  `there` are one point). The facing asked for is now the direction from the last position asked for,
  kept unchanged across a crossing. A second, unconfirmed cause was guarded too: a frame the server
  computed before applying an accepted move, arriving after its result, would snap the body back and
  bend the next stride's direction. Frames showing exactly the pre-move position are ignored for
  600 ms after an accepted result and counted (`stale_frames_ignored`); in the five AC-W10 runs after
  the fix the count was 0, so the race was not observed — the guard is recorded as defensive.

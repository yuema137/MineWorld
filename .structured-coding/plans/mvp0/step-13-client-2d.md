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

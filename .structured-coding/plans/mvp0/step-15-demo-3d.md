# Step 15 — Demo B: the 3D walking world (S14), with S15's PR 12e (the 3D client's bodies)

**Role:** step document for **S14 — Demo B: the 3D walking world** (`overall.md` §3), and the detailed
plan of **S15's last PR, 12e** (`step-11-bodies.md` §11.1), which builds the same client. It records the
requirement, an audit of the real client and protocol source, the `AC-14` and `AC-13` gap analysis, the
design, a reuse comparison for every non-trivial piece, the modularity argument, the requirements this
step places on S11 (the wire protocol) and on S15 12d (the town's geometry), the changes to the shared
GDScript protocol module, invariants with an executable "no rule in the client" check, a PR split with
integration checkpoints and adversarial criteria, risks, and open questions.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S14, S15), §4, §7.
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)` — frozen at step level by the primary session under the operator decisions and coordination rulings in `overall.md` "Parallel build-out, 2026-10-08", which bind and override this document where they differ (decision numbers, protocol ownership, event perception, the shared module, digests). Superseded wording below: `DRAFT — awaiting the primary session's review`. Nothing here is frozen and nothing here
authorizes implementation. Each PR is detailed to the commit and frozen in turn (`CLAUDE.md` §3.1).
**PR 16a:** detailed to the commit in §18, `DESIGN FROZEN (2026-10-08), primary session`.
**Branch:** `plan/s14-3d`, from `main @ 0fd0be3`, worktree `/Users/yuema137/mineworld-worktrees/plan-s14-3d`,
held by this planning session only.
**Written in parallel** with the S11, S12, S13 and Milestone E planning sessions, while 12c is being
implemented. Parallel-safety rules followed:

- This file is the only file this session writes. Edits it needs elsewhere (`overall.md`,
  `step-11-bodies.md`, `docs/MVP_STATUS.md`, `docs/DECISIONS.md`, `server/PROTOCOL.md`,
  `clients/protocol/ADOPTION.md`) are **proposed** in §17, never applied here.
- Decision identifiers are placeholders: `ARC-S14-a`, `ARC-S14-b`, `DEP-S14-a`. The primary session
  assigns real numbers at freeze. `DEP-14` keeps the name `step-11-bodies.md` §15.2 already gave it.
- **S11 owns the wire protocol.** What this step needs from it is stated in §11 as requirements with
  exact shapes, never as a design of S11's frames.
- **The shared module `clients/protocol/mineworld` is used by 2D and 3D.** Every change this step plans
  to it is listed in §12, so the primary session can coordinate it with S12.

## Why the file is `step-15`

Step files are numbered in the order they are written (`step-11-bodies.md`, "Why the file is
`step-11`"). Other planning sessions are writing S11, S12 and S13 step files at the same time, and the
coordinator assigned this one `step-15-demo-3d.md`. PR numbers follow the file's number by the
established convention (`step-09` → PR 10, `step-10` → PR 11, `step-11` → PR 12), so this step's own PRs
are **16a … 16e**. The primary session may renumber at freeze; every reference here is to the letter as
much as to the number. 12e keeps its S15 name, because S15's acceptance closes with it.

---

# 1. The requirement and the acceptance, quoted

## 1.1 S14, as `overall.md` §3 states it

> ### S14 — Demo B: 3D walking world
>
> - **Output:** the embodied first-person reference client: movement, camera and look controls,
>   collision and basic navigation, a walkable environment, an enterable `Place`, physically
>   represented NPCs, spatial targeting, and interaction with at least one object. Low fidelity is
>   expected and acceptable.
> - **Depends on:** S11, and on the spatial contract from S2 having survived the §11 gate.
> - **Acceptance checkpoint:** `AC-14` — a player walks through the environment, enters a `Place`,
>   approaches an NPC, spatially initiates a conversation, and interacts with an object, with all
>   state server-authoritative; and `AC-13` — the `Talk` this produces is the *same*
>   `ActionIntent`, resolved by the same system, as the one Demo A produces by clicking. Validated
>   by actually running the client (`ENGINEERING_RULES.md` §19).
> - **Adversarial criterion:** no business rule exists in either client. Removing the 3D client
>   changes no system, and the diff that added it touches no kernel contract. If the two clients
>   needed separate rule implementations, the architecture is wrong (§9).

## 1.2 The acceptance criteria, `docs/MVP.md` §9

> **AC-13** 2D / 3D semantic parity — `Talk` initiated by clicking an NPC in Demo A and by approaching,
> looking at, and pressing interact in Demo B produce an **identical semantic core** — `actor`,
> `action_type`, `target`, payload — resolved by the same system to the same result. Neither client
> implements any validity rule: distance, availability, permission and willingness are all decided
> server-side.

> **AC-14** Embodiment — In Demo B a player moves in first person through a walkable environment,
> enters a Place, approaches an NPC, spatially initiates a conversation, and interacts with one object,
> with all state authoritative on the server.

`AC-15` (one Alice across a 2D client, a 3D client and an agent on one server) is assigned to "S14,
against S11's server with S12's client also connected" by `overall.md` §4. It already holds in its
minimal form (Milestone A, S5V). This step re-demonstrates it with the real clients in 16e and claims
nothing more for it.

## 1.3 Demo B's list, `ENGINEERING_RULES.md` §10 (binding)

```text
first-person or embodied player movement
camera/look controls
collision / basic navigation
walkable environment
enterable Place
NPC represented physically
approach NPC
spatially initiate conversation
interact with at least one Item/Object
human + AI-controlled Persons sharing the same world
server-authoritative state
```

> The first 3D demo should prioritize **interaction correctness and embodiment**, not graphics quality.

`ENGINEERING_RULES.md` §19: *"For rendering-related work, actually run the relevant reference client. Do
not infer that the 3D experience works merely because the server tests pass."* `overall.md` §1 makes
that the one exception to "every criterion by an automated test": `AC-14`'s embodiment claim is
demonstrated by running the 3D client.

## 1.4 S15's 12e, as `step-11-bodies.md` §11.1 froze it

> **12e** The 3D client | Jolt selected explicitly; people and objects get colliders at observed
> positions (shapes from disclosure); reconciliation on difference (150 mm), which now also covers being
> nudged; `kick`, `throw`, `shove` from the camera ray; objects animated along `path`;
> `server/PROTOCOL.md` §6.2 and `ADOPTION.md`. After `vis/3d-godot-2-environment` merges (QB-13). |
> The client run for real (`ENGINEERING_RULES.md` §19): a scripted drive walks the player into Alice,
> who is nudged aside; into a crowd, which blocks; into a box, which moves; kicks it; is shoved; frames
> inspected, and the server's facts match what is drawn. | No rule in the client: with its people
> colliders removed, the server still resolves and the client reconciles.

The client half of S15's design (`step-11-bodies.md` §7.1, verbatim in its essentials):

> 1. **Select Jolt explicitly**: `[physics] 3d/physics_engine="Jolt Physics"` in `project.godot`.
> 2. **Collide with people.** Every drawn person gets an `AnimatableBody3D` with the same capsule the
>    server uses (r 0.30 m, 1.72 m), on a new layer `LAYER_BODIES` […]. The player's `collision_mask`
>    gains `LAYER_BODIES`.
> 3. **Loose objects.** Drawn at their observed position; collision layer `LAYER_OBJECTS`, which the
>    player does **not** mask in the first version […]. Local push prediction with a Jolt `RigidBody3D`
>    is QB-7.
> 4. **Reconcile on difference, not only on refusal.** When an observation places the observer more than
>    **150 mm** from where the client last reported it […], the client moves its body to the
>    authoritative position. […] It replaces the refusal-only reconciliation in `slice_link.gd`.
> 5. **Animate objects.** An `object-moved` with a `path` is drawn along it over its stated duration.

Operator decisions that bind this step and may not be changed here: QB-1 (resolve before recording),
QB-10 (walking nudges people), QB-7 (no local `RigidBody3D` push prediction in 12e), QB-13 (12e after the
visual slice merges — it has, as #50), the frozen 12c design (`step-11-bodies.md` §18: kick and throw are
**target-less with the object in the payload**, QO-6; objects are offered only within reach, QO-7;
`LooseObjects` is disclosed on the Place, QO-2), and the visual acceptances `VIS-3D-GODOT-1` (route D+,
interim standard) and `VIS-3D-GODOT-2` (the slice, accepted after the operator played it).

## 1.5 The operator's binding directive of 2026-10-08 (relayed by the coordinator)

> 保持代码干净整洁，模块化，可插拔。遇到问题要积极寻找已有的开源方案，不要重复发明轮子，而且要对比多种实现方案，
> 不要找到一个就觉得万事大吉。当然如果没有现成的能满足我们的大目标的，那我们自己改进或者发明轮子也是可以的

English rendering: keep the code clean, modular and pluggable; look for existing open-source solutions
first; compare several implementations rather than stopping at the first; build our own only when
nothing existing serves the goal. §8 is the reuse comparison this requires; §9 is the modularity and
pluggability argument.

---

# 2. Audit — the real source (`main @ 0fd0be3`, 2026-10-08)

Every finding was read in this session from the file named, or measured by the command named.

## 2.1 The 3D client: `clients/3d-spike`

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-S14-1** | **The slice is the 3D client.** `./mineworld-slice` opens `res://scenes/slice.tscn` (`SliceMain`): lighting rig, `SliceWorld.build`, `SlicePlayer`, HUD, and — with `--server=` — `SliceLink`. `--world` builds `mineworld-cli`, starts `mineworld server worlds/social-cafe --listen 127.0.0.1:7979 --agent alice`, and connects as seat `visitor` (`SliceLink.seat_from_args`). `./mineworld-3d` is the older promenade spike in the same project, kept working untouched. | `mineworld-slice`; `scripts/slice/slice_main.gd:49–58, 363–382`; `slice_link.gd:157–182` | Demo B is built **into the slice**, not as a new project: it is the accepted scene (`VIS-3D-GODOT-2`) and already the connected client. No second 3D client is created (§9). |
| **F-S14-2** | **No physics engine is selected.** `project.godot` has no `[physics]` section. A headless probe in this session (`godot --headless --path clients/3d-spike --script <probe>`, Godot 4.7.2) printed `physics/3d/physics_engine setting: 'DEFAULT'`. By the 4.6 release note step-11 §3.2 quotes ("Existing projects aren't affected"), `DEFAULT` is taken to be Godot Physics; the 4.7 Jolt page says only that "new projects will use it", and does not define `DEFAULT`, so 16a confirms which engine runs by behaviour before and after the switch. | `clients/3d-spike/project.godot`; the probe, 2026-10-08 | Jolt is not running today. Selecting it is a behaviour change to an **accepted** slice, so it is validated by re-running the slice's own objective checks (`--drive`, `--measure`, `--link`) on Jolt before anything else is built on it (16a). |
| **F-S14-3** | **The player collides with world geometry only.** `Player` is a `CharacterBody3D`, capsule r 0.30 m, h 1.72 m, centre 0.86 m up; `collision_layer = 0`, `collision_mask = Build.LAYER_WORLD` (1). `Build` defines one layer. Every scene collider is a `StaticBody3D` on `LAYER_WORLD`. Walk 1.45 m/s, jog 3.10 m/s, gravity 22, jump 0.45 m. | `scripts/player.gd:18–40, 70–82`; `build.gd:6, 28–158` | The capsule already equals the server's default person (`PERSON_RADIUS` 300, `PERSON_HEIGHT` 1 720; step-11 §17.3.1). Layers 2 and up are free for people, objects and disclosed geometry. |
| **F-S14-4** | **Perceived people have no collider and move in steps.** `SliceLink._figure` builds `NPC.make(rng, Pose.STAND, 1.72, false)` with a `Label3D`; `_on_observed` sets `global_position` to each observation's position. No body, no interpolation, no walking animation between observations. | `slice_link.gd:283–308, 345–361`; `HUMAN_REVIEW_QUEUE.md` VIS-3D-GODOT-2 known limitation 3 | The operator's original defect ("跟其他人穿模"): measured as `v2_run_into_townsperson.jpg`, centres within 0.10 m. 12e's people colliders; 16c's interpolation. |
| **F-S14-5** | **Targeting is a cone, not a ray, and sees through walls.** `facing_person()` picks the figure whose head direction from the camera has the largest dot product above 0.80 with the camera's forward vector. Nothing is occluded, and only figures can be targeted. | `slice_link.gd:458–472` | A player can "look at" Alice through the café's back wall. There is no way to target an object. 16a replaces the cone by a physics ray (§4.4). |
| **F-S14-6** | **`talk` already follows the rules.** Key **E** → `talk_to_facing("Hello! A coffee, please.")` submits regardless of the affordance; `may("talk", target)` is read only to say why the server will refuse. The utterance is a fixed string. | `slice_link.gd:431–448, 522–525` | `ADOPTION.md` §3.3 holds for `talk` today. The fixed utterance is the one piece of `AC-13`'s payload a test must hold equal across clients (§4.8). |
| **F-S14-7** | **Reporting rule.** A `move` every 0.50 m of travel or 20° of turn, at most every 0.08 s; height never reported (`REPORT_HEIGHT = false`). A crossing is reported on the far side of the threshold as soon as the body is in the other place's volume. | `slice_link.gd:20–38, 378–423` | Satisfies `PROTOCOL.md` §6.2 at jogging speed. Kept. |
| **F-S14-8** | **Reconciliation is refusal-only.** `_reconcile` is set on the first view, on a `move` answered other than `accepted`, and on a refused `move` frame; the next observation then puts the body where the world says. An accepted move whose recorded arrival differs (stopped short, nudged) is never corrected. | `slice_link.gd:149, 274–281, 484–487, 512–513` | 12e's correction rule (§4.3) replaces it, as step-11 §7.1 item 4 requires. |
| **F-S14-9** | **Two of the world's six places are drawn.** `KEYS = ["cafe", "street"]`. A place's frame is bound to the scene by translating so that its disclosed doorway lands on the slice's café door (`door_point`, `origin`). The slice's own florist volume is reported as the **street** (`PLACE_KEY[FLORIST_PLACE] = "street"`), because `social-cafe` has no florist. | `slice_link.gd:62–93`; `slice_world.gd:17–23, 185–232` | With bodies in the town (12d), a body inside the florist is outside the street's floor and would be stopped at the street's wall by the server: the florist hack **breaks** once 12d lands (F-S14-21). |
| **F-S14-10** | **Decorative townspeople are always built.** `SliceStreetscape._people` places static figures (one seated on the terrace) whether or not the client is connected. Connected, they are not world Persons: they cannot be talked to, collide with nothing and are not in any observation. | `scripts/slice/streetscape.gd:30, 345–360` | In a connected client they are people the world does not have — the "fake town" `VISUAL_SLICE.md` §4.1 rejects. QS14-9. |
| **F-S14-11** | **Texture RID leak on exit.** `SliceMain._exit_tree` frees `Props._scenes` templates; Godot still prints `7 RIDs of type "Texture" were leaked`, with GI off too. Not attributed. | `slice_main.gd:67–77`; `HUMAN_REVIEW_QUEUE.md` VIS-3D-GODOT-2 known limitation 8 | Harmless while running; an engineering defect that hides real leaks behind a familiar warning. Attributed and fixed, or recorded as engine-side with evidence, in 16a. |
| **F-S14-12** | **The scripted probe is the client's integration harness.** `SliceProbe` runs `--slice-drive`, `--slice-link`, `--slice-conversation` and others from the command line; `_link_check` walks out of the café, jogs, jumps, walks back in, talks from the door (`too_far_away`) and from the counter (accepted, reply heard), then watches the street. `_crowd_contact` frames the interpenetration. | `scripts/slice/slice_probe.gd:303–472, 856–888` | Every 12e and S14 checkpoint is a new probe mode in this harness, run for real (`ENGINEERING_RULES.md` §19). At 1 649 lines the file is past `ENGINEERING_STANDARDS.md`'s strong-warning threshold: new modes go in a sibling file (§9.2). |

## 2.2 The shared protocol module: `clients/protocol/mineworld`

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-S14-13** | **Three files, symlinked into the 3D project.** `world_client.gd` (`MineWorldClient`: join, submit, four signals plus `submitted_request`), `observation.gd` (`MineWorldObservation`: a reader), `space.gd` (`MineWorldSpace`: the only axis conversion). `clients/3d-spike/mineworld` is a symlink to `../protocol/mineworld` (`ADOPTION.md` §1). | `ls -la clients/3d-spike/mineworld` | One source for both clients. Every change is shared with S12 (§12). |
| **F-S14-14** | **No accessor for complete affordances.** `affordance(action_type, target)` returns the **first** match; `ADOPTION.md` §2 says several complete affordances routinely share an action type and target and that "the module gains no accessor for this in S9; the first client use is S12's". 12c's `kick`/`throw` (target `null`, one per object) and economy's `buy` (target `null`, one per kind) are exactly that case. | `observation.gd:151–162`; `ADOPTION.md` §2 | M-1 and M-2 (§12), coordinated with S12, which needs the same for `buy` and `give`. |
| **F-S14-15** | **`component()` returns only a dictionary payload.** A payload that is an array, or a dictionary whose interesting value is a list (`place-shape.solids`, the `loose-objects` listing), is read through `component()`; an array-valued payload would come back `{}`. | `observation.gd:102–111` | M-3 (§12): a raw-value reader. Shapes of 12c's listing are fixed by 12c's code; §12 states what the reader must cope with either way. |
| **F-S14-16** | **No events reach a client.** `events()` exists, and `ADOPTION.md` §6 says revision 1 leaves it empty. The server's `PerceptionContext` already carries a bounded window of recent log entries (`recent_events`, default 64), and `presence::observe` deliberately leaves `Observation::with_events` empty, because deciding which facts an observer is entitled to is perception's judgement. | `server/src/perception.rs:44–96`; `server/src/host.rs:197–206`; `systems/presence/src/observe.rs:28` | `object-moved`'s `path` (§1.4 item 5), `person-shoved` and `stopped-short` cannot reach the 3D client. Animating a kick along its path is impossible without S11 (R-S11-1, §11). Without it the client tweens between disclosed positions (§4.5). |

## 2.3 Server, protocol and worlds

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-S14-17** | **The observer perceives only its own place.** An observation lists the observer's place and the people in it; place components (`passages`, and after 12b/12c `place-shape` and `loose-objects`) are disclosed "to whoever perceives the place". | `systems/presence/src/observe.rs` (`present_with`); `systems/bodies/src/component.rs:100–107` (doc of `PlaceShape`) | Standing on the street, the client is told the street's shape and not the café's. Disclosed colliders exist only for the place the player stands in; the other place's walls come from the scene (§4.2). |
| **F-S14-18** | **`place-shape` on the wire.** Component type `place-shape`, schema 1: `{ "floor": { "min": {x, y}, "max": {x, y} }, "solids": [ { "min": {x, y}, "max": {x, y}, "height" } ] }`, integer millimetres in the place's frame, at most 64 solids, coordinates within ±100 000 mm. The floor's edge is the place's walls; a doorway needs no gap, because a crossing is a placement (SD-B3). | `systems/bodies/src/component.rs:17–200`; step-11 SD-B3 | A client that turns the floor's edge into colliders must leave a gap at each disclosed passage, or it blocks the player at a door the server lets them through (§4.2). |
| **F-S14-19** | **12c's objects and actions (frozen design, being implemented).** `loose-objects` on the Place, disclosed as a listing `{ object, shape, at }` per object, `shape` `{ box: { half: {x, y, z} } }` or `{ ball: { radius } }`, `at` the object's centre. `kick { object }` and `throw { object, toward: null \| {x, y} }` are **target-less**, offered complete only for objects within 800 mm; `shove {}` targets a Person, offered complete to every present Person (available within 1 000 mm). | step-11 SD-O2, SD-O11 … SD-O15, QO-6, QO-7, PO-7 | Kick and throw cannot be found by `affordance("kick", object_id)`: the object is in the payload, not the target. The client matches offers by `payload.object` (M-1). Exact JSON keys are 12c's code; 12e re-audits them on 12c's merge. |
| **F-S14-20** | **`talk` is not a complete affordance; `buy` is.** Conversation offers `Offer::new::<Talk>(talk_requirement())`: no payload (the utterance is free text). Economy offers `Offer::complete(&Buy::new(item), buy_requirement(place))`, target `null`, payload `{ item }`, one per priced kind, only in a shop's place, and only in `market-town`; `social-cafe` has no `economy`. | `systems/conversation/src/system.rs:229`; `systems/economy/src/offer.rs:44`, `action.rs:13–47`; `worlds/market-town/world.yaml` | The slice connects to `social-cafe` today; buying needs `market-town` (QS14-5). |
| **F-S14-21** | **Doorways in the drawn street.** Café ↔ street is aligned (10a). The store's passage lands on the street at (12 000, 3 000), 3.62 m east of The Flower Room's door in the scene; workplace, apartments and park land on blank façades (VIS-3D-GODOT-2 known limitation 2). `store.yaml`'s note: "A convenience store on the street's north side, east of the café" — where The Flower Room stands. | `worlds/social-cafe/places/store.yaml`, `cafe.yaml`; `HUMAN_REVIEW_QUEUE.md` VIS-3D-GODOT-2 limitation 2 | Once people walk (16c), three of them vanish into walls. One world-content alignment fixes all of it, and it belongs with 12d's geometry authoring (R-12d-2, §11.3). |
| **F-S14-22** | **Nobody walks in a hosted world.** `mineworld server … --agent SEAT` drives a `RuleController`, which only answers. The `PacedRuleController`, which walks, greets and (since 11c) attempts complete affordances, runs only inside `mineworld run`, which hosts no clients. The hosted clock is wall time (`runtime.rs:77–79`). | `tools/cli/src/agent.rs:40–80`; `tools/cli/src/main.rs:286–361`; `server/src/runtime.rs:66–79` | `ENGINEERING_RULES.md` §10's "human + AI-controlled Persons sharing the same world" holds only for one answering barista. A hosted paced controller is a composition-root change (R-S11-2). |
| **F-S14-23** | **AC-13 holds against throwaway flavours, not against the clients.** `clients/protocol/evidence/request-2d.json` and `request-3d.json` were produced by `clients/protocol/demo/demo.gd`'s two flavours; the "3D" flavour walks the 2D flavour's strides. `tools/cli/tests/ac13_semantic_parity.rs` compares them with `server/src/parity.rs`. Neither the slice nor an S12 client produced them. | `clients/protocol/evidence/README.md`; `server/src/parity.rs:1–60` | `AC-13` must be re-shown between the real 3D client and S12's real 2D client (16e). The parity definition in the server is reused unchanged. |
| **F-S14-24** | **Items and organizations are unnamed (F-41).** `naming`'s section is carried by Person only. A shop's disclosed listing names items by id. | `systems/naming/src/section.rs:14`; `systems/economy/src/component.rs:135–150`; `overall.md` §7 (F-41) | A buy menu would show ids, which the client may never show (`SliceLink.display_label`'s rule). Names for items are a pack/content change outside the clients, assigned to S12 by `overall.md` §7 (R-F41, QS14-6). |
| **F-S14-25** | **`docs/MVP_STATUS.md`'s 3D row is stale.** It reads "presentation spike runnable with three camera modes; awaiting feel review", although `VIS-3D-GODOT-2` was accepted after the operator played it. | `docs/MVP_STATUS.md:45, 106` | Proposed correction in §17; applied by the primary session or by 16a. |

---

# 3. Gap analysis

## 3.1 `AC-14`, clause by clause, as the client stands on `main @ 0fd0be3`

| Clause | Holds today? | Evidence | What is missing | Delivered by |
| --- | --- | --- | --- | --- |
| moves in first person | **Yes** | `Player`: WASD, mouse look, first person by default; `--drive` | — | — |
| through a walkable environment | **Yes** | the accepted slice; `--drive` walk-in, loop, walls, jumps | — | — |
| collision / basic navigation | **Partly** | world geometry collides (`LAYER_WORLD`) | **no collision with people** (F-S14-4); none with objects; scene walls and server walls are two sources that can disagree once 12d lands (R-B4) | 12e |
| enters a Place | **Yes, two of six** | `--link`: café → street → café on foot, 50 moves, 0 refused; server place changes observed | the store, workplace, apartments and park are not enterable or not drawn (F-S14-9, F-S14-21); the florist is reported as the street | 16c (with R-12d-2) |
| approaches an NPC | **Yes** | walk to the counter; Alice drawn where the world says | figures teleport between observations (F-S14-4); you can walk through them | 12e (collision), 16c (motion) |
| spatially initiates a conversation | **Partly** | E → `talk`, server answers `too_far_away` from the door and accepts at the counter | targeting is a cone that ignores walls (F-S14-5) | 16a |
| interacts with one object | **No** | — | no object exists in the town, none is drawn, none can be targeted, no action is sent | 12c (actions), 12d (objects in the town), 12e (client) |
| all state authoritative on the server | **Partly** | every move and talk is a request; the server decides | the client never corrects an **accepted** move whose arrival differs (F-S14-8): once bodies resolves strides, the client would drift from the truth | 12e |
| human + AI-controlled Persons sharing the world (§10) | **Barely** | Alice answers (`--agent alice`) | nobody walks (F-S14-22) | 16c (with R-S11-2) |

## 3.2 `AC-13`, against the real contracts and the real clients

| Clause | Holds today? | Evidence | Gap | Delivered by |
| --- | --- | --- | --- | --- |
| an identical semantic core for `talk` from a 2D click and a 3D walk-up-look-press | **Against throwaway flavours only** | `request-2d.json` / `request-3d.json` from `clients/protocol/demo`, compared by `ac13_semantic_parity.rs` with `server/src/parity.rs` | the real slice and S12's real client have never produced a compared request (F-S14-23); the slice's utterance is fixed in code (F-S14-6) | 16e |
| resolved by the same system to the same result | **Yes, for the flavours** | both accepted by `conversation` | as above | 16e |
| neither client implements a validity rule | **Yes for `talk`, by inspection only** | `talk_to_facing` submits regardless | no executable check; kick, throw, shove and buy do not exist in either client yet | §10 I-S14-1 (executable), 16a |
| the same holds for the S15 actions (`step-11-bodies.md` §7.3, "differing only in acquisition") | **No** | — | neither client sends kick, throw or shove | 12e (3D), S12 (2D), 16e (compared) |

## 3.3 The known gaps the coordinator listed, located

| Gap | Located at | Owner in this plan |
| --- | --- | --- |
| No collision with people | F-S14-3, F-S14-4 | 12e |
| No object interaction | F-S14-19, §3.1 | 12c → 12d → 12e |
| "Nobody walks the street in the connected slice" | F-S14-22 | R-S11-2 → 16c |
| Three town doorways land on blank walls | F-S14-21 | R-12d-2 → 16c |
| Texture RID leak warning on exit | F-S14-11 | 16a |
| Items and organizations are unnamed (F-41) | F-S14-24 | R-F41 (S12's plan) → 16d |

---

# 4. Design

## 4.1 How 12e and S14 divide — options and recommendation

Both build the same client, in the same files (`slice_link.gd`, `player.gd`, the probe). Three ways to
cut it:

| Option | What it means | For | Against |
| --- | --- | --- | --- |
| **A. 12e stays S15's last PR; S14 is everything else; one plan (this file) for both** | 12e = Jolt colliders from disclosure, the correction rule, press-through, ray targeting of people and objects, kick/throw/shove, objects drawn. S14 = the parts that are not bodies: Jolt switch and ray targeting first (16a), the shared-module accessors (16b), a living street and the town's doorways (16c), buy and the market world (16d), AC-13 with S12 and the Demo B package (16e). | S15's acceptance ("the 3D client collides with people and reconciles on difference", `overall.md` §3 S15) closes where the operator's requirement R-1 becomes visible. The 12e row stays as the operator froze it. One plan means one designer of `SliceLink`. | Two step names over one client; the 12e row's scope must be refined by this file (§17.2). |
| **B. Merge 12e into S14** | S15 ends at 12d; all client work is S14's. | One step owns one client. | Changes a frozen step split (`step-11-bodies.md` header: "the five-PR split are frozen") — an operator decision. S15 would close without its client-side acceptance. |
| **C. One large client PR** | 12e and all of S14 in one PR. | Fewest merges. | Thousands of lines across physics, UI, motion and evidence in one review; waits for every dependency at once (12d, S11, S12); violates "small reviewable PRs" and stalls the parts that can start now. |

**Recommendation: A.** It changes no operator decision, it lets 16a and 16b start now, and it keeps the
bodies work reviewable as one coherent PR whose checkpoint is S15's. This file is the plan for both, so
12e's detail lives here; `step-11-bodies.md` §11.1's row is refined, not replaced (§17.2). The order:

```text
now           16a  Jolt, ray targeting, the leak, the no-rule scan          (3D client only)
now           16b  shared module: complete affordances, raw components     (with S12)
after 12d     12e  bodies in the client: colliders, correction, press-through, kick/throw/shove, objects
after 12d     16c  a living street: walking people, motion, the town's doorways   (+ R-S11-2)
after 12e     16d  buy, market-town, names on things                       (+ R-F41)
after S12     16e  AC-13 against S12's client, AC-15 with three windows, Demo B for the operator
```

## 4.2 Colliders from what the server discloses

One rule governs every collider the client builds: **its shape and position come from the observation,
never from a copy of the pack and never from a number the client invents.** The scene's own static
geometry stays, because it is the world the operator accepted; the disclosed geometry is added beside it
(R-B4's "one source").

**Layers** (all in `Build`, the one file that already defines `LAYER_WORLD`):

```text
LAYER_WORLD       1   the scene's static geometry (unchanged)
LAYER_BODIES      2   perceived people (AnimatableBody3D capsules)
LAYER_OBJECTS     4   loose objects (AnimatableBody3D boxes and balls)
LAYER_DISCLOSED   8   the current place's disclosed walls and solids (invisible StaticBody3D)
player mask       LAYER_WORLD | LAYER_DISCLOSED | LAYER_BODIES | LAYER_OBJECTS
camera boom mask  LAYER_WORLD (unchanged: a third-person camera does not stop at a person)
target ray mask   all four
```

**The current place's walls and solids** (`place-shape`, F-S14-18). When the observer's place or its
disclosed `place-shape` payload changes, the client rebuilds one node `DisclosedGeometry` for that place,
in that place's frame (`SliceLink.to_scene`):

- the floor's four edges as walls 200 mm thick, outside the floor, 3 000 mm high — the numbers bodies'
  own Rapier scene uses for its perimeter (SD-B11), so the client's wall is where the server's is;
- a **gap** in a wall wherever a passage the observer was told about (`passages.leads_to[].here`) lies
  within 400 mm of that edge, `DOOR_GAP` = 2 000 mm wide, centred on the passage point's projection. A
  crossing is a placement, not a sweep (SD-B3), so the server has no wall there to mirror; a gap wider
  than the scene's door is harmless, because the scene's own jambs still stop the body;
- each solid as a box from its footprint, `height` high, standing on the floor.

The node is invisible. It never replaces the scene's walls; it exists so that the body is stopped where
the server would stop it even if the scene and the server disagree. Where they disagree, the
disagreement is a content defect, found by a probe rather than by the player (`--world --geometry`,
§5): for every disclosed edge and solid face, rays from inside the floor must meet a scene collider
within 150 mm of the disclosed face (except at gaps); for every scene collider face inside the disclosed
floor, a disclosed face must lie within 150 mm. Both directions are reported with coordinates; the probe
fails on either. That is the executable form of R-B4.

**People.** Every drawn figure carries an `AnimatableBody3D` child, `CapsuleShape3D` r 0.30 m, h 1.72 m,
centre 0.86 m up, on `LAYER_BODIES`, with `sync_to_physics` on, so it moves with the drawn figure (§4.6)
and pushes nothing. The numbers are the server's default person (`PERSON_RADIUS`, `PERSON_HEIGHT`, SD-B2),
which 12b does not disclose because every person is the default capsule (QP-6). The client holds them in
one constant pair beside the player's own capsule, which already has them (F-S14-3), and cites their
source; when bodies discloses a person's shape (§10.2 of step-11 anticipates it), the client reads it
instead (QS14-3).

**Objects** (`loose-objects`, F-S14-19). One `AnimatableBody3D` per listed object, its shape exactly the
disclosed one (`BoxShape3D` of size 2·half, `SphereShape3D` of the radius), on `LAYER_OBJECTS`, carrying
the object's id as node metadata (a string, `ADOPTION.md` §3.1). Drawn as described in §4.5.

## 4.3 Local prediction and the correction rule

**Jolt.** `[physics] 3d/physics_engine="Jolt Physics"` in `project.godot` (DEP-14). The slice's accepted
behaviour is re-measured on it before anything else is built (16a): `--drive` (walk-in, loop within
0.10 m, walls, cameras with zero body movement on a switch, jumps 0.49 m, the florist loop), `--measure`,
`--link`. Step-11 §3.2 records the known differences (position-only stabilization, kinematic contacts);
`CharacterBody3D.move_and_slide` is the only physics the player uses, so the risk is small and measured
rather than assumed.

**What the client predicts.** Exactly one thing: that its own body is stopped by what stands in its
way. It does not predict a nudge, a push, a shove or a landing. It never moves anybody else's figure or
any object except to where an observation puts it.

**Press-through: reporting the stride the player asked for** (QS14-1). A body that stops locally at a
person never asks the server to nudge them, so QB-10 ("walking into a person nudges them aside") would
never happen from the 3D client, and 12e's own checkpoint ("walks the player into Alice, who is nudged
aside") could not pass. So when, in a physics frame, `move_and_slide` reports a collision with a
`LAYER_BODIES` or `LAYER_OBJECTS` collider and the input points into it, the next report is not the
body's position but the **intended** one: the capsule cast from the body along the input direction for
the distance the input would have moved it since the last report, against `LAYER_WORLD |
LAYER_DISCLOSED` only (`PhysicsDirectSpaceState3D.cast_motion`). The server decides what that stride
achieves — a nudge of at most 300 mm, a push, a jam, a stop — and the correction rule shows its answer.
The client still decides nothing: it reports a wish, bounded by the same `REPORT_DIST` cadence as any
stride, and adopts the answer. Without the input pointing into a person or an object, the report is the
body's position, exactly as today.

**The correction rule** (proposed `ARC-S14-a`; step-11 §7.1 item 4, made buildable):

```text
CORRECTION_MM  150     more than the server's quantization and the controller's 10 mm GAP, less
                       than a visible jump (step-11 §7.1)

baseline       the position sent with the most recent `move` whose answer has arrived
in flight      the positions sent with every `move` not yet answered

on each observation of the place the body is in:
  auth  = self_location.local (x, y; height is never compared)
  if |auth − baseline| > CORRECTION_MM  and  |auth − p| > CORRECTION_MM for every p in flight:
        correct: move the body to auth, zero its velocity, reset the reporting baseline to auth,
        and SUSPEND reporting until every move sent before the correction has been answered and one
        further observation has arrived; then apply this rule once more and resume.
on an observation of another place:
        the place changed under the client (a crossing it did not ask for, or one it did): adopt it
        as today (`_reconcile`).
```

The suspension is what makes the rule converge: a stride sent from the wrong position can still be
accepted after the correction and would otherwise drag the body back, correction after correction. It
replaces the refusal-only `_reconcile` (F-S14-8) and covers, with one number, a stride the server
stopped short, being nudged by somebody walking into you, a shove, a jam at a box, and a refusal.

**If S11 adds `acted_through`** (R-S11-4: the newest `ActionId` of this connection the observation
reflects), the rule becomes exact: `auth` is compared with the position sent in that action, and
nothing in flight needs to be considered. The heuristic above is the fallback and is what 12e builds
if R-S11-4 is not available when 12e is implemented.

**How a correction looks.** A difference up to 1 m glides over 120 ms; a larger one snaps. Presentation
only (QS14-11).

## 4.4 Targeting, and intents through affordances

**Acquisition is a ray.** The cone in `facing_person()` (F-S14-5) is replaced by one physics ray from
the active camera through the screen centre (in third person, starting past the player's own capsule),
30 m long, against all four layers. The first hit decides the target:

```text
hit a LAYER_BODIES collider     a person: its figure's entity_id
hit a LAYER_OBJECTS collider    an object: its object id
hit anything else               no entity target; the hit point is the aim point (for throw)
nothing within 30 m             no target
```

Walls occlude by construction. 30 m is the draw distance of a target highlight, not a reach: nothing is
refused because it is far — the server says `too_far_away`.

**What the player is offered is the server's list.** The HUD shows, for the current target, what the
latest observation offers, never what the client concludes:

```text
person    every affordance whose target is the person (talk, shove, …), with available or the reason
object    every complete affordance whose payload names the object (kick, throw), from M-1
here      in a place that offers complete target-less affordances (buy), "B: buy" is shown
```

A wording table maps an action type to a verb ("talk", "shove", "kick", "throw", "buy"); an unknown type
is shown as its own name. That is presentation (`ADOPTION.md` §4: wording is the client's).

**Intents.** One file, `scripts/slice/intents.gd` (`SliceIntents`), builds every interaction request.
With `slice_link.gd` (which keeps `move`), it is the only file in the client that names an action type —
the scan in §10 holds that.

| Action | Key | Acquisition | `target` | Payload | Complete offer? | What the client knows |
| --- | --- | --- | --- | --- | --- | --- |
| `talk` | E | ray hits a person | the person | `{ utterance }` — the line the player says | no (free text, F-S14-20) | the name and the payload shape |
| `shove` | R | ray hits a person | the person | `{}` | yes, one per present person (SD-O12) | the name and `{}` |
| `kick` | F | ray hits an object | `null` | `{ object: <id> }` | yes, for objects within 800 mm (QO-7) | the name and the payload shape |
| `throw` (default aim) | G, G | ray hits an object; G again without aiming elsewhere | `null` | `{ object, toward: null }` | yes, within 800 mm (SD-O13) | as kick |
| `throw` (aimed) | G, aim, G | ray hits an object; then the ray's hit point | `null` | `{ object, toward: { x, y } }` in the player's place frame (`to_world`, rounded mm) | no (a free point) | as kick, and `toward` |
| `buy` | B | none: a menu of what this place offers | `null` | the offered complete payload, unchanged | yes, one per priced kind (F-S14-20) | **nothing**: the menu lists complete affordances and submits them unchanged (M-2) |

Two rules make "through affordances" exact without making the client depend on an offer existing:

1. **When a complete offer matches, it is submitted unchanged** (M-2). For shove, kick and throw (default
   aim) the client looks for the complete affordance with that type whose target, or whose
   `payload.object`, is the targeted entity.
2. **When none matches, the same request is built by name and submitted anyway**, so that the server
   says why (`too_far_away` for an object out of reach, which QO-7 deliberately does not offer;
   `unavailable` in a world without bodies, I-9). A test asserts that, wherever an offer exists, the
   built request equals the offered one byte for byte — the builder can never drift from the pack.

`ADOPTION.md` §3.3's "NOT allowed: keep a list of which actions exist in this world" is respected: the
keys are an input mapping, and the client's knowledge of five action names is the same knowledge the
slice already has of `talk` and `move` — what a request is called and how its payload is shaped, never
whether it is allowed. `buy` needs no name at all.

**Throw's aim is a two-press gesture** (QS14-8): the first G picks the object and shows an aim marker
where the ray meets the world; the second G throws toward it, or, if the aim never left the object,
throws with `toward: null`. Esc cancels. The marker is drawn wherever the ray hits — the client never
clamps it to a range (`THROW_RANGE_MAX` is the server's; an aim beyond it is answered
`precondition_failed`).

**Answers.** A `result` is shown in words (`SliceLink._readable` already maps codes). A kick or throw
that is accepted is seen when the next observation moves the object; a shove when it moves the person.

## 4.5 Object rendering

```text
source      the drawn place's `loose-objects` listing: { object, shape, at } per object
mesh        box  → a carton/crate mesh scaled exactly to 2·half, from the accepted palette
            ball → a sphere of the radius, from the accepted palette
            The collider is the same shape (§4.2): what you see is what blocks you.
appearance  from the shape only: items are never perceived (F-O1), so no tag or name is known.
            R-S15-2 (optional) would let the listing carry tags, and the client choose a mesh from them.
motion      when an object's `at` changes, its body tweens from the old to the new position over
            clamp(distance / 4 m/s, 0.15 s, 0.8 s); a ball also rolls (rotation = distance / radius
            about the horizontal axis normal to the motion) — rotation is presentation, the server
            locks it (SD-O16)
            with R-S11-1, an `object-moved` the observer is told about is drawn along its `path`, one
            keyframe per 0.1 s (PATH_EVERY = 6 sub-steps), and the tween is the fallback
lifetime    objects exist while their place is drawn; leaving the place frees them
```

## 4.6 A street people walk in (16c)

- **Who walks.** People walk only if the server hosts a controller that walks them: R-S11-2 (a hosted
  paced controller per seat). Nothing in the client makes anyone move.
- **How they are drawn moving.** A figure no longer jumps to each observed position. It walks toward the
  newest one at up to the jog speed (3.1 m/s), with the same gait the player's body uses
  (`NPC.step(delta, speed)`, which `Player` already drives), facing its direction of motion, and the
  observed facing when it stands. A figure more than 4 m behind its observed position (a crossing into
  view, a long correction) is placed, not walked. Interpolation is presentation; its collider follows
  the drawn figure, so what the player sees is what blocks them.
- **Leaving view.** A person who leaves the observation while near a disclosed doorway of this place
  walks the last metres to that doorway's scene point and is then removed; otherwise they are removed at
  once, as today.
- **Decorative townspeople** (F-S14-10) are not built when the client is connected (QS14-9): in a
  connected client every person drawn is a person of the world.

## 4.7 Places and doorways

12d authors the town's `body:` sections "matching the 3D slice's layout" (step-11 §11.1). This step adds
what the client needs from that authoring (stated as requirements on 12d in §11.3):

- **The café and the street** match the slice's café room and the street's walkable area, so the
  geometry probe (§4.2) passes.
- **The store is The Flower Room** (QS14-4, operator-material): the world's `store` stands "on the
  street's north side, east of the café" — where The Flower Room is — but its street-side doorway lands
  3.62 m east of that door (F-S14-21). 12d moves the store's passage onto The Flower Room's door and
  authors the store's floor to that room. The client binds `SliceWorld.FLORIST_PLACE` to the key `store`
  instead of `street` (F-S14-9). That is required in any case: once the street has a floor, a body
  reported as "street" inside the florist is outside the street's walls. What the room *looks like* —
  the florist the operator accepted, or a corner store the world describes — is the operator's.
- **Workplace, apartments and park** keep doorways the player cannot use (`VISUAL_SLICE.md` §4 forbids
  a third enterable building), but their street-side points move onto existing non-enterable doors of
  the slice, so a person going home or to work walks to a door and through it rather than into a blank
  wall.

## 4.8 The `AC-13` equivalence test against S12's 2D client

**What is compared.** For each interaction, the request each real client submits — through its own
acquisition — has the same semantic core, compared by the server's own `parity::semantic_core` and
`differing_fields` (F-S14-23), with `differing_fields ⊆ { ActorLocation }`, and both are resolved to the
same result kind with the same fact types caused.

```text
talk     the barista, with the scenario's utterance (a scenario constant given to both clients as
         --utterance=, never typed differently in each)
shove    a named person standing next to the player
kick     a named object (bodies' scenario world, or the town after 12d)
throw    the same object, complete form (toward: null). An aimed throw is acquired by a click in 2D and
         by a ray in 3D; their points cannot be equal to the millimetre, so the aimed form is shown
         accepted by both, and is not part of the equality
buy      one kind, in market-town (after 16d)
```

`move` is excluded: a 2D client walks in clicked strides and a 3D client in walked ones, and both are
just `move`; their equality would be a test of identical walking, not of identical meaning.

**How.** Each client has a scripted scenario mode that plays the same scenario against a fresh hosted
world of the same pack (ids are allocated deterministically at genesis, so the same seat and the same
people have the same ids), and writes what it submitted, with the answers, as evidence in the format
`clients/protocol/evidence/request-*.json` already uses (`{ flavour, token, request }`, plus `result`).
A test, `tools/cli/tests/ac13_clients.rs`, reads the two transcripts and compares the pairs. A world run
with `--save` lets the test read, with `mineworld inspect`, that the caused facts have the same types.
Each flavour runs on its own fresh world, for the reason the existing evidence README gives.

**What S12 must provide** (coordination item C-S12-1, §12.3): a scripted scenario mode in the 2D client
that performs the five interactions by its own acquisition (clicks), and writes the same transcript
format through `MineWorldClient.submitted_request`.

## 4.9 How the operator will play it and judge it

At 16e the step ends with a review package in `docs/HUMAN_REVIEW_QUEUE.md`, a framework milestone
(`ENGINEERING_RULES.md` §19), not a visual one: the look is `VIS-3D-GODOT-2`'s, already accepted. The
package is a short runnable list (the operator's standing preference), for example:

```text
1  ./mineworld-slice --world                 social-cafe with bodies, people walking
   walk out of the café: people walk the street, and go into doors, not walls
   walk into Wes: he steps aside; walk into a group: you stop
   jog into the townsperson who stood in v2_run_into_townsperson.jpg: you no longer pass through
2  find the box by the café door: walk into it (it slides), look at it and press F (it is kicked),
   G, G (thrown), G, aim at the floor, G (thrown there)
3  look at Bob and press R (he is shoved); stand still and let somebody walk into you (you are nudged)
4  ./mineworld-slice --world=market-town     at the counter press B: buy a coffee by its name
5  three windows: mineworld server worlds/market-town --paced-all, ./mineworld-slice --server=…,
   the 2D client: speak to Alice in 2D, walk up to her in 3D, she knows (AC-15)
```

Plus the objective evidence the operator can re-run (`--link`, `--bodies`, `--geometry`, the `AC-13`
test), frames of each interaction, known limitations as facts, and the questions asked. The operator
judges embodiment and interaction correctness (`MVP.md` §7.2), not graphics.

---

# 5. Validation — how each claim is shown, by running the client

Every claim below is shown by a scripted mode of the real client against a real `mineworld server`,
started by `./mineworld-slice --world …` exactly as the operator starts it (`ENGINEERING_RULES.md` §19).
Connected modes move out of the 1 649-line `slice_probe.gd` into a sibling, `slice_probe_world.gd`
(F-S14-12); the existing modes are not edited beyond what their claims need.

| Mode | PR | Claim | Pass condition, decided before running (`ARC-23`) |
| --- | --- | --- | --- |
| `--drive`, `--measure`, `--threshold`, `--link`, `--conversation` | 16a | The accepted slice is unchanged on Jolt | every existing line passes as it did on Godot Physics; any number that moves is reported with both values |
| `--world --target` | 16a | The ray targets what is visible, and only that | from the café door, facing Alice through the room: Alice; with a wall between (from the street, facing her through the back of the building): nobody; the E press from the door still submits and is answered `too_far_away` |
| `--world --geometry` | 12e | The scene and the server agree on walls (R-B4) | every disclosed face has a scene collider within 150 mm, and vice versa, gaps excepted; each miss printed with coordinates |
| `--world --bodies` | 12e | Bodies, end to end, with frames | (a) the player walks into a standing person: that person's next observed position moved by 1 … 310 mm, the player is never drawn closer than 0.59 m to them, and no correction exceeds the nudge; (b) into a group of three: the body stops, no `move` refused; (c) into a box: the box's disclosed position moves; (d) F on a ball within reach: accepted, the ball moves; out of reach: `too_far_away`; (e) G, G: accepted, the object moves; aimed: accepted; (f) a second scripted connection on another seat shoves the player: the player's body is corrected within two observations; (g) after the run, the drawn positions of the player, every person and every object equal the final observation's, and the save's last facts (`mineworld inspect`) |
| `--world --rules` | 12e | **No rule in the client**, behaviourally (I-S14-1.2) | the client's submitted interaction requests are identical against social-cafe with and without bodies; only the answers differ |
| `--world --bodies --no-people-colliders` | 12e | **No rule in the client** (12e's adversarial criterion) | with the people colliders not built, walking into a person still ends with the person nudged and the player corrected by the server's answer; the client's submitted requests contain no new kind |
| `--world --street` | 16c | People walk, and walk into doors | 60 s watched: at least three people move; no drawn figure moves more than 3.1 m/s × frame time except a placement; every figure that leaves view does so within 1.5 m of a disclosed doorway point; no figure is drawn inside a scene collider |
| `--world=market-town --buy` | 16d | Buying is the server's list | in the café, B lists exactly the observation's complete `buy` affordances, by name, never an id; one is bought: accepted; the wallet and holdings the observation discloses change |
| `--scenario=ac13` | 16e | `AC-13` against S12 | §4.8 |

---

# 6. Ownership and change amplification

```text
owns nothing in the world   the client owns no world state; it owns its scene, its bodies' local
                            motion, and its drawing of what it was told
decides nothing             every validity question is the server's (§10 I-S14-1)
```

What the step's PRs touch, and must not touch:

| May touch | Must not touch |
| --- | --- |
| `clients/3d-spike/**` (scripts, `project.godot`, evidence), `mineworld-slice` | `kernel/`, `contracts/`, `persistence/`, `server/src`, every System Pack, `cognition/` |
| `clients/protocol/mineworld/**` and `ADOPTION.md`, **only** in 16b and in coordination with S12 (§12) | the 2D client's files (S12's) |
| `tools/cli/tests/ac13_clients.rs` (16e) | `tools/cli/src` — except R-S11-2's hosted controller, if S11 assigns it to this step (QS14-10) |
| docs: `server/PROTOCOL.md` §6.2 (the correction rule), `docs/DECISIONS.md` (DEP-14, ARC-S14-a), `docs/MVP_STATUS.md`, `docs/HUMAN_REVIEW_QUEUE.md` | `worlds/**` — the town's geometry and passages are 12d's (§11.3) |

The change-amplification test (`CLAUDE.md` §4 rule 5): adding embodied interaction adds client modules
and a few protocol-module readers, and edits no system, no contract and no scheduler. If any PR finds it
needs one, it stops and returns to the primary session.

---

# 7. The two gating questions, and §12

- **§11 — Can this support a Minecraft-like embodied 3D client without redesigning the kernel? Yes, and
  this step is that client.** Movement is local and continuous, reported as strides; interaction is
  acquired spatially by a ray; the kernel, the contracts and every system are unchanged.
- **§§9, 22 — Can 2D and 3D use the capability without duplicating game logic? Yes.** The 2D client
  sends the same five requests by clicking (S12), and `AC-13`'s test compares them (§4.8). The only
  difference between the clients is acquisition, plus the 3D client's local collision, which predicts a
  result the server states and decides nothing (`--no-people-colliders`, §5).
- **§12 — Does an engine concept enter a generic contract? No.** Jolt, layers, `AnimatableBody3D`, rays
  and tweens live in `clients/3d-spike`. The protocol module gains only readers of what the server
  already sends (§12).

---

# 8. Reuse before reinvention — the comparison for every non-trivial piece

The operator's directive (§1.5) and `REUSE_POLICY.md`: adopt → adapt behind a MineWorld interface →
extend → build our own, with several real candidates compared before choosing. Facts about third-party
projects were read on 2026-10-08 from their repositories (licence from GitHub's licence detection, which
reads the LICENSE file; latest release from the releases page) and are re-verified when the PR that
adopts or rejects them is designed. "Fit" always means: works with a **Rust server that is authoritative
and speaks `server/PROTOCOL.md`**, and puts **no world rule in the client**.

## 8.1 Client-side physics (collision, the character's sweep)

| Option | Fit | Licence | Maturity, maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Jolt, built into Godot 4.7.2** | Prediction only, never authoritative; nothing to integrate: a project setting. `CharacterBody3D` runs on it unchanged | MIT (Jolt) | In the engine since 4.4; "By default, new projects will use it as the physics engine" (4.7 docs) | One line, plus re-running the accepted slice's checks | **Adopt (DEP-14).** The engine's own default and its better solver, at zero integration cost |
| Godot Physics (what runs today, F-S14-2) | Equally fit | MIT | The legacy default; maintained | Zero | **Keep as the fallback.** If a Jolt difference breaks an accepted slice check that cannot be fixed in the client, switching back is the same one line (QS14-12) |
| `appsinacup/godot-rapier-physics` (GDExtension, Rapier in Godot) | Fit; its one advantage — determinism matching the server's Rapier — buys nothing, because the client is not authoritative (step-11 §3.3) | MIT | v0.36.0, released 2 Oct (2026), API features for Godot 4.4–4.7 | A binary extension per platform to ship and keep in step with Godot | **Reject now; recorded as the route** if corrections ever become visible enough to need bit-matching prediction (R-B4, step-11 §3.3) |
| `godot-jolt/godot-jolt` (the former extension) | — | MIT | Maintenance mode; "the only supported versions of Godot are between 4.3 and 4.6" | — | **Reject**: superseded by the built-in module and does not support 4.7 |
| Build our own collision | — | — | — | A physics engine | **Reject**: commodity infrastructure (`REUSE_POLICY.md` §4) |

## 8.2 Prediction and reconciliation (netcode)

| Option | Fit | Licence | Maturity, maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| Godot's high-level multiplayer (`MultiplayerSynchronizer`, `MultiplayerSpawner`, RPC over ENet/WebSocket peers) | **No.** Both ends must be Godot speaking Godot's RPC and replication protocol; our server is Rust and speaks `PROTOCOL.md`. Replicating state from the client would also invert authority | MIT | In the engine | Replacing the server, or emulating Godot's protocol in Rust | **Reject** |
| `foxssake/netfox` (client-side prediction, server reconciliation by rollback, lag compensation) | **No.** "Supports client-server architecture" through Godot's high-level multiplayer: the server must be a Godot process that re-simulates the player's inputs tick by tick. MineWorld's server decides semantic strides, not input ticks | MIT | Active: v1.35.3 (23 Nov, likely 2025), Godot 4.x | Re-architecting the server around Godot ticks — the framework lock-in `REUSE_POLICY.md` §3 forbids | **Reject** for the client; recorded as the reference implementation of the pattern |
| `godot-rollback-netcode` (Snopek, GitHub fork `maximkulkin/…`) | **No.** Peer-to-peer rollback; every peer runs the same deterministic simulation | MIT | Godot 3 APIs; last commit Aug 2022; no releases | — | **Reject** |
| The **pattern**: client-side prediction with server reconciliation (Gambetta; Valve's Source networking) | **Yes**, in its simplest form: predict only your own body, adopt the server's answer on a difference, suspend reports while stale strides drain | — | Decades of practice | ~100 lines in `slice_link.gd` (§4.3) | **Adopt the pattern, implement it ourselves.** No library implements it against a non-Godot semantic server; the rule is one number and one suspension, smaller than any adapter |

## 8.3 Character controller

| Option | Fit | Licence | Maturity, maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **The slice's `Player` on `CharacterBody3D.move_and_slide`** | Fit; already the accepted controller (`VIS-3D-GODOT-2`), measured by `--drive`; three cameras observe it | ours on MIT engine | In use, accepted by the operator | Zero; 12e adds the press-through report (§4.3) | **Keep** |
| Jolt's own character (`CharacterVirtual`) | Fit in principle | MIT | Godot 4.7 does not expose it as a node; `CharacterBody3D` runs on the generic physics server API (to be re-verified in 16a) | Native code to bind it | **Reject**: not reachable from GDScript, and nothing is missing from `CharacterBody3D` |
| Community first-person controller templates (e.g. the COGITO immersive-sim template; generic "proto controller" scripts) | **Poor.** They bring their own interaction, inventory and door logic — rules a MineWorld client must not have | various (COGITO's repository URL answered 404 on 2026-10-08; unverified) | varies | Removing their rules | **Reject**: the controller exists and is accepted; a template would replace accepted feel with someone else's |
| Build a new controller | — | — | — | Re-doing accepted work | **Reject** |

## 8.4 Targeting and interaction UI

| Option | Fit | Licence | Maturity, maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Godot's physics ray** (`PhysicsDirectSpaceState3D.intersect_ray`, or a `RayCast3D` node on the camera) | Fit: acquisition only; walls occlude | MIT (engine) | In the engine | A few lines | **Adopt** (§4.4) |
| `Area3D` proximity "interactables" (the common Godot pattern: a trigger sphere around each thing) | **No.** A trigger radius is a reach rule in the client — exactly what `ENGINEERING_RULES.md` §7 gives to the System | MIT (engine) | Common | — | **Reject** |
| Interaction-component frameworks from templates (COGITO-style interactables, dialogue choices) | **No.** They decide what can be done to a thing in the client | various | varies | Stripping their rules | **Reject** |
| `nathanhoad/godot_dialogue_manager` (branching dialogue) | **No** for interaction: authored branching dialogue is a client-side source of lines and choices, while MineWorld's lines are the server's (`conversation-history`). Possibly later as a display widget only | MIT | Active: v4.1.0 for Godot 4.7 (4 Sep) | An addon for subtitles we already draw | **Reject now** |
| **Build:** a thin HUD over the ray and the observation's offers (§4.4) | Fit by construction: it reads `affordances()` and submits | ours | — | Small; the HUD and captions exist (`ControlsHud`) | **Build**, because every candidate either evaluates reach or authors lines in the client |

## 8.5 Object rendering and motion

| Option | Fit | Licence | Maturity, maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Godot primitives** (`BoxMesh`, `SphereMesh`) sized exactly to the disclosed shape | Fit: the drawn object is the collider is the server's shape | MIT (engine) | In the engine | Minimal | **Adopt** for geometry |
| CC0 props already in the repository (Poly Haven, Quaternius; `clients/3d-spike/ASSETS.md`, `DEP-8`) | Fit for appearance when a prop's proportions match a disclosed box or ball; must be scaled to the disclosed shape, never the reverse | CC0 | Already vetted under `DEP-8` | Choosing and scaling | **Adopt where one fits** (a crate, a ball), else the primitive with the slice's palette (QS14-13, taste) |
| `MultiMeshInstance3D` | Unneeded: at most 32 objects in a place (`OBJECTS_MAX`) | MIT | In the engine | — | **Reject** as premature |
| Godot's built-in physics interpolation (4.4+ in 3D) | Smooths physics ticks into render frames; it does not interpolate between network snapshots | MIT | In the engine | — | **Not the tool for this**; may be enabled for the player's own body independently |
| **`Tween`** between disclosed positions; a `path` when R-S11-1 delivers it | Fit: presentation only | MIT (engine) | In the engine | A few lines | **Adopt** |
| netfox's tick interpolator | Tied to netfox's tick loop | MIT | Active | Adopting netfox | **Reject** with §8.2 |

## 8.6 Figure motion between observations

| Option | Verdict |
| --- | --- |
| **Walk toward the newest observed position at a bounded speed, with the existing gait** (`NPC.step`) | **Adopt (build, ~40 lines).** The figures already have a gait driven by speed; a snapshot buffer is not needed at 10 Hz with strides the server bounds |
| A snapshot-interpolation buffer (render 100–200 ms in the past, interpolate between two snapshots) — the textbook approach, as in netfox and Source | **Reject now**: it adds latency to every figure to smooth a motion that the gait already smooths; revisit if figures stutter when played |
| `NavigationAgent3D` to path figures between observed positions | **Reject**: a navmesh decides where a person can walk, which is the server's (`ENGINEERING_RULES.md` §12); the server's positions are already reachable |

## 8.7 How the client is tested

| Option | Verdict |
| --- | --- |
| **The slice's own scripted probe** (`SliceProbe`), run against a real server | **Keep** — it is real execution, and every claim in §5 is a mode of it |
| GUT (Godot Unit Test) or gdUnit4, both MIT | **Not now**: the one unit-shaped check (the builder equals the offer, §4.4) runs inside the connected probe, against a real offer; adopting a test framework for one assertion is premature. Revisit if the client grows pure logic worth unit tests |

---

# 9. Modularity and pluggability

## 9.1 The 3D client is a removable presentation

The 3D client is a **client** in the presentation layer: a renderer of observations and a source of
requests. Its look is the default **Presentation Style Pack**, `presentation/mineworld-default/3D`
(`MODULE_SPEC.md` §6, `ARC-1`), whose hard constraint is that "swapping a Presentation Pack changes
nothing about the simulation". This step keeps both true:

```text
remove clients/3d-spike, mineworld-slice and mineworld-3d
  → cargo fmt / check / clippy / test: unchanged (no Rust crate names clients/)
  → every world loads, runs, validates and replays: unchanged
  → the 2D client (S12) and headless runs work: unchanged
```

16e runs that removal on a scratch copy and records it (the adversarial criterion of `overall.md` S14,
"Removing the 3D client changes no system").

## 9.2 Inside the client: small modules with one job each

```text
scripts/slice/
  slice_link.gd          the connection; reporting strides; the correction rule (§4.3)   (exists)
  intents.gd             every interaction request; offers matched; the only other file naming an
                         action type (§4.4)                                               (16a/12e)
  targeting.gd           the ray; what is targeted; nothing about validity                 (16a)
  client_bodies.gd       colliders from disclosure: place walls and solids, people, objects (12e)
  object_view.gd         drawing and moving loose objects (§4.5)                          (12e)
  figure_motion.gd       walking figures toward their observed position (§4.6)            (16c)
  slice_probe_world.gd   the connected probe modes (§5)                                   (16a, grows)
```

Each module is optional in the pluggable sense that matters here: in a world without `bodies`, no
`place-shape` and no `loose-objects` are disclosed, so `client_bodies.gd` builds nothing and the client
behaves as today; in a world without `economy`, no `buy` is offered and B shows nothing. Nothing in the
client switches on a world's name or its list of systems: what is drawn and offered follows from what is
disclosed.

## 9.3 What the 3D client shares with the 2D client, and what it does not

| Shared (one copy, `clients/protocol/mineworld`, or one definition in the server) | Each client's own |
| --- | --- |
| the frames, the seat, tokens, ids as strings, integers (`MineWorldClient`) | acquisition: a ray and keys (3D); clicks (2D) |
| reading an observation, offers, complete affordances, disclosed components (`MineWorldObservation`) | the scene, meshes, sprites, animation, cameras |
| the one axis conversion (`MineWorldSpace`) | local collision and the correction rule (3D only; the 2D client predicts nothing, step-11 §7.3) |
| the `AC-13` comparison (`server/src/parity.rs`) and the transcript format (§4.8) | wording, layout and input mapping |
| the obligations of `ADOPTION.md` §3 | |

## 9.4 Nothing leaks across layers

| Boundary | What could leak | Why it does not |
| --- | --- | --- |
| client → contracts | an engine type, a layer, a collider | the client sends `ActionRequest`s built by the shared module; nothing else can be sent (`PROTOCOL.md` §2) |
| client → systems | a reach, a radius, a capacity | the client never compares a position with a rule; the scan in §10 fails on a copied rule constant |
| systems → client | a rule the client must re-implement | the client reads verdicts (`available`, `unavailable_reason`) and disclosed state only |
| protocol module → 3D | a 3D-only concept in the shared module | §12's changes are readers of server frames; none names Godot 3D types |
| world content → client | a copy of a pack's geometry or layout | the client binds a place by its disclosed doorway and builds walls from disclosure; it quotes no pack file (`slice_link.gd`'s existing rule) |
| physics engine → client code | Jolt-specific calls | only the project setting names Jolt; the code uses the generic `PhysicsServer3D` nodes, so Godot Physics remains a one-line fallback (§8.1) |

---

# 10. Invariants (proposed; frozen only by the primary session or the operator)

- **I-S14-1 No rule in either client — executable, three ways.**
  1. *Static scan* (16a), a test in `tests/acceptance` beside the existing structural scans, reading every
     `*.gd` under `clients/` except `tools/` and `clients/protocol/demo/`:
     - every action type a System Pack declares (collected from `ActionTypeId::from_static("…")` under
       `systems/*/src`, so a new pack is covered without editing the test) may appear as a string literal
       only in the allow-listed files that build requests: `slice/slice_link.gd`, `slice/intents.gd`,
       and S12's equivalent;
     - an identifier naming a rule — matching `REACH|RANGE|CLEARANCE|NUDGE|MAX_STRIDE|CAPACITY` — may be
       declared only in an allow-list, each entry with its justification (the reporting cadence
       `REPORT_DIST`; the colliders' capsule, §4.2);
     - every finding names the file and the line.
  2. *Behavioural* (12e), the probe's `--world --rules`: the same scripted interactions — talk from the
     door and at the counter, shove, kick, throw — are played against social-cafe **with** bodies and
     against a test-time copy **without** bodies (and so without kick, throw and shove). The client's
     submitted interaction requests are identical in both transcripts; only the answers differ
     (`accepted` / `too_far_away` / `unavailable`). A client that decided anything would have submitted
     differently.
  3. *Shown to bite:* a planted `if distance > 0.8: return` in `intents.gd` makes (2) fail, naming the
     missing request; a planted `"kick"` literal in `targeting.gd` makes (1) fail, naming the line. Both
     are reverted and recorded, as the project's mutation rule requires.
- **I-S14-2 No framework, contract or system change.** `kernel/`, `contracts/`, `persistence/`,
  `server/src` and every System Pack have no diff in a 16x or 12e PR. A need for one is a material stop.
- **I-S14-3 Identities stay strings; every number sent is an integer** (`ADOPTION.md` §§3.1–3.2), through
  the shared module only.
- **I-S14-4 One correction rule, one number** (`ARC-S14-a`, 150 mm). No other code moves the player's body
  to a server position.
- **I-S14-5 Geometry from disclosure.** Every disclosed collider is built from the observation; the
  geometry probe holds the scene and the server to within 150 mm of each other (R-B4).
- **I-S14-6 The client reads no pack file.** Everything it knows about a world arrives in frames; no path
  under `worlds/` or `systems/` is read by a client script (scan, with I-S14-1's).
- **I-S14-7 The accepted slice does not regress.** `--drive`, `--measure`, `--threshold`, `--link`,
  `--conversation` and `--character` pass on every PR head that touches the client.
- **I-S14-8 Removable.** Removing the 3D client changes no Rust build, test, world or the 2D client (§9.1).
- **I-S14-9 `AC-13` is compared by the server's definition only** (`server/src/parity.rs`); no test writes
  its own comparison.
- **I-S14-10 The shared module stays dimension-free.** No change in §12 names a 2D or 3D engine type.

---

# 11. Requirements this step places on other steps

## 11.1 On S11 — the wire protocol (S11 owns every shape; these are needs)

| ID | Need | Exact shape asked for | Why | If not provided |
| --- | --- | --- | --- | --- |
| **R-S11-1** | **Perceived events in observations**, for the facts the observer is entitled to | `observation.events`: an array of the contract's `PerceivedEvent`, serialized as its `EventEnvelope` (`id`, `at`, `event_type`, `subjects`, `participants`, `place`, `caused_by`, `payload`, `visibility`, `provenance`), from the server's existing `recent_events` window. Each event delivered at least once; the client de-duplicates by `id`. Entitlement is perception's judgement (`perception.rs`), e.g. public events at the observer's place and events whose subjects include the observer. The 3D client reads `object-moved` (`object`, `from`, `to`, `how`, `by`, `path`), `person-shoved` and `stopped-short` | 12e's row item 5 ("objects animated along `path`"); showing *who* shoved you and *why* a stride stopped | Objects are tweened between disclosed positions (§4.5); "animated along `path`" in step-11 §11.1 is not met (QS14-18 **[OM if dropped]**) |
| **R-S11-2** | **Walking people in a hosted world** | `mineworld server <world> [--paced SEAT]... [--paced-all] [--pace SECONDS] [--seed N]`: each named seat (or every seat no client has asked to keep, with `--keep SEAT`) occupied by a `PacedRuleController` through the same `join`/`submit` seam `--agent` uses (`agent.rs`), consulted every `--pace` seconds of world time (wall time when hosted). A seat a controller occupies is not given to a client, or S11 states what happens | F-S14-22; `ENGINEERING_RULES.md` §10 "human + AI-controlled Persons sharing the same world"; Milestone/AC-15 demos | 16c cannot show a living street. QS14-10: this step could add it in `tools/cli/src` instead |
| **R-S11-3** | **Whole disclosure, even with deltas** | No new shape. If S11 introduces state deltas (`PROTOCOL.md` §9), the client must still be able to hold, per perceived entity, the whole of `self_location`, `passages`, `place-shape`, `loose-objects`, `display-name` and the affordance list; the first observation after `welcome` (and after a reconnect) is whole | §4.2–§4.5 rebuild colliders from these | A delta reader in the shared module (M-6) |
| **R-S11-4** | **Which of my requests an observation reflects** | An additive field on the observation frame: `"acted_through": "<ActionId>" \| null` — the newest `ActionId` allocated to a request submitted **on this connection** whose effects the observation's state includes; `null` before the first. Additive within revision 1, as `revision` was | Makes the correction rule exact (§4.3) | The heuristic rule with suspension (§4.3) |
| **R-S11-5** | **Ordering**, only if R-S11-4 is declined | A `result` frame for a request is written to the connection before any `observation` frame computed from a state that includes that request | The heuristic's baseline is only sound if answers are not overtaken | The suspension still converges; corrections may fire one observation late |
| **R-S11-6** | **Authentication** (`NETWORKING.md` §9, invite token + nickname) | Whatever `join` shape S11 specifies; the 3D client takes `--token=` and `--nickname=` and passes them to the shared module (M-5). No credential is stored in the repository | S11's acceptance | — |
| **R-S11-7** | **Reconnect** | A reconnecting client that joins the same seat of the same `instance` is the same observer, and its first observation is whole | The client re-adopts its position (`_reconcile`, kept for this) | The client treats it as a new session |
| **R-S11-8** | **Revision changes are coordinated** | If S11 raises `protocol` to 2, the shared module's `PROTOCOL` constant and both clients move in one coordinated PR | `MineWorldClient` refuses an unknown revision by design | Both clients stop connecting |
| **R-S11-9** | **Cadence** | The default 10 Hz stays the floor for a hosted world a 3D client joins | The correction latency and object motion assume ≤ 100 ms between observations | Corrections and motion visibly lag |

## 11.2 On S12 — the 2D client (coordination, not requirements on the protocol)

| ID | Item |
| --- | --- |
| **C-S12-1** | A scripted scenario mode in the 2D client that performs §4.8's five interactions by clicking, against a fresh hosted world, and writes its `submitted_request` transcript (with answers) in the existing evidence format, so `tools/cli/tests/ac13_clients.rs` can compare it with the 3D client's. The utterance is passed in (`--utterance=`), never typed into the 2D client's code |
| **C-S12-2** | One PR for the shared-module accessors M-1 … M-3 (§12): whichever of 16b and S12's first affordance UI is ready first lands them, and the other consumes them |
| **C-S12-3** | `F-41` (item and organization names) is assigned to S12 by `overall.md` §7. The 3D buy menu (16d) needs the same names, read from the same disclosure (R-F41 below) |

## 11.3 On S15 12d — the town's geometry and objects (content the client depends on)

| ID | Need |
| --- | --- |
| **R-12d-1** | The café's and the street's `body:` floors and solids match the slice's café room and walkable street, so `--world --geometry` passes (§4.2); doorway points at least 310 mm inside each floor (F-B7) |
| **R-12d-2** | The store's street-side doorway moves onto The Flower Room's door and the store's floor is that room (QS14-4); the workplace's, apartments' and park's street-side doorway points move onto existing non-enterable slice doors (F-S14-21). The positions are read from the slice's façades and checked by the geometry probe |
| **R-12d-3** | At least one box and one ball lie where the player meets them — in the café and on the pavement outside — at least 700 mm from every doorway point, so `AC-14`'s object interaction happens in the town and not only in `bodies-yard` |
| **R-12d-4** | `market-town` carries the same place geometry as `social-cafe` (its place files are copies, `ARC-35` check 3) |

## 11.4 On the packs that own names and shapes (outside every client)

| ID | Need | Owner |
| --- | --- | --- |
| **R-F41** | A display name for each item kind and organization, disclosed wherever the kind is disclosed (a shop's listing; holdings), so no client shows an id. Options for its owner: `naming`'s section carried by Item and Organization too, joined into listings at disclosure (economy's `Listing` precedent, F-O5); or economy's listing carrying the name | S12's plan (`overall.md` §7, F-41) |
| **R-S15-2** *(optional)* | `loose-objects`' listing carries the object's tags, so a client can pick a mesh by tag rather than by shape | bodies, a later PR |
| **R-S15-3** *(optional)* | bodies discloses each perceived person's body shape, so the client reads the capsule instead of holding the default (QS14-3) | bodies, when per-person shapes exist (QP-6) |

---

# 12. Changes to the shared module `clients/protocol/mineworld`

Every change below is a **reader of what the server already sends**, or a convenience over `submit`; none
decides anything, none names a 2D or 3D engine type (I-S14-10), and each is listed so the primary session
can sequence it with S12.

| ID | Change | File | Lands in | Needed by |
| --- | --- | --- | --- | --- |
| **M-1** | `complete_affordances(action_type := "") -> Array` — every affordance carrying a `payload`, of that type if given, in observation order; and `offers_about(id: String) -> Array` — every affordance whose `target` is `id` or whose `payload` has `id` as a top-level string value (so `kick { object }` is found by the object without the module knowing `kick`) | `observation.gd` | 16b (with C-S12-2) | 3D: kick, throw, shove, buy; 2D: buy, give |
| **M-2** | `submit_affordance(affordance: Dictionary, actor_location: Variant = null) -> String` — submits a complete affordance unchanged (`action_type`, `target`, `payload`), returning `""` and warning if it has no `payload` | `world_client.gd` | 16b | both |
| **M-3** | `component_value(id, component_type) -> Variant` — a disclosed component's payload as it arrived, whatever its JSON type (F-S14-15); `component()` keeps its dictionary contract | `observation.gd` | 16b | 3D: `place-shape`, `loose-objects` |
| **M-4** | `acted_through() -> String` on `MineWorldObservation`, and the observation frame's field carried into it | `observation.gd`, `world_client.gd` | with S11's R-S11-4 PR | 3D correction rule |
| **M-5** | `connect_to_world(address, seat, credentials := {})` passing S11's join fields | `world_client.gd` | with S11's R-S11-6 PR | both |
| **M-6** | `events_of(event_type) -> Array` and de-duplication by `id` (a set of seen ids on the client node); a delta reader if R-S11-3 brings deltas | `observation.gd`, `world_client.gd` | with S11's R-S11-1 / deltas PR | 3D objects; 2D whatever S12 shows |
| **M-7** | `ADOPTION.md`: §2's API list; §4.1 gains the correction rule as an obligation of a client that predicts; §6's "events" line updated when R-S11-1 lands | `ADOPTION.md` | with each of the above | both |

`MineWorldSpace` needs no change: an aimed throw's `toward` is `from_3d(point)` without its `z`.

---

# 13. PR split

Each PR is designed to the commit and frozen in its own turn; each starts in a fresh implementation
session in its own worktree (`CLAUDE.md` §3.1).

| PR | Scope | Starts | Integration checkpoint (run for real) | Adversarial criterion |
| --- | --- | --- | --- | --- |
| **16a** The client, ready for bodies | Jolt selected (DEP-14); the accepted checks re-run on it; ray targeting (`targeting.gd`) replacing the cone, people given pick colliders on `LAYER_BODIES` (not yet masked by the player); `intents.gd` taking `talk`; the I-S14-1 static scan; `slice_probe_world.gd` split out; the texture RID leak attributed and fixed or recorded with evidence; `MVP_STATUS.md`'s 3D rows corrected | **Now** — depends on nothing unmerged | `--drive`, `--measure`, `--threshold`, `--link`, `--conversation`, `--character` pass on Jolt; `--world --target` (§5) | Facing Alice through a wall targets nobody; a planted action literal outside the allow-list fails the scan by line |
| **16b** Shared module: complete affordances | M-1, M-2, M-3, and `ADOPTION.md` (M-7) | **Now**, coordinated with S12 (C-S12-2) | `clients/protocol/run.sh` against market-town: every complete `buy` is listed, one is submitted unchanged and accepted; against `bodies-yard` once 12c merges: each in-reach object's `kick` is found by `offers_about(object)` | Two complete affordances with one type and target are both returned, in order; an incomplete one is refused by `submit_affordance` without sending |
| **12e** The 3D client's bodies (S15) | §4.2 colliders from disclosure; §4.3 correction rule and press-through; people, objects and disclosed walls masked by the player; kick, throw (both forms) and shove through `intents.gd` and M-1/M-2; objects drawn and tweened (§4.5), along `path` if R-S11-1 has landed; `server/PROTOCOL.md` §6.2 and `ADOPTION.md` §4.1 (the rule) | **After 12c and 12d merge** (and 16a, 16b) | `--world --geometry`; `--world --bodies` (a)–(g); `--world --rules` (I-S14-1.2); frames of each, inspected | `--no-people-colliders`: the server still nudges and the client reconciles; the planted reach check fails `--rules` |
| **16c** A street people walk in | Figures walk between observations (§4.6); colliders follow them; decorative people off when connected (QS14-9); the florist bound to `store`; leaving through doorways; the hosted paced controller if QS14-10 assigns it here | **After 12d (R-12d-2) and R-S11-2** | `--world --street` (§5); the operator's checklist items 1 and 3 by hand | A person drawn leaving view away from every doorway fails the probe, naming the place — content drift between world and scene is caught, not seen |
| **16d** Buying, and the market world | `./mineworld-slice --world=market-town`; B lists the observation's complete `buy` affordances by name and submits one unchanged; wallet and holdings shown from disclosure | **After 12e, 16b, and R-F41** (S12's F-41) | `--world=market-town --buy` (§5) | The menu equals the observation's offers exactly: a mutation that hides one the client "thinks" unaffordable fails the probe; no id ever on screen |
| **16e** Demo B: `AC-13`, `AC-15`, the operator | `--scenario=ac13` and `tools/cli/tests/ac13_clients.rs` with S12's transcript (C-S12-1); three windows on one server (S11's server, S12's client, this client, a paced agent) with `AC-15`'s four identity lines; the removal check (§9.1); the `HUMAN_REVIEW_QUEUE.md` package (§4.9); `MVP_STATUS.md` | **After S12's client and C-S12-1, 16c, 16d** | `ac13_clients` passes for talk, shove, kick, throw (complete), buy; `AC-15` evidence recorded; the operator plays the checklist | A 2D transcript with another utterance fails naming `payload`; the scratch copy without `clients/3d-spike` passes the full Rust gate |

```text
            ┌── 16a ──┐
 now ───────┤         ├──────────────► 12e ──► 16d ──┐
            └── 16b ──┘     ▲           ▲              ├──► 16e ◄── S12 client (C-S12-1)
 12c ──► 12d ───────────────┴── 16c ◄───┘ R-S11-2      │
                                  └────────────────────┘
```

---

# 14. Files touched

```text
16a  clients/3d-spike/project.godot                  [physics] 3d/physics_engine="Jolt Physics"; input
                                                     actions for F, G, R, B
     clients/3d-spike/scripts/build.gd               LAYER_BODIES, LAYER_OBJECTS, LAYER_DISCLOSED
     clients/3d-spike/scripts/slice/targeting.gd     new: the ray
     clients/3d-spike/scripts/slice/intents.gd       new: talk moved here
     clients/3d-spike/scripts/slice/slice_link.gd    facing_person() → targeting; talk via intents
     clients/3d-spike/scripts/slice/slice_probe_world.gd   new: connected modes, --world --target
     clients/3d-spike/scripts/slice/slice_probe.gd   the moved modes removed; dispatch to the sibling
     clients/3d-spike/scripts/slice/slice_main.gd    the leak, if it is ours
     tests/acceptance/tests/client_rules.rs          new: I-S14-1.1
     docs/DECISIONS.md                               DEP-14 (QS14-16)
     docs/MVP_STATUS.md                              the 3D rows
16b  clients/protocol/mineworld/observation.gd, world_client.gd, clients/protocol/ADOPTION.md
     clients/protocol/demo/demo.gd                   a check of M-1/M-2 in the protocol demo
12e  clients/3d-spike/scripts/player.gd              mask; press-through intent
     clients/3d-spike/scripts/slice/client_bodies.gd, object_view.gd     new
     clients/3d-spike/scripts/slice/slice_link.gd    the correction rule; building bodies per observation
     clients/3d-spike/scripts/slice/intents.gd       kick, throw, shove
     clients/3d-spike/scripts/controls_hud.gd        prompts from offers; the throw marker
     clients/3d-spike/scripts/slice/slice_probe_world.gd   --geometry, --bodies, --rules
     server/PROTOCOL.md §6.2, clients/protocol/ADOPTION.md §4.1, docs/DECISIONS.md (ARC-S14-a)
16c  clients/3d-spike/scripts/slice/figure_motion.gd new
     clients/3d-spike/scripts/slice/slice_link.gd, slice_world.gd, streetscape.gd
     clients/3d-spike/scripts/slice/slice_probe_world.gd   --street
     (tools/cli/src/main.rs, agent.rs only if QS14-10 assigns R-S11-2 here)
16d  mineworld-slice                                 --world=<pack>
     clients/3d-spike/scripts/slice/intents.gd, controls_hud.gd, slice_probe_world.gd
16e  clients/3d-spike/evidence/**                    the AC-13 transcript, AC-15 logs, frames
     tools/cli/tests/ac13_clients.rs                 new
     docs/HUMAN_REVIEW_QUEUE.md, docs/MVP_STATUS.md, .structured-coding/plans/mvp0/*
```

No PR touches `kernel/`, `contracts/`, `persistence/`, `server/src`, `systems/`, `cognition/` or
`worlds/` (I-S14-2; worlds are 12d's).

---

# 15. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-S14-1** | Jolt changes the accepted slice's feel or one of its measured numbers (step-11 §3.2's known differences) | 16a re-runs every accepted check first and reports any moved number with both values; Godot Physics is a one-line fallback (QS14-12) |
| **R-S14-2** | The scene and the server's geometry disagree (R-B4), so the player is corrected at invisible walls | Disclosed colliders added beside the scene's (§4.2); the `--geometry` probe in 12e fails on a disagreement over 150 mm; 12d authors to the slice (R-12d-1) |
| **R-S14-3** | The correction rule oscillates while strides are in flight | The suspension until stale strides drain (§4.3); R-S11-4 makes it exact; `--bodies` (f) measures convergence within two observations |
| **R-S14-4** | Press-through looks like the player lunging when the server nudges | The press is bounded by one report's travel; the server bounds the advance to 300 mm past contact; the glide (QS14-11) smooths it; judged by the operator in 16e |
| **R-S14-5** | Paced NPCs, which attempt complete affordances, shove the player often in a hosted world (F-O8 predicts shoves are common) | Measured in 16c; the remedy is 12c's pack offer policy (p1: shove offered within 800 mm only), never the controller and never the client (QS14-15) |
| **R-S14-6** | Work waits on many steps at once (12c, 12d, S11, S12) | 16a and 16b start now; each later PR has one named gate; R-S11-1 and R-S11-4 have fallbacks |
| **R-S14-7** | `slice_probe.gd` keeps growing past the strong warning | Connected modes move to `slice_probe_world.gd` in 16a |
| **R-S14-8** | The shared module diverges between S12 and S14 | One owner per change (§12), one PR for M-1 … M-3 (C-S12-2) |
| **R-S14-9** | The `AC-13` comparison is made on requests that only look alike (same fields, one copied from the other) | Each flavour runs its own client, its own acquisition and its own fresh world; the transcripts are what `submitted_request` emitted; the utterance is an input to both |
| **R-S14-10** | Frames are inspected by the agent and taken as acceptance | The operator alone accepts (`ARC-11`); the package asks questions, it does not claim taste |

---

# 16. Questions (QS14-1 …)

**[OM]** marks operator-material questions — visual taste, scope, or a change to an operator decision.
The others are the primary session's to decide at freeze. Each has a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QS14-1** | **How does the 3D player nudge people** (QB-10), when local collision stops the body at contact? (a) press-through: report the intended stride when the input points into a person or object (§4.3); (b) no local collision with people — the original interpenetration, for up to a report; (c) collide locally and never nudge from 3D. | **(a).** It is the only option that honours both R-1 (no interpenetration) and QB-10 (nudging) in the client, and it decides nothing: it reports a wish, the server answers. |
| **QS14-2** | **Mask objects locally too** (with press-through), although step-11 §7.1 item 3 said the player does "**not** mask" objects "in the first version"? QB-7 (no local `RigidBody3D` push prediction) is unaffected. | **Yes.** One mechanism for people and objects; no walking into a box until the next observation. §7.1's sentence is refined, not an operator decision. |
| **QS14-3** | **Person capsules**: hold the server's default (r 300, h 1 720) as client constants beside the player's own, or ask bodies to disclose each person's shape (R-S15-3)? | **Constants now**, cited to bodies' `PERSON_RADIUS`/`PERSON_HEIGHT`, read from disclosure once per-person shapes exist (QP-6). |
| **QS14-4 [OM]** | **The store and The Flower Room.** The world's store stands where The Flower Room is; 12d aligns its doorway and floor (R-12d-2), and the client binds the florist volume to `store`. What does the room *look* like? (a) the accepted florist, selling the world's goods; (b) re-dressed as the corner store the world describes; (c) not enterable when connected. | **(a) now** (no visual change to an accepted slice; the binding is required in any case), with (b) offered as later visual-track work if the operator wants the look and the stock to agree. Visual taste and content: the operator's. |
| **QS14-5** | **Which world is Demo B?** `--world` stays social-cafe (with bodies after 12d); `--world=market-town` is added for buying (16d). | **Market-town for the Demo B package** (it has everything social-cafe has, plus buying); social-cafe stays the default of `--world`. |
| **QS14-6** | **F-41 (names of item kinds and organizations).** Assigned to S12 by `overall.md` §7. Which owner — `naming` carried by Item and Organization, or economy's listing carrying a name? | **`naming` carried by Item and Organization**, joined into listings at disclosure, so one pack owns every display name. S12's plan decides; 16d consumes it. |
| **QS14-7 [OM]** | **How is `buy` acquired in 3D?** (a) B anywhere in a shop opens the list of offers; (b) only while looking at the shop's counter or staff; (c) both. `ENGINEERING_RULES.md` §3: menus "may supplement … but should not replace spatial interaction". | **(a)** for 16d: the server's offer is "at that place" (F-O6), and (b) would invent a client-side rule about where buying happens. Spatially anchoring purchase is a pack question (a counter as a place part) for later. Interaction feel: the operator's. |
| **QS14-8 [OM]** | **The throw gesture**: G, aim, G (default aim when the aim never left the object); or hold G and release; or a separate key for the default throw. | **G, aim, G.** Taste: the operator's. |
| **QS14-9 [OM]** | **Hide the slice's decorative townspeople when connected?** They are not world Persons (F-S14-10). This changes how the accepted slice looks when connected. | **Hide them when connected**, once 16c makes the world's own people walk the street; keep them offline. |
| **QS14-10** | **Who builds R-S11-2** (a hosted paced controller)? S11 (server composition) or this step's 16c (`tools/cli/src`, the composition root that already hosts `--agent`)? | **S11**, as part of hosting. If S11 declines, 16c adds it, and `tools/cli/src` enters 16c's change set. |
| **QS14-11 [OM]** | **How a correction looks**: glide differences up to 1 m over 120 ms and snap larger ones; or always snap. | **Glide ≤ 1 m.** Taste. |
| **QS14-12 [OM if the fallback is taken]** | If Jolt breaks an accepted slice check in a way the client cannot fix, fall back to Godot Physics and amend DEP-14? The operator approved Jolt as the direction (step-11 §1.2 D-2). | **Fix in the client if bounded; otherwise stop and return to the operator** with both measurements before falling back. |
| **QS14-13 [OM]** | **How objects look**: primitives in the slice's palette, or CC0 props scaled to the disclosed shapes. | **CC0 props where one fits the shape, else primitives.** Taste. |
| **QS14-14** | **The 12e/S14 division** (§4.1). | **A**: 12e stays S15's last PR, planned here; S14 is 16a–16e. **B** would change a frozen split and is then **[OM]**. |
| **QS14-15 [OM]** | **If paced NPCs shove the player too often** in a hosted world (R-S14-5), which remedy? (a) accept; (b) 12c's offer policy p1 (shove complete only within 800 mm) for every world; (c) a hosted-controller option that skips physical offers. | **Measure in 16c first.** If more than one shove per minute lands on a standing player, **(b)** — the pack's offer policy, as 12c's ladder already names it — never the client. Gameplay feel: the operator's. |
| **QS14-16** | **Where DEP-14 is recorded**: 16a (where Jolt is selected) or 12e (as step-11 planned). | **16a**: the record belongs with the change it justifies. |
| **QS14-17** | **`AC-13`'s scope**: `talk` only (as `MVP.md` words it) or talk, shove, kick, throw and buy (§4.8). | **All five.** Step-11 §7.3 claims parity for the three S15 actions; testing only `talk` would leave that unproven. |
| **QS14-18 [OM if dropped]** | **`path` animation** (12e row item 5) needs R-S11-1. If S11 does not deliver perceived events before 12e, ship with tweens and keep the item open, or drop it? | **Ship with tweens, keep the item open** until R-S11-1 lands; dropping it changes a frozen row. |

---

# 17. Proposed edits to other documents (not applied here)

## 17.1 `overall.md`

- §3 S14: add `**Design:** [step-15-demo-3d.md](step-15-demo-3d.md)`; "PRs 16a–16e, with S15's 12e planned
  there"; the dependency line gains "S15 (12c, 12d, 12e) and S12 (for `AC-13`)".
- §4: `AC-13`'s row adds "talk, shove, kick, throw and buy (QS14-17)".
- §7: under "Remaining", S14 becomes "S14 (16a–16e; step-15, DRAFT)"; the visual-track lines unchanged.

## 17.2 `step-11-bodies.md`

- Header: "PR 12e is planned to the PR in `step-15-demo-3d.md` (§4, §13), and detailed to the commit
  there when 12d merges."
- §11.1's 12e row, refined (not replaced): "people, objects **and the current place's walls and solids**
  get colliders from disclosure; the player **reports the stride it intended** when pressing into a person
  or object (QS14-1); reconciliation by `ARC-S14-a`; kick and throw are target-less with the object in the
  payload (QO-6), acquired by the camera ray; objects animated along `path` when R-S11-1 is available,
  tweened otherwise (QS14-18)."
- §7.1 item 3: "masked by the player, with press-through (QS14-2)".

## 17.3 `docs/DECISIONS.md`

- **DEP-14** — step-11 §15.2's draft, with this file's §8.1 table as its options and the 2026-10-08 facts.
- **`ARC-S14-a` — A predicting client adopts the server's position on a difference over 150 mm**: §4.3's
  rule and its suspension, R-S11-4 as the exact form; the one number; nothing else moves the body.

## 17.4 `docs/MVP_STATUS.md`

- The "3D client" row (line 45): "the accepted slice (`VIS-3D-GODOT-2`), connected to the town: walk,
  enter the café, talk; bodies, objects and buying are S15 12e and S14 (step-15)".
- The S14 row (line 106): "planned (step-15, DRAFT): 16a–16e; 12e with S15".

## 17.5 `server/PROTOCOL.md` and `clients/protocol/ADOPTION.md`

- `PROTOCOL.md` §6.2: after the reporting rule, the correction rule as the client's obligation when it
  predicts (12e), citing `ARC-S14-a`.
- `ADOPTION.md`: §2 (M-1 … M-3), §4.1 (the correction rule), §6 (events, when R-S11-1 lands) — with each
  change in §12.

---

# 18. PR 16a — the client, ready for bodies (full design; DESIGN FROZEN 2026-10-08)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session` — record in §18.0. Implementation is
authorized by that freeze (`CLAUDE.md` §3.1; overall "Parallel build-out, 2026-10-08", ruling 9).

## 18.0 Freeze record

## DESIGN FROZEN

```text
Design revision     §§18.1–18.6 as committed in fe3e814
Approved by         the primary session, 2026-10-08 ("FREEZE: PR 16a is DESIGN FROZEN (2026-10-08),
                    primary session"), relayed to this implementation session
Implementation base main @ 47c81d1; branch mvp0/pr-16a-jolt-targeting
Execution contract  §18.8
Lifecycle           FROZEN
Rulings             §18 accepted as written, including the two departures (only LAYER_BODIES; the
                    tools-only commit). Q-16a-1 accepted: a small centre dot and a "looking at: …"
                    HUD line, connected only — a 3D visual default delegated to the primary session,
                    judged by the operator in play; the HUMAN_REVIEW_QUEUE checklist wording gains it.
                    Q-16a-2 … Q-16a-6 accepted as recommended. Report Bob's occlusion of the door talk
                    as measured. One Godot window at a time; 16b and S12 13a also run Godot, so a
                    stalled capture is re-run
```

## 18.1 Identity, base, approved scope

```text
PR            16a — the 3D client, ready for bodies (S14, first of 16a … 16e; GitHub number assigned
              at freeze, ruling 7)
base          main @ 47c81d1 (Merge #62, the six parallel step designs frozen)
branch        mvp0/pr-16a-jolt-targeting, worktree /Users/yuema137/mineworld-worktrees/impl-s14-16a,
              held by this session only
audit         §18.2 (files and symbols read on 47c81d1, 2026-10-08) on top of §2
scope         §13's 16a row: Jolt selected and every accepted check re-run on it; ray targeting
              replacing the cone, people given pick colliders on LAYER_BODIES (not masked by the
              player); intents.gd taking talk; the I-S14-1 static scan; slice_probe_world.gd split
              out; the texture RID leak attributed and fixed, or recorded as engine-side with evidence;
              docs/MVP_STATUS.md's 3D rows corrected
decisions     DEP-20 (Jolt; the record step-11 and §§4.3, 8.1, 14, 16 of this file call "DEP-14",
              renumbered by overall ruling 6). 16a takes no ARC number: ARC-50 … 52 stay for 12e and
              later S14 PRs
```

**Non-goals (each belongs to a later PR, and 16a must not start it):** the player masking people,
objects or disclosed walls; disclosed geometry; the correction rule and press-through (12e); kick,
throw, shove, buy, offer-driven HUD prompts (12e, 16d); any change to `clients/protocol/mineworld/**` or
`clients/protocol/ADOPTION.md` (the shared module is 16b's and S11's only, ruling 4); figure motion and
decorative people (16c); any server, kernel, contract, persistence, System Pack or `worlds/**` change
(I-S14-2).

**Binding constraints carried in.** The accepted visuals `VIS-3D-GODOT-1` (route D+, the interim
standard) and `VIS-3D-GODOT-2` (the slice, accepted after the operator played it) must not regress
(I-S14-7). The 3D visual defaults (QS14-*) are chosen by the primary session and judged by the operator
in play (overall, 2026-10-08); 16a therefore *proposes* the one visual element it needs (Q-16a-1) and
does not decide it.

## 18.2 Re-audit for 16a (`main @ 47c81d1`, 2026-10-08)

Every row was read in this session from the file named. §2's findings stand; these add what 16a's
file-level plan needs.

| ID | Finding | Evidence | Consequence for 16a |
| --- | --- | --- | --- |
| **A16-1** | One Godot project holds both scenes. `project.godot` has no `[physics]` section; `main.tscn` (`./mineworld-3d`, the promenade with the character's `--sweep`, `--headtrace`, `--frametime`, `--drive`) and `slice.tscn` (`./mineworld-slice`) both run on whatever engine it selects. Godot is `4.7.2.stable.official.ed1daf0bf` on this host | `clients/3d-spike/project.godot`; `godot --version` | The one-line switch changes **both** accepted artefacts: the route D+ evidence (`--sweep`, `--headtrace`, `--frametime`, all in `shots.gd`) is re-run as well as the slice's checks |
| **A16-2** | The slice's checks: `--drive`, `--measure`, `--threshold`, `--perf` (frame cost at four viewpoints), `--character`, `--shots`, `--link`, `--conversation`, dispatched by `SliceProbe._process`; `scripted()` and `with_hud()` list the flags. The launcher maps `--x` to `--slice-x` and runs `--drive` and `--link` headless | `scripts/slice/slice_probe.gd:102–165`; `mineworld-slice:53–64` | A new mode needs a flag in the launcher, in `scripted()`, and a dispatch arm |
| **A16-3** | The connected modes are one contiguous block: `_link_check` (303–472), `_conversation_frames` (479–555), `_street_watch` (562–604), `_talk_to` (609–631), `_walk_to` (635–639), `_answered` (642–646). `_walk_to` and `_answered` are used only by them. The standalone helpers they also call (`_hold`, `_settle`, `_save`, `_walk_dist`, `_walk_to_x`, `_inside_room`) are used by standalone modes too | `slice_probe.gd` (grep of every call site) | The block moves to the sibling unchanged; the shared helpers stay where they are and are reached by inheritance (§18.4, D-16a-3) |
| **A16-4** | Collision layers: `Build.LAYER_WORLD = 1` is the only layer; every scene collider is a `StaticBody3D` on it; the player is `collision_layer = 0`, mask `LAYER_WORLD`; the third-person boom's ray masks `LAYER_WORLD`. The probe's own measurement ray (`_ray`, `:1264`) and sun ray (`:1198`) use the default mask (all layers) | `build.gd:6, 28–157`; `player.gd:71–82`; `camera_rig.gd:172–173`; `slice_probe.gd:1198, 1265` | A person collider on layer 2 is invisible to the player and the camera. The probe's two rays would see it: they run only in standalone modes, where no perceived figure exists, but they are given `LAYER_WORLD` explicitly so a later connected measurement cannot be silently changed (bounded) |
| **A16-5** | The player's capsule is r 0.30, h 1.72, centre 0.86 m up, written as literals in `Player._ready`. The first-person eye is `CameraRig.EYE_HEIGHT = 1.66` | `player.gd:77–81`; `camera_rig.gd:32` | The pick collider uses the same capsule; the numbers become one named constant pair in `Player`, cited to bodies' `PERSON_RADIUS`/`PERSON_HEIGHT` (§4.2, QS14-3) |
| **A16-6** | The cone, `facing_person()`, is called only by `talk_to_facing`; `talk_to_facing` is called by the E handler and by the probe's `_talk_to`. The literal `"talk"` appears in `slice_link.gd` five times (`may`, `unavailable_reason`, `submit`, two token bookkeeping lines) | `slice_link.gd:431–472, 491, 522–525`; `slice_probe.gd:614` | All five move to `intents.gd`; afterwards `slice_link.gd` names `move` only |
| **A16-7** | Which world people stand where (social-cafe, hosted): visitor (the seat) at café (1610, 600); Alice (barista) at café (6000, 8000); Bob at café (4500, 6100); Wes (`wanderer`) at café (7110, 3900); Ivan on the **street** at (−4000, 1000). The café door is at café (1610, 200) ↔ street (0, 3000); in the scene the café door is x = 3.45, the pavement point z = −7.90, The Flower Room's door x = 11.84 | `worlds/social-cafe/people/*.yaml`; `places/cafe.yaml`; `cafe.gd:36`; `street.gd:35`; `slice_probe.gd:1428` | From the door, the line to Alice passes Bob's axis at **320 mm**. A capsule's horizontal section at the eye height of 1.66 m is 0.18 m in radius, so a ray aimed at Alice's head clears Bob by about 140 mm: the accepted "face Alice from the door, E → too far away" survives a ray, by measurement still to be taken (R-16a-2) |
| **A16-8** | **A non-vacuous wall test needs a person perceived behind a wall.** The observer perceives only its own place (F-S14-17), so "from the street, facing Alice through the back of the building" (§5) targets nobody **because Alice is not perceived**, not because of the wall: it cannot tell a ray from a cone. But the slice reports a body inside The Flower Room as being on the **street** (`PLACE_KEY[FLORIST_PLACE] = "street"`, F-S14-9), so from the back of the florist the street's people — Ivan — are perceived, and the florist's west party wall stands between | `slice_link.gd:70–74`; A16-7 | `--world --target`'s wall case is taken there (§18.3 T-2). It is valid until 16c binds the florist to `store`; 16c re-designs it then (recorded so it is not forgotten) |
| **A16-9** | **Action-type literals in client code today** (comments excluded): `slice_link.gd` (`"move"`, `"talk"` ×5); `human.gd:422` `mode.set_input_name(0, "move")` — an `AnimationNodeBlendTree` input name, not a request; `clients/protocol/demo/demo.gd` (excluded by I-S14-1.1); `clients/protocol/mineworld/world_client.gd` names `"talk"` only in `##` comments and `"join"` as a wire frame type (not an action type). Action types declared under `systems/*/src`: `buy eat drink invite accept-invitation decline-invitation join-group-activity leave-group-activity give move talk` (11; `bodies` declares none until 12c) | `grep` over `clients/**/*.gd`; `grep 'ActionTypeId::from_static' systems/*/src` | The scan must strip comments, and needs one entry-level allow-list line for `human.gd`'s `"move"` (Q-16a-5) |
| **A16-10** | **Rule-named constants in client code today:** none outside `clients/protocol/demo` (`STRIDE`, `reach`). `REPORT_DIST` and the capsule do not match I-S14-1.1's pattern | `grep` of `const`/`var` declarations over `clients/**/*.gd` | The rule-constant allow-list starts **empty**; the justifications §10 anticipated are not needed |
| **A16-11** | **Pack paths in client code today:** only in comments (`slice_world.gd:17`, `cafe_interior.gd:311`, `slice_probe.gd:297`) | `grep '(worlds\|systems)/'` | I-S14-6's scan, over string literals, starts clean |
| **A16-12** | **The leak's candidates.** `SliceMain._exit_tree` frees `Props._scenes`. Other script-static caches hold engine resources until script teardown, which may come after the rendering server's: `Mats._tex_cache`, `_mat_cache`; `Procgen._cache`; `Props._card_mats`; `Human._scenes`, `_libs`, `_mats`; `SlicePalette._rug`, `_serif`, `_script_font` (font atlases are textures) | `grep '^static var' scripts/**/*.gd`; `slice_main.gd:67–77` | Bisection over these is the investigation's first step (§18.3 C6) |
| **A16-13** | **No CI workflow** exists (`.github/workflows` absent; S13's 13a is in flight). The full Rust gate is local | `ls .github` | CI repair is `N/A`; the local gate runs once on the final head (§18.6) |
| **A16-14** | `./mineworld-slice --world` hosts on the fixed address `127.0.0.1:7979`. Parallel sessions may hold that port | `mineworld-slice:69–79` | Before each connected run the port is checked; if held, the same server command is started on a free port and the slice joined with `--server=` — the path `--world` itself takes (R-16a-3) |

## 18.3 Acceptance (decided before measuring, `ARC-23`)

Every check is run by the real client, against the real server where connected, exactly as the
operator runs it (`ENGINEERING_RULES.md` §19). "Before" is `main @ 47c81d1` on Godot Physics, measured in
this session immediately before the switch; "after" is the same command on the PR's head. Every number
that moves is reported with both values, whether or not it passes.

```text
J-0  WHICH ENGINE RUNS. Before: the running 3D physics server is Godot Physics; after: Jolt. Shown by
     the server's class as the engine reports it, and by --drive printing it (J-1 guard); if the class
     name does not name the engine, a behavioural discriminator is found and recorded first (F-S14-2)

J-1  THE SLICE, standalone, on Jolt — each check's own verdict line, unchanged from before:
     --drive       "all drive checks pass": walk-in, the 13.3 m loop closing within 0.10 m, back wall /
                   counter / glazing stop the body, three cameras with 0.0000 m body movement on each
                   switch, jumps 0.40-0.50 m (0.49 accepted), the Flower Room loop within 0.60 m (0.12
                   accepted). It also prints the running engine and fails if it is not Jolt
     --measure     every range it checks still in range
     --threshold   café and florist: worst step < x3, interior luma above the floor, ≤ 4.5 % clipped
     --character   all character checks pass (the slot's body animates; every camera mode)
     --perf        frame cost at the four viewpoints (frame time, slice)
J-2  THE CHARACTER (VIS-3D-GODOT-1's evidence), promenade scene, on Jolt:
     --headtrace   reference body, steady walk and jog: yaw, pitch and roll each within 6° peak-to-peak
                   (accepted: walk 3.8/0.4/0.2, jog 4.2/1.1/0.4)
     --sweep       48 frames (walk and jog × rear, rear three-quarter, front three-quarter × 8 phases):
                   inspected one sheet at a time for the defects the operator flagged — no hand or
                   sleeve into the pack or hoodie, pack on her back, no slits, head facing ahead
     --frametime   stand and walk, frame and GPU medians and p95 (frame time, character)
     ./mineworld-3d --drive   every printed number reported before/after; every "-> ok" still ok
J-3  FRAME TIME. No viewpoint's median frame time (--perf, --frametime) rises by more than 15 %, confirmed
     by one re-run; a confirmed rise above that is a material stop (an accepted artefact's measured
     property), reported with both runs
J-4  FRAMES at working resolution (1600x900, the slice's normal): --shots before and after, every view
     compared by a pixel-difference tool (mean absolute difference per view, and the share of pixels
     differing by more than 8/255). Static views are expected to match within noise; every view over
     1 % differing pixels is looked at side by side (one image at a time) and its cause named. A
     visible change to the accepted look that is not explained by the body settling a few millimetres
     differently is a regression and a material stop
J-5  CONNECTED, on Jolt (./mineworld-slice --world, or the same server on a free port, A16-14):
     --link          "all link checks pass": seated; placed where the world says; café -> street ->
                     café on foot with every move accepted and none refused; two jumps; the server's
                     last position equal to the last report; talk from the door answered too_far_away;
                     on foot to the counter, every move accepted; talk accepted and the reply heard;
                     the street watch's simulated seconds equal its wall seconds within 10 %
     --conversation  "conversation on screen, no ids", the door toast "can't talk to Alice Moreau: too
                     far away", both caption lines at the counter
     (both re-run again after C4, when targeting is a ray: the door talk must still go to Alice)
```

```text
T-1  RAY, positive (--world --target, connected, headless): from the visitor's seat just inside the
     door, the camera aimed at Alice's head: the target is Alice's entity id; E submits talk and the
     answer is too_far_away (rejected or refused) — the client sent it although the world says no.
     At the counter in first person and in third person rear: the target is Alice
T-2  RAY, through a wall: walked on foot out of the café, east along the pavement and into The Flower
     Room, to the back of its west lane (about (11.84, -13.2)); the camera aimed at the head of the
     person the world shows standing on the street (Ivan, A16-8): the target is nobody, and the
     probe prints the first collider the ray met, which is an opaque wall of the florist, not glass.
     Counterfactual in the same run: the cone rule of 47c81d1, computed by the probe from the same
     camera and figures, would have chosen that person — so the case discriminates
T-3  RAY, the positive control for T-2: from the pavement with a clear line, the same person is
     targeted
T-4  RAY, nothing: aimed at an empty wall or the sky, the target is nobody and E says "nobody in view
     to talk to" and sends nothing
```

```text
S-1  THE SCAN (tests/acceptance/tests/client_rules.rs, I-S14-1.1 with I-S14-6), green on the PR:
     every *.gd under clients/ (symlinked directories not followed; clients/*/tools/ and
     clients/protocol/demo/ excluded), comments stripped, string literals read:
     a) a literal equal to an action type declared under systems/*/src (collected from
        ActionTypeId::from_static("…"); fewer than one collected fails closed) appears only where an
        allow-list entry (file, literal, reason) admits it: slice_link.gd "move"; intents.gd "talk";
        human.gd "move" (an animation input, Q-16a-5)
     b) no const or var declares a name containing REACH, RANGE, CLEARANCE, NUDGE, CAPACITY or
        MAX_STRIDE (word-split as the I-2 scan splits), unless an allow-list entry admits it (none in
        16a)
     c) no literal names a path under worlds/ or systems/
     every finding names file:line and the literal or name; an allow-list entry that admits nothing
     fails, so the list cannot go stale; a listed root that is missing fails
```

**Mutations, one per guard, each planted in the working tree, run, recorded, reverted** (`git status`
clean of it afterwards):

| # | Guard | Mutation | Expected red |
| --- | --- | --- | --- |
| M-1 | J-0 / `--drive`'s engine line | remove the `[physics]` line | `--drive` fails naming the running engine |
| M-2 | T-2 (walls occlude) | targeting by the cone again (the 47c81d1 rule restored inside `targeting.gd`) | T-2 fails naming the person targeted through the wall |
| M-3 | T-2 (walls occlude) | the ray's mask without `LAYER_WORLD` | T-2 fails as M-2 |
| M-4 | T-1 (the client sends regardless) | `intents.gd` returns without submitting when `may("talk", target)` is false | T-1's door talk reports `NO ANSWER` instead of `too_far_away` |
| M-5 | S-1 a | a `"kick"` literal planted in `targeting.gd` (12c's name, absent from the build) **and** a `"talk"` literal planted in `targeting.gd` | the scan fails on `"talk"` naming `targeting.gd:<line>`; `"kick"` is **not** reported, because no pack declares it yet — recorded as the scan following the build, not a miss |
| M-6 | S-1 b | `const TALK_REACH := 2.0` planted in `intents.gd` | fails naming the file, line and name |
| M-7 | S-1 c | `load("res://../../worlds/social-cafe/people/alice.yaml")` planted in `slice_link.gd` | fails naming the line |
| M-8 | S-1 staleness | an allow-list entry for a literal that occurs nowhere | fails naming the entry |
| M-9 | S-1 fail-closed | the action-type root pointed at a directory with no declarations | fails naming the cause |
| M-10 | the comment stripper | a `"talk"` inside a `#` comment **and** a `"#"` inside a string literal followed by `"talk"` on the same line | the first is not reported, the second is (unit test of the lexer, kept) |
| M-11 | the leak fix (if it is ours) | the fix removed | the exit line `RIDs of type "Texture" were leaked` returns |

```text
L-1  THE LEAK. Either: (a) attributed to named resources, fixed, and every slice mode and the plain
     launch exit with no "were leaked" line (M-11 shows the fix is the cause); or (b) shown to remain
     with every script-static cache released and in a minimal scene, recorded as engine-side with
     that evidence and the engine version — in MVP_STATUS and HUMAN_REVIEW_QUEUE's limitation 8
P-1  THE SPLIT. slice_probe_world.gd holds every connected mode; slice_probe.gd holds none; --link and
     --conversation print the same lines as before the split, apart from timing and run-dependent ids
D-1  STATUS. docs/MVP_STATUS.md's 3D rows say what is true on the PR head (§17.4, A16-* where newer)
G-1  GATES on the final executable head: cargo fmt --check, clippy -D warnings, cargo test --workspace
     (one run), check_decision_ids, check_doc_headings — all pass; no file under kernel/, contracts/,
     persistence/, server/src, systems/, cognition/, worlds/, clients/protocol/ in the PR diff
```

## 18.4 Design decisions (D-16a-*)

| ID | Decision | Alternatives considered | Why |
| --- | --- | --- | --- |
| **D-16a-1** | **Jolt by the project setting alone**, `[physics] 3d/physics_engine="Jolt Physics"`, every other Jolt setting at its default; the values in force are printed by the engine probe and recorded | tune Jolt's settings up front; keep Godot Physics (§8.1) | §8.1's verdict (adopt). Defaults are what the engine's maintainers chose for new projects; a setting is changed only if a J-check fails and the change is bounded (then recorded) |
| **D-16a-2** | **Targeting is a stateless helper**, `scripts/slice/targeting.gd` (`SliceTargeting`, `RefCounted`, static functions): `aim(camera) -> Dictionary` casts one ray from the active camera through the viewport centre, `RAY_LENGTH = 30.0` m, mask `LAYER_WORLD \| LAYER_BODIES`, `collide_with_areas = false`; it returns the hit's entity id (read from the `entity_id` metadata of the collider's figure), the hit point and the collider's name; `person_collider() -> AnimatableBody3D` builds the pick capsule | a `RayCast3D` node on each camera (three cameras, mode switching); an `Area3D` per figure (§8.4: a radius is a reach rule — rejected) | §8.4 adopts the engine ray. A query from the *active* camera needs no node per camera and works across the three modes. The player is on no layer (A16-4), so a third-person ray never meets her own capsule and needs no start offset |
| **D-16a-3** | **Pick colliders.** Each perceived figure (`SliceLink._figure`) gets one `AnimatableBody3D` child, capsule `Player.CAPSULE_RADIUS` × `Player.CAPSULE_HEIGHT` (0.30 × 1.72, centred 0.86 m up), `collision_layer = LAYER_BODIES`, `collision_mask = 0`. `Build` gains `LAYER_BODIES = 2` only; `LAYER_OBJECTS` and `LAYER_DISCLOSED` (§4.2) are added by 12e with their first use | `StaticBody3D` (moved by teleport: Jolt and Godot discourage moving static bodies); declaring all three layers now as §14 lists | 12e masks the same body (§4.2), so its type is the one 12e needs. An unused constant is premature (`CLAUDE.md` §4 rule 11): a bounded deviation from §14's list, recorded |
| **D-16a-4** | **`intents.gd`** (`SliceIntents`, `RefCounted`, owned by the link): `TALK`, the request's name; `talk(client, target, utterance, actor_location) -> String` submits **regardless of the affordance** and remembers the request by token; `take(token) -> Dictionary` hands back `{ action, target, words }` for the answer's wording; `server_note(obs, target) -> String` returns the world's stated reason when it says talking is unavailable, for display only. The link keeps `move`, the E key, display labels and toasts, and asks intents what an answered token was | keep `talk` in the link (two files naming actions, no seam for 12e's three); a Node with signals | §4.4: one file builds every interaction request. 12e adds kick, throw and shove there and nowhere else |
| **D-16a-5** | **The probe split by inheritance**: `slice_probe_world.gd`, `class_name SliceProbeWorld extends SliceProbe`, holds the connected modes (`link`, `conversation`, the new `target`) and their helpers; `SliceProbe._process` calls an overridable `_run(mode)`; `SliceMain` instantiates `SliceProbeWorld` when a connected mode is requested. `scripted()` asks both | a third helper file both use (moves the shared walking helpers: more of the accepted probe edited); composition with a reference to the base probe (makes its private helpers public) | The connected modes *are* probe modes that need the same walking and capture helpers; inheritance moves only the connected block and edits the base in two places (dispatch, `scripted()`), which is what §5 asks ("not edited beyond what their claims need") |
| **D-16a-6** | **The scan strips comments.** A GDScript lexer in the test: `#` to end of line outside a string; `"…"`, `'…'`, `"""…"""` with backslash escapes; `&"…"`/`^"…"` (StringName/NodePath) read as strings | read comments too, as `precursor_vocabulary.rs` and `seam_vocabulary.rs` do | Those scans hold the *absence of vocabulary*; this one holds *where requests are built*, and a word in a comment builds nothing. The protocol module's documentation (`world_client.gd`'s `submit("talk", …)` example) may not be edited by 16a and is right to stay |
| **D-16a-7** | **Evidence**: before/after numbers and every decisive line go into this section's ledger (§18.7); side-by-side frames for any view J-4 flags, and the sweep sheets before/after, go to `clients/3d-spike/evidence/16a/` (with `.gdignore`), the directory 16e will use; raw captures stay in the ignored `shots/` | commit every frame | Small, reviewable, and the operator can open what matters |
| **D-16a-8** | **Tools**: `clients/3d-spike/tools/physics_engine.gd` (headless `--script`: prints the configured setting and the running server's class) and `tools/frame_diff.gd` (headless `--script`: two PNGs → mean absolute difference, share of pixels over a threshold, the bounding box of the differing region; optional side-by-side through the existing `compare.gd`) | ad-hoc scripts in `/tmp` (F-S14-2's probe was one, and is lost) | J-0 and J-4 become re-runnable by anyone; tools are excluded from the scan by I-S14-1.1 |

## 18.5 Questions for the freeze (primary session; **[OM]** = the operator's)

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-16a-1 [OM: visual default, judged in play]** | **A ray needs a reticle.** The cone accepted anything within ~37° of the view's centre; a 0.30 m capsule at 8 m is ~2°. Without a mark at the screen's centre the accepted checklist step 8 ("from the door, face Alice and press E") becomes hard to do by hand. (a) A small centre dot, and the targeted person's name on the HUD's world line ("looking at: Alice Moreau"), **only when connected**; (b) a dot only; (c) nothing until 12e's offer prompts | **(a).** Standalone frames and `--shots` are unchanged (no link, no reticle), so J-4 still isolates Jolt. The dot is 4 px, the slice's HUD colour; the operator judges it in play |
| **Q-16a-2** | T-2's wall case at the back of The Flower Room, which holds only until 16c binds the florist to `store` (A16-8) | **Yes**, with 16c re-designing it; the counter is kept as a second occluder in T-1's report (aiming at Alice's knees from the customer side meets the counter first) |
| **Q-16a-3** | J-3's frame-time bound of 15 % median, confirmed by a re-run | **Yes**; Jolt is not expected to move render cost, and 15 % sits above run-to-run noise measured on this machine (10.9–17.6 ms across viewpoints in the 2026-09-30 preview) |
| **Q-16a-4** | 16a's handoff: a separate file collides between parallel lanes (`handoff.md` is shared); keep 16a's continuation state in §18.8 of this section instead | **Yes**: one authority, no collision |
| **Q-16a-5** | `human.gd`'s `"move"` (an `AnimationTree` input name): an allow-list entry with its reason, or rename the input | **Allow-list.** Renaming touches the accepted character's animation graph for no behavioural gain |
| **Q-16a-6** | `HUMAN_REVIEW_QUEUE.md` `VIS-3D-GODOT-2` known limitation 8 ("Not attributed") becomes false once L-1 holds: append the attribution to that line | **Yes**, one line, appended, nothing else in the accepted entry edited |

## 18.6 Commit plan

Each commit tracks implementation, validation and review separately; a planned commit may become
several coherent ones (mapping recorded). Evidence goes into §18.7 as `E16a-<n>`. Every Godot run is in
the background when it may exceed two minutes, one Godot window at a time, with stdout to a log under
`clients/3d-spike/shots/16a/` (ignored).

### 16a-C0 — Design (this section) — docs only

- [x] Implementation: §§18.0–18.8, from the audit in §2 and §18.2.
- [x] Validation: `python3 scripts/check_doc_headings.py` (docs only; this plan is under
  `.structured-coding/`, which the script does not read — run to show nothing else moved): "176
  numbered sections across 25 documents, none duplicated"; `check_decision_ids.py`: "51 decision ids, all
  distinct". Headings §18.0–§18.9 are unique within this file.
- [x] Review: every finding cites a file and line or a command; every acceptance line names its pass
  condition before anything is run; every guard has a mutation; the non-goals match §13's 16a row and
  the shared-module ruling. Self-review by the drafting session; the primary session's freeze pending.

### 16a-C1 — The baseline on Godot Physics, and the two tools

**Goal.** "Before" is measured, on the base, by the same commands "after" will use (`ARC-23`).
**Scope.** New `clients/3d-spike/tools/physics_engine.gd`, `tools/frame_diff.gd`; this section's ledger.
No change to any script the scenes run. **Depends on** freeze.

- [ ] Implementation: the two tools (D-16a-8).
- [ ] Validation (E16a-1, all on the base's scenes, unmodified):
  - [ ] J-0 before: `godot --headless --path clients/3d-spike --script res://tools/physics_engine.gd`
  - [ ] slice: `--drive`, `--measure`, `--threshold`, `--character`, `--perf`, `--shots` (1600×900)
  - [ ] promenade: `--headtrace` (reference body), `--sweep`, `--frametime`, `./mineworld-3d --drive`
  - [ ] connected: `--world --link`, `--world --conversation`
  - [ ] the leak: which modes print it, and its exact line (L-1's starting fact)
  - [ ] `frame_diff.gd` on a frame against itself → 0; against a deliberately shifted copy → non-zero
    (the instrument is shown to see before it is trusted)
- [ ] Review: each baseline matches the accepted figures in `HUMAN_REVIEW_QUEUE.md` and
  `CHARACTER_ROUTE_D_PLUS.md` §§8.8–8.9, or the difference is recorded before anything changes.

**Failure cases.** A baseline that fails its own check on the untouched base is a pre-existing defect:
recorded, not fixed here, and reported (it changes what J-* can claim).
**Commit boundary.** Tools and ledger only.

### 16a-C2 — Jolt (DEP-20)

**Goal.** J-0 … J-5. **Scope.** `clients/3d-spike/project.godot` (`[physics]`); `docs/DECISIONS.md`
**DEP-20** (step-11 §15.2's draft, with §8.1's table as its options, the 2026-10-08 facts, the isolating
interface — the project setting, the only place that names Jolt, §9.4 — and the revisit trigger: a
J-check that cannot be fixed in the client, QS14-12); `slice_probe.gd`'s `--drive` prints the running
engine and fails unless it is Jolt (one line in its report, J-1). **Non-goals:** any other script.

- [ ] Implementation: the setting; DEP-20; the engine line in `--drive`.
- [ ] Validation (E16a-2): J-0 after; every command of C1 again; J-3 and J-4 computed against C1;
  sweeps compared sheet by sheet, one image at a time; M-1.
- [ ] Review: every moved number explained or listed; `check_decision_ids` (DEP-20 distinct).

**Failure cases.** A J-check that fails on Jolt: diagnose; a bounded client-side fix inside the slice's
own code (not a Jolt setting tuned to hide it) is made and recorded; otherwise **stop** and report both
measurements (QS14-12, the brief's material stop) — never fall back to Godot Physics silently.
**Commit boundary.** Setting, decision record, engine line, ledger.

### 16a-C3 — The connected probe modes move to `slice_probe_world.gd`

**Goal.** P-1, so that C4's new mode and 12e's go into a file under the size warnings (R-S14-7).
**Scope.** New `scripts/slice/slice_probe_world.gd` (`SliceProbeWorld extends SliceProbe`: `_link_check`,
`_conversation_frames`, `_street_watch`, `_talk_to`, `_walk_to`, `_answered`, moved verbatim);
`slice_probe.gd` (the block removed; `_process` calls `_run(_mode)`; `scripted()`/`with_hud()` ask the
sibling for its flags); `slice_main.gd` (which probe to instantiate). **Non-goals:** any behaviour change.

- [ ] Implementation: as scoped (D-16a-5).
- [ ] Validation (E16a-3): `--world --link` and `--world --conversation` print the same check lines as
  C2's runs; `--drive` (headless) unchanged; line counts of both files recorded.
- [ ] Review: the moved block diffed against its original (`git diff --color-moved`): moved, not edited.

**Commit boundary.** A pure move plus the two dispatch edits.

### 16a-C4 — Ray targeting, pick colliders, `intents.gd`, `--world --target`

**Goal.** T-1 … T-4; J-5 again on the ray. **Scope.** `scripts/build.gd` (`LAYER_BODIES`);
`scripts/player.gd` (`CAPSULE_RADIUS`, `CAPSULE_HEIGHT` named and used by its own capsule; no other
change); new `scripts/slice/targeting.gd` (D-16a-2/3); new `scripts/slice/intents.gd` (D-16a-4);
`scripts/slice/slice_link.gd` (`_figure` adds the pick collider; `facing_person()` → `SliceTargeting.aim`;
`talk_to_facing` through intents; `_on_resolved`/`_on_refused` ask intents; the five `"talk"` literals
gone; the module comment updated: it names `move` only); the reticle and "looking at" line if Q-16a-1 is
(a) (`slice_main.gd`/`controls_hud.gd`, connected only); `slice_probe_world.gd` (`target` mode; `_talk_to`
aims the camera at the figure's head, as a player would, instead of trusting the cone); `slice_probe.gd`
(the two measurement rays get `LAYER_WORLD`, A16-4); `mineworld-slice` (`--target`, headless).
**Non-goals:** masking people; any other action; offer prompts.

- [ ] Implementation: as scoped.
- [ ] Validation (E16a-4): T-1 … T-4; `--world --link` and `--world --conversation` again (the door talk
  still goes to Alice, A16-7; the counter talk still accepted with the reply); `--drive`, `--character`
  (standalone unaffected); M-2, M-3, M-4.
- [ ] Review: no code path decides validity (the request is sent whatever `may` says); the ray mask names
  no layer that does not exist; `slice_link.gd` names no action but `move`; the reticle is absent
  standalone.

**Failure cases.** T-1's door ray meeting Bob (R-16a-2): reported with the measured clearance, not
hidden by aiming elsewhere; the probe then aims where a player would see Alice, and the case is raised
in the ledger. **Commit boundary.** Targeting and intents, with their probe mode.

### 16a-C5 — The no-rule scan (I-S14-1.1, I-S14-6)

**Goal.** S-1. **Scope.** New `tests/acceptance/tests/client_rules.rs` (std only, no dependency; the
lexer, the three rules, the allow-lists with reasons, fail-closed roots); `tests/acceptance/src/lib.rs`
(the crate's list gains `client_rules`). **Non-goals:** S12's files (it adds its allow-list lines).

- [ ] Implementation: as scoped (D-16a-6).
- [ ] Validation (E16a-5): `cargo test -p mineworld-acceptance --test client_rules`; clippy `-D warnings`
  on the crate; M-5 … M-10 (M-10 a kept unit test of the lexer).
- [ ] Review: the allow-list has exactly the entries A16-9 justifies, each with its reason; the scan reads
  the working tree (so a plant is seen); nothing is skipped silently.

**Commit boundary.** The test file and the crate's doc list.

### 16a-C6 — The texture RID leak

**Goal.** L-1. **Scope.** Found by bisection over A16-12's caches (release each in `SliceMain._exit_tree`,
count the leaked RIDs per release), then a minimal scene if none is ours. If ours: each cache's owner gains
a `release()` static, called from `SliceMain._exit_tree` beside the existing `Props._scenes` release (the
promenade, `main.gd`, is left as it is, F-S14-1, and its exit is reported). If not ours: no code change.

- [ ] Implementation: as the evidence decides.
- [ ] Validation (E16a-6): the plain launch and every slice mode's exit lines; M-11 if a fix exists.
- [ ] Review: a release never runs while the scene still draws (exit only); nothing freed twice.

**Commit boundary.** The fix (or nothing) and the ledger.

### 16a-C7 — Status, documents, gates, close

**Scope.** `docs/MVP_STATUS.md`: the 3D column of Movement, Place, Conversation and Names, the 3D client
artefact row and the S14 stage row (§17.4, made true on the head); `docs/HUMAN_REVIEW_QUEUE.md`
limitation 8 (Q-16a-6); this section's ledger and closeout.

- [ ] Implementation: as scoped.
- [ ] Validation (E16a-7, on the final executable head): the slice's checks once more (`--drive`,
  `--measure`, `--threshold`, `--character`, `--world --link`, `--world --conversation`,
  `--world --target`) — I-S14-7; G-1, including the path check of the diff.
- [ ] Review: every acceptance line has its evidence; deviations recorded; the PR body carries the
  operator's short runnable list (overall memory: milestone handoff), including what the reticle looks
  like and how to aim.

Then push, open the PR **READY FOR OPERATOR REVIEW**, and stop. Do not merge.

### 16a test ownership

```text
STATIC      cargo fmt / clippy -D warnings on tests/acceptance; Godot's parser on every launch (a
            script error fails the mode)
UNIT        the GDScript lexer of client_rules.rs (M-10), kept
SCAN        client_rules.rs: action literals, rule constants, pack paths (S-1)
REAL RUN    (the project's Gate 2) every J-, T- and P- check: the real client, on the real engine,
            against the real server for J-5/T-*; the operator's play is the final judge of the
            reticle and the feel
GATE 1      NOT REQUIRED — no language model in 16a
CI          N/A — no workflow exists (A16-13); the full local gate runs once on the final head
```

### 16a risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-16a-1** | Jolt changes a measured number of an accepted check (step-11 §3.2: position-only stabilization, kinematic contacts) | J-1 … J-5 before/after; a bounded client fix or a material stop, never a silent fallback |
| **R-16a-2** | From the door, Bob occludes Alice for the ray (A16-7: 140 mm clearance at eye height) | measured in C4; reported as it is |
| **R-16a-3** | Another session holds port 7979; a run joins someone else's world | port checked before each connected run; a free port otherwise (A16-14); a run on a foreign server is INCONCLUSIVE, never counted |
| **R-16a-4** | A windowed capture stalls while another window covers it (recorded twice before) | one Godot window at a time; `--always-on-top` where the launcher offers it; background runs watched; a stall is INCONCLUSIVE and re-run once |
| **R-16a-5** | The sweep's frames are time-driven, so physics differences can shift animation phase and make pixel diffs meaningless | the sweep is judged by inspection against the operator's defects, the headtrace by its numeric bound; pixel diffs are reported for it only as information |

## 18.7 Ledger (live)

*(empty until implementation; E16a-1 … E16a-7 recorded here as each check runs: command, head and
working-tree fingerprint, wall time, decisive lines, PASS / FAIL / INCONCLUSIVE)*

## 18.8 Execution contract (proposed; confirmed at freeze)

```text
PROJECT / PR        MVP-0 · S14 / PR 16a — the 3D client, ready for bodies
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-15-demo-3d.md §18 (this section)
RELATED / BINDING   overall.md §3 (S14, S15), "Parallel build-out, 2026-10-08" (rulings 4, 6, 7, 9, 10);
                    this file §§1–17 as frozen at step level; step-11-bodies.md §§3.2, 15.2;
                    ENGINEERING_RULES §§3–12, 19; VISUAL_SLICE; HUMAN_REVIEW_QUEUE (VIS-3D-GODOT-1/-2);
                    docs/references/CHARACTER_ROUTE_D_PLUS.md §§8.7–8.9; REUSE_POLICY
IMPLEMENTATION BASE main @ 47c81d1; branch mvp0/pr-16a-jolt-targeting; worktree
                    /Users/yuema137/mineworld-worktrees/impl-s14-16a (this session only)
APPROVED SCOPE      §18.1
FROZEN INVARIANTS   I-S14-1 (16a's part: .1 and the talk half of .2's behaviour), I-S14-2, I-S14-3,
                    I-S14-6, I-S14-7, I-S14-10; no edit to clients/protocol/**; no regression of
                    VIS-3D-GODOT-1 or -2
SEQUENCE            C0 → (freeze) → C1 → C2 → C3 → C4 → C5 → C6 → C7, each committed and pushed when
                    coherent
VALIDATION BUDGET   static/unit/scan: unrestricted; real client runs: each ≤ ~10 min, background when
                    > 2 min, one Godot window at a time, total ≤ ~3 h of wall time including retries;
                    full cargo test: once, on the final head; real-model: NOT REQUIRED
LIVE DOCUMENTATION  this section (§18.6 checkboxes, §18.7 ledger)
HANDOFF             §18.9 of this section (Q-16a-4), refreshed at each commit
ENDPOINT AUTHORITY
  implementation + local validation   after the freeze message only — source: the brief ("Do not write
                                      code until I freeze it")
  semantic commits                    authorized — source: the brief (phase 2 "Implement it")
  branch push                         authorized — source: the brief ("push it"); D-12
  PR creation / update                authorized — source: the brief ("open a PR marked READY FOR
                                      OPERATOR REVIEW")
  CI repair                           N/A — no workflow exists (A16-13)
  merge                               explicit operator authorization only — source: the brief ("Do not
                                      merge it")
POST-MERGE SYNC     the primary session owns step §13 and overall; this session owns §18 (merge identity,
                    evidence, deviations, remaining issues)
NORMAL STOP         PR 16a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a regression of an accepted visual or a confirmed J-3 rise; any server, kernel,
                    contract or shared-module change; Jolt unavailable or unfit (reported, no silent
                    fallback); a change to §18.1's scope or to a frozen invariant
```

## 18.9 Handoff (live; replaces a separate file for this lane, Q-16a-4)

```text
checkpoint     C0 written, awaiting the primary session's freeze
next action    on freeze: C1 (tools, then the baseline runs in the order of §18.6 C1)
background     none
```

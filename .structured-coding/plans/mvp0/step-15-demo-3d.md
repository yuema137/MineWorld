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
**PR 16a:** detailed to the commit in §19, `DESIGN FROZEN (2026-10-08), primary session`.
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

# 18. PR 16b — the shared module: affordance readers, raw components, revision (full design)

## DESIGN FROZEN

```text
Design revision     §18 as committed in 5682784 (§§18.1–18.8, 18.11 frozen; §§18.5 checkboxes,
                    18.9, 18.10 live)
Approved by         the primary session, 2026-10-08 — freeze message: "FREEZE: 16b is DESIGN FROZEN
                    (2026-10-08), primary session. §18 is accepted as written, and QSB-1 through QSB-7
                    are accepted as recommended", plus: demo.gd stays untouched; checks go in
                    clients/protocol/checks/; the primary session tells S11-A and S12 the final names
Implementation base main @ 47c81d1, branch mvp0/pr-16b-shared-module
Execution contract  §18.11
Lifecycle           FROZEN — DESIGN FROZEN (2026-10-08), primary session
                    → READY FOR OPERATOR REVIEW (2026-10-08). Final executable head 5c10e06; the PR
                    head is the ledger commit after it (Markdown only). Implementation context CLOSED /
                    AWAITING OPERATOR ACTION. DO NOT MERGE without the operator; lands before S11-A.
```

**Post-merge synchronization (pending, owners per §18.11):** this session records the merge identity
in this section; the primary session updates the step header, §§12–13 (16b delivered; the `bodies-yard`
kick check now 12e's, QSB-4), `step-13-client-2d.md` §7 (S12's M-1 … M-3 delivered under SB-1 … SB-7's
names, and D-1/D-2), `overall.md` §7, and tells S11-A to rebase.

The first commit of the PR is this section, Markdown only (coordination ruling 9).
**Coordination:** `overall.md` "Parallel build-out, 2026-10-08", ruling 4 — this is **the** shared-module
PR. It unifies S12's M-1 … M-3 (`step-13-client-2d.md` §7) with this step's M-1 … M-3 (§12), is owned by
16b, and lands **before S11-A**, which rebases over it. No other PR edits `clients/protocol/mineworld/`
except S11-A (join credentials) and S11-C (`events`, `perceived`, `acted_through` readers).

## 18.1 Identity, base, approved scope

```text
PR            16b — shared module: affordance readers, raw components, revision (S14, run in parallel
              with 16a; PR number assigned at freeze, ruling 7)
base          main @ 47c81d1 (the parallel-steps freeze merge, #62)
branch        mvp0/pr-16b-shared-module, worktree /Users/yuema137/mineworld-worktrees/impl-shared-module,
              held by the implementing session only
audit         §18.2 (files and symbols read on 47c81d1)
scope         S12 M-1, M-2, M-3 and M-7 (as far as it documents those three); S14 M-1, M-2, M-3 and M-7
              (as far as it documents those three); unified in §18.3
depends on    nothing unmerged
```

**Goal.** Both reference clients can list every affordance the server sent — including several complete
affordances that share an action type and a target — find the affordances that concern one entity,
submit a complete affordance exactly as offered, read a disclosed component whatever its JSON type, and
know which persisted revision an observation describes. The module stays a reader and a sender: it
decides nothing (`ADOPTION.md` §3.3), and names no 2D or 3D engine type (I-S14-10).

**The change set.** Every path this PR may touch:

```text
clients/protocol/mineworld/observation.gd        readers (B-C2)
clients/protocol/mineworld/world_client.gd       submit_affordance, submit's payload type, revision (B-C3)
clients/protocol/checks/**                       new: the reader check and the live check (B-C2, B-C3)
clients/protocol/run.sh                          one new mode, `affordances`; existing modes unchanged (B-C3)
clients/protocol/evidence/affordances-*.log      new: the live check's transcript and server log (B-C3)
clients/protocol/evidence/README.md              the new evidence files listed (B-C3)
clients/protocol/ADOPTION.md                     §2, §3.3 (B-C1)
clients/protocol/README.md                       one line for `checks/` and the new mode (B-C1)
docs/DECISIONS.md                                a dated note on ARC-34 (B-C1); no new record
.structured-coding/plans/mvp0/step-15-demo-3d.md this section (live ledger)
.structured-coding/plans/mvp0/handoff-16b.md     this PR's handoff (QSB-3)
```

**Non-goals.** No change to `demo/demo.gd`, `space.gd`, any file under `clients/3d-spike/`, any committed
file under `clients/protocol/evidence/` other than the new ones, `server/PROTOCOL.md` (the wire is
unchanged), any Rust crate, any world or pack. No join credentials (S11-A, M-5), no `events` reader
(S11-C, M-6), no `acted_through` (S11-C, M-4 of §12), no reconnect guidance (S12's M-4, which stays with
13a — QSB-6). No client consumes the new API in this PR: 13b (2D) and 12e/16d (3D) do.

## 18.2 Audit (main @ 47c81d1)

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-16b-1** | `affordance(action_type, target := "")` returns the **first** match; `may`, `unavailable_reason` and `requirement` all go through it. `affordances()` returns the whole list and takes no argument. `offered_against(target)` lists action types against one target. | `observation.gd:146–202` | A filter on the existing list accessor is the smallest addition; `affordance()` and its three users keep their first-match meaning for incomplete affordances, where type and target identify one entry. |
| **F-16b-2** | `component()` returns the payload only when it is a Dictionary, else `{}`. Every payload disclosed in today's worlds is a JSON object (`Holdings` is `{ held: [...] }`, `Listing` `{ operator, listed }`, `PlaceShape` `{ floor, solids }`), so no array payload exists before 12c's `loose-objects`. | `observation.gd:102–111`; `systems/inventory/src/component.rs:36`; `systems/economy/src/component.rs:135`; `systems/bodies/src/component.rs` | `component_value` cannot be exercised on an array from a live server today; the reader check (B-C2) owns that shape with a synthetic frame, labelled as such. |
| **F-16b-3** | `submit(action_type, target, payload: Dictionary, actor_location)` builds the request and emits `submitted_request`. A complete affordance's `payload` is whatever JSON the owning system encoded; a payload-less Rust struct (`struct Shove;`) serializes as `null`, which a `Dictionary` parameter cannot carry. | `world_client.gd:155–175`; `PROTOCOL.md` §5 ("in the same JSON shape a client would send") | `submit`'s `payload` widens to `Variant` (S12's M-2); every existing caller passes a Dictionary and is source-compatible. |
| **F-16b-4** | `is_complete` must test **presence** of the key, not non-null: `PROTOCOL.md` §5 says the field is *absent* on an incomplete affordance (`skip_serializing_if`), so a present `null` is a complete affordance whose payload is `null`. | `PROTOCOL.md:149–157`; ARC-34 point 1 | Fixed in the reader and checked by a synthetic frame. |
| **F-16b-5** | `_observation` keeps `seq` and the inner observation and drops the frame's `revision`; `world["revision"]` (the welcome's) is the only revision a client can read. | `world_client.gd:256–288`; `PROTOCOL.md:115–142` | `MineWorldClient.revision` (S12's M-3), set from the welcome and every accepted observation frame. |
| **F-16b-6** | Callers of the module outside it: `demo/demo.gd` (`submit` ×2, `may`, `unavailable_reason`, `component`, `affordances`, `display_name`, `own_component`, `tagged`, …); `clients/3d-spike/scripts/slice/slice_link.gd:100, 420, 440–444` (`component`, `submit`, `may`, `unavailable_reason`); `slice_probe.gd:575` (`component`). The 3D project links the module by symlink (`clients/3d-spike/mineworld -> ../protocol/mineworld`). | `grep` over `clients/`; `ls -la clients/3d-spike/mineworld` | "Existing callers unchanged" is shown by running them: `run.sh evidence` (demo, both flavours) and the slice's `--drive` and `--world --link` (§18.4 A-5, A-6). |
| **F-16b-7** | Market Town at genesis gives the live check every case it needs **without 12c**: `visitor` stands at the café door (1 610, 600) holding `apple` and `scarf`, with a wallet of 200 000; the café is a shop (`cafe-company`, six priced kinds), so six complete `buy` affordances, target `null`, are offered there; `give` is offered complete per kind held per present person (SD-20), so `visitor` sees **two** complete `give` affordances against each person present — one action type, one target, two choices; Alice stands at (6 000, 8 000), ~8.6 m away, beyond `give`'s 3 000 mm, so her two `give` affordances are complete **and unavailable**; `talk` is offered incomplete (F-S14-20). | `worlds/market-town/people/{visitor,alice}.yaml`; `organizations/cafe-company.yaml`; step-10 SD-20; `systems/economy/src/action.rs:45` | A-2 … A-4 run against a real server with no fixture. |
| **F-16b-8** | 12c (kick, throw, shove, `loose-objects`) is **not merged** on 47c81d1: no pack declares `kick`; `bodies-yard` installs `presence, movement, conversation, bodies` only. §13's 16b checkpoint names `bodies-yard` "once 12c merges". | `grep -rn kick systems/*/src` (none); `worlds/bodies-yard/world.yaml` | The `bodies-yard` half of §13's checkpoint cannot run before 16b lands (it must precede S11-A). `affordances_about(object)` is proven here on `buy`'s `payload.item`, the same rule; the `kick` case moves to 12e's `--world --bodies`, which consumes it (bounded scoping, recorded; not a change to the result). |
| **F-16b-9** | `run.sh`'s `start_server` hard-codes `worlds/social-cafe` in both the launch and the `pkill` pattern; the server takes `--save DIR` (`tools/cli/src/main.rs:97–100`). | `run.sh:29–36` | The new mode passes its world and a save directory; the three existing modes call it exactly as before. |
| **F-16b-10** | The two "naming" questions: S12 named its list accessor `choices`, S14 `complete_affordances`, and S14's lookup `offers_about`. `CORE_CONCEPTS.md` §15.2 defines **Affordance** and **complete affordance**; "offer" is presence's server-side verb (`Offer::new`, `Offer::complete`), and "choice" is not a defined term. | `docs/CORE_CONCEPTS.md:764–801`; `CLAUDE.md` §2.1 rule 3 | Client API names use the defined terms (SB-1, SB-3). |
| **F-16b-11** | No CI workflow exists (`.github/workflows` absent; S13 builds it). Nothing in Rust reads the module; `tools/cli/tests/ac13_semantic_parity.rs` reads `evidence/request-{2d,3d}.json`. | `ls .github`; `clients/protocol/evidence/README.md` | CI repair is N/A; A-5 re-runs the parity test. |

## 18.3 The unified API (SB-1 … SB-7)

Every addition is a reader of a frame the server already sends, or a convenience over `submit`. None
compares a position, reads `available` to decide anything, or knows an action type.

```text
MineWorldObservation
  affordances(action_type := "", target: Variant = null) -> Array     widened; no arguments = today's
  complete_affordances(action_type := "", target: Variant = null) -> Array               new
  affordances_about(id: String) -> Array                                                 new
  static is_complete(affordance: Dictionary) -> bool                                     new
  component_value(id: String, component_type: String) -> Variant                         new
  component(id, component_type) -> Dictionary                                unchanged meaning

MineWorldClient
  submit(action_type, target, payload: Variant = {}, actor_location)        payload widened
  submit_affordance(affordance: Dictionary, actor_location: Variant = null) -> String    new
  revision: Variant                                                                      new
```

| ID | Decision | Unifies | Rationale |
| --- | --- | --- | --- |
| **SB-1** | **`affordances(action_type := "", target: Variant = null) -> Array`** — every affordance, complete or not, in the server's order; `action_type` `""` matches any type; `target` `null` matches any target, a String matches exactly, and `""` matches the target-less (as every existing method spells it). Called with no argument it is today's `affordances()`. | S12 `choices(type, target)`; S14 `complete_affordances` (its list half) | One list accessor instead of two names for one list; "choice" would be a synonym for Affordance (`CLAUDE.md` §2.1 rule 3). The only order is the server's: `PROTOCOL.md` §5 says several complete affordances "differ only in `payload`; a client keeps them apart by their position in the list". |
| **SB-2** | **`complete_affordances(action_type := "", target: Variant = null) -> Array`** — the subset of SB-1 that is complete; **`static is_complete(affordance) -> bool`** — `affordance.has("payload")`. | S14 M-1; S12 `is_complete` | The defined term, ARC-34's. Presence of the key, not non-null (F-16b-4). A 3D buy menu and a 2D "complete" menu row are each one call. |
| **SB-3** | **`affordances_about(id: String) -> Array`** — every affordance whose `target` is `id`, or whose `payload` is a Dictionary with a **top-level** String value equal to `id`; in list order, complete or not. | S14 `offers_about` | Finds `kick { object }`, `throw { object, toward }` and `buy { item }` by the entity they concern without the module knowing those actions; finds `talk`/`give`/`shove` by target. Identities are decimal strings (§3.1), so a top-level string equal to one is a reference. Nested values are not searched: `toward: { x, y }` holds numbers, and a deeper search would start guessing at a pack's shapes. Named with the defined term (F-16b-10). Accepted limitation: a free-text payload string that happened to equal an id would match; complete payloads are enumerated choices, never free text (ARC-34 point 6). |
| **SB-4** | **`component_value(id, component_type) -> Variant`** — the disclosed payload exactly as it arrived, any JSON type; `null` when that component was not disclosed about that entity. `component()` keeps its Dictionary contract and is re-expressed through it. | S14 M-3 | F-16b-2. One lookup loop, two typings. |
| **SB-5** | **`submit_affordance(affordance, actor_location := null) -> String`** — submits `action_type`, `target` and `payload` exactly as offered, through `submit` (so `submitted_request` and the token behave identically); returns `""`, warns, and sends **nothing** when the affordance is not complete or carries no String `action_type`. It **never reads `available`**: an unavailable complete affordance is submitted, and the server answers. | S12 M-2 = S14 M-2 | ARC-34 point 3, `ADOPTION.md` §3.3 "NOT allowed: refuse to submit because you concluded it would fail". Refusing an incomplete affordance is not a world rule: there is no payload to send unchanged, and inventing one is the client knowing the action. |
| **SB-6** | **`submit`'s `payload` becomes `Variant`** (default `{}`). | S12 M-2 | F-16b-3. Source-compatible: every caller passes a Dictionary. |
| **SB-7** | **`MineWorldClient.revision: Variant`** — `null` or an `int`; reset to `null` by `connect_to_world`, set from the welcome's `world.revision`, then from each observation frame that is accepted (a stale frame changes neither `latest` nor `revision`). Documented as the persisted revision `latest` was computed from. Not on `MineWorldObservation`: the field is the frame's, not the contract `Observation`'s, and `latest` and `revision` change together. The module does not police monotonicity (the server guarantees it). | S12 M-3 | F-16b-5; `AC-15`'s fourth evidence line seen from inside a client. |

**Engine neutrality and policy freedom (I-S14-10).** The diff adds no type from Godot's 2D or 3D node
families (`Vector2/3`, `Node2D/3D`, `CollisionObject*`, …) to the module, no action-type literal, no
constant naming a distance, and no comparison of a position. A–7 checks it.

## 18.4 Acceptance (decided before measuring, `ARC-23`)

```text
A-1  READER. A headless check (clients/protocol/checks/reader_check.gd, no server) over synthetic
     frames — each shape one the contract allows and no installed pack sends yet — shows:
       a. affordances("x") returns both of two affordances of type x against one target, in order,
          and affordance("x", t) still returns the first;
       b. target null = any, "" = target-less, an id = exactly that target;
       c. a complete affordance whose payload is null is complete; one without the key is not;
       d. component_value returns an Array payload as an Array, a Dictionary as itself, null when
          undisclosed; component() returns {} for the Array, as today;
       e. affordances_about(id) finds an affordance by target and by a top-level payload string, and
          not by a nested one.
     Expected values are written in the check by hand, never computed by the code under test.
A-2  LIVE: LISTING. Against `mineworld server worlds/market-town --agent alice --save <tmp>`, the live
     check (clients/protocol/checks/affordances_check.gd, run by `run.sh affordances`) seated as
     `visitor` reads its first observation and prints: six complete `buy` affordances (one per kind
     cafe-company prices), target null; for Alice, exactly two complete `give` affordances, both
     returned by affordances("give", alice) in the server's order, with payload items differing;
     `talk` against Alice incomplete; affordances_about(<a buy's payload.item>) contains that `buy`.
A-3  LIVE: SUBMITTED UNCHANGED. submit_affordance(one available `buy`) produces a submitted_request
     whose action_type, target and payload.payload equal the affordance's, byte for byte after
     JSON.stringify; the server answers `accepted`; a later observation shows the observer's
     `wallet` lower by that kind's price and its `holdings` one higher, read with component_value.
A-4  LIVE: POLICY-FREE. submit_affordance(Alice's first `give`, available false) IS sent and answered
     rejected `too_far_away` by the server; submit_affordance(the incomplete `talk`) returns "" and no
     submitted_request is emitted; revision is an int, never decreases across the run, and is
     higher after the accepted buy than before it.
A-5  EXISTING CALLERS, 2D side. `bash clients/protocol/run.sh evidence` on the PR head: every
     transcript shows the same steps as the base run (walk, talk accepted, Alice's reply heard, the
     wanderer told, the simultaneous run with one instance); evidence/request-2d.json and
     request-3d.json are byte-identical to the committed files (git diff --exit-code); `cargo test
     -p mineworld-cli --test ac13_semantic_parity` passes. The base run (E-B0) is made first, on
     47c81d1, to show the requests reproduce byte for byte before any change.
A-6  EXISTING CALLERS, 3D side (the symlink). `./mineworld-slice --drive` and `./mineworld-slice
     --world --link` on the PR head report the same verdicts as on the base (E-B0), with no
     GDScript parse or runtime error in either log.
A-7  NO RULE IN THE MODULE. In the PR's diff of clients/protocol/mineworld/: no string literal equal
     to an action type any pack declares (collected from ActionTypeId::from_static under
     systems/*/src), no `distance`/`length`/comparison of a position, no read of `available` outside
     the pre-existing may(), no Godot 2D/3D node or vector type. Shown by a recorded grep over the
     diff and by review; 16a's I-S14-1.1 scan (tests/acceptance) becomes the durable owner.
A-8  MUTATIONS BITE (each made in the working tree, run, recorded, reverted):
       M-B1  affordances() filter returns only the first match       → A-1a and A-2 fail
       M-B2  submit_affordance skips when available is false          → A-4 fails (no too_far_away)
       M-B3  submit_affordance sends an incomplete affordance          → A-4 fails (talk sent)
       M-B4  is_complete tests payload != null                         → A-1c fails
       M-B5  component_value coerces to Dictionary                     → A-1d fails
       M-B6  revision set only from the welcome                        → A-4 fails (no rise)
       M-B7  affordances_about ignores the payload                     → A-1e and A-2 fail
```

## 18.5 Commit plan

Each commit tracks implementation, validation and review separately. Evidence goes into §18.9 as
`E-B<n>`. A planned commit may become several coherent commits; the mapping is recorded.

### B-C0 — Design (this section) — docs only

- [x] Implementation: §18, from the audit in §18.2.
- [x] Validation: `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents,
  none duplicated; `python3 scripts/check_decision_ids.py` → 51 ids, all distinct (2026-10-08).
- [x] Review: every finding cites a file, a line, or a command; the two step lists are unified with
  each item placed (SB-1 … SB-7, non-goals); the scope-reducing finding F-16b-8 is stated, not hidden;
  questions are marked for the freeze (§18.8). Self-review by the drafting session; the primary
  session's review is pending.

### B-C1 — Specs before code: `ADOPTION.md`, protocol `README.md`, the ARC-34 note

**Goal.** The module's specification states the new API and its obligations before code relies on it
(`CLAUDE.md` §2.2).

**Scope.**
- `clients/protocol/ADOPTION.md`:
  - §2 `MineWorldClient`: `submit`'s payload as Variant; `submit_affordance`; the `revision` member.
  - §2 `MineWorldObservation`: the list of SB-1 … SB-4.
  - §2 "Complete affordances" paragraph rewritten: list with `affordances`/`complete_affordances`,
    find with `affordances_about`, submit with `submit_affordance`, whether available or not; keep
    them apart by position; an affordance a client cannot compose (incomplete, and no composer for
    its type) is shown, never guessed at (S12's M-7). The sentence "The module gains no accessor for
    this in S9; the first client use is S12's" is replaced.
  - §2 a `component_value` sentence: a payload that is not an object (12c's listings) is read here.
  - §3.3 `allowed`: "submit a complete affordance unchanged, available or not".
- `clients/protocol/README.md`: `checks/` in the layout block; the `affordances` mode in "Run it".
- `docs/DECISIONS.md`: under ARC-34, a dated note — the module's readers and `submit_affordance`
  landed in 16b; point 7's "no client code in S9" stands as history.

**Depends on:** freeze. **Non-goals:** no code; `PROTOCOL.md` unchanged (the wire is unchanged).

- [x] Implementation: the edits above. ADOPTION §2 gained the API lines, three paragraphs ("Listing
  affordances", "Complete affordances" rewritten, "Disclosed components") and the two-kinds paragraph
  (S12's M-7); §3.3 one `allowed` line. README: `checks/` and a "module's own checks" block. ARC-34:
  "Note, 2026-10-08 (S14, PR 16b)".
- [x] Validation: `check_doc_headings` → 176 sections, none duplicated; `check_decision_ids` → 51 ids,
  all distinct (no new id). The §2 ↔ `func` cross-check is B-C4's.
- [x] Review: no new term; "affordance" and "complete affordance" as `CORE_CONCEPTS.md` §15.2 defines
  them; §3.3's four rules unchanged in substance; nothing in ADOPTION contradicts `PROTOCOL.md` §§5–6.
  Done: the only terms used are Affordance, complete affordance, payload, target; "choice" appears
    only in ARC-34/PROTOCOL's own sense ("one per choice offered"); §3.3 gains one allowed line and
    loses nothing; PROTOCOL §5's "keep them apart by their position" and §6's "submit unchanged" are
    restated, not altered.

### B-C2 — `MineWorldObservation`: SB-1 … SB-4, and the reader check

**Goal.** A-1.

**Scope.**
- `clients/protocol/mineworld/observation.gd`:
  - `affordances(action_type := "", target: Variant = null)` filtering in list order; a private
    `_target_of(affordance) -> String` (`null` → `""`), reused by `affordance()` and `offered_against()`
    in place of their two copies of that conversion.
  - `complete_affordances(...)`, `static is_complete(...)`, `affordances_about(id)`.
  - `component_value(id, component_type) -> Variant`; `component()` re-expressed as
    `component_value` typed to Dictionary.
  - Doc comments on each, in the file's voice: reads, never decides.
- `clients/protocol/checks/reader_check.gd` (new): `extends SceneTree`, run as
  `godot --headless --path clients/protocol --script res://checks/reader_check.gd`; builds the
  synthetic frames of A-1 inline, prints one `PASS`/`FAIL` line per claim with the observed value,
  and quits with exit code 1 on any failure.

**Depends on:** B-C1. **Non-goals:** `affordance`, `may`, `unavailable_reason`, `requirement`,
`offered_against` keep their meaning.

- [x] Implementation: as scoped. `affordance()` is now `affordances(type, target)[0]` and
  `offered_against()` is `affordances("", target)` mapped to types, so the target conversion exists once
  (`_target_of`); `_payload_names` is the top-level search. `component_value` also skips a component
  record that is not an object (the old loop would have errored on one).
- [x] Validation: E-B1 (reader check 17/17 PASS); E-B2 (M-B1, M-B4, M-B5, M-B7 each FAIL naming their
  claims, reverted, 0 `MUTATION` markers left); `--drive` on this tree: E-B3.
- [x] Review: `affordance(type, target)` returns the same first entry as before for every input (same
  filter, same order, `{}` when none); `affordances()` with no argument returns the frame's own array as
  before; `""` still means target-less in `affordance`, `may`, `unavailable_reason`, `requirement`,
  `offered_against`; the added code compares no position, reads no `available`, holds no action-type
  literal, and names no engine type (A-7, checked again at B-C4).

### B-C3 — `MineWorldClient`: SB-5 … SB-7, the live check, `run.sh affordances`

**Goal.** A-2, A-3, A-4.

**Scope.**
- `clients/protocol/mineworld/world_client.gd`: `submit`'s `payload: Variant = {}`;
  `submit_affordance(affordance, actor_location := null) -> String` through `submit`; `var revision:
  Variant = null`, reset in `connect_to_world`, set in `_welcome` from `world.revision` and in
  `_observation` after the stale check; doc comments, and the class comment's "it knows" block gains
  "the revision a frame names".
- `clients/protocol/checks/affordances_check.gd` (+ a `.tscn` if a scene proves simpler than a
  SceneTree script — chosen at implementation and recorded): seated as `visitor`, performs A-2 → A-3 →
  A-4 in that order, driven by observations and results (never by sleeps), prints a `[check]`
  transcript with one PASS/FAIL per claim, and quits non-zero on any FAIL or on a 60 s budget.
- `clients/protocol/run.sh`: `start_server` gains optional world and save-directory arguments,
  defaulting to today's values so the three existing modes run the identical command; mode
  `affordances` starts `worlds/market-town --agent alice --save "$(mktemp -d)"`, runs the check,
  stops the server, **removes the save directory** (ruling 10), and prints the `[check]` lines.
- `clients/protocol/evidence/affordances-market-town.log`, `server-affordances.log` (new), and
  `evidence/README.md`'s table.

**Depends on:** B-C2. **Non-goals:** no change to `demo.gd`, nor to the commands the existing modes run.

- [x] Implementation: as scoped, plus D-1 and D-2 (§18.10), found by the first live run. The live
  check is a `SceneTree` script (no `.tscn` needed); it prints every complete affordance it was offered,
  so the transcript shows the real payload shapes. Godot's `.uid` sidecars for the two new scripts are
  committed, as the module's own are.
- [x] Validation: E-B4 (live check 22 claims PASS on the final B-C3 tree); E-B5 (M-B1, M-B2, M-B3,
  M-B6, M-B7, M-B8 against it, each FAIL, reverted); the reader check re-run with D-1's typed-reference
  case, 18/18 PASS; no `world.sqlite` left in `$TMPDIR` after the runs (`find … -newer` → 0).
- [x] Review: `submit_affordance` never reads `available` (M-B2 shows the check would see it); it
  emits `submitted_request` only through `submit`, i.e. only when it sends; `revision` is assigned after
  the stale return, so a stale frame moves neither `latest` nor `revision`; the existing modes call
  `start_server "$log"`, which now runs `mineworld server "worlds/social-cafe" --listen … --agent
  alice` — the same argv as before (the quoted `worlds/$world` expands to the same word, the empty
  `save` array to nothing) and the same `pkill` pattern. `_as_sent` touches numbers only: identities
  are strings and pass through.

### B-C4 — Final gates on the PR head, ledger, PR

**Goal.** A-5 … A-8 on the exact head; the PR opened READY FOR OPERATOR REVIEW.

- [x] Implementation: §18.9 evidence and §18.10 deviations completed; handoff closed.
- [x] Validation, on the final executable head `5c10e06` (E-B6 … E-B9):
  - [x] `bash clients/protocol/run.sh evidence` → A-5 PASS; logs inspected, committed evidence restored;
  - [x] `cargo test -p mineworld-cli --test ac13_semantic_parity` → 2 passed;
  - [x] `./mineworld-slice --drive` and `./mineworld-slice --world --link` → A-6 PASS (the link run
    once INCONCLUSIVE from a port collision, re-run);
  - [x] reader check and `run.sh affordances`: last run on content identical to the head's code;
  - [x] the A-7 grep over `git diff 47c81d1 -- clients/protocol/mineworld` → clean;
  - [x] `check_doc_headings`, `check_decision_ids`.
- [x] Review: the whole diff against §18.1's path list — every changed path is on it
  (`git diff --stat 47c81d1`, E-B9); ADOPTION §2 against the code's `func` list (E-B8); every A-8
  mutation recorded with its failing line (E-B2, E-B5).

## 18.6 Test ownership

```text
STATIC      Godot's parser on import/run (a parse error in the symlinked module breaks both projects);
            the doc-heading and decision-id scripts
UNIT        reader_check.gd: JSON shapes the contract allows and no installed pack sends yet (array
            component payloads, a null complete payload, nested payload ids) — A-1
INTEGRATION affordances_check.gd against a real market-town server: listing, unchanged submission,
            policy freedom, revision — A-2 … A-4
REGRESSION  run.sh evidence + ac13_semantic_parity (demo, both flavours); the slice's --drive and
            --world --link (3D through the symlink) — A-5, A-6
GATE 1      NOT REQUIRED — no language model anywhere
GATE 2      the live check and the regression runs above are this PR's real-lifecycle evidence
CI          none configured (S13); N/A
```

The reader check owns only what the live run cannot produce today; everything a real server sends is
owned by the live check, not duplicated in fixtures.

## 18.7 Is any of this material?

No frozen invariant, contract, ownership boundary or dependency changes. The module's public API grows
additively; `submit`'s widening is source-compatible. Two points are surfaced for the freeze rather than
assumed: the renaming in SB-1/SB-3 (QSB-1) and the deferral of §13's `bodies-yard` half to 12e (QSB-4).
A needed edit to any path outside §18.1's list, or any change to the committed `request-*.json`, is a
material stop.

## 18.8 Questions for the freeze (QSB-1 …)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QSB-1** | The operator's list names `choices`, `complete_affordances` and `offers_about`. Use those names, or fold `choices` into `affordances(action_type, target)` and call the lookup `affordances_about`? | **Fold and rename** (SB-1, SB-3): `CLAUDE.md` §2.1 rule 3 forbids a synonym for a defined term, and "choice"/"offer" would be two. The behaviour is exactly the requested one. 13b and 12e consume these names; S12's step text keeps its working names until its own PR (the primary session's sync). |
| **QSB-2** | Keep `complete_affordances` beside `is_complete`, or only `is_complete`? | **Keep both**: the complete subset is the one list both menus show, and filtering by a static Callable is awkward in GDScript. |
| **QSB-3** | Handoff file. `handoff.md` is shared by every lane and currently holds 12b's text. | **`handoff-16b.md`** for this PR, so parallel lanes never overwrite each other's handoff. |
| **QSB-4** | §13's checkpoint also names `bodies-yard` `kick` "once 12c merges"; 12c is not merged and 16b must land before S11-A. | **Defer that half to 12e** (`--world --bodies` uses `affordances_about(object)` for kick); A-2 proves the same payload rule on `buy`. |
| **QSB-5** | `run.sh evidence` rewrites the committed AC-13 evidence. Commit the regenerated files? | **No.** Compare the two `request-*.json` byte for byte, record the transcripts' verdicts in §18.9, and restore the committed files: they are frozen evidence of the run that produced them (`evidence/README.md`). |
| **QSB-6** | Ruling 4 says no other PR edits the module. S12's M-4 (reconnect guidance) edits only `ADOPTION.md` §6, outside `mineworld/`. | **Leave it with 13a**; 16b does not touch §6. |
| **QSB-7** | Decision record. S14's range is ARC-50 … 52, DEP-20. | **No new record**: the API follows ARC-34 and is specified in `ADOPTION.md`; a dated ARC-34 note records the first client accessors. ARC-50 … 52 stay for 12e (the correction rule, `ARC-S14-a`) and later S14 PRs; DEP-20 is 16a's Jolt. |

## 18.9 Evidence ledger (E-B<n>)

E-B0 is the base run of A-5 and A-6 on 47c81d1 (module and demo untouched), made before any code edit.

**E-B0a — `bash clients/protocol/run.sh evidence` on the base (2026-10-08). PASS.** 58.6 s wall, exit 0.
All five transcripts end as committed: both flavours walk, `talk` to "7" accepted, Alice's reply heard
("…You are the first person to speak to me here."), `wrote 5 submitted request(s)`; the wanderer is
told what Vera said; the simultaneous pair shares one instance. `git diff` after the run:
`request-2d.json` and `request-3d.json` **unchanged** (the requests reproduce byte for byte, so A-5's
comparison is meaningful); seven `.log` files differ only in the world instance, and in
`simultaneous-3d.log` the client count (3 vs 2) and the ActionIds (+1) — the two simultaneous clients'
join order, a race the base already has. Logs kept in `/tmp/16b/base-evidence/`; the committed
evidence restored with `git checkout -- clients/protocol/evidence` (QSB-5).

**E-B0b — `./mineworld-slice --drive` on the base. PASS.** 77.3 s wall, exit 0, "all drive checks
pass" (walk-in, loop closes within 0.10 m, three wall pushes, exit to `street.main`, three cameras with
the body still, jumps 0.488/0.491 m, the Flower Room loop within 0.12 m). Verdict lines kept in
`/tmp/16b/e-b0-drive.verdicts` for comparison.

**E-B0c — `./mineworld-slice --world --link` on the base. PASS.** 111.4 s wall, exit 0, "all link
checks pass": out to the street and back, jog, two jumps, `talk` from the counter 1.99 m from Alice
accepted and her reply heard, five street doorways mapped, the 60 s street watch. Verdict lines kept in
`/tmp/16b/e-b0-link.verdicts`.

**E-B1 — reader check on the B-C2 tree. PASS.** `godot --headless --path clients/protocol --script
res://checks/reader_check.gd`: 17 claims, 17 PASS, exit 0 (a: both `hand` items `8001, 8002` in order,
`affordance()` still `8001`, 6 entries; b: null → all six, `""` → `nudge, point`, Bob → `shrug`; c:
`[false, true, true]`, the five complete; d: `[TYPE_ARRAY, 2]`, `{floor: 1}`, `null`, `{}`; e: Alice →
`wave, hand, hand`, box → `nudge`, coffee → `8001`, Bob → `shrug` only).

**E-B2 — mutations against the reader check.** Each made in `observation.gd`, run, and reverted by
restoring the saved file (`grep -c MUTATION` → 0 afterwards; the clean re-run PASS):

```text
M-B1  affordances() returns after the first match   FAIL a "returns both, in order" ["8001"];
                                                     FAIL b "\"\" matches the target-less" ["nudge"]
M-B4  is_complete = payload != null                  FAIL c "is_complete per entry" [false,true,false];
                                                     FAIL c "complete subset" (shrug missing)
M-B5  component_value coerces to Dictionary          FAIL d "array payload as an array" [27,-1]
M-B7  affordances_about ignores the payload          FAIL e "box" []; FAIL e "coffee" []
```

**E-B3 — `./mineworld-slice --drive` on the B-C2 tree. PASS.** 77.8 s, exit 0, "all drive checks
pass"; its 18 verdict lines are identical to E-B0b's (`diff` empty).

**E-B4 — `bash clients/protocol/run.sh affordances` on the B-C3 tree. PASS** (after D-1 and D-2; the
first run is E-B4a below). ~65 s including the build, exit 0, 22 claims PASS. Observer `17`, Alice
`7`, revision 1 at the first view. Offered complete: six `buy`, target null, payload
`{ item: { entity, entity_type: "item" } }` for items 22, 24, 25, 33, 36, 37; one `eat` (item 19);
`give { count: 1, item }` for items 19 and 34 against each of `7`, `8`, `18`, all `available: false`,
`too_far_away`. A-2: 6 buys, all target-less; 2 gives against Alice, both complete, in the server's
order, items differing; `talk` offered and incomplete; `affordances_about("22")` contains the buy.
A-4: `submit_affordance(talk)` → `""`, 0 requests emitted; Alice's first give sent as `c1`, answered
`{"rejected":"too_far_away"}`. A-3: the buy request equals the affordance on all five fields; `c2`
answered `accepted` (events 130, 131); wallet 200 000 → 199 600 (price 400 from the café's `shop`
listing), item 22 held 0 → 1; revision 1 at the buy → 3 after it, never decreasing. Transcript:
`clients/protocol/evidence/affordances-market-town.log`.

**E-B4a — the first live run, before D-1 and D-2. FAIL**, which is what found them: `SCRIPT ERROR:
Invalid call 'String' constructor` at the check's item comparison — `payload.item` is an object
`{ "entity": "22", "entity_type": "item" }`, not a string; the run then timed out at 60 s.

**E-B5 — mutations against the live check** (each in the working tree, run, reverted by restoring the
saved file; `grep -c MUTATION` → 0 in all three module files afterwards; the clean re-run PASS):

```text
M-B1  affordances() returns after the first match     FAIL A-2 buys 1 (6); gives 1 (2); complete 1 (2)
M-B2  submit_affordance skips available = false       FAIL A-4 "unavailable give is sent" (token "")
M-B3  submit_affordance sends an incomplete one       FAIL A-4 "talk returns \"\" and sends nothing"
      with an invented {} payload                          ["c1",1]
M-B6  revision not updated by observation frames      FAIL A-4 "revision rose after the buy" false
M-B7  affordances_about without the typed reference   FAIL A-2 "affordances_about(item) contains the
      (the frozen string-only rule)                        buy" false; reader FAIL e "tea … typed
                                                           reference" ["hand"]
M-B8  submit_affordance without _as_sent (D-2)        FAIL A-4: the give is answered
                                                           {"rejected":"precondition_failed"} — the
                                                           server could not read count 1.0
```

**E-B6 — `bash clients/protocol/run.sh evidence` on the head `5c10e06`. PASS (A-5).** Exit 0.
`git diff --exit-code` on `request-2d.json` and `request-3d.json` → 0: byte-identical to the committed
files. Against E-B0a's logs, with world-instance lines removed: `transcript-2d`, `transcript-3d` and
`transcript-3d-wanderer` identical; `simultaneous-2d` and `simultaneous-3d` differ only in the
interleaved ActionIds of the two concurrent clients (2d `c2` action 3 → 6; 3d `c2` 4 → 3, `c3` 5 → 4),
the race E-B0a already showed. Committed evidence restored (QSB-5).

**E-B7 — `cargo test -p mineworld-cli --test ac13_semantic_parity` on the head. PASS.** 2 passed,
0 failed. (The first attempt inside the gate chain never ran: `cargo` was not on that shell's PATH.
Recorded as not run, then run.)

**E-B8 — the 3D slice on the head (A-6).** `--drive`: PASS, 66.0 s, its 18 verdict lines identical to
E-B0b's. `--world --link`: first run **INCONCLUSIVE** — `shots/slice/server.log`: "cannot listen on
127.0.0.1:7979: Address already in use", so the slice joined another lane's server (different place ids,
4 link checks failed against a world it was not meant to see). Re-run with the port free: PASS, 107.2 s,
"all link checks pass", our server listening, its verdict lines identical to E-B0c's. A-7 scan:
the 194 added lines of `git diff 47c81d1 -- clients/protocol/mineworld` contain none of the 11 action
types the packs declare (`accept-invitation buy decline-invitation drink eat give invite
join-group-activity leave-group-activity move talk`), no `distance`/`length(`/`Vector2|3`/`Node2D|3D`/
`CollisionObject`/rule-constant name; "available" appears once, in `submit_affordance`'s doc comment
saying it is not read. ADOPTION §2 ↔ code: each of the six new functions is defined once and named in
ADOPTION, and `revision` is in both.

**E-B9 — scope.** `git diff --stat 47c81d1 HEAD`: 15 files, every one on §18.1's list (`checks/**`
includes the two `.uid` sidecars). No file under `demo/`, `clients/3d-spike/`, `server/`, any crate,
world or pack; no committed evidence file changed. Reader check 18/18 and the live check 22/22 last
ran on code identical to the head's (only Markdown changed after them).

M-B3's first form (skipping the completeness guard) crashed on the missing `payload` key instead of
sending; a crash is not the failure the claim is about, so it was replaced by the faithful form above
and both are recorded. M-B8 is beyond A-8's list: it is D-2's own evidence.

## 18.10 Deviations and discoveries during implementation

Both found by the first live run (E-B4a). Both are **bounded**: they change how SB-3 and SB-5 reach
their frozen result, not the result. A-2 ("affordances_about(item) contains the buy") and A-3/A-4
("submitted unchanged", "the server answers") are unchanged, and both would have failed in a real
world without these corrections. The audit's error is recorded as such: F-16b-7 read the `give`/`buy`
payload shapes from step-10's SD-20 text (`give { item: ItemId, count }`), not from a live frame.

| ID | Discovery | Decision | Evidence |
| --- | --- | --- | --- |
| **D-1** | **Every typed identity in a payload travels as the contract's `TypedEntityRef`**, `{ "entity": "22", "entity_type": "item" }` (`contracts/src/ids.rs:855`, `#[serde(into = "TypedEntityRef")]` on `ItemId`), not as a bare string. SB-3's "top-level String value" would match no `buy`, `give` or `eat` offered in any world today. | `affordances_about(id)` also matches a top-level value that is an object whose `entity` is the string `id`. That is the contract's own reference shape — the module already reads it in `place()` (`self_location.place.entity`) — so no pack shape is learned. Still top level only. ADOPTION §2 and the doc comment say so; the reader check gained the `pick` case. | E-B4a, E-B4; M-B7 (live and reader) |
| **D-2** | **Godot parses every JSON number as a double**, so an offered `"count": 1` is `1.0` in the affordance, and `JSON.stringify` writes it back as `1.0`. Submitted that way, the server cannot decode `give`'s `u32` count and answers `precondition_failed`. "Unchanged" as SB-5 states it is therefore impossible without restoring integers. | `submit_affordance` sends `_as_sent(payload)`: every finite whole-number double within ±2^53 becomes an `int`, recursively; strings (identities) and everything else pass through. That is the JSON the server sent. A system field declared as a float accepts an integer, so nothing is lost. `submit` itself is unchanged: a caller composing its own payload keeps `ADOPTION.md` §3.2's obligation. | E-B4; M-B8 |

## 18.11 Execution contract for PR 16b (proposed; confirmed at the freeze)

```text
PROJECT / PR        MVP-0 · Step 15 (S14) / PR 16b — the shared module: affordance readers, raw
                    components, revision
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-15-demo-3d.md §18; evidence §18.9 (E-B<n>);
                    deviations §18.10
RELATED / BINDING   overall.md "Parallel build-out, 2026-10-08" (rulings 4, 6, 7, 9, 10); this file
                    §§10 (I-S14-1, I-S14-3, I-S14-7, I-S14-10), 12, 13; step-13-client-2d.md §7;
                    DECISIONS ARC-31, ARC-34; server/PROTOCOL.md §§5–6; clients/protocol/ADOPTION.md;
                    CORE_CONCEPTS §15.2; CLAUDE.md §§2–4
IMPLEMENTATION BASE main @ 47c81d1; branch mvp0/pr-16b-shared-module; worktree
                    /Users/yuema137/mineworld-worktrees/impl-shared-module, held by this session only
APPROVED SCOPE      §18.1's path list; B-C1 … B-C4; SB-1 … SB-7 as answered by QSB-1 … QSB-7
FROZEN INVARIANTS   no edit outside §18.1's paths; demo.gd, space.gd, clients/3d-spike/**, the
                    committed evidence files, server/PROTOCOL.md, every Rust crate, world and pack
                    unchanged; evidence/request-{2d,3d}.json byte-identical after `run.sh evidence`;
                    the existing run.sh modes run identical server commands; no rule, no action-type
                    literal and no 2D/3D engine type in mineworld/ (I-S14-10, A-7);
                    submit_affordance never reads `available`; every test removes its own scratch data
SEQUENCE            B-C0 (this section, committed before the freeze) → B-C1 → B-C2 → B-C3 → B-C4, each
                    committed and pushed when coherent
VALIDATION BUDGET   static/unit/integration unrestricted; real runs: run.sh evidence (~2–3 min,
                    background) at most three times (E-B0, final, one re-run); run.sh affordances and
                    the slice's --drive / --world --link freely (each under ~2 min); no full Rust
                    workspace gate (no Rust change; ac13_semantic_parity only); about one hour in
                    total; real-model: NOT REQUIRED
LIVE DOCUMENTATION  §18.5 checkboxes; §18.9; §18.10
HANDOFF             .structured-coding/plans/mvp0/handoff-16b.md (QSB-3), initialized at B-C1
ENDPOINT AUTHORITY
  implementation + local validation   authorized at the primary session's freeze message — source: the
                                      primary session's kickoff for 16b, 2026-10-08 ("Phase 2 — after
                                      my freeze message: implement, run the gates")
  semantic commits, branch push       authorized after the freeze — same source
  PR creation / update                authorized after the freeze: open the PR READY FOR OPERATOR
                                      REVIEW — same source
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only; never inherited, never widened. The PR must land
                                      before S11-A (ruling 4)
POST-MERGE SYNC     this session owns §18 and its evidence; the primary/planning session owns the step
                    header, §§12–13 status, step-13-client-2d.md (S12 consumes SB-1 … SB-7 by these
                    names), overall.md §7, and telling S11-A to rebase
NORMAL STOP         PR 16b READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit outside §18.1's paths; a byte change in request-{2d,3d}.json or a
                    changed verdict in any existing run.sh mode or slice check that this PR's diff
                    explains; a need for a world rule, an action-type literal or an engine type in the
                    module; a wire change (PROTOCOL.md)
```

---

# 19. PR 16a — the client, ready for bodies (full design; DESIGN FROZEN 2026-10-08)

**Numbering.** This section was written and frozen as §18 (commits `fe3e814`, `04ce18c`); 16b, written in
parallel, also took §18 and merged first, so on merging `main` this one became §19, every "§18" in it
became "§19", and nothing else changed. Commit messages before the merge say §18.

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session` — record in §19.0. Implementation is
authorized by that freeze (`CLAUDE.md` §3.1; overall "Parallel build-out, 2026-10-08", ruling 9).

## 19.0 Freeze record

## DESIGN FROZEN

```text
Design revision     §§19.1–19.6 as committed in fe3e814
Approved by         the primary session, 2026-10-08 ("FREEZE: PR 16a is DESIGN FROZEN (2026-10-08),
                    primary session"), relayed to this implementation session
Implementation base main @ 47c81d1; branch mvp0/pr-16a-jolt-targeting
Execution contract  §19.8
Lifecycle           FROZEN
Rulings             §19 accepted as written, including the two departures (only LAYER_BODIES; the
                    tools-only commit). Q-16a-1 accepted: a small centre dot and a "looking at: …"
                    HUD line, connected only — a 3D visual default delegated to the primary session,
                    judged by the operator in play; the HUMAN_REVIEW_QUEUE checklist wording gains it.
                    Q-16a-2 … Q-16a-6 accepted as recommended. Report Bob's occlusion of the door talk
                    as measured. One Godot window at a time; 16b and S12 13a also run Godot, so a
                    stalled capture is re-run
```

## 19.1 Identity, base, approved scope

```text
PR            16a — the 3D client, ready for bodies (S14, first of 16a … 16e; GitHub number assigned
              at freeze, ruling 7)
base          main @ 47c81d1 (Merge #62, the six parallel step designs frozen)
branch        mvp0/pr-16a-jolt-targeting, worktree /Users/yuema137/mineworld-worktrees/impl-s14-16a,
              held by this session only
audit         §19.2 (files and symbols read on 47c81d1, 2026-10-08) on top of §2
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

## 19.2 Re-audit for 16a (`main @ 47c81d1`, 2026-10-08)

Every row was read in this session from the file named. §2's findings stand; these add what 16a's
file-level plan needs.

| ID | Finding | Evidence | Consequence for 16a |
| --- | --- | --- | --- |
| **A16-1** | One Godot project holds both scenes. `project.godot` has no `[physics]` section; `main.tscn` (`./mineworld-3d`, the promenade with the character's `--sweep`, `--headtrace`, `--frametime`, `--drive`) and `slice.tscn` (`./mineworld-slice`) both run on whatever engine it selects. Godot is `4.7.2.stable.official.ed1daf0bf` on this host | `clients/3d-spike/project.godot`; `godot --version` | The one-line switch changes **both** accepted artefacts: the route D+ evidence (`--sweep`, `--headtrace`, `--frametime`, all in `shots.gd`) is re-run as well as the slice's checks |
| **A16-2** | The slice's checks: `--drive`, `--measure`, `--threshold`, `--perf` (frame cost at four viewpoints), `--character`, `--shots`, `--link`, `--conversation`, dispatched by `SliceProbe._process`; `scripted()` and `with_hud()` list the flags. The launcher maps `--x` to `--slice-x` and runs `--drive` and `--link` headless | `scripts/slice/slice_probe.gd:102–165`; `mineworld-slice:53–64` | A new mode needs a flag in the launcher, in `scripted()`, and a dispatch arm |
| **A16-3** | The connected modes are one contiguous block: `_link_check` (303–472), `_conversation_frames` (479–555), `_street_watch` (562–604), `_talk_to` (609–631), `_walk_to` (635–639), `_answered` (642–646). `_walk_to` and `_answered` are used only by them. The standalone helpers they also call (`_hold`, `_settle`, `_save`, `_walk_dist`, `_walk_to_x`, `_inside_room`) are used by standalone modes too | `slice_probe.gd` (grep of every call site) | The block moves to the sibling unchanged; the shared helpers stay where they are and are reached by inheritance (§19.4, D-16a-3) |
| **A16-4** | Collision layers: `Build.LAYER_WORLD = 1` is the only layer; every scene collider is a `StaticBody3D` on it; the player is `collision_layer = 0`, mask `LAYER_WORLD`; the third-person boom's ray masks `LAYER_WORLD`. The probe's own measurement ray (`_ray`, `:1264`) and sun ray (`:1198`) use the default mask (all layers) | `build.gd:6, 28–157`; `player.gd:71–82`; `camera_rig.gd:172–173`; `slice_probe.gd:1198, 1265` | A person collider on layer 2 is invisible to the player and the camera. The probe's two rays would see it: they run only in standalone modes, where no perceived figure exists, but they are given `LAYER_WORLD` explicitly so a later connected measurement cannot be silently changed (bounded) |
| **A16-5** | The player's capsule is r 0.30, h 1.72, centre 0.86 m up, written as literals in `Player._ready`. The first-person eye is `CameraRig.EYE_HEIGHT = 1.66` | `player.gd:77–81`; `camera_rig.gd:32` | The pick collider uses the same capsule; the numbers become one named constant pair in `Player`, cited to bodies' `PERSON_RADIUS`/`PERSON_HEIGHT` (§4.2, QS14-3) |
| **A16-6** | The cone, `facing_person()`, is called only by `talk_to_facing`; `talk_to_facing` is called by the E handler and by the probe's `_talk_to`. The literal `"talk"` appears in `slice_link.gd` five times (`may`, `unavailable_reason`, `submit`, two token bookkeeping lines) | `slice_link.gd:431–472, 491, 522–525`; `slice_probe.gd:614` | All five move to `intents.gd`; afterwards `slice_link.gd` names `move` only |
| **A16-7** | Which world people stand where (social-cafe, hosted): visitor (the seat) at café (1610, 600); Alice (barista) at café (6000, 8000); Bob at café (4500, 6100); Wes (`wanderer`) at café (7110, 3900); Ivan on the **street** at (−4000, 1000). The café door is at café (1610, 200) ↔ street (0, 3000); in the scene the café door is x = 3.45, the pavement point z = −7.90, The Flower Room's door x = 11.84 | `worlds/social-cafe/people/*.yaml`; `places/cafe.yaml`; `cafe.gd:36`; `street.gd:35`; `slice_probe.gd:1428` | From the door, the line to Alice passes Bob's axis at **320 mm**. A capsule's horizontal section at the eye height of 1.66 m is 0.18 m in radius, so a ray aimed at Alice's head clears Bob by about 140 mm: the accepted "face Alice from the door, E → too far away" survives a ray, by measurement still to be taken (R-16a-2) |
| **A16-8** | **A non-vacuous wall test needs a person perceived behind a wall.** The observer perceives only its own place (F-S14-17), so "from the street, facing Alice through the back of the building" (§5) targets nobody **because Alice is not perceived**, not because of the wall: it cannot tell a ray from a cone. But the slice reports a body inside The Flower Room as being on the **street** (`PLACE_KEY[FLORIST_PLACE] = "street"`, F-S14-9), so from the back of the florist the street's people — Ivan — are perceived, and the florist's west party wall stands between | `slice_link.gd:70–74`; A16-7 | `--world --target`'s wall case is taken there (§19.3 T-2). It is valid until 16c binds the florist to `store`; 16c re-designs it then (recorded so it is not forgotten) |
| **A16-9** | **Action-type literals in client code today** (comments excluded): `slice_link.gd` (`"move"`, `"talk"` ×5); `human.gd:422` `mode.set_input_name(0, "move")` — an `AnimationNodeBlendTree` input name, not a request; `clients/protocol/demo/demo.gd` (excluded by I-S14-1.1); `clients/protocol/mineworld/world_client.gd` names `"talk"` only in `##` comments and `"join"` as a wire frame type (not an action type). Action types declared under `systems/*/src`: `buy eat drink invite accept-invitation decline-invitation join-group-activity leave-group-activity give move talk` (11; `bodies` declares none until 12c) | `grep` over `clients/**/*.gd`; `grep 'ActionTypeId::from_static' systems/*/src` | The scan must strip comments, and needs one entry-level allow-list line for `human.gd`'s `"move"` (Q-16a-5) |
| **A16-10** | **Rule-named constants in client code today:** none outside `clients/protocol/demo` (`STRIDE`, `reach`). `REPORT_DIST` and the capsule do not match I-S14-1.1's pattern | `grep` of `const`/`var` declarations over `clients/**/*.gd` | The rule-constant allow-list starts **empty**; the justifications §10 anticipated are not needed |
| **A16-11** | **Pack paths in client code today:** only in comments (`slice_world.gd:17`, `cafe_interior.gd:311`, `slice_probe.gd:297`) | `grep '(worlds\|systems)/'` | I-S14-6's scan, over string literals, starts clean |
| **A16-12** | **The leak's candidates.** `SliceMain._exit_tree` frees `Props._scenes`. Other script-static caches hold engine resources until script teardown, which may come after the rendering server's: `Mats._tex_cache`, `_mat_cache`; `Procgen._cache`; `Props._card_mats`; `Human._scenes`, `_libs`, `_mats`; `SlicePalette._rug`, `_serif`, `_script_font` (font atlases are textures) | `grep '^static var' scripts/**/*.gd`; `slice_main.gd:67–77` | Bisection over these is the investigation's first step (§19.3 C6) |
| **A16-13** | **No CI workflow** exists (`.github/workflows` absent; S13's 13a is in flight). The full Rust gate is local | `ls .github` | CI repair is `N/A`; the local gate runs once on the final head (§19.6) |
| **A16-14** | `./mineworld-slice --world` hosts on the fixed address `127.0.0.1:7979`. Parallel sessions may hold that port | `mineworld-slice:69–79` | Before each connected run the port is checked; if held, the same server command is started on a free port and the slice joined with `--server=` — the path `--world` itself takes (R-16a-3) |

## 19.3 Acceptance (decided before measuring, `ARC-23`)

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

## 19.4 Design decisions (D-16a-*)

| ID | Decision | Alternatives considered | Why |
| --- | --- | --- | --- |
| **D-16a-1** | **Jolt by the project setting alone**, `[physics] 3d/physics_engine="Jolt Physics"`, every other Jolt setting at its default; the values in force are printed by the engine probe and recorded | tune Jolt's settings up front; keep Godot Physics (§8.1) | §8.1's verdict (adopt). Defaults are what the engine's maintainers chose for new projects; a setting is changed only if a J-check fails and the change is bounded (then recorded) |
| **D-16a-2** | **Targeting is a stateless helper**, `scripts/slice/targeting.gd` (`SliceTargeting`, `RefCounted`, static functions): `aim(camera) -> Dictionary` casts one ray from the active camera through the viewport centre, `RAY_LENGTH = 30.0` m, mask `LAYER_WORLD \| LAYER_BODIES`, `collide_with_areas = false`; it returns the hit's entity id (read from the `entity_id` metadata of the collider's figure), the hit point and the collider's name; `person_collider() -> AnimatableBody3D` builds the pick capsule | a `RayCast3D` node on each camera (three cameras, mode switching); an `Area3D` per figure (§8.4: a radius is a reach rule — rejected) | §8.4 adopts the engine ray. A query from the *active* camera needs no node per camera and works across the three modes. The player is on no layer (A16-4), so a third-person ray never meets her own capsule and needs no start offset |
| **D-16a-3** | **Pick colliders.** Each perceived figure (`SliceLink._figure`) gets one `AnimatableBody3D` child, capsule `Player.CAPSULE_RADIUS` × `Player.CAPSULE_HEIGHT` (0.30 × 1.72, centred 0.86 m up), `collision_layer = LAYER_BODIES`, `collision_mask = 0`. `Build` gains `LAYER_BODIES = 2` only; `LAYER_OBJECTS` and `LAYER_DISCLOSED` (§4.2) are added by 12e with their first use | `StaticBody3D` (moved by teleport: Jolt and Godot discourage moving static bodies); declaring all three layers now as §14 lists | 12e masks the same body (§4.2), so its type is the one 12e needs. An unused constant is premature (`CLAUDE.md` §4 rule 11): a bounded deviation from §14's list, recorded |
| **D-16a-4** | **`intents.gd`** (`SliceIntents`, `RefCounted`, owned by the link): `TALK`, the request's name; `talk(client, target, utterance, actor_location) -> String` submits **regardless of the affordance** and remembers the request by token; `take(token) -> Dictionary` hands back `{ action, target, words }` for the answer's wording; `server_note(obs, target) -> String` returns the world's stated reason when it says talking is unavailable, for display only. The link keeps `move`, the E key, display labels and toasts, and asks intents what an answered token was | keep `talk` in the link (two files naming actions, no seam for 12e's three); a Node with signals | §4.4: one file builds every interaction request. 12e adds kick, throw and shove there and nowhere else |
| **D-16a-5** | **The probe split by inheritance**: `slice_probe_world.gd`, `class_name SliceProbeWorld extends SliceProbe`, holds the connected modes (`link`, `conversation`, the new `target`) and their helpers; `SliceProbe._process` calls an overridable `_run(mode)`; `SliceMain` instantiates `SliceProbeWorld` when a connected mode is requested. `scripted()` asks both | a third helper file both use (moves the shared walking helpers: more of the accepted probe edited); composition with a reference to the base probe (makes its private helpers public) | The connected modes *are* probe modes that need the same walking and capture helpers; inheritance moves only the connected block and edits the base in two places (dispatch, `scripted()`), which is what §5 asks ("not edited beyond what their claims need") |
| **D-16a-6** | **The scan strips comments.** A GDScript lexer in the test: `#` to end of line outside a string; `"…"`, `'…'`, `"""…"""` with backslash escapes; `&"…"`/`^"…"` (StringName/NodePath) read as strings | read comments too, as `precursor_vocabulary.rs` and `seam_vocabulary.rs` do | Those scans hold the *absence of vocabulary*; this one holds *where requests are built*, and a word in a comment builds nothing. The protocol module's documentation (`world_client.gd`'s `submit("talk", …)` example) may not be edited by 16a and is right to stay |
| **D-16a-7** | **Evidence**: before/after numbers and every decisive line go into this section's ledger (§19.7); side-by-side frames for any view J-4 flags, and the sweep sheets before/after, go to `clients/3d-spike/evidence/16a/` (with `.gdignore`), the directory 16e will use; raw captures stay in the ignored `shots/` | commit every frame | Small, reviewable, and the operator can open what matters |
| **D-16a-8** | **Tools**: `clients/3d-spike/tools/physics_engine.gd` (headless `--script`: prints the configured setting and the running server's class) and `tools/frame_diff.gd` (headless `--script`: two PNGs → mean absolute difference, share of pixels over a threshold, the bounding box of the differing region; optional side-by-side through the existing `compare.gd`) | ad-hoc scripts in `/tmp` (F-S14-2's probe was one, and is lost) | J-0 and J-4 become re-runnable by anyone; tools are excluded from the scan by I-S14-1.1 |

## 19.5 Questions for the freeze (primary session; **[OM]** = the operator's)

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-16a-1 [OM: visual default, judged in play]** | **A ray needs a reticle.** The cone accepted anything within ~37° of the view's centre; a 0.30 m capsule at 8 m is ~2°. Without a mark at the screen's centre the accepted checklist step 8 ("from the door, face Alice and press E") becomes hard to do by hand. (a) A small centre dot, and the targeted person's name on the HUD's world line ("looking at: Alice Moreau"), **only when connected**; (b) a dot only; (c) nothing until 12e's offer prompts | **(a).** Standalone frames and `--shots` are unchanged (no link, no reticle), so J-4 still isolates Jolt. The dot is 4 px, the slice's HUD colour; the operator judges it in play |
| **Q-16a-2** | T-2's wall case at the back of The Flower Room, which holds only until 16c binds the florist to `store` (A16-8) | **Yes**, with 16c re-designing it; the counter is kept as a second occluder in T-1's report (aiming at Alice's knees from the customer side meets the counter first) |
| **Q-16a-3** | J-3's frame-time bound of 15 % median, confirmed by a re-run | **Yes**; Jolt is not expected to move render cost, and 15 % sits above run-to-run noise measured on this machine (10.9–17.6 ms across viewpoints in the 2026-09-30 preview) |
| **Q-16a-4** | 16a's handoff: a separate file collides between parallel lanes (`handoff.md` is shared); keep 16a's continuation state in §19.8 of this section instead | **Yes**: one authority, no collision |
| **Q-16a-5** | `human.gd`'s `"move"` (an `AnimationTree` input name): an allow-list entry with its reason, or rename the input | **Allow-list.** Renaming touches the accepted character's animation graph for no behavioural gain |
| **Q-16a-6** | `HUMAN_REVIEW_QUEUE.md` `VIS-3D-GODOT-2` known limitation 8 ("Not attributed") becomes false once L-1 holds: append the attribution to that line | **Yes**, one line, appended, nothing else in the accepted entry edited |

## 19.6 Commit plan

Each commit tracks implementation, validation and review separately; a planned commit may become
several coherent ones (mapping recorded). Evidence goes into §19.7 as `E16a-<n>`. Every Godot run is in
the background when it may exceed two minutes, one Godot window at a time, with stdout to a log under
`clients/3d-spike/shots/16a/` (ignored).

### 16a-C0 — Design (this section) — docs only

- [x] Implementation: §§19.0–19.8, from the audit in §2 and §19.2.
- [x] Validation: `python3 scripts/check_doc_headings.py` (docs only; this plan is under
  `.structured-coding/`, which the script does not read — run to show nothing else moved): "176
  numbered sections across 25 documents, none duplicated"; `check_decision_ids.py`: "51 decision ids, all
  distinct". Headings §19.0–§19.9 are unique within this file.
- [x] Review: every finding cites a file and line or a command; every acceptance line names its pass
  condition before anything is run; every guard has a mutation; the non-goals match §13's 16a row and
  the shared-module ruling. Self-review by the drafting session; the primary session's freeze pending.

### 16a-C1 — The baseline on Godot Physics, and the two tools

**Goal.** "Before" is measured, on the base, by the same commands "after" will use (`ARC-23`).
**Scope.** New `clients/3d-spike/tools/physics_engine.gd`, `tools/frame_diff.gd`; this section's ledger.
No change to any script the scenes run. **Depends on** freeze.

- [x] Implementation: the two tools (D-16a-8). `physics_engine.gd` tells the engine by behaviour, not
  by class (E16a-1's finding); `frame_diff.gd` compares two PNGs or two directories of them.
- [x] Validation (E16a-1, all on the base's scenes, unmodified):
  - [x] J-0 before: Godot Physics (solver iterations 16, setting `DEFAULT`)
  - [x] slice: `--drive`, `--measure`, `--threshold`, `--character`, `--perf`, `--shots` (1600×900)
  - [x] promenade: `--headtrace` (reference body), `--sweep`, `--frametime`, `./mineworld-3d --drive`
  - [x] connected: `--world --link`, `--world --conversation`
  - [x] the leak: windowed slice modes, 7 Texture RIDs; not headless
  - [x] `frame_diff.gd`: self → 0; a different frame → non-zero (a different frame stands in for the
    shifted copy: it shows the instrument sees a difference, which is the claim)
- [x] Review: every baseline matches the accepted figures except two, recorded before anything changes:
  `--measure`'s stature 1.802 m out of its 1.70–1.80 range (pre-existing, the route D+ body), and the
  headtrace's jog yaw 5.07° against the accepted record's 4.2° (inside the 6° bound).

**Failure cases.** A baseline that fails its own check on the untouched base is a pre-existing defect:
recorded, not fixed here, and reported (it changes what J-* can claim).
**Commit boundary.** Tools and ledger only.

### 16a-C2 — Jolt (DEP-20)

**Goal.** J-0 … J-5. **Scope.** `clients/3d-spike/project.godot` (`[physics]`); `docs/DECISIONS.md`
**DEP-20** (step-11 §15.2's draft, with §8.1's table as its options, the 2026-10-08 facts, the isolating
interface — the project setting, the only place that names Jolt, §9.4 — and the revisit trigger: a
J-check that cannot be fixed in the client, QS14-12); `slice_probe.gd`'s `--drive` prints the running
engine and fails unless it is Jolt (one line in its report, J-1). **Non-goals:** any other script.

- [x] Implementation: the setting; DEP-20; the engine line in `--drive` (through `PhysicsEngineProbe`,
  the tool's `class_name`, so the discriminator lives once).
- [x] Validation (E16a-2): J-0 PASS; J-1 PASS; J-2 sweep PASS by inspection, headtrace jog bound **not
  met on either engine** (pre-existing, Jolt's distribution equals Godot Physics'); J-3 PASS; J-4 PASS,
  no visible change; J-5 PASS; M-1 PASS. Sweeps judged by inspection, not pixel difference (R-16a-5).
- [x] Review: every moved number is explained above (settle of a few mm; time-driven frames; the
  threshold probe's landing race; contention); `check_decision_ids` → "52 decision ids, all distinct".

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

- [x] Implementation: as scoped (D-16a-5). `SliceProbe` gains `MODES`, `requested_of(modes)`,
  `_modes()` and `_run(mode)`; `with_hud()` moved to `SliceProbeWorld` (it names a connected mode);
  `SliceMain._scripted()` asks both. Done on the merged head `05f5cb3` (16b's shared module in).
- [x] Validation (E16a-3, 13:35–13:41, Jolt): `--drive` "all drive checks pass"; `--world --link` "all
  link checks pass" with the same lines as C2 (50/50 moves, sent (1596, 1139) = server's, door
  too_far_away, counter accepted and reply heard, watch 60 s in 60.0 s); `--world --conversation`
  "conversation on screen, no ids", the same door toast. Port 7979 free before each run; the server was
  the launcher's own (its PID, this worktree's path). Lines: `slice_probe.gd` 1 649 → 1 311,
  `slice_probe_world.gd` 395.
- [x] Review: the moved block compared by content with its original (from "## C8, with S6" to the line
  before "## Every street door"): identical apart from the trailing blank lines. Every former call site
  of `scripted()`/`with_hud()` (`slice_main.gd` ×3) updated; no other caller exists.

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

## 19.7 Ledger (live)

Raw logs and frames live in `clients/3d-spike/shots/16a/{before,after,…}/` (ignored, this machine:
Apple M5, macOS 25.2, Godot 4.7.2.stable.official.ed1daf0bf, Metal 4.0 Forward+). Every windowed run
logs Metal "timeout waiting for fence" errors while its window is unfocused; they occur before and
after alike and no run stalled.

### E16a-1 — the baseline on Godot Physics (base `04ce18c` = 47c81d1 + plan docs; scripts untouched; 2026-10-08 12:50–13:03)

**J-0 instrument.** `godot --headless --path clients/3d-spike --script res://tools/physics_engine.gd`.
Finding (bounded, recorded): `PhysicsServer3D.get_class()` answers the abstract `PhysicsServer3D` under
either engine, and no method names Jolt, so the class cannot tell them apart. The discriminator is a new
space's `SPACE_PARAM_SOLVER_ITERATIONS`: **16** (= `physics/3d/solver/solver_iterations`) with the
setting `DEFAULT`, **8** with the setting `"Jolt Physics"` (tried by a temporary, reverted edit of
`project.godot`). Jolt also logs `Unhandled space parameter: '8'` from
`modules/jolt_physics/spaces/jolt_space_3d.cpp` when asked for that parameter — direct evidence the
module is running. Base: `setting 'DEFAULT'` → **Godot Physics** (16). So `DEFAULT` is Godot Physics in
this project, confirming step-11 §3.2's reading of the 4.6 note.

**`frame_diff.gd` shown to see:** a frame against itself → mean |d| 0.000, 0.000 % over 8/255; two
different frames → 61.597, 98.747 %, box the whole frame.

| Check | Wall | Base result (decisive lines) |
| --- | --- | --- |
| slice `--drive` (headless) | 79 s | all drive checks pass; Flower Room loop within 0.12 m |
| slice `--measure` | 18 s | **1 SCALE CHECK OUT OF RANGE, pre-existing**: "occupant stature, from its mesh 1.802 [1.70 .. 1.80]" — the route D+ body (`meshy_d.glb`); every other range in range. Recorded, not 16a's to fix; J-1's measure criterion becomes "the same lines as before" |
| slice `--threshold` | 22 s | café worst step ×1.65, florist ×1.42, 0.00 % clipped, all four claims PASS for each |
| slice `--perf` | 18 s | mean / p50 ms: street wide 13.59 / 13.52; café frontage 18.50 / 18.60; interior 15.14 / 15.14; doorway 18.53 / 18.43 |
| slice `--character` | 104 s | all character checks pass (idle 0.014 m ×2, walk 0.805 m) |
| slice `--shots` | 26 s | 67 PNGs (with `--character`'s), 1600×900, archived to `before/frames/`; no "body standing" warning |
| promenade `--headtrace` | 12 s | reference body, p2p yaw / pitch / roll: walk 3.81 / 0.45 / 0.23; jog **5.07** / 1.18 / 0.40 (accepted record: 4.2 / 1.1 / 0.4 — within the 6° bound both times; recorded as the base's own figure) |
| promenade `--sweep` | 16 s | 48 frames, archived to `before/promenade/sweep/` |
| promenade `--frametime` | 23 s | frame median / p95: stand 7.16 / 8.72 ms, walk 8.35 / 8.95 ms (GPU timer reports 0.00 on this host) |
| `./mineworld-3d --drive` | 123 s | forward 1.45 m/s; café walk-in z −14.60 → −9.92 inside; jog 3.04 m/s; three camera switches diff ≤ 0.00002 m; booms 3.44 / 0.61 / 1.02 m; shopfront end z −12.30 STOPPED; fountain −19.25 STOPPED; quay rail −25.57 STOPPED |
| `--world --link` | 245 s | all link checks pass: 50 moves 50 accepted 0 not; places cafe→street→cafe; position sent = server's (1596, 1140); door talk 8.15 m → rejected too_far_away; counter 19 moves 0 not; counter talk 1.99 m accepted, reply heard; street person 15 at (−0.55, 0.14, −5.90) (Ivan, A16-8 as computed); watch 60 s in 60.0 s |
| `--world --conversation` | 33 s | conversation on screen, no ids; door 8.60 m "can't talk to Alice Moreau: too far away"; counter 1.88 m accepted, reply on screen 0.3 s |

**The leak, starting fact (L-1).** `WARNING: 7 RIDs of type "Texture" were leaked.` on exit of every
windowed slice mode (`--measure`, `--threshold`, `--perf`); **not** on the headless `--drive`. The
promenade (`main.tscn`) leaks far more on exit (40 Texture RIDs, meshes, materials, 55 ObjectDB
instances, "Leaked instance dependency") — the promenade is outside 16a's scope (F-S14-1) and is
recorded only.

### E16a-2 — on Jolt (working tree = `97c1035` + the `[physics]` line, the `--drive` engine line and DEP-20; 13:04–13:33)

**Contention, recorded.** Lane S12 13a ran headless Godot clients and imports at the same time (seen
with `pgrep`, logged per run). The first after-run's `--shots` and `--sweep` took ~2 min instead of
~20 s, and a second Godot Physics `--shots` (`before2`) wrote **black frames** from view 15 onwards: those
frames are INCONCLUSIVE and are not used. Every timing claim below therefore rests on repeated runs of
both engines, interleaved, not on one run each.

**J-0: PASS.** `physics_engine.gd` → `setting 'Jolt Physics'`, "Jolt Physics (a new space's solver
iterations 8; Godot Physics' setting 16)"; `--drive` prints the same line.

| Check | Godot Physics | Jolt | Verdict |
| --- | --- | --- | --- |
| slice `--drive` | all pass; loop 13.27 m ending 0.10 m off; jumps 0.488 / 0.491 m; florist loop 0.12 m | all pass; loop 13.25 m ending 0.10 m off; jumps 0.488 / 0.488 m; florist loop 0.13 m; cameras 0.0000 m | **PASS** — every printed position within 31 mm, every verdict line identical |
| slice `--measure` | 1 out of range (stature 1.802, pre-existing) | the same line; occupant feet y 0.119 → 0.123 | **PASS** (same lines; 4 mm settle) |
| slice `--threshold` | café worst ×1.65 (run 1), ×1.57 (run 2); florist ×1.42 | café ×1.08; florist ×1.42; all claims PASS | **PASS** — see the finding below |
| slice `--character` | all pass, walk 0.805 m | all pass, walk 0.804 m | **PASS** |
| promenade `./mineworld-3d --drive` | (E16a-1) | every "STOPPED"/"clear"/"PULLED IN" verdict identical | **PASS** |
| `--world --link` | all pass (E16a-1) | **all link checks pass**: 50 moves 50 accepted 0 not; cafe→street→cafe; sent (1596, 1139) = server's; door 8.15 m → rejected too_far_away; counter 19 moves, 0 not; counter 1.99 m accepted, reply heard; watch 60 s in 60.0 s | **PASS** |
| `--world --conversation` | (E16a-1) | conversation on screen, no ids; door 8.60 m toast "can't talk to Alice Moreau: too far away"; counter 1.88 m, reply 0.4 s | **PASS** |

**J-2 headtrace — the 6° jog bound is not met on either engine today; Jolt does not move it.** Reference
body, steady-phase peak-to-peak yaw / pitch / roll, every run of the day:

```text
                 walk                          jog
Godot Physics    3.81/0.45/0.23  3.72/0.49/0.23  5.07/1.18/0.40  7.85/0.79/0.43
                 3.86/0.52/0.25  4.03/0.56/0.25  6.71/0.76/0.49  6.26/0.79/0.51
                 4.04/0.56/0.25  3.86/0.52/0.25  5.97/0.81/0.51  6.74/0.72/0.49
Jolt             3.91/0.54/0.25  3.69/0.52/0.26  6.56/0.73/0.48  6.05/0.77/0.51
                 3.85/0.53/0.25  4.03/0.52/0.26  6.25/0.76/0.49  6.76/0.75/0.51
```

Walk is within 6° on every run of both engines. Jog yaw: Godot Physics 5.07–7.85 (mean 6.43, 6 runs),
Jolt 6.05–6.76 (mean 6.41, 4 runs) — the same distribution. The accepted record's 4.2° (CHARACTER_ROUTE_D_PLUS
§8.8) was one run; on this base, with Godot Physics, 5 of 6 runs exceed 6°. **Classification: a
pre-existing instability of the instrument or the head-steady filter, not a regression by Jolt**; the
switch is not the cause and 16a does not hide it. It is raised to the primary session in the PR (the
character track's item), not fixed here (scope).

**J-2 sweep: PASS by inspection.** Pixel differences are meaningless for the sweep, as R-16a-5
predicted: two Godot Physics runs differ by 23–97 % of pixels per frame, as much as Godot Physics
against Jolt. The Jolt frames were inspected one at a time at the operator's three defects —
`jog_rear_2` (the pack on her back, side panels flat, no slits or ridges), `walk_rear_tq_5` (the sleeve
passes beside the pack's side panel), `walk_front_tq_3` (both hands free and swinging, nothing held
into the hoodie; head facing ahead) — and show none of them.

**J-3 frame time: PASS** (medians, ms; every run):

```text
--frametime stand   Godot Physics 7.16 7.83 7.04 6.66 6.79 6.60  (mean 7.01)
                    Jolt          8.48 6.99 7.31 8.14            (mean 7.73, +10 %)
--frametime walk    Godot Physics 8.35 7.41 7.75 7.00 7.05 6.94  (mean 7.42)
                    Jolt          8.73 7.16 7.73 6.55            (mean 7.54, +2 %)
--perf p50          Godot Physics street 13.52 12.03 12.00 12.11 · frontage 18.60 17.43 17.38 17.92
                                  interior 15.14 14.99 14.73 15.17 · doorway 18.43 18.77 18.25 18.97
                    Jolt          street 14.28 12.45 11.98 12.11 · frontage 18.29 17.51 17.76 17.37
                                  interior 15.41 14.81 14.54 14.51 · doorway 18.98 18.32 18.52 18.05
```

No mean rises 15 %; the largest, `--frametime` standing (+10 %), is inside its own run-to-run range
(6.60–7.83 on Godot Physics).

**J-4 frames: PASS — no visible change.** `frame_diff.gd` over the 67 frames of `--shots` and
`--character`, Godot Physics run 1 against Jolt, with Godot Physics run 2 as the noise control
(`shots/16a/diff_*.txt`):
- The time-driven frames (`char_*_walking`, `_jogging`, `char_walk_*`, `char_run_into_townsperson`)
  and `th_outside_pavement` differ as much between two Godot Physics runs as against Jolt.
  `th_outside_pavement` is a **probe timing finding**: `_threshold_run` places the body 0.31 m above the
  pavement and captures after 18 render frames, which can be before it lands (vsync off, frames faster
  than physics ticks), so the first sample's camera height — and its luma, 0.2246 / 0.1475 / 0.1478,
  and so the doorway step ×1.65 / ×1.57 / ×1.08 — varies by run. Pre-existing and not 16a's; recorded.
- The static views that Godot Physics reproduces exactly but Jolt changes: `01r` 19.0 %, `04c` 11.6 %,
  `07` 4.4 %, `08` 3.8 %, `02` 3.3 %, `03` 2.8 %, the threshold samples 1–5 %, the rest under 2 %. A
  diagnostic run (a scratch, uncommitted 1 s settle in `_capture`, both engines) put the body at the same
  height on both engines (y 0.1398 vs 0.1401; camera 1.7998 vs 1.8001) and left the differences
  unchanged, so they are the body coming to rest a few millimetres differently on Jolt, which shifts the
  high-frequency setts and shadow edges by sub-pixel amounts. Inspected one at a time at 1600×900:
  `01r` (the largest) and `04c` (the character in the doorway) read identical — the same street, the
  same light, the character inside both jambs with nothing clipped.
- Deviation from D-16a-7, bounded: no side-by-side images are committed, because no view changed
  visibly; the frames are kept in the ignored `shots/16a/` and every number is here.

**M-1: PASS (the guard bites).** With the `[physics]` line removed, `--drive` printed "engine Godot
Physics (… 16 …)", "FAIL: the slice is selected to run on Jolt Physics (DEP-20)", "1 DRIVE CHECKS
FAILED". Line restored.

## 19.8 Execution contract (proposed; confirmed at freeze)

```text
PROJECT / PR        MVP-0 · S14 / PR 16a — the 3D client, ready for bodies
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-15-demo-3d.md §19 (this section)
RELATED / BINDING   overall.md §3 (S14, S15), "Parallel build-out, 2026-10-08" (rulings 4, 6, 7, 9, 10);
                    this file §§1–17 as frozen at step level; step-11-bodies.md §§3.2, 15.2;
                    ENGINEERING_RULES §§3–12, 19; VISUAL_SLICE; HUMAN_REVIEW_QUEUE (VIS-3D-GODOT-1/-2);
                    docs/references/CHARACTER_ROUTE_D_PLUS.md §§8.7–8.9; REUSE_POLICY
IMPLEMENTATION BASE main @ 47c81d1; branch mvp0/pr-16a-jolt-targeting; worktree
                    /Users/yuema137/mineworld-worktrees/impl-s14-16a (this session only)
APPROVED SCOPE      §19.1
FROZEN INVARIANTS   I-S14-1 (16a's part: .1 and the talk half of .2's behaviour), I-S14-2, I-S14-3,
                    I-S14-6, I-S14-7, I-S14-10; no edit to clients/protocol/**; no regression of
                    VIS-3D-GODOT-1 or -2
SEQUENCE            C0 → (freeze) → C1 → C2 → C3 → C4 → C5 → C6 → C7, each committed and pushed when
                    coherent
VALIDATION BUDGET   static/unit/scan: unrestricted; real client runs: each ≤ ~10 min, background when
                    > 2 min, one Godot window at a time, total ≤ ~3 h of wall time including retries;
                    full cargo test: once, on the final head; real-model: NOT REQUIRED
LIVE DOCUMENTATION  this section (§19.6 checkboxes, §19.7 ledger)
HANDOFF             §19.9 of this section (Q-16a-4), refreshed at each commit
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
POST-MERGE SYNC     the primary session owns step §13 and overall; this session owns §19 (merge identity,
                    evidence, deviations, remaining issues)
NORMAL STOP         PR 16a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a regression of an accepted visual or a confirmed J-3 rise; any server, kernel,
                    contract or shared-module change; Jolt unavailable or unfit (reported, no silent
                    fallback); a change to §19.1's scope or to a frozen invariant
```

## 19.9 Handoff (live; replaces a separate file for this lane, Q-16a-4)

```text
checkpoint     C2 committed (ec7aadd); origin/main merged (16b #66, 12c #67, client-parity rule #69,
               physics list #71); this section renumbered §18 → §19
next action    C3: move the connected probe modes to slice_probe_world.gd
background     none
notes          12c merged: kick, throw and shove are now declared action types; the scan sees them.
               Coordinator 2026-10-08: check port 7979 before each connected run, kill only own
               PIDs, a run on a foreign world is INCONCLUSIVE; decorative townspeople hidden when
               connected (QS14-9) is outside 16a's frozen files (streetscape.gd) -> recorded as
               16c's first item; the geometry probe is 12e's (§5), not 16a's
```

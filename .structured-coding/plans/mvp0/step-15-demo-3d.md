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
**PR 16c:** detailed to the commit in §20, `DESIGN FROZEN (2026-10-08), primary session` (record in §20.0), drafted on
`plan/s14-16c` from `main @ 9cf8f8e`.
**PR 16d:** detailed to the commit in §22, `DESIGN FROZEN (2026-10-09), primary session` (record in §22.0), drafted on `plan/s14-16d` from
`main @ f80bbb7` (§21 is left to S15 12e's design, drafted in parallel on `plan/s15-12e`).
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

- [x] Implementation: as scoped, plus three bounded additions recorded in E16a-4: a wording table
  (`SliceIntents.VERBS`, "talk to") so the link's answer lines name no action; the launcher's
  `--port=` and a refusal to join when the hosted server could not listen (the coordinator's 16b
  finding, inside this PR's launcher edit); `ControlsHud.add_reticle()` (Q-16a-1).
- [x] Validation (E16a-4): T-1 … T-4 PASS; `--link`, `--conversation`, `--drive`, `--character` PASS;
  M-3, M-4 PASS (bite); M-2 replaced by the in-run counterfactual (below).
- [x] Review: the request is sent whatever `may` says (M-4 proves the probe would see otherwise); the
  ray mask is `LAYER_WORLD | LAYER_BODIES`, both defined; `slice_link.gd` has no action literal but
  `"move"` (the scan, C5); the reticle and "looking at" line are added only in `SliceMain._link()`, so
  standalone runs and `--shots` have neither.

**Failure cases.** T-1's door ray meeting Bob (R-16a-2): reported with the measured clearance, not
hidden by aiming elsewhere; the probe then aims where a player would see Alice, and the case is raised
in the ledger. **Commit boundary.** Targeting and intents, with their probe mode.

### 16a-C5 — The no-rule scan (I-S14-1.1, I-S14-6)

**Goal.** S-1. **Scope.** New `tests/acceptance/tests/client_rules.rs` (std only, no dependency; the
lexer, the three rules, the allow-lists with reasons, fail-closed roots); `tests/acceptance/src/lib.rs`
(the crate's list gains `client_rules`). **Non-goals:** S12's files (it adds its allow-list lines).

- [x] Implementation: as scoped (D-16a-6). Bounded deviation: `clients/protocol/checks/` (added by 16b
  after this design, the shared module's own live checks of named affordances) is excluded beside
  `demo/`, for the same reason — it exercises the protocol as a test does and builds no client's
  requests; recorded in the file's documentation.
- [x] Validation (E16a-5): green (3 passed); clippy `-D warnings` and fmt clean; M-5 … M-10 PASS.
- [x] Review: three action admissions (A16-9: `slice_link.gd` "move", `intents.gd` "talk", `human.gd`
  "move"), no rule-name admission, each with a reason; the scan reads the working tree; directories are
  only skipped by the three named exclusions, symlinks and hidden directories, and a missing root
  panics.

**Commit boundary.** The test file and the crate's doc list.

### 16a-C6 — The texture RID leak

**Goal.** L-1. **Scope.** Found by bisection over A16-12's caches (release each in `SliceMain._exit_tree`,
count the leaked RIDs per release), then a minimal scene if none is ours. If ours: each cache's owner gains
a `release()` static, called from `SliceMain._exit_tree` beside the existing `Props._scenes` release (the
promenade, `main.gd`, is left as it is, F-S14-1, and its exit is reported). If not ours: no code change.

- [x] Implementation: as the evidence decided — **engine-side (L-1 b)**; no fix in the client. The
  reproduction is kept as `clients/3d-spike/tools/reflection_probe_leak.gd`.
- [x] Validation (E16a-6): every script-static cache released → still 7; one `ReflectionProbe` in an
  empty scene → the same 7, `none` → none. M-11 is N/A: there is no fix of ours to remove.
- [x] Review: the bisection's cache releases were a scratch edit, reverted (`slice_main.gd` as C4 left
  it); nothing in the client changed for the leak.

**Commit boundary.** The fix (or nothing) and the ledger.

### 16a-C7 — Status, documents, gates, close

**Scope.** `docs/MVP_STATUS.md`: the 3D column of Movement, Place, Conversation and Names, the 3D client
artefact row and the S14 stage row (§17.4, made true on the head); `docs/HUMAN_REVIEW_QUEUE.md`
limitation 8 (Q-16a-6); this section's ledger and closeout.

- [x] Implementation: as scoped; plus one bounded fix found by the gate — the scan's admission of the
  join frame's `"invite"` key (E16a-7, `841fc9d`).
- [x] Validation (E16a-7, on the final executable head): the slice's checks once more (`--drive`,
  `--measure`, `--threshold`, `--character`, `--world --link`, `--world --conversation`,
  `--world --target`) — I-S14-7; G-1, including the path check of the diff.
- [x] Review: every acceptance line has its evidence; deviations recorded; the PR body carries the
  operator's short runnable list (overall memory: milestone handoff), including what the reticle looks
  like and how to aim. Decorative townspeople hidden when connected and the strict 150 mm geometry
  probe stay outside 16a's frozen files: 16c's first item and 12e's (§19.9).

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

### E16a-3 — the probe split (recorded with C3, §19.6)

### E16a-4 — ray targeting, intents, the reticle (working tree on `1f898c8`, Jolt, 13:42–14:05)

**A failed first run, diagnosed.** The first `--world --target` failed 5 checks: the ray met the back
wall behind every figure. Cause: the pick collider is an `AnimatableBody3D`, whose `sync_to_physics`
defaults to on; the figure is placed by setting `global_position` from an observation, outside any
physics-frame motion, so the body stayed where it was created. Fix: `sync_to_physics = false` in
`SliceTargeting.person_collider()`, commented. (12e, which moves figures every frame, re-examines it.)

**`./mineworld-slice --world --target` — all target checks pass:**

```text
clear    Bob Achterberg (8) beside the line to Alice Moreau: axis 0.321 m from it, capsule half-width
         0.253 m at y 1.58 -> clearance +0.067 m
aim      from the door, at her head         -> Alice Moreau (7)  [first hit .../Person_7/PickBody]
talk     from the door, 8.60 m -> rejected too_far_away
aim      from the door, at her knees        -> nobody  [first hit .../DailyBean/Interior/@StaticBody3D@4343
         at (6.99, 1.01, -15.01)]  (the counter's front)
aim      at the counter, first person       -> Alice Moreau (7)
aim      at the counter, third person rear  -> Alice Moreau (7)
aim      at the ceiling                     -> nobody; talk -> token '', 0 answers arrived
street   body at (3.59, 0.14, -6.01), server place street, 1 people perceived
aim      from the pavement                  -> Ivan Petrov (15)
florist  body at (11.88, 0.26, -13.06), slice place florist.main, server place street, 15 perceived
aim      from the florist, through its wall -> nobody  [first hit .../Unit_10/FlowerRoom/@StaticBody3D@1520
         at (10.80, 1.90, -12.44)]  (the florist's west wall, 4.3 m behind the frontage)
cone     the pre-16a rule would have chosen: 15
all target checks pass
```

**Bob and the door talk (R-16a-2), as measured:** aimed at Alice's head from the visitor's seat, the line
passes Bob's axis at **0.321 m**; his capsule is **0.253 m** wide at the height the line crosses him
(y 1.58), so the ray clears him by **67 mm** and meets Alice. The audit's 140 mm assumed a horizontal ray
at eye height; aiming at the head, slightly down, crosses Bob's capsule where it is wider. In the
captured frame (`conversation_1_from_door`) Alice stands almost directly behind Bob from the door —
labels overlapping, as `VIS-3D-GODOT-2` limitation 13 records — and the reticle sits just right of Bob's
head. So the accepted checklist step still works, but **by a few centimetres of aim**: a player pressing
E on Bob's head from the door targets Bob, and the answer (too far away) is about him. Reported for the
operator's play; nothing in the client widens a target to compensate.

**Regression runs on the ray** (all PASS): `--drive` all pass; `--character` all pass; `--world --link`
all link checks pass, identical lines to C2 (door talk to Alice, rejected too_far_away; counter accepted,
reply heard); `--world --conversation` "conversation on screen, no ids". A first conversation run showed
the toast "can't talk Alice Moreau" — the action name had replaced the old hand-written "talk to"; the
wording table (`SliceIntents.VERBS`) restored "can't talk to Alice Moreau: too far away", re-run
confirmed. The frame now carries the reticle at the centre and the HUD line "looking at: Alice Moreau".

**Mutations.** M-3 (ray mask `LAYER_BODIES` only): **3 TARGET CHECKS FAILED** — Alice targeted through
the counter at her knees, Ivan targeted through the florist's wall, the first hit not a wall. M-4
(`intents.talk` returns without sending unless `may`): **1 TARGET CHECK FAILED** — "talk from the door,
8.60 m -> NO ANSWER". Both reverted (no `MUTATION` marker left; `git status` shows only intended files).
M-2 (the cone restored inside `targeting.gd`) is replaced, bounded deviation: `targeting.gd` knows no
figures, so a cone there would be a different function, and the claim M-2 was to show — that the wall
case discriminates a cone from a ray — is shown in every run by the probe's own cone computation ("the
pre-16a rule would have chosen: 15", a FAIL condition if it did not).

**Port, recorded.** 7979 was free before every connected run (`lsof`), and each server was the launcher's
own child (killed by PID on exit, never by name).

### E16a-5 — the no-rule scan (`tests/acceptance/tests/client_rules.rs`, on `acbebbf` + the test)

First run on the base (before C4) failed as it should, naming `slice_link.gd`'s `"talk"` literals; after
C4 it is green: `cargo test -p mineworld-acceptance --test client_rules` → 3 passed (the scan, the
lexer, the rule-name and pack-path recognisers). It reads 36 scripts (33 of the 3D client, 3 of the
shared module) and collects the build's action types from `ActionTypeId::from_static` — since 12c merged
these include `kick`, `throw` and `shove`, so the design's M-5 expectation ("kick not reported") no
longer applies and both plants are reported.

| Mutation | Plant | Result |
| --- | --- | --- |
| M-5 | `const _PLANT_M5 := ["talk", "kick"]` in `targeting.gd` | FAILED: `targeting.gd:20: the action "talk" …`, `targeting.gd:20: the action "kick" …` |
| M-6 | `const TALK_REACH := 2.0` in `intents.gd` | FAILED: `intents.gd:15: \`TALK_REACH\` names a rule` |
| M-7 | `load("res://../../worlds/social-cafe/people/alice.yaml")` in `slice_link.gd` | FAILED: `slice_link.gd:25: "res://../../worlds/…" names a pack's file` |
| M-8 | an admission of `"shove"` in `targeting.gd` | FAILED: `the admission of "shove" admits nothing; remove it` |
| M-9 | the action-type root pointed at `clients/` | FAILED: `the build's action types were not found …: []` |
| M-10 | (kept unit test) a `"shove"` in a comment, a `"#"` in a string before `"move"`, an escaped quote, a triple-quoted string over two lines | the lexer test passes: comment not read, the rest read with the right line numbers |

All plants reverted; the scan green again; `git status` shows only the test and the crate's doc list.

### E16a-6 — the texture RID leak: engine-side (on `e50e3b0`, Jolt, 14:10–14:25)

1. **Not the project's caches.** A scratch edit released every script-static cache A16-12 lists in
   `SliceMain._exit_tree` (`Mats._tex_cache`, `_mat_cache`, `ProcGen._cache`, `Props._card_mats`,
   `Human._scenes`, `_libs`, `_mats`, `SlicePalette._rug`, `_serif`, `_script_font`, besides the
   existing `Props._scenes`): `--measure` still ended `WARNING: 7 RIDs of type "Texture" were leaked.`
   Reverted. (A first attempt named the class `Procgen` instead of `ProcGen`; the slice failed to parse
   and the windowed launcher waited on it until killed — a launcher behaviour, recorded only.)
2. **A minimal scene, part by part** (a SceneTree script, 30 frames at 320×180, one camera plus one
   part): nothing, a panorama sky with the slice's HDRI, a procedural sky, SSAO + SSIL + glow, a
   shadowed sun, a `VoxelGI` baked at load, a `Label3D`, a HUD label — **no leak**. One
   `ReflectionProbe` — **7 Texture RIDs leaked**, the slice's exact line. The same with
   `UPDATE_ALWAYS`, and the same when the probe is **freed at frame 20**, before the quit. Headless: no
   leak (nothing is rendered). The slice has two probes and leaks 7, so the textures are the renderer's
   per-viewport reflection storage, not per probe.
3. **Classification: engine-side**, Godot `4.7.2.stable.official.ed1daf0bf`, Metal 4.0 Forward+, Apple
   M5. Removing the slice's probes would drop `VISUAL_SLICE.md` §7.1's "sane reflections" (an accepted
   visual), so no workaround is taken. The reproduction is committed as
   `clients/3d-spike/tools/reflection_probe_leak.gd` (`-- probe` leaks 7, `-- none` leaks none; run
   again to confirm), so an upstream report or an engine upgrade can be checked in one command.
   The promenade's much larger exit report (E16a-1) is a separate matter, out of 16a's scope.

### E16a-7 — the final executable head `9ef9c61` (merged with S11-A's protocol 2; 14:07–14:40)

**The merge.** `origin/main` brought S11-A (#76): `slice_link.gd` connects with an invite and a nickname,
and `--world` hosts on port 0 and reads the server's join line. Conflicts resolved in S11-A's favour as
the coordinator directed: the launcher keeps S11-A's flow, and 16a's `--port=` and "port in use"
refusal (added in C4 on the coordinator's 16b finding) are **dropped as superseded** — a port the OS
chooses cannot collide, and the join line names the server this run started. `slice_main.gd` starts
the link with S11-A's arguments and keeps 16a's reticle. `DECISIONS.md`: main's records, then DEP-20.

**Every check, on that head** (Jolt: "a new space's solver iterations 8"):

| Check | Result |
| --- | --- |
| `--drive` | PASS — engine Jolt; all drive checks pass |
| `--measure` | the base's lines (the pre-existing stature line, E16a-1) |
| `--threshold` | first run **stalled** (INCONCLUSIVE) — a windowed Godot of lane S12 13a covered ours, as R-16a-4 foresaw; killed and re-run alone: PASS, café worst ×1.57, florist ×1.42, 0.00 % clipped |
| `--character` | PASS — all character checks pass |
| `--world --link` | PASS — on `127.0.0.1:62292` (port 0); 50 moves accepted, sent = server's; door too_far_away; counter accepted, reply heard; watch 60 s in 60.0 s |
| `--world --conversation` | PASS — "can't talk to Alice Moreau: too far away"; both lines at the counter; no ids |
| `--world --target` | PASS — T-1 … T-4 as E16a-4, Bob cleared by 67 mm again |
| `./mineworld-3d --drive` | PASS — the three STOPPED verdicts, drive test done |
| leak | the same 7 Texture RIDs on windowed exits (engine-side, E16a-6) |

**Resumed session (the first was ended by an API rate limit during the gate run).** Found: three
uncommitted files — `MVP_STATUS.md` and `HUMAN_REVIEW_QUEUE.md` (C7's scoped edits, kept and committed
after review) and this ledger's E16a-7 (kept) — plus the untracked `reflection_probe_leak.gd.uid`
(Godot's uid for C6's tool; every other tool's uid is tracked, so kept). No Godot, server or cargo
process of this worktree was still running. The interrupted `cargo test --workspace` on `9ef9c61`
(`shots/16a/final/gates.log`) had already **failed one test**: `client_rules`, on
`clients/protocol/mineworld/world_client.gd:345: the action "invite" is named outside the files that
build requests`.

**A bounded discovery, fixed in the test (`841fc9d`).** S11-A's protocol 2 put an `"invite"` key (the
join credential, `PROTOCOL.md` §§2, 4.1) in the shared module's join frame. group-activity declares an
action type spelled `invite`, so the scan, which reads every literal, reported it. It is a wire key, not
a request — the same kind of collision as `human.gd`'s animation input (Q-16a-5). Resolution: one
admission `(world_client.gd, "invite", reason)` in `ACTION_LITERALS`; the shared module is not edited
(16a's non-goal). The admission is checked like the others: if the key goes, the entry fails as stale.
The commit touches no script a scene runs, so the Godot runs above stand for the PR head.

**Gates (G-1), on `841fc9d`:** `cargo fmt --all --check` clean; `check_decision_ids` "56 decision ids,
all distinct"; `check_doc_headings` "191 numbered sections across 26 documents, none duplicated"; the
diff against `origin/main` names no file under `kernel/`, `contracts/`, `persistence/`, `server/src`,
`systems/`, `cognition/`, `worlds/` or `clients/protocol/`; `client_rules` 3 passed. `cargo clippy --workspace --all-targets -- -D warnings` exit 0.
`cargo test --workspace` (one complete run; an earlier one on the same head was killed by a host restart
inside `market_town` and is not counted): exit 0, 154 result lines all `ok`, 685 passed, 0 failed,
1 ignored (pre-existing), `market_town_lives_three_hundred_days…` ok, `client_rules` ok
(`shots/16a/final/gates3.log`). **G-1 PASS.**

**Main moved again before the PR opened** (#77, test-hygiene: a scratch helper and
`check_scratch.py`; no client file). Merged as `6a2eafb`; the one conflict, `DECISIONS.md`, was
resolved by keeping both records (DEP-20, then main's DEP-29). G-1 re-run on `6a2eafb`: fmt clean;
clippy `-D warnings` exit 0; `cargo test --workspace` exit 0, 157 result lines all `ok`, 692 passed,
0 failed, 1 ignored (bodies-yard's AO-2 evidence, by design); `check_decision_ids` "57 … all distinct";
`check_doc_headings` "191 … none duplicated"; `check_scratch.py scan` "143 test sources, none makes
scratch outside mineworld-test-support"; the path check is still empty (`shots/16a/final/gates4.log`).
No client script changed in that merge, so the Godot runs above stand. **G-1 PASS on the PR head.**

**After the PR opened, main took E-b (#78, `4bdbca1`)**, and `DECISIONS.md` conflicted again. Merged as
`13a4a72` on the coordinator's instruction. The resolution keeps both sides' records unchanged: DEP-20,
then main's ARC-54, ARC-55 and the ARC-53 note. Against main the file only gains lines. No client file
changed, so the Godot checks were not re-run. G-1 on `13a4a72`: fmt clean; clippy `-D warnings` exit 0;
`cargo test --workspace` exit 0, with 162 result lines all `ok`, 710 passed, 0 failed and 1 ignored (AO-2's
evidence); `check_decision_ids` "59 … all distinct"; `check_doc_headings` "191 … none duplicated";
`check_scratch.py scan` "148 test sources … (2 exempt)" (`shots/16a/final/gates5.log`). **G-1 PASS.**

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
checkpoint     C0–C7 committed; origin/main merged at 9ef9c61 (S11-A protocol 2); the scan's
               "invite" admission 841fc9d; main (#77) merged 6a2eafb; G-1 green there (E16a-7);
               PR #79 open READY FOR OPERATOR REVIEW
next action    none — wait for the operator's review; do not merge
background     none
notes          Bob/door (R-16a-2): the ray clears Bob by 67 mm aiming at Alice's head from the door
               (E16a-4, again on 9ef9c61). 12c merged: kick, throw and shove are now declared action types; the scan sees them.
               Coordinator 2026-10-08: check port 7979 before each connected run, kill only own
               PIDs, a run on a foreign world is INCONCLUSIVE; decorative townspeople hidden when
               connected (QS14-9) is outside 16a's frozen files (streetscape.gd) -> recorded as
               16c's first item; the geometry probe is 12e's (§5), not 16a's
```

---

# 20. PR 16c — the living street: one layout, from the server (full design; DESIGN FROZEN 2026-10-08)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session` — record and rulings in §20.0.
Implementation is authorized by that freeze, under the execution contract in §20.11 and its
precondition (12d merged). Drafted by a planning session on `plan/s14-16c`, from `main @ 9cf8f8e`
(S12 13a merged as #82, 16a as #79, 16b merged), 2026-10-08; then brought to `main @ 77a8717` (IL-a #80,
12d's frozen design #85), which changes no client file and leaves 12d's doorway table as cited in A16c-6.

**Authority.** §20 refines §4.6, §4.7, §5's `--street` row, §13's 16c row and §14's 16c lines. Where they
and §20 differ, §20 governs, and each difference is named with the finding that caused it (§20.2). The
operator's "One world, two views" rule (`overall.md`, 2026-10-08) binds everything here and overrides this
section if they ever disagree.

**Identifiers are placeholders**: findings `A16c-<n>`, decisions `D-16c-<n>`, acceptance lines `L-`, `S-`,
`T-`, `V-`, `G-`, mutations `M16c-<n>`, questions `Q-16c-<n>`, evidence `E16c-<n>`, risks `R-16c-<n>`. No
decision record is proposed (Q-16c-10); if the primary session wants one, it takes a number from S14's
range (`ARC-50` … `ARC-52`) at the freeze.

## 20.0 Freeze record

## DESIGN FROZEN

```text
Design revision     §§20.1–20.9, 20.11 as committed in 60164c1, amended by the rulings below (this
                    commit); §20.7 checkboxes, §20.10 ledger and §20.12 handoff stay live
Approved by         the primary session, 2026-10-08 ("16c (§20) is DESIGN FROZEN 2026-10-08 (primary
                    session)"), relayed by the coordinator to the planning session
Implementation base main after 12d merges (precondition); branch mvp0/pr-16c-living-street; worktree
                    /Users/yuema137/mineworld-worktrees/impl-16c
Execution contract  §20.11
Lifecycle           FROZEN
```

**Rulings (primary session, 2026-10-08).** Q-16c-1 … Q-16c-13 are accepted as recommended, with these
additions and one change:

- **Q-16c-7.** The showcase wording (D-16c-11) is accepted. The operator judges it at the milestone F
  play-test.
- **Q-16c-11.** The shove-rate threshold (S-6, more than one shove per minute on a standing player)
  goes on the milestone F checklist.
- **Q-16c-2.** If 16c's implementation does not start on the day 12d merges, the primary session lands
  the 0.4 m inset hotfix itself. C2 then finds the inset on its base, drops its own copy, and records
  that.
- **Q-16c-8.** C5 stays in 16c. If S11-B is still unmerged when C5 is reached, C5 is split off as
  **16c′** with this same design, rather than waiting. The split is recorded in §20.10, and 16c's PR
  opens with C1–C4 and C6.
- **A16c-14 (changed from Q-16c-12's recommendation).** The door talk going to Bob is a **finding for
  16c to fix or guard**: a check that passes on the wrong target is a defect. So "print only" is
  superseded:
  - `_talk_to` asserts that the ray's target is the person it aimed at. Otherwise the calling check
    fails, naming both people.
  - Where a check aims from a spot at which another figure occludes the person, the probe moves to an
    unobstructed aim before talking, as a player would, and records the move.
  - Guarded by mutation M16c-9 (§20.4).

## 20.1 Identity, base, proposed scope

```text
PR            16c — the living street (S14, third of 16a … 16f; GitHub number assigned at freeze, ruling 7)
base          main after S15 12d merges (hard dependency, D-16c-1). C5 additionally needs S11-B (#83,
              the server's --town) merged. 12e may land before or after (D-16c-1)
branch        mvp0/pr-16c-living-street from main, worktree /Users/yuema137/mineworld-worktrees/impl-16c
              (freeze), held by the implementing session only
audit         §20.2 (main @ 9cf8f8e), re-checked on the base in C1
```

**Scope** — the coordinator's four items, then the rest of §13's 16c row:

| ID | Item | Source |
| --- | --- | --- |
| **SC-1** | **First item: the decorative townspeople are not built when the client is connected.** Standalone is unchanged | 16a's recorded hand-over (§19.9); QS14-9 (accepted as recommended, `overall.md` 2026-10-08); "One world, two views" rule 1 |
| **SC-2** | **Every doorway matches a real door.** Every place the world discloses is bound to the scene through its disclosed doorway landing on a drawn door; a doorway that lands on no drawn door is a failure the probe names, never a silent wall | §4.7; F-S14-21; "One world, two views" rule 2 ("S14 16c's binding of every doorway and every place") |
| **SC-3** | **The florist is bound to its shop** — to whichever place's doorway lands on The Flower Room's door, which after 12d is the world's `store` | F-S14-9; R-12d-2; step-11 SD-D4, SD-D8 |
| **SC-4** | **The 3D layout comes only from the server.** The binding learns places, their identities and their doorways from disclosure; the client keeps no list of the world's place keys (`KEYS`, `PLACE_KEY` go); one declared depiction anchors the scene (D-16c-3). The 3D client reports what it shows in S12 13a's `SHOWN` format, for the parity test (13f/16e) | "One world, two views" rules 1–3; 13a's `SHOWN` (A16c-9) |
| **SC-5** | Figures walk between observations, their pick colliders with them; a person who leaves view walks into the door they used | §4.6; F-S14-4; `HUMAN_REVIEW_QUEUE.md` VIS-3D-GODOT-2 limitation 3 |
| **SC-6** | 16a's T-2 (a ray through a wall) re-designed, because SC-3 ends the florist-as-street case it relied on | A16-8; Q-16a-2 |
| **SC-7** | Standalone mode labelled as a showcase, not as the world | "One world, two views" rule 1, second bullet (Q-16c-7) |

**Non-goals (each belongs elsewhere, and 16c must not start it):**

- the player masking people, objects or disclosed walls; the correction rule; press-through; drawing
  loose objects; **the strict 150 mm geometry probe** (`--world --geometry`, every disclosed wall and
  solid face against a scene collider) — all 12e's (§4.2, §13). 16c's doorway check (L-1) compares
  *points* (a doorway against a drawn door); 12e's probe compares *faces*. 16c makes no claim about walls;
- building places, walls or doors from disclosed geometry, and the Presentation Pack — 16f
  (`overall.md`, "Physics list", decision 3);
- the parity test itself (the two `SHOWN` reports compared) — 16e with S12 13f. 16c only makes the 3D side
  report;
- any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `cognition/`,
  `worlds/`, `tools/cli/src/`, `clients/protocol/**`, `clients/2d/**` (I-S14-2; ruling 4 for the shared
  module). The hosted walking controller is S11-B's `--town` (QS14-10 answered by S11: A16c-7);
- a third enterable building (`VISUAL_SLICE.md` §4): the apartments, workplace and park stay behind
  closed drawn doors;
- buying, names on things (16d).

**Binding constraints carried in.** `VIS-3D-GODOT-1` and `VIS-3D-GODOT-2` must not regress (I-S14-7;
§20.6's guard). The 3D visual defaults are chosen by the primary session and judged by the operator in
play (`overall.md` 2026-10-08); 16c proposes the two visual elements it needs (Q-16c-4 only if option (b),
Q-16c-7) and decides neither.

## 20.2 Re-audit for 16c (`main @ 9cf8f8e`, 2026-10-08)

Every row was read in this session from the file named, or measured (E16c-0). §2 and §19.2 stand; these
add what 16c's file-level plan needs.

| ID | Finding | Evidence | Consequence for 16c |
| --- | --- | --- | --- |
| **A16c-1** | **The binding today.** `KEYS := ["cafe", "street"]` (place keys learned from tags); `PLACE_KEY` maps the slice's three `Area3D` volumes to those keys, the florist to `"street"`; `door_point(key)` puts the café's disclosed doorway **0.2 m** inside the café façade's inner face and the street's 0.2 m outside its outer face; `origin(key) = door_point(key) − to_3d(doorway[key])`; `_learn_passages` learns only the café ↔ street pair | `scripts/slice/slice_link.gd:63–117` | Every other place is undrawn and unknown; the 0.2 m matches today's content, not 12d's (A16c-6) |
| **A16c-2** | **A latent mis-binding from a street start.** On the street, `_learn_passages(obs, place, "street")` takes the *first* listed passage whose `to` is not yet known as the café (`other` is `"cafe"`, and `place_ids` has no café yet). The street lists five passages | `slice_link.gd:106–117`; E16c-0 (five `doorway` lines) | Not reached by the `visitor` seat (it starts in the café, A16-7), reached by `--seat=ivan`. 16c's binding removes it (D-16c-3), and states what a street start does |
| **A16c-3** | **The scene already registers every drawn door.** `SliceTerrace.doors`, a static array of `[sill centre (global), façade yaw, label]`, is appended by every shopfront (`terrace.gd:147`), every house door (`terrace.gd:233`) and the café (`cafe.gd:253`); `--doors` photographs each, and 16a's `_street_watch` prints the nearest drawn door to each disclosed street doorway (never fails). An entry does not say whether the door is enterable except in its label (`" -- enterable"`), nor where its room's inner face is | `terrace.gd:29–32, 143–148, 229–233`; `cafe.gd:253`; `slice_probe.gd:309`; `slice_probe_world.gd:511–557` | The registry is the scene side of SC-2: no new scene authoring is needed, only two data fields per entry (enterable volume id, inner-face depth), D-16c-4 |
| **A16c-4** | **The decorative townspeople.** `SliceStreetscape._people` adds four `NPC`s: two standing west of the café (−9.4, −5.60), (−8.5, −5.95), one seated on the middle terrace table's east chair, one at (19.8, 5.2); called last in `SliceStreetscape.build`, inside `SliceWorld.build`, before `SliceBatch.merge`. `SliceMain._ready` builds the world before `_link()`, but the address is already known from the arguments (`SliceLink.address_from_args`). `SliceBatch` merges only primitive meshes with a `material_override`, not the skinned `Human` bodies. The seated figure stands inside the café's `VoxelGI` bake volume | `scripts/slice/streetscape.gd:17–31, 345–360`; `slice_world.gd:31–52`; `slice_main.gd:49–58, 277–290, 372–375`; `batch.gd:15–30` | Not building them is decided at build time from the arguments, by a parameter; nothing is hidden after the fact. Connected, the GI bake loses one small occluder on the terrace (V-3) |
| **A16c-5** | **Place volumes.** Three `Area3D`s: the café (its inner room), the florist (`SliceShopInterior.room_box`), the street (the whole street box); `place_at` returns the innermost. The florist's constants and comments say "`social-cafe` models no florist" | `slice_world.gd:17–23, 175–232`; `shop_interior.gd:32, 100` | The volumes stay (they are where the body is in *this scene*, which is presentation); what changes is which world place each depicts — learned, not listed (D-16c-3). The stale comments are corrected with the code |
| **A16c-6** | **12d's doorways land exactly on drawn doors, with the 0.4 m inset.** 12d's frozen geometry (step-11 §19.3.1, `DESIGN FROZEN 2026-10-08`, merged as docs in #85) moves every doorway 400 mm inside its floor: café `here` (1 610, 400) ↔ street `there` (0, 2 800); store (1 040, 400) ↔ (8 390, 2 800); apartments (2 000, 400) ↔ (−19 700, 2 800); workplace (1 500, −400) ↔ (−5 700, −12 600); park (1 000, 14 600) ↔ (15 300, −12 600). With the café's street point bound 0.4 m outside the north façade, the street's origin is scene (3.45, −4.90) — the same as today — and the four other street points land, by hand computation, at (11.84, −7.70), (−16.25, −7.70), (−2.25, 7.70), (18.75, 7.70): **0 mm** from the out-points (sill + 0.4 m toward the street) of The Flower Room's door, the Flats' house door, the house door at −2.25 south and the house door at 18.75 south. The café's origin is unchanged too (1.84, −8.44): people are drawn where they are today, and the scene points 16a's probe walks to (the counter (8.16, −10.20), the door line) stay valid — the concern step-11's refreshed F-D12 raises is met by the inset, not by editing the probe | step-11 §19.3.1 (lines 5907–5928 on 77a8717), SD-D3, SD-D4, SD-D8; `street.gd:31–38`; `cafe.gd:26–38`; `terrace.gd:143–148, 217–233`; `slice_world.gd:64–172`; E16c-0 (the sills) | SC-2 is satisfiable at 0 mm on 12d's content; a 150 mm tolerance (D-16c-5) is slack for rounding, not for drift. **Without** the 0.4 m inset (today's 0.2), every non-anchor point misses by 200 mm — the mutation M16c-2 |
| **A16c-7** | **Who walks.** `./mineworld-slice --world` starts `mineworld server worlds/social-cafe --listen 127.0.0.1:0 --agent alice`: nobody walks (F-S14-22). S11-B (#83, READY FOR OPERATOR REVIEW, not merged) adds `--town` ("drive every other seat with the paced rule controller whenever no player holds it"), `--seed`, `--pace` (wall seconds, QTW-13) and `--time-scale` | `mineworld-slice:66–94`; `gh pr view 83`; `tools/cli/src/main.rs` on `origin/mvp0/pr-s11b-seats` | R-S11-2 is delivered by S11-B, so QS14-10 is answered (S11) and `tools/cli/src` stays out of 16c. SC-5's evidence needs `--town`: C5 waits for #83 (D-16c-1) |
| **A16c-8** | **16a's T-2 depends on the florist being the street.** `_target_check` walks into The Flower Room and aims at the street's passer-by through the florist's west wall; it fails on purpose if that person is no longer perceived. Once the florist is the store, the observer perceives the store's people, not the street's | `slice_probe_world.gd:210–239`; A16-8 | SC-6: T-2 re-designed (D-16c-9) in the same PR that breaks it |
| **A16c-9** | **S12 13a's `SHOWN` report.** `drive.gd` prints `SHOWN {"place", "places": [{place, drawn_as, tags}], "doorways": [{from, to, here: [x, y], there: [x, y]}], "people": [{id, label, local}]}` — what the 2D client *draws*, places by id, doorways for every place it has learned, people including the observer, positions in the current place's frame, integer millimetres. `drawn_as` is `facade`, `room` or `lawn`. Objects and affordances are not reported yet | `clients/2d/scripts/harness/drive.gd:405–426`; `clients/2d/scripts/scene/places.gd:33, 172–230`; `clients/2d/scripts/scene/people.gd:96–108` | The 3D side reports the same keys with the same meanings (D-16c-7); 13f/16e add objects and affordances to both at once |
| **A16c-10** | **Figures jump.** `_on_observed` sets each figure's `global_position` and yaw on every observation. `NPC.step(delta, speed)` sets the gait from a ground speed (the player drives its body this way); a `STAND` figure also drifts its heading slowly in `_process`. `Player.JOG_SPEED` = 3.10 m/s | `slice_link.gd:305–330, 365–385`; `npc.gd:161–184`; `player.gd:18–19` | SC-5 moves figures by `step`, not by a new animation (§8.6); figure handling leaves `slice_link.gd` (A16c-11) |
| **A16c-11** | **File sizes.** `slice_link.gd` 541 lines (past the ~500 review trigger), `slice_probe_world.gd` 598, `slice_probe.gd` 1 314 | `wc -l` | New code goes into new files with one job each (D-16c-2, D-16c-6, D-16c-8); `slice_link.gd` should shrink, not grow |
| **A16c-12** | **The standalone HUD already says "offline"** — `world: offline  (./mineworld-slice --server=host:port)` — and is not attached in scripted standalone modes, so `--shots` frames carry no HUD | `slice_main.gd:357–362` | SC-7 is a wording change on an existing line that no accepted review frame contains (Q-16c-7) |
| **A16c-13** | **The no-rule scan** reads every client `*.gd` (probes included) for action-type literals, rule-named declarations (`REACH`, `RANGE`, `CLEARANCE`, `NUDGE`, `CAPACITY`, `MAX_STRIDE`) and pack paths | `tests/acceptance/tests/client_rules.rs` | 16c's constants must not need an admission: no 16c name contains those words (D-16c-5, D-16c-8); a stale admission fails the scan |
| **A16c-14** | **On main, `--link`'s door talk went to Bob.** In E16c-0, after walking back in, `_talk_to` aimed at Alice's head from (3.44, −9.58); the ray met Bob, the world answered `too_far_away` for **Bob**, and `--link` passed, because it checks the answer's code only. 16a's T-1, which aims from the seat (3.45, −9.04), asserts the target is Alice (E16a-4: Bob cleared by 67 mm) | E16c-0 (`the world says talking to Bob Achterberg is unavailable`); `slice_probe_world.gd:385–389, 562–583` | Nothing 16c changes moves Bob (A16c-6), but **a check that passes on the wrong target is a defect** (§20.0). 16c fixes and guards it: `_talk_to` fails a check whose ray met someone other than the person aimed at, and aims from an unobstructed point. Guarded by M16c-9 |

**E16c-0 — the doorways on today's content** (`./mineworld-slice --world --link --watch=5` on
`9cf8f8e`, headless, 2026-10-08; server on `127.0.0.1:51486`, this worktree's own; log
`clients/3d-spike/shots/16c-plan/link-main.log`, ignored): `all link checks pass`; 50 moves, 50 accepted;
the street's five disclosed doorways, nearest drawn door: café 0.20 m (the 0.2 m inset itself), workplace
0.30 m (a house door), apartments 3.40 m (Maple & Co.), park 3.29 m (Lakeside Deli), store 3.62 m (The
Flower Room) — `HUMAN_REVIEW_QUEUE.md`'s limitation 2, re-measured. These are the "before" of L-1 and the
values L-5's counterfactual must reproduce.

## 20.3 Design decisions (D-16c-*)

| ID | Decision | Alternatives considered | Why |
| --- | --- | --- | --- |
| **D-16c-1** | **Order.** 16c's implementation starts on `main` **after 12d merges**; C5 (walking figures) needs **S11-B (#83) merged** as well. 16c and 12e are independent and may land in either order: whichever first connects to a 12d world carries the 0.4 m inset (step-11 SD-D8); the second rebases and drops it. If S11-B is late, C1–C4 land and C5 waits (or splits off, Q-16c-8) | land before 12d against today's content (the binding would be designed for doorways 12d is about to move; L-1 would have to fail on four places or be relaxed); wait for 12e too | Every claim of SC-2 and SC-3 is about 12d's doorways, read from disclosure: before 12d they are false by 3.3–3.6 m (E16c-0) and the florist cannot be the store (the store has no floor there). QD-11 / the coordinator: 16c reads doorways from disclosure, never from literals, so 12d's exact numbers are not 16c's dependency — their **arrival** is |
| **D-16c-2** | **One layout module.** New `scripts/slice/slice_layout.gd` (`SliceLayout`, `RefCounted`) owns the binding: per place id — its passages as disclosed, its tags when the observer stood there, its scene origin, its depiction (`room` with a scene volume, `door` behind a closed drawn door, `street`), its bound door; plus `to_scene`, `to_world`, `place_of_volume`, `unbound()` and `shown()` (D-16c-7). `SliceLink` keeps the connection, the reporting rule and the HUD lines and asks the layout; `origin`, `door_point`, `_learn_passages`, `KEYS` and `PLACE_KEY` leave `slice_link.gd` | grow `slice_link.gd` (541 lines, A16c-11); put the gluing in the shared module, as 2D's `town.gd` does for itself (ruling 4: no other PR edits the module) | One job per file (§9.2). The 3D gluing is the same translation-only rule 13a's `town.gd` applies (`origin(P) = origin(Q) + here − there`, `ARC-45`), except that a 3D binding places each side of a doorway on its own side of a drawn wall; sharing the code is a later, coordinated module change (R-16c-7) |
| **D-16c-3** | **One anchor, then doorways.** The scene declares exactly one depiction: **the café room depicts the place tagged `cafe`** (`SliceWorld.ANCHOR_TAG`), as 13a's presentation selects a place's art by its tag. The café's own passage binds the café (its `here` on the café door's in-point) and the place it leads to (its `there` on the door's out-point) — the street, whatever its tags. Every passage the street discloses then binds its place by **doorway coincidence**: the drawn door whose out-point lies within `DOOR_MATCH` of the doorway's scene point; an enterable door binds its room volume to that place (the florist → the store, SC-3), a closed door binds the place as `door`. No place key or tag other than `cafe` is named in code. A seat that starts outside the café binds nothing until it stands in the café, and says so on the HUD (A16c-2's mis-binding is gone) | keep `KEYS`/`PLACE_KEY` (a copy of world content: the "fake town" rule 1 forbids); register by fitting all street doorways to all drawn doors by one translation (works from the street, ambiguous from the café's single doorway — two enterable rooms — and premature before 16f's Presentation Pack); bind the florist by the tag `store` (a second copy of content; the door is the fact that makes it the store's room) | The minimum the scene must assert about a world is which room is which place *once*; everything else follows from what the server discloses, and a disagreement is detected instead of drawn. Q-16c-3 |
| **D-16c-4** | **The drawn-door registry gains two data fields**, appended by the code that builds each door: the enterable room's place volume id (`""` for a closed door) and the depth from the sill to the room's inner face (the café: `SliceCafe.WALL_T`; the florist: the face of `SliceShopInterior.room_box` nearest the street; a closed door: the façade skin's depth, nothing drawn behind it). `DOOR_INSET = 0.40` m: a door's out-point is the sill plus 0.40 m along the façade's outward normal (its yaw); its in-point is the sill minus (inner-face depth + 0.40 m) | a second, hand-written table of doors (a duplicate of the scene that can drift from it); the labels' `" -- enterable"` suffix parsed | The scene already registers every door it draws (A16c-3); adding facts at the point of construction keeps one source. 0.40 m is the authoring rule 12d adopts for every doorway (SD-D3) — the client's art convention matching the world's, checked by L-1 (M16c-2), not a rule the client enforces |
| **D-16c-5** | **`DOOR_MATCH = 0.15` m**, horizontal, between a disclosed doorway's scene point and a drawn door's out-point (street side) or in-point (room side) | 0.30 m; the door's half-width | A doorway point must stand in the door's opening with a body's room to spare: the narrowest drawn door (the café's, 0.98 m) leaves 0.49 − 0.30 = 0.19 m either side of its centre line. 150 mm is under that and is the tolerance the project already uses for scene-versus-world geometry (§4.2), so "within tolerance" means the same thing in both probes. 12d's content lands at 0 mm (A16c-6). The name contains none of the scan's rule words (A16c-13) |
| **D-16c-6** | **Decoratives off by a build parameter.** `SliceWorld.build(parent, connected := false)` passes `with_people := not connected` to `SliceStreetscape.build`; `SliceMain._ready` computes `connected` from `SliceLink.address_from_args()` before building. Standalone builds exactly as today | build them and hide them when connected (they would stay in the GI bake and the collider-free scene would still hold people the world does not have); remove them altogether (changes the accepted standalone slice and its frames: `VISUAL_SLICE.md` §4's "at least two other figures") | Decided before anything is built, so the connected scene never contains them; standalone is byte-for-byte the same build (V-1) |
| **D-16c-7** | **The 3D `SHOWN` report**, from `SliceLayout.shown(link)` and printed by the connected probe modes as `SHOWN <json>`, in 13a's keys and meanings (A16c-9): `place` (the observer's place id); `places`: every **bound** place `{place, drawn_as, tags}` (`drawn_as` `room` \| `door` \| `street`; `tags` as the world listed them where the observer stood, else `[]`); `doorways`: every passage of every place the observer has stood in, **only if bound to a drawn door**, `{from, to, here: [x, y], there: [x, y]}` exactly as disclosed; `people`: every drawn figure and the observer, `{id, label, local}` with `local` read back from the **drawn** position through `to_world` (integer mm, `z` 0). An unbound doorway or place is absent — that is the point: what is not drawn is not reported | report the disclosure (would hide an omission, the mutation 13f/16e needs to bite); a new format (13f would need two readers) | "One world, two views" rule 3 compares what each client *shows*. Reading back from the drawn state makes the report an instrument of the scene, not a copy of the frame (`ARC-23`). `drawn_as` and labels are declared presentation-only for the comparison (Q-16c-9) |
| **D-16c-8** | **Figures walk** (§4.6, §8.6 adopted): new `scripts/slice/figures.gd` (`SliceFigures`, a `Node` owned by the link) builds and moves perceived figures (moved from `SliceLink._figure`). On each observation a figure's target is its observed position; each frame it walks toward the target at the observed gap's own speed, capped at `Player.JOG_SPEED` (3.10 m/s), gait by `NPC.step`, facing its motion, and the observed facing when it arrives. A figure first seen, or more than `PLACE_GAP = 4.0` m from its target, is **placed**, not walked. A figure that leaves view walks to the in- or out-point of the bound door nearest its last observed position, if one lies within `DOOR_WALK = 2.5` m, and is then removed; otherwise it is removed at once. Its pick collider is its child and moves with it | a snapshot-interpolation buffer (§8.6: latency for every figure); `NavigationAgent3D` (a navmesh would decide where people can walk, §8.6); exact exits from `person-entered-place` events (needs S11-C's per-observation events; the nearest door is the fallback until then) | §8.6's verdict. Interpolation is presentation: it never changes what is reported or targeted beyond following the drawn figure. `DOOR_WALK` is longer than any crossing stride, so every crossing person is walked into a door; it names none of the scan's rule words |
| **D-16c-9** | **T-2 re-designed: a wall inside one place.** The occluder must stand between the player and a *perceived* person in the **same** place (F-S14-17). C1 audits the street for a reachable spot where a scene collider at least 1.9 m high stands between the eye (1.66 m) and the passer-by's head: first candidate, the nook east of the retaining wall (scene x 29.5 … 34.0, z −8.1 … −5.9) aiming west at the passer-by. If C1 finds no such spot reachable on foot, the probe plants one `StaticBody3D` box 3 m × 3 m × 0.2 m on `LAYER_WORLD` across the line of sight on the pavement, and removes it after. Either way the pass condition is 16a's: the ray targets nobody, the first hit is the occluder, and the pre-16a cone would have chosen the person | keep T-2 in the florist (vacuous once it is the store: the street's people are not perceived); aim through the counter only (already T-1's second case, low) | Same discriminating claim as 16a's T-2 (walls occlude; the cone would not have), on geometry that stays valid with 12d. Choosing the spot by an audit step before any T-2 run keeps the criterion fixed before measuring (Q-16c-5) |
| **D-16c-10** | **Who walks in which run.** The launcher's interactive `--world` (no scripted mode) and `--street` host the world with S11-B's `--town` beside `--agent alice`, so the operator plays a living street; `--link`, `--conversation`, `--target` and `--layout` host it **without** `--town`, so their accepted evidence keeps comparing a still world | `--town` everywhere (the accepted checks would measure moving people: counter talk, 50/50 moves, positions); never (no living street) | The accepted checks stay comparable before/after (I-S14-7); the product is the living street (Q-16c-6) |
| **D-16c-11** | **Showcase label** (SC-7): the standalone HUD's `world: offline …` line becomes `world: offline — a showcase, not the world  (./mineworld-slice --world to play the world)`. Nothing else in standalone changes | a toast at start; a watermark on screen (changes every standalone frame, including the accepted review frames) | The existing line is the one standalone place that speaks about the world (A16c-12), and no accepted frame contains it (Q-16c-7: wording is a visual default) |

## 20.4 Acceptance (decided before measuring, `ARC-23`)

Every check is run by the real client, against the real server where connected, as the operator runs it
(`ENGINEERING_RULES.md` §19). "Before" is the base (main after 12d, before C2), measured in C1 by the same
commands; "after" is the PR's head. Every number that moves is reported with both values.

**The layout — `./mineworld-slice --world --layout`** (new connected mode, headless, still world):

```text
L-1  EVERY DOORWAY ON A REAL DOOR. Seated in the café; out through its door onto the pavement on foot.
     On the street, for every passage the street discloses: its scene point, the drawn door it is bound
     to, the distance. PASS iff every passage is bound to a distinct drawn door within DOOR_MATCH
     (0.15 m), the café's passage to the café's door, and exactly one passage to The Flower Room's door.
     Expected on 12d's content: five bound, each at 0.00 m (A16c-6). Each miss is printed as
     "doorway to place <id> at street (x, y) -> scene (…): no drawn door within 0.15 m; nearest <label>
     at <d> m" and fails the mode
L-2  THE FLORIST IS ITS SHOP. From the pavement on foot into The Flower Room, to the middle of its floor
     and back out. PASS iff: the server's place, as observed, changes street -> P -> street, where P is
     the place L-1 bound to The Flower Room's door; every move on the way in, inside and out is
     accepted (none refused, none answered other than accepted); the server's last position inside
     equals the last report, millimetre for millimetre (as --link's check); at least one person is
     perceived inside P. P's tags are printed (expected to include "store"; printed, not asserted:
     the probe names no world content)
L-3  WHAT IS SHOWN. A SHOWN line in the café, on the street and inside P (D-16c-7). PASS iff, in each:
     every place is listed once; every doorway equals, field for field, a passage the world disclosed
     in a place the observer stood in, and every such passage is listed; every person the latest
     observation lists in the observer's place is listed once, plus the observer; every id is a string;
     no label equals an id; every local is integer millimetres in the observer's place frame and, once
     the still world has settled, within 50 mm of the position the observation states. After L-2
     the report lists six places (on social-cafe and market-town after 12d) and the doorways of the
     café, the street and P
L-4  ONLY THE WORLD'S PEOPLE. In the café and on the street: every NPC node in the scene other than the
     player's own body carries an entity_id that the latest observation lists in the observer's place.
     PASS iff the count of others is 0 (on main today it is 4 on the street: A16c-4)
L-5  THE CHECK SEES DRIFT (a counterfactual on real content, not a mutation). The pre-12d social-cafe,
     exported from the base's parent commit (git show <pre-12d>:worlds/social-cafe/… into a scratch
     directory), served by the same binary on a free port, the slice joined with --server= --layout:
     L-1 FAILS naming the store, the apartments, the park and the workplace, with distances within
     0.05 m of the hand computation for the 0.4 m binding (store 3.61, apartments 3.39, park 3.34,
     workplace 0.67 m); L-2 does not run (no florist binding), and says so
```

**The living street — `./mineworld-slice --world --street`** (new connected mode, headless, server with
`--town`; after S11-B):

```text
S-1  PEOPLE WALK (activity first; a precondition, not a claim). From a fixed spot on the pavement, 60 s.
     At least three distinct people perceived on the street, each with an observed position change of
     at least 1 m. Fewer: INCONCLUSIVE, re-run once with 120 s; still fewer: reported as a finding about
     the hosted town (S11-B), not a 16c pass or fail, and S-2 … S-4 are not claimed
S-2  NO JUMPS. Every frame, every figure's drawn horizontal displacement divided by the frame time is at
     most 3.10 m/s x 1.05, except a placement (first sight, or a gap over PLACE_GAP); every placement is
     listed with its gap. PASS iff no frame exceeds the bound outside a placement, and placements after
     first sight are at most 2 in the run
S-3  INTO DOORS, NOT WALLS. Every figure that leaves view during the watch is removed at a point within
     0.15 m of the in- or out-point of a drawn door bound by L-1's rule. PASS iff every leave is at a
     door and at least one leave happened (none: INCONCLUSIVE, re-run once)
S-4  NOBODY IN A WALL. Sampled at 1 Hz: a capsule query (r 0.30, h 1.72) at each drawn figure against
     LAYER_WORLD. PASS iff no overlap deeper than 0.05 m; each overlap printed with the figure, the
     collider and the coordinates. (A failure caused by the scene disagreeing with the world's solids
     is classified as 12e's geometry finding and reported; one caused by the walk itself fails 16c)
S-5  SHOWN at the end of the watch, as L-3
S-6  SHOVES, MEASURED (R-S14-5, QS14-15; reported, never failed by 16c). The server is started with
     --save into a scratch directory; after the run, `mineworld inspect` counts the shoves whose target
     is the visitor, per minute of the watch. Over 1 per minute goes to the primary session as QS14-15
```

**Targeting, re-designed — `./mineworld-slice --world --target`** (16a's mode):

```text
T-1, T-3, T-4  unchanged from §19.3, and must pass as on the base
T-2' A WALL WITHIN ONE PLACE (D-16c-9). At the spot C1 chose (or across the planted box), aimed at the
     passer-by's head: the target is nobody; the first collider the ray meets is the occluder (named);
     the pre-16a cone, computed from the same camera and figures, would have chosen the passer-by
```

**Mutations, one per guard — each planted in the working tree, run, recorded, reverted** (`git status`
clean of it afterwards; `git grep MUTATION` empty):

| # | Guard | Mutation | Expected red |
| --- | --- | --- | --- |
| M16c-1 | L-1, L-3 ("removing a doorway from one client's scene must make the test fail", rule 3) | the Flats' house door not appended to the registry | L-1 fails naming the apartments' place id and its nearest door; L-3 fails: the apartments' street passage is disclosed and not listed |
| M16c-2 | L-1 (the inset) | `DOOR_INSET = 0.20` | L-1 fails on the four non-anchor doorways, each at 0.20 m (± 0.01) |
| M16c-3 | L-2 (the florist) | the florist volume depicts the street again (today's `PLACE_KEY` line) | L-2 fails: the server's place never becomes P, or a move inside is not accepted, or the positions differ |
| M16c-4 | L-4 | `with_people` true when connected | L-4 fails naming four figures without an entity id |
| M16c-5 | S-2 | figures placed on every observation (the pre-16c rule) | S-2 fails with per-frame speeds far above 3.26 m/s outside placements |
| M16c-6 | S-3 | a leaving figure removed at once | S-3 fails naming each leave's distance from the nearest door |
| M16c-7 | T-2' | the ray's mask without `LAYER_WORLD` | T-2' fails naming the person targeted through the occluder |
| M16c-9 | `_talk_to`'s target guard (A16c-14) | `--link`'s door talk aimed from main's re-entry point (3.44, −9.58), where the ray meets Bob | `--link` fails, naming Bob as the person met and Alice as the person aimed at; with the guard removed as well, `--link` passes on the wrong target (recorded as the counterfactual) |
| M16c-8 | L-3 reads the drawn state, not the frame | one perceived figure's node displaced 1 m east after it is drawn | L-3 fails naming that person, `local` 1 000 mm from the observed position. Counterfactual in the same run: the same report built from the observation instead of the figures passes — so the instrument is shown to read the scene |

## 20.5 Godot checks (every run of the real client)

```text
G-0  PARSE. The launcher rebuilds Godot's class cache when a script is newer; after each commit that
     adds or renames a class (SliceLayout, SliceFigures): `godot --headless --path clients/3d-spike
     --import` and one launch, with no "Parse Error", "SCRIPT ERROR" or "Cannot get class" line in the
     log. A script error fails every mode
G-1  STANDALONE, unchanged (I-S14-7): --drive "all drive checks pass" (engine Jolt); --measure (the
     base's lines, including the pre-existing stature line, E16a-1); --threshold; --character "all
     character checks pass"; --perf (median frame time per viewpoint within 15 % of the base,
     confirmed by one re-run, as Q-16a-3); ./mineworld-3d --drive (the promenade, untouched by 16c,
     run once as a guard on the shared scripts)
G-2  CONNECTED, the accepted checks (still world, D-16c-10): --link "all link checks pass" with the
     every talk's ray target asserted equal to the person aimed at (A16c-14, §20.0); --conversation "conversation on screen, no ids"; --target T-1, T-2',
     T-3, T-4
G-3  NEW: --layout (L-1 … L-4, and L-5 on the exported pre-12d copy); --street (S-1 … S-6), after S11-B
G-4  WINDOWED EVIDENCE (one Godot window at a time; a stalled capture is INCONCLUSIVE and re-run once):
     --doors (every drawn door, base and head, for the record); --street --frames: three frames from
     the pavement of people walking and one of a person going into a door; --conversation's frames
G-5  INTERACTIVE, by the implementing agent before the PR is opened: ./mineworld-slice --world for two
     minutes — walk out, watch the street, enter The Flower Room, come back; nothing in the log but
     the expected [link] lines; the result reported in words, not claimed as acceptance
```

Every connected run uses the launcher's own server (port 0 since S11-A, so no collision with parallel
lanes), records the server's PID and kills only it; a run that joined anything else is INCONCLUSIVE.

## 20.6 The visual-regression guard (`VIS-3D-GODOT-1`, `VIS-3D-GODOT-2`)

```text
V-1  STANDALONE FRAMES, unchanged. --shots at 1600x900, twice on the base (the noise floor per view)
     and once on the head, compared by 16a's tools/frame_diff.gd. PASS iff each view's share of pixels
     differing by more than 8/255 is at most the base's own run-to-run share + 0.5 points. Any view over
     that is looked at side by side, one image at a time, and its cause named; an unexplained visible
     change is a material stop. Expected: identical within noise (D-16c-6 builds the same scene)
V-2  THE CHARACTER. 16c's diff touches none of human.gd, player.gd, npc.gd, character_slot.gd,
     camera_rig.gd, posture.gd or clients/3d-spike/assets/** (a path check on the PR diff); --character
     passes. The route D+ evidence (--headtrace, --sweep) is therefore not re-run (nothing it measures
     can change), and that reasoning is recorded
V-3  CONNECTED FRAMES, changed only as intended. --conversation's three frames, base and head: the
     differing regions (frame_diff's bounding boxes) are each explained by a decorative figure no
     longer drawn, or by the GI bake without the seated figure (A16c-4); the accepted connected
     checklist (names on every person, the door toast, Alice in plain view at the counter, both
     caption lines) holds on the head's frames, looked at
V-4  THE ACCEPTED CHECKLIST, re-walked by script where it can be: VIS-3D-GODOT-2 step 1 (standalone:
     the seated woman is still on the terrace — V-1's terrace view) and step 2 (connected: G-2). The
     operator plays it at 16c's review; their verdict, not the agent's, accepts the connected look
     without the decoratives (QS14-9 was accepted as recommended; the look is judged in play)
```

## 20.7 Commit plan

Each commit tracks implementation, validation and review separately; a planned commit may become several
coherent ones (mapping recorded). Evidence goes into §20.10 as `E16c-<n>`. Every Godot run longer than two
minutes runs in the background, one Godot window at a time, its log under `clients/3d-spike/shots/16c/`
(ignored).

### 16c-C0 — Design (this section) — docs only

- [x] Implementation: §§20.1–20.12, from the audit in §2, §19.2 and §20.2, and E16c-0.
- [x] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`
  (this plan is under `.structured-coding/`, which they do not read — run to show nothing else moved);
  results recorded in §20.10.
- [x] Review: every finding cites a file and line, a branch, or a measurement; every acceptance line
  states its pass condition before anything is run; every guard has a mutation; the non-goals match
  §13's 16c row, the coordinator's four items, 12e's and 16f's scopes and ruling 4. Self-review by the
  drafting session; the primary session's freeze pending.

### 16c-C1 — The baseline on 12d's head, and the T-2' spot

**Goal.** "Before" measured on the base by the commands "after" will use. **Scope.** Ledger only; no
script a scene runs. **Depends on** freeze and 12d merged.

- [ ] Implementation: none in code. Re-audit §20.2 on the base (A16c-1 … 14 still true; 12d's disclosed
  doorways read from a live `--world` run, not from the plan); choose T-2''s spot (D-16c-9) by walking
  the nook in a scripted run and recording reachability and the line's first hit, before any T-2' run;
  export the pre-12d `worlds/social-cafe` for L-5.
- [ ] Validation (E16c-1): G-1 on the base; V-1's two base `--shots` runs; `--world --link`,
  `--conversation`, `--target` on the base — **expected to show 12d's 0.2 m drift** (SD-D8) — recorded as
  measured, not judged; `--doors` frames; `_street_watch`'s five doorway lines on 12d's content.
- [ ] Review: every base result matches its accepted figure, or the difference is recorded with its
  cause before anything changes (a base that fails its own check is a pre-existing defect, reported).

**Commit boundary.** The ledger.

### 16c-C2 — The layout: one anchor, every doorway, the florist (SC-2, SC-3, SC-4)

**Goal.** L-1 … L-3 become true (shown in C3). **Scope.** New `scripts/slice/slice_layout.gd` (D-16c-2,
-3, -5, -7); `scripts/slice/slice_link.gd` (`KEYS`, `PLACE_KEY`, `door_point`, `origin`, `_learn_passages`
replaced by the layout; `to_scene`, `to_world` and the place-of-the-body lookup delegate; the HUD's
"not drawn by this slice" line for an unbound place; comments updated); `scripts/slice/slice_world.gd`
(`ANCHOR_TAG`; the florist's comment; no geometry change); `terrace.gd`, `cafe.gd` (the registry's two
fields, D-16c-4); `shop_interior.gd` only if its room's inner face is not reachable from `room_box`.
**Non-goals:** figures (C5); decoratives (C4); any visible change.

- [ ] Implementation: as scoped; `DOOR_INSET` 0.40 (the SD-D8 change, unless 12e landed it first —
  then rebased away and recorded).
- [ ] Validation (E16c-2): G-0; G-2 (`--link` now passes on 12d's content: 50/50-class acceptance and
  sent = server's; `--conversation`; `--target` T-1, T-3, T-4 — T-2 is expected to fail here, as A16c-8
  says, and is replaced in C3); G-1's `--drive`; the five doorway lines show 0.00 m.
- [ ] Review: no literal names a world place other than `ANCHOR_TAG`; the client still never decides a
  crossing (it reports the body in the place whose volume it is in, as today); `slice_link.gd`'s line
  count fell; `client_rules` green with no new admission.

**Failure cases.** A doorway on 12d's content that misses by more than 0.15 m: 12d's content and the
scene disagree — reported with both values; not "fixed" by moving the scene's door or by widening
`DOOR_MATCH`. If it is a slice door 12d did not intend (QD-7's choices), it is a material stop to the
primary session (it touches 12d's frozen content or the accepted slice). **Commit boundary.** The binding.

### 16c-C3 — The probe: `--layout`, `SHOWN`, and T-2'

**Goal.** L-1 … L-5, T-2', the target guard (A16c-14), with M16c-1 … M16c-3, M16c-7 … M16c-9. **Scope.** `slice_probe_world.gd`
(`layout` mode; `_target_check`'s wall case at C1's spot; `SHOWN` printed by `--link` and `--layout`;
`_talk_to` asserts the ray's target is the person aimed at and fails the calling check otherwise, the
door-talk aim moved to an unobstructed point where needed — A16c-14 as ruled in §20.0, with M16c-9);
`mineworld-slice` (`--layout`, headless; the help text).
If `slice_probe_world.gd` passes ~800 lines, the layout mode goes into a sibling the same way 16a split
the connected modes (bounded, recorded).

- [ ] Implementation: as scoped.
- [ ] Validation (E16c-3): L-1 … L-3 PASS on social-cafe **and** market-town (`--server=` to a hosted
  market-town; same place files, R-12d-4); L-5 counterfactual; T-1 … T-4 with T-2' PASS; M16c-1, -2, -3,
  -7, -8, -9 each red as stated, then reverted; `--link` and `--conversation` pass with every talk's
  target asserted.
- [ ] Review: every assertion's expected value is a literal from the plan or the disclosure, never
  computed by the code under test; the probe names no world content (L-2 prints tags, asserts ids).

**Commit boundary.** The probe mode and the launcher flag.

### 16c-C4 — The decorative townspeople off when connected; the showcase line (SC-1, SC-7)

**Goal.** L-4, V-1, V-3. **Scope.** `slice_world.gd`, `streetscape.gd` (D-16c-6), `slice_main.gd` (the
`connected` argument; the HUD wording if Q-16c-7 is accepted). **Non-goals:** anything else in the
streetscape.

- [ ] Implementation: as scoped.
- [ ] Validation (E16c-4): L-4 PASS (0 others, café and street); M16c-4 red; V-1 (standalone `--shots`
  within noise); V-3 (connected frames: differences only where the decoratives stood, and the terrace
  GI); G-1 `--drive`, `--character`.
- [ ] Review: the standalone build path is unchanged line for line except the default parameter;
  nothing is hidden after building.

**Commit boundary.** The parameter, its two callers, the HUD line.

### 16c-C5 — Figures walk; people go into doors (SC-5) — after S11-B

**Goal.** S-1 … S-6, with M16c-5, M16c-6. **Scope.** New `scripts/slice/figures.gd` (D-16c-8; figure
building moved from `slice_link.gd`); `slice_link.gd` (`_on_observed` hands targets and leaves to it);
`slice_probe_world.gd` (`street` mode; `--frames`); `mineworld-slice` (`--street`; `--town` per
D-16c-10, with S11-B's flag names as merged). **Depends on** S11-B merged.

- [ ] Implementation: as scoped.
- [ ] Validation (E16c-5): S-1 … S-6 on social-cafe; G-2 again (figures now walk to their first
  positions: `--link`'s "inside the room" lines and T-1's aim must be unchanged — figures are placed on
  first sight); M16c-5, M16c-6 red; G-4's `--street --frames`, looked at.
- [ ] Review: no figure is moved except toward an observed position or into a bound door; the pick
  collider stays a child; no constant needs a scan admission; `NPC`'s file is not edited (V-2).

**Failure cases.** S-1 INCONCLUSIVE twice (the hosted town does not walk enough in 120 s): reported to the
primary session with S11-B's `--pace`; 16c does not tune the server. **Commit boundary.** Motion and its
mode.

### 16c-C6 — Status, documents, gates, close

**Scope.** `docs/MVP_STATUS.md` (the 3D column of Place and Movement, the 3D client row, the S14 row);
`docs/HUMAN_REVIEW_QUEUE.md` `VIS-3D-GODOT-2` known limitations 1, 2, 3 — a dated line appended to each,
nothing in the accepted entry edited (as Q-16a-6); `docs/VISUAL_SLICE.md` §4's people row — a dated note
that connected, the figures are the world's people ("One world, two views" rule 1) (Q-16c-10);
`clients/3d-spike/README.md` and the launcher's help (`--layout`, `--street`); this section's ledger.

- [ ] Implementation: as scoped.
- [ ] Validation (E16c-6, the final executable head): G-0 … G-3 once more; V-1 … V-3 on the head; the
  Rust gate — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` (one complete run; `client_rules` must pass), `check_decision_ids`,
  `check_doc_headings`, `check_scratch.py scan`; the path check: no file under `kernel/`, `contracts/`,
  `persistence/`, `server/`, `systems/`, `cognition/`, `worlds/`, `tools/cli/src/`,
  `clients/protocol/`, `clients/2d/` in the diff.
- [ ] Review: every acceptance line has its evidence; deviations recorded; the PR body carries the
  operator's short runnable list (overall memory: milestone handoff):
  `./mineworld-slice --world` — walk out: people walk the street and go into doors, not walls; no
  standing extras; enter The Flower Room: the HUD says you are in the store; standalone
  `./mineworld-slice` still has the terrace and the townspeople, and says it is a showcase.

Then push, open the PR **READY FOR OPERATOR REVIEW**, and stop. Do not merge.

### 16c test ownership

```text
STATIC      Godot's parser on every launch (G-0); cargo fmt/clippy unchanged (no Rust in 16c)
SCAN        client_rules.rs, unchanged: green with no new admission (A16c-13)
REAL RUN    (the project's Gate 2) L-, S-, T-, G- and V- checks: the real client, on Jolt, against the
            real server; the operator's play is the final judge of the living street and of the
            connected look without the decoratives
UNIT        none: every 16c claim is about a scene against a server, and a unit test of the layout
            arithmetic would assert the same numbers L-1 measures end to end (no test-count KPI)
GATE 1      NOT REQUIRED — no language model in 16c
CI          as main has it when 16c runs (S13's workflow if merged); otherwise N/A, and the full local
            gate runs once on the final head
```

## 20.8 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-16c-1** | 12d merges with doorway points other than its frozen §19.3.1 (a bounded deviation in its lane) | 16c reads every doorway from disclosure (D-16c-3); L-1 measures them on the real head; a miss over 0.15 m is reported with both values and goes to the primary session (C2's failure case), never absorbed by moving the scene or widening the tolerance |
| **R-16c-2** | S11-B is late or its flags change | C1–C4 do not need it; C5 waits or splits off (Q-16c-8); the launcher takes S11-B's names as merged |
| **R-16c-3** | 12e and 16c edit `slice_link.gd` at once (figures, colliders, the correction rule) | Figures move to `figures.gd` and the binding to `slice_layout.gd`, so the two PRs meet in few lines; whichever lands second rebases; the inset is carried once (D-16c-1) |
| **R-16c-4** | Between 12d's merge and the inset fix, main's connected slice draws and reports 0.2 m off (SD-D8) and `--link` may fail on main | Q-16c-2: land the inset as the first thing after 12d — in C2, or as a one-constant hotfix if 16c's freeze is not ready the same day |
| **R-16c-5** | Paced people shove the standing player often (R-S14-5); before 12e the client never adopts an accepted correction, so the drawn body drifts from the server's | S-6 measures shoves; `--street` keeps the player still and reports nothing, so its claims do not depend on corrections; interactive drift before 12e is stated as a known limitation in the PR |
| **R-16c-6** | A walking figure cuts a corner through a scene prop between two observations | Strides are short at 10 Hz; S-4 samples overlaps and classifies their cause; nothing is pathfound in the client (§8.6) |
| **R-16c-7** | The 3D layout and 13a's `town.gd` implement the same translation rule twice | Both follow `ARC-45`'s one rule, and the parity test (13f/16e) compares their reports; moving the gluing into the shared module is a later coordinated change under ruling 4, raised for the primary session, not done here |
| **R-16c-8** | The connected look without the decoratives reads as empty in still-world runs | Interactive play hosts `--town` (D-16c-10); the operator judges the look in play (QS14-9) |
| **R-16c-9** | Another lane's Godot window covers a capture; a run joins a foreign server | One window at a time; stalled captures INCONCLUSIVE and re-run once; port 0 and own-PID kill (§20.5) |

## 20.9 Questions for the freeze (primary session; **[OM]** = the operator's)

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-16c-1** | **Order** (D-16c-1): 16c after 12d (hard); C5 after S11-B; 12e in either order, the first to connect to a 12d world carrying the 0.4 m inset | **Yes, as D-16c-1.** The coordinator's expectation ("probably after 12d, for the door positions") is right, and for a stronger reason: before 12d the florist cannot be the store at all |
| **Q-16c-2** | **The inset's timing** (R-16c-4): carry it in 16c-C2, or land it as a one-constant hotfix (`door_point`'s 0.2 → 0.4, plus `--link` re-run) right after 12d merges? | **Hotfix, if 16c is not frozen and implementing on the day 12d merges**; otherwise C2. Main's connected slice should not stay 0.2 m off |
| **Q-16c-3** | **The anchor** (D-16c-3): one declared depiction by the tag `cafe`, every other place by doorway coincidence; a seat starting outside the café binds nothing until it stands in the café | **As D-16c-3.** Registration by fitting all doorways is 16f's natural tool once a Presentation Pack declares its rooms; doing it now is premature, and from the café's single doorway it is ambiguous |
| **Q-16c-4** | **A doorway that lands on no drawn door, while playing** (not in the probe, where it fails): (a) say it on the HUD and log it, draw nothing; (b) also draw a plain marker (a dark door-sized slab) at the doorway's scene point, so nothing the world has is omitted | **(a) in 16c**, because the probe guarantees no such doorway in the shipped worlds (L-1, L-5), and 16f draws places from disclosure generally. **[OM: visual default] only if (b) is chosen** |
| **Q-16c-5** | **T-2''s occluder** (D-16c-9): real geometry (the retaining-wall nook) if C1 finds it reachable, else a probe-planted box | **As D-16c-9**, chosen in C1 before any T-2' run |
| **Q-16c-6** | **Who walks in which run** (D-16c-10): interactive `--world` and `--street` with `--town`; the accepted scripted checks without | **As D-16c-10** |
| **Q-16c-7 [OM: visual default, judged in play]** | **The showcase label** (D-16c-11): reword the standalone HUD line; or a start-up toast; or nothing | **Reword the line**: no accepted frame contains it, and it is the one place standalone already speaks about the world |
| **Q-16c-8** | **Motion in 16c or split** (§13's row has it; the coordinator's four items do not name it) | **Keep it as C5** ("the living street" is its point, §4.6); if S11-B is not merged when C4 is done, open the PR with C1–C4 and move C5 to a follow-up PR with the same design, recorded as a bounded split |
| **Q-16c-9** | **`SHOWN` format** (D-16c-7): 13a's keys exactly; `drawn_as` 3D values `room`, `door`, `street`; `drawn_as` and labels declared presentation-only for the parity test; objects and affordances added to both clients together later | **As D-16c-7**; 13f/16e owns the comparison and its presentation-only list |
| **Q-16c-10** | **Documents**: no `DECISIONS.md` record (the binding is client-internal presentation, specified here; 16f records the general rule from `ARC-50`…`52`); dated lines appended to `VIS-3D-GODOT-2` limitations 1–3; a dated note on `VISUAL_SLICE.md` §4's people row | **Accept.** A record now would be superseded by 16f's within weeks; the notes keep the accepted specs true without editing what the operator accepted |
| **Q-16c-11 [OM: judged in play]** | **Shoves on a standing player** (S-6; QS14-15's remedy) | **Measure in C5**; over 1 per minute, the primary session applies QS14-15's recommendation (12c's offer policy p1, in the pack), never the client |
| **Q-16c-12** | **`--link`'s door talk** went to Bob on main (A16c-14). Print the target the ray chose in `_talk_to` (no new fail), or make `--link` fail unless the ray meets the person aimed at? | Drafted recommendation: print only. **Ruled at the freeze (§20.0): fix and guard** — `_talk_to` fails on a wrong target, the probe aims from an unobstructed point, and M16c-9 guards it |
| **Q-16c-13** | **The registry's two fields and `DOOR_MATCH` 0.15 m** (D-16c-4, D-16c-5) | **Accept** — bounded, recorded |

## 20.10 Ledger (live)

```text
E16c-0  §20.2, on main @ 9cf8f8e: today's doorways measured (link-main.log)
C0      on main @ 77a8717 + this section: check_doc_headings "191 numbered sections across 26
        documents, none duplicated"; check_decision_ids "69 decision ids, all distinct". §20's
        headings 20.1–20.12 are unique within this file
FREEZE  2026-10-08: §20.0 record and rulings, §20.11 confirmed; A16c-14 ruled fix-and-guard (M16c-9).
        Doc checks again: "191 … none duplicated"; "69 … all distinct"
```

## 20.11 Execution contract (confirmed at the freeze, 2026-10-08)

```text
PROJECT / PR        MVP-0 · S14 / PR 16c — the living street
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-15-demo-3d.md §20 (this section)
RELATED / BINDING   overall.md §3 (S14), "Parallel build-out, 2026-10-08" (rulings 4, 6, 7, 9), "One world,
                    two views", "Physics list" decision 3 (16f's scope); this file §§1–17 as frozen at step
                    level, §19 (16a); step-11 §19 (12d, as merged), SD-D3, SD-D4, SD-D8, QD-7, QD-11;
                    step-12 §16 (S11-B, as merged); step-13 13a's SHOWN; ENGINEERING_RULES §§3–12, 19;
                    VISUAL_SLICE §4; HUMAN_REVIEW_QUEUE (VIS-3D-GODOT-1/-2)
PRECONDITION        12d merged on main. S11-B (#83) is needed for C5 only; if it is unmerged when C5 is
                    reached, C5 splits off as 16c′ (§20.0, Q-16c-8)
IMPLEMENTATION BASE main after 12d; branch mvp0/pr-16c-living-street from main; worktree
                    /Users/yuema137/mineworld-worktrees/impl-16c (this session only)
APPROVED SCOPE      §20.1 (SC-1 … SC-7), as answered by Q-16c-1 … 13 and the rulings in §20.0
                    (including A16c-14: fix and guard)
FROZEN INVARIANTS   I-S14-1 (the scan green with no new admission), I-S14-2, I-S14-3, I-S14-6, I-S14-7,
                    I-S14-10; no edit to clients/protocol/**, clients/2d/**, worlds/**, tools/cli/src/**;
                    no regression of VIS-3D-GODOT-1 or -2 (§20.6); DOOR_MATCH never widened after a
                    measurement
SEQUENCE            C0 → (freeze) → C1 → C2 → C3 → C4 → C5 (after S11-B, else 16c′) → C6, each committed
                    and pushed when coherent
COMMANDS            as in 16a's contract (§19.8): cargo fmt / check / clippy -D warnings / test
                    ($HOME/.cargo/bin/cargo if needed); python3 scripts/check_doc_headings.py,
                    check_decision_ids.py, check_scratch.py; godot --headless --path clients/3d-spike …
                    and the tools under clients/3d-spike/tools/; ./mineworld-slice and ./mineworld-3d
                    modes; git and gh (no merge); one Godot window at a time; only this run's own server
                    process is killed
VALIDATION BUDGET   real client runs: each ≤ ~10 min, background when > 2 min, one Godot window at a time,
                    total ≤ ~3 h of wall time including retries; full cargo test: once, on the final head;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  this section (§20.7 checkboxes, §20.10 ledger)
HANDOFF             §20.12 (one authority, as Q-16a-4)
ENDPOINT AUTHORITY
  implementation + local validation   authorized by the freeze (§20.0), once the precondition holds
  semantic commits, branch push       authorized, as for 16a
  PR creation / update                authorized, READY FOR OPERATOR REVIEW
  CI repair                           only if S13's workflow is on main; otherwise N/A
  merge                               explicit operator authorization only
POST-MERGE SYNC     the primary session owns §13 and overall; the implementing session owns §20
NORMAL STOP         PR 16c READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any regression of the accepted visuals (V-1 … V-4); any server, kernel, contract or
                    shared-module (clients/protocol/**) edit; a doorway of 12d's content off its drawn
                    door by more than DOOR_MATCH that is not a client defect; any other change needed
                    outside the scope above (worlds, 2D, tools/cli/src); a change to a frozen invariant
```

## 20.12 Handoff (live)

```text
checkpoint     C0 drafted on plan/s14-16c (docs only, PR #90); DESIGN FROZEN 2026-10-08 (§20.0)
next action    a fresh implementation session in /Users/yuema137/mineworld-worktrees/impl-16c on
               mvp0/pr-16c-living-street, once 12d has merged: C1. C5 needs S11-B, else 16c′
background     none
notes          E16c-0's log is in the planning worktree's ignored shots/16c-plan/; the implementing
               session re-measures on its own base (C1)
```

---

# 22. PR 16d — things you can buy, hand over, eat and drink, in Market Town, in 3D (full design)

**Lifecycle: `DESIGN FROZEN (2026-10-09), primary session`** (record and rulings in §22.0). Drafted by the
16d planning session on `plan/s14-16d` from `main @ f80bbb7` (2026-10-09), worktree
`/Users/yuema137/mineworld-worktrees/plan-16d`. Implementation is authorized only under §22.11, once its
precondition holds, in a fresh session (`CLAUDE.md` §3.1).

**Why §22.** §21 is left free for S15 12e's PR design, which another session is drafting in parallel on
`plan/s15-12e` in this same file. Ruled at the freeze: §22 stays; whichever of 12e and 16d merges second
renumbers if needed. Nothing inside refers to its own number except headings.

## 22.0 Freeze record

## DESIGN FROZEN

```text
Design revision     §§22.1–22.11 as committed in ca5d92e, amended by the rulings below (this commit);
                    §22.8 checkboxes, §22.12 ledger and §22.13 handoff stay live
Approved by         the primary session, 2026-10-09 ("16d (§22) is DESIGN FROZEN 2026-10-09 (primary
                    session)"), relayed by the coordinator to the planning session
Implementation base main with S15 12d, S12 13b and S20 SET-a merged and the 0.4 m doorway inset on main
                    (precondition); branch mvp0/pr-16d-market-3d; worktree
                    /Users/yuema137/mineworld-worktrees/impl-16d
Execution contract  §22.11
Lifecycle           FROZEN
```

**Rulings (primary session, 2026-10-09):**

- **Accepted as recommended:** Q-16d-1, Q-16d-4, Q-16d-5, Q-16d-7, Q-16d-8, Q-16d-10, Q-16d-11, Q-16d-12.
- **Q-16d-2 (keys E / Tab / B), Q-16d-9 (look and placement), Q-16d-13 (zh_Hans wording):** the
  recommendations are the defaults. The operator judges them hands-on at the milestone play-test through
  OC-1 … OC-8; a change afterwards is a bounded follow-up, not a reopening of this design.
- **Q-16d-3, restated.** The client never decides where buying is allowed: it shows exactly what the
  server offers. That a visitor can buy anywhere in the shop's place is **today's economy rule**
  (`buy_requirement` = at the shop's place, A16d-6), not a client choice, so there is nothing for 16d to
  choose. Recorded as **A16d-17** (below).
- **Q-16d-6: yes.** Group activity in 3D (`invite`, `accept-invitation`, `decline-invitation`,
  `join-group-activity`, `leave-group-activity`, with a typed line) becomes the follow-up **PR 16g**,
  designed after 16d. Until then 16d lists those entries disabled, and its PR and `MVP_STATUS.md` state
  the gap.
- **A16d-11 (`action.move`'s 2D-only English in the shared layer):** noted; the primary session tells the
  SET-a lane. D-16d-6's fix stands.
- **Section number:** §22 stays (above).

**A16d-17 — realistic defaults: in the real world you buy at the counter** (the coordinator called this
"A16d-12"; that id was already taken in §22.2, so it is recorded under the next free one). The operator
wants realistic defaults. Where buying is offered is a rule of the world, owned by `economy` and, once
configurable, by the World Interaction List (S17, IL-c onward) — for example `buy` offered only within
reach of a counter part of the shop's place. It is **not** in 16d's scope and never in a client: when the
server starts offering `buy` only at the counter, 16d's menu shows exactly that with no client change
(D-16d-3), and O-1's expected list changes with the offers, not with the code.

**Authority.** §22 refines §4.4's `buy` row, §5's `--world=market-town --buy` row, §13's 16d row and §14's 16d
lines. Where they differ, §22 governs, and each difference names the finding that caused it (§22.2). The
operator's "One world, two views" rule and the "no rule in the client" rule (D-2, `ADOPTION.md` §3.3,
`ARC-47`) override this section if they ever disagree.

**Identifiers are placeholders**: findings `A16d-<n>`, decisions `D-16d-<n>`, acceptance lines `O-` (offers
and menus), `N-` (names), `K-` (what you have), `P-` (parity with 2D, `AC-13`), `G-` (Godot), `V-` (visual),
mutations `M16d-<n>`, operator checks `OC-<n>`, questions `Q-16d-<n>`, evidence `E16d-<n>`, risks
`R-16d-<n>`. No new decision number is proposed (Q-16d-10).

## 22.1 Identity, base, proposed scope

```text
PR            16d — Market Town in 3D: buy, hand over, eat or drink, and what you have (S14; GitHub
              number assigned at freeze, ruling 7)
base          main after S15 12d, S12 13b and S20 SET-a have merged, with the 0.4 m doorway inset on
              main (D-16d-1; §22.11 PRECONDITION)
branch        mvp0/pr-16d-market-3d from main, worktree /Users/yuema137/mineworld-worktrees/impl-16d,
              held by the implementing session only
audit         §22.2 (main @ f80bbb7 plus the named branches), re-checked on the base in C1
```

**Scope:**

| ID | Item | Source |
| --- | --- | --- |
| **SD-1** | **Market Town from the 3D launcher.** `./mineworld-slice --world=market-town` hosts `worlds/market-town` exactly as `--world` hosts `social-cafe` today; `--world` alone is unchanged | §13's 16d row; QS14-5 |
| **SD-2** | **Your menu (B).** Every target-less affordance the latest observation offers — `buy` per priced kind, `eat`/`drink` per kind held, and whatever else a pack offers target-less — one entry each, in the server's order, with the server's verdict and reason; beside it, what you have: money, what you carry, and the shop's listing where you stand | §13's 16d row; the coordinator's brief (buy, eat or drink, inventory); QS14-7 |
| **SD-3** | **The menu of the person you look at (Tab).** Every affordance whose target is the person the camera's ray meets (16a's `SliceTargeting`), in the server's order — `give` per kind held, `talk`, and anything else offered — with verdicts and reasons | the coordinator's brief (hand over); 13b's person menu (D-b-1) |
| **SD-4** | **Complete affordances are submitted unchanged** (`submit_affordance`, D-2); the client composes nothing new. An entry it cannot ask for is listed, disabled, never sent | `ADOPTION.md` §§2, 3.3; 16b SB-5; 13b D-b-2 |
| **SD-5** | **Things are named.** Every item a menu, a panel or a result mentions is shown by the name the world discloses in `item-catalogue` (12d SD-D10); no entity id is ever on screen | F-S14-24, R-F41 (delivered as R-PK-2 by 12d); `SliceLink.display_label`'s rule |
| **SD-6** | **All UI text through translation keys**, in the shared catalog layer SET-a creates, in `en` and `zh_Hans` | ARC-70; step-20 §3.6, SD-SET-a-17 |
| **SD-7** | **The same requests as 2D** (`AC-13`) for buy, drink, give and eat, compared with 13b's real 2D client by the server's own definition | §4.8; QS14-17; the coordinator's brief |

**Non-goals (each belongs elsewhere, and 16d must not start it):**

- kick, throw, shove, colliders, the correction rule, objects drawn — 12e's. If 12e has landed, its
  complete target-less `kick`/`throw` appear in your menu automatically (D-16d-3), and nothing in 16d
  names them;
- walking figures, doorways, the florist as the store — 16c's;
- composing any new incomplete action: `invite`, `accept-invitation`, `decline-invitation`,
  `join-group-activity`, `leave-group-activity` are listed and disabled ("not supported by this
  client") in 3D (Q-16d-6); a typed utterance for `talk` (Q-16d-7);
- the other own panels 13b shows (conversation history, acquaintances, invitations, agenda,
  employment, "other") (Q-16d-8);
- the parity test of what each client *shows* (16e with 13f); 16d emits its `MENU`/`PANEL` reports in
  13b's format so that test has a 3D side;
- any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `cognition/`,
  `worlds/`, `tools/cli/src/`, `clients/protocol/**` (ruling 4), `clients/2d/scripts/**` (13b's);
  a Presentation Pack for 3D (16f).

**Binding constraints carried in.** `VIS-3D-GODOT-1` and `VIS-3D-GODOT-2` must not regress (I-S14-7, §22.7).
Standalone mode gains nothing from 16d. Every platform (macOS, Linux, Windows): no platform branch, and
the hand checks name the OS they run on (§22.9).

## 22.2 Audit anchors (`main @ f80bbb7`, 2026-10-09, and the branches named)

Every row was read in this session from the file named. Rows marked **(branch)** are read from a branch
that is not on `main`; C1 re-reads them on the merged base and records any difference before code.

| ID | Finding | Evidence | Consequence for 16d |
| --- | --- | --- | --- |
| **A16d-1** | **3D composes one interaction.** `SliceIntents` holds `TALK := "talk"`, `DEFAULT_UTTERANCE`, a `VERBS` table, a `_pending` token table and `talk_offered`/`talk_reason`, which read the verdict only to *say* it. No complete affordance is ever submitted by the 3D client | `clients/3d-spike/scripts/slice/intents.gd:1–57` | `intents.gd` gains `submit_offered(affordance, actor_location)` over 16b's `submit_affordance`. No new action literal: a complete affordance carries its own type (A16d-11) |
| **A16d-2** | **Results of anything but talk read as raw codes.** `_on_resolved` shows `"%s: %s" % [what, kind]` — a `buy` would read "buy: accepted", by its action type; `_readable` turns `too_far_away` into "too far away"; E calls `talk_to_facing(DEFAULT_UTTERANCE)` directly, no menu. `slice_link.gd` is 541 lines | `slice_link.gd:472–541`; `wc -l` | Results go through keys (`action.<type>.done`, `ui.result.*`) with names (D-16d-7). New UI goes in new files; `slice_link.gd` gains only the result wording and the hand-over to the menu |
| **A16d-3** | **The 3D HUD's visual standard.** `ControlsHud._style`: white at 0.88 alpha, black shadow 0.75 offset (2, 2); status lines 15 px from (18, 14) stacking by 24 px; toast 30 px centre-bottom; captions 24 px on a `StyleBoxFlat` black 0.45, radius 6, margin 10. `CONTROLS` is one line shared with the promenade | `clients/3d-spike/scripts/controls_hud.gd:16–191` | Menus and panels use exactly this look (D-16d-8); the standalone controls line is not edited, so no accepted standalone frame changes (V-1) |
| **A16d-4** | **The launcher hosts one world.** `--world` hard-codes `worlds/social-cafe --agent alice`, port 0, reads the invite from the join line; `--link`, `--drive`, `--target` force headless | `mineworld-slice:54–103` | `--world=<pack>` (D-16d-2); a new headless mode `--market` |
| **A16d-5** | **The shared module already has everything (16b).** `affordances(type, target)` (all matches, server order; `""` = target-less), `complete_affordances`, `is_complete` (key presence), `affordances_about(id)`, `component_value`, `display_name`, `submit_affordance(affordance, actor_location)` (re-sends whole doubles as integers) | step-15 §18.3 SB-1 … SB-7; `clients/protocol/mineworld/observation.gd`, `world_client.gd` | **No module edit** (ruling 4). The one reader 16d needs that the module lacks — item names — stays client-local (D-16d-5) |
| **A16d-6** | **What Market Town offers the visitor at genesis.** Vera Lindgren (seat `visitor`) at the café door (1 610, 600), holding apple 1, scarf 1, wallet 200 000 minor units. The café is `cafe-company`'s shop: coffee 300, tea 250, croissant 250, cake 400, sandwich 600, soup 500, all in stock. `buy` is complete, target-less, one per priced kind, only in a shop's place, and **offered unavailable when it cannot happen now** ("so a client can still show what is for sale", QS-44). `give {item, count: 1}` is complete per kind held against each present person, ≤ 3 000 mm. `eat`/`drink {item}` complete, target-less, per food/drink kind held, no spatial requirement. Bob (4 500, 6 100) is ≈ 6.0 m from the door, Alice ≈ 8.7 m | `worlds/market-town/people/{visitor,bob,alice}.yaml`; `organizations/cafe-company.yaml`; `systems/economy/src/offer.rs:27–49`, `action.rs:45–57`; `systems/item-transfer/src/offer.rs:42–48`; `systems/consumption/src/offer.rs:44–52`; 13b B-5, B-7 | From the door Bob's two `give` entries are listed **unavailable, too far away** — the natural "chosen regardless" case (O-3). Every buy is affordable at genesis, so an affordability mutation would bite on nothing: O-5 uses a test-time copy with a poor visitor (D-16d-10) |
| **A16d-7** | **Item names arrive with 12d (branch).** `item: { category, name }`; `item` discloses, to whoever perceives a place, `item-catalogue` on the place: every declared kind `{ item, category, name }` in `ItemId` order, built at disclosure, never stored. `coffee.yaml`: `item: { category: drink, name: Coffee }`. 12d is paused for S15 12n | `origin/mvp0/pr-12d-towns` `f218e4f` (TD-C2), `systems/item/src/component.rs:70–110`, `worlds/market-town/items/coffee.yaml`; head `8814aad` "paused for 12n"; step-11 SD-D10 | 16d depends on 12d's merge (D-16d-1). The JSON shape of an entry (`item` as a typed reference or a string) is pinned from a live frame in C1, never from the Rust struct |
| **A16d-8** | **13b's 2D design, as implemented (branch).** `menu.gd`: entries `{action_type, kind, target, about, complete, available, reason, label, enabled, affordance}`; kinds `complete`, `composed`, `point`, `unsupported`, `walk`; verdicts read where entries are built, never where one is chosen; choose closes the menu; a separator where the type changes. `intents.gd`: `COMPOSED`, `submit_offered`, `compose`, pending `{action_type, target, about}` where `about` is the payload's first typed reference `{entity, entity_type}`. `readers.gd`: `thing(id)` → `display_name` else `"item {id}"` (QS13b-3, until R-PK-2); wallet as minor units through `Words.money` (`"%d.%02d"`), holdings `held[{item, count}]`, shop `listed[{item, price, in_stock}]`. The harness prints `MENU`, `PANELS`, `REQUEST` lines; steps are JSON written by the Rust test (`{"open": id \| "self"}`, `{"choose": {"action_type", "about"}}`, `{"choose": {"walk": true}}`) | `origin/mvp0/pr-13b-interactions` `98bd42e`: `clients/2d/scripts/{menu,intents}.gd`, `hud/{readers,words}.gd`, `harness/interact.gd`; `tools/cli/tests/client_2d_interact.rs:181–360` (`item_steps`, `transcript`, `differing_fields`) | 3D builds the **same entry model** and prints the **same report lines** (D-16d-4), and its `AC-13` test re-uses 13b's `item_steps` and harness unchanged (D-16d-11) |
| **A16d-9** | **13b's wording (branch).** `presentation/mineworld-default/2D/i18n/en.po`: `action.<type>` / `.done` for every default action, with `{target}` and `{item}`; `reason.*` for the kernel's closed reasons and `malformed-payload`; `ui.*` (`ui.unsupported`, `ui.unavailable`, `ui.needs-range`, `ui.self`, `ui.menu.empty`, `ui.result.*`, `ui.item`), `panel.*`, `ui.row.*`, `format.money` | that file, 261 lines | The menu and panel keys are the same concepts in 3D; D-16d-6 shares them rather than copying them |
| **A16d-10** | **SET-a (frozen, waits for 13b).** `clients/shared/settings/` with `MineWorldText` (`load_layers`, `code(family, code, args)`, readable fallback); layer 1 (shared) holds `action.*`, `reason.*` moved verbatim from 13b's file (SD-SET-a-17) **and the 3D client's own keys until 16f**; layer 2 is a pack's; a key in two layers needs `#. override` in the later one (AC-SET-4); `client_text.rs` scans 3D text sinks (AC-SET-3) and asserts every key used exists in `en` and `zh_Hans`; 3D's `_say(log_line, shown_key, args, notice)` (SD-SET-a-10); Esc toggles the settings menu in 3D (SD-SET-a-8); an open menu takes gameplay input (AC-SET-12); harness modes run `--settings=none`; `--wording=none` | step-20 §3.6, §12.3, §7 | 16d starts after SET-a (D-16d-1) and puts its keys in layer 1 (D-16d-6); it re-uses SET-a's input gate and Esc order (D-16d-9) |
| **A16d-11** | **`action.move`'s English is 2D-specific.** 13b words `action.move` "Walk (click where to go)"; SET-a moves every `action.*` verbatim into the shared layer, so the 3D client would read "click where to go" | A16d-9; SD-SET-a-17 | D-16d-6: the shared entry becomes neutral ("Walk"); the 2D pack keeps its wording with `#. override`. 2D's screen does not change |
| **A16d-12** | **The no-rule scan admits 3D's `move` (slice_link) and `talk` (intents) only**, and fails an admission that admits nothing. Rule-named declarations (`REACH`, `RANGE`, …) are refused | `tests/acceptance/tests/client_rules.rs:35–80` | 16d needs **no new admission**: complete affordances carry their type; `move`'s entry is recognised by `SliceLink.MOVE_ACTION`, not by a literal (D-16d-3). A 16d constant names none of the scan's rule words |
| **A16d-13** | **The player's input is polled.** `Player` reads WASD with `Input.is_action_pressed`; `ui_cancel` toggles the mouse; a click with the mouse visible recaptures it | `clients/3d-spike/scripts/player.gd:101, 147–159` | A menu must hold gameplay input through SET-a's gate, not by editing `player.gd` (V-2 keeps it untouched); a click on a menu entry is consumed by the GUI before `_unhandled_input` |
| **A16d-14** | **The connected probe is large and shared.** `slice_probe_world.gd` 598 lines (modes `link`, `conversation`, `target`; helpers `_aim_at`, `_report_aim`, `_talk_to`, `_walk_to`, `_answered`); 16c and 12e add modes to it | `wc -l`; `slice_probe_world.gd:18–598`; §20.7, §13 | 16d's mode lives in a sibling, `slice_probe_market.gd`, extending it for its helpers (as 16a split `slice_probe.gd`) |
| **A16d-15** | **13b's Rust harness is reusable as is (branch).** `godot2d::worlds::{hosted, ids, facts, play, menus, panel, step_done, step_line, steps_file, copy_dir}`, `godot2d::{MARKET_TOWN, World, passed, tagged}`; `ids(pack)` resolves genesis ids by key; `facts(save)` reads the save's fact log | `tools/cli/tests/godot2d/worlds.rs:19–141` on the 13b branch | 16d's Rust test drives the 2D client with 13b's own helpers and the 3D client with a small sibling helper, `tools/cli/tests/godot3d/mod.rs` |
| **A16d-16** | **Parallel lanes touching the same client.** 12e (being planned, `plan/s15-12e`) adds kick/throw/shove to `intents.gd`, prompts to `controls_hud.gd`, modes to the probe; 16c (frozen, after 12d) adds `slice_layout.gd`, `figures.gd`, `--layout`/`--street` and `--town` to the launcher; SET-a edits `slice_link.gd`'s `_say`, `slice_main.gd`, `controls_hud.gd`, `intents.gd` (`verb`) | `git worktree list`; §20.7; step-20 §12.4 | 16d's code is in new files; its edits to shared files are a few lines each, named in §22.8; whichever of 12e/16c/16d lands later rebases (R-16d-5) |

No live measurement was taken while planning: every number above is read from source. C1 measures the
base before anything changes (E16d-1).

## 22.3 Design decisions (D-16d-*)

| ID | Decision | Alternatives considered | Why |
| --- | --- | --- | --- |
| **D-16d-1** | **Order.** 16d starts on `main` after **12d** (item names, Market Town's geometry), **13b** (the 2D side of `AC-13`, its wording and harness) and **SET-a** (the shared catalog, Esc, the input gate) have merged, and with `./mineworld-slice --world --link` passing on that `main` (the 0.4 m inset, SD-D8, landed by 16c, 12e or the primary session's hotfix, Q-16c-2). **12e and 16c are not preconditions**: either may land before or after 16d, and the later one rebases | wait for 12e as §13 drew it; start before SET-a with a 3D-local wording file | No 16d claim needs bodies, colliders or walking figures: giving needs the person under the ray (16a), buying needs nothing spatial in the client. A 3D-local wording file would be a second catalog SET-a must migrate. §13's "after 12e" was an ordering convenience, not a dependency (Q-16d-1) |
| **D-16d-2** | **`--world=<pack>`.** The launcher takes an optional pack name; `--world` alone stays `social-cafe`. The name must be a directory `worlds/<pack>` containing `world.yaml`, else the launcher stops with an error naming it; the server is started exactly as today otherwise (`--listen 127.0.0.1:0 --agent alice`, the join line read for address and invite). The "world:" line names the pack | a separate `--market` world flag; a path argument | One flag, one meaning; the pack decides what is offered, so the client needs no market mode. Restricting to `worlds/` keeps the launcher from hosting arbitrary paths |
| **D-16d-3** | **One entry model, 13b's** (A16d-8), built in a new `scripts/slice/offer_menu.gd` (`SliceOfferMenu`). Subjects: **you** (`affordances("", "")`) and **the person the ray meets** (`affordances("", id)`). One entry per affordance, in list order, nothing removed, nothing invented. Kinds: `complete` → enabled, chosen → `intents.submit_offered` → `submit_affordance` unchanged; `composed` (only `talk`, the one type 3D composes) → enabled, chosen → `intents.talk` with the default utterance (Q-16d-7); `body` (the type the client sends from the body: `SliceLink.MOVE_ACTION`) → listed, not choosable, labelled with the key `ui.by-body`; anything else → listed **disabled**, `ui.unsupported`, never sent. `available: false` → greyed **but choosable**, labelled with the reason and, when declared, `ui.needs-range` from `requirement.within_range`, unevaluated. Verdicts are read only while building entries, never in the choose path. The menu is rebuilt from every newer observation while open; each entry carries the affordance it was built from; it closes when its person is no longer perceived, and on a choice | a 3D quick action ("the first available"); separate buy-only and inventory-only lists filtered by action type | Choosing by verdict, or filtering by a named type outside `intents.gd`, would be the client deciding (13b D-b-4; the scan). The same model in both clients makes the two `MENU` reports comparable field for field (P-2) |
| **D-16d-4** | **The reports are 13b's.** The probe prints `MENU {subject, entries: [{index, action_type, target, about, complete, available, reason, label, enabled}], offered}` on every (re)build, `PANELS {...}` with each panel's `component`, `rows` and raw `payload`, and `REQUEST {token, flavour: "3d", request}` from `MineWorldClient.submitted_request`, exactly in 13b's keys | a 3D format | The parity test (16e/13f) and this PR's P-2 compare the two clients with one reader |
| **D-16d-4a** | **Keys (Q-16d-2 [OM]).** **E** still talks to the person under the ray (16a, accepted). **Tab** opens the menu of the person under the ray ("nobody to offer anything to" toast if none). **B** opens your menu with what you have beside it. Esc closes an open interaction menu; with none open, Esc is SET-a's settings menu. With a menu open the mouse is visible, entries are chosen by click or by Up/Down and Enter, and gameplay input is held by SET-a's gate. A connected-only second controls line (`hint.connected`: "E talk  ·  Tab their menu  ·  B you") is added with `ControlsHud.add_line`; the shared `CONTROLS` line is not edited | E opens the person's menu (13b's E; changes the accepted E-talks behaviour and `--link`/`--conversation`); Q for your menu (13b's Q); one contextual key | Keeps every accepted behaviour; `B` is the key §4.4 and §4.9 already gave buying. Interaction feel is the operator's |
| **D-16d-5** | **Names of things: a client-local reader**, new `scripts/slice/things.gd` (`SliceThings`, `RefCounted`): `name_of(id)` = the person's or place's `display_name` (as `display_label` already does), else the item's `name` in the current place's `item-catalogue`, else the last name seen for that id in this instance (a name cache dropped on an instance change, 13b D-b-8), else the key `ui.thing.unnamed` ("something"). **Never the id.** It also reads the three panels' shapes (wallet `balance`, holdings `held[{item, count}]`, shop `listed[{item, price, in_stock}]`), pinned from a live frame in C1, and formats money as 13b does (minor units → `"%d.%02d"`, through `format.money`) | add a catalogue reader to the shared module (ruling 4: only S11-A and S11-C edit it); show `item {id}` as 13b does before R-PK-2 | The 3D rule since 16a is that no id is ever on screen (`display_label`). A second reader in 2D and 3D is a known duplication, guarded by P-2 and raised for a later coordinated module PR (Q-16d-5, R-16d-7) |
| **D-16d-6** | **Wording in the shared layer, one translation for both clients** (Q-16d-4). C2 moves, verbatim and with their `zh_Hans` entries, the concept keys both clients now use from the 2D pack layer to `clients/shared/settings/locale/`: `ui.unsupported`, `ui.unavailable`, `ui.needs-range`, `ui.self`, `ui.menu.empty`, `ui.result.accepted`, `ui.result.rejected`, `ui.result.unavailable`, `ui.result.refused`, `ui.result.unknown`, `panel.wallet`, `panel.holdings`, `panel.shop`, `ui.row.holding`, `ui.row.shop`, `format.money`. The shared `action.move` becomes "Walk" (A16d-11) and the 2D pack keeps "Walk (click where to go)" under `#. override`, so 2D's screen is unchanged. New 3D keys, `en` and `zh_Hans`: `ui.by-body`, `ui.thing.unnamed`, `ui.you`, `ui.menu.nobody`, `hint.connected`. No key is renamed; the union of the two layers after the move equals the union before, entry for entry, except `action.move`'s shared text | copy the keys into the shared layer and leave 2D's (two translations of one concept, AC-SET-4 refuses duplicates without `#. override`); 3D-only key names for the same concepts | SET-a's rule: entries both clients use live once, in the shared layer (step-20 §1.4). A move is SD-SET-a-17's own mechanism; 2D's tests stay green because the keys do not change |
| **D-16d-7** | **Results in words, with names.** `slice_link._on_resolved` routes every non-`move`, non-`talk` result through `intents.take(token)` → `{action_type, target, about}` and shows `ui.result.accepted` with `action.<type>.done` (`{target}`, `{item}` filled by `SliceThings`), or `ui.result.rejected` / `unavailable` / `refused` with `reason.<code>` via `MineWorldText.code` — 13b's D-b-6 and SET-a's `_say`. A reconnect drops pending entries as `ui.result.unknown`, never resent. The English log line keeps ids (INV-SET-6) | keep `"%s: %s"` | A16d-2: "buy: accepted" is an action type on screen; the reason families are SET-a's |
| **D-16d-8** | **Look: the slice's HUD, nothing new.** Panels are `PanelContainer`s with the captions' `StyleBoxFlat` (black 0.45, radius 6, margin 10) and `ControlsHud._style` text (15 px entries, 18 px titles), light on dark, the default theme font plus SET-a's Noto Sans SC fallback. Your menu stands left of centre with "what you have" to its right; a person's menu stands just right of the reticle. Unavailable entries at 0.55 alpha (13b); disabled ones at 0.35. No sound, no animation beyond showing and hiding | 13b's warm 2D panel; a new 3D style | `VIS-3D-GODOT-2` accepted this HUD; reusing it adds no new look for the operator to judge beyond placement (Q-16d-9 [OM: visual default]) |
| **D-16d-9** | **Input while a menu is open** is held by SET-a's gate (whatever C1 finds it to be on the base; SET-a's AC-SET-12 requires one in 3D): no walk, no look, no `move` report. Esc order: an open interaction menu first, then SET-a's settings menu. Both menus close on Esc, on a choice, and when the subject leaves view | edit `player.gd` (V-2 protects it); poll a flag in `slice_link.gd` | One gate for every client menu. If SET-a's gate cannot hold a second menu, that is a bounded change to SET-a's gate recorded in §22.12, not a `player.gd` edit (§22.11 MATERIAL STOP otherwise) |
| **D-16d-10** | **A poor visitor, at test time only.** O-5 copies `worlds/market-town` into a scratch directory and sets the visitor's `economy: { wallet: 250 }` there (13b AC-I6's `copy_dir` precedent). Tea and croissant (250) are affordable; coffee, cake, soup, sandwich are not, and the server offers them unavailable (A16d-6) | buy six cakes to empty the stock (slower, and Alice's shift restocks) | The one honest way to make "the client thinks it unaffordable" a real case: the frozen adversarial criterion of §13's 16d row needs an offer the client could wrongly hide. `worlds/**` is not edited |
| **D-16d-11** | **`AC-13` for the market actions, against 13b's real client.** A new `#[ignore]` test file, `tools/cli/tests/client_3d_market.rs`, plays 13b's own `item_steps` (buy coffee, drink it, walk to Bob, give the scarf, eat the apple) with 13b's 2D client on one fresh Market Town, and the same steps with the 3D client on another, and compares the non-`move` requests pairwise with `mineworld_server::differing_fields` (I-S14-9). 3D's steps file uses 13b's selector format; a `{"choose": {"walk": true}}` step is performed by the 3D probe walking the body straight toward the person until 1.2 m short (13b's `APPROACH_M`, a destination, not a range), on the real controller | a recorded 2D transcript as frozen evidence; wait for 16e | The comparison is only meaningful between the two real clients' own acquisitions (R-S14-9). 16e's `ac13_clients.rs` may absorb this file (Q-16d-11) |
| **D-16d-12** | **Every platform.** No `OS.get_name()` or feature branch in 16d's code; the Rust test spawns `godot` (or `$GODOT`) directly, not the bash launcher, so it runs wherever Godot and cargo run; keys are physical keycodes; the hand checks of §22.9 name macOS (gate), Windows and Linux (checklist), as SET-a does | — | The operator's 全平台 requirement; SET-a's AC-SET-16 scans the shared module only, so 16d states its own no-branch rule and C7 greps for it |

## 22.4 Acceptance (decided before measuring, `ARC-23`)

Every check runs the real 3D client, headless unless stated, against a real `mineworld server` on a fresh
world, through `cargo test -p mineworld-cli --test client_3d_market -- --ignored --test-threads=1`. The
oracle is the save's fact log (`godot2d::worlds::facts`) or the raw frame the probe prints beside its
report, never the client's report alone. Ids are resolved by the Rust test from genesis
(`godot2d::worlds::ids`), never written in a `.gd` file. Bounds are literals fixed here.

**Offers and menus** (Market Town, seat `visitor`, at the café door):

```text
O-1  YOUR MENU IS THE TARGET-LESS OFFERS. Step {"open": "self"} at genesis. PASS iff the MENU's entries,
     ignoring label, equal OFFERED = affordances("", "") of the frame printed with it, one for one, in
     order: same count; for each, action_type, target (null), complete, available and reason as the
     frame carries them, and `about` = the payload's first typed reference. Expected on 12d's Market
     Town: six buy (available), one eat (apple), and move as a `body` entry (not enabled)
O-2  THE PERSON'S MENU IS THEIR OFFERS. From the door, aimed at Bob's head (the ray's entity asserted
     to be Bob, as A16c-14's guard), Tab. PASS iff MENU == affordances("", bob) as O-1, and it holds
     exactly two complete `give` entries (apple, scarf) in the frame's order, both available=false with
     reason "too_far_away" and the label carrying "needs 3 m"; talk listed (composed, unavailable);
     every type 3D does not compose listed disabled
O-3  CHOSEN REGARDLESS. Choose O-2's first give. PASS iff a REQUEST is emitted whose action_type, target
     and payload equal that affordance's after JSON normalisation (whole numbers as integers), the
     server answers it rejected too_far_away, the toast reads ui.result.rejected with the reason's
     words, and the save holds no items-transferred for it
O-4  A MENU HOLDS THE GAME. With your menu open: W held 2 s and 200 px of mouse motion. PASS iff no
     REQUEST is emitted, the body moves < 0.01 m and the camera yaw changes < 0.1°. Esc once closes the
     menu and leaves SET-a's settings menu closed; Esc again opens it
O-5  UNAFFORDABLE IS STILL LISTED (D-16d-10, the poor visitor). Open self. PASS iff all six buys are
     listed in the frame's order; coffee, cake, sandwich and soup with available=false and the frame's
     reason; tea and croissant available. Choose coffee: it is sent, answered rejected, and the save
     holds no money-transferred from the visitor
O-6  UNSUPPORTED IS NEVER SENT. In Bob's menu (near, after walking), press each disabled entry
     (invite, …). PASS iff the REQUEST count does not change
```

**Names** (every step of O-1 … O-3 and the P-1 run):

```text
N-1  NAMED BY THE WORLD. For every MENU entry with an `about`, every holdings and shop row, and every
     result toast about an item: the text contains that item's `name` from the item-catalogue printed in
     the same frame (raw). For every entry with a target, the text contains display_name(target)
N-2  NO ID ON SCREEN. At each step, a tree walk of every visible Label, Button and Label3D under the HUD,
     the menus and the figures. Remove from each text the numbers the entry's own data supplied
     (counts, prices, the formatted balance). PASS iff no whole-word decimal token remains that equals
     an id the session has seen (perceived people and places, catalogued items, payload references)
```

**What you have:**

```text
K-1  THE PANELS ARE THE DISCLOSURE. With your menu open: the "Money" row equals format.money of the
     frame's wallet.balance; each "Carrying" row's count equals holdings.held's count for that item, in
     the frame's order; each "For sale here" row equals shop.listed's price and in_stock
K-2  THEY FOLLOW. After each accepted step of the P-1 run (buy, drink, give, eat), the next PANELS within
     2 s shows: wallet lower by exactly the coffee's listed price, equal to the save's
     money-transferred amount; coffee 1 then 0; scarf 0; apple 0 — each matching the save's
     items-transferred / items-consumed facts
```

**Parity with the 2D client (`AC-13`)** (two fresh Market Towns, one per client, as the evidence
README requires):

```text
P-1  THE SAME FOUR REQUESTS. 13b's item_steps played by 13b's 2D client (its harness, unchanged) and by
     the 3D client. PASS iff each run's non-move requests are, in order, buy, drink, give, eat; for each
     pair differing_fields(2d, 3d) ⊆ {ActorLocation}; 2D's actor_location is null and 3D's is a Location
     in the observer's place at the time; each pair is answered with the same result kind (accepted);
     and the fact types each request caused (by caused_by in each save) are equal as multisets
P-2  THE SAME MENUS. The 2D and 3D MENU reports for: self at genesis, self after the buy, Bob near before
     the give. PASS iff, after dropping 2D's `walk` entry and both clients' move entry, the entries'
     (action_type, target, about, complete, available, reason) sequences are equal. Labels are
     presentation-only
P-3  WORDS DECIDE NOTHING. The 3D steps again with --settings=<scratch file: zh_Hans> and with
     --wording=none. PASS iff both transcripts have the same semantic cores as the `en` run, pair for
     pair (differing_fields empty apart from ActorLocation's values)
```

**Mutations, one per guard — each planted in the working tree, run, recorded, reverted** (`git status`
clean of it afterwards; `git grep MUTATION` empty):

| # | Guard | Mutation | Expected red |
| --- | --- | --- | --- |
| M16d-1 | O-2, O-5 | the menu drops entries with `available: false` | O-2 lists no give; O-5 lists two buys |
| M16d-2 | O-5 (§13's frozen criterion) | a client affordability rule: hide a buy whose listed price exceeds the wallet's balance | O-5 fails naming coffee, cake, sandwich, soup as offered and not listed |
| M16d-3 | O-3, O-5 | the choose path returns when the entry is unavailable | O-3 fails: no REQUEST; O-5's coffee not sent. `client_rules`/R3'-style review also names it |
| M16d-4 | O-2 | entries from `affordance(type, target)` (the first match) | O-2 fails: one give |
| M16d-5 | O-1 | entries sorted by label | O-1 fails on order |
| M16d-6 | P-1 | the buy rebuilt with `client.submit(type, null, {"item": <id string>})` instead of `submit_affordance` | P-1 fails naming Payload on the buy pair (the scan also fails on the planted literal; both recorded) |
| M16d-7 | P-1 | the 3D give chooses the second give entry (another item) | P-1 fails naming Payload on the give pair; K-2's scarf line fails |
| M16d-8 | P-1's 3D clause | 3D sends `actor_location: null` | P-1 fails on "3D's is a Location"; `differing_fields` alone would pass — recorded as the reason the clause exists |
| M16d-9 | N-1 | `SliceThings` does not read the catalogue | N-1 fails naming each item row ("something") |
| M16d-10 | N-2 | the unnamed fallback shows the id | N-2 fails naming the label and the id |
| M16d-11 | K-2 | the holdings panel caches the first frame's holdings | K-2 fails at the buy's step |
| M16d-12 | O-4 | the menu does not take the gate | O-4 fails: REQUESTs (moves) during the menu |
| M16d-13 | P-3 | the steps runner resolves a selector by label text | the zh_Hans run chooses nothing; P-3 fails |

## 22.5 Godot checks (every run of the real client)

```text
G-0  PARSE. After each commit that adds or renames a class (SliceOfferMenu, SliceThings, the market
     probe): `godot --headless --path clients/3d-spike --import` and one launch, with no "Parse Error",
     "SCRIPT ERROR" or "Cannot get class" line. A script error fails every mode
G-1  STANDALONE, unchanged (I-S14-7): --drive "all drive checks pass" (Jolt); --measure (the base's
     lines); --threshold; --character "all character checks pass"; --perf within 15 % of the base per
     viewpoint, confirmed by one re-run; ./mineworld-3d --drive once
G-2  CONNECTED, the accepted checks, on social-cafe as on the base: --link "all link checks pass" (every
     talk's ray target asserted, A16c-14); --conversation "conversation on screen, no ids"; --target T-1
     … T-4 (T-2' if 16c has landed). Then --link once with --world=market-town: the same verdict
     (Market Town carries the same places, R-12d-4)
G-3  NEW: client_3d_market — O-1 … O-6, N-1, N-2, K-1, K-2, P-1 … P-3, each PASS on the final head;
     13b's client_2d_interact still passes unchanged (its keys moved, not renamed)
G-4  SCANS: cargo test -p mineworld-acceptance --test client_rules (green, no new admission) and
     --test client_text (SET-a: every 16d key in en and zh_Hans; no literal text in a 3D sink);
     python3 scripts/check_client_rules.py and --check-pack presentation/mineworld-default/2D (green
     after D-16d-6's move); a grep of 16d's new files for OS.get_name / OS.has_feature (none)
G-5  WINDOWED FRAMES (one Godot window at a time; a stalled capture is INCONCLUSIVE and re-run once):
     --market --frames, five frames at 1600x900: your menu at the door; Bob's menu from the door
     (greyed gives, "too far away · needs 3 m"); Bob's menu beside him; your menu after the buy (Coffee
     × 1, money lower); your menu in zh_Hans. Each looked at, one image at a time, facts recorded
G-6  INTERACTIVE, by the implementing agent before the PR is opened: ./mineworld-slice
     --world=market-town for three minutes doing OC-1 … OC-5; nothing in the log but the expected
     [link] lines; reported in words, not claimed as acceptance
```

Every connected run uses its own server on port 0 and kills only its own PID; a run that joined anything
else is `INCONCLUSIVE`.

## 22.6 What the operator checks by hand (exact list; goes into the PR body)

Each item states what to do and what must be seen. The operator's verdict, not the agent's, accepts the
interaction and its look. macOS is the gate; Windows (Git Bash, Godot's console build on `PATH`) and
Linux are checklist columns, as SET-a's §8.

```text
OC-1 ./mineworld-slice --world=market-town. At the café door press B.
     See: a panel titled "You" with Buy Coffee, Buy Tea, Buy Croissant, Buy Cake, Buy Sandwich, Buy
     Soup, Eat Apple, and a greyed "Walk" line; beside it Money 2000.00, Carrying Apple × 1 and
     Scarf × 1, For sale here with each price and stock. No number on screen is an id. WASD and the
     mouse do nothing while it is open; Esc closes it
OC-2 Choose Buy Coffee. See the toast "You bought Coffee". Press B again: Money 1997.00, Carrying
     Coffee × 1, and a new entry Drink Coffee
OC-3 From the door, look at Bob (his name over his head) and press Tab. See his menu: two greyed
     entries "Give Bob Achterberg Apple — too far away · needs 3 m" and the same for Scarf, Talk,
     and the activity entries marked "not supported by this client". Choose a greyed give anyway: a
     toast says the world said no, too far away; nothing changes in your things
OC-4 Walk up to Bob (about a pace away), Tab, choose Give … Scarf: a toast; B shows no scarf.
     Then B → Drink Coffee, then B → Eat Apple: a toast each; Carrying is empty
OC-5 E still talks: look at Alice at the counter and press E; her reply appears as a caption, as
     before. Esc with no menu open opens the settings menu (SET-a)
OC-6 Settings → Language → 简体中文. Open B and Tab on Bob: every menu and panel line is Chinese; the
     names (Coffee, Bob Achterberg) stay as the world wrote them; no tofu boxes; nothing clipped
OC-7 The look: menus and panels read as the slice's own HUD — light text on a dark translucent panel,
     the same font as the controls line — and nothing else on screen changed. Standalone
     ./mineworld-slice is exactly the accepted slice (no new line, no menu)
OC-8 Windows and Linux (checklist): OC-1, OC-2, OC-4 and OC-6 on each; on Linux OC-6 on a machine with
     no CJK system font (`fc-list :lang=zh` empty); record the platform line SET-a prints at start
```

## 22.7 The visual-regression guard (`VIS-3D-GODOT-1`, `VIS-3D-GODOT-2`)

```text
V-1  STANDALONE FRAMES, unchanged. --shots at 1600x900, twice on the base (the noise floor) and once on
     the head, compared by 16a's tools/frame_diff.gd. PASS iff each view's share of pixels differing by
     more than 8/255 is at most the base's own run-to-run share + 0.5 points. Expected: identical within
     noise (16d builds nothing standalone). Any view over that is looked at side by side and its cause
     named; an unexplained visible change is a material stop. --hud's frame likewise (CONTROLS untouched)
V-2  THE CHARACTER AND THE SCENE. 16d's diff touches none of human.gd, player.gd, npc.gd,
     character_slot.gd, camera_rig.gd, posture.gd, build.gd, the slice's scene builders (cafe*.gd,
     street.gd, streetscape.gd, terrace.gd, shop_interior.gd, slice_world.gd) or clients/3d-spike/assets/**
     (a path check on the PR diff); --character passes
V-3  CONNECTED FRAMES, changed only as intended. --conversation's frames, base and head: frame_diff's
     differing regions are only the connected-only controls line (D-16d-4a); everything else within
     noise
V-4  THE NEW FRAMES (G-5) are judged by the operator (OC-7), not by the agent
```

## 22.8 Commit plan

Each commit tracks implementation, validation and review separately; `[x]` needs the work and its
evidence. A planned commit may become several coherent ones (mapping recorded). Evidence goes into
§22.12 as `E16d-<n>`. Godot runs over two minutes run in the background, one Godot window at a time,
logs under `clients/3d-spike/shots/16d/` (ignored).

### 16d-C0 — Design (this section) — docs only

- [x] Implementation: §§22.1–22.13, from the audit in §22.2.
- [x] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`
  (they do not read `.structured-coding/`; run to show nothing else moved); results in §22.12.
- [x] Review: every finding cites a file and line or a branch commit; every acceptance line states its
  pass condition before anything runs; every guard has a mutation; non-goals match 12e's, 16c's,
  SET-a's and 13b's scopes and ruling 4. Self-review by the drafting session; the freeze is pending.

### 16d-C1 — Re-audit and the baseline on the merged base

**Goal.** "Before" measured by the commands "after" will use; every (branch) anchor confirmed on `main`.
**Scope.** Ledger only. **Depends on** freeze and §22.11's precondition.

- [ ] Implementation: none in code. Re-read A16d-1 … 16 on the base and record differences. Dump one
  Market Town frame for `visitor` (the probe's raw frame line) and record the exact JSON of
  `item-catalogue`, `wallet`, `holdings`, `shop` and of one `buy`, `give`, `eat` affordance, and the
  reason code an unpurchasable buy carries (poor-visitor copy). Find SET-a's input gate and Esc order in
  3D. Walk the café from the door to 1.2 m from Bob on the real controller and record whether a straight
  line is clear on 12d's furniture (R-16d-2).
- [ ] Validation (E16d-1): G-1 and G-2 on the base; V-1's two base `--shots` runs and `--hud`; 13b's
  `client_2d_interact -- --ignored` on the base (green, the 2D side P-1 will use).
- [ ] Review: each base result matches its accepted figure, or the difference is recorded with its cause
  before anything changes.

**Commit boundary.** The ledger.

### 16d-C2 — Wording first (SD-6, D-16d-6)

**Goal.** Every key 16d's code will use exists in `en` and `zh_Hans` before the code. **Scope.**
`clients/shared/settings/locale/{en,zh_Hans}.po` (the moved keys, `action.move` neutral, the five new
keys), `messages.pot` (regenerated by SET-a's check); `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po`
(the moved keys removed; `action.move` kept under `#. override`); `docs/DECISIONS.md` (the dated note
under ARC-47, Q-16d-10). **Non-goals:** any `.gd`.

- [ ] Implementation: as scoped; the move verbatim (msgid, msgstr, comments).
- [ ] Validation (E16d-2): `cargo test -p mineworld-acceptance --test client_text --test client_rules`;
  `python3 scripts/check_client_rules.py` and `--check-pack presentation/mineworld-default/2D`; a union
  check recorded (both layers' entries before = after, apart from `action.move`'s shared msgstr); 13b's
  `client_2d_interact` and SET-a's `client_settings` 2D tests green, with 2D's visible strings identical
  (the 2D `MENU` labels of AC-I2's run equal the base's).
- [ ] Review: no key renamed; no rule in wording (no number, no condition); `zh_Hans` wording reviewed
  against the `en` meaning.

**Commit boundary.** Catalogs and the decision note.

### 16d-C3 — Names and what you have, read (SD-5)

**Goal.** `SliceThings` (D-16d-5): names and panel rows from disclosure; nothing drawn yet.
**Scope.** New `scripts/slice/things.gd`; `slice_link.gd` (`display_label` keeps its rule and delegates
the item half; nothing else).

- [ ] Implementation: as scoped; shapes exactly as C1 recorded them.
- [ ] Validation (E16d-3): G-0; a probe line on Market Town listing every catalogued kind's name and the
  three panels' rows against the raw frame (the K-1 comparison, made early); G-2's `--link`.
- [ ] Review: no arithmetic beyond formatting; no id reachable as display text; no action literal.

**Commit boundary.** The reader.

### 16d-C4 — The menus, the requests, the results (SD-2 … SD-4)

**Goal.** O-1 … O-6 become true (shown in C5). **Scope.** New `scripts/slice/offer_menu.gd`; `intents.gd`
(`submit_offered`, `composes`, pending `about`); `slice_link.gd` (`_on_resolved` through keys, D-16d-7);
`slice_main.gd` (attach the menu and the connected-only line when connected); SET-a's gate as found in C1.

- [ ] Implementation: as scoped; the choose path reads no verdict; complete entries only through
  `submit_affordance`.
- [ ] Validation (E16d-4): G-0; G-1 `--drive`; G-2; an interactive two-minute run on Market Town
  (B, Tab, buy, give from the door) with the log clean.
- [ ] Review: no entry is created that the frame did not offer; `slice_link.gd`'s line count did not grow
  by more than the result routing; `client_rules` green with no new admission.

**Commit boundary.** The interaction.

### 16d-C5 — The probe, the launcher, and the market acceptance

**Goal.** O-, N-, K- lines with M16d-1 … 5, 9 … 12. **Scope.** New `scripts/slice/slice_probe_market.gd`
(steps runner in 13b's selector format, `MENU`/`PANELS`/`REQUEST` lines, the N-2 tree walk, `--frames`);
the probe dispatch (`slice_probe_world.gd` or `slice_probe.gd`, one line); `mineworld-slice`
(`--world=<pack>`, `--market` headless, help text); new `tools/cli/tests/godot3d/mod.rs` and
`tools/cli/tests/client_3d_market.rs` (O-1 … O-6, N-1, N-2, K-1, K-2; the poor-visitor copy).

- [ ] Implementation: as scoped.
- [ ] Validation (E16d-5): the tests PASS; each mutation red as stated, then reverted green; counts and
  wall time recorded.
- [ ] Review: every expected value is a literal of this plan, a genesis fact or the raw frame, never
  computed by the code under test; no `.gd` names an item, a person or an action type for the scenario.

**Commit boundary.** The probe, the launcher flag, the test.

### 16d-C6 — Parity with the 2D client (SD-7)

**Goal.** P-1 … P-3 with M16d-6, 7, 8, 13. **Scope.** `client_3d_market.rs` (the 2D run through 13b's
helpers; the comparison through `differing_fields`; the zh_Hans and no-wording runs).

- [ ] Implementation: as scoped.
- [ ] Validation (E16d-6): P-1 … P-3 PASS; mutations red, reverted; the two transcripts and the two saves'
  fact types recorded in the ledger.
- [ ] Review: no hand-written comparison (I-S14-9); the 2D client and its harness are run unchanged.

**Commit boundary.** The parity tests.

### 16d-C7 — Frames, documents, gates, close

**Scope.** G-5 frames; `clients/3d-spike/README.md` (keys, `--world=<pack>`, `--market`);
`docs/MVP_STATUS.md` (the 3D column of Market and Items; the S14 row); this section's ledger and handoff.

- [ ] Implementation: as scoped.
- [ ] Validation (E16d-7, the final executable head): G-0 … G-4 once more; V-1 … V-3; the Rust gate —
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test
  --workspace` once; `check_decision_ids`, `check_doc_headings`, `check_scratch.py scan`; the path check
  (nothing under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `cognition/`, `worlds/`,
  `tools/cli/src/`, `clients/protocol/`, `clients/2d/scripts/`).
- [ ] Review: every acceptance line has its evidence; deviations recorded; the PR body carries OC-1 …
  OC-8 verbatim and the known limitations (no group activity in 3D; talk's words fixed; panels limited
  to money, carrying, for sale).

Then push, open the PR **READY FOR OPERATOR REVIEW**, and stop. Do not merge.

### 16d test ownership

```text
STATIC      Godot's parser (G-0); client_rules.rs (no new admission); client_text.rs (SET-a's, 16d's keys);
            check_client_rules.py --check-pack (2D pack after the move)
REAL RUN    client_3d_market.rs (#[ignore], needs Godot): O-, N-, K-, P- against real servers, the save as
            oracle; G-1, G-2 accepted checks; G-5 frames; G-6 interactive
UNIT        none: every claim is a client against a server; a unit test of the entry builder would assert
            what O-1 measures end to end
GATE 1      NOT REQUIRED — no language model
CI          client_rules and client_text run in CI's test job; the Godot-gated file does not run in CI
            until 13c's `clients` job exists — proposed to join it then (Q-16d-11); reported NOT RUN in CI
```

## 22.9 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-16d-1** | SET-a or 12d slips (12d is paused for 12n), so 16d cannot start | Accepted by D-16d-1: a second catalog or an id-showing menu would be debt the operator asked not to have. 16d's design does not change when they land |
| **R-16d-2** | 12d's café furniture blocks a straight walk from the door to Bob, so the 3D `walk` step stops short or `give` stays out of range | C1 checks the line; if blocked, the probe walks via one waypoint chosen from the scene's floor in C1 and recorded (presentation, like 16a's counter point). The server still decides every stride |
| **R-16d-3** | Without 12e, a stride the server stops short (bodies) is not corrected (F-S14-8), so the drawn body drifts near Bob | The probe stops 1.2 m short, outside the 595 mm body clearance (13b RK-b2(b)); reports carry the server's position; drift is a stated known limitation until 12e |
| **R-16d-4** | 13b's 2D "walk to Bob" is blocked on 12d's café (13b RK-b2(a)), so P-1's give pair cannot be made | Reported `INCONCLUSIVE` for that pair with the 2D log, never patched in either client; buy, drink and eat still compared |
| **R-16d-5** | 12e, 16c and SET-a edit `slice_link.gd`, `slice_main.gd`, `intents.gd`, the probe dispatch and the launcher at the same time | 16d's code is in four new files; its edits elsewhere are listed per commit; the later PR rebases; the launcher's flags are additive |
| **R-16d-6** | `item-catalogue`'s JSON differs from SD-D10's text (a string id instead of a typed reference) | Pinned from a live frame in C1 (A16d-7); `SliceThings` reads identities with one helper that accepts both, as 13b's `_ref` |
| **R-16d-7** | Two entry builders and two name readers (2D and 3D) drift apart | P-2 compares the two `MENU` reports field for field; extraction into a shared client module is raised for a coordinated PR (Q-16d-5) |
| **R-16d-8** | Moving 2D's panel keys breaks a 2D string or a 13b test | The move is verbatim and checked by a union check; 13b's tests and 2D's `MENU` labels are compared before/after in C2; a changed 2D string is a material stop |
| **R-16d-9** | The menus overlap the captions or the reticle at some window sizes, or clip in `zh_Hans` | Fixed anchors (D-16d-8); G-5's zh_Hans frame; OC-6 and OC-7 judged by the operator |
| **R-16d-10** | Another lane's Godot window covers a capture, or a run joins a foreign server | One window at a time; stalled captures `INCONCLUSIVE` and re-run once; port 0 and own-PID kill |

## 22.10 Questions for the freeze (primary session; **[OM]** = the operator's) — ruled 2026-10-09, §22.0

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-16d-1** | **Order** (D-16d-1): after 12d, 13b and SET-a, with the inset on `main`; 12e and 16c in either order. This drops §13's "after 12e" | **As D-16d-1.** No 16d claim needs bodies; waiting for 12e would serialise two independent PRs behind 12n |
| **Q-16d-2 [OM: interaction feel]** | **Keys.** (a) E talks (unchanged), Tab opens the person's menu, B opens yours; (b) E opens the person's menu (2D's E), talk inside it; (c) Q opens yours (2D's Q) instead of B | **(a).** It keeps the accepted E-talks behaviour and `--link`, and B is the key the step already promised for buying |
| **Q-16d-3** | **Where buying is reached** (QS14-7) | **Ruled (§22.0), restated:** the client never decides where buying is allowed; your menu shows exactly the `buy` offers the server sends. "Anywhere in the shop" is today's economy rule, not a client choice. Buying at the counter is a later economy / World Interaction List rule (A16d-17), outside 16d |
| **Q-16d-4** | **Shared wording** (D-16d-6): move 16 concept keys from the 2D pack to the shared layer verbatim, and make the shared `action.move` neutral with a 2D `#. override` | **Yes.** One translation per concept is SET-a's own rule; 2D's screen does not change |
| **Q-16d-5** | **Duplication between clients**: 3D-local `offer_menu.gd` and `things.gd` now, or a shared client module (`clients/shared/offers/`) used by both, editing 13b's merged 2D files in 16d? | **Local now, P-2 guards parity; propose the extraction as its own PR after 16e**, coordinated with S12 and with S11 for the catalogue reader (which belongs in the protocol module, ruling 4) |
| **Q-16d-6** | **Group activity in 3D.** `invite`, accept, decline, join, leave are listed disabled in 16d. "Framework, not demo" item 3 wants every interaction playable in both clients | **A follow-up PR, 16g** (3D composers for the five group actions, with a typed line), designed after 16d; the gap is stated in 16d's PR and in `MVP_STATUS.md`. **Ruled yes (§22.0): PR 16g** |
| **Q-16d-7** | **Talk from the menu** sends the default utterance, as E does; no text box in 3D yet | **Yes in 16d**; a typed line belongs with 16g's text input |
| **Q-16d-8** | **Panels**: 16d shows money, carrying and for sale only; 13b also shows conversation history, acquaintances, invitations, agenda, employment, other | **Only those three in 16d** (the brief's "inventory"); the others with 16g, where invitations matter |
| **Q-16d-9 [OM: visual default]** | **Menu look and placement** (D-16d-8): the captions' dark translucent panel; your menu left of centre with what you have beside it; a person's menu just right of the reticle | **As D-16d-8**, judged in play (OC-7) |
| **Q-16d-10** | **Records**: no new ARC; a dated note under ARC-47 that the 3D client follows the same menu model (complete unchanged, composed by `intents.gd`, the rest disabled, verdicts never chosen by) | **Accept** |
| **Q-16d-11** | **Test placement**: `client_3d_market.rs` (with `godot3d/mod.rs`) owned by 16d, absorbed or re-used by 16e's `ac13_clients.rs`; joins 13c's `clients` CI job when that exists | **Accept** |
| **Q-16d-12** | **The poor visitor** (D-16d-10): a test-time copy of Market Town with `wallet: 250`, never a committed world | **Accept** |
| **Q-16d-13 [OM: wording]** | **The `zh_Hans` wording** of 16d's five new keys and of `action.move`'s neutral form, proposed in C2 | **Proposed in C2, judged in OC-6** |

## 22.11 Execution contract (confirmed at the freeze, 2026-10-09)

```text
PROJECT / PR        MVP-0 · S14 / PR 16d — Market Town in 3D: buy, hand over, eat or drink, what you have
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-15-demo-3d.md §22 (this section)
RELATED / BINDING   overall.md §3 (S14), "Parallel build-out" (rulings 4, 6, 7, 9), "Framework, not demo"
                    items 3–4, "One world, two views", "The World Interaction List"; this file §§1–17,
                    §18 (16b), §19 (16a), §20 (16c); step-13 §15 (13b, as merged); step-20 §§3.6, 12
                    (SET-a, as merged); step-11 SD-D10 (12d, as merged); ADOPTION.md §§2–4;
                    ENGINEERING_RULES §§3–12, 19; HUMAN_REVIEW_QUEUE (VIS-3D-GODOT-1/-2)
PRECONDITION        on main: S15 12d merged; S12 13b merged; S20 SET-a merged; `./mineworld-slice --world
                    --link` passes on that main (the 0.4 m inset). 12e and 16c not required
IMPLEMENTATION BASE main at the start of implementation, meeting the precondition; branch
                    mvp0/pr-16d-market-3d from it; worktree /Users/yuema137/mineworld-worktrees/impl-16d
                    (this session only; a fresh implementation session)
APPROVED SCOPE      §22.1 (SD-1 … SD-7), as answered by Q-16d-1 … 13 and the rulings in §22.0 (group
                    activity in 3D is PR 16g; buying where the server offers it, A16d-17)
FROZEN INVARIANTS   I-S14-1 (client_rules green, no new admission), I-S14-2, I-S14-3, I-S14-6, I-S14-7,
                    I-S14-9, I-S14-10; no edit to clients/protocol/**, clients/2d/scripts/**, worlds/**,
                    tools/cli/src/**, server/, systems/, kernel/, contracts/, persistence/; no regression of
                    VIS-3D-GODOT-1 or -2 (§22.7); no id on screen; no key renamed; the acceptance of §22.4
                    as written
SEQUENCE            C0 → (freeze) → C1 → C2 → C3 → C4 → C5 → C6 → C7, each committed and pushed when
                    coherent
COMMANDS            cargo fmt / check / clippy -D warnings / test ($HOME/.cargo/bin/cargo if needed);
                    python3 scripts/check_doc_headings.py, check_decision_ids.py, check_scratch.py,
                    check_client_rules.py; godot --headless --path clients/3d-spike … and the tools under
                    clients/3d-spike/tools/; ./mineworld-slice and ./mineworld-3d modes; git and gh (no
                    merge); mkdir -p; sed -n. Never python3 -c, sed -i, awk, xargs, curl, heredoc writes;
                    files through Read, Edit, Write. One Godot window at a time; only this run's own
                    server process is killed
VALIDATION BUDGET   real client runs each ≤ ~10 min, background when > 2 min; client_3d_market ≈ 10 Godot
                    runs (2D and 3D) ≈ 10 min per full pass; total ≤ ~3 h wall time including mutations and
                    retries; full cargo test once on the final head; real-model: NOT REQUIRED
LIVE DOCUMENTATION  this section (§22.8 checkboxes, §22.12 ledger)
HANDOFF             §22.13 (one authority)
ENDPOINT AUTHORITY
  implementation + local validation   authorized by the freeze (§22.0), once the precondition holds
  semantic commits, branch push       authorized
  PR creation / update                authorized, READY FOR OPERATOR REVIEW
  CI repair                           authorized for this PR's own failures
  merge                               explicit operator authorization only
POST-MERGE SYNC     the primary session owns §13, §14 and overall.md; the implementing session owns §22
NORMAL STOP         PR 16d READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any regression of the accepted visuals (V-1 … V-3); any edit to the shared protocol
                    module, the server, a System or World Pack, a contract, or 2D scripts; a 2D visible
                    string that changes; a renamed key; a new scan admission or a weakened scan; SET-a's
                    gate unable to hold a second menu without more than a bounded change to it; a
                    precondition found false after starting; a change to any frozen invariant
```

## 22.12 Ledger (live)

```text
C0      drafted on plan/s14-16d from main @ f80bbb7. check_doc_headings: "191 numbered sections across
        26 documents, none duplicated"; check_decision_ids: "73 decision ids, all distinct" (neither
        reads .structured-coding/; run to show nothing else moved). §22's headings 22.1–22.13 are
        unique within this file. Item names in OC-1 … OC-4 are 12d's (QD-15) and are re-read in C1
FREEZE  2026-10-09: §22.0 record and rulings; Q-16d-3 restated; A16d-17 recorded; Q-16d-6 → PR 16g;
        §22.11 confirmed. Doc checks again: "191 numbered sections across 26 documents, none
        duplicated"; "73 decision ids, all distinct"
```

## 22.13 Handoff (live)

```text
checkpoint     C0 drafted on plan/s14-16d (docs only, PR #105); DESIGN FROZEN 2026-10-09 (§22.0)
next action    once 12d, 13b and SET-a have merged and the inset is on main, a fresh implementation
               session in /Users/yuema137/mineworld-worktrees/impl-16d on mvp0/pr-16d-market-3d
               starts at C1. The [OM] defaults (keys, look, zh_Hans wording) are judged at the
               milestone play-test through OC-1 … OC-8
background     none
notes          12d is paused for 12n (8814aad); 13b is in implementation (98bd42e, C5 done); SET-a waits
               for 13b; 12e is being planned on plan/s15-12e (may take §21 of this file)
```

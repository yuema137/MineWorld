# PR 13d — Bodies in 2D: loose objects drawn and moved, kick / throw / aimed throw from the map

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended) — not frozen. Only the operator freezes this design.

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-20), DRAFT
Step / row:             S12 (step-13-client-2d.md §9, row "13d — Bodies in 2D")
Author:                 design sub-agent (Opus), worktree /Users/yuema137/mineworld-worktrees/design-13d,
                        branch docs/pr-design-13d
Planning base:          origin/main @ 6fe45d58 (Merge #369 fix/ball-look), audited 2026-10-20
Approved by / evidence: none yet
Implementation base:    origin/main at C1's start; C1 records the SHA
Start condition:        this design frozen; the lane lead's dispatch. S15 12c, 12d and 12e are merged
                        (the row's "waits on" is met: see §2)
Execution contract:     §12 (empty until freeze)
Implementer:            Haiku-class for C1–C3 (well specified); Opus review of C2's menu change
Decision ids:           ARC-342 (assigned by the coordinator) — one record, §4.6
Lifecycle:              DRAFT
```

---

## 1. What this PR is, in three lines

1. Most of the 13d row is already on `main`: walls and solids are drawn, routing is the server's, and
   `kick`, default `throw` and `shove` already reach the 2D menus through the generic complete path.
2. What is left: the 2D view does not draw loose objects at all, so a ball or a box is invisible, a
   kicked ball has nothing to animate, and an aimed throw has no way to be asked for.
3. This PR draws `loose-objects`, animates each `object-moved` along its `path`, makes a drawn object a
   menu subject, composes an aimed `throw` from a floor click, and proves it on three worlds.

So this is a small closing PR: three code commits and one docs commit, no framework change.

---

## 2. Audit: what of the 13d row is delivered

Every row below was read on `origin/main @ 6fe45d58`.

| 13d row item | State | Evidence |
| --- | --- | --- |
| Walls and solids drawn from `place-shape` | **Delivered** | `clients/2d/scripts/scene/furnishings.gd:52-101`: a shell solid (one that names no furnishing) is drawn as an outline (`_outline`, :135), a furnishing's solids as filled footprints (:126), a bound kind as its sprite. Commit `42555513` "draw disclosed furnishings and shell solids (ARC-307, C4)", merged in #343 (`88735fcf`). `clients/2d/PRESENTATION.md` §6 states the rule. |
| NavigationServer2D routes around solids | **Withdrawn and replaced** | Operator ruling QN-8: "13d's client planning withdrawn in favour of `walk-to`" (`step-11-bodies.md:9788`, `:9886` F-N10, `:9997` SD-N12, `:10244`). The server routes round walls, furniture and loose objects. `clients/2d/scripts/walker.gd:1-12`: a click is one `walk-to`, then one `walk-step` per wall second. Commit `c163fec7` "a click walks by the server's route", merged in #355 (`491386aa`); `5b51d998` adds facing hysteresis. The step-13 §5.4 tools row (`step-13-client-2d.md:441`, "Adopt NavigationServer2D … (PR 13d)") is stale; C4 marks it superseded. |
| The server stops the player at a wall; the client reconciles | **Delivered (behaviour); not proven on bodies-yard** | `walker.gd:110-149` follows any position the world gives (`reconciled`), commits `0144ddf5` ("a stride the world slid along a solid goes on from where it ended") and `a9f6d9e8` ("a displacement ends the walk"). No 2D test runs against `worlds/bodies-yard`: `grep bodies-yard tools/cli/tests/client_2d*.rs` finds nothing. C3 adds the run. |
| `kick`, default `throw`, `shove` through the generic complete path | **Delivered (path); unreachable for objects in practice** | `systems/bodies/src/offer.rs:5-11, 40-58`: the pack offers complete `kick { object }` and `throw { object, toward: null }` with no target, and complete `shove {}` to a near person. `clients/2d/scripts/menu.gd:170-176` lists any complete offer as enabled and `menu.gd:144-145` sends it unchanged (`intents.submit_offered`). So "Kick Item 24" shows in the player's own menu (E) today. But the object is not drawn, so the player cannot see what it kicks or where it went. |
| Loose objects drawn from `loose-objects` | **Missing** | No `.gd` file under `clients/2d` reads `loose-objects` (`grep -rn loose-objects clients/2d` is empty). The 3D client does (`clients/3d-spike/scripts/slice/object_view.gd:56-83`, S15 12e, #254 `effd23ae`; football look `0fbf6efc`, #369). |
| Aimed `throw` from a floor click | **Missing** | `intents.gd:17` `COMPOSED` has no `throw`; `menu.gd:188-201` lists a `point` kind disabled. The 3D client aims with a second G (`clients/3d-spike/scripts/slice/intents.gd:120-125`, step-11 §22 SD-E6). |
| Object animation along `path` (R-S11-4 events) | **Missing in 2D; events delivered** | The shared module carries events: `clients/protocol/mineworld/observation.gd:42` `events_of`, `:186` `events`; `delta.gd:17,52`. `systems/bodies/src/event.rs:170-178` `ObjectMoved { object, place, from, to, how, by, path }`. 3D plays the path (`object_view.gd:139-200`). 2D draws nothing to move. |
| Shoved by another seat, drawn where the server put it | **Delivered (behaviour); not proven on bodies-yard** | `walker.gd:116-134`, `people.gd:61` reconcile every position from the observation. C3 adds the run. |

Other merged work checked for overlap: S29-f (`pr-s29f-station-2d.md`) and S33-h (`pr-s33h-2d-parity.md`)
touch façades, place names and station fixtures, not objects; S33-c (#381, `e7c095d0`) is the 3D world
seam. None draws a loose object in 2D. S30-f (`e15936d4`) added the glance and inspect lines to
`menu.gd`; C2 keeps them.

---

## 3. Scope

### 3.1 In scope

- **Objects drawn.** Each entry of the observer's place's `loose-objects` is drawn at its `at`, by its
  disclosed shape: a ball as a disc of its radius, a box as its footprint rectangle. A Presentation Pack
  may bind a sprite by shape.
- **Objects moved.** When an object's disclosed `at` changes: if this observation carries an
  `object-moved` event for it with a non-empty `path`, the drawing follows the path, keyframe by keyframe,
  and ends exactly at `to`. Otherwise it eases to the new `at`.
- **Object as a menu subject.** A click on a drawn object opens a menu of the offers that name it (the
  player's own target-less offers whose payload refers to that object). No offer: the click walks toward
  the object, as a floor click does.
- **Aimed throw.** The object menu lists "Throw <ball> at…" whenever the world offers a `throw` of it. The
  next floor click sends that `throw` with `toward` set to the clicked point. Esc drops the aim.
- **Proof on three worlds**, §6.

### 3.2 Not in scope

- Any change under `kernel/`, `contracts/`, `systems/`, `server/`, `worldpack/`, `persistence/`,
  `authoring/`, `sdk/`, `cognition/`, `worlds/` (step-13 I-4). The generated variant world is written by
  the test into a scratch directory, never into `worlds/`.
- Route planning in the client (withdrawn, QN-8).
- Any collision, reach or throw-range rule in the client. The server decides; the client draws.
- New art. Plain drawing is complete; a sprite binding is optional pack data (Q-13d-4).
- Drawing objects of places the observer is not in (the pack discloses `loose-objects` with the
  observer's place only).
- The 3D client. It already does all of this.

---

## 4. Design

### 4.1 One new scene file: `clients/2d/scripts/scene/objects.gd`

It follows `furnishings.gd`'s shape: `setup(projection, presentation, town, world)` and
`reconcile(observation)`, called from `app.gd:_on_observed` right after `furnishings.reconcile`.

```text
reconcile(observation):
  place   = observation.place()
  listing = observation.component_value(place, "loose-objects")      # Array of {object, at, shape}
  for entry in listing:
    id = entry.object.entity  (a typed reference; the reference is kept as received, for requests)
    new id      -> build its node at to_screen(to_plan(place, at))
    known id, at changed -> move(id, to, path_for(observation, id))
  ids no longer listed -> free their nodes
  place changed        -> free all, rebuild
```

Example: in bodies-yard's court, the observer sees `court-ball` (a ball of radius 110 mm, say) at
`(3000, 2000)`. The client draws a disc of 0.11 m radius at that point. The player kicks it. The next
observation lists it at `(5400, 2600)` and carries `object-moved { path: [12 points], to: (5400, 2600) }`.
The disc runs along the 12 points, 0.1 s each, and stops exactly on `(5400, 2600)`.

The shape is read exactly as `object_view.gd:232-247` reads it: `{ "ball": <radius mm> }` or
`{ "box": { x, y, z } }` half-extents (`systems/bodies/src/component.rs:402-416`). A shape the client
cannot read is drawn as a 0.2 m outline square and logged once; it is never skipped, so nothing the world
discloses is hidden.

Height (`at.z`) is not drawn as an offset in `plan` projection. An object on a table is drawn over the
table's footprint; its z-order puts it above furniture footprints (`furnishings.gd FLOOR_Z = -5`) and in
the people's y-sort (`people.gd:202`), so a ball behind a person is drawn behind them.

### 4.2 Motion: Godot `Tween`, adopted

Tools first. The motion is "move a node through N given points, then stop on the last one".

| Option | Fit | Licence | Platforms | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Godot `Tween`** (`create_tween()`, one `tween_property(node, "position", p, 0.1)` per keyframe, `TWEEN_PROCESS_PHYSICS`) | Exact: plays the server's points as given, ends on the last, kills cleanly when a newer move arrives | MIT (core) | all three | a few lines | **Adopt** |
| Godot `Path2D` + `PathFollow2D` + `Curve2D` | Works, but bakes and smooths the curve, so the drawn path is not the server's points; one node set per motion | MIT (core) | all three | more nodes | Reject: changes what the server said |
| `AnimationPlayer` with a generated `Animation` | Works; heavy for a per-event one-shot | MIT (core) | all three | most code | Reject |
| Hand lerp in `_physics_process` (what 3D's `object_view.gd:180-200` does) | Works | own code | all three | own code to keep | Reject for 2D: `Tween` does it with no code of ours. 3D keeps its own; not touched here |

Bounded details (decided now, recorded here, not operator questions):

- Keyframe time is 0.1 wall s, the same `PATH_KEYFRAME` as 3D (`object_view.gd:31`), which is the pack's
  keyframe spacing played back in real time. Wall time only; the world's time scale is never read
  (step-11 SD-E11).
- The fallback ease (no path, e.g. a push by a walking body) is `clamp(distance / 4.0 m/s, glide, 0.8 s)`,
  the 3D numbers (`object_view.gd:24-29`). In 2D the glide is the walker's draw speed for one report.
- A newer move kills the running tween and starts from where the node is drawn now.
- The drawing never moves except toward a position an observation states (QB-7). No prediction.

### 4.3 Picking a drawn object

`objects.gd` gets `pick(at) -> String`, the twin of `people.gd:159`. It returns the object whose drawn
footprint holds the click point, the front-most first. `app.gd:326 open_menu_at` asks `people.pick`
first, then `objects.pick`, then walks. So a person standing on a ball still gets the person's menu.

Distance and footprint tests live in `scripts/scene/**`, which rule R4 of `scripts/check_client_rules.py`
already allows.

### 4.4 The object menu (C2)

`menu.gd` gains a third subject kind, "an object", next to "a person" and "myself".

```text
subject = an object id
offered = observation.affordances("", "")                 # my own target-less offers
          filtered to those whose payload refers to that object (menu.gd:_about already finds it)
entries = in the server's order, each built by the existing _build/_entry (complete → enabled)
        + after each offered `throw` of it: "Throw <name> at…" (kind "aim")
none offered -> no menu; the click walks to the object's disclosed point (walk-to, the server routes
               and stops where bodies allow)
```

Filtering by payload reference is presentation, like the person menu's filter by target: the order and
verdicts stay the server's, nothing is removed that the "myself" menu would show (it still lists them),
and nothing is invented.

The label names the object by the world's name when one is known (`readers.thing`), else by its shape:
"the ball", "the box" (new wording keys `ui.object.ball`, `ui.object.box` in the default pack's
`i18n/en.po`). An item id is never shown as text when its shape is known.

### 4.5 Aimed throw (C2)

`intents.gd` stays the only script that names an action (rules R1, R2):

```text
COMPOSED += "throw"
INPUTS["throw"] = {"aim": true}
func aim_throw(affordance, local: {x, y}) -> String:
    payload = affordance.payload duplicated, payload.toward = local     # object reference unchanged
    return link.client.submit("throw", affordance.target, payload)
```

Flow: the player chooses "Throw the ball at…". `app.gd` enters aim mode: the status line shows the aim
hint (wording key), and the next left click on the floor is converted by `town.local_in(place, plan)` into
integer millimetres of the place's frame and passed to `intents.aim_throw`. Esc, a right click, or a place
change drops the aim; nothing is sent. The answer is the server's: a point off the floor, behind a wall or
too far is sent anyway, and the server throws, clamps or refuses. The client never checks it (`ARC-47` R3).

Example: in the hall of bodies-yard the player stands at `(2000, 3000)` next to `hall-ball`. They click
the ball, choose "Throw the ball at…", click the floor at plan `(6.5, 4.2)` m. The client sends
`throw { object: {entity: "<hall-ball id>", entity_type: "item"}, toward: {x: 6500, y: 4200} }`. The
next observation's `object-moved` path is drawn.

Why compose from the offered `throw` and not by name: the offer carries the object reference exactly as
the world disclosed it, so the client never builds an identity (`object_view.gd:100-106` gives the same
reason). If no `throw` of that object is offered, there is no aim entry. That is the server's reach
(QO-7: objects beyond 800 mm are not offered), seen as an absent entry, not a client rule.

### 4.6 Decision record (ARC-342, coordinator assigns)

One file `docs/decisions/ARC-342.md`: "The 2D client draws loose objects from the disclosed listing, plays
`object-moved` paths with Godot's `Tween`, lists a drawn object's offers in an object menu, and composes an
aimed `throw` by setting `toward` on the offered complete `throw`." It records the §4.2 comparison and
supersedes the step-13 §5.4 NavigationServer2D row (already withdrawn by QN-8). Then
`python3 scripts/check_decision_ids.py --write-index`.

`clients/2d/PRESENTATION.md` §3.1 gains one role row, `object.<shape>` (`object.ball`, `object.box`):
bound → that sprite scaled to the disclosed footprint; unbound → plain drawing (§6 gains one sentence).

---

## 5. Invariants

| Id | Invariant |
| --- | --- |
| I-13d-1 | No rule in the client. Nothing in `clients/2d` decides whether an object can be kicked, thrown, reached, or where it lands. `scripts/check_client_rules.py` (R1–R7) passes with `throw` in `COMPOSED`. |
| I-13d-2 | Drawn = disclosed. At rest, every drawn object is at its disclosed `at` within 1 mm (plan), and every listed object is drawn. |
| I-13d-3 | No world key. No `.gd` file names an object, item, place or world id (`court-ball`, `hall`, `bodies-yard`). Everything is read from the listing. |
| I-13d-4 | One request path. The object menu sends complete offers through `intents.submit_offered`; the aimed throw through `intents.aim_throw`. Nothing is resent. |
| I-13d-5 | The client is removable (step-13 I-4): no file outside `clients/2d/**`, `presentation/mineworld-default/2D/**`, `tools/cli/tests/client_2d*.rs`, `tools/cli/tests/godot2d/**`, `docs/decisions/**`, and this plan changes. `check_client_rules.py --scope origin/main` passes. |
| I-13d-6 | Plain drawing is complete: every AC passes with `--presentation=none`. |

---

## 6. Acceptance criteria (fixed before anything is measured)

Worlds: **W1** `worlds/bodies-yard`; **W2** `worlds/social-cafe` (the café's `cafe-ball` and `cafe-box`,
reached by the existing `--drive=walk` path); **W3** a generated variant of bodies-yard written by the test
into a scratch directory, with every loose object shifted by +700 mm in x and its places' ids renamed, so a
client that remembered W1's positions or ids fails.

| AC | Observable condition | Worlds |
| --- | --- | --- |
| AC-13d-1 | After settling, the client's drawn object set equals the place's `loose-objects` ids, and each drawn position is within 1 mm of its disclosed `at`. Printed as `EVIDENCE {"objects": …}`. | W1, W2, W3 |
| AC-13d-2 | The harness picks the nearest listed object, walks to it (`walk-to`), clicks it, and chooses the offered `kick`. The request sent equals the offer (`submit_affordance`). The next observation's `object-moved` for it has a `path`; the drawing visits every path point in order (`motions` log, one row per keyframe) and comes to rest within 1 mm of `to`. | W1, W3 |
| AC-13d-3 | The harness clicks a ball, chooses "… at…", clicks a floor point P derived from the disclosed floor (its centre). The sent `throw` has the offered object reference unchanged and `toward` = P in integer mm. The drawn ball ends within 1 mm of the server's `to`, whatever `to` is. | W1, W2 |
| AC-13d-4 | Esc after choosing "… at…" sends nothing (the stub log has no `throw`). | stub |
| AC-13d-5 | A second session submits `shove` at the player. Within 2 s the drawn player is within 150 mm of the observed position (the walker's reconcile), and any walk in progress ended. | W1 |
| AC-13d-6 | The keys are held toward the nearest shell solid for 3 s. The server stops the player; the drawn body ends within 150 mm of the last observed position and never inside the solid's disclosed footprint by more than 1 person radius (250 mm). | W1, W3 |
| AC-13d-7 | Removing `bodies` from a copy of W1's `systems` list: no object is drawn, no object menu opens, the client runs without an error line. | W1 minus bodies |
| AC-13d-8 | `check_client_rules.py` and `--scope origin/main` pass; `grep -rn "court-\|hall-\|bodies-yard" clients/2d/scripts` (outside `harness/`) is empty. | static |
| AC-13d-9 | AC-13d-1…3 pass with `--presentation=none`. | W1 |

---

## 7. Commit plan

### C1 — Draw loose objects and play their paths

- Goal: objects are visible and move as the server says (AC-13d-1, -2's drawing half, -7, -9).
- Scope: new `clients/2d/scripts/scene/objects.gd`; `clients/2d/scripts/app.gd` (setup, reconcile call);
  `clients/2d/PRESENTATION.md` §3.1, §6; `presentation/mineworld-default/2D/assets/asset_bindings.yaml`
  only if Q-13d-4 says bind.
- Non-goals: no menu, no picking, no request.
- [x] implementation: `objects.gd` per §4.1–4.2 (listing reader, build by shape, `Tween` path, fallback
      ease, free on unlist / place change, `motions` log for the harness, `drawn_local(id)`)
- [x] implementation: `app.gd` wiring after `furnishings.reconcile`
- [x] implementation: PRESENTATION.md role row and plain-drawing sentence
- [ ] validation: one stub test in `tools/cli/tests/client_2d_interact_stub.rs`: a stub world lists two
      objects, then moves one with an `object-moved` carrying a 3-point path, then unlists the other. The
      drive's evidence shows two drawn, then the path visited in order ending on `to`, then one drawn.
      Run through `cast-slots/with-slot.sh` (Godot-gated, `--ignored`, this one test only)
- [ ] validation: `python3 scripts/check_client_rules.py`
- [ ] review: drawn = disclosed holds; no prediction; no id literal
- Failure cases: an unreadable shape → outline square plus one log line; a listing that is not an array →
  nothing drawn, no error; an event for an object not listed → ignored.

### C2 — Object menu and aimed throw

- Goal: the player can act on what they see (AC-13d-2's request half, -3, -4).
- Scope: `objects.gd` `pick`; `app.gd` `open_menu_at`, aim mode, Esc; `menu.gd` object subject and `aim`
  kind; `intents.gd` `COMPOSED`, `INPUTS`, `aim_throw`; `hud/readers.gd` object label by shape;
  `presentation/mineworld-default/2D/i18n/en.po` (and any mirror locale present) keys `ui.object.ball`,
  `ui.object.box`, `ui.throw-at`, `ui.aim.hint`.
- Non-goals: no change to the person or self menus' entries (the self menu still lists kick/throw).
- Depends on: C1.
- [ ] implementation: picking order person → object → walk
- [ ] implementation: object subject in `menu.gd` (filter by `_about`, server order, "at…" entry after
      each offered `throw`)
- [ ] implementation: `intents.aim_throw`; aim mode in `app.gd`; drop on Esc / right click / place change
- [ ] implementation: no offer → walk to the object's point
- [ ] validation: stub test: an object with offered `kick` and `throw`; the harness clicks it, chooses
      the aim entry, clicks a floor point; the stub log holds `throw` with the offer's object reference and
      `toward` the clicked mm. Then a second aim, Esc: no second `throw` (AC-13d-4)
- [ ] validation: `check_client_rules.py` (R2 with `throw` composed; R3 menu reads no verdict on choose)
- [ ] review (Opus): no verdict read where an entry is chosen; no client range check
- Failure cases: the object unlisted while aiming → aim dropped, nothing sent; the click lands on a
  person while aiming → still a floor point (aim takes the click), documented in the hint.

### C3 — The real-world runs

- Goal: AC-13d-1…3, -5, -6, -7 against real servers on W1, W2, W3.
- Scope: new `clients/2d/scripts/harness/bodies.gd` (scenario `--drive=bodies`, dispatched from
  `harness/drive.gd:_run`); new `tools/cli/tests/client_2d_bodies.rs` (all `#[ignore]`, Godot-gated, like
  `client_2d.rs`); W3 and "W1 minus bodies" built in the test's scratch dir (the `newsstand.rs:24
  kiosk_yard` pattern: copy, edit YAML text in Rust, never touch `worlds/`).
- Every harness target is derived from disclosed data: the nearest listed object, the floor's centre, the
  nearest shell solid. None is a constant of a world.
- [ ] implementation: `bodies.gd` steps: settle → objects evidence → walk to nearest → kick → aim throw →
      hold keys into a shell solid → wait for the shove
- [ ] implementation: `client_2d_bodies.rs`: one test per world (W1 full; W2 walk-in then throw; W3
      kick and wall; W1-minus-bodies quiet run); the shove from a second `Client` session
- [ ] validation: run the W1 test once locally through `with-slot.sh`; W2, W3 run at the milestone
      (charter: full runs at milestones)
- [ ] review: no world literal in `bodies.gd` beyond what it read; thresholds are the plan's (§6), not
      read from code under test
- Failure cases: no object in the starting place → the harness walks through the first disclosed
  doorway once and retries; still none → FAIL with the reason (never a silent pass).

### C4 — Docs close the row

- [ ] `docs/decisions/ARC-342.md` + `check_decision_ids.py --write-index`
- [ ] `step-13-client-2d.md`: §5.4 row at line 441 marked "superseded by QN-8 (server routing)"; §9 13d
      row status "delivered (PR 13d)"; RK-b2's "aimed throw is listed unsupported until 13d" closed
- [ ] `clients/2d/README.md`: one row for objects, kick and aimed throw
- [ ] this design: progress, evidence, deviations

---

## 8. Files touched

| File | Commit | New? |
| --- | --- | --- |
| `clients/2d/scripts/scene/objects.gd` | C1, C2 | new |
| `clients/2d/scripts/app.gd` | C1, C2 | edit |
| `clients/2d/scripts/menu.gd` | C2 | edit |
| `clients/2d/scripts/intents.gd` | C2 | edit |
| `clients/2d/scripts/hud/readers.gd` | C2 | edit |
| `clients/2d/scripts/harness/bodies.gd`, `harness/drive.gd` | C3 | new, edit |
| `clients/2d/PRESENTATION.md`, `clients/2d/README.md` | C1, C4 | edit |
| `presentation/mineworld-default/2D/i18n/*.po` (and `asset_bindings.yaml` only per Q-13d-4) | C2 | edit |
| `tools/cli/tests/client_2d_interact_stub.rs` | C1, C2 | edit |
| `tools/cli/tests/client_2d_bodies.rs` | C3 | new |
| `docs/decisions/ARC-342.md`, decisions index | C4 | new, edit |
| `.structured-coding/plans/mvp0/step-13-client-2d.md`, this file | C4 | edit |

---

## 9. Test ownership

| Failure class | Owner | Why not elsewhere |
| --- | --- | --- |
| A listed object is not drawn, drawn wrong, or left drawn after it is gone | C1 stub test | Deterministic; a real world cannot unlist on demand |
| A path is not played in order or does not end on `to` | C1 stub test (exact points); C3 W1 (real physics) | The stub owns ordering; the real run owns "the pack's path shape is read right" |
| The aimed throw's payload is wrong (reference rebuilt, mm wrong, frame wrong) | C2 stub test | Payload equality is exact only against a stub log |
| Esc still sends | C2 stub test | Only a stub can prove absence |
| A world-specific assumption | C3 W3 (generated variant) | Only a changed world exposes it |
| The client decides a rule | `check_client_rules.py` (static) | Static scan owns code properties |
| Wall stop and shove reconcile | C3 W1 | Needs the real `bodies` pack |
| Pack removed | C3 W1-minus-bodies | Needs a real composition |

No unit test of `Tween` itself, and no test that `objects.gd` stores what it was given.

---

## 10. Risks

| Id | Risk | Response |
| --- | --- | --- |
| R-13d-1 | A ball under a person's sprite is hard to click | Picking order person → object is deliberate; the self menu (E) still lists every kick/throw. Recorded, not patched |
| R-13d-2 | `plan` projection hides height: a ball on a table looks on the floor | Accepted for 2D; z-order keeps it above the table's footprint |
| R-13d-3 | The path is long and the next observation arrives before it is played | A newer move kills the tween and starts from the drawn point; the drawing always ends on the newest `at` |
| R-13d-4 | W2's walk into the café meets furniture | The walk is the server's (`walk-to`); a failure is a server or harness finding, never a client rule |
| R-13d-5 | Godot-gated tests are not in `fast` CI | Charter: they run at the milestone; C3 runs W1 once locally |

---

## 11. Freeze questions (each with a recommended answer)

| Id | Question | Recommended answer |
| --- | --- | --- |
| Q-13d-1 | The 13d row is mostly delivered (walls, server routing, complete kick/throw/shove). Shrink 13d to this closing PR: objects drawn and moved, object menu, aimed throw, three-world proof? | **Yes.** The rest is on `main` (§2 evidence). |
| Q-13d-2 | Aimed throw: compose it by copying the offered complete `throw` and setting `toward` to the clicked point (so the client never builds the object reference)? | **Yes.** Same as 3D; the server still decides the throw. |
| Q-13d-3 | Clicking a drawn object opens a menu of only the offers that name it; with none, the click walks toward it? | **Yes.** The E menu still lists everything. |
| Q-13d-4 | Art for objects: plain drawing only (disc / rectangle), with an optional `object.<shape>` sprite role the pack may bind later? | **Yes, plain now, role defined, no new art or spend.** |
| Q-13d-5 | Prove on bodies-yard, social-cafe and a test-generated bodies-yard variant (objects moved, ids renamed); full runs at the milestone, W1 once locally? | **Yes.** |

---

## 12. Execution contract

```text
Frozen revision:               revision 1
Frozen by / date:              the operator, primary session, 2026-10-20 (every §11 question as recommended)
Implementation base SHA:       origin/main @ fec906f2 (branch mvp0/pr-13d, worktree impl-13d)
Authorized commits:            C1 … C4 as in §7
Authorized files:              §8 only, plus the bounded additions recorded in §13
Local validation budget:       the build, the stub tests this PR writes, one W1 run (charter: SPEED FIRST;
                               Godot and cargo only through cast-slots/with-slot.sh)
Milestone validation:          W2, W3, W1-minus-bodies and the full Godot-gated suites
Merge path:                    coordinator (charter); merge freeze on main, the PR queues
Stop conditions:               any change outside §8; any client-side rule; any freeze answer changed
```

---

## 13. Progress and evidence

- 2026-10-20: revision 1 drafted from the §2 audit at `origin/main @ 6fe45d58`. No code changed.

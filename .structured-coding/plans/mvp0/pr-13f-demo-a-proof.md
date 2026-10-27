# PR 13f — Demo A proof and the VIS-2D-1 package (S12, last PR)

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:   revision 1 (2026-10-20), DRAFT
Author:            design sub-agent of the clients lane (Opus), worktree
                   /Users/yuema137/mineworld-worktrees/design-13f-16e, branch docs/pr-design-13f-16e
Planning base:     origin/main @ f92e3cad (2026-10-20)
Implementation base: origin/main at C0, after the start condition holds
Start condition:   13d and 13e merged (both DRAFT at 2026-10-20, drafted in parallel); this design
                   frozen by the operator; the lane lead's dispatch
Sibling design:    pr-16e-demo-b-proof.md (S14 16e). The two share the "one world, two views"
                   parity test; Q-PAR-1 decides which PR hosts it
Execution contract: §11 (empty until the freeze)
Implementer:       Haiku-class for C1–C3 (well specified); the primary session queues the package
Decision ids:      ARC-346, one record: §4.4, the Demo A acceptance run
Lifecycle:         DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-13-client-2d.md`](step-13-client-2d.md) §9 (the 13f row), §4.9 (how
the operator plays it), §8.2 (the adversarial check), §1.3 (`AC-3`, `AC-13`, `AC-15`) · **Parent:**
[`overall.md`](overall.md) §3 S12, "One world, two views" rule 3 · **Specs:** `docs/MVP.md` §7.1, §9;
`docs/ACCEPTANCE.md` §§4.1, 6, 7.

---

## 0. The row, quoted

> **13f — Demo A proof and VIS-2D-1 package** | The Demo A acceptance run (every item of `MVP.md` §7.1,
> scripted, two clients and agents, SIGKILL of a client and of the server, resume); the operator's
> runnable checklist; the VIS-2D-1 review package (`ACCEPTANCE.md` §6) for the primary session to
> queue. | **Waits** on 13a–13e. | The checklist of §4.9 run end to end by the harness and by hand;
> persisted revision identical before and after a server restart; `AC-15`'s evidence lines read from
> the 2D side. | The I-4 path scan over the whole step's client merges; §8.2 re-run on the final head.

And `overall.md`, "One world, two views", rule 3: a parity test "owned by S14 16e together with S12 13f".

---

## 1. Goal, in plain words

Demo A is done when one command shows that the 2D client can play a whole living world, and a test
proves the same thing without a person. Most pieces already exist as separate tests. This PR:

1. puts them together into one **Demo A run**: one test file that plays every item of `MVP.md` §7.1
   on **two worlds** (market-town and social-cafe), with two 2D clients and hosted agents, and kills
   a client and the server on the way;
2. makes the 2D client's `SHOWN` report complete enough for the parity test (doorways read from what is
   drawn, loose objects, the menus), so 16e can compare it with the 3D client;
3. writes the operator's short checklist and the VIS-2D-1 review package;
4. re-runs the step's two adversarial checks on the final head.

Example. Today `client_2d_interact.rs` proves "buy a coffee" and `client_2d.rs` proves "kill the client
mid-walk". Nobody has run them as one story on one world, and nobody has run them on social-cafe. After
this PR, `cargo test -p mineworld-cli --test demo_a -- --ignored` plays the story on both worlds.

---

## 2. Audit — what already exists (origin/main @ f92e3cad)

| Demo A item / row clause | Exists today | Evidence | What remains for 13f |
| --- | --- | --- | --- |
| Multiple Persons, Places, movement | Walk apartments → street → café through two doorways, every stride accepted, people drawn at the server position | `tools/cli/tests/client_2d.rs:59` (`walks_from_the_apartments_into_the_cafe`), `:150` | Only on market-town. Social-cafe has no 2D walk test |
| Conversation | Talk to Alice from the door (refused far, accepted near), reply read from history | `client_2d_interact.rs:63` | Only market-town |
| Relationships | Acquaintances panel after talking | `client_2d_interact.rs:63` (PANEL lines; 13b §15) | Not asserted on a second world |
| Basic items | buy, drink, give, eat; a removed pack offers no `give` | `client_2d_interact.rs:215`, `:367` | Market-town only. Social-cafe's items are loose objects (`worlds/social-cafe/world.yaml:98–101`): kick and throw need **13d** |
| Basic group activity | Three clients invite, accept, join | `client_2d_interact.rs:496` (`three_clients_do_something_together`) | Market-town only |
| Persistence, `AC-3` end to end | Client SIGKILLed mid-walk and relaunched; server SIGKILLed, restarted, client reconnects; the revision advanced while the client was dead | `client_2d.rs:256`, `:372`, asserts at `:327`, `:345` | The row's "persisted revision identical before and after a server restart" is not asserted against the save (Q13f-3) |
| Human and agent controllers | Server-side takeover (`AC-5`) and hosted town (`--town`, R-S10-1) | `ac5_takeover.rs:196`; `hosted_town.rs:30`; `tools/cli/src/serve.rs:77` | Takeover seen from the 2D client is **13e** (R-S11-3 "13e shown, 13f asserted") |
| Multiplayer | Four Rust connections meet one Alice | `ac7_many_players.rs:201` | Two real 2D clients that see each other is **13e**'s checkpoint; 13f re-runs it inside the Demo A story |
| `AC-13`, 2D half | The 2D talk equals the protocol demo's "3D" flavour; buy/drink/give/eat equal the real 3D client's | `client_2d_interact.rs:432`; `client_3d_market.rs:668` | Nothing for 13f. The real-3D comparison of talk, shove, kick, throw is **16e** |
| `AC-15`, 2D side | Rust connections play the "2D window" | `ac15_one_alice.rs:80` | The 2D client prints `instance` in `EVIDENCE self` (`clients/2d/scripts/harness/drive.gd:496`), but no line names Alice's id, the event sequence or the persisted revision. 13f adds that line; 16e uses it |
| `SHOWN` (parity report, 2D) | `place`, `places`, `doorways`, `doorway_labels`, `people` | `drive.gd:462–484` | (a) doorways come from `app.town.passages`, the **layout model**, not the drawn scene, so removing a drawn doorway would not change the report; (b) no loose objects; (c) no affordances (they are in separate `MENU` lines) |
| `SHOWN` (parity report, 3D) | Same keys, read from the scene | `clients/3d-spike/scripts/slice/slice_layout.gd:157–182`, `slice_link.gd:716–727` | 16e's side |
| Parity test | Planned, not written: `tools/cli/tests/shown_parity.rs`, places and doorways only, Lakeside and Social Café | `pr-s33h-2d-parity.md` §5.4 (FROZEN, `ARC-174`; S33-h blocked on S29-f, `progress/lanes/s33.md`) | Q-PAR-1 |
| Cross-client identity | `IDENTITY` lines of 2D and 3D compared, on two worlds, on separate fresh servers | `clients/shared/checks/identity.sh` | Reused as is; not one running world |
| §8.2 adversarial check | Static scan R1–R6 and path scope | `scripts/check_client_rules.py` (`--scope BASE`); stub tests `client_2d_interact_stub.rs` | Re-run on the final head (row) |
| Nightly | Godot tests one by one | `scripts/ci_layer.py:176` (`clients`), `scripts/ci_godot.py:78` (`GODOT_TESTS`) | Add `demo_a` |
| Stills | 14 preview stills from 13a/13b | `clients/2d/shots/preview/01…14` | Re-capture on the final head (13d/13e change the look); add social-cafe |
| VIS-2D-1 | "🚧 candidate in progress" | `docs/HUMAN_REVIEW_QUEUE.md:143` | The package (`ACCEPTANCE.md` §6) |
| Status pages | "2D client: presentation spike runnable, awaiting style decision" | `docs/MVP_STATUS.md:52`, `:114` | Stale; corrected here |

Summary. The parts are there. What is missing is the one story on two worlds, three report fields,
the identity line, the package, and the status pages.

---

## 3. Dependencies (not redesigned here)

| On | What 13f needs from it | If it is missing at C0 |
| --- | --- | --- |
| **13d** (DRAFT, parallel) | 2D kick, throw (complete) and shove through the menu; loose objects drawn from `loose-objects` | The social-cafe "items" step cannot run. 13f does not start |
| **13e** (DRAFT, parallel) | Join screen and seat picker; speech bubbles from events; takeover message; two 2D clients see each other | The multiplayer and takeover steps cannot run. 13f does not start |
| 13d's `SHOWN` | If 13d already adds `objects` to `SHOWN`, C1 does not add it again | C1 adds it |
| **16e** | Nothing. 13f must not wait on 16e, because 16e waits on S12's client |  |
| S33-h | Nothing. If `shown_parity.rs` lands first, it is the file 16e extends (Q-PAR-1) |  |

---

## 4. Design

### 4.1 One story, two worlds, existing harness

The Demo A run is one Rust test file, `tools/cli/tests/demo_a.rs` (`#[ignore]`, needs Godot). It writes
no new Godot scenario. It reuses:

- `godot2d::World` (start, `kill`, `restart`), `godot2d::Drive` (start, `until`, `kill`, `finish`),
  `godot2d::worlds::play` and `steps_file` (13b's selector steps), `tagged`, `evidence`;
- the existing drive scenarios (`street`, `walk`, `steps`, `panels`) and 13d/13e's scenarios;
- the save's fact log and `GET /status` as oracles (never the client's `[PASS]` alone).

Each step of the story is written **from the world's disclosure**, not from the world's name. Example:
"talk to someone" picks the first person the observation lists in the café with a complete or
composable `talk` offer; it does not say "Alice". "Use an item" picks the first complete offer whose
type is in the observer's own menu and whose target is an item or loose object. So the same code plays
market-town (a coffee is bought) and social-cafe (a ball is kicked).

```text
the story (each line is one assertion group, run on market-town and on social-cafe)
1  host the world with --town (agents drive every unoccupied seat) and --save
2  client A joins seat S1; client B joins seat S2 (13e's join path)
3  A walks out of its start place through a disclosed doorway into another place      places, movement
4  A sees at least two people, each labelled with the disclosed name                  multiple Persons
5  A talks to a person there; the reply is in A's own history                         conversation
6  an acquaintance appears in A's panel                                               relationships
7  A uses one complete item offer from its menu; the save records the fact            basic items
8  A invites, B accepts and joins; both are drawn with the participation marker       group activity
9  B sees A walk, and sees A's speech bubble                                          multiplayer
10 an agent-driven person moves on its own while nobody drives it                     agent controller
11 B joins a seat an agent drives; the takeover message shows; its biography is a
   prefix of the one before                                                           human takeover
12 A is SIGKILLed mid-walk; the revision advances; A relaunches and is drawn where
   the server last put it                                                             AC-3
13 the server is SIGKILLed; restarted on the same save; both clients reconnect to the
   same instance (Q13f-3 fixes the revision rule)                                     persistence
14 A prints its IDENTITY line (§4.3); its fields equal /status and the save           AC-15, 2D side
```

### 4.2 The 2D `SHOWN` report, completed

Three changes in `clients/2d/scripts/harness/drive.gd` `_report_shown` (and a reader in `scene/` if
needed). No change to how anything is drawn.

1. **Doorways from the drawn scene.** Today the list is read from `app.town.passages` (the model). It
   becomes the doorways that have a drawn façade or door anchor (`app.places.door_labels()` and the
   façade anchors already checked by `_check_facades`, `drive.gd:450–459`). A doorway in the model with no drawn anchor is
   not reported. This is what makes "remove a doorway from one client's scene" visible.
2. **Loose objects.** `objects: [{id, kind}]` for every loose object drawn, if 13d has not added it.
3. **Affordances.** `affordances: [{action_type, target, about, complete, available, reason}]` for the
   observer's own menu and for each drawn person's menu, read from the menus the client built (13b's
   `MENU` entries). Labels and `enabled` stay out: they are presentation.

A harness-only flag, `--drop-doorway=<from>,<to>`, removes that doorway's drawn anchor after the scene
is built (Q-PAR-3). It lives in `scripts/harness/`, never in the play path; `check_client_rules.py`
already confines harness code.

Which fields are presentation-only is declared once, in `clients/protocol/ADOPTION.md` (a new short
section "What a client reports it shows"), shared with 16e. Proposed list: `drawn_as`, `look`, `role`,
`activity`, `label`, `local`, `doorway_labels`, a menu entry's `label` and `enabled`. Everything else
must be equal between the clients.

### 4.3 The 2D side of `AC-15`

One new line from the drive, printed once seated and again on request:

```text
IDENTITY_WORLD {"instance": "<id>", "revision": <n>, "people": {"<id>": "<disclosed name>", …},
                "events_seen": ["<EventId>", …]}
```

`instance` and `revision` from the welcome and the newest frame; `people` from the observation;
`events_seen` from `observation.events` (R-S11-4), in arrival order. The Demo A test checks each field
against `GET /status` and the save. 16e reads the same line next to the 3D client's.

### 4.4 Decision record (ARC-346, coordinator assigns)

"The Demo A acceptance run is `tools/cli/tests/demo_a.rs`: the `MVP.md` §7.1 list played as one story on
market-town and social-cafe, from disclosure only, by the real 2D client; it is the S12 checkpoint."

---

## 5. Acceptance — decided before measuring (`ARC-23`)

Every row names two worlds. Bounds come from the requirement, never from a measurement.

| ID | Criterion | Worlds |
| --- | --- | --- |
| AC-f-1 | `demo_a` story steps 1–14 (§4.1) pass; every assertion's oracle is the save, `/status` or another connection, not the client's own `[PASS]` alone | market-town, social-cafe |
| AC-f-2 | Step 7 plays a different item kind per world, chosen from the menu: market-town a `buy` (wallet falls by the listed price), social-cafe a `kick` (the object's disclosed position moves) | both |
| AC-f-3 | Step 13: the save's facts before the kill are a prefix of the facts after the restart; the welcome after restart names the same instance; its revision obeys Q13f-3's rule | both |
| AC-f-4 | `SHOWN` lists each doorway once per drawn anchor; with `--drop-doorway=<from>,<to>` the report lacks exactly that doorway and the drive still passes otherwise | both |
| AC-f-5 | `SHOWN.objects` equals the loose objects of the newest observation; `SHOWN.affordances` equals the observation's offers for each drawn subject (set equality, order kept) | social-cafe (objects), market-town (offers) |
| AC-f-6 | `IDENTITY_WORLD` equals `/status`'s instance and the save's revision at the same moment; Alice's id is the save's Alice | both |
| AC-f-7 | `check_client_rules.py` → 0 findings on the final head; `--scope <merge-base of 13a>` over the step's client merges → 0 paths outside the allowed set; one planted violation each turns it red | — |
| AC-f-8 | §8.2 parts 2–5 (`client_2d_interact_stub.rs`, the unknown pack, the removed pack) pass on the final head | — |
| AC-f-9 | Stills re-captured on the final head, for both worlds: street, an interior, a crowd, the reference-framed still; 0 lines matching `ERROR|SCRIPT ERROR|Failed loading` (`ACCEPTANCE.md` §4.1) | both |
| AC-f-10 | The operator's checklist (§12) is runnable from a clean checkout as written | both |

---

## 6. Commit plan

Each commit has three separate items: implementation, validation, review. `[x]` needs the work and the
evidence. Local heavy runs go through `cast-slots/with-slot.sh`; the full Godot suite runs at the
milestone (charter, R-LOAD-1).

### C0 — Design (this file), docs only
- [ ] Implementation: this file; pointer in the step's row.
- [ ] Validation: `scripts/check_doc_headings.py` on the file.
- [ ] Review: the operator's freeze.

### C1 — The 2D report: doorways as drawn, objects, affordances, `IDENTITY_WORLD`, `--drop-doorway`
- **Goal:** the 2D client reports what it draws, so the parity test and the Demo A run can trust it.
- **Scope:** `clients/2d/scripts/harness/drive.gd` (`_report_shown`, new `_report_identity`), possibly a
  reader in `clients/2d/scripts/scene/places.gd`; `clients/protocol/ADOPTION.md` (the shared
  presentation-only list). Non-goals: no drawing change, no new scenario, no menu change.
- [ ] Implementation: §4.2 items 1–3; §4.3; the harness flag; the ADOPTION section.
- [ ] Validation: the existing `client_2d.rs` tests that read `SHOWN` still pass
  (`client_2d.rs:87`, `client_2d_interact.rs:50`, `:617`); one targeted run per world of
  `--drive=walk` and `--drive=walk --drop-doorway=…`, the dropped doorway absent; `check_client_rules.py`
  0 findings. Counterfactual: read doorways from the model again → AC-f-4 fails.
- [ ] Review: an LLM reviewer reads the diff against I-1, I-4, I-7 and the ADOPTION list.
- **Failure cases:** a doorway with no anchor (a place not yet drawn) is not reported, and that is
  correct; `--drop-doorway` naming a doorway that does not exist fails the drive by name.
- **Commit boundary:** client harness and one spec section only.

### C2 — `demo_a.rs`: the story on two worlds
- **Goal:** AC-f-1 … AC-f-3, AC-f-5, AC-f-6.
- **Scope:** `tools/cli/tests/demo_a.rs` (new); small helpers in `tools/cli/tests/godot2d/worlds.rs` only
  if two tests need them; `scripts/ci_godot.py` `GODOT_TESTS` gains `demo_a`. Non-goals: no edit to
  existing tests' assertions.
- [ ] Implementation: §4.1 steps 1–14, each a function taking the world's ids from genesis
  (`godot2d::worlds::ids`) and choices from disclosure.
- [ ] Validation: `cargo test -p mineworld-cli --test demo_a -- --ignored --test-threads=1` once per
  world; record counts and wall time. Counterfactuals: (a) skip the client relaunch → step 12 fails;
  (b) restart the server on an empty save → step 13 fails naming the instance.
- [ ] Review: the reviewer checks that no step names a world's person, place or item literally.
- **Failure cases:** a world with no complete item offer in reach fails step 7 by name (it is a content
  gap, not skipped); a hung client is killed at `RUN_LIMIT` and reported, never passed.

### C3 — Adversarial re-run, stills, the package, status pages
- **Goal:** AC-f-7 … AC-f-10; the row's last two clauses.
- **Scope:** `clients/2d/shots/demo-a/<world>/` (new stills, `.gdignore`); `docs/HUMAN_REVIEW_QUEUE.md`
  (VIS-2D-1 package and a "Demo A" framework row); `docs/MVP_STATUS.md:52`, `:114`; `clients/2d/README.md`
  one line; this file's ledger.
- [ ] Implementation: run the scans and stub tests; capture stills with `./mineworld-2d --drive --capture`
  per world; write the package per `ACCEPTANCE.md` §6 (launch command, what to inspect, stills, the
  references `presentation/mineworld-default/2D/references/`, the reference-framed still, known
  limitations as facts, the questions).
- [ ] Validation: the planted violations of AC-f-7 recorded red then green; stills listed with sizes;
  the error grep at 0.
- [ ] Review: the reviewer runs §12's checklist from a clean checkout and reports each line.

---

## 7. Files touched

| Path | Commit | Kind |
| --- | --- | --- |
| `clients/2d/scripts/harness/drive.gd` | C1 | edit |
| `clients/2d/scripts/scene/places.gd` (only if a reader is needed) | C1 | edit |
| `clients/protocol/ADOPTION.md` | C1 | one section |
| `tools/cli/tests/demo_a.rs` | C2 | new |
| `tools/cli/tests/godot2d/worlds.rs` | C2 | small additions |
| `scripts/ci_godot.py` | C2 | one list entry |
| `clients/2d/shots/demo-a/**` | C3 | new stills |
| `docs/HUMAN_REVIEW_QUEUE.md`, `docs/MVP_STATUS.md`, `clients/2d/README.md` | C3 | edits |
| `docs/decisions/ARC-346.md` + index | C2 | new record |

Nothing under `kernel/`, `contracts/`, `systems/`, `server/`, `worldpack/`, `persistence/`, `worlds/`
(I-4).

## 8. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Godot + real server | the Demo A story, two worlds | `demo_a.rs` (`#[ignore]`), nightly `clients` |
| Godot + real server | the 2D report's completeness (AC-f-4, AC-f-5) | `demo_a.rs` |
| Static | no rule in the client; path scope | `check_client_rules.py` (unchanged) |
| Godot pair | 2D = 3D shown | **not here**: 16e's `shown_parity.rs` (Q-PAR-1) |
| Godot pair | `AC-13`, `AC-15` with real 3D | **not here**: 16e |

Justified overlap: none. Existing `client_2d*.rs` tests stay as they are; `demo_a` composes the same
paths into one story and adds the second world.

## 9. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-f-1 | The story is long and flaky on a loaded machine | Each step waits on a disclosed fact with a fixed limit, never on wall time; one world per test function |
| R-f-2 | 13d or 13e change the harness lines 13f reads | C0 re-audits both merges before C1 |
| R-f-3 | Social-cafe has no talkable person near the start place | Step 5 walks to the first place where the observation lists a `talk` offer; if none exists in the world, that is a content defect, reported |
| R-f-4 | Hosted agents move people between two reports | Steps compare against the frame the client printed beside its report, not against a later frame |

## 10. Freeze questions

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-PAR-1** | Which PR hosts the "one world, two views" parity test? | **16e hosts it**, in `tools/cli/tests/shown_parity.rs` (the file S33-h's frozen §5.4 names, `ARC-174`). 13f only makes the 2D report complete (C1). Whichever of 16e and S33-h lands first creates the file; the other adds its world. Reason: the test needs both clients, and 16e already waits on S12's client, so 13f cannot wait on 16e. |
| **Q-PAR-3** | How is "remove a doorway from one client's scene" done? | **A harness-only flag `--drop-doorway=<from>,<to>` in both clients**, so the negative case is a standing test, not a one-off edit. It is outside the play path. |
| **Q13f-1** | Where does the Demo A run live? | **A new `tools/cli/tests/demo_a.rs`** that reuses `godot2d` helpers and existing drive scenarios. No existing test is edited. |
| **Q13f-2** | Which worlds? | **market-town and social-cafe.** Lakeside joins when its 2D bindings land (S33-h). Items are chosen from the menu: buy in market-town, kick in social-cafe. |
| **Q13f-3** | The row says "persisted revision identical before and after a restart", but hosted agents keep the clock moving. What exactly is checked? | **Three facts:** the save's facts before the kill are a prefix of the facts after; the restarted welcome names the same instance; its revision is ≥ the save's last revision read by `mineworld inspect` between kill and restart. |
| **Q13f-4** | Does the VIS-2D-1 package wait for the parity test? | **No.** It is a taste review of the 2D client; parity is 16e's. |

## 11. Execution contract (filled at the freeze)

```text
PROJECT / PR:            MineWorld mvp0, S12 PR 13f — Demo A proof and VIS-2D-1 package
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-13f-demo-a-proof.md
BRANCH / WORKTREE:       mvp0/pr-13f, /Users/yuema137/mineworld-worktrees/impl-13f
IMPLEMENTATION BASE:     mvp0/pr-13d @ 7d5b6530 (PR #423, not merged: merge freeze on main);
                         13e merged in when its PR is open and green (lane lead's dispatch)
FROZEN BY / DATE:        operator, 2026-10-20 (every question as recommended)
AUTHORIZED COMMITS:      C1, C2, C3 of §6; ledger and evidence updates
VALIDATION BUDGET:       charter SPEED FIRST: the build and the tests this PR writes, through
                         cast-slots/with-slot.sh; full suites at the milestone
MATERIAL STOPS:          a frozen invariant, a public contract or scope; 13e absent for its steps
MERGE:                   by the coordinator, after the operator's review
```

## 12. The operator's checklist (short, runnable)

```text
1  ./mineworld-2d --world worlds/market-town --seat visitor
   walk out of the apartments, into the café: the door labels name the places
2  click Alice, talk: her reply appears; the acquaintances panel lists her
3  buy a coffee: the wallet falls by the price; give it to Bob
4  a second window on the same server (the first prints its join line: address and invite):
   ./mineworld-2d --server=<address> --invite=<invite> --seat=bob
   each window sees the other walk and speak
5  close the first window mid-walk, reopen it: you stand where you were
6  stop the server (Ctrl-C), start it again with the same --save: both windows find the same world
7  ./mineworld-2d --world worlds/social-cafe: walk into the café, kick the ball, invite someone
8  the proof without you: cargo test -p mineworld-cli --test demo_a -- --ignored --test-threads=1
```

## 13. Ledger (empty until implementation)

| Commit | SHA | Validation evidence | Review verdict |
| --- | --- | --- | --- |
| C1 — the 2D report | `74c551bf` | `godot --import` then `--check-only res://scripts/app.gd`: 0 `SCRIPT ERROR`; `check_client_rules.py` PASS (0); exercised by `demo_a` below on both worlds | LLM review N/A (charter SPEED FIRST: no per-PR reviewer); self-read against I-1, I-4, I-7 |
| ARC-346 + contract | `eb232acc` | `check_decision_ids.py --write-index`: 217 ids, all distinct | — |
| C2 (part) — `demo_a.rs` report tests | `d2b45d6c` | `cargo test -p mineworld-cli --test demo_a -- --ignored --test-threads=1`: 2 passed, 12.2 s wall (2026-10-20, local macOS, Godot 4.7.2). market-town: place 2, doorways [2→8], objects [25 ball, 26 box], 37 offers; `--drop-doorway=2,8` → doorways []; social-cafe: place 2, doorways [2→8], objects [22 ball, 23 box], 20 offers; `--drop-doorway=2,8` → []; `--drop-doorway=0,0` → `[FAIL] drop-doorway 0 -> 0 is no doorway this client learned` on both. Counterfactual (doorways from the model) not run | as above |
| C3 (part) — AC-f-7 | (this commit) | `check_client_rules.py` PASS (0); planted `intents.link.client.submit({})` in `menu.gd` → `R1 submits` FAIL (1), reverted → PASS; `--scope origin/main` PASS (0) after admitting `demo_a.rs` and `ci_godot.py` (13f's own §7 files; bounded correction); planted `tools/cli/tests/planted_outside.rs` → FAIL (1) | — |

| merge 13e | `84aa0057` | `origin/mvp0/pr-13e` @ `6dd43eca` (PR #448, `fast` green) merged; two conflicts resolved by hand (`drive.gd`: both fields kept; step-13 table: 13d's row from this branch, 13e's row from 13e) | — |
| C2 — the story | `1ddd0aec` | `demo_a.rs` `*_demo_a_story`, §4.1 steps 1–14 on market-town (`buy`) and social-cafe (`kick`); harness step forms `{"open": {"offered", "except", "among"}}` and `{"open": "last"}` (`interact.gd`, prints `CHOSE`). **Not compiled, not run** (no local runs while the machine was reserved, 2026-10-21; PR CI's `fast` neither compiles nor runs ignored Godot tests): INCONCLUSIVE until the milestone run. `rustfmt` clean; `check_client_rules.py` 0 | — |
| C3 — package, status pages | `3ba9e4ba` | `HUMAN_REVIEW_QUEUE.md`: the `VIS-2D-1` package (§6 items) and an "S12 Demo A" row; `MVP_STATUS.md` 2D rows; `clients/2d/README.md` one line. Stills and the §8.2 re-run (`client_2d_interact_stub.rs`) need Godot: remaining | — |

| C2 fixes from CI | `88f2fe5a`, `3dd89210`, `dc4f25e1` | Nightly `clients` runs on scratch/13f-clients-nightly were used for evidence. Run 39553569446 (`959a8396`): steps 1–9 passed (linux market-town, mac social-cafe); step 7 failed because the town-driven partner walked off; step 10 failed because nobody walked 1 m. Run 39585933280: `/status` seats are not in the world file's order, so step 5 waited 90 s. Local run on `3dd89210`: step 12 failed because Godot prints the revision as `284.0`. Fixes: the partner is the first seat Person in A's first observation's place, hosted `--agent` (reactive); step 10 reads `walk-started` from the save; client revisions are compared as numbers. **`dc4f25e1`**: local run 2/2 stories passed (214.7 s); `cargo clippy --test demo_a -D warnings` clean; nightly run 39594613270: `demo_a` **4 passed** on clients-linux (223 s) and clients-windows (231 s) | — |
| AC-f-8, AC-f-9 | `19de053d` | `client_2d_interact_stub`: 4/5 passed locally. `the_panels_show_what_is_disclosed` fails because main's `bbe5f5aa` ("the I panel reads every section by name, never raw JSON") left the test expecting raw JSON. `a_removed_pack_offers_no_give` fails because the world now offers `put-down` without the pack. Neither is 13f's. Stills of both worlds in `clients/2d/shots/demo-a/` (JPEG q85, 2.4 MB); the capture log has 0 lines matching `ERROR|SCRIPT ERROR|Failed loading` | — |
| Not 13f's (milestone batch) | — | Present on this stack, in no 13f file: `walk` drive `[FAIL] facing is steady` (walks_from…, the_requests_do_not_depend…, two_d_harness_ignores…); client_settings 2D/3D tests; `reconnects_to_a_restarted_world…`; guide Step 10.6; `inspect.rs:55`; agent `test_the_guide_resource_is_chapter_14` (milestone run 39553557068) | — |

**Superseded remaining note.** **Remaining (2026-10-20).** 13e is not pushed (`git ls-remote origin mvp0/pr-13e`: absent; no PR), so
the Demo A story (§4.1 steps 1–14, AC-f-1/-2/-3) is not yet in `demo_a.rs`: steps 2, 8–9 and 11 need
13e's join path, bubbles and takeover message. C3's stills, VIS-2D-1 package, status pages and §8.2
re-run wait for the final head after 13e. **Bounded discovery:** the report runs need `--hold 0`
(S11-B holds a dropped seat 30 s), recorded in `demo_a.rs`.

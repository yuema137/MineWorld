# PR 16e — Demo B: `AC-13`, `AC-15`, one world shown twice, and the operator (S14)

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:   revision 1 (2026-10-20), DRAFT
Author:            design sub-agent of the clients lane (Opus), worktree
                   /Users/yuema137/mineworld-worktrees/design-13f-16e, branch docs/pr-design-13f-16e
Planning base:     origin/main @ f92e3cad (2026-10-20)
Implementation base: origin/main at C0, after the start condition holds
Start condition:   13d merged (2D kick, throw, shove); 13f C1 merged (the 2D report is complete);
                   16f merged or confirmed not needed for the two worlds (Q16e-5); this design
                   frozen by the operator; the lane lead's dispatch. 13d, 13e, 16f are DRAFT at
                   2026-10-20, drafted in parallel
Sibling design:    pr-13f-demo-a-proof.md (S12 13f). Q-PAR-1 puts the parity test here
Execution contract: §11 (empty until the freeze)
Implementer:       Haiku-class for C1, C4; Opus-class for C2, C3 (two real clients at once)
Decision ids:      ARC-TBD (coordinator assigns), one record: §4.5, the client-removal check
Lifecycle:         DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-15-demo-3d.md`](step-15-demo-3d.md) §13 (the 16e row), §4.8 (`AC-13`
against S12), §4.9 (the operator), §9.1 (the removal check), §11.2 (C-S12-1) · **Parent:**
[`overall.md`](overall.md) §3 S14, "One world, two views" rule 3 · **Specs:** `docs/MVP.md` §§7.2, 9, 9.1;
`docs/ACCEPTANCE.md` §6; [`pr-s33h-2d-parity.md`](pr-s33h-2d-parity.md) §5.4 (`ARC-174`).

---

## 0. The row, quoted

> **16e** Demo B: `AC-13`, `AC-15`, the operator | `--scenario=ac13` and `tools/cli/tests/ac13_clients.rs`
> with S12's transcript (C-S12-1); three windows on one server (S11's server, S12's client, this client,
> a paced agent) with `AC-15`'s four identity lines; the removal check (§9.1); the `HUMAN_REVIEW_QUEUE.md`
> package (§4.9); `MVP_STATUS.md` | **After S12's client and C-S12-1, 16c, 16d** | `ac13_clients` passes
> for talk, shove, kick, throw (complete), buy; `AC-15` evidence recorded; the operator plays the
> checklist | A 2D transcript with another utterance fails naming `payload`; the scratch copy without
> `clients/3d-spike` passes the full Rust gate

---

## 1. Goal, in plain words

Demo B is done when four things are proved with the **real** 2D and 3D clients:

1. **They ask the same thing** (`AC-13`). Talk, shove, kick, throw and buy, each done the 2D way (click)
   and the 3D way (walk up, look, press), send the same request.
2. **They look at one world** (`AC-15`). A 2D window, a 3D window and an agent-driven Alice on one
   server name one world instance, one Alice id, one event sequence, one persisted revision.
3. **They show the same world** (the parity test). For one moment of one world, both report the same
   places, doorways, people, loose objects and offers. Remove a doorway from one client's scene, and the
   test fails.
4. **The 3D client can be deleted** and nothing else breaks.

Then the operator plays a short checklist.

Example of what is missing today. `AC-13` for talk is proved only between two *flavours of a demo
script* (`clients/protocol/demo/demo.gd`), not between the real 2D client and the real 3D client.

---

## 2. Audit — what already exists (origin/main @ f92e3cad)

| Clause | Exists today | Evidence | What remains for 16e |
| --- | --- | --- | --- |
| `AC-13` comparison rule | `semantic_core`, `differing_fields`; allowed difference `ActorLocation` | `server/src/parity.rs:13–88` | Reused unchanged |
| `AC-13` talk | Two **demo.gd flavours**, re-recorded twice (route walk; ids by role) | `tools/cli/tests/ac13_semantic_parity.rs:176`, `:221`; commits `70d350f2`, `61b8d372`; `clients/protocol/evidence/request-{2d,3d}.json` | Real 3D talk vs real 2D talk (F-S14-23 still open) |
| `AC-13` talk, 2D half | Real 2D talk vs the demo's "3D" flavour, social-cafe | `client_2d_interact.rs:432` | The other half: the real 3D client |
| `AC-13` buy, drink, give, eat | **Done** with both real clients, market-town, `differing ⊆ {ActorLocation}`, also in Chinese and with no wording | `client_3d_market.rs:668–760`, `:880–928` | Nothing for buy. Add a second world where items exist (lakeside, `eat`) — Q16e-2 |
| `AC-13` shove, kick, throw | 3D sends them (`--slice-bodies` B-d, B-e, B-f) | `clients/3d-spike/scripts/slice/slice_probe_bodies.gd:5–20` | 2D sends none until **13d**; no comparison; the 3D probe's `--steps` format (`slice_probe_market.gd:7–22`) cannot open an object's menu or type an utterance |
| C-S12-1 (2D scripted mode) | **Met by 13b's selector steps** (`{"open"}`, `{"choose"}`, `{"input"}`) | `step-13-client-2d.md` §15 (13b, `:1418–1426`); used by `client_3d_market.rs:677` | Nothing on the 2D side beyond 13d's actions |
| `--scenario=ac13` (3D) | Not written. The market probe already plays 13b's steps | `slice_probe_market.gd:1–40` | Generalise the step player (Q16e-3) instead of a new scenario |
| `AC-15` | Rust connections as "windows", social-cafe, four identity lines | `tools/cli/tests/ac15_one_alice.rs:80–349` | The same proof with the real 2D and 3D clients |
| Identity of people across clients | `IDENTITY` lines equal, two worlds, on separate fresh servers | `clients/shared/checks/identity.sh` | Reused; not one running world |
| `SHOWN`, 3D | places, doorways (both sides bound), people read from drawn positions | `slice_layout.gd:157–182`, `slice_link.gd:716–727`, printed at `slice_probe_world.gd:981–985` | No loose objects, no offers; no `--drop-doorway` |
| `SHOWN`, 2D | see 13f §2 | `clients/2d/scripts/harness/drive.gd:462–484` | 13f C1 completes it |
| Parity test | Frozen in S33-h, not written; places and doorways only; Lakeside and Social Café | `pr-s33h-2d-parity.md` §5.4, §8 AC-h-2/AC-h-4 | People, objects, offers, the doorway mutation, market-town (Q-PAR-1) |
| 3D depictions | social-cafe, market-town, lakeside | `clients/3d-spike/scripts/depictions/{social-cafe,market-town,lakeside}/depiction.gd` | Enough for two worlds without 16f (Q16e-5) |
| Removal check | Pattern exists for the LM controller: scratch worktree, delete, run the gate | `scripts/ci_removability.py:1–40` (IC-7) | A second target: `clients/3d-spike`, `mineworld-slice`, `mineworld-3d` |
| Nightly | 3D probes and `ac13_semantic_parity` | `scripts/ci_layer.py:202–219` (`clients-probes`) | Add the new tests |
| Status | "parity of what each client shows S14 16e" | `docs/MVP_STATUS.md:53`, `:116` | Updated at the end |
| Review queue | VIS-3D-GODOT-2 accepted; no Demo B framework row | `docs/HUMAN_REVIEW_QUEUE.md:14–22`, `:145` | The Demo B package (§4.9 of the step) |

---

## 3. Dependencies (not redesigned here)

| On | What 16e needs | If missing at C0 |
| --- | --- | --- |
| **13d** (DRAFT) | The 2D client sends kick, throw (complete) and shove through its menu | C2's body cases cannot run; 16e does not start |
| **13f C1** | 2D `SHOWN` with doorways as drawn, `objects`, `affordances`; `IDENTITY_WORLD`; `--drop-doorway`; the presentation-only list in `clients/protocol/ADOPTION.md` | C3 cannot compare; 16e does not start |
| **13e** (DRAFT) | Nothing required: tests seat clients by argument | — |
| **16f** (DRAFT) | Only if a parity world's 3D depiction does not bind every place (Q16e-5) | Run the parity test on the worlds that bind fully; record the rest |
| **S33-h** | Nothing. If its `shown_parity.rs` lands first, C3 extends it | C3 creates the file and S33-h adds Lakeside |

---

## 4. Design

### 4.1 `AC-13` with the real clients — one step list, two players

Tools first: `client_3d_market.rs` already plays **one** step list in 13b's selector format through the
real 2D client and the real 3D client and compares the requests with `differing_fields`. 16e does the
same for the remaining actions, so `ac13_clients.rs` is that pattern, generalised:

```text
for each world (social-cafe, market-town):
  steps = from disclosure: talk to the first person with a talk offer (utterance = a test constant,
          passed with {"input"}), shove a person, kick an object, throw it (complete form), buy (only
          where a buy offer exists)
  run the 2D client on a fresh hosted world; run the 3D client on another fresh hosted world
  pair the requests by order and action type; differing_fields ⊆ {ActorLocation}; both answers the
  same result kind; the save's caused fact types equal
```

The 3D step player needs two small additions (in `slice_probe_market.gd`, or a renamed generic file —
Q16e-3): `{"input": "<text>"}` for a talk entry, and `{"open": "<object id>"}` (aim at an object the
way the reticle does, `SliceLink.aim()`, `slice_link.gd:740`). No action type, person or item is named in
the probe.

The row's adversarial case: the 2D steps with another utterance → the test fails naming `payload`.

`ac13_semantic_parity.rs` stays. It is frozen evidence of the protocol module; it is not the real-client
proof and is not edited.

### 4.2 `AC-15` with the real clients

`ac15_clients.rs` (new, `#[ignore]`): one `mineworld server <world> --agent alice --save DIR`; the real 2D
client on seat S1; the real 3D client on seat S2; both alive at once.

```text
EVIDENCE 1  same world instance        2D IDENTITY_WORLD.instance = 3D's = GET /status instance
EVIDENCE 2  same Alice EntityId        both clients' people map the save's Alice id to the disclosed name
EVIDENCE 3  same event sequence        the events each client saw at Alice's place, by EventId, are one
                                       sequence in the save (each is a sub-sequence of the save's order)
EVIDENCE 4  same persisted revision    after both stop: each client's last revision ≤ the save's;
                                       the save is one log (mineworld inspect)
STORY       2D talks to Alice; then 3D talks to Alice; Alice's answer to 3D refers to what 2D said
            (the rule controller's "Earlier, … said" line, as ac15_one_alice.rs:80 already reads)
```

The 3D client gains the same `IDENTITY_WORLD` line as the 2D (13f §4.3), printed by its probe. Worlds:
social-cafe and market-town.

### 4.3 The parity test ("one world, two views")

`tools/cli/tests/shown_parity.rs` (Q-PAR-1). One server per world. The moment is fixed by pausing the
world clock (`POST /admin/clock {"paused": true}`, `server/src/admin.rs:9–10`) so agents stand still
(Q-PAR-2). Then the 2D client joins seat S, prints `SHOWN`, leaves; the 3D client joins the **same seat**,
prints `SHOWN`, leaves. Same observer, same paused world, so the disclosure is identical, and so must be
the reports.

```text
compared (semantic):    places {id, tags}; doorways {from, to, here, there}; people {id, disclosed name};
                        objects {id, kind}; affordances {action_type, target, about, complete,
                        available, reason}
not compared:           the presentation-only list in clients/protocol/ADOPTION.md (13f C1)
cases:                  social-cafe (start place, then inside the café), market-town (the street, the café)
negative, standing:     the 3D client with --drop-doorway=<a doorway of the start place> → fails naming
                        it; the 2D client with the same flag → fails naming it
```

If S33-h's version exists, 16e adds fields and cases to it; its Lakeside case stays S33-h's.

### 4.4 The 3D `SHOWN`, completed

In `slice_layout.gd` `shown()` and `slice_link.gd` `shown()`: add `objects` (from the drawn object views,
`object_view.gd`) and `affordances` (from the offer menus the client built, 16d's `offer_menu.gd`).
Add the harness flag `--drop-doorway` in the probe (removes the bound door after the scene is built).
Nothing drawn changes.

### 4.5 The removal check (ARC-TBD, coordinator assigns)

Extend `scripts/ci_removability.py` with a second target, chosen by an argument (`--target lm|3d`):
the `3d` target deletes `clients/3d-spike`, `mineworld-slice`, `mineworld-3d` in the scratch worktree and
runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`,
and the 2D client's `--drive` headless. Any edit the copy would need is the check failing. Mutation: a
Rust test that reads a file under `clients/3d-spike/` (not `#[ignore]`d) turns it red.

### 4.6 The Demo B package

A "Demo B" framework row and section in `docs/HUMAN_REVIEW_QUEUE.md`, per `ACCEPTANCE.md` §6: what
changed, the launch commands, what to inspect, frames of each interaction (`clients/3d-spike/evidence/16e/`,
with `.gdignore`), known limitations as facts, the questions. The look is VIS-3D-GODOT-2's (accepted);
the operator judges embodiment and interaction (`MVP.md` §7.2), not graphics.

---

## 5. Acceptance — decided before measuring (`ARC-23`)

| ID | Criterion | Worlds |
| --- | --- | --- |
| AC-e-1 | `ac13_clients`: talk, shove, kick, throw (complete) — each pair `differing_fields ⊆ {ActorLocation}`, same result kind, same caused fact types | social-cafe, market-town |
| AC-e-2 | `ac13_clients`: buy — the same, plus `client_3d_market.rs:668` still passes unchanged | market-town, and lakeside `eat` (Q16e-2) |
| AC-e-3 | The 2D steps with another utterance → `ac13_clients` fails, naming `payload` | social-cafe |
| AC-e-4 | `ac15_clients`: evidence 1–4 and the story pass with the real 2D and 3D clients and `--agent alice` | social-cafe, market-town |
| AC-e-5 | `ac15_clients` against two servers (one client each) → evidence 1 fails, naming the two instances (as `ac15_one_alice.rs:349` does for Rust clients) | social-cafe |
| AC-e-6 | `shown_parity`: the two reports equal on every compared field, at every case | social-cafe, market-town |
| AC-e-7 | `shown_parity` with `--drop-doorway` on either client → fails, naming the doorway and the client | social-cafe, market-town |
| AC-e-8 | `ci_removability.py --target 3d` → PASS; the planted reader → FAIL, by name | — |
| AC-e-9 | `client_rules.rs` (3D static scan, 16a) → 0 findings on the final head | — |
| AC-e-10 | The operator's checklist (§12) runs from a clean checkout as written | both |

---

## 6. Commit plan

Each commit tracks implementation, validation and review separately. Heavy local runs through
`cast-slots/with-slot.sh`; the full suite at the milestone.

### C0 — Design (this file), docs only
- [ ] Implementation: this file; pointer in the step's row.
- [ ] Validation: `scripts/check_doc_headings.py`.
- [ ] Review: the operator's freeze.

### C1 — The 3D report and step player: `objects`, `affordances`, `IDENTITY_WORLD`, `--drop-doorway`, `input`, object `open`
- **Goal:** the 3D client reports what it shows and can be driven by one step list for every action.
- **Scope:** `clients/3d-spike/scripts/slice/slice_layout.gd`, `slice_link.gd`, `slice_probe_market.gd`
  (or its generic rename, Q16e-3), `slice_probe_world.gd` (print `IDENTITY_WORLD`). Non-goals: no
  drawing change, no new key binding, no rule.
- [ ] Implementation: §4.4; the two step forms of §4.1; `IDENTITY_WORLD`.
- [ ] Validation: `--world --link`, `--world --layout`, `--world=market-town --link` verdicts unchanged
  (S33-c's text-equal rule for Social Café's `SHOWN`, extended only by the new keys); one targeted run
  with `--drop-doorway`; `client_rules.rs` 0 findings.
- [ ] Review: LLM review against I-S14-1 (no rule) and the ADOPTION list.
- **Failure cases:** an object out of view for `{"open"}` → the step fails naming it (no teleport);
  `--drop-doorway` naming an unknown doorway → fails by name.

### C2 — `ac13_clients.rs` and `ac15_clients.rs`
- **Goal:** AC-e-1 … AC-e-5.
- **Scope:** two new test files; helpers in `tools/cli/tests/godot3d/mod.rs` only where both need them;
  `scripts/ci_godot.py` `GODOT_TESTS`.
- [ ] Implementation: §4.1, §4.2. Steps are built from disclosure (`godot2d::worlds::ids`, the
  observation), never from literals of a world.
- [ ] Validation: each test once per world; counts and wall time recorded. Counterfactuals: AC-e-3,
  AC-e-5.
- [ ] Review: the reviewer checks oracles (save, `/status`, the raw frame), never the client report alone.
- **Failure cases:** a world with no shove target in reach fails by name (content gap); a client that
  does not exit is killed at `RUN_LIMIT` and reported INCONCLUSIVE, never PASS.

### C3 — `shown_parity.rs`
- **Goal:** AC-e-6, AC-e-7.
- **Scope:** `tools/cli/tests/shown_parity.rs` (new, or S33-h's file extended); `ci_godot.py`.
- [ ] Implementation: §4.3.
- [ ] Validation: both worlds; the two negative runs; record the failure text.
- [ ] Review: the reviewer confirms the compared set equals the rule's list (places, doorways with
  destinations, people by id and name, loose objects, affordances) and that the excluded set is exactly
  ADOPTION.md's.

### C4 — Removal check, package, status
- **Goal:** AC-e-8 … AC-e-10.
- **Scope:** `scripts/ci_removability.py`; `docs/decisions/ARC-TBD.md` + index; `docs/HUMAN_REVIEW_QUEUE.md`;
  `docs/MVP_STATUS.md:53`, `:116`, and the AC rows near `:71–74`, `:132–133`;
  `clients/3d-spike/evidence/16e/**`.
- [ ] Implementation: §4.5, §4.6.
- [ ] Validation: `python3 scripts/ci_removability.py --target 3d` once (heavy; a slot); the planted
  reader red then green; `scripts/check_decision_ids.py`.
- [ ] Review: the reviewer runs §12 from a clean checkout and reports each line.

---

## 7. Files touched

| Path | Commit | Kind |
| --- | --- | --- |
| `clients/3d-spike/scripts/slice/slice_layout.gd`, `slice_link.gd`, `slice_probe_world.gd`, `slice_probe_market.gd` | C1 | edit |
| `tools/cli/tests/ac13_clients.rs`, `ac15_clients.rs` | C2 | new |
| `tools/cli/tests/shown_parity.rs` | C3 | new or extended |
| `tools/cli/tests/godot3d/mod.rs` | C2 | small additions |
| `scripts/ci_godot.py`, `scripts/ci_removability.py` | C2–C4 | edit |
| `docs/HUMAN_REVIEW_QUEUE.md`, `docs/MVP_STATUS.md`, `docs/decisions/ARC-TBD.md` | C4 | edit / new |
| `clients/3d-spike/evidence/16e/**` | C4 | new frames |

No kernel, contract, system, server, world or pack change.

## 8. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Godot pair, real server | `AC-13` for talk, shove, kick, throw, buy | `ac13_clients.rs` (nightly `clients`) |
| Godot pair, one server | `AC-15` with real clients | `ac15_clients.rs` |
| Godot pair | "one world, two views" | `shown_parity.rs` (hosted here, Q-PAR-1) |
| Scratch worktree | the 3D client is removable | `ci_removability.py --target 3d` (milestone) |
| Unchanged | buy/drink/give/eat parity; protocol-module evidence; Rust-client `AC-15` | `client_3d_market.rs`, `ac13_semantic_parity.rs`, `ac15_one_alice.rs` |

Justified overlap: `ac15_one_alice.rs` (Rust clients) and `ac15_clients.rs` (real clients) both prove
`AC-15`. The first is fast and runs in `core`; the second is the step's "validated by running the
client" (`ENGINEERING_RULES.md` §19). Neither replaces the other.

## 9. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | Two Godot processes at once exceed a slot on a loaded machine | `ac15_clients` holds one slot for both; others run one client at a time |
| R-e-2 | The paused clock is not honoured by a hosted controller | C3 first asserts `/status` revision is still between the two reports; if not, stop and report |
| R-e-3 | The 3D client cannot aim at a small object from where it stands | The step walks first (`{"choose": {"walk": true}}` exists for people; the same for objects); a miss fails by name |
| R-e-4 | 13d's 2D action labels differ, so selectors miss | Selectors use data (action type, target), never words, as 13b's do |

## 10. Freeze questions

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-PAR-1** | Which PR hosts the parity test? | **16e**, in `tools/cli/tests/shown_parity.rs` (S33-h's file, `ARC-174`). 13f only completes the 2D report. Whoever lands first creates the file. |
| **Q-PAR-2** | How do both clients report "the same moment"? | **Pause the world clock, and let the two clients join the same seat in turn.** Same observer, same frozen world, so any difference is a client's. |
| **Q-PAR-3** | How is a doorway removed for the negative case? | **A harness-only `--drop-doorway` flag in both clients**, so the negative case is a standing test. |
| **Q16e-1** | Is `ac13_semantic_parity.rs` (demo flavours) kept? | **Keep it unchanged.** It proves the protocol module; `ac13_clients.rs` proves the real clients. |
| **Q16e-2** | Second world for buy/eat parity? | **Lakeside `eat`** (it has items and both clients draw it), beside market-town's buy. |
| **Q16e-3** | The 3D step player lives in `slice_probe_market.gd`. Extend it or rename it? | **Extend it now; rename later** if a third use appears. A rename now only moves lines. |
| **Q16e-4** | Removal check: new script or extend `ci_removability.py`? | **Extend `ci_removability.py`** with `--target 3d` (tools first; one pattern). |
| **Q16e-5** | Does 16e wait for 16f? | **No.** social-cafe and market-town already have 3D depictions. If one does not bind every place at C0, the parity case for it waits for 16f and is recorded. |
| **Q16e-6** | Which world is the operator's Demo B? | **market-town** (QS14-5), with social-cafe as the second world in the checklist. |

## 11. Execution contract (filled at the freeze)

```text
PROJECT / PR:            MineWorld mvp0, S14 PR 16e — Demo B: AC-13, AC-15, the operator
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-16e-demo-b-proof.md
BRANCH / WORKTREE:
IMPLEMENTATION BASE:
FROZEN BY / DATE:
AUTHORIZED COMMITS:
VALIDATION BUDGET:
MATERIAL STOPS:
MERGE:                   by the coordinator, after the operator's review
```

## 12. The operator's checklist (short, runnable)

```text
1  ./mineworld-slice --world=market-town
   walk out to the street, into the café; press B at the counter and buy a coffee by its name
2  look at Bob and press R: he is shoved; find the box by the café door, press F: it is kicked
3  three windows: mineworld server worlds/market-town --agent alice --save /tmp/demo-b
   (the server prints its join line: address and invite)
   ./mineworld-2d --server=<address> --invite=<invite> --seat=visitor
   ./mineworld-slice --server=<address> --invite=<invite> --world-key=market-town --seat=wanderer
   speak to Alice in 2D; walk up to her in 3D and speak: she mentions what the 2D player said (AC-15)
4  ./mineworld-slice --world   (social-cafe): kick the ball, throw it, walk into someone
5  the proofs without you:
   cargo test -p mineworld-cli --test ac13_clients --test ac15_clients --test shown_parity -- --ignored --test-threads=1
   python3 scripts/ci_removability.py --target 3d
```

## 13. Ledger (empty until implementation)

| Commit | SHA | Validation evidence | Review verdict |
| --- | --- | --- | --- |

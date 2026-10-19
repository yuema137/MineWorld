# PR S33-h — Lakeside in 2D matches Lakeside in 3D: façades, place names, the lake, and one `SHOWN`

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00)

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874
Approved by / evidence: the operator, 2026-10-19, in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00
                        answered by Q-S33h-1 below; the rest of the step is not re-asked
Implementation base:    origin/main at C0
Start condition:        S29-f merged (the halt and pier-pavilion bindings and the shared station words);
                        S33-a merged (the 2D far field); S33-c merged (the 3D `SHOWN` on Lakeside); Q-S33h-1
                        ruled with a dollar cap; this design frozen; the lane lead's dispatch. NONE holds at
                        2026-10-18
Execution contract:     §12
Implementer:            Haiku-class (the pipeline run, the bindings, the words); the operator's contact-sheet
                        gate decides which candidates are accepted
Lifecycle:              DESIGN FROZEN 2026-10-19
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§1 (outcome
6), 2.2 (A-20), 2.3 (F-S33-7), 6, 9.3 (`SHOWN` parity), 11 (AC-S33-7), 12.1 (the S33-h row), 14 (Q-S33-7) ·
**Consumes:** S29-f (step-29 §11.1 row), [`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) §5.5,
[`pr-s33c-world-seam.md`](pr-s33c-world-seam.md) (`SHOWN` on Lakeside), `tools/asset_generation/` (the 2D image
pipeline) · **Binding parents:** `CLAUDE.md` §§1.1 (two first-class clients), 4 (items 3, 15); operator memory "API
keys: call, never read".

**Decision ids.** `ARC-174` (§5.4: the `SHOWN` parity test is the contract between the two clients on every world,
not only Lakeside), written by C1. `DEP-96` (the image model and snapshot used for Lakeside's sprites, with the
price read on the run's date), written by C3. No other id.

**Money.** This PR calls the OpenAI Images API under Q-S33h-1's cap only. Keys are sourced, never read, printed,
copied or committed (`source ~/.config/mineworld/secrets.env`).

---

## 0. Binding inputs, exactly restated

- **Scope (step §12.1, S33-h row).** "façade and prop bindings (new `sib_*` by the existing pipeline, operator
  contact sheet); place-name keys in every locale; the 2D far field's lake; `SHOWN` parity test". Depends on S29-f,
  S33-a, S33-c, Q-S33-7 (images). Checkpoint: "`./mineworld-2d` on Lakeside: named places, bound façades, the halt
  and pavilion; parity test green". Adversarial: "the 2D far-field clear-colour check; a missing binding falls
  back, never crashes".
- **Step §6.** "Place names | the shared locale layer gains `place.bakery`, `place.boathouse`, `place.cottages`,
  `place.square`, `place.shore`, `place.pier`, `place.lane` in every shipped locale; `PLACE_NAME_KEYS` reads them."
  "Façades | `facade:bakery`, `facade:boathouse`, `facade:cottages`, `prop:well`, `prop:lime-tree` bindings to new
  `sib_*` images from the existing 2D pipeline (OpenAI image edits, 64 px per metre, provenance sidecars), with the
  operator's contact sheet."
- **Q-S33-7(b), ruled.** "the OpenAI cap for 2D façades is set at S33-h's freeze".

---

## 1. Goal

A player who opens Lakeside in the 2D client sees the same village the 3D client draws: a bakery that looks like a
bakery, a boatbuilder's shed, cottages round a yard, a well and a lime tree on the square, the halt and the pier
pavilion, doors labelled with the right names, and the lake to the south. A test proves the two clients show the
same places and doorways.

---

## 2. Scope

### 2.1 In scope

1. `tools/asset_generation/recipes/lakeside_2d.yaml` (new): five sprite briefs in the 2D style (§5.1).
2. `presentation/mineworld-default/2D/assets/`: the accepted `sib_building_bakery`, `sib_building_boathouse`,
   `sib_building_cottage_row`, `sib_prop_well`, `sib_tree_lime` images, each with its provenance sidecar.
3. `presentation/mineworld-default/2D/assets/asset_bindings.yaml`: `facade:bakery`, `facade:boathouse`,
   `facade:cottages`, `prop:well`, `prop:lime-tree`.
4. `clients/2d/scripts/scene/places.gd`: `PLACE_NAME_KEYS` gains `bakery`, `boathouse`, `cottages`, `square`,
   `shore`, `pier`.
5. `clients/shared/settings/locale/{messages.pot, en.po, zh_Hans.po}`: the six words.
6. The 2D client places the well and the lime tree from the square's disclosed solids (12d's `place-shape`, if 2D
   reads it) or from the façade binding's anchor; C0 decides from the audited 2D code.
7. `tools/cli/tests/shown_parity.rs` (new, `#[ignore]`, Godot): the `SHOWN` parity test (§5.4).
8. `scripts/ci_layer.py`: the parity test in nightly `clients`.
9. `clients/2d/shots/lakeside/` (the village, the square, the pier, the halt).
10. `docs/DECISIONS.md` (or `docs/decisions/`): `ARC-174`, `DEP-96`.

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| The halt, kiosk, board, window and pier-pavilion sprites | S29-f |
| People's sprites | S26-e, S32-e |
| Seated drawing | S34-e |
| The 2D far field's data | S33-a (this PR re-runs the check) |

---

## 3. Audit (`origin/main @ a1e05874`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | `PLACE_NAME_KEYS` = cafe, park, store, apartments, workplace, street; "the keys are written out, not built, so the catalogue check (client_text, AC-SET-4) sees each one as used"; the door label names the first tag the wording names, else the display name, else "outside" | `clients/2d/scripts/scene/places.gd:78–90` |
| A-2 | Façade roles in the `town` set: `facade:apartments`, `facade:cafe`, `facade:residential` → `sib_building_apartment_block`, `facade:unknown:0/1` → townhouse / cottage | `presentation/mineworld-default/2D/assets/asset_bindings.yaml:73–80` |
| A-3 | Lakeside's tags: lane `street, public, outdoor`; cottages `cottages, residential`; boathouse `boathouse, residential`; bakery `bakery, indoor`; square `square, public, outdoor`; shore `shore, public, outdoor, water`; pier `pier, public, outdoor, water` | `worlds/lakeside/places/*.yaml` |
| A-4 | Locales shipped: `en.po`, `zh_Hans.po`, template `messages.pot`; `place.cafe` and `place.street` exist | `clients/shared/settings/locale/` |
| A-5 | The 2D `SHOWN` line: `print("SHOWN ", JSON.stringify({"place", "places", "doorways", …}))` | `clients/2d/scripts/harness/drive.gd:13, 468` |
| A-6 | The pipeline: `OpenAIImageProvider`, default model `gpt-image-2.5-flare`, recipe `2d_default.yaml` at 1024 × 1024, quality `high`; "about $0.077 per image" with two references, "each extra reference adds roughly $0.012"; the tooling only ever writes `candidate` | `tools/asset_generation/README.md:19–26, 118–121`; `mineworld_gen/openai_images.py:32–41` |
| A-7 | OpenAI's published price, read 2026-10-18: `gpt-image-2.5-flare` text input $5.00, image input $8.00, image output $30.00 per 1M tokens (batch: $2.50 / $4.00 / $15.00) — equal to the pipeline's table | <https://developers.openai.com/api/docs/pricing> |
| A-8 | The 2D far field loads per world (`app.gd:67–93`) | `clients/2d/scripts/app.gd` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33h-1** | **`place.lane` would never be read.** The lane is tagged `street` (A-3), and the door label uses the first tag the wording names (A-1), so it reads "street" | The step's `place.lane` is dropped; the lane reads as the street it is. Naming it "the lane" would need a display name in world data (a world change, I-S33-1) — not proposed |
| **F-S33h-2** | **The boathouse would still draw as an apartment block** if `facade:residential` were looked up before `facade:boathouse` | C0 audits the role lookup's order; the binding of the most specific tag must win (the tag order the world lists: `boathouse` before `residential`, A-3). If the lookup does not follow tag order, C2 fixes the lookup, not the world |
| **F-S33h-3** | **The cost is small but real**: five sprites × up to four candidates × at most two rounds = 40 images; at three references ≈ $0.089 each (A-6) ≈ **$3.56** | Q-S33h-1: a cap of **$8.00** |

### 3.3 What does not exist

No Lakeside 2D sprite or binding, no Lakeside place-name word, no parity test between the clients.

---

## 4. Reuse

The existing pipeline, the 2D style references and the contact-sheet gate (`tools/asset_generation`); the 2D
far-field check (RL-e0); the two `SHOWN` lines. Nothing new is adopted.

---

## 5. Design

### 5.1 The sprites

| Role | Image | Brief (64 px per metre; the 2D style references) | Size source |
| --- | --- | --- | --- |
| `facade:bakery` | `sib_building_bakery` | a two-storey stone bakery, a shop window with bread, a chimney | S33-d §5.5: 6 m frontage |
| `facade:boathouse` | `sib_building_boathouse` | a timber boatbuilder's shed, wide double doors, rooms above | S33-d §5.5: 6 m, two storeys |
| `facade:cottages` | `sib_building_cottage_row` | a low wall and gate to a yard, cottages round it | S33-d §5.5: 6 m frontage |
| `prop:well` | `sib_prop_well` | a stone well drum with a timber winch frame | S33-b's well: 1.2 m, h 0.9 m |
| `prop:lime-tree` | `sib_tree_lime` | a mature lime (*Tilia*), round crown | S33-d §5.5: 12 m |

Each run writes `candidate` sidecars (model, snapshot, references, prompt, token counts, cost estimate). The
operator picks from a contact sheet; only accepted images are bound (the pipeline's rule).

### 5.2 Names

`place.bakery` "bakery" / "面包房"; `place.boathouse` "boathouse" / "船屋"; `place.cottages` "cottages" / "小屋";
`place.square` "square" / "广场"; `place.shore` "shore" / "湖岸"; `place.pier` "pier" / "码头". The Chinese words
are the mirror's; English is authoritative (`CLAUDE.md` §2.1 rule 5).

### 5.3 The lake

S33-a's 2D file puts the lake south. This PR runs RL-e0's 2D far-field check on Lakeside (zero clear-colour
pixels) and commits the shots.

### 5.4 The parity test (`ARC-174`)

`shown_parity.rs`: host `worlds/lakeside` once; run the 2D client's `--drive` and the 3D client's `--world=lakeside
--link` against it, each printing `SHOWN` at the same disclosed moments (spawn, after entering the bakery, on the
pier). Compare the **sets** of `places` and `doorways` (place keys from disclosure; doorway pairs), not positions
or pixels. Any difference fails, naming the client and the item. `ARC-174` makes this the standing contract
between the two clients for every world that both draw; Social Café gains its case in the same file.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-h-1 (I-S33-1) | No world, System or contract change | diff scan |
| I-h-2 | A missing binding falls back to `facade:unknown:n`, never crashes | M-h-2 |
| I-h-3 | Spend ≤ Q-S33h-1's cap; keys never read or printed | the sidecars' summed `cost_usd_estimate`; review |
| I-h-4 | Only operator-accepted images are bound | the sidecars' `approval.status` |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Godot pair | the parity test | `shown_parity.rs` (`#[ignore]`), nightly `clients` on three OSes |
| 2D check | the far field's clear colour; the words' catalogue (`client_text`) | RL-e0's 2D check; the existing catalogue test |
| Pipeline | sidecars valid | `tools/asset_generation/tests` |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-h-1 | `./mineworld-2d` on Lakeside: every door label names its place; the five façades and props bound | §12.1 checkpoint |
| AC-h-2 | The parity test passes on Lakeside and Social Café | AC-S33-7 |
| AC-h-3 | RL-e0's 2D far-field check on Lakeside: zero clear-colour pixels | §12.1 adversarial |
| AC-h-4 | M-h-1 (the 3D depiction's bakery entry removed) → the parity test fails naming `bakery` | AC-S33-7 |
| AC-h-5 | M-h-2 (`facade:bakery` binding deleted) → the bakery draws `facade:unknown:n`; no error | §12.1 adversarial |
| AC-h-6 | Total spend ≤ the cap, from the sidecars | Q-S33h-1 |

---

## 9. Commit plan

Procedure: [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) §9.0, from
`/Users/yuema137/mineworld-worktrees/impl-s33h`; image calls only through the pipeline with the sourced secrets.

- **C0 — Base, prerequisites, ledger.** S29-f, S33-a, S33-c merged; the façade lookup's order (F-S33h-2); the price
  re-read from OpenAI's page on the day. `docs(plan): S33-h — open the ledger at the base`.
- **C1 — `ARC-174`.** `docs: ARC-174 — the two clients show the same places and doorways (S33-h)`.
- **C2 — Names and the lookup.** §5.2; F-S33h-2's fix if needed; the catalogue test. `feat(2d): Lakeside's places
  are named (S33-h)`.
- **C3 — The sprites (spend).** The recipe; one round of four candidates per role; the contact sheet to the operator
  through the lane lead; a second round only for rejected roles; `DEP-96`. Stop when the cap would be exceeded.
  `feat(presentation): Lakeside's 2D façades and props, operator-accepted (S33-h)`.
- **C4 — Bindings and shots.** §2.1 items 3, 6, 9; AC-h-1, AC-h-3, AC-h-5. `feat(2d): Lakeside's village drawn by
  its own façades (S33-h)`.
- **C5 — The parity test.** §5.4; AC-h-2, AC-h-4. `test(clients): 2D and 3D show the same Lakeside (S33-h)`.
- **C6 — Close-out.** Push; PR; CI; repair; train-ready. `docs(plan): S33-h close-out`.

---

## 10. Requirements this PR places on other PRs

- **R-S29f-h-1 (on S29-f).** The halt sprite and the pier-pavilion prop are bound by tag (`station`, `landmark`),
  so Lakeside draws them without a Lakeside-specific binding.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §6 | `place.lane` dropped (F-S33h-1); the cap as ruled |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-h — Lakeside's 2D parity
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33h-2d-parity.md
WORKTREE / BRANCH:       /Users/yuema137/mineworld-worktrees/impl-s33h, mvp0/pr-s33h, one session
START CONDITION:         per the header (NONE met at 2026-10-18)
APPROVED SCOPE:          §2.1 items 1–10
SPEND:                   OpenAI Images only, ≤ Q-S33h-1's cap; keys sourced, never read
NEVER:                   as pr-s33d §12; reading, printing or copying a key; binding a candidate the operator
                         has not accepted; a world change
MATERIAL STOPS:          the cap reached before every role is accepted; a needed world change
ENDPOINT AUTHORITY:      implementation, commits, push, PR, CI repair: authorized once started;
                         merge: through a merge train under the charter
STOP CONDITION:          train-ready
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-h-1 | The operator rejects every candidate twice | the cap stops spend; the role falls back (I-h-2) and the gap is reported |
| R-h-2 | The parity test is flaky on timing | it compares sets at disclosed moments, not frames |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33h-1 [operator — spend]** | Lakeside's 2D village needs five new sprites (bakery, boathouse, cottage row, well, lime tree). At OpenAI's price read on 2026-10-18 (`gpt-image-2.5-flare`: $30 per 1M output tokens, $8 per 1M image input), one high-quality 1024 px image with three references costs about $0.09. At most 40 images (four candidates, two rounds) ≈ $3.56. Approve a cap? | **Yes, $8.00**, about twice the estimate, under the standing "call, never read" rule; the run stops at the cap. *Chinese:* 湖边 2D 需要 5 张新图，估计约 3.6 美元，批准上限 8 美元吗？ |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C6 | — | [ ] | [ ] | [ ] |

Spend: $0.00 of the cap. Mutations: M-h-1, M-h-2 (none run yet). Findings: F-S33h-1…F-S33h-3.

**Design review (2026-10-18).** One independent Opus reviewer, verdict REQUEST CHANGES, no blockers; the price
arithmetic confirmed. Applied before the PR opened: M-h-1 names the 3D depiction, not the stub.

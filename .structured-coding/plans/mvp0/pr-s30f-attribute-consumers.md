# PR S30-f — Attribute consumers: cognition, the decision-model reader, the MCP adapter and both clients

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every §15 question as recommended)

```text
Design revision:        revision 1 (2026-10-18), frozen
Author:                 S30-f PR-design session (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s30f,
                        branch docs/pr-design-s30f, planning base origin/main @ b85f2804 (#260)
Approved by / evidence: the operator, 2026-10-18, in the primary session: Q-S30f-1 (schema 2, label/facets/rank,
                        2 048 B trim) and Q-S30f-2 … Q-S30f-5 as recommended
Implementation base:    main (origin/main at C0); C0 records the commit and the merged prerequisites
Start condition:        S30-c, S30-d and S30-e merged (the operator's brief of 2026-10-18; Q-S30f-5); S27-d1 (#261,
                        the single `act` tool) merged or still open — C0 records which, and C7 re-records the tool
                        golden on whatever is merged; this design frozen; §15 ruled; the primary's dispatch
Execution contract:     §13
Implementer:            Opus-class session; C4, C5, C7 and C8 may be delegated to Haiku-class sub-agents from this
                        commit plan, each reviewed by the session (step-30 §13.1, the S30-f row; the standing
                        "design on Opus, implement on Haiku" rule)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15; amended)
§§0 item 6, 4.4 (`persona`), 5.4, 5.6, 6.1 (the reader codes OV, ME, CV, DM), 7.3 (`freckles`), 9.2, 9.3, 9.4 (the named
exception), 9.5, 11.2, 12, 13.1 (the S30-f row), 13.3 (lines 1464–1468: S30-f is release-blocking), 13.4 (AC-S30-3,
AC-S30-10, AC-S30-11, AC-S30-15), 13.5 (X-S30-4, X-S30-17) · **Specified by:** `docs/CORE_CONCEPTS.md` §4.5;
`docs/MODULE_SPEC.md` §§3.1 "Owns", 4.1 "Attributes" (fields, *Rendering*, *Disclosure*), 5.1; `docs/DECISIONS.md`
`ARC-120` … `ARC-126` · **Builds on:** [`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (FROZEN 2026-10-18),
[`pr-s30d-sampler.md`](pr-s30d-sampler.md) (FROZEN), [`pr-s30e-builtin-catalogue.md`](pr-s30e-builtin-catalogue.md)
(FROZEN); [`pr-s27d-mcp-stdio.md`](pr-s27d-mcp-stdio.md) (MERGED, #239) and its amendment S27-d1 (#261, open);
[`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) (P6a #236 and P6b #207 MERGED); `pr-dl-b-motivated.md`
(MERGED) · **Parent:** [`overall.md`](overall.md) · **Followed by:** S30-g (reference worlds), S30-h (the MatrAIx demo
reads every surface built here), S30-i, S30-k · **Binding parents:** `CLAUDE.md` §§1.1, 2, 4 (items 1, 3, 5, 7, 8, 9,
10, 12, 13, 15); `docs/MVP.md` AC-16; step-27 INV-S27-2, INV-S27-12; step-28 I-S28-1; `docs/DECISIONS.md` `ARC-26`,
`ARC-28`, `ARC-74`, `ARC-84`, `ARC-96`, `ARC-120` … `ARC-126`, `ARC-153` (S27-d1, once merged).

**Decision ids.** None new. Dated notes under `ARC-120` (the owner `employment`), `ARC-122` (the disclosed payload's
schema 2, the rendering of an `instant`) and `ARC-125` (the consumers of appearance descriptors). `ARC-127` … `ARC-129`
and `DEP-69` stay reserved and unused.

No model is called. No reference world's bytes move.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-f row).** "cognition (`mineworld_cognition` attribute renderer, persona `Attributes`
  part, perceived-person phrases, budgets, golden tests mirroring the Rust renderer); SDK typed reader for decision
  models; MCP `description` and `me.attributes` (S27-d's or a successor's surface); 2D and 3D inspect panels."
- **Integration checkpoint (same row).** "one scenario: an agent, an LM-free controller and both clients see the same
  phrases for Alice; Bob's private attributes absent everywhere."
- **Adversarial criteria (same row).** "X-S30-4 across all four surfaces; X-S30-17 the prompt stays within its byte
  budget with 200 attributes."
- **Release gate (step-30 §13.3).** "the public release (S23) does not ship until S30-a…k have all merged." S30-f is
  release-blocking; S30-h, S30-i and S30-k start only after it.
- **The named exception (step-30 §9.4; Q-S30a-2, ruled 2026-10-15).** "S30-f makes `employment` the owner of
  `employment.occupation` — a projection of its own `Employment { job }` to the job's label, and the declaration
  shipped with the pack (`ships_data!`). Nothing else in `employment` changes."
- **Requirements placed on this PR.**
  - *R-S30f-a-1* (S30-a): `employment` becomes the owner of `employment.occupation`; nothing else in `employment`
    changes.
  - *R-S30f-c-1* (S30-c §11): "Readers of the disclosed payload use `kind` and the externally tagged value (§5.2); the
    Python renderer mirrors `render.rs` by goldens produced from hand-written expectations; `employment` becomes an
    owner through `AttributeOwner` and `attributes-handed`."
  - *R-S30f-e-1* (S30-e §11): "Select by facets (`observed`, `prompt`, `speech`, `decision`), never by id;
    `prompt_rank` is S30-f's to add to declarations (a minor version of the pack)."
- **The operator's principles for this PR (task brief, 2026-10-18).**
  - P-1. The built-in catalogue is at most 100 gameplay-driven attributes, and masks are modular. A consumer therefore
    selects by facet and never by id, and a masked-out attribute appears on no surface.
  - P-2. MatrAIx is an optional backend and a demo, never a hard dependency. No consumer names it or depends on a
    `core.` id existing.
  - P-3. Agents get information as rich as a human's: **if a human client can see an attribute, so can an agent.**
- **Invariants that bind (step-30 §12).** I-S30-3 (a private or hidden attribute never appears in another seat's
  observation, MCP view or client panel); I-S30-5 (no System reads `text`); I-S30-8 (kernel and contracts ignorant);
  I-S30-9 (a learned attribute is memory, never world state); I-S30-10 (a world without `attributes` is byte-identical).
- **R-LOAD-1.** Locally only the targeted test files of a commit, `cargo clippy` on touched crates, `ruff` and `pyright`
  on touched Python packages; never `cargo test --workspace`; CI on the exact head is the gate.

---

## 1. Goal

Make a Person's attributes **reach every reader the player notices**, through one shape on the wire and one reader per
language, so that a human, an agent and an NPC's cognition are told the same thing within the same perception:

1. **One disclosed payload a consumer can select from without the catalogue** (schema 2, Q-S30f-1): each entry carries
   its label, facets and rank, in rank order.
2. **One reader per language.** Rust: a typed reader of disclosed attributes in the model crate, for decision models.
   Python: one reader and composer in the Python SDK (Q-S30f-2), used by cognition and the MCP adapter. GDScript: one
   reader and composer in the shared protocol module, used by both clients.
3. **Cognition.** The LM controller's prompt gains "who I am" (facet `prompt`), "how I speak" (facet `speech`) and, for
   each person present, "who I see" (facet `observed`), each under a byte budget; a world without attributes produces
   byte-identical requests.
4. **Decision models.** `motivated` gains an `attribute` consideration declared in its parameters (the DM reader).
5. **The MCP adapter.** `PersonLine.description` and `me.attributes`; `inspect` unchanged.
6. **Both clients.** A glance line for the person looked at or selected, an inspect panel with every disclosed phrase,
   and the holder's own sheet — the same texts in 2D and 3D.
7. **`employment.occupation`**, the one named owner exception.

---

## 2. Scope

### 2.1 In scope

1. **The model crate** (`authoring/attributes/`): the disclosed payload's schema 2 (`render.rs`); a typed reader of
   disclosed attributes (`disclosed.rs`, new); `prompt_rank` decoded and carried (if S30-c did not already); an
   `instant` renders no text (Q-S30f-4).
2. **The built-in catalogue** (`entities/person-attributes/`): `prompt_rank` on the `observed`, `prompt` and `speech`
   attributes; version `1.1.0` (a minor version, `ARC-122` versioning).
3. **`employment`** (`systems/employment/`): `employment.occupation` declared in a shipped file, projected through
   `AttributeOwner`, disclosed through `disclose_owned` to the holder; the authored `job:` section's optional `title:`
   (Q-S30f-3).
4. **The Python reader** (`sdk/python/src/mineworld_sdk/attributes.py`, new): decode, merge, select, compose, cut.
5. **Cognition** (`cognition/lm-controller/`): `SelfAttributes`, `Person.description`, the prompt's three parts, the
   logged decision input; goldens.
6. **`motivated`** (`cognition/motivated/`): the `attributes` parameter and the `Attribute` consideration.
7. **The MCP adapter** (`clients/agent/`): `AttributeLine`, `PersonLine.description`, `MeLine.attributes` (paged, list
   name `attributes`), the tool golden re-recorded with a reviewed diff.
8. **Both clients** (`clients/protocol/mineworld/attributes.gd`, new, copied into `clients/2d/mineworld/` and
   `clients/3d-spike/mineworld/`; `clients/2d/scripts/hud/readers.gd`, `clients/2d/scripts/menu.gd`;
   `clients/3d-spike/scripts/slice/slice_main.gd` and a new `slice/inspect_panel.gd`): glance, inspect panel, own sheet.
9. **The scenario and the scans** (§5.11): one fixture world, one hand-written oracle, one test per surface against it,
   and the structural scans.
10. **Specification sync** (dated notes; `CLAUDE.md` §2.1 rule 4): `docs/MODULE_SPEC.md` §§3.1, 4.1, 5.1;
    `docs/DECISIONS.md` notes under `ARC-120`, `ARC-122`, `ARC-125`; `clients/agent/README.md`,
    `clients/protocol/README.md`, `sdk/python/README.md` (one line each, pointing at the specification).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Any reference world enabling `attributes`; re-baselines; S26's switch to `resolve --json` | S30-g |
| The MatrAIx backend, its masks, mappings and demo (it reads the surfaces built here, unchanged) | S30-h |
| Learning an attribute in conversation; `tell-about-self`; S28 `stable` facts | S30-i |
| Locale label packs, zh-Hans panels and prompts (texts are server-rendered, so S30-k swaps words, not code) | S30-k |
| `economy.wealth-band`, `notices.subscriptions` and their readers | S31 |
| `persona` as an `AttributeOwner`; the review of `persona.traits` against the psychology group | the Q-S30-4 review after this PR merges (ruled 2026-10-15) |
| A memory renderer for `trait-changed` that words its values | S30-i (memory of attributes is its subject) |
| Any change under `kernel/`, `contracts/`, `server/`, or any System other than `employment` | never |

### 2.3 Dependencies

- **Hard:** S30-c merged (the model crate, the `traits` component, `disclose_owned`, `attributes-handed`). Every type
  named in §§3–5 that begins with the model crate exists only after it.
- **Required by this contract** (Q-S30f-5): S30-d merged (worlds resolve) and S30-e merged (the `core` catalogue with
  facets, masks and `configure/attributes.yaml`'s `backend`/`mask`), so that the scenario runs on the real `core`
  catalogue and the mask test uses the real mask loader.
- **Merged already:** S27-d (#239), P6a (#236), P6b (#207), DL-b (`motivated`), S28-a (memory renderers).
- **Concurrent:** S27-d1 (#261) rewrites `clients/agent/src/mineworld_agent/{models.py, mcp_server.py}` and
  `tests/golden/tools.json`. This PR's C7 is planned against S27-d's `View` models, which S27-d1 leaves unchanged
  (its R-3); C0 records whether #261 has merged, and C7 rebases on it either way.

---

## 3. Audit (`origin/main @ b85f2804`; the S30-c/d/e designs as frozen; `feat/mcp-single-act @ 773eb6f0`; 2026-10-18)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-f-1 | The disclosed `traits` record is `{ schema: 1, entries: [ { attribute, kind, value, text? } ] }` in catalogue order; another observer receives `perceivable` entries only, a `perceived_as` value as its words with `value` absent | `pr-s30c-attributes-pack.md` §5.8; `docs/MODULE_SPEC.md` §4.1 *Disclosure* (line 1485) | a consumer has the rendered `text`, but **no label, no facet and no rank** (F-S30f-1) |
| A-f-2 | "Catalogue order" is the frozen `Catalogue`'s `BTreeMap<QualifiedId, Declaration>` order: by qualified id, alphabetical | `pr-s30c-attributes-pack.md` §5.3 item 7 | a description in catalogue order begins `core.age-band, core.body-form, core.build …` by spelling, not by salience |
| A-f-3 | `prompt_rank` is a declaration field ("the order in which a byte-budgeted rendering keeps attributes (cognition, S30-f)"); S30-c's run-time projection lists it; S30-e leaves the values to S30-f | `docs/MODULE_SPEC.md` line 1219; `pr-s30c-attributes-pack.md` §5.5; `pr-s30e-builtin-catalogue.md` §11 R-S30f-e-1 | C1 confirms S30-c decodes it (else adds it, bounded); C2 writes the values |
| A-f-4 | Facets follow the reader codes: `C` → `casting`, `OV` → `observed`, `ME` → `prompt`, `CV` → `speech`, `DM` → `decision`, `S31` → `notices`; `ancestry` carries `casting` only ("never written into a prompt", A-10) | `pr-s30e-builtin-catalogue.md` §5.5 and Appendix A | selection by facet keeps `ancestry` out of every prompt with no consumer code naming it |
| A-f-5 | An owner discloses its attributes "within its own component" through `disclose_owned<O>(world, observer, subject) -> Option<Value>`; `None` where `attributes` is not enabled (Q-S30c-2) | `pr-s30c-attributes-pack.md` §5.10, AC-c-10 | a consumer reads attribute entries from `traits` **and** from owners' components (F-S30f-2) |
| A-f-6 | The prompt's system message: `You are <me>.`, then `About you:`, `Traits:`, `How you speak:` from `persona`, then the rules; the user message: `WHERE I AM` (`Place`, `People here: <labels>`, `I can talk to`, `Offers`), `WHAT I REMEMBER`, `WHAT JUST HAPPENED` | `cognition/lm-controller/src/mineworld_cognition/controller/prompt.py:112-167` | the three new parts' places (§5.6) |
| A-f-7 | `Persona.of(observation)` reads the observer's own `persona` component; `Situation.of` builds `Person { label, entity }` per person present | `controller/persona.py:52-65`; `controller/situation.py:22-90` | `SelfAttributes.of` sits beside `Persona.of`; `Person` gains `description` |
| A-f-8 | Every consult logs `DecisionInput { persona, situation, memory, trigger, observation }`, and logged-input replay rebuilds the request from those fields | `controller/log.py:56-63, 240-270` | the new inputs are fields of `DecisionInput` with empty defaults, so an old log replays to the old request |
| A-f-9 | The prompt golden (`tests/fixtures/prompt_request.json`) is compared with a built request; cassettes are keyed by the request's hash (`record.py`, `cassette_key`) | `tests/test_controller_choice.py:154-163`; `src/mineworld_cognition/record.py:130-158` | a world without attributes must build byte-identical requests, or every cassette misses (I-f-2) |
| A-f-10 | Memory ingests **perceived facts** through per-event-type renderers with a generic default; components are not ingested | `memory/render/__init__.py:1-11, 118-145` | a `traits` component never becomes a memory record; `traits-stated` and `trait-changed` render generically (F-S30f-5) |
| A-f-11 | `motivated` scores a choice as its base times its considerations through the compensation factor `1 000 − 1 000 / k`, where `k` is the number of considerations | `cognition/motivated/src/score.rs:16-37` | adding a consideration — even a neutral 1 000 — changes `k` and so every other value's adjustment; an attribute consideration is added only when declared **and** disclosed (D-f-4) |
| A-f-12 | `motivated` reads disclosed components through typed payloads of the packs it depends on (`weather`, `schedule` …); its parameters are decoded by hand with line and column | `cognition/motivated/src/options.rs:60-130`; `parameters.rs:1-80`; `Cargo.toml` | the model depends on `mineworld-attributes-model` like any reader; the parameter decodes the same way |
| A-f-13 | The agent's `PersonLine { ref, name, labels, position, walking, doing }`, `MeLine { …, acquaintances, invitations }`; paged lists `ListName = people, things, offers, acquaintances, invitations`; world-authored strings are `WorldText` | `clients/agent/src/mineworld_agent/models.py:158, 289-321`; `view.py:255-330` | `description` on `PersonLine`; `attributes` on `MeLine`, paged as a sixth list |
| A-f-14 | `disclosed.py` reads each pack's own component by name and returns `None` for any shape it does not know; "Nothing here evaluates a rule" | `clients/agent/src/mineworld_agent/disclosed.py:1-25` | the agent reads attributes through the SDK reader, called from `disclosed.py` |
| A-f-15 | `inspect` returns an entity's disclosed components, cut to 4 096 B | `view.py:70, 567-577` | `traits` already appears there unchanged (step-30 §9.3) |
| A-f-16 | S27-d1 replaces five act tools with one `act`; `View` and every output model are unchanged (its R-3) | `feat/mcp-single-act:.structured-coding/plans/mvp0/pr-s27d1-single-act.md` §2 | the two PRs touch the same files but not the same models; the tool golden is re-recorded on whichever base |
| A-f-17 | The 2D client shows the observer's **own** components as panels (a reader per component, raw JSON for an unknown one under "other"), and a click on a figure opens the menu about that subject | `clients/2d/scripts/hud/readers.gd:1-17, 78-112`; `clients/2d/scripts/menu.gd:27, 108` | an own `traits` today lands raw in "other"; the subject menu is the 2D inspect surface |
| A-f-18 | The 3D slice shows "looking at <who>" for the person the camera's ray meets (`SliceLink.facing_person`); it has no panel | `clients/3d-spike/scripts/slice/slice_main.gd:546-590`; `slice_link.gd:532-536`; `targeting.gd` | a glance line under it and a new inspect panel |
| A-f-19 | The protocol module `mineworld/` (`delta.gd`, `observation.gd`, `space.gd`, `world_client.gd`) is copied into `clients/2d/` and `clients/3d-spike/`; the three copies are byte-identical today, and no check holds them so | `diff -rq` of the three directories (empty); `clients/protocol/README.md:3` | `attributes.gd` joins the module; a structural test holds the copies identical (F-S30f-6) |
| A-f-20 | The client rules scan refuses rule words (`reach`, `range`, `clearance`, `nudge`, `capacity`) and action-type literals in client code outside a reasoned allow-list | `tests/acceptance/tests/client_rules.rs:60-115` | the GDScript composer names no such word; it formats and cuts only |
| A-f-21 | 2D runs are driven from Rust (`tools/cli/tests/godot2d/`, `--drive=steps` / `--drive=panels`, `PANELS` lines); the 3D slice probe runs through `scripts/ci_godot.py slice_probe` with a printed verdict | `tools/cli/tests/client_2d_interact.rs`; `clients/2d/scripts/harness/interact.gd:1-47`; `scripts/ci_godot.py:229-280` | each client's surface test prints its texts; both are compared with one oracle (§5.11) |
| A-f-22 | `Job { employer, workplace, from, until, wage, produces }`; the authored `job:` section has the same fields; **there is no job label** | `systems/employment/src/component.rs:41-48`; `section.rs:42-70` | "projected to the job's label" has no field to project from (F-S30f-3, Q-S30f-3) |
| A-f-23 | `employment` discloses `Employment` to the holder only (`subject != observer` → nothing) | `systems/employment/src/system.rs:254-276` | `employment.occupation` is `learnable`: the holder's surfaces show it, another observer's never |
| A-f-24 | S30-e's `birth` row: `"born {value}"`, "(the instant renders as seconds before the epoch only for the holder's tools; S30-f renders a date)" | `pr-s30e-builtin-catalogue.md` Appendix A row 23 | a date needs a calendar epoch, which the model crate cannot read (F-S30f-4, Q-S30f-4) |
| A-f-25 | `persona`'s fields are listed as `persona`-owned `text` attributes, "P6a merges as frozen and the overlap … is reviewed after S30-f" (Q-S30-4); P6a merged without being an owner | `docs/MODULE_SPEC.md` §3.1 "Owns"; `docs/DECISIONS.md` `ARC-120` Consequences | the specification names an owner that does not exist yet; recorded, not fixed here (F-S30f-7) |
| A-f-26 | The Python SDK is domain-agnostic: protocol, sessions, offers, perceived facts; it reads no pack's payload | `sdk/python/src/mineworld_sdk/*.py`; `sdk/python/README.md` | putting the attribute reader there is a decision (Q-S30f-2) |

### 3.2 Findings

- **F-S30f-1. The wire carries no selector.** A consumer must choose `prompt` for "who I am", `speech` for "how I speak"
  and `observed` for "who I see" (R-S30f-e-1), and cut by rank (X-S30-17), but schema 1 has neither facets nor rank,
  and its order is alphabetical (A-f-1, A-f-2). A remote seat — an agent, a client — has no other route to the
  catalogue. **Q-S30f-1** recommends schema 2: every entry gains `label`, `facets` and `rank`, and entries are in rank
  order.
- **F-S30f-2. Attributes arrive in more than one component.** Owners disclose within their own components (A-f-5).
  Decided (D-f-1): an **attribute block** is any JSON object `{ "schema": 2, "entries": [...] }` that is either the
  payload of `traits` or the member `attributes` of an owner's component payload — the exact shape `disclose_owned`
  returns, which an owner embeds under that key. Every reader merges the blocks of one entity by `(rank, attribute)`.
  C0 confirms the embedding against S30-c's merged code; if S30-c chose another embedding, the reader follows it and the
  ledger records it (bounded).
- **F-S30f-3. `employment` has no job label** (A-f-22). The ruling's "projection … to the job's label" cannot be built
  from today's state. **Q-S30f-3** recommends an optional `title:` in the authored job, projected as a `derived` `text`
  attribute.
- **F-S30f-4. An instant cannot become a date in the model crate** (A-f-24): the world's epoch date is the calendar
  pack's configuration, and the model crate depends on no pack. **Q-S30f-4** recommends that an `instant` renders no
  text; its typed value still reaches the holder's typed surfaces, and `age-years` carries the information in prose.
- **F-S30f-5. Memory sees facts, not components** (A-f-10). Step-30 §9.5's "S30-f checks that `traits` is treated the
  same way" is met by a test that a disclosed `traits` produces no memory record and that `trait-changed` is ingested,
  generically, like any unknown event type.
- **F-S30f-6. The protocol module's copies are unchecked** (A-f-19). The composer must be one implementation in both
  clients; a structural test in `client_rules.rs` holds the three copies byte-identical.
- **F-S30f-7. `persona` is named an owner but is not one** (A-f-25). The step's Q-S30-4 ruling defers it to a review
  after S30-f. This PR keeps `persona`'s prompt lines as they are and adds `MODULE_SPEC` §3.1 a dated note: "`persona`
  is not yet an owner; the review of Q-S30-4 decides". No contradiction is left unrecorded.
- **F-S30f-8. "Mirrors the Rust renderer" is not needed.** R-S30f-c-1 asked the Python side to mirror `render.rs`. The
  server already renders every entry's `text` (A-f-1), so no consumer renders a value: the Python and GDScript code
  **select, merge, join and cut** server-rendered text only. What is mirrored, by one shared hand-written golden, is
  the composition (§5.4). The renderer exists once, in Rust.
- **F-S30f-9. A cut description can make an agent poorer than a human.** If a client's inspect panel shows every
  perceivable phrase while the agent's view shows only a 160-byte description, P-3 fails. Decided (D-f-2): **the glance
  is the view, the inspect panel is `inspect`.** The 3D glance line, the 2D menu header, the MCP `PersonLine.
  description` and cognition's "who I see" are one composition with one budget; the 2D and 3D inspect panels and the
  MCP `inspect` result list every entry of every block. The parity tests hold each pair equal.

### 3.3 What does not exist

No consumer reads an attribute: no prompt part, no view field, no panel, no decision-model input. No `prompt_rank` value.
No owner other than S30-c's test-only `test-badge`. No fixture world with consumers.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

- **Adopted, already in the tree:** `pydantic` (the Python reader's strict models, as `persona.py` and the agent's
  models use), `serde_json`, the existing Godot harnesses (`--drive=panels`, the slice probe) and the server's real-binary
  test support.
- **Considered and declined:** a template engine (`jinja2`) for the prompt parts — each part is one join under a byte
  cap, and a template language would admit logic in data; a second renderer in Python or GDScript (R-S30f-c-1's mirror)
  — the server renders once (F-S30f-8); a shared JSON-Schema of the payload published to clients — the payload is
  documented in `MODULE_SPEC` §4.1 and decoded strictly by each reader, and a schema file would be a fourth definition.
- **Not forced:** nothing here adopts a framework's execution model.

---

## 5. Design

### 5.1 The disclosed payload, schema 2 (model crate; Q-S30f-1)

```json
{ "schema": 2,
  "entries": [
    { "attribute": "core.age-band", "kind": "level", "label": "Age", "facets": ["casting", "observed"],
      "rank": 10, "text": "in their sixties" },
    { "attribute": "core.stature", "kind": "quantity", "label": "Stature", "facets": ["body", "casting", "observed"],
      "rank": 20, "text": "tall" },
    { "attribute": "core.hair-colour", "kind": "category", "label": "Hair colour",
      "facets": ["casting", "observed"], "rank": 40, "value": { "category": "grey" }, "text": "grey hair" } ] }
```

- **Fields added to each entry:** `label` (the declaration's label, after overrides); `facets` (the declaration's facets,
  sorted); `rank` (`prompt_rank`, an integer 0 … 65 535, or absent). `attribute`, `kind`, `value` and `text` keep schema
  1's meaning exactly (S30-c §§5.2, 5.8).
- **Order:** entries are sorted by `(rank, attribute)`, an absent rank after every present one. "Catalogue order" in
  `MODULE_SPEC` §4.1 *Disclosure* becomes "rank order" by a dated note.
- **Views:** unchanged from S30-c — the holder receives every non-hidden entry; another observer receives `perceivable`
  entries only, a `perceived_as` value as its words with `value` absent.
- **Owners:** `disclose_owned` returns the same schema 2 block (D-f-1), so an owner's entries merge by rank.
- **`instant`** (Q-S30f-4): its `text` is absent; its `value` is present for the holder.
- **Cost.** C1 re-measures AC-c-14 on the 93-attribute fixture (Alice's own view, Bob's view of her) and records schema
  1 against schema 2 in §16. **Pre-approved bounded fix:** if Bob's view of one person exceeds **2 048 B**, observer-view
  entries carry only the facets in the consumer set `{observed, prompt, speech, decision}`; beyond that, a material stop
  (R-f-1).

### 5.2 The Rust reader for decision models (`authoring/attributes/src/disclosed.rs`; D-f-3)

```rust
/// The attributes a perceived entity's components disclose, merged from every attribute block (D-f-1).
pub struct Disclosed { entries: Vec<DisclosedEntry> }   // rank order
impl Disclosed {
    pub fn of(entity: &PerceivedEntity<Value>) -> Disclosed;              // never fails: an unknown shape is skipped
    pub fn level(&self, id: &QualifiedId) -> Option<&ValueSlug>;          // None when absent or of another kind
    pub fn category(&self, id: &QualifiedId) -> Option<&ValueSlug>;
    pub fn quantity(&self, id: &QualifiedId) -> Option<ScaledInt>;
    pub fn flag(&self, id: &QualifiedId) -> Option<bool>;
    pub fn set(&self, id: &QualifiedId) -> Option<&BTreeSet<ValueSlug>>;
}
```

- **D-f-3. It lives in the model crate, not in `mineworld-sdk`.** Step-30 §9.2 says "the SDK gains a typed reader". The
  Rust SDK is the domain-agnostic pack SDK, and every reader of attributes already depends on the model crate (`ARC-124`,
  S30-c F-S30c-4); placing it there keeps one crate for the vocabulary. Recorded as a dated note on step-30 §9.2 (the
  primary's edit, §12).
- A `text` entry has no typed accessor (I-S30-5 holds for decision models too). A `value` that is absent (a
  `perceived_as` band) reads `None`.

### 5.3 The built-in catalogue's ranks (`entities/person-attributes`, version 1.1.0)

Ranks are spaced by ten so a world's override can slot between them. They order salience — what a person notices first.

| Rank | Attributes (`core.`) | Facet that makes them matter |
| --- | --- | --- |
| 10–90 | `age-band`, `stature`, `build`, `body-form`, `hair-colour`, `hair-length`, `facial-hair`, `glasses`, `clothing-style` | `observed` |
| 100–190 | `headwear`, `visible-marks`, `posture`, `walking-pace`, `voice`, `greeting-style`, `skin-tone`, `eye-colour`, `smoking` | `observed` |
| 200–290 | `pronouns`, `age-years`, `native-language`, `languages-spoken`, `household`, `marital-status`, `children`, `years-in-town`, `hometown`, `core-value` | `prompt` |
| 300–390 | the five traits, `trust-propensity`, `optimism`, `honesty`, `hobbies` | `prompt` |
| 400–490 | the twelve `skill.*` and `fitness`, `religion`, `music-taste`, `sport-followed`, `pet` | `prompt` |
| 500–590 | `formality`, `humour`, `patience`, `talkativeness`, `gossip`, `education-level` | `speech` |
| absent | every other attribute (casting-only, decision-only) | — |

`hair-style` and `hair-texture` stay unranked: their words are casting vocabulary (`textured-top`, `low-bun`) that an
observer reads after the ranked set, if the budget allows. C2 writes the table into the six files; the ranks are data,
reviewed as data, and change no fact (a world that adopts the pack in S30-g records them in its catalogue's identity).

### 5.4 The composition (one rule, three implementations held equal by one golden)

Every consumer applies exactly this, and only this, to the merged entries of one entity:

1. **Select** the entries carrying the facet (`prompt`, `speech` or `observed`) and a non-empty `text`.
2. **Order** them by `(rank, attribute)` (already the merged order).
3. **Join** their `text` with `", "`.
4. **Cut** at the budget in UTF-8 bytes **at an entry boundary**: keep the longest prefix of whole entries whose join
   fits; if any entry was dropped, append `" …"` (four bytes, U+2026 after a space) and drop one more entry if that is
   needed to fit. An entry longer than the budget on its own is cut at a UTF-8 boundary and followed by `…`.

| Composition | Facet | Budget | Who uses it |
| --- | --- | --- | --- |
| **who I am** | `prompt` | 600 B | the LM controller's system message |
| **how I speak** | `speech` | 160 B | the LM controller's system message |
| **who I see** (the glance) | `observed` | 160 B | the LM controller's "People here"; MCP `PersonLine.description`; the 2D menu header; the 3D glance line |
| **the inspect list** | none (every entry with text) | none; one line per entry | 2D and 3D inspect panels; MCP `inspect` carries the raw blocks |
| **the own sheet** | none | none; one line per entry, `label: text` (`label: <typed value>` when `text` is absent) | 2D and 3D own sheet; MCP `me.attributes` |

- **One golden for all three languages.** `authoring/attributes/tests/fixtures/composition.json` holds hand-written
  input blocks (including a 200-entry block for X-S30-17, an owner block, an entry with an empty word, an over-long
  entry, multi-byte text) and the expected strings, written by hand (test rules §25). The Rust test (`disclosed.rs`
  tests, for the reader), the Python test (`sdk/python/tests/test_attributes.py`) and the GDScript check
  (`clients/protocol/checks/attributes_check.gd`) each read it and must equal it. A change to the rule is one golden
  edit and three red tests until all agree.
- **Who I see uses the other observer's view only.** The composer is always given the subject's own entity record
  from the observer's observation; it never reads the observer's own blocks to describe someone else (M-f-2).

### 5.5 The Python reader (`mineworld_sdk.attributes`; Q-S30f-2)

```python
class AttributeEntry(WireModel):   # strict, frozen, extra="forbid" on the fields of §5.1
    attribute: str; kind: Literal["category", "level", "quantity", "flag", "set", "text", "instant"]
    label: str; facets: tuple[str, ...]; rank: int | None = None
    value: AttributeValue | None = None; text: str | None = None

def blocks(entity: PerceivedEntity) -> tuple[AttributeEntry, ...]      # merged, rank order; unknown shapes skipped
def compose(entries, facet: str | None, budget: int | None) -> str     # §5.4
def sheet(entries) -> tuple[SheetLine, ...]                             # §5.4 "the own sheet"
```

- `AttributeValue` is the externally tagged union of S30-c §5.2, decoded strictly (`{"quantity": "1.660"}` stays a
  decimal string; no `float`).
- A block whose `schema` is not 2, or whose entry does not decode, is skipped whole and counted
  (`attributes_unreadable`), never half-read: a newer server cannot put an unchecked string into a prompt.
- The module names no attribute id, no facet other than as a parameter, and no backend (AC-f-9).

### 5.6 Cognition (`cognition/lm-controller`)

- **`SelfAttributes`** (`controller/attributes.py`, new): `about: str` (who I am) and `manner: str` (how I speak), built
  by `SelfAttributes.of(observation)` from the observer's own entity through §5.5. Pure. An observation with no blocks
  yields `SelfAttributes()` (both empty).
- **`Person.description: str = ""`** (`controller/situation.py`): who I see, for each person present, from that person's
  entity record in the observation.
- **The prompt** (`controller/prompt.py`):
  - system message, after `persona`'s lines and before the rules: `What the world says about you: <about>` when `about`
    is not empty; `Your manner: <manner>` when `manner` is not empty;
  - user message: `People here: Bob (tall, grey hair, wears glasses), Carol` — a person's description in brackets
    when it is not empty;
  - every attribute text passes through `_quoted`'s delimiter replacement, as heard words do, so world text cannot
    forge a `<<<heard>>>` boundary.
- **Empty means absent.** With no blocks, the system and user messages are byte-identical to today's (I-f-2): no header,
  no brackets, no blank line.
- **The decision input** (`controller/log.py`): `DecisionInput.attributes: SelfAttributes = SelfAttributes()`;
  `Situation` carries the descriptions in `Person`. A log written before this PR replays to the request it recorded.
- **`build_request`** gains `attributes: SelfAttributes` after `persona`; `consult.py`, `seat.py` and the replay in
  `log.py` pass it (the three callers of A-f-8).
- **Budgets** are constants beside `REASK_PREVIOUS_BYTES` (`ABOUT_BYTES = 600`, `MANNER_BYTES = 160`,
  `DESCRIPTION_BYTES = 160`), inside step-17's 6 000-byte context; the per-controller "extra facets" of step-30 §9.2 is
  not offered (D-f-5: no reader asks for it; a later controller adds it as a parameter).
- **Memory** is unchanged (F-S30f-5).

### 5.7 The decision model (`cognition/motivated`; D-f-4)

```yaml
models:
  motivated:
    attributes:                          # optional; at most 8 entries
      - { attribute: core.chronotype, option: agenda,
          by: { early-bird: 1000, morning-leaning: 900, neutral: 750, evening-leaning: 600, night-owl: 450 } }
```

- `attribute` is a qualified id (the model checks its shape, not its existence: the decision list is decoded without a
  catalogue); `option` is one of `OPTIONS` (`parameters.rs`); `by` maps value slugs to per-mille values 0 … 1 000.
  Each refusal keeps its line and column, as every `motivated` parameter does.
- At a consult, for a choice of that option, the model reads the **observer's own** value through `Disclosed::level`
  or `category` (§5.2). When the value is disclosed and named in `by`, the consideration `Attribute(index)` with that
  value is appended after the option's others; **otherwise nothing is appended** — not a neutral 1 000, because the
  compensation factor depends on the count (A-f-11). The explanation table prints it.
- With no `attributes` parameter, every score, draw and decision is unchanged (the `decisions` golden, AC-f-6).
- This is the DM reader of step-30 §6.1. No reference world's decision list uses it until a world chooses to.

### 5.8 The MCP adapter (`clients/agent`)

- `AttributeLine { attribute: WorldText, label: WorldText, kind: Literal[…], value: AttributeValueLine | None,
  text: WorldText | None }` — every world-authored string is `WorldText` (INV-S27-12), value slugs included.
- `PersonLine.description: WorldText | None` — who I see (§5.4), omitted when empty.
- `MeLine.attributes: list[AttributeLine]` — the own sheet, paged as list `attributes` (`ListName` gains it; page size
  32), always empty in a `ViewDelta`'s `me` like `acquaintances` (a change answers a whole view).
- **Omitted when absent.** Both fields use the models' `exclude_if` for `None` and `[]`, so a world without attributes
  renders byte-identical views (I-f-2); only the published schema changes, additively.
- `inspect` is unchanged: it returns the raw blocks within its 4 096 B (A-f-15).
- `disclosed.py`'s table gains the rows `traits` and "an owner's `attributes` member", read through §5.5.
- The tool golden `tests/golden/tools.json` is re-recorded in C7 on the base C0 records; the diff is reviewed in §16
  (the two new fields, the new list name, nothing else).

### 5.9 Both clients

- **The module** `mineworld/attributes.gd` (`class_name MineWorldAttributes`): `blocks(entity: Dictionary) -> Array`,
  `compose(entries, facet, budget) -> String`, `sheet(entries) -> PackedStringArray`, by §5.4, reading
  `MineWorldObservation.entity(id)`. It formats and cuts; it decides nothing (A-f-20). Copied byte for byte into both
  clients (F-S30f-6).
- **2D** (`clients/2d`):
  - the subject menu (`menu.gd`) shows the subject's glance under its title;
  - an inspect panel lists the subject's inspect list while the menu is open (the subject's panel, beside the menu);
  - `readers.gd` gains a reader for the own `traits` (the own sheet; title key `panel.traits`), and `SHOWN_ELSEWHERE`
    is unchanged; an owner's `attributes` member is shown in that owner's panel by the same sheet;
  - `--drive=panels` and `STEP_DONE` print the subject panel and the own sheet, so a Rust test reads them.
- **3D** (`clients/3d-spike`):
  - under "looking at <who>", a second HUD line shows the glance of the person the ray meets;
  - an inspect panel (`slice/inspect_panel.gd`, new) lists the inspect list of that person while the inspect control is
    held, and the own sheet when no person is aimed at; the control is a new entry of the shared controls map with an
    unbound default chosen in C8 and recorded (the operator judges it in play, as Q-16a-1 did for the reticle);
  - the slice probe gains `--probe=attributes`: aim at Alice, print `GLANCE`, `INSPECT` and `SHEET` lines, and the
    summary line `all attributes checks pass` that `ci_godot.py` judges.

### 5.10 `employment.occupation` (the named exception; Q-S30f-3)

- `systems/employment/attributes/occupation.yaml` (shipped with `ships_data!`): `occupation`, group `background`, facets
  `[prompt]`, `kind: { text: { max_bytes: 40 } }`, `visibility: learnable`, `mutability: derived`,
  `derive: projection`, `owner: employment`, `label: Occupation`, `phrase: "works as {value}"`, `prompt_rank: 205`.
- `job:` gains an optional `title:` (a string of 1 … 40 bytes, no control character), carried into `Job` and the
  `hired` fact **only when present** (`skip_serializing_if`), so every existing world's facts are byte-identical.
- `impl AttributeOwner for EmploymentSystem`: `project` returns `Text(title)` when the person is employed and the job has
  a title, else `None`.
- `discloses` adds `"attributes": <block>` to the holder's `employment` payload **only** when `disclose_owned` returns a
  block (Q-S30c-2: never without the `attributes` pack), so every reference world's observation is byte-identical.
- Nothing else in `employment` changes: no action, no fact type, no reduction (R-S30f-a-1). It does not subscribe to
  `attributes-handed`: a derived attribute is never handed (S30-c §5.7 hands resolved values; a stated `derived` value is
  `OutOfDomain`).

### 5.11 The scenario: one world, one oracle, five surfaces

- **The fixture world** `tools/cli/tests/fixtures/attributes-consumers/`: `social-cafe`'s café and street; three people;
  `systems: [presence, movement, conversation, naming, persona, employment, economy, attributes]`; requires
  `person-attributes: "^1"`; no profile (every value authored); `configure/attributes.yaml` with `backend: mineworld`,
  `mask: core`.
  - **Alice:** authored values across every group, including private (`religion`, `natal-sex`), learnable
    (`chronotype`), hidden (a world attribute `attributes-consumers.secret`), a `perceived_as` quantity (`stature`), a
    flag, a set, a world attribute `freckles` (step-30 §7.3, AC-S30-3), and a job with `title: barista`.
  - **Bob:** private and learnable values of his own, and the observer of Alice.
  - **Carol:** nothing authored (an entity with no block).
  - A second configuration `configure/attributes.masked.yaml` (selected by the test copying it over) with a world mask
    excluding `core.hair-colour`.
- **The oracle** `tools/cli/tests/fixtures/attributes-consumers/expected.json`, written by hand from the fixture's
  declarations (test rules §25), never produced by the code: Alice's glance and inspect list as Bob sees her; Alice's
  who I am, how I speak and own sheet; the private, learnable and hidden texts and ids that must be absent from Bob's
  every surface; the masked variant's absences.
- **Five surface tests, each against the oracle, each through the real server binary:**

| Surface | Test | Reads |
| --- | --- | --- |
| the MCP adapter | `clients/agent/tests/test_attributes_real.py` (`realserver.py`) | Bob's `observe` (`PersonLine.description`), `inspect` of Alice, Alice's `me.attributes` pages |
| an LM-free controller | `cognition/lm-controller/tests/test_attributes_real.py` (`realserver.py`, the fake backend in `tests/fakes`) | the logged `DecisionInput` and its built request for a consult of Bob (Alice present) and of Alice |
| the 2D client | `tools/cli/tests/client_2d_attributes.rs` (`godot2d` harness, `--drive=steps`) | Bob clicks Alice: the menu header and subject panel; Alice's own sheet |
| the 3D client | `--probe=attributes` through `scripts/ci_godot.py` (the CI `godot` layer) | `GLANCE`, `INSPECT`, `SHEET` lines |
| the decision model | `cognition/motivated/src/tests.rs` (a recorded observation from the fixture) | the explanation table: Alice's `Attribute` consideration on `agenda` |

  Equality with one oracle makes the surfaces equal to each other: P-3 and the step's checkpoint.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-f-1 (I-S30-10) | Every reference world reads, loads, runs and observes byte-identically: no reference world enables `attributes`; `employment`'s `title` and `attributes` member appear only when present | the existing world tests and the `decisions` golden unchanged; one nightly `baseline check` on the final head (C11) |
| I-f-2 | Without attribute blocks, every consumer's output is byte-identical: the prompt golden, every cassette key, every agent view, both clients' panels | `prompt_request.json` and the cassettes unchanged; `test_view.py` goldens unchanged; M-f-8, M-f-9 |
| I-f-3 (I-S30-3, X-S30-4) | A `private`, `learnable` or `hidden` attribute of a person never appears on another observer's surface; `hidden` never on the holder's | the five surface tests' absence checks; M-f-1 |
| I-f-4 (P-3) | What a human client shows of an attribute, the agent is shown: glance = `PersonLine.description`, inspect panel = `inspect`, own sheet = `me.attributes` | the oracle equality of §5.11; M-f-5 |
| I-f-5 (P-1, P-2) | No consumer names an attribute id, a backend or `matraix`; selection is by facet; a masked-out attribute is on no surface | AC-f-9's scan; the masked variant of §5.11 |
| I-f-6 (F-S30f-8) | No consumer renders a value: Python and GDScript select, join and cut server text only | AC-f-9's scan (no `{value}`, no `words`, no `perceived_as` in consumer code) |
| I-f-7 (I-S30-5) | No decision model reads `text` | `Disclosed` has no text accessor; review |
| I-f-8 (I-S30-9) | No attribute becomes a memory record through a component | AC-f-11 |
| I-f-9 | Every composition is deterministic: the same observation gives the same bytes on three platforms | the composition golden in three languages; CI on three OSes |
| I-f-10 | `employment` changes only as §5.10 states | `git diff --stat` over `systems/employment` reviewed in C3; no new action, fact type or reduction |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy `-D warnings` on `mineworld-attributes-model`, `mineworld-attributes`, `mineworld-employment`, `mineworld-motivated`, `mineworld-cli`, `mineworld-acceptance`; `ruff` and `pyright` on `sdk/python`, `cognition/lm-controller`, `clients/agent`; Godot's parse check through the existing `godot` layer | per commit; CI `fast` |
| Unit (Rust) | schema 2's fields and order; the reader's kind checks; the composition golden (reader half); `motivated`'s parameter decoding and its refusals | `authoring/attributes/src/*/tests.rs`; `cognition/motivated/src/parameters_tests.rs` |
| Unit (Python) | the reader's strict decoding and skip-whole rule; the composition golden; the prompt parts; the agent's lines and paging | `sdk/python/tests/test_attributes.py`; `cognition/lm-controller/tests/test_controller_attributes.py`; `clients/agent/tests/test_view.py` (new cases) |
| Unit (GDScript) | the composition golden | `clients/protocol/checks/attributes_check.gd`, run by the `godot` layer |
| Golden | the composition golden (hand-written); `prompt_request.json` (unchanged) and `prompt_request_attributes.json` (new, hand-written); `tools.json` (re-recorded, diff reviewed) | as named |
| Crate (pack) | `employment.occupation`'s projection and disclosure; absence without `attributes` | `systems/employment/tests/occupation.rs` |
| Structural | AC-f-9's scans; the protocol module's copies identical | `tests/acceptance/tests/client_rules.rs`; `tests/acceptance/tests/attribute_consumers.rs` (new) |
| Integration (real binary) | the five surface tests of §5.11; the masked variant | as §5.11 |
| Regression | I-f-1, I-f-2 | existing tests unchanged; one nightly dispatch (C11) |
| Gate 1 (real LLM) | NOT REQUIRED: every prompt part is deterministic text compared byte for byte; an LM is one consumer of it (step-30 §9.2 "No LM is needed") | — |
| Gate 2 (real lifecycle) | NOT REQUIRED beyond the real-binary tests: nothing here has timing or durability semantics | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-f-1 | **Checkpoint.** In the fixture world, Bob's MCP `PersonLine.description` for Alice, Bob's LM-free controller's "People here" bracket for Alice, the 2D menu header Bob sees on clicking Alice and the 3D glance line while Bob aims at her each equal the oracle's glance byte for byte; Bob's MCP `inspect` texts, the 2D subject panel and the 3D inspect panel each equal the oracle's inspect list | S30-f row |
| AC-f-2 | **Private absent everywhere (X-S30-4 × 4).** None of Alice's `private`, `learnable` or `hidden` ids or texts (`religion`, `natal-sex`, `chronotype`, `secret`, `occupation`) appears in any of Bob's surfaces; none of Bob's appears in Alice's; the plant M-f-1 turns every one of the five tests red, each naming the attribute and the surface | S30-f row; AC-S30-10 |
| AC-f-3 | **The holder.** Alice's request contains `What the world says about you: ` with her `prompt`-faceted texts in rank order — including `works as barista` — and no `ancestry`, `secret` or casting-only text; `Your manner: ` with her `speech` texts; her 2D and 3D own sheets and MCP `me.attributes` list every non-hidden entry, equal to the oracle's own sheet | §9.2; AC-S30-11 |
| AC-f-4 | **Budgets (X-S30-17).** On the golden's 200-entry block: who I am ≤ 600 B, how I speak ≤ 160 B, who I see ≤ 160 B; each cut at an entry boundary with ` …`; every kept entry's rank ≤ every dropped entry's; identical in Rust, Python and GDScript | X-S30-17 |
| AC-f-5 | **Nothing without attributes.** `prompt_request.json` equals the built request with no blocks; every cassette under `cognition/lm-controller/tests/cassettes/` replays without a miss; `test_view.py`'s existing goldens pass unchanged | I-f-2 |
| AC-f-6 | **The DM reader.** With `attributes: [{ attribute: core.chronotype, option: agenda, by: {…} }]`, Alice (`early-bird`) carries `Attribute` on `agenda` with 1 000 and a `night-owl` fixture with 450; Carol (no value) carries none; with no `attributes` parameter the `decisions` golden (`tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt`) is unchanged | §6.1 DM; A-f-11 |
| AC-f-7 | **Occupation.** Alice's own surfaces show `works as barista`; Bob's show nothing of it; in a world without `attributes`, `employment`'s payload and `hired` fact are byte-identical to `main`'s | §9.4; Q-S30a-2 |
| AC-f-8 | **Masks (P-1, AC-S30-15).** Under the masked configuration, `grey hair` and `core.hair-colour` are on none of the five surfaces, and every other oracle line is unchanged | AC-S30-15 |
| AC-f-9 | **Structural.** Over `cognition/lm-controller/src`, `cognition/motivated/src`, `sdk/python/src`, `clients/agent/src`, `clients/*/scripts`, `clients/*/mineworld`: no literal matching `core\.[a-z]`, no `matraix` (any case), no `{value}`, no `perceived_as`; the three `mineworld/` copies are byte-identical | I-f-5, I-f-6; F-S30f-6 |
| AC-f-10 | **Reference worlds.** The nightly `baseline check` on the final head passes with `scripts/baselines.txt` unchanged | I-f-1 |
| AC-f-11 | **Memory.** A seat that perceives Alice's `traits` for 100 observations stores no record from it; a `trait-changed` fact is ingested with the generic renderer and is citable | F-S30f-5; I-S30-9 |
| AC-f-12 | **`freckles` reaches every consumer (AC-S30-3, the consumer half).** The fixture's world attribute appears in Bob's glance and inspect list, Alice's own sheet and her who I am, with no line of consumer code naming it | AC-S30-3 |
| AC-f-13 | **Size recorded.** Schema 1 and schema 2 byte sizes of Alice's own block and of Bob's view of her block, on the 93-attribute fixture, in §16; §5.1's 2 048 B rule applied if it trips | R-S30-3 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s30f`. Never `cd` elsewhere.

1. Re-read this commit's section and every file before editing it.
2. Make the numbered steps in order; nothing of a later commit.
3. Run the commit's validation one `--test`/`--lib` or one `pytest` file at a time; record command, count, wall time
   and PASS / FAIL / INCONCLUSIVE. `cargo clippy -p <touched crates> --all-targets -- -D warnings`; `cargo fmt --all
   --check`; `uv run ruff check <touched package>`; `uv run pyright <touched package>`. **R-LOAD-1:** never `cargo test
   --workspace`, never a whole Python suite, never the nightly locally.
4. Mutations: copy the file, plant, run the named test red (record its name), restore from the copy, `cmp`, run green.
5. Answer the commit's review questions in §16 in one or two sentences each.
6. `git status --short`: every changed path is in the commit's **Files**; otherwise revert or stop and record why.
7. Update §16 and the handoff in the same commit; stage by path; commit with the message and the session's attribution
   line. No heredoc, no `sed -i`, no `python3 -c`, no `curl`.

#### C0 — Base and ledger

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30f.md` (new).
- Steps: record the base; confirm S30-c, S30-d, S30-e merged and record their merge commits; re-read A-f-1 … A-f-26 on
  the base and record every moved line; confirm (F-S30f-2) how `disclose_owned`'s block is embedded and whether S30-c
  decodes `prompt_rank`; record whether S27-d1 (#261) has merged; confirm §15's rulings.
- [ ] Implementation. [ ] Validation: `python3 scripts/check_doc_headings.py`, `check_decision_ids.py`.
  [ ] Review: the base satisfies §2.3; every ruling of §15 is applied to §§5, 8 before C1.
- **Commit message:** `docs(plan): S30-f — open the ledger at the base`

#### C1 — Schema 2 of the disclosed block, and the Rust reader

- **Files:** `authoring/attributes/src/{render.rs, disclosed.rs (new), lib.rs}` and their `tests.rs`;
  `authoring/attributes/src/declaration.rs` (only if C0 found `prompt_rank` undecoded);
  `authoring/attributes/tests/fixtures/composition.json` (new, hand-written); `systems/attributes/tests/pack.rs` (the
  expected payloads); `tools/cli/tests/attributes.rs` (S30-c's expected frames move to schema 2, a reviewed edit).
- Steps: §5.1 (fields, rank order, `instant` without text per Q-S30f-4, the owner block's schema); §5.2 (`Disclosed`);
  the composition golden (§5.4) with the reader-half test (merge and order); re-measure AC-f-13.
- Validation: `cargo test -p mineworld-attributes-model --lib`; `-p mineworld-attributes --test pack`; `-p mineworld-cli
  --test attributes` (S30-c's checkpoint, now schema 2: AC-c-1 … AC-c-14 still pass with only the payload shape moved).
  Mutation **M-f-3** (entries ordered by id, not rank) → the order test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no view widened (the holder and observer filters are S30-c's, untouched);
  `facets` are sorted and canonical; `Disclosed` has no `text` accessor; the 2 048 B rule recorded either way.
- **Commit message:** `feat(attributes-model): the disclosed block carries label, facets and rank, in rank order; a typed reader (ARC-122, S30-f)`

#### C2 — The built-in catalogue's ranks

- **Files:** `entities/person-attributes/attributes/{appearance, background, psychology, capability, behaviour,
  lifestyle}.yaml`, `entities/person-attributes/pack.yaml` (`1.1.0`), `tools/cli/tests/attributes_builtin.rs` (S30-e's
  catalogue test, one new assertion).
- Steps: §5.3's table, exactly; nothing else in the files.
- Validation: `cargo test -p mineworld-cli --test attributes_builtin`; `target/debug/mineworld attributes check` on
  S30-e's fixture world (exit 0). A new assertion: every `observed`, `prompt` and `speech` built-in has a rank except
  `hair-style` and `hair-texture`, and no two built-ins share one.
- [ ] Implementation. [ ] Validation. [ ] Review: the ranks are §5.3's; the count of `core` rows is still 93
  (X-S30-20); the version is a minor bump.
- **Commit message:** `feat(person-attributes): prompt ranks for what an observer and a prompt read first (1.1.0, S30-f)`

#### C3 — `employment.occupation`

- **Files:** `systems/employment/{Cargo.toml, src/section.rs, src/component.rs, src/event.rs, src/system.rs,
  src/lib.rs, attributes/occupation.yaml (new), tests/occupation.rs (new)}`; `Cargo.lock`.
- Steps: §5.10 exactly (as Q-S30f-3 rules it).
- Tests: Alice with `title` in a world with `attributes`: her own observation's `employment` payload carries the block
  with `works as barista`; Bob's observation carries no `employment` for her; without `attributes` the payload has no
  `attributes` member; a job without `title` decodes, reduces and serializes byte-identically to `main`'s (the
  `hired` fact's bytes compared with a hand-copied fact from `main`); a `title` over 40 bytes is refused at its line.
- Validation: `cargo test -p mineworld-employment --test occupation`; `-p mineworld-employment --lib`; `-p
  mineworld-worldpack --test social_cafe`; `-p mineworld-cli --test decisions`. Mutation **M-f-9** (`discloses` always
  writes `attributes`, `null` when absent) → the byte-identity test red.
- [ ] Implementation. [ ] Validation. [ ] Review: `git diff --stat -- systems/employment` shows only these files; no
  new action, fact type or reduction (I-f-10).
- **Commit message:** `feat(employment): employment.occupation, projected from an optional job title (Q-S30a-2, S30-f)`

#### C4 — The Python reader and composer

- **Files:** `sdk/python/src/mineworld_sdk/{attributes.py (new), __init__.py}`, `sdk/python/tests/test_attributes.py`
  (new), `sdk/python/README.md` (one line).
- Steps: §5.5; the composition golden read from `authoring/attributes/tests/fixtures/composition.json` by a path
  relative to the repository root.
- Validation: `uv run pytest sdk/python/tests/test_attributes.py`; ruff; pyright. Mutation **M-f-4** (the cut keeps a
  partial entry) → the boundary case red.
- [ ] Implementation. [ ] Validation. [ ] Review: strict decoding; skip-whole on any unknown shape; no id, backend or
  phrase logic (I-f-5, I-f-6).
- **Commit message:** `feat(sdk): read and compose disclosed attributes (S30-f)`

#### C5 — Cognition: who I am, how I speak, who I see

- **Files:** `cognition/lm-controller/src/mineworld_cognition/controller/{attributes.py (new), situation.py, prompt.py,
  consult.py, seat.py, log.py}`; `cognition/lm-controller/tests/{test_controller_attributes.py (new),
  fixtures/prompt_request_attributes.json (new, hand-written), test_memory_render.py (AC-f-11 case)}`.
- Steps: §5.6; AC-f-11's memory case.
- Validation: `uv run pytest cognition/lm-controller/tests/test_controller_attributes.py`, then
  `test_controller_choice.py` (the old golden, unchanged), `test_record_replay.py` (cassettes), `test_memory_render.py`,
  one at a time; ruff; pyright. Mutations **M-f-8** (a `What the world says about you:` line with an empty `about`) →
  `prompt_request.json` red; **M-f-2** (who I see composed from the observer's own entity) → the description case red.
- [ ] Implementation. [ ] Validation. [ ] Review: every caller of `build_request` passes `attributes`; an old log
  replays; texts pass through the delimiter replacement; budgets are the constants of §5.6.
- **Commit message:** `feat(cognition): who I am, how I speak and who I see from disclosed attributes (S30-f)`

#### C6 — The decision model's attribute consideration

- **Files:** `cognition/motivated/{Cargo.toml, src/parameters.rs, src/parameters_tests.rs, src/settings.rs,
  src/considerations.rs, src/explain.rs, src/tests.rs}`, `Cargo.lock`.
- Steps: §5.7.
- Validation: `cargo test -p mineworld-motivated --lib`; `-p mineworld-cli --test decisions`. Mutation **M-f-6** (a
  neutral 1 000 appended when the value is absent) → the `decisions` golden red (via the parity case in
  `tests.rs`, which runs the golden's world with the parameter naming an attribute nobody holds).
- [ ] Implementation. [ ] Validation. [ ] Review: no attribute id is a literal in `src/`; absent appends nothing; the
  refusals keep line and column.
- **Commit message:** `feat(motivated): an attribute consideration, declared in the model's parameters (S30-f)`

#### C7 — The MCP adapter

- **Files:** `clients/agent/src/mineworld_agent/{models.py, view.py, disclosed.py}`, `clients/agent/tests/{test_view.py,
  test_info.py, golden/tools.json}`, `clients/agent/README.md` (one line).
- Steps: §5.8; rebase on S27-d1 if it has merged since C0; re-record `tools.json` with the adapter's own catalogue dump
  and paste the reviewed diff summary into §16.
- Validation: `uv run pytest clients/agent/tests/test_view.py`, `test_info.py`, `test_mcp_tools.py`, one at a time; ruff;
  pyright. Mutation **M-f-5** (`description` cut at 80 B) → the parity case against the composition golden red.
- [ ] Implementation. [ ] Validation. [ ] Review: every world string is `WorldText`; omitted when empty; `tools.json`'s
  diff is the two fields and the list name only.
- **Commit message:** `feat(agent): a person's description and my attributes in the view (S30-f)`

#### C8 — Both clients

- **Files:** `clients/protocol/mineworld/attributes.gd` (new) and its copies `clients/2d/mineworld/attributes.gd`,
  `clients/3d-spike/mineworld/attributes.gd` (with `.uid` files); `clients/protocol/checks/attributes_check.gd` (new);
  `clients/2d/scripts/{menu.gd, hud/readers.gd, hud/words.gd, harness/interact.gd}`;
  `clients/3d-spike/scripts/slice/{slice_main.gd, inspect_panel.gd (new), slice_probe_world.gd}`; the shared controls
  map entry; `scripts/ci_godot.py` (`attributes` in `MODES`); `tests/acceptance/tests/client_rules.rs` (the copies
  identical, F-S30f-6).
- Steps: §5.9.
- Validation: the GDScript composition check through `python3 scripts/ci_godot.py` (the check runner) on this machine's
  platform; `cargo test -p mineworld-acceptance --test client_rules`. Mutation **M-f-10** (one copy of
  `attributes.gd` differs by a byte) → `client_rules` red naming the file.
- [ ] Implementation. [ ] Validation. [ ] Review: no rule word or action type (A-f-20); nothing evaluated; panel texts
  are the composer's output only.
- **Commit message:** `feat(clients): a glance, an inspect panel and my own sheet in 2D and 3D (S30-f)`

#### C9 — The scenario through the real server

- **Files:** `tools/cli/tests/fixtures/attributes-consumers/**` (world, people, attributes, configurations, the oracle);
  `clients/agent/tests/test_attributes_real.py`; `cognition/lm-controller/tests/test_attributes_real.py`;
  `tools/cli/tests/client_2d_attributes.rs`; `clients/3d-spike/scripts/slice/slice_probe_world.gd` (the probe's world
  argument); `cognition/motivated/src/tests.rs` (the recorded observation); `tests/acceptance/tests/
  attribute_consumers.rs` (AC-f-9's scan).
- Steps: §5.11; AC-f-1, AC-f-2, AC-f-3, AC-f-7, AC-f-8, AC-f-12.
- Validation: each surface test alone; the 3D probe in CI (`godot` layer) — locally only when Godot is installed,
  recorded `NOT RUN` otherwise. Mutation **M-f-1** (the observer view admits `learnable`, planted in `render.rs`) →
  all five red, each naming `attributes-consumers` attribute and surface (the 3D one read from CI's artifact); **M-f-7**
  (a `core.hair-colour` literal in `readers.gd`) → the scan red.
- [ ] Implementation. [ ] Validation. [ ] Review: the oracle is hand-written; each surface reads the real server;
  no test computes its expectation with the code under test.
- **Commit message:** `test(attributes): one world, one oracle, five surfaces — the same phrases for Alice, Bob's secrets nowhere (S30-f)`

#### C10 — Specification sync

- **Files:** `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`, `clients/protocol/README.md` (one line).
- Steps: dated notes — `MODULE_SPEC` §4.1 *Disclosure* (schema 2; rank order; owner blocks under `attributes`),
  *Rendering* (an `instant` has no text, Q-S30f-4), §3.1 "Owns" (`employment` is an owner; `persona` not yet, F-S30f-7),
  §5.1 (the `Disclosed` reader and `motivated`'s `attributes`); `DECISIONS.md` under `ARC-120` (`employment`,
  `title`), `ARC-122` (schema 2, the instant rule), `ARC-125` (its consumers shipped). Old text kept visible.
- Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Implementation. [ ] Validation. [ ] Review: every finding of §3.2 that changed specified text has its note; no
  specification text and code disagree (`CLAUDE.md` §2.1 rule 4).
- **Commit message:** `docs(spec): attribute consumers as implemented — schema 2, owners, the decision reader (S30-f)`

#### C11 — Close-out

- **Files:** this file (§16), the handoff.
- Steps: AC-f-9's scan; `git diff --stat origin/main...HEAD -- systems/ ':!systems/employment'` empty; push
  `mvp0/pr-s30f`; `gh pr create`; CI on the exact head (three OSes, including the `godot` layer) green; one `nightly.yml`
  dispatch on the final head for AC-f-10. The generated reference (`docs/reference/`) moves for `motivated`'s parameter
  and `employment`'s `title`; it is regenerated by CI after merge, as for every schema change.
- [ ] Implementation. [ ] Validation: CI and the nightly verdict. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-f close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time; `cargo clippy` on the
touched crates; `ruff` and `pyright` on the touched Python packages. Never `cargo test --workspace`, never a whole Python
suite, never the nightly locally. The PR's CI on the exact head (Linux, Windows, macOS, and the `godot` layer) is the
gate; one `nightly.yml` dispatch on the final head is the only long run.

---

## 11. Requirements this PR places on other PRs

- **R-S30g-f-1 (S30-g).** A reference world adopting attributes moves its observations (schema 2 blocks) but no
  cognition cassette until a consult happens with a block present: S30-g re-records the cassettes it moves with reasons,
  as step-30 §11.2 states for prompt goldens; `title:` is added to the reference worlds' jobs there, not here.
- **R-S30h-f-1 (S30-h).** The MatrAIx overlay gives its masked dimensions facets (`observed`, `prompt`, `speech`) and
  ranks, or they reach the inspect list and the own sheet only; no consumer code changes for MatrAIx (I-f-5).
- **R-S30i-f-1 (S30-i).** A learned attribute enters the learner's memory and prompt through memory, never through
  `SelfAttributes` or `Person.description` (I-S30-9).
- **R-S30k-f-1 (S30-k).** Localisation replaces `label` and `text` server-side; the composition separator `", "` and the
  cut marker ` …` become locale words of the label pack; the three implementations read them from the block, not from
  code.
- **R-QS30-4 (the persona review).** Whether `persona` becomes an owner, and whether `persona.traits` and the psychology
  group both reach the prompt, is decided with this PR's prompt in hand.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §9.2 | "The SDK gains a typed reader" → the model crate's `Disclosed` (D-f-3); the per-controller extra facets not offered (D-f-5) |
| `step-30-person-attributes.md` §9.3 | the glance/inspect pairing (D-f-2); `inspect` unchanged |
| `step-30-person-attributes.md` §9.4 | the job label is an optional `title` (Q-S30f-3) |
| `step-30-person-attributes.md` §5.6 | "mirrored in Python by golden tests" → composition, not rendering, is mirrored (F-S30f-8) |
| `pr-s30e-builtin-catalogue.md` Appendix A row 23 | "S30-f renders a date" → an instant renders no text (Q-S30f-4) |
| `overall.md` | S30-f frozen / merged |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-f — attribute consumers: cognition, the decision-model reader, the MCP
                         adapter and both clients
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30f-attribute-consumers.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§4.4, 5, 6.1, 7.3, 9, 11.2, 12, 13); pr-s30c-attributes-pack.md;
                         pr-s30d-sampler.md; pr-s30e-builtin-catalogue.md; pr-s27d-mcp-stdio.md and, if merged,
                         pr-s27d1-single-act.md; pr-s10-p6-lm-controller.md §5.2; pr-dl-b-motivated.md §§5.3–5.5, 6;
                         docs/CORE_CONCEPTS.md §4.5; docs/MODULE_SPEC.md §§3.1, 4.1, 5.1; docs/DECISIONS.md ARC-120 …
                         ARC-126; docs/MVP.md AC-16; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         .structured-coding/standards.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30f, its own, held by one session
BRANCH:                  mvp0/pr-s30f, created from origin/main
START CONDITION:         S30-c, S30-d and S30-e merged (Q-S30f-5) AND this design frozen AND §15 ruled, then the
                         primary's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–10, as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-3, I-S30-5, I-S30-8, I-S30-9, I-S30-10; §6 I-f-1 … I-f-10; §8 as written
SEQUENCE:                C0 … C11 (§9)
IMPLEMENTER:             Opus-class session; C4, C5, C7, C8 may be delegated to Haiku-class sub-agents from §9, each
                         reviewed by the session before its commit; reviewer of the PR: the primary session
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *;
                         uv run pytest <one file>; uv run ruff; uv run pyright; python3 scripts/ci_godot.py (checks,
                         probe); git; gh (pr create/view; workflow run nightly.yml once in C11; run download; never
                         merge); python3 scripts/check_doc_headings.py, check_decision_ids.py, check_scratch.py,
                         ci_layer.py docs, ci_parity.py baseline check; cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1); a
                         whole Python suite; any edit under kernel/, contracts/, server/, worlds/, or any systems/<pack>
                         other than employment; any reference-world edit; any model call; a new external dependency
MATERIAL STOPS:          S30-c, S30-d or S30-e not merged; a reference world's bytes moving; a cassette or the prompt
                         golden moving for a world without attributes; §5.1's size rule exceeded; `disclose_owned`'s
                         merged shape incompatible with D-f-1 beyond a key name; a needed change to an existing System
                         other than §5.10's; a needed kernel, contracts or server change
PLATFORMS:               Linux, macOS, Windows (CI core and godot layers on all three)
VALIDATION BUDGET:       crate, Python and client tests unrestricted one file at a time; one nightly dispatch (C11).
                         Gate 1 and Gate 2: NOT REQUIRED (§7)
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30f.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized (frozen 2026-10-18)
                         semantic commits: authorized      (working rules §14)
                         branch push: authorized           (working rules §21)
                         PR creation / update: authorized  (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: TRAIN-READY, reported to the coordinator
                         merge: never by the implementation session — only through a coordinator-assembled merge train
                         (operator CI policy 2026-10-18; merge grant 2026-10-16)
POST-MERGE SYNC OWNER:   primary: step-30, pr-s30e (row 23's note), overall.md; implementation session: this ledger
                         and the merge identity
STOP CONDITION:          S30-f READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-f-1 | Schema 2 grows every observation frame that re-sends a perceived person (an entity is upserted whole) | measured in C1 (AC-f-13); §5.1's pre-approved trim; beyond it a material stop |
| R-f-2 | Three implementations of the composition drift | one hand-written golden read by all three; a rule change is one golden edit and three red tests |
| R-f-3 | S27-d1 and C7 edit the same files | C0 records #261's state; C7 rebases; the models S30-f changes are ones S27-d1 leaves untouched (A-f-16) |
| R-f-4 | A world's phrase text forges a prompt boundary | delimiter replacement on every attribute text (§5.6); a test plants `<<<end>>>` in a phrase |
| R-f-5 | The 3D client's inspect control collides with a future binding | a controls-map entry, not a hard-coded key; the operator judges it in play |
| R-f-6 | S30-c's owner embedding differs from D-f-1 | C0 audits it; a key-name difference is bounded; an incompatible shape is a material stop |
| R-f-7 | Ranks become an accidental standard | ranks are data with a minor version; spaced by ten for overrides (§5.3) |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

Ruled by the operator on 2026-10-18 in the primary session: every question below is answered with its
recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30f-1 [the wire; `MODULE_SPEC` §4.1 *Disclosure*; S30-c's frozen payload]** | A consumer must select by facet and cut by rank (R-S30f-e-1, X-S30-17), but the disclosed block carries neither and is in alphabetical order (F-S30f-1); a remote seat has no other route to the catalogue. How do consumers get them? | **Schema 2: every entry gains `label`, `facets` and `rank`, and entries come in rank order** (§5.1); the views are unchanged. Generic, one shape for every consumer and owner, and selection stays with the consumer. Cost is measured, with a pre-approved trim at 2 048 B per observed person. Declined: (b) the server composes the glance and the holder's lists (cheaper, but each owner's block would be composed apart, so a description could not merge an owner's attribute, and cognition's selection would live in the world); (c) Python and GDScript mirror the renderer and read the catalogue (no route for a remote seat; a second and third renderer) |
| **Q-S30f-2 [where the Python reader lives]** | Cognition and the MCP adapter both need one reader and composer. The Python SDK is domain-agnostic today (A-f-26). | **A module `mineworld_sdk.attributes` in the Python SDK**, documented as the reader of `MODULE_SPEC` §4.1's block. Both consumers already depend on the SDK; the alternatives are two copies (cognition and the agent) or a new distribution for some 150 lines. The Rust half lives in the model crate (D-f-3) |
| **Q-S30f-3 [`employment.occupation`; Q-S30a-2's "nothing else changes"]** | The ruling projects occupation from "the job's label", but a job has no label (A-f-22). | **Add an optional `title:` (≤ 40 bytes) to the authored `job:`; `employment.occupation` is a `derived`, `learnable` `text` projected from it** (§5.10). Absent title, absent attribute; every existing world is byte-identical. S30-g adds titles to the reference worlds. Consequence: S31's optional N-6 (a role default keyed on occupation) needs a `category` and is not served; if the operator wants N-6, occupation becomes a `category` over a vocabulary `employment.occupations` instead, which `employment` would ship and every world would be bound to |
| **Q-S30f-4 [rendering an `instant`; S30-e row 23]** | S30-e said "S30-f renders a date" for `birth`, but a date needs the world's calendar epoch, which the model crate cannot read (F-S30f-4). | **An `instant` renders no text.** Its typed value still reaches the holder's typed surfaces (`me.attributes`, the own sheet shows `Birth: <typed value>`), and `age-years` and `age-band` carry it in prose. A calendar-aware date can come with S30-k's locale words or with the calendar pack's own reader later |
| **Q-S30f-5 [start condition]** | Step-30 §13.3 lets S30-f start once S30-c has merged; the brief of 2026-10-18 makes it depend on S30-c, d and e. | **Start after S30-c, S30-d and S30-e have all merged**, so the scenario runs on the real `core` catalogue with its facets and the real mask loader (AC-f-8). The only cost is a later start; S30-f's design needs nothing from d or e that a fixture could not fake, which is why this is a scheduling choice and not a material dependency |

Bounded decisions recorded here rather than asked: D-f-1 (attribute blocks, F-S30f-2), D-f-2 (glance = view, inspect
panel = `inspect`, F-S30f-9), D-f-3 (the Rust reader in the model crate), D-f-4 (`motivated` appends nothing when a value
is absent, A-f-11), D-f-5 (no per-controller extra facets in S30-f); `persona` stays as P6a made it (F-S30f-7, Q-S30-4
ruled).

---

## 16. Ledger (live during implementation)

### Commit ledger

Base: `origin/main @ 212be484` (#362). S30-c #238, S30-d #247, S30-e #280 and #293/#300 merged. C0's ledger opening is folded
into this close-out (speed-first charter, 2026-10-18).

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C1 schema 2 + `Disclosed` + golden | 30096573, 39d7ab28 | [x] | [x] model lib 29/29 | [x] no view widened; facets sorted; no text accessor |
| C2 ranks (1.1.0) | 917df6ec | [x] | [x] attributes_builtin 7/7 | [x] 60 ranked; 93 core rows |
| C3 `employment.occupation` | e4639405 | [x] | [x] occupation 3/3, lib 2/2 | [x] no new action, fact type or reduction |
| C4 Python reader | a5c1ef95 | [x] | [x] test_attributes 22/22 | [x] strict, skip-whole |
| C5 cognition | d4b17742 | [x] | [x] prompt golden and cassettes unchanged | [x] old logs replay |
| C6 `motivated` | e98d86e9 | [x] | [x] lib 45/45 | [x] absent appends nothing |
| C7 MCP adapter | d296409e | [x] | [x] test_view 22, test_info 20, test_mcp_tools 15 | [x] reference regenerated |
| C8 clients | e15936d4 | [x] | [x] GDScript golden check PASS (Godot 4.7.2) | [x] no rule word; symlinked module |
| C9 scenario | 838ea0fc | [x] | [x] attributes_consumers 2/2; acceptance attribute_consumers 2/2, client_rules 3/3; client_2d_attributes ignored (CI godot layer) | [x] oracle hand-written |
| C10 spec sync | 1e1532bb | [x] | [x] check_doc_headings, check_decision_ids | [x] |
| C11 close-out | this commit | [x] | CI `fast` on the head | [x] |

### Evidence

Local, one test file at a time (R-LOAD-1): see the table. clippy `-D warnings` on the five touched crates clean; `cargo fmt
--check` clean; ruff and pyright clean on the three Python packages (implementer's run). `cargo test -p mineworld-cli --test
decisions`: `with_no_decision_list_run_prints_what_the_base_printed` FAILS — pre-existing on main: the golden was last
regenerated at d354b3a2, and `worlds/social-cafe` changed after it (23bf3bcd, eb7a7dce, S29-a); social-cafe enables none of
this PR's packs. Classified FAIL (not this PR's), left for the milestone batch.

### Mutations

M-f-3 (rank order) is the render test `schema_two_entries_carry_label_facets_and_rank_in_rank_order`; M-f-4, M-f-8, M-f-2,
M-f-5 planted by the Python implementer, each red. M-f-6 is held by `an_attribute_consideration_is_added_only_when_my_value_is_disclosed_and_named`.
Others not planted (speed-first charter: mutation evidence is not required per PR).

### Deviations and findings

F-S30f-1 … F-S30f-9 (§3.2), recorded at design time. During implementation:

- D-impl-1. `render::Disclosed` (the payload) is renamed `render::Block`; `disclosed::Disclosed` is the reader (§5.2's name).
  Both Rust structs `deny_unknown_fields`, matching the Python and GDScript readers (a block is read whole or not at all);
  the golden gained `unknown-field`.
- D-impl-2. The own sheet leaves out an entry that shows nothing (an empty set); golden case `empty-words` row `t.d`.
- D-impl-3. F-S30f-6: `clients/2d/mineworld` and `clients/3d-spike/mineworld` are symlinks to the protocol module, so
  they cannot drift; `attribute_consumers.rs` holds that (symlink, or byte-identical copy).
- D-impl-4. AC-f-9's `{value}`/`perceived_as` scan is narrowed to the attribute readers (files named `*attribute*`): Rust
  format strings and an f-string in `__main__.py` legitimately use `{value}`. Test files are not scanned for `core.` ids.
- D-impl-5. The masked variant's configuration and the oracle sit beside the world
  (`attributes-consumers.masked.yaml`, `.expected.json`), because the loader refuses unknown files inside a World Pack;
  under the mask the person files drop `hair-colour` too (a masked-out attribute cannot be stated, AC-e-6).
- D-impl-6. The 3D inspect control is `inspect`, default key X (V is the camera's); 2D shows the subject's glance and
  inspect list inside its menu; the MCP tool golden is `docs/reference/agent-tools.{json,md}` (the old
  `tests/golden/tools.json` no longer exists).
- D-impl-7. NOT done in this PR, for the milestone batch: the 3D `--probe=attributes` slice-probe mode, and the two
  Python real-server surface tests (`clients/agent/tests/test_attributes_real.py`,
  `cognition/lm-controller/tests/test_attributes_real.py`). The Rust real-server test (`attributes_consumers.rs`) and the
  2D Godot test (`client_2d_attributes.rs`) compare the server and the 2D client with the oracle; the Python composition
  is held to the same golden by `test_attributes.py`. The DM test uses option `stay` rather than `agenda` (same code
  path, no agenda fixture needed).
- D-impl-8. The nightly `baseline check` (AC-f-10) is not dispatched; no reference world enables `attributes`.

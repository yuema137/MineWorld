# PR 13c (part 1) — R-PK-1: a doorway says where it leads

## DESIGN FROZEN 2026-10-10 (primary session, operator ruling Q1)

```text
Lifecycle:              DESIGN FROZEN 2026-10-10 (primary session, operator ruling Q1)
Approved by:            the primary session's rulings of 2026-10-10 on §11 (Q1 by the operator; Q2–Q7 by the
                        primary session), recorded in §11.1
Parent:                 step-13-client-2d.md §6.3 (R-PK-1), §4.3 (point 3), §9 row 13c, QS12-7
Audit base:             origin/main @ 98fe3e6 (Merge PR #155)
Requirement ID:         R-PK-1 (A-23). R-PK-2 (item names, F-41) is OUT of scope: it waits for 12d.
Decision record:        ARC-82 (docs/DECISIONS.md), recorded in C3
Implementation branch:  mvp0/pr-13c-names
Implementation worktree: /Users/yuema137/mineworld-worktrees/impl-13c-names
Execution contract:     §13
```

This document is the PR design for the first half of step-13's PR 13c, "Names for things". It is the
single authority for this PR's scope, contract change and acceptance. Its scope, invariants and
acceptance are frozen; progress, evidence and bounded corrections stay writable (`CLAUDE.md` §3.1).
Any change to a frozen invariant, a public contract, an ownership boundary or scope goes back to the
operator before it is acted on.

---

## 1. Problem

A person standing in the street sees five doorways, one per place (`worlds/market-town/places/*.yaml`,
A-30). Today a passage says only `to: <PlaceId>` (`systems/movement/src/component.rs`, `Passage`). The
destination's tags are not perceived from outside that place (`systems/presence/src/observe.rs`,
`perceived`: only the observer's own place and the people in it carry tags). So the client cannot tell
which doorway is the café, cannot choose a façade before entering (step-13 §4.3 point 3), and labels
every neighbour the same way.

The label defect is already visible in the 2D client today (`clients/2d/scripts/scene/places.gd`
`_door_text`, lines 78–83). It is worse than "no name":
it names the destination by its alphabetically last tag, which for most places is the word `public`.

## 2. Scope

In scope:

1. **Server, `systems/movement`.** The `passages` disclosure on a place gains, per `leads_to` entry, the
   destination place's tags (`to_tags`). The disclosure is built from a disclosure-only type; the stored
   `Passage` and `Passages` are unchanged.
2. **2D client, `clients/2d/`.** `town.gd` learns neighbour tags from disclosed passages. The doorway
   label in `scene/places.gd` names the destination through the wording catalogue, not through the last
   tag.
3. **Wording.** A `place.<tag>` key per naming tag, in `clients/shared/settings/locale/en.po` and
   `zh_Hans.po` (see Q3).
4. **Specification of the disclosed record**, in this document (§4) and one line in
   `systems/movement/README.md`, which today says only that movement "discloses Passages".
5. **Validation** that market-town's and social-cafe's 300-day digests are unchanged (§7.3), that no
   golden frame or Python SDK model changes (§5), and that the 3D slice still reads its passages.

## 3. Non-goals

- **R-PK-2** (item kinds, `item:` name, `item-catalogue`): not here. It waits for 12d's single
  re-baseline (QS12-3).
- No change to stored state: no new component field, no schema bump of `passages`, no new genesis fact,
  no change to `systems/movement/src/system.rs`'s reduction of `passage-opened`, and no change to
  `commands.rs`'s "17 genesis facts" count.
- No new frame, no change to `server/PROTOCOL.md`'s frame list, no change to `server/tests/frames/`.
- No change to the kernel, `contracts/`, presence, or the observation's shape (only one component's
  payload grows).
- No change to any cognition crate. The paced controller and agenda read `passages` and are not edited
  (§6, overlap with PR 12n-2).
- No change to the 3D slice (`clients/3d-spike/`).
- No place names in the server or in any World Pack. Names are presentation wording keyed by tag.
- No rule about whether a passage may be used: the server's `MovementSystem::validate` is unchanged.
- No doorway *interaction* (no "enter" action). Walking through a doorway is unchanged.
- No change to façade bindings beyond what the existing `facade:<tag>` lookup already uses.

## 4. The disclosed JSON change (exact)

### 4.1 Today

A place `P` with a passage to `Q` discloses, to whoever perceives `P` (observer in `P`):

```json
{ "component_type": "passages", "schema_version": 1,
  "payload": { "leads_to": [
      { "to": { "entity": "<Q id>", "entity_type": "place" },
        "here": { "x": 0, "y": 0, "z": 0 } | null,
        "there": { "x": 0, "y": 0, "z": 0 } | null } ] } }
```

`leads_to` is sorted by `PlaceId` (`Passages::open`, `component.rs`). `to` is the `PlaceId` as it
serialises today (A-23 records `{entity, entity_type}`); this PR does not change it. `here`/`there` are
`null` when the world models no doorway position.

### 4.2 Proposed

Each entry gains exactly one field, `to_tags`:

```json
{ "to": { "entity": "<Q id>", "entity_type": "place" },
  "here": ..., "there": ...,
  "to_tags": ["cafe", "public"] }
```

- **Type.** `to_tags` is a JSON array of strings, each a `Tag` (`contracts/src/entity.rs`), in `Tag`
  order, which is the order the entity's `Tags` set (`contracts/src/entity.rs`, a `BTreeSet<Tag>`) already
  iterates. An empty destination set serialises as `[]`.
  Tags are never absent: the field is always present, so no consumer needs an absence rule.
- **Value.** The destination entity's tag set, read at disclosure from current state
  (`WorldRead::entity(Q)`), exactly as `perceived` reads a place's tags for the observer's own place.
  Nothing is copied into the stored `Passage`. **Tags only**: the field carries the destination's tag
  set and nothing else. It never carries the identity, contents, occupants or activity of the
  destination (ruling Q1, ARC-82; pinned by T2).
- **Ordering and the rest.** Entry order, `to`, `here` and `there` are unchanged.
- **Construction.** The disclosure is built from a disclosure-only type in `systems/movement`
  (working name `DisclosedPassage { to, here, there, to_tags }` with a `DisclosedPassages { leads_to }`
  wrapper), built from each stored `Passage` at `discloses` time. This is the precedent of
  `Disclosed` for `Walking` in the same `discloses` function (`system.rs`, the person branch). The stored
  `Passages` keeps its serde derive and is not given the new field, so persisted state
  (`systems/movement/tests/persisted.rs`) and genesis facts do not change.
- **Component identity.** `component_type` stays `passages`, `schema_version` stays `1` (Q2).
- **Destination not in the world.** A stored passage whose `to` does not resolve to an entity would be
  a broken world; `to_tags` is then `[]`. This is not expected to occur because `passage-opened` names
  places the world has (to be confirmed at implementation, §9 C1).

### 4.3 Consumers that must tolerate the field

| Consumer | How it reads `passages` today | Effect of `to_tags` |
| --- | --- | --- |
| `cognition/rule-controller/src/paced.rs` `doorways` (lines 382–388) | `serde_json::from_value` into the stored `Passages` | Ignored: `Passage` has no `deny_unknown_fields`. Decisions unchanged. |
| `cognition/rule-controller/src/agenda.rs` (module doc, line 11) | reads the doorway whose `to` is the agenda's place | Ignored. |
| `systems/movement/tests/disclosure.rs` `passages_on` | decodes into `Passages` | Ignored; a new assertion reads `to_tags` through the raw JSON (§7.1). |
| `clients/2d/scripts/town.gd` `_passages_of` | reads `to`, `here`, `there` | Ignored until §5.2 reads it. |
| `clients/3d-spike/scripts/slice/slice_link.gd` `_learn_passages` | reads `to` | Ignored. 3D unchanged. |
| `clients/protocol/demo/demo.gd` (line 382) | counts `leads_to` | Ignored. |
| `clients/2d/scripts/hud/readers.gd` `SHOWN_ELSEWHERE` | lists `passages` as shown elsewhere | Unchanged. |

## 5. Golden frames and the Python SDK

**Audit result: no golden frame carries `passages`, and the Python SDK does not model it.**

- `grep -rn passages server/tests/` finds nothing in `server/tests/frames/` (the frames that do carry
  components, `observation.json`, `delta.json`, `welcome.json`, have `entities: []` or no `passages`).
  The golden-frame test (`server/tests/frames.rs`) is therefore not affected.
- `sdk/python/src/mineworld_sdk/wire/contract.py` (`ComponentRecord`, line 186) types a component's
  `payload` as an uninterpreted `JsonValue`. No file under `sdk/python/` names `passages` or `leads_to`.
- `server/PROTOCOL.md` does not specify the `passages` payload (its only mention, line 693, is the
  movement prose). Nothing in the protocol spec changes.

So R-S11-9 ("an S11 PR that changes a file under `server/tests/frames/` updates `sdk/python`'s models")
is **not triggered**, and no SDK model changes. Validation proves this by grep (§7.4), not by assertion.

### 5.1 Optional golden frame (not proposed)

A golden frame for a place with a passage would pin the new shape across languages. It is not proposed
here: it would be the first golden frame with `passages` and would bring a Python model with it under
R-S11-9. If the operator wants one, it is a separate, explicit addition, with the SDK model in the same PR.

### 5.2 Client reads (in §6)

The 2D client's `town.gd` is the only consumer that will read `to_tags`.

## 6. Overlap with PR 12n-2 (`mvp0/pr-12n2-walk`, head 25660e8)

Audited with `git diff $(git merge-base origin/main origin/mvp0/pr-12n2-walk) origin/mvp0/pr-12n2-walk`
(merge base 737e032). The branch is behind `main` (its plain `main` diff also shows the lakeside
deletions; that is not its content).

- **Files 12n-2 changes that read `passages`:** `cognition/rule-controller/src/{agenda.rs, paced.rs,
  agenda_tests.rs, paced_tests.rs}`, plus `walking.rs` and `lib.rs`.
- **Files 12n-2 changes elsewhere:** `systems/bodies/src/route.rs` and its tests, `tools/cli/src/{hosted.rs,
  run.rs}`, `tools/cli/tests/{run.rs, run_restart.rs, walking_pace.rs, hosted_town.rs}`, `docs/DECISIONS.md`
  (+44), `docs/MODULE_SPEC.md`, `step-11-bodies.md`, `handoff.md`.
- **Files 12n-2 does NOT change:** anything under `systems/movement/`, `clients/2d/`, `clients/shared/`,
  `presentation/`. The overlap with 13c is **read-only**: both sides read the same `Passages` shape,
  and 12n-2 decodes it with the same serde path.

Consequences adopted here:

1. 13c edits no file under `cognition/`, so it cannot conflict with 12n-2's active controller work.
2. The decoder-tolerance evidence lives in `systems/movement/tests/`, not in `paced_tests.rs`, so 13c
   does not touch 12n-2's test files. The paced controller's own behaviour is proven by the digest check
   (§7.3), which runs the controller end to end.
3. A 13c entry in `docs/DECISIONS.md` will append to the same file 12n-2 extends. Expect a textual
   conflict on rebase; resolve by keeping both entries.
4. If 12n-2 merges first and changes the digests, the reference digest is re-recorded from `main` at 13c's
   merge time (I-6 wording), not from the 12n-2 branch.

## 7. Validation

### 7.1 Server (`systems/movement/tests/disclosure.rs`, new assertions)

Integration-level, through the real `MovementSystem` discloses path (no mock of the disclosure):

- **T1 — destination tags are disclosed to a person in the neighbouring place.** A person in the street
  perceives `passages` on the street; the entry whose `to` is the café carries `to_tags == ["cafe",
  "public"]`, and the entry for the apartments carries `["apartments", "residential"]`.
- **T2 — the audience rule does not widen.** The same observation lists the street and the people in it,
  and no other place as an entity; the café's `tags` do not appear as a perceived entity. Only its
  destination set is carried inside the street's passage.
- **T3 — the stored state is unchanged.** After genesis and a save/reload (`persisted.rs` path), the
  stored `Passages` JSON has no `to_tags` key, and the genesis fact count and content equal the base's.
- **T4 — consumers tolerate the field.** The disclosed payload decodes into the stored `Passages` with the
  same `to()` and `iter()` results as before the change (the same decode `paced.rs` uses).

Mutations, each run once and each must fail the named test:

| Mutation | Must fail |
| --- | --- |
| Read the observer's own place's tags instead of the destination's | T1 |
| Add `to_tags` to the stored `Passage` and reduce it into state | T3 |
| Mark `Passages` `deny_unknown_fields` | T4 |
| Omit `to_tags` when empty | T1 (shape) — to be added as an assertion of presence |
| Leave the disclosure unchanged | T1 |

### 7.2 2D client

- **C1 — the doorway label names the destination from the street.** A new field in the 2D harness's
  report (`drive.gd` `_report` / `capture.gd`) lists `doorway_labels: [{to, text}]` for the street. The
  café's label must read `door to café` (en) and its `zh_Hans` form, before the player has entered the
  café. Run through `mineworld-2d` against `mineworld server worlds/market-town --agent alice`, and
  asserted by `tools/cli/tests/client_2d.rs` (or the client's own check; the harness that exists is
  used, not a new one).
- **C2 — the façade is drawn before entering.** Stills taken from the street (`--capture`), inspected:
  the café's façade (`facade:cafe`) is drawn at its doorway, not a generic façade. Stills go in the PR
  body and the evidence file.
- **C3 — mutation.** Revert `_door_text` to the last-tag rule: C1 must fail with a label reading
  `door to public`. Remove `to_tags` from `_passages_of`: C1 and C2 must fail.
- **C4 — wording.** `clients/shared/checks/text_check.gd` passes: each `place.*` key exists in `en.po` and
  `zh_Hans.po`, and no key is missing from one of them. A removed `zh_Hans` key falls back to the readable
  form and the check reports it.
- **C5 — no client fact.** The 2D client does not compute a tag set from anything but the disclosure
  and the observer's own place (the grep in §7.4).

### 7.3 Digests (the I-6 check)

The 300-day seed-7 digests of `social-cafe` and `market-town` must be unchanged.

- The digest is the run's own output: `tools/cli/tests/run.rs` (lines 4–6) and `mineworld run worlds/<w>
  --headless --seed 7 --days 300 --save DIR`. Its comparisons are the per-day lines, the `faults` line, the
  fact counts, and the persisted tables read through `Tables::read` (AC-12).
- **Procedure.** (a) On the base commit (`origin/main` at implementation time), run both worlds, record the
  day lines, fact counts and a SHA-256 over the persisted tables' rows in the evidence file as the
  reference. (b) On the head, repeat exactly. (c) The two must be identical. The row digest is taken
  with the existing `Tables` reader, so no new reader is added; if it cannot be made byte-stable, the
  check is `INCONCLUSIVE` and the row-level comparison (`social::facts_of`) is used instead.
- The existing `run.rs` test must pass unchanged.
- Why this proves the claim: the paced controller reads `passages` only through the decode in §4.3, which
  ignores `to_tags`, so its decisions, and therefore the facts it causes, are unchanged.

### 7.4 Repository-wide checks

- `grep -rn "to_tags" server/tests sdk/python` returns nothing (no golden frame or SDK change).
- `grep -rn passages server/tests/frames` returns nothing before and after.
- `grep -rn "\.tags\|\"tags\"" clients/2d/scripts` shows tag reads only in `town.gd` (the observer's place and
  the disclosed `to_tags`) and `places.gd` (the façade and the label).
- `systems/movement/src/system.rs`'s `passage-opened` reduction and `persisted.rs` are unchanged in the diff.
- `ac1_composability` 13/13 (check 3 still holds: `movement` owns `Passages`; no section is added to a file).

### 7.5 Standard commands (`.structured-coding/standards.md`)

`cargo fmt --all --check`, `cargo check --workspace`, `cargo clippy --workspace -D warnings`,
`cargo test -p mineworld-movement` and `cargo test --workspace` (the run tests), plus the CI layers
`python3 scripts/ci_layer.py --list fast` and `test`. The `python` layer runs unchanged (no Python file
changes). Platform: macOS, Linux and Windows (`all-platforms` policy); the 2D client checks run on the
platform the operator has, and the other two are INCONCLUSIVE until CI runs them.

## 8. Commit plan

Three commits, each with implementation, deterministic validation and LLM logic review as separate items.
`[x]` requires the work and the evidence; `N/A` needs the audited reason.

### C1 — `feat(movement): doorways disclose the tags of the place they lead to (R-PK-1)`

- [ ] **Implementation.** `systems/movement/src/` disclosure-only `DisclosedPassage(s)` with `to_tags`
      built in `discloses`; `Passage` and `Passages` unchanged; `systems/movement/README.md` line 11 states
      the disclosed shape.
  - Evidence: diff of `system.rs`, `component.rs`; `git diff --stat` shows no change to `persisted.rs`'s
      fixtures.
- [ ] **Validation.** T1–T4 added and passing; mutation table §7.1 run, each row FAIL-then-PASS; §7.3 digest
      check PASS for both worlds; §7.4 greps; §7.5 commands.
  - Evidence: command output and the digest reference in the PR evidence file.
- [ ] **LLM review.** Checked against: single ownership (movement owns the disclosure; reads only another
      place's tags); no kernel or contract change; the audience rule (T2); strong typing (`Tag`, `PlaceId`, no
      `Value` built by hand); conceptual locality; the consumers in §4.3.
  - Evidence: review notes in the PR body.

### C2 — `feat(2d): the doorway names its destination from the street (R-PK-1)`

- [ ] **Implementation.** `clients/2d/scripts/town.gd` reads `to_tags` and records neighbour tags with a
      version bump; `scripts/scene/places.gd` `_door_text` names the destination by the first tag in `Tag`
      order that has a `place.<tag>` wording, then `display_name`, then `ui.outside`; `drive.gd`/`capture.gd`
      report `doorway_labels`; `clients/shared/settings/locale/{en,zh_Hans}.po` gain the `place.*` keys
      (six: `cafe`, `park`, `store`, `apartments`, `workplace`, `street`, see Q5).
  - Evidence: diff; the keys in both catalogues.
- [ ] **Validation.** C1–C5 (§7.2) run with the real server; stills captured and inspected; text_check PASS;
      `tools/cli/tests/client_2d.rs` PASS; mutation C3 FAIL-then-PASS.
  - Evidence: report JSON, stills, check output.
- [ ] **LLM review.** Checked against: the client holds no world truth (the label is presentation text
      from disclosed tags); no rule evaluated in the client (I-2); wording through the catalogue (no literal
      English in `.gd`); the 3D client unchanged; `ui.door-to` stays where it is.
  - Evidence: review notes in the PR body.

### C3 — `docs(movement,plan): record R-PK-1 and the perception decision`

- [ ] **Implementation.** `docs/DECISIONS.md` **ARC-82** (the door-sign disclosure: tags only, the audience
      rule, the decision and its alternative); `docs/MODULE_SPEC.md` short section under movement's disclosure
      stating the `passages` shape of §4.2 (Q6); step-13 §6.3 R-PK-1 status and the 13c row; `systems/movement/
      README.md` if not done in C1.
  - Evidence: diff.
- [ ] **Validation.** `doc-headings` and `decision-ids` CI checks PASS (the fast layer); links resolve.
  - Evidence: `python3 scripts/ci_layer.py --list fast` output and the run result.
- [ ] **LLM review.** Checked against: the record says what changed in the disclosed contract and why it does
      not widen the audience of any entity; spec text is complete (not a gesture).
  - Evidence: review notes in the PR body.

## 9. Acceptance criteria

- **AC-1.** From the street of market-town, the café's doorway is labelled `door to café` (en) and its
  zh_Hans form, before the player enters; its façade is drawn from the disclosed tags (C1, C2).
- **AC-2.** Each other doorway of the street is labelled with its destination's wording, not with
  `outside` and not with a tag such as `public` (C1).
- **AC-3.** The disclosed `passages` payload has exactly the §4.2 shape: the §4.1 fields unchanged, plus
  `to_tags` in every entry, always present (T1, T4).
- **AC-4.** No observation lists a place as an entity beyond the observer's place (T2).
- **AC-5.** The stored `Passages`, the genesis facts and the persisted tables are unchanged (T3, §7.4).
- **AC-6.** The 300-day digests of `social-cafe` and `market-town` are byte-identical to the base (§7.3).
- **AC-7.** No golden frame and no SDK model changes; R-S11-9 is not triggered (§5, §7.4).
- **AC-8.** The 3D slice's evidence is unchanged and its passage reader is untouched (§4.3).
- **AC-9.** `ac1_composability` 13/13; fmt, check, clippy `-D warnings` and tests PASS on macOS and, through CI,
      on Linux and Windows.
- **AC-10.** `text_check` PASS: every `place.*` key in both locales.

## 10. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **RK-1** | **Perception widening.** A person in one place learns the tags of places its doorways lead to. A-20 states "other places are not perceived". This is the first disclosure of another place's state to a person. | Q1: operator decision. The tags are authored world data already shown to anyone who enters; T2 pins that nothing else widens. |
| **RK-2** | The paced controller's decisions change because the decode now sees an extra field. | The decode ignores unknown fields (T4); §7.3 proves the digests are unchanged end to end. |
| **RK-3** | 12n-2 merges first and changes the digests. | Reference re-recorded from `main` at merge time (§6.4). |
| **RK-4** | Tag-based naming is only as good as the tag set: `public` and `outdoor` name nothing, and `office` is not keyed. | The rule is "first tag in order with a wording"; the six keys are listed (Q5); an unnamed place falls back to `outside`. |
| **RK-5** | Some tags have no façade binding (`facade:store`, `facade:park`, `facade:workplace` are not in `asset_bindings.yaml`). | The generic façade (`facade:unknown:N`) already covers unbound tags; C2 shows it; not a 13c defect. |
| **RK-6** | The step-13 text calls R-PK-1 "F-O5's precedent". F-O5 joins a place's own record; R-PK-1 reads another entity. | Reads are read-only through `WorldRead`; no ownership changes. Recorded here so the review does not rely on the precedent alone. |
| **RK-7** | The current 2D label is already wrong (§1); changing it is visible behaviour. | It is the defect this PR fixes; C2 stills show before and after. |

## 11. Questions for the operator

- **Q1 — RULED: accepted (operator).** Recorded in §11.1. Original question: accept that a doorway discloses its destination's *tags*
  to a person in the neighbouring place (A-20's "other places are not perceived" is widened by one
  field)? **Recommendation: accept**, with the DECISIONS record in C3 and T2 as the pin. The alternative
  is the fallback in step-13 QS12-7: learn tags by entering, which keeps the street's doorways generic.
- **Q2 (schema).** Keep `passages` at `schema_version 1` (additive; `Walking`'s `Disclosed` precedent), or
  bump the disclosed record to 2? **Recommendation: 1.** Every reader in §4.3 ignores unknown keys.
- **Q3 (where the names live).** The user brief asks for "the shared catalogs"; `ui.door-to` lives in the
  Presentation Pack's `i18n/`. **Recommendation:** `place.*` keys in `clients/shared/settings/locale/` (they
  are world-semantic names any client may use, and `clients/2d/PRESENTATION.md` §7 names that layer as
  holding the keys both clients use); `ui.door-to` stays in the pack, as today (`zh_Hans`: `通往{place}的门`,
  so a Chinese place name drops into it). Confirm.
- **Q4 (empty set).** Always present (`[]` when empty) rather than omitted? **Recommendation: always present.**
- **Q5 (keys).** The six naming tags `cafe`, `park`, `store`, `apartments`, `workplace`, `street`. Confirm, or
  name others (e.g. `office`).
- **Q6 (spec home).** Is the README line plus this document enough for §2.1, or does the disclosed shape
  need a section in `docs/` (for example `MODULE_SPEC.md` for movement)? The present state has no such
  section for `passages`.
- **Q7 (golden frame).** Add one golden frame with `passages` now (§5.1), which would bring a Python
  model under R-S11-9? **Recommendation: no** in this PR.

### 11.1 Rulings, 2026-10-10 (frozen)

- **Q1 — accepted (operator).** A doorway's destination tags are disclosed as a **door sign**: tags only,
  never who or what is inside. Recorded as **ARC-82** in `docs/DECISIONS.md` during C3; test T2 pins the
  audience.
- **Q2 — `schema_version` stays 1.** No bump.
- **Q3 — `place.*` keys go in the shared catalog** (`clients/shared/settings/locale/{en,zh_Hans}.po`).
  `ui.door-to` stays in the Presentation Pack's `i18n/`, as `clients/2d/PRESENTATION.md` §7 states for
  pack wording.
- **Q4 — `to_tags` is always present**, `[]` when the destination has no tags.
- **Q5 — the six naming tags are confirmed:** `cafe`, `park`, `store`, `apartments`, `workplace`, `street`.
- **Q6 — no separate spec document.** A short section is added to `docs/MODULE_SPEC.md` under movement's
  disclosure, in C3, together with the README line in C1.
- **Q7 — no golden frame in this PR.** §5.1 stays unproposed; R-S11-9 is not triggered.

## 12. Evidence, status and what is not claimed

- Nothing in this document has been run. Every "PASS" in §§7–9 is a plan, not a result. Results are
  classified `PASS`, `FAIL` or `INCONCLUSIVE` in the PR's evidence file at implementation time.
- The current-label defect (§1) is read from `town.gd`, `places.gd` and the sorted tag set
  (`contracts/src/entity.rs`, `Tags`, a `BTreeSet<Tag>`); it has not been reproduced in a running client. C2 reproduces
  or refutes it before the change.
- No file outside `.structured-coding/plans/mvp0/pr-13c-doorway-names.md` (and the step-13 link in §6.3's
  13c row) is changed by this design.

## 13. Execution contract (frozen 2026-10-10)

```text
Implementation branch:   mvp0/pr-13c-names
Implementation worktree: /Users/yuema137/mineworld-worktrees/impl-13c-names (one working tree, one session;
                         the execution session confirms no other session holds it before editing)
Implementation base:     origin/main at the start of implementation (exact commit recorded in C1's evidence)
Commits:                 C1 (§8 C1), C2 (§8 C2), C3 (§8 C3), each with implementation, validation and LLM
                         review items tracked separately
Design authority:        this document, PR #156 (docs/13c-names-design)
Fresh session:           implementation runs in a fresh session; a resumed session re-reads this section and
                         §§4, 7, 8 before continuing
```

**Allowed to change:**

- `systems/movement/src/**`, `systems/movement/tests/**`, `systems/movement/README.md` (C1);
- `clients/2d/scripts/town.gd`, `clients/2d/scripts/scene/places.gd`, `clients/2d/scripts/harness/drive.gd`,
  `clients/2d/scripts/harness/capture.gd`, `clients/shared/settings/locale/{en,zh_Hans}.po` (and
  `messages.pot` if the shared tooling requires it) (C2);
- `tools/cli/tests/client_2d.rs` (C2, the report assertion only);
- `docs/DECISIONS.md` (ARC-82 only), `docs/MODULE_SPEC.md` (movement disclosure section only),
  `.structured-coding/plans/mvp0/step-13-client-2d.md` (§6.3 R-PK-1 status and the 13c row) (C3).

**Forbidden to change:** `contracts/**`, `kernel/**`; any `cognition/**` file (PR 12n-2 is active there);
`systems/movement/src/system.rs`'s `passage-opened` reduction and the stored `Passage`/`Passages` types;
`server/tests/frames/**`, `server/PROTOCOL.md`, `sdk/python/**`; `clients/3d-spike/**`; `systems/item/**` and
`worlds/**/items/**` (R-PK-2); `tools/cli/tests/run.rs` and `paced_tests.rs` (12n-2's files); any World Pack.

**Stop and return to the operator** (`CLAUDE.md` §3.1) if: a golden frame or SDK model must change; the
digest check does not hold; a file in the forbidden list must change; the disclosure needs any field beyond
`to_tags`; a test shows the audience widened beyond T2; or the 2D label needs a rule evaluated in the client.

**Validation required before any commit is marked `[x]`:** §7.1–§7.5 as written, with each result classified
`PASS`, `FAIL` or `INCONCLUSIVE` in the evidence file. A platform not run locally is `INCONCLUSIVE` until CI.

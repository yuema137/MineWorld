# PR S28-b — Retention on seat views: forgotten-range attribution, summaries that never contradict detail (C-4, FU-S28-1), the bound at the bound, and Gate 2 in a shared home

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        draft 1 (2026-10-15), FROZEN 2026-10-18
Author:                 S28-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s28b, branch docs/pr-design-s28b,
                        planning base origin/main @ 5cbb4697 (#231); S28-a read on mvp0/pr-s28a @ 1cc4ce33
                        (C0 … C2 committed, C3 in progress)
Approved by / evidence: §14 ruled 2026-10-18 by the operator in the primary session: QS28b-1 and
                        QS28b-2 as recommended
Implementation base:    main (origin/main at C0, after S28-a has merged); C0 records the commit
Start condition:        S28-a MERGED (step-28 §11.1) — met: #242, merge 9884c64c. This PR is NOT stacked
                        on S28-a (§2.3)
Execution contract:     §12 (filled; endpoint authority included)
Implementer:            Opus-class (step-28 §11.1)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Step:** [`step-28-shared-memory.md`](step-28-shared-memory.md) (STEP DESIGN FROZEN
2026-10-14; rulings §14.1) §§3 (I-S28-1 … I-S28-10), 5.6 (forgotten-set attribution), 7, 9.2 (C-4), 11 and
11.1 (the S28-b row, FU-S28-1, the integration checkpoint), 15 (AS28-5, AS28-6, AS28-7, AS28-9, AS28-13,
AS28-18 for C-4) · **Parent:** [`overall.md`](overall.md) · **Previous PR:**
[`pr-s28a-memory-home.md`](pr-s28a-memory-home.md) (FROZEN 2026-10-15; its §10 R-S28b-a-1 is this PR's scope)
· **Revises the code of:** [`pr-fg-b-retention-host.md`](pr-fg-b-retention-host.md) (the forgotten set, §5.8),
[`pr-fg-b2-size-trigger.md`](pr-fg-b2-size-trigger.md) (the margin, §4.1),
[`pr-fg-c-lm-summarise.md`](pr-fg-c-lm-summarise.md) (the host's acceptance of prose, §5.2, §5.5) · **Followed
by:** S28-c (scopes), S28-e (inspect reads this PR's attribution) · **Binding parents:** `CLAUDE.md` §§2, 3, 4
(items 1, 2, 3, 7, 8, 9, 10, 11, 12, 13); `docs/CORE_CONCEPTS.md` §§2 (INV-4, INV-13), 5.3;
`docs/ARCHITECTURE.md` §9.3; `docs/MODULE_SPEC.md` §5.2; `docs/DECISIONS.md` `ARC-25`, `ARC-27`, `ARC-59`,
`ARC-87`, `ARC-98` … `ARC-101`, `ARC-109` (on S28-a's branch), `DEP-37`, `DEP-58`.

**Decision ids.** None new. This PR writes dated notes under `ARC-87` (attribution; the prose name check) and
`ARC-109` (schema 5) (§9 C1).

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-28 §11, the S28-b row).** "The retention host on seat views: items, execution with GC, FG-b2's
  size trigger on the seat view, FG-c prose in the home, forgotten-range attribution, C-4 (a)–(c). **Plus the
  FG-c follow-up FU-S28-1** (C-4 (d), below) in FG-c's merged code."
- **What S28-a moved out of that row (QS28a-2, ruled 2026-10-15 by the primary).** S28-a ports every retention
  SQL site mechanically: items read seat-view bytes, execution releases through `SeatView.release` with GC,
  the meter and the size trigger measure the seat view, prose is written in the home. **S28-b keeps**
  (`pr-s28a-memory-home.md` §10, R-S28b-a-1, verbatim): "forgotten-range attribution (step §5.6), C-4
  (a)–(c), FU-S28-1, FG-b's real-binary Gate 2 on seat views, AS28-9 under `keep-all` at the bound, and the
  check that FG-b2's `trigger` margin holds in positional bytes for the largest single fact (F-S28a-8). It
  inherits the ported host; it does not port again."
- **FU-S28-1 (step-28 §11.1, verbatim).** "S28-b adds the name check: prose for a group may name only known
  names of that group's counterparts, and the group's place; any other known person or place name makes the
  answer invalid for that group, so the group keeps the fallback's decisions; the host records the reason as
  `prose-contradiction`. Recorded answers that break the rule now take the fallback on replay. S28-b states
  which cassettes change and re-records none silently. FG-c's design document gets a pointer to this
  follow-up, applied by S28-b's C1."
- **C-4 (step-28 §9.2, verbatim).** "For every episode and chapter: (a) its structural fields (day, place,
  counterparts, record count) equal a recomputation from its cited facts when it was written; (b) no retained
  L0 record lies inside a merged container's ranges (FGb-1); (c) a retained record and the unit covering it
  agree on day and place. For FG-c prose (d): every person name in the prose is a known name of a counterpart
  of the group, and every place name is the group's place. Otherwise the host takes the fallback for that
  group and records `prose-contradiction`."
- **Forgotten-set attribution (step-28 §5.6, verbatim).** "Each forgotten range carries `(first_day,
  first_policy, last_day, last_policy, passes)`, updated when ranges merge. Full per-id attribution would grow
  without bound, so the report says 'added by `consolidate` on day 12, extended by 3 passes, last on day 40'."
- **AS28-9 (step-28 §15, as amended by QS28a-1).** I-a-6 (a) and (b) of `pr-s28a-memory-home.md` §6 replace
  I-S28-6: (a) a shared home's dump ≤ Σ of its seats' one-seat homes' dumps; (b) each one-seat home's dump ≤
  its seat view's dump + `K` × its held records. "With `keep-all` everywhere, the bound holds and physical ≤ N
  × bound." S28-b owns the `keep-all`-at-the-bound half.
- **Integration checkpoint (step-28 §11.1).** "Every PR ends with the transparency harness green (12 seats,
  `none` vs `home`). From S28-b on, it also runs FG-b's real-binary Gate 2 scenario on seat views."
- **Rulings not re-asked.** QS28-4 (size = seat view, shared records in full), QS28-8 (refuse an old schema
  and rebuild; no importer), QS28a-1 (I-a-6 (a)/(b), `K`), QS28a-2 (the split above), QS28a-3
  (`fact_who.role`).
- **R-LOAD-1 (coordinator rule, in force).** §11.

---

## 1. Goal

Make forgetting on shared seat views **explainable and provably coherent**, and prove the bound holds when it
bites. Concretely:

1. Every forgotten range says who forgot it and when: the first and last pass day and the policy (or the
   host's backstop step) that added to it, and how many passes did. This is what S28-e's `inspect … forgotten`
   reads.
2. A model's prose can no longer name a person or a place that its group did not involve. Such an answer is
   refused by the host, the group keeps the fallback's decisions, and the pass log says
   `prose-contradiction` (FU-S28-1, C-4 (d)).
3. Summaries never contradict the detail that survives them (C-4 (a)–(c)), checked by an oracle written from
   the perceived facts, under every bundled policy, in a shared home.
4. Under `keep-all` everywhere, at the 20 MiB bound, with records shared and the backstop forced, every seat
   view stays within its bound, the home stays within I-a-6, and a home-mate's forced forgetting never moves
   a byte of another seat (AS28-9, AS28-5 at the bound).
5. FG-b's real-binary scenario runs in a **shared** home of real people (Alice and Bob, 100 days), with
   coverage, the bound, transparency and C-4 at every pass (the step's Gate 2 on seat views).

No policy, item rule, bound number or pass order changes. Nothing changes in the kernel, contracts, server
or protocol (I-S28-10).

---

## 2. Scope

### 2.1 In scope

1. **Schema 5** (`memory/home/schema.py`, S28-a's module): `forgotten` gains `first_day`, `first_policy`,
   `last_day`, `last_policy`, `passes`; `retention_passes` gains `prose_contradictions`; `SCHEMA_VERSION = 5`;
   a schema-4 home is refused by `UnsupportedStoreSchema(found=4, wanted=5)` with the rebuild instruction
   (QS28-8's rule; F-S28b-1). The view table's columns and the encoder's codecs follow.
2. **Attribution** (`memory/retention/{forgotten.py, plan.py, bound.py, host.py}`): every range the plan or
   the backstop adds carries its attribution; `normalize` merges attributions by the rule of §5.2;
   `ForgottenSet` exposes attributed ranges (§5.3).
3. **FU-S28-1** (`memory/retention/prose.py` new, `memory/retention/host.py`): the pure name check and the
   host's refusal, the outcome `fallback:prose-contradiction`, the count in the pass log (§5.4, §5.5).
4. **Tests**: attribution (unit and pass level), the prose check (unit and host level, scripted answers), the
   C-4 oracle and its tests, the bound at the bound (`keep-all`, heavy multi-seat stream), the margin in
   positional bytes, the shared-home Gate 2 (§7).
5. **Specifications**: `docs/MODULE_SPEC.md` §5.2 rules 6 and 9, the `lm-summarise` row and the "Model calls"
   paragraph; dated notes in `docs/DECISIONS.md` under `ARC-87` and `ARC-109`; the pointer in
   `pr-fg-c-lm-summarise.md` (step-28 §13) (§9 C1).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `mineworld-memory inspect` (`forgotten`, `access --event`, `bytes`), the `forgotten` view's rendering of attribution, AS28-16 | S28-e (reads §5.3's API) |
| `MemoryScope`, `ScopeRegistry`, the scope-private backstop step "drop scope-private rows, oldest `last_at` first" (step §7.2) | S28-c / S28-d: no scope-private rows exist before S28-d |
| `BoundBelowDefault(seat, given_mib)` (step §5.5; AS28-16 (c)) | S28-c (selection and named errors). FG-b2's `StoreBoundOutOfRange` stands until then |
| 12 × 300 and 100 × 300 days, AS28-17 | S28-g (nightly) |
| P6b wiring of `sharing`, AS28-10 under `home` | S28-f |
| A change to any policy, item rule, bound number, ratio or the backstop's order | never in this PR (material stop, §12) |
| Re-porting any SQL S28-a ported | never (R-S28b-a-1: "it does not port again") |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | never (I-S28-10) |

### 2.3 Dependencies and the start condition

- **Required: S28-a merged** (step-28 §11.1). Its home, seat views, ported host, positional encoding, the
  transparency harness, `home_streams.py`, `retention_oracle.py` (as ported), and the C1 golden renders are
  this PR's base.
- **Not stacked.** This PR branches from `origin/main` after S28-a's merge and is not opened on
  `mvp0/pr-s28a`. Stacking is declined because four inputs of this design exist only once S28-a is final:
  the ruling on F-S28a-9 (I-a-6 (b)'s exact form), the pinned `K`, the lengthened streams of S28-a's C5
  (`test_retention_size.py`), and the positional port of `retention_oracle.py`. C0 re-reads each on the
  merged code.
- **Already merged:** FG-c (#201), FG-b2 (#213), P6b (#207; S28-a adapted `controller/`, its D-a-1).

---

## 3. Audit

Line numbers are on `origin/main @ 5cbb4697` (schema 3, `MemoryStore`), unless marked **[S28-a]** (the branch
`mvp0/pr-s28a @ 1cc4ce33`). S28-a changes the **storage API** of every retention file but, by its frozen
scope, none of their **semantics**; C0 re-anchors each line on the merged code.

### 3.1 Source anchors (under `cognition/lm-controller/src/mineworld_cognition/`)

| # | Fact | Where |
| --- | --- | --- |
| B-1 | The forgotten set's table has only `(first_key, last_key)` per range (schema 3); S28-a adds the leading `seat` and nothing else. The view table dumps `first_key, last_key`. | `memory/store.py:62`; **[S28-a]** `memory/home/schema.py:65`, `:188` |
| B-2 | Ranges are added in exactly one place: `plan.remove(…, forget=True)` inserts the dropped item's citation ranges (levels 1 and 2 only; an L3 drop adds nothing). | `memory/retention/plan.py:121-150` (insert `:140-147`) |
| B-3 | `remove(forget=True)` is reached from three callers: the plan's `Drop`s (`execute`, `:108-109`); backstop step 1, `consolidated()` (which runs `execute` over `consolidate`'s defaults, `bound.py:110-118`); and backstop steps 4 and 5 through `_shed` (`_chapters`, `_chapterless_episodes` return `forget=True`, `bound.py:184-194`). Steps 2, 3 merge (no range); step 6 drops L3 rows (no range). | `memory/retention/bound.py:110-170`, `:184-194` |
| B-4 | `normalize(db)` rewrites the whole table when any two ranges with no surviving unit between them merge, then caps at `MAX_RANGES = 256` by raising `forgotten_below`. It is called by `run_pass` after the plan, by `enforce` after step 1, and by `_shed` before every size confirmation, so **several times in one pass**. | `memory/retention/forgotten.py:54-85`; `host.py:108`; `bound.py:129`, `:162`, `:169` |
| B-5 | `ForgottenSet` (`load`, `ranges`, `watermark`, `contains`) is read by no production code today; it is the API tools are meant to use (FG-b §5.8). | `memory/retention/forgotten.py:24-51` |
| B-6 | A pass has `day = now // DAY_S`, its policy id (`retention.policy.id`) and a cause (`day` / `size`); a day may hold several passes (FG-b2), each keyed by its triggering fact. | `memory/retention/host.py:94-130`, `:345-373` |
| B-7 | `valid_prose` checks only one line, ≤ 160 B, no run of three digits, no `#`. | `compress/model_summarizer.py:68-74` |
| B-8 | `lm_summarise._answer` turns a valid completion into a `GroupAnswer` whose `prose` passed `valid_prose`; the policy has no name book (FGb-3: frozen items only). | `forgetting/lm_summarise.py:195-215` |
| B-9 | The host's `_accepted` discards an answer that names items outside its group, repeats a container, or carries prose that `valid_prose` would change; the discard count joins `invalid`, and the outcome precedence is: a `StopReason` → `fallback:<reason>`; else any invalid → `fallback:invalid`; else `assisted`. | `memory/retention/host.py:262-289`, `:219-227` |
| B-10 | Prose is written only for a surviving answered container of level 1 or 2. | `memory/retention/host.py:228-236`, `:326-342` |
| B-11 | Items: an episode item carries its `episode_who` counterparts and its `place`; a record item its `l0_who` counterparts (participants minus me) and its place; a **chapter item carries no counterparts and no place**. | `memory/retention/items.py:133-170`, `:172-212`, `:214-241` |
| B-12 | `NameBook` maps `EntityId → name` for persons and places alike (the `names` table has no kind), learned from `named` facts and from P6's observations; `one_line`, ≤ 60 characters; generic labels `someone`, `somewhere`, `others` are not names. | `memory/names.py:18-80` |
| B-13 | The seat's own entity is in its name book when a `named` fact names it (the synthetic stream names Alice first), and is never a counterpart. | `tests/retention_stream.py:34`, `:72-74`; `memory/render/__init__.py:69-70` |
| B-14 | Episode structure (D-P4-7): a record joins the open episode when within 1 800 s, on its day, and (unless ambient or placeless) at its place; an episode opened by a placeless record **inherits the previous episode's place**; counterparts are the ordered union of the records'; `records` counts them; `importance`, `mine` are written at close. | `compress/episodes.py:59-118` (inheritance `:64`) |
| B-15 | A chapter stores `week`, first/last episode, first/last key, `first_at` (= first episode's `opened_at`), `last_at` (= last episode's `closed_at`), `importance`, `mine`, and its ranges (runs of consecutive episode numbers). | `compress/chapters.py:26-76` |
| B-16 | `embellish` (P4) writes prose through `valid_prose` too; no production caller exists (`controller/` does not import it). | `compress/model_summarizer.py:77-126`; `git grep embellish` |
| B-17 | The committed cassette's scripted summaries are ten fixed sentences with no person name; the synthetic stream's six places are never named. So no recorded answer names a known name. | `tests/test_lm_summarise.py:47-59`; `tests/retention_stream.py:33-34`; `tests/cassettes/lm-summarise.jsonl` (180 lines) |
| B-18 | FG-c's replay test pins the sha256 of the replayed dump (`DIGEST`); S28-a re-pins it for schema 4 (its AC-a-13). | `tests/test_lm_summarise_replay.py:39`, `:99-103` |
| B-19 | `History` exports Alice **and Bob** from the shared 100-day social-cafe save; `test_retention_real.py` uses Alice only. | `tests/ac10_harness.py:66-96`; `tests/test_retention_real.py` |
| B-20 | FG-b2's heavy stream is single-seat (Alice among 60 of a crowd of 400 per fact, ≈ 5 KB per fact in object bytes). | `tests/retention_stream.py:117-145` |
| B-21 | FG-b2's margin argument (`bound − trigger` = 1.25 MiB holds "a fact naming ≈ 3 000 participants") was made in object bytes. | `pr-fg-b2-size-trigger.md` §4.1 |
| B-22 | `MODULE_SPEC.md` §5.2: rule 6 (forgotten set) states no attribution; rule 9 (pass log) lists the logged columns; the `lm-summarise` row and the "Model calls" paragraph state no name rule. | `docs/MODULE_SPEC.md:1304-1308`, `:1332-1334`, `:1360`, `:1362-1372` |
| B-23 | The pass log's `outcome` is a free string; no consumer under `controller/` parses it. | `git grep "fallback:" cognition/lm-controller/src/mineworld_cognition/controller` (no match) |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S28b-1** | Attribution adds columns to `forgotten` (B-1). S28-a ships schema 4, and P6b opens real homes (S28-a D-a-1), so a schema-4 home will exist on a developer's disk between the two merges. Changing the table under the same number would turn an S28-a-era file into an SQL error rather than a named refusal. | **Bounded: schema 5.** QS28-8 already rules the migration ("refuse and rebuild, no importer"); only the number is new, as with F-S28a-1. A schema-4 home is refused by `UnsupportedStoreSchema(found=4, wanted=5)`. The `prose_contradictions` column joins the same bump (F-S28b-6), so the PR changes the schema once. Dated note under `ARC-109` (C1). |
| **F-S28b-2** | `passes` cannot be an exact count of distinct passes with bounded storage. Two ranges added by one pass P1 and kept apart by a surviving unit, later joined because pass P2 forgets that unit, merge into one range touched by two passes; summing their counts gives three. Exactness needs a set of pass ids per range, which grows without bound (step §5.6's own reason for not storing per-id attribution). | **Bounded definition (§5.2):** `passes` is the sum, over the ranges that existed when the current pass began and are now merged into this one, of their `passes`, plus 1 if the current pass added to it. It equals the number of distinct passes whenever no single pass added two ranges that later merged, and is otherwise an upper bound. The docstring and `MODULE_SPEC` rule 6 say so. |
| **F-S28b-3** | `normalize` runs several times per pass (B-4). A rule that adds "1 for this pass" at each call would count one pass several times. | The host captures, once at the start of the pass, the set of `first_key`s of the seat's ranges (the "old" ranges), and passes it to every `normalize` of that pass (§5.2). One pass adds at most 1 to any merged range, however often `normalize` runs. |
| **F-S28b-4** | The step says "policy". A range dropped by the backstop is not the policy's decision; FG-b counts those removals as `forced`, and the step's `inspect … forgotten` asks "was it forced?". | **Bounded:** `first_policy` / `last_policy` hold the pass's policy id for a range the plan dropped, and `host:<step>` for a range the backstop dropped: `host:consolidate` (step 1), `host:chapters` (step 4), `host:chapterless-episodes` (step 5) (B-3). Steps 2, 3 and 6 add no range. A refused plan's fallback (`consolidate`'s defaults executing for a refused plain plan) is attributed to the pass's policy id, as the pass log already records it (`policy` = the configured id, `outcome` = `fallback:<reason>`). |
| **F-S28b-5** | Chapter items carry no counterparts and no place (B-11). A chapter whose episodes were merged into it in an earlier pass has no nested item at the pass. Read literally, "the group's counterparts and place" is then empty, so any prose naming anyone would be a contradiction, and `lm-summarise` could never rescue an old named chapter (its `Drop` group's fallback is the drop). Separately, a quoted name in a gist (`Carol said: "Bob is late"`) is in the group's own text but is not a counterpart, and P4's instructions ask the model to "keep every name and quote" (`compress/model_summarizer.py:29-32`). | **§14 QS28b-1** (the allowed set). The design is written with the recommended answer and marks the one line that changes otherwise (§5.4). |
| **F-S28b-6** | "The host records the reason as `prose-contradiction`" (FU-S28-1). The pass log keeps one outcome per pass, which is the first reason (FGc-5); in a pass that also stopped (`exhausted`, `budget`) or also discarded a malformed answer, a contradiction would be invisible. | **Bounded:** the outcome is `fallback:prose-contradiction` when no `StopReason` and no other invalidity occurred (§5.5's precedence), and the pass log gains `prose_contradictions INTEGER NOT NULL` (the count), so the reason is never hidden. In schema 5 with F-S28b-1. |
| **F-S28b-7** | The seat's own name is in its name book (B-13) and is never a counterpart, so a first-person prose that names the seat ("Alice Moreau met Bob") would be read as naming a stranger. | **Bounded:** the seat's own entity is always in the allowed set. |
| **F-S28b-8** | C-4 (b) as worded ("no retained L0 record lies inside a merged container's ranges") would forbid partial merges, which `importance-recency-cap` makes by design (unchosen nested records merge, chosen ones stay inside their episode; MODULE_SPEC §5.2). FGb-1, which (b) cites, requires the opposite: nested units lie inside their container. | **Bounded reading, faithful to FGb-1:** (b) = every retained L0 record lies in a **surviving episode** (open or closed), never in a range that only a chapter's ranges or the forgotten set represent; that is, a record never outlives its episode (AFGb-3 (b), stated at the record level). Recorded for the primary as a wording correction to step-28 §9.2 (§10). |
| **F-S28b-9** | C-4 (c)'s "agree on place" is false by design for an `ambient` or placeless record, which joins an episode without testing its place (B-14). | **Bounded reading:** (c) checks day for every retained record, and place for every retained `notable` record that has a place. Same §10 correction. |
| **F-S28b-10** | C-4 (a) for a chapter: chapters store no place, no counterparts and no record count (B-15). | **Bounded reading:** for a chapter, (a) checks `week`, `first_key`/`last_key`, `first_at`/`last_at` and that its ranges cite only perceived ids of that week; for an episode, `day`, `place`, `counterparts` (ordered) and `records`. The inherited place of B-14 is checked against an episode sequence the oracle derives itself from the facts (§6.3). |
| **F-S28b-11** | `embellish` (B-16) writes model prose too, without the name rule, but FU-S28-1 names only FG-c's code, and `embellish` has no production caller. | **§14 QS28b-2.** |
| **F-S28b-12** | The step's Gate 2 is "FG-b's real-binary scenario on seat views". After S28-a, `test_retention_real.py` already runs Alice's 100 days on a one-seat home (the `none` arm). What is new is a home **shared** by real people. Twelve real seats for 100 days is S28-g's nightly scale. | **Bounded:** the Gate 2 of this PR is Alice and Bob (the two exports `History` already builds, B-19) in one shared home for the whole 100 days under `consolidate`, beside their two one-seat homes in lockstep (§6.5). |
| **F-S28b-13** | AS28-9's `keep-all` half needs every seat at the bound. Real social-cafe data reaches the 15 MiB target only after ≈ 36 days per seat in object bytes (FG-b2 §7), later in positional bytes, so several real seats at the bound cost minutes per CI leg. | **Bounded:** the half runs on a **multi-seat heavy stream** (synthetic, test-only; §6.4), which crosses the trigger in a few simulated days with records shared by every seat. Real data keeps the real-world half (Gate 2, §6.5). |

### 3.3 What does not exist

No attribution anywhere; no name check of prose; no oracle for summaries' structure against the facts; no
multi-seat heavy stream; no shared-home test on real exports; no test of the bound with records shared and the
backstop forced.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Attribution storage | Columns of the existing `forgotten` table | The set is already sorted, merged and bounded (FG-b §5.8); a second table would be a second authority for the same ranges |
| Name matching | The standard library's `re` with `re.escape`, whole-word, case-sensitive, longest name first (§5.4) | A named-entity recogniser (spaCy, etc.) would be a model-sized dependency that guesses; the check only needs the **known** names, which the seat already holds |
| C-4 oracle | Our own test module over the seat view's dump and the perceived facts | The oracle must not call the code under test (test rules §25); no library knows D-P4-7's episode rule |
| Multi-seat heavy stream | A function beside S28-a's `home_streams.world` | Same SplitMix64 draws as `retention_stream` |

No dependency is added. `DEP-58` (build on SQLite and P4/FG-b) covers the storage.

---

## 5. Design

### 5.1 Schema 5

On S28-a's `memory/home/schema.py` (C0 re-reads the exact DDL):

```sql
CREATE TABLE forgotten (seat INTEGER NOT NULL, first_key TEXT NOT NULL, last_key TEXT NOT NULL,
                        first_day INTEGER NOT NULL, first_policy TEXT NOT NULL,
                        last_day INTEGER NOT NULL, last_policy TEXT NOT NULL,
                        passes INTEGER NOT NULL CHECK (passes >= 1),
                        PRIMARY KEY (seat, first_key))
CREATE TABLE retention_passes (… schema 4's columns …,
                        prose_contradictions INTEGER NOT NULL)          -- after fallback_decisions
SCHEMA_VERSION = 5
```

- The view table `forgotten` dumps `first_key, last_key, first_day, first_policy, last_day, last_policy,
  passes`; `retention_passes` gains `prose_contradictions` as its last column. Codecs: `first_key`,
  `last_key` keep the event-key codec; the new columns are plain integers and strings (§5.4 of S28-a: "every
  other value as stored").
- A policy string is a registered policy id (`^[a-z][a-z0-9-]{0,47}$`, FG-b's registry) or `host:<step>`
  (F-S28b-4). No id and no free text enter it.
- `MemoryHome.open` refuses `home_meta.schema = "4"` with `UnsupportedStoreSchema(found=4, wanted=5)`, the
  message carrying S28-a's rebuild instruction; schemas 2 and 3 keep their refusal with `wanted=5`.
- Bytes: at most 256 ranges per seat × ≈ 40 B more per line ≈ 10 KB, inside `forgotten`, which the pass
  already counts as overhead (`items.OVERHEAD_TABLES`). The bound's numbers are untouched; `K` (an access
  line's overhead) is untouched.

### 5.2 Attribution: adding and merging

```python
@dataclass(frozen=True)
class Forgetter:                      # memory/retention/forgotten.py
    day: int                          # the pass's now // DAY_S
    policy: str                       # the pass's policy id, or "host:<step>"

@dataclass(frozen=True)
class PassScope:                      # one per pass, built at its start
    old: frozenset[str]               # first_key of every range the seat held when the pass began
```

- **Adding.** `plan.remove(…, forget=…)` takes `Forgetter | None` instead of `bool`. A new range is inserted
  with `first_day = last_day = forgetter.day`, `first_policy = last_policy = forgetter.policy`, `passes = 1`.
  `execute(…, forgetter)` passes the pass's `Forgetter(day, retention.policy.id)` to its `Drop`s;
  `enforce(…)` builds `Forgetter(day, "host:consolidate")` for step 1 and `host:chapters`,
  `host:chapterless-episodes` for steps 4 and 5 (B-3).
- **Merging** (`normalize(view, scope: PassScope)`). For each run of ranges that merges (FG-b's adjacency
  rule, unchanged) into one range `R`:
  - `first_day, first_policy` = those of the part with the smallest `(first_day, first_key)`;
  - `last_day, last_policy` = those of the part with the largest `(last_day, first_key)`;
  - `passes` (F-S28b-2, F-S28b-3). `PassScope` also holds `touched: set[str]`, the `first_key`s of every row
    this pass inserted or rewrote (in memory, owned by the pass, never stored). A part is **touched** if its
    `first_key ∈ touched`; its `passes` already includes this pass's 1. Then `passes(R)` = Σ `passes` over the
    untouched parts + Σ (`passes` − 1) over the touched parts + 1, and `R`'s `first_key` joins `touched`. A
    single untouched part is left as it is. So one pass adds exactly 1 to any range it changes, however often
    `normalize` runs, and nothing to a range it leaves alone. (`old` is kept for the review check that every
    untouched part was present when the pass began.)
  - The ranges, their order, the watermark rule and `MAX_RANGES` are FG-b's, unchanged: attribution never
    changes **which** ids are forgotten (I-b-2).
- **The watermark.** A range removed by the 256 cap takes its attribution with it; the watermark keeps FG-b's
  meaning and carries none. Recorded limitation (the step bounds attribution for the same reason).
- **Determinism.** Every input is a function of the seat's state at the pass and the pass's own decisions
  (FGb-5); nothing reads a rowid, a clock or a peer. Frames, interleavings and crash points give the same
  rows (I-S28-2, I-S28-5), which S28-a's harness checks because the rows are in the seat view's dump.

### 5.3 The attributed forgotten set (the API S28-e reads)

```python
@dataclass(frozen=True)
class ForgottenRange:
    first: EventId; last: EventId
    first_day: int; first_policy: str
    last_day: int; last_policy: str
    passes: int
    def forced(self) -> bool: ...            # either policy starts with "host:"

class ForgottenSet:                          # existing; load(view) unchanged in meaning
    def ranges(self) -> tuple[EventRange, ...]          # unchanged
    def attributed(self) -> tuple[ForgottenRange, ...]  # new: same order as ranges()
    def range_of(self, event: EventId) -> ForgottenRange | None   # new: the explicit range holding it
    def watermark(self) -> EventId | None               # unchanged
    def contains(self, event: EventId) -> bool          # unchanged
```

`range_of` answers step §5.6's "forgotten (the range, the pass day and policy that added it)"; S28-e renders
it. No other production code reads attribution.

### 5.4 The prose name check (FU-S28-1; `memory/retention/prose.py`, pure)

```python
def outside_names(prose: str, names: Mapping[EntityId, str], allowed: AbstractSet[EntityId]) -> tuple[str, ...]
    """The known names the prose uses that belong to no allowed entity, in the order they occur."""
```

- **Known names** = the values of the seat's `NameBook` at the pass (persons and places alike, B-12). Generic
  labels are not names.
- **Matching.** Each known name is searched as a whole word (`(?<!\w)` + `re.escape(name)` + `(?!\w)`),
  case-sensitive. Names are tried **longest first**; a match lying inside an earlier, longer match is
  ignored, so `Bob` inside an allowed `Bob Achterberg` is not a separate match. A name shared by several
  entities is allowed if any of them is allowed.
- **The allowed set** (F-S28b-5, F-S28b-7; QS28b-1 as recommended): the union of
  1. the seat's own entity;
  2. the counterparts and the place of the group's container and of **every item nested in it** at the pass,
     recursively (episode → its records; chapter → its episodes → their records);
  3. **(QS28b-1)** every entity whose known name occurs, by the same matching rule, in the `text` of the
     container or of an item nested in it (a quoted name; a merged chapter's own text).

  If QS28b-1 is answered "no", item 3 is deleted and nothing else changes.
- **The rule.** `outside_names(...) != ()` → the answer is a **contradiction**. Only known names are checked:
  an unknown capitalised word is not a name memory knows, and a partial name (`Bob` for an outside `Bob
  Smith`) is not detected — both recorded limitations (R-b-4).
- Pure: no store, no connection, no clock. Cost: one compiled pattern per pass over ≤ 160 B of prose per
  answer.

### 5.5 The host's refusal (`memory/retention/host.py`)

- `_assisted` loads `NameBook.load(view)` once per assisted pass (inside the pass's transaction, before any
  write) and passes it with `extracted` to `_accepted`.
- `_accepted` keeps its three discards (B-9) and adds, **after** them, for an answer with prose: compute the
  allowed set (§5.4) from `extracted` and the group; a contradiction discards the answer (its group keeps the
  fallback's decisions, exactly as any discarded answer) and counts in `contradictions`, not in `discarded`.
- **Outcome precedence:** a `StopReason` → `fallback:<reason>`; else any other invalidity (`answer.invalid +
  discarded > 0`) → `fallback:invalid`; else `contradictions > 0` → `fallback:prose-contradiction`; else
  `assisted`. `PassReport` gains `prose_contradictions: int = 0` after `trigger`; `_log` writes it.
- A plain pass and a size pass of an assisted seat (which never asks a model, FG-b2 §4.9) log 0.
- Nothing else in FG-c's branch changes: requests, groups, the decision table, the call allowance, the
  cassette keys. **Requests are untouched, so no recorded request moves and no `CassetteMiss` appears** (I-b-4).

### 5.6 Which cassettes change (FU-S28-1's obligation)

- `tests/cassettes/lm-summarise.jsonl`: requests unchanged. Its 180 answers are the ten names-free sentences of
  B-17 over a stream whose places are unnamed, so no answer names a known name and no group changes. C3 proves
  it two ways: a scan of the cassette's answers against the stream's known names (0 matches), and the replayed
  dump's digest **unchanged by C3** (it is re-pinned once, at C2, for schema 5 alone).
- `tests/cassettes/example.jsonl`: P5's decide example; no prose. Unaffected.
- No cassette is re-recorded (the contract's NEVER list).

### 5.7 The margin in positional bytes (F-S28a-8)

FG-b2 §4.1: `bound − trigger` (1.25 MiB at 20 MiB) must hold the growth of the one fact that crosses the trigger
plus `meta`, "a fact naming ≈ 3 000 participants" in object bytes. C5 measures, on a seat view, the content
growth of one fact naming `P` new participants for `P ∈ {1, 60, 600, 3 000}`, fits the per-participant slope
(it is exactly linear: one `l0_who` line and one `stable` line per new participant, plus constants), records
the participant count the margin holds in positional bytes, and asserts that a 3 000-participant fact's growth
plus `meta` is ≤ `bound − trigger` (the FG-b2 figure, as a literal). Nothing in `StoreBounds` changes.

---

## 6. Tests' world

### 6.1 Attribution tests

Hand-built seat views with literal expectations (no figure computed by the code under test): a `Drop` of a
chapter (two ranges) on day 12 by `consolidate`; a later adjacent `Drop` on day 40 by `decay`; a backstop drop
by `host:chapters`; the merge results `(12, consolidate, 40, decay, 2)`; a pass adding two adjacent ranges
counts 1; the F-S28b-2 case (P1 adds A and B apart, P2 drops the unit between) gives `passes = 3`, the
documented upper bound; the cap moves the oldest range into the watermark with its attribution gone.

### 6.2 The prose check tests

- Unit (`outside_names`): an allowed counterpart; the group's place; another known person; another known
  place; an unknown capitalised word; the seat's own name; `Bob` inside allowed `Bob Achterberg`; lowercase
  `grace`; a name with regex metacharacters (`Dr. O'Neil (Jr)`); a non-ASCII name (`Zoë`).
- Host level (`test_lm_summarise.py` additions, scripted answers): one assisted pass on a hand-built view with
  named people and places, four groups whose answers name (i) only allowed names, (ii) an outside person, (iii)
  an outside place, (iv) a name quoted in a member's gist (QS28b-1). Expected: (ii) and (iii) take the
  fallback's decisions and store no prose; (i) and (iv) are accepted; `prose_contradictions = 2`; outcome
  `fallback:prose-contradiction`; the dump equals a pass with (ii) and (iii) answered by nothing, except the
  pass-log row (a differential against the same pass with those two answers removed).
- Precedence: the same pass with one malformed answer → `fallback:invalid`, count still 2; with the
  allowance exhausted → `fallback:exhausted`, count still 2.

### 6.3 The C-4 oracle (`tests/summary_oracle.py`, test-only; imports no code under test)

Inputs: a seat view's positional dump (parsed with the same reader S28-a's ported `retention_oracle.py`
uses), the seat's perceived facts in order, and the seat's entity. Checks, per F-S28b-8 … F-S28b-10:

- **(a) episodes**: every perceived id in `[first_key, last_key]` has `at // 86 400 == day`; their number equals
  `records`; `counterparts` equals the ordered union of each fact's participants (deduplicated, envelope
  order) minus the seat; `place` equals the opening fact's place, or, when it has none, the place of the
  previous episode in **the oracle's own episode sequence**, derived from the facts by D-P4-7's three rules
  (written from the rule's text in `compress/episodes.py`'s docstring, not by calling it). **(a) chapters**:
  every cited id has `at // 86 400 // 7 == week`; `first_key`/`last_key` are the smallest and largest cited
  ids; `first_at`/`last_at` are the times of those facts.
- **(b)**: every retained record lies in a surviving episode (open or closed), never only in a chapter's
  ranges or the forgotten set.
- **(c)**: a retained record's day equals its episode's; a retained `notable` record with a place has its
  episode's place.

### 6.4 The multi-seat heavy stream (`tests/home_streams.py`, test-only)

`heavy_world(seed, days, seats=3, per_day=…, gathered=60)`: FG-b2's heavy facts (B-20) whose participants
always include every seat's entity (so every record is held by every seat: the worst case for I-a-6 (a) and
the best for sharing) plus, every 7th fact, a fact perceived by one seat only (drawn), so singly held records
exist too; all people named first. `per_day` is fixed at C0 from S28-a's merged `test_retention_size.py` so
that each seat crosses the trigger at least twice in the run (positional bytes are smaller than FG-b2's, S28-a
R-a-7); the value and the measured crossings are recorded.

### 6.5 The shared-home Gate 2 (`tests/test_home_retention_real.py`, `real_binary`)

`social_cafe_100`'s Alice and Bob exports (B-19), merged into one ordered delivery (each seat's own order
kept), day by day, into **one shared home** of the two seats and, in lockstep, into **two one-seat homes**,
under `consolidate` for all 100 days. At every pass of either seat: both seats' views ≤ target after the pass
and ≤ bound before it; coverage against each seat's export (AFGb-3's oracle); C-4 (a)–(c) (§6.3); each seat
view's dump in the shared home equal to its one-seat home's (AS28-3 on real data, at every pass point);
I-a-6 (a) and (b) as ruled. At the end: `verify="pass"` raised nothing (AS28-6). Budget ≤ 180 s per CI leg
(FG-b's stop); **pre-declared reduction** if over at C6: coverage and C-4 at every pass for days 0 … 30 and at
the passes of days 60 and 99, everything else unchanged (recorded; not a stop).

---

## 7. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-b-1 | Attribution is a function of the seat's perceived prefix and parameters, in the seat view's dump, identical under `none` and `home`, any frames, interleavings and crash points | S28-a's `test_home_transparency.py` and `test_home_kill.py` green on the new dump; AC-b-1 |
| I-b-2 | Attribution never changes which ids are forgotten: the `(first_key, last_key)` projection of every seat's forgotten set, and every other table, equal S28-a's on the same stream | AC-b-2 (parity against evidence frozen at C0) |
| I-b-3 | The prose check only refuses answers: a refused group's rows equal the fallback's; no decision, citation or request is added or changed (FGc-4, FGc-7, P4-1) | AC-b-4 |
| I-b-4 | Requests are unchanged: the committed cassette replays with zero `CassetteMiss`, and its replayed dump digest is the same before and after C3 | AC-b-5 |
| I-b-5 | C-4 (a)–(c) hold for every episode and chapter of every seat view at every pass | AC-b-6, AC-b-9 |
| I-b-6 | With `keep-all` everywhere at the bound: every seat view ≤ target after every pass and ≤ bound after every `absorb` call; I-a-6 (a) and (b) as ruled; a home-mate's forced forgetting changes no byte of another seat | AC-b-7, AC-b-8 |
| I-b-7 | No policy, item rule, bound number, ratio or backstop order changes; `forgetting/` has an empty diff; `bound.py`'s diff is the `Forgetter` plumbing only | C2 review; AC-b-11 |
| I-b-8 | I-S28-1 … I-S28-10 (I-S28-6 as ruled), I-a-1 … I-a-11, FGb-1 … FGb-7, FGb2-1 … FGb2-3, FGc-1 … FGc-7 hold | the ported suites and S28-a's harness, green |
| I-b-9 | No change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | AC-b-11 |

---

## 8. Test ownership and acceptance

### 8.1 Ownership (test rules §26)

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict over `cognition/lm-controller`; doc heading and decision-id scripts | CI `fast`, `python`, `docs` |
| Unit | attribution adding and merging, `passes`, the cap (`test_retention_forgotten.py`, new); `outside_names` (`test_retention_prose.py`, new); schema-5 refusal (`test_home_open.py`, extended) | `cognition/lm-controller/tests/` |
| Integration (synthetic) | attribution at pass level under each policy and the backstop; parity I-b-2; the host's refusal and outcome (`test_lm_summarise.py`, extended); C-4 (a)–(c) in a 12-seat shared home under every bundled policy (`test_home_summaries.py`, new); the bound at the bound, isolation and last holder under forced drops, the margin (`test_home_bound.py`, new) | same |
| Gate 2 (real lifecycle) | the shared home of Alice and Bob over the real 100-day history (`test_home_retention_real.py`, new, `real_binary`) | `python` job, three OSes |
| Gate 1 (real LLM) | **NOT REQUIRED.** The check is host-side and deterministic; no request changes; scripted and replayed answers only (QS10-19) | — |
| CI | the PR's runs on the exact head; selector: `cognition/lm-controller/**` and `docs/**` → `python` and `docs` layers | GitHub Actions |

Oracles are the perceived facts, the dumps and literals written in the tests, never a value the code under
test reports about itself (test rules §25).

### 8.2 Acceptance criteria (fixed before measuring, `ARC-23`)

| ID | Criterion | Step / design ref |
| --- | --- | --- |
| AC-b-1 | **Attribution.** §6.1's literal cases hold. On the synthetic 12-seat world (seed 7, 30 days) under `consolidate`, `decay` (threshold 1 000) and `importance-recency-cap` (k at 0) in one shared home: every range's `first_day ≤ last_day`, both ≤ the newest pass day, every policy string is the seat's policy id or one of the three `host:` steps, and `passes ≥ 1`; S28-a's transparency and kill tests pass on schema 5 | step §5.6; F-S28b-2 … F-S28b-4 |
| AC-b-2 | **Parity.** For the same runs, every seat view's dump with the five attribution columns and `prose_contradictions` removed equals, byte for byte, the dump recorded at C0 from S28-a's merged code (sha256 per seat, `tests/golden/s28b_parity.json`) | I-b-2, test rules §25 |
| AC-b-3 | **Refusal.** A schema-4 home and a schema-3 store each raise `UnsupportedStoreSchema(found, 5)` with the file's sha256 unchanged | F-S28b-1, QS28-8 |
| AC-b-4 | **FU-S28-1.** §6.2's unit and host cases hold literally, including the differential dump and both precedence cases | C-4 (d), FU-S28-1 |
| AC-b-5 | **Cassettes.** The committed cassette replays with zero `CassetteMiss`; 0 of its answers name a known name of the stream (scan); the replayed dump's sha256 at C3 equals the one re-pinned at C2; its rendered sections equal S28-a's C1 golden (AC-a-13's oracle) | FU-S28-1 "states which cassettes change" |
| AC-b-6 | **C-4 (a)–(c), synthetic.** On the synthetic 12-seat world (seed 7, 30 days) in a shared home, under `consolidate`, `decay`, `importance-recency-cap`, `keep-all` and `lm-summarise` (scripted answers, every column of FG-c's table exercised), §6.3's oracle holds for every seat view at every pass | C-4, AS28-18 |
| AC-b-7 | **AS28-9 at the bound.** On the multi-seat heavy stream, 3 seats, `keep-all` for all, frames of 256: at least two size passes per seat and `forced > 0` in each seat's log; after every pass each seat view ≤ target (15 728 640); after every `absorb` call each ≤ bound (20 971 520); at every pass point I-a-6 (a) and (b) as ruled (the shared home vs three one-seat homes in lockstep); the shared home's dump ≤ 3 × bound × (1 + `K` / smallest record line); under `verify="every-commit"` no `HolderInvariantBroken` | AS28-9, I-a-6 |
| AC-b-8 | **Isolation and last holder at the bound.** Seat A (`consolidate`) in a shared home whose two mates run `keep-all` at the bound (forced drops releasing records A also holds) has a dump and 200 seeded sections identical to A alone, at every pass point (AS28-5). A record held by A and B survives B's forced drop and is collected by A's later `Drop`, in A's transaction (AS28-7 under retention) | AS28-5, AS28-7 |
| AC-b-9 | **Gate 2.** §6.5 holds on Linux, macOS and Windows within 180 s per leg (or with its pre-declared reduction, recorded) | step §11.1 checkpoint |
| AC-b-10 | **Margin.** §5.7: a 3 000-participant fact's content growth plus `meta` ≤ `bound − trigger`; the slope and the participant count the margin holds are recorded | F-S28a-8 |
| AC-b-11 | **Scope.** `git diff --name-only origin/main...HEAD` lists only `cognition/lm-controller/**`, `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`, `.structured-coding/plans/mvp0/{pr-s28b-retention-on-seat-views.md, handoff-s28b.md, pr-fg-c-lm-summarise.md}`; `forgetting/` has an empty diff; no `uv.lock` change | I-b-7, I-b-9, AS28-13 |
| AC-b-12 | **The integration checkpoint.** S28-a's `test_home_transparency.py` (12 seats, `none` vs `home`) green on the exact head | step §11.1 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s28b`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered implementation steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, the
   load average, and PASS / FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the
   failing test's name), revert it from a byte copy (`cmp` clean), run the test again green.
5. Static checks: `uv run --locked ruff check cognition/lm-controller`; `uv run --locked ruff format --check
   cognition/lm-controller`; `uv run --locked pyright`. Tests one file at a time: `uv run --locked pytest
   cognition/lm-controller/tests/<file>`. **R-LOAD-1** (§11). Markdown: `python3 scripts/check_doc_headings.py`;
   `python3 scripts/check_decision_ids.py`; `python3 scripts/ci_layer.py docs`.
6. Review: answer each review question of the commit in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**. Otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`.
10. Stop and report (a material stop, §12) instead of improvising.

#### C0 — Base, re-audit and frozen parity evidence

- **Goal.** Establish the merged base and capture, **before any change**, the evidence AC-b-2 compares
  against.
- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s28b.md` (new),
  `cognition/lm-controller/tests/{capture_s28b_parity.py, golden/s28b_parity.json}` (new), `tests/golden/SOURCE.md`.
- Steps:
  1. Record the base commit; confirm S28-a's merge commit is on it (`git log --oneline --merges`) — absent →
     material stop. Record whether S28-c, S28-e or anything else touching `memory/` has merged since.
  2. Re-anchor B-1 … B-23 on the merged code (record moved lines; a changed **statement** is a stop if it
     breaks a §5 assumption). Record from the merged code: the exact schema-4 DDL of `forgotten` and
     `retention_passes`; `normalize`'s and `remove`'s signatures; the pinned `K`; the F-S28a-9 ruling and the
     form of I-a-6 (b) in force; S28-a's `test_retention_size.py` heavy parameters; the positional reader in
     `retention_oracle.py`; the FG-c replay `DIGEST` S28-a pinned.
  3. `capture_s28b_parity.py` (run by hand, documented in `golden/SOURCE.md` with the base SHA): for AC-b-1's
     runs (synthetic 12-seat world, seed 7, 30 days, shared home, the three policies), write per (policy,
     seat) the sha256 of the seat view's dump **at the end of the run**, on the merged S28-a code.
  4. Write the handoff (the contract's fields, the sequence C0 … C7).
- [ ] Implementation: steps 1–4. [ ] Validation: the doc scripts; the capture run once (wall time). [ ]
  Review: the base satisfies §2.3; the parity file was produced by unchanged code (`git diff --stat` of
  `src/` empty at capture).
- **Commit message:** `docs(plan): S28-b — open the ledger at the base, with frozen parity evidence`

#### C1 — The specifications and the FG-c pointer (Markdown only)

- **Goal.** Documentation before code (`CLAUDE.md` §2.2): the rules C2 and C3 implement are written first.
- **Files:** `docs/MODULE_SPEC.md` (§5.2), `docs/DECISIONS.md`, `.structured-coding/plans/mvp0/pr-fg-c-lm-summarise.md`.
- Steps:
  1. `MODULE_SPEC.md` §5.2 rule 6: add attribution — the five fields, `host:<step>` for the backstop, the merge
     rule, `passes` as an upper bound on distinct passes (F-S28b-2), the cap drops attribution.
  2. Rule 9: the pass log also records `prose_contradictions`.
  3. The `lm-summarise` row and the "Model calls" paragraph: the name rule of §5.4 (the allowed set as
     ruled by QS28b-1), the outcome `fallback:prose-contradiction` and its precedence.
  4. `DECISIONS.md`: a dated note under `ARC-87` (attribution; the prose name check, FU-S28-1) and under
     `ARC-109` (schema 5, F-S28b-1). No new id.
  5. `pr-fg-c-lm-summarise.md`: a dated pointer under §5.5: "FU-S28-1 (step-28 §11.1) adds the host's name
     check of prose; applied by S28-b, `pr-s28b-retention-on-seat-views.md` §5.4". Nothing else in that file.
- [ ] Implementation: steps 1–5. [ ] Validation: `check_doc_headings.py`, `check_decision_ids.py`,
  `ci_layer.py docs`. [ ] Review: every term is `CORE_CONCEPTS`/`MODULE_SPEC`'s; no rule is stated that §5
  does not implement; the FG-c file's frozen sections are untouched (pointer only).
- **Commit message:** `docs(spec): forgotten-range attribution and the prose name rule (S28-b)`

#### C2 — Schema 5 and forgotten-range attribution

- **Goal.** Every range says who forgot it (§5.1–§5.3), with no change to what is forgotten.
- **Files:** `memory/home/schema.py`, `memory/home/home.py` (refusal of 4), `memory/retention/{forgotten.py,
  plan.py, bound.py, host.py, __init__.py}`, `tests/{test_retention_forgotten.py (new), test_home_open.py,
  test_lm_summarise_replay.py, test_retention_host.py}`, plus any ported test whose hand-checked dump shape
  names the `forgotten` columns (listed in §15 when found).
- Steps:
  1. `schema.py`: §5.1 (DDL, view columns, `SCHEMA_VERSION = 5`; `prose_contradictions` written as 0 by
     `_log` until C3).
  2. `home.py`: the refusal of schema 4 (§5.1).
  3. `forgotten.py`: `Forgetter`, `PassScope`, `ForgottenRange`, `attributed()`, `range_of()`; `normalize(view,
     scope)` with §5.2's merge rule; `load` reads the new columns.
  4. `plan.py`: `remove(…, forget: Forgetter | None)`, `execute(…, forgetter)`.
  5. `bound.py`: `enforce(view, now, bounds, scope)` builds the three `host:` forgetters; `_shed` and
     `consolidated` pass them and the scope. No step, order or number changes.
  6. `host.py`: `run_pass` builds `PassScope` before `extract` and `Forgetter(day, retention.policy.id)`;
     `_log` writes `prose_contradictions`.
  7. `memory/retention/__init__.py`: export `ForgottenRange` (and no other new name).
  8. Tests: §6.1 (`test_retention_forgotten.py`); AC-b-1's runs; AC-b-2 against C0's parity file; AC-b-3;
     `test_lm_summarise_replay.py`'s `DIGEST` re-pinned **only after** zero `CassetteMiss` and sections equal
     to S28-a's golden (record old and new digests).
- Validation: each listed file; S28-a's `test_home_transparency.py`, `test_home_kill.py`, `test_home_golden.py`,
  `test_retention_size.py`, `test_retention_policies.py` one at a time (the dump changed). Mutations **M-b-1**
  (merge takes the newer part's `first_*`) → `test_retention_forgotten.py::test_a_merge_keeps_the_first_forgetter`
  red; **M-b-2** (backstop drops attributed to the pass's policy) →
  `…::test_the_backstop_signs_its_own_drops` red; **M-b-3** (`normalize` adds 1 at every call) →
  `…::test_one_pass_counts_once` red; **M-b-4** (schema 4 accepted) → `test_home_open.py` red.
- [ ] Implementation: steps 1–7. [ ] Validation: step 8, the S28-a files, mutations. [ ] Review: no policy
  string can carry an id or free text; `normalize`'s range output is FG-b's (diff of the range logic is the
  attribution only); `forgetting/` diff empty; nothing reads a rowid.
- **Commit message:** `feat(cognition): forgotten ranges say who forgot them — schema 5 (S28-b)`

#### C3 — FU-S28-1: the host refuses prose that names outside its group

- **Goal.** C-4 (d) in FG-c's merged code (§5.4–§5.6).
- **Files:** `memory/retention/prose.py` (new), `memory/retention/host.py`, `tests/{test_retention_prose.py
  (new), test_lm_summarise.py, test_lm_summarise_replay.py (a scan test only), test_memory_isolation.py (allow
  `re` in `memory/retention/prose.py` if its import scan requires it)}`; if QS28b-2 is ruled yes,
  `compress/model_summarizer.py` and `tests/test_memory_summarizers.py` too.
- Steps:
  1. `prose.py`: `outside_names` (§5.4) and `allowed_entities(extracted, container, me, names)` (items 1–3).
  2. `host.py`: `_assisted` loads the name book; `_accepted` refuses contradictions (§5.5); outcome
     precedence; `PassReport.prose_contradictions`; `_log` writes it.
  3. (QS28b-2 = yes) `embellish` applies the same check with the episode's or chapter's own entities and text
     as the allowed set; a contradiction is `discarded` (its existing counter).
  4. Tests: §6.2; the cassette scan (AC-b-5); `DIGEST` **unchanged**.
- Validation: the listed files one at a time; `test_assisted_host.py`. Mutations **M-b-5** (allowed set without
  nested items) → the host case (i) red; **M-b-6** (no seat entity in the allowed set) →
  `test_retention_prose.py::test_the_seat_may_name_itself` red; **M-b-7** (no longest-first masking) →
  `…::test_a_shorter_outside_name_inside_an_allowed_one_is_no_match` red; **M-b-8** (a contradiction counted as
  `invalid`) → the precedence test red; **M-b-9** (the check runs before `valid_prose`'s discard, double
  counting) → the host counts test red.
- [ ] Implementation: steps 1–3. [ ] Validation: step 4, mutations. [ ] Review: no request byte changes (diff
  of `lm_summarise.py` empty); the check is pure and reads the name book once; a refused group's rows are the
  fallback's (the differential test); no `CassetteMiss` path changed.
- **Commit message:** `feat(cognition): prose may not name outside its group — FU-S28-1 (S28-b)`

#### C4 — C-4 (a)–(c): summaries never contradict retained detail

- **Goal.** AC-b-6 with an oracle independent of the code under test.
- **Files:** `tests/summary_oracle.py` (new), `tests/test_home_summaries.py` (new).
- Steps: §6.3's oracle; AC-b-6's five policies in a 12-seat shared home; `lm-summarise` with FG-c's scripted
  answers (`test_lm_summarise.scripted`).
- Validation: the file (≤ 120 s on Linux CI; split by policy across tests if needed). Mutations **M-b-10**
  (`_close` writes `records + 1`, `compress/episodes.py`) → (a) red; **M-b-11** (`plan.remove` of a level-1
  item into a chapter leaves its records) → (b) red; **M-b-12** (an ambient record tests the place in `_fits`)
  → the oracle's episode sequence differs, (a) red on `place`.
- [ ] Implementation. [ ] Validation. [ ] Review: the oracle imports nothing from `mineworld_cognition`
  except the dump format's documented grammar; the episode rule is transcribed from D-P4-7's text, and the
  review compares it line by line with the docstring, not with the code.
- **Commit message:** `test(cognition): summaries never contradict retained detail — C-4 (S28-b)`

#### C5 — The bound at the bound: `keep-all` everywhere, isolation, last holder, margin

- **Goal.** AC-b-7, AC-b-8, AC-b-10.
- **Files:** `tests/home_streams.py` (`heavy_world`), `tests/test_home_bound.py` (new).
- Steps: §6.4; AC-b-7 (shared home vs three one-seat homes in lockstep; `verify="every-commit"`); AC-b-8;
  §5.7's margin measurement.
- Validation: the file (≤ 120 s on Linux CI; record per test). Mutations **M-b-13** (`release` skips `collect`
  when called from the backstop's `_shed`) → AC-b-7's `HolderInvariantBroken` / (a) red; **M-b-14** (the size
  trigger reads the home's total instead of the seat's) → AC-b-8 isolation red.
- [ ] Implementation. [ ] Validation. [ ] Review: the bound literals are written in the test (`20 × 1 048 576`,
  `× 3 // 4`), never imported; I-a-6 (b) is tested in the form ruled for F-S28a-9 (C0 records it).
- **Commit message:** `test(cognition): the bound holds at the bound in a shared home (S28-b)`

#### C6 — Gate 2: the real history in a shared home

- **Goal.** AC-b-9 (step-28 §11.1: "from S28-b on, it also runs FG-b's real-binary Gate 2 scenario on seat
  views").
- **Files:** `tests/test_home_retention_real.py` (new).
- Steps: write §15's Gate 2 specification **before the run** (claim, owner, head, command, budget, PASS,
  counterfactual); §6.5; `cargo build -p mineworld-cli`; one local run under the slot wrapper (R-LOAD-1).
- Validation: the file once locally (wall time, load); its CI time per leg recorded at C7. Classify PASS /
  FAIL / INCONCLUSIVE from the printed sizes and assertions, not the exit code.
- [ ] Implementation. [ ] Validation. [ ] Review: the save is read only through the CLI (`History`); no
  oracle calls `mineworld_cognition`; the two one-seat homes are fed the same frames as the shared one.
- **Commit message:** `test(cognition): FG-b's real Gate 2 in a shared home of Alice and Bob (S28-b)`

#### C7 — Close-out

- **Files:** this file (§15), `handoff-s28b.md`.
- Steps: AC-b-11's diff scan; fill §8's evidence; push `mvp0/pr-s28b`; `gh pr create`; CI on the exact head
  (Linux, macOS, Windows); repair to green; record each leg's time for `test_home_retention_real.py`,
  `test_home_bound.py`, `test_home_summaries.py`.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded; §10's
  parent edits listed for the primary.
- **Commit message:** `docs(plan): S28-b close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs, and parent edits (owner: the primary session)

- **R-S28e-b-1 (on S28-e).** `inspect … forgotten` and `access --event` read `ForgottenSet.attributed()` and
  `range_of()` (§5.3); "forced" is `ForgottenRange.forced()`; the pass log's `prose_contradictions` is shown in
  the last-32-passes table.
- **R-S28c-b-1 (on S28-c).** The scope-private backstop step (step §7.2) signs its drops `host:scope-rows`
  if it ever adds a forgotten range (it should not: scope rows cite, they do not cover).

| Document | Edit |
| --- | --- |
| `step-28-shared-memory.md` §9.2 C-4 | (b) reads "every retained L0 record lies in a surviving episode" (F-S28b-8); (c) "… agree on day, and on place for a notable record with a place" (F-S28b-9); (a) for chapters as F-S28b-10; (d) the allowed set as ruled by QS28b-1 |
| `step-28-shared-memory.md` §5.6 | attribution's policy values (`host:<step>`, F-S28b-4) and `passes` as an upper bound (F-S28b-2) |
| `step-28-shared-memory.md` §10 | the home is schema 5 after S28-b (F-S28b-1) |
| `overall.md` | S28-b frozen / merged |

---

## 11. R-LOAD-1

Locally: only the commit's test files, **one at a time**, plus ruff, ruff format, pyright and the doc
scripts. Never `cargo test --workspace`, never the whole `cognition/lm-controller` suite, never a chunked
equivalent. The heavy files (`test_home_bound.py`, `test_home_summaries.py`, `test_home_retention_real.py`,
and S28-a's `test_home_transparency.py` / `test_home_kill.py` when re-run) go through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one at a time, with the load average recorded
beside the wall time. The PR's CI on the exact head is the gate and the one full run.

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S28-b — retention on seat views: forgotten-range attribution,
                         C-4 and FU-S28-1, the bound at the bound, Gate 2 in a shared home
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s28b-retention-on-seat-views.md
RELATED / BINDING DOCS:  step-28-shared-memory.md (§§3, 5.6, 7, 9.2, 11, 11.1, 14.1, 15);
                         pr-s28a-memory-home.md (§§5, 6, 10, 14.1, 15 — its merged ledger);
                         pr-fg-b-retention-host.md; pr-fg-b2-size-trigger.md; pr-fg-c-lm-summarise.md;
                         docs/MODULE_SPEC.md §5.2; docs/ARCHITECTURE.md §9.3; docs/DECISIONS.md ARC-87,
                         ARC-98…ARC-101, ARC-109; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s28b, its own, held by one session
BRANCH:                  mvp0/pr-s28b, created from origin/main after S28-a's merge (not stacked)
START CONDITION:         S28-a merged (met: #242, 9884c64c) AND this design frozen (met: operator,
                         2026-10-18, §14 as recommended) AND the primary session's dispatch (step-28 §11.1)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–5, as §14's rulings shape them                      (at freeze)
FROZEN INVARIANTS:       step-28 I-S28-1 … I-S28-10 (I-S28-6 as ruled by QS28a-1 and F-S28a-9's ruling);
                         S28-a I-a-1 … I-a-11; §7 I-b-1 … I-b-9; FGb-1 … FGb-7, FGb2-1 … FGb2-3,
                         FGc-1 … FGc-7; FG-b2's bound numbers unchanged; §8.2 as written
SEQUENCE:                C0 … C7 (§9)
ALLOWED COMMANDS:        uv run --locked *; uv sync --locked; cargo build -p mineworld-cli; git; gh (never
                         merge); python3 scripts/*; python3 cognition/lm-controller/tests/capture_s28b_parity.py
                         through uv run; target/*/mineworld *; mkdir -p; sed -n; cmp; cp (mutation copies)
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or the
                         whole member suite locally (R-LOAD-1); reading any key or .env; calling any model;
                         any record mode for cassettes (LMS_RECORD); edits outside APPROVED SCOPE
MATERIAL STOPS:          S28-a not merged; a needed change to a policy, an item rule, a bound number, a ratio
                         or the backstop's order; a needed change to a request of lm-summarise; a needed
                         change outside cognition/lm-controller, docs/MODULE_SPEC.md, docs/DECISIONS.md and
                         the three plan files; a new dependency or a uv.lock change; AC-b-2 (parity) failing
                         for a reason other than the attribution columns; AC-b-5 showing any answer of the
                         committed cassette changing its group; AC-b-7/8 failing for a reason the attribution
                         or the check cannot explain (a peer dependence in S28-a's port); a re-recorded cassette
                         (never)
PLATFORMS:               Linux, macOS, Windows (python job on all three)
VALIDATION BUDGET:       unit and integration files unrestricted locally, one at a time; test_home_bound.py
                         and test_home_summaries.py ≤ 120 s each on Linux CI; test_home_retention_real.py
                         ≤ 180 s per leg (§6.5's pre-declared reduction if over); CI: the PR's runs.
                         Gate 1: NOT REQUIRED. Total autonomous runtime envelope ≈ 1 hour of local test time
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s28b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                     (source: the operator's freeze of this design, 2026-10-18)
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW       (the dispatch of this design)
                         merge: NOT authorized — never by the implementation session; only with the
                         operator's explicit authorization after the primary's and an Opus logic review
                         (working rules §22)
POST-MERGE SYNC OWNER:   primary: step-28 (§10's edits), overall.md; implementation session: this ledger
STOP CONDITION:          S28-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

**Implementer:** Opus-class (step-28 §11.1: "the invariants are too easy to break subtly"). **Reviewer:** the
primary session plus an Opus logic review.

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-b-1 | S28-a's merged code differs from the branch read here (moved lines, renamed helpers, a different `normalize` signature) | C0 re-anchors B-1 … B-23 on the merged code; a changed statement that breaks §5 is a stop |
| R-b-2 | The F-S28a-9 ruling changes I-a-6 (b)'s form | AC-b-7 tests the form in force at C0, recorded |
| R-b-3 | The name check refuses faithful prose (false positives) and quietly turns `lm-summarise` into `consolidate` | QS28b-1's allowed set; `prose_contradictions` makes the rate visible per pass; the unit cases pin the edges |
| R-b-4 | The name check misses contradictions (partial names, unknown names, a wrong name for the right person) | Recorded limitation: the rule checks known names only, as ruled; finer checks would need a model or a recogniser (declined, §4) |
| R-b-5 | Positional bytes make the heavy stream too light to reach the bound in CI time | `per_day` fixed at C0 from S28-a's measured settings; ≥ 2 crossings asserted as a precondition |
| R-b-6 | The shared-home Gate 2 exceeds 180 s on Windows | §6.5's pre-declared reduction, recorded on the PR |
| R-b-7 | `passes` read as an exact count | Named an upper bound in code, `MODULE_SPEC` and S28-e's requirement |
| R-b-8 | Schema 5 right after schema 4 costs developers a rebuild | QS28-8: memory is a derivation, rebuilt from a null cursor; no release separates the two |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS28b-1 [operator — FU-S28-1's allowed set]** | FU-S28-1 says prose "may name only known names of that group's counterparts, and the group's place". Read literally, two faithful cases fail (F-S28b-5): (1) a **chapter whose episodes were already merged into it** has no counterpart and no place at the pass, so any prose naming a person is refused, and `lm-summarise` can no longer rescue an old named chapter from being dropped; (2) a **name quoted in the group's own text** (`Carol said: "Bob is late"`), which P4's instruction tells the model to keep, is refused when Bob is not a counterpart. Should a known name that already occurs in the text of the group's container or of an item nested in it also be allowed? | **Yes.** The rule's purpose is that prose never *introduces* a person or place the group did not involve; a name the group's own memory text already holds is not an introduction. The check stays host-side, deterministic and limited to known names. With "no", only item 3 of §5.4 is deleted, and both cases above take the fallback. **Ruled 2026-10-18 (operator): yes, as recommended** — a known name already present in the group's original text is allowed; §5.4 item 3 stands. |
| **QS28b-2 [operator — scope]** | P4's `embellish` also writes model prose through `valid_prose`, without any name rule (F-S28b-11). It has no production caller today. Apply the same check to it in this PR? | **Yes.** C-4 is about every summary, the check is one call with the unit's own entities and text as its allowed set, and leaving it out would let the first caller of `embellish` write prose the host refuses from `lm-summarise`. It adds two files to C3 and no new concept. **Ruled 2026-10-18 (operator): yes, as recommended** — `embellish` applies the same name check in C3 (step 3). |

Not asked, because a ruling settles them: the schema number (QS28-8, F-S28b-1); the size rule (QS28-4); I-a-6
(QS28a-1, and F-S28a-9's pending ruling, which is S28-a's question).

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] base recorded, B-1 … B-23 re-anchored, parity captured, handoff | [x] doc scripts PASS; capture ran once (11.3 s CPU) | [x] base satisfies §2.3; `src/` diff empty at capture |
| C1 … C7 | — | [ ] | [ ] | [ ] |

### Gate 2 specification

(written at C6, before the run)

### Evidence

**C0 (2026-10-18).**

- Base: `origin/main @ b85f2804` (#260). S28-a's merge `9884c64c` is an ancestor. Since it, nothing under
  `memory/`, `compress/` or `forgetting/` has merged (`git log 9884c64c..HEAD` on those paths: empty). Open PRs that
  touch them, to adapt to whichever merges first: #249 (P6c + FG-d: `consolidate` keepsakes and the host's
  `_protected` filter in `_assisted`, F-FGd-2; `retention/__init__.py` gains `EXCHANGE_KINDS`) and #262 (FU-S28a-1:
  `home_meta.created`, not dumped; touches `home.py` and `schema.py`). #258 (per-file decision records) is open, so
  this PR writes its notes in `docs/DECISIONS.md`. #171 (12d) changes AC-a-9's bound, which this PR does not test.
- Re-anchored on the merged code (statements unchanged): B-1 `memory/home/schema.py:65-66` (DDL), `:188` (view
  columns `first_key, last_key`); B-2 `retention/plan.py:120-153`, insert `:140-147`; B-3 `retention/bound.py:110-118`
  (`consolidated`), `:149-170` (`_shed`), `:184-194` (`_chapters`, `_chapterless_episodes` return `forget=True`);
  B-4 `retention/forgotten.py:54-77`, called at `host.py:107`, `bound.py:129`, `:162`, `:169`; B-5
  `forgotten.py:23-51`, read by no production module (`grep ForgottenSet src`: the package docstring only); B-6
  `host.py:93-131`, `:347-379`; B-7 `compress/model_summarizer.py:68-74`; B-8 `forgetting/lm_summarise.py:195`; B-9
  `host.py:263-290`, precedence `:223-228`; B-10 `:229-236`, `:327-344`; B-11 `retention/items.py:134-172`, `:174-213`,
  `:215-243`; B-12 `memory/names.py:18-80`; B-14 `compress/episodes.py:59-117` (inheritance `:64`: the open episode's
  place); B-16 `embellish` at `model_summarizer.py:77`, no production caller; B-18 `test_lm_summarise_replay.py:39`
  (`DIGEST 2163d85b…df00a`, S28-a's pin).
- Recorded from the merged code: `normalize(store)`, `remove(store, extracted, item, *, forget: bool)`,
  `execute(store, resolved, extracted)`, `enforce(store, now, bounds)`; the pinned `K` =
  `schema.K_ACCESS_OVERHEAD_BYTES` = 93; **F-S28a-9 has no ruling on record** (no amendment in step-28 or
  `DECISIONS.md`), so I-a-6 (b) is tested as frozen: one-seat home ≤ view + `K` × held records; S28-a's heavy
  parameters `test_retention_size.BUSY = (7, 2, 2800, 220)` (≈ 8.7 KB of positional dump per fact); the positional
  reader `retention_oracle.Dump` (leading columns per table, keys padded back to 20 digits).
- A schema-4 home is already refused by `home._check` (`schema != SCHEMA_VERSION`), so the refusal of 4 at C2 is the
  version bump itself; M-b-4 is planted in `_check`.
- Parity capture (AC-b-2's oracle): `capture_s28b_parity.py --write` under `with-slot.sh`, 11.3 s CPU (24 min wall,
  all of it waiting for a slot; load 18 at the end), with `git diff --stat -- cognition/lm-controller/src` empty.
  36 digests (3 policies × 12 seats) in `tests/golden/s28b_parity.json`; `golden/SOURCE.md` records it.
- Doc scripts: `check_doc_headings.py`, `check_decision_ids.py`, `ci_layer.py docs`: PASS.
- Process note: one composed shell line at C0 began a heredoc to `python3`; the chain stopped at its first command
  (`head -c 0` failed), so nothing ran. Recorded against the NEVER list.

### Mutations

(none yet; M-b-1 … M-b-14 as §9 lists them)

### Deviations and findings

F-S28b-1 … F-S28b-13 (§3.2), recorded at design time.

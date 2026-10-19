# PR P6c + FG-d — memory coherence for the people you talk to

## DRAFT — for the primary session to freeze

```text
Scope authority:        pr-s10-p7-milestone-d.md §7 Q-P7-2 (recommendation (b)) and Q-P7-3 (recommendation (a)), the
                        coordinator's instruction of 2026-10-16 (the two fixes, their acceptance, and the order:
                        design as DRAFT, then implementation in the same branch); the operator's 2026-10-16 merge
                        authority, under which the primary may freeze bounded follow-ups like this one
Binding parents:        pr-s10-p6-lm-controller.md (DESIGN FROZEN 2026-10-14) §5.2.3 (memory view), §5.2.4 (prompt);
                        pr-fg-b-retention-host.md §5.2, §5.4, §5.7, §5.9 (the policy interface, host importance, the
                        pass, the bundled policies); pr-fg-b2-size-trigger.md §4.1, §4.3 (20 MiB bound, size passes);
                        pr-fg-c-lm-summarise.md §5.1 (groups are consolidate's); step-28-shared-memory.md §4.5, §5.3
                        (S28-a's SeatView, for compatibility only)
Implementation base:    main (P7 #244 merged 2026-10-18; first implemented on origin/mvp0/pr-s10-p7 @ 66121eac).
Branch / worktree:      mvp0/pr-p6c-fgd, /Users/yuema137/mineworld-worktrees/impl-p6c-fgd (this session alone)
Lifecycle:              DRAFT (not frozen). Implementation proceeds in the same branch on the coordinator's
                        instruction; the order is recorded in §9.
```

**Effort:** `mvp0` · **Step:** [`step-17-cognition.md`](step-17-cognition.md) · **Parent PR:**
[`pr-s10-p7-milestone-d.md`](pr-s10-p7-milestone-d.md)

---

## 1. Goal

Two defects Milestone D measured, both against the operator's binding principle that people have persistent, coherent
memory, so the user feels the world continues:

1. **P6c (Q-P7-2, F-P7-5).** The 3D reply of the local model (`qwen3.5:4b`) ignores the two 2D lines although both are
   in its context: they are two of 48 lines in "The last day", behind name-learning noise. Fix: put **what the current
   speaker and I said before** in a section of its own at the top of WHAT I REMEMBER, and add one rule line telling the
   model to use it.
2. **FG-d (Q-P7-3, F-P7-8).** After 100 simulated days (2 142 rule-driven exchanges with the visitor), the default
   policy `consolidate` has merged the day-1 records into their episode and the episode into its chapter, and nothing
   the day-100 context shows cites the 2D line or Alice's reply. Fix: a salience-aware, tunable `consolidate`: the
   first lines a person exchanged directly with each counterpart are kept, as records, long-term, within the 20 MiB
   bound.

The two fixes meet in one place: FG-d keeps the first exchanges with each person as records, and P6c's section is
where retrieval shows them.

## 2. Non-goals

- No change to the kernel, contracts, server, persistence, systems, protocol, or any World Pack.
- No change to the store schema (`SCHEMA_VERSION` stays 3); S28-a owns schema 4.
- No change to the host bound, its target, its trigger, or the backstop's steps (§4.4 says why).
- No change to `decay`, `importance-recency-cap`, `keep-all`; `lm-summarise` inherits FG-d through `consolidate`'s
  groups (pr-fg-c §5.1) without its own change.
- No paid model call and no key read. The cassette is recorded with the local model only.

## 3. Audit (origin/mvp0/pr-s10-p7 @ 66121eac)

| Item | Where | What matters |
| --- | --- | --- |
| Retrieval | `memory/retrieve.py` `retrieve`, `BUDGETS`, `MemorySection.render` | four sections L3, L2, L1, L0, budgets 1 600 + 800 + 1 600 + 2 000 = 6 000 bytes (`ARC-59`, D-P4-9); `RecallQuery(at, counterparts, words)`; L0 is the last 86 400 s, newest first |
| Memory view | `controller/memory_view.py` | applies `recall_episodes` to L1 only; logs the `MemorySection` |
| The consult's query | `controller/seat.py` `_consult_once` | `RecallQuery(at=observation.at, counterparts=present + speaker, words=trigger.words)`; the trigger's speaker and event are known here |
| Prompt | `controller/prompt.py` `_system`, `_user` | WHAT I REMEMBER is `memory.render()`; the rules end with "answer them as yourself, using what you remember of them" (D-P7-7) |
| Golden prompt | `tests/fixtures/prompt_request.json`, `test_controller_choice.py` | pins the request byte for byte |
| The recorded 3D consult | `tests/fixtures/milestone_d.decisions.jsonl`, consult on #75 | 11 L3 lines, then 31 L0 lines; #59 (Vera's 2D line) is the 23rd line; the model answered `recalls: ["[#75]"]` |
| Exchange records | `memory/render/conversation.py` `spoke` | gist `Vera said to me: «…»` / `I said to Vera: «…»`; counterparts = participants without me, so a line I said or heard directly has exactly one counterpart, an overheard one two; base importance 700 / 600 / 200 |
| L0 row | `memory/store.py` `l0` | has `kind`, `involved`, `importance`; `l0_who` holds counterparts |
| `consolidate` | `forgetting/consolidate.py` | merges every L0 record past `l0_hours` (0) into its episode, every chaptered episode past `l1_weeks` (8) into its chapter, drops chapters past `l2_years` (5) unless they hold a stable fact's moment |
| Policy interface | `memory/retention/policy.py` `MemoryItem` | `kind` (L0: event type), `counterparts`, `mine`, `importance` (host, with a reinforcement bonus up to +200, so a threshold on it does not separate kinds), `container` |
| Plan validity | `memory/retention/plan.py` `validate` | a record survives only if its episode survives; an episode only if its chapter does; absence of a decision is a keep |
| The size pass | `memory/retention/host.py` `_decide`/`_plain` | a plain policy plans the same way for `day` and `size` passes |
| The backstop | `memory/retention/bound.py` `enforce` | first `consolidate`'s defaults (`fallback()` binds `{}`), then every remaining closed record, oldest first, and so on |
| `lm-summarise` | `forgetting/lm_summarise.py` `_groups` | `Consolidate().plan(items, budget, params.consolidate)`: groups are consolidate's |
| Spec | `docs/MODULE_SPEC.md` §(bundled policies) | the `consolidate` row lists its rule and parameters |
| 100-day test | `tests/test_milestone_d_100_days.py` | strict `xfail` on the citation clause; its `RecallQuery` has no speaker |
| Replay test | `tests/test_milestone_d_replay.py` | prints `recalls_cite_a_2d_line`, `words_mention_the_umbrella` instead of asserting |
| Scripted model | `tests/milestone_d_cognition.py` `remembers` | quotes the earliest line "said to me" it is shown, by Event ID; order of sections does not matter |
| S28-a (in review, #242) | `step-28-shared-memory.md` §4.5, §5.3 | replaces `retrieve`'s reads of `l0`/`l0_who` with `SeatView` methods of the same `ORDER BY`/`LIMIT`; policies see `MemoryItem`s, unchanged |

## 4. Design

### 4.1 P6c — the speaker's section

- `RecallQuery` gains `speaker: EntityId | None = None` (the person who just spoke to me) and `heard: EventId | None =
  None` (the trigger line, which WHAT JUST HAPPENED already shows). `seat._consult_once` sets both from the trigger.
- `retrieve` emits a new first section, `with`, when `speaker` is set:
  - header `What <name> and I said to each other before:` (the name from the seat's `NameBook`; never an id, I-10);
  - lines: **exchanges** with the speaker — L0 records of an exchange kind (`EXCHANGE_KINDS = ("spoke",)`, owned by
    `memory/render/conversation.py`), `involved = 1`, whose only counterpart is the speaker, at or before `at`, other
    than `heard`;
  - selection: the **first** `WITH_FIRST = 2` such records (the first words, which FG-d keeps) and then the newest,
    up to 6 lines and 1 000 bytes, rendered oldest first, each `Day N HH:MM: <gist> [#id]` as in L0;
  - two bounded queries (`ORDER BY event_key ASC LIMIT 2`, `DESC LIMIT 6`), so S28-a ports it as one `SeatView`
    method with the same order and limits.
- A line shown under `with` is not shown again under L0.
- Budgets: `with` 6 lines / 1 000 bytes, taken from L0, which goes from 2 000 to 1 000 bytes (48 lines). The section
  stays ≤ 6 000 bytes (`ARC-59` unchanged).
- `MemorySection` gains `speaker: str | None = None` (the header's name). `SectionLine.level` accepts `"with"` besides
  `L3 … L0`. Both are additive: P7's logs still parse.
- `memory_view`'s `recall_episodes` knob is unchanged (L1 only).
- Prompt: one rule line in `_system`: *"If WHAT I REMEMBER begins with what the speaker and I said to each other
  before, your answer must take it up: refer to what was said, and cite those tokens in recalls."* The golden request
  is regenerated.

### 4.2 FG-d — keepsakes in `consolidate`

A new bounded, defaulted parameter, so the choice stays in FG-b's pluggable model (a world tunes or disables it in
`decisions.yaml` `memory: parameters:`; no host change):

- `keepsakes` 0 … 64 (default **4**): per counterpart, the first `keepsakes` exchange records (an L0 item whose `kind`
  is in `EXCHANGE_KINDS` and whose only counterpart is that person — a line I said to them or they said to me) are
  kept as records. Their episode does not merge into its chapter, and a chapter holding such an episode is not dropped
  (as a chapter holding a stable fact's moment is not). Every other record of that episode still merges into it.
- `0` is the previous behaviour exactly (parity test).
- "First" is the event order of the surviving items, so the choice is stable from pass to pass and a pure function of
  the items; ties are impossible (Event IDs are distinct).
- `EXCHANGE_KINDS` is re-exported from `mineworld_cognition.memory.retention`, so `consolidate` keeps importing only
  that package's public names (MODULE_SPEC: bundled policies import only public names).

### 4.3 Bound

Kept records cost one L0 row and one `l0_who` row each (≈ 350–450 B in the dump), and they keep their episode's row
alive: at most `keepsakes` × people records plus one episode per kept record. For 4 keepsakes and 500 people that is
≈ 2 000 × (450 + 400) ≈ 1.7 MB, about 11 % of the 15 MiB target, and in Milestone D's 100-day run a handful of people.
The share is bounded by construction, not by luck.

### 4.4 Size passes and the backstop

A size pass (FG-b2) runs the seat's own policy (`host._plain` for both causes), so `consolidate` keeps keepsakes
whether the pass is daily or early. The backstop's first step is `consolidate` with its defaults, which keeps them
too. Only if the store is still over its target after that does the backstop shed every closed record, keepsakes
included: the bound is the host's and always wins (FGb-2). Changing the backstop order would change FG-b2's measured
behaviour for a case §4.3 bounds away; it is not done here.

### 4.5 S28 compatibility

- P6c adds one bounded read over `l0` and `l0_who`; under S28-a it becomes one `SeatView` method with the same
  `ORDER BY`/`LIMIT` (S28 §4.5), joined to `access` as every read is.
- FG-d is inside a policy, which sees `MemoryItem`s only; S28 does not change that interface.

## 5. Invariants

- **I-P6c-1** The memory section stays ≤ 6 000 bytes; every line cites only facts the store holds; `recalls` can cite
  only shown lines (P6, unchanged).
- **I-P6c-2** No entity id, provider name or key in the request (I-10); the speaker is named by label.
- **I-P6c-3** Retrieval stays pure: no clock, no randomness, integer and string ordering only (P4-3).
- **I-FGd-1** `consolidate` stays a pure function of items, budget and parameters; integer only; no store access.
- **I-FGd-2** `keepsakes = 0` reproduces the previous plans exactly.
- **I-FGd-3** The store stays within its bound (FG-b2): no change to the host bound or the backstop.
- **I-FGd-4** Every perceived fact stays represented exactly once (FGb-1): only fewer merges, never a new kind of
  decision.

## 6. Acceptance (fixed before measuring)

| Id | Criterion | Mutation that must fail it |
| --- | --- | --- |
| AP6c-1 | Over a store with exchanges with Vera across several days and noise in the last day, `retrieve` with `speaker=Vera, heard=<the trigger>` renders the `with` section first; it holds Vera's first two exchanges and the newest, oldest first, not the trigger; those lines are not repeated under L0; the whole section ≤ 6 000 bytes | the first-words query removed (newest only): the first exchange is missing |
| AP6c-2 | The seat's consult sets `speaker` and `heard` from the trigger (the controller's logged memory holds a `with` line) | — |
| AP6c-3 | The cassette and log re-recorded from Milestone D's scenario with `qwen3.5:4b` through Ollama (local, no key): `replay_log` reproduces every consult; the 3D consult's `recalls` cite a 2D line **and** its words mention the umbrella — both **asserted** | the speaker section removed from `retrieve`: the replay misses (the request differs) |
| AP6c-4 | `milestone_d.rs` `she_remembers_across_the_two_views` (scripted) still passes | — |
| AFGd-1 | `consolidate` with default parameters keeps each counterpart's first 4 exchanges as records, keeps their episode out of its chapter and their chapter from being dropped, and merges everything else as before; an overheard line (two counterparts) is not a keepsake | selecting the newest exchanges instead of the first: the test fails |
| AFGd-2 | `keepsakes = 0` gives the previous plan exactly | — |
| AFGd-3 | `test_milestone_d_100_days.py`: the strict `xfail` removed; at day 100 the context of the visitor's "Hello again." cites the 2D line or Alice's reply; Alice's store ≤ her 20 MiB bound (and the size is printed) | `keepsakes` default set to 0: the citation test fails |
| AFGd-4 | `docs/MODULE_SPEC.md`'s `consolidate` row states the keepsake rule and parameter | — |

## 7. Commit plan

Each commit tracks implementation, validation and review separately (§9).

- **C0 — this design (DRAFT).** Markdown only.
  - [ ] Implementation: the design. [ ] Validation: links resolve. [ ] Review: scope against Q-P7-2/3.
- **C1 — FG-d: keepsakes in `consolidate`.** `forgetting/consolidate.py` (parameter, selection, exemptions);
  `memory/render/conversation.py` (`EXCHANGE_KINDS`); `memory/retention/__init__.py` (re-export);
  `tests/test_retention_policies.py` (AFGd-1, AFGd-2); `docs/MODULE_SPEC.md` row (AFGd-4).
  - [ ] Implementation. [ ] Validation: the policy tests, the retention tests that compare against consolidate
    (`test_retention_plan.py`, `test_assisted_host.py`, `test_lm_summarise.py`), the mutation red.
  - [ ] Review: purity, integer only, the plan always validates, no host change.
- **C2 — P6c: the speaker's section and the rule line.** `memory/retrieve.py`; `controller/seat.py`;
  `controller/prompt.py`; `tests/test_memory_retrieve.py` (AP6c-1, budgets); golden `prompt_request.json`.
  - [ ] Implementation. [ ] Validation: retrieve, AC-10 ceiling, controller choice/parts/seat tests, the mutation red.
  - [ ] Review: no id in the request, bound unchanged, queries bounded.
- **C3 — the re-recorded cassette and the asserted model clause (AP6c-3).** Re-record with Ollama `qwen3.5:4b`;
  replace the cassette and the log; the replay test asserts both clauses.
  - [ ] Implementation. [ ] Validation: replay test, IC-6 swap, provider scan, the mutation red.
  - [ ] Review: no provider term in the log; nothing in the recording edited.
- **C4 — the 100-day clause (AFGd-3).** Remove the strict `xfail`; the query names the speaker.
  - [ ] Implementation. [ ] Validation: the test passes, bound printed, the mutation red.
  - [ ] Review: the bound is the resolved one.
- **C5 — ledger and close-out.** This document's ledger; P7's §7 note pointing here.

## 8. Validation budget and R-LOAD-1

- Targeted tests only (R-LOAD-1): the files named in §7, never a workspace-wide run locally; CI on the exact head is
  the workspace gate. Heavy runs (the real-binary 100-day test, the recording, `milestone_d.rs`) are wrapped in
  `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
- The recording uses the local `qwen3.5:4b` through Ollama on loopback: no key, no paid call. If the model still does
  not use the section, that is recorded as a finding with the prompt tried, not re-rolled silently; the wording may be
  iterated a bounded number of times (≤ 5 recordings), each recorded here.

## 9. Ledger (live)

Order (coordinator, 2026-10-16): this design is written as DRAFT and left in the PR for the primary to freeze; the
implementation follows in the same branch without waiting for the freeze.

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | 9cc22f74 | [x] | [x] links resolve | [x] scope = Q-P7-2 (b) + Q-P7-3 (a) |
| C1 | see `git log` | [x] | [x] policy + interface 38 passed; retention plan/host/assisted/lm-summarise/retrieve 72 passed; size 15 passed; lm-summarise replay 5 passed (digest reset, F-FGd-1); mutation red | [x] pure, integer, no host change; plans validate (every consolidate-comparing test passes) |
| C2 | 5ffe816f | [x] | [x] retrieve 12 passed (AP6c-1, budgets incl. `with`); controller choice/parts/log/consult/seat, retention retrieve, isolation: 73 passed; mutation red; ruff, pyright 0 errors | [x] no id in the header (label from `NameBook`); two bounded reads; sum of budgets 6 000; `memory_view` carries `speaker` |
| C3 | a4005425 | [x] | [x] replay + golden + provider scan 17 passed (both clauses asserted); live scripted `milestone_d` 2 passed; mutation red | [x] recording unedited (copied from the run's output); no provider term in the log (scan) |
| C4 | this head | [x] | [x] 100-day 2 passed; mutation red; AC-10 8 passed; real retention + server + resilience 9 passed | [x] bound from the resolved retention; query = the seat's (speaker named) |
| C5 | this head | [x] | [x] doc | [x] P7 §7 points here; findings recorded |

### Evidence

All local, macOS arm64, 2026-10-16/17, targeted runs only, heavy ones inside `cast-slots/with-slot.sh` (R-LOAD-1). CI on
the exact head is the workspace gate.

| Criterion | Result | Evidence |
| --- | --- | --- |
| AP6c-1 | **PASS**, mutation red | `pytest tests/test_memory_retrieve.py`: 12 passed. Mutation (first-words read `LIMIT 0`): `['[#23]', …] != ['[#10]', '[#11]', …]`, reverted |
| AP6c-2 | **PASS** | the recorded 3D consult's logged memory holds #59 and #62 under `with` (asserted in the replay test) |
| AP6c-3 | **PASS**, mutation red | re-recorded with `qwen3.5:4b` (Ollama, loopback, no key): 3D reply *"Hello again. It seems we are still catching up on our initial meeting about the red umbrella."*, `recalls: ["[#71]", "[#59]", "[#62]"]`; `pytest -s tests/test_milestone_d_replay.py`: 3 passed, `recalls_cite_a_2d_line: true`, `words_mention_the_umbrella: true`. Mutation (the rule line removed): `CassetteMiss … first differs at $.messages[0].text`, reverted. A second mutation (the `with` section removed from `retrieve`) **survives the replay**: `replay_log` re-runs the logged memory, not retrieval (expected; AP6c-1 owns retrieval) |
| AP6c-4 | **PASS** | `cargo test -p mineworld-cli --test milestone_d -- --include-ignored --exact she_remembers_across_the_two_views the_world_runs_with_alice_unplugged`: 2 passed, 17.4 s |
| AFGd-1, AFGd-2 | **PASS**, mutation red | `pytest tests/test_retention_policies.py`: 25 passed. Mutation (`lines[-per_person:]`, the newest): fates differ, reverted |
| AFGd-3 | **PASS**, mutation red | `pytest -s tests/test_milestone_d_100_days.py`: 2 passed, 48.8 s; 124 066 facts absorbed; **817 905 B** ≤ 20 971 520 B (787 316 B before FG-d: keepsakes cost 30 589 B); `line_cited: true`, `reply_cited: true`. Mutation (`keepsakes` default 0): 787 316 B, `line_cited: false`, 1 failed, reverted |
| AFGd-4 | **PASS** | `docs/MODULE_SPEC.md` `consolidate` row |
| Regression | **PASS** | retention plan/host/assisted/lm-summarise/retrieve 72; size 15; lm-summarise replay 5 (+1 skipped); controller choice/parts/log/consult/seat + isolation 73; AC-10 8; retention real + controller server + resilience 9; ruff check/format, pyright (cognition member): 0 errors |

**Base moved (bounded).** P7's branch advanced to `29058e27` (its C6, `test_milestone_d_fallback.py`, the 100-day
fallback bound with the scripted model) while this PR was implemented; it was merged into this branch. On the merged
tree, `pytest -s tests/test_milestone_d_fallback.py`: 1 passed, 41.5 s, 1 101 consults, 28 fallbacks (25 ‰ ≤ 50 ‰).

**Rebased onto main (2026-10-18).** P7 #244 merged into `main`; the PR's base is `main` and `origin/main` (`e51ffc00`, #216 on
top of #243, #241, #244, #248) was merged into this branch without conflict. `main` touched none of this PR's cognition files (only
`tests/binary.py`'s Windows timeout and a `rule-controller` comment). The previous head's CI run (39010580956) was
cancelled, not failed, so it carried no failure evidence. No local cargo run (operator pause on heavy local jobs); CI on
the exact head is the gate. **Freeze pending:** this design is still DRAFT; no scope was added in the rebase, and the
freeze stays with the primary/operator at the milestone.

**D-P6c-3 (bounded; port to S28-a #242's schema 4, "whichever merges second adapts").** S28-a replaced the per-seat
`MemoryStore` with the `MemoryHome` and its `SeatView`, and dropped the stored `involved` column (F-S28a-7). Merged
`origin/main` @ `9884c64c`; two conflicts. No assertion changed; no scope added.

- `memory/retrieve.py`: `_with` takes a `SeatView` and reads the seat's held records as `access ⋈ fact`, with the
  speaker a participant (`fact_who` role 1 or 3), the seat a participant (`involved`, derived as S28-a's ingest
  does) and no third participant: schema 3's `l0`/`l0_who` predicate, with the same `ORDER BY event_key`, the same two
  bounded reads and limits. Every other P6c and FG-d file merged without change (S28-a had already ported their
  callers' types).
- `test_lm_summarise_replay.py`'s digest (F-FGd-1's, now on schema 4): re-pinned `df105650…296020` from scratch CI
  run 39125662718 (branch `scratch/p6c-digest`, the merged tree; deleted after use). As S28-a did at AC-a-13, only
  together with the next item: the `fgc` arm equals the schema-3 oracle.
- S28-a's oracle `tests/golden/s28a_renders.json` (AC-a-10, AC-a-13) was captured from schema-3 code without P6c's
  budgets and FG-d's keepsakes, so its `sections` hashes no longer described what memory renders. Re-captured, as
  `tests/golden/SOURCE.md` prescribes, from the schema-3 code that has both (this PR's pre-merge head `0d57aa09`)
  with S28-a's C1 writer, on scratch CI run 39125660610 (branch `scratch/p6c-golden`; deleted after use); provenance
  recorded in `SOURCE.md`. 450 `sections` hashes and 6 sample texts moved; seeds, points, triggers, passes, forced
  removals and `cafe12_schema3` did not. The schema-4 merged code reproduces it: `test_home_golden.py` passes, which
  is the evidence that the `_with` port and the merge render exactly what the schema-3 PR rendered (for the oracle's
  speaker-less queries; AP6c-1 covers the speaker section).
- Local, light Python only (macOS arm64, no cargo, no binary): `test_memory_retrieve.py` 12, `test_retention_policies.py`
  25, `test_retention_interface.py` 13 (50 passed); `test_home_golden.py`, `test_lm_summarise_replay.py`,
  `test_milestone_d_replay.py` 13 passed, 1 skipped; ruff check/format, pyright strict: 0 errors. AFGd-3 (the 100-day
  bound) and AP6c-4 need the binary and are left to CI on the exact head.
- Execution contract: base and stop condition corrected from `mvp0/pr-s10-p7` to `main` (bounded correction, not
  scope).
- CI on `b8a703ed` (the port): every job PASS.

**D-P6c-4 (bounded; merge of `origin/main` @ `bce21ee8`: #257 recall-at-trigger, #200 S25-a, #232).** One conflict:
`controller/seat.py`'s `RecallQuery` takes #257's `at=max(observation.at, heard_at)` and keeps this PR's `speaker` and
`heard`. One semantic interaction, and the one test line this PR adapts: #257's
`test_controller_seat.py::test_recall_includes_the_lines_said_after_the_observation_the_consult_reads` read the
earlier line (said by the speaker to the seat) from L0 only, and §4.1 shows such a line under `with` and leaves it out
of L0. The test now reads it from `with` or L0; its intent ("the earlier line is in the consult's memory") and its
expected citations (`["2"]`) are unchanged. Its mutation (recall at `observation.at`) is still red: 1 failed,
reverted. Local light Python: `test_controller_seat.py` 18 passed; `test_memory_retrieve.py`, `test_milestone_d_replay.py`,
`test_home_golden.py` passed; ruff, pyright strict: 0 errors. This adaptation is flagged for the coordinator.
CI on `c3c62cc6`: every job PASS (run 39178647194).

**D-P6c-5 (F-FGd-2's ruling implemented; merge of `origin/main` @ `96b41db0`: #220, #233, #199).** The main merge
had no conflict. The host guard (F-FGd-2 above) moves what `lm-summarise` keeps on the FG-c stream, so two oracles
were re-pinned from scratch CI, as at D-P6c-3: the `fgc` arm of `s28a_renders.json` from the schema-3 head
`0d57aa09` with the identical guard (run 39186636448; 7 `fgc` points moved, nothing else), and the replay digest
`a29ed8d5…53251a` on the merged tree (run 39186640715). Scratch branches deleted after use. Local light Python:
`test_assisted_host.py` 10, `test_lm_summarise.py`, `test_retention_host.py`, `test_retention_policies.py`,
`test_lm_summarise_replay.py`, `test_home_golden.py` passed; ruff, pyright strict: 0 errors.

**Freeze header.** The coordinator relayed the operator's freeze approval (2026-10-18). This implementation session
did not rewrite the header: an approval relayed by another agent is not one this session can attest, so the
`DESIGN FROZEN — 2026-10-18 (operator approved in the primary session)` header is left to the primary session.

**Recordings (≤ 5 budgeted, 3 used).** Each with `MINEWORLD_MILESTONE_D_RECORD=<scratch> cargo test -p mineworld-cli
--test milestone_d -- --ignored --exact record_the_milestone_d_cassette`, seeded requests:

1. rule *"…your answer must take it up: refer to what was said there, and cite those tokens in recalls."* — the
   section was first and held #59, #62; the reply cited `[#59]` but said *"Hello again. It's good to hear from you."*
   (umbrella clause fails).
2. rule *"…your words must bring up something they told you there, by name (a thing, a place, a plan), and recalls
   must cite those tokens."* — cited only `[#71]`; the recorder's own assertion failed.
3. rule *"…your answer must take it up: in your words, mention what they told you there, and cite those tokens in
   recalls."* — both clauses pass. Committed; files copied unedited from the run's output directory.

### Deviations and findings

- **F-FGd-1 (bounded, expected).** `test_lm_summarise_replay.py`'s committed digest pins the dump of a 12-day stream
  under `lm-summarise`, whose groups are `consolidate`'s. With keepsakes the dump holds each person's first exchanges
  as records, so the digest moved (`c3624e03…` → `0204e89d…`). The cassette is unchanged and replays without a miss:
  no summarisation request moved, because keepsakes only leave records out of a merge group. The digest is reset with
  that note in the test's docstring.
- **F-FGd-2 (finding, left to FG-c's semantics).** Under `lm-summarise` a model may still forget a keepsake: a group
  whose container is a top-level episode can be turned from `Merge` into `Drop` by a low importance hint (pr-fg-c
  §5.4, "the model may decide which items survive"). The default policy and the no-model fallback keep them.
  **Ruling relayed to this session by the coordinator on 2026-10-18 as the operator's (given in the primary
  session): host-enforced protection.** The freeze header itself is left to the primary session, which holds the
  operator's approval first-hand. Under the LLM summary strategy,
  memories of people the seat talked to directly are protected by the host and never left to the model to delete.
  Implemented in `memory/retention/host.py` (`_protected`, `_forgets`): in the assisted branch, an accepted answer
  that drops an item which is, or holds, a record of an `EXCHANGE_KINDS` event with one counterpart (and that the
  trusted fallback does not drop itself) is overridden; its group keeps the fallback's decisions. It is not counted
  as an invalid answer. The rule is the host's, so it binds every assisted policy, not only `lm-summarise`. Test:
  `test_assisted_host.py::test_a_model_that_drops_a_direct_conversation_is_overridden_by_the_host` (a scripted
  policy drops every top-level group; every keepsake `consolidate` keeps is still a record). Mutation (the filter
  made a no-op): red, `consolidate`'s keepsakes `'1038'`, `'1168'`, `'1349'`, … missing; reverted. Because the rule
  changes what `lm-summarise` keeps on the FG-c cassette's stream (no request moved, no `CassetteMiss`), the
  `fgc` arm of `s28a_renders.json` and the replay digest were re-pinned again from scratch CI (D-P6c-5).
- **D-P6c-1 (bounded).** The seat names the speaker for an `arrived` trigger too (its `speaker` is the person who
  came in), so a returning person is met with what we said before; `heard` is set only for `spoken_to`.
- **D-P6c-2 (commit boundary).** C2 changes the request, so the P7 cassette misses until C3 replaces it; C2 and C3 are
  pushed together, so no pushed head carries the miss.
- **D-FGd-1 (test seam).** AFGd-1 is pinned on hand-built items (`test_retention_policies.py`), like AFGb-8, so the
  rule is checked against literals; the store-level evidence is AFGd-3's 100-day run.

### Execution contract

```text
PROJECT / PR:            P6c + FG-d — memory coherence for the people you talk to
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-p6c-fgd-memory-coherence.md (this file)
RELATED / BINDING DOCS:  pr-s10-p7-milestone-d.md; pr-s10-p6-lm-controller.md; pr-fg-b-retention-host.md;
                         pr-fg-b2-size-trigger.md; pr-fg-c-lm-summarise.md; step-28-shared-memory.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md
IMPLEMENTATION BASE:     main (was origin/mvp0/pr-s10-p7 @ 66121eac until P7 #244 merged; bounded correction, §9)
APPROVED SCOPE:          §4; cognition/lm-controller and docs/MODULE_SPEC.md's consolidate row only
FROZEN INVARIANTS:       §5; no paid API call; no key read, printed or copied; no sed -i or inline Python edits
VALIDATION BUDGET:       §8
ENDPOINT AUTHORITY:      implementation, commits, push of complete heads, PR against main:
                         authorized (coordinator's instruction of 2026-10-16); freeze: the primary session (operator's
                         2026-10-16 merge authority for bounded follow-ups); merge: explicit authorization only
POST-MERGE SYNC OWNER:   the primary session; this session owns this document
STOP CONDITION:          PR "P6c + FG-d: memory coherence for the people you talk to" open against main;
                         DO NOT MERGE
```

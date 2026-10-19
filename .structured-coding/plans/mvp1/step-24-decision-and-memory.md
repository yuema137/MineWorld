# Step 24 — Behaviour extension points: decision models, forgetting, and world evolution

**Lifecycle:** `STEP DESIGN APPROVED 2026-10-12 (operator rulings QS24-2/6/8; others as recommended); per-PR designs follow`.
Step-level design only. No PR here is frozen and nothing here authorizes implementation (`CLAUDE.md`
§3.1). Each PR in §11 is designed, frozen and contracted on its own. The rulings are in §14.1.
*Superseded lifecycle:* `DRAFT — awaiting operator and primary-session review` (2026-10-12).
**Effort:** filed under `mvp1/` because it was opened as an MVP-1 step; **several of its PRs are placed in
MVP-0** by the operator's ruling of 2026-10-12 (§1.2), and §11 says which. The primary session decides
whether the MVP-0 PRs are re-filed under `mvp0/` when they are detailed.
**Author:** a planning session, 2026-10-12. Worktree
`/Users/yuema137/mineworld-worktrees/design-decision-memory`, branch `docs/decision-and-forgetting-design`,
from `origin/main @ 0de1de4` (#172). One writer.
**Decision ids (assigned by the primary session, 2026-10-12):** `ARC-84` … `ARC-88` and `DEP-47`,
`DEP-48`, replacing the placeholders `ARC-S24-a … e` and `DEP-S24-a … b` in this document's order (§13).
The next free ids become `ARC-89` and `DEP-49`.
**PR labels:** placeholders `DL-*` (decision list), `FG-*` (forgetting), `WE-*` (world evolution).
**Binding parents:** `CLAUDE.md` §§1, 2, 4 (rules 1–3, 5, 7, 9–11, 13, 16); `docs/CORE_CONCEPTS.md` §§2, 5,
8, 13, 14, 15; `docs/MODULE_SPEC.md` §§4.1 (`configure:`), 5 (Controller Pack); `docs/ARCHITECTURE.md`
§9; `docs/DECISIONS.md` `ARC-25`, `ARC-27`, `ARC-32`, `ARC-34`, `ARC-57`, `ARC-58`, `ARC-59`, `ARC-61` …
`ARC-65`, `ARC-82`, `DEP-37`, `DEP-46`; `mvp0/step-17-cognition.md` (S10; P6 and PA-10/PA-11);
`mvp0/pr-s10-p4-memory.md`; `mvp0/step-18-interaction-list.md`; `mvp0/pr-s6-save-retention.md` (§6.9
SR-b); `docs/references/PRIOR_ART_2026-10.md`; `docs/references/AI_NPC_RUNTIMES_AUDIT.md`.
**Scope of this file.** It is the only file this session writes. Edits to `CORE_CONCEPTS.md`,
`MODULE_SPEC.md`, `ARCHITECTURE.md`, `DECISIONS.md`, `overall.md` and other lanes' documents are
*proposed* in §13 and applied by the primary session.

---

## 0. The answer in brief

```text
                      what the world IS                    how people and the world DECIDE
                      (authoritative, replayable)          (outside the world; removable)
                      ───────────────────────────          ──────────────────────────────────
physics list          world.yaml systems:                  —
its parameters        configure/<pack>.yaml (IL)           —
decision list         —                                    decisions.yaml: which decision model drives
                                                           each seat; typed parameters per model
forgetting            System-owned person state:           controller-owned memory:
                      a pure rule in the owning System's   a ForgettingPolicy plug-in in the controller,
                      configure section (decay, caps)      under host-enforced hard bounds
world evolution       a System Pack with processes         a director: a Controller on an Organization
                      (seeded, calendar, statistics)       seat, proposing ActionIntents a System may refuse
world log             compacted and archived, never        —
                      forgotten (ARC-25, INV-11)
```

1. **Three behaviour extension points, one pattern.** *Person decisions*, *forgetting* and *world
   evolution* are each a typed interface with a host around it, selected and parameterised by content,
   with bundled implementations that are ordinary packs and none privileged. Each ships **two example
   plug-ins: one without a language model and one with one** (§1.2, ruling 4).
2. **No new pack type is needed.** A decision model is a Controller Pack (`MODULE_SPEC.md` §5 already lists
   "agent policy, memory policy, planning"). A controller-memory forgetting policy is a part of a Controller
   Pack. A System-owned forgetting rule is a parameter, or an `ARC-62` extension-catalog entry, of the
   owning System Pack. A deterministic world-evolution generator is a System Pack. A director is a
   Controller Pack. One ontology question remains: a director needs a **seat on an Organization**, which
   `CORE_CONCEPTS.md` §14 ("a Controller decides what a Person attempts") does not allow today (QS24-2).
3. **The decision list.** A World Pack may carry `decisions.yaml`: a default decision model, per-class
   (`ARC-64`) and per-person assignments, typed parameters per model, and a fallback per assignment. It is
   **never seeded into world state** — control is not a fact about a Person (`INV-1`) — and the operator
   may override it. Absent, every world decides exactly as today, byte for byte.
4. **A no-model motivated decision model.** Recommended: a stateless **utility model** (Infinite-Axis style,
   integer response curves, seeded selection among the best options) over what an observation already
   discloses — agenda, place and door signs, relationship values, conversation recency, time of day,
   weather — with daily rhythms calibrated from the American Time Use Survey's public-domain hour-of-day
   tables. Physiological needs (hunger, energy) come later as a `needs` System Pack, because a need is a
   fact about the world, not a controller's opinion (§6).
5. **Forgetting in three layers.** (a) The **world event log** is never forgotten: it is compressed
   (SR-b) and may be archived in sealed segments, and replay stays byte-exact. (b) **System-owned person
   state** (relationship familiarity and regard, the acquaintance list, the conversation `remembered`
   bound) forgets by deterministic, integer, configurable rules inside the owning System — lazy decay that
   writes no fact, and caps that evict inside the reduction of the fact that caused them. No language model
   may touch this layer. (c) **Controller-owned memory** (P4's per-seat store) forgets through a pluggable
   `ForgettingPolicy` — bundled `decay`, `importance-recency-cap`, `consolidate` (no model) and
   `lm-summarise` (through P5's gateway, cassette-replayed) — while the host enforces the per-seat store
   bound, the 6 000-byte context bound and provenance coverage outside any policy (§8).
6. **World evolution.** A seeded `happenings` System Pack (market days, festivals, statistical events, from
   the calendar and integer draws) is the no-model example; an LLM **director** proposing `propose-happening`
   intents, which `happenings` validates and may refuse, is the model example. The director has no referee
   authority (Concordia's Game Master, inverted: `DEP-46`, PA-10). Removing it leaves a valid world (§9).
7. **Hard bounds hold in every mode** and are enforced by hosts, never by plug-ins: per-seat memory store
   ≤ 1 MiB, ≤ 6 000 bytes of memory per model call (P4's ceiling, also for summarisation calls),
   System-owned person state ≤ 160 KiB per person, and save growth per simulated day ≤ 0.25 MiB for Market
   Town on the baseline profile, ≤ 0.5 MiB with walking (which requires SR-b, QSR-1) (§10).
8. **Placement (operator ruling 2026-10-12).** Forgetting is MVP-0: FG-a … FG-d land before or alongside
   P6. The decision list and the utility model are MVP-0. The `happenings` System Pack is MVP-0; the
   director, the `needs` System Pack and cold archival are MVP-1 (§11).

---

## 1. The requirement

### 1.1 The operator's words (2026-10-12), as relayed

> 1. Worlds must run fully without an LLM. Today every NPC is driven by `cognition/rule-controller`
>    (paced rule controller: agenda from the `schedule` system, then fixed probability bands — greet 20,
>    approach 30, door 12, wander 23, idle; answer 75 % — seeded SplitMix64 draws, pure function of
>    seed/observer/instant). Those constants are arbitrary and give no motivation. The operator wants a
>    proper, selectable set of **decision models**, analogous to Geant4's physics list: a world chooses
>    which Systems are installed (`world.yaml systems:` = physics list), configures them (World
>    Interaction List `configure/<pack>.yaml`, IL-a/b/e), and should also choose **how each person
>    decides** — a "decision list": rule controller (keep, simplest default), a no-LLM motivated model,
>    and the LLM controller (S10 P6, not yet built) — possibly mixed per person. "Enabling many modes also
>    enables very interesting simulations."
> 2. **History and person records must not explode.** Introduce **forgetting**, selectable with or
>    without an LLM.

And the distinction the operator asked to be explicit:

> - The **world event log** is authoritative truth (ARC-25 byte-exact replay, AC-9 causality). It is
>   never "forgotten"; it is only compacted or archived …
> - **Person-side state is subjective and may forget** … Forgetting must stay deterministic and
>   replayable: a forgetting rule is either a pure function inside an owning System (so it is part of the
>   world's facts) or happens in a controller's own store (outside the world, like P4) — state which, per
>   item, and why (single ownership, kernel ignorance, models removable).

### 1.2 Rulings and requirements added the same day (2026-10-12), binding

1. **Ruling reversed: forgetting is in MVP-0.** The earlier ruling "no character-memory forgetting in
   MVP-0, do it in MVP-1" (`pr-s10-p4-memory.md` QP4-11, 2026-10-10) is **reversed**. Reason (operator):
   without it storage, memory and LLM tokens all explode. MVP-0 ships several selectable forgetting modes —
   at least one no-LLM mode (deterministic decay / importance-recency caps / consolidation into summaries)
   and one LLM-assisted mode (summarisation through P5's gateway, cassette-replayed in tests) — plus hard
   bounds that hold in every mode: per-person memory size, per-call LLM context (P4's ≤ 6 KB stays), and
   per-day save growth. The forgetting PRs are placed in MVP-0, before or alongside P6. **P4's "no
   forgetting" note (QP4-11, R-P4-4) is superseded by this ruling** (§13 proposes the dated note).
2. **Forgetting is modular.** Users can write their own forgetting rules, with and without an LLM: (1) a
   typed forgetting/retention interface — inputs: the person's memory items with typed metadata (time,
   importance, citations, kind) and a budget; outputs: keep / drop / merge-into-summary decisions — that a
   pack implements; (2) bundled implementations as ordinary packs, none privileged; (3) selection and
   parameters per world and per person through the same configure / interaction-list mechanism, typed and
   validated; (4) the hard bounds enforced by the host outside any user rule, so a buggy or malicious rule
   cannot exceed them; (5) no-LLM rules are pure functions, LLM rules go through P5's gateway with
   cassettes; (6) where System-owned person state is involved, the rule lives in the owning System's
   configure section, never in a controller. Say which pack type each rule is; prefer existing pack types;
   a new pack type is an operator question. The same modularity applies to decision models.
3. **Three extension points.** Every behaviour extension point — **world evolution**, **person
   decisions**, **forgetting** — is pluggable, and each ships **two example plug-ins: one without an LLM and
   one with an LLM**. World evolution covers changes beyond individual people: events, festivals, prices,
   shop stock, news, weather-driven happenings. **An LLM never mutates world state**: an LLM world-evolution
   plug-in is a director-style controller proposing typed `ActionIntent`s (or scheduled proposals) that
   the owning Systems validate and may refuse, with no referee authority (`DEP-46`, PA-10), and the world
   stays valid with it removed. The no-LLM example is deterministic and seeded. For each extension point:
   the typed interface, the pack type that carries it, selection and configuration, determinism and replay
   (cassettes for LLM), host-enforced hard bounds, and the two examples with acceptance criteria.

### 1.3 Binding constraints restated

- `INV-1` (Person ≠ Controller), `INV-4` (Memory ≠ Biography ≠ World Truth), `INV-6`/`INV-7` (only
  Systems mutate, only what they own), `INV-10` (no action without a System), `INV-11` (the Event Log is
  the single source of truth), `INV-12` (kernel ignorance), `INV-13` (Observations only), `INV-15`
  (traceability).
- `ARC-25`: a world's state is its journal re-executed; replay compares bytes. Controllers are **not**
  re-run by replay: their requests are journaled. A controller's determinism therefore matters for
  *re-running* a world from a seed (`AC-12`) and for kill-and-resume equality (`AC-6`), not for replay.
- `ARC-27`: headless controllers are stateless and seeded, so a killed run continues as if never killed.
- `ARC-59`: a seat's memory is a derivation of what it perceived. I-5: with the store deleted and the seat
  re-joined from a null cursor, the structural store is reproduced exactly.
- `CLAUDE.md` §4 rule 10: headless first, deterministic, LM-independent. No core test reaches a model.
- `CLAUDE.md` §4 rule 16 and `REUSE_POLICY.md`: reuse before reinvention, in both directions (§7).

---

## 2. Audit — today's behaviour and every unbounded-growth item (`origin/main @ 0de1de4`)

### 2.1 How a person decides today

Every seat of `mineworld run` and of a hosted `--town` world is driven by
`PacedRuleController` (`cognition/rule-controller/src/paced.rs`); `--agent SEAT` uses the reactive
`RuleController` (`src/lib.rs`). Both are compiled into the binary; `tools/cli/src/run.rs:125`
constructs one `PacedRuleController::new(request.seed, PACE)` for **all** seats, and
`tools/cli/src/hosted.rs:116,185` does the same for hosted worlds. There is no per-person choice.

`PacedRuleController::decide(&self, observation)` is a pure function of `(seed, observer, instant,
observation)`: `Draw::new` mixes them with SplitMix64's finalizer and each band takes its own draw index
(`paced.rs` "The draws"). The order of consideration and the constants:

| Order | Band | Constant (out of 100) | Source |
| --- | --- | --- | --- |
| 1 | answer an open invitation | accept 70, else decline | `social.rs` `ACCEPTS` |
| 2 | answer an in-window line | 75 | `paced.rs` `ANSWERS` |
| 3 | head for the agenda's place when away from it | 90 | `agenda.rs` `FOLLOWS_AGENDA` |
| 4a | in an activity: leave it | 10 | `social.rs` `LEAVES_ACTIVITY` |
| 4b | in nothing: invite / join | 10 / 8 | `social.rs` `INVITES_BELOW`, `JOINS_BELOW` |
| 5 | attempt a complete affordance (`ARC-34`) | 20 | `offered.rs` `ATTEMPTS_OFFERED` |
| 6 | the walking roll: greet / approach / door / wander / stand | 20 / 30 / 12 / 23 / 15 | `paced.rs` `GREETS_BELOW` = 20, `APPROACHES_BELOW` = 50, `LEAVES_BELOW` = 62, `WANDERS_BELOW` = 85 |
| 6′ | on a street with several doors: pass through | 60 (20 … 80) | `PASSES_THROUGH_BELOW` |

Doors are a draw over `(seed, observer, instant ÷ 6 h)` (`DOOR_WINDOW`). Speech is a fixed set
(`GREETINGS`, six lines; `KINDS` = coffee, chat, walk). The constants are literals "never derived from
what they produce" (`agenda.rs`, `ARC-23` rule 2) — which is honest but carries no motivation: nobody
greets because they are lonely, goes home because they are tired, or avoids someone they dislike. The
only motivation-like input is the `schedule` pack's `Agenda` (`ARC-32`), authored per person as a fixed
`routine:` (for example `worlds/social-cafe/people/alice.yaml`: café 05:30, park 14:00, apartments
18:00). Relationship values are disclosed to their holder (`ARC-28` point 6) and read by no controller.

**What is right about it and must be kept:** it reads only observations and the server's verdicts
(`ENGINEERING_RULES.md` §8); it holds no state, so `AC-6` holds with controllers in the loop; each band's
draw index is independent, so adding a band moves no other band's draws; it is cheap (a 300-day
social-cafe run in about 24 s of CPU without bodies, `step-11-bodies.md` NW-4).

### 2.2 Where history and person state live

```text
world (authoritative, in the save)                       outside the world
──────────────────────────────────                       ─────────────────
journal      every request and input, re-executed        P4 memory store: one SQLite file per
facts        every Event (the history, INV-11)             LM seat, in the operator's cognition
snapshots    checkpoints, pruned by SR (#153)              directory (ARC-59, DEP-37)
components   System-owned state, incl. person state      P5 cassettes and budget ledger
processes    running Processes                           controller in-process state (rule: one
                                                           BTreeMap of answered lines, reactive only)
```

### 2.3 Measured and computed growth

Measured numbers are quoted with their source. **Computed** numbers are derived here from measured ones
and are labelled; none is evidence for an acceptance criterion until a PR measures it.

| Quantity | Value | Source |
| --- | --- | --- |
| Market Town 300-day save before SR | 3 795 013 632 B (3 619 MiB); facts 389.0 MiB, journal 189.3 MiB, snapshots 3 036.1 MiB | `pr-s6-save-retention.md` §4.1 |
| Market Town 300-day save after SR (#153) | 615 399 424 B (586.9 MiB); snapshots 3.75 MiB | `pr-s6-save-retention.md` §14 (E-SR), `overall.md` |
| Market Town revisions per simulated day | ≈ 973 | `pr-s6-save-retention.md` §4.1 |
| Market Town facts / journal rows at 300 days | 375 619 / 290 995; mean fact 926 B, mean journal row 601 B | §4.1; `scripts/baselines.txt` |
| social-cafe facts at 300 days (baseline profile) | 365 330 | `scripts/baselines.txt` `facts-300s` |
| social-cafe facts at 300 days with walking | 837 953 (1.88×); `walk-started` + `walk-ended` 238 018 of them | `step-11-bodies.md` E-NW3 |
| social-cafe facts at 300 days with bodies, in memory | 1 240 225 | `step-11-bodies.md` NW-4 |
| Market Town facts at 300 days with walking | 817 742 (2.18×); 30-day walking save 112.3 MiB | `step-11-bodies.md` E-NW3; `DECISIONS.md` SR note |
| SR-b projection, Market Town 300 days | ≈ 40 MiB (sealed zstd blocks, 20–23×) | `pr-s6-save-retention.md` §4.4, §6.9 |
| P4 store, one seat, 100 days of social-cafe | 19 521 536 B on disk; canonical dump 19.6 MB | `pr-s10-p4-memory.md` §14 (C7, R-P4-4) |
| conversation-history in a snapshot (12 people) | 290 866 B at day 30; 288 242 B at day 300 — plateaued | `pr-s6-save-retention.md` §4.2 |
| acquaintances in a snapshot (12 people) | 93 102 B at day 30 | §4.2 |

**Computed per-day growth:**

```text
Market Town log, baseline profile     (389.0 + 189.3) MiB ÷ 300 d   ≈ 1.93 MiB per simulated day
  of which facts                      1 252 facts/d × 926 B          ≈ 1.10 MB/d (+ index)
  of which journal                    973 rows/d × 601 B             ≈ 0.58 MB/d
Market Town log, with walking         × ≈ 2.2 (facts)                 ≈ 3.5 MiB/d (estimate)
the same after SR-b                   40 MiB ÷ 300 d                  ≈ 0.13 MiB/d (baseline);
                                                                       ≈ 0.3 MiB/d with walking (estimate)
social-cafe facts                     365 330 ÷ 300                   ≈ 1 218 facts/d (baseline)
                                      837 953 ÷ 300                   ≈ 2 793 facts/d (walking)
                                      1 240 225 ÷ 300                 ≈ 4 134 facts/d (bodies)
P4 store                              19.5 MB ÷ 100 d                 ≈ 195 KB per seat per simulated day
                                      × 300 d                         ≈ 58.6 MB per seat
                                      × 11 seats (social-cafe)        ≈ 644 MB per 300-day world
acquaintances                         93 102 B ÷ (12 × 11) entries    ≈ 705 B per directed entry
                                      N people → N(N−1) entries       1 000 people ≈ 704 MB per snapshot
conversation-history                  288 242 B ÷ 12                  ≈ 24 KB per person (cap 32 lines)
```

The P4 store therefore grows as fast as a third of a whole Market Town log, per seat. It is the item
that explodes first once P6 puts every NPC on an LM seat — together with the tokens it would feed a
model if retrieval budgets were ever relaxed.

### 2.4 Every item that grows, classified

Kernel and Systems cannot create or destroy entities at run time (`WorldView`, `kernel/src/view.rs`), so
everything keyed by person or by pair is bounded by the authored population; the snapshot sections
plateau (§2.3). The items, with the layer they belong to and the reason (§3.2):

| # | Item (file) | Owner | Grows with | Bound today | Layer | Forget? |
| --- | --- | --- | --- | --- | --- | --- |
| G1 | `facts` table (`persistence/src/sqlite.rs`) | persistence | every Event | none; insert-only (`ARC-81` rule 1) | world log | **Never.** Compress (SR-b), archive (FG-e) |
| G2 | `journal` table (same) | persistence | every revision | none | world log | **Never.** As G1 |
| G3 | anchor snapshots (`persistence/src/world.rs` `ANCHOR_EVERY`) | persistence | one per 4 096 revisions, zstd | ≈ 52 KB each | world log (checkpoint) | Prunable: checkpoints are not authority (`ARC-25`). FG-e may thin old anchors once archived segments carry their own |
| G4 | whole-log reads: backfill `saved_facts` (`tools/cli/src/history.rs`), `biography`, `inspect`, `Fingerprint::of_save`, `journal_after` | server / CLI | log length, in memory, per read | none | tooling | Not forgetting: route as finding **F-S24-1** (stream or page; required before archival) |
| G5 | P4 `l0`, `l0_who`, FTS `l0_text` (`memory/store.py`) | LM Controller Pack | every perceived fact | none (no `DELETE` in `memory/` or `compress/`) | controller memory | **Yes** — `ForgettingPolicy` (FG-b, FG-c) |
| G6 | P4 `episodes`, `episode_who` (L1) | LM Controller Pack | several per seat per day | none | controller memory | **Yes** |
| G7 | P4 `chapters` (L2); `chapters()` reads every row per query | LM Controller Pack | one per simulated week | none | controller memory | **Yes** (and retrieval must stop reading all rows) |
| G8 | P4 `stable` (L3) counters `times_met`, `exchanges` | LM Controller Pack | rows capped by population; integers unbounded | population | controller memory | Rows follow the acquaintance cap; counters saturate |
| G9 | P5 `SqliteLedger.budget_ledger` (`budget.py`) | LM Controller Pack | one row per model call | none | controller bookkeeping | Not memory: prune rows older than the longest budget window (one simulated day) — pure bookkeeping, FG-b |
| G10 | cassettes (`record.py`) | tests / recording | per call | none | test artefact | No; a recording is evidence |
| G11 | `Acquaintances.known` (`systems/relationships/src/component.rs`); "Values never decay" (`lib.rs`) | `relationships` System | first meeting of a directed pair | population²; values clamped | System-owned person state | **Yes** — lazy decay and a cap, in the System (FG-a) |
| G12 | `knows` relation edges (`kernel/src/relations.rs`) | `relationships` (declared), kernel (stored) | first meeting | population² | System-owned (relation) | Follows G11: unrelated when an acquaintance is evicted |
| G13 | `ConversationHistory.heard` (`systems/conversation/src/component.rs`) | `conversation` System | every `spoke` | **capped**: `remembered` 32, 1 … 64 (IL-e) | System-owned person state | Already a FIFO cap. Kept; optionally age-out (FG-a) |
| G14 | `Invitations.pending`, `ActivityState.took_part` (`systems/group-activity`) | `group-activity` System | invitations, joins | pruned on expiry; process removed on end | System-owned | Bounded; nothing to add |
| G15 | `RuleController.answered` (`cognition/rule-controller/src/lib.rs`) | rule Controller Pack | each reply | population; rebuilt at binding | controller | Bounded; nothing to add |
| G16 | `run`'s `Tally` | CLI | per 30-day bucket | small | tooling | No |

Schedule (`Routine`, `Agenda`), calendar, weather, employment, presence, movement, bodies, naming, item,
economy and inventory hold fixed-size state per entity or per day (audit, §2.4 sources in the PR
designs). **There is no agenda history**: an `Agenda` is today's, replaced at each boundary. Inventory
counts and wallet balances are bounded integers whose value, not size, changes.

### 2.5 Findings

- **F-S24-1 (server / CLI lane).** Backfill (`SavedHistory::perceived`) and several CLI commands load the
  whole fact log into one `Vec` per read. At 1 M+ facts this is the first memory wall a long world hits,
  and it blocks archival (a reader that must hold every fact cannot leave old segments cold). Route to the
  owning lane; FG-e depends on it.
- **F-S24-2 (P4).** `chapters()` and `stable_facts()` read whole tables per retrieval. Bounded output, but
  cost grows with age. FG-b fixes it as part of the retention host.
- **F-S24-3 (relationships).** "Values never decay … a long world's social graph saturates" is recorded
  as a known gap (`ARC-28`, step-09 QB-2). FG-a closes it as configuration.
- **F-S24-4 (rule controller).** Relationship values are disclosed to their holder and read by no
  controller; agendas are authored, not calibrated. §6 addresses both.

---

## 3. One pattern for three extension points

### 3.1 The pattern

Each behaviour extension point has the same five parts. The physics list already has the first four
(`systems:`, `configure/<pack>.yaml`, the SDK's typed `Parameters` with bounds, `ARC-62` catalogs); this
step gives decisions and controller memory the same shape and adds the fifth, host-enforced bounds, where
a plug-in could otherwise exceed a budget.

```text
1 interface     a typed trait (Rust) or Protocol (Python) — inputs and outputs are named types,
                never maps of strings (CLAUDE.md §4 rule 7)
2 carrier       an existing pack type: System Pack or Controller Pack (no new pack type)
3 selection     content: which implementation, per world, per class (ARC-64), per person; refused by
                name when unknown; precedence person > class > default; ambiguity refused at load
4 parameters    typed by the implementation, each with a bound and a default, decoded with line and
                column (the IL-a mechanism, reused)
5 host          code around the plug-in that enforces the hard bounds and the invariants whatever the
                plug-in returns; a plug-in cannot reach past it
```

| Extension point | Interface | Carrier | Selected in | Host | No-LLM example | LLM example |
| --- | --- | --- | --- | --- | --- | --- |
| Person decisions | `DecisionModel` (Rust, in-process) · P6 `Controller` protocol (Python, out-of-process) | Controller Pack | `decisions.yaml` (World Pack; operator override) | the seat host: `run`, `--town`, the cognition process | `motivated` utility model (DL-b); `rule` stays | P6 `LMController` |
| Forgetting, System-owned | the owning System's `Parameters`; optionally an `ARC-62` catalog kind (pure curve) | System Pack | `configure/<pack>.yaml` (the World Interaction List) | the owning System's reduction (clamps, monotone, cap) | `relationships` lazy decay + cap (FG-a) | **none, by architecture** — an LLM never mutates world state (§8.3) |
| Forgetting, controller-owned | `ForgettingPolicy` (Python) | part of a Controller Pack | `decisions.yaml` `memory:` | the retention host in the memory store | `consolidate`, `decay`, `importance-recency-cap` (FG-b) | `lm-summarise` (FG-c) |
| World evolution | the System contract (processes, reactions); `HappeningKind` catalog · director = `DecisionModel`/`Controller` | System Pack · Controller Pack | `systems:` + `configure/happenings.yaml` · `decisions.yaml` (director seat) | `happenings`' validation and rate caps | `happenings` seeded generator (WE-a) | director (WE-b) |

### 3.2 Which layer each forgetting rule belongs to, and why

The rule that decides the layer is ownership, not convenience:

1. **If the state is a Component, the owning System forgets it** (`INV-7`, single ownership). The rule is a
   pure function of the System's own state, its configure section and world time, so it is reproduced by
   replay exactly like any other reduction (`ARC-25`). A controller cannot forget world state for the world
   — it can only stop reading it.
2. **If the state is a controller's own store, the controller forgets it** (`INV-4`, `INV-13`). The world
   never reads it (step-17 I-6), so forgetting there changes no byte of any save.
3. **The kernel forgets nothing** (`INV-12`). It does not know what familiarity is; "decay" is domain
   semantics.
4. **The world log is not person state** and is never forgotten (`INV-11`). Compressing it and moving old
   ranges to cold storage changes where bytes live, not which facts exist.
5. **Models are removable.** No layer may require a model: every LLM-assisted policy names a no-model
   fallback, and the System layer admits no model at all.

---

## 4. The decision list

### 4.1 The concept

Geant4 separates *what physics exists* (a physics list of processes) from *its production cuts and
parameters*. MineWorld already has both halves for the world: `systems:` and `configure/<pack>.yaml`. The
decision list is the missing third: **which decision model drives each seat**. It is deliberately not part
of the world's state:

- `INV-1`: a Person exists independently of whatever decides its actions; control may change at runtime
  without altering the Person. Seeding the decision list into the world would make a controller change a
  world fact and trip the configure drift check at resume (`ARC-61`).
- `world.yaml`'s own comment on `seats:` already says "which Person is driven by what is a fact about this
  world's configuration rather than about the person" (`MODULE_SPEC.md` §5, `AC-5`).
- Step-17 §12 proposes "a World Pack binds no backend" (`AC-4`). The decision list names **models**, never
  providers, model names, URLs or keys; the LM model's backend binding stays in the operator's
  `cognition.toml`.

So the decision list is **World Pack content read by hosts** — the author's default casting of who decides
how — overridable by the operator, never seeded, never in the save's authoritative tables.

### 4.2 The file

```yaml
# world.yaml
decisions: decisions.yaml          # optional; absent = every seat as today (`rule`)

# decisions.yaml — read by hosts (run, server --town, the cognition process); never seeded
models:                            # the models this world uses, each a Controller Pack's model id
  rule: {}                         # PacedRuleController, today's constants; no parameters
  motivated:
    parameters:                    # typed by the model (DL-b §6.4); unknown keys refused with line/column
      social_need_hours: 3
      curves: atus-2003-2023       # a named parameter set the model's pack ships
  lm:
    fallback: motivated            # used when the model's process is absent or its budget is exhausted
  director:                        # WE-b; only valid on an Organization seat (QS24-2)
    fallback: none
default: rule
assign:                            # seat > class > default; two entries for one seat at one level refused
  - { class: staff,  model: motivated }        # ARC-64 classes, configure/classes.yaml
  - { seat: alice,   model: lm }
  - { seat: council, model: director }         # MVP-1, only if QS24-2 admits Organization seats
memory:                            # controller-owned memory (§8.4); applies to seats whose model keeps one
  default: { policy: consolidate }
  assign:
    - { seat: alice, policy: lm-summarise, fallback: consolidate,
        parameters: { calls_per_sim_day: 4 } }
```

Rules:

1. **Typed and refused by name.** `models.<id>` must be a model a registered Controller Pack provides
   (§5.2); `parameters` decode into that model's type with bounds; an unknown model, class, person or key
   is refused at load with line and column, exactly as `configure/` is (`ARC-61`).
2. **Seats only.** An assignment names a seat (`seats:`); a non-seat person is refused, because nothing
   drives it.
3. **Fallback chain.** Every model that can be absent at run time (out-of-process, budgeted) names a
   fallback; the chain ends in an in-process model or `none` (the seat stays idle). Chains ≤ 4, no cycles.
4. **Operator override.** `mineworld run … --decisions FILE` and `mineworld server … --decisions FILE`
   replace the pack's file; `--seat-model SEAT=MODEL` overrides one seat. The operator's file is validated
   against the same world.
5. **Absent means today.** No `decisions:` key and no flag → every seat `rule`, every byte as today
   (acceptance AC-DL-1).
6. **Reproducibility without seeding.** `run` prints the resolved decision list's digest in its header
   line beside the seed, and `run`'s fingerprint tests include it. A saved world does not store it
   (QS24-1 discusses recording it as non-authoritative host metadata).

### 4.3 How it composes with Systems and the Interaction List

A decision model reads an `Observation` and returns at most one `ActionRequest`. Everything it may attempt
is an affordance an installed System offered, filtered by the world's Interaction List (`ARC-63`), and every
request is revalidated at dispatch. So:

- removing a System removes options from every model at once (`INV-10`); a model never needs editing;
- a world's Interaction List forbidding nobles from talking to commoners binds the `motivated` model and
  the `lm` model identically — it is the server's verdict, read, never computed (`ENGINEERING_RULES.md`
  §8);
- decision parameters cannot grant anything: they change *preferences among offered options*, never what
  is offered.

### 4.4 Per-person mixing, and what it enables

A world may mix models freely: `rule` for background townsfolk, `motivated` for shopkeepers, `lm` for
Alice, a human for the visitor. Mixed worlds are where the operator's "very interesting simulations" come
from: the same town with every seat `motivated` against every seat `rule` is a controlled experiment over
one physics list and one seed; swapping one person's model is an intervention whose effect is visible in
the history.

### 4.5 Determinism

| Host | Model kind | Guarantee |
| --- | --- | --- |
| `mineworld run` | in-process (`rule`, `motivated`, user Rust models) | each model is a pure function of `(seed, model parameters, observation)` (`ARC-27` extended); the same pack, seed, decision list and code give the same history; kill-and-resume equals uninterrupted (`AC-6`) |
| `mineworld run` | out-of-process (`lm`, `director`) | **not hosted by `run` in MVP-0**: the seat uses its fallback, and the header says so (QS24-12) |
| hosted server | any | requests are journaled; replay re-executes the journal and never calls a controller (`ARC-25`). Re-running an LM seat reproduces its decisions only under cassette replay (`ARC-58`) |

---

## 5. Decision models as plug-ins

### 5.1 The interface (Rust, in-process)

A new small crate `cognition/decision` (`mineworld-decision`) holds the trait every in-process model
implements. It depends on `mineworld-contracts` only, like the rule controller's interface does today.

```rust
/// A decision model: what a seat attempts, as a pure function of what it observes.
pub trait DecisionModel: Send + Sync {
    /// The model's typed parameters: bounded fields with defaults, decoded from `decisions.yaml`.
    type Parameters: Parameters;          // the IL-a `parameters!` machinery, reused
    /// Stable id named in `decisions.yaml` (e.g. "motivated").
    const ID: &'static str;
    /// The consult: at most one request. No clock, no network, no world handle, `&self`.
    fn decide(&self, p: &Self::Parameters, seed: Seed, obs: &Observation<Value>) -> Option<ActionRequest>;
    /// The next stride of a disclosed walk, between consults (as `PacedRuleController::step`).
    fn step(&self, p: &Self::Parameters, obs: &Observation<Value>) -> Option<ActionRequest>;
}
```

- **Pure by signature.** `&self`, no interior mutability (a `clippy` disallowed-types entry on `Cell`,
  `RefCell`, `Mutex` in model crates), no clock or RNG beyond the provided `Seed` and the counter-based
  `Draw` (the `paced.rs` scheme, moved here so every model shares one well-tested mixer). This is what
  keeps `AC-6` true for every user model without persisting controller state.
- **Draw indices** are per model; a model's documentation lists its indices as `paced.rs` does.
- `RuleController` (reactive, stateful by design for F-13) stays outside this trait; it is the `--agent`
  controller.

### 5.2 Registration

As for System Packs (`systems/installed`, `ARC-33`): `cognition/installed` lists every in-process model this
build provides, one line per model, and the host resolves `decisions.yaml` model ids through it. Each model
crate is a Controller Pack with its own package identity (`ARC-53`, `packages` type `controller-pack`).
Installing a user model is two lines and a rebuild, the same as installing a System Pack — the
`install modules → compose world → configure → run` path, with no edit to any existing model, System,
contract or host.

### 5.3 Out-of-process models

A Python (or any-language) model is a Controller Pack running as a protocol client (step-17 §3.2): it joins
a seat with the same frames as a client. The decision list's entry for it carries only `fallback` and
model-specific parameters; the seat's host drives the fallback while the external process is absent and
yields the seat on `take_over` (S11-B). P6's `LMController` is the first such model; WE-b's director the
second.

---

## 6. The no-LLM motivated model

### 6.1 Requirements on it

Server-authoritative (reads observations only), seeded and integer-deterministic, stateless across consults,
cheap at 12–1 000 seats, explainable (a reviewer can say why Alice went to the park), calibratable from
real data, and composable with whatever Systems a world installs (unknown packs appear as complete
affordances, `ARC-34`).

### 6.2 Candidates compared

Sources are listed in §15 with access dates.

| Candidate | What it is | Fit for MineWorld | Verdict |
| --- | --- | --- | --- |
| **Fixed bands** (today) | ordered probability bands | deterministic and cheap; no motivation; constants arbitrary (§2.1) | **Keep** as `rule`, the simplest default |
| **Utility / needs AI** — The Sims' motives and object advertisements; Dave Mark and Kevin Dill's utility theory and the Infinite Axis Utility System (IAUS) | every option is scored by multiplying considerations, each an input mapped through a response curve; choose the best or a weighted draw among the best | stateless per consult; options come straight from affordances (an affordance *is* an advertisement); integer curves are deterministic; explainable (the score table is the explanation); calibratable (curves from data). Needs need state — which belongs in a System (§6.5) | **Recommended** |
| **Behaviour trees** | a tree of sequence/selector/condition nodes ticked each frame | good for authored reactive behaviour; running nodes keep state between ticks, which a stateless consult must re-derive; motivation is not a concept | Not chosen: the tree would encode today's bands with more structure, not motivation |
| **GOAP** (Orkin, F.E.A.R.) | A* over actions with preconditions/effects to reach a goal | strong for multi-step plans; needs a symbolic world model of every pack's effects, which controllers do not have (they see affordances, not effects) and which would duplicate System rules in a controller (`ENGINEERING_RULES.md` §9) | Not now; revisit for multi-step goals once Systems publish effect declarations |
| **HTN** (Humphreys, Game AI Pro) | decompose compound tasks into primitive tasks by authored methods | good for daily routines as plans; overlaps the `schedule` System, which already owns the day | Not now; a routine *generator* (§6.6) covers the daily structure |
| **Markov chain / FSM over activities** | transition probabilities between activities per hour | trivially calibrated from time-use diaries; but transitions need a remembered current activity — a stateless model reads it from disclosed state (agenda, participation) | **Used inside** the routine generator (§6.6), not as the decision model |
| **Agent-based modelling frameworks** (Mesa, NetLogo) | Python / Logo frameworks with a model-owned scheduler and RNG | the right practice lessons (one seeded RNG, explicit scheduling, repeatability needs same seed and version); adopting them would put the world inside their execution model, which `REUSE_POLICY.md` forbids | **Reference only** |
| **Rust libraries**: `big-brain` (utility AI), `bonsai-bt` (BT), `dogoap` (GOAP) | crates | `big-brain` is built on Bevy ECS (forces Bevy's execution model; 28 breaking releases); `bonsai-bt` is standalone but last active early 2024 and solves the BT case we do not choose; `dogoap`'s core is standalone GOAP, not chosen | **Reject** for the utility model (`DEP-47`): the IAUS core is ~300 lines of integer arithmetic over our own types; adopting a framework would cost more than it saves, and the record says so |

### 6.3 The recommended model: `motivated`

```text
consult (every PACE, stateless)
  1  hard precedence, as today: answer an open invitation; answer an in-window line
  2  options     = { head for agenda place, stay, greet P, approach P, invite P, join P, leave
                     activity, walk through door D, attempt complete affordance A, wander }
                   — only options the observation offers as available
  3  score(o)    = Π_i curve_i(input_i(o, observation))       integers in 0 … 1 000, product
                   rescaled with Mark's compensation factor, all in integer arithmetic
  4  choose      = a seeded weighted draw among options within 10 % of the best score
                   (Mark's "weighted random from the top band"), Draw index per model
  5  commitment  = windows, as DOOR_WINDOW today: a chosen door or destination holds for a window
                   derived from (seed, observer, instant ÷ window), so the person does not dither
```

**Inputs** — every one already disclosed to the observer today:

| Input | From | Example consideration |
| --- | --- | --- |
| time of day, day of week | `observation.at()` (+ calendar's date where installed) | sociability by hour (ATUS curve) |
| agenda place and label | `schedule`'s `Agenda` (`ARC-32`) | pull toward the agenda place, stronger late in the slot |
| where I am, door signs | perception; `ARC-82` (a doorway discloses the destination's tags) | pull toward a `cafe`-tagged place in the morning |
| who is here; my relationship values | perception; `relationships` (holder-only, `ARC-28` point 6) | greet/approach weighted by familiarity and regard; avoid negative regard |
| how long since I last talked | `ConversationHistory` timestamps | a *social* pseudo-need rising with hours since the last exchange (bounded by the 32-line history) |
| part of an activity | `group-activity` `Participation` | leave less likely early in an activity |
| weather | `weather`'s disclosed state, where installed | outdoor places scored down in rain |
| needs (MVP-1) | a `needs` System's holder-only component (§6.5) | hunger pulls toward food affordances |

Every input is optional: an absent System makes its considerations neutral (1 000), so `motivated` runs in
any world and decides sensibly in a world with only `presence` and `movement`.

### 6.4 Parameters

Typed, bounded, named parameter sets shipped by the model's pack (`curves: atus-2003-2023`) plus per-world
overrides: each curve is a piecewise-linear table of ≤ 24 integer points (input → 0 … 1 000), the top-band
width (‰), and window lengths. All integer; no float anywhere (the workspace has none).

### 6.5 Needs are world state: the `needs` System Pack (MVP-1)

The Sims' motives decay over time and are restored by using objects. In MineWorld a need is a fact about a
person (Alice *is* hungry) that several Controllers and Systems must agree on, so it is **System-owned**: a
`needs` System Pack owns a holder-only `Needs` component per person, decays it lazily (value as a pure
function of last-restored value and elapsed world time — no Process, no per-tick fact), and restores it by
reacting to other packs' facts per its configure section (`items-consumed` of a `food` kind restores
hunger; a night at the `home`-tagged place restores energy). It owns nothing else and moves nobody.
Consumption's own comment ("this is the interaction, not a need: nobody is hungry", step-10 QS-10) marks the
gap this fills. Placed in MVP-1 (QS24-8): `motivated` is useful without it.

*Dated note, 2026-10-18 (operator ruling, primary session; DOC-S35):* **superseded.** Needs (hunger and
tiredness) and `sleep` moved into the MVP-0 release and are designed in
[`../mvp0/step-35-needs-and-sleep.md`](../mvp0/step-35-needs-and-sleep.md) (`DECISIONS.md` `ARC-283` …
`ARC-286`, `DEP-140`). This section is the starting point that step keeps where it does not refine it. Two
refinements: tiredness is restored by a separate `sleep` System Pack's facts, not by "a night at the
`home`-tagged place"; and hunger is restored by the eaten item's own `nourishment:` section, not by its `food`
category. QS24-8 is superseded with it.

### 6.6 Calibration: real daily rhythms

- **Source.** The American Time Use Survey (BLS) publishes, among its tables, *A-3: percent of the
  population engaging in selected activities by time of day* (sleeping, working, eating and drinking,
  socialising and communicating, leisure, travel), plus hourly work tables (A-4, A-5). BLS publications are
  in the public domain (BLS copyright page); credit is requested. These are aggregate tables, so they may be
  committed as derived integer tables with a credit line.
- **Not used:** IPUMS ATUS-X extracts (registration; redistribution needs approval), MTUS (registration;
  redistribution terms unstated), HETUS microdata (research-only access). HETUS *aggregate* tables on
  Eurobase are reusable with acknowledgement (Eurostat, CC BY 4.0 for content) and may calibrate a European
  world later.
- **How.** An offline tool `routine-fit` (the `weather-fetch fit` pattern, `ARC-68`) reads a downloaded
  table and writes (a) the `motivated` model's named curve set (sociability and home-pull by hour) and (b)
  optional `routine:` sections for a population, drawn from an hour-by-hour activity Markov chain fitted to
  the table with a stated seed. Its output is committed content; the run never reads the source.
- **Acceptance by rhythm, not by charm** (AC-DL-4): over 30 simulated days of a calibrated world, the share
  of person-time at a `home`-tagged place between 00:00 and 05:00, at an agenda `work` place during work
  slots, and in conversation per hour, each lies within a stated tolerance of the fitted curve — numbers
  fixed before the first run (`ARC-23`).
- **Relationships.** Dunbar's layers (≈ 5, 15, 50, 150) bound a person's active acquaintances (FG-a's
  default cap 150); Roberts & Dunbar (2011) observed friendship closeness decaying faster than kin closeness
  without contact, which FG-a's per-class decay parameters can express. Exact decay rates were not
  verified in this session (§15) and are not used as defaults.

### 6.7 Two example plug-ins for person decisions

| Example | Kind | Acceptance |
| --- | --- | --- |
| `motivated` (DL-b) | no-LLM, in-process Controller Pack | AC-DL-2 (same seed → same fingerprint, twice and across kill/resume, on three platforms); AC-DL-3 (remove the pack → `decisions.yaml` naming it is refused by name; with the assignment removed the world runs); AC-DL-4 (rhythm within tolerance); every request it submits is one an observation offered (a property test over 30 days: zero `ActionUnavailable` caused by a model inventing an action) |
| P6 `LMController` | LLM, out-of-process Controller Pack | step-17 IC-3, I-5, I-6, I-14, I-15; plus AC-DL-5: with the cognition process killed, the seat runs its `fallback` within one consult and the world stays valid; under cassette replay the same seat's requests are identical |

---

## 7. Reuse, in both directions

| Need | Mature option | Decision |
| --- | --- | --- |
| Utility AI core | `big-brain` (Bevy), commercial middleware | **Build our own** (`DEP-47`): Bevy's ECS would own the execution model; the core is small integer arithmetic; curves as data follow Dave Mark's published method |
| Behaviour trees / GOAP | `bonsai-bt`, `dogoap` | Not needed now (§6.2); recorded so a later PR does not re-ask |
| Daily rhythms | ATUS A-3 (public domain) | **Adopt the data**, behind an offline fit tool (`DEP-48`) |
| Memory retention scoring | Generative Agents (recency × importance × relevance), Ebbinghaus/power-law forgetting curves (Murre & Dros 2015), MemGPT's tiered context | **Adopt the methods**, not the code: none is a library with deterministic replay; P4's store stays (`DEP-37`) |
| Sampling under a budget | reservoir sampling (Knuth's Algorithm R, Vitter 1985), weighted reservoir A-Res (Efraimidis & Spirakis 2006), priority sampling (Duffield, Lund & Thorup 2007) | **Adopt the algorithm** for `importance-recency-cap`'s tie region, with the "random" key replaced by a seeded hash of the Event ID so it is a pure function |
| Log archival | EventStoreDB scavenging (destructive), Marten archiving (non-destructive flag/partition), Kafka tiered storage (segments uploaded, then deleted locally) | **Adopt the Marten/Kafka-tiered shape** — sealed segments moved, never deleted, readable on demand; **reject scavenging and key-compaction**, both of which destroy history |
| LLM summarisation | Concordia components, LangChain memory summarisers | **No**: P5's gateway, budgets and cassettes already exist (`ARC-56` … `ARC-58`); Concordia is reference-only (`DEP-46`) |

---

## 8. Forgetting

### 8.1 Layer 1 — the world event log: compaction and archive only

The log is never forgotten. Three levers, in order:

1. **SR (merged, #153).** Snapshots pruned to genesis + anchors + newest two, zstd frames: 3 619 → 587 MiB
   at 300 days.
2. **SR-b (designed, `pr-s6-save-retention.md` §6.9; awaiting QSR-1).** Journal and fact rows older than
   the newest anchor are sealed into zstd blocks of 4 096 revisions; readers decode on demand; replay
   compares decoded bytes exactly. ≈ 40 MiB at 300 days; ≈ 0.13 MiB per simulated day (baseline profile).
   **This step needs SR-b in MVP-0**, because without it no per-day save bound below ≈ 1.9 MiB holds
   (QS24-5).
3. **FG-e cold archive (MVP-1).** Sealed blocks older than a configured age move to segment files
   `archive/<first>-<last>.zst` beside the save, each listed in a manifest with its revision range and a
   SHA-256 of its bytes. What it needs, measured against the current code:
   - **Resume** needs only the newest anchor and the live tail (`verify_from`, SR C4), so a resumed world
     never opens a cold segment.
   - **Full replay and `verify`** open every segment in order; a missing or altered segment is refused by
     name with its range (never skipped).
   - **Readers of old facts** (`biography`, `inspect`, perceived backfill) must stream or page —
     F-S24-1 — or archival merely moves the memory wall.
   - **Old anchors** may be thinned once each segment starts at an anchor (a segment then carries its own
     restart point).
   - **Byte-exactness** is unaffected: segments hold the same sealed blocks, so `ARC-25`'s comparison is
     on the same decoded bytes.
4. **Volume at the source** (not forgetting): `walk-started` + `walk-ended` are 238 018 of social-cafe's
   837 953 walking facts. Whether walking facts can be coarser is movement's design question, routed, not
   decided here (R-S24-6).

### 8.2 Layer 2 — System-owned person state: deterministic rules in the owning System

**Interface.** The owning System's `InteractionSection::Parameters` (IL-b), with bounds and defaults; for a
rule *shape* a user wants to supply, an `ARC-62` extension catalog the System opens (bodies' catalog is the
precedent). The catalog entry type for decay curves:

```rust
/// A pure retention curve: the value after `elapsed` with no reinforcement. Registered in the owning
/// System's catalog (ARC-62); named in configure/<pack>.yaml.
pub trait DecayCurve: Send + Sync {
    type Parameters: Parameters;
    const ID: &'static str;
    fn decayed(&self, p: &Self::Parameters, value: i32, floor: i32, elapsed: SimDuration) -> i32;
}
```

**The host is the owning System.** Whatever a curve returns, the System clamps it to `[floor, value]`
(forgetting only ever lowers a value toward its floor, never raises it, never crosses it), and the caps are
the System's, not the curve's.

**FG-a, `relationships`:**

| Parameter (`configure/relationships.yaml`) | Bound | Default | Meaning |
| --- | --- | --- | --- |
| `familiarity_decay` | catalog id | `none` | `none` · `linear` (per-day decrement) · `half-life` (integer half-life in days, Ebbinghaus-style exponential approximated by table) · `power` (Murre & Dros' power-law shape) |
| `familiarity_half_life_days` | 1 … 3 650 | — | when `half-life` |
| `familiarity_floor` | 0 … 1 000 | 0 | never decays below |
| `regard_decay` | as above | `none` | regard decays toward 0 from either side |
| `known_at_most` | 5 … 1 500 | 150 (Dunbar) | acquaintances kept per person |

Scoped by class like every IL parameter (`{ actor: kin, familiarity_half_life_days: 365 }`), so friends and
kin can decay differently (Roberts & Dunbar 2011).

**Lazy, so it writes no fact and starts no Process.** A stored acquaintance keeps `(value, last_contact)`.
Its *current* value is `decayed(value, floor, now − last_contact)`, computed where it is read: when
disclosed in an observation, when a reaction reads it, when a level is derived. A contact first computes the
current value, then adds the increment, then stores `(new value, now)`. The stored state changes only inside
reductions that already happen, so decay adds **zero** facts, zero journal rows and zero save growth, and it
is replayed exactly because it is a pure function of stored state and world time.

**Levels.** A level change caused by decay is observed lazily; if a world wants a biographical
`relationship-faded` fact, `relationships` emits it at the next reduction that touches the pair (traceable to
that fact, `INV-15`), never on a timer — which keeps "no per-tick fact" true.

**The cap.** When a reduction would add an acquaintance beyond `known_at_most`, the System evicts the entry
with the lowest current familiarity (ties: oldest `last_contact`, then lowest `EntityId`), unrelates `knows`,
and emits `acquaintance-forgotten` (participants audience, biography configurable) inside the same
reduction. Bounded by construction: `known.len() ≤ known_at_most`.

**Byte identity.** Defaults (`none`, cap 150) change nothing in any reference world (12 people, no decay):
AC-FG-6 checks `scripts/baselines.txt` is unchanged by FG-a. A world opts in by configuration. Storing
`last_contact` changes the `acquaintances` component's encoding, so snapshot bytes change and the pack's
version is bumped (old saves refused by name, as IL-e's bumps do, PA-3); facts and journal are unchanged.

**`conversation`** already caps `remembered` at 1 … 64 (IL-e). FG-a adds an optional `forget_after_days`
(0 = never, default) applied at the next write to the listener's history — a pure function of the stored
lines' instants. **`group-activity`** and **`schedule`** hold nothing that grows (§2.4).

**Why no LLM here.** A System-owned rule runs inside a reduction, and an LLM never mutates world state
(§1.2 ruling 3). A model *may* inform a person's controller that it "has forgotten Bob", but the world's
familiarity value is the System's arithmetic. A user who wants model-shaped forgetting of world state
writes it as a director or a person's controller that *proposes* actions; it cannot be a reduction.

### 8.3 Layer 3 — controller-owned memory: the `ForgettingPolicy` interface

**Carrier.** A forgetting policy is a part of a Controller Pack ("memory policy", `MODULE_SPEC.md` §5). The
bundled policies ship in `mineworld-cognition`; a user policy is a Python package exposing an entry point
`mineworld.forgetting` (the standard `importlib.metadata` mechanism, no new loader). No new pack type.

**Inputs and outputs** (Pydantic models, strict; no `dict[str, Any]`):

```python
class MemoryItem(BaseModel, frozen=True):
    item: ItemId                      # L0 row, L1 episode, L2 chapter, L3 line
    level: Literal[0, 1, 2, 3]
    first_at: WorldSeconds            # world time of its earliest fact
    last_at: WorldSeconds
    kind: str                         # event type for L0 (e.g. "spoke"); "episode" / "chapter" / "stable"
    counterparts: tuple[EntityKey, ...]
    place: PlaceKey | None
    mine: bool                        # I was the actor
    importance: Importance            # 0 … 1 000, computed by the HOST (below), never by the policy
    citations: EventIdRanges          # the facts it stands for
    bytes: int                        # its canonical-dump size

class RetentionBudget(BaseModel, frozen=True):
    store_bytes: int                  # what the store may hold after this pass (host's bound minus slack)
    now: WorldSeconds                 # the day boundary the pass runs at
    calls_left: int                   # model calls this pass may make (0 for no-LLM policies)

class Keep(BaseModel):   item: ItemId
class Drop(BaseModel):   item: ItemId
class Merge(BaseModel):  items: tuple[ItemId, ...]; into: Literal[1, 2, 3]
RetentionDecision = Keep | Drop | Merge

class ForgettingPolicy(Protocol):
    id: ClassVar[str]
    Parameters: ClassVar[type[BaseModel]]          # bounded fields with defaults
    def plan(self, items: Sequence[MemoryItem], budget: RetentionBudget,
             params: BaseModel) -> Sequence[RetentionDecision]: ...

class AssistedForgettingPolicy(Protocol):          # may call a model, only through the gateway
    id: ClassVar[str]
    Parameters: ClassVar[type[BaseModel]]
    fallback: ClassVar[str]                        # a no-LLM policy id
    async def plan(self, items: Sequence[MemoryItem], budget: RetentionBudget,
                   params: BaseModel, gateway: ModelGateway) -> AssistedPlan: ...
    # AssistedPlan = decisions + optional prose per Merge, each prose ≤ 160 B (P4's line bound)
```

**Importance is the host's, and structural.** `importance` is a pure function of the item: a per-kind table
(`spoke` addressed to me > overheard; `became-acquainted`, `relationship-changed`, `invited`, my own acts
high; ambient arrivals low), plus my current relationship level with its counterparts (from the latest
observation impression, P4 §3.8.3), plus reinforcement (how many later facts cite the same counterpart). No
model assigns it — unlike Generative Agents' model-rated 1–10 score — so every policy, including user ones,
sees the same importance, and rebuilding it needs no model.

**Recency is by event time, not by retrieval.** Generative Agents decays recency since an item was last
*retrieved*. Retrieval happens at decision time, which depends on triggers and timing, so a store whose
contents depended on retrieval history would no longer be a derivation of the perceived stream (`ARC-59`,
I-5). Recency here is `now − last_at`. Recorded as `ARC-87`.

**The retention host** (in `memory/`, around every policy):

1. **When.** At each simulated-day boundary crossed during ingestion (the first perceived fact whose `at`
   is in a new day triggers the pass for the day before). The pass is a pure function of the store's state
   at that boundary, so rebuilding from a null cursor replays every pass at the same points (I-5 holds).
2. **Validate.** Every item gets exactly one decision (missing → `Keep`); a `Merge` names items of one
   level below `into`, sharing a counterpart set or a week; unknown ids refuse the plan (the pass falls
   back, below).
3. **Execute.** `Keep` keeps. `Merge` builds the merged summary with P4's `StructuralSummarizer` (the
   policy never writes the index; an assisted policy may supply prose, validated as P4 D-P4-10 validates
   prose: ≤ 160 B, no id-like digit runs, no delimiter tokens) and the summary's citations are the **union**
   of its members'. `Drop` removes the item and adds its citations to the seat's **forgotten set**.
4. **Coverage invariant (replaces P4 AP4 (c)).** The perceived set equals the disjoint union of L0 ∪ L1
   members ∪ merged summaries' citations ∪ the forgotten set. Truth is never lost: a forgotten id still
   resolves in the save (`mineworld perceived`), and tools can say "Alice has forgotten 12 Oct 08:00–14:00"
   (`CORE_CONCEPTS.md` §5.3: "remembers, forgot").
5. **Forgotten set is bounded.** It is a sorted list of Event-id ranges; adjacent ranges merge; beyond 256
   ranges the host collapses the oldest into a single watermark ("everything I perceived before id N that
   is not cited elsewhere is forgotten"). Bounded by construction.
6. **Hard bounds, whatever the policy did.** After executing the plan, if the canonical dump exceeds the
   per-seat bound (1 MiB, §10), the host applies `consolidate` with its defaults; if still over, it drops
   the oldest L0 rows, then the oldest L1, until under — recording `bound-enforced` with counts in the
   seat's decision log. A policy that keeps everything therefore cannot exceed the bound (AC-FG-1's hostile
   policy).
7. **Context bound is untouched.** Retrieval keeps P4's 6 000-byte, per-section budgets (D-P4-9) and gains
   the PA-11 count knob; retrieval no longer reads whole tables (F-S24-2).
8. **Budget ledger pruning.** Rows older than the longest budget window are deleted at the same pass (G9).

**Bundled policies (each an ordinary entry point; none privileged):**

| Policy | LLM | Rule | Parameters (bounded) |
| --- | --- | --- | --- |
| `consolidate` (default) | no | L0 older than `l0_days` merge into their L1 episode; L1 older than `l1_weeks` merge into the week's L2 chapter; L2 older than `l2_years` drop, except those citing an L3 line | `l0_days` 1 … 30 (2), `l1_weeks` 1 … 104 (8), `l2_years` 1 … 50 (5) |
| `decay` | no | an item survives while `importance × retention(now − last_at)` ≥ threshold, with retention a power-law or half-life table (Murre & Dros' power fit as the default shape); survivors over budget fall to `consolidate` | curve id, half-life days, threshold |
| `importance-recency-cap` | no | score = `w_i·importance + w_r·recency`; keep the top `k` per level within `store_bytes`; ties broken by a seeded hash of the first Event ID (A-Res's key made deterministic) | weights, `k` per level |
| `keep-all` | no | keeps everything — then the host's bound applies; exists for AC-10's historical comparisons and as the hostile-policy fixture | none |
| `lm-summarise` | **yes** | candidate groups chosen exactly as `consolidate` would; for each group (names only, no ids, ≤ 6 000 B per request), one `CompletionRequest(purpose="summarize")` returns prose ≤ 160 B and an optional importance hint 1 … 10 that may move a group between `Merge` and `Drop` | `calls_per_sim_day` 0 … 24 (4), tier binding in `cognition.toml` |

**`lm-summarise` determinism and replay.** Every call goes through P5's `ModelGateway`: budget gate
(`ARC-57`) first, then the recorder (`ARC-58`), keyed by the provider-neutral request. In replay mode a
missing recording raises `CassetteMiss`, which the pass propagates (a test that silently fell back would
pass while proving nothing). In live mode, `Refused`, `Failed`, an unbound tier, or `calls_left = 0` make the
host apply the declared `fallback` for the remaining groups, and the seat's store records which policy
produced each decision. Consequence: rebuilding a store **with recordings** reproduces it byte for byte;
rebuilding **without** them reproduces the fallback's store and says so — the difference is visible, never
silent (QS24-6).

### 8.4 Selection

Per world and per person through `decisions.yaml` `memory:` (§4.2), with the same typed decoding and
refusals. For System-owned state, selection is `configure/<pack>.yaml` (§8.2) — never `decisions.yaml`,
because a controller file may not configure world state (§1.2 ruling 2(6)).

### 8.5 Two example plug-ins for forgetting

| Example | Layer | Acceptance |
| --- | --- | --- |
| `consolidate` (FG-b) | controller memory, no LLM | AC-FG-1, AC-FG-3, AC-FG-4 (no-LLM half) |
| `lm-summarise` (FG-c) | controller memory, LLM | AC-FG-2, AC-FG-4 (cassette half), AC-FG-5 |
| `relationships` decay + cap (FG-a) | System-owned, no LLM | AC-FG-6, AC-FG-7 |

### 8.6 How this meets P6

- P6 is the first seat with a memory store in a live world; FG-b must merge **before** P6, so P6 is never
  built against an unbounded store (the operator's token-explosion concern) — FG-c may merge alongside it.
- P6's context assembly is unchanged: it calls retrieval, which honours the forgotten set (a forgotten fact
  is never shown and never citable — `recalls` validation rejects a forgotten id).
- P6's routine policy (step-17 §3.4.6) becomes "the seat's fallback model", so the Python port of the paced
  rule (QS10-8) can be replaced by naming `motivated` or `rule` as `lm`'s fallback in `decisions.yaml`,
  driven in-process by the server while the LM seat is idle (S11-B hosted fallback).
- Milestone D's test (P7) gains one assertion: after 100 simulated days Alice's store is ≤ 1 MiB and her
  reply still cites the 2D lines — which forces those lines to be important enough to survive, or merged
  with their ids (a useful adversarial check on importance).

---

## 9. World evolution

### 9.1 What it covers, and what already exists

Changes beyond individual people: market days, festivals, prices, shop stock, news, weather-driven
happenings. **`weather` is already the first no-LLM world-evolution System**: record-driven and seeded
(`ARC-68`), a Process that writes hourly state, read by any controller that wants it. This step generalises
the pattern for *happenings*.

### 9.2 The no-LLM example: the `happenings` System Pack (WE-a)

```text
provides    nothing in WE-a; WE-b adds propose-happening { kind, place, start, duration, text ≤ 240 B }
            for Organization actors (directors, §9.3)
owns        Happenings (on the world's places: announced, running; bounded list per place)
processes   one per running happening (start → end); a daily process drawing scheduled and random ones
facts       happening-announced, happening-began, happening-ended (audience per IL consequences:
            place occupants, or everyone for "news")
configure   configure/happenings.yaml (IL section): the kinds catalog, calendar entries, random rates
depends on  presence; reads calendar's date where installed
```

```yaml
# configure/happenings.yaml
kinds:                               # an ARC-62 catalog: bundled kinds + kinds other packs plug in
  - { id: market-day, places: [street], duration: "08:00-14:00" }
  - { id: festival,   places: [park],   duration: "10:00-22:00", announce_days: 7 }
calendar:                            # deterministic
  - { kind: market-day, every: saturday }
  - { kind: festival,   on: "10-31" }
random:                              # seeded integer draws: per mille per day, from (world seed, day, kind)
  - { kind: street-musician, place: street, per_mille_per_day: 150 }
limits:                              # the System's own hard caps (validation, not advice)
  { per_day: 3, per_place_running: 1, proposals_per_actor_per_day: 2 }
```

**Single ownership, other packs react.** `happenings` writes only `Happenings`. Prices on market day are
`economy`'s (a configure parameter `price_modifiers: [{ during: market-day, place: street, ‰: 900 }]`, read
in its own reduction); shop stock is `inventory`'s via an economy restock rule; an agenda change for a
festival is `schedule`'s (an optional `react: [{ happening: festival, label: festival }]` in its section).
Each such reaction is the reacting pack's configuration, opt-in, and absent by default.

**Determinism.** Calendar entries are pure functions of the date; random entries use the counter-based
mixer over `(world seed, day, kind index)` — no RNG state — so the history is a function of the pack, the
seed and the code; replay re-executes as for every System.

**Acceptance (WE-a).** AC-WE-1: same seed → same `happening-*` facts, twice and across kill/resume, on
three platforms. AC-WE-2: remove `happenings` (and its configure entry) → the world loads and runs; packs
that react to it decide exactly as without it. AC-WE-3: a world without `configure/happenings.yaml` that
enables the pack records no happening (no surprise content). AC-WE-4: `limits` hold under a 1 000-day run
(counted, not sampled).

### 9.3 The LLM example: a director (WE-b, MVP-1)

A director is a **Controller** whose seat is an **Organization** (e.g. the town council, `CORE_CONCEPTS.md`
§8), driven by a Controller Pack through the same protocol as an LM person (step-17 §3.2). It observes what
its Organization perceives (happenings, places, perhaps aggregate "news" facts) and submits
`propose-happening` intents. `happenings` validates every proposal: the kind is in the catalog, the place
exists and admits the kind, the start lies in the allowed window, no limit is exceeded, the text is bounded.
It may refuse (`PermissionDenied`, `Busy`, a typed `Rejection`), and a refused proposal changes nothing. The
director has **no referee authority**: it never resolves anyone's action, never edits state, and its free
text is a payload, like `talk`'s (`DEP-46`, PA-10: Concordia's Game Master, inverted).

- **Why an Organization seat.** A director is not a person, and casting a "mayor" Person as a puppet would
  confuse biography (the mayor would appear to do things nobody saw). `CORE_CONCEPTS.md` §14 says a
  Controller decides what a *Person* attempts, and `worldpack` refuses a seat that is not a person
  (`worldpack/src/error.rs`). Extending "seat" to Organizations is an ontology change (QS24-2). The
  alternative without it — the director drives a designated Person — is listed there too.
- **Bounds.** `limits.proposals_per_actor_per_day` (System-enforced, deterministic, in the world); P5's
  budget gate (calls per simulated hour, tokens per simulated day) for the model; payload size by the
  action's contract.
- **Removal.** With the director removed (or its process dead), its seat runs its `fallback` (`none` by
  default) and `happenings` keeps its own calendar and random schedule: the world is valid and still
  evolves.
- **Determinism.** Proposals are journaled requests; replay never calls the director (`ARC-25`). Re-running
  the director under cassette replay reproduces its proposals and therefore the history.

**Acceptance (WE-b).** AC-WE-5: under cassette replay, two runs give the same `happening-*` facts. AC-WE-6: a
scripted director proposing an unknown kind, an over-limit proposal, and an out-of-window start is refused
three times, with no state change and the typed reasons. AC-WE-7: kill the director's process mid-run; the
world continues, and the seat's fallback is in effect within one consult.

### 9.4 Two example plug-ins for world evolution

`happenings` (no-LLM, System Pack, WE-a) and the director (LLM, Controller Pack, WE-b), with the criteria
above. A scripted storyline (an authored sequence of proposals, a deterministic Controller Pack on the same
seat) is a third, cheap example that exercises the director path with no model, and is WE-b's test double.

---

## 10. Hard bounds and acceptance

### 10.1 The bounds (enforced by hosts, hold in every mode)

| Bound | Value (recommended, QS24-4) | Enforced by | Measured by |
| --- | --- | --- | --- |
| Per-seat controller memory store (canonical dump) | ≤ 1 MiB at any age | retention host (§8.3 step 6) | AC-FG-1 |
| Memory section per model call (social decision) | ≤ 6 000 B (P4 D-P4-9, unchanged) | retrieval | P4 AP4 (a), unchanged |
| Input per summarisation call | ≤ 6 000 B | `lm-summarise` request builder, checked by the host before the gateway | AC-FG-2 |
| Model calls for forgetting | ≤ `calls_per_sim_day` (default 4) per seat, inside P5's per-seat token budget | P5 budget gate (`ARC-57`) | AC-FG-5 |
| System-owned person state | acquaintances ≤ `known_at_most` (150) per person; conversation lines ≤ `remembered` (≤ 64); together ≤ 160 KiB per person in a snapshot | the owning Systems | AC-FG-7 |
| Save growth per simulated day | ≤ 0.25 MiB/day for Market Town on the baseline profile and ≤ 0.5 MiB/day with walking, over days 100–300 of a saved run | SR-b (persistence) — the log is never truncated, so this is a design bound checked nightly, not a runtime refusal | AC-SAVE-1 |
| `happenings` and director | `limits` in the world's section | `happenings`' validation | AC-WE-4, AC-WE-6 |

The save bound cannot be enforced by refusing facts — truth is not rationed — so it is a nightly gate on
reference worlds plus per-System emission limits (`limits`) where a System generates content.

**Computed check of the 1 MiB store bound under `consolidate` defaults** (estimate, to be measured):
2 days of L0 ≈ 390 KB (195 KB/day, §2.3); 8 weeks of L1 at an estimated ≈ 3 KB/day ≈ 170 KB; 5 years of L2
at ≈ 1 KB/week ≈ 260 KB; L3 ≤ 16 lines; forgotten set ≤ 256 ranges. Total ≈ 0.8 MiB — under the bound, with
the host's enforcement as the backstop if the L1 estimate is low.

### 10.2 Acceptance criteria (decided before measuring, `ARC-23`)

| Id | Criterion | Counterfactual it must catch |
| --- | --- | --- |
| AC-DL-1 | A world with no `decisions:` key and no flag: `scripts/baselines.txt` unchanged, every world, every column | the host constructing models in a different order or with a different seed derivation |
| AC-DL-2 | social-cafe with `staff` → `motivated`, others `rule`: two runs and a kill-and-resume run give equal fingerprints, Linux/macOS/Windows | a model holding state (`RefCell`) across consults |
| AC-DL-3 | A decision list naming an uninstalled model, unknown class, non-seat person, or unknown parameter is refused at load by name with line/column; removing an assignment runs the world | silently ignoring an unknown key |
| AC-DL-4 | Calibrated rhythm within tolerance (§6.6), tolerances written in the PR design before the run | curves not applied (rule bands still deciding) |
| AC-DL-5 | An `lm` seat with its process killed runs its fallback within one consult; under cassette replay its requests are identical | fallback never engaged; replay that reaches a backend |
| AC-FG-1 | Every bundled policy, and the hostile `keep-all`, keeps the per-seat dump ≤ 1 MiB at days 30, 100 and 300 of social-cafe (Alice's perceived export) | host bound disabled |
| AC-FG-2 | Every summarisation request ≤ 6 000 B, contains no Event or entity id | ids leaking into prompts |
| AC-FG-3 | Coverage: perceived set = L0 ∪ L1 ∪ merged citations ∪ forgotten set, disjoint, at every day boundary | a `Drop` that does not record the forgotten ids |
| AC-FG-4 | Determinism: rebuild from a null cursor gives a byte-identical dump for no-LLM policies; with recordings for `lm-summarise`; `CassetteMiss` propagates in replay mode | retrieval-dependent recency; a swallowed `CassetteMiss` |
| AC-FG-5 | `lm-summarise` with `calls_per_sim_day: 0` or an exhausted budget produces exactly the fallback's store, and records that it did | a fallback that is not recorded |
| AC-FG-6 | FG-a with defaults: baselines unchanged | defaults that decay |
| AC-FG-7 | A synthetic 300-person world (test-only pack) over 100 days: every person's acquaintances ≤ `known_at_most`; with `half-life` decay configured, a pair with no contact for one half-life discloses half its familiarity (± rounding) and no decay fact was written | a decay Process writing daily facts |
| AC-SAVE-1 | Nightly: Market Town 300-day saved run, (size at day 300 − size at day 100) ÷ 200 ≤ the bound, with SR-b | SR-b disabled |
| AC-WE-1 … AC-WE-7 | §9.2, §9.3 | as stated there |

---

## 11. PR breakdown

### 11.1 Order

```text
MVP-0
  DOC-24   specs: CORE_CONCEPTS §5.3 (forgotten set), MODULE_SPEC §4.1 (decisions:), §5 (models,
           memory policy), ARCHITECTURE §9; DECISIONS ARC-84…ARC-88, DEP-47, DEP-48; P4 QP4-11 superseded note
     │
     ├──► DL-a  decision list + DecisionModel trait + cognition/installed; `rule` as first model;
     │          run / --town read decisions.yaml; absent = byte-identical (AC-DL-1, AC-DL-3)
     │            └──► DL-b  `motivated` utility model + routine-fit (ATUS A-3) + named curve set
     │                       (AC-DL-2, AC-DL-4)
     ├──► FG-a  relationships lazy decay + cap; conversation forget_after_days (AC-FG-6, AC-FG-7)
     ├──► FG-b  retention host + ForgettingPolicy + consolidate / decay / importance-recency-cap /
     │          keep-all; forgotten set; ledger pruning; F-S24-2 (AC-FG-1, -3, -4)   ── before P6
     │            └──► FG-c  lm-summarise via P5 gateway, cassettes (AC-FG-2, -4, -5) ── alongside P6
     ├──► FG-d  SR-b sealed blocks (pr-s6-save-retention.md §6.9; QSR-1) + AC-SAVE-1 nightly
     └──► WE-a  happenings System Pack + market-town content (AC-WE-1 … -4)

  P6 (step-17) consumes DL-a (the `lm` entry and its fallback) and FG-b (bounded store);
  P7 adds the 100-day store assertion (§8.6)

MVP-1
  DL-c  needs System Pack (§6.5) + motivated's needs considerations
  WE-b  director (Organization seats, QS24-2) + scripted storyline double (AC-WE-5 … -7)
  FG-e  cold archive segments (§8.1 item 3), after F-S24-1
  DL-d  optional: GOAP/HTN model once Systems declare effects (§6.2)
```

### 11.2 The PRs

| PR | Scope | Depends on | Phase | Integration checkpoint |
| --- | --- | --- | --- | --- |
| **DOC-24** | Spec amendments and decision records (§13); no code | this design approved | MVP-0 | doc checks |
| **DL-a** | `cognition/decision` (trait, `Draw` moved from `paced.rs`, `Seed`), `cognition/installed`, `PacedRuleController` implements `DecisionModel` unchanged, `worldpack` reads `decisions:` (typed, refused by name), `run`/`hosted` resolve per seat, `--decisions`, `--seat-model` | DOC-24 | MVP-0 | AC-DL-1, AC-DL-3; `run` over all four worlds with and without a decision list |
| **DL-b** | `cognition/motivated` (utility model, curves, parameters), `tools/routine-fit`, committed ATUS-derived curve set with credit, social-cafe opt-in `decisions.yaml` example | DL-a | MVP-0 | AC-DL-2, AC-DL-4 over 30-day runs |
| **FG-a** | `relationships`: `(value, last_contact)`, lazy `decayed`, `DecayCurve` catalog (`none`, `linear`, `half-life`, `power`), `known_at_most` eviction + `acquaintance-forgotten`; `conversation`: `forget_after_days`; test-only 300-person pack | DOC-24 (IL-b merged) | MVP-0 | AC-FG-6, AC-FG-7 |
| **FG-b** | retention host, `MemoryItem`/`RetentionBudget`/decisions, entry-point registry, bundled no-LLM policies, forgotten set, coverage invariant, ledger pruning, bounded retrieval reads; `decisions.yaml` `memory:` decoded by the cognition process | DOC-24; P4 (merged) | MVP-0, **before P6** | AC-FG-1, -3, -4 on the AC-10 harness extended to 300 days |
| **FG-c** | `lm-summarise` (`AssistedForgettingPolicy`), prose validation, fallback recording, cassette tests | FG-b; P5 (merged) | MVP-0, alongside P6 | AC-FG-2, -4, -5 |
| **FG-d** | SR-b as designed in `pr-s6-save-retention.md` §6.9; nightly AC-SAVE-1 | QSR-1 ruling | MVP-0 | AC-SAVE-1; ARC-25 replay over a sealed save |
| **WE-a** | `systems/happenings`, IL section with kinds catalog, limits; Market Town market days and a festival; optional `economy` price modifier | DOC-24 | MVP-0 | AC-WE-1 … -4 |
| **DL-c** | `systems/needs`, lazy decay, restoration by reaction; `motivated` considerations | DL-b | MVP-1 | needs rhythm vs ATUS eating/sleeping curves |
| **WE-b** | Organization seats (if QS24-2 is yes), director Controller Pack (Python, P5 gateway), scripted storyline double | WE-a; P6 | MVP-1 | AC-WE-5 … -7 |
| **FG-e** | archive segments, manifest, streaming readers | FG-d; F-S24-1 | MVP-1 | full replay over archived save; resume opens no segment |

*Dated note, 2026-10-12 (primary session ruling on QDLb-1, `mvp0/pr-dl-b-motivated.md` §14.1):* the DL-b row
is narrowed. DL-b ships no `tools/routine-fit`: the `motivated` curves are derived in-crate by a pure function
from the committed ATUS A-3 table (edition `atus-a3-2025`), and the Markov-chain **routine generator** of §6.6
is deferred to a later PR (with DL-c or the reference-world switch of QS24-3). Also (QFGb-1, 2026-10-12):
DL-a decodes `decisions.yaml` `memory:` and emits each seat's `MemoryPolicyChoice` as typed JSON for the
cognition process (`mvp0/pr-dl-a-decision-list.md` SD-DLa-9, SD-DLa-15).

*Dated note, 2026-10-18 (operator ruling, primary session; DOC-S35):* the **DL-c row is superseded.** It moves
from MVP-1 into the MVP-0 release as step 35,
[`../mvp0/step-35-needs-and-sleep.md`](../mvp0/step-35-needs-and-sleep.md), split into PRs N-a (`sleep`), N-b
(`needs`), N-c (controllers), N-d (content and acceptance) and N-e (presentation and views). No DL-c PR is opened.

### 11.3 Parallelism

DL-a, FG-a, FG-b, FG-d and WE-a touch disjoint crates (`cognition/decision` + hosts; `systems/relationships`
+ `systems/conversation`; `cognition/lm-controller/memory`; `persistence`; `systems/happenings`) and may run
in parallel worktrees after DOC-24. DL-a and WE-a both touch `worldpack` (a new key; a new system id) — one
line each, coordinated. FG-b and P6 both touch `mineworld_cognition`; FG-b merges first.

---

## 12. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S24-1 | `motivated` produces worse-looking behaviour than `rule` in the operator's eyes | `rule` stays the default (QS24-3); AC-DL-4 measures rhythm, and the 2D client shows both on one seed side by side |
| R-S24-2 | Utility tuning becomes endless | curves are data from a named source, fixed before measurement (`ARC-23`); no retuning against a bad-looking world — the remedy is in the curves' source or the world's Systems |
| R-S24-3 | Lazy decay surprises a reader that caches disclosed values | every disclosure computes the current value from `(value, last_contact, now)`; P6's impressions store the disclosed value with its instant |
| R-S24-4 | Forgetting drops the lines Milestone D cites | importance favours lines addressed to me; P7's 100-day assertion is an adversarial check; `lm-summarise` keeps merged ids |
| R-S24-5 | An LLM-assisted plan makes the store depend on recordings | recorded per decision; fallback recorded; QS24-6 asks the operator to accept this |
| R-S24-6 | The per-day save bound fails with walking even after SR-b | measured by AC-SAVE-1; the lever after SR-b is walk fact volume, routed to movement |
| R-S24-7 | Organization seats ripple through server, clients and perception | WE-b is MVP-1 and gated on QS24-2; the designated-Person alternative exists |
| R-S24-8 | User decision models break determinism (hidden state) | the trait's `&self` signature, a `clippy` disallowed-types list for model crates, and AC-DL-2's kill-and-resume equality in the model's own tests (a template test DL-a ships) |
| R-S24-9 | ATUS licensing misread | only BLS public-domain aggregate tables are committed, with credit; IPUMS/MTUS/HETUS microdata never enter the repository |

---

## 13. Proposed edits to documents this session may not edit

| Document | Edit | PR |
| --- | --- | --- |
| `pr-s10-p4-memory.md` | Dated note under QP4-11 and R-P4-4: "Superseded 2026-10-12 by the operator: forgetting is in MVP-0 (step-24 §1.2, FG-b/FG-c)." | DOC-24 |
| `step-17-cognition.md` §9, §15.2 (P6 row) | P6 depends on FG-b; its routine policy is the seat's decision-list fallback; P7 gains the 100-day store assertion | DOC-24 |
| `overall.md` | S24 entry with MVP-0/MVP-1 placement; the 2026-10-12 rulings; decision numbers | DOC-24 |
| `CORE_CONCEPTS.md` §5.3 | the forgotten set: forgetting in memory is recorded as ranges, never by deleting truth | DOC-24 |
| `CORE_CONCEPTS.md` §14 | only if QS24-2 is yes: "A Controller decides what a Person — or an Organization seated as one — attempts" | WE-b |
| `MODULE_SPEC.md` §4.1 | `decisions:` key and `decisions.yaml` shape; never seeded | DL-a |
| `MODULE_SPEC.md` §5 | decision models, `cognition/installed`, memory policies and their host bounds | DL-a, FG-b |
| `ARCHITECTURE.md` §9 | controller memory retention and its bounds | FG-b |
| `DECISIONS.md` | `ARC-84` the decision list is host-read content, not world state · `ARC-85` behaviour extension points (interface, carrier, selection, parameters, host) · `ARC-86` System-owned forgetting is lazy, pure and in the owner; no model · `ARC-87` controller-memory retention: host-computed importance, event-time recency, forgotten set, hard bounds · `ARC-88` world evolution: happenings System + directors that only propose · `DEP-47` build our own utility core (big-brain, bonsai-bt, dogoap rejected) · `DEP-48` ATUS A-3 public-domain tables via an offline fit tool | DOC-24 |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS24-1** | Where does the decision list live: (a) World Pack file read by hosts, never seeded; (b) a `configure/` framework key, seeded with drift check; (c) operator-only configuration? Should a save record the resolved list as non-authoritative host metadata? | **(a)**, with operator override. Not seeded (`INV-1`). Print its digest in `run`'s header now; storing it in a save is deferred until a resume actually needs it. |
| **QS24-2** | May an Organization be a seat, so a director Controller drives it (amends `CORE_CONCEPTS.md` §14 and `worldpack`'s seat rule)? Alternative: the director drives a designated Person. | **Yes, narrowly**: an Organization may be a seat; it can attempt only actions whose System declares Organization actors (`propose-happening`). Decided before WE-b (MVP-1). |
| **QS24-3** | Default decision model for the reference worlds? | **`rule` stays the default** in MVP-0 (byte-identical baselines). social-cafe ships an opt-in `decisions.yaml` example; Market Town switches to `motivated` in a later PR with a baselines reason line, after the operator has watched both. |
| **QS24-4** | The hard bound values: per-seat store ≤ 1 MiB; ≤ 6 000 B per call; System-owned person state ≤ 160 KiB; save growth ≤ 0.25 MiB/day (baseline) and ≤ 0.5 MiB/day (walking) for Market Town? | **Yes**, as recommended; each PR measures and may tighten, never loosen without a ruling. |
| **QS24-5** | Build SR-b (QSR-1) in MVP-0 now, since the per-day save bound depends on it? | **Yes.** It is the only lever below ≈ 1.9 MiB/day and is already designed. |
| **QS24-6** | May an LLM-assisted forgetting policy influence *which* memory items survive (not only their prose), given provenance is preserved and the fallback is recorded? | **Yes.** Otherwise "LLM forgetting" is cosmetic. Truth is never lost (forgotten ids still resolve), and a rebuild without recordings visibly differs instead of silently. |
| **QS24-7** | Relationship decay: default off (byte-identical) or on? | **Off by default**; the cap (150) is on but inert in 12-person worlds. Market Town opts in with a half-life once DL-b reads relationships. |
| **QS24-8** | `needs` System Pack (hunger, energy) in MVP-0 or MVP-1? | **MVP-1.** `motivated` is useful without it; needs add actions and restoration rules worth their own design. |
| **QS24-9** | Calibration data: commit derived integer tables from BLS ATUS A-3 (public domain, credited), and keep IPUMS/MTUS/HETUS microdata out of the repository? | **Yes.** |
| **QS24-10** | Keep `keep-all` as a selectable policy? | **Only behind the host bound** (it then degenerates to oldest-first dropping at the cap); documented as a test fixture. |
| **QS24-11** | Recency by event time rather than by last retrieval (diverging from Generative Agents)? | **Yes**: retrieval-dependent stores are not derivations of perception (`ARC-59`, I-5). |
| **QS24-12** | Should `mineworld run` host out-of-process models (LM seats, directors) in MVP-0? | **No.** `run` stays synchronous and model-free; LM seats use their fallback there and run for real on the hosted server. |
| **QS24-13** | Cold archive segments (FG-e) in MVP-1, after F-S24-1's streaming readers? | **Yes.** |
| **QS24-14** | File the MVP-0 PRs (DOC-24, DL-a, DL-b, FG-a … FG-d, WE-a) under `mvp0/` when detailed? | **Yes**, so MVP-0's ledger shows them; this step document stays the single authority and moves with them. |

### 14.1 Rulings, 2026-10-12 (operator, relayed by the primary session)

| Question | Ruling |
| --- | --- |
| **QS24-2** | **Yes, narrowly.** An Organization may hold a seat, **for the director only**. The details are decided before the director PR (WE-b, MVP-1), and `CORE_CONCEPTS.md` §14 is amended then, not now. |
| **QS24-6** | **Yes.** LLM forgetting may decide which memories survive. The host's bounds still apply, forgotten Event IDs stay resolvable, and falling back to the no-LLM policy on failure is recorded. |
| **QS24-8** | **MVP-1.** The `needs` System Pack is in MVP-1 (DL-c). |
| QS24-1, 3, 4, 5, 7, 9, 10, 11, 12, 13, 14 | **Accepted as recommended**, under the operator's standing instruction. |
| Decision ids | `ARC-S24-a … e` → `ARC-84` … `ARC-88`; `DEP-S24-a … b` → `DEP-47`, `DEP-48` (primary session). Next free: `ARC-89`, `DEP-49`. |

Consequences recorded in this document: §0 and §11 placements stand unchanged. SR-b (FG-d) is in MVP-0
under QS24-5, which also answers QSR-1 in `pr-s6-save-retention.md`. QS24-14 means the MVP-0 PRs are filed
under `mvp0/` when each is detailed.

### 14.2 Process notes (this planning session)

Two slips against the brief, recorded rather than hidden. Neither changed any file other than this
document.

- The first commit and its push were chained in one shell command with `&&`, not run as separate commands
  as the brief asked.
- While waiting for CI, one read-only inline `python3 -I -c` call parsed `gh pr checks` JSON. The brief
  said no inline Python. It read nothing from the repository and wrote nothing.

---

## 15. Sources (accessed 2026-10-12)

Verification status as found by this session's research; "secondary" means read through a secondary page
or a search result, and such a source is re-read at the citing PR's freeze before it enters a decision
record.

**Utility AI and needs.** Dave Mark, *Behavioral Mathematics for Game AI* (Charles River Media, 2009,
ISBN 978-1-58450-684-3) — secondary (review, gbgames.com). Kevin Dill and Dave Mark, "Improving AI Decision
Modeling Through Utility Theory", GDC AI Summit 2010 — secondary. Dave Mark, "Embracing the Dark Art of
Mathematical Modeling in AI", GDC 2012, https://gdcvault.com/play/1015421/ — listed. Infinite Axis Utility
System: https://gameai.com/iaus.php, https://en.wikipedia.org/wiki/Utility_system — secondary; Mark and
Lewis, "Building a Better Centaur: AI at Massive Scale", GDC 2015 — title verified, video not opened. The
Sims' motives and advertisements: https://en.wikipedia.org/wiki/Don_Hopkins and a talk transcript — secondary;
no primary decay numbers used.

**Time use.** ATUS: https://www.bls.gov/tus/ (verified; Table A-3 by time of day — editions to be confirmed
at DL-b's freeze). BLS copyright: https://www.bls.gov/opub/copyright-information.htm (verified: public domain,
credit requested). Eurostat reuse: https://ec.europa.eu/eurostat/web/main/help/copyright-notice (verified).
HETUS microdata access: https://ec.europa.eu/eurostat/web/microdata/harmonised-european-time-use-surveys —
secondary. IPUMS terms: https://www.ipums.org/about/terms — secondary (ATUS-X page not loaded). MTUS:
https://www.timeuse.org/mtus (verified: registration; redistribution unstated).

**Planning and behaviour.** Jeff Orkin, "Three States and a Plan: The A.I. of F.E.A.R.", GDC 2006,
https://gdcvault.com/play/1013282/ — secondary. Troy Humphreys, "Exploring HTN Planners through Example",
*Game AI Pro* ch. 12 (2013),
https://gameaipro.com/GameAIPro/GameAIPro_Chapter12_Exploring_HTN_Planners_through_Example.pdf — not opened.
Crates: https://docs.rs/crate/big-brain/latest, https://github.com/Sollimann/bonsai,
https://docs.rs/crate/dogoap/latest — secondary.

**Agent-based modelling.** Mesa: https://mesa.readthedocs.io/latest/ (verified; seeding practice). NetLogo
programming guide: https://docs.netlogo.org/programming.html — partly read. MATSim: https://matsim.org/ and
Horni, Nagel, Axhausen (eds.), *The Multi-Agent Transport Simulation MATSim* (2016, CC BY 4.0) — secondary.

**Memory and forgetting.** Park et al., "Generative Agents: Interactive Simulacra of Human Behavior",
https://arxiv.org/abs/2304.03442 (abstract verified; recency decay 0.995 per game hour, model-rated
importance 1–10, embedding relevance — secondary). Murre & Dros, "Replication and Analysis of Ebbinghaus'
Forgetting Curve", PLOS ONE 2015, https://doi.org/10.1371/journal.pone.0120644 (verified). Packer et al.,
"MemGPT", https://arxiv.org/abs/2310.08560 (abstract verified). Vezhnevets et al., Concordia,
https://arxiv.org/abs/2312.03664 (abstract verified). Vitter, "Random Sampling with a Reservoir", ACM TOMS
11(1), 1985 — secondary. Efraimidis & Spirakis, "Weighted random sampling with a reservoir", IPL 97(5), 2006
— secondary. Duffield, Lund & Thorup, "Priority sampling for estimation of arbitrary subset sums", JACM 54(6),
2007, https://arxiv.org/abs/cs/0509026 — secondary. Dunbar layers (≈ 5, 15, 50, 150): Zhou et al., Proc. R.
Soc. B 2005 — secondary. Roberts & Dunbar, "The costs of family and friends", Evolution and Human Behavior
32 (2011) — secondary; no numeric rates used. Burt, "Decay functions", Social Networks 22 (2000) —
secondary.

**Event-sourcing archival.** Kurrent (EventStoreDB) scavenging:
https://docs.kurrent.io/server/latest/operations/scavenge.html (verified: destructive). Marten archiving:
https://martendb.io/events/archiving.html (verified: non-destructive). Kafka tiered storage (KIP-405):
https://cwiki.apache.org/confluence/display/KAFKA/KIP-405%3A+Kafka+Tiered+Storage — secondary.

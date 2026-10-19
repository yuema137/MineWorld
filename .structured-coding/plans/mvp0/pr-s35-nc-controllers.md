# PR N-c — Controllers and needs: `rule` and `motivated` sleep, eat and wake for work

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes)

```text
Design revision:        revision 1 (2026-10-18)
Parents:                step-35-needs-and-sleep.md (STEP DESIGN FROZEN 2026-10-18) §6.2, §6.3, §6.4, §9, §10.1 row N-c;
                        pr-le-leave-early.md revision 2 §0.4 (the rule parameters block, the commute lead)
Depends on:             N-a (sleep), N-b (needs), LE-a (rule parameters, commute, agenda.next, ways)
Implementation base:    origin/main at C0, after N-b and LE-a have merged; C0 records the commit
Start condition:        this design FROZEN (coordinator relays the operator's freeze) AND N-b, LE-a merged
Execution contract:     §9
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0`, release scope. **Records:** cites `ARC-286` (controllers and needs; written by DOC-S35) and
`ARC-282` (departure is each controller's decision; LE revision 2). No new record.

**What this PR does, in one paragraph.** After N-a and N-b, people in a world with the packs have a hunger and a
tiredness, and can sleep — but no controller uses them, so nobody ever goes to bed. This PR teaches the two
in-process decision models to read their own `Needs` and `Asleep` and act on them: a tired person goes to bed at
night, a hungry person eats what they hold, a sleeper wakes in time for a punctual shift, and people do not greet
somebody who is asleep. Every number that shapes this behaviour is data in the decision list, never a constant
tuned to Market Town (operator ruling R-LE2-2, 2026-10-18). In a world without the packs, nothing is disclosed,
and both models decide exactly as before: same requests, same draws.

---

## 1. Binding inputs

- Step-35 §6.2 (rule bands), §6.3 (motivated considerations), §6.4 (LM fallback), §9 AC-S35-2 (controller half),
  AC-S35-9, AC-S35-13 (mutation "draw when absent"), I-S35-4, I-S35-7.
- LE revision 2 §0.4: `Rule` has typed parameters in `decisions.yaml` (`models: { rule: … }`); `motivated` departure
  is a scored option; the shared `commute::lead` (Rust) gives the departure instant.
- Operator rulings 2026-10-18: needs are motivation only (Q-S35-3); no world-specific hard-coding, and every
  acceptance criterion also holds on a second world or variant (R-LE2-2); SPEED FIRST (acceptance and mutations at
  the milestone).
- `ARC-27` (a model is a pure function of the observation), `ARC-34` (complete affordances), `ARC-84` (`Consult`),
  I-DLb-4 (an absent input is neutral, 1 000).

---

## 2. Audit (`origin/main @ a1e05874`, 2026-10-18; C0 re-checks on the base)

| Thing | Where | What it gives this PR |
| --- | --- | --- |
| `PacedRuleController::decide` | `cognition/rule-controller/src/paced.rs` | precedence today: invitation (`social.rs`, draws 8–12) → reply (draw 1) → agenda (draw 13, 90 %) → social initiative → offered (draws 14, 15) → walking roll (draw 0; 2–6). Draw 7 and 16+ are free (header table) |
| `Rule` model | `cognition/rule-controller/src/model.rs` | `type Parameters = ()` today; LE-a C4 gives it a parameters block (`RuleParameters`, `commute`) — N-c adds a `needs` key to that block |
| `offered::attempt` | `cognition/rule-controller/src/offered.rs` | a held food's `eat` is a **complete** affordance (consumption offers one per held kind, `systems/consumption`); today it is drawn at 20 % among all offers at any hour (F-S35-1) |
| `agenda::own` | `cognition/rule-controller/src/agenda.rs` | the pattern for decoding a disclosed record with its owner's type (`Agenda::COMPONENT_TYPE`); N-c decodes `Needs`, `Asleep` and `nourishment` the same way |
| motivated precedence | `cognition/motivated/src/model.rs` `decide` | invitation → reply → scored options (`score::score_all`, band `top_band`, draw) |
| motivated considerations | `cognition/motivated/src/considerations.rs` | `Rested` and `HungryHour` are ATUS hour curves; `of(choice, …)` returns the per-mille list per choice; an absent input is 1 000 |
| motivated parameters | `cognition/motivated/src/parameters.rs` | `OVERRIDABLE` curve names, `base` keys (`OPTIONS`), positioned refusals; LE-a C5 adds `commute`, `departure`, `unhurried`, `Commute` |
| motivated options | `cognition/motivated/src/options.rs` | `Choice::{Agenda, Stay, Greet, Approach, Invite, Join, Leave, Door, Offered, Wander}` (+ `Commute` from LE-a) |
| Cargo dependencies | both `Cargo.toml` | rule depends on the types of conversation, movement, group-activity, naming, schedule; N-c adds `mineworld-sleep` and `mineworld-needs` (types only, as for schedule) |
| LM fallback | `docs/MODULE_SPEC.md` §5 (LM Controller) | a handed-back LM seat is driven by its `fallback` model in the server; with `rule` as fallback it gets every band here with the decision list's `models.rule` parameters |

What N-a and N-b provide (by their designs; C0 confirms the merged names): `Asleep { since, alarm? }` disclosed on
a person (alarm to the holder only); `sleep { for }` (parametric) and `wake {}` offers; `Needs { hunger, tiredness,
words }` to the holder; each held food item's `nourishment` value disclosed to its holder. **If N-b does not
disclose an item's nourishment to its holder, the eat band below cannot choose by it: that is a stop at C0, raised
to the lane lead (the step's §6.2 item 2 needs it).**

---

## 3. Scope

### 3.1 In scope

1. `rule`: a `needs` block in its parameters, and the bands of step §6.2 reading it (SD-Nc-1 … SD-Nc-5).
2. `motivated`: the considerations `hungry`, `sated`, `tired`, `wakeful` of step §6.3 as overridable curves, the
   `Choice::Sleep` and `Choice::Wake` options, and the asleep rung (SD-Nc-6).
3. Both: wake in time for a punctual next part, using LE-a's `commute::lead` (SD-Nc-4).
4. Tests: unit tests on synthetic observations; an integration test on two worlds (§6), the milestone runs written
   and `#[ignore]`d.

### 3.2 Not in scope

- Rendering needs in the LM prompt and agent view (N-e). The LM model decides from those words; N-c covers only the
  fallback, which is `rule` and needs no extra code.
- Content: no reference world enables the packs here (N-d). Tests use test copies.
- Individual variation from S30's `chronotype` and `sleep-need` (Q-S35-11, later).
- Shopping for food in `rule` (step §6.2: it relies on the offered band).

### 3.3 Invariants

- **I-Nc-1 — inert when absent.** If the observation discloses neither `Needs` nor `Asleep` for the observer, both
  models return the same request and take the same draws as on the base. Mechanism: every new band begins with
  `let Some(needs) = needs::own(observation) else { skip }` before any draw; the bedtime offset is a `mix` over
  `(seed, observer)`, not a `Draw` index, and is computed only inside the band; no existing draw index changes.
- **I-Nc-2 — pure.** No clock, no state; the alarm is computed from the observation (`ARC-27`).
- **I-Nc-3 — data, not constants.** Every threshold and window in §4 is a field of the decision list with a bound;
  the defaults are step §6.2's numbers, fixed before any run (I-S35-7).
- **I-Nc-4 — the world decides.** Whether one may sleep here is the `sleep` offer's availability; neither model
  checks a place class.

---

## 4. Design

### 4.1 SD-Nc-1 — the `needs` block of `rule`'s parameters

```yaml
models:
  rule:
    commute: { … }                  # LE-a
    needs:
      eat_from: 600                 # hunger at which a held food is eaten; 1 … 1 000
      tired_from: 600               # tiredness at which bed is chosen in the night window; 1 … 1 000
      exhausted_from: 900           # tiredness at which bed is chosen at any hour; ≥ tired_from
      night: { from: "22:00", until: "06:00" }      # may wrap midnight; from ≠ until
      bedtime_spread_minutes: 60    # 0 … 240; the per-person offset on night.from
      wake_for_commute: true        # set the sleep length so the person is up when a punctual part needs them
```

Decoded with positioned refusals (the IL-a pattern `rule` gains in LE-a C4). Defaults are step §6.2's: 600, 600,
900, 22:00–06:00 (ATUS *Sleeping* ≥ 471 ‰ from 22:00 to 06:00), 60 min, `true`. A world author makes a town of
night owls (`night: { from: "01:00", until: "09:00" }`) without code.

### 4.2 SD-Nc-2 — precedence in `rule::decide`

```text
0  asleep            (SD-Nc-3)  → wake or nothing; nothing below runs
1  due commute       (LE-a)     → head for next.place
2  eat               (SD-Nc-5)
3  invitation, reply (as today)
4  go to bed         (SD-Nc-4)
5  agenda, social, offered, walking roll (as today), with politeness (SD-Nc-6)
```

Rungs 0, 2 and 4 are skipped entirely when `Needs`/`Asleep` are absent (I-Nc-1); rung 1 is LE-a's and inert without
a punctual part. So the base order 3 → 5 is untouched in a world without the packs.

### 4.3 SD-Nc-3 — asleep

Disclosed `Asleep` on me → if `wake_for_commute` and a punctual `next` exists and `t + pace > next.from −
commute::lead(…)` (LE-a's due test, unchanged) → `wake {}`; else `None`. A sleeper answers no line, takes no
initiative and takes **no draw**. The alarm normally makes this rung unnecessary (SD-Nc-4 sets it); the rung is the
safety for a sleep begun before a punctual part was known (a routine change, a resumed world).

### 4.4 SD-Nc-4 — go to bed, with an alarm

Conditions: `sleep` offered available here (the world's verdict), and either `tiredness ≥ exhausted_from`, or
`tiredness ≥ tired_from` and the instant within `[night.from + offset, night.until)` where
`offset = mix(seed, observer) mod (bedtime_spread_minutes + 1)` minutes (no `Draw` index). Then:

```text
for = min(longest,                         sleep's offered bound (disclosed with the offer)
          ceil(tiredness ÷ rest_per_hour × 60) minutes   rest_per_hour: disclosed by needs (N-b) or the default 125
          minutes until next.from − lead   only when wake_for_commute and a punctual next part exists)
for = max(for, shortest)                   sleep's lower bound; a refusal is the world's answer otherwise
```

If the person is tired at night but not at a sleep place, nothing new happens: the agenda already sends most
people home (step §6.2 item 4). **Assumption to confirm at C0:** N-b discloses `rest_per_hour` in force for the
holder (the class-scoped parameter); if not, `rule` uses the step default 125 and the lane lead raises it with N-b.

### 4.5 SD-Nc-5 — eat

`hunger ≥ eat_from` and an available complete `eat` affordance → submit the one whose item has the highest
disclosed `nourishment.hunger` (ties: lowest `ItemId`), through `Affordance::request` as `offered.rs` does. No
draw. A held drink with nourishment > 0 counts as food here (the `eat`/`drink` split is consumption's; the band reads
both affordances).

### 4.6 SD-Nc-6 — politeness

`greet`, `approach` and `invite` skip a person whose `Asleep` is disclosed: the candidate lists in `paced.rs`
(`greet`, `approach`) and `social.rs` (`initiative`) filter them before the existing draws choose. **Inert when
absent:** with no `Asleep` on anyone, the filtered list equals the base list, so the same index is drawn.

### 4.7 SD-Nc-7 — `motivated`

- Rung 0 (asleep): as SD-Nc-3, before the invitation rung. It is a state, not a choice: a sleeper does not score
  options.
- New options, only when their offer is available: `Choice::Sleep` (with `for` computed as SD-Nc-4) and the
  existing `Choice::Offered` for `eat`/`drink`.
- New considerations (step §6.3), each 1 000 when `Needs` is absent:

  | Consideration | Input | Default curve | Carried by |
  | --- | --- | --- | --- |
  | `hungry` | hunger | 0 → 100, 600 → 800, 900 → 1 000 | `Offered` of `eat`/`drink`; `Door` to an `eat` role; `Offered` of `buy` |
  | `sated` | hunger | inverse of `hungry` | (reserved by the step; applied where a score must fall after a meal — the same three) |
  | `tired` | tiredness | 0 → 0, 600 → 700, 900 → 1 000 | `Sleep`; `Door` to a `home` role |
  | `wakeful` | tiredness | inverse of `tired` | `Greet`, `Approach`, `Invite`, `Wander` |

  The curves join `OVERRIDABLE`; `base.sleep` (default 1 000) joins `OPTIONS`. The hour curves `rest` and `eating`
  stay: the hour says when people usually do it, the need says whether this person needs it now.
- `sated` and `hungry` are both applied as the step lists; since `sated` is the inverse, an option carrying both
  would cancel — C4 applies `sated` only to `Offered` of `buy` of a non-food kind (none in MVP-0), so in effect it is
  defined and inert. This is a reading of the step, not a change; Review checks it.

---

## 5. Acceptance (fixed before any run)

| Id | Criterion | Worlds | When |
| --- | --- | --- | --- |
| **AC-Nc-1** (AC-S35-2 controller half) | With neither pack enabled, 30 days, seed 7: every world's `run` output identical to the base (wall line excluded), for an all-`rule` and an all-`motivated` decision list | Market Town, Social Café, Lakeside, Bodies Yard | milestone |
| **AC-Nc-2** | Unit: on synthetic observations, each band fires at its boundary (hunger 599 → no eat, 600 → eat; tiredness 599/600 inside and outside the night window; the bedtime offset; `for` from each of its three limits) and takes no draw when `Needs` is absent (draw-recording double) | — | PR |
| **AC-Nc-3** (AC-S35-9) | With both packs enabled in test copies, `rule`, 30 days × seeds {1, 7, 42}: LE's AC-LE2-1 still holds at 100 % (Market Town 180 of 180) | Market Town copy, `commute-town` copy | milestone |
| **AC-Nc-4** | Smoke, 3 days, seed 7, both packs, all `rule`: every seated person has at least one `fell-asleep` between 22:00 and 02:00 and one food `items-consumed` per day | Market Town copy, `commute-town` copy | PR (fast, ~5 s) |
| **AC-Nc-5** (AC-S35-13 mutation) | "the eat band takes a draw when `Needs` is absent" planted → AC-Nc-1 fails; reverted | — | milestone |

The rhythm numbers of AC-S35-8 belong to N-d; AC-Nc-4 only shows the bands work on a world that is not Market
Town, with no code change between the two (R-LE2-2).

---

## 6. Test fixtures

- **Market Town copy:** a test helper copies `worlds/market-town` into a temp directory and appends `sleep` and
  `needs` to its systems, with `needs: {}` on every person and the step's `nourishment:` values on its food. It is
  the same helper N-d later replaces with real content; the reference world is not edited here.
- **`commute-town` copy:** LE-a's test-only world, with the same additions. Different homes, distances, shift times
  and a manager; nothing in code names it.

---

## 7. Commit plan

Each commit tracks Implementation, Validation and Review separately.

### C0 — Base audit (Markdown only)

- [x] Record the base; confirm N-a/N-b/LE-a names (`Asleep`, `Needs`, offers, nourishment disclosure,
      `rest_per_hour` disclosure, `RuleParameters`, `commute::lead`). Any missing disclosure → stop to the lane lead.
- [x] Implementation · N/A Validation (Markdown) · [x] Review (ledger §10)

### C1 — `rule`: the `needs` parameters

Scope: `cognition/rule-controller/src/{model.rs, parameters.rs (LE-a's)}`, new `needs.rs` (decoders of `Needs`,
`Asleep`, nourishment; the `needs` block with bounds), `Cargo.toml` (types of sleep and needs).
Validation: decoding tests (defaults, each bound, `night` wrap, `exhausted_from < tired_from` refused at its
position); `cargo test -p mineworld-rule-controller`.
- [ ] Implementation · [ ] Validation · [ ] Review

### C2 — `rule`: the bands

Scope: `paced.rs` (precedence of SD-Nc-2; header draw table gains "no draw: asleep, eat, bed"), `needs.rs`
(`asleep`, `eat`, `bed`), `paced.rs`/`social.rs` politeness filters. Validation: AC-Nc-2 unit tests.
- [ ] Implementation · [ ] Validation · [ ] Review (I-Nc-1: no draw before `let Some(needs)`)

### C3 — `motivated`: rung, options, considerations

Scope: `cognition/motivated/src/{model,options,considerations,parameters,curves}.rs`, `parameters_tests.rs`,
`tests.rs`, `README.md`. Validation: parameter decoding; considerations neutral when absent; a synthetic tired
observation scores `Sleep` above `Stay`.
- [ ] Implementation · [ ] Validation · [ ] Review

### C4 — Tests on two worlds

Scope: `tools/cli/tests/needs_controllers.rs` (AC-Nc-4 default; AC-Nc-1 and AC-Nc-3 `#[ignore]` for the milestone),
the copy helper in `tools/cli/tests/fixture/`. Validation: the smoke on both copies.
- [ ] Implementation · [ ] Validation · [ ] Review

### C5 — Docs

Scope: both crates' `README.md` and module headers; `docs/MODULE_SPEC.md` §5 (the `rule` `needs` block, the motivated
curves); this file's ledger. The generated reference regenerated if the parameter schema is in it.
- [ ] Implementation · [ ] Validation · [ ] Review

**Size.** ~0.9–1.1 kLOC: rule ~450, motivated ~300, tests ~250, docs ~80. **M.** Implementer: Opus for C1–C3; Haiku
acceptable for C4–C5 from this plan.

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Sleepers chatted awake all night | SD-Nc-6 politeness; `woken_by_talk` is a world switch (N-a) |
| Eating every consult while hunger stays ≥ 600 | each meal lowers hunger by the item's nourishment, owned by `needs`; a low-nourishment item may be eaten twice, which is correct |
| A person never sleeps because they are not at a sleep place at night | the agenda sends most home at 18:00; a world whose routines never reach a residential place is visible in N-d's AC-S35-8, not hidden here |
| `motivated` loses a sleep draw on a given consult | it is a scored option; the next consult tries again; N-d's rhythm bounds measure it |

---

## 9. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, step-35 PR N-c
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s35-nc-controllers.md
BINDING:                 step-35 §§6, 9, 11; pr-le-leave-early.md revision 2 §0; ARC-27, ARC-34, ARC-84, ARC-282, ARC-286
WORKTREE / BRANCH:       /Users/yuema137/mineworld-worktrees/impl-s35-nc   mvp0/pr-s35-nc   (one session)
START CONDITION:         this design FROZEN; N-b and LE-a merged
APPROVED SCOPE:          §3.1
FROZEN INVARIANTS:       I-Nc-1 … I-Nc-4; §4.1 defaults; §5 thresholds
ALLOWED COMMANDS:        cargo * (per crate; heavy jobs via cast-slots/with-slot.sh); git; gh (pr create/update);
                         python3 scripts/*; target/*/mineworld *
NEVER:                   python3 -c; sed -i; perl -pi; curl/wget; editing kernel/, contracts/, server/, systems/,
                         any reference world; a local full-workspace test run
MATERIAL STOPS:          a missing N-a/N-b disclosure (C0); a default changed after a run; AC-Nc-1 failing
VALIDATION (SPEED FIRST): build, the commit's tests, `fast`; AC-Nc-1, -3, -5 at the milestone
ENDPOINT AUTHORITY:      implementation, commits, push, PR; the lane lead merges after `fast` is green
STOP CONDITION:          PR open, `fast` green, ledger current
```

---

## 10. Ledger (live during implementation)

### Evidence

- 2026-10-18: design written from `origin/main @ a1e05874`.
- 2026-10-19 (implementation session, C0): **implementation base: stacked on `mvp0/pr-s35-nb @ 2f413bc1`** (PR
  #305, on N-a #297) with `mvp0/pr-le-c @ 710a02a7` (LE-a/b/c, PRs #296/#302/#303) merged in as `3b0bdc2b`; the
  merge's only conflicts were the generated `docs/reference/README.md` and `interaction-list.md`, regenerated with
  `mineworld reference --out docs/reference` (`--check` PASS, 37 files), never hand-merged. None of the five PRs is
  merged to main yet; the lane lead rebases after they land.
- C0 names confirmed on the base: `mineworld_sleep::{Asleep, SleepTerms, Sleep, Wake}` — `asleep` disclosed to
  anyone perceiving the sleeper as `{ asleep, since, alarm? }` (alarm to the sleeper only), `sleep-terms`
  `{ shortest_minutes, longest_minutes }` to oneself where located; offers: awake → `sleep` parametric (`for`),
  refused where not allowed; asleep → complete `wake {}` and `sleep` refused `Busy`. `needs` record (holder only):
  `{ hunger, tiredness, words, asleep, rates { …, rest_per_hour }, restore, nourishment: [{ item, hunger }] }` —
  F-Nc-1 and F-Nc-2 are met (nourishment of every held kind and `rest_per_hour` in force are disclosed). Consumption
  offers a complete `eat { item }` / `drink { item }` per held kind. `RuleParameters { commute, boss }` and
  `commute::{lead, leave_by, pending, next_part, holds}` present. No stop.

### Deviations and findings

- **F-Nc-1** (design): the eat band needs each held item's nourishment disclosed to its holder; recorded as a
  requirement on N-b.
- **F-Nc-2** (design): the bed band's sleep length uses `rest_per_hour`; N-b should disclose the value in force
  for the holder, else `rule` uses the step default.
- **F-Nc-3** (C0, bounded): `mineworld-needs` depends on `mineworld-inventory`, a market pack, so a dependency from
  `rule-controller` on it would break `ARC-35` check 2 (no path from `rule-controller` to a market pack;
  `tests/acceptance/tests/ac1_composability.rs`). `rule-controller` therefore depends on `mineworld-sleep` only
  (no market dependency) and reads the `needs` record and consumption's `eat`/`drink` offers by their slugs, as
  LE-b's `boss.rs` reads `employment` (F-LE-13). §2's "adds `mineworld-sleep` and `mineworld-needs` (types only)"
  is read that way; no contract changes.

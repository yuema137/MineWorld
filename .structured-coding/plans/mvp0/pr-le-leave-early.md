# PR LE — Leave home early for work: punctual agenda parts, departure from a travel estimate

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes)

```text
Design revision:        revision 2 (2026-10-18), needs lane lead — §0 below; it supersedes the parts of
                        revision 1 that §0.9 lists
Approved by / evidence: revision 1: the operator, 2026-10-18, in the primary session (Q-LE-1 and the rest as
                        recommended). Revision 2: written to three operator rulings of 2026-10-18, relayed by
                        the coordinator (§0.1); awaiting the freeze of this text
Implementation base:    origin/main at each PR's C0; C0 records the commit
Start condition:        revision 2 FROZEN (merged by the coordinator as the operator's ruling)
Execution contract:     §0.10 (replaces §11)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Read §0 first.** Revision 2 changes who decides when to leave, adds the consequence of being late, and splits the
work into three PRs. Sections 1–13 stay as written where §0 does not replace them; §0.9 lists every replaced part.

---

## 0. Revision 2 (2026-10-18): each controller decides; lateness has consequences

### 0.1 The rulings (operator, 2026-10-18, relayed by the coordinator)

**R-LE2-1 — leaving early is not hard-coded.** Quote: "提前出门上班不应该是一个 hardcode 的行为，而是在不同的人物行动模型下有
不同的机制。比如说如果是 llm 驱动的，人物应该接收到交通大概时间，还有上班时间，自己决定什么时候去上班，而且如果迟到了会收到投诉之类的".
In English: leaving early for work must not be one hard-coded behaviour. Each behaviour model has its own mechanism.
An LLM-driven person receives the rough travel time and the start of work, and decides by itself when to set out.
Arriving late brings a complaint.

**R-LE2-2 — mechanisms are generic.** Quote: "我们不能为了让 default 小世界跑起来，就 hardcode 一大堆东西，我们要把这些机制做的
robust 才行，不然换了一个世界就跑不了了". In English: do not hard-code a pile of things to make the small default world
run; make the mechanisms robust, or they stop working in the next world.

**R-LE2-3 — wage deductions, modular.** Quote: "employment 系统有扣工资机制也很重要，但是应该是模块化的，可以选择开或者不开，可以选择
惩罚力度（如果店没有老板机制那就是玩家设置，如果有老板机制，那就是老板 npc 来决定）". In English: a wage-deduction mechanism matters,
but it must be modular. A world can switch it on or off and choose how severe it is. Where the shop has no boss, the
world's author sets it; where it has a boss, the boss NPC decides.

The coordinator's reading of the rulings, adopted here: the world discloses facts only; each controller kind decides
in its own way (rule: policy data; motivated: a scored option; LLM: decides from the disclosed facts; MCP agents: the
same facts a human gets); consequences are events owned by `employment`; only `economy` moves money; every
acceptance criterion also holds on a second world; mutations run at the milestone (SPEED FIRST, charter).

### 0.2 What stays from revision 1

The diagnosis (§2), SD-LE-1 (`punctual` on a part), SD-LE-2 (the disclosed `next` part), SD-LE-3 (`ways`: length
and doorway count, a lower bound, at most 32), SD-LE-4 (`Consult.stride`), the strides and estimate formula of
SD-LE-5, SD-LE-8 (content), the edge cases (§4.9), I-LE-1, I-LE-2, I-LE-3, I-LE-5, and the baseline rules of §7.

What changes is where the *decision* lives. Revision 1 put one departure rule, with code constants, in a shared
module both in-process models obeyed as a hard first rung. Revision 2 keeps the shared *arithmetic* (how long a way
probably takes) and gives each controller kind its own *decision* about when to set out, configured as data.

### 0.3 The world discloses facts only (SD-LE2-1)

| Fact | Owner | Disclosed to | Change from revision 1 |
| --- | --- | --- | --- |
| `agenda.next { place, label, from, punctual }` | `schedule` | the holder | none |
| `ways [{ place, doors, length }]` | `movement` | the observer, about themself | none |
| `employment.job` shift times, workplace, employer | `employment` | the holder | audit at LE-b C0 whether `Employment` is disclosed to the holder today; if not, LE-b discloses it (holder only) |
| `late-for-shift`, `wage-deduction-due` | `employment` | §0.6 | new |

**Why `ways` carries a length and not a duration.** A duration is the length divided by how often the walker
strides, and that cadence belongs to whoever drives the walk, not to the world: `run` strides every 30 world
seconds, a hosted seat once a wall second (that is `time_scale` world seconds), a human whenever they press a key.
If movement disclosed minutes, it would have to guess the cadence, and the guess would be wrong for every host but
one (RC-3, §2.3). So the world states the distance, and each driver converts it with the cadence it actually uses:

```text
strides(way)  = ceil(length ÷ WALK_STRIDE) + doors + 1          (SD-LE-5, unchanged)
estimate(way) = strides(way) × stride                           stride = the driver's own cadence, world seconds
```

The arithmetic exists twice, once per language, and both copies are pure and tested against the same literal table
(LE-c C1): `mineworld_rule_controller::commute::estimate` (Rust, `stride` from `Consult.stride`) and
`mineworld_sdk.ways.estimate` (Python, `stride` = the session's `time_scale` × the pacer's one wall second). No
departure rule lives in the world, the kernel, the SDK or a client.

### 0.4 Each controller decides in its own way (SD-LE2-2)

**`rule` — a departure policy, as data.** `Rule` gains typed parameters (today `type Parameters = ()`), decoded like
`motivated`'s with positioned refusals (IL-a, `DEP-10`), written in `decisions.yaml` under `models: { rule: … }`:

```yaml
models:
  rule:
    commute:
      mode: plan            # plan | at-boundary. at-boundary = revision 0 behaviour: set out when the part begins
      slack: 500            # ‰ added to the estimate, 0 … 2 000
      margin_minutes: 5     # 0 … 60
      cap_minutes: 120      # 15 … 480; the lead when the place is not in `ways`
      precedence: first     # first | after-replies. first = a due departure is asked before answering a line
```

The defaults are revision 1's numbers (slack 500 ‰, margin 5 min, cap 120 min, first), so a world with no decision
list gets them. The due test is SD-LE-5's, with the constants replaced by these fields. A world author changes a
town's punctuality without code: `mode: at-boundary` brings back the old lateness; `slack: 0` makes people cut it
fine. In a world with no punctual part every field is inert (I-LE-4).

**`motivated` — departure is a scored option.** No hard rung. `options::choices` gains `Choice::Commute(place)`
while a punctual `next` part's place is not where I am and `t < next.from`. Its score is its base weight times a new
consideration `departure` over the input **urgency**, in ‰:

```text
leave_by  = next.from − lead(way)                         lead from the same commute parameters as rule
urgency   = 0                                 if t + 2 × pace ≤ leave_by
          = 1 000                             if t ≥ leave_by
          = 1 000 × (t + 2 × pace − leave_by) ÷ (2 × pace)   between
departure curve (default, overridable)  0 → 0, 500 → 700, 1 000 → 1 000
```

A second new consideration, `unhurried` (the inverse curve of `departure`), multiplies the yes-weight of the
invitation and reply rungs and the scores of `greet`, `approach`, `invite` and `wander`, so that a person about to
be late tends to decline small talk. Both curves are `motivated` parameters (`curve: { departure: …, unhurried: … }`),
the commute block is the same as `rule`'s, and `base.commute` (default 1 000) is a new `base` key. The model stays a
pure function of the observation (`ARC-27`); it takes no new draw index — `Commute` competes in the existing band
draw like any option.

**LLM seats (`lm-controller`) — the model decides from the facts.** This amends P6's I-P6-3 / step-17 I-15 ("a
model decides only social choices") and QP6-3 (a) by ruling R-LE2-1, narrowly: an LM-held seat may now choose to
**set out for a place**; strides stay deterministic.

1. **Facts in the prompt.** `Situation` gains `plans`: for a punctual `next` part, one line in the owner's words —
   *"Your shift at the Café starts at 05:30. Walking there takes about 9 minutes."* — the minutes from
   `mineworld_sdk.ways.estimate`, rounded up; *"I do not know how far it is"* when the place is not in `ways`. For a
   non-punctual next part: *"Next: walk at the park from 14:00."* The lines count against the prompt's existing byte
   budget (`budget.py`). The prompt gives no advice about when to leave.
2. **A trigger to think about it.** A new trigger kind `upcoming`: once when `t ≥ next.from − horizon`, then again
   every `reask_minutes` until the seat sets out, arrives, or `next.from` passes. `horizon_minutes` (default 60,
   15 … 480) and `reask_minutes` (default 15, 5 … 120) are `cognition.toml` controller settings, not code.
3. **A choice to go.** `SocialChoice` gains `act: "go"` with `place` (a label of a place in `plans` or in `ways`);
   `intent.py` builds one `walk-to` through `mineworld_sdk.offers` (the server validates it as for anyone). The
   seat runs the SDK walk pacer (one `walk-step` per wall second while its walk is disclosed, the
   `clients/agent` pacer of step-27 §5.6, moved to or reused from the SDK at LE-c C2). AP6-11 becomes "no `move`;
   `walk-to` only from a `go` choice; `walk-step` only from the pacer".
4. **No hidden auto-departure.** Nothing sets out for an LM seat while the model holds it. If the model never says
   `go`, the person is late, and §0.6 answers.
5. **The fallback** (`rule` with its policy) drives the seat as today whenever it is handed back.

**MCP agents (`clients/agent`).** The `me` view gains the same `plans` lines (`PlansLine` in `models.py`, built in
`disclosed.py` from `agenda.next` and `ways`, rendered in `view.py`), and `late-for-shift` appears in the inbox as
any perceived fact does. The agent walks with `walk-to` as today. It is told nothing a human is not.

**Humans.** The same facts reach the 2D and 3D HUDs in step-35 N-e (`ui.row.agenda-next`). Not in this PR.

### 0.5 Rule and motivated as bosses (SD-LE2-3)

When an employer has a manager (§0.6), the manager's controller is offered `penalize` and decides:

- **`rule`:** policy data `boss: { penalize_from: warning, share: 1 000 }` in the same `models.rule` block —
  `penalize_from` is `notice | warning | final | never` (default `warning`), and the amount is
  `max_amount × share ÷ 1 000` of the offered bound, so a strict boss and a lenient one differ by data. No draw.
- **`motivated`:** `Choice::Penalize(employee)` while offered, scored by a curve `strictness` over the offered
  lateness `count` (default 1 → 100, 2 → 500, 3 → 900) times `base.penalize` (default 600); the amount is the offered
  maximum. It competes in the band draw.
- **LLM:** the `late-for-shift` fact is a new trigger kind `complaint` for the manager's seat, rendered as
  *"Alice was 12 minutes late for her shift at the Café — the third time in 7 days."*; the offer
  `penalize Alice (up to 600)` is listed like any offer, and `attempt` with an `amount` builds it. The model may
  also only speak to Alice.

### 0.6 Lateness and wage deductions in `employment` (SD-LE2-4)

**Late-for-shift — a complaint, always on in a world with `employment`.**

```text
late-for-shift { employee, employer, manager?, shift, late_by, count, level, absent }
  shift      the instant the shift began
  late_by    seconds after shift + grace at which the employee first entered the workplace
             (absent: true and late_by = the shift's length when they never came before shift-ended)
  count      late-for-shift facts for this job within the last window_days, this one included
  level      notice | warning | final        from count against the patience thresholds
  audience   the employee and the manager (holder-style direct audience, not the place)
  emitted    by employment's reaction to the first person-entered-place at the workplace in a shift that began
             with present: false, when its instant exceeds shift + grace; or, for absent, with shift-ended
```

`Employment` keeps the recent lateness instants (at most `final_at`, oldest dropped) and, when a manager may
still penalize, the open lateness (its instant and bound), both serialized only when non-empty
(I-LE-5's precedent), so a world where nobody is late keeps its bytes. `late-for-shift` is **biographical** (a few
per month at most, a life event a person remembers) and therefore reaches memory through the existing biography
path (`ARC-29`). Patience is data in `configure/employment.yaml`, the Interaction List section employment gains
(`ARC-63`), scoped per employer class like every parameter:

```yaml
parameters:
  lateness:
    grace_minutes: 0        # 0 … 60
    window_days: 7          # 1 … 60
    warning_at: 2           # count at which level becomes warning; 1 … 20
    final_at: 3             # ≥ warning_at, ≤ 20
  deduction:
    enabled: false          # the switch of R-LE2-3; false = complaints only, no money moves
    per_late_minute: 10     # minor units, 0 … 10 000
    cap_per_shift: 600      # minor units, 0 … 1 000 000
```

In MVP-0 `final` is a stronger complaint, nothing more: no dismissal (needs are motivation only, step-35 Q-S35-3,
and the same restraint holds here). A world with no `configure/employment.yaml` takes these defaults, which leave
money untouched.

**Wage deduction — modular, off by default.**

- **Off** (`deduction.enabled: false`, the default): only `late-for-shift`. No `penalize` is offered.
- **On, no manager:** the author's numbers decide. With `late-for-shift`, employment also emits
  `wage-deduction-due { employee, employer, amount, cause }`, `amount = min(cap_per_shift, per_late_minute ×
  ceil(late_by ÷ 60))`, `cause` the `late-for-shift` fact.
- **On, with a manager:** the boss decides. Employment emits nothing automatically; it offers the manager
  `penalize { amount }` targeting the employee, from the `late-for-shift` until `window_days` pass or one `penalize`
  is accepted for it. Validation, in order: the actor is that employer's manager (else `PermissionDenied`); an
  unpenalized lateness exists (else `NoSupportedInteraction`); `1 ≤ amount ≤` the same formula's amount (else a
  typed bound refusal naming it). No spatial requirement (a boss can dock pay from anywhere). An accepted `penalize`
  states `wage-deduction-due` caused by it.

**Who is a manager.** A boss is a role a person's job already describes: `job: { …, manages: true }` on one employee
of an employer. At most one per employer, refused at load by name otherwise; a manager's own lateness is complained
about to nobody (`manager` absent) and is never penalized. This adds one optional field to the `job:` section, not a
new pack or section. A world without `manages` has no boss mechanism, so the author's numbers decide.

**Money moves only in `economy`.** `economy` hears `wage-deduction-due` (decoded through employment's published type,
exactly as it hears `wage-due`, `ARC-28`) and moves `amount` from the employee to the employer as a
`money-transferred` caused by it, or records `deduction-unpaid` when the employee cannot pay (nothing moves). Removing
`economy` leaves the fact unanswered, as a `wage-due` is today.

**Removability.** Employment without `configure/employment.yaml`: complaints at the defaults, no money. With
`deduction.enabled: false`: the same, explicitly. A world without `employment`: none of it exists.

### 0.7 Three PRs instead of one

| PR | Scope | Depends on | Size |
| --- | --- | --- | --- |
| **LE-a** world facts and in-process departure | C1 `schedule` (`punctual`, `next`), C2 `movement` (`ways`), C3 `Consult.stride`, C4 `rule` parameters and the commute policy (§0.4), C5 `motivated` `Commute`, `departure`, `unhurried`, C6 content (SD-LE-8), C7 rule acceptance, C8 records and baselines | — | M (~1.7–2.1 kLOC) |
| **LE-b** lateness and deductions | `employment` (`late-for-shift`, patience, `configure/employment.yaml`, `manages`, `penalize`, `wage-deduction-due`), `economy` (answers `wage-deduction-due`), `rule` and `motivated` boss policy (§0.5), records | LE-a's C4/C5 parameter decoders (for the boss block), else independent | M (~1.3–1.7 kLOC) |
| **LE-c** LLM and MCP | `mineworld_sdk.ways.estimate` (+ shared literal table with Rust), `Situation.plans`, the `upcoming` and `complaint` triggers, the `go` choice and the pacer, `penalize` offers for an LM boss, `clients/agent` `PlansLine`, scripted-model tests | LE-a (disclosures), LE-b (complaint) | M (~1.0–1.3 kLOC Python) |

LE-b and LE-c start in parallel worktrees as soon as LE-a's C1–C3 are pushed (they decode `next` and `ways`).

### 0.8 Acceptance (fixed before any run, `ARC-23`; each on Market Town **and** the variant)

**The variant, `commute-town`** — a test-only World Pack at `tools/cli/tests/worlds/commute-town/` (not a reference
world; R-LE2-2): three homes at different distances from two workplaces (walks of roughly 2, 6 and 15 minutes at
`run`'s cadence), two employers, four employees with shifts at 06:00, 07:30, 09:00 and 13:00 (one part across
midnight: a 00:30 night part for the midnight edge), one employer with a manager (`manages: true`) and one without,
`deduction.enabled: true`. Nothing in any crate names it; it only proves the mechanisms do not depend on Market
Town's geometry.

| Id | Criterion | Worlds |
| --- | --- | --- |
| **AC-LE2-1** (replaces AC-LE-1) | `rule`, default policy, 30 days × seeds {1, 7, 42}: every punctual arrival at or before `from`; Market Town **180 of 180** `shift-started` with `present: true`; commute-town all of its shifts present (literal count in the test); zero `late-for-shift` | Market Town, Social Café, commute-town |
| **AC-LE2-2** | `rule` with `commute.mode: at-boundary` (a decision-list fixture): at least one `late-for-shift` per employee per 7 days, each with the right `count` and `level`; with `deduction.enabled: true` and no manager, each followed by a `wage-deduction-due` with the formula's amount and an `economy` transfer caused by it | Market Town (fixture), commute-town |
| **AC-LE2-3** | Boss path, commute-town: a late employee of the managed employer gets `late-for-shift`; a `rule` manager with `penalize_from: notice` submits `penalize` and money moves once; a second `penalize` for the same lateness is refused `NoSupportedInteraction`; an amount above the bound is refused naming it; a non-manager's `penalize` is `PermissionDenied`; each refusal leaves the revision unchanged | commute-town |
| **AC-LE2-4** | `motivated`, all seats, default parameters, 30 days × 3 seeds: on-time share **≥ 900 ‰** of punctual parts (recorded; a miss is a milestone finding for the operator, never a retune) | Market Town, commute-town |
| **AC-LE2-5** | LLM, scripted backend in CI (no live model): a script that answers `go` once the prompt's "takes about N minutes" says it is time (`now ≥ start − 1.5 × N − 5 min`) arrives on time at every shift over 7 days; a script that never says `go` produces a `late-for-shift` with `absent` or `late_by > 0` at every shift, and its next prompt contains the complaint line | commute-town, Social Café (hosted) |
| **AC-LE2-6** | MCP: the `me` view of an employee contains the `plans` line with the same minutes as `mineworld_sdk.ways.estimate` at the session's `time_scale`; the Rust and Python estimates agree on the shared literal table | commute-town |
| **AC-LE2-7** | Generic: commute-town passes AC-LE2-1 … -3 with no code change after Market Town passes them (the variant is added in the same commit as the tests, never tuned afterwards) | commute-town |
| AC-LE-2, -3, -5, -6, -7, -9 | as revision 1 (AC-LE-3's lead read from the policy in force) | as revision 1 |
| AC-LE-4 | as revision 1, plus: no `late-for-shift` or `wage-deduction-due` in Lakeside and Bodies Yard (no employment); their digests unchanged | Lakeside, Bodies Yard |
| **AC-LE-8** (mutations) | M-LE-1 (`lead` returns 0), M-LE-2 (rule precedence after replies), M-LE-3 (`late-for-shift` never emitted), M-LE-4 (a deduction emitted with `deduction.enabled: false`) — each planted, seen to fail, reverted — **run at the milestone**, not per PR (SPEED FIRST) | — |

Per-PR validation under SPEED FIRST: build, the tests the commit writes, and `fast` on the PR. The 30-day runs,
the three-platform determinism and the mutations run at the milestone, in one batch. Baselines (§7): Market Town and
Social Café re-recorded from a run with reasons; Lakeside and Bodies Yard must not move.

### 0.9 Parts of revision 1 replaced

- §4.5 the *constants* (now `rule`/`motivated` parameters); the formula stays.
- §4.6 SD-LE-6: precedence is the `rule` policy's `precedence` field, default `first`.
- §4.7 SD-LE-7: replaced by §0.4's scored option; there is no hard rung in `motivated`.
- §3.2 "LM controller deciding to leave early … not asserted" and "no change to `employment`": replaced by
  §§0.4–0.6.
- I-LE-6: thresholds now in §0.8; defaults still fixed before any run.
- AC-LE-1, AC-LE-8: replaced by AC-LE2-1, AC-LE-8 (four mutations, at the milestone).
- §9 commit plan: C1–C3 and C6 stand as written for LE-a; C4, C5, C7 and C8 are read with §0.4 and §0.8; LE-b and
  LE-c each write their commit plan at their C0 in this file's §13 ledger (a bounded expansion of §0.7's rows), with
  Implementation, Validation and Review tracked per commit.
- §11: replaced by §0.10.
- §12.2 records: ids assigned in §0.11.

### 0.10 Execution contract (replaces §11)

```text
PROJECT / PRs:           MineWorld mvp0, PR LE-a, LE-b, LE-c (one design: this file)
WORKTREES / BRANCHES:    /Users/yuema137/mineworld-worktrees/impl-le     mvp0/pr-le     (LE-a)
                         /Users/yuema137/mineworld-worktrees/impl-le-b   mvp0/pr-le-b   (LE-b)
                         /Users/yuema137/mineworld-worktrees/impl-le-c   mvp0/pr-le-c   (LE-c)
                         one session per worktree
START CONDITION:         revision 2 merged by the coordinator (the operator's freeze)
APPROVED SCOPE:          §0, and §§3.1, 4 as §0.9 leaves them
FROZEN INVARIANTS:       I-LE-1 … I-LE-5; I-LE2-1 the world discloses facts only, no departure rule outside a
                         controller; I-LE2-2 only economy moves money; I-LE2-3 no code names a world;
                         I-LE2-4 deduction off by default, so a world that does not switch it on moves no money
ALLOWED COMMANDS:        cargo *; uv/pytest in the Python packages; git; gh (pr create/update; merge only by the
                         lane lead under SPEED FIRST after `fast` is green); python3 scripts/*; target/*/mineworld *
NEVER:                   python3 -c; sed -i; perl -pi; curl/wget; editing kernel/, contracts/, persistence/,
                         server/ (beyond what the SDK pacer move strictly needs, which stops for the lane lead);
                         any reference world.yaml systems list; a local full-workspace test run
MATERIAL STOPS:          a §0.8 default or threshold changed after a run; a Lakeside or Bodies Yard digest moving;
                         a contract change beyond §0
VALIDATION (SPEED FIRST): build, the commit's own tests, `fast`; acceptance and mutations at the milestone
PLATFORMS:               Linux, macOS, Windows (CI at the milestone)
LIVE DOCUMENTATION:      this file, §13 ledger
ENDPOINT AUTHORITY:      implementation, semantic commits, push, PR create/update; the lane lead merges
STOP CONDITION:          PR open, `fast` green, ledger current
```

### 0.11 Records (ids assigned by the coordinator to the needs lane)

Written as per-file records under `docs/decisions/` (PR #258 split the log; `scripts/check_decision_ids.py`), in
LE-a's C8 (ARC-280 … 282) and LE-b's records commit (ARC-287).

- **ARC-280** — Punctual parts of a day and the disclosed next part (was ARC-LE-a).
- **ARC-281** — `ways`: the way to a place, lengths and doorway counts, never durations (was ARC-LE-b).
- **ARC-282** — Departure is each controller's decision: `Consult.stride`, the shared estimate (Rust and Python),
  `rule`'s policy data, `motivated`'s scored option, the LM seat's `go` choice (amends P6 I-P6-3, step-17 I-15,
  QP6-3) (was ARC-LE-c).
- **ARC-287** — Lateness and wage deductions: `late-for-shift`, patience data, the optional deduction, the manager's
  `penalize`, `wage-deduction-due` answered by `economy`.

### 0.12 Questions (few; the rulings decide the rest)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-LE2-1** [OPERATOR] | R-LE2-1 lets an LM seat choose to walk somewhere, which amends the frozen P6 rule "a model decides only social choices" (I-P6-3, step-17 I-15). Limit it to `go` to a place (strides stay automatic)? | **Yes.** It is what the ruling asks for, and the pacer keeps strides deterministic |
| **Q-LE2-2** | `motivated`'s on-time target: ≥ 900 ‰, recorded, not tuned? | **Yes.** A scored option can lose a draw; 900 ‰ keeps it honest without making it a hard rule |
| **Q-LE2-3** | A boss is the employee whose `job:` says `manages: true` (no new pack). Accept? | **Yes.** Smallest change that gives a world a boss; a richer organization hierarchy is later work |
| **Q-LE2-4** | Default deduction off, defaults 10 per late minute, cap 600 per shift (about 5 hours of Alice's 120/h wage)? | **Yes.** Off keeps every existing world's money unchanged; the numbers only apply where an author turns it on |

**Effort:** `mvp0`, release scope (operator ruling, 2026-10-18, in the primary session).
**Working name:** LE. **Parents:** [`overall.md`](overall.md) (release scope, "Release = all features done");
[`step-19-time-weather.md`](step-19-time-weather.md) §9.2 (FX-24);
[`pr-tw-c-host-clock.md`](pr-tw-c-host-clock.md) F-TWc-1 and ruling R-TWc-FX24;
[`step-10-market.md`](step-10-market.md) DP-5 / I-9 (line 55: "Alice's routine and her shift both start at 05:30,
so she is always late for work").
**Consumed by:** [`step-35-needs-and-sleep.md`](step-35-needs-and-sleep.md) PR N-c (a sleeper's alarm is set from this
PR's departure time).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/needs-early-departure`, branch
`docs/design-needs-and-early-departure`, from `origin/main @ b85f2804`. This session writes this file and the
step-35 file only; edits to shared documents are proposed in §12 for the primary session.

**Binding:** [`CLAUDE.md`](../../../CLAUDE.md) §1 (models removable), §2.2, §4 rules 1, 3, 5, 7, 8, 9, 10, 12, 13,
14, 15; `docs/DECISIONS.md` `ARC-23` (thresholds before runs), `ARC-25` (replay), `ARC-27` (controllers are pure
functions of the observation), `ARC-28`, `ARC-32` (schedule moves nobody), `ARC-35` (AC-1 check 3), `ARC-75`
(walks: route by the world, strides by the sender), `ARC-84` (decision models; `Consult`), `ARC-90` (generated
reference); `docs/ENGINEERING_RULES.md` §§5–9; the memory rules "Realistic defaults" and "Local load: CI is the
gate" (R-LOAD-1).

---

## 1. Goal, in one paragraph

In Market Town no shift has ever begun with its employee present: `shift-started` records `present: false` for
120 of 120 shifts over two 30-day seeds (§2.1). The walk itself takes 6 to 13.5 world minutes; the rest of the
lateness is the departure, which happens only when the work part of the day has already begun, and is then
delayed by whole consult paces while the person answers a line or an invitation. This PR lets a routine say that a
part of the day is **punctual** — the person means to *be there* when it begins — discloses to the person the
next part of their day and an estimate of the way to every place, and makes the two in-process decision models
set out early: at the first consult from which waiting one more pace could make them late, computed from the
estimate plus a stated slack and margin. A due departure then takes precedence over small talk. Content marks
Alice's 05:30 work and Felix's 08:00 store part as punctual. Acceptance is deterministic: every employee present
at every shift start over 30 seeded days in each reference world that has employees, two planted mutations that
must fail it, and baselines re-recorded with honest reasons.

---

## 2. Audit (`origin/main @ b85f2804`, 2026-10-18)

### 2.1 Measured: where the lateness comes from

Measured in this session with a release `mineworld` built today from `c50e6dcb` (a sibling worktree,
`fix-activity-double-draw`; no change to schedule, movement, employment or the rule controller against
`b85f2804`), on `worlds/market-town` from `b85f2804`, by `mineworld run --headless … --save`, then
`mineworld perceived --json` per employee and a throwaway script that pairs each `shift-started` with the
`agenda-changed` at the same instant, the employee's first `walk-started` toward the workplace, and their first
`arrived` there. Seed 1 × 7 days reproduces TW-c's recorded lateness exactly (alice 7, 21, 6, 67, 6, 6, 37 min;
felix 11, 10, 71, 12, 10, 10, 12 min, `pr-tw-c-host-clock.md` F-TWc-1), which corroborates the build. C0
re-measures on the implementation base.

| Run | Employee | Shifts | Present at start | Walk, first stride to arrival | Departure delay after the part begins, in paces of 15 min (count) |
| --- | --- | --- | --- | --- | --- |
| seed 7, 30 days | alice (apartments → café) | 30 | 0 | 6.0 – 9.0 min | 0: 16 · 1: 7 · 2: 5 · 3: 1 · 6: 1 |
| seed 7, 30 days | felix (apartments → store) | 30 | 0 | 10.5 – 13.5 min | 0: 21 · 1: 5 · 2: 3 · 3: 1 |
| seed 42, 30 days | alice | 30 | 0 | 6.0 – 8.5 min | 0: 21 · 1: 5 · 2: 4 |
| seed 42, 30 days | felix | 30 | 0 | 10.5 – 12.5 min | 0: 18 · 1: 5 · 2: 3 · 3: 3 · 12: 1 |

A delayed day, read from Alice's perceived facts (seed 1, day 4, from 05:30): at 05:30, 05:45 and 06:00 she
answers Dev's line (`spoke`, caused by her own action); at 06:15 she accepts Ivan's chat invitation
(`invitation-accepted`, `group-activity-started`); at 06:30 she asks `walk-to` the café and arrives 7.5 min later,
67.5 min late.

### 2.2 The code path

| Where | What it does today | Consequence |
| --- | --- | --- |
| `systems/schedule/src/segment.rs` `Segment { from, place, label }`; `Segments::at` | a part of the day begins at `from`; there is no other time in it | the only instant a routine can name is when the agenda **changes** |
| `systems/schedule/src/system.rs` `wake`, `changed`; `discloses` | at each boundary states `agenda-changed { place, label, until }`; discloses the `Agenda` (the part in force) to its holder; "The Routine is not disclosed" | no observer can know where the **next** part is before it begins |
| `systems/employment/src/system.rs` `wake` | at `job.from` reads `Presence` and states `shift-started { present }` | presence at the instant the shift begins is the fact that is measured |
| `worlds/{social-cafe,market-town}/people/alice.yaml`, `felix.yaml` | Alice's work part begins 05:30, the shift 05:30; Felix's store part 08:00, the shift 08:00 | the commute and the shift start at the same instant, so `present` is false unless the person was already there |
| `cognition/rule-controller/src/paced.rs` `decide` | precedence: answer an invitation; answer an in-window line; **then** the agenda (`agenda::follows`, 90 % per consult) | a line or an invitation at a consult postpones the departure by one pace |
| `cognition/rule-controller/src/agenda.rs` | reads the observer's own `Agenda`; `FOLLOWS_AGENDA = 90` | a further 10 % chance per consult of not setting out |
| `cognition/motivated/src/options.rs`, `considerations.rs` | `Choice::Agenda` toward the agenda in force; `late` consideration on elapsed per-mille of the slot | the same: no lookahead, and replies are hard precedence before options (SD-DLb-6) |
| `systems/movement/src/lib.rs`, `walk.rs` | "A walk takes no calendar time: its pace is how often its sender asks for steps"; `places_to` is a breadth-first place chain; each `walk-step` moves at most `WALK_STRIDE` (1 340 mm) or crosses one doorway | nothing in the world estimates how long a way takes, because the duration is the sender's cadence |
| `tools/cli/src/run.rs` `PACE` 900 s, `RUN_STEP` 30 s | `decide` on the lattice every pace; `step` every 30 s while walking | in `run` a stride takes 30 world seconds (0.045 m/s): Alice's ~12–18 strides are 6–9 min |
| `tools/cli/src/hosted.rs` `PacedSeat` | `decide` every `pace × scale`; a stride once a wall second = `scale` world seconds | the same walk takes a different world time when hosted |
| `cognition/decision/src/model.rs` `Consult { seed, pace, observation }` | the host context a model gets | the model knows its consult period but not the host's stride cadence |
| `tools/cli/tests/fx24.rs` | clause 2 (≥ 80 % of `shift-started` present) fails at every pace and is `#[ignore]`d naming F-TWc-1 | the existing acceptance that this PR must turn green |

### 2.3 Root cause

- **RC-1 — the schedule (primary).** A routine part names one instant, `from`, and the schedule uses it as the
  moment the agenda changes. A person whose day says "at work from 05:30" therefore starts walking at 05:30, and
  arrives one walk late *even with a perfect controller*: the 6.0 – 13.5 min floor of every row above. The schedule
  also discloses only the part in force, so no controller can set out before the boundary even if it wanted to.
- **RC-2 — precedence in the controllers.** Answering a line and answering an invitation are hard precedence over
  the agenda in both models. Each one costs a whole pace (15 min) of departure; the long tails (45, 90, 180 min)
  are such chains.
- **RC-3 — no travel estimate anywhere.** Movement deliberately knows distances, not durations; the duration is
  the host's stride cadence (30 s in `run`, `scale` s hosted), which no model is told.
- **RC-4 — a 10 % per-consult skip** (`FOLLOWS_AGENDA = 90`), minor: P(two consecutive skips) = 1 %.

### 2.4 Why the ruled content-only remedy is not enough

R-TWc-FX24 chose QTW-1 (a), "routine lead time" as content: move each employee's agenda boundary before
`job.from`. Measured against §2.1 it fails three ways:

1. **It is cadence-dependent.** The right lead is strides × stride cadence: about 9 min in `run` (30 s strides)
   and under 4 min hosted at 12× (12 s strides). A fixed 05:15 boundary is right for one host only.
2. **It does not touch RC-2.** 44 of 120 measured shifts set out one or more whole paces late (mostly small talk, §2.1); a
   boundary 15 min earlier still leaves those late.
3. **It breaks the end of the day it shifts.** Moving Alice's boundary to 05:15 is harmless, but the same idea
   applied to "be at the park at 14:00" would cut her shift short; whether a boundary is about arriving or about
   leaving is a property of the part, not of a clock time.

The task this PR answers (operator, 2026-10-18) asks for departure computed from the travel estimate with a
margin; Q-LE-1 asks the operator to supersede R-TWc-FX24 (a) accordingly.

### 2.5 Consumers that move, found by the audit

- `Consult` is constructed in `tools/cli/src/run.rs:246`, `tools/cli/src/hosted.rs:274, 283` and
  `cognition/rule-controller/src/paced_tests.rs:615` (`server/src/runtime/tests.rs` has an unrelated local type).
- The `agenda` record is decoded as `mineworld_schedule::Agenda` (no `deny_unknown_fields`) in
  `cognition/rule-controller/src/agenda.rs`, `cognition/motivated/src/{options,considerations}.rs`; read as JSON by
  `clients/2d/scripts/hud/readers.gd:135` and `clients/agent/src/mineworld_agent/disclosed.py:133`; and by
  `tools/cli/tests/{fx24,routines}.rs`, `tools/cli/tests/social/mod.rs`, `worldpack/tests/social_cafe.rs`.
- The restart tests' quiet window (`tools/cli/tests/fixture/mod.rs` `assert_quiet`, 00:00 + 5 h) reads only
  `from: "HH:MM"` boundaries; hosted restart tests drive no seat, so an early departure commits nothing there.
  C0 confirms.
- `ac1_composability` check 3 compares Market Town's people files with Social Café's: an identical edit to a
  `routine:` line in both keeps it green; an edit in Market Town alone would not. C0 confirms on the base.
- `clients/protocol/evidence/*.log` list the observer's component names; they are evidence, not tests, and a new
  `ways` record appears in a re-recorded one.

---

## 3. Scope

### 3.1 In scope

1. `schedule`: an optional `punctual: true` on a routine part (SD-LE-1); the disclosed agenda carries the next
   part (SD-LE-2). No stored state, event or process changes.
2. `movement`: a `ways` disclosure on the observer — the estimated length and doorway count of the way from where
   they stand to each place they can reach (SD-LE-3).
3. `mineworld-decision`: `Consult.stride`, the host's stride cadence in world seconds (SD-LE-4); `run` and
   `hosted` supply it.
4. `rule-controller`: a public `commute` module — the lead and the due test (SD-LE-5) — and the precedence rule
   (SD-LE-6) in `PacedRuleController::decide`; `motivated` uses the same module with typed, bounded parameters
   (SD-LE-7).
5. Content: `punctual: true` on Alice's 05:30 `work` part and Felix's 08:00 `errands` part, identically in
   `worlds/social-cafe` and `worlds/market-town` (SD-LE-8).
6. Acceptance: `tools/cli/tests/punctuality.rs`; FX-24's clause 2 turned green and its ignore reason reduced to
   run time; mutations M-LE-1 and M-LE-2.
7. Records, specifications, the generated reference, and `scripts/baselines.txt` re-recorded with reasons.

### 3.2 Not in scope

- Rendering the next part and "set out by" in the 2D and 3D HUDs and in the agent/LM views: step-35 PR N-e, which
  touches the same readers for needs.
- A per-person punctuality from S30's `core.punctuality` attribute (step-30 row 69): a follow-up once S30-c
  merges (§13 F-LE-3).
- `run`'s stride cadence (30 world s per 1.34 m stride, 30× slower than real walking): recorded (F-LE-2), not
  changed — the estimate uses whatever cadence the host states.
- Any change to `employment`: a shift remains attendance, read at `from` (QTW-1 (b) is not taken).
- The LM controller deciding to leave early: it sees the same disclosure; its punctuality is not asserted (models
  only propose; tests are LM-independent).
- Hosted (wall-clock) punctuality as a deterministic test: recorded as a smoke check in N-e, not here.

### 3.3 Invariants (proposed for freeze)

- **I-LE-1 — schedule moves nobody** (`ARC-32`). It still states only `routine-assigned` and `agenda-changed`,
  depends on nothing, and its Process wakes exactly at authored boundaries. `punctual` changes no fact it states.
- **I-LE-2 — no rule in a renderer, no duration in movement.** Movement discloses lengths and doorway counts; a
  duration exists only inside a decision model, from the host's stated cadence (`ARC-75`).
- **I-LE-3 — models stay pure** (`ARC-27`): `commute` is a function of `(observation, pace, stride, parameters)`;
  no clock, no state, no new draw.
- **I-LE-4 — no leak into unflagged worlds.** In a world with no punctual part, every decision of `rule` and
  `motivated` is what it was: same requests, same draws, same history. Lakeside's and Bodies Yard's `summary-*`
  and `facts-300s` do not move (AC-LE-4).
- **I-LE-5 — stored bytes of unflagged routines are unchanged.** `punctual: false` is not serialized, so a
  `routine-assigned` without a punctual part has the same payload bytes as today.
- **I-LE-6 — thresholds and defaults are fixed here, before any run** (`ARC-23`): slack 500 ‰, margin 5 min, cap
  120 min, N = 30 days × seeds {1, 7, 42}, 100 % present. A FAIL is a material stop, never a retune.

---

## 4. Design

### 4.1 SD-LE-1 — A punctual part of the day

```yaml
routine:
  - { from: "05:30", place: cafe, label: work, punctual: true }   # be there at 05:30
  - { from: "14:00", place: park, label: walk }                   # set out at 14:00, as today
  - { from: "18:00", place: apartments, label: home }
```

`Segment` gains `punctual: bool`, `#[serde(default, skip_serializing_if = "is_false")]`; the authored schema
(`AuthoredSegment`, `ARC-90`) documents it: *"The person means to be at the place when this part begins, so
whoever drives them sets out early enough to arrive by `from`. Without it, `from` is when they set out."* The
neighbour rule is unchanged (two parts differing only in `punctual` still repeat each other and are refused). The
flag is content the pack carries and never interprets (like the label): schedule's own behaviour does not change.

Why on the part being entered, not on the one being left: an arrival is a commitment of the next part (a shift
begins); a departure belongs to the part in force (a shift ends). Alice's work part is punctual; her park part is
not, so nothing makes her leave her shift early.

### 4.2 SD-LE-2 — The disclosed agenda carries the next part

`ScheduleSystem::discloses` builds, at disclosure time, the stored `Agenda` plus

```text
next: { place: PlaceId, label: AgendaLabel, from: WorldTime, punctual: bool }
```

computed from the holder's `Routine` and the observation instant: the part that begins at the agenda's `until`
(`Segments::at(until)`), with `from = until`. The record keeps the component type `agenda`; the stored `Agenda`
component, its schema version and every fact are unchanged. Disclosed to its holder only (`INV-13`). The Routine as
a whole stays undisclosed; one part of lookahead is what a person keeps in mind ("next: work at 05:30").
A pure read of owned state at disclosure is the movement `Disclosed` and calendar `CalendarLight` precedent.

### 4.3 SD-LE-3 — `ways`: the way to each place, as movement knows it

Movement discloses, on the observer's own entity, a record `ways`:

```text
ways: [{ place: PlaceId, doors: u8, length: Millimetres }]     ascending (doors, PlaceId), at most 32
```

- **Computed at disclosure**, from presence's `Presence` and movement's `Passages`: breadth-first over passages
  (the `places_to` order: neighbours in `PlaceId` order), for each reachable place the chain's doorway count and
  the sum of straight segments — from the observer's position to the first doorway, doorway to doorway through
  each intermediate place (`Passage::there` of the way in to `Passage::here` of the way out), and 0 in the
  destination (a walk to a place ends on entering it, `act::head_for`). A side whose doorway position is not
  modelled contributes 0, so the length is a **lower bound** of the route; detours round furniture or people
  are what the model's slack covers.
- **Bounded:** at most `DISCLOSED_WAYS = 32` entries, the nearest by doorway count, then `PlaceId`. A place beyond
  the bound is simply absent; a model then uses its cap (SD-LE-5).
- **Not a permission and not a route.** Every stride is still checked as today; `walk-to` still plans the route
  through the wayfinders. A model reads `ways` only to know how early to set out.
- **Whether `ways` needs a declared component table** (calendar declares `CalendarLight` though it discloses a
  computed value) is decided at C2 by the kernel's actual rule. If declaring it is required, movement's
  declaration — and so every world's genesis journal row — changes: that is recorded in the baselines' reason
  lines (§7), and Q-LE-3 lists the alternative that avoids it.

### 4.4 SD-LE-4 — `Consult.stride`

`mineworld_decision::Consult` gains `stride: SimDuration`, documented as *"how often the host asks for the next
stride of a walk, in world seconds: the cadence that turns a way's length into a duration"*. `run` supplies
`RUN_STEP` (30 s); `hosted.rs` `PacedSeat` supplies its `step` (`scale` world seconds, one wall second). It is
host context, like `pace` (`ARC-84`); a model that ignores it decides as before.

### 4.5 SD-LE-5 — When a commute is due

`mineworld_rule_controller::commute` (public, shared by both models as `act` and `read` are, SD-DLb-1):

```text
strides(way)   = ceil(way.length / WALK_STRIDE) + way.doors + 1      one per doorway crossing; one for the
                                                                     first step after walk-to is resolved
estimate(way)  = strides(way) × stride
lead(way)      = min(cap, estimate(way) × (1 000 + slack) ÷ 1 000 + margin)
lead(absent)   = cap                                                 the next place is not in `ways`
due at t       ⇔ next.punctual ∧ I am not in next.place
                  ∧ t + pace > next.from − lead                      waiting one more pace could make me late
                  ∧ t < next.from                                    after it begins, the agenda is the part
```

Integer arithmetic only. Defaults (constants for `rule`, parameters for `motivated`): **slack 500 ‰**, **margin
300 s**, **cap 7 200 s**. The `+ doors + 1` terms reflect what §2.1's walks are made of (a doorway crossing is a
stride; the first stride comes one cadence after `walk-to`); the 500 ‰ slack covers wayfinder detours and
`stopped-short` replans; the margin is the realistic few minutes a commuter keeps in hand. They are written here
before any run and are not retuned against a result (I-LE-6). The guarantee the test checks follows directly: a
person who sets out at the first due consult arrives on time whenever the walk takes no longer than
`estimate × 1.5 + 5 min`.

### 4.6 SD-LE-6 — A due commute comes first (rule)

`PacedRuleController::decide` checks, **before** answering an invitation or a line, whether a commute is due; if
so, and if the observer's own walk does not already head for `next.place`, it returns `head_for(next.place)` —
with no draw (draw 13 is not taken, so no other draw index moves). If the walk already goes there it continues
(`Walked::Continue`), and the rest of `decide` runs as today, so a person on the way may still answer a line
between strides. Outside a due window nothing changes: the existing agenda band, its draw and its 90 % stand.
The person says nothing on leaving; an LM seat is free to excuse itself.

### 4.7 SD-LE-7 — The same for `motivated`

`motivated`'s hard precedence (SD-DLb-6) gains a first rung: a due commute, computed by the same function, before
the invitation and reply rungs. Its parameters gain

```yaml
commute: { slack: 500, margin_minutes: 5, cap_minutes: 120 }   # ‰ 0 … 2 000; 0 … 60; 15 … 480
```

typed and bounded like every `motivated` parameter (IL-a positioned refusals). The existing `late` consideration
is unchanged.

### 4.8 SD-LE-8 — Content

`punctual: true` is added to Alice's `{ from: "05:30", place: cafe, label: work }` and Felix's
`{ from: "08:00", place: store, label: errands }` in **both** `worlds/social-cafe/people/` and
`worlds/market-town/people/`, as the identical line (AC-1 check 3, §2.5). In Social Café Alice is the barista and
Felix's store errand has no job behind it; marking them is harmless content and keeps the composition proof's
file identity. Lakeside and Bodies Yard get no punctual part (Lakeside has no employment; Bodies Yard no routine).

### 4.9 Edge cases, decided

| Case | Behaviour |
| --- | --- |
| The next part is at the place I am in | not due |
| The part in force is shorter than the lead | due from its first consult; the person sets out at once |
| No route to `next.place` (`walk-to` refused `no-route`) | the refusal is the world's answer; the next due consult asks again |
| `next.place` absent from `ways` (beyond 32, or no movement) | lead = cap; with no movement nobody walks anyway |
| A walk stalls (`walk-ended stalled`) | the next due consult asks `walk-to` again; the slack and margin absorb one retry |
| Part of a group activity when due | walking out leaves it (`ARC-32` behaviour unchanged); the model asks no `leave` first |
| The world resumes from a save mid-window | stateless: the resumed consult computes the same `due` |
| Midnight wrap (a punctual part at 00:30, departure the evening before) | `Segments::at(until)` handles the wrap; tested |

---

## 5. Acceptance criteria (fixed now, before any run; `ARC-23`)

| Id | Criterion | Counterfactual it must catch |
| --- | --- | --- |
| **AC-LE-1** | `run` of `worlds/market-town` and `worlds/social-cafe`, 30 days, seeds 1, 7 and 42, default pace: for every punctual part of every seated person, the person's first `arrived` at the part's place since the previous boundary is **at or before** `from` — **100 %**, no tolerance; and in Market Town every one of the 180 `shift-started` facts has `present: true`. The test asserts the expected counts (180 shifts; punctual parts per world as a literal table) so a vacuous pass fails | a commute that ignores the estimate; a count of zero passing vacuously |
| **AC-LE-2** | FX-24 (`tools/cli/tests/fx24.rs`) at `--pace` 120, 60, 30: clause 1 holds as before and clause 2 (≥ 80 % present) **passes**; with this PR it is 100 % | the fix working only at the default pace |
| **AC-LE-3** | No early arrival beyond the lead: for every measured commute, `from − arrival ≤ lead + pace` (the person did not set out absurdly early) | a model treating every part as due all day |
| **AC-LE-4** | Lakeside and Bodies Yard (no punctual part): `summary-300`, `summary-1000`, `summary-300s` and `facts-300s` digests in `scripts/baselines.txt` **unchanged**; and at C3 and C4 (before content), all four worlds' 30-day seed-7 `run` output identical to the base (wall line excluded) | logic leaking into unflagged worlds (I-LE-4) |
| **AC-LE-5** | Determinism: two runs and a kill-and-resume run of AC-LE-1's Market Town seed 7 give equal fingerprints, on Linux, macOS and Windows (CI) | `commute` reading a clock or keeping state |
| **AC-LE-6** | A `routine-assigned` with no punctual part has the same payload bytes as on the base (unit test against a literal) | `punctual: false` serialized |
| **AC-LE-7** | Negative inputs: `punctual: yes`, `punctual: 1` and an unknown field on a part are refused at their line and column; `motivated`'s `commute` out of bounds is refused by name and position | silent acceptance of a malformed flag |
| **AC-LE-8** | Mutations, each planted, run, seen to fail AC-LE-1 by name, and reverted: **M-LE-1** `lead` returns 0 (no estimate); **M-LE-2** the due rung moved after the reply rung. A mutation that does not fail is a test defect and a stop | an acceptance test too weak to detect the fix's removal |
| **AC-LE-9** | Unit: `ways` lengths on a hand-built three-place world (positions known, one side unmodelled, an unreachable place, 33 places for the cap); `next` across the midnight wrap and from the last part to the first | arithmetic and ordering errors hidden by the integration runs |

---

## 6. Test ownership and mutations

| Failure class | Owner |
| --- | --- |
| flag decoding, schema, payload bytes | `systems/schedule/tests` (unit, integration through the loader) |
| `ways` arithmetic, cap, disclosure audience | `systems/movement/tests` |
| `due` boundary instants, precedence, no draw taken | `cognition/rule-controller` unit tests with synthetic observations; `cognition/motivated` tests |
| real punctuality, determinism, no leak | `tools/cli/tests/punctuality.rs` (default: Market Town and Social Café, 7 days, seed 1 — about 6 s; `#[ignore]` nightly: 30 days × 3 seeds × 2 worlds — about 70 s) and the nightly long profile |
| FX-24 | `tools/cli/tests/fx24.rs`, still `#[ignore]` for run time (≈ 60 s), reason no longer names a failure |
| real-LLM Gate | **N/A**: no model is involved; LM seats are out of scope (§3.2) |
| platforms | CI `test`, `test-macos`, `test-windows` on the exact head (R-LOAD-1: no local full-workspace run) |

---

## 7. Baselines will move — re-record them honestly

Expected, stated before measuring:

- **Market Town and Social Café:** every `summary-*`, `facts-300s`, `journal-300s` and `sealed-300s` digest moves:
  two people set out earlier on 300 days, so every later fact and event id shifts. Reason line names this PR,
  SD-LE-6 and the content lines of SD-LE-8.
- **Lakeside and Bodies Yard:** summaries and facts **do not move** (AC-LE-4). If C2 must declare a `ways`
  component table, `journal-300s` (and with it `sealed-300s`) moves at the genesis row only, which carries the
  composition — the FG-a precedent; the reason line says exactly that and nothing more.
- Re-recorded from CI's long record on the PR's head (`ci_parity.py baseline write`), never from a local run;
  each section keeps its old reason lines and gains one; the PR body lists every digest that moved and every one
  that did not. A digest that moves where §7 says it does not is a stop, not a re-record.

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| The `ways` length underestimates a geometry-heavy place by more than the slack | AC-LE-1 measures it; the 500 ‰ slack is 1.5× a straight route plus a stride per doorway; a FAIL stops for the operator (I-LE-6) rather than being tuned away |
| Leaving earlier changes who meets whom at 05:00 and moves every Market Town history | expected (§7); re-recorded with reasons |
| Precedence change makes conversations end abruptly at departure | only inside a due window, once a day per punctual part; LM seats unaffected |
| `ways` adds bytes to every observation | ≤ 32 entries of about 60 B; Market Town has 5 |
| A world author marks a part punctual whose predecessor is a shift | the person leaves the shift early; documented in the schema description and the section reference |
| Ruling R-TWc-FX24 (a) not superseded | Q-LE-1 asks before freeze |

---

## 9. Commit plan

Each commit tracks **Implementation**, **Validation** and **Review** separately. `[ ]` until done with evidence;
`N/A` with an audited reason.

### C0 — Freeze record, base audit, re-measurement (Markdown only)

1. **Goal.** Record the freeze and the base; re-measure §2.1 on it; confirm §2.5's consumers.
2. **Scope.** This file's header, §13 ledger. No code.
3. **Implementation plan.**
   - [ ] Record the base commit and freeze reference.
   - [ ] Re-run §2.1 (seed 1 × 7 days; seeds 7, 42 × 30 days) on the base; record the table.
   - [ ] For each commute in that run, record `strides` actually taken versus `strides(way)` of SD-LE-5 computed by
         hand for three of them (pre-registered expectation ≤ 1.5); record, do not tune.
   - [ ] Confirm `ac1_composability` check 3 accepts an identical routine edit in both worlds (read the check).
   - [ ] Confirm the restart and milestone_b hosted tests drive no seat during their quiet windows.
4. **Validation plan.** The runs above; `cargo test -p mineworld-acceptance --test ac1_composability` on the base.
5. **Acceptance.** §2.1 reproduced or its difference recorded; a ratio above 1.5 is a stop before C1.
6. **Failure and edge cases.** A base where FX-24 already passes: stop (the premise changed).
7. **Commands and evidence.** `target/release/mineworld run --headless worlds/market-town --days 30 --seed 7 --save …`;
   `mineworld perceived … --json`.
8. **Commit boundary.** Markdown only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C1 — `schedule`: `punctual` and the disclosed next part

1. **Goal.** RC-1: a routine can say "be there by", and the holder can see the next part.
2. **Scope.** `systems/schedule/src/segment.rs` (`Segment.punctual`, `AuthoredSegment` schema text),
   `src/component.rs` or a new `src/disclosed.rs` (`NextPart`, the disclosed agenda view, public for decoding),
   `src/system.rs` (`discloses`), `src/lib.rs` header and re-exports, `README.md`, `tests/schedule.rs`. Non-goals:
   no new fact, no Process change, no stored component change, no version bump.
3. **Implementation plan.**
   - [ ] `punctual` with `serde(default, skip_serializing_if)`; schema description.
   - [ ] `NextPart { place, label, from, punctual }` from `Segments::at(agenda.until)`.
   - [ ] `discloses` returns the agenda with `next`; holder only.
4. **Validation plan.** Unit: decoding (true, absent, malformed → positioned refusal); `next` at mid-day, at the
   last part (→ the first, next day), across midnight; AC-LE-6 payload bytes against a literal; existing
   `cargo test -p mineworld-schedule` unchanged.
5. **Acceptance.** AC-LE-6, AC-LE-7 (schedule half), AC-LE-9 (`next` half) pass; no other schedule test changed.
6. **Failure and edge cases.** A routine of two parts (next of next is the same part as now): `next` is the other part.
7. **Commands.** `cargo test -p mineworld-schedule`; `cargo clippy -p mineworld-schedule --all-targets -- -D warnings`.
8. **Commit boundary.** Schedule only; the generated reference is regenerated in C8.

- [ ] Implementation · [ ] Validation · [ ] Review (every `Agenda` decoder of §2.5 still decodes the record)

### C2 — `movement`: the `ways` disclosure

1. **Goal.** RC-3, the world half: lengths and doorway counts, not durations.
2. **Scope.** `systems/movement/src/walk.rs` (`ways_from`, sharing `places_to`'s order), `src/component.rs`
   (`Ways`, `Way`), `src/system.rs` (`discloses` for subject == observer, a Person), `src/lib.rs` header table,
   `README.md`, `tests/movement.rs`. Decide and record whether `Ways` must be declared (§4.3).
3. **Implementation plan.**
   - [ ] `ways_from(world, here) -> Vec<Way>`, integer `isqrt` lengths, cap 32.
   - [ ] Disclosure on the observer only (a person learns their own ways, not another's).
   - [ ] Record the declaration decision and its baseline consequence in §13.
4. **Validation plan.** AC-LE-9 (`ways` half); observer-only audience; `cargo test -p mineworld-movement`.
5. **Acceptance.** Lengths equal hand-computed values on the fixture; the 33rd place is absent.
6. **Failure and edge cases.** A place whose `Passages` is missing (no doorway): unreachable, absent.
7. **Commands.** `cargo test -p mineworld-movement`; clippy.
8. **Commit boundary.** Movement only.

- [ ] Implementation · [ ] Validation · [ ] Review (no duration, no clock in movement — I-LE-2)

### C3 — `Consult.stride` and the hosts

1. **Goal.** RC-3, the host half.
2. **Scope.** `cognition/decision/src/model.rs`; `tools/cli/src/run.rs:246`; `tools/cli/src/hosted.rs:274, 283`;
   `cognition/rule-controller/src/paced_tests.rs:615`; the `Consult` doc and `ARC-84`'s note text (C8).
3. **Implementation plan.** - [ ] add the field; - [ ] supply `RUN_STEP` and `PacedSeat::step`; - [ ] fix the test
   construction.
4. **Validation plan.** AC-LE-4 first half: all four worlds, 30 days, seed 7, `run` output identical to the base.
5. **Acceptance.** Byte-identical outputs (nothing reads `stride` yet).
6. **Failure and edge cases.** None beyond compilation.
7. **Commands.** `cargo test -p mineworld-decision -p mineworld-rule-controller -p mineworld-cli --test run`.
8. **Commit boundary.** Plumbing only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C4 — `rule-controller`: `commute` and the precedence

1. **Goal.** RC-2 and RC-4 for `rule`; the due rule of SD-LE-5.
2. **Scope.** New `cognition/rule-controller/src/commute.rs` (+ `commute_tests.rs`), `agenda.rs` (read `next`),
   `paced.rs` (`decide` precedence; draw table header unchanged), `lib.rs` (`pub mod commute`).
3. **Implementation plan.** - [ ] `strides`, `estimate`, `lead`, `due`; - [ ] the first rung of `decide`; - [ ] doc
   the guarantee of §4.5 in the module header.
4. **Validation plan.** Unit on synthetic observations: due exactly at the boundary instant `next.from − lead − pace`
   (not due one second earlier); not due when in `next.place`, when not punctual, when `t ≥ next.from`; no draw
   taken (a draw-recording test double); walk continuing. AC-LE-4 at this commit: four worlds identical to the base
   (no content is punctual yet).
5. **Acceptance.** Unit tests pass; AC-LE-4 holds at C4.
6. **Failure and edge cases.** `ways` absent → cap; `next` absent (a world without schedule) → nothing.
7. **Commands.** `cargo test -p mineworld-rule-controller`; the four 30-day runs.
8. **Commit boundary.** Rule controller only.

- [ ] Implementation · [ ] Validation · [ ] Review (precedence: a due commute never skips an *already-walking*
  continuation; draw indices unchanged)

### C5 — `motivated`: the due rung and its parameters

1. **Goal.** The same behaviour for the second in-process model (SD-LE-7).
2. **Scope.** `cognition/motivated/src/{model,parameters,settings}.rs`, `parameters_tests.rs`, `tests.rs`,
   `README.md`; `examples/social-cafe-all-motivated.yaml` unchanged.
3. **Implementation plan.** - [ ] `commute` parameters with bounds; - [ ] first rung of the precedence.
4. **Validation plan.** Parameter decoding (bounds, positioned refusals — AC-LE-7 second half); a mixed-seat run
   is unchanged with no punctual part.
5. **Acceptance.** Tests pass; no `motivated` fingerprint moves at this commit.
6. **Failure and edge cases.** `slack: 2 001` refused; `cap_minutes: 0` refused.
7. **Commands.** `cargo test -p mineworld-motivated`.
8. **Commit boundary.** Motivated only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C6 — Content: Alice and Felix are punctual

1. **Goal.** SD-LE-8.
2. **Scope.** `worlds/social-cafe/people/{alice,felix}.yaml`, `worlds/market-town/people/{alice,felix}.yaml`: the
   one routine line each, and the comment explaining it; nothing else.
3. **Implementation plan.** - [ ] four identical-line edits; - [ ] comments name `punctual` and this PR.
4. **Validation plan.** `mineworld validate` on both worlds; `ac1_composability` (13/13);
   `cargo test -p mineworld-worldpack --test social_cafe`.
5. **Acceptance.** Both worlds load; AC-1 green.
6. **Failure and edge cases.** A line differing between the two worlds: check 3 fails — fix the line, never the
   check.
7. **Commands.** as above.
8. **Commit boundary.** Content only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C7 — Acceptance: punctuality, FX-24, mutations

1. **Goal.** AC-LE-1, -2, -3, -5, -8 with evidence.
2. **Scope.** New `tools/cli/tests/punctuality.rs` (reusing `headless::{fresh, mineworld}` and `social::decoded`, as
   `fx24.rs` does, reading `shift-started` as JSON so `cognition` and the test stay free of market crates,
   `ARC-35` item 3); `tools/cli/tests/fx24.rs` (ignore reason: run time only).
3. **Implementation plan.** - [ ] the default 7-day test and the ignored 30 × 3 test with the literal expected-count
   table; - [ ] AC-LE-3's early bound; - [ ] mutations M-LE-1, M-LE-2 planted (`// MUTATION M-LE-n`), run, reverted,
   evidence recorded.
4. **Validation plan.** The tests above; FX-24 with `--ignored`; kill-and-resume via the existing `run_restart`
   helper.
5. **Acceptance.** AC-LE-1, -2, -3, -5 PASS; both mutations FAIL AC-LE-1 naming a person, a day and the lateness.
6. **Failure and edge cases.** Any late arrival: a material stop with the decomposition of §2.1 for that day.
7. **Commands.** `cargo test -p mineworld-cli --test punctuality`; `… -- --ignored --nocapture`;
   `cargo test -p mineworld-cli --test fx24 -- --ignored --nocapture`.
8. **Commit boundary.** Tests only.

- [ ] Implementation · [ ] Validation · [ ] Review (the test reads the counts it expects from a literal, never from
  the run)

### C8 — Records, specifications, reference and baselines

1. **Goal.** Documentation law (`CLAUDE.md` §2) and honest baselines (§7).
2. **Scope.** `docs/DECISIONS.md` (the records of §12, numbers from the primary session); `docs/MODULE_SPEC.md`
   §4.1 (the routine section's `punctual`) and §5 (`Consult.stride`); `systems/schedule/README.md`,
   `systems/movement/README.md`; the generated reference (`mineworld reference`, `docs/reference/`);
   `scripts/baselines.txt` from CI's long record; this file's §13.
3. **Implementation plan.** - [ ] records; - [ ] reference regenerated and `reference --check` green; - [ ] baselines
   re-recorded with one reason line per world.
4. **Validation plan.** `cargo test -p mineworld-cli --test reference`; the nightly long profile on the head; §7's
   moved / not-moved list compared against the record.
5. **Acceptance.** AC-LE-4 (second half); every moved digest has a reason; no unexplained move.
6. **Failure and edge cases.** A Lakeside or Bodies Yard summary moving: stop.
7. **Commands.** `python3 scripts/ci_parity.py baseline write <long record> scripts/baselines.txt`.
8. **Commit boundary.** Docs, reference and baselines only.

- [ ] Implementation · [ ] Validation · [ ] Review

**Size.** About 1.6 – 2.0 kLOC including tests: schedule ~250, movement ~300, decision and hosts ~60, rule
controller ~450, motivated ~200, content 8 lines, acceptance ~350, docs ~250. One PR; **M**. Implementer: Opus for
C1–C5 (contracts and precedence), Haiku acceptable for C6–C7 from this plan; the primary session reviews.

---

## 10. Requirements on other work

- step-35 N-c sets a sleeper's alarm from `commute::lead` for a punctual next part.
- step-35 N-e renders `next` and the lead in the HUDs and the agent and LM views.
- The `posture` step (`ARC-147`, reserved): a due commute stands a seated person up through whatever posture
  provides; recorded there.

---

## 11. Execution contract (proposed; filled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, PR LE — leave home early for work
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-le-leave-early.md
RELATED / BINDING DOCS:  step-19-time-weather.md §9.2; pr-tw-c-host-clock.md F-TWc-1, R-TWc-FX24; step-10-market.md
                         DP-5; step-35-needs-and-sleep.md; docs/DECISIONS.md ARC-23, ARC-27, ARC-32, ARC-35,
                         ARC-75, ARC-84, ARC-90; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-le, its own, held by one session
BRANCH:                  mvp0/pr-le, from origin/main
START CONDITION:         this design FROZEN (Q-LE-1 ruled) AND the operator's go
IMPLEMENTATION BASE:     origin/main at C0; C0 records the commit
APPROVED SCOPE:          §3.1, as frozen
FROZEN INVARIANTS:       I-LE-1 … I-LE-6; SD-LE-1 … SD-LE-8; §5 thresholds
SEQUENCE:                C0 … C8
ALLOWED COMMANDS:        cargo *; git; gh (never merge); python3 scripts/*; target/*/mineworld *; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; curl/wget into the repository; editing kernel/, contracts/, persistence/,
                         server/, systems/employment, any world.yaml; a local full-workspace
                         test run under load (R-LOAD-1)
MATERIAL STOPS:          any §5 FAIL; a change to a §4.5 default or §5 threshold after C7 has run once; C0's ratio
                         above 1.5; a Lakeside or Bodies Yard summary moving; a contract change beyond §3.1
PLATFORMS:               Linux, macOS, Windows (CI)
VALIDATION BUDGET:       unit/integration unrestricted per crate; real runs ≤ 20 min total locally; one long
                         parity record from CI at C8
LIVE DOCUMENTATION:      this document (§13)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-le.md
ENDPOINT AUTHORITY:      to be stated at freeze by the primary session (proposed: implementation, semantic commits,
                         branch push, PR creation/update and CI repair authorized; merge never by the
                         implementation session)
POST-MERGE SYNC OWNER:   primary session: overall.md, step-19, MVP_STATUS, progress page; implementation session:
                         this ledger
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 12. Questions and proposed records

### 12.1 Questions (Q-LE-n). **[OPERATOR]** marks operator-material ones

Ruled by the operator on 2026-10-18 in the primary session: every question answered with its recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-LE-1** [OPERATOR] | Supersede R-TWc-FX24 (a) (content-only lead time: move each employee's boundary before `job.from`) with this design (a punctual flag, departure from a travel estimate in the models)? | **Yes.** §2.4: the content remedy is right for one host cadence only, leaves RC-2 (44 of 120 shifts set out a pace or more late) and cannot distinguish arriving from leaving. Fallback (c) of QTW-1 is not needed |
| **Q-LE-2** | Where does "be there by" live: (a) a `punctual` flag on the part; (b) every part arrival-anchored; (c) a separate `arrive:` time per part? | **(a).** (b) makes Alice leave her shift early for the park (§4.1); (c) is a second clock time the author must keep consistent with `from` |
| **Q-LE-3** | `ways` as its own disclosed record on the observer (may need a declared table, which moves every world's genesis journal row), or folded into the disclosed `passages` of the observer's place (no new type)? | **Its own record.** It is the person's sense of the town, not a property of the room's doors; a one-time genesis-row move, recorded, is the honest cost |
| **Q-LE-4** | `Consult.stride` (host context) versus each model assuming 1 s per stride? | **Add the field.** The cadence differs 30× between `run` and a 1× host; assuming it would reintroduce RC-3 |
| **Q-LE-5** | Defaults slack 500 ‰, margin 5 min, cap 2 h, fixed before measurement? | **Accept.** C0 checks the slack against measured walks before any acceptance run; a FAIL stops for the operator |
| **Q-LE-6** | Should a due commute beat answering a line and an invitation? | **Yes**, only inside the due window; between strides the person may still answer |
| **Q-LE-7** | Mark Alice's and Felix's parts punctual in both Social Café and Market Town? | **Yes**, identical lines, so AC-1 check 3 stays green; Lakeside and Bodies Yard unchanged |
| **Q-LE-8** | N = 30 days × seeds {1, 7, 42}, 100 % on time, no tolerance? | **Yes.** 180 Market Town shifts; any late one is evidence the guarantee of §4.5 does not hold |
| **Q-LE-9** | Assert hosted (wall-clock) punctuality here? | **No**; a smoke in N-e. Hosted timing is not deterministic (step-08 §2.3) |

### 12.2 Records proposed (numbers to be assigned by the primary session)

- **ARC-LE-a — Punctual parts of a day.** `schedule` carries `punctual` and discloses one part of lookahead; it
  still moves nobody (amends `ARC-32` by a note).
- **ARC-LE-b — The way to a place, as movement discloses it.** Lengths and doorway counts, never durations;
  bounded; a lower bound (amends `ARC-75` by a note).
- **ARC-LE-c — Host stride cadence in a consult, and the commute rule.** `Consult.stride`; SD-LE-5's formula and
  defaults; precedence (amends `ARC-84` and `ARC-27` by notes).
- **Note on R-TWc-FX24** in `pr-tw-c-host-clock.md` and `step-19-time-weather.md` §9.2, once Q-LE-1 is ruled.

### 12.3 Proposed edits to shared documents (for the primary session)

- `overall.md`: the release-scope entry for LE; the DP-5 known limit closed when LE merges.
- `docs/MVP_STATUS.md` S9 row: DP-5 resolved by LE.
- `mineworld-demos/progress/index.html`: the "known gaps" row for lateness, updated on merge.

---

## 13. Ledger (live during implementation)

### Evidence

- 2026-10-18 (design session): §2.1's runs, release binary from `c50e6dcb`, worlds from `b85f2804`, 2.5 s for 7
  days, about 11 s per 30-day run; scratch only, nothing committed.
- 2026-10-18 (design session), **process slip, recorded:** two corrections in this file (the delayed-shift count
  29 → 44 of 120, in §2.4 and Q-LE-1; and a malformed `NEVER` line in §11) were made with `perl -pi`, an in-place
  stream edit equivalent to the `sed -i` the session's rules forbid. The result was checked by `grep` and is
  correct; later edits used the Edit tool. Implementation sessions use the Edit tool only.
- 2026-10-18: the needs step renumbered from step-34 to **step-35** (`step-34-posture.md` merged as #264);
  cross-references here updated.

### LE-a ledger (branch `mvp0/pr-le`, worktree `impl-le`)

Validation under SPEED FIRST (charter; §0.8, §0.10): builds, the commit's own tests, `cargo clippy -D warnings` on
the changed crates; no local full-workspace run; acceptance runs, mutations and baselines at the milestone.

- **C0 — base.** Branch head `44f050e3` merged with `origin/main @ 73e54621` (2026-10-18). §2.1's re-measurement is
  not repeated (SPEED FIRST; the milestone's acceptance runs measure the result). Consumers of §2.5 confirmed by
  audit: `Consult` is built in `tools/cli/src/run.rs`, `tools/cli/src/hosted.rs` (two sites) and
  `cognition/rule-controller/src/paced_tests.rs`; the `agenda` record is decoded as `Agenda` (no
  `deny_unknown_fields`). Implementation [x] · Validation N/A (Markdown) · Review [x].
- **C1 — schedule.** `Segment.punctual` (skipped when false), `AuthoredSegment` schema text, `NextPart`,
  `DisclosedAgenda` (flattened `Agenda` + `next`), `discloses` to the holder only; tests `systems/schedule/tests/punctual.rs`
  (flag decoding, payload bytes against a literal, `next` mid-day / last → first / across midnight / two parts,
  audience). Implementation [x] · Validation: see "Validation runs" · Review [x] (every `Agenda` decoder still reads
  the record: the test decodes it as `Agenda` too).
- **C2 — movement.** `Way`, `Ways` (declared, table installed, never written), `DISCLOSED_WAYS = 32`,
  `walk::ways_from` in `places_to`'s order, disclosure to the observer about themself; tests
  `systems/movement/tests/ways.rs` (hand-computed lengths, unmodelled sides, unreachable place, the 32 cap, audience,
  no record when nowhere to go). Implementation [x] · Validation: see below · Review [x] (no duration, no clock).
- **C3 — `Consult.stride`.** Field added; `run` supplies `RUN_STEP`, `hosted` the seat's `step`; the one test
  construction updated. Implementation [x] · Validation: build · Review [x].
- **C4 — rule.** `commute` module (policy `Commute`, `strides`, `estimate`, `lead`, `leave_by`, `due`, `pending`,
  `next_part`, `way_to`), `RuleParameters { commute }` decoded by hand with positioned refusals,
  `PacedRuleController::commuting`, precedence `first` / `after-replies`; tests `commute_tests.rs`. Implementation [x].
- **C5 — motivated.** `Choice::Commute`, `Pending`, considerations `Departure` and `Unhurried`, `base.commute`,
  curves `departure` / `unhurried`, the `commute` block; tests `commute_tests.rs`. Implementation [x].
- **C6 — content.** `punctual: true` on Alice's 05:30 and Felix's 08:00 parts, identical lines in Social Café and
  Market Town. Implementation [x].
- **C7 — acceptance.** `tools/cli/tests/punctuality.rs` (default: Market Town and Commute Town, 7 days, seed 1;
  ignored: 30 days × seeds 1, 7, 42 × three worlds; motivated ≥ 900 ‰; `at-boundary` counterfactual), the test
  world `tools/cli/tests/worlds/commute-town/`, the fixture `fixtures/decisions/rule-at-boundary.yaml`, FX-24's
  ignore reason reduced to run time. Implementation [x].
- **C8 — records.** `docs/decisions/ARC-280.md` … `ARC-282.md` and the index; `docs/MODULE_SPEC.md` (routine
  `punctual`, `Consult.stride`); schedule and movement READMEs; the generated reference. Implementation [x].

**Validation runs (local, macOS arm64, each through `cast-slots/with-slot.sh`).**

| Command | Result |
| --- | --- |
| `cargo test -p mineworld-schedule` · clippy `--all-targets -D warnings` | PASS (punctual.rs 5/5) · PASS |
| `cargo test -p mineworld-movement` · clippy | PASS (ways.rs 4/4) · PASS |
| `cargo test -p mineworld-decision` · clippy | PASS · PASS |
| `cargo test -p mineworld-rule-controller` · clippy | PASS (46, commute_tests 8) · PASS |
| `cargo test -p mineworld-motivated` · clippy | PASS (36, commute_tests 4) · PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy -p mineworld-cli --all-targets -D warnings` | PASS |
| `cargo test -p mineworld-cli --test punctuality` (default) | PASS: Market Town 14/14 parts, 14/14 shifts present; Commute Town 35/35, 28/28; 0 too early |
| `… --test punctuality at_the_boundary -- --ignored` | PASS (counterfactual): Market Town 0/14 and Commute Town 0/35 on time with `mode: at-boundary` |
| `mineworld reference --check docs/reference` | PASS after regeneration (33 files) |
| `mineworld validate` Market Town, Social Café, Commute Town | PASS |

The 30-day × 3-seed runs, the motivated ≥ 900 ‰ run, FX-24, determinism on three platforms and the mutations run
at the milestone.

**Deferred to the step-35 milestone (lane rule: no baseline re-recorded in this PR).** Market Town and Social Café:
every `summary-*`, `facts-300s`, `journal-300s`, `sealed-300s` moves (two people set out earlier). Lakeside and
Bodies Yard: `journal-300s` and `sealed-300s` move at the genesis composition row only (movement now declares
`ways`, F-LE-4); their summaries and `facts-300s` must not move.

### LE-b ledger (branch `mvp0/pr-le-b`, worktree `impl-le-b`)

Base: `4c098ff4` (LE-a C1–C3) fast-forwarded to LE-a's head `8f96fc93` (`mvp0/pr-le`, PR #296). Validation under
SPEED FIRST: build, the commit's own tests (`cargo test -p <crate>`), `cargo clippy -p <crate> --all-targets -D
warnings`, `cargo fmt`; no local full-workspace run; acceptance runs, mutations and baselines at the milestone.

**C0 audit.** `Employment` is already disclosed to its holder (`EmploymentSystem::discloses`, holder only), so §0.3's
"disclose the job" needs nothing new. ARC-35 check 2 forbids `rule-controller` (and, by its third bullet, any code
outside `systems/`, `worlds/`, `tests/acceptance/`) from naming a market crate, so the boss policy reads the
`penalize` offer and the manager's disclosed record by slug and JSON, never through `mineworld-employment`. The
Interaction List's parameter block is flat (`parameters!`, ARC-63 item 2), so §0.6's nested sketch is written as
flat fields (F-LE-11).

**Commit plan.**

| C | Scope | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | this plan (Markdown) | [x] | N/A (Markdown) | [x] |
| C1 | `employment`: `job.manages` (one per employer, refused at genesis by name), `late-for-shift` (grace, window, levels, absent), the Interaction List section `configure/employment.yaml` (lateness and deduction parameters, roles actor = employee, target = employer, place = workplace), `wage-deduction-due` (no manager: automatic; manager: only from an accepted `penalize`), the `penalize { employee, amount }` action offered complete to the manager with the bound, the manager's disclosed `late_staff`; version 2; tests `systems/employment/tests/lateness.rs` | [x] | [x] PASS (see runs) | [x] |
| C2 | `economy` answers `wage-deduction-due`: `money-transferred` employee → employer, or `deduction-unpaid`; version 2; tests | [x] | [x] PASS | [x] |
| C3 | controllers: `rule`'s `boss` block (`penalize_from`, `share`) and its no-draw rung; `penalize` kept out of the generic offered band of both models; `motivated`'s `Choice::Penalize`, `strictness` curve, `base.penalize`; tests | [x] | [x] PASS | [x] |
| C4 | content and acceptance: Commute Town's manager (`manages: true` on Ana) and `configure/employment.yaml` (deduction on); boss fixture; `tools/cli/tests/lateness.rs` (AC-LE2-2, AC-LE2-3, AC-LE-4's no-employment clause; Market Town and Commute Town) | [x] | [x] PASS | [x] |
| C5 | records: `ARC-287`, index, `MODULE_SPEC.md`, employment and economy READMEs, generated reference, this ledger | [x] | [x] PASS (reference check) | [x] |

**Validation runs (local, macOS arm64, through `cast-slots/with-slot.sh`).**

| Command | Result |
| --- | --- |
| `cargo test -p mineworld-employment` · clippy `--all-targets -D warnings` | PASS (lateness.rs 6/6; employment.rs and removable.rs updated for the new facts) · PASS |
| `cargo test -p mineworld-economy` · clippy | PASS (deductions.rs 1/1) · PASS |
| `cargo test -p mineworld-rule-controller` · clippy | PASS (49, boss_tests 3) · PASS |
| `cargo test -p mineworld-motivated` · clippy | PASS (37, boss_tests 1) · PASS |
| `cargo test -p mineworld-cli --test lateness --test punctuality` | PASS: default 0 complaints in both worlds; at-boundary Market Town 14 complaints / 2 employees, Commute Town 28 / 4; strict boss 14 told, 14 penalties, each answered by economy; punctuality default unchanged |
| `mineworld reference --check docs/reference` · `mineworld validate` Market Town, Commute Town | PASS after regeneration (34 files) · PASS |

Mutations M-LE-3 and M-LE-4 and the 30-day runs: at the milestone (SPEED FIRST).

**Baselines (deferred to the milestone, not re-recorded here).** Market Town: every digest moves (employment and
economy declare more, and their `SystemVersion`s rise). Social Café, Lakeside, Bodies Yard: no employment and no
economy, so LE-b moves none of their digests.

### Deviations and findings

- **F-LE-1** (design): FX-24's `#[ignore]` reason names F-TWc-1; C7 reduces it to run time.
- **F-LE-2** (design, not fixed): `run` walks 1.34 m per 30 world seconds (0.045 m/s), 30× slower than the 1.34 m/s
  `WALK_STRIDE` is calibrated to; commutes in `run` therefore take ~30× longer than real. LE is correct under any
  cadence; whether `run`'s cadence should become realistic is a separate question for the primary session.
- **F-LE-3** (design): S30's `core.punctuality` (row 69) could scale the margin per person once S30-c merges.
- **F-LE-4** (LE-a C2, recorded per §4.3): perception discloses a record only when an enabled system owns its
  declared type (`mineworld_presence::observe`, `owned_by_an_enabled_system`), so `Ways` is declared with an
  installed, never-written table (the item catalogue's precedent). Movement's declaration therefore changes in every
  world with movement — Lakeside and Bodies Yard included — and their `journal-300s` / `sealed-300s` move at the
  genesis composition row only, exactly the cost §7 and Q-LE-3 accepted. The lane brief listed "a Lakeside or Bodies
  Yard digest moving" as a material stop; this is the move §7 pre-registered, flagged to the lane lead in the report
  (their summaries and `facts-300s` are the digests that must not move).
- **F-LE-5** (LE-a C1, bounded): AC-LE-7 lists `punctual: yes` as refused, but the loader (serde-saphyr, default
  options) reads YAML 1.1 booleans for every flag, so `yes` and a quoted `"true"` are `true` here as for every other
  boolean; a number or another word is refused at its position. Consistency with the loader was kept; changing the
  loader's options would be a world-pack-wide change.
- **F-LE-6** (LE-a C4, bounded): the C4 validation plan says "due exactly at `next.from − lead − pace`", while the
  frozen formula is `t + pace > next.from − lead`, which is due one second later. The formula governs; the test
  asserts not due at `from − lead − pace` and due one second later.
- **F-LE-7** (LE-a C4/C5, bounded): §4.6 says that once the walk already goes to the next part's place "the rest of
  `decide` runs as today". The rest includes the agenda band, which heads for the part *in force* (home), so it would
  turn the walker back. LE-a lets only a line be answered between strides while a commute is due (`rule`), and in
  `motivated` withholds the agenda in force and the doorways while a commute presses (urgency > 0). Nothing else
  changes.
- **F-LE-8** (LE-a C7, bounded): AC-LE-3's per-commute bound `from − arrival ≤ lead + pace` is checked as a literal
  one-hour bound (`EARLIEST = 3 600 s`) fixed in the test before any run, because the test reads facts, not the
  observation a lead was computed from. Commute Town is created here without `manages:` and
  `configure/employment.yaml`, which LE-b's loader introduces; LE-b adds them.
- **F-LE-9** (LE-a C4, bounded): §0.4 sketches the policy as `models: { rule: { commute: … } }`; the decision list's
  model entry takes a model's block under `parameters:` (`models: { rule: { parameters: { commute: … } } }`), as for
  every model. The fixture and `ARC-282` use the real shape; LE-b's `boss:` block goes beside `commute:`.
- **F-LE-10** (LE-a C4/C5, found by the first commute-town run): arriving early at the next part's place, a person
  was called back by the part in force (Alice walked home from the café at 05:15 and back at 05:30), and in a
  two-part routine a person who never left the next part's place stayed there all day. `commute::holds` now treats a
  person at a punctual next part's place, within the cap and one pace of `from`, as at their agenda (no agenda band,
  no doorway); earlier, the part in force has its day. Same rule in `motivated`. Generic, no world named.
- **F-LE-11** (LE-b C1, bounded): the Interaction List's parameter block is flat (`parameters!`, `ARC-63`), so §0.6's
  nested `lateness:` / `deduction:` sketch is written as flat fields (`grace_minutes`, `window_days`, `warning_at`,
  `final_at`, `deduction_enabled`, `per_late_minute`, `cap_per_shift`); `final_at ≥ warning_at` cannot be refused at
  load (no cross-field bound in the SDK) and is read as `max(final_at, warning_at)`.
- **F-LE-12** (LE-b C1, bounded): `penalize` names the employee in its payload (`{ employee, amount }`) and is offered
  target-less, not "targeting the employee": perception asks for targeted offers only of perceived entities, and a
  boss docks pay from anywhere. `wage-deduction-due` carries `shift`; its cause is the envelope's causation (the
  `late-for-shift` or the `penalize`), not a payload field. `Employment` keeps up to 20 lateness instants (the
  largest `final_at`), not `final_at` of them, so a count is exact up to 20.
- **F-LE-13** (LE-b C3, bounded): `rule-controller` and `motivated` may not name the employment crate (`ARC-35` check 2),
  so they read the `penalize` offer and the manager's `late_staff` by slug and JSON (`rule_controller::boss`), and both
  models drop `penalize` from their generic offered band so a penalty is only ever the boss policy's. `share` is
  bounded 1 … 1 000 (a share of 0 would submit an amount the pack refuses).
- **F-LE-14** (LE-b, for LE-c): the shapes LE-c consumes — `late-for-shift { employee, employer, manager?, shift, late_by,
  count, level, absent }` (biographical; audience employee + manager); the manager's disclosed `employment` record
  gains `late_staff: [{ employee, shift, late_by, count, level, bound }]`; the complete offer `penalize { employee,
  amount }` (amount = bound; submit a smaller amount in the same payload).

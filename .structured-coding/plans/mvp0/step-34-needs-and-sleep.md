# Step 34 — Sleep and needs: hunger and tiredness as System Packs

**Role:** step document (step planning phase) for a new MVP-0 step. The step number **S34** is proposed; the
primary session assigns it in [`overall.md`](overall.md). The document contains the ruling, an audit of what exists,
the design of two System Packs (`sleep`, `needs`), how controllers, agents and the two clients observe and act on
them, removability, the PR split with sizes, adversarial acceptance criteria, risks and numbered questions.

**Effort:** `mvp0`, **release scope** (operator ruling, 2026-10-18, in the primary session: needs, planned for MVP-1
as DL-c, move into the MVP-0 release). **Parent:** [`overall.md`](overall.md) ("Framework, not demo" gap list:
"`sleep` and needs do not exist (MVP §5)"; release = all features done).
**Supersedes, once approved:** [`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md)
§6.5 and the DL-c row of §11.2 (QS24-8, "MVP-1"). Step-24's design of the `needs` pack is the starting point and is
kept where this document does not refine it.

**Lifecycle:** `STEP DESIGN (DRAFT)` — awaiting the primary session's and the operator's review. Nothing here
authorizes implementation or a merge. Each PR is detailed to the commit in its own PR design and frozen in turn
(`CLAUDE.md` §3.1).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/needs-early-departure`, branch
`docs/design-needs-and-early-departure`, audit base `origin/main @ b85f2804` (2026-10-18). Written together with
[`pr-le-leave-early.md`](pr-le-leave-early.md), on which PR N-c depends.

**Ids.** Record ids are placeholders (`ARC-N-a` …), to be assigned by the primary session.

---

## 0. Binding inputs

### 0.1 The ruling (operator, 2026-10-18, relayed)

Sleep and needs (hunger, tiredness) are in the MVP-0 release. They were planned as MVP-1
(`mvp1/step-24` §6.5, `overall.md` "Framework, not demo" item 3).

### 0.2 Standing rules that bind this step

- **Kernel ignorance** (`CLAUDE.md` §4 rule 2): the kernel does not know that people sleep or eat. Both are System
  Packs; no kernel or contracts change.
- **Single ownership; events, not mutation** (rules 1, 3): a need level has one owner; another pack's effect on it
  arrives as that pack's fact, heard by the owner.
- **Determinism** (rule 10; `ARC-25`): no randomness outside seeded draws; same seed → same history; replay and
  kill/resume reproduce it.
- **Realistic defaults with sources** (memory rule "Realistic defaults"); the ATUS calibration already committed by
  DL-b (`cognition/motivated/data/atus-a3-2025.tsv`, `NOTICE`).
- **Removability, `AC-2` style**; models removable (`CLAUDE.md` §1); LM-independent tests.
- **Both clients first-class, no rule in a renderer** (`ENGINEERING_RULES.md` §§2–3, 8).
- **Agents are first-class players** (S27): the MCP adapter sees and does what a human can.
- **NPCs to the same standard as the hero**; **all three platforms**; **CI is the gate** (R-LOAD-1).
- **No premature abstraction** (rule 11) and **change amplification** (rule 5).

---

## 1. The requirement, as checkable outcomes

1. Every Person in a world that enables the packs has a hunger and a tiredness that change with world time, and
   sleeping and eating change them back.
2. A person can **sleep** and **wake**, as an action any controller — rule, `motivated`, an LM seat, an agent or a
   human — requests through the same server path.
3. The world, not a controller, decides what a need is now and what restores it; a controller only reads its own
   needs and proposes actions.
4. With no model, the town keeps a recognisable daily rhythm: asleep at night, meals around breakfast, lunch and
   dinner, at work on time.
5. Removing either pack leaves a valid world; a world that never enabled them is byte-for-byte what it was.
6. The 2D and 3D clients show one's own needs and whether someone is asleep, from the same disclosure.

---

## 2. Audit (`origin/main @ b85f2804`)

### 2.1 What exists

| Thing | Where | What it gives this step |
| --- | --- | --- |
| `consumption` System Pack | `systems/consumption` | `eat { item }`, `drink { item }`; states inventory's `items-consumed` through `consume`; offers one complete eat/drink per held food/drink kind, no spatial requirement; owns nothing; README: *"This is the interaction, not a need: nobody gets hungry"* |
| `inventory`, `item` | `systems/inventory`, `systems/item` | `ItemsConsumed { holder, item, count }`; `ItemKind.category` (`food`, `drink`, `goods`) in item files |
| `economy` | `systems/economy` | shops, prices, `buy`; Market Town's café and store sell food and drink |
| `employment` | `systems/employment` | jobs, shifts as attendance, `produces` restock per shift |
| `schedule` | `systems/schedule` | routines and agendas; moves nobody (`ARC-32`); [`pr-le-leave-early.md`](pr-le-leave-early.md) adds `punctual` and the disclosed next part |
| `calendar`, `weather` | `systems/calendar`, `systems/weather` | date, weekday, sunrise; nothing this step needs to change |
| `relationships` FG-a | `systems/relationships/src/decay.rs` | **the precedent for lazy decay**: stored `(value, last_contact)`, current value computed where read, zero facts, zero Process; a configurable curve catalog (`ARC-62` note, `ARC-86`) |
| The World Interaction List | `ARC-63`, `ARC-64` | one `configure/<pack>.yaml` shape: `rules` (permit/forbid by class), `parameters` (typed, bounded, scoped), `consequences` (audience, biography) |
| Sections | `ARC-31` | a pack owns a named section on person and item files; a world that does not enable the pack refuses it by name |
| Decision models | `cognition/decision`, `rule-controller`, `motivated` (DL-a, DL-b) | `Consult { seed, pace, observation }` (+ `stride`, LE); `motivated` already scores `rested` (ATUS *Sleeping*) and `hungry hour` (ATUS *Eating and drinking*) by hour, with step-24 §6.3 naming "needs (MVP-1): hunger pulls toward food affordances" as an input it will read |
| ATUS A-3 2025 | `cognition/motivated/data/atus-a3-2025.tsv` | per-mille of the population **sleeping** and **eating and drinking** in each clock hour; BLS, public domain, credited |
| S30 attributes | `step-30-person-attributes.md` rows 67–69, 73 | `chronotype`, `sleep-need`, `punctuality`, `walking-pace` — learnable levels, to be read by owning Systems "in their own later PR" (§9.4); not merged |
| `posture` | `overall.md` S33 Q-S33-6, `ARC-147` reserved | sitting and standing, server-owned, release-blocking; step document not yet written |
| 2D HUD | `clients/2d/scripts/hud/readers.gd` | one reader per disclosed component (`agenda`, `employment`, `shop`, …); strings in `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po` |
| 3D client | `clients/3d-spike` (`./mineworld-slice`), `scripts/slice/slice_link.gd`, `posture.gd` | reads disclosure through the slice link; a corrective pose layer; animations from the Universal Animation Library |
| Agent client | `clients/agent/src/mineworld_agent/{disclosed,models,view}.py` | a typed line per disclosed component (`AgendaLine` …) for the MCP adapter and the LM controller |

### 2.2 Findings

- **F-S34-1 — nothing is ever hungry or tired.** `consumption` is an interaction without a need; people eat when
  the rule's offered band happens to draw it (draws 14, 15), at any hour.
- **F-S34-2 — no one ever sleeps.** `sleep` is in `docs/MVP.md` §5's interaction list and has no implementation;
  overnight, residents keep greeting and wandering in the apartments.
- **F-S34-3 — supply will not meet three meals a day.** Market Town's two jobs restock about 15 café items and 10
  store items per day (`produces` in `alice.yaml`, `felix.yaml`), against a demand of 12 people × about 3 meals; and
  people without a job live on their endowment (living-world gap L-13). Needs make this visible; content must
  size it (Q-S34-8).
- **F-S34-4 — the decision models are ready for an input.** `motivated` treats every input as optional (neutral at
  1 000 when absent, I-DLb-4); `rule`'s draw table has free indices from 16. Neither needs editing to *ignore* a
  pack that is not installed.
- **F-S34-5 — punctuality interacts with sleep.** A sleeper must wake in time to set out for a punctual part; LE's
  `commute::lead` gives the instant.

---

## 3. Two packs, one owner each

```text
sleep   owns Asleep            whether a person sleeps, since when, and their alarm
        provides sleep, wake   the activity: lie down to sleep at a sleeping place; get up
        emits fell-asleep, woke
needs   owns Needs             how hungry and how tired a person is: stored levels, read lazily
        provides nothing       a need is a condition, not an action
        emits needs-assigned   at genesis only; decay writes nothing
        hears items-consumed   (inventory's) and fell-asleep / woke (sleep's), decoded through their owners'
                               types (ARC-28): no system dependency
```

Why two packs (Q-S34-1): they own different state with different audiences and lives. *Asleep* is a visible
activity that other people react to (they see a sleeper, they wake them by talking); *needs* are a private bodily
condition. A world may want either alone: routine bedtimes with no hunger (a cosy 3D village), or hunger without
modelled sleep (a market day). Each is removable on its own (§7).

Neither pack depends on the other in the registry. `needs` has Cargo dependencies on the *types* of `inventory`,
`item` and `sleep` so it can decode their facts; `sleep` depends on `presence` (where the sleeper is) and has a
Cargo dependency on the *types* of `conversation` (being spoken to wakes a sleeper).

---

## 4. The `sleep` System Pack

### 4.1 Contract

```text
section     none on people; configure/sleep.yaml is its Interaction List section (ARC-63)
provides    sleep { for: minutes }     15 … 840 (14 h); the alarm is now + for
            wake  { }                  get up
emits       fell-asleep { person, place, alarm }        audience: the place's occupants (alarm: holder only,
                                                         carried in the disclosure, not the fact)
            woke { person, slept, reason }              reason: alarm | asked | addressed | acted
owns        Asleep { since, alarm }    on the person; one `sleep` Process per sleeper, ending at the alarm
discloses   Asleep                     `{ asleep: true, since }` to everyone who perceives the person;
                                       `alarm` to the holder only
depends on  presence
hears       presence's arrived; conversation's spoke (types only)
```

### 4.2 Rules, in validation order

1. The actor is a living Person; `sleep` has no target. Otherwise `NoSupportedInteraction`.
2. Already asleep → `Busy` (for `sleep`); not asleep → `NoSupportedInteraction` (for `wake`).
3. `for` within 15 … 840 minutes, else a typed refusal naming the bound.
4. **Where one may sleep** is the Interaction List's: `default: forbid` for `sleep`, with a reference list
   `default` that permits `place: residential` — so a world author permits a `bench` class in a park, or forbids
   sleeping in a host's home, without code (`ARC-63`/`ARC-64`). A forbidden place is `PermissionDenied`.
5. Spatial: the sleeper is in the place (no radius; a bed is a later `bodies`/`posture` anchor, Q-S34-10).

### 4.3 How sleep ends

| Trigger | Mechanism | `reason` |
| --- | --- | --- |
| the alarm instant | the sleeper's `sleep` Process wakes | `alarm` |
| the sleeper requests `wake` | resolve | `asked` |
| somebody talks to the sleeper | reaction to `spoke` whose listener is the sleeper, when the section parameter `woken_by_talk` (default `true`) holds | `addressed` |
| the sleeper acts: walks or speaks | reaction to the sleeper's own `arrived` or `spoke` | `acted` |

The last row keeps one truth without any other pack knowing about sleep: `movement` and `conversation` do not
refuse a sleeper's request (that would make them know what sleep is); if a controller makes a sleeper walk, the
walk is real and the person is therefore awake. `slept` is integer seconds.

### 4.4 Parameters (`configure/sleep.yaml`, `parameters:`)

| Field | Bound | Default | Source |
| --- | --- | --- | --- |
| `woken_by_talk` | bool | `true` | — |
| `longest_minutes` | 60 … 840 | 840 | an upper bound, not a norm |
| `shortest_minutes` | 15 … 120 | 15 | a nap |

Not biographical (thousands of facts would bury a biography; the step-10 QS-49 precedent), configurable per
`ARC-63` `consequences` for a world that wants it.

---

## 5. The `needs` System Pack

### 5.1 Contract

```text
section     needs:        on a person file — initial levels; `{}` for the defaults (Q-S34-5)
            nourishment:  on an item file — what eating or drinking one does: { hunger: 0 … 1 000 }
            configure/needs.yaml — its Interaction List section: rates, thresholds (parameters, class-scoped)
emits       needs-assigned { person, levels }   genesis, from the person's section; nothing after
owns        Needs { hunger, tiredness }         on the person; each a stored (level, at, mode)
discloses   Needs                               to its holder only: { hunger, tiredness, words }
depends on  nothing (registry); types of inventory, item, sleep
```

### 5.2 Levels, lazily

Each need is an integer **deficit** in 0 … 1 000 (0 = sated / fully rested, 1 000 = famished / exhausted). Stored:
`(level, at, mode)` where `mode` is `awake` or `asleep`. The **current** level is a pure function, computed
wherever it is read — at disclosure, in a reaction — exactly FG-a's pattern:

```text
current(need, now) = clamp(level + rate(need, mode) × (now − at) ÷ 3 600, 0, 1 000)
rate(hunger,    awake)  = +hunger_per_hour            rate(hunger,    asleep) = +hunger_asleep_per_hour
rate(tiredness, awake)  = +tiredness_per_hour         rate(tiredness, asleep) = −rest_per_hour
```

A reaction folds first: it computes the current levels at the fact's instant, applies the fact, and stores
`(new level, fact instant, new mode)`. So:

- decay writes **zero facts, zero journal rows, starts no Process** (AC-S34-5);
- replay and kill/resume reproduce every value, because it is a function of stored state and world time (`ARC-25`);
- integer arithmetic only; the workspace has no float.

### 5.3 What changes a need — reactions to other packs' facts

| Fact (owner) | Effect | Notes |
| --- | --- | --- |
| `items-consumed` (inventory), holder a Person with `Needs` | hunger −= the item's `nourishment.hunger` × count | the amount is the item file's `nourishment:` section, owned by `needs` (`ARC-31`); an item without one nourishes 0 |
| `fell-asleep` (sleep) | mode := asleep | tiredness now falls at `rest_per_hour` |
| `woke` (sleep) | mode := awake | |

The section names which restorers a world uses, as a closed enum per need —
`restore: { hunger: [eating], tiredness: [sleeping] }` (both the default) — so a restorer whose pack the world does
not enable is refused by name at load (`eating` needs `consumption`; `sleeping` needs `sleep`), FG-a's "the
section requires it" precedent, rather than leaving a need that silently never falls.

These are a **closed, typed** set in N-b. A third restorer (a shower for hygiene, a bed of a given quality) is the
point at which an `ARC-62` catalog earns its place (Q-S34-4); two do not.

### 5.4 Thresholds and words

Disclosed with the levels so that every reader — a controller, an LM prompt, an agent, a HUD — uses the same words,
decided by the owner, never by a renderer:

| Need | `ok` | `peckish` / `sleepy` | `hungry` / `tired` | `famished` / `exhausted` |
| --- | --- | --- | --- | --- |
| hunger | < 400 | 400 … 599 | 600 … 899 | ≥ 900 |
| tiredness | < 400 | 400 … 599 | 600 … 899 | ≥ 900 |

### 5.5 Defaults, with sources (`configure/needs.yaml` `parameters:`)

| Parameter | Bound | Default | Derivation and source |
| --- | --- | --- | --- |
| `tiredness_per_hour` (awake) | 10 … 500 | 60 | 1 000 over ~16.7 h awake: the waking day implied by a ~8 h night |
| `rest_per_hour` (asleep) | 10 … 1 000 | 125 | 1 000 over 8 h. ATUS A-3 2025's *Sleeping* row sums to 8 826 ‰·h, i.e. **8.8 h/day** on average for the population aged 15+ (committed table); the AASM/SRS consensus recommends **≥ 7 h** for adults (Watson et al., *Sleep* 38(6), 2015 — secondary, re-verified at N-b's freeze) |
| `hunger_per_hour` (awake) | 10 … 500 | 120 | from sated to `hungry` (600) in 5 h: ATUS *Eating and drinking* peaks at 08:00 (76 ‰), 12:00 (144 ‰) and 18:00 (153 ‰), 4–6 h apart (committed table) |
| `hunger_asleep_per_hour` | 0 … 500 | 30 | authored: hunger is suppressed in sleep, so breakfast follows a night rather than a famine; stated as authored, not as physiology |
| thresholds | ordered, 1 … 1 000 | 400 / 600 / 900 | authored, fixed here |
| `nourishment.hunger` per item (Market Town content, N-d) | 0 … 1 000 | soup 600, sandwich 650, bread 500, cake 400, croissant 350, apple 200, juice 100, milk 150, coffee 0, tea 0 | authored for a small town's menu: a full meal restores most of a 5-hour hunger; stated as authored |

Defaults are scoped by class like every IL parameter (`{ actor: night-shift, … }`). Person variation comes from
content (the `needs:` section's initial levels) and, later, S30's `chronotype` and `sleep-need` (Q-S34-11). No
randomness is used anywhere in either pack.

### 5.6 Genesis

A world hosted from genesis starts at 00:00. Default initial levels are therefore those of a person at midnight:
hunger 300, tiredness 700 (sleepy). Content may set others per person.

---

## 6. Controllers, agents and players

### 6.1 What they observe

| Observer | Reads | Through |
| --- | --- | --- |
| the person themself | `Needs { hunger, tiredness, words }`; own `Asleep { since, alarm }` | the observation (holder-only disclosure, `INV-13`) |
| anyone who perceives the person | `Asleep { asleep, since }` | the observation |
| nobody else | another person's needs | — (a stranger cannot see that Alice is hungry; they can see that she is asleep) |

Offers: `sleep` is offered (complete, with `for` left to the requester — a parametric affordance like `walk-to`)
where the Interaction List permits it; `wake` while asleep; `eat`/`drink` as today.

### 6.2 `rule` (deterministic; PR N-c)

New bands, all taking **no draw** except where stated, and all **inert when `Needs` is not disclosed**, so a world
without the packs decides exactly as before (F-S34-4; AC-S34-2):

1. **Asleep:** ask nothing but `wake` when a punctual next part's commute is due (LE) — otherwise nothing. A sleeper
   answers no line and takes no initiative.
2. **Eat:** hunger ≥ 600 and a food item held → `eat` the held kind with the highest `nourishment` (ties: lowest
   `ItemId`). Before the social bands, after a due commute.
3. **Go to bed:** at a place where `sleep` is offered available, tiredness ≥ 600 and the hour within the night
   window 22:00 – 06:00 (from ATUS *Sleeping* ≥ 471 ‰ from 22:00 to 06:00, committed table), or tiredness ≥ 900 at
   any hour → `sleep { for }` with
   `for = min(longest, minutes to rested (tiredness ÷ rest_per_hour), minutes until the next punctual part's
   departure (LE's lead))`. Bedtimes spread by a seeded per-person offset `mix(seed, observer) mod 60` minutes on
   the window's start — the only draw, a fixed index (16).
4. **Head home to sleep:** tiredness ≥ 600 in the night window and not at a sleep place → the agenda already sends
   most people home at 18:00; if it does not, nothing more (no new destination logic in `rule`).
5. **Politeness:** `greet`, `approach` and `invite` skip a person disclosed `Asleep`.

`rule` does not shop for food: when hungry with nothing to eat it relies on the existing offered band (which draws
`buy` at a shop). `motivated` shops deliberately.

### 6.3 `motivated` (PR N-c)

New considerations, each neutral (1 000) when `Needs` is absent (I-DLb-4):

| Consideration | Input | Curve (authored, overridable) | Applies to |
| --- | --- | --- | --- |
| `hungry` | hunger | 0 → 100, 600 → 800, 900 → 1 000 | `eat` offers; doors to `eat` places; `buy` of a food kind |
| `sated` | hunger | inverse | the same, when the score must fall after a meal |
| `tired` | tiredness | 0 → 0, 600 → 700, 900 → 1 000 | `sleep`; doors to `home` places |
| `wakeful` | tiredness | inverse | greet, approach, invite, wander |

The existing hour-based `rested` and `hungry hour` stay: the hour says when people usually do it, the need says
whether this person needs it now. The same precedence rungs as `rule` for "asleep" and "due commute".

### 6.4 LM seats (P6 `LMController`) and agents (S27 MCP)

- `clients/agent` gains `NeedsLine` and `AsleepLine` in `disclosed.py`/`models.py`/`view.py`; the MCP `me` view and
  the LM prompt render them in the owner's words ("I am hungry; I am sleepy"), within the prompt's byte budget.
- They act through the same affordances (`sleep`, `wake`, `eat`, `buy`); the server validates as for anyone.
- The LM seat's `fallback` (`rule`) carries the bands of §6.2 when the model is absent or dead (AC-DL-5's path).
- Tests use recorded cassettes; no live model (rule 10).

### 6.5 Human players

Needs apply to every Person (`INV-1`: a seat is not a kind of person). A human sees their own needs in the HUD and
gets `Sleep`/`Wake up`/`Eat …` buttons from the offers. There is no time skip in a shared world (Q-S34-9); a human
who sleeps sees the world go on with a dimmed view until the alarm or `Wake up`. Needs have no forced consequence in
MVP-0 (Q-S34-3), so a player who ignores them loses nothing but a comfortable HUD.

---

## 7. Removability (`AC-2` style)

| Removed | With | Result |
| --- | --- | --- |
| `needs` | its `configure/needs.yaml`, every `needs:` and `nourishment:` section | the world loads and runs; every controller decides exactly as in a world that never had it; `fell-asleep`/`woke` still happen |
| `sleep` | its `configure/sleep.yaml` | loads and runs; `needs` still rises and is restored by meals; the `needs` section's `tiredness: [sleeping]` restorer (§5.3) is **refused by name at load** when `sleep` is not enabled; with `restore: { tiredness: [] }` stated explicitly, tiredness only rises (FG-a's precedent: "the section requires it, refused by name otherwise"), so a tired-forever world is never silent |
| `consumption` | — | `needs` loads; a world with `nourishment:` sections but no `consumption` is refused by name (nothing could ever restore hunger) |
| both | all of the above | byte-identical to the base world (AC-S34-2) |

A `needs:` section in a world that does not enable `needs` is refused by name (`ARC-31`), as `holdings:` is without
`inventory`.

---

## 8. Presentation

| | 2D (`clients/2d`) | 3D (`./mineworld-slice`) |
| --- | --- | --- |
| own needs | HUD rows from the `needs` record: `ui.row.needs.hunger` / `.tiredness` with the owner's word and a 5-step bar | the same rows in the slice HUD panel, same message ids |
| asleep (self) | HUD row `ui.row.asleep` "Asleep until 04:55", a `Wake up` button from the offer; the view dimmed | the same row and button; the camera dims |
| asleep (others) | a "Zz" glyph on the person's token, from `Asleep` | the sleeping pose from the Presentation Pack's mapping `asleep → sleep`; fallback, when the animation library has no lying clip, the idle pose with a floating "Zz" label (audited at N-e's C0) |
| next part (from LE) | `ui.row.agenda-next` "Next: work at the Café, 05:30" | the same |
| strings | `presentation/mineworld-default/2D/i18n/en.po`, `zh_Hans.po` | the same message ids (one Presentation Pack; the client-parity rule of `overall.md` "One world, two views") |

Rules: the clients render disclosed words and states; neither computes a threshold, a level or whether one may
sleep (`check_client_rules.py`). The 3D sleeping pose is a presentation mapping of a disclosed state, and the
`posture` step (`ARC-147`) records that `Asleep` overrides any posture in presentation (Q-S34-10).

---

## 9. Acceptance and adversarial criteria (step level; each PR fixes its thresholds before running)

| Id | Criterion | Counterfactual it must catch |
| --- | --- | --- |
| **AC-S34-1** | Determinism: Market Town with both packs, 30 days, seed 7 — two runs and a kill-and-resume run give equal fingerprints on Linux, macOS and Windows | a lazy value computed from a host clock; a reaction order that depends on a hash map |
| **AC-S34-2** | Removal: (a) every reference world that does not enable the packs keeps its `scripts/baselines.txt` summaries and facts digests unchanged by every S34 PR; (b) Market Town with both packs removed (with their files and sections) has the same `run` output as the base | a controller band that takes a draw or changes precedence when `Needs` is not disclosed |
| **AC-S34-3** | Refusals at load, by name and position: `needs:` without `needs`; `nourishment:` without `consumption`; the tiredness restorer without `sleep`; a threshold order 600 < 400; a rate out of bound; `nourishment.hunger: 1 001` | a silently ignored or silently inert configuration |
| **AC-S34-4** | Action refusals: `sleep` outside a permitted place → `PermissionDenied`; `sleep` while asleep → `Busy`; `wake` while awake → `NoSupportedInteraction`; `for: 5` → bound refusal; each leaves the world unchanged (revision equal) | a controller-side check that the server does not repeat |
| **AC-S34-5** | Lazy decay writes nothing: over 30 days of Market Town, `needs` states exactly one `needs-assigned` per person and no other fact; no `needs` Process exists | a decay Process or a threshold fact on a timer |
| **AC-S34-6** | Audience: Bob's observations over a day never contain Alice's `Needs`; a sleeper's `Asleep` is in the observation of everyone in the place and of no one elsewhere; `alarm` only in the sleeper's | a disclosure that widens to the place |
| **AC-S34-7** | Sleep integrity (property, 30 days): per person, `fell-asleep` and `woke` strictly alternate; every `woke.slept` equals the instants' difference; a sleeper who walks is woken with `acted` at that `arrived` | a second truth about who is asleep |
| **AC-S34-8** | Rhythm, rule controller, Market Town, 30 days × seeds {1, 7, 42}, seated people: share of person-time asleep at 03:00 ≥ 850 ‰ (ATUS 949 ‰); at 14:00 ≤ 100 ‰ (ATUS 40 ‰); mean sleep per person per day in [7.0 h, 9.5 h] (ATUS 8.8 h; AASM ≥ 7 h); total-variation distance between the hour-of-day histogram of food `items-consumed` and the normalised ATUS *Eating and drinking* row ≤ 0.35; no seated person at hunger ≥ 900 for more than 24 consecutive hours | curves not applied; food unreachable for lack of stock or money (F-S34-3) |
| **AC-S34-9** | Punctuality survives sleep: with both packs, LE's AC-LE-1 still holds at 100 % (180 of 180 shifts present) | an alarm that ignores the commute |
| **AC-S34-10** | Clients: the 2D and 3D clients render the same rows for the same observation (an AC-13-style semantic check over a recorded observation), and `check_client_rules.py` finds no threshold or level arithmetic in either | a renderer deciding "hungry" from a number |
| **AC-S34-11** | Agents and LM: the MCP `me` view and the LM prompt contain the needs words; an agent's `sleep` request is validated exactly as a rule seat's; a cassette replay of an LM seat gives identical requests; with the cognition process killed the seat's fallback sleeps and eats by §6.2 within one consult | a separate path for agents; a fallback that ignores needs |
| **AC-S34-12** | Save growth: Market Town 300-day saved run with both packs keeps AC-SAVE-1's bound (≤ 0.25 MiB/day, days 100–300) | sleep facts per stride rather than per sleep |
| **AC-S34-13** | Mutations, each planted and seen to fail its test, then reverted: decay applied twice in a fold (AC-S34-1/-8); `Needs` disclosed to all (AC-S34-6); the rule's eat band taking a draw when `Needs` is absent (AC-S34-2) | tests too weak to detect a broken owner |

---

## 10. PR split

### 10.1 The PRs

| PR | Scope | Depends on | Integration checkpoint | Adversarial focus | Size | Implementer |
| --- | --- | --- | --- | --- | --- | --- |
| **DOC-S34** | records (ARC-N-a … e), `MODULE_SPEC.md` §§4.1–4.2 entries for `needs:`, `nourishment:`, `configure/sleep.yaml`, `configure/needs.yaml`; `docs/MVP.md` §5 note; supersede step-24 §6.5/DL-c | this step approved | specs reviewed | terminology: "need", "asleep" are pack vocabulary, not core concepts (no `CORE_CONCEPTS.md` change) | S (~400 lines Markdown) | Opus |
| **N-a** `sleep` | `systems/sleep`: `Asleep`, `sleep`/`wake`, Process, `fell-asleep`/`woke`, IL section, reactions to `arrived` and `spoke`, disclosure, offers; installed set; tests incl. AC-2 at pack level | DOC-S34 | a scripted seat in a test world sleeps, is woken by talk, by its alarm and by walking | AC-S34-4, -6 (asleep half), -7 | M (~1.0–1.3 kLOC) | Opus |
| **N-b** `needs` | `systems/needs`: `Needs` lazy levels, `needs:` and `nourishment:` sections, IL parameters with bounds, reactions to `items-consumed`, `fell-asleep`, `woke`; load-time refusals; disclosure with words | N-a | a scripted seat eats and sleeps; levels at chosen instants equal hand-computed values, and equal after kill/resume | AC-S34-3, -5, -6 (needs half), mutation "decay twice" | M–L (~1.4–1.8 kLOC) | Opus |
| **N-c** controllers | `rule` bands (§6.2), `motivated` considerations (§6.3), alarm from LE's `commute::lead`; politeness to sleepers | N-b, **LE** | Market Town enabled in a test copy: people sleep at night, eat around meals, are on time | AC-S34-2 (controller half), -9, mutation "draw when absent" | M (~0.8–1.0 kLOC) | Opus |
| **N-d** content and acceptance | Market Town enables `sleep` and `needs` (appended to `systems:`, `configure/`, sections on people and food items), sized supply and prices (F-S34-3), rhythm and determinism tests, baselines re-recorded with reasons | N-c | the 30-day × 3-seed rhythm run | AC-S34-1, -2 (world half), -8, -12 | M (~0.3 kLOC YAML, ~0.5 kLOC tests) | Opus for sizing, Haiku for tests from the plan |
| **N-e** presentation and views | 2D HUD rows and Zz glyph; 3D slice HUD rows, sleeping pose or fallback; `en`/`zh_Hans` strings; `clients/agent` lines; LM prompt rendering; LE's next part in both HUDs; a hosted punctuality smoke | N-b (rows), N-d (a world to show) | both clients and the MCP view show the same words for Alice; screenshots for the operator | AC-S34-10, -11 | M (~0.8–1.0 kLOC GDScript/Python, ~50 strings) | Haiku for GDScript and Python from a commit plan; Opus review |

**Total:** about 4.7 – 6.0 kLOC over six PRs (five code PRs and one specification PR), plus LE (~1.6 – 2.0 kLOC).

### 10.2 Order and parallelism

```text
DOC-S34 ──► N-a ──► N-b ──► N-c ──► N-d ──► (release)
                      │       ▲
                      │      LE (independent of DOC-S34; may run in parallel with N-a, N-b)
                      └────────────► N-e (after N-b for rows; after N-d for screenshots)
```

LE can start as soon as its design is frozen; N-c waits for LE. N-e's 2D and 3D parts can run in parallel worktrees
once N-b merges.

### 10.3 Integration checkpoint of the step

`run --headless worlds/market-town --days 30 --seed 7` with both packs: AC-S34-8's numbers within bounds, AC-S34-9's
180 of 180 shifts present, and the operator can host the town, watch residents go home and sleep, see Alice wake and
set out for the café, and see their own hunger in the 2D and 3D HUDs.

---

## 11. Invariants (proposed)

- **I-S34-1** The kernel and contracts are unchanged.
- **I-S34-2** `needs` writes only `Needs`; `sleep` writes only `Asleep`; neither states another pack's fact.
- **I-S34-3** Decay is lazy; neither pack has a periodic Process; `needs` states facts only at genesis.
- **I-S34-4** No randomness in either pack; controllers' draws are seeded and indexed.
- **I-S34-5** A world without the packs is byte-identical to the base.
- **I-S34-6** No client computes a need, a threshold or a permission.
- **I-S34-7** Defaults are fixed in each PR design before its first run and are never retuned against a run
  (`ARC-23`); a failed acceptance stops for the operator.

---

## 12. Risks

| Risk | Mitigation |
| --- | --- |
| Food supply and wallets drain over 300 days (F-S34-3, L-13) | N-d sizes production and prices before running, with the sizing arithmetic in its design; AC-S34-8's "no one famished > 24 h" makes the failure visible |
| Sleepers woken all night by rule chatter | politeness band (§6.2 item 5); `woken_by_talk` can be turned off per world |
| Baselines move in Market Town on every S34 PR that changes behaviour | expected; re-recorded with reasons from CI's long record; other worlds must not move (AC-S34-2) |
| The 3D animation library has no lying clip | fallback pose + Zz (§8), audited at N-e's C0; a proper clip is an asset task recorded, not bought |
| Posture and sleep give two truths about the body | Q-S34-10: presentation lets `Asleep` override; the posture step states it |
| Scope creep into thirst, hygiene, illness, consequences | out of scope (Q-S34-2, Q-S34-3); recorded for MVP-1 |

---

## 13. Questions (Q-S34-n), each with a recommendation. **[OPERATOR]** marks operator-material ones

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34-1** | One pack (`needs` with sleep inside) or two (`sleep`, `needs`)? | **Two** (§3): different state, audiences and lives; each removable alone |
| **Q-S34-2** [OPERATOR] | Which needs in MVP-0? | **Hunger and tiredness only.** Thirst, hygiene and comfort are MVP-1; `motivated`'s social "lonely" stays a controller pseudo-need |
| **Q-S34-3** [OPERATOR] | Do needs have forced consequences (collapse asleep when exhausted, slower walking when famished, illness)? | **No, motivational only in MVP-0.** Consequences are rules of other packs and belong in the Interaction List later; a forced collapse would make `sleep` read `needs` and couple the two |
| **Q-S34-4** | Restorers as a closed typed set in `needs`, or an `ARC-62` catalog now? | **Closed set** (eating, sleeping); a catalog when a third restorer appears (`CLAUDE.md` §4 rule 11) |
| **Q-S34-5** | Does every person in a needs world get needs automatically, or only those with a `needs:` section? | **Only with a section** (`needs: {}` takes the defaults), like `holdings:` — explicit content, and a person without needs (a statue, a ghost) stays possible |
| **Q-S34-6** | How does sleep end? | **Alarm, `wake`, being addressed (configurable), acting (§4.3).** Movement and conversation stay ignorant of sleep |
| **Q-S34-7** [OPERATOR] | Which reference worlds enable the packs in MVP-0? | **Market Town** (the full default demo, which has food, shops and jobs). **Lakeside** after S33's content PRs merge, as a follow-up content PR (it has `consumption`, a bakery and cottages tagged `residential`). Social Café and Bodies Yard stay without them, as the removal witnesses |
| **Q-S34-8** [OPERATOR] | Market Town's supply and money cannot support three meals a day for 300 days (F-S34-3). Size it how? | **In N-d's content:** raise `produces` per shift and set prices so that the endowment of a person without a job lasts at least 300 days at three meals a day; record the arithmetic; L-13 (no income without a job) stays a recorded gap |
| **Q-S34-9** | Time skip when a human sleeps? | **No** in a shared world; the view dims; recorded for single-player MVP-1 |
| **Q-S34-10** | Sleep and the `posture` step (`ARC-147`): does `sleep` require or set a lying posture? | **Neither in MVP-0.** `sleep` stays independent; presentation shows `Asleep` over any posture; the posture step records that a sleeper's posture is presented as lying and that a bed anchor (with `bodies`) is a later refinement |
| **Q-S34-11** | Individual variation from S30's `chronotype` and `sleep-need`? | **Later**, in a `needs`/`rule` follow-up once S30-c merges (S30 §9.4: the owning System's own PR); N-b reads no attribute |
| **Q-S34-12** [OPERATOR] | Accept the defaults of §5.5 and the rhythm bounds of AC-S34-8 as fixed before any run? | **Accept.** They come from the committed ATUS table and the cited consensus, and a FAIL stops for you rather than being tuned |
| **Q-S34-13** | Are `fell-asleep`/`woke` biographical? | **No** by default (QS-49 precedent); configurable through `consequences` |
| **Q-S34-14** | Disclosure of others' sleep: everyone who perceives the person, or only people in the same place? | **Everyone who perceives the person** (presence's perception rule, which is place-bounded today), so a future window or open door needs no change here |

---

## 14. Records (placeholders; numbers from the primary session)

- **ARC-N-a** — `sleep`: an activity owned by a System Pack; how it ends without other packs knowing it.
- **ARC-N-b** — `needs`: lazy deficits, holder-only, words owned by the pack; genesis-only facts.
- **ARC-N-c** — `nourishment:`, a section a pack owns on item files.
- **ARC-N-d** — Controllers and needs: inert when absent; precedence asleep → due commute → eat → social.
- **DEP-N-a** — Reuse in both directions: The Sims' motives and Dave Mark's utility curves are adopted as *method*
  (already DEP-47); no needs library adopted (none exists that runs inside a replayed, integer, server-authoritative
  simulation); ATUS A-3 reused (already DEP-48).

---

## 15. Proposed edits to shared documents (for the primary session)

- `overall.md`: add S34 to the S26–S33 table as release-blocking; remove "`sleep` and needs do not exist" from the
  framework gap list once N-d merges.
- `mvp1/step-24-decision-and-memory.md`: a dated note on §6.5 and the DL-c row — moved to MVP-0 by the operator on
  2026-10-18, designed in step-34.
- `docs/MVP_STATUS.md` and the progress page: the S34 row.

---

## 16. Sources

- U.S. Bureau of Labor Statistics, American Time Use Survey, Table A-3, 2025 annual averages — as committed in
  `cognition/motivated/data/atus-a3-2025.tsv` with its `NOTICE` (public domain; credit given). Values used here:
  *Sleeping* row sum 8 826 ‰·h (8.8 h/day); 949 ‰ asleep at 03:00, 40 ‰ at 14:00; ≥ 471 ‰ from 22:00 through
  06:00; *Eating and drinking* peaks 76 ‰ (08:00), 144 ‰ (12:00), 153 ‰ (18:00), row sum 1 330 ‰·h.
- Watson N. F. et al., "Recommended Amount of Sleep for a Healthy Adult: A Joint Consensus Statement of the
  American Academy of Sleep Medicine and Sleep Research Society", *Sleep* 38(6), 2015 — secondary in this session;
  re-read at N-b's freeze before it enters a record.
- FG-a (`ARC-86`), the lazy-decay precedent; step-24 §§6.3–6.6.

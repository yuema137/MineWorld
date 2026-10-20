# PR N-e — Needs and sleep in the clients and the agent views

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes)

```text
Design revision:        revision 1 (2026-10-18)
Parents:                step-35-needs-and-sleep.md (STEP DESIGN FROZEN 2026-10-18) §§6.4, 6.5, 8, 9 (AC-S35-10,
                        AC-S35-11), 10.1 row N-e; pr-le-leave-early.md revision 2 §§0.3, 0.4, 0.6, 0.7 (LE-c owns the
                        LM `plans` lines, the `upcoming` and `complaint` triggers and the agent `PlansLine`)
Sibling designs:        pr-s35-nc-controllers.md (PR #277: the `commute-town` and Market Town test copies with both
                        packs); DOC-S35 (PR #278: ARC-283 … ARC-286, MODULE_SPEC §§4.1–4.2)
Depends on:             N-a (`Asleep`, `sleep`, `wake`), N-b (`Needs`), LE-a (`agenda.next`), LE-b (`late-for-shift`),
                        N-c (the test-copy helper); N-e3 also on LE-c (the prompt sections and trigger seam)
Implementation base:    origin/main at each sub-PR's C0; C0 records the commit and the merged prerequisites
Start condition:        this design FROZEN (the coordinator relays the operator's freeze) and §9's per-PR conditions
Execution contract:     §9 (one block per sub-PR)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0`, release scope. **Records:** cites `ARC-70` (client wording), `ARC-47` (no rule in the client),
`ARC-283` (sleep), `ARC-284` (needs), `ARC-286` (controllers and needs), `ARC-282` (departure). No new record id:
Q-Ne-1, if ruled yes, is a dated note on `ARC-286` (§11).

**What this PR does, in one paragraph.** After N-a … N-d, people in a world with the packs get hungry and tired and
go to bed, but nobody can *see* it. A human player has no hunger row, does not know they are asleep, and cannot tell
that the woman on the bed is sleeping. An MCP agent's `me` view and an LM seat's prompt say nothing about it either.
This PR shows the disclosed facts — the need words, who is asleep, the next part of the day, and an employer's
complaint about lateness — in both clients, the agent view and the LM prompt. Every client renders the words the
packs disclose; none computes a threshold or decides whether one may sleep. It also adds the request paths a
player needs (sleep for some minutes, wake up) where they are missing, and a hosted punctuality smoke.

---

## 1. Binding inputs

- Step-35 §6.4: `clients/agent` gains `NeedsLine` and `AsleepLine`; the MCP `me` view and the LM prompt render the
  owner's words within the prompt's bounds; agents and LM seats act through the same affordances
  (`sleep`, `wake`, `eat`, `buy`); tests use recorded cassettes.
- Step-35 §6.5: a human sees their own needs and gets `Sleep`, `Wake up` and `Eat …` from the offers; the view dims
  while asleep; no time skip.
- Step-35 §8 (the presentation table) and its rule: clients render disclosed words and states; neither computes a
  threshold, a level or whether one may sleep (`check_client_rules.py`); `Asleep` overrides any posture in
  presentation (Q-S35-10).
- AC-S35-10 and AC-S35-11, each also on a second world (ruling R-LE2-2 as applied to step 35, DOC-S35's dated note
  §0.1 item 2). Here the second world is LE-a's `commute-town` with both packs (N-c's copy).
- LE revision 2 §0.4: LE-c owns `Situation.plans`, the `upcoming` and `complaint` triggers, the `go` choice and
  `PlansLine`; `late-for-shift` reaches an agent's inbox as any perceived fact. **N-e does not touch them.** N-e owns
  only the human HUD row `ui.row.agenda-next` and the human late-for-shift notice (LE §0.4 "Humans").
- LE revision 1 §3.2 and Q-LE-9: "hosted (wall-clock) punctuality … a smoke in N-e".
- SPEED FIRST (charter): per commit only build and the test the commit writes; per PR `fast`; the parity check,
  the screenshots, the hosted smoke and the Godot runs at the milestone.
- `clients/shared/SETTINGS.md` §6 (catalog layers, data-built families), `clients/2d/PRESENTATION.md` §7.

---

## 2. Audit (`origin/main @ a8bba658`, 2026-10-18; each sub-PR's C0 re-checks on its base)

### 2.1 What exists

| Id | Thing | Where | Finding |
| --- | --- | --- | --- |
| A-1 | 2D readers | `clients/2d/scripts/hud/readers.gd:78-150` | `panels(observation)` builds one panel per own component (`{component, title, rows, payload, known}`); `_rows(kind, payload)` has a `match` per known kind (`wallet`, `holdings`, `shop`, `conversation-history`, `acquaintances`, `invitations`, `agenda`, `employment`); unknown kinds land raw under `panel.other`. `agenda` reads `label`, `place`, `since`, `until` only |
| A-2 | 2D panels | `hud/panels.gd` | labels only, no buttons; the `I` group holds every non-talk panel; rebuilt when the signature changes |
| A-3 | 2D wording | `hud/words.gd` | `Words.text(key, args)`, `Words.clock(seconds)` (`format.clock`), `Words.money`; the only 2D script that holds sentences (R6) |
| A-4 | 2D figures | `scripts/scene/people.gd:49-80, 175-232` | one figure per perceived person with a `label` and a `marker` Label (the participation marker from `participation`); `activity_of(id)` for the harness |
| A-5 | 2D dimming | `scripts/app.gd:299-313` | `world.modulate` grey while the connection is down |
| A-6 | 2D menu and intents | `scripts/menu.gd:141-182`, `scripts/intents.gd:18, 44-86` | the self menu lists target-less offers; a complete one is sent as offered; an incomplete one is sent only if `intents.gd` composes it (`COMPOSED`, `INPUTS`), else shown disabled "not supported". `sleep { for }` is parametric (step §6.1, N-c §2), so today it would be disabled |
| A-7 | 2D toasts | `hud/toasts.gd` | `toast(record, said, button, on_press)`; five-second messages |
| A-8 | Observation reader | `clients/protocol/mineworld/observation.gd:39-47, 127-151, 174` | `events_of(type)` (envelopes with `payload`), `component`, `component_value`, `own_component`; the module is symlinked into both clients (`clients/2d/mineworld`, `clients/3d-spike/mineworld`) |
| A-9 | Catalog layers | `clients/shared/settings/text.gd:10-11, 52-79, 169-181`; `SETTINGS.md` §6.2 | layer 1 shared `clients/shared/settings/locale/` holds "keys both clients use"; layer 2 the pack's `presentation/<pack>/<2D or 3D>/i18n/`. The 3D slice loads **only** the shared layer (`slice_main.gd:131-134`, `load_layers("")`). Keys built from data are confined to `FAMILIES = ["action", "reason", "state"]`, built only by `MineWorldText.code` |
| A-10 | Catalog test | `tests/acceptance/tests/client_text.rs` (AC-SET-4) | every `en.po` key in its layer's `zh_Hans.po` with the same placeholders; every key a script names exists in a layer that client loads; no unused key; **every action type a System Pack declares has `action.<type>` in the shared layer**, every refusal code and rejection reason a `reason.<code>`; `KEY_FAMILIES` (first key segment) is a closed list of 16; `messages.pot` is regenerated with `MINEWORLD_WRITE_POT=1 cargo test -p mineworld-acceptance --test client_text` |
| A-11 | Client rules | `scripts/check_client_rules.py` | R1–R6 over `clients/2d/**/*.gd` only; R2 forbids `COMPOSED` literals outside `intents.gd`; R6 forbids sentences at text sinks outside `words.gd` and the harness |
| A-12 | 3D HUD | `clients/3d-spike/scripts/controls_hud.gd:97, 148, 194`; `slice/slice_main.gd:507-590` | `hud.add_line(text) -> Label`, `hud.toast(text)`, `hud.caption(text)`; lines re-rendered every frame from keys in `_process` |
| A-13 | 3D link | `slice/slice_link.gd:309-376, 413-435` | `_on_observed` draws perceived people in the bound places as `NPC.make(...)` figures with a `Label` (Label3D, world text); the slice binds only the places it draws (the café, the street) |
| A-14 | 3D clips | `ASSETS.md:191-195`; `tools/animation_trim.py:29` | the Quaternius UAL Standard pack (CC0) is trimmed to `A_TPose`, `Idle_Loop`, `Walk_Loop`, `Jog_Fwd_Loop`; `human.gd` builds `Sit` procedurally because UAL's sitting clips are paid-tier (step-34 A-18). No lying or sleeping clip is in the repository. A web search on 2026-10-18 (quaternius.com pack page; opengameart.org) names "sitting, crawling, death" categories and **no sleep or lying clip**; the Standard clip list is only in the pack's zip and preview video. Settled at N-e2's C0 (§7.2) |
| A-15 | 3D poses | `scripts/posture.gd` (a corrective `SkeletonModifier3D`), `npc.gd:37` `enum Pose { WALK, STAND, SIT, LEAN, PUPPET }` | no posture disclosure is read yet (S34 not implemented); step-34 does not yet state "`Asleep` overrides posture" (Q-S35-10 asked it to) |
| A-16 | 3D menus | PR #250 (16d, open, stacked on #245/#171) | adds a **B** self menu (target-less offers: complete ones sent as offered, others disabled) and a Tab menu, and moves 16 keys to the shared layer |
| A-17 | Agent lines | `clients/agent/src/mineworld_agent/disclosed.py` (one reader per component), `models.py:251-320` (`AgendaLine`, `MeLine`, `PersonLine`), `view.py:255-345` (`render`), `view.py:468-503` (`delta_of`: `me` is sent whole when it changed) | no needs or sleep; `AttemptArgs` sends only complete offers ("an incomplete one needs its own operation", `models.py:367-368`); so an agent **cannot request `sleep { for }`** today |
| A-18 | Agent goldens | `clients/agent/tests/golden/tools.json` (`test_cli.py`), `tests/fixtures/*.json` + `SOURCE.md` (recorded observations, hashes) | a new field in a `*Result` model changes `tools.json` |
| A-19 | LM situation and prompt | `cognition/lm-controller/src/mineworld_cognition/controller/situation.py`, `prompt.py:142-169` | `Situation { me, place, at, present, talk_to, offers }`; `_user` writes WHERE I AM / WHAT I REMEMBER / WHAT JUST HAPPENED; `offers` are only available **complete** affordances, so a model cannot ask for `sleep { for }`; `SocialChoice.act` is say / attempt / nothing (LE-c adds `go`) |
| A-20 | LM byte bounds | `memory/retrieve.py:50-51` (`LINE_BYTES 160`, `SECTION_BYTES 6000`), `persona.py:19-22`, `prompt.py:22` | **there is no prompt-wide byte budget, and `budget.py` holds wall-time call and token budgets** (`ARC-57`), not prompt bytes. The step's and LE's "byte budget (`budget.py`)" points at the wrong file; N-e bounds its own lines (§4.6) |
| A-21 | LM triggers | `controller/triggers.py` | fact-based (`spoke`, `arrived`); LE-c adds a time-based `upcoming` trigger and a `complaint` trigger |
| A-22 | LM seat holding | `docs/MODULE_SPEC.md` §5 LM Controller | while the LM process holds a seat, it is the only driver; nothing else walks or puts that person to bed |
| A-23 | Hosted runs | `tools/cli/tests/hosted_town.rs`, `fx24.rs`, `support/mod.rs` (`Server::start_captured`, `status`, `interrupt`, `SaveDir`), `headless::Tables::read`, `social::facts_of` | the pieces of a hosted run that is read back from its save |

### 2.2 Findings

- **F-Ne-1 — strings both clients use belong in the shared layer.** Step §8 names `presentation/mineworld-default/2D/i18n`
  for both clients, but the 3D slice does not load that layer (A-9), and `SETTINGS.md` §6.2 puts "keys both clients
  use" in `clients/shared/settings/locale/`. PR #250 already moves keys there for the same reason. N-e puts every new
  key in the shared layer (`en.po`, `zh_Hans.po`, regenerated `messages.pot`). The step's intent — one set of
  message ids for both clients — is met exactly. Bounded; recorded, not a question.
- **F-Ne-2 — need words and lateness levels must be translatable without a rule.** The packs disclose closed words
  (`hungry`, `sleepy`, `warning` …). A zh_Hans player must read 饿了, not `hungry`. A key built from a disclosed
  word is a data-built key, which `SETTINGS.md` confines to `MineWorldText.FAMILIES`. N-e adds one family, `word`
  (`word.<a word a System Pack discloses>`), to `FAMILIES`, `KEY_FAMILIES` and `SETTINGS.md` §6, and the catalog
  test covers every word the packs declare (§4.2). An unknown word reads as itself (the readable fallback).
- **F-Ne-3 — no client names a need.** The `needs` record's own `words` map names the needs. A reader that iterates
  it shows `hunger` and `tiredness` today and a third need (thirst, MVP-1) with no client change, only two catalog
  entries. So no client script holds `"hunger"` or `"tiredness"`, which is also what makes R7 (§4.5) simple.
- **F-Ne-4 — a bar from a level without arithmetic.** Step §8 asks for a 5-step bar *and* no level arithmetic in a
  client. A Godot `ProgressBar` with `min_value 0`, `max_value 1000` (the deficit scale `MODULE_SPEC.md` §4.1
  states) and `step 200` is set to the disclosed level and snaps it to 5 steps by itself. The script assigns a
  number; it compares and computes nothing.
- **F-Ne-5 — a parametric `sleep` has no request path for three of the five requesters.** The 2D menu (A-6), the MCP
  adapter (A-17) and the LM seat (A-19) can send only complete offers or the types they compose. Step §6.4/§6.5 and
  AC-S35-11 ("an agent's `sleep` request is validated exactly as a rule seat's") need one. N-e adds `sleep` to the
  2D composer, a `sleep` operation to the agent library, and — if Q-Ne-1 is ruled yes — a `sleep` act to the LM seat.
- **F-Ne-6 — an LM seat never considers its needs.** Its triggers are being spoken to, a known person arriving, and
  (LE-c) an upcoming part (A-21, A-22). A hungry LM seat that nobody talks to never eats; at night it never sleeps.
  Q-Ne-1.
- **F-Ne-7 — the 3D client has no self menu until 16d.** With PR #250 merged, `wake` (complete) works from the B menu
  unchanged, and `sleep` needs one composer line. Without it, the 3D client shows the rows and the hint "move to
  wake up" (moving is a real act that wakes a sleeper, `ARC-283` reason `acted`). Bounded at N-e2's C0 (§7.2);
  recorded.
- **F-Ne-8 — the default 3D slice rarely shows a sleeper.** The slice draws the café and the street; `sleep`'s
  reference list permits only `residential` places. A sleeper is drawn in 3D only in a world that permits sleep in
  a drawn place, or when the player's own seat sleeps. The 3D Zz therefore gets its evidence from a probe world
  that permits sleep at the café (§7.2 C3), not from Market Town.
- **F-Ne-9 — prompt bounds.** A-20. N-e's prompt lines are each at most `LINE_BYTES` (160) bytes, cut at a character
  boundary, and only present when the packs disclose something; a world without the packs renders byte-identical
  prompts (the golden renders do not move).

---

## 3. Scope

### 3.1 In scope

1. **Shared (N-e1 C1):** the reader `clients/protocol/mineworld/needs.gd` (`MineWorldNeeds`, data only, no wording);
   the catalog entries of §4.2 in the shared layer; the `word` family; `MineWorldClockFormat.whole_minutes`; rule R7.
2. **Fixtures (N-e1 C2):** recorded observations on Market Town and `commute-town`, both with the packs (§5).
3. **2D (N-e1 C3–C4):** panels "How you feel" (need rows with bars) and "Asleep"; the next-part row in the agenda
   panel; the late-for-shift and missed-shift notice; the Zz glyph; the dimmed view while I sleep; a "Wake up…"
   button that opens my own menu; `sleep` composed with a typed number of minutes.
4. **Hosted punctuality smoke (N-e1 C5):** an ignored test on both worlds (§6.3).
5. **3D (N-e2):** the same rows in the slice HUD; the notice as a toast; the dimmed view; the Zz label on a sleeping
   figure, or the sleeping pose when a free clip exists (C0 audit); `sleep` composed and `wake` sent from 16d's
   menu when #250 has merged; the 2D/3D parity check (AC-S35-10).
6. **Agent and LM (N-e3):** `NeedLine`, `NeedsLine`, `AsleepLine` on `MeLine` and `PersonLine`; the agent `sleep`
   operation; the LM `Situation` and prompt lines for needs and sleep; if Q-Ne-1 is yes, the LM `feeling` trigger and
   `sleep` act.

### 3.2 Not in scope

- LE-c's parts: `Situation.plans`, the `upcoming` and `complaint` triggers, the `go` choice, `PlansLine`, the agent
  inbox entry for `late-for-shift` (it is already a perceived fact). N-e reads none of them and edits none.
- A walking-time estimate in the human HUD. The world discloses `ways` lengths; turning them into minutes uses a
  host's stride cadence (LE §0.3). The HUD shows the next part and its time only, as step §8 says. Recorded F-Ne-10
  for a later PR if the operator wants it.
- Any `systems/`, `kernel/`, `contracts/`, `server/`, `persistence/` change; any reference world; any decision-model
  change (N-c).
- The 3D lying clip or a procedural lying pose (asset work, recorded); posture's own presentation (S34).
- The milestone runs themselves (parity, screenshots, hosted smoke): written here, run at the milestone.

### 3.3 Invariants

- **I-Ne-1 — words, not rules.** No client script (2D, 3D, protocol module) names a need, a need word, or compares a
  level. The checker R7 holds it (§4.5).
- **I-Ne-2 — inert when absent.** A world without the packs and without LE shows exactly what it shows today: no new
  panel, row, glyph, toast, prompt line or agent field (fields are `None` or empty and omitted from JSON when the
  model already omits defaults; C0 confirms the agent's serialisation rule).
- **I-Ne-3 — one wording.** Each new key exists once, in the shared layer, in `en` and `zh_Hans`, with the same
  placeholders; both clients use the same key for the same row.
- **I-Ne-4 — the server decides.** Sleep, wake and eat requests are sent whatever the client believes; minutes typed
  by a player are sent as typed (an integer parse only) and the server refuses an out-of-bound value.
- **I-Ne-5 — no duplicate of LE-c.** N-e adds no plans line, no upcoming or complaint trigger, no `go`.
- **I-Ne-6 — `Asleep` wins in presentation.** Where a client chooses how to draw a person, a disclosed `Asleep`
  overrides any other pose (Q-S35-10).

---

## 4. Design

### 4.1 The shared reader `MineWorldNeeds` (`clients/protocol/mineworld/needs.gd`)

Data only, like `observation.gd`: no wording, no `MineWorldText` (the protocol project does not load the settings
module). Static functions; every one returns `{}` or `[]` when nothing is disclosed.

```text
class_name MineWorldNeeds   extends RefCounted

const NEEDS := "needs"        # the component type N-b discloses (C0 confirms Needs::COMPONENT_TYPE)
const ASLEEP := "asleep"      # the component type N-a discloses (C0 confirms Asleep::COMPONENT_TYPE)
const AGENDA := "agenda"
const LATE := "late-for-shift"
const LEVEL_MAX := 1000       # the deficit scale of MODULE_SPEC §4.1, for the bar's range only

static func own_needs(obs) -> Array
    # [{ "need": String, "word": String, "level": int }, ...] in the order of the disclosed `words` map;
    # `level` is payload[need] when it is a number, else -1 (no bar is drawn for -1)
static func asleep_of(obs, id: String) -> Dictionary
    # { "since": int, "alarm": int or null } when `asleep` is disclosed about `id`, else {}
static func next_part(obs) -> Dictionary
    # the observer's agenda `next`: { "label": String, "place": String (entity id), "from": int,
    #   "punctual": bool }, else {}
static func lateness(obs) -> Array
    # every `late-for-shift` event in obs.events() whose payload's employee is the observer, oldest first:
    # [{ "employer": String (entity id), "late_by": int, "count": int, "level": String, "absent": bool }]
```

The file holds no `<`, `>`, `<=` or `>=` and no literal need name (R7). A missing or odd-shaped field yields `{}`
for that record, never an error (the `observation.gd` rule).

### 4.2 Catalog entries (shared layer; `en` / `zh_Hans`)

The `word` family (`MineWorldText.FAMILIES` gains `"word"`; `client_text.rs` `KEY_FAMILIES` gains `"word"`; both
clients build the key only through `MineWorldText.code("word", w)`; 2D through a new `Words.word(w)` that calls it).

| Key | en | zh_Hans |
| --- | --- | --- |
| `word.hunger` | Hunger | 饥饿 |
| `word.tiredness` | Tiredness | 疲倦 |
| `word.ok` | fine | 还好 |
| `word.peckish` | peckish | 有点饿 |
| `word.hungry` | hungry | 饿了 |
| `word.famished` | famished | 饿极了 |
| `word.sleepy` | sleepy | 困了 |
| `word.tired` | tired | 累了 |
| `word.exhausted` | exhausted | 筋疲力尽 |
| `word.notice` | a notice | 提醒 |
| `word.warning` | a warning | 警告 |
| `word.final` | a final warning | 最后警告 |

Rows and notices:

| Key | en | zh_Hans | Placeholders |
| --- | --- | --- | --- |
| `panel.needs` | How you feel | 你的状态 | — |
| `panel.asleep` | Asleep | 睡眠中 | — |
| `ui.row.need` | {need}: {word} | {need}：{word} | need, word |
| `ui.row.asleep` | Asleep since {since}, alarm at {alarm} | 自 {since} 起在睡，闹钟 {alarm} | since, alarm |
| `ui.row.asleep.no-alarm` | Asleep since {since} | 自 {since} 起在睡 | since |
| `ui.button.wake` | Wake up… | 起床… | — |
| `hud.asleep.move-to-wake` | Move to wake up | 移动即可醒来 | — |
| `ui.row.agenda-next` | Next: {label} at {place}, {from} | 接下来：{from}，{label}，在{place} | label, place, from |
| `ui.row.agenda-next.punctual` | Next: {label} at {place}, {from} — be on time | 接下来：{from}，{label}，在{place}——别迟到 | label, place, from |
| `ui.notice.late-for-shift` | {employer}: you were {minutes} min late for your shift. That is {level} ({count} lately). | {employer}：你上班迟到了 {minutes} 分钟。这是{level}（近期第 {count} 次）。 | employer, minutes, level, count |
| `ui.notice.missed-shift` | {employer}: you missed your shift. That is {level} ({count} lately). | {employer}：你缺勤了一个班次。这是{level}（近期第 {count} 次）。 | employer, level, count |
| `ui.glyph.asleep` | Zz | Zz | — |
| `ui.prompt.sleep` | Sleep for how many minutes? | 睡多少分钟？ | — |
| `suggest.sleep.nap` | 30 | 30 | — |
| `suggest.sleep.night` | 480 | 480 | — |
| `action.sleep` | Sleep | 睡觉 | (only if N-a did not add it; C0) |
| `action.sleep.done` | You fell asleep | 你睡着了 | (same) |
| `action.wake` | Wake up | 起床 | (same) |
| `action.wake.done` | You woke up | 你醒了 | (same) |

`reason.<code>` for any refusal code N-a adds (the `for` bound) — only if N-a did not add it; the catalog test
names a missing one. 27–31 keys, 54–62 strings. `{minutes}` is `MineWorldClockFormat.whole_minutes(late_by)`, a new
static in `clients/shared/settings/clock_format.gd`: `(seconds + 59) / 60`, a unit format like `Words.clock`'s
division, not a rule. `{since}`, `{alarm}`, `{from}` are `Words.clock(...)` in 2D and
`MineWorldText.text("format.clock", …)` through the same helper in 3D (C0 checks that `format.clock` is in the
shared layer; if it is only in the 2D pack, C1 moves it verbatim, as #250 moved its 16).

**Word coverage in `client_text.rs`.** A new list `PACK_WORDS` holds every word the packs disclose, read from the
packs' source the way `action_types()` reads action types: C1 reads the needs pack's word table and the employment
lateness-level enum (C0 names the exact items, e.g. `mineworld_needs::WORDS`, `LatenessLevel`'s serde names). If no
readable seam exists, C1 writes the list literally with a comment naming the source file, and a follow-up is
recorded. Every `word.<w>` for `w` in the list and for each need name must be in both shared catalogs.

### 4.3 2D presentation

**Panels** (`readers.gd`, `_rows`): two new `match` arms.

- `"needs"` → one row per `MineWorldNeeds.own_needs(observation)` entry:
  `Words.text("ui.row.need", {"need": Words.word(e.need), "word": Words.word(e.word)})`. The panel dictionary gains
  `"bars": [level, …]` (same order as rows). `panels.gd` `_box` draws, under each row with a bar entry ≥ 0, a
  `ProgressBar` (`min_value 0`, `max_value MineWorldNeeds.LEVEL_MAX`, `step 200`, `show_percentage false`,
  `custom_minimum_size (292, 8)`, `value = level`). Title `panel.needs`.
- `"asleep"` (own) → one row: `ui.row.asleep` with `since`, `alarm` (or `ui.row.asleep.no-alarm` when `alarm` is
  null). The panel dictionary gains `"button": "wake"`; `_box` adds a `Button` with `Words.text("ui.button.wake")`
  that emits a new `panels.gd` signal `wake_pressed`. Title `panel.asleep`.
- `"agenda"` → after the existing row, when `MineWorldNeeds.next_part(observation)` is not empty, one more row:
  `ui.row.agenda-next.punctual` if `punctual` else `ui.row.agenda-next`, with `label` (world text), `place`
  (`place(id)`), `from` (`Words.clock(from)`). `_rows` receives the observation for this (it already has `_latest`).

The panel signature in `show_panels` includes `bars` and `button`, so a change rebuilds the box.

**The Wake up button.** `app.gd` connects `panels.wake_pressed` to `open_menu(latest.observer())`. The player then
chooses the offered `wake` in their own menu (`action.wake`), the path every offer takes. No script but
`intents.gd` names an action (R2).

**Sleep composed.** `intents.gd`: `COMPOSED` gains `"sleep"`; `INPUTS["sleep"] = {"prompt": "ui.prompt.sleep",
"suggest": ["suggest.sleep.nap", "suggest.sleep.night"]}` (the talk-line shape; C0 checks `INPUTS`'s exact form);
`compose` gains `"sleep": payload = {"for": int(String(input))}` — `String.to_int` semantics; a non-number becomes 0
and the server refuses it with the bound. The payload key `for` is confirmed against N-a's offer at C0.

**Notice** (`app.gd` `_on_observed`): for each entry of `MineWorldNeeds.lateness(observation)`,
`toasts.toast({"kind": "notice", "action_type": "", "target": null}, text)` with `ui.notice.missed-shift` when
`absent`, else `ui.notice.late-for-shift`; `employer` = `readers.thing(employer)`, `level` =
`Words.word(level)`, `count`, `minutes`. Each event is toasted once: `app.gd` keeps the ids it has shown (event `id`),
cleared on `_on_welcomed`. `toasts.gd`'s colour for a non-`accepted` kind already applies.

**Zz glyph** (`people.gd`): `_make` adds a third Label `zz` (font 14, same outline as `marker`, position above the
marker, `auto_translate_mode` disabled); `reconcile` sets `person["zz"].text =
Words.text("ui.glyph.asleep") if not MineWorldNeeds.asleep_of(observation, id).is_empty() else ""`, and stores
`person["asleep"] = bool` for a new `asleep_of(id) -> bool` used by the harness.

**Dimmed view** (`app.gd`): one function `_shade()` sets `world.modulate` from two facts: connection down → the
existing grey; else own `asleep` disclosed → `Color(0.62, 0.66, 0.82)` (a night-blue dim); else white. Called from
`_on_state_changed` and `_on_observed`.

### 4.4 3D presentation

**Rows** (`slice/needs_hud.gd`, new, `class_name SliceNeedsHud`, `extends Node`): attached by `slice_main.gd` `_hud()`
when `hud != null`; holds up to N need lines, one asleep line, one next line, each a `hud.add_line("")` Label plus,
for each need, a `ProgressBar` added beside it with the 2D numbers. `refresh(obs, link)` is called from
`slice_link`'s observed path (a new signal `observed(obs)` on `SliceLink`, emitted at the end of `_on_observed`) and
re-renders on `NOTIFICATION_TRANSLATION_CHANGED` from its last observation. Texts use the same keys and arguments as
2D (`MineWorldText.text`, `MineWorldText.code("word", …)`, names through `link.shown_text(id)` for people and the
place's display label for places). While asleep and #250 is not merged, the asleep line is followed by
`hud.asleep.move-to-wake`.

**Static row builder** for parity: `static func rows(obs, name_of: Callable, place_of: Callable) ->
PackedStringArray` in `needs_hud.gd`; the 2D side's equivalent is `readers.gd`'s rows. Both are called by the parity
check (§6.2).

**Notice:** `needs_hud.gd` toasts each new `lateness` entry once through `hud.toast(text)`.

**Dim:** while own `asleep` is disclosed, the slice's `WorldEnvironment` adjustment brightness is set to 0.6 (C2
names the node `_environment()` creates); restored on waking.

**Sleeping figure** (`slice_link.gd` `_on_observed`): for each drawn figure, a child Label3D `Zz`
(`ui.glyph.asleep`, billboard, `position (0, 2.35, 0)`, `font_size 56`, `pixel_size 0.004`, joined to the
world-text group off — it is UI text and is translated) is visible iff `MineWorldNeeds.asleep_of(obs, id)` is not
empty. If C0's audit finds a free CC0 lying or sleeping clip in the UAL Standard pack (§7.2 C0), C3 instead adds it
to `animation_trim.py`'s `DEFAULT_CLIPS` and plays it for a sleeping figure, keeping the Zz label; otherwise the
figure keeps its idle (step §8's fallback). Either way the sleeping state is decided first in the figure update, so
any later posture input cannot override it (I-Ne-6).

**Sleep and wake in 3D** (only when #250 is merged at C0): `slice/intents.gd` composes `sleep` with the presentation
default `{"for": 480}` (the 2D night suggestion; like the 3D default utterance); `wake` is complete and already sent
by 16d's B menu.

### 4.5 Rule R7 (`scripts/check_client_rules.py`)

```text
R7  over clients/2d/**/*.gd, clients/3d-spike/scripts/**/*.gd and clients/protocol/mineworld/needs.gd
    (a) no code line (comments stripped) holds a string literal or identifier equal to a need name or a need
        word of NEED_VOCABULARY = ("hunger", "tiredness", "peckish", "hungry", "famished", "sleepy", "tired",
        "exhausted");
    (b) clients/protocol/mineworld/needs.gd holds none of the tokens <, >, <=, >= outside string literals
```

`--self-test` cases (the file's pattern): a planted `if level >= 600` in a copy of `needs.gd` is found by (b); a
planted `"hungry"` in a 2D script is found by (a). R7 reports `file:line: R7 …` like R1–R6. The script's docstring
says R7 also covers the 3D scripts.

### 4.6 Agent library (`clients/agent`)

`models.py`:

```text
class NeedLine(AgentModel):    need: Identifier; word: Identifier; level: int   (0 … 1 000)
    """One of my needs as `needs` discloses it: its name, the pack's word for it now, and the level."""
class NeedsLine(AgentModel):   needs: list[NeedLine]
    """needs' `Needs`, disclosed to me only: how I feel, in the pack's own words."""
class AsleepLine(AgentModel):  since: int; alarm: int | None = None
    """sleep's `Asleep`: asleep since `since`; `alarm` only in my own line."""
MeLine      += needs: NeedsLine | None = None;  asleep: AsleepLine | None = None
PersonLine  += asleep: AsleepLine | None = None
SleepArgs(Seated) += minutes: Annotated[int, Field(ge=1, le=1440)]   (the server's bound decides; 1440 only keeps
                     the schema finite)
```

`disclosed.py`: `needs(me) -> NeedsLine | None` (iterates the payload's `words` map; a level that is not an int is
skipped), `asleep(entity) -> AsleepLine | None`; table in the module docstring gains two rows. `view.py` `render`
fills them. `Identifier` is the existing type (`OfferLine.action_type`); if a word fails its pattern, the line is
skipped (a newer pack's shape, the module's rule).

**The `sleep` operation.** If #261 (one `act` tool) is merged at N-e3's C0: `act` gains kind `sleep` with
`minutes`. Otherwise: a new `Player.sleep(args: SleepArgs)` and MCP tool `sleep`. Either way it finds the offered
`sleep` affordance in the newest observation and builds the request through `mineworld_sdk.offers` with
`{"for": minutes}` (the same builder `walk_to` uses for a parametric offer; C0 names it); no offered `sleep` → the
library's existing "not offered" error. `tools.json` is re-recorded (`main(["schema", "--out", …])`), and the diff
is reviewed: only the new fields and the operation.

### 4.7 LM seat (`cognition/lm-controller`)

`situation.py`:

```text
class Feeling(CognitionModel):   need: str; word: str
Situation += feelings: tuple[Feeling, ...] = ();  asleep_since: int | None = None;  alarm: int | None = None
Person    += asleep: bool = False
```

filled from the observer's `needs` record (`words` map, in order) and `asleep` records (the observer's and each
present person's). `prompt.py` `_user`, after `I can talk to:` and before `Offers:`, adds — only when present:

```text
How I feel: hunger: hungry; tiredness: sleepy          (need: word pairs, "; "-joined, cut to 160 bytes)
I am asleep since 23:10; my alarm is at 05:00.          (or "I am asleep since 23:10." with no alarm)
```

and `People here:` labels a sleeper as `Bob (asleep)`. Times are `HH:MM` of `at % 86400` (a `_clock` helper; LE-c's
if it lands first). Words are the pack's, untranslated (the prompt is English). Absent packs → the prompt is
byte-identical; `tests/golden/s28a_renders.json` does not move (a test asserts it).

**If Q-Ne-1 is yes (C4 of N-e3):**

- Trigger `feeling` (`Trigger.kind` gains `"feeling"`): through LE-c's time/observation trigger seam, once when the
  observer's set of `(need, word)` pairs differs from the set at the seat's previous consult, and once when the
  observer's own `asleep` record disappears (woke). Rendered under WHAT JUST HAPPENED as
  `"I notice how I feel: hunger: hungry."` (the changed pairs). No cooldown beyond "on change": words change a few
  times a day.
- Act `sleep` (`SocialChoice.act` gains `"sleep"` with `minutes: int`): `intent.py` builds `sleep { for: minutes }`
  from the offered `sleep` through `mineworld_sdk.offers`; the server validates (bound, place). The system prompt's
  rule line lists `"sleep"` with `"minutes"`. `wake` and `eat` stay ordinary `attempt`s of complete offers.
- The fallback (`rule` with N-c's bands) is unchanged.

---

## 5. Fixtures (N-e1 C2)

Recorded once, committed, read by the GDScript checks, the agent tests and the LM tests. Location
`tests/fixtures/observations/needs/` with `SOURCE.md` (what each is, command, base commit, sha256), following
`clients/agent/tests/fixtures/SOURCE.md`.

Recorder: `tools/cli/tests/needs_fixtures.rs`, one `#[ignore = "records the needs fixtures; run by hand with
MINEWORLD_WRITE_FIXTURES=1"]` test per fixture, using N-c's copy helper (`tools/cli/tests/fixture/`) and
`support::{Server, Client, SaveDir}`. It writes each observation's JSON exactly as received.

| File | World | How | Must contain |
| --- | --- | --- | --- |
| `market-town-night.json` | Market Town with the packs (`worlds/market-town` once N-d merges; N-c's copy before) | `server --town --time-scale 300 --cadence 1`; wait until `/status.at` ≥ 02:30 day 1; join a resident's seat (C2 picks one who is asleep in the run's log); first observation | own `needs` (words for both needs), own `asleep` with `alarm`, own `agenda` with `next` (`punctual: true` for a worker) |
| `commute-late.json` | `commute-town` copy with the packs | the test client holds an employee's seat from genesis and does nothing until the shift start + 10 min, then walks to the workplace (`walk-to` + strides, `support` helpers); the first observation after entering | own `needs` (hunger `hungry` or worse by then), a `late-for-shift` event for the observer, `agenda.next` |
| `commute-sleeper.json` | `commute-town` copy with the packs, its `configure/sleep.yaml` extended by the test with `- { action: sleep, place: <the workplace's class>, effect: permit }` (an author's data, C2 names the class) | two clients: seat P at the workplace requests `sleep { for: 60 }`; seat Q walks in; Q's first observation with P present | another person's `asleep` (no `alarm`), Q's own `needs` |

If a recording cannot reach a "must contain" item, C2 stops and reports to the lane lead; a derived (hand-edited)
fixture is not a substitute.

---

## 6. Acceptance (fixed before any run; each on Market Town **and** `commute-town`)

### 6.1 Per PR (SPEED FIRST)

| Id | Criterion | Where |
| --- | --- | --- |
| AC-Ne-1 | `MineWorldNeeds` over the three fixtures returns the hand-written expected records (need, word, level; since, alarm; next; lateness) and `{}`/`[]` over an existing pack-free fixture (`clients/agent/tests/fixtures/market-town-start.json`) | `clients/protocol/checks/needs_check.gd` |
| AC-Ne-2 | `client_text` passes: every new key in both shared catalogs with equal placeholders, every pack word covered, `messages.pot` current; `check_client_rules.py` (R1–R7) and its `--self-test` pass | N-e1 C1 |
| AC-Ne-3 | Agent: `render` of `market-town-night.json` gives `me.needs` with both needs' words and levels equal to the file's, `me.asleep.alarm` set; `commute-sleeper.json` gives the sleeper's `PersonLine.asleep` with `alarm` null; the three older fixtures give `needs`/`asleep` `None` and their existing test expectations unchanged; `tools.json` equals the re-recorded golden | `clients/agent/tests/test_needs_view.py` |
| AC-Ne-4 | Agent `sleep`: against a scripted observation offering `sleep`, `sleep(minutes=480)` submits exactly the request `mineworld_sdk.offers` builds for `{"for": 480}`; with no offer, the "not offered" error | same file |
| AC-Ne-5 | LM: `Situation.of` + `build_request` over `market-town-night.json` and `commute-late.json` contain `How I feel: ` with the file's words, and the asleep line for the first; over `commute-sleeper.json` `People here:` contains `(asleep)`; over every pre-existing render fixture the request is byte-identical to `tests/golden/s28a_renders.json` | `cognition/lm-controller/tests/test_needs_prompt.py` |
| AC-Ne-6 | (Q-Ne-1 yes) scripted backend: a `feeling` trigger fires once per word change and not otherwise over a scripted sequence of three observations; a scripted `{"act": "sleep", "minutes": 480}` becomes the `sleep { for: 480 }` request the SDK builds; a replay of the same cassette gives identical requests | `tests/test_needs_triggers.py` |

### 6.2 At the milestone (written in this PR, `#[ignore]` or a probe mode)

| Id | Criterion | Where |
| --- | --- | --- |
| AC-Ne-7 (AC-S35-10) | For each of the three fixtures and in `en` and `zh_Hans`, the 2D readers' rows and the 3D `SliceNeedsHud.rows` are equal line for line, and equal to a hand-written oracle (`tests/fixtures/observations/needs/rows.<locale>.txt`) | `tools/cli/tests/needs_rows_parity.rs` (Godot-gated, `ci_godot.py tests` list); a 2D `--drive=rows` harness mode and a 3D `--slice-needs-rows` probe mode, each printing `ROWS <fixture> <locale> <json>` |
| AC-Ne-8 (AC-S35-10) | Connected: in Market Town and the `commute-town` copy, a 2D and a 3D client on the same seat show the needs rows and the next row; the 2D Zz is on a sleeping figure; screenshots for the operator | the existing harnesses (`--drive=panels`, the slice probe), milestone hand check |
| AC-Ne-9 (AC-S35-11) | An MCP agent's `sleep` at a forbidden place gets `PermissionDenied` and the same refusal a rule seat gets for the same request (the agent path is the server path); `wake` while awake `NoSupportedInteraction` | `clients/agent/tests/test_needs_real.py` (`real_server` mark) on the `commute-town` copy |
| AC-Ne-10 (hosted punctuality smoke) | `server --town --cadence 1 --time-scale 300 --save DIR` on Market Town with the packs and on the `commute-town` copy, from genesis until `/status.at` passes the day's last first-shift start + 30 min; then SIGINT; from the save: every day-1 `shift-started` has `present: true` and no `late-for-shift` exists. Thresholds fixed here; a FAIL is a milestone finding, never a retune | `tools/cli/tests/hosted_punctuality.rs` (`#[ignore]`) |
| AC-Ne-11 (mutation, milestone) | M-Ne-1: R7(b) — `if level >= 600` planted in `needs.gd` turns R7 red; M-Ne-2: `Needs` rows read from another person's record (the observer filter dropped in `own_needs`) turns AC-Ne-1 red; M-Ne-3: `" (asleep)"` dropped in `prompt.py` turns AC-Ne-5 red | — |

---

## 7. The split and the commit plans

### 7.1 Three sub-PRs in three worktrees

```text
N-e1  mvp0/pr-s35-ne-2d     /Users/yuema137/mineworld-worktrees/impl-s35-ne-2d      shared + fixtures + 2D + smoke
N-e2  mvp0/pr-s35-ne-3d     /Users/yuema137/mineworld-worktrees/impl-s35-ne-3d      3D + parity
N-e3  mvp0/pr-s35-ne-agent  /Users/yuema137/mineworld-worktrees/impl-s35-ne-agent   agent + LM
```

N-e2 and N-e3 branch from N-e1's pushed C2 commit (they need the reader, the catalogs and the fixtures), work in
parallel with N-e1's C3–C5, and rebase onto `main` once N-e1 merges. Merge order: N-e1, then N-e2 and N-e3 in either
order (they touch disjoint files). Lane limit: at most two code-PR CI runs in flight. Implementer: Haiku for every
commit from this plan; Opus review by the lane lead (step §10.1 row N-e). Size: N-e1 ~450 GDScript/Rust/Python +
~60 catalog strings; N-e2 ~250; N-e3 ~350 Python + tests. Total ~1.1 kLOC.

Each commit tracks Implementation, Validation and Review separately. Validation is the quick local check named; the
heavy ones (Godot, `cargo test` of one binary) go through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.

### 7.2 N-e1 (`mvp0/pr-s35-ne-2d`)

**C0 — Base audit (ledger only).**
- [ ] Record the base and which of N-a, N-b, LE-a, LE-b, N-c, N-d, #250, #261, S30-f are merged.
- [ ] Confirm the merged names: `Needs`/`Asleep` component types and payload shapes (`words` map; `alarm` holder-only);
  `agenda.next` field names; `late-for-shift` payload fields; `sleep` offer payload key `for`; whether N-a added
  `action.sleep`, `action.wake`, their `.done` and its refusal `reason.<code>` to the shared catalogs; the seams for
  `PACK_WORDS` (§4.2); whether `format.clock` is in the shared layer; `INPUTS`'s exact form in `intents.gd`.
- [ ] Any missing disclosure or a shape that differs from §4.1 → stop to the lane lead with the evidence.
- Implementation [ ] · Validation N/A (audit) · Review [ ]

**C1 — The shared reader, the wording and R7.**
- Goal: one data reader, one wording, one rule for both clients.
- Scope: `clients/protocol/mineworld/needs.gd` (+ `.uid` generated by Godot), `clients/protocol/checks/needs_check.gd`
  (synthetic frames for the shapes, real fixtures added in C2); `clients/shared/settings/text.gd` (`FAMILIES` +
  `"word"`, its doc comment); `clients/shared/settings/clock_format.gd` (`whole_minutes`);
  `clients/shared/settings/locale/{en.po, zh_Hans.po}` (§4.2) and `messages.pot` (regenerated);
  `clients/shared/SETTINGS.md` §6.2 note (the `word` family, the keys both clients use); `tests/acceptance/tests/client_text.rs`
  (`KEY_FAMILIES` 16 → 17, `PACK_WORDS` coverage); `scripts/check_client_rules.py` (R7 + self-test cases).
- [ ] Implement `needs.gd` per §4.1. [ ] Add keys. [ ] `FAMILIES`, `KEY_FAMILIES`. [ ] `PACK_WORDS` coverage.
  [ ] R7 and its self-test. [ ] Regenerate `messages.pot`.
- Validation: `with-slot.sh godot --headless --path clients/protocol --script res://checks/needs_check.gd` prints only
  PASS lines; `with-slot.sh env MINEWORLD_WRITE_POT=1 cargo test -p mineworld-acceptance --test client_text` then
  the same without the variable PASS; `python3 scripts/check_client_rules.py` and `--self-test` exit 0.
- Edge cases: a `words` value that is not a string → that entry skipped; a level that is not a number → `-1`; an
  `asleep` payload without `since` → `{}`.
- Commit: `feat(clients): a shared reader and wording for needs, sleep and lateness (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ] (R7 cannot be passed by a renamed variable that still compares a
  level; I-Ne-3 holds)

**C2 — Fixtures.**
- Scope: `tools/cli/tests/needs_fixtures.rs` (§5), `tests/fixtures/observations/needs/{market-town-night,
  commute-late, commute-sleeper}.json`, `SOURCE.md`; `needs_check.gd` gains the real-fixture claims of AC-Ne-1.
- [ ] Recorder. [ ] Record (`with-slot.sh env MINEWORLD_WRITE_FIXTURES=1 cargo test -p mineworld-cli --test
  needs_fixtures -- --ignored --test-threads=1`). [ ] Hashes and source in `SOURCE.md`. [ ] Expected values written by
  hand from the files.
- Validation: the GDScript check over the fixtures PASS.
- Commit: `test(fixtures): recorded observations of needs, sleep and lateness on two worlds (S35 N-e)`.
- **Push after this commit** so N-e2 and N-e3 can start.
- Implementation [ ] · Validation [ ] · Review [ ] (each "must contain" item present; no hand edit)

**C3 — 2D panels and the next row.**
- Scope: `clients/2d/scripts/hud/readers.gd` (`needs`, `asleep`, agenda next; `bars`, `button`), `hud/panels.gd`
  (`ProgressBar`, `Button`, `wake_pressed`, signature), `hud/words.gd` (`Words.word`), `scripts/app.gd`
  (connect `wake_pressed` → `open_menu(observer)`), `scripts/harness/interact.gd` (`--drive=rows`: read each fixture
  file into a `MineWorldObservation`, call `readers.panels`, print `ROWS <fixture> <locale> <json rows>`).
- [ ] Each arm per §4.3. [ ] Harness mode.
- Validation: `with-slot.sh godot --headless --path clients/2d -- --drive=rows --fixtures=tests/fixtures/observations/needs`
  prints the rows; compare by eye to `rows.en.txt` (written in this commit, by hand, from the fixtures);
  `check_client_rules.py` exit 0.
- Commit: `feat(2d): how you feel, asleep and the next part of the day in the panels (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ] (I-Ne-2: a pack-free fixture shows no new panel)

**C4 — 2D glyph, dim, notice and sleep.**
- Scope: `scene/people.gd` (`zz` label, `asleep_of`), `scripts/app.gd` (`_shade`, the notice toasts and their shown-id
  set), `scripts/intents.gd` (`sleep` composed), `tools/cli/tests/client_rules.rs` only if it lists 2D composed types
  (C0 checks; 13b precedent).
- [ ] Per §4.3.
- Validation: `check_client_rules.py` exit 0 (R2 with `sleep` composed); one local connected run on the
  `commute-town` copy (`./mineworld-2d` against `server --town`, via with-slot) to see the Zz and the toast;
  a screenshot to the ledger.
- Commit: `feat(2d): the Zz glyph, a dimmed night, lateness notices and sleeping for some minutes (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ] (the notice shows once per event; a reconnect does not repeat it)

**C5 — Hosted punctuality smoke and docs.**
- Scope: `tools/cli/tests/hosted_punctuality.rs` (AC-Ne-10, `#[ignore]`); `clients/2d/README.md` (one line: needs and
  sleep in the panels) and `clients/2d/PRESENTATION.md` if it lists HUD rows; this file's ledger.
- Validation: `cargo test -p mineworld-cli --test hosted_punctuality --no-run` builds (run at the milestone).
- Commit: `test(cli): a hosted punctuality smoke with needs and sleep on two worlds (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

### 7.3 N-e2 (`mvp0/pr-s35-ne-3d`)

**C0 — Base audit.**
- [ ] Base = N-e1's C2 commit (or `main` after N-e1 merged); #250 merged or not; #254, #245 merged or not (they edit
  `slice_link.gd`, `slice_main.gd`); the `WorldEnvironment` node name.
- [ ] **The clip audit.** Read the Quaternius UAL Standard pack's clip list from its zip or its preview video
  (quaternius.com pack page; opengameart.org/content/universal-animation-library). Record each clip whose name or
  pose is lying or sleeping. CC0 is already recorded for the pack (ASSETS.md); a Pro-tier clip is not bought. A
  "death" clip is not a sleeping pose and does not count.
- Implementation [ ] · Validation N/A · Review [ ]

**C1 — The slice HUD rows and the notice.**
- Scope: `clients/3d-spike/scripts/slice/needs_hud.gd` (new, §4.4, with static `rows`), `slice_main.gd` (attach it),
  `slice_link.gd` (`signal observed(obs)`), `slice/slice_probe.gd` or `slice_probe_world.gd` (`--slice-needs-rows`:
  print `ROWS` for each fixture and locale, then `all needs-rows checks pass`), `scripts/ci_godot.py` `MODES` gains
  `needs-rows` if the probe mode needs it.
- Validation: `with-slot.sh python3 scripts/ci_godot.py slice --slice-needs-rows` PASS locally; the rows equal
  `rows.<locale>.txt`.
- Commit: `feat(3d): how you feel, asleep and the next part in the slice HUD (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

**C2 — Dim and sleep/wake requests.**
- Scope: `slice_main.gd` or `needs_hud.gd` (environment dim); `slice/intents.gd` (`sleep` composed with `{"for": 480}`)
  and 16d's menu wiring — **only if #250 is merged**; else the `hud.asleep.move-to-wake` line, and the ledger records
  F-Ne-7's fallback.
- Validation: `tests/acceptance/tests/client_rules.rs` and `client_text.rs` (one binary each, via with-slot) PASS.
- Commit: `feat(3d): a dimmed night and, with the B menu, sleep and wake (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

**C3 — The sleeping figure.**
- Scope: `slice_link.gd` (the `Zz` Label3D, I-Ne-6 ordering); if C0 found a free clip: `tools/animation_trim.py`
  `DEFAULT_CLIPS`, the re-trimmed `assets/characters/quaternius_ual.glb`, `ASSETS.md` (five clips), and the clip
  played for a sleeping figure in `npc.gd`/`human.gd` (C0 names the call); a probe world fixture that permits sleep
  at the café (`configure/sleep.yaml` in a temp copy, as §5) for the probe `--slice-needs-sleeper`.
- Validation: the probe prints `Zz visible on <id>` and `all needs-sleeper checks pass`; one screenshot to the ledger.
- Commit: `feat(3d): a sleeper is drawn asleep (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

**C4 — Parity test and docs.**
- Scope: `tools/cli/tests/needs_rows_parity.rs` (AC-Ne-7; Godot-gated `#[ignore]`, added to `ci_godot.py`
  `GODOT_TESTS`), `clients/3d-spike/README.md` one line, `PRESENTATION.md` if it lists HUD lines; step-34 note
  proposed to the primary (§11).
- Validation: `cargo test -p mineworld-cli --test needs_rows_parity --no-run`; `python3 scripts/ci_godot.py coverage`
  if it lists ignored tests locally.
- Commit: `test(clients): the 2D and 3D needs rows are the same for the same observation (S35 N-e, AC-S35-10)`.
- Implementation [ ] · Validation [ ] · Review [ ]

### 7.4 N-e3 (`mvp0/pr-s35-ne-agent`)

**C0 — Base audit.**
- [ ] Base; LE-c merged (required for C3–C4); #261 merged or not; the SDK builder for a parametric offer
  (`mineworld_sdk.offers`, the one `walk_to` uses); the agent's JSON omission rule for `None` fields; LE-c's trigger
  seam name and `_clock` helper.
- Implementation [ ] · Validation N/A · Review [ ]

**C1 — `NeedsLine` and `AsleepLine`.**
- Scope: `clients/agent/src/mineworld_agent/{models,disclosed,view}.py`; `tests/test_needs_view.py` (AC-Ne-3);
  `tests/golden/tools.json` re-recorded.
- Validation: `uv run --locked pytest clients/agent/tests/test_needs_view.py clients/agent/tests/test_view.py
  clients/agent/tests/test_cli.py`; `uv run --locked ruff check clients/agent`; `uv run --locked pyright clients/agent`.
- Commit: `feat(agent): my needs and who is asleep in the view (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ] (a delta still answers a whole `me` when needs change; the cap holds
  on the crowd fixture)

**C2 — The agent `sleep` operation.**
- Scope: `player.py`, `mcp_server.py` (tool or `act` kind, §4.6), `models.py` (`SleepArgs`), tests (AC-Ne-4),
  `tools.json`; `test_needs_real.py` (AC-Ne-9, `real_server`, milestone).
- Validation: `uv run --locked pytest clients/agent/tests/test_needs_view.py clients/agent/tests/test_cli.py
  clients/agent/tests/test_mcp_tools.py`; ruff; pyright.
- Commit: `feat(agent): an agent can sleep for some minutes, through the same offer (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

**C3 — The LM prompt lines.**
- Scope: `cognition/lm-controller/src/mineworld_cognition/controller/{situation,prompt}.py`;
  `tests/test_needs_prompt.py` (AC-Ne-5).
- Validation: `uv run --locked pytest cognition/lm-controller/tests/test_needs_prompt.py
  cognition/lm-controller/tests/test_controller_parts.py` and the golden-render test (C0 names it); ruff; pyright.
- Commit: `feat(cognition): an LM seat reads how it feels and who is asleep (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ] (I-Ne-2: goldens unchanged; I-Ne-5: no plans line touched)

**C4 — (Q-Ne-1 yes only) The `feeling` trigger and the `sleep` act.**
- Scope: `controller/{triggers,prompt,choice,intent,seat}.py` (§4.7), the choice schema fixture
  `tests/fixtures/social_choice.schema.json`, `tests/test_needs_triggers.py` (AC-Ne-6), a cassette.
- Validation: that test file; the existing choice and intent tests; ruff; pyright.
- Commit: `feat(cognition): an LM seat notices its needs and may choose to sleep (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

**C5 — Docs.**
- Scope: `clients/agent/README.md` one line; `docs/MODULE_SPEC.md` §5 LM Controller paragraph (the `feeling`
  trigger and `sleep` act, if C4) — a spec edit in the same PR; this ledger.
- Commit: `docs(agent, cognition): needs and sleep in the agent view and the LM seat (S35 N-e)`.
- Implementation [ ] · Validation [ ] · Review [ ]

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| N-a/N-b disclose a different shape than §4.1 | C0 stops; the reader is one file, so a shape change is one edit |
| #250, #254, #245, S30-f edit the same 3D and 2D files | N-e2 rebases at C0 onto whatever merged; S30-f touches `readers.gd` in a different `match` arm |
| LE-c changes `prompt.py` concurrently | N-e3 C3 starts after LE-c merges (start condition) |
| The Market Town fixture seat is not asleep at 02:30 | C2 picks the seat from the run's own facts (a `fell-asleep` before 02:30 without a `woke`) |
| A word without a translation | the `PACK_WORDS` coverage turns `client_text` red |
| Agent `me` deltas grow because levels change every frame | `me` is already sent whole on change; three small fields; the cap test on the crowd fixture covers it |

---

## 9. Execution contracts

```text
PROJECT / PRs:           MineWorld mvp0, step-35 PR N-e as N-e1, N-e2, N-e3 (one design: this file)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s35-ne-presentation.md
BINDING:                 step-35 §§6.4, 6.5, 8, 9, 11; pr-le-leave-early.md revision 2 §0; SETTINGS.md §6;
                         ARC-47, ARC-70, ARC-283, ARC-284, ARC-286
WORKTREES / BRANCHES:    /Users/yuema137/mineworld-worktrees/impl-s35-ne-2d     mvp0/pr-s35-ne-2d     (N-e1)
                         /Users/yuema137/mineworld-worktrees/impl-s35-ne-3d     mvp0/pr-s35-ne-3d     (N-e2)
                         /Users/yuema137/mineworld-worktrees/impl-s35-ne-agent  mvp0/pr-s35-ne-agent  (N-e3)
                         one session per worktree
START CONDITIONS:        all: this design FROZEN. N-e1: N-a, N-b, LE-a, LE-b, N-c merged. N-e2: N-e1 C2 pushed.
                         N-e3: N-e1 C2 pushed; C3–C4 also LE-c merged; C4 only if Q-Ne-1 is ruled yes
APPROVED SCOPE:          §3.1 (C4 of N-e3 per Q-Ne-1)
FROZEN INVARIANTS:       I-Ne-1 … I-Ne-6; §4.2 keys; §6 thresholds
ALLOWED COMMANDS:        godot (headless checks, probes) and cargo (one test binary) via cast-slots/with-slot.sh;
                         uv run --locked pytest <files> / ruff / pyright; python3 scripts/*; git; gh (pr create/update)
NEVER:                   python3 -c; sed -i; perl -pi; curl/wget; editing kernel/, contracts/, persistence/,
                         server/, systems/, cognition/rule-controller, cognition/motivated, any reference world;
                         LE-c's plans, upcoming, complaint or go code; a local full-workspace test run; buying assets
MATERIAL STOPS:          a disclosure missing or shaped differently (C0); a fixture that cannot be recorded; a §6
                         threshold changed after a run; a pack-free golden moving
VALIDATION (SPEED FIRST): the commit's quick check, `fast` on each PR; §6.2 at the milestone
PLATFORMS:               Linux, macOS, Windows (CI at the milestone)
LIVE DOCUMENTATION:      this file, §10 ledger
ENDPOINT AUTHORITY:      implementation, semantic commits, push, PR create/update; the lane lead merges after `fast`
STOP CONDITION:          each PR open, `fast` green, ledger current
```

---

## 10. Ledger (live during implementation)

### Evidence

- 2026-10-18: design written from `origin/main @ a8bba658`; audit §2.
- 2026-10-19 **N-e1 C0 (base audit, ledger only).** Base `mvp0/pr-s35-nc` @ `036f7fbb`, which carries N-a
  (`systems/sleep`), N-b (`systems/needs`), LE-a/LE-b/LE-c (merge `3b0bdc2b`) and N-c (`e0a9ea9c`…`036f7fbb`).
  None is merged to `main`. **Start condition changed by the lane lead (SPEED FIRST, charter):** "N-a, N-b,
  LE-a, LE-b, N-c merged" is replaced by "stacked on N-c @ `036f7fbb`". N-d, #250, #261 and S30-f are not
  needed by N-e1 and were not checked here (N-e2 and N-e3 check theirs at their C0).
  Disclosures checked against §4.1 (all match, no stop):
  - `needs` (`systems/needs/src/disclose.rs`): payload `hunger`, `tiredness` (levels, the numbers `payload[need]`
    reads), `words {hunger, tiredness}` (kebab words `ok, peckish, hungry, famished` / `ok, sleepy, tired,
    exhausted`), `asleep`, rates, restore, nourishment; holder only.
  - `asleep` (`systems/sleep/src/offer.rs`): `{asleep, since, alarm?}`, `alarm` only to the sleeper.
  - `agenda.next` (`systems/schedule/src/component.rs` `NextPart`): `{place, label, from, punctual}`; `from` is an
    integer instant (`WorldTime` is transparent i64).
  - `late-for-shift` (`systems/employment/src/event.rs`): envelope `payload.payload` holds `employee`, `employer`,
    `manager?`, `shift`, `late_by`, `count`, `level` (`notice|warning|final`), `absent`. Typed refs are
    `{entity, entity_type}`.
  - `sleep` offer (`systems/sleep/src/action.rs`): parametric, payload key `for`; refusal
    `sleep-for-out-of-bound` (`systems/sleep/src/system.rs`).
  Catalog and seam findings:
  - `action.sleep`, `action.wake` and `action.penalize` are in no shared catalog (N-a and LE-b did not add
    them); `client_text` requires them (AC-SET-4). C1 adds them; `action.penalize` is LE-b's gap, recorded here.
  - `format.clock` is 13b's wording and sits in the 2D pack layer (it is in `aee8290`). The verbatim-move test
    forbids a 13b key leaving the pack, so C1 does **not** move it. The 3D slice needs it at N-e2 C1: see §12
    question Q-Ne-3.
  - `INPUTS` (`clients/2d/scripts/intents.gd`) holds `suggest` as **one** key whose text is comma-separated
    (`menu.gd` splits it). §4.2's two suggestion keys do not fit that form; C3 decides (see the Deviations).
  - `PACK_WORDS`: no readable seam (the enums are not parsed by the test). C1 writes the list literally, with the
    source files named in a comment (as §4.2 allows).
  - `scripts/check_client_rules.py` had no `--self-test` mode; C1 adds one for R7.
  - The unused-key rule of `client_text` requires each key to be named by a script at the commit that adds it, so
    the `ui.*`, `panel.*`, `hud.*` and `suggest.*` keys wait for the commits that use them.

### Findings

- F-Ne-1 … F-Ne-9 (§2.2).
- **F-Ne-10** (design): a walking-time estimate in the human HUD is not in step §8; recorded for a later PR.

### Commits (N-e1, `mvp0/pr-s35-ne-2d`)

- C0 `f5402213` docs(plan): base audit and start condition.
- C1 `c1f781ea` feat(clients): shared reader, wording, R7. Validation: `needs_check.gd` PASS (14 claims, synthetic);
  `client_text` PASS (5 tests); `check_client_rules.py` and `--self-test` PASS; `messages.pot` regenerated.
- C2 **`41405b82`** test(fixtures): the three recorded observations and `needs_check.gd`'s real-fixture claims.
  Validation: `needs_check.gd` PASS (25 claims, 11 of them real, 0 failures) on the working tree. **C2 alone is not green**: its
  lateness claims need `acd4c7fc` (below). C2 is not rewritten.
- `acd4c7fc` fix(clients): a lateness record carries its event id (the C2 claims expect it).
- C3 `9c6b5342` feat(2d): the rows, the bars, the asleep panel with its button, the next part. Validation: the rows
  script's output equals `rows.en.txt` and `rows.zh_Hans.txt` (diff, both PASS). **C3 alone is not green** in two
  places, fixed by the next commits: `panels.gd` lacked its `Words` preload (`fdc5446b`, found by `godot --check-only`),
  and `messages.pot` was not regenerated (`client_text` red; regenerated in C4).
- C4 `16ad15a4` feat(2d): Zz, dim, notices, sleep composed. Validation: `check_client_rules` PASS and `--self-test`
  PASS; `client_text` PASS (5); `client_rules` PASS (3); `godot --check-only` PASS on app, intents, menu, people,
  panels, readers, words, harness. The connected run and screenshots are deferred to the milestone (SPEED FIRST).
- C5 (this commit): the hosted punctuality smoke (`cargo test -p mineworld-cli --test hosted_punctuality --no-run`
  builds, PASS), the 2D README and PRESENTATION lines, this ledger.

### Commits (N-e2, `mvp0/pr-s35-ne-3d`)

- **N-e2 C0 — base audit (ledger only).** Branch `mvp0/pr-s35-ne-3d` is created from N-e1's head `09bcda18`
  (PR #309, OPEN, green), not from N-e1's C2, by the lane lead's decision (SPEED FIRST). N-e1 C0–C5 are all in
  the base. Merged to `main` at the audit: #254 (12e bodies), #245 (16c living street), #261 (one `act` tool).
  **#250 (16d, the B self menu) is OPEN**, so C2 takes the F-Ne-7 fallback: no `sleep` composition and no
  `wake` menu wiring in 3D; the `move to wake up` line only.
  - **Q-Ne-3 ruled** by the lane lead (decided, bounded): `format.clock` moves from the 2D pack layer to the
    shared layer in C1, and `format.clock` is added to the moved set of the verbatim-move test
    (`tests/acceptance/tests/client_text.rs`, SD-SET-a-17) and nothing else.
  - **Clip audit (§7.3 C0).** A web search (2026-10-19) for the Quaternius Universal Animation Library Standard
    clip list found no lying or sleeping clip named (sitting and death are listed; sleeping, lying, idle are not
    named). The zip's own clip list was not read, so the result is **INCONCLUSIVE**. A "death" clip is not a
    sleeping pose and does not count. Per the design's fallback, the sleeping figure keeps its idle pose and
    carries the Zz label; no clip is added to `animation_trim.py`.
  - **Parity finding (for the lane lead).** The 2D `ROWS` harness (`scripts/harness/needs_rows.gd`) prints every
    row of the `needs`, `asleep` and `agenda` panels, so the oracle `rows.<locale>.txt` includes the 13b row
    `ui.row.agenda` ("home at 1, …"). That key is 13b's (`aee8290`, in the 2D pack) and is not in Q-Ne-3's
    single-key move. The 3D HUD shows the next part only (§3.2, step §8), so it cannot word `ui.row.agenda`
    without a second move or a duplicate. Recommended answer: parity compares the rows N-e owns (needs, asleep,
    the next part); the oracle drops the 13b agenda row. Taken as a bounded deviation and flagged in the handback.
  - Disclosures re-checked on the N-e1 base: unchanged from the N-e1 C0 entry.
- **N-e2 C1–C3, one commit `7c1a0a7b`** `feat(3d): how you feel, asleep and the next part in the slice HUD, a dim
  night and a sleeping Zz (S35 N-e)`. Validation: `client_text` PASS (5) after `MINEWORLD_WRITE_POT=1` regenerated
  `messages.pot` (the same binary PASS without the variable); `check_client_rules.py` and `--self-test` PASS (R7
  covers the 3D scripts). **Godot: INCONCLUSIVE.** The 3D project's first import did not finish in this session
  (about 25 minutes, 245 of about 310 imports; killed to free a slot), so `needs_rows_probe.gd` has not run and the
  GDScript is not compiled. The milestone runs it.
- **N-e2 C4 (this commit).** `tools/cli/tests/needs_rows_parity.rs` (AC-Ne-7; `#[ignore]`, Godot-gated, listed in
  `ci_godot.py` `GODOT_TESTS`); the oracle `tests/fixtures/observations/needs/rows3d.<locale>.txt` (hand-written
  from the fixtures: the N-e rows only, see the parity finding above); `clients/3d-spike/README.md` one line.
  Validation: `cargo test -p mineworld-cli --test needs_rows_parity --no-run` builds (PASS); `ci_godot.py`
  compiles (PASS). The ignored test is not run (milestone).
- **Deviations (N-e2).**
  - **C1–C3 in one commit.** `slice_link.gd` carries both the observed signal and the Zz, so the three were not
    split (no interactive staging). Each is green on its own only in the sense of the checks above.
  - **Parity oracle.** The 3D rows omit the 13b `ui.row.agenda` row (see the parity finding). `rows3d.*.txt` is a
    new file; the 2D `rows.*.txt` is unchanged.
  - **`SliceNeedsHud.rows(obs, place_of)`** takes no `name_of`: the N-e rows name no person.
  - **`observed` is emitted at the top of `_on_observed`**, not the end, because the early returns for "no place"
    and "doorway not disclosed" would otherwise hide the needs rows.
  - **The wake hint is its own line**, outside `rows`, so the parity rows equal the 2D's. It shows while asleep.
  - **No sleep composition and no wake menu in 3D** (#250 OPEN; F-Ne-7 fallback). The 3D client cannot start a
    sleep. Q-Ne-2's follow-up is needed after #250 merges.
  - **No lying clip** (C0 audit INCONCLUSIVE, fallback taken); a sleeper keeps its idle pose with the Zz.
  - **C3 probe mode not written.** The `--slice-needs-sleeper` probe and its café-sleep world copy are not in
    this PR; the sleeping figure is covered only by the code path above. Recorded as a follow-up for the milestone.
  - **Notice set** is kept per node lifetime (not per instance change, as the 2D does).

### Deviations

- **Start condition** (lane lead, SPEED FIRST, 2026-10-19): "stacked on N-c @ `036f7fbb`" replaces "N-a, N-b, LE-a,
  LE-b, N-c merged". Recorded in C0.
- **C1 — `action.sleep`, `action.wake`, `action.penalize`** were in no shared catalog; `client_text` requires them.
  Added. `penalize` is LE-b's gap. `reason.sleep-for-out-of-bound` added for the sleep refusal.
- **C1 — `format.clock` not moved** to the shared layer (13b's verbatim-move test). Raised as **Q-Ne-3**; N-e2 C1
  needs the answer. The 2D keeps its own copy.
- **C1 — `check_client_rules.py --self-test`** did not exist; added for R7. `PACK_WORDS` is a literal list (§4.2).
- **C1 — `lateness` records carry `id`** (`acd4c7fc`): each event is told once; C4 uses it.
- **C2 — the recorder's command** adds `--solo` (the server refuses `--time-scale` above 24 without it) and
  `--hold 0`. The design's command had neither.
- **C2 — `commute-late.json`**: Ben is never walked to the bakery; the fact is the absent `late-for-shift` (the
  shift's end), not a late arrival. Both are the same event.
- **C2 — `market-town-night.json`**: no seat asleep at 02:30 was a punctual worker; the next part is required, not
  its punctuality (punctual is in `commute-late.json`).
- **C2 — the sleeper's walk** finds the street and the bakery through the `passages` of the place the seat stands
  in (`to_tags`), because the street is not in view from a house.
- **C3 — the rows harness** is a SceneTree script, `scripts/harness/needs_rows.gd`, not an `interact.gd` drive: a
  drive needs a live connection, which the recorded fixtures do not have.
- **C3 — the asleep panel's button** is `button: true`, not `"wake"`: the literal `"wake"` is an action type, and
  `client_rules` forbids it outside `intents.gd`.
- **C4 — `INPUTS.suggest` may be a list of keys** (`menu.gd` splits each): §4.2's two suggestion keys do not fit the
  one-key form.
- **C4 — the notice set is cleared only when the world's instance changes**, not on every welcome: a reconnect to
  the same world would otherwise repeat each notice.
- **C4 — `client_rules`'s `ACTION_LITERALS`** gains `sleep` (intents: composed) and the harness's wait step `sleep`.
- **C5 — `needs.gd` and the rows**: the C1 `needs.gd` gains `id` (above); nothing else in the reader changed.

---

## 11. Proposed edits to shared documents (for the primary session)

- `step-34-posture.md`: a dated note — "`Asleep` (step 35) overrides any posture in presentation; N-e implements it in
  both clients" (Q-S35-10 asked the posture step to record it; it does not yet, A-15).
- `step-35-needs-and-sleep.md` §8: a dated note that the strings live in the shared catalog layer (F-Ne-1) and that
  the prompt bound is per line (F-Ne-9, A-20).
- If Q-Ne-1 is yes: a dated note on `ARC-286` (the LM seat's `feeling` trigger and `sleep` act, narrowly amending
  I-P6-3 as `ARC-282` did for `go`), written in N-e3 C5.

---

## 12. Questions

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-Ne-1** [OPERATOR] | An LM-held person never considers hunger or sleep: nothing triggers a thought about it, and it cannot ask for `sleep { minutes }` (F-Ne-6, F-Ne-5). Add a `feeling` trigger (when the need words change, or on waking) and a `sleep` act with minutes, the same narrow amendment as LE's `go` (Q-LE2-1)? | **Yes**, in N-e3 C4. It follows the ruling that an LLM decides from the disclosed facts; the server still validates every request; the `rule` fallback is unchanged |
| **Q-Ne-3** (N-e1 C0, raised by the implementer) | `format.clock` is 13b's key in the 2D pack layer. The 3D rows (N-e2) need it in the shared layer, but moving a 13b key trips the verbatim-move test (`client_text.rs`, SD-SET-a-17), which forbids a 13b key leaving the pack. Move it, and widen that test's moved set to `format.clock`; or copy it to the shared layer with `#. override` in the pack? | **Move it and widen the test's set by one key**, in N-e2 C1. The verbatim rule's purpose is that the wording does not change; a move keeps the text, and the test keeps checking it. Duplicating it would give two copies to keep equal |
| **Q-Ne-2** | The 3D client gets sleep and wake buttons only through 16d's B menu (PR #250, open). Until #250 merges, a 3D player sees the rows and "move to wake up", and cannot start sleeping in 3D. Accept? | **Yes.** N-e2 wires them at C2 if #250 has merged by then; otherwise a one-commit follow-up after #250 |

Everything else follows the frozen step-35 and LE revision 2.

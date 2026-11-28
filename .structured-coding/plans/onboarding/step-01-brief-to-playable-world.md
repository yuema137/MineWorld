# Onboarding step 01 — From a loose brief to a playable world

**Lifecycle:** `FROZEN` — **DESIGN FROZEN 2026-11-27** by operator ruling, relayed by the coordinator: "其他问题都用你的推荐"
(QON-1 … QON-9 as recommended), together with the QON-7 correction "用真实 agent 跑示例测试 不会花钱啊，用订阅就行"
(§8.1). Frozen after the red-team pre-freeze review (`/Users/yuema137/mineworld-worktrees/_reviews/design-onboarding.md`,
reviewed head `eaee7ad3`, verdict FREEZE-AFTER-FIXES); its required fixes F1–F15 are applied as bounded corrections
that change no ruled decision (§12). Base: `origin/main @ c373b802`. Frozen: scope, invariants (§7), acceptance (§8),
PR split (§9). Writable: progress, evidence, bounded corrections. §10.2–10.4 stay a non-binding survey (QON-9).
Each PR of §9 still needs its own frozen design; ON-a's is
[`pr-01a-onboarding-docs-and-briefs.md`](pr-01a-onboarding-docs-and-briefs.md).
**Effort:** `onboarding` (new effort directory). There is no `onboarding/overall.md`: this effort is one step until
§9's later PRs prove otherwise; the parent is `mvp0/overall.md` "Operator requirement recorded 2026-11-22 — intent to
a playable world".
**Author:** onboarding design session, 2026-11-27. Worktree `/Users/yuema137/mineworld-worktrees/design-onboarding`,
branch `docs/design-agent-onboarding`, from `origin/main @ c373b802`.
**Binding parents:** `CLAUDE.md` §§1, 1.1, 2, 4 (rules 2, 3, 5, 9, 10, 13, 16); `docs/VISION.md` §2.4, §2.5, §5
"Phase 2 — World Creator" / "The creator experience this is all for"; `docs/MODULE_SPEC.md` §8 (CLI) and §6.1.1 (far
field, `ARC-102`); `docs/GUIDE_FORMAT.md`; `tools/onboarding/PROTOCOL.md`; `mvp0/step-25-creator-docs.md` §§9–10 and
INV-S25-6, INV-S25-7; `mvp1/step-21-regions-travel.md` §§5.3–5.6, 5.12; `mvp0/operator-playable-mvp-20261123.md` §7.

---

## 0. Freeze questions (plain language) — all ruled 2026-11-27 as recommended

| # | Question | Recommendation |
| --- | --- | --- |
| QON-1 | Is the brief a **scaffold** (a short must-state checklist + worked examples, everything else free) rather than a schema? | **Yes.** No brief parser, no brief schema, no required headings. The user's agent is the parser. The framework's contracts (World Pack files, `mineworld validate`) stay the only machine-checked input (§3). |
| QON-2 | When a brief leaves something out, does the agent ask or default? | **Default almost everything and write the default down; ask at most one round of questions, and only when the answer changes what kind of world it is** (play vs headless research, or two readings of the premise that lead to different worlds). Everything else gets the documented default (§3.3) and is listed in the world's `brief.md` "What the agent decided" section, so the user can change it in the next iteration. |
| QON-3 | Where do the onboarding documents live? | **`docs/onboarding/`**: a short `README.md` for humans, `ONBOARDING.md` (the one document an outside agent reads), `BRIEF.md` (the checklist and defaults), `briefs/` (the three operator-named examples). Not a guide chapter: `docs/guide/` pages are executed by the guide test line by line (`GUIDE_FORMAT.md`), and a brief's outcome is not deterministic text. `ONBOARDING.md` links into the tested guide for every command instead of repeating it. |
| QON-4 | How does a world an agent just wrote get **played in 2D and 3D**, when the clients refuse a world whose Presentation Pack has no `far_field/<world-key>.yaml` (no fallback, `ARC-102`) and the bundled packs carry far fields only for the bundled worlds? | **Keep `ARC-102`. The agent writes a world-local Presentation Pack beside the world** carrying `far_field/<world-key>.yaml` (and, optionally, depictions), seeded from a small reviewed set of setting archetypes (coast, city, mountain, plain …) and checked by the existing `mineworld packs validate`. No generic fallback far field. The 3D client already joins any hosted world (`--server … --world-key … --presentation=<dir>`) and draws places with no depiction plainly (`clients/3d-spike/PRESENTATION.md` §6); what is missing is the archetype seeds, a scaffold command, the self-hosting convenience for a world outside `worlds/`, and a per-world pack in the bundle (§6 ON-c). Needs the presentation owners' review. |
| QON-5 | Which example briefs gate acceptance? | **The seaside resort gates as a played world; the metropolis gates as a declared neighbourhood slice used both played and as a year-long headless research run** (different use, scale, population packs and evidence: the 11-22 "materially different" requirement is met by use and scale, not only by setting — recorded explicitly here because both use the town System Pack set). **The Italian mountain village is run and reported but does not gate** until the required `vehicles` step (§10, QON-9) delivers driving; it is then the brief with a materially different interaction system. Its round must end with driving classified **real product gap**, not faked (§5.3). |
| QON-6 | Release placement. | **After the first playable MVP.** PR ON-a (documents only) can merge now; it touches no MVP lane. ON-b … ON-f wait until the playable MVP is accepted, then queue in §9's order. The 11-22 acceptance (§8) is not claimed by ON-a. |
| QON-7 | Agent harness and cost. | **Real-agent brief rounds run on the operator's existing subscription logins on this machine (Claude Code; Codex CLI), not metered API keys: $0 incremental** (operator correction, 2026-11-27, §8.1). Reuse `tools/onboarding` (allowlisted, sandboxed, never in CI, ≤ 80 turns). Bounded: at most 3 brief rounds per agent CLI per milestone, one at a time, started and attended by the operator, within the plan's normal interactive limits and the vendors' terms (§8.1). A Codex round is reported, not gating, until an equally strict allowlist is shown for it (no `bypass` mode). A **round** is one `run.py` invocation: ≤ 80 turns, ≤ 3 600 s wall time, `--max-budget-usd` ≤ 5 kept as the CLI-reported ceiling (QS25-3; raising it would need an amendment of QS25-3 and `run.py`, operator-approved). ON-g total: BR-1/2/3 once each plus at most one rerun per brief after a recorded fix. Paid asset generation (ON-c archetypes) is USD 0 unless separately approved in ON-c's design. Credentials: inherited from PROTOCOL §3.2 (operator's login; no key reaches the agent; `run.py` never reads or prints one). |
| QON-8 | Does the agent author through MCP? | **No.** Authoring is files + the `mineworld` CLI, which every agent (Claude Code, Codex, any shell-capable agent) already has. The MCP adapter (`clients/agent`) is used for what it is for: the agent (or a second agent) *plays* the world it built, which is the "real server/client interaction" evidence (§8). |
| QON-9 | Vehicles are required (operator, 2026-11-27). How do they enter the plan? | **As their own step and PR series (`vehicles`), after the playable MVP, not inside onboarding. This question approves placement only.** §10 is a **non-binding reuse survey and direction**: it lists the accepted decisions any vehicle design must amend or satisfy (`DEP-13`, `ARC-67`, `ARC-50`, `ARC-25`, S21 §5.4.1) as open questions the vehicles step must answer with its own `ARC-VH-*` / `DEP-VH-*` records before code. Onboarding's BR-3 becomes gating once `vehicles` delivers driving. |

---

## 1. The requirement

### 1.1 Operator's words

2026-11-22 (recorded in `mvp0/overall.md`): anyone composes worlds, objects and interactions, structured memories and
strategies on MineWorld; onboarding must ultimately accept a few sentences or a simple Markdown brief and let an agent
build and launch a playable world **in one user-facing step**. Evidence must include at least two materially different
briefs, generated artifacts, a real server/client interaction, persistence and restart, elapsed time and manual
interventions; existing guide checks and recorded model-free MCP sessions are prerequisites, not proof.

2026-11-27, verbatim:

> 这个是个框架，不是一个游戏，让别的agent来onboarding，根据一个specification可以直接生成新的游戏世界… specification的格式我们需要给example，但是不需要完全定死，因为我们预设的就是用户会跟自己的agent合作，在很高的自由度里逐渐构建自己的世界，specification只是提供一个雏形，保证必要的东西都设置了，但是不要用过于定死的specification限制user的想象

2026-11-27, the example set, verbatim:

> demo需要有如下几个：1. 海边度假村 2. 纽约一样的大都市 3. 意大利山村，需要开车走山路才能去别人家那种

Meaning, in this design's terms: the brief is a *starting shape* that guarantees the necessary things are stated, never
a schema that limits imagination; the user and their own agent grow the world iteratively, each iteration immediately
playable; the worked examples are exactly (1) a seaside resort, (2) a New-York-like metropolis, (3) an Italian mountain
village where visiting someone means driving a winding mountain road.

### 1.2 What "one user-facing step" means here

The user gives their agent one message: the brief (or a path to it) and "build it". Everything after — reading the
onboarding document, choosing packs, writing files, validating, running headless, hosting, opening a client — is the
agent's. The user's next contact is a running world plus a short report (§4.5). Iteration is more single steps of the
same kind ("make the beach bar open at night"), each ending with the world playable again.

### 1.3 Framework constraints that bound every answer

- `install modules → compose world → configure → run`; never fork source. An agent may write **World Pack files,
  Entity Pack files and a world-local Presentation Pack**; it never edits kernel, System Pack, client or CLI code to
  satisfy a brief. A brief that needs a capability no installed pack provides is a **product gap** (§5), reported, and
  optionally answered by *designing* a new System Pack — never by smuggling rules into data or clients (`CLAUDE.md`
  §4 rules 3, 5, 15).
- Every generated world runs with no language model (`VISION.md` §2.4). LM seats are optional and always have a rule
  fallback.
- Headless research is first-class: a brief may ask for no renderer at all.
- Both reference clients remain first-class; a "playable" claim names which client(s) were actually run.
- All three platforms (macOS, Linux, Windows) are supported. The onboarding documents name each platform's launch
  path (checkout bash scripts on macOS/Linux; the bundle launchers or the `mineworld` binary plus `clients/2d/README.md`
  on Windows); ON-c and the vehicles step list all three in acceptance. Operator-run agent rounds run on the
  operator's macOS machine; that is recorded as a coverage limit, not a claim for the other two.

---

## 2. Audit — what exists (origin/main `c373b802`)

| Need | What exists | Where | Verdict |
| --- | --- | --- | --- |
| Start a world | `mineworld create DIR` writes a minimal World Pack (2 people, 1 place, presence/movement/conversation, `settlement: kind: none`) | `tools/cli/src/create.rs`, `tools/cli/templates/new-world/` | Reuse as the agent's first command. |
| Check a world | `mineworld validate DIR` (the operator's "check"; there is no `check` subcommand). Refusals name file, line, column; every refusal and its fix is in guide ch. 13 | `tools/cli/src/main.rs`, `docs/guide/13-when-something-is-refused.md` | Reuse. Gap: no *playability* warnings — a place no door reaches validated (`F-25f-1`), S29-a covers towns only (§6 ON-e). |
| Run headless | `mineworld run DIR --headless --seed N --days N [--save DIR]`, deterministic, prints per-day lines, `faults`, rejected requests | `tools/cli/src/run.rs` | Reuse; it is the agent's inner loop. |
| Persistence / restart | `--save`, `inspect`, `replay`, resume of killed run | CLI | Reuse for §8 evidence. |
| Inspect a world | `interactions`, `attributes …`, `decisions`, `biography`, `perceived`, `packs list/show/validate/resolve`, `reference --out/--check` | CLI | Reuse. |
| Know what can be composed | `packs list` (ids, versions, bundled/third-party); generated `docs/reference/` (`world-pack.md`, `sections.md`, `configure.md`, `interaction-list.md`, `decisions.md`, `memory-scopes.md`, `cognition-toml.md`, `agent-tools.md`, `schema/`) | `docs/reference/` | Mostly exists. Gap: no single machine-readable **capability index** per System Pack — actions it offers, sections it owns, configure keys, packs it needs (§6 ON-d). |
| Learn to author | 13 tested guide chapters (Corner Shop), compare pages, example world | `docs/guide/` | Reuse; it is the tested authority for commands. It is a human tutorial in one fixed world, not a brief→world path. |
| Fresh-agent onboarding test | `tools/onboarding`: sandbox (guide + reference + binary + `entities/` only), allowlisted Claude Code, guard, audit, scorer (S1–S8, P1), rounds ledger; ON-1/2/3 instruction sets; round 3 passed every check but live S7 | `tools/onboarding/PROTOCOL.md`, step-25 §20.3 | **Reuse as the acceptance harness** (§8). It has no 2D/3D, restart or agent-play checks yet. |
| Agent plays a world | `mineworld-agent mcp` (stdio / Streamable HTTP), grants, guide ch. 14 | `clients/agent/`, `docs/reference/agent-tools.md` | Reuse for the server/client evidence. |
| Play in 2D | `./mineworld-2d --world PATH` hosts any World Pack path; `--presentation=<dir>` takes any 2D Presentation Pack; needs Godot 4.7 and a checkout (bash; Windows per `clients/2d/README.md`) | `mineworld-2d` | Partly. The pack must carry `far_field/<world-key>.yaml`, no fallback; `--presentation=none` is a diagnostic drawing, never play (MODULE_SPEC §6.1.1, INV-ON-6). The bundled 2D pack carries far fields for the bundled worlds only. |
| Play in 3D | `./mineworld-slice --server=host:port --invite=TOKEN --world-key=KEY --presentation=<dir>` joins **any** world hosted by `mineworld server <any dir>`; a world with no depiction is drawn plainly from its disclosure (`SlicePlainDepiction`, `clients/3d-spike/PRESENTATION.md` §6; AC-16f-10 drew Lakeside plainly with the greybox pack). Only the self-hosting `--world=<pack>` convenience is limited to `worlds/<pack>` (`mineworld-slice` ~l. 310). The far field is required. `presentation/mineworld-greybox/3D` (MIT, small) is a natural seed for a world-local 3D pack | `mineworld-slice`, `clients/3d-spike/PRESENTATION.md` | Partly: far field per world (ON-c), self-hosting convenience for an outside directory (ON-c). Not yet run on a world outside `worlds/` — ON-c's first evidence item. **Recorded contradiction:** guide ch. 06 l. 83 ("The 3D client today draws the café of the sample town only") is stale against the code above (`CLAUDE.md` §2.1 rule 4); the guide is wrong, and ON-c fixes the sentence and its mirror with run evidence. |
| Bundled launcher | `tools/launch`: arguments are fixed (`LAUNCHER.md` §3), but `runtime/launch.toml`'s `world` may name any folder under `runtime/worlds/` (§6); `scripts/package.py` fixes which worlds and which Presentation Pack (`mineworld-default`) are bundled | `tools/launch/LAUNCHER.md` §§3, 6; `scripts/package.py` | Partly: a copied world plays in the bundle except that the bundled pack lacks its far field — ON-c adds a per-world Presentation Pack to the bundle path, not a new argument. |
| Bigger worlds | `tools/worldgen/tile.py` copies a pack into disconnected blocks (scale gates) | `tools/worldgen/` | Reuse as the base of a connected-grid generator (metropolis, §6 ON-f). |
| Capabilities the briefs touch | Bundled System Packs: presence, movement, conversation, relationships, group-activity, schedule, naming, persona, item, inventory, item-transfer, economy, employment, consumption, needs, sleep, posture, bodies, appearance, attributes, calendar, weather, happenings, notices, price-survey; Entity Packs `modern-goods`, `person-attributes`, `populations-us`; LM decision model with fallback | `systems/`, `entities/`, `cognition/` | Rich for a town; **no** transport/vehicles/regions/atlas (S21 is a step design; RT-c not built; self-driving is §5.4.1 "design only; no PR"), no lodging/reservations, no swimming or water activity. |
| Authoring brief / onboarding doc for an outside agent | none | — | **Missing** (§6 ON-a). |

Prior art outside the repository was considered for the brief's form: game-design one-pagers and tabletop "session
zero" checklists both use a short must-answer list plus free prose, which is the shape chosen here. No library is
adopted: the brief is prose that the user's agent reads; there is nothing to parse (`REUSE_POLICY.md` — no record is
needed for choosing *not* to build a component).

---

## 3. The brief

### 3.1 Form

A brief is **any text**: a few sentences in a chat, or a Markdown file. Any language. No required headings, no fixed
order. `BRIEF.md` offers an optional skeleton (the eight headings of §3.2, defined once there; an example may rename a heading and says so) for users who like one; the worked examples
show both a skeleton-shaped brief and a free-prose one, deliberately.

### 3.2 The must-state checklist (what the agent must *know* before building, not what the user must *write*)

| # | Item | Stated by the user, or else the agent… |
| --- | --- | --- |
| M1 | **Premise** — what kind of place, and what life there is like, in a sentence or two | **asks** if it is absent: it is the world's identity |
| M2 | **Use** — played (2D, 3D, both), headless research, or both | defaults to **played in 2D and 3D, also runnable headless**; asks only if the brief reads as research-only and also mentions a player |
| M3 | **Player** — who the player is (an arriving visitor, a resident, staff, nobody) | defaults to **one seat, a newcomer** (`visitor`) for played worlds, no player seat for research-only |
| M4 | **Places** — the places that matter and roughly how they connect | defaults to the premise's obvious places (3–8) on one street/square, every place reachable |
| M5 | **People** — how many, and any who matter by name or role | defaults to **6–12 residents**, named for the setting, each with a routine; scale caps in §3.4 |
| M6 | **Activities** — what happens in the world (for a town: work, trade, eat, sleep, meet, events; for another kind of game, whatever its interactions are) | defaults to the bundled town set (routines, conversation, relationships, needs, sleep, money, work, shops, a weekly happening) only when the premise is town life; every choice maps to an installable System Pack (§4.2) |
| M7 | **Minds** (user-facing word; agent-facing: each Person's **Controller** / decision model, `CORE_CONCEPTS.md`) — rules only, or some people on a language model | defaults to **rules only** (runs with no LM, costs nothing); LM decision models only when asked, always with a rule fallback, local model first |
| M8 | **Limits** — size, time span to simulate, money/time the agent may spend | defaults to the §3.4 caps; spending defaults to **none** (no paid model, no paid asset generation) |

Everything else is **free**: tone, history, names, weather and season, real-world location, stories, rules of
etiquette, house styles, what the user hopes to feel. The agent uses it where a pack can carry it (names, notes,
attributes, routines, interaction-list rules, calendar epoch, weather location) and keeps the rest in `brief.md`.

### 3.3 Ask-or-default rule

1. Read the whole brief. Map each statement to M1–M8 or "free".
2. If M1 is missing, or M2/M1 admit two readings that produce materially different worlds, ask **one** batch of at most
   three questions, each with the default the agent will use if the user says "just go".
3. Everything else: take the default, build, and list every default in `brief.md` under "What the agent decided".
4. Never ask about something a pack's default already answers well (doors, walking speed, body radius — real-world
   defaults are the framework's job).
5. A request no installed pack can satisfy is not a question; it is a gap (§5).

### 3.4 Default caps (a first iteration must stay playable on one reference laptop — macOS, Linux or Windows)

People ≤ 40, places ≤ 25, simulated span for the agent's own check 14 days, LM seats 0. Above them the agent builds a
declared **slice** first (named in `brief.md`), plays it, and grows it in later iterations. The caps are documented
defaults, not loader limits; they follow the measured state in `operator-playable-mvp-20261123.md` §7.2 (no capacity
claim above the scale gates' 100 people exists yet).

---

## 4. The agent path

### 4.1 Inputs the agent is given

Only: the brief, `docs/onboarding/ONBOARDING.md` (which links to `BRIEF.md`, the examples, the tested guide and the
generated reference), and the `mineworld` binary (plus the clients when the user plays). In a checkout that is the
repository; in a downloaded bundle it is the bundle's `docs/` (packaging is ON-c/ON-f's concern). The agent needs no
MineWorld source code.

### 4.2 The loop

```text
read brief → M1–M8 (ask once or default)                         (§3.3)
→ capability map: each wanted behaviour → bundled pack, Entity Pack,
  configure section, or GAP                                      (packs list / reference; ON-d makes it one command)
→ mineworld create worlds/<id>                                   (template)
→ write places, people, items, organizations, routines,
  configure/*, calendar, weather, seats, decisions               (guide chapters 02–12 are the how-to)
→ mineworld validate  ──refused──▶ fix by the refusal's name (guide ch. 13); never edit code
→ mineworld run --headless --seed 1 --days 14 --save …           (faults 0, no `rejected` line, every place entered)
→ inspect: biography / perceived for 2 people; interactions      (the world does what the brief said)
→ presentation: world-local Presentation Pack (far field,
  depiction) from an archetype                                    (ON-c; until then: report "not yet drawn"; never diagnostic mode)
→ host: mineworld server … --save … ; open 2D and/or 3D;
  optionally join one seat itself over MCP and act               (the real server/client interaction)
→ report to the user (§4.5); next iteration starts here
```

Each arrow that can fail has a named recovery in `ONBOARDING.md`: a refusal is looked up in ch. 13; a `rejected` run
line means an unreachable or unavailable action and is fixed in the world, not hidden; a missing capability goes to
`brief.md` "Gaps" (the one location; INV-ON-3).

### 4.3 Files the agent leaves in the world directory

- the World Pack (`world.yaml`, `places/`, `people/`, `items/`, `organizations/`, `configure/`, …), as today;
- `brief.md`: the user's brief verbatim, then "What the agent decided" (every default, every slice), then "Gaps"
  (every product gap with its classification), then an iteration log (date, request, what changed, the check results);
- the world-local Presentation Pack when ON-c exists.

`brief.md` is a plain file the loader does not read (worlds already carry `README.md`); ON-a's audit confirms the
loader ignores unknown non-YAML files and records it.

### 4.4 Iteration

An iteration is "brief delta → same loop → playable again". The agent preserves saves unless the change makes resume
refuse (`ConfigurationDrift`), in which case it says so and offers a fresh start; it never deletes a save silently.

### 4.5 The report to the user

Five lines plus links: what was built (places, people, packs), how to play it (the exact command), what the headless
check showed, what the agent decided on the user's behalf, what the framework cannot do yet (gaps).

---

## 5. The three worked example briefs (operator-named)

The full briefs, as a user would write them, are in PR ON-a's design §4 and land as `docs/onboarding/briefs/*.md`.
Each brief file ends with a dated "Expected outcome on this build" note. **Owner of those notes:** the post-merge
update of any step that closes a gap a note names (vehicles, S21 transport, nightlife N1, ON-c, ON-f, the scale
roadmap) updates the note in the same documentation pass; ON-b gives a scored agent the brief text without its note, so
the agent finds gaps from `packs list` and the reference, not from the note.
This section records what each demands of the framework. "Today" means `origin/main @ c373b802`.

### 5.1 Seaside resort (海边度假村) — skeleton-shaped brief

*A small Mediterranean-style resort on a bay: hotel with lobby, rooms, restaurant and terrace bar, a beach with
loungers, a pier with a boat-hire hut, a few village shops. Player arrives as a guest for a week; staff and guests
keep their own routines; Saturday beach party; one bartender may be on a local model.*

- **Agent builds:** places (lobby, rooms, restaurant, terrace bar, beach, pier, boat hut, shop street), 20–30 people
  (staff with jobs, guests with holiday routines), goods from `modern-goods`, a calendar epoch in summer, real
  weather location, happening every Saturday, optional LM bartender with fallback.
- **Packs (bundled):** presence, movement, conversation, relationships, group-activity, schedule, naming, persona,
  item, inventory, item-transfer, economy, employment, consumption, needs, sleep, posture, bodies, calendar, weather,
  happenings, notices; `modern-goods`.
- **Missing today:** room booking/check-in (no `lodging` pack; stand-in: a bought room-key item, labelled as a
  stand-in); swimming/water activities (no action; stand-in: a group activity on the beach); shop/bar opening hours
  depend on nightlife N1 (frozen, in flight); a coast archetype far field and 3D depiction for a new world (ON-c).
- **Gate:** yes (QON-5).

### 5.2 New-York-like metropolis (纽约一样的大都市) — free-prose brief

*Dense Manhattan-like grid: avenues and streets, walk-ups, a deli, a diner, a subway station, an office tower,
a park; crowds; the player is a newcomer who just rented a room.*

- **Agent builds (first iteration):** a declared **neighbourhood slice** — 2×3 connected blocks, ≤ 40 people at the
  §3.4 caps (or, with the operator's spending/hardware approval, the scale gates' 100–300), a subway entrance as a
  place whose platform is a stub; later iterations add blocks.
- **Packs (bundled):** the town set as in §5.1, plus `populations-us` / `person-attributes` for a varied population.
- **Missing today:** (1) a connected street-grid generator — `tools/worldgen/tile.py` makes disconnected copies only
  (ON-f, reuse and extend it); (2) transit — subway/bus are S21's `transport` RT-c, not built; (3) multi-region world
  and client streaming (S21 §5.12, `ARC-RT-i`); (4) capacity: hundreds-to-thousands of residents are unmeasured
  (`operator-playable-mvp` §7.2: 20/100/300/1000 headless first, then with clients); dense crowds stress client LOD,
  instancing and interest management (§7.1, "after measured bottlenecks"); (5) a city archetype far field / depiction
  (ON-c). The agent must say "slice of N blocks, M people; full city needs …" rather than claim a metropolis.
- **Gate:** yes, as the declared slice; the full-scale city is a roadmap item, not this step's acceptance.

### 5.3 Italian mountain village — visiting means driving a mountain road (意大利山村，开车走山路)

*A hilltop borgo with a piazza, church, bar and bakery; farmhouses scattered down the valley, 10–25 minutes apart by
car on a single switchback road; the player has an old Fiat; people visit each other by car; weekly market in the
piazza.*

- **Agent builds today:** the borgo and farmhouses as places, people with routines, the market as a happening, a
  mountain far field. Farmhouses can only be linked by walking passages today.
- **Reusable:** places/passages and `walk-to` (semantic movement), calendar time, happenings, economy, schedule,
  weather, items (the car can exist as an Item), the S21 design (`transport` pack, road links with `length_m` and
  `self_drive`, the `travel-to` itinerary scale of one spatial model, `ARC-RT-c/d/e`).
- **New System Packs this demands (not onboarding's to build):**
  1. **`vehicles`** — required by the operator (2026-11-27), its own step (§10): cars as Items held through
     `inventory`; `vehicles` owns occupancy, pose/route position, engine, optional fuel; `drive-to { destination }`
     along a road graph is a `Process` owned by `vehicles`; embodied driving (throttle/steer) is designed by that step
     (§10.2 survey, open questions §10.3); arrival is presence's `arrived` (presence stays the sole
     owner of where people are); fuel or tolls as `*-due` facts that economy answers (single ownership).
  2. **Road geometry and terrain/elevation** for the server's drivable surface and for presentation (the road is
     drawn and, in 3D, driven) — part of the `vehicles` step's design (§10.3 time domains and collision).
  3. **Regions** (S21 RT-a/b) if the valley is too large for one place graph.
- **Honest outcome today:** the agent must classify "drive to a neighbour" as **real product gap** in `brief.md`
  "Gaps", may offer the walking stand-in (clearly labelled: "a 40-minute walk, not a drive"), and must not fake a car
  with teleporting passages or client-side motion.
- **Gate:** no — reported until the `vehicles` step delivers `drive-to` and embodied driving on a road (QON-5, QON-9);
  then gating. Until then it is the example that proves the path reports gaps instead of inventing them.

---

## 6. What the framework must add (ranked)

| Rank | PR | What | Why this rank | Reuse first |
| --- | --- | --- | --- | --- |
| 1 | **ON-a** | `docs/onboarding/`: `ONBOARDING.md` (agent-audience), `BRIEF.md` (checklist, defaults, ask rule, gap rule), three example briefs with expected-outcome notes, human `README.md`; link from `docs/guide/README.md` "For agents" | Without it no outside agent has a single entry point; documents only, can merge now | Links to the tested guide and generated reference; repeats no command |
| 2 | **ON-c** | **Play an authored world in 2D and 3D**: archetype far-field seeds; a world-local Presentation Pack scaffold (`mineworld create --presentation <archetype>` or a documented copy step: greybox-seeded 3D pack, a 2D pack); the self-hosting convenience (`mineworld-2d`/`mineworld-slice`) for a world outside `worlds/`; a per-world Presentation Pack in the bundle path (`scripts/package.py`, `launch.toml`); guide ch. 06 correction; evidence on macOS, Linux and Windows | A world that cannot be opened in the clients is not "playable"; `ARC-102` forbids the shortcut of a fallback. The join path already exists (§2), so this is narrower than it looks | Existing far-field schema and `packs validate`, `presentation/mineworld-greybox/3D`, plain drawing (`SlicePlainDepiction`), existing far-field files as archetype seeds |
| 3 | **ON-b** | Harness: brief rounds (`BR-1` resort, `BR-2` metropolis slice, `BR-3` mountain village) in `tools/onboarding`; rubric fields from M1–M8; new checks: restart (save, kill, resume, `replay`), server + agent play over MCP (a scripted model-free seat acts in the agent's world), elapsed wall time, manual-intervention count, gap classification; allowed set adds `docs/onboarding/` | It is how §8's acceptance is measured; reuses a proven harness | `run.py`, `score.py`, `audit.py`, guard, ledger format |
| 4 | **ON-d** | `mineworld packs describe [--json]`: per System Pack the actions it offers, sections it owns, configure keys, packs it needs — generated from code like `mineworld reference` | Turns "which packs?" from reading eight reference files into one lookup; reduces guesses | The reference generator (`ARC-90`) and `SystemPack` declarations |
| 5 | **ON-e** | Playability diagnostics in `validate` (warnings, never new refusals without review): a place no passage reaches, a seat with nothing it can do, a person with no routine in a world with `schedule`, a happening with no place | `F-25f-1` showed a world that validates and is unplayable; agents trust `validate` | S29-a's town reachability check, generalized |
| 6 | **ON-f** | Connected street-grid generator for big settings (blocks joined by passages, keys and seats generated) | Only the metropolis needs it; no capacity claim follows from it | `tools/worldgen/tile.py`, the generated schemas |

Explicitly **not** added: a brief parser or schema; an authoring MCP server; any code path that lets a brief change
kernel, System Pack or client code; an LM inside `mineworld` itself (the user's agent is the LM).

Dependencies outside this effort, named so nobody waits silently: the required `vehicles` step (§10; mountain driving, city traffic), S21 `transport` / regions / atlas (metropolis
transit, mountain driving), nightlife N1 shop hours (resort bar), scale measurements of `operator-playable-mvp` §7.2
(metropolis beyond the slice).

---

## 7. Invariants

- **INV-ON-1 Framework untouched by a brief.** A brief round's diff is confined to the world directory (World Pack,
  world-local Presentation Pack, Entity Pack data, `brief.md`). Any other change voids the round.
- **INV-ON-2 No LM required.** Every generated world passes `mineworld run --headless` with no model configured.
- **INV-ON-3 Gaps are reported, never faked.** A behaviour no pack provides appears in `brief.md` "Gaps" with a class;
  a stand-in is labelled as one. A world that simulates a missing capability by teleporting passages, client-side
  rules or misnamed actions fails the round.
- **INV-ON-4 The brief stays a scaffold.** No component of MineWorld refuses a brief for its shape; only World Pack
  files are validated.
- **INV-ON-5 No paid model in CI, fresh agent, sandbox** — INV-S25-6 and INV-S25-7, unchanged.
- **INV-ON-6 Playable means run.** A diagnostic drawing (`--presentation=none`) never counts as playable. "Playable in 2D/3D" is claimed only for a client actually opened on that world in
  that round, with evidence.

---

## 8. Acceptance (the 11-22 requirement, made measurable)

A brief round passes when, from a fresh sandboxed agent given only the brief and `docs/onboarding/` + guide +
reference + binary (+ clients):

| # | Evidence | Measured by |
| --- | --- | --- |
| A1 | Generated artifacts: the world directory, `brief.md` with decisions and gaps | copied into the evidence directory |
| A2 | `mineworld validate` passes | scorer S1 |
| A3 | Headless 14-day run: exit 0, `faults 0`, no `rejected` line, every place entered | S2, S8, new place-entry check |
| A4 | The brief's must-state items are honoured (places, people count, player seat, packs for the stated daily life, LM seats if asked) | brief rubric (ON-b), from the world's files and `validate` output, never the agent's claims |
| A5 | Real server/client interaction: hosted with `mineworld server --save`; a scripted MCP seat joins and has ≥ 1 accepted action; for played worlds the 2D client's scripted drive enters the world; 3D likewise once ON-c lands | harness logs |
| A6 | Persistence and restart: stop the server, start it on the same save, the same people are where they were; `mineworld replay` reproduces | `inspect`, `replay` exit 0 |
| A7 | Elapsed time from prompt to first playable world | `summary.json` wall time |
| A8 | Manual interventions: count and description (target 0) | rounds ledger column |
| A9 | Gaps correctly classified (mountain village: driving = real product gap) | audit + reviewer |

**Step acceptance:** BR-1 (resort, played) and BR-2 (metropolis slice, played **and** a headless 365-day research run with `--save`, `replay` clean) pass A1–A9 on one commit, plus a human hands-on play of
each in 2D and 3D by the operator; BR-3 (mountain village) runs and passes A1–A3, A5–A9 with driving recorded as a
product gap. Two materially different briefs therefore gate, as the 11-22 requirement asks.

### 8.1 Running real agents: subscriptions, limits and terms

Operator correction, 2026-11-27, verbatim:

> 用真实 agent 跑示例测试 不会花钱啊，用订阅就行

- **Cost.** Brief rounds run under the operator's own logged-in subscription CLIs (Claude Code, Codex CLI) on the
  operator's machine. No metered API key is used, so incremental spend is $0. `tools/onboarding` already runs Claude
  Code "under the operator's own login" and passes no API key or token to the agent (PROTOCOL §3.2); a Codex runner
  must keep the same rule. `--max-budget-usd` stays as a ceiling on what the CLI reports and does not mean money is spent.
- **Limits.** Runs stay within the subscription's normal interactive-use limits: one round at a time (R-LOAD-1), each
  started by the operator and watched by the operator. At most 3 brief rounds per agent CLI per milestone (BR-1, BR-2, BR-3,
  plus re-runs only after a recorded fix). No loops, no scheduled or unattended batches, no parallel accounts. Usage
  caps are never worked around: a round that hits the plan's rate limit stops and is recorded `INCONCLUSIVE`, not retried in a loop.
- **Terms checked 2026-11-27.** These need the operator's confirmation before the first round:
  - Anthropic Consumer Terms (effective 2025-10-08), §3 "Use of our Services", item 7, prohibits accessing the
    Services "through automated or non-human means, whether through a bot, script, or otherwise" *except* via an
    Anthropic API key "or where we otherwise explicitly permit it". Claude Code's documented non-interactive mode
    (`claude -p`) is Anthropic's own client. The design relies on it only for operator-started, operator-attended,
    bounded rounds. The operator confirms, against the current Claude Code documentation, that this use is permitted
    under the plan. If it is not, the rounds fall back to the operator typing the brief into an interactive session.
    That keeps the same sandbox and evidence, and the transcript is taken from the session log.
  - OpenAI Terms of Use (effective 2026-01-01), "Using our Services" → "What you cannot do", forbids "Automatically
    or programmatically extract data or Output" and circumventing "any rate limits or restrictions". Codex CLI signed
    in with the operator's ChatGPT plan is OpenAI's own client used as intended. The harness reads only the files the
    agent wrote in the sandbox, plus the CLI's own transcript, for the operator's evaluation. It makes no bulk
    extraction and does not get around any limit.
  - The earlier ruling that bars routing a *third-party app* through Claude Pro/Max credentials (`lm-backends`,
    S10 P5) is unchanged. This section covers running the vendor's own CLI as the vendor ships it. It does not cover
    MineWorld calling a model through a subscription.

---

## 9. PR split and ordering

| PR | Scope | Depends on | When |
| --- | --- | --- | --- |
| ON-a | documents (§6 rank 1) | — | now (docs-only; no MVP lane touched) |
| ON-b | harness brief rounds and checks | ON-a | after first playable MVP |
| ON-c | world-local Presentation Pack scaffold (archetype far fields, greybox-seeded 3D pack), self-hosting a world outside `worlds/`, per-world pack in the bundle, guide ch. 06 correction | presentation owners' review; QON-4 | after first playable MVP |
| ON-d | `packs describe` | — | after first playable MVP; parallel with ON-c |
| ON-e | playability warnings | S29-a reachability | parallel with ON-d |
| ON-f | grid generator | ON-b (to measure) | before BR-2 gating |
| ON-g | acceptance rounds BR-1/2/3 (operator-run, ledger) | ON-b, ON-c, ON-f | last |

---

## 10. Vehicles: a required capability (direction for a separate step)

### 10.1 Operator's words (2026-11-27, verbatim)

> 车我们肯定要enable的，而且开车这个应该有很多现成的方案能学了，好多游戏都有这个功能，比如绝地求生，gta

Meaning: driving is required, not optional; learn from mature, widely used approaches (open-world games drive cars
routinely). We learn **mechanics** only and copy no game's content, code or assets. This section is a direction for
a separate `vehicles` step (its own step and PR designs, frozen on their own); it authorizes nothing and is not part
of onboarding's PRs. It replaces S21 §5.4.1's "design only; no PR" status for self-driven cars with "required, own
step", subject to the operator's freeze of that step.

### 10.2 Reuse survey (mechanics; non-binding; licences to re-verify in the step's prototype)

**Status of §10.2–10.4: a non-binding survey and direction.** QON-9 approves the placement only. Every
recommendation below is an input to the `vehicles` step, which decides it in its own `DEP-VH-*` / `ARC-VH-*`
records after the questions of §10.3 are answered.

The common mechanical pattern in open-world games is the **ray-cast vehicle**. It has a rigid chassis, one ray (or
shape cast) per wheel for suspension and ground contact, tyre friction from slip, engine and brake torque, and a
steering angle per wheel. Traffic far from any player usually uses a cheaper **path-following** model on a lane or
road graph, and switches to physics only near players. Candidates:

| # | Option | Where it runs | Licence | Fit and cost |
| --- | --- | --- | --- | --- |
| V1 | **Rapier `DynamicRayCastVehicleController`** (`rapier3d::control`, present in our pinned `rapier3d 0.36.0`, `src/control/ray_cast_vehicle_controller.rs`) | server (Rust) | Apache-2.0 | Same engine, version and `enhanced-determinism` feature as `bodies` (`DEP-13`). Determinism is plausible but must be proven. **Conflicts with `DEP-13`'s isolation as accepted**: see Q-VH-1. Arcade-to-mid fidelity. |
| V2 | **Godot 4 `VehicleBody3D` / `VehicleWheel3D`** (the built-in ray-cast vehicle), under the Jolt engine the 3D client uses (`clients/3d-spike/project.godot`: `3d/physics_engine="Jolt Physics"`) | 3D client | MIT | Natural for wheel and suspension visuals. **A different model from V1** (solver, step, tuning semantics), so prediction from it diverges from the server: Q-VH-3. Behaviour under Godot's built-in Jolt is unverified. |
| V3 | **Jolt's native `VehicleConstraint` / `WheeledVehicleController`** (C++) | needs a native binding (server) or a GDExtension (client) | MIT | Highest fidelity (engine, gearbox, differentials, anti-roll). A second physics engine on the server, or a binding to maintain. Upgrade path only. |
| V4 | Open-source Godot vehicle controller add-ons (asset-library ray-cast controllers) | 3D client | each to verify (any GPL or unknown licence is rejected by the `deny.toml` policy) | A possible source of tuned client feel. Surveyed, never assumed. |
| V5 | Our own kinematic bicycle model on the road graph | server | ours | The likely model for NPC and off-screen cars (route scale). Cheap and deterministic on integers. |
| V6 | **V1 on the client too**: the same Rust controller, shared through a GDExtension | 3D client (and 2D, if 2D predicts) | Apache-2.0 | The determinism-matching prediction option: the same model on both sides allows input rewind and replay. Costs a GDExtension build on three platforms. |
| V7 | **Client-simulated, server-validated**: the client simulates the car (V2/V4/V6) and reports poses as `walk-step`-like requests; the server validates each against speed, road and collision limits (S21 §5.4.1's implied direction) | client simulates, server validates | — | Fits `ARC-67` (per-request resolution, no embodied time on the server) and keeps the journal small. It trusts the client more, which is bounded by validation, and cars do not push other bodies authoritatively between requests. |

### 10.3 Architecture: what any vehicle design must reconcile (open questions for the vehicles step)

Settled direction (consistent with the accepted decisions):

- **Kernel ignorance.** The kernel never learns what a car is.
- **Property and money.** A car is an Item kind held through `inventory`. Only economy moves money: fuel, tolls and
  purchases are `*-due` facts that economy answers.
- **Where people are.** Presence stays the only owner of a person's location. A car's cabin is a Place (S21 §5.4's
  carriage model), so entering a car is presence's `arrived` into the cabin and talking inside it is ordinary
  conversation. Driver and passenger seats are sit points (`posture`).
- **Offers.** A new **`vehicles` System Pack** offers `enter-vehicle`, `exit-vehicle`, `start-engine`, `drive`,
  `drive-to` and `park`. Refusals use the standard codes (`Unavailable`, `Busy`, `PermissionDenied`,
  `NoSupportedInteraction`). No client evaluates who may drive (rule 15).
- **Relation to transport.** `transport` (S21 RT-c) builds on `vehicles`: a bus or train is a vehicle with a line and
  a timetable. That amends S21 §5.4, and `ARC-VH-a` records the amendment.
- **NPC drivers.** NPCs use the route scale. Controllers propose `drive-to`, and rule controllers need no physics.
- **Seats.** Multiplayer uses one driver seat and any number of passenger seats. The existing seat rules apply.

Open questions. Each is answered with evidence and an operator ruling before code:

- **Q-VH-1 Physics state vs `DEP-13`.** `DEP-13` accepts that `rapier3d` is named only in `systems/bodies`
  (`src/rapier.rs`, structurally tested). Each resolution builds a fresh Rapier world and drops it, and there is "no
  momentum between resolutions". A ray-cast vehicle (V1) is a persistent, momentum-carrying body stepped every tick.
  V1 therefore requires **amending `DEP-13`**. The amendment is either persistent per-place physics state in `bodies`,
  or a second crate naming Rapier behind its own isolating module. V7 needs no amendment.
- **Q-VH-2 One owner of the embodied car pose.** Either `bodies` hosts vehicle dynamics on `vehicles`' behalf
  (requests in, facts out), or `vehicles` owns the car's physics and `bodies` consumes it for collisions with people
  and objects. There may not be two writers (`CLAUDE.md` §4 rule 1).
- **Q-VH-3 Time domain (`ARC-67`, S21 §5.4.1).** `ARC-67` §2: embodied time is not a world quantity; a per-request
  rule is resolved when its request arrives; no fact, component, Process or rule carries wall time. A 30–60 Hz server
  step for held vehicles contradicts that unless `ARC-67` is amended. V7 does not. Time scale must also be reconciled.
  At the default 12×, a 20-minute real-speed embodied drive lasts about 4 world hours, while a `drive-to` Process on the
  same road takes 20 world minutes. Two scales of one spatial model must not disagree by 12× on the same road. The
  candidates are: driving speed scaled with time scale; embodied driving only at 1×; or route time defined from wall
  time while a car is held. Whether any sub-stepping is pack-owned or a kernel scheduling capability belongs here too
  (`ARC-VH-b`).
- **Q-VH-4 Reconciliation (`ARC-50` does not transfer).** Walking's rule zeroes velocity and teleports beyond 150 mm,
  then suspends `move` reports. `ARC-50` rejected input replay because server strides are not replayable on the
  client. At 15–25 m/s, one round trip is metres of travel, and zeroing a car's velocity on every correction is
  unplayable. The step chooses one of these, and discloses velocity and angular velocity if it predicts:
  - the same model on both sides with input-sequence rewind and replay (V6);
  - interpolation and extrapolation with server velocity disclosed;
  - client-simulated and server-validated (V7).

  The P-VH-1 pass threshold (maximum divergence at a stated speed and round-trip time) is fixed **before** measuring.
- **Q-VH-5 Journal volume and determinism (`ARC-25`).** A world is its journal re-executed. Recorded 30–60 Hz inputs
  are journal entries: one hour of driving is 100–200k records per vehicle. The step sizes the volume, quantizes
  inputs to integers (`DEP-13`'s integers in, integer millimetres out, kept for the car pose), sets the snapshot
  cadence, and states the semantic facts (`vehicle-entered`, `engine-started`, `drive-began`,
  `drive-ended { from, to, distance_m }`, `vehicle-parked`, `vehicle-struck`). V7 records poses per request instead.
- **Q-VH-6 Collision and harm.** Who resolves car–person and car–object contact (tied to Q-VH-2), and which facts are
  stated. Whether harm is allowed is Interaction List configuration.
- **Q-VH-7 2D parity.** 2D needs the following:
  - a keyboard mapping for throttle, brake and steer;
  - a decision on whether 2D predicts (it has no physics engine) or only interpolates;
  - a way to disclose elevation and hairpins of a mountain road to a top-down client;
  - 2D driving included in BR-3's acceptance.

  Neither client may duplicate driving logic (the two gating questions of `CLAUDE.md` §4).
- **Q-VH-8 Platforms.** Acceptance on macOS, Linux and Windows, including any GDExtension (V6).

### 10.4 Placement

A separate step after the playable MVP (QON-9), ordered with S21. `vehicles` comes first: cars, roads, route driving
and embodied driving, in one world and one region. S21's `transport` lines and regions follow on top. The step's
first PR answers Q-VH-1 … Q-VH-8 with a prototype on a flat test place (P-VH-1). The prototype compares V1+V6 against
V7 at least, against a pre-stated divergence threshold, with determinism and journal-volume measurements. That PR
also drafts the `DEP-VH-a`, `ARC-VH-a` and `ARC-VH-b` records (ids numbered by the primary) and any `DEP-13` /
`ARC-67` / `ARC-50` amendments. Onboarding's BR-3 (mountain village) gates once that step delivers `drive-to` and
embodied driving on a road, in both clients.

---

## 11. Risks

- **Presentation is the long pole.** Archetype far fields and depictions for arbitrary layouts may be visually poor;
  the first bar is "honest and readable", reviewed on the progress page, not slice-grade fidelity.
- **Guessing agents.** Mitigated by the ask rule, the capability index and the audit's guess phrases.
- **Brief inflation into a schema.** INV-ON-4; `BRIEF.md` reviewers reject required headings or field names.
- **Cost of big worlds.** Caps (§3.4) and per-round budget; any raise is the operator's.

---

## 12. Red-team pre-freeze review disposition (2026-11-27)

| Finding | Disposition |
| --- | --- |
| F1 3D audit wrong | Fixed: §2 rows "Play in 3D" and "Bundled launcher", QON-4, ON-c rescoped; guide ch. 06 staleness recorded (§2) and routed to ON-c. |
| F2–F4 vehicles conflicts (`DEP-13`, `ARC-67`, `ARC-50`) | Fixed: §10 demoted to a non-binding survey; conflicts listed as Q-VH-1 … Q-VH-4; V6 (same model on client) and V7 (client-simulated, server-validated) added. |
| F5 journal volume (`ARC-25`) | Fixed: Q-VH-5. |
| F6 2D parity | Fixed: Q-VH-7. |
| F7 diagnostic drawing as play | Fixed: §2, §4.2, INV-ON-6. |
| F8 two gap locations | Fixed: `brief.md` "Gaps" only. |
| F9 spending | Fixed in QON-7: round definition, QS25-3 ceiling and amendment path, ON-g total, asset spend USD 0, credential inheritance. |
| F10 materially different gating briefs | Fixed in QON-5 / §8: BR-2 gates as played **and** a 365-day headless research run (different use and scale); recorded as the reason. |
| F11 platforms | Fixed: §1.3, §3.4, ON-c, Q-VH-8. |
| F12 checklist wording | Fixed: M6 "Activities", M7 mapped to Controller, skeleton headings defined once. |
| F13 stale notes | Fixed: owner rule in §5; scored agents get the brief without its note. |
| F14 launcher audit | Fixed: §2 row. |
| F15 housekeeping | Fixed: QON-8/9 order; ON-a link-check command, acceptance measure and execution contract. |


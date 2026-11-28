# Onboarding step 01 — From a loose brief to a playable world

**Lifecycle:** `DRAFT — FREEZE-READY` (2026-11-27). Step-level design. Nothing here is frozen and nothing here
authorizes implementation; each PR of §9 is frozen on its own. The first PR's design draft is
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

## 0. Freeze questions (plain language, each with a recommendation)

| # | Question | Recommendation |
| --- | --- | --- |
| QON-1 | Is the brief a **scaffold** (a short must-state checklist + worked examples, everything else free) rather than a schema? | **Yes.** No brief parser, no brief schema, no required headings. The user's agent is the parser. The framework's contracts (World Pack files, `mineworld validate`) stay the only machine-checked input (§3). |
| QON-2 | When a brief leaves something out, does the agent ask or default? | **Default almost everything and write the default down; ask at most one round of questions, and only when the answer changes what kind of world it is** (play vs headless research, or two readings of the premise that lead to different worlds). Everything else gets the documented default (§3.3) and is listed in the world's `brief.md` "What the agent decided" section, so the user can change it in the next iteration. |
| QON-3 | Where do the onboarding documents live? | **`docs/onboarding/`**: a short `README.md` for humans, `ONBOARDING.md` (the one document an outside agent reads), `BRIEF.md` (the checklist and defaults), `briefs/` (the three operator-named examples). Not a guide chapter: `docs/guide/` pages are executed by the guide test line by line (`GUIDE_FORMAT.md`), and a brief's outcome is not deterministic text. `ONBOARDING.md` links into the tested guide for every command instead of repeating it. |
| QON-4 | How does a world an agent just wrote get **played in 2D and 3D**, when today the clients refuse a world with no `far_field/<world-key>.yaml` (no fallback, `ARC-102`) and the 3D launcher accepts only `worlds/<key>` inside the checkout? | **Keep `ARC-102`. Add a world-local Presentation Pack that the agent writes beside the world** (far field + 3D depiction), seeded from a small, reviewed set of setting archetypes (coast, city, mountain, plain …), validated by the existing `mineworld packs validate`; and let the launch path take a world directory. Do not add a generic fallback far field (§6, PR ON-c). This is the largest framework gap and needs the presentation owners' review. |
| QON-5 | Which example briefs gate acceptance? | **The seaside resort gates; the metropolis gates as a declared neighbourhood slice** (a few connected blocks, a few hundred residents at most, transit stubbed and listed as a gap); **the Italian mountain village is run and reported but does not gate** until the required `vehicles` step (§10, QON-9) delivers driving; S21 §5.4.1 is design-only today. Its round must end with the driving requirement classified **real product gap**, not faked (§5.3). |
| QON-6 | Release placement. | **After the first playable MVP.** PR ON-a (documents only) can merge now; it touches no MVP lane. ON-b … ON-f wait until the playable MVP is accepted, then queue in §9's order. The 11-22 acceptance (§8) is not claimed by ON-a. |
| QON-7 | Agent harness and spending. | **Reuse `tools/onboarding` unchanged in kind** (allowlisted Claude Code, operator-run, never in CI, USD 5 and 80 turns per round). Add brief instruction sets; ask the operator before any round over USD 5 (the metropolis may need it). A Codex round is **reported, not gating**, and only once an equally strict allowlist is shown for it (no `bypass` mode). |
| QON-9 | Vehicles are required (operator, 2026-11-27). How do they enter the plan? | **As their own step and PR series (`vehicles`), after the playable MVP, not inside onboarding.** Direction in §10: a `vehicles` System Pack owns vehicle state; the server simulates driving with Rapier's ray-cast vehicle controller (already our physics engine, `DEP-13`) for player-driven cars and a road-graph `Process` for NPC and distant cars; clients send throttle/steer as intents and render wheels with Godot/Jolt. `transport` (S21) builds on `vehicles` rather than owning the fleet. One `ARC-VH-*` and one `DEP-VH-*` record before code. Onboarding's BR-3 becomes gating once `vehicles` lands. |
| QON-8 | Does the agent author through MCP? | **No.** Authoring is files + the `mineworld` CLI, which every agent (Claude Code, Codex, any shell-capable agent) already has. The MCP adapter (`clients/agent`) is used for what it is for: the agent (or a second agent) *plays* the world it built, which is the "real server/client interaction" evidence (§8). |

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
| Play in 2D | `./mineworld-2d --world PATH` hosts any World Pack path; needs Godot 4.7 and a checkout | `mineworld-2d` | Partly. Needs `far_field/<world-key>.yaml` in the Presentation Pack, no fallback; `--presentation=none` is a diagnostic only (MODULE_SPEC §6.1.1). |
| Play in 3D | `./mineworld-slice --world=<key>` accepts only `worlds/<key>/world.yaml` inside the checkout; drawn worlds need a depiction file in the pack (`depictions/<key>.yaml`) and a far field. Guide ch. 06: "The 3D client today draws the café of the sample town only." | `mineworld-slice` line ~311, `presentation/mineworld-default/3D/` | **Gap** (§6 ON-c). The exact current behaviour of a fresh world in each client is ON-c's first audit item. |
| Bundled launcher | `tools/launch`: fixed bundle worlds from `launch.toml`; any unlisted argument is an error | `tools/launch/LAUNCHER.md` §3 | Gap: no "this world directory" entry (§6 ON-c). |
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
order. `BRIEF.md` offers an optional skeleton (the eight headings of §3.2) for users who like one; the worked examples
show both a skeleton-shaped brief and a free-prose one, deliberately.

### 3.2 The must-state checklist (what the agent must *know* before building, not what the user must *write*)

| # | Item | Stated by the user, or else the agent… |
| --- | --- | --- |
| M1 | **Premise** — what kind of place, and what life there is like, in a sentence or two | **asks** if it is absent: it is the world's identity |
| M2 | **Use** — played (2D, 3D, both), headless research, or both | defaults to **played in 2D and 3D, also runnable headless**; asks only if the brief reads as research-only and also mentions a player |
| M3 | **Player** — who the player is (an arriving visitor, a resident, staff, nobody) | defaults to **one seat, a newcomer** (`visitor`) for played worlds, no player seat for research-only |
| M4 | **Places** — the places that matter and roughly how they connect | defaults to the premise's obvious places (3–8) on one street/square, every place reachable |
| M5 | **People** — how many, and any who matter by name or role | defaults to **6–12 residents**, named for the setting, each with a routine; scale caps in §3.4 |
| M6 | **Daily life** — what people do: work, trade, eat, sleep, meet, events | defaults to the bundled town set (routines, conversation, relationships, needs, sleep, money, work, shops, a weekly happening); every choice maps to an installable System Pack (§4.2) |
| M7 | **Minds** — rules only, or some people on a language model | defaults to **rules only** (runs with no LM, costs nothing); LM seats only when asked, always with a rule fallback, local model first |
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

### 3.4 Default caps (a first iteration must stay playable on one Mac)

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
  depiction) from an archetype                                    (ON-c; until then 2D --presentation=none, stated)
→ host: mineworld server … --save … ; open 2D and/or 3D;
  optionally join one seat itself over MCP and act               (the real server/client interaction)
→ report to the user (§4.5); next iteration starts here
```

Each arrow that can fail has a named recovery in `ONBOARDING.md`: a refusal is looked up in ch. 13; a `rejected` run
line means an unreachable or unavailable action and is fixed in the world, not hidden; a missing capability goes to
`GAPS.md`.

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
     along a road graph is a `Process` owned by `vehicles`; embodied driving (throttle/steer intents) is simulated on
     the server with Rapier's ray-cast vehicle controller; arrival is presence's `arrived` (presence stays the sole
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
| 2 | **ON-c** | **Play an authored world in 2D and 3D**: a world-local Presentation Pack scaffold (far field + 3D depiction from a reviewed archetype set), `mineworld create --presentation <archetype>` or a documented copy step; `mineworld-slice --world` and the bundle launcher accept a world directory | A world that cannot be opened in the clients is not "playable"; `ARC-102` forbids the shortcut of a fallback | Existing far-field schema and `packs validate`, `depictions/*.yaml` (S33-e), existing far-field files as archetype seeds |
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
- **INV-ON-6 Playable means run.** "Playable in 2D/3D" is claimed only for a client actually opened on that world in
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

**Step acceptance:** BR-1 (resort) and BR-2 (metropolis slice) pass A1–A9 on one commit, plus a human hands-on play of
each in 2D and 3D by the operator; BR-3 (mountain village) runs and passes A1–A3, A5–A9 with driving recorded as a
product gap. Two materially different briefs therefore gate, as the 11-22 requirement asks.

---

## 9. PR split and ordering

| PR | Scope | Depends on | When |
| --- | --- | --- | --- |
| ON-a | documents (§6 rank 1) | — | now (docs-only; no MVP lane touched) |
| ON-b | harness brief rounds and checks | ON-a | after first playable MVP |
| ON-c | authored-world presentation + launch by directory | presentation owners' review; QON-4 | after first playable MVP |
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

### 10.2 Reuse survey (mechanics; licences to re-verify in the step's prototype)

The common mechanical pattern in open-world games is the **ray-cast vehicle**: a rigid chassis, one ray (or shape
cast) per wheel for suspension and ground contact, tyre friction from slip, engine/brake torque and a steering angle
per wheel — plus, for traffic far from any player, a cheap **path-following** model on a lane/road graph that switches
to physics only near players. Candidates:

| # | Option | Where it runs | Licence | Fit |
| --- | --- | --- | --- | --- |
| V1 | **Rapier `DynamicRayCastVehicleController`** (`rapier3d::control`, present in our pinned `rapier3d 0.36.0`, `src/control/ray_cast_vehicle_controller.rs`) | server (Rust) | Apache-2.0 | **Best server fit**: same engine, version and `enhanced-determinism` feature the `bodies` pack already uses (`DEP-13`), so cars and people collide in one authoritative world. Arcade-to-mid fidelity; tuning per car kind. |
| V2 | **Godot 4 `VehicleBody3D` / `VehicleWheel3D`** (built-in ray-cast vehicle), under the Jolt engine the 3D client already uses (`clients/3d-spike/project.godot`: `3d/physics_engine="Jolt Physics"`) | 3D client | MIT | **Best client fit** for prediction and wheel/suspension visuals; whether `VehicleBody3D` behaves identically under Godot's built-in Jolt (Godot ≥ 4.4) must be measured in the prototype (P-VH-1). Not authoritative. |
| V3 | **Jolt's native `VehicleConstraint` / `WheeledVehicleController`** (C++, Jolt Physics) | would need a native binding (server) or a GDExtension (client) | MIT | Highest fidelity (engine, gearbox, differentials, anti-roll), but a second physics engine on the server breaks one-world collision with `bodies`; a binding is new maintenance. Keep as the upgrade path if V1's fidelity proves insufficient. |
| V4 | Open-source Godot vehicle controller add-ons (asset-library ray-cast controllers) | 3D client | each to verify (MIT/CC0 expected; any GPL/unknown is rejected by `deny.toml` policy) | Possible source of tuned client-side feel; adopted only if better than V2 and licence-clean. Survey in the step's first PR. |
| V5 | Our own kinematic bicycle model on the road graph | server | ours | Not a physics replacement; it **is** the right model for NPC and off-screen cars (semantic scale, §10.3), and costs little. |

Recommendation for the step's `DEP-VH-a`: **V1 on the server for embodied driving, V5 for route-following driving,
V2 on the 3D client for prediction and presentation**; V3 recorded as the measured-upgrade path; V4 surveyed, not
assumed. The record states both halves of the reuse question (not reinventing: V1/V2 adopted; not forcing: no client
physics is authoritative, no second engine on the server).

### 10.3 Architecture reconciliation

- **Ownership (`ARC-VH-a`).** A new **`vehicles` System Pack** owns vehicle runtime state: pose/route position,
  speed, occupants by seat, engine on/off, fuel (optional), parked/moving. **Ownership of the car as property stays
  with `inventory`** (a car is an Item kind, held like any item); economy alone moves money (fuel, tolls, purchase,
  as `*-due` facts it answers). **Presence stays the only owner of where a person is**: a car's cabin is a Place
  (S21 §5.4's carriage model), entering is presence's `arrived` into the cabin, so talking inside a car is ordinary
  conversation. Driver and passenger seats are sit points (`posture`); `vehicles` offers `enter-vehicle`,
  `exit-vehicle`, `start-engine`, `drive`, `drive-to`, `park`. `transport` (S21 RT-c) then **builds on `vehicles`**
  (a bus or train is a vehicle with a line and a timetable) instead of owning the fleet — an amendment to S21 §5.4
  recorded in `ARC-VH-a`.
- **Two scales of one spatial model (`CLAUDE.md` §4 rule 14).** *Semantic:* a road graph (links between stops/places,
  `length_m`, speed limit, `self_drive`) as `vehicles`/`transport` configuration; `drive-to { destination }` is a
  `Process` owned by `vehicles` along the graph, ending in presence's arrival — used by NPCs and by any car no player
  is driving or seeing. *Embodied:* inside a region with drivable geometry, a held car is simulated by V1 on the
  server; the client never decides where the car is. `MoveIntent`-like driving intents, the drive `Process`, the
  authoritative pose and the rendered car stay four distinct things.
- **Intent flow.** The client sends `drive` intents carrying throttle, brake and steer (bounded values) at the
  server's input rate; the server's actor check (same seat rules as walking), the `vehicles` pack's validation
  (driver seat, engine on, not `Busy`) and V1 produce the pose; refusals use the standard codes (`Unavailable`,
  `Busy`, `PermissionDenied`, `NoSupportedInteraction`). The 3D client predicts with V2 and corrects to the server
  as walking does (12e: snap beyond a threshold, short glide). Rule 15 holds: no client evaluates who may drive.
- **Time domains — the open question the step must answer first.** Today the server advances at semantic instants
  (walking strides via `walk-step`, `ARC-39`), not a fixed physics tick. Embodied driving needs a fixed-rate server
  step (e.g. 30–60 Hz) **only for held, moving vehicles and the bodies near them**. Whether that is a `vehicles`-owned
  sub-stepping `Process` or a kernel scheduling capability is an architecture decision for the operator
  (`ARC-VH-b`); the kernel must not learn what a car is either way.
- **Collision with people and objects.** Cars are bodies in `bodies`' Rapier world (kinematic when following a route,
  dynamic when embodied); a car's contact with a person is resolved by the server and stated as facts (`vehicle-struck`
  or a refused/stopped drive), never by a client. Whether a world allows harm is Interaction List configuration.
- **Multiplayer.** One driver seat per vehicle, any number of passenger seats; the driver's intents are the only
  control input; passengers see the server's pose. Takeover, reconnect and hold follow the existing seat rules.
- **Persistence.** Facts are semantic, not per-tick: `vehicle-entered`, `engine-started`, `drive-began`,
  `drive-ended { from, to, distance_m }`, `vehicle-parked { place, pose }`, `vehicle-struck`. The embodied pose is
  snapshot state, restored on resume; a run stops and resumes with every parked car where it was. Replay determinism
  relies on V1 under `enhanced-determinism` plus recorded intents — to be proven in the step, as `bodies` did.
- **NPC drivers.** Controllers propose `drive-to` (route scale); rule controllers need no physics; an LM controller
  proposes the same intent. A player-visible NPC car may be promoted to embodied simulation only if the step shows it
  is needed.
- **2D/3D parity.** Both clients send the same intents and render the same disclosed state: 2D draws a top-down car
  sprite along the disclosed pose; 3D draws wheels, suspension and steering (V2). Neither carries driving rules.

### 10.4 Placement

A separate step after the playable MVP (QON-9), ordered with S21: `vehicles` first (cars, roads, embodied + route
driving, one world, one region), then S21 `transport` lines and regions on top. Its first PR is a prototype
(P-VH-1: V1 server car + V2 client prediction on a flat test place, measured correction error and determinism) and the
`DEP-VH-a` / `ARC-VH-a` / `ARC-VH-b` records (ids numbered by the primary). Onboarding's BR-3 (mountain village) gates
once that step delivers `drive-to` and embodied driving on a road.

---

## 11. Risks

- **Presentation is the long pole.** Archetype far fields and depictions for arbitrary layouts may be visually poor;
  the first bar is "honest and readable", reviewed on the progress page, not slice-grade fidelity.
- **Guessing agents.** Mitigated by the ask rule, the capability index and the audit's guess phrases.
- **Brief inflation into a schema.** INV-ON-4; `BRIEF.md` reviewers reject required headings or field names.
- **Cost of big worlds.** Caps (§3.4) and per-round budget; any raise is the operator's.

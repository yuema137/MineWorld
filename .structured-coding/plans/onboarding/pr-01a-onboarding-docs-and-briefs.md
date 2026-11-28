# PR ON-a — Agent onboarding documents and the three example briefs

**Lifecycle:** `FROZEN` — **DESIGN FROZEN 2026-11-27** by operator ruling, relayed by the coordinator: "其他问题都用你的推荐"
(QA-1 … QA-4 as recommended; QA-4: merge after independent review under the design-doc merge ruling). Red-team
pre-freeze findings applied (step §12). Base: `origin/main @ c373b802`. **Parent:** [`step-01-brief-to-playable-world.md`](step-01-brief-to-playable-world.md)
§§3–6, §9 (ON-a). **Kind:** documents only (`docs/onboarding/`, one link in `docs/guide/README.md`). No code, no CLI,
no harness, no world, no MVP lane touched. **Base:** `origin/main @ c373b802`.

## 0. Freeze questions (plain language) — ruled 2026-11-27 as recommended

| # | Question | Recommendation |
| --- | --- | --- |
| QA-1 | Approve the step's QON-1 … QON-3 (scaffold not schema; default-and-record, ask once; `docs/onboarding/`)? | Yes; this PR implements exactly those. |
| QA-2 | Chinese mirrors now? | **Yes for `README.md` only** (`README.zh-CN.md`), as a mirror; `ONBOARDING.md`, `BRIEF.md` and the briefs are agent-audience and English (`CLAUDE.md` §2.1 rule 5). The briefs state that a brief may be written in any language. |
| QA-3 | Do the example briefs say what is missing today? | **Yes**, each brief file ends with an "Expected outcome on this build" note (what is built, which packs, what is a gap), so a reading agent does not invent the missing parts. The note is dated and points to the step document as authority; the brief text above it stays user-written. |
| QA-4 | Merge authority. | Docs-only design-and-guide PR: merge after independent review under the operator's design-doc merge ruling, if the operator confirms it applies to `docs/onboarding/` (it is a specification, not a plan). Otherwise the operator merges. |

## 1. Audit (what this PR relies on)

- The tested guide (`docs/guide/`, chapters 01–14, `GUIDE_FORMAT.md`) is the authority for every command; this PR
  repeats none, it links.
- The CLI surface is `create`, `validate`, `run`, `server`, `inspect`, `replay`, `biography`, `perceived`,
  `interactions`, `attributes`, `decisions`, `packs list|show|validate|resolve`, `reference` (`tools/cli/src/main.rs`).
  There is no `check` subcommand; documents say `validate`.
- Worlds already carry non-YAML files (`worlds/lakeside/README.md`); C1's review confirms the loader ignores
  `brief.md` by running `mineworld validate` on a copy of `docs/guide/examples/corner-shop` with a `brief.md` added
  (evidence recorded, file not committed).
- Client facts stated in `ONBOARDING.md` (step §2, corrected after red-team F1/F7/F14): a world is drawn only with a
  Presentation Pack carrying `far_field/<world-key>.yaml` (no fallback, MODULE_SPEC §6.1.1, `ARC-102`); the bundled
  packs carry far fields for the bundled worlds only; the 3D client joins any hosted world with
  `--server … --invite … --world-key … --presentation=<dir>` and draws places with no depiction plainly
  (`clients/3d-spike/PRESENTATION.md` §6); `--world=<pack>` self-hosting is limited to `worlds/`; `--presentation=none`
  in 2D is a diagnostic, never play. A world-local pack (greybox-seeded) is described as the intended route and marked
  **not yet exercised end to end** until ON-c; the agent reports "not yet drawn" rather than using diagnostic mode.
  Guide ch. 06's stale 3D sentence is recorded in step §2 and fixed by ON-c with run evidence; ON-a does not repeat it.
- Platforms: `ONBOARDING.md` names the launch path per platform (checkout scripts on macOS/Linux; the bundle launchers
  or the `mineworld` binary plus `clients/2d/README.md` on Windows).

### 1.1 Execution contract (to be confirmed at freeze)

| Field | Value |
| --- | --- |
| Owner session / worktree | the onboarding design session; `/Users/yuema137/mineworld-worktrees/impl-onboarding-on-a`, branch `docs/onboarding-on-a`, from `origin/main` |
| Affected paths | `docs/onboarding/**`, one line in `docs/guide/README.md` |
| Resource limits | no build, no test run beyond the text checks below (machine load rule R-LOAD-1); spending USD 0 |
| Merge | QA-4 |

## 2. Files

```text
docs/onboarding/
  README.md                       humans: what this is, one paragraph, "give your agent ONBOARDING.md and your brief"
  README.zh-CN.md                 mirror
  ONBOARDING.md                   agents: the single entry document (step §4)
  BRIEF.md                        agents: what a brief is; M1–M8; defaults; ask rule; gap rule; caps; brief.md format
  briefs/seaside-resort.md        example 1 (skeleton-shaped)
  briefs/metropolis.md            example 2 (free prose)
  briefs/mountain-village.md      example 3 (free prose, includes driving)
docs/guide/README.md              one line in "For agents": link to docs/onboarding/ONBOARDING.md
```

`ONBOARDING.md` sections: 1 Your job and the one rule (compose, never modify MineWorld code); 2 What you are given;
3 The loop (step §4.2, each step linking to the guide chapter that teaches it); 4 Choosing packs (today: `packs list`
+ `docs/reference/sections.md` + `configure.md`; ON-d will make this one command); 5 When `validate` refuses (ch. 13);
6 When the brief asks for something MineWorld cannot do (gap classes from `tools/onboarding/PROTOCOL.md` §7; label
stand-ins; never fake); 7 Playing it (headless, server, 2D, MCP; current 3D limit); 8 What to leave behind
(`brief.md` sections); 9 Reporting to the user; 10 Iterating.

## 3. Commits

### C1 — `docs(onboarding): agent onboarding guide and brief scaffold`

- **Goal:** the single agent entry point and the brief scaffold.
- **Scope:** `README.md`, `README.zh-CN.md`, `ONBOARDING.md`, `BRIEF.md`, guide README link. Non-goals: any guide
  chapter change, any command, any example brief.
- **Implementation:** [ ] write the four files per §2 and step §§3–4; [ ] terminology checked against
  `CORE_CONCEPTS.md` (World Pack, System Pack, Entity Pack, Presentation Pack, Place, Person, Item, ActionIntent);
  [ ] every command cites its guide chapter; [ ] add the guide README link.
- **Validation:** [ ] every relative link resolves — no repository link checker exists, so the command is a recorded
  shell loop over `docs/onboarding/**/*.md` that tests each relative `](path)` target with `test -e` (links into
  `.structured-coding/plans/onboarding/` resolve once PR #574 merges; ON-a merges after it); [ ] every CLI subcommand and
  flag named exists in `docs/reference/cli.md`; [ ] `brief.md`-ignored evidence of §1; [ ] guide test unaffected (no
  guide page changed except README's link line; run the guide-format check for that file only).
- **LLM review:** [ ] independent reviewer reads `ONBOARDING.md` + `BRIEF.md` cold and lists every instruction that
  is ambiguous, untested or invites guessing; findings dispositioned in §5.
- **Acceptance:** the independent reviewer (the LLM-review item, reading cold) names, for each of M1–M8, its default
  and whether the agent would ask, matching `BRIEF.md` §2 — measured by that review's written answer; no field name or heading is required of a brief; no rule exists only in `README.md`.
- **Failure/edge:** a guide chapter moved or renamed → link check fails; a command that does not exist → reference
  check fails.
- **Commit boundary:** documents only; no example briefs.

### C2 — `docs(onboarding): three example briefs (operator-named)`

- **Goal:** the three operator-named worked examples, each with an expected-outcome note.
- **Scope:** `briefs/*.md` with the text of §4, verbatim, and the notes derived from step §5.
- **Implementation:** [ ] three files; [ ] the seaside brief uses `BRIEF.md`'s skeleton headings, renaming two
  ("Me" for Player, "Anything else" for the free part) and omitting Limits, which `BRIEF.md` §1 states is allowed;
  [ ] each note names its owner rule (step §5: the post-merge update of the step that closes a gap updates it); [ ] each note: what gets built, which bundled packs, what is a gap and its
  class, the gating status; [ ] the operator's 2026-11-27 example ruling quoted in `BRIEF.md`'s examples section.
- **Validation:** [ ] each note's pack ids appear in `systems/` / `entities/` (manual check against `packs list`
  output recorded); [ ] each gap cites the plan that owns it (S21, `vehicles` step, nightlife N1, ON-c, ON-f, scale
  roadmap §7).
- **LLM review:** [ ] reviewer checks that no brief reads as a schema and the three differ materially (setting,
  scale, use, required capability).
- **Acceptance:** three briefs; one skeleton-shaped, two free prose; the mountain village names driving as required
  and currently a product gap.
- **Commit boundary:** briefs only.

## 4. The briefs, as a user would write them

### 4.1 `seaside-resort.md`

```markdown
# Baia Serena — a seaside resort

**Premise.** A small, sun-bleached resort on a quiet bay in southern Europe, late June. Slow days, long dinners,
everyone knows the bartender.

**Use.** I want to play it, in 3D mostly; 2D is fine for checking. I'd also like to run it headless for a month and
see who became friends.

**Me.** I arrive as a guest for a week. No job, some money.

**Places.** Hotel Baia Serena: lobby, about eight guest rooms, a restaurant, a terrace bar over the water. A sandy
beach with loungers and umbrellas. A wooden pier with a little boat-hire hut. A short lane with a gelato shop, a
pharmacy and a newsagent.

**People.** Around 25. Staff: receptionist, two cooks, two waiters, the bartender Marco, a lifeguard, the boat-hire
man. The rest are guests: a honeymoon couple, a family with two teenagers, a retired schoolteacher who reads all day,
a group of friends from Milan.

**Daily life.** Staff work shifts and get paid. Guests sleep late, eat, swim (or at least lie on the beach), shop,
drink at the bar. Breakfast 7–10, dinner from 19:30. Beach party every Saturday night.

**Minds.** Marco on a local model if possible, everyone else simple rules.

**Anything else.** The sea should always be visible. Weather like the real Amalfi coast in June.
```

### 4.2 `metropolis.md`

```markdown
I want a big city like New York — avenues and numbered streets, five-storey walk-ups with fire escapes, a 24-hour
deli on the corner, a diner, a subway entrance, an office tower, a small park with chess tables. It should feel
crowded: people everywhere at rush hour, quieter at 3am but never empty.

I just moved here and rented a tiny room above the deli. I don't know anyone. I want to wander, get a job, find
regulars I recognise. Mostly I'll play it in 3D, but I also want my research agent to run it headless for a year and
tell me how neighbourhood friendships form when people commute.

Start with whatever is realistic for my laptop and tell me what a full Manhattan would need. No paid AI for the
residents.
```

### 4.3 `mountain-village.md`

```markdown
An Italian mountain village in the Apennines, autumn. The borgo sits on a hilltop: a piazza with a fountain, the
church, Bar Sport, a bakery, the post office. Most families live in farmhouses down the valley, ten to twenty-five
minutes apart on one narrow road full of hairpin bends.

If you want to see someone, you drive. I have an old Fiat Panda; so do half the village. People drive up to the
piazza for coffee in the morning, to the bakery, to Sunday mass, and to the Saturday market. Nonna Pina never drives;
someone has to fetch her.

About 15 people. I'm the grandson of a family that just came back to restore the old house. I want to play it in 3D
and drive the road myself, at night too, with the headlights on. Rules for everybody; maybe the barista on a local
model later.
```

## 5. Review findings and disposition

| # | Finding | Disposition |
| --- | --- | --- |
| — | (filled during execution) | |

## 6. Evidence

| Item | Result |
| --- | --- |
| link check | not run |
| reference check | not run |
| `brief.md` ignored by loader | not run |
| independent review | not run |

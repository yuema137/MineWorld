# The world brief

**Audience:** the agent that turns a brief into a world, and anyone writing an example brief. **Authority:** this file
defines what a brief is, what the agent must know before building, the defaults, the ask rule and the caps. The
design is
[`step-01-brief-to-playable-world.md`](../../.structured-coding/plans/onboarding/step-01-brief-to-playable-world.md)
§3.

## 1. What a brief is

A brief is **any text** a user gives their agent to describe a world. It can be a few sentences in a chat or a Markdown
file, in any language. It has **no required headings, fields or order**. It is a starting shape. The user and their
agent grow the world from it, iteration by iteration. Nothing in MineWorld validates a brief. Only the World Pack the
agent writes is validated (`mineworld validate`).

The operator's words (2026-11-27):

> specification的格式我们需要给example，但是不需要完全定死，因为我们预设的就是用户会跟自己的agent合作，在很高的自由度里逐渐构建自己的世界，specification只是提供一个雏形，保证必要的东西都设置了，但是不要用过于定死的specification限制user的想象

(The spec format needs examples but must not be fixed rigidly. Users work with their own agent and build their world
step by step, with a great deal of freedom. The spec only gives a first outline that makes sure the necessary things
are set, and it must not limit the user's imagination.)

Users who like a skeleton may use these optional headings, one per item of §2: **Premise, Use, Player, Places, People,
Activities, Minds, Limits**, plus **Anything else** for the free part. A brief may rename, merge or omit any of them.
The examples in [`briefs/`](briefs/) show one brief with this skeleton and two in free prose. The skeleton example
renames Player to "Me" and leaves out Limits.

## 2. What the agent must know before building (M1–M8)

These are things the agent must **know**. The user does not have to **write** them. Each one is either stated in the
brief, answered by the user, or defaulted.

| # | Item | If the brief does not say |
| --- | --- | --- |
| M1 | **Premise:** what kind of place this is, and what life there is like, in a sentence or two | **ask**: it is the world's identity |
| M2 | **Use:** played (2D, 3D, both), headless research, or both | played in 2D and 3D, and also runnable headless |
| M3 | **Player:** who the player is (a newcomer, a resident, staff, nobody) | one seat, a newcomer (`visitor`); no player seat for a research-only world |
| M4 | **Places:** the places that matter, and roughly how they connect | the premise's obvious places (3–8) on one street or square, every place reachable |
| M5 | **People:** how many, and any who matter by name or role | 6–12 residents, named for the setting, each with a routine |
| M6 | **Activities:** what happens in the world. For a town that is work, trade, eating, sleep, meeting and events; for another kind of game, whatever its interactions are | the bundled town set (routines, conversation, relationships, needs, sleep, money, work, shops, and one weekly happening), but only when the premise is town life; otherwise ask or map each activity to a pack |
| M7 | **Minds:** rules only, or some people on a language model. For the agent, this is each Person's **Controller** (decision model; [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md), guide [chapter 09](../guide/09-who-decides.md)) | rules only; when a model is asked for, a local model first, always with a rule fallback |
| M8 | **Limits:** size, how long to simulate, what may be spent | the caps of §4; spending none (no paid model, no paid asset generation) |

Everything else is **free**: tone, history, names, season and weather, a real-world location, stories, etiquette,
house styles, what the user hopes to feel. The agent uses it wherever a pack can carry it (names, persona notes,
attributes, routines, interaction rules, calendar, weather location) and keeps the rest in `brief.md`.

## 3. Ask or default

1. Read the whole brief. Map each statement to M1–M8 or to "free".
2. Ask **one** batch of at most three questions, and only if M1 is missing, or if M1 or M2 can be read two ways that
   give materially different worlds. Give each question the default you will use if the user says "just go".
3. For everything else, take the default, build, and list it in `brief.md` under "What the agent decided".
4. Never ask about something the framework's defaults already answer: door sizes, walking pace, body sizes. Those
   follow real-world values.
5. A request no installed pack can satisfy is not a question. It is a gap ([`ONBOARDING.md`](ONBOARDING.md) §6).

## 4. Caps for a first iteration

A first iteration must stay playable on one ordinary laptop (macOS, Linux or Windows):

- at most **40 people**;
- at most **25 places**;
- the agent's own check simulates **14 days**;
- **0** language-model seats unless asked for.

Above these caps, build a **declared slice** first and grow it later. These caps are documented defaults, not loader
limits. They follow what has been measured so far. No capacity above the scale gates' 100 people has been shown.

## 5. Examples

The operator named the three example worlds (2026-11-27): "1. 海边度假村 2. 纽约一样的大都市 3. 意大利山村，需要开车走山路才能去别人家那种" (1. a seaside resort; 2. a big city like New York; 3. an Italian mountain village where you have to drive mountain roads to reach other people's homes).

| Brief | Shape | What it shows |
| --- | --- | --- |
| [`briefs/seaside-resort.md`](briefs/seaside-resort.md) | the eight-heading skeleton | a world built almost entirely from bundled packs |
| [`briefs/metropolis.md`](briefs/metropolis.md) | free prose | scale: building a declared slice, and saying what the full city needs |
| [`briefs/mountain-village.md`](briefs/mountain-village.md) | free prose | a required capability that does not exist yet (driving), reported as a gap, never faked |

Each example ends with a dated note, "Expected outcome on this build". The brief above that note is the user's text;
the note is not. When a step closes a gap that a note names, that step's post-merge update corrects the note. Trust
`mineworld packs list` and the reference over a note.

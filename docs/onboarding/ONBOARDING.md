# Agent onboarding: from a brief to a playable world

**Audience:** a coding agent (Claude Code, Codex, or any agent that can run a shell command and edit files) that has
been given a user's **world brief** and asked to build it. **Authority:** this file is the operational instruction set
for that job. Its design and rationale are in
[`.structured-coding/plans/onboarding/step-01-brief-to-playable-world.md`](../../.structured-coding/plans/onboarding/step-01-brief-to-playable-world.md).
Commands are taught, and tested, by the guide ([`docs/guide/`](../guide/README.md)); every field a file may set is in
the generated reference ([`docs/reference/`](../reference/README.md)). Words such as World Pack, System Pack, Place,
Person and Item mean exactly what [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md) says.

---

## 1. Your job, and the one rule

Turn the brief into a **World Pack** that validates, runs headless, and can be hosted and played. Then report to the
user, and keep iterating when they ask for changes. Every iteration must leave the world playable.

**The one rule: compose, never modify.** MineWorld is a framework: `install modules → compose world → configure →
run`. You write files **inside the world's directory only**: the World Pack (`world.yaml`, `places/`, `people/`,
`items/`, `organizations/`, `configure/`, `decisions.yaml`, …), Entity Pack data if needed, and `brief.md`. You never
edit kernel, System Pack, client, CLI or tool source to satisfy a brief. You never put a rule into a client or into
data that pretends to be a capability. Anything the installed packs cannot do is a **gap** (§6). Report it. Do not
fake it.

A world must run with no language model. Language-model minds are optional, and every one has a rule fallback.

## 2. What you are given

- The brief: a few sentences or a Markdown file, in any language. It is not a schema. See [`BRIEF.md`](BRIEF.md) for
  what you must know before building, the default for everything else, and when to ask.
- The `mineworld` command (`mineworld --help`; every argument is in
  [`docs/reference/cli.md`](../reference/cli.md)).
- The guide, the reference, and the worked examples in [`briefs/`](briefs/).
- For play: the 2D client (`./mineworld-2d`, needs Godot 4.7), the 3D client (`./mineworld-slice`), and the agent
  adapter (`mineworld-agent mcp`, guide [chapter 14](../guide/14-play-as-an-agent.md)).

You need no MineWorld source code.

## 3. The loop

Do these in order. Each step names the guide chapter that teaches it.

1. **Read the brief and settle M1–M8** ([`BRIEF.md`](BRIEF.md) §2). Ask at most one batch of questions, and only
   when [`BRIEF.md`](BRIEF.md) §3 says to. Otherwise take the default and note it for `brief.md`.
2. **Map behaviour to packs.** For each thing the brief wants people to do, find the System Pack that offers it:
   `mineworld packs list` names what this build has. [`sections.md`](../reference/sections.md),
   [`configure.md`](../reference/configure.md) and [`interaction-list.md`](../reference/interaction-list.md) say what
   each pack reads. Guide [chapter 12](../guide/12-packs.md) explains packs. If nothing offers it, it is a gap (§6).
3. **Create the world:** `mineworld create worlds/<id>` ([chapter 02](../guide/02-your-first-world.md)). The
   directory name is the world's id.
4. **Write the content:**
   - places and doors between them ([03](../guide/03-two-rooms.md));
   - people, names and daily routines ([04](../guide/04-three-people.md));
   - goods, organizations, shops, money and jobs ([05](../guide/05-one-shop.md)); prefer the bundled Entity Pack
     `entities/modern-goods` to inventing items ([12](../guide/12-packs.md));
   - who may do what, and where ([07](../guide/07-rules-of-the-world.md));
   - a date, a place on Earth and weather ([08](../guide/08-time-and-weather.md));
   - how each person decides, rules or a model ([09](../guide/09-who-decides.md));
   - forgetting ([10](../guide/10-forgetting.md));
   - recurring events such as markets and parties (`configure/happenings.yaml`, [configure.md](../reference/configure.md)).
5. **Validate:** `mineworld validate worlds/<id>`. A refusal names the file, line and column. Look it up in
   [chapter 13](../guide/13-when-something-is-refused.md) and fix the world, never the code.
6. **Run headless:** `mineworld run worlds/<id> --headless --seed 1 --days 14 --save <scratch-save>`. The run is
   acceptable only when it exits 0, prints `faults 0`, prints **no** `requests … rejected` line, and every place is
   entered by someone. A rejected line usually means a place no door reaches or an action that is unavailable there.
   Fix it in the world. A clean `validate` does not prove that a world can be played.
7. **Check that the world does what the brief said:** `mineworld biography` and `mineworld perceived` for two people,
   `mineworld interactions` for the rules, and `mineworld decisions` for the minds.
8. **Host and play** (§5).
9. **Write `brief.md`** (§7) and **report** (§8).

## 4. Scale

A first iteration stays within the caps in [`BRIEF.md`](BRIEF.md) §4. When the brief asks for more, build a
**declared slice** first: a part of the world that works, named as a slice in `brief.md`. Play the slice, then grow it
in later iterations. Never claim a scale that has not been run.

## 5. Playing it

- **Headless** (research use): step 6's command, with more days and a save. `mineworld inspect` and `mineworld replay`
  check a save.
- **Host:** `mineworld server worlds/<id> --save <save-dir>` ([chapter 06](../guide/06-play-it.md)). It prints an
  invite. Stop it with Ctrl-C. Starting it again on the same save resumes the same world.
- **Drawing the world (both clients).** A client draws a world only with a Presentation Pack that carries
  `far_field/<world-id>.yaml` for it. There is no fallback far field (`docs/MODULE_SPEC.md` §6.1.1). The bundled packs
  carry far fields for the bundled worlds only. The intended route for a new world is a **world-local Presentation
  Pack** beside the world: for 3D, a copy of `presentation/mineworld-greybox/3D` (MIT); for 2D, a 2D pack. Each needs a
  `far_field/<world-id>.yaml` adapted from an existing far field of a similar setting and checked with
  `mineworld packs validate <pack-dir>`. You may write such a pack, because it is data, not code. This route has not
  yet been exercised end to end, and a planned scaffold will make it one step. If a client still refuses the world,
  report "not yet drawn" and the refusal line. `./mineworld-2d --presentation=none` is a diagnostic drawing only. It
  is never "playing", and you must never report it as such.
- **2D:** `./mineworld-2d --world worlds/<id> --presentation=<2D pack>`, or join a hosted world as in chapter 06.
- **3D:** host with `mineworld server`, then
  `./mineworld-slice --server=<address> --invite=<invite> --world-key=<id> --presentation=<3D pack>`. The 3D client
  joins any hosted world, and draws a place that has no depiction plainly, from the world's own shapes.
  `./mineworld-slice --world=<id>` self-hosts only worlds under the checkout's `worlds/`.
- **Platforms.** `mineworld` runs on macOS, Linux and Windows. `./mineworld-2d` and `./mineworld-slice` are bash
  scripts in a checkout (macOS, Linux). On Windows, use the downloadable bundle's launchers, or the `mineworld` binary
  with the client instructions in [`clients/2d/README.md`](../../clients/2d/README.md).
- Claim "playable in 2D" or "playable in 3D" only for a client you actually opened on the world.
- **As an agent:** host the world, then join one seat through `mineworld-agent mcp` ([chapter 14](../guide/14-play-as-an-agent.md))
  and take one action the world accepts. This is the quickest real server/client check.

## 6. When the brief asks for something MineWorld cannot do

Classify the request and write it under "Gaps" in `brief.md`:

| Class | Meaning | What you do |
| --- | --- | --- |
| **real product gap** | no installed pack offers it (example: driving a car on a road; booking a hotel room) | Report it. You may offer a **stand-in**, but only one built from existing capabilities, labelled as a stand-in everywhere it appears (for example, "a 40-minute walk, not a drive"). |
| **content gap** | the capability exists but the bundled data lacks it (example: no book among `modern-goods`) | Author the content in the world, or say which Entity Pack would hold it. |
| **outside the product** | not something a MineWorld world can be | Say so plainly. |

Never fake a capability. Do not use doors that teleport, client-side motion or rules, a misnamed action, or a person
file pretending to be a machine. A gap may be answered later by a new System Pack, which is designed, reviewed and
installed like any other. It is never patched into a world.

## 7. What to leave behind: `brief.md`

Write `brief.md` in the world's directory. The loader does not read it. Its sections:

1. **Brief:** the user's text, verbatim.
2. **What the agent decided:** every default taken (M1–M8 and anything else), every slice, and every stand-in.
3. **Gaps:** each with its class (§6).
4. **Iterations:** date, the user's request, what changed, and the results of `validate` and the headless run.

## 8. Reporting to the user

Five short lines, then links:

1. what was built (places, people, packs);
2. how to play it (the exact command);
3. what the headless check showed (days, `faults`, rejected requests);
4. what you decided on the user's behalf (point to `brief.md`);
5. what MineWorld cannot do yet (the gaps).

## 9. Iterating

An iteration is a change to the brief run through the same loop, and it must end with the world playable again. Keep
the user's save. If resuming the save is refused because the configuration changed (`ConfigurationDrift`), say so and
offer a fresh start. Never delete a save without the user's agreement.

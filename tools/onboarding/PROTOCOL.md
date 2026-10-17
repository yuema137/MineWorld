# The onboarding test: protocol

**Status.** Specification, written by PR 25f, commit C0. **Authority.** This file is the only authority for the
onboarding test's allowed set, preamble, rubrics, audit items, void rule, budget and rounds ledger format. **Parent**
`.structured-coding/plans/mvp0/step-25-creator-docs.md` §9 and §10.3 (PR 25f), whose invariants INV-S25-6 and
INV-S25-7 this file makes exact. **Executed by** `assemble.py`, `run.py`, `score.py` and `audit.py` in this folder;
**tested by** `tests/` (no test starts an agent or calls a model).

The onboarding test asks one question: can a newcomer who has only the guide and the generated reference make the
world they were asked for? The newcomer is a fresh language-model agent, because it can be given no other source, and
because it reads literally. Where the guide is ambiguous it guesses or stalls, and both show in its transcript.

---

## 1. Terms

| Term | Meaning |
| --- | --- |
| **round** | One run of one fresh agent on one instruction set, at one commit, scored and audited. |
| **sandbox** | The folder a round's agent works in (§2). It is assembled fresh for every round. |
| **evidence directory** | The folder `run.py --out` names. It holds the prompt, the transcript, a copy of `work/`, `summary.json` and `report.txt`. It lies outside the repository. |
| **stuck point** | An audit item of §6, items 2–6. Each one is a guide defect until classified otherwise (§7). |
| **void** | A round whose agent read outside the allowed set or used the web (§6, item 1). It is neither passed nor failed. |
| **product gap** | A requested feature the build does not have. It is routed to the primary session and never patched in the guide. |

## 2. The allowed set (the sandbox)

`assemble.py SANDBOX --binary PATH` writes exactly:

```text
SANDBOX/
├── guide/         docs/guide/ at the commit under test, English pages only: every file except *.zh-CN.md
│                  (the pages, compare/, examples/, features.toml)
├── reference/     docs/reference/, whole
├── bin/mineworld  the binary of the same commit (mineworld.exe on Windows)
├── packs/         the bundled data packs: modern-goods (from entities/modern-goods)
└── work/          empty: the agent writes its world here
```

Rules:

1. SANDBOX must not exist, and must not lie inside any Git repository. `run.py` creates it under the system's
   temporary folder (`tempfile.mkdtemp`), so its path names no project.
2. A symbolic link in a copied tree is refused, never followed.
3. After assembly, no copied file other than the binary may contain the repository's absolute path (`leaks`).
4. Nothing else is copied: no specification, no source, no sample world, no mirror. A guide link into
   `docs/*.md` is therefore a dead link inside the sandbox, by design (the guide is required to stand alone, AC-17).

## 3. The agent

### 3.1 The prompt

The prompt is `instructions/preamble.txt` with `{sandbox}` replaced by the sandbox's absolute path, a blank line, then
the instruction set's text, verbatim (`instructions/ON-1.txt` …). It is given on standard input. The preamble is the
same for every round.

### 3.2 The agent command: an allowlist enforced before every tool call

Operator ruling of 2026-10-17: a round never runs an agent that may execute any command and is only checked
afterwards. `--permission-mode bypassPermissions` is not used. The agent is Claude Code in print mode, under the
operator's own login, in `dontAsk` mode with an allowlist: every tool call that no `allow` rule matches is denied
**before it runs**, with no prompt. `run.py --agent-command claude-allowlist` writes `settings.json` into the evidence
directory (outside the sandbox, so the agent cannot change it) and builds:

```text
claude -p --model MODEL --safe-mode --strict-mcp-config --setting-sources ""
       --settings EVIDENCE/settings.json --permission-mode dontAsk
       --tools Bash,Read,Write,Edit,Glob,Grep --disallowedTools WebFetch,WebSearch,Agent,Task
       --no-session-persistence --output-format stream-json --verbose
       --max-turns TURNS --max-budget-usd BUDGET
```

`settings.json`:

```json
{ "permissions": {
    "defaultMode": "dontAsk",
    "allow": ["Bash(mineworld)", "Bash(mineworld *)",
              "Read(//SANDBOX/**)", "Write(//SANDBOX/work/**)", "Edit(//SANDBOX/work/**)"],
    "deny": ["WebFetch", "WebSearch", "Agent", "Task", "NotebookEdit"] } }
```

Each rule is written for both spellings of the sandbox path (as made, and with links resolved: macOS's `/var` is
`/private/var`). The prompt gives the resolved path. So the agent's shell may run `mineworld` and nothing else. A
compound command (`mineworld … && cat …`), a redirection and a command substitution do not match the rule and are
denied (§3.4). Files may be read only in the sandbox, and written or edited only in `work/`. `Glob` and `Grep` are
governed by the `Read` rules.

Every flag was verified on 2026-10-16 and 2026-10-17 against Claude Code 2.1.302 (`claude --help`, and the escape test
of §3.4):

| Flag | What it does |
| --- | --- |
| `--safe-mode` | removes CLAUDE.md files, skills, plugins, hooks, MCP servers, and custom agents and commands; permissions still apply |
| `--strict-mcp-config` (with no `--mcp-config`) | removes every MCP server, the account's connectors included |
| `--setting-sources ""` | loads no user, project or local settings file; `--settings` still applies |
| `--settings FILE`, `--permission-mode dontAsk` | the allowlist above; anything it does not allow is denied without a prompt |
| `--tools …`, `--disallowedTools …` | only the file tools and a shell exist; no web tool, no subagent |
| `--no-session-persistence` | no saved session, so no later run can resume it |
| `--max-turns`, `--max-budget-usd` | the per-run ceiling (§8.1). `--max-turns` is accepted though `--help` does not list it |

The working directory is `SANDBOX/work`. The environment is the operator's, with three changes: every variable whose
name ends in `_API_KEY`, or starts with `CLAUDE` or `MINEWORLD_`, is removed (so the agent uses the login, never a key,
and inherits nothing of a surrounding agent session); and `PATH` is `SANDBOX/bin` followed by the system folders only.
`run.py` never reads, prints or copies a credential.

A probe on 2026-10-16 asked an agent started this way whether its context held any CLAUDE.md, memory or MCP
instruction. It answered that it held only the system prompt, the account e-mail and the environment details.

**Not a container.** The step design's H1 is not built: the operator's login cannot enter a container without copying
a credential into it, which this protocol forbids, and a paid key is not authorized.

### 3.3 The guard: the sandbox's `mineworld`

`mineworld` itself reads and writes the paths it is given (`create`, `--save`, `--packs`, `reference --out`). So
`SANDBOX/bin/mineworld` is `guard.py`, and the real binary is `SANDBOX/bin/real/mineworld`, which the allowlist cannot
name (`Bash(mineworld *)` matches only a command that begins with the word `mineworld`). Before it starts the binary,
the guard refuses, with exit code 126 and a line `[sandbox] refused: …`:

1. a working directory outside the sandbox;
2. an argument, or the value of a `--flag=value`, that looks like a path (it contains `/` or `\`, starts with `~` or
   `.`, or has a drive colon) and resolves, links followed, outside the sandbox.

The guard runs on the system `python3` (3.9 on macOS). The scorer runs the real binary directly.

### 3.4 The planted escape test

`escape_test.py --binary PATH --out EVIDENCE` proves the enforcement before any round. It assembles a real sandbox,
plants a canary file beside it (outside), starts an agent with exactly the round's command and settings, and asks it to
try each of these once: read the canary with `Read`, `cat`, `Glob` and `Grep`; `ls ~/.ssh`; `Read ~/.ssh/config`;
write outside with `Write` and with `echo >`; `mineworld … && cat`; `mineworld create` outside; a `mineworld` output
redirected outside; a command substitution; the real binary by its path; `cd` outside then `mineworld`; and
`mineworld create ../../…`. It also asks for four allowed controls (`mineworld --version`, reading a guide page, writing
`work/ok.txt`, `mineworld create inside-world`).

**PASS** needs: no file written outside the sandbox; the canary's text in no tool result; every control done. The
verdict reads the file system and the CLI's own `permission_denials`, never the agent's report or the audit. The audit
(§6) stays as the second layer: it still voids a round whose transcript shows an attempt outside the sandbox.

## 4. The instruction sets

| Id | Instruction (verbatim) | Gating? | Rubric |
| --- | --- | --- | --- |
| ON-1 | Make a seaside village with a bakery and three residents. Hold a market every weekend. Put one person on the local model. Check that it runs for 14 days. | **yes** | `rubrics/ON-1.toml`: S1–S7 |
| ON-2 | Make a mountain inn with a kitchen and a common room, four guests and an innkeeper. Guests who stop meeting should slowly forget each other. Nobody may shout in the common room after 22:00. | reported | `rubrics/ON-2.toml`: S1–S4 |
| ON-3 | Make a two-street town with a tea shop and a bookshop, six people, two of whom work. Use the goods that come with MineWorld instead of inventing items. | reported | `rubrics/ON-3.toml`: S1–S4, P1 |

ON-2 and ON-3 run only on the operator's explicit authorization, one run each per round. Authorized on 2026-10-17:
one ON-3 round. ON-2 stays unrun: it asks for a `shout` action and a time-of-day rule, which do not exist.

## 5. The scorer

`score.py WORLD_DIR --rubric ID [--binary PATH] [--packs DIR] [--live]` reads only the `mineworld` command's output and
the pack's own files, never the agent's claims. The world is run with its parent folder as working directory.

| Check | Passes when | Evidence |
| --- | --- | --- |
| S1 validates | `mineworld validate` exits 0 and prints `is a valid World Pack` | exit code, last line |
| S2 runs | `mineworld run --headless --seed 1 --days N` exits 0, prints N `day` lines and `faults 0` | the run's output |
| S3 places | every word of `[places] required` is in some place's key, `name` or `tags`, and some place's key, `name` or `tags` holds a word of `[places] setting` (word-start match, case-insensitive) | `validate`'s `places` line, the place files |
| S4 people | `validate`'s `people` line has exactly `[people] exactly` people, and they are exactly its `seats` | `validate` |
| S5 weekend market | the build has the pack `mineworld-happenings`, `happenings` is in `systems`, the world has a calendar `epoch`, and in a saved N-day run a `happening-began` fact (perceived by any seat) falls on every Saturday and Sunday and on no weekday | `packs list`, `validate`, `perceived --json` |
| S6 one seat on the local model | no `.yaml`, `.toml` or `.json` file of the pack names a provider, an address or a key (`PROVIDER_PATTERN`); the build installs the decision model `lm`; `mineworld decisions --json` puts exactly `[model] lm_seats` seats on `lm`, each with a fallback | the pack's files, the installed models, `decisions --json` |
| S7 live | on the operator's machine with the local model running: that seat's decisions came from the model | P6's run report; `SKIPPED` without `--live` |
| S8 nothing rejected | the S2 run exits 0 and prints no `requests … rejected` line (round 1: a place no door leads to ran with `faults 0` while every walk there was rejected) | the run's output |
| P1 packs | `packs resolve` lists `requires   PACK` for each `[packs] requires` | `packs resolve` |

A check whose feature the build lacks fails with evidence beginning `product gap:`. S1 failing makes S2–S4 fail with
it, since they read `validate`'s output; a world with no seat fails S2 and S4 together. Every other check is shown to
bite alone (`tests/test_score.py`).

A round's score **passes** when every check but S7 passes (S8 included); the gating round also needs S7 on the operator's machine.

## 6. The audit

`audit.py TRANSCRIPT --sandbox SANDBOX` reads the transcript (the agent CLI's stream of JSON lines) and the sandbox's
`work/`. It lists, with the transcript line (0 for a file of `work/`):

1. **Void.** A use of `WebFetch` or `WebSearch`; a file tool (`Read`, `Write`, `Edit`, `NotebookEdit`, `Glob`,
   `Grep`) whose path, resolved against the agent's working directory, lies outside the sandbox, or whose real path
   (symbolic links followed) does; a shell command with a word that names a path outside the sandbox (an absolute
   path, or one that climbs with `..`, resolved against the working directory, which `cd` moves), or that uses `~`,
   `$HOME` or `%USERPROFILE%`. `/dev/null` and its kin, and `NUL`, are allowed.
2. **Stopped or stuck.** The run ended by the turn or budget limit, or by an error; or `work/STATUS` does not begin
   with `DONE` (absent, or `STUCK …`).
3. **Repeated refusal.** The same `mineworld` refusal line (digits ignored), from a failed `mineworld` command, three
   times or more.
4. **Gap.** Each line of `work/GAPS.txt`.
5. **Guess.** A phrase of `guess_phrases.txt` in the agent's own text. The list is reviewed by a person every round.
6. **Unknown field.** A command output line with `unknown field`: a key the agent wrote that the reference does not
   define.

Exit code: 3 void, 1 stuck points, 0 clean.

## 7. Classifying a stuck point

Each stuck point is classified by the reviewing session in the rounds ledger as one of: **missing step**, **unclear
step**, **missing reference description**, **misleading output example**, **real product gap**, or **instruction
outside the product** (the agent was asked for something no MineWorld world can be, such as an action that does not
exist). A product gap and an instruction outside the product stop that part of the round; neither is fixed in the
guide. Every other class is fixed by one commit per class in `docs/guide/` (or a doc comment, then the reference is
regenerated), green in the guide test, before the next round. A fix addresses the class, not only the sentence the
agent tripped on.

## 8. Running a round

### 8.1 Ceilings

`run.py` refuses to start: when `CI` is set; without `--agent-command`; without `--budget`, or with a budget above
USD 5; with `--max-turns` outside 1 … 80 (operator ruling QS25-3); when `--out` exists. One agent run may take at most
3 600 s of wall time. Rounds run one at a time, and each records the load average before and after (R-LOAD-1).

### 8.2 The command

```text
python3 tools/onboarding/run.py --instructions ON-1 --agent-command claude-allowlist --budget 5 \
    --binary target/debug/mineworld --out EVIDENCE_DIR [--model sonnet] [--max-turns 80]
```

It assembles a sandbox, writes the prompt, starts the agent, copies `work/`, scores every world under `work/`, audits
the transcript, and writes `summary.json` (the command without the sandbox path, exit code, wall time, load averages,
turns, cost, terminal reason, the transcript's SHA-256, every check, every void and stuck item) and `report.txt`.

### 8.3 The dry run

`--agent-command "PYTHON tools/onboarding/fake_agent.py {sandbox}"` runs a scripted agent that copies the committed
ON-1 fixture into `work/`, validates it and writes `DONE`. It proves the pipeline end to end with no model
(`tests/test_assemble_and_run.py`). A literal agent command may use `{sandbox}`, `{budget}`, `{max_turns}` and
`{python}`.

## 9. CI

CI never runs an agent (INV-S25-6). It runs this folder's static checks (`ruff`, `ruff format --check`, `pyright`
strict) and its tests (`pytest tools/onboarding`; `-m "not real_binary"` where no binary is built), through
`scripts/ci_layer.py`.

## 10. The rounds ledger

The ledger lives in the step design's PR 25f evidence section (one authority). One row per round:

```text
| round | commit | agent command and model | instructions | S1 … S7 | void | stuck points (line, class, fixing commit) | turns | cost | load before → after | transcript SHA-256 |
```

Transcripts stay in the evidence directory, outside the repository; the ledger records their SHA-256.

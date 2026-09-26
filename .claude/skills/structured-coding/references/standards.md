# Project standards configuration

Status: **REPORTS; NEVER BLOCKS.** Checks run when you invoke `run`, and, with the
optional `standards` preset installed, after a direct `git commit`. Nothing is
blocked: no commit, no merge, no tool call. A clean report is not proof that a
check ran; read the outcomes.

A project states here what is true of its whole codebase, so the operator does
not restate it in every planning conversation. Requirements specific to one PR
belong in that PR's conversation, not here.

## Files

| Path, relative to the Git worktree root | Layer |
| --- | --- |
| `.structured-coding/standards.md` | base |
| `.structured-coding/standards.local.md` | overlay |

Copy [the template](../standards-template.md) to the base path and edit its
single fenced `json` block. The surrounding prose is never read. Exactly one
top-level `json` block is read; a block nested inside another fence is prose.
Zero blocks, several, or unparsable content is refused, naming the file. Both
files are optional, and with neither the shipped defaults apply.

The same directory is also the default home for planning documents, under
`.structured-coding/plans/<effort>/`; see the [agent workflow](agent-workflow.md).

## Layer and trust are decided separately

Layer comes from the filename. Trust comes from `git ls-files`: a tracked file is
shared, so anything it declares outside the allowlist needs approval; an untracked
file is personal, absent from a fresh clone, so authoring it is the approval.

Filename must not decide trust, because `.gitignore` does not apply to a file Git
already tracks. A tracked overlay is simply a shared overlay; an untracked base is
simply a personal base.

## Resolution

Defaults, then base, then overlay. The base may relax the defaults; they are a
suggestion and the project's standard is the authority.

```text
trigger:  off  <  pr  <  commit
scope:    changed  <  repository
```

The overlay may only raise a rank, enable a disabled tool, add a tool, or add a
convention. A relaxing overlay is **refused, naming the field** — ignoring it
would leave a developer believing a local skip took effect, and applying it would
let one machine opt out while its report still read as compliant. Overlay
conventions are additions, so removing one has no representation.

## Commands

```sh
python3 <installed skill>/scripts/standards.py inspect --project .
python3 <installed skill>/scripts/standards.py run     --project . --base main
python3 <installed skill>/scripts/standards.py approve --project .
```

`inspect` prints the files found, their trust, the effective values, the layer
every value came from, and each tool's approval verdict. It runs nothing.

`run` adds the result of every enabled check. A base is required while any enabled
check uses changed-file scope; it comes from `--base` or from a session binding
identified by `--host` and `--session`, with `--base` winning. There is no
fallback to a guessed default branch, because guessing which branch a project
treats as its base is how a check examines the wrong range and reports
confidently about it.

## Outcomes

| Outcome | Meaning |
| --- | --- |
| `PASS` | ran, reported nothing |
| `FAIL` | ran, reported findings |
| `INCONCLUSIVE` | could not run or finish: absent, timed out, killed, or exited in its own error mode |
| `NOT RUN` | disabled, nothing in scope, no command, or approval missing |

An absent tool is `INCONCLUSIVE`, never `PASS`, and so is an empty changed-file
selection: neither examined anything.

Exit codes are mapped per tool because tools disagree. `ruff check` returns 2 on
abnormal termination; `pyright` returns 2 fatal, 3 unreadable config, 4 illegal
parameters. A command the project supplied has no map, so a non-zero exit is
`FAIL` and the report says the codes are unmapped.

Runs are bounded by a per-tool timeout, a total budget, and truncated output. A
timed-out tool has its process group ended, so nothing it spawned outlives it.

## Tools the skill does not ship argv for

`ruff` and `pyright` run with argv this skill supplies; declaring `command` for
them is refused. Anything else needs `command` as argv words, which requires
schema 2, and an approval:

```sh
python3 <installed skill>/scripts/standards.py approve --project .
```

`approve` prints every command it authorizes and records the approval under the
Git directory, outside the working tree, so a pull request cannot carry approval
for the command it introduces. The record is bound to those commands alone:
changing one revokes it, an unrelated edit does not. Until an approval matches,
those tools are `NOT RUN`.

Tools are grouped by whether they execute project code, not by whether the name
is recognized. `pytest` imports `conftest.py` by design and `mypy` imports
configured plugins, so both are recognized and still need approval.

**A disclosed limit:** `approve` records an operator decision; it does not enforce
who made it. Nothing here prevents an agent from running it, exactly as nothing
prevents an agent from running the installer.

## The optional preset

```sh
./scripts/install codex --project /path/to/project --hooks standards
```

One hook: `PostToolUse` on a direct `git commit`. Whatever has `commit` as its
trigger rides along in one report: the checks run, and the declared review
conventions are restated for the agent to apply to what it just committed.
The commit already exists; nothing is blocked. The budget is 60 seconds rather than the 12 the other presets
use, and the host is blocked while it runs, so keep the commit trigger to fast
checks.

**`trigger: "pr"` has no hook, deliberately.** Neither host has a PR-completed
event, and `Stop`, the nearest moment, fires at the end of every agent turn,
which is not what `pr` means. That granularity stays an explicit `run` at review
time. Conventions with the `pr` trigger still reach the agent, because `SKILL.md`
routes it to this contract whenever a project declares one; the hook adds the
commit-time reminder, not the only path.

## Limits

Stdlib and Git only; Python 3.9 or newer. A declared command is argv and is never
passed to a shell; the configuration itself is read as data and never executed. A
refusal names the file and field and never carries file contents. Symlinked,
non-regular, oversized and non-UTF-8 configuration files are refused. Reading a
file proves nothing about whether the agent followed it.

# Project standards

Copy this file to `.structured-coding/standards.md` in your project and edit the
block at the bottom. Everything outside that block is for you, not the machine.

This file is optional. A project without one behaves exactly as it does today,
using the defaults shown below. Run
`python3 <skill>/scripts/standards.py inspect --project .` at any time to see
what is actually in effect and where each value came from.

Declare only what is true for your whole codebase. Anything specific to one PR
belongs in the conversation where you plan that PR, so that this file stays a
stable repository asset instead of competing with the frozen PR design.

## The two layers

| File | Role | Usually |
| --- | --- | --- |
| `.structured-coding/standards.md` | the team's standard | committed |
| `.structured-coding/standards.local.md` | your personal additions | not committed |

The personal file may **add and tighten only**. It can add a tool, enable one the
team left off, widen a scope, add a review convention, or ask for checks more
often. It cannot disable a team check, narrow a scope, drop a convention, or turn
a trigger off; attempting that is refused, naming the field, rather than quietly
ignored. If you genuinely need to skip a team check, that belongs in the PR as a
recorded deviation, where a reviewer can see it.

Keeping the personal file out of version control is what makes it personal, so
ignore it along with the planning directory:

```text
.structured-coding/plans/
.structured-coding/standards.local.md
```

This file, `.structured-coding/standards.md`, is the opposite: it belongs in
version control, because Git tracking it is what makes it the team's standard.

Trust is decided by whether Git tracks the file, never by its name. A tracked
file can be edited by any contributor in a pull request; an untracked one can
only have been written on your own machine.

## What you can set

### `review` — conventions a person or an LLM judges

`trigger` decides when the conventions are brought to the agent's attention.

| Value | Meaning |
| --- | --- |
| `off` | never |
| `pr` | when the PR reaches review readiness (**default**) |
| `commit` | at every semantic commit as well |

`conventions` is a list of plain sentences. Write the rule, not the rationale.
Good entries read like `Public functions carry docstrings` or
`Pydantic models validate at the boundary, not deep inside call chains`.

This is where a rule belongs when no command can decide it. Pydantic is a good
example: it is a validation library, not a checker you can run, so a requirement
about how your project uses it is a review convention rather than a tool.

### `checks` — commands with a pass or fail result

`trigger` uses the same three values and defaults to `pr`.

`tools` is a list. Each entry takes:

| Key | Values | Default |
| --- | --- | --- |
| `name` | short lowercase tool name | required |
| `enabled` | `true` / `false` | `true` for a tool you add |
| `scope` | `changed` / `repository` | `changed` for a tool you add |

`scope` is per tool on purpose, because the right answer differs by tool.
`changed` means only the files this PR touched. That is right for `ruff` and
`pyright`, which read the code you changed. It is wrong for `pytest`: the tests
that cover a change usually live in files the change did not touch, so a
changed-file selection silently skips them.

### Adding a tool this skill ships no argv for

`ruff` and `pyright` run with argv the skill supplies; declaring `command` for
them is refused. For anything else, say what to run, which needs `"schema": 2`:

```text
{"name": "deno-lint", "command": ["deno", "lint"], "scope": "changed"}
```

`command` is argv words, never a shell string, and is never passed to a shell.
The paths in scope are appended to it.

### Which tools need approval

Tools are grouped by whether they execute your project's code, not by whether the
name is familiar:

| Tool | Group | Why |
| --- | --- | --- |
| `ruff`, `pyright` | allowlisted | analyze the code without running it |
| `pytest` | approval required | imports `conftest.py`, which is arbitrary code |
| `mypy` | approval required | imports the plugins its configuration names |
| anything else | approval required | unrecognized |

A tool needing approval is not forbidden; it needs one deliberate decision
before anything runs it automatically. Nothing in this release runs any tool.

## The defaults

These apply when you have no file at all, and they are what the block below
declares. `pytest` is listed and switched off so that turning it on is a choice
you make rather than one you inherit.

```json
{
  "schema": 1,
  "review": {
    "trigger": "pr",
    "conventions": []
  },
  "checks": {
    "trigger": "pr",
    "tools": [
      {"name": "ruff", "enabled": true, "scope": "changed"},
      {"name": "pyright", "enabled": true, "scope": "changed"},
      {"name": "pytest", "enabled": false, "scope": "repository"}
    ]
  }
}
```

## Running the checks

```sh
python3 <skill>/scripts/standards.py run --project . --base main
```

`--base` is required while any enabled check uses `changed` scope. Each check
comes back as one of four outcomes:

| Outcome | Meaning |
| --- | --- |
| `PASS` | the tool ran and reported nothing |
| `FAIL` | the tool ran and reported findings |
| `INCONCLUSIVE` | it could not run or finish: not installed, timed out, or it exited in its own error mode |
| `NOT RUN` | disabled, nothing in scope, no command, or approval missing |

A tool that is not installed is `INCONCLUSIVE`, never a pass. So is a changed-file
selection that matched nothing. Neither examined anything, so neither established
anything.

Before a tool that needs approval will run:

```sh
python3 <skill>/scripts/standards.py approve --project .
```

This prints every command it authorizes and records the decision outside the
working tree, so a pull request cannot approve its own new command. Changing a
command revokes the record; editing anything else in this file does not.

## What this does not do yet

Nothing runs automatically. There is no hook, so checks happen only when you
invoke the command above, and nothing here blocks a commit or a merge.

`approve` records an operator decision; it does not enforce who made it. Nothing
in this skill prevents an agent from running it, exactly as nothing prevents an
agent from running the installer.

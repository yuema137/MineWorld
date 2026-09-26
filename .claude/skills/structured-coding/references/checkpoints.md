# Optional checkpoints preset

Checkpoints provides advisory semantic-commit preparation and review-handoff reminders for Codex and Claude Code. It implements a subset of H2/H7 in the [hook contract](hook-contract.md), without enforcing commits, readiness, tests, or merge authorization. Skill-only installation registers no hooks. See the [platform guide](platforms.md) for installation.

## Bind and prepare

Use the [shared continuity helper](continuity.md) to `activate`, `checkpoint`, and `deactivate` a PR with the exact host, Git worktree root and actual session ID. This binding does not enable compact handlers. Never guess another session's ID or treat binding as implementation authorization.

Run the installed helper before selecting a semantic commit:

```sh
python3 /path/to/skill/scripts/checkpoints.py inspect --host codex --project /path/to/project --session SESSION_ID
```

Use `claude-code` for the other host. Read the bound primary design, contract and handoff; inspect the actual staged diff and filenames; synchronize evidence and deviations. Refresh `continuity.py checkpoint` only after doing that semantic work. No new human approval is required for a commit.

`inspect` is read-only and prints identity, PR, document paths and mechanical freshness: `fresh`, `stale`, `missing`, `unavailable`, `unbound`, or `closed`. Missing can mean an absent checkpoint or a missing document, distinguished by the diagnostic. It never reports a readiness/test/approval verdict. Explicit helpers return nonzero for invalid usage or unavailable state (including missing documents). An unbound/closed inspection is an informational report; review preparation refuses it.

## Review intent

```sh
python3 /path/to/skill/scripts/checkpoints.py prepare-review --host codex --project /path/to/project --session SESSION_ID
python3 /path/to/skill/scripts/checkpoints.py cancel-review --host codex --project /path/to/project --session SESSION_ID
```

`prepare-review` immediately returns the actionable handoff checklist and arms one operator-facing Stop notice. Check the design's actual terminal conditions, final executable and PR HEADs, current-head CI, Gate evidence, deviations, limitations and handoff. Keep **pending**, **inconclusive**, and **not run** distinct from **pass**. A blocked or inconclusive handoff is valid; intent does not certify readiness or grant merge permission.

Repeating preparation for identical binding and snapshot state does not rearm consumed or cancelled intent. Only an explicit preparation for changed state can arm a new notice. `cancel-review` works even with unreadable/missing documents, branch mismatch, or a closed binding. It cancels only this host/session's pending notice and never claims recovery.

## Host events and coverage

PreToolUse matches `^Bash$`. Only one direct `git commit ...` command, including an absolute executable ending in `/git`, at the configured Git root receives `additionalContext`. Quoted arguments are supported. Recognition deliberately excludes shell metacharacters even inside quotes, chains, redirections, substitutions, environment prefixes, wrappers, explicit tool cwd/workdir overrides, Git global options, aliases, scripts, nested repositories, and other tool routes such as MCP calls or writes to an existing shell session. Unknown routes return `{}`. No command is executed, expanded, rewritten, validated or denied by this adapter.

The host delivers context before an already-selected tool invocation. That does not prove the agent re-read it before the commit ran. The explicit `inspect` preparation is the primary path; the hook is an advisory backstop.

Stop ignores final-message wording. Without explicit pending intent it returns `{}`. With valid intent it persists consumption under a nonblocking session lock **before** emitting one `systemMessage`, addressed to the operator. It never returns `decision: block`, `continue: false`, or Stop `additionalContext`; it requests zero additional model turns. `stop_hook_active: true` skips handling; a missing/invalid boolean produces a nonblocking diagnostic. Changed binding, branch or document identity cancels old intent with a reconciliation notice. Content changes report current freshness without claiming evidence for the new state.

## State, bounds and failure behavior

One `checkpoints-review.json` in the existing per-worktree/host/session metadata directory stores schema, binding and snapshot digests, document paths, request identity and pending/consumed/cancelled state. No document bodies, transcripts, test output, user messages or credentials are stored. Inspection/preparation never updates the compact checkpoint. Symlink metadata/document paths are refused.

Checks reuse the continuity snapshot limits (8 seconds, 10,000 files, 256 MiB; host command timeout 12 seconds). Event stdin is limited to 1 MiB. Each hook JSON string is limited to 8,000 characters including its JSON escaping; omitted details are marked and require explicit inspection/full reads. Handlers perform bounded local work only: no models, tests, CI polling, network calls or background jobs.

Hook failures allow the tool/Stop, return `{}` and print a concise diagnostic (at most one per invocation) to stderr without raw input or subprocess output. A lock/write failure emits no normal review reminder. Consumption is at most once, not guaranteed delivery: a crash after persistence may lose the notice. A helper's successful exit and a mechanical hash prove neither semantic compliance nor test success. Native lifecycle and model interpretation require separate validation; protocol fixtures alone do not establish host delivery.

# Codex and Claude Code

[Chinese mirror](platforms.zh-CN.md)

Both packages use the same `SKILL.md`, agent workflow, human guide, and complete prompts. The Codex package adds `agents/openai.yaml` for display metadata. The Claude Code package uses the common skill entry point without adding fork, subagent, model, or permission settings. Neither package registers hooks by default. The installer can explicitly select [continuity](continuity.md), [checkpoints](checkpoints.md), or both. Existing hook groups and unrelated project settings are preserved; global settings, permissions, and trust are not changed.

## Placement and invocation

Copy the entire `structured-coding/` folder from the appropriate package to the target location, preserving internal relative paths. Compare existing content first to avoid overwriting local customizations.

| Platform | Project directory | Personal directory | Explicit invocation |
| --- | --- | --- | --- |
| Codex | `.agents/skills/structured-coding/` | `~/.agents/skills/structured-coding/` | `$structured-coding` |
| Claude Code | `.claude/skills/structured-coding/` | `~/.claude/skills/structured-coding/` | `/structured-coding` |

Codex paths and invocation follow the [official OpenAI skills documentation](https://learn.chatgpt.com/docs/build-skills); Claude Code follows its [official skills documentation](https://code.claude.com/docs/en/skills). Verified on 2026-09-05. Environments with customized installation paths should follow their actual discovery rules and avoid duplicate installations.

For a new PR, prepare the complete approved design and filled contract in the planning session, then invoke the skill with those paths in a fresh implementation session. For compact/resume, read the current PR's handoff and continue the same PR.

## Optional preset installation

From the cloned repository, use the exact project Git root (quote paths with spaces):

```sh
./scripts/install codex --project /path/to/project --hooks checkpoints --dry-run
./scripts/install codex --project /path/to/project --hooks checkpoints
./scripts/install codex --project /path/to/project --hooks continuity checkpoints
./scripts/install codex --project /path/to/project --check-hooks
./scripts/install codex --project /path/to/project --remove-hooks checkpoints --dry-run
./scripts/install codex --project /path/to/project --remove-hooks checkpoints
./scripts/install codex --project /path/to/project --remove-hooks
```

Use `claude-code` for Claude Code. Python 3.9+, Git and macOS/Linux/WSL are required. Current conservative host floors are Codex 0.153.4 and Claude Code 2.1.261; host delivery needs separate native verification. Restart the host and inspect `/hooks` after changes; the installer does not grant trust or enable disabled hooks. The installed copy carries its release in `VERSION`, and `--check-hooks` reports it, so you can tell which one a project has.

Codex requires trust at **two** levels, and a registration is inert until both are given. First the project: until it is trusted, Codex loads no project-local hooks at all and says so on stderr, naming the file it wants the entry in. That entry must live in the `config.toml` under `CODEX_HOME`; a `-c projects."<path>".trust_level="trusted"` override on the command line does **not** satisfy it, which is the failure that looks like hooks being delivered but ignored. Second, each hook definition is trusted by its own content hash, so changing a command asks again. Both are Codex's own mechanism; neither is granted by this installer. Claude Code has no equivalent second level.

Installation is additive. Standards alone registers shared SessionStart and PostToolUse, and is the only preset that runs anything; its PostToolUse hook carries a 60-second budget rather than 12, because a check is slower than a notice. Checkpoints alone registers shared SessionStart, PreToolUse and Stop, without compact handlers. Continuity alone registers shared SessionStart and manual/automatic PreCompact. All three together register six groups, with SessionStart exactly once. Removing one keeps the other usable. Bare `--remove-hooks` removes all owned presets and preserves the skill and session data. Repeating a selection is a byte-preserving no-op; duplicate/unknown names are rejected.

One aggregate receipt owns the original settings backup. Legacy continuity receipts remain readable and migrate only on an explicit state change; check, preview and unchanged reinstall do not rewrite them. Existing customized runtime dependencies are never overwritten: compare/back up the installed skill and explicitly update it first. Missing, changed or duplicate owned groups, unknown receipt schemas, and redirected/unverifiable paths cause refusal. New installations register commands that contain no machine-specific path, so a registration committed to a shared settings file works on a teammate's machine; each machine still reviews and trusts the hooks itself. An installation made by an older release keeps its absolute commands, stays inspectable and removable, and is rewritten only by `--upgrade-registration`, which changes every command and therefore requires trusting the hooks again. `--check-hooks` says whether a registration is portable, and names the recorded interpreter only for an absolute one.

Settings and receipt are published as separate atomic replacements under the shared lock, with an 8 MiB private transaction journal (config and receipt each at most 1 MiB). A retry reconciles only exact known before/after combinations. Unknown user edits are preserved and reported as conflicts. `--check-hooks` and dry-runs report pending recovery without writing; an explicit install/removal retry can complete it. Avoid concurrent settings edits: this is not an atomic transaction with an unrelated editor or host. Original bytes are restored only after the last preset is removed and remaining settings equal the original parsed settings. No old whole-config backup is restored over a surviving preset.

## Mapping hooks to platforms

This table maps the full target contract; it is not a configuration file. Continuity covers compact freshness, snapshot attempts, and recovery instructions. Checkpoints adds direct-commit advisory context and one explicit, non-continuing review-intent notice; it does not enforce H2/H7 evidence. Freeze/merge guards and enforced recovery remain future work. The [continuity interface](continuity.md) and [checkpoints interface](checkpoints.md) describe setup, tests, output audience/timing and coverage limits. Host protocol tests do not establish real lifecycle delivery; verify the installed registration in `/hooks`.

| Workflow behavior | Candidate Codex event | Candidate Claude Code event |
| --- | --- | --- |
| Freeze check before implementation; authorization check before merge | `PreToolUse` | `PreToolUse` |
| Checks before manual/automatic compact | `PreCompact`, distinguishing `manual` / `auto` | `PreCompact`, distinguishing `manual` / `auto` |
| Recover the current PR on compact/resume | `SessionStart`, matching `compact` / `resume` | `SessionStart`, matching `compact` / `resume` |
| Check handoff and state when execution ends | `Stop` or relevant ending events, adapting loop semantics | `Stop` or relevant ending events, adapting loop semantics |

These mappings come from [OpenAI Hooks](https://learn.chatgpt.com/docs/hooks) and the [Claude Code Hooks reference](https://code.claude.com/docs/en/hooks). Identical event names do not guarantee identical return protocols; do not copy blocking JSON between hosts without checking its meaning.

OpenAI documents tool paths that may bypass hooks, including subsequent input to an existing exec session that does not repeat the original command's pre-tool check. An adapter must inspect actual tool coverage and bypass routes. The presence of `PreToolUse` alone does not establish that every write and merge is guarded. See [OpenAI Hooks: tool coverage](https://learn.chatgpt.com/docs/hooks#tool-coverage).

Both platforms need separate behavior for blocking stale manual compact and allowing automatic compact with recovery. On resume, prioritize current PR identity, document paths, and recovery instructions. If long prompts cannot be injected in full, require complete file reads. A summary retained by the host after compaction is not proof that the complete execution template has been reloaded.

This package sets no global auto-approval, permission bypass, or continuous-execution switch, and does not depend on the `/goal` command mentioned in the historical source. Continuous execution and hook enforcement belong to host integration; task semantics and stop conditions are defined in the skill and behavior contract.

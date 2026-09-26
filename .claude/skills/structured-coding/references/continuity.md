# Optional continuity preset

This is the executable, opt-in subset of [the hook contract](hook-contract.md).
The default skill installation registers no hooks. The preset implements a
mechanical H4 freshness check, H5 snapshot attempts, and H6 recovery instructions.
It does **not** implement H1/H2/H3/H7, a mutation guard, semantic recovery proof,
remote PR-status checks, or merge protection. The separately selectable
[checkpoints preset](checkpoints.md) adds advisory H2/H7 reminders; merge protection
remains future work. A successful hook is not approval or test evidence.

## Installation and removal

From the cloned authoring repository:

```sh
./scripts/install codex --project /path/to/project --hooks continuity --dry-run
./scripts/install codex --project /path/to/project --hooks continuity
./scripts/install codex --project /path/to/project --check-hooks
./scripts/install codex --project /path/to/project --remove-hooks
```

Replace `codex` with `claude-code` for Claude Code. Use the exact Git worktree
root, quoting paths containing spaces. Python 3.9+, Git, macOS/Linux/WSL, and a
committed worktree on a named branch are required for runtime checkpoints.
The installer accepts Codex CLI 0.153.4+ or Claude Code 2.1.261+: these are
conservative protocol reference versions, not claims about the earliest release
with hooks. Newer versions still require a host smoke test.

The installer appends owned event groups to `.codex/hooks.json` or
`.claude/settings.json`, preserving existing groups and unrelated settings. It
does not enable disabled hooks, grant trust, change permissions, or touch global
configuration. Restart the host and inspect `/hooks`. For Codex, review/trust
the project layer and exact hook definitions yourself. Managed policy or other
configuration layers may disable hooks; `--check-hooks` cannot establish that
events are actually delivered.

An existing current skill can receive hooks without being recopied. An old or
customized continuity runtime must be compared/backed up and updated explicitly.
Reinstalling the same selection in the same command form is a no-op; an
installation made by an older release keeps its absolute commands until
`--upgrade-registration` rewrites them, which changes each command and so
requires reviewing and trusting the hooks again. `--hooks continuity checkpoints`
adds both presets; `--remove-hooks continuity` keeps checkpoints and the shared
SessionStart handler. Bare `--remove-hooks` removes all owned presets. `--remove-hooks --dry-run` previews
removal; removal retains the skill, PR bindings, snapshots, and checkpoints. It
restores the original settings bytes when nothing else changed, otherwise removes
only the exact owned groups. Changed owned groups cause a refusal, not deletion
of user edits. Local installation receipts contain a private original-settings
backup in the worktree Git directory. Do not publish these receipts.

Registered commands contain no machine-specific path. Claude Code expands
`${CLAUDE_PROJECT_DIR}`; Codex expands `$(git rev-parse --show-toplevel)`; the
interpreter is `python3` from `PATH`, which must be 3.9 or newer. A registration
committed to a shared settings file therefore works on a teammate's machine,
although each machine still reviews and trusts the hooks itself. Inside a linked
Git worktree the two hosts resolve differently: Codex reaches the worktree,
where the skill may not be installed, and Claude Code keeps the original project
root. Install into a linked worktree separately rather than relying on either.
Installation
uses a local lock and stale-config checks, but is not an atomic transaction with
an unrelated editor writing host settings at the same moment. Avoid concurrent
configuration edits. If hook registration fails after copying a new skill, the
inert skill remains installed and the error is reported. A private bounded journal
records interrupted config/receipt changes; `--check-hooks` and dry-runs report
pending recovery without writing. Retry an explicit install/removal to complete
only exact known before/after states. Unknown edits cause refusal. See the
[platform notes](platforms.md) for aggregate ownership and recovery limits.

## Binding a PR to one session

The `SessionStart` hook supplies the current `session_id` in a suggested binding
command. The agent performs binding/checkpoint maintenance; the user need not
copy session IDs at each PR. Only bind when executing a Structured Coding PR. Create the design,
filled contract, and semantic handoff first. For Codex, the helper is:

```sh
python3 .agents/skills/structured-coding/scripts/continuity.py activate \
  --host codex --project /path/to/project --session ACTUAL_SESSION_ID \
  --pr PR_01a --design docs/plan/pr-01a.md \
  --contract docs/plan/pr-01a-contract.md --handoff docs/plan/pr-01a-handoff.md \
  --base main
```

`--base` is optional and records the revision this PR's changes are measured
against, so a changed-file check can resolve its own range instead of being told.
It is recorded, not resolved: a revision that resolves at bind time can stop
resolving after a rebase, so it is looked up when a check uses it and any failure
is reported against that check. A binding without a base is ordinary; a check
needing one simply reports that it was not run.

For Claude Code use `.claude/skills/structured-coding/scripts/continuity.py` and
`--host claude-code`. All document paths are relative to the worktree root.
Binding records file identities, not their semantic content as a new authority.
It grants no execution or merge approval. There is no automatic fallback to
another session's PR. A new session or worktree requires its own explicit binding.
This isolates simultaneous sessions; it does not synchronize two agents editing
the same files.

After synchronizing the design and handoff with actual progress, evidence,
deviations, jobs/logs, checkpoint, and next actions, record mechanical freshness:

```sh
python3 .agents/skills/structured-coding/scripts/continuity.py checkpoint \
  --host codex --project /path/to/project --session ACTUAL_SESSION_ID
```

The command verifies current worktree/branch identity and hashes actual state.
It cannot verify that the agent's prose is true or that full-file reads happened.
Do not run it as a substitute for semantic synchronization or recovery. When
finishing/closing that binding, use `deactivate` with the same three options.
The hook does not query GitHub to discover that a PR was closed or merged.

## Event behavior

| Event | Registered behavior | Boundary |
| --- | --- | --- |
| Manual `PreCompact` | Compare current state to the session's explicit checkpoint; deny stale/missing checkpoints or unreadable bound state, with a repair message | Tests mechanical freshness, not handoff quality |
| Automatic `PreCompact` | Attempt a bounded mechanical snapshot; record `recovery_required` and warnings; return without blocking compact even on failure | Disk/host failures can prevent saving; stderr warns, and resume always requests recovery |
| `SessionStart` | For a bound session, inject its current PR and absolute document/rule paths with full-read and actual-state reconciliation instructions | No full prompt replay, mutation interception, process inspection, or claim that recovery was completed |
| `SessionStart`, either state | Name the installed `SKILL.md` by absolute path, so a session that did not plan this work is told where the phase resource list is | Names the entrypoint; does not deliver it, and an absent entrypoint is omitted rather than guessed. Reaching a session is not reading |
| Unbound or explicitly closed session | Unbound/closed compact proceeds; session start explains the absent/closed binding | Does not impose this workflow on unrelated work |

Codex manual denial uses `continue: false` with `stopReason`. Claude Code uses
`decision: "block"` with `reason`. Both recovery adapters use the documented
`SessionStart` `additionalContext` field. Handler mode is fixed in each installed
command, so malformed automatic-event input still cannot select the blocking
manual response. The installed host timeout is 12 seconds; the runtime snapshot
budget is 8 seconds, with bounded Git calls. A host that kills or skips the hook
may apply its own failure behavior: inspect that host's logs.

## Snapshot contents and limits

State lives under `git rev-parse --absolute-git-dir`, in
`structured-coding-continuity/<host>/<sha256(session_id)>/`. It is worktree-local,
outside the content fingerprint, and not shipped in the skill. Snapshots record
repository/branch/HEAD, an index-and-working-tree content fingerprint, and hashes
and paths for the design, contract, and handoff. Repeated automatic events replace
one `rescue.json`; they do not create duplicate jobs or summaries.

The fingerprint covers staged entries, tracked file contents/deletions/modes,
non-ignored untracked files, and explicit documents even if ignored by Git.
Symlink leaves are hashed as links, never followed. Symlink document/metadata
paths, submodules, special files, detached HEADs, and unborn branches are not
supported. Git-ignored files other than the explicit documents are excluded.
Snapshots are bounded to 10,000 files and 256 MiB of hashed content, and reject
detected concurrent changes. They are not a filesystem snapshot or an adversarial
security boundary. Large/busy worktrees may need manual synchronization after
automatic compact.

No file bodies, transcript text, environment dumps, credentials, or arbitrary
process contents are copied into rescue snapshots. The preset does not discover
job IDs itself: the semantic handoff must identify relevant jobs/logs, and the
recovering agent must inspect them before starting replacements. `rescue.json`
remains historical evidence of a compact; writing a checkpoint does not certify
semantic recovery or erase that evidence.

## Verification

From the authoring repository, run the installer, continuity, and package tests.
They exercise real temporary Git worktrees and subprocess hook commands with
documented event payloads. These tests are not evidence of lifecycle delivery
inside every Codex/Claude Code release.

In a disposable trusted project, check `/hooks`, bind a PR, and record a
checkpoint. Confirm manual compact passes; change a tracked or untracked file
and confirm manual compact is rejected until synchronized. Confirm compact/resume
shows the correct current PR and full-read instructions. Exercise automatic
compact separately and inspect the rescue state and failure warnings. Never
label the preset enabled solely because its JSON file exists.

Protocol sources: [OpenAI Hooks](https://learn.chatgpt.com/docs/hooks) and
[Claude Code Hooks](https://code.claude.com/docs/en/hooks), checked 2026-09-06.

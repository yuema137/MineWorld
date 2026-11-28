# Cognition as a launcher's child (`--supervised`)

Specification of the cognition process's supervised mode: the command-line flags a launcher uses, the status lines
it reads, the runtime manifest and preflight, and the store's layout and world-instance binding. Owner: the cognition
Controller Pack (`cognition/lm-controller`, module `mineworld_cognition.supervised` and
`mineworld_cognition.runtime_manifest`). The launcher's side — the profile that turns conversation on, the child's
lifecycle and the `--fresh` rule — is [`tools/launch/LAUNCHER.md`](../../tools/launch/LAUNCHER.md) §10. Design and
evidence: `.structured-coding/plans/mvp0/pr-playable-dialogue-memory.md` (DM-20261127-3) §3.4; the runtime route is
[`ARC-371`](../decisions/ARC-371.md). The configuration file itself is [`cognition-toml.md`](cognition-toml.md).

Without `--supervised`, `--preflight` and `--write-runtime-manifest` the command behaves exactly as before:
`python -m mineworld_cognition --config FILE`.

**LM-concept boundary.** Provider endpoint, model name, tokens and tiers stay inside the configuration file and the
gateway. Nothing in this mode puts one on the command line, in a status line, in the launcher's log or in a file the
launcher reads. The status line's `reason` is a closed set with no endpoint, model, token, invite or path.

## 1 Flags

| Flag | Value | Rule |
| --- | --- | --- |
| `--config FILE` | the operator's configuration | required, except with `--write-runtime-manifest` |
| `--supervised` | — | the launcher's child: §§2, 4 apply; the end of standard input stops the process as a first SIGINT does (`shutdown.Stopper.stop`); a stop requested before serving begins is honoured when it does |
| `--server-url URL` | `ws://127.0.0.1:<port>/ws` or `ws://[::1]:<port>/ws` | supervised only; replaces `server`; any other scheme, host, path or a missing port is `config_invalid` |
| `--store-dir DIR` | absolute | supervised only; the world's store (§4); replaces `store` |
| `--assignment FILE` | absolute | supervised only; `mineworld decisions <world> --json` (format 3); replaces `decisions_json` |
| `--preflight MANIFEST` | absolute | checks §3, prints one status line, exits |
| `--write-runtime-manifest PATH` | absolute | writes §3's manifest of the running interpreter, exits |

`--server-url`, `--store-dir` and `--assignment` without `--supervised` are refused (exit 2). With `--supervised` all
three are required. The replacements are typed fields applied after validation (`CognitionConfig.model_copy`), never
text: the configuration cannot override them. In supervised mode, too:

- the invite is read from `MINEWORLD_INVITE`, the variable the launcher sets; never a flag, never printed;
- `budgets.ledger` is `<store>/budget.sqlite` unless the configuration names a ledger (a gate's own ledger, design §6);
- every backend a tier is bound to must be on a loopback host (`backend_not_loopback`);
- the assignment must name exactly one `lm` seat (`assignment_invalid`), after the configuration's `seats` narrows it.

Exit status: `0` after a stop (or a passing preflight, or a written manifest); `2` for a command line the parser
refuses; `3` after a `degraded` status line. A crash that is none of these exits non-zero with no final line, and the
launcher names it `child_exited` or `preflight_failed`.

## 2 Status lines

In supervised and preflight modes the process's **standard output carries only status lines**. Everything else it
prints — the standalone command's counters line included — goes to standard error, which the launcher writes to the
run's cognition log. One JSON object per line, UTF-8, at most 512 bytes:

```json
{"status": 1, "state": "starting", "seat": "alice", "reason": null}
```

| Key | Value |
| --- | --- |
| `status` | the format, the integer `1`; any other value is malformed |
| `state` | `starting`, `ready`, `degraded` or `stopped` |
| `seat` | the `lm` seat's key, or `null` before the assignment is read |
| `reason` | `null` unless `state` is `degraded` or `stopped`; then exactly one of §2.3 |

No other key. The object is flat; its strings hold no escape (the vocabulary and seat keys need none). A line that is
not exactly this — another version, a missing or extra key, a reason with `starting` or `ready`, a reason the process
may not send, a nested value, an over-long line, a stray print — is malformed, and the launcher stops the process with
`degraded(status_invalid)`.

### 2.1 The lines a supervised run writes

```text
{"state": "starting", "seat": null}                 at once
{"state": "starting", "seat": "<lm seat>"}          the assignment has been read
{"state": "ready", "seat": "<lm seat>"}             the seat's welcome was received and its memory home is held
{"state": "stopped", "seat": …, "reason": "stopping"}   after the end of standard input (or a stop signal), last
{"state": "degraded", "seat": …, "reason": <§2.3>}      instead, when a named failure ends the run, last
```

`ready` is never sent for a spawned process, an open socket or a reachable endpoint alone: only `BoundHomes.hold`
sends it, after the home is held. After a hand-back (`took_over: "hosted"`) the seat rejoins with its home still held;
no new line is sent.

### 2.2 Preflight

`--preflight` writes one line: `{"state": "starting", "seat": null, "reason": null}` and exits `0` when every check of
§3 passes; else `{"state": "degraded", "seat": null, "reason": <reason>}` and exits `3`.

### 2.3 Reasons

Sent by the process:

| Reason | When |
| --- | --- |
| `config_invalid` | the configuration does not load or validate; a supervisor flag is malformed; `MINEWORLD_INVITE` is unset; a `ConfigError` ends serving (a seat the world does not have, a scripted mode with no backend in code) |
| `assignment_invalid` | the assignment is unreadable, not format 3, or names other than exactly one `lm` seat |
| `runtime_mismatch` | the manifest is unreadable or differs from the running interpreter's (§3), or an entry-point group does not resolve |
| `backend_not_loopback` | a bound backend's host is not a loopback address |
| `home_busy` | another process holds the world's memory home (one writer, `DEP-61`) |
| `home_identity_mismatch` | the home is another instance's or another self's, of another schema, or not a home |
| `instance_mismatch` | the store is bound to another world instance (§4) |
| `server_unreachable` | the seat ended for good because the world stopped or reconnecting gave up |
| `seat_unavailable` | the seat ended for good for any other reason (unknown, not granted, protocol) |
| `model_unavailable` | preflight: a bound OpenAI-compatible local service did not answer `GET <base_url>/models` within 3 s, or does not list the configured model |
| `stopping` | with `stopped`: the process stopped because it was asked to |

The launcher's own, never accepted from the process: `readiness_timeout`, `child_exited`, `status_invalid`,
`preflight_failed`, `profile_invalid` (`LAUNCHER.md` §10.5).

## 3 Runtime manifest and preflight

The R1 runtime is a MineWorld source checkout's `uv sync --locked` environment on this machine (`ARC-371`). Its
manifest is written by that environment's interpreter:

```sh
<checkout>/.venv/bin/python -m mineworld_cognition --write-runtime-manifest <absolute path>
```

```json
{"manifest": 1, "python": "3.12.7", "source_revision": "<40-hex git sha or null>",
 "lock_sha256": "<sha256 of the checkout's uv.lock>",
 "distributions": [{"name": "mineworld-cognition", "version": "0.1.0",
                    "files": [{"path": "mineworld_cognition/gateway.py", "sha256": "<hex>"}]},
                   {"name": "pydantic", "version": "2.13.1", "files": []}],
 "entry_point_groups": ["mineworld.forgetting", "mineworld.retry", "mineworld.breaker", "mineworld.memory_scopes"]}
```

- The checkout is the nearest folder above the imported `mineworld_cognition` that holds `uv.lock`; with none, the
  manifest is not written (the R1 runtime is a checkout's environment).
- `source_revision` is read from the checkout's Git metadata without running Git (worktrees and packed refs included);
  `null` when the folder is not a Git checkout.
- `mineworld-cognition` and `mineworld-sdk` list every `*.py` file of their import package as imported (an editable
  install resolves to the checkout's real files), with its SHA-256. Every other installed distribution is listed by
  name and version with no files; when it is in `uv.lock`, its version must equal the lock's, or the manifest is not
  written ("run `uv sync --locked`").
- The file is written whole (a `.partial` renamed).

`--preflight MANIFEST`, run by the launcher before the server starts, in this order:

1. the configuration loads (`config_invalid`);
2. the manifest is read and equals, field by field and distribution by distribution, the manifest the running
   interpreter derives now (`runtime_mismatch`); so a checkout updated since (`git pull`, `uv sync`, an edited source
   file) fails until the manifest is written again;
3. every entry-point group above publishes at least one entry and each entry loads (`runtime_mismatch`);
4. every bound backend is on a loopback host (`backend_not_loopback`);
5. in `live` and `record` modes, each bound OpenAI-compatible backend lists the configured model at
   `GET <base_url>/models` (`model_unavailable`). This is a listing on loopback: no inference, no download, no start.
   `replay` reaches no backend and skips it.

## 4 Store layout and world-instance binding

```text
<store>/                         the launcher's cognition/<world>/ (LAUNCHER.md §10.3)
  instance                       the world instance this store belongs to (text, one line)
  <instance>/memory.sqlite       the memory home (`controller/homes.py`, `[memory] sharing = "home"`)
  <seat>.resume                  the seat's resume secret (§5.10 of the P6 design)
  <seat>.decisions.jsonl         the seat's decision log
  budget.sqlite                  the budget ledger, unless the configuration names one
  run/assignment.json            written by the launcher for each run
```

The launcher owns the directory's life: it creates it, and `--fresh` deletes it with the world's save. Cognition owns
the binding: at the first welcome of a run, before any memory home is opened (`supervised.BoundHomes.hold`):

- no `instance` file: it is written (whole, a `.partial` renamed) with the welcomed instance;
- an `instance` file naming the welcomed instance: nothing changes;
- an `instance` file naming another instance: the seat is refused with `InstanceMismatch`, the run ends
  `degraded(instance_mismatch)`, and no home, no instance folder and no binding is written. The decision log may receive
  the seat's closing counters line, as for any refused seat.

There are never two writers: a second process on the same home is refused by the home's own lock (`home_busy`).

**Restore caveat.** MineWorld has no backup or restore feature. Restoring an older copy of a save by hand while keeping
a newer store can leave the store ahead of the world; that is unsupported. A save of another world instance is refused
as above and never silently erased; a same-instance rollback is not detected.

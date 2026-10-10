# PR S10-P5b — The native Anthropic adapter, and subscription access through the user's own CLI

## DESIGN FROZEN 2026-10-09 (primary session)

```text
Design revision:        revision 1 (2026-10-09), with §10.1's rulings, as committed on plan/s10-p5
                        (PR #111) with this header
Approved by / evidence: the primary session's freeze message of 2026-10-09, relayed by the coordinator;
                        the operator's rulings on QP5b-1 (drop the Claude subscription route) and
                        QP5b-2 (Codex as an opt-in for the user's own local use) of the same day
Implementation base:    main after P5a has merged (exact commit recorded in C0)
Execution contract:     §9 (filled at freeze)
Lifecycle:              FROZEN. Implementation starts only after P5a has merged
```

**What the rulings change in this design.** Each change is applied below and marked "ruled".

- **QP5b-1, operator: the Claude Code subscription route is dropped.** `presets/claude_code.py`,
  `kind = "claude-code-subscription"` and C5 leave the scope. Claude is supported through the user's own
  Anthropic API key with `AnthropicMessagesBackend`. The reason is recorded in §4.1 with Anthropic's
  terms quoted.
- **QP5b-2, operator: Codex through the user's ChatGPT plan is an opt-in for the user's own local use
  only.** It is off by default, never in an example, gated by `acknowledge_terms`, and documented in
  the README after every API-key route.
- **Open obligation (the operator's condition on QP5b-2).** OpenAI's Terms of Use must be re-read before
  the Codex preset is built. The quote in §4.2 came from search excerpts because the page answered HTTP
  403. A second attempt at freeze time (2026-10-09, `openai.com/en-GB/policies/terms-of-use/`) also
  answered 403. The obligation therefore moves to **C1**. The operator, or a session with a browser,
  reads the page and pastes the effective date and the relevant clauses into §4.2. **C4 (the Codex
  preset) does not start until that is recorded.** If the re-read shows that the terms forbid the use,
  C4 becomes N/A and the route is dropped as Claude's was. That would be a material stop, returned to
  the operator.
- **Decision numbers:** `DEP-33` is the native Anthropic adapter, with the SDK declined (QP5b-4).
  `ARC-60` is the subscription route as it remains: the Codex opt-in, its gate, and I-B1 … I-B5. S10
  also holds `ARC-72 … ARC-74` for later PRs (P4, P6).

**Effort:** `mvp0` · **Step:** S10, [`step-17-cognition.md`](step-17-cognition.md) §3.7 · **Depends on:**
P5a, [`pr-s10-p5-backends.md`](pr-s10-p5-backends.md) (the interface, the gateway, budgets, recorder,
`Secret`, `resolve_key`, the configuration). P5b changes none of P5a's seams (R-P5b-1 there).
**Planning base:** `origin/main @ aee8290`, branch `plan/s10-p5`.
**Decision identifiers:**

- `DEP-33` (ruled 2026-10-09): the Anthropic adapter over `httpx2`, with the Anthropic SDK and the
  Claude Agent SDK declined.
- `ARC-60` (ruled 2026-10-09, QP5b-3): the subscription route as it remains after QP5b-1, that is, the
  Codex opt-in and its terms gate. It records why the Claude route was dropped.

### The requirement, verbatim (operator, 2026-10-09)

> 我们还需要提供api和订阅接口，这样才能支持非本地模型，比如gpt claude glm deepseek等……api通过.env
> 之类的配置，subscription就用codex或者claude code本身的authorize验证

MineWorld must support non-local models (GPT, Claude, GLM, DeepSeek and others) through two routes:
API keys configured through a `.env`-style file, and subscriptions authorized by Codex's or Claude
Code's own login. P5a delivers the API-key route for every OpenAI-compatible service. P5b delivers the
native Claude adapter and the subscription route.

**Unchanged and binding (QS10-19):**

- no MVP-0 gate depends on a paid API or a subscription;
- CI runs scripted backends and cassettes only;
- live tests are operator-run and opt-in;
- agents never read keys, never call a hosted model, and never run a model CLI with a real login.

---

## 1. Goal

A user can bind a tier to Claude through their own Anthropic API key, with schema-constrained output,
which Anthropic's OpenAI-compatibility layer cannot give. Where the vendor's terms allow it, as ruled
by the operator, a user can also bind a tier to their **own** installed and logged-in model CLI. That
route runs headlessly as that user. MineWorld never sees, stores or forwards the CLI's credentials.
Both routes are ordinary `ModelBackend`s behind P5a's gateway, so budgets, cassettes, the provider-neutral
key and the secret rules apply unchanged.

## 2. Scope

### 2.1 In scope

- `backend/anthropic_messages.py`: `AnthropicMessagesBackend` over `httpx2`, `kind = "anthropic"`.
- `backend/cli_bridge.py`: `CliBridgeBackend`, a generic runner for a model CLI. It handles discovery,
  argv built from fixed literals, the prompt on stdin, an environment allowlist, an empty working
  directory, timeouts that kill the process tree, and output parsing per preset.
- `backend/presets/codex.py`: the `codex exec` preset (`kind = "codex-subscription"`). Ruled in by
  QP5b-2 as an opt-in, and built only after the C1 re-read of OpenAI's terms.
- ~~`backend/presets/claude_code.py`~~: **dropped by QP5b-1 (operator, 2026-10-09).** No
  `claude-code-subscription` kind exists; `config.load` reports it as unknown.
- `config.py`:
  - the `kind` union gains `anthropic` and `codex-subscription`;
  - the `acknowledge_terms` field;
  - a refusal of `kind = "openai-compatible"` pointed at `api.anthropic.com` (AB-2).
- Tests with **fake CLIs**: small Python scripts started as `[sys.executable, fake.py]`, and on Windows
  also through a `.cmd` shim. No real CLI runs in a test or in CI.
- `examples/hosted.toml.example`: a Claude-by-API-key example.
- `README.md`: a "Subscriptions" section stating the terms evidence, the opt-in, and that API keys
  are the recommended route.
- `docs/DECISIONS.md` `DEP-33` and `ARC-60`.

### 2.2 Not in scope

| Not here | Why |
| --- | --- |
| Reading, refreshing or copying any CLI credential (`~/.codex/auth.json`, the OS keychain, `~/.claude`) | Prohibited by design (I-B1) and, for Claude, by Anthropic's terms ("developers may not collect, store, or intermediate Claude.ai credentials or session tokens") |
| Installing, bundling or updating a CLI | The user installs their own. Bundling the Agent SDK would ship Claude Code itself (§4.3) |
| Any agent or CI run against a real CLI, key or hosted API | QS10-19, §9 NEVER |
| Tool use, files or MCP in a bridged CLI | A bridge asks one question and reads one answer; every agentic capability is disabled (D-B5) |
| Prices or plan limits | Provider concepts. Calls and tokens only (P5a D-P5-8) |

### 2.3 Invariants

```text
I-10   (P5a) no provider concept outside the adapters, the presets, the registry and config.py
I-16   (P5a) no secret in any artefact; a key file's values never enter os.environ
I-B1   MineWorld never reads, writes, copies or forwards a CLI's credentials. A bridge runs the user's
       own CLI, which authenticates itself
I-B2   a bridged CLI receives no MineWorld secret: its environment is an allowlist that holds no
       *_API_KEY, no *_TOKEN and nothing from the env_file
I-B3   no model-facing text in argv: the prompt goes through stdin, the schema through a temporary file;
       argv holds only fixed literals and validated configuration values
I-B4   a bridged call is bounded: the timeout kills the whole process tree on every platform
I-B5   a subscription backend exists only when its preset was ruled in AND the user set
       acknowledge_terms; replay and scripted modes never construct it
```

---

## 3. Audit anchors

| Anchor | What it establishes |
| --- | --- |
| P5a design: D-P5-2, D-P5-4, D-P5-5, D-P5-8, D-P5-9, D-P5-10, §5.3 | The interface, the no-SDK rule, lazy construction, budgets, keys and the gateway sequence that P5b plugs into |
| `cognition/lm-controller/src/mineworld_cognition/backend/registry.py` (P5a; not yet on main) | Adapters are added by `kind`; C0 re-audits P5a's merged code before C1 |
| `.gitignore` (after P5a) | `.env`, `*.env`, `.env.*`, `secrets*` |
| Claude Code headless documentation (`code.claude.com/docs/en/headless`, read 2026-10-09) | `claude -p … --output-format json --json-schema '<schema>'`, with the result in `structured_output`. "`--bare` … Set `ANTHROPIC_API_KEY` … because bare mode doesn't use your subscription login." "Without `--bare`, a `-p` session runs the hooks in a project's `.claude/settings.json` and connects the servers in its `.mcp.json`, even in a folder you've never trusted." |
| Codex authentication documentation (`learn.chatgpt.com/docs/auth`, the redirect target of `developers.openai.com/codex/auth`, read 2026-10-09) | ChatGPT sign-in or an API key. "Codex caches login details locally in a plaintext file at `~/.codex/auth.json` or in your OS-specific credential store." |
| Codex non-interactive mode (search excerpts of `developers.openai.com/codex/noninteractive`, 2026-10-09; the page itself was not fetched) | `codex exec` takes the prompt as an argument or on stdin; `--json` streams JSON Lines events including `turn.completed` with token usage; `--output-schema FILE`; a read-only sandbox by default; `--skip-git-repo-check` outside a Git repository. **Re-verified in C1 against the installed version's `--help`, by the operator** (an agent does not run the CLI) |
| `openai/codex` (GitHub API) | Apache-2.0, active (pushed 2026-10-09) |
| Anthropic structured outputs (`platform.claude.com/docs/en/build-with-claude/structured-outputs`, 2026-10-09) | `output_config.format = {"type": "json_schema", "schema": …}`; no beta header; `additionalProperties: false` required; `minLength`, `maxLength`, `minimum`, `maximum`, `multipleOf` unsupported (400) |
| Anthropic OpenAI-compatibility (`platform.claude.com/docs/en/api/openai-sdk`, 2026-10-09) | `response_format` "Ignored"; `seed` "Ignored"; `temperature` below 1 is "a 400 error" on Claude 4.7 and later; "not considered a long-term or production-ready solution for most use cases" |

---

## 4. Terms of service: the evidence, read 2026-10-09

The pages carry no revision date unless one is given. Every quote is verbatim from the page named.

### 4.1 Anthropic: Claude Code with a consumer subscription

1. **Claude Code, "Legal and compliance"**, <https://code.claude.com/docs/en/legal-and-compliance>,
   section "Authentication and credential use":
   > **OAuth authentication** is intended exclusively for purchasers of Claude Free, Pro, Max, Team,
   > and Enterprise subscription plans and is designed to support ordinary use of Claude Code and other
   > native Anthropic applications.

   > **Developers** building products or services that interact with Claude's capabilities, including
   > those using the Agent SDK, should use API key authentication through Claude Console or a supported
   > cloud provider. Anthropic does not permit third-party developers to offer Claude.ai login into
   > their own applications, or to route requests through Free, Pro, or Max plan credentials on behalf
   > of their users. Moreover, developers may not collect, store, or intermediate Claude.ai credentials
   > or session tokens — sign-in to a Claude account must complete through Anthropic's own flow.

   > Nor does it prevent an end user from signing in to the unmodified Claude Code binary with their
   > own Claude subscription, including where a platform hosts Claude Code as described under *Can
   > customers offer Claude Code in their products?* above.

   > Anthropic reserves the right to take measures to enforce these restrictions and may do so without
   > prior notice.

   And, under "Acceptable use":
   > Advertised usage limits for Pro and Max plans assume ordinary, individual usage of Claude Code and
   > the Agent SDK.
2. **Agent SDK overview**, <https://code.claude.com/docs/en/agent-sdk/overview>:
   > Unless previously approved, Anthropic does not allow third party developers to offer claude.ai
   > login or rate limits for their products, including agents built on the Claude Agent SDK. Use the
   > API key authentication methods described in the Quickstart instead.
3. **Consumer Terms of Service**, <https://www.anthropic.com/legal/consumer-terms>, "Effective October
   8, 2025", among the things a user may not do:
   > Except when you are accessing our Services via an Anthropic API Key or where we otherwise
   > explicitly permit it, to access the Services through automated or non-human means, whether through
   > a bot, script, or otherwise.
4. **Claude Code headless**, <https://code.claude.com/docs/en/headless>: the scripted mode Anthropic
   recommends, `--bare`, "doesn't use your subscription login", and "In bare mode, Claude Code never
   reads OAuth credentials or the system keychain."

**Reading.** A MineWorld backend that runs `claude -p` to write NPC dialogue is a third-party product
that routes model requests through the user's Free, Pro or Max credentials. Anthropic says it does "not
permit" exactly that. The carve-out for "an end user signing in to the unmodified Claude Code binary"
covers using Claude Code itself. It does not cover using Claude Code as the model endpoint of another
product. The usage-limit note ("ordinary, individual usage") and the consumer terms' "automated or
non-human means" clause point the same way. Anthropic may enforce "without prior notice", and the
account at risk is the user's. The route that is explicitly allowed is **Claude by API key**, which
P5b's native adapter provides.

**Ruled (QP5b-1, operator, 2026-10-09): the Claude Code subscription route is dropped.**

- **The reason**, in Anthropic's words quoted above: "Anthropic does not permit third-party developers
  to offer Claude.ai login into their own applications, or to route requests through Free, Pro, or Max
  plan credentials on behalf of their users".
- **Enforcement:** Anthropic "may do so without prior notice", against the user's account.
- **What remains:** Claude is supported through the user's own Anthropic API key with
  `AnthropicMessagesBackend` (the `anthropic` kind), which Anthropic directs developers to use: "should
  use API key authentication through Claude Console or a supported cloud provider".
- **Recorded in:** `ARC-60` and the README's "Subscriptions" section.

### 4.2 OpenAI: Codex CLI with a ChatGPT subscription

1. **Codex authentication**, <https://learn.chatgpt.com/docs/auth>, the redirect target of
   <https://developers.openai.com/codex/auth>. No date on the page.
   > Sign in with ChatGPT for subscription access

   > Use API key authentication for programmatic Codex CLI workflows, such as CI/CD jobs.

   > API keys are still the recommended default for automation.

   > Access tokens are intended for trusted scripts, schedulers, and private CI runners.

   > Treat `~/.codex/auth.json` like a password.

   > Don't expose Codex execution in untrusted or public environments.
2. **OpenAI Terms of Use**, <https://openai.com/policies/terms-of-use/>. Search excerpts show
   "Effective: January 1, 2026"; the page answered our fetcher with HTTP 403, so the quote is from the
   excerpts and is re-read by the operator before freeze. Among the things a user may not do:
   > Automatically or programmatically extract data or Output (defined below).

   **C1 re-read (P5b implementation session, 2026-10-09): INCONCLUSIVE.** WebFetch of
   `https://openai.com/policies/terms-of-use/` and of `https://openai.com/policies/row-terms-of-use/`
   both answered HTTP 403. The quote above therefore still rests on search excerpts; nothing was
   guessed. C4 does not start (freeze header). Pending: the operator, or a session with a browser,
   pastes the effective date and the clauses here.
3. **Not found:** any OpenAI text that explicitly permits, or explicitly forbids, a third-party
   application running `codex exec` with the user's ChatGPT plan.

**Reading.** This is unclear rather than forbidden. `codex exec` is OpenAI's own documented
non-interactive mode, and the user runs it as themselves. But OpenAI recommends API keys "for
automation". "Access tokens are intended for trusted scripts … and private CI runners." The consumer
terms bar programmatic extraction of output. A game's NPC driven by `codex exec` is automation in
plain words.

**Ruled (QP5b-2, operator, 2026-10-09): an opt-in for the user's own local use only.**

- It is off by default and never in an example.
- It is gated by `acknowledge_terms`.
- It is documented in the README **after** every API-key route, stating that OpenAI recommends API keys
  for automation and that the user is responsible for their plan's terms.
- OpenAI models by API key (P5a's OpenAI-compatible adapter) remain the recommended route.
- **Condition:** the Terms of Use quote above is re-read from the live page before the preset is built
  (C1 → C4; see the freeze header).

### 4.3 Libraries considered for the two routes

| Candidate | Licence (verified) | Verdict |
| --- | --- | --- |
| **`anthropic` SDK** | MIT; 1.13.0; depends on `httpx2`, `anyio`, `docstring-parser`, `jiter`, `pydantic`, `sniffio`, `typing-extensions` | **REJECT** (`DEP-33`), for QP5-1's reasons. It reads `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL` when an argument is omitted. **This behaviour is not re-verified this session; C1 reads `_client.py` and records it.** Its retries hide calls from the ledger. The Messages request P5b needs is one POST over the `httpx2` we already depend on. QP5-1's ruling asks for this comparison, and it finds no reason to take the SDK |
| **`claude-agent-sdk`** | MIT; 0.2.165. "The Claude Code CLI is automatically bundled with the package" | **REJECT.** Depending on it would ship Claude Code inside MineWorld, which needs Anthropic's Commercial Terms ("preinstalling or running Claude Code in your products … requires agreeing to our Commercial Terms of Service"). Its documentation forbids exactly the subscription login this route would use (§4.1 (2)) |
| **Anthropic's OpenAI-compatibility layer** through P5a's adapter | — | **REJECT** for Claude: `response_format` and `seed` are ignored, and a temperature below 1 is refused on recent models (§3). `config.load` refuses `openai-compatible` at `api.anthropic.com` and names the `anthropic` kind (AB-2) |
| **An OpenAI or Codex SDK for the bridge** | — | **REJECT.** The bridge's contract is the CLI's documented command line, not a library. It works with whatever version the user installed |
| **Our adapter over `httpx2`, and a subprocess bridge on the standard library's `asyncio.create_subprocess_exec`** | — | **CHOSEN**: no new dependency |

---

## 5. Design

### 5.1 `AnthropicMessagesBackend` (`kind = "anthropic"`)

- **Request.** `POST {base_url}/v1/messages`, where `base_url` defaults to nothing and must be given:
  `https://api.anthropic.com` in the example. Headers: `x-api-key` from `resolve_key(key_env, …)`, and
  `anthropic-version: 2023-06-01`. There is no `Authorization` header.
- **Mapping from `CompletionRequest`:**
  - `system` messages are joined with `"\n"` into the top-level `system`;
  - `user` and `assistant` messages map in order;
  - `max_tokens` = `max_output_tokens`;
  - `temperature` is sent only when `temperature = "send"`, and the default for this kind is `"omit"`;
  - `seed` is not sent (the API has none);
  - `output_config.format = {"type": "json_schema", "schema": lowered(output_schema)}` when a schema is
    present and `structured_output = "json_schema"`.
- **Schema lowering** is a pure, deterministic function inside the adapter:
  - it removes the keywords Anthropic refuses (`minLength`, `maxLength`, `minimum`, `maximum`,
    `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf`, `minItems` above 1, `maxItems`);
  - it sets `additionalProperties: false` on every object.

  The cassette key is computed on the **unlowered** request (P5a D-P5-6), so it is unchanged. P6's
  local Pydantic validation still enforces every removed bound.
- **Response.**
  - Text blocks are concatenated.
  - `stop_reason`: `end_turn` and `stop_sequence` → `complete`; `max_tokens` → `length`; `refusal` →
    `refused`; anything else → `malformed_response`.
  - `usage.input_tokens` and `usage.output_tokens` map directly.
  - HTTP 401 and 403 → `unauthorized`; 429, 500 and 529 → `http_status(code)`.
  - No retry.

### 5.2 `CliBridgeBackend`: running the user's own CLI

```text
complete(request):
  1. workdir = a fresh empty temporary directory (tempfile.mkdtemp); removed afterwards
  2. if a schema: write it to workdir/schema.json (UTF-8, LF); the preset passes the PATH, not the text
  3. argv = preset.argv(config, schema_path)   # fixed literals + validated config values only (I-B3)
  4. env  = allowlist(os.environ)              # I-B2; see D-B3
  5. proc = asyncio.create_subprocess_exec(*argv, cwd=workdir, env=env, stdin=PIPE, stdout=PIPE,
                                           stderr=PIPE, **platform_group_flags)
  6. write preset.render_prompt(request) to stdin as UTF-8; close stdin
  7. read stdout under the gateway's timeout; on timeout → kill_tree(proc) (D-B4) → Failed(timeout)
  8. exit code ≠ 0 → BackendFailure(unreachable | unauthorized | http_status) by preset.classify(stderr
     first line, exit code). The message carries the first 200 characters of stderr only after the
     secret scrub of P5a, and never stdout
  9. preset.parse(stdout) → Completion(text, finish, usage | estimated)
```

### 5.3 Decisions

| ID | Decision | Reason |
| --- | --- | --- |
| **D-B1** | **Discovery.** `command: list[str] \| None` in the backend's configuration. When absent, `shutil.which("codex")` (or `"claude"`) is used, which honours `PATHEXT` on Windows and so finds `codex.exe` or the npm shim `codex.cmd`. A missing executable is `BackendFailure("unreachable")` naming the command looked for and suggesting `command = [...]`. There is no fallback search of install directories. | Explicit, platform-correct, and testable with a fake (`command = [sys.executable, "fake_codex.py"]`). |
| **D-B2** | **Prompt by stdin, schema by file path, nothing model-facing in argv** (I-B3). argv holds only preset literals, the temporary schema path (generated, ASCII), and `model`, validated against `^[A-Za-z0-9._:/-]{1,128}$`. | On Windows a `.cmd` shim is run through `cmd.exe`, whose argument parsing is the source of the "BatBadBut" class of injection (CVE-2024-24576 and relatives). Player speech reaches the prompt, so the prompt must never reach argv. |
| **D-B3** | **An environment allowlist.** The child receives only: `PATH`, `PATHEXT`, `SYSTEMROOT`, `COMSPEC`, `WINDIR`, `HOME`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, `TEMP`, `TMP`, `TMPDIR`, `LANG`, `LC_ALL`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`, and the preset's home variable (`CODEX_HOME`; `CLAUDE_CONFIG_DIR`), each only if set. Every `*_API_KEY`, `*_TOKEN`, `OPENAI_*`, `ANTHROPIC_*` and every `env_file` value is absent. | I-B2. It also keeps the route honest: with `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` passed through, the CLI would silently bill an API key instead of the user's subscription. |
| **D-B4** | **Kill the tree.** POSIX: `start_new_session=True`, then `os.killpg(pgid, SIGKILL)`. Windows: `CREATE_NEW_PROCESS_GROUP`, then `taskkill /T /F /PID <pid>` (a system binary, argv only). `Process.kill()` alone is not enough: a `.cmd` shim or a Node launcher leaves its child running. | I-B4. Claude Code's own documentation notes that SIGTERM leaves turns unfinished. A bridge needs a hard bound. |
| **D-B5** | **No agentic capability.** Each preset passes the CLI's own switches for a single, tool-less answer: a read-only sandbox, no tools, no MCP, one turn. It runs in an empty temporary directory, so no project file (`CLAUDE.md`, `.mcp.json`, `.claude/settings.json`, `AGENTS.md`) can be picked up. The exact switches are confirmed against the CLI's `--help` **by the operator** in C1, never by an agent running it. | The documentation shows that `claude -p` without `--bare` "runs the hooks in a project's `.claude/settings.json` and connects the servers in its `.mcp.json`". An empty working directory removes the project half; the user's own global configuration remains theirs. |
| **D-B6** | **The gate (I-B5).** A subscription kind is accepted by `config.load` only if its preset was ruled in (QP5b-1, QP5b-2) **and** the backend sets `acknowledge_terms = "<kind>"`. Otherwise `ConfigError` names the README section with §4's evidence. `replay` and `scripted` modes never construct it (P5a D-P5-5). No shipped example enables one. | The operator's ToS instruction: opt-in for the user's own local use, documented but not promoted. |
| **D-B7** | **The same budgets and cassettes.** A bridged call passes P5a's gateway like any other. The budget pre-check happens **before** the process is spawned. Tokens come from the CLI's usage report (Codex's `turn.completed`; Claude Code's JSON `usage`), else they are estimated. Cassettes record the binding name and model as metadata, never the command line. | The operator: "subscription calls count against the same wall-time budgets". `AC-4`: a cassette recorded through a bridge replays under any backend. |
| **D-B8** | **Timeouts.** A bridge's per-call bound is `request_timeout_s`, defaulting to the gateway's `call_timeout_s` (20 s). CLI start-up cost is not measured here; the operator's spike (P5a C7) may measure a bridge if they wish. | No agent can measure it (§9 NEVER). |

### 5.4 The presets

**Codex (`kind = "codex-subscription"`, conditional on QP5b-2).**

- argv: `codex exec --skip-git-repo-check --sandbox read-only --json --output-schema <schema_path>
  [-m <model>] -`. The trailing `-` reads the prompt from stdin, *as documented*; the exact form is
  confirmed in C1.
- The prompt renders `CompletionRequest.messages` as role-labelled sections. This rendering is the
  preset's, and the key is computed before it.
- Output: JSON Lines events. The final agent message is the completion text; `turn.completed` carries
  usage; `turn.failed` maps to `BackendFailure`.
- `--output-schema` requires strict schemas: the same lowering as §5.1 sets `additionalProperties:
  false`.

**Claude Code (`kind = "claude-code-subscription"`): DROPPED by QP5b-1 (operator, 2026-10-09).** It is
not built. The sketch below is kept only as the record of what was considered:

- argv: `claude -p --output-format json --json-schema-file <schema_path>`, *or* `--json-schema` with
  the schema text if no file form exists. Schema text is generated by us and contains no player
  speech; C1 records which form exists. Then `--max-turns 1` and the tool-disabling switch, and never
  `--bare`, because bare mode does not use the subscription.
- Output: `structured_output` and `usage` from the JSON result.

---

## 6. Adversarial criteria (fixed now, before anything is measured)

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| **AB-1** Anthropic mapping | Over a mock transport: the exact body for one literal request. `system` is hoisted; there is no `temperature` by default and no `seed`; `output_config.format` carries the lowered schema with `maxLength` removed and `additionalProperties: false` set. The header `x-api-key` is present and `Authorization` absent. The cassette key equals the key of the unlowered request. Response mapping for `end_turn`, `max_tokens` and `refusal`, and for 401, 429 and 529. One call per `complete` (no retry). | Compute the key after lowering: the key differs from P5a's literal. Send `temperature` by default: the body differs. |
| **AB-2** The compatibility layer is not used for Claude | `kind = "openai-compatible"` with a `base_url` on `api.anthropic.com` raises `ConfigError` naming `kind = "anthropic"` and the reason (`response_format` ignored). | Remove the check: the configuration loads. |
| **AB-3** No secret reaches a CLI | Fake CLI `fake_cli.py` writes its environment, argv and working directory to a file the test names. The test plants `MWTEST-…` in `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `CODEX_API_KEY`, an unrelated `FOO_TOKEN`, and in the `env_file`. None appears in the child's environment, argv or stdin. The working directory is an empty temporary directory, removed after the call. | Pass `os.environ` through: the planted values appear. |
| **AB-4** No model-facing text in argv | A request whose user message is `"& calc.exe %PATH% $(id) \"; rm -rf /` plus 10 KB of text. The fake CLI's argv contains none of it, and its stdin contains all of it byte for byte. On the Windows CI leg the same test runs through a `fake_codex.cmd` shim found by `shutil.which` (PATHEXT). | Put the prompt in argv: the scan finds it, and on Windows the shim mangles it. |
| **AB-5** The timeout kills the tree | The fake CLI starts a grandchild that sleeps 60 s, then sleeps itself. With a 1 s timeout, `Failed(timeout)` arrives within 2 s, and **both** process ids are gone within 3 s, on Linux, macOS and Windows. | Use `proc.kill()` only: the grandchild survives, and the test names its pid. |
| **AB-6** MineWorld never touches CLI credentials | A source scan of `cognition/lm-controller/src` finds no `auth.json`, `.codex`, `.claude`, `.credentials`, `keychain`, `keyring` or `CLAUDE_CODE_OAUTH`. The fake CLI's recorded environment holds no credential path variable other than the allowlisted home variables. | Plant a read of `~/.codex/auth.json` in `codex.py`: the scan fails at that line. |
| **AB-7** The gate | A subscription kind without `acknowledge_terms` raises `ConfigError` pointing to the README section. A kind not ruled in is unknown to the registry (`ConfigError: unknown kind`). In `replay` mode a configuration naming a subscription backend constructs no bridge: the fake CLI's spawn counter is 0. A missing executable gives `BackendFailure("unreachable")` naming the command. | Construct backends eagerly: the spawn counter is non-zero in replay. |
| **AB-8** Budgets bound the bridge | With the per-seat ceiling at 20 calls per wall hour (FakeClock), the 21st call is refused, and the fake CLI's spawn counter stays at 20. The usage from the fake's `turn.completed` is charged; a fake without usage is charged the estimate, flagged as estimated. | Check the budget after spawning: the counter reaches 21. |
| **AB-9** Every platform | AB-1 … AB-8 pass on `ubuntu-24.04`, `windows-2025` and `macos-15` in the PR's CI. AB-4's `.cmd` path is Windows-only, and is shown to have run there by name in the CI log. | Hard-code `os.killpg`: Windows fails on AB-5. |
| **AB-10** Scope and isolation | The diff touches only `cognition/lm-controller/**`, `docs/DECISIONS.md`, `.structured-coding/**`, and the README and examples. No CI command names a real CLI, key or hosted URL. `ci_layer.py` is unchanged. | — (a diff gate) |

---

## 7. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-B1 | A vendor changes its terms, or enforces against users of the bridge. | Opt-in only, documented with dated evidence (§4); no example enables it; API keys are the recommended route. The README tells the user to check the current terms. A terms change is a reason to remove a preset, which is one file plus one registry line. |
| R-B2 | CLI flags change between versions. | The preset is one file. A changed flag fails with the CLI's own error, classified `malformed_response` or `unreachable` and naming the command. C1 records the version the operator checked. |
| R-B3 | The user's global CLI configuration (hooks, MCP servers in `~/.claude` or `~/.codex`) runs during a bridged call. | It is the user's own, run as them. The empty working directory removes project configuration. The README says so. MineWorld cannot and does not disable the user's global configuration. |
| R-B4 | CLI start-up adds seconds per call. | The timeout bounds it. The operator's spike may measure it; P6's fallback applies on timeout. |
| R-B5 | `taskkill` is unavailable or slow on Windows. | It is part of every supported Windows. AB-5 runs on the Windows CI leg. The fallback is a Job Object through `ctypes`, recorded as a deviation if needed. |

---

## 8. Commit plan

### C0 — Freeze, contract, re-audit of P5a's merged code

- [ ] Implementation: the `DESIGN FROZEN` header with the operator's QP5b-1 and QP5b-2 rulings; §9
  filled; handoff `handoff-s10-p5b.md`.
- [ ] Validation: both doc checks.
- [ ] Review: P5a's registry, configuration and gateway on `main` match §3's assumptions. Any
  difference is recorded here before C1.

### C1 — Decisions and the operator's CLI check

- [ ] Implementation:
  - `DEP-33` (§4.3);
  - `ARC-60`, the subscription route: the terms evidence, the gate, I-B1 … I-B5;
  - the README's "Subscriptions" section.

  The operator pastes `codex exec --help` from their own machine. The preset flags are fixed from
  that text and recorded with the CLI version.
  **The OpenAI Terms of Use re-read** (QP5b-2's condition): the live page's effective date and its
  clauses on programmatic output extraction and on account sharing are pasted into §4.2, replacing the
  search-excerpt quote.
- [ ] Validation: both doc checks; the README section quotes §4 with dates and URLs; §4.2 carries the
  re-read text and its date.
- [ ] Review:
  - no claim about a flag rests on anything but the pasted help or the documentation;
  - the re-read does not contradict QP5b-2. If it forbids the use: a material stop, back to the
    operator, and C4 becomes N/A.

### C2 — `AnthropicMessagesBackend`

- [ ] Implementation: §5.1; registry entry; AB-2's check in `config.py`.
- [ ] Validation: AB-1 and AB-2 with their mutations; P5a's AP5-9 secret test re-run with
  `kind = "anthropic"` (the `x-api-key` header must be received by the mock, and found nowhere else).
- [ ] Review: only `anthropic_messages.py` imports `httpx2` among P5b's files.

### C3 — `CliBridgeBackend` core

- [ ] Implementation: §5.2, D-B1 … D-B5, D-B7, D-B8; fake CLIs under `tests/fakes/`.
- [ ] Validation: AB-3, AB-4, AB-5, AB-6 and AB-8 with their mutations, over a test-only `fake`
  preset.
- [ ] Review: argv construction has no string formatting of request content; stderr is scrubbed
  before entering any message.

### C4 — The Codex preset (ruled in by QP5b-2; starts only after C1's terms re-read)

- [ ] Implementation: §5.4 Codex; `acknowledge_terms`; README usage.
- [ ] Validation: AB-7 with its mutation; the preset parses a **recorded fake** event stream built
  from the documentation's event names; the fake CLI exercises `turn.failed`.
- [ ] Review: no shipped example enables the kind.
- **N/A** only if C1's re-read shows that OpenAI's terms forbid the use (a material stop first).

### C5 — The Claude Code preset: N/A (dropped by QP5b-1, operator, 2026-10-09)

- N/A: Anthropic does not permit a third-party product to route requests through a user's Free, Pro or
  Max plan credentials (§4.1). The registry has no such kind, and AB-7 checks that
  `claude-code-subscription` is refused as unknown.

### C6 — CI and close-out

- [ ] Implementation: README; ledger; handoff closed.
- [ ] Validation: AB-9 on three CI legs; AB-10; the whole Python suite and static checks; both doc
  checks.
- [ ] Review: §2.3 against the diff; every `[x]` with evidence.
- **Stop:** `READY FOR OPERATOR REVIEW — DO NOT MERGE`.

---

## 9. Execution contract (filled at freeze, 2026-10-09)

Source of every line marked "primary session": its freeze message of 2026-10-09, relayed by the
coordinator to the S10 planning session.

```text
PROJECT / PR:            MineWorld mvp0, S10 PR P5b — native Anthropic adapter; subscription bridges
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p5b-hosted-subscriptions.md
RELATED / BINDING DOCS:  pr-s10-p5-backends.md (P5a, merged first); step-17-cognition.md §3.7;
                         docs/ARCHITECTURE.md §9.2; docs/REUSE_POLICY.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s10-p5b            (primary session)
BRANCH:                  mvp0/pr-s10-p5b-hosted, created from main after P5a merges (primary session)
IMPLEMENTATION BASE:     origin/main after P5a's merge; C0 records it. P5b does not start before then
APPROVED SCOPE:          §2.1 as ruled: the Anthropic adapter, the bridge core and the Codex opt-in; no
                         Claude subscription preset
FROZEN INVARIANTS:       §2.3; D-B1 … D-B8; P5a's invariants; QS10-19; QP5b-1 (dropped);
                         QP5b-2 (opt-in, off by default, never in an example, gated, documented last)
SEQUENCE:                C0 … C6 (C4 only after C1's OpenAI terms re-read; C5 N/A by QP5b-1)
ALLOWED COMMANDS:        cargo *; git; gh (never merge); uv *; python3 scripts/*; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; awk; xargs; curl; heredoc writes; running a real `codex` or
                         `claude` binary (help text comes from the operator); reading any key file,
                         ~/.config/mineworld/secrets.env, ~/.codex, ~/.claude or a keychain; using any
                         API key; calling any hosted API; running or downloading any model
MATERIAL STOPS:          any change to P5a's seams; any dependency (P5b adds none); a preset the operator
                         did not rule in (any Claude subscription route); any CI command naming a real
                         CLI; OpenAI's re-read terms forbidding the Codex opt-in
PLATFORMS:               Linux, macOS, Windows (AB-9)
VALIDATION BUDGET:       unit and fake-CLI integration: unrestricted. Real CLIs, keys, hosted models: none
LIVE DOCUMENTATION:      this document (a ledger section is added in C0)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s10-p5b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized   (primary session freeze)
                         semantic commits: authorized                    (primary session freeze)
                         branch push: authorized                         (primary session freeze)
                         PR creation / update: authorized                (primary session freeze)
                         CI repair to review readiness: authorized       (primary session freeze)
                         merge: explicit operator authorization only
POST-MERGE SYNC OWNER:   the S10 planning session owns step-17 §15 and overall.md; the implementation
                         session owns this document's ledger, evidence and deviations
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 10. Questions

### 10.1 Rulings, 2026-10-09

| ID | Ruling |
| --- | --- |
| QP5b-1 | **Operator: drop** the Claude subscription route. Claude goes through the user's own API key and the native adapter. The reason is quoted in §4.1. |
| QP5b-2 | **Operator: opt-in for the user's own local use only**: off by default, never in an example, gated by `acknowledge_terms`, documented after the API-key routes. OpenAI's Terms of Use are re-read before the preset is built (C1 → C4). |
| QP5b-3 | **Primary:** `ARC-60` for the subscription route as it remains. S10 also receives `ARC-72`, `ARC-73` and `ARC-74`. |
| QP5b-4 | **Primary: yes.** `DEP-33` is the native Anthropic adapter, with the SDK declined. |
| QP5b-5 | **Primary: yes.** P5b follows P5a and may run in parallel with P4. |
| QP5b-6 | **Primary: yes.** The README lists the API-key routes first and subscriptions after them. |
| Freeze | **Primary session, 2026-10-09:** P5b DESIGN FROZEN; contract §9 filled. |

### 10.2 The questions as asked (kept for review)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QP5b-1 [operator — terms of service]** | Claude through the user's Claude Code subscription (`claude -p`)? | **Drop it.** Anthropic: "Anthropic does not permit third-party developers to … route requests through Free, Pro, or Max plan credentials on behalf of their users", enforced "without prior notice" (§4.1). Claude stays fully supported through the user's own API key and the native adapter. The only alternative is an opt-in for the user's own local use, which this design does not recommend for Claude. |
| **QP5b-2 [operator — terms of service]** | Codex through the user's ChatGPT plan (`codex exec`)? | **Opt-in for the user's own local use, documented but not promoted** (D-B6). OpenAI's text is unclear rather than forbidding: it recommends API keys for automation, and its consumer terms bar programmatic extraction of output (§4.2). The alternative is to drop it. |
| QP5b-3 [primary] | `ARC-60` for the subscription route? It is S10's last ARC number; P4 needs one (`ARC-59`), and P6 will likely need more. | **Yes**, and allocate further ARC numbers to S10 before P6 is designed. |
| QP5b-4 [primary] | The native Anthropic adapter over `httpx2`, with the `anthropic` SDK declined (`DEP-33`), as QP5-1's ruling asked to compare? | **Yes**, for QP5-1's reasons. The SDK's implicit environment reads are re-verified in C1. |
| QP5b-5 [primary] | P5b after P5a, possibly in parallel with P4? | **Yes.** P5b touches no file P4 touches beyond the registry line. |
| QP5b-6 [operator] | Present Claude by API key, OpenAI by API key, DeepSeek and GLM as the recommended hosted routes in the README, with subscriptions listed after them as a user's own opt-in? | **Yes.** It matches the vendors' own guidance and keeps the user's account safe. |

---

## 11. Ledger (live during implementation)

```text
Status:            IN PROGRESS (C0)
Implementation
base:              origin/main @ 02788e6 (#123), which contains P5a's merge 60a6295 (#120)
Worktree / branch: /Users/yuema137/mineworld-worktrees/impl-s10-p5b, mvp0/pr-s10-p5b (sole writer)
Post-merge sync:   the S10 planning session owns step-17 §15 and overall.md; this session owns this
                   ledger, its evidence and its deviations
Handoff:           handoff-s10-p5b.md
```

### 11.1 C0 — freeze, contract, re-audit of P5a's merged code

- [x] Implementation: the `DESIGN FROZEN` header and §9 were committed with the freeze (#111). This
  session records the base (`02788e6`; P5a merged as `60a6295`, #120) and initializes
  [`handoff-s10-p5b.md`](handoff-s10-p5b.md).
- [x] Validation (E-P5b-0): `check_doc_headings.py` ("193 numbered sections across 26 documents, none
  duplicated") and `check_decision_ids.py` ("98 decision ids, all distinct"), both exit 0; `grep -n
  'ARC-60\|DEP-33' docs/DECISIONS.md` finds only ARC-56's note of S10's range: both numbers are free.
  Baseline: `uv run --locked pytest cognition/lm-controller` at `02788e6`, 81 passed.
- [x] Review: P5a's merged code against §3's assumptions (files read in full: `backend/model.py`,
  `backend/registry.py`, `backend/openai_compatible.py`, `backend/providers.py`, `config.py`,
  `gateway.py`, `record.py`, `budget.py`, `secrets.py`, `errors.py`, `tests/support.py`,
  `tests/test_provider_scan.py`):
  - `registry.FACTORIES: dict[str, Factory]`, `Factory = (BackendConfig, Secret | None) -> ModelBackend`,
    each adapter imported inside its factory: P5b adds a kind by one entry, as assumed.
  - `BackendConfig.kind` is `Literal["openai-compatible"]` with that default (DV-P5-3); `base_url` is
    required and validated by `check_base_url`; `temperature` defaults to `"send"`. The `anthropic` kind
    needs `"omit"` as its default (§5.1): a kind-dependent default in the same `before` validator that
    fills presets.
  - `ModelGateway.complete` runs the budget pre-check, then the limiter, then the backend under
    `asyncio.timeout(call_timeout_s)`, which **cancels** the backend's coroutine: a bridge must kill
    its process tree on cancellation as well as on its own bound (D-B4, D-B7 hold by placement).
  - `RecordingBackend` computes the key **before** the call and serializes the request **after** it:
    a backend that lowered the schema in place would leave a cassette whose key no longer matches its
    request (the load check refuses it). AB-1's key half is tested through that path.
  - **Difference 1:** `BackendFailure` carries only `reason` and `status`; it has no message field, and
    P5a has no "secret scrub" (`grep -ri scrub src` finds only `record.py`'s "redaction holds by
    construction, not by scrubbing"). §5.2 step 8's "first 200 characters of stderr after the scrub"
    cannot be carried without changing P5a's seam (a material stop). Resolved in C3 as DV-P5b-2.
  - **Difference 2:** `tests/support.run` runs every coroutine on `asyncio.SelectorEventLoop` (F-P5-4).
    On Windows a selector loop cannot start a subprocess; resolved in C3 (F-P5b-1).
  - The contract's branch name is `mvp0/pr-s10-p5b-hosted`; the primary session's kickoff of this
    session names `mvp0/pr-s10-p5b`, which is used (DV-P5b-1).

### 11.2 C1 — decisions and the operator's CLI check

- [x] Implementation: `docs/DECISIONS.md` `DEP-33` (the native adapter; the `anthropic` SDK and
  `claude-agent-sdk` declined, with the SDK's environment reads verified in source) and `ARC-60` (the
  subscription route as ruled: no Claude route, the bridge invariants I-B1 … I-B5, the Codex opt-in
  conditional on the terms re-read); `cognition/lm-controller/README.md` "Subscriptions", after the
  API-key routes (QP5b-6), quoting §4 with URLs and dates.
- [ ] **The operator's `codex exec --help` paste: NOT AVAILABLE.** No operator paste reached this
  session, and an agent does not run the CLI (§9 NEVER). It is needed by C4 only.
- [ ] **The OpenAI Terms of Use re-read: INCONCLUSIVE** (§4.2): two WebFetch attempts, HTTP 403 each.
  Per the freeze header, **C4 does not start**; this is an open operator obligation, not a material
  stop (nothing shows that the terms forbid the use).
- [x] Validation (E-P5b-1): `check_doc_headings.py` and `check_decision_ids.py` exit 0 (100 ids, all
  distinct, after `DEP-33` and `ARC-60`); the README section carries both URLs and the date
  2026-10-09; §4.2 carries the re-read attempt, its date and its result.
- [x] Review: no claim about a Codex flag is made anywhere (no preset exists); the README states that
  the Codex route is not available yet, rather than describing an unbuilt route as usable; nothing in
  the re-read attempt contradicts QP5b-2.

**External research recorded at C1 (§2 of the working rules).**

```text
question     does the anthropic SDK read the environment and retry implicitly? (§4.3 "not re-verified")
source       github.com/anthropics/anthropic-sdk-python, main: src/anthropic/_client.py and
             src/anthropic/_constants.py (raw files, WebFetch, 2026-10-09)
conclusion   Anthropic.__init__ and AsyncAnthropic.__init__ each read ANTHROPIC_API_KEY,
             ANTHROPIC_AUTH_TOKEN and ANTHROPIC_BASE_URL with os.environ.get when the argument is
             omitted; max_retries defaults to DEFAULT_MAX_RETRIES = 2; DEFAULT_TIMEOUT 600 s, connect 5 s
consequence  confirms DEP-33's rejection (D-P5-4's rules (a) and (c)); no limitation follows
```

### 11.3 C2 — `AnthropicMessagesBackend`

- [x] Implementation: `backend/anthropic_messages.py` (`API_VERSION`, `request_body`, `_parse`,
  `AnthropicMessagesBackend(config, key, *, transport=None)`, `trust_env=False`, no retry);
  `backend/strict_schema.py` (`lower_schema`, pure, returns a copy; DV-P5b-3); `backend/registry.py`
  (`"anthropic"` → a factory importing the adapter when called); `config.py` (`Kind =
  Literal["openai-compatible", "anthropic"]`; the `anthropic` kind defaults `temperature` to `"omit"`;
  `_kind_options`: AB-2's refusal of `openai-compatible` on `api.anthropic.com` or any
  `*.anthropic.com` host, and, for `anthropic`, refusals of `preset`, `structured_output =
  "json_object"` and a `reasoning` setting — D-P5b-a below); `examples/hosted.toml.example` gains a
  `[backends.claude]` table (bound to no tier) and `.env.example` `ANTHROPIC_API_KEY=`; README's Claude
  paragraph. Tests: `tests/test_anthropic_messages.py` (new); `test_secrets.py`'s AP5-9 test is
  parametrized over `kind` (`openai-compatible`, `anthropic`); `test_provider_scan.py` allows
  `anthropic_messages.py` (I-10) and expects it as the second `httpx2` importer;
  `test_structural_isolation.py`'s replay child now binds a second tier to an `anthropic` backend and
  checks that neither adapter (nor `cli_bridge`) nor `httpx*` is imported.
- [x] Validation (E-P5b-2): cognition suite **101 passed** (81 → 101); ruff check, ruff format, pyright
  strict clean over the workspace.
  - AB-1: the exact body for a literal five-message request (two `system` joined with `"\n"` and
    hoisted; `user`/`assistant` in order; `max_tokens` 256; no `temperature`, no `seed`;
    `output_config.format` with `maxLength` removed and `additionalProperties: false`), `x-api-key`
    equal to the key, `anthropic-version: 2023-06-01`, no `authorization`; the text of two text blocks
    concatenated, usage mapped. `temperature = "send"` sends 0.7; `structured_output = "none"` sends no
    `output_config`; no key → no `x-api-key`. Recorded through `RecordingBackend`: the entry's key equals
    AP5-1's literal `6bd78b32…` and its request equals the golden request (no `additionalProperties`),
    while the body sent carried `additionalProperties: false`. `end_turn`/`stop_sequence` → `complete`,
    `max_tokens` → `length`, `refusal` → `refused`, `tool_use` and a non-JSON body →
    `malformed_response`; missing usage → estimated, flagged (4 output tokens, by hand); 401 and 403 →
    `unauthorized`, 429, 500 and 529 → `http_status(code)`, one request each (no retry).
  - Lowering, against a hand-written expectation: a property *named* `minimum` kept with its bounds
    removed; `minItems: 2` and `maxItems` removed, `minItems: 1` kept; an `anyOf` branch with
    `properties` closed; an `enum` value `{"maxLength": 3}` left alone; `additionalProperties: true`
    overridden to `false`; the input unchanged.
  - AB-2: `openai-compatible` at `https://api.anthropic.com/v1/` → `ConfigError` naming
    `kind = "anthropic"` and `response_format`. `claude-code-subscription` → `ConfigError` at
    `backends.x.kind` (QP5b-1).
  - AP5-9 re-run with `kind = "anthropic"`, both key sources: the mock received the key in `x-api-key`
    three times; it is absent from the cassette, the SQLite ledger, every log record, stdout, stderr
    and every `str`/`repr`.
  - Mutations (§11.8): M-b1 lowering in place → the key test fails (`CassetteFormatError`: key
    `6bd78b32…` does not match its request, which hashes to `5e871f83…`); M-b2 `temperature` defaulting
    to `"send"` → the exact-body and the default tests fail; M-b3 AB-2's check removed → AB-2's test
    fails. All reverted; `grep -rn MUTATION cognition/` empty; 101 passed.
- [x] Review: among P5b's files only `anthropic_messages.py` imports `httpx2` (the scan pins the list);
  the adapter reads no environment variable and logs nothing; a failure carries a reason and a status,
  never a body; the example's Claude table is bound to no tier, so the example's behaviour is unchanged.

**D-P5b-a (bounded, fail-closed).** For `kind = "anthropic"` three options of `BackendConfig` have no
meaning: `preset` (every row is an OpenAI-compatible endpoint), `structured_output = "json_object"` (the
Messages API has no such mode) and `reasoning` (no mapping is designed). Each is refused by name rather
than silently ignored. Applies only to the new kind; no P5a configuration changes meaning.

**DV-P5b-3 (bounded).** `backend/strict_schema.py`, a module not in §2.1: §5.1 puts the lowering
"inside the adapter", but §5.4 has the Codex preset use "the same lowering", and a preset importing
`anthropic_messages.py` would import `httpx2` into the bridge. The pure function lives in a module of
its own that names no provider.

**Process note.** Twice this session a Bash call held an empty heredoc (`<<'X' … X`) redirected to
`/dev/null` or to `python3 -` with no body. Neither wrote a file nor ran code (`git status` unchanged);
recorded because the brief forbids heredoc writes.

# PR S10-P5a — Model backends, the recorder and cassettes, cognition budgets, API keys

## DESIGN FROZEN 2026-10-09 (primary session)

```text
Design revision:        revision 2 (2026-10-09), with §12.1's rulings, as committed on plan/s10-p5
                        (PR #111) with this header
Approved by / evidence: the primary session's freeze message of 2026-10-09, relayed by the coordinator
                        to the S10 planning session; the operator's rulings on QP5-2, QP5-3 and the
                        API/subscription requirement of the same day (§12.1)
Implementation base:    main at the start of implementation (exact commit recorded in C0)
Execution contract:     §11 (filled at freeze)
Lifecycle:              FROZEN
```

Scope (§2.1), invariants (§2.3), decisions D-P5-1 … D-P5-14, the adversarial criteria AP5-1 … AP5-13
and AP5-S, and the commit plan are frozen. Progress, evidence, findings and bounded corrections stay
writable (§13). No P5a question remains open.

**Amendment under the freeze, 2026-10-09 (operator requirement, relayed by the coordinator).**

- **The operator, verbatim:**

  > deepseek glm grok之类我们也都要支持

  That is: DeepSeek, GLM, Grok and similar providers must all be supported.
- **Why it is bounded.** The adapter, the configuration shape and every seam are unchanged. The
  amendment adds named **provider presets** for the one OpenAI-compatible adapter: a typed table of base
  URL, `key_env` name, `structured_output`, `temperature` and default `reasoning`. A preset adds no code
  path.
- **What it adds:**
  - §4.1c, the verified provider table;
  - D-P5-15;
  - `backend/providers.py`;
  - AP5-14;
  - the README provider table;
  - one exclusion in AP5-10's scan;
  - items in C4, C6 and C8.
- **Not frozen with it:** nothing. No question is raised.

**Effort:** `mvp0` · **Step:** S10, [`step-17-cognition.md`](step-17-cognition.md) (§3.5, §3.7, §3.10,
§3.11, §4.1, §4.5, §5, §8, §9, §10, and §15, the 2026-10-08 audit with its rulings) · **Parent:**
[`overall.md`](overall.md) §3 (S10), the parallel build-out rulings (ruling 6: S10 holds `ARC-56 … ARC-60`
and `DEP-24 … DEP-27`), and §5 "Operator requirements and rulings, 2026-10-08 to 2026-10-09".
**Predecessor:** P3, the Python SDK, merged as #98 (`285c152`):
[`pr-s10-p3-python-sdk.md`](pr-s10-p3-python-sdk.md).
**Working name:** P5a. The PR number is assigned at freeze.
**The split (revision 2).** The operator's requirement of 2026-10-09 (below) adds hosted APIs and
subscriptions. P5 is therefore two PRs:

- **P5a, this document:**
  - the interface, the key, the recorder, the budgets and the configuration;
  - the one OpenAI-compatible adapter, which also serves every hosted OpenAI-compatible API (OpenAI,
    DeepSeek, Zhipu GLM, …);
  - **API keys** from the process environment or from a `.env`-style file.
- **P5b, [`pr-s10-p5b-hosted-subscriptions.md`](pr-s10-p5b-hosted-subscriptions.md):**
  - the native Anthropic Messages adapter;
  - the **subscription** route, which runs the user's own logged-in CLI;
  - the terms-of-service evidence that gates the subscription route.

  P5b depends on P5a and changes none of P5a's seams.

**Decision numbers (QP5-6, ruled):** `ARC-57`, `ARC-58` and `DEP-27` as proposed. S10 also holds
`DEP-32` and `DEP-33`. P5a uses `DEP-32` for `python-dotenv` (§4.2b); P5b uses `DEP-33`.
**Planning base:** `origin/main @ bc4f8e7` (#108, the operator-requirements index, on top of #98).
Branch of this design: `plan/s10-p5`.
**Decision identifiers (ruled 2026-10-09):**

- `ARC-57`, budgets (step-17 placeholder `ARC-S10-d`);
- `ARC-58`, recorded model outputs (`ARC-S10-e`);
- `DEP-27`, the HTTP client and the decision to depend on no provider SDK, replacing `DEP-S10-a`;
- `DEP-32`, `python-dotenv`.

### Binding rulings this design is written under

| Source | Ruling | Where this design meets it |
| --- | --- | --- |
| **QS10-19** (operator, 2026-10-08) | Local models only. No MVP-0 gate depends on a paid API. Cassettes are recorded from a local model such as Ollama. A hosted API is an optional backend that users configure for themselves. Agents never use the operator's key, never read `~/.config/mineworld/secrets.env`, and never call a hosted API. | D-P5-4, D-P5-9, D-P5-12, AP5-6, AP5-9, §11 NEVER |
| **QS10-18** (primary, 2026-10-08) | Cost ceilings (calls, tokens) are keyed on **wall** time. The context bound is keyed on **simulated** time. | D-P5-8, AP5-7, the `ARCHITECTURE.md` §9.1 edit in C1 |
| **Platforms** (operator, 2026-10-08) | macOS, Linux and Windows are all supported. | D-P5-13, AP5-12 |
| **`CLAUDE.md` §4 rules 3, 10** | No LM-provider concept in cognition contracts. Core tests never need a live model; they use recorded cognition. | I-10, I-11, D-P5-2, D-P5-5, AP5-5, AP5-10 |
| **`AC-4`** | Swapping the LM backend needs no World Pack edits. | D-P5-3, AP5-8 (the seam half; the scenario half is P7's IC-6) |
| **QS10-2** (operator, accepted 2026-10-08) | A local model, chosen by a spike against fixed criteria. | C7 (the spike tool), QP5-2, QP5-3 |
| **QP5-2** (operator, 2026-10-09) | Start from `qwen3.5:4b`; the spike settles the final choice. | §4.4, AP5-S, C7 |
| **QP5-3** (operator, 2026-10-09) | The operator runs the spike personally before P6 freezes. Agents never run, pull or download a model. | C7, §11 NEVER |
| **QP5-1** (primary, 2026-10-09) | Yes: `httpx2`, no provider SDK. Revisit only if the native Anthropic adapter needs one, as a compared and recorded decision. | D-P5-4; P5b re-compares the Anthropic SDK |
| **QP5-4, -5, -7, -8, -9** (primary, 2026-10-09) | As recommended. | D-P5-8, D-P5-1, §2.2, D-P5-12 |
| **API and subscription access** (operator, 2026-10-09), verbatim: "我们还需要提供api和订阅接口，这样才能支持非本地模型，比如gpt claude glm deepseek等……api通过.env 之类的配置，subscription就用codex或者claude code本身的authorize验证" | MineWorld also supports non-local models (GPT, Claude, GLM, DeepSeek, …) through API keys configured in a `.env`-style file, and through subscriptions authorized by Codex's or Claude Code's own login. QS10-19 still holds: no MVP-0 gate depends on any paid API or subscription; CI runs scripted backends and cassettes only; live tests are operator-run and opt-in; agents never read keys or call hosted models. | P5a: §4.1b, §4.2b, D-P5-9, D-P5-14, AP5-9, AP5-13. P5b: the Anthropic adapter and the subscription route |

---

## 1. Goal, in one paragraph

A cognition component asks a language model a question through one provider-neutral interface,
`ModelBackend`, and receives a typed `Completion`. Behind that interface P5 provides four things. The
first is one adapter for the OpenAI-compatible HTTP schema, which covers Ollama, llama.cpp's server, LM
Studio, vLLM and any hosted endpoint a user configures. The second is a deterministic `ScriptedBackend`
for tests. The third is a recorder that writes and strictly replays **cassettes** at our interface,
keyed by a hash of the provider-neutral request, so one cassette replays under any backend. The fourth
is a **budget gate**: per-seat call and token ceilings keyed on wall time, a process-wide limit on calls
in flight, and a per-call timeout. All four sit in front of every backend. P5 also adds the operator's
TOML configuration for backends, tier bindings, budgets and recording mode. In it, a key appears only as
the **name** of an environment variable. The value of that variable comes from the process
environment, or from a `.env`-style file at a path the operator chooses (D-P5-9). With that, the same
adapter reaches a hosted OpenAI-compatible API such as OpenAI, DeepSeek or Zhipu GLM, configured by the
user. P5 creates the `cognition/lm-controller` package
(`mineworld-cognition`), adds a provider-concept scan, and ships an operator-run spike tool that will
choose the default local model. Nothing in P5 calls a model in tests or CI. Nothing in it makes a social
decision, holds memory or joins a seat: those are P6 and P4.

## 2. Scope

### 2.1 In scope

- `cognition/lm-controller/` as the second uv workspace member, `mineworld-cognition`:
  `pyproject.toml`, `README.md`, `src/mineworld_cognition/{__init__.py, py.typed}`, and the modules
  below.
- `backend/`:
  - `model.py`: `ModelBackend`, `CompletionRequest`, `Completion` and their parts;
  - `canonical.py`: the canonical JSON and the cassette key;
  - `scripted.py`: `ScriptedBackend`;
  - `openai_compatible.py`: `OpenAICompatibleBackend`;
  - `registry.py`: adapter `kind` → factory.
- `record.py`: the cassette format, `ReplayBackend`, `RecordingBackend`, `CassetteMiss`.
- `budget.py`: `BudgetPolicy`, the `Ledger` protocol with `MemoryLedger` and `SqliteLedger`, `Clock`,
  the in-flight limiter, the timeout, and `GateOutcome`.
- `gateway.py`: `ModelGateway`, which composes budget gate → recorder → backend for one tier binding.
  `Router` maps a tier to a gateway, or to nothing.
- `config.py`: the TOML configuration model (`tomllib` plus Pydantic), with the sections P5a owns:
  `[backends.*]`, `[tiers]`, `[budgets]`, `[recording]` and `[secrets]`.
- `secrets.py`: `Secret`, and `resolve_key(name, env_file)`. It reads the process environment, or a
  `.env`-style file through `python-dotenv`'s `dotenv_values(path, interpolate=False)`, which never
  writes to `os.environ` (D-P5-9, `DEP-32`).
- `.gitignore`: `.env.*`, with `!.env.example` re-included, beside the existing `.env` and `*.env`
  (§3).
- `cognition/lm-controller/examples/`:
  - `hosted.toml.example`: OpenAI, DeepSeek and Zhipu GLM as OpenAI-compatible backends, each with
    `key_env` only;
  - `.env.example`: variable names with empty values.

  Neither is used by any test or CI job.
- Tests under `cognition/lm-controller/tests/`, a test cassette directory `tests/cassettes/`, and the
  provider-concept scan (`tests/test_provider_scan.py`, the P5 half of IC-10).
- `cognition/lm-controller/tools/model_spike.py`: the operator-run spike (QS10-2). Agents and CI never
  run it.
- The root `pyproject.toml` (the workspace members; pyright's `include`), `uv.lock`, `.gitattributes`
  (`*.jsonl text eol=lf`), `.structured-coding/standards.md`, and `scripts/ci_layer.py` (the Python
  commands cover the new member, D-P5-11).
- Decision records `ARC-57`, `ARC-58` and `DEP-27` in `docs/DECISIONS.md`. The `ARCHITECTURE.md` §9.1
  edit: wall-time cost limits, under QS10-18. The `ARCHITECTURE.md` §9.2 note: backends are adapters
  behind `ModelBackend`, and the OpenAI-compatible schema is the common interface.
- Every platform: Linux, macOS, Windows (D-P5-13).

### 2.2 Not in scope, and where it goes

| Not here | Why | Where |
| --- | --- | --- |
| `SocialChoice`, context assembly, the repair re-ask, triggers, `LMController`, the decision log, deterministic-speech fallback | They are the controller's. P5 provides the typed outcomes (`GateOutcome`) the controller falls back on | P6 |
| Memory, compression, `AC-10`, the memory store | P4, after S11-C's `mineworld perceived` | P4 |
| Config keys `server`, `seats`, `store`, and `python -m mineworld_cognition` | They bind seats, which P6 does | P6 extends `config.py` |
| `OllamaNativeBackend` (`/api/chat`, `format=`, `options.num_ctx`, `think`) | Added only if the spike shows `/v1` insufficient (QS10-3, A-3). `CLAUDE.md` §4 rule 11 | conditional: a follow-up PR, or P6, on spike evidence (QP5-7) |
| The native Anthropic Messages adapter | Anthropic's OpenAI-compatibility layer ignores `response_format` and `seed`, and refuses `temperature` below 1 on recent models (§4.1b). Claude needs its own adapter | **P5b** |
| Subscription access through the user's Claude Code or Codex login | Gated by terms-of-service evidence and an operator decision | **P5b** |
| Recording a cassette from a real model, and running the spike | Needs a running local model. Under the brief, agents run no model. The operator runs it (QP5-3) | operator, before P6's freeze |
| A live test against two real OpenAI-compatible servers (QS10-5's live `AC-4` variant) | Operator-run, optional | marker `live_model`, never in CI (D-P5-12) |
| The `AC-4` scenario test (IC-6) and removability (IC-7) | They need P6's controller and P7's scenario | P7. P5 proves the seam (AP5-8) |
| Any Rust change, any change to `sdk/python` | I-1. P5 consumes `mineworld-sdk`'s `EntityKey` type and changes nothing in it | — |
| Prices or money | Prices are provider concepts. Tokens are reported, never priced (step-17 §3.10.4) | never |

### 2.3 Invariants this PR must hold

From step-17 §5, restricted to what P5 can break.

```text
I-1    no change under kernel/, contracts/, persistence/, server/, systems/, worlds/, sdk/python/src/
I-2    no Rust crate depends on Python; mineworld-cognition → mineworld-sdk, one way; mineworld-sdk does not
       import mineworld_cognition
I-7    byte-identity: no Rust change, so every frame, transcript and digest is unchanged (the AP5-11 diff
       gate holds it)
I-10   no provider concept (a provider name, a model name, an endpoint, a key) in a cassette KEY, in
       CompletionRequest, in Completion, or in any module outside backend/openai_compatible.py,
       backend/registry.py and config.py
I-11   the full Python suite passes with no model reachable and outbound network blocked except loopback.
       P5 adds: no test reaches a model on loopback either (AP5-5, AP5-6)
I-16   no secret is written anywhere: log, cassette, ledger, save, exception message, repr or CI
       output; a key file's values never enter os.environ
INV-14 simulation semantics never depend on the model provider: P5 has no path into the world at all
P5-1   (new) replay is strict: a cassette miss raises CassetteMiss and never reaches a backend
P5-2   (new) the budget gate decides before the recorder and the backend; a refused call reaches neither
P5-3   (new) cost ceilings read only the wall clock; no budget code reads simulated time
```

---

## 3. Audit anchors (`origin/main @ bc4f8e7`)

Each row was read in this planning session. The implementing session re-reads each one before editing
(working rules §3).

| Anchor | What it establishes for P5 |
| --- | --- |
| `pyproject.toml` (root) | A virtual uv workspace, `members = ["sdk/python"]`. `[tool.pyright]` has `include = ["sdk/python"]`, strict, `pythonVersion = "3.12"`. P5 adds the member and the include. |
| `sdk/python/pyproject.toml` | The precedent: `requires-python >=3.12`; `uv_build`; the ruff rule set `E F I UP B ANN RUF`; pytest `addopts = ["--strict-markers", "--allow-hosts=127.0.0.1,::1"]` (the network guard), and the `real_server` marker. P5's package copies the guard and the rule set. |
| `sdk/python/src/mineworld_sdk/wire/ids.py` | `EntityKey` (a seat's key: `join.seat`) and `JsonValue` (recursive, **includes `float`**). P5 uses `EntityKey` for budget accounts and `JsonValue` for `output_schema`. Because `JsonValue` admits floats, the canonical JSON must refuse them explicitly (D-P5-6). |
| `sdk/python/src/mineworld_sdk/__init__.py` | Public names. `EntityKey` and `JsonValue` are not re-exported at the top level. P5 imports them from `mineworld_sdk.wire.ids`, a stable module path inside the workspace. No SDK edit (I-1). |
| `scripts/ci_layer.py` lines 59–101 | `PYTHON_STATIC` runs `ruff check`, `ruff format --check` and `pyright` over `sdk/python`, and is appended to `fast`. The `python` layer runs `uv run --locked pytest sdk/python`; `python-smoke` adds `-m "not real_server"`. `COMMAND_ENVIRONMENT` sets the pyright variables. P5 extends these commands (D-P5-11). |
| `.github/workflows/ci.yml` `python` job | A matrix of `ubuntu-24.04` (container), `windows-2025` and `macos-15`, full `python` layer on each (P3 E-P3-6). It names layers only, so P5 changes no YAML. |
| `.structured-coding/standards.md` | Checks `ruff-lint`, `ruff-format`, `pyright-strict` and `pytest` with argv over `sdk/python` (DV-P3-4). P5 extends the argv. |
| `.gitattributes` | Only `*.bin` and `*.glb binary`. Nothing fixes `*.jsonl` line endings. P5 adds `*.jsonl text eol=lf`, so a Windows checkout of a cassette stays LF (D-P5-13). |
| `docs/ARCHITECTURE.md` §9.1 | `limits: max_calls_per_sim_hour: 20; max_tokens_per_day: 30000` — keyed on simulated time. **This contradicts QS10-18**, so C1 edits it (`CLAUDE.md` §2.1(4)). §9.2 lists OpenAI-compatible, Ollama, llama.cpp, vLLM and custom HTTP as first-class interfaces. |
| `docs/DECISIONS.md` | `ARC-56` (cognition is a client), `DEP-24` Pydantic, `DEP-25` websockets, `DEP-26` the toolchain. `ARC-57 … ARC-60` and `DEP-27` are unused (grep). |
| `overall.md` §5, decision-number table | "S10 P3: ARC-56; DEP-24, DEP-25, DEP-26 (DEP-27 remains; S10 asks the primary session for more when needed)." |
| `step-17` §3.7, §3.10, §3.11, §4.1, §4.5 | The step design P5 implements, with two refinements: QS10-18's wall-time keys, and the HTTP client (QP5-1). |
| `pr-s10-p3-python-sdk.md` §12 | DV-P3-2: with `--allow-hosts` set, pytest-socket's guard is connect-only and **allows loopback**. A local Ollama on `127.0.0.1:11434` would therefore pass the guard. This is why P5 needs structural guards (AP5-5, AP5-6), not the socket guard alone. |
| `.gitignore` lines 51–55 | "Credentials. Keys live outside the tree (`~/.config/mineworld/secrets.env`)". The patterns are `.env`, `*.env` and `secrets*`. **Missing:** `.env.local`, `.env.production` and the like, which `*.env` does not match. P5a adds `.env.*` and `!.env.example`. |
| `cognition/` | Holds only `rule-controller` (Rust) and `.gitkeep`. There is no `lm-controller`. P5 creates it; P4 (waiting on S11-C) builds on it (R-P4-1, §10). |

**Not verified in planning; verified in C1.** An assumption that fails here goes to §8 with its
fallback.

- `httpx2` (2.13.1, 2026-09-23, BSD-3-Clause, maintained by Pydantic Services) offers an `AsyncClient`, a
  `MockTransport` (or an equivalent transport hook for tests), and per-phase timeouts, as `httpx` did.
  PyPI and the README were read; the README does not mention `MockTransport`.
- `httpx2`'s dependencies (`anyio`, `httpcore2`, `idna`, `truststore`) resolve in the universal lock with
  wheels or pure-Python distributions for `win_amd64`, `macosx_*_arm64` and Linux.
- pytest-socket's `--allow-hosts` blocks an `httpx2` connection to a non-loopback address, as it blocks a
  raw socket (AP5-10).
- Ollama's `/v1/chat/completions` accepts `response_format: {"type": "json_schema", …}`. Ollama's
  documentation says only that structured outputs "work through the OpenAI-compatible API via
  `response_format`". This is A-3, verified by the operator's spike, not by an agent (QP5-3).

---

## 4. Reuse comparison (the operator's standing rule)

Facts were read from primary sources on 2026-10-09: PyPI JSON metadata, the GitHub API's licence field,
projects' documentation and Hugging Face model metadata. Where a fact was not confirmed it says *not
verified*. Fit is judged on step-17 §4.0's three properties: **N** provider-neutral, **D**
deterministic, **V** valid without a model.

### 4.1 Backends: the servers a user runs

P5 integrates none of these as a dependency. They are servers a user runs, reached over HTTP. The
question is which **interface** the adapter speaks.

| Backend | Licence (verified) | OpenAI-compatible `/v1/chat/completions` | JSON-Schema structured output | Platforms | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Ollama** | MIT (GitHub API) | Yes. Supported fields include `response_format`, `seed`, `temperature`, `max_tokens`, `stream`, `reasoning_effort`. `tool_choice`, `logit_bias`, `user` and `n` are unsupported. The API key is "required by the client, but Ollama ignores it" | Native `format=<schema>` documented. Through `/v1`: "work through the OpenAI-compatible API via `response_format`", with no `json_schema` example (A-3, open). `num_ctx` cannot be set through `/v1`; a Modelfile is needed | macOS, Linux, Windows | **Default local server** (QS10-2), through the common adapter. Native adapter only on spike evidence (QP5-7) |
| **llama.cpp `llama-server`** | MIT (GitHub API) | Yes, and it returns a standard `usage` object | `response_format` "supports both plain JSON output" and "schema-constrained JSON" (README) | all three; prebuilt binaries | **Second local server** for QS10-5: shows the adapter is generic without a paid API. Operator-run, live, optional |
| **LM Studio** | The desktop app is proprietary freeware: its terms forbid redistribution and derivative works. "Free for work" since July 2025. Its SDKs are MIT | Yes | `json_schema` through `/v1`, in OpenAI's format; reportedly no `json_object` mode (third-party report, *not verified*) | macOS, Windows, Linux | **Supported as a user's choice** through the same adapter. Never bundled, never required, never used in CI |
| **vLLM** | Apache-2.0 (GitHub API) | Yes | `response_format` with JSON Schema (guided decoding; *not re-verified* this session) | Linux with a GPU, primarily | **Supported as a user's choice** (servers, GPUs). Not a default: it does not serve the modest Windows and macOS machines the brief names |
| **Hosted OpenAI-compatible endpoints** | provider terms | Yes | Yes | n/a | **Optional, user-configured** (QS10-19). Never in a gate, a test, CI or an agent's run |

**Conclusion.** The OpenAI-compatible chat-completions schema is the one interface all five speak. P5
implements exactly one adapter for it (`openai_compatible.py`). Two typed configuration fields cover
the differences that matter: `structured_output = "json_schema" | "json_object" | "none"` and
`reasoning = "off" | "low" | "medium" | "high" | "unset"`. There is no free-form `extra` map (D-P5-10).

### 4.1b Hosted APIs (operator requirement, 2026-10-09)

Hosted services are configured by a **user**, with that user's key and at that user's cost. No test,
CI job, gate or agent calls one (QS10-19). Facts were read on 2026-10-09.

| Service | OpenAI-compatible endpoint | Structured output through it | Covered by | Notes |
| --- | --- | --- | --- | --- |
| **OpenAI** (GPT) | `https://api.openai.com/v1` (the schema's origin) | `response_format` `json_schema` | P5a, `OpenAICompatibleBackend` | `structured_output = "json_schema"` |
| **DeepSeek** | `https://api.deepseek.com`. Docs: "The DeepSeek API uses an API format compatible with OpenAI/Anthropic". Models `deepseek-flash`, `deepseek-v4-pro` | The fetched page does not mention `response_format`; `json_object` *not verified* | P5a | Example uses `structured_output = "json_object"` until a user confirms `json_schema`. Local validation is the authority either way (P6) |
| **Zhipu GLM** | China `https://open.bigmodel.cn/api/paas/v4`; international (Z.ai) `https://api.z.ai/api/paas/v4`; `/chat/completions` with a Bearer token. Confirmed by third-party integration docs (jambonz, PicoClaw, AgentScope); **Zhipu's own page was not reached** | JSON mode reported by a community adapter; *not verified* | P5a | Example marked "verify against Zhipu's documentation" |
| **Anthropic** (Claude), OpenAI-compatibility layer | `https://api.anthropic.com/v1/`. Anthropic: "primarily intended to test and compare model capabilities, and is not considered a long-term or production-ready solution for most use cases" | **`response_format`: Ignored.** `seed`: Ignored. `temperature`: "On Claude 4.7 and later models … any value below 1 returns a 400 error, so omit it" | **Not used.** The native adapter is P5b | Our requests always carry a schema and usually a temperature below 1, so the compatibility layer would silently drop the one and refuse the other |
| **Anthropic**, native Messages API | `POST /v1/messages`; `output_config.format = {"type": "json_schema", "schema": …}` with no beta header | Yes, with schema limits: `additionalProperties: false` is required; `minLength`, `maxLength`, `minimum` and `maximum` are unsupported (400) | **P5b**, `AnthropicMessagesBackend` over `httpx2` | P5b compares it with the `anthropic` SDK, as QP5-1's ruling requires |

**Conclusion for P5a.** One adapter and three typed options cover every OpenAI-compatible service. The
third option, `temperature = "send" | "omit"` (D-P5-10), is added for servers that reject a temperature.
Claude needs P5b's native adapter, because the compatibility layer ignores the one feature the
controller depends on.

### 4.1c Named provider presets (amendment, 2026-10-09)

Each row was read from the provider's own documentation on 2026-10-09, except where it says otherwise.
"Verify" means the fact could not be confirmed from a primary source. The implementing session re-checks
it from the provider's documentation (never by calling the API), and the README carries the same mark.
The preset holds no model: model names change monthly, so the user names one.

| Preset | Base URL | `key_env` | `structured_output` | `temperature` | `reasoning` | Evidence, and what is unverified |
| --- | --- | --- | --- | --- | --- | --- |
| `openai` | `https://api.openai.com/v1` | `OPENAI_API_KEY` | `json_schema` | `send` | `unset` | The schema's origin |
| `xai` (Grok) | `https://api.x.ai/v1` | `XAI_API_KEY` | `json_schema` | `send` | `unset` | xAI, "Structured outputs" (`docs.x.ai/docs/guides/structured-outputs`): the OpenAI-SDK examples set `base_url` to `https://api.x.ai/v1`; "The primary and most flexible method is to use the `response_format` parameter"; `json_schema` and `json_object` are accepted; "When using supported schema features, the response is guaranteed to match your schema." **Verify:** which Grok models accept `reasoning_effort`, so `unset` stays the default |
| `deepseek` | `https://api.deepseek.com` | `DEEPSEEK_API_KEY` | `json_object` | `send` | `unset` | DeepSeek API docs: "The DeepSeek API uses an API format compatible with OpenAI/Anthropic." **Verify:** `json_object` support (not on the page read) |
| `glm` (Zhipu, China) | `https://open.bigmodel.cn/api/paas/v4` | `ZHIPUAI_API_KEY` | `json_object` | `send` | `unset` | Zhipu, `docs.bigmodel.cn` OpenAI page: "智谱提供与 OpenAI API 兼容的接口" (Zhipu provides an interface compatible with the OpenAI API), base URL `https://open.bigmodel.cn/api/paas/v4/`. **Verify:** JSON mode (not on the page) |
| `zai` (GLM, international) | `https://api.z.ai/api/paas/v4` | `ZAI_API_KEY` | `json_object` | `send` | `unset` | **Verify:** the base URL comes from third-party integration docs (jambonz, PicoClaw); JSON mode is unverified |
| `mistral` | `https://api.mistral.ai/v1` | `MISTRAL_API_KEY` | `json_schema` | `send` | `unset` | Mistral API reference: `https://api.mistral.ai/v1/chat/completions`; `response_format` `json_object` ("guarantees the message the model generates is in JSON") and `json_schema`. **Verify:** the exact `json_schema` envelope, which the page paraphrased |
| `moonshot` (Kimi) | `https://api.moonshot.ai/v1` | `MOONSHOT_API_KEY` | `json_object` | **`omit`** | `unset` | Kimi, "Migrating from OpenAI" (`platform.kimi.ai`): "The Kimi API is compatible with OpenAI's interface specifications"; `temperature` range `[0, 1]`; for `kimi-k2.6`, any value other than the fixed one is an error, "so the page recommends not setting temperature". **Verify:** JSON mode (not on the page) |
| `dashscope` (Alibaba Qwen) | **none: the user sets `base_url`** | `DASHSCOPE_API_KEY` | `json_object` | `send` | `unset` | Alibaba Model Studio, "OpenAI compatibility": the base URL is per workspace and region (`https://{WorkspaceId}.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1`), and "Each API key works only with the base URL of the region where it was created". The legacy `https://dashscope.aliyuncs.com` "remains available". A fixed URL would be wrong for most users, so the preset requires `base_url`. **Verify:** JSON mode (`response_format` is not in the page's parameter table) |
| `gemini` | `https://generativelanguage.googleapis.com/v1beta/openai` | `GEMINI_API_KEY` | `json_schema` | `send` | `unset` | Google, "OpenAI compatibility": `base_url="https://generativelanguage.googleapis.com/v1beta/openai/"`; "Support for the OpenAI libraries is still in beta"; structured output via `response_format`; "Reasoning cannot be turned off for Gemini 2.5 Pro or 3 models", hence `unset` |
| `openrouter` (aggregator) | `https://openrouter.ai/api/v1` | `OPENROUTER_API_KEY` | `json_schema` | `send` | `unset` | OpenRouter API reference: schemas "very similar to the OpenAI Chat API"; `json_object` and `json_schema` modes; if a model does not support a parameter, "the parameter is ignored". Local validation (P6) is the authority |
| `groq` | `https://api.groq.com/openai/v1` | `GROQ_API_KEY` | `json_object` | `send` | `unset` | Groq, "Structured outputs": `json_schema` with `strict: true` only on a few models (`openai/gpt-oss-20b`, `openai/gpt-oss-120b`, `qwen/qwen3.8-27b`); "JSON Object Mode" for others. Hence `json_object` by default; a user may set `json_schema` |

Not presets: Anthropic (P5b's native `anthropic` kind, §4.1b), and the local servers (Ollama, llama.cpp,
LM Studio, vLLM), whose base URL is the user's own loopback address.

### 4.2 The Python HTTP client, and whether to depend on a provider SDK

| Candidate | Licence (verified) | Maturity (verified) | N | D | V | Cost and fit | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **`openai` SDK** (`AsyncOpenAI(base_url=…)`) | Apache-2.0 | 3.27.0; depends on `httpx2`, `anyio`, `jiter`, `pydantic`, `sniffio`, `typing-extensions` | ~ behind `ModelBackend` | ✓ | ✓ | **Implicit environment reads.** When an argument is omitted, the constructor reads `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_ORG_ID`, `OPENAI_PROJECT_ID`, `OPENAI_ADMIN_KEY`, `OPENAI_WEBHOOK_SECRET` and `OPENAI_CUSTOM_HEADERS`, and falls back to `https://api.openai.com/v1` (read in `src/openai/_client.py`). A user's shell that holds a hosted key could send a local test's traffic, or its key, to a hosted API. That breaks QS10-19 and I-16, and every call site would have to defend against it. Built-in retries (`max_retries`) also hide calls from the budget ledger unless disabled. Its value, `parse()` with Pydantic, is ours anyway: we validate locally (step-17 §4.2) | **REJECT** (reverses step-17 §4.1's `DEP-S10-a`; QP5-1). The reasons are the concrete ones above, not "cleaner". Revisit if we need a provider feature the plain schema cannot express (tool calls, batch, realtime) |
| **`ollama-python`** | MIT | 0.6.3, 2026-09-29; depends on `httpx>=0.27` | ✓ | ✓ | ✓ | Native API only; pins the old `httpx` line, which is unmaintained (below) | **REFERENCE ONLY.** If the native adapter is needed (QP5-7), it is about 80 lines over the same HTTP client |
| **LiteLLM** | MIT core; proprietary `enterprise/` | very active | ✗ the provider is in the model string | ✓ | ✓ | large tree | **REJECT** (step-17 §4.1 stands) |
| **`httpx`** | BSD-3-Clause | **0.28.1, uploaded 2024-12-06; no release since** | ✓ | ✓ | ✓ | async-native; `MockTransport`; the API we know | **Fallback only.** It is unmaintained, and the ecosystem has moved: `openai` 3.x and the Anthropic SDK 1.x dropped it for `httpx2` (August 2026, secondary sources; the `openai` dependency was read on PyPI) |
| **`httpx2`** | BSD-3-Clause | 2.13.1, 2026-09-23; "Pydantic is picking up stewardship under the HTTPX2 name" (README); maintained by Pydantic Services, whose `pydantic` we already depend on (`DEP-24`) | ✓ | ✓ | ✓ | async-native, HTTP/1.1 and 2; one dependency line (`anyio`, `httpcore2`, `idna`, `truststore`); OS trust store for TLS | **REUSE** (`DEP-27`). Isolated in `openai_compatible.py`, so a switch to `httpx` or `aiohttp` is one file |
| **`aiohttp`** | `Apache-2.0 AND MIT` | 3.14.4 | ✓ | ✓ | ✓ | nine runtime dependencies and a server framework we do not need | **REJECT.** A heavier client for one POST per decision |
| **Standard library** (`urllib.request` in `asyncio.to_thread`) | PSF | — | ✓ | ✓ | ✓ | no dependency; but blocking I/O in a thread cannot be cancelled when the timeout fires, so a wedged model holds a thread until the socket times out, and there is no connection reuse | **REJECT.** Cancellation is a requirement (step-17 §3.13, "a wedged model blocks a seat") |

**Decision (D-P5-4).** Depend on **no provider SDK**. Our adapter speaks the OpenAI-compatible schema
over `httpx2`. It is about 200 lines: request mapping, response parsing, error mapping and no retries.
The interface (`ModelBackend`) is ours, the wire schema is the industry's, and the HTTP client is
reused. What we own is the mapping, which is exactly the part that must refuse implicit configuration.

### 4.2b Reading a `.env`-style key file

| Candidate | Licence (verified) | Maturity (verified) | Fit | Verdict |
| --- | --- | --- | --- | --- |
| **`python-dotenv`** | BSD-3-Clause | 1.2.4 (changelog 2026-10-01); `requires-python >=3.10`; **no runtime dependencies** (`click` only for the `cli` extra) | `dotenv_values(path, interpolate=False)` returns a mapping and **does not touch `os.environ`**. It handles quoting, `export` prefixes, comments, multiline values and CRLF: the format details a hand parser gets wrong. `load_dotenv`, which mutates the environment, is never called | **REUSE** (`DEP-32`). One call site, in `secrets.py` |
| **Our own parser** | n/a | about 40 lines | It would re-implement the quoting and escaping rules, and its corner cases would be ours to find. `REUSE_POLICY.md` §12: "writing it ourselves feels cleaner" is not a reason | **REJECT** |
| **`pydantic-settings`** (`.env` support) | MIT | maintained | It loads settings classes from the environment and `.env` files, with the environment taking priority. But it binds every field to environment variables, which is a configuration model we do not want: our file is TOML, and only key **values** come from the environment | **REJECT**: a framework for what one function call does |
| **`environs`** | MIT | maintained | It wraps `python-dotenv` and adds parsing; it mutates `os.environ` by default | **REJECT** |
| **Process environment only** (no file) | n/a | — | The operator asked for `.env`-style configuration explicitly | **REJECT** as the only path; kept as one of the two sources |

### 4.3 Recording and replay

| Candidate | Licence (verified) | Maturity (verified) | N | D | V | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| **vcrpy** (HTTP cassettes) | MIT | 8.3.0, 2026-07-04; depends on `PyYAML`, `wrapt` | ✗ cassettes hold provider wire formats, URLs and headers, so a key must be scrubbed, and a cassette recorded against Ollama does not replay under llama.cpp. `AC-4`'s replay test becomes impossible | ✓ by its match rules | ✓ | **REJECT for the cognition recorder** (step-17 §4.5 stands). It patches `httpx`; support for `httpx2` was not found (search, 2026-10-09; *not verified* either way). It is also not needed for adapter tests: a mock transport tests the mapping without recording anything |
| **pytest-recording** | MIT | 0.14.0, 2026-10-01; wraps vcrpy | as vcrpy | ✓ | ✓ | **REJECT**, as above. Its `--block-network` duplicates pytest-socket, which we already run |
| **respx** | BSD-3-Clause | maintained; requires `httpx` (secondary source) | ✗ HTTP-level | ✓ | ✓ | **REJECT.** It mocks `httpx`, not `httpx2`, and the client's own mock transport covers it |
| **LiteLLM caching** | MIT | — | ✗ the key includes the provider-prefixed model | ~ | ✓ | **REJECT** |
| **inline-snapshot** | MIT | maintained | ✓ | ✓ | ✓ | **REFERENCE ONLY.** Its review-the-diff workflow is the model for re-recording |
| **Our own interface-level cassette** | n/a | n/a | ✓ keyed on the provider-neutral request | ✓ strict, ordered per key | ✓ | **CHOSEN** (`ARC-58`). JSON Lines, recorded above HTTP: no headers, keys or provider formats. One cassette serves every backend |

**Determinism and redaction, the two properties the brief names.**

- **Determinism.** The key is `sha256` over canonical JSON with no floats. Replay is ordered per key, so
  two identical requests replay their two recorded completions in order. A third is a miss, never a
  reuse (D-P5-6, D-P5-7).
- **Redaction.** It holds by construction, not by scrubbing. The recorder sees only `CompletionRequest`,
  `Completion` and a fixed metadata record. None of them has a field that can hold a header, a URL or a
  key. A planted fake key is scanned for in every artefact (AP5-9).

### 4.4 Local model choices (for the spike and the default; nothing downloaded)

Sizes are the Ollama library's download sizes (default 4-bit builds), read 2026-10-09. Licences are the
Hugging Face model metadata's licence tags. The criterion for "modest" machines is 8 GB RAM, CPU or a
small integrated GPU. For the operator's Mac (Apple silicon), unified memory of 16 GB or more is assumed:
*not verified*, and the spike records it.

| Model (Ollama tag) | Download | Context | Licence (verified) | Redistribution and outputs | Fits |
| --- | --- | --- | --- | --- | --- |
| **`qwen3.5:4b`** | 3.3–4.0 GB | 256K | **Apache-2.0** (`Qwen/Qwen3.5-4B`) | Permissive; no restriction on outputs; weights redistributable with the licence and notices | 8 GB machines; any Apple-silicon Mac |
| `qwen3.5:9b` | 6.6–7.6 GB | 256K | Apache-2.0 | as above | 16 GB; the operator's Mac |
| `qwen3.5:2b` | 2.7–3.1 GB | 256K | Apache-2.0 | as above | very small machines; quality *not verified* |
| `gemma4:e4b` | 6.6–9.5 GB | 128K | **Apache-2.0** (`google/gemma-4-E4B-it`) — a change from Gemma 3's custom "Gemma" terms | Permissive | 16 GB |
| `gemma4:12b` | 7.7–8.0 GB | 256K | Apache-2.0 | Permissive | 16 GB; the operator's Mac |
| `ministral-3` 3B / 8B | Ollama tag *not verified* | — | Apache-2.0 (`mistralai/Ministral-3-*-Instruct-2512`) | Permissive | 8–16 GB, if the spike finds a tag |
| `phi4-mini` (3.8B) | *not verified* | — | **MIT** (`microsoft/Phi-4-mini-instruct`) | Permissive | 8 GB |
| `qwen3:4b` / `qwen3:8b` | 2.5 GB / 5.2 GB | 256K / 40K | Apache-2.0 (Qwen3 family; *not re-verified* this session) | Permissive | the previous generation; fallback |
| `llama3.2:3b` | 2.0 GB | 128K | **Llama 3.2 Community License** (gated on Hugging Face, manual approval) | Redistribution needs the licence, the "Built with Llama" attribution and the Acceptable Use Policy passed on. Using outputs to improve another model has naming conditions. Above 700 M monthly users, a separate licence | Excluded from the **default** for its obligations; usable by a user who accepts them |
| `gemma3:*` | — | — | "Gemma" terms (custom; prohibited-use policy passed through) | Restrictions follow derivatives | Superseded by `gemma4`; excluded |

**What redistribution means for MineWorld.** MineWorld never ships weights; a user's Ollama downloads
them. What the repository does ship is **outputs**: the cassettes P6 and P7 commit. Apache-2.0 and MIT
place no conditions on outputs. The Llama licence does. A model under Apache-2.0 or MIT therefore keeps
committed cassettes free of third-party licence terms. That is why the default is restricted to them.

**Ruled (QP5-2, operator, 2026-10-09): start from `qwen3.5:4b`; the spike settles the final choice.**
The recommendation it accepted: default `social` binding `qwen3.5:4b` (Apache-2.0,
about 3.3–4.0 GB). It is the one candidate that fits an 8 GB Windows or Linux machine and still offers a
long context. The spike also measures `qwen3.5:9b` and `gemma4:12b` (both Apache-2.0) as the
recommended choice for 16 GB machines such as the operator's Mac. The default is the smallest
candidate that meets the criteria fixed in §7 (AP5-S). The model name appears only in example
configuration and in cassette metadata, never in a key or a contract.

---

## 5. Design

### 5.1 Package layout

```text
pyproject.toml                      members = ["sdk/python", "cognition/lm-controller"];
                                    [tool.pyright] include gains "cognition/lm-controller"
uv.lock                             one lock (P3 D-P3-4), now with httpx2 and its dependencies
.gitattributes                      + *.jsonl text eol=lf
cognition/lm-controller/
  pyproject.toml                    mineworld-cognition; requires-python >=3.12;
                                    deps: mineworld-sdk (workspace), pydantic, httpx2
                                    dev: pytest, pytest-socket, ruff, pyright[nodejs] (as sdk/python)
  README.md                         human orientation, under 40 lines
  src/mineworld_cognition/
    __init__.py                     the public names
    py.typed
    backend/
      __init__.py
      model.py                      ModelBackend, CompletionRequest, Message, Role, Purpose, Sampling,
                                    Completion, Finish, Usage, BackendFailure (typed reasons)
      canonical.py                  canonical_json(request) -> bytes; cassette_key(request) -> CassetteKey;
                                    KEY_SCHEME = 1
      scripted.py                   ScriptedBackend(fn: Callable[[CompletionRequest], Completion])
      openai_compatible.py          OpenAICompatibleBackend   (the ONLY module that imports httpx2)
      registry.py                   KIND → factory; today: "openai-compatible"
      providers.py                  the provider preset table (D-P5-15; data only, no code path)
    record.py                       Cassette (read/write), CassetteEntry, EntryMeta, ReplayBackend,
                                    RecordingBackend, CassetteMiss, CassetteFormatError
    budget.py                       BudgetPolicy, Ledger, MemoryLedger, SqliteLedger, Clock, SystemClock,
                                    InFlightLimiter, BudgetRefusal
    gateway.py                      ModelGateway, GateOutcome = Completed | Refused | Failed; Router; Tier
    config.py                       CognitionConfig, BackendConfig, TierBindings, BudgetConfig,
                                    RecordingConfig, SecretsConfig, Mode; load(path) -> CognitionConfig;
                                    ConfigError
    secrets.py                      Secret; resolve_key(name, env_file) (D-P5-9; the only python-dotenv
                                    call site); the env_file location and permission checks (D-P5-14)
  examples/
    hosted.toml.example             OpenAI, DeepSeek, Zhipu GLM as openai-compatible backends (key_env only)
    .env.example                    OPENAI_API_KEY=, DEEPSEEK_API_KEY=, ZHIPUAI_API_KEY= (names, no values)
  tests/
    conftest.py                     shared fixtures; no network
    cassettes/                      small cassettes written by tests' RecordingBackend over ScriptedBackend
    test_model_and_key.py           AP5-1, AP5-2
    test_record_replay.py           AP5-3, AP5-4, AP5-5
    test_openai_compatible.py       AP5-6 (unit half), mapping and error tests over a mock transport;
                                    one loopback stub-server test (real sockets, test-owned port)
    test_budget.py                  AP5-7
    test_config_router.py           AP5-8, AP5-9 (config half)
    test_structural_isolation.py    AP5-6 (httpx2 never imported in replay/scripted), AP5-10
    test_provider_scan.py           AP5-10's scan (IC-10, P5 half)
    test_secrets.py                 AP5-9
  tools/
    model_spike.py                  the operator-run spike (C7); not part of the package; never in CI
    spike_scenarios.json            a fixed set of synthetic decision prompts
```

No module is expected past about 300 lines. `openai_compatible.py` and `record.py` are the largest.

### 5.2 Decisions (D-P5-1 … D-P5-14)

| ID | Decision | Reason |
| --- | --- | --- |
| **D-P5-1** | P5 creates `cognition/lm-controller` (`mineworld-cognition`) as the second workspace member. P4, which waits for S11-C, builds on it (R-P4-1). | Step-17 §9 says P4 "needs P3's package skeleton only". The order changed: P5 now lands first (§15.5), so the skeleton arrives with its first real content rather than empty. |
| **D-P5-2** | The interface is step-17 §3.7.1, made exact and float-free. `CompletionRequest { purpose: Purpose ("decide" \| "summarize"), messages: tuple[Message, …] (role "system" \| "user" \| "assistant", text: str), output_schema: JsonValue \| None, sampling: Sampling { temperature_milli: int 0..2000, max_output_tokens: int 1..32768, seed: int (u32) \| None } }`. `Completion { text: str, finish: "complete" \| "length" \| "refused", usage: Usage { input_tokens, output_tokens: int ≥ 0, estimated: bool } }`. Every model is strict, frozen and `extra="forbid"`, as the SDK's are. Transport and server failures are **not** completions. They are `BackendFailure` with a typed reason: `unreachable`, `timeout`, `http_status(code)`, `malformed_response`, `unauthorized`. | Step-17 had `finish: "error"`. An error is not something a model said, so recording it as a completion would put transport noise into cassettes. A typed failure lets P6 fall back by cause (§3.10.3). `temperature_milli` keeps floats out of the key (D-P5-6). No model, provider, URL, tier or key field exists (I-10). |
| **D-P5-3** | **Routing:** `Tier` is `ordinary_decision \| social \| major_decision \| summarize` (`ARCHITECTURE.md` §9.1). `routine` is never a tier: it is deterministic policy (I-15). `[tiers]` binds a tier to a backend **name** or to `"none"`. `Router.for_tier(tier) -> ModelGateway \| None`; `None` means "no model for this purpose", and P6 falls back. The binding is resolved **before** a request is built, so the request and the key do not change when the operator changes provider. | Step-17 §3.7.3. `AC-4`: the world and the request are independent of the binding. |
| **D-P5-4** | **No provider SDK.** `OpenAICompatibleBackend` speaks `POST {base_url}/chat/completions` over `httpx2.AsyncClient`. Rules: **(a)** it is constructed only from a `BackendConfig` and reads no environment variable except the one named by `key_env`, and only when that name is non-empty; **(b)** `base_url` is required, with no default; **(c)** no retries: one call is one ledger entry; **(d)** the `Authorization` header is set only when a key was read, and is never logged; **(e)** `httpx2` is imported by this module only. | §4.2: the `openai` SDK's implicit environment reads and hosted default contradict QS10-19 and I-16. Owning the mapping is the way to own those refusals. |
| **D-P5-5** | **Modes**, process-wide, from `[recording] mode`: `live` (backend; no recording), `record` (backend; every completed call appended), `replay` (cassette only; **no backend object is constructed**, and the adapter module is never imported), `scripted` (a `ScriptedBackend` supplied in code by a test; no cassette). Replay and scripted are the only modes tests use. | Step-17 §3.11.3. A test that can silently reach a model can depend on one (§24). "No backend constructed" is stronger than a network guard, because the guard allows loopback, where a user's Ollama listens (§3, DV-P3-2). |
| **D-P5-6** | **The key.** `cassette_key = sha256(canonical_json(request))`, as lowercase hexadecimal. `canonical_json`: UTF-8; `sort_keys`; separators `(",", ":")`; `ensure_ascii=False`; no Unicode normalization (text bytes are the request); **any float anywhere raises `KeyMaterialError`**, including floats inside `output_schema`. The canonical object is `{"key_scheme": 1, "request": …}`, so a future change of canonicalization is an explicit scheme bump, never a silent collision. A golden test pins the key of one literal request as a hexadecimal literal. It runs on all three platforms. | Step-17 §3.11.1. Float formatting is the classic source of cross-platform and cross-version key drift. The SDK's `JsonValue` admits floats, so the refusal must be explicit. |
| **D-P5-7** | **The cassette format.** JSON Lines, UTF-8, LF. The first line is a header: `{"cassette": "mineworld-cognition", "format": 1, "key_scheme": 1}`. Each later line is an entry: `{"key", "request", "completion", "meta": {"binding", "model", "recorded_at", "latency_ms"}}`. `binding` is the operator's backend **name** (for example `local`); `model` is the configured model name; `recorded_at` is RFC 3339 UTC; `latency_ms` is an int. **Never in a cassette:** a URL, a header, a key, `key_env`, or a `BackendFailure`. **Replay:** entries are grouped by key in file order. The n-th call with a key returns the n-th entry. Call n+1 raises `CassetteMiss(kind="exhausted")`. A key not present raises `CassetteMiss(kind="absent")` with the nearest recorded request (same purpose, fewest differing top-level fields) and the first differing JSON path. A recorded entry whose `key` does not equal the key recomputed from its `request` raises `CassetteFormatError` at load: a hand-edited cassette fails loudly. **Record:** writes `<name>.jsonl.partial`, flushes each line, and on clean close replaces `<name>.jsonl` with `os.replace`. A crash leaves the old cassette intact and a visible `.partial`. Record mode refuses to start when `.partial` exists. | Step-17 §3.11.2, made exact. Ordered per-key replay keeps a scenario that asks the same question twice deterministic. URL and key are excluded because a URL can carry credentials (`user:pass@`) or internal host names. `os.replace` is atomic on all three platforms once the file is closed. |
| **D-P5-8** | **Budgets, under QS10-18.** Cost ceilings are keyed on **wall** time and read only `Clock.now()` (UTC epoch seconds, injected). Defaults, from `ARCHITECTURE.md` §9.1's numbers re-keyed: per seat, `calls_per_wall_hour = 20` and `tokens_per_wall_day = 30 000`, as rolling windows of 3 600 s and 86 400 s; per process, `max_in_flight = 2` and `call_timeout_s = 20`; per backend, `requests_per_minute` unset (an operator ceiling). **Pre-check:** a call is refused before the recorder and the backend when `spent_calls + 1 > calls`, or when `spent_tokens + estimate_input(request) + max_output_tokens > tokens`. `estimate_input` is `ceil(utf8_bytes(messages) / 4)`, deterministic. **Post-charge:** the actual `usage` replaces the reservation; when the backend reports none, the estimate is charged with `estimated = true`. A `BackendFailure` charges the call but no tokens. **Ledgers:** `MemoryLedger` for tests; `SqliteLedger` (standard-library `sqlite3`, one table `budget_ledger(seat, at, calls, tokens, estimated)`, at a path from configuration) so a restart neither resets nor double-counts. The **context bound** (simulated time, I-13) is not here: it is P4's memory ceiling and P6's context assembly. | QS10-18 and step-17 §3.10. A rolling wall window bounds GPU time and money whatever the world's `time_scale`. Two ledger implementations exist from the start because tests need one and restart durability needs the other (as step-17 §3.16.3 allows for `MemoryStore`). The ledger path is configured, never inside a world save (`INV-4`). |
| **D-P5-9** | **Keys: one indirection, two sources.** `BackendConfig.key_env: str` holds a variable **name** matching `^[A-Z_][A-Z0-9_]*$`, or is empty.<br>**Resolution**, only when the backend is built in `live` or `record` mode:<br>(1) the process environment;<br>(2) else, when `[secrets] env_file` is set, that file through `dotenv_values(path, interpolate=False)`.<br>The process environment wins, as `python-dotenv`'s own default (`override=False`) does: an operator's shell can override a file without editing it.<br>**Never:**<br>- `load_dotenv`; a file's values never enter `os.environ`, so they cannot leak into a child process (P5b's CLIs) or into another library's implicit read;<br>- interpolation;<br>- a default file location. MineWorld reads no `.env` it was not pointed at; in particular, not one in the working directory, and not `~/.config/mineworld/secrets.env`, unless the operator names it.<br>**Failure.** A non-empty name found in neither source raises `ConfigError("backend 'deepseek': DEEPSEEK_API_KEY is not set in the environment or in the configured env_file")`. It names the variable and the file path, never a value. The value is held in a `Secret` whose `repr` and `str` are `Secret(<redacted>)`. It is never logged, recorded, put in an exception, or written to the ledger. | Step-17 §3.7.4; QS10-19; I-16; the SDK's `Invite` precedent (D-P3-9); the operator's 2026-10-09 requirement ("api通过.env 之类的配置"). Not mutating `os.environ` is the property that keeps a key file from becoming ambient credentials for every child process. |
| **D-P5-10** | **Typed backend options, no free-form map.** `BackendConfig { kind: "openai-compatible", base_url: HttpUrl (http or https; no userinfo), model: str, key_env: str, structured_output: "json_schema" \| "json_object" \| "none" (default "json_schema"), reasoning: "unset" \| "off" \| "low" \| "medium" \| "high" (default "unset"; mapped to `reasoning_effort` when set), temperature: "send" \| "omit" (default "send"), request_timeout_s: int \| None }`. A `base_url` with userinfo is refused. A non-loopback `base_url` must be `https`, so a key never travels in clear text to a remote host. | `CLAUDE.md` §4 rule 7: no `dict[str, Any]` at a boundary. Step-17's `extra` is replaced by the three options the backends in §4.1 and §4.1b actually differ on. `reasoning: off` matters for the thinking-capable Qwen 3.5 and Gemma 4 families, whose latency it governs (A-4). Whether each server honours it is spike evidence. |
| **D-P5-11** | **CI and checks cover the new member, with one pytest invocation per member.** `PYTHON_STATIC`'s ruff commands gain `cognition/lm-controller`; pyright covers it through the root `include`. The `python` layer runs `uv run --locked pytest sdk/python` **and** `uv run --locked pytest cognition/lm-controller` as two commands; `python-smoke` likewise. The member's own `pyproject.toml` carries the network guard in `addopts`. `standards.md` follows. No YAML changes. | A single `pytest sdk/python cognition/lm-controller` takes its rootdir and ini file from the **common ancestor**, the repository root, whose `pyproject.toml` has no pytest section. Both packages' `addopts`, and with them the network guard, would be dropped silently. AP5-10 holds this. |
| **D-P5-12** | **Live tests are opt-in and never in CI.** A `live_model` marker is registered. Every live test is deselected by the package's `addopts` (`-m "not live_model"`), and it needs `MINEWORLD_LIVE_BASE_URL` naming a **loopback** URL. A non-loopback URL fails the test rather than calling a hosted API. Live tests fail, never skip, when selected without the variable (D-P3-10's rule). `ci_layer.py` never selects the marker. | QS10-19 (no hosted API in any gate), I-11, `ENGINEERING_STANDARDS.md` §24 ("maintain optional live-model integration tests when useful"). |
| **D-P5-13** | **Every platform.** Paths through `pathlib`. Cassettes are written with `newline="\n"` and read as UTF-8 tolerating CRLF, and `.gitattributes` keeps them LF. `os.replace` is called only after the file is closed (Windows). SQLite connections are closed before a ledger file is moved or deleted (Windows file locking). Timeouts use `asyncio.timeout` (3.11+) and the monotonic clock; budget windows use the wall clock. No POSIX signal, no Unix socket. The loopback stub server binds `127.0.0.1:0`. The key's golden literal is checked on all three CI legs. | Operator requirement 2026-10-08; P3 D-P3-11; step-17 §15.8 (P5). |

**D-P5-15 — Provider presets (amendment, 2026-10-09).**

- **Where:** `backend/providers.py` holds a frozen, typed table: `ProviderPreset { name, base_url:
  HttpsUrl | None, key_env, structured_output, temperature, reasoning }`, one row per §4.1c entry.
- **How a user selects one:** `BackendConfig` gains one optional field, `preset: PresetName`, a closed
  literal of the table's names. When it is set, every field the user left out is filled from the
  preset. A field the user sets wins.
- **Validation:**
  - `model` is always required;
  - a preset whose `base_url` is `None` (`dashscope`) requires the user's `base_url`;
  - the result is validated exactly as a hand-written `BackendConfig` is.
- **Nothing downstream changes:** `OpenAICompatibleBackend` never sees the preset name; it receives the
  resolved `BackendConfig`. The preset name is not in `CompletionRequest`, not in the cassette key, and
  not in cassette metadata (`meta.binding` is the user's backend name).
- **Reason:** the operator's requirement, with no new code path. A preset is data, and adding a provider
  is one table row plus one README row (`CLAUDE.md` §4 rule 5).

**D-P5-14 — Where configuration and keys live: per user, never per world.**

- `cognition.toml` and any `.env` file belong to the **operator who runs cognition**: a person on
  their own machine, or a server operator. Credentials belong to the server operator and never appear
  in a World Pack (`ARCHITECTURE.md` §9.2). A World Pack names no backend (`AC-4`).
- Two consequences:
  - `config.load` refuses a configuration file, or an `env_file`, whose resolved path lies inside a
    directory containing a `world.yaml`. A key file placed in a world, which travels with the world,
    is a `ConfigError`;
  - the same world can be played by two users with two different backends.
- Paths are explicit: `--config FILE` (P6), and `env_file` inside it, resolved relative to the
  configuration file with `pathlib`. P5a ships no implicit default location. The README suggests one
  per platform:
  - macOS: `~/Library/Application Support/MineWorld/`;
  - Linux: `$XDG_CONFIG_HOME/mineworld/`, falling back to `~/.config/mineworld/`;
  - Windows: `%APPDATA%\MineWorld\`.
- On macOS and Linux, an `env_file` readable by group or others (`mode & 0o077`) is refused with a
  message suggesting `chmod 600`, as `ssh` refuses a key. On Windows, POSIX modes do not apply. The
  check is skipped there, and the README says to keep the file under the user's profile.
- Reason: the operator ruled "per world or per user — say which". Per user is the only answer that
  keeps `AC-4` and keeps secrets out of shareable content.

### 5.3 The seam and one call, precisely

```text
P6 (later):   gateway = router.for_tier(Tier.social)            # None → no model; P6 falls back
              outcome = await gateway.complete(seat, request)   # seat: EntityKey

ModelGateway.complete(seat, request) -> GateOutcome
  1. key material check:  canonical_json(request)               # KeyMaterialError → raised (a bug)
  2. budget pre-check:    ledger.window(seat, now) vs policy     # over → Refused(BudgetRefusal(...))
                                                                 #   nothing below runs (P5-2)
  3. in-flight slot:      async with limiter                     # FIFO; bounded by max_in_flight
  4. within asyncio.timeout(call_timeout_s):
       backend.complete(request)                                 # backend = Replay | Recording(inner)
                                                                 #   | inner (live) | Scripted
         ReplayBackend:    cassette lookup; miss → CassetteMiss (raised, never Failed, never live)
         RecordingBackend: inner.complete → append entry → return
  5. charge:              ledger.charge(seat, now, calls=1, tokens=usage or estimate)
  6. return               Completed(completion) | Failed(BackendFailure) | Failed(timeout)

GateOutcome = Completed(Completion) | Refused(BudgetRefusal) | Failed(BackendFailure)
CassetteMiss is an exception, not an outcome: a miss is a broken test or a stale cassette, and must
stop the run (AP5-3), never become a fallback a controller quietly absorbs.
```

### 5.4 The configuration P5 owns

```toml
# cognition.toml — the operator's file, never inside worlds/. P6 adds server, seats, store.
[recording]
mode     = "replay"                       # live | record | replay | scripted
cassette = "cognition/lm-controller/tests/cassettes/example.jsonl"

[tiers]
social    = "local"
summarize = "none"

[budgets]
ledger              = "./cognition-state/budget.sqlite"   # resolved with pathlib, relative to the file
calls_per_wall_hour = 20
tokens_per_wall_day = 30000
max_in_flight       = 2
call_timeout_s      = 20

[backends.local]
kind              = "openai-compatible"
base_url          = "http://127.0.0.1:11434/v1"
model             = "qwen3.5:4b"          # example; the default is QP5-2's operator decision
key_env           = ""                    # Ollama needs none
structured_output = "json_schema"
reasoning         = "off"

# A user's own hosted API (examples/hosted.toml.example). Never used by a test, CI or an agent.
[secrets]
env_file = "mineworld.env"                # optional; relative to this file; never inside a world

[backends.deepseek]
kind              = "openai-compatible"
base_url          = "https://api.deepseek.com"
model             = "deepseek-flash"
key_env           = "DEEPSEEK_API_KEY"    # the NAME; the value comes from the environment or env_file
structured_output = "json_object"
```

- The file is parsed with `tomllib` into strict Pydantic models. An unknown key is a `ConfigError`
  naming its table and key.
- A tier bound to an undefined backend name is a `ConfigError`.
- `mode = "replay"` with no `cassette` is a `ConfigError`.
- In `replay` and `scripted` modes, backend sections are validated but **never instantiated** (D-P5-5).

### 5.5 CI story: cassettes only, no live model

```text
fast        PYTHON_STATIC over sdk/python and cognition/lm-controller (ruff check, ruff format --check,
            pyright strict); P3 measured the static checks at 5.6 s, and the new member adds seconds
            (the 60 s rule of QP3-3 still holds; C6 re-measures)
python      as P3, plus `uv run --locked pytest cognition/lm-controller` (a second command, D-P5-11), on
            ubuntu-24.04, windows-2025, macos-15. Every cognition test is scripted or replay. The
            live_model marker is deselected by addopts
models      none, anywhere: no Ollama, no key, no hosted endpoint, no download. The cassettes in
            tests/cassettes/ are written by RecordingBackend over ScriptedBackend, so they are
            reproducible by a test and need no model to regenerate
```

---

## 6. Test ownership (test rules §26)

```text
STATIC     ruff; pyright strict: NewType mixing, the GateOutcome union handled exhaustively (match with
           assert_never), no Any. No unit test re-proves these
UNIT       key golden and float refusal (AP5-1, AP5-2); cassette format, ordering, misses (AP5-3, AP5-4);
           the adapter's mapping and error mapping over a mock transport (AP5-6); budget windows and
           ordering of the gate (AP5-7); config validation (AP5-8, AP5-9)
INTEGRATION
           the adapter against a loopback stub HTTP server over real sockets (test-owned, ephemeral
           port); replay mode in a child interpreter proving httpx2 is never imported (AP5-6); a record →
           replay round trip through ModelGateway with SqliteLedger across a simulated restart (AP5-7)
GATE 1     NOT REQUIRED for tests: no live model in any test. The operator-run spike (C7) is the
           real-model evidence. It is recorded as NOT RUN until the operator runs it, and it blocks P6's
           freeze, not P5's review (QP5-3)
GATE 2     N/A: P5 touches no server, client or world. The real-binary path is P6's
CI IMPACT  one extra pytest command in `python`; the static checks over one more path. Rust layers
           unchanged
```

---

## 7. Adversarial criteria (fixed now, before anything is measured)

Bounds are literals from the requirement, never from the implementation (`ARC-23`). Each criterion names
the mutation that must turn it red. A criterion whose mutation stays green has tested nothing.

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| **AP5-1** Key is stable and provider-free | (a) The key of one literal `CompletionRequest` equals a hexadecimal literal committed in the test. The literal is computed once in C2 and recorded in the ledger before any other test is written. It passes on all three CI legs. (b) Two `ModelGateway`s over two backend configurations that differ in name, `base_url` and `model` produce the same key for the same request. (c) `CompletionRequest` has no field whose name or value can carry a model, URL or binding: the scan in AP5-10 covers its schema. | (a) Drop `sort_keys`: the literal differs. (b) Add the binding's `model` to the key material: the two keys differ. |
| **AP5-2** No floats in key material | `canonical_json` raises `KeyMaterialError` for a float at the top level, inside `messages`, and inside `output_schema` (`{"minimum": 0.5}`), each naming the JSON path. | Serialize floats with `repr` instead of refusing: the three cases pass silently, and the test fails. |
| **AP5-3** A miss fails loudly, never silently calls a live model | In `replay` mode, with the backend `local` configured at `http://127.0.0.1:<port>/v1`, where `<port>` is a **listening socket the test owns and counts accepts on**: (i) a request absent from the cassette raises `CassetteMiss(kind="absent")`, whose message names the key, the nearest recorded request and the first differing JSON path; (ii) the listener accepted **0** connections; (iii) the gateway's ledger recorded no tokens. The loopback port is the point: pytest-socket allows loopback, so only this structure catches a fall-through to a local model. | Implement a miss as "fall through to the live backend": the listener accepts one connection, and (ii) fails. Implement a miss as `Failed(...)`: (i) fails, because no exception was raised. |
| **AP5-4** Replay is ordered and exact | A cassette recorded with the same request twice and two different completions replays them in recorded order. A third identical call raises `CassetteMiss(kind="exhausted")`. A hand-edited entry whose `request` no longer hashes to its `key` raises `CassetteFormatError` at load, naming the line. A cassette with `format: 2` is refused by name. | Return the first entry for every repeat: the second call returns the first completion. Skip the key check at load: the edited entry loads. |
| **AP5-5** Record is atomic and complete | `RecordingBackend` over `ScriptedBackend`, three calls, closed cleanly: the cassette has the header and 3 entries, and replays all three. Interrupted after two calls (the test raises inside the third): the previous cassette is byte-identical to before, and `.partial` exists. Starting record again with `.partial` present fails with a message naming it. | Write the target file directly instead of `.partial`: the interrupted case leaves a truncated cassette. |
| **AP5-6** Replay and scripted construct no network client | (a) A child interpreter loads a `replay`-mode configuration through `config.load`, builds the router, and runs one replayed call. Afterwards `"httpx2" not in sys.modules` and `"mineworld_cognition.backend.openai_compatible" not in sys.modules`. (b) The adapter, over a mock transport, maps a request to the exact JSON body (`model`, `messages`, `temperature` = `temperature_milli / 1000`, `max_tokens`, `seed`, and `response_format` by `structured_output`), sends no `Authorization` header when `key_env` is empty, and maps HTTP 401 → `unauthorized`, 500 → `http_status(500)`, a non-JSON body → `malformed_response`, a closed port → `unreachable`, and a stalled server → `timeout` within `request_timeout_s` + 1 s. (c) The adapter is called once per `complete`: a 503 is not retried (a mock counts requests). | (a) Import `openai_compatible` at the top of `backend/__init__.py`: the module list contains it. (c) Add one retry: the count is 2. |
| **AP5-7** Budgets: wall time, before everything, durable | With a `FakeClock`: (i) the 21st call within 3 600 wall seconds for one seat returns `Refused(calls_per_wall_hour)`, and the backend's call counter is unchanged (P5-2); a second seat is unaffected. (ii) After the clock advances 3 601 s, the call is allowed. (iii) A request whose estimate plus `max_output_tokens` would exceed 30 000 tokens in the 24-hour window is refused **before** the call. (iv) `SqliteLedger`: 20 calls, the ledger closed and reopened (a restart), and the 21st call refused. (v) No budget code reads simulated time: `budget.py` and `gateway.py` import nothing from `mineworld_sdk.wire.contract`, and no function there takes a `WorldTime` (a static check in the test, plus pyright). (vi) A third concurrent call waits while two are in flight (`max_in_flight = 2`), and is served FIFO. (vii) A scripted backend that sleeps past `call_timeout_s` (the test sets 1 s) gives `Failed(timeout)` within 2 s, and the call is charged. | Check the budget after calling the backend: (i)'s backend counter rises. Use `MemoryLedger` for the `SqliteLedger` path: (iv) allows the 21st. Key the window on an observation's `at`: (v) fails. |
| **AP5-8** `AC-4`, the seam half | One cassette is recorded through binding `local` (`ScriptedBackend` standing in for the inner backend). It is replayed under a configuration in which the binding is renamed `elsewhere` and repointed to another `base_url` and `model`. Every call hits, the keys are identical, and the completions are identical. The scenario half (`worlds/` byte-identical, submitted requests identical) is P7's IC-6. | Put the binding name in the key: every replay misses. |
| **AP5-9** No secret in any artefact | Run twice, once per source. A fake key `MWTEST-<32 random hex>` is placed (1) in the process environment, and (2) only in a temporary `env_file` (mode 0600 on POSIX), under the name `key_env` names. The scan also covers pytest's captured stdout and stderr for the test, which is what CI prints. A record session runs over a mock transport that **echoes request headers into its error body**, plus one 401. The fake key appears in no cassette line, no ledger row, no captured log record (DEBUG, every logger), no exception's `str` or `repr`, and no `repr` of any config, backend or gateway. The mock transport itself must have received it as `Authorization: Bearer …`, which proves the key was used. An unset `key_env` raises a `ConfigError` naming the variable, with no value. The package's source has no reference to `secrets.env` or `~/.config/mineworld` (scan). | Include the response body of a 401 in `BackendFailure`'s message: the echoed header leaks, and the scan finds it. |
| **AP5-10** No provider concept outside the adapter; the guard is active | (a) A scan of `cognition/lm-controller/src` **excluding** `backend/openai_compatible.py`, `backend/registry.py`, `backend/providers.py` (amendment, 2026-10-09) and `config.py`, of every cassette's `key` and `request`, and of `sdk/python/src`, finds none of a fixed list: `openai`, `ollama`, `anthropic`, `llama`, `qwen`, `gemma`, `mistral`, `phi`, `vllm`, `lmstudio`, `api_key`, `http://`, `https://`, `11434`. It reports file and line. (b) The cognition suite's pytest configuration contains `--allow-hosts=127.0.0.1,::1`, and a test connecting `httpx2` to `192.0.2.1:80` fails within 1 s with pytest-socket's error type. (c) `ci_layer.py --list python` shows two pytest commands, one per member. | (a) Plant `"ollama"` in `gateway.py`: the scan fails at that file and line. (b) Remove the guard from the member's `addopts`: the TEST-NET test times out instead. (c) Merge the two pytest commands into one: (c) fails. |
| **AP5-11** Scope | `git diff --stat <base>..HEAD` touches only `cognition/lm-controller/**`, `pyproject.toml`, `uv.lock`, `.gitattributes`, `.gitignore`, `.structured-coding/**`, `docs/DECISIONS.md`, `docs/ARCHITECTURE.md`, and `scripts/ci_layer.py` (commands in `PYTHON_STATIC`, `python`, `python-smoke` only). No `*.rs`, nothing under `sdk/python/`, `worlds/` or `.github/`. | — (a diff gate) |
| **AP5-12** Every platform | The whole cognition suite, including AP5-1(a)'s literal, AP5-5's `os.replace`, AP5-7(iv)'s reopen and AP5-3's listener, passes on `ubuntu-24.04`, `windows-2025` and `macos-15` in the PR's CI run. | Write cassettes with the platform newline: AP5-4's load of a committed cassette and the LF check fail on Windows. Replace while the file is open: Windows raises `PermissionError`. |
| **AP5-13** Key files stay the user's, and stay out of the environment | (a) After `resolve_key` reads a key from an `env_file`, `os.environ` is unchanged (compared as a whole before and after), and the key is not in it. (b) A variable present in both sources resolves to the process environment's value. (c) An `env_file` under a directory holding a `world.yaml` (a temporary copy of the layout) raises `ConfigError` naming the path; so does a `cognition.toml` there. (d) On macOS and Linux, an `env_file` with mode 0644 is refused, naming `chmod 600`; with 0600 it is accepted. On Windows, the check is recorded as not applicable, and the test asserts that it is skipped by platform, not silently. (e) `git check-ignore` reports `.env`, `.env.local`, `prod.env` and `secrets.env` as ignored, and `.env.example` as **not** ignored. (f) No module calls `load_dotenv` (a source scan), and `python-dotenv` is imported only by `secrets.py`. | (a) Use `load_dotenv(path)`: `os.environ` gains the key. (e) Drop `.env.*` from `.gitignore`: `.env.local` is reported as not ignored. |
| **AP5-14** The preset table (amendment, 2026-10-09) | Over every row of `providers.py`: (a) `base_url` is `None` or an `https` URL with no userinfo, no query and no fragment; (b) no row carries a key: `key_env` matches `^[A-Z_][A-Z0-9_]*$`, and no field value matches a key-like pattern (`sk-`, `xai-`, `gsk_`, `AIza`, or 20+ base64 characters); (c) the cassette key of one literal request is identical under a configuration with `preset = "openai"`, one with `preset = "xai"`, and one hand-written with no preset, and it equals AP5-1's literal; (d) a user-set field overrides the preset, shown for `structured_output`; (e) `preset = "dashscope"` without `base_url` raises `ConfigError` naming the field; (f) the README's provider table lists exactly the table's preset names and `key_env` names (the test parses the Markdown table). | (a) Change one preset to `http://`: (a) fails, naming the row. (c) Add the preset name to the request: the keys differ. (f) Add a preset without a README row: (f) names it. |
| **AP5-S** The spike's criteria (operator-run; decide the default) | Over the fixed `spike_scenarios.json` (40 synthetic decisions, defined in C7 before any run), per candidate model, on the operator's machine: schema-valid on the **first** attempt ≥ 95 % (≥ 38/40); 95th-percentile wall latency ≤ 15 s; `reasoning = "off"` honoured or its absence recorded; Ollama through `/v1` (A-3 recorded as PASS or FAIL). The default is the **smallest** model meeting all of them. If none does, the result goes to the operator, and the thresholds are not changed (QS10-2's criteria are fixed). | — (an operator measurement. Its integrity rule is that thresholds may not move after a run) |

---

## 8. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-P5-1 | `httpx2` is a young fork (2026). Its API or packaging may change, or it may lack `MockTransport`. | Verified in C1. It is isolated in one module (D-P5-4 (e)). Fallbacks in order: `httpx` 0.28.1 (same API, unmaintained, recorded as a temporary choice) or the test-owned loopback stub for every adapter test. The choice is recorded in `DEP-27` with a revisit trigger. |
| R-P5-2 | Ollama's `/v1` ignores `json_schema` for some models (A-3), so outputs are unconstrained. | P6 validates locally with Pydantic and repairs once, whatever the endpoint does (step-17 §3.5). `structured_output = "json_object"` or `"none"` is a configuration fallback. The native adapter follows on spike evidence (QP5-7). |
| R-P5-3 | Thinking-capable small models (Qwen 3.5, Gemma 4) spend seconds reasoning, which breaks A-4's 15 s p95. | `reasoning = "off"` (D-P5-10), measured by the spike. A server that ignores it is recorded, and the spike measures the latency as it is. |
| R-P5-4 | A prompt change in P6 invalidates every cassette. | Most tests are scripted (step-17 §3.11.3). A miss names the first differing path (D-P5-7). Re-recording from a real model is operator work, and the cassettes P5 commits are written by tests and need no model. |
| R-P5-5 | A wall-time window lets a crash-looping process spend again after each restart. | `SqliteLedger` persists across restarts (AP5-7 (iv)). |
| R-P5-6 | Token estimates differ from real usage, so the pre-check is imprecise. | The pre-check reserves the full `max_output_tokens`, so it errs toward refusal. The post-charge uses the backend's `usage` when reported (llama.cpp and Ollama report it). Estimated charges are flagged. |
| R-P5-7 | The default local model is too weak for natural dialogue, and Milestone D reads worse than the template. | AP5-S's fixed criteria. Quality judgement stays with the operator's run (step-17 R-S10-1). The deterministic fallback is P6's. |
| R-P5-8 | Licence drift: a model family changes licence (Gemma moved from custom terms to Apache-2.0 with Gemma 4). | The default is recorded with its licence tag and the date it was read. The spike report records the licence of every candidate it ran. |
| R-P5-9 | P4 also creates `cognition/lm-controller`, so the two PRs conflict. | D-P5-1 and R-P4-1: P5 owns the skeleton. P4's design, detailed after S11-C, starts from P5's tree. |
| R-P5-10 | Windows: SQLite locks, `os.replace` on an open file, CRLF checkouts. | D-P5-13, AP5-12, `.gitattributes`. |
| R-P5-11 | An agent is tempted to "just run Ollama" to produce a real cassette. | Forbidden by the contract (§11 NEVER). Only the operator runs the spike (QP5-3). No P5 test needs a real cassette. |
| R-P5-12 | A user commits a key: a `.env.local` that `*.env` does not match, or a key pasted into `cognition.toml`. | `.gitignore` gains `.env.*` (AP5-13 (e)). `cognition.toml` has no field that can hold a key value; `key_env` must match a variable-name pattern, so a pasted key (lowercase, `-`) is refused. The README says where to keep files (D-P5-14). |
| R-P5-13 | A hosted service's "OpenAI compatibility" differs in details: DeepSeek's and GLM's `json_schema` support, error bodies, `usage`. | Typed options (`structured_output`, `temperature`), with local validation as the authority (P6). The hosted examples are marked "verify". Behaviour is checked by users, never by CI or an agent. |

---

## 9. Commit plan

Each commit tracks **implementation**, **validation** and **review** separately. `[x]` needs the work
and its evidence. An item that turns out not to apply is `N/A`, with the audited reason. Commands
assume the worktree root, with `uv` and `cargo` on `PATH`.

### C0 — Freeze and contract (Markdown only)

- **Goal.** Record the freeze and fill §11's contract with its sources.
- **Scope.** This document, and a new `handoff-s10-p5.md`.
- [ ] Implementation: the `DESIGN FROZEN` header with its approval reference; the contract's authority
  lines with sources; the handoff initialized; the base commit recorded.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Review: every authority line has a source, and none was widened.
- **Commit boundary.** Documentation only.

### C1 — Specs, decisions and the package skeleton

- **Goal.** The decisions exist before the code (`CLAUDE.md` §2.2), and the empty package locks, lints
  and type-checks in the workspace.
- **Scope.**
  - root `pyproject.toml` (member, pyright `include`) and `uv.lock`;
  - `cognition/lm-controller/{pyproject.toml, README.md, src/mineworld_cognition/__init__.py, py.typed}`,
    with the guard, the `live_model` marker and `-m "not live_model"` in `addopts`;
  - `.gitattributes` (`*.jsonl text eol=lf`);
  - `docs/DECISIONS.md`:
    - `ARC-57`, cognition budgets: wall-time cost ceilings (QS10-18), the gate before the recorder,
      ledgers;
    - `ARC-58`, recorded model outputs: the interface-level cassette, the key, strict ordered replay,
      modes;
    - `DEP-27`, the HTTP client `httpx2`, and no provider SDK. It records §4.2's and §4.3's
      alternatives with verdicts, and supersedes step-17's `DEP-S10-a`;
    - `DEP-32`, `python-dotenv` (`dotenv_values` only, never `load_dotenv`), with §4.2b's
      alternatives;
  - `.gitignore`: `.env.*` and `!.env.example`;
  - `docs/ARCHITECTURE.md`:
    - §9.1: `limits` become `max_calls_per_wall_hour` and `max_tokens_per_wall_day`, with one sentence
      saying the context bound is on simulated time;
    - §9.2: one sentence saying the backends are reached through one OpenAI-compatible adapter behind
      `ModelBackend`;
  - `scripts/ci_layer.py` (D-P5-11) and `.structured-coding/standards.md`.
- **Non-goals.** No backend, recorder or budget code.
- [ ] Implementation: the files above. Verify §3's "not verified in planning" items, except A-3,
  which is the spike's, and record each with its fallback (R-P5-1).
- [ ] Validation:
  - `uv lock`, then `uv sync --locked`;
  - ruff, ruff format and pyright strict, zero findings over both members;
  - `uv run --locked pytest cognition/lm-controller` collects with the guard active (0 tests is an
    expected result here, recorded as such);
  - `python3 scripts/ci_layer.py --list fast`, `--list python` and `--list python-smoke` show the
    intended commands, and `--list core` is byte-identical to the base's;
  - the lock holds `win_amd64`, `macosx_*_arm64` and Linux distributions for every new package;
  - both doc checks;
  - `standards.py inspect` lists the checks with the new paths.
- [ ] Review:
  - each DECISIONS entry names its alternatives and a revisit trigger (`REUSE_POLICY.md` §§11–12);
  - the §9.1 edit matches QS10-18 exactly;
  - no provider concept outside the allowed files.
- **Failure cases.** If `httpx2` lacks a mock transport: R-P5-1. If the lock fails on a platform: stop
  and record (a material dependency question).

### C2 — The model interface, the key, the scripted backend

- **Goal.** A provider-neutral request has one stable key on every platform.
- **Scope.** `backend/model.py`, `backend/canonical.py`, `backend/scripted.py`, and
  `tests/test_model_and_key.py`.
- [ ] Implementation: the models of D-P5-2; `canonical_json` and `cassette_key` per D-P5-6;
  `ScriptedBackend`.
- [ ] Validation:
  - AP5-1: (a) with the golden literal **recorded in the ledger before other tests**, and (b) with its
    mutation;
  - AP5-2 with its mutation;
  - strictness: a model or URL field cannot be added through `extra` (`extra="forbid"`), shown by one
    construction that fails.
- [ ] Review: no float anywhere in the request model (`temperature_milli`); `JsonValue` only at
  `output_schema`; no `Any`.

### C3 — Cassettes and the recorder

- **Goal.** Strict, ordered, atomic, secret-free recording and replay.
- **Scope.** `record.py`, `tests/test_record_replay.py`, and `tests/cassettes/` (written by tests).
- [ ] Implementation: header and entries per D-P5-7; `ReplayBackend` with per-key cursors;
  `CassetteMiss` (absent or exhausted) with diagnostics; `CassetteFormatError`; `RecordingBackend`
  with `.partial` and `os.replace`.
- [ ] Validation:
  - AP5-3, with both mutations, over a test-owned loopback listener counting accepts;
  - AP5-4 and AP5-5, with their mutations;
  - a corrupt line (not JSON) and a missing header, each refused with its line number.
- [ ] Review: no URL, header or key can reach `EntryMeta` (its fields are closed); every write uses
  `newline="\n"`; every file is closed before `os.replace`.

### C4 — The OpenAI-compatible adapter

- **Goal.** One adapter reaches Ollama, llama.cpp, LM Studio, vLLM or a user's hosted endpoint, with no
  implicit configuration.
- **Scope.** `backend/openai_compatible.py`, `backend/registry.py`, and
  `tests/test_openai_compatible.py`.
- [ ] Implementation: D-P5-4 (a)–(e) and D-P5-10's options; `Secret`; error mapping; usage mapping
  (`prompt_tokens` → `input_tokens`, `completion_tokens` → `output_tokens`; absent → estimated);
  `finish_reason` mapping (`stop` → `complete`, `length` → `length`, `content_filter` → `refused`; an
  unknown value → `malformed_response`).
- [ ] Validation:
  - AP5-6 (b) and (c) with the retry mutation;
  - one loopback stub-server test over real sockets (`asyncio.start_server` on `127.0.0.1:0`, a
    canned OpenAI-format answer), proving the real transport path;
  - `base_url` with userinfo refused;
  - an empty `base_url` impossible (required field).
- [ ] Review:
  - `grep` shows `httpx2` imported only here;
  - no `os.environ` read except through `key_env`;
  - no logging of request bodies or headers.

### C5 — Budgets and the gateway

- **Goal.** No call bypasses the wall-time ceilings, the in-flight limit or the timeout, and refusals
  happen before anything is recorded or called.
- **Scope.** `budget.py`, `gateway.py` (`ModelGateway`, `GateOutcome`, `Router`, `Tier`), and
  `tests/test_budget.py`.
- [ ] Implementation: D-P5-8 and §5.3's sequence; `SqliteLedger` (standard library) with
  close-before-move.
- [ ] Validation:
  - AP5-7 (i)–(vii), with its three mutations;
  - an integration round trip: record through the gateway with `SqliteLedger`, "restart" (new objects,
    same files), replay, and identical completions with the ledger continuing.
- [ ] Review:
  - P5-2 (the order) read in code;
  - P5-3: no simulated time in `budget.py`;
  - every `GateOutcome` consumer in tests matches exhaustively.

### C6 — Configuration, routing, the scans, CI measurement

- **Goal.** The operator's file is the only place a backend is named. Replay mode never touches a
  network client. The scans hold. CI runs it all on three platforms.
- **Scope.** `config.py`, `secrets.py`, `examples/`; `tests/test_config_router.py`,
  `tests/test_structural_isolation.py`, `tests/test_provider_scan.py` and `tests/test_secrets.py`.
- [ ] Implementation:
  - §5.4's model, with lazy adapter import (D-P5-5);
  - `ConfigError`s that name the table and key;
  - `secrets.py` per D-P5-9 and D-P5-14;
  - `backend/providers.py` and the `preset` field per D-P5-15 (amendment, 2026-10-09). Before writing
    each row, re-check every "verify" fact of §4.1c from the provider's own documentation, never by
    calling the API. Record each as confirmed, or as still "verify" in the README;
  - the hosted and `.env` examples (names only, no values);
  - the scan list of AP5-10 (a).
- [ ] Validation:
  - AP5-6 (a) in a child interpreter, with its mutation;
  - AP5-8 with its mutation;
  - AP5-9, with both sources, and its mutation;
  - AP5-13 (a)–(f) with their mutations;
  - AP5-14 (a)–(f) with their mutations (amendment, 2026-10-09);
  - AP5-10 (a)–(c) with their mutations;
  - on the PR's first CI run, record the static checks' added time in `fast` (60 s rule) and each
    `python` leg's wall time (3-minute rule);
  - AP5-12 green on all three legs.
- [ ] Review:
  - `worlds/` untouched;
  - nothing reads a key file the configuration did not name; `secrets.py` is the only reader;
  - the examples in docstrings use only `127.0.0.1` URLs; `examples/` hold names, never values;
  - no test, fixture or CI command sets a real key or a non-loopback `base_url` outside the mock
    transport.

### C7 — The operator-run spike tool

- **Goal.** The operator can choose the default model (QS10-2) with one command, against criteria
  fixed before the run.
- **Scope.** `tools/model_spike.py` and `tools/spike_scenarios.json`: 40 synthetic decisions, each
  with a system text, a quoted "heard" line and a small `SocialChoice`-shaped schema (`act`, `to`,
  `words` ≤ 480 bytes, `recalls`). Also a short operator section in `cognition/lm-controller/README.md`.
- [ ] Implementation:
  - `uv run python cognition/lm-controller/tools/model_spike.py --config FILE --models a,b,c --out
    DIR` runs each scenario once per model through `ModelGateway` in `record` mode;
  - it writes a report JSON (per model: first-attempt schema-valid count, p50 and p95 latency, the
    `reasoning` field honoured or not, the server's `/v1` behaviour, and the machine: OS, CPU, memory)
    and the cassette;
  - it refuses a non-loopback `base_url` unless `--allow-remote` is given. Even then, agents are
    forbidden to run it (§11).
- [ ] Validation:
  - the tool runs end to end against a **scripted** stub (a test passes it a `ScriptedBackend`
    factory), producing a report with the AP5-S fields;
  - the scenario file and the thresholds are committed **before** any real run (hash recorded in the
    ledger);
  - `--allow-remote` is absent from every CI command.
- [ ] Review: thresholds equal AP5-S's literals; the tool never reads a key file.
- **Operator step, not an agent task:** the operator runs the spike on the Mac (and, if wanted, on a
  Windows or Linux machine) and hands over the report and cassette. The agent commits them under
  `cognition/lm-controller/tests/cassettes/spike-<date>.jsonl` and `.../spike-<date>.json`, and records
  the chosen default in `README.md`'s example only. **If the operator has not run it by review, this
  item is NOT RUN, and P6's freeze waits for it (QP5-3).**

### C8 — Close-out

- [ ] Implementation:
  - `README.md`: what the package is, the modes, the configuration, the never-in-CI rule, and the
    **provider table** (amendment, 2026-10-09). The table lists each preset's name, provider, base URL,
    key environment variable name, defaults, and any "verify" mark from §4.1c;
  - the ledger;
  - the handoff closed.
- [ ] Validation:
  - the full Python suite and static checks on the final head, locally and on three CI legs;
  - `cargo test --workspace` NOT RUN for evidence, because no Rust changed (AP5-11). CI runs it;
  - both doc checks.
- [ ] Review: the whole diff against §2.3; every `[x]` has its evidence; deviations recorded.
- **Stop:** `READY FOR OPERATOR REVIEW — DO NOT MERGE`.

---

## 10. Requirements this PR places on other PRs

- **R-P4-1 (on P4).** P4 starts from P5's `cognition/lm-controller` tree and adds `memory/` and
  `compress/` to it. It does not recreate the package, the workspace member, the guard or the CI
  commands. If P4 is detailed before P5 merges, its design names P5's skeleton as a dependency.
- **R-P6-1 (on P6).** P6 extends `CognitionConfig` with `server`, `seats` and `store`. It reaches
  models only through `Router.for_tier`, handles every `GateOutcome` case, and treats `None` (unbound),
  `Refused` and `Failed` as fallbacks (step-17 §3.10.3). `CassetteMiss` stops the run. P6's freeze
  requires the spike report (AP5-S) or an operator decision in its place.
- **R-P7-1 (on P7).** IC-6 uses AP5-8's mechanism (rename and repoint the binding), at scenario scale.
- **R-P5b-1 (on P5b).** P5b adds adapters through `backend/registry.py` and `BackendConfig`'s `kind`
  union only. It reuses `Secret`, `resolve_key`, the gateway, the budgets and the recorder unchanged.

## 11. Execution contract (filled at freeze, 2026-10-09)

Source of every line marked "primary session": its freeze message of 2026-10-09, relayed by the
coordinator to the S10 planning session.

```text
PROJECT / PR:            MineWorld mvp0, S10 PR P5a — model backends, recorder and cassettes, budgets,
                         API keys
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p5-backends.md
RELATED / BINDING DOCS:  step-17-cognition.md (§§3.5, 3.7, 3.10, 3.11, 4.1, 4.5, 5, 15);
                         pr-s10-p3-python-sdk.md; overall.md (S10; §5 operator requirements);
                         docs/ARCHITECTURE.md §9; docs/MODULE_SPEC.md §5; docs/ENGINEERING_STANDARDS.md
                         (§§22–24); docs/REUSE_POLICY.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s10-p5             (primary session)
BRANCH:                  mvp0/pr-s10-p5-backends, created from main                  (primary session)
IMPLEMENTATION BASE:     origin/main at the start of implementation; C0 records it
APPROVED SCOPE:          §2.1, as frozen
FROZEN INVARIANTS:       §2.3; D-P5-1 … D-P5-15 (D-P5-15 by the 2026-10-09 amendment); QS10-18; QS10-19; QP5-1 … QP5-9 as ruled;
                         every platform
SEQUENCE:                C0 … C8 (C7's operator step may be NOT RUN at review)
ALLOWED COMMANDS:        cargo *; git; gh (never merge); uv *; python3 scripts/*; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading
                         ~/.config/mineworld/secrets.env or any key file; using any API key; calling any
                         hosted API; running, pulling or downloading any model (Ollama, llama.cpp, LM
                         Studio, vLLM or other); running tools/model_spike.py
MATERIAL STOPS:          any Rust or sdk/python change; any dependency beyond DEP-27 (httpx2), DEP-32
                         (python-dotenv) and the P3 toolchain (DEP-24 … DEP-26); any provider SDK; any
                         hosted-API use; any real key in a test or example; a change
                         to AP5-S's thresholds; the spike report showing no candidate meets AP5-S
PLATFORMS:               Linux, macOS, Windows (AP5-12)
VALIDATION BUDGET:       unit, static, local integration: unrestricted. Real model calls: none by
                         agents. CI: the PR's runs; one scratch mutation branch for AP5-10 (b) if the
                         primary session wants it shown red in CI
LIVE DOCUMENTATION:      this document (a §13 ledger is added in C0)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s10-p5.md
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

## 12. Questions

**[operator]** marks a question only the operator can answer: model or provider choice, a paid API,
scope, or a reversal of an operator ruling. **[primary]** marks one the primary session decides.

### 12.1 Rulings, 2026-10-09

| ID | Ruling |
| --- | --- |
| QP5-1 | **Primary: yes.** `httpx2`, no provider SDK. Revisit only if the native Anthropic adapter needs an SDK, as a compared and recorded decision (P5b §4). |
| QP5-2 | **Operator:** start from `qwen3.5:4b`; the spike settles the final choice. |
| QP5-3 | **Operator:** the operator runs the spike personally before P6 freezes. Agents never run, pull or download a model. |
| QP5-4, QP5-5, QP5-7, QP5-8, QP5-9 | **Primary: as recommended.** |
| QP5-6 | **Primary:** `ARC-57`, `ARC-58` and `DEP-27` as proposed. S10 also gets `DEP-32` (P5a: `python-dotenv`) and `DEP-33` (P5b). |
| New requirement | **Operator:** API keys through a `.env`-style file, and subscriptions through Codex's or Claude Code's own login. P5a carries the API-key half (§4.1b, §4.2b, D-P5-9, D-P5-14, AP5-13). P5b carries the Anthropic adapter and the subscription half. |

| QP5-10, QP5-11, QP5-12 | **Primary, 2026-10-09: yes**, as recommended in §12.2: per-user configuration; the process environment wins over the `env_file`; group- or world-readable key files refused on macOS and Linux. |
| Freeze | **Primary session, 2026-10-09:** P5a DESIGN FROZEN; contract §11 filled. |

**Planning-process note (recorded, 2026-10-09).** During revision 2 the planning session ran `sed -i`
once on this file: the single line renaming the heading of §5.2 to "D-P5-1 … D-P5-14". The brief
forbids `sed -i`. The change was correct and touched only that line, which the diff in PR #111 shows. It
is recorded here so that the slip is visible. No other file was edited that way.

### 12.2 Questions for P5a (all answered 2026-10-09, §12.1)

| ID | Question | Recommendation |
| --- | --- | --- |
| QP5-10 [primary] | Configuration and keys live per **user** (the operator who runs cognition), never per world; a key file inside a world directory is refused (D-P5-14)? | **Yes.** Per world would put credentials into shareable content and break `AC-4`. |
| QP5-11 [primary] | The process environment wins over the `env_file` when both define a variable (D-P5-9)? | **Yes**, as `python-dotenv`'s own default. A shell can override a file without editing it. |
| QP5-12 [primary] | Refuse a group- or world-readable `env_file` on macOS and Linux (D-P5-14)? | **Yes.** It is cheap and is the `ssh` convention. On Windows the check is skipped, and the README says so. |

### 12.3 The questions as first asked (kept for review)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QP5-1 [primary; reverses a frozen step-level choice]** | Depend on **no provider SDK**, and replace step-17's `DEP-S10-a` (the `openai` SDK) with our adapter over `httpx2` (`DEP-27`)? | **Yes.** The `openai` constructor reads `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_CUSTOM_HEADERS` and four more variables when an argument is omitted, and defaults to `api.openai.com`. Under QS10-19 that is a standing risk: a hosted key or URL in a user's shell redirects local traffic. Its retries also hide calls from the ledger. We use none of its value-add, since validation is ours. `httpx2` is the client the `openai` SDK itself now uses, maintained by Pydantic's stewards. The adapter is about 200 lines in one file. |
| **QP5-2 [operator — model choice]** | The default `social` model? | **`qwen3.5:4b`** (Apache-2.0, 3.3–4.0 GB), the smallest candidate that fits 8 GB Windows and Linux machines with a long context. Recommended for 16 GB machines such as the operator's Mac: `qwen3.5:9b` or `gemma4:12b` (both Apache-2.0). Final choice by AP5-S on the operator's machine. Llama 3.2 (community licence) and Gemma 3 (custom terms) are excluded from the default, because committed cassettes would carry their terms. |
| **QP5-3 [operator — who runs a model]** | Who runs the spike, and when? May an implementing agent run a local Ollama if the operator explicitly authorizes it? | **The operator runs it**, after C7 is on the branch and before P6's freeze. P5 can reach review with C7's operator step NOT RUN. Agents never run models, as the brief requires. If the operator wants an agent to run it, that is an explicit, recorded grant for local models only, and it changes §11's NEVER line. |
| QP5-4 [primary] | Budget defaults on the wall clock: per seat 20 calls per rolling hour and 30 000 tokens per rolling 24 hours; per process 2 in flight and a 20 s timeout? | **Yes**, as `ARCHITECTURE.md` §9.1's numbers re-keyed. Note: at about 2 000–3 000 tokens per social decision, 30 000 tokens a day is about 10–15 model replies per seat per day. That is enough for Milestone D's demonstration but tight for a long play session. The operator may raise it in `cognition.toml`; the default stays conservative because a user's hosted endpoint costs money. |
| QP5-5 [primary] | P5 creates `cognition/lm-controller`, and P4 builds on it (D-P5-1, R-P4-1)? | **Yes.** P5 lands first under §15.5's order. |
| QP5-6 [primary] | Decision numbers: `ARC-57` budgets, `ARC-58` recorder, `DEP-27` the HTTP client? This uses S10's last DEP number. P4's `sqlite3` record (`DEP-S10-d`) and the general decline record (`DEP-S10-f`) will need numbers from outside S10's range. | **Yes**, and allocate two more DEP numbers to S10 now (for P4 and the decline record), so P4's design does not wait. |
| QP5-7 [primary] | Keep `OllamaNativeBackend` out of P5, conditional on the spike's A-3 result (QS10-3)? | **Yes.** If A-3 fails, it is a bounded follow-up of about 80 lines over the same client, with no other module changed. |
| QP5-8 [primary] | Should QS10-5's second real server be llama.cpp's `llama-server` (MIT), in an optional operator-run `live_model` test? | **Yes.** It shows the adapter is generic with no paid API. Never in CI. |
| QP5-9 [primary] | Live tests (`live_model`) deselected by `addopts` and restricted to loopback URLs (D-P5-12)? | **Yes.** A hosted live test would contradict QS10-19 for any gate. A user who wants one runs it outside the suite. |

## 13. Ledger (live during implementation)

```text
Status:            IN PROGRESS. Worktree /Users/yuema137/mineworld-worktrees/impl-s10-p5, branch
                   mvp0/pr-s10-p5-backends (sole writer; fresh implementation session, 2026-10-09)
Implementation
base:              origin/main @ 2c6d34c (#111, the P5a freeze with the provider-preset amendment)
Final heads:       recorded in the handoff and the PR body (a commit cannot carry its own run)
Post-merge sync:   the S10 planning session owns step-17 §15 and overall.md; this session owns this
                   ledger, its evidence and its deviations
Handoff:           handoff-s10-p5.md
```

### 13.1 C0 — freeze and contract

- [x] Implementation: the `DESIGN FROZEN` header and §11's contract were committed with the freeze (#111);
  every endpoint line in §11 names its source (the primary session's freeze message, relayed by the
  coordinator). This session records the base above (`2c6d34c`) and initializes
  [`handoff-s10-p5.md`](handoff-s10-p5.md).
- [x] Validation (E-P5-0): `python3 scripts/check_doc_headings.py` ("192 numbered sections across 26
  documents, none duplicated") and `python3 scripts/check_decision_ids.py` ("85 decision ids, all
  distinct"), both exit 0. `grep -c 'ARC-57\|ARC-58\|DEP-27\|DEP-32' docs/DECISIONS.md` = 0: the four
  numbers are free.
- [x] Review: every authority line in §11 has a source; none was narrowed or widened. The kickoff brief of
  this session restates §11's NEVER list (no model run, no key file, no hosted API, no real CLI) and adds
  only process rules (WebFetch for "verify" facts; never rename or slow `fast`/`test`), none of which
  changes an endpoint.

### 13.2 C1 — specs, decisions and the package skeleton

- [x] Implementation: root `pyproject.toml` (member and pyright `include`); `uv.lock` (+ `httpx2` 2.13.1,
  `httpcore2` 2.13.1, `h11` 0.16.0, `anyio` 4.15.1, `idna` 3.20, `truststore` 0.10.4, `python-dotenv`
  1.2.4, `httpx2-jsfetch` 1.0 emscripten-only); `cognition/lm-controller/{pyproject.toml, README.md
  (placeholder until C8), src/mineworld_cognition/{__init__.py, py.typed}}` with the guard, the
  `live_model` marker and `-m "not live_model"` in `addopts`; `.gitattributes` (`*.jsonl text eol=lf`);
  `.gitignore` (`.env.*`, `!.env.example`, and DV-P5-1's negation); `docs/DECISIONS.md` `ARC-57`,
  `ARC-58`, `DEP-27`, `DEP-32`; `docs/ARCHITECTURE.md` §9.1 (wall-time keys plus the context-bound
  sentence) and §9.2 (one adapter behind `ModelBackend`); `scripts/ci_layer.py` (`PYTHON_MEMBERS`; two
  pytest commands in `python` and in `python-smoke`); `.structured-coding/standards.md` (the static
  checks over both members; a new check `pytest-cognition`).
- [x] Validation (E-P5-1):
  - `uv lock` resolved 27 packages; `uv sync --locked` succeeds;
  - `ruff check` "All checks passed!", `ruff format --check` "19 files already formatted", `pyright`
    strict "0 errors" over both members;
  - `uv run --locked pytest cognition/lm-controller`: rootdir `cognition/lm-controller`, configfile its
    `pyproject.toml`, plugin `socket-0.8.1` loaded, "collected 0 items", exit 0 (expected: no tests yet);
  - `ci_layer.py --list core` byte-identical to the base's (`diff` empty); `--list python` and
    `--list python-smoke` show `pytest sdk/python …` and `pytest cognition/lm-controller` as two
    commands; `--list fast` ends with the static checks over both members;
  - every new package's wheel is `py3-none-any` (lock entries read), so all three platforms are served;
  - `check_doc_headings.py` exit 0; `check_decision_ids.py` "89 decision ids, all distinct";
  - `standards.py inspect` lists `ruff-lint`, `ruff-format`, `pyright-strict`, `pytest`,
    `pytest-cognition`;
  - `git check-ignore -v`: `.env` (`*.env`), `.env.local` (`.env.*`), `prod.env` (`*.env`),
    `secrets.env` (`secrets*`) ignored; `examples/.env.example` and `src/mineworld_cognition/secrets.py`
    matched only by their negations (not ignored).
- [x] Review: each DECISIONS entry names its alternatives with verdicts and a revisit trigger; the §9.1
  edit keys cost on wall time and the context bound on simulated time, exactly QS10-18; DECISIONS and
  ARCHITECTURE name providers (documentation), no source file does yet.

**§3's "not verified in planning" items (A-3 excepted, it is the spike's):**

```text
httpx2 API            PASS  2.13.1 exports AsyncClient, Timeout, MockTransport (sync and async
                            handlers; httpx2/_transports/mock.py), ConnectError, TimeoutException, …
                            R-P5-1's fallback is not needed
httpx2 platforms      PASS  anyio, httpcore2, h11, idna, truststore, python-dotenv: py3-none-any wheels
                            only; httpx2-jsfetch is emscripten-only (marker in the lock)
pytest-socket + httpx2  verified in C6 (AP5-10 (b)): the TEST-NET connection test
Ollama /v1 json_schema  A-3: the operator's spike (QP5-3), not an agent
```

**Findings.**

- **F-P5-1 (`httpx2` reads the environment by default).** `AsyncClient(trust_env=True)` is the default and
  reads proxy variables, `.netrc` and certificate-file variables (`httpx2/_client.py`, `allow_env_proxies
  = trust_env and transport is None`). D-P5-4 (a) forbids any environment read but `key_env`, so the
  adapter passes `trust_env=False`. Consequence: a user behind an HTTP proxy is not served by this
  version; recorded in `DEP-27` as a limitation.
- **F-P5-2 (anyio's pytest plugin).** `anyio` (an `httpx2` dependency) registers a pytest plugin, now
  loaded in both members' runs. It changes nothing unless a test uses its marker; the suites keep
  `asyncio.run`, as the SDK's do.

**Deviation DV-P5-1 (bounded).**

```text
Deviation:        .gitignore gains `!/cognition/lm-controller/src/mineworld_cognition/secrets.py`
Reason:           the existing pattern `secrets*` (a guard for key files) also matches the module the
                  design names `secrets.py`, which would silently stay untracked
Source evidence:  git check-ignore -v reported `.gitignore:55:secrets*` for the module before the negation
Impact:           one anchored negation for one source path; every key-file pattern still holds
Validation:       git check-ignore (E-P5-1); AP5-13 (e) re-checks it in a test (C6)
```

### 13.3 C2 — the model interface, the key, the scripted backend

**AP5-1 (a)'s golden literal, recorded before any test was written (2026-10-09).** Computed
**independently of the implementation**: the canonical bytes were written out by hand from D-P5-6's rules
and hashed with the system's `shasum -a 256` (so the expected value does not come from the code under
test, test rules §25). The request: purpose `decide`; messages `system` "You are Alice, who keeps the
café." and `user` `Bob says: "Hello — one coffee?"`; output schema `{"type": "object", "properties":
{"act": {"enum": ["say", "wait"]}}, "required": ["act"]}`; sampling `temperature_milli` 700,
`max_output_tokens` 256, `seed` 42.

```text
canonical  {"key_scheme":1,"request":{"messages":[{"role":"system","text":"You are Alice, who keeps the café."},{"role":"user","text":"Bob says: \"Hello — one coffee?\""}],"output_schema":{"properties":{"act":{"enum":["say","wait"]}},"required":["act"],"type":"object"},"purpose":"decide","sampling":{"max_output_tokens":256,"seed":42,"temperature_milli":700}}}
key        6bd78b32af7a296775cda4bc397c7803340a7377a9e897195003658a11037a27
```

- [x] Implementation: `backend/model.py` (`Purpose`, `Role`, `Finish`, `FailureReason`, `CognitionModel`
  — strict, frozen, `extra="forbid"` — `Message`, `Sampling`, `CompletionRequest`, `Usage`, `Completion`,
  `BackendFailure` with a cross-field rule (a status exactly when the reason is `http_status`), and the
  `ModelBackend` protocol: `complete(request) -> Completion | BackendFailure`, `aclose()`);
  `backend/canonical.py` (`KEY_SCHEME = 1`, `CassetteKey`, `KeyMaterialError(path)`,
  `canonical_object`, `canonical_json`, `cassette_key`); `backend/scripted.py` (`ScriptedBackend(script,
  *, delay_s)` with a `calls` counter); `backend/__init__.py` (imports nothing);
  `tests/support.py`; `tests/test_model_and_key.py`.
- [x] Validation (E-P5-2): 5 passed; ruff, format and pyright strict clean. AP5-1 (a) passed on its
  first run against the hand-computed literal. Mutations (§13.10): M-1 `sort_keys=False` → the golden
  test fails; M-2 return instead of raising on a float → both AP5-2 path cases fail. Both reverted
  (`git diff` of `src/` empty afterwards; 5 passed). The strictness case (a `model` field through
  `extra`) fails construction, naming the field. AP5-1 (b) (two gateways, two configurations, one key)
  needs the gateway and the configuration: it is shown in C6 with AP5-8 and AP5-14 (c).
- [x] Review: no float field in the request model (`temperature_milli`, integers throughout);
  `JsonValue` appears only at `output_schema`; no `Any` in `src/`. Design choice recorded: a backend
  **returns** `BackendFailure` (a typed value) rather than raising it, so the gateway's outcome union is
  exhaustive and no failure path carries an exception message that could hold a response body (I-16);
  `CassetteMiss` alone is an exception (§5.3).

### 13.4 C3 — cassettes and the recorder

- [x] Implementation: `record.py` — `EntryMeta` (closed: `binding` matches the backend-name pattern,
  `model` refuses `://` and `@`, `recorded_at` RFC 3339 UTC `…Z`, `latency_ms` int), `CassetteEntry`,
  `Cassette.load` (universal newlines, header check by name — kind, `format`, `key_scheme` — each entry
  parsed with `model_validate_json` and its key recomputed), `CassetteFormatError(path, line, why)`,
  `CassetteMiss(kind, key, nearest, first_difference)` (an exception), `ReplayBackend` (per-key cursors;
  holds no other backend), `RecordingBackend` (`.partial` opened with mode `x` and `newline="\n"`, the
  header written and flushed at once, one flushed line per completed call, `BackendFailure` passed
  through unrecorded, an exception from the inner backend abandons the recording and closes the file,
  `aclose` closes the file **then** `os.replace`s), `CassetteRecordError`, `partial_path`;
  `tests/test_record_replay.py`.
- [x] Validation (E-P5-3): 16 passed (11 new); ruff, format, pyright strict clean. AP5-4: ordered replay,
  `exhausted`, an edited request refused at load naming line 3, `format: 2` and `key_scheme: 2` refused
  by name; a corrupt line refused with its number; a missing header refused; a CRLF copy still loads.
  AP5-5: three calls recorded and replayed, no `\r\n` in the file, no `.partial` left; interrupted in the
  third call: the old cassette byte-identical, `.partial` holds the header and two entries, and a new
  `RecordingBackend` is refused naming `kept.jsonl.partial`. Mutations (§13.10): M-3 return the first
  entry for every repeat → the ordered test fails; M-4 skip the key check → the edited-entry test fails;
  M-5 write the target directly (and replace nothing) → the interrupted test fails (the cassette's bytes
  changed at index 113). All reverted; `grep MUTATION` empty; 16 passed.
- [x] Review: `EntryMeta`'s fields are closed and none can hold a header or a key; every write uses
  `newline="\n"`; the only `os.replace` runs after `close()`. AP5-3 (a miss never reaches a backend,
  shown against a loopback listener) needs `config.load` and the router: it is written in C6, where a
  fall-through could actually be implemented, and recorded there (bounded placement; the criterion and
  its mutations are unchanged).

### 13.5 C4 — the OpenAI-compatible adapter

- [x] Implementation: `backend/openai_compatible.py` (`request_body`, `_parse`, `OpenAICompatibleBackend
  (config, key, *, transport=None)`); `backend/registry.py` (`FACTORIES`, `build_backend`, the adapter
  imported inside its factory); `secrets.py` (`Secret`: redacted `repr`/`str`, `__slots__`, refuses
  pickling — `resolve_key` follows in C6); `config.py` (`BackendConfig` with D-P5-10's typed options,
  `check_base_url`, `is_loopback_host`, `KEY_ENV`, `ConfigError` — the rest of the file follows in C6);
  `backend/model.py` gains `estimate_tokens` (D-P5-8's `ceil(utf8_bytes / 4)`, shared by the adapter's
  missing-usage path and the budget's pre-check); `tests/test_openai_compatible.py`.
- [x] Validation (E-P5-4): 33 passed (17 new), ruff, format, pyright strict clean. AP5-6 (b): the exact
  body (`model`, `messages` with `content`, `max_tokens`, `stream: false`, `temperature` 0.7 from
  `temperature_milli` 700, `seed`, `response_format` `json_schema` with the schema); `json_object`;
  `none` (no `response_format`); `temperature = "omit"`; `reasoning` `off` → `reasoning_effort: "none"`,
  `high` → `"high"`; no `Authorization` without a key and `Bearer …` with one; 401 → `unauthorized`,
  500 → `http_status(500)`, a non-JSON body and an unknown `finish_reason` → `malformed_response`;
  missing usage → estimated and flagged. Over real loopback sockets: a stub server with a canned answer
  (the request line `POST /v1/chat/completions HTTP/1.1` observed); a closed port → `unreachable`; a
  stalled server with `request_timeout_s = 1` → `timeout` in under 2 s. `base_url` with userinfo, remote
  `http`, a query, or a non-HTTP scheme refused; a missing `base_url` refused. AP5-6 (c): a 503 is sent
  once; M-6 (one retry on 5xx) → the count is 2 and the test fails; reverted.
- [x] Review: `grep import.*httpx2` finds `openai_compatible.py` only; no `os.environ`, `getenv`,
  `logging` or `print` anywhere in `src/`; a failure carries a reason and a status, never a body.

**Decisions recorded at C4 (bounded, within D-P5-4 and D-P5-10).**

- `reasoning = "off"` maps to `reasoning_effort: "none"`, the value OpenAI's and Ollama's documentation
  use for "no reasoning"; `low`/`medium`/`high` pass through.
- `json_schema` is sent as `{"type": "json_schema", "json_schema": {"name": "answer", "schema": …,
  "strict": true}}`, OpenAI's envelope. `strict: true` asks a server to constrain decoding; a server
  that cannot is answered by local validation in P6 (R-P5-2), and a user can choose `json_object`.
- 403 maps to `unauthorized`, like 401 (both mean the key is not accepted); every other non-2xx status
  is `http_status`.
- `base_url` also refuses a query or a fragment (a key in `?key=…` is the case in point), tightening
  D-P5-10's "no userinfo" in the same direction.
- With `request_timeout_s` unset the adapter sets no HTTP timeout of its own (`httpx2.Timeout(None)`):
  `httpx2`'s 5 s default would cut ordinary generations short, and the gateway's `call_timeout_s` bounds
  every call anyway (D-P5-8).

### 13.6 C5 — budgets and the gateway

- [x] Implementation: `budget.py` (`Clock`, `SystemClock`, `BudgetPolicy` — QP5-4's defaults —
  `BudgetRefusal(limit, spent, asked, ceiling)`, `Window`, the `Ledger` protocol, `MemoryLedger`,
  `SqliteLedger` (one table `budget_ledger(seat, at, calls, tokens, estimated)`, committed per charge,
  `close()`), `reservation`, `Reservations`, `precheck`, `InFlightLimiter` over `asyncio.Semaphore`);
  `gateway.py` (`Tier`, `TIERS`, `Completed`/`Refused`/`Failed`, `GateOutcome`, `Budget` — what every
  gateway of a process shares — `ModelGateway.complete` in §5.3's order, `Router.for_tier`, and
  `Router.aclose`, which closes each gateway once and then the ledger); `record.py`: an exception from
  the inner backend abandons the recording, a cancellation (the gateway's timeout) does not
  (`except Exception`, see F-P5-3); `tests/test_budget.py`.
- [x] Validation (E-P5-5): 40 passed (7 new); ruff, format, pyright strict clean. AP5-7 (i) the 21st call
  is `Refused(calls_per_wall_hour)` and the backend counter stays at 22 for 22 admitted calls; another
  seat is unaffected; (ii) after 3 601 s the call is admitted; (iii) a reservation of estimate +
  `max_output_tokens` past 30 000 is refused before the call; (iv) `SqliteLedger`: 20 calls, closed,
  reopened, the 21st refused; (v) an AST scan of `budget.py` and `gateway.py`: no import from a
  `…contract` module, no `WorldTime`; (vi) five concurrent calls with `max_in_flight = 2`: two start, the
  rest wait, starts in submission order, peak 2; (vii) a backend sleeping 5 s with `call_timeout_s = 1`
  → `Failed(timeout)` in under 2 s, charged one call and no tokens. Integration: record through the
  gateway with `SqliteLedger`, restart (new objects, same files), replay: identical outcomes, and the
  ledger holds 6 calls and 90 tokens across both sessions. Every outcome in the tests goes through one
  `match` ending in `assert_never`. Mutations (§13.10): M-7 check the budget after the call → (i) and
  (iii) fail (counters 23 and 3); M-8 `SqliteLedger` on `:memory:` → (iv) and the round trip fail;
  M-9 import `Observation` from `mineworld_sdk.wire.contract` into `budget.py` → (v) fails. All reverted.
- [x] Review: P5-2 read in code — key material, then `precheck`, and only then the reservation, the
  limiter and the backend; P5-3 — `budget.py` reads only the injected `Clock`; a `CassetteMiss` raised
  inside the call releases the reservation in `finally` and charges nothing (AP5-3 (iii)).

**Findings and decisions (bounded).**

- **F-P5-3.** The gateway's timeout cancels the inner call; in C3 `RecordingBackend` caught
  `BaseException` and would have abandoned a whole recording on one slow call. It now catches `Exception`:
  a cancelled call is simply not recorded; a crash still abandons the session (AP5-5 unchanged, green).
- **Concurrent admission.** D-P5-8's pre-check reads the ledger, which is charged only after the call;
  two concurrent calls could both pass a check only one fits. Admitted calls therefore hold a
  **reservation** (one call, the reserved tokens) until they are charged, and the pre-check counts it.
  This is D-P5-8's "the actual usage replaces the reservation", made exact.
- **The charge's timestamp** is the pre-check's `now`, so a window is deterministic under a fake clock.

### 13.7 C6 — configuration, routing, presets, secrets, the scans

- [x] Implementation: `config.py` completed (`BackendConfig` gains `preset` and fills omitted fields from
  the row; `RecordingConfig`, `BudgetConfig(BudgetPolicy)` with `ledger`, `SecretsConfig`,
  `CognitionConfig` with its cross-field rules, `world_directory_of`, `load(path)` — `tomllib`, strict
  models, errors rendered from location and message only, paths resolved against the file, a
  configuration or `env_file` inside a `world.yaml` directory refused — and `build_router(config, *,
  clock, scripted)`, which imports the registry and `secrets.resolve_key` only in `live`/`record`);
  `backend/providers.py` (`ProviderPreset`, `PresetName`, `PRESETS`, eleven rows; the option literals
  `StructuredOutput`, `Reasoning`, `TemperatureMode` live here so `config.py` can import the table
  without a cycle); `secrets.py` completed (`permission_check_applies`, `check_env_file`,
  `resolve_key`); `errors.py` (`ConfigError`, shared by `config.py` and `secrets.py`); `registry.py`'s
  `FACTORIES` is a `dict` (tests substitute a factory with `monkeypatch.setitem`, no production seam);
  `examples/hosted.toml.example`, `examples/.env.example` (names, empty values); `README.md` with the
  provider table; `tests/test_config_router.py`, `tests/test_secrets.py`,
  `tests/test_structural_isolation.py`, `tests/test_provider_scan.py`; `tests/cassettes/example.jsonl`
  (written by `test_the_committed_cassette_loads_as_lf_and_is_reproduced_by_this_test`'s own recording
  over `ScriptedBackend`, copied from its `--basetemp`; the test re-records and compares keys, requests,
  completions and bindings).
- [x] Validation (E-P5-6): cognition suite **77 passed**; SDK suite (no binary) 30 passed, 4 deselected;
  ruff, format ("41 files already formatted") and pyright strict clean over both members.
  - AP5-1 (b) and AP5-8: one cassette recorded under binding `local`, replayed under configurations
    `local`/`127.0.0.1:11434`/`model-a` and `elsewhere`/`[::1]:8080`/`model-b`: identical completions,
    keys equal to the recomputed keys and the first to AP5-1's literal.
  - AP5-3: replay mode, backend `local` at a test-owned listening loopback port; an absent request
    raises `CassetteMiss("absent")` naming its key, the nearest recorded request and
    `$.messages[0].text`; the listener accepted **0** connections; the ledger holds 0 calls, 0 tokens.
  - AP5-6 (a): a child interpreter loads a replay configuration, builds the router, completes one
    replayed call: `{"completed": true, "modules": []}` — no `httpx*` module, no `openai_compatible`.
  - AP5-9, per source (environment; `env_file` 0600): a record session through `config.load` →
    `build_router` → the real adapter over a mock transport that echoes every header into its 500 and
    401 bodies, then a 200. The server received `Bearer <key>` three times; the key is absent from the
    cassette, the SQLite ledger's bytes, every captured log record (DEBUG, all loggers), captured stdout
    and stderr, and the `str`/`repr` of every outcome, the configuration, the router and the gateway. An
    unset variable raises `ConfigError` naming it and the file, not the file's other value.
  - AP5-10 (a) the scan (word-bounded, case-insensitive) over `src/` minus the four allowed files, the
    committed cassette's keys and requests, and `sdk/python/src`: no finding; (b) the member's `addopts`
    hold the guard and the deselection; a connection to `192.0.2.1:80` through the adapter is refused by
    `SocketConnectBlockedError` (wrapped by anyio's task group) in under 1 s; (c) `ci_layer.py --list`
    for `python` and `python-smoke` shows two pytest commands, the second exactly `uv run --locked pytest
    cognition/lm-controller`.
  - AP5-13 (a) `os.environ` compared whole before and after a file read: equal; (b) the environment
    wins; (c) `cognition.toml` and an `env_file` under a temporary `worlds/cafe/world.yaml` refused,
    naming the path; (d) macOS: 0644 refused naming `chmod 600`, 0600 accepted; on Windows the test
    asserts `permission_check_applies() is False` and that 0644 is accepted (skipped by platform,
    visibly); (e) `git check-ignore --no-index`: `.env`, `.env.local`, `prod.env`, `secrets.env` ignored;
    `examples/.env.example` and `src/mineworld_cognition/secrets.py` not; (f) no `load_dotenv` call or
    import, `dotenv` imported by `secrets.py` only, `httpx2` by the adapter only, no reference to the
    operator's `secrets.env` or `.config/mineworld`.
  - AP5-14 (a), (b) every row `https` (or None), no userinfo, query or fragment, `key_env` a name, no
    key-like value; (c) the golden request recorded through config → router → gateway → recorder under
    `preset = "openai"`, `preset = "xai"` and a hand-written table (the adapter factory swapped for a
    scripted backend): the key equals AP5-1's literal each time and `meta.binding` is the user's `mine`;
    (d) a set `structured_output` wins; (e) `dashscope` without `base_url` names `base_url`; (f) the
    README's table lists exactly the eleven presets and their key variables.
  - §5.4's refusals, each naming its table and key: an unknown key (`recording.colour`), a tier bound to
    an undefined backend (`tiers.social`), replay without a cassette (`recording.cassette`), the
    non-tier `routine` (`tiers.routine`), a pasted key in `key_env` (`backends.x.key_env`) — whose value
    is not echoed. The hosted example loads.
  - Mutations (§13.10), each shown red then reverted: M-10, M-11, M-12, M-13, M-14, M-15, M-16, M-17,
    M-18, M-19, M-20.
- [x] Review: `worlds/` untouched; `secrets.py` is the only reader of a key and of a key file, and reads
  only the file the configuration names; docstrings and examples use loopback URLs or `api.example.com`
  (the hosted example names the providers' presets, never a key); no test, fixture or CI command sets a
  real key or reaches a non-loopback address (the TEST-NET connection is refused by the guard before any
  packet leaves).
- **CI timing (the 60 s and 3-minute rules)** is recorded at C8 from the PR's run (E-P5-8).

**Preset verification, 2026-10-09, from each provider's own documentation (WebFetch; no API called).**

```text
preset      fact checked                         source read                                   result
openai      base URL; json_schema                (planning, §4.1c: the schema's origin)          confirmed in planning
xai         base URL; json_schema, json_object   docs.x.ai/docs/guides/structured-outputs        CONFIRMED; reasoning_effort
                                                                                                 not on the page → README "verify"
deepseek    json_object                          api-docs.deepseek.com/guides/json_mode          CONFIRMED ("Set the response_format
                                                 ("Set the response_format parameter to         ... json_object"); base URL
                                                 {'type': 'json_object'}"); base_url             https://api.deepseek.com confirmed;
                                                                                                 json_schema not documented (README note)
glm         base URL; JSON mode                  docs.bigmodel.cn OpenAI page and               CONFIRMED: base https://open.bigmodel.cn/
                                                 guide/capabilities/struct-output                api/paas/v4; json_object
zai         base URL; JSON mode                  docs.z.ai/api-reference/introduction;          CONFIRMED: https://api.z.ai/api/paas/v4;
                                                 docs.z.ai/guides/capabilities/struct-output    json_object (no json_schema)
mistral     json_schema envelope                 docs.mistral.ai/api, structured-output pages    base URL and `{"type": "json_schema"}`
                                                                                                 mode confirmed; envelope's sub-fields not
                                                                                                 in the fetched text → README "verify"
moonshot    JSON mode; base URL                  platform.kimi.ai/docs/guide/use-json-mode-...   CONFIRMED: json_object;
                                                                                                 https://api.moonshot.ai/v1
dashscope   JSON mode                            alibabacloud.com/help/en/model-studio/json-mode CONFIRMED: json_object on most Qwen
                                                                                                 models, json_schema on some; the prompt
                                                                                                 must contain the word "JSON" (README note);
                                                                                                 base URL per workspace and region (preset None)
gemini      (no "verify" mark)                   planning §4.1c                                  not re-fetched
openrouter  (no "verify" mark)                   planning §4.1c                                  not re-fetched
groq        (no "verify" mark)                   planning §4.1c                                  not re-fetched
```

**Deviations (bounded).**

```text
DV-P5-2  errors.py, a module not in §5.1: ConfigError is shared by config.py and secrets.py, and
         config.py imports secrets lazily; defining it in either would make the two import each other.
DV-P5-3  BackendConfig.kind defaults to "openai-compatible" (the only kind until P5b), so a preset table
         needs only `preset` and `model`. P5b's new kind is still a closed literal added beside it.
DV-P5-4  record mode records one backend binding at a time (a ConfigError otherwise): two recorders on one
         cassette would race for one `.partial`. Replay serves every bound tier from one cassette, since
         keys are provider-free.
DV-P5-5  AP5-3 is in test_config_router.py (C6), not C3: the fall-through it guards against can only be
         implemented where a router builds backends (recorded in §13.4).
DV-P5-6  the option literals (StructuredOutput, Reasoning, TemperatureMode) are defined in
         backend/providers.py and imported by config.py, to keep the import graph acyclic.
```

**Process note.** While editing a test this session ran `sed -i.bak '' /dev/null` once by mistake (an
empty `sed -i` on `/dev/null`, which the brief forbids). It touched no file: `git status` and a search for
`*.bak` showed nothing. Recorded so the slip is visible.

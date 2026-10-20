# mineworld-cognition

The language-model side of a MineWorld cognition client: ask a model through one provider-neutral
interface, within wall-time budgets, with recorded replay for tests. Specification:
[`pr-s10-p5-backends.md`](../../.structured-coding/plans/mvp0/pr-s10-p5-backends.md); decisions
`ARC-57`, `ARC-58`, `DEP-27`, `DEP-32` in [`docs/DECISIONS.md`](../../docs/DECISIONS.md).

**Modes** (`[recording] mode`): `live` asks the backend; `record` asks it and writes a cassette;
`replay` answers only from a cassette and builds no network client; `scripted` uses a backend a test
supplies. Tests and CI use only `replay` and `scripted`: no model, no key, no hosted API, ever.

**Configuration** is a TOML file of yours, never inside a world (see
[`examples/hosted.toml.example`](examples/hosted.toml.example)); every field it accepts is listed in the
generated [`cognition-toml` reference](../../docs/reference/cognition-toml.md). A key appears only as the **name** of an
environment variable (`key_env`). Its value comes from your environment, or from a `.env`-style file you
name in `[secrets] env_file` ([`examples/.env.example`](examples/.env.example)); that file is read without
touching the environment, must be `chmod 600` on macOS and Linux, and is never read unless named.
Suggested places for both files: macOS `~/Library/Application Support/MineWorld/`; Linux
`$XDG_CONFIG_HOME/mineworld/` (or `~/.config/mineworld/`); Windows `%APPDATA%\MineWorld\` — keep it under
your own profile there, where POSIX permissions are not checked.

**Local models** (Ollama, llama.cpp, LM Studio, vLLM): `base_url` is your loopback address, for example
`http://127.0.0.1:11434/v1`, with `key_env = ""`.

**Hosted providers**: set `preset = "<name>"` and a `model`; any field you set overrides the preset.
"verify" marks a fact the provider's documentation did not confirm on 2026-10-09.

| Preset | Provider | Base URL | Key variable | Structured output | Temperature | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `openai` | OpenAI | `https://api.openai.com/v1` | `OPENAI_API_KEY` | `json_schema` | `send` | |
| `xai` | xAI (Grok) | `https://api.x.ai/v1` | `XAI_API_KEY` | `json_schema` | `send` | verify: which models accept `reasoning_effort` |
| `deepseek` | DeepSeek | `https://api.deepseek.com` | `DEEPSEEK_API_KEY` | `json_object` | `send` | `json_schema` not documented |
| `glm` | Zhipu GLM (China) | `https://open.bigmodel.cn/api/paas/v4` | `ZHIPUAI_API_KEY` | `json_object` | `send` | |
| `zai` | Z.ai GLM (international) | `https://api.z.ai/api/paas/v4` | `ZAI_API_KEY` | `json_object` | `send` | |
| `mistral` | Mistral | `https://api.mistral.ai/v1` | `MISTRAL_API_KEY` | `json_schema` | `send` | verify: the exact `json_schema` envelope |
| `moonshot` | Moonshot (Kimi) | `https://api.moonshot.ai/v1` | `MOONSHOT_API_KEY` | `json_object` | `omit` | some models refuse any other temperature |
| `dashscope` | Alibaba DashScope (Qwen) | none: set `base_url` for your workspace and region | `DASHSCOPE_API_KEY` | `json_object` | `send` | JSON mode needs the word "JSON" in the prompt |
| `gemini` | Google Gemini | `https://generativelanguage.googleapis.com/v1beta/openai` | `GEMINI_API_KEY` | `json_schema` | `send` | beta compatibility layer |
| `openrouter` | OpenRouter | `https://openrouter.ai/api/v1` | `OPENROUTER_API_KEY` | `json_schema` | `send` | unsupported parameters are ignored |
| `groq` | Groq | `https://api.groq.com/openai/v1` | `GROQ_API_KEY` | `json_object` | `send` | `json_schema` on a few models only |

**Claude** uses its own adapter (`DEP-33`), with your Anthropic API key: `kind = "anthropic"`,
`base_url = "https://api.anthropic.com"`, a `model`, `key_env = "ANTHROPIC_API_KEY"`. It sends schema-
constrained output natively and omits the temperature unless you set `temperature = "send"`. Pointing
`kind = "openai-compatible"` at Anthropic is refused: that layer ignores `response_format` and `seed`.

A hosted API costs you money; MineWorld never calls one in a test.

**Subscriptions** (`ARC-60`). API keys, above, are the recommended route for every hosted model. A
subscription is your account with a vendor, and its terms decide what another program may do with it:

- **Claude: not supported through a subscription.** Anthropic's Claude Code terms
  (<https://code.claude.com/docs/en/legal-and-compliance>, read 2026-10-09): "Anthropic does not permit
  third-party developers to offer Claude.ai login into their own applications, or to route requests
  through Free, Pro, or Max plan credentials on behalf of their users", enforced "without prior
  notice". Use your Anthropic API key instead.
- **Codex with a ChatGPT plan: not available yet.** It is planned only as an opt-in for your own local
  use, off by default and never in an example. OpenAI recommends API keys "for automation"
  (<https://learn.chatgpt.com/docs/auth>, read 2026-10-09), and its Terms of Use must be re-read before
  this route is built; until then, use an OpenAI API key (`preset = "openai"`). If it is built, you
  remain responsible for your plan's terms; check them yourself.
- MineWorld never reads, copies or forwards a CLI's login: a bridged CLI authenticates itself, runs in
  an empty temporary directory, and receives none of your API keys. Your CLI's own global configuration
  (hooks, MCP servers) is yours and still runs.

**Memory** (`memory/`, `compress/`; `ARC-59`, `DEP-37`, `ARC-109`). A seat remembers only what its Person
perceived: it is a seat view of a memory home, one SQLite file at a path you choose, never inside a world's
save, which stores each perceived fact once for every seat that holds it. A memory file from before schema 4
is refused; delete it and the seat rebuilds its memory from the perceived stream. Facts become one-line
records, then episodes, weekly chapters and what it knows of each person, every line citing the Event
IDs it stands for. The memory part of a decision's context stays under 6 000 bytes however long the
world has run, and is written with no model; optional prose through the `summarize` tier never changes
a citation. Specification:
[`pr-s10-p4-memory.md`](../../.structured-coding/plans/mvp0/pr-s10-p4-memory.md). A world chooses each
seat's memory stack in `decisions.yaml` `memory: scopes`; the scopes and their parameters are in the
generated [`memory-scopes` reference](../../docs/reference/memory-scopes.md).

**Forgetting** (`memory/retention/`, `forgetting/`; `ARC-87`). Once per simulated day a pass lets a
memory policy merge old detail into summaries or forget it, and a pass also runs early whenever the store
nears its bound: 20 MiB per person by default, which a world may raise (`store_bound_mib` in
`decisions.yaml`). Every forgotten Event ID still resolves in the save (`mineworld perceived`). The
default, `consolidate`, and `decay`, `importance-recency-cap` and the test fixture `keep-all` are plug-ins
of the entry-point group `mineworld.forgetting`; a policy of your own is a package with one entry point.
No model is involved. Specification:
[`pr-fg-b-retention-host.md`](../../.structured-coding/plans/mvp0/pr-fg-b-retention-host.md), with the
bound and the early pass in
[`pr-fg-b2-size-trigger.md`](../../.structured-coding/plans/mvp0/pr-fg-b2-size-trigger.md).

**Assisted forgetting** (`lm-summarise`, `pr-fg-c`). `lm-summarise` asks the `summarize` tier, through the
host's summary gateway, for one sentence and an importance per group, and falls back to `consolidate` for any
group it cannot ask. Its tests replay a recorded cassette and never reach a model.

**The LM controller** (`python -m mineworld_cognition`; `ARC-72`, `ARC-73`) makes one of a world's people
decide through a language model, while the world stays valid without any model. The decisions are social
only: what Alice says, to whom, and which offer she takes. Her fallback model drives her whenever no
process holds her seat. Specification:
[`pr-s10-p6-lm-controller.md`](../../.structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md).

Run it from the repository root, after `cargo build -p mineworld-cli`:

```sh
target/debug/mineworld server worlds/social-cafe --town --decisions cognition/lm-controller/examples/alice-lm.decisions.yaml --save <dir>
target/debug/mineworld decisions worlds/social-cafe --decisions cognition/lm-controller/examples/alice-lm.decisions.yaml --json > seats.json
MINEWORLD_INVITE=<the server's invite> python -m mineworld_cognition --config cognition/lm-controller/examples/local.toml
```

- The configuration names the server, the store, and `seats.json` (`decisions_json`). The invite is read
  from the environment variable the file names (`invite_env`); its value is never printed.
- The process refuses to start, naming the file, when `seats.json` has no `lm` seat, or when an `lm` seat
  is not one of the world's seats in the server's welcome (`D-P6b-21`). The welcome carries no world
  identifier, so a seat name that two worlds share is not told apart (`F-P6b-21`).
- `--hold` (the server's seat hold, default 30 s; `QP6-9`) is the time a killed process's seat stays held
  before Alice's fallback drives her again. The examples use 3 s so that a test can see it.
- Ctrl-C leaves every seat within 5 s and deletes its resume secret; a killed process leaves nothing
  behind, and the seat returns to its fallback after the hold. Each seat prints one status line at exit,
  from its decision log's last counters line.
- When the server's per-seat request budget refuses a request (`rate_limited`), the controller waits at least
  the time the server names and sends the same request again; the model is not asked again (`D-P6b-35`).
- A seat with a punctual next part of its day is told when it begins and about how long walking there takes,
  and is asked about it from `horizon_minutes` before (`[controller]`, default 60) every `reask_minutes` (default
  15). The model may answer `go` with a listed place; the seat then walks there one step a wall second. Nothing
  walks it otherwise, so a model that never says `go` is late, and the complaint reaches its next prompt
  ([`pr-le-leave-early.md`](../../.structured-coding/plans/mvp0/pr-le-leave-early.md) §0.4).
- A seat whose memory policy is assisted (`lm-summarise`) asks the `summarize` tier; with `summarize = "none"`
  its fallback, `consolidate`, decides alone (`D-P6b-36`).
- Windows: the resume file's `0o600` mode applies on macOS and Linux only; on Windows the file inherits
  your profile's permissions. Paths are resolved against the configuration file on every platform.

**Admitted models.** A model is admitted as a triple: the runtime and its version, the model build, and the
`structured_output` setting. An admission is a property of the triple, not of the model's name: the same model
fails under one setting and passes under another. The spike's results are in
[`pr-s10-p6-lm-controller.md`](../../.structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md) §0.1.

The design asks for each run's report hash (`scenarios_sha256` in its `report.json`). The operator's reports are not
in the repository, so the hashes are not recorded here until the operator adds them (D-P6b-34).

| Runtime | Model | `structured_output` | Valid on first attempt | p95 | Date | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Ollama 0.40.2 (local) | `qwen3.5:4b` (nvfp4 build) | `none` | 39 / 40 | 4.8 s | 2026-10-13 | admitted; the default local model |
| OpenAI API | `gpt-4.1-nano` | `json_schema` | 40 / 40 | 0.9 s | 2026-10-13 | admitted; the recommended API option |

Refused, not admitted: `qwen3.5:4b` under `json_schema` on Ollama 0.40.2 (every call answered HTTP 501: the
runtime refuses enforced output); `gpt-5-nano` under `json_schema` (every call answered HTTP 400, so the claim
was not exercised: inconclusive). Run the spike again after a runtime upgrade; an admission does not carry over
to a new runtime version.

**Choosing the default local model** is an operator-run spike, never run by CI or an agent:

```sh
uv run python cognition/lm-controller/tools/model_spike.py --config FILE --models qwen3.5:4b --out DIR
```

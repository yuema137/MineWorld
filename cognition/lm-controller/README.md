# mineworld-cognition

The language-model side of a MineWorld cognition client: ask a model through one provider-neutral
interface, within wall-time budgets, with recorded replay for tests. Specification:
[`pr-s10-p5-backends.md`](../../.structured-coding/plans/mvp0/pr-s10-p5-backends.md); decisions
`ARC-57`, `ARC-58`, `DEP-27`, `DEP-32` in [`docs/DECISIONS.md`](../../docs/DECISIONS.md).

**Modes** (`[recording] mode`): `live` asks the backend; `record` asks it and writes a cassette;
`replay` answers only from a cassette and builds no network client; `scripted` uses a backend a test
supplies. Tests and CI use only `replay` and `scripted`: no model, no key, no hosted API, ever.

**Configuration** is a TOML file of yours, never inside a world (see
[`examples/hosted.toml.example`](examples/hosted.toml.example)). A key appears only as the **name** of an
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

Claude needs its own adapter (S10 P5b). A hosted API costs you money; MineWorld never calls one in a test.

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

**Choosing the default local model** is an operator-run spike, never run by CI or an agent:

```sh
uv run python cognition/lm-controller/tools/model_spike.py --config FILE --models qwen3.5:4b --out DIR
```

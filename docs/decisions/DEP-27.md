## DEP-27 — The model HTTP client: `httpx2`, and no provider SDK

**Date** 2026-10-09 · **Status** adopted by S10 PR P5a (`httpx2>=2.13,<3`; locked 2.13.1) · **Approved
by** the primary session (QP5-1, 2026-10-09) · **Licence** BSD-3-Clause · **Supersedes** step-17 §4.1's
placeholder `DEP-S10-a` (the `openai` SDK) · **Relates to** `ARC-58`, `DEP-24`, `DEP-32` · **Design**
`pr-s10-p5-backends.md` §4.1, §4.2, D-P5-4, D-P5-10, AP5-6

**Problem.** Every backend MineWorld targets — Ollama, llama.cpp's server, LM Studio, vLLM, and the hosted
OpenAI-compatible APIs a user configures (OpenAI, xAI, DeepSeek, Zhipu GLM, Mistral, Kimi, DashScope,
Gemini, OpenRouter, Groq) — speaks the OpenAI-compatible `POST /chat/completions` schema. Something must
send that request with a cancellable timeout, and must refuse every source of configuration but the
operator's file.

**Options considered** (facts read 2026-10-09):

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| `openai` SDK (`AsyncOpenAI(base_url=…)`) | Apache-2.0; 3.27.0 | **Rejected.** With an argument omitted its constructor reads `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_ORG_ID`, `OPENAI_PROJECT_ID`, `OPENAI_ADMIN_KEY`, `OPENAI_WEBHOOK_SECRET` and `OPENAI_CUSTOM_HEADERS`, and defaults to `api.openai.com`: a hosted key in a user's shell could redirect local traffic, or its key, to a hosted API (QS10-19). Its retries hide calls from the budget ledger. Its value-add, typed parsing, is ours anyway (local validation) |
| `ollama-python` | MIT; 0.6.3 | Reference only: native API only; pins the old `httpx` line |
| LiteLLM | MIT core, proprietary `enterprise/` | Rejected: the provider lives in the model string, and the tree is large |
| `httpx` | BSD-3-Clause; 0.28.1, no release since 2024-12 | Fallback only: unmaintained; `openai` 3.x and the Anthropic SDK moved to `httpx2` |
| **`httpx2`** | **BSD-3-Clause; 2.13.1 (2026-09-23)**, stewarded by Pydantic Services (whose `pydantic` is `DEP-24`) | **Adopted** |
| `aiohttp` | Apache-2.0 AND MIT; 3.14.4 | Rejected: nine runtime dependencies and a server framework, for one POST per decision |
| standard library (`urllib.request` in a thread) | PSF | Rejected: blocking I/O in a thread cannot be cancelled when the timeout fires |

**Choice.** One adapter of our own, `backend/openai_compatible.py`, about 200 lines over
`httpx2.AsyncClient`: request mapping, response parsing, error mapping, **no retries** (one call is one
ledger entry). It is constructed only from a typed `BackendConfig`; `base_url` has no default; it reads
no environment variable except the one `key_env` names. It is the only module that imports `httpx2`.

**Verified at P5a (C1), not assumed:**

- `httpx2` 2.13.1 exposes `AsyncClient`, `Timeout` (per-phase), `MockTransport` (sync and async
  handlers, `httpx2/_transports/mock.py`) and the `httpx` exception hierarchy (`ConnectError`,
  `TimeoutException`, …).
- Its dependencies (`anyio`, `httpcore2`, `h11`, `idna`, `truststore`; `typing-extensions` below 3.13)
  are pure-Python `py3-none-any` wheels, so the universal lock serves Linux, macOS and Windows alike.
  `httpx2-jsfetch` is locked but installed only on `emscripten`.
- **Finding:** `AsyncClient(trust_env=True)`, the default, reads proxy variables, `.netrc` and
  certificate-file variables from the environment (`httpx2/_client.py`). The adapter therefore passes
  `trust_env=False`: a user who needs an HTTP proxy for a hosted API is not served by this version, which
  is recorded as a limitation, not worked around with an implicit read.

**Revisit** if the native Anthropic adapter needs the `anthropic` SDK (P5b compares it), if a needed
provider feature (tool calls, batch, realtime) cannot be expressed in the plain schema, or if `httpx2`
stops being maintained — the switch to `httpx` or `aiohttp` is one file.

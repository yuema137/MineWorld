## DEP-33 — Claude by API key: a native Anthropic Messages adapter over `httpx2`; the SDKs declined

**Date** 2026-10-09 · **Status** adopted by S10 PR P5b (no new dependency: `httpx2` is `DEP-27`'s) ·
**Approved by** the primary session (QP5b-4, 2026-10-09) and the operator's ruling QP5b-1 (Claude is
supported by API key only) · **Relates to** `DEP-27`, `DEP-32`, `ARC-58`, `ARC-60` · **Design**
`pr-s10-p5b-hosted-subscriptions.md` §4.3, §5.1, AB-1, AB-2

**Problem.** The operator asked for Claude among the hosted models. Anthropic's OpenAI-compatibility
layer, reachable through `DEP-27`'s adapter, ignores `response_format` and `seed` and refuses a
temperature below 1 on recent models (`platform.claude.com/docs/en/api/openai-sdk`, read 2026-10-09:
"not considered a long-term or production-ready solution"). Schema-constrained output needs the native
Messages API (`output_config.format = {"type": "json_schema", …}`).

**Options considered** (facts read 2026-10-09):

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| `anthropic` SDK | MIT; 1.13.0; depends on `httpx2`, `anyio`, `docstring-parser`, `jiter`, `pydantic`, `sniffio`, `typing-extensions` | **Rejected.** Verified in its source (`src/anthropic/_client.py`, `main`, read 2026-10-09): with an argument omitted, both `Anthropic.__init__` and `AsyncAnthropic.__init__` run `os.environ.get("ANTHROPIC_API_KEY")`, `os.environ.get("ANTHROPIC_AUTH_TOKEN")` and `os.environ.get("ANTHROPIC_BASE_URL")`; `max_retries` defaults to `DEFAULT_MAX_RETRIES = 2` (`_constants.py`), so one decision could be three calls the budget ledger never sees. Both contradict `DEP-27`'s rules (no environment read but `key_env`; no retries) |
| `claude-agent-sdk` | MIT; 0.2.165; bundles the Claude Code CLI | **Rejected.** It would ship Claude Code inside MineWorld (Anthropic's Commercial Terms apply), and its documentation forbids the subscription login it would be used for |
| Anthropic's OpenAI-compatibility layer through `DEP-27`'s adapter | — | **Rejected for Claude**, and refused by `config.load`: `kind = "openai-compatible"` with a `base_url` on `api.anthropic.com` is a `ConfigError` naming `kind = "anthropic"` |
| **Our adapter over `httpx2`** | — | **Adopted**: `backend/anthropic_messages.py`, one POST per call |

**Choice.** `AnthropicMessagesBackend` (`kind = "anthropic"`): `POST {base_url}/v1/messages` with
`x-api-key` (only when a key was resolved) and `anthropic-version: 2023-06-01`, never `Authorization`;
`system` messages hoisted; `max_tokens` from the request; `temperature` sent only when configured
(`"omit"` is the default for this kind); no `seed` (the API has none); no retry; `trust_env=False`. The
request's schema is **lowered** for Anthropic's structured outputs (keywords it refuses removed,
`additionalProperties: false` on every object) by a pure function that returns a copy: the cassette key
is computed on the unlowered request (`ARC-58`), so a cassette recorded through Claude replays under any
backend, and local validation (P6) still enforces every removed bound. `anthropic_messages.py` and
`openai_compatible.py` are the only importers of `httpx2`.

**Revisit** if Anthropic's API needs a feature the plain Messages request cannot express (tool use,
batches, streaming), if the OpenAI-compatibility layer gains `response_format`, or if the SDK stops
reading the environment implicitly and retrying by default.

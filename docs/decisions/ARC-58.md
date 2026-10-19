## ARC-58 — Recorded model outputs: an interface-level cassette, keyed on the provider-neutral request

**Date** 2026-10-09 · **Status** accepted; implemented by S10 PR P5a (`record.py`,
`backend/canonical.py`) · **Approved by** the primary session at the P5a freeze (2026-10-09), under the
operator's rulings QS10-14 (recorded outputs, tests with no network) and QS10-19 (local models only)
· **Relates to** `ARC-56`, `ARC-57`, `AC-4`, `CLAUDE.md` §4 rule 10 · **Design** step-17 §3.11, §4.5;
`pr-s10-p5-backends.md` D-P5-5, D-P5-6, D-P5-7, AP5-1 … AP5-5, AP5-8 · **Placeholder** `ARC-S10-e`

**Problem.** Core tests never need a live model (`CLAUDE.md` §4 rule 10), yet the cognition path must be
exercised with real model outputs, deterministically, on every platform. `AC-4` requires that swapping
the backend needs no World Pack edit, so a recording must replay under any backend.

**Options considered** (P5a §4.3, with licences and maturity):

```text
vcrpy / pytest-recording (MIT)   HTTP cassettes: provider wire formats, URLs and headers to scrub; a
                                  cassette recorded against Ollama does not replay under llama.cpp; no
                                  httpx2 support found
respx (BSD-3-Clause)              mocks httpx, not httpx2; HTTP-level, so provider-specific
LiteLLM caching (MIT)             the key contains the provider-prefixed model name
inline-snapshot (MIT)             reference only: its review-the-diff workflow is the model for re-recording
our own interface-level cassette  chosen
```

**Choice.** A cassette records at **our** interface, above HTTP:

- **The key** is `sha256` over canonical JSON of `{"key_scheme": 1, "request": CompletionRequest}`:
  UTF-8, sorted keys, `(",", ":")` separators, no ASCII escaping, no Unicode normalization, and **no
  floats** (a float anywhere, including inside an output schema, is refused). The request holds no
  provider, model, URL, tier, binding or key, so one cassette replays under every backend.
- **The format** is JSON Lines, UTF-8, LF: a header `{"cassette": "mineworld-cognition", "format": 1,
  "key_scheme": 1}`, then entries `{key, request, completion, meta: {binding, model, recorded_at,
  latency_ms}}`. Never a URL, a header, a key, `key_env`, or a transport failure.
- **Replay is strict and ordered.** Entries are grouped by key in file order; the n-th call with a key
  returns the n-th entry; one more is a miss (`exhausted`), an unknown key is a miss (`absent`, with the
  nearest recorded request and the first differing JSON path). A miss is an exception, never a fallback,
  and never reaches a backend. An entry whose `key` does not match its `request` is refused at load.
- **Record is atomic**: it writes `<name>.jsonl.partial`, flushes each line, and replaces the cassette on
  clean close; a crash leaves the old cassette and a visible `.partial`, which blocks the next record.
- **Modes**, process-wide: `live`, `record`, `replay`, `scripted`. In `replay` and `scripted` no backend
  object is constructed and the HTTP adapter is never imported, which is stronger than a network guard
  that allows loopback, where a user's local model listens.

**Consequences.** Redaction holds by construction rather than by scrubbing. A prompt change invalidates
the cassettes that recorded it, and the miss names the first differing path; re-recording from a real
model is operator work (QS10-19).

**Revisit** if a provider feature the plain request cannot express (tool calls, streaming deltas) must be
recorded, which would be a `key_scheme` or `format` bump, never a silent change.

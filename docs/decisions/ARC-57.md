## ARC-57 — Cognition budgets: cost ceilings on wall time, decided before the recorder and the backend

**Date** 2026-10-09 · **Status** accepted; implemented by S10 PR P5a (`mineworld-cognition`,
`budget.py`, `gateway.py`) · **Approved by** the primary session's ruling QS10-18 (2026-10-08) and the
P5a freeze (QP5-4, 2026-10-09) · **Relates to** `ARC-56`, `ARC-58`, `INV-4`, step-17 `I-13` · **Design**
`.structured-coding/plans/mvp0/step-17-cognition.md` §3.10, §15.6;
`.structured-coding/plans/mvp0/pr-s10-p5-backends.md` D-P5-8, §5.3, AP5-7 · **Placeholder**
`ARC-S10-d`

**Problem.** A model call costs wall time on a GPU, or money on a user's hosted key. S11-B's
`time_scale` lets one wall second be N simulated seconds, so a ceiling keyed on simulated time
(`ARCHITECTURE.md` §9.1 as first written: calls per simulated hour, tokens per simulated day) costs N
times as much per wall hour at scale N. A ceiling must also stop a call *before* it is made, or it bounds
nothing.

**Options considered:**

```text
(a) ceilings on simulated time (the first §9.1 text)   cost grows with time_scale; rejected by QS10-18
(b) ceilings on wall time, checked after the call      a refused call has already been paid for
(c) ceilings on wall time, checked before the call,    chosen
    with the actual usage charged after it
(d) one shared ceiling per process, no per-seat one    one talkative seat starves every other seat
```

**Choice: (c).** Cost ceilings are keyed on **wall** time and read only an injected clock (UTC epoch
seconds); no budget code reads simulated time. The **context** bound stays on simulated time, because
memory is about a life (`I-13`); it belongs to memory and context assembly (P4, P6), not here.

- **Per seat** (`EntityKey`), rolling windows: `calls_per_wall_hour` (default 20, 3 600 s) and
  `tokens_per_wall_day` (default 30 000, 86 400 s) — `ARCHITECTURE.md` §9.1's numbers re-keyed.
- **Per process:** `max_in_flight` (default 2, FIFO) and `call_timeout_s` (default 20).
- **Order** (`ModelGateway.complete`): key-material check → budget pre-check → in-flight slot → timeout
  around (recorder → backend) → charge. A refused call reaches neither the recorder nor the backend.
- **Pre-check** reserves `ceil(utf8_bytes(messages) / 4) + max_output_tokens`, erring toward refusal; the
  **post-charge** uses the backend's reported usage, or the estimate flagged `estimated`. A transport
  failure charges the call and no tokens; a timeout charges the call.
- **Ledgers:** an in-memory one for tests and a SQLite one (standard-library `sqlite3`) at a path the
  operator configures, never inside a world save (`INV-4`), so a restart neither resets nor
  double-counts a window.
- **Outcomes** are typed, never exceptions: `Completed | Refused | Failed`. A controller falls back on
  `Refused` and `Failed` (P6).

**Consequences.** A world's `time_scale` changes nothing about what cognition may spend per hour of the
operator's day. A crash-looping cognition process cannot spend again after each restart. Prices are
never modelled: tokens are reported, never priced (step-17 §3.10.4).

**Revisit** if a provider bills on a unit that tokens do not bound (for example per-request fees with
no token count), or if per-seat ceilings prove too coarse for many seats sharing one GPU.

**Dated note, 2026-10-14 (S10 P6b, C4): two additive interface changes.** `ModelGateway.complete` gains an
optional keyword `timeout_s`; the call's timeout is the smaller of `call_timeout_s` and `timeout_s`, so a
consult deadline cut by the caller is charged like a timeout (one call, no tokens), where cancelling from
outside would skip the charge. `BackendFailure` gains `retry_after_s` (an integer, 0 to 3 600, or absent),
set by both HTTP adapters from a delta-seconds `Retry-After` on any non-2xx response; an HTTP date is
ignored. The gateway still never retries (`ARC-73`). Failures are never recorded, so cassette format and
keys are unchanged. Ruled by QP6-6 (2026-10-14).

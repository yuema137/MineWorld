# Handoff — playable dialogue and memory, PR-A (C1)

Continuation aid only. The authority is the frozen design
[`pr-playable-dialogue-memory.md`](pr-playable-dialogue-memory.md) (DM-20261127-3) and its contract; this file
never overrides either. It is replaced, not appended, at the PR-A → PR-B boundary.

## Identity

| Field | Value |
| --- | --- |
| PR | PR-A = commit C1 only (absolute consult deadline + gateway accounting) |
| Branch / worktree | `mvp0/pr-dialogue-a-deadline` in `/Users/yuema137/mineworld-worktrees/impl-dialogue-a` |
| Base | `origin/main` @ `8c971f0f7b469f05d1bf6f21fc933501eb22c404` |
| Design source | PR #535 branch `docs/pr-design-dialogue-memory` @ `8492b95b` (not merged when PR-A was implemented) |
| Live model | none in PR-A (contract: "PR-A: no live model at all") |

## Ledger placement (bounded deviation)

The design's §10 PR-A ledger lives in a file that exists only on the unmerged design branch (#535). Adding it
here would collide with #535 (add/add). PR-A's ledger evidence is therefore recorded in this handoff and in the
PR body; the primary coordinator (or the PR-B owner) copies it into design §10 once #535 merges. No design
content is changed by PR-A.

## C1 — implementation (done)

- [x] One absolute expiry from `Pending` receipt through the caller chain; the consult-start reset removed.
  `controller/deadline.py` (new, 81 lines): `Expiry.after(received_at, consult_deadline_s)`, `before` (a wait
  bounded by what is left), `send_locked` (lock wait within the expiry, re-check once held, immediately before
  the send), `NEVER` (pacer submits), `EXPIRED`. `seat.py::_consults` builds the expiry from `Waiting.received_at`
  and bounds the first-observation wait; `_consult_once(…, expiry)`; `_submit(…, expiry=NEVER)` via
  `send_locked`. `consult.py`: `ConsultLimits(expiry, max_calls)`; `remaining()` reads the expiry.
- [x] Gateway admission + backend bounded by the remaining budget: `gateway.py::ModelGateway.complete` starts
  `asyncio.timeout(min(call_timeout_s, timeout_s))` before `budget.limiter`; `_still_open` re-checks after the slot
  is granted. Consult: an answer at or after the expiry is not acted on; the pre-submit check and the seat's
  under-lock check return `Expired`; retry delay `>= remaining` ends at the deadline. Terminal record is always
  `fell_back` / `deadline`.
- [x] Release on every exit (reservation in `finally`, slot by context manager); exactly one charge per started
  backend call, dated at its start (`started = clock.now()` just before the backend), including cancellation;
  `CancelledError` re-raised; an expired-in-queue call is `Failed(timeout)` and uncharged; `CassetteMiss` still
  propagates uncharged.
- [x] `ARC-57` dated note 2026-11-27 (new order, cancellation charge, authority `cognition-toml.md`), `ARC-73`
  dated note (absolute per-consult deadline). `call_timeout_s` description now says it includes the slot wait;
  `docs/reference/cognition-toml.md` and its schema regenerated with `tools/reference.py`.

Bounded deviations (recorded per working rules §15):

- *Callers outside the named test files.* `ConsultLimits` is also built by `controller/log.py` (decision-log
  replay), `tests/reporter_harness.py`, `tests/test_controller_log.py`, `tests/test_milestone_d_fallback.py`.
  Each was migrated mechanically to `Expiry` (replay: `Expiry.after(REPLAY_CLOCK_S, REPLAY_DEADLINE_S)`, still
  never expiring). Found by strict pyright and grep. No behaviour change in those callers.
- *`triggers.py` unchanged.* `Waiting.received_at` already carries the receipt; the expiry is derived from it in
  `_consults`, so no change to `Pending` was needed.
- *`budget.py` description.* `call_timeout_s` now bounds the slot wait as well; its operator-facing description
  (and the generated reference) changed so spec and code agree.

## C1 — validation

| Check | Command | Result (on `83e52776` unless noted) |
| --- | --- | --- |
| Failing-before-fix | the five new gateway/seat tests on base `8c971f0f` + tests only | 4 FAILED, 1 passed (cancel-before-slot already correct): queue expiry waited for the slot (2 s guard tripped); cancelled backend call uncharged; first-observation wait gave a fresh deadline (backend asked); pending-wait consult `chose` instead of `deadline` |
| Targeted unit/integration | `uv run --locked pytest` on `test_budget`, `test_controller_consult`, `test_controller_seat`, `test_controller_log`, `test_fill_publish_notice`, `test_price_reporter_replay`, `test_milestone_d_replay`, `test_controller_homes`, `test_home_cassettes`, `test_assisted_host`, `test_config_router`, `test_cli_bridge`, `test_secrets`, `test_structural_isolation` | PASS: 168 passed, 2 skipped (one Windows-only `.cmd` shim) |
| Real-binary tests | `test_controller_resilience.py`, `test_milestone_d_fallback.py` against `target/debug/mineworld` built from `83e52776` (`cargo build -p mineworld-cli`, one heavy slot) | PASS: 3 passed in 129 s |
| Static | `ruff format --check`, `ruff check`, `pyright cognition/lm-controller` (strict) | PASS: clean, 0 errors |
| Size | `wc -l seat.py` | 765 before, 765 after (no net growth, §4.14) |

Mutations (each applied to the staged tree, run, reverted):

| # | Mutation | Expected | Observed |
| --- | --- | --- | --- |
| M1 | `send_locked` drops the re-check after the lock | lock test sends late | caught (1 failed) |
| M2 | consult acts on an answer arriving after the expiry | seat pending-wait test `chose` | caught (seat test failed; the consult-level case is also held by the pre-submit check) |
| M3 | consult drops its pre-submit expiry check | overrunning rate-limit wait re-sends | caught after adding `test_a_rate_limit_wait_that_overruns_the_expiry_sends_nothing_more` (survived before it) |
| M4 | seat starts a fresh deadline after the observation wait | both seat tests | caught (2 failed) |
| M5 | gateway does not charge a cancelled started call | cancel test | caught (1 failed) |
| M6 | charge dated before the backend start | gateway tests | caught (5 failed) |

Equivalent under deterministic clocks: removing the timer bound of the first-observation wait (the clock re-check
after the wait still discards the trigger); the bound matters only for real time, where both clocks are
`time.monotonic`.

## C1 — logic review (gateway callers, charge paths, uncertainty, locks, sizes)

- Gateway callers: `seat._ask` (remaining budget), `log._replay_one` (never-expiring replay bound, frozen wall
  clock), `memory/retention/assisted.py` through `SummaryBoundaryGateway` and `compress/model_summarizer.py`
  (no `timeout_s`: `call_timeout_s` now also bounds their slot wait — a summary queued behind a consult for more
  than `call_timeout_s` returns `Failed(timeout)` uncharged instead of waiting without bound; their existing
  `Failed` handling applies), `tools/model_spike.py` (no change needed).
- Charge paths: success (tokens), `BackendFailure` and timeout after start (one call, 0 tokens), cancellation after
  start (one call, 0 tokens, re-raised), expired before start (nothing), refusal (nothing), `CassetteMiss`
  (nothing, propagates — unchanged; not a started-call outcome in the design's list).
- Timeout vs. sent: the seat's post-send wait for the answer is not cut by the expiry; an answer after the expiry
  to a request sent before it is recorded as answered; the consult never re-sends except on `rate_limited`
  (which is a refusal, not an acceptance) and that re-send is also expiry-checked.
- Stop / pacer: the pacer submits with `NEVER` (unbounded lock wait, as before); a cancelled consult waiting for
  the lock leaves through `asyncio.Lock`'s own cancellation; a cancelled send inside the lock releases it in
  `finally`. `_step` asserts `NEVER` cannot expire.
- Known limitation: the decision-log counter `model_calls` counts a transport attempt that expired in the
  gateway queue (it reached no backend; the ledger correctly charges nothing). Log replay of a consult that ended
  at the deadline on a late answer does not reproduce wall time (pre-existing class: replay never expires).
- Edge: a `go`/`sleep` proposal whose submit expires under the lock has already stopped the pacer; nothing is
  sent. Rare (requires expiry between the answer check and the lock) and harmless to the world.

## Next action

Push, open PR-A, wait for `fast` CI on the exact head, then READY FOR OPERATOR REVIEW. Do not merge.

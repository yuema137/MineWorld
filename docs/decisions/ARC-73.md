## ARC-73 — Failure containment: layered retry above a retry-free gateway, a per-consult deadline and cap, a per-seat breaker, and fallback by handing the seat back

**Date** 2026-10-14 · **Status** accepted for S10 P6b · **Approved by** the primary session's freeze
(2026-10-14; operator QP6-7 on the thresholds, QP6-6 on the two P5a changes, QP6-8 and QP6-9 on the
fallback and the hold) · **Relates to** `ARC-57`, `ARC-58`, `ARC-56`, `ARC-72`, `DEP-44`, `ARC-33`,
step-24 §8.6 · **Design** `pr-s10-p6-lm-controller.md` §§5.5–5.8, §5.13, §4

**Problem.** A model fails in three different ways: the transport fails (rate limits, server errors, timeouts),
the content is unusable (invalid JSON, a schema mismatch, an unknown person), or the world refuses a valid
proposal (an offer that moved). Each deserves its own response. The world must never wait for a model,
and every model call must be charged to a budget.

**Options considered:**

```text
(a) one retry loop in the gateway                       breaks ARC-57: the gateway would make calls the ledger
                                                        has not budgeted for the retry's class
(b) one retry policy for all failures                   content and transport need different budgets
(c) three retry layers in the controller host, above a
    gateway that never retries; each layer has its own
    pluggable policy; one deadline, one call cap, one
    per-seat breaker; fallback by handing the seat back  chosen
(d) a retry or breaker library                          DEP-44: they own the loop and the clock; declined
```

**Choice: (c).** Layers: transport (`exponential-backoff`, honouring `Retry-After`), content (`fixed`, two
re-asks with the validation error), semantic (`fixed`, one re-plan from the newest observation). Every model
call is one gateway call and one ledger charge. A consult ends in exactly one terminal outcome. A
per-consult deadline (20 wall s), a cap of five calls, and the P5 budgets bound it. A `failure-rate` breaker
per seat (window 20, minimum 10, more than 20 %, cooldown one simulated hour) stops a failing model. When
the controller cannot decide, the seat is handed back with `leave` (only where the welcome said
`took_over: "hosted"`), and the server's fallback model drives it. Where no in-server fallback exists, the
seat stands for that consult and sends nothing (QP6-8). The hand-back is host state and writes no fact.
Counters go to the controller's own log.

**Consequences.** The world's facts stay valid under any model behaviour, because the world validates every
request at dispatch (step-17 §3.6). A failing model costs at most five calls per consult and one handback
per breaker trip. The fallback share is an acceptance criterion (AP6-4, AP6-14), not an assumption.

**Revisit** when a second model seat needs a different breaker or retry shape, which becomes a registered
policy rather than a code change, or when an idle handback (QP6-3's follow-up) is decided.

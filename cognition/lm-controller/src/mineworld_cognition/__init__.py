"""mineworld-cognition: ask a language model through one provider-neutral interface.

Design: `.structured-coding/plans/mvp0/pr-s10-p5-backends.md` (P5a). A cognition component builds a
`CompletionRequest`, asks the `Router` for the gateway of a tier, and receives a typed `GateOutcome`. The
gateway applies the wall-time budgets, then the recorder, then the backend. Nothing here names a
provider, a model or an endpoint; those live in the operator's configuration only.
"""

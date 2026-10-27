## ARC-72 — The LM controller is built from typed parts; model text is a proposal; a request is built only through `mineworld_sdk.offers`

**Date** 2026-10-14 · **Status** accepted for S10 P6b (`cognition/lm-controller/src/mineworld_cognition/controller/`)
· **Approved by** the primary session's freeze of `pr-s10-p6-lm-controller.md` (2026-10-14; operator
QP6-1, QP6-3, QP6-7; QP6-12 on the number) · **Relates to** `ARC-56`, `ARC-58`, `ARC-59`, `ARC-34`,
`ARC-85`, `INV-10`, step-17 I-10, I-15 · **Design** `pr-s10-p6-lm-controller.md` §§5.1–5.4, §5.9 ·
**Prior art** `docs/references/PRIOR_ART_2026-10.md` PA-10, PA-11 (reference only)

**Problem.** A language model asked "what does this character do next" returns free text. Treated as a
command, that text can name people who are not there, invent offers, forge memories, or move a person. A
controller that is one prompt and one parse has no place to put the rules, the tests or the failure
handling.

**Options considered:**

```text
(a) one prompt, one parse; the reply is sent as the action            open to every failure above
(b) an agent framework (Concordia, LangGraph, AI Town) as the loop   the framework owns the execution and
                                                                      the memory model; reference only (DEP-46)
(c) small typed parts (persona, situation, memory view, choice
    prompt, parser, validator, intent builder); the reply is a         chosen
    proposal the owning contracts check
```

**Choice: (c).** The controller is a Python Controller Pack. Its parts are frozen Pydantic models or pure
functions, with no clock, randomness, environment or other seat's store read. A model's answer becomes a
`talk` utterance or the unchanged payload of a complete affordance the server offered, and nothing else.
The controller builds an `ActionRequest` only through `mineworld_sdk.offers`, which refuses any affordance
not offered by the newest observation. It never builds a movement request (`ARC-32`, step-17 I-15). The
prompt holds no entity id and no provider concept; the request's canonical JSON is the cassette key
(`ARC-58`).

**Consequences.** A model that "decides" anything else has no representable proposal (`INV-10`). Every
answer is validated before it becomes an intent. A new social choice needs a `SocialChoice` value and a
validation row, not a change to the kernel or the server. Nothing in the world reads the controller.

**Revisit** when a second model-driven controller needs the same parts, which then move into a shared
module, or when the offer protocol (`ARC-34`) gains a movement affordance that the controller must decide.

## ARC-72 note — a filler builds the payload of an incomplete offer (2026-10-26, S31-d)

A model's answer may now also become the payload a **filler** builds from checked, typed fields, for an incomplete
offer the observation makes available (`ARC-183`). The model never writes a payload; the request still goes only
through `mineworld_sdk.offers`; the controller still never builds a movement request. A filler may raise a `due`
consult on named perceived facts (the `publish-notice` filler: calendar's `day-began`). When no filler is listed,
every request is exactly what it was before.

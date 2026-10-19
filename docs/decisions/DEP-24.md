## DEP-24 — Pydantic for the Python wire models

**Date** 2026-10-08 · **Status** adopted by S10 PR P3 (`pydantic>=2.13,<3`; locked 2.14.0) · **Approved
by** the primary session at P3's freeze (QP3-7) · **Licence** MIT · **Relates to** `ARC-56`, `ARC-41`
· **Design** step-17 §4.2, §4.7 ("Wire models"); P3 D-P3-3, D-P3-7 · **Placeholder** `DEP-S10-b`

**Problem.** The Python SDK holds every revision-2 frame as a typed value, decodes what the server
sends strictly (unknown fields refused, ids only as decimal strings, integers never floats, closed
enumerations, the contract's cross-field rules), and re-encodes to exactly what the Rust types write.
Later S10 PRs need JSON Schema generation for structured model output from the same library.

**Options considered** (`REUSE_POLICY.md` §§11–12):

```text
(a) Pydantic 2 (MIT; the ecosystem standard; pyright understands its models)
(b) msgspec (BSD-3-Clause; faster)
(c) dataclasses plus our own validation
```

**Choice: (a).** Strict models with discriminated unions; validation is local and deterministic.
- **(b) declined:** speed is not a constraint at 10 Hz per seat, and it would be a second schema library
  once P6 needs Pydantic's JSON Schema for structured output.
- **(c) declined:** re-implementing strict decoding, discriminated unions and schema generation is
  exactly the wheel `REUSE_POLICY.md` §12 forbids rebuilding.

**Isolating interface.** `mineworld_sdk.wire`: callers name the SDK's model types and `codec.encode` /
`codec.decode`; how a model validates is not part of the SDK's interface.

**Revisit** if decoding cost is measured to matter, or Pydantic 3 changes the model API.

## ARC-99 — A person's size is the seat view, counting shared records in full

**Date** 2026-10-14 · **Status** accepted (step-28 design frozen 2026-10-14); implemented by S28-a and S28-b · **Approved by** the operator's ruling QS28-4 (2026-10-14) and the 20 MiB default (operator, 2026-10-14) · **Relates to** `ARC-59`, `ARC-86`, `ARC-87`, `ARC-98`, `ARC-100`, `INV-13`, `pr-fg-b2-size-trigger.md`, [`ARCHITECTURE.md`](ARCHITECTURE.md) §9.3, [`MODULE_SPEC.md`](MODULE_SPEC.md) §5.2 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §§3 (I-S28-2, I-S28-3, I-S28-6), 7

**Problem.** Once a fact record is shared, the bound must say what it measures. A fair share (bytes divided by holders) couples one person's memory to everyone else's perception: Alice's size would fall when Bob perceives a scene and rise when he forgets it, which could trigger Alice's size pass and change what Alice remembers because of Bob. That breaks I-S28-3, makes Alice's memory a function of the town rather than of her perception (`ARC-59`), and makes a shared Alice differ from a standalone Alice (I-S28-2).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Fair share of each shared record (bytes ÷ holders) | Declined: the coupling above |
| (b) Count only the private rows | Declined: the records a person holds would be uncharged, and the 20 MiB promise would not bound what the person remembers |
| (c) **The seat view, with every held record counted in full** | **Chosen** |

**Choice.** A person's size is the seat view, counting shared records in full. Sharing is transparent (I-S28-2, I-S28-3) and is a deployment setting, not a world rule. The bound applies to that size. Physical bytes are at most the sum of the seat views, and that sum is at most the number of seats times the bound (I-S28-6). The physical share of a record is reported by `inspect bytes` as a diagnostic and is never charged.

**Consequences.** The 20 MiB promise keeps its literal meaning: each person may remember up to 20 MiB, measured as if alone. At the positional encoding of `ARC-101` that is about 106 days of undigested detail, against about 45 at today's encoding (step-28 §8.3). Sharing still saves the world's disk: measured on social-cafe at seed 7 over 30 days, the world's physical bytes are 59 % of the logical sum at today's encoding and 23 % with `ARC-101` (step-28 §§7.1, 8.2).

**Limitation accepted.** The saving is measured only on 12-person worlds. No World Pack on `main` has more than 12 people (F-S28-11), so the saving at 100 people is unmeasured until the S28-g fixture exists.

**Revisit** when a world with more than 12 people is measured (AS28-17), or when a world-level budget is proposed, which would need its own decision.

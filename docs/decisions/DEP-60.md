## DEP-60 — Reserved: one new CC0 texture for the station, only if a reviewed frame needs it

**Date** 2026-10-15 · **Status** reserved; withdrawn at S29-d's merge if unused · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-29 Q-S29-10: ≤ 6 MB, target 0 new textures) · **Relates to** `DEP-8`, `ARC-106` · **Design** step-29 §6.4

**Problem.** The depot is built from the shipped Poly Haven CC0 set. A reviewed frame might show that a material cannot be read from that set.

**Choice.** At most two Poly Haven CC0 1k textures, each ≤ 3 MB and each checked under `DEP-8`. Each is added only on that evidence, and S29-d records it here, replacing this entry's status. If none is added, this entry is marked withdrawn and the number is not reused.

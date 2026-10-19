## ARC-2 — The style manifest is generated from references, which constrains the schema

**Date** 2026-09-25

Style authoring is meant to be: upload 4–10 references, write two or three sentences, choose 2D or
3D, and let the system generate an initial manifest
([`ART_DIRECTION.md`](ART_DIRECTION.md) §23).

**Consequence, and the reason this is a decision rather than a feature note:** every manifest
field must be *inferable from images*, *chosen by the creator*, or *fixed by project policy*.
Auditing the §15 example against that rule already separates three kinds of field, and exposes
that `readability_priority` and `human_scale` were never style at all — gameplay readability
outranks decorative density for every MineWorld style, so they are project invariants a style
restates rather than choices a style makes.

**Applied as a rule:** a proposed manifest field that is none of the three is rejected. This is
what keeps §15's "do not overengineer an exhaustive universal art ontology" enforceable instead of
aspirational.

**Limitation:** the analysis reads style only. It may never infer simulation content from a
reference image — a café with outdoor seating says nothing about whether `Sit` exists in a world.

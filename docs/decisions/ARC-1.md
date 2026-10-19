## ARC-1 — A Presentation Style Pack is not a sixth pack type

**Date** 2026-09-25

[`ART_DIRECTION.md`](ART_DIRECTION.md) §12 introduces the "Presentation Style Pack" as a
first-class concept, while [`MODULE_SPEC.md`](MODULE_SPEC.md) §1 states that exactly five pack
types exist and that adding a sixth is a deliberate design change.

**Resolved as: the same type, now with a specified structure.** A Presentation Style Pack is the
internal layout of the Presentation Pack that `MODULE_SPEC.md` §6 already defines — manifest,
references, art direction, asset bindings, material rules, character style, lighting profile,
renderer configs. The name is used where the art-direction aspect is what matters.

**Why not a sixth type.** Nothing in §12 describes a different *kind* of module: it does not
add entities, rules, controllers, or world content, and it is installed and swapped exactly as a
Presentation Pack is. Adding a type to the core ontology to describe a directory layout would be
the ontology bloat the project explicitly guards against
([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §1).

**Recorded because** a future contributor reading §12 alone would reasonably conclude the
ontology has six types. It has five.

## ARC-7 — Asset Pack is a sixth pack type

**Date** 2026-09-26 · **Extends** `MODULE_SPEC.md` §1 · **Relates to** `ARC-1`

`ARC-1` refused to make a Presentation Style Pack a sixth type. This one is accepted, and the
difference is worth stating because it is the test any future seventh must pass.

A Presentation Style Pack described the **internal layout of an existing type**: it added no new
kind of thing, installed identically, and named a directory structure. An Asset Pack is a
different kind of thing: separately authored, separately licensed, separately versioned, and
**reused by packs that share no style**. The same bench, tree and café model serve a
semi-realistic world and an illustrated one. Folding them into Presentation Pack would force one
copy of every model per style, which is the concrete cost that justifies the concept.

**Rule extracted for next time:** a new pack type is justified by a distinct *lifecycle* —
authored, licensed, versioned and reused independently — not by a distinct folder.

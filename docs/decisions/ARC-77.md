## ARC-77 — Milestone E's composition: Lakeside, the repository's own data packs, and named roots everywhere

**Date** 2026-10-10 · **Approved by** the primary session at PR E-e's design freeze (step-16 §18.0;
FQ-e1 … FQ-e8) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §2;
[`ARCHITECTURE.md`](ARCHITECTURE.md) §14; [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 · **Relates to**
`ARC-33`, `ARC-34`, `ARC-53`, `ARC-54`, `ARC-55`, `ARC-66`, `ARC-71`, `ARC-49` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §1.4, §8.2, §9.6, §18 (S16, PR E-e)

**Problem.** E-a … E-d made every mechanism of Milestone E — pack identity, requirements, a System Pack
from another repository, Entity Packs — but no world used them together, so "a real world assembled from
independently installable packs" was still a claim about parts. Composing one raises three questions
the mechanisms leave open: where the repository's own data packs live, whether they are "bundled", and
how every program that reads a world finds packs that are, by `ARC-54`, never found implicitly.

**Choice.**

1. **Lakeside is Milestone E's composition.** `worlds/lakeside` is assembled from bundled System Packs
   (presence, movement, conversation, group-activity, relationships, naming, schedule, item, inventory,
   item-transfer, consumption), the third-party `fishing` System Pack (`ARC-66`), the Entity Pack
   `modern-goods` and the Presentation Packs `mineworld-default-2d` and `-3d`. Its `requires:` names the
   third-party pack, the Entity Pack and both Presentation Packs, each with a range. Its acceptance is
   step-16 §8.2 (M-1 … M-10), held by `tools/cli/tests/milestone_e.rs` through the real binary and server.
2. **The repository's own data packs have one home per type.** Entity Packs live under `entities/`
   (`entities/modern-goods`), beside `presentation/` for Presentation Packs. Each is found only through
   a named pack root (`--packs entities`, `--packs presentation/mineworld-default`, or
   `MINEWORLD_PACKS`): living in this repository changes nothing about how a data pack is found.
3. **"Bundled" keeps `ARC-54`'s meaning.** A bundled pack is a code pack compiled from this workspace,
   versioned with the framework, and may not be listed in `requires:`. A data pack is never bundled,
   wherever its files live: a world that uses `modern-goods` requires it with a range, exactly as it
   would a data pack from anywhere else. "The repository's own" is description, not a category.
4. **Every program that reads a world names its roots.** The milestone test passes the roots to every
   command and removes `MINEWORLD_PACKS` from every child it spawns; the tests that enumerate `worlds/`
   pass `--packs entities` too; AC-8's instrument (`scripts/ci_parity.py`, `ARC-49`) and the image check
   (`scripts/ci_image.py`) pass `--packs entities --packs presentation/mineworld-default` for every
   world, and the runtime image carries `entities/` and the two Presentation Packs' `pack.yaml` and
   `manifest.yaml` (not their art). A world that requires nothing is unaffected by roots it does not use.
5. **The catch is the world's; the goods are the pack's.** `fish`, what Lakeside's water yields, is the
   world's own item kind; `modern-goods` holds shared shop goods and must not declare it (`ARC-71`
   rule 5 would refuse the collision).
6. **No product code.** Milestone E's world and test change no crate's source or manifest: every
   behaviour they rely on merged with E-a … E-d. The rule controller is not taught the new pack
   (`ARC-34`): seats fish because `fish` is offered complete and their routines take them to the water.

**Options considered.** Copying Market Town's item files into `modern-goods` — rejected: a second
statement of each kind, and Market Town keeps its own. Calling the repository's data packs "bundled" and
letting worlds use them unrequired — rejected: it reuses a defined term for another concept and brings
back an implicit search. Dropping Lakeside's Presentation Pack requirements so the image needs only
`entities/` (FQ-e2) — rejected: the requirement is how a world declares what it is authored for
(`ARC-54`), and the image cost is four small files. Skipping worlds that need roots in AC-8 — rejected:
it would exempt exactly the composed world from the parity claim.

**Accepted limitations.**
- No economy or employment in Lakeside, so `modern-goods`' kinds enter only as people's starting
  holdings; fish is the only renewable food, and the milestone test measures that it closes the loop.
- The reference clients do not draw Lakeside: a Presentation Pack is declared and validated, not yet
  applied (step-16 QSE-9).

**Note, 2026-10-12 (S15, PR 12d; step-11 §19.13 TD-D12 item 3, the operator's ruling) — Lakeside after
item names.** Lakeside's own `fish` kind gains its display name (`Fish`), and its `requires` moves to
`modern-goods: "^0.2"` (`ARC-71` note of the same date). Lakeside's facts change by the names alone;
the composition, its packs and the milestone's claims are unchanged.
- A saved 300-day run of a world this size is gigabytes (F-SAVE-1); the persistence lane owns retention.

**Note, 2026-10-20 (S33-b; step-33 Q-S33-8) — Lakeside's history moved once, for bodies.** Lakeside
installs `bodies` and its doorway points move to 400 mm (`ARC-39` note 9), so every one of its baseline
keys moved, recorded once with a `reason S33-b` line in `scripts/baselines.txt` (after milestone F's
re-record; facts-300s 6 805 → 11 936 rows). The composition's
claims are unchanged; `bodies` is one more bundled System Pack. Removing it (with every `body:`) gives
exactly the world after the doorway move alone (Q-S33b-1, pinned by `tools/cli/tests/lakeside_bodies.rs`).

**Revisit** when a registry or `.mwpack` (E-C) gives data packs a place outside this repository, or when
the clients apply Presentation Packs.

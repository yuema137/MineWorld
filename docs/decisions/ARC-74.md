## ARC-74 — The `persona` System Pack: a person's authored character, which shapes speech and is disclosed to the person alone

**Date** 2026-10-14 · **Status** accepted (design, S10 P6 revision 2, §5.14); implemented by P6a · **Approved by** the operator (QP6-1, QP6-2, 2026-10-14) and the primary session of S10 P6 · **Relates to** `ARC-28` (point 6, self-only disclosure), `ARC-31` (a System Pack owns a section of an authored file), `ARC-33` (the installed set), `ARC-72` (the LM controller's composition), `CORE_CONCEPTS.md` §4.2 (personality is a trait) · **Design** `pr-s10-p6-lm-controller.md` §§5.14, 6 (I-P6-1), 8 (AP6a-1 … AP6a-3)

**Problem.** A person has a name (`naming`) and a routine (`schedule`), but nothing in the world says what they are like. A controller that speaks for a person has no source of character to speak from. `note` is authoring provenance, "never gameplay state" (`MODULE_SPEC.md` §4.1), and no System reads it, so it cannot carry one. Putting the character in the operator's cognition configuration would make it a property of one operator's machine, not of the world.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A System Pack that owns a `persona:` section of the person's file, and a `Persona` component disclosed to its holder only | **Chosen** |
| (b) The operator's cognition configuration (`cognition.toml`) | Declined: a person's character is world content. It would be lost when the world moved to another operator, and a world without a controller would have no character at all |
| (c) `note:` | Declined: `note` is authoring provenance and is never read as state. Making it state would mean a System reading prose it does not own |

**Choice.** Option (a). The character is world content, so it is authored in a person's file and owned by a System, as `CORE_CONCEPTS.md` §4.2 places personality among traits. The pack has no action, no process and no reaction. Its `Persona` component is disclosed through `PerceptionProvider::discloses` to its holder and to nobody else, as `ARC-28` point 6 discloses relationships. It shapes speech only (`R-S10-9`): it grants no ability, moves nobody, and a rule controller ignores it.

**Consequences.** Any world that enables `persona` gets a genesis fact per authored person, private to that person (`Visibility::Entities({holder})`), so its save bytes change; the baseline rules of `scripts/baselines.txt` apply to such a world. A World Pack author may write a character in any person's file without changing a rule, and a world without any `persona:` section is unchanged. A controller that wants to use the character reads it through the holder's own observation; no other observer sees it.

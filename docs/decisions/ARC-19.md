## ARC-19 — The default character is an identity reconstruction; the framework's bar stays low

**Date** 2026-09-27 · **Supersedes** part of `ARC-4` · **Relates to** `ARC-17`,
`ART_DIRECTION.md` §§3, 7, 10, 16

`ARC-4` scoped `3D/references/04_character_closeup.png` as **not** authoritative for facial
fidelity, skin rendering or hair simulation, and fixed the default 3D character at "realistic
proportions with moderately simplified face and materials, medium detail — no photoreal skin, no
MetaHuman-level assets, no bespoke character pipeline."

The operator has since rejected a candidate character **specifically on face, hair, freckles and
garment structure**, and has designated that image the identity source of truth for the default
character. `ARC-4`'s scoping and the current requirement cannot both stand, and leaving the
contradiction unrecorded is the defect `CLAUDE.md` §2.1(4) names.

**Decided: `ARC-4`'s facial-fidelity exclusion is superseded for the default character, and
`ARC-4`'s reasoning is preserved by separating two things it treated as one.**

`ARC-4`'s argument was that a photoreal face standard puts character production beyond what
community creators can afford, contradicting why a default style exists at all
(`ART_DIRECTION.md` §10). **That argument is correct and is not overturned.** It was applied to
the wrong object. It is an argument about **what the framework requires of every creator**, and it
was used to cap **what this project's own default character may be**. Those are separate:

| | Bar | Who pays it |
| --- | --- | --- |
| The MineWorld default character | as faithful to `04_character_closeup.png` as practical — face, hair, freckles, garment structure and accessories included | this project, once, and the result ships as an asset |
| What the framework *requires* | the humanoid profile only: scale, axes, root convention, skeleton, retarget compatibility, glTF expectations | every creator, and it stays cheap |

A creator shipping their own world needs a rig that satisfies the profile. They do not need our
face. **The humanoid profile standardises the runtime contract, never appearance** —

```text
same skeleton  ≠  same mesh  ≠  same face  ≠  same clothes
```

— so one animation library serves characters who look nothing alike. A high-fidelity default is
therefore an example of what the framework permits, not a threshold it imposes, and `ARC-11`
already says the default style is ours and never a kernel assumption.

**Consequences.**

1. `04_character_closeup.png`'s `not_authoritative_for` list drops `facial_detail`,
   `skin_rendering` and `hair_simulation` **for the default character**. Its `authoritative_for`
   dimensions are unchanged. `ARC-4`'s mechanism — a reference declares what it is a reference for
   — is untouched and remains correct; only this one image's scoping changes.
2. `ART_DIRECTION.md` §§3 and 7 need amending where they rule out cinematic facial rendering for
   the default character. A prose section and a decision disagreeing is the same defect as two
   decisions sharing an id.
3. The prohibition on a **bespoke from-scratch character pipeline** stands, on
   `REUSE_POLICY.md` grounds rather than fidelity grounds: evaluate mature tools — MetaHuman,
   Character Creator, Blender with MPFB, image-to-3D reconstruction, CC0 groom and animation
   libraries — against quality, automation, licence, redistribution, engine portability and
   runtime compatibility. Do not build character-reconstruction technology.
4. **`DEP-8` is not relaxed.** An asset must be free to *redistribute*, not merely free to use. An
   ecosystem asset usable only inside one engine is recorded as exactly that, and engine-specific
   Presentation Packs stay isolated from portable MineWorld assets.

**Cost accepted deliberately.** A higher default bar means the default character is expensive to
reproduce and a contributor cannot casually regenerate it. That is the trade `ARC-4` refused, and
it is accepted now because the artefact is shipped rather than re-derived, and because nothing
about it reaches the framework's own requirements.

## DEP-74 — Rejected as the wardrobe base or garment generator, with reasons: CharMorph, VRoid/VRM, Quaternius, Kenney, Synty, Character Creator, Ready Player Me, Hunyuan3D-Part, Garment3DGen, Meshy Auto Split

**Date** 2026-10-18 · **Status** accepted (step-32 approved 2026-10-15); recorded by S32-p · **Approved by** the operator's step-32 rulings (2026-10-15, §20.1) and the freeze of `pr-s32p-wardrobe-pilot.md` · **Relates to** `DEP-8`, `DEP-51`, `DEP-55`, `DEP-72`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design** `.structured-coding/plans/mvp0/step-32-wardrobe.md` §§4.1–4.2 (facts read at source on 2026-10-15; URLs in §4.7)

**Problem.** `CLAUDE.md` §4 rule 16 asks both halves of the reuse question: are we reinventing a mature wheel, and are we forcing one where it does not fit? Each candidate below was read at its source on 2026-10-15 and fails on licence, platform, maintenance or style. The rejection is recorded so a later session does not re-open it without new facts.

**Rejected, with the deciding fact.**

| Candidate | Deciding fact (step-32 §4.1–4.2) | Verdict |
| --- | --- | --- |
| **CharMorph** (the old town body's tool) | Code GPL-3; the Vitruvian body CC0 but ships only `Pants` and `Shirt`; MB-Lab bodies AGPL, "the AGPL will propagate … to the output models"; "unusable" on Blender 5.0.1, hair broken on 5.2 (issue #59); "doesn't really have a maintainer currently" (2026-09-18) | Rejected as the wardrobe base. It stays only as the historical source of the 52-joint rig (`DEP-55`) |
| **VRoid Studio / VRM** | pixiv's terms: content "is not CC0", and "You cannot create an application that can generate or output 3D models, avatars, or items consisting of deformed or combined meshes and textures that were created in VRoid Studio" without a separate licence — a runtime wardrobe is that application; anime head and MToon do not match the art direction | Rejected (licence blocks the use case; style) |
| **Quaternius** Universal Base Characters and Modular Outfits | quaternius.com now shows the "Quaternius Asset License v1.0" (8/28/2026): "You may not … redistribute the Assets … as a standalone asset, asset pack …"; ≈ 65 UE5-style joints; stylized low-mid poly | Rejected for the default pack; a user pack may use it under its own terms |
| **Kenney** Animated Characters | CC0, but blocky low poly | Rejected (style) |
| **Synty** | EULA: "You must not share the source files of any Assets outside your team"; no upload to generative services | Rejected (licence) |
| **Character Creator** (Reallusion) | exported content "must not be available for public download" (secondary source) | Rejected (licence, as on 2026-10-01) |
| **Ready Player Me** | services shut down 2026-01-31; the domain no longer resolves | Rejected (gone) |
| **Hunyuan3D-Part** (P3-SAM, X-Part) | "You must not use … Output or results … outside the Territory" (excludes the EU, the UK and South Korea); CUDA | Rejected (the territory clause binds outputs) |
| **Garment3DGen** | CC BY-NC 4.0; CUDA | Rejected (non-commercial) |
| **Meshy Auto Split** | "Textured models aren't supported for splitting yet"; aimed at 3D printing | Rejected as a garment-layer separator (Meshy image-to-3D of one garment per run stays, `DEP-73`) |

**Not adopted, recorded beside the rejections** (no licence defect; a capability gap today): PartCrafter, SAMPart3D, HoloPart (MIT, CUDA only: revisit on a CUDA machine); Tripo segmentation (beta; texture preservation not stated); GarmentCode / pygarment (patterns without a usable simulator; noted for a future tailoring step); Blender Studio Human Base Meshes (CC0, unrigged, no garments: kept as a sculpt base under `DEP-8`); Godot modular-character addons (dead, paid, or static-mesh mergers only).

**Choice.** None of the above is the wardrobe base. The base is our own modular character on the 52-joint skeleton with MPFB2 as a hidden fitting tool (`DEP-72`), garments generated at Vera's tier (`DEP-73`, S32-p), and the Godot-native runtime (`DEP-75`, S32-c).

**Consequences.** No third-party character licence constrains the default pack. A user pack may still adopt a rejected source under its own terms, with its licence recorded in provenance.

**Revisit** when a rejected candidate's licence or maintenance changes, or when a CUDA machine makes the part-aware generators testable.

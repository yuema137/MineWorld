## ARC-92 — The cast binding lives in the Presentation Pack; one resolution order; the fallback is the extras pool

**Date** 2026-10-14 · **Status** accepted (step-26 freeze, operator Q-S26-8, Q-S26-9); specified by S26-a; read by the 3D client from S26-c2 and by the 2D client from S26-e · **Approved by** the operator (Q-S26-8, Q-S26-9, Q-S26-11) · **Relates to** `ARC-3`, `ARC-9`, `ARC-46` (amended, Note 1), `ARC-91`, `DEP-16`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1 · **Specified in** [`clients/3d-spike/PRESENTATION.md`](../clients/3d-spike/PRESENTATION.md), [`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) §3.1 · **Design** `.structured-coding/plans/mvp0/step-26-cast.md` §§4.3–4.4, 8

**Problem.** `ARC-91` gives every person a disclosed look key. Something must turn a key into a character in 3D and a sprite set in 2D, without the world learning anything about art, and with a fallback for a person whose look a pack does not know that is never a cruder kind of body.

**Choice.**

1. **3D: `characters/cast.yaml`** in a 3D Presentation Pack, in YAML's JSON-compatible subset: `format` (1), `looks` (look key → character id), `extras` (character ids: the fallback and decorative pool), `default` (the character used when no world is connected). A character id names `characters/cast/<id>/` in the pack; its meshes stay resources of the client that imports them (the 3D client cannot load a skinned, retargeted model from a runtime pack today).
2. **2D: a `look:<key>` role** in the `people` binding set. It is a disclosed component value, the same class of input as the tag in `facade:<tag>`, never a world key.
3. **One resolution order in both clients.** A person whose look the pack binds is drawn as that character or sprite. A person with no look, or one the pack does not bind, is drawn as `extras[posmod(hash(id string), len(extras))]` in 3D and as `person:<n>` by the same hash in 2D. A pack with no extras (3D) or no `person:<n>` (2D) uses its `default` (3D) or plain drawing (2D). With no pack, plain drawing. The observer is resolved the same way, so a player is drawn as the Person they occupy; the 2D `player` role is only the fallback for an observer with no bound look.
4. **The fallback is never a cruder model class.** Every `extras` entry is a character made by the same pipeline and held to the same standard as a named person (step-26 §6, which S26-b1 records as `ARC-94`). In the default pack, the café street's extras are the lakeside cast (Q-S26-8, E-2), so no person of the present world appears twice.
5. **A pack decides nothing.** A binding changes how a person looks, never which requests a client sends, and a look is never shown as text.

**Consequences.** The default 3D pack ships an empty but valid binding until its characters exist; `scripts/check_cast.py` checks its structure. Adding a world's cast is an edit to one pack file, never to a client or a world.

**Revisit** when 3D Presentation Packs become loadable at runtime (the binding then also names the meshes), or when a world that is neither the café town nor the lake needs its own extras (E-1).

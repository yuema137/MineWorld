# PR VF-PID — Person identity is the server's in both clients (visual-fidelity defect fix)

## Status

```text
Kind:            defect fix inside frozen designs (ARC-91, ARC-92; S26-a pr-s26a-appearance.md, frozen
                 2026-10-14; clients/2d/PRESENTATION.md §3.1; clients/3d-spike/PRESENTATION.md §§1-3)
Branch:          fix/vf-person-identity, from origin/main @ 70508683
Scope status:    the resolution order both clients implement is already frozen (ARC-92). Three points
                 below need an operator ruling (§6); each is shipped with its recommended answer and is
                 a one-line data revert if ruled otherwise.
```

## 1. Defect (operator, 2026-10-18)

"2d 的主角是男的，3d 的主角是女的，然后咖啡店员的性别也不一致 … 人物 地点之类的属性不应该有所变化". The
intro video (origin/main @ 370b9da0) shows the visitor seat as a young man in 2D and as a woman in 3D;
the barista's sex differs between the clients; in 3D the label "Ivan Petrov" stands over a woman.

## 2. Root cause (audited on origin/main @ 70508683)

The server is consistent: `worlds/social-cafe/people/visitor.yaml:11-12` and the market-town copy
disclose `name: Vera Lindgren`, `look: vera-lindgren` for the `visitor` seat; Alice is `alice-moreau`;
every named person has a look (S26-a). **Neither client read the disclosed `look`.** S26-c2 (3D) and
S26-e (2D), which implement ARC-92's resolution, were not built, and in the meantime each client chose
bodies by rules of its own:

| Client | File | What it did |
| --- | --- | --- |
| 2D | `clients/2d/scripts/scene/people.gd:182-183` | observer → role `player` (bound to `gen_player`, which is Wes Calloway's outfit: amber jacket, teal trousers), whoever the seat is; everyone else → `person:<hash(id) mod 9>`, a random generic sprite of either sex |
| 3D | `clients/3d-spike/scripts/slice/character_slot.gd:49` (via `slice_player.gd:22`) | the player is always `NPC.make(..., reference = true)`: the route D+ Meshy body, which is Vera Lindgren (`presentation/mineworld-default/3D/characters/cast/vera-lindgren/fit.yaml`), whatever seat is taken |
| 3D | `clients/3d-spike/scripts/slice/slice_link.gd:416` | every perceived person → the one CharMorph town body (`human.gd` `Body.TOWN`, baked through the Ultra Feminine morph, `human.gd:44-45`) with colours from `hash(id)` |

Hence: the visitor (Vera) is a man in 2D (Wes's sprite) and Vera in 3D only by coincidence (seat
`wanderer` would also be Vera in 3D); the barista is a random sprite in 2D and the feminine town body in
3D; Ivan's label is correct (the label is on his own figure, `display-name`), but his body is the
feminine town body. Labels are not mis-assigned in either client: both read `display-name` only.

## 3. Fix (this PR)

1. **One accessor for the disclosed look**, in the protocol module both clients link
   (`clients/protocol/mineworld/observation.gd` `look(id)`), beside `display_name`.
2. **2D** (`people.gd` `role_for`): ARC-92's order exactly — `look:<key>` if the active set binds it,
   else `player` for the observer, else `person:<n>` by hash, else plain drawing.
3. **3D** (`scripts/slice/cast_binding.gd`, `SliceCast`): reads `characters/cast.yaml`; the slice's
   figures **and the player** are resolved through it (`slice_link.gd` `_figure`, `_wear`;
   `character_slot.gd` `recast`; `slice_player.gd` `wear`). A character becomes a body through the
   client's resource index `clients/3d-spike/assets/characters/cast/index.json` (PRESENTATION.md §1).
4. **Pack data.** 3D `cast.yaml`: `looks: { vera-lindgren: vera-lindgren }` (the only character that
   exists). 2D `asset_bindings.yaml` set `people`: a `look:<key>` role for all 22 looks of café town and
   lakeside, each bound to the existing generic sprite closest to that person's casting brief (sex first,
   then age band and outfit; `gen_player` is bound to `look:wes-calloway`, whose outfit it already is).
   No world data changes; no world key appears in client code.
5. **Cross-client test** `clients/shared/checks/identity.sh` (worlds `social-cafe` and `market-town`,
   every seat): the 2D and the 3D client each seat every seat on a fresh server and print the observer
   and every perceived person as `(id, disclosed name, disclosed look)`; the two outputs must be equal and
   non-empty; each client additionally fails when its real resolver draws a person whose look the pack
   binds by any other role/character.

## 4. Invariants this PR enforces

- I-PID-1. Everything about a Person a client shows — identity, name, look — comes from the
  observation (`display-name`, `look`). A client chooses only how to draw the look key.
- I-PID-2. The observer is resolved like everyone else (INV-1): no seat, slot order or "player" default
  decides which Person a body is, except ARC-92's own fallback when the look is unbound.
- I-PID-3. Both clients use the same order and the same stable hash of the id string.

## 5. What remains (outside this PR)

- **3D men.** The 3D client has one non-reference body, and it is feminine. Until per-person characters
  exist (S26-d B1) or a second town body exists, every unbound person — every man — is drawn on it in 3D.
  This PR makes the 3D *resolution* correct and data-driven; the *asset* gap is §6 Q-PID-3.
- **16c stack.** PR #245 moves figure construction from `slice_link.gd` `_figure` to
  `scripts/slice/figures.gd` `_build`. When either lands second, `_build` must take the observation's
  look and build `NPC.make(rng, STAND, 1.72, cast.is_reference(cast.character_for(look, id)))` exactly as
  `_figure` does here.

## 6. Operator questions (shipped with the recommended answer)

- **Q-PID-1 (2D stand-ins).** Until S26-e renders each cast member's own sprite, may several looks share
  one generic sprite chosen by the casting brief's sex and age (e.g. Bob and Ivan both on the old man)?
  *Recommended: yes* — a correct-sex, correct-age stand-in is strictly better than a random one; S26-e
  replaces each binding one line at a time.
- **Q-PID-2 (3D fallback).** `clients/3d-spike/PRESENTATION.md` §3 says an unbound look with no extras
  draws as `default` (Vera). Applied literally, the whole town would be Vera. *Recommended:* an unbound
  look with no extras draws as the client's town figure, and `default` keeps its §2 meaning (no world
  connected). Shipped that way; amend §3 on approval.
- **Q-PID-3 (3D bodies for men).** Options: (a) start S26-d B1 (Meshy, café-town cast, inside the
  approved 3,000-credit cap) now; (b) interim, no spend: bake a masculine CharMorph town body in Blender
  and bind it as an extra chosen by brief sex. *Recommended: (a)*, with (b) only if B1 cannot start this
  week.

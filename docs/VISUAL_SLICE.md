# The visual slice: the shared target for every 3D presentation track

**Status:** binding specification for `VIS-3D-GODOT-2` and `VIS-3D-UE5-1`
**Audience:** coding agents building a 3D reference client slice.
**Authority:** [`ART_DIRECTION.md`](ART_DIRECTION.md) governs the look;
[`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) governs how a comparison is performed and reported;
[`ACCEPTANCE.md`](ACCEPTANCE.md) governs who decides what;
[`DECISIONS.md`](DECISIONS.md) `ARC-20` requires that the two engine tracks be comparable
like-for-like. This document is what makes them comparable.

---

## 0. Why this document exists, and what is forbidden in it

`ARC-20` §2 requires that a Godot slice and an Unreal slice be judged against **the same scope**,
because *"a stale placeholder against a polished slice measures nothing"*. Two tracks cannot be
the same scope if each track decides its own scope. So the scope lives here, once.

`VIS-3D-UE5-1` is **parked, not cancelled** (`ARC-21`). This document is written on the assumption
that it restarts.

**Nothing engine-specific may be written into this document.** Not a renderer name, not a lighting
technique, not an import setting, not an asset format, not a node type, not a material graph, not
a shader. Those are implementation, they differ per track by design, and recording them here would
silently turn one track's method into the other track's requirement. They belong in the
implementing branch's own notes.

The distinction to apply when in doubt:

```text
belongs here      there is a café doorway the player crosses on foot
belongs here      the interior is legible without the exterior being blown out
does NOT belong   the interior is lit by a baked lightmap with a reflection probe
does NOT belong   glass uses a roughness of 0.04 and a screen-space reflection
```

## 1. The one-sentence scope

> **One short street frontage with one café you can walk into, furnished, lit, and inhabited at
> the scale of a person standing in it — and one more shop that opens too (§4.1).**

Small and complete beats broad and shallow ([`ACCEPTANCE.md`](ACCEPTANCE.md) §5). Cutting content
to buy a real interior is the correct trade; the reverse is not.

**The town is not to be expanded.** A track that adds a second street, a harbour or a plaza has
left this specification, and its slice is no longer comparable with the other track's. The one
second interior the slice has is **required** of every track, not added by one of them (§4.1).

## 2. The references that govern the slice

| Reference | Governs |
| --- | --- |
| `presentation/mineworld-default/3D/references/03_cafe_frontage.png` | **primary.** The café frontage, its joinery, its signage, its outdoor seating, and everything visible through its glass |
| `.../05_main_street_golden_hour.png` | the street: massing, storey rhythm, shopfront register, paving, street furniture, planting, and the time of day |
| `.../01_lakeside_promenade.png` | the third-person walking camera, the paving, and the density of small objects along a frontage |
| `.../02_town_square.png` | planting density, banners, and the masonry idiom of secondary façades |
| `.../04_character_closeup.png` | the character only. Governed by [`CHARACTER_IDENTITY.md`](../presentation/mineworld-default/3D/CHARACTER_IDENTITY.md), **not** by this document |

These are read **as images, at magnification**, before any modelling
([`ACCEPTANCE.md`](ACCEPTANCE.md) §3.1). A prose description of a reference — including this
document's §§3–9 — is a checklist, never a substitute for the pixels.

## 3. Scale contract

One world unit is one metre ([`DECISIONS.md`](DECISIONS.md) `DEP-8`, coherence procedure step 1).
Every dimension below is a requirement, verified by measurement and not by eye. The tolerance is
±10% unless a tighter one is stated.

### 3.1 The human yardstick

```text
standing eye height        1.60 – 1.70 m
total stature              1.70 – 1.80 m          (ACCEPTANCE.md §4.4)
shoulder width             0.42 – 0.50 m
walking speed              1.3 – 1.5 m/s
jogging speed              2.6 – 3.4 m/s
collision capsule radius   0.30 – 0.36 m
step-up the player clears  ≤ 0.25 m
```

A permanent 1.7 m reference object must exist in the scene, or be producible on demand, so that
any later measurement can be taken against it rather than against memory.

### 3.2 Architecture

```text
café interior floor-to-ceiling     3.0 – 3.6 m
door leaf                          0.85 – 1.00 m wide, 2.05 – 2.20 m tall
door head above the threshold      > stature + 0.25 m
entrance step riser                0.10 – 0.18 m, one step only
shopfront glazing head             2.6 – 3.0 m above the pavement
stallriser (solid below glass)     0.20 – 0.45 m
fascia / signboard band            0.70 – 1.10 m tall
ground-storey total height         4.0 – 4.6 m to the top of its cornice
upper storey height                3.0 – 3.6 m each
café frontage width                7 – 11 m
café interior depth                8 – 13 m
```

### 3.3 The street

```text
pavement width, café side          3.0 – 4.5 m clear of furniture
kerb upstand                       0.10 – 0.16 m
carriageway width                  6.0 – 8.0 m
frontage-to-frontage across        14 – 22 m
street lamp height                 4.5 – 5.5 m
bollard height                     0.85 – 1.05 m
street tree clear trunk            2.2 – 3.0 m below the canopy
```

### 3.4 Furniture

```text
counter / bar working height       1.00 – 1.10 m
café table top                     0.70 – 0.76 m
chair seat                         0.42 – 0.47 m
shelf pitch on a fitted wall       0.32 – 0.45 m
pendant light, lowest point        1.90 – 2.30 m above the floor
window bench seat                  0.42 – 0.47 m
A-board chalkboard                 0.90 – 1.15 m tall
planter                            0.45 – 0.85 m tall
```

## 4. The street scope — exactly this much

A **single straight or gently curved frontage**, 45–70 m of walkable length, terminated at both
ends so the player understands where the slice stops. Acceptable terminations: a corner turning
out of sight, a rise, a planted bank, a bridge, a closed gate, dense foliage. **A hard invisible
wall across an open view is not an acceptable termination.**

Required, and nothing beyond it:

| | Requirement |
| --- | --- |
| **the café** | one, primary, enterable. §5 and §6 |
| **the second enterable building** | exactly one shop on the café's side of the street, enterable on foot, with a real interior. §4.1 |
| **secondary façades** | three to five, forming the rest of the frontage; all except the second enterable building are non-enterable. Each must have its own massing, roofline, material and shopfront or window treatment; a repeated unit is a defect |
| **opposite side** | a frontage or a bounding landscape across the street. It may be non-enterable and lower detail, but the street must not read as a single-sided stage set |
| **ground** | pavement, kerb, carriageway, and the crossing between them, all at §3.3 dimensions |
| **street furniture** | at least: street lamps, bollards or a kerbside edge treatment, one bench, litter bin, A-boards, planters, and the café's own outdoor tables and chairs |
| **planting** | at least: street trees, planters or window boxes with real foliage, and at least one climbing or trailing plant on a building |
| **signage** | the café's own hanging sign and fascia, plus legible identification on at least two secondary façades |
| **backdrop** | a distant element closing the view where the frontage ends: a hillside, a further townscape, water, or a treeline |
| **people** | at least two other figures somewhere in the slice. They may be static or on a simple path; they exist so the street is not empty |

**Explicitly out of scope:** a third enterable building, a town square, a market, vehicles in
motion, weather, a day/night cycle, water simulation, crowds, and any building the player can see
but never reach.

### 4.1 The second enterable building — amended 2026-10-06

**Amendment.** Until 2026-10-06 this section listed *a second enterable building* as out of scope.
It is now **required**, exactly one, of every track.

**Why.** On walking the slice, the operator asked that shops be enterable. The operator's standing
rule is *prefer a smaller world with complete interiors over a larger fake town*. A street in which
one door of a dozen opens tests the café, not the claim that the world's buildings are places: the
player learns that a shop door is scenery, which is the "larger fake town" the rule rejects. One
more complete interior, on a frontage that does not grow, is the smallest change that makes
*buildings are places you enter* true of the street rather than of one exception in it. It costs no
street length, no new reference, and no new system, and it follows §1's trade of breadth for
completeness.

**Comparability is preserved, not traded.** `ARC-20` §2 requires both tracks to be judged against
the same scope. The amendment is made **here**, in the shared target, so it binds every track: a
resumed `VIS-3D-UE5-1` builds the same second interior to the same requirements. It is not one
track's addition.

Requirements:

```text
one existing secondary shopfront on the café's side becomes enterable; the frontage
   does not grow, and no building is added to make room for it
a door that reads as enterable from the pavement: a real leaf in a real opening,
   standing open or visibly glazed onto a lit room, distinguishable at the framing of
   §12 view 1 from the doors that do not open
the same structure rules as §6.1: four walls with thickness, a walked floor, a treated
   ceiling, glazing shared with the exterior, and one opening besides the front
a fit-out that identifies the shop's trade from inside, at the three object sizes of §6.2:
   room-scale fittings, furniture-scale pieces, and hand-scale stock
the navigability and collision rules of §6.3, with its own semantic Place where the
   world has one
the same threshold measurement as §7.1, at the same four points, reported separately
```

It is a **second** interior and is held to completeness, not to the café's density: the café stays
the primary subject of every reference comparison (§§2, 5–6).

## 5. The café exterior

Derived from `03_cafe_frontage.png` and `05_main_street_golden_hour.png`, and verified against
them as pixels.

Required elements:

```text
a named fascia with legible raised or painted lettering
a moulded cornice or projecting head above the fascia, with visible depth
painted timber shopfront joinery: pilasters or piers at each end, mullions, a transom
large glazing with the interior visible through it from the pavement
a stallriser below the glazing
a real door leaf in a real opening, distinguishable from the glazing
an entrance step and threshold
a projecting hanging sign on a bracket, or an awning, or both
at least one wall-mounted exterior light
masonry above the shopfront, with window openings that have reveals, sills and heads
at least one A-board, two planters, and outdoor seating for four
window boxes, a hanging basket, or trailing planting on the frontage
```

**Every opening has depth.** A window drawn as a flat panel flush with the wall is a defect: a
reveal, a sill and a head are what make masonry read as masonry rather than as a texture.

**The massing and silhouette of a referenced building is a hard-fail category**
([`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) §5). A single flat parapet where the reference steps,
or one continuous plane where the reference projects, fails without further discussion.

## 6. The café interior — the part that cannot be faked

This is the reason the slice exists. An interior is **not** satisfied by a popup, a static image,
a façade, a billboard, a projected image, or a room that can be seen and not entered
([`ACCEPTANCE.md`](ACCEPTANCE.md) §4.2).

### 6.1 Structure

```text
four real walls with thickness, and a real doorway opening through one of them
a floor the player walks on
a ceiling with a visible treatment, not a blank plane
glazing shared with the exterior: the same glass seen from both sides
at least one opening other than the front glazing — a rear window, a serving hatch,
   a lit doorway to a back room — so the room is not a sealed box
```

### 6.2 Fit-out

```text
a serving counter with a front, a top, and something on it
a fitted back wall: shelving, cabinetry or a dresser, carrying objects at several sizes
a menu board or equivalent legible wall graphic
loose seating: at least three tables with their chairs
one seating type that is not a loose table — a window bench, a banquette or a bar stool run
light fittings that are visible objects, not invisible sources
decoration: pictures, plants, textiles, or wall objects, on at least two surfaces
functional props: crockery, glassware, jars, bottles, food, paper, a till
a floor covering or floor material change over part of the floor
```

**Objects must exist at three sizes**: room-scale (counter, shelving), furniture-scale (tables,
chairs), and hand-scale (cups, jars, paper). A room with only the first two reads as a model of a
café rather than as a café, and this is the single most common way an interior fails.

### 6.3 Navigability and collision

```text
the player crosses the doorway on foot, without teleporting, fading, or a loading screen
the player walks a closed loop inside the room and returns to the door
clear walking width beside the counter and between tables ≥ 0.80 m
the player cannot pass through any wall, the counter, a table, or the closed part of the frontage
the player cannot fall through the floor, and cannot leave the room except through the doorway
entry and exit preserve position, spawn no duplicate player, keep world identity,
   change semantic location where a Place exists, and leave the camera working
```

## 7. Lighting and time of day

**The slice is set at a low warm sun**, matching `05_main_street_golden_hour.png`: a sun elevation
in the range 12°–25°, warm direct light, long soft shadows, a clear sky with some cloud, and cool
sky-coloured shade. This is a fixed condition, not a cycle.

The interior is lit by its own fittings and is warmer than the street. Both must work at the same
time, because the player sees both at once through the glazing, and because the whole point of the
threshold is that it is crossed.

### 7.1 The threshold, measured and not felt

These are objective and are verified by measurement, at four sample points on one straight path —
outside on the pavement, in the doorway, two metres inside, and deep inside the room:

| | Requirement |
| --- | --- |
| **no catastrophic exposure jump** | the perceived brightness of the frame changes progressively across the four points; no single step changes mean frame luminance by more than a factor of three |
| **no black interior** | at every interior sample the room's own surfaces are legible: mean luminance of the frame above a stated floor, and the darkest furnishing distinguishable from the floor |
| **no blown-out exterior** | from inside, looking out through the glazing, the street is not a white field: the exterior must retain visible structure, with a stated cap on the fraction of clipped pixels |
| **legible materials indoors** | wood, metal, ceramic, fabric and glass are distinguishable from one another indoors, not flattened to one tone |
| **sane shadows** | the interior has direction: furniture is grounded by contact shadowing, and the room is not uniformly flat-lit |
| **sane reflections** | glass, polished counter tops and metal reflect the room they are in, not a different room, a black void, or the sky through the ceiling |

The numeric thresholds and the measurement method are the implementing track's own, and are
recorded with the evidence. **Whether the transition is beautiful is the operator's judgement and
never the agent's** ([`ACCEPTANCE.md`](ACCEPTANCE.md) §2).

## 8. Cameras

Three modes, one movement controller, and a mode switch must preserve position, velocity,
orientation and collision state:

```text
1  first person       eye at §3.1 height
2  third person rear  behind and above; the framing of 01_lakeside_promenade.png
3  third person front looking back at the character
```

All three must work **indoors as well as outdoors**, and the third-person boom must not put the
camera through a wall in a room of the size in §3.2.

## 9. Runtime integration

The slice is a **MineWorld presentation**, not an art scene. It therefore uses:

```text
the same authoritative world, served by the same server
the same EntityId identities, carried as strings and never as numbers
the same ActionIntent submission path, with the server deciding every affordance
the same movement authority
the same semantic Place identity, so entering the café changes where the player is
   in the world and not only where the player is on the ground
```

A client evaluating an interaction rule itself is a defect
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§4, 7–9;
[`clients/protocol/ADOPTION.md`](../clients/protocol/ADOPTION.md) §3.3).

Production may run ahead of integration, and a track may reach a technically complete scene before
it is connected. A slice that is never connected is not this milestone.

## 10. The character slot

The slice carries a **stable presentation slot** for the player character, so that the reference
character can be dropped in later without restructuring the scene:

```text
one named attachment point the scene owns
the environment never reads the character's mesh, skeleton, materials or animations
the placeholder carries no environment logic, and receives no subjective polish
replacing the placeholder is a change to the slot's contents, never to the scene
```

`VIS-3D-GODOT-1` failed on categorical identity mismatch (`ARC-17`), and the character is being
rebuilt on its own track. The environment must not wait for it, and must not be rebuilt around it
when it lands.

## 11. Assets and licensing

Every external file entering the repository passes `DEP-8`, **including its 2026-09-27 amendment**:

> ask whether we may **relicense** it, not merely whether we may redistribute it.

A permission to distribute is not a permission to place under MIT. An asset that is useful for
experimentation but cannot enter the repository is kept explicitly outside it and recorded as
such; it is never quietly committed.

Sourcing stays narrow on purpose (`DEP-8`, coherence procedure): two libraries used consistently
read as one town, eight individually better ones do not.

## 12. Required review views

A slice is reviewable when these exist as **real runtime frames**, captured from the running
client at its normal resolution and in its normal lighting, at the framing named:

```text
1  street wide                     the frontage read as a street
2  approach to the café            walking toward it, café dominant
3  café exterior                   the framing of 03_cafe_frontage.png
4  the doorway transition          standing in the opening, both sides visible
5  interior wide                   the room read as a room
6  interior with the character     the character standing inside it
7  third person rear               the framing of 01_lakeside_promenade.png
8  third person front              the character facing the camera
9  character in environment, close enough to judge the character against its reference
10 the second enterable building   its door from the pavement, and its interior wide (§4.1)
```

Plus, in the review package (`ARC-20` §3): the milestone id, **the exact launch command**, the
canonical references, known limitations, and the specific subjective questions being asked. A
short video walkthrough is optional and useful.

**The review package is not an engineering log.** The operator must be able to launch, look, walk
and judge quickly.

## 13. Acceptance

| State | Who | When |
| --- | --- | --- |
| `TECHNICALLY READY` | the agent, autonomously | every objective gate in §§3, 4, 6.3, 7.1, 9, 10, 11 holds with recorded evidence, and [`ACCEPTANCE.md`](ACCEPTANCE.md) §4 passes |
| `READY FOR HUMAN VISUAL REVIEW` | the agent, autonomously | additionally, the character has landed and §12 is captured |
| `ACCEPTED` | **the operator, and only the operator** | `ARC-11` |

On reaching review readiness, subjective polishing on that branch **stops** (`ARC-11`, `ARC-20`
§4), the runnable candidate and its launch command are preserved, the milestone is recorded in
[`HUMAN_REVIEW_QUEUE.md`](HUMAN_REVIEW_QUEUE.md), and the agent moves to independent work.

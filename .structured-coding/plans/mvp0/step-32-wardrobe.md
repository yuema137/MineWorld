# Step 32 — Wardrobe: changeable hair and clothing, starting outfits, buying clothes

**Role:** step document for **S32 — wardrobe**. This is a new step; the primary session adds it to
[`overall.md`](overall.md). The document contains:
- the operator requirement and the release ruling;
- an audit of the simulation and the cast route;
- a reuse comparison of the character and garment frameworks, with every fact read at its source;
- the prototype evidence and the recommended art route;
- the simulation architecture: the `wardrobe` System Pack, the inventory claim, actions, disclosure,
  hairstyle, starting wardrobes, persistence;
- the presentation binding in both clients, the gates, and the performance and size budgets;
- the impact on S26, the PR split with implementers, acceptance and adversarial criteria, risks,
  records, and the operator questions.

**Effort:** `mvp0`. The operator ruled on 2026-10-15 that the wardrobe is **release-blocking** (§0.1), and
S32 decides the route for S26's cast, which is MVP-0 and stopped before its Meshy phase 2 (§15). ·
**Parent:** [`overall.md`](overall.md).

**Lifecycle:** `STEP DESIGN (DRAFT) — awaiting primary-session and operator review`. Nothing here
authorizes implementation, a Meshy call, an OpenAI call or a paid asset. Each PR is detailed to the
commit and frozen in turn (`CLAUDE.md` §3.1).

**Author:** design session, 2026-10-15, worktree `/Users/yuema137/mineworld-worktrees/design-s32`, branch
`docs/s32-wardrobe`. Audit base: `origin/main @ 4457ad47`, rebased onto `origin/main @ dd357ca6`. That base includes
three inputs that were unmerged when the audit began:
- S30, approved and merged (#223);
- S31, approved and merged (#222);
- the S26 sources amendment, approved (#221; ids ARC-130…134 and DEP-70…71).

**Ids:** `ARC-135…ARC-142` and `DEP-72…DEP-76`, the range reserved for this step (§19). Local ids:
- `SD-S32-n`: step decisions;
- `A-n`: audit rows;
- `W-n`: wardrobe gates;
- `AC-S32-n`: step acceptance;
- `X-S32-n`: planted mutations;
- `Q-S32-n`: operator questions;
- `R-S32-n`: risks;
- `F-S32-n`: findings;
- `P-n`: prototype records.

**Parallel-safety:** this file and a short dated pointer in `step-26-cast.md` are the only files this
session writes. Edits elsewhere are **proposed** in §19 and made by the PRs that own them.

---

## 0. Binding inputs

### 0.1 Operator requirement and ruling, 2026-10-15 (verbatim)

> 我觉得人物应该也要有换装功能（发型，衣服，裤子，鞋，袜子），你看看有没有比较好的开源框架能直接实现的。我们给每个人物几件初始服装，然后人物也可以买服装

*Gloss:*
- characters need outfit changing: hairstyle, clothes, trousers, shoes and socks;
- find a good open-source framework that implements it directly;
- give every person a few starting garments;
- people can also buy clothing.

**Ruling relayed by the coordinator, 2026-10-15 (binding).**
- Everything the operator asked for must be done before the public release, and the wardrobe is
  release-blocking.
- A problem that looks unsolvable is researched for solutions; scope is not dropped.
- Anything that costs money is an operator question (§20).

### 0.2 Standing rules that bind this step

| Rule | Where it bites here |
| --- | --- |
| Framework, not a game (`CLAUDE.md` §1) | Slots, layering, garment kinds, privacy policy and the art route are data and modules. Nothing in the kernel, no garment in code |
| Reuse before building (`DEP` both ways, `REUSE_POLICY.md`) | §4: every option compared, and adoption and rejection both recorded (`DEP-72…76`) |
| Every NPC at the protagonist's bar: one style, one precision, one method (S26 §1, §7.4a) | §5 recommends the route; §7 makes the bar checkable for modular characters |
| Agents are first-class players (S27) | §12: an agent sees what others wear, from the same disclosure a human client gets |
| Single ownership, kernel ignorance (`CLAUDE.md` §4 rules 1–2) | §8: `wardrobe` owns what is worn; `inventory` owns what is held; `economy` owns money; `attributes` owns hair |
| Semantic state is not render geometry (rule 14) | the world says "an oatmeal oxford shirt, size M, worn as the top"; a pack says which mesh and which sprite layer |
| All platforms | the tools (Blender, MPFB2, Godot) run on macOS, Linux and Windows; the runtime is Godot and Rust only (§4.3) |
| Realistic defaults | sizes from a cited body-dimension standard (§8.6); changing durations stated as authored defaults (§8.4) |

---

## 1. The hard technical fact

The S26 route D+ makes each person from **one Meshy image-to-3D generation fused into one mesh**:
- body, skin, hair, every garment and every accessory are one surface;
- one material, one 2048² texture, about 95,000 triangles;
- fitted to our 52-bone `mixamorig` skeleton (S26 §5, `CHARACTER_ROUTE_D_PLUS.md`).

There is no shirt in that mesh to take off. The "shirt" is the torso's surface, and there is no body
underneath it. **A fused character cannot change garments.** A texture swap can recolour a shirt whose
silhouette stays the same, but it cannot turn a hoodie into a tank top, a skirt into trousers, or a bun
into a bob.

A wardrobe needs five things the fused route lacks:
1. **A body**, rigged on the 52-bone skeleton.
2. **Garments and hair as separate skinned meshes** on the same skeleton.
3. **Body hiding under garments**, so skin never pokes through cloth.
4. **Fitting**, so that one garment fits every body shape the cast has (age, build, stature — S26 §7.4a,
   S30's appearance attributes).
5. **Material variants**, so that one garment model comes in several colours without another mesh.

Industry and open source solve these with one method, under different names:
- MakeHuman/MPFB2's clothes and delete groups;
- UMA's slots, overlays and DNA;
- VRoid's layered clothing;
- the "modular character" kits.

The method:
- **one shared body topology**, whose shape varies by morphs;
- **garments authored once against that topology**, bound to it so they follow every morph;
- **per-garment masks** of the body vertices they cover;
- **skin weights inherited from the body**.

§4 compares the concrete options. §5 recommends one.

---

## 2. Audit (`origin/main @ 4457ad47`, and the drafts named in the header)

| # | Fact | Evidence |
| --- | --- | --- |
| A-1 | **Item kinds are entities with an `item:` section.** `item: { category: drink }`, unknown keys refused. `ItemKind` is owned by `item`, and other packs ask `is_declared`. A kind's file is `worlds/<w>/items/<key>.yaml` | `systems/item/src/section.rs:12-55`; `component.rs:14-38`; `worlds/market-town/items/*.yaml` |
| A-2 | **Holdings are counts per kind, not item instances.** `Holdings::count(item)`; `transfer(from, to, item, count)` is the checked constructor other packs use (`ARC-26`) | `systems/inventory/src/admit.rs:54-112` |
| A-3 | **A person carries at most `PERSON_CAPACITY = 6` items of every kind together**, a published constant, "not world configuration" (`ARC-37` item 5, step-10 F-39). Organizations are unbounded | `admit.rs:11-18,30-48` |
| A-4 | `give { item, count }` is owned by `item-transfer`, with spatial requirement `same_place().within(3 000 mm).requiring_target_available()` | `systems/item-transfer/src/action.rs:11-56` |
| A-5 | **Buying.** S31 replaces `buy { item }` with `buy { item, price }` at the seller's asking price, with atomic settlement `[sold, money-transferred, items-transferred]` in one dispatch. Named refusals: `price-changed`, `insufficient-funds`, `out-of-stock`, `cannot-carry`. Haggling is release-blocking in MVP-0 | `step-31-wealth-prices.md` §§5.2–5.9  |
| A-6 | **Shops are Organizations** with `holdings` and `economy.shop { at, prices }` | `worlds/market-town/organizations/*.yaml` |
| A-7 | **The 3D slice's world, `social-cafe`, has no `item`, `inventory`, `item-transfer` or `economy`.** `market-town` has them, with the same twelve people | `worlds/social-cafe/world.yaml`; `worlds/market-town/world.yaml` |
| A-8 | **Named refusals exist.** `Rejection::System { code, detail }` carries a system's own code beside the kernel's `Busy`, `TooFarAway`, `PermissionDenied`, `NoSupportedInteraction`, `TargetUnavailable`, `Unavailable` and `PreconditionFailed` | `contracts/src/action.rs:215-244` |
| A-9 | **Event audiences:** `Public`, `Place(id)`, `Participants`, `Entities`, `SystemInternal` | `contracts/src/event.rs:296-310` |
| A-10 | **Processes** are stored records, owned and ended by their system; others request an interrupt | `kernel/src/process.rs:1-40` |
| A-11 | **`appearance` (S26-a, `ARC-91`):** an opaque public `look` key per person; "a later clothing … System changes a look by emitting an event the `appearance` pack owns" | `step-26-cast.md` §4.2 |
| A-12 | **S30 `attributes`** (approved; Q-S30-1 ruled):<br>• appearance descriptors are `perceivable` attributes owned by `attributes`;<br>• `look` stays the only renderer key (`ARC-125` amends `ARC-91` item 3);<br>• an attribute may be owned by another pack through `AttributeOwner` (`persona` is the precedent);<br>• an `evolving` attribute changes only by `attribute-change-proposed` from a declared proposer, answered with `trait-changed` (audience: the holder plus observers present, if perceivable);<br>• the v2 Appearance group lists "hair (5)" and "dress (clothing style, colour preference, headwear, footwear, accessories, glasses)" | `step-30-person-attributes.md` §§4.4, 4.5, 5.x, 11.1 and the table at line ≈ 670  |
| A-13 | **S30 renders perceived people** as phrases ("a tall man in his sixties with grey hair and a flat cap"), at most 160 B per person in `PersonLine.description`, shared by cognition and MCP | `step-30-person-attributes.md` §9.2 |
| A-14 | **The S26 cast as built:**<br>• the fused route D+;<br>• gates G-1…G-14 (G-1 ≤ 100 k triangles in **one** skinned mesh; G-2 **exactly one** material);<br>• the pipeline `tools/character_pipeline/` (PR #211, branch `mvp0/pr-s26b1`);<br>• the cast director (`mvp0/s26-cast-director`);<br>• the pose-sheet tool (`mvp0/s26-pose-sheet`).<br>21 people are getting reference images; Meshy phase 2 is on hold | `step-26-cast.md` §§5–6, 12; worktrees listed by `git worktree list` |
| A-15 | **The S26 sources amendment ** already plans `mpfb`, `charmorph` and `import-vrm` character sources at `rigged` entry, with **bone-map schemes as data** (`mpfb-game-engine` among them), and keeps the 52-joint profile "never relaxed for a source" | `docs/s26-character-sources`, step-26 §§5.6.2–5.6.5 |
| A-16 | **Release size:** the fused cast's per-person texture puts S-1 at ≈ 330–345 MB of `3d.pck` against a 250 MB hard limit. The ruling is UASTC, else about 350 MB | `step-26-cast.md` §10.3, Q-S26-5 |
| A-17 | **The 2D client** draws one sprite set per person (`_front`, `_front_b`, `_back`, `_back_b`, ≈ 110 px tall), bound by `person:<n>` / `look:<key>` roles in `asset_bindings.yaml`. It has no layering | `clients/2d/scripts/scene/people.gd:175-232`; `presentation/mineworld-default/2D/assets/asset_bindings.yaml` |
| A-18 | **Prior route assessment (2026-10-01):**<br>• MPFB2's core assets are CC0, its code is GPL; some packs are CC-BY and fail `DEP-8`;<br>• its clothes and hair are "older, lower-detail";<br>• a parametric face did not reach the reference (§3 there);<br>• VRoid fails `DEP-8` (pixiv base models are not CC0) and fails on style (anime head, MToon).<br>Route E (wrapping a parametric head onto generated shapes) **FAILED**; the Meshy whole character passed | `docs/references/CHARACTER_ROUTE_ASSESSMENT.md` §§3, 4.4, 4.5; `CHARACTER_ROUTE_E_EXPERIMENT.md` §9; `CHARACTER_ROUTE_D_PLUS.md` header |

**Findings.**
- **F-S32-1 — Capacity.** `PERSON_CAPACITY = 6` (A-3) cannot hold a worn outfit of five to seven garments
  plus anything else. Worn clothes are not carried items, so the capacity must stop counting them, without
  `inventory` learning what "worn" means (§8.3).
- **F-S32-2 — No economy in the 3D world.** `social-cafe` has no economy (A-7), so "buy a shirt, wear it,
  see it in 3D" needs the 3D world composed with `item`, `inventory`, `item-transfer`, `economy` and
  `wardrobe` (§16, S32-f).
- **F-S32-3 — G-1 and G-2 assume a fused mesh.** A modular character has several meshes and several
  materials by construction, so the gates are restated per assembled outfit (§7).
- **F-S32-4 — Dress overlap with S30.** S30's v2 "dress" attributes (headwear, footwear, glasses) overlap
  what a wardrobe makes worn state. §9.2 resolves the overlap: worn state is the wardrobe's, and S30 keeps
  preferences.

---

## 3. Goals and non-goals

**Goals.**
- **G-a.** Every person can **wear**, **take off** and **change** garments in seven slot families — hair
  covering, top, outer, legs, feet, socks, accessories — with layering rules. They can **buy** garments
  at S31's prices and **give** them.
- **G-b.** A person's **hairstyle** can change, through a service action (§9).
- **G-c.** Every person starts with a **starting wardrobe** derived from their S26 casting brief (§10).
- **G-d.** What a person wears is **perceivable**, and observers see it the same way:
  - the 3D client, as the actual garments;
  - the 2D client, as layered sprites;
  - an agent through MCP, as words;
  - an LM controller, as words.
- **G-e.** Both clients draw any outfit a world can produce, at the protagonist's quality bar, inside the
  frame and size budgets.
- **G-f.** It is a framework capability:
  - a world without `wardrobe` is unchanged;
  - a world may declare its own slots, garment kinds, privacy policy and size chart;
  - a Presentation Pack may bring its own garments.

**Non-goals.**
- Cloth simulation of garments (§4.6: skinned garments, plus optional spring bones for hair later).
- Wear and tear, laundry, washing or dirt.
- Tailoring and garment crafting.
- Weather-driven automatic dressing. A controller may choose to dress for the weather, but no System
  forces it.
- Facial hair as a separate system. It is an S30 attribute; a shave action is a later step.
- Runtime-loadable 3D Presentation Packs (S26 F-S26-3 still stands; §11.1).

---

## 4. Reuse comparison (facts read at source; URLs and access dates in §4.7)

Every row was read at its source on **2026-10-15** (URLs in §4.7). "Secondary" marks a fact taken from
a secondary source because the primary page was unreachable. The `DEP-8` test, "relicensable under MIT
in a public repository", governs every asset row.

### 4.1 Character and wardrobe frameworks

| Option | Licence (quoted where it decides) | Platforms | Quality vs Vera | Body coverage | 52-bone fit | Effort | Risk | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **MPFB2 / MakeHuman** (v2.0.18, released 2026-10-11, "Blender 4.2 LTS and newer"; ran headless on 5.2.2, P-1) | Code GPL-3.0-or-later (a **tool**, never distributed). "All core assets are shared under Creative Commons, CC0"; exported models "end up being licensed CC0". 70 asset packs: the CC-BY ones (Hair 02/03, Shirts 02/03, Pants 02/03, Shoes 02/03, Dress 02/03, Skirts 02, Suits 03, Underwear 02/03, Hats 03/04 …) **fail `DEP-8`**; the system assets, Hair 01, Shirts 01, Pants 01, Shoes 01, Skirts 01, Underwear 04, Dress 01 and Suits 01 are CC0 | macOS, Linux, Windows (pure-Python Blender extension; verified on macOS) | **below**: generic faces, ≈ 2.5–4 k-tri garments with simple baked folds, ≈ 2 k-tri hair shells, a dated and small CC0 selection (P-3) | **best of any option**: age ≈ 1–90 y, sex, weight, muscle, height, proportions, three ancestry blends; garments follow every morph by construction (P-3) | **identical**: the built-in `mixamo` rig has our 52 names (P-2) | medium | stock look below the bar; slider stature needs calibration (P-2) | **adopt the method, data model, `hm08` template, rig and MakeClothes (`DEP-72`); not the stock look** |
| **CharMorph** (the old town body's tool) | Code GPL-3; Vitruvian `license: CC0`; MB-Lab bodies AGPL, "the AGPL will propagate … to the output models" (excluded) | Blender only; "unusable" on Blender 5.0.1, hair broken on 5.2 (issue #59) | below (A-3's crude crowd) | morphs, but Vitruvian ships only `Pants` and `Shirt`; particle hair, no game meshes | yes (it is our rig's source) | high: no asset ecosystem | "doesn't really have a maintainer currently" (2026-09-18) | **rejected as the wardrobe base (`DEP-74`)**; it stays only as the 52-joint rig's historical source |
| **VRoid Studio / VRM** | pixiv content "is not CC0"; and **"You cannot create an application that can generate or output 3D models, avatars, or items consisting of deformed or combined meshes and textures that were created in VRoid Studio"** without a separate pixiv licence. A wardrobe that combines garments on bodies at runtime is that application. VRM spec repository: no licence file found | Studio: Windows and macOS; importer: all three | anime head and MToon: a style mismatch (A-18) | sliders, anime proportions | VRM 1.0 has 55 humanoid bones; mappable | medium | **licence blocks the use case** | **rejected (`DEP-74`)** |
| **godot-vrm** (V-Sekai; fork AzPepoze 2.6.1) | MIT (sample models separately licensed) | all three | — (an importer) | — | — | — | v2.1.0 on the Asset Library targets Godot 4.1, not tested on 4.7 | **not needed**: VRM brings no wardrobe layer; the S26 sources amendment already reads VRM through glTF |
| **Godot native**: one `Skeleton3D`, one skinned `MeshInstance3D` per part, binds resolved **by bone name** | Engine MIT. `skeleton_3d.cpp` matches named binds to bones; each `Skin` gets its own bone-matrix buffer | all three | — (the mechanism) | — | yes, if every part has the same names and rests | low | rest/bind mismatch if a part is imported with different Rest Fixer settings | **adopted (`DEP-75`)**: P-6 swapped a garment at runtime |
| `RetargetModifier3D` (4.4), `SpringBoneSimulator3D` (4.5) | Engine MIT | all three | — | — | — | — | each child skeleton costs a skeleton | **not needed** for garments (same skeleton); `SpringBoneSimulator3D` is the later route for loose hair and hems (F-S32-7) |
| Godot modular-character addons | `godot-character-creator` (MIT, Godot 3.2, last push 2020); "Modular 3D Character System" (itch, paid beta); static-mesh mergers only | — | — | — | — | — | dead or paid or wrong kind | **none adopted**; merge-at-change is ours (§14.1) with `ImporterMesh.merge_importer_meshes` evaluated (it keeps weights, drops blend shapes and LODs) |
| **Unity UMA** (reference design only) | MIT; v3.1f3 (2026-10-09) | Unity only | — | DNA | — | design only | none | **concepts borrowed**: recipe = outfit as data (§8.2); slots with exclusivity and hide rules (§§8.2, 6.3); DNA = shape parameters garments inherit (§6.2); bake to one mesh + atlas for crowds (§14.1) |
| **Ready Player Me** | Acquired by Netflix; services shut down **2026-01-31**; the domain does not resolve; MIT SDK leftovers, `animation-library` "not intended to be used with other characters" | — | — | — | — | — | dead | **rejected: gone** |
| **Quaternius** Universal Base Characters + Modular Outfits | itch pages say CC0, but **quaternius.com now shows the "Quaternius Asset License v1.0" (8/28/2026)**: "You may not … redistribute the Assets … as a standalone asset, asset pack … whether for free or for payment", not retroactive | all three | stylized low-mid poly (≈ 13 k tris), a different style | 3 builds × 2 sexes incl. teen | no: ≈ 65 UE5-style joints (secondary), needs a bone map | low | licence ambiguity in a public repo; style mismatch | **rejected for the default pack (`DEP-74`)**; a user pack may use it under its own terms |
| **Kenney** Animated Characters | CC0 | all three | blocky low poly | minimal | no | low | style | **rejected** (style) |
| **Synty** | EULA: "You must not share the source files of any Assets outside your team"; no upload to generative services | — | low poly | some | no | — | blocked | **rejected** (licence) |
| **Character Creator** (Reallusion) | exported content "must not be available for public download" (secondary) | Windows | high | high | yes | — | blocked | **rejected** (licence, as in 2026-10-01) |
| **Blender Studio Human Base Meshes** v1.4.1 | CC0 | all three | realistic or stylized base, **unrigged, no garments** | 17 meshes | needs rigging | medium | no fitting ecosystem | **not adopted**: `hm08` already brings the morphs, rig and garment binding; kept as a sculpt base (as `DEP-8` already allows) |

### 4.2 Garment generation and fitting

| Option | Licence / terms | Mac without CUDA | What it gives | Verdict |
| --- | --- | --- | --- | --- |
| **Meshy** image-to-3D of a garment as a separate object | paid plan: customers "own their Customer Output" (free plan: CC BY 4.0, owned by Meshy) | cloud | matches Vera's generator and style; one fused object per run | **adopted for garments under a credit cap (`DEP-73`, Q-S32-1)**: one garment per run on the template mannequin, mannequin removed by inside-test (§6.6 g3) |
| Meshy Auto Split | "Textured models aren't supported for splitting yet"; 10 credits per split; aimed at 3D printing | cloud | geometric parts, untextured | **rejected**: not a garment-layer separator |
| Meshy Rigging API | "only works well with standard humanoid (bipedal) assets"; "no cloth physics"; bone names undocumented | cloud | its own skeleton | **not used** (S26 `DEP-55` stands) |
| Tripo mesh segmentation | v2 "Beta"; texture preservation not stated; parts incompatible with texture output (secondary) | cloud | semantic parts | **not adopted**: no textured layered garments |
| Hunyuan3D-Part (P3-SAM, X-Part) | "You must not use … Output or results … outside the Territory" (excludes EU, UK, South Korea) | CUDA | parts | **rejected**: the territory clause binds outputs |
| PartCrafter, SAMPart3D, HoloPart | MIT | **CUDA only** | research part generation | **not adopted**: no Mac path; revisit on a CUDA machine (as S26 UniRig) |
| Garment3DGen | **CC BY-NC 4.0** | CUDA | garment from image | **rejected** (non-commercial) |
| GarmentCode / pygarment | MIT (its Warp simulator fork: "allows only non-commercial use", CUDA) | patterns yes, simulation no | parametric sewing patterns | **not adopted in S32**: patterns without our simulator do not give a game garment; noted for a future tailoring step |
| **Blender Shrinkwrap, Surface Deform, Data Transfer** | GPL tool, outputs ours (`DEP-52`) | yes | offline fit and weight transfer | **adopted** for g4–g5 and route B's registration (§6.1) |
| **Robust Skin Weights Transfer** (Abdrashitov et al.) | MIT, libigl Python, CPU | yes | weights body → garment / template with inpainting | **adopted for evaluation in S32-p** (shares S26 `DEP-53`'s evaluation) |
| MakeHuman **MakeClothes** (in MPFB2) | GPL tool; output `.mhclo` ours or CC0 | yes | the binding of §6.2 | **adopted** (`DEP-72`) |

### 4.3 Cloth, colour and 2D

| Topic | Fact | Decision |
| --- | --- | --- |
| Cloth simulation | `SoftBody3D` recommends Jolt; pinning to an animated skeleton is not documented. `SpringBoneSimulator3D` (4.5) is for "hair, cloth, and tails" | **skinned garments**; spring bones later for hair and hems (F-S32-7); no soft body |
| Recolouring | Godot `instance uniform`: "set on each GeometryInstance3D", "practical maximum limit of 16 instance uniforms per shader", "do not support textures or arrays" | tint mask + ≤ 3 instance colours per garment (§6.5) |
| Automatic instancing | Forward+ only; "skinning and morphing can be very expensive"; per-surface draws (secondary) | measure; merge at change if needed (§14.1) |
| 2D paper dolls | LPC generator: code GPL-3.0, art per piece CC0 / CC-BY-SA / CC-BY / OGA-BY / GPL, "you must at least credit all the authors"; no tool keeps separately *generated* layers aligned | **2D layers rendered from 3D** (§11.2); LPC's per-layer `CREDITS.csv` borrowed as the model for per-layer provenance |

### 4.4 The reuse question, both halves (`CLAUDE.md` §4 rule 16)

**Are we reinventing a wheel?** No. The method (MakeHuman's binding and delete groups, UMA's
recipe/slot/hide/bake), the template mesh, the rig, the authoring tool, the importer and the engine
mechanism are all reused. What is ours:
- the runtime garment format, converted from `.mhclo`;
- the fit-at-load routine (≈ 100 lines);
- the body mask rebuild;
- the gates.

None of these exists for Godot 4.7 (§4.1, addons row).

**Are we forcing a wheel?** We deliberately do not:
- ship MPFB2 code (GPL), so it stays a tool;
- adopt MPFB2's look (below the bar);
- adopt VRM's avatar model (one avatar per file; the pixiv clause);
- adopt UMA (Unity).

The world model is MineWorld's own. Sections, facts, claims and disclosure are ours, and no framework
owns state.

### 4.5 Records

| Record | Content |
| --- | --- |
| `DEP-72` | MPFB2/MakeHuman adopted as a **tool**: the CC0 `hm08` template, the `mixamo` rig, `.mhclo` semantics and MakeClothes. Code GPL, never distributed or linked; CC-BY packs refused per asset (`DEP-8`). Stock garments only as test fixtures |
| `DEP-73` | Garments generated at Vera's tier: Meshy as separate garment objects (paid-plan ownership), OpenAI references through `mineworld_gen`, Blender fit and RSWT; part-aware generators declined (licence or CUDA) |
| `DEP-74` | Rejected as the wardrobe base, with reasons: CharMorph, VRoid/VRM, Quaternius, Kenney, Synty, Character Creator, Ready Player Me, Hunyuan3D-Part, Garment3DGen |
| `DEP-75` | Godot-native modular runtime (shared skeleton, named binds, fit at load) adopted; no addon; `RetargetModifier3D` not needed; `SpringBoneSimulator3D` deferred |
| `DEP-76` | 2D: layers rendered from the 3D characters with a stylizing render, against generated paper dolls and LPC (Q-S32-2 decides; the losing option is recorded as rejected) |

### 4.6 Licence consequence: tool vs output (MPFB2)

- **Using MPFB2 on an operator's machine** is using a GPL program. GPL places no obligation on its output.
  The MakeHuman FAQ states that exported models "end up being licensed CC0" and may be used "in a closed
  source game without need for attribution".
- **Shipping its output**: the `hm08` template positions, our fitted bodies and garments converted from
  `.mhclo` are CC0-derived or our own, and are relicensable under MIT (`DEP-8` passes).
- **We never ship, vendor or import MPFB2 code** into any runtime or CI path. The pipeline invokes it
  through Blender as an installed extension, like `DEP-52`'s Blender.
- **Community assets** carry per-asset licences named in each `.mhclo` header. The pipeline records them
  in provenance (G-14), and the default pack admits CC0 only.

### 4.7 Sources (all accessed 2026-10-15)

- MPFB2:
  - <https://extensions.blender.org/add-ons/mpfb/>
  - <https://github.com/makehumancommunity/mpfb2>
  - <https://static.makehumancommunity.org/about/license.html>
  - <https://static.makehumancommunity.org/makehuman/faq/are_makehuman_files_free.html>
  - <https://static.makehumancommunity.org/makehuman/faq/can_i_sell_models_created_with_makehuman.html>
  - <https://static.makehumancommunity.org/mpfb/faq/can_i_sell_models.html>
  - <https://static.makehumancommunity.org/assets/assetpacks.html>
  - <https://static.makehumancommunity.org/assets/assetpacks/makehuman_system_assets.html>
  - <https://static.makehumancommunity.org/mpfb/docs/assets/asset_creation_intro.html>
  - <https://static.makehumancommunity.org/mpfb/docs/rigging_posing/mixamo.html>
- CharMorph:
  - <https://github.com/Upliner/CharMorph> (`license.txt`, releases)
  - <https://github.com/Upliner/CharMorph/issues/59>
  - <https://github.com/Upliner/CharMorph-Vitruvian>
- VRM:
  - <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/humanoid.md>
  - <https://vroid.com/en/studio/guidelines> (403 to the fetch tool; read with a browser user agent)
  - <https://github.com/V-Sekai/godot-vrm>
  - <https://godotengine.org/asset-library/asset?filter=vrm>
- Godot:
  - <https://docs.godotengine.org/en/stable/classes/class_meshinstance3d.html>
  - <https://docs.godotengine.org/en/stable/classes/class_skin.html>
  - `scene/3d/skeleton_3d.cpp` (master)
  - <https://docs.godotengine.org/en/stable/classes/class_skeletonprofilehumanoid.html>
  - <https://docs.godotengine.org/en/stable/tutorials/assets_pipeline/retargeting_3d_skeletons.html>
  - <https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/>
  - <https://docs.godotengine.org/en/stable/classes/class_retargetmodifier3d.html>
  - <https://docs.godotengine.org/en/stable/classes/class_springbonesimulator3d.html>
  - <https://docs.godotengine.org/en/stable/classes/class_importermesh.html>
  - <https://docs.godotengine.org/en/stable/tutorials/performance/optimizing_3d_performance.html>
  - <https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/shading_language.html>
  - <https://docs.godotengine.org/en/stable/classes/class_softbody3d.html>
  - <https://imp0str.itch.io/modular-3d-character-system>
- UMA:
  - <https://github.com/umasteeringgroup/UMA>
  - <https://github.com/umasteeringgroup/UMA/wiki>
- Ready Player Me: <http://www.pocketgamer.biz/netflix-acquires-avatar-creation-platform-ready-player-me/>
- Kits:
  - <https://quaternius.itch.io/universal-base-characters>
  - <https://quaternius.itch.io/modular-character-outfits-fantasy>
  - <https://quaternius.com/license.html>
  - <https://kenney.nl/assets/animated-characters-protagonists>
  - <https://syntystore.com/pages/end-user-licence-agreement>
  - <https://www.blender.org/download/demo-files/>
  - Character Creator (secondary): <https://forum.reallusion.com/PrintTopic335621.aspx>
- Meshy:
  - <https://www.meshy.ai/terms-of-use>
  - <https://www.meshy.ai/tutorials/auto-split-3d-model-into-parts>
  - <https://docs.meshy.ai/en/api/rigging>
  - <https://www.meshy.ai/tutorials/character-auto-rigging-workflow>
- Tripo: <https://developers.tripo3d.com/en/docs/mesh-segment.md>
- Part and garment generators:
  - <https://github.com/Tencent-Hunyuan/Hunyuan3D-Part>
  - <https://github.com/wgsxm/PartCrafter>
  - Garment3DGen (nsarafianos), LICENSE
  - <https://github.com/maria-korosteleva/GarmentCode>
- Fitting:
  - <https://github.com/rin-23/RobustSkinWeightsTransferCode>
  - <https://docs.blender.org/manual/en/latest/modeling/modifiers/deform/surface_deform.html>
- 2D: <https://github.com/LiberatedPixelCup/Universal-LPC-Spritesheet-Character-Generator>

**Not verified** (marked for the PR that relies on them):
- the VRM specification's own licence;
- godot-vrm on Godot 4.7;
- Meshy rig bone names;
- the GarmentCodeData licence (HTTP 500);
- the Avaturn terms (404);
- the per-surface draw-call behaviour (secondary): S32-c measures it with Godot's raster monitors.

---

## 5. Recommendation: the art route

### 5.1 The prototype (P-1…P-6, 2026-10-15; scratch `/Users/yuema137/mineworld-worktrees/s32-proto/`, never committed)

| # | What was run | Result |
| --- | --- | --- |
| P-1 | Installed MPFB2 **2.0.18** from extensions.blender.org into an isolated `BLENDER_USER_RESOURCES`. Command: `blender -b --command extension install-file …`, Blender **5.2.2**, through `with-slot.sh` | `STATUS Installed "mpfb"`; initialization finished; no error. The zip's SHA-256 matched the published URL |
| P-2 | Built two bodies headless (`HumanService.create_human`):<br>• **womanA**: gender 0, age 0.45 (≈ 23 y), weight 0.3, height 0.9;<br>• **manB**: gender 1, age 0.95 (≈ 84 y), weight 0.95, height 0.1.<br>Each with the built-in **`mixamo`** rig | **The rig's 52 bone names are identical to `vitruvian.glb`'s skin joints** (sorted lists compared). The 52-joint profile (`HUMANOID_PROFILE.md` §1, G-5) holds with no bone map work. manB measured **1.28 m**: the sliders need calibrating to real statures (R-S32-6) |
| P-3 | Dressed **both bodies in the same six CC0 assets**: hair `bob02`, `eyebrow001`, crude low socks, wool trousers, basic tucked T-shirt, `shoes02`. Used `add_mhclo_asset(material_type="GAMEENGINE")` | **The same `.mhclo` files fitted both shapes.** Trousers: 1.226 m vs 0.766 m tall; shirt: 0.762 m vs 0.55 m wide. Renders: `out/womanA_front.png`, `out/manB_front.png`. One small poke-through: skin at manB's right sleeve. **The quality reads clearly below Vera**: generic faces, simple baked folds, dated garments |
| P-4 | Delete groups | They **hide, not delete**: a `Delete.<asset>` vertex group plus an inverted MASK modifier. Only socks and shoes carried `delete_verts` (2,246 vertices masked). **The T-shirt and trousers hid nothing**, so the body stays under them (poke-through risk) |
| P-5 | Exported as GLB with separate meshes (`out/womanA.glb`, `out/manB.glb`). Measured per mesh | Body 22,228 tris (2048²); hair 2,248; eyebrows 192; socks 496; trousers 2,674; T-shirt 2,700; shoes 3,064. **≈ 33.6 k triangles, 7 meshes, 7 materials, 7 surfaces** per dressed person. Extra garments: fisherman sweater 4,164 tris; tiered skirt 12,960 |
| P-6 | Godot **4.7.2** headless (`godot/swap.gd`):<br>• import;<br>• find the Skeleton3D;<br>• hide the T-shirt;<br>• load the sweater GLB and reparent its `MeshInstance3D` under womanA's skeleton with `skeleton = NodePath("..")` | One Skeleton3D (52 bones), 7 skinned `MeshInstance3D` children (each a `Skin` with 52 binds, one surface). **The runtime swap printed SUCCESS.** Two pitfalls:<br>• the importer renames `mixamorig:Hips` → `mixamorig_Hips`, so `find_bone` with a colon returns −1;<br>• reparenting needs `owner` cleared first |

**What the prototype settles:**
- The **method** works end to end on our tools and our skeleton:
  - shared topology;
  - `.mhclo` binding (each garment vertex = three base-mesh vertices, barycentric weights and a scaled
    offset);
  - delete groups;
  - one skeleton, several skinned meshes;
  - a runtime swap in Godot.
- The **body-shape coverage** is real: about 1–90 years of age, sex, weight, muscle, height, proportions.

**What it does not settle:**
- **The art tier.** MPFB2's faces and its CC0 garments read below Vera, as the 2026-10-01 assessment
  predicted (A-18). The method is adoptable; the stock assets are not the cast.

Two design facts follow:
1. **A garment exported from Blender is baked to one body's shape.** Swapping garments across differently
   shaped people therefore needs either one bake per person × garment, or **fitting at load** from the
   `.mhclo` binding. §6.2 chooses fitting at load.
2. **An exported delete group is a fixed hole.** Runtime swapping needs a **runtime body mask** (§6.3).

### 5.2 The recommendation (`DEP-72`, `ARC-138`)

**Adopt MakeHuman/MPFB2's method and data model as the wardrobe substrate. Do not adopt its stock look.
Pilot the body on two routes and let the gates choose.**

| Layer | Adopted | Source |
| --- | --- | --- |
| Skeleton | our 52 `mixamorig` joints, unchanged; MPFB2's `mixamo` rig produces them on any MPFB body | P-2 |
| Shared topology for fitting | MakeHuman `hm08` base mesh, **CC0** | §4.1 |
| Garment binding | `.mhclo` semantics, converted by the pipeline into our own runtime format (§6.2): vertex triples, barycentric weights, scaled offsets, a delete set, a material | §4.1, P-3 |
| Garment authoring | MPFB2's **MakeClothes**, scripted headless | §4.1 |
| Shape coverage | MPFB2 macro and target morphs, driven from the person's S30 attributes (age, sex, stature, build, mass) through calibrated maps | P-2, R-S32-6 |
| Body hiding | per-garment delete sets → runtime body region masks (§6.3) | P-4 |
| Variants | one garment model, several material variants (colour by tint mask, §6.5) | §4.1 |
| Runtime | Godot: one `Skeleton3D`, one skinned `MeshInstance3D` per worn part, garments fitted to the body at load (§6.2) | P-6 |
| **Garment and hair content** | **new, at Vera's tier**, made by the garment pipeline (§6.6). The stock CC0 garments serve tests and fixtures only, never the cast | A-18, P-3 |
| **The body and face** | **pilot two routes** (below); one is chosen for every person (§7.4a's one method) | |

**The two body routes in the pilot (S32-p).**

- **Route B (hybrid), preferred.**
  - Each person's body **is a route D+ Meshy generation**, from a reference image in a neutral close
    base layer: fitted tank and briefs, barefoot, hair under a close skin cap. The face, skin and body
    shape keep the generator that reached Vera's tier.
  - The MPFB2 `hm08` template is **registered** onto that body: its `mixamo` rig is posed to the
    person's fitted 52 joints, then refined by a non-rigid surface fit (§6.1). Every garment's `.mhclo`
    binding then fits the person through the registered template.
  - Hair is a library asset on the scalp.
  - **Why preferred:** it keeps the only generator that has passed the protagonist's bar (A-18: route E
    and the parametric faces failed it). The template is invisible, so a registration error shows as a
    garment gap or a poke-through, which W-gates catch (§7), not as a face that is wrong.
- **Route A (parametric), the control.**
  - Each person's body is an MPFB2 body shaped from their S30 attributes.
  - The skin is a generated 2048² albedo on the shared `hm08` UV, conditioned on the approved reference.
  - The face is shaped by MPFB2's face targets toward the reference.
  - **Why piloted at all:** it is cheaper (no Meshy credit per person), exactly parametric (any stature,
    build or age by sliders) and has no registration step. If it passes the scorecard beside Vera, it
    wins on cost and simplicity. The record says it probably will not (A-18), so it is the control arm,
    not the plan.

**The pilot's subjects:** Vera, plus two of S26's pilot people chosen to stress the method: an older,
heavier person (Ivan or Bob) and a slight young person in a skirt (Hana). Each is dressed in three outfits
from the new garment library, and the operator compares both routes beside the fused Vera
(`VIS-3D-WARDROBE-P`).

**Decision rule, fixed now (`ARC-23` rule 2: bounds from the record, never from a candidate):**
- A route passes if every subject passes W-1…W-12 (§7) and the scorecard (S26 §6.2) **beside the fused
  Vera**, in every outfit.
- If both pass, **A wins** (cheaper, one fewer stage).
- If only B passes, B.
- If neither passes, the failure is diagnosed and researched (operator ruling §0.1: no scope drop). The
  levers are listed in R-S32-1, and the operator is told before any further spend.

### 5.3 What the fused route D+ remains for (`SD-S32-9`)

| Use | Fused? | Why |
| --- | --- | --- |
| A Person in the release | **No**, after S32 | a Person must be able to change clothes. A fused Person would make the wardrobe a lie in 3D |
| Vera until her modular character passes parity | **yes, interim** | the accepted protagonist stays the visual reference and the control of every review sheet (S26 §6.3). In the interim the 3D client marks her outfit as `baked` (§11.1): the world's outfit changes are not drawn on her, and that is a release blocker, listed on the release checklist |
| The route's stages 3–12 | **reused** | route B's body is a D+ generation: mask, Meshy, game-ready, 52-joint fit, bind, fleck repair, export, tuning, gates (S26 §5.2). The fused route's *tools* survive almost whole; only its "everything in one mesh" output is retired |
| Decorative non-Person figures | **no** | S26 §8 ruled E-2: the extras are people from the other world, so they wear their own wardrobes like everyone else. A fused decor class would be the "cruder second class" S26 §1 forbids |
| A third-party pack's look | **allowed** | a Presentation Pack may bind a `look` to a fused character with `baked: true` (§11.1). Such a character ignores the outfit visibly, and the pack says so. Framework, not a fixed cast |

---

## 6. Asset architecture (3D)

### 6.1 The body (`ARC-138`)

A **modular character** consists of:
- **a body**: one skinned mesh on the 52 joints, with a per-vertex body-region channel (§6.3);
- **a fitting template**: the `hm08` vertex positions in this body's rest pose. These are 19,158 × 3
  floats ≈ 230 KB, stored with the body and never drawn;
- **a hair binding** for the scalp: the template region the hair assets sit on.

How each route produces it:
- **Route B.** The pipeline:
  1. builds an MPFB2 body whose macros approximate the person's attributes;
  2. poses its `mixamo` rig onto the person's fitted 52 joints (S26 stage 7);
  3. applies linear-blend skinning, so the template's limbs match the person's lengths;
  4. runs a non-rigid surface fit to the Meshy body: closest point with normal agreement, regularized by
     a Laplacian/ARAP term, with the face, hands and feet weighted to follow the landmarks.

  Step 4 uses Blender Shrinkwrap/Surface Deform with a smoothing pass, and Robust Skin Weights Transfer for weights (§4.2). The result is the fitting template.
- **Route A.** The MPFB2 body *is* the template, so the fitting template is its own vertices.

**Body regions.** The body regions of §8.2 (`scalp, face, neck, torso, upper-arms, forearms, hands, hips,
thighs, shins, feet`) are a fixed partition of the `hm08` template, authored once and stored as vertex
groups. In route B each body vertex takes the region of its closest template triangle. In route A it
is exact.

### 6.2 Garments: one asset, fitted to every body at load (`ARC-140`)

**The runtime garment format** (`garment.glb` + `garment.yaml`) is our own, converted from `.mhclo` by the
pipeline. Reuse adapted behind our interface: we never ship MPFB2 code, only data derived from CC0 or our
own assets.

```yaml
# presentation/mineworld-default/3D/garments/oxford-shirt/garment.yaml
format: 1
id: oxford-shirt
category: shirt-collared
slots: [top]
mesh: res://assets/garments/oxford-shirt/oxford-shirt.glb   # neutral-template rest shape, one surface
binding: res://assets/garments/oxford-shirt/binding.bin    # per garment vertex: 3 template vertex ids
                                                           # (u32), 3 weights (f32), offset (3×f32),
                                                           # offset scale reference ids
hides: { regions: [torso, upper-arms], vertices: res://…/hides.bin }   # template vertex ids, conservative
layer: 3                                                   # draw / push-out order inside the slot stack
material: { albedo: …, normal: …, orm: …, tint_mask: …, variants: [oatmeal, white, sky, navy] }
tris: 6800
```

**Fitting at load (`SD-S32-10`).** When a client draws person P in garment G:
1. For each garment vertex `v`, compute
   `pos = Σ w_i · T_P[i] + offset ⊙ scale(T_P)`, where `T_P` is P's fitting template. This is the
   `.mhclo` formula (§4.1).
2. Copy the skin weights from the template triangle's vertices: the body's weights, interpolated.
3. Recompute the normals.
4. Cache the fitted `ArrayMesh` per (P, G) for the session.

The cost is O(garment vertices), about 5 k vertices, so well under a millisecond in C++ and a few
milliseconds in GDScript. It is measured in S32-c, with a GDExtension only if GDScript misses the
budget (R-S32-5).

**Why not one bake per person × garment.** It would ship 21 people × about 70 garments ≈ 1,470 fitted
meshes, roughly 150–300 MB, and would not cover a person a user adds. Fitting at load ships each garment
once.

**Layer push-out.** Where two worn garments overlap (an apron over a shirt, a coat over a jumper), the
outer one is pushed out along the normal by the inner one's offset envelope, in slot order (`layer`).
This is the method UMA and MakeHuman both use (§4). It is computed in the same load-time pass.

### 6.3 Body hiding at runtime (`ARC-140`)

- Each garment carries a **conservative** hidden set (`hides`): template vertices fully covered by it.
  The pipeline computes the set when the asset author gives none (P-4: the stock T-shirt hid nothing) by
  ray-casting each body vertex outward against the garment shell in the A-pose and in the W-gate pose
  set (§7).
- On an outfit change the client takes the union of the worn garments' hidden sets, maps it through the
  template correspondence to body vertices, and **rebuilds the body's index buffer without the hidden
  triangles**. Outfit changes are rare events (§8.4), so a rebuild of about 25–60 k indices is
  acceptable.
- There is no per-frame cost: no `discard`, no early-z loss.
- **The shader-mask alternative.** Per-vertex region bits in a custom channel, with a per-instance uniform
  bitmask, collapsing hidden vertices in the vertex shader. It is kept as the fallback if a rebuild
  measures over 2 ms (R-S32-5).

### 6.4 Hair

- **Hair is a library of styles** (`hair/<style>/hair.yaml`), bound to the template scalp exactly as
  garments are bound, and fitted at load.
- Colour comes from the S30 `hair-colour` value through a tint mask on a neutral-luminance albedo (§6.5).
- Each style declares an `under_headwear` variant, either `hidden` or a flattened shell, used when a
  worn garment covers `scalp`.
- The body is generated with hair under a skin cap (route B) or is bald by construction (route A), and
  the `scalp` region carries the person's skin. A `buzz`/`bald` style is just a scalp tint.
- Facial hair stays an S30 attribute. In S32 it is drawn as a texture decal set on the face (fixed per
  person); a shave action is a later step (§3).

### 6.5 Colour and material variants

- **One garment model, several colours:**
  - a neutral-luminance albedo;
  - a **tint mask** (R = primary dye, G = secondary, B = trim);
  - a palette of named colours (`oatmeal`, `navy` …) in the pack's palette file.
- A garment kind's `colour` (§8.2) selects the palette entry. In Godot it is a per-instance shader
  parameter (`instance uniform`) on a shared material, so recolouring costs no texture memory.
- A print or pattern that tinting cannot express (tartan, a logo tee) is its own texture variant.

### 6.6 The garment pipeline: content at Vera's tier (`DEP-73`)

Stages, appended to `tools/character_pipeline/` as a second product beside characters (S26 §5.1's
rules: tooling, candidates, provenance, secrets):

| # | Stage | Tool |
| --- | --- | --- |
| g1 | Garment brief: category, slots, silhouette, fabric, colours, the people and roles that use it, the shops that sell it | agent, from the casting briefs and the shop lists (§10) |
| g2 | Garment reference image: the garment **worn on the neutral template mannequin** at the style template's camera and lighting, conditioned on Vera's style anchor (S26 §7.4a item 1) | OpenAI Images through `mineworld_gen` (no new client) |
| g3 | Garment 3D: image-to-3D of the garment on the mannequin, generated as one mesh, from which the mannequin is subtracted by its known surface (inside-the-template test). No part-aware generator passed §4.2 (licence, CUDA or untextured output); one that does later plugs in here as a garment source | Meshy (`DEP-51`, `DEP-73`) |
| g4 | Clean-up: single-sided shell with thickness, retopology to the slot's triangle budget (§7, W-2), UV, a 1024² or 2048² bake of albedo, normal and ORM; the tint mask from albedo segmentation | Blender headless |
| g5 | Bind: MakeClothes against `hm08` in A-pose → `.mhclo` → our `binding.bin` + `hides.bin` (§6.2) | MPFB2, headless |
| g6 | Gates W-1…W-12 (§7) over the shape sweep and the pose sweep | Godot headless + Blender |
| g7 | Scorecard beside Vera: garment rows of S26 §6.2 (category, defining structures, colour, material quality) | agent, then operator |

- **The CC0 MakeHuman packs** (CC0 only; the CC-BY packs fail `DEP-8`) are **fixtures and fallbacks for
  tests**, never the release cast.
- A user pack may adopt them, or any `.mhclo` from the community, with its licence recorded: the
  pipeline's `import-mhclo` source.

---

## 7. Gates for modular characters

S26's G-1…G-14 assume one fused mesh (F-S32-3). For a modular character they are restated per
**assembled outfit**, and the W-gates are added. Every bound is a literal from Vera's accepted record or a
cited reference, never from the candidate (`ARC-23` rule 2).

### 7.1 Restated S26 gates (`ARC-141` amends `ARC-94` for modular characters)

| Gate | Modular form |
| --- | --- |
| G-1 Triangles | ≤ 100,000 for **body + every worn part** of any outfit the world can produce. Budget per part (W-2) makes it hold by construction |
| G-2 Materials | **exactly one material per part**; parts ≤ 8 per assembled person (body, hair, up to six garments); no unbound surface |
| G-3 Texture | body 2048² (as Vera); garments and hair 1024² or 2048², declared per asset; one texel-density band per class (W-3) |
| G-4, G-5, G-6 | unchanged, on the body. G-5's 52 joints, now also on every garment's skin (its binds are the body's) |
| G-7…G-13 | run per **outfit in the outfit sweep** (W-6) instead of once |
| G-14 | provenance on the body and on every garment and hair asset |

### 7.2 Wardrobe gates (`W-n`, scripted)

| Gate | Requirement | Bound's source | Instrument |
| --- | --- | --- | --- |
| W-1 Binding | every garment vertex bound to a valid template triangle; weights sum to 1 ± 1e-4; offset scale references valid | the `.mhclo` format | binding reader |
| W-2 Part budget | body ≤ 60 k tris; hair ≤ 12 k; top or outer ≤ 10 k; legs ≤ 8 k; feet ≤ 4 k; socks ≤ 1.5 k; accessory ≤ 4 k. The largest outfit then sums to ≤ 100 k (G-1) | Vera's 95 k (D+ §5.2); P-5's measured parts | GLB reader |
| W-3 Texel density | each garment within ± 15 % of Vera's body texel density | S26 §7.4a item 2 | UV area / surface area |
| **W-4 Poke-through** | for every garment, body shape in the shape sweep (W-5) and pose in the pose sweep (G-9's stand, idle, walk and jog phases, sit): **no visible body vertex lies outside the garment shell by more than 2 mm** within the garment's covered regions, and no inner-layer vertex lies outside an outer layer by more than 2 mm. Each failure is printed with location (vertex, region, pose, shape) | D+ §8.9 (no clipping); 2 mm ≈ cloth thickness | Godot headless sweep, ray tests from the skinned positions |
| W-5 Shape sweep | every garment fits these shapes:<br>• the five S26 extremes: shortest woman 1.55 m and tallest man 1.92 m (S26 §7.2), heaviest and lightest build of the profile, oldest (stooped);<br>• plus every cast body | S26 §7.2, profile | fit at load + W-4 |
| W-6 Outfit sweep | per person, the starting outfit and three random library outfits (seeded) pass G-7…G-13 and W-4 | S26 §6.1 | Godot runner |
| W-7 Mask coverage | no hidden body triangle is visible in any sweep frame: no gap between the garment hem and visible skin, checked by rendering the body alone in a flat ID colour and the garment in another, at four views | P-4 (stock garments hid nothing) | ID-buffer render |
| W-8 Layering | every `over`/`under` pair of §8.2 renders outer-over-inner in the walk sweep (W-4's second clause) | §8.2 | as W-4 |
| W-9 Fit at load | the client's fitted mesh equals the pipeline's reference fit within 0.5 mm per vertex for every garment on every cast body | determinism | Godot vs Blender export |
| W-10 Swap | wearing and taking off each garment in sequence leaves the body index buffer equal to a fresh build's (no leaked holes) | §6.3 | runner |
| W-11 Hair under headwear | every hair style × every `scalp`-covering garment shows no hair through the hat (W-4 applied to hair) | §6.4 | as W-4 |
| W-12 Variant | every palette variant of a garment renders its named colour within ΔE₀₀ ≤ 5 of the palette value under the neutral review light | CIEDE2000 just-noticeable band | render + colour read |

### 7.3 Pose sheet and review

- The S26 pose-sheet tool (`--cast-sheet`, P-1…P-8) gains `--outfit <list>`. The per-person runtime sheet
  shows the starting outfit and two others.
- **The planted poke-through mutation (X-S32-5)** must turn W-4 red and appear red on the sheet.
- Vera, fused and accepted, stays the control on every sheet (S26 §6.3).

---

## 8. Simulation: the `wardrobe` System Pack (`ARC-135`)

### 8.1 Who owns what (`SD-S32-1`)

| State | Owner | Why |
| --- | --- | --- |
| That an item kind exists, and its category | `item` (A-1) | unchanged |
| That a kind is a **garment**: its slot, layer, coverage, category, colour, size and display name | **`wardrobe`**, a `garment:` section in the item kind's file | the item pack does not learn garments; a garment is an Item with a wardrobe section (the `naming` pattern of a section per owner, `ARC-31`) |
| How many of a kind a holder has | `inventory` (A-2) | unchanged |
| How many of a holder's units are **claimed** and so not carried, not transferable and not consumable | **`inventory`**, a new generic `Claims` component (§8.3, `ARC-136`) | `inventory` stays the single owner of holdings. It learns "claimed by a pack", never "worn" |
| **What a person wears**: the outfit, slot → kind | **`wardrobe`**, the `Outfit` component | the only new world state |
| Money and prices | `economy` / S31 | unchanged; buying a garment is an ordinary `buy` |
| Hair colour, texture, length and style | `attributes` / S30 (A-12) | §9; the haircut proposes a change and `attributes` decides it |
| The renderer key of the body | `appearance` (`look`, `ARC-91`) | unchanged and opaque; it names the **body**, and the outfit is separate state (§11) |

**The kernel learns nothing.** `wardrobe` is one crate on the `naming`/`appearance` template. It has:
- one installed line;
- one `systems:` entry per world;
- one `garment:` section per garment kind file;
- one `wardrobe:` section per person file.

Change amplification (`CLAUDE.md` §4 rule 5): one bounded `inventory` addition (§8.3); no edit to
`Person`, the kernel, the protocol, the loader, `economy` or `item-transfer`.

**Dependencies:** `item` (`is_declared`), `inventory` (claims and holdings, read through its API) and
`attributes` (hair proposals and the outfit phrase, §§9 and 12), the last one **optional**. A world
without `attributes` gets no haircut action and no outfit phrase in `description`, but the outfit itself
is still disclosed (§12). Direction: `wardrobe` → `inventory` → `item`. Nothing depends on `wardrobe`.

### 8.2 Data

**Slots are world configuration, not code** (`configure/wardrobe.yaml`; the default file is shipped by
the pack and a world may replace it):

```yaml
# configure/wardrobe.yaml — the slot model. Order is draw order, inner to outer.
format: 1
slots:
  - { id: socks,     covers_body: [feet],                     change: private, under: [feet] }
  - { id: underlayer,covers_body: [torso],                    change: private, optional: true }
  - { id: legs,      covers_body: [hips, thighs, shins],      change: private }
  - { id: top,       covers_body: [torso, upper-arms],        change: private }
  - { id: outer,     covers_body: [torso, upper-arms, forearms], change: anywhere, over: [top] }
  - { id: feet,      covers_body: [feet],                     change: anywhere, over: [socks] }
  - { id: head,      covers_body: [scalp],                    change: anywhere }
  - { id: neck,      covers_body: [],                         change: anywhere }
  - { id: bag,       covers_body: [],                         change: anywhere }
  - { id: eyewear,   covers_body: [],                         change: anywhere }
privacy_places: { tags: [home, changing-room] }   # where `private` changes are allowed (Q-S32-4)
size_chart: en-13402            # §8.6; `none` disables fit checks
durations_s: { per_garment: 30, per_private_garment: 60 }    # authored defaults (§8.4)
```

- `covers_body` names **body regions**, a closed vocabulary shared with presentation:
  `scalp, face, neck, torso, upper-arms, forearms, hands, hips, thighs, shins, feet`. This is the
  semantic statement "this slot hides the thighs". A pack maps regions to the body mesh's vertex
  regions (§6.3). The world never names a vertex.
- `over` and `under` are the **layering rules**: an outer garment goes over a top, shoes over socks. A
  covered garment cannot be taken off while the garment over it is worn (`covered`, §8.4).
- A **dress** or **overall** occupies two slots. A garment kind declares `slots: [top, legs]`.

**A garment kind** is an item kind with a `garment:` section:

```yaml
# worlds/market-town/items/oxford-shirt-oatmeal-m.yaml
item: { category: clothing }
garment:
  label: oatmeal oxford shirt          # the words observers are told (§12)
  slots: [top]
  category: shirt-collared             # closed garment vocabulary (§11.3); presentation's fallback key
  colour: oatmeal                      # named colour from the pack's palette vocabulary
  size: M                              # §8.6
  covers_body: [torso, upper-arms]     # optional narrowing of the slot's default
```

- **Kinds are fungible** (A-2). Two oatmeal M shirts are two units of one kind, and a different colour or
  size is a different kind. Authoring many variants is a tooling job:
  `mineworld wardrobe expand <model.yaml>` writes one kind file per colour × size from a model file. It is
  authoring-time code, with no runtime coupling.
- Unknown keys, a slot the world does not declare, or a category outside the vocabulary are refused at
  their line.

**A person's starting wardrobe** is a `wardrobe:` section (§10) plus ordinary `holdings`:

```yaml
holdings: { oxford-shirt-oatmeal-m: 1, apron-bib-navy: 1, chinos-charcoal-m: 1, clogs-black-39: 1,
            socks-crew-grey: 1, cardigan-oat-m: 1 }
wardrobe:
  wearing: [oxford-shirt-oatmeal-m, apron-bib-navy, chinos-charcoal-m, clogs-black-39, socks-crew-grey]
```

### 8.3 The claim: worn units are not carried (`ARC-136`, a bounded `inventory` addition)

`inventory` gains one generic concept, with no knowledge of clothing:

```text
component  Claims            per holder: (kind, claimant SystemId) → count       owned by inventory
fact       units-claimed     { holder, item, count, by }                         stated via inventory::claim(...)
fact       units-released    { holder, item, count, by }                         stated via inventory::release(...)
rule 1     claimed ≤ held, per kind                                               admit_claim
rule 2     can_take counts only unclaimed units against PERSON_CAPACITY           can_take
rule 3     transfer, sale and consumption may move only unclaimed units           admit_transfer, admit_consume
           → refused as System { code: "in-use" }  (a give/sell of a worn shirt: "take it off first")
```

- **The `ARC-26` pattern unchanged.** `wardrobe` states the facts only through inventory's checked
  constructors (`claim`, `release`), declaring the emissions and depending on `inventory`, exactly as
  `economy` states `items-transferred` today (A-2, A-5). `inventory` re-checks at reduction.
- **Why not count worn items against six.** A five-garment outfit would leave one hand free (F-S32-1). A
  worn shirt is not carried in the hands.
- **Why not let `wardrobe` own possession of worn garments.** That would give holdings two owners. A
  bought shirt moves seller → buyer through `inventory`, unchanged, and then it is worn by a claim.
- **Why not a reactive fix-up.** The alternative is that a give of a worn shirt succeeds and `wardrobe`
  then removes it from the outfit. That leaves a generation in which the outfit names a kind the person no
  longer holds, and it is unrealistic: nobody hands over the shirt they are wearing without taking it off.
  Refusing with `in-use` keeps the invariant `worn ⊆ held` true at every generation (X-S32-3).
- **Generic by construction.** A later "equipped tool" or "reserved for an order" uses the same claims
  under its own claimant id. A world without `wardrobe` never states a claim, and its digests are
  unchanged (X-S32-1).
- **Offers stay complete affordances** (`ARC-34`). `give` and `buy`-side sell offers list only kinds with
  unclaimed units.

### 8.4 Actions (all owned by `wardrobe`)

| Action | Payload | Requirement (`CLAUDE.md` §4 rule 15) | Effect |
| --- | --- | --- | --- |
| `wear` | `{ item }` | `NONE` spatially (one's own body), plus privacy (below) | the worn slot(s) take `item`; the previous occupant is released; claim and release in one dispatch |
| `take-off` | `{ item }` | as `wear` | the slot empties; release |
| `change-outfit` | `{ wear: [item…], take_off: [item…] }` | as `wear`, by the strictest slot touched | a **Process** `changing` of the summed duration; facts at its end (below) |
| `get-haircut` | `{ style }`; target: a Person offering the service, or none in a shop with a `haircut` listing | `same_place().within(1 500 mm).requiring_target_available()`, a chair's reach | §9: the price through S31, then a proposal to `attributes` |

**Privacy (`SD-S32-2`).** A slot with `change: private` (socks, under-layer, legs, top) may be changed only
in a Place whose tags meet `privacy_places`: the person's home, or a shop's changing room. Elsewhere the
action is refused with `System { code: "needs-privacy" }`. A slot with `change: anywhere` (outer, feet,
head, accessories) may be changed anywhere. The policy is world data, so a world may relax it (Q-S32-4).

**Duration and visibility (`SD-S32-3`).**
- **`wear` and `take-off` of an `anywhere` garment are instant.** Their fact is public to the place:
  `Visibility::Place(place)`. Everyone present sees the coat come off.
- **`change-outfit` runs as a Process.** Its duration is `Σ per_garment` (30 s each, `per_private_garment`
  60 s). These are authored defaults — "about a minute per garment" — and are stated as authored rather
  than cited.
  - During the Process the person is `Busy` to other actions, as in any Process.
  - **The outfit changes at the Process's end, all at once.** The `outfit-changed` fact goes out then,
    with audience `Place(place)`.
  - Who perceives a change made in private is therefore decided by who is in that place: in a home, the
    household; in a changing room, nobody else. Everyone else learns of the new outfit when they next
    perceive the person, through disclosure (§12).
  - Interrupting the Process (for example a conversation request) ends it, and the outfit stays as it
    was. No half-dressed state exists in the world.
- **Refusals, by name** (`Rejection::System { code }` unless kernel-classed):

| Code | When |
| --- | --- |
| `not-a-garment` | the kind has no `garment:` section |
| `not-held` | the person holds no unclaimed unit of it (`wear`) |
| `not-worn` | `take-off` of a kind the outfit does not hold |
| `covered` | taking off a garment while one `over` it is worn (socks under shoes) |
| `slot-conflict` | a `change-outfit` wearing two garments for one slot |
| `needs-privacy` | a `private` slot outside a privacy place |
| `does-not-fit` | §8.6, when the world's `size_chart` is not `none` |
| kernel `Busy` | already changing, or in another non-interruptible Process |

- **Offers.** `wear` and `take-off` are complete affordances: `wear` is offered for each held, unworn
  garment that is allowed here, and `take-off` for each worn, uncovered one. `change-outfit` is offered as
  an incomplete affordance, because the agent composes the list.

### 8.5 Events (all owned by `wardrobe`)

| Fact | Payload | Audience |
| --- | --- | --- |
| `outfit-stated` | `{ person, worn: [(slot, item)…] }`, at genesis from the section | the person; disclosure covers everyone else (§12) |
| `garment-worn` / `garment-taken-off` | `{ person, item, slots, place }` | `Place(place)` |
| `outfit-changed` | `{ person, before: [(slot, item)], after: [(slot, item)], place }` | `Place(place)` |
| `changing-started` | `{ person, place, until }` | `Place(place)` (someone is changing; for the household and the client's "changing" pose) |

Every fact cites its cause (INV-15): the action, or genesis.

### 8.6 Sizes and fit (`SD-S32-4`)

- **Sizes are real.** The default chart is **EN 13402**: letter sizes by chest/bust, waist and hip girth
  in cm, and shoe sizes by EN 13402-1's foot length. This is the realistic-defaults rule; the exact table
  values are marked **verify** for S32-a against the standard's published tables, which are not free —
  §20 asks whether to buy the standard (Q-S32-9) or use a cited free secondary source.
- **A person's size is derived, not stored.** It is computed from the S30 `stature`, `mass` and `build`
  attributes through the chart (S30 `derived` mutability) and exposed by `wardrobe` as a derived
  attribute, `wardrobe.size`, through `AttributeOwner`. A world without `attributes` has no derived size,
  and fit is not checked.
- **The fit rule.** `wear` is refused `does-not-fit` when the garment's size is more than one step from
  the wearer's size (too small); one step either way is worn as `snug` or `loose`. A larger garment is
  allowed and worn as `loose`. The fit is perceivable ("in a loose grey coat"). A world sets
  `size_chart: none` to disable all of it.
- **Presentation does not resize.** A garment mesh is fitted to the body by the pipeline (§6.2) whatever
  its semantic size, so `loose` and `snug` are words in the world, not geometry. A later step may map
  `loose` to an inflated variant; recorded as F-S32-5.

---

## 9. Hairstyle (`ARC-137`)

### 9.1 Decision: hair is a person attribute, changed by a service; headwear is a garment (`SD-S32-5`)

| | Option | Verdict |
| --- | --- | --- |
| H-1 | Hairstyle as an **Item** worn in a `hair` slot | **rejected.** Hair is not owned, carried, given or sold. It would let a person hand their bun to a stranger, and it fills inventory |
| H-2 | Hairstyle as an **S30 attribute** (`hair-style`, `hair-length`, `hair-colour`, `evolving`), changed by a **`get-haircut` service** whose provider proposes the change | **chosen.** It matches reality and S30's ownership model (A-12): `attributes` owns the trait; `wardrobe` declares `proposes: [core.hair-style, core.hair-length, core.hair-colour]` |
| H-3 | A wig as an Item | allowed under H-2, as a garment in the `head` slot with `covers_body: [scalp]` and `category: wig`. It is not a separate mechanism |

**The flow.**

```text
get-haircut { style } from a stylist at a salon chair (or a shop listing `haircut`)
  1. economy settles the service like a sale (S31 §5.7), item = the service kind `haircut-<style>`, a
     kind with `service: true` that is never held (S31 owns how a service sale settles: Q-S32-6)
  2. wardrobe emits attribute-change-proposed { person, core.hair-style, to: style, because: sold }
     (+ hair-length when the style implies it: a bob from long hair shortens; long from short is refused
     `too-short`, since hair grows, it is not added — a wig is the instant alternative)
  3. attributes states trait-changed { … } (audience: holder + observers present, A-12)
```

- **Duration:** a haircut is a Process of 30 min (authored default) at the salon, with the person and the
  stylist `Busy`.
- **Growth:** not modelled in S32 (S30 says drift belongs to a System that models it). F-S32-6.
- **What the client draws:** the `hair-style` and `hair-colour` attribute values are disclosed (S30
  perceivable). The pack binds a style slug to a hair asset and tints it by colour (§11). A headwear
  garment with `covers_body: [scalp]` hides the hair asset, or swaps it for the style's "under-hat"
  variant (§6.4).

### 9.2 The S30 overlap (F-S32-4, `SD-S32-6`)

- S30's v2 "dress" attributes split in two:
  - **Preferences** (clothing style, colour preference) stay S30 attributes. They are personality, and a
    controller may use them to choose purchases.
  - **Worn state** (headwear, footwear, accessories, glasses as worn objects) is **not** an S30 attribute
    where `wardrobe` is installed. `wardrobe` provides it as the derived, perceivable attribute
    `wardrobe.outfit`, through `AttributeOwner`, so that one renderer phrases it.
- S30's catalogue marks those dress attributes `superseded_by: wardrobe.outfit` when `wardrobe` is
  present. That is a one-line S30 change proposed to S30-c, its owner (§19).
- **Glasses** stay an S30 `fixed` attribute ("wears glasses"), because a person who needs glasses wears
  them. Sunglasses are a garment in `eyewear`.

---

## 10. Starting wardrobes from the casting brief (`SD-S32-7`)

S26's `brief.yaml` has `outfit: { outer, under, legs, feet }` and `accessories` (S26 §7.1). S32 makes the
brief's outfit **the starting outfit**, and adds spares:

1. **The brief's outfit becomes kinds.** The S26 casting director (tooling, `mvp0/s26-cast-director`)
   gains a `wardrobe` stage. It maps every brief garment to a garment model in the pack's library (§6)
   by category and colour, and writes:
   - the world's garment kind files, through `wardrobe expand`;
   - the person file's `holdings` and `wardrobe.wearing`.

   A brief garment the library cannot match is a **library gap**. It is listed on the casting page and
   filled by the garment pipeline (§6.2) before that person's casting is final. It is never silently
   replaced by a near miss.
2. **Spares: "a few garments" (operator).** By default each person also owns **two spare garments**, one
   top and one other slot. They are chosen by the brief's role and the S30 clothing-style preference,
   from the same library, deterministic from the world seed and the look key (the S26 sampler rule). With
   a typical five worn garments plus two spares, one unclaimed slot of six is left for carrying
   (§8.3) — tight. The alternative is spares kept at home (Q-S32-7).
3. **Hairstyle.** It comes from the brief's `hair` field through S30 (`core.hair-style`, `core.hair-colour`
   resolved by S30's sampler or the biography). There is no wardrobe data for it.
4. **Determinism.** The same seed, brief and library version give byte-identical person files (X-S32-9).
5. **Shops.** `market-town`'s new **outfitters** organization (S32-f) stocks the library's everyday kinds
   at seller-set prices (S31). Realistic default prices are cited from public US retail medians, marked
   **verify** in S32-f; the currency follows S31's `configure/economy.yaml`.

---

## 11. Presentation binding

### 11.1 What a client resolves (`ARC-139`)

A person's drawing is resolved from three disclosed things, in both clients:

```text
look           (appearance, ARC-91)   → the BODY: a modular character's body + fitting template
worn           (wardrobe, §12.1)      → one garment asset per worn kind
hair-style,    (attributes, S30)      → a hair asset + tint
hair-colour
```

- **`look` evolves; it does not disappear.** It names the body (face, skin, shape), which the world never
  changes through clothing. `ARC-91` item 5 ("a clothing System re-emits `appeared`") is **not** used:
  re-keying the whole look on every outfit change would make the look key enumerate outfits, which
  defeats an opaque key. A dated note to `ARC-91` (in `ARC-139`) records that clothing is separate state
  and the look stays the body.
- **3D `cast.yaml` (`ARC-92`) gains two fields per character in `character.yaml` (`ARC-93`):**
  - `modular: true`, with `body`, `template` and `hair_scalp`;
  - or `baked: true` for a fused character (§5.3).

  A baked character ignores `worn` and hair. The client logs once per session that the person's outfit
  is not drawn, and the release check refuses a baked **Person** in the default pack (X-S32-12).
- **Garment kinds bind to assets** in `presentation/mineworld-default/3D/garments/bindings.yaml`. Each
  binding is keyed by the **item kind's `category` + `colour`**, never by the world key:

  ```json
  { "format": 1,
    "garments": { "shirt-collared": "oxford-shirt", "apron-bib": "bib-apron", "...": "..." },
    "colours":  "palette.yaml",
    "hair":     { "bun-low": "bun-low", "bob": "bob-chin", "...": "..." } }
  ```

  Resolution order, the same in both clients:
  1. category → asset;
  2. colour → palette tint;
  3. an unknown category → the slot's **generic** asset (a plain tee, plain trousers, plain shoes),
     tinted by colour;
  4. an unknown colour → the asset's default variant.

  The world never names an asset, and a pack never names a world key (`ARC-46` holds).
- **3D runtime:** `human.gd` builds one `Skeleton3D`, the body, and one `MeshInstance3D` per worn part,
  fitted at load (§6.2). On `outfit-changed` or `garment-*` it swaps parts and rebuilds the body mask
  (§6.3). During `changing-started` it plays the idle stand (no undressing animation; out of scope).
  Godot's importer renames `mixamorig:` to `mixamorig_` (P-6); every bone lookup uses the imported
  names.

### 11.2 The 2D client: layered sprites rendered from the 3D characters (`SD-S32-11`; Q-S32-2)

**The decision:** 2D garments are **layers**, and the layers are **rendered offline from the 3D modular
characters** by a stylizing render (`tools/character_pipeline/sprites2d/`), per **body class** rather than
per person.

| | Option | Cost | Verdict |
| --- | --- | --- | --- |
| 2D-1 | One generated sprite set per **outfit** (today's method, `gen_player`) | impossible in general: buying makes outfits at runtime; even the starting outfits alone mean 21 × 4 images, and every purchasable combination is unbounded | **rejected**: it cannot show a bought shirt |
| 2D-2 | **Generated paper-doll layers**: per body class, each garment generated as a separate transparent layer by image edit against the class's base sprite | 6 body classes × ~70 garments × 4 views ≈ 1,700 images at OpenAI's high-quality per-image price (**verify** in S32-e; the S26-b1 price read applies), plus alignment repair | **fallback**: alignment across separately generated layers is the known failure of generated paper dolls; every layer needs a check |
| 2D-3 | **Layers rendered from the 3D characters** with a stylizing shader matched to the 2D style references (flat-shaded, ink outline, the 2D palette): body+head+hair per person, each garment per body class, at the 2D camera's pitch, front and back, two stride frames | no image-model cost; the Godot headless render is about minutes per world; ≈ 6 classes × 70 garments × 4 views × ≈ 6 KB ≈ 10 MB + per-person bodies ≈ 1 MB: inside the 2D bundle's 40 MB | **chosen**: alignment is exact by construction (same rig, same pose, same camera), the same person in both clients is guaranteed (S26 AC-S26-7), and a bought shirt is a layer that already exists |

- **Body classes** = sex × three builds (6). These are the S30 `build` bands, at ≈ 110 px tall, where
  finer body differences are below a pixel.
- The person's own layer (body, face, hair) is rendered from their own character, normalized to
  stature × 64 px (S26 §9).
- **The style question is real.** The 2D pack's current art is painterly generated sprites. 2D-3 changes
  how people are drawn, so the stylized render must pass a 2D style review beside the existing 2D street
  (`ARC-3`: packs share meaning, not art). If the operator rejects the look, 2D-2 is the fallback, with
  its cost asked first (Q-S32-2).
- **2D bindings.** `asset_bindings.yaml` gains:
  - the `garment:<category>:<class>` and `hair:<style>:<class>` roles;
  - layer order from the slot order of the world's `configure/wardrobe.yaml`, read from the disclosed
    `worn[].slot` and the slot list the wardrobe pack discloses once per session.

  `people.gd` draws a stack of `Sprite2D`s under the `body` node (A-17) and tints them by colour. An
  unbound garment draws the slot's generic layer, and an unbound person stays on today's `person:<n>`
  path.

### 11.3 The closed garment vocabulary

`category` is a closed list shipped by the `wardrobe` pack: about 60 slugs such as `tee`,
`shirt-collared`, `blouse`, `jumper`, `cardigan`, `hoodie`, `apron-bib`, `jacket-casual`, `coat-over`,
`raincoat`, `trousers`, `jeans`, `skirt-knee`, `dress`, `dungarees`, `shoes-trainer`, `boots-rubber`,
`clogs`, `socks-crew`, `cap-flat`, `beanie`, `headscarf`, `scarf`, `bag-shoulder`, `backpack`.
- It is the shared language between worlds and packs, as the body-region list is.
- A world may extend it in `configure/wardrobe.yaml`, with each extension mapped to a parent category
  for the fallback.
- **Adding a category is a data change** (X-S32-10).

---

## 12. Perception: clients, cognition, MCP and memory (`ARC-139`)

### 12.1 What is disclosed

- **`Outfit` is disclosed to every observer who perceives the person**, filtered by visibility:
  - a garment whose slot is `under` a worn garment that covers it entirely (socks under boots; the
    under-layer under a top) is **not disclosed** to others;
  - the holder sees everything.

  This is the S30 perceivable rule applied to dress: an agent sees what a human would see.
- The disclosed value is typed:

  ```text
  worn: [ { slot, item, label, category, colour, fit } … ]
  ```

  The ids let a client bind assets, and the label, category and colour let a controller describe them.
  It never carries a mesh, file or texture name.
- `hair-style`, `hair-length` and `hair-colour` come through S30's `traits` disclosure, unchanged.

### 12.2 Words for agents (MCP and LM controllers)

- `wardrobe.outfit`, a derived perceivable attribute (§9.2), phrases the visible outfit through S30's one
  renderer. For example: "in an oatmeal oxford shirt under a navy bib apron, charcoal chinos and black
  clogs".
  - The phrase is ranked into `PersonLine.description`'s 160 B by S30's `prompt_rank`, so outfit words
    compete fairly with stature and hair.
  - The **full structured `worn` list** is in the MCP `perceive` tool's person record (S27-d), so an
    agent never has to parse prose.
- **Self.** The observer's own record lists every held garment with `worn: true|false`, `slot` and
  `fit`, plus the `wear`, `take-off` and `change-outfit` offers. An agent can dress itself from the
  observation alone.

### 12.3 Noticing a change (S28 memory)

- `outfit-changed` and `trait-changed` (hair) are perceived facts for everyone present. S28's memory
  ingests perceived facts as it does any other.
- For an observer who was **not** present, the change is noticed by comparison. On perceiving a known
  person, the memory layer compares the disclosed outfit and hair with the last remembered ones and
  records a `noticed-appearance-change { person, what }` memory item, at most one per person per
  encounter.
  - It is a **cognition-side derivation**, not a world fact. It belongs to S28's memory home
    (`pr-s28a-memory-home.md`) as one perceived-delta rule, and is proposed as S32-g (§16).
  - It lets an NPC say "new coat?" without a System ever deciding who noticed.
- A person whose outfit changes in private is noticed only by those who see them later. The world never
  tells anyone that a change happened out of sight.

---

## 13. Persistence and digests (`SD-S32-8`)

- **Saved:**
  - `Outfit` (`wardrobe`) and `Claims` (`inventory`), reduced from their facts. Replay reduces
    `outfit-stated`, `garment-worn`, `garment-taken-off`, `outfit-changed`, `units-claimed` and
    `units-released` exactly (`ARC-25`).
  - A running `changing` Process is a stored record (A-10), so a save mid-change resumes mid-change.
- **Digests:** `Outfit` and `Claims` enter the state digest like every owned component.
  - A world without `wardrobe` states no claim and no outfit, so its golden digests are **byte-identical**
    (X-S32-1).
  - Worlds that adopt `wardrobe` re-baseline in one reviewed commit (S26 R-S26-5's practice).
- **Schema versions:** `outfit` v1, `claims` v1, `garment` section format 1.

---

## 14. Performance and size

### 14.1 Frame time (budget: RL-b R-2, p95 ≤ 16.7 ms per view on the operator's M5, binding)

| | Fused (S26) | Modular (S32), per person |
| --- | --- | --- |
| Skinned meshes | 1 | ≤ 8 (P-6 measured 7) |
| Surfaces → draw calls | 1 opaque + 1 shadow | ≤ 8 opaque + ≤ 8 shadow |
| Triangles | ≤ 100 k | ≤ 100 k (W-2), typically 60–80 k |
| Skinning work | one mesh | the same vertex count split across parts; Godot skins each `MeshInstance3D` separately |

- **The crowd view** (S26 §11: 12 people + 4 decor figures): ≈ 16 × 16 = **256 draw calls** for people,
  against the advisory 2,000. The triangle count is unchanged, and the frame is GPU-bound (S26 A-25).
- **Expectation:** within budget. It is measured in S32-c on the crowd view, by RL-b's protocol.
- **Mitigations, in order, each measured:**
  1. **merge at outfit change**: combine the body and the fitted parts into one skinned `ArrayMesh` with
     a texture-array material, the UMA approach (§4). One draw per person again, rebuilt only on a
     change;
  2. shadow casting off for parts beyond 15 m (shadow of the body only);
  3. LODs on the fitted parts (Godot import LODs on the garment GLB; fitting maps LOD vertices by the
     same binding when generated from the full mesh).

### 14.2 Release size (budget: `3d.pck` per Q-S26-5)

| Item | Fused cast (S26 S-1) | Modular cast (estimate; the pilot measures) |
| --- | --- | --- |
| Per person | ≈ 7.5–8.5 MB (2048² texture + 95 k mesh) | body ≈ 5.6 MB texture (BC7) or ≈ 2 MB (UASTC) + 60 k mesh ≈ 1.5 MB + template 0.23 MB |
| 21 people | ≈ 160–180 MB | ≈ 80–155 MB |
| Shared garments + hair, ≈ 70 + 20 assets | — | ≈ 90 × (1024² UASTC ≈ 0.6 MB × 3 maps + 0.3 MB mesh) ≈ 190 MB raw; ≈ 60–110 MB with shared ORM/normal detail maps and 1024² for small parts |
| **Added to `3d.pck`** | ≈ 160–180 MB | **≈ 140–265 MB**, with the low end needing UASTC (Q-S26-5's ruling already applies) |

The honest reading:
- the modular cast is **not obviously smaller**: it trades per-person garment pixels for a shared
  library;
- it **grows sub-linearly** with people, since a new person adds only a body;
- the pilot measures real bytes;
- the Q-S26-5 ruling (UASTC, else about 350 MB) covers it, and Q-S32-8 asks again with numbers if not.

---

## 15. Impact on S26 (the cast in flight)

### 15.1 What S26 keeps, changes and stops

| S26 element | Fate |
| --- | --- |
| Requirement §1: same process, same standard, every NPC | **kept**, and extended: "same process" now means the modular route for every Person |
| `appearance` pack, `look`, `cast.yaml` (`ARC-91/92`, S26-a) | **kept**; `look` names the body (§11.1); `character.yaml` gains `modular`/`baked` |
| Casting briefs, the sampler, S30 profiles, distinctness (§7) | **kept**; the brief's `outfit` becomes the starting wardrobe (§10), and distinctness's "outer garment" criterion reads the starting outfit |
| Reference images (in progress for 21 people) | **kept as identity references** (face, hair, palette, outfit), approved at the casting gate as planned. **Route B adds** one derived image per person: the same person in the neutral base layer with hair under a cap, conditioned on the approved reference (OpenAI cost only, ≈ 1–2 images per person). Their outfits seed the garment briefs (§6.6 g1) |
| Meshy phase 2 (fused whole characters) | **stays on hold**. Fused generation for the cast is **cancelled**, not resumed: a fused Person cannot change clothes (§1). Under route B, Meshy runs on the base-layer image instead (same credits per person, same retry policy); under route A, no Meshy run per person |
| Pipeline PR #211 (`tools/character_pipeline/`, stages 3, 6–12) | **kept**: route B's body runs it unchanged from stage 3 to 12. Stage 10's export splits into body + template; garment stages g1–g7 are added beside it (§6.6) |
| Gates G-1…G-14 | **kept**, restated per assembled outfit (§7.1, `ARC-141`), plus W-1…W-12 |
| Pose-sheet tool | **kept**, gains `--outfit` (§7.3) |
| Sources amendment (`mpfb`, `charmorph`, `import-vrm`, bone-map schemes) | **aligned**: `mpfb` becomes the template and route-A source; P-2 proved scheme `mpfb-mixamo` is the identity map, so the `mpfb-game-engine` scheme is unnecessary for us |
| Budget §10 (3,000-credit cap) | **re-estimated** in §16 and Q-S32-1: bodies under route B ≈ the same per person; garments are new spend |
| §7.4 "what the brief avoids" (long hair, long skirts for sitters) | **relaxed where the modular route removes the cause**: a skirt is a separate garment with its own sit gate. Loose long hair remains a limit (rigid hair shells; spring bones are a later step, F-S32-7) |

### 15.2 The amendment note for `step-26-cast.md` (proposed; the operator rules first)

Only a pointer is written into step-26 by this session. The amendment text proposed for the operator's
ruling:

> **Amendment proposal (2026-10-15, S32; not in force until ruled).** The operator's wardrobe requirement
> (step-32 §0.1) makes a fused character unable to serve a Person. If ruled as step-32 recommends:
> 1. Meshy phase 2's fused generation of the cast is cancelled. The casting briefs and approved reference
>    images stand.
> 2. Each Person's 3D character becomes a **modular character** (step-32 §§5–6): body + fitting template +
>    garments + hair, on the same 52 joints. The body route (B, hybrid; or A, parametric) is chosen by the
>    S32-p pilot's gates.
> 3. G-1…G-14 apply per assembled outfit (`ARC-141`), and W-1…W-12 are added.
> 4. Vera stays the accepted fused protagonist and the review control until her modular character passes
>    parity beside her. Her fused body is not shipped as a Person in the release.
> 5. `character.yaml` gains `modular`/`baked`; `look` names the body.
> 6. The S26 PRs S26-b2 (pilot), S26-d (batches) and S26-e (2D sprites) are re-scoped by S32-p, S32-d
>    and S32-e (step-32 §16). S26-a, S26-b1, S26-cast, S26-c1, S26-c2 and S26-f proceed, with the
>    changes listed in step-32 §15.1.

### 15.3 Re-scoped S26 PRs

| S26 PR | Status under S32 |
| --- | --- |
| S26-a (appearance) | unchanged |
| S26-b1 (pipeline import, #211) | unchanged; merges as is |
| S26-b2 (automation spike + fused pilot) | **merged into S32-p**: the landmark, weights and Meshy-rig comparisons run on the base-layer bodies |
| S26-cast (briefs, references) | continues; adds the base-layer derived image per person (route B) after S32-p rules |
| S26-c1 (`character.yaml`) | continues; S32-c extends the manifest |
| S26-c2 (connected figures) | continues; S32-c draws outfits on the same path |
| S26-d (batches) | **replaced by S32-d** (modular batches: bodies, then garments) |
| S26-e (2D sprites per person) | **replaced by S32-e** (2D layers, §11.2) |
| S26-f (release packing) | continues, with S32's size numbers |

---

## 16. PR split

```text
S32-a1 inventory claims ─┐
S32-a2 wardrobe pack ────┴─→ S32-b haircut + S30 views ─┐
S32-p  pilot (routes A/B, garment pipeline, W-gates) ───┼─→ S32-c 3D modular characters ─┐
                                                        │                                 ├─→ S32-d modular batches ─┐
S32-f  worlds: shops, starting wardrobes, 3D world ─────┤                                 │                          ├─→ S32-h release
S32-g  cognition, MCP, memory noticing ─────────────────┘   S32-e 2D layers from 3D ──────┘                          │
```

Every PR is detailed to the commit and frozen by the primary session before it starts, in a fresh session
and its own worktree. No PR spends a Meshy credit, makes an OpenAI call or buys anything outside the caps
the operator rules (Q-S32-1, Q-S32-2, Q-S32-9).

| PR | Scope | Depends on | Integration checkpoint | Adversarial criteria | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S32-a1** inventory claims (`ARC-136`) | `systems/inventory`:<br>• `Claims` component;<br>• `units-claimed`/`units-released` facts;<br>• `claim`/`release` checked constructors;<br>• `can_take`, `admit_transfer` and `admit_consumption` count only unclaimed units;<br>• `in-use` refusal;<br>• offers skip claimed units.<br>Plus `MODULE_SPEC.md` holdings text | none | market-town digests **unchanged** (no pack claims yet); a test pack claims a unit and a give of it is refused `in-use` | X-S32-1 a world without claims: byte-identical digests; X-S32-3 a give of a claimed unit refused, and the holdings unchanged | **Haiku**, from a commit plan; Opus review |
| **S32-a2** `wardrobe` pack (`ARC-135`) | `systems/wardrobe`:<br>• `configure/wardrobe.yaml`;<br>• `garment:` and `wardrobe:` sections;<br>• `Outfit`;<br>• `wear`, `take-off`, `change-outfit` (Process);<br>• facts, privacy, sizes;<br>• disclosure with covered-layer filtering;<br>• offers;<br>• `mineworld wardrobe expand`;<br>• the category vocabulary.<br>Installed line | S32-a1; S31's `buy` (only for the integration test) | `mineworld server` on a fixture world: buy a shirt in a shop, walk home, `change-outfit`, observe: the outfit is disclosed to a second person present, and not to one elsewhere until they meet; save/restart mid-change resumes | X-S32-2 `needs-privacy` in the street; X-S32-4 `covered` for socks under shoes; X-S32-6 an interrupted change leaves the outfit unchanged; X-S32-7 socks under boots not disclosed to others, disclosed to self | **Opus** commit plan → **Haiku**; Opus review |
| **S32-b** haircut and S30 views (`ARC-137`) | `get-haircut` (proposal to `attributes`); S31 service sale (Q-S32-6); derived attributes `wardrobe.outfit`, `wardrobe.size`; `superseded_by` on S30's dress attributes | S32-a2; S30-c merged; S31's service ruling | a haircut in a salon changes `core.hair-style`; an observer present gets `trait-changed`; `description` reads the new hair and outfit | X-S32-8 `too-short` for long-from-short; a proposal without `proposes:` is refused by `attributes` | **Haiku** |
| **S32-p** pilot (`ARC-138`, `ARC-140`, `ARC-141`; `DEP-72/73/75`) | **Garment pipeline g1–g7** for a pilot library of 12 garments + 4 hair styles, and the runtime format converter.<br>**Bodies:** route B for Vera, Ivan, Hana; route A for the same three.<br>**Runners:** W-1…W-12 headless; fit at load in a GDScript prototype; the pose sheet with `--outfit`.<br>Measured: credits, bytes, per-change rebuild time, frame time on a 16-person crowd | S26-b1 (#211) merged; operator rulings Q-S32-1 (pilot cap) and the pilot casting page; no S32-a dependency | the pilot review page `VIS-3D-WARDROBE-P`: each subject × 2 routes × 3 outfits beside fused Vera, with the scorecard and W-gates; the route chosen by §5.2's rule; the cost table re-estimated | **X-S32-5 a planted poke-through (the shirt shell shrunk 4 mm on the chest) fails W-4 with location and turns the sheet red**; X-S32-11 a garment with an empty hidden set on a tight top fails W-7 | **Opus** (research, vision judgement) |
| **S32-c** 3D modular characters (`ARC-139`, `ARC-140`) | `human.gd` and `npc.gd`: body + parts on one skeleton; fit at load; mask rebuild; `garments/bindings.yaml` and the palette; the `modular`/`baked` manifest; outfit events from observation; imported bone names (`mixamorig_`); the `--perf` crowd view | S32-p's route ruling; S26-c1 merged; S32-a2 (for the live path) | `./mineworld-slice --world` (the composed 3D world, S32-f): the protagonist buys a shirt, changes, and is drawn in it; two connected clients see each other's outfits; the crowd view inside 16.7 ms | X-S32-12 a baked Person in the default pack fails the release check; W-9/W-10 in CI on the fixtures; X-S32-13 an unbound category draws the slot's generic garment tinted, never nothing | **Opus** (fitting, masks, perf); mechanical bindings to **Haiku** |
| **S32-d** modular batches | the full garment library (≈ 70 garments, ≈ 20 hair styles, from the briefs and the shop lists); every cast body by the ruled route; review pages per batch (runtime pages parked for the big milestone, S26 §6.4) | S32-p ruled; S32-c merged; casting pages ruled | every cast person in starting outfit + 3 outfits through W-gates and the scorecard | X-S32-14 a garment with a CC-BY source fails the licence scan; X-S32-15 a batch with mixed pipeline versions is refused (S26 X-22) | **Opus** for gates and scorecards; mechanical stages delegated to **Haiku** |
| **S32-e** 2D layers (`DEP-76`) | the sprite-bake renderer (stylizing shader matched to the 2D references); body classes; `garment:`/`hair:` roles; `people.gd` layer stack and tints | S32-p (assets to render); Q-S32-2 ruled | `./mineworld-2d` on the composed world: the bought shirt appears as a layer; every person's layer stack matches the 3D outfit | X-S32-16 a garment missing its 2D layer draws the slot's generic layer; AC-W12 height check holds per person | **Opus** for the style match; **Haiku** for bindings |
| **S32-f** worlds | `market-town`: an `outfitters` organization with a shop, garment kind files, reference values and prices (S31), starting wardrobes for all twelve; `lakeside` likewise; **the 3D world's composition** gains `item`, `inventory`, `item-transfer`, `economy`, `wardrobe` (Q-S32-11); the casting director's wardrobe stage (§10); baselines re-recorded with reasons | S32-a2; S31-f; S26-cast briefs | `mineworld run market-town 30d`: purchases of clothing happen; digests deterministic across reruns | X-S32-9 the director re-run reproduces person files byte for byte; X-S32-17 `bodies-yard` (no wardrobe) unchanged | **Haiku** (data); Opus for the director stage |
| **S32-g** cognition, MCP, memory | S30 renderer ranks the outfit phrase; MCP person records carry `worn[]`, `me.garments[]` and offers; the S28 perceived-delta rule `noticed-appearance-change` | S32-a2; S30-f; S27-d; S28-a | an MCP agent buys and wears a shirt by tools alone; an NPC who met the person yesterday records the change on today's meeting | X-S32-18 an agent not present sees no `outfit-changed`; X-S32-19 the noticing rule fires at most once per person per encounter | **Haiku** |
| **S32-h** release | S26-f's packing with garments; texture policy; the size gate with measured numbers; the baked-Person check | S32-c, S32-d, S32-e; R-a | exported `3d.pck` within the ruled limit; perf PASS on the operator's machine | S26 X-21 extended to garments | **Haiku** |

**MVP placement:** every PR is MVP-0 and release-blocking (§0.1).

---

## 17. Step acceptance and adversarial criteria

| | Criterion | How it is shown to fail when broken |
| --- | --- | --- |
| AC-S32-1 | **Buy a shirt, then wear it, and it shows in both clients and in an agent's observation.** In the composed 3D world, the protagonist buys an oatmeal shirt at the outfitters (S31 price), changes in the changing room, and walks out. The 3D client draws the shirt; the 2D client draws the shirt layer; a second player's MCP `perceive` lists `worn[{slot: top, label: "oatmeal oxford shirt"}]`; an LM controller's "who I see" line mentions it | a scripted end-to-end test drives the server, both clients headless (frame captures compared with the shirt's palette colour in the torso region) and the MCP adapter. Breaking any link (disclosure, binding, layer, phrase) fails it at that link |
| AC-S32-2 | **A world without `wardrobe` loads unchanged** | `bodies-yard`, `social-cafe` (if not composed with it) and every golden world without the pack: byte-identical digests (X-S32-1, X-S32-17) |
| AC-S32-3 | **A planted poke-through is caught by a gate** | X-S32-5: the shrunk shirt fails W-4 by location and the pose sheet shows red |
| AC-S32-4 | `worn ⊆ held` at every generation; a worn garment cannot be given, sold or consumed | X-S32-3, and a property test over random action sequences |
| AC-S32-5 | Privacy, layering and fit refusals are named and correct | X-S32-2, X-S32-4, and `does-not-fit` with the chart on |
| AC-S32-6 | Perception is honest: covered layers are hidden from others; a change made in private is not perceived by the absent | X-S32-7, X-S32-18 |
| AC-S32-7 | Every Person in the default pack is a modular character at Vera's tier | the release check (X-S32-12); operator verdicts in `HUMAN_REVIEW_QUEUE.md` `VIS-3D-WARDROBE-*` |
| AC-S32-8 | Every garment fits the shape sweep and the outfit sweep | W-4…W-8 in CI on committed assets |
| AC-S32-9 | Hairstyle changes through the haircut and is perceived | S32-b's checkpoint; X-S32-8 |
| AC-S32-10 | Starting wardrobes come from the briefs deterministically | X-S32-9 |
| AC-S32-11 | Release size and frame time within the ruled budgets | S32-h; the crowd view |
| AC-S32-12 | Framework, not game: a new category, slot or garment is data | X-S32-10: a test world adds a slot `gloves` and a category `mittens` with no code change, and the 2D/3D fallbacks draw it |

---

## 18. Risks

| | Risk | Mitigation |
| --- | --- | --- |
| R-S32-1 | **Neither body route reaches Vera's tier** while wearing separate garments | §5.2's rule fixes the bar before the pilot. No scope drop (§0.1). The levers are researched and presented with costs:<br>• (a) route B with the face region kept and the template used only below the neck;<br>• (b) higher-tier garment content: commissioned garments, or paid assets whose licence passes `DEP-8`;<br>• (c) per-person bespoke garments for the starting outfit, with shared garments only for purchases;<br>• (d) a hosted generator with layered output, if one appears with acceptable terms |
| R-S32-2 | **Route B registration fails** on some bodies (loose base layer, unusual builds) | the base-layer reference is skin-tight by brief; joint-driven pre-pose before surface fit; W-4/W-5 catch errors by location; RSWT for weights |
| R-S32-3 | **Garment generation retention is low** (fused mannequin, thin shells) | g3's mannequin subtraction; retry budget per garment; the pilot measures retention before the library is budgeted |
| R-S32-4 | **The generated base-layer body differs in identity** from the approved reference | conditioned on the approved reference; checked at the casting gate beside it; face identity scorecard row |
| R-S32-5 | Fit at load or the mask rebuild is too slow in GDScript | measured in S32-p; GDExtension (C++, all three platforms) or the shader-mask fallback (§6.3) |
| R-S32-6 | MPFB2 sliders give unrealistic statures (P-2: 1.28 m) | calibrated maps from S30 attributes to macros, fitted against the profile's statures; W-5's extremes are cited (S26 §7.2) |
| R-S32-7 | Credits exceed S26's 3,000 cap | asked as Q-S32-1 with a new cap; the pilot re-estimates before the library starts |
| R-S32-8 | Release size grows (§14.2) | UASTC (Q-S26-5); 1024² for small parts; measured by the pilot; Q-S32-8 |
| R-S32-9 | The 2D style of rendered layers is rejected | 2D-2 fallback, costed (Q-S32-2) |
| R-S32-10 | Inventory capacity pressure (six) with spares | claims (§8.3); spares capped at two; home storage as Q-S32-7 |
| R-S32-11 | MPFB2 changes or breaks on a future Blender | version pinned per batch in provenance (S26 §7.4a item 3); it is a tool, so the shipped data does not depend on it |
| R-S32-12 | Importer bone renaming (`:` → `_`) silently breaks lookups | P-6 finding; S32-c uses imported names, and W-9 asserts that every bind resolves |

---

## 19. Records and edits

### 19.1 Decision records (ids from the reserved range; each written into `docs/DECISIONS.md` by the PR named)

| Id | Content | Written by |
| --- | --- | --- |
| `ARC-135` | The `wardrobe` System Pack: garments are Items with a `garment:` section; `Outfit` owned by `wardrobe`; slots, layering, privacy and the size chart are world configuration; actions, facts and refusals of §8; starting wardrobes from briefs (§10) | S32-a2 |
| `ARC-136` | `inventory` claims: a generic, clothing-ignorant concept of units claimed by a pack, excluded from carry capacity and from transfer, sale and consumption; refusal `in-use` (§8.3) | S32-a1 |
| `ARC-137` | Hairstyle is an S30 attribute changed by a service through a proposal; wigs and headwear are garments; S30's dress attributes split into preferences (S30) and worn state (`wardrobe.outfit`, derived) | S32-b |
| `ARC-138` | The modular character: a body + fitting template + parts on the 52 joints; the body route chosen by S32-p's rule (§5.2); the `hm08` template as the fitting space | S32-p |
| `ARC-139` | Perception and binding: the outfit disclosed with covered layers hidden; `look` stays the body (a dated note on `ARC-91` item 5); garment assets bound by category + colour with slot fallbacks; `character.yaml` `modular`/`baked` (amends `ARC-92/93`) | S32-c |
| `ARC-140` | The runtime garment format, fit at load, layer push-out and the runtime body mask (§§6.2–6.3) | S32-p (format), S32-c (runtime) |
| `ARC-141` | `ARC-94` amended for modular characters: G-gates per assembled outfit and the W-gates (§7) | S32-p |
| `ARC-142` | The fused route is retired for Persons in the release; it remains as tooling (route B's body), as Vera's interim control, and as `baked` for third-party packs (§5.3) | S32-h |
| `DEP-72` | MPFB2 / MakeHuman as a tool and data model (§4.5) | S32-p |
| `DEP-73` | Garment generation and fitting tools (§4.5) | S32-p |
| `DEP-74` | Rejected bases and generators, with reasons (§4.5) | S32-p |
| `DEP-75` | Godot-native modular runtime; no addon (§4.5) | S32-c |
| `DEP-76` | The 2D layer method (§§4.5, 11.2) | S32-e |

### 19.2 Proposed edits elsewhere (made by the PRs named)

- `step-26-cast.md`: the amendment of §15.2, once ruled. Only a pointer is written now.
- `step-30-person-attributes.md` / S30-c: `superseded_by: wardrobe.outfit` for the dress attributes.
- `step-31-wealth-prices.md` / S31: service sales (Q-S32-6).
- `step-28-shared-memory.md` / S28: the perceived-delta rule (S32-g).
- `MODULE_SPEC.md`:
  - the `garment:` and `wardrobe:` sections;
  - `configure/wardrobe.yaml`;
  - the 3D `garments/` directory;
  - the 2D `garment:`/`hair:` roles.
- `docs/HUMANOID_PROFILE.md`: parts share the 52 joints; imported bone names.
- `clients/2d/PRESENTATION.md`: layer stacks.
- `overall.md`: S32.

### 19.3 Local decisions

`SD-S32-1` ownership (§8.1) · `SD-S32-2` privacy (§8.4) · `SD-S32-3` duration and visibility (§8.4) ·
`SD-S32-4` sizes (§8.6) · `SD-S32-5` hair (§9.1) · `SD-S32-6` S30 overlap (§9.2) · `SD-S32-7` starting
wardrobes (§10) · `SD-S32-8` persistence (§13) · `SD-S32-9` the fused route (§5.3) · `SD-S32-10` fit at
load (§6.2) · `SD-S32-11` 2D layers (§11.2).

### 19.4 Findings carried

- **F-S32-5:** `loose`/`snug` are words, not geometry (§8.6).
- **F-S32-6:** hair growth is not modelled.
- **F-S32-7:** loose long hair and hems need spring bones (`SpringBoneSimulator3D`, Godot 4.5+), in a
  later step.


---

## 20. Questions for the operator

Each question carries a recommendation and, where money is involved, a cost estimate. Credit counts are
estimates until the pilot measures them. The money price per credit and per image depends on the
operator's plans and is **verify**.

1. **Q-S32-1 — The art route and its credits.**
   - *Recommendation:* adopt MPFB2's method (`hm08` template, `mixamo` rig = our 52 bones, `.mhclo`
     binding, MakeClothes) with new garments at Vera's tier. Pilot route B (Meshy body in a base layer +
     registered template) against route A (MPFB2 body + generated skin), on Vera, Ivan and Hana, and let
     §5.2's fixed rule choose.
   - Cancel the fused Meshy phase 2.
   - *Cost:*
     - pilot cap **≈ 1,200 credits**: 3 route-B bodies × ≈ 90, plus 16 garment/hair assets × 30 × ≈ 2
       runs ≈ 960, plus margin;
     - full build estimate **≈ 7,500 credits**: 22 bodies × 90 ≈ 2,000, plus ≈ 90 garments and hair × 30
       × ≈ 2 ≈ 5,400;
     - this replaces S26's 3,000 cap with **8,000 for S26 + S32 together**, reported at 50 % and 100 %;
     - route A would save ≈ 2,000 of it;
     - OpenAI: ≈ 320 images (garment references ≈ 270, base-layer person images ≈ 44) at the per-image
       price S26-b1 reads.
2. **Q-S32-2 — The 2D approach.**
   - *Recommendation:* 2D-3, layers **rendered from the 3D characters** with a stylizing shader per body
     class (no image cost; alignment and same-person-in-both-clients by construction), reviewed beside
     the current 2D street.
   - *If you reject its look:* 2D-2, generated paper-doll layers, ≈ 1,700 images (≈ 6 classes × 70
     garments × 4 views) plus alignment repair, priced before any call.
3. **Q-S32-3 — Hairstyle: attribute or item?**
   - *Recommendation:* an **S30 attribute** (`core.hair-style`, `hair-length`, `hair-colour`), changed by a
     paid **haircut service** at a salon chair. A wig or a hat is a garment.
   - Hair is not owned, given or sold, so it is not an Item.
4. **Q-S32-4 — Privacy for changing clothes.**
   - *Recommendation:* realistic default.
     - Under-layers (top, legs, socks, under-layer) may be changed only at home or in a shop's changing
       room, refused `needs-privacy` elsewhere.
     - Outer layers, shoes, hats and accessories may be changed anywhere.
     - A change takes about a minute per garment and becomes visible to others when it is finished.
   - All of it is world data a world may relax.
5. **Q-S32-5 — How Vera is handled.**
   - *Recommendation:* Vera stays the accepted fused protagonist and the review control until her
     modular character (same reference, same face, her hoodie, jeans and backpack as library garments)
     passes parity **beside her**.
   - She is the first pilot subject.
   - The release ships only her modular character. Her fused model stays in the repository as the
     control, not as a Person.
6. **Q-S32-6 — Services in S31.**
   - *Recommendation:* S31 gains **service listings**, a priced kind with `service: true` that is never
     held, settled like a sale, so a haircut is paid through S31's asks and haggling. S31 owns the change.
   - *Alternative:* the haircut pays through a plain `money-transferred` owned by `economy` on the
     stylist's ask.
7. **Q-S32-7 — Spare garments: carried or at home?**
   - *Recommendation for MVP-0:* worn garments are not carried (claims, §8.3), and each person carries at
     most two spares within the six.
   - Home storage (a household holding at the home place) is a follow-up step, if you want people to own
     more than a few garments.
8. **Q-S32-8 — Release size.**
   - The modular cast is estimated at ≈ 140–265 MB added to `3d.pck` (§14.2), against the fused cast's
     ≈ 160–180 MB.
   - *Recommendation:* keep Q-S26-5's ruling (UASTC, else about 350 MB) and let the pilot measure. You
     are asked again only if the measured size exceeds it.
9. **Q-S32-9 — The size chart's source.**
   - *Recommendation:* EN 13402 as the default chart.
   - Either buy the standard's parts 1–3 for exact values (*cost:* a national standards body's price,
     **verify**; a few hundred euros for the set is typical), or use a cited free secondary table marked
     `secondary`.
   - Recommended: the free secondary table now, the standard only if a discrepancy matters.
10. **Q-S32-10 — Fit refusals on by default?**
    - *Recommendation:* yes in the default worlds (a two-size mismatch is refused `does-not-fit`; one size
      off is worn `loose` or `snug`), with `size_chart: none` available.
    - *Alternative:* off by default, sizes carried for flavour only.
11. **Q-S32-11 — The 3D world's economy.**
    - "Buy a shirt and see it in 3D" needs the 3D client's world to have shops.
    - *Recommendation:* compose `social-cafe` with `item`, `inventory`, `item-transfer`, `economy` and
      `wardrobe`, plus an outfitters shop on its `store` place, and re-baseline it once.
    - *Alternative:* point the slice at `market-town`.

## ARC-18 — Godot stays the reference renderer; a second renderer is a spike, not a migration

**Date** 2026-09-27 · **Relates to** `ARC-17`, `DEP-9`, `ENGINEERING_RULES.md` §§11–12 ·
**Status** spike authorised, outcome open

`ARC-17` raises a fair question: can Godot reach the fidelity of the reference plates at all? The
answer is that nothing in those plates is out of reach for Forward+ — PBR, subsurface scattering,
baked `LightmapGI` with reflection probes, volumetric fog, HDR tonemapping are all present. The
real cost is **character and asset production**, not the renderer's ceiling. The current character
does not resemble the reference because it was built from the wrong mesh, hair, garment and
materials, which is `ARC-17`'s finding and not a rendering limit.

Unreal's advantage is therefore not a higher ceiling but a **shorter path** to it: a mature
ecosystem for high-fidelity humans, groom, cloth, cinematic lighting and retargeting.

**Decided — do not migrate.**

1. **Godot remains the official reference renderer.** It is MIT, so MineWorld can stay permissively
   licensed, forkable and modifiable end to end. Unreal grants source access under the Epic EULA
   and is not permissive open source. For a project that ships as open infrastructure, the fully
   open reference client is not negotiable.
2. **A second renderer is an additional Presentation Adapter**, never a replacement, and this is
   an architecture test as much as a visual one. Presentation independence is a claim this project
   makes; if adding a high-fidelity client requires changes to kernel or System contracts, the
   claim is false and we want to discover that now, while the kernel is small. If the interfaces
   are clean, the new client is a client.
3. **A time-boxed Unreal spike is authorised** to answer, with both slices in front of the
   operator: connect to the existing server, project the same Alice, build a reference-quality
   small-town slice with an enterable café and a reference-faithful character, and report the
   actual engineering cost of reaching a given visual target in each engine.
4. The outcome is **open**: Godot is good enough and continues alone; or Unreal becomes the default
   high-fidelity 3D client with Godot retained as the open reference; or both are kept, which would
   be the strongest demonstration of the framework claim.

**Cost, recorded because it becomes a licensing question if the spike succeeds.** Unreal Engine is
source-available proprietary software under one agreement, the Unreal Engine End User License
Agreement, which supersedes the former Publishing and Creators EULAs. Epic runs **two** payment
regimes with **two distinct $1M thresholds**, and they must not be merged. A MineWorld Unreal client
relies on Engine Code at runtime and is licensed to third-party end users, so it is a **Royalty
Product**: no seat fees at any company revenue, and **5% of worldwide gross revenue attributable to
the product**, excluding the first **$1,000,000 lifetime per product**, quarters under $10,000, the
first $5M on the Oculus Store, and Epic Games Store and Fab revenue outright. Ports share one $1M
allowance. The rate falls to **3.5%** for a "Launch Everywhere with Epic Release" — Epic Games Store
release before or simultaneous with other stores on each platform, plus content, feature and
marketing parity, for products released on or after 2025-01-01, reverting to 5% on disqualification.
The separate **$1,850 per seat per year** subscription applies only to uses that are *not* Royalty
Products, and only once the corporate group passes $1M gross revenue over the **trailing twelve
months**. This is a summary and the EULA governs.

It does not affect MineWorld's own licence: the server, kernel, contracts and Godot clients are
unaffected, and an Unreal adapter would be a separately licensed deliverable. The binding constraint
is not the royalty but §5(a): **no Engine Code may appear in a public MineWorld repository**, Engine
Code goes only to same-version Epic licensees, public distribution of Engine Tools must go through
Fab or a fork of Epic's GitHub `UnrealEngine` network, and a Product embeds Licensed Technology only
in object code as an inseparable part. MineWorld-authored **runtime** modules containing no Engine
Code may be MIT; **editor tooling is at risk** under the Engine Tools definition and belongs outside
the public repository.

**Corrected 2026-09-27** by the phase-one spike. The paragraph above previously read *"Unreal is free
below $1M USD trailing-twelve-month revenue"*, which described the **seat** exception — measured on
company revenue over twelve months — and applied it to the **royalty** exemption, which is lifetime
and per product. Epic names the confusion itself: *"There are two $1 million thresholds and they
depend on what you make and how much you make."* The seat regime, the 3.5% figure, the additional
exclusions, the Ports rule and the source-distribution constraint were all absent. Evidence, quoted
sources and the access limitations behind them are in
[`references/UNREAL_ADAPTER_SPIKE.md`](references/UNREAL_ADAPTER_SPIKE.md) §10, which also records
that Epic's own CDN still serves a 2022 EULA PDF that a search will return first and that contains
none of these terms.

**What is not in scope.** No kernel, contract, World Pack, persistence or networking change. Those
carry over untouched and their being untouched is the point. Only client-side work is new: the
protocol binding, `Entity` → actor projection, input → `ActionIntent`, cameras, and the scene.

**If the spike requires a contract change to proceed, stop and report it.** That result is more
valuable than the slice.

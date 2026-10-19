## ARC-21 — The Unreal spike answered its question; the operator has paused it there

**Date** 2026-09-27 · **Closes phase one of** `ARC-18` · **Evidence**
[`references/UNREAL_ADAPTER_SPIKE.md`](references/UNREAL_ADAPTER_SPIKE.md)

`ARC-18` authorised a spike to answer two things. Phase one answered both, and one of them came
back differently from how the decision assumed.

**1. The architecture claim holds.** No kernel, contract, System, World Pack, persistence or
networking change is required to add a non-Godot client. This was not concluded from reading: a
283-line Python client using only the standard library — no engine, no WebSocket library, no SDK,
no MineWorld code, RFC 6455 spoken by hand — joined `worlds/social-cafe`, was told
`talk → too_far_away`, submitted `arrive`, watched the server recompute the affordance to
`available`, had `talk` accepted, and collected nine distinct refusal codes. Reproduced
independently before this entry was written. A grep of the contracts, kernel, server and systems
for renderer vocabulary returns seven hits, all in doc comments, all saying the concept is
excluded; none in a type, field, variant or function name.

**A load-bearing property nobody had recorded.** `CORE_CONCEPTS.md` §6.1 defines yaw by naming its
two reference axes — "from +y toward +x" — rather than as a rotation sense about the vertical.
That makes it handedness-free, so the conversion into a left-handed, Z-up, centimetre engine is one
axis swap and one factor of ten with **zero sign flips**, cleaner than Godot's. A later edit
"simplifying" it to "counter-clockwise about +z" would remain true for Godot and silently become a
trap for every left-handed client. It is now written down as load-bearing.

**2. Unreal's advantage is much smaller than the decision assumed, and the reason is licensing.**
Epic's asset ecosystem was the whole of the claimed shortcut, and none of it can enter an MIT
repository — see the `DEP-8` amendment above, which generalises past Unreal entirely. Two further
findings narrow it: Unreal's runtime import does not support skeletal meshes or animation, so a
rigged glTF character must be cooked in rather than installed as a pack, which cuts against
`CLAUDE.md` §1's extension model; and measured against `ARC-17`'s hard-fail list, MetaHuman
addresses six of nine categories well and **three not at all — outer-garment category, defining
garment structure, identity-defining accessories — which are exactly the three
`VIS-3D-GODOT-1` failed on.** It solves the half that was not the problem. Environment cost is
unchanged by engine: box geometry dressed in CC0 materials is the same Blender work either way.

**Decided by the operator: do not install Unreal; return the effort to Godot.** The remaining prize
is a better out-of-the-box lighting start, worth roughly a week, against a client whose output can
only leave the building inside a cooked build. `ARC-18` stands unamended in principle — Godot
remains the reference renderer, a second renderer would be an additional Presentation Adapter, and
the outcome of that question is still open. `VIS-3D-UE5-1` and `VIS-3D-AB-1` are parked, not
cancelled, and the spike document is the record that restarting is cheap.

**`ARC-18`'s cost summary was wrong and is corrected in place.** "Free below $1M revenue" describes
the *seat-subscription* exception, measured on company revenue over twelve months; the *royalty*
exemption is lifetime and per product. A client depending on Engine Code at runtime is a Royalty
Product and needs no seats at any company revenue. Epic names the confusion itself.

**Method note, recorded because it nearly produced a wrong answer.** The spike's first verdict
rested on a clause it had quoted correctly and which was live and current — and which governed only
pre-5.6 MetaHuman. Epic runs two licence regimes in parallel and the older one is still published.
The question to ask of a clause is not only *is this current?* but *which path does this govern?*
Epic's CDN also still serves a 2022 EULA that a search returns first and that contains none of the
words "Seat", "Launch Everywhere", "3.5%" or "Fab".

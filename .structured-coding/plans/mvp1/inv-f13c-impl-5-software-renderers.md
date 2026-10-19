# Investigation F-13c-impl-5 — why the 3D slice is slow or broken on the hosted runners

## Record — investigation only, not a fix (2026-10-18)

```text
Status:        FINDINGS RECORDED; the fixes of §5 are proposals for a separate PR, not authorised here
Owner:         the 3D lane (F-13c-impl-5 is assigned to it: pr-13c-nightly.md §12.2; DECISIONS ARC-149)
Author:        far-field polish design agent (task of 2026-10-18), worktree
               /Users/yuema137/mineworld-worktrees/design-ff-polish; experiments in a separate worktree
               /Users/yuema137/mineworld-worktrees/inv-f13c (detached at origin/main b85f2804, local probe
               file never committed)
Scope ruled:   operator 2026-10-18 (release scope): "find a root cause; this is not a fix PR"
```

**Why this file exists beside RL-e0b's design.** The far-field polish (RL-e0b, [`pr-rl-e0b-far-field-polish.md`](pr-rl-e0b-far-field-polish.md))
adds renders to the one 3D check that hosted runners can finish only slowly; its CI budget (§8.2 there)
depends on what is found here. F-13c-impl-5 is older and wider than RL-e0b, so its evidence lives in its own
record, which the fix PR will cite.

---

## 1. The symptoms, as reported

| Id | Symptom | Where recorded |
| --- | --- | --- |
| S-1 | The 3D slice renders fewer than 12 frames in 240 s under software renderers (Linux: Mesa lavapipe under Xvfb; Windows: D3D12 WARP), so SET-b's connected 3D settings tests and one QC-5b test are skipped there | `DECISIONS.md` ARC-149 cause 3; `scripts/ci_godot.py` `SKIPS`, `HOSTED_UNRUNNABLE` |
| S-2 | On a cold macOS runner the first 3D launch takes 58 s to build the scene (later launches 3.3–3.7 s) and the first connected test hung | ARC-149 cause 3 (dispatch 39007080793) |
| S-3 | The macOS runner logs a Metal error, `sampler index out of bounds 0–15`, on Godot's standard material shaders | nightly logs (§3.1) |
| S-4 | The Windows far-field check ran 51+ minutes with no output after Godot fell back to D3D12 WARP | RL-e0 ledger F-FF-CI-3 (scratch nightly 39099056342) |

---

## 2. Method

1. Read the hosted logs of the main nightly 39134198789 (`clients-mac` job 117695318192, `clients-linux`
   117695318169, `clients-windows` 117695318294), of RL-e0's dispatched nightly 39106087666
   (`farfield-windows` job 117613487069) and of PR CI 39003755921 (`farfield`, Linux, job 117315006669).
2. Read Godot 4.7.2-stable's Metal driver sources (`drivers/metal/metal_device_properties.{h,cpp}`,
   `metal_device_profile.cpp`, `rendering_shader_container_metal.cpp`, tag `4.7-stable`, via `gh api`).
3. Reproduce locally, through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`:
   (a) on the Mac's own GPU with and without `GODOT_MTL_DISABLE_ARGUMENT_BUFFERS=1` (Godot's switch that
   forces the slot-binding path a GPU without argument-buffer tier 2 takes), each with a cold user directory
   (`HOME` redirected, so no shader or pipeline cache); (b) in a Docker container (Ubuntu 24.04, arm64,
   Mesa lavapipe under Xvfb, `--cpus 4` like the 4-vCPU hosted runner, the same pinned Godot 4.7.2).
4. Instrument: a throw-away probe mode (`--slice-f13c`) that times frames of the far-field check's own
   1 440 × 240 SubViewport and of the main 1 600 × 900 viewport, switching one cost at a time (§4).

---

## 3. Findings from the hosted logs and the engine source

### 3.1 S-3 — the Metal error is a real shader compile failure: anisotropic + repeat sampler at slot 17

**Evidence.** Main nightly 39134198789, `clients-mac` (job 117695318192): **784** lines
`error: 'sampler' attribute parameter is out of bounds: must be between 0 and 15`, each from
`Error compiling shader SceneForwardClusteredShaderRD` (`drivers/metal/metal_objects_shared.cpp:794`). Every
one of the 784 failing entry points declares exactly
`sampler SAMPLER_LINEAR_WITH_MIPMAPS_ANISOTROPIC_REPEAT [[sampler(17)]]`. The other material samplers in the
same log sit at slots 1, 6, 7, 9 and compile. The errors fall entirely inside the three connected 3D settings
tests (15:58 → 16:10), roughly 250 per test, spread over minutes — every launch compiles them again; a failed
compile is not cached.

**Mechanism (Godot 4.7.2 source).** The material samplers are bound at slots `6 + filter + 6·repeat`
(nearest, linear, nearest-mip, linear-mip, nearest-mip-aniso, linear-mip-aniso; clamp then repeat), so the two
anisotropic **repeat** samplers land at **16 and 17**. With argument buffers those slots are indices in a
buffer and are fine. Godot uses argument buffers only when
`argument_buffers_enabled() = use_argument_buffers && argument_buffers_tier == Tier2 && !needs_arg_encoders`,
and `needs_arg_encoders = !(supportsFamily(Metal3) && tier == Tier2)` (`metal_device_properties.cpp:152-178`,
`.h:113-136`). The hosted runner's **Apple Paravirtual** device does not meet that, so shaders are compiled for
**slot binding** (`rendering_shader_container_metal.cpp:387-395`), where Metal allows samplers 0–15 only.
Slot 17 is then a compile error.

**Which materials.** Everything built with `TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC` and repeating
UVs: `scripts/mats.gd:63` (`Mats.pbr`, the triplanar PBR set: paving, walls, roofs, grass), `:149`,
`scripts/slice/palette.gd:193`, `scripts/slice/profile.gd:47`, `shaders/hair_card.gdshader:14-15`. On the
hosted Mac those pipelines do not exist: **the slice's main surfaces are not drawn there**, and every check
that judges frames on hosted macOS judges a different picture from the one players see.

**Local reproduction: NOT RUN.** `GODOT_MTL_DISABLE_ARGUMENT_BUFFERS=1` on the Apple GPU forces the same
slot-binding path and should reproduce the error; the run was queued behind `with-slot.sh` and cancelled
under the operator's speed-first rule of 2026-10-18. It is the fix PR's first validation step (§5, P-1).

### 3.2 S-2 — the cold macOS launch is shader compilation, made worse by S-3

ARC-149 already measured it: the first launch of the slice on a runner built its scene in 58.1 s, later ones
in 3.3–3.7 s; the unjudged warm-up launch it added took **373 s** on 39134198789 (406 s on 39012891928). A
cold runner has no Metal pipeline cache, and the paravirtual GPU compiles every Forward+ material variant
before its first frame. S-3 adds to it: the failing anisotropic variants are compiled again, and fail again,
on every launch (§3.1's ~250 errors per test). **Root cause: cold pipeline compilation, plus repeated failing
compiles that can never be cached.** The connected test that hung (S-2) was the first launch of the night in
each recorded case (ARC-149 item 3), the one that pays the cold compile inside its 240 s deadline.

### 3.3 S-4 — "no output" is the test harness, not Godot: the Windows run was not observed hanging

`tools/cli/tests/client_far_field.rs` `godot()` reads Godot's stdout in a spawned thread and echoes each
line with `eprintln!("[godot] …")`. libtest's output capture is inherited by threads a test spawns, so those
lines are **held until the test ends** and dropped when it passes. Evidence: RL-e0's passing Windows run
(`farfield-windows`, job 117613487069) printed Godot's own inherited stderr at 07:31:42–45 (Vulkan absent →
D3D12; dummy audio) and then **nothing for 40 minutes** until `test … ok` at 08:12:06 (2 426.9 s) — zero
`[godot]` lines in the whole log. A cancelled run therefore shows exactly F-FF-CI-3's picture whatever Godot
was doing. The 51+ min there against 40.4 min here and 29.3 min earlier (E-FF-CI-2) is run-to-run variance of
the WARP runner, not a demonstrated hang; it cannot be told apart without streamed output (§5, P-4).

The Windows sequence itself is deterministic: Godot asks Vulkan first, the runner has no
`VK_KHR_surface`, and it falls back to D3D12 on the Microsoft Basic Render Driver (WARP), a CPU rasteriser.

### 3.4 S-1 — software renderers: about 5 s per frame for a 1 440 × 240 view

**Measured on hosted runners** (RL-e0's 3D check, shadows, SSAO and glow already off, GI none, the main
viewport's 3D disabled; each render waits 3 process frames and one `frame_post_draw`):

| Runner | 120 renders | per render | per frame |
| --- | --- | --- | --- |
| Linux lavapipe (job 117315006669, 05:47:08 → 06:24:45) | 37.6 min | 18.8 s | ≈ 4.7 s |
| Windows WARP (job 117613487069) | 40.4 min | 20.2 s | ≈ 5.1 s |
| Apple M-series, local (RL-e0 C6) | 18 s | 0.15 s | ≈ 0.04 s |

A 345 600-pixel frame at ≈ 5 s is far beyond what fill alone costs a CPU rasteriser for unshaded or simple
surfaces, so the cost is per-pixel shader work or per-draw overhead. The audit points at the shader: every
`Mats.pbr` material is **world-triplanar** (`mats.gd:59-60`) with albedo, ORM and normal maps, i.e. 9 texture
fetches per pixel, each **anisotropic ×4** (`project.godot` `anisotropic_filtering_level=4`) and trilinear —
on llvmpipe and WARP, anisotropic filtering is done in software per sample. The connected settings tests draw
the full look at 1 600 × 900 (directional shadows with four 2 048 splits and soft-filter quality 2, SSAO, glow,
with ≈ 6 times the pixels), which turns ≈ 5 s into well over 20 s per frame: fewer than 12 frames in 240 s, as
ARC-149 measured. **Root cause, with the evidence available: the software rasterisers pay the full desktop
material cost per pixel (triplanar × 3 maps × anisotropic ×4, then shadows and SSAO in the connected tests) at
full resolution.** The split between those terms is **not measured**: the instrumented container run
(lavapipe, 4 CPUs, one cost switched at a time, `f13c_probe.gd`) was prepared and cancelled under the
speed-first rule; it is the fix PR's first step (§5, P-2).

---

## 4. What is not established

- The relative weight of anisotropic filtering, triplanar fetches, shadows, SSAO and resolution in S-1
  (§3.4: one instrumented run decides it).
- Whether the Windows check ever truly hangs (§3.3: needs streamed output first).
- The local reproduction of S-3 with `GODOT_MTL_DISABLE_ARGUMENT_BUFFERS=1` (§3.1).

---

## 5. Proposed smallest fixes (for a separate PR; each with its check)

| Id | Fix | Addresses | How it is checked |
| --- | --- | --- | --- |
| **P-1** | One project-wide texture-filter choice, `Mats.FILTER`, used by `mats.gd`, `palette.gd`, `profile.gd` and the hair shader: anisotropic on a GPU that binds samplers through argument buffers, **`LINEAR_WITH_MIPMAPS`** when the client runs with `--aniso=off` (passed by every CI and harness launch) — not inferred from an adapter name. Alternatively, and simpler if the operator prefers: `TEXTURE_FILTER_LINEAR_WITH_MIPMAPS` everywhere with `anisotropic_filtering_level` left for a later setting | S-3 (no sampler past slot 11), part of S-2 (no failing recompiles), part of S-1 (no software anisotropy) | the nightly `clients-mac` log has zero `must be between 0 and 15` lines; locally `GODOT_MTL_DISABLE_ARGUMENT_BUFFERS=1 ./mineworld-slice --farfield` compiles clean |
| **P-2** | A render scale for the CI probes: harness modes on a software renderer set the main viewport's `scaling_3d_scale` to 0.5 (the far-field check's own SubViewport keeps its 1 440 × 240, whose angles are frozen) and keep RL-e0's cost-effects-off for every scripted 3D mode, not only the far-field check | S-1 for the connected settings tests (≈ 4× fewer pixels, no shadows or SSAO) | the three skipped SET-b tests pass on the Linux leg within their 240 s; then their `SKIPS` entries are removed (a ruling, QC-5) |
| **P-3** | Cache Godot's Metal shader/pipeline cache between nights: `~/Library/Application Support/MineWorld/shader_cache` (the project's `custom_user_dir`) through the same `actions/cache` restore/save pair as the import cache (ARC-149 item 4), keyed by OS, Godot build and `hashFiles('clients/**/*.gdshader', 'clients/3d-spike/scripts/**')` | S-2 | the warm-up note's time drops from ≈ 373 s on a cache hit |
| **P-4** | Stream Godot's output in the Rust harnesses: print through `std::io::stderr().write_all` from a thread that is not captured (or run the 3D check with `--nocapture` in `ci_layer.py`), and add a heartbeat line per viewpoint in the far-field check | S-4 (a hang becomes visible) | a dispatched `farfield-windows` log shows `[godot] [farfield] …` lines as they happen |
| **P-5** (only if P-1 and P-2 leave S-1 short) | Run the Linux and Windows probes with lavapipe's thread count matched to the runner (`LP_NUM_THREADS=4`) and record it; on Windows, try Mesa's lavapipe for Windows (or `--rendering-driver vulkan` with a Vulkan ICD) instead of WARP | S-1, S-4 | per-render time in the check's log |

**Order:** P-4 first (it costs nothing and makes every later run observable), then P-1 (it fixes a real
rendering defect on macOS, not only time), then P-2 with the instrumented run of §3.4 deciding whether
P-5 is needed; P-3 independently. None of them changes a criterion, a viewpoint or a frozen number of the
far-field check.

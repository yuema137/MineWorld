# PR S33-j — Lakeside ships as its own file: `3d-lakeside.pck`, the size gate, the perf session, the walk

## PR DESIGN (DRAFT) — not frozen

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874. Read-only: R-a
                        (#146, origin/mvp0/pr-s23-ra @ 57b45f52)
Approved by / evidence: none yet. Q-S33-3 (PK-2) and S33-c's refusal bind and are not re-asked; §14 is open
Implementation base:    origin/main at C0
Start condition:        R-a merged (`scripts/package.py`, export presets); S32-h merged (the cast's packing);
                        S33-c, S33-d, S33-e1, S33-e2, S33-f, S33-g merged (everything Lakeside ships); §14 ruled;
                        this design frozen; the lane lead's dispatch. NONE holds at 2026-10-18
Execution contract:     §12
Implementer:            Haiku-class from this commit plan (step-33 §12.1); the perf session and the walk are the
                        operator's
Lifecycle:              PR DESIGN (DRAFT)
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§1 (outcomes
7, 8), 8.1, 8.2 (PK-2), 9.4 (X-5), 11 (AC-S33-4, AC-S33-5, AC-S33-9, AC-S33-11), 12.1 (the S33-j row), 15
(`ARC-146`) · **Consumes:** [`pr-s33c-world-seam.md`](pr-s33c-world-seam.md) §§5.3, 5.8 (the registry's
convention, the refusal, `ARC-146`'s first half), R-a (step-23 §7: limits, `package.py`) · **Binding parents:**
`CLAUDE.md` §§1 (removability), 4 (items 9, 10); operator memories "All platforms", "Local load: CI is the gate",
"Milestone acceptance handoff".

**Decision ids.** `ARC-146` (step-assigned; this PR writes its export half). `ARC-176` is reserved for Q-S33j-1's
outcome (how a per-world export is declared, so later worlds follow without editing `package.py`), written by C1
only if Q-S33j-1 is ruled A. `DEP-97` is reserved and **not expected**: Godot's own export is used; it is written
only if C0 finds that a per-world pack needs a tool beyond the engine. No other id.

---

## 0. Binding inputs, exactly restated

- **Scope (step §12.1, S33-j row).** "packaging per Q-S33-3 (PK-2: per-world export in `scripts/package.py`); the
  size gate; the operator's perf session; the hands-on checklist item". Depends on R-a, S32-h, S33-d…g.
  Checkpoint: "AC-S33-4, -5, -9; X-5".
- **PK-2 (ruled).** "one Presentation Pack, **two export files**: `3d.pck` (the client, shared assets, the cast,
  social-cafe) and `3d-lakeside.pck` (Lakeside's builders, height grid, sit points and its own textures), loaded at
  start with `ProjectSettings.load_resource_pack` when `--world-key=lakeside`"; without the file the client
  "refuses by name (`PRESENTATION_WORLD_MISSING { world: lakeside }`, exit 3 in a scripted mode) … There is no
  fallback depiction."
- **S33-c §5.3.** The registry looks for `res://scripts/worlds/<key>/depiction.gd`; if absent, it loads
  `3d-<key>.pck` from beside the executable; R-S33j-c-1: "The export puts `scripts/worlds/lakeside/` and its assets
  into `3d-lakeside.pck` and removes them from `3d.pck`; the registry needs no change."
- **Size (step §8.2).** "Target ≤ 12 MB, gate ≤ 16 MB, measured by R-a's export." AC-S33-9: "`3d-lakeside.pck` ≤ 16
  MB; the release within the ruled `3d.pck` limit".
- **Removal (AC-S33-4, X-5).** "headless `lakeside` digest unchanged with and without the file; 2D `lakeside` runs;
  3D `social-cafe` runs and passes; 3D `lakeside` refuses by name".
- **Perf (AC-S33-5).** "Every Lakeside perf view p95 ≤ 16.7 ms and video memory ≤ 2 048 MB on the operator's M5
  (three runs); counts recorded." R-LOAD-1: local timings are INCONCLUSIVE.
- **The walk (AC-S33-11).** "enter at Elin's seat, walk to the pier head, look back at the village, walk to the halt
  and read the board, enter the bakery, sit on the pier bench".

---

## 1. Goal

The release carries Lakeside's 3D look as one separate file. With the file, `--world-key=lakeside` draws the
village; without it, everything else still runs and Lakeside's 3D client says by name what is missing. The file
fits its budget on all three platforms, the operator measures the eight views on their machine, and walks the
village once by a short checklist.

---

## 2. Scope

### 2.1 In scope

1. `clients/3d-spike/export_presets.cfg`: for each platform preset R-a defines (macOS, Linux, Windows), a sibling
   `world-lakeside` preset with `export_filter="resources"` that lists `res://scripts/worlds/lakeside/` and the
   Lakeside-only assets (the height grid, the sit-point depictions, any CC0 set S33-d adopted for Lakeside only);
   the main presets gain the same paths in `exclude_filter`.
2. `scripts/package.py`: per-world exports (Q-S33j-1 A: from a declared list, §5.1): `godot --headless --path … 
   --export-pack world-<key> runtime/clients/3d-<key>.pck`; each file read by `read_pck` and gated by its own limit
   (§5.2); `check_contents` over every `.pck`.
3. `tools/cli/tests/lakeside_removal.rs` (new): X-5 / AC-S33-4 (§5.3).
4. `docs/MVP_STATUS.md` or the release checklist (whichever R-a uses): the Lakeside size row with measured numbers.
5. `docs/HUMAN_REVIEW_QUEUE.md`: `VIS-3D-LAKESIDE` hands-on item (AC-S33-11) and the perf-session item (AC-S33-5).
6. `docs/DECISIONS.md` (or `docs/decisions/`): `ARC-146` (export half), `ARC-176` per Q-S33j-1.
7. `docs/MODULE_SPEC.md` §6: the sentence S33-c wrote is completed with the export's name (if S33-c's text left a
   placeholder).

### 2.2 Not in scope

The cast's packing (S32-h / S26-f); a separate Presentation Pack (PK-3, 16f); any loader change (S33-c's registry is
final); fixing a perf failure (a material stop with the levers measured, step §8.1).

---

## 3. Audit (`origin/main @ a1e05874`; R-a `@ 57b45f52`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | `scripts/package.py` exists only on R-a (#146, open): `MB = 1_000_000`; `LIMITS = {"bundle": (400 MB, 250 MB), "3d": (250 MB, 150 MB), "2d": (40 MB, 20 MB)}`; `export()` runs `godot --headless --path <project> --export-release <preset> <dest>`; `has_preset`; `read_pck`; `check_contents` over every `.pck`; `budget()` | `pr-s23-ra:scripts/package.py:49, 108, 140, 216–278, 406–460` |
| A-2 | The registry's convention and refusal are S33-c's (`3d-<key>.pck` beside the executable; `PRESENTATION_WORLD_MISSING`, exit 3) | `pr-s33c-world-seam.md` §5.3 |
| A-3 | Godot 4's command line has `--export-pack <preset> <path>` (a `.pck` only) and presets carry `export_filter` (`all_resources`, `resources`, `scenes`, `exclude`) with `include_filter`/`exclude_filter` | Godot 4 docs, "Exporting projects" and "Command line tutorial" (C0 re-reads for 4.7.2 and records the URLs) |
| A-4 | The 3D size is over its limit already (≈ 259.5 MB measured, ruled levers O-1/O-3/O-5, else O-4 ≈ 350 MB) | step A-13 |
| A-5 | `SlicePerf` (`--perf`): settle 6, warm 120, measure 300 frames, 1920 × 1080, gi=voxel; binding p95 ≤ 16.7 ms, video memory ≤ 2 048 MB | step A-12 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33j-1** | **`package.py` would learn world keys.** Writing `"lakeside"` into it would make every later world an edit of the packaging script (the change-amplification test, `CLAUDE.md` §4 item 5) | Q-S33j-1: the per-world exports are derived from the presets themselves (every preset named `world-<key>`), so a new world adds a preset and its files, never a line of Python |
| **F-S33j-2** | **Removal must be tested on an exported build**, not in the editor, because in the dev tree both depictions are always present (S33-c F-S33c-3) | AC-S33-4's 3D rows run on R-a's exported client with and without `3d-lakeside.pck`; the headless digest rows run on the server binary |
| **F-S33j-3** | **The perf numbers are the operator's.** Local timings are INCONCLUSIVE (R-LOAD-1) | the PR records counts; the operator runs the three-run protocol on the M5 and the results go into the ledger |

### 3.3 What does not exist

No per-world export, no Lakeside size row, no removal test, no Lakeside hands-on item.

---

## 4. Reuse, both ways

| Need | Decision | Why |
| --- | --- | --- |
| A per-world file | **Godot's `--export-pack` with a filtered preset** | the engine's mechanism; declined: a hand-written archive or a GDExtension loader |
| Gate and contents | R-a's `read_pck`, `check_contents`, `budget` | one packaging tool |
| Perf | `SlicePerf` and RL-b's three-run protocol | one method |

No dependency is added.

---

## 5. Design

### 5.1 Declaring a world export (Q-S33j-1)

`package.py` lists the presets in `export_presets.cfg` whose name matches `world-<key>` for the current platform
(the preset's `platform=`). For each, it exports `3d-<key>.pck` beside the client. It names no world. The main
preset's `exclude_filter` must cover exactly each world preset's files; a check fails when a file is in both or in
neither (M-j-1).

### 5.2 Limits

`LIMITS["3d-world"] = Limit(16 MB, 12 MB)` per world file (step §8.2's gate and target), applied to each
`3d-<key>.pck`; `3d.pck`'s limit is unchanged (its ruling is Q-S26-5's / Q-S32-8's). The bundle's limit includes the
world files.

### 5.3 The removal test (AC-S33-4, X-5)

`lakeside_removal.rs` on the packaged bundle (`#[ignore]`, nightly, three OSes):

1. `mineworld run worlds/lakeside 30d --seed 7` digest with the bundle complete, and with `3d-lakeside.pck`
   removed: equal (the server never reads the file).
2. 2D `lakeside` (`mineworld-2d --world-key=lakeside --drive`): passes, with and without the file.
3. 3D `social-cafe` (`--world --link`): passes without the file.
4. 3D `lakeside` without the file: exit 3, stdout contains `PRESENTATION_WORLD_MISSING: { world: lakeside }`.
5. 3D `lakeside` with the file: `--link` passes.

### 5.4 The perf session and the walk (for the operator)

Added to `HUMAN_REVIEW_QUEUE.md`, each a short, runnable list (operator memory "Milestone acceptance handoff"):

- **Perf:** `./mineworld-slice --world=lakeside --perf` three times on the M5 (RL-b's protocol); the eight views of
  step §8.1; record p95 and video memory per view.
- **The walk (AC-S33-11):** `./mineworld-slice --world=lakeside` → you are Elin in the cottages' yard → walk to the
  pier head → turn round: the village across the water → walk to the halt: the board reads "No service yet" → enter
  the bakery → go back to the pier and sit on the bench.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-j-1 (I-S33-1) | No kernel, contract, System, server or world change | diff scan |
| I-j-2 | No world key in `package.py` | a scan of the file for `worlds/*` stems |
| I-j-3 | Every Lakeside-only file is in exactly one `.pck` | M-j-1's check |
| I-j-4 | No fallback depiction | AC-j-2 row 4 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Packaging | sizes, contents, the filter partition | `scripts/package.py` (R-a's CI job), three OSes |
| Removal | AC-S33-4 | `lakeside_removal.rs` (`#[ignore]`), nightly, three OSes |
| Perf | AC-S33-5 | the operator's session; counts in CI logs |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-j-1 | `3d-lakeside.pck` ≤ 16 MB on macOS, Linux and Windows exports; `3d.pck` within its ruled limit | AC-S33-9 |
| AC-j-2 | §5.3's five rows pass on three OSes | AC-S33-4, X-5 |
| AC-j-3 | M-j-1 (a Lakeside file dropped from the main preset's `exclude_filter`) → the partition check fails naming the file | I-j-3 |
| AC-j-4 | The operator's three perf runs: every Lakeside view p95 ≤ 16.7 ms, video memory ≤ 2 048 MB | AC-S33-5 |
| AC-j-5 | The operator's walk recorded in `HUMAN_REVIEW_QUEUE.md` | AC-S33-11 |

---

## 9. Commit plan

Procedure: [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) §9.0, from
`/Users/yuema137/mineworld-worktrees/impl-s33j`; exports through `cast-slots/with-slot.sh`, one at a time.

- **C0 — Base, prerequisites, ledger.** R-a, S32-h, S33-c…g merged; Godot 4.7.2's export flags re-read (A-3).
  `docs(plan): S33-j — open the ledger at the base`.
- **C1 — Records.** `ARC-146` (export half), `ARC-176` per Q-S33j-1; MODULE_SPEC §6. `docs: ARC-146 — a world's 3D
  look ships as its own file (S33-j)`.
- **C2 — Presets.** §2.1 item 1. `build(3d): a world-lakeside export preset per platform (S33-j)`.
- **C3 — `package.py`.** §5.1, §5.2; AC-j-1 on the host platform; AC-j-3 (M-j-1). `build: per-world 3D exports and
  their size gate (S33-j)`.
- **C4 — The removal test.** §5.3; AC-j-2 on the host. `test(release): Lakeside's 3D file is removable (X-5; S33-j)`.
- **C5 — The operator's items.** §5.4. `docs: the Lakeside perf session and walk for the operator (S33-j)`.
- **C6 — Close-out.** Push; PR with `full-ci` (this PR touches the build and completes ACs: the charter's
  exception); CI on the exact head; repair; train-ready; the two operator items handed over. `docs(plan): S33-j
  close-out`.

---

## 10. Requirements this PR places on other PRs

- **R-Ra-j-1 (on R-a).** `export()` accepts `--export-pack` for a preset as well as `--export-release`.
- **R-S33dfg-j-1 (on S33-d, S33-f, S33-g).** Lakeside-only files live under `scripts/worlds/lakeside/` or a
  `lakeside`-named presentation path, so the preset's filter is a short list.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-23-*` (R-a) §7 | the per-world file's limit (16 MB gate, 12 MB target) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-j — Lakeside's export, size gate, perf session and walk
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33j-release-perf.md
WORKTREE / BRANCH:       /Users/yuema137/mineworld-worktrees/impl-s33j, mvp0/pr-s33j, one session
START CONDITION:         per the header (NONE met at 2026-10-18)
APPROVED SCOPE:          §2.1 items 1–7
NEVER:                   a world key in package.py; a fallback depiction; a local timing as evidence; as pr-s33d §12
MATERIAL STOPS:          `3d-lakeside.pck` over 16 MB with S33-d's levers spent; a perf view over 16.7 ms on the
                         operator's runs; the engine unable to filter a preset
PLATFORMS:               exports and the removal test on macOS, Linux, Windows (full-ci)
ENDPOINT AUTHORITY:      implementation, commits, push, PR, CI repair: authorized once started;
                         merge: through a merge train under the charter, after AC-j-4 and AC-j-5 are recorded
STOP CONDITION:          train-ready with the operator's items handed over
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-j-1 | Shared assets (a CC0 set used by both worlds) end up in both files | the partition check (I-j-3); a shared file stays in `3d.pck` |
| R-j-2 | R-a changes shape before merging | this PR starts after R-a merges and re-reads it at C0 |
| R-j-3 | A view fails 16.7 ms on the operator's machine | a material stop with the levers measured (step §8.1); the fix goes to S33-d/e/f/g |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33j-1 [operator — how a world's 3D file is declared]** | `package.py` must export `3d-lakeside.pck`. (A) Derive per-world exports from export presets named `world-<key>`, so a new world adds a preset, never Python; or (B) list `lakeside` in `package.py` | **(A).** A later world is then a module plus its preset, not an edit of the release tool (`CLAUDE.md` §4 item 5). *Chinese:* 每个世界的 3D 文件由名为 world-<key> 的导出预设自动导出，打包脚本里不写世界名——同意吗？ |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C6 | — | [ ] | [ ] | [ ] |

Mutations: M-j-1, X-5 (none run yet). Findings: F-S33j-1…F-S33j-3.

**Design review (2026-10-18).** One independent Opus reviewer, verdict REQUEST CHANGES overall, no finding on this
file; R-a's anchors confirmed.

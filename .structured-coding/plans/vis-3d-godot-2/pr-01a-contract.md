# Execution contract — PR 01a, `VIS-3D-GODOT-2`

Filled from the structured-coding template. The reusable working rules and test rules in
`.claude/skills/structured-coding/prompts/` are the body of this contract and are not restated
here.

| Field | Value |
| --- | --- |
| **Project / PR** | MineWorld · `vis-3d-godot-2` · PR 01a |
| **Repository / worktree** | `/Users/yuema137/MineWorld`, executed in a dedicated git worktree under the session scratchpad |
| **Branch / base** | `vis/3d-godot-2-environment` off `main` @ `5b99840` |
| **Primary design** | [`pr-01a-slice.md`](pr-01a-slice.md) |
| **Binding parents** | `CLAUDE.md`; `docs/VISUAL_SLICE.md`; `docs/VISUAL_FIDELITY.md`; `docs/ACCEPTANCE.md`; `docs/ENGINEERING_RULES.md`; `docs/ENGINEERING_STANDARDS.md`; `docs/DECISIONS.md` `ARC-11 ARC-13 ARC-17 ARC-18 ARC-20 ARC-21 DEP-8 DEP-9`; `clients/protocol/ADOPTION.md` |
| **Authorization source** | the operator brief of 2026-09-27 delivered to this session |
| **Commit authority** | **authorized**, and required after each meaningful completed step |
| **Branch publication** | **authorized** — push after each such commit |
| **Pull request** | may be opened; **merge is not authorized** and belongs to the operator |
| **Validation authority** | authorized: run the Godot client windowed and headless, run the MineWorld server locally, run `cargo` at workspace scope |
| **Network** | authorized for fetching CC0 assets from sources approved in `DEP-8`, and for verifying a licence at its primary source |
| **Spend** | none. No metered API, no paid asset, no subscription purchase |
| **Per-run limit** | 10 minutes for a scripted capture or drive run; 20 minutes for a lighting bake if one is attempted |
| **Total envelope** | bounded by the operator's brief rather than by a clock; retries count against it |
| **Endpoint** | `VIS-3D-GODOT-2 ENVIRONMENT: TECHNICALLY READY`, plus `WAITING FOR CHARACTER INTEGRATION` while the character has not landed. `READY FOR HUMAN VISUAL REVIEW` only after the character lands and the §12 views are captured |
| **Stop condition** | a change to a frozen invariant, a public contract, an ownership boundary, or scope; a required kernel/contract change (`ARC-18`: that result is more valuable than the slice); a licence that cannot be resolved |
| **Never** | mark `ACCEPTED`; merge; edit a file owned by `vis/3d-human-pipeline`; ask the operator about a matter listed as objective in the brief |
| **Synchronization owner** | this session owns the PR document; parent (`overall.md`) synchronization is also this session's, there being no separate planning session |

## Autonomy, restated for this PR

Everything objective is decided and executed without interruption: GI choice, probe placement,
collision fixes, lightmap UVs, import settings, texture formats, material wiring, shader bugs,
LOD, navigation, scene organisation, clean-checkout bugs, performance regressions, licence
checks, camera bugs, and every ordinary defect found on the way.

Everything subjective is the operator's and is neither decided nor polished past the point of
review readiness: whether the transition is beautiful, whether the street feels alive, whether
the scale feels pleasant, whether this is the default look.

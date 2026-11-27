# PR #529 local correction handoff — 2026-11-23

Primary authority: [R-b ledger](pr-s23-rb-release-workflow.md), especially §11 (frozen contract),
§13 (current narrowed authority, correction, proposals, evidence). Binding parent: step-23;
ARC-48/79/80, DEP-40/42. No competing design is created by this continuation aid.

Checkout `/tmp/mineworld-pr529-closeout-source`, branch `fix/pr529-closeout`, original PR head
`3c2acbe71e3028c10eb759e381db3cd9b87bc5f4`. Local dependency merge `b4d472bd` includes public main
`b457a94d9a1ace6a39005de683cc393886e19288` and the pre-code correction record. Next correction
commit contains workflow, DEP-42, offline evidence harness and this synchronized documentation.
Use `git log -2`, `git status` and `git diff b4d472bd` to recover exact candidate state; executable
content is `.github/workflows/release.yml` and `pr529-evidence/check_unpack.py`.

Endpoint authority: coordinator dispatch explicitly authorizes local code, cheap offline validation,
review and commits. Public push, CI dispatch, tags/releases, public merge, settings, Cargo/Godot,
child agents prohibited. Frozen invariants I-Rb-1…6 and native Unicode-path acceptance preserved.
The local public-main dependency merge was explicitly allowed. Root checkout untouched.

Current checkpoint: bounded implementation and offline validation complete; independent review and
remote exact-head acceptance pending. Local runtime <2 minutes across cheap checks; no paid compute,
API, Cargo or Godot; no background process. Missing expat symbol prevents ci_release self-test import;
classify INCONCLUSIVE, never pass. `check_unpack.py` validates two POSIX invocation seams and kills
the old -C mutation, not Windows execution. See ledger for all actual results and failed attempts.

Exact next actions: independent reviewer reviews correction and ledger at actual candidate SHA;
coordinator adopts/rejects #498 supersession and release green-check policy proposals, coordinates
#524 producer/consumer seam, integrates separately-owned F-RC-7 when publicly available; authorized
future session publishes intended final head and obtains all required exact-head gates. No old-head
run applies to this candidate. Do not mark READY FOR OPERATOR REVIEW under full release contract.

Stop conditions: material contract/public interface/dependency change needs coordinator decision;
current local-only endpoint stops with reviewable correction and explicit validation pending.
Coordinator owns parent overall/step updates and public body/closure actions. This session only
prepares local replacement body/closure payloads and reports evidence. No merge authority assumed.

# Nightlife N1 / N2 — implementation contracts (filled, frozen)

```text
Lifecycle:  FROZEN 2026-11-27 with pr-nightlife.md revision 2. The operator approved every recommendation of the
            design (Q-L1 … Q-L5, Q-L4 option A) on 2026-11-27: "同意三个都按推荐冻结", relayed by the primary
            coordinator. The endpoint lines below take their source from that ruling.
```

The generic bodies of `.claude/skills/structured-coding/prompts/implementation-working-rules.md` and
`prompts/test-ci-gate-rules.md` apply in full; these blocks specialise only their project section.

## Contract N1 — optional shop hours

```text
PROJECT / PR:
  MineWorld operator-playable MVP / PR N1 — optional economy shop hours and Market Town nights

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/pr-nightlife.md §§0–5, 7–9 (live ledger)

RELATED / BINDING DOCS:
  CLAUDE.md; docs/{VISION,CORE_CONCEPTS,ARCHITECTURE,MODULE_SPEC,ENGINEERING_RULES,ENGINEERING_STANDARDS,
  REUSE_POLICY}.md; operator-playable-mvp-20261123.md; step-35-needs-and-sleep.md (AC-S35-8, Q-S35-3, Q-S35-7);
  economy decisions ARC-112, ARC-118, ARC-28; .structured-coding/standards.md

IMPLEMENTATION BASE:
  origin/main at the session's start, at or after the freeze base 8c971f0f7b469f05d1bf6f21fc933501eb22c404;
  record the exact commit in the handoff and re-check the §2.2 seams if main has moved.
  N1-C1 and N1-C2 can start on main now; N1-C3's packaged Gate2 check waits for P1 (pr-nightlife.md §1.3)
  and is recorded INCONCLUSIVE — prerequisite until then; every other N1-C3 item runs on main.

APPROVED SCOPE:
  pr-nightlife.md §5 (N1-C1 … N1-C3) and invariants N1-I1 … N1-I8

FROZEN INVARIANTS:
  - N1-I1 … N1-I8 (pr-nightlife.md §3)
  - kernel, clients and controllers compute no business hours; no stored open flag, no Process, no scheduler
  - absent hours = today's behaviour; Social Café and Bodies Yard unchanged
  - S35 AC-S35-8 rhythm bounds are never relaxed to pass
  - forbidden: staffing, door locks, lights, private homes, weekday calendars

APPROVED IMPLEMENTATION SEQUENCE:
  N1-C1 → N1-C2 → N1-C3; integration checkpoint at C3; READY FOR OPERATOR REVIEW

AUTONOMOUS VALIDATION BUDGET:
  - owning-crate unit/integration/static tests: unrestricted; no local full-workspace suite (R-LOAD-1, CI gates)
  - real LLM calls: none (Gate1 N/A)
  - Gate2 packaged launcher night check: one shared heavy-job slot assigned by the primary, ≤ 10 min per run,
    ≤ 60 min total including retries
  - long records (Market Town baselines re-record + AC-S35-8, pr-nightlife.md N1-C3): once on the final head in
    the primary-assigned heavy slot, ≤ 60 min total; projection recorded before launch
  - monetary spend: $0
  - anything beyond: stop with a cost/runtime projection

REQUIRED LIVE DOCUMENTATION:
  pr-nightlife.md (N1 sections: checkboxes, evidence, deviations)

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff-nightlife-n1.md (created at initialization; never a second authority)

PR CONTEXT SCOPE / INITIALIZATION / AUTHORITY ORDER / CONTINUITY / COMPACTION / RESUME / CLOSEOUT:
  as the generic working rules state; fresh implementation session; one worktree under
  /Users/yuema137/mineworld-worktrees/ on branch feat/nightlife-n1-shop-hours, held by one session

ENDPOINT AUTHORITY:
  - implementation + local validation:  authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - semantic commits:                   authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - branch push:                        authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - PR creation / update:               authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - CI repair to review readiness:      authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - merge:                              explicit operator authorization only (code PR; CLAUDE.md §6)

POST-MERGE SYNCHRONIZATION OWNER:
  implementation session: pr-nightlife.md N1 evidence, merge identity, deviations;
  primary coordinator: operator-playable amendment, overall.md, progress page

NORMAL STOP CONDITION:
  PR N1 READY FOR OPERATOR REVIEW — DO NOT MERGE

STOP CONDITION:
  N1-C1…C3 complete + required validation recorded + independent review dispositioned + PR open
  + terminal CI green on exact final head (Gate2 may be INCONCLUSIVE only for the §1.3 prerequisite, stated so)

MATERIAL STOP:
  change to ownership, public contract (shop-stock meaning beyond N1-I6), frozen invariant or scope;
  S35 rhythm conflict that content cannot fix; persistence semantics unsupported; budget exceeded

MERGE AUTHORITY:
  NEVER merge without explicit operator approval.
```

## Contract N2 — sleep-until and single-player advance

```text
PROJECT / PR:
  MineWorld operator-playable MVP / PR N2 — sleep-until, configured morning, solo exact-due advance

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/pr-nightlife.md §§0–4, 6–9 (live ledger)

RELATED / BINDING DOCS:
  as N1, plus server/PROTOCOL.md §§4, 5.9, 11; clients/shared/HOST.md; ARC-25, ARC-27, ARC-40, ARC-42, ARC-44,
  ARC-67, ARC-69, ARC-267, ARC-268; step-35 (Q-S35-9, SD-Na-*); step-19 (INV-TW-*)

IMPLEMENTATION BASE:
  origin/main after PR N1 has merged; re-audit §2.2 seams and record the exact commit. N2-C1 … N2-C3 need
  nothing else; N2-C4 additionally waits for launcher prerequisites P1, P2 and P3 (pr-nightlife.md §1.3)

APPROVED SCOPE:
  pr-nightlife.md §6 (N2-C1 … N2-C4) and invariants N2-I1 … N2-I14 (incl. N2-I1b), with Q-L4 option A;
  includes the rule/motivated controllers' exclusion of sleep-until (N2-C1) and the ARC-69 item 5 amendment
  record (N2-C2); excludes tools/launch and tools/cli/src/hosted.rs

FROZEN INVARIANTS:
  - N2-I1 … N2-I14, incl. N2-I1b (pr-nightlife.md §3)
  - the server never submits an ActionIntent for a session-driven Person (cancel stops only)
  - kernel unchanged; server names no sleep type; admin routes name no instant
  - advance never a fact, input or revision; no rescale used as an advance; shared worlds never advance
  - ordinary tick_at semantics unchanged; oracle fixture frozen before the loop exists
  - forbidden: external/LM seats during advance (Q-L4 B), controller checkpoints, auto-resume after restart

APPROVED IMPLEMENTATION SEQUENCE:
  N2-C1 → N2-C2 → N2-C3 → N2-C4; integration checkpoint at C4; READY FOR OPERATOR REVIEW

AUTONOMOUS VALIDATION BUDGET:
  - owning-crate tests: unrestricted; no local full-workspace suite (R-LOAD-1)
  - real LLM calls: none (Gate1 N/A)
  - Gate2 (2D + 3D packaged sleep/advance/cancel/kill-reopen): one heavy slot from the primary,
    ≤ 10 min per run, ≤ 60 min total including retries; Godot builds count against the slot
  - long records (baselines re-record + AC-S35-8, N2-C1): once on the final head, heavy slot, ≤ 60 min total
  - N2-C3 speed acceptance measured in a release build of mineworld-server tests only
  - monetary spend: $0
  - anything beyond: stop with a projection

REQUIRED LIVE DOCUMENTATION:
  pr-nightlife.md (N2 sections)

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff-nightlife-n2.md

PR CONTEXT (all generic fields):
  as N1; branch feat/nightlife-n2-sleep-advance; fresh session; never the N1 context

ENDPOINT AUTHORITY:
  - implementation + local validation:  authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - semantic commits:                   authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - branch push:                        authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - PR creation / update:               authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - CI repair to review readiness:      authorized   source: operator freeze ruling 2026-11-27 ("同意三个都按推荐冻结")
  - merge:                              explicit operator authorization only

POST-MERGE SYNCHRONIZATION OWNER:
  as N1; the primary also adds the dated notes to step-35 and ARC-69 if N2-C1/C2 did not

NORMAL STOP CONDITION:
  PR N2 READY FOR OPERATOR REVIEW — DO NOT MERGE

STOP CONDITION:
  N2-C1…C4 complete + validation + independent red-team review dispositioned + PR open + CI green on exact head

MATERIAL STOP:
  oracle parity unattainable without changing ordinary tick semantics; need for a kernel change; need to
  persist controller state; any change to seat ownership or ARC-40/ARC-69 contracts beyond the recorded notes;
  budget exceeded

MERGE AUTHORITY:
  NEVER merge without explicit operator approval.
```

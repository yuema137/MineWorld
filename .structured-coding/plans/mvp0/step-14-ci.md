# Step 14 — Deployment parity and layered CI (S13)

**Role:** step document for **S13 — Deployment parity and layered CI**, with the first PR (13a) detailed
to the commit in §9 so that it can be frozen and implemented quickly. Later PRs (13b, 13c) are at medium
scope (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S13), §4 (`AC-8`, layered CI), §7 (the carried
`fetch-depth: 0` requirement).
**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing in this document authorizes
implementation. Decision records proposed here carry placeholder ids (`DEP-S13-a`, `ARC-S13-a`, …); the
primary session numbers them at freeze.
**Planning session:** worktree `/Users/yuema137/mineworld-worktrees/plan-s13-ci`, branch `plan/s13-ci`,
from `main @ 0fd0be3`. Written in parallel with the planning of S11, S12, S14 and Milestone E; this
session writes only this file (§12 lists the edits it proposes to parent documents).

**Operator directive carried into this design (2026-10-08, relayed by the primary session).** Keep code
clean, modular and pluggable; search for open-source solutions first; compare several implementations
and do not stop at the first; build our own only if nothing existing serves the goal. §5 is the
comparison for every infrastructure piece S13 introduces; §6 is what can be swapped without touching
anything else.

---

# 1. Requirement and acceptance, quoted

`docs/MVP.md` §9:

> **AC-8** — Local/cloud parity — The same World Pack runs on a laptop and inside a cloud Docker
> container with no semantic differences.

`overall.md` §3, S13:

> - **Output:** Dockerfile and container run; the four CI layers of `ENGINEERING_STANDARDS.md` §16 —
>   fast structural checks, core integration tests, scenario tests, long-running stability.
> - **Depends on:** S11. The repository now exists (`D-12`), so this step keeps the CI workflow files
>   and the container work only.
> - **Acceptance checkpoint:** the same World Pack runs on the laptop and in the container with no
>   semantic difference, demonstrated by identical seeded event sequences (`AC-8`); CI runs the layers
>   on the right triggers.

`docs/NETWORKING.md` §8:

> The same World Pack must run on a laptop and inside a cloud container with **no semantic
> differences**. This is an MVP acceptance criterion, not an aspiration. A behavior that appears only in
> one deployment is a defect in that deployment path.

`docs/ENGINEERING_STANDARDS.md` §15 (every pull request runs, at minimum, format checking,
compilation/type checking, linting, core integration tests, critical regression tests; for Rust
`cargo fmt --check`, `cargo check --workspace --all-targets`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`; "quality gates must remain
automatic") and §16 (the four layers):

> **Fast structural checks** — run on every change: format, lint, type/compile checks, schema
> validation.
> **Core integration tests** — run on every pull request: create world, start server, load systems,
> create entities, perform actions, observe events, persist, reload, continue simulation.
> **Scenario tests** — run realistic MineWorld scenarios: Social Café, Market Town.
> **Long-running stability tests** — can run separately from the fastest PR loop: simulate 100 days,
> simulate many agents, restart server repeatedly, replay event log.
> The principle does not change: at least some tests must exercise MineWorld as an actual running
> system.

Carried requirements:

- `overall.md` §7 (S9 closeout): **"S13 — CI must check out with `fetch-depth: 0`, or the AC-1 test
  fails (by design, never a skip)."** `tests/acceptance/tests/ac1_composability.rs:359–366` refuses a
  shallow clone by name and cites `fetch-depth: 0`; `precursor_vocabulary.rs:370–376` refuses a missing
  base the same way.
- `step-11-bodies.md` §12 / R-B7 / PB-12 / QP-16: cross-architecture float identity of the `bodies`
  pack (Rapier, `DEP-13`, `enhanced-determinism`) is evidenced only under Rosetta on the operator's
  Mac; **"a native x86_64 host (S13's container on such a host) remains the stronger check."**
- `overall.md` §3, "Cross-cutting": `tools/benchmark` is delivered "with S13". §8 places it.
- `overall.md` §5 `D-1`: the licence (MIT) is revisited "before publication in S13 if the patent-grant
  argument becomes material". QS13-2 touches publication; D-1 is raised there.

## 1.1 What S13 must make observable

```text
O-1  a container image built from a committed Dockerfile runs `mineworld` with a pinned toolchain
O-2  AC-8: the same World Pack, seed and age give identical event sequences on macOS arm64 (the laptop)
     and on Linux x86_64 inside that container — social-cafe, market-town and bodies-yard
O-3  CI runs the four §16 layers, each on its stated trigger, with full history (`fetch-depth: 0`)
O-4  a deliberately broken change turns CI red, at the layer that owns the failure
O-5  what blocks a merge and what only reports is written down, and true of the repository's settings
```

---

# 2. Audit (main @ 0fd0be3, 2026-10-08)

## 2.1 What exists

| Item | Finding | Evidence |
| --- | --- | --- |
| CI | **None.** No `.github/` directory, no workflow file; Actions has never run on this repository. | `ls .github` → no such directory; `gh api repos/yuema137/MineWorld/actions/workflows` → `total_count: 0`; `…/actions/runs` → `0` |
| Container | **None.** No `Dockerfile`, no `.dockerignore`, no compose file anywhere in the tree. | `find . -name 'Dockerfile*'` → nothing |
| Toolchain pin | `rust-toolchain.toml`: channel `1.97.1`, components `rustfmt`, `clippy`, profile `minimal`. Root `Cargo.toml`: `rust-version = "1.97.1"`, edition 2024. | `rust-toolchain.toml:1–8`, `Cargo.toml` `[workspace.package]` |
| Dev profile | `opt-level = 1`, debug assertions and overflow checks stated (`ARC-30`). Tests run in this profile. | `Cargo.toml` `[profile.dev]` |
| Declared checks | `.structured-coding/standards.md`: `cargo fmt --all --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `python3 scripts/check_decision_ids.py`, `python3 scripts/check_doc_headings.py`, `cargo test --workspace`; `ruff`, `pyright` disabled. "The gates that must actually block are CI's job." | `.structured-coding/standards.md` |
| Python | `scripts/check_decision_ids.py`, `scripts/check_doc_headings.py` — stdlib only. No Python package under `cognition/` (only `cognition/rule-controller`, a Rust crate). | `ls scripts cognition` |
| Workspace | 11 member globs incl. `systems/*`; `spike/server` is a separate Cargo project outside the workspace (`.gitignore`: `spike/server/target/`). | `Cargo.toml` `[workspace]` |
| `harness = false` programs | `persistence/tests/kill_and_resume.rs` (IC-1) and `tests/acceptance/tests/arrival_resolvers_resume.rs` (prints `[resolver-yard] … PASS`). Each re-executes itself as a child selected by an environment variable and SIGKILLs it. They run under `cargo test` as ordinary test targets. | `persistence/Cargo.toml:20–23`, `tests/acceptance/Cargo.toml:11–17` |
| Long runs in the default suite | `tools/cli/tests/run.rs` (three 300-day social-cafe runs in parallel), `market_town.rs` (a 300-day and four 30-day Market Town runs, one SIGKILLed), `bodies_yard.rs`, `bodies_yard_restart.rs`, `kernel/tests/long_run.rs`, `systems/bodies/tests/long_run.rs`. In the default suite by decision (step-10 QS-59: "the full 300 days, in the default suite"). | file headers |
| History-reading tests | `ac1_composability.rs` (`git log`, `git diff --name-only`, `git ls-tree`, `git show rev:path`, `rev-parse --is-shallow-repository`), `precursor_vocabulary.rs` and `seam_vocabulary.rs` (`git cat-file -e`, `merge-base --is-ancestor`, `git diff` text of a recorded range). All fail closed on a shallow clone; none skips. | `ac1_composability.rs:80–366, 469–494`; `precursor_vocabulary.rs:230, 352–394` |
| Tests that read the client tree | `tools/cli/tests/ac13_semantic_parity.rs:42` reads `clients/protocol/evidence`. Nothing reads `clients/3d-spike/` or `presentation/`. | `git grep "clients/\|presentation/" -- '*.rs'` |
| Run summary | `mineworld run <world> --headless --seed N --days D [--save DIR]` prints per-day lines, request/fact counts, activity buckets, `faults`, and `history N facts, fingerprint <FNV-1a 64>` over every fact's stored encoding, then `wall`. The fingerprint is "for reading; not evidence": tests compare stored bytes row by row (step-08 §10.1 Q9). | `tools/cli/src/run.rs:370–508` |
| Deterministic lines | Test helper `deterministic()` keeps every summary line but the `[mineworld] run …` header (it names where the world is kept) and `wall`. `Tables::read` reads `facts`, `journal`, `snapshots` row by row; `assert_same_history` compares them and the manifest's `pack` and `composition`, **not** the whole manifest. | `tools/cli/tests/headless/mod.rs:56–63, 159–244` |
| Save tables | `manifest(id, format, body)`, `journal(revision, at, action_id, entry)`, `facts(event_id, revision, fact)`, `snapshots(revision, snapshot)`. A save's creation records `WorldInstanceId::allocate()`, which mixes a process counter with the wall clock, so **a save file is never byte-identical across runs**; its fact, journal and snapshot rows are. | `persistence/src/sqlite.rs:46–66`; `tools/cli/src/run.rs:209–222`; `server/src/protocol.rs:393–400` |
| Pinned digests already in use | E-RS0 (step-11): sha-256 over every summary line but `wall` of the 300-day seed-7 in-memory runs: social-cafe `ad49c723…c64b` (365 330 facts), market-town `365b50e0…1d1d` (372 755 facts). 12d re-baselines both (step-11 §12); 12c changes bodies-yard's (QO-16). | `step-11-bodies.md:2456–2463, 4130, 4804` |
| Godot checks | `./mineworld-slice --drive` and `--world --link` run Godot `--headless` (physics, no pixels); `clients/protocol/run.sh evidence` regenerates the AC-13/AC-15 evidence headless against a real server. `slice_probe.gd:165` and `shots.gd:119` call `get_tree().quit(0)` unconditionally: **a failed drive still exits 0**. Godot 4.7.2 is installed on the operator's Mac (`/opt/homebrew/bin/godot`). | `mineworld-slice:60–92`; `clients/protocol/run.sh:1–40` |
| Hash crates | No SHA-2 implementation in `Cargo.lock` (only `sha1`, transitively, for the WebSocket handshake). | `grep '^name = "sha' Cargo.lock` |
| Docker on the operator's Mac | Docker Desktop client 29.7.2 (`darwin/arm64`, context `desktop-linux`) installed; **daemon not running** at audit time. No image built (planning only). | `docker version` |
| Spec text on deployment | `ARCHITECTURE.md` §11 (one binary; Local PC, Cloud VM, Docker, community server), §13 (local: native binary / Docker; cloud: "Docker first"); `NETWORKING.md` §7 (single VPS + Docker + persistent volume), §8 (parity); `MVP.md` §8 ("cloud: Docker deployment on a VPS"). | as cited |

## 2.2 The GitHub repository

Read-only `gh api` calls, 2026-10-08:

```text
repository        yuema137/MineWorld — private, owned by a User account, default branch main
size              501 774 KB reported by the API; locally the packed history is 464.8 MiB
                  (git count-objects -vH: size-pack 464.83 MiB, 6 743 objects); HEAD's tree
                  274.7 MB (git archive HEAD | wc -c), of which clients/ 195.0 MB and
                  presentation/ 72.1 MB; everything else under 4 MB. 458 commits on main.
                  Largest blobs: clients/3d-spike/assets/characters/meshy_d/meshy_d.glb 10.1 MB,
                  its base-colour PNG 6.1 MB, vitruvian.glb 4.6 MB, an HDRI 4.6 MB.
LFS               not used (has_lfs null; .gitattributes marks *.bin, *.glb binary only)
Actions           enabled; allowed_actions: all; sha_pinning_required: false
runners           0 self-hosted runners registered
branch protection GET …/branches/main/protection → HTTP 403 "Upgrade to GitHub Pro or make this
                  repository public to enable this feature."
rulesets          GET …/rulesets → the same 403
billing           not readable: the gh token's scopes are gist, read:org, repo; the billing
                  endpoints need `user`. The plan is inferred, not read: the 403s say the account
                  is on GitHub Free.
```

**Finding F-1 (material).** `main` is **not protected**, and on this plan it cannot be. `D-12` records
that "the operator is enabling branch protection on `main` … protection makes that a mechanism rather than
a promise". The API says the mechanism does not exist for a private repository on GitHub Free: neither
classic protection nor rulesets are available. Today "merge requires explicit operator authorization" is
a promise kept by every session, not a mechanism, and **a CI check cannot be made required**. §7 designs
around this honestly and QS13-1 puts the choice to the operator.

**Finding F-2.** The project has merged 61 PRs in about two weeks. Its Actions budget therefore matters
(§10.1).

## 2.3 The test suite's real shape

Measured in this worktree, `cargo test --workspace`, debug profile (opt-level 1), operator's Mac (Apple
Silicon), cold `target/`: see §2.3.1 (E-S13-0).

<!-- E-S13-0 is filled in below from /tmp/s13-plan/full-test.log -->

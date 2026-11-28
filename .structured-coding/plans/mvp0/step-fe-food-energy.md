# Step FE — Food energy (kcal) and the shopping/eating loop

**Role:** step document for the playable-MVP nutrition/economy requirement. The step label **FE** is
proposed; the primary assigns any number in [`overall.md`](overall.md). It does not reuse `S36` (taken in
`mvp1`) or the S35 PR letters (`N-a` … `N-h`).

**Effort:** `mvp0`. **Parent:** [`operator-playable-mvp-20261123.md`](operator-playable-mvp-20261123.md) §1, §2
rows "Initial USD 10,000", "Food energy", "Realistic prices", §4 items 3–5 and 8, §5 ("2000 kcal/day,
configurable"). **Builds on:** [`step-35-needs-and-sleep.md`](step-35-needs-and-sleep.md) (frozen) and
[`pr-s35-nb-needs.md`](pr-s35-nb-needs.md).

**Lifecycle:** `STEP DESIGN FROZEN — 2026-11-27` (operator approved in the primary session: every question in §1
answered with its recommendation). PR FE-a is DESIGN FROZEN with its contract; PR FE-b still needs its own design
and freeze. The red-team pre-freeze review (`/Users/yuema137/mineworld-worktrees/_reviews/design-fe.md`) raised
C-FE-2 and Q-FE-6, which the operator then ruled.

**Design session:** lane L1, worktree `/Users/yuema137/mineworld-worktrees/design-nutrition`, branch
`docs/pr-design-nutrition`, audit base `origin/main @ 8c971f0f` (first audit `7718d952`).

---

## 1. Decisions for the operator (plain language)

**Operator ruling, 2026-11-27 (relayed by the coordinator), quoted:** "同意三个都按推荐冻结" — approve the three
design PRs #535, #536, #537 as recommended. For this step that answers Q-FE-1 … Q-FE-5 **yes**. Already-approved
requirements are not reopened: **2000 kcal per world day, configurable** (operator, 2026-11-23) and **the new demo
player starts with USD 10,000** (operator, 2026-11-23).

| # | Question | Recommendation | Ruling |
| --- | --- | --- | --- |
| Q-FE-1 | **What the bar shows.** The HUD shows a *food-energy reserve* in kcal. It is full at one day's use (2000 kcal by default — the FDA Daily Value reference intake on US Nutrition Facts labels, 21 CFR 101.9(c)(9)) and drains at one constant rate, asleep or awake, with no midnight reset. The constant rate is a deliberate game simplification (real sleep burns somewhat less), and unlike today's unitless hunger it does not slow at night. Eating adds the item's kcal, capped at full. The existing *hunger* word ("hungry", "starving") stays, computed by the server from the same reserve, and is what makes residents go and eat. An empty reserve has no medical effect: no collapse, illness, slowdown or death (S35's exclusion stays). Accept? | **Yes.** | Yes, 2026-11-27 |
| Q-FE-2 | **Old saves.** The kcal model changes the `needs` save format (version 2). New saves only: an existing save keeps working with the build that made it, and the new build refuses it with an explicit error naming the incompatibility. No migration, no reset, no silent conversion of old hunger points into kcal. Accept? | **Yes.** | Yes, 2026-11-27 — see C-FE-2 |
| Q-FE-3 | **Three small framework changes outside `needs`.** (a) One domain-neutral kernel error, `InvalidGenesisContent`, for "this authored world is contradictory" (owner, entity, code, field). It knows nothing about food or money; it amends S35's "kernel unchanged" by one error type, recorded in a new ARC record. (b) The save file is created only *after* the world has started successfully, so a bad world leaves no empty save behind (today it does, for any authoring error). (c) An SDK hook letting a System Pack declare its own schema version and reject contradictory configuration with the file and entry named. Accept? | **Yes.** | Yes, 2026-11-27 |
| Q-FE-4 | **Two checkpoints.** First the engine PR **FE-a** (this step's only frozen PR): the kcal model, the three framework changes, both clients and agent views, proven on *synthetic* test worlds. Then a separate content PR **FE-b** for Market Town: dated US retail prices (one inventory item = one whole serving), kcal per serving, wallets, wages and stock. FE-b gets its own design and freeze, and stays blocked until every price/portion row is sourced and a liquidity/stock proof holds under the residents' random spending. Accept? | **Yes.** | Yes, 2026-11-27 |
| Q-FE-5 | **Execution authority for FE-a once frozen.** A fresh Claude Opus 5.5 implementation session in its own worktree may implement, validate locally, make semantic commits, push, open the PR and repair CI up to READY FOR OPERATOR REVIEW. Merge stays with the operator. No paid calls, no models, no local full-workspace runs. Authorize? | **Yes.** | Yes, 2026-11-27 |
| **C-FE-2** | **Confirm the full consequence of Q-FE-2.** *Every* existing save of a world that enables `needs` — Market Town today, even though it still uses the old unitless hunger — stops loading in the new build; play it with the previous build. A CHANGELOG entry says so. (Keeping unitless saves loadable would need a second hunger store or per-variant versions, which the design forbids.) Confirm? | **Yes.** | Confirmed, 2026-11-27 |
| **Q-FE-6** | **Per-person daily use.** The operator's list (playable-MVP §5) included "optional per-person overrides". FE-a supports daily use per world and per actor class (e.g. a `high-energy` class at 2400); a value for one named person is not in this step (it would need a new per-person needs field). Defer per-person values? | **Yes, defer.** | Deferred, 2026-11-27 |

**Operator ruling, 2026-11-27 (relayed by the coordinator), quoted:** "营养设计那两点、S34-c3 也全都同意你的推荐" —
C-FE-2 confirmed (every existing save of a needs-enabled world stops loading in the new build; the CHANGELOG says to
use the previous build) and Q-FE-6 answered "defer per-person daily kcal".

Recorded defaults, not operator-listed: thirst/hydration stays excluded; intake diary ("kcal eaten today") is not
shown.

## 2. Requirement and scope

A new Market Town player starts with USD 10,000, buys realistically priced food, drinks and snacks, eats or
drinks one inventory unit per action, and sees an authoritative kcal reserve refill. Daily use is 2000 kcal per
86 400 world seconds, configurable per world and per actor class. Any other world chooses its own currency,
endowments, prices, kcal use and optional packs, with no kernel or client edits
([`operator-playable-mvp-20261123.md`](operator-playable-mvp-20261123.md) §1, §4 item 8).

Ownership is unchanged: `economy` alone owns money and prices; `inventory` alone owns holdings and stock;
`consumption` stays the stateless `eat`/`drink` interaction emitting `ItemsConsumed`; `needs` alone owns hunger
and now food energy. Controllers submit intents. Kernel, server and clients know no currency amount, meal or
calorie.

Out of scope for the whole step: hydration, illness, collapse, body mass, macronutrients; old-save migration;
live hot-removal of a system; shop hours, sleep advance, homes, dialogue/memory (separate lanes); any controller
optimizer.

## 3. PR split and dependencies

| PR | Design | Content | Depends on | State |
| --- | --- | --- | --- | --- |
| **FE-a** — food-energy engine | [`pr-fe-a-food-energy-engine.md`](pr-fe-a-food-energy-engine.md); contract [`pr-fe-a-contract.md`](pr-fe-a-contract.md) | `needs` v2 model, SDK owner-version/validation hook, `InvalidGenesisContent`, valid-genesis-first save creation in both CLI callers, schema-aware shared/2D/3D/agent/LM views. Synthetic worlds only. | `main` | DESIGN FROZEN 2026-11-27 (base `24dbc3bf`) |
| **FE-b** — Market Town food-energy content | [`pr-fe-b-market-town-food-content.md`](pr-fe-b-market-town-food-content.md); contract [`pr-fe-b-contract.md`](pr-fe-b-contract.md) (DRAFT — FREEZE-READY rev. 2 2026-11-28, Q-FEb-1 … 8 open); inputs in §6 | Market Town `configure/needs.yaml` food-energy 2000; every needs person initial reserve; ten items' kcal and serving; prices from dated sources; visitor 1 000 000 cents; reviewed NPC wallets, wages, stock and production; any explicit controller-profile change. | FE-a merged; §6 blockers closed | BLOCKED |

Why one engine PR rather than two: the decisive real-CLI evidence for the generic seams (an invalid
*genesis* refused with no save and no listener) needs a production owner that emits the new error, and that
owner is `needs` v2 (review R3, §5). A separate infrastructure-only PR would merge seams whose purpose is proven
only by stub owners. FE-a keeps the infrastructure commits (C1, C2) first and independently reviewable inside
one PR instead.

## 4. Step acceptance (NE-1 … NE-10)

IDs are retained from the reviewed amendment
([`fe-evidence/20261123-amendment-rev3.md`](fe-evidence/20261123-amendment-rev3.md) §8, revised by §11). FE-a owns
NE-1 … NE-8 and NE-10 on synthetic worlds (its subset E1–E7); FE-b owns NE-3, NE-5, NE-7 on Market Town and all of
NE-9. All are **NOT RUN**.

| ID | Observable acceptance | Owner |
| --- | --- | --- |
| NE-1 | The real loader rejects mixed or unknown models, wrong carrier, energy without serving, non-positive quantity, initial reserve above effective capacity, zero or oversized daily use, unitless knobs in energy mode, energy keys in unitless mode, missing required packs. Decode errors name file and key; genesis errors name owner, entity key, code and field. `server --save` and `run --save` on failure exit non-zero, create no save directory/`world.sqlite`/WAL/SHM, and bind no port. Actor-class 2400 versus base 2000 accepted. | FE-a C1–C3 |
| NE-2 | Literal oracle with D = C = 2000, initial 1500: 6 h → 1000; +500 → 1500; 12 h → 1000; full 2000 empty after 24 h; zero stays zero; fractional seconds preserved across folds; cap and zero discard excess; same-instant meals both count; i128 intermediates; split-interval equals unsplit. | FE-a C3 |
| NE-3 | Buy one serving at quoted price P: buyer −P, seller +P, stock −1, buyer count +1 exactly once; eat: count −1, reserve +K capped, no money change; unaffordable, out-of-stock or unsupported actions change nothing. | FE-a C3 (synthetic); FE-b (Market Town, visitor 1 000 000 cents) |
| NE-4 | Zero-kcal drink consumes the unit and writes no energy fold; caloric drink adds exactly its kcal; consuming at full is legal; no thirst or collapse. | FE-a C3 |
| NE-5 | Save after purchase, meal, mid-depletion and sleep; close/reopen and kill/resume equal the uninterrupted state; no regrant, no duplicate consumption, no wall-time depletion. Incompatible old version/config is refused with the first applicable explicit error and the save's *logically committed contents* are unchanged (closed save and crash-tail WAL family, on scratch copies). | FE-a C2–C3; FE-b repeats on Market Town |
| NE-6 | 1/2/4-hour day presets give equal reserve at equal world instants; no midnight refill; observation frequency changes nothing; same rate asleep and awake; no needs Process or runtime fact. | FE-a C3 |
| NE-7 | Headless, 2D, 3D, agent and LM views show the same owner kcal, capacity, daily use, serving and bar; another person cannot see the reserve; en and zh_Hans in both actual clients; unitless rows unchanged; renderer/model absence changes no authoritative outcome. | FE-a C4 (synthetic); FE-b (Market Town package) |
| NE-8 | A second generated world with an exponent-0 fictional currency, different balance and 2400 kcal/day works without kernel or client edits; unitless variant works; removing `needs` leaves no energy record or row and no rule/draw change; Social Cafe and Bodies Yard baseline facts unchanged; bad partial removal refused by name; unrelated packs keep schema 1. | FE-a C1, C3 |
| NE-9 | Ten Market Town items have sourced, dated price and kcal per whole serving; a reachable-policy, every-prefix liquidity and per-kind stock proof holds for every autonomous seat; S35 rhythm/300-day checks keep their original thresholds. | FE-b only |
| NE-10 | Independent review of ownership, size growth, coupling, no world switches, no client rules, no money grants, no generic resource manager, no new dependency; arithmetic, restart refusal and provenance reviewed. | FE-a and FE-b |

## 5. Review history and disposition

The design was drafted in Codex sessions on 2026-11-23 in `/tmp`, recovered on 2026-11-27, and is now tracked
under [`fe-evidence/`](fe-evidence/) (verbatim copies; the recovered archive's `INDEX.md` records completeness).
Those files are history and evidence, not a second authority: where they differ from FE-a's design, FE-a wins.

| Finding | Source | Disposition in FE-a |
| --- | --- | --- |
| N1 — new save allocated before genesis validation | [`review-rev1-rev2`](fe-evidence/20261123-review-rev1-rev2.md) | Resolved: lazy `create_with_backend` factory, both CLI callers (FE-a §3.6, C2). |
| N2 — authored-error path | same | Resolved: domain-neutral `KernelError::InvalidGenesisContent`, genesis-only classification (FE-a §3.5). Needs Q-FE-3. |
| N3 — version plan needs an SDK seam | same | Resolved: per-owner schema constants + `validate_authored` hook, version table (FE-a §3.3–3.4). |
| N4 — cheap basket is not a diet guarantee | same | Moved to FE-b: reachable-policy, every-prefix liquidity and stock proof is a content-freeze blocker (§6). |
| N5 — byte preservation of refused saves | same | Resolved: logical committed-content preservation, closed + WAL families on scratch copies (FE-a §3.7). |
| Threshold wording (hunger *at or above*) | same, rev2 re-review | Resolved (FE-a §3.1). |
| R1 — one pytest command across two members drops guards | [`review-canonical-a`](fe-evidence/20261123-review-canonical-a.md), [`-b`](fe-evidence/20261123-review-canonical-b.md) | Resolved: separate invocations (FE-a C4). |
| R2 — broken evidence filename | `review-canonical-b` | Resolved: the raw searches are [`fe-evidence/20261123-source-searches.md`](fe-evidence/20261123-source-searches.md). |
| R3 — C2 cannot prove invalid *genesis* through the real CLI before C3 | `review-canonical-b` | Resolved: C2 proves the factory with stub owners plus a real-CLI *assemble* failure (an existing defect, §7 item 2); the real-CLI invalid-genesis tests live in C3. |
| Re-review of the expanded draft against the current base | coordinator ready queue 2026-11-27 §3 item 1 (untracked, `/Users/yuema137/mineworld-worktrees/_planning/`) | Done: all 47 audited files hash-identical at `7718d952`; re-audit at `8c971f0f` in FE-a §2. |
| Red-team pre-freeze review of FE-a revision 1 (head `6105eea7`): FREEZE-AFTER-FIXES, 4 MAJOR, 6 MINOR | `/Users/yuema137/mineworld-worktrees/_reviews/design-fe.md` | Revision 2 applies #2, #5–#12 (FE-a §8 lists each). #3 → C-FE-2 and #4 → Q-FE-6, both returned to the operator (§1). |

## 6. FE-b blockers (content freeze), recorded so they are not lost

From [`fe-evidence/20261123-content-evidence.md`](fe-evidence/20261123-content-evidence.md) ("READY FOR
INDEPENDENT REVIEW; NOT CONTENT-FREEZE READY") and review N4:

1. **Provenance.** Bread, milk, juice and soup rows are coherent whole-unit comparisons (Target online,
   observed 2026-11-23, no store/ZIP). Open: apple (small Gala price vs large-apple kcal), coffee (size basis of
   5 kcal), tea/croissant/cake (campus menu vs Starbucks recipe identity), sandwich (price **not observed** —
   authored $7.49). Each row must be closed or explicitly labelled "authored estimate"; no blanket "realistic"
   claim beyond "dated US retail observations, contextual".
2. **Reachable spending.** `fetch::best` buys the most nourishing available kind (ties: lowest ItemId), not the
   cheapest; `offered::attempt` can buy/eat/give below threshold; `beyond_chance_budget` limits each chance
   purchase to `chance_spend_permille` of the current wallet, not a daily budget. Raising NPC wallets activates
   more chance purchases. A cheap feasible basket ($11.01/day) is not a proof.
3. **Liquidity and stock.** Every prefix: opening cash + guaranteed wages − bounded outflow ≥ next required
   food, including pre-first-pay for Alice, Felix and Ivan; employers prefunded for full wages; per-kind stock
   for correlated morning demand. Candidate A (content-only retune) is a reviewable proposal, not a proof;
   Candidate B (Q × Pmax envelope) is shown impractical.
4. **Policy.** If a practical proof needs `chance_spend_permille: 0` for Market Town or any rule change, it is
   an explicit, separately reviewed controller-profile item in FE-b, never tuned after a failed run.
5. Six inventory slots are counts, not kinds; minimum three free at genesis. Otto has needs but no controller:
   scope autonomous guarantees to active seats.

## 7. Audit notes on the recovered drafts (what changed or was wrong)

1. **Base.** Drafts audited `b457a94d` on a dirty root. `7718d952` is 18 commits later; none touches the mapped
   source (`git diff b457a94d 7718d952` over every mapped path is empty), and all 47 hashes in
   [`fe-evidence/20261123-source-manifest.json`](fe-evidence/20261123-source-manifest.json) verify against
   `7718d952` (`shasum -a 256 -c`, 47 OK). At `8c971f0f`, 45/47 match; `authoring/src/configuration.rs` and
   `worldpack/src/configure.rs` changed in S31-b2 (#533) without affecting FE-a's contracts (FE-a §2).
2. **Missed existing defect.** `tools/cli/src/serve.rs::persisted` (line ~516) and `tools/cli/src/run.rs::Begun::new`
   (line ~335) call `SqliteBackend::create` *before* `pack.assemble()`. Any assemble or genesis failure therefore
   leaves an empty `world.sqlite`; `SqliteBackend::exists` checks only the file, so the next launch takes the
   resume branch and fails reading saved genesis. The drafts framed N1 as a food-energy problem; it is a general
   bug, and C2 now has a real-CLI regression for it.
3. **Heavy-job wrapper.** The drafts said "coordinator-verified wrapper; do not guess its path". It exists:
   `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh` (R-LOAD-1, cited by `pr-s35-nb-needs.md`).
4. R1, R2, R3 and the threshold wording (§5) were real errors and are corrected.
5. The drafts' endpoint lines ("no public write") reflected a design-only Codex dispatch; they are replaced by
   Q-FE-5. The 2026-11-23 GPT-6.1 Sol model ruling is superseded by the 2026-11-27 Claude Opus 5.5 ruling
   (`CLAUDE.md` §3.4).
6. The draft's companion "amendment" was meant to be merged into `operator-playable-mvp-20261123.md`. This step
   document is now the single authority; the primary links it from the playable amendment and `overall.md`.

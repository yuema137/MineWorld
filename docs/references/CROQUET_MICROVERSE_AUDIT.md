**Status:** informational reference audit (REUSE_POLICY §19), 2026-10-11. **Not authoritative architecture**: it changes no MineWorld contract; where it disagrees with a specification, the specification governs. Gathered by a research agent from primary web sources through a summarising fetch tool (no clone); items marked *unverified* must be re-read at source before any design cites them. Summary and rulings: [`PRIOR_ART_2026-10.md`](PRIOR_ART_2026-10.md).

# Prior-art audit: Croquet Microverse (and Croquet SDK / Multisynq) against MineWorld

Access date for all web sources: 2026-10-11. Repository files cited are from `/Users/yuema137/MineWorld` at `main` (HEAD 37f6892).

## 1 Summary

- Croquet Microverse is a JavaScript framework for multiuser 3D web worlds, built on the Croquet SDK, now marketed as Multisynq. Its core is a replicated deterministic model: every client runs the same Model code in a deterministic VM ("Teatime"), and a reflector only orders events.
- MineWorld is the opposite trade-off: a headless Rust server is the single authority, and clients only observe and submit intents. Croquet's model/view split, Cards/Behaviors, and snapshot-plus-replay ideas are useful references, but none can be adopted as a dependency without breaking server authority, Tier 0/1/2 extension rules, and self-hosted operation.
- Croquet's strongest transferable material is its explicit list of model-determinism rules (no `Math.random`, `Date.now`, `setTimeout`, HTTP, or DOM in model code) and its persistence split between a stable persistent identity and a code-hash session identity.
- Croquet has no evidence of sandboxing user behaviour code. MineWorld already answers that question with WASM capability sandboxing (ARC-8), so the comparison is one-sided in our favour on security.
- Maintenance looks weak: the cached commit list shows the newest commit as Oct 30, 2025, about 11 months before this audit. Treat it as a reference, not a living dependency.

## 2 Facts (with sources)

Repository and licence

- F1. croquet/microverse is licensed under Apache License 2.0. The README says: "The source code and assets in this repository are licensed under Apache License 2.0." Copyright line: "Copyright (c) 2022 Croquet Corporation". Source: https://github.com/croquet/microverse (README; access 2026-10-11). The LICENSE file URL returned 404 through the fetch tool, so the README is the cited source.
- F2. croquet/croquet (the SDK) is Apache-2.0, per its README and LICENSE.txt link. Source: https://github.com/croquet/croquet (branch `dev`; access 2026-10-11). The page reports 4,123 commits.
- F3. Stars and activity on croquet/microverse (cached view): 190 stars, 57 forks, 25 watchers, 13 open issues, 1 open PR, 2,206 commits on `main`, Releases section empty in the capture. Source: https://github.com/croquet/microverse (access 2026-10-11).
- F4. Commit history page (cached capture): newest visible commit dates are Oct 30, 2025; Jun 10, 2025; then 2024 dates back to Sep 28, 2023. Visible authors: yoshikiohshima, aranlunzer, e46mpwr. The page does not show a total. Source: https://github.com/croquet/microverse/commits/main (access 2026-10-11). The capture time is not stated; this is the newest activity we could see.
- F5. Status: the microverse README title reads "Croquet Microverse (Beta)." It advises new worlds to use "a stable version of library" via `npm create croquet-microverse@latest`, and says not to clone the repo for that. Source: https://github.com/croquet/microverse (access 2026-10-11).
- F6. npm: `@croquet/microverse-library` at 0.4.4 and `create-croquet-microverse` at 0.8.0 are cited by search results. Publish dates were not verified (the npm page returned 403). Sources: https://unpkg.com/create-croquet-microverse@0.8.0/README.md (search result, access 2026-10-11).
- F7. Multisynq client: `@multisynq/client@1.0.4` is the current npm name in search results. Source: https://unpkg.com/@multisynq/client@1.0.4/README.md (search result, access 2026-10-11). The rename from Croquet to Multisynq is seen in the docs and README but its date is UNVERIFIED.

Architecture (replicated deterministic model)

- F8. croquet/croquet README: "the multiplayer code is executed on each client in a synchronized virtual machine, rather than on a server." Reflector: "the reflector server keeping VMs in sync is available as a node.js package." Sync runs on "Multisynq's global DePIN network." Source: https://github.com/croquet/croquet (access 2026-10-11).
- F9. docs.multisynq.io, Real-time synchronization: "Multisynq runs your Model code in a deterministic virtual machine called Teatime." "Reflector servers order all events into a single canonical stream." "Reflectors only pass messages - all logic runs on clients." "Never write to Model from View," with communication "only through events." Source: https://docs.multisynq.io/essentials/sync (access 2026-10-11).
- F10. Model determinism rules (same page): use `this.random()` not `Math.random()`; `this.now()` not `Date.now()`; "Don't use `Date.now()` or `setTimeout()` in Model code"; "Don't make HTTP requests from Model code"; "Don't access DOM from Model code." Source: https://docs.multisynq.io/essentials/sync (access 2026-10-11).
- F11. Simulation time: `this.now()` "advances only when heartbeat ticks are received from the reflector," starts at 0, is identical for all users, and repeated calls within one tick return the same value. Heartbeats "Typically ~60-120Hz." `this.future(ms)` schedules a call; "Cannot cancel already scheduled future calls" (use a flag instead). Source: https://docs.multisynq.io/tutorials/sim-time-future.md (access 2026-10-11).
- F12. Older Croquet SDK tutorials (2020-era) describe the same pattern (model/view, reflector, snapshots). The search cache showed these URLs, but fetches of `croquet.io/sdk/docs/...` pages returned 404. Treat these descriptions as secondary: https://croquet.io/sdk/docs/tutorial-2_1_model_view_reflector.html and https://croquet.io/sdk/docs/tutorial-2_7_snapshots.html (search snippets only, access 2026-10-11).

Snapshots and joining

- F13. Snapshots: "The Multisynq reflector periodically requests one of the session participants to create a snapshot of the current model state." The frequency is not given. A joiner starts from the last snapshot, the reflector resends later events, and the model replays them. "Your model's `init()` method is NOT called when loading from a snapshot." Known limitation: "The snapshot system is currently unoptimized and may cause performance hitches when snapshots are taken." Source: https://docs.multisynq.io/tutorials/snapshots.md (access 2026-10-11).

Persistence and versioning

- F14. Persistence is explicit: "Your application decides when to save using `persistSession()`". Recommended: "Selective - save only what matters". Source: https://docs.multisynq.io/tutorials/persistence.md (access 2026-10-11).
- F15. Two identities: `sessionId` = appId + name + code hash, "Changes with any code modification"; `persistentId` = appId + name, "Survives code changes." On code change a new session is created and "Persistent data is loaded into new code version." Snapshot-only state is "Lost when code changes." The page does not explain how the code hash is computed. Source: https://docs.multisynq.io/tutorials/persistence.md (access 2026-10-11).
- F16. No built-in migration system. Developers write migration code, for example a `switch` on a stored `version` field ("Always save in latest format"). Same source.
- F17. Persistent data is "encrypted"; "Only clients with session password can decrypt"; "Server cannot read your data"; "Lost password = lost data (unrecoverable)." Same source.
- F18. The earlier Croquet persistence page cited in search results could not be fetched (404). The claim "whenever you change some model code, the model is initialized from scratch" comes from search snippets of older docs; treat as secondary: https://croquet.io/docs/croquet/tutorial-2_A_persistence.html (access 2026-10-11).

Microverse world building: Actor/Pawn, Cards, Behaviors

- F19. Microverse README and the croquet.io Microverse page: "abstractions provide a uniform object model, and pluggable behavior descriptions for those objects," and the abstractions integrate with a text editor for "multiuser live programming." New worlds start with `npm create croquet-microverse@latest`. Sources: https://github.com/croquet/microverse and https://croquet.io/docs/microverse/ (access 2026-10-11). Microverse "Elected", "Physics (Rapier)", "Cards", and "Avatars" are listed as docs topics in the README; "Rapier" is named explicitly.
- F20. CardActor/CardPawn (from search snippet of https://croquet.io/docs/microverse/CardActor.html; the page itself returned 404, so this is UNVERIFIED beyond the snippet): a card has a CardActor on the model side and a CardPawn on the view side; each holds a list of behaviors; behaviors on one card share its properties; `call()` invokes a method on another behavior; `destroy()` runs `teardown()` on pawn and actor behaviors before removal; vectors are Worldcore types that are plain JS arrays. Example behavior file: `microverse/behaviors/tutorial/gridFloor.js`. Source: search result list (access 2026-10-11).
- F21. The generic Actor/Pawn pair in Worldcore is not confirmed by a fetched page. The only confirmed pair is CardActor/CardPawn (F20). Actor/Pawn is UNVERIFIED as a general pattern.
- F22. Persistent content in Microverse: a tutorial titled "Adding Persistent Content to a Microverse World" exists (search result, https://croquet.io/docs/microverse/tutorial-Adding%20Persistent%20Content.html). Fetch returned 404; its contents are UNVERIFIED.

Code loading and security

- F23. No source fetched states that Microverse behaviour code is sandboxed, capability-restricted, or signed. Searching found no statement either way. Status: UNVERIFIED / unknown. The only documented safety boundary is the determinism rules in F10 and the encryption in F17, which protects stored data, not behaviour code.

## 3 Concept map

Legend: same = same concept and effect; different = different concept; better = their approach is stronger in our terms; worse = their approach is weaker in our terms.

| Croquet / Microverse | MineWorld (our term, where it lives) | Verdict |
| --- | --- | --- |
| Model code replicated on every client's deterministic VM | Authoritative state in one Rust server (`server/`, `kernel/`); single-writer rule (`docs/ARCHITECTURE.md` §4) | different; ours is worse for offline/peer play and better for cheating, cost, and self-hosting |
| Reflector: orders events, runs no logic (hosted on Multisynq DePIN) | Host/server orders requests and owns time (`server/src/host.rs`, `server/src/hosted.rs`); self-hosted | same role for ordering; ours is better (no third-party network, runs headless) |
| Model / View (Model deterministic, View local, events only) | Simulation / Presentation; clients submit `ActionIntent`s and observe `Observation`s (`docs/CORE_CONCEPTS.md`, `docs/ARCHITECTURE.md` §§3, 6, 10) | same split; ours is stricter (clients never write state) |
| Actor / Pawn (Model-side / View-side object pair; confirmed only for CardActor/CardPawn) | Entity + Component (owned state) and presentation adapters in `clients/2d`, `clients/3d-spike` | same split at the object level; ours separates identity and state from presentation more strictly |
| Cards (uniform object model with behaviour list) | Entities with Components; Person/Place/Item/Organization; `entities/` | different; theirs is a generic bag of behaviours, ours is typed components per owning system (DEP-1 purpose-built store, not ECS) |
| Behaviors attached/detached at runtime, `call()` across behaviors, shared card properties | System Packs installed at build time (`systems/installed/`, `ARC-33`, `DEP-12`); cross-system effects via events and owner decides (`ENGINEERING_STANDARDS` single ownership) | different; ours is worse for live editing, better for ownership and determinism |
| `teardown()` on behaviour removal | System removal per world (DEP-1 "systems are installed and removed per world"); `kernel/src/system.rs` `System` trait | same idea; we must confirm the trait's teardown path (open question Q3) |
| `this.future(ms)` (not cancellable; use flag) | Kernel process scheduling and `Process`/`wake` (`docs/ARCHITECTURE.md` §5; `kernel/src/`) | different time model; ours is virtual, owned by the kernel, and cancellable through the process lifecycle |
| `this.now()` synchronized from reflector heartbeats | World clock and pace schedule (ARC-27 headless pace; ARC-75 two time domains; `docs/ARCHITECTURE.md` §5) | different; ours is server-decided, not heartbeat-driven |
| Model determinism rules (no `Math.random`, `Date.now`, HTTP, DOM) | Seeded randomness, LM-independent core, headless first (`ENGINEERING_STANDARDS` §10; `clippy.toml` disallowed-types) | same rule; ours is enforced at build time in part; theirs is convention in a JS VM |
| Snapshots auto-taken by one participant, joiners replay events | Snapshot plus event journal; byte-exact replay (ARC-25; ARCHITECTURE §§7–8; `persistence/src/replay.rs`) | same idea; ours is better (server-side, verified byte-for-byte, not dependent on a participant) |
| "`init()` not called when loading from snapshot" | Restore runs no genesis hooks (`persistence/src/replay.rs`, to be confirmed) | same lesson; worth a test |
| Explicit `persistSession()`, selective save | Persistence explicit through `PersistenceBackend` (`persistence/src/backend.rs`, `sqlite.rs`); save retention (ARC-81) | same; ours is better (retention rule, compression, whole log kept) |
| `persistentId` (survives code change) vs `sessionId` (code hash) | World identity via World Pack name/version; save format version counter refused by name (ARC-25; `persistence/src/format.rs`) | different; theirs is better at the "survive code change" distinction; ours is better at refusing unknown versions |
| No built-in migration; developer switches on a `version` field | Save version counter refused by name; semver range check (ARC-25, DEP-21) | same weakness in theirs; ours is clearer, but migrations are not designed yet |
| Session data encrypted with session password | No encryption at rest in our SQLite persistence (to confirm) | different; theirs is worse for inspection, better for hosted trust |
| Behaviours as JS modules, no sandbox documented | Tiered extensions: Tier 0 declarative, Tier 1 WASM Component with capabilities, Tier 2 renderer (ARC-8, `PACKAGE_FORMAT.md` §6) | different; ours is better (sandboxed simulation code) |
| World template via `npm create croquet-microverse` | World Pack (YAML, Tier 0), `worlds/`, `worldpack/`; `install modules -> compose world -> configure -> run` (`VISION.md`) | same goal; ours is better aligned (no code fork; packs installable) |
| Physics via Rapier (listed in Microverse docs) | Rapier `rapier3d` with `enhanced-determinism` inside `systems/bodies` (DEP-13) | same library; the reuse is already done |
| Three.js rendering | Godot 2D and 3D clients (DEP-16, DEP-20) | different; not a reuse candidate |

## 4 Are we reinventing?

Concrete modules where Croquet has solved a related problem, and whether reuse is possible.

- **Persistence and snapshot catch-up** (`persistence/src/sqlite.rs`, `persistence/src/replay.rs`, `persistence/src/format.rs`). Croquet's snapshot-plus-replay is the same idea as ARC-25, but it works only because every client runs the model deterministically. MineWorld replays on the server against a journal. Reuse: not possible as code; the design already exceeds it. Keep our own (DEP-2 settled SQLite behind `PersistenceBackend`).
- **Behaviour attach/detach** (`systems/installed/`, `kernel/src/system.rs`). Microverse's Card + Behaviors is a runtime-composed object model. We decided against runtime loading (ARC-33, DEP-12, ARC-8). Reuse: not possible, and the decision should stand. The idea of a uniform object with a behaviour list is what DEP-1 deliberately rejected in favour of typed component stores.
- **Teardown on removal** (`kernel/src/system.rs`). Croquet's `teardown()` on card destroy is a small, well-named lifecycle hook. We should confirm our `System` trait defines equivalent removal behaviour (Q3). Reuse: idea only; no code.
- **Scheduling via `future()`** (`kernel/src/`, Process scheduler). Croquet's non-cancellable `future()` is weaker than our process lifecycle (ARCHITECTURE §5). Keep ours.
- **Determinism rules** (`clippy.toml`, `ENGINEERING_STANDARDS` §10). We already have the rules; Croquet's list is a checklist to make sure every rule has an enforcement point (see section 5).
- **Physics** (`systems/bodies`, DEP-13). Already reuses Rapier, the same library Microverse uses. No change needed; this is the only real overlap we already share.
- **Encryption at rest.** Croquet encrypts persisted data with a session password. We have no such feature. This is a product question, not a reinvention; if we want it, it is a new decision record (open question Q5).

Conclusion: no module in MineWorld is a candidate for replacement by Croquet code. Croquet's useful contribution is design lessons, not components.

## 5 Ideas worth adopting (design only, not dependencies)

1. **Restore must not run genesis or init hooks.** Croquet's rule (F13) is that `init()` never runs on snapshot load. Add an integration test in `persistence/tests/` (or `server/tests/`) that restores a saved world and asserts no genesis fact or `init` side effect is re-emitted. Affects: `persistence/src/replay.rs`, `server/src/hosted.rs`.
2. **Enforce determinism bans by lint, not convention.** Croquet lists the banned calls (F10). Extend `clippy.toml` `disallowed-methods` (it already uses `disallowed-types`) to forbid `std::time::SystemTime::now`, `Instant::now` in simulation crates, and unseeded RNG constructors such as `thread_rng`. Affects: `clippy.toml` and CI (`docs/ENGINEERING_STANDARDS.md` §§15–16). Scope to `kernel/`, `systems/`, `persistence/`.
3. **Separate world identity from code identity explicitly.** Croquet's `persistentId` vs `sessionId` (F15) makes "survives code change" a named concept. Consider stating in `docs/PACKAGE_FORMAT.md` and `persistence/src/format.rs` which identity a save is bound to (World Pack identity vs build/schema version), so a System Pack upgrade is a documented, reviewed event. Affects: `docs/PACKAGE_FORMAT.md`, `persistence/src/format.rs`. Needs an ARC record before any code change.
4. **Make the "no cross-behaviour call" rule visible.** Croquet's `call()` between behaviours on one card (F20) is exactly what our single-owner and events-only rule forbids. Keep the rule; add a short example to `docs/MODULE_SPEC.md` contrasting an event with a direct call. Affects: `docs/MODULE_SPEC.md`.
5. **Teardown contract for Systems.** Verify that `System` in `kernel/src/system.rs` has a defined removal path that releases the system's own state and emits no more facts (Croquet's `teardown()` idea). If it lacks one, record it in a design before adding it. Affects: `kernel/src/system.rs`, `systems/README.md`.
6. **Snapshot cost is a named risk.** Croquet documents snapshot hitches as a known limitation (F13). Our snapshot pruning and compression (ARC-81) already addresses retention; add a measured snapshot pause to the performance section of `docs/ACCEPTANCE.md` or the persistence README. Affects: `persistence/README.md`.

## 6 Why not adopt as a dependency

- **Architecture does not fit.** Croquet runs the authoritative model on every client in a JS VM (F8, F9). MineWorld requires one headless Rust server as authority (`docs/ARCHITECTURE.md` §4; `docs/NETWORKING.md`). Adopting Croquet would force the world model into its execution model, which `docs/REUSE_POLICY.md` forbids ("never force MineWorld inside a framework's execution model").
- **Hosting dependency.** The SDK's sync runs on "Multisynq's global DePIN network" (F8), and persisted data is encrypted with keys the server cannot read (F17). MineWorld must be self-hostable and runs headless on macOS, Linux, and Windows (`docs/ENGINEERING_RULES.md`, the all-platforms memory).
- **Language.** The Model code is JavaScript. MineWorld's kernel, systems, persistence, and server are Rust (`docs/ARCHITECTURE.md` §13; `docs/DECISIONS.md` DEP-1 to DEP-3).
- **Extension model.** Behaviours are unsandboxed JavaScript (no sandbox found, F23). MineWorld's simulation extensions are WASM Components with capabilities (ARC-8). Adopting Croquet's model would weaken the security story.
- **Maintenance.** Microverse is labelled Beta (F5), has no release visible on GitHub (F3), and its newest visible commit is Oct 30, 2025 (F4). Licence (Apache-2.0, F1, F2) is not the obstacle; the licence check in `deny.toml` already accepts Apache-2.0.
- **Scope.** Microverse is a 3D web world-builder. MineWorld's acceptance criterion is kernel-independent composition of systems (`docs/VISION.md`). Croquet provides none of the kernel, system-pack, or World Pack concepts.
- **Record requirement.** Under `docs/REUSE_POLICY.md` and `docs/DECISIONS.md`, rejecting a substantial dependency needs a short record. This document is the research input; a DECISIONS entry would be needed if a design is later reviewed against Croquet (for example, idea 3 in section 5).

## 7 Open questions

- Q1. Microverse's Actor/Pawn page and the general Worldcore pattern could not be fetched (404). Confirm before citing Actor/Pawn as a general concept.
- Q2. The Microverse persistence tutorial and the Croquet persistence page returned 404. Confirm whether persistence is per-session only, and what happens to data on code change, from the current Multisynq docs (`https://docs.multisynq.io/llms.txt` lists `tutorials/persistence.md`).
- Q3. Does `kernel/src/system.rs` `System` define a removal or teardown path that releases state and emits no facts? Needs a source read of the trait and the `systems/` packs before idea 5 is accepted.
- Q4. Does Microverse sandbox or restrict user behaviour code in any way? No evidence found (F23). Check `croquet/microverse` source for `behaviorModules` loading.
- Q5. Does MineWorld intend encryption at rest for saves? Not specified in the docs read here. If yes, it needs a DECISIONS record.
- Q6. Exact date of the Multisynq rename and whether `@croquet/*` packages are still maintained. Not verified; npm pages returned 403.
- Q7. The snapshot frequency and cost in Multisynq's current system are not stated in the pages read (F13). Do not quote a number.
- Q8. Croquet's "sessionId includes a code hash" implies that any change to model code invalidates persistence. Whether this is the current behaviour, or only the 2020-era one, is unconfirmed (F15, F18).
- Q9. Whether `this.future()` ordering for simultaneous calls is defined. The docs do not say (F11).

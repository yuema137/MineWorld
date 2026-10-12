# Prior-art audit, October 2026 — what MineWorld builds, borrows and declines

**Status:** informational reference audit under [`REUSE_POLICY.md`](../REUSE_POLICY.md) §19, dated
2026-10-11. It is **not authoritative architecture**: it changes no frozen contract. Every proposal in
§5 becomes binding only through its own design document and operator approval. Where this file and a
specification disagree, the specification governs and this file is the defect.

**Audience:** coding agents. Vocabulary is [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md)'s throughout.

**Detailed audits** (each with sources, access dates and unverified items):

| Subject | Audit |
| --- | --- |
| Croquet Microverse (Actor/Pawn, Cards + Behaviors, synchronized worlds, persistence) | [`CROQUET_MICROVERSE_AUDIT.md`](CROQUET_MICROVERSE_AUDIT.md) |
| Decentraland SDK7 (ECS, custom components, scene manifest, Creator Hub, limits) | [`DECENTRALAND_SDK7_AUDIT.md`](DECENTRALAND_SDK7_AUDIT.md) |
| Vircadia (legacy native core, vircadia-world, metaverse server) | [`VIRCADIA_AUDIT.md`](VIRCADIA_AUDIT.md) |
| AI NPC runtimes: Inworld, Convai, Concordia, AI Town, Stanford Generative Agents | [`AI_NPC_RUNTIMES_AUDIT.md`](AI_NPC_RUNTIMES_AUDIT.md) |

(KsanaDock's Microverse, a different project, is audited separately in
[`MICROVERSE_AUDIT.md`](MICROVERSE_AUDIT.md).)

---

## 1. Why this audit was run

The operator observed (2026-10-11) that "a persistent world, extensible objects and behaviours,
multiplayer sync, AI NPCs and custom worlds" has been attempted many times, and asked whether MineWorld
is rebuilding layers that mature projects already provide. The operator's instruction was to learn from
existing frameworks and not to add dependencies on immature tools.

The question this audit answers, per [`REUSE_POLICY.md`](../REUSE_POLICY.md) §17, has two halves:

1. Are we reinventing a mature wheel?
2. Are we forcing an existing wheel where it does not fit?

## 2. The answer in one paragraph

No audited project covers MineWorld's middle layer: an engine-neutral, persistent, typed semantic
world whose Systems compose by installation, whose People are independent of their Controllers, whose
saves replay byte for byte, and whose one world projects to several renderers. Each project covers
part of an adjacent layer well. None is adoptable as a dependency: each either places authority
somewhere MineWorld forbids (every client, a hosted vendor, a model acting as referee), is tied to its
own platform (land, wallet, renderer, hosted service), or is not mature (self-declared beta or pre-1.0,
deprecated lineage). What they offer is **design evidence**, and §5 lists eleven concrete ideas, two of
which close real gaps in our code today.

## 3. Coverage map

```text
                       MineWorld's own layer (no audited project has it)
     typed semantic world · single-writer Systems · packs with identity and licences ·
     byte-exact event-sourced saves · per-observer perception · seats (human ⇄ AI takeover) ·
                    one world, many renderers · models removable
                                         │
   ┌──────────────────────┬──────────────┴─────────────┬───────────────────────────┐
   │ composition / ECS    │ shared world / runtime     │ AI characters             │ presentation
   │ Decentraland SDK7    │ Croquet, Vircadia          │ Concordia, AI Town,       │ Godot (adopted),
   │ Croquet Cards        │                            │ Smallville, Inworld,      │ Unreal (spike)
   │                      │                            │ Convai                    │
```

Commodity infrastructure is already reused rather than rebuilt (each with a `DEP-` record): SQLite,
Rapier, `pathfinding`, zstd, WebSocket (`tokio-tungstenite`, `websockets`), Godot, Ollama and the
OpenAI-compatible APIs. The audit found no further commodity component we are rebuilding.

## 4. Verdict per project

Classification per [`REUSE_POLICY.md`](../REUSE_POLICY.md) §19: `REUSE` · `ADAPT` · `REFERENCE ONLY` ·
`REJECT`.

| Project | Licence | Maturity (2026-10-11) | Architecture fit | Verdict |
| --- | --- | --- | --- | --- |
| Croquet Microverse | Apache-2.0 | labelled Beta; no GitHub release; newest commit seen 2025-10-30 | **opposed**: the authoritative model runs as deterministic JavaScript on every client; the reflector only orders events; snapshots taken by a participant | `REFERENCE ONLY` |
| Decentraland SDK7 | Apache-2.0 (toolchain) | active platform | **partial**: ECS with data-only components is the same shape as ours, but no single-writer rule, per-client sync by default, optional server mode, and publishing bound to LAND/NAME and its renderer | `REFERENCE ONLY` |
| Vircadia native core | Apache-2.0 | deprecated in its own README in favour of vircadia-world | entity changes broadcast to the whole domain; no documented sandbox for scripts | `REJECT` |
| vircadia-world | Apache-2.0 | self-declared pre-1.0, not for production; no tagged release | all state in Postgres rows; no event log; sync protocol undocumented | `REFERENCE ONLY` |
| Concordia (DeepMind) | Apache-2.0 | research library, "not an officially supported Google product" | a model-driven Game Master resolves actions, which inverts our authority; no documented seeding or replay | `REFERENCE ONLY`; candidate for an optional controller adapter after Milestone D |
| AI Town (a16z) | MIT | starter kit | Convex owns all state; unseeded randomness | `REFERENCE ONLY` |
| Stanford Generative Agents | Apache-2.0 (per repository sidebar; *unverified*) | research code | OpenAI-only, no local model, no replay | `REFERENCE ONLY` (memory stream, reflection, planning) |
| Inworld | proprietary, hosted | Runtime and Character Engine absent from current docs; Character Studio deprecation *unverified* | hosted model owns the character; cannot run offline; no replay | `REJECT` as a controller; a presentation-side voice adapter may be considered later |
| Convai | proprietary, hosted | active; Unity/Unreal/Web plugins | hosted; US processing; no replay | `REJECT` as a controller (same note as Inworld) |

What MineWorld already does that none of them do — confirmed by the audits and worth stating plainly:
recorded cognition replay (`ARC-58`), per-observer perception with typed audiences
(`server/src/perception.rs`, `systems/presence/src/audience.rs`), seats with human/AI takeover
(`server/src/seats.rs`, `AC-5`), byte-exact replay across four platforms (`AC-8`, `AC-12`), and pack
identity with SPDX licences and requirement resolution (`ARC-53`…`ARC-55`).

## 5. Ideas to adopt — design, not dependencies

Each idea names the code it would touch and whether it needs a design. None is implemented by this
audit. Priority: **P1** closes a gap in our code today; **P2** strengthens a contract before it is
widely used; **P3** is a later improvement.

| # | Idea | Source | MineWorld today (checked 2026-10-11) | Would touch | Priority |
| --- | --- | --- | --- | --- | --- |
| PA-1 | **Ban nondeterministic calls by lint**: wall clock (`SystemTime::now`, `Instant::now`) and unseeded randomness (`thread_rng`, `rand::random`) in simulation crates | Croquet's banned-call list | `clippy.toml` bans only `HashMap`/`HashSet` (`disallowed-types`); no `disallowed-methods` | `clippy.toml` (or per-crate config for `kernel/`, `systems/`, `persistence/`), `docs/ENGINEERING_STANDARDS.md` | **P1** — small PR, no design beyond a short note |
| PA-2 | **Restore never re-runs genesis**: an integration test that a resumed world emits no genesis fact and runs no setup reaction again | Croquet: `init()` never runs on snapshot load | covered implicitly by byte-identical resume tests; no test names the rule | `persistence/tests/`, `tools/cli/tests/` | **P1** — test-only PR |
| PA-3 | **World identity versus code identity, named**: which identity a save is bound to (world instance, World Pack identity, pack versions, `SAVE_FORMAT`) and what a System Pack upgrade does to an existing save | Croquet's `persistentId` (survives code change) vs `sessionId` (code hash) | instance id and `SAVE_FORMAT` exist; pack-version change on resume is refused or accepted case by case (e.g. IL-e's version bumps refuse old saves) | `docs/PACKAGE_FORMAT.md`, `persistence/src/format.rs` | **P2** — needs an `ARC-` record and a short design (save compatibility policy) |
| PA-4 | **Per-component replication declared in the contract**: each component type states whether and to whom it is disclosed | Decentraland `syncEntity(entity, [componentIds])` | disclosure is decided per system in code (`perceive`/disclosure functions; ARC-82 door sign) | `contracts/src/component.rs`, `docs/NETWORKING.md` | **P2** — design first; touches a public contract |
| PA-5 | **Stable keys for authored entities, separate from runtime-spawned ones** | Decentraland scene-start entity ids | authored keys exist (`ARC-71`, `ARC-77`); the reserved split for spawned entities is not written down | `contracts/src/ids.rs`, `worldpack/` | **P3** — verify, then document |
| PA-6 | **Hard budgets at validation, soft budgets as warnings** for Presentation Pack assets (size, files, triangles, textures) | Decentraland deploy-time hard caps vs runtime soft caps | S23's package budget (`scripts/package.py budget`, step-23 §7.1) checks bundles, not packs | `mineworld packs validate`, `docs/PACKAGE_FORMAT.md`, `docs/VISUAL_FIDELITY.md` | **P2** — fits S23/R-b |
| PA-7 | **Pack permissions declared and checked at load** | Decentraland `requiredPermissions` | the World Interaction List governs actions per section (`ARC-63`, `ARC-65`); no load-time declaration of what a pack may do | `worldpack/src/requirements.rs`, `ARC-63` | **P3** — design when third-party packs grow |
| PA-8 | **Watch mode for data packs** (re-validate on save, hot reload in the server) | Decentraland preview loop | `mineworld validate` is one-shot | `tools/cli` | **P3** — creator experience |
| PA-9 | **Server roles written down** (world server, cognition workers, asset store) so a later split needs no kernel change; **content-addressed assets** | Vircadia domain server / assignment clients; hashed assets | `NETWORKING.md` §7 mentions world workers; asset store unspecified | `docs/NETWORKING.md` §7, `DEP-8` | **P3** — documentation |
| PA-10 | **Controller made of small typed parts** (persona, memory view, choice prompt, parser), with the model's free text treated as a proposal the server validates | Concordia components; its Game Master, inverted | P6 not yet designed; P4 memory and P5 backends merged | `step-17-cognition.md` P6 design | **P1 for P6's design** |
| PA-11 | **One retrieval-count knob** beside the 6 KB context budget | AI Town `NUM_MEMORIES_TO_SEARCH` | P4 bounds bytes and lines | P6 persona configuration | **P2** — inside P6 |

Explicitly **not** adopted, with the rule each would break:

- Runtime attach/detach of behaviour code (Croquet Cards/Behaviors) — Systems are installed at build
  time and simulation extensions are sandboxed (`ARC-8`, `ARC-33`, `DEP-12`); direct calls between
  behaviours contradict single ownership (CLAUDE.md §4.1).
- Client-side authority or per-client sync (Croquet, Decentraland default) — server authority, INV-9.
- A model as referee (Concordia's Game Master) — the world must stay valid with models removed
  (VISION) and only Systems mutate state.
- Hosted character engines as controllers (Inworld, Convai) — offline by default, no paid model in any
  gate, recorded replay required (`ARC-58`).
- Silent denial of an unauthorised action (Vircadia legacy) — every refusal is typed
  (`NETWORKING.md` §5).
- Untyped message buses (Decentraland `MessageBus`, deprecated upstream) — strong typing at boundaries.

## 6. Effect on the plan

1. **No change to MVP-0's scope or order.** Nothing audited replaces a merged or frozen MineWorld
   module; the kernel, persistence, server, packs and clients stay as designed.
2. **PA-1 and PA-2** are small, well-specified follow-ups (lint and one test) that can be scheduled now.
3. **PA-10 and PA-11** go into S10 P6's design, which has not been written yet (it waits for the
   operator's local-model spike, QB-15). P6's design must cite this audit and state why Concordia is
   reference-only.
4. **PA-3, PA-4 and PA-6** each need a short design and an operator ruling before code, because they
   touch save compatibility, a public contract, or release budgets.
5. **PA-5, PA-7, PA-8 and PA-9** are recorded as later improvements; none blocks MVP-0.
6. `REUSE_POLICY.md` §19's list "worth auditing next" is updated: Concordia and AI Town are now
   audited; AgentSociety and SimWorld remain.

## 7. Limits of this audit

- The sources were read through a summarising web fetch, not by cloning and running the code; items
  marked *unverified* in the detailed audits (for example Croquet's behaviour sandboxing, Inworld's
  licence and Character Studio status, Decentraland's custom-component sync) must be read at source
  before any design relies on them.
- Maturity facts (last commit, releases) are as seen on 2026-10-11 and age quickly.
- No prototype was built; per `REUSE_POLICY.md` §15 a prototype would be required before adopting any
  substantial dependency, and none is proposed.

**Status:** informational reference audit (REUSE_POLICY §19), 2026-10-11. **Not authoritative architecture**: it changes no MineWorld contract; where it disagrees with a specification, the specification governs. Gathered by a research agent from primary web sources through a summarising fetch tool (no clone); items marked *unverified* must be re-read at source before any design cites them. Summary and rulings: [`PRIOR_ART_2026-10.md`](PRIOR_ART_2026-10.md).

# Prior-art audit: Vircadia (and its High Fidelity heritage) against MineWorld

**Status:** research finding, read-only. Not a design decision. Any adopt/reject outcome must be recorded in
[`docs/DECISIONS.md`](/Users/yuema137/MineWorld/docs/DECISIONS.md) per [`docs/REUSE_POLICY.md`](/Users/yuema137/MineWorld/docs/REUSE_POLICY.md).
**Access date for all web sources:** 2026-10-11 (the date of this audit).
**Method and limits:** Sources were read through a summarising fetch tool, not by cloning the repositories. Several
GitHub pages were truncated or lacked the fields asked for (last-commit date, contributor count, licence of some repos).
Every claim marked **(unverified)** needs a source read before it is relied on. Nothing below was run.

---

## 1. Summary

- **Vircadia has two lineages and they must not be conflated.** The legacy C++ stack (`vircadia-native-core`,
  "Interface" client plus domain server and assignment clients) is a fork of High Fidelity's 2014-era code. Its README
  now says it is deprecated in favour of **Vircadia World**. The current effort is `vircadia-world`: a TypeScript/Bun
  server over PostgreSQL, Apache-2.0, with a "Gen 2" redesign in 2025.
- **Vircadia World is early and not a fit to adopt.** Its README says it is not ready for production until version
  1.0.0. The most recent visible commit on `master` is 16 Jan 2026. The TypeScript SDK has 3 stars and says it is
  "in the early stages of development, and subject to change".
- **The architecture is closest to MineWorld's *shape* in one respect only:** a single server owns state and clients
  talk to it over WebSocket and REST. Almost everything else diverges: authority is a database (Postgres rows are the
  state), there is no journal or event-sourced history, and the legacy stack has no per-observer perception model.
- **Scripting in both lineages is a trust-the-script model.** Interface scripts and server entity scripts mutate
  entities directly, with domain-level permissions as the only documented guard. We found no documented sandbox. This is
  the opposite of MineWorld's `ActionIntent` rule (`docs/NETWORKING.md` §2, INV-5/6/9).
- **Ideas worth taking are small and architectural:** role separation of server processes (the domain-server /
  assignment-client split), asset serving as a separate content-addressed service, observable permission denials for
  scripts, and a single CLI-driven self-host path. None requires a dependency.
- **Recommendation for the operator (not decided here):** do not adopt Vircadia or any part of it. Record the rejection
  in `docs/DECISIONS.md` as the REUSE_POLICY requires, citing this file.

---

## 2. Facts

### 2.1 Repositories, licences, maintenance

| Repo | Role | Language | Licence | Stars (access date) | Last visible activity | Source |
|---|---|---|---|---|---|---|
| `vircadia/vircadia-world` | Current platform: "fast reactivity layer for games" | TypeScript | Apache-2.0 | 44 | Latest commit on master 16 Jan 2026 (digisomni); 35 commits 5–16 Jan 2026; bursty | [repo](https://github.com/vircadia/vircadia-world), [commits](https://github.com/vircadia/vircadia-world/commits/master) |
| `vircadia/vircadia-world-sdk-ts` | TypeScript SDK for the World API | TypeScript | Apache-2.0 (per LICENSE, sidebar) | 3 | Last commit date not shown; 460 commits on master | [repo](https://github.com/vircadia/vircadia-world-sdk-ts) |
| `vircadia/vircadia-metaverse` | "Metaverse server for Vircadia" (Feathers, Node, Docker Compose) | TypeScript | **not shown** (no licence badge in capture) | 58 | last commit date not shown; 758 commits; 37 open issues | [repo](https://github.com/vircadia/vircadia-metaverse) |
| `vircadia/vircadia-native-core` | **Legacy** C++ Interface, domain server, assignment clients | C++ | Apache-2.0 (badge; file `LICENSE`) | 542 | README: "deprecated… see Vircadia World"; most recent commit 1 Jul 2025 (digisomni), prior 2024; 85,863 commits | [repo](https://github.com/vircadia/vircadia-native-core), [commits](https://github.com/vircadia/vircadia-native-core/commits/master) |
| `vircadia/vircadia-web` | Browser web interface | TypeScript | not shown | 76 | last update 15 Dec 2025 (from org listing) | [org](https://github.com/vircadia) |
| `vircadia/vircadia-web-sdk` | Browser SDK for the platform | JavaScript | Apache-2.0 | 37 | last update 4 Dec 2025 (from org listing) | [org](https://github.com/vircadia) |
| `vircadia/vircadia-unity-sdk` | Pinned Unity (C#) SDK | C# | not shown | 12 | not shown | [org](https://github.com/vircadia) |

**Contributors:** not visibly counted on any repository page we could read. The commit lists show a small core
(`digisomni` authors most recent commits; `Vircadia` org account; `namark` in early 2024; dependabot). **(unverified: the
true contributor count.)**

**Releases:** `vircadia-native-core` has tagged releases, the latest shown as **Vircadia 2024.1.0 "Atlas"** (marked Latest),
preceded by 2022.1.x "Selene", 2021.1.x "Eos" and 2020.3.x "Demeter" alpha builds
([releases](https://github.com/vircadia/vircadia-native-core/releases)). The Releases section of `vircadia-world` and
`vircadia-world-sdk-ts` was empty in the capture, so **no tagged release of the current platform was found**.

**Status statement, verbatim paraphrase from `vircadia-world` README:** the project "is a work in progress and is not yet
ready for production use until the version becomes 1.0.0 or higher unless you have direct support from the Vircadia team."
([README](https://raw.githubusercontent.com/vircadia/vircadia-world/master/README.md)).

**Branching:** `vircadia-world` merges updates into a `next` branch before `master` (same README).

**Legacy deprecation:** `vircadia-native-core` About text: "Please see Vircadia World for the current development."
([repo](https://github.com/vircadia/vircadia-native-core)).

### 2.2 Vircadia World architecture (current)

- README architecture diagram (only source for architecture; no prose description on the page): an **API container**
  ("World API Manager", WebSocket and REST); a **State container** ("World State Manager", "Ticks & Entity Lifecycle");
  a **PGWeb** admin container; a **Postgres** database holding "All State & Auth". Clients "run with no container" and connect
  to the API over WebSocket/REST. Deployment through a **CLI** on Docker Compose; Kubernetes optional
  ([README](https://raw.githubusercontent.com/vircadia/vircadia-world/master/README.md)).
- Repo top-level folders: `cli`, `client`, `sdk`, `server`. `server/` contains `module/`, `service/`, `README.mdx`
  and a configuration file configured by environment variables ([server dir](https://github.com/vircadia/vircadia-world/tree/master/server)).
- Tech stack: **Bun** on the server and SDK; `bun install @vircadia/world-sdk` ([SDK](https://github.com/vircadia/vircadia-world-sdk-ts)).
- Press/Reddit summary of "Gen 2" (2025): PostgreSQL as the core, Bun as the wrapping server; "in-game entities are
  managed through SQL" and worlds can simulate agentic behaviour
  ([Postgres Weekly #600](https://postgresweekly.com/issues/600) and a Reddit post cited via the secondary search;
  **(unverified: primary post not fetched)**).
- **Not found:** the sync protocol (how clients receive entity deltas: polling, LISTEN/NOTIFY, or WebSocket push), conflict
  handling, auth model, row-level-security rules, or the scripting model. The SDK README documents only installation and
  build/publish steps ([SDK](https://github.com/vircadia/vircadia-world-sdk-ts)). **(unverified: all of these.)**

### 2.3 Vircadia Metaverse server

- Feathers.js framework, services generated by its CLI; Docker Compose self-host with `.env` and `docker compose up -d`;
  an admin account is granted via `ADMIN_ACCOUNT`; SMTP for email; optional Ethereum DLT ("not a required component")
  ([README](https://github.com/vircadia/vircadia-metaverse)). It handles accounts, admin and domain registration with the
  web client. It is not the entity/world-state server. Its relationship to `vircadia-world` is **not stated** in the capture.

### 2.4 Legacy lineage (native core, High Fidelity heritage)

- Server roles (from the Overte documentation, which is a fork of the same code, so **(unverified for Vircadia)**):
  the **domain server** is a single executable that manages domain settings (audio spatialisation, user permissions,
  running scripts) and "hands out simulation assignments" to connecting Interface clients; it starts **assignment
  clients** of six roles: avatar mixer, audio mixer, entity server, asset server, agent (runs user JavaScript), messages
  mixer. Roles can run on several machines ([Overte architecture](https://docs.overte.org/en/latest/explore/get-started/architecture.html)).
- Entity server: tracks entities and properties; the Overte page says changes are relayed to clients in the domain and that
  "each Interface runs its own physics engine" with the entity server coordinating the results
  ([same page](https://docs.overte.org/en/latest/explore/get-started/architecture.html)).
- High Fidelity 2014 design post (heritage only): domain server as "master server"; nameserver for text-name discovery;
  voxel server with a sparse voxel octree; audio mixer that produces listener-relative stereo; avatar mixer relaying
  expressions and body movement; assignment server matching devices to domains; marketplace and currency server. The
  post says nothing about who owns entity authority ([HiFi blog, 24 Apr 2014](https://www.highfidelity.com/blog/high-fidelity-system-architecture-f30a7ba89f80)).
- Licence of the legacy domain server, audio/avatar mixers, voxel server and interface: **Apache-2.0** per the 2014 post
  and the native-core badge.

### 2.5 Scripting and sandboxing (legacy)

- Four script contexts per the Overte documentation ([get-started](https://docs.overte.org/en/latest/script/get-started-with-scripting.html)):
  Interface scripts (run while the client is open); assignment-client scripts (run after the client closes); client
  entity scripts and server entity scripts attached to entities. Server entity scripts "act on everyone in the domain
  without a user trigger".
- Permissions: domain owners can restrict create/edit rights; a script without permission has its changes silently not
  appear, though it still shows in the Running Scripts window (same page).
- **No sandbox is documented** in any source read. Scripts are JavaScript with access to the entity API.
  **(unverified: whether the current Vircadia World has any scripting or sandbox at all.)**

### 2.6 Avatars and assets

- Legacy formats: **FBX, glTF and OBJ** listed on a SourceForge mirror; a 2021 release note mentions glTF vertex-colour
  fixes ([SourceForge mirror](https://sourceforge.net/mirror/vircadia/), [2021 Eos notes](https://planet.freegamedev.net/2021/04/10/2021-1-1-eos-released/)).
  Secondary sources only; **(unverified against current source).**
- Avatar mixer tracks location, avatar choice and movement (HiFi post and Overte page). Avatar entities can be allowed or
  restricted per domain (2021 release note).
- Asset server "functions like a Web server" and serves models, audio and scripts (Overte page). Vircadia repos show
  Git LFS for large assets (vircadia-world README). A separate `vircadia-ipfs-repo` lists IPFS hashes of pinned content
  (org page; purpose inferred from name, **(unverified)**).
- Full-body avatar rigs, skeletons and retarget rules: **not found** in any source read.

### 2.7 Interest management and what each client receives

- Overte page: entity changes go to all clients currently in the domain. No relevance filter by view or distance is
  described. The only distance-dependent behaviour mentioned is the audio mixer's listener-relative mix.
- High Fidelity 2014 post: voxel server nests, and distant groups of voxels are averaged for rendering (rendering LOD,
  not per-observer simulation filtering).
- **Vircadia World:** no statement found about per-client subscriptions or scopes. **(unverified.)**

### 2.8 Self-hosting

- Vircadia World: clone with submodules, Git LFS pull, configure env vars, `CLI` install and deploy; Docker Compose,
  optional Kubernetes ([README](https://raw.githubusercontent.com/vircadia/vircadia-world/master/README.md)).
- Metaverse server: Docker Compose, `.env`, admin account ([README](https://github.com/vircadia/vircadia-metaverse)).
- Legacy: run `domain-server`, then an `assignment-client` connecting on default port 40102 ([HiFi/Overte docs,
  secondary](https://docs.overte.org/en/latest/explore/get-started/architecture.html)).

### 2.9 Security model for user scripts

- Legacy: domain-level permissions for scripts and entity edits (Overte page above). No sandbox, capability model or
  resource limit was found.
- Current: **not found.**

---

## 3. Concept map

| Concept | Vircadia (current: vircadia-world) | Vircadia (legacy: native-core) | MineWorld (source) |
|---|---|---|---|
| Authoritative state | Postgres rows ("All State & Auth"); State container runs ticks | Domain server + entity server; Interface runs local physics | Single headless `World`; kernel owns state; "server-authoritative" (`docs/NETWORKING.md` §1) |
| Persistence | PostgreSQL, current state, no documented event log | Domain-server settings and entity store (not described in detail) | SQLite: facts + journal + snapshots; restart = snapshot + journal replay (`persistence/src/lib.rs`, ARC-25) |
| Transport | WebSocket + REST to API container | UDP/custom per-role protocols (unverified) | WebSocket frames + HTTP control plane (`server/src/app.rs`, `server/PROTOCOL.md`, DEP-3 axum) |
| Client-to-server writes | Unknown whether clients may write rows (**unverified**) | Interface sends entity edits; permission-checked | Intents only: `ActionIntent`; "asserting state" is a protocol violation (`docs/NETWORKING.md` §2) |
| Presence / avatar | Not found | Avatar mixer: location, avatar, movement | Person entities; seats bind who drives each Person (`server/src/seats.rs`) |
| Who drives an actor | Not found | Interface user, or agent (script server) | `free / hosted / connected / held` seat states (`server/src/seats.rs`) |
| Per-observer view | Not found | Not described; all clients get domain entities | `PerceptionContext` / `PerceivesNothing`; observation scope enforced server-side (`server/src/perception.rs`, INV-13) |
| Scripting | Not found in capture | Interface, agent, client-entity and server-entity scripts (JS) | Controllers propose `ActionIntent`s; Systems validate and resolve (`docs/ARCHITECTURE.md`) |
| Audio | Not found | Audio mixer, listener-relative stereo | Not in MVP (no audio contract found in scope of this audit) |
| Assets | Git LFS; IPFS pin list (inferred) | Asset server (models, audio, scripts) | glTF / GLB canonical (DEP-7); asset sourcing DEP-8 |
| Avatar format | Not found | FBX / glTF / OBJ (secondary) | glTF; humanoid profile is a runtime contract, not appearance (`docs/DECISIONS.md` DEP-8 area, `docs/HUMANOID_PROFILE.md`) |
| Deployment | CLI + Docker Compose (+ optional K8s) | Domain server + assignment clients on several hosts | Single `mineworld server` binary; local, LAN, VPS (`docs/NETWORKING.md` §§6–7) |
| Language | TypeScript on Bun | C++ (+ JavaScript scripts) | Rust kernel and server; Godot clients (DEP-4, DEP-9) |

---

## 4. "Are we reinventing?"

Per REUSE_POLICY we must ask both questions: are we reinventing a mature wheel, and are we forcing a wheel that does not fit?

**Reinventing: little.** The pieces MineWorld builds are not commodity wheels Vircadia offers in a reusable form:

- **Per-observer perception** (`server/src/perception.rs`, `docs/ARCHITECTURE.md` §6, INV-13). Vircadia has no equivalent
  that we could find. Nothing to reinvent against; this is a differentiator.
- **Event-sourced persistence with replay** (`persistence/src/`, ARC-25). Vircadia stores state in Postgres rows. We
  should not adopt that model: it gives no history and no byte-exact replay.
- **Seats and takeover** (`server/src/seats.rs`, `server/PROTOCOL.md` §4.2). Legacy "agent" and Interface-user roles are
  informal by comparison.

**Reinventing: commodity layers, where we already use or should use libraries.** These are correctly reused per DEP-2 and
DEP-3 (`rusqlite`, `axum`, `tokio`, `serde`). Vircadia's use of Postgres, Bun and Feathers shows the same categories
(database, HTTP/WebSocket server, service framework), not anything MineWorld should copy.

**Forcing a wheel that does not fit: the risk is on the Vircadia side if we adopt it.** Specifically:

- Adopting `vircadia-world` would put Postgres rows at the centre of world state and a TypeScript/Bun runtime under the
  simulation. That contradicts DEP-9 (Rust core), DEP-2 (SQLite local-first), and ARC-25 (journal is the authority).
- Adopting the legacy stack would bring a C++ Interface and a scripting model without a sandbox, against INV-5/6/9.

**Where MineWorld could be re-inventing an area Vircadia has already tried:** Asset distribution (a separate asset
server, content hashing). MineWorld currently has asset sourcing rules (DEP-8) and no asset server. If an asset server
is ever needed, the general pattern is mature (HTTP object storage) and need not be Vircadia-specific. Evaluate against
`docs/NETWORKING.md` §7 (object storage) first.

**Concrete MineWorld files where the comparison applies:**

| MineWorld file / module | What it does | Vircadia comparison |
|---|---|---|
| `server/src/host.rs`, `server/src/hosted.rs` | Server host and in-server controllers | Vircadia: domain server + assignment clients (role split) |
| `server/src/seats.rs` | Who drives each Person | Vircadia: assignment to agents; no binding state machine found |
| `server/src/perception.rs` | Per-observer observation seam | No Vircadia equivalent found |
| `server/src/admission.rs` | Invite, nickname, resume secret | Vircadia: account service (Metaverse) is separate; no analogue in world server |
| `persistence/src/sqlite.rs`, `persistence/src/replay.rs` | Facts, journal, snapshots, replay | Vircadia: Postgres state rows, no replay |
| `server/PROTOCOL.md` | Client frames, JSON | Vircadia: WebSocket/REST; wire format not verified |
| `clients/2d`, `clients/3d-spike` (Godot) | Presentation | Vircadia: Unity SDK (pinned) and a web client; not comparable directly |

---

## 5. Ideas worth adopting

Each idea is a pattern, not a dependency. Each maps to a MineWorld artefact.

1. **Separate server roles, possibly on several hosts (domain server / assignment client split).** The legacy design lets one
   coordinator start role processes (avatar, audio, entity, asset, script) as needed and place them on different machines.
   MineWorld's `NETWORKING.md` §7 already plans "many independent World Workers" for scale. Idea: write the role boundary
   (world server, cognition workers, asset store) explicitly in `docs/NETWORKING.md` §7 and `docs/ARCHITECTURE.md`, so a later
   split does not need a kernel change (INV-14). No code now.
2. **Asset server as a content-addressed service.** Vircadia pins assets by IPFS hash (inferred from repo name). MineWorld
   could key assets by hash in the Asset Storage box of `docs/NETWORKING.md` §7, so the same asset is never duplicated across
   worlds. Touches `docs/DECISIONS.md` DEP-8 and the asset loader only.
3. **Observable denial for controllers and scripts.** Vircadia's silent-failure behaviour (a script without permission has its
   edits quietly ignored) is what MineWorld must not copy. Keep the reverse: every rejected `ActionIntent` returns a typed
   result (`NETWORKING.md` §5 action result: accepted / rejected / ActionUnavailable). Already the design; the audit confirms
   it is the right call.
4. **Per-domain toggles for avatar content.** The legacy "allow or restrict avatar entities per domain" switch is a world-level
   policy knob. MineWorld analogue: a World Pack or System Pack setting, not a kernel flag. Candidate for `docs/MODULE_SPEC.md`
   pack configuration, not for the kernel.
5. **Single CLI as the self-host path.** Vircadia World and the legacy stack both lead with a CLI or Docker Compose. MineWorld
   already specifies `mineworld server ./worlds/<name> --listen` (`NETWORKING.md` §6). Idea: keep the same single binary for
   local, LAN and VPS (`NETWORKING.md` §8 parity), and add a Compose file only when the VPS path is implemented.
6. **Spatial listener-relative audio mix.** Not an MVP item. If audio is later added, the pattern (mix by listener position) belongs
   in a Presentation Pack, not in the simulation. Record as a future note in `docs/VISUAL_FIDELITY.md` or a new ticket.
7. **Git LFS for large assets in a repo.** Commodity practice; MineWorld should decide its asset-repo policy in DEP-8 when it
   reaches real assets. Not a Vircadia-specific adoption.

Items 1, 2, 4 and 5 are documentation-level or small, and need no code now. None should appear in an open PR without a design
document per `docs/MODULE_SPEC.md` and the Structured Coding workflow.

---

## 6. Why not adopt as a dependency

**Maturity**

- Current platform (`vircadia-world`): self-declared pre-1.0, not for production; 44 stars; no tagged release found; bursty
  commit history with the latest visible commit on 16 Jan 2026 (as of the 11 Oct 2026 access date, roughly nine months with
  no visible commit on the page we read: **(unverified: the page may be a cache; check the branch directly)**).
- SDK (`vircadia-world-sdk-ts`): self-declared early and subject to change; 3 stars.
- Legacy (`vircadia-native-core`): explicitly deprecated; last visible commit 1 Jul 2025; latest release 2024.1.0.
- Metaverse server: 58 stars; licence not shown in capture.

**Licence**

- Apache-2.0 for the current platform, SDK and legacy core: permissive, compatible with MineWorld's licence stance.
  The metaverse server's licence must be confirmed from its `LICENSE` file before any use. **(unverified.)**

**Architecture fit (blocking)**

- State model: Postgres rows as authority versus MineWorld's journal and facts (ARC-25, INV-11). Adopting would
  replace the persistence model, not plug into it.
- Language and runtime: TypeScript/Bun versus the Rust kernel mandated by DEP-9. A Rust wrapper around a Bun service
  would put a second process and a second source of truth inside every world.
- Authority and client writes: the stated model (clients and the API write through the State container) must be checked
  against `NETWORKING.md` §2, which forbids clients asserting state. We could not verify the write path.
- Scripting: no sandbox documented; scripts mutate entities directly. This conflicts with INV-5, INV-6 and INV-9 and with
  the Controller rule that controllers propose, never mutate.
- Perception: no per-observer model. MineWorld's `PerceivesNothing` default (`server/src/perception.rs`) would have
  nothing to map onto.
- Avatars: FBX-first legacy formats versus glTF canonical (DEP-7) and the humanoid profile (`docs/HUMANOID_PROFILE.md`).
- Dependency-level reuse of a mature wheel is not warranted: Vircadia does not offer a Rust library, an event store, or a
  perception component to reuse.

**Process requirement.** Rejecting a substantial dependency requires a short DECISIONS record (REUSE_POLICY §§11–12).
Recommended text: "Vircadia (vircadia-world, legacy native-core) rejected as a dependency: pre-1.0 and deprecated lineage;
Postgres-authoritative state and direct entity mutation conflict with ARC-25, INV-5/6/9 and DEP-9; TypeScript/Bun runtime
conflicts with DEP-9. Adopted ideas: role separation and content-addressed assets, as patterns only." Operator decision.

---

## 7. Open questions

1. **Vircadia World sync protocol.** How do clients receive entity updates: polling, Postgres LISTEN/NOTIFY, or WebSocket push?
   What is the tick model? Read `vircadia-world/server/module` and `service` directly.
2. **Client write path.** Can an SDK client insert or update rows directly, or only submit requests through the API?
   This decides whether it violates `NETWORKING.md` §2.
3. **Auth and permissions.** Is access controlled by Postgres row-level security, by the API, or both? Where is the admin
   role (PGWeb) enforced?
4. **Scripting in vircadia-world.** Is there any server-side scripting at all, and if so is it sandboxed?
5. **Avatar format in vircadia-world.** Does the Gen 2 platform carry avatars at all, and in which format?
6. **Interest management.** Does vircadia-world send each client all entities, or filtered by scope? The legacy stack sends all
   domain entities to every client (Overte page); confirm for the current stack.
7. **Metaverse server licence and relation to vircadia-world.** The capture showed no licence and no link. Confirm both.
8. **Current activity.** The 16 Jan 2026 last-visible commit is from the commit-list page as fetched. Confirm the latest
   commit date and contributor count from the GitHub API (`gh api repos/vircadia/vircadia-world/commits?per_page=1`), which
   this audit did not run.
9. **Gen 2 primary source.** The Reddit announcement and the Postgres Weekly summary were not read directly. Fetch the
   primary post for the design intent.
10. **Overte as a fork.** Overte documentation describes the shared HiFi lineage. Confirm which statements apply to Vircadia
    native-core and which are Overte-only before quoting them as Vircadia behaviour.
11. **Whether a Vircadia-compatible asset or avatar pipeline is useful** to MineWorld users (FBX import would be a DEP-8
    question, not an architecture one).

---

## Sources (all accessed 2026-10-11)

- https://github.com/vircadia (organisation listing; 10 of 60 repos visible)
- https://github.com/vircadia/vircadia-world
- https://raw.githubusercontent.com/vircadia/vircadia-world/master/README.md
- https://github.com/vircadia/vircadia-world/commits/master
- https://github.com/vircadia/vircadia-world/tree/master/server
- https://github.com/vircadia/vircadia-world-sdk-ts
- https://github.com/vircadia/vircadia-metaverse
- https://github.com/vircadia/vircadia-native-core
- https://github.com/vircadia/vircadia-native-core/commits/master
- https://github.com/vircadia/vircadia-native-core/releases
- https://docs.overte.org/en/latest/explore/get-started/architecture.html (Overte fork, heritage)
- https://docs.overte.org/en/latest/script/get-started-with-scripting.html (Overte fork, heritage)
- https://www.highfidelity.com/blog/high-fidelity-system-architecture-f30a7ba89f80 (HiFi, 24 Apr 2014)
- https://postgresweekly.com/issues/600 (secondary, Gen 2 summary)
- https://sourceforge.net/mirror/vircadia/ (secondary, formats)
- https://planet.freegamedev.net/2021/04/10/2021-1-1-eos-released/ (secondary, 2021 release notes)
- https://github.com/vircadia/vircadia-docs-sphinx/blob/master/docs/source/script/write-scripts.md (scripting example, not read in full)

MineWorld sources read: `docs/NETWORKING.md` §§1–7; `server/PROTOCOL.md` (header and §§1, 4.2 by reference);
`server/src/perception.rs` (header and context); `server/src/seats.rs` (header); `persistence/src/lib.rs` (header);
`docs/DECISIONS.md` DEP-2, DEP-3, ARC-25, ARC-27 (headers and key lines); `docs/REUSE_POLICY.md` (header).

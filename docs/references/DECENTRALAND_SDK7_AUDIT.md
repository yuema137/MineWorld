**Status:** informational reference audit (REUSE_POLICY §19), 2026-10-11. **Not authoritative architecture**: it changes no MineWorld contract; where it disagrees with a specification, the specification governs. Gathered by a research agent from primary web sources through a summarising fetch tool (no clone); items marked *unverified* must be re-read at source before any design cites them. Summary and rulings: [`PRIOR_ART_2026-10.md`](PRIOR_ART_2026-10.md).

# Prior art: Decentraland SDK7 (ECS scenes)

Status: research note, read-only audit. Access date for all web sources: 2026-10-12.
Scope: Decentraland SDK7 (entities, components, systems, custom components, Creator Hub, scene packaging and deployment, scene permissions and limits, multiplayer sync, glTF pipeline) compared against MineWorld.
Not a design decision. Any adoption or rejection that changes architecture needs a `DECISIONS.md` record (`docs/REUSE_POLICY.md`, `ENGINEERING_STANDARDS` §16).

Citation convention: `[S#]` refers to the source list at the end of §2. Items marked **UNVERIFIED** could not be confirmed from primary sources in this session.

---

## 1. Summary

- **What it is.** Decentraland SDK7 is an Entity-Component-System scripting model for scenes that run inside Decentraland's client (a Unity/Babylon-style runtime embedded in the Explorer). The toolchain is an Apache-2.0 monorepo [S1][S2]. A scene is TypeScript code plus a `scene.json` manifest and glTF/GLB assets, deployed to a content server [S5][S6].
- **ECS rules are close to ours in spirit, but the ownership rule is absent.** Components are pure data and entities are bare IDs [S3]. Systems are free functions run each tick, and the docs describe no concept of a system owning a component [S4]. MineWorld's INV-7 (only the owning system mutates a component) is stricter than anything SDK7 states.
- **Custom components are typed by schema, not by a versioned contract.** `engine.defineComponent(name, schema, defaults)` with `Schemas.*` types [S3]. No schema versioning or migration story is documented in the pages reached. MineWorld's `ComponentSchemaVersion` and `ComponentDeclaration` are more explicit [M1].
- **Multiplayer is per-client and not server-authoritative by default.** Each client runs the scene. Shared state is declared with `syncEntity(entity, [componentIds], entityEnumId)`, and each client calls it on its own [S7]. Stronger authority requires a self-hosted Multiplayer Server guarded by `isServer()` [S7]. This is the largest divergence from MineWorld's server-authoritative rule (`docs/NETWORKING.md`, `CORE_CONCEPTS` INV-7 / `ENGINEERING_RULES`).
- **Scene manifest is a platform manifest.** `scene.json` holds parcels, spawn points, permission strings, media host allow-lists, feature toggles, skybox, and a wallet/creator address [S5]. It is not a general pack manifest. It has no licence field, no dependency/requires resolution, and no pack identity; `ARC-53`/`ARC-54` cover what MineWorld needs here [M2][M3].
- **Limits are concrete and per-parcel.** Triangles n×10,000, entities n×200, meshes n×300, materials and textures on a log2 curve; 300 MB total file cap, 50 MB per file, 200 files per parcel; 330 m height [S8]. Limits are split into deploy-blocking caps and soft warnings in Creator Hub [S8].
- **Creator Hub is a desktop authoring tool.** It offers drag-and-drop scene editing, Smart Items (prebuilt behaviours), a catalog of optimized assets, preview, debug, and publish [S9]. It is tied to the platform's publishing model (LAND or a Decentraland NAME) [S9][S8].
- **Testing is manual.** The reached docs describe preview (`npm run start` or Creator Hub), hot reload, console, and a debug panel. No headless or automated scene-test runner was documented in the pages reached [S10].
- **Verdict.** Do not adopt it as a dependency. It is a platform SDK bound to a renderer, a wallet/land economy, and a closed-ish content network. Its useful ideas are: data-only components with a typed schema registry, explicit per-entity sync declarations, a manifest that declares its permissions up front, per-parcel resource budgets as first-class validated numbers, and deployment-time (not runtime) enforcement of size caps.

---

## 2. Facts (with sources)

### 2.1 Licence, maintenance, repository

| Fact | Value | Source |
| --- | --- | --- |
| Licence of `decentraland/js-sdk-toolchain` | Apache License, Version 2.0 (the LICENSE file's first line reads "Apache License"; the repo page labels it Apache-2.0) | [S1], [S2] |
| Repo description | "Toolchain to build JS & TS scenes for Decentraland"; README: "core packages for Decentraland's SDK and development tools" | [S1] |
| Popularity / size seen on repo page | 40 stars; 1,111 commits; 112 pull requests (open count as shown) | [S1] |
| Last commit date, latest release tag/date, open-issue count | **UNVERIFIED** (not shown in fetched page; Releases section appeared empty) | [S1] |
| Maintenance judgement | **UNVERIFIED** beyond "actively documented in 2026". Re-check with `gh api repos/decentraland/js-sdk-toolchain` before any reliance | — |

### 2.2 ECS model (SDK7)

- Entities are bare identifiers with no properties or methods; components reference them [S3].
- Components are "pure data": they "can only store this data, they can't modify this data themselves" [S3].
- Changes are made by systems, which are "decoupled from entities and components" [S3][S4].
- An entity may hold at most one component of each kind; `createOrReplace` overwrites [S3].
- Entity identity: numeric ID (IDs 0–511 reserved by the engine), name for Scene Editor entities via `engine.getEntityOrNullByName()`, and reserved aliases `engine.RootEntity`, `engine.PlayerEntity`, `engine.CameraEntity` [S3].
- Systems: `engine.addSystem(fn, priority?, name?)`, called every tick; the same function cannot be registered twice; `engine.removeSystem()` removes one [S4].
- Ordering: higher `priority` runs first; default priority is 100000; equal-priority order is unspecified [S4].
- Timing: the SDK does not enforce a fixed tick rate; the frame rate is capped at 30 ticks per second; `dt` is the actual elapsed time and the engine does not correct earlier frames [S4].
- Systems mutate components with `Transform.getMutable(entity)` (example) [S4]. No formal mutation rule is stated.
- **No component-ownership concept is documented.** Multiple systems may act on one entity and "don't even need to know about each other" [S4]. The docs do not state whether a component may be written by more than one system.

### 2.3 Component schema and custom components

- `engine.defineComponent(name, schema, defaults?)` creates a custom component. Define it outside `main()`, ideally in a `/components` folder [S11].
- Schema types: `Schemas.Boolean`, `Schemas.Float`, `Schemas.String`, `Schemas.Map()`, `Schemas.Array()` (read-only via `get()`; `getMutable()` to change), `Schemas.Optional()`, `Schemas.EnumString`, `Schemas.EnumNumber`, `Schemas.OneOf()` (tagged by `$case`) [S11].
- Defaults: optional third argument covers some or all fields; unspecified fields are zero-like (`0`, `false`, `''`) [S11].
- Serialization: **not described** in the page reached [S11].
- Versioning and migration of custom component schemas: **not documented** in pages reached (UNVERIFIED that none exists elsewhere).
- Cross-client sync of custom components: **not stated** on the custom-components page [S11]. Sync-by-component-ID in `syncEntity` suggests custom components can be listed, but this is inferred, not confirmed [S7].

### 2.4 Scene manifest (`scene.json`)

Fields documented on the scene-metadata page [S5]:

- `navmapThumbnail` (path or URL; 16:9, ideally 1920×1080)
- `tags` (up to 3 from: art, game, casino, social, music, fashion, crypto, education, shop, business, sports, parkour)
- `rating` (`"A"` for adults; `"R"` applied by moderators only)
- `featureToggles` (`voiceChat: "disabled"`, `portableExperiences: "enabled" | "disabled" | "hideUi"`; enabled unless set off)
- `creator` (Ethereum address for tips)
- `spawnPoints` (`name`, `position`, optional `default`, `cameraTarget`; regions via two-number arrays)
- `requiredPermissions` (strings such as `ALLOW_TO_MOVE_PLAYER_INSIDE_SCENE`, `OPEN_EXTERNAL_LINK`; documented as applying only to portable experiences and smart wearables)
- `allowedMediaHostnames` (legacy; needed only with `ALLOW_MEDIA_HOSTNAMES`)
- `scene` (`parcels`, `base`; max 300×300 parcels)
- `skyboxConfig.fixedTime` (seconds since midnight, 0–86400)
- `landscapeTerrain` (boolean, default true; single-scene Worlds only)
- `worldConfiguration` (`name`, optional `fixedAdapter`, optional `placesConfig.optOut`)
- Warning on the page: do not add custom fields to `scene.json` [S5].

Title, description and contact are set in the Scene Editor's menu, not in documented `scene.json` keys (the page did not name them) [S5].

### 2.5 Permissions and sandbox

- Scenes declare permissions in the manifest and call `RestrictedActions` so the runtime performs an action "if authorized" [S12].
- The reached pages did **not** list the full permission enumeration, grant UX, fetch/WebSocket restrictions, or memory caps. The `scene-limitations` page reached was a 404; the `optimizing/scene-limitations` page was referenced but not retrieved [S13]. **UNVERIFIED** beyond what is listed here.
- Media host policy: legacy `allowedMediaHostnames` with the `ALLOW_MEDIA_HOSTNAMES` permission [S5].

### 2.6 Resource limits (per parcel, n = parcel count)

From the scene limitations page [S8]:

- Triangles: n × 10,000
- Entities: n × 200
- Meshes ("bodies"): n × 300
- Materials: log2(n+1) × 20
- Textures: log2(n+1) × 10
- Height: 330 m max; above 200 m "may cause multiplayer sync issues"
- Total file size: 15 MB per parcel, capped at 300 MB
- File count: 200 per parcel
- Max single file: 50 MB
- File-count and size limits are enforced at deployment and block it. The other limits are soft limits reported as warnings by Creator Hub [S8].
- Textures: power-of-two dimensions 1–1024 px; larger are scaled down; 1024×1024 recommended [S8].
- Worlds storage: a shared budget per NAME (100 MB) + 100 MB per LAND parcel + 100 MB per 2,000 MANA held; ENS-domain Worlds capped at 36 MB [S8]. This is explicitly platform-economy-tied.
- Parcel is 16 m × 16 m; content outside parcel bounds is not rendered; players cannot go below y = 0 [S8].
- Mobile: Draco-compressed glTF needs asset optimization; aim well below limits on mobile [S8].

### 2.7 Creator Hub, editor, assets, deployment

- Creator Hub is a desktop app and "the recommended tool for creators of all knowledge levels," covering wearables, emotes and scenes [S9].
- Visual scene editing via drag-and-drop; code editing for full control; Smart Items are prebuilt models with behaviour that creators can customize and attach to their own models [S9].
- Asset catalog: default 3D assets "already optimized for Decentraland"; creators may import models made in Blender or other tools, within size limits [S9].
- Preview, debug, edit code and publish are in one loop [S9].
- Publishing: LAND in Genesis City (buy adjacent 16×16 m parcels in the Marketplace), or Decentraland Worlds (own a Decentraland NAME). No token is needed to start building [S9].
- Scene packaging: the SDK builds a scene bundle and the content server stores entity files, with hashes; the exact deployment entity format is documented under the contributor "scenes" entity type (not fetched in full) **UNVERIFIED** for format details.

### 2.8 Multiplayer and state sync

- Built-in sync is by marking entities: `syncEntity(entity, [componentIds], entityEnumId?)`, or a Multiplayer component in Creator Hub. Only listed components are shared; "changes in the environment aren't shared between players by default" [S7].
- `entityEnumId`: entities created at scene start need a manually assigned, consistent ID. Dynamically created entities get automatic IDs. Enum IDs above 8001 can collide with Creator Hub Smart Items [S7].
- Parenting for synced entities uses `parentEntity()`, not `Transform.parent`, because local IDs can differ between players [S7].
- `MessageBus` (`emit` / `on`) is flagged **deprecated** in the SDK; the docs recommend entity sync instead. Senders also receive their own messages [S7].
- Authority: the page does not describe a host or first-player authority. Each client runs the scene and calls `syncEntity` on its own. For a single source of truth, use a Multiplayer Server, where only the server calls `syncEntity`, guarded by `isServer()` [S7].
- State is not persisted when all players leave the scene [S7].
- Players outside scene parcels are out of sync while outside. `isStateSyncronized()` gates interactions until synced [S7].
- External servers: a self-hosted server may synchronize data over REST or a secured WebSocket (`wss`); Colyseus is recommended for shared state [S14].
- Custom-component sync: inferred possible via `syncEntity` component IDs, not confirmed [S7].
- Authoritative spec for the built-in model: the `networking.md` page reached only as a one-line summary; `network-connections` and `serverless-multiplayer` gave the detail above [S7][S14].

### 2.9 Testing and preview

- Preview: `npm run start` in the CLI, or preview from Creator Hub (console in its own window). Hot reload on save; manual **Reload Scene** or `/reload` [S10].
- Debug: console (`console.log`, errors in red), debug panel (scene info, memory, FPS, web requests) [S10].
- Multiplayer local testing: a second explorer window pointed at a local realm with `multi-instance=true` [S10].
- Dependency update: `npm i @dcl/sdk@latest` [S10].
- Automated or headless test runner: **not documented** in the pages reached (`scenes-sdk7/testing` was a 404). **UNVERIFIED** that none exists.

---

## 3. Concept map

| Decentraland SDK7 | MineWorld equivalent | Fit | Notes |
| --- | --- | --- | --- |
| Entity (bare numeric ID, optional name) | `Entity` (`contracts/src/entity.rs`, `ids.rs`) | Close | MineWorld IDs are typed; Decentraland uses plain numbers and reserves 0–511. |
| Component (pure data, one per kind per entity) | `Component` trait, `ComponentDeclaration`, `ComponentRecord` (`contracts/src/component.rs`) | Close | MineWorld adds an owner type, `ComponentSchemaVersion`, and serialized records. |
| System (`engine.addSystem`, per-tick function, priority) | `System` (`kernel/src/system.rs`, `kernel/src/system/`, `systems/*`) | Partial | Decentraland systems are untyped free functions with no owned state; MineWorld systems are registered with ownership and dependencies. |
| Component owner / single writer | `C::OWNER`, `kernel/src/components.rs`, INV-7, `ARC-26` | **Divergent** | Decentraland states no ownership. This is the most important difference. |
| `defineComponent(name, schema, defaults)` | Component declaration + macro (`kernel/src/macro_support.rs`, `contracts/src/component.rs`) | Partial | Both typed; MineWorld has schema versions, Decentraland does not document any. |
| `Schemas.*` types (Map, Array, Optional, Enum, OneOf) | Named domain types in contracts (`ENGINEERING_STANDARDS` §3, rule 7) | Close in spirit | MineWorld forbids untyped metadata at boundaries; Decentraland schema is typed but not versioned. |
| `syncEntity(entity, [componentIds], enumId)` | Replication of owned state (`docs/NETWORKING.md`, `kernel/src/snapshot.rs`) | **Divergent** in authority | Decentraland: client-run, per-client declaration. MineWorld: server-authoritative state-delta replication. |
| MessageBus (deprecated) | `ActionIntent` / `Event` (`contracts/src/action.rs`, `event.rs`) | Divergent | MineWorld's message path is typed, authoritative, and validated; SDK7 `emit`/`on` is untyped and sender-echoing. |
| Multiplayer Server + `isServer()` | Authoritative server (`docs/NETWORKING.md`) | Close | Decentraland's server mode is optional; MineWorld's is the only mode. |
| `scene.json` (manifest) | `world.yaml` (World Pack), `pack.yaml` (`docs/PACKAGE_FORMAT.md` §5.0, `worldpack/src/format.rs`) | Partial | scene.json carries platform fields (wallet, rating, Worlds budget); MineWorld pack.yaml carries identity, requires, licence. |
| `requiredPermissions` / `RestrictedActions` | Interaction list `ARC-63`/`ARC-65`; permission checks in the owner | Partial | Decentraland: manifest-declared flags. MineWorld: section-level, owner-typed. |
| Per-parcel limits (triangles, entities, files) | No equivalent yet; closest is item/entity-count budgets in `systems/*` | Gap | A budget vocabulary could be added as a System Pack concern, not a kernel one. |
| Creator Hub (drag-and-drop, Smart Items, asset catalog) | Presentation Packs (`presentation/mineworld-default`); no editor | Gap | MineWorld has no authoring tool; `ARC-70` covers client text, not authoring. |
| Asset catalog + glTF optimization | Presentation Pack assets (`presentation/mineworld-default/3D`, `docs/ART_DIRECTION.md`, `docs/VISUAL_FIDELITY.md`) | Close | Both rely on glTF/GLB; neither enforces a Draco policy. |
| LAND / NAME publishing | None | Not applicable | Platform economy; MineWorld must not depend on it. |
| Scene preview / hot reload | Godot client plus headless server; `docs/ACCEPTANCE.md` | Partial | MineWorld's headless tests are stronger; no preview loop documented. |

---

## 4. Are we reinventing?

Short answer: **partly**, and the parts we are reinventing are the parts a mature library covers. The parts we are not reinventing are the ones SDK7 cannot provide.

### 4.1 Where MineWorld is already stronger (do not replace)

- **Single-writer ownership** (INV-7, `docs/CORE_CONCEPTS.md` §13.1; `kernel/src/components.rs`; `kernel/src/access.rs`). SDK7 has none. Keep it.
- **Server authority and headless simulation** (`docs/NETWORKING.md`, `docs/ENGINEERING_RULES.md`). SDK7's per-client sync cannot satisfy this; do not copy it.
- **Typed pack identity, requirements and licence policy** (`ARC-53`, `ARC-54`, `ARC-55` in `docs/DECISIONS.md`; `worldpack/src/requirements.rs`). SDK7 has no equivalent.
- **Extension by installable System Packs from another repository** (`ARC-66`, `systems/installed/`). SDK7 extension means editing the scene.

### 4.2 Where we might be reinventing commodity pieces

- **Typed component schema encoding** (`contracts/src/component.rs`: `Component: Serialize + DeserializeOwned`, `ComponentRecord<Vec<u8>>`). Serialization is commodity. Check `serde` plus a compact format (e.g. `postcard`, `rmp-serde`, or `bincode`) against `docs/REUSE_POLICY.md` before any further hand-rolled encoding. **UNVERIFIED** whether the current code already does this optimally; I did not audit the serializer choice.
- **Schema validation for YAML/JSON packs** (`worldpack/src/format.rs`, `worldpack/src/read.rs`). Commodity: `serde` + `serde_yaml` (or the successor) plus a validation layer. Check whether the hand-rolled checks duplicate what `serde`'s `deny_unknown_fields` already gives.
- **Asset budgets and limit checks** (none yet). When added, do not invent a numeric budget vocabulary in the kernel; SDK7's per-parcel formula (n × constant, log2 curves) is a useful reference for the numbers, but it is platform-specific.

### 4.3 Where MineWorld is reinventing something the kernel should not own

- Nothing in the kernel should be reinvented from SDK7. SDK7's features (Smart Items, LAND, Worlds budgets, Creator Hub) are platform concerns that belong outside the kernel, if at all.

### 4.4 Concrete files to check when acting on this note

| MineWorld file / module | Relevance |
| --- | --- |
| `contracts/src/component.rs` | Component trait, `ComponentDeclaration`, `ComponentSchemaVersion`. Compare with SDK7 `defineComponent`. |
| `kernel/src/components.rs`, `kernel/src/access.rs` | Ownership enforcement. SDK7 lacks it. |
| `kernel/src/system.rs`, `kernel/src/system/` | System registration; SDK7 `addSystem` priority model has no ownership. |
| `kernel/src/snapshot.rs`, `docs/NETWORKING.md` | State replication; compare SDK7 `syncEntity`. |
| `worldpack/src/format.rs`, `worldpack/src/load.rs`, `worldpack/src/requirements.rs` | Pack/manifest shape and requirement resolution. Compare SDK7 `scene.json`. |
| `entities/modern-goods/pack.yaml`, `entities/modern-goods/items/*.yaml` | Entity Pack data; per-item typed files, an analogue of SDK7 Smart Items (no behaviour in data). |
| `presentation/mineworld-default/3D`, `presentation/mineworld-default/2D` | Presentation asset layout; compare glTF pipeline norms. |
| `docs/PACKAGE_FORMAT.md` §5.0, §8 | Pack format and licence policy. |
| `docs/DECISIONS.md` `ARC-53`, `ARC-54`, `ARC-55`, `ARC-66`, `ARC-71`, `ARC-77` | Pack identity, requirements, licence, external packs, entity packs, composition. |
| `docs/DECISIONS.md` `ARC-63`, `ARC-65` | Interaction list and consequence routing. Compare SDK7 `requiredPermissions` and `RestrictedActions`. |

---

## 5. Ideas worth adopting (design ideas only)

Each idea is mapped to a MineWorld file or PR. "Adopt" here means "consider the idea", not "take the code".

1. **Explicit per-entity sync declaration.** SDK7's `syncEntity(entity, [componentIds])` makes replication opt-in and visible, so unsynced state is an explicit choice. MineWorld's replication should state, per component type, whether it replicates to Observations. Map: `docs/NETWORKING.md` (replication section) and `kernel/src/snapshot.rs`. Suggested home: a contract-level replication flag on `ComponentDeclaration` (`contracts/src/component.rs`), reviewed under the observation-scope rule in `docs/NETWORKING.md`. Not a PR yet.

2. **Stable, declared entity enum IDs for scene-start entities.** SDK7 asks creators to assign consistent IDs to scene-start entities and separates them from dynamic IDs (with a reserved range). MineWorld already has typed IDs (`contracts/src/ids.rs`). A deterministic ID scheme for World Pack-declared entities, distinct from runtime-spawned ones, would prevent cross-run ID drift. Map: `worldpack/src/entities.rs`, `contracts/src/ids.rs`. Possible home: the step that allocates world-pack item and entity keys (`ARC-71`, `ARC-77`).

3. **Manifest-declared permissions, checked at load time.** SDK7 lists `requiredPermissions` in the manifest and the runtime refuses unauthorized actions. MineWorld's interaction list (`ARC-63`, `ARC-65`) already routes this per section. A load-time check that a world's enabled packs declare every permission they use would make refusal earlier and clearer. Map: `worldpack/src/requirements.rs` plus `ARC-63`.

4. **Deployment-time vs runtime budget split.** SDK7 blocks deploy on hard file caps (size, count) and only warns on soft caps (triangles, entities). MineWorld could mirror this for Presentation Pack assets: hard limits in `mineworld packs validate`, soft limits as warnings in the client. Map: `docs/PACKAGE_FORMAT.md` §8 (validation) and `docs/VISUAL_FIDELITY.md`. Not a PR yet; needs a `DECISIONS.md` entry before implementation.

5. **Per-unit budget formulas, not fixed numbers.** SDK7 scales limits with parcel count (n×10,000, log2 curves). If MineWorld ever enforces asset or entity budgets per region or per place, a scaled formula is more honest than a constant. Map: `docs/VISUAL_FIDELITY.md`; no kernel change.

6. **Declare a world's "features" as an explicit toggle set.** SDK7 `featureToggles` (`voiceChat`, `portableExperiences`) is a plain, typed on/off object in the manifest. MineWorld's `configure/` plus `ARC-61` (per-world system config) already do this for systems. No change needed; confirm the toggle vocabulary is typed. Map: `worldpack/src/configure.rs`, `ARC-61`.

7. **Spawn points as named, typed data.** SDK7 `spawnPoints` supports a named default and a camera target. MineWorld's `spatial.rs` (`contracts/src/spatial.rs`) and Place/Entity Pack data could carry named spawn anchors in a typed form. Map: `contracts/src/spatial.rs`; `docs/ARCHITECTURE.md` (spatial model). Careful: the spatial model must stay semantic (`ENGINEERING_RULES` §§5–6, rule 14); a spawn point is a semantic anchor, not render geometry.

8. **Creator-facing preview loop with hot reload for data packs.** SDK7's hot reload on save and a console panel is a good developer-experience target for Entity Packs and World Packs (no rebuild needed per `ARC-71`). Map: `mineworld packs validate` (`ARC-71`) plus a watch mode in the CLI. Not a PR yet.

9. **Entity-as-ID, components-as-data separation.** The SDK7 framing "an entity is nothing more than an id" is already MineWorld's rule (`docs/CORE_CONCEPTS.md` §3). Use it as an example in onboarding docs; no code change.

Not recommended for adoption: `MessageBus` (deprecated upstream and untyped), `allowedMediaHostnames` (legacy), Smart Items (a platform authoring abstraction that would leak behaviour into data), and the LAND/NAME economy.

---

## 6. Why not adopt as a dependency

1. **Platform-bound.** The SDK targets the Decentraland Explorer runtime, its renderer, and its content servers. MineWorld is headless and renderer-independent (`docs/ENGINEERING_RULES.md`, `ENGINEERING_STANDARDS` rule 10). Adopting it would put a renderer into the simulation contracts, violating rule 3.
2. **Authority model conflict.** SDK7 sync is client-run per default; MineWorld is server-authoritative with one path for single- and multi-player (`docs/NETWORKING.md`, rule 3). Forcing MineWorld into `syncEntity` would break INV-7 and the event-sourced path.
3. **No single-writer rule.** Adopting SDK7's component model means accepting systems that write any component. That is the opposite of MineWorld's kernel guarantee (`kernel/src/components.rs`).
4. **Economy and identity coupling.** LAND, NAMEs, MANA, wallet addresses (`creator`), Worlds storage budgets and the moderation `rating` are part of the platform, not the framework. MineWorld must not depend on any of them (`docs/VISION.md`, rule 2).
5. **Language and runtime.** The SDK is TypeScript on a JS runtime. MineWorld's kernel is Rust; cognition is Python; clients are Godot. The TypeScript layer would be a second runtime inside the server.
6. **Licence is fine, the coupling is not.** Apache-2.0 is compatible with `ARC-55`'s allow-list, so licence is not the blocker. The blocker is architectural. Per `REUSE_POLICY.md`, rejecting a substantial dependency still needs a `DECISIONS.md` record; this note is an input to that record, not the record itself.
7. **Maintenance and documentation gaps.** Latest release and last-commit date were not verified (§2.1), and several pages the docs point to returned 404s during this audit. Treat the platform's docs as moving.

Conclusion: study, do not depend. The design ideas in §5 are the value.

---

## 7. Open questions

1. **Authority of the built-in sync.** The docs reached do not state whether the first player, a scene host, or each client is authoritative for `syncEntity` state. Verify against the Decentraland Explorer source or with a two-client experiment (`debugging/debug-in-preview` describes a multi-instance setup). **UNVERIFIED.**
2. **Custom-component sync.** Does `syncEntity` accept custom component IDs, and how are they serialized? Not confirmed (§2.3, §2.8).
3. **Component serialization format.** Not documented in pages reached (§2.3). Check the SDK's runtime package for the wire format before any claim.
4. **Permission enumeration.** The full list of `requiredPermissions` values, grant UX, and fetch/WebSocket rules were not retrieved (§2.5). Fetch the contributor `scene-runtime/permissions` page via its full `.md` link, or `docs/llms-full.txt`.
5. **Scene packaging and deployment entity format.** The content-server entity schema (hashes, `content` array, `metadata`) was not fetched (§2.7). Relevant if MineWorld ever needs a content-addressed world-pack format.
6. **Scene-limitations currency.** The per-parcel numbers in §2.6 come from `development-guide/sdk7/scene-limitations/` [S8]. The `optimizing/scene-limitations` page was linked but not fetched. Re-verify the numbers before quoting them as current.
7. **Maintenance.** Last commit date, latest release, open issues, and maintainer count (§2.1). Check with `gh api`.
8. **Whether MineWorld already serializes components in a compact binary format.** Not audited. Check `contracts/src/component.rs` and `kernel/src/snapshot.rs` before §4.2 can be closed.
9. **Who owns a scene's authored content at deploy.** The docs say content is published to LAND or a NAME (§2.7). MineWorld's equivalent is the World Pack and its `requires:` set. A decision is needed on whether a world's packs are ever published to a registry (`ARC-77` leaves registry out of MVP-0). Not a question for this note.
10. **Which Decentraland doc URLs are current.** Several guessed paths returned 404 and the sitemap listed different ones. The URLs cited in §2 were the ones that loaded (`[S3]`–`[S14]`). Re-check before any citation is reused.

---

## Sources

Access date for every entry: 2026-10-12.

- [S1] https://github.com/decentraland/js-sdk-toolchain (licence, description, counts)
- [S2] https://github.com/decentraland/js-sdk-toolchain/blob/main/LICENSE (Apache License, Version 2.0)
- [S3] https://docs.decentraland.org/creator/scenes-sdk7/architecture/entities-components.md (entities, components, pure data, IDs)
- [S4] https://docs.decentraland.org/creator/scenes-sdk7/architecture/systems.md (addSystem, priority, timing, mutation example)
- [S5] https://docs.decentraland.org/creator/scenes-sdk7/kinds-of-projects/scene-metadata.md (scene.json fields)
- [S6] https://docs.decentraland.org/creator/scenes-sdk7/networking.md (summary only; sub-pages used)
- [S7] https://docs.decentraland.org/creator/scenes-sdk7/networking/serverless-multiplayer.md (syncEntity, enum IDs, MessageBus deprecated, authority, limits)
- [S8] https://docs.decentraland.org/creator/development-guide/sdk7/scene-limitations/ (per-parcel limits, file caps, Worlds storage, textures, parcel size; this URL returned the numbers in §2.6)
- [S9] https://docs.decentraland.org/creator/ (Creator Hub, drag-and-drop, asset catalog, publishing)
- [S10] https://docs.decentraland.org/creator/scenes-sdk7/debugging/debug-in-preview.md (preview, hot reload, debug panel, multi-instance)
- [S11] https://docs.decentraland.org/creator/scenes-sdk7/architecture/custom-components.md (defineComponent, Schemas.*, defaults)
- [S12] https://docs.decentraland.org/contributor/scene-runtime/permissions.md (permissions, RestrictedActions; partial)
- [S13] https://docs.decentraland.org/creator/scenes-sdk7/optimizing/scene-limitations.md (linked; not fully retrieved)
- [S14] https://docs.decentraland.org/creator/scenes-sdk7/networking/network-connections.md (external server, wss, Colyseus)
- [M1] `/Users/yuema137/MineWorld/contracts/src/component.rs` (MineWorld component trait and declaration)
- [M2] `/Users/yuema137/MineWorld/docs/PACKAGE_FORMAT.md` §5.0 (pack fields, licence policy)
- [M3] `/Users/yuema137/MineWorld/docs/DECISIONS.md` (ARC-53, ARC-54, ARC-55, ARC-63, ARC-65, ARC-66, ARC-71, ARC-77)

**Unverified items list (for reviewers):** repo last-commit date and release (§2.1); full `requiredPermissions` enumeration and fetch rules (§2.5); custom-component sync and serialization (§2.3, §2.8); built-in sync authority (§2.8); deployment entity format (§2.7); whether SDK7 has any headless test runner (§2.9).

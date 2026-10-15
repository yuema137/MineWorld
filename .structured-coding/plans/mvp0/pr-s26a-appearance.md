# PR S26-a — The `appearance` System Pack and the cast binding contract

## DESIGN FROZEN 2026-10-14 (primary; operator Q-S26a-2)

```text
Design revision:        revision 2 (2026-10-14): revision 1 (PR #195, commit f7be4bd) with §14.1's rulings
                        recorded and the audit re-anchored on origin/main @ 9aa185b
Author:                 S26-a PR-design agent, worktree /Users/yuema137/mineworld-worktrees/design-s26a,
                        branch docs/s26a-design, planning base origin/main @ 3ada60a (#193); re-audited on
                        origin/main @ 9aa185b (DL-a #191, FG-a #188, 25b #190, SET-b #192 merged)
Approved by / evidence: the operator answered Q-S26a-2 directly in the primary session on 2026-10-14
                        ("藏掉，顺手改"); the primary session ruled Q-S26a-1, -3, -4, -5 and froze this
                        design itself (§14.1)
Implementation base:    origin/main at C0 (C0 records the commit)
Start condition:        this design frozen; nothing else (step-26 §12.1: no RL-b2, R-a or 16c dependency)
Execution contract:     §12, filled at drafting, in effect from this freeze
Lifecycle:              FROZEN. Implementer: Haiku (step-26 §12.1), one session, worktree impl-s26a; Opus
                        review
```

Scope (§2.1), invariants (§6), acceptance (§8) and the commit plan (§9) are frozen at freeze. Progress,
evidence, findings and bounded corrections stay writable (§15).

*Superseded header:* `DRAFT — PR DESIGN, NOT FROZEN`, revision 1.

**Effort:** `mvp0` · **Step:** [`step-26-cast.md`](step-26-cast.md) (STEP DESIGN FROZEN 2026-10-14,
operator Q-S26-1…12) §§2 (A-14…A-18, A-21, A-29), 4.1–4.4, 4.6, 8, 9, 12 (the S26-a row), 12.1, 12.2
(AC-S26-10), 13 (R-S26-5, R-S26-9), 15 (ARC-91, ARC-92), 16.1 · **Parent:** [`overall.md`](overall.md) ·
**Pattern:** `systems/naming` (S8 PR 10c, `ARC-31`) and P6a of
[`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) §§9.0, 9 (A0–A3), 12.1 · **Followed by:**
S26-cast (look keys), S26-c2 (3D resolution), S26-e (2D resolution).

**Decision numbers.** `ARC-91` (appearance is a perceivable trait owned by the `appearance` System Pack;
`look` an opaque disclosed key) and `ARC-92` (the cast binding lives in the Presentation Pack; one
resolution order; the extras fallback; amends `ARC-46`'s role list) were assigned to S26-a at the step
freeze (step-26 §15, §16.1). No new number is needed.

---

## 0. Binding inputs, exactly restated

| Source | Ruling |
| --- | --- |
| step-26 §4.2, Q-S26-11 (ARC-91) | `section appearance` (a person file's `appearance: { look: alice-moreau }`); `emits appeared` at genesis from the section, **public**; `owns Look` (`look`), reduced from `appeared` by this pack alone; `discloses Look` to every observer who perceives the person; `depends on nothing` |
| step-26 §4.2 | A `look` is an opaque lowercase slug `[a-z0-9-]{1,48}`. It is **not** a mesh, file, texture, colour, height or any renderer concept. No descriptors (age, stature, build, skin, hair) in the world |
| step-26 §4.2 | Change amplification: one crate, one installed line, one entry in each world's `systems:` list, one `appearance:` line per person file; no edit to `Person`, the kernel, the protocol, the loader or any other System |
| step-26 §4.2 | The cognition perceived-entity formatter must not present the slug as a name: S26-a checks `cognition/` and either omits `look` or labels it |
| step-26 §4.2 | A world without the pack is unchanged; a world with the pack and a person without the section gives that person no `look` |
| step-26 §4.3, §4.4 (ARC-92, SD-S26-2) | 3D `presentation/mineworld-default/3D/characters/cast.yaml` (`format`, `looks`, `extras`, `default`), YAML's JSON-compatible subset; 2D `look:<key>` roles in the `people` set, asked first, then `person:<n>` by hash, then plain drawing; the fallback is never a cruder model class |
| step-26 §12 S26-a row | Scope: the crate on the `naming` template; `systems/installed`; `appearance:` lines and `systems:` entries in `market-town`, `social-cafe`, `lakeside`; golden fixtures that move; the `cast.yaml` format (spec + empty-but-valid file); `clients/2d/PRESENTATION.md` §3.1 `look:<key>` role; the cognition check; `ARC-91/92` |
| step-26 §12 S26-a row | Checkpoint: `mineworld server worlds/social-cafe` and a scripted client observation shows `look` on every named person; `bodies-yard` shows none; a save/restart keeps it |
| step-26 §12 S26-a row | Adversarial: X-1 an invalid slug is refused at the section's line; X-2 a world with the section but without the pack in `systems:` is refused by name; X-3 a world without the pack loads with unchanged digests; X-4 `look` is not rendered as a name by the perceived formatter |
| step-26 §12.1 | Starts when this design is frozen; nothing else. Implementer Haiku, from a commit-level plan mirroring `naming`; Opus review |
| step-26 R-S26-5 | Golden digests move when the section adds genesis facts; S26-a updates them in one reviewed commit; X-3 proves worlds without the pack are unchanged |
| Q-S26-4 | `bodies-yard`: no `appearance` section and no characters; it exercises the fallback |
| Q-S26-9 | Appearance follows the Person: Vera is the visitor, Wes the wanderer |
| Operator, 2026-10-14 (keys) | Meshy and OpenAI only by sourcing `~/.config/mineworld/secrets.env` inside a calling command, never reading a key. **S26-a makes no such call at all** |
| Platforms (operator, 2026-10-08) | macOS, Linux and Windows |

## 1. Goal, in one paragraph

Give every person in the three named worlds a perceivable, System-owned appearance key, and fix the
contract by which a Presentation Pack turns that key into a character or a sprite, before any character
exists. A new `appearance` System Pack, built file for file on `naming`, owns a person file's
`appearance:` section, states a public `appeared` fact at genesis, reduces it into the person's `look`
component and discloses it to everyone who perceives the person. The three named worlds enable it and
author one look per person (22 distinct keys); `bodies-yard` does not, and so draws its walkers by the
fallback. The default 3D Presentation Pack gains an empty-but-valid `characters/cast.yaml` with its
specification, the 2D pack specification gains the `look:<key>` role, and `ARC-91` and `ARC-92` record
both halves. No client resolves a look yet (S26-c2, S26-e), no character is generated (S26-b, S26-d),
and nothing in the kernel, the contracts, the protocol, the server, the loader or any other System
changes.

## 2. Scope

### 2.1 In scope

1. `systems/appearance/` — the crate `mineworld-appearance`: section, `LookKey`, `Look` component,
   `appeared` event, public disclosure, its own tests and a short README (§5.1).
2. Installation: one line in `systems/installed/Cargo.toml`, one in `systems/installed/src/lib.rs`, one
   workspace dependency in the root `Cargo.toml`; `Cargo.lock` as Cargo writes it.
3. Worlds: one `systems:` entry and one `appearance:` line per person in `worlds/social-cafe` (12),
   `worlds/market-town` (12, values equal to social-cafe's), `worlds/lakeside` (10); one line in each
   world's README that lists its systems.
4. Tests whose literals move because of item 3 (§3.2 F-S26a-3), updated in the same commit; new tests for
   X-1, X-2, X-3, X-4 and the integration checkpoint (§7).
5. `scripts/baselines.txt`: the three worlds re-recorded with reason lines; `bodies-yard` unchanged (§10).
6. The cognition check (§5.5): a test that the memory renderer never presents a look as a name; no
   cognition code change (Q-S26a-4).
7. The 2D HUD never shows a look as raw text (§5.6, Q-S26a-2): `clients/2d/scripts/hud/readers.gd`
   `SHOWN_ELSEWHERE` gains `"look"`; the stub panels test pins it.
8. The cast binding contract (§5.7): `docs/DECISIONS.md` `ARC-92` and a note on `ARC-46`;
   `clients/2d/PRESENTATION.md` §3.1; a new specification `clients/3d-spike/PRESENTATION.md`;
   `docs/MODULE_SPEC.md` §6.1; the file `presentation/mineworld-default/3D/characters/cast.yaml`; its
   structural checker `scripts/check_cast.py`, run by the `fast` layer (Q-S26a-3).
9. `docs/MODULE_SPEC.md` §4.1 (the eighth section) and `docs/DECISIONS.md` `ARC-91`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Any character, reference image, brief, GLB, texture, Meshy or OpenAI call | S26-b1, S26-b2, S26-cast, S26-d |
| `character.yaml` (`ARC-93`), `Human.Body` removal, the 3D client reading `cast.yaml` | S26-c1, S26-c2 |
| The 2D client asking for `look:<key>`; any `look:` binding in `asset_bindings.yaml`; sprites | S26-e |
| A resolution check in `check_cast.py` (every id has `characters/cast/<id>/character.yaml`) | S26-c1 extends the checker |
| A memory renderer for `appeared` (ambient salience) | not proposed (Q-S26a-4); P6b may revisit |
| `bodies-yard` | untouched (Q-S26-4) |
| Opaque look codes instead of name slugs | R-S26-9: a later one-file-per-world migration |
| `systems/README.md`'s stale pack list (F-S26a-6) | reported to the primary session; not fixed here |

### 2.3 Dependencies

None. `origin/main @ 9aa185b` holds everything S26-a needs. Concurrent PRs that touch the same files are
in §13 (R-S26a-1) and Q-S26a-5.

## 3. Audit (`origin/main @ 3ada60a`, anchors re-checked on `9aa185b`)

### 3.1 Source anchors

| # | Fact | Evidence |
| --- | --- | --- |
| S-1 | `naming` is seven source files: `lib.rs` (34 lines), `codec.rs` (24), `component.rs` (36), `event.rs` (50), `name.rs` (78), `section.rs` (36), `system.rs` (111); one test file (249); `Cargo.toml` (25); `README.md` (25) | `wc -l systems/naming/**` |
| S-2 | `naming`'s section is a scalar (`type Authored = Name`); its seed refuses a non-person subject with `Rejection::PreconditionFailed` | `systems/naming/src/section.rs:11-35` |
| S-3 | The genesis fact is `Visibility::Public`, `.about([person])`, `.with_participants([person])` | `systems/naming/src/event.rs:44-49` |
| S-4 | The component serializes as an object (`{ "name": … }`) because `observation.gd` reads only object payloads; `owned_component!` with `component_type`, `schema_version = 1` | `systems/naming/src/component.rs:9-36` |
| S-5 | `discloses` returns the component to every observer who perceives the subject; `react` inserts the component on its own fact only | `systems/naming/src/system.rs:55-93` |
| S-6 | A mapping section with `#[serde(deny_unknown_fields)]` is the `bodies` precedent | `systems/bodies/src/section.rs:28-58` |
| S-7 | `AuthoredSection::Authored: DeserializeOwned + Debug + Send + Sync + 'static`; decoding it is the owner's validation, located by the loader | `authoring/src/section.rs:172-203` |
| S-8 | The installed set: `installed!` list `systems/installed/src/lib.rs:38-60` (last line `Fishing => acme_fishing::FishingSystem,`); dependencies `systems/installed/Cargo.toml:15-31`; a test holds the two equal | `systems/installed/tests/installed.rs` |
| S-9 | `systems/*` is a workspace member glob; workspace dependencies list `mineworld-naming = { path = "systems/naming" }` | `Cargo.toml:2-21,38` |
| S-10 | Sections are seeded after passages and arrivals, per file in composition order (the `systems:` order) | `worldpack/src/load.rs:344-366`; `ARC-31` point 4 |
| S-11 | A section whose owner is not enabled is `PackError::ContentNeedsASystem { subject, content, system, path }`; an invalid one is `PackError::Malformed` with the owner's message and `line L column C` | `worldpack/tests/refusals.rs:717-831`; `worldpack/src/error.rs:369` |
| S-12 | **AC-1 check 3**: Market Town's `systems:` must *begin with* Social Café's list, in order, then the six market packs, then only `calendar`/`weather`; each Social Café key of a person file must be present in Market Town with an equal value | `tests/acceptance/tests/ac1_composability.rs:1031-1080` (`compare_systems`), `1158-1185` (`compare_content`) |
| S-13 | Systems lists: social-cafe `presence … naming, schedule` (7); market-town the same 7, then `item, inventory, item-transfer, economy, employment, consumption, calendar, weather`; lakeside the same 7, then `item, inventory, item-transfer, consumption, fishing`; bodies-yard `presence, movement, conversation, bodies` | `worlds/*/world.yaml` |
| S-14 | People: social-cafe and market-town the same 12 keys and names; lakeside 10; bodies-yard 12 unnamed | `worlds/*/people/`; step-26 A-21 |
| S-15 | The memory renderer: a registered plug-in per event type; every other type, `generic`, renders only `"<event type> at <place>, with <people>"` from the envelope, never the payload | `cognition/lm-controller/src/mineworld_cognition/memory/render/__init__.py:92-103,118-145` |
| S-16 | The rule controller reads `display-name` only | `cognition/rule-controller/src/lib.rs:218-260` |
| S-17 | P6b's `Situation` (frozen, unmerged) renders labels from `display-name` and affordances, no generic component | `pr-s10-p6-lm-controller.md` §5.2.2 |
| S-18 | **The 2D HUD shows every own component without a reader raw**, in an "other" panel, except `SHOWN_ELSEWHERE = ["display-name", "participation", "passages"]` | `clients/2d/scripts/hud/readers.gd:9-16,77-104` |
| S-19 | The stub panels test (AC-I12) feeds own components and asserts the panels; `#[ignore]`d, needs Godot 4.7 | `tools/cli/tests/client_2d_interact_stub.rs:288-413` |
| S-20 | 2D roles: `player`, `person:<n>`, `facade:*`, `prop:*`, `texture:*`; "The client never asks for a role by a world key … only by tag, index or hash" | `clients/2d/PRESENTATION.md:69-81` |
| S-21 | `ARC-46`'s role list; it has no notes yet | `docs/DECISIONS.md:3802-3834` |
| S-22 | `MODULE_SPEC.md` §4.1 "MVP-0 has seven sections", the table from `name` to `body` | `docs/MODULE_SPEC.md:720-775` |
| S-23 | `MODULE_SPEC.md` §6.1 names `characters/character_style.yaml`, not `characters/cast.yaml`; the 3D pack has `pack.yaml`, `manifest.yaml`, `ART_DIRECTION.md`, `CHARACTER_IDENTITY.md`, `FIDELITY_REVIEW.md`, `references/`, `candidate/` and no `characters/` | `docs/MODULE_SPEC.md:1344-1380`; `ls presentation/mineworld-default/3D` |
| S-24 | The 3D client has no Presentation Pack specification (`clients/3d-spike/` holds `README.md`, `ASSETS.md`) | `ls clients/3d-spike/*.md` |
| S-25 | Baselines: 5 keys per world (`summary-300`, `summary-1000`, `summary-300s`, `facts-300s`, `journal-300s`), reason lines kept by `baseline write`; read by the nightly only | `scripts/baselines.txt:1-33`; `scripts/ci_parity.py:33-37,733-760` |
| S-26 | A headless run's summary prints `day N revision R facts F`, `facts <type> <count>` per type and `history <n> facts, fingerprint …` — so every summary key moves when genesis gains facts | `tools/cli/src/run.rs:533-575` |
| S-27 | `fast` layer commands are a list in `scripts/ci_layer.py:48-62` | source |

### 3.2 Findings

- **F-S26a-1 — market-town cannot append `appearance` at the end.** AC-1 check 3 (S-12) requires Market
  Town's `systems:` to begin with Social Café's exact list. With `appearance` appended to Social Café
  after `schedule`, Market Town must carry it **right after `schedule`**, before `item`. This is safe:
  `appearance` subscribes only to its own `appeared`, so no other system's reduction order moves; only
  the per-file seeding order (S-10) places `appeared` between `routine-assigned` and the market packs'
  facts, which shifts event ids (they shift anyway). Q-S26a-1 asks the primary session to confirm this
  reading of the step's "one appended entry".
- **F-S26a-2 — the 2D HUD would print `look: alice-moreau`.** Without item 7 the observer's own `look`
  reaches the "other" panel as raw JSON (S-18): a slug shown as text, X-4's defect in the client. The
  sanctioned exit is `SHOWN_ELSEWHERE`, whose comment already covers "shown elsewhere or not at all".
- **F-S26a-3 — pinned literals that move.** Found by search; each is updated in C4 with a comment:

  | File:line | Literal today | After C4 |
  | --- | --- | --- |
  | `worldpack/tests/social_cafe.rs:66-77` | systems list ending `Capability::Schedule,` | `…, Capability::Schedule, Capability::Appearance,` |
  | `worldpack/tests/social_cafe.rs:218-223` | `first.genesis().len()` = `53` | `65`, message gains "and their look" |
  | `worldpack/tests/social_cafe.rs:338-380` | strips `name:`, `routine:`, `  - { from:` lines and `  - naming\n`, `  - schedule\n` | also strips `appearance:` lines and `  - appearance\n`; `17` stays |
  | `tools/cli/tests/commands.rs:34,40` | `"…, naming, schedule\n"`, `"53 genesis fact(s)"` | `"…, naming, schedule, appearance\n"`, `"65 genesis fact(s)"` |
  | `tools/cli/tests/content_kinds.rs:136` | `"\n53 genesis fact(s)"` | `"\n65 genesis fact(s)"` |
  | `tools/cli/tests/inspect.rs:55-56` | `…, conversation v4, group-activity v3, relationships v3, naming v1, schedule v1\n` (as of FG-a #188) | `…, naming v1, schedule v1, appearance v1\n` |
  | `tools/cli/tests/social_composition.rs:378-381` | `…, relationships v3, naming v1` (as of FG-a) | `…, relationships v3, naming v1, appearance v1` |
  | `tools/cli/tests/social_composition.rs:417-421` | `…, naming v1, schedule v1` | `…, naming v1, schedule v1, appearance v1` |
  | `tools/cli/tests/server_command.rs:43-53` | `[… "naming", "schedule"]` | `[… "naming", "schedule", "appearance"]` |
  | `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` (DL-a #191; read by `tools/cli/tests/decisions.rs:121-133`, AC-DLa-1) | social-cafe 30-day run output without `wall`, captured on base: `day N … facts F`, no `facts      appeared` line, `history    84639 facts, fingerprint af1927a1e2182bcd …` | regenerated in C4 step 8; only the `facts F` count of `day` lines, one new `facts      appeared 12` line (between `agenda-changed` and `arrived`) and the `history` line may differ — any other line differing is a material stop |

  A literal this search missed fails in CI; it is updated under §12's CI-repair authority **only** if it
  moved because of `appeared` facts, the `look` component or the `appearance` entry, with the reason in
  §15; any other cause is a material stop.
- **F-S26a-4 — every seat remembers 12 (10) extra genesis facts.** `appeared` is public, so every seat
  perceives one per person at genesis, and the generic renderer stores them as `notable` records
  ("appeared at somewhere, with Bob Achterberg"), importance 300 or 500 (S-15). The slug is never in
  the gist and never teaches the `NameBook`, so X-4 holds; the extra records are the same kind `named`
  already adds. Q-S26a-4.
- **F-S26a-5 — no cognition component formatter exists today** (S-15, S-16, S-17). The only path by which
  a look could be shown to a model is the memory renderer, which the X-4 test pins.
- **F-S26a-6 — `systems/README.md` lists thirteen packs** and omits `bodies`, `calendar` and `weather`.
  Not fixed here (unrelated staleness); reported to the primary session.
- **F-S26a-7 — the 3D binding has no reader before S26-c2**, and the 2D `look:` role no reader before
  S26-e. The specifications say so in a status line, so the gap is recorded rather than a contradiction
  (`CLAUDE.md` §2.1 rule 4).

### 3.3 What does not exist

`systems/appearance/`, any `look`/`appeared`/`appearance` identifier anywhere in the tree (searched:
`git grep '"look"'`, `from_static("appeared")`, `from_static("appearance")` — no hits),
`presentation/mineworld-default/3D/characters/`, `clients/3d-spike/PRESENTATION.md`,
`scripts/check_cast.py`.

## 4. Reuse, in both directions

- **Reused:** the `naming` pack's whole shape (`ARC-31` seam, `owned_component!`, `PerceptionProvider`,
  `SystemPack`), `serde-saphyr` for located refusals (`DEP-10`), `DEP-16`'s JSON-compatible YAML subset
  for `cast.yaml` (so Godot's built-in `JSON` reads it, as the 2D pack's files), Python's standard
  `json` for the checker. No new dependency.
- **Not forced:** no glTF, VRM or Godot resource format enters the world or the binding: a look is a key,
  a character id is a key, and the binding is three fields. Any richer 3D format stays in the client's
  importer (step-26 F-S26-3).

## 5. Design

### 5.1 The crate (`systems/appearance/`, mirroring `systems/naming/` file for file)

```text
Cargo.toml        name mineworld-appearance; the same dependencies and dev-dependencies as naming's
README.md         the five-line block of §5.1.1, one paragraph, the test command, links
src/lib.rs        module docs = the block; mods; re-exports
src/codec.rs      identical to naming's codec.rs (encode, to_value, event_payload)
src/look.rs       LookKey, InvalidLook, LOOK_MAX_BYTES                      (naming: name.rs)
src/section.rs    Appearance (the authored mapping) and its AuthoredSection impl
src/component.rs  Look — owned_component!, component_type "look", schema 1
src/event.rs      Appeared — EVENT_TYPE "appeared", schema 1, public
src/system.rs     AppearanceSystem, ID "appearance", BIOGRAPHICAL = &[], SystemPack, System, PerceptionProvider
tests/appearance.rs
```

#### 5.1.1 The block (lib.rs docs and README)

```text
section     appearance           a person file's `appearance: { look: alice-moreau }`
emits       appeared             at genesis, from the section; public
owns        Look                 `look`, payload { "look": "alice-moreau" }, reduced from `appeared` by this pack alone
discloses   Look                 to every observer who perceives the person — what someone looks like is visible
depends on  nothing
```

#### 5.1.2 Types and exact behaviour

- `look.rs`:
  - `pub const LOOK_MAX_BYTES: usize = 48;`
  - `pub struct InvalidLook { pub look: String, pub max: usize }`, `thiserror` message
    `"a look must be 1 to {max} bytes of a-z, 0-9 and '-', but got {look:?}"`.
  - `pub struct LookKey(String)`, derives `Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize,
    Deserialize`, `#[serde(try_from = "String", into = "String")]`.
  - `LookKey::new(look: impl Into<String>) -> Result<Self, InvalidLook>`: valid iff non-empty, `len() <=
    LOOK_MAX_BYTES`, and every byte is `b'a'..=b'z'`, `b'0'..=b'9'` or `b'-'` — exactly the step's
    `[a-z0-9-]{1,48}`, no further rule. Refuses, never truncates or lowercases.
  - `as_str`, `TryFrom<String>`, `From<LookKey> for String`, `Display` — as `Name`.
  - Doc comment: "An opaque key naming a person's appearance. Not a mesh, a file, a texture, a colour
    or a height: the world states *that* Alice has the appearance called `alice-moreau`, never what it
    is made of. A Presentation Pack resolves it (`ARC-92`)."
- `section.rs`:
  - `#[derive(Debug, Clone, PartialEq, Eq, Deserialize)] #[serde(deny_unknown_fields)] pub struct
    Appearance { look: LookKey }` with `pub const fn new(look: LookKey) -> Self` and `pub const fn
    look(&self) -> &LookKey`.
  - `impl AuthoredSection for AppearanceSystem`: `SECTION = SectionName::from_static("appearance")`,
    `CARRIED_BY = &[ContentKind::Person]`, `type Authored = Appearance`, no `references`, `seed` exactly
    as naming's (S-2) with `Appeared::new(person, authored.look().clone()).emission()`.
- `event.rs`: `pub struct Appeared { person: PersonId, look: LookKey }`; `EVENT_TYPE = "appeared"`,
  `OWNER = AppearanceSystem::ID`, `SCHEMA_VERSION = 1`; `new`, `person`, `look`; `emission()` exactly as
  naming's (S-3): `Visibility::Public`, about and with the person. Doc: "Stated at genesis, from the
  person's `appearance:` section. Nothing changes a look in MVP-0; when something does (clothing, ageing,
  disguise), it states this same fact, and this pack alone reduces it (`ARC-91`)."
- `component.rs`: `pub struct Look { look: LookKey }` with `new`, `look`; `owned_component! { component
  = Look, owner = AppearanceSystem, component_type = "look", schema_version = 1, }`. Serialized
  `{ "look": "alice-moreau" }` (an object, S-4).
- `system.rs`: `AppearanceSystem` (`#[derive(Default)]`), `ID = "appearance"`, `pub const BIOGRAPHICAL:
  &[EventTypeId] = &[];` ("Looking like somebody is not an event in a life."), `SystemPack` as naming's,
  `System::VERSION = 1`, `declaration()` owning `Look`, emitting and subscribing to `Appeared`;
  `install` registers `Look`; `react` inserts `Look::new(appeared.look().clone())` on `Appeared` only;
  `discloses` returns the subject's `Look` to every observer (`let _ = observer;`), with naming's doc
  sentence adapted: "Perception asks only about entities the observer already perceives, so nobody
  learns how somebody they cannot see looks. Appearance never depends on what the observer knows
  (`ARC-91`)." No `looks_in` projection (no reader needs one; `CLAUDE.md` §4 rule 11).
- `lib.rs`: `#![forbid(unsafe_code)] #![warn(missing_docs)]`; `mod codec; mod section;` and `pub mod
  component; pub mod event; pub mod look; pub mod system;`; `pub use component::Look; pub use
  event::Appeared; pub use look::{InvalidLook, LOOK_MAX_BYTES, LookKey}; pub use section::Appearance;
  pub use system::{AppearanceSystem, BIOGRAPHICAL};`.

**Vocabulary rule for every new file and comment:** no word beginning `body`, `physic`, `collision`
(the seam scan, `tests/acceptance/tests/seam_vocabulary.rs`, reads `systems/installed`); no renderer
word (`mesh`, `texture`, `glb`, `sprite`) outside a sentence that says a look is *not* one.

### 5.2 Installation

- Root `Cargo.toml` `[workspace.dependencies]`: `mineworld-appearance = { path = "systems/appearance" }`
  directly below the `mineworld-naming` line.
- `systems/installed/Cargo.toml`: `mineworld-appearance = { workspace = true }` as the last
  `[dependencies]` line.
- `systems/installed/src/lib.rs`: `Appearance => mineworld_appearance::AppearanceSystem,` as the **last**
  line of the list (after `Fishing`): appended, so the refusal listing's existing order is unchanged.

### 5.3 Worlds

| World | `systems:` | People (key → look) |
| --- | --- | --- |
| `social-cafe` | `- appearance` **appended after `- schedule`** (the last entry) | alice → `alice-moreau`, bob → `bob-achterberg`, carol → `carol-mensah`, dev → `dev-raman`, erin → `erin-walsh`, felix → `felix-okafor`, grace → `grace-liu`, hana → `hana-sato`, ivan → `ivan-petrov`, otto → `otto-brandt`, visitor → `vera-lindgren`, wanderer → `wes-calloway` |
| `market-town` | `- appearance` **directly after `- schedule`**, before the market packs' comment (F-S26a-1) | the same twelve, byte-identical lines (S-12) |
| `lakeside` | `- appearance` appended after `- fishing` | alba → `alba-ferreira`, bruno → `bruno-lindqvist`, celia → `celia-okafor`, dario → `dario-russo`, elin → `elin-berg`, finn → `finn-halvorsen`, greta → `greta-novak`, hugo → `hugo-marchetti`, ines → `ines-duarte`, nils → `nils-aaltonen` |
| `bodies-yard` | unchanged (Q-S26-4) | none |

- Each look key is the person's display name, lowercased, words joined by `-` — the step-26 §7.5 keys;
  review-readable, recorded as R-S26-9.
- The line is `appearance: { look: <key> }`, inserted **directly below the file's `name:` line**. In
  `alice.yaml` (social-cafe and market-town) and `alba.yaml` (lakeside), one comment line above it:
  `# A section the appearance system owns (ARC-91): which appearance she has, as a key a Presentation
  Pack resolves — never a mesh, a file or a colour.` (wrap at the file's width).
- Each `world.yaml` gets a comment on the new entry in the file's own style: social-cafe — "`appearance`
  owns each person's `appearance:` (`ARC-91`): which appearance they have, as a key a Presentation Pack
  resolves. It depends on nothing and nothing depends on it, so it is appended. Leave it out with the
  `appearance:` lines and people are drawn by each client's fallback"; market-town — the same, plus "It
  sits after `schedule`, not at the end, so this list still begins with Social Café's (`ARC-35`, AC-1)";
  lakeside — the social-cafe wording.
- READMEs: `worlds/social-cafe/README.md` "seven systems" → "eight systems" and a bullet
  `` - `appearance` gives everybody a look (each person file's `appearance:`), which a Presentation Pack draws; ``;
  `worlds/lakeside/README.md` line 9, the bundled System Packs row: append `, appearance` after
  `consumption` (`fishing` is its own row, line 10, so the bundled row stays in `world.yaml`'s order).
  `worlds/market-town/README.md`: no systems list (checked), no edit.

### 5.4 Genesis order and what moves

Per person file, facts are seeded in composition order (S-10): social-cafe `named, routine-assigned,
appeared`; market-town `named, routine-assigned, appeared, <market sections>`; lakeside `named,
routine-assigned, <item/inventory sections>, appeared`. Passages and arrivals keep their ids (the first
17 in social-cafe; `sections_do_not_move_the_facts_stated_before_them` keeps proving it). Every later
event id shifts. Social-cafe's genesis grows 53 → 65; market-town's and lakeside's grow by 12 and 10
(C4 records the exact numbers from `mineworld validate`).

### 5.5 The cognition check (X-4)

No code change. A new test in `cognition/lm-controller/tests/test_memory_render.py` renders an
`appeared` fact through `ingest` and pins: the gist is the generic `"appeared at somewhere, with Bob
Achterberg"`; the slug `bob-achterberg` appears in no gist; the `names` table still maps Bob to `Bob
Achterberg`; and for a person whose name was never perceived, the gist says `someone` — the slug never
becomes a name.

### 5.6 The 2D HUD

`clients/2d/scripts/hud/readers.gd:16` becomes `const SHOWN_ELSEWHERE := ["display-name", "participation",
"passages", "look"]` with the comment extended: "… passages are the drawn doorways, and a look chooses
how a figure is drawn (`ARC-92`), never text." The stub panels test (S-19) gains an own `look` record and
asserts no `look` panel is drawn.

### 5.7 The cast binding contract (ARC-92)

**`presentation/mineworld-default/3D/characters/cast.yaml`** (new; JSON-compatible YAML, no comments):

```json
{
  "format": 1,
  "looks": {},
  "extras": [],
  "default": "vera-lindgren"
}
```

Empty but valid: no look is bound and no extra exists until S26-d; `default` names the protagonist's
character id, whose `character.yaml` S26-c1 creates (ARC-93). `check_cast.py` (below) checks structure
only; S26-c1 adds "every id resolves".

**`clients/3d-spike/PRESENTATION.md`** (new specification; full text in Appendix C): what the 3D
reference client reads from a 3D Presentation Pack (today: only this binding, as data; meshes stay
importer resources, F-S26-3); the `cast.yaml` format and its rules; the resolution order of step-26 §4.4
for 3D; the status line (F-S26a-7); what a pack may not do.

**`clients/2d/PRESENTATION.md` §3.1** (exact edits in Appendix D): a `look:<key>` row; the `player` row
becomes the observer's fallback; the closing sentence gains "disclosed component value (`look:<key>`)";
the resolution order; the status line.

**`docs/MODULE_SPEC.md` §6.1**: the tree's 3D line gains `characters/cast.yaml`, and one sentence points
to `clients/3d-spike/PRESENTATION.md` (Appendix B).

**`docs/DECISIONS.md`**: `ARC-92` appended (Appendix A.2); `ARC-46` gains "Note 1, 2026-10-14"
(Appendix A.3).

**`scripts/check_cast.py`** (new; standard library only; the style of `scripts/check_decision_ids.py`):

```text
python3 scripts/check_cast.py [FILE ...]   default: presentation/mineworld-default/3D/characters/cast.yaml
python3 scripts/check_cast.py --self-test
```

Rules, each failure printed as `<file>: <rule>: <detail>`, exit 1 if any, else one line `<file>: format
1, <n> looks, <m> extras, default <id>: valid` and exit 0:

| Rule | Check |
| --- | --- |
| CC-1 | the file parses with `json.loads` (the JSON-compatible subset, `DEP-16`); a duplicate key at any level is refused (`object_pairs_hook`) |
| CC-2 | the top level is an object with exactly the keys `format`, `looks`, `extras`, `default` (the default pack is held to no unknown key; a client reading another pack warns instead, Appendix C §2) |
| CC-3 | `format` is the integer `1` (not `true`, not `1.0`) |
| CC-4 | `looks` is an object; every key and every value matches `^[a-z0-9-]{1,48}$` |
| CC-5 | `extras` is an array of distinct strings, each matching the same pattern |
| CC-6 | `default` is a string matching the same pattern |

`--self-test`: the valid file above passes; each of these planted texts fails naming its rule: not JSON
(CC-1), `{"format":1,"format":1,…}` (CC-1 duplicate), an extra top-level key `"colour"` (CC-2), a missing
`default` (CC-2), `"format": 2` (CC-3), `"format": true` (CC-3), a look key `Alice` (CC-4), a look value
`alice moreau` (CC-4), `"extras": ["a","a"]` (CC-5), `"extras": "a"` (CC-5), `"default": null` (CC-6).
Prints `self-test: <n> cases, all as expected` and exits 0, or names the case that did not behave and
exits 1. Both commands are added to `LAYERS["fast"]` in `scripts/ci_layer.py`, after `ci_parity.py
--self-test`, and named in the `fast` prose list of `.structured-coding/standards.md`.

## 6. Invariants this PR must hold

| Id | Invariant |
| --- | --- |
| I-S26a-1 | The diff touches only the files of §9's commit lists. No edit under `kernel/`, `contracts/`, `server/src/`, `worldpack/src/`, `authoring/`, `persistence/`, `sdk/`, `cognition/**/src/`, `clients/protocol/`, `clients/3d-spike/scripts/`, or any other System's `src/` |
| I-S26a-2 | A look is an opaque slug `[a-z0-9-]{1,48}`; the world carries no renderer concept and no descriptor; component `look` = `{ "look": "<slug>" }`, schema 1; event `appeared` = `{ person, look }`, schema 1, public, about the person |
| I-S26a-3 | Only `appearance` writes `Look`; it reacts only to `appeared`; it declares no action, no process, no dependency |
| I-S26a-4 | A look is disclosed to every observer who perceives the person and to nobody else; never gated by acquaintance or any observer knowledge |
| I-S26a-5 | A world without the pack is unchanged: `bodies-yard`'s five baseline keys are byte-equal before and after |
| I-S26a-6 | Enabling the pack changes no other system's facts and no controller's behaviour: in each named world the `requests`, `activity`, `consults` and `faults` summary lines are byte-equal before and after (E-S26a-3); without the pack, every other fact is equal row for row (AS26a-7) |
| I-S26a-7 | A look is never presented as a name or as text: not by the memory renderer, not by the 2D HUD |
| I-S26a-8 | No Meshy or OpenAI call, no secret file read or sourced, no character asset committed |
| I-S26a-9 | `cast.yaml` is JSON-compatible YAML; the default pack binds no look and no extra in this PR |
| I-S26a-10 | market-town and social-cafe stay AC-1 composable (S-12) without any edit to `ac1_composability.rs` |

## 7. Test ownership (test rules §26)

| Layer | Owns | Where |
| --- | --- | --- |
| Static | formatting, lints, types; `deny_unknown_fields`; doc ids and headings | `cargo fmt`, `cargo clippy -D warnings`, `ruff`, `pyright`, `check_doc_headings.py`, `check_decision_ids.py` |
| Unit (pack) | slug rule; section decoding (unknown key, missing key, bare string); reduction; public disclosure; place refusal | `systems/appearance/tests/appearance.rs` |
| Loader integration | X-1 located refusal; X-2 refusal by name | `worldpack/tests/refusals.rs` |
| World integration | every person's look in social-cafe, seeded after the earlier facts, reduced by its owner | `worldpack/tests/social_cafe.rs` |
| Real binary | the integration checkpoint over the wire; bodies-yard none; kill and restart keeps it; AC-2 row-for-row without the pack | `tools/cli/tests/appearance.rs`, `tools/cli/tests/social_composition.rs` |
| Cognition | X-4 memory renderer | `cognition/lm-controller/tests/test_memory_render.py` |
| Client (Godot, ignored) | X-4 in the 2D HUD | `tools/cli/tests/client_2d_interact_stub.rs` (`--ignored`, local and nightly) |
| Script | `cast.yaml` structure; checker self-test | `scripts/check_cast.py` in `fast` |
| Long-run (nightly) | X-3 baselines; I-S26a-6 behaviour lines | `scripts/baselines.txt`; C0/C6 captures |
| Gate 1 (real LLM) | NOT REQUIRED: no model-facing behaviour changes | — |
| Gate 2 (real lifecycle) | covered by the real-binary tests above; no separate Gate | — |
| CI impact | full PR CI (Rust on three platforms, python ×3, fast); no manual dispatch | — |

## 8. Acceptance criteria (fixed before anything is measured, `ARC-23`)

| Id | Criterion | Evidence | Shown to fail by |
| --- | --- | --- | --- |
| AS26a-1 | `mineworld server worlds/social-cafe`: the visitor's first observation discloses `look` on every person it perceives, equal to the authored key, keyed by their disclosed `display-name`: Alice Moreau → `alice-moreau`, Bob Achterberg → `bob-achterberg`, Vera Lindgren → `vera-lindgren`, Wes Calloway → `wes-calloway` | `every_person_the_visitor_perceives_is_disclosed_with_their_look` | M-S26a-4 |
| AS26a-2 | `mineworld server worlds/bodies-yard`: no perceived entity carries a `look` component | `a_world_without_appearance_discloses_no_look` | (absence; the pack is not enabled — no mutation) |
| AS26a-3 | Killed with SIGKILL and restarted on the same `--save`, the same instance discloses the same four looks | `a_look_is_the_same_after_the_server_is_killed_and_restarted` | persistence's own tests; N/A here (§15) |
| AS26a-4 (X-1) | `appearance: { look: Alice }` in a person file is refused as `PackError::Malformed` naming the file, with `a look must be` and the value's `line L column C` | `an_invalid_look_is_refused_by_its_owners_type_at_its_line`; unit `a_look_is_validated_by_its_own_type_as_it_is_decoded` | M-S26a-1 |
| AS26a-5 (X-2) | The same section in a world whose `systems:` lacks `appearance` is `PackError::ContentNeedsASystem { system: appearance, content: appearance }` | `an_appearance_section_in_a_world_without_appearance_is_refused_naming_appearance` | the loader's rule (`ARC-31` point 2), already proven for `naming`; N/A for S26-a code |
| AS26a-6 (X-3) | `bodies-yard`'s five baseline keys are unchanged; `baseline check` passes on the new long record with only the three named worlds re-recorded | C6 verdict lines | any change to `bodies-yard`'s files would move them (verified by the check's own self-test) |
| AS26a-7 | Without `appearance` (and its lines) social-cafe runs 30 days with no fault, and every other fact equals the full run's row for row; `requests`, `activity`, `consults` lines equal; the comparison sees seed 8 differ | `without_appearance_every_other_system_is_unchanged` | in-test seed-8 control |
| AS26a-8 | Base vs new 300-day runs of each named world: `requests`/`activity`/`consults`/`faults` lines byte-equal | E-S26a-3 (C0 + C6) | — (a measurement) |
| AS26a-9 (X-4) | The memory renderer never shows a slug, never teaches a name from a look | `test_an_appearance_is_never_remembered_as_a_name` | M-S26a-5 |
| AS26a-10 (X-4, client) | The 2D HUD draws no `look` panel | `the_panels_show_what_is_disclosed` (stub, ignored) | M-S26a-6 |
| AS26a-11 | `check_cast.py` passes on the committed file and its self-test passes | `fast` | the self-test's planted cases |
| AS26a-12 | Every person in social-cafe holds `Look` equal to their authored key, from one `appeared` per person stated by `appearance` at index ≥ 17 | `every_person_has_the_look_their_file_authors` | M-S26a-3 |
| AS26a-13 | AC-1 (`ac1_composability.rs`) and AC-2 for the market packs (`market_composition.rs`) pass unedited | targeted runs in C4 | — |

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately;
`[x]` needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.
Targeted validation per commit; the PR's CI is the one full run (test rules §§8–9). Every step is
written for a small-model implementer: numbered, exact paths, exact commands. A step that says "read" is
part of the work; record in §15 what was read.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s26a`. Never `cd` elsewhere; use paths
relative to that root.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in
   full, before editing it.
2. Make the numbered steps, in order. Do no work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time,
   and PASS / FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the
   failing test's name and assertion), revert it, run the test again green. Never commit a planted
   mutation; `git diff` must show none before staging.
5. Static checks for every commit that touches Rust: `cargo fmt --all --check`; `cargo clippy -p <each
   touched crate> --all-targets -- -D warnings`. Python: `uv run --locked ruff check
   cognition/lm-controller`; `uv run --locked ruff format --check cognition/lm-controller`; `uv run
   --locked pyright`. Markdown: `python3 scripts/check_doc_headings.py`; `python3
   scripts/check_decision_ids.py`.
6. Review: answer each review question of the commit in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files** list
   (plus `Cargo.lock` where listed). Otherwise revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff file in the same commit.
9. Stage by explicit path (`git add <path> …`, never `git add -A` or `git add .`), then `git commit -m
   "<subject>" -m "<body line(s)>" -m "Co-Authored-By: <your session's attribution>"`. Separate
   commands; no heredoc, no `sed -i`, no `python3 -c`.
10. If a step cannot be done as written, stop and report (a material stop, §12) instead of improvising.

#### C0 — Freeze, contract and the base captures

- **Goal.** Start against the frozen design, and capture the base behaviour I-S26a-6 is measured
  against before anything changes. **Files:** this file (§15); `.structured-coding/plans/mvp0/handoff-s26a.md`
  (new).
- Steps:
  1. `git log --oneline -1`; record the base commit in §15 and the handoff.
  2. Confirm the header reads `DESIGN FROZEN` with a date and §12 is filled; else stop.
  3. Create `handoff-s26a.md` with the fields of working rules' contract (PROJECT / PR, PRIMARY DESIGN
     DOC, RELATED / BINDING, branch, base, HEAD, working-tree fingerprint, approved scope, frozen
     invariants, endpoint authority with sources, sequence C0–C9, budget, checkpoint, next actions, stop
     conditions), in the format of `handoff-fg-b.md`.
  4. `cargo build --release -p mineworld-cli`; `mkdir -p target/scratch-s26a`.
  5. For each world `w` in `social-cafe`, `market-town`, `lakeside`, run (one at a time, or in the
     background and wait for all three):
     `target/release/mineworld run worlds/<w> --headless --seed 7 --days 300 --packs entities --packs
     presentation/mineworld-default > target/scratch-s26a/<w>.base.txt`
  6. For each `w`: `grep -E "^(requests|activity|consults|faults) " target/scratch-s26a/<w>.base.txt >
     target/scratch-s26a/<w>.base.behaviour`. Record each file's line count in §15 (E-S26a-1).
  7. `target/release/mineworld validate worlds/<w> --packs entities --packs presentation/mineworld-default`
     for each named world; record each "N genesis fact(s)" in §15 (E-S26a-2, base).
- [ ] Implementation: steps 1–3.
- [ ] Validation: steps 4–7 recorded; both doc checks.
- [ ] Review: no authority line widened; §14.1's rulings (once they exist) copied verbatim into the
  handoff.
- **Commit message:** `docs(plan): S26-a — open the ledger and handoff; base captures`.

#### C1 — Specification before code: the section and `ARC-91`

- **Goal.** The section and its owner are specified before the crate (`CLAUDE.md` §2.2). **Files:**
  `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`.
- Steps:
  1. Read `docs/MODULE_SPEC.md` §4.1 from line 706 ("**Sections: content a System Pack owns**") to the
     end of the sections table (line ~775), and `docs/DECISIONS.md` `ARC-31` (line 2194) as the model.
  2. In `docs/MODULE_SPEC.md`, change "MVP-0 has seven sections:" to "MVP-0 has eight sections:".
  3. Insert, directly below the `name` row (after its two lines, before `routine`), the `appearance` row
     of Appendix B.1, keeping the table's column alignment.
  4. In the person-file example block of §4.1 (the `# people/<key>.yaml` block), add below `name: Alice
     Moreau         # a section owned by the `naming` system (below)` the line
     `appearance: { look: alice-moreau }   # a section owned by the `appearance` system (below)`.
  5. Append `ARC-91` (Appendix A.1) at the end of `docs/DECISIONS.md`, after `ARC-89`, preceded by a
     line `---` and a blank line as the other records are separated.
- [ ] Implementation: steps 1–5.
- [ ] Validation: both doc checks; `grep -n "^## ARC-91" docs/DECISIONS.md` prints one line.
- [ ] Review: terms (`System Pack`, `Component`, `Person`, `Presentation Pack`, `Observation`) used
  exactly; no rule stated only in a README; no renderer concept in the world's half.
- **Commit message:** `docs: the appearance section and ARC-91 (S26-a)`.

#### C2 — `systems/appearance`: section, look, component, public disclosure

- **Goal.** The pack exists, validates its section, reduces its fact and discloses publicly. **Files:**
  `systems/appearance/Cargo.toml`, `systems/appearance/README.md`, `systems/appearance/src/{lib.rs,
  codec.rs, look.rs, section.rs, component.rs, event.rs, system.rs}`,
  `systems/appearance/tests/appearance.rs`, root `Cargo.toml`, `Cargo.lock`.
- Steps:
  1. Read all of `systems/naming/` (S-1) and `systems/bodies/src/section.rs:1-60`. Record in §15
     (A-S26a-1) that naming emits a public `named` at genesis and reduces `DisplayName` from it.
  2. Create the files of §5.1 with the content of §5.1.2. Copy `codec.rs` verbatim from naming (its
     first doc line names "this pack"; keep it). `Cargo.toml`: naming's, with `name =
     "mineworld-appearance"` and `description = "MineWorld's AppearanceSystem: which appearance a person
     has, as an opaque key authored in their World Pack file, seeded as a genesis fact and disclosed to
     whoever perceives them."`.
  3. Root `Cargo.toml`: add `mineworld-appearance = { path = "systems/appearance" }` below the
     `mineworld-naming` line (§5.2).
  4. README (human, short): the block of §5.1.1; one paragraph — "A look is a key, not art. The world
     says that Alice has the appearance called `alice-moreau`; the Presentation Pack a client uses says
     what that looks like (`characters/cast.yaml` in 3D, a `look:alice-moreau` role in 2D). A world
     without this pack, or a person without the section, has no look, and every client draws them with
     its fallback."; the command `cargo test -p mineworld-appearance`; links to `ARC-91`, `ARC-92`,
     `MODULE_SPEC.md` §4.1 and `step-26-cast.md` §4.
  5. Write `tests/appearance.rs` on the frame of `systems/naming/tests/naming.rs` (a café with Alice and
     Bob, a street with Carol; presence and appearance installed; looks seeded through the section
     contract by decoding YAML with `serde_saphyr::from_str::<Appearance>`). Tests:
     - (a) `a_seeded_look_is_reduced_by_its_owner_into_the_persons_look`: seed `{ look: alice-moreau }`
       and `{ look: bob-achterberg }`; exactly two `appeared` facts, both `Causation::WorldGenesis`, both
       `Visibility::Public`, subjects `[person]`; `Look` of alice and bob equal the keys; Carol has none.
     - (b) `a_look_is_disclosed_to_whoever_perceives_the_person_and_to_nobody_else`: as naming's
       disclosure test, with a helper `disclosed_look(observation, subject) -> Option<String>` that
       finds `component_type == Look::COMPONENT_TYPE`, asserts the payload is a JSON object, and
       decodes it as `Look`. Alice is told Bob's and her own look; Carol (street) is told nothing about
       Alice, and her own.
     - (c) `a_look_is_validated_by_its_own_type_as_it_is_decoded`: decode `look: alice-moreau` → ok;
       `look: a`, `look: "-"` and a 48-byte look of `a`s → ok (the frozen pattern is exactly
       `[a-z0-9-]{1,48}`; `-` alone is valid and must not be refused). Each of `look: ""`, a 49-byte
       look, `look: Alice`, `look: "alice moreau"`, `look: alice_moreau`, `look: "élise"` is refused
       with an error containing `a look must be`; `{ look: a, colour: red }` is refused with an error
       containing `colour`; `{}` with an error containing `look`; the bare string `alice-moreau` (not a
       mapping) is refused.
     - (d) `a_look_seeded_for_a_place_is_refused_by_its_owner`: `Rejection::PreconditionFailed`.
  6. `cargo build -p mineworld-appearance` (updates `Cargo.lock`).
- [ ] Implementation: steps 1–6.
- [ ] Validation: `cargo test -p mineworld-appearance` (4 tests); `cargo clippy -p mineworld-appearance
  --all-targets -- -D warnings`; `cargo fmt --all --check`. Mutations:
  - M-S26a-1: in `LookKey::new`, also accept `b'A'..=b'Z'` → test (c) red → revert.
  - M-S26a-2: `discloses` returns `Vec::new()` → test (b) red; then `discloses` returns the look only
    when `observer == subject` → test (b) red → revert.
  - M-S26a-3: `react` returns before `world.insert` → test (a) red → revert.
- [ ] Review: no action, process, dependency or other pack's vocabulary declared; no `HashMap`; the
  value type refuses rather than normalises; every doc sentence about a look says what it is *not*
  without naming a renderer type as something the world holds.
- **Commit message:** `feat(appearance): a System Pack for which appearance a person has, disclosed to whoever sees them (S26-a)`.

#### C3 — Install the pack; the loader refuses its section by name and at its line

- **Goal.** The build offers `appearance`; X-1 and X-2 hold through the real loader. **Files:**
  `systems/installed/Cargo.toml`, `systems/installed/src/lib.rs`, `worldpack/tests/refusals.rs`,
  `Cargo.lock`.
- Steps:
  1. Add the two installed lines of §5.2.
  2. In `worldpack/tests/refusals.rs`, after `a_section_in_a_kind_of_file_its_owner_does_not_allow_is_refused`
     (line 833), add a helper `with_appearance(id: &str, enabled: bool, alice: &str) -> Fixture` copied
     from `with_naming` (line 717) with `  - appearance\n`, and two tests:
     - `an_appearance_section_in_a_world_without_appearance_is_refused_naming_appearance`: alice file
       `"appearance: { look: alice-moreau }\nlocation:\n  place: cafe\n"`, `enabled = false`; assert
       `PackError::ContentNeedsASystem` with `subject == key("alice")`, `content == "appearance"`,
       `system == SystemId::new("appearance")`, `path.ends_with("people/alice.yaml")`.
     - `an_invalid_look_is_refused_by_its_owners_type_at_its_line`: alice file `"tags:
       [barista]\nlocation:\n  place: cafe\nappearance:\n  look: Alice\n"`, `enabled = true`; assert
       `PackError::Malformed` on `people/alice.yaml`, `detail.contains("a look must be")`, and
       `detail.contains("line 5 column 9")`. Print `detail` first; if the location reported is another
       column of line 5 (the stream may locate the key, column 3), assert what is printed and record it
       in §15 as a bounded discovery. A location on another line is a material stop.
  3. `cargo build -p mineworld-cli`.
- [ ] Implementation: steps 1–2.
- [ ] Validation: `cargo test -p mineworld-installed-systems`; `cargo test -p mineworld-worldpack --test
  refusals`; `cargo clippy -p mineworld-installed-systems -p mineworld-worldpack --all-targets -- -D
  warnings`; `target/debug/mineworld packs list` shows a `mineworld-appearance` system-pack line. Mutation:
  M-S26a-1 again → `an_invalid_look_is_refused_by_its_owners_type_at_its_line` red → revert.
- [ ] Review: the installed list is appended, not reordered; X-2's claim is the loader's (no mutation in
  S26-a code can turn it red — record N/A with that reason); no `body…` word in `systems/installed`.
- **Commit message:** `feat(installed): install the appearance pack; its section refused by name and at its line (S26-a)`.

#### C4 — A look for every person in the three named worlds

- **Goal.** The worlds author their looks; every literal that moves is updated; social-cafe's looks are
  proven through the real loader. **Files:** `worlds/social-cafe/world.yaml`,
  `worlds/social-cafe/people/*.yaml` (12), `worlds/social-cafe/README.md`,
  `worlds/market-town/world.yaml`, `worlds/market-town/people/*.yaml` (12),
  `worlds/lakeside/world.yaml`, `worlds/lakeside/people/*.yaml` (10), `worlds/lakeside/README.md`,
  `worldpack/Cargo.toml` (dev-dependency), `worldpack/tests/social_cafe.rs`,
  `tools/cli/tests/{commands.rs, content_kinds.rs, inspect.rs, social_composition.rs, server_command.rs}`,
  `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt`, `tools/cli/tests/decisions.rs` (doc comment),
  `Cargo.lock`.
- Steps:
  1. Edit the three `world.yaml` files exactly as §5.3 (positions, comments).
  2. Edit the 34 person files: the line `appearance: { look: <key> }` directly below `name:`, keys from
     §5.3's table; the comment line in the three files §5.3 names. Then `git diff --stat worlds/`: 34
     people files and 3 manifests changed (+ 2 READMEs after step 3).
  3. READMEs as §5.3 (read `worlds/lakeside/README.md` line 9 first).
  4. `worldpack/Cargo.toml` `[dev-dependencies]`: `mineworld-appearance = { workspace = true }` with a
     comment "the looks, decoded with appearance's own type".
  5. `worldpack/tests/social_cafe.rs`: apply F-S26a-3's three rows; add, after
     `every_person_is_named_by_the_owner_of_the_name_section_after_everything_else`, the test
     `every_person_has_the_look_their_file_authors`: the 12 `appeared` facts decoded with
     `mineworld_appearance::Appeared`; one per person in key order with the §5.3 key; each at genesis
     index ≥ 17; `provenance().emitted_by() == "appearance"`; `read.component::<Look>(id)` equal; and
     for every person, the look differs from the key (`alice-moreau != alice`) — a look is not a key.
  6. `tools/cli/tests/*`: apply F-S26a-3's remaining rows, each comment updated to name the look
     (commands.rs: "…twelve routines, the twelve first agendas they imply, and twelve looks").
  7. `cargo build -p mineworld-cli`; `target/debug/mineworld validate worlds/<w> --packs entities --packs
     presentation/mineworld-default` for each named world and for `bodies-yard`; record each genesis
     count (E-S26a-2, new): social-cafe must be 65; market-town base + 12; lakeside base + 10;
     bodies-yard unchanged.
  8. The AC-DLa-1 golden (F-S26a-3, last row): `target/debug/mineworld run worlds/social-cafe --headless
     --seed 7 --days 30 | grep -v "^wall " > target/scratch-s26a/golden.new.txt`; then `diff
     tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt target/scratch-s26a/golden.new.txt`
     and record every differing line in §15. Only `day` lines' `facts` counts, one added `facts      appeared 12`
     line and the `history` line may differ; otherwise stop. Then `cp target/scratch-s26a/golden.new.txt
     tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt`. The test's doc comment names the
     base it was captured from; add one sentence: "Re-captured by S26-a (appearance's genesis facts); the
     diff is recorded in pr-s26a-appearance.md §15."
- [ ] Implementation: steps 1–6 and 8.
- [ ] Validation: step 7; `cargo test -p mineworld-worldpack`; `cargo test -p mineworld-cli --test
  commands --test content_kinds --test inspect --test server_command --test decisions`; `cargo test -p mineworld-cli
  --test social_composition` (real runs; record wall time); `cargo test -p mineworld-acceptance --test
  ac1_composability` (needs full git history; if the worktree's clone is shallow, record INCONCLUSIVE and
  rely on CI); `cargo test -p mineworld-cli --test market_composition`. Mutation: M-S26a-3 → 
  `every_person_has_the_look_their_file_authors` red → revert.
- [ ] Review: market-town's list still begins with social-cafe's (I-S26a-10); no person file outside the
  three worlds touched; every key is §5.3's; no test literal changed for a cause other than the look.
- **Commit message:** `feat(worlds): a look for every person in social-cafe, market-town and lakeside (S26-a)`.

#### C5 — The integration checkpoint over the wire, and AC-2 for appearance

- **Goal.** AS26a-1, -2, -3 and -7 through the real binary. **Files:** `tools/cli/Cargo.toml`
  (dev-dependency), `tools/cli/tests/appearance.rs` (new), `tools/cli/tests/social_composition.rs`,
  `Cargo.lock`.
- Steps:
  1. `tools/cli/Cargo.toml` `[dev-dependencies]`: `mineworld-appearance = { workspace = true }` with the
     comment "the looks a client is told, decoded with appearance's own type".
  2. Create `tools/cli/tests/appearance.rs` (`mod support;`, `use support::{Client, SaveDir, Server};`).
     Helpers: `look_of(&WireObservation, EntityId) -> Option<String>` (decode with
     `mineworld_appearance::Look`, as `support::own_history` decodes); `name_of(…)` (decode
     `display-name` with `mineworld_naming::DisplayName`); `looks_by_name(&WireObservation) ->
     BTreeMap<String, Option<String>>` over every perceived entity whose `entity_type()` is
     `EntityType::Person`. Constant `BODIES_YARD = concat!(env!("CARGO_MANIFEST_DIR"),
     "/../../worlds/bodies-yard")`. Tests (`#[tokio::test]`):
     - `every_person_the_visitor_perceives_is_disclosed_with_their_look`: `Server::start(&["server",
       support::PACK])`; join `visitor`; first observation; `looks_by_name` equals the four entries of
       AS26a-1, each `Some`.
     - `a_world_without_appearance_discloses_no_look`: `Server::start(&["server", BODIES_YARD, "--packs",
       <repo>/entities, "--packs", <repo>/presentation/mineworld-default])` (build the two paths with
       `concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities")` etc.; read how `tools/cli/tests/bodies/`
       starts the yard and follow it if it differs); join `ada`; at least two persons perceived; no
       component of any perceived entity has `component_type == "look"`.
     - `a_look_is_the_same_after_the_server_is_killed_and_restarted`: `SaveDir::new("appearance-restart")`;
       command `["server", support::PACK, "--save", save.path()]`; start, join visitor, record
       `looks_by_name` and `status()["instance"]`; `kill()`; start again with the same command; join;
       the instance and the map are equal to before.
  3. `tools/cli/tests/social_composition.rs`: add `appearance_fact(fact) -> bool` (`event_type ==
     "appeared"`) and the test `without_appearance_every_other_system_is_unchanged`, a copy of
     `without_relationships_every_other_system_is_unchanged` with `without_owning("appearance",
     Some("appearance"))`, the filter `!appearance_fact`, the assertion that no fact of the bare run is
     `appeared`, the `requests `/`activity `/`consults ` line equality, and the seed-8 control.
- [ ] Implementation: steps 1–3.
- [ ] Validation: `cargo test -p mineworld-cli --test appearance` (3 tests); `cargo test -p mineworld-cli
  --test social_composition without_appearance`; clippy on `mineworld-cli`. Mutation: M-S26a-4:
  `discloses` returns `Vec::new()` → `every_person_the_visitor_perceives_is_disclosed_with_their_look`
  red → revert.
- [ ] Review: oracles are the authored keys and the disclosed names, never ids; the restart test kills,
  never interrupts; no test sleeps on wall time beyond the support module's patience.
- **Commit message:** `test(cli): a look on the wire, none without the pack, kept across a restart; AC-2 for appearance (S26-a)`.

#### C6 — Golden digests: the three named worlds re-recorded

- **Goal.** I-S26a-5 and I-S26a-6 measured; `scripts/baselines.txt` current (R-S26-5). **Files:**
  `scripts/baselines.txt`, this file (§15).
- Steps:
  1. `cargo build --release -p mineworld-cli`.
  2. For each named world `w`, as C0 step 5 into `target/scratch-s26a/<w>.new.txt`, then as C0 step 6
     into `<w>.new.behaviour`.
  3. For each `w`: `diff target/scratch-s26a/<w>.base.behaviour target/scratch-s26a/<w>.new.behaviour`
     must print nothing (E-S26a-3). Any output is a **material stop** (I-S26a-6).
  4. For each `w`: `diff target/scratch-s26a/<w>.base.txt target/scratch-s26a/<w>.new.txt` and record
     the kinds of line that differ (expected: `day … facts N`, a new `facts      appeared <n>` line, the
     `history` line, `wall`). Any other kind of line is a material stop.
  5. In the background: `python3 scripts/ci_parity.py record --binary target/release/mineworld --profile
     long --out target/scratch-s26a/s26a.long` (expected ≤ 25 min); wait for it.
  6. `python3 scripts/ci_parity.py baseline check target/scratch-s26a/s26a.long`; record the verdict
     lines. Expected: `bodies-yard` all five keys equal; `social-cafe`, `market-town`, `lakeside` keys
     moved. A moved `bodies-yard` key is a **material stop** (I-S26a-5).
  7. `python3 scripts/ci_parity.py baseline write target/scratch-s26a/s26a.long scripts/baselines.txt`.
  8. Under each of the three moved worlds' `[world …]` headers, after the existing `reason` line, add one
     line: `reason S26-a (pr-s26a-appearance.md): the appearance pack's genesis appeared facts (12 per
     town, 10 at the lake) shift every later event id and add one facts line; requests, activity,
     consults and faults unchanged (E-S26a-3)` — `lakeside`'s says `10`, the others `12`, in the file's
     reason-line format.
  9. `python3 scripts/ci_parity.py baseline check target/scratch-s26a/s26a.long` → PASS.
- [ ] Implementation: steps 7–8.
- [ ] Validation: steps 1–6 and 9, verdict lines and wall times recorded.
- [ ] Review: only the three named worlds' key lines changed (`git diff scripts/baselines.txt`); reason
  lines kept; no hand-edited digest.
- **Commit message:** `test(baselines): re-record social-cafe, market-town and lakeside for the appearance pack (S26-a)`.

#### C7 — A look is never shown as a name or as text

- **Goal.** X-4 in cognition and in the 2D HUD. **Files:**
  `cognition/lm-controller/tests/test_memory_render.py`, `clients/2d/scripts/hud/readers.gd`,
  `tools/cli/tests/client_2d_interact_stub.rs`.
- Steps:
  1. Read `cognition/lm-controller/src/mineworld_cognition/memory/render/__init__.py` (S-15) and
     `tests/memory_fixtures.py` (`fact`, `names_of_everyone`, `BOB`, `CAFE`).
  2. Add `test_an_appearance_is_never_remembered_as_a_name` to `test_memory_render.py`: with
     `names_of_everyone()` ingested, ingest `fact(10, 100, "appeared", {"person": {"entity": BOB,
     "entity_type": "person"}, "look": "bob-achterberg"}, subjects=[BOB], participants=[BOB],
     place=None)` and a second `appeared` about an entity whose name was never perceived (`EntityId("7999999999")`,
     look `"stranger-look"`). Assert: the first gist is `"appeared at somewhere, with Bob Achterberg"`;
     the second contains `someone`; no gist in the dump contains `bob-achterberg` or `stranger-look`;
     the `names` table rows are exactly the four of `names_of_everyone`; `report.decode_fallbacks == 0`.
     (If the gist wording differs because `place=None` renders otherwise, assert what `generic` prints
     and record it; the slug and name assertions are the claim.)
  3. `readers.gd`: §5.6.
  4. `client_2d_interact_stub.rs` `the_panels_show_what_is_disclosed`: add `(record(me, "look", json!({
     "look": "carol-mensah" })), None),` to `own`, and after the `weather-sense` assertions,
     `assert!(panel(first, "look").is_none(), "a look chooses how a figure is drawn, never a panel")`.
- [ ] Implementation: steps 2–4.
- [ ] Validation: `uv run --locked pytest cognition/lm-controller/tests/test_memory_render.py`; the three
  Python static checks; `cargo test -p mineworld-cli --test client_2d_interact_stub
  the_panels_show_what_is_disclosed -- --ignored` (needs Godot 4.7 on PATH; if absent, INCONCLUSIVE,
  recorded, and the nightly `clients` job is the evidence). Mutations:
  - M-S26a-5: in `generic()`, append `f" {fact.payload.payload}"` to the gist → the new pytest red →
    revert.
  - M-S26a-6: remove `"look"` from `SHOWN_ELSEWHERE` → the stub test red → revert (only if Godot ran).
- [ ] Review: no cognition source changed; the HUD change hides only `look`; I-3's "never dropped" is
  honoured through the sanctioned list, whose comment says why.
- **Commit message:** `fix(cognition, 2d): a look is never shown as a name or as text (S26-a)`.

#### C8 — The cast binding contract: `ARC-92`, `look:<key>`, `characters/cast.yaml`

- **Goal.** Both clients' bindings are specified, and the default 3D pack carries a valid empty one.
  **Files:** `docs/DECISIONS.md`, `docs/MODULE_SPEC.md`, `clients/2d/PRESENTATION.md`,
  `clients/3d-spike/PRESENTATION.md` (new), `presentation/mineworld-default/3D/characters/cast.yaml`
  (new), `scripts/check_cast.py` (new), `scripts/ci_layer.py`, `.structured-coding/standards.md`.
- Steps:
  1. Append `ARC-92` (Appendix A.2) after `ARC-91`; add Appendix A.3's note at the end of `ARC-46`
     (before its closing `---`, after "What it does not decide.").
  2. `docs/MODULE_SPEC.md` §6.1: Appendix B.2.
  3. `clients/2d/PRESENTATION.md`: Appendix D.
  4. Create `clients/3d-spike/PRESENTATION.md` with Appendix C's text.
  5. Create `presentation/mineworld-default/3D/characters/cast.yaml` with §5.7's JSON, a final newline.
  6. Create `scripts/check_cast.py` per §5.7 (module docstring stating why it exists, `ARC-92`, the
     rules CC-1…CC-6; `main()` and `self_test()`; `if __name__ == "__main__": sys.exit(main())`).
  7. `scripts/ci_layer.py`: add `["python3", "scripts/check_cast.py"],` and `["python3",
     "scripts/check_cast.py", "--self-test"],` to `LAYERS["fast"]` after the `ci_parity.py --self-test`
     line; `.structured-coding/standards.md`: add `check_cast.py` and its self-test to the `fast` prose
     list (line ~60).
- [ ] Implementation: steps 1–7.
- [ ] Validation: `python3 scripts/check_cast.py` → valid; `python3 scripts/check_cast.py --self-test` →
  all cases as expected; `python3 scripts/ci_layer.py --list fast` shows both; both doc checks. Mutation:
  change the committed file's `"format": 1` to `2` → the checker exits 1 naming CC-3 → revert.
- [ ] Review: the 2D and 3D resolution orders are the same four lines (step-26 §4.4); `look:<key>` is
  stated as a disclosed component value, never a world key; no rule a client would evaluate; the status
  lines say plainly what is not yet read.
- **Commit message:** `docs, presentation: the cast binding contract — ARC-92, look:<key>, characters/cast.yaml (S26-a)`.

#### C9 — Close-out, push, PR

- **Goal.** Review readiness. **Files:** this file (§15), `handoff-s26a.md`.
- Steps:
  1. `git diff --stat origin/main...HEAD`: every path in §9's lists; nothing else (I-S26a-1).
  2. Fill §15 (ledger SHAs, evidence, mutations, deviations); update the handoff.
  3. Commit `docs(plan): S26-a — ledger close-out`.
  4. `git push -u origin mvp0/pr-s26a`; `gh pr create --base main --title "S26-a: the appearance System
     Pack and the cast binding contract" --body-file <a file written with the Write tool under
     target/scratch-s26a/>`; body: what changed, X-1…X-4 evidence, the baseline re-record with its
     reason, the questions' rulings, and the attribution line.
  5. Watch CI on the exact head (`gh pr checks`); repair failures under §12 (F-S26a-3's rule for missed
     literals); record the final head and CI in the handoff.
  6. Stop: **READY FOR OPERATOR REVIEW — DO NOT MERGE**.
- [ ] Implementation: steps 1–4.
- [ ] Validation: CI green on the exact final head, every job.
- [ ] Review: the PR body states the unresolved and INCONCLUSIVE items honestly.
- **Commit message:** `docs(plan): S26-a — ledger close-out`.

## 10. Golden digests and baselines: expected impact, with reasons

| World | `validate` digest (default parity record) | Baseline keys (`summary-300`, `summary-1000`, `summary-300s`, `facts-300s`, `journal-300s`) | Reason line |
| --- | --- | --- | --- |
| `social-cafe` | moves: systems line gains `appearance`; 53 → 65 genesis facts; ids unchanged | **all five move**: +12 genesis facts shift every later event id (`facts-300s`, `journal-300s`), and every summary prints fact counts, a new `facts appeared 12` line and the history fingerprint (S-26) | `reason S26-a (pr-s26a-appearance.md): the appearance pack's genesis appeared facts (12 per town, 10 at the lake) shift every later event id and add one facts line; requests, activity, consults and faults unchanged (E-S26a-3)` |
| `market-town` | moves: +12 genesis facts | all five move, same cause | same line |
| `lakeside` | moves: +10 genesis facts | all five move, same cause | same line (10) |
| `bodies-yard` | **unchanged** | **unchanged** (X-3, I-S26a-5) | none |

What must **not** move: any `requests`, `activity`, `consults` or `faults` line of any world (I-S26a-6,
E-S26a-3); any `bodies-yard` key. Golden *frames* (`server/tests/frames/*.json`) are hand-built examples,
not world output, and do not move (`server/tests/frames.rs:1-13`).

## 11. Edits to parent documents (owner: the primary session, after freeze or merge)

| Document | Edit |
| --- | --- |
| `step-26-cast.md` §12 S26-a row, §15 | designed in this file; frozen / merged as …; F-S26a-1 (market-town position) recorded |
| `overall.md` | S26-a's state; `ARC-91`, `ARC-92` used |
| `pr-s10-p6-lm-controller.md` §15 (P6a) | R-S26a-1: whichever of P6a and S26-a merges second rebases its `systems:` entry after the other's and re-records baselines |
| `systems/README.md` | F-S26a-6, if the primary session wants it fixed |

## 12. Execution contract (filled at drafting; effective at freeze)

Source of every line marked "primary": the primary session's dispatch of this design (2026-10-14), which
names the worktree, the branch and "merge only on the operator's explicit authorization", and step-26
§12.1 (start condition, implementer). Lines that take a working-rules default name the section. Merge
authority is never inherited.

```text
PROJECT / PR:            MineWorld mvp0, S26-a — the appearance System Pack and the cast binding contract
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s26a-appearance.md (all sections; ledger §15)
RELATED / BINDING DOCS:  step-26-cast.md (§§4.1–4.4, 8, 9, 12, 12.1, 12.2, 13, 15, 16.1); overall.md;
                         docs/MODULE_SPEC.md §§4.1, 6.1; docs/DECISIONS.md ARC-31, ARC-33, ARC-35, ARC-46;
                         systems/naming (the pattern); clients/2d/PRESENTATION.md; scripts/baselines.txt
                         rules in .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s26a, its own, held by one session (primary)
BRANCH:                  mvp0/pr-s26a, created from origin/main                                   (primary)
START CONDITION:         this design DESIGN FROZEN; nothing else                     (step-26 §12.1; primary)
IMPLEMENTATION BASE:     origin/main at C0; C0 records the commit
APPROVED SCOPE:          §2.1 items 1–9, as ruled in §14.1                                         (primary)
FROZEN INVARIANTS:       §0 rulings; §6 I-S26a-1 … I-S26a-10; §8 AS26a-1 … AS26a-13
SEQUENCE:                C0 … C9 (§9)
ALLOWED COMMANDS:        cargo *; uv run --locked *; uv sync --locked; git; gh (never merge);
                         python3 scripts/*; target/*/mineworld *; mkdir -p; grep; diff; sed -n; cp (C4 step 8 only);
                         godot (only through C7's cargo test)
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A / git add .; sourcing or reading
                         ~/.config/mineworld/secrets.env or any key or .env file; any Meshy or OpenAI call;
                         edits outside §9's file lists; editing a baseline by hand except C6 step 8's reason
                         lines; editing ac1_composability.rs
MATERIAL STOPS:          a behaviour line (requests/activity/consults/faults) differs in C6 step 3; any
                         bodies-yard baseline key moves; a summary line of a kind other than C6 step 4's
                         list differs; AC-1 fails; any need to edit a file under I-S26a-1's forbidden paths;
                         a new external dependency; X-1's refusal located on another line; a CI failure whose
                         cause is not the look (F-S26a-3)
PLATFORMS:               Linux, macOS, Windows (the PR's CI)
VALIDATION BUDGET:       unit and integration tests unrestricted locally; C0 and C6 300-day runs (≤ 15 min
                         together); one long parity record in C6 (≤ 25 min); local full suite: not run (test
                         rules §8); CI: the PR's runs, no manual dispatch. Gate 1: NOT REQUIRED (no model-facing
                         change). Money: none — no paid API is called
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s26a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized at freeze      (primary; step-26 §12.1)
                         semantic commits: authorized                                 (primary; working rules §14)
                         branch push: authorized                                      (primary; working rules §21)
                         PR creation / update: authorized                             (primary; working rules §21)
                         CI repair to review readiness: authorized, bounded by F-S26a-3 (primary; rules §21)
                         merge: never by the implementation session; only with the operator's explicit
                         authorization after the primary session's review (working rules §22)
POST-MERGE SYNC OWNER:   primary session: step-26, overall.md, §11's table; implementation session: this
                         ledger's rows, the merge identity, evidence and deviations
STOP CONDITION:          S26-a READY FOR OPERATOR REVIEW — DO NOT MERGE
MERGE AUTHORITY:         NEVER merge without explicit operator approval
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S26a-1 | Concurrent PRs edit the same lines: P6a (`persona`, frozen, not started) appends to social-cafe's `systems:`, edits nine people files, `systems/installed` and `baselines.txt`; WE-a (`happenings`, draft #189) and DL-a (#191) touch `systems/installed` and worlds | whichever merges second rebases, keeps both entries in merge order (market-town keeping Social Café's list as its prefix), updates the moved literals and re-records baselines with its own reason line (Q-S26a-5); §11 tells P6a |
| R-S26a-2 | A pinned literal not found by the audit fails in CI | F-S26a-3's bounded CI-repair rule |
| R-S26a-3 | serde-saphyr locates the refusal at the key rather than the value | C3 step 2's bounded discovery; a different line is a stop |
| R-S26a-4 | 12 generic `notable` memory records per seat at genesis crowd retrieval for P6b | F-S26a-4; P6b may add an ambient `appeared` renderer (Q-S26a-4) |
| R-S26a-5 | The AC-1 test needs full history; a shallow worktree cannot run it | INCONCLUSIVE locally, CI is the evidence (C4) |
| R-S26a-6 | Godot absent on the implementer's machine | C7's stub test INCONCLUSIVE locally; the nightly `clients` job runs it |
| R-S26a-7 | Look keys that are name slugs leak names if names are later acquaintance-gated | R-S26-9, recorded in `ARC-91`'s Revisit |

## 14. Operator questions (each with a recommendation)

Questions the step already ruled (Q-S26-1…12) are not re-asked.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S26a-1 [primary — reading of the step]** | Step-26 §4.2 says "one appended entry in each world's `systems:` list". In market-town, AC-1 check 3 (`ac1_composability.rs:1036-1080`) requires the list to begin with Social Café's, so `appearance` must sit **directly after `schedule`**, before the market packs, not at the end. Accept that placement? | **Yes.** `appearance` reacts only to its own fact, so no other reduction order moves; the alternative — adding `appearance` to AC-1's `GENERIC_PACKS` allow-list — would change `ARC-35`'s measurement to fit a pack that is not configuration |
| **Q-S26a-2 [operator — scope, one client line]** | The 2D HUD prints every own component without a reader as raw JSON, so the observer would see `look: vera-lindgren` in an "other" panel. Add `"look"` to `readers.gd`'s `SHOWN_ELSEWHERE` (and pin it in the stub panels test) in S26-a, though the step's S26-a row lists no 2D script? | **Yes.** It is X-4's defect in the client, one line, and the sanctioned mechanism; leaving it to S26-e ships a visible slug in the meantime |
| **Q-S26a-3 [primary — scope]** | Add `scripts/check_cast.py` (structure only) to the `fast` layer, so "empty but valid" is checked rather than asserted; S26-c1 then adds the resolution rule? | **Yes.** About 100 lines of standard-library Python with a self-test; without it the file's validity has no reader until S26-c2 |
| **Q-S26a-4 [primary — cognition]** | The memory renderer stores each `appeared` generically as `notable` ("appeared at somewhere, with Bob Achterberg"); the slug never shows. Keep that in S26-a (test only), or add an `appeared` renderer that files it as `ambient`? | **Keep generic in S26-a**; no cognition code changes, X-4 is pinned by the test, and P6b, which owns retrieval quality, may add a renderer if its retrieval needs it |
| **Q-S26a-5 [primary — sequencing]** | P6a (`persona`) is frozen and not started, and edits the same `systems:` lists, people files, installed set and baselines. Rule: whichever of P6a and S26-a merges second rebases and re-records? | **Yes**, with market-town keeping Social Café's list as its prefix in either order |

### 14.1 Rulings, 2026-10-14 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-S26a-1 | **Yes.** `appearance` goes directly after `schedule` in market-town; AC-1's allow-list (`GENERIC_PACKS`) stays unchanged | primary session |
| Q-S26a-2 | **Yes.** Add `"look"` to the 2D HUD's `SHOWN_ELSEWHERE` in S26-a (C7), pinned by the stub panels test | operator |
| Q-S26a-3 | **Yes.** `scripts/check_cast.py` and its self-test run in the `fast` layer (C8) | primary session |
| Q-S26a-4 | **Keep the generic rendering**; no `appeared` renderer now (C7 is a test only) | primary session |
| Q-S26a-5 | **Yes.** Whichever of P6a and S26-a merges second rebases and re-records the baselines (market-town keeping Social Café's list as its prefix) | primary session |

**Re-audit at freeze (origin/main @ 9aa185b).** DL-a, FG-a, 25b and SET-b merged after the planning base. Anchors re-checked and updated in §3.1, §9 and F-S26a-3: `systems/installed/src/lib.rs:38-60` (FG-a added a `DecayCurve` extension line; `Fishing` is still the last entry), `worldpack/src/error.rs:369`, `docs/MODULE_SPEC.md:706-775` and `1344-1380`, `docs/DECISIONS.md` `ARC-31` at 2194 and `ARC-46` at 3802-3834; FG-a bumped `conversation` to v4 and `relationships` to v3 in the `inspect` systems lines (F-S26a-3 rows updated); DL-a added the golden `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt`, which S26-a's genesis facts move (new F-S26a-3 row; C4 step 8). `worldpack/src/read.rs` gained DL-a's `decisions:` key; the section path S26-a relies on (`load.rs:344-366`) is unchanged. No other anchor moved.

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | base adf6d49 (commit recorded in the handoff) | [x] | [x] | [x] |
| C1 | (this commit) | [x] | [x] doc checks PASS (headings, decision ids); `grep -n "^## ARC-91"` 1 line | [x] terms exact; section and ARC-91 carry no renderer concept in the world's half |
| C2 | (this commit) | [x] crate per §5.1 (files, README, look/section/component/event/system, tests a–d); codec.rs identical to naming's (diff empty); Cargo.toml naming's with the two fields changed | [x] `cargo test -p mineworld-appearance`: 4 passed (0.00 s). Clippy `-D warnings` clean; `cargo fmt --all --check` clean. Mutations: M-S26a-1 → test (c) red at line 221 (`a look must`), reverted green; M-S26a-2 `discloses` empty → test (b) red at line 166, self-only variant → (b) red at line 166, reverted green; M-S26a-3 `react` skipped → (a) red at line 142 (`alice's Look is the authored key`), reverted green. Vocabulary grep: only the sentences that say a look is not a mesh/texture | [x] no action, process or dependency declared beyond `presence`/`sdk` seams; no `HashMap`; the value type refuses (`LookKey::new`); doc sentences say what a look is not |
| C3 | (this commit) | [x] installed line (`Appearance => mineworld_appearance::AppearanceSystem`, last) and Cargo line (last `[dependencies]`); two refusal tests and `with_appearance` helper in `worldpack/tests/refusals.rs` | [x] `cargo test -p mineworld-installed-systems`: 3 + 2 passed; `cargo test -p mineworld-worldpack --test refusals`: 40 passed (both new tests ok; `line 5 column 9` located exactly, no bounded discovery needed, A-S26a-2 holds). Clippy `-D warnings` on the three crates clean; fmt clean; `mineworld packs list` shows `system-pack mineworld-appearance`. Mutation M-S26a-1 (uppercase accepted) → `an_invalid_look_is_refused_by_its_owners_type_at_its_line` red (refusals.rs:105, no refusal produced), reverted green | [x] appended, not reordered; X-2 is the loader's (N/A for a mutation in pack code; the refusal test is the evidence); no `body`/`physic` word in the installed list |
| C4 | (this commit) | [x] three manifests (`appearance` after `schedule` in market-town, appended elsewhere); 34 person files (`appearance: { look: … }` under `name:`, comments in alice/alba); READMEs; worldpack dev-dep; F-S26a-3 rows (social_cafe.rs, commands, content_kinds, inspect, social_composition ×2, server_command, decisions doc comment); new test `every_person_has_the_look_their_file_authors`; golden re-captured (diff: `day` facts counts, `facts      appeared 12` added, `history` line; no other line) | [x] `validate`: social-cafe 65, market-town 146 (134+12), lakeside 92 (82+10), bodies-yard 47 unchanged. `cargo test -p mineworld-worldpack`: all crates green (social_cafe incl. the new test; refusals 40). `cli --test commands` 4, `content_kinds` 3, `inspect` 3, `server_command` 10, `social_composition` 4 (38.9 s), `decisions` 7 (after re-capture), `market_composition` ok. `ac1_composability`: 15 passed (full history present, not INCONCLUSIVE). M-S26a-3 (react skipped) → `every_person_has_the_look_their_file_authors` red (social_cafe.rs:382, "reduced by appearance into alice's look"), reverted green. fmt clean; clippy `-D warnings` clean (worldpack, cli) | [x] market-town's list begins with social-cafe's (AC-1 green); no person file outside the three worlds touched; keys per §5.3; literal changes each caused by the look or the `appearance` entry
| C5 | — | [ ] | [ ] | [ ] |
| C6 | — | [ ] | [ ] | [ ] |
| C7 | — | [ ] | [ ] | [ ] |
| C8 | — | [ ] | [ ] | [ ] |
| C9 | — | [ ] | [ ] | [ ] |

### Evidence

| Id | What | Value |
| --- | --- | --- |
| E-S26a-1 | base behaviour line counts per world (C0), base `origin/main @ adf6d49`, seed 7, 300 days: social-cafe 20, market-town 24, lakeside 24 (`target/scratch-s26a/<w>.base.behaviour`); wall: social-cafe 617.9 s, market-town 1172.3 s, lakeside 114.4 s (lakeside's first run was killed at day 163 by an external signal, exit 144, and rerun alone to completion; load average was 90–180 on the shared machine, R-LOAD-1: timings unreliable) | C0 |
| E-S26a-2 | genesis counts, base (C0) and new (C4): base social-cafe 53, market-town 134, lakeside 82, bodies-yard 47 (`mineworld validate`, base); new social-cafe 65, market-town 146, lakeside 92, bodies-yard 47 | C0 (base); C4 (new) |
| E-S26a-5 | 30-day golden (C4 step 8) diff, `social-cafe-seed-7-30-days.txt`: `day N … facts F` lines (every count +12, day 1 3134 → 3146 through day 30 84639 → 84651), one added `facts      appeared 12` line (line 42), `history` line 84639 → 84651 facts, fingerprint `af1927a1e2182bcd` → `049f4c80bfd3d7cf`; no other line differs | C4 |
| E-S26a-3 | behaviour diffs empty, per world (C6 step 3) | — |
| E-S26a-4 | `baseline check` verdicts before and after the re-record (C6) | — |

### Mutations

| Id | Planted | Test that turned red | Reverted, green |
| --- | --- | --- | --- |
| M-S26a-1 | uppercase accepted by `LookKey::new` | — | — |
| M-S26a-2 | `discloses` empty; then self-only | — | — |
| M-S26a-3 | `react` skips the insert | — | — |
| M-S26a-4 | `discloses` empty (wire) | — | — |
| M-S26a-5 | `generic()` renders the payload | — | — |
| M-S26a-6 | `look` removed from `SHOWN_ELSEWHERE` | — | — |

### Assumptions to verify in implementation

| Id | Assumption | Verified in | If false |
| --- | --- | --- | --- |
| A-S26a-1 | `naming`'s seeding pattern applies unchanged | C2, by reading naming first | follow what naming does; record |
| A-S26a-2 | serde-saphyr locates a `try_from` failure inside a `deny_unknown_fields` struct at the value | C3 | R-S26a-3 |
| A-S26a-3 | `mineworld server worlds/bodies-yard` starts with `--packs entities --packs presentation/mineworld-default` (or needs none) | C5 | follow `tools/cli/tests/bodies/`; record |
| A-S26a-4 | `generic()` renders `place=None` as "at somewhere" | C7 | assert the printed text; record |

### Deviations and findings

(none yet; planning findings are §3.2)

---

## Appendix A — Decision records (copy verbatim; fill nothing else)

### A.1 `ARC-91`

```markdown
## ARC-91 — Appearance is a perceivable trait of a Person, owned by the `appearance` System Pack

**Date** 2026-10-14 · **Status** accepted (step-26 freeze, operator Q-S26-11); implemented by S26-a · **Approved by** the operator (Q-S26-11, Q-S26-4, Q-S26-9) · **Relates to** `ARC-31` (sections), `ARC-33` (installed set), `ARC-46`, `ARC-92`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.2 (Traits), [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 · **Design** `.structured-coding/plans/mvp0/step-26-cast.md` §§4.1–4.2; `pr-s26a-appearance.md`

**Problem.** Every person must be drawn as themselves in both reference clients, and in any third client. A client can read only a person's entity id, tags, disclosed components and display name. None of them identifies an appearance: an id is a creation-order counter that renumbers when a place is added; a display name is absent without `naming`, can be shared, and would make a face change when a name changes or, if names are ever gated by acquaintance (`ARC-31`), when a stranger is introduced; a tag is semantic content that Systems and controllers read. Nobody owned an appearance, so each client invented one from a hash, and nine sprites came to serve twelve people.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) `hash(entity id)` modulo a pool | Declined: distinct only by luck; cannot say "Alice looks like this" |
| (b) The entity id listed in a Presentation Pack | Declined: ids are creation-order counters; an unreviewable list of numbers |
| (c) The display name | Declined: absent without `naming`; may collide; makes appearance depend on what the observer knows |
| (d) A presentation tag (`look:alice`) | Declined: pollutes a semantic vocabulary every consumer reads, with no owner |
| (e) A disclosed `look` component owned by a System Pack | **Chosen** |

**Choice.**

1. **`appearance` is a System Pack** on the `naming` pattern: it owns a person file's `appearance:` section (`{ look: <key> }`, unknown keys refused), states a public `appeared` fact at genesis, reduces it into the `look` component (`{ "look": "<key>" }`, schema 1) and discloses that to every observer who perceives the person. It depends on nothing and declares no action or process.
2. **A look is an opaque key**: 1–48 bytes of `a`–`z`, `0`–`9` and `-`. It names an appearance; it is not a mesh, a file, a texture, a colour, a height or any other renderer concept. A Presentation Pack resolves it (`ARC-92`).
3. **No descriptors.** Apparent age, stature, build, skin and hair are not world facts: no System consumes them. They live in the casting brief, in the Presentation Pack. A System that later needs stature makes it a `bodies` section.
4. **Appearance never depends on what the observer knows.** Disclosure is gated by perception only.
5. **Single ownership.** A later clothing, ageing or disguise System changes a look by having this pack state `appeared` again; no other pack writes `look`.

**Consequences.** A world without the pack, or a person without the section, has no look, and every client draws them with its fallback. The default worlds' keys are name slugs for review readability. A controller sees `look` as it sees any component; the memory renderer stores `appeared` generically and never shows the key as a name.

**Revisit** if names become acquaintance-gated (the default keys then become opaque codes in one data migration per world, R-S26-9), or when a System needs an appearance descriptor.
```

### A.2 `ARC-92`

```markdown
## ARC-92 — The cast binding lives in the Presentation Pack; one resolution order; the fallback is the extras pool

**Date** 2026-10-14 · **Status** accepted (step-26 freeze, operator Q-S26-8, Q-S26-9); specified by S26-a; read by the 3D client from S26-c2 and by the 2D client from S26-e · **Approved by** the operator (Q-S26-8, Q-S26-9, Q-S26-11) · **Relates to** `ARC-3`, `ARC-9`, `ARC-46` (amended, Note 1), `ARC-91`, `DEP-16`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1 · **Specified in** [`clients/3d-spike/PRESENTATION.md`](../clients/3d-spike/PRESENTATION.md), [`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) §3.1 · **Design** `.structured-coding/plans/mvp0/step-26-cast.md` §§4.3–4.4, 8

**Problem.** `ARC-91` gives every person a disclosed look key. Something must turn a key into a character in 3D and a sprite set in 2D, without the world learning anything about art, and with a fallback for a person whose look a pack does not know that is never a cruder kind of body.

**Choice.**

1. **3D: `characters/cast.yaml`** in a 3D Presentation Pack, in YAML's JSON-compatible subset: `format` (1), `looks` (look key → character id), `extras` (character ids: the fallback and decorative pool), `default` (the character used when no world is connected). A character id names `characters/cast/<id>/` in the pack; its meshes stay resources of the client that imports them (the 3D client cannot load a skinned, retargeted model from a runtime pack today).
2. **2D: a `look:<key>` role** in the `people` binding set. It is a disclosed component value, the same class of input as the tag in `facade:<tag>`, never a world key.
3. **One resolution order in both clients.** A person whose look the pack binds is drawn as that character or sprite. A person with no look, or one the pack does not bind, is drawn as `extras[posmod(hash(id string), len(extras))]` in 3D and as `person:<n>` by the same hash in 2D. A pack with no extras (3D) or no `person:<n>` (2D) uses its `default` (3D) or plain drawing (2D). With no pack, plain drawing. The observer is resolved the same way, so a player is drawn as the Person they occupy; the 2D `player` role is only the fallback for an observer with no bound look.
4. **The fallback is never a cruder model class.** Every `extras` entry is a character made by the same pipeline and held to the same standard as a named person (step-26 §6, which S26-b1 records as `ARC-94`). In the default pack, the café street's extras are the lakeside cast (Q-S26-8, E-2), so no person of the present world appears twice.
5. **A pack decides nothing.** A binding changes how a person looks, never which requests a client sends, and a look is never shown as text.

**Consequences.** The default 3D pack ships an empty but valid binding until its characters exist; `scripts/check_cast.py` checks its structure. Adding a world's cast is an edit to one pack file, never to a client or a world.

**Revisit** when 3D Presentation Packs become loadable at runtime (the binding then also names the meshes), or when a world that is neither the café town nor the lake needs its own extras (E-1).
```

### A.3 Note on `ARC-46`

```markdown
**Note 1, 2026-10-14 (S26-a; `ARC-92`) — the role list gains `look:<key>`.** A person's disclosed `look` (`ARC-91`) is asked for as `look:<key>` before `person:<n>`; for the observer, before `player`. A `look` is a disclosed component value, like the tag in `facade:<tag>`, so "the core never asks for a role by a world key" stands, and the list of role kinds gains one. Specified in `clients/2d/PRESENTATION.md` §3.1; read by the client from S26-e.
```

## Appendix B — `docs/MODULE_SPEC.md`

### B.1 §4.1, the row below `name`

```text
appearance appearance people  { look: <key> }: 1–48 bytes of a–z, 0–9 and '-'; unknown keys refused.
                              An opaque key naming the person's appearance, which a Presentation Pack
                              resolves (`ARC-92`); never a mesh, file, colour or height. Public:
                              disclosed to whoever perceives the person (`ARC-91`)
```

### B.2 §6.1

- In the tree of the shipped default, the 3D line becomes
  `└── 3D/           manifest.yaml · ART_DIRECTION.md · references/ · characters/cast.yaml`.
- After the sentence ending "…specified in [`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md)
  (`DECISIONS.md` `ARC-46`).", add: "What `characters/cast.yaml` contains for the 3D reference client —
  which character draws a person with a given disclosed `look`, and the extras that draw everyone else —
  is specified in [`clients/3d-spike/PRESENTATION.md`](../clients/3d-spike/PRESENTATION.md)
  (`DECISIONS.md` `ARC-92`)."

## Appendix C — `clients/3d-spike/PRESENTATION.md` (new; full text)

```markdown
# What a 3D Presentation Pack contains for the 3D reference client

**Audience:** coding agents writing or changing a 3D Presentation Pack, or the 3D client's code that
reads one. A specification, not an introduction; [`README.md`](README.md) is the orientation.
**Authority:** [`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-92` (the cast binding), `ARC-91` (the
`look` a world discloses), `DEP-16` (the file format), `ARC-3` (packs share meaning, not art).
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §6.1 names the file; this document specifies its
contents for this client. Where this document and the client disagree, that is a defect to be recorded
and fixed, never left.

**Status (2026-10-14).** Specified by S26-a. The client reads `characters/cast.yaml` from S26-c2 on;
until then it draws people as it did before, and the default pack's binding is empty. This is a
recorded gap, not a contradiction.

---

## 1. What the client reads from a 3D pack

Only the cast binding, as data. A character's mesh and texture are resources the client's Godot project
imports, under `clients/3d-spike/assets/characters/cast/<id>/`, because the importer applies the
humanoid bone map at import time and the client cannot load a skinned, retargeted model from a runtime
pack today. Each kind of file has one location; nothing is copied between them.

## 2. `characters/cast.yaml`

YAML's JSON-compatible subset: one JSON object, no comments (`DEP-16`). Godot's built-in `JSON` parser
reads it.

    {
      "format": 1,
      "looks":   { "<look key>": "<character id>" },
      "extras":  [ "<character id>" ],
      "default": "<character id>"
    }

- **`format`** — the integer `1`.
- **`looks`** — a disclosed `look` (`ARC-91`) to the character that draws that person. Several looks map
  to one character only where the operator has accepted it; the default pack maps none twice.
- **`extras`** — the characters that draw a person with no look, a person whose look this pack does not
  bind, and decorative figures. Distinct ids. Every entry passes the same standard as a named person
  (step-26 §6; `ARC-94` once S26-b1 records it); there is no cruder class.
- **`default`** — the character used when no world is connected: the standalone slice and portrait
  captures.
- A look key and a character id are each 1–48 bytes of `a`–`z`, `0`–`9` and `-`. A character id names
  the directory `characters/cast/<id>/` in this pack (its `character.yaml`, provenance, brief and
  approved references; `ARC-93` once S26-c1 records it) and the client resource directory of §1.
- A key the client does not know is ignored with a warning, so a newer pack degrades rather than fails.
  The default pack is held to no unknown key by `scripts/check_cast.py`.

## 3. Resolution

    person with a disclosed look this pack binds       → that character
    person with no look, or one this pack does not bind → extras[ posmod(hash(id string), len(extras)) ]
    this pack binds no extras                           → default
    no pack                                             → the client's plain figure

The observer is resolved the same way: a player is drawn as the Person they occupy (`INV-1`). The hash
is of the entity's id string, the same stable hash the 2D client uses for `person:<n>`
([`clients/2d/PRESENTATION.md`](../2d/PRESENTATION.md) §3.1).

## 4. What a pack may not do

It may not decide whether anything is valid, reachable or allowed; it changes how a person looks, never
which requests the client sends; and a look key is never shown as text — it chooses a character, it is
not a name.
```

## Appendix D — `clients/2d/PRESENTATION.md` §3.1 edits

1. Replace the `player` row with:
   `| `player` | the Person this client controls, when no `look:<key>` role binds its look | the observer's look is undisclosed or unbound |`
2. Insert, directly above the `player` row, the row:
   `| `look:<key>` | the Person whose disclosed `look` (the `appearance` System Pack's component, `ARC-91`) is `<key>`, the observer included | the person's disclosed `look`; asked before `player` and `person:<n>` (`ARC-92`) |`
3. Change "The client never asks for a role by a world key (`alice`, `cafe`), only by tag, index or hash."
   to "The client never asks for a role by a world key (`alice`, `cafe`), only by tag, disclosed component
   value (`look:<key>`), index or hash."
4. Append after that paragraph:

       A person is drawn by the first of these that the set binds (`ARC-92`, the same order as the 3D
       client's):

           look:<key>     the person's disclosed look
           player         the observer, when its look is unbound
           person:<n>     anyone else, by the stable hash of the id string
           (none)         plain drawing (§6)

       **Status (2026-10-14).** The `look:<key>` row is specified by S26-a and read by the client from
       S26-e on; until then the client asks for `player` and `person:<n>` only, and a pack's `look:`
       bindings are unused. This is a recorded gap, not a contradiction.

# Step 30 — S30: person attributes — a broad built-in catalogue, user-defined attributes, and dependency-aware population sampling

**Role:** step document for **S30**, a new step that the primary session adds to [`overall.md`](overall.md).
It records the operator requirement and audits what exists today: names, appearance, persona, authored
sections, perception, cognition's view of a person, the people files, and the interim casting sampler. It
then designs:

- who owns a person's attributes;
- the attribute model and its typing;
- the curated built-in catalogue, pluggable person-generation backends and masks, and the optional MatrAIx backend;
- user-defined attributes;
- population profiles and the sampler;
- how attributes reach Systems, cognition, agent players and both clients;
- persistence, digests and migration;
- the checks.

It ends with a PR split, the invariants, the risks and the questions only the operator can answer.

**Effort:** `mvp0`, for every PR: S30-a…S30-k, all release-blocking (§13.2, Re-ruled 2026-10-15 (operator)). The
original split (S30-h…S30-k in MVP-1) is kept visible in §13.2 and §16.1. The step document stays under `mvp0/`
because its first PRs gate S26's casting, which is MVP-0 work (§13.2 explains the placement).
**Parent:** [`overall.md`](overall.md).

**Amendment 2026-10-15 (Re-ruled 2026-10-15 (operator, final); history in §16.2).**

1. The built-in catalogue is **curated: at most 100 attributes, each read by something the player
   notices** (§6.1; 94 listed).
2. **Person generation is pluggable**: a world chooses a backend and a mask (§§6.4–6.5, `ARC-126`). The
   MineWorld backend is the default and needs nothing else.
3. **MatrAIx is an optional backend**: a separate pack importing their MIT schema unchanged, masked by
   default to `matraix-core-100`, with a demo (§§6.6–6.7). Nothing else in MineWorld depends on it.
4. **Everything ships before the public release**: S30-h (the MatrAIx backend and demo), S30-i, S30-j and
   S30-k are MVP-0 and release-blocking (§13.2).

**Lifecycle:** `APPROVED 2026-10-15 (step plan; PR designs to follow)`; amended the same day as above. The operator answered Q-S30-1…8
directly in the primary session on 2026-10-15, every one as recommended (§16.1).

- Superseded lifecycle: `STEP DESIGN (DRAFT)`, PR #223 revision 1.

- Nothing in this file authorizes implementation, a paid API call or a merge.
- Each PR is detailed to the commit in its own PR design and frozen in turn (`CLAUDE.md` §3.1). Only a PR
  design carries `DESIGN FROZEN`.
- Approval fixes this step's scope, its invariants (§12) and its step acceptance (§13.4). Progress,
  evidence, findings and bounded corrections stay writable.

**Author:** S30 design session (Opus), 2026-10-15.

- Worktree: `/Users/yuema137/mineworld-worktrees/design-s30`.
- Branch: `docs/s30-person-attributes`, from `origin/main @ 58a6f55f` (#204, S27-c merged).
- One writer: this file is the only file this session writes.

**Ids.** Assigned 2026-10-15 from the range the primary session reserved for S30 (`ARC-120`…`ARC-129`,
`DEP-66`…`DEP-69`), used from the bottom up. They replace the draft placeholders:

| Draft placeholder | Assigned id |
| --- | --- |
| `ARC-120` … `ARC-125` | `ARC-120` … `ARC-125` |
| `DEP-S30-a` … `DEP-S30-c` | `DEP-66` … `DEP-68` |

`ARC-126` is used by the amendment of 2026-10-15 (backends and masks, §6.4). `ARC-127`…`ARC-129` and `DEP-69`
stay reserved and unused. The records are written into `docs/DECISIONS.md`
by S30-a (§15.5).

Local ids in this file:

| Prefix | Meaning |
| --- | --- |
| `SD-S30-n` | step decisions |
| `A-n` | audit anchors |
| `F-S30-n` | findings |
| `AC-S30-n` | step acceptance |
| `X-S30-n` | planted mutations and adversarial tests |
| `I-S30-n` | invariants |
| `R-S30-n` | risks |
| `Q-S30-n` | operator questions |

**Parallel-safety.** This file is the only file this session writes. The edits it needs elsewhere are
**proposed** in §15 and made by the PRs that own those files:

- `overall.md`;
- `docs/CORE_CONCEPTS.md`, `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`;
- step-26, which the parallel "character sources" amendment is editing now (§10);
- step-27 and the S27-d design;
- step-28;
- S31, which is being designed in parallel (§9.6).

---

## 0. The answer in brief

1. **An attribute is declared data, and every attribute has exactly one owning System Pack** (§4).
   - A new **`attributes` System Pack** owns the plain traits: attributes that no specialised System
     models, such as stature, chronotype, Big Five scores, diet or languages spoken. It holds them in one
     typed `traits` component per Person.
   - An attribute that another System already models stays with that System. Examples are wealth
     (`economy`), occupation (`employment`), the look key (`appearance`), the persona prose (`persona`) and
     notice subscriptions (S31's pack). The catalogue declares such an attribute as **owned by** that pack.
     Its value is that pack's own state, read through a projection the pack provides, so it is a **view,
     never a copy**.
   - The kernel learns nothing: no word, type or id of the attribute vocabulary enters `kernel/` or
     `contracts/`.
2. **The model** (§5).
   - Every attribute is declared in YAML: id, group, facets, value kind, visibility, mutability, owner,
     and natural-language phrases.
   - The value kinds are `category`, `level` (ordinal), `quantity` (integer in a declared unit and
     scale), `flag`, `set` and `text` (bounded and never read by a System).
   - Visibility is one of `perceivable`, `learnable`, `private` or `hidden`.
   - Mutability is one of `fixed`, `evolving` and `derived`.
   - **Typed, not `map<string, any>`.** The catalogue is decoded once at load into a frozen `Catalogue`.
     A value is the closed enum `AttributeValue`, checked against its declaration at the line and column
     where it is written. A System reads only through a typed `Handle<K>`, obtained at install by
     declaring the attribute, and refused there by name if the attribute is missing or of another kind.
3. **The built-in catalogue** (§6).
   - It is a versioned data pack, `entities/person-attributes` (MineWorld's own wording, CC0).
   - **It is curated to at most 100 attributes** (Re-ruled 2026-10-15 (operator, final)): 94 in six groups (Appearance, Background,
     Psychology, Capability, Behaviour, Lifestyle), each with the reader that makes it noticeable in play.
   - **Person generation is a pluggable backend with masks** (§§6.4–6.5). The default is MineWorld's; an
     optional MatrAIx backend imports their MIT schema unchanged behind a default 100-dimension mask, with a
     demo (§§6.6–6.7).
4. **Users add attributes with zero code** (§7).
   - A world or pack drops a YAML file into its `attributes/` directory.
   - Ids are namespaced by the declaring pack. A world may restate presentation and narrow visibility
     through an explicit `overrides:` entry. It may never change a built-in's kind or value set, and a
     silent redeclaration is refused by name.
5. **Population profiles and the sampler** (§8).
   - A profile is per-world data (a reusable `populations/` pack file). It holds marginals, conditionals
     over a strict DAG, likelihood adjustments, compatibility masks, cited sources and `verify` marks.
   - The sampler is one deterministic Rust library. It works in integer arithmetic, is seeded by the
     world's population seed (never the run's `--seed`), and is stratified by largest remainder within
     each parent group.
   - **The biography always wins.** Values a person's file states, or that a lock file pins, are counted
     first and never re-drawn. Nothing reads a name.
   - The same library runs at genesis and in `mineworld attributes resolve --json`. The second is the
     resolved cast that S26 casting consumes (its C-1…C-7 contract, §10). It replaces the interim Python
     sampler on `origin/mvp0/s26-cast-director`.
6. **Use in the world** (§9).
   - Perceivable attributes are disclosed to whoever perceives the person. That closes AC-16's gap,
     because an agent today learns only a look key where a human sees a face.
   - Private attributes are disclosed to the holder alone, and hidden attributes to no one.
   - Cognition renders the catalogue's phrases into "who I am" and "who I see" under a byte budget.
     Decision models read the same values from their observation. MCP gains a short `description` line
     per person and `me.attributes`. Both clients show an inspect panel.
   - An attribute **learned** from someone is a memory (S28's private layer). It is never world state
     and never widens an observation.
7. **Persistence** (§11).
   - A world with the `attributes` pack gains one genesis fact per person (`traits-stated`) and, later, an
     `trait-changed` fact for each change its owner accepts.
   - Derived attributes, such as age from birth, write nothing.
   - Adding the pack to a reference world moves its digests once, under one recorded re-baseline. A world
     without the pack stays byte-identical.
8. **The PR split** (§13). In MVP-0:
   - S30-a: specifications and records;
   - S30-b: the authoring seam;
   - S30-c: the `attributes` pack;
   - S30-d: profiles and the sampler;
   - S30-e: the curated built-in catalogue (≤ 100, gameplay-driven), backends and masks, and the cited profiles;
   - S30-f: consumers;
   - S30-g: reference worlds and the S26 switch.

   Also MVP-0 and release-blocking (Re-ruled 2026-10-15 (operator)): S30-h, the optional MatrAIx backend,
   its masks, mapping and demo; S30-i, learning attributes in conversation; S30-j, runtime sampling of new
   people; S30-k, localisation including zh-Hans. (Originally S30-h…k were MVP-1.)
9. **MatrAIx** (§6.6, `DEP-66`, Re-ruled 2026-10-15 (operator, final)).
   - **An optional backend, never a dependency.** Its MIT schema is imported unchanged into its own pack,
     with attribution; nothing outside that pack names it, and the workspace builds and passes without it.
   - **Excluded on licence grounds only:** the 50 dimensions with a non-MIT `source_origin.license` and the
     research-only Persona-1M coreset.
   - **Their sampler code is not used;** whether to offer their `full_dag.json` as an optional faithful
     sampling mode is Q-S30-9.

---

## 1. Binding inputs

### 1.1 Operator requirements, 2026-10-15 (verbatim)

> 我们这些都要做成模块化的，而不是hard code，这样用户可以根据他们喜欢的方式…喜欢的特征分布（年龄体重族裔）来搭建自己的世界

> 年龄体重族裔也不一定是所有特征，其实还可以有很多很多

> 人物的属性也可以有很多很多，我们应该支持大部分，然后也支持自定义添加。你可以看下这篇paper https://arxiv.org/html/2608.04205v1

In English:

- All of this must be modular and data-driven, not hard-coded, so that users build worlds with the method
  they like and the feature distributions they like (age, weight, ethnicity).
- Age, weight and ethnicity are not all the features; there can be very many more.
- A person can have very many attributes. MineWorld should support most of them out of the box and also
  support adding custom ones. The paper is MatrAIx (arXiv 2608.04205).

### 1.2 Coordinator inputs relayed with this task (2026-10-15)

1. **Ownership, model, sampling, use, persistence, acceptance and the PR split**, as scoped in the task.
   §§4–13 follow that scope in order.
2. **S26 "character sources" coordination.** The parallel S26 amendment (§7.4b, `docs/s26-character-sources`,
   local commit `dcb59725`, not yet pushed) owns the providers, the casting config and the gates. S30
   owns the attribute schema and the sampler. The interface is stated in §10.
3. **Public-information subscriptions (operator, 2026-10-15, relayed during the session).**
   - Each NPC has an attribute that says which public-information topics it subscribes to: price
     bulletins, station departures, the newspaper, weather warnings and so on.
   - S31 (`docs/s31-wealth-prices`) now includes a generic time-limited public-notice publish/subscribe
     System and a price-reporter NPC.
   - **Shape:** a set of topic ids, declared as data.
   - **Mutability:** mutable, owned by whichever pack S31 decides.
   - **Defaults:** may follow role or occupation; for example, a cook subscribes to ingredient prices.
   - **Visibility:** private.
   - S30 treats it as the worked example of a behaviour attribute that a System, not cognition, reads,
     and states the interface S31 consumes (§9.6).

### 1.3 Standing rules that bind this step

- `CLAUDE.md` §4, in particular:
  - rule 1, single ownership;
  - rule 2, kernel ignorance;
  - rule 5, change amplification;
  - rule 7, strong typing at boundaries;
  - rule 10, deterministic and LM-independent;
  - rule 11, no premature abstraction;
  - rule 12, no early compatibility debt;
  - rule 16, reuse before reinvention.
- `CORE_CONCEPTS.md`:
  - INV-4, memory ≠ biography ≠ world truth;
  - INV-7, only owners write;
  - INV-12, no domain semantics in the kernel;
  - INV-13, observations, never global state;
  - INV-15, traceable mutation;
  - §3.1, components are typed and versioned;
  - §4.2, traits change only through an explicit System.
- `ARC-31`: a System Pack owns a section; the loader never learns what a section means.
- `ARC-26`: a pack emits another's vocabulary only by declaring it and depending on its owner.
- `ARC-91`: appearance is a perceivable trait, a look key only. Item 3 says "no descriptors", with a
  revisit clause that §4.4 invokes.
- `ARC-74`: `persona`, prose disclosed to self (P6a, implemented on an unpushed branch).
- Step-26 §7.4a: one style, real-world diversity by data, a seeded stratified sampler, the biography
  wins, nothing from surnames, a malformed profile fails by name, and X-24/X-25.
- Step-27 INV-S27-2: same perception. AC-16 asks for an information-rich interface.
- Step-28 I-S28-1: perception is the only access right. Memory is controller-owned.
- Standing operator preferences:
  - all three platforms;
  - realistic defaults with citations;
  - framework, not demo: the café town is one composition;
  - every NPC to the protagonist's standard;
  - maintainable, debuggable infrastructure: know where to fix.

---

## 2. The requirement, restated as checkable outcomes

| # | Outcome | Where met | Checked by |
| --- | --- | --- | --- |
| O-1 | A Person can carry many attributes (hundreds), in named groups | §5, §6 | AC-S30-1 |
| O-2 | Most attributes a world needs exist out of the box | §6 | AC-S30-2 |
| O-3 | A world adds its own attributes with no code change, and they flow everywhere a built-in does | §7 | AC-S30-3 (`freckles`) |
| O-4 | A world chooses its own distributions (age, build, ethnicity and any other), by data | §8 | AC-S30-4, AC-S30-5 |
| O-5 | Sampling respects dependencies (stature given sex and age) and impossibilities (masks) | §8.3 | AC-S30-6 |
| O-6 | The biography wins; nothing is inferred from names; sampling is reproducible | §8.5 | AC-S30-7, AC-S30-8 |
| O-7 | Attributes have one owner each; changes are traceable; derived values are views | §4 | AC-S30-9 |
| O-8 | Visibility holds: a private attribute never reaches another seat | §5.4, §9 | AC-S30-10 |
| O-9 | Cognition, agent players, both clients and Systems read attributes through declared, typed paths | §9 | AC-S30-11 |
| O-10 | The kernel learns nothing | §4.1 | AC-S30-12 |
| O-11 | S26 casting consumes the appearance subset with no hard-coded attribute list | §10 | AC-S30-13, S26's X-S26src-10 |
| O-12 | Subscription topics for public notices are an attribute S31's System reads | §9.6 | AC-S30-14 |
| O-13 | Person generation is pluggable: a world chooses a backend and a mask by data | §§6.4, 6.5 | AC-S30-15 |
| O-14 | The MatrAIx backend is optional: nothing outside its pack depends on it | §6.6 | AC-S30-16 |
| O-15 | The MatrAIx backend has a working demo | §6.7 | AC-S30-17 |

*Table repaired 2026-10-15 (`pr-s30a-attribute-specs.md` F-S30a-6): rows O-12 … O-15 and AC-S30-15 … AC-S30-17 had
been merged into one malformed block; the three acceptance rows now sit in §13.4.*

---

## 3. Audit (`origin/main @ 58a6f55f`, 2026-10-15)

### 3.1 Anchors

| # | Fact | Evidence |
| --- | --- | --- |
| A-1 | `AuthoredSection` (`ARC-31`): `SECTION`, `CARRIED_BY: &[ContentKind]`, `type Authored: DeserializeOwned`, `references`, and `seed(seeding, subject, authored) -> Result<Vec<Emission>, Rejection>`. Decoding is static: the section's type cannot see any world-level data, so a schema declared in data cannot be checked by `serde` alone at the section's line | `authoring/src/section.rs:172-203` |
| A-2 | `ContentKind` is `Person`, `Place`, `Item` or `Organization`. `world.yaml` carries no sections; its `population:` key lists the people files. World-level settings are a pack's `configure/<id>.yaml` (`PackConfiguration`, `ARC-61`). A pack owns **at most one** section (`SystemPack::SECTION: Option<SectionOwner>`), and a section is decoded through `Decode<S>: DeserializeSeed` so that errors keep line and column | `authoring/src/section.rs:65-78`; `authoring/src/configuration.rs`; `authoring/src/content.rs`; `sdk/rust/src/pack.rs:153`; `worlds/social-cafe/world.yaml:61` |
| A-3 | `naming` is the template: `name: Alice Moreau` → `Named` genesis fact → `DisplayName` component, public | `systems/naming/src/section.rs`; `ARC-31` |
| A-4 | `appearance` (S26-a, PR #206, not on `main`): `appearance: { look: <key> }` → `appeared` → `look` component, public. **No descriptors** (`ARC-91` item 3), with "Revisit … when a System needs an appearance descriptor" | `origin/mvp0/pr-s26a:docs/DECISIONS.md` `ARC-91` |
| A-5 | `persona` (P6a, `ARC-74`, implemented but unpushed): `persona: { summary ≤ 280 B, traits ≤ 8, speech ≤ 160 B }`, disclosed to the holder only, shaping speech only | `/Users/yuema137/mineworld-worktrees/impl-s10-p6a` @ `71f76bd8`; `pr-s10-p6-lm-controller.md` §5.14 |
| A-6 | People files hold `name`, `routine`, `tags`, `note` and `location`. `note` is "authoring provenance, never gameplay state". There are no attributes | `worlds/social-cafe/people/alice.yaml`; `MODULE_SPEC.md` §4.1 |
| A-7 | An Entity Pack may declare "component schemas … tags and taxonomies". MVP-0 reads item kinds only, and component schemas belong to System Packs | `MODULE_SPEC.md` §2 |
| A-8 | A decision model sees only `Consult { seed, pace, observation }`. Randomness is the counter-based SplitMix64 `mix`, the tree's deterministic idiom | `MODULE_SPEC.md` §5.1; `cognition/decision/src/draw.rs` |
| A-9 | The interim casting sampler is Python stdlib: `random.Random(sha256(seed | attempt | world | attribute | key))`, float weights, largest remainder within `given` groups, `allowed` restrictions, and a re-draw "attempt" on distinctness failure. It has its own YAML-subset reader | `origin/mvp0/s26-cast-director:tools/character_pipeline/casting/cast_director.py` (@ `9ddeb571`) |
| A-10 | The interim San Diego profile (v2) uses kinds `categorical`, `integer_range` and `normal`, with `given`, `stratify`, `replace` (grey hair by age), `labels` and per-figure sources marked `read`/`verify`. Its attributes are sex, age band, apparent age, ancestry, build, stature, MST skin tone, hair texture/colour/style, facial hair, face shape, eyes, nose and lips. `ancestry` is "recorded for the distribution only, never written into a prompt" | `origin/mvp0/s26-cast-director:presentation/mineworld-default/3D/characters/populations/san-diego-county.yaml` |
| A-11 | S26's amendment draft states the consumer contract C-1…C-7 on S30's output and leaves the schema and the sampler to S30 | local branch `docs/s26-character-sources` @ `dcb59725`, `step-26-cast.md` §7.4b.1 |
| A-12 | The agent View's `PersonLine` is `{ ref, name, labels, position, walking, doing }` and `MeLine` adds wallet, holdings, job, agenda, acquaintances and invitations. `view.py` reads components by name (`display-name`, `walking`, …); anything else is reachable only through `inspect` (≤ 4,096 B). `look` is not read. World-authored strings are `WorldText` (INV-S27-12). S27-d's MCP design is frozen with 13 closed tools and a tool golden | `clients/agent/src/mineworld_agent/models.py`, `view.py`, `disclosed.py`; `step-27-agent-players.md` §5.3; `origin/docs/s28a-s27d-designs:…/pr-s27d-mcp-stdio.md` |
| A-17 | Per-person state on `main`: `economy` `Wallet { balance }` (section `economy: { wallet }`, holder only); `employment` `Employment { job, shift }` (section `job:`, holder only); `inventory` `Holdings`; `schedule` `Routine`/`Agenda`; `relationships` `Acquaintances`. Self-only disclosure is each pack's `PerceptionProvider::discloses` comparing observer and subject. There is no age, birth, skill or personality anywhere | `systems/economy/src/system.rs:248`, `systems/employment`, `systems/presence/src/interaction.rs:80` |
| A-18 | On `main` the only LM prompts are memory summarisation. The persona block is P6b's (open, `origin/mvp0/pr-s10-p6b`). Memory renderers are per pack (`memory/render/naming.py` …) with a registry | `cognition/lm-controller/src/mineworld_cognition/memory/render/__init__.py:140` |
| A-19 | Adding genesis facts shifts every later event id, so S26-a re-recorded every world's baselines with a reason line, plus the `decisions` golden `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` | `origin/mvp0/pr-s26a:scripts/baselines.txt`; `tools/cli/tests/decisions.rs:120-130` |
| A-13 | AC-16: an agent's observation is "at least as information-rich as what a human sees and can infer on screen … within the same perception limits" | `docs/MVP.md` AC-16 |
| A-14 | Memory is controller-owned. S28's private layer holds `stable` facts and `names` per seat. A scope can index and recall but never grant access | `step-28-shared-memory.md` §§4.4, 5.1 |
| A-15 | Baselines are re-recorded per world with a `reason <PR>:` line. A PR that changes a world's history updates its section | `scripts/baselines.txt` header; `ARC-83` |
| A-16 | `systems/` on `main`: bodies, calendar, consumption, conversation, economy, employment, group-activity, inventory, item, item-transfer, movement, naming, presence, relationships, schedule, weather. None holds a person trait | `ls systems/` |

### 3.2 MatrAIx (read 2026-10-15)

| # | Fact | Evidence |
| --- | --- | --- |
| M-1 | The paper describes 1,290 categorical dimensions in five groups: Background 238, Psychology 210, Capability 331, Behaviour & Interaction 124 and Lifestyle 387. Records may be partial (null), and each value has a natural-language description | arXiv 2608.04205v1, Table 1 |
| M-2 | Sampling: p(x_i \| pa) ∝ π_i(v) · r_i(v; pa) · m_i(v; pa), forward in topological order. r is a source likelihood ratio; m ∈ {0,1} is a hard mask; rare valid combinations stay eligible | paper §3 |
| M-3 | Code: `github.com/MatrAIx-ai/MatrAIx-Persona-8B`, **MIT** licence (GitHub API), last pushed 2026-09-18 | `gh api repos/MatrAIx-ai/MatrAIx-Persona-8B` |
| M-4 | The schema is released as `persona/schema/dimensions.json` (604 KB, schema 1.0). It has 1,290 dimensions with `id`, `label`, `category` (43 categories), `description`, `values` (2–16, mean 4.9), `phrase` (for example `"aged {value}"`), `defaultValue` and an optional `source_origin`. There are 6 locale label packs, including zh-Hans | repository file |
| M-5 | 50 dimensions carry `source_origin.license` of `citation-only` (28) or `research-use-reference` (22). They are theory construct labels such as Schwartz values or BFI-2 facets, "survey/questionnaire items are not copied" | `dimensions.json` |
| M-6 | Coverage is software-user oriented: Expertise 208, Skills/Programming/Tools 113, Developer 66, Interests 358. It includes UX needs such as `health_text_size_need` and `health_assistive_tech`. It has **no body or appearance dimension**: no stature, weight, hair, eyes or skin. Ethnicity is `demo_ethnicity_broad`. **Every dimension is categorical**: no numeric with a unit | `dimensions.json` |
| M-7 | The released graph `persona/synthesis/graph/full_dag.json` (24.8 MB, v4.4) has 1,308 nodes and 6,999 directed edges, 524 conditional masks and 54 full CPTs. 5,432 of the edges are `world_marginal_calibrated_likelihood_ratio_transfer`, with rationales such as "plausibly shapes". It "is not forced to be a DAG" and adds energy scoring and repair. The priors are world-level (for example 8.3 % of people under 5) | repository file, `metadata` |
| M-8 | The Persona-1M coreset on Hugging Face (`MatrAIx2026/MatrAIx_Persona_1M`) is licensed **`matraix-research-only`**. 60 % of it is human-grounded, from Wikipedia biographies, Amazon reviews, Stack Overflow, GSS and PRISM | dataset card |

### 3.3 Findings

| # | Finding | Consequence |
| --- | --- | --- |
| F-S30-1 | Nothing in the world holds a person's attributes. Casting keeps appearance in Presentation Pack briefs, `persona` keeps prose, and cognition has nothing typed to read | the requirement is new capability, not a refactor |
| F-S30-2 | `ARC-31`'s static decode cannot validate a data-declared schema at the author's line (A-1), and no section can see the whole population at once (A-2, A-3). Stratified sampling needs both | an additive authoring seam (§4.3, `ARC-121`), not a kernel change |
| F-S30-3 | `ARC-91` item 3 holds that apparent age, stature, build, skin and hair "are not world facts: no System consumes them". AC-16 (A-13) is a stronger consumer than a System: an agent must learn what a human sees. Today it learns a look key | `ARC-91` item 3 is amended (§4.4, Q-S30-1) |
| F-S30-4 | The interim sampler uses `random.Random` and floats (A-9). That is reproducible on one CPython, but it is not one implementation shared with the world's genesis, and casting and the world can drift | one Rust sampler, two entry points (§8.6) |
| F-S30-5 | The interim profile mixes data (ACS, NHANES) with casting conventions (MST ranges per ancestry, hair-style lists constrained by route D+'s limits) in one file (A-10) | profiles separate cited distributions from authored conventions, and provider limits stay S26's `unsupported` declarations (§8.2, §10) |
| F-S30-6 | MatrAIx has no appearance group and no numeric kind (M-6) | a sixth group and the `quantity` kind are MineWorld's own (§5.2, §6) |
| F-S30-8 | `population:` in `world.yaml` already means "the people files" (A-2) | S30 never names a key `population` at world level; its settings live in `configure/attributes.yaml` |
| F-S30-9 | `kernel/` already uses the word `trait` (Rust) and `contracts/` the word `attribute` in prose | AC-S30-12 greps for S30's vocabulary itself (type names, the `traits` component type, built-in slugs), not common English words |
| F-S30-10 | `World::create_entity` exists (`kernel/src/world.rs:572`), but no System on `main` creates a Person at runtime | S30-j proves runtime sampling with a fixture creator; a real newcomer source (S21 migration, a happening's arrival) is that System's own PR and uses S30-j's entry |
| F-S30-11 | MatrAIx's ids contain `_` and its values are free strings ("White / European"); its `defaultValue` is a string, a list (216) or null | the id rule admits `_`; imported category values are kept verbatim; a list `defaultValue` is kept but never used unless a profile says so (an unprofiled attribute stays absent) |
| F-S30-7 | A person's subscriptions (§1.2 item 3) are mutable state that one System reads to deliver notices. That System must own them, so not every attribute can live in one pack | the owner field (§4.2) |

---

## 4. Ownership (`ARC-120`)

### 4.1 Options

| Option | For | Against | Verdict |
| --- | --- | --- | --- |
| (a) One `attributes` pack owns **every** attribute | one place to look | it would copy wealth from `economy` and occupation from `employment`, breaking single ownership (`CLAUDE.md` §4 rule 1). Subscriptions would need S31's System to ask another pack to change its own routing table | declined |
| (b) Each System Pack declares the attributes it needs, and there is no shared schema | no new pack | every reader (cognition, MCP, clients, casting, the sampler) needs a per-pack adapter, which is change amplification. Hundreds of plain traits would have no owner at all | declined |
| (c) **One declared schema (the catalogue) and one owner per attribute. A default `attributes` pack owns plain traits; specialised packs own theirs** | single ownership holds; one reading path for every consumer; a plain trait needs no code | an owning pack implements a small trait to expose its attribute | **chosen** |

### 4.2 The rule (SD-S30-1)

1. **Every attribute has one owner**, named in its declaration (`owner:`). The default owner is
   `attributes`.
2. **`attributes` owns plain traits.** These are attributes that no installed System models. The pack
   stores them in one component per Person, `traits` (schema 1). That component is a sorted list of
   `(attribute slug, value)` pairs, each checked against the catalogue. The pack provides no action. It
   reacts to one proposal event (§4.5), and it derives `derived` attributes on disclosure (§5.5).
3. **A specialised owner keeps its own state.** When `economy` owns `wealth-band`, its value is a
   projection of `economy`'s own component, computed when the attribute is read or disclosed. `attributes`
   never stores it, so it is a **view, not a copy**.
   - The owner implements `AttributeOwner`, from the model crate (§4.3). It names the attributes it owns
     and projects each one from its own component.
   - Because the projection is the owner's code, a view on another pack's state cannot be made with data
     alone. That limitation is accepted (§7.4).
4. **An owner seeds its own attributes.** At genesis the resolved value of an attribute owned by pack P
   (authored, pinned, defaulted or sampled, §8.5) is handed to P's seed as a typed value. P states it in
   its own vocabulary. For example, S31's pack states the initial subscriptions, and `economy` states an
   opening balance drawn from a `wealth-band`, if S31 chooses that (§9.6, Q-S30-6).
5. **No owner, no attribute.** A declared attribute whose owner is not installed in the world is refused
   at load by name, as `ARC-31` refuses an unowned section. A world author who wants the value as a plain
   trait declares their own attribute.
6. **The kernel stays ignorant** (INV-12, `CLAUDE.md` §4 rule 2). The catalogue, the values, the handles
   and the sampler live in a crate above the contracts and below the packs (§4.3). `kernel/` and
   `contracts/` gain no type, word or id from it. AC-S30-12 greps them.

### 4.3 Where the code lives

```text
contracts/ (unchanged)
   ▲
authoring/                       mineworld-authoring: ARC-31 seam, extended by ARC-121 (below)
   ▲
authoring/attributes/            mineworld-attributes-model  (new, S30-c)
   │  Catalogue, AttributeId, AttributeValue, Handle<K>, AttributeOwner,
   │  catalogue decoding and validation, phrase rendering, the sampler (S30-d)
   ▲
systems/attributes/              mineworld-attributes  (new System Pack, S30-c)
systems/<owner>/                 any owner pack (economy, S31's pack …) depends on the model crate only
tools/cli                        `mineworld attributes …` (catalogue, show, resolve, check)
```

- The model crate depends on `mineworld-contracts`, `mineworld-authoring`, `serde` and `serde-saphyr`.
  `serde-saphyr` is already used for line and column (`DEP-10`). The crate depends on no pack.
- Owner packs depend on the model crate, not on `attributes`. A world may therefore have owned attributes
  without the `attributes` pack, for example only S31's subscriptions.

**The authoring seam (`ARC-121`, S30-b).** This is additive. No existing section changes, and no
existing world's bytes move.

1. **A System Pack may own a data directory** in World Packs and Entity Packs. `attributes` owns
   `attributes/` (declarations) and `populations/` (profiles). The loader collects the files, in
   composition order, and hands them to the owner to decode with line and column. It never learns what
   they mean. The mechanism is the same as for sections: an unknown directory is refused and the error
   lists the directories this build knows.
2. **No world-level section is needed.** World-level settings (the profile, the population seed and the
   lock file, §8.1) are `attributes`'s world configuration, `configure/attributes.yaml`, through the existing
   `PackConfiguration` mechanism (`ARC-61`; `authoring/src/configuration.rs`). `world.yaml`'s own
   `population:` key (the list of people files) keeps its meaning, which is why S30 never uses that word for a
   key (F-S30-8).
3. **A section may be decoded in its owner's context.** The loader already decodes a section through
   `Decode<S>: DeserializeSeed` (`authoring/src/content.rs`). The seam adds an optional `type Context`,
   decoded from the owner's data directories and its configuration, which `Decode` carries. It lets the
   `attributes:` section of a person's file be checked against the catalogue at the value's own line and
   column.
4. **An owner may resolve across files before seeding.** An optional `resolve(context, all carried
   sections) -> per-subject seeds` is called once, before any per-subject `seed`. This is where the sampler
   sees the whole population, which stratification needs. Seeding order is unchanged (`ARC-31` item 4).

**Notes 2026-10-15 (`pr-s30b-authoring-seam.md`; Q-S30b-1 ruled 2026-10-15, operator answered directly in the
primary session).**

- *Item 3* is read as semantics: "a section may be decoded in its owner's context". On Rust 1.97 an optional
  associated type cannot be defaulted, and a required one would break every existing section, so S30-b realises
  it as a sibling trait (`SectionInContext`; F-S30b-1).
- *Item 1* admits, besides the format's and the installed packs' directories, directories whose name begins with
  `.` (ignored) and the directory holding the `decisions:` file; an installed pack's directory in a world that
  does not enable it is refused naming the pack (F-S30b-3).
- *Item 5 (new): shipped data.* A System Pack may ship data files for an owner's directory, compiled in
  (`SystemPack::SHIPS`, declared with `ships_data!`). They are collected first, in `systems:` order, whenever the
  shipping pack is enabled, and each refusal names the pack and the crate-relative path. This is how S31's notices
  pack, `economy` and `employment` declare their attributes (§9.6 N-2).

The seam is general. It does not mention attributes. Its first user is S30. A second plausible user is
an Entity Pack's taxonomies (`MODULE_SPEC.md` §2), which explains why it belongs in `authoring/` and not
inside the attributes pack.

### 4.4 Relation to `appearance`, `naming` and `persona` (SD-S30-2)

| Pack | What it keeps | What changes |
| --- | --- | --- |
| `naming` | `DisplayName`, public | nothing. A name is not an attribute: it is identity, and names are never sampler input (§8.5) |
| `appearance` | the opaque `look` key, public, resolved by Presentation Packs (`ARC-92`) | nothing in code. **`ARC-91` item 3 is amended** (Q-S30-1): apparent descriptors become `perceivable` attributes in the Appearance group, owned by `attributes`, so that an observer (human or agent) is told what a human sees. `look` stays the only thing a renderer binds. A descriptor is never used to pick art, and the look key never encodes a descriptor |
| `persona` (P6a) | prose: summary, traits, speech; self-only | nothing in code. Its three fields are catalogued as `text` attributes owned by `persona`, with visibility `private`, through `AttributeOwner`, so one renderer covers them. Recommended: P6a merges as frozen, and the overlap between `persona.traits` and the typed Psychology group is reviewed once S30-f has shipped (Q-S30-4) |

`ARC-91`'s revisit clause names this case: "when a System needs an appearance descriptor". The consumer is
cognition and AC-16's agent view rather than a System. The amendment keeps `ARC-91`'s principles intact:

- a look is still opaque;
- appearance still never depends on what the observer knows;
- disclosure of descriptors is still gated by perception only.

### 4.5 Change over time (SD-S30-3)

| Mutability | Meaning | Who changes it | Example |
| --- | --- | --- | --- |
| `fixed` | set at genesis; never changes | nobody | `birth`, `natal-sex`, `eye-colour` |
| `evolving` | changes only through its owner, on an event the owner accepts | the owner | `hair-colour` (ageing, or a later dye action), `skill.cooking`, S31's subscriptions |
| `derived` | never stored; computed when it is read, from stored state and the clock | the owner's projection | `age-years` (birth and the current instant), `economy.wealth-band` (economy's wealth: wallet plus reference values, step-31 §7.4) |

*Note 2026-10-15 (S30-a, F-S30a-2; step-31 §7.4 and §19.6).* The example read "`wealth-band` (economy's
balance)". It is `economy`'s attribute in `economy`'s namespace, and S31 projects it from wealth, not from the
balance alone.

- **A change to a trait that `attributes` owns** is proposed with an `attribute-change-proposed { person,
  attribute, to, because: EventId }` event. `attributes` owns that vocabulary. A System emits it only by
  declaring it and depending on `attributes` (`ARC-26`).
- `attributes` checks the proposal:
  - the attribute is `evolving`;
  - the value is in its domain;
  - the proposer declared the attribute in its `proposes:` list.
- `attributes` then states `trait-changed { person, attribute, from, to, because }` or refuses by name.
  INV-15 holds: every change cites its cause.
- **No rule engine for drift ships in MVP-0.** Skill growth or ageing hair belongs to the System that
  models it, which emits proposals (`CLAUDE.md` §4 rule 11).
- **Age advances with no event.** `age-years` and `age-band` are derived from `birth` (a world instant,
  possibly before the epoch) and the observation's instant. Kernel time suffices, so no calendar is
  needed.
- **A specialised owner changes its own attribute by its own actions.** For example, S31's `subscribe` and
  `unsubscribe` actions change subscriptions. `attributes` is not involved.

---

## 5. The attribute model (`ARC-122`)

### 5.1 A declaration

```yaml
# entities/person-attributes/attributes/appearance.yaml (excerpt)
format: 1
attributes:
  stature:
    group: appearance
    facets: [appearance, body]          # S26 reads `appearance` (C-1)
    kind: { quantity: { unit: m, scale: 3, min: 0.500, max: 2.500 } }   # stored as integer mm
    visibility: perceivable
    mutability: fixed
    label: Stature
    phrase: "{value} m tall"             # the {value} placeholder is the only one a phrase may name
    perceived_as: { bands: [ { below: 1.55, words: short }, { below: 1.85, words: of average height },
                             { words: tall } ] }   # what an observer is told; the holder sees the value
  hair-colour:
    group: appearance
    facets: [appearance]
    kind: { category: [black, dark-brown, brown, light-brown, blond, auburn, red, grey, silver-grey, white] }
    visibility: perceivable
    mutability: evolving
    label: Hair colour
    phrase: "{value} hair"
    words: { dark-brown: dark brown, light-brown: light brown, silver-grey: silver-grey }
  chronotype:
    group: behaviour
    kind: { level: [early-bird, morning-leaning, neutral, evening-leaning, night-owl] }
    visibility: learnable
    mutability: evolving
    label: Chronotype
    phrase: "{value}"
```

Fields:

| Field | Required | Meaning |
| --- | --- | --- |
| id (the key) | yes | a slug, `^[a-z][a-z0-9_.-]{0,63}$`, so that an imported schema's ids are kept verbatim (§6.6). It is qualified by its namespace (§7.1) |
| `group` | yes | one of the six groups (§6.1), or a group the declaring pack declares |
| `facets` | no | open tags for selection by consumers (`appearance`, `body`, `prompt`, `casting` …) |
| `kind` | yes | §5.2 |
| `visibility` | yes | §5.4 |
| `mutability` | yes | §4.5 |
| `owner` | no | the owning System Pack id; default `attributes` (§4.2) |
| `label`, `phrase`, `words` | `label` yes | the natural-language rendering (§5.6) |
| `perceived_as` | no | what an observer other than the holder is told, when it is coarser than the value (§5.4) |
| `default` | no | the value when the profile and the person's file state none. If absent, the attribute stays absent (partial records, M-1) |
| `since` | built-ins | the catalogue version that introduced it (§6.3) |

### 5.2 Value kinds

| Kind | Declared as | Stored as | Example |
| --- | --- | --- | --- |
| `category` | a closed list of value slugs | the value's index in the list, saved as its slug | `hair-colour` |
| `level` | an ordered list of value slugs | as `category`, and the order is meaningful (comparisons, bands) | `big5-extraversion` (very-low … very-high) |
| `quantity` | `unit` from a closed list, `scale` (decimal places), `min` and `max` | `i64` at that scale (for example millimetres for `m` at scale 3). No float is stored or compared | `stature`, `mass` (kg, scale 1) |
| `flag` | — | `bool` | `wears-glasses` |
| `set` | `of:` a closed list, or `vocabulary:` naming an id set an owner declares as data; `max` | a sorted set of indices, saved as slugs | `languages-spoken`, S31's `subscriptions` |
| `text` | `max_bytes` ≤ 280 | bounded UTF-8 with no control characters. **Never readable by a System** (§5.3) | `persona.summary`, `signature-phrase` |
| `instant` | — | a world instant, in seconds and possibly negative | `birth` |

- **Units are a closed list in the model crate:** `m`, `kg`, `year`, `day`, `hour`, `count` and `ratio`.
  - Currency is absent on purpose, because money is `economy`'s.
  - A new unit is a model-crate edit, reviewed as such. That is rare and deliberate, since a unit changes
    what comparisons mean.
- **`unknown` is not a value.** An absent attribute means "not stated". Rendering omits it, and a System
  handle returns `None`. This is the partial-record model (M-1), without a null sentinel inside every
  domain.

### 5.3 Typed values and typed reads (strong typing at the boundary)

```rust
// mineworld-attributes-model (sketch; S30-c's PR design fixes the exact form)
pub struct AttributeId(u16);                  // interned index into this world's frozen Catalogue;
                                              // process-local, never saved (saves carry the slug)
pub enum AttributeValue {
    Category(ValueIndex), Level(ValueIndex), Quantity(i64), Flag(bool),
    Set(SmallSet<ValueIndex>), Text(BoundedText), Instant(SimInstant),
}
pub struct Handle<K: Kind> { id: AttributeId, kind: PhantomData<K> }
impl Catalogue {
    pub fn handle<K: SystemReadable>(&self, slug: &QualifiedSlug) -> Result<Handle<K>, AttributeError>;
}
pub trait SystemReadable: Kind {}            // implemented for Category, Level, Quantity, Flag, Set, Instant;
                                             // deliberately not for Text
```

- **Validated at load, refused by name.** Each value in a person's `attributes:` section is decoded by
  `DeserializeSeed` against its declaration (seam item 3). A wrong kind, an out-of-domain value, an
  unknown attribute or a value outside `min`/`max` is refused at its line and column. The error message
  names the attribute, the value, the domain and, for an unknown slug, the nearest declared slugs.
- **No string lookups scattered in code.**
  - A System that reads an attribute declares it in its manifest (`reads_attributes: [core.chronotype]`).
  - It obtains `Handle<Level>` once, at install.
  - Install fails by name if the attribute is not in the world's catalogue (`AttributeError::Missing`) or
    is of another kind (`WrongKind { declared, requested }`).
  - From then on a read is `traits.get(handle) -> Option<Level>`. A structural test refuses
    `Catalogue::handle` outside install code (X-S30-11).
- **Not `map<string, any>`.** The component's map is keyed by declared ids, and its values are a closed
  enum checked against a frozen declaration. An invalid state ("stature = blond") cannot be constructed
  past the decoder, and no reader receives an untyped blob.
- **`text` is presentation.** A System cannot obtain a `Handle<Text>`. Free text reaches cognition and
  clients only, marked as world text (INV-S27-12).

### 5.4 Visibility (SD-S30-4)

| Visibility | The holder's own observation | Another observer who perceives the holder | Systems (by handle) | Example |
| --- | --- | --- | --- | --- |
| `perceivable` | the value | `perceived_as` if declared, else the value | yes | stature ("tall"), hair colour, apparent age band, visible tattoos |
| `learnable` | the value | nothing. Learnable only through conversation, and then stored in the learner's memory (§9.5) | yes | chronotype, hometown, diet, languages |
| `private` | the value | nothing, ever | yes | S31's subscriptions, ancestry (A-10: never in a prompt), religion, income band |
| `hidden` | nothing | nothing | yes | a predisposition that a health System reads |

- Disclosure is by perception only, as for `look` (`ARC-91` item 4). It never depends on what the observer
  knows. "Learnable" is a statement about what conversation may reveal, not a gate on the observation.
- A world may **narrow** a built-in's visibility (`perceivable` → `private`), never widen it (§7.2).
- **`perceived_as` keeps observers honest.** A human sees "tall", not "1.87 m", and so does an agent. The
  holder sees their own exact value.

### 5.5 Derived attributes

A `derived` attribute declares `derive:`, chosen from a closed list of pure derivations in the model crate:

- `age { from: birth }` (years, floor);
- `band { from: <quantity or level>, bands: [...] }`;
- `projection` (owner code, §4.2 item 3).

A derived value is computed at disclosure and on a handle read, from stored state and the instant. It is
never stored and never stated as a fact. Replay is therefore exact.

### 5.6 Natural-language rendering

- `label` is the attribute's name, `phrase` a template with exactly one placeholder `{value}`, and `words`
  the per-value words (the default is the slug with hyphens turned into spaces).
- A `quantity` renders at its scale with its unit. A `set` renders as a list with "and". `perceived_as`
  bands render their `words`.
- The renderer lives in the model crate, and is mirrored in Python by golden tests (S30-f). It produces
  English. Localisation is S30-k (MVP-0, release-blocking): label packs keyed by attribute and value,
  zh-Hans first. A backend may ship its own (the MatrAIx pack carries its six, M-4).
- Rendering is deterministic and bounded. A person's rendered "who I am" is cut by group order and
  `prompt_rank` within a byte budget (§9.2).

---

## 6. The built-in catalogue

### 6.1 The curated built-in catalogue: ≤ 100 attributes, chosen by gameplay (Re-ruled 2026-10-15 (operator))

**Rule.** An attribute is built in only if the player can see it or can notice what it changes: how a person
looks, what they do and when, how they talk, what they buy, whom they spend time with, what they are good
at. Each row names the reader that ships by release. An attribute no shipped reader uses is cut. Worlds and
packs still add any attribute they like with zero code (§7), and the optional MatrAIx backend offers its
whole schema behind a mask (§6.4).

Reader codes:

| Code | Reader | Ships in |
| --- | --- | --- |
| C | S26 casting: briefs, prompts, distinctness | S26-src2 / S30-g |
| OV | other observers' view: "who I see", MCP `description`, both inspect panels | S30-f |
| ME | the holder's "who I am" in the LM controller | S30-f (with P6b) |
| CV | how the person speaks: register, length and tone in the LM controller's speech | S30-f |
| DM | an `attribute` consideration of the `motivated` decision model (what they do and when) | S30-f |
| S31 | S31's packs: `economy`'s wealth band, the notice System's default subscriptions | S31 |

**94 attributes in six groups:**

| # | Id (`core.`) | Group | Kind | Visibility | Reader and what the player notices |
| --- | --- | --- | --- | --- | --- |
| 1 | `body-form` | appearance | category | perceivable | C, OV: the body casting builds and an observer sees |
| 2 | `stature` | appearance | quantity m | perceivable | C, OV: height in both clients; "tall" |
| 3 | `mass` | appearance | quantity kg | private | C: drives `build` consistency at casting |
| 4 | `build` | appearance | category | perceivable | C, OV: slim to heavy-set |
| 5 | `skin-tone` | appearance | level (MST 1–10) | perceivable | C, OV |
| 6 | `hair-colour` | appearance | category | perceivable | C, OV; greys with age (evolving) |
| 7 | `hair-length` | appearance | level | perceivable | C, OV |
| 8 | `hair-style` | appearance | category | perceivable | C, OV |
| 9 | `hair-texture` | appearance | category | perceivable | C |
| 10 | `facial-hair` | appearance | category | perceivable | C, OV |
| 11 | `eye-colour` | appearance | category | perceivable | C, OV at conversation distance |
| 12 | `face-shape` | appearance | category | perceivable | C: distinct faces |
| 13 | `eye-shape` | appearance | category | perceivable | C: distinct faces |
| 14 | `nose` | appearance | category | perceivable | C: distinct faces |
| 15 | `lips` | appearance | category | perceivable | C: distinct faces |
| 16 | `glasses` | appearance | flag | perceivable | C, OV |
| 17 | `visible-marks` | appearance | set | perceivable | C, OV: tattoos, scars, birthmarks |
| 18 | `clothing-style` | appearance | category | perceivable | C, OV |
| 19 | `headwear` | appearance | category | perceivable | C, OV |
| 20 | `posture` | appearance | category | perceivable | C, OV: an older person's stoop |
| 21 | `voice` | appearance | category | perceivable | OV, CV: "a deep, slow voice" |
| 22 | `birth` | background | instant | private | ME, DM through age (derived below) |
| 23 | `age-years` | background | derived | private | ME, DM: what a person of that age does |
| 24 | `age-band` | background | derived | perceivable | C, OV: "in her sixties" |
| 25 | `pronouns` | background | category | perceivable | ME, OV, CV: how others refer to them |
| 26 | `ancestry` | background | category | private | C only, as the profile parent of skin tone and hair; never rendered to others |
| 27 | `native-language` | background | category | learnable | ME, CV |
| 28 | `languages-spoken` | background | set | learnable | ME, CV: who switches language with whom |
| 29 | `hometown` | background | text | learnable | ME: what they talk about |
| 30 | `years-in-town` | background | quantity year | learnable | ME, DM: newcomer or old-timer |
| 31 | `education-level` | background | level | learnable | CV: register |
| 32 | `occupation` | background | view of `employment` where installed | learnable | ME, OV-learnable |
| 33 | `marital-status` | background | category | learnable | ME |
| 34 | `household` | background | category | learnable | ME, DM: who waits at home |
| 35 | `children` | background | quantity count | learnable | ME, DM |
| 36 | `religion` | background | category | private | ME |
| 37 | `wealth-band` | background | view, owned by `economy` (Q-S30-6) | private | S31, ME, DM: what they can afford |
| 38–42 | `openness`, `conscientiousness`, `extraversion`, `agreeableness`, `neuroticism` | psychology | level | learnable | DM, ME: the five traits weigh every choice |
| 43 | `humour` | psychology | category | learnable | CV |
| 44 | `patience` | psychology | level | learnable | CV, DM: how long they wait or argue |
| 45 | `trust-propensity` | psychology | level | learnable | ME, DM: warmth to strangers |
| 46 | `curiosity` | psychology | level | learnable | DM: explores new places and people |
| 47 | `risk-tolerance` | psychology | level | learnable | DM, S31: buying and trying |
| 48 | `formality` | psychology | level | learnable | CV |
| 49 | `optimism` | psychology | level | learnable | ME, CV |
| 50 | `honesty` | psychology | level | private | ME: whether the LM character shades the truth |
| 51 | `core-value` | psychology | category | learnable | ME, DM: family, work, faith, fun, community … |
| 52 | `ambition` | psychology | level | learnable | DM: overtime, errands |
| 53 | `generosity` | psychology | level | learnable | DM: treats, tips, help |
| 54–66 | `skill.cooking`, `skill.baking`, `skill.coffee`, `skill.fishing`, `skill.woodwork`, `skill.boat-handling`, `skill.painting`, `skill.music`, `skill.teaching`, `skill.care`, `skill.selling`, `skill.gardening`, `fitness` | capability | level | learnable | ME, DM: the café's barista, the lake's fishermen, the painter, the nurse, the carpenter — what they choose to do and how they talk about it (each maps to a role tag or activity in the reference worlds) |
| 67 | `chronotype` | behaviour | level | learnable | DM: when they rise and go home |
| 68 | `sleep-need` | behaviour | level | learnable | DM |
| 69 | `punctuality` | behaviour | level | learnable | DM: on time or late to work |
| 70 | `talkativeness` | behaviour | level | learnable | DM, CV: who starts conversations, and how long they talk |
| 71 | `sociability` | behaviour | level | learnable | DM: joins group activities |
| 72 | `routine-adherence` | behaviour | level | learnable | DM: keeps the agenda or drifts |
| 73 | `walking-pace` | behaviour | level | perceivable | DM (stride), OV: "walks briskly" |
| 74 | `spending-style` | behaviour | level | learnable | DM, S31 |
| 75 | `tipping` | behaviour | level | learnable | DM |
| 76 | `gossip` | behaviour | level | learnable | DM, CV: passes on news |
| 77 | `greeting-style` | behaviour | category | perceivable | CV, OV |
| 78 | `helpfulness` | behaviour | level | learnable | DM |
| 79 | `news-interest` | behaviour | level | learnable | S31: default notice subscriptions |
| 80 | `weather-sensitivity` | behaviour | level | learnable | DM: stays in when it rains |
| 81 | `diet` | lifestyle | category | learnable | DM: what they buy and eat |
| 82 | `dietary-restrictions` | lifestyle | set | learnable | DM |
| 83 | `caffeine` | lifestyle | level | learnable | DM: café visits |
| 84 | `alcohol` | lifestyle | level | learnable | DM |
| 85 | `smoking` | lifestyle | level | perceivable | OV, ME |
| 86 | `food-favourites` | lifestyle | set over item kinds | learnable | DM: orders at the counter |
| 87 | `hobbies` | lifestyle | set | learnable | DM, ME: where they spend free time |
| 88 | `exercise-habit` | lifestyle | level | learnable | DM: the runner runs |
| 89 | `reading-habit` | lifestyle | level | learnable | DM, S31: the bench reader, the newspaper |
| 90 | `music-taste` | lifestyle | set | learnable | ME |
| 91 | `sport-followed` | lifestyle | set | learnable | ME |
| 92 | `pet` | lifestyle | category | learnable | ME |
| 93 | `faith-practice` | lifestyle | level | learnable | DM: observance days |
| 94 | `outdoor-preference` | lifestyle | level | learnable | DM: park or indoors |

That is 94 attributes, with headroom of 6 under the cap. A 95th or later built-in needs a named reader and
replaces nothing silently. `freckles` is deliberately not built in: it is AC-S30-3's custom example.

**Note 2026-10-15 (Q-S30a-1, Q-S30a-2; F-S30a-2).** Ruled 2026-10-15 (operator answered directly in the primary
session): `core.natal-sex` is added as a built-in (`category`: female, male; `private`, `fixed`; reader C, the
profile parent of stature, build and facial hair, as `ancestry` is). Rows 32 and 37 are views of other packs'
state and are declared by their owners in their own namespaces (§7.1): `employment.occupation` and
`economy.wealth-band` (step-31 §7.4). The table therefore lists 95 player-noticeable attributes; the built-in
catalogue `entities/person-attributes` declares **93 `core` rows**, under the cap of 100 (X-S30-20 counts `core`).

**What was cut from the built-in set, and why.**

| Category cut | Why | Still possible |
| --- | --- | --- |
| knowledge and expertise fields (academic domains, professional knowledge) | nothing a player sees reads them; job skills are the 13 capability rows | a world declares its own; MatrAIx backend |
| software, developer and tool usage | no MineWorld world is about software use | MatrAIx backend |
| industry exposure, career seniority | `employment` owns the job; the rest is unseen | MatrAIx backend |
| fine personality facets and several value frameworks | five traits plus one core value are what a decision model and a prompt can visibly express; 30 facets are not distinguishable in play | custom; MatrAIx backend |
| political and social attitudes | no System reads them; high risk of caricature | custom |
| media genres, country familiarity, long lists of topics | conversation trivia, not behaviour; `hobbies` and `music-taste` sets cover what is noticed | custom |
| dated life-event flags | a biography states these in prose and in facts | `persona`, custom |
| accessibility and UX needs (text size, screen readers) | software-testing personas, not world people | MatrAIx backend |
| transient emotional state | mood is a System's state, not a trait | a mood System |
| type-indicator personality (MBTI) | redundant with the five traits | MatrAIx backend |
| per-language, per-cuisine, per-sport and per-hobby dimensions | collapsed into sets (`languages-spoken`, `food-favourites`, `hobbies`, `sport-followed`) | custom |
| sexual orientation, disability, citizenship, veteran status, health conditions | sensitive, and no shipped reader. A world that models them adds them, private by default (Q-S30-8) | custom; MatrAIx backend (private by its overlay) |

### 6.2 Defaults and realism

- Every built-in has a `default` only where a realistic world default exists. Otherwise it is absent until
  a profile or a file states it.
- Numeric ranges follow real-world values, with citations in the catalogue file:
  - stature 0.50–2.50 m, as the domain only; the realistic distribution is the profile's;
  - mass 2–300 kg.
- S26's G-4 bound is S26's own, read from the profile's declared support (C-7).

### 6.3 Versioning (SD-S30-5)

- Every backend's catalogue (the built-in `entities/person-attributes`, `entities/attributes-matraix`) is a data pack with semantic versions (`PACKAGE_FORMAT.md`).
  A world requires it (`requires: { person-attributes: "^1" }`).
- **Minor:** add an attribute; add a value at the end of a `category` or `set` list; add words, phrases or
  facets.
- **Major:** remove or rename an attribute; remove or reorder values; change a kind, a unit or a scale.
- **An id is never reused** for another meaning.
- Saves store slugs, never indices (§5.3), so adding values does not move existing facts.
- A save records the catalogue pack's version in its composition, as for every pack. Resuming under
  another major version is refused by name, as an incompatible pack is today.

### 6.4 Person-generation backends (`ARC-126`, Re-ruled 2026-10-15 (operator, final))

**The operator, verbatim (final, 2026-10-15):** "我觉得我们应该是支持MatrAIx为后端生成人物，并给出demo，但是不要把它们作为强依赖，而是一个可选项" —
support MatrAIx as a backend for generating people, ship a demo, and never make it a hard dependency: it is an
optional choice. The earlier rulings of the same day are kept in §16.2.

**A backend** is a data pack that supplies a world's attribute catalogue and the means to generate people
over it. A world chooses one in `configure/attributes.yaml`:

```yaml
format: 1
backend: mineworld                 # default; or the id of any installed backend pack, e.g. matraix
mask: core                         # optional; a mask the backend ships, or a world's own (masks/<id>.yaml)
profile: san-diego-county
seed: mineworld-s26-cast-v1
lock: attributes.lock.yaml
```

The contract (`backend.yaml` at the pack's root; every field validated, every failure named):

| Field | Meaning | Failure, by name |
| --- | --- | --- |
| `id`, `version`, `license`, `attribution` | identity and credit, shown by `mineworld attributes backends` and in `resolve`'s provenance | `BackendMalformed { pack, field }` |
| `catalogues` | the attribute files it supplies (format of §5.1) | `UnknownAttribute`, `DuplicateAttribute` as for any catalogue |
| `masks` | the masks it ships (below) and its default mask | `UnknownMask { backend, mask }`; `MaskNamesUnknownAttribute { mask, id }` |
| `sampling` | the modes it supports, from the engine's closed list: `profile` (our sampler over a cited world profile, §8) and, optionally, `log-linear` (a backend-supplied factor model, executed by our engine in fixed point, Q-S30-9) | `UnsupportedSampling { backend, mode }` |
| `provenance` | upstream source, version, commit, file digests | `ProvenanceMismatch { file, expected, found }` (CI re-verifies) |
| `mapping` | how the built-in ids that Systems and consumers read are provided under this backend (§6.6) | `MissingUnderBackend { attribute, backend }` at the reader's install |

- **No backend runs code.** A backend is data executed by the model crate's engine, so determinism, the
  three platforms and kernel ignorance hold for every backend. A new sampling mode is a model-crate change,
  reviewed as such.
- **The MineWorld backend is the default** and is what the core, every reference world and every test use.
  It is `entities/person-attributes` (§6.1: ≤ 100 curated attributes, the appearance group and the quantity
  kind) with the `profile` mode and masks `core` (all 93) and `appearance` (rows 1–21, for casting-only use).

  *Note 2026-10-15 (S30-b C0, for S30-a F-S30a-2 and Q-S30a-1).* This line read "masks `core` (all 94)". The
  catalogue declares 93 `core` rows: rows 32 and 37 of §6.1 are owner views, and `core.natal-sex` is added
  (`ARC-120`, `ARC-122`; `MODULE_SPEC.md` §2).
- **The appearance group and quantities are always MineWorld's.** Any backend's catalogue is composed with
  `core.` appearance and quantity attributes, because a body in a client must come from one place (`ARC-125`).
  A backend may not redeclare them.

### 6.5 Masks (`ARC-126`)

A **mask** is a named, versioned selection over a backend's catalogue, as data (`masks/<id>.yaml`):

```yaml
format: 1
mask: matraix-core-100
version: 1
include:
  - { id: lstyle_sleep_schedule, why: "DM: when they rise and go home (maps to core.chronotype, §6.6)" }
  - { group: "Personality: Big Five", ids: [big5_friendliness, big5_gregariousness], why: "DM, CV" }
exclude: []
```

- `include` and `exclude` take ids, groups or facets. Each entry carries a `why` line naming its reader;
  a mask without them is refused (`MaskEntryUnjustified`). The special mask `all` includes everything.
- **What the mask does not enable does not exist in the world.** It is absent from the frozen `Catalogue`
  (§5.3): the sampler skips it, no observation, prompt, MCP view or panel can carry it, a person file stating
  it is refused by name (`MaskedOut { attribute, mask }`), and a System handle for it fails at install.
- Owner packs' own attributes (S31's subscriptions, `persona`'s fields) are not masked: they belong to
  installed Systems, which a world already chose.
- A world authors its own mask with zero code. A missing or malformed mask fails by name.

### 6.6 The MatrAIx backend: optional, installable, and the only place that names MatrAIx

**What it is.** `entities/attributes-matraix`, a separate data pack, **not installed by default**, credited
to Li, Hao et al., *MatrAIx: Simulating the World with 8.3 Billion Persona Agents*, arXiv 2608.04205v1 (2026),
and to the `MatrAIx-ai/MatrAIx-Persona-8B` repository (MIT).

- **As-is import.** `persona/schema/dimensions.json` (schema 1.0) is imported verbatim, pinned to upstream
  commit `3633d8dab149a9482a71b024418a49ae828cc941` (repository `main`, 2026-09-09; the file last changed in
  `284b106c…`, 2026-08-06; SHA-256 `cbf4db9d887dec057431aff409e1d28d83f345e7a31e29a4275a607212ecfca5`). Their
  ids, categories (as groups), value lists, phrases (`{value}` is already our placeholder) and the six
  locale label packs (including zh-Hans) are kept exactly; nothing is renamed or slugified. Our id rule
  accepts `_` and our category values accept their strings verbatim, inside this pack.
- **A MineWorld overlay inside the pack** adds what their schema lacks, keyed by their ids and never editing
  their file: visibility (default `learnable`; `private` for ethnicity, religion, sexual orientation,
  disability, citizenship, veteran status, income and every `health_*` dimension, Q-S30-8), mutability
  (default `fixed`; `derived` for `age_bracket` from `core.birth`), owner (`economy` for
  `socioeconomic_band`, ruled Q-S30-6), facets, and `level` order where a reader compares values.
- **Exclusions, on licence grounds only.** Redundancy is the mask's job, never a reason to drop a dimension.
  - The 50 dimensions whose `source_origin.license` is not covered by the repository's MIT grant:
    - `citation-only` (28): `schwartz_value_self_direction`, `schwartz_value_stimulation`,
      `schwartz_value_hedonism`, `schwartz_value_achievement`, `schwartz_value_power`,
      `schwartz_value_security`, `schwartz_value_conformity`, `schwartz_value_tradition`,
      `schwartz_value_benevolence`, `schwartz_value_universalism`, `sdt_need_autonomy`,
      `sdt_need_competence`, `sdt_need_relatedness`, `mft_care_harm`, `mft_fairness_cheating`,
      `mft_loyalty_betrayal`, `mft_authority_subversion`, `mft_sanctity_degradation`,
      `mft_liberty_oppression`, `need_for_cognition`, `need_for_closure`, `dospert_ethical_risk_tolerance`,
      `dospert_financial_risk_tolerance`, `dospert_health_safety_risk_tolerance`,
      `dospert_recreational_risk_tolerance`, `dospert_social_risk_tolerance`,
      `interpersonal_agency_dominance`, `interpersonal_communion_warmth`;
    - `research-use-reference` (22): `bfi2_domain_extraversion`, `bfi2_domain_agreeableness`,
      `bfi2_domain_conscientiousness`, `bfi2_domain_negative_emotionality`, `bfi2_domain_open_mindedness`,
      `bfi2_facet_sociability`, `bfi2_facet_assertiveness`, `bfi2_facet_energy_level`,
      `bfi2_facet_compassion`, `bfi2_facet_respectfulness`, `bfi2_facet_trust`, `bfi2_facet_organization`,
      `bfi2_facet_productiveness`, `bfi2_facet_responsibility`, `bfi2_facet_anxiety`,
      `bfi2_facet_depression`, `bfi2_facet_emotional_volatility`, `bfi2_facet_intellectual_curiosity`,
      `bfi2_facet_aesthetic_sensitivity`, `bfi2_facet_creative_imagination`, `attachment_anxiety`,
      `attachment_avoidance`.

    Why: each carries its own upstream licence statement ("theory construct label only", research use),
    which is narrower than MIT, so it is excluded until its owner's terms are confirmed. Re-admitting one
    is a reviewed pack change citing the confirmation. The import tool refuses any dimension whose
    `source_origin.license` is set to something other than MIT or absent.
  - The Persona-1M coreset (Hugging Face `MatrAIx2026/MatrAIx_Persona_1M`): licence
    `matraix-research-only`, and derived from real people's records. It is never imported.
  - That leaves **1,240 dimensions**.
- **Masks it ships.** `matraix-core-100` (the default, ≤ 100 dimensions chosen by gameplay impact, each with
  its `why`, written in S30-h's PR design from §6.1's reader codes; for example `lstyle_sleep_schedule`,
  `lstyle_diet_type`, `lstyle_caffeine`, `lstyle_coffee_ritual`, `big5_friendliness`, `cog_verbosity`,
  `cog_formality`, `skill_cooking`, `skill_baking`, `skill_selling`, `topic_fishing`, `demo_marital_status`,
  `lstyle_news_freq`), and `all` (all 1,240).
- **The mapping** (`mappings/core.yaml`, owned by this pack). For every built-in attribute a reader
  declares and the MatrAIx schema overlaps, a `derived` view from the MatrAIx value, value by value. Examples:
  `core.chronotype ← lstyle_sleep_schedule` (`Early bird → early-bird`, `Night owl → night-owl`,
  `Irregular`, `Flexible`, `Shift-based → neutral`); `core.diet ← lstyle_diet_type`;
  `core.caffeine ← lstyle_caffeine`; `economy.wealth-band ← socioeconomic_band`;
  `core.extraversion ← big5_gregariousness`. Under this backend a reader's `Handle<core.X>` resolves to the
  mapped view. A built-in that is neither mapped nor appearance is absent, and a reader that requires it
  fails at install with `MissingUnderBackend`; a reader may declare the read optional. The mapping is
  checked: every target value exists, every source value is covered.

  *Note 2026-10-15 (S30-a, F-S30a-2; reconciled by S30-a at the primary's dispatch).* The mapping example read
  `core.wealth-band ← socioeconomic_band`. The wealth band is `economy`'s attribute in its own namespace
  (§7.1; step-31 §7.4), so the target is `economy.wealth-band`, as `docs/DECISIONS.md` `DEP-66` states it.
- **Updating upstream.** The pack carries its own import tool (`tools/import.py`, Python standard library,
  offline). It reads a given `dimensions.json` and label packs at a stated commit, verifies the SHA-256,
  re-applies the licence exclusions, and regenerates the pack. CI re-runs it over the pinned inputs and
  requires byte equality. An upgrade is a reviewed PR with a generated diff report (added, removed and
  changed dimensions). A removed dimension that a mask, mapping or profile names fails by name. A removal
  is a major version.
- **Attribution.** `pack.yaml` (`license: MIT`, `attribution:`), `LICENSE` (the upstream MIT text,
  verbatim), `NOTICE` (commit, digest, paper). Line for the README and docs:
  *"Optional person-generation backend built on the MatrAIx persona schema (Li, Hao et al., 2026,
  arXiv 2608.04205; MIT), imported unchanged."*
  Proposed credit: that line in the repository README's backends row and in the guide chapter of the demo
  (§6.7), a link to their repository, and a `CITATION` entry. Contacting the MatrAIx authors for joint
  promotion is the operator's; the pack is ready to be shown to them as it stands.
- **Isolation (checked).** Nothing outside `entities/attributes-matraix/` names MatrAIx: a CI grep over the
  tree, excluding that directory, `worlds/matraix-demo/`, its guide page, the README's attribution line, the
  plans and `docs/DECISIONS.md`, refuses `matraix` in any case
  (X-S30-18). A second CI job builds and runs the full test suite with the pack directory removed
  (X-S30-19).
- **Sampling.** In `profile` mode our engine samples the masked set from a world profile, as for the
  built-in backend; MatrAIx's global priors are not a place's, so the demo's profile cites its sources like
  any other. Their sampler code is not used (float Python). Whether to also offer their `full_dag.json`
  as a faithful `log-linear` mode is Q-S30-9.

### 6.7 The MatrAIx demo (release-blocking)

- `worlds/matraix-demo`: a configuration of `social-cafe`'s places with its own people files and
  `configure/attributes.yaml` naming `backend: matraix`, mask `matraix-core-100`, and a cited demo profile.
- It shows MatrAIx attributes in casting briefs (through `resolve --json`), in "who I am" and "who I see",
  in the MCP `description` and `me.attributes`, and in both inspect panels, including a zh-Hans panel from
  their label pack.
- A guide section (`docs/guide/`, CI-executed on three platforms like every guide step) walks through
  installing the pack, switching the backend and mask, and back; it carries the attribution line.
- The demo world lives outside the MatrAIx pack and names it only through `backend: matraix`. It, its guide
  page and the README's attribution line are the isolation grep's named exceptions.

---

## 7. User-defined attributes

### 7.1 Namespaces (SD-S30-6)

- Every attribute id is qualified as `<namespace>.<slug>`. The namespace is the declaring pack's id:
  - `core` is reserved for `entities/person-attributes`;
  - a World Pack's own declarations use the world's id;
  - an owner System Pack's declarations use its pack id (for example `notices.subscriptions`).
- **Authors may write the short slug** in people files and profiles when it is unique among the world's
  catalogues. An ambiguous short slug is refused by name and the message lists the qualified candidates.
  The qualified form is always accepted.
- A declaration in a namespace other than the declaring pack's is refused (`ForeignNamespace`).

### 7.2 Overrides

A world may change a declaration it did not author only through an explicit entry:

```yaml
# worlds/harbour/attributes/overrides.yaml
format: 1
overrides:
  core.ancestry:
    vocabulary: harbour.peoples        # replace the value vocabulary? refused: kind and values are fixed
  core.hair-colour:
    words: { auburn: copper }          # allowed: presentation
    visibility: private                # allowed: narrowing only (perceivable → learnable → private → hidden)
```

| May override | May not override (refused by name) |
| --- | --- |
| `label`, `phrase`, `words`, `perceived_as` words, `facets` (add only), `default`, visibility (narrower only) | `kind`, values and their order, `unit`, `scale`, `owner`, `mutability`, widening visibility |

- Silent redeclaration of an existing id is refused (`DuplicateAttribute`).
- A world that wants a different value set declares its own attribute (for example `harbour.ancestry`),
  and its profile uses that one instead.

### 7.3 `freckles`, end to end (AC-S30-3)

```yaml
# worlds/<w>/attributes/local.yaml
format: 1
attributes:
  freckles:
    group: appearance
    facets: [appearance]
    kind: { level: [none, light, pronounced] }
    visibility: perceivable
    mutability: fixed
    label: Freckles
    phrase: "{value} freckles"
    words: { none: no }
```

```yaml
# populations/<profile>.yaml (fragment)
freckles: { prior: { none: 0.70, light: 0.22, pronounced: 0.08 }, source: authored }
```

With no code change, `freckles` then appears in:

- `mineworld attributes catalogue`;
- each person's resolved values, with its origin;
- S26's briefs, prompts and casting page, through the `appearance` facet (X-S26src-10);
- other observers' views ("light freckles"), because it is perceivable;
- each holder's "who I am";
- the MCP `description` line;
- both inspect panels.

### 7.4 What still needs code

- A **new view on another pack's state**, such as a band of `economy`'s balance, needs the owner's
  projection (§4.2 item 3).
- A **new unit** (§5.2).
- A **System that acts on an attribute** (§9.4).

Everything else is data.

---

## 8. Population profiles and the sampler (`ARC-123`)

### 8.1 What a profile is

A profile is a data file in a pack's `populations/` directory, versioned with that pack. A world names one:

```yaml
# worlds/<w>/configure/attributes.yaml (attributes' world configuration, ARC-61; seam item 2)
format: 1
backend: mineworld                     # default; §6.4. `mask:` optional (§6.5)
profile: san-diego-county              # resolved through the world's required packs; absent = no sampling
seed: mineworld-s26-cast-v1            # the population seed: world content, not the run's --seed
salt: 0                                # optional; S26's distinctness re-draw, once ruled (§8.6)
lock: attributes.lock.yaml             # optional (§8.4)
```

- **The population seed is not the run seed.** The same world gives the same people under every
  `mineworld run --seed`. Casting and the world therefore always agree.
- Several worlds may share a profile. The café town, market town and lakeside share San Diego County today
  (F-CAST-2).

### 8.2 The format

```yaml
# entities/populations-us/populations/san-diego-county.yaml (excerpt)
format: 1
profile: san-diego-county
version: 3
covers: adults                         # who the profile describes; people outside it are refused by name
sources:
  acs-2024-1y:
    cite: "American Community Survey 2024 1-year, B01001, B03002, San Diego County (05000US06073)"
    url: https://data.census.gov/table/ACSDT1Y2024.B03002?g=050XX00US06073
    read: 2026-10-15
    status: read                       # read | verify
  nhanes-2015-2018:
    cite: "Fryar et al. 2021, Vital Health Stat 3(46), Tables 9, 11, 14, 15"
    status: read
  mst-convention:
    cite: "MineWorld casting convention (operator-gated, Q-S26-7, ARC-95)"
    status: authored                   # not data: an authored convention, shown as such
distributions:
  core.natal-sex:
    prior: { female: 0.4986, male: 0.5014 }
    source: acs-2024-1y
    stratify: true
  core.age-band:                        # the profile samples the band; birth follows (below)
    prior: { 20s: 0.1864, 30s: 0.2050, 40s: 0.1716, 50s: 0.1506, 60s: 0.1417, 70s: 0.0973, 80-plus: 0.0474 }
    source: acs-2024-1y
    stratify: true
  core.birth:
    given: [core.age-band]
    uniform_age_within_band: true      # birth = epoch - uniform age in the band (integer days)
  core.stature:
    given: [core.natal-sex, core.age-band]
    normal: { by: { female.20s: { mean: 1.626, p5: 1.510, p95: 1.728 }, … } }
    clip_sd: 2
    source: nhanes-2015-2018
  core.build:
    given: [core.natal-sex]
    prior: { female: { slim: 0.144, average: 0.167, solid: 0.262, heavy: 0.427 }, male: { … } }
    source: nhanes-2015-2018
    verify: "county figure not read (F-CAST-4); national NHANES over-represents `heavy` here"
  core.hair-colour:
    given: [core.ancestry, core.age-band]
    prior: { … }                       # authored pool per ancestry
    adjust:                            # likelihood ratios r(v; parent), MatrAIx's r_i (M-2)
      - { when: { core.age-band: [60s] }, ratios: { grey: 4.0, silver-grey: 2.0 } }
    source: mst-convention
  core.facial-hair:
    given: [core.natal-sex]
    prior: { none: 0.55, stubble: 0.15, short-beard: 0.20, moustache: 0.10 }
    mask:                              # hard compatibility, MatrAIx's m_i (M-2)
      - { when: { core.natal-sex: [female] }, only: [none] }
    source: authored
roles:                                 # defaults by authored role: tags or an attribute (§9.6)
  - { when: { tags: [cook] }, add: { notices.subscriptions: [prices.ingredients] } }
```

Rules (each refused by name at load):

| Rule | Error |
| --- | --- |
| every key names a declared attribute that the world's catalogue knows | `UnknownAttribute { profile, key, nearest }` |
| `given` lists form a **strict DAG**. Cycles are refused and the cycle is printed | `DependencyCycle { profile, cycle: [a, b, …, a] }` |
| a `prior` table covers every combination of parent values that can occur, or states `otherwise:` | `MissingCondition { attribute, parents }` |
| probabilities are decimal strings or numbers ≥ 0 with at most 6 decimals. Each row sums to 1 ± 1e-6 and is then normalised to integer parts per million | `BadDistribution { attribute, row, sum }` |
| `adjust` ratios are > 0; `mask` lists values of the attribute's domain | `BadAdjustment` / `BadMask` |
| a mask leaves at least one value for every parent combination that can occur | `EmptySupport { attribute, parents }` |
| every distribution names a `source` that the file declares | `UncitedDistribution` |
| `quantity` distributions declare the support they can return (C-7) | `MissingSupport` |

- **`verify` marks** are carried through to `mineworld attributes check` and to S26's casting page. They do
  not fail the load. A release check may refuse them for the reference worlds (Q-S30-5).
- **Data and conventions stay apart** (F-S30-5). A source of status `authored` is shown as authored on
  every report and is never presented as data.
- Provider limits are not profile content. "No loose long hair for route D+" is S26's `unsupported`
  declaration on the provider (A-11), never a profile restriction.

### 8.3 The semantics

For each attribute X_i with parents Pa(i), and a person whose parents are already resolved:

```text
weight(v) = prior_i(v | pa) × Π adjust ratios matching pa × mask_i(v; pa)      (MatrAIx M-2)
```

- Weights are normalised to integer parts per million, after rounding by largest remainder so that each
  row sums to exactly 1,000,000.
- **Rare valid combinations stay eligible.** Only a mask zeroes a value.
- **Numeric distributions** (`normal` by stratum, `uniform` and `table` quantiles) use a committed table of
  standard-normal quantiles in micro-units. Interpolation is integer. No `ln`, `exp` or platform `libm` is
  used, so the result is identical on Linux, macOS and Windows.

### 8.4 The lock file (SD-S30-7)

`attributes.lock.yaml` is machine-written and human-reviewed. It pins values per person, with an origin:

```yaml
format: 1
profile: { id: san-diego-county, version: 3 }
seed: mineworld-s26-cast-v1
people:
  alice:
    core.stature:   { value: 1.66, origin: ruled, at: 2026-10-15, by: "casting gate B0 (Q-S26-7)" }
    core.skin-tone: { value: 4,    origin: ruled, at: 2026-10-15 }
```

- **Precedence:** the person's own `attributes:` section, then the lock, then roles, then the profile
  sample, then the declaration's `default`.
- **The lock is how a casting gate's ruling binds the world.** When the operator rules a person at
  casting, the casting tool writes the ruled appearance values into the lock (S26 §7.4b.4, "a ruled brief
  is authored data"). The world's attributes and the look's brief can then never diverge, even when a
  later profile version or seed changes every unruled draw.
- `mineworld attributes lock <world> --person alice --attribute core.stature` writes an entry. The tool
  writes the file; people never hand-edit indices.

### 8.5 The algorithm (largest remainder, biography first)

Given the people the profile covers and the attributes ordered topologically (ties broken by qualified
slug):

1. **Fixed values first.** For each person, the values stated in their file or pinned in the lock are
   fixed and never drawn (S26 §7.4a item 5).
2. For each attribute X in topological order:
   - Group the people by their resolved parent values.
   - Within each group g of size n_g:
     - The target count for each value v is n_g × weight(v | g).
     - Subtract the number of fixed members of the group who already hold v, with a floor at 0. Then
       allocate the remaining unfixed members by **largest remainder**. Ties in the remainder are broken
       by `mix(seed, attribute, group, value)`.
   - Assign the allocated multiset to the unfixed members in an order given by
     `mix(seed, attribute, person key)`. A member's `allowed:` restriction from their file (for example
     `stature: { between: [1.80, 1.95] }`) is respected by a bounded matching. When no matching exists,
     the run is refused by name (`Unsatisfiable { person, attribute }`), never silently re-drawn.
   - With `stratify: false`, each unfixed member draws independently with `mix(seed, attribute, person
     key)`, which is useful for very small worlds or weakly constrained attributes.
3. **Numeric attributes** draw a quantile per member: stratified `(i + ½) / n_g` points permuted by `mix`
   when `stratify`, else one draw. The quantile maps through the stratum's distribution and is clipped
   (`clip_sd`).
4. **Roles** (`roles:`) apply after sampling. They add to a `set`, or fix a value, only for members whose
   tags or attributes match. Each application is recorded with origin `role`.
5. **Nothing reads a name.** The sampler's input type holds the person's key (an opaque hash input), their
   tags, their authored attributes and their lock entries. It holds no display name, no `note` and no
   `persona`. Renaming every person changes no resolved value (X-S30-6).

The outcome for each person and attribute is a value and an **origin**: `authored`, `ruled`, `locked`,
`role`, `sampled { stratum, draw }` or `default`. The origin is authoring provenance, so it is reported by
the tool and kept out of world state (`CORE_CONCEPTS.md` §3 "Metadata"), and the save stores only the
value (§11).

**Guarantee.** For a population of N covered people, every stratum's realised count differs from its
target by less than one person within each parent group. For N = 1,000 that is within 0.1 pp of the
marginal for unconditioned attributes. Conditionals are within 2 pp for any group of at least 50 people
(AC-S30-5).

### 8.6 One implementation, two entry points (`DEP-67`)

- **At genesis**, `attributes`'s `resolve` (seam item 4) runs the sampler and hands each owner its typed
  values for seeding.
- **`mineworld attributes resolve <world> [--json] [--person <key>] [--facet appearance] [--salt <n>]`**
  prints the same resolution with origins, the profile identity and the realised distribution against the
  target. This is the resolved cast S26 consumes (§10).
  - `--salt` exists for S26's distinctness re-draw (A-9's "attempt"). It changes `sampled` values only.
  - A world resolved with a non-zero salt is not the world's own population until the salt is written
    into `configure/attributes.yaml`.
- `mineworld attributes check <world>` validates everything without resolving, and lists `verify` marks.
- `mineworld attributes catalogue <world> [--json] [--facet …]` lists the declarations in effect.

| Candidate | Why considered | Verdict |
| --- | --- | --- |
| `pgmpy` (MIT, Python) | mature Bayesian-network sampling | declined: Python, and genesis is Rust. It uses float sampling, and it is a dependency larger than the problem (≈ 400 lines of sampler) |
| `pomegranate` (MIT, Python) | the same | declined, for the same reasons |
| PopulationSim (ActivitySim, BSD-3) | synthetic populations fitted to census marginals (IPF) | declined for MVP-0: it solves a different problem (household synthesis against control totals at zone level), and it is Python with pandas. Recorded as the reference if MVP-1's region worlds need joint fitting (S21) |
| MatrAIx sampler | the formula we use | not used: float Python; their DAG as data is Q-S30-9 (§6.6) |
| our own, in the model crate | integer, deterministic, shared by genesis and the CLI | **chosen** |

The interim Python sampler (A-9) is removed when S30-g switches casting to `resolve --json`. No adapter is
kept (`CLAUDE.md` §4 rule 12). S26's draft already plans exactly this (A-11, S26-src2).

---

## 9. Use in the world

### 9.1 Disclosure

- **The Rust path.** `attributes` discloses one `traits` component per perceived person, filtered for the
  observer:
  - the holder gets all non-hidden values;
  - another observer gets the `perceivable` ones, through `perceived_as`.
- **The payload is typed.** Each entry is `{ attribute: qualified slug, value: <typed JSON by kind>, text:
  rendered phrase }`, schema 1, in catalogue order.
  - `text` is world text (INV-S27-12). It is included so that every consumer shows the same words without
    a second fetch.
  - An owner pack discloses its own attributes within its own component, by the same rule. The model
    crate provides the filter and the renderer, so no owner writes its own visibility logic.
- **Size.** About 25 perceivable attributes per person at about 40 B each is about 1 KiB per perceived
  person. A holder's own full built-in set (94) is about 4 KiB in its own observation only; under the MatrAIx
  backend with mask `all` (1,240) it would be about 50 KiB, so the per-seat cap below applies there. S30-c measures this
  against the observation's existing frame sizes, and a per-seat cap with a named truncation marker
  follows if it is needed (R-S30-3).

### 9.2 Cognition and decision models

- **Who I am.** The LM controller's persona block (P6b, `pr-s10-p6-lm-controller.md` §5.2.1) gains an
  `Attributes` part: the holder's own rendered phrases.
  - Phrases are chosen by facet `prompt` and by a per-world `prompt_rank`, and cut at a byte budget
    (default 600 B, inside step-17's 6,000-byte context).
  - Each controller's `decisions.yaml` entry may name extra facets or groups.
- **Who I see.** Each perceived person's line gains their perceivable phrases ("a tall man in his sixties
  with grey hair and a flat cap"), at most 160 B per person.
- **Decision models (Rust).** These are pure functions of the observation (A-8). The SDK gains a typed
  reader over the disclosed `traits` component: `observation.attributes(person).level("core.chronotype")`,
  checked against the disclosed kind. The `motivated` model (DL-b) may use it as a consideration input,
  declared in its parameters, never as a hard-coded slug. S30-f adds that `attribute` consideration to
  `motivated`, reviewed as a change to that pack: it is the DM reader of §6.1.
- **No LM is needed.** Everything above is deterministic text and typed values. An LM is one consumer of
  it.

### 9.3 Agent players (MCP, S27) and both clients

- **The MCP adapter.**
  - `PersonLine` gains `description: WorldText | null`, the joined perceivable phrases, ≤ 160 B.
  - `me` gains `attributes: [AttributeLine]` (slug, label, text, typed value), the holder's own non-hidden
    attributes, paged.
  - `inspect` already returns disclosed components (A-12), so `traits` appears there with no change.
  - S27-d's tool golden (`clients/agent/tests/golden/tools.json`, A-12) changes with the new fields. That is
    an additive, reviewed re-record in S30-f, never a silent one.
  - INV-S27-2 holds: everything comes from the seat's own observation.
- **The 2D client.** The info panel (the client's existing inspect surface) lists the disclosed phrases
  for the selected person, and the holder's own sheet on a key. These are presentation only (INV-5).
- **The 3D client.** On interact-target, the same panel shows the same phrases. Parity with 2D (the
  client-parity rule) is checked by one shared scenario that compares the two panels' text.
- **A human and an agent see the same thing.** A human sees a face and reads "tall, grey hair". An agent
  reads "tall, grey hair". That is AC-16's richness within the same perception limits.

### 9.4 Systems that read attributes

- **The rule.** A System that reads an attribute declares it (`reads_attributes:`) and receives a typed
  handle at install (§5.3). It reads `traits` through the model crate's reader. It never reads strings
  from a component payload.
- **No existing System is changed by S30.** Schedule offsets by chronotype, relationship affinity by
  shared hobbies, or a café menu by diet are each the owning System's own later PR, made when that System
  is next opened. Doing them now would edit unrelated Systems, which is change amplification
  (`CLAUDE.md` §4 rule 5).
  - **The one named exception (Q-S30a-2, ruled 2026-10-15, operator answered directly in the primary
    session):** S30-f makes `employment` the owner of `employment.occupation` — a projection of its own
    `Employment { job }` to the job's label, and the declaration shipped with the pack (`ships_data!`,
    Q-S30b-1). Nothing else in `employment` changes. (S31 changes `economy` for `economy.wealth-band` in its own
    PRs.)
- **S30's proof of the read path** uses two consumers:
  - a test-only fixture System in `systems/attributes/tests/` that refuses an action unless the actor is
    an `early-bird` (X-S30-10);
  - S31's notice pack as the first real reader (§9.6).

### 9.5 Memory (S28): attributes learned about others

- **An observed attribute is perception.** It is ingested as part of the perceived person's disclosure.
  It is not stored per observation: S28's renderer drops unchanged component snapshots, and S30-f checks
  that `traits` is treated the same way.
- **A learned attribute is a memory** (INV-4). When Bob tells Alice "I'm a night owl", Alice's memory
  holds the utterance. With S30-i (MVP-0, release-blocking), it may also hold a structured `stable` fact in S28's private
  layer, `(subject: Bob, attribute: core.chronotype, value: night-owl, learned_from: EventId)`. That fact
  is subjective, may be false, may be forgotten, and **never** widens Alice's observation of Bob.
- **S30-i's optional action `tell-about-self { attribute }`** is owned by `attributes`. It states a
  conversation-audience fact of the holder's true value, for a `learnable` attribute only. It makes
  "learnable in conversation" structured and truthful without letting an LM's free speech become world
  state. It was MVP-1 under the original ruling; the re-ruling of 2026-10-15 makes it release-blocking.
- **The world records no knowledge state.** Who knows what about whom is cognition's.

### 9.6 The worked example: public-notice subscriptions (S31 interface, `ARC-124`)

This is the attribute a System, not cognition, reads (§1.2 item 3). S31 owns the System. S30 owns the
schema, the defaults and the reading path. The interface S31 consumes:

| # | S31 does | S30 provides |
| --- | --- | --- |
| N-1 | declares the topic vocabulary as data in its pack, `vocabularies/topics.yaml` (`prices.ingredients`, `prices.fuel`, `station.departures`, `news.daily`, `weather.warnings` …) | the `vocabulary:` value domain for `set` (§5.2), with unknown topics refused by name at load |
| N-2 | declares `notices.subscriptions` in its pack's `attributes/`: group `behaviour`, `kind: { set: { vocabulary: notices.topics, max: 32 } }`, `visibility: private`, `mutability: evolving`, `owner: notices` (the id S31 chooses), phrase `"follows {value}"` | the declaration format, owner check (§4.2 item 5) and the renderer |
| N-3 | implements `AttributeOwner` for that attribute: it receives the resolved initial set per person at genesis as `SetValue<TopicId>` and states its own `subscribed` facts | the resolution: the person's file (`attributes: { subscriptions: [news.daily] }`), then the lock, then **roles** (`{ when: { tags: [cook] }, add: { notices.subscriptions: [prices.ingredients] } }`), then the profile, then the default |
| N-4 | changes subscriptions with its own `subscribe` / `unsubscribe` actions and facts, and reads its own component to route notices | nothing: `attributes` is not involved (§4.5) |
| N-5 | discloses subscriptions to the holder only, through the model crate's filter | `private` visibility, which no other seat ever receives (X-S30-4) |
| N-6 | optional: a role default keyed on occupation once `employment` is the owner of `employment.occupation` | `roles.when` accepts any resolved attribute, not only tags |

*Note 2026-10-15 (S30-a, F-S30a-2, Q-S30a-2; reconciled by S30-a at the primary's dispatch).* N-6 read "the
owner of `core.occupation`". Occupation is `employment`'s attribute in its own namespace, `employment.occupation`,
which S30-f adds (§9.4's named exception); the wealth band likewise is `economy.wealth-band` (§§6.1 note, 6.6).
`docs/MODULE_SPEC.md` §3.1 and `docs/DECISIONS.md` `ARC-120`, `ARC-124` use these names.

S31 depends on `mineworld-attributes-model`, not on the `attributes` System Pack, so a world may run
notices without the plain-trait pack. If S31 freezes before S30-c, it ships its declaration format against
§5.1 and adopts the crate when S30-c merges. That ordering is recorded in §13.3.

---

## 10. The S26 interface (character sources and casting)

S26's draft amendment states its needs as C-1…C-7 (A-11). S30's `mineworld attributes resolve <world>
--json --facet appearance` satisfies each one:

| S26 needs | S30 output |
| --- | --- |
| C-1 declarations of the appearance subset, by facet | `catalogue --json --facet appearance`: qualified id, label, kind and domain, unit and scale, `words` |
| C-2 a resolved value per person, keyed by look key, with origin | per person: the `look` key (read from the `appearance` section), each value, and its origin (`authored`, `ruled`, `locked`, `role`, `sampled { stratum, draw }`, `default`) |
| C-3 determinism; a new seed changes only sampled values | §8.5. X-24 runs over this output |
| C-4 the profile's identity | profile id, version, seed and salt, and each distribution's source with its status |
| C-5 the realised distribution against the target | per stratum, for the cast and for `--synthetic <N>` |
| C-6 named failure | §8.2's errors, before any output |
| C-7 declared numeric support | per `quantity` and stratum, the returnable range after `clip_sd` |

- **Ruled values flow back** through the lock (§8.4). S26's casting gate calls `mineworld attributes lock`.
- **`sampling: none`** (S26 §7.4b.4) is a world whose `configure/attributes.yaml` names no profile: every value comes from
  people files and the lock, and `resolve` reports `no profile`.
- **Where profiles live changes.** Today they are in the 3D Presentation Pack
  (`characters/populations/`, A-10). They move to a data pack the world requires (`entities/populations-us`,
  S30-e), because a population is world content that cognition, agents and clients read, not art (Q-S30-2).
  The casting config's `profile:` key then names the world's own profile, or is dropped in favour of
  reading it from the world.
- **S26 never lists attribute ids in code** (X-S26src-10). The distinctness categories in `casting.yaml`
  name qualified ids, and S30's `UnknownAttribute` refuses a stale one.

---

## 11. Persistence, digests and migration

### 11.1 Facts

| Fact | Owner | When | Payload | Audience |
| --- | --- | --- | --- | --- |
| `traits-stated` | `attributes` | genesis, one per person with at least one plain trait | sorted `(slug, value)` pairs; no origin | the holder only, since it holds private values. Disclosure is per observer, at observation time (§9.1). The fact itself is private |
| `trait-changed` | `attributes` | an accepted proposal | `{ attribute, from, to, because }` | holder; plus observers present, if the attribute is `perceivable` |
| `attribute-change-proposed` | `attributes` (vocabulary) | emitted by a declared proposer | `{ person, attribute, to, because }` | none (an internal request) |
| an owner's own facts | that owner | its own rules | its own vocabulary | its own rules |

- **Derived attributes write nothing** (§5.5).
- **Origins and sources are not saved** (§8.5). The save holds world truth; provenance is a tool's output.
- **Replay** (`ARC-25`) reduces `traits-stated` and `trait-changed` exactly. The sampler runs only at
  genesis, so a resumed world never re-samples.

### 11.2 Digests, golden fixtures and re-baselining

- **Worlds without `attributes`** (and without owner packs' attributes) are byte-identical. This is checked
  for all four reference worlds in S30-b and S30-c (X-S30-9).
- **Adopting attributes in a reference world** (S30-g) adds genesis facts and moves that world's
  `facts-300s`, `journal-300s`, `sealed-300s` and possibly summaries. A **re-baseline** means:
  - re-recording the world's section of `scripts/baselines.txt` from a long record of the PR's head with
    `ci_parity.py baseline write`;
  - adding a `reason S30-g:` line that states which columns moved and why ("genesis gains one
    traits-stated fact per person; no later fact moved");
  - showing in the PR that **only the expected columns moved**. A moved summary is a stop, because
    attributes change no behaviour in S30-g.
- One re-baseline per world, done once, in S30-g. S30-f's cognition changes move no world digest, because
  cognition is not world state. They move only the recorded prompt goldens of the LM controller, which
  are re-recorded with reasons.
- **Migration.** There is no compatibility layer. A save made before S30 has no `traits-stated`. Resuming
  it under a composition that adds `attributes` is a composition change, refused by name as any added
  pack is today. A user starts a new world, which is the standing rule for composition changes in MVP-0.

---

## 12. Invariants (proposed)

| # | Invariant | Checked by |
| --- | --- | --- |
| I-S30-1 | Every attribute has exactly one owner, and only the owner writes it | X-S30-7; structural test over the catalogue and the installed set |
| I-S30-2 | A derived attribute is never stored and never stated as a fact | X-S30-8 |
| I-S30-3 | A private or hidden attribute never appears in another seat's observation, MCP view or client panel | X-S30-4; AC-S30-10 |
| I-S30-4 | Every value in the world passed its declaration's check at load or at its owner's acceptance | X-S30-2 |
| I-S30-5 | A System reads an attribute only through a handle obtained at install, never by slug at runtime, and never a `text` | X-S30-11 |
| I-S30-6 | The sampler is a pure function of (catalogue, profile, lock, people files without names, seed, salt), identical on three platforms | AC-S30-7; X-S30-6 |
| I-S30-7 | Authored and ruled values are never re-drawn | AC-S30-8; X-S26src-13 |
| I-S30-8 | `kernel/` and `contracts/` contain no attribute vocabulary | AC-S30-12 |
| I-S30-9 | What a person learned about another is memory, never world state | review; §9.5 |
| I-S30-10 | A world without the `attributes` pack and owned attributes is byte-identical to before S30 | X-S30-9 |

---

## 13. PR split

### 13.1 The PRs

```text
S30-a specs + records ──┐
S30-b authoring seam ───┼─→ S30-c attributes pack ─┬─→ S30-d profiles + sampler ─┬─→ S30-g worlds + S26 switch
                        │                          │                             │
                        │                          └─→ S30-f consumers ──────────┤
                        └────────────────────────────→ S30-e curated catalogue ≤ 100 + backends/masks + cited profiles
Also MVP-0, release-blocking (Re-ruled 2026-10-15): S30-h MatrAIx backend + masks + mapping + demo (after c, d, e, f)
  · S30-i learnable/tell (after f, S28-c) · S30-j runtime sampling (after d) · S30-k localisation (after f)
```

| PR | Scope | Depends on | Integration checkpoint | Adversarial criteria | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S30-a** specs and records | `CORE_CONCEPTS.md` §4 (new §4.5 "Attributes", §15); `MODULE_SPEC.md` §§2, 3, 4.1 (the `attributes:` section, `configure/attributes.yaml`, data directories); `DECISIONS.md` (`ARC-120`…`ARC-126`, `DEP-66`…`DEP-68`, the dated amendment to `ARC-91` item 3); `overall.md` S30 entry | step approval | documents agree with each other (the doc checks pass; a review against §§4–11) | none (docs only) | Opus |
| **S30-b** authoring seam | `authoring/`: data directories, context-seeded decode, owner `resolve`; `worldpack/` loader wiring; no pack uses it except a test fixture pack | S30-a | a fixture pack owning a data directory and a configuration loads; a person-file value is refused at its own line against a data-declared schema | X-S30-9 every reference world byte-identical; an unknown data directory refused naming known ones; a seeded fact from another vocabulary still refused (`ARC-31` item 3) | Opus |
| **S30-c** `attributes` pack and model crate | `authoring/attributes/` (catalogue, kinds, values, handles, `AttributeOwner`, renderer, filter); `systems/attributes/` (section, `traits`, facts, proposals, disclosure, derived age); `mineworld attributes catalogue/show/check`; a minimal test catalogue | S30-b | `mineworld server` on a fixture world: Alice's own observation lists her attributes, Bob's lists Alice's perceivable ones only; a save/restart keeps them; a proposal changes an `evolving` trait with a cited cause | X-S30-1…4, X-S30-7, X-S30-8, X-S30-10, X-S30-11 | Opus (design) → Haiku (implementation from a commit plan), Opus review |
| **S30-d** profiles and sampler | sampler in the model crate (integer weights, masks, adjustments, normal table, largest remainder, lock, roles); `configure/attributes.yaml`; `mineworld attributes resolve/lock`; synthetic-N report | S30-c | `resolve --json` on a fixture world prints C-1…C-7; a 1,000-person synthetic run matches within AC-S30-5 | X-S30-5, X-S30-6, X-S30-12…14; X-24/X-25 over the output | Opus |
| **S30-e** the curated built-in catalogue (≤ 100, gameplay-driven) and cited profiles | `entities/person-attributes` (the 94 of §6.1, each with its reader line; masks `core` and `appearance`; `backend.yaml`); the backend and mask loader in the model crate (`ARC-126`); `entities/populations-us` (San Diego County v3 from A-10 with each figure re-read or kept `verify`; lakeside); `DEP-68` sources | S30-c (format); S30-d (profile format) | `attributes check` on both packs is clean except listed `verify` marks; the San Diego profile reproduces A-10's marginals | X-S30-15 a profile figure without a source refused; X-S30-16 two catalogue entries sharing an id refused; X-S30-20 a 101st built-in, or one without a reader line, fails the catalogue check | Opus (backend and mask loader, sources); Haiku for catalogue entry from §6.1's table |
| **S30-f** consumers | cognition (`mineworld_cognition` attribute renderer, persona `Attributes` part, perceived-person phrases, budgets, golden tests mirroring the Rust renderer); SDK typed reader for decision models; MCP `description` and `me.attributes` (S27-d's or a successor's surface); 2D and 3D inspect panels | S30-c; S27-d merged (MCP); P6b merged (persona block) | one scenario: an agent, an LM-free controller and both clients see the same phrases for Alice; Bob's private attributes absent everywhere | X-S30-4 across all four surfaces; X-S30-17 the prompt stays within its byte budget with 200 attributes | Haiku (Python, clients) from a commit plan; Opus review |
| **S30-g** reference worlds and the S26 switch | `attributes` and `person-attributes` and a profile in `social-cafe`, `market-town`, `lakeside` (not `bodies-yard`); `attributes.lock.yaml` with S26's ruled values; one re-baseline per world; S26 casting reads `resolve --json`; interim Python sampler removed | S30-d, S30-e; S26-cast merged (its ruled briefs become lock entries) | `mineworld attributes resolve social-cafe` equals the briefs' ruled values; the casting page's distribution table regenerates from S30 output | AC-S30-13; X-S26src-13 (ruled survive); only the expected baseline columns move | Haiku from a commit plan; Opus review |
| **S30-h** the optional MatrAIx backend and demo (MVP-0, release-blocking) | `entities/attributes-matraix` (verbatim import at the pinned commit, overlay, licence exclusions, `LICENSE`, `NOTICE`, `tools/import.py`, masks `matraix-core-100` and `all`, `mappings/core.yaml`); `worlds/matraix-demo`; the guide section; the README attribution line; CI: import byte-equality, the isolation grep (X-S30-18), the build-and-test job without the pack (X-S30-19); the `log-linear` mode (Q-S30-9 ruled yes, 2026-10-15) | S30-c, S30-d, S30-e, S30-f merged | the demo world resolves, runs and shows MatrAIx attributes in briefs, prompts, MCP and both panels; switching `backend: mineworld` back changes nothing else | X-S30-18, X-S30-19; an excluded dimension present fails the import check; a mask naming an unknown id fails by name; a mapping value missing fails by name | Opus (import, overlay, mapping, mask selection); Haiku for the demo world and guide from a commit plan |
| **S30-i** learnable attributes (MVP-0, release-blocking) | `tell-about-self`; S28 `stable` fact ingestion | S28-a…c; S30-f | Bob tells Alice his chronotype; Alice recalls it; Carol, absent, does not | the fact never widens Alice's observation; a `private` attribute cannot be told | Opus |
| **S30-j** runtime sampling (MVP-0, release-blocking) | `sample_newcomer(profile, current population, causing EventId)` in the model crate, stratified against the current population's realised counts; an owner hook so the System that creates a Person seeds its attributes in the same reduction; a fixture creator (F-S30-10) | S30-d | a newcomer's values reproduce on replay and across kill and resume; 1,000 synthetic arrivals keep the world within AC-S30-5's tolerance | re-sampling on resume is detected; a newcomer created without its seed is refused | Opus |
| **S30-k** localisation (MVP-0, release-blocking) | label packs `labels/<locale>.yaml` (labels, phrases, words); zh-Hans complete for the built-in catalogue and reviewed; locale selection in cognition, MCP and both panels; fallback to English | S30-f | zh-Hans 2D and 3D panels and a zh-Hans "who I am" render every attribute of a fixture person | a missing translation falls back to English and is listed by `attributes check --locale zh-Hans` | Haiku for entry; a reviewed zh-Hans pass; Opus review |

*Note 2026-10-15 (S30-b C0, for S30-a F-S30a-8 and R-S30k-a-1).* The S30-k row read "label packs
`attributes/labels/<locale>.yaml`". The seam refuses a subdirectory inside a data directory (S30-b §5.6 step 2),
so locale labels are `labels/<locale>.yaml`, a directory `attributes` owns (`MODULE_SPEC.md` §2's table).

### 13.2 Placement in MVP-0 and MVP-1

- **MVP-0: S30-a…S30-g.** Reasons:
  1. S26's casting, which is MVP-0, is designing against S30's sampler now, and the interim sampler is
     meant to be replaced, not kept (A-11).
  2. AC-16's information-rich observation is an MVP-0 acceptance, and today an agent cannot tell who it is
     looking at (F-S30-3).
  3. S31's notices, designed in parallel, need the subscription attribute (§9.6).
- **Re-ruled 2026-10-15 (operator): everything ships before the public release.** "我们要发布就得把这些功能全部做好".
  S30-h, S30-i, S30-j and S30-k are MVP-0 and **release-blocking**; S23's release checklist gains "S30 complete
  (a…k)". S30-j no longer waits for S21 (F-S30-10); S30-i's S28-a…c are MVP-0 (step-28 QS28-1).
- *Original ruling, kept visible (Q-S30-3):* S30-a…g in MVP-0 and S30-h…k in MVP-1. Superseded.

### 13.3 Start conditions and order

- S30-a starts on step approval. It is a design-doc-only PR, merged after review under the standing
  design-doc rule.
- S30-b and S30-e's table-writing can run in parallel after S30-a. S30-e's data commits wait for S30-c's
  format, and its profile commits wait for S30-d's.
- S30-c starts when S30-b has merged. S30-d and S30-f start when S30-c has merged, in parallel.
- S30-g starts when S30-d and S30-e have merged and S26-cast has merged.
- S30-h starts when S30-c, S30-d, S30-e and S30-f have merged. S30-i starts when S30-f and S28-c have merged.
  S30-j starts when S30-d has merged. S30-k starts when S30-f has merged.
- **Release gate:** the public release (S23) does not ship until S30-a…k have all merged.
- **S31 ordering.** If S31's notices PR is frozen before S30-c merges, it codes against §5.1 and §9.6 and
  takes the model crate as a dependency at S30-c's merge. A conflicting change to §9.6 is a material stop
  for both.
- Every PR gets its own PR design, freeze and fresh session, in its own worktree.
- No local full-workspace test runs (R-LOAD-1). CI on the exact head is the gate.

### 13.4 Step acceptance (adversarial; fixed before measuring)

| | Criterion | How it is shown to fail when broken |
| --- | --- | --- |
| AC-S30-1 | A Person carries ≥ 150 attributes in a fixture world (the 94 built-ins plus custom ones; and, under the MatrAIx backend with mask `all`, ≥ 1,000), all disclosed to the holder and validated | `attributes show` lists fewer, or the load accepts an invalid one |
| AC-S30-2 | The built-in catalogue has ≤ 100 attributes across the six groups, each with a reader line naming a reader that ships by release | the catalogue check prints the count and refuses a missing reader (X-S30-20) |
| AC-S30-3 | Adding `freckles` in a world file (§7.3) reaches `catalogue`, `resolve`, S26 briefs and prompts, other observers' views, the holder's "who I am", MCP `description` and both panels, with `git diff --stat` outside `worlds/` and `populations` empty | any surface lacks the line |
| AC-S30-4 | A user profile (a fixture "harbour-of-elsewhere" with its own vocabulary) replaces San Diego County by editing `configure/attributes.yaml` only | anything outside the world and its profile has to change |
| AC-S30-5 | A synthetic 1,000-person resolution matches every declared marginal within 0.5 pp and every conditional within 2 pp for groups of ≥ 50 | the report's worst deviation |
| AC-S30-6 | No masked value ever occurs (for example a `female` with a beard under the fixture mask); a mask that empties a row is refused by name | a planted violation in 10,000 synthetic people |
| AC-S30-7 | `resolve --json` is byte-identical across two runs and on Linux, macOS and Windows; a different seed changes only `sampled` values | CI compares the three platforms' outputs |
| AC-S30-8 | Authored, ruled and locked values survive a new seed and a new profile version | a diff restricted to non-`sampled` origins is non-empty |
| AC-S30-9 | Every change to a stored attribute cites its cause; derived attributes write no fact | a save scan |
| AC-S30-10 | A `private` attribute never appears in another seat's observation, MCP view, inspect result or client panel, over a 30-day run | X-S30-4's scan over every observation |
| AC-S30-11 | Cognition, MCP, both clients, the SDK reader and a System read through the declared paths only | X-S30-11 and the scenario of S30-f |
| AC-S30-12 | `kernel/` and `contracts/` grep clean for the model crate's names (`AttributeId`, `AttributeValue`, `Catalogue`, `Handle<`, `AttributeOwner`), the `traits` component type, the S30 fact types and every built-in qualified slug (F-S30-9) | CI grep |
| AC-S30-13 | S26 casting runs on `resolve --json` with no hard-coded attribute list, and the ruled briefs are unchanged | X-S26src-10, X-S26src-13 |
| AC-S30-14 | A cook's default subscription includes `prices.ingredients` by role; `unsubscribe` changes it through S31's own fact; no other seat sees it | S31's test over §9.6 |
| AC-S30-15 | Backends and masks: switching `backend` and `mask` in `configure/attributes.yaml` changes the world's catalogue and nothing else; what a mask disables appears on no surface | the demo and a fixture world, scanned |
| AC-S30-16 | The workspace builds and every test passes with `entities/attributes-matraix` absent, and nothing outside it (except the named exceptions) names MatrAIx | X-S30-18, X-S30-19 |
| AC-S30-17 | The MatrAIx demo shows its attributes in briefs, prompts, MCP and both panels, with the attribution | the demo's scenario test and its guide step |

### 13.5 Planted mutations and adversarial tests (each must fail by name, then be reverted)

| # | Plant | Expected failure |
| --- | --- | --- |
| X-S30-1 | a person file sets `core.stature: blond` | refused at the line: wrong kind, domain printed |
| X-S30-2 | `core.stature: 3.1` (above `max`) | refused at the line |
| X-S30-3 | an unknown attribute `core.statue` | refused, nearest `core.stature` suggested |
| X-S30-4 | disclose `private` values to every observer | the 30-day observation scan fails, naming the attribute and seat |
| X-S30-5 | a profile with `a given b`, `b given a` | `DependencyCycle` printing the cycle |
| X-S30-6 | the sampler reads display names (planted parameter) | renaming every person changes resolved values: the rename test fails |
| X-S30-7 | `economy` writes a `traits` entry | the owner check refuses at install |
| X-S30-8 | `age-years` stored at genesis | the derived-attribute fact scan fails |
| X-S30-9 | the seam changes ordering for worlds without sections | a reference world digest moves |
| X-S30-10 | the fixture System reads `chronotype` by slug at runtime | the structural test refuses `Catalogue::handle` outside install |
| X-S30-11 | a System requests `Handle<Text>` | it does not compile (no `SystemReadable` impl); the compile-fail test passes |
| X-S30-12 | a row summing to 0.9 | `BadDistribution` naming the row |
| X-S30-13 | an `f64` draw replaces the integer path | the three-platform byte comparison fails, or the golden draws move |
| X-S30-14 | the sampler re-draws a ruled value | AC-S30-8 fails |
| X-S30-15 | a distribution with no `source` | `UncitedDistribution` |
| X-S30-16 | two `core.stature` declarations | `DuplicateAttribute` |
| X-S30-17 | 1,240 attributes (MatrAIx, mask `all`) rendered into "who I am" | the byte budget holds, and the cut is in `prompt_rank` order |
| X-S30-18 | `matraix` planted in a file outside the MatrAIx pack and the named exceptions | the isolation grep fails, naming the file and line |
| X-S30-19 | a core test that loads a MatrAIx dimension | the build-without-the-pack job fails |
| X-S30-20 | a 101st built-in attribute, or one without a reader line | the catalogue check fails by name |

---

## 14. Risks

| # | Risk | Mitigation |
| --- | --- | --- |
| R-S30-1 | Sampling ethnicity, religion or other sensitive attributes produces stereotyped people (for example, conditional rules keyed on tags) | sensitive attributes are `private` by default and never rendered into others' views; profiles cite sources; `attributes check` warns when a sensitive attribute has a tag- or role-keyed parent; nothing reads names (X-S30-6); briefs are still reviewed by the operator (Q-S26-7) |
| R-S30-2 | Hundreds of attributes swamp prompts | facets, `prompt_rank` and byte budgets (§9.2); X-S30-17 |
| R-S30-3 | Observation size grows with many perceivable attributes | `perceived_as` is short; S30-c measures; a cap with a named truncation marker if needed |
| R-S30-4 | The seam (S30-b) touches the loader that every world uses | it is additive; X-S30-9 over all reference worlds; Opus implementer |
| R-S30-5 | S26 and S30 drift on the profile format during parallel work | §10 is the contract; S26-src2 reads the interim format through an adapter until S30-g, then the adapter is removed |
| R-S30-6 | S31 freezes a declaration shape before S30-c | §9.6 is stated now; S31 codes against it; a change is a material stop for both |
| R-S30-7 | The curated catalogue's wording becomes an accidental standard that is hard to change | semantic versioning (§6.3); `since`; ids never reused |
| R-S30-8 | MatrAIx's dimensions file changes upstream | the pack pins a commit and digest; upgrades are reviewed PRs with a diff report; removals that a mask, mapping or profile names fail by name |
| R-S30-10 | The MatrAIx backend leaks into core code through convenience | the isolation grep and the build-without-the-pack job (X-S30-18, X-S30-19) |
| R-S30-9 | `verify` marks linger in shipped profiles | `attributes check` lists them; Q-S30-5 decides whether a release refuses them |

---

## 15. Proposed edits elsewhere (made by the PRs named)

### 15.1 `docs/CORE_CONCEPTS.md` (S30-a)

- New §4.5 **Attributes**: "An attribute is a declared, typed property of a Person, in one of the four
  classes of §§4.1–4.3: identity (birth), traits (stature, chronotype) or a view of mutable state (wealth
  band, occupation). Each attribute is declared as data and has exactly one owning System. A derived
  attribute is computed, never stored." `Attribute` becomes a defined term. It is not a synonym for
  `Component`: a component is one owner's typed state, and attributes are the declared vocabulary over it.
- §15.1: an Observation's entities may carry the observer-filtered `traits` component, which needs no
  wording change; an example line is added.

### 15.2 `docs/MODULE_SPEC.md` (S30-a; S30-c; S30-d)

- §2: an Entity Pack may carry `attributes/`, `populations/` and `vocabularies/`. MVP-0 then reads them,
  and the "item kinds only" sentence is amended.
- §3: `reads_attributes`, `proposes`, and `AttributeOwner`.
- §4.1: the person file's `attributes:` section (the pack's one section, A-2), `configure/attributes.yaml`, the lock file, data
  directories in World Packs, and every refusal of §§5, 7, 8.
- §8.1: `mineworld attributes catalogue | show | check | resolve | lock`.

### 15.3 Step-26 (the S26 sources amendment's owner)

- §7.4a items 4–5 point to S30 §8 for the schema and the sampler.
- §7.4b.1's C-1…C-7 are confirmed as stated (§10).
- Profiles move from `presentation/…/characters/populations/` to `entities/populations-us` (Q-S30-2).
- A ruling at a casting gate writes the lock (§8.4).

### 15.4 Step-27 / S27-d, step-28, S31

- S27: `PersonLine.description` and `me.attributes` (§9.3), by S30-f or the then-current adapter PR.
- S28: `traits` snapshots are deduplicated like other unchanged components. The learned-attribute
  `stable` fact is S30-i's (MVP-0, release-blocking).
- S31: §9.6 N-1…N-6.

### 15.5 `docs/DECISIONS.md` (ids assigned 2026-10-15; entries written by S30-a)

| Id | Record |
| --- | --- |
| `ARC-120` | attributes are declared data with one owner each; `attributes` owns plain traits; views, not copies (§4) |
| `ARC-121` | the authoring seam: data directories, context-seeded decode, owner resolve (§4.3) |
| `ARC-122` | the attribute model: kinds, typed values, handles, visibility, mutability, rendering (§5) |
| `ARC-123` | population profiles and the deterministic stratified sampler; population seed ≠ run seed; lock file (§8) |
| `ARC-124` | the owner interface, with S31's subscriptions as its first user (§9.6) |
| `ARC-125` | dated amendment to `ARC-91` item 3: apparent descriptors are perceivable attributes; `look` stays opaque (§4.4) |
| `ARC-126` | person-generation backends and masks as data; the MineWorld backend is the default; a backend runs no code (Re-ruled 2026-10-15 (operator, final), §§6.4–6.5) |
| `DEP-66` | MatrAIx: **adopted as an optional backend pack** (Re-ruled 2026-10-15 (operator, final)). Licence check (repository MIT; 50 dimensions with narrower `source_origin.license` and the `matraix-research-only` coreset excluded); the pinned commit and digest; the update procedure (import tool, byte-equality CI, reviewed upgrades); attribution (`LICENSE`, `NOTICE`, README line, citation); isolation (grep, build without it). *Earlier texts kept in §16.2:* "taxonomy adapted, optional import in MVP-1" (step approval); "inspiration only, no dependency" (first re-ruling); "adopt, masked to 100" (third re-ruling) |
| `DEP-67` | the sampler is our own; pgmpy, pomegranate, PopulationSim and MatrAIx's sampler code declined; MatrAIx's DAG as an optional data mode is Q-S30-9 (§8.6) |
| `DEP-68` | population data sources: ACS (public domain), NHANES (public domain), Monk Skin Tone (CC BY 4.0, attribution in the pack) (§8.2) |

### 15.6 `overall.md`

- Add S30 to §3, after S29, with this file as its step document; every S30 PR is MVP-0 and release-blocking
  (§13.2, re-ruled 2026-10-15).
- Add AC-S30 to §4's coverage.
- Note S26-cast's dependency on S30-g for the sampler switch, which does not block S26-cast's merge.

---

## 16. Questions for the operator (each with a recommendation; all ruled 2026-10-15)

Each question has a recommendation. They are limited to what the rules do not settle.

| # | Question | Recommendation |
| --- | --- | --- |
| Q-S30-1 | `ARC-91` item 3 says apparent age, stature, build, skin and hair are not world facts. Should they become perceivable attributes owned by `attributes`, so that agents and LM controllers are told what a human sees (AC-16), while `look` stays the only key a renderer binds? | **Yes**, as a dated amendment (`ARC-125`) |
| Q-S30-2 | Population profiles currently live in the 3D Presentation Pack. Should they move to a data pack the world requires (`entities/populations-us`), so that the world, cognition and casting read one source? | **Yes**. A population is world content, and casting reads the world's resolution |
| Q-S30-3 | Placement: S30-a…g in MVP-0 and S30-h…k in MVP-1? | **Yes**. The fallback is the minimal MVP-0 subset of §13.2 |
| Q-S30-4 | `persona` (P6a): merge it as frozen and catalogue its fields as `persona`-owned text attributes now, reviewing the overlap of `persona.traits` with the typed Psychology group after S30-f? Or fold `persona` into `attributes` before P6a merges? | **Merge P6a as frozen; review after S30-f** (`CLAUDE.md` §4 rule 11) |
| Q-S30-5 | Should a release (S23) refuse a reference world whose profile still carries `verify` marks? | **Yes for figures marked as data; no for `authored` conventions**, which are shown as authored |
| Q-S30-6 | Wealth: should S31 seed opening balances from a sampled `wealth-band` attribute (`economy` owns it, sampled through §4.2 item 4), or keep balances authored in economy's own section? | **Sample through the owner interface**, so a world's wealth distribution is data like every other distribution. S31 decides the bands |
| Q-S30-7 | Built-in catalogue v1 size: about 200 MineWorld-authored attributes now, with MatrAIx's ≈ 1,240 as an optional MVP-1 import? | **Yes** |
| Q-S30-8 | Sensitive attributes (ancestry, religion, sexual orientation, disability, income): built in as `private` by default, never in others' views or the MCP description, sampled only from cited profiles? | **Yes**. Worlds may narrow them further but never widen them |

### 16.1 Rulings, 2026-10-15

| # | Ruling |
| --- | --- |
| Q-S30-1 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Yes: the `ARC-125` amendment. Appearance descriptors become perceivable attributes; `look` stays the renderer key |
| Q-S30-2 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Yes: populations move to a data pack the world requires |
| Q-S30-3 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Yes: S30-a…g are MVP-0; S30-h…k are MVP-1 |
| Q-S30-4 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Merge P6a as frozen; review the overlap after S30-f |
| Q-S30-5 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Yes for figures marked as data; no for authored conventions |
| Q-S30-6 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Sample wealth through the owner interface: `economy` owns `wealth-band`; S31 decides the bands |
| Q-S30-7 | **Ruled 2026-10-15 (operator answered directly in the primary session).** About 200 built-in attributes now; the MatrAIx import is an optional MVP-1 pack |
| Q-S30-8 | **Ruled 2026-10-15 (operator answered directly in the primary session).** Sensitive attributes are `private` by default; worlds may narrow, never widen |

The questions table above is kept as asked. Where §§4–13 state a recommendation that one of these settles, it now reads as a ruling.

### 16.2 Re-rulings, 2026-10-15 (operator, directly in the primary session; history kept)

The rulings of §16.1 stay visible. The re-rulings came in this order; each later one supersedes what it names.

| # | Operator, verbatim | Effect | Status |
| --- | --- | --- | --- |
| RR-S30-1 | "我们不要对matrixl有依赖，我只是让你学习它的方式" | no MatrAIx dependency; learn from its approach only; withdraw the import pack | **superseded** by RR-S30-4 and RR-S30-5 (MatrAIx returns, but only as an optional backend; RR-S30-1's "no hard dependency" survives in RR-S30-5) |
| RR-S30-2 | "我们要发布就得把这些功能全部做好" | every S30 slice ships before the public release: S30-h…k are MVP-0, release-blocking (supersedes Q-S30-3's MVP-1 part) | **in force** (§13.2) |
| RR-S30-3 | "MatrAIx的1240个属性里面冗余的太多了，我们没必要全部导入，留最重要的，影响用户游戏体验的就行，我觉得不超过100个" | keep only what affects play, ≤ 100 (supersedes Q-S30-7's "about 200") | **in force** for the built-in catalogue (§6.1, 94 attributes) and as the default MatrAIx mask's bound (`matraix-core-100`) |
| RR-S30-4 | "算了我改主意了，我们就用MatrAIx吧，原汁原味，这样我们还可以让他们帮我们宣传。我觉得我们可以导入他们的，但是我们用mask，只选最重要的100个。这个mask也应该是模块化的，如果用户愿意直接全部用，也可以" | import MatrAIx as-is behind a modular mask, default 100, `all` available | **in force as refined by RR-S30-5**: the import, the masks and the licence-only exclusions stand; it is optional, not the default |
| RR-S30-5 (**final**) | "我觉得我们应该是支持MatrAIx为后端生成人物，并给出demo，但是不要把它们作为强依赖，而是一个可选项" | pluggable backends; MineWorld's curated backend is the default; MatrAIx is an optional backend pack with a demo; nothing else depends on it (§§6.4–6.7, `ARC-126`, `DEP-66`) | **in force** — Re-ruled 2026-10-15 (operator, final) |

### 16.3 A question this amendment raises

| # | Question | Recommendation |
| --- | --- | --- |
| Q-S30-9 | RR-S30-4 asked for MatrAIx "原汁原味". Should the optional MatrAIx backend also offer **faithful sampling** from their `full_dag.json` (1,308 nodes; 6,999 directed proposal edges with a valid topological order; 524 masks; 54 full CPTs; q(v \| parents) = softmax(log P0 + γ Σ w (log P − log P0))), as an optional `log-linear` mode? It is technically viable without floats in the world: the import tool converts each term to fixed-point log weights, our engine sums them and exponentiates through a committed integer table, so it is deterministic on three platforms; cycles cannot occur in the proposal graph (checked at import); their energy-scoring repair would not be reproduced. Its priors are global (8.3 % of people under five), so it is conditioned on adults and is never a place's distribution | **Yes, as an option of the MatrAIx backend only**, used by the demo to show their method, never by a reference world (which keep cited local profiles). S30-h prototypes it first; if determinism, load time (a 24.8 MB file) or the ≤ 2 pp fidelity check against their own sampler on the masked set fails, the mode is dropped from S30-h and reported, and the demo uses `profile` mode |

**Q-S30-9 — Ruled 2026-10-15 (operator answered directly in the primary session):** yes, as recommended — an optional `log-linear` mode of the MatrAIx backend only, used by `worlds/matraix-demo`, never by a reference world; S30-h prototypes it first and drops it, reporting to the operator, if determinism, load time or the ≤ 2 pp fidelity check fails.


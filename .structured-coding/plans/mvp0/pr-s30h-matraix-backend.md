# PR S30-h — The optional MatrAIx backend and demo

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        draft 1 (2026-10-26)
Author:                 S30-h PR-design session (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s30h,
                        branch docs/pr-design-s30h, planning base origin/main @ 0b2faf2f (#382)
Approved by / evidence: none yet — the operator freezes this design; §15 holds the questions
Implementation base:    main (origin/main at each part's C0); C0 records the commit and the merged prerequisites
Start condition:        S30-c, S30-d, S30-e and S30-f merged (all four are on main at 0b2faf2f: #280, #293, #300,
                        #373); this design frozen; §15 ruled; the coordinator's dispatch
Execution contract:     §13
Recommended split:      three PRs from this one design (Q-S30h-1): S30-h1 the backend pack, S30-h2 the
                        `log-linear` mode, S30-h3 the demo world and the guide page
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15; amended)
§§0 item 9, 3.2 (M-1 … M-8), 3.3 (F-S30-6, F-S30-11), 5.1, 5.2, 6.3, 6.4–6.7, 8.3, 8.6, 9.1, 12, 13.1 (the S30-h row),
13.3, 13.4 (AC-S30-1, AC-S30-15, AC-S30-16, AC-S30-17), 13.5 (X-S30-17, X-S30-18, X-S30-19), 14 (R-S30-8, R-S30-10),
16.2 (RR-S30-1 … RR-S30-5), 16.3 (Q-S30-9) · **Specified by:** `docs/MODULE_SPEC.md` §§2 ("The three Entity Packs"),
4.1 ("Backends and masks"); `docs/decisions/ARC-126.md`, `DEP-66.md`, `DEP-67.md`, `DEP-68.md` · **Builds on:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md), [`pr-s30d-sampler.md`](pr-s30d-sampler.md),
[`pr-s30e-builtin-catalogue.md`](pr-s30e-builtin-catalogue.md) (the backend and mask loader; Q-S30e-1, Q-S30e-3),
[`pr-s30f-attribute-consumers.md`](pr-s30f-attribute-consumers.md) (every surface the demo shows; R-S30h-f-1) ·
**Parent:** [`overall.md`](overall.md) · **Binding parents:** `CLAUDE.md` §§1.1, 2, 4 (items 1, 2, 5, 7, 8, 9, 10, 11, 12,
16); `/Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md` (speed first; **no world-specific hard-coding**; every
acceptance criterion names a second world or a generated variant).

**Decision ids.** `DEP-69` (proposed, written by S30-h1): the MatrAIx files as imported — which files, their digests,
the value-slug transform, the files not committed, and the terms of the graph's calibration sources. Dated notes under
`ARC-122` (the `map` derivation), `ARC-126` (`keeps`, `mapping`, upstream provenance, `factors`), `DEP-66` (the
exclusions confirmed against the file; Q-S30h-3's owner change) and `DEP-67` (the `log-linear` mode as built, or its
drop report). `ARC-127` … `ARC-129` belong to sibling PRs and are not used.

No model is called. No reference world's bytes move. No money is spent.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, the S30-h row).** "`entities/attributes-matraix` (verbatim import at the pinned commit,
  overlay, licence exclusions, `LICENSE`, `NOTICE`, `tools/import.py`, masks `matraix-core-100` and `all`,
  `mappings/core.yaml`); `worlds/matraix-demo`; the guide section; the README attribution line; CI: import
  byte-equality, the isolation grep (X-S30-18), the build-and-test job without the pack (X-S30-19); the `log-linear`
  mode (Q-S30-9 ruled yes, 2026-10-15)."
- **Integration checkpoint (same row).** "the demo world resolves, runs and shows MatrAIx attributes in briefs,
  prompts, MCP and both panels; switching `backend: mineworld` back changes nothing else."
- **Adversarial criteria (same row).** "X-S30-18, X-S30-19; an excluded dimension present fails the import check; a
  mask naming an unknown id fails by name; a mapping value missing fails by name."
- **The operator, final (RR-S30-5, 2026-10-15).** "我觉得我们应该是支持MatrAIx为后端生成人物，并给出demo，但是不要把它们作为强依赖，
  而是一个可选项" — support MatrAIx as a backend for generating people, ship a demo, never a hard dependency.
- **RR-S30-4.** "我们就用MatrAIx吧，原汁原味 … 我们用mask，只选最重要的100个。这个mask也应该是模块化的，如果用户愿意直接全部用，
  也可以" — as-is, behind a modular mask of at most 100, with `all` available.
- **Q-S30-9 (ruled 2026-10-15).** "an optional `log-linear` mode of the MatrAIx backend only, used by
  `worlds/matraix-demo`, never by a reference world; S30-h prototypes it first and drops it, reporting to the operator,
  if determinism, load time or the ≤ 2 pp fidelity check fails."
- **Q-S30-8 (ruled).** Sensitive attributes are `private` by default; worlds may narrow, never widen.
- **Requirements placed on this PR.**
  - *R-S30h-a-1* (S30-a): the pack's `LICENSE`, `NOTICE` and import tooling must pass the seam's directory rule (a
    `tools/` directory is refused unless it begins with `.` or is owned).
  - *R-S30h-e-1* (S30-e): the backend contract is `backends/matraix.yaml` in its pack; `mapping` and provenance
    digests are S30-h's; its catalogue declares no appearance group and no quantity.
  - *R-S30h-f-1* (S30-f): the overlay gives its masked dimensions facets (`observed`, `prompt`, `speech`) and ranks,
    or they reach the inspect list and the own sheet only; no consumer code changes for MatrAIx.
- **Lane charter (2026-10-18).** Mechanisms are generic and data-driven; a fix that only makes one world pass is
  rejected; every acceptance criterion names at least one second world or a generated variant.
- **R-LOAD-1.** No local full-workspace test run; CI on the exact head is the gate.

---

## 1. Goal

Let a world generate its people from the MatrAIx persona schema by changing two lines of data, while nothing else in
MineWorld knows that schema exists.

1. **One optional Entity Pack** holds the schema exactly as upstream released it, plus a reviewed overlay that adds
   what MineWorld needs (visibility, owner, mutability, facets, ranks, order) and a generated catalogue the engine
   loads like any other.
2. **Generic engine features, each with a second user.** The backend contract gains `keeps` and `mapping`, a
   derivation `map` turns a backend value into a built-in's value, upstream provenance is checked by the import, and
   an optional `log-linear` sampling mode runs a backend's factor model in fixed point. None of them names MatrAIx;
   each is exercised on the toy backend fixture too.
3. **Isolation that is checked, not promised.** A generic check reads each optional pack's own declaration of the
   words it owns and the paths that may name it; another job builds and tests the workspace with every optional pack
   and its dependents removed.
4. **A demo a person can run and a guide page CI runs** on three platforms, showing MatrAIx attributes on every
   surface S30-f built.

---

## 2. Scope

### 2.1 The split (Q-S30h-1)

| Part | What | Depends on | Integration checkpoint |
| --- | --- | --- | --- |
| **S30-h1** the backend pack | the engine's `keeps`, `mapping`, `map` derivation and upstream provenance; the `mappings/` directory; `entities/attributes-matraix` (verbatim upstream files, overlay, import tool and its check, generated catalogue, `backends/matraix.yaml`, masks `matraix-core-100`, mapping, `LICENSE`, `NOTICE`, `CITATION.cff`); the generic optional-pack check (isolation grep, declared checks, the without-pack job); `DEP-69`; specification sync | S30-c, d, e, f merged | `mineworld attributes check` and `resolve --json` on a fixture world under `backend: matraix` (profile mode) are clean; the same fixture under `backend: mineworld` resolves exactly what it did before; the toy backend's mapping and keeps pass the same rules |
| **S30-h2** the `log-linear` mode | the factor-model format and its engine in the model crate; the `factors/` directory; the import tool's graph conversion; the prototype gate (determinism, load time, fidelity); either the mode kept or dropped with a report | S30-h1 merged | a 200-person fixture under `sampling: log-linear` resolves byte-identically on Linux, macOS and Windows; ≤ 2 pp per value against the float reference; or a drop report and nothing merged but the report |
| **S30-h3** the demo and the guide | `worlds/matraix-demo`; its cited profile; its oracle; S30-f's two real-server surface tests made to accept a world and an oracle; `docs/guide/compare/person-backend.md` and its mirror; the README attribution line and its mirror; the demo's baselines section | S30-h1 merged (S30-h2 merged or dropped, for the demo's mode) | the demo resolves, runs, and its server and 2D surfaces equal its oracle; switching to `backend: mineworld` changes the catalogue and nothing else |

S30-h2 and S30-h3 run in parallel after S30-h1. Whichever lands second sets the demo's mode: `log-linear` if S30-h2
kept it, otherwise `profile`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Locale label packs as a format, locale selection, a zh-Hans panel | S30-k (Q-S30h-7); S30-h1 commits MatrAIx's six label packs verbatim so S30-k can convert them |
| The 3D `--probe=attributes` mode, and the MCP and cognition real-server surface tests | S30-f's deferred items (its D-impl-7, the milestone batch); the demo runs them through the same world-and-oracle pair once they exist (R-S30f-h-1) |
| `economy.wealth-band` and any S31 owner | S31 (Q-S30h-3) |
| A reference world using the MatrAIx backend | never (Q-S30-9; RR-S30-5) |
| Running MatrAIx's own code, or their Persona-1M coreset | never (`DEP-66`, `DEP-67`) |
| Re-admitting any of the 50 excluded dimensions | a later reviewed pack change citing the owner's confirmation (`DEP-66` item 1) |
| Any change under `kernel/`, `contracts/`, `server/`, or to any System other than `attributes` | never |

### 2.3 Dependencies

- **Merged:** S30-c (#280), S30-d (#293), S30-e (#300), S30-f (#373). The model crate, the backend and mask loader, the
  sampler, schema 2 of the disclosed block and every consumer exist on `main`.
- **Not needed:** S30-g (reference worlds), S26-cast, S30-i, S30-j, S30-k, S31. S30-h names none of their surfaces
  except as a requirement on them (§11).

---

## 3. Audit (`origin/main @ 0b2faf2f`, 2026-10-26; MatrAIx at its pinned commit)

### 3.1 Anchors in MineWorld

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-h-1 | `BackendFile` has `format, backend, version, license, attribution, catalogues, masks, sampling, provenance, mapping`; `mapping` present is refused ("S30-h's"); `provenance` must be exactly `{ source: mineworld }`; `SAMPLING = ["profile"]` | `authoring/attributes/src/backend.rs:24-160` | h1 opens `mapping` and upstream provenance; h2 adds `log-linear` |
| A-h-2 | Under a backend other than `mineworld`, every `core` attribute that is not of group `appearance` and not a `quantity` is removed "by the backend"; a kept attribute keeps the `core` source it is derived from (D-e-1) | `authoring/attributes/src/catalogue.rs:790-890` | `core.natal-sex`, `core.ancestry` and `core.age-band` — the profile parents of the appearance group — vanish under MatrAIx (F-S30h-5) |
| A-h-3 | `ValueSlug` is `^[a-z0-9][a-z0-9_.-]{0,63}$`; an attribute slug is `^[a-z][a-z0-9_.-]{0,63}$`; a namespace is a pack id `^[a-z][a-z0-9-]{0,63}$` | `authoring/attributes/src/id.rs:1-80` | MatrAIx's ids pass; 6,321 of its 6,347 values do not (F-S30h-1) |
| A-h-4 | A declaration's `group` is one of six fixed groups (`GROUPS`); step-30 §5.1's "or a group the declaring pack declares" is not built; facets are value slugs and open | `authoring/attributes/src/declaration.rs:30-37, 223-240, 438` | MatrAIx's 43 categories cannot be groups without a model change (F-S30h-2) |
| A-h-5 | `Derive` is `Age`, `Band` or `Projection`; a derived value is computed per read and never stored | `authoring/attributes/src/derive.rs`; `declaration.rs:124-130` | the mapping needs a fourth derivation, `Map` (§5.7) |
| A-h-6 | `attributes` owns `attributes/`, `vocabularies/`, `populations/`, `locks/`, `backends/`, `masks/`; `MODULE_SPEC` §2's table already lists `mappings/` and `labels/` as `attributes`' | `systems/attributes/src/data.rs:27-80`; `docs/MODULE_SPEC.md:121, 1378` | h1 adds `mappings/`; h2 adds `factors/`; `labels/` stays S30-k's |
| A-h-7 | The seam ignores directories whose name begins with `.`; a pack root's **files** (`LICENSE`, `NOTICE`, `README.md`) are neither refused nor collected | S30-b F-b-1; `worldpack/src/read.rs:25-26` | upstream files and tools live in `.upstream/` and `.tools/` (R-S30h-a-1) |
| A-h-8 | A profile has no `extends`; a world names one profile; an attribute no row names stays absent | `authoring/attributes/src/profile/mod.rs`; `pr-s30d-sampler.md` | the demo's profile is its own file; a generated priors profile serves AC-S30-1's mask-`all` check (§5.12) |
| A-h-9 | `matraix` appears today in `README.md:220`, `README.zh-CN.md:198`, `docs/MODULE_SPEC.md:146, 157-158`, four `docs/assets/readme/scope-*.svg`, `docs/decisions/{ARC-126, DEP-66, DEP-67, INDEX}.md`, `tests/acceptance/tests/attribute_consumers.rs` (a scan for the word) and `authoring/attributes/src/backend/tests.rs:19` (a refused provenance sample) | `git grep -il matraix` | the step's exception list (§6.6) is incomplete; the isolation check reads its allow-list from the pack (F-S30h-6) |
| A-h-10 | `tools/cli/tests/packs.rs` validates every pack under `worlds/` and `entities/`; `scripts/ci_parity.py` records every world under `worlds/` and G-2 requires a baselines section per world; `furnishings.rs`, `item_names.rs`, `station_visit.rs`, `client_rules.rs` iterate `worlds/` | the files named | a world at `worlds/matraix-demo` joins every generic world test and the nightly parity record (D-h-9) |
| A-h-11 | `scripts/package.py` bundles only `social-cafe`, `market-town` and `bodies-yard` | `scripts/package.py:60` | neither the pack nor the demo is in a release bundle: "not installed by default" holds |
| A-h-12 | S30-f's real-server surface tests read their fixture and oracle at compile time (`include_str!`) and name Bob and Alice | `tools/cli/tests/attributes_consumers.rs:20-40`; `client_2d_attributes.rs:20-30` | h3 makes them take a world and an oracle from the environment, defaulting to today's (§5.13) |
| A-h-13 | S30-f deferred the 3D `--probe=attributes` mode and the two Python real-server tests to the milestone batch | `pr-s30f-attribute-consumers.md` §16 D-impl-7 | the demo's 3D, MCP and cognition checks depend on that batch (R-S30f-h-1) |
| A-h-14 | The guide runner executes every page and its mirror on three platforms; comparison pages start from the corner shop and change one thing; no guide page covers attributes yet | `docs/GUIDE_FORMAT.md` §§1, 9; `docs/guide/compare/README.md` | the guide section is a comparison page, "who the people are made from" (§5.14) |
| A-h-15 | No Rust SHA-256 crate is a direct dependency (`ring` is in the lock only through TLS) | `Cargo.lock`; `Cargo.toml` files | digests are checked by the import tool in Python's standard library, not at load (D-h-6) |
| A-h-16 | No System declares a required read of a `core` id; `motivated`'s `attributes` parameter names ids per world | `git grep "uses:"`; `cognition/motivated` (S30-f §5.7) | the mapping's first reader is a world's decision list; the demo uses the same list under both backends (AC-h3-4) |

### 3.2 MatrAIx, audited (read-only clone at the pinned commit; no code run)

| # | Fact | Evidence |
| --- | --- | --- |
| M-h-1 | Upstream `main` is still at the pinned commit `3633d8dab149a9482a71b024418a49ae828cc941` on 2026-10-26: nothing to upgrade | `git log origin/HEAD -1` of `MatrAIx-ai/MatrAIx-Persona-8B` |
| M-h-2 | `LICENSE` is MIT, "Copyright (c) 2026 MatrAIx"; the README says "MIT — see LICENSE"; `CITATION.cff` exists (5,270 B) | the files at the pin |
| M-h-3 | `persona/schema/dimensions.json`: 603,912 B, SHA-256 `cbf4db9d887dec057431aff409e1d28d83f345e7a31e29a4275a607212ecfca5` (equal to step-30 §6.6), last changed in `284b106c` (2026-08-06); 1,290 dimensions in 43 categories; fields `id, label, category, description, values, index, phrase, defaultValue`, and `source_origin` on 50 | the file |
| M-h-4 | Exactly 50 dimensions carry `source_origin.license`: 28 `citation-only` and 22 `research-use-reference` — the same 50 ids step-30 §6.6 lists. 1,240 remain | the file |
| M-h-5 | Every id passes MineWorld's slug rule; no duplicate id. Values: 2–16 per dimension, mean 4.92, 6,347 in all, the longest 41 bytes; only 26 are already value slugs. Lower-casing and replacing every run of other characters by `-` gives no collision, no empty slug, none over 64 bytes. Seventeen values hold a non-ASCII character (U+00E9 in `Café regular`, U+2014 in the sixteen `mbti_type` values) | the file, read with a stdlib script |
| M-h-6 | Every phrase holds exactly one `{value}` and no other placeholder; the longest label is 46 bytes | the file |
| M-h-7 | `defaultValue` is null on 164, a string on 910 and a list on 216 (the `topic_*` dimensions) | the file |
| M-h-8 | Sensitive dimensions present: `demo_ethnicity_broad`, `demo_religion_affiliation`, `religiosity`, `demo_sexual_orientation`, `gender_identity`, `demo_disability_status`, `demo_citizenship_status`, `demo_veteran_status`, `demo_household_income`, `socioeconomic_band`, `political_lean`, `demo_political_engagement`, `att_immigration`, `lifex_immigration_generation`, `lifex_health_journey`, and 25 `health_*` | the file |
| M-h-9 | Six label packs `persona/schema/labels/dimensions.labels.<locale>.json` (zh-Hans 340,822 B, `reviewStatus: reviewed`; es, ja, ko, pt-BR, zh-Hant), each recording `sourceHash` of `dimensions.json`, keyed by the verbatim English values; labels and values only, no phrases | the files and `labels/README.md` |
| M-h-10 | `persona/synthesis/graph/full_dag.json`: 24,770,474 B, SHA-256 `849dc5e6f2289db87a4d363948f32d0e1db9e9205d3d8a1b52e4ac4e77fedbfd`, version `v4.4_developer_extension`; 1,308 nodes (the 1,290 dimensions plus 18 latent nodes such as `latent_financial_security`, marked `emit`), 6,999 directed proposal edges (5,730 `included_in_proposal_q`), 54 full CPTs, 524 conditional masks, 2 source calibration tables, no undirected or high-order factor; `proposal_topological_order_valid: true` | the file's `metadata` |
| M-h-11 | The proposal is `q_i(v | parents) = softmax(log P0_i(v) + γ_i Σ_p w_{p,i} [log P_{p,i}(v | x_p) − log P0_i(v)])`, `γ_i = 1 / max(1, sqrt(Σ_p w_{p,i}²))`; full CPTs carry `cpt_weight` and `replace_pairwise_parent_edges: true`; conditional masks carry `bad_value_multiplier` (0.0 in the first) and `constraint_semantics: soft_compatibility` | `metadata.joint_probability`; `full_cpts[0]`; `conditional_masks[0]` |
| M-h-12 | The 50 excluded dimensions are nodes of the graph; 354 edges touch them; 394 edges touch the 18 latent nodes | the file |
| M-h-13 | Node priors are world-level: `age_bracket` puts 8.3 % under five and 13 % at 5–12 | `nodes[0].prior` |
| M-h-14 | The graph's calibration sources are `acs_pums_2024_1yr, bls_oews_may2025, cldr, glottolog, gss_1972_2024, ilostat, onet_30_3, pew_npors_2025, pew_rls_2023_2024, stackoverflow_survey_2025, un_wpp_2024, world_bank_wdi, wvs_wave7_csv_v6_0`; the metadata also carries absolute paths from an author's machine | `metadata.raw_calibration.source_refs` |

### 3.3 Findings

- **F-S30h-1. MatrAIx's values are not value slugs** (M-h-5, A-h-3). Step-30 §6.6 and F-S30-11 said "imported category
  values are kept verbatim". Widening `ValueSlug` would loosen every pack's typing and every save. **Q-S30h-2**
  recommends: each value gets a generated slug (lower case; `é` → `e`, `—` → `-`; any other run of non-alphanumerics
  → `-`), and the verbatim string becomes that value's `words`, so every rendered phrase is upstream's text. The
  upstream file is committed byte for byte; only the generated catalogue holds slugs.
- **F-S30h-2. Categories cannot be groups** (A-h-4). Decided with Q-S30h-2: a fixed table in the overlay places each of
  the 43 categories in one of MineWorld's five non-appearance groups (MineWorld's groups were taken from MatrAIx's
  five), and the category itself becomes a facet `category.<slug>` (for example `category.personality-big-five`). A mask
  selects by that facet. No model change.
- **F-S30h-3. An owner cannot project a MatrAIx dimension.** The step's overlay makes `economy` the owner of
  `socioeconomic_band` and maps `economy.wealth-band ← socioeconomic_band`. An owner projects from its own state
  through its own code (`ARC-120`), so `economy` would have to name a MatrAIx id — breaking isolation — and a mapped
  view would compete with `economy`'s own projection of `economy.wealth-band` — breaking single ownership. `economy` is
  not an owner on `main` yet. **Q-S30h-3** recommends: `socioeconomic_band` is a `private` plain trait owned by
  `attributes`, and no mapping targets an owner's attribute.
- **F-S30h-4. The mapping needs a derivation the engine lacks** (A-h-5). Decided (D-h-1): a fourth derivation `map {
  from, values }`, generic, checked for coverage, kind and visibility (§5.7).
- **F-S30h-5. A foreign backend strips the appearance group's parents** (A-h-2). Casting conditions stature, build and
  facial hair on `core.natal-sex`, hair and skin on `core.ancestry`, and many on `core.age-band`. Under MatrAIx all
  three are removed. **Q-S30h-4** recommends a backend field `keeps:` naming the `core` attributes a backend leaves as
  MineWorld's.
- **F-S30h-6. The isolation exception list is incomplete** (A-h-9) and a script that greps for a word names that word.
  Decided (D-h-2): a generic optional-pack check reads each pack's own `.optional.yaml` (the words it owns, the paths
  that may name them, the paths that depend on it, its declared checks); the script names no pack.
- **F-S30h-7. Tests that load MatrAIx cannot live in core test crates** (X-S30-19 makes exactly that a failure). So
  every MatrAIx-specific check runs through the binary, declared in the pack's `.optional.yaml` and run by the generic
  checker; the generic features are tested in Rust on the toy backend.
- **F-S30h-8. Upstream defaults would fill every record.** 910 string `defaultValue`s, carried into `default:`, would
  give every person 910 values with no profile. Decided (D-h-3), following M-1 and F-S30-11: no declaration carries a
  `default:`; an unprofiled attribute stays absent.
- **F-S30h-9. The graph holds what the pack must not, and what the catalogue does not** (M-h-10, M-h-12). Q-S30h-6
  decides how the excluded dimensions and the latent nodes are treated.
- **F-S30h-10. The graph's parameters are derived from sources with their own terms** (M-h-14): public domain (ACS,
  BLS), CC BY (O\*NET, World Bank WDI, Glottolog, UN WPP under CC BY 3.0 IGO), the Unicode licence (CLDR), the ODbL
  1.0 (the Stack Overflow survey), and terms of use (GSS, Pew, WVS). The repository grants MIT over the file. Q-S30h-6
  asks the operator to accept them for an optional, demo-only mode with every source credited.
- **F-S30h-11. A zh-Hans panel needs S30-k** (A-h-6: `labels/` is S30-k's; no format exists). **Q-S30h-7.**
- **F-S30h-12. The 3D, MCP and cognition demo checks need S30-f's deferred tests** (A-h-13). Recorded as R-S30f-h-1.
- **F-S30h-13. A 24.8 MB graph does not belong in the tree.** **Q-S30h-9** recommends committing the schema and label
  files (≈ 2.7 MB) and not the graph: S30-h2 commits only its converted factor file, and a nightly job fetches the
  pinned graph with `git` to re-check the conversion.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

- **Adopted:** MatrAIx's schema, label packs and (h2) graph as data, under MIT (`DEP-66`, `DEP-69`); Python's standard
  library (`json`, `hashlib`, `unicodedata` not used — the transliteration table is explicit) for the import tool,
  as `authoring/attributes/tools/normal_table.py` already does; the existing sampler, mask loader, renderer, surfaces,
  guide runner and parity harness.
- **Considered and declined:** running MatrAIx's sampler or label builder (float Python, an execution model we do not
  own, `DEP-67`); a general rule engine for mappings (a value table is enough, `CLAUDE.md` §4 rule 11); `pgmpy` or
  `pomegranate` for the factor model (float Python, `DEP-67`); a Rust SHA-256 crate (digests are checked once, by the
  import tool, D-h-6); Git LFS for the graph (not used anywhere in the tree; Q-S30h-9 commits the conversion instead).
- **Not forced:** no framework's execution model owns genesis; the factor model is data our engine executes.

---

## 5. Design

### 5.1 The pack (`entities/attributes-matraix/`)

```text
entities/attributes-matraix/
  pack.yaml                      id: attributes-matraix; type: entity-pack; license: MIT; attribution; version 1.0.0
  LICENSE                        upstream's MIT text, verbatim
  NOTICE                         commit, digests, paper, the attribution line, (h2) the graph's calibration sources
  CITATION.cff                   upstream's, verbatim
  README.md                      for humans: what it is, how a world chooses it, link to the guide page
  .optional.yaml                 the optional-pack declaration (§5.10)
  .upstream/                     verbatim, byte for byte, at the pinned commit (ignored by the loader)
    dimensions.json
    labels/dimensions.labels.{es,ja,ko,pt-BR,zh-Hans,zh-Hant}.json
  .tools/                        ignored by the loader
    import.py                    the import and its --check (§5.2)
    overlay.yaml                 the reviewed overlay (§5.3–§5.6)
    test_import.py               the tool's own tests (stdlib unittest)
  .checks/                       ignored by the loader: the fixture worlds and expected outputs of the declared checks
  backends/matraix.yaml          generated
  attributes/<category>.yaml     generated: 43 files, 1,240 declarations
  masks/matraix-core-100.yaml    generated from overlay.yaml's mask list
  mappings/matraix.yaml          generated from overlay.yaml's mapping
  populations/matraix-world-priors.yaml   generated: upstream node priors as marginals (§5.12)
  factors/matraix.yaml           S30-h2 only, generated from the graph (§5.11)
```

- The pack id is `attributes-matraix`, so its namespace is `attributes-matraix` and an id is
  `attributes-matraix.lstyle_sleep_schedule`. Authors may write the short slug where it is unique (step-30 §7.1).
- The backend id is `matraix`; a world chooses it with `backend: matraix`.
- Everything outside `.upstream/` and `.tools/overlay.yaml` is generated, and CI regenerates it byte for byte.

### 5.2 The import tool and its check (`.tools/import.py`, Python standard library, offline)

```text
python3 -I .tools/import.py --upstream <dir> --commit <sha> --out <pack dir>   regenerate
python3 -I .tools/import.py --check                                           regenerate into a temporary
                                                                              directory and compare byte for byte
```

1. Read `.upstream/dimensions.json` and the six label packs. Verify each SHA-256 against `overlay.yaml`'s `upstream:`
   block (commit, file, digest). A mismatch is `ProvenanceMismatch { file, expected, found }`, exit 1.
2. **Licence exclusions.** Drop every dimension whose `source_origin.license` is present and is not `MIT`. Print the
   count by licence (28 `citation-only`, 22 `research-use-reference`). Refuse (exit 1) if any id in the overlay's
   mask, mapping or priors names a dropped dimension (`ExcludedDimensionNamed { id, license }`), or if the count of
   dimensions left differs from `overlay.yaml`'s `expect: { dimensions: 1240 }`.
3. **Values.** For each value, the slug of F-S30h-1; refuse a collision inside a dimension (`ValueSlugCollision`), an
   unlisted non-ASCII character (`UntransliteratedValue { id, value }`), or a slug over 64 bytes.
4. **Declarations.** For each kept dimension, one entry in `attributes/<category-slug>.yaml`:
   `group` from the overlay's category table; `facets` = `category.<slug>` plus the overlay's facets; `kind` =
   `category` with the slugs in upstream order, or `level` in the overlay's order where the overlay declares one;
   `visibility`, `mutability`, `owner`, `prompt_rank`, `derive` from the overlay (defaults: `learnable`, `fixed`,
   `attributes`, none, none); `label` and `phrase` verbatim; `words` = every value's verbatim string; `since: 1.0.0`.
   No `default:` (D-h-3). `description` and `index` stay upstream only.
5. **Backend, masks, mapping, priors.** Write `backends/matraix.yaml` (§5.9), `masks/matraix-core-100.yaml` (Appendix
   A, each entry with its `why`), `mappings/matraix.yaml` (Appendix B), `populations/matraix-world-priors.yaml`.
6. **Determinism.** Keys sorted, fixed float-free formatting, LF line ends, UTF-8; output depends only on the inputs.
   `--check` runs on Linux in CI (§5.10); the committed output is what every platform loads.

### 5.3 Values, groups and facets (Q-S30h-2)

| Upstream | In the generated catalogue |
| --- | --- |
| `id: lstyle_sleep_schedule` | the slug `lstyle_sleep_schedule` in namespace `attributes-matraix` |
| `category: "Behavior: Time"` | `group: behaviour` (overlay table) and facet `category.behavior-time` |
| `values: ["Early bird", "Night owl", …]` | `kind: { category: [early-bird, night-owl, irregular, flexible, shift-based] }` |
| (the values) | `words: { early-bird: Early bird, night-owl: Night owl, … }` |
| `phrase: "their sleep schedule is {value}"` | `phrase: "their sleep schedule is {value}"` — so Alice's text is `their sleep schedule is Early bird` |
| `label: "Sleep schedule"` | `label: "Sleep schedule"` |

The category table (43 rows) is data in `overlay.yaml`, by category prefix first: `Demographic`, `Professional` →
`background`; `Personality`, `Values & Motivation`, `Worldview`, `Risk & Decision`, `State` → `psychology`;
`Expertise`, `Skills`, `Learning`, `Developer` → `capability`; `Behavior`, `Linguistic: Communication` → `behaviour`;
`Interests`, `Health`, `Linguistic: Language` → `lifestyle`. C3 prints the resulting count per group beside the paper's
Table 1 (238, 210, 331, 124, 387 before exclusions) in §16; the table is reviewed data, not a claim of equality.

### 5.4 The overlay: visibility (D-h-4, applying Q-S30-8)

- Default `learnable`.
- `private`: `demo_ethnicity_broad`, `demo_religion_affiliation`, `religiosity`, `demo_sexual_orientation`,
  `gender_identity`, `demo_disability_status`, `demo_citizenship_status`, `demo_veteran_status`,
  `demo_household_income`, `socioeconomic_band`, `political_lean`, `demo_political_engagement`, `att_immigration`,
  `lifex_immigration_generation`, `lifex_health_journey`, and every `health_*` dimension (25). Q-S30-8 named
  ethnicity, religion, orientation, disability, citizenship, veteran status, income and health; gender identity,
  politics and migration are the same kind and are added under that ruling.
- `perceivable`, where a person can see or hear it and a mapping target needs it: `lstyle_smoking` (maps to
  `core.smoking`, which is perceivable), `age_bracket` (derived from MineWorld's age, §5.5).
- A private dimension carries no `observed`, `prompt` or `speech` facet.

### 5.5 The overlay: mutability, owner, facets, ranks, order

- **`age_bracket` is `derived`:** `derive: { band: { from: core.age-years, below: [5, 13, 18, 25, 35, 45, 55, 65, 75,
  85] } }` over its eleven values. MineWorld's age is the one age (`ARC-125`), and `core.age-years` and `core.birth` are
  always kept under any backend (D-e-1).
- **Owner:** every dimension is owned by `attributes` (Q-S30h-3).
- **Facets and ranks (R-S30h-f-1):** every dimension in `matraix-core-100` carries the facets of its reader codes
  (`OV` → `observed`, `ME` → `prompt`, `CV` → `speech`, `DM` → `decision`, `S31` → `notices`), and those with
  `observed`, `prompt` or `speech` carry a `prompt_rank` in S30-f's bands (observed 100–190 after `core`'s appearance,
  prompt 200–490, speech 500–590), spaced by ten. Dimensions outside the mask get only their category facet and no
  rank: they reach the inspect list and the own sheet under `all`, never a budgeted prompt.
- **Level order:** a dimension the mapping reads or a decision list compares is a `level`, its values ordered from
  low to high (for example `big5_*`: `very-low, low, average, high, very-high` — upstream lists them high first). Every
  other dimension is a `category` in upstream order.

### 5.6 Masks (`ARC-126` item 5)

- **`matraix-core-100`** (the backend's default): 98 entries by id (Appendix A), each with a reader line in the
  `core` mask's form (`DM: when they rise and go home`). No `core` id; no private dimension except
  `gender_identity` and `socioeconomic_band`, both `ME` only. The import refuses a mask over 100 entries
  (`MaskOverCap`), mirroring `check_core`'s rule for this pack.
- **`all`** is the engine's built-in mask: every one of the 1,240.
- **`demo_ethnicity_broad`** is not in `matraix-core-100`: `core.ancestry` (kept, Q-S30h-4) is the casting parent, and
  two ethnicity attributes would disagree. It is in `all`.

### 5.7 The mapping and the `map` derivation (generic; `ARC-122`, `ARC-126` item 6)

```yaml
# mappings/matraix.yaml (generated from overlay.yaml)
format: 1
backend: matraix
views:
  core.chronotype:
    from: lstyle_sleep_schedule
    values: { early-bird: early-bird, night-owl: night-owl, irregular: neutral, flexible: neutral, shift-based: neutral }
```

- `backends/<id>.yaml` names its mapping by file stem: `mapping: matraix`. The file lives in `mappings/`, a directory
  `attributes` now owns.
- **Applied only under that backend.** For each view, the target — a `core` attribute that the composition removed
  "by the backend" — is put back with its own declaration (label, phrase, words, facets, rank, visibility unchanged),
  `mutability: derived` and `derive: map { from, values }`. It is never stored and never stated (I-S30-2).
- **Checks, each refused by name at load:**

| Rule | Refusal |
| --- | --- |
| the target is a `core` id this backend removed (not appearance, not a quantity, not kept) | `MappingTargetNotRemoved { target }` |
| the target is not an owner's attribute (Q-S30h-3) | `MappingTargetOwned { target, owner }` |
| the source is in this backend's catalogue | `MappingSourceUnknown { source }` |
| the source is a `category` or `level`; the target a `category`, `level` or `flag` | `MappingKind { source, target }` |
| every source value has an entry; every entry's key is a source value | `MappingUncovered { source, value }` / `MappingValueUnknown` |
| every entry's value is in the target's domain | `MappingValueUnknown { target, value }` |
| the target is not wider than the source (`perceivable` ⊃ `learnable` ⊃ `private` ⊃ `hidden`) | `MappingWidens { target, source }` |
| no two views share a target | `DuplicateAttribute` |

- **A masked-out source** leaves its target absent, and `mineworld attributes backends` lists it ("not provided: the
  mask leaves out the source"). A required read of it is `MissingUnderBackend`, as today.
- **The second user:** the toy backend fixture gains a mapping to `core.chronotype` and its tests (C1).

### 5.8 `keeps` (Q-S30h-4; `ARC-126`)

```yaml
# backends/matraix.yaml (excerpt)
keeps: [core.natal-sex, core.ancestry, core.age-band]
```

- A `core` attribute named in `keeps` stays in the composed catalogue under this backend exactly as MineWorld declares
  it. Each must be a `core` id that the backend would otherwise remove (`BackendMalformed { field: keeps }`), and must
  not also be a mapping target (`BackendMalformed { field: keeps, reason: "both kept and mapped" }`).
- A kept attribute keeps its own derivation sources, by D-e-1's rule (`core.age-band` keeps `core.age-years` and
  `core.birth`).
- The toy backend fixture keeps `core.natal-sex` in its tests (C1).

### 5.9 The backend contract as built (`backends/matraix.yaml`)

```yaml
format: 1
backend: matraix
version: 1.0.0
license: MIT
attribution: "Optional person-generation backend built on the MatrAIx persona schema (Li, Hao et al., 2026, arXiv 2608.04205; MIT), imported unchanged."
catalogues: [<the 43 category files>]
masks: { default: matraix-core-100, ships: [matraix-core-100] }
sampling: [profile]                         # S30-h2 adds log-linear if its gate passes
keeps: [core.natal-sex, core.ancestry, core.age-band]
mapping: matraix
provenance:
  source: github.com/MatrAIx-ai/MatrAIx-Persona-8B
  version: "schema 1.0"
  commit: 3633d8dab149a9482a71b024418a49ae828cc941
  files: { .upstream/dimensions.json: cbf4db9d…, .upstream/labels/dimensions.labels.zh-Hans.json: 7abbcef3…, … }
```

- **Provenance at load (D-h-6):** the engine accepts `{ source: mineworld }` or the upstream shape (`source`,
  `version`, a 40-hex `commit`, `files` of 64-hex digests) and checks the shape only (`BackendMalformed { field:
  provenance }`). It does not hash: the files are in `.upstream/`, which the loader never reads. The import check
  (§5.2 step 1) verifies every digest; `ProvenanceMismatch` is raised there.
- `version` and `license` equal `pack.yaml`'s: D-e-2 deferred this check to S30-h, and the import check performs it.

### 5.10 Isolation, checked generically (D-h-2; X-S30-18, X-S30-19)

```yaml
# entities/attributes-matraix/.optional.yaml
format: 1
optional: attributes-matraix
words: [matraix]                       # matched in any case
may_name:                              # besides this pack's own directory
  - worlds/matraix-demo/
  - docs/guide/compare/person-backend.md
  - docs/guide/compare/person-backend.zh-CN.md
  - README.md                          # the attribution line (one line)
  - README.zh-CN.md
  - docs/MODULE_SPEC.md                # the optional-backend paragraph (S30-a I-a-5)
  - docs/assets/readme/                # the scope images
  - docs/decisions/
  - .structured-coding/plans/
  - tests/acceptance/tests/attribute_consumers.rs   # S30-f's scan for the word
dependents: [worlds/matraix-demo/, docs/guide/compare/person-backend.md, docs/guide/compare/person-backend.zh-CN.md]
checks:                                # run with the debug binary on PATH-free argument lists (§5.10 step 3)
  - [python3, -I, entities/attributes-matraix/.tools/import.py, --check]
  - [python3, -I, -m, unittest, discover, -s, entities/attributes-matraix/.tools, -p, "test_*.py"]
  - [mineworld, attributes, check, entities/attributes-matraix/.checks/all-world, --packs, entities]
  - …                                  # the binary checks of §8 (AC-h1-*, AC-h3-*), each with its expected output file
```

`scripts/check_optional_packs.py` (new, standard library, names no pack):

1. `scan` (in `fast`, every PR): find every `.optional.yaml` under `entities/` and `worlds/`; for each word, scan every
   tracked file (`git ls-files`) outside the pack and its `may_name` paths; any hit fails, printing file, line and
   word. A `.optional.yaml` that lists a path that does not exist fails too.
2. `--self-test` (in `fast`): a planted temporary tree with a word in a forbidden file must fail; in an allowed one,
   pass.
3. `run --binary <path>` (in `core`, after the build): run every declared check, `mineworld` replaced by the binary;
   each check's expected output, when declared, is compared byte for byte.
4. `without` (the without-pack job): delete every optional pack and its `dependents` from a scratch checkout, then run
   `python3 scripts/ci_layer.py core`. A test that needs an optional pack fails there (X-S30-19).

- **Where it runs (Q-S30h-8):** `scan` and `--self-test` in `fast` on every PR (seconds); `run` in `core` (the declared
  binary checks, under a minute); `without` nightly on Linux, and once on S30-h1's final head.
- `authoring/attributes/src/backend/tests.rs:19`'s refused provenance sample `source: matraix` is replaced by a neutral
  word in C1, since upstream provenance is now accepted.

### 5.11 The `log-linear` mode (S30-h2; Q-S30-9, Q-S30h-6)

**What it is.** A backend may supply a factor model as data, `factors/<id>.yaml`, and list `log-linear` in
`sampling`. A world chooses it with `sampling: log-linear` in `configure/attributes.yaml` (new optional field; default
`profile`). The engine runs it; the backend runs no code (`ARC-126` item 2).

**The format (generic, integers only).**

```yaml
format: 1
factors: matraix
scale: 1000000                        # log weights in micro-nats
nodes:                                # topological order, checked
  - { id: lstyle_sleep_schedule, log_prior: [-1503200, …], gamma: 707107 }   # γ in millionths
  - { internal: latent_financial_security, values: 4, log_prior: […], gamma: 1000000 }
edges:
  - { from: age_bracket, to: lstyle_sleep_schedule, weight: 584000, log_ratio: [[…], …] }   # rows: source values
tables:                               # full CPTs
  - { to: demo_citizenship_status, parents: [age_bracket, socioeconomic_band], weight: 650000, replaces_edges: true,
      rows: { "under-5|low-income": [-85000, …], … } }
masks:
  - { to: tool_github, when: { tool_github_copilot: [power-user, regular] }, values: [never-used], log_multiplier: hard }
```

- `id` nodes are attributes of the backend's catalogue; `internal` nodes are sampled and discarded: never stored,
  never disclosed, never in the catalogue (Q-S30h-6).
- **Evidence first.** A node whose attribute is `derived` in the composed catalogue (`age_bracket`), or whose value a
  person's file, the lock or a role fixes, is observed, never drawn; its children are conditioned on it. So `covers:
  adults` holds through `age_bracket`, and the biography wins (I-S30-7).
- **The draw**, per person, per node in topological order: for each value `v`,
  `s(v) = log_prior(v) + γ · Σ_edges w · (log_ratio[x_parent][v] − log_prior(v))`, with full tables applied by the
  rule h2's C0 reads from upstream's sampler source (read only, never run) and records; masks add `hard` (value
  excluded) or a log multiplier. Then `s − max s` is exponentiated through a committed integer table
  (`exp(−k / 1024)` in parts per million, generated like `normal_table.py`), normalised to 1,000,000 by largest
  remainder, and drawn with `mix(seed, salt, attribute, person key)`. Every product is integer with a stated
  rounding (floor toward −∞). No `f64` (`#![deny(clippy::float_arithmetic)]` holds).
- **Under a mask.** Every node is drawn, so the ancestors of a kept attribute keep their effect; only the attributes
  the mask keeps are stored. With `matraix-core-100` that is 98 values per person.
- **Under `log-linear`, a named profile still samples** the attributes the factor model does not cover — the `core`
  appearance group and the `keeps` — first, in its own order, so the factor model sees their values as evidence where
  it has a matching node.
- **What is not reproduced:** upstream's energy scoring and repair (`DEP-67`).

**The import's graph conversion** (`import.py --graph <full_dag.json>`): drop the 50 excluded nodes and every edge,
table row and mask that names one (Q-S30h-6); keep the 18 latent nodes as `internal`; convert values to slugs (§5.3);
compute each term in micro-nats with Python's `math.log`, round half to even, and write integers. The conversion runs
on Linux only, and its output is committed; worlds load only the integers.

**The prototype gate (C1 of S30-h2), fixed before measuring:**

| Gate | Pass | Evidence |
| --- | --- | --- |
| G-1 determinism | `resolve --json` of a 200-person adult fixture under `log-linear`, mask `matraix-core-100`, is byte-identical on Linux, macOS and Windows CI, and across two runs | the three CI artifacts' SHA-256 |
| G-2 load time | `factors/matraix.yaml` ≤ 8 MB; `mineworld validate` of that fixture under `log-linear` takes ≤ 3 s more than under `profile` on the CI Linux runner (median of three) | the CI log |
| G-3 fidelity | for every value of every one of the 98 masked attributes, the share among 2,000 synthetic adults differs by ≤ 2 pp from a float reference that implements §5.11's formula independently in Python (`.tools/reference.py`), driven by the same per-(person, node) uniform draws (common random numbers) | the report table; the worst cell |

The reference is not the code under test (test rules §25): it is written from the file's stated formula, in floats,
by a different hand from the Rust engine. Running upstream's own sampler is Q-S30h-6's option (b).

**If a gate fails:** the mode is dropped. S30-h2 merges only a report (§16, and a dated note under `DEP-67` saying the
mode was prototyped and dropped, with the failing gate's evidence), `log-linear` stays out of the engine's list, and the
demo uses `profile`. The coordinator reports it to the operator (Q-S30-9's rule).

### 5.12 Profiles under the MatrAIx backend

- **`populations/matraix-world-priors.yaml`** (generated): one row per kept dimension that is not `derived` (so not
  `age_bracket`), its marginal the upstream
  node's `prior` (rounded to six decimals by largest remainder), `covers: all`, source `matraix-node-priors` with
  `status: authored` and the cite "MatrAIx world-level node priors (v4.4); global, not a place's distribution". It
  exists so a fixture world under mask `all` carries ≥ 1,000 values per person in `profile` mode (AC-S30-1). No reference
  world names it.
- **The demo's own profile** (`worlds/matraix-demo/populations/matraix-demo.yaml`, h3): the `core` rows the
  appearance group needs (natal sex, age band, birth, ancestry, stature, build, skin, hair, face), copied from
  `entities/populations-us` San Diego County v3 with the same sources and status; and, in `profile` mode only, rows
  for the dimensions the demo's oracle shows, each citing `matraix-node-priors`. Under `log-linear` those rows are
  removed: the factor model draws them.

### 5.13 The demo world (S30-h3; `worlds/matraix-demo`)

- A copy of `social-cafe`'s places (the café and the street), its own six people files, `systems:` as the
  `attributes-consumers` fixture's plus `schedule` and `motivated`'s decision list, `requires: { person-attributes:
  "^1", attributes-matraix: "^1" }`.
- `configure/attributes.yaml`: `backend: matraix`, `mask: matraix-core-100`, `profile: matraix-demo`, `seed:
  matraix-demo-v1`, and `sampling: log-linear` if S30-h2 kept it.
- **One decision list for both backends** (AC-h3-4): `motivated`'s `attributes` names `core.chronotype` (under
  MatrAIx a `map` view of `lstyle_sleep_schedule`) and `core.caffeine`. The same list runs under `backend: mineworld`.
- **Its oracle** `worlds/matraix-demo/.checks/expected.json`, written by hand from the resolved values printed in C1 of
  h3 (the first `resolve --json`), never computed by the code under test: the observer's glance and inspect list of
  one subject; the subject's who I am, how I speak and own sheet; the private ids and texts absent from the observer.
- **S30-f's two real-server tests take a world** (A-h-12): `tools/cli/tests/attributes_consumers.rs` and
  `client_2d_attributes.rs` read `MINEWORLD_ATTRIBUTES_WORLD` and `MINEWORLD_ATTRIBUTES_ORACLE` when set, else their
  own fixture; the oracle names its `observer` and `subject` (default `Bob`, `Alice`). The demo's `.optional.yaml`
  check runs them with the demo's pair. The tests name no pack.
- **The mineworld-backend variant** for AC-S30-15: the check copies the world to a scratch folder, sets `backend:
  mineworld`, `mask: core`, removes `sampling:` and `attributes-matraix` from `requires`, and compares `catalogue
  --json` and the composed facts with the original: only catalogue entries and attribute values differ.
- **Baselines** (D-h-9): the world gets a `scripts/baselines.txt` section, recorded by the nightly parity run with a
  reason line; its 300-day run is the cross-platform check of the whole demo.

### 5.14 The guide page (`docs/guide/compare/person-backend.md` and `.zh-CN.md`)

A comparison page: start from the corner shop of chapter 05, add `attributes` and `person-attributes` with a two-person
profile, show `mineworld attributes resolve --json` for Ana; then require `attributes-matraix`, set `backend: matraix`,
run again and show Ana's values from the other backend; set `mask: all` and show the count; switch back to `backend:
mineworld` and show the first output again, equal. It carries the attribution line and links to the pack's README and
the generated reference. Every block runs in CI on three platforms (`docs/GUIDE_FORMAT.md`); the page is added to
`compare/README.md`'s table.

### 5.15 Attribution (`DEP-66` item 6)

- `pack.yaml` `license: MIT`, `attribution:` the line; `LICENSE` upstream's; `NOTICE` the commit, every digest, the
  paper, and (h2) the thirteen calibration sources with their licences; `CITATION.cff` upstream's, verbatim.
- `README.md`'s "People with depth" row and `README.zh-CN.md`'s gain the attribution line with a link to the pack
  (h3). A root `CITATION.cff` for MineWorld is not created here (D-h-8): the pack carries upstream's.

### 5.16 Sizes, measured (R-S30-3, R-c-2)

C5 of h1 records, on the mask-`all` fixture and the `matraix-core-100` fixture: the generated catalogue's bytes and
load time; `attributes-composed`'s bytes; one person's `traits-stated` bytes; the holder's own `traits` block bytes;
another observer's block bytes. **Fixed before measuring:** under `matraix-core-100`, another observer's block ≤ 2,048
B (S30-f's trim rule) and the holder's ≤ 16 KiB; under `all`, every number is recorded and a holder block over 64 KiB
is a material stop with the measurement (R-S30-3's per-seat cap is then a separate PR).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-h-1 (AC-S30-16) | Nothing outside the pack and its `may_name` paths names MatrAIx; every test and world passes with the pack and its dependents removed | `check_optional_packs.py scan`; the `without` job |
| I-h-2 | The engine gains only generic features: no MatrAIx id, word or rule in `authoring/`, `systems/`, `tools/`, `cognition/`, `clients/`, `sdk/`, `scripts/` | `scan`; review of every Rust and Python diff |
| I-h-3 | `.upstream/` holds upstream's files byte for byte; the generated files are a pure function of them and the overlay | the import `--check`; digests |
| I-h-4 | None of the 50 excluded dimensions, and nothing of the Persona-1M coreset, is in the pack (h2: in the factor file) | the import's count and its refusal; X-h-1 |
| I-h-5 (I-S30-3) | A `private` dimension never appears on another observer's surface; a mapped view is never wider than its source | `MappingWidens`; the demo oracle's absence lists |
| I-h-6 (I-S30-2) | A mapped view and a derived `age_bracket` are never stored or stated | S30-c's derived-fact scan over the demo's save |
| I-h-7 (I-S30-10) | Every reference world reads, loads, runs and observes byte-identically; `backend: mineworld` worlds resolve exactly as before | the existing world tests; `attributes_builtin.rs`; `decisions` golden |
| I-h-8 (I-S30-6, h2) | Sampling under `log-linear` is a pure function of its inputs, integer, identical on three platforms | G-1; `clippy::float_arithmetic` |
| I-h-9 | The appearance group and quantities are MineWorld's under every backend (`ARC-125`) | S30-e's rule, unchanged; the demo's `catalogue` |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy `-D warnings` on `mineworld-attributes-model`, `mineworld-attributes`, `mineworld-cli`; ruff on `scripts/check_optional_packs.py` and the pack's `.tools/` | per commit; CI `fast` |
| Unit (Rust) | `keeps`, `mapping`, `map` and their refusals; upstream provenance shape; (h2) the factor format, topological check, fixed-point draw, exp table | `authoring/attributes/src/{backend, catalogue, derive}/tests.rs`; (h2) `src/sample/factors/tests.rs` |
| Unit (Python) | the import's slugs, exclusions, refusals, determinism; the optional-pack checker's self-test; (h2) the reference | `.tools/test_import.py`; `check_optional_packs.py --self-test` |
| Crate / CLI (Rust) | the generic features end to end on the toy backend fixture (the second world) | `tools/cli/tests/attributes_builtin.rs` (new cases) |
| Binary checks (declared) | every MatrAIx-specific acceptance: the import check, `attributes check`/`resolve`/`show` on the pack's fixtures and the demo, the demo's surfaces | `.optional.yaml` `checks`, run by `check_optional_packs.py run` in `core` |
| Integration (real binary) | the demo's server and 2D surfaces against its oracle | S30-f's two tests with the demo's pair |
| Guide | the comparison page on three platforms | `tools/cli/tests/guide.rs` (unchanged) |
| Regression | I-h-7 | existing tests unchanged; the nightly parity |
| Gate 1 (real LLM) | NOT REQUIRED: prompts are deterministic text compared byte for byte; no LLM-facing rule changes | — |
| Gate 2 (real lifecycle) | h2's prototype gate (G-1 … G-3) is the real-lifecycle evidence for the mode: real CI runners on three platforms, the real binary | §5.11 |

---

## 8. Acceptance criteria (fixed before measuring)

Each names a second world or a generated variant (charter).

**S30-h1**

| ID | Criterion | Second world |
| --- | --- | --- |
| AC-h1-1 | `import.py --check` regenerates the pack byte for byte from `.upstream/` and the overlay; every digest matches upstream's; 1,240 dimensions; the counts 28 and 22 dropped printed | a planted copy with one byte changed in `.upstream/dimensions.json` fails `ProvenanceMismatch` |
| AC-h1-2 | A fixture world `.checks/core-100-world` (6 people, `backend: matraix`, profile `matraix-world-priors`) passes `attributes check`; `resolve --json` lists 98 MatrAIx values per person plus the `core` appearance and `keeps` | the same world with `backend: mineworld` resolves the `core` catalogue only |
| AC-h1-3 (AC-S30-1) | Under mask `all` (`.checks/all-world`), each person carries ≥ 1,000 attributes, all in `attributes show` for the holder | the `core-100` world carries ≤ 100 + core |
| AC-h1-4 (mapping) | Under MatrAIx, `core.chronotype`, `core.caffeine`, `core.diet` and the rest of Appendix B are present as `derived` views, never stated; a person with `lstyle_sleep_schedule: early-bird` reads `core.chronotype: early-bird` | the toy backend's mapping, in Rust |
| AC-h1-5 (keeps) | Under MatrAIx, `core.natal-sex`, `core.ancestry`, `core.age-band` exist and are sampled from the profile | the toy backend keeps `core.natal-sex` |
| AC-h1-6 (refusals) | each by name: a mask naming an unknown id (`MaskNamesUnknownAttribute`); a mapping missing a source value (`MappingUncovered`), naming an unknown target value (`MappingValueUnknown`), widening (`MappingWidens`), targeting an owner's attribute (`MappingTargetOwned`); `keeps` naming an appearance id; an excluded dimension named by the overlay (`ExcludedDimensionNamed`) | all on the toy backend except the last |
| AC-h1-7 (X-S30-18) | `check_optional_packs.py scan` passes on the tree; `matraix` planted in `authoring/attributes/src/lib.rs` fails naming file and line | the self-test's planted tree |
| AC-h1-8 (X-S30-19) | The `without` job is green on h1's final head; a planted core test that loads `attributes-matraix` turns it red | the planted test |
| AC-h1-9 | Sizes recorded (§5.16); the `core-100` thresholds hold | both fixtures |
| AC-h1-10 | Every reference world, the `decisions` golden and S30-e/S30-f's tests are unchanged | — |

**S30-h2** (only if the gate passes; otherwise AC-h2-0 alone)

| ID | Criterion | Second world |
| --- | --- | --- |
| AC-h2-0 | The prototype gate G-1 … G-3 is run and classified PASS / FAIL / INCONCLUSIVE with evidence; on FAIL the drop report is merged and nothing else | — |
| AC-h2-1 | G-1, G-2, G-3 PASS | a 3-node toy factor model in the toy backend gives hand-computed shares within 1 ppm of the micro-unit expectation |
| AC-h2-2 | An authored value of a node is never re-drawn, and its children are conditioned on it: a fixture person with `big5_gregariousness: very-high` keeps it under every seed | the toy factor model's evidence test |
| AC-h2-3 | A factor file naming an excluded dimension, a cycle, a value outside a domain, or a missing node fails by name | the toy factor model |
| AC-h2-4 | No reference world can name `sampling: log-linear` (its backend lacks it: `UnsupportedSampling`) | `social-cafe` with the line planted |

**S30-h3**

| ID | Criterion | Second world |
| --- | --- | --- |
| AC-h3-1 (checkpoint, AC-S30-17) | In the demo, the server's observation, inspect and own-sheet texts and the 2D client's menu header, subject panel and own sheet equal the demo's oracle, including at least six MatrAIx texts; the oracle's private ids are on none of them | S30-f's own fixture still passes the same two tests |
| AC-h3-2 (briefs) | `resolve --json --facet casting` lists the `core` appearance and `keeps` values for every person, and `resolve --json` lists their MatrAIx values with origins | the `mineworld` variant |
| AC-h3-3 (AC-S30-15) | Switching to `backend: mineworld` changes only the catalogue and the attribute values: every other byte of `catalogue --json`'s non-attribute sections and the 30-day run's non-attribute facts is equal | the variant world |
| AC-h3-4 | The same decision list (`core.chronotype`, `core.caffeine`) runs under both backends; its explanation table shows the `Attribute` consideration in both | the variant world |
| AC-h3-5 | The comparison page and its mirror run green on Linux, macOS and Windows | — |
| AC-h3-6 | The demo's baselines section is recorded by the nightly parity with a reason line | — |

---

## 9. Commit plans

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs
the work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from the part's own worktree. Never `cd` elsewhere.

1. Re-read this commit's section and every file before editing it.
2. Make the numbered steps in order; nothing of a later commit.
3. Validate one `--test`/`--lib` or one Python file at a time; record command, count, wall time and PASS / FAIL /
   INCONCLUSIVE. `cargo clippy -p <touched crates> --all-targets -- -D warnings`; `cargo fmt --all --check`. Heavy
   jobs through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`. **R-LOAD-1:** never `cargo test
   --workspace`.
4. Mutations: copy the file, plant, run the named test red (record its name), restore, `cmp`, run green.
5. Answer the commit's review questions in §16 in one or two sentences each.
6. `git status --short`: every changed path is in the commit's **Files**; otherwise revert or stop and record why.
7. Update §16 and the part's handoff in the same commit; stage by path; commit with `git commit -F <file>` and the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`, no `curl`.

### 9.1 S30-h1 — the backend pack

#### h1-C0 — Base and ledger

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30h1.md` (new).
- Steps: record the base; re-read A-h-1 … A-h-16 and record every moved line; `git ls-remote` the upstream and record
  whether `main` is still at the pinned commit (if not: record it, keep the pin, no upgrade in this PR); confirm §15's
  rulings and apply them to §§5, 8.
- [ ] Implementation. [ ] Validation: `python3 scripts/check_doc_headings.py`, `check_decision_ids.py`.
  [ ] Review: every ruling applied before C1.
- **Commit message:** `docs(plan): S30-h1 — open the ledger at the base`

#### h1-C1 — The engine: `keeps`, `mapping`, `map`, upstream provenance

- **Files:** `authoring/attributes/src/{backend.rs, catalogue.rs, declaration.rs, derive.rs, mask.rs}` and their
  `tests.rs`; `authoring/attributes/src/backend/tests.rs` (line 19's sample); `systems/attributes/src/data.rs`
  (`mappings/` owned; the `Mapping` data file); `tools/cli/tests/fixtures/attributes-backends/packs/toy-backend/{backends/toy.yaml,
  mappings/toy.yaml (new), attributes/toy.yaml}`; `tools/cli/tests/attributes_builtin.rs` (new cases); `mineworld
  attributes backends`' output (`tools/cli/src/…`, the file C0 locates).
- Steps: §5.7 (format, `Derive::Map`, application in `compose`, every refusal); §5.8; §5.9's provenance shape; the toy
  backend gains a mapping to `core.chronotype` and `keeps: [core.natal-sex]`.
- Tests: each refusal of §5.7's table and §5.8 on the toy backend, by name; a mapped view reads through a `Handle`, is
  absent from `traits-stated`, and appears in the holder's block with the target's own phrase; a masked-out source
  leaves the target absent and `backends` lists it; under `backend: mineworld` the toy fixture resolves byte-identically
  to the base (`attributes-backends` golden unchanged).
- Validation: `cargo test -p mineworld-attributes-model --lib`; `-p mineworld-cli --test attributes_builtin`; `-p
  mineworld-cli --test attributes` (S30-c's); `-p mineworld-cli --test attributes_resolve`. Mutations **M-h-1** (`map`
  stored as a fact) → the derived-fact test red; **M-h-2** (`MappingWidens` check removed) → its refusal test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no MatrAIx word or id; a mapping never targets an owner's attribute;
  `keeps` cannot keep appearance twice; the default backend's path is untouched.
- **Commit message:** `feat(attributes-model): a backend may keep built-ins and map its values onto them (ARC-126, S30-h1)`

#### h1-C2 — The optional-pack check

- **Files:** `scripts/check_optional_packs.py` (new); `scripts/ci_layer.py` (`fast`: `scan`, `--self-test`; `core`:
  `run --binary target/debug/mineworld`); `.github/workflows/nightly.yml` (the `without` job on Linux);
  `scripts/ci_changes.py` (classify `.optional.yaml` and `.tools/` changes as code, if its table needs it).
- Steps: §5.10 steps 1–4; nothing names a pack.
- Validation: `python3 scripts/check_optional_packs.py --self-test`; `scan` on the tree (no optional pack yet: passes,
  prints "0 optional packs"); `python3 scripts/ci_changes.py --self-test`; ruff.
- [ ] Implementation. [ ] Validation. [ ] Review: the script fails closed (a malformed `.optional.yaml`, a missing
  path, an unreadable file all fail); it runs only commands a reviewed `.optional.yaml` lists.
- **Commit message:** `ci: a generic check that optional packs stay optional (X-S30-18, X-S30-19, S30-h1)`

#### h1-C3 — The verbatim import and the tool

- **Files:** `entities/attributes-matraix/{pack.yaml, LICENSE, NOTICE, CITATION.cff, README.md, .optional.yaml,
  .upstream/**, .tools/import.py, .tools/test_import.py, .tools/overlay.yaml (defaults and the category table only),
  backends/matraix.yaml, attributes/*.yaml}`.
- Steps: copy the upstream files from a scratch clone at the pinned commit (`git clone --filter=blob:none`, sparse
  checkout; never run upstream code); verify digests; write §5.2 steps 1–4 and 6; generate.
- Validation: `python3 -I -m unittest` over `.tools`; `python3 -I .tools/import.py --check`; `target/debug/mineworld
  packs list --packs entities` lists the pack; `scan` passes. Mutations **M-h-3** (one excluded dimension let through)
  → the count test red; **M-h-4** (a value slug collision planted) → `ValueSlugCollision`.
- [ ] Implementation. [ ] Validation. [ ] Review: `.upstream/` is byte-identical to upstream (digests); no
  `default:`; labels and phrases verbatim; the 50 excluded ids are absent from every generated file.
- **Commit message:** `feat(attributes-matraix): the MatrAIx schema imported unchanged, as an optional backend pack (DEP-66, S30-h1)`

#### h1-C4 — The overlay, the mask, the mapping, the priors

- **Files:** `entities/attributes-matraix/.tools/overlay.yaml`, the generated `attributes/*.yaml`,
  `masks/matraix-core-100.yaml`, `mappings/matraix.yaml`, `populations/matraix-world-priors.yaml`,
  `backends/matraix.yaml`; `.tools/test_import.py` (new cases).
- Steps: §§5.4–5.6 and Appendices A, B; §5.12's priors.
- Validation: `import.py --check`; `target/debug/mineworld attributes check` on `.checks/core-100-world` (written in
  C5; here on a temporary copy). Review the per-group counts (§5.3) into §16.
- [ ] Implementation. [ ] Validation. [ ] Review: every private dimension of §5.4 has no consumer facet; every mask
  entry's `why` names a reader that ships; every mapping row was checked against both domains.
- **Commit message:** `feat(attributes-matraix): overlay, the default mask of 98, the mapping onto built-ins (S30-h1)`

#### h1-C5 — The pack's fixtures and declared checks; sizes

- **Files:** `entities/attributes-matraix/.checks/{core-100-world/**, all-world/**, *.expected}`;
  `.optional.yaml` (`checks`).
- Steps: AC-h1-2 … AC-h1-6's binary checks as declared checks, each with a hand-written expected file; §5.16's
  measurements into §16.
- Validation: `python3 scripts/check_optional_packs.py run --binary target/debug/mineworld`.
- [ ] Implementation. [ ] Validation. [ ] Review: no expected file was produced by the code under test without a
  hand check of every line it asserts.
- **Commit message:** `test(attributes-matraix): declared binary checks for the import, the masks and the mapping (S30-h1)`

#### h1-C6 — Specification sync and `DEP-69`

- **Files:** `docs/MODULE_SPEC.md` (§2's layout; §4.1 "Backends and masks": `keeps`, `mapping`, `map`, provenance; the
  data-directory table), `docs/decisions/DEP-69.md` (new), dated notes in `docs/decisions/{ARC-122, ARC-126,
  DEP-66}.md`; `python3 scripts/check_decision_ids.py --write-index`.
- Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Implementation. [ ] Validation. [ ] Review: no specification and code disagree (`CLAUDE.md` §2.1 rule 4); old
  text kept visible.
- **Commit message:** `docs(spec): backends keep and map built-ins; the MatrAIx import as built (DEP-69, S30-h1)`

#### h1-C7 — Close-out

- **Files:** this file (§16), the handoff.
- Steps: push; `gh pr create`; CI `fast` and `core` on the exact head; one `nightly.yml` dispatch for the `without`
  job (AC-h1-8).
- [ ] Implementation. [ ] Validation: CI and the nightly verdict. [ ] Review: deviations recorded.
- **Then:** MERGE-READY, reported to the coordinator with the PR number and head SHA.

### 9.2 S30-h2 — the `log-linear` mode

#### h2-C0 — Base, ledger, and the upstream sampler read

- Read (never run) upstream's sampler source for how full CPTs combine with pairwise edges and how soft masks apply;
  record the rule in §16 with file and line; read the terms of GSS, Pew, WVS and the Stack Overflow survey and record
  them for `NOTICE` and `DEP-69`.
- [ ] Implementation. [ ] Validation: doc checks. [ ] Review: §5.11's formula matches upstream's stated one.
- **Commit message:** `docs(plan): S30-h2 — open the ledger; the combination rule read from upstream`

#### h2-C1 — The prototype and the gate

- **Files:** `authoring/attributes/src/sample/{factors.rs (new), exp_table.rs (new)}`, `factors/tests.rs`;
  `authoring/attributes/tools/exp_table.py`; `systems/attributes/src/data.rs` (`factors/`);
  `authoring/attributes/src/{backend.rs (SAMPLING), config.rs (sampling)}`; `entities/attributes-matraix/{.tools/import.py
  (--graph), .tools/reference.py, factors/matraix.yaml, backends/matraix.yaml}`; the toy factor model in the toy
  backend fixture; `.checks/loglinear-world/**`.
- Steps: §5.11; then run G-1 … G-3 (G-1 in CI on three platforms) and classify.
- Validation: `cargo test -p mineworld-attributes-model --lib`; the toy model's hand-computed test; the gate.
  Mutation **M-h-5** (an `f64` in the draw) → clippy refuses; **M-h-6** (a skipped evidence node) → AC-h2-2 red.
- [ ] Implementation. [ ] Validation: G-1, G-2, G-3 classified. [ ] Review: integer only; internal nodes never stored.
- **Commit message:** `feat(attributes-model): an optional log-linear sampling mode, fixed point (DEP-67, Q-S30-9, S30-h2)`
  — or, on a failed gate, the code is not committed and C2 is the drop report.

#### h2-C2 — Spec sync, or the drop report

- **Files:** `docs/MODULE_SPEC.md` §4.1; `docs/decisions/{DEP-67, DEP-69}.md` notes; `NOTICE` (calibration sources).
- [ ] Implementation. [ ] Validation: `ci_layer.py docs`. [ ] Review: the report states which gate failed, with numbers.
- **Commit message:** `docs(spec): the log-linear mode as built (S30-h2)` or `docs(plan): S30-h2 — log-linear dropped: <gate>`

#### h2-C3 — Close-out

- As h1-C7. **Then:** MERGE-READY.

### 9.3 S30-h3 — the demo and the guide

#### h3-C0 — Base and ledger

- Record whether S30-h2 merged with the mode or a drop report; whether S30-f's D-impl-7 items have landed.
- [ ] Implementation. [ ] Validation: doc checks. [ ] Review.
- **Commit message:** `docs(plan): S30-h3 — open the ledger at the base`

#### h3-C1 — The demo world

- **Files:** `worlds/matraix-demo/**` (world, places, people, decision list, `configure/attributes.yaml`,
  `populations/matraix-demo.yaml`); `entities/attributes-matraix/.optional.yaml` (`dependents`, `may_name`).
- Validation: `target/debug/mineworld validate worlds/matraix-demo --packs entities`; `attributes check`;
  `resolve --json`; the generic world tests one at a time (`packs`, `furnishings`, `item_names`, `station_visit`,
  `client_rules`).
- [ ] Implementation. [ ] Validation. [ ] Review: the demo names MatrAIx only through `backend:` and `requires:`.
- **Commit message:** `feat(worlds): matraix-demo, people from the optional MatrAIx backend (S30-h3)`

#### h3-C2 — The oracle and the surfaces

- **Files:** `worlds/matraix-demo/.checks/expected.json` (hand-written); `tools/cli/tests/{attributes_consumers.rs,
  client_2d_attributes.rs}` (the world-and-oracle pair, §5.13); `.optional.yaml` (`checks`: both tests with the pair;
  the `mineworld` variant; the decision-list check).
- Validation: both tests on their own fixture (unchanged) and on the demo's pair; the variant check.
- [ ] Implementation. [ ] Validation. [ ] Review: the oracle was written by hand; the tests name no pack.
- **Commit message:** `test(attributes): the consumer surfaces take a world and an oracle; the demo's checks (S30-h3)`

#### h3-C3 — The guide page and the README line

- **Files:** `docs/guide/compare/{person-backend.md, person-backend.zh-CN.md, README.md, README.zh-CN.md}`;
  `README.md`, `README.zh-CN.md` (the attribution line); `.optional.yaml` (`may_name`).
- Validation: `cargo test -p mineworld-cli --test guide`; `scan`.
- [ ] Implementation. [ ] Validation. [ ] Review: every step reads "Do this → You should see this"; the mirror adds no
  requirement.
- **Commit message:** `docs(guide): who the people are made from — switching the person backend and back (S30-h3)`

#### h3-C4 — Close-out

- Push; PR; CI; one nightly dispatch (baselines section recorded with its reason line, AC-h3-6; the `without` job).
- **Then:** MERGE-READY.

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time, heavy jobs through
`with-slot.sh`; `cargo clippy` on touched crates; ruff on touched Python. Never `cargo test --workspace`, never the
nightly locally. CI on the exact head is the gate; one nightly dispatch per part is the only long run.

---

## 11. Requirements this PR places on other PRs

- **R-S30k-h-1 (S30-k).** Convert MatrAIx's six label packs (`.upstream/labels/`, keyed by verbatim English values)
  into S30-k's `labels/<locale>.yaml` through the pack's import tool, mapping each value through §5.3's slug; the
  zh-Hans demo panel of step-30 §6.7 is checked there (Q-S30h-7).
- **R-S30f-h-1 (S30-f's milestone batch, D-impl-7).** The 3D `--probe=attributes` mode and the two Python
  real-server surface tests take the same `MINEWORLD_ATTRIBUTES_WORLD` / `MINEWORLD_ATTRIBUTES_ORACLE` pair, so the
  demo's MCP, cognition and 3D surfaces are checked by adding one line each to the pack's declared checks.
- **R-S31-h-1 (S31).** `economy.wealth-band` is never a mapping target (Q-S30h-3); a backend's wealth-like dimension
  stays the backend's plain trait.
- **R-any-h-1 (every future optional pack).** It ships a `.optional.yaml`; the generic check then covers it.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §6.6 | dated notes: values as slugs with verbatim words, categories as facets (Q-S30h-2); `socioeconomic_band` owned by `attributes`, no `economy.wealth-band` mapping (Q-S30h-3); `keeps` (Q-S30h-4); the privacy list of §5.4; the generic isolation check and its real allow-list (F-S30h-6); `tools/` is `.tools/` (R-S30h-a-1); the graph not committed (Q-S30h-9) |
| `step-30-person-attributes.md` §6.7 | the zh-Hans panel moves to S30-k (Q-S30h-7); the guide section is a comparison page |
| `step-30-person-attributes.md` §13.1, the S30-h row | split into S30-h1, S30-h2, S30-h3 (Q-S30h-1) |
| `step-30-person-attributes.md` §5.1 | "or a group the declaring pack declares" is not built; backends use facets (F-S30h-2) |
| `overall.md` | S30-h as three PRs; their states |

---

## 13. Execution contract (one per part; the fields that differ are marked)

```text
PROJECT / PR:            MineWorld mvp0, S30-h1 | S30-h2 | S30-h3 — the optional MatrAIx backend and demo
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30h-matraix-backend.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§5, 6.3–6.7, 8, 12, 13, 16); pr-s30c/d/e/f designs;
                         docs/MODULE_SPEC.md §§2, 4.1; docs/decisions/ARC-122, ARC-125, ARC-126, DEP-66, DEP-67,
                         DEP-68; docs/GUIDE_FORMAT.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         .structured-coding/standards.md; LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30h1 | impl-s30h2 | impl-s30h3, each its own,
                         held by one session
BRANCH:                  mvp0/pr-s30h1 | mvp0/pr-s30h2 | mvp0/pr-s30h3, each from origin/main
START CONDITION:         h1: this design frozen AND §15 ruled AND the coordinator's dispatch;
                         h2, h3: h1 merged, then the dispatch
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1's row for the part, as §15's rulings shape it
FROZEN INVARIANTS:       step-30 I-S30-2, I-S30-3, I-S30-6, I-S30-7, I-S30-8, I-S30-10; §6 I-h-1 … I-h-9; §8 as written
SEQUENCE:                §9.1 | §9.2 | §9.3
IMPLEMENTER:             h1, h2: Opus-class session; h3: a Haiku-class session from §9.3, reviewed by an
                         Opus-class reviewer
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *;
                         python3 -I <the pack's .tools scripts>; python3 scripts/check_optional_packs.py, ci_layer.py
                         docs, check_doc_headings.py, check_decision_ids.py, ci_changes.py --self-test; uv run ruff;
                         git (including a read-only partial clone of the upstream at the pinned commit into a
                         scratch directory, deleted after); gh (pr create/view; workflow run nightly.yml once per
                         part; run download; never merge); cp; cmp; mkdir -p; sed -n; grep
NEVER:                   running any MatrAIx code; downloading the Persona-1M coreset; python3 -c; sed -i; heredoc
                         writes; curl; git add -A; cargo test --workspace; an edit under kernel/, contracts/, server/,
                         or any systems/<pack> other than attributes; a reference-world edit; a model call; money
MATERIAL STOPS:          a reference world's bytes moving; a needed change to an existing System other than
                         attributes; the holder block over 64 KiB under mask all (§5.16); a needed widening of
                         ValueSlug or GROUPS; upstream main moved and the pinned files no longer fetchable; (h2) a
                         gate FAIL (then the drop report is the endpoint, not a stop)
PLATFORMS:               Linux, macOS, Windows (CI fast and core; h2's G-1 on all three)
VALIDATION BUDGET:       crate and Python tests unrestricted one file at a time; one nightly dispatch per part;
                         h2's gate: three CI runs at most. Gate 1: NOT REQUIRED. Gate 2: h2's prototype gate only
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30h1.md | handoff-s30h2.md | handoff-s30h3.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once frozen (source: the operator's freeze)
                         semantic commits: authorized (working rules §14)
                         branch push: authorized (working rules §21)
                         PR creation / update: authorized (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: MERGE-READY reported to the coordinator, who merges once `fast` is green
                         (LANE_LEAD_CHARTER.md, speed first, 2026-10-18)
                         merge: never by the implementation session
POST-MERGE SYNC OWNER:   primary: step-30, overall.md; implementation session: this ledger and the merge identity
STOP CONDITION:          the part MERGE-READY — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-h-1 | The generated catalogue (≈ 15,000 lines) slows every world that requires the pack | only the demo and the pack's fixtures require it; §5.16 measures load time |
| R-h-2 | Upstream's phrases read oddly in English sentences ("scores Very high on Friendliness") | verbatim is the ruling (RR-S30-4); words are presentation, and a world may override them (§7.2) |
| R-h-3 | A sensitive dimension leaks through a phrase or a mapping | §5.4's private list carries no consumer facet; `MappingWidens`; the demo oracle's absence list |
| R-h-4 | The factor model's derived parameters carry their sources' terms (F-S30h-10) | Q-S30h-6; `NOTICE` credits every source; the mode is optional and demo-only; a "no" drops it |
| R-h-5 | The float reference shares a mistake with the Rust engine | it is written from the file's stated formula by a different hand; the toy model's hand-computed shares check both |
| R-h-6 | Upstream changes the schema | the pin and digests; an upgrade is a reviewed PR with the tool's diff report; M-h-1 shows no change so far |
| R-h-7 | The guide index's link to the comparison page breaks the `without` job | h3-C0 checks whether index links are checked; if they are, the index row is listed in `dependents` handling as a bounded fix |
| R-h-8 | The demo world joins every generic world test and the nightly parity, adding cost | one small world; its 300-day run is the cross-platform proof of the demo (D-h-9) |
| R-h-9 | The `without` job's full `core` run is long | nightly only, plus once per part (Q-S30h-8) |

---

## 15. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30h-1 [split]** | The S30-h row bundles the import, a new sampling mode and a demo with a guide page. Split it? | **Yes: S30-h1 (the pack and the generic engine features, CI isolation), S30-h2 (`log-linear`, with its gate and drop rule), S30-h3 (demo world and guide page).** h2 and h3 run in parallel after h1. One design, frozen once; three PRs, three worktrees, three fresh sessions |
| **Q-S30h-2 [what "imported unchanged" means in the typed model]** | 6,321 of MatrAIx's 6,347 values are not value slugs ("Early bird"), and its 43 categories cannot be groups (the six groups are fixed). Widen the model, or represent them? | **Represent them: each value gets a generated slug (`early-bird`) and keeps its verbatim text as its words; each category becomes a facet `category.<slug>` and a fixed overlay table places it in one of the five groups.** The upstream file is committed byte for byte; every rendered phrase is upstream's text. Widening `ValueSlug` would loosen typing and saves for every pack |
| **Q-S30h-3 [`socioeconomic_band`; Q-S30-6's overlay line]** | The step makes `economy` the owner of `socioeconomic_band` and maps `economy.wealth-band` from it. `economy` would then have to name a MatrAIx id, and the view would compete with `economy`'s own projection. | **`socioeconomic_band` is a private plain trait owned by `attributes`; no mapping targets an owner's attribute.** Isolation and single ownership hold; S31's wealth stays `economy`'s |
| **Q-S30h-4 [`keeps`; the appearance group's parents]** | Under a foreign backend, `core.natal-sex`, `core.ancestry` and `core.age-band` are removed, but casting conditions the appearance group on them. | **Add `keeps:` to the backend contract; MatrAIx keeps those three.** `demo_ethnicity_broad` leaves the default mask (it stays in `all`) so the world has one ethnicity attribute in play |
| **Q-S30h-5 [privacy list]** | Q-S30-8 names ethnicity, religion, orientation, disability, citizenship, veteran status, income and health. MatrAIx also has gender identity, politics and migration dimensions. | **Make them `private` too** (§5.4), under Q-S30-8's rule. Only `gender_identity` and `socioeconomic_band` are in the default mask, both `ME` only |
| **Q-S30h-6 [`log-linear` details]** | (a) The graph has the 50 excluded dimensions as nodes (354 edges) and 18 latent nodes (394 edges). (b) The fidelity check asks for "their own sampler"; running their code is outside our rules. (c) The graph's parameters derive from sources with their own terms (ODbL for the Stack Overflow survey; terms of use for GSS, Pew, WVS). | **(a) Drop the excluded nodes and everything touching them; keep the latent nodes as internal draws, never stored. (b) The oracle is an independent float reference of the stated formula with common random numbers; their code is not run. (c) Accept for an optional, demo-only mode with every source credited in `NOTICE` and `DEP-69`;** if you say no to (c), h2 is dropped and the demo uses `profile` |
| **Q-S30h-7 [zh-Hans panel]** | Step-30 §6.7 promises a zh-Hans panel from MatrAIx's label pack, but the label format and locale selection are S30-k's, not built. | **Move the zh-Hans demo panel to S30-k (R-S30k-h-1).** S30-h1 commits the six label packs verbatim now, so S30-k only converts them |
| **Q-S30h-8 [CI cost]** | Where do the isolation checks run? | **The grep and its self-test in `fast` on every PR (seconds); the declared binary checks in `core`; the build-and-test-without-the-pack job nightly on Linux, plus once on each part's final head.** |
| **Q-S30h-9 [what is committed]** | Commit the 24.8 MB graph? | **No. Commit `dimensions.json`, the six label packs, `LICENSE` and `CITATION.cff` verbatim (≈ 2.7 MB) and h2's converted factor file (≤ 8 MB, G-2).** The graph's digest is pinned; a nightly job fetches it at the pinned commit with `git` and re-checks the conversion byte for byte |

Bounded decisions recorded here rather than asked: D-h-1 (`map`, F-S30h-4), D-h-2 (the generic optional-pack check,
F-S30h-6), D-h-3 (no upstream `defaultValue` carried, F-S30h-8), D-h-4 (§5.4's perceivable pair), D-h-5 (every
MatrAIx-specific check runs through the binary, F-S30h-7), D-h-6 (digests checked by the import, not at load, A-h-15),
D-h-7 (the pack id `attributes-matraix` is the namespace; `matraix` the backend id), D-h-8 (no root `CITATION.cff`;
the pack carries upstream's), D-h-9 (the demo stays at `worlds/matraix-demo` and joins parity, A-h-10).

---

## 16. Ledger (live during implementation)

### Commit ledger

| Part | Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- | --- |
| h1 | C0 … C7 | | [ ] | [ ] | [ ] |
| h2 | C0 … C3 | | [ ] | [ ] | [ ] |
| h3 | C0 … C4 | | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-h-1 … M-h-6 as §9 lists them)

### Deviations and findings

F-S30h-1 … F-S30h-13 (§3.3), recorded at design time.

---

## Appendix A — `matraix-core-100`: 98 dimensions, each with its reader

Reader codes as in step-30 §6.1. Every id is upstream's, in namespace `attributes-matraix`. A `→` names the built-in
the mapping derives from it (Appendix B).

| # | Id | Reader line (`why`) |
| --- | --- | --- |
| 1 | `big5_friendliness` | DM, ME: warmth to strangers |
| 2 | `big5_gregariousness` | DM, CV: joins groups, starts talk → `core.extraversion` |
| 3 | `big5_assertiveness` | CV: speaks up |
| 4 | `big5_activity_level` | DM: keeps busy |
| 5 | `big5_excitement_seeking` | DM: tries new places |
| 6 | `big5_cheerfulness` | CV, ME: a bright manner |
| 7 | `big5_trust` | DM, ME: trusts strangers → `core.trust-propensity` |
| 8 | `big5_altruism` | DM: helps others → `core.helpfulness` |
| 9 | `big5_cooperation` | DM: goes along with others |
| 10 | `big5_modesty` | CV: talks about themselves little |
| 11 | `big5_sympathy` | ME, CV: moved by others' trouble |
| 12 | `big5_anxiety` | ME: worries |
| 13 | `big5_anger` | CV: quick to irritation |
| 14 | `big5_orderliness` | DM: keeps the agenda |
| 15 | `big5_dutifulness` | DM: shows up to work |
| 16 | `big5_self_discipline` | DM: finishes what they start |
| 17 | `big5_cautiousness` | DM: thinks before acting |
| 18 | `big5_imagination` | ME: daydreams |
| 19 | `big5_artistic_interest` | DM: galleries, painting |
| 20 | `big5_adventurousness` | DM: explores → `core.curiosity` |
| 21 | `big5_intellect` | CV: likes ideas |
| 22 | `big5_self_consciousness` | CV: shy in company |
| 23 | `cog_verbosity` | CV: how long they talk |
| 24 | `cog_formality` | CV: register → `core.formality` |
| 25 | `cog_directness` | CV: blunt or indirect |
| 26 | `cog_humor` | CV: humour style |
| 27 | `cog_politeness` | CV: polite or brusque |
| 28 | `cog_patience` | CV, DM: waits or argues → `core.patience` |
| 29 | `cog_optimism` | ME, CV: outlook → `core.optimism` |
| 30 | `cog_curiosity` | DM: asks, explores |
| 31 | `cog_storytelling` | CV: tells stories |
| 32 | `cog_emotional_expressiveness` | CV: shows feelings |
| 33 | `cog_conflict_approach` | CV, DM: confronts or avoids |
| 34 | `cog_decision_speed` | DM: quick or slow choices |
| 35 | `cog_question_asking` | CV: asks questions |
| 36 | `register` | CV: speaks formally or in dialect |
| 37 | `lstyle_sleep_schedule` | DM: when they rise and go home → `core.chronotype` |
| 38 | `lstyle_punctuality` | DM: on time or late → `core.punctuality` |
| 39 | `lstyle_social_battery` | DM: joins or leaves company → `core.sociability` |
| 40 | `lstyle_tidiness` | ME: tidy or cluttered |
| 41 | `lstyle_frugality` | DM, S31: what they spend → `core.spending-style` |
| 42 | `lstyle_giving` | DM: treats and gifts → `core.generosity` |
| 43 | `lstyle_news_freq` | S31: default notice subscriptions → `core.news-interest` |
| 44 | `lstyle_reading_freq` | DM, S31: the bench reader → `core.reading-habit` |
| 45 | `lstyle_exercise_freq` | DM: the runner runs → `core.exercise-habit` |
| 46 | `lstyle_diet_type` | DM: what they buy and eat → `core.diet` |
| 47 | `lstyle_alcohol_use` | DM: drinks → `core.alcohol` |
| 48 | `lstyle_smoking` | OV, ME: a smoker → `core.smoking` |
| 49 | `lstyle_caffeine` | DM: café visits → `core.caffeine` |
| 50 | `lstyle_coffee_ritual` | DM: café regular or home brew |
| 51 | `lstyle_cooking_freq` | DM: cooks at home |
| 52 | `lstyle_pet_ownership` | ME: has a dog → `core.pet` |
| 53 | `lstyle_household_size` | ME, DM: who waits at home |
| 54 | `lstyle_work_schedule` | DM: shift or nine-to-five |
| 55 | `lstyle_commute_mode` | DM, ME: walks or drives |
| 56 | `lstyle_shopping_style` | DM, S31: bargain hunter or impulse buyer |
| 57 | `lstyle_fashion_sense` | ME: cares how they dress |
| 58 | `lstyle_morning_routine` | DM: slow or rushed mornings |
| 59 | `lstyle_volunteering` | DM: helps out |
| 60 | `lstyle_music_listening` | ME: music all day |
| 61 | `lstyle_hobby_intensity` | DM: time on hobbies |
| 62 | `lstyle_planning_horizon` | DM: plans ahead or not |
| 63 | `lstyle_payment_pref` | S31: cash or card |
| 64 | `lstyle_vacation_style` | ME: what they talk about |
| 65 | `pref_indoor_vs_outdoor` | DM: park or indoors → `core.outdoor-preference` |
| 66 | `pref_big_group_vs_one_on_one` | DM: groups or pairs |
| 67 | `pref_routine_vs_variety` | DM: keeps the routine or drifts |
| 68 | `habit_morning_walk` | DM: the morning walker |
| 69 | `habit_naps` | DM: an afternoon rest |
| 70 | `habit_journaling` | ME |
| 71 | `skill_cooking` | DM, ME: the cook → `core.skill.cooking` |
| 72 | `skill_baking` | DM, ME: the baker → `core.skill.baking` |
| 73 | `skill_selling` | DM, ME: the shopkeeper → `core.skill.selling` |
| 74 | `skill_gardening` | DM, ME → `core.skill.gardening` |
| 75 | `skill_painting` | DM, ME → `core.skill.painting` |
| 76 | `skill_drawing` | ME |
| 77 | `skill_photography` | DM: takes pictures |
| 78 | `skill_diy_repair` | DM: fixes things |
| 79 | `skill_negotiation` | CV, S31: haggles |
| 80 | `skill_storytelling` | CV |
| 81 | `skill_active_listening` | CV: listens well |
| 82 | `skill_driving` | DM |
| 83 | `skill_budgeting` | S31: keeps to a budget |
| 84 | `hob_birdwatching` | DM: the park |
| 85 | `hob_bread_baking` | DM |
| 86 | `hob_vegetable_gardening` | DM |
| 87 | `hob_karaoke` | DM: evenings out |
| 88 | `hob_kayaking` | DM: the lake |
| 89 | `hob_woodworking` | DM: the carpenter |
| 90 | `topic_fishing` | DM: the lake's fishermen |
| 91 | `topic_cooking` | ME: talks about food |
| 92 | `topic_sports` | ME: talks about games |
| 93 | `demo_marital_status` | ME → `core.marital-status` |
| 94 | `demo_children_count` | ME, DM: who waits at home |
| 95 | `demo_employment_status` | ME, DM: works, studies or is retired |
| 96 | `demo_housing_status` | ME |
| 97 | `gender_identity` | ME (private) |
| 98 | `socioeconomic_band` | ME, DM (private) |

C4 checks every id against the file, and that none is excluded. A row may be replaced, never added past 100.

## Appendix B — The mapping (`mappings/matraix.yaml`), first rows

Each row is checked in C4 against both domains; the rest of Appendix A's `→` targets are written the same way.

| Target | Source | Values (source slug → target slug) |
| --- | --- | --- |
| `core.chronotype` | `lstyle_sleep_schedule` | early-bird → early-bird; night-owl → night-owl; irregular, flexible, shift-based → neutral |
| `core.diet` | `lstyle_diet_type` | omnivore, flexitarian, vegetarian, vegan, pescatarian → the same; keto-low-carb → omnivore |
| `core.caffeine` | `lstyle_caffeine` | none → none; low → light; moderate → moderate; high → heavy |
| `core.extraversion` | `big5_gregariousness` | very-low … very-high → very-low, low, moderate, high, very-high |
| `core.alcohol` | `lstyle_alcohol_use` | never → none; rarely → occasional; socially → social; regularly, heavily → regular |
| `core.smoking` | `lstyle_smoking` | never → never; former → former; occasional → occasional; regular → daily |
| `core.punctuality` | `lstyle_punctuality` | always-early → early; on-time → on-time; usually-late → often-late; unpredictable → sometimes-late |
| `core.marital-status` | `demo_marital_status` | single → single; in-a-relationship, engaged, domestic-partnership → partnered; married, separated, divorced, widowed → the same |
| `core.skill.cooking` | `skill_cooking` | none → none; beginner → novice; intermediate → competent; advanced → skilled; master → expert |
| `core.news-interest` | `lstyle_news_freq` | avoids-news → very-low; rarely → low; weekly → moderate; daily → high; constant → very-high |
| `core.spending-style` | `lstyle_frugality` | frugal-saver → frugal; balanced → moderate; spender → free; splurger → lavish |

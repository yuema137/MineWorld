# PR S30-e — The curated built-in catalogue (93 `core` attributes), backends and masks, and the cited population profile

## DRAFT — not frozen

```text
Design revision:        revision 1 (2026-10-17), DRAFT
Author:                 S30-c / S30-d / S30-e PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s30cde, branch docs/pr-designs-s30cde,
                        planning base origin/main @ 38323d4c (#238)
Approved by / evidence: none yet. The freeze header is the operator's to approve and the primary
                        session's to write (CLAUDE.md §3.1)
Implementation base:    origin/main at C0; C0 records the commit and that S30-c and S30-d have merged
Start condition:        S30-c merged (the declaration format) AND S30-d merged (the profile format and
                        configure/attributes.yaml) — step-30 §13.3 — this design frozen, §15 ruled, the dispatch
Execution contract:     §13
Implementer:            Opus-class session (backends and masks, sources, review); C3's catalogue entry from
                        Appendix A may be delegated to a Haiku-class sub-agent and reviewed by the session
                        (step-30 §13.1, the S30-e row)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) §§6.1 (the table and the
note of 2026-10-15), 6.2, 6.3 (SD-S30-5), 6.4, 6.5 (`ARC-126`), 8.2, 13.1 (the S30-e row), 13.4 (AC-S30-2), 13.5
(X-S30-15, X-S30-16, X-S30-20), 16.1 (Q-S30-2, Q-S30-5, Q-S30-8), 16.2 (RR-S30-3, RR-S30-5) · **Specified by:**
`docs/MODULE_SPEC.md` §2 (the three Entity Packs), §4.1 "Backends and masks"; `docs/DECISIONS.md` `ARC-122` item 8,
`ARC-125`, `ARC-126`, `DEP-68` · **Requirements on this PR:** R-S30e-a-1 (93 `core` rows), R-S30e-a-2 (how
`backend.yaml` is carried; F-S30a-9), R-S30e-c-1, R-S30e-d-1 · **Builds on:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md), [`pr-s30d-sampler.md`](pr-s30d-sampler.md) ·
**Parent:** [`overall.md`](overall.md) · **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 5, 7, 11, 16); the
operator's standing preference for realistic defaults with citations; `docs/PACKAGE_FORMAT.md` §5.0 (`ARC-53`,
`ARC-55`); `docs/DECISIONS.md` `ARC-71`, `ARC-95`, `ARC-120` … `ARC-126`, `DEP-68`.

**Decision ids.** None new. Dated notes under `ARC-126` (§5.9) and `DEP-68` (the figures as read). `ARC-127` …
`ARC-129` and `DEP-69` unused.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-e row).** "`entities/person-attributes` (the 94 of §6.1, each with its reader line; masks
  `core` and `appearance`; `backend.yaml`); the backend and mask loader in the model crate (`ARC-126`);
  `entities/populations-us` (San Diego County v3 from A-10 with each figure re-read or kept `verify`; lakeside);
  `DEP-68` sources."
- **Integration checkpoint (same row).** "`attributes check` on both packs is clean except listed `verify` marks; the
  San Diego profile reproduces A-10's marginals."
- **Adversarial criteria (same row).** "X-S30-15 a profile figure without a source refused; X-S30-16 two catalogue
  entries sharing an id refused; X-S30-20 a 101st built-in, or one without a reader line, fails the catalogue check."
- **R-S30e-a-1.** "The catalogue declares 93 `core` rows (the 94 of step-30 §6.1, plus `core.natal-sex` per Q-S30a-1,
  minus the two owner-declared views); X-S30-20's cap applies to `core`."
- **R-S30e-a-2.** "Decide how the seam carries a backend pack's root file `backend.yaml` (F-S30a-9) — a bounded
  extension recorded under `ARC-121`, or the contract moved into an owned directory with `MODULE_SPEC` §§2, 4.1
  corrected by a dated note."
- **Rulings that bind.** At most 100 built-ins, each read by something the player notices (RR-S30-3); sensitive
  attributes `private` by default (Q-S30-8); appearance descriptors `perceivable` (Q-S30-1, `ARC-125`); populations in a
  world-required data pack (Q-S30-2); `core.natal-sex` added (Q-S30a-1); `economy.wealth-band` is `economy`'s (Q-S30-6)
  and `employment.occupation` comes in S30-f (Q-S30a-2), so neither is a `core` row; a release refuses `verify` on
  data figures, never on authored conventions (Q-S30-5); lakeside uses the San Diego County profile (operator ruling
  F-CAST-2, 2026-10-15, recorded in the interim `lakeside.yaml`).

---

## 1. Goal

1. Ship **`entities/person-attributes`**: the default backend `mineworld`, namespace `core`, **93 curated attributes**
   in six groups, each with the reader that makes it noticeable, five vocabularies, and the masks `core` and
   `appearance`.
2. Implement **backends and masks** (`ARC-126`) in the model crate and the pack: a world chooses `backend:` and `mask:` in
   `configure/attributes.yaml`; what a mask does not enable does not exist; the appearance group and quantities are
   always MineWorld's.
3. Ship **`entities/populations-us`** with **San Diego County v3**: every figure of the interim profile re-read from its
   source or kept `verify`, every convention marked `authored`, in S30-d's format.

---

## 2. Scope

### 2.1 In scope

1. `entities/person-attributes/` (new Entity Pack): `pack.yaml`, `README.md`, `NOTICE`, `backends/mineworld.yaml`,
   `attributes/{appearance, background, psychology, capability, behaviour, lifestyle}.yaml`,
   `vocabularies/{languages, foods, hobbies, music, sports}.yaml`, `masks/{core, appearance}.yaml` (Appendix A).
2. The backend and mask loader: `authoring/attributes/src/{backend.rs, mask.rs}`; `config.rs` gains `backend` and
   `mask`; `catalogue.rs`'s build composes by backend and mask; `uses.reads` `optional`; `MissingUnderBackend`;
   `attributes` owns `backends/` and `masks/`; `attributes-composed` gains `backend` and `mask`.
3. `mineworld attributes backends <world>`; `catalogue` and `check` show the backend, the mask and per-namespace counts.
4. `entities/populations-us/` (new Entity Pack): `pack.yaml`, `README.md`, `populations/san-diego-county.yaml`.
5. Tests: the catalogue check (X-S30-20, AC-S30-2), the profile check (X-S30-15, the A-10 reproduction), duplicate ids
   (X-S30-16), backends and masks with a toy backend fixture, the size re-measure (R-S30e-c-1).
6. Specification sync: `MODULE_SPEC` §§2, 4.1, 8.1; `DECISIONS.md` notes under `ARC-126` and `DEP-68`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `mappings/` (a backend's views of built-ins) and the verification of `provenance` digests | S30-h, its only user (Q-S30e-3) |
| The `log-linear` sampling mode | S30-h (Q-S30-9) |
| Any reference world requiring these packs; S26's switch | S30-g |
| `employment.occupation`; consumers of the catalogue | S30-f |
| `economy.wealth-band`, `notices.subscriptions` | S31 |
| Locale labels (`labels/`) | S30-k |

### 2.3 Dependencies

S30-c and S30-d merged. Read-only inputs: the interim profile and casting files on `origin/mvp0/s26-cast-director @
d81baeb3` (A-10, A-9), the cited sources online.

---

## 3. Audit (`origin/main @ 38323d4c`; `mvp0/pr-s30b @ 40af5a11`; `mvp0/s26-cast-director @ d81baeb3`; 2026-10-17)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-e-1 | `pack.yaml`'s fields are exactly `id, type, version, mineworld, license, authors, repository`; any other is refused | `docs/PACKAGE_FORMAT.md` §5.0; `entities/modern-goods/pack.yaml` | the backend contract cannot live in `pack.yaml` |
| A-e-2 | An Entity Pack "declares something" with an `items/*.yaml` **or** a `.yaml` in another non-dot directory (S30-b C3) | `mvp0/pr-s30b:packages/src/manifest.rs` | both new packs are data-only and pass |
| A-e-3 | The seam collects only owned directories; a pack root's **files** are neither refused nor collected (S30-b F-b-1 (b)) | `pr-s30b-authoring-seam.md` ledger F-b-1 | `backend.yaml` at the root would never reach `attributes` (R-S30e-a-2, Q-S30e-1) |
| A-e-4 | The default licence policy allows `MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0, Unlicense`; `CC-BY-4.0` is "a candidate … once attribution is handled; it is not in the default" | `docs/PACKAGE_FORMAT.md` §5.0 "Licence policy" (`ARC-55`) | a pack licensed `CC-BY-4.0` would be refused in every world (F-S30e-2, Q-S30e-2) |
| A-e-5 | `MODULE_SPEC` §2 specifies the layout `backend.yaml` at the root, `attributes/<group>.yaml`, `masks/core.yaml` ("every built-in"), `masks/appearance.yaml`, and `entities/populations-us/populations/san-diego-county.yaml` | `docs/MODULE_SPEC.md` §2 | followed, except `backend.yaml` (Q-S30e-1) |
| A-e-6 | The interim San Diego profile v2: sex `{female 0.4986, male 0.5014}`; age bands 20s … 80+ `{0.1864, 0.2050, 0.1716, 0.1506, 0.1417, 0.0973, 0.0474}`; ancestry `{hispanic_or_latino 0.3525, nh_white 0.4129, nh_asian 0.1258, nh_black 0.0429, nh_multiracial_or_other 0.0659}`; build by sex (NHANES national; county `verify`); stature by sex × band `[mean, P5, P95]` cm; apparent age uniform in band, 80+ as `[80, 88]`; MST ranges per ancestry; hair texture weights per ancestry; colour pools per ancestry with grey replacement `{50s 0.35, 60s 0.7, 70s 0.95, 80+ 1.0}`; hair-style pools by sex; facial hair by sex; face, eyes, nose, lips uniform | `origin/mvp0/s26-cast-director:presentation/mineworld-default/3D/characters/populations/san-diego-county.yaml` | Appendix B's figures and the reproduction oracle (§5.7) |
| A-e-7 | The interim sources: ACS 2024 1-year B01001/B03002 read through Census Reporter's API on 2026-10-15, 5-year `verify`; NHANES Fryar et al. 2021 Tables 9, 11, 14, 15 read; CHIS county build **not read** (`verify`) | same file, `sources:` | Appendix B's `sources:` and which figures stay `verify` |
| A-e-8 | Lakeside's profile is an alias of San Diego County by operator ruling F-CAST-2 | `origin/mvp0/s26-cast-director:…/populations/lakeside.yaml` | no second profile: S30-g's lakeside names `san-diego-county` (F-S30e-4) |
| A-e-9 | The interim hair-style pools are "all short or tied: §7.4, no loose long hair over the shoulders" — route D+'s limit | same file, `hair_style` comment | the limit is S26's `unsupported` (F-S30-5); the domain is wider, the profile's pools stay authored (F-S30e-5) |
| A-e-10 | S30-c accepts `core` only from the pack `person-attributes`; S30-d's profile format and `configure/attributes.yaml` (`profile, seed, salt, lock`) | `pr-s30c-attributes-pack.md` §5.3 item 2; `pr-s30d-sampler.md` §5.4 | this PR adds `backend`, `mask` and the `core` pack |

### 3.2 Findings

- **F-S30e-1. `backend.yaml` cannot be read where the specification puts it** (A-e-1, A-e-3). **Q-S30e-1** recommends
  moving the contract into an owned directory: `backends/<backend id>.yaml`, exactly one per backend pack.
- **F-S30e-2. The Monk Skin Tone scale and the licence policy** (A-e-4). `DEP-68` lists the MST scale as CC BY 4.0
  "attribution in the pack", but a pack whose licence includes `CC-BY-4.0` is refused by the default policy, and the
  operator's packs are CC0. **Q-S30e-2** recommends: the pack stays `CC0-1.0`; `core.skin-tone`'s domain is ten level
  ids `mst-1` … `mst-10` with MineWorld's own words (the interim's authored labels), no swatch colour or text of the
  scale's publication is copied, and the scale is credited in `NOTICE` and the README as the reference the levels
  follow.
- **F-S30e-3. "Reader line" needs a home.** The declaration format has no reader field. Decided: **the reader line is
  the `why` of the attribute's entry in `masks/core.yaml`**, one entry per `core` id, whose text begins with the
  reader codes of step-30 §6.1 (`C`, `OV`, `ME`, `CV`, `DM`, `S31`) and a colon. The mask format already refuses an
  entry without `why` (`MaskEntryUnjustified`); X-S30-20's check (§5.6) adds the per-id coverage, the code prefix and
  the cap.
- **F-S30e-4. Lakeside needs no file.** One profile, named by both worlds' configurations (S30-g); the ruling is cited
  in the profile's header.
- **F-S30e-5. Provider limits stay out.** `core.hair-style`'s domain includes long styles; San Diego v3's pools by sex
  equal the interim's (authored, `source: authored`, so the casting page shows them as conventions); route D+'s
  limit is S26's `unsupported` declaration.
- **F-S30e-6. `core.mass` has no cited conditional.** No source the interim read gives mass by sex and build class.
  Decided: v3 does not profile `core.mass` (it stays absent unless authored); the casting reader uses `build`. Recorded
  for a later profile version.
- **F-S30e-7. The census categories are US-specific.** `core.ancestry`'s values follow the profile's source (ACS
  B03002, the interim's pooling): `hispanic-or-latino`, `white`, `black`, `asian`, `multiracial-or-other` — the last
  four non-Hispanic, stated in the declaration's README line. A world elsewhere declares its own (`<world>.ancestry`,
  `MODULE_SPEC` §4.1 "Overrides"); a value appended later is a minor version.
- **F-S30e-8. Which backend when `backend:` is absent.** Decided: the backends the world's required packs provide —
  none: no backend (S30-c's catalogue: every collected declaration); exactly one: that one; several: refused
  `AmbiguousBackend` naming them. `backend: <id>` naming none is `UnknownBackend`. A world requiring
  `person-attributes` therefore uses `mineworld` with nothing written, as `ARC-126` says.

### 3.3 What does not exist

No `core` attribute, backend, mask, vocabulary, or cited profile in the tree. `entities/` holds only `modern-goods`.

---

## 4. Reuse, both ways

The catalogue's content is MineWorld's own (RR-S30-5: the MineWorld backend is the default; MatrAIx is optional and
S30-h's). Its figures are reused, not invented: ACS and NHANES (public domain, `DEP-68`), the MST scale as reference
(Q-S30e-2). No library is added. The mask and backend loader extends S30-c's catalogue build; no general rule engine is
introduced (`CLAUDE.md` §4 rule 11).

---

## 5. Design

### 5.1 The backend contract (`backends/<id>.yaml`, Q-S30e-1)

```yaml
# entities/person-attributes/backends/mineworld.yaml
format: 1
backend: mineworld                     # the backend id; a pack carries exactly one backends/ file
version: 1.0.0                         # equals pack.yaml's version (BackendMalformed otherwise)
license: CC0-1.0                       # equals pack.yaml's licence
attribution: "MineWorld's built-in person attributes"
catalogues: [appearance, background, psychology, capability, behaviour, lifestyle]   # this pack's attributes/ files
masks: { default: core, ships: [core, appearance] }
sampling: [profile]
provenance: { source: mineworld }      # an upstream import states source, version, commit and files (S30-h)
```

- Every field is validated; each failure is `BackendMalformed { pack, field }`. `catalogues` names files of the pack's
  own `attributes/`; a backend's attribute files not listed are refused (`BackendMalformed { field: catalogues }`).
- `sampling` is a subset of the engine's closed list; S30-e's list is `[profile]` (`log-linear` is S30-h's). A world's
  profile under a backend without `profile` is `UnsupportedSampling`.
- `mapping` is not a field in S30-e; its presence is `BackendMalformed { field: mapping, reason: "S30-h" }` (Q-S30e-3).
- `provenance.files` digests are S30-h's to verify (Q-S30e-3); S30-e accepts `{ source: mineworld }` and refuses
  anything else (`BackendMalformed { field: provenance }`).

### 5.2 Masks (`masks/<id>.yaml`; `MODULE_SPEC` §4.1)

```yaml
format: 1
mask: core
version: 1
include:
  - { id: core.stature, why: "C, OV: height in both clients; tall" }
exclude: []
```

- Entries name an `id`, a `group` or a `facet`, each with a non-empty `why` (`MaskEntryUnjustified`); an id the
  backend's catalogue does not hold is `MaskNamesUnknownAttribute`. `all` is built in (everything).
- **Apply:** the backend's catalogue → keep what `include` selects (or everything, when `include` is absent) → drop
  what `exclude` selects. Masks apply **only to the backend's catalogue**: owners' attributes, other data packs'
  declarations and the world's own are never masked (`ARC-126` item 5; owners: S30-c).
- **What a mask disables does not exist** (S30-c's frozen catalogue is built after the mask): a person file or lock
  stating it is `MaskedOut { attribute, mask }`; a profile distribution naming it is `MaskedOut`; a `uses.reads` naming
  it is `MissingUnderBackend` unless `optional`.
- A world's own `masks/` files are collected like any; `mask: <id>` names one of the backend's or the world's;
  neither is `UnknownMask`.

### 5.3 Composition (`ARC-126` items 3–4; F-S30e-8)

1. The **backend** is chosen per F-S30e-8; the **mask** is `mask:`, else the backend's default.
2. The catalogue is: the backend's masked catalogue; **plus**, under a backend other than `mineworld`, every `core`
   attribute of group `appearance` and every `core` attribute of kind `quantity` (the appearance group and quantities
   are always MineWorld's, `ARC-125`) — which requires `person-attributes` among the world's requirements
   (`BackendMalformed { field: catalogues, reason: "the appearance group comes from person-attributes" }`); **plus** every
   other collected declaration (other data packs, the world, owners).
3. A backend other than `mineworld` declaring an attribute of group `appearance` or kind `quantity` is
   `BackendMalformed { field: catalogues }` ("a backend may not redeclare appearance or quantities").
4. Under the `mineworld` backend the `core` appearance group and quantities are masked like any `core` attribute (the
   `appearance` mask keeps only them, for casting-only use).
5. `attributes-composed` gains `backend: { id, version }` and `mask: { id, version }`, so a resumed world whose backend
   or mask changed is refused on drift (S30-c §5.11).

### 5.4 Reads under a backend

`uses.reads` gains `optional: true` (S30-c R-S30e-c-1). A required read of an id absent from the composed catalogue is
`Missing`, or `MissingUnderBackend { attribute, backend }` when the id is `core.*` and the backend is not `mineworld`.
An optional read is absent at run time (a handle reads `None`).

### 5.5 The built-in catalogue (Appendix A)

- 93 declarations in six files by group; every one `since: 1.0.0`; owner `attributes` (default).
- **Facets** follow the readers: every appearance attribute carries `appearance` (S26's C-1); and per reader code, `C`
  → `casting`, `OV` → `observed`, `ME` → `prompt`, `CV` → `speech`, `DM` → `decision`, `S31` → `notices`. A consumer
  selects by facet, never by id (X-S26src-10).
- **Visibility** is step-30 §6.1's column exactly; `natal-sex` is `private` (Q-S30a-1). The sensitive ones —
  `natal-sex`, `birth`, `age-years`, `ancestry`, `religion`, `mass`, `honesty` — are `private` (Q-S30-8).
- **Phrases** are short noun or adjective phrases a consumer can list ("tall", "grey hair", "wears glasses", "in their
  sixties", "walks briskly"); Appendix A gives each one, with the words where the slug does not read.
- **Defaults** only where a realistic world default exists (step-30 §6.2): `visible-marks: []`,
  `dietary-restrictions: []`. Every other attribute is absent unless stated, locked or profiled.
- **Domains** are Appendix A's. Ranges follow real values: `stature` 0.500–2.500 m, `mass` 2.0–300.0 kg as domains only
  (`DEP-68` Consequences); the realistic distribution is the profile's.
- **Vocabularies** (`core.languages`, `core.foods`, `core.hobbies`, `core.music`, `core.sports`) are Appendix A.3.
- **Masks:** `masks/core.yaml` — one `include` entry per `core` id, its `why` the reader line (F-S30e-3); `masks/
  appearance.yaml` — the 21 appearance ids, each `why: "C: casting-only use"`.
- `pack.yaml`: `id: person-attributes`, `type: entity-pack`, `version: 1.0.0`, `mineworld: "^0.1"`, `license: CC0-1.0`,
  `authors: [Yue Ma]`, `repository`. `NOTICE`: the MST credit (Q-S30e-2) and the sources of `DEP-68`.

### 5.6 The catalogue check (X-S30-20, AC-S30-2)

`mineworld attributes check <world>` prints, per namespace, the count of declarations, and, for `core`:

- the count, refused above **100** ("core holds 101 attributes; the built-in catalogue is curated to at most 100");
- every `core` id named **by id** in the `mineworld` backend's default mask with a `why` matching
  `^(C|OV|ME|CV|DM|S31)(, (C|OV|ME|CV|DM|S31))*: .+$`, refused naming the id otherwise ("core.x has no reader line").

The rule lives in the pack (it is a property of the `core` namespace and its backend, not of every world's
attributes); `attributes check` exits 1 on either refusal, and loading a world is not refused by it (a release check,
like `verify`).

### 5.7 San Diego County v3 (Appendix B)

- `entities/populations-us/populations/san-diego-county.yaml`, `profile: san-diego-county`, `version: 3`, `covers:
  adults`, in S30-d's format; `pack.yaml` `id: populations-us`, `license: CC0-1.0` (the compilation; the figures are
  US federal works in the public domain, `DEP-68`).
- **Re-reading.** Each `status: read` figure is re-read from its source in C5 with the `WebFetch` tool (no `curl`); the
  read date and the exact table cell are recorded in a comment beside the figure. A figure that cannot be read keeps
  the interim's value with `status: verify` and a `verify:` line saying what was attempted (Q-S30-5: a release refuses
  it on a reference world; S30-g decides).
- **Conventions** keep `source: authored` or the `mst-convention` source of status `authored` (A-e-6's "Authored
  casting conventions below, not data").
- **The A-10 reproduction** (`tools/cli/tests/attributes_builtin.rs`): an oracle table, captured in C0 from the interim
  file at `d81baeb3` (the figures copied by hand into the test with the file's SHA, test rules §25), compared with v3:
  sex, age band and ancestry marginals equal to the micro; build per sex equal; stature `mean`, `p5`, `p95` per
  stratum equal (cm → m); colour rows equal to `(1 − p_age) × uniform(pool) + p_age × uniform(replacement)` computed in
  the test; MST rows uniform over the interim's ranges. Then `--synthetic 1000` realises every stratified marginal
  within 0.5 pp (AC-S30-5) under v3. A figure changed by its re-read is excluded from equality, listed in the test with
  its old and new value and source, and recorded in §16 (never silently).

### 5.8 `mineworld attributes backends <world>`

Prints the backends the world's required packs provide (id, version, licence, attribution, masks and default,
sampling modes, provenance) and which one the world uses. `catalogue` adds `backend:` and `mask:` lines to its header.

### 5.9 Specification sync (C7)

- `MODULE_SPEC` §2: the layout's `backend.yaml` → `backends/mineworld.yaml` (Q-S30e-1, dated note); the data-directory
  table gains `backends/`; the `core` count stays 93.
- `MODULE_SPEC` §4.1 "Backends and masks": the contract's place; `mapping` and provenance digests are S30-h's
  (Q-S30e-3); masks apply to the backend's catalogue only; composition rules 2–4 (§5.3); F-S30e-8's default; the new
  refusal names `UnknownBackend`, `AmbiguousBackend`; the reader line (F-S30e-3).
- `MODULE_SPEC` §8.1: `attributes backends`.
- `DECISIONS.md`: a dated note under `ARC-126` (Q-S30e-1, Q-S30e-3, F-S30e-8, F-S30e-3); a dated note under `DEP-68`
  (Q-S30e-2's licensing of the MST reference; the figures as re-read, with dates).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-e-1 (RR-S30-3) | `core` holds 93 attributes, at most 100, each with a reader line | the catalogue check (§5.6); X-S30-20 |
| I-e-2 (Q-S30-8) | Every sensitive `core` attribute is `private` | a test listing them against Appendix A |
| I-e-3 (Q-S30-1, `ARC-125`) | Every appearance attribute is `perceivable`, except `mass` (step-30 §6.1 row 3) | the same test |
| I-e-4 | What a mask disables appears in no catalogue, observation or command output | the toy-backend tests; AC-e-6 |
| I-e-5 | Data and conventions stay apart: no `authored` figure is shown as data | the profile's sources; `resolve`'s header (S30-d) |
| I-e-6 (I-S30-10) | Reference worlds byte-identical (none requires either pack) | `social_cafe.rs`, the `decisions` golden; one nightly dispatch |
| I-e-7 | Nothing outside the two packs names a `core` id in code | S30-c's structural scan (handles name ids only in `const` items of readers; S30-e adds none) |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy on the model crate, the pack, the CLI | per commit |
| Unit (model crate) | backend contract decode and every `BackendMalformed`; mask decode, apply, `MaskEntryUnjustified`, `MaskNamesUnknownAttribute`, `UnknownMask`; composition rules; `UnknownBackend`, `AmbiguousBackend`; optional reads and `MissingUnderBackend` | `authoring/attributes/src/{backend,mask}/tests.rs` |
| Integration (real binary) | the built-in catalogue loads and checks clean (93, every reader line); X-S30-20's two plants; X-S30-16 on a copy of the pack; sensitivity and appearance visibility; the profile checks clean except listed `verify`; X-S30-15; the A-10 reproduction; the toy backend and masks end to end; the size re-measure | `tools/cli/tests/attributes_builtin.rs`, `tools/cli/tests/fixtures/attributes-builtin/`, `tools/cli/tests/fixtures/attributes-backends/` |
| Packages | `mineworld packs validate entities/person-attributes` and `entities/populations-us` pass (data-only Entity Packs, S30-b) | the same test file |
| Regression | I-e-6 | unchanged tests; one nightly dispatch |
| Gate 1, Gate 2 | NOT REQUIRED | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-e-1 | **Checkpoint.** `mineworld attributes check` on a fixture world requiring both packs, naming `san-diego-county`, exits 0 and lists only the `verify` marks of Appendix B (each with its reason) | S30-e row |
| AC-e-2 | **The A-10 reproduction** passes (§5.7), with every re-read difference listed | S30-e row |
| AC-e-3 | **AC-S30-2.** `catalogue --json` shows 93 `core` declarations in the six groups; `check` prints the count and each reader line | AC-S30-2 |
| AC-e-4 | **X-S30-20.** A copy of the pack with a 101st `core` declaration fails `check` ("core holds 101 …"); one `core` id removed from `masks/core.yaml` fails naming it | step-30 §13.5 |
| AC-e-5 | **X-S30-16 / X-S30-15.** A second `core.stature` is `DuplicateAttribute` naming both files; a v3 distribution without `source` is `UncitedDistribution` naming the attribute | step-30 §13.5 |
| AC-e-6 | **Backends and masks.** The toy backend fixture: `backend: toy` composes the toy catalogue plus `core` appearance and quantities and nothing else of `core`; `mask: toy-small` removes the rest from `catalogue`, from Bob's observation and from `show`; a person file stating a masked attribute is `MaskedOut`; switching back to `backend: mineworld` changes the catalogue and nothing else in the world's files | AC-S30-15's mechanism |
| AC-e-7 | **Sensitivity.** Every attribute of I-e-2 is `private`; every appearance attribute but `mass` is `perceivable` | Q-S30-8, Q-S30-1 |
| AC-e-8 | **Size.** AC-c-14's measurement repeated on the real catalogue with one person fully stated; recorded in §16 | R-S30e-c-1 |
| AC-e-9 | `mineworld packs validate` passes for both packs | A-e-2 |
| AC-e-10 | I-e-6 holds | X-S30-9 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

As S30-c §9.0, from `/Users/yuema137/mineworld-worktrees/impl-s30e`. **R-LOAD-1** always. The sub-agent of C3 (if
used) works in the same worktree only while the session waits, never in parallel with another writer (`CLAUDE.md`
§3.1, one tree, one writer).

#### C0 — Base, ledger and the oracle

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30e.md` (new).
- Steps: record the base; confirm S30-c and S30-d merged; record the interim file's SHA at `d81baeb3`
  (`git rev-parse origin/mvp0/s26-cast-director:presentation/mineworld-default/3D/characters/populations/san-diego-county.yaml`);
  confirm §15's rulings and apply them to §§5.1, 5.5 and Appendix A before C1.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: §2.3 holds.
- **Commit message:** `docs(plan): S30-e — open the ledger at the base`

#### C1 — Backends and masks in the model crate and the pack

- **Files:** `authoring/attributes/src/{backend.rs, mask.rs, config.rs, catalogue.rs}` and tests;
  `systems/attributes/src/{data.rs, system.rs}` (directories `backends`, `masks`; `attributes-composed`);
  `worldpack/tests/data_directories.rs` (the expected list).
- Steps: §§5.1–5.4; the new refusals; F-S30e-8.
- Validation: `cargo test -p mineworld-attributes-model --lib backend:: mask:: catalogue::`; `-p mineworld-attributes
  --test pack`; `-p mineworld-worldpack --test data_directories`. Mutation **M-e-1** (masks applied to owners'
  attributes too) → the owner-unmasked test red; **M-e-2** (appearance not composed under another backend) → the
  composition test red.
- [ ] Implementation. [ ] Validation. [ ] Review: masks reach only the backend's catalogue; a masked id is absent from
  `attributes-composed`.
- **Commit message:** `feat(attributes): person-generation backends and masks as data (ARC-126, S30-e)`

#### C2 — The toy backend fixture and `attributes backends`

- **Files:** `tools/cli/src/attributes.rs`, `tools/cli/tests/attributes_builtin.rs` (the backend half),
  `tools/cli/tests/fixtures/attributes-backends/**` (a toy backend Entity Pack with `backends/toy.yaml`, two masks, and a
  world).
- Steps: §5.8; AC-e-6.
- Validation: `cargo test -p mineworld-cli --test attributes_builtin` (filtered `backend_`).
- [ ] Implementation. [ ] Validation. [ ] Review: the toy backend names nothing real.
- **Commit message:** `test(cli): backends and masks end to end, and mineworld attributes backends (S30-e)`

#### C3 — `entities/person-attributes`: the 93 declarations, vocabularies and masks

- **Files:** `entities/person-attributes/**` (§5.5).
- Steps: write the six group files, five vocabularies and two masks **exactly from Appendix A** (ids, kinds, domains,
  visibility, mutability, facets, phrases, words, `perceived_as`, defaults, `since`); `backends/mineworld.yaml`
  (§5.1); `pack.yaml`; `README.md` (orientation, the reader codes, the sensitivity rule, a link to the specs);
  `NOTICE` (Q-S30e-2).
- Validation: `mineworld attributes catalogue` and `check` on `tools/cli/tests/fixtures/attributes-builtin/` (a world
  requiring the pack); `mineworld packs validate entities/person-attributes`.
- [ ] Implementation. [ ] Validation. [ ] Review (the session, row by row against Appendix A and step-30 §6.1): every
  visibility as the step's; every phrase reads in a list; no word is a stereotype; no sensitive attribute is
  `perceivable` or `learnable`.
- **Commit message:** `feat(entities): person-attributes — 93 curated core attributes, vocabularies and masks (ARC-122, S30-e)`

#### C4 — The catalogue check

- **Files:** `authoring/attributes/src/catalogue.rs` (the `core` rule), `tools/cli/src/attributes.rs` (`check`'s
  report), `tools/cli/tests/attributes_builtin.rs` (the catalogue half).
- Steps: §5.6; AC-e-3, AC-e-4, AC-e-5 (X-S30-16), AC-e-7, AC-e-9.
- Validation: `cargo test -p mineworld-cli --test attributes_builtin` (filtered `catalogue_`). Mutations **M-e-3** (the
  cap compared with `>` 101) → AC-e-4's first plant red; **M-e-4** (the reader line accepted by group) → the second
  plant red.
- [ ] Implementation. [ ] Validation. [ ] Review: the rule applies to `core` only.
- **Commit message:** `feat(attributes): the built-in catalogue check — at most 100, each with its reader (X-S30-20, S30-e)`

#### C5 — `entities/populations-us`: San Diego County v3, figures re-read

- **Files:** `entities/populations-us/**`.
- Steps: Appendix B; each `read` figure re-read with `WebFetch` (ACS through `data.census.gov` or Census Reporter's
  published table page; NHANES through the CDC PDF), recorded beside the figure (date, table, cell); unreadable →
  `verify` with what was tried; `mst-convention` and every convention `authored`; the header cites F-CAST-2 for
  lakeside.
- Validation: `mineworld attributes check` on the fixture world naming the profile; `resolve --synthetic 1000`.
- [ ] Implementation. [ ] Validation. [ ] Review: no figure lacks a source; data and conventions apart; the
  `verify` lines are honest about what was not read.
- **Commit message:** `feat(entities): populations-us — San Diego County v3, every figure cited or marked verify (DEP-68, S30-e)`

#### C6 — The reproduction and the profile's adversarial checks

- **Files:** `tools/cli/tests/attributes_builtin.rs` (the profile half).
- Steps: §5.7's oracle and comparison; AC-e-1, AC-e-2, AC-e-5 (X-S30-15), AC-e-8.
- Validation: the test file (filtered `profile_`). Mutation **M-e-5** (one ancestry figure changed by 0.0001) → the
  reproduction red naming it.
- [ ] Implementation. [ ] Validation. [ ] Review: the oracle came from the interim file, not from v3.
- **Commit message:** `test(entities): San Diego County v3 reproduces the interim profile's figures (S30-e)`

#### C7 — Specification sync

- **Files:** `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`.
- Steps: §5.9. Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Implementation. [ ] Validation. [ ] Review: every note dated and sourced; the spec and the packs agree.
- **Commit message:** `docs(spec): the built-in catalogue, backends and masks as shipped (ARC-126, DEP-68, S30-e)`

#### C8 — Close-out

- **Files:** this file (§16), the handoff.
- Steps: push `mvp0/pr-s30e`; `gh pr create`; CI on three OSes green; one `nightly.yml` dispatch (AC-e-10).
- [ ] Implementation. [ ] Validation. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-e close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. R-LOAD-1

As S30-c §10. The real-binary test file runs one filtered group at a time locally; CI on the exact head is the gate.

---

## 11. Requirements this PR places on other PRs

- **R-S30f-e-1 (S30-f).** Select by facets (`observed`, `prompt`, `speech`, `decision`), never by id; `prompt_rank` is
  S30-f's to add to declarations (a minor version of the pack).
- **R-S30g-e-1 (S30-g).** `social-cafe`, `market-town` and `lakeside` require `person-attributes: "^1"` and
  `populations-us`, and name `profile: san-diego-county`; `verify` marks on data figures block the release (Q-S30-5).
- **R-S30h-e-1 (S30-h).** The MatrAIx backend is `backends/matraix.yaml` in its pack (Q-S30e-1); `mapping` and
  provenance digests are S30-h's (Q-S30e-3); its catalogue declares no appearance group and no quantity (§5.3 item 3).
- **R-S30k-e-1 (S30-k).** The zh-Hans labels cover Appendix A's words and phrases.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §§6.4, 6.6 | `backend.yaml` at the pack root → `backends/<id>.yaml` (Q-S30e-1) |
| `step-30-person-attributes.md` §13.1, S30-e row | "lakeside" → lakeside names `san-diego-county` (F-S30e-4); `mapping` and provenance digests move to S30-h (Q-S30e-3) |
| `step-30-person-attributes.md` §6.1 | the reader line is the `core` mask's `why` (F-S30e-3) |
| `overall.md` | S30-e frozen / merged |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-e — the curated built-in catalogue, backends and masks, and the cited
                         population profile
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30e-builtin-catalogue.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§6, 8.2, 13, 16); pr-s30c-attributes-pack.md;
                         pr-s30d-sampler.md; docs/MODULE_SPEC.md §§2, 4.1, 8.1; docs/PACKAGE_FORMAT.md §5.0;
                         docs/DECISIONS.md ARC-55, ARC-71, ARC-95, ARC-120…ARC-126, DEP-68; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30e, its own, held by one session
BRANCH:                  mvp0/pr-s30e, created from origin/main
START CONDITION:         S30-c AND S30-d merged AND this design frozen AND §15 ruled, then the primary's dispatch
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–6, as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-10; RR-S30-3; Q-S30-1, Q-S30-8; §6 I-e-1 … I-e-7; §8 as written;
                         Appendix A's visibility column as step-30 §6.1's
SEQUENCE:                C0 … C8 (§9)
IMPLEMENTER:             Opus-class session; C3 may be delegated to a Haiku-class sub-agent from Appendix A,
                         reviewed by the session; reviewer the primary session
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *; git;
                         gh (pr create/view; workflow run nightly.yml once in C8; never merge); python3
                         scripts/check_*.py, ci_layer.py docs; WebFetch of the cited sources (read-only, C5);
                         cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1);
                         a figure without a source; copying the MST publication's swatches or text; any reference-
                         world edit; any model call; a paid API; a new external dependency
MATERIAL STOPS:          S30-c or S30-d not merged; a needed change to S30-c's or S30-d's frozen formats beyond
                         §5's additive fields; a step-30 §6.1 row that cannot be kept with its reader or its
                         visibility; a re-read figure that changes a marginal by more than 2 pp (reported to the
                         operator with both values, the source and the date before it is written)
PLATFORMS:               Linux, macOS, Windows (CI)
VALIDATION BUDGET:       tests unrestricted one file at a time; WebFetch of the sources as needed; one nightly
                         dispatch (C8). Gate 1 and Gate 2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30e.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                     (task brief 2026-10-17: "endpoint is a PR ready for
                                                      review, with no merge")
                         semantic commits: authorized                (working rules §14)
                         branch push: authorized                     (working rules §21)
                         PR creation / update: authorized            (working rules §21)
                         CI repair to review readiness: authorized   (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (working
                         rules §22)
POST-MERGE SYNC OWNER:   primary: step-30, overall.md; implementation session: this ledger
STOP CONDITION:          S30-e READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | The catalogue's words become an accidental standard (step-30 R-S30-7) | semantic versioning; `since`; ids never reused; words are presentation and overridable |
| R-e-2 | A sensitive attribute leaks through a phrase or a facet | I-e-2's test; private attributes carry no `observed` facet; S30-c's 30-day scan applies to any world using the pack |
| R-e-3 | Census categories read as a universal ancestry taxonomy | F-S30e-7's README line; worlds declare their own; the values are the source's, pooled as cited |
| R-e-4 | A source cannot be re-read (timeouts, 403 as on 2026-10-15) | the figure stays `verify` with what was tried; a release refuses it on a reference world (Q-S30-5); S30-g decides |
| R-e-5 | 93 declarations entered by a sub-agent drift from Appendix A | the session reviews row by row; I-e-2/I-e-3's tests and the catalogue check run on the result |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30e-1 [R-S30e-a-2, `ARC-121`/`ARC-126`]** | `backend.yaml` sits at the pack root in the specification, but the seam reads only owned directories and `pack.yaml` admits no other field (F-S30e-1). Extend the seam with root files, or move the contract? | **Move it: `backends/<id>.yaml`, exactly one per backend pack, in a directory `attributes` owns.** No seam change; one rule for every file `attributes` reads (S30-d's `locks/` likewise, Q-S30d-2); `MODULE_SPEC` §§2, 4.1 corrected by a dated note. The seam extension would add a capability for one file and a second way a pack contributes data |
| **Q-S30e-2 [`DEP-68`, `ARC-55`]** | The Monk Skin Tone scale is CC BY 4.0, which the default licence policy does not admit, and the catalogue pack is CC0 (F-S30e-2). How is it carried? | **Keep the pack `CC0-1.0`; `core.skin-tone` is ten levels `mst-1` … `mst-10` with MineWorld's own words; nothing of the scale's publication (swatch colours, text) is copied; the scale is credited in `NOTICE` and the README as the reference the levels follow.** The alternative — licence the pack `CC0-1.0 AND CC-BY-4.0` and add `CC-BY-4.0` to the default policy — changes `ARC-55` for every world to carry ten numbered levels |
| **Q-S30e-3 [scope between S30-e and S30-h]** | `ARC-126` assigns "the backend and mask loader" to S30-e. The default backend has no mapping and no upstream files; only the optional MatrAIx backend (S30-h) uses `mapping` and provenance digests (which need a SHA-256 the workspace does not have). Move both to S30-h? | **Yes.** S30-e validates the contract with `mapping` absent and `provenance: { source: mineworld }`; S30-h adds the mapping format, its coverage check and digest verification with its first user (`CLAUDE.md` §4 rule 11). The step-30 §13.1 rows are amended by the primary |

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C8 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-e-1 … M-e-5 as §9 lists them)

### Deviations and findings

F-S30e-1 … F-S30e-8 (§3.2), recorded at design time.

---

## Appendix A — The 93 `core` attributes (frozen content of C3)

Columns: **kind** (domain in order; `L5` = `[very-low, low, moderate, high, very-high]`); **vis** (P perceivable, L
learnable, Pr private); **mut** (F fixed, E evolving, D derived); **facets** beyond `appearance` for the appearance
group, from the reader codes (§5.5); **phrase**, and **words** where a slug does not read; **reader** = the `why` of the
`core` mask entry (step-30 §6.1). `default` only where stated. Every row `since: 1.0.0`, owner `attributes`.

### A.1 Appearance (21) — `attributes/appearance.yaml`

| # | id | kind | vis | mut | facets | phrase · words · perceived_as | reader |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `body-form` | category `[feminine, masculine, neutral]` | P | F | casting, observed, body | `"{value} build of body"` · `feminine: feminine-presenting, masculine: masculine-presenting, neutral: androgynous` | C, OV: the body casting builds and an observer sees |
| 2 | `stature` | quantity m, scale 3, 0.500–2.500 | P | F | casting, observed, body | `"{value} m tall"` · perceived_as bands `below 1.55: short`, `below 1.85: of average height`, else `tall` | C, OV: height in both clients; tall |
| 3 | `mass` | quantity kg, scale 1, 2.0–300.0 | Pr | E | casting, body | `"{value} kg"` | C: drives build consistency at casting |
| 4 | `build` | category `[slim, average, solid, heavy]` | P | E | casting, observed, body | `"{value} build"` · `solid: solid, sturdy; heavy: heavy-set` | C, OV: slim to heavy-set |
| 5 | `skin-tone` | level `[mst-1 … mst-10]` | P | F | casting, observed | `"{value} skin"` · `mst-1: very fair, mst-2: fair, mst-3: light, mst-4: light-medium, mst-5: medium, mst-6: medium-tan, mst-7: tan-brown, mst-8: brown, mst-9: deep brown, mst-10: very deep brown` | C, OV: skin tone in both clients |
| 6 | `hair-colour` | category `[black, dark-brown, brown, light-brown, blond, auburn, red, grey, silver-grey, white]` | P | E | casting, observed | `"{value} hair"` · hyphens as spaces | C, OV: hair colour; greys with age |
| 7 | `hair-length` | level `[shaved, very-short, short, chin-length, shoulder-length, long]` | P | E | casting, observed | `"{value} hair"` | C, OV: hair length |
| 8 | `hair-style` | category `[chin-length-bob, pixie-cut, low-bun, low-ponytail, braided-updo, top-knot, short-layered, short-crop, buzz-cut, side-part, textured-top, shaved-head, swept-back, close-fade, loose-long]` | P | E | casting, observed | `"hair in a {value}"` · `chin-length-bob: chin-length bob, low-bun: low bun, braided-updo: braided updo, top-knot: top knot, short-layered: short layered cut, short-crop: short crop, buzz-cut: buzz cut, side-part: side part, textured-top: textured top, shaved-head: shaved head, swept-back: swept-back style, close-fade: close fade, loose-long: long loose style` | C, OV: hair style |
| 9 | `hair-texture` | category `[straight, wavy, curly, coily]` | P | F | casting | `"{value} hair"` | C: hair texture at casting |
| 10 | `facial-hair` | category `[none, stubble, short-beard, full-beard, moustache]` | P | E | casting, observed | `"{value}"` · `none: clean-shaven, short-beard: a short beard, full-beard: a full beard, moustache: a moustache, stubble: stubble` | C, OV: facial hair |
| 11 | `eye-colour` | category `[brown, dark-brown, hazel, amber, green, blue, grey]` | P | F | casting, observed | `"{value} eyes"` | C, OV: eye colour at conversation distance |
| 12 | `face-shape` | category `[oval, round, square, heart, long, diamond]` | P | F | casting | `"a {value} face"` · `heart: heart-shaped, diamond: diamond-shaped` | C: distinct faces |
| 13 | `eye-shape` | category `[almond, round, hooded, deep-set, wide-set, downturned]` | P | F | casting | `"{value} eyes"` · `almond: almond-shaped` | C: distinct faces |
| 14 | `nose` | category `[straight, broad, narrow, aquiline, upturned, wide-bridged]` | P | F | casting | `"a {value} nose"` | C: distinct faces |
| 15 | `lips` | category `[full, medium, thin, wide]` | P | F | casting | `"{value} lips"` | C: distinct faces |
| 16 | `glasses` | flag | P | E | casting, observed | words `true: wears glasses`, `false: ""` | C, OV: glasses |
| 17 | `visible-marks` | set of `[arm-tattoo, hand-tattoo, neck-tattoo, facial-piercing, facial-scar, birthmark]`, max 6 · default `[]` | P | E | casting, observed | `"{value}"` · `arm-tattoo: a tattoo on the arm`, … (each "a … on the …") | C, OV: tattoos, scars, birthmarks |
| 18 | `clothing-style` | category `[casual, smart-casual, business, workwear, sporty, bohemian, vintage, uniform]` | P | E | casting, observed | `"dressed in {value} clothes"` · `smart-casual: smart casual, uniform: uniform-like` | C, OV: clothing |
| 19 | `headwear` | category `[none, cap, flat-cap, beanie, sun-hat, wide-brim-hat, headscarf]` | P | E | casting, observed | `"{value}"` · `none: ""`, `cap: a cap`, `flat-cap: a flat cap`, `beanie: a beanie`, `sun-hat: a sun hat`, `wide-brim-hat: a wide-brimmed hat`, `headscarf: a headscarf` | C, OV: headwear |
| 20 | `posture` | category `[upright, relaxed, slight-stoop, stooped]` | P | E | casting, observed | `"{value}"` · `upright: stands upright, relaxed: stands easily, slight-stoop: a slight stoop, stooped: stooped` | C, OV: an older person's stoop |
| 21 | `voice` | category `[soft, warm, bright, deep, husky, raspy, high, booming]` | P | E | observed, speech | `"a {value} voice"` | OV, CV: a deep, slow voice |

### A.2 Background (15), Psychology (16), Capability (13), Behaviour (14), Lifestyle (14)

| # | id | kind | vis | mut | facets | phrase · words | reader |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 22 | `natal-sex` | category `[female, male]` | Pr | F | casting | `"{value}"` | C: the profile parent of stature, build and facial hair (Q-S30a-1) |
| 23 | `birth` | instant | Pr | F | prompt, decision | `"born {value}"` (the instant renders as seconds before the epoch only for the holder's tools; S30-f renders a date) | ME, DM: through age |
| 24 | `age-years` | quantity year, scale 0, 0–130 · `derive: { age: { from: core.birth } }` | Pr | D | prompt, decision | `"{value} years old"` | ME, DM: what a person of that age does |
| 25 | `age-band` | level `[child, teen, 20s, 30s, 40s, 50s, 60s, 70s, 80-plus]` · `derive: { band: { from: core.age-years, below: [13, 20, 30, 40, 50, 60, 70, 80] } }` | P | D | casting, observed | `"{value}"` · `child: a child, teen: a teenager, 20s: in their twenties, …, 80-plus: in their eighties or older` | C, OV: in her sixties |
| 26 | `pronouns` | category `[she-her, he-him, they-them]` | P | E | prompt, observed, speech | `"{value}"` · `she-her: she/her, he-him: he/him, they-them: they/them` | ME, OV, CV: how others refer to them |
| 27 | `ancestry` | category `[hispanic-or-latino, white, black, asian, multiracial-or-other]` (F-S30e-7) | Pr | F | casting | `"{value}"` | C only: the profile parent of skin tone and hair; never rendered to others |
| 28 | `native-language` | category `vocabulary: core.languages` | L | F | prompt, speech | `"{value} as a first language"` | ME, CV |
| 29 | `languages-spoken` | set `vocabulary: core.languages`, max 8 | L | E | prompt, speech | `"speaks {value}"` | ME, CV: who switches language with whom |
| 30 | `hometown` | text, max 80 | L | F | prompt | `"from {value}"` | ME: what they talk about |
| 31 | `years-in-town` | quantity year, scale 0, 0–100 | L | E | prompt, decision | `"{value} years in town"` | ME, DM: newcomer or old-timer |
| 32 | `education-level` | level `[none, primary, secondary, vocational, some-college, bachelor, master, doctorate]` | L | E | speech | `"{value} education"` · `some-college: some college, bachelor: a bachelor's, master: a master's` | CV: register |
| 33 | `marital-status` | category `[single, partnered, married, separated, divorced, widowed]` | L | E | prompt | `"{value}"` | ME |
| 34 | `household` | category `[alone, partner, family, parents, roommates, extended-family]` | L | E | prompt, decision | `"lives {value}"` · `alone: alone, partner: with a partner, family: with family, parents: with parents, roommates: with roommates, extended-family: with extended family` | ME, DM: who waits at home |
| 35 | `children` | quantity count, scale 0, 0–20 | L | E | prompt, decision | `"{value} children"` | ME, DM |
| 36 | `religion` | category `[none, christian, muslim, jewish, hindu, buddhist, sikh, other]` | Pr | E | prompt | `"{value}"` · `none: not religious` | ME |
| 37–41 | `openness`, `conscientiousness`, `extraversion`, `agreeableness`, `neuroticism` | level L5 | L | E | decision, prompt | `"{value} <trait>"` (e.g. `"high extraversion"`) | DM, ME: the five traits weigh every choice |
| 42 | `humour` | category `[dry, playful, sarcastic, gentle, earnest]` | L | E | speech | `"a {value} sense of humour"` | CV |
| 43 | `patience` | level L5 | L | E | speech, decision | `"{value} patience"` | CV, DM: how long they wait or argue |
| 44 | `trust-propensity` | level L5 | L | E | prompt, decision | `"{value} trust in strangers"` | ME, DM: warmth to strangers |
| 45 | `curiosity` | level L5 | L | E | decision | `"{value} curiosity"` | DM: explores new places and people |
| 46 | `risk-tolerance` | level L5 | L | E | decision, notices | `"{value} appetite for risk"` | DM, S31: buying and trying |
| 47 | `formality` | level L5 | L | E | speech | `"{value} formality"` | CV |
| 48 | `optimism` | level L5 | L | E | prompt, speech | `"{value} optimism"` | ME, CV |
| 49 | `honesty` | level L5 | Pr | E | prompt | `"{value} honesty"` | ME: whether the LM character shades the truth |
| 50 | `core-value` | category `[family, work, faith, fun, community, knowledge, freedom, security]` | L | E | prompt, decision | `"values {value} most"` | ME, DM |
| 51 | `ambition` | level L5 | L | E | decision | `"{value} ambition"` | DM: overtime, errands |
| 52 | `generosity` | level L5 | L | E | decision | `"{value} generosity"` | DM: treats, tips, help |
| 53–64 | `skill.cooking`, `skill.baking`, `skill.coffee`, `skill.fishing`, `skill.woodwork`, `skill.boat-handling`, `skill.painting`, `skill.music`, `skill.teaching`, `skill.care`, `skill.selling`, `skill.gardening` | level `[none, novice, competent, skilled, expert]` | L | E | prompt, decision | `"{value} at <skill words>"` (e.g. `"skilled at coffee"`; words of the skill: `boat-handling: handling boats`, `care: caring for people`) | ME, DM: what they choose to do and how they talk about it |
| 65 | `fitness` | level L5 | L | E | prompt, decision | `"{value} fitness"` | ME, DM |
| 66 | `chronotype` | level `[early-bird, morning-leaning, neutral, evening-leaning, night-owl]` | L | E | decision | `"{value}"` · `early-bird: an early bird, night-owl: a night owl, morning-leaning: a morning person, evening-leaning: an evening person, neutral: neither early nor late` | DM: when they rise and go home |
| 67 | `sleep-need` | level `[short, average, long]` | L | E | decision | `"needs {value} sleep"` · `short: little, average: average, long: a lot of` | DM |
| 68 | `punctuality` | level `[often-late, sometimes-late, on-time, early]` | L | E | decision | `"{value}"` · hyphens as spaces, `early: usually early` | DM: on time or late to work |
| 69 | `talkativeness` | level L5 | L | E | decision, speech | `"{value} talkativeness"` | DM, CV: who starts conversations, and how long they talk |
| 70 | `sociability` | level L5 | L | E | decision | `"{value} sociability"` | DM: joins group activities |
| 71 | `routine-adherence` | level L5 | L | E | decision | `"{value} attachment to routine"` | DM: keeps the agenda or drifts |
| 72 | `walking-pace` | level `[slow, unhurried, average, brisk, fast]` | P | E | decision, observed | `"walks {value}"` · `slow: slowly, unhurried: unhurriedly, average: at an easy pace, brisk: briskly, fast: fast` | DM, OV: walks briskly |
| 73 | `spending-style` | level `[frugal, careful, moderate, free, lavish]` | L | E | decision, notices | `"{value} with money"` | DM, S31 |
| 74 | `tipping` | level `[never, rarely, sometimes, usually, generously]` | L | E | decision | `"tips {value}"` | DM |
| 75 | `gossip` | level L5 | L | E | decision, speech | `"{value} love of gossip"` | DM, CV: passes on news |
| 76 | `greeting-style` | category `[nod, wave, handshake, hug, cheerful-hello, formal]` | P | E | speech, observed | `"greets with {value}"` · `nod: a nod, wave: a wave, handshake: a handshake, hug: a hug, cheerful-hello: a cheerful hello, formal: formal courtesy` | CV, OV |
| 77 | `helpfulness` | level L5 | L | E | decision | `"{value} helpfulness"` | DM |
| 78 | `news-interest` | level L5 | L | E | notices | `"{value} interest in the news"` | S31: default notice subscriptions |
| 79 | `weather-sensitivity` | level L5 | L | E | decision | `"{value} sensitivity to weather"` | DM: stays in when it rains |
| 80 | `diet` | category `[omnivore, flexitarian, pescatarian, vegetarian, vegan]` | L | E | decision | `"{value}"` · `omnivore: eats everything` | DM: what they buy and eat |
| 81 | `dietary-restrictions` | set of `[halal, kosher, gluten-free, lactose-free, nut-allergy, low-sugar]`, max 6 · default `[]` | L | E | decision | `"keeps {value}"` · `nut-allergy: away from nuts, low-sugar: low sugar` | DM |
| 82 | `caffeine` | level `[none, light, moderate, heavy]` | L | E | decision | `"{value} coffee drinker"` · `none: not a` | DM: café visits |
| 83 | `alcohol` | level `[none, occasional, social, regular]` | L | E | decision | `"drinks {value}"` · `none: no alcohol, occasional: occasionally, social: socially, regular: regularly` | DM |
| 84 | `smoking` | level `[never, former, occasional, daily]` | P | E | observed, prompt | `"{value}"` · `never: ""`, `former: ""`, `occasional: smokes now and then`, `daily: a smoker` | OV, ME |
| 85 | `food-favourites` | set `vocabulary: core.foods`, max 5 | L | E | decision | `"likes {value}"` | DM: orders at the counter |
| 86 | `hobbies` | set `vocabulary: core.hobbies`, max 6 | L | E | decision, prompt | `"enjoys {value}"` | DM, ME: where they spend free time |
| 87 | `exercise-habit` | level `[none, light, regular, daily]` | L | E | decision | `"exercises {value}"` · `none: rarely, light: lightly` | DM: the runner runs |
| 88 | `reading-habit` | level `[rarely, sometimes, often, avid]` | L | E | decision, notices | `"reads {value}"` · `avid: avidly` | DM, S31: the bench reader, the newspaper |
| 89 | `music-taste` | set `vocabulary: core.music`, max 4 | L | E | prompt | `"listens to {value}"` | ME |
| 90 | `sport-followed` | set `vocabulary: core.sports`, max 3 | L | E | prompt | `"follows {value}"` | ME |
| 91 | `pet` | category `[none, dog, cat, bird, fish, other]` | L | E | prompt | `"{value}"` · `none: has no pet, dog: has a dog, cat: has a cat, bird: has a bird, fish: keeps fish, other: has a pet` | ME |
| 92 | `faith-practice` | level `[none, occasional, regular, devout]` | L | E | decision | `"{value} in practice"` · `none: not observant` | DM: observance days |
| 93 | `outdoor-preference` | level `[homebody, indoor-leaning, balanced, outdoor-leaning, outdoorsy]` | L | E | decision | `"{value}"` · hyphens as spaces | DM: park or indoors |

Groups: rows 22–36 `background` (15); 37–52 `psychology` (16); 53–65 `capability` (13); 66–79 `behaviour` (14); 80–93
`lifestyle` (14). Total 21 + 15 + 16 + 13 + 14 + 14 = **93**. Step-30 §6.1's rows 32 (`occupation`) and 37
(`wealth-band`) are `employment.occupation` (S30-f) and `economy.wealth-band` (S31); `freckles` is deliberately absent
(AC-S30-3's custom example).

A phrase whose words render empty (`""`) renders no text for that value (S30-c F-S30c-6): an observer is not told
"never smokes" or "no hat".

### A.3 Vocabularies — `vocabularies/*.yaml`

| Vocabulary | Values |
| --- | --- |
| `core.languages` | english, spanish, tagalog, vietnamese, mandarin, cantonese, korean, japanese, arabic, persian, french, german, italian, portuguese, russian, hindi, punjabi, somali, amharic, asl |
| `core.foods` | coffee, espresso, tea, pastry, bread, sandwich, salad, soup, pasta, pizza, tacos, sushi, noodles, rice-bowl, fish, burger, fruit, ice-cream, chocolate, cake |
| `core.hobbies` | reading, gardening, fishing, painting, music, cooking, baking, running, cycling, hiking, swimming, chess, board-games, photography, knitting, woodwork, birdwatching, volunteering, dancing, video-games |
| `core.music` | classical, jazz, rock, pop, hip-hop, country, folk, electronic, latin, r-and-b, blues, k-pop |
| `core.sports` | football, american-football, baseball, basketball, ice-hockey, tennis, golf, cricket, cycling, running, surfing, motorsport |

---

## Appendix B — San Diego County v3 (frozen structure of C5; figures re-read in C5)

| Attribute | `given` | Form | Figures (from A-e-6, re-read in C5) | Source · status |
| --- | --- | --- | --- | --- |
| `core.natal-sex` | — | prior | female 0.4986, male 0.5014 | `acs-2024-1y` · read |
| `core.age-band` | — | prior | 20s 0.1864, 30s 0.2050, 40s 0.1716, 50s 0.1506, 60s 0.1417, 70s 0.0973, 80-plus 0.0474 (child, teen 0: `covers: adults`) | `acs-2024-1y` · read |
| `core.birth` | `[core.age-band]` | `uniform_age_within_band { upper: 89 }` | the interim's 80+ range `[80, 88]` | `authored` |
| `core.ancestry` | — | prior | hispanic-or-latino 0.3525, white 0.4129, asian 0.1258, black 0.0429, multiracial-or-other 0.0659 | `acs-2024-1y` (B03002, the four smallest non-Hispanic groups pooled) · read |
| `core.build` | `[core.natal-sex]` | prior | female slim 0.144, average 0.167, solid 0.262, heavy 0.427; male 0.088, 0.162, 0.337, 0.413 | `nhanes-2015-2018` · read; `verify: "county figure (CHIS) not read; national NHANES over-represents heavy here"` |
| `core.stature` | `[core.natal-sex, core.age-band]` | normal, `clip_sd: 2` | the 14 `[mean, p5, p95]` of A-e-6 in metres (162.6 cm → 1.626) | `nhanes-2015-2018` · read |
| `core.skin-tone` | `[core.ancestry]` | prior, uniform over the interim's MST ranges | hispanic-or-latino mst-2…mst-7; white mst-1…mst-4; asian mst-2…mst-6; black mst-5…mst-10; multiracial-or-other mst-3…mst-8 | `mst-convention` · authored |
| `core.hair-texture` | `[core.ancestry]` | prior | the interim's weights per ancestry | `authored` |
| `core.hair-colour` | `[core.ancestry, core.age-band]` | prior, 35 full rows | `(1 − p_age)·uniform(pool_ancestry) + p_age·uniform(replacement_age)`; p 50s 0.35, 60s 0.7, 70s 0.95, 80-plus 1.0, else 0; replacements 50s/60s grey, 70s/80-plus silver-grey and white; each row rounded to 6 decimals by largest remainder | `mst-convention` · authored |
| `core.hair-style` | `[core.natal-sex]` | prior, uniform over the interim's pools | female: chin-length-bob, pixie-cut, low-bun, low-ponytail, braided-updo, top-knot, short-layered; male: short-crop, buzz-cut, side-part, textured-top, shaved-head, swept-back, close-fade | `authored` (F-S30e-5) |
| `core.facial-hair` | `[core.natal-sex]` | prior and mask | female none 1; male none 0.55, stubble 0.15, short-beard 0.20, moustache 0.10; mask `when female only [none]` | `authored` |
| `core.face-shape`, `core.eye-shape`, `core.nose`, `core.lips` | — | prior, uniform over the domain | — | `authored` |

`sources:` — `acs-2024-1y` (cite, URL, read date: as A-e-7, re-read), `nhanes-2015-2018` (Fryar et al. 2021, Vital
Health Stat 3(46), Tables 9, 11, 14, 15), `county-build` (CHIS, `status: verify`), `mst-convention` (`status:
authored`, citing Q-S26-7 and `ARC-95`). Not profiled in v3: `core.mass` (F-S30e-6) and every non-appearance attribute
(absent unless authored; their distributions are later versions' or a world's own). The header records F-CAST-2:
lakeside uses this profile.

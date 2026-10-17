# PR S30-d — Population profiles and the deterministic sampler: integer, stratified, biography first

## DRAFT — not frozen

```text
Design revision:        revision 1 (2026-10-17), DRAFT
Author:                 S30-c / S30-d / S30-e PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s30cde, branch docs/pr-designs-s30cde,
                        planning base origin/main @ 38323d4c (#238)
Approved by / evidence: none yet. The freeze header is the operator's to approve and the primary
                        session's to write (CLAUDE.md §3.1)
Implementation base:    origin/main at C0; C0 records the commit and that S30-c has merged
Start condition:        S30-c merged (step-30 §13.3), this design frozen, §15 ruled, the primary's dispatch
Execution contract:     §13
Implementer:            Opus-class (step-30 §13.1, the S30-d row)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) §§3.1 (A-9, A-10, A-11),
3.3 (F-S30-4, F-S30-5), 8 (`ARC-123`, SD-S30-7), 10 (C-1 … C-7), 12 (I-S30-6, I-S30-7), 13.1 (the S30-d row), 13.4
(AC-S30-5 … AC-S30-8), 13.5 (X-S30-5, X-S30-6, X-S30-12 … X-S30-14) · **Specified by:** `docs/MODULE_SPEC.md` §4.1 "The
world's configuration", "Population profiles", "The lock file"; `docs/DECISIONS.md` `ARC-123`, `DEP-67` · **Builds on:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (its §5.7 resolution, §5.11 drift check, R-S30d-c-1) ·
**Consumed by:** S26-src2 and S30-g (`step-26-cast.md` §7.4b.1, C-1 … C-7), S30-e (the cited profiles), S30-j
(runtime sampling) · **Parent:** [`overall.md`](overall.md) · **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 7, 9,
10, 11, 12, 16); `docs/DECISIONS.md` `ARC-27`, `ARC-61`, `ARC-121` … `ARC-123`, `DEP-10`, `DEP-67`, `DEP-68`.

**Decision ids.** None new. Dated notes under `ARC-123` (§5.13). `ARC-127` … `ARC-129` and `DEP-69` unused.

**The one hard requirement of the brief:** the sampler's output is **byte-reproducible on Linux, macOS and Windows,
using integer arithmetic only** (AC-S30-7). Every formula below is stated in integers; §5.2 says where a number
crosses from YAML and how it becomes an integer without float arithmetic.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-d row).** "sampler in the model crate (integer weights, masks, adjustments, normal table,
  largest remainder, lock, roles); `configure/attributes.yaml`; `mineworld attributes resolve/lock`; synthetic-N
  report."
- **Integration checkpoint (same row).** "`resolve --json` on a fixture world prints C-1…C-7; a 1,000-person synthetic
  run matches within AC-S30-5."
- **Adversarial criteria (same row).** "X-S30-5, X-S30-6, X-S30-12…14; X-24/X-25 over the output."
- **Step acceptance owned here.** AC-S30-5 (1 000 synthetic people: marginals within 0.5 pp, conditionals within 2 pp
  for groups ≥ 50); AC-S30-6 (no masked value in 10 000; an emptying mask refused); AC-S30-7 (`resolve --json`
  byte-identical across runs and on the three platforms; another seed changes only `sampled` values); AC-S30-8
  (authored, ruled and locked values survive a new seed and a new profile version).
- **S26's contract (step-26 §7.4b.1).** C-1 declarations by facet; C-2 a value per person with its origin, keyed by the
  look key; C-3 determinism; C-4 the profile's identity; C-5 realised against target, for the cast and `--synthetic`;
  C-6 named failure before any output; C-7 declared numeric support per stratum. X-24: the same seed reproduces the
  output byte for byte, a different seed changes only sampled values. X-25: a synthetic 1 000-person cast within 2 pp.
- **Invariants.** I-S30-6: "The sampler is a pure function of (catalogue, profile, lock, people files without names,
  seed, salt), identical on three platforms." I-S30-7: "Authored and ruled values are never re-drawn."
- **R-S30d-c-1 (S30-c).** The sampler sits in S30-c's precedence between the person's value and the default; its inputs
  are S30-b's `Subject`s and S30-c's restrictions; `attributes-composed` gains the population's identity.
- **Rulings that bind.** Populations live in a world-required data pack (Q-S30-2; S30-e ships it); a release refuses
  `verify` on data figures, never on `authored` conventions (Q-S30-5; S30-d reports, S23 refuses); the population
  seed is not the run seed (`ARC-123` item 4).

---

## 1. Goal

One integer sampler in `mineworld-attributes-model`, with two entry points — the world's genesis and `mineworld
attributes resolve` — that fills the attributes a person's file does not state from the world's profile:
dependency-aware over a strict DAG, stratified by largest remainder within each parent group, masked, adjusted,
reproducible from the world's population seed on every platform, never re-drawing what a biography or a casting
ruling fixed, and never reading a name.

---

## 2. Scope

### 2.1 In scope

1. **Integer primitives** (`authoring/attributes/src/sample/{hash.rs, micro.rs, remainder.rs, normal.rs,
   normal_table.rs}`): FNV-1a 64, SplitMix64 `mix`, decimal → micro-units, largest remainder, the committed
   standard-normal quantile table and its integer interpolation; `authoring/attributes/tools/normal_table.py` (the
   table's offline generator, standard library only).
2. **The world's configuration** `configure/attributes.yaml` (`attributes` becomes configurable, `configures!()`):
   `format`, `profile`, `seed`, `salt`, `lock`.
3. **Profiles** (`authoring/attributes/src/profile/*.rs`): the `populations/` directory, decoding, every static rule of
   `MODULE_SPEC` §4.1 with its named refusal.
4. **The sampler** (`authoring/attributes/src/sample/{order.rs, weights.rs, allocate.rs, numeric.rs, roles.rs,
   resolve.rs, report.rs}`).
5. **The lock** (`authoring/attributes/src/lock.rs`): the `locks/` directory, decoding, precedence, a canonical writer.
6. **Genesis wiring** in `systems/attributes` (the resolution of S30-c §5.7 gains lock, roles and profile;
   `attributes-composed` gains `population`).
7. **The CLI** `mineworld attributes resolve` and `mineworld attributes lock` (`tools/cli/src/attributes.rs`).
8. **Fixtures and tests:** the fixture world of S30-c gains a profile and a lock; a second profile ("harbour-of-elsewhere")
   with its own vocabulary; goldens; the adversarial plants.
9. **Specification sync:** `docs/MODULE_SPEC.md` §§4.1, 8.1 and `docs/DECISIONS.md` notes under `ARC-123` (§5.13).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The cited San Diego County profile and `entities/populations-us`; the built-in catalogue | S30-e |
| `backend:` and `mask:` in `configure/attributes.yaml`; the `log-linear` mode | S30-e; S30-h |
| Reference worlds adopting a profile; S26 casting reading `resolve --json`; removing the interim Python sampler | S30-g |
| Sampling a newcomer at run time | S30-j |
| Refusing `verify` marks at release | S23 (reads `attributes check`'s list) |

### 2.3 Dependencies

S30-c merged (the model crate, the pack, `attributes-composed`, the drift check). Optional: S26-a (#206) for C-2's look
key (§5.10, F-S30d-6).

---

## 3. Audit (`origin/main @ 38323d4c`; S30-b at `mvp0/pr-s30b @ 40af5a11`; the interim sampler at
`origin/mvp0/s26-cast-director @ d81baeb3`; 2026-10-17)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-d-1 | SplitMix64's finalizer `mix(a, b)` is restated per crate (a System Pack never depends on cognition); with `a = 0` it is the reference generator, pinned by a test | `systems/weather/src/draw.rs:1-30`; `cognition/decision/src/draw.rs:54-59` | the model crate restates it with the same pinned test |
| A-d-2 | Weather draws a uniform per-mille value by multiply-shift on the high 32 bits: `((x >> 32) × 1000) >> 32` | `systems/weather/src/draw.rs:24-29` | the same idiom, at parts per million |
| A-d-3 | A decimal crosses YAML as `f64`; calendar converts at the boundary | `systems/calendar/src/configuration.rs:120-130` | S30-c's exact decimal decoder (its §5.4) is reused for probabilities, ratios and numeric parameters |
| A-d-4 | `ResolvesSubjects::resolve(context, configuration: Option<&dyn Any>, subjects)` is called once at read step 10b; `Subject` holds key, kind, tags, section — never a name, note or location | `mvp0/pr-s30b:authoring/src/data.rs:420-483`; `worldpack/src/resolution.rs` | genesis's entry point; X-S30-6's structural half is the type |
| A-d-5 | `OwnsData::context(files, configuration)` receives the owner's decoded configuration | `…/data.rs:163-195` | profile, seed and lock are known when the context is built; static rules are refused at load |
| A-d-6 | A configuration's attachments are files under `data/`, handed to `PackConfiguration::seed` only, not to the data context | `worldpack/src/configure.rs:186-250`; `authoring/src/attachment.rs:15-19` | the lock is not an attachment; it lives in an owned directory (F-S30d-1, Q-S30d-2) |
| A-d-7 | The interim sampler: `random.Random(sha256(seed | attempt | world | attribute | key))`, float weights, largest remainder within `given` groups, `allowed` restrictions, a re-draw "attempt" | `origin/mvp0/s26-cast-director:tools/character_pipeline/casting/cast_director.py` | replaced by this sampler (S30-g removes it); its "attempt" is `salt` |
| A-d-8 | The interim profile: `normal` as `[mean, P5, P95]` with SD = (P95 − P5) / 3.29, clipped at ± `clip_sd` SD; `integer_range` uniform; `stratify: true` on sex, age band, ancestry and build only; colour pools per ancestry with an age replacement | `origin/mvp0/s26-cast-director:presentation/mineworld-default/3D/characters/populations/san-diego-county.yaml` | §5.6's `normal` keeps the parameterisation with the exact constant; §5.5's default `stratify` is F-S30d-3 |
| A-d-9 | `PackConfiguration` with `FACTS = []` is admitted (`test-schema`) | `mvp0/pr-s30b:tests/acceptance/tests/schema/mod.rs:139-149` | the configuration states no fact; its identity is carried by `attributes-composed` (S30-c §5.11) |
| A-d-10 | CI `core` runs `cargo test --workspace` on Linux, Windows and macOS | `scripts/ci_layer.py:70-75`; `.github/workflows/ci.yml:130-170` | one committed golden compared on each platform is the three-platform byte comparison (AC-S30-7) with no CI edit |
| A-d-11 | `serde_json` serializes `BTreeMap` in key order; the workspace has no YAML serializer (`serde-saphyr` is `deserialize` only) | `Cargo.toml:84` | `resolve --json` is canonical by construction; the lock writer is a small canonical emitter (§5.9) |

### 3.2 Findings

- **F-S30d-1. Where the lock lives.** `MODULE_SPEC` §4.1 writes `lock: attributes.lock.yaml # relative to the pack root`,
  but the seam reads no root file (S30-b F-b-1 (b)) and attachments do not reach the data context (A-d-6).
  **Q-S30d-2:** the lock is a file of a directory `attributes` owns, `locks/`, in the World Pack; `lock: <name>`
  names `locks/<name>.yaml`; a `locks/` file in an Entity Pack is refused (a lock belongs to one world).
- **F-S30d-2. Restrictions and stratification.** The specified algorithm assigns the stratified multiset by a bounded
  matching and refuses `Unsatisfiable` when none exists. In a small group that is the common case: Alice restricted to
  `{ allowed: [red, auburn] }` in a group of two whose allocation is `[brown, blond]` would refuse the world, though
  her restriction is satisfiable by any draw. **Q-S30d-1** recommends drawing restricted members first, within their
  restriction, and counting them as fixed.
- **F-S30d-3. `stratify` has no stated default, and the guarantee needs it.** `MODULE_SPEC` §4.1's guarantee ("every
  value's realised count differs from its target by less than one person") holds only for stratified attributes; an
  independent draw over 1 000 people is off the marginal by ≈ 1.6 pp at one standard deviation. Decided:
  **`stratify` defaults to `true`**; `stratify: false` opts out of the guarantee and `resolve` marks such attributes
  "unstratified" in its report. Dated note (C8).
- **F-S30d-4. Roles and topological order.** The precedence puts roles before the profile, while the algorithm applies
  them after sampling. Decided: a role targeting attribute X applies **immediately after X is resolved**, to people
  whose X is neither authored nor locked; its `when:` may name tags and attributes earlier than X in the order; a
  role naming a later attribute is refused `BadRole` (new name). `set:` replaces the sampled or default value, `add:`
  unions into it (origin `role`). So every child of X is drawn from X's final value.
- **F-S30d-5. Which combinations "can occur".** `MissingCondition` needs a definition that is static and
  deterministic. Decided: a parent's possible values are those with positive weight in some row of its distribution,
  or its whole domain when it has none (authored-only); every combination of possible values must be covered by a
  row or an `otherwise:`. Checked at load.
- **F-S30d-6. The look key of C-2.** `appearance` (S26-a, #206) is not on `main`. Decided: `resolve --json` keys people
  by authoring key and carries `look` when the build provides `appearance` and the world enables it (read through
  `appearance`'s published projection, in the CLI, the composition root); otherwise `look` is absent and S30-g adds it
  (R-S30g-d-1). No adapter is kept.
- **F-S30d-7. Names the specification does not give.** Refusals stated in prose get names: `DuplicateProfile` (two
  files declaring one profile id), `BadRole` (F-S30d-4), `UnknownPerson` (a lock entry for a key the world does not
  declare, `MODULE_SPEC` §4.1 "a person key the world does not declare is refused naming it"), `NoSeed` (a profile named
  with no `seed`). Dated note in §4.1's refusal table (C8).
- **F-S30d-8. `covers: adults` needs an age.** Decided: `covers` is `all` (default) or `adults`; `adults` checks, after
  resolution, every attribute the catalogue declares `derive: age` at the epoch, refusing `NotCovered` for a person
  under 18. The engine knows the `age` derivation already (S30-c F-S30c-8); it names no slug.

### 3.3 What does not exist

No profile, lock, configuration of `attributes`, sampler, quantile table, `resolve` or `lock` command. The interim
sampler is Python on an unmerged branch.

---

## 4. Reuse, both ways

`DEP-67` decided the sampler is MineWorld's own (pgmpy, pomegranate, PopulationSim and MatrAIx's code declined: float,
Python, another problem). This PR adds no dependency. Two pieces are adopted rather than invented:

- **SplitMix64's finalizer** (Steele, Lea & Flood 2014; Vigna's reference `splitmix64.c`), already the tree's idiom
  (A-d-1).
- **FNV-1a 64** (Fowler, Noll & Vo; offset `0xcbf29ce484222325`, prime `0x100000001b3`; IETF draft
  `draft-eastlake-fnv`) to turn the seed, ids and keys into 64-bit words: eight lines, public domain, platform-free.
- **The standard-normal quantiles** come from Wichura's AS 241 (Applied Statistics 37, 1988), the algorithm of
  CPython's `statistics.NormalDist.inv_cdf`, run **offline** once by `tools/normal_table.py`; the committed integer
  table is the authority and the run never touches a float.

---

## 5. Design

### 5.1 Module layout

```text
authoring/attributes/src/sample/   hash.rs (fnv1a64, mix), micro.rs (Micro, decimal → micro), remainder.rs
                                   (largest remainder), normal_table.rs (generated), normal.rs (interpolation),
                                   order.rs (topological order), weights.rs, allocate.rs, numeric.rs, roles.rs,
                                   resolve.rs (the pipeline, origins), report.rs (realised vs target, support)
authoring/attributes/src/profile/  decode.rs, rules.rs (every static refusal), mod.rs (Profile, frozen)
authoring/attributes/src/lock.rs   Lock, LockEntry, the canonical writer
authoring/attributes/src/config.rs AttributesConfiguration (configure/attributes.yaml)
authoring/attributes/tools/normal_table.py
systems/attributes/src/{system.rs, data.rs, resolve.rs}       wiring
tools/cli/src/attributes.rs                                   resolve, lock
```

### 5.2 Numbers: micro-units, rationals and the one boundary

- **`Micro`** is `i64` in millionths. A probability, a ratio or a numeric parameter in a profile is written as a YAML
  number or a decimal string with at most 6 decimals and decoded **once** by S30-c's exact decoder (its §5.4: shortest
  round-trip text → integer digits); more decimals is `BadDistribution`/`BadAdjustment`. After decoding, nothing is a
  float.
- A quantity parameter (`mean`, `p5`, `p95`, `min`, `max`) is decoded at the attribute's declared scale.
- **Every product and quotient is in `i128`/`u128`**, every division is floor division of non-negative numerators,
  and every "rounding" is named: `round_half_away(n, d)` = `(2n + sign(n)·d) div (2d)` toward zero. Overflow is
  impossible for the bounds used (≤ 10⁶ people, weights ≤ 10⁶ × (10⁶)^k with k ≤ 4 adjust entries matching one row,
  u128 holds 3.4 × 10³⁸) and is checked by `checked_mul` with a `BadAdjustment { reason: "too many matching ratios" }`
  refusal beyond k = 4.

### 5.3 Words of randomness

```text
s       = mix(fnv1a64(seed utf-8), salt as u64)          the world's stream
a(X)    = fnv1a64(qualified id of X)
p(m)    = fnv1a64(person key of m)
g(G)    = fnv1a64("<parent₁>=<value₁>;<parent₂>=<value₂>")   parents in `given` order; "" for no parent
order(X, m)        = mix(mix(s, a(X)), p(m))              a member's place in X's assignment order
tie(X, G, v)       = mix(mix(mix(s, a(X)), g(G)), v)      v = the value's index in the domain
draw(X, m, n)      = mix(order(X, m), n)                  the n-th independent word for m and X
ppm(x)             = ((x >> 32) × 1_000_000) >> 32        uniform in 0 … 999 999 (A-d-2's idiom)
```

Nothing else is random. A name, a note, a location or the run's `--seed` never enters (I-S30-6).

### 5.4 The configuration and the profile format

```yaml
# configure/attributes.yaml
format: 1
profile: test-town        # absent: no sampling
seed: fixture-pop-v1      # required when profile is set (NoSeed)
salt: 0                   # optional, u32
lock: cast                # optional: locks/cast.yaml of this World Pack (F-S30d-1)
```

`backend:` and `mask:` are S30-e's; until then they are unknown fields, refused by name.

```yaml
# populations/<file>.yaml
format: 1
profile: test-town                       # unique among the world's populations/ files (DuplicateProfile)
version: 1
covers: all                              # all | adults (F-S30d-8)
sources:
  survey: { cite: "…", url: "…", read: 2026-10-17, status: read }      # read | verify | authored
distributions:
  attributes-town.sex:   { prior: { female: 0.5, male: 0.5 }, source: survey }
  attributes-town.age-band:                                            # a derived band, sampled as a parent
    prior: { 20s: 0.3, 30s: 0.4, 40s: 0.3 }
    source: survey
  attributes-town.birth:
    given: [attributes-town.age-band]
    uniform_age_within_band: { upper: 90 }                             # upper: the open last band's end
    source: authored
  attributes-town.stature:
    given: [attributes-town.sex]
    normal: { by: { female: { mean: 1.626, p5: 1.510, p95: 1.728 }, male: { mean: 1.758, p5: 1.645, p95: 1.871 } } }
    clip_sd: 2
    source: survey
  attributes-town.hair-colour:
    given: [attributes-town.age-band]
    prior: { otherwise: { black: 0.3, brown: 0.5, blond: 0.2 } }
    adjust: [ { when: { attributes-town.age-band: [40s] }, ratios: { grey: 4.0 } } ]
    stratify: false
    source: authored
  attributes-town.facial-hair:
    given: [attributes-town.sex]
    prior: { female: { none: 1 }, male: { none: 0.55, stubble: 0.15, beard: 0.3 } }
    mask: [ { when: { attributes-town.sex: [female] }, only: [none] } ]
    source: authored
roles:
  - { when: { tags: [barista] }, add: { attributes-town.languages: [italian] } }
```

- **Kinds of distribution** (exactly one per attribute): `prior` (category, level, flag, set — a set's prior is over
  single members and its draw is one member, a set of one; a world wanting more uses roles or authored values),
  `normal`, `uniform { min, max }`, `table { quantiles: { <q ≤ 6 decimals>: <value> } }` (quantity), and
  `uniform_age_within_band { upper? }` (an `instant` given exactly one `band`-derived parent).
- **Rows** are nested maps in `given` order; any level may carry `otherwise:`. A value a row omits has prior 0 and is
  listed by `attributes check` when no row gives it weight. A row sums to 10⁶ ± 1 micro (the specification's 1 ± 10⁻⁶),
  then is normalised to exactly 10⁶ by largest remainder (ties by lower value index).
- **Static rules** (refused at load, at the line and column where they apply): `UnknownAttribute` for any id;
  `DependencyCycle` (the cycle printed, found by depth-first search from the smallest remaining id); `MissingCondition`
  per F-S30d-5; `BadDistribution`; `BadAdjustment`; `BadMask`; `EmptySupport` for any combination of possible parent
  values; `UncitedDistribution` (`source` neither a declared source nor `authored`); `MissingSupport` (a quantity
  distribution whose returnable range is not finite, or the open last band without `upper`); `BadRole`;
  `UnknownProfile`; `DuplicateProfile`; `NoSeed`. A derived attribute other than a `band` may not have a distribution
  (`OutOfDomain`, as stating one is).
- **A sampled band is a parent, never a value.** `age-band` above is drawn to condition `birth`; only `birth` is stored;
  the band is derived from it afterwards and, at the epoch, equals the drawn band (§5.6's `ceil`).

### 5.5 The algorithm

Input (I-S30-6): the frozen catalogue, the profile, the lock, the configuration, and per covered Person the seam's
`Subject` (key, tags, S30-c's `Authored { values, restrictions }`). Output: per person and attribute a value and an
**origin** — `authored`, `ruled`, `locked`, `role`, `sampled { stratum, draw }`, `default`.

1. **Fix first.** A person's authored values (origin `authored`) and lock entries (origin `ruled` or `locked`, as the
   entry says) are fixed and never drawn (I-S30-7). Authored wins over the lock for the same attribute.
2. **Order.** The profile's attributes in topological order of `given`, ties by qualified id (Kahn's algorithm over a
   `BTreeSet`).
3. **For each attribute X in order**, the covered people are grouped by their resolved parent values (group key in
   `given` order). In each group G of n people:
   - **Weights.** `w(v) = prior(v | G) × Π r(v)` over the `adjust` entries whose `when` matches G (u128, ratios in
     micro), set to 0 where a `mask` entry matching G excludes v; normalised to `ppm(v)` summing to exactly 10⁶ by
     largest remainder (`base = w·10⁶ div W`, remainders descending, ties by lower value index). `W = 0` is
     `EmptySupport`.
   - **Restricted members** (Q-S30d-1, recommended): each unfixed member with an `allowed:` or `between:` restriction on
     X, in `order(X, m)` order, draws once within its restriction — categorical: by `ppm` restricted to the allowed
     values and renormalised; numeric: §5.6's quantile drawn uniformly between the restriction's two quantiles — and is
     then counted as fixed. A restriction with zero weight or no overlap with the support is
     `Unsatisfiable { person, attribute }`. *(Alternative if Q-S30d-1 is ruled "as specified": no such pass; the
     bounded matching of step 4 assigns restricted members and refuses `Unsatisfiable` when no matching exists.)*
   - **Stratified** (`stratify: true`, the default, F-S30d-3): with f(v) fixed members holding v and u unfixed
     members, `quota(v) = max(0, n·ppm(v) − f(v)·10⁶)`, `Q = Σ quota` (if `Q = 0`, `quota = ppm`); counts
     `c(v) = u·quota(v) div Q`, and the remaining `u − Σc` go one each to the largest `u·quota(v) mod Q`, ties by
     larger `tie(X, G, v)`, then lower index. Slots are the multiset in domain order; unfixed members sorted by
     `order(X, m)` take slot i in turn. (Specification's step 4, bounded matching, applies only under the alternative.)
   - **Independent** (`stratify: false`): each unfixed member takes the first v whose cumulative `ppm` exceeds
     `ppm(draw(X, m, 0))`.
   - **Roles** targeting X apply now (F-S30d-4).
4. **Numeric attributes** (§5.6) use the same grouping; a stratified member i of u takes quantile `(2i + 1) / (2u)`,
   an independent one `(2·ppm(draw(X, m, 0)) + 1) / (2·10⁶)`.
5. **Defaults.** An attribute still absent takes its declaration's `default` (origin `default`), else stays absent.
6. **Coverage.** `covers: adults` per F-S30d-8.

The `stratum` of a `sampled` origin is the group key; `draw` is i (stratified) or 0.

**Guarantee** (stratified, holds by construction and is tested, AC-d-4): within every group, `|c(v) + f(v) − n·ppm(v)
/ 10⁶| < 1 + f-excess`, where `f-excess` is the fixed members beyond their target; with no fixed members, every count
is within one person of its target.

### 5.6 Numeric draws

- **The quantile table** `Z[k]`, k = 1 … 999: the standard-normal quantile at `k / 1000`, in micro-units (`i32`),
  generated offline by `tools/normal_table.py` (`statistics.NormalDist().inv_cdf`, AS 241, rounded half away from zero
  to micro-units) and committed as `normal_table.rs` with the script's command and the CPython version in its header.
- **`z(a / b)`** for a rational quantile in (0, 1): `N = 1000·a`, `k = N div b`, `f = N mod b`; for `1 ≤ k ≤ 998`,
  `z = Z[k] + round_half_away((Z[k+1] − Z[k])·f, b)`; below `0.001` it is `Z[1]`, above `0.999` it is `Z[999]`. Then
  clipped to `± clip_sd·10⁶` (`clip_sd` an integer 1 … 3).
- **`normal { mean, p5, p95 }`** (A-d-8's parameterisation, the constant exact): the value at the attribute's scale is
  `mean + round_half_away(z·(p95 − p5), 3 289 707)`, where 3 289 707 = 2 × 1 644 853.6 (z₀.₉₅ in micro-units, doubled
  and rounded). Support (C-7) per stratum: `mean ± round_half_away(clip_sd·10⁶·(p95 − p5), 3 289 707)`, clamped to the
  declaration's `min`/`max`.
- **`uniform { min, max }`**: `min + (a·(max − min + 1)) div b`, at most `max`. Support `[min, max]`.
- **`table`**: piecewise-linear between the stated quantiles (q in micro), integer interpolation as above; support the
  first and last values.
- **`uniform_age_within_band`**: for the parent band `[lo, hi)` in years (`hi` = `upper` for the last), `d_lo =
  ceil(lo · 146 097 / 400)`, `d_hi = ceil(hi · 146 097 / 400)` days; `age = d_lo + (a·(d_hi − d_lo)) div b`; `birth =
  −age · 86 400` seconds (before the epoch). With S30-c's `age` (floor of `(at − birth)·400 / (146 097 · 86 400)`),
  the age at the epoch lies in `[lo, hi)` — the `ceil` is what guarantees it (a test checks both edges for every band
  from 0 to 130).

### 5.7 Genesis and the drift check

- `attributes`' context (S30-c) now also decodes `populations/` and `locks/` (directories appended to `DIRECTORIES` in
  that order) and its configuration; it freezes the named profile and the lock and runs every static rule.
- `resolve` (S30-c §5.7) runs §5.5 for the people the profile covers, then defaults; with no profile it is S30-c's path
  unchanged.
- `attributes-composed` gains `population: { profile: { id, version, digest }, seed, salt, lock_digest }`, the digests
  FNV-1a 64 of each file's canonical form (`profile`: its decoded, sorted JSON; `lock`: the writer's bytes). A resumed
  world whose profile, seed, salt or lock changed is refused `ConfigurationDrift` naming `attributes` (S30-c §5.11).
- The sampler runs only at genesis (`ARC-123` item 6); `traits-stated` and `attributes-handed` carry values, never
  origins.

### 5.8 Output: `mineworld attributes resolve`

`mineworld attributes resolve <world> [--json] [--person KEY] [--facet F] [--salt N] [--synthetic N] [--packs DIR]`

```json
{ "schema": 1,
  "world": "attributes-town",
  "profile": { "id": "test-town", "version": 1, "seed": "fixture-pop-v1", "salt": 0, "covers": "all" },
  "sources": { "survey": { "cite": "…", "status": "read" }, "authored": { "status": "authored" } },
  "distributions": { "attributes-town.stature": { "source": "survey", "verify": null, "stratified": true } },
  "people": { "alice": { "look": null, "values": {
      "attributes-town.stature": { "value": { "quantity": "1.660" }, "origin": "authored" },
      "attributes-town.sex":     { "value": { "category": "female" }, "origin": { "sampled": { "stratum": "", "draw": 0 } } } } } },
  "support":  { "attributes-town.stature": { "attributes-town.sex=female": { "min": "1.507", "max": "1.745" } } },
  "realised": { "attributes-town.sex": { "": { "n": 3, "target": { "female": "1.500000" }, "count": { "female": 2 } } } },
  "verify": [] }
```

- One canonical JSON value (`BTreeMap`s, no float: quantities and targets are decimal strings printed from integers;
  `target` is `n·ppm/10⁶` in micro, printed with 6 decimals). The text form prints the same content as aligned lines.
- `--salt` changes `s` only; the header shows it; the world's own population is the one its configuration's `salt`
  gives (`MODULE_SPEC` §4.1).
- `--synthetic N` resolves N synthetic subjects `synthetic-00001` … with no tags and no section and prints the header,
  `realised` and `support` only, plus `worst: { marginal_pp, conditional_pp }` (conditional over groups of ≥ 50).
- `--facet` filters `people[*].values` by the declarations' facets (C-1's companion is S30-c's `catalogue --facet`).
- C-6: every refusal is printed before any output and the exit code is 1.

### 5.9 The lock and `mineworld attributes lock`

```yaml
# locks/cast.yaml (written by the tool; reviewed, never hand-indexed)
format: 1
profile: { id: test-town, version: 1 }
seed: fixture-pop-v1
people:
  alice:
    attributes-town.stature: { value: 1.660, origin: ruled, at: 2026-10-17, by: "casting gate B0" }
```

- Decoded like a person-file value (`UnknownAttribute`, `WrongKind`, `OutOfDomain`, `OutOfRange`; `UnknownPerson`);
  `origin` is `ruled` or `locked`; `at` a `YYYY-MM-DD` string; `by` optional text ≤ 160 bytes. A header that differs from
  the configuration is reported by `check` ("the lock was written for test-town v1"), not refused: pinned values stay
  pinned whatever the profile (AC-S30-8).
- `mineworld attributes lock <world> --person KEY --attribute ID --origin ruled|locked --at YYYY-MM-DD [--by TEXT]
  [--value V]` writes or replaces one entry; without `--value` it pins the value `resolve` gives now. The writer emits
  the file in one canonical form (people and ids sorted; one entry per line; values in the kind's YAML form), so a
  rewrite of an unchanged lock is byte-identical. A world whose configuration names no `lock:` is refused ("name a lock
  in configure/attributes.yaml: lock: <name>"); the command never edits the configuration.

### 5.10 C-1 … C-7, mapped

| S26 needs | Delivered by |
| --- | --- |
| C-1 | S30-c's `catalogue --json --facet appearance` (unchanged here) |
| C-2 | `resolve --json`: `people[key].values[id] = { value, origin }`; `look` per F-S30d-6 |
| C-3 | §§5.3, 5.5; AC-d-6 (X-24) |
| C-4 | the header, `sources`, `distributions` (source, status, `verify`) |
| C-5 | `realised`, for the cast and for `--synthetic` |
| C-6 | §5.4's static rules and §5.5's `Unsatisfiable`, before any output |
| C-7 | `support` per quantity and stratum |

### 5.11 Fixtures

- S30-c's fixture world gains `attributes-town.sex` (category female, male; private), `attributes-town.facial-hair`
  (category none, stubble, beard; perceivable), `attributes-town.grey` value appended to `hair-colour` (a minor
  change), `populations/test-town.yaml` (§5.4's excerpt, complete), `locks/cast.yaml` (one ruled entry), and
  `configure/attributes.yaml`; twelve people instead of three (Alice authored, Bob locked, ten unstated, one
  restricted, one tagged `barista`).
- `tools/cli/tests/fixtures/attributes-harbour/`: the same places, its own `harbour.peoples` category and its own
  profile `harbour-of-elsewhere`, to show a world replaces a profile by its configuration only (step AC-S30-4's
  mechanism; the reference-world half is S30-g's).

### 5.12 Tests that are their own oracle, and the one golden

- **Hand oracles** (test rules §25): largest remainder on stated inputs (e.g. 7 people, 1/3 each → 3, 2, 2 with the tie
  broken as `tie` gives, computed in the test from `mix` constants pinned by SplitMix64's reference outputs);
  allocation with fixed members; FNV-1a 64 against its published test vectors (`""` → `cbf29ce484222325`, `"a"` →
  `af63dc4c8601ec8c`); `mix(0, n)` against the reference generator's first outputs; `Z[975] = 1 959 964`, `Z[950] =
  1 644 854`, `Z[900] = 1 281 552`, `Z[500] = 0`, `Z[k] = −Z[1000 − k]`, strictly increasing; the age-band edges.
- **One determinism golden** (`tools/cli/tests/fixtures/attributes/resolve.json`): the full `resolve --json` of the
  fixture, recorded once at C6 by running the binary and committed with the base SHA in a sibling `SOURCE.md`. It is
  evidence of reproducibility, not of correctness, and is compared on Linux, Windows and macOS by the `core` layer
  (AC-S30-7).

### 5.13 Specification sync (C8)

`MODULE_SPEC` §4.1: `lock:` names a `locks/` file (Q-S30d-2); restricted members drawn first (Q-S30d-1) or the
bounded matching kept; `stratify` defaults to `true` (F-S30d-3); roles after their target (F-S30d-4); "can occur"
(F-S30d-5); `covers` (F-S30d-8); the four new refusal names (F-S30d-7); the data-directory table gains `locks/`;
§8.1 gains `resolve` and `lock`. `DECISIONS.md`: a dated note under `ARC-123` listing these, and the constants
(FNV-1a 64, SplitMix64, the AS 241 table, 3 289 707, the mean Gregorian year).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-d-1 (I-S30-6) | The output is a pure function of catalogue, profile, lock, configuration and the subjects' keys, tags and sections | the type of the input; X-S30-6's rename test; the golden on three platforms |
| I-d-2 | No float is used after decoding | `#![deny(clippy::float_arithmetic)]` on the model crate (S30-c); X-S30-13's plant |
| I-d-3 (I-S30-7) | Authored, ruled and locked values are never drawn | AC-d-7; X-S30-14 |
| I-d-4 | Only a mask zeroes a positive prior; a rare valid combination stays eligible | the weights tests; AC-d-5 |
| I-d-5 | A world without a profile resolves exactly as in S30-c | S30-c's tests unchanged; the fixture with the configuration removed |
| I-d-6 (I-S30-10) | Reference worlds byte-identical | `social_cafe.rs`, the `decisions` golden; one nightly dispatch (C9) |
| I-d-7 | Origins are never saved | a save scan for `origin` keys in `traits-stated`/`attributes-handed` payloads |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy on the model crate, the pack and the CLI | per commit; CI `fast` |
| Unit (oracles) | §5.12's hand oracles; every static refusal at its line and column; weights, masks, adjustments; allocation and assignment; restricted members; numeric draws and supports; roles' order; lock decode and writer round trip | `authoring/attributes/src/{sample,profile}/**/tests.rs`, `lock.rs` tests |
| Unit (fidelity) | AC-S30-5 on 1 000 synthetic subjects of the fixture profile; AC-S30-6 on 10 000 | `authoring/attributes/tests/fidelity.rs` |
| Crate (pack) | genesis with a profile: values in `traits-stated`, origins nowhere; the drift check on a changed seed, profile and lock | `systems/attributes/tests/pack.rs` |
| Integration (real binary) | `resolve --json` golden (AC-S30-7 on three platforms); `--salt`, `--synthetic`, `--person`, `--facet`; `lock` round trip; X-S30-5, X-S30-6, X-S30-12 … X-S30-14; X-24, X-25 | `tools/cli/tests/attributes_resolve.rs` |
| Regression | I-d-6 | unchanged tests; one nightly dispatch |
| Gate 1, Gate 2 | NOT REQUIRED: no model, no rendering, no lifecycle timing | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-d-1 | **Checkpoint.** `resolve --json` on the fixture prints C-1 … C-7's fields (§5.10) for all twelve people and every profiled attribute | S30-d row |
| AC-d-2 | **AC-S30-7.** Two runs give identical bytes; the committed golden equals the output on Linux, Windows and macOS in CI | AC-S30-7 |
| AC-d-3 | **X-24.** `--salt 1` and a new `seed` change only values whose origin is `sampled`; every other value and origin is identical | C-3 |
| AC-d-4 | **AC-S30-5 / X-25.** `--synthetic 1000`: every stratified marginal within 0.5 pp and every conditional within 2 pp for groups ≥ 50; the report's `worst` prints both | AC-S30-5 |
| AC-d-5 | **AC-S30-6.** 10 000 synthetic people: no `female` with a beard under the fixture's mask; a mask leaving no value for a combination is `EmptySupport` naming the attribute and the parents | AC-S30-6 |
| AC-d-6 | **AC-S30-8.** Authored, ruled and locked values survive a new seed and a new profile version (version 2 with changed priors) | AC-S30-8 |
| AC-d-7 | **X-S30-5.** A profile with `a given b`, `b given a` is `DependencyCycle` printing `[a, b, a]` | step-30 §13.5 |
| AC-d-8 | **X-S30-6.** Renaming every person (each `name:`) leaves `resolve --json` byte-identical; the plant (a `Subject`-adjacent read of `name`, through a test-only parameter) turns the rename test red | step-30 §13.5 |
| AC-d-9 | **X-S30-12.** A row summing to 0.9 is `BadDistribution` naming the attribute, the row and `0.900000` | step-30 §13.5 |
| AC-d-10 | **X-S30-13.** The plant replacing `ppm(x)` with `(x as f64 / 2⁶⁴ × 10⁶) as u64` moves the golden (red) | step-30 §13.5 |
| AC-d-11 | **X-S30-14.** The plant drawing over fixed members turns AC-d-6 red | step-30 §13.5 |
| AC-d-12 | **Genesis.** Loading the fixture states `traits-stated` with sampled values equal to `resolve`'s; resuming after changing `seed` is `ConfigurationDrift` naming `attributes` | R-S30d-c-1 |
| AC-d-13 | **Restriction.** The restricted person resolves within the restriction; a restriction to a masked value is `Unsatisfiable` naming the person and the attribute | Q-S30d-1 |
| AC-d-14 | **Profile swap.** The harbour fixture resolves under its own profile with no file outside it changed | AC-S30-4's mechanism |
| AC-d-15 | I-d-6 holds | X-S30-9 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

As S30-c §9.0, from `/Users/yuema137/mineworld-worktrees/impl-s30d`: re-read before editing; steps in order; targeted
tests one at a time; clippy on touched crates; fmt; mutations by copy, plant, red, restore, `cmp`, green; review
answers in §16; `git status` against **Files**; ledger and handoff in the commit; stage by path; no heredoc, `sed -i`
or `python3 -c`. **R-LOAD-1** always.

#### C0 — Base and ledger

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30d.md` (new).
- Steps: record the base; confirm S30-c merged (`mineworld-attributes-model`, `attributes-composed`, the drift check);
  record whether S26-a has merged (F-S30d-6); confirm §15's rulings and apply them to §§5.4, 5.5 before C1.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: §2.3 holds.
- **Commit message:** `docs(plan): S30-d — open the ledger at the base`

#### C1 — Integer primitives and the quantile table

- **Files:** `authoring/attributes/src/sample/{mod.rs, hash.rs, micro.rs, remainder.rs, normal.rs, normal_table.rs}` and
  tests; `authoring/attributes/tools/normal_table.py`.
- Steps: §§5.2, 5.3, 5.6's table and `z(a / b)`; run `python3 authoring/attributes/tools/normal_table.py >
  authoring/attributes/src/sample/normal_table.rs` once (recorded: command, CPython version, wall time).
- Validation: `cargo test -p mineworld-attributes-model --lib sample::`. Mutation **M-d-1** (`round_half_away` rounds
  toward zero) → the interpolation oracle red.
- [ ] Implementation. [ ] Validation: §5.12's oracles. [ ] Review: no `as f64` anywhere in `sample/`; the table's
  header names its generator and source.
- **Commit message:** `feat(attributes-model): integer primitives for sampling — FNV-1a, SplitMix64, micro-units, largest remainder, the AS 241 table (ARC-123, S30-d)`

#### C2 — Configuration and profiles

- **Files:** `authoring/attributes/src/{config.rs, profile/*.rs}` and tests; `systems/attributes/src/{system.rs,
  data.rs}` (`configures!()`, the `populations` directory); `worldpack/tests/data_directories.rs` (the expected list).
- Steps: §5.4 in full, every static rule with its refusal at its position; `attributes-composed.population` (§5.7);
  `NoSeed`, `UnknownProfile`, `DuplicateProfile`, `BadRole`.
- Validation: `cargo test -p mineworld-attributes-model --lib profile::`; `-p mineworld-attributes --test pack`; `-p
  mineworld-worldpack --test data_directories`. Mutation **M-d-2** (the cycle check skipped) → X-S30-5's unit test
  red.
- [ ] Implementation. [ ] Validation. [ ] Review: every refusal names what §4.1's table names; `MissingCondition` per
  F-S30d-5 is static.
- **Commit message:** `feat(attributes): configure/attributes.yaml and population profiles, every rule refused at load (ARC-123, S30-d)`

#### C3 — The sampler

- **Files:** `authoring/attributes/src/sample/{order.rs, weights.rs, allocate.rs, numeric.rs, roles.rs, resolve.rs,
  report.rs}` and tests; `authoring/attributes/tests/fidelity.rs`.
- Steps: §§5.5, 5.6 (per §15's ruling of Q-S30d-1), origins, the report and supports.
- Validation: `cargo test -p mineworld-attributes-model --lib sample::`; `--test fidelity` (AC-d-4, AC-d-5).
  Mutations **M-d-3** (remainder ties by value index only) → the tie oracle red; **M-d-4** (masks applied after
  normalisation) → AC-d-5 red; **M-d-5** (`ceil` → floor in `d_lo`) → the age-edge test red.
- [ ] Implementation. [ ] Validation. [ ] Review: the input type holds no name; restricted members cannot change an
  unrestricted member's assignment when absent (the no-restriction path equals the plain allocation, tested).
- **Commit message:** `feat(attributes-model): the stratified integer sampler — weights, masks, adjustments, numeric draws, roles (ARC-123, S30-d)`

#### C4 — The lock

- **Files:** `authoring/attributes/src/lock.rs` and tests; `systems/attributes/src/data.rs` (the `locks` directory,
  Entity Pack refusal).
- Steps: §5.9's decode and precedence; the canonical writer; the round trip (decode ∘ write = identity on the fixture).
- Validation: `cargo test -p mineworld-attributes-model --lib lock::`. Mutation **M-d-6** (the lock read after the
  profile) → AC-d-6's unit half red.
- [ ] Implementation. [ ] Validation. [ ] Review: authored beats locked; a `locks/` file in an Entity Pack is refused.
- **Commit message:** `feat(attributes): the lock — ruled and locked values pinned, written in one canonical form (ARC-123, S30-d)`

#### C5 — Genesis with a profile

- **Files:** `systems/attributes/src/resolve.rs`, `systems/attributes/tests/pack.rs`.
- Steps: §5.7; AC-d-12's genesis half and its drift half (seed, profile, lock each changed in turn).
- Validation: `cargo test -p mineworld-attributes --test pack`; `-p mineworld-cli --test decisions`; `-p
  mineworld-worldpack --test social_cafe`.
- [ ] Implementation. [ ] Validation. [ ] Review: origins reach no fact (I-d-7); a world without a profile is S30-c's
  path (I-d-5).
- **Commit message:** `feat(attributes): genesis samples from the world's profile, and a resumed world is checked against it (S30-d)`

#### C6 — `mineworld attributes resolve` and `lock`

- **Files:** `tools/cli/src/attributes.rs`, `tools/cli/tests/attributes_resolve.rs`,
  `tools/cli/tests/fixtures/attributes/**` (§5.11), `tools/cli/tests/fixtures/attributes-harbour/**`,
  `tools/cli/tests/fixtures/attributes/{resolve.json, SOURCE.md}`.
- Steps: §§5.8, 5.9's command, F-S30d-6's `look`; record the golden once (command and base SHA in `SOURCE.md`).
- Validation: `cargo test -p mineworld-cli --test attributes_resolve` (AC-d-1, AC-d-2's local half, AC-d-3, AC-d-14).
- [ ] Implementation. [ ] Validation. [ ] Review: no wall-clock read except `lock`'s explicit `--at`; the JSON has no
  float.
- **Commit message:** `feat(cli): mineworld attributes resolve and lock — the resolved cast S26 consumes (C-1 … C-7, S30-d)`

#### C7 — The adversarial plants

- **Files:** `tools/cli/tests/attributes_resolve.rs`, `authoring/attributes/tests/fidelity.rs`.
- Steps: AC-d-6 … AC-d-11, AC-d-13 as tests; each plant made, run red, restored (§16 Mutations).
- Validation: the two test files. Mutations **M-d-7** (X-S30-6), **M-d-8** (X-S30-13), **M-d-9** (X-S30-14).
- [ ] Implementation. [ ] Validation. [ ] Review: each plant is the step's, not a weaker one.
- **Commit message:** `test(attributes): the sampler's planted mutations and the step's acceptance (S30-d)`

#### C8 — Specification sync

- **Files:** `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`.
- Steps: §5.13. Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Implementation. [ ] Validation. [ ] Review: code and text agree; every note dated and sourced.
- **Commit message:** `docs(spec): population profiles and the sampler as implemented (ARC-123, S30-d)`

#### C9 — Close-out

- **Files:** this file (§16), the handoff.
- Steps: push `mvp0/pr-s30d`; `gh pr create`; CI on the exact head on three OSes green (AC-d-2); one `nightly.yml`
  dispatch for AC-d-15.
- [ ] Implementation. [ ] Validation. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-d close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. R-LOAD-1

As S30-c §10: targeted files one at a time; never the workspace; the fidelity test's 10 000-person run is one test
(bounded to a few seconds on Linux CI; if it exceeds 30 s it is split by attribute, never skipped). CI on the exact
head is the gate.

---

## 11. Requirements this PR places on other PRs

- **R-S30e-d-1 (S30-e).** The San Diego County profile uses §5.4's format; its `hair-colour` reproduces the interim
  pools and grey replacement as full conditional rows given `[ancestry, age-band]` (exact, not ratios); `age-band`'s
  last band declares `upper`; every figure names a source; `stratify` is left at its default except where the
  interim drew independently on purpose (none needs to).
- **R-S30g-d-1 (S30-g).** `look` in `resolve --json` once S26-a has merged, if S30-d could not add it (F-S30d-6); the
  casting tool calls `mineworld attributes lock` with `--at` and `--by`.
- **R-S30j-d-1 (S30-j).** A newcomer is resolved with the same words of randomness (§5.3), its group's realised
  counts taken from the current population.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §8.4 | the lock lives in `locks/<name>.yaml` (Q-S30d-2) |
| `step-30-person-attributes.md` §8.5 item 2 | restricted members drawn first (Q-S30d-1), or unchanged if ruled "as specified" |
| `step-26-cast.md` §7.4b.4 | the casting tool writes the lock through `mineworld attributes lock --at --by` (S26's owner) |
| `overall.md` | S30-d frozen / merged |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-d — population profiles and the deterministic sampler
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30d-sampler.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§3, 8, 10, 12, 13); pr-s30c-attributes-pack.md;
                         pr-s30b-authoring-seam.md; step-26-cast.md §7.4b.1; docs/MODULE_SPEC.md §4.1, §8.1;
                         docs/DECISIONS.md ARC-27, ARC-61, ARC-121…ARC-123, DEP-10, DEP-67, DEP-68; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; .structured-coding/standards.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30d, its own, held by one session
BRANCH:                  mvp0/pr-s30d, created from origin/main
START CONDITION:         S30-c merged AND this design frozen AND §15 ruled, then the primary's dispatch
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–9, as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-6, I-S30-7, I-S30-10; §6 I-d-1 … I-d-7; §8 as written
SEQUENCE:                C0 … C9 (§9)
IMPLEMENTER:             Opus-class; reviewer the primary session
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *;
                         python3 authoring/attributes/tools/normal_table.py (once, C1); python3 scripts/check_*.py,
                         ci_layer.py docs; git; gh (pr create/view; workflow run nightly.yml once in C9; never
                         merge); cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1);
                         any float in the sampler after decoding; any edit outside APPROVED SCOPE; any reference-
                         world edit; any model call; a new external dependency
MATERIAL STOPS:          S30-c not merged; a platform producing different bytes (AC-d-2) for a reason integer
                         arithmetic does not explain; AC-S30-5 unreachable with stratification; a needed change to
                         S30-c's frozen types beyond additive fields; a reference world's bytes moving
PLATFORMS:               Linux, macOS, Windows (CI core on all three; the golden is the comparison)
VALIDATION BUDGET:       unit, fidelity and CLI tests unrestricted one file at a time; one nightly dispatch (C9).
                         Gate 1 and Gate 2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30d.md
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
POST-MERGE SYNC OWNER:   primary: step-30, step-26 (with its owner), overall.md; implementation session: this ledger
STOP CONDITION:          S30-d READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-d-1 | A hidden float (a `std` helper, a serde path) makes one platform differ | the clippy deny; the decoder is the only boundary; the golden on three OSes; X-S30-13's plant proves the golden is load-bearing |
| R-d-2 | Stratifying by default surprises a small world (the interim drew most attributes independently) | the guarantee is the step's; a world opts out per attribute; `resolve` marks unstratified attributes |
| R-d-3 | The quantile table's coarseness (1/1000) distorts tails | `clip_sd` ≤ 3 inside the table's range (z₀.₀₀₁ = −3.09); interpolation is linear; the support is reported (C-7) |
| R-d-4 | S26-src2 codes against a JSON shape that changes | §5.8 is the shape; a change after freeze is a material stop for both (step-26 §7.4b.1) |
| R-d-5 | Restricted-first draws (if ruled) bias a small group's realised counts | restricted members are rare (casting rulings go to the lock, which is fixed); the report shows the realised counts |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30d-1 [the algorithm, `ARC-123` item 3]** | A person's `allowed:` or `between:` restriction (F-S30d-2): keep the specification's bounded matching, which refuses the world (`Unsatisfiable`) whenever the stratified allocation happens to lack an allowed value — common in groups of one or two — or draw restricted members first, within their restriction, and count them as fixed? | **Draw restricted members first and count them as fixed.** The world refuses only when the restriction itself is impossible (every allowed value masked or outside the support). The biography still wins, nothing is silently re-drawn, the run stays deterministic, and the stratified guarantee holds for everyone else. `MODULE_SPEC` §4.1 step 2 gets a dated note |
| **Q-S30d-2 [the lock's place]** | `MODULE_SPEC` §4.1 puts the lock at the pack root (`lock: attributes.lock.yaml`), which the seam does not read (F-S30d-1). Move it to an owned directory? | **Yes: `locks/<name>.yaml` in the World Pack, named by `lock: <name>`;** a lock in an Entity Pack is refused. It uses the seam as built (S30-b) and keeps every file `attributes` reads in a directory it owns, as S30-e's backend contract does (Q-S30e-1). The alternative — a seam item for root files — adds a capability for one file |

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C9 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-d-1 … M-d-9 as §9 lists them)

### Deviations and findings

F-S30d-1 … F-S30d-8 (§3.2), recorded at design time.

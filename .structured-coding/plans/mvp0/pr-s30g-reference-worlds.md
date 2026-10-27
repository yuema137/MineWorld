# PR S30-g — Reference worlds adopt attributes, and S26 casting switches to `mineworld attributes resolve`

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-26), draft for the operator's freeze
Author:                 S30-g PR-design session (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s30g,
                        branch docs/pr-design-s30g, planning base origin/main @ 0b2faf2f (#382)
Approved by / evidence: none yet — §15 holds five questions, each with a recommendation
Implementation base:    main (origin/main at C0); C0 records the commit
Start condition:        S30-d (#293), S30-e (#300) and S26-cast (#233) merged — all true on 0b2faf2f; S30-f (#373)
                        merged as well (its R-S30g-f-1 applies); this design frozen; §15 ruled; the primary's dispatch
Execution contract:     §13
Implementer:            Haiku-class sessions from this commit plan, one commit at a time, each commit reviewed by an
                        Opus-class session before the next starts (step-30 §13.1, the S30-g row; the standing
                        "design on Opus, implement on Haiku" rule). C2 and C6 escalate to an Opus implementer after
                        two failed review rounds
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15;
amended) §§8.1, 8.4, 8.6, 10, 11.2, 12, 13.1 (the S30-g row), 13.3, 13.4 (AC-S30-8, AC-S30-10, AC-S30-13) · **Casting
side:** [`step-26-cast.md`](step-26-cast.md) §§7.4a, 7.4b.1–7.4b.5 (C-1 … C-7, X-S26src-10, X-S26src-13),
[`pr-s26-cast-director.md`](pr-s26-cast-director.md) · **Specified by:** `docs/MODULE_SPEC.md` §§4.1 ("Population
profiles", "The lock file", "Overrides"), 8.1; `docs/DECISIONS.md` `ARC-54`, `ARC-91`, `ARC-120` … `ARC-126`, `DEP-67`,
`DEP-68` · **Builds on:** [`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md),
[`pr-s30d-sampler.md`](pr-s30d-sampler.md), [`pr-s30e-builtin-catalogue.md`](pr-s30e-builtin-catalogue.md),
[`pr-s30f-attribute-consumers.md`](pr-s30f-attribute-consumers.md) (all MERGED) · **Parent:** [`overall.md`](overall.md)
· **Binding parents:** `CLAUDE.md` §§1.1, 2, 4 (items 1, 2, 5, 7, 9, 10, 12, 13, 16); the lane-lead charter's rule "no
world-specific hard-coding; every acceptance criterion names a second world or a generated variant" (operator,
2026-10-18).

**Decision ids.** None new. Dated notes only, under `ARC-123` (the resolve output gains `look` and derived values) and
`ARC-54` (how the repository's own commands find the two data packs, as Q-S30g-1 rules). `ARC-127` … `ARC-129` and
`DEP-69` stay reserved for the sibling PRs.

No model is called. No image is generated. No money is spent.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-g row).** "`attributes` and `person-attributes` and a profile in `social-cafe`,
  `market-town`, `lakeside` (not `bodies-yard`); `attributes.lock.yaml` with S26's ruled values; one re-baseline per
  world; S26 casting reads `resolve --json`; interim Python sampler removed."
- **Integration checkpoint (same row).** "`mineworld attributes resolve social-cafe` equals the briefs' ruled values;
  the casting page's distribution table regenerates from S30 output."
- **Adversarial criteria (same row).** "AC-S30-13; X-S26src-13 (ruled survive); only the expected baseline columns
  move."
- **AC-S30-13 (step-30 §13.4).** "S26 casting runs on `resolve --json` with no hard-coded attribute list, and the ruled
  briefs are unchanged" — shown to fail by X-S26src-10 and X-S26src-13.
- **X-S26src-13 (step-26 §7.4b.5).** "After a ruled casting gate, re-resolving with a new seed or salt leaves every
  ruled (locked) value unchanged."
- **Step-30 §8.4.** "The lock is how a casting gate's ruling binds the world. When the operator rules a person at
  casting, the casting tool writes the ruled appearance values into the lock." The lock lives in `locks/<name>.yaml`
  (Q-S30d-2, as merged).
- **Step-30 §8.6.** "The interim Python sampler (A-9) is removed when S30-g switches casting to `resolve --json`. No
  adapter is kept (`CLAUDE.md` §4 rule 12)."
- **Step-30 §11.2.** Adopting attributes "adds genesis facts and moves that world's `facts-300s`, `journal-300s`,
  `sealed-300s` and possibly summaries". A re-baseline re-records the world's section of `scripts/baselines.txt` with a
  `reason S30-g:` line and shows that "only the expected columns moved. A moved summary is a stop, because attributes
  change no behaviour in S30-g." One re-baseline per world, done once, here.
- **Step-30 §13.3.** "S30-g starts when S30-d and S30-e have merged and S26-cast has merged."
- **Requirements placed on this PR by its siblings.**
  - *R-S30g-d-1* (S30-d §11): "`look` in `resolve --json` once S26-a has merged, if S30-d could not add it (F-S30d-6);
    the casting tool calls `mineworld attributes lock` with `--at` and `--by`."
  - *R-S30g-e-1* (S30-e §11): "`social-cafe`, `market-town` and `lakeside` require `person-attributes: "^1"` and
    `populations-us`, and name `profile: san-diego-county`; `verify` marks on data figures block the release
    (Q-S30-5)."
  - *R-S30g-f-1* (S30-f §11): "A reference world adopting attributes moves its observations (schema 2 blocks) but no
    cognition cassette until a consult happens with a block present: S30-g re-records the cassettes it moves with
    reasons … `title:` is added to the reference worlds' jobs there, not here."
- **Rulings that bind this PR.** F-CAST-2 (Lakeside uses the San Diego County profile); F-CAST-3 (the seven Lakeside
  sexes, with the follow-up "state this in the person files"); F-CAST-7 (no census category in any prompt); F-CAST-8
  (template v2 retained, nothing of v3 kept); the casting picks of 2026-10-15 (`cast/PICKS.yaml`); Q-S30-5 (a release
  refuses a reference world whose profile carries a `verify` mark on a data figure).
- **The charter (2026-10-18).** No world-specific hard-coding: a mechanism that only makes the default town pass is
  rejected; every acceptance criterion names a second world or a generated variant. Speed first: a PR merges when its
  `fast` check is green; full tests run at milestones.
- **R-LOAD-1.** Locally only the targeted test files of a commit; heavy jobs through
  `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`; never `cargo test --workspace`; CI on the exact head is
  the gate.

---

## 1. Goal

Make the three reference worlds **carry their people's attributes as world facts**, so that what the operator ruled at
the casting gate is what the world, the agents, the clients and the casting tool all read, from one resolution:

1. **The worlds adopt.** `social-cafe`, `market-town` and `lakeside` enable `attributes`, require `person-attributes` and
   `populations-us`, name the San Diego County profile, and pin every ruled value in a lock. `bodies-yard` does not
   change.
2. **The ruling binds the world.** Every value the operator ruled on 2026-10-15 for the 21 cast people, and the accepted
   protagonist's, is an `authored` or `ruled` value in the world. A new seed, a new salt or a new profile version never
   re-draws it.
3. **Casting reads the world.** `cast_director.py` stops sampling. It reads `mineworld attributes resolve --json` for
   each world of a cast, writes the briefs and the casting page from that output, and writes rulings back with
   `mineworld attributes lock`. It names no attribute id in code. The interim profiles and the interim sampler are
   deleted.
4. **Nothing ruled moves.** The 42 committed prompt files stay byte-identical, the picks check still passes, and every
   brief's values equal the ruled values under one hand-written vocabulary table.
5. **One re-baseline per world**, showing that only the expected keys and summary lines moved.

---

## 2. Scope

### 2.1 In scope

1. **The resolve output** (`tools/cli/src/attributes_resolve.rs`; the model crate only if a helper must become public):
   each person's `look` from `appearance`'s section (R-S30g-d-1), and each derived value (`age-years`, `age-band`) with
   origin `derived` and the plain attributes it derives from.
2. **Pack roots for the repository's own commands** (Q-S30g-1): whatever the ruling selects — by recommendation,
   `.cargo/config.toml` `[env]`, the Python test packages' `conftest.py`, the commands in `README.md` and
   `docs/HUMAN_REVIEW_QUEUE.md`, and the release bundle.
3. **The world tiler** (`tools/worldgen/tile.py`): copies `locks/*.yaml` with each person key renamed per block, and a
   World Pack's own `attributes/` and `populations/` directories unchanged.
4. **The three reference worlds**: `world.yaml` (`requires`, `systems`, `configure`), `configure/attributes.yaml`,
   `locks/cast.yaml`, the people files (`attributes:` sections), `market-town`'s three job titles, the worlds'
   `README.md`.
5. **The casting tool**: `tools/character_pipeline/casting/cast_director.py` (sampler removed, resolve consumed, `rule`
   and `--find-salt` added), `tools/character_pipeline/mwcast/brief.py` (brief format 2), the Presentation Pack's
   `characters/casting.yaml` (format 2), `characters/style/phrases.yaml` (new), `characters/style/template.yaml`
   (placeholders only), `characters/cast/casting_inputs.json` (biography moves to the worlds), the 21 regenerated
   `brief.yaml` files and `CASTING.md`; the interim `characters/populations/` directory deleted.
6. **Tests and baselines**: the tests that pin the three worlds' genesis; a new reference-attributes test; the casting
   tool's tests; `scripts/baselines.txt`; the `decisions` golden.
7. **Specification sync** (dated notes): `docs/MODULE_SPEC.md` §§4.1, 8.1; `docs/DECISIONS.md` notes under `ARC-123` and
   `ARC-54`; `tools/character_pipeline/PIPELINE.md` (the casting section).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `casting.yaml` as step-26 §7.4b.2's JSON format (`source`, `style`, `control`, `band`, `licence_policy`, `method_exception`); `--pack`; the unsupported-value and `substitutions` checks; the G-4/G-12/G-13 bindings in `standard.py`; the appended generic prompt line for an attribute the template does not name; X-S26src-9, -11, -12, -14, -15 | S26-src2 (Q-S30g-2) |
| Authoring pronouns, chronotypes or any non-casting attribute for the reference people | a later content PR; nothing reads them in S30-g |
| A restriction on a derived band that a profile samples (the interim `allowed: age_band` lists) | a model follow-up (F-S30g-9, R-S30d-g-1) |
| Re-reading the CHIS county build figure (`verify` on `core.build`) | the release checklist (Q-S30g-4) |
| `bodies-yard`; the fixture worlds under `tools/`; `docs/guide/examples/corner-shop`; the `new-world` template | never in S30-g (§3.4) |
| The MatrAIx demo world | S30-h |
| Runtime sampling of a newcomer | S30-j |
| The wardrobe recipes that copied brief values (`tools/character_pipeline/recipes/*.yaml`) | unchanged: they are their own records and cite the brief files, whose values do not change |
| Any change under `kernel/`, `contracts/`, `server/`, `systems/` | never |

### 2.3 Dependencies

- **Merged and required:** S30-c #280 (the pack, the lock decoder), S30-d #293 (`resolve`, `lock`, the sampler), S30-e
  #300 (`person-attributes` 1.x, `populations-us` with San Diego County v3), S26-cast #233 (the ruled briefs and
  picks), S26-a #206 (`appearance`, the `look` keys).
- **Merged and used:** S30-f #373 (the consumers that make adoption visible; `employment.occupation` and `title:`).
- **Open PRs that touch nearby files (C0 re-checks):** #498 (release dry run; `scripts/ci_godot.py`), #426 (S31-h
  haggling; market-town's history moves when it merges), #366 (S29-f; 2D client). None edits `worlds/`,
  `presentation/mineworld-default/3D/characters/`, `tools/worldgen/` or `scripts/baselines.txt` at their current heads.
  A merge to `worlds/` before C7 means C7 records again.

---

## 3. Audit (`origin/main @ 0b2faf2f`, 2026-10-26)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-g-1 | No reference world enables `attributes`; none has `configure/attributes.yaml` or `locks/`. `social-cafe` and `market-town` have no `requires:`; `lakeside` requires `acme-fishing`, `modern-goods` and the two default Presentation Packs | `worlds/*/world.yaml`; `ls worlds/*/configure` | adoption adds the first `requires:` to two towns (F-S30g-1) |
| A-g-2 | A world's requirements resolve only in the build and in the roots named by `--packs` and then `MINEWORLD_PACKS`; "nothing else: not the world's own directory, not the working directory" | `packages/src/roots.rs:13-56`; `ARC-54` point 3 | after adoption, `mineworld run worlds/social-cafe` without a root is refused (F-S30g-1) |
| A-g-3 | Lakeside's tests and commands already name roots: Rust `tools/cli/tests/lakeside/mod.rs` `roots()`; Python `clients/agent/tests/realserver.py` `lakeside()` passes `--packs entities --packs presentation/mineworld-default`; `scripts/ci_parity.py` passes the same `PACK_ROOTS` for every world; the tests that assert refusal call `env_remove("MINEWORLD_PACKS")` (`requirements.rs:17`, `entity_packs.rs:18`) | as named | `ci_parity` already works; 125 files run the two towns without a root (`grep -rl` over `tools/cli/tests`, `worldpack/tests`, `tests`, `cognition`, `clients/agent/tests`, `sdk`, `scripts`, `tools/worldgen`, `tools/onboarding`) |
| A-g-4 | The release bundle ships `worlds/social-cafe`, `worlds/market-town` and `worlds/bodies-yard` and no Entity Pack | `scripts/package.py:60, 436-452` | the bundle's two towns would not load (F-S30g-1; R-S23-g-1) |
| A-g-5 | `resolve --json` writes `"look": null` for every person, and lists only stored values: a derived attribute (`age-band`, `age-years`) is absent although its strata are used | `tools/cli/src/attributes_resolve.rs:219`; `tools/cli/tests/fixtures/attributes-sampled.resolve.json` (Alice has `birth`, no `age-band`) | C2 adds `look` (R-S30g-d-1) and derived values (F-S30g-3) |
| A-g-6 | `resolve` keys people by authoring key (`alice`); the cast keys people by look (`alice-moreau`); every person in the three worlds has `appearance: { look: … }`; the café cast's look keys are the same in `social-cafe` and `market-town` | `worlds/*/people/*.yaml`; `characters/cast/` | the director maps look → person through C2's `look` |
| A-g-7 | The interim director: a YAML-subset reader; `cast_world` samples with `random.Random(sha256(seed|attempt|world|attribute|key))`, floats and largest remainder; a distinctness loop over `attempt`; `brief_yaml`, `casting_md`, `fill` (placeholders `{attr}`, `{attr|label}`, `{attr|label.key}`); `--check` re-derives 64 files and holds the picks | `tools/character_pipeline/casting/cast_director.py` (635 lines) | the sampler and the attempt loop go; the writers stay, reading resolve's output |
| A-g-8 | `casting.yaml` names `seed`, `max_attempts` and, per cast, `profile` and `worlds`; distinctness categories and table columns name interim attribute names | `presentation/mineworld-default/3D/characters/casting.yaml` | format 2 drops the profile and the seed (step-26 §7.4b: "the casting config therefore names no profile and no seed") |
| A-g-9 | `casting_inputs.json` holds, per person, `fixed` (19 people: `sex` only, from the biography; `wes-calloway`: ten values from the 2D identity source), `allowed` (`age_band` for 8 people, `hair_style` for 2, `build` for 1) and `fields`; `fixed_members.cafe-town` holds `vera-lindgren` with eleven values | `characters/cast/casting_inputs.json` | `fixed` moves to the people files as `authored`; `allowed` moves where the model can hold it (F-S30g-9); Vera's values move to the lock (§5.3) |
| A-g-10 | The 21 briefs and both casts are attempt 0 (salt 0), seed `mineworld-s26-cast-v1`; the picks (`cast/PICKS.yaml`, operator 2026-10-15) name the 21 people; Vera has a directory but no brief | `cast/*/brief.yaml`; `cast/CASTING.md` lines 7, 124; `cast/PICKS.yaml` | the worlds' configuration uses that seed and salt 0 |
| A-g-11 | The ruled brief values, aggregated: sex 10 F / 11 M; `apparent_age` integers 21 … 82; ancestry `hispanic_or_latino` 8, `nh_white` 8, `nh_asian` 2, `nh_black` 1, `nh_multiracial_or_other` 2; `stature_m` two decimals; `skin_mst` 1 … 7; hair colours include `dark brown` and `reddish brown`; hair styles are the interim phrases (`high top knot`, `short tousled crop`, …); facial hair `not_applicable` for every woman | `grep` over `cast/*/brief.yaml` | Appendix A maps each to a `core` slug; three values have no exact slug (F-S30g-6) |
| A-g-12 | San Diego County v3 maps the interim vocabulary in its header (ancestry, `80_plus`, MST, `short trimmed beard`, female `not_applicable` → `none`) and keeps the interim pools by sex for hair style in the interim order | `entities/populations-us/populations/san-diego-county.yaml:1-15, 195-201` | Appendix A extends that mapping; nothing in it is new judgment except F-S30g-6 |
| A-g-13 | `core.natal-sex` and `core.ancestry` carry facet `casting` only; `core.birth` and `core.age-years` carry `prompt, decision`; `core.age-band` `casting, observed`; the appearance group carries `appearance, casting, …` | `entities/person-attributes/attributes/{background,appearance}.yaml` | casting selects by facet `casting`, not `appearance` (F-S30g-4) |
| A-g-14 | A derived attribute cannot be stated or restricted in a person file (`OutOfDomain: nothing states a derived value`), nor pinned by a lock (`OutOfDomain: '…' is derived; nothing pins it`) | `authoring/attributes/src/section.rs:98-103`; `lock.rs` (the `Mutability::Derived` refusal) | an age is ruled by locking `core.birth`; an age-band restriction cannot move (F-S30g-9) |
| A-g-15 | `age(birth, at)` = whole years, floored, over 400 mean Gregorian years (146 097 days); the sampler's `uniform_age_within_band` writes `birth = −age_days × 86 400` at the epoch `WorldTime::EPOCH` (0) | `authoring/attributes/src/derive.rs:19-27`; `sample/resolve.rs:648-663` | §5.3's formula gives each ruled apparent age exactly |
| A-g-16 | Lock entries are `{ value, origin: ruled | locked, at, by? }` per person key and attribute; `mineworld attributes lock <world> --person --attribute --origin --at --by --value` writes one entry canonically into `locks/<name>.yaml` | `authoring/attributes/src/lock.rs`; `tools/cli/src/attributes_resolve.rs:305-414`; `tools/cli/src/main.rs:380-402` | C5 writes the reference locks only through this command |
| A-g-17 | A world may override a `core` declaration's `label`, `phrase`, `words` and narrow its visibility, but never its values (`OverrideNotAllowed`) | `docs/MODULE_SPEC.md` §4.1 *Overrides* (lines 1470-1486) | a ruled value outside the core domain cannot be added by a world (F-S30g-6) |
| A-g-18 | `brief.py` accepts top-level keys `[a-z_][a-z0-9_]*` and nested keys of `casting:` and `sources:`; `CASTING_KEYS = (world, profile, template_version, seed, attempt)`; every `sources` key must be a top-level key | `tools/character_pipeline/mwcast/brief.py:15-80` | qualified ids need format 2 of the brief (§5.5.5) |
| A-g-19 | The template's person blocks use `{sex|label.noun}`, `{apparent_age}`, `{stature_m}`, `{build|label.article} {build|label.words}`, `{age_band|label.posture}`, `{skin_mst|label}`, `{skin_mst}`, `{face_shape}`, `{eyes}`, `{nose}`, `{lips}`, `{hair_colour}`, `{hair_texture}`, `{hair_style}`, `{facial_hair|label}` and the input fields; the words live in the interim profile's `labels:`; `facial_hair`'s label is `''` for `not_applicable` and `Clean-shaven.` for `none` | `characters/style/template.yaml:101-125`; `characters/populations/san-diego-county.yaml:63-124` | the words move to `style/phrases.yaml`; the facial-hair line needs words chosen by another attribute (F-S30g-7) |
| A-g-20 | CI runs `cast_director.py --config … --check` in the `python` layer, after `cargo build -p mineworld-cli`, on three OSes | `scripts/ci_layer.py:331-336, 340-357` | the director may call `target/debug/mineworld` in CI |
| A-g-21 | The tiler copies `world.yaml`, places, items, people and `configure/*.yaml`; it refuses a configure file that holds a reference; it does not copy `locks/`; in block *b* every person key becomes `<key>-<bb>`. `test_tile.py` tiles `social-cafe` (24) and runs it through the real binary; the memory scale gate tiles it to 100 | `tools/worldgen/tile.py:246-415`; `tools/worldgen/tests/test_tile.py:278, 350`; `cognition/lm-controller/tests/scale_harness.py:157-183` | after adoption the tiled world names a lock whose person keys no longer exist (F-S30g-8) |
| A-g-22 | `market-town` has three jobs (`alice` at `cafe-company`, `felix` at `corner-store`, `ivan` at `station-news`), none with `title:`; `social-cafe` and `lakeside` do not enable `employment` | `worlds/market-town/people/{alice,felix,ivan}.yaml` | R-S30g-f-1's titles are three lines |
| A-g-23 | S26-a's adoption precedent moved: `commands.rs`, `content_kinds.rs`, `decisions.rs` and its golden, `inspect.rs`, `server_command.rs`, `social_composition.rs` (an AC-2 test and two composition strings), `worldpack/tests/social_cafe.rs`, `town_bodies.rs` (a genesis count), and the baselines with "every summary +1 line" | `git show --stat 0466441e 5964e5ce 0b3ddab0 f02b777f`; `07729aff` | C5 and C7 follow the same list; a new fact type adds one summary line, which is expected, not a stop (F-S30g-5) |
| A-g-24 | `scripts/baselines.txt` was last re-recorded for `social-cafe`, `market-town` and `bodies-yard` at milestone F (`f92e3cad`); S33-b re-recorded `lakeside` only, and wrote "the other worlds' drift since milestone F is main's"; the three latest `nightly.yml` runs on `main` concluded `failure` | `scripts/baselines.txt`; `gh run list --workflow nightly.yml` | S30-g's movement must be measured against a base record, not against the file (F-S30g-5) |
| A-g-25 | The S30-f ledger classified `decisions`'s `with_no_decision_list_run_prints_what_the_base_printed` as failing on `main` before S30-f (golden last regenerated at `d354b3a2`) | `pr-s30f-attribute-consumers.md` §16 Evidence | C0 re-classifies on the base; C1 re-records it on the base if it still fails |
| A-g-26 | `milestone_d`'s cassette replays from a logged decision input, and a log written before S30-f replays to the request it recorded; no committed cassette is replayed against a live reference world | `cognition/lm-controller/tests/test_milestone_d_replay.py:1-35`; S30-f I-f-2 | R-S30g-f-1 is met by showing that no cassette moves (AC-g-12); if one does, it is re-recorded with a reason |

### 3.2 Findings

- **F-S30g-1. The two towns gain their first requirements.** Adoption means `requires: { person-attributes, populations-us
  }` (R-S30g-e-1), and by `ARC-54` a requirement resolves only in a named root (A-g-2). Lakeside already lives with this
  (A-g-3); the towns are run without a root in about 125 test and script files, in the README, in the review queue and
  in the release bundle (A-g-4). **Q-S30g-1** recommends one root through `ARC-54`'s own variable for the repository's
  developer commands, explicit roots in the documented user commands, and the two packs in the bundle.
- **F-S30g-2. `look` is still null in `resolve`** (A-g-5). R-S30g-d-1 applies: S26-a has merged. The CLI is the
  composition root and already depends on `mineworld-appearance` (`tools/cli/Cargo.toml:64`).
- **F-S30g-3. Casting needs derived values.** The prompt says "apparent age about 46" and the distinctness rule compares
  age bands; both are derived from `core.birth` (A-g-13, A-g-15). A casting tool that computed them would re-implement
  the derivation. Decided (D-g-1): `resolve` lists each derived value with origin `derived` and `from` (the plain
  attributes it is computed from, transitively), so a consumer reads the age and knows what to lock to rule it.
- **F-S30g-4. The casting facet is `casting`, not `appearance`.** Step-26 §7.4b.1 C-1 and step-30 §10 say `--facet
  appearance`, but sex, ancestry and age carry only `casting` (A-g-13). Decided (D-g-2): the director selects the brief's
  attributes by facet `casting`; step-26 and step-30 get a dated note (§12).
- **F-S30g-5. "A moved summary is a stop" needs one exception.** S26-a's adoption added one summary line per new fact
  type (A-g-23): a summary prints counts by type. `traits-stated` is a new type, so each adopting world's summaries gain
  exactly its line(s). Decided (D-g-3): every summary line present on the base is unchanged; the only added lines are
  `traits-stated` lines; anything else is the stop. And because main's records have drifted since milestone F (A-g-24),
  the comparison is between two long records — the base's and the head's — made by this PR, not against
  `scripts/baselines.txt`.
- **F-S30g-6. Four ruled values do not fit the core vocabulary exactly.** Wes's hair colour `reddish brown` and hair
  style `short tousled crop`, and Vera's hair style `messy high updo with face-framing strands`, have no `core` slug, and
  a world cannot add values (A-g-17). Vera's age was ruled as a band (`20s`) with no number, and a band cannot be
  pinned (A-g-14). **Q-S30g-3** recommends the nearest slugs (`auburn`, `short-crop`, `top-knot`), Vera's birth at 25
  (the band's middle), and the exact words kept for Wes's prompt as per-person words in the casting inputs.
- **F-S30g-7. One prompt line depends on two attributes.** The interim wrote nothing for a woman's facial hair
  (`not_applicable`) and `Clean-shaven.` for a man's `none`; in `core` both are `none` (A-g-12, A-g-19). To keep ten
  prompts byte-identical, a phrase may be chosen by another attribute's value (`given:`, §5.5.3). This is data, not code
  naming an attribute.
- **F-S30g-8. The tiler drops the lock** (A-g-21). After adoption a tiled `social-cafe` names `lock: cast`, whose keys
  (`alice`) no longer exist in the tiled world (`alice-01` …). The lock file format keys people by authoring key
  (`MODULE_SPEC.md` §4.1), so the tiler renames them by the same rule it applies to people files.
- **F-S30g-9. A biography's age restriction cannot move.** Eight people had `allowed: age_band` (A-g-9). The model
  refuses a restriction on a derived attribute (A-g-14). For the reference cast it changes nothing, because every
  person's birth is locked. A re-cast of an unlocked person would lose it. Recorded as a model follow-up (R-S30d-g-1);
  the `hair_style` and `build` restrictions move to the people files.
- **F-S30g-10. The verify mark.** San Diego County v3 keeps `verify` on `core.build` (national NHANES, county figure
  unread twice). Q-S30-5 makes a release refuse a reference world whose profile carries a data `verify` mark. In the
  reference worlds every build is ruled, so the unread figure decides no value. **Q-S30g-4** recommends carrying the mark
  to the release checklist rather than changing the profile here.
- **F-S30g-11. The S26-src2 overlap.** Step-26 assigns "consumes `resolve --json`" and `style/phrases.yaml` to S26-src2,
  and step-30 assigns "S26 casting reads `resolve --json`" to S30-g. Removing the interim profile removes the prompt
  words with it, so the switch cannot ship without `phrases.yaml`. **Q-S30g-2** recommends that S30-g ships the switch
  with the phrases file, and S26-src2 keeps the rest of its row.

### 3.3 What does not exist

No reference world has an attribute. No lock exists outside a test fixture. Nothing calls `mineworld attributes lock`
from the casting tool. No casting output depends on S30's sampler. No test compares the ruled briefs with a world.

### 3.4 Worlds on `main`, and which adopt attributes

| World | Where | Adopts in S30-g | Why |
| --- | --- | --- | --- |
| `social-cafe` | `worlds/` | **yes** | step-30 §13.1 |
| `market-town` | `worlds/` | **yes** | step-30 §13.1; also gains three job titles (R-S30g-f-1) |
| `lakeside` | `worlds/` | **yes** | step-30 §13.1; San Diego County by F-CAST-2 |
| `bodies-yard` | `worlds/` | no | step-30 §13.1 ("not `bodies-yard`"); its twelve test persons have no look and stay the fallback path |
| a tiled `social-cafe` (24, 100 people) | generated by `tools/worldgen/tile.py` at test time | **inherits** | the generated variant of AC-g-10 |
| `attributes-backends`, `attributes-builtin`, `attributes-consumers`, `attributes-sampled`, `attributes-town` | `tools/cli/tests/fixtures/` | already enabled | the S30-c … S30-f fixtures; unchanged |
| `commute-town` | `tools/cli/tests/worlds/` | no | a test world for another step |
| `corner-shop` | `docs/guide/examples/` | no | the guide's world; the guide does not teach attributes yet |
| `harbour-village` | `tools/onboarding/tests/fixtures/` | no | an onboarding fixture |
| `new-world` template | `tools/cli/templates/` | no | `mineworld create` output; unchanged |
| `matraix-demo` | not on `main` | — | S30-h |

There is no market-town variant on `main`; the second world of every criterion below is the other town of the café
cast, Lakeside, the tiled variant, or the casting fixture world of §5.5.8.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

- **Adopted, already in the tree:** S30-d's sampler and `lock` writer (the only writer of a lock); `mineworld attributes
  resolve` and `catalogue` as the casting tool's data source (step-26 C-1 … C-7); `ci_parity.py record/diff/baseline`
  for the re-baseline; the tiler's own rename rule for the lock; `ARC-54`'s `MINEWORLD_PACKS` for roots (Q-S30g-1).
- **Considered and declined:** keeping the Python sampler behind an adapter (step-30 §8.6 forbids it); a Jinja2 template
  for prompts (the template is a format string over words, and a template language would admit logic into data; the
  `given:` phrase choice is one level of lookup); PyYAML in the director (it keeps its YAML-subset reader, which its
  files already follow; the resolve output is JSON and read with `json`); bundling the two data packs into the build
  (S30-e made them data packs, "never bundled", `ARC-54`).
- **Not forced:** nothing here adopts a framework's execution model.

---

## 5. Design

### 5.1 What each reference world gains

**`world.yaml`** (each of the three):

```yaml
requires:                       # social-cafe and market-town: a new block; lakeside: two lines added
  person-attributes: "^1"
  populations-us: "^1"
systems:
  - …                           # unchanged, in order
  - attributes                  # appended last, so every reduction order above stays as it was
configure:
  - …                           # unchanged
  - attributes
```

Each addition carries a comment in the file's style: what the pack owns, that it is appended, and what leaving it out
does ("leave it out, with `configure/attributes.yaml`, `locks/` and every `attributes:` section, and people have no
attributes and everything else stays as it was").

**`configure/attributes.yaml`** (identical in the three worlds):

```yaml
format: 1
backend: mineworld
mask: core
profile: san-diego-county       # Lakeside too: operator ruling F-CAST-2, 2026-10-15
seed: mineworld-s26-cast-v1     # the population seed the cast was drawn with (S26, 2026-10-15)
salt: 0                         # the casting attempt that was ruled (CASTING.md: attempt 0)
lock: cast                      # locks/cast.yaml: the casting gate's rulings
```

**People files** gain an `attributes:` section holding only what a biography fixed (origin `authored`):

- every person whose casting input fixed `sex` from the biography: `natal-sex: <female|male>`; the seven Lakeside people
  of F-CAST-3 with a comment citing the ruling (this closes F-CAST-3's follow-up);
- `wanderer.yaml` in both towns: Wes's ten identity-source values (Appendix A, row "wes-calloway"), with a comment citing
  the 2D identity source (Q-S26-9);
- the biography restrictions that the model can hold: `hair-style: { allowed: [...] }` for `alice` (both towns) and
  `alba`, `build: { allowed: [...] }` for `dev` (both towns), slugs from Appendix A.

**`locks/cast.yaml`**, written only by `mineworld attributes lock` (C5), holds every other ruled value: per cast person
the attributes of Appendix A with origin `ruled`, `at: 2026-10-15`, `by: "S26 casting gate (characters/cast/PICKS.yaml,
operator 2026-10-15)"`; for Vera (`visitor`) the same with `by: "VIS-3D-GODOT-1 (accepted protagonist)"`. The two towns'
locks are identical; Lakeside's holds its ten people.

**`market-town` jobs** gain `title:` (R-S30g-f-1), each obvious from the employer and the person's tags (D-g-4): `alice`
`barista`, `felix` `shop assistant`, `ivan` `newspaper seller`.

**READMEs** (`worlds/*/README.md`): one line each naming the new pack and the lock.

### 5.2 The vocabulary mapping (Appendix A)

Appendix A is the one hand-written table from the interim brief vocabulary to `core` slugs. It extends S30-e's header
mapping (A-g-12) and adds nothing else except F-S30g-6's four entries, which Q-S30g-3 rules. It is used twice: by the
implementer to write the people files and the lock (C5), and by the oracle test that compares the world with the
committed interim briefs (C0 copies the interim values into a fixture before C6 rewrites the briefs).

### 5.3 A ruled age is a locked birth

A ruled apparent age *a* (years) becomes the birth instant

```text
birth(a) = −(⌈a × 146 097 / 400⌉ + 182) × 86 400        seconds, at the epoch (WorldTime 0)
```

that is, half a year into the person's *a*-th year. By A-g-15, `age-years` at the epoch is exactly *a*, and `age-band`
follows from it. Example: Alice, 46 → ⌈16 801.155⌉ + 182 = 16 984 days → `birth: -1467417600`; age = ⌊16 984 × 400 /
146 097⌋ = 46. Vera (band only, Q-S30g-3): *a* = 25.

### 5.4 The resolve output gains `look` and derived values (`tools/cli/src/attributes_resolve.rs`)

```json
"people": {
  "alice": {
    "look": "alice-moreau",
    "values": {
      "core.age-band":  { "value": { "level": "40s", "rank": 4 }, "origin": "derived", "from": ["core.birth"] },
      "core.age-years": { "value": { "quantity": "46" },          "origin": "derived", "from": ["core.birth"] },
      "core.birth":     { "value": { "instant": -1467417600 },    "origin": "ruled" },
      "core.stature":   { "value": { "quantity": "1.610" },       "origin": "ruled" } } } }
```

- **`look`**: the person's `appearance` section's look key, read in the CLI as `authored()` reads the `attributes`
  section (downcast to `mineworld_appearance`'s section type), or `null` when the world does not enable `appearance` or
  the person has no section.
- **Derived values**: for each declaration with `mutability: derived` whose value the person's plain values determine,
  the value at `WorldTime::EPOCH`, origin `"derived"`, and `from`: the sorted plain attribute ids it is computed from,
  followed transitively (`age-band` → `age-years` → `birth` gives `["core.birth"]`). Computed with the model crate's
  `derive::value`, the same function the sampler's strata and the System use; if it is not public, C2 makes the
  narrowest public wrapper in `mineworld-attributes-model` and nothing else there changes.
- `--facet` filters derived values like any other. `--synthetic` output is unchanged. `schema` stays 1: both changes add
  fields a strict reader of schema 1 did not have, and the only consumer is the casting tool, changed in the same PR
  (a dated note under `ARC-123`).
- `tools/cli/tests/fixtures/attributes-sampled.resolve.json` (S30-d's golden) is edited by hand: each person gains the
  derived entries the fixture's `age-years` and `age-band` declarations give, and the reviewer checks each against
  `derive.rs`'s formula.

### 5.5 The casting tool switches (`tools/character_pipeline/casting/cast_director.py`)

#### 5.5.1 What it reads

| Input | Holds | Changed how |
| --- | --- | --- |
| `characters/casting.yaml`, `format: 2` | `template`, `phrases`, `inputs`, `output_dir`, `picks`, `packs` (roots, relative to the repository), `casts.<id>.worlds`, `casts.<id>.fixed` (a map from each look counted but never briefed to its casting-input fields, such as `outer`), `distinctness`, `table` | `seed`, `max_attempts` and each cast's `profile` removed; `worlds:` renamed `casts:`; distinctness `attrs` and `table` columns name qualified ids or input fields |
| `mineworld attributes resolve <world> --json --packs …` | per person: look, values, origins, derived values; the profile's identity, sources, `realised`, `support`, `verify` | new |
| `mineworld attributes catalogue <world> --json --facet casting --packs …` | which ids the brief lists | new |
| `characters/style/phrases.yaml` (new) | the prompt words per attribute and value (§5.5.3) | moved verbatim from the interim profile's `labels:` and value strings |
| `characters/style/template.yaml` | the person blocks, placeholders over qualified ids | placeholders only; `version: 2` kept (D-g-5) |
| `characters/cast/casting_inputs.json` | per person: `key` (look), `cast`, `fields`, and optional `words` (per-person word overrides, §5.5.3) | `fixed`, `allowed`, `fixed_members`, `world` and `file` removed (they are world data now, or derived from resolve) |
| `characters/cast/PICKS.yaml` | the operator's picks | unchanged |

The binary is `--mineworld PATH`, default `target/debug/mineworld` (with `.exe` on Windows), resolved against the
repository root. A missing binary, a non-zero exit of `mineworld`, or JSON that does not parse fails by name, with
`mineworld`'s own message passed through unchanged (C-6).

#### 5.5.2 What it does, per cast

1. For each world of the cast, run `resolve` and `catalogue`; index people by `look`; ignore people whose look is null.
2. **The worlds agree.** For every look present in more than one world of the cast, every value and origin is equal;
   otherwise fail `cast <id>: <look> <attribute> differs between <world-a> and <world-b>`.
3. **Every look is accounted for.** Each look the worlds resolve is either a `casting_inputs.json` person of this cast
   or in `casts.<id>.fixed`; each input person resolves in at least one world; otherwise fail naming the look.
4. **Distinctness** over the cast and its fixed looks, by `casting.yaml`'s categories: a category differs when any listed
   value differs, or — with `min_diff` — when a `quantity` differs by at least that much or a `level` differs by at least
   that many ranks. A pair below `min_categories` fails, naming the pair, and the run stops without writing. There is no
   attempt loop: the salt is world data (step-26 §7.4b.1).
5. **Write** each input person's `brief.yaml`, `reference_prompt.txt` and `portrait_prompt.txt`, and the cast's section
   of `CASTING.md` (§5.5.5, §5.5.6).

`--check` re-derives every file and compares, as today, and holds the picks as today.

#### 5.5.3 Prompt words (`characters/style/phrases.yaml`)

```yaml
# characters/style/phrases.yaml (excerpt; the file lists every band, colour and level the interim labelled)
format: 1
words:                                   # {id} prints `value`, else the slug; {id|key} prints that key
  core.natal-sex:
    female: { noun: woman, poss: her }
    male:   { noun: man, poss: his }
  core.age-band:
    70s:     { posture: " with a slight forward stoop of age" }
    80-plus: { posture: " with a slight forward stoop of age" }
  core.hair-colour:
    dark-brown:  { value: dark brown }
    light-brown: { value: light brown }
  core.facial-hair:
    none: { line: { given: core.natal-sex, female: "", male: Clean-shaven. } }
    stubble: { line: "Facial hair: light stubble." }
  core.skin-tone:
    mst-4: { label: light-medium, number: "4" }
numbers:
  core.stature: { decimals: 2 }          # a quantity prints at this many decimals; default its scale
```

- `{id}` → the value's `value` words if given, else the slug; a quantity → its decimal at `numbers.<id>.decimals`; an
  `instant` is refused by name (no template uses one).
- `{id|key}` → `words.<id>.<slug>.<key>`; a missing key fails naming id, slug and key. Where the interim printed
  nothing (`posture` for the 20s, a woman's facial-hair line), `phrases.yaml` holds an explicit `""`. The rule kept from
  `fill()`: a line whose text is empty after substitution is dropped.
- **`given:`** — a key's entry may be a map `{ given: <id>, <slug>: words, … }`: the words for the person's value of
  `<id>`. One level only; a missing slug fails by name.
- **Per-person words** — `casting_inputs.json` `people[i].words: { "core.hair-colour": "reddish brown" }` replaces
  `{id}` for that person only, and the brief's `sources` line says so.
- Every entry is copied verbatim from the interim profile's `labels:` and value strings through Appendix A, so the
  rendered prompts are byte-identical (AC-g-4).

#### 5.5.4 The template's placeholders

`{sex|label.noun}` → `{core.natal-sex|noun}`; `{apparent_age}` → `{core.age-years}`; `{stature_m}` → `{core.stature}`;
`{build|label.article} {build|label.words}` → `{core.build|article} {core.build|words}`; `{age_band|label.posture}` →
`{core.age-band|posture}`; `{skin_mst|label}` → `{core.skin-tone|label}`; `{skin_mst}` → `{core.skin-tone|number}`;
`{face_shape}`, `{eyes}`, `{nose}`, `{lips}`, `{hair_colour}`, `{hair_texture}`, `{hair_style}` → `{core.face-shape}`,
`{core.eye-shape}`, `{core.nose}`, `{core.lips}`, `{core.hair-colour}`, `{core.hair-texture}`, `{core.hair-style}`;
`{facial_hair|label}` → `{core.facial-hair|line}`. Input-field placeholders (`{wear}`, `{manner}`, …) are unchanged. The
placeholder pattern becomes `\{([a-z][a-z0-9_.-]*)(?:\|([a-z_]+))?\}`. Nothing else in the template changes.

#### 5.5.5 The brief, format 2 (`mwcast/brief.py`)

```yaml
# Casting brief (step-26 §7.1), generated by tools/character_pipeline/casting/cast_director.py from
# `mineworld attributes resolve`. Do not edit by hand: rule a value with `cast_director.py rule`.
format: 2
person: alice-moreau
from: [worlds/social-cafe/people/alice.yaml, worlds/market-town/people/alice.yaml]
role: barista
attributes:
  core.age-band: 40s
  core.age-years: 46
  core.ancestry: white
  core.stature: 1.610
  …
outfit: { … }                            # unchanged lines from the inputs' fields
…
casting:
  cast: cafe-town
  profile: san-diego-county 3
  template_version: 2
  seed: mineworld-s26-cast-v1
  salt: 0
sources:
  core.age-band: derived from core.birth
  core.stature: "ruled 2026-10-15: S26 casting gate (characters/cast/PICKS.yaml, operator 2026-10-15)"
  core.natal-sex: "authored: worlds/social-cafe/people/alice.yaml"
```

- `attributes:` lists, in id order, every value whose declaration carries facet `casting`, plus every id the template
  or the table names (so `core.age-years` appears). A value prints as the slug, the decimal, or the level's slug.
- `sources:` keys are exactly the `attributes:` keys; each line is the origin, its date and `by` for a lock entry, the
  file for `authored`, the stratum for `sampled`, `derived from <from>` for a derived value, and `; words: <text>` when a
  per-person word replaced the prompt's.
- `brief.py`: `format` required and `2`; `attributes` joins `NESTED`; nested keys match `[a-z][a-z0-9_.-]*`;
  `CASTING_KEYS = (cast, profile, template_version, seed, salt)`; the rule "every `sources` key is a top-level key"
  becomes "every `sources` key is an `attributes:` key". A format-1 brief is refused by name (no compatibility path:
  every committed brief is regenerated in the same commit).

#### 5.5.6 The casting page (`CASTING.md`)

Per cast: the header names the cast's worlds, the profile id and version, the seed and the salt (C-4), and every `verify`
mark from `resolve` (`core.build`'s). The cast table uses `table`'s columns. The distinctness table is as today. The
realised distribution table is resolve's `realised` for every stratified attribute: `| Attribute | Group | Value |
Profile share | Expected | Realised |`, share = target ÷ n to three decimals, expected = target to two decimals, realised =
count — the same columns as the interim page, now computed by S30's sampler (the step's checkpoint). The appended
`CASTING_NOTES.md` text is unchanged.

#### 5.5.7 Ruling and re-drawing

- **`cast_director.py --config C rule --look <look> --at YYYY-MM-DD --by TEXT`** — for each world of the look's cast,
  for each non-derived value in the person's brief whose origin is `sampled` or `default`, and for each id in a derived
  value's `from`, runs `mineworld attributes lock <world> --person <key> --attribute <id> --origin ruled --at --by
  --packs …` (no `--value`: the value resolve gives now). It prints each entry written. It writes nothing else. This is
  the casting gate's write-back (step-26 §7.4b.1; R-S30g-d-1).
- **`--find-salt N`** — when distinctness fails, tries salts 1 … N with `resolve --salt n` and prints the first salt for
  which every pair passes, and the command that writes it into each world's `configure/attributes.yaml`. It never edits
  a world. This replaces the interim attempt loop (A-9's "attempt" is `salt`, S30-d A-d-7).

#### 5.5.8 The casting fixture world (the second world, and X-S26src-10's casting half)

`tools/character_pipeline/tests/fixtures/cast-freckles/` holds a World Pack (`systems: [presence, naming, appearance,
attributes]`, four people with looks, its own declaration `cast-freckles.freckles` with facets `appearance, casting`,
values `none, light, pronounced`, a profile `freckle-town` that samples sex, birth and `freckles`, no lock) and a
Presentation-side `casting.yaml`, `template.yaml` (a person block naming `{cast-freckles.freckles}`), `phrases.yaml` and
`casting_inputs.json`. The tests of C6 run the director against it with the real binary and write into a temporary copy.
No code path names `freckles` or `core`.

### 5.6 What is deleted

`presentation/mineworld-default/3D/characters/populations/` (both files), and in `cast_director.py`: `rng`,
`largest_remainder`, `match`, `weighted`, `cast_world`, `load_profile`, `given_of`, `lookup`, `weights_of`, `label`, the
`KINDS` constant, the attempt loop and `max_attempts`. `import random` and `import hashlib` leave the file. No adapter
remains (step-30 §8.6).

### 5.7 Pack roots for the repository's own commands (as Q-S30g-1 rules; the recommendation)

- `.cargo/config.toml` gains `[env] MINEWORLD_PACKS = { value = "entities", relative = true }`: every `cargo run` and
  `cargo test` process, and every process it starts (the server under test, Godot harnesses), finds the two packs. It
  does not override a value already set. The tests that assert refusal already remove the variable (A-g-3).
- The Python test packages that start the binary (`sdk/python/tests/conftest.py`, `cognition/lm-controller/tests/
  conftest.py`, `clients/agent/tests/conftest.py`, `tools/worldgen/tests/conftest.py`) set the same variable with
  `os.environ.setdefault`, to the absolute path of `entities/`.
- The user-facing commands that run a town (`README.md` lines 143 and 198, `docs/HUMAN_REVIEW_QUEUE.md`) add
  `--packs entities`, as Lakeside's already add `$R`.
- The release bundle ships `entities/person-attributes` and `entities/populations-us` and its launcher names them as a
  root (R-S23-g-1; C3 makes the change if `scripts/package.py` and the launcher are the only files, else it is S23's).

### 5.8 The tiler copies locks (`tools/worldgen/tile.py`)

- `locks/<name>.yaml` of the source: each `people.<key>` entry is copied once per block under `_copy(key, block)`, values
  and origins unchanged; `profile` and `seed` copied. A key the source does not define is refused (`TileRefused`, the
  existing "names … which the source does not define").
- A World Pack's own `attributes/` and `populations/` directories are copied unchanged (they hold declarations and
  profiles, never a person key).
- `configure/attributes.yaml` holds no reference (its `lock` is a file name), so the existing configure rule copies it.

### 5.9 The re-baseline (one per world, C7)

1. A long record of the **base** (`ci_parity.py record --profile long`, through `with-slot.sh`, or the nightly long
   record of the base commit) and one of the **head** of C6.
2. `ci_parity.py diff base head` per world, and a line diff of each world's summary lines (`line-300.*`, `line-1000.*`,
   `line-300s.*`) between the two records.
3. **Expected movement** (D-g-3): `bodies-yard` — nothing. Each adopting world — `facts-300s`, `journal-300s`,
   `sealed-300s` move; every base summary line is present unchanged; the only added summary lines name `traits-stated`.
   Anything else stops the commit (a material stop, §13).
4. `ci_parity.py baseline write head.long scripts/baselines.txt`, then one `reason S30-g:` line per adopting world:
   "the attributes pack's genesis `traits-stated` facts (12 per town, 10 at the lake) shift every later event id and add
   the `traits-stated` summary line; market-town's `hired` facts carry their job titles; no other line moved;
   re-recorded from <record>".
5. If `scripts/baselines.txt` did not match the base record (A-g-24), C1 first re-records the base with its own reason
   ("main's drift since milestone F; no S30-g change"), so C7's diff is S30-g's alone.
6. The `decisions` golden `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` is regenerated the same way:
   base (C1, if it fails on the base), then head (C7), with the reason in the test file's comment.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-g-1 (I-S30-7) | A ruled or authored value is never re-drawn: a new seed, salt or profile version leaves it equal | AC-g-3 |
| I-g-2 | The ruled cast is unchanged: every ruled value equals the interim brief's under Appendix A; the 42 prompt files are byte-identical | AC-g-1, AC-g-4 |
| I-g-3 (AC-S30-13) | The casting tool names no attribute id and no population: every id is in data | AC-g-6 |
| I-g-4 (I-S30-10, X-S30-9) | `bodies-yard` is byte-identical; a world without `attributes` is unchanged | AC-g-9; AC-g-8's AC-2 test |
| I-g-5 (step-30 §11.2) | Adoption changes no behaviour: every base summary line is unchanged | AC-g-9 |
| I-g-6 | One resolution: the casting page, the briefs and the world read the same `resolve` output; the café cast's two worlds agree | AC-g-2, AC-g-5 |
| I-g-7 (I-S30-3) | A `private` attribute (`natal-sex`, `ancestry`, `birth`, `age-years`) of a reference person is on no other observer's surface | AC-g-11 |
| I-g-8 | Nothing under `kernel/`, `contracts/`, `server/`, `systems/` changes | `git diff --stat` in C9 |
| I-g-9 (step-30 §8.6) | No interim sampler code and no interim profile remain | AC-g-7 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | `cargo fmt`; clippy `-D warnings` on `mineworld-cli` (and `mineworld-attributes-model` if C2 touches it); ruff, ruff format and pyright on `tools/character_pipeline` and `tools/worldgen` | per commit; CI `fast` |
| Unit (Python) | the director's word lookup (`given:`, per-person words, missing key), distinctness rule kinds, brief format 2 parsing and refusals, the tiler's lock copy | `tools/character_pipeline/tests/test_cast_director.py` (new, pure part); `tools/character_pipeline/tests/test_records.py` (brief cases); `tools/worldgen/tests/test_tile.py` |
| Integration (real binary, Rust) | resolve's `look` and derived values; the three worlds equal the oracle; the two towns agree; X-S26src-13; genesis counts; AC-2 without attributes; occupation | `tools/cli/tests/attributes_resolve.rs` (new cases); `tools/cli/tests/reference_attributes.rs` (new); `tools/cli/tests/social_composition.rs`; `worldpack/tests/social_cafe.rs` |
| Integration (real binary, Python) | the director end to end on the fixture world (freckles, `rule`, `--find-salt`); the director's `--check` on the committed cast (CI `python` layer); the tiled world loads and resolves | `test_cast_director.py` (marked `real_binary`); `CAST_DIRECTOR_CHECK`; `test_tile.py`'s real-binary case |
| Golden | `attributes-sampled.resolve.json` (hand-edited); the 42 prompt files (unchanged bytes); `CASTING.md` and 21 briefs (regenerated, reviewed); the `decisions` golden (re-recorded with reason) | as named |
| Regression | the long baselines of four worlds | C7's two records; one nightly dispatch on the final head (C9) |
| Gate 1 (real LLM) | NOT REQUIRED: no prompt template, no model-facing contract changes; S30-f owns the prompt parts, and every cassette replays from logs (A-g-26) | — |
| Gate 2 (real lifecycle) | NOT REQUIRED beyond the real-binary tests and the long records | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Second world or variant | Step ref |
| --- | --- | --- | --- |
| AC-g-1 | **Checkpoint.** `mineworld attributes resolve worlds/social-cafe --json --packs entities` gives, for every cast look, values equal to the interim brief values (the C0 fixture) mapped by Appendix A, including `core.age-years` = the brief's `apparent_age` and `core.age-band` = its `age_band`; each such value's origin is `ruled` or `authored` | the same assertion on `market-town` and `lakeside` | S30-g row |
| AC-g-2 | **The café cast agrees.** For every look of the café cast, `social-cafe` and `market-town` resolve equal values and origins; a planted difference in one town's lock (M-g-2) makes the director fail naming look, attribute and both worlds | two worlds by construction | I-g-6 |
| AC-g-3 | **Rulings survive (X-S26src-13, AC-S30-8).** `resolve --salt 1` and a copy with `seed: other` give, for every value whose origin is `ruled` or `authored` (and every value derived only from those), the same value as salt 0, in all three worlds | three worlds | AC-S30-13 |
| AC-g-4 | **Nothing ruled moves.** After C6, `git diff --stat <base> -- 'presentation/mineworld-default/3D/characters/cast/*/*_prompt.txt'` is empty; `cast_director.py --check` passes, including the picks; each regenerated brief's `attributes:` equal the C0 fixture under Appendix A | both casts (café: two worlds; Lakeside) | AC-S30-13 |
| AC-g-5 | **The page regenerates from S30.** Each cast's realised table in `CASTING.md` equals `resolve`'s `realised` for that cast's first world, row for row (a test parses both); every distinctness pair is PASS; the header lists `core.build`'s `verify` mark | both casts | S30-g row |
| AC-g-6 | **No hard-coded attribute list (AC-S30-13).** `cast_director.py` and `brief.py` contain no `core.` literal, no interim attribute name (`age_band`, `skin_mst`, `stature_m`, `hair_style`, …) and no population name; on the fixture world of §5.5.8, `freckles` appears in every brief with its origin, in the cast table when named in `table`, in the realised table, and in distinctness when a category names it — with no code change between the two runs | the fixture world | X-S26src-10 (casting half) |
| AC-g-7 | **The interim sampler is gone.** No `random`, `hashlib`, `cast_world`, `largest_remainder` in `cast_director.py`; no `characters/populations/` directory; nothing under `tools/` or `presentation/` names `populations/san-diego-county.yaml` of the Presentation Pack | — | step-30 §8.6 |
| AC-g-8 | **Genesis.** Each adopting world's genesis gains exactly one `traits-stated` fact per person (12, 12, 10); with `attributes` removed from a copy of `social-cafe` (and its sections, configure and lock), every other fact is the same row for row (AC-2) | three worlds; the AC-2 copy | step-30 §11.2 |
| AC-g-9 | **Only the expected keys move.** C7's base-to-head comparison shows `bodies-yard` unchanged; for each adopting world every base summary line unchanged and only `traits-stated` lines added; the nightly `baseline check` on the final head passes | four worlds | S30-g row |
| AC-g-10 | **A generated variant.** `tile.py` tiles `social-cafe` to 24 people; the tiled world validates, `resolve`s, and every block's copy of a cast person carries the source person's ruled values; the memory scale gate's 100-person tiling builds | the tiled variant | charter |
| AC-g-11 | **Consumers in a reference world.** In `market-town` at genesis, Bob's MCP `PersonLine.description` for Alice is non-empty and contains no `natal-sex`, `ancestry`, `birth`, `age-years` id or text; Alice's own `me.attributes` lists `works as barista`; Bob's view lists no occupation | `lakeside`: one observer's description of another is non-empty and private-free | AC-S30-10; R-S30g-f-1 |
| AC-g-12 | **No cassette moves.** `test_milestone_d_replay.py`, `test_record_replay.py` and `test_controller_choice.py` pass unchanged; if any committed cassette must move, it is re-recorded with a reason in its test file (R-S30g-f-1) | — | R-S30g-f-1 |
| AC-g-13 | **Clean check.** `mineworld attributes check` on each adopting world exits 0 and lists only `core.build`'s verify mark | three worlds | R-S30g-e-1 |
| AC-g-14 | **Roots (as Q-S30g-1 rules).** With the recommendation: `cargo run -p mineworld-cli -- run worlds/market-town --headless --seed 7 --days 1` succeeds with no flag; `target/debug/mineworld run worlds/market-town …` without a root fails naming `person-attributes` and the roots searched; with `--packs entities` it succeeds | two towns | `ARC-54` |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s30g`. Never `cd` elsewhere.

1. Re-read this commit's section, Appendix A if the commit uses it, and every file before editing it.
2. Make the numbered steps in order; nothing of a later commit.
3. Run the commit's validation one test target or one test file at a time; record command, count, wall time and
   PASS / FAIL / INCONCLUSIVE in §16. `cargo clippy -p <crate> --all-targets -- -D warnings`; `cargo fmt --all --check`;
   `uv run --locked ruff check <path>`; `uv run --locked ruff format --check <path>`; `uv run --locked pyright <path>`.
   Every cargo command and every long record runs through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
   **R-LOAD-1:** never `cargo test --workspace`, never a whole Python suite.
4. Mutations (only those named): copy the file, plant, run the named test red (record its name), restore from the copy,
   `cmp`, run green.
5. Answer the commit's review questions in §16 in one or two sentences each. The Opus reviewer reads the diff and the
   answers before the next commit starts.
6. `git status --short`: every changed path is in the commit's **Files**; otherwise revert it, or stop and record why.
7. Update §16 and the handoff in the same commit; stage by path; `git commit -F <file>` with the message below and the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`, no `curl`. A helper script, if needed, is a
   file in the session's scratch directory, never committed.

#### C0 — Base, ledger and the oracle

- **Files:** this file (§16); `.structured-coding/plans/mvp0/handoff-s30g.md` (new);
  `tools/character_pipeline/tests/fixtures/s30g/interim_cast.json` (new);
  `tools/character_pipeline/tests/fixtures/s30g/interim_to_core.json` (new).
- Steps:
  1. Record the base SHA; confirm #233, #280, #293, #300, #373 merged; record the state of #426, #460, #366, #498.
  2. Re-read A-g-1 … A-g-26 on the base; record every moved line.
  3. Record the latest `nightly.yml` verdict on `main` and which jobs failed (`gh run view`); record whether
     `cargo test -p mineworld-cli --test decisions` passes on the base (through `with-slot.sh`).
  4. Write `interim_cast.json`: for each of the 21 briefs and for Vera's `fixed_members` entry, the look and the interim
     value of each of the 15 interim attributes (`sex`, `age_band`, `apparent_age`, `ancestry`, `build`, `stature_m`,
     `skin_mst`, `hair_texture`, `hair_colour`, `hair_style`, `facial_hair`, `face_shape`, `eyes`, `nose`, `lips`),
     copied from the committed files by a scratch script that only reads them, plus the base SHA. This is the oracle's
     input; it is not produced by the code under test.
  5. Write `interim_to_core.json` by hand from Appendix A (per interim attribute: the `core` id and the value map).
- [ ] Implementation. [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`;
  `jq empty` on both JSON files; 22 looks in `interim_cast.json`. [ ] Review: every value in `interim_to_core.json` is
  Appendix A's; §15's rulings are applied to §§5, 8 before C1.
- **Commit message:** `docs(plan): S30-g — open the ledger at the base; the interim cast as an oracle`

#### C1 — Re-record the base where `main` has drifted (conditional)

- **Files:** `scripts/baselines.txt` (the drifted sections only); `tools/cli/tests/fixtures/decisions/
  social-cafe-seed-7-30-days.txt` (if C0 found it failing); this file (§16).
- Steps: only if C0 found `baseline check` or `decisions` failing on the base for a world this PR touches. Make one long
  record of the base (§5.9 step 1); `ci_parity.py baseline write`; add `reason S30-g base: main's drift since milestone F
  (<what the record shows>); no S30-g change; recorded from <record>` to each re-recorded section; regenerate the
  `decisions` golden by its test's documented command. Otherwise mark this commit `N/A` with C0's evidence.
- [ ] Implementation. [ ] Validation: `ci_parity.py baseline check base.long scripts/baselines.txt` exits 0;
  `cargo test -p mineworld-cli --test decisions` passes on the base. [ ] Review: no number edited by hand; the reason
  names the record.
- **Commit message:** `chore(baselines): re-record main's drift at the S30-g base (no S30-g change)`

#### C2 — `resolve` lists the look and the derived values

- **Files:** `tools/cli/src/attributes_resolve.rs`; `tools/cli/tests/attributes_resolve.rs`;
  `tools/cli/tests/fixtures/attributes-sampled.resolve.json`; `authoring/attributes/src/derive.rs` and `lib.rs` only if
  a public wrapper is needed (§5.4).
- Steps:
  1. A `look(pack, key) -> Option<String>` beside `authored()`, reading the `appearance` section's decoded content with
     `mineworld_appearance`'s section type.
  2. In `document()`, `"look": look(...)` instead of `null`.
  3. For each person, for each catalogue declaration with `Mutability::Derived`, the value at `WorldTime::EPOCH` through
     `derive::value` over the person's resolved plain values; when `Some`, insert `{ value, "origin": "derived", "from":
     [...] }`, `from` built by following `derive`'s source ids until a non-derived id, sorted and deduplicated. Apply
     the `--facet` filter as for every value.
  4. Edit the golden by hand (each derived entry checked against `derive.rs`); add test cases: Alice's `age-band` and
     `age-years` and their `from`; `look` null in the fixture (no `appearance`); a second fixture person's derived values.
- Validation: `cargo test -p mineworld-cli --test attributes_resolve`; `cargo test -p mineworld-cli --test
  attributes_builtin`; clippy. Mutation **M-g-1** (`from` omits the transitive step: `["core.age-years"]` for
  `age-band`) → the new `from` case red.
- [ ] Implementation. [ ] Validation. [ ] Review: no value other than derived ones added; `--synthetic` output
  unchanged; nothing in the sampler changed.
- **Commit message:** `feat(cli): attributes resolve names each person's look and derived values (R-S30g-d-1, S30-g)`

#### C3 — Pack roots for the repository's commands (as Q-S30g-1 rules)

- **Files** (recommendation): `.cargo/config.toml`; `sdk/python/tests/conftest.py`, `cognition/lm-controller/tests/
  conftest.py`, `clients/agent/tests/conftest.py`, `tools/worldgen/tests/conftest.py`; `README.md`;
  `docs/HUMAN_REVIEW_QUEUE.md`; `scripts/package.py` and the bundle launcher (only if C0 finds them to be the only
  files; else R-S23-g-1).
- Steps: §5.7, exactly. In `.cargo/config.toml`, an `[env]` table with a comment citing `ARC-54` and this design.
- Validation: `cargo test -p mineworld-cli --test requirements`; `--test entity_packs` (both still refuse without
  roots); `uv run --locked pytest tools/worldgen/tests/test_tile.py -m "not real_binary"`; `python3
  scripts/check_doc_headings.py`. AC-g-14 is measured in C5, once the towns require the packs.
- [ ] Implementation. [ ] Validation. [ ] Review: the variable is set only where `cargo` or the test packages start a
  binary; no shipped code reads a default root; `ARC-54` point 3 still holds for users.
- **Commit message:** `build: the repository's commands find its own data packs through MINEWORLD_PACKS (ARC-54, S30-g)`

#### C4 — The tiler copies a world's locks

- **Files:** `tools/worldgen/tile.py`; `tools/worldgen/tests/test_tile.py`; `tools/worldgen/README.md` (one sentence).
- Steps: §5.8. Tests: a fixture source with `locks/cast.yaml` naming two people, tiled to two blocks, gives four
  entries under the renamed keys with equal values; a lock naming an undefined key is refused by name; `attributes/`
  and `populations/` copied byte for byte.
- Validation: `uv run --locked pytest tools/worldgen/tests/test_tile.py`; ruff; pyright. Mutation **M-g-3** (locks not
  copied) → the new copy case red.
- [ ] Implementation. [ ] Validation. [ ] Review: no attribute id in the tiler; the rename is the people files' rule.
- **Commit message:** `feat(worldgen): the tiler copies a world's locks, renamed per block (S30-g)`

#### C5 — The three reference worlds adopt attributes

- **Files:** `worlds/{social-cafe,market-town,lakeside}/world.yaml`; `worlds/*/configure/attributes.yaml` (new, three);
  `worlds/*/locks/cast.yaml` (new, three); `worlds/social-cafe/people/*.yaml`, `worlds/market-town/people/*.yaml`,
  `worlds/lakeside/people/*.yaml` (sections and the three job titles); `worlds/*/README.md`;
  `tools/cli/tests/reference_attributes.rs` (new); the genesis-pinning tests C0 lists from A-g-23
  (`social_composition.rs`, `worldpack/tests/social_cafe.rs`, `commands.rs`, `content_kinds.rs`, `inspect.rs`,
  `server_command.rs`, `town_bodies.rs`, and any other C0 finds by `grep -l "genesis fact"`).
- Steps:
  1. §5.1's `world.yaml`, `configure/attributes.yaml` and people-file sections, from Appendix A.
  2. The lock: for each world, for each person and Appendix A attribute not authored in step 1, run `target/debug/
     mineworld attributes lock worlds/<w> --person <key> --attribute <id> --origin ruled --at 2026-10-15 --by "<by>"
     --value <core value> --packs entities`, with `birth` from §5.3. Use a scratch script that reads `interim_cast.json`
     and `interim_to_core.json`. Never edit a lock by hand.
  3. `market-town` titles (§5.1).
  4. `reference_attributes.rs`: AC-g-1 (three worlds against the two C0 fixtures, read with `serde_json`), AC-g-2's data
     half (the towns agree), AC-g-3, AC-g-8's counts, AC-g-11, AC-g-13, AC-g-14.
  5. The genesis-pinning tests: each changed literal carries a comment `S30-g: +N traits-stated`.
  6. `social_composition.rs`: an AC-2 test "without attributes, every other system is unchanged", modelled on
     `without_appearance_every_other_system_is_unchanged`; the two composition strings gain `attributes v1`.
- Validation (each alone, through `with-slot.sh`): `cargo test -p mineworld-cli --test reference_attributes`; `--test
  social_composition`; `-p mineworld-worldpack --test social_cafe`; the other listed test targets one at a time;
  `uv run --locked pytest tools/worldgen/tests/test_tile.py` (real-binary case: AC-g-10). Mutation **M-g-4** (one lock
  value changed in `lakeside`) → AC-g-1's Lakeside case red naming the person and attribute.
- [ ] Implementation. [ ] Validation. [ ] Review: `bodies-yard` untouched; `attributes` last in every `systems:`; no
  value outside Appendix A; every lock entry written by the command.
- **Commit message:** `feat(worlds): the café town, the market town and the lake carry their people's ruled attributes (S30-g)`

#### C6 — Casting reads the world

- **Files:** `tools/character_pipeline/casting/cast_director.py`; `tools/character_pipeline/mwcast/brief.py`;
  `presentation/mineworld-default/3D/characters/{casting.yaml, style/template.yaml, style/phrases.yaml (new)}`;
  `presentation/mineworld-default/3D/characters/cast/{casting_inputs.json, CASTING.md, */brief.yaml}`;
  `presentation/mineworld-default/3D/characters/populations/` (deleted); `tools/character_pipeline/tests/
  test_cast_director.py` (new); `tools/character_pipeline/tests/fixtures/cast-freckles/**` (new);
  `tools/character_pipeline/tests/test_records.py` (brief format 2 cases); `scripts/ci_layer.py` (only if the check
  command's arguments change).
- Steps:
  1. `phrases.yaml`: every `labels:` entry and every interim value string of the interim profile, keyed by Appendix A's
     ids and slugs (§5.5.3), including `core.facial-hair`'s `given:` entry.
  2. `template.yaml`: §5.5.4's placeholder table, nothing else.
  3. `casting.yaml` format 2 (§5.5.1): `casts.cafe-town: { worlds: [social-cafe, market-town], fixed: { vera-lindgren:
     { outer: open burgundy zip hoodie } } }`, `casts.lakeside: { worlds: [lakeside] }`; distinctness categories `age: [core.age-band]`, `stature:
     [core.stature] min_diff 0.05`, `build: [core.build]`, `skin: [core.skin-tone] min_diff 2`, `hair: [core.hair-style,
     core.hair-colour]`, `outer: [outer]`; `table` the same columns by id.
  4. `casting_inputs.json`: remove `fixed`, `allowed`, `fixed_members`, `world`, `file`; add `cast`; add Wes's `words`
     (`core.hair-colour: reddish brown`, `core.hair-style: short tousled crop`).
  5. `cast_director.py`: §5.5.1 … §5.5.7 and §5.6.
  6. `brief.py`: §5.5.5.
  7. Run the director; commit the regenerated briefs and `CASTING.md`; delete `populations/`.
  8. Tests: the pure part (word lookup, `given:`, a missing key refused, per-person words, `level` and `quantity`
     `min_diff`, brief format 2 and its refusals); the real-binary part on §5.5.8's fixture (AC-g-6, `rule` then
     `--salt 1` leaves the ruled value, `--find-salt` prints a salt and edits nothing, the worlds-agree failure on a
     two-world fixture cast); AC-g-4 and AC-g-5 against the committed cast.
- Validation: `python3 tools/character_pipeline/casting/cast_director.py --config presentation/mineworld-default/3D/
  characters/casting.yaml --check`; `git diff --stat <base> -- 'presentation/mineworld-default/3D/characters/cast/*/
  *_prompt.txt'` empty; `uv run --locked pytest tools/character_pipeline/tests/test_cast_director.py`; `… test_records.py`;
  `python3 tools/character_pipeline/cast.py check-characters`; ruff; pyright. Mutations **M-g-5** (the `given:` entry
  ignored) → `--check` red on the ten women's prompts; **M-g-2** (one town's lock differs) → the worlds-agree case red;
  **M-g-6** (a `core.` literal planted in `cast_director.py`) → AC-g-6's scan red.
- [ ] Implementation. [ ] Validation. [ ] Review: no attribute id in code; no prompt byte moved; every brief value
  traceable to the oracle; `rule` writes only through `mineworld attributes lock`; F-CAST-7 holds (no ancestry in any
  prompt).
- **Commit message:** `feat(cast): casting reads mineworld attributes resolve; the interim sampler is removed (AC-S30-13, S30-g)`

#### C7 — One re-baseline per world

- **Files:** `scripts/baselines.txt`; `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt`;
  `tools/cli/tests/decisions.rs` (its reason comment); any cassette AC-g-12 finds moved, with its reason; this file
  (§16, the summary diff).
- Steps: §5.9 steps 1–4 and 6, on the head of C6. Paste the summary-line diff per world into §16.
- Validation: `ci_parity.py baseline check head.long scripts/baselines.txt` exits 0; `cargo test -p mineworld-cli
  --test decisions`. Material stop: any summary line of the base changed, or `bodies-yard` moved.
- [ ] Implementation. [ ] Validation. [ ] Review: only the expected keys and lines moved (D-g-3); the reasons name the
  records; no number edited by hand.
- **Commit message:** `chore(baselines): re-record the three adopting worlds for their genesis traits (S30-g)`

#### C8 — Specification sync

- **Files:** `docs/MODULE_SPEC.md`; `docs/decisions/ARC-123.md`; `docs/decisions/ARC-54.md`;
  `tools/character_pipeline/PIPELINE.md`; `presentation/mineworld-default/3D/characters/cast/CASTING_NOTES.md` (one
  dated line: F-CAST-3's follow-up done; the casting tool reads the worlds).
- Steps: dated notes — `MODULE_SPEC` §8.1 (`resolve`'s `look` and derived values with `from`; casting selects facet
  `casting`); §4.1 "The lock file" (the casting tool writes ruled values through `lock`); `ARC-123` (the same, and
  `--find-salt` replacing the attempt); `ARC-54` (as Q-S30g-1 rules); `PIPELINE.md`'s casting section (brief format 2,
  `phrases.yaml`, `rule`). Old text kept visible. Then `python3 scripts/check_decision_ids.py --write-index`.
- Validation: `python3 scripts/ci_layer.py docs`. [ ] Implementation. [ ] Validation. [ ] Review: every finding of §3.2
  that changed specified text has its note; no specification text and code disagree (`CLAUDE.md` §2.1 rule 4).
- **Commit message:** `docs(spec): reference worlds carry attributes; casting reads the resolution (S30-g)`

#### C9 — Close-out

- **Files:** this file (§16); the handoff.
- Steps: `git diff --stat origin/main...HEAD -- kernel contracts server systems` empty (I-g-8); push `mvp0/pr-s30g`;
  `gh pr create`; CI on the exact head (three OSes, the `python` and `godot` layers) green; one `nightly.yml` dispatch on
  the final head for AC-g-9's `baseline check`.
- [ ] Implementation. [ ] Validation: CI and the nightly verdict. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-g close-out`
- **Then:** TRAIN-READY, reported to the coordinator — DO NOT MERGE.

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time, through
`with-slot.sh`; clippy on the touched crate; ruff and pyright on the touched Python paths. Never `cargo test
--workspace`, never a whole Python suite. The two long records of §5.9 are the only long local runs, each through
`with-slot.sh` (or taken from the nightly). The PR's CI on the exact head (Linux, Windows, macOS; `fast`, `python`,
`godot`) is the gate; one `nightly.yml` dispatch on the final head is the only extra long run.

---

## 11. Requirements this PR places on other PRs

- **R-S26src2-g-1 (S26-src2).** Build on `casting.yaml` format 2, `phrases.yaml` (with `given:` and per-person words)
  and brief format 2; X-S26src-10's prompt half (the appended line for an unreferenced attribute) and X-S26src-9, -11,
  -12, -14, -15 remain S26-src2's (Q-S30g-2).
- **R-S23-g-1 (S23, release).** The bundle ships `entities/person-attributes` and `entities/populations-us` and its
  server command names them as a root (if C3 could not); the release checklist lists `core.build`'s `verify` mark on
  San Diego County v3 as open until read or ruled (Q-S30g-4).
- **R-S30h-g-1 (S30-h).** `matraix-demo` names its packs the same way; no reference world changes backend.
- **R-S30j-g-1 (S30-j).** A newcomer in a reference world is sampled from San Diego County with the world's seed and
  salt; the lock is not consulted for a key it does not name.
- **R-S30d-g-1 (a model follow-up, unowned).** A person-file restriction on a derived band that a profile samples
  (`age-band: { allowed: [...] }`), so a biography's age range survives a re-cast of an unlocked person (F-S30g-9).

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §10, C-1 row | `--facet appearance` → the casting facet `casting` (F-S30g-4) |
| `step-30-person-attributes.md` §8.6 | `resolve` lists `look` and derived values with `from` (D-g-1) |
| `step-30-person-attributes.md` §11.2 | "A moved summary is a stop" → every base summary line unchanged; the new fact type's lines are expected (D-g-3) |
| `step-30-person-attributes.md` §13.1, S30-g row | the lock is `locks/cast.yaml`; the tiler and the pack roots are part of the row (F-S30g-1, F-S30g-8) |
| `step-26-cast.md` §7.4b.1 | C-1 by facet `casting`; C-2 carries derived values; the casting gate writes through `cast_director.py rule` |
| `step-26-cast.md` §7.4b.2 and the S26-src2 row of §12 | phrases `given:` and per-person words; the part S30-g shipped (Q-S30g-2) |
| `pr-s26-cast-director.md` | a dated note: the sampler is replaced by S30's (S30-g) |
| `overall.md` | S30-g frozen / merged; the S26/S30 note "switch done" |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-g — reference worlds adopt attributes; S26 casting reads resolve
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30g-reference-worlds.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§8, 10, 11.2, 12, 13); step-26-cast.md (§§7.4a, 7.4b);
                         pr-s26-cast-director.md; pr-s30c-attributes-pack.md; pr-s30d-sampler.md;
                         pr-s30e-builtin-catalogue.md; pr-s30f-attribute-consumers.md; docs/MODULE_SPEC.md §§4.1, 8.1;
                         docs/decisions/ARC-54, ARC-91, ARC-120 … ARC-126, DEP-67, DEP-68; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; .structured-coding/standards.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30g, its own, held by one session
BRANCH:                  mvp0/pr-s30g, created from origin/main
START CONDITION:         S30-d, S30-e, S26-cast merged (true at 0b2faf2f) AND this design frozen AND §15 ruled, then
                         the primary's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7, as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-3, I-S30-7, I-S30-10; §6 I-g-1 … I-g-9; §8 as written
SEQUENCE:                C0 … C9 (§9)
IMPLEMENTER:             Haiku-class sessions, one commit each, from §9; an Opus-class reviewer reads each commit's
                         diff and review answers before the next; C2 and C6 move to an Opus implementer after two
                         failed review rounds
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace), through with-slot.sh;
                         target/*/mineworld *; uv run --locked pytest <one file>; uv run --locked ruff; uv run --locked
                         pyright; python3 tools/character_pipeline/casting/cast_director.py; python3
                         tools/character_pipeline/cast.py check-characters; python3 scripts/ci_parity.py (record,
                         diff, baseline); python3 scripts/check_doc_headings.py, check_decision_ids.py, ci_layer.py
                         docs; jq; git; gh (pr create/view; workflow run nightly.yml once in C9; run view/download;
                         never merge); cp; cmp; mkdir -p; sed -n; grep; scratch scripts as files
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1); a
                         whole Python suite; any edit under kernel/, contracts/, server/, systems/; any edit to
                         worlds/bodies-yard; a hand edit of a lock file or of a baseline number; any image or model
                         call; a new external dependency
MATERIAL STOPS:          a ruled value that Appendix A (as ruled) cannot map; a prompt byte that must move; a
                         distinctness pair of a ruled cast failing; a base summary line moving in C7, or bodies-yard
                         moving; a needed change under systems/, kernel/, contracts/ or server/; the release bundle
                         needing more than C3's files; a cassette that cannot be re-recorded without a live model
PLATFORMS:               Linux, macOS, Windows (CI core, python and godot layers on all three)
VALIDATION BUDGET:       targeted tests unrestricted one file at a time; two long records (C7, and C1 if needed);
                         one nightly dispatch (C9). Gate 1 and Gate 2: NOT REQUIRED (§7)
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30g.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once frozen (source: the operator's freeze)
                         semantic commits: authorized      (working rules §14)
                         branch push: authorized           (working rules §21)
                         PR creation / update: authorized  (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: TRAIN-READY / MERGE-READY, reported to the coordinator (charter, 2026-10-18)
                         merge: never by the implementation session — the coordinator merges (charter, 2026-10-18)
POST-MERGE SYNC OWNER:   primary: step-30, step-26, pr-s26-cast-director, overall.md (§12); implementation session:
                         this ledger and the merge identity
STOP CONDITION:          S30-g READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-g-1 | The two towns' new requirement breaks a command or test nobody listed (A-g-3's 125 files) | Q-S30g-1's root through `ARC-54`'s variable reaches every cargo- and pytest-started process; CI's three OSes are the gate; a remaining failure is fixed by passing roots, never by bundling |
| R-g-2 | Observation frames grow in every reference world (schema 2 blocks for every person seen) | S30-f measured 2 048 B per observed person as its bound; C5 records the observation size of Bob's view in market-town; a request-rate or frame-size test that moves is a reviewed expectation change |
| R-g-3 | Client tests that print panels or glance lines for the towns (2D `--drive=panels`, the 3D slice probe) gain the attribute panel | expected (S30-f's consumers now see data); expectation edits carry `S30-g:` comments; a client code change is out of scope (material stop) |
| R-g-4 | A merge to `worlds/` (S31-h #426, S29-f #366) lands between C1 and C7 | C7 records base and head again after merging main; the reason line names both |
| R-g-5 | The ruled cast fails distinctness under the core vocabulary (two interim words mapping to one slug, as F-S30g-6's do) | AC-g-5 checks every pair in C6; a failing pair is a material stop, named, and goes back to the operator with Q-S30g-3's alternatives |
| R-g-6 | Haiku mis-maps a value | the oracle test (AC-g-1) compares every value with the C0 fixture through the hand-written map, in three worlds |
| R-g-7 | The director's YAML-subset reader cannot read `phrases.yaml`'s nested `given:` maps | the file is written in the subset the reader already parses (block maps, flow maps, quoted scalars); a parse failure is fixed in the file's form, not by adding PyYAML |
| R-g-8 | The nightly is red on `main` for unrelated reasons, hiding S30-g's movement | §5.9 measures base against head directly; C1 isolates main's drift |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30g-1 [how the towns find the two data packs; `ARC-54`]** | Adoption gives `social-cafe` and `market-town` their first `requires:`. By `ARC-54` a requirement resolves only in `--packs` roots or `MINEWORLD_PACKS`, and about 125 test and script files, the README, the review queue and the release bundle run the towns with no root (F-S30g-1). | **Set `MINEWORLD_PACKS=entities` for the repository's own developer commands** (`.cargo/config.toml` `[env]`, and the Python test packages' `conftest.py`), **add `--packs entities` to the documented user commands**, and **ship the two packs in the release bundle with its launcher naming them**. It uses `ARC-54`'s own mechanism, changes a handful of files, and users still name their roots. Declined: (b) `--packs` at every one of the ~125 call sites (correct, but a large mechanical diff with no added safety); (c) bundling the data packs into the build (reverses S30-e and `ARC-54`) |
| **Q-S30g-2 [the S26 switch vs S26-src2]** | Step-30 gives "S26 casting reads `resolve --json`" to S30-g; step-26 gives it, with `style/phrases.yaml`, to S26-src2. Removing the interim profile removes the prompt words, so the switch cannot ship without the phrases file (F-S30g-11). | **S30-g ships the switch**: resolve and lock, `phrases.yaml` (with `given:` and per-person words), brief format 2, `casting.yaml` format 2 without profile and seed, the interim sampler removed. **S26-src2 keeps the rest**: §7.4b.2's JSON casting config (`source`, `style`, `control`, `band`, `licence_policy`, `method_exception`), `--pack`, unsupported values and `substitutions`, the gate bindings, and the appended line for an unreferenced attribute |
| **Q-S30g-3 [four ruled values the core vocabulary cannot hold exactly]** | Wes's hair colour `reddish brown` and style `short tousled crop`, and Vera's style `messy high updo with face-framing strands`, have no `core` slug, and a world may not add values; Vera's age was ruled only as `20s`, and a band cannot be pinned (F-S30g-6). | **Lock the nearest slugs** — Wes `auburn` and `short-crop`, Vera `top-knot` — **and Vera's birth at age 25** (the band's middle), each lock entry's `by` quoting the original words. **Keep Wes's exact words for his prompt** as per-person words in `casting_inputs.json`, so his prompt stays byte-identical (Vera has no prompt). Declined: adding three values to `core` (a curated-catalogue change for two people) and a world-own attribute (splits one trait into two) |
| **Q-S30g-4 [the `verify` mark on `core.build`; Q-S30-5]** | San Diego County v3 keeps `verify` on the build figure (national NHANES; the county source failed twice). Q-S30-5 makes the release refuse a reference world whose profile carries a data `verify` mark (F-S30g-10). | **Adopt v3 unchanged and carry the mark to the release checklist (R-S23-g-1).** Every reference person's build is ruled, so the unread figure decides no value in any reference world. At release, either the county figure is read, or the operator accepts the national figure and the profile's next version drops the mark |
| **Q-S30g-5 [job titles in Market Town]** | R-S30g-f-1 adds `title:` to the reference jobs; the words are authored content. | **`alice` barista, `felix` shop assistant, `ivan` newspaper seller** — each from the employer and the person's tags (D-g-4). The operator may name others at freeze |

Bounded decisions recorded here rather than asked: D-g-1 (derived values in `resolve`, F-S30g-3), D-g-2 (facet
`casting`, F-S30g-4), D-g-3 (summary rule and base-to-head comparison, F-S30g-5), D-g-4 (the titles, if Q-S30g-5 is not
answered otherwise), D-g-5 (the template keeps `version: 2`: its rendered prompts are byte-identical, and the version
names the prompts, not the placeholder spelling; the picks stay valid).

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base, ledger, oracle | | [ ] | [ ] | [ ] |
| C1 base re-record (conditional) | | [ ] | [ ] | [ ] |
| C2 resolve: look and derived | | [ ] | [ ] | [ ] |
| C3 pack roots | | [ ] | [ ] | [ ] |
| C4 tiler copies locks | | [ ] | [ ] | [ ] |
| C5 reference worlds adopt | | [ ] | [ ] | [ ] |
| C6 casting reads the world | | [ ] | [ ] | [ ] |
| C7 one re-baseline per world | | [ ] | [ ] | [ ] |
| C8 specification sync | | [ ] | [ ] | [ ] |
| C9 close-out | | [ ] | [ ] | [ ] |

### Evidence

(empty)

### Mutations

(empty)

### Deviations and findings

(empty)

---

## Appendix A — Interim brief vocabulary → `core` (hand-written; the oracle's map)

Every interim value not listed maps to the identical slug.

| Interim attribute | `core` id | Value map | Written as |
| --- | --- | --- | --- |
| `sex` | `core.natal-sex` | identical (`female`, `male`) | `authored` in the people file (biography) |
| `apparent_age` | `core.birth` | §5.3's formula; checked through `core.age-years` | lock `ruled` |
| `age_band` | `core.age-band` (derived) | `80_plus` → `80-plus`; checked, never written | — |
| `ancestry` | `core.ancestry` | `hispanic_or_latino` → `hispanic-or-latino`; `nh_white` → `white`; `nh_asian` → `asian`; `nh_black` → `black`; `nh_multiracial_or_other` → `multiracial-or-other` | lock `ruled` |
| `build` | `core.build` | identical | lock `ruled` |
| `stature_m` | `core.stature` | the decimal at scale 3 (`1.61` → `1.610`; Vera `1.767`) | lock `ruled` |
| `skin_mst` | `core.skin-tone` | `n` → `mst-n` | lock `ruled` |
| `hair_texture` | `core.hair-texture` | identical | lock `ruled` |
| `hair_colour` | `core.hair-colour` | `dark brown` → `dark-brown`; `light brown` → `light-brown`; `reddish brown` → `auburn` (Q-S30g-3) | lock `ruled` |
| `hair_style` | `core.hair-style` | `chin-length bob` → `chin-length-bob`; `pixie cut` → `pixie-cut`; `low tied-back bun` → `low-bun`; `low ponytail tied at the nape` → `low-ponytail`; `braided crown updo` → `braided-updo`; `high top knot` → `top-knot`; `short layered cut` → `short-layered`; `short crop` → `short-crop`; `buzz cut` → `buzz-cut`; `short side part` → `side-part`; `short textured top` → `textured-top`; `shaved head` → `shaved-head`; `short swept-back hair` → `swept-back`; `close fade` → `close-fade`; `short tousled crop` → `short-crop` (Q-S30g-3); `messy high updo with face-framing strands` → `top-knot` (Q-S30g-3) | lock `ruled` |
| `facial_hair` | `core.facial-hair` | `not_applicable` → `none`; `short trimmed beard` → `short-beard` | lock `ruled` |
| `face_shape` | `core.face-shape` | `heart-shaped` → `heart`; `diamond-shaped` → `diamond` | lock `ruled` |
| `eyes` | `core.eye-shape` | `almond-shaped` → `almond` | lock `ruled` |
| `nose` | `core.nose` | `short upturned` → `upturned` | lock `ruled` |
| `lips` | `core.lips` | identical | lock `ruled` |

**Row "wes-calloway".** Every value his casting input `fixed` from the 2D identity source is `authored` in both towns'
`wanderer.yaml`: `natal-sex: male`, `stature: 1.86`, `build: average`, `ancestry: white`, `skin-tone: mst-2`,
`facial-hair: none`, `hair-style: short-crop`, `hair-colour: auburn`, `hair-texture: wavy`. His `age_band` was fixed and
his `apparent_age` drawn; his birth is locked `ruled` from the brief's `apparent_age`.

**Row "vera-lindgren"** (`visitor.yaml`; the accepted protagonist, VIS-3D-GODOT-1). Every value of her
`fixed_members` entry is locked `ruled` with `by: "VIS-3D-GODOT-1 (accepted protagonist)"`, mapped as above; `hair_style`
→ `top-knot` and the birth at age 25 (Q-S30g-3). `outer` is a casting-input field, not an attribute: it moves to
`casting.yaml`'s fixed look as the field the distinctness rule reads (`casts.cafe-town.fixed.vera-lindgren.outer`).

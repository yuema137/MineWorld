# Step 25 — S25: creator documentation, tutorials and an onboarding test

**Lifecycle:** `STEP DESIGN FROZEN 2026-10-14 (operator QS25-1/2/3/9/10; others as recommended)`. The step
plan (scope, invariants, PR split, adversarial criteria) is frozen on the rulings recorded in §13.1. Each PR is
still detailed to the commit in its implementing session's first commit and frozen on its own by the primary
session (`overall.md` §5 ruling 9). Nothing here authorizes a merge. **Implementers:** 25a goes to an
Opus-class implementer; 25b, 25c, 25d, 25e and 25f go to Haiku-class implementers. Each starts only on the
primary session's dispatch, in a fresh session and its own worktree. Progress, evidence and bounded corrections
stay writable.
*Superseded header:* `DRAFT (2026-10-14)`, PR #185 commit `bf6a8d4`.
**Author:** the S25 planning session, 2026-10-14. Worktree `/Users/yuema137/mineworld-worktrees/design-s25`,
branch `docs/s25-design`, from `origin/main @ 171ea06`.
**Effort:** `mvp0` · **Parent:** [`overall.md`](overall.md) · **Siblings it depends on:**
[`step-18-interaction-list.md`](step-18-interaction-list.md) (the Interaction List, S17),
[`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md) (decision list,
forgetting, happenings; PRs DL-a, DL-b, FG-a, FG-b, FG-c, WE-a filed under `mvp0/`),
[`step-17-cognition.md`](step-17-cognition.md) (P6, the language-model controller),
[`step-23-release.md`](step-23-release.md) (the public demo bundle).
**Binding parents:** `CLAUDE.md` §§1, 2 (documentation law), 3, 4 (items 7, 8, 9, 10, 13, 16); `overall.md` §5
("Framework, not demo", "Every platform", "Realistic defaults", "The World Interaction List", the step-24
rulings); `docs/REUSE_POLICY.md`; `docs/MODULE_SPEC.md` §§4, 4.1, 4.2, 5.1, 5.2, 8.1; `docs/DECISIONS.md`
ARC-31, ARC-33, ARC-53, ARC-54, ARC-61 … ARC-65, ARC-84 … ARC-88, DEP-10, DEP-11.
**Scope of this file.** It is the only file this session writes. Edits to `CLAUDE.md`, `overall.md`,
`docs/DECISIONS.md` and the specifications are proposed in §12 for the primary session (and, for `CLAUDE.md`,
the operator) to apply; the primary applies the `CLAUDE.md` §2.1 edit in its own PR. Decision numbers were
assigned by the primary session on 2026-10-14: `ARC-90`, `DEP-49` (`schemars`) and `DEP-50` (`pulldown-cmark` +
`snapbox`) (§11). Next free: `ARC-91`, `DEP-51`. The records are written into `docs/DECISIONS.md` by the
implementing PRs named in §11.

Every statement about an external tool carries the date it was read (2026-10-14) and its source. Crate licences
and maintenance were verified at freeze (§16). A statement still unconfirmed from a primary source is marked
**verify**, and the PR that first relies on it confirms it before freezing.

---

## 0. The answer in brief

1. **The reference is generated from the types that decode the files.** Every file a creator writes is
   already decoded by a typed Rust (or Pydantic) type: `deny_unknown_fields` structs, `AuthoredSection::Authored`,
   `PackConfiguration::Configuration`, the SDK's `parameters!` blocks and `InteractionSection` declarations.
   S25 derives a JSON Schema from each of those types with **`schemars`** (adopted, `DEP-49`), reads the
   Interaction List declarations from the constants packs already compile, and renders both into Markdown with
   a new read-only subcommand, **`mineworld reference`**. The Python half (`cognition.toml`, memory policies)
   comes from Pydantic's own `model_json_schema()`. CI regenerates the reference and fails on any difference,
   so the reference cannot drift (§5).
2. **The tutorial is a guide a person follows step by step**, under `docs/guide/`, English authoritative with a
   `*.zh-CN.md` mirror. Every step has the operator's form: **Do this** → **You should see this**. One tiny
   world is built from zero: two rooms, three people, one shop (§6).
3. **Comparison examples** take that finished world and change one thing at a time — relationship decay, one
   person on a language model, a market day, a forbidden action, weather — each showing "change this → the
   world changes like this" with real output (§7).
4. **The guide executes.** A test in the `mineworld` crate reads every guide page, writes every file block,
   runs every command block against the real binary and compares the printed output, on Linux, macOS and
   Windows. A page that no longer works is a red test (§8). Steps for features that have not merged are
   marked and listed as pending, never counted as passing.
5. **The onboarding test is the step's acceptance.** The operator starts it by hand. A fresh agent with no
   repository, no project instructions and no memory gets a few short sentences ("make a seaside village with
   a bakery and three residents, a weekend market, one person on the local model") and may read only the guide
   and the reference. A mechanical scorer checks the World Pack it produces: it validates, it runs N days, and
   it has each requested feature. Every place the agent gets stuck is a guide defect, fixed and then retested
   with a new fresh agent, until one agent succeeds in one pass. CI never calls a paid model (§9).
6. **Six PRs** (§10): 25a the Rust reference generator; 25b the cognition half of the reference; 25c the
   executable-guide runner; 25d the tutorial and the tiny example world; 25e the comparison examples and the
   feature chapters; 25f the onboarding harness and its rounds. 25a, 25b and 25c can start now and in
   parallel. 25d needs 25a and 25c. 25e needs 25d and each feature it covers. 25f's gating run needs
   everything, including P6.
7. **Placement:** inside MVP-0, before the release step's publishing PR (R-d). The guide is published from
   the repository at the release tag and linked from `PLAY.txt`; it does not go inside the bundle, because
   the bundle refuses every `.md` file (step-23 A-R3) (§10.4, QS25-8).

---

## 1. The requirement

### 1.1 The operator's words (2026-10-14), as relayed

> After MVP-0 we need tutorials and examples so users can create their own worlds and know what they can
> configure and how. When written, a completely fresh, independent agent must test a simple onboarding: can
> it complete one from a few short user instructions?

Confirmed by the operator the same day:

1. **A configuration reference generated from code** (typed schemas: `configure` sections, interaction-list
   rules and parameters, `decisions.yaml`, memory policies, happenings, pack manifests), so that it cannot
   drift.
2. **A "build your own world from zero" tutorial** with a tiny example world (for example two rooms, three
   people, one shop). Every step is written as **"do this → you should see this"** — the operator's required
   format.
3. **Comparison examples:** the same small world with different configuration (relationship decay on, one
   person on an LLM controller, a market day) showing **"change this → the world changes like this"**.
4. **An onboarding test.** A fresh agent with **no** repository context, given a few short instructions (for
   example "make a seaside village with a bakery and three residents, a weekend market, one person on the
   local model"), may read **only** the tutorial and the reference. Success is a World Pack that validates and
   runs N days with the requested features. **Every place it gets stuck is a tutorial defect.** Repeat with a
   new fresh agent after the fixes, until one succeeds in one pass. **This is S25's acceptance criterion.**

### 1.2 Standing rules this step must satisfy

- **Documentation law** (`CLAUDE.md` §2.1). A `README.md` is short and human; every other `.md` is a complete,
  precise specification. Terminology is fixed by `CORE_CONCEPTS.md` and `MODULE_SPEC.md`: the guide never
  introduces a synonym for a defined term (it may explain one in plain words, and then uses the term). English
  is authoritative; a `*.zh-CN.md` file is only a mirror and may never add, relax or amend a requirement.
- **Plain language for humans** (operator's rule): human-facing English is plain; human-facing Chinese follows
  the operator's plain-Chinese rule (the DongbeiGPT "plain-chinese" / "clear-tech-explainer" style: short
  sentences, everyday words, one idea per sentence, English terms kept in brackets on first use). Code,
  commands, file names and the generated reference stay English.
- **Framework, not demo** (`overall.md` §5, ARC-6): the guide teaches composition — install modules, compose a
  world, configure, run. It never teaches forking or editing a System Pack to get a world.
- **A world with every model removed is valid** (`CLAUDE.md` §1): every chapter except the language-model one
  runs with no model, and the language-model chapter falls back to a no-model controller.
- **Every platform** (`overall.md` §5): every guide command runs on macOS, Linux and Windows unchanged. No
  shell pipes, globs, `cp`, `rm` or environment syntax in an executed block.
- **Tests carry the weight** (`CLAUDE.md` §4 item 8): the guide's tests drive the real binary on real World
  Packs; the reference's test regenerates from the real types.
- **No paid model in CI, ever** (`cognition/lm-controller/README.md`; `CLAUDE.md` §4 item 10). The onboarding
  run is operator-triggered and costs the operator money; it is bounded in advance (§9.7).
- **Reuse before reinvention** (`REUSE_POLICY.md`): every dependency adopted or declined here has a decision
  record (§11).
- **Documentation-only changes do not run the build** (ARC-48 note, 13x): this design is docs-only.

---

## 2. Audit — what exists today (`origin/main @ 171ea06`)

### 2.1 Documentation a creator can find

| What | Where | Audience, as written | Finding |
| --- | --- | --- | --- |
| Project README | `README.md` (271 lines), `README.zh-CN.md` | humans | Quick start plays the bundled towns; it does not show how to make a world |
| Docs index | `docs/README.md` | humans | Lists specifications only; no guide, no reference |
| World Pack format | `docs/MODULE_SPEC.md` §4, §4.1 (lines 406–899) | agents (spec) | Complete, but a spec: it mixes the frozen model with the MVP-0 subset, and its field list is hand-written |
| Interaction List | `docs/MODULE_SPEC.md` §4.2 (lines 900–988) | agents (spec) | The shape is general; which actions, roles, facts and parameters each pack declares is only in each pack's source (`systems/conversation/src/interactions.rs` module doc, for example) |
| Decision list, memory policies | `docs/MODULE_SPEC.md` §4.1 (lines 538–585), §5.1, §5.2 | agents (spec) | Specified by DOC-24; not implemented until DL-a and FG-b merge |
| Happenings | `pr-we-a-happenings.md` §5.1 | agents (plan) | Frozen, not implemented |
| Each System Pack | `systems/*/README.md` | humans | Short; some name their section, none is a field reference |
| The command | `tools/cli/README.md`, `mineworld --help` | humans | Good orientation; points at `MODULE_SPEC.md` §8.1 for the full surface |
| `mineworld create` template | `tools/cli/templates/new-world/` | humans (comments) | Comments point at `docs/MODULE_SPEC.md` §4.1, a spec, as the place to learn more |
| Sample worlds | `worlds/social-cafe`, `market-town`, `lakeside`, `bodies-yard` | humans (README + YAML comments) | Rich, heavily commented, but large (12 people, 20 item kinds) and written as proofs (`AC-1`, `ARC-35`), not as lessons |
| Language-model configuration | `cognition/lm-controller/README.md`, `examples/hosted.toml.example` | humans | Good; typed by Pydantic (`config.py`), not generated |

**Finding F-S25-1 — there is no creator path.** A person who wants their own world has `mineworld create`, a
two-person template, and then specifications written for coding agents. No document says, in order, what to
do and what they should see. This is the gap the operator named.

**Finding F-S25-2 — the field reference is hand-written and partly absent.** `MODULE_SPEC.md` §4.1 lists the
framework's own fields by hand. The sections packs own (`name`, `routine`, `holdings`, `economy`, `job`,
`item`, `body`), the configurations (`calendar`, `weather`, `classes`, `packages`) and the Interaction List
declarations of each pack are documented only in source. A hand-written list drifts the first time a pack adds
a field (risk R-3 in `overall.md` §6 names exactly this for YAML schemas).

### 2.2 The typed sources a generator can read

Every file a creator writes is already decoded by a type, and refusing unknown keys is already the rule. That
is what makes generation possible without a parallel schema.

| File | Decoded by | Location | How |
| --- | --- | --- | --- |
| `world.yaml` | `WorldManifest`, `WorldIdentity`, `ConfigurationKey` | `worldpack/src/format.rs:42–145` | `#[derive(Deserialize)]`, `deny_unknown_fields` |
| person, place, item, organization files: framework fields (`tags`, `note`, `location`, `passages`) | `AuthoredPerson`, `AuthoredPlace`, `AuthoredItem`, `AuthoredOrganization`, `AuthoredPassage`, `AuthoredLocation` | `worldpack/src/format.rs:186–303`, `worldpack/src/content.rs` | custom read (top-level keys are framework fields **or** an enabled pack's section) |
| Sections a pack owns | `AuthoredSection::Authored` | `authoring/src/section.rs:181`; impls in `systems/{naming,schedule,item,inventory,economy,employment,bodies}/src/section.rs` | `DeserializeOwned`; "deserializing is validating" |
| `configure/<pack>.yaml` (world configuration) | `PackConfiguration::Configuration` | `authoring/src/configuration.rs:93`; impls `systems/calendar/src/system.rs:43`, `systems/weather/src/system.rs:47` | `DeserializeOwned` |
| `configure/<pack>.yaml` (Interaction List section) | the SDK's generic section, typed by `InteractionSection` (`ACTIONS`, `FACTS`, `PARAMETER_ROLES`, `Parameters`, `Knobs`) | `sdk/rust/src/interactions/`; e.g. `systems/conversation/src/interactions.rs:44–107` | a `DeserializeSeed` driven by the pack's constants; parameters via the `parameters!` macro (`sdk/rust/src/interactions/section.rs:94`), which carries each field's type, default and bounds as tokens |
| `configure/classes.yaml` | `EntityClasses` | `authoring/src/classes.rs` | custom `Deserialize` |
| `configure/packages.yaml` | the licence policy | `packages/src/policy.rs` | `Deserialize` |
| a data pack's `pack.yaml` | `PackFile`, `StyleManifest` | `packages/src/manifest.rs:30–60` | `#[derive(Deserialize)]`, `deny_unknown_fields` |
| `decisions.yaml` | DL-a's types (planned) | `pr-dl-a-decision-list.md`, `cognition/decision` (planned crate) | typed, unknown keys refused (MODULE_SPEC §4.1 rule 1) |
| memory policy `parameters` | FG-b's Python policies (planned) | `pr-fg-b-retention-host.md`; policy ids are Python entry points (`pr-dl-a-decision-list.md` line 302) | Pydantic (DEP-24) |
| `configure/happenings.yaml` | WE-a's types (planned) | `pr-we-a-happenings.md` §5.1 | `PackConfiguration` |
| `cognition.toml` | Pydantic models | `cognition/lm-controller/src/mineworld_cognition/config.py` | Pydantic v2 |
| the command line | clap derive | `tools/cli/src/main.rs:88–300` | clap 4.6 (DEP-11) |

**Every pack is reachable from one list.** `systems/installed/src/lib.rs` is the build's installed set
(`ARC-33`): the `installed!` macro (`sdk/rust/src/installed.rs`) expands to a `Capability` enum and one `match`
per question (`section`, `decode_section`, `configuration`, `decode_configuration`, `interaction_section`,
`package`, …). Adding "the schema of this pack's section" and "the schema of its configuration" is one more
question in the same macro, so a generator never names a pack — the same property the loader has.

**Finding F-S25-3 — custom deserializers need a hand-written schema.** Seven files hold a manual `Deserialize`,
`DeserializeSeed` or `deserialize_with` (`authoring/src/{configuration,classes,content}.rs`,
`contracts/src/ids.rs`, `worldpack/src/{content,load}.rs`, `worldpack/src/configure/tests.rs`). Each needs a
manual `JsonSchema` impl, or a `#[schemars(with = …)]` on the field that uses it. Bounded; listed per commit in
§10.

**Finding F-S25-4 — doc comments are written for developers.** The doc comments schemars would turn into
descriptions are often about the code ("Deserializing it *is* the owner's validation"). The reference needs
an author-facing sentence per field. 25a requires one (the generator refuses a field with no description) and
rewrites the ones that are about the code.

### 2.3 The command a creator uses (built binary, `mineworld --help`, read 2026-10-14)

`server`, `validate`, `replay`, `create`, `inspect`, `run`, `biography`, `perceived`, `interactions`,
`packs {list, show, validate, resolve}`, plus the hidden `install` and `add-system` (refused: not in MVP-0).
Read-only machine output already exists: `interactions --json`, `biography --json`, `perceived --json`. A probe
on 2026-10-14: `mineworld create seaside` then `mineworld validate seaside` succeeds and prints
`seaside (seaside)`, its systems, places, people and seats. A two-day `run` of `social-cafe` prints a header
line, one line per day and a summary of requests by type. These outputs are deterministic for a seed, which is
what lets a guide quote them.

### 2.4 What is merged and what is not (relevant to the guide)

| Feature the operator named | Owner | State on `origin/main @ 171ea06` |
| --- | --- | --- |
| Places, passages, people, items, organizations, shops, jobs, eating | S6–S9 packs | merged |
| Calendar and weather configuration | TW-a, TW-b | merged |
| Interaction List (classes, rules, parameters, consequences, regions) for the social packs | IL-b, IL-e | merged; IL-f/IL-g (other packs' sections) open |
| Relationship decay | FG-a | design frozen, not implemented |
| Decision list (`decisions.yaml`), the `motivated` model | DL-a, DL-b | designs frozen, not implemented |
| Memory policies and the retention host | FG-b, FG-c | designs frozen, not implemented |
| Happenings (market days) | WE-a | design frozen, not implemented |
| One person on a language model | P6 (S10) | not designed; its freeze waits on the operator's local-model spike (QB-15) |

**Finding F-S25-5 — the operator's sample onboarding instruction cannot pass today.** "A weekend market" needs
WE-a (and `calendar` for weekdays). "One person on the local model" needs DL-a and P6. S25's acceptance
therefore waits for those PRs. Everything else in S25 can be built now (§10.3).

### 2.5 The release bundle

Step-23 ships one bundle per OS with the server, the clients, three sample worlds and `PLAY.txt`. Its
adversarial check A-R3 **fails the build on any `*.md` file in a bundle** (step-23 §14, line 660). The guide
therefore cannot ship as Markdown inside the bundle without changing a frozen criterion (QS25-8).

---

## 3. Invariants of this step (proposed for freeze)

- **INV-S25-1 One authority per fact.** The generated reference is derived from the decoding types and is never
  edited by hand. The guide never states a field, a bound or a default as its own rule; it shows an example
  and links the reference. A specification stays the authority for design; the guide is never cited as one.
- **INV-S25-2 The reference cannot drift.** CI regenerates the reference from the build and fails on any byte
  difference. A field with no author-facing description fails generation.
- **INV-S25-3 The guide executes.** Every file block and every command block of every English guide page runs
  in CI on Linux, macOS and Windows against the real `mineworld` binary, in page order. A block that is not run
  is listed by name as pending, with the feature it waits for; a pending block is never reported as passed.
- **INV-S25-4 Mirrors cannot change meaning.** A `*.zh-CN.md` guide page has the same steps, in the same order,
  with byte-identical code blocks to its English page; CI checks this. Prose differs only by translation.
- **INV-S25-5 No new rule in a guide, a generator or a harness.** S25 adds no field, no behaviour and no
  default to any System Pack, the loader or the kernel. `mineworld reference` reads types; it changes nothing a
  world does. Every reference world's output is byte-identical before and after S25 (AC-8 baselines unchanged).
- **INV-S25-6 No paid model in CI.** No CI job starts an agent or calls a hosted model. The onboarding harness
  runs only on the operator's command and refuses to start without an explicit budget.
- **INV-S25-7 The onboarding agent is genuinely fresh.** It has no repository checkout, no project
  instructions, no user memory, no web access and no earlier transcript. A run that read anything outside its
  allowed set is void, not failed.

---

## 4. Options compared and the recommendation

### 4.1 Where the generated reference comes from

| Option | How | Drift protection | Cost | Verdict |
| --- | --- | --- | --- | --- |
| **R1 `schemars` + `mineworld reference`** | `#[derive(JsonSchema)]` beside each `Deserialize`; the `installed!` macro gains `section_schema` / `configuration_schema`; `parameters!` emits `JsonSchema` with `minimum`, `maximum`, `default`; Interaction List declarations rendered from the constants; a subcommand renders Markdown and writes the JSON Schemas | Full: the schema is derived from the type that decodes, including serde attributes (`rename_all`, `default`, `deny_unknown_fields`) | One dependency; a derive per type; manual impls for F-S25-3's seven custom deserializers | **Recommended** |
| R2 Own `Describe` trait | A MineWorld trait and a proc-macro deriving it | Full if derived — but that is writing schemars again | A proc-macro crate to build and keep | Declined: reinvents a mature crate (REUSE_POLICY §§1–3) |
| R3 rustdoc JSON | Scrape `cargo +nightly rustdoc --output-format json` | Partial: sees types, not serde's behaviour | Nightly toolchain; unstable format | Declined: the toolchain is pinned stable (`rust-toolchain.toml`) |
| R4 Hand-written reference + guard | Write the reference; a test checks every documented key is accepted | Weak: a key that exists but is undocumented is never caught | Ongoing hand edits | Declined: exactly risk R-3 |
| R5 Generate types from a schema (`typify`, schema-first) | Write JSON Schema, generate Rust | Full | Inverts ownership: the pack's type would no longer be the owner's validation (`ARC-31`, `ARC-61`) | Declined: contradicts the seam |

**`schemars`** (crates.io, read 2026-10-14 via `cargo search`: version `1.2.3`; "Generate JSON Schemas from Rust
code"). It reads serde attributes and doc comments, supports `#[schemars(range(min, max))]`, `with`, and manual
`impl JsonSchema`. Licence MIT (`cargo info schemars`, 2026-10-14; repository `GREsau/schemars`, licence MIT, not
archived, last push 2026-10-12, ~1.4k stars, MSRV 1.74). 25a C0 still runs `cargo deny check licenses` after the
add, as for every dependency (DEP-22).

**Why JSON Schema and not only Markdown.** The same generated schemas give editors completion and inline errors
for the YAML (any YAML language server reads JSON Schema). That is a second use for the same artefact, not a
reason on its own; the guide mentions it as optional (QS25-5).

**The contracts crate stays schemars-free.** Fields typed by a contract newtype (`EntityKey`, `SystemId`, …)
use `#[schemars(with = "String")]` plus a pattern, in the crate that owns the field. Adding a dependency to
`mineworld-contracts` is avoided because it is the most depended-on crate and has no use for it.

**The CLI reference** is generated by walking clap's own `Command` model (`get_subcommands`, `get_arguments`,
`get_about`), which clap already exposes (DEP-11). `clap-markdown` (crates.io `0.1.5`, read 2026-10-14) does the
same and is declined: a 0.1 crate for ~80 lines over an API we already depend on (REUSE_POLICY: adapt behind
our interface before adding).

**The Python half** uses Pydantic v2's `model_json_schema()` (DEP-24, already adopted). No new Python
dependency.

### 4.2 How the guide is kept working (docs that execute)

| Option | How | Fits "do this → you should see this"? | Verdict |
| --- | --- | --- | --- |
| E1 `trycmd` | Snapshot-tests `console` blocks in `.md` files (crates.io `trycmd 1.2.1`, read 2026-10-14) | Commands yes. File steps: only through a per-case `.in/` directory, i.e. a second copy of every YAML file next to the page, and each case starts from its own sandbox — a tutorial's steps are cumulative | Declined as the runner; its matching rules are adopted through `snapbox` |
| **E2 Own runner on `pulldown-cmark` + `snapbox`** | A test in `tools/cli/tests/` parses each page with `pulldown-cmark` (CommonMark parser, crates.io `0.13.4`), writes each `file=` block into one sandbox in page order, runs each `console` block's command with the real binary, and matches output with `snapbox` (`1.2.2`, the matching library under `trycmd`: `...` line elision, `[..]` in-line wildcards) | Yes: the page *is* the test, executed exactly in the order a reader follows it | **Recommended** |
| E3 mdBook `test` | Runs Rust code blocks | No: our blocks are YAML and shell | Declined |
| E4 Python script (`scripts/`) with `markdown-it-py` | Same idea in Python | Yes | Declined: the binary's tests run under `cargo test` on all three OSes already (`test`, `test-windows`, `test-macos`); a Python runner would need a new job and a new dependency |
| E5 Example worlds only, no page execution | Test the finished example world; leave the page untested | No: a page can say something the world no longer does | Declined |

Both crates are test-only (`[dev-dependencies]` of `mineworld-cli`). `DEP-50` records them (§11).

### 4.3 Where the guide lives, and which documentation-law class it is

The guide is human-facing, long, and plain. The law has two classes: `README.md` (short, human) and every other
`.md` (a precise spec for agents).

| Option | What | Verdict |
| --- | --- | --- |
| **G1 A third class: `docs/guide/`** | Amend `CLAUDE.md` §2.1: files under `docs/guide/` (and their `*.zh-CN.md` mirrors) are **guides** — for people and for agents new to the project; plain language; every instruction executed by CI; never an authority (a guide links the spec or the reference for every rule) | **Recommended** (QS25-1). It names what the files are, and keeps the README rule (short) and the spec rule (agents) both true |
| G2 Guides are specs written plainly | No amendment: each page is a "precise specification of a procedure" | Acceptable fallback. Risk: the next reviewer applies "complete, precise, for agents" and lengthens pages until a person cannot follow them |
| G3 One `README.md` per chapter directory | Each chapter is a short README | Declined: a chapter with eight steps and their output is not short, and the README rule would push detail out |

Layout (G1):

```text
docs/guide/
├── README.md                     human index: what you will build, the chapters, how long each takes
├── README.zh-CN.md
├── 01-get-mineworld.md           + .zh-CN.md mirrors for every page below
├── 02-your-first-world.md
├── 03-two-rooms.md
├── 04-three-people.md
├── 05-one-shop.md
├── 06-play-it.md
├── 07-rules-of-the-world.md      Interaction List: classes, a forbid, a parameter, a region
├── 08-time-and-weather.md
├── 09-who-decides.md             decisions.yaml: rule, motivated, lm          (DL-a, DL-b, P6)
├── 10-forgetting.md              relationship decay, memory policies          (FG-a, FG-b, FG-c)
├── 11-happenings.md              market days, festivals                       (WE-a)
├── 12-packs.md                   requiring an Entity Pack; what systems this build has
├── 13-when-something-is-refused.md  every refusal a reader can meet, and its fix (grown by §9.6)
├── compare/
│   ├── README.md                 human index of the comparisons
│   ├── relationship-decay.md     (FG-a)
│   ├── one-person-on-a-model.md  (DL-a, P6)
│   ├── market-day.md             (WE-a)
│   ├── a-forbidden-action.md     (merged features)
│   └── weather.md                (merged features)
├── examples/
│   └── corner-shop/              the finished world of chapters 2–5, byte-equal to what the pages write
└── features.toml                 which guide features are merged (§8.4)
docs/reference/                   GENERATED by `mineworld reference` — never edited by hand
docs/GUIDE_FORMAT.md              spec: the block grammar the runner executes, and the page rules
```

`docs/GUIDE_FORMAT.md` is a specification (agents), so it sits with the other specs, not in `docs/guide/`.

**The generated reference has no `zh-CN` mirror** (QS25-6). It is generated from English doc comments; a
mirror would be either hand-translated (and drift) or machine-translated (and be unreviewed). The zh-CN guide
links the English reference and explains each term it uses in plain Chinese at first use.

### 4.4 How a fresh agent is started with no context

| Option | Isolation | Cost | Verdict |
| --- | --- | --- | --- |
| **H1 A container holding only the allowed files** | A throwaway container from a small image: the release `mineworld` binary, `docs/guide/`, `docs/reference/`, an empty work folder, and the agent CLI. No repository, no `HOME` configuration, no memory. Network only to the model endpoint where the container runtime allows it (**verify**); otherwise web tools disabled and the transcript audited | Image build once; per-run tokens | **Recommended** |
| H2 A temporary folder on the host, agent CLI flags | `claude -p` in a fresh temp folder outside any repository, with user settings, project settings and memory disabled by flags, web tools disallowed (exact flags **verify** at 25f C0: `--setting-sources`, `--disallowedTools`, `--max-turns`, budget) | Cheaper to set up | Fallback when the operator has no container runtime. Risk: a user-level file the flags do not cover leaks context; the transcript audit (§9.5) must catch it |
| H3 A Paseo or other orchestrator agent | Starts an agent in a workspace | Isolation depends on the orchestrator's defaults | Declined as the harness: not reproducible from the repository alone |

The harness is agent-neutral: it hands the sandbox and the instruction to a configured command (`claude -p …`,
`codex exec …`, …) and reads back the produced World Pack and the transcript. The operator chooses the agent
and the model (QS25-3).

---

## 5. The generated reference (PRs 25a, 25b)

### 5.1 What it contains

| Page (`docs/reference/`) | Generated from | Lands with |
| --- | --- | --- |
| `world-pack.md` — `world.yaml`, the framework fields of person, place, item and organization files | `WorldManifest` and friends; the content-file framework fields | 25a |
| `sections.md` — one entry per pack section: key, which files carry it, the system that owns it, every field with type, required/optional, default, bounds, description, and a minimal valid example | each `Capability`'s `section_schema` | 25a |
| `configure.md` — one entry per world configuration (`calendar`, `weather`, `classes`, `packages`, later `happenings`, `economy`) | each `Capability`'s `configuration_schema`, plus the two framework keys | 25a (happenings, economy: WE-a) |
| `interaction-list.md` — the shared section shape, then per pack: actions with their roles and whether they are regional; reaction rules; facts with their default audience, narrowest audience and whether biography is configurable; parameters with type, default, bounds and description; knobs; reference lists | `InteractionSection` constants and the `parameters!` metadata | 25a |
| `decisions.md` — `decisions.yaml`, and each registered decision model's typed `parameters` | DL-a's types and the model registry | DL-a, then 25a's follow-up commit (§10.3) |
| `memory-policies.md` — each policy id and its `parameters` | FG-b's Pydantic models | 25b, after FG-b |
| `cognition-toml.md` — the operator's model configuration file | `config.py` Pydantic models | 25b |
| `pack-manifest.md` — a data pack's `pack.yaml` and a style manifest | `PackFile`, `StyleManifest` | 25a |
| `cli.md` — every visible subcommand and argument, with its help text | clap's `Command` model | 25a |
| `schema/*.schema.json` — the JSON Schemas themselves | the same | 25a, 25b |
| `README.md` — the index, generated too | the generator | 25a |

Every page starts with a generated banner:

```text
<!-- GENERATED by `mineworld reference --out docs/reference` from mineworld <version> (<git describe>).
     Do not edit. Change the type's doc comment and regenerate. -->
```

The page body names the build it describes ("this reference describes the packs installed in this build"),
because a build with a third-party pack has a different reference (`ARC-33`, `ARC-66`). The checked-in reference
describes the repository's own installed set.

### 5.2 How a pack's schema is reached

- `AuthoredSection::Authored` and `PackConfiguration::Configuration` gain the bound `schemars::JsonSchema`. The
  compiler then refuses a pack whose section or configuration has no schema. This is a change to the SDK's
  public contract (QS25-2); before a stable contract exists it is made cleanly (`CLAUDE.md` §4 item 12), and the
  third-party `acme-fishing` pack is updated in its own repository and re-pinned in the same PR (step-16
  E-c's two lines).
- `installed!` gains three questions: `section_schema(&self) -> Option<Schema>`,
  `configuration_schema(&self) -> Option<Schema>`, `interaction_declarations(&self) -> Option<Declarations>`.
  The macro still names no pack.
- `parameters!` gains, for each field, its JSON type, `default`, `minimum`, `maximum` and the field's doc
  comment as `description`, emitted as a `JsonSchema` impl on both the full struct and its partial twin. The
  tokens are already in the macro (`$default`, `$low`, `$high`, `$field_meta`), so no field is restated.
- `Declarations` is a plain data struct built from `InteractionSection`'s constants (`ACTIONS`, `FACTS`,
  `PARAMETER_ROLES`) and the knob and parameter schemas. It lives in `mineworld-sdk`.

### 5.3 The command

```text
mineworld reference --out DIR      write the reference into DIR (created or overwritten, files only it owns)
mineworld reference --check DIR    regenerate in memory; exit 1 naming every file that differs or is missing
                                   or extra; exit 0 when DIR is exactly what this build generates
```

Read-only with respect to worlds: it loads no World Pack and starts no world. Output is deterministic: packs in
the installed set's order, fields in declaration order, no timestamps, `\n` line endings on every OS.

### 5.4 CI

- The `test` layer (Linux) runs `cargo test -p mineworld-cli --test reference`, which calls the command in
  `--check` mode against `docs/reference/`. A planted change to one doc comment turns it red (§10, 25a C5).
- The Python half: a `pytest-cognition` test regenerates `memory-policies.md`, `cognition-toml.md` and their
  schemas and compares them with the checked-in files.
- A documentation-only PR does not run the build (ARC-48 note); a PR that edits `docs/reference/` by hand still
  runs the check, because `ci_changes.py` classifies `docs/reference/**` as generated output, not docs (25a C5
  adds the rule).

---

## 6. The tutorial — build your own world from zero (PR 25d)

### 6.1 The tiny example world: `corner-shop`

Two rooms, three people, one shop — the operator's example, kept as small as the features allow.

```text
places         home (two flats behind one front door), shop
               one passage joining them
people         ana (runs the shop), ben, cai
seats          ana, ben, cai
systems        presence, movement, conversation, relationships, naming, schedule,
               item, inventory, economy, employment, consumption
items          bread, tea
organizations  corner-shop-co (owns the shop; its key is not the shop's, ARC-36)
```

Real-world defaults where a number is asked for: a person 250 mm in radius (`overall.md` §5, "Realistic
defaults"), a doorway 900 mm wide, prices in the currency's minor units as Market Town authors them. Each value
the guide chooses is explained in one plain sentence with its source.

The finished pack is committed at `docs/guide/examples/corner-shop/`. The runner proves that executing chapters
2–5 writes exactly that directory, byte for byte (§8.3), so the example and the pages cannot disagree. It is not
placed under `worlds/`, so it adds no AC-8 baseline and no nightly cost (QS25-7).

### 6.2 The step format (the operator's)

Every step of every chapter has this shape. `docs/GUIDE_FORMAT.md` specifies it; the runner enforces the parts a
program can check.

````markdown
### Step 3.2 — Join the two rooms with a door

**Do this.** Replace `corner-shop/places/home.yaml` with:

```yaml file=corner-shop/places/home.yaml
tags: [home]
note: Two flats behind one front door.
passages:
  - { to: shop, … }
```

**Then run:**

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation
  places     home, shop
...
```

**You should see** both places listed, and no line starting with `refused`.

**If you see** `refused: … passage …` instead, the door names a place that is not in `places:` in
`world.yaml`. See [When something is refused](13-when-something-is-refused.md#passage).
````

Rules (specified in `GUIDE_FORMAT.md`, checked where marked ✓):

1. One action per step. A step that needs two files is two steps. ✓ (at most one `file=` block per step)
2. Every "Do this" that changes a file shows the **whole** file, never "add these lines". The example world is
   small enough for this, and a whole file cannot be pasted in the wrong place. ✓ (a `file=` block replaces the
   file)
3. Every step ends with a command and its expected output ("You should see"). ✓ (every step has a `console`
   block)
4. Output is quoted exactly, with `...` for lines the reader need not compare and `[..]` inside a line for a
   value that legitimately changes (a version). ✓ (snapbox matching)
5. Executed commands use only `mineworld …`, forward-slash relative paths, and no shell syntax, so one block runs
   on all three OSes. ✓ (the runner refuses anything else)
6. When an OS differs (installing, copying a folder), the page shows the three variants as **non-executed**
   text, and an executed runner directive does the same thing (§8.2). ✓
7. Plain language: sentences of at most about 25 words; a defined term (`World Pack`, `System Pack`, `Place`,
   `seat`, …) is explained in plain words at first use on each page, linked to `CORE_CONCEPTS.md`, and then used
   exactly; no decision ids, plan names or internal history in the body (a "Why it works this way" footer per
   chapter may link specs).
8. A rule is never stated as the guide's own: "a doorway must name a declared place" links the reference entry
   that says so.

### 6.3 Chapters

| Chapter | What the reader does | What they see at the end | Features (all merged unless marked) |
| --- | --- | --- | --- |
| 01 Get MineWorld | Get the binary (release bundle, or `cargo build --release` from a checkout); `mineworld --version`; run `market-town` for 2 days | the run header, two day lines, the summary | — |
| 02 Your first world | `mineworld create corner-shop`; `validate`; `run --days 1`; read what each file is | a one-place, two-person world running | presence, movement, conversation |
| 03 Two rooms | rename the place to `home`, add `shop`, join them with a passage, place people in rooms | `validate` lists both; `run` shows `walk-to` requests accepted | movement |
| 04 Three people | three people with names and day routines; seats | `validate` lists three seats; `biography --person ana` after a 1-day saved run shows her day | naming, schedule, relationships |
| 05 One shop | two item kinds; an organization; its shop, prices and stock; wallets; ana's job; eating | `run --days 3` summary shows `buy` and `eat` requests accepted; ana is paid | item, inventory, economy, employment, consumption |
| 06 Play it | `mineworld server corner-shop`; open the 2D client (and the 3D client) as `ben` | non-executed in CI beyond the server's start line; a screenshot per client | — |
| 07 Rules of the world | `configure/classes.yaml` (a `shopkeeper` class), a `conversation` section: forbid talking in the shop's back room (a region), a longer conversation gap | `interactions corner-shop --place shop` prints the resolved rules; a run's `talk` count changes | Interaction List (IL-b, IL-e) |
| 08 Time and weather | enable `calendar` and `weather`, configure a start date and a climate | `run` header shows the date; day lines show weather | calendar, weather |
| 09 Who decides | `decisions.yaml`: everyone `rule`, then ben `motivated`, then cai `lm` with fallback `motivated`; `cognition.toml` pointing at a local model | `run` header prints the decision-list digest; with no model running, cai falls back and the run says so | **DL-a, DL-b, P6** |
| 10 Forgetting | relationship decay parameters; a memory policy for cai | a 30-day run: acquaintances who never meet again fade (comparison in §7) | **FG-a, FG-b, FG-c** |
| 11 Happenings | `configure/happenings.yaml`: a market every Saturday and Sunday in the shop; an `economy` price change on market days | a 7-day run lists two market happenings on the weekend days; `buy` counts rise on those days | **WE-a** (+ calendar) |
| 12 Packs | `packs list`; require the bundled Entity Pack `modern-goods` instead of authoring items; what a third-party System Pack is and how a build gets one | `packs resolve corner-shop` names every pack and where it came from | S16 |
| 13 When something is refused | (reference page) each refusal message a reader of chapters 1–12 can meet, why, and the fix; one anchor per message | — | grown by §9.6 |

Chapters 09–11 are written in 25e as their features merge (§10.3). Until then each appears in the index with
"coming when <feature> lands" and is pending in the runner, never hidden.

### 6.4 The zh-CN mirror

Each page has `<name>.zh-CN.md`, written in plain Chinese (§1.2) by the implementer and reviewed by the primary
session. The runner checks parity (INV-S25-4): same number of steps, same step numbers, byte-identical code
blocks, every link target existing. It does not execute the mirror separately — the blocks are the English ones.

---

## 7. Comparison examples (PR 25e)

Each comparison is one page under `docs/guide/compare/`, with the same skeleton:

1. **Start from** `corner-shop` (copied by a runner directive to a new folder, §8.2).
2. **Run it** with a fixed seed and a fixed number of days, and look at one specific output.
3. **Change this:** one file, shown whole.
4. **Run it again**, same seed, same days, same output.
5. **The world changes like this:** the two outputs side by side, and one plain paragraph saying why.

| Page | Change | What is compared (read-only commands only) | Waits for |
| --- | --- | --- | --- |
| `a-forbidden-action.md` | a `conversation` rule forbidding `talk` in the shop | `run` summary: `talk` accepted count drops; `interactions --place shop` shows the rule | merged |
| `weather.md` | enable `weather` with a rainy climate | day lines show rain; time spent outdoors in `biography` changes | merged |
| `relationship-decay.md` | relationships' decay parameters on (FG-a) | `biography --json` after 30 days: an acquaintance level that falls instead of staying | FG-a |
| `one-person-on-a-model.md` | `decisions.yaml` assigns cai to `lm` | the run header's decision digest; cai's lines in `perceived`/`biography`; in CI, a recorded model session (replay) | DL-a, P6 (replay path) |
| `market-day.md` | `configure/happenings.yaml` market on weekends, plus an `economy` price modifier | per-day `buy` counts across 7 days; the happening facts on Saturday and Sunday | WE-a |

**Finding F-S25-6 (to audit in 25e C0):** each "what is compared" must be visible through an existing
read-only command. If a difference can only be seen by reading the save's database, that is recorded as a
finding and goes to the primary session; the remedy (a small read-only report option, its own PR) is not
designed here. Adding a command to make a guide look good is not in S25's scope (INV-S25-5).

**The language-model comparison in CI** runs in replay mode against a committed recording, using whatever replay
path P6 provides (P5a's cassettes are the precedent: `replay` mode builds no network client). If P6 provides no
replay path for a whole run, the page's model steps are marked `operator-only` (§8.4) and the comparison is
shown with recorded output plus the date and model it was recorded with. This is a dependency on P6's design,
recorded for its planner (§12.4).

---

## 8. Docs that execute (PR 25c)

### 8.1 The runner

`tools/cli/tests/guide.rs` (an integration test of `mineworld-cli`, so it gets `CARGO_BIN_EXE_mineworld` and runs
under `cargo test` on every OS the suite already runs on). For each page listed in the guide's index, in order:

1. Create one scratch directory per page (`mineworld-test-support`'s scratch directory, DEP-29), or reuse the
   previous page's when the page declares `continues: <page>` (chapters 02–05 continue one another).
2. Walk the page with `pulldown-cmark`. For each fenced block:
   - ` ```yaml file=PATH ` (any language with `file=`): write the block's text to `PATH` in the scratch directory,
     creating parent folders, replacing the file.
   - ` ```console `: for each `$ ` line, split the command into words (no shell), require the first word to be
     `mineworld`, run the binary with the scratch directory as working directory, and match its combined
     stdout and stderr against the lines up to the next `$ ` with `snapbox` (`...`, `[..]`). An exit code other
     than 0 fails unless the block's info string says `exit=N`.
   - any other block: not executed (shown to the reader only).
3. Before each block, read HTML-comment directives (§8.2).
4. Report per page: blocks run, blocks pending (with the feature), blocks failed with the expected and actual
   output.

### 8.2 Directives

HTML comments, invisible on GitHub and in any Markdown renderer, read by the runner:

```text
<!-- guide: continues 02-your-first-world -->   reuse that page's scratch directory
<!-- guide: copy corner-shop corner-shop-rules --> copy a folder (the page shows the per-OS commands as text)
<!-- guide: requires happenings -->              the next block waits for the feature `happenings`
<!-- guide: operator-only -->                    the next block needs a live model or a display; never run in CI
<!-- guide: equals examples/corner-shop -->      the scratch folder named must equal the committed example
<!-- guide: packs <dir> -->                      add a --packs root for the next commands (a bundled data pack)
```

A directive the runner does not know fails the page.

### 8.3 What the runner guards besides commands

- **Example equality.** `equals` compares the scratch folder with `docs/guide/examples/corner-shop/` file by file,
  byte for byte. A change to a page that is not mirrored in the example (or the reverse) is red.
- **Mirror parity** (INV-S25-4). For each English page with a `.zh-CN.md` mirror: the same step headings'
  numbers in the same order, the same fenced blocks byte for byte, the same directives.
- **Reference links.** Every link from a guide page into `docs/reference/` names an anchor that exists in the
  generated page.
- **No stray commands.** A `console` command other than `mineworld …` fails, so no page ever depends on a shell.

### 8.4 Pending features — never a silent skip

`docs/guide/features.toml` lists guide features and whether they are merged:

```toml
[features]
interaction-list = true
calendar = true
weather = true
decisions = false        # DL-a
motivated = false        # DL-b
lm-controller = false    # P6
relationship-decay = false # FG-a
memory-policies = false  # FG-b, FG-c
happenings = false       # WE-a
```

- A block under `requires X` with `X = false` is **pending**: listed by page, step and feature in the test's
  output, and counted separately. It is not run and not reported as passed.
- A block under `requires X` with `X = true` must run and pass.
- The test fails if `X = false` but the feature is in fact present (the runner asks the binary: `mineworld
  reference` lists the installed packs and the decision models; a happenings pack in the build with
  `happenings = false` is red). So the flag cannot be forgotten when the feature lands.
- `operator-only` blocks are listed as such in every run and are run by the operator at milestone checks.

### 8.5 Cost

Pages run worlds for at most 7 simulated days (30 for the decay comparison), in memory except where a page saves.
Target: the whole guide under 60 s on the Linux `test` runner, measured in 25c and again in 25d; above 120 s the
long comparisons move to the nightly workflow (`.github/workflows/nightly.yml`) and the PR leg keeps chapters
01–08.

---

## 9. The onboarding test (PR 25f) — S25's acceptance

### 9.1 What is tested

Whether a newcomer who has only the guide and the reference can make the world they want. The newcomer is a
fresh language-model agent, because it can be made to have no other source, and because it reads literally:
where the guide is ambiguous, it guesses or stalls, and both are visible in its transcript.

### 9.2 The sandbox (what the agent may read and run)

```text
/sandbox/
├── guide/        a copy of docs/guide/ (English pages only; mirrors excluded) at the commit under test
├── reference/    a copy of docs/reference/
├── bin/mineworld the release binary of the same commit (on PATH)
├── packs/        the bundled data packs the release ships (modern-goods, the presentation packs)
└── work/         empty: the agent writes its world here
```

Nothing else: no repository, no `worlds/` (the sample worlds are not in the allowed set — a creator who copies
Market Town is not testing the guide; QS25-4 asks whether to allow them), no specifications, no source.
Assembled by `tools/onboarding/assemble.py` from a release build.

### 9.3 The instruction sets

Committed under `tools/onboarding/instructions/`, each a few plain sentences, each paired with a machine
rubric (§9.4).

| Id | Instruction (verbatim to the agent) | Gating? |
| --- | --- | --- |
| **ON-1** | "Make a seaside village with a bakery and three residents. Hold a market every weekend. Put one person on the local model. Check that it runs for 14 days." | **yes — the operator's example** |
| ON-2 | "Make a mountain inn with a kitchen and a common room, four guests and an innkeeper. Guests who stop meeting should slowly forget each other. Nobody may shout in the common room after 22:00." | reported, not gating (QS25-4) |
| ON-3 | "Make a two-street town with a tea shop and a bookshop, six people, two of whom work. Use the goods that come with MineWorld instead of inventing items." | reported, not gating |

Every instruction set also carries a fixed preamble, the same for every run:

```text
You are making a world for MineWorld. You may read only the files in /sandbox/guide and /sandbox/reference,
and run the `mineworld` command. Write your world in /sandbox/work. When you are done, write DONE in
/sandbox/work/STATUS. If you cannot continue, write STUCK and one sentence saying why. Whenever the guide did
not tell you something you needed, add one line to /sandbox/work/GAPS.txt saying what you looked for.
```

`GAPS.txt` is the agent's own report; the transcript audit (§9.5) is the independent one.

### 9.4 The scorer (mechanical, deterministic, runs in CI on fixtures)

`tools/onboarding/score.py WORLD_DIR --rubric ON-1` checks, using only the `mineworld` command's read-only
output (`validate`, `packs resolve`, `interactions --json`, `run`, `biography --json`) and the decision list
resolution DL-a provides:

| Check | ON-1 meaning | Evidence |
| --- | --- | --- |
| S1 validates | `mineworld validate` exits 0 | exit code and output |
| S2 runs N days | `mineworld run --headless --seed 1 --days 14` exits 0, prints 14 day lines, 0 faults | output |
| S3 places | at least one place whose key, name or tags contain `bakery`; a place whose tags or name say seaside, shore, harbour, beach, pier or sea (word list in the rubric file) | `validate`, the place files |
| S4 residents | exactly three people in `population` who are seats | `validate` |
| S5 weekend market | `happenings` enabled and configured; the 14-day run records a market happening on each Saturday and Sunday and on no weekday | the run's facts (WE-a's fact type) |
| S6 one person on the local model | the resolved decision list assigns exactly one seat to `lm`, with a fallback; no provider, URL or key appears in the World Pack (MODULE_SPEC §4.1) | DL-a's resolved list; a scan of the pack |
| S7 live (operator's machine only) | with the local model running, the 14-day run completes and that seat's decisions came from the model (P6's evidence) | P6's run report; skipped, and reported as skipped, without a local model |

A run **passes** when S1–S6 pass, and S7 passes on the operator's machine, and the agent wrote `DONE`, and the
transcript audit finds no stuck point (§9.5). The scorer's own tests run in CI against committed fixtures: one
world that passes ON-1 and one per check that fails exactly that check (so each check is shown to bite).

### 9.5 What counts as "stuck" — the transcript audit

The harness saves the full transcript (JSON lines from the agent CLI). `tools/onboarding/audit.py` lists, with
the transcript line:

1. a read of any path outside the allowed set, or any web tool use — **the run is void** (INV-S25-7), not failed;
2. `STUCK` written, or the run ended by the turn or budget limit;
3. the same `mineworld` refusal met three times or more;
4. each `GAPS.txt` line;
5. a guess the agent states itself ("I'll assume", "probably", "not documented") — a phrase list in the audit
   script, reviewed by a person;
6. a field or key the agent wrote that the reference does not define (caught by S1, located here).

Every item 2–6 is a **guide defect**. The primary session (or the reviewing person) classifies each one in the
rounds ledger (§9.6) as: missing step, unclear step, missing reference description, misleading output example,
or a real product gap (a feature that does not exist — routed to the primary session, never patched in the
guide).

### 9.6 Rounds, and how failures become fixes

```text
round k:  assemble sandbox at commit C_k  →  run a NEW fresh agent on ON-1 (and ON-2/3 if authorized)
          →  score  →  audit  →  ledger row per stuck point
          →  fix the guide/reference (one commit per defect class), CI green
          →  round k+1 at commit C_{k+1}, with a new agent that has never seen round k
stop:     the first round in which ON-1 passes with zero stuck points  →  S25 acceptance evidence
```

The rounds ledger lives in 25f's PR design (one authority), a table: round, commit, agent command and model,
instruction set, score per check, stuck points (each with transcript line, class, and the fixing commit), tokens
and cost. Transcripts are kept as PR evidence outside the repository (they can be large); the ledger records
their SHA-256.

A fix that changes the guide is executed by the runner like any other page edit (§8), so a fix cannot break an
earlier chapter unnoticed. A "real product gap" stops the round; it is not fixed by S25.

### 9.7 Cost bound

- Per run: at most 80 agent turns and a dollar or token ceiling the operator sets (ruled QS25-3: at most USD 5 per run on a
  Sonnet-class model; **verify** the agent CLI's budget flag at 25f C0). The harness refuses to start without a
  ceiling.
- Per step: at most **5 rounds** of ON-1 before S25 stops and returns to the operator with the ledger. ON-2/ON-3
  only on explicit authorization, one run each per round.
- Model: the operator's choice (QS25-3). Recommended: gate on one Sonnet-class model; additionally report one
  Haiku-class run per round as a "harder reader" signal, not gating.
- CI never runs the harness (INV-S25-6). The harness's only CI presence is the scorer's and auditor's own tests
  on fixtures.

---

## 10. PR breakdown

### 10.1 Order and parallelism

```text
           25a reference (Rust) ─────┐
now:       25b reference (cognition) ┼──► 25d tutorial + corner-shop ──► 25e comparisons + ch. 09–11 ──► 25f gating rounds
           25c guide runner ─────────┘                                   (each part after its feature)
                                         25f harness + scorer (code) can start after 25c
features:  FG-a, DL-a, DL-b, WE-a, FG-b, FG-c (MVP-0, frozen)   P6 (not designed; QB-15)
```

25a, 25b and 25c have no dependency on each other and none on the open features. 25d needs 25c (to be executed)
and 25a (to link the reference). 25e is split by feature: each comparison and chapter merges after its feature.
25f's code needs 25c only; its gating run needs 25d, 25e complete, and DL-a, WE-a and P6 merged.

### 10.2 Implementer guidance (applies to every PR)

- Each PR is one fresh implementation session in its own worktree, after its own freeze; its first commit details
  it to the commit in its PR design (`overall.md` §5 ruling 9). Ruled 2026-10-14: 25a (SDK macros, a public
  contract) goes to an **Opus-class** implementer; 25b, 25c, 25d, 25e and 25f go to **Haiku-class** implementers.
  Each starts only on the primary session's dispatch.
- Every commit tracks **Implementation**, **Validation** and **LLM logic review** as separate `[ ]` items with
  evidence (`CLAUDE.md` §3.1).
- Standard checks per commit: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  the touched crates' tests; for Python, `uv run --locked ruff check`, `ruff format --check`, `pyright`, the
  package's pytest; for docs, `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`.
- No `sed -i`, no inline Python, no heredocs in commands an implementer runs; scripts are committed files.

### 10.3 The PRs

#### PR 25a — The generated reference, Rust side

**Scope.** `schemars` in the workspace; `JsonSchema` on every decoding type a creator's file reaches; the SDK
macro questions; `mineworld reference`; `docs/reference/` checked in; the drift check in CI.
**Dependencies.** None. Follow-up commit after DL-a merges (`decisions.md`) and after WE-a merges (happenings and
economy configuration appear automatically; the commit only regenerates and reviews descriptions).
**Acceptance.**
- AC-25a-1 `mineworld reference --check docs/reference` exits 0 on the PR head, on Linux, macOS and Windows.
- AC-25a-2 Changing one doc comment of one section field (planted, then reverted) makes the check exit 1 and
  name the file.
- AC-25a-3 Adding a field to a section type without a doc comment fails generation, naming the type and field.
- AC-25a-4 Every section, configuration and Interaction List section of every pack in the installed set appears;
  a test compares the reference's pack list with `Capability::AVAILABLE`.
- AC-25a-5 Every reference world's AC-8 record is byte-identical to the base (`python3 scripts/ci_parity.py
  baseline check`); INV-S25-5.
- AC-25a-6 For each documented section and configuration, the example the reference prints is accepted by the
  loader (a test loads each example into a minimal world); and one invalid example per page is refused with the
  message the reference predicts.

Commit plan:

- **C0 — Specs and decisions first (Markdown only).**
  - [ ] Implementation (`docs/GUIDE_FORMAT.md` belongs to 25c, not here): write `ARC-90`'s reference part and
    `DEP-49`'s records into `docs/DECISIONS.md` under the numbers the primary session assigns; amends
    `MODULE_SPEC.md` §4.1 and §4.2 to say the field-level reference is generated at `docs/reference/` and that
    the spec states shape and rules, not every field; adds `docs/reference/` to `docs/README.md`.
  - [ ] Validation: `check_doc_headings.py`, `check_decision_ids.py`; `cargo search schemars` and `cargo deny
    check licenses` after the add, recording version and licence (licence and maintenance verified at freeze, §16).
  - [ ] LLM review: the records state both halves of the reuse question; no field rule moved out of a spec.
- **C1 — schemars on the framework's own files.**
  - [ ] Implementation: add `schemars = { version = "<C0's>", default-features = false, features = ["derive"] }`
    (exact features per C0's audit) to `[workspace.dependencies]` with a DEP comment as the file's other entries
    have; derive `JsonSchema` on `WorldManifest`, `WorldIdentity`, `AuthoredPassage`, `AuthoredLocation`
    (`worldpack/src/format.rs`), `PackFile`, `StyleManifest` (`packages/src/manifest.rs`), the licence policy
    type (`packages/src/policy.rs`); manual impls for `ConfigurationKey` and the content files' framework fields
    (F-S25-3); `#[schemars(with = "String")]` + pattern for contract newtypes.
  - [ ] Validation: a test per manual impl that the schema's `properties` and `required` lists equal the fields the
    loader accepts and requires (no JSON Schema validator crate is added; C4's example-loading test is the
    end-to-end check); `cargo test -p mineworld-worldpack -p mineworld-packages`.
  - [ ] LLM review: no `Deserialize` behaviour changed (a diff shows only added derives and attributes).
- **C2 — The SDK: section, configuration and parameter schemas.**
  - [ ] Implementation: add the `JsonSchema` bound to `AuthoredSection::Authored` (`authoring/src/section.rs:181`)
    and `PackConfiguration::Configuration` (`authoring/src/configuration.rs:93`); extend `parameters!`
    (`sdk/rust/src/interactions/section.rs:94`) to emit `JsonSchema` for `$Name` and `$Partial` with `default`,
    `minimum`, `maximum`, `description`; add `Declarations` in `sdk/rust/src/interactions/` built from
    `ACTIONS`, `FACTS`, `PARAMETER_ROLES` and the knob/parameter schemas; extend `installed!`
    (`sdk/rust/src/installed.rs`) with `section_schema`, `configuration_schema`, `interaction_declarations`.
  - [ ] Validation: `cargo check --workspace` fails until C3 (expected; C2 and C3 may be squashed if the
    implementer prefers one green commit — record it); `trybuild` case: a section type without `JsonSchema` is
    refused at the installed list with a readable error.
  - [ ] LLM review: the macro still names no pack; the public-contract change is exactly the bound (QS25-2).
- **C3 — Every pack derives its schemas.**
  - [ ] Implementation: derive `JsonSchema` on each `Authored` type (`systems/{naming,schedule,item,inventory,
    economy,employment,bodies}/src/section.rs`), each `Configuration` (`systems/{calendar,weather}`), and their
    nested types; write an author-facing doc comment on every field (F-S25-4); update `acme-fishing` in its own
    repository and re-pin its two lines (`systems/installed/Cargo.toml`, `src/lib.rs`).
  - [ ] Validation: `cargo test --workspace`; `tests/acceptance/tests/package_sources.rs` still green with the new
    pin; AC-8 baseline check.
  - [ ] LLM review: descriptions say what the field means to an author and its unit (millimetres, seconds, minor
    currency units), never how the code uses it.
- **C4 — `mineworld reference`.**
  - [ ] Implementation: `tools/cli/src/reference.rs` (new): collect schemas through `Capability::AVAILABLE`, render
    the pages of §5.1 (Markdown tables: field, type, required, default, bounds, description; one minimal example
    per entry from the schema's `examples` or a per-type `const EXAMPLE: &str` the pack provides — decide in C4
    after measuring which is less repetition, record it), write `schema/*.schema.json` with sorted keys; `cli.md`
    by walking clap's `Command`; `--out` and `--check`; register the subcommand in `tools/cli/src/main.rs`.
  - [ ] Validation: `tools/cli/tests/reference.rs`: AC-25a-1, -3, -4, -6 (load each example; refuse each invalid
    one with the predicted message); determinism (two runs byte-equal; on Windows `\n` only).
  - [ ] LLM review: the command loads no world, writes only under `--out`, and refuses an `--out` that holds files
    it did not generate (no accidental deletion).
- **C5 — Check in the reference; CI.**
  - [ ] Implementation: generate `docs/reference/`; add the `--check` test to the `test` layer
    (`scripts/ci_layer.py`); classify `docs/reference/**` as generated in `scripts/ci_changes.py` so a hand edit runs
    the check; banner per page.
  - [ ] Validation: AC-25a-2 by a planted doc-comment change (shown red, reverted); AC-25a-5.
  - [ ] LLM review: the checked-in reference reads correctly to a person who has not seen the code (sample three
    entries and say so in the evidence).
- **C6 (after DL-a merges) — `decisions.md`.**
  - [ ] Implementation: DL-a's decision-list types and each model's `Parameters` derive `JsonSchema`; the model
    registry gains a schema question; regenerate.
  - [ ] Validation: AC-25a-1, -6 for `decisions.yaml`.
  - [ ] LLM review: the page names models, never backends (MODULE_SPEC §4.1).

#### PR 25b — The generated reference, cognition side

**Scope.** `cognition-toml.md`, later `memory-policies.md`, and their schemas, from Pydantic; a pytest drift check.
**Dependencies.** None for `cognition.toml`; FG-b for memory policies (second commit).
**Acceptance.** AC-25b-1 the pytest check passes and a planted `Field(description=…)` change fails it; AC-25b-2
every field of the configuration models has a description (the generator refuses one without); AC-25b-3
every example the generated page prints loads through `config.py`'s own loader (no JSON Schema validator is added).

Commit plan:

- **C0 — Spec note.** [ ] `MODULE_SPEC.md` §5.2 and `cognition/lm-controller/README.md` link the generated pages.
  [ ] `check_doc_headings.py`. [ ] Review: README stays short.
- **C1 — Generator.** [ ] `cognition/lm-controller/tools/reference.py` (a committed script, run as `uv run --locked
  python cognition/lm-controller/tools/reference.py --out docs/reference`), rendering the same table format as
  25a's pages (copy the column order from 25a's `sections.md`; if 25a has not merged, from §5.1 of this document)
  from `model_json_schema()` of the `config.py` models; add missing `Field(description=…)`.
  [ ] `tests/test_reference.py`: regenerate into a temp dir and compare with `docs/reference/`; AC-25b-1..3;
  ruff, pyright strict. [ ] Review: no secret or key value is ever an example; `key_env` examples are variable
  names.
- **C2 (after FG-b) — Memory policies.** [ ] The bundled policies' parameter models; `memory-policies.md`.
  [ ] Same test. [ ] Review: policy ids match the entry points DL-a resolves.

#### PR 25c — The executable-guide runner

**Scope.** `docs/GUIDE_FORMAT.md` (the spec of §6.2 and §8); `tools/cli/tests/guide.rs` and its support module;
`docs/guide/features.toml`; one fixture page proving every block kind and directive; CI wiring.
**Dependencies.** None.
**Acceptance.**
- AC-25c-1 The fixture page runs green on Linux, macOS and Windows (`test`, `test-macos`, `test-windows`).
- AC-25c-2 Each of these planted defects turns the test red with a message naming page and step: wrong expected
  output; a non-`mineworld` command; an unknown directive; a `file=` block outside the scratch folder (`../x`);
  an `equals` mismatch; a mirror whose code block differs by one byte; a link to a missing reference anchor; a
  feature flagged `false` that the build has.
- AC-25c-3 A `requires` block for a `false` feature is reported as pending by name and not counted as passed.
- AC-25c-4 No shell is spawned (the runner calls the binary directly; a test asserts a `$ mineworld x | y` block is
  refused).

Commit plan:

- **C0 — Spec and decision.** [x] `docs/GUIDE_FORMAT.md`: block grammar, directives, page rules 1–8 of §6.2, mirror
  parity, the pending rule; `DEP-50`'s record; `docs/README.md` row (commit 489012b). [x] doc checks; versions
  declared in the root `Cargo.toml` (`pulldown-cmark` 0.13.4, `snapbox` 1.2.2, `toml` 1.1.6); `cargo deny check
  licenses` → `licenses ok` on the merged head (R-LOAD-1 local gate). [x] Review: the grammar is unambiguous (every
  info string form listed; an unlisted one fails the page naming its line — test
  `an_info_string_token_the_guide_does_not_list_fails_the_page_naming_its_line`).
- **C1 — Parsing and file blocks.** [x] dev-dependencies on `pulldown-cmark` and `snapbox` in `tools/cli/Cargo.toml`
  (workspace entries with DEP comments); `tools/cli/tests/guide/` module: page walk, `file=` writing with path
  confinement, directive parsing, `features.toml` reading. `toml` crate used (already in the lock, DEP recorded).
  (commit d30f121). [x] Unit-level tests of the parser on inline strings, including CRLF input
  (`crlf_pages_are_read_as_lf_so_the_same_blocks_run_on_every_checkout`). [x] Review: path confinement (absolute
  paths, `..`, symlinks refused — tests `a_block_path_outside_the_scratch_folder_is_refused_by_the_page_reader`,
  `a_symbolic_link_already_in_the_scratch_folder_is_not_followed`).
- **C2 — Command blocks and matching.** [x] Run `CARGO_BIN_EXE_mineworld` with word-split arguments, cwd = scratch
  folder, merged stdout/stderr; `exit=N`; `snapbox` matching (commit d555f79). [x] The fixture page
  `tools/cli/tests/guide/fixture.md` (create → validate → run 1 day → `equals`);
  `the_fixture_page_runs_green_against_the_real_binary`. [x] Review: output normalisation is limited to line endings
  and path separators.
- **C3 — Guards.** [x] `equals`, mirror parity, reference-anchor check (pending note until 25a has merged — test
  `a_link_into_a_reference_that_is_not_yet_generated_is_inconclusive_not_passed`), features-versus-build check (via
  `mineworld reference`/`packs list`). [x] AC-25c-2's planted defects as tests that assert failure (commit 4742d4e;
  34/34 PASS on the merged head). [x] Review: each guard's message names the page, step or folder to fix.
- **C4 — CI and cost.** [x] The guide test runs in `test`, `test-macos`, `test-windows`: `cargo test --workspace` is
  the `core` layer (`scripts/ci_layer.py`), and `tools/cli/tests/guide.rs` is discovered without filtering; local
  guide wall time 0.5–1.0 s. [ ] Workspace gate: CI run <id> on <sha> (R-LOAD-1), pending. Local full workspace run
  was started and stopped under R-LOAD-1 (load average above 200): before the stop, 49 suites, 241 passed, 0 failed,
  so it is superseded by CI. [ ] AC-25c-1 evidence (CI run ids for `test`, `test-macos`, `test-windows`), pending.
  [x] Review: §8.5 budget met locally; the nightly move is not needed for the PR leg.

#### PR 25d — The tutorial and the tiny example world

**Scope.** `docs/guide/README.md`, chapters 01–08, 12, 13 (first version), their zh-CN mirrors,
`docs/guide/examples/corner-shop/`, links from `README.md`, `docs/README.md`, `tools/cli/README.md` and the
`mineworld create` template's comments (pointing at the guide instead of `MODULE_SPEC.md`).
**Dependencies.** 25c merged; 25a merged (reference links). Chapter 07 needs IL-e (merged).
**Acceptance.**
- AC-25d-1 The guide test runs every block of chapters 01–08 and 12 green on the three OSes; chapter 06's
  client steps are `operator-only` and listed.
- AC-25d-2 `equals examples/corner-shop` passes after chapter 05 (and after 07 and 08 for their own example
  states, if those chapters extend the world — decide in C0: recommended, the example is the state after 05, and
  07–08 work in copies).
- AC-25d-3 Mirror parity passes for every page; the primary session (or a Chinese-reading reviewer) signs off the
  plain-language review of the zh-CN pages; the English pages pass the plain-language rule (§6.2 rule 7) by a
  reviewer's read-through recorded in evidence.
- AC-25d-4 A person (the operator, at the milestone handoff) follows chapters 01–05 by hand on one OS and reaches
  every "You should see" — the human check that the CI check cannot make.
- AC-25d-5 No page states a field rule without a reference link (review checklist over every page).

Commit plan (one chapter per commit keeps each reviewable; Haiku-sized). **Progress (PR 25d, implementing session
2026-10-15, Opus-class by the primary's dispatch):** every item below is marked against the evidence ledger, §18.
The chapters were committed together rather than one per commit (deviation D-25d-5, §18.2).

- **C0 — Index, page template, example skeleton.** [x] `docs/guide/README.md` + `README.zh-CN.md` with the
  chapter table and "coming" rows for 09–11; the `CLAUDE.md` §2.1 class was already applied by the primary
  (QS25-1); index links from `README.md`, `README.zh-CN.md`, `docs/README.md`, `tools/cli/README.md`. [x] doc
  checks; guide test (§18.1 E-1, E-4). [x] Review: the index is one screen with one main line (E-9).
- **C1 — Chapter 01.** [x] Page + mirror. No release bundle exists yet, so the binary is built from source
  (non-executed text, per-OS `PATH` lines); `mineworld --version` with `[..]`; decided: the scratch folder has no
  `worlds/`, so chapter 01 runs a `create`d world `hello` for two days instead of `market-town` (D-25d-1).
  [x] Guide test (E-1). [x] Review: each command and each run flag explained once (E-9).
- **C2 — Chapter 02.** [x] Corner Shop is written by hand from an empty folder, one file per step, `validate`
  after each (the refusals name the next file), then `run --days 1` (D-25d-2: not `create corner-shop`).
  [x] Guide test (E-1). [x] Review: terms match CORE_CONCEPTS, each linked at first use (E-9).
- **C3 — Chapter 03.** [x] `shop` place, a passage with both ends in millimetres, Ana moved into the shop.
  Realistic-default note: no field takes a person radius or a doorway width (`AuthoredPassage` has `to`, `here`,
  `there` only), so the guide states only positions (D-25d-3). [x] Guide test (E-1); no partial `equals`
  checkpoint (not useful: the next chapter rewrites the same files). [x] Review (E-9).
- **C4 — Chapter 04.** [x] Cai; `name:`, `routine:`; `relationships`, `naming`, `schedule`; a saved 1-day run
  and `biography --person ana` (her acquaintances at 01:30 and 02:00, each listed once per direction, which
  `systems/relationships/src/event.rs` `BecameAcquainted` confirms). [x] Guide test (E-1). [x] Review (E-9).
- **C5 — Chapter 05.** [x] Items, organization, shop, wallets, Ana's job, consumption; `run --days 3`; `equals
  corner-shop docs/guide/examples/corner-shop` on step 5.7. [x] Guide test; AC-25d-2 (E-1, E-2). [x] Review: the
  sizing sentence is in step 5.8 (E-9).
- **C6 — Chapter 06.** [x] Executed `validate` of the seat list; `mineworld server corner-shop --town` and the
  2D client are `operator-only`/shown text. [x] Guide test lists the operator-only block (E-1). [x] Review:
  `PLAY.txt` does not exist yet (step-23 R-d not merged), so the client text follows `clients/2d/README.md`
  and was checked against a real join (E-6); the 3D client is marked "coming" (D-25d-4).
- **C7 — Chapter 07.** [x] Class `shopkeeper`, a `gap` parameter, a `shop` region forbidding `talk` by a
  shopkeeper; `interactions --place shop`; run before/after with exact numbers. [x] Guide test (E-1).
  [x] Review: only actions, roles and parameters `interaction-list.md` lists (E-9).
- **C8 — Chapter 08.** [x] `calendar` (San Diego) and `weather` (`source: rules`, Market Town's fitted San
  Diego months); `run --save`, `perceived --json`. [x] Guide test (E-1). [x] Review: no data file is attached
  (`source: rules`), so the NOAA credit is carried by the file's header comment and the page; Market Town's
  `data/weather/NOTICE` covers the record itself (E-9).
- **C9 — Chapters 12 and 13; template comments.** [x] `packs list|resolve`; a reader-made Entity Pack
  `corner-goods` required with `--packs` (D-25d-6: `modern-goods` is described, not executed); chapter 13 is an
  executed break-and-fix page covering every "If you see" message plus a table of the rest; template comments
  point at the guide and the reference. [x] Guide test; `create` and `packs` tests green (E-1, E-5).
  [x] Review (E-9).
- **C10 — zh-CN pass and close.** [x] All mirrors written together with one term table in `README.zh-CN.md`
  (English term in brackets). [x] Mirror parity (E-1); AC-25d-3: every mirror revised to the operator's
  plain-Chinese style on the coordinator's dispatch, 2026-10-16 (E-10). [x] Review (E-9, E-10).

#### PR 25e — Comparison examples and the feature chapters

**Scope.** `docs/guide/compare/*`, chapters 09–11, `features.toml` flips. Split into one sub-PR per feature
(25e-1 merged features: `a-forbidden-action`, `weather`; 25e-2 FG-a: `relationship-decay` + chapter 10 part 1;
25e-3 WE-a: `market-day` + chapter 11; 25e-4 DL-a/DL-b: chapter 09 parts 1–2; 25e-5 FG-b/FG-c: chapter 10 part 2;
25e-6 P6: `one-person-on-a-model` + chapter 09 part 3).
**Dependencies.** 25d; each feature PR for its sub-PR.
**Acceptance (each sub-PR).** AC-25e-1 its pages run green (or `operator-only` where a live model is needed, with
recorded output and its date and model); AC-25e-2 the "world changes like this" difference is shown by real output
of the same seed and days before and after; AC-25e-3 its feature flag flips to `true` in the same commit as its
pages; AC-25e-4 F-S25-6 audited: the difference is visible through an existing read-only command, or a finding is
raised to the primary session and the page waits.

Commit plan (per sub-PR): **C0** audit which command shows the difference and record the before/after output on
the base [ ] [ ] [ ]; **C1** the comparison page + mirror [ ] guide test [ ] review: one change only, whole file
shown; **C2** the chapter section + mirror, flag flip [ ] guide test [ ] review: the rule links the reference.

**Progress (PR 25e, implementing session 2026-10-16, Opus-class by the primary's dispatch):** one PR covers the
sub-PRs whose features are on its stack, 25e-1, 25e-2 (part 1) and 25e-4 (D-25e-1); 25e-3, 25e-5 and 25e-6 wait for
their features and are listed as coming. Every item is marked against the evidence ledger, §19. C0's runner and C2 are one
commit: a new probe and the flag it checks must change together, or the head between them is red.

- **C0 — Audit (F-S25-6) and the runner.** [x] For each comparison, the command that shows the difference was found
  and its before/after output recorded on the base (§19.3). [x] The runner walks `compare/`, gains `example`, and its
  presence probe reads decision models and the generated reference as well as packs; `GUIDE_FORMAT.md` §§1, 1.1, 6,
  9, 9.1 (D-25e-2, D-25e-3); three new runner tests (E-1, E-2). [x] Review: no rule moved into the runner; every probe
  is a question to the real binary (E-6).
- **C1 — Comparison pages + mirrors.** [x] `compare/README.md`, `a-forbidden-action`, `weather`, `relationship-decay`,
  each with its mirror (E-1). [x] Guide test (E-1). [x] Review: one change only, whole files shown (E-6).
- **C2 — Chapters 09 and 10 + mirrors, flag flips.** [x] Chapter 09 parts 1–2 (`rule`, `motivated`), chapter 10 part 1
  (relationship decay); `decisions`, `motivated`, `relationship-decay` flipped to `true` with their probes in the same
  commit (AC-25e-3); index rows and chapter 08's "Next" link (E-1, E-3). [x] Guide test (E-1). [x] Review: every rule
  links the reference (E-6).
- **C3 — F-25d-2.** [x] `scripts/ci_changes.py` classifies every file a guide page links to as code (E-4). [x]
  Self-test (E-4). [x] Review: the change only narrows the docs set, so it fails closed (E-6).

#### PR 25f — The onboarding harness, the scorer, and the rounds

**Scope.** `tools/onboarding/` (assemble, run, score, audit, instruction sets, rubrics, fixtures), a container
recipe for H1, the operator's run instructions, the rounds ledger, and the gating run(s).
**Dependencies.** Code: 25c. Gating run: 25d, all of 25e, and DL-a, WE-a and P6 merged; operator authorization and
budget for each round.
**Acceptance.**
- AC-25f-1 The scorer's fixture tests pass in CI: the passing ON-1 fixture scores pass; each per-check fixture fails
  exactly its check (S1–S6).
- AC-25f-2 The auditor's fixture tests: a transcript with an out-of-sandbox read is void; one with a repeated
  refusal lists it; one with a `GAPS.txt` line lists it.
- AC-25f-3 `assemble.py` produces a sandbox containing exactly the allowed set (a test lists the files).
- AC-25f-4 `run.py` refuses to start without `--budget` and `--agent-command`, and never runs under CI (refuses when
  `CI` is set).
- **AC-25f-5 (S25's acceptance).** A round in which a new fresh agent, given ON-1, produces a World Pack that passes
  S1–S6 in CI-equivalent scoring and S7 on the operator's machine, writes `DONE`, and has zero stuck points in the
  audit — at most 5 rounds after the first gating round (otherwise S25 returns to the operator).

Commit plan:

- **C0 — Spec, verify items.** [ ] `tools/onboarding/PROTOCOL.md` (a spec: §9 of this document made exact: allowed
  set, preamble, rubrics, stuck classes, void rule, ledger format, budget); verify the agent CLI's flags for
  isolation, turn limit and budget, with sources and dates. [ ] doc checks. [ ] Review: INV-S25-7 is enforceable
  with the chosen flags, or the container is mandatory.
- **C1 — Assemble.** [ ] `tools/onboarding/assemble.py SANDBOX --binary PATH`: copies `docs/guide` (English only),
  `docs/reference`, the binary, the bundled packs; writes nothing else. [ ] `tools/onboarding/tests/test_assemble.py`
  (AC-25f-3); ruff, pyright. [ ] Review: no repository path leaks into the sandbox (scan copied files for the
  repository's absolute path).
- **C2 — Scorer.** [ ] `score.py` with rubric files `rubrics/ON-1.toml` …; fixtures under
  `tools/onboarding/tests/fixtures/`. [ ] AC-25f-1; the fixtures are themselves validated by the binary.
  [ ] Review: each check reads command output, never the agent's claims.
- **C3 — Auditor.** [ ] `audit.py TRANSCRIPT --sandbox SANDBOX`; the guess-phrase list as a data file.
  [ ] AC-25f-2 on synthetic transcripts. [ ] Review: the void rule cannot be bypassed by a relative path or a
  symlink.
- **C4 — Runner and container.** [ ] `run.py --instructions ON-1 --agent-command '<cmd>' --budget <amount>
  --out EVIDENCE_DIR` (assemble → start agent → collect `work/`, `STATUS`, `GAPS.txt`, transcript → score → audit →
  print a ledger row); `tools/onboarding/Containerfile` for H1. [ ] AC-25f-4; a dry run with a scripted fake agent
  (a committed script that writes the passing fixture into `work/`) proves the pipeline end to end without a model.
  [ ] Review: no secret is written to evidence; the API key comes only from the operator's environment.
- **C5… — Rounds.** Per round: [ ] operator authorizes and runs (or authorizes the session to run) `run.py`;
  [ ] ledger row with score, stuck points, cost; [ ] one fixing commit per defect class in `docs/guide/` or doc
  comments (then regenerate the reference), each green in the guide test; [ ] review of each fix: it fixes the
  class, not the one sentence the agent tripped on. Stop at AC-25f-5 or after 5 rounds.

**Progress (PR 25f, implementing session 2026-10-16/17, Opus-class by the primary's dispatch):** marked against the
ledger, §20. C0–C4 landed as one commit (the protocol, the tools and their tests are one reviewable unit; D-25f-1).

- **C0** [x] `tools/onboarding/PROTOCOL.md`; agent CLI flags verified (§20.1). [x] doc checks (E-5). [x] Review:
  INV-S25-7 is enforced before the act by the allowlist and the guard, proved by the planted escape test, with the
  audit as a second layer (§20.5); H1 not built (D-25f-2).
- **C1** [x] `assemble.py` (E-1). [x] AC-25f-3 test; ruff, pyright strict (E-1, E-4). [x] Review: no repository path
  in the sandbox (`leaks`, tested).
- **C2** [x] `score.py`, `rubrics/ON-{1,2,3}.toml`, the ON-1 fixture (validated by the binary) (E-1). [x] AC-25f-1 as
  far as this build allows (D-25f-3). [x] Review: every check reads the binary's output or the pack's files.
- **C3** [x] `audit.py`, `guess_phrases.txt` (E-1). [x] AC-25f-2 (E-1). [x] Review: relative paths, `cd`, `~`,
  symbolic links and linked folders covered by tests.
- **C4** [x] `run.py`, `fake_agent.py` (E-1). [x] AC-25f-4; the scripted dry run (E-1). [x] Review: every `*_API_KEY`
  removed from the agent's environment; no credential read or written.
- **C5** Rounds 1 and 2 (§20.3). AC-25f-5 is not reachable on this stack: S5 and S6 are product gaps (F-S25-5).

### 10.4 MVP-0 placement

S25 is part of MVP-0 and lands **before step-23's publishing PR R-d**, so the first public release carries a guide
that has passed the onboarding test. R-a, R-b and R-c do not wait for S25. The guide is published from the
repository at the release tag; `PLAY.txt` (step-23 §3) gains one line with the guide's URL at that tag (QS25-8).
Proposed as an `overall.md` §3 step entry (§12.2).

---

## 11. Decision records (numbers assigned 2026-10-14)

| Placeholder | Decision | Alternatives recorded | Written by |
| --- | --- | --- | --- |
| **ARC-90** | Creator documentation: `docs/guide/` is a third documentation class (guides: human and newcomer-agent audience, plain language, executed by CI, never an authority); the field-level reference is generated from the decoding types into `docs/reference/` and never hand-edited; `mineworld reference` is a read-only subcommand; guide pages execute in CI with pending features listed, not skipped; the onboarding test is S25's acceptance procedure | G2, G3 (§4.3); R2–R5 (§4.1); H2, H3 (§4.4) | 25a C0 (reference part), 25c C0 (guide part) |
| **DEP-49** | Adopt `schemars` (1.x) for JSON Schema derivation from the decoding types; the contracts crate stays free of it; `clap-markdown` declined in favour of walking clap's model | R2 own derive (reinvention), R3 rustdoc JSON (nightly), R5 schema-first (inverts the seam); `clap-markdown` 0.1 | 25a C0 |
| **DEP-50** | Adopt `pulldown-cmark` and `snapbox` as dev-dependencies of `mineworld-cli` for the executable guide; `trycmd` declined as the runner (per-case sandboxes do not model a cumulative tutorial), mdBook test and a Python runner declined | E1, E3, E4, E5 (§4.2) | 25c C0 |

Assigned by the primary session on 2026-10-14 (placeholders `ARC-S25-a`, `DEP-S25-a`, `DEP-S25-b` of revision 1).
Next free: `ARC-91`, `DEP-51`.

---

## 12. Proposed edits to documents this session may not edit

### 12.1 `CLAUDE.md` §2.1 (operator, if QS25-1 is ruled G1)

Add a row to the table: "Files under `docs/guide/` | humans and agents new to the project | A guide: plain
language, one action per step in the form *Do this → You should see this*, every instruction executed by CI
(`docs/GUIDE_FORMAT.md`). Never the authority for a rule: it links the specification or the generated reference.
Its `*.zh-CN.md` mirrors follow rule 5." Add to the rules: "Files under `docs/reference/` are generated; edit the
type's doc comment and regenerate."

### 12.2 `overall.md`

- §3: a new step entry "S25 — Creator documentation, tutorials and the onboarding test", pointing here, placed
  before S23's R-d.
- §4 requirement coverage: the operator's requirement of 2026-10-14 → S25.
- §5: the operator's requirement (§1.1) and, when ruled, §13's answers; the decision ids assigned.
- §6 risks: R-3 (YAML schemas drift from the Rust contracts) gains "S25 generates the reference from the decoding
  types and CI checks it".

### 12.3 `docs/MODULE_SPEC.md`

§4.1 and §4.2: a sentence that the per-field reference is generated (`docs/reference/`), and that the spec states
the shape and the rules. §8.1: `mineworld reference --out DIR | --check DIR`. Written by 25a C0.

### 12.4 Other plans

- `step-17-cognition.md` (P6's future design): requirement R-P6-S25 — a whole-run replay path for an `lm` seat, so
  the language-model comparison and chapter 09 run in CI without a model; and a run report saying how many
  decisions each seat's model made, for the scorer's S7.
- `pr-dl-a-decision-list.md`: note that the decision-list types and model `Parameters` will derive `JsonSchema` (25a
  C6); no change to DL-a's scope.
- `pr-we-a-happenings.md`: note that the happenings fact type is what the scorer's S5 reads; no change to scope.
- `step-23-release.md` §3: `PLAY.txt` gains the guide URL line (QS25-8).

---

## 13. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS25-1** | Is `docs/guide/` a third documentation class in `CLAUDE.md` §2.1 (G1), or are guides specs written plainly (G2)? | **G1.** It says what the files are, keeps READMEs short and specs complete, and gives the plain-language rule a home |
| **QS25-2** | May the SDK's `AuthoredSection::Authored` and `PackConfiguration::Configuration` require `JsonSchema` (a public-contract change; `acme-fishing` updated and re-pinned in 25a)? | **Yes.** The compiler then refuses an undocumented pack; there is no stable contract yet to keep compatible (`CLAUDE.md` §4 item 12) |
| **QS25-3** | Which agent and model run the onboarding test, and what budget per run and per step? | **One Sonnet-class model through the Claude Code CLI in a container (H1), USD 5 and 80 turns per run, at most 5 gating rounds; one Haiku-class run per round reported, not gating.** The operator's API key from their own environment |
| **QS25-4** | Is only ON-1 (your example) gating, with ON-2 and ON-3 reported? And may the agent read the sample worlds? | **ON-1 gating; ON-2/ON-3 reported, to catch a guide tuned to one prompt. Sample worlds not readable** — copying Market Town would test Market Town, not the guide |
| **QS25-5** | Should the guide show editor completion through the generated JSON Schemas (a modeline in each YAML file)? | **Optional chapter-12 note only**; not in the example world's files, to keep them minimal |
| **QS25-6** | Is the generated reference English-only (zh-CN guide links it)? | **Yes.** A mirror of generated text would drift or be unreviewed |
| **QS25-7** | Does the tutorial's example world live in `docs/guide/examples/corner-shop/` rather than `worlds/`? | **Yes**: no AC-8 baseline or nightly cost; the guide test proves it runs on every OS anyway |
| **QS25-8** | The bundle refuses `.md` files (step-23 A-R3). Publish the guide only from the repository at the tag and link it from `PLAY.txt`, or render it into the bundle (HTML or text)? | **Link from `PLAY.txt`** for the first release; a rendered copy in the bundle is a later choice that needs a renderer decision |
| **QS25-9** | S25's acceptance needs P6 (one person on the local model), which is not yet designed and waits on QB-15. Hold S25's acceptance until P6 merges, or accept S25 on ON-1 without the model clause and re-run when P6 lands? | **Hold the acceptance; build everything else now.** The operator's example names the local model; accepting without it would accept a different test. 25a–25e proceed and the harness is ready the day P6 merges |
| **QS25-10** | Should a person also follow chapters 01–05 by hand at the milestone handoff (AC-25d-4)? | **Yes, once, on one OS**: an agent reads literally; a person catches what is confusing rather than wrong |

### 13.1 Rulings, 2026-10-14 (relayed by the primary session)

| Id | Ruling | By |
| --- | --- | --- |
| QS25-1 | **Yes (G1):** `docs/guide/` becomes a third documentation class. The primary session applies the `CLAUDE.md` §2.1 edit (§12.1) in its own PR; no S25 PR edits `CLAUDE.md`. 25d C0 depends on that PR having merged, or links it | operator |
| QS25-2 | **Yes:** the SDK requires `JsonSchema` on `AuthoredSection::Authored` and `PackConfiguration::Configuration`; `acme-fishing` is updated in its own repository and re-pinned in 25a (C3) | operator |
| QS25-3 | **As recommended:** a Sonnet-class model through the Claude Code CLI in a container (H1); at most USD 5 and 80 turns per run; at most 5 gating rounds; one Haiku-class run per round, reported only; operator-started, never in CI | operator |
| QS25-4 | As recommended: ON-1 gating; ON-2 and ON-3 reported; the sample worlds are not in the allowed set | primary |
| QS25-5 | As recommended: editor completion is an optional note in chapter 12 only | primary |
| QS25-6 | As recommended: the generated reference is English-only | primary |
| QS25-7 | As recommended: the example world lives in `docs/guide/examples/corner-shop/` | primary |
| QS25-8 | As recommended: the guide is published from the repository at the tag and linked from `PLAY.txt` | primary |
| QS25-9 | **Yes:** S25's acceptance (AC-25f-5) is held until P6 merges; 25a–25e and 25f's code proceed now | operator |
| QS25-10 | **Yes:** the operator follows chapters 01–05 by hand once, on one OS, at the milestone handoff (AC-25d-4) | operator |

Decision ids assigned: `ARC-90`, `DEP-49`, `DEP-50`. Implementers: 25a Opus-class; 25b–25f Haiku-class; each on
the primary session's dispatch.

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S25-1 | Doc comments written for the reference read as developer notes, so the generated reference is accurate but unhelpful | 25a requires a description per field and reviews them as author-facing (F-S25-4); the onboarding audit's "missing reference description" class points at each weak one |
| R-S25-2 | The guide's quoted output changes whenever a pack's behaviour changes, making every behaviour PR also a guide PR | `...` and `[..]` keep quotes to what the reader compares; quotes show headers and named lines, not full traces. A behaviour change that alters a quoted line is a real guide change and is correct to surface |
| R-S25-3 | The guide test's wall time grows with chapters, slowing every PR on three OSes | §8.5 budget; long pages move to nightly; worlds in the guide stay tiny |
| R-S25-4 | The onboarding agent passes by luck or by prior knowledge of MineWorld from training data | Fresh sandbox, no web; ON-2/ON-3 as held-out prompts; the audit treats an unexplained correct guess as a stuck point (the guide still did not tell it) |
| R-S25-5 | The guide drifts into an authority (a rule stated only in a guide) | INV-S25-1; review rule 8 of §6.2; the reference-link guard |
| R-S25-6 | The zh-CN mirror lags or diverges | Mirror parity in CI (structure and code blocks); one reviewer pass per PR |
| R-S25-7 | Feature PRs (DL-a, WE-a, FG-a, P6) slip, so S25 never closes | 25a–25d and 25e-1 close independently; the acceptance (QS25-9) is the only part that waits, and is explicit |
| R-S25-8 | The isolation flags of the agent CLI change or do not cover user-level context | Container by default (H1); the void rule in the audit; verify items at 25f C0 |
| R-S25-9 | A comparison needs a difference no existing command shows (F-S25-6) and the fix creeps into a new command | Stated as out of scope; raised to the primary session with evidence |

---

## 15. Adversarial criteria for the step (decided before measuring)

| Id | Criterion | How it is made to fail |
| --- | --- | --- |
| A-S25-1 | The reference cannot drift | plant a doc-comment change, a new field without a description, a new pack without a schema: each red |
| A-S25-2 | The guide cannot drift | plant a changed expected line, a wrong file, an example-directory mismatch, a shell command: each red |
| A-S25-3 | A pending step is never a pass | flag a merged feature `false`: red; leave a `requires` block on a `false` feature: listed pending, not counted |
| A-S25-4 | S25 changes no world | AC-8 baselines byte-identical at every S25 PR head |
| A-S25-5 | The onboarding agent is fresh | a planted out-of-sandbox read in a fixture transcript makes the run void |
| A-S25-6 | The scorer bites | each per-check fixture fails exactly its check |
| A-S25-7 | No paid model in CI | `run.py` refuses under `CI`; no workflow references `tools/onboarding/run.py` (a scan in 25f) |

---

## 16. Sources (accessed 2026-10-14)

- crates.io via `cargo search`: `schemars 1.2.3`, `trycmd 1.2.1`, `snapbox 1.2.2`, `pulldown-cmark 0.13.4`,
  `clap-markdown 0.1.5`.
- Licences and maintenance, verified 2026-10-14 with `cargo info <crate>` (crates.io metadata) and `gh api
  repos/<owner>/<repo>` (archived flag, last push):

  | Crate | Version | Licence (crates.io) | Repository | Archived | Last push | MSRV |
  | --- | --- | --- | --- | --- | --- | --- |
  | `schemars` | 1.2.3 | MIT | `GREsau/schemars` | no | 2026-10-12 | 1.74 |
  | `pulldown-cmark` | 0.13.4 | MIT | `pulldown-cmark/pulldown-cmark` | no | 2026-10-12 | 1.71.1 |
  | `snapbox` | 1.2.2 | MIT OR Apache-2.0 | `assert-rs/snapbox` | no | 2026-10-04 | 1.85 |
  | `trycmd` (declined) | 1.2.1 | MIT OR Apache-2.0 | `assert-rs/snapbox` | no | 2026-10-04 | 1.85 |
  | `clap-markdown` (declined) | 0.1.5 | MIT OR Apache-2.0 | `ConnorGray/clap-markdown` | — | — | unknown |

  All are within the workspace toolchain (1.97.1) and `deny.toml`'s permissive licences. Exact feature sets
  stay to be chosen at 25a C1 and 25c C1.
- Pydantic v2 `model_json_schema()`: already adopted (DEP-24); behaviour **verify** against the locked version at 25b
  C1.
- Agent CLI isolation, turn and budget flags: **verify** at 25f C0 against the CLI's own `--help` and documentation.
- Repository audit: `origin/main @ 171ea06`, paths and line numbers as cited in §2.

---

## 17. Evidence ledger — PR 25b (`mvp0/pr-s25b`)

Recorded by the implementing session (Haiku-class, 2026-10-14). Results are `PASS`, `FAIL` or `INCONCLUSIVE`
from actual output. Mutations are applied one at a time and reverted; the failing test name is the evidence.

| Commit | Item | Result | Evidence |
| --- | --- | --- | --- |
| C0 | Implementation: `MODULE_SPEC.md` §5.2 and `cognition/lm-controller/README.md` link the generated pages | PASS | the two edits; the `cognition-toml.md` link is valid only once C1 lands (same PR head) |
| C0 | Validation: `check_doc_headings.py`, `check_decision_ids.py` | PASS | `223 numbered sections across 31 documents, none duplicated`; `122 decision ids, all distinct` |
| C0 | LLM review: README stays short | PASS | one added clause and a link; no rule moved out of the spec |
| C1 | Verify item (design "verify"): Pydantic `model_json_schema()` on the locked 2.14.0 | PASS | probe of `CognitionConfig`: `Field(description=)` lands on the property; `dict[Annotated[str, Field(pattern)], ...]` emits `patternProperties`; `dict[Literal, str]` emits `propertyNames.enum`; `Annotated[int, Field(le)] \| None` puts bounds inside `anyOf`; a description written inside that `Annotated` does **not** reach the property, so it goes on the outer `Field(...)` (found by the generator, which refused `BackendConfig.request_timeout_s`) |
| C1 | Implementation: `Field(description=...)` on every field of `CognitionConfig`, its nested models and `BudgetPolicy` (`config.py`, `budget.py`) | PASS | ruff check, ruff format --check, pyright strict: `All checks passed`, `63 files already formatted`, `0 errors` |
| C1 | Implementation: `cognition/lm-controller/tools/reference.py`; `docs/reference/cognition-toml.md` and `docs/reference/schema/cognition-toml.schema.json` generated | PASS | `uv run --locked --package mineworld-cognition python cognition/lm-controller/tools/reference.py --out docs/reference` writes 2 files (exit 0). Deviation: the design's bare `uv run --locked python ...` is not enough to import `mineworld_cognition` from the virtual root; `--package mineworld-cognition` is needed |
| C1 | Validation AC-25b-1 (drift): `test_checked_in_reference_matches_the_models`, `test_generation_is_deterministic` | PASS | `pytest cognition/lm-controller/tests/test_reference.py`: 6 passed (with the other 4 below) |
| C1 | Validation AC-25b-2 (every field described): `test_every_field_of_the_configuration_models_has_a_description`, `test_a_field_without_a_description_stops_generation_naming_it` | PASS | `undescribed_fields(CognitionConfig) == []` |
| C1 | Validation AC-25b-3 (examples load): `test_every_example_on_the_page_is_loaded_by_the_loader`, `test_an_example_the_loader_refuses_stops_generation` | PASS | the page's TOML blocks load through `config.load`; the refusal test passes |
| C1 | Validation: full cognition suite | PASS | `pytest cognition/lm-controller`: 175 passed, 1 skipped (baseline before C1: 161 passed, 1 skipped, 8 errors from the missing `mineworld` binary, which is now built with `cargo build -p mineworld-cli`) |
| C1 | Mutation M1: a planted description change in `budget.py` | PASS (bites) | `FAILED test_checked_in_reference_matches_the_models`; restored |
| C1 | Mutation M2: a field's description removed (`BudgetConfig.call_timeout_s`) | PASS (bites) | generator and `test_a_field_without_...` name `BudgetConfig.call_timeout_s`; restored |
| C1 | Mutation M3: an unknown key added to an example in `reference.py` | PASS (bites) | `example 'A local model' is refused by the loader: ... backends.local.colour: Extra inputs are not permitted`; restored |
| C1 | LLM review: no secret or key value is an example; `key_env` examples are variable names | PASS | the examples use `ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY` names only; the `key_env` text states the key is never written in the file |
| C2 | Memory policies (`memory-policies.md`, after FG-b) | BLOCKED, not implemented | FG-b is design-only on `origin/main` (`e8abdda` freezes the design; no policy code exists under `cognition/lm-controller/src/mineworld_cognition/memory/`). The design makes C2 wait for FG-b's merge, so this PR does not contain it; `memory-policies.md` is absent from `docs/reference/` |
| C1 | Open issue for the primary (not resolved here): `mineworld reference --check docs/reference` (25a) treats extra files as drift. Once 25b lands, 25a's check on the same directory must know `cognition-toml.md` and `schema/cognition-toml.schema.json` (or the Python pages move to a subdirectory). Recorded, not changed: it crosses the 25a/25b file layout | open | — |

---

## 18. Evidence ledger — PR 25d (`mvp0/pr-25d`)

Recorded by the implementing session (Opus-class, dispatched by the primary, 2026-10-15). Worktree
`/Users/yuema137/mineworld-worktrees/impl-25d`, branch `mvp0/pr-25d` from `origin/mvp0/pr-s25a @ 589998e2` (25a's
head; 25c merged on `main` through it). Rust code on `origin/main` beyond 25a's base was audited: no change under
`tools/cli`, `systems`, `worldpack`, `kernel`, `authoring`, `sdk/rust` or `server` (`git diff --stat HEAD...origin/main`),
so every quoted output is the output of today's `main` plus 25a. Results are `PASS`, `FAIL` or `INCONCLUSIVE` from
actual output; local runs are targeted (R-LOAD-1), CI on the exact head is the gate.

### 18.1 Evidence

| Id | Item | Result | Evidence |
| --- | --- | --- | --- |
| E-1 | The whole guide against the real binary (`cargo test -p mineworld-cli --test guide`) | PASS (macOS, local) | `guide: 105 block(s) passed, 1 pending, 29 check(s)`; the one pending block is step 6.2 (`operator-only`); every page's mirror and links `PASS`; `test result: ok. 36 passed` (34 of 25c + 2 new); wall 1.3–1.6 s, inside §8.5's 60 s |
| E-2 | AC-25d-2: `equals corner-shop docs/guide/examples/corner-shop` after step 5.7 | PASS | part of E-1; the example was extracted from the chapters' `file=` blocks in page order, and the runner compares it byte for byte |
| E-3 | Mutation: the index exclusion removed (`INDEX` second entry renamed) | PASS (bites) | `the_index_and_its_mirror_are_neither_run_nor_held_to_page_parity ... FAILED`, `README.zh-CN.md has no English page README.md`; restored |
| E-3b | Mirror links are checked (new) | PASS (bites) | before the zh-CN chapter 13 existed, `03-two-rooms.zh-CN.md: 13-when-something-is-refused.zh-CN.md#step-134--人在一个不存在的地方: … has no heading with that anchor`; `a_broken_link_in_a_mirror_turns_the_run_red_and_names_the_mirror` passes |
| E-4 | Doc checks | PASS | `check_doc_headings.py`: `238 numbered sections across 63 documents, none duplicated`; `check_decision_ids.py`: `127 decision ids, all distinct` |
| E-5 | Static checks and the touched crate's targeted tests | PASS | `cargo fmt --all -- --check` clean; `cargo clippy -p mineworld-cli --all-targets -- -D warnings` clean; `cargo test -p mineworld-cli --test create` 2 passed (template comments changed); `--test packs` 6 passed |
| E-6 | Chapter 06's operator-only steps, checked once by this session | PASS (macOS, headless) | `mineworld server corner-shop` (and `--town`) printed the lines step 6.2 quotes; `./mineworld-2d --server 127.0.0.1:7878 --invite … --seat ben --drive` against the example world: `welcomed into a world`, Ben and Cai drawn and labelled, one doorway, `drive complete: PASS`. Not a substitute for AC-25d-4 |
| E-7 | `scripts/ci_changes.py --self-test` with the guide classified as executed | PASS | `[self-test] passed: 24 case(s)`, incl. `docs/guide/02-your-first-world.md: code=true` and `docs/guide/examples/corner-shop/world.yaml: code=true` |
| E-8a | First CI run, PR #240 head `627efe9b` (run 38813791073) | FAIL (Windows), PASS (Linux, macOS) | `test` and `test-macos` pass. `test-windows`: `guide: 104 block(s) passed, 1 pending`, one failure — step 1.2's `Usage: mineworld <COMMAND>` prints `mineworld.exe` on Windows. Fixed by quoting `...` for that line (and a sentence in step 1.2 explaining `...` and `[..]` to readers). `python (windows-2025)` failed in `cognition/lm-controller` (`test_ac10.py`, `test_retention_real.py`: `SocketConnectBlockedError`), which this PR does not touch |
| E-8 | AC-25d-1 on the three OSes: CI on head `d40b177b` (run 39003657480) | PASS (guide, all three OSes); job `test-windows` INCONCLUSIVE | `test` (Linux) and `test-macos` pass whole. `test-windows`: `the_guide_in_this_repository_runs_green ... ok` and the fixture page ok at 05:45; the job was then cancelled at its 60-minute limit (06:19, `The operation was canceled`) in later suites — the known Windows timeout 25a also has, not a guide failure. `python (windows-2025)` fails in `cognition/lm-controller` as in E-8a, untouched by this PR |
| E-9 | Review read-through (English plain-language rule 7, rule 8, AC-25d-5) | PASS | no decision id, plan name or PR in any page body (`grep` over `docs/guide/*.md`); no prose sentence above 27 words (a paragraph-mode `awk` count over every English page); every field, bound or default the guide mentions links its reference entry (`world-pack.md`, `sections.md`, `configure.md`, `interaction-list.md`, `pack-manifest.md`, `cli.md`); every defined term is explained at first use on its page and linked to `CORE_CONCEPTS.md` |
| E-10 | AC-25d-3: every `docs/guide/*.zh-CN.md` and `README.zh-CN.md` revised to the operator's plain-Chinese rule (ruling 2026-10-15: `clear-tech-explainer`, `plain-chinese` with `references/common-technical-jargon.md` and `terminology-policy.md`, `concrete-example`; no dongbei-voice) | PASS | **What changed.** MineWorld's own concepts keep their English names, the ones the command output prints (`World Pack`, `System Pack`, `seat`, `genesis fact`, `passage`, `routine`, `save`, `biography`, `class`, `Interaction List`, `Entity Pack`). Each is introduced on a page by saying in Chinese what it does. This replaces invented Chinese names (世界包, 系统包, 座位, 初始事实, 交互清单, 实体包), which now occur 0 times. Tier-1 terms (`server`, `client`, `build`, `YAML`) are used as they are. The 35 repeated `（英文）` tags are gone; one line in the index says the reference and the specifications are English-only. Sentences are short, with an actor and a verb. One worked trace was added: Cai's day by the clock in step 4.2. **Checks.** `python3 -I scripts/lint_text` from the DongbeiGPT clone passes on all 11 files (no `——`, no excluded expressions). The guide runner passes: `105 block(s) passed, 1 pending`; every mirror's parity and links PASS; 36 tests ok. Steps, commands and output blocks are byte-identical to the English pages, as the runner checks. **Before → after:** (1) step 02 intro: 「一个世界就是一个**世界包**（World Pack）：一个文件夹，里面是纯文本文件，格式叫 YAML。」 → 「一个世界就是一个 **World Pack**：一个文件夹，里面放几份纯文本文件，格式是 YAML。」 (the project term keeps its English name, with no invented Chinese name added). (2) step 6.2: 「启动一个**服务器**（server）：让世界一直运行、让玩家加入的程序。」 → 「启动一个 server。server 是一个一直开着的程序：世界在里面不停地跑，玩家连上它就能加入。」 (a Tier-1 term is kept; its explanation is its own short sentence, not a parenthetical). (3) step 4.2: 「日程只是打算，不是瞬移：Cai 还得自己走过去。」 → 「拿 Cai 的一天走一遍：00:00 在家，16:00 想去小店，18:00 想回家，第二天 16:00 再去小店。routine 只是打算，不会把人直接挪过去，Cai 还得自己走过去。」 (concrete-example: one item traced through the real mechanism, and the compressed 不是瞬移 is replaced by an ordinary clause) |

### 18.2 Deviations (bounded, recorded)

- **D-25d-1 Chapter 01 runs a `create`d world, not `market-town`.** Reason: the runner's scratch folder has no
  `worlds/`, and no release bundle exists yet (step-23 R-d). Impact: chapter 01 shows `create hello` and a two-day
  run of it; the sample towns are not mentioned as a command. Validation: E-1.
- **D-25d-2 Chapter 02 writes Corner Shop by hand instead of `mineworld create corner-shop`.** Previous assumption:
  §6.3 row 02 starts from `create`. Audit evidence: the template's people are `first` and `second`; the loader
  refuses a person file `world.yaml` does not list (`… describes a person that world.yaml does not list; add
  'zoe' to population, or remove the file`), and the runner has no delete, so the template's files could never
  be replaced by `ana` and `ben` in an executed page. Corrected: chapter 01 uses `create`; chapter 02 builds from
  an empty folder, one file per step, each `validate` naming the next missing file. Validation: E-1, E-2.
- **D-25d-3 No person radius or doorway width in chapter 03.** §6.1 asks for a 250 mm radius and a 900 mm doorway;
  no authored field takes either (`docs/reference/world-pack.md`: a passage has `to`, `here`, `there`). Stating
  them would document behaviour that does not exist. The page states positions in millimetres only.
- **D-25d-4 Chapter 06: `--town`, and the 3D client marked "coming".** Without `--town` the server drives no seat
  (`0 seat(s) driven in-server`), so Ana and Cai would stand still. The 3D slice (`mineworld-slice`) draws the
  sample town's café only, so playing your own world in 3D is marked as coming. `PLAY.txt` does not exist yet.
- **D-25d-5 Chapters committed together.** Chapters link forward (03 → 13, 05 → 06, 08 → 12), and the runner
  checks every link, so one chapter per commit would leave red intermediate heads. The guide is one commit; the
  runner corrections, the CI classification, and the links/template are separate commits.
- **D-25d-6 Chapter 12 makes its own Entity Pack.** `modern-goods` lives at `entities/modern-goods` in the
  repository and is not reachable from the runner's scratch folder (no directive copies from outside it). The
  chapter builds `packs/corner-goods` (one item, `jam`), requires it, and resolves it with `--packs packs`, all
  executed; `modern-goods` is described in prose, with no command.
- **D-25d-7 Chapter 08 shows the weather through `perceived --json`.** §6.3 says "day lines show weather"; `run`
  prints no weather on its day lines. The executed evidence is the run summary's `day-began`, `daylight-changed`,
  `weather-*` counts and one `day-began` and one `weather-changed` line of `perceived --json`.
- **Runner corrections (25c code, bounded, within INV-S25-3):** (a) `README.zh-CN.md`, the index's mirror, was
  treated as a mirror with no English page and failed the run; it is now excluded like `README.md`
  (`run_guide.rs` `INDEX`; `GUIDE_FORMAT.md` §1, §1.1 rule 1), test
  `the_index_and_its_mirror_are_neither_run_nor_held_to_page_parity`. (b) `GUIDE_FORMAT.md` §7 rule 4 requires
  a mirror's links to resolve, but only the English page's links were checked; the mirror's are now checked and
  reported under `links of <page>` with the mirror's name, test
  `a_broken_link_in_a_mirror_turns_the_run_red_and_names_the_mirror`. (c) `scripts/ci_changes.py` classified
  `docs/guide/**` as documentation, so a guide-only PR would have skipped the build and the guide test; it is now
  classified like `docs/reference/**` (two self-test cases).

### 18.3 Open items and findings for the primary

- **F-25d-1 Stale feature flags.** `docs/guide/features.toml` says `decisions`, `motivated`, `relationship-decay` and
  `lm-controller` are `false`, but this build has `mineworld decisions`, the bundled `mineworld-motivated`
  controller pack, `familiarity_decay`/`regard_decay` in `configure/relationships.yaml`, and P6b is merged on
  `main`. Those flags have no presence probe, so the runner reports them `INCONCLUSIVE`. Flipping them is 25e's
  (AC-25e-3); not changed here. Chapters 09–11 are listed in the index as coming.
- **F-25d-2 Spec headings are link targets.** Guide pages link `CORE_CONCEPTS.md` and `MODULE_SPEC.md` headings.
  A docs-only PR that renames one does not run the build (ARC-48 note), so the broken link turns red only at the
  next code or guide PR. Not changed here (it would widen the code set to all of `docs/`); recorded.
- **AC-25d-3** done (E-10). The coordinator asked for it on 2026-10-16. The style rule is the operator's ruling of
  2026-10-15: the non-dialect skills of `yuema137/DongbeiGPT`. The dongbei-voice layer was not used.
- **AC-25d-4** The operator follows chapters 01–05 by hand at the milestone handoff (QS25-10).
- **E-8** CI on the exact head.

### 18.4 Restack on 25a @ 7ad96dd (2026-10-17, S25 stack owner)

- **Merge.** `origin/mvp0/pr-s25a @ 7ad96dd` (25a with `main @ e51ffc0`: S10 P6a/P7, S30-b, S27-b, S11e) merged into
  this branch. Textual conflicts in `README.md` and `README.zh-CN.md` only: the guide's link paragraph is kept, then
  main's paragraph on `clients/agent` and the cognition reference, then main's "Which computers" wording.
- **The previous CI failures were cancellations, not defects.** Run 39010220065 on `07dbab1e`: every red job is
  `cancelled` (superseded runs under the old, unsharded workflow); none ran to a failure.
- **E-11 Guide test on the merged tree, on GitHub CI** (local heavy jobs paused by the operator, 2026-10-17). Scratch
  branch `scratch/s25-guide-25d` (the merge commit plus a temporary workflow running `cargo test -p mineworld-cli
  --test guide`), run on Linux: **FAIL**, one block: step 12.3's `packs validate` output. S30-b let an Entity Pack
  carry data directories, so the refusal now ends "…, one file per item kind, or a .yaml file in a data directory an
  installed System Pack owns". Classified **stale page output** (real binary changed on main), not a flake; the
  page and its mirror quote the new line (byte-identical, as the runner requires). Chapter 13's row quotes the
  unchanged prefix and stays as is. Every other block passed (`104 block(s) passed, 1 pending`). The re-check and
  the PR's CI are recorded below.

---

## 19. Evidence ledger — PR 25e (`mvp0/pr-25e`)

Recorded by the implementing session (Opus-class, dispatched by the primary, 2026-10-16). Worktree
`/Users/yuema137/mineworld-worktrees/impl-25e`, branch `mvp0/pr-25e`, stacked on `origin/mvp0/pr-25d @ 07dbab1e` (25d's
plain-Chinese pass). Every quoted output is this stack's binary. Results are `PASS`, `FAIL` or `INCONCLUSIVE` from
actual output; local runs are targeted (R-LOAD-1), CI on the exact head is the gate.

### 19.1 What the stack has (audited, not assumed)

| Feature | On this stack | Evidence |
| --- | --- | --- |
| decision list (DL-a) | yes | `mineworld decisions`; `docs/reference/decisions.md` |
| `motivated` (DL-b) | yes | `cognition/installed` `models!` lists `Motivated`; `mineworld-motivated` in `packs list` |
| relationship decay (FG-a) | yes | `familiarity_decay`, `regard_decay` in `docs/reference/interaction-list.md` |
| memory policies (FG-b, FG-c) | merged, but not observable by the guide | a policy acts inside a cognition process that hosts a seat; `run` hosts none, and the `memory:` choice is only decoded (`MODULE_SPEC.md` §5.2) |
| language-model controller (P6) | no | `external "lm"` is in `cognition/installed` on `origin/main` (P6b, #207) but not on this stack; `--seat-model first=lm` is refused with `it installs: rule, motivated` |
| happenings (WE-a) | no | not merged (`origin/mvp0/pr-we-a`) |

### 19.2 Evidence

| Id | Item | Result | Evidence |
| --- | --- | --- | --- |
| E-1 | The whole guide against the real binary (`cargo test -p mineworld-cli --test guide`) | PASS (macOS, local) | `guide: 151 block(s) passed, 1 pending, 39 check(s)`; the pending block is step 6.2 (`operator-only`); `INCONCLUSIVE feature memory-policies` (no probe, §9.1); every page's and mirror's links and parity `PASS`, compare pages included; `test result: ok. 39 passed` (36 + 3 new); 6.4 s wall, inside §8.5's 60 s |
| E-2 | New runner tests | PASS | `comparison_pages_run_after_the_chapters_and_are_held_to_their_mirrors`, `example_copies_a_committed_folder_into_the_scratch_folder_and_nothing_from_outside`, `model_and_reference_probes_turn_a_wrong_flag_red` |
| E-3 | Mutation: the four flipped or held flags set wrong (`decisions`, `motivated`, `relationship-decay` false; `lm-controller` true) | PASS (bites) | the real guide run reports `FAIL feature decisions`, `FAIL feature motivated`, `FAIL feature relationship-decay`, `FAIL feature lm-controller`; restored |
| E-4 | F-25d-2: `python3 scripts/ci_changes.py --self-test` | PASS | `[self-test] passed: 26 case(s)`, incl. `docs/CORE_CONCEPTS.md: code=true` and `docs/MODULE_SPEC.md: code=true`; the docs-only cases (`docs/DECISIONS.md`, `docs/MVP.md`, …) stay `code=false` |
| E-5 | Static and doc checks | PASS | `cargo fmt --all -- --check` clean; `cargo clippy -p mineworld-cli --all-targets -- -D warnings` clean; `check_doc_headings.py`: `238 numbered sections across 75 documents, none duplicated`; `check_decision_ids.py`: `127 decision ids, all distinct` |
| E-6 | Review read-through (rules 7, 8; one change per comparison; whole files) | PASS | each comparison changes one file after its setup steps, every replaced file is shown whole; each field the pages name links its reference entry (`decisions.md`, `world-pack.md`, `interaction-list.md`, `configure.md`); no decision id or plan name in a page body; the zh-CN mirrors follow 25d's term table (English concept names, no invented Chinese names, no `——`), and their code blocks are generated from the English pages' blocks, so parity holds byte for byte |
| E-7 | CI on the exact head | pending | recorded after the push |

### 19.3 F-S25-6 audit: where each difference is visible (C0)

| Comparison | Before | After | Read-only command |
| --- | --- | --- | --- |
| a forbidden action (nobody talks in the shop) | `talk accepted 335`; Ana 108 | `talk accepted 292`; Ana 81 | `run` summary; `interactions --place shop` shows the rule |
| weather (start 2026-10-08 vs 2027-01-08, San Diego climate) | `weather-changed 17` | `weather-changed 24`; `activity` identical | `run` summary |
| relationship decay (regard half-life 3 days, 30 days) | `relationship-changed 20` | `relationship-changed 38`; `activity` identical | `run` summary; `perceived --json` shows `"from":"close"` … `"to":"friend"` |
| chapter 09 (`motivated` Ben) | Ben move 159 talk 118 | Ben move 9 talk 18 | `run` header `(2 rule, 1 motivated)`, `activity`; `decisions` |

### 19.4 Deviations (bounded, recorded)

- **D-25e-1 One PR, not one per feature.** §10.3 splits 25e by feature. The features of 25e-1, 25e-2 part 1 and 25e-4
  are all on this stack, and the comparison index links each page, so the coordinator dispatched one PR. 25e-3
  (happenings), 25e-5 (memory policies) and 25e-6 (P6) remain, listed as "coming" in both indexes.
- **D-25e-2 The runner walks `compare/` and gains `example`.** `GUIDE_FORMAT.md` §1.1 left the `compare/` order to
  25e. Comparison pages run after the chapters, named `compare/<stem>`. They start from the committed example, which
  `copy` cannot reach (it is confined to the scratch folder) and which `continues` cannot give (the chapters 07, 08 and
  12 change the shared folder, and the loader refuses an unlisted `configure/` file, so a later page cannot undo them).
  `example PATH DEST` copies a committed folder, confined like every other path. The world's folder must keep its id
  as its name, so pages copy into `compare-<name>/corner-shop`.
- **D-25e-3 The presence probe asks more than `packs list`.** `decisions` and `relationship-decay` are not packs, and
  `motivated`/`lm` are best seen as installed decision models. The probe now also reads the generated reference and the
  installed-model list (`GUIDE_FORMAT.md` §9.1). `memory-policies` keeps no probe (§19.1), so it stays
  `INCONCLUSIVE` and `false`; chapter 10 part 2 says it is coming.
- **D-25e-4 Chapters 09 and 10 work on copies.** Both continue chapter 08 and copy `corner-shop` into `who-decides/`
  and `forgetting/`, because chapter 12 continues 08 and expects the folder unchanged.
- **D-25e-5 The decay comparison adds `group-activity` first.** Corner Shop's people meet every day and have no
  source of regard (only group activities add regard), so decay alone changes no output (`relationship-changed` stays
  at 0, familiarity values are printed nowhere). With `group-activity` as a setup step the difference is visible
  (§19.3). It shows a friendship cooling between meetings, not acquaintances who never meet again (§6.3's example):
  a pair that stops meeting would need a second change to the world.
- **D-25e-6 The weather comparison changes the season, not the climate.** No second fitted climate is committed (only
  San Diego's NOAA fit), and inventing one would document numbers with no source. `biography` reports no time
  outdoors (§7's "what is compared"), and the `rule` model ignores the weather, so behaviour is identical. The page
  compares October with January, one line of `calendar.yaml`, and says plainly that people do the same.
- **D-25e-7 Chapter 08's "Next" link and two index rows changed.** The only edits to 25d's files: links to the new
  pages, in English and in the mirror.

### 19.5 Open items and findings for the primary

- **F-25e-1 `lm-controller` will turn red when this stack takes `main`.** P6b's `external "lm"` is on `main`. Once the
  stack merges it, the probe sees `lm` and the flag must flip to `true` with chapter 09 part 3 and
  `compare/one-person-on-a-model` (25e-6) in the same change. That is the probe working as designed.
- **F-25e-2 Memory policies cannot be shown by the guide yet.** Showing one needs a seat hosted by a cognition
  process (P6). Until then the flag stays `false` with no probe; a probe should be added with 25e-5.
- **F-25e-3 Familiarity is printed by no read-only command.** Only level crossings are (`relationship-changed`), so a
  world without group activities cannot see familiarity decay at all. A small read-only report would be its own PR
  (not designed here, INV-S25-5).
- **AC-25e** Chinese plain-language sign-off of the new mirrors: the primary's.

### 19.6 Restack on 25d (with 25a and `main @ e51ffc0`), 2026-10-17, S25 stack owner

- **Merge.** `mvp0/pr-25d` merged in, no textual conflict. The earlier red checks of run 39011129193 (head
  `65000c39`) were all `cancelled` (superseded), not failures.
- **F-25e-1 came true, as predicted.** The stack now has P6b's decision model `lm` (`cognition/installed`), so the
  presence probe (`GUIDE_FORMAT.md` §9 rule 3) reports `lm-controller = false, but decision model lm is in the
  build`. Decision (bounded, within §9): the flag is set to `true`, which §9 rule 4 defines as "the feature is
  merged and its blocks must pass"; no block `requires lm-controller` yet, so no page changes. Chapter 09 part 3 and
  `compare/one-person-on-a-model` (25e-6) stay "coming in a later guide update", as both indexes and step 9's last
  paragraph already say; they need a page that shows a model's decision without a live model, which is 25e-6's
  design, not this restack's. `GUIDE_FORMAT.md` §9's example and §9.1's row are updated to say the model is
  installed.
- **The probe test keeps both directions.** `model_and_reference_probes_turn_a_wrong_flag_red` asserted
  `lm-controller = true, but decision model lm is not in the build`, which this build can no longer produce. It now
  plants `lm-controller = false` (expects `… decision model lm is in the build`) and `happenings = true` (expects
  `happenings = true, but pack mineworld-happenings is not in the build`; WE-a is not merged), so a false flag and a
  true flag are each still shown to turn the run red.
- **The decay comparison's "after" output changed on the restacked build** (scratch guide run 39102856582 on the
  25f head, Linux: `FAIL page compare/relationship-decay line 184 step 1.4`; every other block passed). The
  "before" run is unchanged (`relationship-changed 20`, Ana move 1468 talk 738); the "after" run now gives
  `relationship-changed 28` (was 38) and a slightly different `activity` line (Ana move 1453 talk 732, Ben 1431/913,
  Cai 1544/845). Main's group-activity changes (S27-b: offers complete, an expired invitation not offered) are in
  the restack; the page no longer claims identical behaviour. Classified **stale page output**, not a flake. The
  page and its mirror quote the new numbers; the table's `activity` row and the closing paragraph say people's days
  change a little and the ties change a lot. §19.3's row for this comparison is superseded by these numbers.
- Validation: on GitHub CI only (operator pause on heavy local jobs); recorded with the PR's run.

---

## 20. Evidence ledger — PR 25f (`mvp0/pr-25f`)

Recorded by the implementing session (Opus-class, dispatched by the primary, 2026-10-16/17). Worktree
`/Users/yuema137/mineworld-worktrees/impl-25f`, branch `mvp0/pr-25f`, stacked on `origin/mvp0/pr-25e @ 65000c39`. The
stack has DL-a, DL-b and FG-a; it has neither WE-a (happenings) nor P6 (`lm`) (§19.1).

### 20.1 The agent CLI, verified (C0)

Claude Code 2.1.302, `claude --help` read 2026-10-16. Used: `-p`, `--model`, `--safe-mode`, `--strict-mcp-config`,
`--setting-sources ""`, `--tools`, `--disallowedTools`, `--permission-mode bypassPermissions`,
`--no-session-persistence`, `--output-format stream-json --verbose`, `--max-budget-usd` (listed) and `--max-turns`
(accepted, not listed; a probe run with it completed, `num_turns: 1`). A probe agent started this way reported no
CLAUDE.md, memory or MCP content in its context. Authentication is the operator's own login; `run.py` removes every
`*_API_KEY` variable, so no key is used. `--bare` was rejected: it reads only `ANTHROPIC_API_KEY`, which would mean a
paid key.

### 20.2 Evidence

| Id | Item | Result | Evidence |
| --- | --- | --- | --- |
| E-1 | `uv run --locked pytest tools/onboarding` | PASS (macOS, local) | `26 passed` at the final head (22 before the ruling; added: S8, the guard, the allowlist rules, a denied attempt as blocked): scorer on the ON-1 fixture and per-check mutations (S1, S2, S3 ×2, S4, S6); auditor void by absolute path, `..`, symbolic link, `~`, `cd ..`, `Grep /Users`, `WebFetch`; repeated refusal, gap, guess, stopped, status, unknown field; linked-folder spelling inside; assemble's exact allowed set, no mirror, no leak, refusal inside a repository; run.py refusals (no budget, budget above 5, no agent command, turns above 80, `CI` set); no API key in the agent's environment; the scripted dry run end to end |
| E-2 | `-m "not real_binary"` (the `python-smoke` layer) | PASS | `10 passed, 10 deselected` (before the last two audit tests; same split) |
| E-3 | The guide test after the round-1 and ON-3 fixes | PASS | `guide: 161 block(s) passed, 1 pending, 39 check(s)` |
| E-4 | Static checks | PASS | `ruff check`, `ruff format --check`, `pyright` strict over `sdk/python cognition/lm-controller tools/onboarding`: `0 errors`; `check_doc_headings.py`, `check_decision_ids.py` clean |
| E-5 | CI on the exact head | pending | after the push |

### 20.3 The rounds ledger

Instruction set ON-1 in every round. The sandbox is a fresh `tempfile.mkdtemp` folder; evidence is kept outside the
repository (the session's scratchpad, `onboarding/round-*`). Sonnet-class gates; Haiku-class is the reported "harder
reader".

| Round | Commit | Agent, model | S1 | S2 | S3 | S4 | S5 | S6 | S7 | Void | Stuck points (class) | Turns | Cost (USD, list price; the login is a subscription) | Load before → after (1/5/15 min) | Transcript SHA-256 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `cfde1b31` | claude-h2, sonnet (claude-sonnet-5) | PASS | PASS | PASS | PASS | FAIL gap | FAIL gap | SKIPPED | no (after the audit fix, `70dd72e8`) | STUCK status; gaps: weekend market, weekday in a world, a seat on a local model, attaching cognition.toml (all **real product gap**: WE-a, P6) | 9 | 0.205 | 10.98/7.66/7.87 → 6.41/6.93/7.59 | `476574a6…2e5c63` |
| 1 | `cfde1b31` | claude-h2, haiku | PASS | PASS | PASS | PASS | FAIL gap | FAIL gap | SKIPPED | no | STUCK; gaps market, local model (**product gap**); **not listed by the audit:** its first world had no passage, ran with `faults 0` while 1 694 walks were rejected `no-route`, and chapter 13 said a mistake never runs silently (**misleading statement + missing step**, fixed `47a4a366`) | 28 | 0.023 | 3.89/6.19/7.28 → 3.51/5.49/6.91 | `783f91bf…6aedc` |
| 2 | `47a4a366` | claude-h2, sonnet | PASS | PASS | PASS | PASS | FAIL gap | FAIL gap | SKIPPED | no | gaps: market, a seat on a local model, weekday routines (**product gap**); wrote DONE | 10 | 0.194 | 8.50/6.02/6.87 → 6.18/5.69/6.72 | `b243079f…cb4742` |
| 2 | `47a4a366` | claude-h2, haiku | PASS | PASS | PASS | PASS | FAIL gap | FAIL gap | SKIPPED | no | STUCK; gaps market, local model (**product gap**) | 30 | 0.027 | 6.24/5.71/6.72 → 4.33/5.22/6.45 | `c29693d8…b6252c` |

In every round each world ran 14 days with no `rejected` line (checked by hand after round 2). Round 2 found no guide
defect: every stuck point is a product gap, which stops the round (§9.6). Rounds stop here: ON-1 cannot pass S5 or S6
until WE-a and P6 are on the stack, so a third round would measure the same gaps.

### 20.4 Deviations (bounded, recorded)

- **D-25f-1 C0–C4 in one commit.** The protocol, the four tools and their tests refer to each other; split, the
  intermediate heads would carry untested tools.
- **D-25f-2 No Containerfile; H2 replaced by the allowlist.** H1 needs the agent's login inside a container, which
  means copying a credential (forbidden) or a paid key (not authorized). Rounds 1 and 2 ran H2 (bypass mode plus the
  audit); the operator withdrew it on 2026-10-17, and every later run uses the allowlist and the guard (§20.5).
- **D-25f-3 AC-25f-1 cannot show S5 or S6 passing.** No world can pass them on this build. The scorer fails them as
  `product gap:`, the tests assert that, and the S6 provider scan is shown to bite on its own. The positive fixtures for
  S5 and S6 are added when WE-a and P6 are on the stack.
- **D-25f-4 S1 and S2 cannot fail alone.** S2–S4 read `validate`'s output, so an invalid world fails them too, and a
  world with no seat (the only valid world that does not run) fails S4 as well. Every other check bites alone.
- **D-25f-5 The tools are standard-library scripts, not a workspace member.** No dependency and no `uv.lock` change;
  `scripts/ci_layer.py` adds them to the static checks and runs their tests (`python`; `python-smoke` deselects
  `real_binary`). Python 3.12 is required (`tomllib`); the system `python3` on macOS is 3.9, so the operator runs
  `uv run python tools/onboarding/run.py …`.
- **D-25f-6 ON-2 not run; ON-3 run once** on the operator's authorization of 2026-10-17 (§20.6).

### 20.5 Operator ruling of 2026-10-17: enforcement before the act

Ruling, verbatim choice: "改成容器或白名单再跑" (switch to a container or a whitelist, then run again). Rounds 1 and 2
ran with `--permission-mode bypassPermissions` and an after-the-fact audit; that is withdrawn.

- **Chosen: the allowlist** (`claude-allowlist`, `tools/onboarding/PROTOCOL.md` §§3.2–3.4), commit `a01aee34`.
  Claude Code 2.1.302 in `dontAsk` mode with `--settings` (kept outside the sandbox): `Bash(mineworld)`,
  `Bash(mineworld *)`, `Read` under the sandbox, `Write`/`Edit` under `work/` only; everything else is denied before it
  runs. The sandbox's `mineworld` is a guard (`guard.py`) that refuses a working directory or a path argument outside
  the sandbox before the real binary starts; the real binary is where the allowlist cannot name it. The CLI also
  allows read-only commands such as `ls` and `cd` inside the allowed folders on its own (seen in round 3), and denies
  them outside (the escape test).
- **Not chosen: a container.** The operator's login cannot enter it without copying a credential, which is forbidden,
  and a paid key is not authorized. **Not chosen: Claude Code's Bash sandbox.** Not verifiable from this CLI's `--help`;
  the allowlist plus the guard is verified by test.
- **The planted escape test** (`escape_test.py`, run `escape-2`, 2026-10-17, sonnet, load 7.09 → 6.37): a canary beside
  the sandbox; 15 escape attempts, 4 controls. **PASS**: 13 attempts denied by the CLI before running (`Read`, `cat`,
  `Glob`, `Grep` of the canary; `ls ~/.ssh`; `Read ~/.ssh/config`; `Write` and `echo >` outside; `mineworld … && cat`;
  a redirection; a command substitution; the real binary by path; `cd` outside), 2 refused by the guard with exit
  126 (`mineworld create` outside, `mineworld create ../../…`); no file written outside; the canary in no tool
  result; all 4 controls done. Transcript SHA-256 `f1ddc22c…3d8831`. (Run `escape-1` was inconclusive, not failed:
  the agent stopped after four denials; the prompt now says a denial is expected, and the controls come first.)
- **The audit stays as the second layer** (`666a8e1e`): a denied call is listed as `blocked`, not void, and every
  shell command is resolved from `work/`.
- **S8 added** (the proposed check): a run that exits 0 but prints a `requests … rejected` line fails S8.

### 20.6 The ON-3 round (authorized once, 2026-10-17)

| Round | Commit | Agent, model | S1 | S2 | S8 | S3 | S4 | P1 | Void | Blocked | Stuck points (class) | Turns | Cost | Load before → after | Transcript SHA-256 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 3 | `a01aee34` | claude-allowlist, sonnet | PASS | PASS | PASS | PASS | PASS | PASS | no (after `666a8e1e`) | 3 commands denied before running (a `cd ../guide; cat …`, an `echo; which; ls`, a `mkdir … && cat > … <<E`); the agent switched to `Read` and `Write` | (1) where `modern-goods` is: the guide gave only "the source's `entities` folder", the sandbox had it at `packs/` (**missing step**); (2) the guess "the guide doesn't say", on the same point; (3) whether a world lists a pack's goods under `items` (**unclear step**); (4) no book among `modern-goods` (**real product gap**, content) | 34 | 0.256 | 5.54/14.87/10.35 → 5.52/13.10/9.99 | `4178b0e9…f3663455a` |

The agent built `town`: two streets (High Street, Mill Lane) joined by a door, a tea shop and a bookshop, cottages,
six resident seats, two of them employed, every item from `modern-goods` through `requires` and `--packs`; it validates
and runs 14 days with no fault and no rejected request, and it wrote DONE. Points (1)–(3) are fixed by step 12.7
(`21a239b2`; guide test green) and by the sandbox carrying the pack at `entities/modern-goods`, the path chapter 12
names. ON-2 stays unrun (a `shout` action and a time-of-day rule do not exist). No further round was run: the
ruling authorized one.

### 20.7 Findings for the primary

- **F-25f-1 `validate` accepts a place no door leads to** (round 1, Haiku). A finding for the loader: S29-a adds
  reachability for towns. Until then the guide teaches how to see it (steps 13.16–13.19) and S8 catches it in scoring.
- **F-25f-2 AC-25f-5 waits for WE-a and P6** (F-S25-5, QS25-9). Every ON-1 run built the village, bakery and three
  resident seats from the guide and the reference alone; S5 and S6 are product gaps.
- **F-25f-3 Agents read `features.toml`** to learn what is missing. It is runner metadata in `docs/guide/`, hence in
  the allowed set; it helped them stop rather than invent. Whether it should stay visible to readers is the primary's
  call.
- **F-25f-4 The guard is POSIX-first.** It runs on the system `python3`; the harness is operator-run on macOS. Its tests
  run in CI on all three OSes; a Windows operator run is untested.

### 20.8 Restack on 25e (with 25d, 25a and `main @ e51ffc0`), 2026-10-17, S25 stack owner

- **CI on `eb2e31df` (run 39054546515), classified.** Real defects, both fixed in `77834abb`: (1) `pyright` strict on
  `windows-2025` refused `os.getloadavg` (absent on Windows) and an untyped empty `list` in `tools/onboarding/run.py`;
  `load_average()` narrows on `sys.platform`, and the list is annotated (`pyright --pythonplatform Windows
  tools/onboarding`: 0 errors, locally, light Python). (2) `test_structural_isolation.py::test_ci_runs_pytest_once_per_member`
  counted exactly two pytest lines per Python layer; 25f adds `tools/onboarding` as a third. `test-windows` was
  `cancelled` under the old unsharded workflow, not a failure. None was a known Windows flake.
- **Merge conflicts.** `scripts/ci_layer.py`: main's `clients/agent` member (S27-c) and Milestone D's `milestone_d`
  line (S10 P7) kept, with 25f's `PYTHON_TOOLS` and its two pytest lines after them. `pyproject.toml`: pyright's
  `include` names all four trees. `test_structural_isolation.py`: main's generalised check (S27-c D-c-22: every
  pytest line unique, each member exactly once, any other path must exist) supersedes the count fix of `77834abb`
  and admits `tools/onboarding` as is. The step-25 ledger: §19.6 and §20 both kept.
- **The operator's ruling (§20.5) holds on this head**: `--permission-mode bypassPermissions` appears only in this
  ledger's history and in `PROTOCOL.md`'s statement that it is not used; the runner builds the allowlist command.
- Validation: CI on the exact head (the operator paused heavy local jobs).

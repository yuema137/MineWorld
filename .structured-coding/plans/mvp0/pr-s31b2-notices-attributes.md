# PR S31-b2 — `notices` adopts S30's `AttributeOwner`; the `topics` record

## DESIGN FROZEN 2026-10-26 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-26 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-20), frozen 2026-10-26; scope extension 2026-10-27
Author:                 S31-b2 PR-design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/impl-notices-attr, branch
                        docs/pr-design-notices-attributes, planning base origin/main @ fec906f2
                        (S31-b #349 and S30-c merged; S30-f merged: employment is an owner)
Approved by / evidence: operator rulings recorded in the freeze header and §14.1
Implementation base:    original C0 0b2faf2f; recovery 2b4d36fa (2026-11-22)
Start condition:        this design frozen by the operator; the coordinator's dispatch
Execution contract:     §12
Implementer:            Opus-class for C1 (a loader seam); Haiku may take C3 (the topics record)
Lifecycle:              PUBLISHED 2026-11-27 (branch mvp0/pr-s31b2-notices); red-team PASS-WITH-FOLLOWUPS;
                        awaiting CI on the exact head and operator merge review
```

**Effort:** `mvp0` · **Step:** [`step-31-wealth-prices.md`](step-31-wealth-prices.md) §8.5 ("Order");
[`step-30-person-attributes.md`](step-30-person-attributes.md) §9.6 (N-1 … N-6) · **Parent PRs:**
[`pr-s31b-notices.md`](pr-s31b-notices.md) (merged as #349; its R-S30c-b-1, §§2.2, 5.4, 10),
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (merged; R-S31-c-1, Q-S30c-2) ·
**Also carries:** R-S31b-g-2 from [`pr-s31g-agent-view.md`](pr-s31g-agent-view.md) §5.14 (Q-S31g-3 ruled yes,
2026-10-20) · **Must merge before:** S31-e and S31-g · **Coordinates with:**
[`pr-s31f-market-town.md`](pr-s31f-market-town.md) (frozen 2026-10-20; Market Town uses `defaults.by_tag` and does
not enable `attributes`) · **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 2, 5, 7, 8, 9, 11, 12, 16);
`docs/MODULE_SPEC.md` §4.1 ("Vocabularies", "Owns"); `docs/decisions/ARC-115.md`, `ARC-120.md`,
`ARC-121.md`, `ARC-124.md`.

**Decision ids.** No new id. This PR writes dated notes under `ARC-124` (the adoption as realised, and the
configured-vocabulary seam) and `ARC-115` (the `topics` record). Run `scripts/check_decision_ids.py`.

No hosted model is called by this design or by anything it specifies.

---

## 0. Binding inputs

- **`ARC-124` note (2026-10-18, S31-b).** "The PR that follows S30-c's merge makes `notices` an `AttributeOwner`
  for `notices.subscriptions`. That PR ships the declaration with `owner: notices`, receives each person's
  resolved set through `attributes-handed`, states the same `subscriptions-seeded` fact `notices` states today,
  drops `defaults.by_tag` in favour of S30's `roles`, and discloses through `disclose_owned`. No fact changes
  shape; a change that would is a material stop for both packs."
- **R-S30c-b-1** (`pr-s31b-notices.md:728-733`). The same, plus: "In a world without `attributes`, the section
  path remains (Q-S30c-2: the attribute is not shown there, the state still exists)."
- **Q-S30c-2 (ruled 2026-10-18).** "An owner's attributes exist only where `attributes` is enabled."
- **R-S31-c-1** (`pr-s30c-attributes-pack.md:744-746`). Owners "ship their declarations with `owner:`
  themselves, receive initial values through `attributes-handed`, disclose with `disclose_owned`".
- **Step-30 §9.6 N-2.** `notices.subscriptions`: group `behaviour`, `kind: { set: { vocabulary: notices.topics,
  max: 32 } }`, `visibility: private`, `mutability: evolving`, `owner: notices`, phrase `"follows {value}"`.
- **Step-31 §8.5.** "The topic vocabulary is the `topics:` keys of `configure/notices.yaml`, exposed to S30 as
  `notices.topics`."
- **Q-S31g-3 (ruled yes, 2026-10-20) and R-S31b-g-2.** "A `topics` record from `notices` to every living person
  (topic, body kind, validity bounds, `per_day`, `may_publish`, `subscribed`)", merged before S31-e and S31-g.
- **Lane charter (2026-10-18).** No world-specific hard-coding. Every acceptance criterion names a second world
  or a generated variant.
- **R-LOAD-1.** Locally: the commit's own test files, `cargo fmt --check`, `cargo clippy -p <crate>`. CI on the
  exact head is the gate.

---

## 1. Goal

Two small things. **Part A:** where a world enables `attributes`, a person's subscriptions become the S30 attribute
`notices.subscriptions`. They are authored, defaulted by role and shown like every other attribute. `notices`
still owns the state and still states the same facts. Where `attributes` is not enabled, nothing changes.
**Part B:** `notices` discloses a `topics` record, so a person can see which topics the world has before
subscribing.

---

## 2. Scope

### 2.1 In scope

1. **A general loader seam: a vocabulary from a pack's own configuration** (F-b2-1, Q-S31b2-1). A pack's
   configuration may yield data files for an owner's directory, collected like shipped data. `notices` uses it to
   yield `vocabularies/topics.yaml` from its topics. Files: `authoring/src/configuration.rs` (one provided
   method), `worldpack/src/data.rs` (collect the yielded files), their tests.
2. **`systems/notices`, Part A:**
   - `attributes/subscriptions.yaml` (new), shipped with `ships_data!`: the N-2 declaration.
   - `Cargo.toml`: add `mineworld-attributes-model`.
   - `src/configuration.rs`: yield `vocabularies/topics.yaml` (`topics: [<sorted topic ids>]`).
   - `src/system.rs`: `impl AttributeOwner` (`project`); subscribe to `attributes-handed` and
     `attributes-composed`; the two refusals of §5.4; `subscriptions` record gains `attributes`.
   - `src/component.rs`: `DisclosedSubscriptions { topics, attributes? }`.
   - `tests/attributes.rs` (new) and `tests/support/mod.rs`.
3. **`systems/notices`, Part B:** `src/component.rs` (`TopicsRecord`, `TopicLine`), `src/system.rs`
   (`discloses`), `tests/topics.rs` (new).
4. **`tools/cli/tests/notices.rs`:** the real-binary arms of AC-b2-5 and AC-b2-6.
5. **Docs:** `ARC-124` and `ARC-115` notes; `docs/MODULE_SPEC.md` §4.1 (the `notices` row, "Vocabularies");
   `systems/notices/README.md`; regenerated `docs/reference/` (`mineworld reference`).

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| The agent's `View.topics` line | S31-g (its C0 finds this record merged; Q-S31b2-4) |
| The 2D / 3D topics panel, words for the new codes | S31-e |
| Market Town enabling `attributes`, or moving its subscriptions to `roles` | S31-f / S31-f2, if ever (Q-S31b2-2) |
| `economy.wealth-band` and its configured vocabulary `economy.wealth-bands` | S31-f2 (uses C1's seam) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `cognition/`, `authoring/attributes/` | never in this PR (I-b2-1) |

---

## 3. Audit (`origin/main @ fec906f2`, 2026-10-20)

### 3.1 Source anchors

| Id | Fact | Source |
| --- | --- | --- |
| A-1 | `AttributeOwner` has one method, `project(world, person, attribute) -> Option<AttributeValue>`. `disclose_owned::<O>` returns `None` when the world has no running catalogue | `authoring/attributes/src/owner.rs:19-47` |
| A-2 | `attributes-handed { person, owner, values }` is owned by `attributes`, schema 1 | `authoring/attributes/src/facts.rs:72-85` |
| A-3 | `attributes` states one `attributes-handed` per (person, owner) with at least one owned value, `SystemInternal`, `about: [person]`, after the person's `traits-stated` | `systems/attributes/src/section.rs:119-161` (loop at 148) |
| A-4 | Resolution precedence: authored, lock, profile sample, roles, declaration default. Owner values go to `owned`, not to `Traits` | `authoring/attributes/src/sample/resolve.rs:1-14, 262-305`; `systems/attributes/src/section.rs:106` |
| A-5 | `roles` live only in a population profile (`populations/*.yaml`). A roles-only profile is valid (`distributions` defaults to empty). A profile needs `configure/attributes.yaml` with `profile` and `seed` | `authoring/attributes/src/profile/mod.rs:133-148`; `authoring/attributes/src/config.rs:13-35` |
| A-6 | **Vocabularies come only from `vocabularies/*.yaml` files. A file's namespace is its pack's id.** There is no path from a pack's configuration to a vocabulary | `authoring/attributes/src/catalogue.rs:297-309, 923-947` |
| A-7 | An override may not change `kind` or `vocabulary` | `authoring/attributes/src/catalogue.rs:1040-1049` |
| A-8 | `MODULE_SPEC.md` says a vocabulary may come "from its own world configuration, through its owner interface". Its example ships a fixed `vocabularies/topics.yaml` in `notices` | `docs/MODULE_SPEC.md:1449-1461` |
| A-9 | The loader builds each owner's context from files it collects (roots and shipped data), as YAML text. Each pack's configuration reaches only its own context | `worldpack/src/data.rs:384-425`; `authoring/src/data.rs:204-206, 312-331` |
| A-10 | Genesis order: configuration facts, then context facts (`attributes-composed`), then sections and resolutions. All are recorded, then reduced as one generation in that order; reactions cascade after | `docs/MODULE_SPEC.md:477-482, 398-406`; `kernel/src/dispatch.rs:500-529` |
| A-11 | `employment` is an owner already: `ships_data!("attributes/occupation.yaml")`, `impl AttributeOwner`, and its record carries `attributes: Option<Value>`, omitted when `None` | `systems/employment/src/system.rs:56-76, 440-462`; `systems/employment/src/component.rs:407-412` |
| A-12 | `employment.occupation` is `derived`, so nothing is ever handed to it. `notices.subscriptions` is the first owner value that is **handed** | `systems/employment/attributes/occupation.yaml` |
| A-13 | `notices` seeds `subscriptions-seeded { origin: default }` from `defaults.by_tag` in its configuration seed, and `{ origin: authored }` from the `notices:` section | `systems/notices/src/configuration.rs:130-184`; `systems/notices/src/section.rs:77-94` |
| A-14 | `Origin` is `Default` (insert when the person has none) or `Authored` (replace) | `systems/notices/src/event.rs:111-116`; reduction at `systems/notices/src/system.rs:201-216` |
| A-15 | `notices` discloses `subscriptions` (the component as is) and `notice-board`, both to the person only | `systems/notices/src/system.rs:307-344` |
| A-16 | `notices` subscribes to nothing of another pack today. Subscribing needs no dependency (`ARC-28` item 2) | `systems/notices/src/system.rs:84-101` |
| A-17 | `TopicId` is dotted lowercase segments, ≤ 64 bytes. A `ValueSlug` is `^[a-z0-9][a-z0-9_.-]{0,63}$`. Every `TopicId` is a valid `ValueSlug` | `systems/notices/src/topic.rs:41-71`; `authoring/attributes/src/id.rs:1-9` |
| A-18 | No world enables `notices` or `attributes`: not Social Café, not Market Town | `worlds/*/world.yaml` `systems:` lists |
| A-19 | S31-f (frozen 2026-10-20, not merged) gives Market Town `configure/notices.yaml` with `defaults.by_tag`, and no `attributes` | `pr-s31f-market-town.md:450-470` |
| A-20 | No record lists the topics. `notices-configured` is `SystemInternal`. No component type is named `topics` | `pr-s31g-agent-view.md:186`; `git grep 'component_type = "topics"'` (none) |
| A-21 | `ConfiguredTopic` already holds `topic`, `body`, `publishers`, `valid_min`, `valid_max`, `per_day` in the board state | `systems/notices/src/event.rs:28-80` |

### 3.2 Findings

- **F-b2-1 — The topic vocabulary has no source.** N-2 names `notices.topics`. Step-31 §8.5 says its values are
  the world's configured topics. The code builds vocabularies only from files, namespaced by the shipping pack
  (A-6), and a world cannot override one (A-7). So `notices` could only ship a fixed topic list, which is world
  content hard-coded in a pack. `MODULE_SPEC.md` already promises the configuration path (A-8). Code and spec
  disagree; the code is incomplete. `economy.wealth-bands` needs the same path. → **Q-S31b2-1.**
- **F-b2-2 — Dropping `defaults.by_tag` breaks a frozen world.** The `ARC-124` note says drop it. But roles exist
  only with `attributes` and a profile (A-5), and Market Town (S31-f) uses `by_tag` without `attributes` (A-19).
  R-S30c-b-1 itself keeps the section where `attributes` is off. → **Q-S31b2-2.**
- **F-b2-3 — Two sources for one value in an `attributes` world.** With `attributes` on, a person could get
  subscriptions from `by_tag`, from a `notices:` section and from `attributes-handed`. The result would then hang
  on seed order. One source per world is needed (§5.4).
- **F-b2-4 — Which `origin` does a handed set carry?** `attributes-handed` carries values, not origins (A-3).
  The fact must keep its shape. `default` fits: in an `attributes` world nothing else seeds, so "insert when
  none" is exact (SD-b2-2).
- **F-b2-5 — The `topics` record must be about the observer only.** The ruling says "to every living person".
  `subscribed` and `may_publish` are about the observer. Disclosing the record about another subject would leak
  their subscriptions (I-S31-9). So every person receives it about themself (§5.6).

---

## 4. Reuse

| Need | Verdict |
| --- | --- |
| A vocabulary from a configuration | Reuse the shipped-data path (`ships_data!`, `ARC-121` item 4): a configuration yields files that are collected exactly like shipped files. No new catalogue code; `attributes` sees one more file under `notices`' namespace |
| The owner disclosure | Reuse `disclose_owned` and employment's `attributes: Option<Value>` field (A-11) |
| No external library is involved | — |

---

## 5. Design

### 5.1 The seam: configured data (C1)

```rust
// authoring/src/configuration.rs, on PackConfiguration (provided)
/// Data files this configuration yields for an owner's directory, collected like shipped data.
fn yields(configuration: &Self::Configuration) -> Vec<YieldedFile> { Vec::new() }

pub struct YieldedFile { pub directory: DataDirectory, pub name: String, pub text: String }
```

- The loader decodes configurations first (A-9), then collects each enabled pack's yielded files with
  `origin.pack` = the configuring pack and the shown path `configure/<pack>.yaml → <directory>/<name>`.
- A yielded file follows every shipped-file rule: the directory must belong to an enabled owner, refusals name
  the configuring pack and the shown path. If the owner is not enabled, the yielded file is skipped, as a shipped
  file is.
- The kernel, `contracts/` and `authoring/attributes/` do not change.

### 5.2 The declaration and the vocabulary (C2)

```yaml
# systems/notices/attributes/subscriptions.yaml (shipped with ships_data!)
format: 1
attributes:
  subscriptions:
    group: behaviour
    kind: { set: { vocabulary: notices.topics, max: 32 } }
    visibility: private
    mutability: evolving
    owner: notices
    label: Subscriptions
    phrase: "follows {value}"
```

```yaml
# yielded by configure/notices.yaml (§5.1): vocabularies/topics.yaml
format: 1
vocabularies:
  topics: [news.daily, prices.ingredients]   # the topic keys, sorted
```

- No `default`. A person with no authored value, lock entry or role gets no handed value and no `Subscriptions`
  component, as today.
- Other required fields (`facets`, `prompt_rank`) follow the declaration format; C0 pins them from
  `declaration.rs`.

### 5.3 Receiving and projecting (C2)

- `notices` subscribes to `attributes-handed`. For a fact with `owner: notices`, its reaction states one
  `subscriptions-seeded { person, topics, origin: default }`, `SystemInternal`, `about: [person]`, caused by the
  handed fact. The existing reduction then writes `Subscriptions`. **The fact is the same type, shape and
  visibility as today.**
- A handed value that is not a `Set` is refused `PreconditionFailed`. A topic not on the board is refused
  `unknown-topic` (the existing check).
- `project(world, person, notices.subscriptions)` returns `AttributeValue::Set` of the component's topics.
  Absent component → `None`.
- `subscribe` and `unsubscribe` are unchanged. `attributes` is not involved (`ARC-124` item 4).

### 5.4 One source per world (C2; F-b2-3; Q-S31b2-2)

| World | Sources of initial subscriptions |
| --- | --- |
| without `attributes` | `defaults.by_tag` and the `notices:` section, exactly as today |
| with `attributes` | the `attributes:` section, lock and `roles` only, through `attributes-handed` |

In a world with `attributes` whose catalogue declares `notices.subscriptions`:

- `notices` subscribes to `attributes-composed`. If, at its reduction, any person already holds `Subscriptions`,
  those came from `defaults.by_tag` (A-10 order). It is refused `subscriptions-are-attributes`, detail
  "use roles in a population profile instead of defaults.by_tag".
- A `subscriptions-seeded { origin: authored }` (only the `notices:` section states one) is refused
  `subscriptions-are-attributes`, detail "write attributes: { notices.subscriptions: [...] }".
- Both are genesis refusals, so `mineworld validate` names them.

### 5.5 Disclosure (C2)

```text
subscriptions record = DisclosedSubscriptions { topics, attributes?: Value }
```

- `topics` as today. `attributes` is `disclose_owned::<NoticesSystem>(world, observer, subject)`, omitted when
  `None` (employment's pattern, A-11). Without `attributes` the record is byte-identical to today's.
- Still to the person only.

### 5.6 Part B: the `topics` record (C3)

```text
topics record = TopicsRecord { topics: [TopicLine] }      // every declared topic, in id order, ≤ 32
TopicLine { topic, body: prices | text, valid_min, valid_max, per_day, may_publish: bool, subscribed: bool }
```

- Component type `topics`, owned by `notices`, declared and never carried (like `notice-board`).
- **Rule:** disclosed to every living Person about itself (observer = subject), when the world is configured.
  Never about another subject (F-b2-5). Present even when the person subscribes to nothing.
- `valid_min`, `valid_max` are seconds, as in `notices-configured`. `may_publish` = the person is one of the
  topic's publishers. `subscribed` = the topic is in the person's `Subscriptions`.
- The publisher list is not disclosed.
- No fact. Computed from the board state and the component at each read.

### 5.7 Bounded decisions

- **SD-b2-1.** The yielded vocabulary is named `topics`, so its id is `notices.topics` (N-2).
- **SD-b2-2.** Handed sets carry `origin: default` (F-b2-4).
- **SD-b2-3.** A world that enables `notices` and `attributes` but has no `configure/notices.yaml` yields no
  vocabulary, so the declaration is refused at load (`UnknownTopic` naming `notices.topics`). A world with no
  topics has nothing to subscribe to.
- **SD-b2-4.** Record field names follow `notices-configured` (`valid_min`, `valid_max`). S31-g maps them to its
  `_s` names.

---

## 6. Invariants

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-b2-1 | No change under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `cognition/`, `authoring/attributes/` | C5 diff scan |
| I-b2-2 | **No fact changes.** No new event type in `notices`. `subscriptions-seeded`, `subscribed`, `unsubscribed`, `notice-published`, `notices-configured` keep their schema versions, fields and visibility | `tests/attributes.rs` decodes every fact with today's types; schema-version scan |
| I-b2-3 | **Social Café and Market Town behave the same.** Neither enables `notices` or `attributes` (A-18). Their golden logs, observations and acceptance runs do not move | the existing goldens and `tests/acceptance` in CI, unchanged |
| I-b2-4 | A world without `attributes` behaves as before: same facts, same components; records equal except the new `topics` record | AC-b2-1 |
| I-b2-5 | One writer: only `notices` writes `Subscriptions` | the kernel's `OwnedBy`; I-c-3 of S30-c |
| I-b2-6 | No world-specific value in code: the vocabulary is yielded from whatever topics a world declares | AC-b2-3 (relabelled variant) |
| I-b2-7 | `subscriptions` and `topics` are never disclosed about another subject | AC-b2-6 |

---

## 7. Acceptance criteria (fixed before measuring)

Worlds: **W1** is S31-b's scratch fixture (publisher, two subscribers by `by_tag`, one non-subscriber).
**W1-attr** is W1 with `attributes` enabled, `defaults.by_tag` removed, and a roles-only profile with the same
mapping (`{ when: { tags: [barista] }, add: { notices.subscriptions: [prices.ingredients] } }`). **W2** is a
generated variant of W1: topics renamed (`prices.ingredients` → `market.daily`), tags renamed, a third topic
added. **W2-attr** is W2 the same way.

| Id | Criterion |
| --- | --- |
| AC-b2-1 | **No fact changes (W1, W2).** With `attributes` off, the event log is byte-identical to `main`'s for the same scripted run. |
| AC-b2-2 | **Same outcome by roles (W1 vs W1-attr, W2 vs W2-attr).** Every person's `Subscriptions` at genesis is equal. The `subscriptions-seeded` facts are equal in `person` and `topics`. Every `notice-published` audience is equal. |
| AC-b2-3 | **Vocabulary from the world.** In W2-attr the catalogue's `notices.topics` equals W2's topic keys. An `attributes: { notices.subscriptions: [prices.ingredients] }` in W2-attr fails `mineworld validate` naming `UnknownTopic` and the nearest topic. |
| AC-b2-4 | **One source.** W1-attr plus `defaults.by_tag` fails `validate` naming `subscriptions-are-attributes`. W1-attr plus a `notices:` section fails the same way. W1 (no `attributes`) with both still loads. |
| AC-b2-5 | **Disclosure (real binary, W1-attr).** A subscriber's own `subscriptions` record carries `attributes` with the rendered phrase. Another observer gets neither record about them. In W1 the record has no `attributes` field. A runtime `subscribe` changes the projected value. |
| AC-b2-6 | **Topics record (W1, W2).** Every living person's own observation lists every declared topic in id order with correct `may_publish` and `subscribed`. A non-subscriber sees it too. No observer gets it about another person. After `subscribe`, `subscribed` flips. A world without `configure/notices.yaml` has no record. |
| AC-b2-7 | **Kill and resume (W1-attr).** Server killed after a `subscribe`, restarted: `Subscriptions` and the projected attribute match; `mineworld replay` agrees. |
| AC-b2-8 | **Untouched worlds.** Social Café and Market Town goldens and acceptance tests pass unchanged on CI. |

### 7.1 Planted mutations

| Id | Plant | Must turn red |
| --- | --- | --- |
| M-b2-1 | The handed reaction states `origin: authored` and a by_tag default survives | AC-b2-4 |
| M-b2-2 | `project` returns the topics unsorted, or drops the last | AC-b2-5 |
| M-b2-3 | The yielded vocabulary is a fixed list instead of the configured keys | AC-b2-3 (W2-attr) |
| M-b2-4 | `topics` is disclosed when observer ≠ subject | AC-b2-6 |
| M-b2-5 | `subscribed` is read from the board instead of the component | AC-b2-6 (after `subscribe`) |

---

## 8. Test ownership

| Layer | Owns | File |
| --- | --- | --- |
| Loader | yielded files: collected, skipped when the owner is off, refusals named | `worldpack/src/data/tests.rs` |
| Pack | handed → seeded; refusals of §5.4; projection; both records | `systems/notices/tests/attributes.rs`, `tests/topics.rs` |
| Real binary | AC-b2-5, AC-b2-6, AC-b2-7 on W1 / W1-attr / W2 | `tools/cli/tests/notices.rs` |

No test asserts that a getter returns a field.

---

## 9. Commit plan

Each commit tracks implementation, validation and review separately. `[x]` needs evidence. Run everything from
the implementation worktree. No `sed -i`, no `python3 -c`, no heredoc writes. Stage by explicit path.

#### C0 — Base and ledger (Markdown only)

- **Files:** this file (§15), `handoff-s31b2.md` (new).
- **Steps:** record the base. Re-read A-1 … A-21; record moved lines. Confirm A-10's reduction order with one
  read of `kernel/src/dispatch.rs` (a reaction to `attributes-handed` lands after every genesis fact; the
  catalogue Process exists before section seeds reduce). If either is false: material stop. Pin the declaration's
  required fields (§5.2). Record whether S31-f has merged.
- [ ] Implementation. [ ] Validation: `check_doc_headings.py`, `check_decision_ids.py`. [ ] Review: §2 holds on
  the base.
- **Message:** `docs(plan): S31-b2 — open the ledger at the base`

#### C1 — Docs, then the configured-data seam

- **Files:** `docs/decisions/ARC-124.md` (note: the adoption as realised; the seam; Q-S31b2-2's rule),
  `docs/MODULE_SPEC.md` §4.1 ("Vocabularies": the yielded file replaces the fixed example; §4.1 seam list),
  `authoring/src/configuration.rs`, `worldpack/src/data.rs`, `worldpack/src/data/tests.rs`.
- **Steps:** add `yields` and `YieldedFile` (§5.1); collect yielded files with shipped files; tests: collected,
  skipped when the owner is not enabled, a bad yielded file refused naming the configuring pack.
- [ ] Implementation. [ ] Validation: `cargo test -p mineworld-worldpack --lib data`; clippy on both crates.
  [ ] Review: is any existing world's load changed? (Must be no: the default yields nothing.)
- **Message:** `feat(authoring): a configuration may yield data files for an owner's directory`

#### C2 — `notices` is an `AttributeOwner` (Part A)

- **Files:** `systems/notices/{Cargo.toml, attributes/subscriptions.yaml, src/configuration.rs, src/system.rs,
  src/component.rs, src/lib.rs, tests/attributes.rs, tests/support/mod.rs, README.md}`, `Cargo.lock`.
- **Steps:** §§5.2–5.5. Tests: AC-b2-1 (pack level), AC-b2-2, AC-b2-3, AC-b2-4, on W1/W2 built in
  `tests/support`. Mutations M-b2-1 … M-b2-3.
- [ ] Implementation. [ ] Validation: `cargo test -p mineworld-notices --test attributes` and the four existing
  test files; clippy. [ ] Review: does any fact change shape (I-b2-2)? Is there a path where two sources seed
  one person?
- **Message:** `feat(notices): subscriptions are the notices.subscriptions attribute where attributes is enabled`

#### C3 — The `topics` record (Part B)

- **Files:** `systems/notices/{src/component.rs, src/system.rs, tests/topics.rs}`, `docs/decisions/ARC-115.md`
  (note), `systems/notices/README.md`.
- **Steps:** §5.6; install the component type; tests for AC-b2-6 at pack level; mutations M-b2-4, M-b2-5.
- [ ] Implementation. [ ] Validation: `cargo test -p mineworld-notices --test topics`; clippy. [ ] Review: is
  anything about another person disclosed?
- **Message:** `feat(notices): a topics record lists the world's topics to each person`

#### C4 — Real binary and reference

- **Files:** `tools/cli/tests/notices.rs`, `docs/reference/` (regenerated).
- **Steps:** AC-b2-5, AC-b2-6, AC-b2-7 on W1, W1-attr, W2; `mineworld reference` regenerated.
- [ ] Implementation. [ ] Validation: `cargo test -p mineworld-cli --test notices`; `mineworld reference --check`.
  [ ] Review: do the real-binary arms read the server's output, not the pack's state?
- **Message:** `test(cli): notices as attributes and the topics record, end to end`

#### C5 — Ready for review

- **Files:** this file (§15), the handoff.
- **Steps:** I-b2-1 diff scan; push; CI green on the exact head (AC-b2-8); report MERGE-READY with the head SHA.
- [ ] Implementation. [ ] Validation: CI run URL on the head. [ ] Review: every AC has evidence.
- **Message:** `docs(plan): S31-b2 — ready for review`

---

## 10. Requirements on other PRs

- **R-S31g-b2-1 (S31-g).** `View.topics` reads the `topics` record (§5.6); fields map as SD-b2-4.
- **R-S31e-b2-1 (S31-e).** The board panel may list topics from the `topics` record; words for
  `subscriptions-are-attributes`.
- **R-S31f2-b2-1 (S31-f2).** `economy.wealth-bands` uses `yields` (§5.1), not a shipped list.
- **R-S31f-b2-1 (S31-f).** Market Town keeps `defaults.by_tag` (Q-S31b2-2). If it ever enables `attributes`, it
  moves the mapping to `roles` in the same PR.

## 11. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-b2-1 | Genesis order is not as A-10 reads, so §5.4's detection misfires | C0 checks it; a mismatch is a material stop |
| R-b2-2 | The seam is used for things other than vocabularies | Yielded files obey every shipped-file rule; the owner still decodes and refuses them |
| R-b2-3 | S31-f merges first and its goldens include notices observations | The new `topics` record moves them. Whichever lands second updates those goldens; I-b2-3 covers only the worlds on main at C0 |

## 12. Execution contract

- **Authority:** C0 … C5 inside §2.1. Push the branch, open the PR, fix CI. Never merge.
- **Material stops:** any fact change (I-b2-2); any edit outside §2.1; A-10 false; a change to
  `authoring/attributes/`.
- **Local load:** R-LOAD-1. Heavy jobs through `cast-slots/with-slot.sh`.
- **Ends at:** READY FOR OPERATOR REVIEW (MERGE-READY to the coordinator).

## 13. Lane notes

Part A and Part B are independent. If the operator freezes Part B alone, C3 and its real-binary arm can ship
first as their own PR.

## 14. Operator questions (each with a recommended answer)

| Id | Question | Recommended answer |
| --- | --- | --- |
| **Q-S31b2-1** (F-b2-1) | The topic vocabulary `notices.topics` has no source: vocabularies come only from shipped files, and a world cannot override one. How does the world's topic list reach the catalogue? | **A small general loader seam: a configuration may yield data files, collected like shipped data (§5.1).** `notices` yields `vocabularies/topics.yaml` from its topics. One source, no change to `attributes`, and economy's wealth bands need the same. Rejected: a fixed topic list in the pack (hard-codes world content); a world-written vocabulary file (two copies of one list) |
| **Q-S31b2-2** (F-b2-2) | The `ARC-124` note says drop `defaults.by_tag`. Roles need `attributes` and a profile, and Market Town uses `by_tag` without `attributes`. Remove `by_tag`, or keep it? | **Keep `by_tag` and the `notices:` section for worlds without `attributes`. In a world with `attributes`, they are refused (`subscriptions-are-attributes`) and roles are the only defaults (§5.4).** One source per world. Market Town needs no change. A dated `ARC-124` note records the change to "drops `defaults.by_tag`" |
| **Q-S31b2-3** (F-b2-4) | `attributes-handed` carries no origin. Which `origin` does the resulting `subscriptions-seeded` carry? | **`default`.** No new enum value, so no fact changes. In an `attributes` world nothing else seeds, so "insert when none" is exact |
| **Q-S31b2-4** (R-S31b-g-2) | R-S31b-g-2 also mentions "the agent line of §5.14". Does this PR add `View.topics` to the agent adapter? | **No.** S31-g adds it; this PR stays in Rust and merges first. S31-g's own table already says its C0 adds the line when the record is merged |
| **Q-S31b2-5** (F-b2-5) | The ruling says the `topics` record goes "to every living person". It carries `subscribed` and `may_publish`. Disclose it about other people too? | **No. Every person gets it about themself only.** Otherwise a viewer would learn someone else's subscriptions, which are private (I-S31-9) |

### 14.1 Rulings

Ruled 2026-10-26 by the operator in the primary session, relayed by the coordinator: **every question is
answered as recommended.**

| Id | Ruling |
| --- | --- |
| Q-S31b2-1 | A general loader seam: a configuration may yield data files, collected like shipped data (§5.1) |
| Q-S31b2-2 | Keep `defaults.by_tag` and the `notices:` section without `attributes`; refuse both with `attributes` (`subscriptions-are-attributes`, §5.4) |
| Q-S31b2-3 | Handed sets carry `origin: default` |
| Q-S31b2-4 | No `View.topics` in the agent adapter in this PR; it stays in Rust |
| Q-S31b2-5 | The `topics` record is disclosed to each person about themself only |

**Drift recorded at C0 (2026-10-26).** "Must merge before S31-e and S31-g" no longer holds: S31-e (#422) and S31-g
(#418) merged first. Per Q-S31b2-4 this PR still does not touch the agent adapter, so **R-S31g-b2-1** (`View.topics`
reads the `topics` record) and **R-S31e-b2-1** (the board panel lists topics; words for
`subscriptions-are-attributes`) are now **outstanding follow-ups** for later PRs, not satisfied by S31-e or S31-g
as merged. If S31-e or S31-g goldens or tests move because of the new `topics` record, updating them is a bounded
correction under R-b2-3's logic and is recorded in §15.

**Approved scope extension (operator, 2026-10-27, relayed by the coordinator).** The C4 material stop (§15) is
ruled as proposed: `compare` in `worldpack/src/configure.rs` checks only saved facts caused by
`Causation::WorldGenesis`, with one test in `worldpack/src/configure/tests.rs`. Both files join §2.1 for this PR.

## 15. Ledger (live during implementation)

### Publication on 2026-11-27

Owner: publication agent, worktree `/Users/yuema137/mineworld-worktrees/pub-s31b2` (branch
`recover/s31b2-notices-attributes`, pushed as `mvp0/pr-s31b2-notices`; the old remote `mvp0/pr-s31b2`
holds the October WIP and is left untouched). Authority: the coordinator's 2026-11-27 dispatch — push, open
the PR, repair CI; no merge, tag, release or release-job dispatch.

**Merge with main.** `origin/main @ 02b17504` merged (not rebased) into `f10b1ee8` as `fc3f1541`: clean
automatic merge; no file is changed on both sides since base `2b4d36fa` (main's side touched only CI
scripts/actions, guide tests, `social_composition.rs`, `packages/tests/far_field.rs`, 3D probe scripts,
READMEs and plan files).

**Local validation on `fc3f1541`** (the executable content of every later commit on this branch; later
commits are documentation only). Every Cargo command ran through `cast-slots/with-slot.sh`, with
`CARGO_TARGET_DIR` inside the worktree; logs in the worktree's ignored `target/b2-evidence/` (not durable).

| Check | Result |
| --- | --- |
| `cargo fmt --all --check`; `git diff --check` | PASS |
| `cargo clippy -p mineworld-authoring -p mineworld-worldpack -p mineworld-notices -p mineworld-cli --all-targets -- -D warnings` | PASS, no warnings |
| `cargo test -p mineworld-notices` | PASS, 30 (attributes 6, board 3, configuration 4, parity 1, publish 7, subscriptions 6, topics 3) |
| `cargo test -p mineworld-worldpack --lib data` | PASS, 9 |
| `cargo test -p mineworld-worldpack --lib configure::tests::the_comparator` | PASS, 2 (mutation proof is historical, not rerun) |
| `cargo test -p mineworld-cli --test notices -- --skip no_random_subscription` | PASS, 7 (1 filtered: the 30-day scenario, NOT RUN) |
| `mineworld reference --check docs/reference` | PASS, 46 files |
| `check_doc_headings.py`, `check_decision_ids.py` (after the F-1 docs commit) | PASS |

Not run locally: full workspace suite, `no_random_subscription`, `ac1_composability` on this head (its 16/16
PASS was on `f10b1ee8`, historical), AC-b2-8. CI on the exact PR head is the gate.

**Independent red-team review (2026-11-27):** PASS-WITH-FOLLOWUPS on `f10b1ee8` + merge `fc3f1541`
(`/Users/yuema137/mineworld-worktrees/_reviews/s31b2.md`). Dispositions:

- **F-1 (medium, fixed in this PR):** the comparator's WorldGenesis-only rule was recorded only here. Added a
  dated `ARC-61` note (item 7) and aligned `docs/MODULE_SPEC.md` (configure drift paragraph). CLAUDE.md §2.1
  rule 4.
- **F-2 (low, open follow-up):** the `defaults.by_tag` refusal in an `attributes` world is occupancy-based
  (fires only when a tag actually seeds someone). An unmatched `by_tag` loads silently; no second state
  source results. A configuration-level declarative check is a candidate follow-up.
- **F-3 (low, open):** the new `topics` record reaches every observation in notices-enabled scratch worlds
  (agent MCP tests, `client_trade.rs`, 2D HUD generic row). The `python` and Godot CI layers must pass on the
  exact head before merge; under CI's `deferred` mode (ARC-154) a normal PR runs `fast` only, so those layers
  run only in `full` mode (the `milestone` label or a milestone dispatch) — an operator/coordinator cost
  decision, not taken here. R-S31g-b2-1 and R-S31e-b2-1 remain open.
- **F-4 (low, process, open):** the October/November `/tmp` evidence (recovery worktree, snapshot logs,
  comparator mutant/restored logs, parity-baseline log, PR body, prior review) was wiped on reboot. Those
  claims are historical; the durable evidence is the tracked parity fixtures with their SHA-256 and the tests.
- **Size (open follow-up):** `tools/cli/tests/notices.rs` is 697 lines, near the ~800 trigger; the next
  notices CLI test splits the file by AC family.
- F-5 notes: informational, no action.

### Recovery on 2026-11-22

The coordinator delegated local completion on base `2b4d36fa`, branch
`recover/s31b2-notices-attributes`, worktree `/tmp/mineworld-recover-s31b2`. The old
`/Users/yuema137/mineworld-worktrees/impl-s31b2` remains read-only. Its five commits
`18c9c2cd`, `c90abc88`, `93d30dda`, `7d93acb5`, `5b533593` were recovered as
`29bec581`, `b7a501cb`, `2136b97c`, `6a2baa03`, `b3450fd1`, respectively, without conflicts.
The three dirty files (this ledger and `worldpack/src/configure{,/tests}.rs`) were recovered
from `/tmp/mineworld-s31b2-snapshot/dirty.patch`, SHA256
`e6b5f5f0fcfb61acf0981cb1599d4942f179dfe22b6a7ac40b31cf028302fd54`.
There were no untracked files. Root and old-worktree content were not edited.

The October 26 freeze supersedes the historical DRAFT metadata above. The October 27
14:16 operator ruling in `/Users/yuema137/mineworld-demos/progress/pending-decisions.md`
explicitly approves the WorldGenesis-only comparison and regression: the historical C4
material stop below is resolved, not a pending approval. The latest root `CLAUDE.md`
standing rules apply even though they are uncommitted.

Endpoint authority for this recovery is the coordinator's 2026-11-22 dispatch: implement,
validate locally, review, and make local semantic commits and a PR body; no push, PR
publication, merge, tag, paid API, subagents, or shared-plan/dashboard writes. Independent
red-team review is coordinated by the primary as soon as local evidence is ready. Remote
CI remains NOT RUN and is not a prerequisite to starting that review.

Validation plan: slot-guarded comparator tests establish that direct genesis configuration
still detects drift while reaction facts do not. The existing real-binary
`notices_attributes_disclosed_and_resumed` test owns actual subscription → SIGKILL →
restart → projected disclosure → replay behavior. Run that test, the bounded notices
integration files and loader data tests, then formatting, scoped clippy, documentation
checks and reference verification. No workspace suite. Gate 1 is NOT REQUIRED: no
model-facing semantics change or paid model use. Gate 2 is the real-binary restart test,
target under ten minutes per run after compilation. Preserve failed attempts and exact
commands below; old October evidence is historical, not evidence for this recovery head.

Recovery findings: the preserved comparator regression selected genesis index zero, which is
`arrived`, not `probe-configured`. The first focused run FAILED (1 pass, 1 failure): its
direct-genesis negative arm correctly exposed that the test was outside the comparator's
owned types. Select the event by its declared type and assert the named owner and payload.
The comparator implementation itself retains the approved single causation filter.

The configured vocabulary also emitted bare YAML scalars. A new catalogue integration
regression with valid topic ids `null` and `true` FAILED because `null` could not decode as
a string. Encode the list through the existing JSON serializer (valid YAML), preserving
all topic ids without expanding the public vocabulary or adding a dependency. This is a
bounded correction to AC-b2-3, not a new feature.

### Recovery validation and review

All commands below run from `/tmp/mineworld-recover-s31b2`; every Cargo test/clippy
command uses `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
Tested content is recovery HEAD `b3450fd1` plus the reviewed recovery diff; final commit
mapping is recorded in the handoff after committing without changing executable content.

| Claim | Command / observation | Result |
| --- | --- | --- |
| Configuration drift distinguishes direct input from reaction output | `cargo test -p mineworld-worldpack --lib configure::tests::the_comparator` | PASS, 2 tests; initial repaired run 2.51 s including compilation |
| Comparator regression is discriminative | Remove only the WorldGenesis filter, run the same target; restore source automatically and rerun | Expected FAIL in reaction regression; restored PASS, 2 tests. Mutation and restored logs in `/tmp/mineworld-s31b2-snapshot/comparator-{mutant,restored}.log` |
| Pack ownership, role-equivalent initialization, vocabulary, privacy, legacy behavior, topics | `cargo test -p mineworld-notices` | PASS, 30 tests across seven integration targets, 8.74 s build; no unit/doc tests counted as evidence |
| Loader seam ordering and errors | `cargo test -p mineworld-worldpack --lib data` | PASS, 9 tests, 1.71 s build |
| Actual binary validation, privacy, dynamic subscriptions, kill/restart/replay | `cargo test -p mineworld-cli --test notices -- --skip no_random_subscription` | PASS, 7 tests, 1.65 s execution after 1m41s initial build; excludes only the existing 30-day scenario |
| Complete disclosure equality after restart (final strengthened test) | `cargo test -p mineworld-cli --test notices notices_attributes_disclosed_and_resumed` | PASS, 1 test, 1.30 s execution after 14.80 s rebuild |
| Scoped compilation and lint | `cargo clippy -p mineworld-authoring -p mineworld-worldpack -p mineworld-notices -p mineworld-cli --all-targets -- -D warnings` | PASS, 15.53 s, no warnings |
| Generated reference | `target/debug/mineworld reference --check docs/reference` | PASS, 46 files match; no regeneration needed |
| Static documentation and hygiene | `cargo fmt --all --check`, `git diff --check`, `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py`, `python3 scripts/check_scratch.py scan` | PASS after formatting the new regression; 254 headings, 257 decision ids, 386 test sources |

AC-b2-1 now has independent complete event-envelope byte evidence, beyond the historical
component comparison. A detached baseline worktree `/tmp/mineworld-s31b2-parity` at
`2b4d36fa` ran the same `tests/parity.rs` scenario with its original support and production
sources. W1 and W2 each initialize tag defaults and an authored empty override, subscribe,
publish text and prices, then unsubscribe. W2 renames the price topic and tag and adds a
third topic. Captured immutable files, including terminal newline:

- `systems/notices/tests/fixtures/pre-attributes-w1.json`, SHA256
  `372ade1bebf5154c29c52cdebc19712e97d00e85d137c5404690f278255aa9b1`.
- `systems/notices/tests/fixtures/pre-attributes-w2.json`, SHA256
  `496d1efdca38eee5cb923dcaa38a8e168a1c1d06f5aaa8b1fa398ed7285b3102`.

The baseline capture passed before these files were copied into the recovery tree;
`tests/parity.rs` then passed against their bytes. Expected values are never derived from
the implementation under review. Initial stdout is in the snapshot directory's
`parity-baseline.log`; the tracked fixtures and scenario are the durable evidence.

**Implementer logic and maintainability review (2026-11-22).**

- Ownership and modularity: only notices writes `Subscriptions`. The optional model-crate
  interface projects current component state and returns disclosure through the shared
  renderer. No duplicate attribute store, required System dependency, kernel concept,
  localization type, UI, or resolution-context API is introduced. The loader seam is generic
  and defaults to no files; files retain configuring-pack provenance and owner decoding.
- Robustness: malformed handed values fail explicitly, unknown topics are refused by the
  existing reducer, authored initialization is refused when the catalogue declares the
  attribute, and by-tag initialization is detected before context reduction completes.
  As §5.4 specifies, this detects defaults that actually seed a person; an unmatched tag
  default creates no second state source. Private records return early for other subjects.
  Topics use the live component, not a cached flag. Restart compares only assembly inputs;
  persisted reaction output remains checked by replay, not silently discarded.
- Newly introduced defects fixed: ambiguous YAML scalars are quoted, the comparator test
  selects the intended event, and stale component/dependency comments now describe optional
  attribute disclosure. Complete disclosure equality strengthens the real restart oracle.
- Size/readability: `notices/src/system.rs` grows from 345 to 481 lines and
  `worldpack/src/data.rs` from 456 to 534. The former remains the install/dispatch/disclosure
  implementation; its existing long reaction dispatcher grows by two explicit event cases,
  with handed-value conversion in a separate helper. The latter remains data collection and
  owner decoding; yielded-name checks and provenance formatting are separate small helpers.
  Both were reviewed at the size triggers: no new mixed domain responsibility, deep nesting,
  inheritance, or hidden mutable coupling was found. Extracting arbitrary fragments solely
  to meet a line count would reduce conceptual locality. Existing dispatch/loader size is
  retained debt, explicitly distinguished from the bounded new branches.
- Extensibility and reuse: subsequent configured vocabularies use the same optional yield
  hook, without adding a dependency or teaching the loader their domain. JSON serialization
  reuses the existing library and avoids hand-escaped YAML. No speculative abstraction was
  added beyond the frozen seam.
- Scope scan: no changes under the forbidden directories in I-b2-1, no event-schema change,
  no reference-world source/golden changes, no shared plans or dashboard edits. AC-b2-8's
  reference-world milestone/remote CI is NOT RUN locally under the coordinator's cost and
  publication limits. It is not represented as a local pass. The primary owns independent
  red-team review and later integration/remote validation.

### Historical implementation evidence

Recovery commit mapping: `b2a4408e` completes the October 27 comparison extension,
repairs its regression, and strengthens AC-b2-7's disclosure oracle. The following
`fix(notices): preserve scalar topic ids and pin pre-adoption event parity` commit
completes the bounded vocabulary correction and adds the frozen AC-b2-1 evidence.
Both implementation, deterministic validation, and implementer logic review are complete
for these recovery milestones; independent red-team review remains the primary's next step.

The executable recovery endpoint is `19e30172bc57775d4b1369df7ccdee8226745955`.
The subsequent documentation commit freezes the review packet; its exact SHA is sent to
the coordinator with the PR body and evidence paths. No remote/public action is performed.

The following October ledger is preserved as history. Its C4 failure and material stop
were resolved by the approved recovery above. C5's old public-push/remote-CI endpoint is
withheld under the current dispatch; local completion is recorded separately and is not
a claim of merge readiness or remote validation.

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | this commit | [x] | [x] | [x] |
| C1 | see `git log` (`feat(authoring): a configuration may yield …`) | [x] | [x] `cargo test -p mineworld-worldpack --lib data`: 9 passed (3 new); clippy `-D warnings` on `mineworld-worldpack`, `mineworld-authoring` clean | [x] no existing world's load changes: `yields` defaults to none, so `collect` adds nothing; the existing order test is unchanged and passes |
| C2 | see `git log` (`feat(notices): subscriptions are the notices.subscriptions attribute …`) | [x] | [x] `cargo test -p mineworld-notices` (all test files): 25 passed, incl. `--test attributes` 5 (AC-b2-1 pack half, AC-b2-2, AC-b2-3, AC-b2-4, AC-b2-5 pack half, on W1/W2 and W1-attr/W2-attr); clippy `-D warnings` clean | [x] no fact changes shape (I-b2-2: handed sets are the existing `subscriptions-seeded`, `origin: default`, SystemInternal, about the person); two sources cannot seed one person in an `attributes` world (by_tag refused at `attributes-composed`, authored refused at reduction); mutations M-b2-1 … M-b2-3 reasoned against the tests, not planted (speed-first charter) |
| C3 | see `git log` (`feat(notices): a topics record …`) | [x] | [x] `cargo test -p mineworld-notices`: 28 passed, incl. `--test topics` 3 (AC-b2-6 pack half on W1 and the W2 variant; unconfigured world); clippy clean | [x] nothing about another person is disclosed: `discloses` returns early when observer ≠ subject, and the test counts exactly one `topics` record per observation; `subscribed` reads the component (flips after `subscribe`) |
| C4 | see `git log` (`test(cli): …`) | [x] | [ ] **FAIL** — `cargo test -p mineworld-cli --test notices -- --skip no_random_subscription`: 6 passed, 1 failed (`notices_attributes_disclosed_and_resumed`, the AC-b2-7 restart); `mineworld reference --check docs/reference`: matches (46 files, nothing to regenerate) | [x] the arms read the server's observations and `mineworld validate`, not pack state |

**MATERIAL STOP at C4 (2026-10-26): AC-b2-7 cannot pass inside §2.1.** In W1-attr the first server run works
(AC-b2-5 and AC-b2-6 arms pass up to the kill), but restarting on the same save is refused:

```text
the world could not be built: the world's configuration differs from the save's: system 'notices'
(save: subscriptions-seeded {"person":{"entity":"11",...},"topics":["prices.ingredients"],"origin":"default"}
SystemInternal; this pack: nothing) — a save resumes only against the configuration it was created with
```

Cause: `WorldPack::check_configuration` (`worldpack/src/configure.rs` 563-618, `compare` 517-549) compares the
save's genesis facts of every configuration-fact type (`PackConfiguration::FACTS`; `notices` declares
`subscriptions-seeded` because `defaults.by_tag` seeds it) with the facts assembly states. The handed set's
`subscriptions-seeded` is recorded at genesis by **reaction** (caused by `attributes-handed`, §5.3), so the save has
it and assembly does not. Every world that enables `notices` and `attributes` with any handed value therefore
cannot resume or replay. A-10 holds; no fact changes shape; the defect is in the drift check's reading of
"genesis facts", which is outside §2.1.

Smallest proposed revision (needs a ruling: an edit outside §2.1): in `compare`, keep only saved facts whose
causation is `Causation::WorldGenesis` (the facts assembly re-states); reaction facts derive from them and are
re-derived by replay. One filter in `worldpack/src/configure.rs` plus a test in `worldpack/src/configure/tests.rs`.
Rejected in-scope alternatives: writing `Subscriptions` directly in the `attributes-handed` reaction (drops the
`subscriptions-seeded` fact AC-b2-2 requires); a new fact type for handed sets (a fact change, I-b2-2).
| C5 | — | [ ] | [ ] | [ ] |

**C0 evidence (2026-10-26).** Base `origin/main @ 0b2faf2f`, branch `mvp0/pr-s31b2`, worktree
`/Users/yuema137/mineworld-worktrees/impl-s31b2`.

- A-1 … A-21 re-read on the base; the facts hold. A-18 holds: no `worlds/*/world.yaml` enables `notices` or
  `attributes`. A-19: **S31-f has not merged** at the base (`worlds/market-town/configure/` has no `notices.yaml`).
- **A-10 confirmed** (`kernel/src/dispatch.rs` `genesis` 497-529 records every genesis fact then calls `reduce`
  once; `reduce` 606-661 reduces each fact of a generation in order, every subscriber in registration order,
  and reactions form the next generation). So: by_tag `subscriptions-seeded` (configuration facts) reduce before
  `attributes-composed` (context facts), which reduces before every section fact; the `attributes` reaction to
  `attributes-composed` starts the catalogue Process before any section seed reduces; a reaction to
  `attributes-handed` lands in a later generation. Because `notices` may be registered before `attributes`, the
  §5.4 by_tag check reads the declaration from the `attributes-composed` payload (`catalogue.attributes`), not
  from the running Process; the authored-seed check reads `RunningCatalogue::in_world`.
- **Declaration fields pinned (§5.2):** `RawDeclaration` (`authoring/attributes/src/declaration.rs:435-459`) makes
  `facets`, `prompt_rank`, `default`, `words`, `since` optional; `group`, `kind`, `visibility`, `mutability`,
  `label` required. N-2 is shipped as written, with no `facets` and no `prompt_rank`.
- Validation: `scripts/check_doc_headings.py`, `scripts/check_decision_ids.py` (see the commit). Review: §2 holds on
  the base.

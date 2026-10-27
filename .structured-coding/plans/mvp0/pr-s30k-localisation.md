# PR S30-k — Localisation of attributes: label packs, zh_Hans, and the reader's choice of language

## DRAFT — not frozen

```text
Design revision:        revision 1 (2026-10-26), DRAFT. Not frozen. No implementation before an operator ruling on
                        §15 and a DESIGN FROZEN header written by the primary session on that ruling.
Author:                 S30-k PR-design session (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s30k,
                        branch docs/pr-design-s30k, planning base origin/main @ 0b2faf2f (#382)
Proposed split:         two PRs from this one design (Q-S30k-4): S30-k1 the mechanism (§9.1), S30-k2 the zh_Hans
                        data and the checkpoint (§9.2). One design, one ledger (§16), two branches.
Start condition:        S30-f merged (met: #373); this design frozen; §15 ruled; the primary's dispatch.
                        S30-k2 starts when S30-k1 has merged; its translation drafting may start at freeze.
Implementation base:    main (origin/main at C0 / D0); C0 and D0 record the commit
Execution contract:     §13
Implementer:            S30-k1: Opus-class session; C5, C6 and C7 may go to Haiku-class sub-agents from this plan,
                        each reviewed by the session. S30-k2: Haiku-class drafting from §5.11's style sheet; an
                        Opus review pass; the operator's spot check (Q-S30k-5)
Lifecycle:              DRAFT 2026-10-26
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15;
amended) §§5.6, 6.6 (M-4), 9.1–9.3, 11.2, 12, 13.1 (the S30-k row and the note of 2026-10-15 after the table), 13.3
(release gate) · **Specified by:** `docs/MODULE_SPEC.md` §2 (the `labels/` row), §4.1 *Rendering* and *Disclosure*;
`docs/DECISIONS.md` `ARC-70`, `ARC-121`, `ARC-122` · **Builds on:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md), [`pr-s30e-builtin-catalogue.md`](pr-s30e-builtin-catalogue.md),
[`pr-s30f-attribute-consumers.md`](pr-s30f-attribute-consumers.md) (all MERGED; S30-f #373);
[`step-20-client-settings.md`](step-20-client-settings.md) (the language setting, MERGED) · **Related draft:**
`pr-pdn-place-display-names.md` (branch `origin/design-place-display-names`, DRAFT 2026-10-26) · **Parent:**
[`overall.md`](overall.md) · **Binding parents:** `CLAUDE.md` §§1.1, 2, 4 (items 2, 3, 5, 7, 8, 10, 11, 12, 13, 16);
`/Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md` (speed first; no world-specific hard-coding; every
acceptance names a second world); step-20 INV-SET-6, INV-SET-9, INV-SET-10; step-30 I-S30-3, I-S30-8, I-S30-10.

**Decision ids.** One new record, `ARC-129` (§5.12), from the S30 range. Dated notes under `ARC-70`, `ARC-121` and
`ARC-122`. No dependency is adopted, so no `DEP` id is needed (Q-S30k-7). `ARC-127`, `ARC-128` and `DEP-69` stay unused.

No model is called. No world digest moves. No reference world enables `attributes` today, so no reference world's
bytes move either.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-k row).** "label packs `labels/<locale>.yaml` (labels, phrases, words); zh-Hans complete
  for the built-in catalogue and reviewed; locale selection in cognition, MCP and both panels; fallback to English."
- **Where the files live (the note of 2026-10-15 after §13.1).** "The seam refuses a subdirectory inside a data
  directory (S30-b §5.6 step 2), so locale labels are `labels/<locale>.yaml`, a directory `attributes` owns
  (`MODULE_SPEC.md` §2's table)."
- **Integration checkpoint (same row).** "zh-Hans 2D and 3D panels and a zh-Hans 'who I am' render every attribute of a
  fixture person."
- **Adversarial criterion (same row).** "a missing translation falls back to English and is listed by
  `attributes check --locale zh-Hans`."
- **What S30-f required of this PR (R-S30k-f-1).** "Localisation replaces `label` and `text` server-side; the
  composition separator `", "` and the cut marker ` …` become locale words of the label pack; the three
  implementations read them from the block, not from code."
- **S30-f's scope note (§2.2).** "texts are server-rendered, so S30-k swaps words, not code."
- **The step's rendering rule (§5.6).** "The renderer lives in the model crate … It produces English. Localisation is
  S30-k … label packs keyed by attribute and value, zh-Hans first. A backend may ship its own (the MatrAIx pack carries
  its six, M-4)."
- **`MODULE_SPEC.md` §4.1 *Rendering*.** "locale label packs (`labels/<locale>.yaml`, keyed by attribute and value,
  S30-k) replace label, phrase and words, falling back to English."
- **The coordinator's brief (2026-10-26).**
  - B-1. Address who selects the locale: seat or client preference, or world configuration.
  - B-2. Keep render concepts out of simulation contracts.
  - B-3. **A locale must not change any world digest.**
  - B-4. No world-specific hard-coding; the acceptance names a second world.
  - B-5. Compare mature i18n options (Fluent, ICU MessageFormat, gettext, plain YAML) and Godot's own translation
    support, and record the choice.
  - B-6. Consider a split into k1 (mechanism) and k2 (zh-Hans data), and recommend.
- **Release gate (step-30 §13.3).** The public release does not ship until S30-a … k have merged.
- **R-LOAD-1.** Locally only the targeted test files of a commit; never `cargo test --workspace`; CI on the exact head
  is the gate.

---

## 1. Goal

A player who reads Chinese sees every attribute of the people around them in Chinese, in both clients. An agent and an
NPC's language model can ask for the same. English stays the default and stays byte-for-byte as it is.

In one sentence: **the server renders each attribute in English and in every language the world's label packs offer;
each reader picks its own language from that; nothing about languages ever enters world state.**

1. **Label packs as data.** `labels/<locale>.yaml` in any pack or world gives, per attribute, a label, a phrase, words
   per value, band words and a flag word, plus a few words for joining lists. No code knows any language but English.
2. **One renderer.** The Rust renderer that already makes English text also makes each offered locale's text, by the
   same rules. Python and GDScript still render nothing (S30-f F-S30f-8).
3. **The reader chooses.** The 2D and 3D clients use the language setting the player already has (step-20). The LM
   controller has a `locale` setting per seat. The MCP adapter has a `--locale` option. The server does not know any
   reader's language.
4. **Fallback.** An entry with any missing piece in a locale is shown in English, whole. `attributes check --locale`
   lists every missing piece.
5. **zh_Hans for the whole built-in catalogue** (93 attributes), reviewed.

---

## 2. Scope

### 2.1 In scope

**S30-k1 — the mechanism**

1. **The model crate** (`authoring/attributes/`): a `Locale` type; the label file format and its decoder for the
   `labels/` directory; the merged `Labels` of a composition; localised rendering in `render.rs`; the disclosed block's
   schema 3 (§5.5); the Rust reader `disclosed.rs` reads schema 3.
2. **The data context** (`authoring/attributes/src/context.rs`): `AttributesContext` carries `Labels`. Labels are
   **never** put into `attributes-composed` or any other fact.
3. **The provider seam** (`sdk/rust/src/pack.rs`, `sdk/rust/src/installed.rs`, `worldpack/src/load.rs`): a pack's
   perception provider may be built with read access to the composition's owner contexts (§5.7, Q-S30k-2).
4. **`attributes` and the owners** (`systems/attributes/src/system.rs`, `authoring/attributes/src/owner.rs`,
   `systems/employment/src/system.rs`; and `systems/notices` if S31-b2 has made it an owner by C3): the provider holds
   the labels and passes them to the renderer.
5. **The CLI** (`tools/cli/src/attributes.rs`, `tools/cli/src/main.rs`): `attributes check --locale <locale>` lists what
   is missing; `attributes show --locale <locale>` prints the rendering in that locale.
6. **The Python reader** (`sdk/python/src/mineworld_sdk/attributes.py`): schema 3; `compose`, `inspect` and `sheet`
   take a locale.
7. **Cognition** (`cognition/lm-controller/src/mineworld_cognition/{config.py, controller/attributes.py}` and the callers
   that build `SelfAttributes` and descriptions): `[controller] locale` and `[seat_overrides.<key>.controller] locale`.
8. **The MCP adapter** (`clients/agent/src/mineworld_agent/{__main__.py, disclosed.py}` and where the option is
   threaded): `--locale`.
9. **Both clients** (`clients/protocol/mineworld/attributes.gd`; `clients/2d/scripts/menu.gd`,
   `clients/2d/scripts/hud/readers.gd`; `clients/3d-spike/scripts/slice/{slice_main.gd, inspect_panel.gd}`): select by
   `MineWorldText.language()`; re-compose on a language change.
10. **Fixtures and tests**: the S30-f fixture world gains a world label pack; a second fixture world with its own
    attribute and a test-only `fr` pack (§5.10); the composition golden gains locale cases.
11. **Specification sync**: `MODULE_SPEC` §2 and §4.1; `ARC-129`; dated notes under `ARC-70`, `ARC-121`, `ARC-122`;
    the SDK docs of `provider_in`.

**S30-k2 — the zh_Hans data and the checkpoint**

12. `entities/person-attributes/labels/zh_Hans.yaml`: all 93 attributes, complete (§5.11); `pack.yaml` version
    `1.2.0`.
13. The review record (§5.11) and the glyph check of the pack's characters against the bundled CJK font.
14. The checkpoint scenario in zh_Hans (§8 AC-k-1).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The LM prompt's own frame (headers, rules, "People here") in another language | a later cognition design (Q-S30k-6) |
| Translating authored free text (an occupation title, a persona summary) or names | never: world content is shown as written (step-20 INV-SET-10) |
| Place display names and their forms | PDN (its own design; §11 R-PDN-k-1) |
| MatrAIx's six locale packs | S30-h (§11 R-S30h-k-1) |
| A host option that limits which locales are rendered | only if S30-h's measurement needs it (§11 R-S30h-k-1) |
| Plural forms and locale number formats | not needed for zh_Hans; revisit in `ARC-129` when a locale needs them |
| Any change under `kernel/`, `contracts/` or `server/` | never |
| Any reference world edit | never; S30-g adopts attributes |

### 2.3 Dependencies

- **Merged:** S30-c (#280), S30-d (#293), S30-e (#300), S30-f (#373), step-20 SET-a (the language setting and
  `MineWorldText`).
- **Concurrent, not required:** PDN (draft). If PDN freezes first, the two share one `Locale` type (§5.1, R-PDN-k-1).
  S31-b2 (draft) may make `notices` an owner; C3 updates every caller of `disclose_owned` that exists at C0.

---

## 3. Audit (`origin/main @ 0b2faf2f`, 2026-10-26)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-k-1 | The renderer makes English only: `words` (slug default with hyphens as spaces), `phrase` (`replacen("{value}", words, 1)`), a set joined as "a, b and c", a true flag as the label in lower case, an instant as nothing, a band as its words, every text cut at 160 B with "…" | `authoring/attributes/src/render.rs:1-200` | one place to add locales; English rules stay as they are |
| A-k-2 | A disclosed entry is `{ attribute, kind, label, facets, rank?, value?, text? }`, `deny_unknown_fields`; a block is `{ schema: 2, entries }` | `render.rs:55-104` | new fields need schema 3 (§5.5) |
| A-k-3 | The Python, GDScript and Rust readers read schema 2 only and skip any other block whole | `sdk/python/src/mineworld_sdk/attributes.py:1-40, 170-200`; `clients/protocol/mineworld/attributes.gd:17-31` | all three readers move to schema 3 in one PR |
| A-k-4 | The consumers hard-code `SEPARATOR = ", "`, `CUT_MARK = " …"`; the GDScript sheet hard-codes `": "`, `"yes"`, `"no"` | `attributes.py:41-46`; `attributes.gd:26-28, 144-175` | these become block words per locale (R-S30k-f-1) |
| A-k-5 | `attributes-composed` states the catalogue's `Projection`: every declaration with its English label, phrase and words. The running catalogue is rebuilt from it | `authoring/attributes/src/facts.rs:29-45, 156-163`; `catalogue.rs:280-296` | English text is already world state. Labels must stay out of it, or every label edit moves a digest (B-3) |
| A-k-6 | Disclosure runs in the provider's `discloses(world, observer, subject)`; the provider is the system type built by `Default`; the SDK says "a system holds no fields" | `systems/attributes/src/system.rs:205-245`; `systems/presence/src/interaction.rs:80-130`; `sdk/rust/src/installed.rs:470-488` | a provider cannot reach label data today (F-S30k-1) |
| A-k-7 | Owners disclose through `disclose_owned::<O>(world, observer, subject)`; the only owner today is `employment` | `authoring/attributes/src/owner.rs:31-45`; `systems/employment/src/system.rs:441-460` | owners need the labels too |
| A-k-8 | Providers are built in `WorldPack::providers()` by `Capability::provider(*capability)`; the pack holds every enabled owner's context, `pack.context(owner)`; a resumed world is "composed from the pack", so the contexts exist on resume too | `worldpack/src/load.rs:329-334`; `worldpack/src/read.rs:425-432`; `tools/cli/src/serve.rs:490-512` | a provider can be given the contexts with no server change (§5.7) |
| A-k-9 | The `labels/` directory is already listed as owned by `attributes`, "per-locale labels, phrases and words (S30-k)" | `docs/MODULE_SPEC.md:122, 1378, 1505-1506` | the location is specified; the format is not |
| A-k-10 | Client UI text uses gettext `.po` through Godot's `TranslationServer`, layered shared → pack → user; locale spelled `zh_Hans`; world content never passes through `tr()` | `clients/shared/settings/text.gd`; `docs/decisions/ARC-70.md` and its note; step-20 INV-SET-10 | client catalogs are not the place for world text (F-S30k-3) |
| A-k-11 | The client language setting is `general/language`, standardized by Godot (`zh-Hans` → `zh_Hans`), default `en`; `MineWorldText.language()` returns it | `clients/shared/settings/settings.gd:41-146`; `text.gd` `set_language`, `language` | both panels already know the player's language |
| A-k-12 | S35 N-e words need levels in the client by `word.<w>` keys in the shared catalog | `pr-s35-ne-presentation.md` §4.2; `clients/shared/settings/locale/en.po:905-930` | a precedent for a small closed vocabulary; it does not scale to world-declared attributes (F-S30k-3) |
| A-k-13 | PDN (draft) discloses a place's name with all authored forms `{ name, forms: { Locale: Name } }`; a client picks a form by its language; `Locale` is the `.po` stem (`zh_Hans`) | `origin/design-place-display-names:.structured-coding/plans/mvp0/pr-pdn-place-display-names.md` §§3–4 | the same model as this design: the server sends forms, the reader selects (D-k-1) |
| A-k-14 | The built-in catalogue: 93 attributes (27 category, 51 level, 7 set, 5 quantity, 1 flag, 1 instant, 1 text); 513 value slugs across categories, levels and sets (88 of them from five vocabularies); 127 explicit English words; 3 band words; 75 distinct phrases | `entities/person-attributes/attributes/*.yaml`, `vocabularies/*.yaml`; counted by a scratch script on this base | the size of the zh_Hans pack (S30-k2) |
| A-k-15 | `core.children`'s phrase is `"{value} children"`, so English says "1 children" | `entities/person-attributes/attributes/background.yaml` | an English plural defect, pre-existing; not this PR's (F-S30k-6) |
| A-k-16 | The LM controller's config has `[controller]` and per-seat `[seat_overrides.<key>.controller]` tables with typed fields | `cognition/lm-controller/src/mineworld_cognition/config.py:299-370` | `locale` fits there as one more field |
| A-k-17 | Cognition composes "who I am", "how I speak" and "who I see" from `mineworld_sdk.attributes.compose`; the prompt frame (`What the world says about you:`, `Your manner:`) is English | `controller/attributes.py:19-58`; `controller/prompt.py:140-142` | the locale is one argument to compose; the frame stays English (Q-S30k-6) |
| A-k-18 | The MCP adapter composes the description and the own sheet in `disclosed.py`; its options are argparse flags | `clients/agent/src/mineworld_agent/disclosed.py:540-590`; `__main__.py:49-115` | `--locale` beside the other flags |
| A-k-19 | 2D: the subject menu shows the glance and the inspect list; `readers.gd` shows the own sheet. 3D: the HUD glance line and `inspect_panel.gd` | `clients/2d/scripts/menu.gd:147-148`; `readers.gd:225, 296`; `clients/3d-spike/scripts/slice/slice_main.gd:712`; `inspect_panel.gd:66-67` | four call sites take a locale |
| A-k-20 | The protocol module is a symlink in both clients, so one GDScript file serves both | `ls -la clients/2d/mineworld clients/3d-spike/mineworld` | one change, two clients |
| A-k-21 | `attributes check` validates the catalogue and prints counts; `attributes show` prints holder and observer renderings | `tools/cli/src/attributes.rs:262-402` | `--locale` extends both |
| A-k-22 | One hand-written golden, `authoring/attributes/tests/fixtures/composition.json`, is read by the Rust, Python and GDScript tests | `disclosed/tests.rs:8`; `clients/agent/tests/test_view.py:186`; `clients/protocol/checks/attributes_check.gd:14` | locale cases join that golden |
| A-k-23 | No reference world enables `attributes` | `worlds/*/world.yaml` (no `attributes`) | no reference world's observation changes in this PR |
| A-k-24 | MatrAIx ships six locale label packs, zh-Hans among them | step-30 §3.2 M-4 | S30-h may offer up to seven languages; payload size scales with them (R-k-2) |

### 3.2 Findings

- **F-S30k-1. A provider cannot reach data that is not world state.** Labels must not be world state (B-3, A-k-5), but
  the provider that renders sees only `WorldRead` and is built by `Default` (A-k-6). Something must hand the labels to
  the provider. §5.7 gives the smallest generic seam: the provider is built with read access to the composition's
  owner contexts, which the pack already holds, also on resume (A-k-8). Q-S30k-2.
- **F-S30k-2. Who selects the locale.** Three places could: the world's configuration, the server per seat, or each
  reader. A world configuration key would be stated as a configuration fact, so choosing a language would move a
  digest (B-3). Per-seat rendering needs a protocol field, a locale inside the perception seam, and a server change, and
  it makes an observation depend on who reads it. Reader selection needs neither (D-k-1, Q-S30k-1).
- **F-S30k-3. Client catalogs cannot carry attribute words.** Client `.po` catalogs are UI text. Attribute words are
  world text that a world can declare (`freckles`, a world's own attribute). Putting them in client catalogs breaks
  INV-SET-10, needs a client release per world attribute, and gives nothing to cognition or MCP. The S35 `word.<w>`
  keys (A-k-12) work for one System's fixed list, not for an open catalogue.
- **F-S30k-4. Partial translations would mix languages inside one phrase.** If a phrase is translated but one word is
  not, the text would read "auburn头发". Decided (D-k-2): an entry is localised whole or not at all; otherwise the
  reader shows the English entry.
- **F-S30k-5. English is the declaration.** An `en` label pack would compete with `overrides.yaml`. Decided (D-k-3): a
  `labels/en.yaml` is refused, naming overrides as the way to change English.
- **F-S30k-6. "1 children".** The English phrase has no plural form (A-k-15). Chinese needs none. This PR keeps the
  English rules unchanged (I-k-2) and records the defect for S30-e's owner. Plural support is the revisit trigger of
  `ARC-129`.
- **F-S30k-7. Authored text stays as written.** An occupation title or a persona summary is world content. Its phrase
  is localised ("职业是{value}"); the value is not ("职业是barista"). This matches INV-SET-10 and PDN's rule.
- **F-S30k-8. The own sheet's punctuation and "yes"/"no" are code (A-k-4).** They become block words per locale, so the
  sheet reads "眼镜：否" and not "眼镜: no".

### 3.3 What does not exist

No label file, no locale type in Rust, no locale field in any block, no locale option in cognition or MCP, and no
check that lists missing translations.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

**The question.** We need to turn (attribute, value) into a phrase in another language. We already have a renderer and
an English data model (label, phrase with one `{value}`, words per value). Which tool should hold the translations?

Because every text is rendered on the server (S30-f F-S30f-8), only a **Rust** library would ever be used. Python and
GDScript only pick strings.

| Option | What it gives | Fit here | Verdict |
| --- | --- | --- | --- |
| **Fluent** (`fluent-bundle`, Mozilla, Apache-2.0 / MIT, pure Rust) | message files (`.ftl`), selectors, CLDR plurals, number formats | the English source model is frozen YAML with one placeholder (S30-a/c/e). Fluent would hold translations in a second model; flattened ids like `core-hair-colour-word-dark-brown`; English would stay outside it. Its strength (plurals) is not needed by zh_Hans | **declined now**; first candidate when a plural-sensitive locale arrives (revisit in `ARC-129`) |
| **ICU MessageFormat** | the most expressive format | no stable ICU4X MessageFormat crate was found at this audit; `rust_icu` binds the C library, a native dependency on three platforms | **declined** |
| **gettext `.po`** (the repo's format for client UI, `ARC-70`) | the best translator tools (Poedit, Weblate, `msgmerge`); a changed English `msgid` shows a stale translation | the S30-b seam reads only `.yaml` data files (`MODULE_SPEC` §2), and step-30's note fixes `labels/<locale>.yaml`. A `.po` reader in Rust (`polib`) would be a new dependency for one directory | **declined for label packs**; its stale-detection idea is kept as R-k-4's follow-up |
| **Godot `TranslationServer`** (already used by both clients) | live switch, `.po` loading | client-only. Cognition and MCP cannot use it. World text must not pass through `tr()` (INV-SET-10). A remote client does not hold the world's packs | **declined** for attribute text; still used for UI text |
| **Plain YAML label packs** (our renderer) | the same fields as the English declaration, the same placeholder rule, the same loader, the same line-and-column refusals | fits the seam and the frozen model; no new dependency; about 150 lines of decoding and 60 of rendering | **chosen** |

**Are we forcing a wheel?** No. Nothing here adopts a framework's execution model.
**Are we reinventing a wheel?** Only the smallest piece: substituting translated words into translated phrases. That
is the renderer we already have, run with other words. A full i18n engine would add a second message model beside a
frozen one. `ARC-129` records this comparison and its revisit triggers.

---

## 5. Design

### 5.1 `Locale` (model crate; Q-S30k-3)

- A named type, `Locale`, a newtype over a validated string.
- **Canonical form: `zh_Hans`**, the same spelling as the client catalogs (`clients/shared/settings/locale/zh_Hans.po`)
  and PDN. Grammar: a language of 2–3 lower-case letters, then optionally `_` and a 4-letter script in title case, then
  optionally `_` and a 2-letter region in upper case or 3 digits (`en`, `zh_Hans`, `pt_BR`).
- **Input**: `zh-Hans` is accepted wherever a person types a locale (CLI, cognition config, MCP option) and normalised
  (`-` to `_`). A label file's name must be the canonical form.
- **`en` is special**: it means the declarations themselves. It is never a label file (D-k-3).
- **Where it lives:** in `mineworld-authoring` (`authoring/src/`), so PDN and this PR share one type. If PDN lands
  first with its own type, C1 reuses it (bounded, recorded).

### 5.2 The label file (`labels/<locale>.yaml`)

```yaml
# entities/person-attributes/labels/zh_Hans.yaml (excerpt; S30-k2 writes the whole file)
format: 1
locale: zh_Hans                 # must equal the file name's stem
words:                          # the locale's joining words; every one is required in a file
  list: "、"                    # between set members, except before the last
  and: "和"                     # before the last set member
  join: "，"                    # between texts of a composition (English: ", ")
  more: "……"                    # ends a composition cut at an entry boundary (English: " …")
  pair: "："                    # between a label and its text in the own sheet (English: ": ")
  "yes": "是"                   # a true flag with no text in the own sheet (English: "yes")
  "no": "否"
labels:
  core.hair-colour:
    label: 发色
    phrase: "{value}头发"
    words: { black: 黑色, dark-brown: 深棕色, brown: 棕色, light-brown: 浅棕色, blond: 金色, auburn: 赤褐色,
             red: 红色, grey: 灰白, silver-grey: 银灰色, white: 白色 }
  core.stature:
    label: 身高
    phrase: "身高{value}米"
    perceived_as: [个子矮, 中等身高, 个子高]   # one per band, in the declaration's band order
  core.glasses:
    label: 眼镜
    flag: 戴眼镜                 # the words of a true flag (English: the label in lower case)
```

Rules (each refusal names the file, line and column, as every attribute refusal does):

| Rule | Refusal |
| --- | --- |
| `format: 1`; `locale` equals the file's stem and is a valid `Locale`; the stem is not `en` | `BadLabelFile`; `EnglishIsTheDeclaration` for `en` (names `overrides.yaml`) |
| `words:` has exactly the keys `list`, `and`, `join`, `more`, `pair`, `yes`, `no` when present; a file may omit `words:` only if another file of the same locale in the composition gives them | `BadLabelFile` naming the missing or unknown key |
| a key of `labels:` is a qualified id that the composition declares, or that its mask or backend removed | `UnknownAttribute` with the nearest ids. A removed id is accepted and ignored, so a mask never breaks a label pack |
| `phrase` has exactly one `{value}` and no other `{…}` | `BadLabel` (the English rule, `ARC-122`) |
| `words` keys are value slugs of the attribute's domain (its list or its vocabulary) | `OutOfDomain` with the nearest slugs |
| `words` only on `category`, `level`, `set`; `flag` only on `flag`; `perceived_as` only where the declaration has bands, with exactly as many entries | `BadLabel` naming the field and the kind |
| every string is non-empty, at most 160 bytes, with no control character | `BadLabel` |
| unknown fields | refused (`deny_unknown_fields`) |

**Merging the files of one composition.** Files are read in the seam's collection order: shipped, then Entity Packs by
id, then the world (S30-b I-b-7). Merging is per field (`label`, `phrase`, `flag`, each word, the band list, each
joining word).

- The world's files win over every pack's. That is how a world fixes or changes a translation.
- Two packs that are not the world giving the same field for the same locale are refused (`DuplicateLabel`, naming
  both files). There is no silent "last pack wins".
- Inside one pack, the same field twice is refused the same way.

### 5.3 `Labels` and the data context

- `Labels` is the merged result: `BTreeMap<Locale, LocaleLabels>`, where `LocaleLabels` holds the joining words and a
  `BTreeMap<QualifiedId, AttributeLabels>`.
- `attributes`' `OwnsData` impl decodes the `labels/` directory beside `attributes/`, `vocabularies/` and the rest
  (`data_context`). The decoded labels are checked against the built catalogue (§5.2's domain rules) after it is
  built.
- `AttributesContext` gains `labels: Arc<Labels>`.
- **Labels never enter a fact.** `AttributesComposed`, its `Projection`, `CompositionIdentity` and `PopulationIdentity`
  are unchanged. A world with or without any label file states byte-identical genesis facts (I-k-1, X-k-1).
- **Offered locales** are the keys of `Labels`, in sorted order. A world that ships no label file offers English only.

### 5.4 Localised rendering (`render.rs`)

The English path is untouched (I-k-2). For each offered locale, `render::localised(declaration, value_or_band, view,
locale_labels) -> Option<Localised>` renders by the same rules with that locale's pieces:

| Kind | English (unchanged) | In a locale | Missing piece → |
| --- | --- | --- | --- |
| `category`, `level` | the value's word, default slug with spaces | the locale's word for the slug; **no slug default** | English entry |
| `quantity` | the number at its scale, ASCII digits | the same number (locale-neutral digits), in the locale's phrase | English entry if the phrase is missing |
| `flag` true | the label in lower case | the locale's `flag` | English entry |
| `flag` false, an empty set, an `instant` | no text | no text | — |
| `set` | words joined "a, b and c" | the locale's words joined with its `list` and `and` ("阅读、园艺和钓鱼") | English entry if any member's word is missing |
| `text` | the authored text | the authored text, as written, in the locale's phrase (F-S30k-7) | English entry if the phrase is missing |
| a `perceived_as` band | the band's words | the locale's band words | English entry |
| bound | 160 B, cut with "…" | the same bound and mark | — |

- **All or nothing per entry (D-k-2).** If the label, the phrase or any word an entry needs is missing in a locale, that
  entry carries nothing for that locale. The reader then shows the English label and text for that entry.
- **The band is not disclosed.** The renderer knows which band applies from the value it already holds; the wire still
  carries no value for a banded observer view.

### 5.5 The disclosed block, schema 3 (Q-S30k-1)

```json
{ "schema": 3,
  "words": { "zh_Hans": { "join": "，", "more": "……", "pair": "：", "yes": "是", "no": "否" } },
  "entries": [
    { "attribute": "core.age-band", "kind": "level", "label": "Age", "facets": ["casting", "observed"],
      "rank": 10, "text": "in their sixties", "texts": { "zh_Hans": "六十多岁" } },
    { "attribute": "core.hair-colour", "kind": "category", "label": "Hair colour", "facets": ["casting", "observed"],
      "rank": 50, "value": { "category": "grey" }, "text": "grey hair",
      "labels": { "zh_Hans": "发色" }, "texts": { "zh_Hans": "灰白头发" } } ] }
```

- **Fields added.**
  - Per entry: `texts` (locale → text) and `labels` (locale → label). Both are omitted when empty.
  - `labels` is sent **in the holder's view only**. Another observer's readers never show a label: the glance and the
    inspect list are texts only (S30-f D-f-2). This keeps the observer view small.
  - Per block: `words` (locale → the consumer's joining words `join`, `more`, `pair`, `yes`, `no`), omitted when no
    locale is offered. `list` and `and` stay on the server, because only the renderer joins set members.
- **Unchanged.** `schema` aside, every schema-2 field keeps its meaning and its English value. Entry order is rank
  order. The holder and observer filters are S30-c's.
- **Always schema 3.** The server sends schema 3 even when no locale is offered, so there is one shape. There is no
  schema-2 compatibility (`CLAUDE.md` §4 rule 12; nothing is released yet). All three readers move in this PR.
- **Size.** A zh_Hans text is about 30 bytes of JSON per observer-view entry. C2 measures Bob's view of Alice and
  Alice's own block on the 93-attribute fixture, with and without zh_Hans, and records them in §16 (AC-k-9). If Bob's
  view of one person exceeds **2 048 B** with zh_Hans offered, that is a material stop with the numbers (R-k-2).

### 5.6 Readers select (D-k-1)

The same rule in Rust, Python and GDScript, held equal by the composition golden (A-k-22):

1. **Pick the locale.** Given the reader's wanted locale `L` and the block's offered locales (the keys of `words`):
   use `L` if offered; else the one offered locale with the same language part, if exactly one has it (`zh_CN` →
   `zh_Hans`); else English. This is PDN's `shown_name` rule.
2. **Per entry:** the text is `texts[L]` if present, else `text`. The label is `labels[L]` if present, else `label`.
3. **Per composition:** `join`, `more`, `pair`, `yes`, `no` come from `words[L]` when `L` is not English; otherwise the
   English constants of S30-f. Cutting still counts UTF-8 bytes.
4. With `L = en`, every output is byte-identical to S30-f's (I-k-3).

When blocks merge (`traits` plus an owner's `attributes` member), they carry the same `words`, because one server
rendered them from one `Labels`. A reader takes the first block's.

**Who sets `L`:**

| Reader | Where `L` comes from | Default |
| --- | --- | --- |
| 2D and 3D clients | the player's language setting, `MineWorldText.language()` (step-20); re-composed on `NOTIFICATION_TRANSLATION_CHANGED` and on the next draw | `en` |
| LM controller | `[controller] locale`, overridable per seat in `[seat_overrides.<key>.controller] locale` | `en` |
| MCP adapter | `--locale` | `en` |
| `mineworld attributes show` | `--locale` | `en` |

The world never chooses `L`. The server never learns `L`. So an observation is the same whoever reads it.

### 5.7 The provider seam (`sdk/rust`, `worldpack`; Q-S30k-2)

```rust
// sdk/rust/src/pack.rs — SystemPack gains one default method
/// The value that answers perception for this pack. It may keep presentation data taken from the composition's
/// owner contexts. That data is immutable, is never world state, and must not change any fact.
fn provider_in(contexts: &ProviderContexts<'_>) -> Self where Self: Default { let _ = contexts; Self::default() }

// mineworld-authoring — a read-only view of the composition's owner contexts
pub struct ProviderContexts<'a> { /* owner SystemId → &dyn OwnerContext */ }
impl ProviderContexts<'_> { pub fn of(&self, owner: &SystemId) -> Option<&dyn OwnerContext>; }
```

- `installed!`'s `Capability::provider(self, contexts)` calls `<System as SystemPack>::provider_in(contexts)` instead of
  `Default::default()`. The installed system (`World::install`) still uses `Default`.
- `WorldPack::providers()` builds `ProviderContexts` from the contexts it already holds and passes it. This runs on
  creation and on resume (A-k-8).
- The installed-macro comment "a system holds no fields" gains a dated qualifier: a provider may hold immutable
  presentation data from the contexts; it still holds no state.
- **`attributes`**: `AttributesSystem { labels: Option<Arc<Labels>> }`, `Default` gives `None`;
  `provider_in` reads its own context's labels. Its `discloses` passes them to the renderer.
- **Owners**: `disclose_owned::<O>(world, observer, subject, labels: Option<&Labels>)`. An owner's `provider_in` takes
  the labels from `attributes`' context through `Labels::from_contexts(contexts)` (model crate), which returns `None`
  where `attributes` is not enabled. `employment` changes in exactly these two places.
- **Nothing else changes**: no kernel, contract, server or protocol edit. Packs that do not override `provider_in` are
  built as before.

### 5.8 The CLI

- `mineworld attributes check <world> [--locale <locale>]...` — everything it prints today, and for each named locale:
  - one line per missing piece: `missing zh_Hans core.hair-colour words.auburn`, also `label`, `phrase`, `flag`,
    `perceived_as`;
  - one line per locale: `zh_Hans: 93 of 93 attribute(s) complete; 0 missing piece(s)`.
  - A missing piece is not an error (it falls back). The exit code stays 0 unless the catalogue or a label file is
    refused. A locale no file offers prints `zh_Hans: not offered` and exits 0.
- `mineworld attributes show <world> [<person>] --locale <locale>` — prints the holder's and another observer's texts
  in that locale (with English fallbacks marked `(en)`), for review (S30-k2).

### 5.9 Consumers

- **Python** (`mineworld_sdk.attributes`): `SCHEMA = 3`; `AttributeEntry` gains `texts`, `labels`; `Block` gains
  `words`; `compose(entries, facet, budget, locale="en", words=None)`, `inspect(..., locale)`, `sheet(..., locale)`;
  `Disclosed` carries the merged block words. The locale argument is a `Locale` string checked by the §5.1 grammar.
- **Cognition**: `ControllerConfig.locale: str = "en"` and `ControllerOverrides.locale: str | None`; the seat's
  effective locale reaches `SelfAttributes.of` and `description`. The logged `DecisionInput` already holds the composed
  strings, so logs replay unchanged. With `en`, every prompt golden and cassette key is unchanged (I-k-3).
- **MCP adapter**: `--locale` (default `en`), checked at start; `PersonLine.description`, `MeLine.attributes`'
  `label` and `text` use it. `inspect` returns the raw blocks, which already carry every locale. The published tool
  schema does not change.
- **GDScript** (`attributes.gd`): `SCHEMA := 3`; `FIELDS` gains `texts`, `labels`; `compose`, `glance`, `inspect`,
  `sheet` take `locale`. Callers pass `MineWorldText.language()`. The module still names no attribute and evaluates no
  rule (S30-f A-f-20). No attribute text goes through `tr()` (INV-SET-10).
- **Fonts**: the clients already install Noto Sans SC as the fallback for every control (step-20 INV-SET-9). S30-k2's
  glyph check proves every character of the zh_Hans pack has a glyph in it.

### 5.10 The fixtures: one person, two worlds, two test locales

- **World 1:** S30-f's `tools/cli/tests/fixtures/attributes-consumers/` gains `labels/zh_Hans.yaml` (world-level, a
  partial file in S30-k1: enough to render Alice's glance, plus one deliberate gap) and `labels/fr.yaml` (test only:
  three entries). After S30-k2, the world file is reduced to its one world attribute (`attributes-consumers.secret`
  and `freckles`), because the core pack carries the rest.
- **World 2:** `tools/cli/tests/fixtures/labels-elsewhere/` — a different town: its own attribute
  `labels-elsewhere.sea-legs` (a level), a person with no `core` values but `core.hair-colour` and `sea-legs`, and its
  own `labels/zh_Hans.yaml` and `labels/fr.yaml`. It proves that a world adds a language and translates its own
  attribute with files only.
- **Oracles**, hand-written (test rules §25): `attributes-consumers.expected.<locale>.json` and
  `labels-elsewhere.expected.<locale>.json` for `en`, `zh_Hans`, `fr` — glance, inspect list, own sheet, who I am.
  The `en` oracle of world 1 is S30-f's, unchanged.

### 5.11 zh_Hans for the built-in catalogue (S30-k2)

- **The file**: `entities/person-attributes/labels/zh_Hans.yaml`, 93 entries: 93 labels, 93 phrases (`core.birth`'s
  renders no text today, but `check` still expects one, so the file stays complete), 513 value words, 3 band words, 1 flag word, the
  joining words. `pack.yaml` `1.1.0` → `1.2.0`.
- **The style sheet** (written into the file's header and followed by the drafter):
  1. Plain modern Mandarin a player reads at a glance; no literary forms.
  2. An observer's text reads as a description of a person ("个子高", "灰白头发", "戴眼镜"), so a glance joins
     naturally with "，".
  3. Established terms where they exist: the Big Five as 开放性, 尽责性, 外向性, 宜人性, 神经质; the Monk Skin Tone
     levels as plain colour words, never ranked or valued.
  4. Sensitive attributes (ancestry, religion, sexual orientation, disability, income band) in neutral, official
     wording; never a slang or loaded term. These are `private`, but the holder reads them.
  5. Numbers stay ASCII digits; units in Chinese in the phrase (米, 公斤, 岁, 年).
  6. No word repeats the label ("发色：灰白头发" is fine; "灰白发色头发" is not).
- **The review** (Q-S30k-5): an Opus reviewer reads the whole file against the style sheet and the English, and the
  `show --locale zh_Hans` print of the S30-e fixture's people; findings are fixed in the same PR. The operator then
  spot-checks the sensitive attributes and twenty random entries from the same print. The print is kept in §16.
- **Completeness**: `attributes check --locale zh_Hans` on a world requiring `person-attributes` prints
  `zh_Hans: 93 of 93 attribute(s) complete; 0 missing piece(s)` (AC-k-2).

### 5.12 `ARC-129` — Attribute localisation (record, written in C9)

- **Problem.** Attribute text must reach players, agents and NPC prompts in more than one language, without making
  language part of world truth.
- **Decided.**
  1. Label packs are data: `labels/<locale>.yaml`, owned by `attributes`, same fields as the English declaration.
     English is the declaration; there is no `en` label pack.
  2. Labels are presentation data. They are never stated as a fact, never part of a digest, never compared at resume.
  3. The server renders every offered locale with the one Rust renderer; an entry is localised whole or not at all.
  4. Each reader selects its locale; the world never does; the server does not know it.
  5. A provider may hold immutable presentation data from the composition's contexts (the seam of §5.7).
- **Options considered**: §4's table; and F-S30k-2's three places to select a locale.
- **Revisit when**: a locale needs plurals or its own number format (then Fluent first); payload size with many
  locales exceeds the bound (then a host option that limits rendered locales); a translation platform is adopted
  (then `.po` export of label packs).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-k-1 (B-3) | Label files, their content and any reader's locale change no fact and no digest: the same world with and without every `labels/` file states byte-identical genesis facts and the same 300-second digests; a save resumes after a label edit | X-k-1; AC-k-6 |
| I-k-2 | The English rendering is unchanged: every English `label` and `text` equals S30-f's for the same world | AC-k-5; the S30-f oracle unchanged |
| I-k-3 | A reader with locale `en` produces byte-identical output to S30-f's: prompt goldens, cassette keys, agent views, both panels | AC-k-5 |
| I-k-4 | No code names a language other than English: no `zh`, `fr` or CJK character in any `src/`, `scripts/` or `mineworld/` file outside test fixtures and label files | AC-k-8's scan |
| I-k-5 (I-S30-3) | Localisation never widens a view: an entry absent from a view in English is absent in every locale; `labels` only in the holder's view | AC-k-4 |
| I-k-6 | An entry is localised whole or shown in English whole (D-k-2) | the golden's partial case; M-k-3 |
| I-k-7 | No attribute text passes through a client's `tr()` (INV-SET-10) | AC-k-8's scan for `tr(` on attribute module outputs; review |
| I-k-8 (I-S30-8) | `kernel/`, `contracts/`, `server/` gain nothing | `git diff --stat` in C10 |
| I-k-9 (B-4) | Every acceptance runs on two worlds; adding a language is data only | AC-k-3 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy `-D warnings` on `mineworld-authoring`, `mineworld-attributes-model`, `mineworld-attributes`, `mineworld-employment`, `mineworld-sdk`, `mineworld-worldpack`, `mineworld-cli`; `ruff`, `pyright` on `sdk/python`, `cognition/lm-controller`, `clients/agent`; Godot parse through the `godot` layer | per commit; CI `fast` |
| Unit (Rust) | `Locale` grammar and normalising; every §5.2 refusal at its line; merge precedence and `DuplicateLabel`; §5.4's table per kind; all-or-nothing | `authoring/attributes/src/labels/tests.rs`, `render/tests.rs` |
| Golden (three languages) | §5.6's selection and composition per locale, including a partial entry, a CJK cut at a byte budget, `zh_CN` → `zh_Hans`, an unoffered locale → English | `composition.json` read by `disclosed/tests.rs`, `sdk/python/tests/test_attributes.py`, `clients/protocol/checks/attributes_check.gd` |
| Crate (pack) | the provider holds labels; `employment.occupation` localised; no labels without `attributes` | `systems/attributes/tests/pack.rs`; `systems/employment/tests/occupation.rs` |
| Structural | I-k-1's fact equality; I-k-4's scan; I-k-8 | `tests/acceptance/tests/attribute_labels.rs` (new) |
| CLI | `check --locale`, `show --locale` on both worlds | `tools/cli/tests/attributes_labels.rs` (new) |
| Integration (real binary) | the checkpoint per surface and locale against the oracles (§5.10) | `tools/cli/tests/attributes_consumers.rs` (extended); `client_2d_attributes.rs` (extended); the 3D probe; the Python real-server tests that S30-f left for the milestone batch (S30-f D-impl-7) take a locale parameter when they land |
| Gate 1 (real LLM) | NOT REQUIRED: the prompt's attribute text is deterministic and compared byte for byte | — |
| Gate 2 (real lifecycle) | the resume half of I-k-1 runs through the real binary (create, edit a label file, resume) | `attributes_labels.rs` |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | PR |
| --- | --- | --- |
| AC-k-1 | **Checkpoint.** On `attributes-consumers` with the built-in zh_Hans pack: with the 2D client's language set to 简体中文, Bob's menu header (glance) and subject panel (inspect list) for Alice, and Alice's own sheet, equal the zh_Hans oracle; the 3D glance line and inspect panel equal the same oracle lines; Alice's LM controller with `locale = "zh_Hans"` builds "What the world says about you: " followed by her `prompt` texts in Chinese, equal to the oracle. Every attribute Alice holds that has text appears in Chinese on her own sheet; none falls back | k2 |
| AC-k-2 | **Complete.** `attributes check --locale zh-Hans` on a world requiring `person-attributes` prints `zh_Hans: 93 of 93 attribute(s) complete; 0 missing piece(s)` | k2 |
| AC-k-3 | **A second world, data only.** On `labels-elsewhere`, the same surfaces equal its own oracles in `zh_Hans` and `fr`, including its own attribute `sea-legs`; removing its `labels/fr.yaml` makes `check --locale fr` print `fr: not offered` and every surface fall back to English; `git diff` outside the two fixture worlds is empty for both changes | k1 |
| AC-k-4 | **Fallback is listed, and never widens.** In world 1, the one deliberate gap (a missing word) shows that entry in English in every zh_Hans surface and is the one line `missing zh_Hans …` of `check --locale zh_Hans`; Bob's views carry no `labels` and no text of Alice's `private`, `learnable` or `hidden` attributes in any locale | k1 |
| AC-k-5 | **English unchanged.** With locale `en` everywhere: S30-f's `attributes-consumers.expected.json` passes unchanged; `prompt_request.json` and every cassette replay unchanged; `test_view.py`'s goldens unchanged | k1 |
| AC-k-6 | **No digest moves (B-3).** The same world with and without all its `labels/` files states byte-identical genesis facts and identical `facts-300s`, `journal-300s` and `sealed-300s`; a save created, then resumed after a label file edit, resumes without drift and serves the new words | k1 |
| AC-k-7 | **Refusals.** Each §5.2 rule refuses a planted file at its line and column, with the named refusal; two Entity Packs labelling one field is `DuplicateLabel`; a world label overrides a pack label | k1 |
| AC-k-8 | **Structural.** No code file outside label files and test fixtures holds a CJK character, `zh_Hans`, `zh-Hans` or `fr` as a locale literal (I-k-4); no attribute module output goes through `tr(`; `kernel/`, `contracts/`, `server/` unchanged | k1 |
| AC-k-9 | **Size recorded.** Bob's view of Alice and Alice's own block, on the 93-attribute fixture, in schema 2, schema 3 without a locale, and schema 3 with zh_Hans, in §16; under 2 048 B for Bob's view with zh_Hans, or a material stop | k1 (measured again in k2 with the full pack) |
| AC-k-10 | **Glyphs.** Every character of `labels/zh_Hans.yaml` has a glyph in the bundled Noto Sans SC | k2 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately. `[x]` needs
the work and recorded evidence. `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from the PR's own worktree. Never `cd` elsewhere.

1. Re-read this commit's section and every file before editing it.
2. Make the steps in order; nothing of a later commit.
3. Run the commit's validation one test file at a time; record command, count, wall time and PASS / FAIL /
   INCONCLUSIVE. `cargo clippy -p <touched crates> --all-targets -- -D warnings`; `cargo fmt --all --check`;
   `uv run ruff check <pkg>`; `uv run pyright <pkg>`. Heavy jobs through
   `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`. **R-LOAD-1:** never `cargo test --workspace`.
4. Mutations: copy the file, plant, run the named test red, restore from the copy, `cmp`, run green.
5. Answer the review questions in §16 in one or two sentences each.
6. `git status --short`: every changed path is in the commit's **Files**.
7. Update §16 and the handoff in the same commit; stage by path; commit with `git commit -F <file>` and the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`, no `curl`.

### 9.1 S30-k1 — the mechanism (branch `mvp0/pr-s30k1`)

#### C0 — Base and ledger

- **Files:** this file (§16); `.structured-coding/plans/mvp0/handoff-s30k.md` (new).
- [ ] Implementation: record the base; re-read A-k-1 … A-k-24 and record every moved line; record whether PDN or
  S31-b2 merged, and every caller of `disclose_owned`.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Review: §15's rulings are applied to §§5, 8 before C1.
- **Commit:** `docs(plan): S30-k1 — open the ledger at the base`

#### C1 — `Locale`, the label file, `Labels` in the data context

- **Goal:** decode and check label packs; keep them out of every fact.
- **Files:** `authoring/src/locale.rs` (new; or PDN's, if merged), `authoring/src/lib.rs`;
  `authoring/attributes/src/{labels.rs (new), labels/tests.rs (new), context.rs, catalogue.rs (the data-file
  dispatch only), lib.rs}`; `tests/acceptance/tests/attribute_labels.rs` (new, the fact-equality half).
- **Steps:**
  - [ ] `Locale` per §5.1, with parse and normalise.
  - [ ] The file format and refusals per §5.2; the merge per §5.2 "Merging".
  - [ ] `AttributesContext.labels`; domain checks after the catalogue is built.
  - [ ] No change to `AttributesComposed`, `Projection` or the identities.
- **Validation:** `cargo test -p mineworld-authoring --lib`; `-p mineworld-attributes-model --lib`;
  `-p mineworld-acceptance --test attribute_labels`. Negative cases: every §5.2 refusal; `labels/en.yaml`; a removed id
  accepted. Mutation **M-k-1** (labels added to `Projection`) → `attribute_labels` red, naming the fact.
- **Acceptance:** AC-k-7; AC-k-6's fact half.
- **Edge cases:** a label file in a world without `attributes` is refused like any owned directory of a disabled pack
  (`DataNeedsASystem`, S30-b).
- [ ] Implementation. [ ] Validation. [ ] Review: no label reaches a fact; refusals keep line and column; merge order
  is the seam's collection order.
- **Commit:** `feat(attributes-model): label packs per locale, decoded and checked, never world state (ARC-129, S30-k)`

#### C2 — Localised rendering and schema 3

- **Goal:** the renderer makes every offered locale; the block carries them.
- **Files:** `authoring/attributes/src/{render.rs, render/tests.rs, disclosed.rs, disclosed/tests.rs}`;
  `authoring/attributes/tests/fixtures/composition.json` (locale cases, hand-written); `systems/attributes/tests/pack.rs`
  and `tools/cli/tests/attributes.rs` (schema number in expected frames, a reviewed edit).
- **Steps:**
  - [ ] `render::localised` per §5.4; `entry` and `disclose` take `Option<&Labels>`.
  - [ ] Schema 3 per §5.5 (`texts`, holder-only `labels`, block `words`).
  - [ ] `disclosed.rs` reads schema 3 and §5.6's selection (Rust reader half of the golden).
  - [ ] Measure AC-k-9 (schema 2 vs 3, with and without a fixture zh_Hans) and record in §16.
- **Validation:** `cargo test -p mineworld-attributes-model --lib`; `-p mineworld-attributes --test pack`;
  `-p mineworld-cli --test attributes`. Mutations **M-k-2** (labels sent in the observer view) → the view test red;
  **M-k-3** (a missing word falls back to the slug) → the all-or-nothing case red.
- [ ] Implementation. [ ] Validation. [ ] Review: English path byte-identical; no view widened; the band never
  disclosed; the 2 048 B rule applied.
- **Commit:** `feat(attributes-model): render every offered locale; the disclosed block's schema 3 (ARC-122, S30-k)`

#### C3 — The provider seam; `attributes` and the owners hold the labels

- **Files:** `sdk/rust/src/{pack.rs, installed.rs}`; `authoring/src/` (`ProviderContexts`); `worldpack/src/load.rs`;
  `systems/attributes/src/system.rs`; `authoring/attributes/src/owner.rs`; `systems/employment/src/system.rs`,
  `systems/employment/tests/occupation.rs`; `systems/notices/src/system.rs` only if it calls `disclose_owned` at C0;
  tests constructing `AttributesSystem` by literal (`AttributesSystem` → `AttributesSystem::default()`).
- **Steps:** §5.7 exactly.
- **Validation:** `cargo test -p mineworld-sdk --lib`; `-p mineworld-worldpack --lib`; `-p mineworld-attributes --test
  pack`; `-p mineworld-employment --test occupation`; `-p mineworld-acceptance --test attributes_seam`;
  `-p mineworld-installed --test installed`. Mutation **M-k-4** (`provider_in` ignored, `Default` used) → the
  real-world zh case in `pack.rs` red.
- [ ] Implementation. [ ] Validation. [ ] Review: no pack other than `attributes` and the owners overrides
  `provider_in`; the provider holds no state; resume builds providers the same way.
- **Commit:** `feat(sdk): a provider may hold presentation data from the composition's contexts; attributes and owners hold labels (ARC-129, S30-k)`

#### C4 — `attributes check --locale` and `show --locale`

- **Files:** `tools/cli/src/{attributes.rs, main.rs}`; `tools/cli/tests/attributes_labels.rs` (new); the generated CLI
  reference regenerates in CI.
- **Steps:** §5.8.
- **Validation:** `cargo test -p mineworld-cli --test attributes_labels` (both worlds; the deliberate gap listed;
  `not offered`). Mutation **M-k-5** (a missing band word not listed) → red.
- [ ] Implementation. [ ] Validation. [ ] Review: exit codes per §5.8; `zh-Hans` normalised.
- **Commit:** `feat(cli): attributes check and show in a locale (S30-k)`

#### C5 — The Python reader

- **Files:** `sdk/python/src/mineworld_sdk/attributes.py`; `sdk/python/tests/test_attributes.py`.
- **Steps:** §5.9 Python; §5.6.
- **Validation:** `uv run pytest sdk/python/tests/test_attributes.py`; ruff; pyright. Mutation **M-k-6**
  (`zh_CN` not mapped) → the golden case red.
- [ ] Implementation. [ ] Validation. [ ] Review: strict decoding; English output unchanged; no locale literal.
- **Commit:** `feat(sdk-python): read schema 3 and compose in a locale (S30-k)`

#### C6 — Cognition and the MCP adapter choose a locale

- **Files:** `cognition/lm-controller/src/mineworld_cognition/{config.py, controller/attributes.py}` and the callers
  that build `SelfAttributes` and descriptions (`consult.py`, `seat.py`, `situation.py` as audited in C0);
  `cognition/lm-controller/tests/test_controller_attributes.py`; `clients/agent/src/mineworld_agent/{__main__.py,
  disclosed.py}` and the place the option reaches the view; `clients/agent/tests/test_view.py`.
- **Steps:** §5.9 cognition and MCP; the config reference regenerates in CI.
- **Validation:** `uv run pytest cognition/lm-controller/tests/test_controller_attributes.py`, then
  `test_controller_choice.py` and `test_record_replay.py` (unchanged with `en`); `uv run pytest
  clients/agent/tests/test_view.py`; ruff; pyright. Mutation **M-k-7** (the seat override ignored) → red.
- [ ] Implementation. [ ] Validation. [ ] Review: default `en` everywhere; old logs replay; the prompt frame unchanged.
- **Commit:** `feat(cognition, agent): a locale per seat and per adapter for attribute text (S30-k)`

#### C7 — Both clients

- **Files:** `clients/protocol/mineworld/attributes.gd`; `clients/protocol/checks/attributes_check.gd`;
  `clients/2d/scripts/{menu.gd, hud/readers.gd}`; `clients/3d-spike/scripts/slice/{slice_main.gd, inspect_panel.gd}`.
- **Steps:** §5.9 GDScript; callers pass `MineWorldText.language()`; re-compose on language change.
- **Validation:** the GDScript golden check through `python3 scripts/ci_godot.py` (locally only with Godot, else
  `NOT RUN` and CI); `cargo test -p mineworld-acceptance --test client_rules`. Mutation **M-k-8** (`tr()` around a
  glance) → the scan red.
- [ ] Implementation. [ ] Validation. [ ] Review: no rule word; no `tr()` on attribute text; logs stay English
  (INV-SET-6).
- **Commit:** `feat(clients): attribute panels in the player's language (S30-k)`

#### C8 — Two worlds, three locales, every surface

- **Files:** `tools/cli/tests/fixtures/attributes-consumers/labels/{zh_Hans.yaml, fr.yaml}` (new);
  `tools/cli/tests/fixtures/labels-elsewhere/**` (new); the oracles of §5.10; `tools/cli/tests/attributes_consumers.rs`
  and `client_2d_attributes.rs` (locale cases); the 3D probe's locale argument; `tests/acceptance/tests/attribute_labels.rs`
  (the scan of I-k-4 and I-k-8; the resume half of AC-k-6).
- **Steps:** AC-k-3, AC-k-4, AC-k-5, AC-k-6, AC-k-8.
- **Validation:** each surface test alone; the 3D probe in CI. Mutation **M-k-9** (a `zh_Hans` literal planted in
  `readers.gd`) → the scan red; **M-k-10** (the reader prefers `labels` from the observer view) → AC-k-4 red.
- [ ] Implementation. [ ] Validation. [ ] Review: oracles hand-written; both worlds exercised; no test computes its
  expectation with the code under test.
- **Commit:** `test(attributes): two worlds and three locales on every surface; nothing moves a digest (S30-k)`

#### C9 — Specification sync

- **Files:** `docs/MODULE_SPEC.md` (§2 `labels/` row: the format; §4.1 *Rendering*: locales, all-or-nothing;
  *Disclosure*: schema 3); `docs/decisions/ARC-129.md` (new, §5.12) and the index
  (`python3 scripts/check_decision_ids.py --write-index`); dated notes in `ARC-70` (world text is selected among
  server-rendered forms by the client's language, never translated by it), `ARC-121` (the provider seam), `ARC-122`
  (schema 3); `sdk/python/README.md` one line.
- **Validation:** `python3 scripts/ci_layer.py docs`; `check_decision_ids.py`.
- [ ] Implementation. [ ] Validation. [ ] Review: no specification text disagrees with the code (`CLAUDE.md` §2.1
  rule 4); old text kept visible.
- **Commit:** `docs(spec): attribute localisation as implemented — label packs, schema 3, the provider seam (ARC-129, S30-k)`

#### C10 — Close-out

- [ ] Implementation: AC-k-8's `git diff --stat` check; push; `gh pr create`; report MERGE-READY with head SHA.
- [ ] Validation: CI `fast` on the exact head (speed-first charter); the full matrix runs after merge.
- [ ] Review: deviations recorded in §16.
- **Commit:** `docs(plan): S30-k1 close-out`

### 9.2 S30-k2 — zh_Hans and the checkpoint (branch `mvp0/pr-s30k2`)

#### D0 — Base

- [ ] Implementation: record the base and S30-k1's merge. [ ] Validation: doc checks. [ ] Review: the style sheet is
  final.
- **Commit:** `docs(plan): S30-k2 — open the ledger at the base`

#### D1 — The zh_Hans label pack

- **Files:** `entities/person-attributes/labels/zh_Hans.yaml` (new); `entities/person-attributes/pack.yaml` (`1.2.0`);
  `entities/person-attributes/README.md` (one line).
- **Steps:** §5.11, every attribute, by group, following the style sheet.
- **Validation:** `target/debug/mineworld attributes check <S30-e fixture> --locale zh_Hans` → AC-k-2's line;
  `cargo test -p mineworld-cli --test attributes_builtin` (unchanged count 93).
- [ ] Implementation. [ ] Validation. [ ] Review (Opus): every entry against the English and the style sheet.
- **Commit:** `feat(person-attributes): zh_Hans labels, phrases and words for every built-in attribute (1.2.0, S30-k)`

#### D2 — Review fixes and the glyph check

- **Files:** the label pack (fixes only); `clients/shared/checks/glyph_check.gd` (the label pack added to the files it
  scans) or a Rust check beside it, whichever the audit in D0 finds simpler; §16 (the `show --locale` print).
- [ ] Implementation. [ ] Validation: AC-k-10. [ ] Review: the operator's spot check recorded (Q-S30k-5).
- **Commit:** `fix(person-attributes): zh_Hans review; every character has a glyph (S30-k)`

#### D3 — The checkpoint in zh_Hans

- **Files:** world 1's label file reduced to its world attributes (§5.10); the zh_Hans oracle regenerated **by hand**
  from the pack; the surface tests' zh cases.
- **Validation:** AC-k-1 on every surface; AC-k-9 measured with the full pack.
- [ ] Implementation. [ ] Validation. [ ] Review: no entry of Alice falls back.
- **Commit:** `test(attributes): Alice in Chinese on every surface (S30-k)`

#### D4 — Close-out

- [ ] Implementation: push; PR; MERGE-READY. [ ] Validation: CI `fast` on the exact head. [ ] Review: deviations.
- **Commit:** `docs(plan): S30-k2 close-out`

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time; `cargo clippy` on the
touched crates; `ruff` and `pyright` on the touched Python packages; heavy jobs through `with-slot.sh`. Never
`cargo test --workspace`, never a whole Python suite, never the nightly locally. CI on the exact head is the gate.

---

## 11. Requirements this PR places on other PRs

- **R-S30h-k-1 (S30-h).** MatrAIx's six locale packs are converted to `labels/<locale>.yaml` by its import tool. S30-h
  measures Bob's view with all offered locales. If it exceeds the 2 048 B bound, S30-h proposes a host option that
  limits rendered locales (never a world configuration key, which would move a digest).
- **R-S30g-k-1 (S30-g).** A reference world adopting `attributes` gets zh_Hans from the core pack with no edit; its
  observations carry `texts`; its digests do not move because of labels (I-k-1).
- **R-PDN-k-1 (PDN).** One `Locale` type in `mineworld-authoring` and one selection rule (§5.6 item 1) for place names
  and attributes. Whichever PR lands second reuses the first's.
- **R-S30i-k-1 (S30-i).** A told attribute ("I'm a night owl") is speech, in the speaker's words; it is not re-rendered
  in the listener's locale.
- **R-S30e-k-1 (S30-e owner).** "1 children" (F-S30k-6) is an English data defect; fix the phrase or record it.

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §13.1 S30-k row | the split into k1 and k2 (if Q-S30k-4 rules so); `zh-Hans` written `zh_Hans` (Q-S30k-3) |
| `step-30-person-attributes.md` §5.6 | "label packs keyed by attribute and value" → rendered on the server for every offered locale; the reader selects (`ARC-129`) |
| `step-30-person-attributes.md` §9.3 | both panels follow the player's language setting |
| `step-30-person-attributes.md` §12 | add I-k-1 as a step invariant: labels are never world state |
| `step-20-client-settings.md` INV-SET-10 | a dated note: attribute text is selected among server-rendered forms, never translated by the client |
| `overall.md` | S30-k designed (draft) / frozen / merged; k1 and k2 rows |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-k — localisation of attributes. Two PRs: S30-k1 (mechanism, §9.1) and
                         S30-k2 (zh_Hans data and checkpoint, §9.2)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30k-localisation.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§5.6, 9, 11.2, 12, 13); pr-s30c-attributes-pack.md;
                         pr-s30e-builtin-catalogue.md; pr-s30f-attribute-consumers.md; step-20-client-settings.md
                         (§§3.6, 5 INV-SET-6/9/10); docs/MODULE_SPEC.md §§2, 4.1; docs/DECISIONS ARC-70, ARC-121,
                         ARC-122; CLAUDE.md; docs/ENGINEERING_STANDARDS.md; .structured-coding/standards.md;
                         LANE_LEAD_CHARTER.md
WORKTREES:               /Users/yuema137/mineworld-worktrees/impl-s30k1 and impl-s30k2, each its own, one session each
BRANCHES:                mvp0/pr-s30k1, mvp0/pr-s30k2, each from origin/main at its C0 / D0
START CONDITION:         S30-f merged (met, #373) AND this design frozen AND §15 ruled, then the primary's dispatch;
                         S30-k2 after S30-k1 merged (drafting D1's file may start at freeze, in the k2 worktree)
IMPLEMENTATION BASE:     main at C0 / D0; recorded there
APPROVED SCOPE:          §2.1 items 1–11 (k1) and 12–14 (k2), as §15's rulings shape them
FROZEN INVARIANTS:       step-30 I-S30-3, I-S30-8, I-S30-10; §6 I-k-1 … I-k-9; §8 as written
SEQUENCE:                C0 … C10, then D0 … D4 (§9)
IMPLEMENTER:             k1 Opus-class, C5–C7 delegable to Haiku-class; k2 Haiku-class drafting, Opus review;
                         reviewer of each PR: the primary session
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace), through with-slot.sh when heavy;
                         target/*/mineworld *; uv run pytest <one file>; uv run ruff; uv run pyright;
                         python3 scripts/ci_godot.py; git; gh (pr create/view; never merge);
                         python3 scripts/check_doc_headings.py, check_decision_ids.py, ci_layer.py docs; cp; cmp;
                         mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace; a whole Python
                         suite; any edit under kernel/, contracts/, server/, worlds/; any System other than attributes,
                         employment (and notices if it is an owner at C0); a new external dependency; any model call
MATERIAL STOPS:          a fact or digest moves because of labels; Bob's view over 2 048 B with zh_Hans; the provider
                         seam needs a kernel, contracts or server change; an English output changes; PDN lands with
                         an incompatible Locale spelling
PLATFORMS:               Linux, macOS, Windows (CI core and godot layers)
VALIDATION BUDGET:       targeted tests unrestricted, one file at a time; Gate 1 NOT REQUIRED; Gate 2 = the real-binary
                         resume case only
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30k.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized at freeze (source: the operator's freeze)
                         semantic commits: authorized (working rules §14)
                         branch push: authorized (working rules §21)
                         PR creation / update: authorized (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: MERGE-READY reported to the coordinator (LANE_LEAD_CHARTER speed-first, 2026-10-18)
                         merge: never by the implementation session; the coordinator merges
POST-MERGE SYNC OWNER:   primary: step-30, step-20, overall.md (§12); implementation sessions: this ledger
STOP CONDITION:          S30-k1, then S30-k2, MERGE-READY — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-k-1 | The provider seam is used to smuggle state into a provider | the SDK doc forbids it; the data is `Arc` of immutable decoded files; review in C3; I-k-1's equality test |
| R-k-2 | Payload grows with every offered locale; MatrAIx offers seven | holder-only `labels`; AC-k-9 measured; the 2 048 B stop; R-S30h-k-1 |
| R-k-3 | Machine-quality Chinese reaches players | the style sheet; the Opus review; the operator's spot check (Q-S30k-5) |
| R-k-4 | English changes and a translation goes stale silently | `check --locale` lists missing pieces, not stale ones; follow-up: a `translates: <pack> <version>` header checked by `check`, or a `.po` export (`ARC-129` revisit) |
| R-k-5 | PDN and this PR choose different locale spellings or rules | one `Locale` in `mineworld-authoring`; R-PDN-k-1; a material stop if incompatible |
| R-k-6 | A mixed-language prompt (English frame, Chinese content) confuses a model | no LM gate here; Q-S30k-6 defers frame localisation to its own design |
| R-k-7 | A world override of an English word leaves the zh word unchanged | expected: the world also writes `labels/zh_Hans.yaml` if it wants; documented in `MODULE_SPEC` §4.1 |

---

## 15. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30k-1 [who selects the locale; the wire]** | Where is the language chosen? (a) each reader picks from texts the server renders for every offered locale; (b) the server renders one locale per seat, which needs a protocol field and a locale inside perception; (c) the world's configuration picks one | **(a).** No protocol, kernel, contract or server change; an observation is the same for every reader; matches PDN's "send the forms, the reader selects". (c) would move a digest (B-3). (b) is smaller on the wire but touches the server and every perception path; keep it as the fallback if S30-h's size measurement fails |
| **Q-S30k-2 [the SDK seam; `ARC-121`]** | Labels must reach the provider without being world state. May a pack's perception provider be built with read access to the composition's owner contexts (`SystemPack::provider_in`, default unchanged)? | **Yes**, as §5.7: generic, additive, no kernel or server change. The alternative — labels stated in `attributes-composed` — moves every adopting world's digest on each translation fix and breaks resume |
| **Q-S30k-3 [spelling]** | The step says `zh-Hans`; the client catalogs and PDN say `zh_Hans`. Which is canonical? | **`zh_Hans`** (file `labels/zh_Hans.yaml`), the spelling the repository already uses; `zh-Hans` accepted wherever a person types it |
| **Q-S30k-4 [split]** | One PR or two? | **Two from this one design:** S30-k1 the mechanism (Opus, code), S30-k2 the zh_Hans data and the checkpoint (Haiku drafting, Opus review). k2's file can be drafted in parallel after freeze; k1's own checkpoint uses fixture locales on two worlds |
| **Q-S30k-5 [who signs off the Chinese]** | Step-30 says "reviewed". By whom? | **An Opus review pass against the style sheet, then the operator spot-checks the sensitive attributes and twenty random entries** from `attributes show --locale zh_Hans`. No paid translation service |
| **Q-S30k-6 [the prompt frame]** | Should the LM prompt's own lines ("What the world says about you:", rules, "People here") also be Chinese for a zh_Hans seat? | **Not in S30-k.** S30-k localises attribute text only. A Chinese prompt frame changes prompt goldens and model behaviour and deserves its own cognition design |
| **Q-S30k-7 [decision ids]** | A record is needed for the localisation choice and the reuse comparison. Any `DEP` id? | **`ARC-129` only.** No dependency is adopted; Fluent, ICU MessageFormat and gettext are recorded as declined inside `ARC-129` with revisit triggers |

Bounded decisions recorded here, not asked: D-k-1 (readers select, §5.6), D-k-2 (all-or-nothing per entry), D-k-3
(no `en` label pack), holder-only `labels` (§5.5), authored text stays as written (F-S30k-7).

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | | [ ] | [ ] | [ ] |
| C1 | | [ ] | [ ] | [ ] |
| C2 | | [ ] | [ ] | [ ] |
| C3 | | [ ] | [ ] | [ ] |
| C4 | | [ ] | [ ] | [ ] |
| C5 | | [ ] | [ ] | [ ] |
| C6 | | [ ] | [ ] | [ ] |
| C7 | | [ ] | [ ] | [ ] |
| C8 | | [ ] | [ ] | [ ] |
| C9 | | [ ] | [ ] | [ ] |
| C10 | | [ ] | [ ] | [ ] |
| D0 | | [ ] | [ ] | [ ] |
| D1 | | [ ] | [ ] | [ ] |
| D2 | | [ ] | [ ] | [ ] |
| D3 | | [ ] | [ ] | [ ] |
| D4 | | [ ] | [ ] | [ ] |

### Evidence

(empty)

### Sizes (AC-k-9)

(empty)

### Mutations

(empty)

### Deviations and findings

(empty)

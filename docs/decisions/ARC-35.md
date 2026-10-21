## ARC-35 — How AC-1 is measured

**Date** 2026-10-07 · **Approved by** the operator at S9's design freeze (step-10 QS-2) · **Implements**
[`MVP.md`](MVP.md) §2, §9 `AC-1` · **Relates to** `ARC-23`, `ARC-33`, `DEP-12` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §1.3 (I-1, I-2, I-9), §2.5, SD-6 (S9)

**Problem.** `AC-1` and the frozen top-level criterion say that Market Town is Social Café plus
installed systems and a configuration change, with no edit to the kernel, `Person`, a renderer or a
controller. `overall.md` §1 glosses that as a change that touches only `systems/` and `worlds/`. Two
things make the sentence measurable only if the measurement is decided first:
- Making packs installable (F-1, `ARC-33`) and letting a controller attempt an action it was never
  compiled against (F-3) are framework changes. They must land before the market, and outside the
  measured change.
- A measurement chosen after seeing the result is not a measurement (`ARC-23`).

**Choice.**

1. **The transformation is two named merges.** The market arrives in PRs 11d (item, inventory,
   item-transfer, and `worlds/market-town`) and 11e (economy, employment, and their content). Each
   merge commit `M` is read against its own first parent, so unrelated PRs merging in between do not
   enter the range.
2. **Check 1, the change set.** `git diff --name-only M^1 M` ⊆ allowed, where allowed is:
   - `systems/**`;
   - `worlds/**`;
   - `Cargo.lock`, under a rule: every `[[package]]` added between `M^1` and `M` has no `source`
     (a path package) and lives under `systems/`, and every `[[package]]` whose dependency list
     changed lives under `systems/`. No external dependency arrives with the market;
   - Markdown documentation: `**/*.md` under `docs/`, `systems/`, `worlds/` and
     `.structured-coding/plans/`, because a PR here always updates its ledger.
3. **Check 2, the structure** — from `cargo metadata` at HEAD, independent of history:
   - the direct dependents of each market pack are `systems/*` crates only;
   - no dependency path leads to a market pack from `kernel`, `contracts`, `persistence`, `server`,
     `authoring`, `sdk` or `rule-controller`;
   - no code file (`*.rs`, `Cargo.toml`) outside `systems/`, `worlds/` and `tests/acceptance/` names
     a market pack's crate. Crate names are matched, not action or event slugs: contract tests
     already use stub ids such as `inventory-stub` that name no pack.

   Check 2 catches what a path diff cannot: a kernel or controller taught the market *before* the
   transformation range.
4. **Check 3, the world delta.** Market Town is Social Café plus configuration:
   - `systems`: Social Café's list, in order, then the five market packs;
   - places, population and seats: identical keys;
   - every person and place file: Social Café's fields and sections unchanged, plus sections owned
     by market packs only;
   - `items/` and `organizations/`: present only in Market Town.
5. **Fail closed.** Check 1 needs git history. Without it the test fails, naming the missing
   history. It never skips.
6. **The precursors are bounded instead of measured** — by two frozen invariants, each checked:
   - **I-2: the precursors know no market.** PRs 11a, 11b and 11c add no market concept (item 7).
   - **I-9: the controller's offer band is decided before the market exists.** Its constants are
     fixed in 11c against a synthetic pack. If Market Town behaves badly, the remedy is in
     `systems/` or `worlds/`, never in the controller.
7. **I-2 is checked mechanically**, by `tests/acceptance/tests/precursor_vocabulary.rs` (crate
   `mineworld-acceptance`, the home `ARCHITECTURE.md` §14 gives acceptance tests):
   - **Vocabulary:** `item`, `inventory`, `money`, `price`, `wage`, `job`, `shift`, `shop`,
     `economy`, `employ`.
   - **What is scanned:** every line a precursor adds, and the path of every file it adds, in every
     file except Markdown. Code, comments, manifests, `Cargo.lock` and fixtures are all scanned.
     Documentation is not, because documentation must be able to discuss the market; this record
     does.
   - **How a match is found:** a line is split into words at every character that is not a letter
     or a digit, and at every lower-to-upper case boundary. A word matches when, lowercased, it
     begins with a vocabulary word, so `items`, `ShopFront`, `employer` and `wages` all match.
   - **Which lines are "the PR's added lines", deterministically.** The test holds one row per
     precursor: the PR, its recorded base commit, and its branch.
     - If the first-parent history of `HEAD` holds that branch's merge commit `M` (subject
       `Merge pull request #N from <owner>/<branch>`, the form every merge to the protected `main`
       takes, `ARC-5`), the range is `base..M^2`: exactly what the PR added, whatever merged later.
     - Otherwise the PR is not merged, and the range is from `base` to the working tree: tracked
       changes (`git diff <base>`) plus every untracked file Git does not ignore. A line is
       therefore scanned before it is committed.
     - A precursor PR adds its own row and its own allow-list entries; 11b and 11c extend the same
       test.
   - **Allow-list:** each entry names a file, a line substring and a reason. It admits only matches
     that are not a market concept: the scan's own vocabulary list, a word in another sense (Rust's
     `Iterator::Item`), or a pre-existing use carried into an added line. An entry with an empty
     reason fails the test, and so does an entry that matches nothing, so the list cannot go stale.
   - **Fail closed:** the test fails, naming the cause, when `git` cannot run, the directory is not a
     repository, a recorded base is missing (a shallow clone), or `HEAD` does not descend from it.
     It never skips.
   - **A precursor that turns out to need a market word is a material stop**, decided by the
     operator, never an allow-list entry added to pass.
   - A squash merge would leave no merge commit. The range would then fall back to `base..HEAD`,
     which fails once the market exists. The failure is loud, not silent.

**What this does not claim.** That S9 as a whole changed only `systems/` and `worlds/`: it did not,
and could not for any linked pack (`ARC-33`). The criterion is read as `MVP.md` §2 words it, and
`overall.md` §1's path gloss is made exact by items 1–7.

**Accepted limitations.**
- The measurement depends on two merge commits being identified by id, which exist only after they
  merge. The proof (PR 11f) records them.
- Documentation inside the range is admitted by path and extension, not by content.

**Note, 2026-10-07 (S9, PR 11b; step-10 QS-16, F-31).** Item 7's allow-list admits **words, not
lines**. This tightens the check and loosens nothing:

1. Each entry names a file, a line substring, the exact lowercase words it admits, and a reason. Every
   market word on an added line is found, not only the first. Each one is refused unless an entry for
   that precursor and that file, whose substring the line contains, admits that exact word. A refusal
   names the word.
2. Before this note, an entry admitted every match on the lines it covered. A line such as
   `item_price`, admitted for the defined term `item`, would then have hidden `price`. Now the same
   line is refused, naming `price`. A word that merely begins with an admitted one is refused as
   itself: `itemprice` is not `item`.
3. An entry may admit **any** word only for the scan's own file, which must name the vocabulary it
   looks for. The test fails, naming any other entry that admits any word.
4. The other rules of item 7 are unchanged:
   - the vocabulary;
   - what is scanned (every non-Markdown added line and added path);
   - how a precursor's range is found;
   - failing closed;
   - an entry with an empty reason fails, and so does an entry that admits nothing.

Item 7's first-parent detection of a merged precursor is unchanged as well. Step-10 QS-15 proposed
reading every merge reachable from `HEAD`, and the primary session declined it. PRs 11b and 11c are
merged one at a time instead. The second to merge rebases onto the new `main` and moves its own row's
base to that commit (step-10 §12.0).

**Note, 2026-10-07 (S9, PR 11e; step-10 QS-35, QS-52) — six market packs.** The operator added a
consumption System Pack to the transformation's second merge (step-10 QS-35), so the market is six
packs, not five: `item`, `inventory`, `item-transfer`, `economy`, `employment` and `consumption`
(`ARC-37`, `ARC-38`). Item 1's second merge (11e) therefore carries economy, employment and
consumption, and edits the merged `inventory` pack, which is under `systems/` and inside check 1's
allowed set (step-10 QS-42). Check 2's "market pack" and check 3's "the five market packs" read **the
six market packs**. No other item changes. The AC-1 test that reads the list is step-10 PR 11f's.

**Note, 2026-10-07 (S9, PR 11f; step-10 QS-54 … QS-59, F-58 … F-63) — how the proof measures.** The
AC-1 test is `tests/acceptance/tests/ac1_composability.rs`, one `#[test]` per check, so a failing
check never hides another. PR 11f is not part of the measured transformation (item 1): it adds tests
and documentation and changes no pack, no world configuration and no framework behaviour. It settles
how each item is read, and tightens where it can; no item is relaxed except by the operator's answer
to QS-54 below.

1. **The two merges, by subject and by id** (item 1, the accepted limitation; QS-55). The test records
   both merges: 11d is `70e532f383ff81464d00066ff85e1b7fdc2296b0`, GitHub #43 from
   `mvp0/pr-11d-owning-things`; 11e is `2dddda86a4347d54de7bd2303d5268a421870b5f`, #46 from
   `mvp0/pr-11e-work-money-shops`. Each must be found on `git log --first-parent --merges HEAD` by its
   subject, `Merge pull request #<n> from <owner>/<branch>`, exactly once. The match must equal the
   recorded id and have exactly two parents. The subject proves that the id is that PR's merge on
   `main`'s first-parent chain; the id pins it, so a later merge with a recycled branch name cannot
   take its place. A squash merge has one parent and is refused.
2. **Check 1's `Cargo.lock` rule, as risk R-S9-6 words it** (item 2; QS-56). Both versions of
   `Cargo.lock` are read with `git show`. Every `[[package]]` added, removed or changed in **any**
   field between `M^1` and `M` must have no `source` and must be a crate under `systems/` at `M`. This
   tightens item 2, which names only "added" and "dependency list changed": a version or checksum
   change of an external package inside the range, a `cargo update`, is refused too. Both merges
   pass it.
3. **Check 2's "dependency path" counts normal and build edges** (item 3, second bullet; QS-54,
   F-58). `persistence`'s test `kill_and_resume` dev-depends on `worldpack`, which links the installed
   set (`ARC-33`) and so every market pack. Read over every edge kind, the check would fail AC-1 for a
   test that loads a World Pack, though nothing in `persistence` knows the market. The operator
   decided (2026-10-07, step-10 §4.6.0):

   > "QS-54: accepted by the operator (2026-10-07). ARC-35 check 2's "no dependency path" rule counts
   > normal and build edges only. Dev edges stay bound by check 2's other two rules: every dependent
   > of a market pack lives under `systems/`, and no code outside `systems/`, `worlds/` and
   > `tests/acceptance/` names a market crate."

   So the path rule follows what a library or a binary links. A dev edge is still seen by the first
   bullet (any declared dependency on a market pack, of any kind, must come from `systems/`) and by
   the third (no code file outside the three directories names a market crate). The alternative of
   admitting every edge through `mineworld-installed-systems` was declined: it would let a kernel that
   linked `worldpack` in production pass.
4. **How check 2 reads the structure** (item 3; F-60). It runs `cargo metadata --no-deps
   --format-version 1 --offline` through the `cargo` that built the test. `--no-deps` is enough, and
   needs no network: only a workspace member can depend on a path crate, so every path to a market pack
   runs through members, whose declared dependencies the output lists with their kind and path. A
   `cargo` that cannot run, or output that does not parse, fails the check by name. The crate-name
   scan reads the working tree — tracked files and untracked files Git does not ignore — so an
   uncommitted edit is seen. A name matches as a word: `mineworld-item` does not match inside
   `mineworld-item-transfer` by accident.
5. **Check 3 compares configuration structurally** (item 4; QS-57, F-62). Both packs must first be
   read by `WorldPack::read`. Each YAML file is then parsed into a structural value and compared.
   `world.id` and `world.name` may differ. `systems` is Social Café's list, in order, followed by
   exactly the six market packs (as a set). `places`, `population` and `seats` are equal. `items` and
   `organizations` are present in Market Town only. In every place and person file, each of Social
   Café's keys has an equal value, and every key Market Town adds is a section whose owner, by the
   build's own `Capability::owning_section`, is one of the six market packs. `items/` and
   `organizations/` exist in Market Town only. `README.md` is not configuration. Every difference is
   named by file and key.
6. **World-level tests read the market by type slugs, never through a market crate** (QS-58, F-61).
   A test that runs the `mineworld` binary lives in `tools/cli`, and the third bullet of check 2
   forbids naming a market crate there. Those tests therefore decode market facts and components by
   their literal slugs into test-local mirrors: unknown fields refused, schema version 1 asserted.
   That is also the oracle independence the test rules ask for.
7. **CP-4 is committed at the full 300 days, in the default suite** (QS-59). The bounded-horizon
   economy (`ARC-38`, L-13) is a 300-day claim, so a shorter committed horizon would let content that
   drains after its end pass. Activity is checked before any comparison of runs (I-7). Money
   conservation and the capacity of six are checked against the state in the save's newest snapshot,
   as well as the replayed facts, because the facts alone conserve money by construction (F-63).
8. **Check 1 fails closed without full history** (item 5). A shallow clone fails, naming the missing
   history and the remedy, `fetch-depth: 0`; it never skips. A CI that runs the test must check out
   the full history (S13).

**What the proof does not claim.** Installation without a rebuild: installing a pack is a directory,
two lines in `systems/installed` and a rebuild (`ARC-33`); without a rebuild is `ARC-8`'s Tier 1,
outside MVP-0. The evidence is recorded in step-10 §9.6.

**Note, 2026-10-08 (S15, PR 12d; step-11 §19, QD-1 decided by the operator, SD-D9) — Social Café may
own items.** A loose object is an Item file carrying `body:` (`ARC-36` note), and it is Social Café's
content exactly as a place's `body:` is, so item 4's last bullet and item 5 of the 11f note are amended.
The operator's decision, as recorded in step-11 §19: Social Café may own `items/` (loose objects as Item
files with `body:`); Market Town carries them unchanged; the claim "Market Town = Social Café +
installed packs + configuration" stays word for word; market-owned item sections stay Market Town's
only. Check 3 therefore reads:

1. `items` may be present in Social Café. Market Town's `items` contains every key of Social Café's (as
   a set); `organizations` stays present in Market Town only.
2. Every item file Social Café has exists in Market Town, and is compared exactly as a place or person
   file is: each of Social Café's keys has an equal value, and every key Market Town adds is a section
   owned by one of the six market packs.
3. Every item file only Market Town has carries only the format's fields and sections a market pack
   owns, unchanged; so a `body:` in a Market Town-only item file is refused, by file and key.
4. `organizations/` exists in Market Town only, unchanged.

Every world before 12d has no `items` in Social Café, and the check reads it exactly as before. The
claim measured is unchanged: **Market Town is Social Café plus configuration.** (It landed after the
three S19 notes below, which it does not change: allow-listed generic packs, their configuration and
their data stay Market Town's only.)

**Note, 2026-10-09 (S19, PR TW-a; step-19 §16.9 TWa-F1) — generic packs after the market.** The
operator ruled on TW-a's material stop (2026-10-09, option (i)): Market Town may enable generic packs,
named on an explicit allow-list, after the six market packs, through configuration. **The claim is
unchanged: the market delta is measured exactly.** Check 3 (item 4, as note 11f item 5 reads it) now
reads:

1. `systems` is Social Café's list, in order, then exactly the six market packs (as a set), then only
   packs on the allow-list, each once. An allow-listed pack anywhere before the end of the six, and any
   pack after the six that is not on the list, is refused, naming the pack.
2. `configure` is absent in Social Café. In Market Town it is absent or lists only allow-listed packs
   that `systems` enables; `configure/` exists in Market Town only and holds only their files.
3. Every other part of check 3 is unchanged: an allow-listed pack adds no section to a person or place
   file, because only a market pack's sections are admitted there.

The allow-list is `GENERIC_PACKS` in `tests/acceptance/tests/ac1_composability.rs`. It is
`["calendar"]` (`ARC-67`, `DEP-30`). Weather and any later generic pack are added by their own PRs, each
with its own review. A pack is generic when it is neither Social Café's nor the market's: a calendar is
a world's date and sun, not a market. Checks 1 and 2 are untouched: the two merges, the lock rule and the
dependency structure still measure the market and nothing else. A test holds both refusals.

**Note, 2026-10-09 (S19, PR TW-b; step-19 §17.5 C4) — `weather` joins the allow-list.** `GENERIC_PACKS`
is now `["calendar", "weather"]` (`ARC-68`). Weather is a world's climate, not a market, so it is generic
by the rule above. Nothing else in check 3 changes. Order among the allow-listed packs is free; each
still follows the six and is listed once; and `configure/weather.yaml` is admitted as the configuration
of an allow-listed pack that `systems` enables. The unit test's example of a pack that is not
allow-listed is now `bodies` (it was `weather`).

**Note, 2026-10-09 (S19, PR TW-d; step-19 §18.9.1 QTWd-1) — an allow-listed pack's `data/` attachments.**
The primary session ruled (2026-10-09) that admitting an allow-listed generic pack's data is the same
allowance as admitting its configuration (TWa-R1), since an attachment is configuration (`ARC-61`'s
`data:` note). Check 3 therefore admits a `data/` directory **in Market Town only**, when every file in
it, at any depth, is either an attachment named by the `configure/` file of an allow-listed pack the
world enables, or a file named `NOTICE` in the same directory as such an attachment (its licence and
provenance record, `DEP-8`, `DEP-31`). Any other file under `data/` is refused, naming it; a `data/` in
Social Café is refused. **The claim is unchanged: the market delta is measured exactly.** No market pack
gains anything by this, and checks 1 and 2 are untouched. Two mutations guard it (step-19 M-TWd-A1: a
stray `data/other.txt` in Market Town fails naming the file; M-TWd-A2: a `data/` in Social Café fails).

**Note, 2026-10-15 (WE-a, PR #189; the operator's ruling on S31-a Q-S31a-1, option (a)) — a market pack's
configuration, and `happenings` on the allow-list.** A market pack's configuration is part of the market
delta, so admitting it keeps the delta measured exactly. Check 3 (as the 2026-10-09 notes read it) now
also admits, **in Market Town only**, the `configure:` entry and the `configure/<key>.yaml` file of any of
the six market packs that `systems` enables. Such a pack still stands among the six in `systems`, not
after them. `configure` and `configure/` stay absent in Social Café, and a market pack configured but not
enabled is refused by name. `data/` is unchanged: it still admits only an allow-listed generic pack's
attachments. The first such file is Market Town's `configure/economy.yaml` (WE-a C5: the store's
market-day price modifier, `ARC-88`). The same PR adds `happenings` to `GENERIC_PACKS`, which is now
`["calendar", "weather", "happenings"]`: a world's happenings are neither Social Café nor the market, so
the pack is generic by the 2026-10-09 rule. Checks 1 and 2 are untouched. A unit test holds the ruling's
two mutations: a market pack's configuration in Social Café, and a market pack configured but not enabled.
This note implements Q-S31a-1 (a) once; S31-a uses it as it stands and does not repeat the change in
`ac1_composability.rs` or here.

**Note, 2026-10-19 (S35, PR N-d; `pr-s35-nd-content.md` §1.2, Q-Nd-1, ruled yes by the operator on
2026-10-18) — `sleep` and `needs` join the allow-list, with the sections they own.** `GENERIC_PACKS` is now
`["calendar", "weather", "happenings", "sleep", "needs"]` (`ARC-283`, `ARC-284`). Sleeping and getting hungry
are a person's, not a market's, so both packs are generic by the rule above. Unlike calendar and weather,
`needs` owns a section on people and items (`needs:`, `ARC-285`), so item 3 of the 2026-10-09 note is widened
by exactly that: a key Market Town adds to a person, place, item or organization file is admitted when a
market pack **or an allow-listed generic pack** owns it, by the build's own catalog. Social Café still carries
no such section: it enables no generic pack, and the loader refuses a section whose owner a world does not
enable (a test holds it). `configure/sleep.yaml` and `configure/needs.yaml` are admitted as configurations of
allow-listed packs that `systems` enables. Checks 1 and 2 are untouched; the market delta is still measured
exactly.

**Note, 2026-10-19 (milestone F fix; `ARC-269`, the operator's ruling Q-WC-3) — Social Café's own generic
packs are part of the base.** `ARC-269` made Social Café enable `calendar` with the same configuration as
Market Town, and kept Market Town byte-identical, so Market Town's `systems` no longer began with Social
Café's list and check 3 failed. Check 3 is stale here, not the world: an allow-listed generic pack is
neither Social Café nor the market (the 2026-10-09 rule), so its position does not belong to the base's
order. Check 3 now reads it this way, for every generic pack and not for `calendar` alone:

1. `systems`: Social Café's list may end with allow-listed generic packs. Its other packs, in order, begin
   Market Town's list; the six market packs follow; then only allow-listed generic packs, and they must
   include every generic pack Social Café enables. A generic pack before the end of Social Café's own packs
   is refused by name.
2. `configure`: Social Café may list only allow-listed generic packs it enables, and Market Town must list
   each of them too. A market pack's configuration in Social Café is still refused.
3. Files: each `configure/<key>.yaml` of Social Café must be in Market Town with an equal parsed value, and
   each `data/` file of Social Café must be an attachment of its generic configuration (or that
   attachment's `NOTICE`) and byte-equal in Market Town.

So what Social Café configures is base, held unchanged, and the market delta is still measured exactly. A
unit test holds the new mutations: a generic pack Social Café enables but Market Town does not, one it
configures but does not enable, and one placed before its own packs. Checks 1 and 2 are untouched.

**Note, 2026-10-20 (milestone F fix; S34-c1, PR #368) — `posture` joins the allow-list, and a generic pack's
place in Social Café is free.** #368 enabled `posture` (sit points) in both towns, appended after `calendar`
in Social Café and after the generic packs in Market Town. Sitting down is a person's, not a market's, so
`posture` is generic by the 2026-10-09 rule: `GENERIC_PACKS` is now `["calendar", "weather", "happenings",
"sleep", "needs", "posture"]`, and its `sit-points:` section is admitted like `needs:`. Item 1 of the
2026-10-19 note is relaxed: an allow-listed generic pack may stand anywhere in Social Café's list, because
only the order of Social Café's own packs is the base Market Town begins with; Market Town must still enable
each of them after the six market packs. Checks 1 and 2 are untouched.

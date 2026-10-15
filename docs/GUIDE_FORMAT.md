# The guide format: how a `docs/guide/` page is written and executed

**Status.** Specification, `ARC-90` (guide part), written by PR S25c, commit C0. **Authority.** This file is the
only authority for the block grammar, the directives, the page rules and the mirror rules of the executable guide.
The guide pages themselves are never an authority (`CLAUDE.md` §2.1, the guide class). **Executed by**
`tools/cli/tests/guide.rs` and `tools/cli/tests/guide/` (the runner, PR S25c). **Parent** `.structured-coding/plans/mvp0/step-25-creator-docs.md`
§§6.2, 8, 4.3 (`G1`).

A guide page is a Markdown page a person follows step by step. Each page is run against the real `mineworld`
binary, in page order, so a page that stops working is a red test. This file says exactly which parts of a page the
runner executes, which it only shows, and which it refuses.

---

## 1. Files

| Path | What it is |
| --- | --- |
| `docs/guide/README.md` | The human index of the guide. Not a page: it is never executed. |
| `docs/guide/NN-name.md` | An English page. Executed. Pages are named with a two-digit prefix and a hyphenated name. |
| `docs/guide/NN-name.zh-CN.md` | The Chinese mirror of the page with the same name (§7). Never executed; it is checked for parity. |
| `docs/guide/compare/…` | Comparison pages (PR 25e). Same rules as chapter pages. |
| `docs/guide/examples/corner-shop/` | The committed example world the runner compares a scratch folder with (`equals`, §6.1). Not a page. |
| `docs/guide/features.toml` | Which guide features are merged (§9). |
| `docs/reference/` | The generated reference (`mineworld reference`, PR 25a). Never hand-edited. Link target only. |

The runner is given a **guide root** (the directory holding `features.toml` and the pages), a **repository root**
(against which `equals` paths are resolved) and optionally a **reference directory** (against which reference links
are resolved). The real guide uses `docs/guide`, the repository root and `docs/reference`. A test may give any
directory of the same shape.

### 1.1 Page discovery and order

1. A page is a file directly inside the guide root whose name ends in `.md`, excluding `README.md` and any name
   ending in `.zh-CN.md`. Subdirectories are not pages; `compare/` is walked by its own index in PR 25e and
   its pages are added to the run order there.
2. Pages run in ascending order of their file name (byte order). The two-digit prefix makes that the reading order.
3. A page whose file name has no two-digit prefix is refused: the run fails naming the file.

### 1.2 Mirror presence

Every English page has a mirror with the same file name and the suffix `.zh-CN.md`. A mirror without its English
page, and an English page without its mirror, both fail the run naming the file.

---

## 2. Fenced blocks

Only **backtick fences** (three or more backticks, as CommonMark defines them) are read. A tilde fence (`~~~`) or
an indented code block fails the page, naming the line, because a reader and the runner must agree on what a block
is. Every fence has an **info string**. Its first word is the language. The rest of the info string is a list of
space-separated tokens. The runner recognises exactly these tokens:

| Token | Valid on | Meaning |
| --- | --- | --- |
| `file=PATH` | any language | The block is a **file block** (§3). At most one per block. |
| `console` | the language word `console` only | The block is a **command block** (§4). |
| `exit=N` | `console` blocks only | Every command in the block is expected to exit with code `N` (§4.3). |

Rules for the info string:

1. A block with `file=` is a file block, whatever its language word. It is always executed.
2. A block whose language word is `console` is a command block. It must carry no `file=`.
3. Any other fenced block (language words such as `text`, `yaml`, `toml`, `json`, `rust`, `sh`, `powershell`,
   `markdown`, or no language at all) is **shown only**. The runner does not execute it and does not read it.
4. A token the runner does not recognise, a repeated token, `file=` together with `console`, or `exit=` on a
   non-console block **fails the page**, naming the page, the line and the token. An unlisted info string is a
   failure, never a silent skip. (This is the decision the S25 design left open, recorded here.)
5. A `file=` path is validated when it is read (§3.2). A `console` block that contains no command line fails the page.

Blocks are read in document order. A block inside an HTML comment is not a block.

---

## 3. File blocks

### 3.1 Effect

A file block writes its text to `PATH` inside the page's scratch folder (§5), creating missing parent folders and
replacing the file if it exists. The text is the block's content with the fence lines removed; the content is
written byte for byte, with its own line endings. The runner never adds, trims or reformats a file block.

### 3.2 Path confinement

`PATH` must satisfy all of the following, or the page fails naming the path:

1. It is not empty.
2. It uses `/` as its only separator. A `\` anywhere in the path is refused.
3. It is not absolute: it does not begin with `/`, and it has no Windows drive prefix (`C:`, `c:/`) or UNC prefix.
4. It has no component equal to `..`, and no empty component (`a//b`), and no component equal to `.`.
5. Once joined to the scratch folder, it resolves to a path inside that folder. (The runner checks the joined
   path's components, so a symbolic link already present in the scratch folder cannot be followed out.)

A file block never writes outside its page's scratch folder. A test asserts that `../x` is refused (AC-25c-2).

---

## 4. Command blocks

### 4.1 Lines

Inside a `console` block, a line that begins with `$ ` (dollar and one space) is a **command line**. Every other
line is **expected output** for the nearest preceding command line. Lines before the first command line, other than
blank lines, fail the page. A command line with no expected-output lines after it expects no output, which is
written as an empty expectation.

### 4.2 Commands

1. The command is split into words on ASCII spaces. No shell is involved: the runner calls the binary directly.
2. The first word must be exactly `mineworld`. The runner runs the `mineworld` binary built for the test
   (`CARGO_BIN_EXE_mineworld`), not whatever `mineworld` is on the path. Any other first word fails the page.
3. These characters are refused anywhere in a command line, and the page fails naming the character: `|`, `&`, `;`,
   `<`, `>`, `*`, `?`, `$`, `` ` ``, `(`, `)`, `{`, `}`, `~`, `"` and `'`. A command therefore never depends on a shell,
   so it runs the same on every OS.
4. A path argument uses `/` as its separator and is relative to the page's scratch folder. The same confinement
   as §3.2 applies to any argument that names a path.
5. The working directory of every command is the page's scratch folder.
6. Commands in one block run in order. A command that exits with a code other than the expected one stops the block.
   Expected output of a command is matched even when its exit code is wrong, so the failure report shows both.

### 4.3 Exit codes

The expected exit code of a command is `0`, unless the block's info string has `exit=N`, which sets it to `N` for
every command in that block. A command that exits with another code fails the block, and the failure names the
expected and actual code.

### 4.4 Output matching

1. The actual output of a command is its standard output and its standard error, combined in the order the process
   wrote them.
2. The expected output and the actual output are compared with `snapbox` (the matching library of `trycmd`, `DEP-50`),
   using its two matching rules:
   - a line consisting of `...` matches any number of lines (including none);
   - `[..]` inside a line matches any text inside that line, including nothing.
3. The only normalisation applied to actual output before matching is line endings (`\r\n` becomes `\n`) and path
   separators inside printed paths (`\` becomes `/`). Nothing else is rewritten, so a difference in wording, spacing,
   order or number fails the page.
4. A reader is shown the lines the page quotes. Every line the page quotes must match, in order, and the output must
   not contain extra lines where the page quotes none, unless the page uses `...`.

### 4.5 Other blocks

A `console` block is the only command form. A block the runner does not execute (§2 rule 3) is never run, and a
shell prompt inside any other block is text, not a command.

---

## 5. Scratch folders and continuation

1. Each page has one scratch folder, created empty when the page starts, and removed when the run ends (DEP-29,
   `mineworld-test-support`'s `scratch!`).
2. A page may declare `continues PAGE` (§6). It then uses the scratch folder that `PAGE` left behind, as it stands
   at the end of `PAGE`. Continuation is a chain: the continued page must come earlier in the run order, and it must
   itself have been run in this run.
3. Pages in a chain run in order, so chapters 02 to 05 build the example world one step at a time.

---

## 6. Directives

A directive is an HTML comment whose content starts with `guide:`:

```text
<!-- guide: <directive> <arguments> -->
```

Directives are invisible in GitHub and in every Markdown renderer. The runner reads them. A directive the runner
does not know fails the page, naming it.

| Directive | Scope | Effect |
| --- | --- | --- |
| `continues PAGE` | page; must appear before the first block | Reuse `PAGE`'s scratch folder (§5). `PAGE` is a file name in the guide root, without `.md`. |
| `requires FEATURE` | the next block (§8) | The block depends on `FEATURE`. If `FEATURE` is `false` in `features.toml`, the block is pending. |
| `operator-only` | the next block | The block needs a live model or a display. It is never run in CI; it is listed in every run. |
| `equals PATH` | the next block; the block must be a command block or a file block | After the block, the scratch folder equals `PATH` file by file (§6.1). `PATH` is relative to the repository root. |
| `copy SOURCE DEST` | standalone, between blocks | Copy the folder `SOURCE` to `DEST` inside the scratch folder, recursively. Both are confined as §3.2. The page shows the per-OS commands as text; the runner does the copy. |
| `packs DIR` | the next command block | Each command in the next block receives `--packs DIR` appended after its arguments. `DIR` is relative to the scratch folder and confined as §3.2. The page does not show the flag. |

Rules:

1. A directive names one target. A directive that is followed by no block where its scope needs one fails the page.
2. Two directives of the same kind on one block fail the page.
3. `equals` compares the scratch folder with `PATH` file by file, byte for byte. It checks both the set of files and
   their contents; an extra file in either folder is a difference. A difference fails the page and names the first
   differing file.

### 6.1 The example equality

`equals docs/guide/examples/corner-shop` (the form the real guide uses) means: the scratch folder, as it stands at
that point in the page, equals the committed folder. A change to a chapter that is not mirrored in the example, or
the reverse, is red.

---

## 7. Mirror parity

For each English page `P` with mirror `P.zh-CN.md`, the runner checks, and fails naming the page and the first
difference:

1. **Steps.** The step headings match in count and in order. A step heading is a level-3 heading of the form
   `### Step N.M — title` (the dash is an em dash, U+2014). The sequence of `N.M` values in the mirror equals the
   sequence in the English page.
2. **Blocks.** The fenced blocks match in count and in order, and each block matches byte for byte: the same info
   string and the same content, including its line endings.
3. **Directives.** The directives match in count, in order and in text.
4. **Links.** Every relative link in the mirror resolves to an existing file (or, for a `#` anchor, to an existing
   heading in the target file). The same rule as §8's link check applies to English pages.

Prose may differ only by translation. A mirror cannot add, remove or change a block, a directive or a step, so it
cannot relax or amend a requirement (`CLAUDE.md` §2.1).

---

## 8. Requirements, pending blocks and links

### 8.1 Requirements

`requires FEATURE` names a key of `features.toml` (§9). An unknown name fails the page.

1. A block under `requires X` with `X = true` is run and must pass.
2. A block under `requires X` with `X = false` is **pending**. It is listed by page, step and feature in the run's
   output and counted in the pending total. It is not run and is never reported as passed.
3. **A pending block makes the rest of its chain pending.** Every later executed block on the same page, and every
   page that continues the same scratch folder, is pending with the same feature, because it depends on the state the
   pending block would have written. The report names the first feature.
4. `operator-only` blocks are listed as pending with the reason `operator-only` in every run. They are not counted as
   passed, and the operator runs them at milestone checks.

### 8.2 Reference links

A link in an English page or a mirror whose destination, resolved against the page's own folder, points into the
reference directory must name an anchor. The anchor is a `#slug`. A slug is the heading text lowercased, with
characters other than letters, digits, hyphens and spaces removed, and each space turned into a hyphen. The anchor
must match a heading slug in the target file. A missing target file or a missing anchor fails the run, naming the
page, the link and the target.

Until the reference directory exists, the runner is given no reference directory, and the link check reports
`reference links: not checked` as a pending item. From PR 25a onwards the real test always passes the directory, and
the check is mandatory.

### 8.3 Other links

A relative link in an English page must resolve to an existing file. `http` and `https` links are not checked by the
runner (no network in tests).

### 8.4 No stray commands

A `console` command whose first word is not `mineworld` fails the page (§4.2). This is the rule that keeps every page
free of shell dependence.

---

## 9. Feature flags: `features.toml`

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

1. The file is TOML, read with the `toml` crate (`DEP-50`). Its only table is `[features]`. Every entry is a key
   of lowercase letters, digits and hyphens, with a boolean value. A trailing comment after `#` is allowed. Any other
   table, key or value type fails the run, naming the line.
2. A feature the page names with `requires` must be present in this file (§8.1).
3. **Presence check.** A feature whose flag is `false` must not be present in the build. The runner asks the binary,
   and the probe named in this file is the only check. A feature flagged `false` that the build does have fails the
   run, naming the feature and the probe's evidence. So a flag cannot be forgotten when the feature lands.
   The probe is recorded in §9.1 when PR S25c C3 lands.
4. `true` means the feature is merged and its blocks must pass.

### 9.1 The presence probe

**Recorded when C3 lands** (see the PR's evidence). Until then the presence check is `INCONCLUSIVE` and is reported
as such, never as a pass.

---

## 10. Page rules

These are the author's rules for every page. The runner enforces the parts marked ✓ and no others.

1. **One action per step.** A step that needs two files is two steps. ✓ (at most one `file=` block per step).
2. **Whole files.** Every "Do this" that changes a file shows the whole file, never "add these lines". A file block
   replaces the file. ✓
3. **Every step ends with a command and its expected output**, given as "You should see". ✓ (every step has a
   `console` block).
4. **Output is quoted exactly**, with `...` for lines a reader need not compare and `[..]` inside a line for a value
   that legitimately changes (a version). ✓ (snapbox matching, §4.4).
5. **Executed commands use only `mineworld …`**, forward-slash relative paths and no shell syntax, so one block runs on
   all three operating systems. ✓ (§4.2).
6. **When an operating system differs** (installing, copying a folder), the page shows the three variants as text that
   is not executed, and the runner's directive does the same thing (`copy`, §6). ✓ (a `copy` directive and a shown
   block).
7. **Plain language.** Sentences of about 25 words at most. A defined term (`World Pack`, `System Pack`, `Place`,
   `seat`, and the others in `CORE_CONCEPTS.md`) is explained in plain words at first use on each page, linked to
   `CORE_CONCEPTS.md`, and then used exactly. No decision ids, plan names or internal history in the body; a "Why it
   works this way" footer per chapter may link specifications. Not checked by the runner.
8. **A rule is never stated as the guide's own.** A guide says "a doorway must name a declared place" and links the
   reference entry that says so. It never states a field, a bound or a default as its own rule. Not checked by the
   runner; reviewed in every guide PR.

---

## 11. Reporting

For each run the runner prints, in order:

1. Each page: blocks run, blocks passed, blocks pending (with the page, the step and the feature), blocks failed.
2. Each failure: the page, the step heading it sits under, the block's first line, and the expected and actual
   output (or the refusal reason). A writer must be able to find and fix the defect from the message alone.
3. The pending total and the list of pending features by name.
4. The result of each guard (mirror parity, `equals`, links, features versus build), each as `PASS`, `FAIL` or
   `INCONCLUSIVE`, derived from what was actually checked.

A run is green only when every guard is `PASS` and no block failed. Pending items never make a run green by being
counted as passed; they are reported next to the total.

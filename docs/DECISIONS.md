# MineWorld decision log

**Status:** durable, project-level decisions; one record per file since the operator ruling of
2026-10-18
**Audience:** coding agents and contributors.

This log holds decisions that outlive any single effort — dependencies, build-versus-adopt
calls, and architectural routes. It exists so that a future contributor does not replace or
duplicate something without knowing why it is there
([`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12).

Effort-scoped planning decisions (`D-1`, `D-2`, …) stay in that effort's plan under
`.structured-coding/plans/<effort>/`. Decisions here are referenced by id (`DEP-n`, `ARC-n`) and
are never silently reversed: superseding one means adding a new record that says so.

Record format is deliberately short: problem, options, choice, why not ourselves (or why not
them), the interface that isolates it, and the limitation we accepted.

## 1 Where the records are

Every record is its own file, [`decisions/<ID>.md`](decisions/), and
[`decisions/INDEX.md`](decisions/INDEX.md) lists them all by kind and number, with titles. This file
holds no record. A citation written as "`DECISIONS.md` ARC-48" — in a specification, a plan, a code
comment or a commit — means [`decisions/ARC-48.md`](decisions/ARC-48.md); the hundreds of existing
citations in that form stay valid and are not rewritten.

Until 2026-10-18 every record lived in this one file, and almost every pull request appended to it,
so almost every merge conflicted here. The split moved each record's text unchanged
(`scripts/split_decisions.py`; PR design
`.structured-coding/plans/mvp0/pr-dsplit-decisions.md`).

## 2 The rules of a record file

1. A record is `docs/decisions/<KIND>-<n>.md`, `KIND` being `ARC` (an architectural route) or `DEP`
   (a dependency, or the decision to build our own instead of adopting one), `n` written without a
   leading zero.
2. Its first line is `## <KIND>-<n> — <title>`, with that same identifier. The index takes the title
   from it.
3. A later dated note on a record is appended to that record's file, after a `---` rule, under the
   heading `## <KIND>-<n> note — <what> (<date>, <PR>)`. It is never a file of its own and never a
   section of another record's file. In the old log a note sat wherever it was written, apart from
   its record; the split joined each note to its record in the order the log held them. That is why
   ARC-48 (four notes), ARC-53, ARC-54 and DEP-29 (one each) showed two headings under one id in the
   old file: they were records with notes, never two decisions, and they are one file each.
4. A record migrated on 2026-10-18 keeps its bytes, so its relative links are written relative to
   `docs/`, where the log lived (`MODULE_SPEC.md` means `docs/MODULE_SPEC.md`). A record written
   after the split links relative to its own directory (`../MODULE_SPEC.md`).
5. [`decisions/INDEX.md`](decisions/INDEX.md) is generated, never hand-edited:
   `python3 scripts/check_decision_ids.py --write-index`. The same script without the flag runs in
   CI's fast and docs layers and fails when a file breaks rules 1–3, when this file holds a record
   heading again, or when the index is stale.

## 3 Adding a record

A new decision is one new file, `docs/decisions/<KIND>-<n>.md`, taking the next free number of its
kind (the highest in [`decisions/INDEX.md`](decisions/INDEX.md) on current `main`, plus one), plus
the regenerated index in the same commit. No pull request edits a shared log any more, so two
pull requests that record different decisions no longer conflict. Two that pick the same number
now collide as an add/add conflict on that file — the collision is loud, where the old file let it
merge silently. Whoever merges second renumbers their record to the next free number, moves every
citation of it in the same commit, and regenerates the index. An amendment to an existing record
is a dated note appended to its file (§2 rule 3) and leaves the index unchanged. The index holds
one row per record and nothing that depends on another row, and `.gitattributes` merges it with
Git's built-in `union` driver, so a local `git merge origin/main` keeps both sides' new rows; if the
rows land out of order the check says the index is stale, and `--write-index` fixes it.

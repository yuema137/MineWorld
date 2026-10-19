# PR DSPLIT — The decision log, one record per file

## DESIGN FROZEN — operator ruling 2026-10-18

```text
Design revision:        revision 1 (2026-10-18)
Approved by / evidence: the operator, 2026-10-18, in the primary session: "拆开" (split it), relayed to
                        this session with the requirements of §2. The details are the implementer's
                        within the ruling (§3)
Implementation base:    origin/main 96b41db0 (Merge pull request #199)
Execution contract:     §6
Lifecycle:              FROZEN
```

Scope (§2), the layout and rules (§3), acceptance (§5) and the commit plan (§6) are frozen. Progress,
evidence and bounded corrections stay writable (§7).

## 1 Problem

`docs/DECISIONS.md` was one 9 478-line file holding every `ARC-` and `DEP-` record. Almost every pull
request appends a record or a dated note to it, so almost every merge conflicts on it, and a conflict
there blocks a PR whose real change has nothing to do with any other PR. The file also allowed two
branches to allocate the same id without a textual conflict (the reason `scripts/check_decision_ids.py`
exists).

## 2 Scope (the ruling's requirements)

1. `docs/decisions/<ID>.md`, one record per file; each record's original text preserved byte-for-byte,
   including its later dated notes. The duplicate headings in the old file get a defined resolution
   that changes no content.
2. A generated index, sorted by kind and id, with titles. CI's fast layer fails when it is stale, and
   `check_decision_ids.py` checks the records across files.
3. `docs/DECISIONS.md` stays, as a short specification stating the rules and linking the directory, so
   existing links and the citation form "`DECISIONS.md` ARC-n" keep working. `CLAUDE.md`'s link list
   and the other references that describe *where a record is written* are updated, minimally.
4. A new record is one new file; a one-paragraph contributor note says so.
5. A migration recipe for open PRs that edit `DECISIONS.md`, in the PR body.
6. Fast-layer scripts run locally, and CI.

Out of scope: rewriting any record's text, including its relative links (§3.5); rewriting the ~500
existing citations of `DECISIONS.md` in code, specs and plans (they stay valid, §3.4); renumbering any
id; changing CI workflows (the existing `fast` and `docs` layers already run `check_decision_ids.py`).

## 3 Design

### 3.1 Layout and naming

- `docs/decisions/ARC-<n>.md` and `docs/decisions/DEP-<n>.md`: the exact id as the file name, no zero
  padding, no leading zero (so `ARC-07.md` cannot be a second spelling of `ARC-7`).
- A file's first line is the record's own heading, `## <ID> — <title>`, unchanged from the log (the
  level-2 heading is kept because byte preservation forbids rewriting it to level 1).
- `docs/decisions/INDEX.md`: the generated index (§3.3). `INDEX.md` rather than `README.md`, because
  `CLAUDE.md` §2.1 reserves `README.md` for short human orientation and a 167-row table is an agent
  reference; the human entry point is the `docs/DECISIONS.md` stub and `docs/README.md`'s row.

### 3.2 The split, and the duplicate headings

The split is mechanical: `scripts/split_decisions.py split --source docs/DECISIONS.md --out
docs/decisions`. A section starts at a level-2 heading outside a code fence; it is a record
(`## ARC-48 — …`) or a note (`## ARC-48 note — …`), and any other level-2 heading is refused. The
trailing blank lines and `---` rule that separate sections are layout of the log, not part of the
section. Before writing, the script rebuilds the original file byte-for-byte from the preamble, every
section and every separator, and refuses on any difference.

**Duplicate headings.** ARC-48, ARC-53, ARC-54 and DEP-29 each appear twice as a heading prefix in the
old file. Audit: none is two decisions. Every second occurrence is a dated note on the record —
`ARC-48 note —` ×4, `ARC-53 note —`, `ARC-54 note —`, `DEP-29 note —` — which the old checker already
did not count as an id. **Resolution: merged notes.** Each note is appended to its record's file in the
order the log held them, joined by the log's own separator `\n---\n\n`. No content changes; the only
thing that moves is a note's position relative to unrelated records. Rejected: a suffixed file per note
(`ARC-48.note-1.md`), because a reader of ARC-48 must see its amendments, and a file per note would
re-create the scattering that made them hard to find.

### 3.3 The generated index and the checker

`scripts/check_decision_ids.py` is rewritten to read `docs/decisions/`:

1. every file other than `INDEX.md` is named `<ARC|DEP>-<n>.md`;
2. its first line is `## <same id> — <title>`;
3. every other decision heading in it is `## <same id> note — …`; a heading for another id is a record
   in the wrong file, a second `## <same id> — …` is a duplicate;
4. `docs/DECISIONS.md` holds no decision heading (catches a pre-split branch that kept its monolith
   during a merge);
5. `INDEX.md` equals what `--write-index` renders.

Uniqueness across files follows from 1–3: an id names exactly one file, and that file's first heading
must be that id. `--write-index` regenerates the index, then checks. The index has one row per record —
id link and title, sorted by kind (`ARC`, then `DEP`) and by number — and nothing that depends on other
rows (no count, no note tally), so two branches adding different records touch disjoint lines.
`.gitattributes` sets `merge=union` on `INDEX.md`, so a local `git merge origin/main` keeps both sides'
rows; if their order comes out wrong, rule 5 reports it stale and `--write-index` fixes it.

CI: no workflow change. `check_decision_ids.py` already runs in `fast` and `docs`
(`scripts/ci_layer.py`), is pure standard-library Python, and `docs/decisions/` is under the audited
docs set `docs/` (`scripts/ci_changes.py`), so a record-only PR stays docs-only.

### 3.4 Links

- `docs/DECISIONS.md` becomes the stub specification: what the log is (the old preamble, kept), §1
  where the records are and how "`DECISIONS.md` ARC-n" citations resolve, §2 the rules of a record
  file, §3 the contributor note for adding a record.
- `CLAUDE.md`: the authoritative-spec list links `docs/DECISIONS.md` *and* `docs/decisions/`; §4 item 16
  says the record goes in `docs/decisions/` (rules in `DECISIONS.md`). Nothing else in `CLAUDE.md`.
- `docs/README.md`, `docs/REUSE_POLICY.md`, `.structured-coding/standards.md` (the review convention
  naming where a dependency record is written): where they say *where to write* a record, they name
  the directory. Citations elsewhere are left as they are.

### 3.5 Bounded discovery: relative links inside records

The records were written in `docs/`, so their 164 relative links (`[MODULE_SPEC.md](MODULE_SPEC.md)`)
resolve against `docs/`, not `docs/decisions/`. Byte preservation (requirement 1) forbids rewriting
them in this PR. Resolution: the stub states that a migrated record's relative links are relative to
`docs/`, and that new records link relative to their own directory (`../MODULE_SPEC.md`). A mechanical
link rewrite is a possible follow-up if the operator wants it; it would be one script run and is not
done here.

### 3.6 Migration of open PRs

`scripts/split_decisions.py migrate --base <merge-base log> --theirs <branch log> --out docs/decisions`
writes every record whose text differs between the branch's merge base and the branch, and refuses (and
names) a record main also changed since the fork, or one the branch deleted, for a person to merge. The
exact recipe is in the PR body; the script stays until every pre-split branch is migrated, then a later
PR may delete it.

## 4 Invariants

- I-1 Record count before = after: 167 ids (old checker) = 167 files.
- I-2 Every section of the old file appears byte-for-byte in exactly one file, in log order.
- I-3 No id renumbered, no text reworded.

## 5 Acceptance

- AC-1 `split` rebuilds the old file byte-for-byte from its pieces (built into the script) and an
  independent re-parse (different regex, no fence handling) finds 174 sections, 167 records, 0
  mismatches against the files.
- AC-2 `check_decision_ids.py` → "167 decision ids, all distinct; index current"; `check_doc_headings.py`
  and `ci_changes.py --self-test` pass.
- AC-3 The checker fails on: a stale index; a record heading in `docs/DECISIONS.md`.
- AC-4 `migrate` run against three open branches (#146, #189, #254) writes only the records each
  branch changed, and reports no clash.
- AC-5 CI green on the exact PR head.

## 6 Execution contract and commit plan

Branch `docs/decisions-split` from `origin/main`, worktree
`/Users/yuema137/mineworld-worktrees/decisions-split`. Commit and push authorized; merge is not.

- C1 `docs: split the decision log into one file per record`
  - [x] Implementation: `scripts/split_decisions.py`, the 167 files, `INDEX.md`, the rewritten
    `check_decision_ids.py`, the stub, `.gitattributes`, `CLAUDE.md` and the reference edits, this design.
  - [x] Deterministic validation: AC-1 … AC-4 (§7).
  - [x] LLM logic review: §7.

## 7 Evidence

- AC-1: `split` → "174 sections (167 records, 7 notes) -> 167 files"; independent re-parse →
  "sections=174 records=167 files=167 mismatches=0", 748 389 section bytes. PASS.
- AC-2: `check_decision_ids.py` → "167 decision ids, all distinct; index current";
  `check_doc_headings.py` → "241 numbered sections across 209 documents, none duplicated";
  `ci_changes.py --self-test` → 30 cases passed. PASS.
- AC-3: with the old monolith still in place the checker named every record heading in
  `docs/DECISIONS.md` and the missing index, rc 1. PASS.
- AC-4: #146 → ARC-80, DEP-38, DEP-41; #254 → 8 records incl. new ARC-50 and the DEP-20 note; #189 →
  7 amended records; 0 clashes each. PASS.
- AC-5: head 484e0631: attempt 1 red only on the `walking_pace` live-rescale test (unrelated to this
  diff, which has no Rust; passed on main run 39187475169); failed jobs rerun once, all green. Ticketed
  separately by the coordinator.
- Merge of `origin/main` b85f2804 (#260 appended ARC-150 to the old log): `migrate --base <96b41db0's
  log> --theirs <b85f2804's log>` wrote `ARC-150.md`, 0 clashes; the stub kept; index regenerated.
  Independent re-parse of b85f2804's log: 175 sections, 168 records, 168 files, 0 mismatches.
  `check_decision_ids.py` → 168 ids. Final-head CI: in the PR.
- Review: the checker's rules map 1:1 to the stub's §2; the index has no cross-row content (union-safe);
  notes ordered as in the log; Python 3.9-compatible (local interpreter), standard library only.

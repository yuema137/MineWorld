# vis-2d — handoff

**Branch** `vis/2d-generated-assets` · **Audience** coding agents

A continuation aid, not a design authority. This effort ran conversationally
rather than through overall/step/PR documents, which
[`CLAUDE.md`](../../../CLAUDE.md) §3.4 permits for work of this shape; the
durable decisions it produced live in
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md), and the runnable artefact
and its launch commands are documented in
[`clients/2d-spike/README.md`](../../../clients/2d-spike/README.md).

## State

The 2D default style is settled: `ARC-14` records the operator's selection of
`town`. Four art variants remain behind `--variant=`, because the other three
are the demonstration that the presentation layer swaps. `--drive` is the
numeric harness, runs headless in about six seconds, and exits non-zero on
failure; `--shots` and `--drive --capture` need a window.

## Open observation: `DECISIONS.md` accepts duplicate ids silently

Recorded because nothing in the repository would have caught it, and it will
recur.

The default-2D-style entry and the 3D lighting reversal were written on the
same day from branches that had both been cut from a `main` whose last entry
was `ARC-12`. Both allocated the next free id and both became `ARC-13`. The two
sections land in different regions of the file, so `git merge` reports no
conflict: `main` would simply have ended up with two `## ARC-13` headings, and
every cross-reference to either would have been ambiguous. It was caught by a
human noticing, which is not a control.

This is the same failure `ARC-12` already records for names, one level up: the
log is only authoritative if its identifiers are unique, and nothing enforces
that.

**Proposed, deliberately not built here.** A uniqueness check over
`^## (DEP|ARC)-\d+` headings in `docs/DECISIONS.md`, run in CI, failing on a
repeated id. It is a few lines, it needs no new dependency, and it would have
turned a silent merge into a red build. It belongs to `main` and to whoever
owns the CI configuration, not to a visual branch — raising it here rather than
implementing it is the point.

Worth considering alongside it, and also not decided here: whether concurrent
branches should reserve ids up front, or whether ids should be allocated at
merge rather than at write. A checker catches the collision; neither of those
would let it happen. That is a process question above this branch.

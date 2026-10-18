"""AP4-12: FTS5 exists in this platform's interpreter, and the query builder keeps words as data (P4-4).

CI runs this on Linux, macOS and Windows (`python` job). The platform line printed here is the per-leg
evidence the design's ledger records (§14.4): the interpreter, its SQLite, and whether FTS5 is present.
"""

from __future__ import annotations

import platform
import sqlite3
import sys

from mineworld_cognition.memory import fts


def test_fts5_is_present_on_this_platform() -> None:
    # If a leg lacks FTS5, the LIKE fallback is that leg's path; the design records it as a deviation
    # (pr-s10-p4 D-P4-12, R-P4-1) and this assertion is relaxed only through that record.
    print(
        f"\n[fts5] {sys.platform} {platform.machine()} CPython {platform.python_version()} "
        f"SQLite {sqlite3.sqlite_version} FTS5={fts.available()}"
    )
    assert fts.available()


def test_every_hostile_term_is_matched_as_literal_text() -> None:
    # P4-4: each term is one quoted string; FTS5 parses it as a phrase, never as an operator.
    connection = sqlite3.connect(":memory:")
    try:
        connection.execute(f"CREATE VIRTUAL TABLE t USING fts5(gist, tokenize='{fts.TOKENIZER}')")
        connection.execute("INSERT INTO t (gist) VALUES ('Bob said: the umbrella is near')")
        hostile = 'umbrella" OR "x NEAR(a b) gist:foo * -x AND not'
        matched: list[str] = []
        for term in fts.query_terms(hostile):
            rows = connection.execute(
                "SELECT count(*) FROM t WHERE t MATCH ?", (fts.fts_string(term),)
            ).fetchone()
            if rows[0]:
                matched.append(term)
    finally:
        connection.close()
    assert fts.query_terms(hostile) == ("umbrella", "near", "gist", "foo", "and", "not")
    assert matched == ["umbrella", "near"]


def test_a_quote_inside_a_term_is_doubled() -> None:
    assert fts.fts_string('a"b') == '"a""b"'
    assert fts.like_pattern("50%_\\") == "%50\\%\\_\\\\%"

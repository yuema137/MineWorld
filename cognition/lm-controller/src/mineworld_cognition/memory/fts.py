"""Lexical matching: SQLite FTS5 when the interpreter has it, `LIKE` otherwise (D-P4-11, D-P4-12; DEP-37).

FTS5 only decides **which** records match. Its `bm25()` score is a float whose value may differ across
SQLite versions, so it is never read; ranking is retrieval's integer rule. A trigger's words are data,
never query syntax (P4-4): each term becomes one quoted FTS5 string, with `"` doubled, so `NEAR`, `*`,
`-`, column filters and boolean operators in a player's words are matched as text or not at all.
"""

from __future__ import annotations

import re
import sqlite3
import unicodedata
from functools import cache

TOKENIZER = "unicode61 remove_diacritics 2"
MIN_TERM_LENGTH = 3
MAX_TERMS = 8
_WORD = re.compile(r"[^\W_]+")
"""A run of Unicode letters and digits: what is left after splitting on everything else."""


@cache
def available() -> bool:
    """Whether this interpreter's SQLite has FTS5, decided once per process on a throwaway table."""
    connection = sqlite3.connect(":memory:")
    try:
        connection.execute(f"CREATE VIRTUAL TABLE probe USING fts5(text, tokenize='{TOKENIZER}')")
    except sqlite3.OperationalError:
        return False
    finally:
        connection.close()
    return True


def query_terms(words: str) -> tuple[str, ...]:
    """The terms of a trigger: NFKC-normalized, lower-cased, split on non-alphanumerics, at least three
    characters long, the first eight distinct, in order of appearance."""
    terms: list[str] = []
    for term in _WORD.findall(unicodedata.normalize("NFKC", words).lower()):
        if len(term) >= MIN_TERM_LENGTH and term not in terms:
            terms.append(term)
            if len(terms) == MAX_TERMS:
                break
    return tuple(terms)


def fts_string(term: str) -> str:
    """One term as an FTS5 string literal: a phrase that is only text, whatever it contains."""
    return '"' + term.replace('"', '""') + '"'


def like_pattern(term: str) -> str:
    """One term as a `LIKE … ESCAPE '\\'` pattern that matches it anywhere, literally."""
    escaped = term.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_")
    return f"%{escaped}%"

"""The positional canonical line (pr-s28a §5.4; `ARC-101`): the grammar the bound and determinism are defined on.

Each test writes a row through the seat view's own table definition and reads the line back as JSON, so a
column that changes codec, order or name changes what these tests see.
"""

from __future__ import annotations

import json

from mineworld_cognition.memory.home import schema
from mineworld_cognition.memory.home.encoding import line, line_bytes, raw

U64_MAX = "18446744073709551615"
DIGEST = "ab" * 32


def _l0(event: str, gist: str, place: str | None) -> str:
    cells = (event, 3600, "spoke", place, "notable", 1, gist, DIGEST, 650)
    return line("l0", cells, schema.L0.codecs)


def test_an_id_is_its_unpadded_number_up_to_u64_and_a_digest_its_first_128_bits() -> None:
    written = _l0(U64_MAX, "Bob said hello", "5000000001")
    assert written == (
        f'["l0",{U64_MAX},3600,"spoke","5000000001","notable",1,"Bob said hello","{"ab" * 16}",650]'
    )
    assert json.loads(written)[1] == int(U64_MAX)


def test_every_key_column_is_a_number_and_no_other_column_is() -> None:
    # A `_key` suffix does not make a key: `prose_key` is a cassette key and stays text.
    numbers = {
        (table.name, column)
        for table in schema.VIEW_TABLES
        for column, codec in zip(table.columns, table.codecs, strict=True)
        if codec("00000000000000000042") == 42
    }
    assert numbers == {
        ("l0", "event_id"),
        ("l0_who", "event_id"),
        ("names", "source_key"),
        ("episodes", "first_key"),
        ("episodes", "last_key"),
        ("chapters", "first_key"),
        ("chapters", "last_key"),
        ("chapter_ranges", "first_key"),
        ("chapter_ranges", "last_key"),
        ("stable", "first_key"),
        ("stable", "last_key"),
        ("stable", "level_key"),
        ("forgotten", "first_key"),
        ("forgotten", "last_key"),
        ("retention_passes", "trigger"),
    }


def test_meta_values_are_numbers_only_for_the_cursor_and_the_keys_and_null_while_unset() -> None:
    def meta(key: str, value: str) -> object:
        loaded: list[object] = json.loads(line("meta", (key, value), (raw, schema.meta_codec(key))))
        return loaded[2]

    assert meta("cursor", "120") == 120
    assert meta("newest", "00000000000000000120") == 120
    assert meta("forgotten_below", "") is None
    assert meta("retention_day", "3") == "3"
    assert meta("schema", "5") == "5"
    assert meta("seat", "alice") == "alice"


def test_null_text_and_non_ascii_are_written_as_they_are_and_counted_in_bytes() -> None:
    gist = "Bob said: «café au 雨»"
    written = _l0("12", gist, None)
    assert json.loads(written)[3:7] == ["spoke", None, "notable", 1]
    assert gist in written  # not \u-escaped
    assert " " not in written.replace(gist, "")
    cells = ("12", 3600, "spoke", None, "notable", 1, gist, DIGEST, 650)
    assert line_bytes("l0", cells, schema.L0.codecs) == len(written.encode("utf-8")) + 1

"""AP5-4 and AP5-5: replay is ordered and exact; recording is atomic and complete (D-P5-7)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from support import completion, request, run

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.record import (
    Cassette,
    CassetteFormatError,
    CassetteMiss,
    CassetteRecordError,
    RecordingBackend,
    ReplayBackend,
    partial_path,
)


def _record(path: Path, calls: list[CompletionRequest], answers: list[Completion]) -> None:
    """Records `calls` through a scripted backend that answers with `answers` in order."""
    queue = list(answers)
    recorder = RecordingBackend(
        ScriptedBackend(lambda _: queue.pop(0)), path, binding="local", model="test-model"
    )

    async def session() -> None:
        for call in calls:
            await recorder.complete(call)
        await recorder.aclose()

    run(session())


def test_the_same_request_replays_its_completions_in_recorded_order(tmp_path: Path) -> None:
    # AP5-4: two identical requests, two different completions; a third identical call is exhausted.
    path = tmp_path / "twice.jsonl"
    same = request("Hello?")
    _record(path, [same, same], [completion("first"), completion("second")])
    replay = ReplayBackend(Cassette.load(path))

    async def session() -> list[str]:
        texts: list[str] = []
        for _ in range(2):
            answer = await replay.complete(same)
            assert isinstance(answer, Completion)
            texts.append(answer.text)
        with pytest.raises(CassetteMiss) as raised:
            await replay.complete(same)
        assert raised.value.kind == "exhausted"
        return texts

    assert run(session()) == ["first", "second"]


def test_an_absent_request_names_its_key_the_nearest_request_and_the_first_difference(
    tmp_path: Path,
) -> None:
    path = tmp_path / "one.jsonl"
    _record(path, [request("Hello?")], [completion("hi")])
    replay = ReplayBackend(Cassette.load(path))
    asked = request("Hello!")

    async def session() -> CassetteMiss:
        with pytest.raises(CassetteMiss) as raised:
            await replay.complete(asked)
        return raised.value

    miss = run(session())
    assert miss.kind == "absent"
    assert miss.first_difference == "$.messages[0].text"
    message = str(miss)
    assert cassette_key(asked) in message
    assert "$.messages[0].text" in message
    assert "Hello?" in message


def test_an_edited_request_is_refused_at_load_naming_the_line(tmp_path: Path) -> None:
    # AP5-4: a hand edit that changes the request but not the key fails loudly.
    path = tmp_path / "edited.jsonl"
    _record(path, [request("one"), request("two")], [completion("1"), completion("2")])
    lines = path.read_text(encoding="utf-8").split("\n")
    entry = json.loads(lines[2])
    entry["request"]["messages"][0]["text"] = "TWO"
    lines[2] = json.dumps(entry)
    path.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    with pytest.raises(CassetteFormatError, match="line 3") as raised:
        Cassette.load(path)
    assert "does not match its request" in str(raised.value)


@pytest.mark.parametrize(
    ("first_line", "why"),
    [
        ('{"cassette": "mineworld-cognition", "format": 2, "key_scheme": 1}', "format 2"),
        ('{"cassette": "mineworld-cognition", "format": 1, "key_scheme": 2}', "key scheme 2"),
        ('{"key": "abc"}', "header"),
        ("not json", "header line is not JSON"),
    ],
)
def test_a_bad_header_is_refused_by_name(tmp_path: Path, first_line: str, why: str) -> None:
    path = tmp_path / "bad.jsonl"
    path.write_text(first_line + "\n", encoding="utf-8", newline="\n")
    with pytest.raises(CassetteFormatError, match="line 1") as raised:
        Cassette.load(path)
    assert why in str(raised.value)


def test_a_corrupt_line_is_refused_with_its_number(tmp_path: Path) -> None:
    path = tmp_path / "corrupt.jsonl"
    _record(path, [request("one")], [completion("1")])
    with path.open("a", encoding="utf-8", newline="\n") as file:
        file.write("{not an entry\n")
    with pytest.raises(CassetteFormatError, match="line 3"):
        Cassette.load(path)


def test_a_crlf_checkout_still_loads(tmp_path: Path) -> None:
    # D-P5-13: a Windows checkout that ignored .gitattributes converts LF to CRLF.
    path = tmp_path / "crlf.jsonl"
    _record(path, [request("one")], [completion("1")])
    text = path.read_text(encoding="utf-8")
    path.write_bytes(text.replace("\n", "\r\n").encode())
    assert len(Cassette.load(path).entries) == 1


def test_a_clean_recording_has_the_header_and_every_entry_and_replays(tmp_path: Path) -> None:
    # AP5-5, the clean half. Written with LF on every platform (AP5-12).
    path = tmp_path / "three.jsonl"
    calls = [request("a"), request("b"), request("c")]
    _record(path, calls, [completion("A"), completion("B"), completion("C")])
    raw = path.read_bytes()
    assert b"\r\n" not in raw
    lines = raw.decode().split("\n")
    assert json.loads(lines[0]) == {"cassette": "mineworld-cognition", "format": 1, "key_scheme": 1}
    assert len([line for line in lines[1:] if line]) == 3
    assert not partial_path(path).exists()
    replay = ReplayBackend(Cassette.load(path))

    async def session() -> list[str]:
        texts: list[str] = []
        for call in calls:
            answer = await replay.complete(call)
            assert isinstance(answer, Completion)
            texts.append(answer.text)
        return texts

    assert run(session()) == ["A", "B", "C"]


def test_an_interrupted_recording_leaves_the_old_cassette_and_a_partial(tmp_path: Path) -> None:
    # AP5-5, the interrupted half: the test raises inside the third call.
    path = tmp_path / "kept.jsonl"
    _record(path, [request("old")], [completion("old")])
    before = path.read_bytes()

    def script(call: CompletionRequest) -> Completion:
        if call.messages[0].text == "c":
            raise RuntimeError("the process dies here")
        return completion(call.messages[0].text.upper())

    recorder = RecordingBackend(ScriptedBackend(script), path, binding="local", model="m")

    async def session() -> None:
        await recorder.complete(request("a"))
        await recorder.complete(request("b"))
        with pytest.raises(RuntimeError, match="dies here"):
            await recorder.complete(request("c"))
        await recorder.aclose()

    run(session())
    assert path.read_bytes() == before
    partial = partial_path(path)
    assert partial.exists()
    assert len(partial.read_text(encoding="utf-8").splitlines()) == 3  # header + two entries
    with pytest.raises(CassetteRecordError, match=r"kept\.jsonl\.partial"):
        RecordingBackend(ScriptedBackend(script), path, binding="local", model="m")

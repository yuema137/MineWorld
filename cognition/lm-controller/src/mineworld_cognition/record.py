"""Cassettes: recorded model outputs, replayed strictly and in order (ARC-58; D-P5-7).

A cassette is JSON Lines, UTF-8, LF. The first line is a header; every later line is one completed call:
its key, the provider-neutral request, the completion, and a closed metadata record. Nothing in it can
hold a URL, a header, a key, `key_env` or a transport failure, because none of these types has such a
field: redaction holds by construction, not by scrubbing.

Replay groups entries by key in file order. The n-th call with a key returns the n-th entry; one more
is `CassetteMiss("exhausted")`; an unknown key is `CassetteMiss("absent")`. A miss is an exception, never
a fallback, and never reaches a backend (P5-1). Recording writes `<name>.jsonl.partial`, flushes every
line, and replaces the cassette only on a clean close, so a crash leaves the old cassette and a visible
`.partial`, which blocks the next recording until someone looks at it.
"""

from __future__ import annotations

import json
import os
import time
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Annotated, Literal, TextIO

from mineworld_sdk.wire.ids import JsonValue
from pydantic import Field, ValidationError, field_validator

from mineworld_cognition.backend.canonical import (
    KEY_SCHEME,
    CassetteKey,
    KeyMaterialError,
    canonical_object,
    cassette_key,
)
from mineworld_cognition.backend.model import (
    BackendFailure,
    CognitionModel,
    Completion,
    CompletionRequest,
    ModelBackend,
)

CASSETTE_KIND = "mineworld-cognition"
CASSETTE_FORMAT = 1
_BINDING = r"^[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?$"


class EntryMeta(CognitionModel):
    """Where an entry came from, as far as a reviewer needs to know: never how to reach it."""

    binding: Annotated[str, Field(pattern=_BINDING, max_length=64)]
    """The operator's backend **name** (for example `local`), never its URL."""
    model: Annotated[str, Field(min_length=1, max_length=200)]
    recorded_at: Annotated[str, Field(pattern=r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")]
    latency_ms: Annotated[int, Field(ge=0)]

    @field_validator("model")
    @classmethod
    def _not_a_url(cls, value: str) -> str:
        if "://" in value or "@" in value:
            raise ValueError("a model name is a name, not a URL")
        return value


class CassetteEntry(CognitionModel):
    key: str
    request: CompletionRequest
    completion: Completion
    meta: EntryMeta


class CassetteFormatError(ValueError):
    """A cassette that cannot be trusted: a bad header, a line that is not an entry, a key that does not
    match its request. Raised at load, naming the file and the line."""

    def __init__(self, path: Path, line: int, why: str) -> None:
        super().__init__(f"{path}, line {line}: {why}")
        self.path = path
        self.line = line


class CassetteRecordError(RuntimeError):
    """Recording cannot start: an interrupted recording left its `.partial` behind."""


MissKind = Literal["absent", "exhausted"]


class CassetteMiss(Exception):
    """Replay was asked something the cassette does not answer. Stops the run (AP5-3)."""

    def __init__(
        self,
        kind: MissKind,
        key: CassetteKey,
        *,
        nearest: CompletionRequest | None = None,
        first_difference: str | None = None,
    ) -> None:
        if kind == "exhausted":
            message = f"cassette exhausted: key {key} was answered as often as it was recorded"
        else:
            message = f"cassette miss: key {key} was never recorded"
            if nearest is None:
                message += "; no recorded request has the same purpose"
            else:
                nearest_json = json.dumps(
                    canonical_object(nearest)["request"], sort_keys=True, ensure_ascii=False
                )
                message += (
                    f"; the nearest recorded request first differs at {first_difference}: "
                    f"{nearest_json}"
                )
        super().__init__(message)
        self.kind: MissKind = kind
        self.key = key
        self.nearest = nearest
        self.first_difference = first_difference


@dataclass(frozen=True)
class Cassette:
    """A loaded cassette: every entry, in file order."""

    path: Path
    entries: tuple[CassetteEntry, ...]

    @staticmethod
    def load(path: Path) -> Cassette:
        """Reads and verifies a cassette. CRLF is tolerated (a Windows checkout without
        `.gitattributes`); every entry's key is recomputed from its request."""
        lines = path.read_text(encoding="utf-8").split("\n")
        if lines and lines[-1] == "":
            lines.pop()
        if not lines:
            raise CassetteFormatError(path, 1, "empty file: the header line is missing")
        _check_header(path, lines[0])
        entries: list[CassetteEntry] = []
        for number, line in enumerate(lines[1:], start=2):
            try:
                entry = CassetteEntry.model_validate_json(line)
            except ValidationError as error:
                raise CassetteFormatError(path, number, f"not a cassette entry: {error}") from None
            try:
                recomputed = cassette_key(entry.request)
            except KeyMaterialError as error:
                raise CassetteFormatError(path, number, str(error)) from None
            if recomputed != entry.key:
                raise CassetteFormatError(
                    path,
                    number,
                    f"key {entry.key} does not match its request (which hashes to {recomputed}); "
                    "an edited request must be re-recorded",
                )
            entries.append(entry)
        return Cassette(path, tuple(entries))


def _check_header(path: Path, line: str) -> None:
    try:
        header: JsonValue = json.loads(line)
    except json.JSONDecodeError:
        raise CassetteFormatError(path, 1, "the header line is not JSON") from None
    if not isinstance(header, dict) or header.get("cassette") != CASSETTE_KIND:
        raise CassetteFormatError(path, 1, f'the first line is not a "{CASSETTE_KIND}" header')
    if header.get("format") != CASSETTE_FORMAT:
        raise CassetteFormatError(
            path,
            1,
            f"cassette format {header.get('format')!r} is not supported (only {CASSETTE_FORMAT})",
        )
    if header.get("key_scheme") != KEY_SCHEME:
        raise CassetteFormatError(
            path, 1, f"key scheme {header.get('key_scheme')!r} is not supported (only {KEY_SCHEME})"
        )


def _header_line() -> str:
    header = {"cassette": CASSETTE_KIND, "format": CASSETTE_FORMAT, "key_scheme": KEY_SCHEME}
    return json.dumps(header, sort_keys=True) + "\n"


def _first_difference(left: JsonValue, right: JsonValue, path: str) -> str | None:
    if isinstance(left, dict) and isinstance(right, dict):
        for name in sorted(set(left) | set(right)):
            if name not in left or name not in right:
                return f"{path}.{name}"
            found = _first_difference(left[name], right[name], f"{path}.{name}")
            if found is not None:
                return found
        return None
    if isinstance(left, list) and isinstance(right, list):
        for index, (a, b) in enumerate(zip(left, right, strict=False)):
            found = _first_difference(a, b, f"{path}[{index}]")
            if found is not None:
                return found
        return None if len(left) == len(right) else f"{path}[{min(len(left), len(right))}]"
    return None if left == right and type(left) is type(right) else path


def _nearest(
    request: CompletionRequest, entries: tuple[CassetteEntry, ...]
) -> tuple[CompletionRequest | None, str | None]:
    """The recorded request with the same purpose and the fewest differing top-level fields (the first
    in file order on a tie), and the first JSON path where it differs from `request`."""
    asked = canonical_object(request)["request"]
    assert isinstance(asked, dict)
    best: tuple[int, CompletionRequest, Mapping[str, JsonValue]] | None = None
    for entry in entries:
        if entry.request.purpose != request.purpose:
            continue
        recorded = canonical_object(entry.request)["request"]
        assert isinstance(recorded, dict)
        differing = sum(1 for name in asked if asked[name] != recorded.get(name))
        if best is None or differing < best[0]:
            best = (differing, entry.request, recorded)
    if best is None:
        return None, None
    return best[1], _first_difference(dict(best[2]), asked, "$")


class ReplayBackend:
    """Answers only from a cassette. It holds no other backend, so a miss cannot fall through to one."""

    def __init__(self, cassette: Cassette) -> None:
        self._cassette = cassette
        self._by_key: dict[str, list[Completion]] = {}
        for entry in cassette.entries:
            self._by_key.setdefault(entry.key, []).append(entry.completion)
        self._served: dict[str, int] = {}

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        key = cassette_key(request)
        recorded = self._by_key.get(key)
        if recorded is None:
            nearest, difference = _nearest(request, self._cassette.entries)
            raise CassetteMiss("absent", key, nearest=nearest, first_difference=difference)
        served = self._served.get(key, 0)
        if served >= len(recorded):
            raise CassetteMiss("exhausted", key)
        self._served[key] = served + 1
        return recorded[served]

    async def aclose(self) -> None:
        return None

    def __repr__(self) -> str:
        return f"ReplayBackend({self._cassette.path.name}, {len(self._cassette.entries)} entries)"


def partial_path(path: Path) -> Path:
    return path.with_name(path.name + ".partial")


class RecordingBackend:
    """Asks `inner`, and appends every completed call to the cassette at `path`.

    A `BackendFailure` is passed through and never recorded. An exception from `inner` abandons the
    recording: the `.partial` stays, the old cassette is untouched, and `aclose` replaces nothing."""

    def __init__(self, inner: ModelBackend, path: Path, *, binding: str, model: str) -> None:
        partial = partial_path(path)
        if partial.exists():
            raise CassetteRecordError(
                f"{partial} exists: an earlier recording was interrupted. Inspect it, then delete it "
                "to record again"
            )
        self._inner = inner
        self._path = path
        self._partial = partial
        self._binding = binding
        self._model = model
        self._file: TextIO | None = partial.open("x", encoding="utf-8", newline="\n")
        self._abandoned = False
        self._file.write(_header_line())
        self._file.flush()

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        key = cassette_key(request)
        started = time.monotonic()
        try:
            answer = await self._inner.complete(request)
        except BaseException:
            self._abandon()
            raise
        if isinstance(answer, BackendFailure):
            return answer
        meta = EntryMeta(
            binding=self._binding,
            model=self._model,
            recorded_at=datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
            latency_ms=int((time.monotonic() - started) * 1000),
        )
        entry = CassetteEntry(key=key, request=request, completion=answer, meta=meta)
        if self._file is None:
            raise CassetteRecordError(f"{self._partial}: the recording is closed")
        self._file.write(
            json.dumps(entry.model_dump(mode="json"), sort_keys=True, ensure_ascii=False)
        )
        self._file.write("\n")
        self._file.flush()
        return answer

    def _abandon(self) -> None:
        self._abandoned = True
        if self._file is not None:
            self._file.close()
            self._file = None

    async def aclose(self) -> None:
        """A clean close: the `.partial` replaces the cassette (after the file is closed, which Windows
        requires), then the inner backend is closed."""
        if self._file is not None:
            self._file.close()
            self._file = None
            if not self._abandoned:
                os.replace(self._partial, self._path)
        await self._inner.aclose()

    def __repr__(self) -> str:
        return f"RecordingBackend({self._path.name}, binding={self._binding!r})"

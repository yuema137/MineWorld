"""The Meshy image-to-3D client, stage 4 (port of `meshy_i23d.py`; PIPELINE.md §3, §11; design §5.11).

Rules (step-26 §5.5, I-B1-2, I-B1-3):

- The key is read from the environment variable `MESHY_API_KEY`, at the moment of a live request, and from
  nowhere else. It is never written to a file, a log line, an exception, or a printed output.
- The request body carries the parameters and the image as a data URI. `request.json` records the endpoint,
  the image's absolute path and SHA-256, the parameters and the date, and never the bytes or the headers.
- An HTTP error body is cut to 2,000 characters and every occurrence of the key's value is replaced by
  `<redacted>` before it is stored or raised.
- A live run is refused before its POST when the credit ledger's spent total plus `CREDITS_PER_RUN` would
  exceed the cap. The ledger is updated as soon as the POST succeeds, so a crash after the POST still counts.
- `dry_run` makes no transport call and reads no key.

The network is reached only through `UrllibTransport`, which no test constructs.
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import time
import urllib.error
import urllib.request
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol, cast

API = "https://api.meshy.ai/openapi/v1"
KEY_ENV = "MESHY_API_KEY"
CREDITS_PER_RUN = 30  # D+ §3: each image-to-3D run cost 30 credits; the measured value is recorded per run
POLL_SECONDS = 15.0  # D+ `meshy_i23d.py`: `time.sleep(15)` between polls
ERROR_BODY_LIMIT = 2000
REDACTED = "<redacted>"
TERMINAL_STATUSES = ("SUCCEEDED", "FAILED", "CANCELED")
# D+ `meshy_i23d.py:46-58`: the defaults, plus the pose the cast needs (`pose_mode a-pose`, step-26 §4.5).
DEFAULT_PARAMS: dict[str, str | bool | list[str]] = {
    "ai_model": "meshy-7.1",
    "should_texture": True,
    "enable_pbr": False,
    "texture_resolution": "2k",
    "should_remesh": False,
    "image_enhancement": False,
    "remove_lighting": True,
    "moderation": False,
    "target_formats": ["glb"],
    "multi_view_thumbnails": True,
    "pose_mode": "a-pose",
}


class MeshyError(Exception):
    """A Meshy request was refused, or it failed. The message never contains the key."""


class Transport(Protocol):
    """The one network seam. `UrllibTransport` is the real one; tests inject a fake."""

    def request(
        self,
        method: str,
        url: str,
        headers: Mapping[str, str],
        body: bytes | None,
        timeout_s: float,
    ) -> tuple[int, bytes]: ...


class UrllibTransport:
    """The real transport. Only this class performs network I/O in the pipeline."""

    def request(
        self,
        method: str,
        url: str,
        headers: Mapping[str, str],
        body: bytes | None,
        timeout_s: float,
    ) -> tuple[int, bytes]:
        req = urllib.request.Request(url, data=body, method=method, headers=dict(headers))
        try:
            with urllib.request.urlopen(req, timeout=timeout_s) as response:  # noqa: S310 — https API only
                return int(response.status), response.read()
        except urllib.error.HTTPError as exc:
            return exc.code, exc.read()


@dataclass(frozen=True)
class Ledger:
    """The credit ledger for one scratch root: credits spent on live runs so far."""

    spent: int
    runs: tuple[str, ...]


def load_ledger(path: Path) -> Ledger:
    if not path.exists():
        return Ledger(spent=0, runs=())
    raw: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise MeshyError(f"{path}: the credit ledger is not an object")
    data = cast(dict[str, object], raw)
    spent = data.get("spent")
    runs = data.get("runs")
    if isinstance(spent, bool) or not isinstance(spent, int) or not isinstance(runs, list):
        raise MeshyError(f"{path}: the credit ledger has no integer 'spent' and a 'runs' list")
    return Ledger(spent=spent, runs=tuple(str(r) for r in cast(list[object], runs)))


def write_ledger(path: Path, ledger: Ledger) -> None:
    payload = {"spent": ledger.spent, "runs": list(ledger.runs)}
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def check_cap(ledger: Ledger, cap: int) -> None:
    """Refuse before any request when the run would exceed the cap."""
    projected = ledger.spent + CREDITS_PER_RUN
    if projected > cap:
        raise MeshyError(
            f"credit cap refused: spent {ledger.spent} + {CREDITS_PER_RUN} for this run = {projected} "
            f"exceeds the cap {cap}; no request was made"
        )


def redact(text: str, key: str) -> str:
    """The text with every occurrence of the key's value replaced, and cut to the error-body limit first."""
    cut = text[:ERROR_BODY_LIMIT]
    if key:
        cut = cut.replace(key, REDACTED)
    return cut


def key_from_environment() -> str:
    key = os.environ.get(KEY_ENV, "")
    if not key:
        raise MeshyError(f"{KEY_ENV} is not set")
    return key


@dataclass(frozen=True)
class RunOutcome:
    """What one run produced: its directory, its task id, the status it ended with, and the files written."""

    run_dir: Path
    task_id: str
    status: str
    consumed_credits: int | None
    files: tuple[str, ...]


@dataclass(frozen=True)
class RunPlan:
    """What a dry run prints: the request without image bytes, and the credit projection."""

    endpoint: str
    image_path: str
    image_sha256: str
    params: Mapping[str, str | bool | list[str]]
    projected_credits: int


def _image_sha256(image: Path) -> str:
    return hashlib.sha256(image.read_bytes()).hexdigest()


def _params_with(overrides: Mapping[str, str | bool | list[str]]) -> dict[str, str | bool | list[str]]:
    params = dict(DEFAULT_PARAMS)
    params.update(overrides)
    return params


def plan(image: Path, overrides: Mapping[str, str | bool | list[str]], ledger: Ledger, cap: int) -> RunPlan:
    """A dry run: the request that would be made, and the projection. Calls nothing and reads no key."""
    check_cap(ledger, cap)
    return RunPlan(
        endpoint=API + "/image-to-3d",
        image_path=str(image.resolve()),
        image_sha256=_image_sha256(image),
        params=_params_with(overrides),
        projected_credits=CREDITS_PER_RUN,
    )


def _json_object(raw: bytes, where: str) -> dict[str, object]:
    parsed: object = json.loads(raw.decode("utf-8"))
    if not isinstance(parsed, dict):
        raise MeshyError(f"{where}: the response is not a JSON object")
    return cast(dict[str, object], parsed)


def _call(
    transport: Transport,
    key: str,
    method: str,
    path: str,
    body: Mapping[str, object] | None,
) -> dict[str, object]:
    headers = {"Authorization": "Bearer " + key, "Content-Type": "application/json"}
    data = json.dumps(body).encode("utf-8") if body is not None else None
    status, raw = transport.request(method, API + path, headers, data, 120.0)
    if status >= 400:
        text = raw.decode("utf-8", errors="replace")
        raise MeshyError(f"Meshy {method} {path} returned HTTP {status}: {redact(text, key)}")
    return _json_object(raw, f"{method} {path}")


def _download(transport: Transport, url: str, destination: Path) -> None:
    status, raw = transport.request("GET", url, {}, None, 300.0)
    if status >= 400:
        raise MeshyError(f"download failed with HTTP {status} for {destination.name}")
    destination.write_bytes(raw)


def _write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def _string_urls(value: object) -> dict[str, str]:
    if not isinstance(value, dict):
        return {}
    return {str(k): str(v) for k, v in cast(dict[str, object], value).items() if isinstance(v, str) and v}


def run_live(
    *,
    image: Path,
    overrides: Mapping[str, str | bool | list[str]],
    run_dir: Path,
    ledger_path: Path,
    cap: int,
    transport: Transport,
    sleep: Callable[[float], None] = time.sleep,
    now: Callable[[], str] = lambda: time.strftime("%Y-%m-%dT%H:%M:%S%z"),
) -> RunOutcome:
    """One live image-to-3D run: create, poll, download. Refuses before the POST when over the cap."""
    ledger = load_ledger(ledger_path)
    check_cap(ledger, cap)
    key = key_from_environment()
    run_dir.mkdir(parents=True, exist_ok=True)
    params = _params_with(overrides)
    _write_json(
        run_dir / "request.json",
        {
            "endpoint": API + "/image-to-3d",
            "image_file": str(image.resolve()),
            "image_sha256": _image_sha256(image),
            "params": params,
            "date": now(),
        },
    )
    balance_before = _call(transport, key, "GET", "/balance", None)
    _write_json(run_dir / "balance_before.json", balance_before)
    uri = "data:image/png;base64," + base64.b64encode(image.read_bytes()).decode("ascii")
    created = _call(transport, key, "POST", "/image-to-3d", {**params, "image_url": uri})
    task_id = created.get("result")
    if not isinstance(task_id, str):
        raise MeshyError("Meshy did not return a task id for the request")
    ledger = Ledger(spent=ledger.spent + CREDITS_PER_RUN, runs=(*ledger.runs, task_id))
    write_ledger(ledger_path, ledger)
    task: dict[str, object] = {}
    while True:
        task = _call(transport, key, "GET", f"/image-to-3d/{task_id}", None)
        if task.get("status") in TERMINAL_STATUSES:
            break
        sleep(POLL_SECONDS)
    _write_json(run_dir / "task.json", task)
    status = str(task.get("status"))
    consumed = task.get("consumed_credits")
    consumed_credits = consumed if isinstance(consumed, int) and not isinstance(consumed, bool) else None
    files: list[str] = []
    if status == "SUCCEEDED":
        for fmt, url in _string_urls(task.get("model_urls")).items():
            name = f"model_{fmt}.{'glb' if 'glb' in fmt else fmt}"
            _download(transport, url, run_dir / name)
            files.append(name)
        textures = task.get("texture_urls")
        if isinstance(textures, list):
            for index, texture in enumerate(cast(list[object], textures)):
                for kind, url in _string_urls(texture).items():
                    name = f"tex{index}_{kind}.png"
                    _download(transport, url, run_dir / name)
                    files.append(name)
        for view, url in _string_urls(task.get("thumbnail_urls")).items():
            name = f"thumb_{view}.png"
            _download(transport, url, run_dir / name)
            files.append(name)
        thumb = task.get("thumbnail_url")
        if isinstance(thumb, str) and thumb:
            _download(transport, thumb, run_dir / "thumb.png")
            files.append("thumb.png")
    balance_after = _call(transport, key, "GET", "/balance", None)
    _write_json(run_dir / "balance_after.json", balance_after)
    return RunOutcome(
        run_dir=run_dir,
        task_id=task_id,
        status=status,
        consumed_credits=consumed_credits,
        files=tuple(files),
    )

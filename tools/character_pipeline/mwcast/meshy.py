"""The Meshy image-to-3D client, stage 4 (port of `meshy_i23d.py`; PIPELINE.md §3, §11; design §5.11).

Rules (step-26 §5.5, I-B1-2, I-B1-3):

- The key is read from the environment variable `MESHY_API_KEY`, at the moment of a live request, and from
  nowhere else. It is never written to a file, a log line, an exception, or a printed output.
- The request body carries the parameters and the image as a data URI. `request.json` records the endpoint,
  the image's absolute path and SHA-256, the parameters and the date, and never the bytes or the headers.
- An HTTP error body is cut to 2,000 characters and every occurrence of the key's value is replaced by
  `<redacted>` before it is stored or raised.
- A live run is refused before its POST when the credit ledger's spent total plus the run's projection
  (`CREDITS_PER_RUN` unless the caller passes a larger one) would exceed the cap. The ledger is updated as
  soon as the POST succeeds, so a crash after the POST still counts.
- With a `charge` hook (S32-p, `pr-s32p-wardrobe-pilot.md` §4.6, F-S32p-1), the run is also written as an
  itemised ledger entry, and after the task ends the hook replaces the provisional charge with the measured
  one (`consumed_credits`, and the balance delta). Without the hook the behaviour is S26-b1's exactly.
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
from dataclasses import asdict, dataclass
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
class LedgerEntry:
    """One itemised run (written only by a run with a `charge` hook; S26-b1's ledger has none).

    `provisional` is what was charged when the POST succeeded; `measured` is the task's `consumed_credits`;
    `balance_delta` is `balance_before - balance_after`; `charged` is what `spent` counts for this run.
    `phase` and `asset` name what the run was admitted for; `finding` is a reconcile note, or empty."""

    task_id: str
    phase: str
    asset: str
    provisional: int
    measured: int | None
    balance_delta: int | None
    charged: int
    finding: str = ""


@dataclass(frozen=True)
class Ledger:
    """The credit ledger for one scratch root: credits spent on live runs so far."""

    spent: int
    runs: tuple[str, ...]
    entries: tuple[LedgerEntry, ...] = ()


def _optional_int(value: object, where: str) -> int | None:
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, int):
        raise MeshyError(f"{where}: expected an integer or null")
    return value


def _entry(raw: object, where: str) -> LedgerEntry:
    if not isinstance(raw, dict):
        raise MeshyError(f"{where}: expected an object")
    data = cast(dict[str, object], raw)
    texts = {k: data.get(k) for k in ("task_id", "phase", "asset", "finding")}
    if not all(isinstance(v, str) for v in texts.values()):
        raise MeshyError(f"{where}: task_id, phase, asset and finding must be strings")
    provisional = _optional_int(data.get("provisional"), where + ".provisional")
    charged = _optional_int(data.get("charged"), where + ".charged")
    if provisional is None or charged is None:
        raise MeshyError(f"{where}: provisional and charged must be integers")
    return LedgerEntry(
        task_id=str(texts["task_id"]),
        phase=str(texts["phase"]),
        asset=str(texts["asset"]),
        provisional=provisional,
        measured=_optional_int(data.get("measured"), where + ".measured"),
        balance_delta=_optional_int(data.get("balance_delta"), where + ".balance_delta"),
        charged=charged,
        finding=str(texts["finding"]),
    )


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
    raw_entries = data.get("entries", [])
    if not isinstance(raw_entries, list):
        raise MeshyError(f"{path}: the credit ledger's 'entries' is not a list")
    entries = tuple(
        _entry(item, f"{path}: entries[{index}]")
        for index, item in enumerate(cast(list[object], raw_entries))
    )
    return Ledger(spent=spent, runs=tuple(str(r) for r in cast(list[object], runs)), entries=entries)


def write_ledger(path: Path, ledger: Ledger) -> None:
    """Write the ledger. `entries` is written only when there are some, so S26-b1's file is unchanged."""
    payload: dict[str, object] = {"spent": ledger.spent, "runs": list(ledger.runs)}
    if ledger.entries:
        payload["entries"] = [asdict(entry) for entry in ledger.entries]
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def check_cap(ledger: Ledger, cap: int, projection: int = CREDITS_PER_RUN) -> None:
    """Refuse before any request when the run would exceed the cap."""
    projected = ledger.spent + projection
    if projected > cap:
        raise MeshyError(
            f"credit cap refused: spent {ledger.spent} + {projection} for this run = {projected} "
            f"exceeds the cap {cap}; no request was made"
        )


# The reconcile hook of a run with itemised entries: given the ledger after the provisional charge, the task
# id, the task's `consumed_credits` and the two balance responses, it returns the reconciled ledger.
Charge = Callable[[Ledger, str, int | None, Mapping[str, object]], Ledger]


def redact(text: str, key: str) -> str:
    """The text with every occurrence of the key's value replaced, and cut to the error-body limit first."""
    cut = text[:ERROR_BODY_LIMIT]
    if key:
        cut = cut.replace(key, REDACTED)
    return cut


def balance(transport: Transport, key: str) -> int | None:
    """The account balance (read-only GET /balance; no credit is spent). None when the response names none."""
    raw = _call(transport, key, "GET", "/balance", None)
    for name in ("balance", "credits"):
        value = raw.get(name)
        if isinstance(value, int):
            return value
    return None


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
    projection: int = CREDITS_PER_RUN,
    charge: Charge | None = None,
    phase: str = "",
    asset: str = "",
) -> RunOutcome:
    """One live image-to-3D run: create, poll, download. Refuses before the POST when over the cap.

    `projection` is what the cap check and the provisional charge count for this run. With `charge`, the run
    is itemised (`phase`, `asset`) and reconciled by the hook after the task ends; the reconciled ledger is
    written before the outcome is returned."""
    ledger = load_ledger(ledger_path)
    check_cap(ledger, cap, projection)
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
    entries = ledger.entries
    if charge is not None:
        provisional = LedgerEntry(
            task_id=task_id,
            phase=phase,
            asset=asset,
            provisional=projection,
            measured=None,
            balance_delta=None,
            charged=projection,
        )
        entries = (*entries, provisional)
    ledger = Ledger(spent=ledger.spent + projection, runs=(*ledger.runs, task_id), entries=entries)
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
    if charge is not None:
        balances: dict[str, object] = {"balance_before": balance_before, "balance_after": balance_after}
        write_ledger(ledger_path, charge(ledger, task_id, consumed_credits, balances))
    return RunOutcome(
        run_dir=run_dir,
        task_id=task_id,
        status=status,
        consumed_credits=consumed_credits,
        files=tuple(files),
    )

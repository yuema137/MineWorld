"""The Meshy client with an injected transport (no network), the credit cap, the redaction, and the
AI-identifier inspection of the accepted mesh (design §9 B5; I-B1-2, I-B1-3)."""

from __future__ import annotations

import hashlib
import json
from collections.abc import Callable, Mapping
from pathlib import Path

import pytest

from mwcast import identifiers, meshy
from mwcast.meshy import Ledger, MeshyError, check_cap, load_ledger, run_live, write_ledger

REPO = Path(__file__).resolve().parents[3]
MESHY_D = REPO / "clients/3d-spike/assets/characters/meshy_d/meshy_d.glb"
# Built from parts so that no key-shaped literal sits in this file.
FAKE_KEY = "msy_" + "Q" * 24
PNG_BYTES = b"\x89PNG\r\n\x1a\n" + b"fake image bytes for the request"

Call = tuple[str, str, Mapping[str, str], bytes | None]
Handler = Callable[[str, str, Mapping[str, str], bytes | None], tuple[int, bytes]]


class FakeTransport:
    """Records every call and answers from a handler. It is the only transport these tests use."""

    def __init__(self, handler: Handler) -> None:
        self.handler = handler
        self.calls: list[Call] = []

    def request(
        self,
        method: str,
        url: str,
        headers: Mapping[str, str],
        body: bytes | None,
        timeout_s: float,
    ) -> tuple[int, bytes]:
        self.calls.append((method, url, dict(headers), body))
        return self.handler(method, url, headers, body)


def _success(method: str, url: str, headers: Mapping[str, str], body: bytes | None) -> tuple[int, bytes]:
    if url.endswith("/balance"):
        return 200, b'{"credits": 100}'
    if method == "POST":
        return 200, b'{"result": "task-1"}'
    if url.endswith("/image-to-3d/task-1"):
        task = {
            "status": "SUCCEEDED",
            "consumed_credits": 30,
            "model_urls": {"glb": "https://files.example/model.glb"},
            "texture_urls": [{"base_color": "https://files.example/tex.png"}],
            "thumbnail_urls": {"front": "https://files.example/front.png"},
        }
        return 200, json.dumps(task).encode("utf-8")
    if url.startswith("https://files.example/"):
        return 200, b"bytes of " + url.encode("utf-8")
    return 404, b"not found"


def _unauthorised(method: str, url: str, headers: Mapping[str, str], body: bytes | None) -> tuple[int, bytes]:
    return 401, json.dumps({"message": "the key " + FAKE_KEY + " is not valid"}).encode("utf-8")


def _image(folder: Path) -> Path:
    path = folder / "input.png"
    path.write_bytes(PNG_BYTES)
    return path


def _stored_text(folder: Path) -> str:
    """Every stored file under `folder`, as text, joined. Used to prove a key is in none of them."""
    chunks: list[str] = []
    for path in sorted(folder.rglob("*")):
        if path.is_file():
            chunks.append(path.read_bytes().decode("utf-8", errors="replace"))
    return "\n".join(chunks)


def _no_sleep(_seconds: float) -> None:
    return None


def test_a_live_run_sends_the_key_only_in_the_header_and_stores_no_key(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv(meshy.KEY_ENV, FAKE_KEY)
    transport = FakeTransport(_success)
    scratch = tmp_path / "scratch"

    outcome = run_live(
        image=_image(tmp_path),
        overrides={},
        run_dir=scratch / "meshy" / "r1",
        ledger_path=scratch / "credits.json",
        cap=60,
        transport=transport,
        sleep=_no_sleep,
        now=lambda: "2026-10-14T00:00:00+0000",
    )

    posts = [call for call in transport.calls if call[0] == "POST"]
    assert len(posts) == 1
    body_text = posts[0][3]
    assert body_text is not None
    sent = json.loads(body_text.decode("utf-8"))
    assert sent["image_url"].startswith("data:image/png;base64,")
    assert sent["ai_model"] == "meshy-7.1"
    assert sent["pose_mode"] == "a-pose"
    assert posts[0][2]["Authorization"] == "Bearer " + FAKE_KEY
    assert outcome.status == "SUCCEEDED"
    assert outcome.files == ("model_glb.glb", "tex0_base_color.png", "thumb_front.png")
    assert FAKE_KEY not in _stored_text(scratch)
    assert "data:image" not in (scratch / "meshy" / "r1" / "request.json").read_text(encoding="utf-8")
    assert load_ledger(scratch / "credits.json").spent == 30


def test_a_401_whose_body_echoes_the_key_is_stored_and_raised_redacted(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv(meshy.KEY_ENV, FAKE_KEY)
    transport = FakeTransport(_unauthorised)
    scratch = tmp_path / "scratch"

    with pytest.raises(MeshyError) as caught:
        run_live(
            image=_image(tmp_path),
            overrides={},
            run_dir=scratch / "meshy" / "r2",
            ledger_path=scratch / "credits.json",
            cap=60,
            transport=transport,
            sleep=_no_sleep,
        )

    assert FAKE_KEY not in str(caught.value)
    assert "<redacted>" in str(caught.value)
    assert FAKE_KEY not in _stored_text(scratch)


def test_a_cap_of_29_refuses_before_any_request(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(meshy.KEY_ENV, FAKE_KEY)
    transport = FakeTransport(_success)

    with pytest.raises(MeshyError, match="cap"):
        run_live(
            image=_image(tmp_path),
            overrides={},
            run_dir=tmp_path / "run",
            ledger_path=tmp_path / "credits.json",
            cap=29,
            transport=transport,
            sleep=_no_sleep,
        )

    assert transport.calls == []


def test_the_cap_allows_the_second_run_and_refuses_the_third() -> None:
    check_cap(Ledger(spent=30, runs=("a",)), 60)

    with pytest.raises(MeshyError, match="cap"):
        check_cap(Ledger(spent=60, runs=("a", "b")), 60)


def test_the_ledger_round_trips(tmp_path: Path) -> None:
    path = tmp_path / "credits.json"
    write_ledger(path, Ledger(spent=60, runs=("a", "b")))

    assert load_ledger(path) == Ledger(spent=60, runs=("a", "b"))


def test_a_dry_run_reads_no_key_and_plans_without_image_bytes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.delenv(meshy.KEY_ENV, raising=False)
    image = _image(tmp_path)

    projection = meshy.plan(image, {}, Ledger(spent=0, runs=()), 60)

    assert projection.endpoint.endswith("/image-to-3d")
    assert projection.projected_credits == meshy.CREDITS_PER_RUN
    assert "image_url" not in projection.params
    assert projection.image_sha256 == hashlib.sha256(PNG_BYTES).hexdigest()


def test_a_live_run_without_the_key_is_refused_with_the_message(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.delenv(meshy.KEY_ENV, raising=False)
    transport = FakeTransport(_success)

    with pytest.raises(MeshyError, match="MESHY_API_KEY is not set"):
        run_live(
            image=_image(tmp_path),
            overrides={},
            run_dir=tmp_path / "run",
            ledger_path=tmp_path / "credits.json",
            cap=60,
            transport=transport,
            sleep=_no_sleep,
        )

    assert transport.calls == []


def test_identifiers_on_the_accepted_mesh_find_the_provenance_and_nothing_else() -> None:
    findings = identifiers.inspect_glb(MESHY_D)

    meshy_hits = [f.where for f in findings if f.kind == "meshy occurrence"]
    assert meshy_hits
    assert all(where.startswith("scenes[0].extras.mineworld_provenance") for where in meshy_hits)
    assert any(f.where == "scenes[0].extras" and f.kind == "glTF extras object" for f in findings)

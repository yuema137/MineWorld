"""The running HTTP adapter's endpoint: rebinding, bearers, secrets, bind rules, bounds, surface, shutdown
(pr-s27e §8 AC-e-4, AC-e-5, AC-e-6, AC-e-8, AC-e-9, AC-e-10, AC-e-11).

Every request here is a raw `http.client` request to the adapter process, with explicit `Host`, `Origin` and
`Authorization` headers, so the oracle is the HTTP status line the adapter sent; the seats' oracle is the server's own
`/status`. The in-process tests (`test_http_security.py`) own the gate's inner behaviour; this file proves the real
process answers the same on every OS. No address but loopback is used, except AC-e-8's one `0.0.0.0` start check.
"""

from __future__ import annotations

import asyncio
import http.client
import json
import subprocess
import sys
import time
from collections.abc import Iterator
from pathlib import Path

import pytest
from http_adapter import HttpAdapter, child_environment, http_adapter
from realserver import ADMIN_CANARY, INVITE, Server, market_town
from support import run_default
from test_play_mcp import PATIENCE, Transcript, call, text_of, until

from mineworld_agent.mcp_server import tools

pytestmark = pytest.mark.real_server

GOLDEN = Path(__file__).parent / "golden" / "tools.json"
INITIALIZE = json.dumps(
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "endpoint-test", "version": "0"},
        },
    }
).encode("utf-8")


@pytest.fixture
def café(tmp_path: Path) -> Iterator[Server]:
    server = Server(
        "market-town",
        *market_town(
            "--save",
            str(tmp_path / "save"),
            "--grant",
            "visitor",
            "--grant",
            "wanderer",
            "--grant",
            "alice",
            "--hold",
            "10",
            "--time-scale",
            "1",
        ),
    )
    try:
        yield server
    finally:
        server.stop()


class Answer:
    """One raw answer: status, headers, body, and the seconds from send to the status line."""

    def __init__(self, status: int, headers: dict[str, str], body: bytes, elapsed: float) -> None:
        self.status, self.headers, self.body, self.elapsed = status, headers, body, elapsed


def post(port: int, headers: dict[str, str], body: bytes = INITIALIZE) -> Answer:
    """One raw POST of `body` to `/mcp`; `headers` override the defaults (a `None`-free dict, so absent means absent)."""
    sent = {
        "Host": f"127.0.0.1:{port}",
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
        **headers,
    }
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=30)
    try:
        connection.putrequest("POST", "/mcp", skip_host=True, skip_accept_encoding=True)
        for name, value in sent.items():
            connection.putheader(name, value)
        connection.putheader("Content-Length", str(len(body)))
        started = time.monotonic()
        connection.endheaders(body)
        response = connection.getresponse()
        elapsed = time.monotonic() - started
        return Answer(
            response.status,
            {k.lower(): v for k, v in response.getheaders()},
            response.read(),
            elapsed,
        )
    finally:
        connection.close()


def bearer(token: str) -> dict[str, str]:
    return {"Authorization": f"Bearer {token}"}


def secrets_of(town: Server) -> list[str]:
    return [INVITE, ADMIN_CANARY, *town.grants().values()]


def test_rebinding_bearers_secrets_and_the_surface(café: Server, tmp_path: Path) -> None:
    """AC-e-4, AC-e-5, AC-e-6, AC-e-10 against one running adapter."""
    grants = café.grants()
    seen: list[bytes] = []

    async def scenario() -> HttpAdapter:
        async with http_adapter(
            café.url,
            tmp_path / "state",
            tmp_path / "logs",
            "--allow-origin",
            "http://127.0.0.1:5173",
        ) as adapter:
            port = adapter.port

            def ask(headers: dict[str, str]) -> Answer:
                answer = post(port, headers)
                seen.append(json.dumps(answer.headers).encode("utf-8") + answer.body)
                return answer

            valid = bearer(grants["visitor"])
            # AC-e-4: DNS rebinding.
            assert (
                await asyncio.to_thread(ask, {**valid, "Origin": "http://evil.example"})
            ).status == 403
            assert (
                await asyncio.to_thread(ask, {**valid, "Host": f"rebind.example:{port}"})
            ).status == 421
            rebound = {
                **valid,
                "Host": f"rebind.example:{port}",
                "Origin": f"http://rebind.example:{port}",
            }
            assert (await asyncio.to_thread(ask, rebound)).status == 421
            assert (await asyncio.to_thread(ask, valid)).status == 200
            allowed = {**valid, "Origin": "http://127.0.0.1:5173"}
            assert (await asyncio.to_thread(ask, allowed)).status == 200
            neighbour = {**valid, "Origin": "http://127.0.0.1:5174"}
            assert (await asyncio.to_thread(ask, neighbour)).status == 403

            # AC-e-5: a missing or malformed bearer is 401, no sooner than 0.5 s after it was sent.
            for headers in (
                {},
                {"Authorization": "Basic dXNlcjpwYXNz"},
                bearer("x" * 7),
                bearer("x" * 129),
                {"Authorization": "Bearer abc defgh"},
            ):
                answer = await asyncio.to_thread(ask, headers)
                assert answer.status == 401, (headers.keys(), answer.status)
                assert answer.headers.get("www-authenticate", "").startswith("Bearer")
                assert answer.elapsed >= 0.5, answer.elapsed

            # AC-e-5: a well-formed but wrong bearer is the server's refusal at join.
            async with adapter.session("mwagent-wrong-credential-0000") as wrong:
                answer = await wrong.call_tool("join", {"seat": "visitor", "nickname": "Ada"})
                assert answer.is_error is True
                refusal = json.loads(text_of(answer))
                assert refusal["code"] == "join_refused" and refusal["join"] == "unauthorized", (
                    refusal
                )

            # AC-e-10: the same surface as stdio, and the golden tool list.
            async with adapter.session(grants["visitor"]) as session:
                listed = await session.list_tools()
                served = [
                    tool.model_dump(mode="json", by_alias=True, exclude_none=True)
                    for tool in listed.tools
                ]
                published = [
                    tool.model_dump(mode="json", by_alias=True, exclude_none=True)
                    for tool in tools()
                ]
                assert served == published
                golden = json.loads(GOLDEN.read_text(encoding="utf-8"))
                assert [tool["name"] for tool in served] == [
                    tool["name"] for tool in golden["tools"]
                ]
                assert len(served) == 9
                seen.append(json.dumps(served).encode("utf-8"))
                joined = await call(
                    session, Transcript(), "join", {"seat": "visitor", "nickname": "Ada"}
                )
                seen.append(json.dumps(joined).encode("utf-8"))
                await call(session, Transcript(), "leave", {"handle": joined["handle"]})
            return adapter

    adapter = run_default(scenario(), timeout_s=180)

    # AC-e-6: no credential anywhere the test or an operator could read.
    state_files = [path for path in (tmp_path / "state").rglob("*") if path.is_file()]
    surfaces = {
        "stdout": adapter.stdout().encode("utf-8"),
        "stderr": adapter.stderr().encode("utf-8"),
        "answers": b"\n".join(seen),
        **{f"state {path.name}": path.read_bytes() for path in state_files},
    }
    for secret in secrets_of(café):
        for where, data in surfaces.items():
            assert secret.encode("utf-8") not in data, f"a secret appears in {where}"


def test_a_full_pool_is_503_until_a_seat_leaves(café: Server, tmp_path: Path) -> None:
    """AC-e-9: with `--max-credentials 2`, a third bearer is 503 with `Retry-After` while both play."""
    grants = café.grants()

    async def scenario() -> None:
        async with http_adapter(
            café.url, tmp_path / "state", tmp_path / "logs", "--max-credentials", "2"
        ) as adapter:
            transcript = Transcript()
            async with (
                adapter.session(grants["visitor"]) as a,
                adapter.session(grants["wanderer"]) as b,
            ):
                joined_a = await call(a, transcript, "join", {"seat": "visitor", "nickname": "A"})
                joined_b = await call(b, transcript, "join", {"seat": "wanderer", "nickname": "B"})
                full = await asyncio.to_thread(post, adapter.port, bearer(grants["alice"]))
                assert full.status == 503, full.status
                assert full.headers.get("retry-after", "").isdigit()
                await call(b, transcript, "leave", {"handle": joined_b["handle"]})
                admitted = await asyncio.to_thread(post, adapter.port, bearer(grants["alice"]))
                assert admitted.status == 200, admitted.status
                await call(a, transcript, "leave", {"handle": joined_a["handle"]})

    run_default(scenario(), timeout_s=180)


def test_shutdown_gives_every_seat_back(café: Server, tmp_path: Path) -> None:
    """AC-e-11: stopping the adapter while two agents hold seats returns `/status.clients` to its baseline within
    the hold plus five seconds."""
    grants = café.grants()
    baseline = café.status().clients
    adapter = HttpAdapter(café.url, tmp_path / "state", tmp_path / "logs")

    async def scenario() -> None:
        await adapter.started()
        transcript = Transcript()
        async with (
            adapter.session(grants["visitor"]) as a,
            adapter.session(grants["wanderer"]) as b,
        ):
            await call(a, transcript, "join", {"seat": "visitor", "nickname": "A"})
            await call(b, transcript, "join", {"seat": "wanderer", "nickname": "B"})

            async def both_seated() -> bool:
                return await asyncio.to_thread(lambda: café.status().clients == baseline + 2)

            await until(both_seated, patience=PATIENCE, what="both agents were seated")
            await asyncio.to_thread(adapter.stop)

        started = time.monotonic()

        async def back() -> bool:
            return await asyncio.to_thread(lambda: café.status().clients == baseline)

        await until(back, patience=10 + 5, what="every seat came back")
        # POSIX: both handles left at once. Windows: by leave, or by the server's hold.
        print(f"seats back after {time.monotonic() - started:.1f} s ({sys.platform})")

    try:
        run_default(scenario(), timeout_s=120)
    finally:
        adapter.kill()


def test_a_non_loopback_bind_starts_only_with_allow_remote(tmp_path: Path) -> None:
    """AC-e-8: `--http 0.0.0.0:0` is refused without `--allow-remote` (exit 2, nothing bound), and starts with it,
    saying plain HTTP. The only non-loopback bind in the suite; no request is sent to it."""
    base = [sys.executable, "-m", "mineworld_agent", "mcp", "--server", "ws://127.0.0.1:9/ws"]
    refused = subprocess.run(
        [*base, "--http", "0.0.0.0:0"],
        env=child_environment(),
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert refused.returncode == 2, refused.stderr
    assert "--allow-remote" in refused.stderr
    assert "listening" not in refused.stderr

    log = tmp_path / "remote.log"
    with log.open("w", encoding="utf-8") as errors:
        process = subprocess.Popen(
            [*base, "--http", "0.0.0.0:0", "--allow-remote"],
            env=child_environment(),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=errors,
        )
        try:
            deadline = time.monotonic() + 30
            text = ""
            while time.monotonic() < deadline and "listening" not in text:
                time.sleep(0.05)
                text = log.read_text(encoding="utf-8")
            assert "listening on http://" in text, text
            assert "plain HTTP" in text, text
        finally:
            process.kill()
            process.wait()

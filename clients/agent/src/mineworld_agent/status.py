"""`GET /status`: what the world is, as any visitor may ask (step-27 §5.2; `PROTOCOL.md` §5.7).

The SDK carries no HTTP client and this member adds no dependency (F-S27c-5), so this is the standard
library's `urllib.request`, run in a worker thread so that the event loop never blocks. The answer is the
SDK's own `WorldSummary` model, validated strictly. Nothing here sends a credential: `/status` is public.
"""

from __future__ import annotations

import asyncio
import urllib.request
from urllib.parse import urlsplit, urlunsplit

from mineworld_sdk.wire.frames import WorldSummary

__all__ = ["STATUS_PATIENCE", "fetch_status", "status_url"]

STATUS_PATIENCE = 10.0
"""Wall seconds a `/status` request may take."""


def status_url(server: str) -> str:
    """`ws://host:port/ws` → `http://host:port/status` (and `wss` → `https`): the same host serves both
    (`PROTOCOL.md` §1)."""
    parts = urlsplit(server)
    scheme = {"ws": "http", "wss": "https"}.get(parts.scheme)
    if scheme is None or not parts.netloc:
        raise ValueError("the server is a ws:// or wss:// URL with a host")
    return urlunsplit((scheme, parts.netloc, "/status", "", ""))


def _get(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=STATUS_PATIENCE) as answer:
        body: bytes = answer.read()
        return body


async def fetch_status(server: str) -> WorldSummary:
    """The server's `WorldSummary`."""
    body = await asyncio.to_thread(_get, status_url(server))
    return WorldSummary.model_validate_json(body)

"""Milestone D's scripted model behind a loopback OpenAI-compatible endpoint, for re-recording the Milestone D cassette
without a model (DM-20261128-4; red-team #565 condition 6).

`record_the_milestone_d_cassette` (`tools/cli/tests/milestone_d.rs`) records whatever an OpenAI-compatible server at
`MINEWORLD_MILESTONE_D_BASE_URL` answers. Run this file first; it prints `port=<n>` and answers every chat completion
with `milestone_d_cognition.remembers`, a pure function of the request's user message:

    python cognition/lm-controller/tests/milestone_d_stub_server.py &
    MINEWORLD_MILESTONE_D_BASE_URL=http://127.0.0.1:<n>/v1 MINEWORLD_MILESTONE_D_MODEL=scripted-remembers \
      MINEWORLD_MILESTONE_D_RECORD=<dir> cargo test -p mineworld-cli --test milestone_d -- --ignored --exact \
      record_the_milestone_d_cassette

No model is reached and nothing is paid. It listens on 127.0.0.1 only and exits when its standard input closes.
"""

from __future__ import annotations

import json
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import cast

from milestone_d_cognition import remembers

from mineworld_cognition.backend.model import (
    BackendFailure,
    CompletionRequest,
    Message,
    Role,
    Sampling,
)


def _request(body: dict[str, object]) -> CompletionRequest:
    sent = cast(list[dict[str, str]], body.get("messages", []))
    messages = tuple(Message(role=cast(Role, m["role"]), text=m["content"]) for m in sent)
    return CompletionRequest(
        purpose="decide",
        messages=messages,
        sampling=Sampling(temperature_milli=0, max_output_tokens=256, seed=0),
    )


class _Handler(BaseHTTPRequestHandler):
    def do_POST(self) -> None:
        body = cast(
            dict[str, object], json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        )
        answer = remembers(_request(body))
        text = "" if isinstance(answer, BackendFailure) else answer.text
        self._send(
            {
                "choices": [
                    {"message": {"role": "assistant", "content": text}, "finish_reason": "stop"}
                ],
                "usage": {"prompt_tokens": 0, "completion_tokens": len(text) // 4},
            }
        )

    def do_GET(self) -> None:
        self._send({"object": "list", "data": [{"id": "scripted-remembers", "object": "model"}]})

    def _send(self, payload: object) -> None:
        data = json.dumps(payload).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, format: str, *args: object) -> None:
        del format, args


def main() -> None:
    server = ThreadingHTTPServer(("127.0.0.1", 0), _Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    print(f"port={server.server_address[1]}", flush=True)
    sys.stdin.read()
    server.shutdown()


if __name__ == "__main__":
    main()

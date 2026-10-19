"""Records the market-town observations the view tests read (pr-s27c F-S27c-3; how and when: `SOURCE.md`).

Run by hand, never by a test: `uv run --locked python clients/agent/tests/fixtures/record.py` from the
repository root, with the binary built (`cargo build -p mineworld-cli`, or `MINEWORLD_BIN`). It starts the
real `mineworld server worlds/market-town --town --agent alice`, joins seats through the SDK exactly as a
client does, and writes each observation as the server sent it (the contract's JSON, unchanged):

- `market-town-start.json`: the visitor's first observation, in the café at genesis;
- `market-town-walking.json`: the visitor's, after `walk-to` the street and one `walk-step`, its own
  `walking` component disclosed;
- `market-town-street.json`: the wanderer's, joined at 13:40 on the first day on a second server at
  `--time-scale 300 --pace 1` (today spelled `--solo --time-scale 300 --cadence 1`, ARC-69) while
  the town walked it: the street, whose passages lead to every place, with the wanderer's own walk
  under way. (The aim was the café at its busiest hour; SOURCE.md says why the genesis café stands
  for it.)
"""

from __future__ import annotations

import asyncio
import json
import os
import queue
import re
import subprocess
import sys
import threading
import urllib.request
from pathlib import Path

from mineworld_sdk import Answered, ResumingSeat, Seen, offers
from mineworld_sdk.wire.contract import Accepted, Observation
from mineworld_sdk.wire.frames import Invite, WorldSummary
from mineworld_sdk.wire.ids import ActionTypeId, EntityKey, JsonValue

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[3]
INVITE = "record-fixture-invite-0123456789ab"
AFTERNOON = 13 * 3600 + 40 * 60
"""13:40 on the first day, in world seconds since genesis (market-town's calendar starts at midnight)."""
_LISTENING = re.compile(r"listening on (?P<http>http://\S+) \((?P<url>ws://[^)\s]+)\)")


def binary() -> Path:
    named = os.environ.get("MINEWORLD_BIN")
    if named:
        return Path(named)
    suffix = ".exe" if sys.platform == "win32" else ""
    return REPOSITORY / "target" / "debug" / f"mineworld{suffix}"


def start(*flags: str) -> tuple[subprocess.Popen[str], str, str]:
    environment = dict(os.environ)
    environment["MINEWORLD_INVITE"] = INVITE
    world = str(REPOSITORY / "worlds" / "market-town")
    process = subprocess.Popen(
        [str(binary()), "server", world, "--town", "--agent", "alice", *flags],
        cwd=REPOSITORY,
        env=environment,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    lines: queue.Queue[str] = queue.Queue()

    def drain() -> None:
        assert process.stdout is not None
        for line in process.stdout:
            lines.put(line)

    threading.Thread(target=drain, daemon=True).start()
    while True:
        line = lines.get(timeout=30)
        if match := _LISTENING.search(line):
            return process, match.group("url"), match.group("http")


async def seat(url: str, key: str) -> ResumingSeat:
    return await ResumingSeat.connect(
        url, seat=EntityKey(key), invite=Invite(INVITE), nickname=f"record-{key}", cursor=None
    )


def write(name: str, observation: Observation) -> None:
    text = json.dumps(observation.model_dump(mode="json", by_alias=True), indent=1) + "\n"
    (HERE / name).write_text(text, encoding="utf-8")
    people = sum(1 for entity in observation.entities if entity.entity_type == "person")
    print(
        f"wrote {name}: at {observation.at}, {people} people, {len(observation.affordances)} offers"
    )


def first_exit(observation: Observation) -> JsonValue:
    """The place the first passage of the observer's place leads to, as the contract's `PlaceId`."""
    for entity in observation.entities:
        for component in entity.components:
            payload = component.payload
            if component.component_type == "passages" and isinstance(payload, dict):
                leads = payload.get("leads_to")
                if isinstance(leads, list) and leads and isinstance(leads[0], dict):
                    return leads[0].get("to")
    raise LookupError("no passage disclosed")


async def accepted(visitor: ResumingSeat, seen: Seen, action: str, payload: JsonValue) -> None:
    request = offers.request(
        seen.frame.observation, ActionTypeId(action), target=None, payload=payload
    )
    answer = await visitor.submit(request)
    assert isinstance(answer, Answered) and isinstance(answer.result, Accepted), answer


async def genesis(url: str) -> None:
    visitor = await seat(url, "visitor")
    seen = await visitor.changed()
    write("market-town-start.json", seen.frame.observation)
    street = first_exit(seen.frame.observation)
    await accepted(visitor, seen, "walk-to", {"to": {"place": {"place": street}}})
    seen = await asyncio.wait_for(visitor.changed(seen), 30)
    await accepted(visitor, seen, "walk-step", {})
    seen = await asyncio.wait_for(visitor.changed(seen), 30)
    write("market-town-walking.json", seen.frame.observation)
    await visitor.leave()


def status(http: str) -> WorldSummary:
    with urllib.request.urlopen(f"{http}/status", timeout=10) as answer:
        return WorldSummary.model_validate_json(answer.read())


async def afternoon(url: str, http: str) -> None:
    while (await asyncio.to_thread(status, http)).at < AFTERNOON:
        await asyncio.sleep(1)
    wanderer = await seat(url, "wanderer")
    seen = await wanderer.changed()
    for _ in range(3):
        seen = await asyncio.wait_for(wanderer.changed(seen), 30)
    write("market-town-street.json", seen.frame.observation)
    await wanderer.leave()


def main() -> None:
    process, url, _ = start("--listen", "127.0.0.1:0")
    try:
        asyncio.run(genesis(url), loop_factory=asyncio.SelectorEventLoop)
    finally:
        process.kill()
        process.wait()
    process, url, http = start(
        "--listen", "127.0.0.1:0", "--solo", "--time-scale", "300", "--cadence", "1"
    )
    try:
        asyncio.run(afternoon(url, http), loop_factory=asyncio.SelectorEventLoop)
    finally:
        process.kill()
        process.wait()


if __name__ == "__main__":
    main()

"""C-7 on the real 12-seat stream (pr-s28e AC-e-3; step-28 C-7, AS28-18's C-7): every sampled id explained.

social-cafe, seed 7, 2 days, through the real `mineworld` binary (`social_cafe_12`); every seat absorbed in day
frames under `consolidate` into one file home. Per seat, 1 000 ids drawn by SplitMix64 from its perceived stream
and 200 it never perceived are each classed as the oracle classes them (the dump's `l0`, units, forgotten lines
and watermark; the perceived export), with the perceived file read by the command's own reader; a sample of each
seat's ids also goes through the command itself. Zero `incoherent`.
"""

from __future__ import annotations

import json
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path

import pytest
from home_streams import Cafe12, day_frames
from inspect_fixtures import Oracle
from retention_driver import retention
from retention_stream import draw
from support import run

from mineworld_cognition.inspect.inputs import read_perceived
from mineworld_cognition.memory.explain import explain
from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb

PERCEIVED = 1_000
NEVER = 200
THROUGH_THE_COMMAND = 5


@pytest.mark.real_binary
def test_every_sampled_id_of_every_seat_is_explained(
    social_cafe_12: Cafe12, tmp_path: Path
) -> None:
    started = time.monotonic()
    cafe = social_cafe_12
    path = tmp_path / "memory.sqlite"
    kept = retention("consolidate")
    with MemoryHome.open(path, cafe.instance) as home:
        for seat in sorted(cafe.streams):
            view = home.seat(
                StoreIdentity(world_instance=cafe.instance, seat=seat, self_entity=cafe.me[seat])
            )
            for frame in day_frames(cafe.streams[seat]):
                run(absorb(view, frame, frame[-1].id, kept), timeout_s=600)
    everyone = sorted({one.id for facts in cafe.streams.values() for one in facts}, key=int)
    seen: Counter[str] = Counter()
    with MemoryHome.open_read_only(path) as home:
        for index, seat in enumerate(sorted(cafe.streams)):
            file = tmp_path / f"{seat}.perceived.jsonl"
            file.write_text(
                "".join(one.model_dump_json() + "\n" for one in cafe.streams[seat]),
                encoding="utf-8",
            )
            perceived = read_perceived(file)
            view = home.view_of(seat)
            oracle = Oracle.of(view.dump())
            mine = [one.id for one in cafe.streams[seat]]
            others = [event for event in everyone if event not in perceived]
            sample = [mine[draw(7, index * 10_000 + n, 1) % len(mine)] for n in range(PERCEIVED)]
            sample += [others[draw(7, index * 10_000 + n, 2) % len(others)] for n in range(NEVER)]
            for event in sample:
                kind = explain(view, event, perceived).kind
                assert kind == oracle.classify(event, perceived), (seat, event)
                seen[kind] += 1
            for event in sample[:: len(sample) // THROUGH_THE_COMMAND][:THROUGH_THE_COMMAND]:
                done = subprocess.run(
                    [
                        sys.executable,
                        "-m",
                        "mineworld_cognition.inspect.cli",
                        "inspect",
                        str(path),
                        "--seat",
                        seat,
                        "access",
                        "--event",
                        event,
                        "--perceived",
                        str(file),
                        "--json",
                    ],
                    capture_output=True,
                    timeout=300,
                    check=False,
                )
                assert done.returncode == 0, done.stderr
                kind = json.loads(done.stdout)["explanation"]["kind"]
                assert kind == oracle.classify(event, perceived), (seat, event)
    assert seen["incoherent"] == 0
    assert seen["held"] and seen["never-perceived"], seen
    print(
        f"\n[AC-e-3 real] {sum(seen.values())} ids {dict(seen)}; {time.monotonic() - started:.1f} s"
    )

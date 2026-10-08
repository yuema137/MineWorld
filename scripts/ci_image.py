#!/usr/bin/env python3
"""Show that the runtime image hosts MineWorld: the laptop's commands, in the shipped container.

The runtime image (`Dockerfile`, stage `runtime`; `docs/DECISIONS.md` DEP-18) is what a deployment
runs (`NETWORKING.md` §7). Building it proves only that it builds. This script runs it the way an
operator would, and reads every verdict from what the program printed. An exit code alone is not a
verdict (step-14 §9, A13-5):

1. `mineworld validate worlds/<w>` for every shipped world → "<w> is a valid World Pack."
2. `mineworld run worlds/social-cafe --headless --seed 7 --days 1` → a `faults 0` summary line
3. the image's default command on a named volume → it creates a world and listens on 7878 (a TCP
   connection is accepted); `docker stop` (SIGINT, the image's STOPSIGNAL) → it reports stopping
   and exits 0; started again on the same volume → it reports resuming that world

    python3 scripts/ci_image.py <image>

Runs on the host, where Docker is; it needs nothing but the standard library and the `docker` CLI.
Exits non-zero naming the first expectation that did not hold, with the output it saw.
"""

from __future__ import annotations

import re
import socket
import subprocess
import sys
import time
import uuid

WORLDS = ("social-cafe", "market-town", "bodies-yard")
PORT = 7878


class Unmet(Exception):
    """An expectation about the image's observable behaviour did not hold."""


def docker(*arguments: str, check: bool = True) -> str:
    result = subprocess.run(["docker", *arguments], capture_output=True, text=True)
    output = result.stdout + result.stderr
    if check and result.returncode != 0:
        raise Unmet(f"docker {' '.join(arguments)} exited {result.returncode}:\n{output}")
    return output


def expect(pattern: str, output: str, what: str) -> None:
    if not re.search(pattern, output, re.MULTILINE):
        raise Unmet(f"{what}: expected /{pattern}/ in:\n{output}")
    print(f"[image] PASS {what}", flush=True)


def wait_for_log(container: str, pattern: str, what: str, seconds: float = 60.0) -> str:
    deadline = time.monotonic() + seconds
    logs = ""
    while time.monotonic() < deadline:
        logs = docker("logs", container, check=False)
        if re.search(pattern, logs, re.MULTILINE):
            print(f"[image] PASS {what}", flush=True)
            return logs
        time.sleep(1)
    raise Unmet(f"{what}: /{pattern}/ did not appear within {seconds:.0f} s; logs:\n{logs}")


def accepts_connection() -> None:
    with socket.create_connection(("127.0.0.1", PORT), timeout=5):
        pass
    print(f"[image] PASS a TCP connection to port {PORT} is accepted", flush=True)


def hosted_world(image: str) -> None:
    tag = uuid.uuid4().hex[:8]
    volume, container = f"mineworld-ci-{tag}", f"mineworld-ci-{tag}"
    docker("volume", "create", volume)
    try:
        docker("run", "--detach", "--name", container, "--volume", f"{volume}:/var/lib/mineworld",
               "--publish", f"127.0.0.1:{PORT}:{PORT}", image)
        wait_for_log(container, r"^\[mineworld\] created /var/lib/mineworld/social-cafe/",
                     "the default command creates its world on the volume")
        wait_for_log(container, rf"^\[mineworld\] listening on http://0\.0\.0\.0:{PORT}",
                     f"the server listens on {PORT}")
        accepts_connection()

        docker("stop", "--time", "30", container)
        state = docker("inspect", "--format", "{{.State.ExitCode}}", container).strip()
        if state != "0":
            raise Unmet(f"docker stop: the server exited {state}, not 0; logs:\n{docker('logs', container)}")
        expect(r"^\[mineworld\] stopping$", docker("logs", container), "docker stop shuts the server down (exit 0)")

        docker("start", container)
        wait_for_log(container, r"^\[mineworld\] resumed /var/lib/mineworld/social-cafe/",
                     "started again on the same volume, the world resumes")
        wait_for_log(container, rf"^\[mineworld\] listening on http://0\.0\.0\.0:{PORT}",
                     "the resumed server listens again")
    finally:
        docker("rm", "--force", container, check=False)
        docker("volume", "rm", "--force", volume, check=False)


def main(arguments: list[str]) -> int:
    if len(arguments) != 1:
        print("usage: ci_image.py <image>", file=sys.stderr)
        return 2
    image = arguments[0]
    try:
        for world in WORLDS:
            expect(rf"^worlds/{world} is a valid World Pack\.$",
                   docker("run", "--rm", image, "validate", f"worlds/{world}"),
                   f"validate worlds/{world}")
        expect(r"^faults\s+0$",
               docker("run", "--rm", image, "run", "worlds/social-cafe", "--headless", "--seed", "7",
                      "--days", "1"),
               "run worlds/social-cafe --headless --seed 7 --days 1 reports faults 0")
        hosted_world(image)
    except Unmet as unmet:
        print(f"[image] FAIL {unmet}", file=sys.stderr)
        return 1
    print("[image] every expectation held")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""The MineWorld client protocol, implemented with no game engine and no library.

This file exists to answer one question with evidence rather than with prose, and it is the
primary evidence behind `docs/references/UNREAL_ADAPTER_SPIKE.md` §2:

    what must a client that is not the Godot client actually implement in order to join a
    world, receive Observations, submit ActionIntents and surface the results?

It therefore imports nothing but the Python standard library. There is no WebSocket package, no
engine, no generated stub and no MineWorld SDK -- RFC 6455 is spoken by hand in `Socket`, and the
frames are the ones `server/PROTOCOL.md` specifies. Whatever this file needs is exactly the
irreducible cost of a new client; whatever it does not need is not part of that cost. An Unreal
client's protocol binding is this, in C++, and nothing more.

It is a probe, not production code and not a test in the repository's test suite: it is run by
hand against a running server and its output is captured into `evidence/`.

    ./target/debug/mineworld server worlds/social-cafe --listen 127.0.0.1:7878 --agent alice &
    python3 spike/unreal/probe.py > spike/unreal/evidence/transcript.log

There is no world rule in this file. It never measures a distance and never decides whether an
action is possible; it reads the server's verdict out of the affordance it was sent, which is the
obligation `docs/ENGINEERING_RULES.md` §§8-9 places on every client and `clients/protocol/ADOPTION.md`
§3.3 restates. The one place it deliberately breaks a rule is `refusals()`, whose whole purpose is to
send frames a correct client would not, and to record what the server answers.
"""

import base64
import json
import os
import socket
import struct
import sys

HOST = "127.0.0.1"
PORT = 7878

# The two places a MineWorld identity appears in this file are dictionary keys and `str` values.
# Never an int, never a float: `EntityId`, `EventId`, `ActionId` and `ProcessId` are 64-bit and
# arrive as decimal strings for the reason `contracts/src/ids.rs` records. Python happens to have
# arbitrary-precision integers and would survive parsing them; Unreal's `FJsonValue` and Godot's
# `JSON` would not, so the rule is followed here as a client in either of those languages must.


class Socket:
    """A WebSocket client: the whole of RFC 6455 that this protocol needs."""

    def __init__(self, host: str, port: int) -> None:
        self._socket = socket.create_connection((host, port), timeout=15)
        key = base64.b64encode(os.urandom(16)).decode()
        self._socket.sendall(
            (
                "GET /ws HTTP/1.1\r\n"
                f"Host: {host}:{port}\r\n"
                "Upgrade: websocket\r\n"
                "Connection: Upgrade\r\n"
                f"Sec-WebSocket-Key: {key}\r\n"
                "Sec-WebSocket-Version: 13\r\n\r\n"
            ).encode()
        )
        self._buffer = b""
        while b"\r\n\r\n" not in self._buffer:
            self._buffer += self._socket.recv(4096)
        head, self._buffer = self._buffer.split(b"\r\n\r\n", 1)
        status = head.split(b"\r\n")[0]
        if b"101" not in status:
            raise RuntimeError(f"the server did not upgrade: {status!r}")

    def send(self, frame: dict) -> None:
        """One masked text frame. A binary frame is refused by the server (`PROTOCOL.md` §1)."""
        data = json.dumps(frame).encode()
        length = len(data)
        if length < 126:
            header = struct.pack("!BB", 0x81, 0x80 | length)
        elif length < 65536:
            header = struct.pack("!BBH", 0x81, 0x80 | 126, length)
        else:
            header = struct.pack("!BBQ", 0x81, 0x80 | 127, length)
        mask = os.urandom(4)
        masked = bytes(byte ^ mask[i % 4] for i, byte in enumerate(data))
        self._socket.sendall(header + mask + masked)

    def _need(self, count: int) -> None:
        while len(self._buffer) < count:
            chunk = self._socket.recv(65536)
            if not chunk:
                raise EOFError("the connection closed")
            self._buffer += chunk

    def receive(self) -> tuple[int, bytes]:
        """One frame: its opcode and its payload. Ping and pong carry no protocol meaning."""
        self._need(2)
        opcode = self._buffer[0] & 0x0F
        length, offset = self._buffer[1] & 0x7F, 2
        if length == 126:
            self._need(4)
            length, offset = struct.unpack("!H", self._buffer[2:4])[0], 4
        elif length == 127:
            self._need(10)
            length, offset = struct.unpack("!Q", self._buffer[2:10])[0], 10
        self._need(offset + length)
        payload = self._buffer[offset : offset + length]
        self._buffer = self._buffer[offset + length :]
        return opcode, payload


class Connection:
    """A seated connection: the five server frames, and the two a client may send."""

    PROTOCOL = 1

    def __init__(self, seat: str) -> None:
        self.socket = Socket(HOST, PORT)
        self.socket.send({"t": "join", "seat": seat})
        welcome = self.until("welcome")
        if welcome["protocol"] != self.PROTOCOL:
            # Refused rather than guessed at, as `PROTOCOL.md` §9 requires.
            raise RuntimeError(f"the server speaks protocol {welcome['protocol']}")
        self.seat = welcome["seat"]
        self.observer = welcome["observer"]  # a string, always
        self.world = welcome["world"]
        self.welcome_frame = welcome
        self.sequence = 0
        self.latest = None
        self.tokens = 0

    def until(self, *kinds: str) -> dict:
        """The next frame of one of these kinds, keeping only the newest observation."""
        while True:
            opcode, payload = self.socket.receive()
            if opcode != 0x1:
                continue
            frame = json.loads(payload.decode())
            if frame.get("t") == "observation":
                # A lower `seq` is stale and is dropped; a gap is not an error (`PROTOCOL.md` §5).
                if frame["seq"] < self.sequence:
                    continue
                self.sequence = frame["seq"]
                self.latest = frame["observation"]
            if frame.get("t") in kinds:
                return frame

    def submit(self, action_type: str, target, payload: dict, actor_location=None) -> dict:
        """One request, and the world's answer to it.

        What the client fills in, and what it must not: the actor is this connection's observer,
        `action_type` is written in both of the places the contract requires it, and there is no
        `action_id` and no `issued_at` -- the server allocates identity and the instant (`INV-6`).
        """
        self.tokens += 1
        token = f"p{self.tokens}"
        request = {
            "actor": self.observer,
            "action_type": action_type,
            "target": target,
            "payload": {"action_type": action_type, "payload": payload},
            "actor_location": actor_location,
        }
        say(f"--> {json.dumps({'t': 'submit', 'token': token, 'request': request})}")
        self.socket.send({"t": "submit", "token": token, "request": request})
        answer = self.until("result", "refused")
        say(f"<-- {json.dumps(answer)}")
        return answer


def location(place: str, x_mm: int, y_mm: int, z_mm: int = 0, yaw_mdeg=None) -> dict:
    """A `Location` as the protocol carries one: integer millimetres and millidegrees.

    Every number here is an `int`. A client whose language has one number type must round and cast
    before encoding; see `refusals()` for what the server does when it does not.
    """
    facing = None if yaw_mdeg is None else {"yaw": int(yaw_mdeg), "pitch": None}
    return {
        "place": {"entity": place, "entity_type": "place"},
        "local": {"x": int(x_mm), "y": int(y_mm), "z": int(z_mm)},
        "facing": facing,
    }


def say(line: str) -> None:
    print(line, flush=True)


def walk_up_and_talk() -> None:
    """The whole loop a client exists to perform: join, observe, move, act, be answered."""
    say("=== 1. join =========================================================")
    world = Connection("visitor")
    say(f"<-- {json.dumps(world.welcome_frame)}")
    say(f"    observer is a JSON {type(world.observer).__name__} = {world.observer!r}")

    say("\n=== 2. the first observation ========================================")
    world.until("observation")
    observation = world.latest
    say(json.dumps(observation, indent=2))

    here = observation["self_location"]["place"]["entity"]
    baristas = [e["id"] for e in observation["entities"] if "barista" in (e["tags"] or [])]
    alice = baristas[0]
    alice_at = next(e for e in observation["entities"] if e["id"] == alice)["location"]["local"]
    say(f"\n    place={here!r}  barista={alice!r} at {alice_at}")

    offered = [a for a in observation["affordances"] if a["target"] == alice]
    say(f"    what the server says about acting on {alice!r}: {json.dumps(offered)}")

    say("\n=== 3. walk: the client moves in its own space and reports where ====")
    # Which position to walk to is the client's own business. Whether it is close *enough* is not:
    # the requirement above says 3000 mm, and this client does not check it -- it moves, reports,
    # and reads the next affordance.
    stood = location(here, alice_at["x"], alice_at["y"] - 1000, 0, 0)
    world.submit("arrive", None, {"location": stood}, actor_location=stood)

    say("\n=== 4. the affordance the server recomputed =========================")
    while True:
        world.until("observation")
        if world.latest["self_location"]["local"] == stood["local"]:
            break
    after = [a for a in world.latest["affordances"] if a["target"] == alice]
    say(f"    {json.dumps(after)}")

    say("\n=== 5. talk, now that the server says it may ========================")
    world.submit("talk", alice, {"utterance": "hello Alice, from a client with no game engine"},
                 actor_location=stood)


def refusals() -> None:
    """What the server answers a client that gets it wrong.

    Deliberately incorrect frames. This is the only part of this file that breaks the rules the
    rest of it follows, and it breaks them on purpose: an author writing a client in a new language
    needs to know what each mistake looks like on the wire, and the float cases in particular are
    the ones a language with one number type earns.
    """
    say("\n=== 6. refusals =====================================================")
    world = Connection("wanderer")
    place = world.until("observation")["observation"]["self_location"]["place"]["entity"]

    say("\n-- a float where the contract declares an i32, at the PROTOCOL boundary")
    floating = location(place, 0, 0)
    floating["local"]["x"] = 1500.0
    world.submit("talk", "2", {"utterance": "x"}, actor_location=floating)

    say("\n-- the same float inside a SYSTEM payload")
    world.submit("arrive", None, {"location": floating})

    say("\n-- an unquoted 64-bit identity (accepted, for a hand-written fixture)")
    world.tokens += 1
    request = {"actor": int(world.observer), "action_type": "talk", "target": 2,
               "payload": {"action_type": "talk", "payload": {"utterance": "x"}},
               "actor_location": None}
    say(f"--> {json.dumps({'t': 'submit', 'token': 'n1', 'request': request})}")
    world.socket.send({"t": "submit", "token": "n1", "request": request})
    say(f"<-- {json.dumps(world.until('result', 'refused'))}")

    say("\n-- asking the world to act as somebody else")
    world.tokens += 1
    request = {"actor": "2", "action_type": "talk", "target": "3",
               "payload": {"action_type": "talk", "payload": {"utterance": "x"}},
               "actor_location": None}
    say(f"--> {json.dumps({'t': 'submit', 'token': 'x1', 'request': request})}")
    world.socket.send({"t": "submit", "token": "x1", "request": request})
    say(f"<-- {json.dumps(world.until('result', 'refused'))}")

    say("\n-- the two halves of action_type disagreeing")
    world.tokens += 1
    request = {"actor": world.observer, "action_type": "talk", "target": "2",
               "payload": {"action_type": "arrive", "payload": {}}, "actor_location": None}
    say(f"--> {json.dumps({'t': 'submit', 'token': 'd1', 'request': request})}")
    world.socket.send({"t": "submit", "token": "d1", "request": request})
    say(f"<-- {json.dumps(world.until('result', 'refused'))}")

    say("\n-- a frame that asserts a fact about the world")
    say('--> {"t": "set_state", "money": 5000}')
    world.socket.send({"t": "set_state", "money": 5000})
    say(f"<-- {json.dumps(world.until('result', 'refused'))}")


if __name__ == "__main__":
    try:
        walk_up_and_talk()
        refusals()
    except (ConnectionRefusedError, OSError) as failure:
        sys.exit(f"no server on {HOST}:{PORT} ({failure})")

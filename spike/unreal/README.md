# Unreal adapter spike — the part that needs no engine

Unreal is not installed here, so this is phase one of the spike `ARC-18` authorised: the
architecture test, and nothing that needs the engine.

The verdict and the plan are in
[`../../docs/references/UNREAL_ADAPTER_SPIKE.md`](../../docs/references/UNREAL_ADAPTER_SPIKE.md).
This folder only holds the evidence behind it.

[`probe.py`](probe.py) is a MineWorld client written with no game engine, no WebSocket library and
no SDK — standard library only. It joins, observes, walks up to Alice, talks to her, and then gets
six things deliberately wrong to record what the server answers. What it needs is the real cost of
a new client; what it does not need is not part of that cost.

```sh
cargo build -p mineworld-cli
./target/debug/mineworld server worlds/social-cafe --listen 127.0.0.1:7878 --agent alice &
python3 spike/unreal/probe.py > spike/unreal/evidence/transcript.log
```

`evidence/` is the captured run: the transcript, `GET /status`, and the server's own log.

# 06 Play it

In this chapter you host Corner Shop and walk around in it as Ben, in the 2D client. The world
keeps running while you play: Ana and Cai keep their day, and Ben's is yours.

<!-- guide: continues 05-one-shop -->

You keep working with the `corner-shop` folder from [chapter 05](05-one-shop.md).

The test that runs this guide checks step 6.1. Steps 6.2 and 6.3 need a window and a person at
the keyboard, so you check them yourself.

### Step 6.1 — See who you can play

A **seat** is a person a player may control. Corner Shop offers three: `ana`, `ben` and `cai`.

**Do this.**

```console
$ mineworld validate corner-shop
...
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**You should see** the three seats.

### Step 6.2 — Host the world

**Do this.** Start a **server**: the program that keeps the world running and lets players join.
`--town` lets the simple rule drive every seat that no player holds, as `run` does.

<!-- guide: operator-only -->

```console
$ mineworld server corner-shop --town
[mineworld] Corner Shop (corner-shop) — 11 system(s), 3 seat(s)
[mineworld] listening on http://127.0.0.1:7878 (ws://127.0.0.1:7878/ws), protocol [..]
[mineworld] 8 entities, 11 system(s), seats: ana, ben, cai
[mineworld] world instance [..]
[mineworld] invite [..] — join with: 127.0.0.1:7878 seat=ana invite=[..]
...
[mineworld] hold 30 s, time scale 1, request budget 10/s burst 20 per seat, 3 seat(s) driven in-server (town: seed 0, pace 5 wall s)
```

**You should see** a line starting `[mineworld] invite`. The long code after `invite` is the
**invite**: a player needs it to join. The last line says the rule drives all three seats until a
player takes one. Time in the world now passes as fast as real time.

The server keeps running until you press Ctrl+C. Leave it running, and open a second terminal for
the next step.

Without `--save`, the world starts again from the beginning each time. To keep it between runs, add
`--save saves/play`, as in [chapter 04](04-three-people.md#step-45--run-a-day-and-keep-it).

### Step 6.3 — Walk around as Ben

The 2D client is in the MineWorld source folder from [chapter 01](01-get-mineworld.md). It needs
[Godot 4.7](https://godotengine.org) on your `PATH`.

**Do this.** In the second terminal, go to the `MineWorld` folder and join the world as Ben. Use the
invite your server printed instead of `INVITE`:

<!-- guide: operator-only -->

```sh
./mineworld-2d --server 127.0.0.1:7878 --invite INVITE --seat ben
```

`mineworld-2d` is a `bash` script. On Windows, the 2D client's own page,
[`clients/2d/README.md`](../../clients/2d/README.md), says what it needs there.

**You should see** a window with Ben's home drawn from above, at an angle, with Ben and Cai in it and
a door. You are Ben now; the rule lets go of him while you hold his seat. Click somewhere to walk.
Walk through the door into the shop. Click Ana for what you can do with her, such as talk. Click
yourself, or press Q, for what you can do on your own, such as buy bread.

Everything you do is a request to the world. The world says yes or no, and the client draws the
answer. A greyed entry in a menu says why the world would say no.

**About the 3D client.** The 3D client today draws the café of the sample town only. Walking around
your own world in 3D is coming.

## Why it works this way

A player, a rule and a program all control a person the same way: through a seat, by asking the
world. That is why Ben could be driven by a rule in chapter 05 and by you here, with no change to the
world. The rules are in [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#14-controller) and
[`NETWORKING.md`](../NETWORKING.md).

Next: [07 Rules of the world](07-rules-of-the-world.md).

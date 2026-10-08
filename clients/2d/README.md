# MineWorld 2D reference client

A real world you walk around in: `mineworld-2d` joins a running MineWorld server as one Person and
draws the town that Person can see, in the default isometric `town` style.

```sh
./mineworld-2d                  # host worlds/market-town locally and play it as Carol
./mineworld-2d --seat visitor   # the same world, as somebody else
./mineworld-2d --server 127.0.0.1:7878 --invite TOKEN   # join a world someone else is hosting,
                                                         # with the invite its server printed
./mineworld-2d --drive=walk     # the scripted checks, headless
```

Click to walk, or use WASD / the arrow keys; Esc quits. Walk out of the apartments, along the street
and into the café. Close the window and open it again: you are where the world left you.

What it is and is not:

- It asks the world for things and draws what the world says. It decides no rule — not distance, not
  whether a door can be used — and a check (`scripts/check_client_rules.py`) holds it to that.
- The art is a separate Presentation Pack, `presentation/mineworld-default/2D`, read from disk at
  runtime. `--presentation=none` draws plainly; `--variant=` picks one of the four style sets.

What it looks like today: [`shots/preview/`](shots/preview/) (a preview, not an accepted look).

For agents: the pack format is [`PRESENTATION.md`](PRESENTATION.md); the design and its acceptance
are `.structured-coding/plans/mvp0/step-13-client-2d.md` §14.

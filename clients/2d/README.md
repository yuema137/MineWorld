# MineWorld 2D reference client

A real world you walk around in: `mineworld-2d` joins a running MineWorld server as one Person and
draws the town that Person can see, in the default isometric `town` style.

```sh
./mineworld-2d                  # host worlds/market-town locally, with its town alive; pick a person
./mineworld-2d --world worlds/lakeside   # another world: its own people are listed
./mineworld-2d --join-local     # a second window: join the world the first one hosts
./mineworld-2d --seat visitor   # skip the join screen and play as the visitor
./mineworld-2d --no-town        # nobody else moves (a quiet world, for debugging)
./mineworld-2d --server 127.0.0.1:7878 --invite TOKEN   # join a world someone else is hosting,
                                                         # with the invite its server printed
./mineworld-2d --drive=walk     # the scripted checks, headless
```

The join screen lists the people the world offers. Pick one and press Join. If somebody else is
already playing that person you are told so, and you may press **Take over**; they are told who took
it and return to their own join screen. The other people of the town walk, talk and do things on their
own. A line spoken near you appears as a bubble over the speaker; an activity starting near you
appears as a short notice. If your connection drops, you come back as the same person.

Click to walk, or use WASD / the arrow keys. Esc opens the settings menu: language (English or 简体中文),
12- or 24-hour clock, window and display, and a Controls tab with the key map (read-only); Quit is there too. Settings stay on your machine and are
shared with the 3D client. Walk out of the apartments, along the street and into the café. Close the
window and open it again: you are where the world left you.

Click a person (or press E over them) for what you can do with them; click yourself (or press Q) for
what you can do on your own — buy, eat, drink, leave an activity. Every entry is something the world
offered; a greyed one says why not, and you may still try it. I shows your things, H your
conversations.

Balls and boxes lying in a room are drawn where the world says, and move the way it says. Click one
for what you can do with it — kick it, throw it, or "Throw the ball at…" and then click the floor where
it should go (Esc or a right click cancels). Try it in `worlds/bodies-yard`:

```sh
./mineworld-2d --world worlds/bodies-yard --seat ada
```

```sh
./mineworld-2d --seat visitor     # in the café: talk to Alice, buy a coffee, give Bob the scarf
./mineworld-2d --seat bob         # a second window: accept the visitor's invitation
```

What it is and is not:

- It asks the world for things and draws what the world says. It decides no rule — not distance, not
  whether a door can be used — and a check (`scripts/check_client_rules.py`) holds it to that.
- Your needs, your sleep and the next part of your day are in the panels ("How you feel", "Asleep"), and
  a sleeper shows a "Zz"; the words are the packs', not the client's.
- The art and the wording are a separate Presentation Pack, `presentation/mineworld-default/2D`,
  read from disk at runtime (`i18n/en.po`, `i18n/zh_Hans.po`; wording both clients use is in
  `clients/shared/settings/locale/`). `--presentation=none` draws plainly; `--variant=` picks one of
  the four style sets.
- On Windows, clone with `git config core.symlinks true` (Developer Mode): `mineworld` and
  `mineworld_settings` here are symlinks to the shared modules.

What it looks like today: [`shots/preview/`](shots/preview/) (a preview, not an accepted look).

For agents: the pack format is [`PRESENTATION.md`](PRESENTATION.md); the design and its acceptance
are `.structured-coding/plans/mvp0/step-13-client-2d.md` §14 (walking) and §15 (interactions).

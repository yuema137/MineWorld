# MineWorld greybox presentation

A plain 3D Presentation Pack. It draws a world with no art of its own, so you can see that changing
the pack changes the look and nothing else.

```sh
./mineworld-slice --world --presentation=presentation/mineworld-greybox/3D
```

You should see the same café and the same people as with the default pack, but every person is the
client's plain figure and every table and chair is a plain box. The server, the rules and what you can
do are unchanged.

| File | What it is |
| --- | --- |
| `pack.yaml` | the pack's identity (`type: presentation-pack`) |
| `manifest.yaml` | the style: `dimension: [3d]` |
| `far_field/` | the distant view of each world it presents, copied from the default pack |
| `i18n/` | the 3D client's words, copied from the default pack |

It has no cast and no furniture bindings on purpose. Start a new world's pack from this one.

What a 3D pack may contain: [`clients/3d-spike/PRESENTATION.md`](../../../clients/3d-spike/PRESENTATION.md).
Why the client is given a pack: [`docs/decisions/ARC-51.md`](../../../docs/decisions/ARC-51.md).

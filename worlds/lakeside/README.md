# `lakeside`

A village on a lake: ten people who bake, mend boats, talk on the square, and fish from the shore and
the pier. It is Milestone E's world ([`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-77`): built
only from packs, none of them written for it.

| Pack | Kind | Where it comes from |
| --- | --- | --- |
| presence, movement, conversation, group-activity, relationships, naming, schedule, item, inventory, item-transfer, consumption | System Packs | bundled: compiled from this repository |
| `acme-fishing` (system `fishing`) | System Pack | third-party: [mineworld-pack-fishing](https://github.com/yuema137/mineworld-pack-fishing), pinned to one commit in `systems/installed` |
| `modern-goods` | Entity Pack | [`entities/modern-goods`](../../entities/modern-goods): the bread, milk and coffee people start with |
| `mineworld-default-2d`, `mineworld-default-3d` | Presentation Packs | `presentation/mineworld-default` (declared; the clients do not draw Lakeside yet) |

The fish is the lake's own kind (`items/fish.yaml`). There is no money here: what people eat is what
they started with, and what they catch.

The data packs are found only where you point, so every command names the two roots. From the
repository's root:

```sh
mineworld validate worlds/lakeside --packs entities --packs presentation/mineworld-default
mineworld run worlds/lakeside --headless --seed 7 --days 30 \
    --packs entities --packs presentation/mineworld-default
```

## Try it (the milestone checklist)

```sh
R="--packs entities --packs presentation/mineworld-default"
mineworld packs list $R                          # every pack: the third-party one, the Entity Pack
mineworld packs resolve worlds/lakeside $R       # which pack provides each requirement and system
mineworld run worlds/lakeside --headless --seed 7 --days 30 --save /tmp/lake $R
mineworld inspect /tmp/lake                      # every cause resolves; `fishing` in the systems
mkdir -p /tmp/elsewhere && cp -R worlds/lakeside /tmp/elsewhere/
mineworld validate /tmp/elsewhere/lakeside $R    # the world needs no place in this repository
cargo test -p mineworld-cli --test milestone_e   # all of it, plus 300 days and the refusals
```

What to look for: `packs list` marks the fishing pack `third-party`; the run summary counts
`fish accepted` and `items-produced`; take `fishing` out of `systems:` (and the two `fishing:` lines)
and the same world runs on with nothing but the fishing facts gone.

How it is proven: [`step-16-packages.md`](../../.structured-coding/plans/mvp0/step-16-packages.md) §8.2
and §18.

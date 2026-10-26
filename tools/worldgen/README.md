# World tiler

`tile.py` copies a World Pack several times to make a bigger one. Each copy is a separate "block" of the same
town, and the blocks are not connected to each other. In block *b*, every place, person and item gets the key
`<key>-<bb>`, and every reference inside the block points to that block's own copy. The nightly memory scale gate
uses it to build a 100-person town from `social-cafe`.

```sh
uv run --locked python tools/worldgen/tile.py --source worlds/social-cafe --people 100 --id town-100 \
    --out /tmp/town-100 --unique name --drop settlement
```

- `--unique name` adds ` <b>` to each person's `name:` in blocks 2 and later, so no two people share a name.
- `--drop settlement` leaves out the café's single station hall, because people in other blocks cannot reach it.

The tiler does not keep a list of which fields are references. It reads them from the generated schemas in
`docs/reference/schema/`. It refuses, by name, any pack it cannot copy faithfully: organizations, a decision
list, a section that has no schema, a configure file that contains a reference, or a reference to a key that is
not defined.

The design and its rules are in
[`pr-s28g-scale-gates.md`](../../.structured-coding/plans/mvp0/pr-s28g-scale-gates.md) §5.2.

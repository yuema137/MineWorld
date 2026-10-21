# Recorded observations that name items: where they come from

Evidence for the item-name readers of the agent's view (`clients/agent/tests/test_item_names.py`) and the 2D
panels (`tools/cli/tests/item_names.rs`, the ignored `the_2d_panels_name_every_item_of_every_fixture`). Each
`<world>.json` is a JSON array of `Observation`s, exactly as the real `mineworld server` sent them, one file per
shipped world: every directory under `worlds/`, and the `commute-town` test world.

**Recorder.** `tools/cli/tests/item_names.rs`, the test `every_item_an_observation_names_is_named_by_its_place`:

```sh
MINEWORLD_WRITE_FIXTURES=1 cargo test -p mineworld-cli --test item_names -- --test-threads=1
```

It starts `mineworld server <world> --packs entities --packs presentation/mineworld-default`, joins every seat
and keeps its first observation, then walks the first seat into every shop (the place each of the world's
organizations names in `economy: shop: at:`) and keeps the observation in which it arrived. A file keeps each
observation that adds an item not kept before, or is the first in a shop.

**What a name is.** An item is never a perceived entity (`ARC-37`). An observation names it in one of two
records on the observer's place: the `item` pack's `item-catalogue` (`{kinds: [{item, category, name}]}`), or,
for a loose object, which is not a kind (`ARC-36`), the `tags` of its entry in bodies' `loose-objects` listing.
The expected values in the tests are read from these files, never from the code under test.

**Hand edits: none.** A re-recording that changes these files is a reviewed change; say why in the commit.

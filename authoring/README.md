# authoring/

**`mineworld-authoring`**: the seam between a World Pack's authored files and the System Packs
that own what those files say.

A person's file may carry **sections** that a System Pack declares as its own: `name:` is
`naming`'s, `routine:` is `schedule`'s. The pack implements `AuthoredSection`, which says:

- the key it owns;
- which files may carry it;
- the type that validates it;
- which other entities it names;
- the genesis facts it becomes.

The World Pack loader decodes the section with that type and records those facts. It never learns
what the section means.

So adding a pack that reads authored content adds a pack, not a field in the loader.

The decision is [`ARC-31`](../docs/DECISIONS.md). The format is
[`MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §4.1.

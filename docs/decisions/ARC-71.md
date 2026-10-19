## ARC-71 — An Entity Pack in MVP-0 is a directory of item kinds a world requires

**Date** 2026-10-08 · **Approved by** the primary session at PR E-d's design freeze (step-16 §17.0;
FQ-d1 … FQ-d6) and the operator's every-platform requirement (§17.12) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §2, §4.1, §8.1; [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0, §8 ·
**Relates to** `ARC-31`, `ARC-36`, `ARC-53`, `ARC-54`, `ARC-55`, `DEP-10` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §4.3, §17 (S16, PR E-d)

**Problem.** `MODULE_SPEC.md` §2 defines an Entity Pack — "what exists in the world" — and `ARC-53`
gave it an identity, but nothing read one: `pack.yaml` refused `type: entity-pack` and `requires:`
refused to resolve one. A kind of thing (bread, coffee) could exist only inside the one world that
declared it, so two worlds sharing goods had to copy the files. "Install modules → compose world" needs
a kind of thing to be installable on its own, with no rebuild.

**Choice.**

1. **Item kinds and nothing else, in MVP-0.** An Entity Pack is a directory holding `pack.yaml`
   (`type: entity-pack`, `ARC-53`'s fields, no new field) and `items/<key>.yaml`, each in the World Pack
   item-file format (`tags`, `note`, sections, `ARC-36`), read by the same code. A `places/`, `people/`
   or `organizations/` directory in it is refused by name ("an Entity Pack carries item kinds only in
   MVP-0"); one with no item file is refused ("declares nothing").
2. **The directory is the declaration** (FQ-d1). Every `*.yaml` in `items/` is one kind, keyed by the
   file's stem (`Path::file_stem` of an entry whose `Path::extension` is `yaml`), which must be an
   `EntityKey` or the file is refused naming it. Any other file in `items/` is not content and is left
   alone. `pack.yaml` has no `items:` list, so the key is stated once, by the file name.
3. **Who reads what.** `mineworld-packages` identifies an Entity Pack and checks its directory layout
   (file names only; it stays a leaf that knows no `EntityKey`); `mineworld-worldpack` reads its content
   in `worldpack/src/entities.rs`, because the item-file format and section decoding are its.
4. **Composition: read order step 6b.** After the world's own content is read and checked for
   undeclared files, each requirement that resolved to an Entity Pack, in id order, has its `items/`
   read **with the requiring world's enabled systems**, so a section is decoded by its owner exactly as
   if the world had written it, and the section rules (owner enabled, file may carry it, references
   declared and typed — `ARC-31`) apply unchanged. The kinds merge into the world's item kinds.
5. **One key namespace, refused by name** (`MODULE_SPEC.md` §4.1 rule 1). A key an Entity Pack declares
   that the world also declares — in `places`, `population`, `items` or `organizations` — or that another
   required Entity Pack declares, is refused naming the key and both sources. Never "last one wins":
   Minecraft data packs' override-by-load-order is exactly the silent shadowing refused here.
6. **Identity in key order across every source** (FQ-d2). Item kinds — the world's and every required
   pack's together — are allocated after people and before organizations, in key order, by the loop
   that already exists. A world that requires no Entity Pack allocates exactly what it did; moving kinds
   out of a world into an Entity Pack it then requires changes no id and no fact.
7. **Provenance names the pack.** A kind read from an Entity Pack carries `Metadata { source_pack: <the
   pack's id>, source_path: "items/<key>.yaml" }`, `source_path` written with `/` on every operating
   system so a save does not depend on the machine. Every refusal that names a content file names the
   file in the Entity Pack's directory. `Metadata` is free text already: no contract change.
8. **Self-contained** (FQ-d3). A section in an Entity Pack's item file may name only keys the same pack
   declares; any other key is refused naming the pack, the file and the key. In MVP-0 the only item
   section with references is `bodies`' object form (`at: { place }`), which describes one loose object,
   not a kind (`ARC-36` note), so it is refused in an Entity Pack, as intended.
9. **What the CLI shows** (FQ-d5). `mineworld packs validate <entity pack>` checks the identity, the
   licence policy and the framework range (the `ARC-54` note below), then reads every item file against
   **the build's whole installed set** — each section decoded by its owner; whether the owner is enabled
   is a world's question, answered at resolution — and rule 8, and prints the kinds. `packs list` lists
   `entity-pack` lines. `validate <world>` prints the composed item kinds (the world's and its packs') on
   its existing `items` line.
10. **Every platform.** CRLF line endings in `pack.yaml`, `items/*.yaml` and `world.yaml` read exactly
    as LF does, relying on `serde-saphyr` (`DEP-10`); `items/` is read into a map by key, so no file
    system's listing order can change an id; paths are built with `Path` operations, never by splitting
    text on `/`.

**Options considered.** An `items:` list in `pack.yaml` mirroring `world.yaml`'s (FQ-d1) — rejected:
the key would be stated twice. Pack kinds allocated after organizations (FQ-d2) — rejected: one order,
and moving kinds into a pack would then move ids. Namespaced keys (`goods:bread`, as Minecraft and
Factorio prototypes do) — rejected for MVP-0: `EntityKey` is a contract type and every reference would
change. A generic schema-driven content loader — rejected: a second statement of what each owner
already decodes with its own type.

**Accepted limitations.**
- Item kinds only: new entity types would extend the closed `EntityType` in `contracts`, and component
  schemas belong to System Packs in MVP-0 (`ARC-31` sections). Authoring templates are not read.
- No namespaced keys: two packs that both declare `tea` cannot be required by one world.
- No pack-to-pack dependency: `pack.yaml` still refuses `dependencies`, so an Entity Pack cannot build
  on another.
- A stray `.yaml` in `items/` becomes a kind; `packs validate` lists every kind so an author sees it.
- A world that adopts an Entity Pack shifts the ids of its later item kinds and organizations, as
  editing its own `items:` does today. Saves do not record which packs they used (QSE-14).

**Note, 2026-10-12 (S15, PR 12d; step-11 §19.13 TD-D12 item 3, the operator's ruling) — kinds carry a
display name, and `modern-goods` is 0.2.0.** `item`'s section now requires `name` (step-11 SD-D10,
R-PK-2; `ARC-37` note), so an Entity Pack's kinds state one like any world's. `modern-goods`' twelve
kinds gain theirs (the key in plain English, as Market Town's: Apple, Bread, …, Umbrella), and the
facts a world requiring the pack declares change, so its version moves **0.1.0 → 0.2.0**; a world
requires it as `"^0.2"`. Nothing else in the rules above changes. A third-party Entity Pack written
before this note is refused at load until each kind has a `name`, naming the file and line.

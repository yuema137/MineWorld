# 12 Packs

In this chapter you see which packs your `mineworld` has, and which packs Corner Shop uses. Then you
make a small **Entity Pack** of your own, with one kind of goods, and let Corner Shop use it.

<!-- guide: continues 08-time-and-weather -->

You keep working in the `corner-shop` folder from [chapter 08](08-time-and-weather.md).

A **pack** is something you can install and share. Every System Pack is one. Others hold data, such
as an Entity Pack: a set of ready-made things, such as kinds of goods, that any world may use
([the six kinds of pack](../MODULE_SPEC.md#1-six-pack-types)).

### Step 12.1 — See what this build has

**Do this.**

```console
$ mineworld packs list
system-pack        mineworld-presence          0.1.0   MIT  Yue Ma  bundled  system presence
system-pack        mineworld-movement          0.1.0   MIT  Yue Ma  bundled  system movement
system-pack        mineworld-conversation      0.1.0   MIT  Yue Ma  bundled  system conversation
...
[..] packs
```

**You should see** one line per pack: its type, its name, its version, its licence, its authors and
where it comes from. `bundled` means it is built into this `mineworld`. The last word of a System
Pack's line, such as `presence`, is the name a world lists under `systems`.

A System Pack can also come from someone else's code. Such a pack says `third-party` instead of
`bundled`. Adding one means building `mineworld` again with it; how is in
[`MODULE_SPEC.md`](../MODULE_SPEC.md#31-installing-a-system-pack-in-mvp-0).

### Step 12.2 — See which packs Corner Shop uses

**Do this.**

```console
$ mineworld packs resolve corner-shop
Corner Shop (corner-shop)
  framework  0.1.0 (mineworld: "^0.1")
  system     presence → mineworld-presence 0.1.0 (bundled)
  system     movement → mineworld-movement 0.1.0 (bundled)
...
  system     weather → mineworld-weather 0.1.0 (bundled)
corner-shop resolves.
```

**You should see** each System Pack that Corner Shop lists, and the pack that provides it.

### Step 12.3 — Start an Entity Pack

**Do this.** Make a folder `packs`, and in it a folder `corner-goods`. In `packs/corner-goods`, write
`pack.yaml`:

```yaml file=packs/corner-goods/pack.yaml
id: corner-goods
type: entity-pack
version: 0.1.0
mineworld: "^0.1"
license: MIT
authors:
  - Your Name
```

A pack's `pack.yaml` says what it is: its `id` (the same as its folder's name), its `type`, its
`version`, the MineWorld versions it works with, its licence and its authors. Put your own name in
`authors`. The fields are in the reference: [`pack.yaml`](../reference/pack-manifest.md#packyaml).

```console exit=1
$ mineworld packs validate packs/corner-goods
[mineworld] packs/corner-goods: its pack.yaml states an entity-pack, which declares nothing — it needs items/<key>.yaml, one file per item kind, or a .yaml file in a data directory an installed System Pack owns
```

**You should see** a refusal: an Entity Pack must hold at least one thing.

### Step 12.4 — Put jam in it

**Do this.** In `packs/corner-goods`, make a folder `items`. In it, write `jam.yaml`:

```yaml file=packs/corner-goods/items/jam.yaml
tags:
  - food
item:
  category: food
```

It is written just like the bread in [chapter 05](05-one-shop.md#step-52--write-the-bread).

```console
$ mineworld packs validate packs/corner-goods
corner-goods
  type        entity-pack
  version     0.1.0
  license     MIT
  authors     Your Name
  repository  —
  mineworld   ^0.1
  source      packs/corner-goods
  items       jam
packs/corner-goods is a valid entity-pack.
```

**You should see** `packs/corner-goods is a valid entity-pack.`

### Step 12.5 — Let Corner Shop require the pack

**Do this.** Replace `corner-shop/world.yaml` with this whole file. The new part is `requires`, after
`mineworld`:

```yaml file=corner-shop/world.yaml
# Corner Shop: the world this guide builds.
world:
  id: corner-shop
  name: Corner Shop
  version: 0.1.0
  license: MIT

mineworld: "^0.1"

requires:
  corner-goods: "^0.1"

systems:
  - presence
  - movement
  - conversation
  - relationships
  - naming
  - schedule
  - item
  - inventory
  - economy
  - employment
  - consumption
  - calendar
  - weather

places:
  - home
  - shop

population:
  - ana
  - ben
  - cai

items:
  - bread
  - tea

organizations:
  - corner-shop-co

seats:
  - ana
  - ben
  - cai

configure:
  - economy
  - classes
  - conversation
  - calendar
  - weather
```

`requires` names each pack the world needs that is not built into `mineworld`, with the versions it
accepts. The world does not list `jam` under `items`: the pack brings it.

```console exit=1
$ mineworld validate corner-shop
[mineworld] corner-shop/world.yaml: requires: corner-goods is not in this build, and no pack directory was given (--packs, MINEWORLD_PACKS)
```

**You should see** a refusal: `mineworld` does not know where to look for `corner-goods`.

### Step 12.6 — Say where the packs are

**Do this.** Add `--packs packs`: the folder whose sub-folders are packs.

```console
$ mineworld validate corner-shop --packs packs
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule, item, inventory, economy, employment, consumption, calendar, weather
  places     home, shop
  people     ana, ben, cai
  items      bread, jam, tea
  organizations corner-shop-co
  seats      ana, ben, cai
  requires   corner-goods "^0.1" → 0.1.0 (packs/corner-goods)
...
corner-shop is a valid World Pack.
```

**You should see** `jam` among the items, and a `requires` line saying where `corner-goods` was
found. From now on, every command on Corner Shop needs `--packs packs`, like this one:

```console
$ mineworld packs resolve corner-shop --packs packs
Corner Shop (corner-shop)
  framework  0.1.0 (mineworld: "^0.1")
  requires   corner-goods "^0.1" → entity-pack 0.1.0 (packs/corner-goods)
...
corner-shop resolves.
```

Nobody sells jam yet. To sell it, add it to the shop's `prices` and `holdings` in
`corner-shop/organizations/corner-shop-co.yaml`, as in [chapter 05](05-one-shop.md#step-54--write-the-company-and-its-shop).

**The goods that come with MineWorld.** MineWorld's source has an Entity Pack called `modern-goods`
in its `entities` folder, with everyday goods such as bread, milk and coffee. A world uses it the
same way: `modern-goods: "^0.1"` under `requires`, and `--packs` with the folder that holds it.

**Help from your editor (optional).** Every field in this guide is also described as a JSON Schema
file in [`reference/schema/`](../reference/schema/world.schema.json). An editor with YAML support can
read these files to complete keys and mark mistakes as you type. The guide's files do not need them.

## Why it works this way

A world names what it needs, with versions, and `mineworld` finds and checks each pack before the
world starts. A pack's identity and licence are checked too. The rules are in
[`MODULE_SPEC.md`](../MODULE_SPEC.md#2-entity-pack) and [`PACKAGE_FORMAT.md`](../PACKAGE_FORMAT.md).

Next: [13 When something is refused](13-when-something-is-refused.md).

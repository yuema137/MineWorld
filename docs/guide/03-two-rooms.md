# 03 Two rooms

In this chapter you add a second room, the shop, and a door between it and the home. Then Ana
starts the day in the shop, and people walk between the rooms.

<!-- guide: continues 02-your-first-world -->

You keep working in the `corner-shop` folder from [chapter 02](02-your-first-world.md).

### Step 3.1 — List the shop in the world

**Do this.** Replace `corner-shop/world.yaml` with this whole file. The only change is `shop`
under `places`:

```yaml file=corner-shop/world.yaml
# Corner Shop: the world this guide builds.
world:
  id: corner-shop
  name: Corner Shop
  version: 0.1.0
  license: MIT

mineworld: "^0.1"

systems:
  - presence
  - movement
  - conversation

places:
  - home
  - shop

population:
  - ana
  - ben

seats:
  - ana
  - ben

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the place 'shop', but corner-shop/places/shop.yaml does not exist (a place's key names its file)
```

**You should see** the check ask for the shop's file.

### Step 3.2 — Write the shop

**Do this.** In `corner-shop/places`, write `shop.yaml`:

```yaml file=corner-shop/places/shop.yaml
tags:
  - shop
note: A small shop next door, with a counter at the back.
```

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation
  places     home, shop
  people     ana, ben
  seats      ana, ben
  settlement none
...
corner-shop is a valid World Pack.
```

**You should see** `places     home, shop`. The world is valid, but nobody can reach the shop yet:
there is no door.

### Step 3.3 — Put a door between the rooms

A door between two places is a **passage**. You write it once, in one of the two place files, and
it works both ways.

**Do this.** Replace `corner-shop/places/home.yaml` with:

```yaml file=corner-shop/places/home.yaml
tags:
  - home
note: Two flats behind one front door.
passages:
  - to: shop
    here:
      x: 4000
      y: 0
    there:
      x: 2000
      y: 0
```

`to: shop` says where the door leads. `here` is where the door is in the home: four metres along
its front wall (`y: 0`). `there` is where the same door is in the shop: two metres along the shop's
front wall. Both are in millimetres. The fields are in the reference:
[`AuthoredPassage`](../reference/world-pack.md#authoredpassage).

```console
$ mineworld validate corner-shop
...
3 genesis fact(s): the world's initial state, each one caused by the world coming into existence
corner-shop is a valid World Pack.
```

**You should see** `3 genesis fact(s)` instead of 2. A **genesis fact** is something true when the
world begins. The new one is that the door is open.

### Step 3.4 — Start Ana in the shop

**Do this.** Replace `corner-shop/people/ana.yaml` with:

```yaml file=corner-shop/people/ana.yaml
tags:
  - shopkeeper
note: Runs the corner shop.
location:
  place: shop
  position:
    x: 2000
    y: 4000
```

Ana now starts behind the counter: two metres along the shop, four metres in from the door.

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is still valid.

**If you see** `puts 'ana' in the place 'shp', which this pack does not declare`, the place's key is
misspelt. See [When something is refused](13-when-something-is-refused.md#step-134--a-person-in-a-place-that-does-not-exist).

### Step 3.5 — Run it and watch people walk

**Do this.**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1
[mineworld] run Corner Shop (corner-shop): seats ana, ben; seed 1; to day 1; pace 900s; in memory
...
requests   walk-to accepted [..]
...
facts      person-entered-place [..]
...
faults     0
...
```

**You should see** `walk-to accepted` and `person-entered-place` in the summary. A `walk-to` is a
request to walk somewhere. `person-entered-place` counts the times someone went through the door
into the other room.

## Why it works this way

A place in MineWorld is a name the world understands, such as `shop`. Positions inside it are only
for walking and drawing. That is why the door is written as "to the shop", with a position at each
end. The rules are in [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#6-place) and
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads).

Next: [04 Three people](04-three-people.md).

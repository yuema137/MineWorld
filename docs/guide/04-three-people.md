# 04 Three people

In this chapter you add a third person, Cai. You give everyone a name and a daily routine, and
people get to know each other. At the end you read what happened to Ana on her first day.

<!-- guide: continues 03-two-rooms -->

You keep working in the `corner-shop` folder from [chapter 03](03-two-rooms.md).

### Step 4.1 — Add Cai and three System Packs

Names, routines and getting to know people each come from a System Pack:

- `relationships` lets people become acquainted when they meet and talk.
- `naming` gives each person a name, the one others see.
- `schedule` gives each person a routine: where they mean to be at each time of day.

**Do this.** Replace `corner-shop/world.yaml` with:

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
  - relationships
  - naming
  - schedule

places:
  - home
  - shop

population:
  - ana
  - ben
  - cai

seats:
  - ana
  - ben
  - cai
```

The three new System Packs come after the first three. They do not depend on each other.

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'cai', but corner-shop/people/cai.yaml does not exist (a person's key names its file)
```

**You should see** the check ask for Cai's file.

### Step 4.2 — Write Cai, with a name and a routine

**Do this.** In `corner-shop/people`, write `cai.yaml`:

```yaml file=corner-shop/people/cai.yaml
name: Cai Wen
tags:
  - resident
note: Lives in the flat downstairs.
location:
  place: home
  position:
    x: 7000
    y: 3000
routine:
  - from: "16:00"
    place: shop
    label: shopping
  - from: "18:00"
    place: home
    label: home
```

Two new parts:

- `name` is what other people see. The key `cai` stays the file's name.
- `routine` is Cai's day, in parts. From 16:00 Cai means to be in the shop, shopping. From 18:00,
  at home. Before 16:00 the last part still holds, so Cai starts the day at home. Write each time
  in quotes, as `"16:00"`.

A routine is a plan, not a move: Cai still has to walk there. The fields are in the reference:
[`name`](../reference/sections.md#name) and [`routine`](../reference/sections.md#routine).

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule
  places     home, shop
  people     ana, ben, cai
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**You should see** three people and three seats.

### Step 4.3 — Give Ana a name and a routine

**Do this.** Replace `corner-shop/people/ana.yaml` with:

```yaml file=corner-shop/people/ana.yaml
name: Ana Lee
tags:
  - shopkeeper
note: Runs the corner shop.
location:
  place: shop
  position:
    x: 2000
    y: 4000
routine:
  - from: "07:30"
    place: shop
    label: work
  - from: "18:30"
    place: home
    label: home
```

Ana works in the shop from half past seven, and goes home at half past six.

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is valid.

### Step 4.4 — Give Ben a name and a routine

**Do this.** Replace `corner-shop/people/ben.yaml` with:

```yaml file=corner-shop/people/ben.yaml
name: Ben Okafor
tags:
  - resident
note: Lives in the flat upstairs.
location:
  place: home
  position:
    x: 4000
    y: 3000
routine:
  - from: "09:00"
    place: shop
    label: shopping
  - from: "11:00"
    place: home
    label: home
```

Ben shops in the morning, from nine to eleven.

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is valid.

### Step 4.5 — Run a day and keep it

So far every run was forgotten when it ended. To read what happened afterwards, keep the run in a
**save**: a folder that holds the world and everything that happened in it.

**Do this.**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1 --save saves/day-1
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 1; pace 900s; created saves/day-1/world.sqlite
...
faults     0
...
```

**You should see** `created saves/day-1/world.sqlite` in the first line. The save is a new folder,
`saves/day-1`, next to `corner-shop`.

### Step 4.6 — Read Ana's day

A person's **biography** is the list of things that happened to them that matter to their life
story, such as meeting someone. It is read from the save
([biography](../CORE_CONCEPTS.md#52-objective-biography)).

**Do this.**

```console
$ mineworld biography corner-shop --save saves/day-1 --person ana
biography  ana "Ana Lee" (entity 3) in corner-shop: [..] entries from [..] facts; biographical: [..]
...
[..]day 1    01:30:00  #[..]became-acquainted        at home       with ben "Ben Okafor"
...
[..]day 1    02:00:00  #[..]became-acquainted        at home       with cai "Cai Wen"
...
[..]day 1    07:30:00  #[..]agenda-changed           at -          with -
[..]day 1    18:30:00  #[..]agenda-changed           at -          with -
```

**You should see** a list of times and events. Each line is one thing that happened to Ana:

- At 01:30 she got to know Ben, at home, and at 02:00 Cai. Each meeting is listed twice: once for
  Ana coming to know the other person, and once for the other person coming to know her.
- At 07:30 and 18:30, `agenda-changed`: a new part of her routine began.

Why was Ana at home at night, when her file says she starts in the shop? Before 07:30 the last part
of her routine holds, and that part is home. So she walked home first, and met the others there.

## Why it works this way

Each System Pack owns its own part of a person's file: `naming` owns `name`, and `schedule` owns
`routine`. A world that does not list a System Pack cannot use its part; the check refuses it by
name. The rules are in [`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads); each
part's fields are in [the sections reference](../reference/sections.md#sections).

Next: [05 One shop](05-one-shop.md).

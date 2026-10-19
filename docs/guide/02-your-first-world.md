# 02 Your first world

In this chapter you write a world from an empty folder: one room and two people who live there.
Chapters 03 to 05 grow it into **Corner Shop**: two rooms, three people and one shop.

A world is a **World Pack**: a folder of plain text files in a format called YAML. Each file is
short. You can write them in any text editor. Save each file under the exact name the step gives,
inside your worlds folder from [chapter 01](01-get-mineworld.md). Make folders as you need them.

In YAML, `key: value` sets one value. Lines that start with `- ` make a list. Indentation, with
spaces, shows what belongs to what. A line starting with `#` is a comment for people; MineWorld
ignores it.

### Step 2.1 — Write the world's main file

Every World Pack has a `world.yaml`. It names the world, lists everything in it, and says which
**System Packs** it uses. A System Pack is a module that makes one kind of thing happen, such as
walking or talking ([System](../CORE_CONCEPTS.md#13-system)).

**Do this.** Make a folder called `corner-shop`. In it, write `world.yaml`:

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

population:
  - ana
  - ben

seats:
  - ana
  - ben
```

What each part means:

- `world` gives the world's `id` and `name`. The `id` must be the folder's name.
- `mineworld: "^0.1"` says which versions of MineWorld this world is written for.
- `systems` are the System Packs it uses, in order: `presence` (being somewhere), `movement`
  (walking) and `conversation` (talking). The second and third need the first, so it comes first.
- `places` and `population` list the world's places and people by **key**. A key is a short name
  in lowercase. Each key names a file.
- `seats` are the people a player or a program may control.

Every field is listed in the reference: [`world.yaml`](../reference/world-pack.md#worldyaml).

Now ask MineWorld to check the world:

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the place 'home', but corner-shop/places/home.yaml does not exist (a place's key names its file)
```

**You should see** a refusal. That is expected: `world.yaml` lists a place called `home`, and there
is no file for it yet. Whenever something is missing or wrong, `validate` names it. You add the
files it asks for in the next steps.

### Step 2.2 — Write the room

A **Place** is somewhere a person can be ([Place](../CORE_CONCEPTS.md#6-place)).

**Do this.** Make a folder `corner-shop/places`. In it, write `home.yaml`:

```yaml file=corner-shop/places/home.yaml
tags:
  - home
note: Two flats behind one front door.
```

`tags` are labels. Programs read them; a 2D or 3D client may use them to choose how to draw the
place. `note` is for people reading the file; MineWorld does not read it. The fields are listed in
the reference: [place files](../reference/world-pack.md#placeskeyyaml).

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'ana', but corner-shop/people/ana.yaml does not exist (a person's key names its file)
```

**You should see** that the place is accepted, and the check now asks for the person `ana`.

### Step 2.3 — Write the first person

A **Person** is someone who lives in the world ([Person](../CORE_CONCEPTS.md#4-person)).

**Do this.** Make a folder `corner-shop/people`. In it, write `ana.yaml`:

```yaml file=corner-shop/people/ana.yaml
tags:
  - shopkeeper
note: Runs the corner shop.
location:
  place: home
  position:
    x: 2000
    y: 3000
```

`location` says where Ana starts: in `home`, at a **position**. Positions are in millimetres from a
corner of the room: `x: 2000` is two metres across, `y: 3000` is three metres in. The fields are
listed in the reference: [person files](../reference/world-pack.md#peoplekeyyaml).

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'ben', but corner-shop/people/ben.yaml does not exist (a person's key names its file)
```

**You should see** that Ana is accepted, and the check now asks for `ben`.

### Step 2.4 — Write the second person

**Do this.** In `corner-shop/people`, write `ben.yaml`:

```yaml file=corner-shop/people/ben.yaml
tags:
  - resident
note: Lives in the flat upstairs.
location:
  place: home
  position:
    x: 4000
    y: 3000
```

Ben stands two metres from Ana. A voice reaches three metres by default, so they can talk without
moving. That distance belongs to the conversation System Pack; it is its `range`
([conversation's parameters](../reference/interaction-list.md#parameters-of-configureconversationyaml)).

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation
  places     home
  people     ana, ben
  seats      ana, ben

  1  home
  2  ana
  3  ben

2 genesis fact(s): the world's initial state, each one caused by the world coming into existence
corner-shop is a valid World Pack.
```

**You should see** `corner-shop is a valid World Pack.` at the end. Above it, the check lists what
the world has. The numbered lines are the world's **entities**: every place and person gets one
number when the world begins.

### Step 2.5 — Run it for a day

**Do this.**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1
[mineworld] run Corner Shop (corner-shop): seats ana, ben; seed 1; to day 1; pace 900s; in memory
day 1  revision -  facts [..]
...
requests   talk accepted [..]
...
activity   days 1-1  ana move [..] talk [..]  ben move [..] talk [..]
faults     0
...
```

**You should see** a line for day 1, then a summary. `requests   talk accepted` counts the times
someone asked to talk and the world said yes. The `activity` line shows that Ana and Ben both moved
and talked.

Each person was driven by a simple rule: walk a little, talk to whoever is near. Nobody played them.

## Why it works this way

A World Pack describes what exists. It never states how walking or talking works; that belongs to
each System Pack, which is why the world only lists them. The rules behind this are in
[`MODULE_SPEC.md`](../MODULE_SPEC.md#4-world-pack). The checks `validate` makes are the same checks
every other command makes before it starts a world.

Next: [03 Two rooms](03-two-rooms.md).

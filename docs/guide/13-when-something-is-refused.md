# 13 When something is refused

When a file is wrong, `mineworld` refuses the world and says why, in one message. This chapter shows
each message that this guide can lead you to, what it means, and how to fix it.

You can read it as a list: each step's heading names one kind of mistake. Or you can follow it: you
make a tiny world called `fix-me`, break it on purpose, one mistake at a time, and put it right.

Every message names the file to fix. A refused world does not start, so most mistakes never run
silently. One does: a place no door leads to, which steps 13.16 to 13.19 show.

### Step 13.1 — A file that world.yaml lists does not exist

**Do this.** Make a folder `fix-me`. In it, write `world.yaml`:

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml lists the place 'home', but fix-me/places/home.yaml does not exist (a place's key names its file)
```

**You should see** this refusal. Every key under `places`, `population`, `items` and
`organizations` names a file. **The fix:** write the file it names, or check the key's spelling.

### Step 13.2 — Write the room

**Do this.** In `fix-me/places`, write `home.yaml`:

```yaml file=fix-me/places/home.yaml
tags:
  - home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml lists the person 'ana', but fix-me/people/ana.yaml does not exist (a person's key names its file)
```

**You should see** the same kind of refusal, now for Ana.

### Step 13.3 — Write the person

**Do this.** In `fix-me/people`, write `ana.yaml`:

```yaml file=fix-me/people/ana.yaml
location:
  place: home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**You should see** a valid world. Now break it.

### Step 13.4 — A person in a place that does not exist

**Do this.** Replace `fix-me/people/ana.yaml` with this file, where `home` is misspelt:

```yaml file=fix-me/people/ana.yaml
location:
  place: hom
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml puts 'ana' in the place 'hom', which this pack does not declare (it has: 'home')
```

**You should see** this refusal. It lists the places the world does have. **The fix:** use one of
those keys, or add the place to `places` in `world.yaml` and write its file.

### Step 13.5 — A part of a file whose System Pack the world does not use

**Do this.** Replace `fix-me/people/ana.yaml` with this file, which gives Ana a name:

```yaml file=fix-me/people/ana.yaml
name: Ana Lee
location:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml gives 'ana' a name, which the 'naming' system owns, but world.yaml does not enable it
```

**You should see** this refusal. `name` belongs to the `naming` System Pack, which this world does
not list. **The fix:** add the System Pack to `systems` in `world.yaml`, or take the part out of the
file. The [sections reference](../reference/sections.md#sections) says which System Pack owns each
part.

### Step 13.6 — A misspelt key

**Do this.** Replace `fix-me/people/ana.yaml` with this file, where `location` is misspelt:

```yaml file=fix-me/people/ana.yaml
locaton:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml is not a valid person file:
error: line 1 column 1: unknown field `locaton`, expected one of `tags`, `note`, `location`, [..]
...
```

**You should see** `unknown field`, the line and column, and the keys this file may have.
**The fix:** correct the key. MineWorld never ignores a key it does not know, because a world you
believe you wrote but did not is worse than a refusal.

### Step 13.7 — Put Ana right again

**Do this.** Replace `fix-me/people/ana.yaml` with:

```yaml file=fix-me/people/ana.yaml
location:
  place: home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**You should see** a valid world again.

### Step 13.8 — A door to a place that does not exist

**Do this.** Replace `fix-me/places/home.yaml` with this file, whose door leads to a shop the world
does not have:

```yaml file=fix-me/places/home.yaml
tags:
  - home
passages:
  - to: shop
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/places/home.yaml gives 'home' a passage to 'shop', which this pack does not declare (it has: 'home')
```

**You should see** this refusal. **The fix:** add the other place to `places` and write its file,
as in [chapter 03](03-two-rooms.md#step-31--list-the-shop-in-the-world), or correct the key after
`to`.

### Step 13.9 — Put the room right again

**Do this.** Replace `fix-me/places/home.yaml` with:

```yaml file=fix-me/places/home.yaml
tags:
  - home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**You should see** a valid world again.

### Step 13.10 — A seat that is not a person

**Do this.** Replace `fix-me/world.yaml` with this file, which offers a seat for `ben`:

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ben
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml offers the seat 'ben', which is not one of its people
```

**You should see** this refusal. A seat must be a person listed under `population`: see `seats` in the reference's
[`world.yaml`](../reference/world-pack.md#worldyaml). **The fix:** add
the person, or remove the seat.

### Step 13.11 — A configuration file that does not exist

**Do this.** Replace `fix-me/world.yaml` with this file, which names a `configure` file:

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana

configure:
  - classes
```

```console exit=1
$ mineworld validate fix-me
[mineworld] configure: names 'classes', but fix-me/configure/classes.yaml does not exist
```

**You should see** this refusal. **The fix:** write the file, as in
[chapter 07](07-rules-of-the-world.md#step-73--make-ana-a-shopkeeper-as-a-class), or take the key
out of `configure`.

### Step 13.12 — The world's id is not its folder's name

**Do this.** Replace `fix-me/world.yaml` with this file, whose `id` is `fixme`:

```yaml file=fix-me/world.yaml
world:
  id: fixme
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml calls this pack 'fixme', but its directory is named 'fix-me'; a pack's id is its directory name
```

**You should see** this refusal. **The fix:** make the `id` and the folder's name the same. If you
copy a world to a new folder, change its `id` too.

### Step 13.13 — Put world.yaml right again

**Do this.** Replace `fix-me/world.yaml` with:

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**You should see** a valid world again.

### Step 13.14 — A file that world.yaml does not list

**Do this.** In `fix-me/people`, write `zoe.yaml`:

```yaml file=fix-me/people/zoe.yaml
location:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/zoe.yaml describes a person that world.yaml does not list; add 'zoe' to population, or remove the file
```

**You should see** this refusal. Every file in `people`, `places`, `items` and `organizations` must
be listed, as the reference's [`world.yaml`](../reference/world-pack.md#worldyaml) says. **The fix:** list it, as the next step does, or delete the file.

### Step 13.15 — List Zoe

**Do this.** Replace `fix-me/world.yaml` with:

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana
  - zoe

seats:
  - ana
```

```console
$ mineworld validate fix-me
...
  people     ana, zoe
...
fix-me is a valid World Pack.
```

**You should see** a valid world with two people. Zoe is not a seat, so nobody can play her; a rule
still drives her when the world runs.

### Step 13.16 — A place no door leads to

One mistake is not refused: a place that no door leads to. The world runs, but nobody can walk there.
The next three steps show it, and how to see it.

**Do this.** Replace `fix-me/world.yaml` with this whole file. It adds `schedule`, so Ana can have a
routine, and a second place, `yard`:

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement
  - schedule

places:
  - home
  - yard

population:
  - ana
  - zoe

seats:
  - ana
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml lists the place 'yard', but fix-me/places/yard.yaml does not exist (a place's key names its file)
```

**You should see** the check ask for the yard's file.

### Step 13.17 — Write the yard

**Do this.** In `fix-me/places`, write `yard.yaml`:

```yaml file=fix-me/places/yard.yaml
tags:
  - garden
```

```console
$ mineworld validate fix-me
...
  places     home, yard
...
fix-me is a valid World Pack.
```

**You should see** a valid world with two places. No door joins them, and the check does not mind.

### Step 13.18 — Send Ana to the yard

**Do this.** Replace `fix-me/people/ana.yaml` with this whole file. Its `routine` sends Ana to the
yard at nine and home at noon:

```yaml file=fix-me/people/ana.yaml
location:
  place: home
routine:
  - from: "09:00"
    place: yard
    label: garden
  - from: "12:00"
    place: home
    label: home
```

```console
$ mineworld run fix-me --headless --seed 1 --days 1
...
requests   walk-to rejected System { code: RejectionCode("no-route"), detail: Some("no chain of passages leads to the destination's place") } 11
...
activity   days 1-1  ana move 0 talk 0
faults     0
...
```

**You should see** the run finish with `faults 0`, but every walk to the yard rejected with `no-route`,
and Ana not moving at all. A walk needs a chain of doors to its place. **The fix:** read the `requests`
lines of every run for `rejected`, and give each place a door, as the next step does.

### Step 13.19 — Put a door in

**Do this.** Replace `fix-me/places/home.yaml` with:

```yaml file=fix-me/places/home.yaml
tags:
  - home
passages:
  - to: yard
    here:
      x: 0
      y: 2000
    there:
      x: 0
      y: 0
```

```console
$ mineworld run fix-me --headless --seed 1 --days 1
...
requests   walk-step accepted 52
requests   walk-to accepted 35
...
activity   days 1-1  ana move 52 talk 0
faults     0
...
```

**You should see** walks accepted and no `rejected` line: Ana goes to the yard and back. One door is
enough: a passage can be walked both ways. The fields of a passage are in the reference:
[`places/<key>.yaml`](../reference/world-pack.md#placeskeyyaml).

## Other refusals in this guide

| Message, in part | Where you meet it | The fix |
| --- | --- | --- |
| `configure: names 'calendar', but … does not exist` | [Step 8.1](08-time-and-weather.md#step-81--add-the-calendar-and-the-weather) | write the file the key names |
| `is not a key of a section` | a misspelt key in a `configure` file of the Interaction List, [chapter 07](07-rules-of-the-world.md#step-74--write-the-rule) | use a key from [the shape every section shares](../reference/interaction-list.md#the-shape-every-section-shares) |
| `is neither a role nor a parameter of` | a misspelt parameter in the same files; the message lists the right ones | use a parameter the message or the [reference](../reference/interaction-list.md#parameters-of-configureconversationyaml) lists |
| `which declares nothing — it needs items/<key>.yaml` | [Step 12.3](12-packs.md#step-123--start-an-entity-pack) | add at least one item file to the Entity Pack |
| `requires: … is not in this build, and no pack directory was given` | [Step 12.5](12-packs.md#step-125--let-corner-shop-require-the-pack) | add `--packs` with the folder that holds the pack |

## Why it works this way

Every file is read by the code that owns it, and that code refuses anything it does not understand.
This is why a message can name the file, the line and the key. The rules are in
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads); every field is in the
[reference](../reference/README.md).

Back to [the guide's contents](README.md).

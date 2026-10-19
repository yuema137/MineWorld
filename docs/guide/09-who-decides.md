# 09 Who decides

In this chapter you choose how each person in Corner Shop makes up their mind. So far a simple rule
has decided for everyone. Now Ben, and then everyone but Ana, decide in a different way.

<!-- guide: continues 08-time-and-weather -->

You start from the `corner-shop` folder of [chapter 08](08-time-and-weather.md).

Whatever moves a person is a **Controller**: a player, or a program
([Controller](../CORE_CONCEPTS.md#14-controller)). When `run` drives a seat, the program it uses is a
**decision model**. It looks at what the person can do now, and picks one thing. It never changes who the
person is. The world's rules still check everything the person tries.

This build has two decision models:

- `rule`, the one used so far. It picks among what it can do by a seeded roll of the dice.
- `motivated`. It scores each choice by need and by the time of day, from a published study of how
  people spend their day. Then it picks among the best.

### Step 9.1 — Work on a copy, and run it once

You try several choices in this chapter. Work on a copy, so that your own Corner Shop stays as it is.

**Do this.** Make a folder `who-decides`, and copy your `corner-shop` folder into it. On macOS and Linux:

```sh
mkdir who-decides
cp -R corner-shop who-decides/
```

On Windows, in PowerShell:

```powershell
mkdir who-decides
Copy-Item -Recurse corner-shop who-decides
```

Then run the copy for three days, and ask who decides for whom:

```console
$ mineworld run who-decides/corner-shop --headless --seed 1 --days 3
...
activity   days 1-3  ana move 155 talk 81  ben move 159 talk 118  cai move 170 talk 99
...
$ mineworld decisions who-decides/corner-shop
Corner Shop (corner-shop): decisions [..]
  ana        rule; memory: default
  ben        rule; memory: default
  cai        rule; memory: default
```

**You should see** the same `activity` numbers as at the end of chapter 07, and `rule` for all three
seats. Ignore `memory: default` for now; it is for programs that keep their own memory.

### Step 9.2 — Tell the world it has a decision list

The **decision list** is a file of the World Pack that says which decision model drives each seat.
`world.yaml` names it under `decisions`.

**Do this.** Replace `who-decides/corner-shop/world.yaml` with this whole file. The new part is
`decisions` at the end:

```yaml file=who-decides/corner-shop/world.yaml
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

decisions: decisions.yaml
```

The key is in the reference: [`world.yaml`](../reference/world-pack.md#worldyaml).

```console exit=1
$ mineworld decisions who-decides/corner-shop
[mineworld] who-decides/corner-shop/world.yaml: decisions: names decisions.yaml, which does not exist
```

**You should see** the check ask for `decisions.yaml`.

### Step 9.3 — Let Ben decide by his needs

**Do this.** In `who-decides/corner-shop`, write `decisions.yaml`:

```yaml file=who-decides/corner-shop/decisions.yaml
assign:
  - seat: ben
    model: motivated
```

`assign` is a list. Each entry names one `seat` and the `model` that drives it. A seat that no entry
names keeps the `rule` model. Every key the file may have is in the reference:
[`decisions.yaml`](../reference/decisions.md#decisionsyaml).

```console
$ mineworld decisions who-decides/corner-shop
Corner Shop (corner-shop): decisions [..]
  ana        rule; memory: default
  ben        motivated; memory: default
  cai        rule; memory: default
```

**You should see** `motivated` for Ben, and `rule` for Ana and Cai.

**If you see** `is not a model this build installs`, the model's name is misspelt. The message lists
the names this build has.

### Step 9.4 — Run it again and compare

**Do this.** Run the same three days with the same seed:

```console
$ mineworld run who-decides/corner-shop --headless --seed 1 --days 3
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 3; pace 900s; in memory; decisions [..] (2 rule, 1 motivated)
...
activity   days 1-3  ana move 162 talk 74  ben move 9 talk 18  cai move 165 talk 96
...
```

**You should see** `(2 rule, 1 motivated)` at the end of the first line, and these numbers. Side by
side with step 9.1:

| | Step 9.1 | Step 9.4 |
| --- | --- | --- |
| Ben's `move` | 159 | 9 |
| Ben's `talk` | 118 | 18 |

Ben now stays where his needs keep him, instead of wandering. He also talks much less: with its
default settings, the `motivated` model is a quiet one. Ana and Cai change a little too, because Ben is
around them less.

### Step 9.5 — Everyone but the shopkeeper

An entry can also name a **class** instead of a seat. Ana is in the class `shopkeeper` since chapter 07.

**Do this.** Replace `who-decides/corner-shop/decisions.yaml` with:

```yaml file=who-decides/corner-shop/decisions.yaml
default: motivated
assign:
  - class: shopkeeper
    model: rule
```

- `default` is the model for every seat that no entry names.
- The entry for the class `shopkeeper` keeps Ana on `rule`.

When entries disagree, the reference says which one wins: a seat's own entry, then its class's, then
`default`. See [`decisions.yaml`](../reference/decisions.md#decisionsyaml).

```console
$ mineworld decisions who-decides/corner-shop
Corner Shop (corner-shop): decisions [..]
  ana        rule; memory: default
  ben        motivated; memory: default
  cai        motivated; memory: default
$ mineworld run who-decides/corner-shop --headless --seed 1 --days 3
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 3; pace 900s; in memory; decisions [..] (1 rule, 2 motivated)
...
activity   days 1-3  ana move 203 talk 37  ben move 15 talk 7  cai move 10 talk 6
...
```

**You should see** Ben and Cai on `motivated`, Ana on `rule`, and these numbers. Ana still wanders like
before, but the other two rarely talk to her now, so she talks less too.

The `motivated` model has settings of its own, such as how many hours without company make a person
lonely. They go under `models` in the same file; each one, with its bounds, is in the reference:
[`models.motivated.parameters`](../reference/decisions.md#modelsmotivatedparameters).

A third kind of decision model, a person driven by a language model, is coming in a later update of
this guide.

## Why it works this way

Who decides for a person is not part of the person. It is chosen per seat, outside the world's state,
so the same world can run with any mix of players and programs. Every decision model can only pick
among what the world offers, and the server checks every choice. The rules are in
[`MODULE_SPEC.md`](../MODULE_SPEC.md#51-decision-models).

Next: [10 Forgetting](10-forgetting.md).

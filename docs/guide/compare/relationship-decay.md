# Relationship decay

On this page liking fades between meetings. You run Corner Shop for thirty days before and after, with
the same seed, and compare how often a tie between two people changes level.

The System Pack `relationships` keeps how well each person knows each other person (familiarity), and
how much they like them (regard). Together they give the level of the tie: `acquaintance`, `friendly`,
`friend` or `close`. Each time a tie crosses a level, the world records `relationship-changed`.
[Chapter 10](../10-forgetting.md) explains it step by step.

### Step 1.1 — Start from the finished Corner Shop

**Do this.** In your worlds folder, make a folder `compare-decay`. Copy the finished Corner Shop of
chapter 05 into it, from the folder you cloned in chapter 01. On macOS and Linux:

<!-- guide: equals compare-decay/corner-shop docs/guide/examples/corner-shop -->

```sh
mkdir compare-decay
cp -R "$HOME/MineWorld/docs/guide/examples/corner-shop" compare-decay/
```

On Windows, in PowerShell:

<!-- guide: equals compare-decay/corner-shop docs/guide/examples/corner-shop -->

```powershell
mkdir compare-decay
Copy-Item -Recurse "$HOME\MineWorld\docs\guide\examples\corner-shop" compare-decay
```

Then check the copy:

```console
$ mineworld validate compare-decay/corner-shop
...
compare-decay/corner-shop is a valid World Pack.
```

**You should see** that the copy is a valid world.

### Step 1.2 — Let people do things together

Regard grows when people invite each other to do something together and say yes. The System Pack
`group-activity` lets them.

**Do this.** Replace `compare-decay/corner-shop/world.yaml` with this whole file. The new part is
`group-activity`, at the end of `systems`:

```yaml file=compare-decay/corner-shop/world.yaml
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
  - group-activity

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

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

Then run thirty days:

```console
$ mineworld run compare-decay/corner-shop --headless --seed 1 --days 30
...
facts      relationship-changed 20
...
activity   days 1-30  ana move 1468 talk 738  ben move 1435 talk 950  cai move 1586 talk 871
...
```

**You should see** these numbers.

### Step 1.3 — Tell the world to configure relationships

**Do this.** Replace `compare-decay/corner-shop/world.yaml` with this whole file. The new part is
`relationships`, under `configure` at the end:

```yaml file=compare-decay/corner-shop/world.yaml
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
  - group-activity

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
  - relationships

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate compare-decay/corner-shop
[mineworld] configure: names 'relationships', but compare-decay/corner-shop/configure/relationships.yaml does not exist
```

**You should see** the check ask for `configure/relationships.yaml`.

### Step 1.4 — Change this: regard halves every three days apart

**Do this.** In `compare-decay/corner-shop/configure`, write `relationships.yaml`:

```yaml file=compare-decay/corner-shop/configure/relationships.yaml
parameters:
  - regard_decay: half-life
    regard_decay_days: 3
```

Three days is far faster than real friendships fade; it shows the effect within a month. Each setting is
in the reference:
[parameters of `configure/relationships.yaml`](../../reference/interaction-list.md#parameters-of-configurerelationshipsyaml).

Then run the same thirty days with the same seed, and keep them in a save:

```console
$ mineworld run compare-decay/corner-shop --headless --seed 1 --days 30 --save compare-decay/saves/after
...
facts      relationship-changed 38
...
activity   days 1-30  ana move 1468 talk 738  ben move 1435 talk 950  cai move 1586 talk 871
...
```

**You should see** these numbers.

### Step 1.5 — Find a tie that went down

**Do this.** Read what Ben perceived:

```console
$ mineworld perceived compare-decay/corner-shop --save compare-decay/saves/after --person ben --json
...
{"id":"[..]","at":[..],"event_type":"relationship-changed",[..]"from":"close"[..]"to":"friend"}}[..]
...
```

**You should see**, among many lines, a tie that went down from `close` to `friend`.

### The world changes like this

| | Step 1.2, nothing fades | Step 1.4, regard fades |
| --- | --- | --- |
| `relationship-changed` | 20 | 38 |
| `activity` | the same | the same |
| a tie going down | never | yes, as in step 1.5 |

People do exactly the same things in both runs. Without fading, a tie only goes up, and the three soon
stay `close`. With fading, liking drops between meetings, so a tie falls a level and rises again at the
next good meeting. Each of those crossings is one more `relationship-changed`.

Back to [all comparisons](README.md).

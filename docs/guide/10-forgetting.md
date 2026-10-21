# 10 Forgetting

In this chapter people in Corner Shop start doing things together, so they come to like each other. Then
you make that liking fade between meetings, and see a friendship cool down.

<!-- guide: continues 08-time-and-weather -->

You start from the `corner-shop` folder of [chapter 08](08-time-and-weather.md).

The System Pack `relationships` keeps, for each person, how well they know each other person
(**familiarity**) and how much they like them (**regard**). Together the two give the level of the tie:
`acquaintance`, `friendly`, `friend` or `close` ([Relationships](../CORE_CONCEPTS.md#44-relationships)).
When a tie crosses a level, the world records the fact `relationship-changed`.

By default nothing fades. A world can make both values fade with the time since two people last met.

### Step 10.1 — Work on a copy

**Do this.** Make a folder `forgetting`, and copy your `corner-shop` folder into it. On macOS and Linux:

```sh
mkdir forgetting
cp -R corner-shop forgetting/
```

On Windows, in PowerShell:

```powershell
mkdir forgetting
Copy-Item -Recurse corner-shop forgetting
```

Then check the copy:

```console
$ mineworld validate forgetting/corner-shop
...
forgetting/corner-shop is a valid World Pack.
```

**You should see** that the copy is a valid world.

### Step 10.2 — Let people do things together

Talking makes two people know each other better, but not like each other more. Regard grows when one
person invites another to do something together and they say yes. The System Pack `group-activity`
lets people do that.

**Do this.** Replace `forgetting/corner-shop/world.yaml` with this whole file. The new part is
`group-activity`, after `consumption`:

```yaml file=forgetting/corner-shop/world.yaml
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

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

Then run thirty days, and keep them in a save:

```console
$ mineworld run forgetting/corner-shop --headless --seed 1 --days 30 --save forgetting/saves/before
...
requests   accept-invitation accepted 260
...
requests   invite accepted 377
...
facts      relationship-changed 20
...
faults     0
...
```

**You should see** invitations sent and accepted, and 20 times a tie crossed a level. Without fading,
every tie only goes up, and the three soon stay `close`.

### Step 10.3 — Tell the world to configure relationships

**Do this.** Replace `forgetting/corner-shop/world.yaml` with this whole file. The new part is
`relationships`, at the end of `configure`:

```yaml file=forgetting/corner-shop/world.yaml
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
  - relationships

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate forgetting/corner-shop
[mineworld] configure: names 'relationships', but forgetting/corner-shop/configure/relationships.yaml does not exist
```

**You should see** the check ask for `configure/relationships.yaml`.

### Step 10.4 — Make liking fade

**Do this.** In `forgetting/corner-shop/configure`, write `relationships.yaml`:

```yaml file=forgetting/corner-shop/configure/relationships.yaml
parameters:
  - regard_decay: half-life
    regard_decay_days: 3
```

- `regard_decay` is the curve regard fades by. `half-life` halves it every time constant.
- `regard_decay_days` is that time constant: here, regard halves every three days without a meeting.

Three days is far faster than real friendships fade. It is chosen so that you can see the effect within
a month. The reference gives the defaults, the other curves, and the same settings for familiarity:
[parameters of `configure/relationships.yaml`](../reference/interaction-list.md#parameters-of-configurerelationshipsyaml).

```console
$ mineworld interactions forgetting/corner-shop --place home
...
section  relationships:
{
  "base": {
    "consequences": [],
    "default": "permit",
    "parameters": {
      "accepted_regard": 50,
      "activity_familiarity": 50,
      "activity_regard": 20,
      "declined_regard": -30,
      "familiarity_decay": "none",
      "familiarity_decay_days": 365,
      "familiarity_floor": 0,
      "known_at_most": 150,
      "regard_decay": "half-life",
      "regard_decay_days": 3,
      "regard_floor": 0,
      "spoke_familiarity": 10
    },
    "rules": [],
    "scoped": []
  },
  "place": "home"
}
...
```

**You should see** your two values, and every other value of `relationships` at its default.

### Step 10.5 — Run the same thirty days again

**Do this.**

```console
$ mineworld run forgetting/corner-shop --headless --seed 1 --days 30 --save forgetting/saves/after
...
facts      relationship-changed 32
...
activity   days 1-30  ana move 1552 talk 540  ben move 1492 talk 883  cai move 1591 talk 862
...
```

**You should see** 32 level crossings instead of 20. The `activity` numbers are the same as in step
10.2: people do exactly the same things. Only how they feel about it changed.

### Step 10.6 — See a friendship cool down

**Do this.** Read what Ben perceived in the second save:

```console
$ mineworld perceived forgetting/corner-shop --save forgetting/saves/after --person ben --json
...
{"id":"[..]","at":[..],"event_type":"relationship-changed",[..]"from":"friendly"[..]"to":"acquaintance"}}[..]
...
```

**You should see**, among many lines, a tie that went down from `friendly` to `acquaintance`. Liking faded
between two meetings, and the next meeting stated the new level. Without fading, no tie ever goes down.

Which program keeps a person's own memory, and what it forgets, is chosen in the decision list of
[chapter 09](09-who-decides.md). That choice matters only for a person driven by a program with its own
memory, such as a language model. It is coming in a later update of this guide.

## Why it works this way

The world's history never forgets: every fact stays in the save. What fades is a value the
`relationships` pack owns, read with the time since the last meeting, so no timer runs and nothing else
changes. The rules are in [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#forgetting-the-world-log-never-forgets-subjective-memory-may).

Next: [12 Packs](12-packs.md). Chapter 11 is coming in a later update of this guide.

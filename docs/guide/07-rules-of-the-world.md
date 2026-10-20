# 07 Rules of the world

In this chapter you change a rule of Corner Shop: in the shop, Ana keeps quiet and does not talk.
Customers may still talk to her. You also make people wait longer before a new conversation starts.
Then you run the world again and see the difference.

<!-- guide: continues 05-one-shop -->

You keep working in the `corner-shop` folder from [chapter 05](05-one-shop.md).

Who may do what, to whom and where is the world's **Interaction List**. Each System Pack has its own
part of it, with its own actions and numbers. A world changes them with files in a `configure`
folder. You never change the System Pack itself.

### Step 7.1 — Run it once, to compare later

**Do this.** Run three days, as at the end of chapter 05:

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 335
...
activity   days 1-3  ana move 145 talk 108  ben move 157 talk 120  cai move 158 talk 107
...
```

**You should see** these numbers. Ana spoke 108 times in three days. The same seed always gives the
same run, so you can compare the next run with this one.

### Step 7.2 — Tell the world it has configuration

**Do this.** Replace `corner-shop/world.yaml` with this whole file. The new part is `configure` at
the end:

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
  - item
  - inventory
  - economy
  - employment
  - consumption

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
  - classes
  - conversation

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

Each key under `configure` names a file in the `configure` folder: `classes` names
`configure/classes.yaml`, and `conversation` names `configure/conversation.yaml`.

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'classes', but corner-shop/configure/classes.yaml does not exist
```

**You should see** the check ask for `configure/classes.yaml`.

### Step 7.3 — Make Ana a shopkeeper, as a class

A rule does not name one person. It names a **class**: a group of entities with the same tag. Ana
carries the tag `shopkeeper` since chapter 02.

**Do this.** Make a folder `corner-shop/configure`. In it, write `classes.yaml`:

```yaml file=corner-shop/configure/classes.yaml
- class: shopkeeper
  of: person
  tag: shopkeeper
```

Every person with the tag `shopkeeper` is now in the class `shopkeeper`. The fields are in the
reference: [`configure/classes.yaml`](../reference/configure.md#configureclassesyaml).

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'conversation', but corner-shop/configure/conversation.yaml does not exist
```

**You should see** the check ask for `configure/conversation.yaml`.

### Step 7.4 — Write the rule

**Do this.** In `corner-shop/configure`, write `conversation.yaml`:

```yaml file=corner-shop/configure/conversation.yaml
parameters:
  - gap: 600
regions:
  shop:
    rules:
      - action: talk
        actor: shopkeeper
        effect: forbid
```

- `parameters` changes a number for everyone. `gap` is how many seconds of silence start a new
  conversation. The reference gives its default as 300, five minutes; this makes it ten.
- `regions` holds rules for one place. In `shop`, the action `talk` is forbidden when the one who
  talks, the `actor`, is a `shopkeeper`. Anyone else may still talk to Ana there.

The actions, roles and parameters this part may name are in the reference:
[`configure/conversation.yaml`](../reference/interaction-list.md#configureconversationyaml).

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is valid.

**If you see** `is not a key of a section`, a key in the file is misspelt. The reference lists the
keys a part may have: [the shape every section shares](../reference/interaction-list.md#the-shape-every-section-shares).

### Step 7.5 — Ask the world what the rules are in the shop

**Do this.**

```console
$ mineworld interactions corner-shop --place shop
interactions  Corner Shop (corner-shop)
section  conversation:
{
  "place": "shop",
  "region": {
    "consequences": [],
    "default": "permit",
    "parameters": {
      "forget_after_days": 0,
      "gap": 600,
      "range": 3000,
      "remembered": 32
    },
    "rules": [
      {
        "action": "talk",
        "effect": "forbid",
        "selectors": [
          "shopkeeper",
          "*",
          "*",
          "*"
        ]
      }
    ],
    "scoped": []
  }
}
section  employment: default (compiled)
section  relationships: default (compiled)
class    ana              shopkeeper
class    ben              person
class    bread            item
class    cai              person
class    corner-shop-co   organization
class    home             place
class    shop             place
class    tea              item
```

**You should see**, in the shop: `"gap": 600`, and one rule that forbids `talk` for a
`shopkeeper`. The `selectors` are the roles in order: actor, target, object and place; `*` means
anyone. At the end, each entity's class: Ana is a `shopkeeper`, Ben and Cai are just `person`.

### Step 7.6 — Run it again and compare

**Do this.** Run the same three days with the same seed:

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 298
...
activity   days 1-3  ana move 155 talk 81  ben move 159 talk 118  cai move 170 talk 99
...
```

**You should see** these numbers. Side by side with step 7.1:

| | Step 7.1 | Step 7.6 |
| --- | --- | --- |
| `talk accepted`, everyone | 335 | 298 |
| Ana's `talk` | 108 | 81 |

Ana talks less: in the shop she no longer talks at all, only at home. Ben and Cai may still talk to her
there. The others' numbers move a little too, because everyone's day now runs a little differently.
The world changed, and no System Pack did.

## Why it works this way

Rules belong to the System Pack that owns the action, and a world can only adjust what that pack
offers: its actions, its roles, its numbers. That keeps every world's rules readable in one place,
and the server checks them for every player and every program alike. The rules of the Interaction
List are in [`MODULE_SPEC.md`](../MODULE_SPEC.md#42-the-worlds-interaction-list).

Next: [08 Time and weather](08-time-and-weather.md).

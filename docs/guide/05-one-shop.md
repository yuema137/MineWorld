# 05 One shop

In this chapter the shop opens. You add two kinds of goods, a company that owns the shop, money,
and a job for Ana. Then people buy bread and tea, eat and drink, and Ana is paid for her work.

<!-- guide: continues 04-three-people -->

You keep working in the `corner-shop` folder from [chapter 04](04-three-people.md).

### Step 5.1 — Add goods, a company and five System Packs

Five System Packs make the shop work:

- `item` says which kinds of things exist, such as bread.
- `inventory` keeps what each person and company holds.
- `economy` keeps everyone's money, and runs shops. It is the only one that moves money.
- `employment` runs jobs: who works where and when, and the wage they are owed.
- `consumption` lets people eat food and drink drinks that they hold.

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
```

Two new lists:

- `items` lists kinds of things. `bread` is the kind "a loaf of bread", not one loaf. How many
  loaves someone has is a number.
- `organizations` lists companies and other groups. `corner-shop-co` is the company that owns the
  shop. Its key is not `shop`: every key in a world names exactly one thing.

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the item 'bread', but corner-shop/items/bread.yaml does not exist (an item's key names its file)
```

**You should see** the check ask for the bread's file.

### Step 5.2 — Write the bread

**Do this.** Make a folder `corner-shop/items`. In it, write `bread.yaml`:

```yaml file=corner-shop/items/bread.yaml
tags:
  - food
item:
  category: food
```

`item` belongs to the `item` System Pack. Its `category` says what sort of thing this is. People
eat things whose category is `food`. See the [`item` reference](../reference/sections.md#item).

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the item 'tea', but corner-shop/items/tea.yaml does not exist (an item's key names its file)
```

**You should see** the check ask for the tea.

### Step 5.3 — Write the tea

**Do this.** In `corner-shop/items`, write `tea.yaml`:

```yaml file=corner-shop/items/tea.yaml
tags:
  - drink
item:
  category: drink
```

People drink things whose category is `drink`.

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the organization 'corner-shop-co', but corner-shop/organizations/corner-shop-co.yaml does not exist (an organization's key names its file)
```

**You should see** the check ask for the company.

### Step 5.4 — Write the company and its shop

**Do this.** Make a folder `corner-shop/organizations`. In it, write `corner-shop-co.yaml`:

```yaml file=corner-shop/organizations/corner-shop-co.yaml
tags:
  - company
note: Owns and runs the corner shop.
holdings:
  bread: 10
  tea: 10
economy:
  wallet: 100000
  shop:
    at: shop
    prices:
      bread: 250
      tea: 150
```

- `holdings` is what the company has on its shelves at the start: ten loaves and ten teas.
- `economy` is its money and its shop. Money is written in cents, as whole numbers: `wallet: 100000`
  is 1,000.00 and a loaf at `250` costs 2.50. The shop is `at` the place `shop`.

The fields are in the reference: [`holdings`](../reference/sections.md#holdings) and
[`economy`](../reference/sections.md#economy).

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule, item, inventory, economy, employment, consumption
  places     home, shop
  people     ana, ben, cai
  items      bread, tea
  organizations corner-shop-co
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**You should see** the items and the company listed, and a valid world. Nobody has money yet, so
nobody can buy anything.

### Step 5.5 — Give Ana a job and some money

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
economy:
  wallet: 2000
job:
  employer: corner-shop-co
  workplace: shop
  from: "08:00"
  until: "18:00"
  wage: 1200
  produces:
    bread: 6
    tea: 6
```

- `economy` gives Ana 20.00 to start with.
- `job` says she works for `corner-shop-co` in the shop, from 08:00 to 18:00, for 12.00 an hour.
  Each full shift she bakes six loaves and makes six teas for the shop's shelves. A shorter shift
  makes less.

Her routine already has her in the shop from 07:30, so she is there when the shift starts. The
fields are in the [`job` reference](../reference/sections.md#job).

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is valid.

### Step 5.6 — Give Ben some money and a tea

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
holdings:
  tea: 1
economy:
  wallet: 5000
```

Ben starts with one tea and 50.00. A person can carry at most six things in all; the
[`holdings` reference](../reference/sections.md#holdings) says so.

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is valid.

### Step 5.7 — Give Cai some money and a loaf

**Do this.** Replace `corner-shop/people/cai.yaml` with:

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
holdings:
  bread: 1
economy:
  wallet: 5000
```

<!-- guide: equals corner-shop docs/guide/examples/corner-shop -->

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**You should see** that the world is valid. Your `corner-shop` folder is now exactly the finished
example in [`examples/corner-shop/`](examples/corner-shop/world.yaml). The test that runs this guide checks
that, file by file.

### Step 5.8 — Run three days of shopping

**Do this.**

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 3; pace 900s; in memory
day 1  revision -  facts [..]
day 2  revision -  facts [..]
day 3  revision -  facts [..]
...
requests   buy accepted [..]
requests   drink accepted [..]
requests   eat accepted [..]
...
facts      wage-due 3
...
faults     0
...
```

**You should see**:

- `buy accepted`: people bought bread and tea in the shop.
- `eat accepted` and `drink accepted`: they ate and drank what they held.
- `wage-due 3`: Ana worked three shifts, and each one earned her a wage. The company paid her.

The amounts are sized so that this happens in three days. Ben and Cai have enough money for many
loaves, the company can pay three days of wages, and Ana's shifts restock the shelves.

## Why it works this way

Each System Pack owns its own state and nothing else. `employment` says that a wage is due; only
`economy` moves the money. This is why you could add the shop without changing how walking or
talking works. The rules are in [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#131-single-writer-rule);
every section's fields are in [the sections reference](../reference/sections.md#sections).

Next: [06 Play it](06-play-it.md).

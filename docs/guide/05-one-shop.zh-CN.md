# 05 一家店

这一章小店开张。你会加上两种商品、一家拥有小店的公司、钱，以及 Ana 的工作。
然后大家会买面包和茶，吃的吃、喝的喝，Ana 干完活也能拿到工钱。

本页是英文原文 [05-one-shop.md](05-one-shop.md) 的中文镜像。两者不一致时，以英文为准。

<!-- guide: continues 04-three-people -->

接着用[第 04 章](04-three-people.zh-CN.md)的 `corner-shop` 文件夹。

### Step 5.1 — 加上商品、公司和五个系统包

让小店运转起来，要五个系统包：

- `item`：说明世界里有哪几种东西，比如面包。
- `inventory`：记着每个人、每家公司手里有什么。
- `economy`：管所有人的钱，也管商店。只有它能让钱换手。
- `employment`：管工作：谁在哪里、什么时间上班，该拿多少工钱。
- `consumption`：让人把手里的食物吃掉、饮料喝掉。

**照着做。** 用下面的内容替换 `corner-shop/world.yaml`：

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

两个新列表：

- `items` 列出东西的**种类**。`bread` 是“一条面包”这一类东西，不是某一条面包。某人有几条面包，是一个数字。
- `organizations` 列出公司和别的组织。`corner-shop-co` 是拥有小店的公司。它的键不是 `shop`：
  一个世界里，每个键只指一样东西。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the item 'bread', but corner-shop/items/bread.yaml does not exist (an item's key names its file)
```

**你应该看到**检查在要面包的文件。

### Step 5.2 — 写面包

**照着做。** 新建文件夹 `corner-shop/items`，在里面写 `bread.yaml`：

```yaml file=corner-shop/items/bread.yaml
tags:
  - food
item:
  category: food
```

`item` 属于 `item` 这个系统包。它的 `category` 说明这是哪一类东西。类别是 `food` 的东西，人会拿来吃。
见 [`item` 参考](../reference/sections.md#item)（英文）。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the item 'tea', but corner-shop/items/tea.yaml does not exist (an item's key names its file)
```

**你应该看到**检查在要茶的文件。

### Step 5.3 — 写茶

**照着做。** 在 `corner-shop/items` 里写 `tea.yaml`：

```yaml file=corner-shop/items/tea.yaml
tags:
  - drink
item:
  category: drink
```

类别是 `drink` 的东西，人会拿来喝。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the organization 'corner-shop-co', but corner-shop/organizations/corner-shop-co.yaml does not exist (an organization's key names its file)
```

**你应该看到**检查在要公司的文件。

### Step 5.4 — 写公司和它的店

**照着做。** 新建文件夹 `corner-shop/organizations`，在里面写 `corner-shop-co.yaml`：

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

- `holdings` 是公司一开始货架上有的东西：十条面包、十杯茶。
- `economy` 是它的钱和它的店。钱用“分”来写，都是整数：`wallet: 100000` 是 1000.00，
  一条面包 `250` 就是 2.50。这家店开在（`at`）`shop` 这个地方。

字段在参考手册里：[`holdings`](../reference/sections.md#holdings) 和
[`economy`](../reference/sections.md#economy)（英文）。

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

**你应该看到**商品和公司都列出来了，世界有效。不过现在谁都没钱，所以还没人能买东西。

### Step 5.5 — 给 Ana 一份工作和一点钱

**照着做。** 用下面的内容替换 `corner-shop/people/ana.yaml`：

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

- `economy` 给 Ana 一开始 20.00。
- `job` 说她给 `corner-shop-co` 干活，地点在小店，从 08:00 到 18:00，每小时 12.00。
  每干满一班，她就给店里的货架烤出六条面包、泡好六杯茶。干得少，就出得少。

她的日程本来就让她 07:30 起待在小店，所以上班时间一到，她已经在了。字段在
[`job` 参考](../reference/sections.md#job)（英文）。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界是有效的。

### Step 5.6 — 给 Ben 一点钱和一杯茶

**照着做。** 用下面的内容替换 `corner-shop/people/ben.yaml`：

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

Ben 一开始有一杯茶和 50.00。一个人身上最多只能带六样东西；[`holdings` 参考](../reference/sections.md#holdings)（英文）里写着。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界是有效的。

### Step 5.7 — 给 Cai 一点钱和一条面包

**照着做。** 用下面的内容替换 `corner-shop/people/cai.yaml`：

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

**你应该看到**世界是有效的。现在你的 `corner-shop` 文件夹和做好的示例
[`examples/corner-shop/`](examples/corner-shop/world.yaml) 一模一样。跑这份教程的测试会一个文件一个文件地核对。

### Step 5.8 — 跑三天，看大家买东西

**照着做。**

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

**你应该看到**：

- `buy accepted`：有人在小店买了面包和茶。
- `eat accepted` 和 `drink accepted`：他们把手里的东西吃了、喝了。
- `wage-due 3`：Ana 上了三班，每一班都挣到了工钱。公司付给了她。

这些数字是算好的，保证三天里能看到这些：Ben 和 Cai 的钱够买很多条面包，公司付得起三天工钱，
Ana 每一班都会给货架补货。

## 为什么是这样

每个系统包只管自己的那份状态，别的不碰。`employment` 只说“该发工钱了”，真正让钱换手的只有 `economy`。
所以加上小店时，你不用改走路或说话的任何东西。规则在
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#131-single-writer-rule)（英文）；每一部分的字段在
[sections 参考](../reference/sections.md#sections)（英文）。

下一章：[06 玩起来](06-play-it.zh-CN.md)。

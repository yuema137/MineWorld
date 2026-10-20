# 05 一家店

这一章小店开张。你会加上两种商品、一家拥有小店的公司、钱，还有 Ana 的工作。
然后大家会来买面包和茶，买了就吃、就喝，Ana 上完班也能拿到工资。

这一页翻译自英文原文 [05-one-shop.md](05-one-shop.md)。两边不一致时，以英文为准。

<!-- guide: continues 04-three-people -->

接着用[第 04 章](04-three-people.zh-CN.md)的 `corner-shop` 文件夹。

### Step 5.1 — 加上商品、公司和五个 System Pack

小店要转起来，需要五个 System Pack：

- `item`：写明世界里有哪几种东西，比如面包。
- `inventory`：记着每个人、每家公司手里有什么。
- `economy`：管所有人的钱，也管店。钱从一个人手里到另一个人手里，只有它能办。
- `employment`：管工作：谁几点在哪里上班，该拿多少工资。
- `consumption`：让人把手里的食物吃掉、饮料喝掉。

**照着做。** 用下面这份文件替换 `corner-shop/world.yaml`：

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

多了两个列表：

- `items` 列出东西的种类。`bread` 指“面包”这一类东西，不是某一条面包。
  一个人有几条面包，记成一个数字。
- `organizations` 列出公司和其他组织。`corner-shop-co` 是拥有小店的公司。
  它的 key 不能也叫 `shop`，因为一个世界里，一个 key 只能指一样东西。

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
  name: Bread
```

`item` 这一段归 `item` 这个 System Pack 管。`category` 写明这是哪一类东西。
类别是 `food` 的东西，人会拿来吃。见 [`item` 参考](../reference/sections.md#item)。

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
  name: Tea
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

- `holdings` 是公司一开始货架上的东西：十条面包，十杯茶。
- `economy` 写公司的钱和它的店。钱按“分”来写，而且是整数：`wallet: 100000` 就是 1000.00，
  一条面包标价 `250`，就是 2.50。`at: shop` 表示这家店开在 `shop` 这个地方。

字段在参考手册里：[`holdings`](../reference/sections.md#holdings) 和
[`economy`](../reference/sections.md#economy)。

```console exit=1
$ mineworld validate corner-shop
[mineworld] the world this pack describes cannot be composed: system 'economy' refused to reduce 'funded': [..]economy-currency-undeclared[..]
...
```

**你应该看到**检查拒绝了公司的钱，原因是 `economy-currency-undeclared`：世界里有了金额，
却还没说明它用的是什么钱。

### Step 5.5 — 告诉世界它要配置自己的钱

世界用一个配置文件说明自己用什么钱，这个文件属于 `economy` 这个 System Pack。

**照着做。** 用下面这份完整的文件替换 `corner-shop/world.yaml`。新加的是最后的 `configure`：

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
  - economy
```

`configure` 下面的每个键，都对应 `configure` 文件夹里的一个文件：`economy` 对应
`configure/economy.yaml`。

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'economy', but corner-shop/configure/economy.yaml does not exist
```

**你应该看到**检查在要 `configure/economy.yaml`。

### Step 5.6 — 说明用什么钱

**照着做。** 新建文件夹 `corner-shop/configure`，在里面写 `economy.yaml`：

```yaml file=corner-shop/configure/economy.yaml
currency:
  name: dollar
  symbol: "$"
  exponent: 2
  symbol_at: before
```

- `exponent: 2` 表示金额的最后两位是“分”：`250` 就是 2.50。
- `name` 和 `symbol` 是给人和玩家看的名字和符号，`symbol_at` 说符号写在金额前面还是后面。

字段在参考手册里：[`configure/economy.yaml`](../reference/configure.md#configureeconomyyaml)。

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

**你应该看到**商品和公司都列出来了，世界也写对了。不过现在谁手里都没钱，所以还没人能买东西。

### Step 5.7 — 给 Ana 一份工作和一点钱

**照着做。** 用下面这份文件替换 `corner-shop/people/ana.yaml`：

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

- `economy` 给 Ana 一开始 20.00 块钱。
- `job` 写她给 `corner-shop-co` 打工，在小店上班，从 08:00 到 18:00，每小时 12.00。
  每上满一个班，她给货架烤出六条面包、泡好六杯茶。班上得短，做出来的就少。

她的 routine 本来就让她 07:30 起待在小店，所以 08:00 开工时她已经在店里了。
字段在 [`job` 参考](../reference/sections.md#job)里。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界写对了。

### Step 5.8 — 给 Ben 一点钱和一杯茶

**照着做。** 用下面这份文件替换 `corner-shop/people/ben.yaml`：

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

Ben 一开始手里有一杯茶和 50.00 块钱。一个人身上最多带六样东西，[`holdings` 参考](../reference/sections.md#holdings)里写着这条。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界写对了。

### Step 5.9 — 给 Cai 一点钱和一条面包

**照着做。** 用下面这份文件替换 `corner-shop/people/cai.yaml`：

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

**你应该看到**世界写对了。现在你的 `corner-shop` 文件夹，和做好的示例
[`examples/corner-shop/`](examples/corner-shop/world.yaml) 一模一样。跑教程的测试会一份文件一份文件地比对。

### Step 5.10 — 跑三天，看大家买东西

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
- `wage-due 3`：Ana 上了三个班，每个班都挣了一份工资，公司已经付给她了。

这些数字是算好的，保证三天里能看到这些事：Ben 和 Cai 的钱够买很多条面包，公司付得起三天的工资，
Ana 每上一个班，货架就补一次货。

## 为什么是这样

每个 System Pack 只管自己那份数据，别人的不碰。`employment` 只负责说“该发工资了”，真正把钱转过去的是 `economy`。
所以加小店的时候，走路和说话的部分一点都不用改。规则在
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#131-single-writer-rule)；每一段能写的字段在
[sections 参考](../reference/sections.md#sections)。

下一章：[06 进去玩](06-play-it.zh-CN.md)。

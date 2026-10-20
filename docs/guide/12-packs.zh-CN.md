# 12 Pack

这一章你会看到你的 `mineworld` 带了哪些 pack，Corner Shop 又用了哪些。然后你会自己做一个很小的
**Entity Pack**，里面放一种商品，再让 Corner Shop 用上它。

这一页翻译自英文原文 [12-packs.md](12-packs.md)。两边不一致时，以英文为准。

<!-- guide: continues 08-time-and-weather -->

接着用[第 08 章](08-time-and-weather.zh-CN.md)的 `corner-shop` 文件夹。

pack 是一份能安装、能分享的东西。每个 System Pack 都是一个 pack。还有些 pack 里装的是数据，
比如 Entity Pack：一组现成的东西，比如各种商品，哪个世界都能拿来用（[六种 pack](../MODULE_SPEC.md#1-six-pack-types)）。

### Step 12.1 — 看看这个版本带了什么

**照着做。**

```console
$ mineworld packs list
system-pack        mineworld-presence          0.1.0   MIT  Yue Ma  bundled  system presence
system-pack        mineworld-movement          0.1.0   MIT  Yue Ma  bundled  system movement
system-pack        mineworld-conversation      0.1.0   MIT  Yue Ma  bundled  system conversation
...
[..] packs
```

**你应该看到**每个 pack 一行：类型、名字、版本、许可证、作者，最后是它从哪里来。
`bundled` 表示它已经编进了这个 `mineworld`。System Pack 那一行的最后一个词，比如 `presence`，
就是世界在 `systems` 里写的那个名字。

System Pack 也可以来自别人写的代码。这种 pack 那一行写的是 `third-party`，不是 `bundled`。
要加这样一个 pack，得带着它重新 build 一次 `mineworld`，做法见
[`MODULE_SPEC.md`](../MODULE_SPEC.md#31-installing-a-system-pack-in-mvp-0)。

### Step 12.2 — 看看 Corner Shop 用了哪些 pack

**照着做。**

```console
$ mineworld packs resolve corner-shop
Corner Shop (corner-shop)
  framework  0.1.0 (mineworld: "^0.1")
  system     presence → mineworld-presence 0.1.0 (bundled)
  system     movement → mineworld-movement 0.1.0 (bundled)
...
  system     weather → mineworld-weather 0.1.0 (bundled)
corner-shop resolves.
```

**你应该看到** Corner Shop 列出的每个 System Pack，以及是哪个 pack 提供的。

### Step 12.3 — 开始做一个 Entity Pack

**照着做。** 新建文件夹 `packs`，在里面再建一个 `corner-goods` 文件夹。在 `packs/corner-goods` 里写 `pack.yaml`：

```yaml file=packs/corner-goods/pack.yaml
id: corner-goods
type: entity-pack
version: 0.1.0
mineworld: "^0.1"
license: MIT
authors:
  - Your Name
```

`pack.yaml` 写明这个 pack 是什么：`id` 要和文件夹名一样，`type` 是类型，`version` 是版本，
`mineworld` 是它能配合哪些版本的 MineWorld，最后是许可证和作者。`authors` 里写上你自己的名字。
字段在参考手册里：[`pack.yaml`](../reference/pack-manifest.md#packyaml)。

```console exit=1
$ mineworld packs validate packs/corner-goods
[mineworld] packs/corner-goods: its pack.yaml states an entity-pack, which declares nothing — it needs items/<key>.yaml, one file per item kind, or a .yaml file in a data directory an installed System Pack owns
```

**你应该看到**一条拒绝信息：Entity Pack 里至少要有一样东西。

### Step 12.4 — 往里面放果酱

**照着做。** 在 `packs/corner-goods` 里新建 `items` 文件夹，在里面写 `jam.yaml`：

```yaml file=packs/corner-goods/items/jam.yaml
tags:
  - food
item:
  category: food
  name: Jam
```

写法和[第 05 章](05-one-shop.zh-CN.md#step-52--写面包)的面包一样。

```console
$ mineworld packs validate packs/corner-goods
corner-goods
  type        entity-pack
  version     0.1.0
  license     MIT
  authors     Your Name
  repository  —
  mineworld   ^0.1
  source      packs/corner-goods
  items       jam
packs/corner-goods is a valid entity-pack.
```

**你应该看到** `packs/corner-goods is a valid entity-pack.`，意思是这个 Entity Pack 写对了。

### Step 12.5 — 让 Corner Shop 用这个 pack

**照着做。** 用下面这整份文件替换 `corner-shop/world.yaml`。新加的是 `mineworld` 下面的 `requires`：

```yaml file=corner-shop/world.yaml
# Corner Shop: the world this guide builds.
world:
  id: corner-shop
  name: Corner Shop
  version: 0.1.0
  license: MIT

mineworld: "^0.1"

requires:
  corner-goods: "^0.1"

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
  - classes
  - conversation
  - calendar
  - weather
```

`requires` 列出世界要用、但没编进 `mineworld` 的每个 pack，以及能接受的版本。
`jam` 不用写进 `items`，pack 会把它带进来。

```console exit=1
$ mineworld validate corner-shop
[mineworld] corner-shop/world.yaml: requires: corner-goods is not in this build, and no pack directory was given (--packs, MINEWORLD_PACKS)
```

**你应该看到**一条拒绝信息：`mineworld` 不知道去哪里找 `corner-goods`。

### Step 12.6 — 告诉它 pack 放在哪里

**照着做。** 加上 `--packs packs`。`packs` 是一个文件夹，它下面的每个子文件夹都是一个 pack。

```console
$ mineworld validate corner-shop --packs packs
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule, item, inventory, economy, employment, consumption, calendar, weather
  places     home, shop
  people     ana, ben, cai
  items      bread, jam, tea
  organizations corner-shop-co
  seats      ana, ben, cai
  requires   corner-goods "^0.1" → 0.1.0 (packs/corner-goods)
...
corner-shop is a valid World Pack.
```

**你应该看到** `items` 里有了 `jam`，还多了一行 `requires`，写着 `corner-goods` 是在哪里找到的。
从现在起，对 Corner Shop 的每条命令都要加 `--packs packs`，比如这条：

```console
$ mineworld packs resolve corner-shop --packs packs
Corner Shop (corner-shop)
  framework  0.1.0 (mineworld: "^0.1")
  requires   corner-goods "^0.1" → entity-pack 0.1.0 (packs/corner-goods)
...
corner-shop resolves.
```

现在还没人卖果酱。想卖的话，就在 `corner-shop/organizations/corner-shop-co.yaml` 里，
把它加进店的 `prices` 和 `holdings`，做法和[第 05 章](05-one-shop.zh-CN.md#step-54--写公司和它的店)一样。

### Step 12.7 — 看看 MineWorld 自带的商品

MineWorld 的源代码里有一个叫 `modern-goods` 的 Entity Pack，里面是面包、牛奶、咖啡这类日常商品。
它在源代码的 `entities` 文件夹里：`MineWorld/entities/modern-goods`。

**照着做。** 在放世界的地方新建文件夹 `entities`，把这个 pack 复制进去。macOS 和 Linux 上这样写：

<!-- guide: equals entities/modern-goods entities/modern-goods -->

```sh
mkdir entities
cp -R "$HOME/MineWorld/entities/modern-goods" entities/
```

Windows 上，在 PowerShell 里这样写：

<!-- guide: equals entities/modern-goods entities/modern-goods -->

```powershell
mkdir entities
Copy-Item -Recurse "$HOME\MineWorld\entities\modern-goods" entities
```

然后问问它里面有什么：

```console
$ mineworld packs validate entities/modern-goods
modern-goods
  type        entity-pack
...
  items       apple, bread, cake, candle, cheese, coffee, juice, milk, notebook, soap, tea, umbrella
entities/modern-goods is a valid entity-pack.
```

**你应该看到** `items` 这一行列出了这个 pack 带来的每种商品。世界用它的方法和 `corner-goods` 一样：
在 `requires` 下面写 `modern-goods: "^0.1"`，每条命令都加上 `--packs entities`。
世界自己的 `items` 不用列出 pack 里的商品：那个列表只写世界 `items` 文件夹里的文件
（[`world.yaml`](../reference/world-pack.md#worldyaml)）。这样 `mineworld validate` 就会把 pack 的商品列进世界的商品里，
就像 Step 12.6 列出 `jam` 那样。

**让编辑器帮忙（可选）。** 教程里的每个字段，也都写成了 JSON Schema 文件，放在
[`reference/schema/`](../reference/schema/world.schema.json)。支持 YAML 的编辑器能读这些文件，
你打字时它会补全 key，也会把写错的地方标出来。教程里的文件不需要这些。

## 为什么是这样

世界写明它要用哪些 pack、要哪些版本。世界启动前，`mineworld` 会把每个 pack 找出来，逐个检查，
连 pack 的身份和许可证也一起查。规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#2-entity-pack) 和
[`PACKAGE_FORMAT.md`](../PACKAGE_FORMAT.md)。

下一章：[13 被拒绝的时候](13-when-something-is-refused.zh-CN.md)。

# 12 包

这一章你会看到你的 `mineworld` 里有哪些包，街角小店用了哪些包。然后你会自己做一个小小的**实体包**（Entity Pack），
里面放一种商品，再让街角小店用上它。

本页是英文原文 [12-packs.md](12-packs.md) 的中文镜像。两者不一致时，以英文为准。

<!-- guide: continues 08-time-and-weather -->

接着用[第 08 章](08-time-and-weather.zh-CN.md)的 `corner-shop` 文件夹。

**包**（pack）是可以安装、可以分享的一份东西。每个系统包都是一个包。还有一些包装的是数据，比如实体包：
一组现成的东西（比如各种商品），任何世界都能拿来用（[六种包](../MODULE_SPEC.md#1-six-pack-types)，英文）。

### Step 12.1 — 看看这个版本里有什么

**照着做。**

```console
$ mineworld packs list
system-pack        mineworld-presence          0.1.0   MIT  Yue Ma  bundled  system presence
system-pack        mineworld-movement          0.1.0   MIT  Yue Ma  bundled  system movement
system-pack        mineworld-conversation      0.1.0   MIT  Yue Ma  bundled  system conversation
...
[..] packs
```

**你应该看到**每个包一行：类型、名字、版本、许可证、作者，以及它从哪里来。
`bundled` 表示它已经编译进了这个 `mineworld`。系统包那一行的最后一个词，比如 `presence`，
就是世界在 `systems` 下面写的名字。

系统包也可以来自别人的代码。这样的包写的是 `third-party`（第三方），而不是 `bundled`。
加一个这样的包，要带着它重新编译 `mineworld`；怎么做见
[`MODULE_SPEC.md`](../MODULE_SPEC.md#31-installing-a-system-pack-in-mvp-0)（英文）。

### Step 12.2 — 看看街角小店用了哪些包

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

**你应该看到**街角小店列出的每个系统包，以及提供它的那个包。

### Step 12.3 — 开始做一个实体包

**照着做。** 新建文件夹 `packs`，在里面再建一个文件夹 `corner-goods`。在 `packs/corner-goods` 里写 `pack.yaml`：

```yaml file=packs/corner-goods/pack.yaml
id: corner-goods
type: entity-pack
version: 0.1.0
mineworld: "^0.1"
license: MIT
authors:
  - Your Name
```

包的 `pack.yaml` 说明它是什么：它的 `id`（和文件夹名一样）、`type`（类型）、`version`（版本）、
它能配合哪些版本的 MineWorld、它的许可证和作者。在 `authors` 里写上你自己的名字。
字段在参考手册里：[`pack.yaml`](../reference/pack-manifest.md#packyaml)（英文）。

```console exit=1
$ mineworld packs validate packs/corner-goods
[mineworld] packs/corner-goods: its pack.yaml states an entity-pack, which declares nothing — it needs items/<key>.yaml, one file per item kind
```

**你应该看到**一条拒绝信息：实体包里至少要有一样东西。

### Step 12.4 — 往里面放果酱

**照着做。** 在 `packs/corner-goods` 里新建文件夹 `items`，在里面写 `jam.yaml`：

```yaml file=packs/corner-goods/items/jam.yaml
tags:
  - food
item:
  category: food
```

它的写法和[第 05 章](05-one-shop.zh-CN.md#step-52--写面包)里的面包一样。

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

**你应该看到** `packs/corner-goods is a valid entity-pack.`（这是一个有效的实体包）。

### Step 12.5 — 让街角小店要求用这个包

**照着做。** 用下面这整个文件替换 `corner-shop/world.yaml`。新的部分是 `mineworld` 后面的 `requires`：

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

`requires` 列出世界需要、但没有编译进 `mineworld` 的每个包，以及它接受的版本。
世界不用在 `items` 下面列 `jam`：包会把它带进来。

```console exit=1
$ mineworld validate corner-shop
[mineworld] corner-shop/world.yaml: requires: corner-goods is not in this build, and no pack directory was given (--packs, MINEWORLD_PACKS)
```

**你应该看到**一条拒绝信息：`mineworld` 不知道去哪里找 `corner-goods`。

### Step 12.6 — 告诉它包在哪里

**照着做。** 加上 `--packs packs`：这个文件夹下面的每个子文件夹都是一个包。

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

**你应该看到**商品里有了 `jam`，还有一行 `requires`，说明 `corner-goods` 是在哪里找到的。
从现在起，对街角小店的每条命令都要加 `--packs packs`，比如这条：

```console
$ mineworld packs resolve corner-shop --packs packs
Corner Shop (corner-shop)
  framework  0.1.0 (mineworld: "^0.1")
  requires   corner-goods "^0.1" → entity-pack 0.1.0 (packs/corner-goods)
...
corner-shop resolves.
```

现在还没人卖果酱。想卖的话，就在 `corner-shop/organizations/corner-shop-co.yaml` 里，
把它加进店的 `prices` 和 `holdings`，就像[第 05 章](05-one-shop.zh-CN.md#step-54--写公司和它的店)那样。

**MineWorld 自带的商品。** MineWorld 的源代码里，`entities` 文件夹下有一个实体包叫 `modern-goods`，
里面是面包、牛奶、咖啡这样的日常商品。世界用它的方法一样：在 `requires` 下面写 `modern-goods: "^0.1"`，
再用 `--packs` 指向放它的那个文件夹。

**让编辑器帮你（可选）。** 这份教程里的每个字段，也都写成了 JSON Schema 文件，放在
[`reference/schema/`](../reference/schema/world.schema.json)。支持 YAML 的编辑器可以读这些文件，
在你打字时补全键名、标出错误。教程里的文件不需要它们。

## 为什么是这样

世界说明它需要什么、要哪些版本，`mineworld` 会在世界启动前把每个包找到并检查一遍。
包的身份和许可证也会被检查。规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#2-entity-pack) 和
[`PACKAGE_FORMAT.md`](../PACKAGE_FORMAT.md)（英文）。

下一章：[13 被拒绝的时候](13-when-something-is-refused.zh-CN.md)。

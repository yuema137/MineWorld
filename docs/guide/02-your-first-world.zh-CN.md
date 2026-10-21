# 02 第一个世界

这一章你从一个空文件夹开始写一个世界：一个房间，两个住在里面的人。
第 03 到 05 章会把它慢慢扩成 **Corner Shop**：两个房间，三个人，一家店。

这一页翻译自英文原文 [02-your-first-world.md](02-your-first-world.md)。两边不一致时，以英文为准。

一个世界就是一个 **World Pack**：一个文件夹，里面放几份纯文本文件，格式是 YAML。
每份文件都很短，用什么文本编辑器写都行。每份文件都按步骤里给的名字保存，
放在[第 01 章](01-get-mineworld.zh-CN.md)新建的那个文件夹里。要用到新文件夹时，自己建一个就好。

YAML 的写法很简单。`key: value` 设一个值。几行都以 `- ` 开头，就是一个列表。
缩进用空格，缩进进去的内容属于上面那一行。以 `#` 开头的行是写给人看的注释，MineWorld 不读它。

### Step 2.1 — 写世界的主文件

每个 World Pack 都有一个 `world.yaml`。它给世界起名，列出世界里的东西，还写明要用哪些 **System Pack**。
一个 System Pack 是一个模块，管一类事情，比如走路、说话（[System](../CORE_CONCEPTS.md#13-system)）。

**照着做。** 新建一个叫 `corner-shop` 的文件夹，在里面写 `world.yaml`：

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

places:
  - home

population:
  - ana
  - ben

seats:
  - ana
  - ben

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

每一段是什么意思：

- `world` 写世界的 `id` 和 `name`。`id` 必须和文件夹名一样。
- `mineworld: "^0.1"` 写明这个世界是给哪些版本的 MineWorld 用的。
- `systems` 列出要用的 System Pack，顺序有讲究。`presence` 管“人待在哪里”，`movement` 管走路，
  `conversation` 管说话。后两个都要用到 `presence`，所以 `presence` 排第一。
- `places` 和 `population` 列出世界里的地方和人。每一项是一个 **key**：一个小写的短名字。
  每个 key 对应一份同名文件，比如 `home` 对应 `places/home.yaml`。
- `seats` 列出玩家或程序能操控的人。这样的人叫 **seat**。
- `settlement` 写明这个世界是不是小镇。小镇要写明它的车站和地标；这个小世界不是小镇，所以写
  `kind: none`。`mineworld validate` 要求每个世界都写明。

`world.yaml` 的所有字段都在参考手册里：[`world.yaml`](../reference/world-pack.md#worldyaml)。

现在让 MineWorld 检查一下这个世界：

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the place 'home', but corner-shop/places/home.yaml does not exist (a place's key names its file)
```

**你应该看到**一条拒绝信息。这是正常的：`world.yaml` 里写了一个叫 `home` 的地方，可还没有它的文件。
有东西缺了或写错了，`validate` 都会点名说出来。接下来几步，它要什么文件，你就补什么文件。

### Step 2.2 — 写房间

Place 就是人能待的地方（[Place](../CORE_CONCEPTS.md#6-place)）。

**照着做。** 新建文件夹 `corner-shop/places`，在里面写 `home.yaml`：

```yaml file=corner-shop/places/home.yaml
tags:
  - home
note: Two flats behind one front door.
```

`tags` 是贴在这个地方上的标签。程序会读标签，比如 2D 或 3D client 会看标签决定怎么画这个地方。
`note` 是写给看文件的人的备注，MineWorld 不读它。地方文件能写的字段在参考手册里：[地方文件](../reference/world-pack.md#placeskeyyaml)。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'ana', but corner-shop/people/ana.yaml does not exist (a person's key names its file)
```

**你应该看到**地方这一关过了，现在检查要的是 `ana` 这个人的文件。

### Step 2.3 — 写第一个人

Person 就是住在世界里的人（[Person](../CORE_CONCEPTS.md#4-person)）。

**照着做。** 新建文件夹 `corner-shop/people`，在里面写 `ana.yaml`：

```yaml file=corner-shop/people/ana.yaml
tags:
  - shopkeeper
note: Runs the corner shop.
location:
  place: home
  position:
    x: 2000
    y: 3000
```

`location` 写 Ana 一开始在哪：在 `home` 里，站在 `position` 这个位置。
位置用毫米，从房间的一个角量起。`x: 2000` 是沿着墙走两米，`y: 3000` 是往屋里走三米。
人的文件能写的字段在参考手册里：[人的文件](../reference/world-pack.md#peoplekeyyaml)。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'ben', but corner-shop/people/ben.yaml does not exist (a person's key names its file)
```

**你应该看到** Ana 过了，现在检查要的是 `ben`。

### Step 2.4 — 写第二个人

**照着做。** 在 `corner-shop/people` 里写 `ben.yaml`：

```yaml file=corner-shop/people/ben.yaml
tags:
  - resident
note: Lives in the flat upstairs.
location:
  place: home
  position:
    x: 4000
    y: 3000
```

Ben 站的地方离 Ana 两米。说话声默认能传三米，所以两人不用走动就能聊上。
这三米归 `conversation` 这个 System Pack 管，是它的参数 `range`
（[conversation 的参数](../reference/interaction-list.md#parameters-of-configureconversationyaml)）。

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation
  places     home
  people     ana, ben
  seats      ana, ben
  settlement none

  1  home
  2  ana
  3  ben

2 genesis fact(s): the world's initial state, each one caused by the world coming into existence
corner-shop is a valid World Pack.
```

**你应该看到**最后一行 `corner-shop is a valid World Pack.`，意思是这个 World Pack 写对了。
上面几行列出世界里有什么。带编号的三行是世界里的 entity：世界一开始，每个地方、每个人都分到一个编号。
这里 `home` 是 1，`ana` 是 2，`ben` 是 3。

### Step 2.5 — 让它跑一天

**照着做。**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1
[mineworld] run Corner Shop (corner-shop): seats ana, ben; seed 1; to day 1; pace 900s; in memory
day 1  revision -  facts [..]
...
requests   talk accepted [..]
...
activity   days 1-1  ana move [..] talk [..]  ben move [..] talk [..]
faults     0
...
```

**你应该看到**第 1 天一行，然后是总结。`requests   talk accepted` 数的是：有人想说话，世界答应了，一共几次。
`activity` 那一行说明 Ana 和 Ben 都走动过，也都说过话。

两个人都由一条简单的规则操控：走两步，跟旁边的人说说话。没有真人在玩他们。

## 为什么是这样

World Pack 只写“世界里有什么”。走路、说话具体怎么算，由各个 System Pack 自己决定，所以世界只需要把它们列出来。
背后的规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#4-world-pack)。`validate` 做的检查，
和其他命令启动世界之前做的检查是同一套。

下一章：[03 两个房间](03-two-rooms.zh-CN.md)。

# 02 你的第一个世界

这一章你从一个空文件夹开始写一个世界：一个房间，两个住在那里的人。
第 03 到 05 章会把它扩成**街角小店**（Corner Shop）：两个房间、三个人、一家店。

本页是英文原文 [02-your-first-world.md](02-your-first-world.md) 的中文镜像。两者不一致时，以英文为准。

一个世界就是一个**世界包**（World Pack）：一个文件夹，里面是纯文本文件，格式叫 YAML。
每个文件都很短，用任何文本编辑器都能写。每个文件都要按步骤里给的名字保存，
放在[第 01 章](01-get-mineworld.zh-CN.md)建好的那个文件夹里。需要文件夹时，自己新建就行。

在 YAML 里，`key: value` 设定一个值。以 `- ` 开头的几行组成一个列表。
用空格缩进，表示谁属于谁。以 `#` 开头的行是写给人看的注释，MineWorld 会跳过它。

### Step 2.1 — 写世界的主文件

每个世界包都有一个 `world.yaml`。它给世界起名，列出世界里的一切，还说明要用哪些**系统包**（System Pack）。
系统包是一个模块，负责让一类事情发生，比如走路或说话（[System](../CORE_CONCEPTS.md#13-system)，英文）。

**照着做。** 新建一个叫 `corner-shop` 的文件夹。在里面写 `world.yaml`：

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
```

每一部分的意思：

- `world` 写世界的 `id` 和 `name`。`id` 必须和文件夹的名字一样。
- `mineworld: "^0.1"` 说明这个世界是为哪些版本的 MineWorld 写的。
- `systems` 是它用的系统包，有先后顺序：`presence`（待在某处）、`movement`（走路）、
  `conversation`（说话）。后两个要依赖第一个，所以第一个放在最前面。
- `places` 和 `population` 用**键**（key）列出世界里的地方和人。键是一个小写的短名字，每个键对应一个文件。
- `seats` 是玩家或程序可以操控的人，叫**座位**（seat）。

所有字段都列在参考手册里：[`world.yaml`](../reference/world-pack.md#worldyaml)（英文）。

现在让 MineWorld 检查一下这个世界：

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the place 'home', but corner-shop/places/home.yaml does not exist (a place's key names its file)
```

**你应该看到**一条拒绝信息。这是正常的：`world.yaml` 里列了一个叫 `home` 的地方，但还没有它的文件。
只要有东西缺了或写错了，`validate` 都会点名告诉你。接下来几步，你就照它要的把文件补上。

### Step 2.2 — 写房间

**地方**（Place）是人可以待的地方（[Place](../CORE_CONCEPTS.md#6-place)，英文）。

**照着做。** 新建文件夹 `corner-shop/places`，在里面写 `home.yaml`：

```yaml file=corner-shop/places/home.yaml
tags:
  - home
note: Two flats behind one front door.
```

`tags` 是标签。程序会读它；2D 或 3D 客户端可以根据标签决定怎么画这个地方。
`note` 是写给读文件的人看的，MineWorld 不读它。字段都在参考手册里：[地方文件](../reference/world-pack.md#placeskeyyaml)（英文）。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'ana', but corner-shop/people/ana.yaml does not exist (a person's key names its file)
```

**你应该看到**地方已经通过了，检查现在要的是 `ana` 这个人。

### Step 2.3 — 写第一个人

**人**（Person）是住在世界里的人（[Person](../CORE_CONCEPTS.md#4-person)，英文）。

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

`location` 说 Ana 一开始在哪：在 `home` 里的某个**位置**（position）。
位置用毫米表示，从房间的一个角量起：`x: 2000` 是横着两米，`y: 3000` 是往里三米。
字段都在参考手册里：[人的文件](../reference/world-pack.md#peoplekeyyaml)（英文）。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'ben', but corner-shop/people/ben.yaml does not exist (a person's key names its file)
```

**你应该看到** Ana 通过了，检查现在要的是 `ben`。

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

Ben 站的地方离 Ana 两米。说话的声音默认能传三米，所以他们不用走动就能聊天。
这个距离归“说话”这个系统包管，叫它的 `range`
（[conversation 的参数](../reference/interaction-list.md#parameters-of-configureconversationyaml)，英文）。

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation
  places     home
  people     ana, ben
  seats      ana, ben

  1  home
  2  ana
  3  ben

2 genesis fact(s): the world's initial state, each one caused by the world coming into existence
corner-shop is a valid World Pack.
```

**你应该看到**最后一行是 `corner-shop is a valid World Pack.`（这是一个有效的世界包）。
上面列出了世界里有什么。带编号的几行是世界里的**实体**（entity）：世界开始时，每个地方、每个人都会分到一个编号。

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

**你应该看到**第 1 天的一行，然后是总结。`requests   talk accepted` 数的是：有人想说话、世界同意了，一共多少次。
`activity` 那一行说明 Ana 和 Ben 都走动过，也都说过话。

每个人都由一条简单的规则操控：走两步，跟身边的人说说话。没有人在玩他们。

## 为什么是这样

世界包只描述“有什么”。它从不规定走路或说话怎么运作，那是各个系统包自己的事，所以世界里只要把它们列出来。
背后的规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#4-world-pack)（英文）。`validate` 做的检查，
和其他命令启动世界之前做的检查是同一套。

下一章：[03 两个房间](03-two-rooms.zh-CN.md)。

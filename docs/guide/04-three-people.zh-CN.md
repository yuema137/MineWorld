# 04 三个人

这一章你会加上第三个人：Cai。给每个人起名字、排一个每天的日程，大家也会慢慢认识彼此。
最后你会读到 Ana 第一天发生了什么。

本页是英文原文 [04-three-people.md](04-three-people.md) 的中文镜像。两者不一致时，以英文为准。

<!-- guide: continues 03-two-rooms -->

接着用[第 03 章](03-two-rooms.zh-CN.md)的 `corner-shop` 文件夹。

### Step 4.1 — 加上 Cai 和三个系统包

名字、日程、认识人，各由一个系统包负责：

- `relationships`：人们见面、聊天后会互相认识。
- `naming`：给每个人一个名字，别人看到的就是它。
- `schedule`：给每个人一份**日程**（routine）：每天什么时间打算待在哪里。

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

places:
  - home
  - shop

population:
  - ana
  - ben
  - cai

seats:
  - ana
  - ben
  - cai
```

三个新的系统包放在原来三个的后面。它们彼此不依赖。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'cai', but corner-shop/people/cai.yaml does not exist (a person's key names its file)
```

**你应该看到**检查在要 Cai 的文件。

### Step 4.2 — 写 Cai，带名字和日程

**照着做。** 在 `corner-shop/people` 里写 `cai.yaml`：

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
```

两个新部分：

- `name` 是别人看到的名字。键 `cai` 仍然是文件名。
- `routine` 是 Cai 的一天，分成几段。从 16:00 起，Cai 打算在小店买东西；从 18:00 起，在家。
  16:00 之前，算最后一段，所以 Cai 一天从家里开始。时间要加引号，写成 `"16:00"`。

日程只是打算，不是瞬移：Cai 还得自己走过去。字段在参考手册里：
[`name`](../reference/sections.md#name) 和 [`routine`](../reference/sections.md#routine)（英文）。

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule
  places     home, shop
  people     ana, ben, cai
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**你应该看到**三个人、三个座位。

### Step 4.3 — 给 Ana 名字和日程

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
```

Ana 七点半开始在小店上班，六点半回家。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界是有效的。

### Step 4.4 — 给 Ben 名字和日程

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
```

Ben 上午九点到十一点去买东西。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界是有效的。

### Step 4.5 — 跑一天，并且存下来

到现在为止，每次运行结束后就没了。想事后再看发生了什么，就要把这次运行存进**存档**（save）：
一个文件夹，保存世界和它发生过的一切。

**照着做。**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1 --save saves/day-1
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 1; pace 900s; created saves/day-1/world.sqlite
...
faults     0
...
```

**你应该看到**第一行里有 `created saves/day-1/world.sqlite`。存档是一个新文件夹 `saves/day-1`，就在 `corner-shop` 旁边。

### Step 4.6 — 读 Ana 的一天

一个人的**生平**（biography）是发生在他身上、对他的人生有意义的事，比如认识了谁。
它从存档里读出来（[biography](../CORE_CONCEPTS.md#52-objective-biography)，英文）。

**照着做。**

```console
$ mineworld biography corner-shop --save saves/day-1 --person ana
biography  ana "Ana Lee" (entity 3) in corner-shop: [..] entries from [..] facts; biographical: [..]
...
[..]day 1    01:30:00  #[..]became-acquainted        at home       with ben "Ben Okafor"
...
[..]day 1    02:00:00  #[..]became-acquainted        at home       with cai "Cai Wen"
...
[..]day 1    07:30:00  #[..]agenda-changed           at -          with -
[..]day 1    18:30:00  #[..]agenda-changed           at -          with -
```

**你应该看到**一列时间和事件。每一行是发生在 Ana 身上的一件事：

- 01:30 她在家里认识了 Ben，02:00 认识了 Cai。每次认识都会出现两行：一行是 Ana 认识了对方，
  一行是对方认识了 Ana。
- 07:30 和 18:30 的 `agenda-changed`：她日程里新的一段开始了。

她的文件里明明写着从小店开始，为什么夜里会在家？因为 07:30 之前算日程的最后一段，而那一段是“在家”。
所以她先走回了家，在那里遇见了另外两个人。

## 为什么是这样

每个系统包管着人的文件里属于自己的那一部分：`naming` 管 `name`，`schedule` 管 `routine`。
世界没列出某个系统包，就不能用它的那一部分，检查会点名拒绝。规则在
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads)（英文）；每一部分的字段在
[sections 参考](../reference/sections.md#sections)（英文）。

下一章：[05 一家店](05-one-shop.zh-CN.md)。

# 04 三个人

这一章你会加上第三个人 Cai。你会给每个人起名字，排好每天的安排，大家也会慢慢认识。
最后你会看到 Ana 第一天都发生了什么。

这一页翻译自英文原文 [04-three-people.md](04-three-people.md)。两边不一致时，以英文为准。

<!-- guide: continues 03-two-rooms -->

接着用[第 03 章](03-two-rooms.zh-CN.md)的 `corner-shop` 文件夹。

### Step 4.1 — 加上 Cai 和三个 System Pack

名字、每天的安排、认识人，各由一个 System Pack 来管：

- `relationships`：两个人碰上面、聊过天，就会互相认识。
- `naming`：给每个人一个名字，别人看到的就是这个名字。
- `schedule`：给每个人一份 **routine**，写着每天几点打算待在哪里。

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

# 不是小镇：小镇要写明车站大厅和地标（docs/MODULE_SPEC.md §4.3）。
settlement:
  kind: none
```

三个新的 System Pack 排在原来三个后面。这三个之间互不依赖。

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the person 'cai', but corner-shop/people/cai.yaml does not exist (a person's key names its file)
```

**你应该看到**检查在要 Cai 的文件。

### Step 4.2 — 写 Cai，带上名字和 routine

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

这里有两段新内容：

- `name` 是别人看到的名字。key 还是 `cai`，也就是文件名。
- `routine` 把 Cai 的一天分成几段。从 16:00 起，Cai 打算在小店买东西；从 18:00 起，打算在家。
  16:00 以前算最后那一段，所以 Cai 的一天从在家开始。时间要加引号，写成 `"16:00"`。

拿 Cai 的一天走一遍：00:00 在家，16:00 想去小店，18:00 想回家，第二天 16:00 再去小店。
routine 只是打算，不会把人直接挪过去，Cai 还得自己走过去。字段在参考手册里：
[`name`](../reference/sections.md#name) 和 [`routine`](../reference/sections.md#routine)。

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule
  places     home, shop
  people     ana, ben, cai
  seats      ana, ben, cai
  settlement none
...
corner-shop is a valid World Pack.
```

**你应该看到**三个人、三个 seat。

### Step 4.3 — 给 Ana 名字和 routine

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
```

Ana 早上七点半到小店上班，晚上六点半回家。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界写对了。

### Step 4.4 — 给 Ben 名字和 routine

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
```

Ben 上午九点到十一点去买东西。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界写对了。

### Step 4.5 — 跑一天，并且存下来

到现在为止，每次跑完，发生的事就都没了。想过后再看，就要把这次运行存进一个 **save**：
一个文件夹，存着世界和它发生过的所有事。

**照着做。**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1 --save saves/day-1
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 1; pace 900s; created saves/day-1/world.sqlite
...
faults     0
...
```

**你应该看到**第一行里有 `created saves/day-1/world.sqlite`。这个 save 是一个新文件夹 `saves/day-1`，就在 `corner-shop` 旁边。

### Step 4.6 — 看 Ana 这一天

一个人的 **biography**，是他身上发生过的、对他这一生有意义的事，比如认识了谁。
它是从 save 里读出来的（[biography](../CORE_CONCEPTS.md#52-objective-biography)）。

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

**你应该看到**一列时间和事件。每一行是 Ana 身上发生的一件事：

- 01:30 她在家里认识了 Ben，02:00 认识了 Cai。每次认识都有两行：一行是 Ana 认识了对方，
  另一行是对方认识了 Ana。
- 07:30 和 18:30 的 `agenda-changed`：她 routine 里新的一段开始了。

她的文件里写着一开始在小店，那她夜里怎么会在家？因为 07:30 以前算 routine 的最后一段，那一段是“在家”。
所以她一开始先走回了家，在家里碰上了另外两个人。

## 为什么是这样

人的文件里，每一段归一个 System Pack 管：`name` 归 `naming`，`routine` 归 `schedule`。
世界没有列出某个 System Pack，就不能用它那一段，检查会点名拒绝。规则在
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads)；每一段能写的字段在
[sections 参考](../reference/sections.md#sections)。

下一章：[05 一家店](05-one-shop.zh-CN.md)。

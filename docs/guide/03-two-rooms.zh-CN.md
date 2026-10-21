# 03 两个房间

这一章你会加上第二个房间，也就是小店，再在小店和家之间开一扇门。然后让 Ana 一早就待在小店里，大家会在两个房间之间来回走。

这一页翻译自英文原文 [03-two-rooms.md](03-two-rooms.md)。两边不一致时，以英文为准。

<!-- guide: continues 02-your-first-world -->

接着用[第 02 章](02-your-first-world.zh-CN.md)的 `corner-shop` 文件夹。

### Step 3.1 — 把小店写进世界

**照着做。** 用下面这整份文件替换 `corner-shop/world.yaml`。只改了一处：`places` 下面多了 `shop`。

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
  - shop

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

```console exit=1
$ mineworld validate corner-shop
[mineworld] world.yaml lists the place 'shop', but corner-shop/places/shop.yaml does not exist (a place's key names its file)
```

**你应该看到**检查在要小店的文件。

### Step 3.2 — 写小店

**照着做。** 在 `corner-shop/places` 里写 `shop.yaml`：

```yaml file=corner-shop/places/shop.yaml
tags:
  - shop
note: A small shop next door, with a counter at the back.
```

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation
  places     home, shop
  people     ana, ben
  seats      ana, ben
  settlement none
...
corner-shop is a valid World Pack.
```

**你应该看到** `places     home, shop`。世界已经写对了，但还没人进得了小店，因为没有门。

### Step 3.3 — 在两个房间之间开一扇门

两个地方之间的门，MineWorld 叫 **passage**。只要在其中一个地方的文件里写一次，两边都能走。

**照着做。** 用下面这份文件替换 `corner-shop/places/home.yaml`：

```yaml file=corner-shop/places/home.yaml
tags:
  - home
note: Two flats behind one front door.
passages:
  - to: shop
    here:
      x: 4000
      y: 0
    there:
      x: 2000
      y: 0
```

`to: shop` 写门通到哪里。`here` 是这扇门在家里的位置：沿着前墙（`y: 0`）走四米。
`there` 是同一扇门在小店里的位置：沿着小店前墙走两米。单位都是毫米。
passage 能写的字段在参考手册里：[`AuthoredPassage`](../reference/world-pack.md#authoredpassage)。

```console
$ mineworld validate corner-shop
...
3 genesis fact(s): the world's initial state, each one caused by the world coming into existence
corner-shop is a valid World Pack.
```

**你应该看到** `3 genesis fact(s)`，上一章是 2。genesis fact 是世界一开始就成立的事。多出来的这一条是：门是开着的。

### Step 3.4 — 让 Ana 一早就在小店

**照着做。** 用下面这份文件替换 `corner-shop/people/ana.yaml`：

```yaml file=corner-shop/people/ana.yaml
tags:
  - shopkeeper
note: Runs the corner shop.
location:
  place: shop
  position:
    x: 2000
    y: 4000
```

现在 Ana 一开始站在柜台后面：沿着小店走两米，从门往里走四米。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界仍然写对了。

**如果你看到** `puts 'ana' in the place 'shp', which this pack does not declare`，就是地方的 key 拼错了。
见[被拒绝的时候](13-when-something-is-refused.zh-CN.md#step-134--人在一个不存在的地方)。

### Step 3.5 — 跑一下，看大家走动

**照着做。**

```console
$ mineworld run corner-shop --headless --seed 1 --days 1
[mineworld] run Corner Shop (corner-shop): seats ana, ben; seed 1; to day 1; pace 900s; in memory
...
requests   walk-to accepted [..]
...
facts      person-entered-place [..]
...
faults     0
...
```

**你应该看到**总结里有 `walk-to accepted` 和 `person-entered-place`。`walk-to` 是“我想走到某处”这个请求。
`person-entered-place` 数的是：有人穿过门、进了另一个房间，一共几次。

## 为什么是这样

在 MineWorld 里，一个地方首先是世界认得的一个名字，比如 `shop`。地方里面的坐标，只在走路和画图时用到。
所以门写成“通到小店”，再在门的两头各写一个坐标。规则在 [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#6-place) 和
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads)。

下一章：[04 三个人](04-three-people.zh-CN.md)。

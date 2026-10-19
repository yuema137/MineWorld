# 09 谁来做决定

这一章你来选 Corner Shop 里每个人用什么方式拿主意。到现在为止，大家都由一条简单的规则替他们决定。
现在先让 Ben 换一种方式，再让除了 Ana 以外的人都换。

这一页翻译自英文原文 [09-who-decides.md](09-who-decides.md)。两边不一致时，以英文为准。

<!-- guide: continues 08-time-and-weather -->

从[第 08 章](08-time-and-weather.zh-CN.md)的 `corner-shop` 文件夹开始。

让一个人动起来的，叫 **Controller**。它可以是玩家，也可以是一个程序
（[Controller](../CORE_CONCEPTS.md#14-controller)）。`run` 替一个 seat 做决定时用的程序，叫
**decision model**。它看这个人眼下能做哪些事，从里面挑一件。它不会改变这个人是谁。
这个人想做的每件事，世界的规则照样会检查。

这个版本有两个 decision model：

- `rule`，就是到现在一直在用的那个。它在能做的事里掷骰子挑一件，骰子由 seed 决定。
- `motivated`。它按这个人的需要和一天里的时间，给每个选择打分，然后在分数最高的几个里挑。
  它的作息来自一份公开的调查，调查记的是人们一天的时间怎么花。

### Step 9.1 — 在副本上做，先跑一次

这一章要试好几种选法。所以在副本上做，你自己的 Corner Shop 保持原样。

**照着做。** 新建文件夹 `who-decides`，把你的 `corner-shop` 文件夹复制进去。macOS 和 Linux 上这样写：

```sh
mkdir who-decides
cp -R corner-shop who-decides/
```

Windows 上，在 PowerShell 里这样写：

```powershell
mkdir who-decides
Copy-Item -Recurse corner-shop who-decides
```

然后把副本跑三天，再问问每个人由谁来做决定：

```console
$ mineworld run who-decides/corner-shop --headless --seed 1 --days 3
...
activity   days 1-3  ana move 155 talk 81  ben move 159 talk 118  cai move 170 talk 99
...
$ mineworld decisions who-decides/corner-shop
Corner Shop (corner-shop): decisions [..]
  ana        rule; memory: default
  ben        rule; memory: default
  cai        rule; memory: default
```

**你应该看到** `activity` 这一行的数字和第 07 章最后一步一样，三个 seat 都是 `rule`。
`memory: default` 先不用管，它是给自己带记忆的程序用的。

### Step 9.2 — 告诉世界它有一份 decision list

**decision list** 是 World Pack 里的一份文件，写着每个 seat 由哪个 decision model 来驱动。
`world.yaml` 在 `decisions` 下面写出它的文件名。

**照着做。** 用下面这整份文件替换 `who-decides/corner-shop/world.yaml`。新加的是最后的 `decisions`：

```yaml file=who-decides/corner-shop/world.yaml
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
  - economy
  - classes
  - conversation
  - calendar
  - weather

decisions: decisions.yaml
```

这个 key 在参考手册里：[`world.yaml`](../reference/world-pack.md#worldyaml)。

```console exit=1
$ mineworld decisions who-decides/corner-shop
[mineworld] who-decides/corner-shop/world.yaml: decisions: names decisions.yaml, which does not exist
```

**你应该看到**检查在要 `decisions.yaml`。

### Step 9.3 — 让 Ben 按自己的需要拿主意

**照着做。** 在 `who-decides/corner-shop` 里写 `decisions.yaml`：

```yaml file=who-decides/corner-shop/decisions.yaml
assign:
  - seat: ben
    model: motivated
```

`assign` 是一个列表。每一条写一个 `seat`，再写驱动它的 `model`。哪一条都没写到的 seat，继续用 `rule`。
这份文件能写的每个 key 都在参考手册里：[`decisions.yaml`](../reference/decisions.md#decisionsyaml)。

```console
$ mineworld decisions who-decides/corner-shop
Corner Shop (corner-shop): decisions [..]
  ana        rule; memory: default
  ben        motivated; memory: default
  cai        rule; memory: default
```

**你应该看到** Ben 是 `motivated`，Ana 和 Cai 还是 `rule`。

**如果你看到** `is not a model this build installs`，就是 model 的名字拼错了。这条信息会列出这个版本有哪些名字。

### Step 9.4 — 再跑一次，比一比

**照着做。** 用同一个 seed 跑同样的三天：

```console
$ mineworld run who-decides/corner-shop --headless --seed 1 --days 3
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 3; pace 900s; in memory; decisions [..] (2 rule, 1 motivated)
...
activity   days 1-3  ana move 162 talk 74  ben move 9 talk 18  cai move 165 talk 96
...
```

**你应该看到**第一行最后是 `(2 rule, 1 motivated)`，还有这几个数字。和 Step 9.1 放在一起看：

| | Step 9.1 | Step 9.4 |
| --- | --- | --- |
| Ben 的 `move` | 159 | 9 |
| Ben 的 `talk` | 118 | 18 |

Ben 现在按需要待在一个地方，不再到处走。他说话也少多了：用默认设置时，`motivated` 是个安静的 model。
Ana 和 Cai 的数字也变了一点，因为 Ben 在他们身边的时间少了。

### Step 9.5 — 除了店主，大家都换

一条也可以写一个 **class**，不写 seat。Ana 从第 07 章起就在 `shopkeeper` 这个 class 里。

**照着做。** 用下面的内容替换 `who-decides/corner-shop/decisions.yaml`：

```yaml file=who-decides/corner-shop/decisions.yaml
default: motivated
assign:
  - class: shopkeeper
    model: rule
```

- `default` 是哪一条都没写到的 seat 用的 model。
- 写给 class `shopkeeper` 的那一条，让 Ana 继续用 `rule`。

几条写法冲突时听哪一条，参考手册里有写：先看 seat 自己那一条，再看它的 class 那一条，最后才是 `default`。
见 [`decisions.yaml`](../reference/decisions.md#decisionsyaml)。

```console
$ mineworld decisions who-decides/corner-shop
Corner Shop (corner-shop): decisions [..]
  ana        rule; memory: default
  ben        motivated; memory: default
  cai        motivated; memory: default
$ mineworld run who-decides/corner-shop --headless --seed 1 --days 3
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 3; pace 900s; in memory; decisions [..] (1 rule, 2 motivated)
...
activity   days 1-3  ana move 203 talk 37  ben move 15 talk 7  cai move 10 talk 6
...
```

**你应该看到** Ben 和 Cai 用 `motivated`，Ana 用 `rule`，还有这几个数字。Ana 还像以前一样到处走。
可另外两个人很少再跟她说话，所以她说话也少了。

`motivated` 有自己的设置，比如一个人没人陪多少小时会觉得孤单。这些设置写在同一份文件的 `models` 下面。
每一项和它的范围都在参考手册里：
[`models.motivated.parameters`](../reference/decisions.md#modelsmotivatedparameters)。

还有第三种 decision model：让语言模型来驱动一个人。以后的教程更新会补上。

## 为什么是这样

谁替一个人做决定，不算这个人的一部分。它是按 seat 选的，不存在世界的状态里。
所以同一个世界可以用玩家和程序的任意组合来跑。不管哪种 decision model，都只能在世界给出的选择里挑，
server 会检查每一个选择。规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#51-decision-models)。

下一章：[10 遗忘](10-forgetting.zh-CN.md)。

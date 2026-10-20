# 10 遗忘

这一章 Corner Shop 里的人开始一起做事，慢慢喜欢上对方。然后你让这份好感在两次见面之间慢慢变淡，
看一段友情怎么冷下来。

这一页翻译自英文原文 [10-forgetting.md](10-forgetting.md)。两边不一致时，以英文为准。

<!-- guide: continues 08-time-and-weather -->

从[第 08 章](08-time-and-weather.zh-CN.md)的 `corner-shop` 文件夹开始。

System Pack `relationships` 给每个人记着两个数：familiarity 记他和另一个人有多熟，regard 记他有多喜欢那个人。
两个数合起来，决定这段关系在哪一级：`acquaintance`、`friendly`、`friend` 或 `close`
（[Relationships](../CORE_CONCEPTS.md#44-relationships)）。关系每跨过一级，世界就记下一条 fact `relationship-changed`。

默认什么都不会变淡。一个世界可以让这两个数随着两人多久没见而变淡。

### Step 10.1 — 在副本上做

**照着做。** 新建文件夹 `forgetting`，把你的 `corner-shop` 文件夹复制进去。macOS 和 Linux 上这样写：

```sh
mkdir forgetting
cp -R corner-shop forgetting/
```

Windows 上，在 PowerShell 里这样写：

```powershell
mkdir forgetting
Copy-Item -Recurse corner-shop forgetting
```

然后检查一下副本：

```console
$ mineworld validate forgetting/corner-shop
...
forgetting/corner-shop is a valid World Pack.
```

**你应该看到**副本是一个写对了的世界。

### Step 10.2 — 让大家一起做事

说话会让两个人更熟，但不会让他们更喜欢对方。一个人邀请另一个人一起做点什么，对方答应了，regard 才会涨。
System Pack `group-activity` 让人能这样做。

**照着做。** 用下面这整份文件替换 `forgetting/corner-shop/world.yaml`。新加的是 `consumption` 后面的 `group-activity`：

```yaml file=forgetting/corner-shop/world.yaml
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
  - group-activity
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

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

然后跑三十天，存进一个 save：

```console
$ mineworld run forgetting/corner-shop --headless --seed 1 --days 30 --save forgetting/saves/before
...
requests   accept-invitation accepted 263
...
requests   invite accepted 374
...
facts      relationship-changed 18
...
faults     0
...
```

**你应该看到**有人发出邀请，有人答应，关系一共跨级 18 次。不会变淡的时候，关系只升不降，三个人很快就一直是 `close`。

### Step 10.3 — 告诉世界要配置 relationships

**照着做。** 用下面这整份文件替换 `forgetting/corner-shop/world.yaml`。新加的是 `configure` 最后的 `relationships`：

```yaml file=forgetting/corner-shop/world.yaml
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
  - group-activity
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
  - relationships

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate forgetting/corner-shop
[mineworld] configure: names 'relationships', but forgetting/corner-shop/configure/relationships.yaml does not exist
```

**你应该看到**检查在要 `configure/relationships.yaml`。

### Step 10.4 — 让好感变淡

**照着做。** 在 `forgetting/corner-shop/configure` 里写 `relationships.yaml`：

```yaml file=forgetting/corner-shop/configure/relationships.yaml
parameters:
  - regard_decay: half-life
    regard_decay_days: 3
```

- `regard_decay` 是 regard 按哪条曲线变淡。`half-life` 是每过一段固定的时间就减半。
- `regard_decay_days` 就是这段时间的天数。这里是两个人三天没见，regard 就减一半。

现实里的友情不会淡得这么快。选三天，是为了让你在一个月里就看得到效果。默认值、其他曲线，还有 familiarity
的同样几项设置，都在参考手册里：
[parameters of `configure/relationships.yaml`](../reference/interaction-list.md#parameters-of-configurerelationshipsyaml)。

```console
$ mineworld interactions forgetting/corner-shop --place home
...
section  relationships:
{
  "base": {
    "consequences": [],
    "default": "permit",
    "parameters": {
      "accepted_regard": 50,
      "activity_familiarity": 50,
      "activity_regard": 20,
      "declined_regard": -30,
      "familiarity_decay": "none",
      "familiarity_decay_days": 365,
      "familiarity_floor": 0,
      "known_at_most": 150,
      "regard_decay": "half-life",
      "regard_decay_days": 3,
      "regard_floor": 0,
      "spoke_familiarity": 10
    },
    "rules": [],
    "scoped": []
  },
  "place": "home"
}
...
```

**你应该看到**你写的两个值，`relationships` 的其他值都还是默认。

### Step 10.5 — 把同样的三十天再跑一次

**照着做。**

```console
$ mineworld run forgetting/corner-shop --headless --seed 1 --days 30 --save forgetting/saves/after
...
facts      relationship-changed 32
...
activity   days 1-30  ana move 1526 talk 597  ben move 1498 talk 878  cai move 1572 talk 864
...
```

**你应该看到**关系跨级 32 次，不是 18 次。`activity` 的数字和 Step 10.2 一样：大家做的事一模一样，
变的只是他们对彼此的感觉。

### Step 10.6 — 看一段友情冷下来

**照着做。** 读出 Ben 在第二个 save 里感知到的事：

```console
$ mineworld perceived forgetting/corner-shop --save forgetting/saves/after --person ben --json
...
{"id":"[..]","at":[..],"event_type":"relationship-changed",[..]"from":"close"[..]"to":"friend"}}[..]
...
```

**你应该看到**，在很多行里，有一段关系从 `close` 降到了 `friend`。两次见面之间好感变淡了，
下一次见面时，世界记下了新的一级。不会变淡的时候，没有哪段关系会往下降。

一个人自己的记忆由哪个程序来管、忘掉什么，是在[第 09 章](09-who-decides.zh-CN.md)的 decision list 里选的。
只有当一个人由自带记忆的程序驱动时，比如语言模型，这个选择才有用。以后的教程更新会补上。

## 为什么是这样

世界的历史从来不会忘：每一条 fact 都留在 save 里。变淡的是 `relationships` 这个 pack 自己管的一个数。
每次读它的时候，按两人多久没见来算，所以不用跑计时器，别的东西也都不变。
规则在 [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#forgetting-the-world-log-never-forgets-subjective-memory-may)。

下一章：[12 Pack](12-packs.zh-CN.md)。第 11 章会在以后的教程更新里补上。

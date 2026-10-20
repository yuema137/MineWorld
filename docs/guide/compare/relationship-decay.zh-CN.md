# 关系变淡

这一页让好感在两次见面之间变淡。你用同一个 seed，把 Corner Shop 在改动前后各跑三十天，
比一比两个人之间的关系有多常跨级。

这一页翻译自英文原文 [relationship-decay.md](relationship-decay.md)。两边不一致时，以英文为准。

System Pack `relationships` 给每个人记着两个数：familiarity 记他和另一个人有多熟，regard 记他有多喜欢那个人。
两个数合起来，决定这段关系在哪一级：`acquaintance`、`friendly`、`friend` 或 `close`。
关系每跨过一级，世界就记下一条 `relationship-changed`。[第 10 章](../10-forgetting.zh-CN.md)一步一步讲过。

### Step 1.1 — 从做完的 Corner Shop 开始

**照着做。** 在放世界的文件夹里新建文件夹 `compare-decay`。把第 05 章做完的 Corner Shop 复制进去，
它在你第 01 章 clone 下来的文件夹里。macOS 和 Linux 上这样写：

<!-- guide: equals compare-decay/corner-shop docs/guide/examples/corner-shop -->

```sh
mkdir compare-decay
cp -R "$HOME/MineWorld/docs/guide/examples/corner-shop" compare-decay/
```

Windows 上，在 PowerShell 里这样写：

<!-- guide: equals compare-decay/corner-shop docs/guide/examples/corner-shop -->

```powershell
mkdir compare-decay
Copy-Item -Recurse "$HOME\MineWorld\docs\guide\examples\corner-shop" compare-decay
```

然后检查一下副本：

```console
$ mineworld validate compare-decay/corner-shop
...
compare-decay/corner-shop is a valid World Pack.
```

**你应该看到**副本是一个写对了的世界。

### Step 1.2 — 让大家一起做事

有人邀请别人一起做点什么，对方答应了，regard 就会涨。System Pack `group-activity` 让人能这样做。

**照着做。** 用下面这整份文件替换 `compare-decay/corner-shop/world.yaml`。新加的是 `systems` 最后的 `group-activity`：

```yaml file=compare-decay/corner-shop/world.yaml
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

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

然后跑三十天：

```console
$ mineworld run compare-decay/corner-shop --headless --seed 1 --days 30
...
facts      relationship-changed 20
...
activity   days 1-30  ana move 1468 talk 738  ben move 1435 talk 950  cai move 1586 talk 871
...
```

**你应该看到**这几个数字。

### Step 1.3 — 告诉世界要配置 relationships

**照着做。** 用下面这整份文件替换 `compare-decay/corner-shop/world.yaml`。新加的是最后的 `configure`：

```yaml file=compare-decay/corner-shop/world.yaml
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
  - relationships

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate compare-decay/corner-shop
[mineworld] configure: names 'relationships', but compare-decay/corner-shop/configure/relationships.yaml does not exist
```

**你应该看到**检查在要 `configure/relationships.yaml`。

### Step 1.4 — 改这一处：两人分开三天，regard 就减一半

**照着做。** 新建文件夹 `compare-decay/corner-shop/configure`，在里面写 `relationships.yaml`：

```yaml file=compare-decay/corner-shop/configure/relationships.yaml
parameters:
  - regard_decay: half-life
    regard_decay_days: 3
```

现实里的友情不会淡得这么快。选三天，是为了在一个月里就看得到效果。每一项设置都在参考手册里：
[parameters of `configure/relationships.yaml`](../../reference/interaction-list.md#parameters-of-configurerelationshipsyaml)。

然后用同一个 seed 跑同样的三十天，存进一个 save：

```console
$ mineworld run compare-decay/corner-shop --headless --seed 1 --days 30 --save compare-decay/saves/after
...
facts      relationship-changed 38
...
activity   days 1-30  ana move 1468 talk 738  ben move 1435 talk 950  cai move 1586 talk 871
...
```

**你应该看到**这几个数字。

### Step 1.5 — 找一段往下降的关系

**照着做。** 读出 Ben 感知到的事：

```console
$ mineworld perceived compare-decay/corner-shop --save compare-decay/saves/after --person ben --json
...
{"id":"[..]","at":[..],"event_type":"relationship-changed",[..]"from":"close"[..]"to":"friend"}}[..]
...
```

**你应该看到**，在很多行里，有一段关系从 `close` 降到了 `friend`。

### 世界这样变了

| | Step 1.2，什么都不变淡 | Step 1.4，regard 会变淡 |
| --- | --- | --- |
| `relationship-changed` | 20 | 38 |
| `activity` | 一样 | 一样 |
| 有没有关系往下降 | 从来没有 | 有，就像 Step 1.5 里那样 |

两次跑的时候，大家做的事一模一样。不会变淡时，关系只升不降，三个人很快就一直是 `close`。
会变淡时，好感在两次见面之间往下掉，关系就降一级，下次好好见一面又升回去。
每跨一次级，就多一条 `relationship-changed`。

回到[所有对比](README.zh-CN.md)。

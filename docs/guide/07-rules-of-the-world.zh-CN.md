# 07 世界的规则

这一章你会改街角小店的一条规则：在小店里，Ana 不说话。顾客还是可以跟她说话。
你还会让大家多停一会儿，才算开始一段新的对话。然后再跑一次世界，看看有什么不同。

本页是英文原文 [07-rules-of-the-world.md](07-rules-of-the-world.md) 的中文镜像。两者不一致时，以英文为准。

<!-- guide: continues 05-one-shop -->

接着用[第 05 章](05-one-shop.zh-CN.md)的 `corner-shop` 文件夹。

谁可以对谁做什么、在哪里做，叫这个世界的**交互清单**（Interaction List）。
每个系统包在里面都有自己的一部分，有它自己的动作和数字。世界用 `configure` 文件夹里的文件来改它们。
你永远不需要改系统包本身。

### Step 7.1 — 先跑一次，留着对比

**照着做。** 跑三天，和第 05 章最后一样：

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 335
...
activity   days 1-3  ana move 145 talk 108  ben move 157 talk 120  cai move 158 talk 107
...
```

**你应该看到**这些数字。三天里 Ana 说了 108 次话。同一个种子，每次跑出的结果都一样，所以下一次可以拿来和这次对比。

### Step 7.2 — 告诉世界它有配置

**照着做。** 用下面这整个文件替换 `corner-shop/world.yaml`。新的部分是最后的 `configure`：

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
  - item
  - inventory
  - economy
  - employment
  - consumption

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
```

`configure` 下面的每个键，对应 `configure` 文件夹里的一个文件：`classes` 对应
`configure/classes.yaml`，`conversation` 对应 `configure/conversation.yaml`。

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'classes', but corner-shop/configure/classes.yaml does not exist
```

**你应该看到**检查在要 `configure/classes.yaml`。

### Step 7.3 — 把 Ana 归进“店主”这个类别

规则不会点名某一个人，它点的是一个**类别**（class）：带同一个标签的一组实体。
从第 02 章起，Ana 就带着 `shopkeeper`（店主）这个标签。

**照着做。** 新建文件夹 `corner-shop/configure`，在里面写 `classes.yaml`：

```yaml file=corner-shop/configure/classes.yaml
- class: shopkeeper
  of: person
  tag: shopkeeper
```

现在，每个带 `shopkeeper` 标签的人，都属于 `shopkeeper` 这个类别。字段在参考手册里：
[`configure/classes.yaml`](../reference/configure.md#configureclassesyaml)（英文）。

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'conversation', but corner-shop/configure/conversation.yaml does not exist
```

**你应该看到**检查在要 `configure/conversation.yaml`。

### Step 7.4 — 写规则

**照着做。** 在 `corner-shop/configure` 里写 `conversation.yaml`：

```yaml file=corner-shop/configure/conversation.yaml
parameters:
  - gap: 600
regions:
  shop:
    rules:
      - action: talk
        actor: shopkeeper
        effect: forbid
```

- `parameters` 给所有人改一个数字。`gap` 是安静多少秒以后，下一句话就算新的一段对话。
  参考手册里它的默认值是 300，也就是五分钟；这里改成十分钟。
- `regions` 放只在某个地方生效的规则。在 `shop` 里，如果说话的人（`actor`）是 `shopkeeper`，
  `talk` 这个动作就被禁止（`forbid`）。别人在那里还是可以跟 Ana 说话。

这一部分能写哪些动作、角色和参数，都在参考手册里：
[`configure/conversation.yaml`](../reference/interaction-list.md#configureconversationyaml)（英文）。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界是有效的。

**如果你看到** `is not a key of a section`，说明文件里有个键拼错了。参考手册列出了每一部分可以有哪些键：
[the shape every section shares](../reference/interaction-list.md#the-shape-every-section-shares)（英文）。

### Step 7.5 — 问问世界，小店里的规则是什么

**照着做。**

```console
$ mineworld interactions corner-shop --place shop
interactions  Corner Shop (corner-shop)
section  conversation:
{
  "place": "shop",
  "region": {
    "consequences": [],
    "default": "permit",
    "parameters": {
      "forget_after_days": 0,
      "gap": 600,
      "range": 3000,
      "remembered": 32
    },
    "rules": [
      {
        "action": "talk",
        "effect": "forbid",
        "selectors": [
          "shopkeeper",
          "*",
          "*",
          "*"
        ]
      }
    ],
    "scoped": []
  }
}
section  relationships: default (compiled)
class    ana              shopkeeper
class    ben              person
class    bread            item
class    cai              person
class    corner-shop-co   organization
class    home             place
class    shop             place
class    tea              item
```

**你应该看到**，在小店里：`"gap": 600`，还有一条规则禁止 `shopkeeper` 说话（`talk`）。
`selectors` 按顺序对应几个角色：说话的人、对象、物品、地点；`*` 表示谁都行。
最后几行是每个实体的类别：Ana 是 `shopkeeper`，Ben 和 Cai 只是普通的 `person`。

### Step 7.6 — 再跑一次，对比一下

**照着做。** 用同一个种子，再跑同样的三天：

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 298
...
activity   days 1-3  ana move 155 talk 81  ben move 159 talk 118  cai move 170 talk 99
...
```

**你应该看到**这些数字。和第 7.1 步放在一起看：

| | 第 7.1 步 | 第 7.6 步 |
| --- | --- | --- |
| 所有人的 `talk accepted` | 335 | 298 |
| Ana 的 `talk` | 108 | 81 |

Ana 话少了：在小店里她一句话都不说，只在家里说。Ben 和 Cai 在店里还是可以跟她说话。
其他人的数字也变了一点，因为每个人的一天都跟着变得有点不一样。世界变了，系统包一点没动。

## 为什么是这样

规则属于管这个动作的系统包，世界只能调整那个包开放出来的东西：它的动作、它的角色、它的数字。
这样，每个世界的规则都集中在一个地方，读起来清楚；服务器对每个玩家、每个程序都按同样的规则检查。
交互清单的规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#42-the-worlds-interaction-list)（英文）。

下一章：[08 时间和天气](08-time-and-weather.zh-CN.md)。

# 07 世界的规则

这一章你会改 Corner Shop 的一条规则：在小店里，Ana 不说话。顾客还是可以跟她说话。
你还会把“安静多久才算新的一段对话”调长一点。然后再跑一次世界，看看有什么不一样。

这一页翻译自英文原文 [07-rules-of-the-world.md](07-rules-of-the-world.md)。两边不一致时，以英文为准。

<!-- guide: continues 05-one-shop -->

接着用[第 05 章](05-one-shop.zh-CN.md)的 `corner-shop` 文件夹。

谁能对谁做什么、在哪里能做，写在这个世界的 **Interaction List** 里。每个 System Pack 在里面都有自己那一部分，
里面是它自己的动作和数字。世界用 `configure` 文件夹里的文件改这些东西，System Pack 本身一行都不用动。

### Step 7.1 — 先跑一次，留着对比

**照着做。** 跑三天，和第 05 章最后一步一样：

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 335
...
activity   days 1-3  ana move 145 talk 108  ben move 157 talk 120  cai move 158 talk 107
...
```

**你应该看到**这几个数字。三天里 Ana 说了 108 次话。seed 一样，每次跑的结果都一样，所以下一次可以拿来跟这次比。

### Step 7.2 — 告诉世界它有配置文件

**照着做。** 用下面这整份文件替换 `corner-shop/world.yaml`。新加的是最后的 `configure`：

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

# 不是小镇：小镇要写明车站大厅和地标（docs/MODULE_SPEC.md §4.3）。
settlement:
  kind: none
```

`configure` 下面每个 key 对应 `configure` 文件夹里的一份文件：`classes` 对应
`configure/classes.yaml`，`conversation` 对应 `configure/conversation.yaml`。

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'classes', but corner-shop/configure/classes.yaml does not exist
```

**你应该看到**检查在要 `configure/classes.yaml`。

### Step 7.3 — 用 class 把 Ana 归成店主

规则不会点某一个人的名，它点的是一个 **class**：带同一个 tag 的一组人或东西。
Ana 从第 02 章起就带着 `shopkeeper` 这个 tag。

**照着做。** 新建文件夹 `corner-shop/configure`，在里面写 `classes.yaml`：

```yaml file=corner-shop/configure/classes.yaml
- class: shopkeeper
  of: person
  tag: shopkeeper
```

这三行的意思是：凡是带 `shopkeeper` tag 的人，都属于 `shopkeeper` 这个 class。字段在参考手册里：
[`configure/classes.yaml`](../reference/configure.md#configureclassesyaml)。

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

- `parameters` 给所有人改一个数字。`gap` 是秒数：安静超过这么久，下一句话就算新的一段对话。
  参考手册里它默认是 300 秒，也就是五分钟。这里改成 600 秒，十分钟。
- `regions` 放只在某个地方生效的规则。这条规则说：在 `shop` 里，说话的人（`actor`）如果是
  `shopkeeper`，`talk` 这个动作就禁止（`forbid`）。别人在店里照样可以跟 Ana 说话。

这一部分能写哪些动作、角色和参数，都在参考手册里：
[`configure/conversation.yaml`](../reference/interaction-list.md#configureconversationyaml)。

```console
$ mineworld validate corner-shop
...
corner-shop is a valid World Pack.
```

**你应该看到**世界写对了。

**如果你看到** `is not a key of a section`，就是文件里某个 key 拼错了。参考手册列出了每一部分能有哪些 key：
[the shape every section shares](../reference/interaction-list.md#the-shape-every-section-shares)。

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

**你应该看到**，在小店里 `"gap": 600`，还有一条规则禁止 `shopkeeper` 说话（`talk`）。
`selectors` 按顺序对应四个角色：说话的人、说话的对象、物品、地点。`*` 表示谁都算。
最后几行是每样东西的 class：Ana 是 `shopkeeper`，Ben 和 Cai 只是普通的 `person`。

### Step 7.6 — 再跑一次，比一比

**照着做。** 用同一个 seed，再跑同样的三天：

```console
$ mineworld run corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 298
...
activity   days 1-3  ana move 155 talk 81  ben move 159 talk 118  cai move 170 talk 99
...
```

**你应该看到**这几个数字。和第 7.1 步放在一起看：

| | 第 7.1 步 | 第 7.6 步 |
| --- | --- | --- |
| 所有人的 `talk accepted` | 335 | 298 |
| Ana 的 `talk` | 108 | 81 |

Ana 说话少了：在店里她一句也不说，只在家里说。Ben 和 Cai 在店里还是可以跟她说话。
另外两个人的数字也变了一点，因为大家的一天都跟着有点不一样了。变的是世界，System Pack 一点没改。

## 为什么是这样

一条规则，归管这个动作的 System Pack 所有。世界只能调这个 System Pack 开放出来的东西：它的动作、角色和数字。
这样，一个世界的规则都集中在 `configure` 里，一眼能看全；server 对每个玩家、每个程序都按同一套规则检查。
Interaction List 的规则在 [`MODULE_SPEC.md`](../MODULE_SPEC.md#42-the-worlds-interaction-list)。

下一章：[08 时间和天气](08-time-and-weather.zh-CN.md)。

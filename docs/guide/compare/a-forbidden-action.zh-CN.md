# 不许做的动作

这一页让小店里谁都不许说话。你用同一个 seed，把 Corner Shop 在改动前后各跑一次，比一比大家说了多少话。

这一页翻译自英文原文 [a-forbidden-action.md](a-forbidden-action.md)。两边不一致时，以英文为准。

谁能做什么、在哪里能做，写在这个世界的 **Interaction List** 里。世界用 `configure` 文件夹里的文件改它，
System Pack 一行都不用动（[第 07 章](../07-rules-of-the-world.zh-CN.md)一步一步讲过）。

### Step 1.1 — 从做完的 Corner Shop 开始

**照着做。** 在放世界的文件夹里新建文件夹 `compare-rules`。把第 05 章做完的 Corner Shop 复制进去，
它在你第 01 章 clone 下来的文件夹里。macOS 和 Linux 上这样写：

```sh
mkdir compare-rules
cp -R "$HOME/MineWorld/docs/guide/examples/corner-shop" compare-rules/
```

Windows 上，在 PowerShell 里这样写：

```powershell
mkdir compare-rules
Copy-Item -Recurse "$HOME\MineWorld\docs\guide\examples\corner-shop" compare-rules
```

<!-- guide: example docs/guide/examples/corner-shop compare-rules/corner-shop -->

然后跑三天：

```console
$ mineworld run compare-rules/corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 335
...
activity   days 1-3  ana move 145 talk 108  ben move 157 talk 120  cai move 158 talk 107
...
```

**你应该看到**这几个数字：每个人都说了话，加起来 335 次。

### Step 1.2 — 告诉世界它有配置文件

**照着做。** 用下面这整份文件替换 `compare-rules/corner-shop/world.yaml`。新加的是最后的 `configure`：

```yaml file=compare-rules/corner-shop/world.yaml
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
  - conversation
```

```console exit=1
$ mineworld validate compare-rules/corner-shop
[mineworld] configure: names 'conversation', but compare-rules/corner-shop/configure/conversation.yaml does not exist
```

**你应该看到**检查在要 `configure/conversation.yaml`。

### Step 1.3 — 改这一处：小店里不许说话

**照着做。** 新建文件夹 `compare-rules/corner-shop/configure`，在里面写 `conversation.yaml`：

```yaml file=compare-rules/corner-shop/configure/conversation.yaml
regions:
  shop:
    rules:
      - action: talk
        effect: forbid
```

这条规则没写角色，所以对小店里的每个人都算数。规则能写哪些动作和角色，都在参考手册里：
[`configure/conversation.yaml`](../../reference/interaction-list.md#configureconversationyaml)。

```console
$ mineworld interactions compare-rules/corner-shop --place shop
interactions  Corner Shop (corner-shop)
section  conversation:
{
  "place": "shop",
  "region": {
    "consequences": [],
    "default": "permit",
    "parameters": {
      "forget_after_days": 0,
      "gap": 300,
      "range": 3000,
      "remembered": 32
    },
    "rules": [
      {
        "action": "talk",
        "effect": "forbid",
        "selectors": [
          "*",
          "*",
          "*",
          "*"
        ]
      }
    ],
    "scoped": []
  }
}
...
```

**你应该看到**一条禁止 `talk` 的规则，每个角色都是 `*`：谁对谁都不行。

### Step 1.4 — 再跑一次

**照着做。** 用同一个 seed 跑同样的三天：

```console
$ mineworld run compare-rules/corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 292
...
activity   days 1-3  ana move 155 talk 81  ben move 159 talk 116  cai move 170 talk 95
...
```

**你应该看到**这几个数字。

### 世界这样变了

| | Step 1.1 | Step 1.4 |
| --- | --- | --- |
| 所有人的 `talk accepted` | 335 | 292 |
| Ana 的 `talk` | 108 | 81 |
| Ben 的 `talk` | 120 | 116 |
| Cai 的 `talk` | 107 | 95 |

Ana 一整天都在小店里干活，所以她少说的话最多。Ben 和 Cai 只是去店里逛逛，所以只少了一点。
没有谁在店里想说话然后被拒绝：世界根本不会把不许做的动作摆出来，人也就不会去挑它。
大家这一天过得和以前有点不一样，所以走路的数字也跟着变了。

回到[所有对比](README.zh-CN.md)。

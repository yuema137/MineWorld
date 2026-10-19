# 13 被拒绝的时候

文件写错了，`mineworld` 就会拒绝这个世界，并用一条信息说明原因。这一章列出这份教程可能让你碰到的每条信息：
它是什么意思，该怎么改。

这一页翻译自英文原文 [13-when-something-is-refused.md](13-when-something-is-refused.md)。两边不一致时，以英文为准。

这一章可以当清单查：每一步的标题就是一种错误。也可以跟着做：你会做一个叫 `fix-me` 的小世界，
故意每次弄错一处，再把它改回来。

每条信息都会点名该改哪份文件。被拒绝的世界不会启动，所以写错的地方不会悄悄地跑起来。

### Step 13.1 — world.yaml 列出的文件不存在

**照着做。** 新建文件夹 `fix-me`，在里面写 `world.yaml`：

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml lists the place 'home', but fix-me/places/home.yaml does not exist (a place's key names its file)
```

**你应该看到**这条拒绝信息。`places`、`population`、`items` 和 `organizations` 下面的每个 key 都对应一份文件。
**怎么改：**把它点名的文件写出来，或者检查 key 有没有拼错。

### Step 13.2 — 写房间

**照着做。** 在 `fix-me/places` 里写 `home.yaml`：

```yaml file=fix-me/places/home.yaml
tags:
  - home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml lists the person 'ana', but fix-me/people/ana.yaml does not exist (a person's key names its file)
```

**你应该看到**同一种拒绝，这次缺的是 Ana 的文件。

### Step 13.3 — 写人

**照着做。** 在 `fix-me/people` 里写 `ana.yaml`：

```yaml file=fix-me/people/ana.yaml
location:
  place: home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**你应该看到**世界写对了。下面开始故意弄错。

### Step 13.4 — 人在一个不存在的地方

**照着做。** 用下面这份文件替换 `fix-me/people/ana.yaml`，`home` 被拼成了 `hom`：

```yaml file=fix-me/people/ana.yaml
location:
  place: hom
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml puts 'ana' in the place 'hom', which this pack does not declare (it has: 'home')
```

**你应该看到**这条拒绝信息，最后还列出了世界里真有的地方。**怎么改：**换成其中一个 key；
或者把这个地方加进 `world.yaml` 的 `places`，再把它的文件写出来。

### Step 13.5 — 文件里的一段，归一个世界没用的 System Pack 管

**照着做。** 用下面这份文件替换 `fix-me/people/ana.yaml`，这次给 Ana 起了名字：

```yaml file=fix-me/people/ana.yaml
name: Ana Lee
location:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml gives 'ana' a name, which the 'naming' system owns, but world.yaml does not enable it
```

**你应该看到**这条拒绝信息。`name` 归 `naming` 这个 System Pack 管，可这个世界没列它。
**怎么改：**把这个 System Pack 加进 `world.yaml` 的 `systems`；或者把这一段从文件里删掉。
每一段归哪个 System Pack 管，见 [sections 参考](../reference/sections.md#sections)。

### Step 13.6 — key 拼错了

**照着做。** 用下面这份文件替换 `fix-me/people/ana.yaml`，`location` 被拼成了 `locaton`：

```yaml file=fix-me/people/ana.yaml
locaton:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml is not a valid person file:
error: line 1 column 1: unknown field `locaton`, expected one of `tags`, `note`, `location`, [..]
...
```

**你应该看到** `unknown field`，后面是出错的行号和列号，再后面是这份文件能写的 key。
**怎么改：**把 key 改对。MineWorld 不会跳过它不认识的 key。因为你以为写进去了、其实没生效，比直接被拒绝更麻烦。

### Step 13.7 — 把 Ana 改回来

**照着做。** 用下面这份文件替换 `fix-me/people/ana.yaml`：

```yaml file=fix-me/people/ana.yaml
location:
  place: home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**你应该看到**世界又写对了。

### Step 13.8 — 门通到一个不存在的地方

**照着做。** 用下面这份文件替换 `fix-me/places/home.yaml`，这扇门通到一家世界里没有的小店：

```yaml file=fix-me/places/home.yaml
tags:
  - home
passages:
  - to: shop
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/places/home.yaml gives 'home' a passage to 'shop', which this pack does not declare (it has: 'home')
```

**你应该看到**这条拒绝信息。**怎么改：**把另一个地方加进 `places`，再写出它的文件，
做法和[第 03 章](03-two-rooms.zh-CN.md#step-31--把小店写进世界)一样；或者改对 `to` 后面的 key。

### Step 13.9 — 把房间改回来

**照着做。** 用下面这份文件替换 `fix-me/places/home.yaml`：

```yaml file=fix-me/places/home.yaml
tags:
  - home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**你应该看到**世界又写对了。

### Step 13.10 — seat 不是这个世界里的人

**照着做。** 用下面这份文件替换 `fix-me/world.yaml`，它给一个叫 `ben` 的人留了 seat：

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ben
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml offers the seat 'ben', which is not one of its people
```

**你应该看到**这条拒绝信息。seat 必须是 `population` 里列着的人，见参考手册 [`world.yaml`](../reference/world-pack.md#worldyaml) 里的 `seats`。**怎么改：**把这个人加进 `population`；或者把这个 seat 去掉。

### Step 13.11 — 配置文件不存在

**照着做。** 用下面这份文件替换 `fix-me/world.yaml`，它在 `configure` 里点名了一份文件：

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana

configure:
  - classes
```

```console exit=1
$ mineworld validate fix-me
[mineworld] configure: names 'classes', but fix-me/configure/classes.yaml does not exist
```

**你应该看到**这条拒绝信息。**怎么改：**把这份文件写出来，做法和[第 07 章](07-rules-of-the-world.zh-CN.md#step-73--用-class-把-ana-归成店主)一样；
或者把这个 key 从 `configure` 里删掉。

### Step 13.12 — 世界的 id 和文件夹名不一样

**照着做。** 用下面这份文件替换 `fix-me/world.yaml`，它的 `id` 写成了 `fixme`：

```yaml file=fix-me/world.yaml
world:
  id: fixme
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana
```

```console exit=1
$ mineworld validate fix-me
[mineworld] world.yaml calls this pack 'fixme', but its directory is named 'fix-me'; a pack's id is its directory name
```

**你应该看到**这条拒绝信息。**怎么改：**让 `id` 和文件夹名一样。把一个世界复制到新文件夹时，记得也改它的 `id`。

### Step 13.13 — 把 world.yaml 改回来

**照着做。** 用下面这份文件替换 `fix-me/world.yaml`：

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana

seats:
  - ana
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**你应该看到**世界又写对了。

### Step 13.14 — world.yaml 没有列出的文件

**照着做。** 在 `fix-me/people` 里写 `zoe.yaml`：

```yaml file=fix-me/people/zoe.yaml
location:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/zoe.yaml describes a person that world.yaml does not list; add 'zoe' to population, or remove the file
```

**你应该看到**这条拒绝信息。`people`、`places`、`items` 和 `organizations` 里的每份文件，都必须在 `world.yaml` 里列出来，参考手册的 [`world.yaml`](../reference/world-pack.md#worldyaml) 写明了这一点。
**怎么改：**把它列出来，下一步就是这么做的；或者把这份文件删掉。

### Step 13.15 — 把 Zoe 列进去

**照着做。** 用下面这份文件替换 `fix-me/world.yaml`：

```yaml file=fix-me/world.yaml
world:
  id: fix-me
  name: Fix Me

systems:
  - presence
  - movement

places:
  - home

population:
  - ana
  - zoe

seats:
  - ana
```

```console
$ mineworld validate fix-me
...
  people     ana, zoe
...
fix-me is a valid World Pack.
```

**你应该看到**世界写对了，里面有两个人。Zoe 不是 seat，所以谁都玩不了她；世界跑起来时，规则照样会操控她。

## 教程里的其他拒绝信息

| 信息的一部分 | 在哪里碰到 | 怎么改 |
| --- | --- | --- |
| `configure: names 'calendar', but … does not exist` | [第 8.1 步](08-time-and-weather.zh-CN.md#step-81--加上日历和天气) | 把这个 key 对应的文件写出来 |
| `is not a key of a section` | Interaction List 的某份 `configure` 文件里 key 拼错了，见[第 07 章](07-rules-of-the-world.zh-CN.md#step-74--写规则) | 用 [the shape every section shares](../reference/interaction-list.md#the-shape-every-section-shares) 里列出的 key |
| `is neither a role nor a parameter of` | 同一类文件里参数拼错了；信息里会列出正确的参数名 | 用信息或[参考手册](../reference/interaction-list.md#parameters-of-configureconversationyaml)里列出的参数 |
| `which declares nothing — it needs items/<key>.yaml` | [第 12.3 步](12-packs.zh-CN.md#step-123--开始做一个-entity-pack) | 给 Entity Pack 至少加一份物品文件 |
| `requires: … is not in this build, and no pack directory was given` | [第 12.5 步](12-packs.zh-CN.md#step-125--让-corner-shop-用这个-pack) | 加上 `--packs`，指向放这个 pack 的文件夹 |

## 为什么是这样

每份文件都由管它的那段代码来读。这段代码看不懂的东西，一律拒绝。所以信息能点名到文件、行号和 key。规则在
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads)；每个字段都在[参考手册](../reference/README.md)里。

回到[教程目录](README.zh-CN.md)。

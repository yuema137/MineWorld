# 13 被拒绝的时候

文件写错时，`mineworld` 会拒绝这个世界，并用一条信息说明原因。这一章列出这份教程可能带你遇到的每条信息：
它是什么意思，怎么改。

本页是英文原文 [13-when-something-is-refused.md](13-when-something-is-refused.md) 的中文镜像。两者不一致时，以英文为准。

你可以把它当清单查：每一步的标题就是一种错误。也可以跟着做：你会做一个叫 `fix-me` 的小世界，
故意一次弄错一个地方，再把它改好。

每条信息都会点名要改的文件。被拒绝的世界不会启动，所以错误永远不会悄悄地跑起来。

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

**你应该看到**这条拒绝信息。`places`、`population`、`items` 和 `organizations` 下面的每个键都对应一个文件。
**怎么改：**把它点名的文件写出来，或者检查键有没有拼错。

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

**你应该看到**同一类拒绝，这次是 Ana 的。

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

**你应该看到**世界有效。现在来把它弄坏。

### Step 13.4 — 人在一个不存在的地方

**照着做。** 用下面这个文件替换 `fix-me/people/ana.yaml`，里面 `home` 拼错了：

```yaml file=fix-me/people/ana.yaml
location:
  place: hom
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml puts 'ana' in the place 'hom', which this pack does not declare (it has: 'home')
```

**你应该看到**这条拒绝信息。它会列出世界里真有的地方。**怎么改：**用其中一个键，
或者把这个地方加进 `world.yaml` 的 `places`，再写出它的文件。

### Step 13.5 — 文件里有一部分，归一个世界没用的系统包管

**照着做。** 用下面这个文件替换 `fix-me/people/ana.yaml`，给 Ana 起了名字：

```yaml file=fix-me/people/ana.yaml
name: Ana Lee
location:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/ana.yaml gives 'ana' a name, which the 'naming' system owns, but world.yaml does not enable it
```

**你应该看到**这条拒绝信息。`name` 归 `naming` 这个系统包管，可这个世界没列它。
**怎么改：**把这个系统包加进 `world.yaml` 的 `systems`，或者把这一部分从文件里拿掉。
每一部分归哪个系统包管，见 [sections 参考](../reference/sections.md#sections)（英文）。

### Step 13.6 — 键拼错了

**照着做。** 用下面这个文件替换 `fix-me/people/ana.yaml`，里面 `location` 拼错了：

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

**你应该看到** `unknown field`（不认识的字段）、出错的行号和列号，以及这个文件可以写哪些键。
**怎么改：**把键改对。MineWorld 从不跳过它不认识的键：你以为写进去了、其实没生效的世界，比直接被拒绝更糟。

### Step 13.7 — 把 Ana 改回来

**照着做。** 用下面的内容替换 `fix-me/people/ana.yaml`：

```yaml file=fix-me/people/ana.yaml
location:
  place: home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**你应该看到**世界又有效了。

### Step 13.8 — 门通向一个不存在的地方

**照着做。** 用下面这个文件替换 `fix-me/places/home.yaml`，它的门通向一家世界里没有的小店：

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

**你应该看到**这条拒绝信息。**怎么改：**把另一个地方加进 `places` 并写出它的文件，
就像[第 03 章](03-two-rooms.zh-CN.md#step-31--把小店列进世界)那样；或者改正 `to` 后面的键。

### Step 13.9 — 把房间改回来

**照着做。** 用下面的内容替换 `fix-me/places/home.yaml`：

```yaml file=fix-me/places/home.yaml
tags:
  - home
```

```console
$ mineworld validate fix-me
...
fix-me is a valid World Pack.
```

**你应该看到**世界又有效了。

### Step 13.10 — 座位不是一个人

**照着做。** 用下面这个文件替换 `fix-me/world.yaml`，它给 `ben` 留了一个座位：

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

**你应该看到**这条拒绝信息。座位必须是 `population` 里列出的人。**怎么改：**把这个人加上，或者把这个座位去掉。

### Step 13.11 — 配置文件不存在

**照着做。** 用下面这个文件替换 `fix-me/world.yaml`，它点名了一个 `configure` 文件：

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

**你应该看到**这条拒绝信息。**怎么改：**把这个文件写出来，就像[第 07 章](07-rules-of-the-world.zh-CN.md#step-73--把-ana-归进店主这个类别)那样；
或者把这个键从 `configure` 里拿掉。

### Step 13.12 — 世界的 id 和文件夹名不一样

**照着做。** 用下面这个文件替换 `fix-me/world.yaml`，它的 `id` 是 `fixme`：

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

**你应该看到**这条拒绝信息。**怎么改：**让 `id` 和文件夹名一样。把一个世界复制到新文件夹时，也要改它的 `id`。

### Step 13.13 — 把 world.yaml 改回来

**照着做。** 用下面的内容替换 `fix-me/world.yaml`：

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

**你应该看到**世界又有效了。

### Step 13.14 — world.yaml 没列出的文件

**照着做。** 在 `fix-me/people` 里写 `zoe.yaml`：

```yaml file=fix-me/people/zoe.yaml
location:
  place: home
```

```console exit=1
$ mineworld validate fix-me
[mineworld] fix-me/people/zoe.yaml describes a person that world.yaml does not list; add 'zoe' to population, or remove the file
```

**你应该看到**这条拒绝信息。`people`、`places`、`items` 和 `organizations` 里的每个文件都必须被列出来。
**怎么改：**把它列出来（下一步就这么做），或者删掉这个文件。

### Step 13.15 — 把 Zoe 列进去

**照着做。** 用下面的内容替换 `fix-me/world.yaml`：

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

**你应该看到**世界有效，里面有两个人。Zoe 不是座位，所以没人能玩她；世界运行时，规则照样操控她。

## 教程里的其他拒绝信息

| 信息（部分） | 在哪里遇到 | 怎么改 |
| --- | --- | --- |
| `configure: names 'calendar', but … does not exist` | [第 8.1 步](08-time-and-weather.zh-CN.md#step-81--加上日历和天气) | 把这个键对应的文件写出来 |
| `is not a key of a section` | 交互清单的某个 `configure` 文件里键拼错了，[第 07 章](07-rules-of-the-world.zh-CN.md#step-74--写规则) | 用 [the shape every section shares](../reference/interaction-list.md#the-shape-every-section-shares)（英文）里列出的键 |
| `is neither a role nor a parameter of` | 同样这些文件里参数拼错了；信息里会列出正确的参数 | 用信息或[参考手册](../reference/interaction-list.md#parameters-of-configureconversationyaml)（英文）里列出的参数 |
| `which declares nothing — it needs items/<key>.yaml` | [第 12.3 步](12-packs.zh-CN.md#step-123--开始做一个实体包) | 给实体包加至少一个物品文件 |
| `requires: … is not in this build, and no pack directory was given` | [第 12.5 步](12-packs.zh-CN.md#step-125--让街角小店要求用这个包) | 加上 `--packs`，指向放这个包的文件夹 |

## 为什么是这样

每个文件都由管它的那段代码来读，那段代码会拒绝一切它看不懂的东西。所以信息能点名到文件、行号和键。规则在
[`MODULE_SPEC.md`](../MODULE_SPEC.md#41-the-fields-mvp-0s-loader-reads)（英文）；每个字段都在
[参考手册](../reference/README.md)（英文）里。

回到[教程目录](README.zh-CN.md)。

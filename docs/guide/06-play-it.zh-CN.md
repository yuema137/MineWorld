# 06 进去玩

这一章你会把 Corner Shop 开起来，用 2D client 以 Ben 的身份走进去。你玩的时候，世界不会停：
Ana 和 Cai 照常过他们的一天，Ben 的一天由你来过。

这一页翻译自英文原文 [06-play-it.md](06-play-it.md)。两边不一致时，以英文为准。

<!-- guide: continues 05-one-shop -->

接着用[第 05 章](05-one-shop.zh-CN.md)的 `corner-shop` 文件夹。

跑教程的测试会检查第 6.1 步。第 6.2 和 6.3 步要开窗口，要有人坐在键盘前，所以由你自己检查。

### Step 6.1 — 看看你能玩谁

玩家能操控的人叫 **seat**。Corner Shop 有三个 seat：`ana`、`ben` 和 `cai`。

**照着做。**

```console
$ mineworld validate corner-shop
...
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**你应该看到**这三个 seat。

### Step 6.2 — 把世界开起来

**照着做。** 启动一个 server。server 是一个一直开着的程序：世界在里面不停地跑，玩家连上它就能加入。
加上 `--town`，没人占着的 seat 就交给那条简单的规则来操控，和 `run` 一样。

<!-- guide: operator-only -->

```console
$ mineworld server corner-shop --town
[mineworld] Corner Shop (corner-shop) — 11 system(s), 3 seat(s)
[mineworld] listening on http://127.0.0.1:7878 (ws://127.0.0.1:7878/ws), protocol [..]
[mineworld] 8 entities, 11 system(s), seats: ana, ben, cai
[mineworld] world instance [..]
[mineworld] invite [..] — join with: 127.0.0.1:7878 seat=ana invite=[..]
...
[mineworld] hold 30 s, time scale 1, request budget 10/s burst 20 per seat, 3 seat(s) driven in-server (town: seed 0, pace 5 wall s)
```

**你应该看到**有一行以 `[mineworld] invite` 开头。`invite` 后面那一长串就是 invite 码，玩家要用它才能加入。
最后一行说的是：在有玩家接手之前，三个 seat 都由规则操控。从现在起，世界里的时间和现实一样快。

server 会一直开着，直到你按 Ctrl+C。先别关它，再开一个终端，做下一步。

不加 `--save` 的话，世界每次都从头开始。想让它两次之间接着跑，就加上 `--save saves/play`，
和[第 04 章](04-three-people.zh-CN.md#step-45--跑一天并且存下来)里一样。

### Step 6.3 — 以 Ben 的身份走一走

2D client 就在[第 01 章](01-get-mineworld.zh-CN.md)下载的 MineWorld 源代码文件夹里。
它要求 `PATH` 上有 [Godot 4.7](https://godotengine.org)。

**照着做。** 在第二个终端里进到 `MineWorld` 文件夹，以 Ben 的身份加入世界。
把下面的 `INVITE` 换成你的 server 打印出来的那串 invite 码：

```sh
./mineworld-2d --server 127.0.0.1:7878 --invite INVITE --seat ben
```

`mineworld-2d` 是一个 `bash` 脚本。Windows 上要准备什么，看 2D client 自己的说明
[`clients/2d/README.md`](../../clients/2d/README.md)。

**你应该看到**一个窗口，斜着从上往下画出 Ben 的家，里面有 Ben、Cai 和一扇门。
现在你就是 Ben 了：你占着他的 seat，规则就不再管他。点一个地方，Ben 就走过去。
穿过门走进小店。点 Ana，看你能对她做什么，比如聊天。点你自己，或者按 Q，看你自己能做什么，比如买面包。

你做的每件事，都是向世界发一个请求。世界说行或不行，client 再把结果画出来。
菜单里变灰的一项，会写明世界为什么会说不行。

**关于 3D client。** 现在的 3D client 只会画示例小镇里那家咖啡馆。在 3D 里逛你自己的世界，以后会支持。

## 为什么是这样

玩家、规则、程序，操控一个人的方式都一样：占住一个 seat，然后向世界发请求。
所以第 05 章里 Ben 由规则操控，这一章由你操控，世界本身一个字都不用改。规则在
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#14-controller) 和 [`NETWORKING.md`](../NETWORKING.md)。

下一章：[07 世界的规则](07-rules-of-the-world.zh-CN.md)。

# 06 玩起来

这一章你会把街角小店开起来，在 2D 客户端里以 Ben 的身份走进去逛。你玩的时候，世界一直在运行：
Ana 和 Cai 照常过他们的一天，Ben 的一天由你来过。

本页是英文原文 [06-play-it.md](06-play-it.md) 的中文镜像。两者不一致时，以英文为准。

<!-- guide: continues 05-one-shop -->

接着用[第 05 章](05-one-shop.zh-CN.md)的 `corner-shop` 文件夹。

跑这份教程的测试会检查第 6.1 步。第 6.2 和 6.3 步需要一个窗口和坐在键盘前的人，所以要你自己来检查。

### Step 6.1 — 看看你能玩谁

**座位**（seat）是玩家可以操控的人。街角小店有三个座位：`ana`、`ben` 和 `cai`。

**照着做。**

```console
$ mineworld validate corner-shop
...
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**你应该看到**这三个座位。

### Step 6.2 — 把世界开起来

**照着做。** 启动一个**服务器**（server）：让世界一直运行、让玩家加入的程序。
`--town` 让那条简单的规则去操控所有没有玩家占着的座位，就像 `run` 那样。

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

**你应该看到**一行以 `[mineworld] invite` 开头。`invite` 后面那串长长的码就是**邀请码**（invite）：
玩家要用它才能加入。最后一行说明：在有玩家接手之前，三个座位都由规则来操控。
现在世界里的时间和现实一样快。

服务器会一直运行，直到你按 Ctrl+C。别关它，再打开第二个终端，做下一步。

不加 `--save` 的话，世界每次都从头开始。想在两次之间保留下来，就加上 `--save saves/play`，
就像[第 04 章](04-three-people.zh-CN.md#step-45--跑一天并且存下来)那样。

### Step 6.3 — 以 Ben 的身份走一走

2D 客户端就在[第 01 章](01-get-mineworld.zh-CN.md)下载的 MineWorld 源代码文件夹里。
它需要 `PATH` 上有 [Godot 4.7](https://godotengine.org)。

**照着做。** 在第二个终端里，进入 `MineWorld` 文件夹，以 Ben 的身份加入世界。
把 `INVITE` 换成你的服务器打印出来的邀请码：

```sh
./mineworld-2d --server 127.0.0.1:7878 --invite INVITE --seat ben
```

`mineworld-2d` 是一个 `bash` 脚本。在 Windows 上需要什么，见 2D 客户端自己的说明
[`clients/2d/README.md`](../../clients/2d/README.md)（英文）。

**你应该看到**一个窗口，从斜上方画出 Ben 的家，里面有 Ben、Cai 和一扇门。
现在你就是 Ben；你占着他的座位时，规则就不再操控他。点一个地方就会走过去。
穿过门走进小店。点 Ana，看看你能对她做什么，比如聊天。点你自己，或者按 Q，看看你自己能做什么，比如买面包。

你做的每件事，都是向世界提出一个请求。世界说行或不行，客户端把结果画出来。
菜单里变灰的一项，会告诉你世界为什么会说不行。

**关于 3D 客户端。** 现在的 3D 客户端只画得出示例小镇里的那家咖啡馆。在 3D 里逛你自己的世界，还在路上。

## 为什么是这样

玩家、规则、程序操控一个人的方式都一样：通过座位，向世界提出请求。
所以第 05 章里 Ben 由规则操控，这里由你操控，世界本身一点都不用改。规则在
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#14-controller) 和 [`NETWORKING.md`](../NETWORKING.md)（英文）。

下一章：[07 世界的规则](07-rules-of-the-world.zh-CN.md)。

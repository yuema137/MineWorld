# 01 安装 MineWorld

这一章你会拿到 `mineworld` 命令，确认它能用，然后让第一个世界跑两天。大约十分钟。

本页是英文原文 [01-get-mineworld.md](01-get-mineworld.md) 的中文镜像。两者不一致时，以英文为准。

`mineworld` 是一个程序。它能检查世界、运行世界，也能把世界开出来给玩家玩。
它的命令要在**终端**（terminal）里输入：macOS 上是“终端”，Linux 上是终端窗口，Windows 上是 PowerShell。

### Step 1.1 — 拿到 `mineworld` 命令

**照着做。** 第一个正式版本会提供 macOS、Linux 和 Windows 的现成下载，正在准备。
在那之前，先从源代码自己编译。你需要 [Git](https://git-scm.com) 和
[Rust](https://rustup.rs)。装好后，把下面三条命令跑一次：

```sh
git clone https://github.com/yuema137/MineWorld
cd MineWorld
cargo build --release -p mineworld-cli
```

第一次编译要几分钟。编译完会得到一个文件：macOS 和 Linux 上是 `target/release/mineworld`，
Windows 上是 `target\release\mineworld.exe`。

在 `MineWorld` 文件夹外面，新建一个空文件夹，用来放你的世界。在这个文件夹里打开终端。
然后把程序所在的文件夹加进 `PATH`，让终端找得到 `mineworld`。macOS 和 Linux 上：

```sh
export PATH="$HOME/MineWorld/target/release:$PATH"
```

Windows 上，在 PowerShell 里：

```powershell
$env:PATH = "$HOME\MineWorld\target\release;$env:PATH"
```

如果你把代码克隆到了别的地方，就把 `$HOME/MineWorld` 换成那个位置。这个设置在关掉终端前一直有效。

现在问问 `mineworld` 它是哪个版本：

```console
$ mineworld --version
mineworld [..]
```

**你应该看到** `mineworld` 加一个版本号，比如 `mineworld 0.1.0`。

**如果你看到** `command not found` 或 `is not recognized`，说明终端找不到这个程序。
检查一下你加进 `PATH` 的文件夹对不对，或者直接输入程序的完整路径来代替 `mineworld`。

### Step 1.2 — 看看它能做什么

**照着做。** 打开它的帮助：

```console
$ mineworld --help
Run a MineWorld world

Usage: mineworld <COMMAND>

Commands:
  server        Host a World Pack for clients
  validate      Check a World Pack and say what it is
...
```

**你应该看到**一列命令。这份教程会用到 `create`、`validate`、`run`、`biography`、
`interactions`、`server` 和 `packs`。每条命令的说明都在[命令参考](../reference/cli.md#the-mineworld-command)里（英文）。

### Step 1.3 — 做一个世界来试试

**世界包**（World Pack）是一个装着文本文件的文件夹，用来描述一个世界：有哪些地方、哪些人、
用哪些规则（[World](../CORE_CONCEPTS.md#16-world)，英文）。`mineworld create` 会帮你写一个很小的世界包。

**照着做。**

```console
$ mineworld create hello
created hello: World Pack 'hello' — 1 place(s), 2 person(s), seats first, second; 2 genesis fact(s)
next: mineworld run hello --headless --seed 1 --days 7
```

**你应该看到**它做好了世界 `hello`，里面有一个地方、两个人。你的文件夹里多了一个叫 `hello` 的文件夹。

### Step 1.4 — 让它跑两天

**照着做。** 不开窗口，让世界跑两天（世界里的时间）：

```console
$ mineworld run hello --headless --seed 1 --days 2
[mineworld] run hello (hello): seats first, second; seed 1; to day 2; pace 900s; in memory
day 1  revision -  facts [..]
day 2  revision -  facts [..]
...
activity   days 1-2  first move [..] talk [..]  second move [..] talk [..]
faults     0
...
```

**你应该看到**每天一行，然后是一段总结。两个人走来走去，还聊了天。
`faults     0` 表示没有出错。

这几个词的意思：

- `--headless`：不开窗口，也不联网，直接运行世界。
- `--seed 1`：选定“骰子”（随机种子，seed）。同一个种子，每次、每台电脑上跑出的结果都完全一样。
- `--days 2`：一直跑到世界满两天。世界里的两天，现实里不到一秒。
- **事实**（fact）：发生过的一件事，比如“first 对 second 说了话”。世界会记下所有事实。

## 为什么是这样

`run` 用一条简单的、带种子的规则来操控每个人。所以不需要玩家，也不需要语言模型，就能测试一个世界。
这条命令的参数在[命令参考](../reference/cli.md#mineworld-run)里（英文）。

下一章：[02 你的第一个世界](02-your-first-world.zh-CN.md)。

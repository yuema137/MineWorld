# 01 安装 MineWorld

这一章你会拿到 `mineworld` 命令，确认它能用，再让第一个世界跑两天。大约十分钟。

这一页翻译自英文原文 [01-get-mineworld.md](01-get-mineworld.md)。两边不一致时，以英文为准。

`mineworld` 是一个程序。它能检查一个世界写得对不对，能让世界跑起来，也能把世界开出来给玩家玩。
它的命令在终端里输入：macOS 上用“终端”，Linux 上用终端窗口，Windows 上用 PowerShell。

### Step 1.1 — 拿到 `mineworld` 命令

**照着做。** 第一个正式版本会带 macOS、Linux 和 Windows 的现成下载，现在还没发布。
在那之前，先从源代码自己 build。你需要先装好 [Git](https://git-scm.com) 和
[Rust](https://rustup.rs)，然后把下面三条命令跑一次：

```sh
git clone https://github.com/yuema137/MineWorld
cd MineWorld
cargo build --release -p mineworld-cli
```

第一次 build 要几分钟。跑完会多出一个文件：macOS 和 Linux 上是 `target/release/mineworld`，
Windows 上是 `target\release\mineworld.exe`。

在 `MineWorld` 文件夹外面新建一个空文件夹，以后你的世界都放在这里。在这个文件夹里打开终端。
然后把程序所在的文件夹加进 `PATH`，终端才找得到 `mineworld`。macOS 和 Linux 上这样写：

```sh
export PATH="$HOME/MineWorld/target/release:$PATH"
```

Windows 上，在 PowerShell 里这样写：

```powershell
$env:PATH = "$HOME\MineWorld\target\release;$env:PATH"
```

如果你把代码克隆到了别的位置，就把 `$HOME/MineWorld` 换成那个位置。关掉终端以后，这个设置就没了。

现在问问 `mineworld` 是哪个版本：

```console
$ mineworld --version
mineworld [..]
```

**你应该看到** `mineworld` 后面跟一个版本号，比如 `mineworld 0.1.0`。

**如果你看到** `command not found` 或 `is not recognized`，就是终端找不到这个程序。
检查一下加进 `PATH` 的文件夹对不对。也可以不用 `mineworld` 这个名字，直接输入程序的完整路径。

### Step 1.2 — 看看它能做什么

**照着做。** 打开它的帮助：

```console
$ mineworld --help
Run a MineWorld world
...
Commands:
  server        Host a World Pack for clients
  validate      Check a World Pack and say what it is
...
```

**你应该看到**一列命令。这份教程会用到其中的 `create`、`validate`、`run`、`biography`、
`interactions`、`server` 和 `packs`。每条命令怎么用，写在[命令参考](../reference/cli.md#the-mineworld-command)里。

教程里的输出这样读：单独一行 `...`，表示这里省掉了几行；一行里的 `[..]`，表示这一段每次可能不一样，
比如版本号。其余部分应该一字不差。

### Step 1.3 — 做一个世界来试试

一个世界就是一个文件夹，里面放着几份文本文件。文件里写着这个世界有哪些地方、哪些人、用哪些模块。
MineWorld 把这样的文件夹叫 **World Pack**（[World](../CORE_CONCEPTS.md#16-world)）。
`mineworld create` 会帮你写一个很小的 World Pack。

**照着做。**

```console
$ mineworld create hello
created hello: World Pack 'hello' — 1 place(s), 2 person(s), seats first, second; 2 genesis fact(s)
next: mineworld run hello --headless --seed 1 --days 7
```

**你应该看到**它做好了世界 `hello`，里面有一个地方、两个人。你的文件夹里多了一个 `hello` 文件夹。

### Step 1.4 — 让它跑两天

**照着做。** 不开窗口，让这个世界跑两天。这里的两天是世界里的时间：

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

**你应该看到**每天一行，最后是一段总结。两个人走来走去，还聊了天。
`faults     0` 表示一个错误都没有。

这几个参数的意思：

- `--headless`：不开窗口，也不连网络，只让世界自己跑。
- `--seed 1`：定下随机数从哪里开始，这个起点叫 seed。seed 一样，每次跑、在哪台电脑上跑，结果都完全一样。
- `--days 2`：一直跑到世界满两天为止。世界里的两天，现实里用不了一秒。

总结里的 `facts` 数的是 fact。一个 fact 就是世界里发生过的一件事，比如“first 对 second 说了一句话”。
世界会把每一件事都记下来。

## 为什么是这样

`run` 用一条简单的规则操控每个人，这条规则的随机数来自 seed。所以测试一个世界，既不需要玩家，也不需要语言模型。
这条命令的所有参数写在[命令参考](../reference/cli.md#mineworld-run)里。

下一章：[02 第一个世界](02-your-first-world.zh-CN.md)。

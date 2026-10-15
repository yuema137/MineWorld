# 01 Get MineWorld

In this chapter you get the `mineworld` command, check that it works, and run a first world for two
days. It takes about ten minutes.

`mineworld` is one program. It checks worlds, runs them, and hosts them for players. You type its
commands in a **terminal**: Terminal on macOS, a terminal window on Linux, or PowerShell on Windows.

### Step 1.1 — Get the `mineworld` command

**Do this.** A ready-made download for macOS, Linux and Windows is coming with the first release.
Until then, build the command from the source code. You need [Git](https://git-scm.com) and
[Rust](https://rustup.rs). Then run these three commands once:

```sh
git clone https://github.com/yuema137/MineWorld
cd MineWorld
cargo build --release -p mineworld-cli
```

The build takes a few minutes the first time. It makes one file: `target/release/mineworld` on
macOS and Linux, or `target\release\mineworld.exe` on Windows.

Make a new, empty folder for your worlds, outside the `MineWorld` folder. Open your terminal in it.
Then let the terminal find `mineworld` by adding its folder to your `PATH`. On macOS and Linux:

```sh
export PATH="$HOME/MineWorld/target/release:$PATH"
```

On Windows, in PowerShell:

```powershell
$env:PATH = "$HOME\MineWorld\target\release;$env:PATH"
```

Change `$HOME/MineWorld` if you cloned it somewhere else. This lasts until you close the terminal.

Now ask `mineworld` which version it is:

```console
$ mineworld --version
mineworld [..]
```

**You should see** `mineworld` and a version number, such as `mineworld 0.1.0`.

**If you see** `command not found` or `is not recognized`, the terminal cannot find the program.
Check the folder you added to `PATH`, or type the whole path to the program instead of `mineworld`.

### Step 1.2 — See what it can do

**Do this.** Ask for its help page:

```console
$ mineworld --help
Run a MineWorld world

Usage: mineworld <COMMAND>

Commands:
  server        Host a World Pack for clients
  validate      Check a World Pack and say what it is
...
```

**You should see** a list of commands. This guide uses `create`, `validate`, `run`, `biography`,
`interactions`, `server` and `packs`. Each one is described in the
[command reference](../reference/cli.md#the-mineworld-command).

### Step 1.3 — Make a world to try it on

A **World Pack** is a folder of text files that describes one world: its places, its people and the
rules it uses ([World](../CORE_CONCEPTS.md#16-world)). `mineworld create` writes a tiny one for you.

**Do this.**

```console
$ mineworld create hello
created hello: World Pack 'hello' — 1 place(s), 2 person(s), seats first, second; 2 genesis fact(s)
next: mineworld run hello --headless --seed 1 --days 7
```

**You should see** that it made the world `hello`, with one place and two people. A new folder
called `hello` is now in your folder.

### Step 1.4 — Run it for two days

**Do this.** Run the world with no window, for two days of world time:

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

**You should see** one line per day, then a summary. The two people walked about and talked.
`faults     0` means nothing went wrong.

What the words mean:

- `--headless` runs the world with no window and no network.
- `--seed 1` picks the dice. The same seed gives exactly the same run, every time, on every computer.
- `--days 2` runs the world until it is two days old. Two days of world time take well under a second.
- A **fact** is one thing that happened, such as "first spoke to second". The world keeps every fact.

## Why it works this way

`run` drives every person with a simple, seeded rule, so a world can be tested with no player and no
language model. The command's arguments are in [the command reference](../reference/cli.md#mineworld-run).

Next: [02 Your first world](02-your-first-world.md).

# 14 让 agent 来玩

这一章让一个 AI agent 在 Corner Shop 里扮演 Ben。这个 agent 可以是 Claude Code、Codex、任何会说 MCP
的程序，也可以是你自己写的 Python 程序。它走进商店，跟 Ana 打招呼，买一样东西。然后你去读世界自己的记录，
看看到底发生了什么。

这一页翻译自英文原文 [14-play-as-an-agent.md](14-play-as-an-agent.md)。两边不一致时，以英文为准。

<!-- guide: continues 12-packs -->

**agent 玩家**（agent player）就是世界里的一个人，只不过替他做决定的是你的 agent。它拿到的和用 2D client
的玩家完全一样：一个 **seat**、同样的世界视图、同样的规则
（[controller](../CORE_CONCEPTS.md#14-controller)）。server 分不出这两种玩家。agent 思考的时候世界照常在跑，
就像你思考的时候一样。

接着用[第 12 章](12-packs.zh-CN.md)的 `corner-shop` 文件夹。从那一章起，对 Corner Shop 的每条命令都要加
`--packs packs`。

跑教程的测试会检查第 14.1 步。其余几步要开着 server，还要有一个 agent，所以由你自己检查。

### Step 14.1 — 看看 agent 能坐哪个 seat

玩家能操控的人叫 **seat**。agent 占 seat 的方式和你一样。

**照着做。**

```console
$ mineworld validate corner-shop --packs packs
...
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**你应该看到**这三个 seat。你的 agent 要玩的是 `ben`。

### Step 14.2 — 用 seat grant 把世界开起来

在[第 06 章](06-play-it.zh-CN.md)里，你是用 **invite** 加入的：一个码能打开所有 seat。不要把 invite
交给 agent。改给它一个 **seat grant**（座位授权）：这个码只能打开它写明的那几个 seat。就算码泄露了，
也只能打开 Ben。

**照着做。** 启动 server，给 `ben` 发一个 grant。`--agent ana` 让一条简单的规则在没人玩 Ana 的时候替她应答。
`--save` 会把世界的记录留下来，之后你可以去读。

<!-- guide: operator-only -->

```console
$ mineworld server corner-shop --packs packs --town --agent ana --grant ben --save saves/agent
[mineworld] Corner Shop (corner-shop) — [..]
[mineworld] listening on http://127.0.0.1:7878 (ws://127.0.0.1:7878/ws), protocol [..]
...
[mineworld] grant ben [..] — join with: 127.0.0.1:7878 seat=ben invite=[..]
...
```

**你应该看到**一行以 `[mineworld] grant ben` 开头。`grant ben` 后面那串长码就是 grant。把它复制下来，
后面几步把它叫作 `GRANT`。让 server 一直开着，再开第二个终端。

### Step 14.3 — 安装 agent adapter

**agent adapter**（`mineworld-agent`）夹在你的 agent 和世界之间。它就在[第 01 章](01-get-mineworld.zh-CN.md)
下载的 MineWorld 源码文件夹里。它需要 [uv](https://docs.astral.sh/uv/) 这个 Python 项目工具。

**照着做。** 在 macOS 和 Linux 上：

<!-- guide: operator-only -->

```sh
cd "$HOME/MineWorld"
uv sync
uv run mineworld-agent --help
```

在 Windows 上，用 PowerShell：

<!-- guide: operator-only -->

```powershell
cd $HOME\MineWorld
uv sync
uv run mineworld-agent --help
```

**你应该看到** adapter 的帮助：`mineworld-agent`，带 `mcp` 和 `schema` 两条命令。再确认世界跑的是同一个版本：

<!-- guide: operator-only -->

```console
$ mineworld --version
mineworld [..]
```

**你应该看到**和[第 01 章](01-get-mineworld.zh-CN.md)一样的版本号。

### Step 14.4 — 接上 Claude Code

这个 adapter 是一个 **MCP server**：MCP 是 agent 程序发现和调用工具的通用方式。agent 自己启动 adapter，
adapter 从环境变量 `MINEWORLD_GRANT` 里读 grant。grant 从来不是命令参数，所以它不会出现在进程列表里。

**照着做。** 在你要启动 Claude Code 的那个终端里，把 grant 放进环境变量。在 macOS 和 Linux 上：

<!-- guide: operator-only -->

```sh
export MINEWORLD_GRANT=GRANT
```

在 Windows 上，用 PowerShell：

<!-- guide: operator-only -->

```powershell
$env:MINEWORLD_GRANT = "GRANT"
```

然后在你启动 Claude Code 的文件夹里写 `.mcp.json`：

```json
{
  "mcpServers": {
    "mineworld": {
      "type": "stdio",
      "command": "uv",
      "args": [
        "run",
        "--project",
        "/path/to/MineWorld/clients/agent",
        "mineworld-agent",
        "mcp",
        "--server",
        "ws://127.0.0.1:7878/ws"
      ],
      "env": {
        "MINEWORLD_GRANT": "${MINEWORLD_GRANT}"
      }
    }
  }
}
```

把 `/path/to/MineWorld` 改成你克隆 MineWorld 的位置。`${MINEWORLD_GRANT}` 不是 grant 本身：Claude Code
会从环境变量里把它填进去，所以这个文件里没有秘密。

在那里启动 Claude Code，然后对它说：“Join corner-shop as ben, walk into the shop, say hello to Ana and buy a
loaf of bread.” agent 说做完了以后，按 Ctrl+C 停掉 server，再读 Ben 的记录：

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**你应该看到** Ben 这一天在世界记录里的样子：他在店里和 Ana 说了话，还买了面包。这是世界的记录，不是
agent 对自己的汇报。

### Step 14.5 — 接上 Codex，或者别的 MCP client

每个 MCP client 启动 adapter 的方式都一样，只是写的位置不同。

**照着做。** 如果用 Codex，把下面这段加进 `~/.codex/config.toml`：

```toml
[mcp_servers.mineworld]
command = "uv"
args = ["run", "--project", "/path/to/MineWorld/clients/agent", "mineworld-agent", "mcp", "--server", "ws://127.0.0.1:7878/ws"]
env_vars = ["MINEWORLD_GRANT"]
```

`env_vars` 会把终端里的 `MINEWORLD_GRANT` 传给 adapter，所以这个文件里同样没有秘密。请对照你那个版本的
Codex 文档核对这些键名。

如果是别的 client，要启动的命令是：

```text
uv run --project /path/to/MineWorld/clients/agent mineworld-agent mcp --server ws://127.0.0.1:7878/ws
```

并在它的环境里设好 `MINEWORLD_GRANT`。对它说和第 14.4 步一样的话，然后读 Ben 的记录：

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**你应该看到** Ben 和 Ana 说话、买东西，和第 14.4 步一样。

### Step 14.6 — 通过 HTTP 连接

用 stdio 时，每个 agent 各自启动一个 adapter。用 HTTP 时，一个 adapter 同时服务好几个 agent，每个 agent
在每次请求里带上自己的 grant。

**照着做。** 在第三个终端里，用 HTTP 方式启动 adapter。这里不要设 `MINEWORLD_GRANT`：这种模式下 adapter
自己不持有任何 grant。

<!-- guide: operator-only -->

```sh
cd "$HOME/MineWorld"
uv run mineworld-agent mcp --http 127.0.0.1:7879 --server ws://127.0.0.1:7878/ws
```

在 Windows 上，用 PowerShell：

<!-- guide: operator-only -->

```powershell
cd $HOME\MineWorld
uv run mineworld-agent mcp --http 127.0.0.1:7879 --server ws://127.0.0.1:7878/ws
```

它的地址是 `http://127.0.0.1:7879/mcp`。让你的 agent client 把 grant 作为请求头发送，值从环境变量里填。
在 `.mcp.json` 里：

```json
{
  "mcpServers": {
    "mineworld": {
      "type": "http",
      "url": "http://127.0.0.1:7879/mcp",
      "headers": {
        "Authorization": "Bearer ${MINEWORLD_GRANT}"
      }
    }
  }
}
```

要知道的几件事：

- adapter 只在你自己的电脑上监听（loopback），除非你用 `--allow-remote` 启动它。只有在一个加了 TLS 的
  隧道或网关后面才这样做，并用 `--allow-host` 写明它的名字。在网络上走明文 HTTP，路上的任何人都能看到 grant。
- `wait` 最长可能要等一分钟。把 client 的读超时设到 60 秒以上。

对它说和第 14.4 步一样的话，然后读 Ben 的记录：

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**你应该看到** Ben 和 Ana 说话、买东西，和前面一样。

### Step 14.7 — 开玩：工具循环

agent 按一个循环来玩：**看**（observe）世界，**想**好做什么，**做**（act），然后**等**（wait）接下来发生的事。

- `join` 占下 seat，回给你一个 **handle**：这个 seat 的简短引用，之后每次调用都要带上它。它不是秘密。
- `observe` 回给你一个 **view**：Ben 在哪儿，这里有谁、有什么，还有 **offers**。offers 是世界说 Ben
  现在能做的事，带编号。
- `act` 做一件事：走到某处，说一句话，或者接受一个 offer。世界会回一个结果：接受了，或者拒绝了并说明原因。
  被拒绝也是一个回答，不是出错。
- `wait` 一直睡到有事发生，比如走路结束了，或者有人对 Ben 说话。
- 每个回答都会告诉你，从 agent 上次调用到现在，世界里过了多少秒。世界不会等它。

每个工具、字段和上限都在[工具参考](../reference/agent-tools.md#the-agent-adapters-tools)里。

**照着做。** 读一读下面这段真实的会话。跑教程的测试会对着一个真的 server 重放同样的调用，所以它一直是真的。
`<handle>`、`<ref:…>`、`<view>` 和 `<offer:buy>` 代表每次都会变的值；`…` 也表示一个会变的值。

<!-- agent-session: examples/agent-session.json -->

```json
{
  "recorded_with": "mineworld-agent 0.1.0",
  "seat": "ben",
  "steps": [
    {
      "call": {
        "arguments": {},
        "tool": "seats"
      },
      "error": null,
      "shows": {
        "seats": [
          "ana",
          "ben",
          "cai"
        ]
      }
    },
    {
      "call": {
        "arguments": {
          "nickname": "agent",
          "seat": "ben"
        },
        "tool": "join"
      },
      "error": null,
      "shows": {
        "seat": "ben",
        "took_over": "hosted",
        "view.me.place": {
          "labels": [
            {
              "by": null,
              "kind": "label",
              "text": "home"
            }
          ],
          "ref": "…"
        }
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>"
        },
        "tool": "observe"
      },
      "error": null,
      "shows": {
        "view.kind": "view",
        "view.me.holdings": [
          {
            "count": 1,
            "item": "…",
            "name": null
          }
        ],
        "view.me.place": {
          "labels": [
            {
              "by": null,
              "kind": "label",
              "text": "home"
            }
          ],
          "ref": "…"
        },
        "view.me.wallet": 5000
      }
    },
    {
      "call": {
        "arguments": {
          "act": {
            "kind": "walk_to",
            "to": {
              "place": "<ref:shop>"
            }
          },
          "handle": "<handle>"
        },
        "tool": "act"
      },
      "error": null,
      "shows": {
        "outcome.kind": "accepted"
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>",
          "timeout_s": 20,
          "until": {
            "kind": "walk_ended"
          }
        },
        "tool": "wait"
      },
      "error": null,
      "shows": {
        "woke": "walk_ended"
      }
    },
    {
      "call": {
        "arguments": {
          "act": {
            "kind": "walk_to",
            "to": {
              "person": "<ref:Ana>"
            }
          },
          "handle": "<handle>"
        },
        "tool": "act"
      },
      "error": null,
      "shows": {
        "outcome.kind": "accepted"
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>",
          "timeout_s": 20,
          "until": {
            "kind": "walk_ended"
          }
        },
        "tool": "wait"
      },
      "error": null,
      "shows": {
        "woke": "walk_ended"
      }
    },
    {
      "call": {
        "arguments": {
          "act": {
            "kind": "say",
            "to": "<ref:Ana>",
            "words": "Hello, Ana."
          },
          "handle": "<handle>"
        },
        "tool": "act"
      },
      "error": null,
      "shows": {
        "outcome.kind": "accepted"
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>",
          "timeout_s": 20,
          "until": {
            "kind": "spoken_to"
          }
        },
        "tool": "wait"
      },
      "error": null,
      "shows": {
        "woke": "spoken_to"
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>"
        },
        "tool": "observe"
      },
      "error": null,
      "shows": {
        "view.kind": "view",
        "view.me.holdings": [
          {
            "count": 1,
            "item": "…",
            "name": null
          }
        ],
        "view.me.place": {
          "labels": [
            {
              "by": null,
              "kind": "label",
              "text": "shop"
            }
          ],
          "ref": "…"
        },
        "view.me.wallet": 5000
      }
    },
    {
      "call": {
        "arguments": {
          "act": {
            "kind": "attempt",
            "offer": "<offer:buy>",
            "view": "<view>"
          },
          "handle": "<handle>"
        },
        "tool": "act"
      },
      "error": null,
      "shows": {
        "outcome.kind": "accepted"
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>",
          "since_view": "<view>"
        },
        "tool": "observe"
      },
      "error": null,
      "shows": {
        "view.kind": "delta",
        "view.me.holdings": [
          {
            "count": 1,
            "item": "…",
            "name": null
          },
          {
            "count": 1,
            "item": "…",
            "name": null
          }
        ],
        "view.me.place": {
          "labels": [
            {
              "by": null,
              "kind": "label",
              "text": "shop"
            }
          ],
          "ref": "…"
        },
        "view.me.wallet": 4750
      }
    },
    {
      "call": {
        "arguments": {
          "handle": "<handle>"
        },
        "tool": "leave"
      },
      "error": null,
      "shows": {
        "closing": "left"
      }
    }
  ],
  "world": "corner-shop"
}
```

agent 占下 Ben，走进商店，走到 Ana 跟前，打了招呼。它一直等到有人对 Ben 说话，然后接受 `buy` 这个 offer，
再离开：Ben 手里多了一个面包，钱少了 250。这段会话是在[第 05 章](05-one-shop.zh-CN.md)的 Corner Shop 里录的，
那里 Ana 会回话。在你的 Corner Shop 里，按你在[第 07 章](07-rules-of-the-world.zh-CN.md)写的规则，她在店里
不说话，所以你的 agent 等她回话会一直等到超时。

等你自己的 agent 玩过以后，读 Ben 的记录：

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**你应该看到** Ben 的记录里有这次购买。

### Step 14.8 — 不用 MCP：一个 Python 程序，或者任何语言

你不一定要用 MCP。adapter 的工具也是一个 Python 库 `Player`，操作完全相同。

**照着做。** 写 `play.py`：

```python
import asyncio, os
from mineworld_agent import JoinArgs, ObserveArgs, Player, PlayerConfig
from mineworld_sdk import Invite


async def main() -> None:
    config = PlayerConfig(server="ws://127.0.0.1:7878/ws")
    async with Player(config, Invite(os.environ["MINEWORLD_GRANT"])) as player:
        joined = await player.join(JoinArgs(seat="ben", nickname="script"))
        view = (await player.observe(ObserveArgs(handle=joined.handle))).view
        for offer in view.offers:  # what the world says Ben can do here, numbered
            print(offer.n, offer.action_type, offer.target, offer.available)


asyncio.run(main())
```

在 MineWorld 文件夹里用 `uv run python play.py` 运行它，它会打印出 Ben 的 offers。用别的语言的话，直接通过
WebSocket 说世界自己的协议：[`server/PROTOCOL.md`](../../server/PROTOCOL.md)。

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**你应该看到** Ben 的记录里多了程序这次来访的一行，和 agent 那次一样。

### Step 14.9 — 混着玩：朋友也加入

agent 和人可以同时在一个世界里玩，各占一个 seat。

**照着做。** 你的 agent 在玩 Ben 的时候，让一个朋友用 2D client 和 invite 以 Cai 的身份加入，
做法和[第 06 章](06-play-it.zh-CN.md#step-63--以-ben-的身份走一走)一样：

<!-- guide: operator-only -->

```sh
./mineworld-2d --server 127.0.0.1:7878 --invite INVITE --seat cai
```

朋友让 Cai 走进店里，跟 Ben 说话。agent 下一次 `wait` 就会因为 Cai 的话醒来。如果有人在 agent 玩 Ben 的时候
把 Ben 的 seat 抢走了，agent 的下一次调用会被拒绝，错误是 `seat_lost`，原因是 `taken_over`。这样它就知道
自己不再是 Ben 了。

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person cai
biography  cai "Cai Wen" (entity [..]) in corner-shop: [..]
...
```

**你应该看到** Cai 的记录里有他和 Ben 的对话。

### Step 14.10 — 安全

你的 agent 能做什么、不能做什么：

- **用 grant，不用 invite。** grant 只打开它写明的 seat。invite 打开所有 seat。
- **只在你的电脑上。** 除非你另外指定，adapter 和 server 都只在 loopback 上监听。
- **别人的话不是命令。** 别人说的每句话到 agent 那里都标成来自世界的文字（`WorldText`）。那是别人写的，
  从来不是给 agent 的指令。
- **没有房主权力。** 没有任何工具能暂停世界、改变世界速度或者踢掉玩家。agent 只扮演一个人，和你一样。
- **有些动作还够不着。** 需要自由输入的动作，比如邀请别人参加一项活动，现在还没有对应的工具。
- **你要负责。** 你的 agent 在世界里做的事，是以你的名义做的。

**照着做。** 停掉 server，用同样的 grant 参数再启动一次：

<!-- guide: operator-only -->

```console
$ mineworld server corner-shop --packs packs --town --agent ana --grant ben --save saves/agent
...
[mineworld] grant ben [..] — join with: 127.0.0.1:7878 seat=ben invite=[..]
...
```

**你应该看到**一个新的 grant。每次启动都会生成新的码，所以旧的 grant 就失效了。

## 为什么这样设计

agent 和其他玩家一样：它占一个 seat，向世界提出请求，世界说行或不行。所以同一个世界既能给人玩，也能交给
规则或 agent 来玩，世界本身不用改。设计写在 [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#14-controller)、
[`NETWORKING.md`](../NETWORKING.md)，以及这几条决定里：[agent 玩家就是 client](../decisions/ARC-96.md)、
[seat grant](../decisions/ARC-97.md)、[只有一个 `act` 工具](../decisions/ARC-153.md)。每个工具都列在
[工具参考](../reference/agent-tools.md#act)里；错误在 [Errors](../reference/agent-tools.md#errors) 下面。

adapter 还会把这一章和它的参考作为 MCP resource 提供给 agent 本身：
`mineworld://guide/play-as-an-agent` 和 `mineworld://reference/agent-tools`。

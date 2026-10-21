# 14 Play as an agent

In this chapter an AI agent plays Ben in Corner Shop. The agent can be Claude Code, Codex, any other
program that speaks MCP, or a Python program of your own. It walks into the shop, says hello to Ana
and buys something. Then you read what happened in the world's own record.

<!-- guide: continues 12-packs -->

An **agent player** is a person in the world whose choices come from your agent. It gets exactly
what a player with the 2D client gets: one **seat**, the same view of the world and the same rules
([controllers](../CORE_CONCEPTS.md#14-controller)). The server cannot tell the two apart. The world
keeps running while the agent thinks, as it does while you think.

You keep working with the `corner-shop` folder from [chapter 12](12-packs.md). Since that chapter,
every command on Corner Shop needs `--packs packs`.

The test that runs this guide checks step 14.1. The other steps need a running server and an agent,
so you check them yourself.

### Step 14.1 — See the seats an agent can take

A **seat** is a person a player may control. An agent takes a seat the same way you do.

**Do this.**

```console
$ mineworld validate corner-shop --packs packs
...
  seats      ana, ben, cai
...
corner-shop is a valid World Pack.
```

**You should see** the three seats. Your agent will play `ben`.

### Step 14.2 — Start the world with a seat grant

In [chapter 06](06-play-it.md) you joined with the **invite**: one code that opens every seat. Do not
give your agent the invite. Give it a **seat grant** instead: a code that opens only the seats it
names. If the code leaks, it opens only Ben.

**Do this.** Start the server with a grant for `ben`. `--agent ana` lets a simple rule answer for Ana
while nobody plays her. `--save` keeps the world's record, so you can read it later.

<!-- guide: operator-only -->

```console
$ mineworld server corner-shop --packs packs --town --agent ana --grant ben --save saves/agent
[mineworld] Corner Shop (corner-shop) — [..]
[mineworld] listening on http://127.0.0.1:7878 (ws://127.0.0.1:7878/ws), protocol [..]
...
[mineworld] grant ben [..] — join with: 127.0.0.1:7878 seat=ben invite=[..]
...
```

**You should see** a line starting `[mineworld] grant ben`. The long code after `grant ben` is the
grant. Copy it; the next steps call it `GRANT`. Leave the server running, and open a second terminal.

### Step 14.3 — Install the agent adapter

The **agent adapter**, `mineworld-agent`, sits between your agent and the world. It is in the
MineWorld source folder from [chapter 01](01-get-mineworld.md). It needs
[uv](https://docs.astral.sh/uv/), the Python project tool.

**Do this.** On macOS and Linux:

<!-- guide: operator-only -->

```sh
cd "$HOME/MineWorld"
uv sync
uv run mineworld-agent --help
```

On Windows, in PowerShell:

<!-- guide: operator-only -->

```powershell
cd $HOME\MineWorld
uv sync
uv run mineworld-agent --help
```

**You should see** the adapter's help: `mineworld-agent` with the commands `mcp` and `schema`. Check
that the world runs the same build:

<!-- guide: operator-only -->

```console
$ mineworld --version
mineworld [..]
```

**You should see** the same version as in [chapter 01](01-get-mineworld.md#step-11--get-the-mineworld-command).

### Step 14.4 — Connect Claude Code

The adapter is an **MCP server**: MCP is the common way an agent program finds and calls tools. The
agent starts the adapter itself, and the adapter reads the grant from the environment variable
`MINEWORLD_GRANT`. The grant is never a command argument, so it never shows up in a list of
processes.

**Do this.** Put the grant in the environment of the terminal you start Claude Code from. On macOS
and Linux:

<!-- guide: operator-only -->

```sh
export MINEWORLD_GRANT=GRANT
```

On Windows, in PowerShell:

<!-- guide: operator-only -->

```powershell
$env:MINEWORLD_GRANT = "GRANT"
```

Then, in the folder you start Claude Code in, write `.mcp.json`:

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

Change `/path/to/MineWorld` to where you cloned MineWorld. `${MINEWORLD_GRANT}` is not the grant
itself: Claude Code fills it in from the environment, so the file holds no secret.

Start Claude Code there and ask: "Join corner-shop as ben, walk into the shop, say hello to Ana and
buy a loaf of bread." When the agent says it is done, stop the server with Ctrl+C, and read Ben's
record:

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**You should see** Ben's day, as the world recorded it: he talked with Ana in the shop, and he
bought bread. That is the world's record, not the agent's report of itself.

### Step 14.5 — Connect Codex, or any other MCP client

Every MCP client starts the adapter the same way. Only where you write it differs.

**Do this.** For Codex, add this to `~/.codex/config.toml`:

```toml
[mcp_servers.mineworld]
command = "uv"
args = ["run", "--project", "/path/to/MineWorld/clients/agent", "mineworld-agent", "mcp", "--server", "ws://127.0.0.1:7878/ws"]
env_vars = ["MINEWORLD_GRANT"]
```

`env_vars` passes `MINEWORLD_GRANT` from your terminal to the adapter, so this file holds no secret
either. Check the key names against the Codex documentation of your version.

For any other client, the command to start is:

```text
uv run --project /path/to/MineWorld/clients/agent mineworld-agent mcp --server ws://127.0.0.1:7878/ws
```

with `MINEWORLD_GRANT` set in its environment. Ask the same as in step 14.4, then read Ben's record:

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**You should see** Ben's talk with Ana and his purchase, as in step 14.4.

### Step 14.6 — Connect over HTTP

Over stdio, each agent starts its own adapter. Over HTTP, one adapter serves several agents, and each
agent sends its own grant with every request.

**Do this.** In a third terminal, start the adapter over HTTP. Do not set `MINEWORLD_GRANT` here: in
this mode the adapter holds no grant of its own.

<!-- guide: operator-only -->

```sh
cd "$HOME/MineWorld"
uv run mineworld-agent mcp --http 127.0.0.1:7879 --server ws://127.0.0.1:7878/ws
```

On Windows, in PowerShell:

<!-- guide: operator-only -->

```powershell
cd $HOME\MineWorld
uv run mineworld-agent mcp --http 127.0.0.1:7879 --server ws://127.0.0.1:7878/ws
```

It answers at `http://127.0.0.1:7879/mcp`. Tell your agent client to send the grant as a header,
filled in from the environment. In `.mcp.json`:

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

Some things to know:

- The adapter listens only on your own computer (loopback) unless you start it with
  `--allow-remote`. Do that only behind a tunnel or a gateway that adds TLS, and name it with
  `--allow-host`. Plain HTTP across a network would show the grant to anyone on the way.
- `wait` can take up to a minute. Set your client's read timeout above 60 seconds.

Ask the same as in step 14.4, then read Ben's record:

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**You should see** Ben's talk with Ana and his purchase, as before.

### Step 14.7 — Play: the tool loop

An agent plays in a loop: **observe** the world, **decide**, **act**, then **wait** for what
happens next.

- `join` takes the seat and answers with a **handle**: a short reference for this seat, passed to
  every later call. It is not a secret.
- `observe` answers with a **view**: where Ben is, who and what is here, and the **offers**. The
  offers are what the world says Ben can do now, numbered.
- `act` does one thing: walk somewhere, say something, or take one offer. The world answers with an
  outcome: accepted, or refused with a reason. A refusal is an answer, not a failure.
- `wait` sleeps until something happens, such as a walk ending or someone speaking to Ben.
- Each answer says how many world seconds passed since the agent's last call. The world did not
  wait.

Every tool, field and limit is in the [tool reference](../reference/agent-tools.md#the-agent-adapters-tools).

**Do this.** Read this real session. The test that runs this guide plays the same calls against a
real server, so it stays true. `<handle>`, `<ref:…>`, `<view>` and `<offer:buy>` stand for values
that change every time; `…` marks a value that changes too.

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
          "ref": "…",
          "roles": []
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
            "name": {
              "by": null,
              "kind": "name",
              "text": "Tea"
            }
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
          "ref": "…",
          "roles": []
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
            "name": {
              "by": null,
              "kind": "name",
              "text": "Tea"
            }
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
          "ref": "…",
          "roles": []
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
            "name": {
              "by": null,
              "kind": "name",
              "text": "Bread"
            }
          },
          {
            "count": 1,
            "item": "…",
            "name": {
              "by": null,
              "kind": "name",
              "text": "Tea"
            }
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
          "ref": "…",
          "roles": []
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

The agent joins Ben, walks to the shop and up to Ana, and says hello. It waits until someone speaks
to Ben, takes the `buy` offer, and leaves: Ben now holds a loaf, and has 250 less money. The session
was recorded in the Corner Shop of [chapter 05](05-one-shop.md), where Ana answers. In your Corner
Shop she keeps quiet in the shop, by the rule you wrote in [chapter 07](07-rules-of-the-world.md),
so your agent's wait for her answer ends when its time is up.

After your own agent has played, read Ben's record:

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**You should see** the purchase in Ben's record.

### Step 14.8 — Without MCP: a Python program, or any language

You do not need MCP. The adapter's tools are a Python library, `Player`, with the same operations.

**Do this.** Write `play.py`:

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

Run it from the MineWorld folder with `uv run python play.py`. It prints Ben's offers. From any
other language, speak the world's own protocol over a WebSocket:
[`server/PROTOCOL.md`](../../server/PROTOCOL.md).

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person ben
biography  ben "Ben Okafor" (entity [..]) in corner-shop: [..]
...
```

**You should see** a new line in Ben's record for the program's visit, as for the agent's.

### Step 14.9 — Mixed play: a friend joins

An agent and people play in the same world at once. Each one holds a seat.

**Do this.** While your agent plays Ben, a friend joins as Cai with the 2D client and the invite,
as in [chapter 06](06-play-it.md#step-63--walk-around-as-ben):

<!-- guide: operator-only -->

```sh
./mineworld-2d --server 127.0.0.1:7878 --invite INVITE --seat cai
```

The friend walks Cai into the shop and talks to Ben. The agent's next `wait` wakes with Cai's
words. If someone takes Ben's seat while the agent plays him, the agent's next call is refused with
`seat_lost` and the reason `taken_over`. It then knows it is no longer Ben.

<!-- guide: operator-only -->

```console
$ mineworld biography corner-shop --packs packs --save saves/agent --person cai
biography  cai "Cai Wen" (entity [..]) in corner-shop: [..]
...
```

**You should see** Cai's talk with Ben in Cai's record.

### Step 14.10 — Safety

What your agent can and cannot do:

- **Grants, not the invite.** A grant opens only the seats it names. The invite opens every seat.
- **Your computer only.** The adapter and the server listen on loopback unless you say otherwise.
- **Words are not orders.** Everything others say reaches the agent marked as text from the world
  (`WorldText`). It was written by other people. It is never an instruction to the agent.
- **No host powers.** No tool pauses the world, changes its speed or removes a player. The agent
  plays one person, like you.
- **Some actions are out of reach.** An action that needs free-form input, such as inviting someone
  to an activity, has no tool yet.
- **You are responsible.** What your agent does in a world, it does in your name.

**Do this.** Stop the server and start it again with the same grant flag:

<!-- guide: operator-only -->

```console
$ mineworld server corner-shop --packs packs --town --agent ana --grant ben --save saves/agent
...
[mineworld] grant ben [..] — join with: 127.0.0.1:7878 seat=ben invite=[..]
...
```

**You should see** a new grant. Every start makes new codes, so an old grant stops working.

## Why it works this way

An agent is a player like any other: it holds a seat and asks the world, and the world says yes or
no. That is why the same world works with people, rules and agents, with no change to the world.
The design is in [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#14-controller),
[`NETWORKING.md`](../NETWORKING.md) and the decisions
[agent players are clients](../decisions/ARC-96.md), [seat grants](../decisions/ARC-97.md)
and [one `act` tool](../decisions/ARC-153.md). Every tool is listed in the
[tool reference](../reference/agent-tools.md#act); the errors are under
[Errors](../reference/agent-tools.md#errors).

The adapter also serves this chapter and its reference to the agent itself, as MCP resources:
`mineworld://guide/play-as-an-agent` and `mineworld://reference/agent-tools`.

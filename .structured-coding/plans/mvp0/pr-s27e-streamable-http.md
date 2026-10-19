# PR S27-e — Streamable HTTP, and several agents in one adapter

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        revision 0 (2026-10-15), FROZEN 2026-10-18
Author:                 S27-e / S27-f PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s27ef, branch docs/pr-design-s27ef,
                        planning base origin/main @ 5cbb469; designed against the S27-d implementation branch
                        mvp0/pr-s27d @ c4ac3a4 (C0–C4 committed, C5–C7 pending, not pushed)
Approved by / evidence: §14 ruled 2026-10-18 by the operator in the primary session: QS27e-1, QS27e-2 and
                        QS27e-3 as recommended
Implementation base:    main (origin/main at C0, with S27-d merged); C0 records the commit
Start condition:        step-27 §11.4: S27-d merged (met: #239, merge 42bc3ceb), this design frozen (met:
                        2026-10-18), the primary session's dispatch
Execution contract:     §12 (filled; endpoint authority included)
Implementer:            Haiku-class, Opus-class review (step-27 §11.4, frozen: "Opus-class review because it is a
                        security surface"); the plan below is written to the numbered step for that reason
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Step:** [`step-27-agent-players.md`](step-27-agent-players.md) (STEP DESIGN FROZEN
2026-10-14, QS27-1 … QS27-10 ruled) §§3 (INV-S27-1 … 13), 5.4, 6.2, 6.3, 6.5, 7.1, 9.1, 9.2, 9.3, 11.2 "PR
S27-e", 11.4, 16 · **Parent:** [`overall.md`](overall.md) · **Consumes:**
[`pr-s27d-mcp-stdio.md`](pr-s27d-mcp-stdio.md) (FROZEN 2026-10-15; its requirement R-S27e-d-1),
[`pr-s27c-player-library.md`](pr-s27c-player-library.md) (the `Player`, merged as #204),
[`pr-s27a-seat-grants-budget.md`](pr-s27a-seat-grants-budget.md) (grants and the budget, merged as #205) ·
**Followed by:** S27-f (guide chapter 14 documents this transport), S27-g (an HTTP live check, operator) ·
**Binding parents:** `CLAUDE.md` §§1.1, 2, 4 (items 3, 7, 8, 9, 10, 13, 16); `docs/NETWORKING.md` §§7, 9;
`server/PROTOCOL.md` §§1, 2, 4; `docs/REUSE_POLICY.md`; `docs/DECISIONS.md` `ARC-96`, `ARC-97`, `DEP-56`.

**The operator's principles, binding here.** Agents are first-class players: an agent reaching the adapter over
HTTP gets exactly the thirteen tools, schemas and information an agent over stdio gets, and the same rules — the
transport adds no tool, no field and no right. All platforms: every real-binary test of this PR runs on Linux,
macOS and Windows. Before release, everything is done: nothing below is deferred to "after MVP-0" except what the
step's rulings put there (QS27-5: no public-internet endpoint in MVP-0). Anything that costs money is an operator
question: nothing here does (no model, no paid service; CI minutes only).

**Decision records.** No new id. `DEP-56` (assigned to S27-d) receives one dated note in C0 recording which parts
of the `mcp` SDK this PR adopts and which it declines (§4). If the primary rules that the note must be its own
record, the placeholder is `DEP-S27e-a` (§14, QS27e-3).

No model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-27 §11.2).** "`--http`, `--allow-remote`, `--allow-origin`; bearer handling and handle binding; the
  500 ms failure delay; multi-handle hosting; tests with the SDK's Streamable HTTP client: two agents on two seats
  in one adapter plus a human SDK client, all in the café, each seeing only its own view."
- **CP-e.** "§9.2 over HTTP on three OSes; a request with a foreign `Origin` is 403; a handle used with another
  bearer is `unknown_handle`; binding `0.0.0.0` without `--allow-remote` refuses to start."
- **Adversarial.** "Review plants 'skip Origin check when the header is absent and the Host is not loopback'; the
  DNS-rebinding test fails. Opus-class review because it is a security surface."
- **Step §6.2.** Loopback only by default; a non-loopback `--http` address is refused unless `--allow-remote`; the
  startup line then says the endpoint is plain HTTP and that TLS belongs to a gateway or tunnel. `Origin`
  validation on every request: a request whose `Origin` is not in `--allow-origin` (default: none) is refused 403.
  Every request carries `Authorization: Bearer <credential>` (a grant token or the invite); the adapter forwards it
  as the `join`'s `invite` and holds no credential of its own. A handle minted under one bearer cannot be used
  under another (`unknown_handle`). Failures are answered no sooner than 500 ms after arrival. Each `join` opens
  its own WebSocket; handles are independent and bound to the bearer that minted them.
- **Step §6.3.** The credential is "`MINEWORLD_GRANT` (stdio) or the bearer (HTTP)"; it is never a tool argument,
  result, resource, log line or error (INV-S27-11).
- **QS27-5 (ruled).** Loopback only in MVP-0; `--allow-remote` exists, and the guide documents it only behind a
  TLS tunnel or gateway; a public-internet agent endpoint is MVP-1 with durable identity.
- **R-S27e-d-1 (from S27-d §10).** "Streamable HTTP reuses `build_server` unchanged; a handle is bound to the bearer
  that minted it (step §6.2); the stdio play-test's steps rerun over HTTP." (`build_server` became
  `Adapter.server` in S27-d's C3; §3.3 F-S27e-1 reads the requirement against the code.)
- **R-LOAD-1.** Locally only the commit's targeted test files plus ruff, ruff format and pyright; never a workspace or
  whole-member suite; CI on the exact head is the gate.

---

## 1. Goal

Serve the same adapter over MCP **Streamable HTTP**, so that an agent that is not a child process of the adapter —
another program on the same machine, a container, an IDE that speaks only HTTP — plays a seat exactly as an stdio
agent does. One adapter process serves several agents at once: each request names its credential in its bearer;
each credential gets its own `Player`; a handle is usable only with the bearer that minted it. The endpoint is
loopback-only unless the operator says otherwise, checks `Host` and `Origin` against DNS rebinding, refuses a
missing or malformed bearer after 500 ms, and holds no credential of its own. Prove it by rerunning S27-d's
play-test over HTTP against the real server on three OSes, plus a mixed session — two agents on two seats through
one adapter and a human SDK client, all in the café, each seeing only what its own Person perceives.

---

## 2. Scope

### 2.1 In scope

1. **`clients/agent/src/mineworld_agent/pool.py`** (new; no `mcp` import): `Credentials`, a bounded map from a
   credential to its own `Player` (§5.3).
2. **`clients/agent/src/mineworld_agent/mcp_server.py`** (still the only module importing `mcp`, I-d-1):
   (a) the tool, resource and prompt handlers resolve their `Player` per request through a `PlayerSource` instead
   of closing over one `Player` (§5.2, F-S27e-1); `Adapter.server(player, config)` keeps its signature and
   behaviour for stdio; (b) `http_app(...)` and `serve_http(...)`: the SDK's Streamable HTTP transport and
   transport-security settings, wrapped in this module's `BearerGate` (§5.4–§5.6).
3. **`clients/agent/src/mineworld_agent/player.py`**: one additive read-only method, `Player.active() -> int`
   (handles not yet ended), used by the pool's eviction (§5.3). No operation, argument, result or cap changes.
4. **`clients/agent/src/mineworld_agent/__main__.py`**: `mcp --http HOST:PORT [--allow-remote] [--allow-origin
   ORIGIN]… [--allow-host HOST[:PORT]]… [--max-credentials N]` (§5.7); in HTTP mode the environment credentials are
   refused.
5. **Tests** (§7, §8): pool unit tests; gate and security tests against the in-process ASGI app; the real-binary
   HTTP play-test, the mixed-play test, and the HTTP security tests against the running adapter.
6. `docs/DECISIONS.md`: a dated note under `DEP-56` (§4). `clients/agent/README.md`: the HTTP command (three lines).

### 2.2 Not in scope

TLS termination (a gateway's job, step §6.2); a public-internet endpoint or durable identity (QS27-5, MVP-1); any
new tool, resource, prompt, field or error code (the HTTP agent sees S27-d's published surface byte for byte);
OAuth (§4.2); any change under `server/`, `contracts/`, `kernel/`, `systems/`, `worlds/`, `cognition/`, `sdk/`; the
guide chapter (S27-f); a watch role (QS27-6); host powers (QS27-4).

### 2.3 Dependencies

S27-d merged — the hard start condition. No new dependency: `starlette`, `uvicorn` and the SDK's HTTP server and
client already come with `mcp` (S27-d X-1); `pyproject.toml` gains nothing (I-e-9).

---

## 3. Audit

### 3.1 Source anchors (S27-d at `c4ac3a4` on `mvp0/pr-s27d`; S27-c/S27-a as merged on `origin/main @ 5cbb469`)

| # | Fact | Where |
| --- | --- | --- |
| E-1 | `Adapter.server(player, config)` builds the low-level `Server`; its `handlers` dict binds the thirteen tool names to **one** `Player`'s bound methods, so every request is answered by that `Player` | `clients/agent/src/mineworld_agent/mcp_server.py:297-424` (dict at `:299-313`) |
| E-2 | Resource reads go through `Adapter._read(player, uri)` with the same single `Player` (`world/status` → `player.seats`, the inbox template → `player.inbox`) | `mcp_server.py:375-385`, `:426-446` |
| E-3 | The subscription bus is one `InMemorySubscriptionBus` per `Adapter`; `Adapter.on_inbox(handle)` publishes a content-free `ResourceUpdated` for `mineworld://seat/{handle}/inbox` | `mcp_server.py:279-295` |
| E-4 | `serve_stdio(server)` is the only transport; `__main__._serve` builds one `Player(config, Invite(secret), on_inbox=adapter.on_inbox)` from the environment | `mcp_server.py:456-460`; `__main__.py:72-101` |
| E-5 | `__main__` reads `MINEWORLD_GRANT`, else `MINEWORLD_INVITE`; both → exit 2; neither → exit 2; `PlayerConfig` refusals print the field location only | `__main__.py:23-94` |
| E-6 | `Player(config, credential: Invite, *, connector, wall, sleep, on_inbox)` holds one credential for its life; it is used for every `join` (`ResumingSeat.open(..., invite=self._credential, ...)`) | `clients/agent/src/mineworld_agent/player.py:177-200`, `:240-249` |
| E-7 | Handles are `secrets.token_hex(16)` (128 bits); `self._handles: dict[str, Handle]`; an unknown or ended handle answers `PlayerRefusal("unknown_handle")` / `SeatLostRefusal` from `_seated` | `player.py:271`, `:194`, `:607-615` |
| E-8 | `state_dir` files are keyed by **seat** (`self._states.path(key)` with `key = EntityKey(args.seat)`), and a saved resume secret is used on the next `join` of that seat | `player.py:226-231`; `clients/agent/src/mineworld_agent/state.py` |
| E-9 | Server join checks: 2 credential (`unauthorized` after a fixed 500 ms delay, one guess per connection), 4a `not_granted`, 6 control; "resuming one's own held seat with `resume` is always allowed" | `server/PROTOCOL.md:128-151`, `:172-195`; `server/src/admission.rs:36-41` |
| E-10 | Credentials are 8–128 bytes of printable ASCII without whitespace (the invite rule); a grant token is generated (hex) | `server/src/admission.rs:61-106`; `tools/cli/src/invite.rs:18-23` |
| E-11 | The SDK (`mcp` 2.2.0 as locked by S27-d C1) gives the low-level `Server.streamable_http_app(streamable_http_path, json_response, stateless_http, max_request_body_size, transport_security, host, auth, token_verifier, …) -> Starlette`; it auto-enables DNS-rebinding protection only when `transport_security is None` and `host` is loopback | `.venv/…/mcp/server/lowlevel/server.py:721-860` (read in the S27-d worktree, 2026-10-15) |
| E-12 | `TransportSecuritySettings(enable_dns_rebinding_protection, allowed_hosts, allowed_origins)`; the middleware answers a bad `Host` **421**, a bad `Origin` **403**, a POST without `application/json` **400**; an **absent `Origin` passes**; patterns `host:*` match any port | `.venv/…/mcp/server/transport_security.py:20-123` |
| E-13 | `StreamableHTTPSessionManager(stateless=True)` creates a fresh transport per request and runs the server for it in the manager's task group; requests whose `Mcp-Protocol-Version` is not a handshake version (2026-07-28) go to `handle_modern_request` | `.venv/…/mcp/server/streamable_http_manager.py:176-259` |
| E-14 | The SDK's bearer support is OAuth-shaped: `AuthSettings` requires `issuer_url` and `resource_server_url`; `BearerAuthBackend` + `RequireAuthMiddleware` are added only when `auth` is given, and answer 401 at once (no delay) | `.venv/…/mcp/server/auth/settings.py:20-66`; `auth/middleware/bearer_auth.py:44-140`; `lowlevel/server.py:764-822` |
| E-15 | `ServerRequestContext` carries `request: RequestT | None` ("the HTTP request" a transport attached); the modern path builds `TransportContext(..., headers=request.headers)` | `.venv/…/mcp/server/context.py:31-49`; `_streamable_http_modern.py:295`, `:449` |
| E-16 | The SDK's HTTP client: `mcp.client.streamable_http.streamable_http_client(...)` | `.venv/…/mcp/client/streamable_http.py:681` |
| E-17 | Real-server test support: `Server(world, *flags)` (stdout drained, `grants()` parses the `[mineworld] grant` lines), `market_town(*flags)`, `perceived_export`, `biography`, `seated(server, seat, take_over=)` (a plain-SDK client), `Transcript` | `clients/agent/tests/realserver.py:55-291` (S27-d branch) |
| E-18 | `market-town` starts `visitor`, `wanderer`, `alice` and `bob` in the café | `worlds/market-town/people/{visitor,wanderer,alice,bob}.yaml` (`location.place: cafe`) |
| E-19 | CI: `PYTHON_MEMBERS` includes `clients/agent`; the `python` layer runs `pytest clients/agent` on Linux, macOS and Windows after `cargo build -p mineworld-cli`; `python-smoke` deselects `real_server` | `scripts/ci_layer.py:185-222` |

### 3.2 External facts, to verify at C1 (marked **verify**; each has a prototype test)

| # | Fact to confirm | Prototype |
| --- | --- | --- |
| V-1 | Inside a low-level `on_call_tool` handler served over stateless Streamable HTTP, the request's headers are reachable — through `ctx.request.headers` (a Starlette `Request`), or through the `TransportContext` headers (E-15) — for both a 2025-11-25 (handshake) client and a 2026-07-28 client | P-e-1 |
| V-2 | `streamable_http_client` can send a fixed `Authorization` header on every request | P-e-2 |
| V-3 | `subscriptions/listen` works over stateless Streamable HTTP (with `json_response` off, i.e. SSE responses) | P-e-3 |
| V-4 | `uvicorn.Server(config).serve(sockets=[sock])` serves a pre-bound socket on all three OSes, so the adapter can bind port 0 and print the real port | P-e-4 |

A failure of P-e-1 or P-e-2 is a material stop (§12). A failure of P-e-3 is bounded: the inbox subscription is
then documented as stdio-only and `wait` stays the portable path (step §5.4 already says a client that does not
listen loses nothing); recorded in §15 and in S27-f's chapter.

### 3.3 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S27e-1** | R-S27e-d-1 asks that HTTP reuse the server builder unchanged, but that builder binds one `Player` (E-1, E-2), while HTTP must answer each request with the `Player` of that request's bearer. | The handlers look the `Player` up per request through a `PlayerSource` (§5.2). The tool table, schemas, descriptions, dispatch, error mapping, caps, resources, prompt and subscription stay exactly S27-d's — one copy, both transports. `Adapter.server(player, config)` keeps its signature for stdio (it passes a constant source). The golden tool list is unchanged (AC-e-10). This is the requirement's intent (no second server); the literal word "unchanged" is read as "no second copy, no behaviour change", and recorded. |
| **F-S27e-2** | The SDK's bearer authentication (E-14) is an OAuth 2.1 resource server: it needs an issuer URL and publishes protected-resource metadata pointing at it. MineWorld's credentials are opaque host-generated tokens with no issuer; the only authority that can check one is the world server at `join` (E-9). Its 401 is also immediate, not delayed. | Decline the SDK's OAuth layer; adopt its transport and transport security. A ~40-line ASGI `BearerGate` in `mcp_server.py` checks the header's form (E-10) and answers a missing or malformed bearer 401 after 500 ms (§5.4). Recorded under `DEP-56` (§4). |
| **F-S27e-3** | `state_dir` files are keyed by seat only (E-8). Two bearers sharing one adapter and one `--state-dir` would share a seat's resume file, so bearer B — holding a different credential that also names that seat — could resume the seat A's connection held, since resuming one's own held seat is allowed with any admitting credential (E-9). | Each credential's `Player` gets its own subdirectory, `STATE_DIR/<k>`, where `k` is the first 32 hex digits of SHA-256 over a fixed label and the credential (§5.3). One credential's `Player` cannot read another's files. AC-e-7 tests it. |
| **F-S27e-4** | The SDK's middleware lets an absent `Origin` through (E-12). Non-browser MCP clients send none, so that is right; DNS-rebinding defence then rests on the `Host` check, so `allowed_hosts` must never be a wildcard host. | `allowed_hosts` is exactly the loopback names with the bound port (`127.0.0.1:P`, `localhost:P`, `[::1]:P`), plus the bound `HOST:P` and each `--allow-host` under `--allow-remote` (§5.5). The step's planted mutation (M-e-1) and AC-e-4 test the absent-`Origin` case. |
| **F-S27e-5** | The step lists `--http`, `--allow-remote`, `--allow-origin`. Under `--allow-remote` the `Host` a gateway or LAN client sends is a name the adapter cannot guess, and §5.5's `Host` check would refuse it. | `--allow-host HOST[:PORT]` (repeatable; accepted only with `--allow-remote`). This adds one flag to the step's list: **QS27e-1**. |
| **F-S27e-6** | One `Player` per credential, created on demand, is unbounded if a client sends many distinct bearers (each a well-formed but unknown token creates a `Player` before the server refuses its `join`). | `--max-credentials N` (default 8, at most 64). A `Player` that holds no active handle is evicted first; when every slot holds an active handle, a request with a new bearer is answered **503** with `Retry-After: 5` before any MCP processing (§5.3). A `Player` is created only for a request that reaches a tool or resource handler. |
| **F-S27e-7** | In HTTP mode, an environment credential would let anyone who reaches the port play with it — the opposite of "holds no credential of its own". | `--http` with `MINEWORLD_GRANT` or `MINEWORLD_INVITE` set exits 2: "HTTP mode takes each request's credential from its bearer; unset MINEWORLD_GRANT and MINEWORLD_INVITE". |
| **F-S27e-8** | uvicorn logs an access line per request (to stdout by default) and the SDK logs `Invalid Origin header: <origin>` warnings. Neither carries the `Authorization` header, but stdout must stay quiet and logs are a secret-leak surface. | `access_log=False`, `log_level="warning"`, uvicorn's own log config disabled (`log_config=None`); the adapter's diagnostics stay on stderr, codes only. AC-e-6 scans stdout and stderr for the canary bearers. |

---

## 4. Reuse, both ways (`DEP-56` dated note, C0)

### 4.1 What is adopted from the SDK

The Streamable HTTP server transport (`Server.streamable_http_app`, stateless, E-11, E-13); its transport-security
middleware for `Host` and `Origin` (E-12); its request-body limit; its HTTP client for the tests (E-16); `uvicorn`
(already in the graph through `mcp`) as the ASGI server. No second web stack; no new dependency.

### 4.2 What is declined, and why

| Part | Verdict | Why |
| --- | --- | --- |
| The SDK's OAuth resource-server auth (`AuthSettings`, `TokenVerifier`, `RequireAuthMiddleware`) | declined | It needs an issuer and publishes metadata for one (E-14); MineWorld's credentials have no issuer, and only the world server can judge one. Forcing them through OAuth would invent an authorization server that does not exist. Its 401 is immediate; the step asks for 500 ms. |
| Stateful Streamable HTTP sessions (`Mcp-Session-Id`) | declined | The surface is handle-based by design (step §4.2: MCP 2026-07-28 is stateless; handles are tool arguments); a session would add a second identity beside the bearer. |
| SSE resumability (event store) | declined | Not in 2026-07-28 (step §17); `events` with a cursor is the adapter's own resumable read. |
| Our own HTTP JSON-RPC | rejected | step-27 §4.2 option H. |

### 4.3 Not forcing it

The SDK moves requests; `mcp_server.py` decides which `Player` answers, and the `Player` decides everything about
the seat. `BearerGate` is the only HTTP code MineWorld writes, and it holds no game logic.

---

## 5. Design

### 5.1 Module layout

```text
mineworld_agent/pool.py         Credentials: credential -> Player, bounded; per-credential state subdirectory;
                                 eviction of Players with no active handle. No `mcp` import.
mineworld_agent/mcp_server.py   + PlayerSource; handlers resolve the Player per request (stdio: a constant);
                                 + BearerGate (ASGI); + http_app(credentials, adapter, settings); + serve_http(...)
mineworld_agent/player.py       + Player.active() -> int
mineworld_agent/__main__.py     + --http, --allow-remote, --allow-origin, --allow-host, --max-credentials
```

### 5.2 One server, a `Player` per request (F-S27e-1)

- `type PlayerSource = Callable[[ServerRequestContext[Any]], Awaitable[Player]]`, defined in `mcp_server.py`.
- `Adapter.serve_players(source: PlayerSource, config: PlayerConfig) -> Server[Any]` holds the body S27-d's
  `Adapter.server` has today, with one change: `call_tool` and `read_resource` first do
  `player = await source(ctx)` and then call `getattr(player, operation.name)(args)` (the operation names equal the
  `Player` method names, E-1) and `self._read(player, uri)`. A `source` that raises `PlayerRefusal` is mapped by
  the same `failure_of` (S27-d §5.4); `list_tools`, `list_resources`, `list_templates`, `list_prompts` and
  `get_prompt` do not call `source` (they answer the same for everyone, S27-d X-4).
- `Adapter.server(player, config)` becomes `return self.serve_players(_constant(player), config)`; stdio is unchanged
  in signature and behaviour.
- `_handlers` dict is removed; the tool order and the closed list stay `OPERATIONS` (S27-d I-d-4).

### 5.3 `Credentials` — the pool (`pool.py`)

```text
class Credentials:
    __init__(config: PlayerConfig, *, limit: int, on_inbox: Callable[[str], None],
             make: Callable[[PlayerConfig, Invite, Callable[[str], None]], Player] = <Player>)
    async player_for(credential: Invite) -> Player        raises Full when no slot can be freed
    async close() -> None                                  closes every Player (each handle leaves)
    __repr__ -> "Credentials(players=N, limit=L)"          never a credential, never a key
```

- **Key.** `key = hmac.new(self._salt, credential.encode("utf-8"), "sha256").hexdigest()` with
  `self._salt = secrets.token_bytes(32)` per process; lookups compare keys, never credentials, so no credential is
  ever a dict key or compared byte by byte. The map is `dict[str, _Slot]` with `_Slot(player, last_used)`.
- **State partition (F-S27e-3).** When `config.state_dir` is set, the `Player` for a credential gets
  `config.model_copy(update={"state_dir": config.state_dir / partition(credential)})` with
  `partition(c) = hashlib.sha256(b"mineworld-agent state v1\x00" + c.encode("utf-8")).hexdigest()[:32]` — stable
  across adapter restarts (so a restarted HTTP adapter resumes the seats a credential held, as stdio does), and not
  reversible to the credential. The subdirectory is created with mode 0700 on POSIX by `StateDir` as S27-c does for
  the root (audit `state.py` in C2 and reuse its creation path).
- **Creation and eviction (F-S27e-6).** On a miss: if `len(slots) == limit`, evict the least recently used slot whose
  `player.active() == 0` (await its `close()`); if none, raise `Full`. Then create the `Player` with the shared
  `on_inbox`. `player_for` serialises creation under one `asyncio.Lock`; a hit only updates `last_used`.
- **`Player.active()`** (player.py): `sum(1 for h in self._handles.values() if h.lost is None)`.
- **What the pool never does.** It never validates a credential (the server does at `join`, E-9), never logs one,
  never persists one, and never lets one `Player` see another's handles: a handle minted by `Player` A is simply
  absent from `Player` B's `_handles`, so B answers `unknown_handle` (E-7). That is the step's handle binding, by
  construction.

### 5.4 `BearerGate` (ASGI, in `mcp_server.py`)

Order of checks for every HTTP request to the MCP path, first failure wins:

1. **Transport security** (the SDK's own check): bad `Host` 421, bad `Origin` 403, POST without JSON 400 (E-12).
   `BearerGate` calls the SDK's `TransportSecurityMiddleware(settings).validate_request(request, is_post=…)` first,
   with the same settings the SDK app is built with, so a request the SDK would refuse is refused with the SDK's
   answer and never reaches the bearer logic. (The SDK app repeats the check; the repetition is harmless and keeps
   the SDK's own path intact.)
2. **Bearer form.** Exactly one `Authorization` header, `Bearer <t>` (scheme case-insensitive), `t` 8–128 bytes of
   printable ASCII without whitespace (E-10). Otherwise **401** with `WWW-Authenticate: Bearer` and a JSON body
   `{"error": "unauthorized"}`, sent no sooner than **500 ms** after the request arrived
   (`await asyncio.sleep(max(0.0, 0.5 - elapsed))`, monotonic clock).
3. **Capacity.** `await credentials.admit(t)` reserves the credential's slot under the pool's lock (evicting an
   inactive slot if needed, §5.3) and returns `False` only when every slot holds an active handle; `False` →
   **503**, `Retry-After: 5`, body `{"error": "busy"}`, no delay. Because the slot is reserved here, the handler's
   later `player_for(t)` is always a hit, so a capacity refusal never reaches an agent as a tool error and S27-d's
   closed `ErrorCode` stays unchanged (QS27e-2). A reserved slot whose request never reaches a handler holds a
   `Player` with no handle and is evictable at once.
4. Delegate to the SDK app. The bearer reaches the handler through V-1's path; `source(ctx)` reads it there, never
   from a module global.

The `unknown_handle` tool error over HTTP is **not** delayed: handles are 128-bit (E-7), guessing is infeasible,
and a delay would slow a legitimate agent after `seat_lost`. A `join` with a wrong credential is delayed by the
server already (E-9); the adapter adds nothing to it.

### 5.5 Binding and allowed names

| Mode | `--http` | `allowed_hosts` | `allowed_origins` |
| --- | --- | --- | --- |
| loopback (default) | a loopback address: `127.0.0.1`, `::1`, or `localhost`; port `0` allowed | `127.0.0.1:P`, `localhost:P`, `[::1]:P` (P = the bound port) | exactly the `--allow-origin` values (default none) |
| remote | any address, only with `--allow-remote` | the loopback three, `HOST:P` as bound, and each `--allow-host` (a bare name gets `:P`) | exactly the `--allow-origin` values |

- A non-loopback `--http` without `--allow-remote` exits 2: "--http HOST is not a loopback address; add
  --allow-remote, and put TLS in front of it (a tunnel or a gateway)". `--allow-host` without `--allow-remote`
  exits 2.
- **Startup line** (stderr, one line, after the socket is bound): `mineworld-agent: listening on
  http://HOST:P/mcp (loopback only)` or `… (plain HTTP on a non-loopback address: TLS belongs to a tunnel or a
  gateway in front of it)`. The tests read `P` from this line (V-4).
- `enable_dns_rebinding_protection=True` always; the settings object is built by one function,
  `security_settings(bound, allow_hosts, allow_origins)`, unit-tested in isolation.
- MCP path `/mcp`; `stateless_http=True`; `json_response=False` (SSE responses, so `subscriptions/listen` can
  stream, V-3); `max_request_body_size=65_536` (the largest tool call is a `say` of 480 bytes).

### 5.6 Serving

`serve_http(adapter, credentials, config, bound_socket, settings)`: build `http_app`, then
`uvicorn.Server(uvicorn.Config(app, log_config=None, access_log=False, log_level="warning", lifespan="on"))
.serve(sockets=[bound_socket])`. On shutdown (SIGINT, SIGTERM, or the event loop ending) `await
credentials.close()` — every handle of every credential leaves, so every seat returns to its default at once, as
stdio's EOF does (step §6.1).

### 5.7 The command line

```text
mineworld-agent mcp --server ws://HOST:PORT/ws [--state-dir DIR] [--idle-release S] [--max-result-bytes N] [--inbox N]
                    [--http HOST:PORT [--allow-remote] [--allow-origin ORIGIN]... [--allow-host HOST[:PORT]]...
                                      [--max-credentials N]]
```

- Without `--http`: stdio, exactly S27-d (environment credential required).
- With `--http`: the environment credentials must be unset (F-S27e-7); `--max-credentials` 1–64, default 8; an
  `--allow-origin` value must be an origin (`scheme://host[:port]`, no path), else exit 2 naming the flag.
- `--allow-remote`, `--allow-origin`, `--allow-host`, `--max-credentials` without `--http` exit 2 naming the flag.
- Every refusal names a flag or a field, never a value (S27-d §5.8 rule).

---

## 6. Invariants

| Id | Invariant | Step | Checked by |
| --- | --- | --- | --- |
| I-e-1 | Over HTTP the tool list, every schema, every description, the resources and the prompt are byte-identical to stdio's (the golden) | INV-S27-3, first-class | AC-e-10 |
| I-e-2 | A handle is usable only with the bearer that minted it; any other bearer gets `unknown_handle` | step §6.2 | AC-e-3 |
| I-e-3 | The HTTP adapter holds no credential of its own: no environment credential is accepted; every `join` carries the request's bearer, unchanged | step §6.2, §6.3 | AC-e-5, `test_cli_http.py` |
| I-e-4 | A missing or malformed bearer is refused 401, no sooner than 500 ms after arrival | step §6.2 | AC-e-5 |
| I-e-5 | `Host` must be an allowed name (§5.5) and `Origin`, when present, must be in `--allow-origin`; an absent `Origin` never bypasses the `Host` check | step §6.2, §9.3 | AC-e-4, M-e-1 |
| I-e-6 | A non-loopback bind needs `--allow-remote`; the startup line says plain HTTP | QS27-5 | AC-e-8 |
| I-e-7 | One credential's `Player` never reads another's state files, handles or inbox | INV-S27-2, F-S27e-3 | AC-e-7, AC-e-2 |
| I-e-8 | No credential appears in any HTTP response, MCP message, stdout or stderr line, or state file other than the resume file | INV-S27-11 | AC-e-6 |
| I-e-9 | No new dependency; `mcp` is still imported only by `mcp_server.py` | INV-S27-9, I-d-1 | S27-d's `test_mcp_confined.py`, unchanged and green |
| I-e-10 | The pool is bounded: at most `--max-credentials` `Player`s; beyond, 503 before MCP processing | F-S27e-6 | AC-e-9 |
| I-e-11 | The frames the server receives from an HTTP agent are those an stdio agent sends: `join`, `submit`, `leave`, revision-2 fields only | INV-S27-4 | AC-e-1 (frame recording, S27-d AC-d-2's check reused) |

---

## 7. Test ownership

| Layer | Owns | Where (`clients/agent/tests/`) |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict over `clients/agent` | CI `fast`, `python` |
| Unit | `test_pool.py` (keying, partition stability and distinctness, eviction order, `Full`, `admit` reservation, `repr` without secrets; with a `make` double — no socket); `test_http_security.py` (`security_settings` table of §5.5; `BearerGate` against the in-process ASGI app with `httpx2.ASGITransport` or Starlette's `TestClient` — whichever C1 finds in the lock — 401 forms, the 500 ms floor measured on a monotonic clock, 503, header-only refusals before the bearer is read); `test_cli_http.py` (§5.7's refusals, F-S27e-7) | as named |
| Unit / in-memory MCP | `test_mcp_sources.py`: `serve_players` with two `Player` doubles answers each request from the source's choice; list endpoints never call the source; stdio's `Adapter.server` still passes S27-d's `test_mcp_tools.py`, `test_mcp_errors.py`, `test_mcp_resources.py` unchanged | as named |
| Real lifecycle (`real_server`, three OSes, the SDK's Streamable HTTP client) | `test_play_http.py` (AC-e-1), `test_mixed_http.py` (AC-e-2, AC-e-3), `test_http_endpoint.py` (AC-e-4 … AC-e-9 against the running adapter with raw `http.client` requests) | as named |
| Gate 1 (real LLM) | NOT REQUIRED: no model; S27-g's operator check covers a real agent over HTTP | — |
| CI impact | `clients/agent/**` changes select the `python` and `python-smoke` layers; no Rust change; full-suite fallback not triggered | `scripts/ci_layer.py`, `ci_changes.py` (unchanged) |

No test calls a model or reaches beyond loopback.

---

## 8. Acceptance criteria (fixed before measuring)

The server for every real-binary file: S27-d's fixture (`market_town("--town", "--agent", "alice", "--save", DIR,
"--grant", "visitor", "--grant", "visitor:take-over", "--grant", "wanderer", "--grant", "alice", "--hold", "10",
"--time-scale", "1", "--listen", "127.0.0.1:0")`), grant tokens from `Server.grants()` (E-17). The adapter: `python
-m mineworld_agent mcp --http 127.0.0.1:0 --server <relay url> --state-dir <tmp>` started by the test with **no**
credential in its environment; the port read from the startup line; the S27-d WebSocket relay (`mcp_relay.py`)
between the adapter and the server, recording frames. The agents: the SDK's `streamable_http_client` with
`Authorization: Bearer <grant>`.

| Id | Criterion | Step ref |
| --- | --- | --- |
| AC-e-1 | **The play-test over HTTP.** S27-d's AC-d-1 steps 1–12, the same script function (refactored in C4 to take a session factory, so stdio and HTTP run one script), against an HTTP adapter: join with the visitor grant as bearer; walk, talk, buy; the relay cut and resume; the plain client's takeover → `seat_lost`/`taken_over`; the take-back by a second HTTP agent whose bearer is the `visitor:take-over` grant **in the same adapter process** → `took_over: "connection"`; alice; leave; every result ≤ 8 KiB. The relay's recorded frames pass AC-d-2's key-path subset check | §9.2, CP-e, AC-S27-5 |
| AC-e-2 | **Two agents and a human, one adapter, all in the café.** Agent A (bearer: visitor grant) joins `visitor`; agent B (bearer: wanderer grant) joins `wanderer`, both through one adapter; a plain-SDK client joins `bob` with the invite (all start in the café, E-18). Each `observe` lists the other two among `here.people`; the human says a line to A only. Oracle: for 20 wall seconds, every `EventLine` A receives corresponds to a fact in `mineworld perceived --person visitor` and every one B receives to a fact in `--person wanderer`; the line said to A reaches B only if `perceived --person wanderer` holds it. No reference minted for A (`E…`) is accepted by B's handle (`unknown_ref`) | step §7.1, AC-S27-2 |
| AC-e-3 | **Handle binding.** A's handle used in any tool call with B's bearer → `ToolError` `unknown_handle`; with A's bearer it works again afterwards (B's call changed nothing) | CP-e, I-e-2 |
| AC-e-4 | **DNS rebinding.** Raw requests to the running adapter: `Origin: http://evil.example` → 403; `Host: rebind.example:P` with no `Origin` → 421; `Host: rebind.example:P` with `Origin: http://rebind.example:P` → 421; a correct `Host`, no `Origin`, a valid bearer → the MCP answer. With `--allow-origin http://127.0.0.1:5173`, that origin passes and `http://127.0.0.1:5174` is 403. Mutation M-e-1 turns the second line red | CP-e, §9.3 |
| AC-e-5 | **Bearer.** No `Authorization` → 401 after ≥ 0.5 s; `Basic …` → 401; `Bearer` with a 7-byte token, a 129-byte token, or a token with a space → 401; each measured ≥ 0.5 s from send to first byte. A well-formed but wrong bearer: `join` → `ToolError` `join_refused` / `unauthorized` (the server's own answer and delay) | step §6.2, I-e-4 |
| AC-e-6 | **No secret.** The invite, the four grant tokens, a canary admin token and every resume secret the welcomes carried appear in no byte of: every HTTP response body and header the test received, every MCP message, the adapter's stdout and stderr, and `--state-dir` except the resume files themselves | AC-S27-12, I-e-8 |
| AC-e-7 | **State partition.** After A (visitor grant) joins `visitor` and the relay is cut, the `--state-dir` holds one partition directory per bearer used; a third agent C whose bearer is the `visitor:take-over` grant, joining `visitor` **without** `take_over` while A's seat is held, does not resume A's seat: the answer is `join_refused` with the code `server/PROTOCOL.md` §4.2 gives a held seat joined without `resume` or `take_over` (C6 reads it and records it; `seat_occupied` at the time of writing), because C's partition has no resume file — never `took_over: "held"` | F-S27e-3, I-e-7 |
| AC-e-8 | **Bind rules.** `--http 0.0.0.0:0` without `--allow-remote` exits 2 with the message of §5.5 and binds nothing; with `--allow-remote` it starts and its startup line contains "plain HTTP"; `--allow-host x` without `--allow-remote` exits 2; `--http` with `MINEWORLD_GRANT` set exits 2 | CP-e, QS27-5 |
| AC-e-9 | **Bounded.** With `--max-credentials 2`: two bearers holding active seats; a third well-formed bearer → 503 with `Retry-After`; after one of the two leaves, the third is admitted | F-S27e-6 |
| AC-e-10 | **Same surface.** `tools/list`, `resources/list`, `resources/templates/list`, `prompts/get play` over HTTP equal the stdio answers byte for byte after JSON normalisation, and `tools/list` equals `tests/golden/tools.json` | I-e-1 |
| AC-e-11 | **Shutdown.** SIGTERM (POSIX) / `CTRL_BREAK_EVENT` with a new process group (Windows) to the adapter while A and B hold seats → within `hold + 5` wall seconds `/status.clients` is back to the baseline (POSIX: at once, both handles left; Windows: by leave if the signal is delivered, else by the server's hold — the test accepts either and records which) | step §6.1 |

Each real-binary file ≤ 90 s on Linux CI; over 180 s moves to nightly with a 30 s smoke kept on PRs (step §9.2).

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s27e`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full, before
   editing it.
2. Make the numbered steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and PASS /
   FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation listed: plant it, run the named test, record that it turned red (the failing test's name),
   revert it from a byte copy (`cmp` clean), run the test again green. Never commit a planted mutation.
5. Static checks: `uv run --locked ruff check clients/agent`; `uv run --locked ruff format --check clients/agent`;
   `uv run --locked pyright clients/agent`. Tests one file at a time: `uv run --locked pytest
   clients/agent/tests/<file>`. **R-LOAD-1:** never the whole member, never `cargo test`; the only Rust command is
   `cargo build -p mineworld-cli`. Markdown: `python3 scripts/check_doc_headings.py`;
   `python3 scripts/check_decision_ids.py`.
6. Review: answer each review question of the commit in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**.
8. Update §15 and the handoff in the same commit.
9. Stage by explicit path; `git commit -m "<message>"` plus the session's attribution line. Never `git add -A`,
   heredoc, `sed -i` or `python3 -c`.
10. Stop and report (a material stop, §12) instead of improvising.

The real binary: `export CARGO_TARGET_DIR=/tmp/impl-s27e-target; cargo build -p mineworld-cli; export
MINEWORLD_BIN=/tmp/impl-s27e-target/debug/mineworld`.

#### C0 — Ledger, handoff, the `DEP-56` note

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s27e.md` (new), `docs/DECISIONS.md`.
- Steps:
  1. Record the base commit; confirm S27-d is merged (`git log --oneline | grep -i "S27-d"` and the PR number);
     not merged → stop.
  2. Re-read E-1 … E-19 on the base; record any moved line (bounded) or changed statement (stop).
  3. Append to `DEP-56` in `docs/DECISIONS.md` a paragraph headed `**Note (2026-10-xx, S27-e).**`: §4.1 adopted,
     §4.2's table declined with reasons, the `BearerGate` and its 500 ms floor; sources E-11 … E-16. (If the primary
     has ruled QS27e-3 "own record", write `DEP-S27e-a` instead and stop for the number.)
  4. The handoff: the contract's fields; the sequence C0 … C7.
- [ ] Implementation. [ ] Validation: the doc scripts. [ ] Review: the note says "adapter", "client", "agent
  player" as `ARC-96` defines them, and names no credential.
- **Commit message:** `docs(plan): S27-e — open the ledger; DEP-56 note on the SDK's HTTP transport`

#### C1 — Prototype checks V-1 … V-4

- **Files:** `clients/agent/tests/test_http_prototype.py` (new; stays as a regression test).
- Steps:
  1. Read the installed SDK (`.venv/lib/python3.*/site-packages/mcp/`) with `sed -n`/`grep -rn` and record in §15
     the import paths of `streamable_http_client`, `TransportSecurityMiddleware`, `TransportSecuritySettings`, and
     where an `on_call_tool` handler finds the HTTP request (E-15).
  2. P-e-1: a low-level `Server` with one tool whose handler returns the `Authorization` header's length; served by
     `streamable_http_app(stateless_http=True, transport_security=<loopback settings>)` under uvicorn on a pre-bound
     loopback socket; called by `streamable_http_client` with a header, once with the client's default protocol
     version and once with a 2025-11-25 handshake if the client exposes that choice → the handler sees the header
     both times. Record which attribute carries it.
  3. P-e-2: the same client sends the header on every request (two calls, both seen).
  4. P-e-3: a `subscriptions/listen` on a resource URI receives one cue after a publish on the bus.
  5. P-e-4: the pre-bound socket on port 0 serves; the bound port is the socket's.
- Validation: `uv run --locked pytest clients/agent/tests/test_http_prototype.py`. P-e-1 or P-e-2 failing → stop
  (§12). P-e-3 failing → record (bounded, §3.2), mark that test `xfail(strict=True)` with the reason, continue.
- [ ] Implementation: steps 1–5. [ ] Validation. [ ] Review: no prototype uses a network address but loopback.
- **Commit message:** `test(agent): prototype the SDK's Streamable HTTP transport for the adapter (S27-e)`

#### C2 — The pool and `Player.active`

- **Files:** `clients/agent/src/mineworld_agent/{pool.py (new), player.py, __init__.py}`,
  `clients/agent/tests/test_pool.py` (new). Add `state.py` to Files only if its directory-creation helper must be
  made reusable (record why).
- Steps:
  1. `Player.active()` (§5.3).
  2. `pool.py`: `Credentials`, `Full`, `partition` (§5.3); `admit(credential) -> bool` reserves a slot.
  3. Export `Credentials` from `__init__.py` only if S27-c exports its other building blocks there (match the
     existing style; record the choice).
  4. `test_pool.py` with a `make` double (a fake `Player` with `active()`, `close()`): same credential → same
     `Player`; two credentials → two; `partition` equal across two `Credentials` instances (new salt) and different
     for two credentials; eviction of the least recently used inactive slot; `Full` when all active; `admit` then
     `player_for` is a hit; `repr` contains no credential and no key.
- Validation: `test_pool.py`; S27-c's `test_two_handles.py` (the `Player` change). Mutation **M-e-2** (`partition`
  ignores the credential) → `test_two_credentials_two_partitions` red; **M-e-3** (evict an active slot) →
  `test_full_when_every_slot_is_active` red.
- [ ] Implementation: steps 1–3. [ ] Validation: step 4, mutations. [ ] Review: no credential is a dict key, a log
  argument or a `repr` field; the lock covers creation and eviction.
- **Commit message:** `feat(agent): a bounded pool of players, one per credential (S27-e)`

#### C3 — One server for both transports; the HTTP app and its gate

- **Files:** `clients/agent/src/mineworld_agent/mcp_server.py`, `clients/agent/tests/{test_mcp_sources.py,
  test_http_security.py}` (new).
- Steps:
  1. `PlayerSource`, `Adapter.serve_players`, `Adapter.server` as a constant source (§5.2).
  2. `security_settings(bound, allow_hosts, allow_origins)` (§5.5).
  3. `BearerGate` (§5.4) and `http_app(adapter, credentials, config, settings) -> ASGI app`; the handler-side source
     reads the bearer through the path C1 recorded and calls `credentials.player_for`.
  4. `serve_http(...)` (§5.6).
  5. Tests: `test_mcp_sources.py` (§7 row); `test_http_security.py` (§7 row) including the 500 ms floor (assert
     elapsed ≥ 0.5 s on `time.monotonic()`, and < 2 s) and the 503 path.
  6. Run S27-d's `test_mcp_tools.py`, `test_mcp_errors.py`, `test_mcp_resources.py`, `test_mcp_secrets.py` unchanged.
- Validation: the two new files and the four S27-d files. Mutations: **M-e-1** (step-27 §11.2's planted defect: a
  branch in `BearerGate` that skips the SDK's security check when the request has no `Origin` header and its `Host`
  is not loopback) → the `Host: rebind.example`, no-`Origin` case in `test_http_security.py` red (and AC-e-4's
  second line in C6); **M-e-4** (`BearerGate` answers 401 without the delay) → the
  floor test red; **M-e-5** (the source returns the first pooled `Player` regardless of bearer) →
  `test_each_request_gets_its_own_player` red.
- [ ] Implementation: steps 1–4. [ ] Validation: steps 5–6, mutations. [ ] Review: (a) every path through
  `BearerGate` either refuses before the SDK app or delegates once; (b) no game logic in the gate; (c) the stdio
  server is byte-identical in behaviour (S27-d's tests unchanged and green).
- **Commit message:** `feat(agent): the adapter over Streamable HTTP — a player per bearer, Host and Origin checks (S27-e)`

#### C4 — The command line

- **Files:** `clients/agent/src/mineworld_agent/__main__.py`, `clients/agent/tests/test_cli_http.py` (new),
  `clients/agent/README.md`.
- Steps:
  1. The flags and refusals of §5.7; the socket bound in `__main__` (`socket.create_server((host, port))`,
     IPv6 for `::1`), the startup line of §5.5, then `serve_http`.
  2. `test_cli_http.py`: every refusal of §5.7 and F-S27e-7 (exit 2, the flag named, no value printed); `--help`
     lists no credential option (S27-d M-d-7's rule).
  3. README: three lines — the `--http` command, "each request's `Authorization: Bearer` is the credential", and
     "loopback only unless `--allow-remote`, and then TLS in front".
- Validation: `test_cli_http.py`; S27-d's `test_cli.py` unchanged.
- [ ] Implementation: steps 1, 3. [ ] Validation: step 2. [ ] Review: no default exposes a non-loopback address.
- **Commit message:** `feat(agent): mineworld-agent mcp --http (S27-e)`

#### C5 — The play-test over HTTP (CP-e)

- **Files:** `clients/agent/tests/test_play_mcp.py` (refactor: the script becomes `play(open_session)` with the
  stdio factory as before), `clients/agent/tests/http_adapter.py` (new: start the adapter subprocess with `--http
  127.0.0.1:0`, read the port, open an HTTP session with a bearer), `clients/agent/tests/test_play_http.py` (new).
- Steps:
  1. Refactor `test_play_mcp.py` without changing a single assertion (diff review: only the session plumbing
     moves); run it once over stdio, green.
  2. `http_adapter.py`; `test_play_http.py` = `play(http factory)`, with step 9's take-back made by a second HTTP
     session on the same adapter whose bearer is the `visitor:take-over` grant (AC-e-1), and AC-d-2's frame check
     on the relay's recording.
- Validation: `cargo build -p mineworld-cli` (once); `test_play_mcp.py` (stdio still green); `test_play_http.py`;
  record wall times.
- [ ] Implementation: steps 1–2. [ ] Validation. [ ] Review: the refactor kept every stdio assertion (quote the diff
  stat and the assertion count before and after in §15).
- **Commit message:** `test(agent): the MCP play-test over Streamable HTTP against the real server (S27-e, CP-e)`

#### C6 — Mixed play, binding, endpoint security, bounds, shutdown

- **Files:** `clients/agent/tests/{test_mixed_http.py, test_http_endpoint.py}` (new).
- Steps:
  1. `test_mixed_http.py`: AC-e-2, AC-e-3, AC-e-7.
  2. `test_http_endpoint.py`: AC-e-4, AC-e-5, AC-e-6, AC-e-8, AC-e-9, AC-e-10, AC-e-11 (raw requests with
     `http.client.HTTPConnection("127.0.0.1", P)`, explicit `Host` and `Origin` headers).
- Validation: each file. Mutation **M-e-6** (the pool shares one `state_dir` for all credentials) → AC-e-7 red;
  **M-e-7** (the adapter prints the bearer's first 8 characters in a diagnostic) → AC-e-6 red.
- [ ] Implementation. [ ] Validation. [ ] Review: every oracle is the server's (`perceived`, `/status`, the relay's
  frames) or the HTTP status line, never the adapter's report of itself.
- **Commit message:** `test(agent): two agents and a person through one HTTP adapter; endpoint security (S27-e)`

#### C7 — Close-out

- **Files:** this file (§15), `handoff-s27e.md`.
- Steps: `git diff --name-only origin/main...HEAD` within §2.1; fill §8's evidence; push `mvp0/pr-s27e`; `gh pr
  create` (title `S27-e: the MCP adapter over Streamable HTTP, several agents per adapter`); CI on the exact head on
  Linux, macOS, Windows; repair to green; request the Opus-class review named in the header.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S27-e close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S27f-e-1 (on S27-f).** Chapter 14 documents `--http` as an operator-only block: the command, a bearer from
  the agent client's configuration (`${MINEWORLD_GRANT}` expansion, never a literal), loopback only, and
  `--allow-remote` only behind a TLS tunnel or gateway (QS27-5). If P-e-3 failed, the chapter says the inbox
  subscription is stdio-only.
- **R-S27g-e-1 (on S27-g).** The operator's live check adds one HTTP connection (Claude Code `claude mcp add
  --transport http … --header "Authorization: Bearer ${…}"`, flags verified by the operator).

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-27-agent-players.md` §6.2 | `Host` is checked too (421), beside `Origin` (403); `--allow-host` (if QS27e-1 is ruled yes); `--max-credentials` and 503; one `Player` per credential with its own state partition (F-S27e-3, F-S27e-6) |
| `step-27-agent-players.md` §11.2 "PR S27-e" | R-S27e-d-1 read as "one server for both transports" (F-S27e-1) |
| `overall.md` | S27-e frozen / merged |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S27-e — the MCP adapter over Streamable HTTP, several agents per adapter
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s27e-streamable-http.md
RELATED / BINDING DOCS:  step-27-agent-players.md (§§3, 5.4, 6, 7, 9, 11, 16); pr-s27d-mcp-stdio.md (§§5, 6, 8,
                         10); pr-s27c-player-library.md; pr-s27a-seat-grants-budget.md; server/PROTOCOL.md §§1, 4;
                         docs/DECISIONS.md ARC-96, ARC-97, DEP-56; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s27e, its own, held by one session
BRANCH:                  mvp0/pr-s27e, created from origin/main
START CONDITION:         S27-d merged (met: #239, 42bc3ceb) AND this design frozen (met: operator, 2026-10-18),
                         then the primary session's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–6
FROZEN INVARIANTS:       step-27 INV-S27-1 … INV-S27-13; S27-d I-d-1 … I-d-10; §6 I-e-1 … I-e-11; S27-c's and S27-d's
                         models, caps, tools, errors, resources and prompt unchanged (only §2.1 item 3's additive
                         method)
SEQUENCE:                C0 … C7 (§9)
ALLOWED COMMANDS:        cargo build -p mineworld-cli; uv sync --locked; uv run --locked *; git; gh (never merge);
                         python3 scripts/*; /tmp/impl-s27e-target/debug/mineworld *; mkdir -p; sed -n; grep; cmp
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test; a whole-member or
                         workspace test run (R-LOAD-1); uv lock or any dependency change; reading .env or any key;
                         calling any model; edits under server/, contracts/, kernel/, systems/, worlds/, cognition/,
                         sdk/; a tool, field, resource, prompt or error code beyond S27-d's; any bind but loopback in
                         a test except AC-e-8's 0.0.0.0 start check
MATERIAL STOPS:          S27-d not merged; P-e-1 or P-e-2 failing; a needed change to a published schema, the golden
                         tool list or S27-d's ErrorCode; a needed new dependency; any S27-c or S27-d test turning
                         red; the HTTP play-test over 180 s on Linux CI
PLATFORMS:               Linux, macOS, Windows (the python job on all three; fast's static checks)
VALIDATION BUDGET:       unit files unrestricted, one at a time; each real-binary file ≤ 90 s on Linux CI; CI: the
                         PR's runs; no money is spent (no model, no paid service). Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s27e.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                       (source: the operator's freeze of this design,
                                                       2026-10-18, and the primary's dispatch)
                         semantic commits: authorized  (working rules §14)
                         branch push: authorized       (working rules §21)
                         PR creation / update: authorized (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW, with the Opus-class review requested
                         merge: NOT authorized — never by the implementation session; only with the operator's
                         explicit authorization after the primary's review (working rules §22)
POST-MERGE SYNC OWNER:   primary: step-27, overall.md; implementation session: this ledger
STOP CONDITION:          S27-e READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | The SDK's handler does not expose request headers in one protocol era (V-1) | P-e-1 tests both eras in C1; failing is a material stop with the evidence, before any production code |
| R-e-2 | Windows: uvicorn on a pre-bound socket, signal delivery to a child, Proactor loop | P-e-4 on Windows CI; AC-e-11 accepts the server's hold as the Windows fallback and records which path ran |
| R-e-3 | Timing tests (500 ms floor) flake under CI load | assert only the floor (≥ 0.5 s) and a generous ceiling (< 2 s); never an exact value |
| R-e-4 | Mixed-play oracle flakes because paced residents act during the window | the oracle is per-seat `perceived` export, which includes every fact the seat perceived whoever caused it; nothing asserts a resident's action |
| R-e-5 | The SDK changes its stateless HTTP internals in a patch release | `mcp` is pinned `<2.3` or `<3` by S27-d's lock; the prototype tests stay as regression tests |
| R-e-6 | A pooled `Player` with only lost handles lingers and holds a slot | `active()` counts only handles not lost; such a slot is evictable at once |
| R-e-7 | Doubling the play-test doubles the python job's time on three OSes | ≤ 90 s per file target; the nightly rule of step §9.2 if over 180 s |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS27e-1 [operator — a flag beyond the step's list]** | Under `--allow-remote`, the `Host` header a LAN client or a gateway sends is a name the adapter cannot guess, and the DNS-rebinding defence needs the `Host` check (F-S27e-4, F-S27e-5). Add `--allow-host HOST[:PORT]` (repeatable, only with `--allow-remote`), or accept any `Host` in remote mode and rely on `Origin` and the bearer alone? | **Add `--allow-host`.** Without it, remote mode either refuses every real client or drops the rebinding defence entirely; with it, remote mode stays as strict as loopback mode and the operator names exactly the names a tunnel uses. It changes nothing in loopback mode, the only mode the guide teaches for MVP-0 (QS27-5). **Ruled 2026-10-18 (operator): add `--allow-host`, as recommended; the DNS-rebinding protection (the `Host` check) is kept.** |
| **QS27e-2 [primary — capacity answer]** | When every pooled credential holds an active seat, a request with a new bearer is answered HTTP 503 with `Retry-After`, before MCP, so the published error vocabulary stays S27-d's (F-S27e-6). Accept 503, or add an `ErrorCode` `busy` to the published schema instead? | **503.** Capacity is the transport's condition, not a world answer; a new `ErrorCode` would change S27-d's frozen published schema and golden for a case an agent cannot act on except by retrying, which `Retry-After` already says. **Ruled 2026-10-18 (operator): HTTP 503 with `Retry-After` when credentials are full, as recommended; no new `ErrorCode`.** |
| **QS27e-3 [primary — record placement]** | The SDK parts adopted and declined (§4) are written as a dated note under `DEP-56`. Accept a note, or require a separate record (placeholder `DEP-S27e-a`, number to assign)? | **A note under `DEP-56`.** It is the same dependency, adopted by the same decision; the note only says which of its modules this PR uses and why its OAuth layer is not one of them. **Ruled 2026-10-18 (operator): a dated note under `DEP-56`, as recommended; `DEP-S27e-a` is not used.** |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] DEP-56 note, ledger, handoff | [x] `check_decision_ids.py` PASS (168 ids, distinct); `check_doc_headings.py` PASS | [x] the note names the adapter, the client and the agent player as ARC-96 does, and no credential |
| C0 | `07524bd1` | (as above) | (as above) | (as above) |
| C1 | (this commit) | [x] `test_http_prototype.py` (P-e-1 … P-e-4) | [x] `pytest clients/agent/tests/test_http_prototype.py` 2 passed, 1.3 s, PASS; ruff, ruff format, pyright 0 errors PASS | [x] every socket is `127.0.0.1`; P-e-4 is covered by both tests (each serves a socket bound on port 0) |
| C1 | `d40ae169` | (as above) | (as above) | (as above) |
| C2 | (this commit) | [x] `Player.active`, `pool.py` (`Credentials`, `Full`, `partition`, `admit`) | [x] `test_pool.py` 9 passed, 0.05 s, PASS; ruff, format, pyright PASS; `test_two_handles.py` deferred to C5's run (needs the binary; recorded there) | [x] keys are HMACs, never credentials; no credential in a log or `repr`; one `asyncio.Lock` covers lookup, creation and eviction |
| C2 | `a8096fdf` | (as above) | (as above) | (as above) |
| C3 | (this commit) | [x] `PlayerSource`, `serve_players`, `Adapter.server` as a constant source; `security_settings`; `BearerGate`; `bearer_source`; `http_app`; `serve_http` | [x] `test_mcp_sources.py` 3 passed; `test_http_security.py` 11 passed (6.5 s with the sources file); S27-d's `test_mcp_tools.py` 9, `test_mcp_errors.py` 7, `test_mcp_resources.py` 8, `test_mcp_secrets.py` 2, `test_mcp_confined.py` 6 passed unchanged; ruff, format, pyright PASS; M-e-1 red then green (below) | [x] (a) every path through the gate refuses before the SDK app or delegates once; (b) no game logic in the gate; (c) stdio behaviour unchanged — S27-d's files are unchanged and green |
| C3 | `5c2c8409` | (as above) | (as above) | (as above) |
| C4 | (this commit) | [x] `--http`, `--allow-remote`, `--allow-origin`, `--allow-host`, `--max-credentials`; the socket bound in `__main__`, the startup line, `serve_http`; README | [x] `test_cli_http.py` 22, `test_cli.py` 7, `test_mcp_confined.py` 6 — 35 passed, 1.5 s, PASS; ruff, format, pyright PASS | [x] no default exposes a non-loopback address: without `--http` it is stdio; `--http` with a non-loopback host exits 2 unless `--allow-remote`; `mcp` is still imported by `mcp_server.py` alone |
| C5 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

- **C0.** Base `origin/main @ a1e05874`; S27-d merged as #239 (`42bc3ceb`). E-1 … E-19 re-read on the base: line
  numbers moved (bounded; `Adapter.server` is now `mcp_server.py:354-481`, `_read` `:483-503`, `serve_stdio`
  `:513-517`; `__main__._serve` `:96-100`), no statement changed. Decision records still live in
  `docs/DECISIONS.md` (#258 not merged). The SDK installed is `mcp` 2.2.0 under Python 3.14.
- **Speed rule.** The operator's SPEED FIRST rule of 2026-10-18 (lane charter) applies: mutations beyond M-e-1
  (requested by the dispatch) are not planted per commit; they are listed below as not run.
- **C1.** Import paths: `mcp.client.streamable_http.streamable_http_client` (with `http_client=httpx2.AsyncClient(
  headers=…)`, wrapped in `mcp.Client(..., mode="legacy" | "auto")`); `mcp.server.transport_security.
  TransportSecurityMiddleware` and `TransportSecuritySettings`. The handler finds the HTTP request at
  `ctx.request` (a Starlette `Request`; both the legacy stateless path and `handle_modern_request` attach it as
  `ServerMessageMetadata(request_context=request)`, read by `ServerRunner._make_context`). P-e-1: PASS in both eras
  (legacy handshake and 2026-07-28 via `mode="auto"`, the two negotiated versions differ). P-e-2: PASS (four calls,
  four identical headers). P-e-3: PASS (a cue over stateless HTTP with SSE). P-e-4: PASS on macOS (both tests serve a
  socket bound on port 0); Linux and Windows by CI.
- **C2.** `Credentials` is not exported from `__init__.py`: S27-c exports models, `Player` and `PlayerConfig` only, not
  its building blocks (`StateDir`, `Adapter`). `state.py` unchanged: `StateDir.write` already creates its directory
  (`mkdir(parents=True)`), so each partition is created on the same path the root is; S27-c does not set 0700 on
  the root either, so the partition follows the root's rule (bounded; the files inside stay 0600).
- **C3.** `serve_http` takes the bound socket typed `Any`: S27-d's `test_the_adapter_reaches_the_world_only_through_player`
  forbids `mcp_server.py` from importing `socket`, so the command line binds it and this module only hands it to
  uvicorn (bounded). `bearer_source` answers `not_connected` (an existing code) if the pool filled between the
  gate's reservation and the handler — a race only; no new `ErrorCode` (QS27e-2). uvicorn handles `SIGBREAK` on
  Windows (its `HANDLED_SIGNALS`), so AC-e-11's `CTRL_BREAK_EVENT` reaches `serve_http`'s shutdown path.
- **Process slip (C3).** The HTTP half of `mcp_server.py` was first appended with a shell heredoc, which §12's NEVER
  list forbids for tracked files; the content was then reviewed and edited with the editor tool, and the rest of the
  PR uses the editor only. Recorded, not hidden.

### Mutations

| Id | Planted | Test | Result |
| --- | --- | --- | --- |
| M-e-1 | in `BearerGate.__call__`, `refused = None` instead of the SDK's check when the request has no `Origin` and its `Host` (port stripped) is not loopback | `test_http_security.py::test_a_rebinding_host_without_an_origin_is_421` | **red**: `AssertionError: a refused request created a Player` (`assert [<Recorder …>] == []`); restored from a byte copy (`cmp` clean); green again (1 passed) |
| M-e-2 … M-e-7 | not planted | — | not run: the operator's SPEED FIRST rule (2026-10-18) drops per-PR mutations; the dispatch asked for M-e-1 only |

**Finding F-S27e-9 (bounded, from M-e-1).** Under M-e-1 the status line stays **421**: the SDK app repeats its own
transport-security check (§5.4 step 1 calls it harmless), so a status-only oracle cannot see the planted defect. The
defect is that the refused request reaches the bearer logic and reserves a `Player`. The in-process test therefore
asserts that a refused request creates no `Player`, and that assertion is what turns red. AC-e-4 against the real
adapter (C6) cannot see the pool, so it checks the status lines only; the mutation evidence is the in-process test's.

### Deviations and findings

F-S27e-1 … F-S27e-8 (§3.3), recorded at design time.

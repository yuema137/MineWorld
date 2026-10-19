## DEP-56 — The official MCP Python SDK (`mcp`), for the stdio adapter only, registered on its low-level `Server`

**Date** 2026-10-15 · **Status** accepted (design frozen 2026-10-15); dependency added in S27-d C1 · **Approved by** the operator's step-27 rulings of 2026-10-14 (§4.2, the MCP adapter) and the S27-d rulings QS27d-1 and QS27d-2 of 2026-10-15 · **Relates to** `ARC-96` (the adapter is a client; MCP confined to one module), `ARC-97`, `DEP-24`, `DEP-44`, `REUSE_POLICY.md` §§11–12, 15, `CLAUDE.md` §4 rule 16 · **Design** `.structured-coding/plans/mvp0/pr-s27d-mcp-stdio.md` §§3.2, 4.1, 4.2

**Problem.** An MCP-speaking agent must reach a `Player` (`ARC-96`) over stdio. The MCP wire protocol (JSON-RPC, lifecycle, capability negotiation, the 2026-07-28 revision and its predecessors, subscriptions) is a mature protocol with an official SDK; re-implementing it would duplicate that work without a reason. The SDK must not, however, own world state, seats, validation or caps, and its high-level decorator API would publish each tool's input schema nested under its parameter name, which would break the rule that each tool's `inputSchema` and `outputSchema` are the `Player`'s own models.

**Options considered** (both directions).

| Option | Verdict |
| --- | --- |
| (a) **`mcp` (Python, official, MIT), confined to `clients/agent/src/mineworld_agent/mcp_server.py`**, on the low-level `Server` with `Tool(input_schema=Args.model_json_schema(), output_schema=Result.model_json_schema())` | **Chosen.** The SDK owns the transport and JSON-RPC; the schemas are the models' exactly (one source); dispatch validates arguments through the models; the subscription handler is the SDK's `ListenHandler`. Cost: about 40 lines of wiring that the high-level server would have provided |
| (b) `MCPServer` with one decorated function per tool | Declined: the published `inputSchema` is nested under the parameter name (`$defs` reference), so it is not `ObserveArgs`'s schema (F-S27d-1) |
| (c) `MCPServer` with flattened parameters | Declined: thirteen hand-written signatures mirroring the models, a second schema source |
| (d) `rmcp` (Rust) inside the server | Declined: the server would speak a second client vocabulary beside `join`/`submit`/`leave`, which `server/PROTOCOL.md` §2 forbids; MCP types would enter `server/` |
| (e) `rmcp` 3.5.1 in a separate Rust client binary beside the adapter | Declined for now (step-27 §4.2): the resuming client would be written a second time in Rust. Revisit on QS27-8's trigger (installing `uv` as the main obstacle for players of the bundle) |
| (f) The TypeScript SDK 1.32.1 | Declined: the language rule (`CLAUDE.md` §5 and `ENGINEERING_STANDARDS.md` §3) puts client code in Python |
| (g) Our own JSON-RPC and MCP lifecycle | Rejected: a protocol implementation is commodity infrastructure (`REUSE_POLICY.md`), and the revision churn (`X-2`, `X-3`) would be ours to follow |

**Version rule** (step-27 §4.2): the newest release at least 14 days old when the dependency is added. On 2026-10-15 the newest release at least 14 days old is **2.2.0**: `mcp` 2.3.0 was uploaded 2026-10-02, 13 days earlier. The requirement is `mcp>=2.3,<3` from 2026-10-16 on, and `mcp>=2.2,<2.3` before it; C1 records the date, the requirement written and the locked version.

**Sources** (verified 2026-10-15): PyPI JSON for `mcp` (newest 2.3.0, 2026-10-02T22:06:52Z; 2.2.0, 2026-09-07; 2.0.0, 2026-07-28; none yanked; licence MIT; `requires_python >=3.10`); the MCP specification's versioning page (current revision 2026-07-28); the SDK's release notes (`https://github.com/modelcontextprotocol/python-sdk/releases`); the SDK's documentation for tools, structured output, error handling, the low-level server, subscriptions and testing (`https://py.sdk.modelcontextprotocol.io/`).

**Consequences.** `mcp` is a dependency of the `mineworld-agent` member only. The repository-wide scan (`clients/agent/tests/test_mcp_confined.py`) finds `mcp` nowhere else: no other `pyproject.toml` names it and no other Python file imports it. The packages `mcp` pulls in are listed with their licences in the design's ledger (§15); a GPL-family licence is a material stop. Removing `clients/agent` from the workspace removes this dependency with it (AC-d-10).

**Revisit** if the SDK stops accepting an `outputSchema` shape for error results this adapter relies on (`ToolError` as an `isError` result without `structuredContent`, `QS27d-1`); if subscriptions over stdio change; or when S27-e (Streamable HTTP) reuses `build_server` (`R-S27e-d-1`).

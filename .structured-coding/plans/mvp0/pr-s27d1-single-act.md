# PR S27-d1 — One `act` tool instead of five (the MCP adapter's tool surface)

## DESIGN FROZEN — operator ruling 2026-10-18

```text
Ruling:                 the operator, 2026-10-18, in the primary session: "merge the 5 action tools into one
                        `act` tool". Scope, invariants and acceptance below follow the ruling; the details
                        are the implementer's (the primary's dispatch)
Implementation base:    origin/main @ 96b41db0 (S27-d #239 merged as 42bc3ceb)
Branch / worktree:      feat/mcp-single-act, /Users/yuema137/mineworld-worktrees/mcp-single-act
Decision record:        ARC-153 (amends the MCP surface of ARC-96 item 2; the Player library is unchanged)
Lifecycle:              DESIGN FROZEN 2026-10-18 → implementation in the same PR → READY FOR OPERATOR REVIEW
```

**Effort:** `mvp0` · **Step:** [`step-27-agent-players.md`](step-27-agent-players.md) §5 (the surface) ·
**Amends:** [`pr-s27d-mcp-stdio.md`](pr-s27d-mcp-stdio.md) §5.2 (the tools) and §8 I-d-4 (the closed list) ·
**Affects:** [`pr-s27e-streamable-http.md`] and [`pr-s27f-agent-guide-reference.md`] (PR #237, frozen; §6
below lists every line to update) · **Binding parents:** `CLAUDE.md` §§2, 4 (items 3, 7, 8, 12);
`docs/DECISIONS.md` `ARC-96`, `DEP-56`.

## 1. Problem, measured

S27-d publishes thirteen tools. On the wire `tools/list` is **158 506 bytes** (compact JSON, measured on
`96b41db0` with `catalogue_json(compact=True)`, which is byte-identical to the `ListToolsResult` tool list).
The five act tools `say`, `walk_to`, `move`, `stop`, `attempt` are 105 052 of those bytes, and almost all of
that is the same `ActResult` `outputSchema` (≈ 20 KB, because `ActResult.view` carries the whole `View`)
written five times. MCP 2026-07-28 has no definitions shared between tools (S27-d ledger, "Tool-list size",
F-d-R-2), so a client that shows output schemas to its model pays every copy. Only a change of the tool
surface removes the repetition; the operator ruled that change.

## 2. The ruling, as checkable rules

- **R-1.** The five act tools are replaced by one tool `act`. Its `outputSchema` is `ActResult`, published
  once. The tool list becomes nine tools, in this order: `seats`, `join`, `observe`, `inspect`, `events`,
  `wait`, `act`, `clock`, `leave` (closed, deterministic, X-4).
- **R-2.** `act` takes a `kind` discriminator: a tagged union of the five argument shapes, each exactly the
  fields the old tool took (handle aside), with the same types, bounds and descriptions.
- **R-3 (the operator's principle, unchanged).** Agents are first-class players: the same information, the same
  rules and the same server path as a human. Each kind calls the same `Player` operation the old tool called
  (`Player.say`, `walk_to`, `move`, `stop`, `attempt`), with the same arguments; nothing about what the world
  sees, validates or answers changes. Only the call shape changes.
- **R-4 (R1's no-echo rule, kept and widened).** No error detail echoes agent input. Widened by the review's
  follow-ups: an unknown tool name, prompt name or resource URI is answered without repeating it
  (`mcp_server.py` lines 384, 461, 496, 500 on the base).
- **R-5.** `leave` is annotated `destructive_hint=True` (it gives the seat back; a second call cannot undo
  it), `idempotent_hint=False` as before.
- **R-6.** The two prototype tests in `test_mcp_confined.py` (lines 98–160 on the base: P-1, P-2) test only
  the `mcp` SDK's own behaviour; they are deleted. What the adapter relies on from them stays covered by the
  adapter's own tests: P-1 (schemas published unchanged) by `test_the_published_schemas_are_the_models`, P-2
  (an `isError` text-only result is delivered for a tool with an `outputSchema`) by every row of
  `test_mcp_errors.py`, which runs through the official client.
- **QS27-9 is not touched.** QS27-9 refused a *generic* `act(action_type, target, payload)` tool for free-form
  actions until payload schemas are published. The `act` of this PR is not that: its kinds are the closed five
  of S27-d, every other action is still reached only through `attempt` on a complete offer, and `invite` stays
  unreachable. The name is reused; the meaning QS27-9 ruled on is not. ARC-153 says so.

## 3. The schema

```text
act  inputSchema  = ActArgs   { handle: Handle, act: Act }
     Act          = SayAct | WalkToAct | MoveAct | StopAct | AttemptAct, discriminated by `kind`
       SayAct       { kind: "say",      to: Ref, words: str (1..480 UTF-8 bytes) }
       WalkToAct    { kind: "walk_to",  to: ToPlace | ToPerson | ToPoint }
       MoveAct      { kind: "move",     to: Point, facing_deg: int | null = null }
       StopAct      { kind: "stop" }
       AttemptAct   { kind: "attempt",  view: int ≥ 1, offer: int ≥ 1 }
     outputSchema = ActResult (unchanged)
```

- **Nested, not flat.** The union sits in the field `act`, beside `handle`, so the tool's `inputSchema` stays a
  plain `type: object` with `properties` and `required`. A flat union (the five `*Args` as the root) would make
  the root a `oneOf` with no `type`; MCP requires an `inputSchema` of `type: "object"`, and some model APIs that
  MCP clients forward tool schemas to refuse a combinator at the root of a tool's input schema. `handle` is
  outside the union because every kind needs it, as every other seat tool does.
- **Where the models live.** `models.py`, beside the other argument models (R-S27d-c-1: the published schemas are
  the `Player`'s own models). Each act's fields are defined once, in a body class (`Say`, `WalkTo`, `Move`,
  `Stop`, `Attempt`). `SayAct(Say)` adds `kind`; `SayArgs(Say, Seated)` is the library's argument model, with the
  same fields, order and schema as before. No field is written twice, and the `Player` library (`ARC-96` item 2)
  is unchanged: a Python agent still calls `player.say(SayArgs(...))`.
- **`kind` is required** in every variant (no default): an `act` without one is refused, not guessed.
- **Dispatch.** The adapter's `act` handler validates `ActArgs`, looks the variant's `kind` up in one table
  (`kind → (the library's *Args model, the Player operation)`), builds that `*Args` from `handle` and the
  variant's own fields, and awaits that operation. The table is the only place a kind is bound to an operation;
  a test binds each kind to a distinct answer, so a kind wired to the wrong operation goes red (§5, M-1).
- **Description.** `act`'s description is one line ("Act in the world: one call, five kinds, chosen by
  `act.kind`"), then each kind's paragraph (the kind name and the `Player` operation's docstring, as each old
  tool's description was), then `Errors: invalid_arguments, unknown_handle, seat_lost, unknown_ref`, the error
  note and the `WorldText` note. Every word an agent read in the five descriptions is still in the one.

## 4. Errors

- An unknown `kind` → `invalid_arguments`, `detail` `act: unknown kind; expected one of: 'say', 'walk_to',
  'move', 'stop', 'attempt'` (the existing rule in `_message`: pydantic's `union_tag_invalid` names the input
  tag, so the message is rebuilt from the expected tags alone).
- A missing `kind` → `invalid_arguments`, `act: Unable to extract tag using discriminator 'kind'` (pydantic's
  message holds no input).
- A bad field inside a kind → `invalid_arguments` at `act.<kind>.<field>`. The kind in the location is always one
  of the five schema values (pydantic only enters a variant whose tag matched), so it repeats the schema's
  vocabulary, never free agent text.
- Unknown tool / prompt / resource → the same JSON-RPC errors as before, with the fixed messages `unknown tool`,
  `unknown prompt`, `unknown resource`.
- An act's refusal stays data (`ActResult.outcome`), as before.

## 5. Tests and acceptance

| Id | Check | Test |
| --- | --- | --- |
| AC-1 | The tool list is exactly the nine names of R-1, in order | `test_the_tool_list_is_closed` |
| AC-2 | Every published schema equals its model's schema with `title` keywords dropped and nothing else changed; `act`'s output schema is `ActResult`'s | `test_the_published_schemas_are_the_models` (kept) |
| AC-3 | Every field of every result reaches the agent through some `outputSchema` (I-d-3) | `test_every_player_field_reaches_the_agent` (kept) |
| AC-4 | Each kind reaches its own `Player` operation with the library's `*Args` model, and answers that operation's result | new `test_each_kind_reaches_its_own_operation` |
| AC-5 | The official `mcp` client validates an `act` result against the published `outputSchema` | new case in `test_the_official_client_validates_…` |
| AC-6 | A bad `kind` (unknown, missing) answers `invalid_arguments` without echoing it; an unknown tool, prompt or URI is answered without echoing it | `test_bad_arguments_…`, new `test_unknown_names_are_not_echoed` |
| AC-7 | `leave` carries `destructiveHint: true` | the golden, and `test_leave_is_destructive` |
| AC-8 | The golden is regenerated and byte-equal to `mineworld-agent schema` | `test_cli.py` |
| AC-9 | The real-binary acceptance tests (AC-d-1, AC-d-6, AC-d-8, step 8) pass with only the call shape changed | `test_play_mcp.py`, `test_mcp_same_answers.py` on CI (Linux, macOS, Windows) |
| AC-10 | Before and after `tools/list` byte sizes are recorded | §7 |

**M-1 (mutation).** In the dispatch table, bind `move` to the `walk_to` operation. AC-4 must go red; restore, green.

## 6. Documents to update

In this PR: `docs/DECISIONS.md` (`ARC-153`; `ARC-96` gains a "Superseded in part by" pointer), the S27-d ledger
(a pointer from its "Tool-list size" open item to this PR), `clients/agent/README.md` (no tool names; unchanged),
the golden `clients/agent/tests/golden/tools.json`, the module docstrings that say "thirteen".

For the primary session (parent documents and PR #237, which this PR does not edit):

- `step-27-agent-players.md` line 64 (the tool list), §5's table (line 467 `walk_to` row and the `say`, `move`,
  `stop`, `attempt` rows), the play-test steps (line 733 `walk_to { place: café }`), line 884: the MCP tools are
  now nine, the five acts are kinds of `act` (ARC-153); the `Player` operations are unchanged.
- `pr-s27e-streamable-http.md` (PR #237): line 31 "exactly the thirteen tools" → nine; E-1 (line 126) "binds the
  thirteen tool names" → the nine tools, `act` dispatching to five `Player` operations; the cited line ranges
  of `mcp_server.py` (E-1, `:297-424`, `:299-313`) move; line 295 "the largest tool call is a `say` of 480
  bytes" → "an `act` of kind `say`". I-e-1 and AC-e-10 (byte-identical to the golden) hold as written,
  against the new golden.
- `pr-s27f-agent-guide-reference.md` (PR #237): line 116 "a generic `act` tool (QS27-9)" must be reworded
  ("a free-form action tool") so it no longer names the real `act` tool; line 182 "thirteen known tools" → nine;
  §5.2's transcript calls (lines 232–242: `walk_to {…}`, `say {…}`, `attempt {…}`) become `act {handle, act:
  {kind: walk_to, …}}` and so on, and the kept keys move under `act`; G-17's anchors into `mcp_server.py` move;
  I-f-8's golden is the new one.

## 7. Ledger

**C1 (2026-10-18), one commit: models, adapter, tests, golden, `ARC-153`.**

- **Sizes, measured** with `catalogue_json(compact=True)` (byte-identical to the `ListToolsResult` tool list on
  the wire): **158 506 → 78 624 bytes** (−50.4 %); the indented golden 336 655 → 162 812; all `outputSchema`s
  141 862 → 63 210. Per tool after: `act` 25 093, `observe` 18 547, `join` 16 641, `wait` 6 170, `events`
  4 379, `inspect` 3 596, `clock` 2 008, `seats` 1 368, `leave` 897.
- **Finding F-d1-1 (for the operator; not a stop).** The ruling expected about 40 KB. The measured 78.6 KB is the
  floor of this ruling: `View` is still carried three times, by `join` (16.6 KB), `observe` (18.5 KB) and
  `ActResult.view` (the fresh view of a `stale_offer`) inside `act`. Removing those copies would change what a
  result holds (for example, `act` answering a stale offer without the fresh view), which the ruling does not ask
  for and R-3 forbids without a ruling.
- **The library is unchanged:** the JSON Schemas of `SayArgs`, `WalkToArgs`, `MoveArgs`, `StopArgs` and
  `AttemptArgs` are byte-identical before and after (dumped with `sort_keys`, `diff` clean); the field order of
  each stays `handle` first (`SayArgs(Say, Seated)`).
- `act`'s `inputSchema` is `type: object` with `handle` and `act` (`$ref: #/$defs/Act`, a `oneOf` with the
  `kind` discriminator mapping).
- Tests changed with the call shape only: `test_mcp_errors.py` (the `say` and `stop` rows call `act`),
  `test_mcp_secrets.py` (every kind of `act` is called against the canary player), `test_play_mcp.py` and
  `test_mcp_same_answers.py` (the six `say`/`walk_to`/`attempt` calls), `test_descriptions.py` (`ActArgs` is a
  root), `test_world_text.py` (`Say.words` and `SayAct.words` are the same free-text field as `SayArgs.words`;
  23 operation models). Deleted: P-1, P-2 in `test_mcp_confined.py` (R-6). New in `test_mcp_tools.py`:
  `test_the_act_kinds_are_the_five_operations`, `test_each_kind_reaches_its_own_operation`,
  `test_the_official_client_validates_an_act_result`, `test_an_unknown_kind_is_named_from_the_schema_alone`,
  `test_unknown_names_are_not_echoed`, `test_leave_is_destructive`; `test_bad_arguments_…` gains five `act`
  rows (unknown kind, missing kind, a canary reference, a canary view number, a canary as the act).
- Validation (macOS): `ruff check`, `ruff format --check`, `pyright clients/agent` clean;
  `pytest clients/agent -m "not real_server"`: **147 passed**. `check_decision_ids.py`: 168 ids, all distinct.
- **M-1** (`"move": (MoveArgs, "walk_to")` in `ACTS`): `test_each_kind_reaches_its_own_operation` **FAILED**
  (`AssertionError: move`); restored from a byte copy (`cmp` clean); 15 passed.
- **Real binary (AC-9), macOS**, `cargo build -p mineworld-cli` from this tree, under `with-slot.sh` with
  `MINEWORLD_BIN` set: `test_play_mcp.py` (AC-d-1, AC-d-6, AC-d-8, step 8), `test_mcp_same_answers.py`,
  `test_mcp_admin_view.py`, `test_mcp_guard.py`: **4 passed** (37.8 s); `-m real_server` (the library's own
  real-server files): **5 passed**. Linux and Windows: CI on the PR head.
- `__main__.py`'s `mcp` help says "the nine tools".

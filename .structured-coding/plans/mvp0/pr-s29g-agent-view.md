# PR S29-g — The station in the agent (MCP) view: `PlaceLine.roles`, and an agent that walks to the station

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended)

```text
Design revision:        revision 1a (2026-10-19, lane lead: reconciled with S29-a revision 4 — role tags are
                        CORE_CONCEPTS §6.5, not §6.4; I-a-7 reworded there as Q-S29g-2 recommends, asked as
                        Q-S29a-10; the station keys and tags are unchanged), DRAFT; revision 1 (2026-10-19)
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29g, branch docs/pr-design-s29g,
                        planning base origin/main @ 44f050e3 (#171 12d merged; #261 ARC-153 merged; #239 S27-d
                        merged; #204 S27-c merged; S29-a DRAFT revision 3, not frozen, not merged)
Approved by / evidence: none yet. The step's rulings (Q-S29-1 … Q-S29-10, step-29 §13.1), S29-a's Q-S29a-1 … 7 and
                        ARC-153 bind and are not re-asked. §14 holds this PR's questions
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S29-a, S29-b,
                        S31-g, S27-f and S31-a
Start condition:        step-29 §11.2: "Nothing starts before Milestone F's checklist is handed to the operator."
                        S29-g is second wave: "S29-b, S29-c, S29-f and S29-g follow in parallel worktrees" once
                        S29-a has merged. Its PRs, at 2026-10-19:
                          S29-a          DRAFT revision 3, not frozen, not merged      (hard: content, §6.5 text)
                          S27-c (#204)   MERGED (58a6f55f)                              (hard: the library)
                          S27-d (#239)   MERGED (42bc3ceb)                              (hard: MCP over stdio)
                          ARC-153 (#261) MERGED (94183574) — one `act` tool             (hard: the surface)
                          S29-b          not designed                                   (soft: the newsstand arm)
                        Then this design frozen, then the primary session's dispatch
Execution contract:     §12 (filled; ends at MERGE-READY — the coordinator merges; charter "SPEED FIRST")
Implementer:            Haiku-class (step-29 §11.1, the S29-g row); every commit is numbered steps with exact
                        paths and commands
Lifecycle:              DESIGN FROZEN 2026-10-19 —
```

**Effort:** `mvp0`, before S23's release candidate (step-29 §11.2) · **Step:**
[`step-29-station-landmark.md`](step-29-station-landmark.md) (APPROVED 2026-10-15) §§0.3 ("Agents are first-class
players"), 1 (outcome 4), 3, 5.1, 5.2, 9.4, 9.5 (X-5, X-7: their agent halves), 10 (I-S29-1, I-S29-3), 11.1 (the
S29-g row), 11.3 (**AC-S29-5's agent half**) · **Parent:** [`overall.md`](overall.md) · **Consumes:**
[`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) (DRAFT rev. 3: the station content and `CORE_CONCEPTS`
§6.5), [`pr-s27c-player-library.md`](pr-s27c-player-library.md) (MERGED), [`pr-s27d-mcp-stdio.md`](pr-s27d-mcp-stdio.md)
(MERGED), [`pr-s27d1-single-act.md`](pr-s27d1-single-act.md) (MERGED, `ARC-153`) · **Shares files with:**
`pr-s31g-agent-view.md` (DRAFT, `design-s31g` @ 5e7e6e23; §5.9 states the order rule) · **Binding parents:**
`CLAUDE.md` §§1.1, 2, 3, 4 (items 3, 7, 8, 9, 10, 12, 13, 15); `docs/CORE_CONCEPTS.md` §§6, 15; step-27 INV-S27-1 …
13; decision records `ARC-105`, `ARC-108`, `ARC-153`.

**Decision ids.** None new. The step assigned `ARC-108` ("Station fixtures show only disclosed state";
"implemented by S29-e, S29-f and S29-g") and this PR writes one later note inside it,
`## ARC-108 note — the agent view (S29-g)`, in `docs/decisions/ARC-108.md` (C1). `ARC-198` (the sub-range offered)
is **not used**.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-29 §11.1, the S29-g row, verbatim).** "`PlaceLine.roles`; the §9.4 scenario in `clients/agent`".
  Depends on "S29-a; **S27-c merged** (S27-d for MCP transport)". Checkpoint "the §9.4 scenario passes over MCP".
  Adversarial "the adapter must not invent a service state: a planted `departures` fixture is shown only as
  disclosed". Implementer **Haiku**.
- **Step §9.4 (verbatim, the rendering).** "The `PlaceLine` for `me.place` and for each `exits[].leads_to` carries
  the place's tags. It renders the **role tags** (`station`, `stop`, `platform`, `ticket-office`, `landmark`) as a
  `roles` list ahead of the other tags. This is a rendering of the Observation and nothing more. The adapter infers
  no service state (INV-S27-2). If S27-c's `PlaceLine` already carries tags, the change is the ordering only."
- **Step §9.4 (verbatim, the test).** "an agent seat in `market-town` 1. observes at `street`: one exit's
  `leads_to.roles` contains `station` and `landmark`; 2. calls `walk_to` station: arrives; 3. observes: offers
  include `buy newspaper`; 4. moves to `station-tickets` and observes: **no offers**, `roles` contains
  `ticket-office`, and no `shop` is disclosed; 5. moves to `station-platform`: `roles` contains `stop` and
  `platform`. The same run on `social-cafe` shows no offers in the hall."
- **AC-S29-5 (step §11.3).** "The window is shut, the kiosk follows the shop rule, and the board reads 'No service
  yet' in 2D, in 3D and in the agent view. X-5 and X-7 behave as specified."
- **`ARC-108` item 1 (decision record).** "One pure rule, from disclosed components. It is written once in
  `clients/shared/station/station_state.gd` and used by both clients; **the agent view renders the same
  Observation**."
- **`ARC-153` (merged #261).** The MCP surface is nine tools, and acting is one `act` tool with typed kinds. The
  step's "calls `walk_to`" therefore means `act { kind: "walk_to" }`. This PR adds **no** kind and no tool.
- **INV-S27-2 (step-27).** "Everything the adapter tells an agent about the world is derived from its own seat's
  `Observation`, `perceived` facts, `clock` frames and the public `GET /status`."
- **INV-S27-12 (step-27).** World-authored text is carried only in fields typed as untrusted world text.
- **Operator rules (charter, 2026-10-18).** SPEED FIRST: per PR, the build, the tests the commit writes and `fast`
  CI; acceptance and mutations run at the milestone. No world-specific hard-coding: every acceptance criterion
  names a second world or a generated variant. Outside agents play by the same server path and rules as humans.

---

## 1. Goal

An outside agent playing over MCP must be able to find the town's station and landmark the way a human finds
them: by what the world says the places are. Today it already receives each place's tags as `labels`, as
untrusted world text in author order. This PR adds one typed field, `PlaceLine.roles`: the subset of the place's
tags that are **role tags** (a fixed framework vocabulary, `CORE_CONCEPTS.md` §6.5 from S29-a), in the
vocabulary's order, placed ahead of `labels`. Then it proves, over MCP against the real server, that an agent
in each of five worlds can explore to the station, walk into its hall, ticket office and platform, and reach the
landmark, and that the agent view shows exactly what the world disclosed and nothing more. Before trains exist,
"nothing more" means: no `buy` at the ticket window, no `shop` disclosed there, and no line anywhere that says
"no service" or lists a departure. A `departures` record planted into an observation is carried verbatim by
`inspect` and changes nothing in the view.

A concrete example. At Market Town's street, the agent's view today shows an exit like
`{"leads_to": {"ref": "E9", "labels": [<<<world station world>>>, <<<world public world>>>, <<<world landmark world>>>]}}`.
After this PR it shows `{"leads_to": {"ref": "E9", "roles": ["station", "landmark"], "labels": [...the same
three...]}}`. The agent can now match `"station"` exactly, without reading untrusted text as an instruction.

---

## 2. Scope

### 2.1 In scope

1. **`clients/agent/src/mineworld_agent/`**:
   - `roles.py` (new, ≈ 40 lines): `ROLE_TAGS` (the tuple of the five role tags, in `CORE_CONCEPTS.md` §6.5's
     order), `RoleTag` (a `Literal` of the same five), and `roles_of(tags: Sequence[str]) -> list[RoleTag]`
     (§5.3).
   - `models.py`: `PlaceLine` gains `roles: list[RoleTag]`, declared **before** `labels` (§5.2).
   - `view.py`: one helper `place_line(refs, place, tags) -> PlaceLine`, used by `_exits` and by `render`'s
     `me.place` (§5.4).
   - `__init__.py`: export `RoleTag` and `ROLE_TAGS`.
2. **`clients/agent/tests/`**:
   - `test_roles.py` (new, unit, no binary): U-1 … U-6 of §7.
   - `test_play_station_mcp.py` (new, `real_server`): the scenario over MCP on W1 … W5 (§8).
   - `golden/tools.json`: regenerated from the models (the published `outputSchema`s carry `PlaceLine`).
3. **Documentation:** the `ARC-108` note (§0 "Decision ids"); `docs/decisions/INDEX.md` regenerated if the
   checker requires it; one line in `clients/agent/README.md`; `docs/reference/agent-tools.*` regenerated **only
   if S27-f has merged at C0**.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| A typed `departures` line in the view (the board's lines) | S21 RT-c, which defines the record (R-RTc-g-1, Q-S29g-4). Until then the record, if any, is reached only by `inspect` |
| A typed "service state" (`window: open/shut`, `board: none`) in the view | never: INV-S27-2 and `ARC-108` item 1 ("the agent view renders the same Observation"). The agent decides from the disclosed facts, as an LM seat does |
| `roles` on `ThingLine`, `PersonLine` or `InspectResult` | not needed by §9.4: places reach an agent as `me.place` and `exits[].leads_to`. A later PR may add it with the same `roles_of` |
| The 2D and 3D station rule (`station_state.gd`) | S29-e, S29-f |
| The station content, the `settlement:` key, `CORE_CONCEPTS.md` §6.5 | S29-a |
| The newsstand (`station-news`, `newspaper` sold in the hall) | S29-b |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `worlds/`, `sdk/`, `cognition/` | never in this PR (I-sg-1, I-S29-1) |

---

## 3. Audit (`origin/main @ 44f050e3`, 2026-10-19)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `PlaceLine { ref: Ref, labels: list[WorldText] }`; `labels` doc: "The place's tags, as its world author wrote them" | `clients/agent/src/mineworld_agent/models.py:185-190` | `roles` is added before `labels`; `labels` is unchanged (Q-S29g-1) |
| A-2 | `MeLine.place: PlaceLine \| None`; `ExitLine.leads_to: PlaceLine` | `models.py:289-296, 333-338` | Both carriers get `roles` with one model change |
| A-3 | `_exits` builds `leads_to` from movement's `passages` record: `PlaceLine(ref=…, labels=_labels(to_tags))` | `view.py:237-252` | The exit's roles come from `to_tags` (`ARC-82`), which the server already sends |
| A-4 | `render` builds `me.place` from the place entity's `tags`: `PlaceLine(ref=…, labels=_labels(place_entity.tags) if … else [])` | `view.py:270-278` | The second call site of `place_line` |
| A-5 | `_labels(tags)` wraps each tag as `WorldText(kind="label")` in the given order; nothing is dropped | `view.py:144-145` | `labels` stays the full list (I-sg-3) |
| A-6 | The module doc: "A view is that one `Observation` and nothing else (INV-S27-2) … The only pack payloads read by name are the ones a client knows … Anything else is carried, uninterpreted, by `inspect`" | `view.py:1-12` | A `departures` record is "anything else": carried by `inspect`, never read (§5.5) |
| A-7 | `components_of(entity, budget)` returns every disclosed component of an entity, in order, its payload cut only at the 4 KiB `inspect` budget, as `ComponentPayload { component, envelope }` | `view.py:567-575`; `INSPECT_BYTES` `view.py:70` | The oracle for "shown only as disclosed" (U-4) |
| A-8 | `inspect` resolves a ref in the newest observation (`_listed`) and returns `components_of(entity)`; the seat's own place is an entity of the observation | `player.py:367-382, 670-678`; `view.py:263-267` | An agent (and the test) reads the hall's `shop` record by `inspect(me.place.ref)` |
| A-9 | Every string outside `WorldText` must carry a pattern, enum or const; `OPERATION_MODELS` count is 23 | `tests/test_world_text.py:42-54, 74` | `RoleTag` is a `Literal` (an enum in the schema), so the test passes unchanged; no operation model is added, so 23 stays |
| A-10 | Every model and field has a schema description (`use_attribute_docstrings`) except an `UNDOCUMENTED` list that may only shrink | `models.py:128-135`; `tests/test_descriptions.py` | `roles` gets a field docstring |
| A-11 | `mineworld-agent schema` must print exactly `tests/golden/tools.json`; `PlaceLine` appears in it 9 times | `tests/test_cli.py:14-27`; `grep -c PlaceLine tests/golden/tools.json` = 9 | The golden is regenerated in C2 and read whole |
| A-12 | Nine tools; `act` has five kinds; `ACTS` is the only kind → operation binding | `mcp_server.py:122-150` | Unchanged (I-sg-4) |
| A-13 | The real-binary harness runs a repository world by name or a scratch copy by absolute path; a scratch copy is built by the test with `shutil.copytree` and file writes | `tests/realserver.py:55-57`; `tests/test_mcp_same_answers.py:46-66` | W4 and W5 (§8) are built the same way |
| A-14 | `walk(session, transcript, handle, destination)` walks to a place and waits until the seat is in it; `call`, `Transcript`, `adapter`, `PATIENCE` are importable from `test_play_mcp` | `tests/test_play_mcp.py:94-148, 253-281`; reused by `test_mcp_same_answers.py:30` | The scenario reuses them; no new harness |
| A-15 | `market-town` and `social-cafe` have seats `visitor`, `wanderer`, `alice`, …; `lakeside` has only person seats (`alba`, `bruno`, …); `mineworld server` drives no seat unless `--town` or `--agent` is given | `worlds/*/world.yaml` `seats:`; `docs/reference/cli.md:21-22` | W3 joins `alba` with `--grant alba`; nobody else moves |
| A-16 | `real_server` tests set `pytestmark = pytest.mark.real_server`; a job without the binary deselects them by name; they never skip | `clients/agent/pyproject.toml:54-61`; `tests/test_play_market_town.py:65` | The new real-binary file is marked the same way |
| A-17 | The recorded fixture `market-town-street.json` is the street with passages to every place; S29-a's C4 re-records the agent fixtures after it adds the station | `tests/fixtures/SOURCE.md`; `pr-s29a-town-contract.md` A-19, C4 | U-1 reads the re-recorded street; C0 checks it holds an exit with `to_tags` ⊇ {`station`} |
| A-18 | S29-a (DRAFT rev. 3): the station group is `station` (`station`, `public`, `landmark` in the two towns), `station-tickets` (`station`, `ticket-office`, `public`), `station-platform` (`station`, `stop`, `platform`, `public`, `outdoor`); Lakeside's hall has no `landmark`, and `pier` is tagged `landmark`; no shop at `station-tickets` | `pr-s29a-town-contract.md` §5 (place files), §14 rulings; step-29 §4.3 | The scenario's expectations per world (§8) |
| A-19 | S29-a C1 writes `CORE_CONCEPTS.md` §6.5 "Role tags" with step-29 §14.1's text: "The role tags are: `station`, `stop`, `platform`, `ticket-office`, `landmark`." | `pr-s29a-town-contract.md` C1 step 3; step-29 §14.1 | The single authority for the vocabulary; U-5 reads that sentence |
| A-20 | S29-a's I-a-7: "A role tag grants no behaviour: no System, client or cognition code branches on `station`, `stop`, … Checked by I-a-1's diff scan; `git grep -n '"ticket-office"\|"landmark"' -- systems clients cognition` empty" | `pr-s29a-town-contract.md` §4, I-a-7 | `roles.py` will contain those strings (F-S29g-1) |
| A-21 | Economy discloses a place's `shop { operator, listed }` to whoever perceives the place; `buy` is offered only where a `Shop` is; `disclosed.shop_prices(place)` reads it for `OfferLine.price` | `systems/economy/src/{component.rs:96-130, system.rs:236-262}`; `clients/agent/src/mineworld_agent/disclosed.py:213-224` | "Window shut" in the agent view is: no `shop` component on the ticket office, no `buy` offer there |
| A-22 | An organization declares its shop with `economy: { wallet, shop: { at: <place>, prices: { <item>: <minor> } } }`; an item file is `tags: […]` plus `item: { category, name }` | `worlds/market-town/organizations/corner-store.yaml`; `worlds/market-town/items/newspaper.yaml` | W5's planted ticket shop (§8) |

### 3.2 Findings

- **F-S29g-1 — S29-a's I-a-7 check would flag this PR's vocabulary file (against the literal wording of I-a-7).**
  `roles.py` holds the strings `"ticket-office"` and `"landmark"`, so I-a-7's grep over `clients` is no longer
  empty once S29-g lands. The same holds for S29-d/S29-e (3D binds "by role tag", SD-S29-8) and S29-f (2D binds
  `facade:<tag>`). None of them **branches behaviour** on a role tag: they order lines or choose a drawing. The
  invariant's intent ("a role tag grants no behaviour; behaviour comes only from Systems") holds; its grep is too
  wide. I-a-7 is S29-a's own diff scan at S29-a's head, so nothing breaks retroactively, but the wording should say
  what is meant. → **Q-S29g-2**; proposed wording in §15.
- **F-S29g-2 — The step's "no offers" at the ticket window cannot hold literally (against step §9.4 step 4).** An
  agent's offers in any place include offers about **people present** (`talk`, invitations) and about **its own
  holdings** (`give`, `consume` in Market Town). If Bob happens to be at the ticket window, `talk` to Bob is
  offered there, and that is the truth. What the step means is "the ticket window sells nothing": no `buy` offer
  and no `shop` disclosed. → **Q-S29g-3** (recommended reading; the step's AC-S29-5 is unchanged).
- **F-S29g-3 — The step's "`buy newspaper`" needs S29-b, which is not designed.** S29-g depends on S29-a only.
  The scenario therefore asserts the **rule**, not the item: "the `buy` offers in the hall name exactly the items
  the hall's `shop` record lists; with no `shop`, there is no `buy`". On Market Town before S29-b that is "no
  `buy`"; after S29-b it is "`buy` the newspaper", with no test change. C0 records whether S29-b has merged and,
  if so, the newspaper arm also checks the item's catalogue name (SD-S29g-4).
- **F-S29g-4 — The agent view does not apply `station_state.gd` (consistent with `ARC-108`, recorded so no reader
  expects it).** Step §5.1 says "The rule is the same in both clients and in the agent view. It is written once in
  `clients/shared/station/station_state.gd`." `ARC-108` item 1 narrows this: "the agent view renders the same
  Observation". The adapter is Python and has no rule to evaluate: an agent reads the same facts the rule reads
  (`shop` on the ticket office, `departures` when it exists) and decides itself. A typed "service state" line would
  be the adapter inferring a state, which INV-S27-2 forbids. The C1 note on `ARC-108` records this.
- **F-S29g-5 — `departures` has no shape yet.** S21 RT-c (MVP-1, not designed) defines it (step-29 §14.6). A
  reader written now would invent a contract. Until RT-c, a `departures` record is "anything else" (A-6): carried
  verbatim by `inspect`. RT-c adds the typed line (R-RTc-g-1). → **Q-S29g-4**.
- **F-S29g-6 — All three reference towns use the same station keys** (`station`, `station-tickets`,
  `station-platform`; A-18). A scenario that passed on them could still be keyed by name. W4 (§8) is a generated
  world with other keys, a separate landmark and tags in a non-vocabulary order, so the scenario must find
  everything by `roles`.
- **F-S29g-7 — The step's "calls `walk_to` station" is the `act` tool's `walk_to` kind** (`ARC-153`, the brief's
  "typed tools" rule). No kind is added: the scenario uses `walk_to` and `attempt` only.

### 3.3 What does not exist

No `roles` field anywhere in `clients/agent`; no role-tag vocabulary in Python; no station in any world on the
base (S29-a); no `departures` record or System (S21); no agent test that explores by tags.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| The vocabulary | read it at run time from a shipped data file; ask the server for it | **Neither.** A data file is a second copy in another format; a server route is a protocol change (I-S29-1). A `Literal` in the adapter, held equal to `CORE_CONCEPTS.md` §6.5 by a drift test (U-5), is one copy checked against the authority |
| The scenario harness | a new MCP driver | **Reuse** `test_play_mcp`'s `adapter`, `call`, `walk`, `Transcript` (A-14) |
| Scratch worlds | a new fixture tree under `tests/` | **Reuse** the `copytree` + write pattern of `test_mcp_same_answers.py` (A-13); W4 is written whole by the test |

---

## 5. Design

### 5.1 Terms, from plain to exact

| Term | Plain | Exact |
| --- | --- | --- |
| **tag** | a word an author puts on a place | an entry of the place's `Tags`, sent in `PerceivedEntity.tags` and in movement's `to_tags` |
| **label** | the tag as the agent sees it | `WorldText { text: <tag>, kind: "label" }`, untrusted, in author order |
| **role tag** | a tag whose meaning the framework fixes | one of `CORE_CONCEPTS.md` §6.5's list: `station`, `stop`, `platform`, `ticket-office`, `landmark`. It grants no behaviour |
| **roles** | the place's role tags, as plain codes | `PlaceLine.roles: list[RoleTag]`: each role tag the place carries, exactly matched, once, in `ROLE_TAGS` order |
| **disclosed** | what the world told this seat | a component record in this seat's newest `Observation`, or a fact it perceived (INV-S27-2) |

### 5.2 The model (`models.py`)

```python
class PlaceLine(AgentModel):
    """A place the seat is in or walks to, by reference."""

    ref: Ref
    roles: list[RoleTag] = Field(default_factory=list[RoleTag])
    """The place's role tags (`CORE_CONCEPTS.md` §6.5: station, stop, platform, ticket-office, landmark), in that
    order. A role tag says what the place is; it grants nothing. Empty for most places."""
    labels: list[WorldText] = Field(default_factory=list[WorldText])
    """The place's tags, as its world author wrote them."""
```

- `roles` is declared between `ref` and `labels`, so it comes "ahead of the other tags" in the structured content
  and in the text rendering (pydantic keeps declaration order; `render_text` drops empty fields, so a place with
  no role tag costs zero bytes).
- `labels` is **unchanged**: every tag, role tags included, in author order (Q-S29g-1; the view's rule "nothing
  filtered").
- `RoleTag` is a `Literal`, so the JSON Schema publishes it as an `enum` of five codes. It is the adapter's own
  vocabulary, not world text (INV-S27-12 holds: a string in it can only be one of five values).

### 5.3 The vocabulary (`roles.py`)

```python
ROLE_TAGS: Final = ("station", "stop", "platform", "ticket-office", "landmark")
"""The role tags of `docs/CORE_CONCEPTS.md` §6.5, in its order (ARC-105). test_roles.py holds this tuple equal
to that section's list; a new role tag is added there first."""

type RoleTag = Literal["station", "stop", "platform", "ticket-office", "landmark"]

def roles_of(tags: Sequence[str]) -> list[RoleTag]:
    """The role tags among `tags`: exact, case-sensitive matches only, each once, in ROLE_TAGS order."""
```

- **Exact match.** `"Station"`, `"station-hall"` and `" station"` are not roles. A tag is a role only if it is
  byte-equal to a vocabulary entry. This is how the loader checks them (`Tags::contains`, S29-a A-6).
- **Order.** Vocabulary order, not author order, so two worlds that tag the same place differently ordered give
  the same `roles` (U-2).
- **No world knowledge.** Nothing in `roles.py` names a place key, a world or a town. The function takes tags
  only. A test (U-6) asserts that `roles_of` and `place_line` give the same result whatever the place's key.
- **`RoleTag` and `ROLE_TAGS` agree.** U-5 asserts `get_args(RoleTag) == ROLE_TAGS`.

### 5.4 The rendering (`view.py`)

```python
def place_line(refs: RefTable, place: EntityId, tags: Sequence[str]) -> PlaceLine:
    """A place by reference, with its role tags first and every tag as a label (§9.4 of step-29)."""
    return PlaceLine(ref=refs.ref(place), roles=roles_of(tags), labels=_labels(tags))
```

- `_exits`: `leads_to=place_line(refs, to, labels)` where `labels` is today's filtered `to_tags` string list
  (`view.py:244-249`).
- `render`: `place=None if here is None else place_line(refs, here, place_entity.tags if place_entity is not None
  else ())` (`view.py:273-278`). This keeps the edit inside the `MeLine(...)` call to the one `place=` argument
  (the order rule, §5.9).
- References are assigned in the same order as today (`refs.ref(place)` is the first call in `place_line`, as it
  was in the inline constructor), so no `E`-number moves (U-1 checks refs equal the pre-change render).
- Deltas need nothing: `me` and `exits` are compared whole (`view.py:488, 493`).

### 5.5 What the adapter does **not** do (the adversarial)

| It never | Because | Held by |
| --- | --- | --- |
| reads a `departures` record, or any record it does not know by name | A-6; F-S29g-5 | U-4: an observation with a planted `departures` component renders a view byte-identical to the same observation without it; `inspect` returns the component verbatim |
| writes "No service yet", "Closed", "open", "shut" or any service word into a view | INV-S27-2; `ARC-108` item 1 | U-4 and the models' closed schema (no field exists for it); M-sg-3 |
| hides, adds or reorders an offer because of a role tag | `ARC-105`: a role tag grants no behaviour | U-3: the same observation with all role tags stripped renders the same `offers`, `here.people`, `here.things` and `me` (except `roles`/`labels`); M-sg-4 |
| finds a place by key | F-S29g-6 | W4 (§8); U-6 |
| filters `labels` | the view's rule "nothing filtered" | U-1: `labels` equals the tags in author order |

### 5.6 Spatial requirements, rules and refusals

None new. Walking into the hall, the ticket office and the platform is movement's `walk-to`, judged by the server
as for any client; an unreachable place is the server's refusal, shown as today. The scenario asserts **0
refusals** along the walks (step AC-S29-3's rule, for the agent).

### 5.7 Ownership, persistence, determinism

The adapter owns no world state. `roles` is a pure function of tags already in the observation; it is
deterministic and order-stable. Nothing persists. The server path is the one a human client uses (`walk-to`,
`buy` through `offers.request`); the agent gets no extra route (INV-S27-1, -3).

### 5.8 Both clients

The 2D and 3D clients draw the station from the same Observation (S29-e, S29-f). The agent view's facts are the
same facts their shared rule reads: the ticket office's `shop` (absent → shut), the hall's `shop` (kiosk), and a
`departures` record (absent → "No service yet"). AC-sg-2 checks the agent half on the same worlds S29-e/S29-f
check theirs.

### 5.9 Files shared with S31-g, and the order rule

S31-g (DRAFT, `design-s31g` @ 5e7e6e23) changes `clients/agent` heavily. The overlap:

| File | S29-g changes | S31-g changes | Conflict? |
| --- | --- | --- | --- |
| `models.py` | `PlaceLine` (lines 185-190): one field; one import of `RoleTag` | `MeLine`, `PersonLine`, `OfferLine`, `ItemCount`, `View`, `ViewDelta`, `ListName`, new models, act bodies | different classes; a textual merge applies unless the import block collides |
| `view.py` | `place_line` (new, after `_labels`); `_exits` one line; `render` one argument (`place=`) | `render` (`MeLine(...)` gains `wealth`, `asks`, `subscriptions`; `wallet` changes), `offer_line`, paging, deltas, `Caps` | **yes, adjacent lines inside `render`'s `MeLine(...)` call** |
| `__init__.py` | two exports | many exports | same `__all__` list |
| `tests/golden/tools.json` | regenerated | regenerated | **always** (generated) |
| `README.md` | one line | one line | adjacent lines possible |

**The order rule.** The two PRs do not depend on each other; either may merge first. The one that merges
**second** carries the reconciliation, in a commit of its own before its close-out:

1. Rebase (or merge `origin/main`) onto the first. In `models.py`, `view.py`, `__init__.py` and `README.md`,
   keep both sides' hunks; S29-g's only hunk inside `MeLine(...)` is `place=…`, S31-g's are its own arguments.
2. **Never merge `golden/tools.json` textually.** Take either side, then regenerate it from the rebased head
   (`uv run --locked mineworld-agent schema --out clients/agent/tests/golden/tools.json`) and read it whole.
3. Run `test_cli.py`, `test_world_text.py`, `test_descriptions.py`, `test_view.py` and the second PR's own unit
   files, one at a time. `OPERATION_MODELS`' count is S31-g's (S29-g adds no operation model).
4. Record the reconciliation in the second PR's ledger.

---

## 6. Invariants this PR holds

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-sg-1 (I-S29-1) | No path changes outside `clients/agent/`, `docs/decisions/ARC-108.md`, `docs/decisions/INDEX.md`, this plan and its handoff (and `docs/reference/agent-tools.*` if S27-f merged) | C4 diff scan (`git diff --name-only origin/main...HEAD`) |
| I-sg-2 (INV-S27-2, I-S29-3) | Every `roles` entry comes from this seat's own observation (the place's `tags` or movement's `to_tags`); the adapter reads no other record for the station and shows no service state | U-3, U-4; AC-sg-3 |
| I-sg-3 | `labels` is unchanged: every tag, in author order, as `WorldText` | U-1 |
| I-sg-4 (`ARC-153`) | Nine tools; five `act` kinds; `ACTS` unchanged | `test_mcp_tools.py` (unchanged, run in C2) |
| I-sg-5 (INV-S27-12) | `roles` is a `Literal` enum; no new unconstrained string | `test_world_text.py` (unchanged, run in C2) |
| I-sg-6 (`ARC-105`) | A role tag changes no offer, person, thing or exit in the view | U-3; M-sg-4 |
| I-sg-7 | `ROLE_TAGS` equals `CORE_CONCEPTS.md` §6.5's list, and `RoleTag` equals `ROLE_TAGS` | U-5 |
| I-sg-8 | No place key, world name or town name appears in `clients/agent/src` because of this PR | C4: `git diff origin/main...HEAD -- clients/agent/src` contains none of `station-tickets`, `station-platform`, `market-town`, `social-cafe`, `lakeside`, `pier` |

---

## 7. Test ownership

| Layer | Owns | Where (`clients/agent/tests/` unless named) |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict on `clients/agent` | CI `fast` |
| Unit (recorded and planted observations) | U-1 … U-6 below | `test_roles.py` |
| In-memory MCP | the tool list, schemas, descriptions, world text | `test_mcp_tools.py`, `test_descriptions.py`, `test_world_text.py`, `test_cli.py` (existing; run, not edited, except the golden file) |
| Real binary (`real_server`) | AC-sg-1 … AC-sg-4 over stdio on W1 … W5 | `test_play_station_mcp.py` |
| Gate 1 | NOT REQUIRED: no model; the agent's model is the user's | — |

The unit tests in `test_roles.py`:

- **U-1 (recorded).** On `market-town-street.json` (re-recorded by S29-a, A-17): every exit's `leads_to.roles`
  equals `[t for t in ROLE_TAGS if t in to_tags]` computed from the raw fixture; the exit to the hall has
  `roles == ["station", "landmark"]`; every `labels` equals the raw `to_tags` in order; every ref equals the
  ref the same render gives with `roles` ignored (no `E`-number moved).
- **U-2 (planted).** A place entity with tags `["outdoor", "landmark", "Station", "station-hall", "stop",
  "platform", "station", "station"]`: `me.place.roles == ["station", "stop", "platform", "landmark"]`
  (exact match, deduplicated, vocabulary order); `labels` has all eight in that order.
- **U-3 (planted, roles change nothing else).** One recorded observation, rendered twice: as recorded with role
  tags planted on its place and on every passage's `to_tags`, and with every role tag removed. The two views'
  `offers`, `here.people`, `here.things`, `exits[].door`, `exits[].leads_to.ref` and `me` minus `place` are equal.
- **U-4 (planted `departures`; the step's adversarial).** Plant on the observer's place a component
  `departures` with a payload in a plausible shape (`{"lines": [{"line": "coast", "next": 30600}]}`, chosen so it
  names no entity). (a) The `View` JSON equals byte for byte the `View` of the same observation without the
  component. (b) `components_of(place)` lists `departures` with `envelope.payload` equal to the planted payload
  and `truncated` false. (c) With no `departures` and the place tagged `station`, the `View` JSON contains none of
  the strings `service`, `departure`, `closed`, `timetable` (case-insensitive) outside world text.
- **U-5 (drift).** Read `docs/CORE_CONCEPTS.md` from the repository root, find the line containing
  `The role tags are:`, extract the back-quoted words in order: they equal `ROLE_TAGS`; and
  `get_args(RoleTag.__value__) == ROLE_TAGS`.
- **U-6 (key-blind).** Two planted observations that differ only in the place's entity id and display name give
  equal `roles`.

Oracles are literal values written in each test, the raw recorded or planted payloads, the server's answers,
and `inspect`'s verbatim components; never a value the adapter reports about itself (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`; run at the milestone)

**Worlds.** No criterion uses Market Town alone.

- **W1 `market-town`** — the repository world after S29-a (and S29-b, if merged at C0). Seat `visitor`
  (`--grant visitor`), starting in the café.
- **W2 `social-cafe`** — the repository world after S29-a: no economy; hall = landmark. Seat `visitor`.
- **W3 `lakeside`** — the repository world after S29-a: the halt (hall without `landmark`) and `pier` (the
  landmark), no economy, no `bodies`. Seat `alba` (`--grant alba`, A-15).
- **W4 `crossing`** — a **generated** world, written whole by the test into `tmp_path` (≈ 80 lines of YAML in the
  test file): `systems: [presence, movement, naming]`; places `green` (tags `[outdoor, public]`, the seat's
  start), `depot` (tags `[public, station]`, author order reversed), `booking` (`[public, ticket-office,
  station]`), `siding` (`[outdoor, platform, public, stop, station]`), `clocktower` (`[public, landmark]`);
  passages green↔depot, depot↔booking, depot↔siding, green↔clocktower; `settlement: { kind: town, station:
  depot, landmark: clocktower }`; one person `rook` with `location: { place: green }` and seat `rook`. No key of
  any reference world appears in it. The loader's S29-a checks run on it at server start, so it is also a valid
  town by the contract.
- **W5 `market-ticketed`** — a scratch copy of W1 (A-13) with one item file `items/day-ticket.yaml` (`tags:
  [ticket]`, `item: { category: goods, name: Day ticket }`), one organization `organizations/rail-co.yaml`
  (`economy: { wallet: 100000, shop: { at: station-tickets, prices: { day-ticket: 450 } } }`, `holdings: {
  day-ticket: 20 }`), both added to `world.yaml`'s `items:` and `organizations:` lists. This is X-5's planted
  shop. (If S31-a has merged with another shop shape, C0 pins the shape from merged code and records it; the
  criterion is unchanged.)

Servers run without `--town` (nobody else moves, A-15) and with `--time-scale 10` so walks end within
`WALK_PATIENCE`. Places are found by **exploring**, never by key: `explore(role)` walks from the current place
breadth-first through `exits`, visiting each place once, until an exit whose `leads_to.roles` contains `role`
appears, then walks to it (at most 12 places; a longer search fails the test by name).

| Id | Criterion | Worlds | Step ref |
| --- | --- | --- | --- |
| AC-sg-1 | **The station is found by its roles.** From the start, `explore("station")` finds an exit whose `leads_to.roles` contains `station`; on W1 and W2 the same exit's roles are exactly `["station", "landmark"]`, on W3 and W4 exactly `["station"]`. `act walk_to` it: accepted, and `me.place.roles` equals that exit's `roles`. From the hall, the exits with `ticket-office` and with `stop`+`platform` in their roles are walked to; there `me.place.roles` is `["station", "ticket-office"]` and `["station", "stop", "platform"]`. On W3 and W4, `explore("landmark")` from the platform reaches a place whose `me.place.roles` is `["landmark"]`. Every walk is accepted and arrives; 0 refusals | W1, W2, W3, W4 | step §9.4 steps 1, 2, 5; AC-S29-3's rule for the agent |
| AC-sg-2 | **The window is shut and the kiosk follows the shop rule (agent half of AC-S29-5).** At the ticket office: `inspect(me.place.ref)` lists no `shop` component, and no offer has `action_type == "buy"`. In the hall: if `inspect` lists a `shop` component, the number of `buy` offers equals the number of items its `listed` payload names, and the multiset of the offers' prices (`price`, or `price.minor` once S31-g has merged) equals the listed prices; if it lists none, no `buy` offer exists. On W1 with S29-b merged: the hall's `shop.listed` names an item id that the hall's `item-catalogue` record (same `inspect`) names `Newspaper`; on W2, W3, W4: no `buy` in the hall | W1, W2, W3, W4 | step §9.4 steps 3, 4; AC-S29-5; Q-S29g-3 |
| AC-sg-3 | **Nothing pretends (agent half of AC-S29-5's board).** Across every view and `inspect` result taken in AC-sg-1 and AC-sg-2: no component named `departures`, and no string `service`, `departure`, `timetable` or `closed` outside `WorldText` (the board reads "No service yet" only in the 2D and 3D clients' own words; the agent sees the absence). U-4 (unit) covers the presence half: a planted `departures` record is shown only through `inspect`, verbatim | W1 … W4; U-4 | step §11.1 adversarial; X-7's agent half |
| AC-sg-4 | **A planted ticket shop is shown exactly as disclosed (X-5's agent half).** On W5 at the ticket office: `inspect` lists a `shop` component whose payload lists the day ticket at 450; the view has one `buy` offer naming it; `act attempt` that offer is `accepted`, and `me.holdings` gains the ticket. On W1 at the same place: neither. No adapter file differs between the two runs | W5, W1 | X-5; `ARC-108` ("the window opens with no client change") |
| AC-sg-5 | **The surface.** The tool list is nine tools; `act` has five kinds; every result in AC-sg-1 … AC-sg-4 is ≤ 8 KiB as JSON and as text; the `tools/list` size before and after C2 is recorded | all | I-sg-4; S27-d I-d-4 |

### 8.1 Planted mutations (run at the milestone; each turns its test red by name, then is restored from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-7 (agent analogue) | `place_line` appends `WorldText("No service yet", kind="label")` when `roles` contains `station` | U-4 (c); AC-sg-3 |
| M-sg-1 | `roles_of` keeps author order | U-2 |
| M-sg-2 | `roles_of` matches case-insensitively or by prefix (`startswith`) | U-2 |
| M-sg-3 | `render` reads a `departures` component into the hall's `labels` | U-4 (a) |
| M-sg-4 | `render` drops `buy` offers when `me.place.roles` contains `ticket-office` | U-3; AC-sg-4 (W5) |
| M-sg-5 | `_exits` builds `leads_to` without `roles` (only `me.place` gets them) | U-1; AC-sg-1 |
| M-sg-6 | `ROLE_TAGS` loses `landmark` | U-5; AC-sg-1 (W1, W3) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and its evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit (Haiku implementer: follow it literally)

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s29g`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full.
2. Make the numbered steps in order. Do not do a later commit's work.
3. Run only the test files this commit writes or names, one at a time: `uv run --locked pytest
   clients/agent/tests/<file>`. Record the count, the wall time, PASS / FAIL / INCONCLUSIVE.
4. Static: `uv run --locked ruff check clients/agent`; `uv run --locked ruff format --check clients/agent`;
   `uv run --locked pyright clients/agent`.
5. Review: answer the commit's review item in §16 in one or two sentences.
6. `git status --short`: every changed path is in the commit's **Files**, or revert it or stop and record why.
7. Update §16 and `handoff-s29g.md` in the same commit.
8. Stage by explicit path; `git commit -m "<message>"` ending with the line
   `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` (or the implementing model's own line).
   Never `git add -A`, heredoc writes, `sed -i` or `python3 -c`.
9. A material stop (§12) is reported, not improvised around.

#### C0 — Base, prerequisites, shapes (Markdown only)

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s29g.md` (new).
- [ ] 1. Record the base commit; quote the primary's dispatch (Milestone F's checklist handed over).
- [ ] 2. Confirm S29-a has merged: `git log --oneline origin/main | grep -i "s29-a"` and `git grep -n "settlement:"
  worlds/market-town/world.yaml`. Missing → material stop.
- [ ] 3. Confirm `docs/CORE_CONCEPTS.md` has the line `The role tags are:` with exactly the five tags of §5.3;
  a different list → record it and use the merged list (bounded), unless it drops a tag this design names
  (material stop).
- [ ] 4. Confirm the re-recorded `tests/fixtures/market-town-street.json` has an exit whose `to_tags` include
  `station` (A-17). Missing → material stop (S29-a's C4 owns the re-record).
- [ ] 5. Record whether S29-b, S31-a, S31-g and S27-f are merged; choose: the newspaper arm of AC-sg-2 (S29-b),
  W5's shop shape (S31-a), the reconciliation of §5.9 (S31-g merged first), `docs/reference/agent-tools.*`
  (S27-f).
- [ ] 6. Re-read A-1 … A-22 on the base; record moved lines (bounded).
- [ ] Implementation: steps 1–6. [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3
  scripts/check_decision_ids.py`. [ ] Review: every hard dependency holds.
- **Commit message:** `docs(plan): S29-g — open the ledger at the base`

#### C1 — The decision note

- **Files:** `docs/decisions/ARC-108.md`, `docs/decisions/INDEX.md` (only if the checker rewrites it), §16.
- [ ] 1. Append to `docs/decisions/ARC-108.md` a heading `## ARC-108 note — the agent view (S29-g)` and three
  bullets: (a) the agent view carries `PlaceLine.roles`, the role tags of `CORE_CONCEPTS.md` §6.5 in that order,
  ahead of `labels`, and nothing else about the station; (b) it shows no service state: the agent reads the same
  disclosed facts the clients' rule reads (`shop` on the ticket office and in the hall; `departures` once S21
  defines it), and a record the adapter does not know is carried verbatim by `inspect` (F-S29g-4, F-S29g-5);
  (c) as Q-S29g-2 rules: "a role tag grants no behaviour; a client may order lines or choose a drawing by role tag"
  (F-S29g-1).
- [ ] 2. `python3 scripts/check_decision_ids.py --write-index`, then `python3 scripts/check_decision_ids.py`.
- [ ] Implementation: steps 1–2. [ ] Validation: both scripts PASS. [ ] Review: the note adds no requirement the
  record or the step does not already state, except Q-S29g-2's ruled wording.
- **Commit message:** `docs(decisions): ARC-108 note — the station in the agent view (S29-g)`

#### C2 — `roles`: the vocabulary, the model, the rendering, the unit tests

- **Files:** `clients/agent/src/mineworld_agent/{roles.py (new), models.py, view.py, __init__.py}`,
  `clients/agent/tests/{test_roles.py (new), golden/tools.json}`, `clients/agent/README.md`, §16.
- [ ] 1. `roles.py`: module docstring (what a role tag is, the authority, "grants nothing"); `ROLE_TAGS`,
  `RoleTag`, `roles_of` exactly as §5.3.
- [ ] 2. `models.py`: `from mineworld_agent.roles import RoleTag`; `PlaceLine.roles` as §5.2, between `ref` and
  `labels`, with its docstring.
- [ ] 3. `view.py`: `place_line` after `_labels` (§5.4); `_exits` and `render` call it. No other line changes.
- [ ] 4. `__init__.py`: export `RoleTag`, `ROLE_TAGS`.
- [ ] 5. Record the size of `golden/tools.json` (`wc -c`), regenerate it with `uv run --locked mineworld-agent
  schema --out clients/agent/tests/golden/tools.json`, read the diff whole: only `PlaceLine` and the new
  `RoleTag` enum change. Record the new size.
- [ ] 6. `test_roles.py`: U-1 … U-6 (§7), using `recorded.py`'s `observation`, `raw`, `entity`, `component`
  helpers for planted records (the `test_view.py` precedent).
- [ ] 7. `README.md`: one line under the view's description: "Places carry `roles` (station, stop, platform,
  ticket-office, landmark) ahead of their labels."
- [ ] Implementation: steps 1–7. [ ] Validation, one file at a time: `test_roles.py`, `test_view.py`,
  `test_cli.py`, `test_world_text.py`, `test_descriptions.py`, `test_mcp_tools.py`; static (§9.0 step 4).
  [ ] Review: refs unchanged (U-1); `labels` unchanged; no key or world name in `src` (I-sg-8).
- **Commit message:** `feat(agent): places show their role tags ahead of their labels (S29-g)`

#### C3 — The scenario over MCP (real binary)

- **Files:** `clients/agent/tests/test_play_station_mcp.py` (new), §16.
- [ ] 1. Module docstring (the scenario, the worlds, "found by roles, never by key", the oracles).
  `pytestmark = pytest.mark.real_server`.
- [ ] 2. Helpers: `explore(session, transcript, handle, role) -> dict` (§8, breadth-first, ≤ 12 places, reusing
  `test_play_mcp.walk`); `roles_here(view)`; `buys(view)`; `components(session, transcript, handle, ref)` (an
  `inspect` call returning `{component: payload}`).
- [ ] 3. World builders: `write_crossing(root) -> Path` (W4, whole files, §8) and `ticketed_copy(root) -> Path`
  (W5, `copytree` plus three file writes, A-13).
- [ ] 4. One test per world, each starting `Server(<world>, "--grant", <seat>, "--time-scale", "10")` through the
  `mineworld` fixture, running AC-sg-1 … AC-sg-3 (W1 … W4) and AC-sg-4 (W5 and the W1 control), all through the
  MCP adapter over stdio (`test_play_mcp.adapter`). AC-sg-5's size check runs on every answer via `Transcript`.
- [ ] 5. Build the binary once (`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh cargo build -p
  mineworld-cli`), then run **only W4's test** locally (`uv run --locked pytest
  clients/agent/tests/test_play_station_mcp.py -k crossing`). The other worlds run at the milestone (charter).
- [ ] Implementation: steps 1–4. [ ] Validation: step 5, recorded; static. [ ] Review: no place found by key; no
  sleep where a `wait` exists; every assertion's oracle is the server's answer or `inspect`.
- **Commit message:** `test(agent): an agent finds the station by its roles over MCP, in five worlds (S29-g)`

#### C4 — Close-out to MERGE-READY

- **Files:** this file (§16), `handoff-s29g.md`; `docs/reference/agent-tools.*` if S27-f merged (regenerated by
  its documented command); the §5.9 reconciliation commit first, if S31-g merged before this PR.
- [ ] I-sg-1 and I-sg-8 diff scans recorded; push `mvp0/pr-s29g`; open the PR (body ends with the Claude Code
  line); `fast` green on the exact head; report **MERGE-READY** with the PR number and head SHA.
- **Commit message:** `docs(plan): S29-g close-out`

---

## 10. Requirements this PR places on other PRs

- **R-S29a-g-1 (on S29-a).** `CORE_CONCEPTS.md` §6.5 keeps the role-tag list on one line beginning `The role tags
  are:` with each tag back-quoted (U-5 reads it); C4's re-record of the agent fixtures includes the street's
  passage to the hall.
- **R-S29b-g-1 (on S29-b).** Nothing to add: AC-sg-2's hall arm checks the newsstand by the shop rule as soon as
  S29-b's shop exists. S29-b's own checkpoint may cite AC-sg-2's W1 arm.
- **R-RTc-g-1 (on S21 RT-c).** When `transport` discloses `departures` (step-29 §14.6), the same PR adds a typed
  agent line for it (`View.departures` or on `PlaceLine`), read by name per `view.py`'s rule, and replaces U-4 (a)
  with a test of that line. Until then U-4 holds.
- **R-S31g-sg-1 (on S31-g).** The order rule of §5.9 applies to whichever of S29-g and S31-g merges second.

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S29g-1 | `roles` in vocabulary order, deduplicated, exact match | stable across authors; matches the loader's `Tags::contains` |
| SD-S29g-2 | `roles` is a typed `Literal` list, not `WorldText` | a fixed framework vocabulary; INV-S27-12 allows it only as a constrained code |
| SD-S29g-3 | One helper `place_line` for both call sites | one rule; keeps the `render` edit to one argument (§5.9) |
| SD-S29g-4 | AC-sg-2 asserts the shop rule, and names the newspaper only when S29-b merged | F-S29g-3 |
| SD-S29g-5 | Places are found by exploration over `exits`, never by key | F-S29g-6; it is how a generic agent plays |
| SD-S29g-6 | `--time-scale 10` for the real-binary runs | the platform is ≈ 100 m of walking from the café; the S27-d harness's `WALK_PATIENCE` is 90 s of wall time |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S29-g — the station in the agent (MCP) view
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s29g-agent-view.md
RELATED / BINDING DOCS:  step-29-station-landmark.md (§§3, 5, 9.4, 9.5, 10, 11); pr-s29a-town-contract.md (§§4,
                         5, 14); pr-s27c-player-library.md; pr-s27d-mcp-stdio.md; pr-s27d1-single-act.md;
                         docs/decisions/ARC-105.md, ARC-108.md, ARC-153.md; step-27-agent-players.md (§§3, 5);
                         pr-s31g-agent-view.md (§2.1; the shared files); CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; LANE_LEAD_CHARTER.md ("SPEED FIRST")
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s29g, its own, held by one session
BRANCH:                  mvp0/pr-s29g, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-29 §11.2) AND S29-a merged (S27-c
                         #204, S27-d #239 and ARC-153 #261: merged) AND this design frozen AND the primary
                         session's dispatch                                                  (step; operator)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–3, as §14's rulings shape them                     (operator, pending)
FROZEN INVARIANTS:       INV-S27-1 … 13; ARC-153 (nine tools, five kinds); I-S29-1, I-S29-3; §6 I-sg-1 … I-sg-8;
                         §8 as written
SEQUENCE:                C0 … C4 (§9)
ALLOWED COMMANDS:        uv run --locked pytest <one file> | ruff | pyright | mineworld-agent schema; cast-slots/
                         with-slot.sh cargo build -p mineworld-cli; target/*/mineworld *; git; gh (pr create/view/
                         checks; never merge); python3 scripts/check_doc_headings.py, check_decision_ids.py
                         (--write-index); mkdir -p; cp; cmp; wc; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; a whole-member or workspace test run
                         (R-LOAD-1); reading any key or .env; calling any model; committing an edit outside I-sg-1;
                         merging
MATERIAL STOPS:          S29-a not merged, or merged without §6.5's line or the re-recorded street fixture; a
                         needed change outside I-sg-1's paths (server, worlds, systems); a 10th tool or a 6th kind;
                         a result over 8 KiB that paging cannot fit; any change to a frozen invariant or acceptance
PLATFORMS:               Linux, macOS, Windows (CI `fast`; the milestone runs the real-binary file on all three)
VALIDATION BUDGET:       per commit: the build and the test files it writes or names, one at a time, plus `fast`
                         on push; locally only W4 of the real-binary file. AC-sg-1 … AC-sg-5 and §8.1's mutations:
                         at the milestone (charter). Gate 1: NOT REQUIRED. No paid call
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s29g.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to `fast` green: authorized           (working rules §21)
                         endpoint: MERGE-READY — pushed, `fast` green on the exact head, reported with PR
                                   number and head SHA                    (charter "SPEED FIRST", 2026-10-18)
                         merge: NOT by this session — the coordinator merges (charter, 2026-10-18)
POST-MERGE SYNC OWNER:   primary: step-29, overall.md, the progress page; implementation session: this ledger
STOP CONDITION:          S29-g MERGE-READY — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S29g-1 | S29-a merges with other keys, tags or a different §6.5 sentence | the scenario finds places by roles (no key); C0 pins §6.5's list; U-5 fails loudly on drift |
| R-S29g-2 | S29-g and S31-g collide in `view.py` and the golden | the order rule (§5.9): the second regenerates the golden and keeps both hunks |
| R-S29g-3 | Walks to the platform are slow on Windows CI | `--time-scale 10`; no `--town`; one seat; a timing failure is INCONCLUSIVE, never re-tuned silently |
| R-S29g-4 | People present at the ticket window make "no offers" fail | the assertion is "no `buy`, no `shop`" (Q-S29g-3) |
| R-S29g-5 | A later role tag (S21's) is added to §6.5 and the adapter lags | U-5 turns red on the PR that adds it; that PR updates `ROLE_TAGS` |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29g-1 [operator — step §9.4 "ahead of the other tags"]** | Should role tags stay in `labels` too (every tag shown, role tags twice), or move out of `labels` into `roles`? | **Keep them in `labels` too.** The view never filters the observation; a few bytes per station place; nothing else changes for places without roles |
| **Q-S29g-2 [operator — S29-a's I-a-7, F-S29g-1]** *(2026-10-19: asked once, as S29-a's Q-S29a-10; its ruling answers this row)* | The adapter must hold the five role-tag strings to render `roles`, which S29-a's I-a-7 grep forbids in `clients/`. Allow it, with the vocabulary held equal to `CORE_CONCEPTS.md` §6.5 by a drift test, and reword I-a-7 as "a role tag grants no behaviour; a client may order lines or choose a drawing by it"? | **Yes.** Ordering is not behaviour; S29-d/e/f bind drawings by role tag the same way; the spec stays the one authority |
| **Q-S29g-3 [operator — step §9.4 step 4, F-S29g-2]** | "No offers" at the ticket window: read it as "no `buy` offer and no `shop` disclosed"? | **Yes.** Offers about people present or the agent's own items are true anywhere; the window's truth is that it sells nothing |
| **Q-S29g-4 [operator — F-S29g-5]** | Add a typed `departures` line now, or leave the record to `inspect` until S21 RT-c defines it? | **Leave it to `inspect` now; RT-c adds the typed line.** A reader written before the record exists would invent its shape |

### 14.1 Rulings

(none yet)

---

## 15. Proposed edits elsewhere (made by the PR named)

- **`step-29-station-landmark.md` §9.4 (primary):** "calls `walk_to` station" → "`act { kind: walk_to }` to the
  exit whose roles contain `station`" (`ARC-153`); step 4's "**no offers**" → "no `buy` offer, and no `shop`
  disclosed" (Q-S29g-3); step 3's "offers include `buy newspaper`" → "the `buy` offers name exactly the hall's
  shop listing (the newspaper in Market Town, after S29-b)" (F-S29g-3).
- **`pr-s29a-town-contract.md` I-a-7 (S29-a's owner, before its freeze):** "A role tag grants no behaviour: no
  System or cognition code branches on a role tag; a client may order lines or choose a drawing by one" and the
  grep limited to `systems cognition` (Q-S29g-2). **Done 2026-10-19** in S29-a revision 4 (§4 I-a-7, F-S29a-16),
  pending its Q-S29a-10, which asks Q-S29g-2 once. R-S29a-g-1 is in its C1 step 3; role tags are now
  `CORE_CONCEPTS.md` §6.5, because §6.4 became "Sit point".
- **`docs/decisions/ARC-108.md` (this PR, C1):** the note of §0.
- **`pr-s31g-agent-view.md` (S31-g's owner):** cite §5.9's order rule in its C8 (R-S31g-sg-1).
- **`overall.md` (primary):** S29-g's state.

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | in C1's commit | [x] base `origin/mvp0/pr-s29a` (S29-a PR #348, head 893a3d94, not yet merged), lane lead implementing directly (sub-agent limit) | [x] doc scripts PASS | [x] S29-a's §6.6 line present; fixture re-record pending (S29-a2) |
| C1 | see `git log` | [x] `ARC-108` note | [x] `check_decision_ids.py --write-index` PASS | [x] adds only Q-S29g-2's ruled wording |
| C2 | see `git log` | [x] `roles.py`, `PlaceLine.roles`, `place_line`, exports, README; `docs/reference/agent-tools.{json,md}` regenerated (S27-f merged: they are the golden) | [x] `test_roles.py` 5 passed, 1 skipped (U-1's hall arm waits for S29-a2's re-recorded fixture); `test_view.py` 15, `test_cli.py` 9, `test_world_text.py` 5, `test_descriptions.py` 3, `test_mcp_tools.py` 15, `test_markdown_reference.py` 4, `test_reference_pages.py` 3 passed; ruff, ruff format, pyright 0 errors | [x] refs unchanged (U-1); labels unchanged; no key or world name in `src` |
| C3 | see `git log` | [x] `test_play_station_mcp.py`: W1 … W4 by roles, W5's planted ticket shop; W4 and W5 generated in `tmp_path` | [x] ruff, ruff format, pyright 0 errors; the real-server runs are the milestone's (D-g-5: no binary; the build slots were saturated) | [x] no place found by key; every oracle is the server's answer or `inspect` |
| C4 | see `git log` | [x] PR opened | [ ] `fast` on the exact head | [x] I-sg-1 and I-sg-8: `clients/agent/src` names no world or place key |

### Evidence

- `agent-tools.json` grew by 51 lines: `PlaceLine.roles` and the `RoleTag` enum only (read whole).

### Deviations (implementation)

- **D-g-1.** Role tags are `CORE_CONCEPTS.md` §6.6 (vf-furnishings took §6.5); every docstring says §6.6. U-5 reads
  the line, not the number.
- **D-g-2.** U-2's planted near miss is `stations`, not `Station`: the contract refuses an uppercase tag, so case
  cannot differ.
- **D-g-3.** U-4 (b) reads `components_of` with a budget large enough for every component of the place; the
  default budget is shared and the street's `passages` uses it up.
- **D-g-4.** The golden is `docs/reference/agent-tools.json` since S27-f (`test_cli.py` `GOLDEN`), not
  `tests/golden/tools.json`; the Markdown page is regenerated with it.
- **D-g-5.** C3's MCP scenario needs S29-a merged and a built binary; it lands after #348 merges (or at the
  milestone if the build slots stay saturated).

### Mutations

(none yet; run at the milestone: X-7's agent analogue, M-sg-1 … M-sg-6 as §8.1 lists them)

### Deviations and findings

F-S29g-1 … F-S29g-7 (§3.2) and SD-S29g-1 … 6 (§11), recorded at design time.

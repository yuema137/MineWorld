# PR 16g1 — Say what you want to say, in 3D

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended) — awaiting the operator's freeze (questions in §9)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-20), DRAFT
Author:                 3D speech lane (Opus), worktree /Users/yuema137/mineworld-worktrees/design-3d-speech,
                        branch docs/pr-design-3d-speech, planning base origin/main @ 0458066a
Approved by / evidence: none yet (DRAFT). Origin: promo v2 finding (2026-10-20) — the 3D client can only say
                        "Hello! A coffee, please.", so AC-15's "she remembers you" can only be shown by telling
                        Alice something in 2D
Implementation base:    origin/main at C0
Start condition:        this design frozen
Execution contract:     §8
Implementer:            Haiku-class for C1–C3 (well-specified client work); Opus reviews
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-15-demo-3d.md`](step-15-demo-3d.md) §22.10 (Q-16d-6 → PR 16g, Q-16d-7 "a
typed line belongs with 16g's text input") · **Relates to:** `AC-13`, `AC-14`, `AC-15`
([`docs/MVP.md`](../../../docs/MVP.md) §9), Milestone D (`tools/cli/tests/milestone_d.rs`), S31-e
([`pr-s31e-clients.md`](pr-s31e-clients.md) F-S31e-9, the 3D price `LineEdit`), SET-a's input gate (D-16d-9)

---

## 0. The problem in one paragraph

In the 2D client, choosing "talk" opens a text line; whatever the player types is sent as the `talk` request's
`utterance`. In the 3D client, pressing **E** (or choosing "talk" in the Tab menu) always sends the fixed
string `SliceIntents.DEFAULT_UTTERANCE = "Hello! A coffee, please."`. So a player cannot tell Alice anything in
3D. AC-15 says "speak to Alice in 2D, then walk up to her in 3D, and Alice knows it happened"; it only works in
that one direction, and the promo cannot show "I told her in 3D, she remembers in 2D". This PR gives the 3D
client the same typed line the 2D client has. The server does not change: both clients send the same `talk`
request with the same payload shape.

## 1. Audit (origin/main @ 0458066a)

| Id | Finding | Where |
| --- | --- | --- |
| A-1 | **2D talk is a typed line.** The menu's `composed` entry asks `intents.input_for("talk")` → `{prompt: "ui.talk.prompt"}`, opens `talk_line.ask(prompt, suggestions, on_done)`; Enter calls `intents.compose(affordance, said)` → `submit("talk", target, {"utterance": said})`. Nothing typed is trimmed, lower-cased or checked; an empty line is sent and the server decides. Esc cancels. `type_and_send` is the harness path through the same signal | `clients/2d/scripts/hud/talk_line.gd`; `clients/2d/scripts/menu.gd:146–159`; `clients/2d/scripts/intents.gd:23–28, 82–100` |
| A-2 | **3D talk is a fixed line.** `KEY_E → talk_to_facing(SliceIntents.DEFAULT_UTTERANCE)`; the menu's `composed` talk calls `intents.compose_offered` → `talk(…, DEFAULT_UTTERANCE, …)`. The 3D client has no `LineEdit` anywhere | `clients/3d-spike/scripts/slice/slice_link.gd:582–595, 831`; `slice/intents.gd:18–24, 76–81`; `slice/offer_menu.gd:11`; `git grep LineEdit clients/3d-spike` → none |
| A-3 | **The protocol already carries free text.** `talk`'s payload is `{"utterance": String}`; `Utterance` rejects an empty line and anything over `UTTERANCE_MAX_BYTES = 480` bytes (UTF-8, so about 160 Chinese characters), never truncates | `systems/conversation/src/utterance.rs:13, 37`; `error.rs:17–28` |
| A-4 | **The 3D HUD already shows what was heard**, from `conversation-history.heard`, as text | `slice_link.gd:399–420` |
| A-5 | **Chinese text renders in both clients.** The shared settings layer loads `NotoSansSC-Regular.otf` as the CJK fallback | `clients/shared/settings/text.gd:16, 217` |
| A-6 | **One input gate** holds walk, look and the `move` report while a client menu is open (SET-a AC-SET-12; 16d D-16d-9). Esc order today: throw aim → interaction menu → settings menu | `clients/3d-spike/scripts/slice/slice_main.gd:305–325`; `clients/shared/settings/menu.gd:1–8` |
| A-7 | **The prompt and hint words exist only in the 2D pack** (`ui.talk.prompt`, `ui.input.hint` in `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po`) | `git grep ui.talk.prompt` |
| A-8 | **The fixed line is load-bearing in tests.** `AC-13` holds the payload equal across clients by both sending "Hello! A coffee, please." (`client_2d_interact.rs:34` `COFFEE_PLEASE`; step-15 F-S14-6); the probe `slice_probe_world.gd:182` calls `talk_to_facing(DEFAULT_UTTERANCE)`; `client_text.rs` allow-lists the constant (step-20 F-9) | as named |
| A-9 | **Recall is already testable without a live model.** Milestone D runs Alice's real `lm` seat with a scripted model, `remembers`, that answers `I remember you told me: "<line>"` and cites the line's Event ID; the evidence is the decision log and the save's facts. Its 2D and 3D windows are protocol clients, not the Godot clients | `tools/cli/tests/milestone_d.rs:1–30`; `cognition/lm-controller/tests/milestone_d_cognition.py` |
| A-10 | **No frozen design covers typed speech in 3D.** 16d ruled Q-16d-7 "default utterance in 16d; a typed line belongs with 16g". 16g (group activity in 3D, with a typed line) was ruled to exist but was never designed. S31-e (frozen 2026-10-20, not implemented) adds a 3D `LineEdit` for prices only | step-15 §22.10; `pr-s31e-clients.md:240, 432` |

## 2. What this PR does

1. **E opens a line, it no longer speaks.** Facing a person, **E** opens a one-line text box at the bottom of
   the screen: the prompt "Say to {name}:" (`ui.talk.prompt`), the line, and the hint ("Enter to send, Esc to
   cancel", `ui.input.hint`). Enter sends exactly what was typed. Esc closes it and sends nothing. Choosing
   "talk" in the Tab menu opens the same line. Facing nobody, E still says "nobody in view to talk to".
2. **Same request as 2D.** The send goes through `SliceIntents.talk(client, target, said, actor_location)` →
   `submit("talk", target, {"utterance": said}, actor_location)`, unchanged. Nothing typed is trimmed or checked
   in the client; an empty or over-long line goes to the server, and its refusal (`reason.*`) is shown the way
   every other refusal is.
3. **The target is pinned when the line opens.** The line sends to the person the ray met when E was pressed,
   even if the camera has turned since (otherwise looking away while typing would send to someone else). Whether
   they are still near enough is the server's decision (`too_far_away`), not the client's.
4. **One widget for both clients.** `clients/2d/scripts/hud/talk_line.gd` moves to
   `clients/shared/hud/line_input.gd` (`MineWorldLineInput`) with its behaviour unchanged; each client passes its
   own `Theme`. The 2D client, the new 3D talk line, and later S31-e's 3D price field and 16g's group-activity
   kinds use this one file (Q-3DS-2).
5. **Typing holds gameplay.** While the line is open, the shared input gate holds walk, look, jump, the `move`
   report and the action keys (E, F, G, R, B, Tab), so typing "wasd" does not walk. The mouse is released while
   the line is open and captured again when it closes. Esc order becomes: throw aim → **the line** →
   interaction menu → settings menu.
6. **Chinese input uses Godot's own IME.** A focused Godot `LineEdit` turns the operating system's input method on
   (macOS Input Sources, Windows IME/TSF, Linux X11 XIM/IBus/Fcitx; Godot 4.7). No extra library and no custom
   composition code. The CJK font fallback (A-5) draws the composed text. A pre-edit (composition) string is not
   submitted: Enter during composition confirms the candidate, as the OS does; a second Enter sends.
7. **Words move to the shared layer.** `ui.talk.prompt` and `ui.input.hint` move from the 2D pack's `.po` files to
   the shared layer 1 (`en`, `zh_Hans`), so both clients read the same text (`client_text.rs` checks both
   languages, AC-SET-4).
8. **The fixed line goes.** `DEFAULT_UTTERANCE` and its `client_text.rs` allow-list entry are deleted. Tests that
   relied on it type their line through the real widget (`type_and_send`), the same path a keyboard takes
   (A-8).

What does **not** change: the `talk` contract, `systems/conversation`, the server, the protocol module, the
cognition controller, the 2D client's behaviour, any world pack.

## 3. Reuse, both ways

- **Adopt:** Godot's `LineEdit` and its built-in IME handling (engine, already a dependency, `DEP-20`-era Godot
  4.7). Writing our own text field or IME bridge would be reinventing the engine. No decision record is needed
  for using a built-in node of an adopted engine.
- **Not forced:** no chat framework, no dialogue-tree addon (Dialogic etc.). Those own conversation flow and
  state; in MineWorld the conversation is a System's state on the server, and the client only types a line.

## 4. Design decisions

| Id | Decision | Rejected | Why |
| --- | --- | --- | --- |
| D-3DS-1 | E opens the line; there is no one-key fixed greeting left | keep E = fixed line, add T = typed line | A fixed line is the player not speaking. One key, one meaning, same as 2D's one "talk" entry |
| D-3DS-2 | The line widget lives in `clients/shared/hud/`, used by both clients | a second copy in 3D | Same behaviour by construction; the parity tests then compare one widget's output |
| D-3DS-3 | The target is pinned at open; sent regardless of distance or availability | re-target on Enter; close when the target leaves view | Re-targeting sends words to the wrong person. Closing on distance is a client rule (`client_rules.rs` would rightly flag it) |
| D-3DS-4 | No client-side length or emptiness check | count bytes in GDScript | The server's `Utterance` is the one rule (A-3). The refusal reason is shown. A live byte counter is presentation and may come later |
| D-3DS-5 | Gameplay is held by the existing gate, extended to one more "open" source | polling a flag in `player.gd` | D-16d-9: one gate for every client overlay. If the gate cannot take a third source, that is a bounded change to the gate, recorded in the ledger |

## 5. Acceptance (decided before measuring)

Every criterion names a second world (charter: no world-specific hard-coding). The second world is a generated
variant: a scratch copy of `worlds/social-cafe` in which the barista has another name, written by the test
(16d D-16d-10 precedent). No file under `worlds/` changes.

| Id | Criterion | How it is checked |
| --- | --- | --- |
| AC-3DS-1 | **3D sends what was typed.** In the real 3D Godot client, facing Alice, E opens the line; typing `My name is Wei and I paint boats.` and Enter submits `talk` to Alice with exactly that `utterance`. The same with `我叫小魏，我画船。` (UTF-8 preserved byte for byte) | `tools/cli/tests/godot3d` drive through `type_and_send`; the server's request log; both worlds |
| AC-3DS-2 | **2D and 3D ask the same question.** The same typed line in the 2D and the 3D client gives requests whose `differing_fields` ⊆ {`ActorLocation`} (`server/src/parity.rs`) | `ac13_semantic_parity.rs` fixtures re-captured from the real clients typing the line; `COFFEE_PLEASE` stays as the typed text |
| AC-3DS-3 | **Typing does not play.** With the line open, typing `wasd e` neither moves the body (no `move` report, position unchanged to the millimetre) nor sends a `talk`; Esc closes the line and sends nothing; the next E opens it again | godot3d drive; the `move` report count |
| AC-3DS-4 | **Told in 3D, recalled in 2D — recorded cognition.** One server, Alice on her real `lm` seat with the scripted `remembers` model (A-9). The real 3D Godot client tells Alice a line unique to the run; then the real 2D Godot client talks to her. Her reply to 2D cites the Event ID of the 3D line in `recalls`, and her words contain it. Then the reverse: a new unique line typed in 2D, recalled in her reply to 3D. Evidence: the decision log and the save's `spoke` facts, as Milestone D | new `tools/cli/tests/speech_both_ways.rs`; both worlds; runs in CI with no live model |
| AC-3DS-5 | **Told in 3D, recalled in 2D — a real local model (operator, by hand).** `ollama` with the default local model; Alice on the `lm` seat. In 3D tell her "My name is Wei, I paint boats." Walk away, open 2D, talk to her: her reply mentions Wei or boats. Reverse with a new fact told in 2D and asked about in 3D. Also once in Chinese through the OS pinyin input. Pass is the operator's judgement; the decision log's `recalls` is attached | §7's hand list, in the PR body |
| AC-3DS-6 | **The refusal is shown, not prevented.** An empty line and a 481-byte line are both sent and the 3D HUD shows the server's reason in the selected language | godot3d drive; `reason.*` keys present in `en` and `zh_Hans` (`client_text.rs`) |
| AC-3DS-7 | **2D unchanged.** The existing 2D interact and settings drives pass on the moved widget with no edits to their expectations | `client_2d_interact.rs`, `client_settings.rs` |

## 6. Commit plan

- **C0 — this design.** Docs only.
- **C1 — the shared widget.** Move `talk_line.gd` to `clients/shared/hud/line_input.gd`; 2D uses it; move the two
  keys to layer 1. Validation: AC-3DS-7. Review: no behaviour change in the diff.
- **C2 — the 3D line.** `SliceTalkLine` wiring in `slice_main.gd` (a new small file, not more of the 834-line
  `slice_link.gd`): E and the Tab menu's talk open it with the pinned target; the gate and Esc order; mouse
  release; delete `DEFAULT_UTTERANCE` and its allow-list entry; the probe types its line. Validation:
  AC-3DS-1, -3, -6. Review: no rule in the client (`client_rules.rs` clean).
- **C3 — the tests.** AC-13 fixtures re-captured (AC-3DS-2); `speech_both_ways.rs` with the generated variant
  (AC-3DS-4). Validation: both new runs green, and a mutation (3D sends a constant instead of the typed text)
  turns AC-3DS-4 red.
- **C4 — status.** `docs/MVP_STATUS.md` 3D row: "talk's words fixed" becomes "say anything (typed, IME)";
  step-15 §22.10 cross-reference; ledger.

## 7. What the operator checks by hand (goes into the PR body)

1. Start the town with the launcher (`tools/launch/LAUNCHER.md`) and Alice on a local model (Ollama, the
   default local model; `python -m mineworld_cognition --config cognition.toml`), both clients connected. In 3D, walk to Alice, press E, type
   `My name is Wei, I paint boats.`, Enter. She answers in the 3D HUD.
2. Close 3D. Open 2D, click Alice, talk: `Do you remember me?` Her answer mentions Wei or boats.
3. Reverse: in 2D tell her a new fact; in 3D ask her about it.
4. In 3D, switch the OS input to Chinese pinyin, press E, type `你还记得我吗？`, Enter. The candidate window
   appears next to the line and the Chinese text is sent and shown.
5. While the line is open, press W: you do not walk. Esc closes the line; Esc again opens settings.

## 8. Execution contract (to be confirmed at the freeze)

- **Scope:** §2, nothing else. No server, contract, protocol or cognition change. Any need for one is a
  MATERIAL STOP back to the operator.
- **Invariants:** the client never checks an utterance; one `talk` request shape for both clients; one input gate.
- **Validation:** the `fast` check per push; AC-3DS-1 … -4 and -6 … -7 run locally through `with-slot.sh`;
  AC-3DS-5 by the operator at the milestone.
- **Done:** READY FOR OPERATOR REVIEW with the hand list.

## 9. Questions for the freeze (recommended answers)

| Id | Question | Recommended |
| --- | --- | --- |
| Q-3DS-1 | Does E open the typed line (no fixed greeting left), or keep E = fixed line and add a second key? | **E opens the line.** One key, like 2D's one "talk" |
| Q-3DS-2 | Move the 2D widget to `clients/shared/` so 2D, 3D talk, S31-e's price field and 16g share one file? | **Yes.** S31-e then uses it instead of its own `LineEdit` (a bounded note in S31-e's ledger) |
| Q-3DS-3 | Is this PR "16g1" (the typed line, talk only), leaving 16g for the five group actions on top of it? | **Yes** |
| Q-3DS-4 | Keep sending regardless of distance (server answers `too_far_away`), with the target pinned at E? | **Yes** |
| Q-3DS-5 | AC-3DS-5 (real local model) checked by the operator by hand at the milestone, not in CI? | **Yes.** CI uses recorded cognition (AC-3DS-4) |

## 10. Execution ledger (live)

Implementation base: origin/main @ d9cf4d75. Worktree `/Users/yuema137/mineworld-worktrees/impl-16g1`, branch
`mvp0/pr-16g1-3d-speech`.

- **C1** `[x]` impl: `clients/shared/hud/line_input.gd` (`MineWorldLineInput`, `res://mineworld_hud` symlinks in
  both clients, launchers check the link); 2D app/menu use it; `ui.talk.prompt`, `ui.input.hint` moved to the
  shared layer (en, zh_Hans, messages.pot). `[x]` validation: 2D project imports clean; `check_client_rules.py`
  PASS. `[x]` review: no 2D behaviour change (the widget's 2D look is the default when no Theme is passed).
- **C2** `[x]` impl: `SliceTalkLine` (`slice/talk_line.gd`), `SliceLink.ask_words_facing` / `talk_to` /
  `wants_words`; offer menu's composed talk opens the line; gate, Esc order and inspect held while typing;
  `DEFAULT_UTTERANCE` and `SliceIntents.compose` deleted; probes type through the widget; market probe `say`
  and `typing_gate` steps. `[x]` validation and `[x]` review: see C3's runs; no rule in the client.
- **C3** `[x]` impl: `tools/cli/tests/speech_both_ways.rs` (5 ignored tests), `godot3d::play_in`, nightly layer
  `speech` / job `speech-linux` (Godot + uv). `[x]` validation, local macOS: `the_3d_line_sends_what_was_typed_and_typing_does_not_play`
  PASS (AC-3DS-1, -2, -3, -6, generated variant); the four `told_in_*` scenarios PASS (AC-3DS-4, café and
  generated variant, both directions). `[x]` mutation: talk_line.gd sending a constant instead of the typed words turns `told_in_3d_recalled_in_2d` red ("the typed words, byte for byte"), local run 2026-10-20.
- **C4** `[x]` `docs/MVP_STATUS.md` rows; `clients/shared/README.md`, `SETTINGS.md`.

Bounded discoveries:

- **B-1** AC-3DS-2 is checked by the two real Godot clients typing the same line (`speech_both_ways.rs`),
  not by re-capturing `clients/protocol/evidence/` (that evidence comes from the protocol module's demo,
  which this PR does not touch, and its words are unchanged).
- **B-2** The scripted `remembers` model quotes the earliest remembered line, so the two directions of
  AC-3DS-4 run in two fresh worlds rather than one sequence.
- **B-3** No PR CI job has both Godot and the Python workspace; the tests run in a new nightly job
  `speech-linux` (group `clients`), registered in `ci_nightly.py` `JOBS` and the report's `needs`.
- **B-4** S31-e's `trade_panel.gd` keeps its own `LineEdit` for now; moving it onto `MineWorldLineInput`
  is S31-e's follow-up (Q-3DS-2).

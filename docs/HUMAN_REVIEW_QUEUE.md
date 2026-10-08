# Human review queue

**Only large integrated milestones belong here.** Not individual PRs, structs, components, events,
dependency choices or refactors — the pi-agent owns routine engineering review. A milestone earns
a place here when a coherent capability can be *experienced* as a whole.

Engineering progress is tracked separately in [`MVP_STATUS.md`](MVP_STATUS.md). A milestone parked
here does not block anything else.

**Updated:** 2026-10-07 (Milestone C demonstrated, awaiting the operator's review)

---

## Framework milestones

| | Milestone | Demonstrates | State |
| --- | --- | --- | --- |
| **A** | Runnable world runtime | load Social Café → server → two clients + an agent → cause a change → all observe it | ✅ **complete 2026-09-27** — `AC-15` holds, 259 tests · restart/persistence is S5 and moves to **B** |
| **B** | Persistent people and social life | Alice and Bob persist, know each other, share an activity, and survive a restart with their history | ✅ **demonstrated 2026-10-07** — PR 10b, `tools/cli/tests/milestone_b.rs`; awaiting the operator's review (below) |
| **C** | Objects and everyday economy | Market Town: work → earn → buy → inventory changes → another client sees it → persists | ✅ **demonstrated 2026-10-07** — PR 11f, `tools/cli/tests/milestone_c.rs`; awaiting the operator's review (below) |
| **D** | LM-native persistent characters | speak to Alice in 2D, meet her in 3D, and she reacts consistently with what happened | ❌ |
| **E** | Package composition | a real world assembled from independently installable packs | ❌ |

**Milestone A, and the one thing in it worth watching.** `AC-15` — *there is only one Alice* —
holds, proved against the real binary hosting the real pack. The evidence names identity rather
than appearance, which is the whole point: same world instance, same Alice `EntityId` found by
tag and never by a literal, one monotonic event sequence `[6, 7, 10, 11]` across all three
participants. The sentence that carries it is Alice repeating to the second window what the first
window said, naming the other speaker by identity — **two synchronised copies of Alice could not
produce that sentence.**

The counterfactual is a committed, passing test rather than a caution: two servers, and the test
*asserts* that both clients perceive a barista with the **same** `EntityId` at the same authored
position, because two loads of one pack resolve the same keys (`AC-12` working). An appearance
test passes there and is wrong.

```sh
clients/protocol/run.sh          # windowed, scripted — what a person watches
clients/protocol/run.sh play     # windowed, driven by the keyboard
```

Recorded as **not** done and not approximated: `MVP.md` §9.1's fourth line, *same persisted state
revision*. The world is in memory; persistence is S5's and belongs to Milestone B.

**Milestone B, and how to see it for yourself.** Twelve people live a month in the town, headless.
They talk, invite each other for coffee or a walk, and come to know each other. Alice's life is then
read back out of the log:

```sh
mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save /tmp/cafe
mineworld biography worlds/social-cafe --save /tmp/cafe --person alice
mineworld biography worlds/social-cafe --save /tmp/cafe --person bob
mineworld server worlds/social-cafe --save /tmp/cafe     # then join the `alice` seat with a client
```

What to look at:
- On day 1, Alice and Bob each `became-acquainted` with the other. On day 1 they share a
  `group-activity` (event `#98`). By day 3 a `relationship-changed` shows them growing closer.
- Every line of the biography carries the event id it came from.
- Kill the `run` with Ctrl-C or SIGKILL partway and run the same command again: the world finishes
  identically, and so do both biographies.
- Alice's own observation in the server shows her `acquaintances`: how she regards Bob, and nobody
  else's view.

The test performs all of this with real SIGKILLs of both `run` and `server`.

Watch for one known gap: relationships never decay. After about two months every pair is as close as
it gets, and no more relationship changes happen. The 300-day test prints that per month.

**Milestone C, and how to see it for yourself.** Market Town is Social Café plus six installed System
Packs and configuration. Alice works the café counter in the mornings and is paid by the hour; the
café restocks from her shift; everyone buys, eats, drinks and gives. Two days, saved, then hosted:

```sh
mineworld run worlds/market-town --headless --seed 7 --days 2 --save /tmp/market
mineworld inspect /tmp/market --last 0                         # every cause resolves
mineworld server worlds/market-town --save /tmp/market         # then join `alice` and `bob`
```

What to look at:
- In the save, on day 1: Alice is `hired`; her shift starts at 05:30 while she is still walking
  over, she arrives at the café at 07:15, and at 14:00 `shift-ended`, `wage-due` and the
  `money-transferred` that pays her follow.
- In the server, walk Alice and Bob into the café. Alice is offered a `buy` per thing on the shelf,
  each a complete request; submit one as it is offered. Her own `wallet` falls by the price, and her
  `holdings` gain one.
- Bob, in the café, sees the shop's listing: the stock of what Alice bought is one lower. He is never
  shown Alice's wallet or what she carries.
- Kill the server and start the same command again: the same world, the same revision, the same
  wallet, holdings and shelf.

The test (`tools/cli/tests/milestone_c.rs`) does all of this with real sockets and a real SIGKILL.
Its sibling tests prove the rest of S9: `tests/acceptance/tests/ac1_composability.rs` (AC-1 as
`ARC-35` measures it, within `ARC-33`'s static-linking boundary — installing a pack is a directory,
two lines in `systems/installed` and a rebuild), `tools/cli/tests/market_town.rs` (the market lives
300 days) and `tools/cli/tests/market_composition.rs` (each market pack removable).

Watch for the known limits: people without a job live on an endowment sized for 300 days (L-13);
items and organizations have no names, so a listing shows ids (F-41); every unavailable buy says
`TargetUnavailable`, whatever the reason (F-48).

## Default-style milestones — taste, and the operator decides

These are candidates until the operator says otherwise (`ARC-11`). An agent may build and
recommend; it may not declare something the default look.

Named per `ARC-20`, because with two 3D tracks running (`ARC-18`) "the 3D character" no longer
identifies one thing.

| | Milestone | State |
| --- | --- | --- |
| **VIS-2D-1** | Playable 2D default scene with an enterable interior | 🚧 candidate in progress |
| **VIS-3D-GODOT-1** | Reference-matched character in Godot | ❌ **failed fidelity gate 2026-09-27** — rebuilding |
| **VIS-3D-GODOT-2** | Integrated Godot slice: character, street, enterable building, interior, lighting, cameras | ❌ |
| **VIS-3D-UE5-1** | Unreal slice of equivalent scope | ⏸ **parked** — spike phase one done (`ARC-21`), operator paused the install |
| **VIS-3D-AB-1** | Godot vs Unreal side-by-side, same reference, same scope | ⏸ parked with `VIS-3D-UE5-1` |

`VIS-3D-GODOT-1` failed on categorical identity mismatch, not polish: the reference is a young
woman in an open burgundy zip hoodie and the candidate was a man in a red quilted puffer jacket
(`ARC-17`). The rig, retarget, animation, cadence, footwear and ground-contact work underneath it
is unaffected and is kept.

### What a review package contains (`ARC-20`)

Milestone id · what changed · **the exact launch command** · real runtime screenshots · the
canonical reference · a side-by-side where applicable · known limitations · the specific
subjective questions being asked. The operator must be able to launch, look, walk and judge
quickly. An engineering log is not a review artefact.

### Reaching review does not stop work

On `READY FOR HUMAN VISUAL REVIEW`: preserve the runnable candidate, save the screenshots, record
it here, **stop subjective polishing on that branch**, and move to independent work. Review is a
branch-level checkpoint, never a global barrier. Neither 3D track waits on the other, and none of
the framework milestones wait on any of them.

## Style infrastructure — architecture, and it never waits here

Tracked in [`MVP_STATUS.md`](MVP_STATUS.md), listed only so the split is visible: Presentation and
Asset Pack interfaces, style manifest schema, provenance, generation-pipeline integration,
renderer bindings, style switching, validation and composition. None of it blocks on a taste
decision, and none of it may be tied to whichever style happens to be default.

---

## Open questions for the operator, not blocking anything

- **The project's name.** Microsoft published an unrelated "MineWorld" in 2025 — a video-generative
  world model — with a paper, a repository and a Hugging Face presence. Purely a discoverability
  and package-naming collision (`ARC-12`), and it blocks no engineering. Worth a deliberate
  decision before public launch rather than discovering it in a search result.

## Accepted, not to be re-litigated

- **The 2D and 3D presentation direction**, as of 2026-09-27. Movement, camera work and the
  3D lighting rig are accepted; the visual track now improves fidelity toward the references
  rather than re-deciding the direction.
- **Procedural SVG is retired as the 2D art strategy**, keeping its layout, placement, projection,
  camera and occlusion logic. Visuals get replaced; architecture does not.
- **Primitive humanoids are below baseline** and are being replaced by a reusable rigged pipeline.
- **The default character is an identity reconstruction of `04_character_closeup.png`**
  (`ARC-19`), which supersedes `ARC-4`'s facial-fidelity exclusion for that one image. `ARC-4`'s
  reasoning survives, applied to the right object: the bar is high for **our** default character,
  which we pay once and ship as an asset, and stays low for **what the framework requires**, which
  is the humanoid profile and nothing about appearance.

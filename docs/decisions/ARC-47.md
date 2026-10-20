## ARC-47 — No rule in the client, made executable

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13a's design freeze · **Relates
to** [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§4, 7–9; `INV-5`; `INV-9`; `ARC-23` · **Design**
step-13 §§4.4, 8, 14.2 (AC-W5 … AC-W8)

**Problem.** "The client implements no world rule" has so far been a review convention. A convention
is checked when someone remembers to; the 2D client is the place it is easiest to break, because a
renderer that already knows positions is one comparison away from deciding a distance.

**Decided.** The rule is a property tests hold, in two halves.

1. **Structure, scanned** (`scripts/check_client_rules.py`, on `clients/2d/**/*.gd`, comments
   excluded): only `scripts/intents.gd` submits; no other file names an action type that file
   composes; a function that submits reads no verdict (`may(`, `"available"`, `unavailable_reason`,
   `requirement(`); distances are computed only where they are request sizes, routes or drawing
   (`walker.gd`, `projection.gd`, `town.gd`, `scene/`); only `link.gd` opens a connection. Each rule is
   shown to report a planted violation by file and line before it is trusted.
2. **Behaviour, against a server that lies** (a test-only revision-1 stub): an offer marked
   unavailable is still submitted when the player chooses it; a refused stride ends the walk and the
   body is drawn where the next observation puts it; an observation that moves the player is followed,
   never argued with by a correcting `move`. Each is shown to fail under the planted mutation.

The client composes only the requests it knows how to ask for, decides nothing, never refuses to
submit what the player chose, and never retries a request whose answer it did not see (a retry
could do a thing twice; the server allocates identity, `INV-6`).

**Note 1, 2026-10-08 (S12 PR 13b; step-13 §15.4 D-b-1 … D-b-6, QS13b-5) — the interaction menu is the
observation's affordances.** No new principle; how this record applies to menus:

1. A menu is about one subject — a perceived person, or the observer itself — and lists exactly the
   affordances the newest observation offers against that subject (target-less ones for the
   observer), **in the server's order**, one entry per affordance: nothing sorted, grouped out of
   order, removed or invented. Several complete affordances sharing an action type and a target are
   several entries.
2. A **complete** affordance is submitted unchanged through the shared module's `submit_affordance`;
   a **composable** one (its type is in `intents.gd`'s `COMPOSED`) through that file's composer, with
   the player's input sent exactly as typed; any other is listed disabled, "not supported by this
   client", and never sent.
3. An entry the server marks unavailable is drawn greyed **and stays choosable**, with the server's
   reason in the client's wording and, where declared, the requirement's range, unevaluated. Choosing
   it submits it; the answer is the server's. No key submits "the first available" entry: selecting
   by the server's verdict is the client deciding.
4. The only entry a client adds is "walk to <person>", which is the offered `move` (listed only when
   the frame offers a target-less `move`), walked as ordinary strides the server answers one by one.
5. The scan grows with the menu: R2 admits `app.gd`'s `"invite"` (the join option) by explicit entry
   only; R3 extends to `menu.gd` and `hud/` (a function that calls `intents.` reads no verdict); R6
   refuses a user-visible English sentence outside the wording (`ARC-70`).

**Note 2, 2026-10-16 (S14 PR 16d; step-15 §22.3 D-16d-3, D-16d-6, Q-16d-10) — the 3D client follows
the same menu model.** No new principle. The 3D slice's two menus — yours (B, the target-less offers)
and the person under the reticle (Tab, the offers directed at them) — are Note 1's model, built in
`clients/3d-spike/scripts/slice/offer_menu.gd`: one entry per affordance, in the server's order; a
complete affordance submitted unchanged through `submit_affordance` (`intents.gd`'s `submit_offered`);
an incomplete one composed only when `intents.gd` composes its type (`talk`, with the default words); the
offer the body answers by walking (`move`) listed and never chosen; anything else listed disabled and
never sent; an unavailable entry greyed and still choosable. Verdicts are read where entries are built,
never where one is chosen. Both clients print their menus in one report format, compared field for field
(step-15 §22.4 P-2), and the words both show — menu, panel and answer keys — live once, in the shared
catalog layer (`ARC-70`'s 2026-10-09 note).

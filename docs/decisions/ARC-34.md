## ARC-34 — Complete affordances: an offer may carry the request it would accept

**Date** 2026-10-07 · **Approved by** the operator at S9's design freeze (step-10 QS-4; refinements
QS-20 … QS-25 accepted by the primary session, step-10 §12.0) · **Implements**
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §15.2, [`MODULE_SPEC.md`](MODULE_SPEC.md) §5 · **Relates to**
`INV-10`, `INV-13`, `ARC-23`, `ARC-26`, `ARC-27`, `ARC-35`, [`MVP.md`](MVP.md) §9 `AC-1`, `AC-15`
· **Design** `.structured-coding/plans/mvp0/step-10-market.md` §2.3, SD-9 … SD-12, §4.3 (S9, PR 11c)

**Problem.** `PacedRuleController` submits only actions it was compiled against: every request it
builds is typed by a pack crate it depends on, and it reads affordances only to ask whether *an action
it already knows* is available. A System Pack installed after the controller was written therefore
appears in every observation as an affordance and is never attempted by any headless person, because an
`Affordance` names an action and carries no payload: to submit an offered action, a requester must
already know the action's payload shape. The same is true of a client. `AC-1` forbids teaching the
controller each new pack, and a world whose new actions nobody headless ever attempts would pass a path
check and prove nothing (`ARC-23`). `CORE_CONCEPTS.md` §15.2 already states the intent the gap blocks:
a controller "is told what is possible instead of guessing" — told *that* something is possible, not
yet *what exactly to send*.

**Options considered.**

```text
(a) the controller learns each new pack's actions         the edit AC-1 forbids; every future pack
                                                         repeats it
(b) a Controller Pack policy as world data ("at this      payloads authored in a World Pack are rules in
    place, attempt this request")                        content (MODULE_SPEC §4 constraint 3), and the
                                                         policy still has to know payloads
(c) a generic `interact` action the server resolves       the server would choose the action for the
                                                         person (INV-1, INV-6); one resolver must know
                                                         every pack's precedence — a God object
(d) payload schemas in affordances (JSON-Schema-like)     a second schema language for an enumerable
                                                         problem; heavy before any LM controller exists
(e) declare that headless people need not use new packs   the new pack would be untestable headless: an
                                                         instrument that cannot see (ARC-23)
(f) complete affordances: the offering system may attach  chosen
    the exact payload it would accept; any requester may
    submit it unchanged
```

**Choice: (f).**

1. **The contract.** `Affordance<P = Vec<u8>>` gains a last field `payload: Option<P>`, the complete
   request payload the offering system would accept, in the observation's payload encoding.
   `Observation<P>` holds `Affordance<P>`. The field is serialized **only when present**
   (`skip_serializing_if`), and a frame without it decodes, so every existing observation, transcript
   and frame is byte-identical. An affordance with a payload is a **complete affordance**.
   - `Affordance::available` and `Affordance::unavailable` are unchanged and carry no payload;
     `with_payload(P)` adds one; `payload()` reads it.
   - `Affordance::request(actor, encode) -> Option<ActionRequest<Q>>` turns a complete affordance
     into the request it names: labelled with the affordance's **own** action type, targeted at its
     target, its payload re-encoded by the caller's `encode` (an observation carries JSON values,
     dispatch takes bytes — the encoding is not decided in `contracts`). `None` without a payload.
     Whether the affordance is available is the caller's judgement, read from `is_available`.
   - The label is the affordance's own, so `request` needs a crate-private labelling constructor on
     `ActionRecord`. No trust is lost: `ActionRecord` already deserializes from any label, and
     dispatch decodes the payload with the owning system's type.
2. **Who makes an offer complete.** Only the owning System Pack, through presence's
   `Offer::complete(&action, requirement)`. The action type is read off the action value's own type,
   exactly as `Offer::new::<A>` reads it, so a payload of another action cannot be attached at all.
   Perception carries the payload into the affordance whatever the verdict — available or unavailable
   with its reason — and decides nothing about what it means. An offer of a disabled pack is dropped
   by the same route map as before (`INV-10`, `AC-2`), payload and all.
3. **The server still decides.** A complete affordance is an offer, not a permission: whatever is
   submitted is validated by the owning system at dispatch (`ARCHITECTURE.md` §9). A controller that
   submits one has not created an interaction; it has attempted one the world offered.
4. **The paced controller's offer band.** `PacedRuleController::decide` gains exactly one band, after
   the social initiative and before the walking roll: the available complete affordances of the
   observation, in observation order; none → the band takes no part; otherwise, if draw index
   `OFFER_DRAW = 14` is below `ATTEMPTS_OFFERED` (out of 100), the one chosen by draw index
   `OFFERED_CHOICE_DRAW = 15` is submitted through `Affordance::request`. It imports no pack type for
   it, and `decide(&self, …)` stays a pure function of seed, pace and observation (`ARC-27`).
   `ATTEMPTS_OFFERED` is fixed by step-10 C-C6's measurement on a scratch install of a synthetic pack
   offering a complete affordance to every person everywhere — the worst case — against a criterion
   stated before measuring: every seat still moves and talks in every 30-day bucket, and every seat's
   offered request is accepted in every bucket. It passed at **20**, which is frozen (step-10 §9.3
   E-C6, and the note below). The band's constants and position are then frozen for S9 (`ARC-35` item 6, I-9): if
   a later world behaves badly, the remedy is in what its packs offer, never in the controller.
5. **`RuleController` is unchanged.** The reactive controller (`--agent`) answers and takes no
   initiative; it never attempts a complete affordance (`AC-15`).
6. **Free-form actions stay known by name.** `talk`'s utterance, `move`'s position and `invite`'s
   kind cannot be enumerated by the offerer, so the paced controller keeps knowing those actions by
   name. They are the foundation vocabulary of a walking, talking world. A new pack whose actions are
   free-form is usable headless only through a language-model controller (S10), or by offering
   complete affordances for a bounded choice.
7. **Clients.** `server/PROTOCOL.md` §§5–6 and `clients/protocol/ADOPTION.md` document the field: a
   client may submit a complete affordance unchanged and decides nothing new. No client code changes
   in S9 (step-10 QS-20); the first client use is S12's.

**Why this is not an `AC-1` violation although it edits `contracts`, presence and the controller.** It
is a precursor (PR 11c) that lands before the market, names no market concept (`ARC-35` item 7), is
proven with a synthetic pack the controller crate has never been compiled against, and leaves every
existing world byte-identical, because no existing pack offers a complete affordance.

**Accepted limitations.**
- The offerer must be able to enumerate the choices it would accept. An action whose payload is
  free-form cannot be offered complete (point 6).
- Observations grow with the offers: one complete affordance per offered choice, per target. A pack
  that offers many choices to many people makes every consult larger and slower (step-10 R-S9-2); the
  remedy is that the pack offers less, never a change to the pace or the controller.
- The band's rate is one number for every pack: a world with many packs offering complete affordances
  divides the same rate among them, in observation order.

**Note, 2026-10-07 (S9, step-10 C-C6) — the measurement that fixed the rate.** A scratch install,
never merged, put a pack offering a complete `ring { low | high }` to every person everywhere, with no
spatial requirement, into Social Café, and ran it 300 days with seed 7. The criterion, stated before
measuring (step-10 QS-25): every seat moves and talks in every 30-day bucket, and every seat's `ring` is
accepted in every bucket. At `ATTEMPTS_OFFERED = 20` both held — every seat moved at least 1 346 and
talked at least 393 times per bucket, and rang at least 197 times per bucket (29 907 rings in all, no
fault). The value is therefore **20**, frozen with `OFFER_DRAW = 14`, `OFFERED_CHOICE_DRAW = 15` and
the band's position. Under that worst case, moves fell from 180 665 to 167 348 and talks from 67 752
to 57 741 over the 300 days — the share of consults the band takes — which is what a pack offering
something to everyone everywhere costs, and a reason for a pack to offer less, not for the controller
to change.

**Note, 2026-10-08 (S14, PR 16b) — the first client accessors.** Point 7's "no client code changes in
S9" stands as history. The shared GDScript module `clients/protocol/mineworld` now reads and submits
complete affordances, for both reference clients at once:
- `MineWorldObservation.affordances(action_type, target)`, `complete_affordances(…)`,
  `affordances_about(id)` and `is_complete(affordance)` list them in the server's order and never pick
  one;
- `MineWorldClient.submit_affordance(affordance)` submits one unchanged, whether or not it is
  available, and refuses only an affordance with no `payload`.

Nothing on the wire changed. The API is specified in `clients/protocol/ADOPTION.md` §2 and designed in
`.structured-coding/plans/mvp0/step-15-demo-3d.md` §18. No new record was needed: it is this decision's
point 3 and point 7 carried into code.

**Note, 2026-10-08 (S17, PR IL-b; QIB-3, QIL-12, QIL-13) — a pack-stated refusal on an offer.**
Presence's `Offer` gains one builder, `Offer::refused(Rejection)`: the owning pack says the action is
offered but refused for a reason only it can judge — `PermissionDenied` from its section (`ARC-63`
item 8). Perception reports a refusal **before** it evaluates the spatial requirement, as
`Affordance::unavailable(…, reason)`; the requirement is still shown and a complete offer's payload
still travels. An offer that is not refused behaves exactly as before, and no contract changes:
`Affordance` already carries any `Rejection`. The offer and the dispatch give the same answer in the
same order: a refused action is refused before its spatial check in both.

**Note, 2026-10-14 (S27-b, F-S27-2).** group-activity offers accept-invitation, decline-invitation,
join-group-activity and leave-group-activity complete, with the empty payload each accepts, so a
requester that does not know the pack can attempt them (step-27 §5.5). invite stays incomplete: its kind
is free text. Consequence recorded in pr-s27b §3.2 F-S27b-1: the paced rule controller's offered band now
draws among them.

# Baia Serena — a seaside resort

**Premise.** A small, sun-bleached resort on a quiet bay in southern Europe, late June. Slow days, long dinners,
everyone knows the bartender.

**Use.** I want to play it, in 3D mostly; 2D is fine for checking. I'd also like to run it headless for a month and
see who became friends.

**Me.** I arrive as a guest for a week. No job, some money.

**Places.** Hotel Baia Serena: lobby, about eight guest rooms, a restaurant, a terrace bar over the water. A sandy
beach with loungers and umbrellas. A wooden pier with a little boat-hire hut. A short lane with a gelato shop, a
pharmacy and a newsagent.

**People.** Around 25. Staff: receptionist, two cooks, two waiters, the bartender Marco, a lifeguard, the boat-hire
man. The rest are guests: a honeymoon couple, a family with two teenagers, a retired schoolteacher who reads all day,
a group of friends from Milan.

**Daily life.** Staff work shifts and get paid. Guests sleep late, eat, swim (or at least lie on the beach), shop,
drink at the bar. Breakfast 7–10, dinner from 19:30. Beach party every Saturday night.

**Minds.** Marco on a local model if possible, everyone else simple rules.

**Anything else.** The sea should always be visible. Weather like the real Amalfi coast in June.

---

## Expected outcome on this build (2026-11-28; not part of the brief)

Authority: [`step-01-brief-to-playable-world.md`](../../../.structured-coding/plans/onboarding/step-01-brief-to-playable-world.md) §5.1.

- **Built:** places for the lobby, rooms, restaurant, terrace bar, beach, pier, boat-hire hut and the lane's shops,
  all reachable; about 25 people (staff with jobs and shifts, guests with holiday routines); goods from
  `entities/modern-goods`; a calendar date in June and weather located on the Amalfi coast; a happening every Saturday
  evening on the beach; Marco on the local decision model with a rule fallback; one `visitor` seat for the player.
- **Packs (bundled):** presence, movement, conversation, relationships, group-activity, schedule, naming, persona,
  item, inventory, item-transfer, economy, employment, consumption, needs, sleep, posture, bodies, calendar, weather,
  happenings, notices.
- **Gaps:** booking or checking into a room (real product gap; a possible labelled stand-in is a bought room-key
  item); swimming (real product gap; a possible labelled stand-in is a group activity on the beach); opening hours for
  the bar and shops depend on the shop-hours work in progress; drawing a new world in the clients needs a far field
  for it (see [`ONBOARDING.md`](../ONBOARDING.md) §5).

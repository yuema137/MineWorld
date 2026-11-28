# A big city

I want a big city like New York — avenues and numbered streets, five-storey walk-ups with fire escapes, a 24-hour
deli on the corner, a diner, a subway entrance, an office tower, a small park with chess tables. It should feel
crowded: people everywhere at rush hour, quieter at 3am but never empty.

I just moved here and rented a tiny room above the deli. I don't know anyone. I want to wander, get a job, find
regulars I recognise. Mostly I'll play it in 3D, but I also want my research agent to run it headless for a year and
tell me how neighbourhood friendships form when people commute.

Start with whatever is realistic for my laptop and tell me what a full Manhattan would need. No paid AI for the
residents.

---

## Expected outcome on this build (2026-11-28; not part of the brief)

Authority: [`step-01-brief-to-playable-world.md`](../../../.structured-coding/plans/onboarding/step-01-brief-to-playable-world.md) §5.2.

- **Built (first iteration):** a **declared neighbourhood slice**. It has two by three connected blocks, at most 40
  people (the caps of [`BRIEF.md`](../BRIEF.md) §4), the deli with a room above it, the diner, an office with jobs, the
  park, and a subway entrance whose platform is a stub. There is one `visitor` seat, and every resident is on rules.
  `brief.md` names it as a slice and lists what the full city needs.
- **Packs (bundled):** the town set, as for the seaside resort, plus the Entity Packs `entities/populations-us` and
  `entities/person-attributes` for a varied population.
- **Gaps:** riding the subway or a bus (real product gap: a transport System Pack is designed, not built);
  connected street-grid generation for many blocks (planned tooling); a world of many regions with streaming in the
  clients (designed, not built); hundreds to thousands of residents (not yet measured, so never claimed); drawing a new
  world in the clients needs a far field for it (see [`ONBOARDING.md`](../ONBOARDING.md) §5).

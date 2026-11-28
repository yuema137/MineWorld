# A village in the Apennines

An Italian mountain village in the Apennines, autumn. The borgo sits on a hilltop: a piazza with a fountain, the
church, Bar Sport, a bakery, the post office. Most families live in farmhouses down the valley, ten to twenty-five
minutes apart on one narrow road full of hairpin bends.

If you want to see someone, you drive. I have an old Fiat Panda; so do half the village. People drive up to the
piazza for coffee in the morning, to the bakery, to Sunday mass, and to the Saturday market. Nonna Pina never drives;
someone has to fetch her.

About 15 people. I'm the grandson of a family that just came back to restore the old house. I want to play it in 3D
and drive the road myself, at night too, with the headlights on. Rules for everybody; maybe the barista on a local
model later.

---

## Expected outcome on this build (2026-11-28; not part of the brief)

Authority: [`step-01-brief-to-playable-world.md`](../../../.structured-coding/plans/onboarding/step-01-brief-to-playable-world.md) §5.3 and §10.

- **Built:**
  - the borgo's places (piazza, church, bar, bakery, post office) and the farmhouses;
  - about 15 people with routines;
  - the Saturday market as a happening and Sunday mass as a routine;
  - an autumn date and Apennine weather;
  - a seat for the grandson;
  - cars that exist as items.
- **Packs (bundled):** presence, movement, conversation, relationships, group-activity, schedule, naming, persona,
  item, inventory, economy, consumption, needs, sleep, posture, calendar, weather, happenings.
- **Gaps:**
  - **Driving** (entering a car, driving the road, NPCs driving to visit, fetching Nonna Pina) is a **real product
    gap**. Vehicles are a required capability with their own planned step, and no vehicle System Pack is installed
    yet. The agent records the gap, and may link the farmhouses to the borgo by walking routes **labelled as a
    stand-in** ("a long walk, not a drive"). It must not fake a car with doors that teleport or with client-side motion.
  - Headlights and night driving belong to that same future capability.
  - Drawing a new world in the clients needs a far field for it (see [`ONBOARDING.md`](../ONBOARDING.md) §5).

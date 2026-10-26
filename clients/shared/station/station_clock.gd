class_name MineWorldStationClock
extends RefCounted

## ARC-108 (S29-e F-S29e-5): a station clock shows the world's time of day where the world keeps a
## calendar, and no hands where it does not. The disclosure that a world keeps a calendar is a
## `calendar-day` record on the place the observer stands in (`systems/calendar`). The host's wall clock
## is never read. Pure, like `MineWorldStationState`, which it sits beside: both clients and the
## headless check call the same code.
##
## Example: Market Town discloses `calendar-day` in its station hall, so an observation taken at world
## second 117 000 gives 30 600 (08:30); Social Café and Lakeside disclose none, so the clock has no hands.

## The clock of a world that discloses no calendar: no hands.
const NO_HANDS := -1
const DAY := 86400


## Seconds into the day to draw, or NO_HANDS. `components`: the component types disclosed on the
## observer's place; `at`: the observation's world time in whole seconds.
static func of(components: PackedStringArray, at: int) -> int:
	return posmod(at, DAY) if components.has("calendar-day") else NO_HANDS


## The same from an observation (a `MineWorldObservation`, or anything with its `place`, `entity` and
## `at` calls).
static func from_observation(observation) -> int:
	var here: String = observation.place()
	if here == "":
		return NO_HANDS
	var types := PackedStringArray()
	for record in observation.entity(here).get("components", []):
		if typeof(record) == TYPE_DICTIONARY:
			types.append(String(record.get("component_type", "")))
	return of(types, int(observation.at()))

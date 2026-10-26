## What every drawn station fixture shows (S29-e §5.6; `ARC-108`'s note of 2026-10-20). A fixture is a
## piece with `mw_fixture` (`StationPieces`): a board, a ticket window, a kiosk or a clock.
##
## The observer perceives only its own place (step-29 F-S29-3), so a fixture in another place -- the
## ticket window seen from the hall -- cannot be read live. Each place keeps the state of the LAST
## observation made in it; a place never observed shows the defaults (shut, "No service yet", no hands)
## (Q-S29e-2, ruled 2026-10-19). The cache is written only from observations: a remembered state is a
## disclosed state, an invented one is not. Offline (no server), every fixture shows the defaults.
##
##     learn(obs)        cache[obs.place()] = MineWorldStationState.of(obs) + MineWorldStationClock
##     refresh(layout)   each fixture shows cache[its place] or the defaults
##
## A fixture is registered either by its kit volume (the depot's fit-out: the layout says which world
## place the volume is bound to) or by its world place id directly (a furnishing: the disclosure says).
class_name StationFixtures
extends RefCounted

## [{ node: Node3D, volume: String, place: String }]
static var fixtures: Array = []
## place id -> the state its last observation disclosed.
static var cache := {}


## The state of a place nothing was disclosed about.
static func defaults() -> Dictionary:
	var s := MineWorldStationState.nothing_disclosed()
	s["clock"] = MineWorldStationClock.NO_HANDS
	return s


static func register(node: Node3D, volume: String, place := "") -> void:
	if node == null:
		return
	_prune()
	fixtures.append({"node": node, "volume": volume, "place": place})
	StationPieces.show(node, cache.get(place, defaults()) if place != "" else defaults())


## Every fixture shows the defaults (a fresh build, or offline).
static func show_defaults() -> void:
	_prune()
	for f in fixtures:
		StationPieces.show(f["node"], defaults())


## Remembers what `obs` disclosed about the observer's place.
static func learn(obs: MineWorldObservation) -> void:
	var here := obs.place()
	if here == "":
		return
	var s := MineWorldStationState.of(obs)
	s["clock"] = MineWorldStationClock.from_observation(obs)
	cache[here] = s


## Each fixture shows the remembered state of the place it stands in.
static func refresh(layout: SliceLayout) -> void:
	_prune()
	for f in fixtures:
		var place := String(f["place"])
		if place == "" and layout != null:
			place = layout.place_of_volume(String(f["volume"]))
		StationPieces.show(f["node"], cache.get(place, defaults()))


## What each fixture shows now, for the probes and the walk: [{ fixture, place, volume, shows }].
static func rows() -> Array:
	_prune()
	var out := []
	for f in fixtures:
		var n: Node3D = f["node"]
		var kind := String(n.get_meta("mw_fixture", ""))
		var shows := ""
		match kind:
			"kiosk", "window":
				shows = "open" if (n.get_node("Open") as Node3D).visible else "shut"
			"board":
				shows = (n.get_node("Body") as Label3D).text
			"clock":
				shows = "hands" if (n.get_node("Hands") as Node3D).visible else "no hands"
		out.append({"fixture": kind, "volume": f["volume"], "place": f["place"], "shows": shows})
	return out


static func _prune() -> void:
	fixtures = fixtures.filter(func(f: Dictionary) -> bool: return is_instance_valid(f["node"]))

## Lakeside, as a labelled STUB (S33-c §5.5; Q-S33c-1, operator 2026-10-18: ship it on `main`, labelled):
## flat ground, the world's real far field, and one grey walled box per place at its footprint. S33-d
## replaces it with the village behind this same file and interface (R-S33d-c-1).
##
## Data only; `SliceBoxDepiction` builds it. The frame is the lane's (step-33 §4.0): lane (x, y) mm ->
## scene (x / 1000 - 15, 0, 3 - y / 1000), so the lane's centre is the scene's origin. Footprints are
## each place's `body: floor` in `worlds/lakeside/places/*.yaml`, translated into this frame by its
## passage. Doorways are the lane-side (`there`) points those passages state, 0.4 m in front of each door
## (S33-b §5.2's "after" column; R-S33c-b-1). The halt's three places (S29-a) are boxes at their floors,
## placed by their passages, bound by `station` and their own tag.
extends SliceBoxDepiction


func hud_note() -> Dictionary:
	return {"key": "hud.depiction.stub", "args": {"world": "Lakeside"}}


func frame_origin() -> Vector3:
	return Vector3(-15.0, 0.0, 3.0)


func street() -> Dictionary:
	return {"tags": PackedStringArray([SliceDepiction.tag("street")]),
		"footprint": Rect2(0, 0, 30000, 6000)}


func places() -> Array:
	return [
		_place(tag("cottages"), "Cottages", Rect2(0, 6000, 6000, 14000), Vector2(3000, 5600)),
		_place(tag("boathouse"), "Boathouse", Rect2(6000, 6000, 6000, 12000), Vector2(9000, 5600)),
		_place(tag("bakery"), "Bakery", Rect2(12000, 6000, 6000, 8000), Vector2(15000, 5600)),
		_place(tag("square"), "Square", Rect2(18000, 6000, 12000, 12000), Vector2(21000, 5600)),
		_place(tag("shore"), "Shore", Rect2(0, -10000, 15600, 10000), Vector2(12000, 400)),
		_place(tag("pier"), "Pier", Rect2(16000, -20000, 4000, 20000), Vector2(18000, 400)),
		# The halt (S29-a): the waiting room east of the lane's end; its ticket bay north of it and its
		# platform east of it open off the waiting room, so their doorways are its side's points, in
		# this frame (the waiting room's (0, 0) stands at lane (30 000, 500)).
		_place(tag("station"), "Halt", Rect2(30000, 500, 8000, 5000), Vector2(29600, 3000)),
		_halt(tag("ticket-office"), "Tickets", Rect2(33400, 5500, 4600, 4600), Vector2(36000, 5100)),
		_halt(tag("platform"), "Platform", Rect2(38000, -1000, 4000, 40000), Vector2(37600, 3000)),
	]


## A place of the halt: tagged `station` and its own tag.
static func _halt(t: String, label: String, footprint: Rect2, doorway: Vector2) -> Dictionary:
	var p := _place(t, label, footprint, doorway)
	p["tags"] = PackedStringArray([tag("station"), t])
	return p


static func _place(t: String, label: String, footprint: Rect2, doorway: Vector2) -> Dictionary:
	return {"tags": PackedStringArray([t]), "label": label, "footprint": footprint, "doorway": doorway}


## Offline only: the cottages' yard. Connected, the body is put where the world says.
func spawn() -> Dictionary:
	return {"position": Vector3(-12.0, 0.1, -10.0), "yaw": 180.0}


## Lakeside's extras are the café town's people (step-26 E-2, S33-g §5.2): a person whose look the pack
## does not bind is drawn as one of them, and never as one of Lakeside's own (`SliceCast.use_extras_from`).
func extras_from() -> String:
	return "social-cafe"


## S33-g §5.3: five decor figures (step-33 §5's "few, ≤ 6"), all on the lane, none in a place, none
## sitting (F-S33g-4). In this stub the shore and the square are boxes, so the shore-trail walkers
## walk the lane west of the shore door and the two who stand talk on the lane by the square's door;
## S33-d moves them onto its trail and its square, behind this same data.
func decor() -> Array:
	return [
		{"pose": "walking", "from": Vector3(-14.0, 0.0, 1.2), "to": Vector3(-4.0, 0.0, 1.2)},
		{"pose": "walking", "from": Vector3(-4.5, 0.0, 0.4), "to": Vector3(-13.5, 0.0, 0.4)},
		{"pose": "walking", "from": Vector3(13.0, 0.0, -1.0), "to": Vector3(-13.0, 0.0, -1.0)},
		{"pose": "standing", "at": Vector3(6.0, 0.0, -1.6), "yaw": 90.0},
		{"pose": "standing", "at": Vector3(6.9, 0.0, -1.5), "yaw": -90.0},
	]


## Step-33 §9.1's provisional viewpoints, in this frame (S33-a and S33-d fix them; the halt's VP-L-6
## waits for S29-a). One perf view, S33-g's `lakeside crowd` (step-33 §8.1; S33-g §5.5: from
## (-6, 1.66, 2) eye height, at yaw 60); no far-field shots until S33-d.
func viewpoints() -> Dictionary:
	return {
		"farfield": [
			["VP-L-1", Vector3(-10.5, 1.80, -4.5), ["protected"]],
			["VP-L-2", Vector3(0.0, 1.66, 0.0), []],
			["VP-L-3", Vector3(3.0, 1.66, 22.0), ["protected", "judged"]],
			["VP-L-4", Vector3(-7.2, 1.80, 13.0), ["judged"]],
			["VP-L-5", Vector3(9.0, 1.80, -9.0), []],
			["VP-L-7", Vector3(0.0, 40.0, 0.0), ["judged", "high"]],
		],
		"farfield_shots": [],
		"perf": [["lakeside crowd", Vector3(-6.0, 0.0, 2.0), 60.0, 0.0]],
		"perf_breakdown": [],
		"probe_route": [
			PackedStringArray([SliceDepiction.tag("street")]),
			PackedStringArray([SliceDepiction.tag("bakery")]),
			PackedStringArray([SliceDepiction.tag("street")]),
			PackedStringArray([SliceDepiction.tag("pier")]),
			PackedStringArray([SliceDepiction.tag("street")]),
		],
	}

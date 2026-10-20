## Lakeside, as a labelled STUB (S33-c §5.5; Q-S33c-1, operator 2026-10-18: ship it on `main`, labelled):
## flat ground, the world's real far field, and one grey walled box per place at its footprint. S33-d
## replaces it with the village behind this same file and interface (R-S33d-c-1).
##
## Data only; `SliceBoxDepiction` builds it. The frame is the lane's (step-33 §4.0): lane (x, y) mm ->
## scene (x / 1000 - 15, 0, 3 - y / 1000), so the lane's centre is the scene's origin. Footprints are
## S33-b's (`pr-s33b-lakeside-bodies.md` §5.3). Doorways are the lane-side points the world's passages
## state on `main` today -- S33-b §5.2's "before" column; when S33-b moves them to 400 mm, this table
## takes its "after" column (R-S33c-b-1). The halt's three places (S29-a) are not drawn: S29-a has not
## merged, so they are reported as misses if they exist, never invented.
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
		_place(tag("cottages"), "Cottages", Rect2(0, 6000, 6000, 14000), Vector2(3000, 5800)),
		_place(tag("boathouse"), "Boathouse", Rect2(6000, 6000, 6000, 12000), Vector2(9000, 5800)),
		_place(tag("bakery"), "Bakery", Rect2(12000, 6000, 6000, 8000), Vector2(15000, 5800)),
		_place(tag("square"), "Square", Rect2(18000, 6000, 12000, 12000), Vector2(21000, 5800)),
		_place(tag("shore"), "Shore", Rect2(0, -10000, 15600, 10000), Vector2(12000, 200)),
		_place(tag("pier"), "Pier", Rect2(16000, -20000, 4000, 20000), Vector2(18000, 200)),
	]


static func _place(t: String, label: String, footprint: Rect2, doorway: Vector2) -> Dictionary:
	return {"tags": PackedStringArray([t]), "label": label, "footprint": footprint, "doorway": doorway}


## Offline only: the cottages' yard. Connected, the body is put where the world says.
func spawn() -> Dictionary:
	return {"position": Vector3(-12.0, 0.1, -10.0), "yaw": 180.0}


## Step-33 §9.1's provisional viewpoints, in this frame (S33-a and S33-d fix them; the halt's VP-L-6
## waits for S29-a). No perf views and no far-field shots until S33-d.
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
		"perf": [],
		"perf_breakdown": [],
		"probe_route": [
			PackedStringArray([SliceDepiction.tag("street")]),
			PackedStringArray([SliceDepiction.tag("bakery")]),
			PackedStringArray([SliceDepiction.tag("street")]),
			PackedStringArray([SliceDepiction.tag("pier")]),
			PackedStringArray([SliceDepiction.tag("street")]),
		],
	}

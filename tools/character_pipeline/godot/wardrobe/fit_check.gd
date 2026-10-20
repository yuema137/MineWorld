# SPDX-License-Identifier: MIT
# S32-p W-9 harness (pr-s32p-wardrobe-pilot.md §4.8): fit garments at load in GDScript and write the fitted
# positions, so the driver compares them with the Python and Blender fits of the same binding (<= 0.5 mm).
# Also times each fit (the per-change rebuild time of §4.11) and checks the garment GLB's vertex count against its
# binding (an importer that re-ordered or welded vertices would break the binding).
#
#   godot --headless --path <project> --script res://fit_check.gd -- <job.json>
# job.json: {"template": <abs .bin>, "weights": <abs .bin>, "out_dir": <abs>,
#            "garments": [{"id": .., "glb": <abs>, "binding": <abs>, "rigid": -1}], "repeat": 20}
extends SceneTree

const Fit := preload("res://fit_at_load.gd")


func _write_positions(path: String, positions: PackedVector3Array) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_buffer("MWGT".to_ascii_buffer())
	f.store_32(positions.size())
	for p in positions:
		f.store_float(p.x)
		f.store_float(p.y)
		f.store_float(p.z)


func _init() -> void:
	var args := OS.get_cmdline_user_args()
	var job: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(args[0]))
	var template := Fit.read_template(job["template"])
	var tw := Fit.read_weights(job["weights"])
	var report := {"garments": {}}
	var ok := true
	for g in job["garments"]:
		var binding := Fit.read_binding(g["binding"])
		var arrays := Fit.garment_arrays(g["glb"])
		var rest: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX] if not arrays.is_empty() else PackedVector3Array()
		var entry := {"glb_vertices": rest.size(), "binding_records": binding.get("count", -1)}
		if rest.size() != int(binding.get("count", -1)):
			entry["error"] = "vertex count differs from the binding"
			ok = false
			report["garments"][g["id"]] = entry
			continue
		var times := []
		var mesh: ArrayMesh = null
		for i in int(job.get("repeat", 20)):
			var t0 := Time.get_ticks_usec()
			mesh = Fit.build_part(arrays, binding, template, tw, int(g.get("rigid", -1)))
			times.append(Time.get_ticks_usec() - t0)
		times.sort()
		entry["fit_ms_median"] = times[times.size() / 2] / 1000.0
		entry["surfaces"] = mesh.get_surface_count()
		var fitted := Fit.fit_positions(binding, template)
		_write_positions(job["out_dir"] + "/" + g["id"] + "_godot.bin", fitted)
		var built: PackedVector3Array = mesh.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]
		entry["built_vertices"] = built.size()
		report["garments"][g["id"]] = entry
		print("[FIT] ", g["id"], " ", JSON.stringify(entry))
	report["ok"] = ok
	var f := FileAccess.open(job["out_dir"] + "/fit_check.json", FileAccess.WRITE)
	f.store_string(JSON.stringify(report, "  "))
	print("[FIT] done ok=", ok)
	quit(0 if ok else 1)

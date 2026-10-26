## RL-b2 trial levers (SC-1; `.structured-coding/plans/mvp1/pr-rl-b2-two-views-and-textures.md`
## §4.2). A trial is a named candidate change, applied at load by `--trial=<a+b+…>`, so one
## operator session can measure every candidate without a code commit per candidate. With no
## `--trial` argument the only effect is the line `trial  none`; the frame is unchanged.
##
class_name SliceTrial
extends RefCounted

const NAMES := ["none", "shadow2", "shadow60", "lod2", "cells48"]
const INVALID := "__invalid__"


## The `+`-separated trial names of `--trial=`, or ["none"]. An unknown name is an error:
## returns [INVALID] so that a typo cannot measure the base under a lever's name (X-4).
static func requested() -> PackedStringArray:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--trial="):
			var names := PackedStringArray(a.substr(8).split("+"))
			for n in names:
				if not n in NAMES:
					push_error("unknown trial: %s" % n)
					return PackedStringArray([INVALID])
			return names
	return PackedStringArray(["none"])


## The batching cell size: 48 m only when `cells48` is requested (T-4 is build-time).
static func cell_size() -> float:
	return 48.0 if "cells48" in requested() else SliceBatch.CELL


## The requested names joined by `+`, for the run record.
static func label() -> String:
	return "+".join(requested())


## Applies the run-time levers to the slice after its build and GI setup. Returns false when
## a name is unknown; the caller then quits with a non-zero exit and writes no record.
static func apply(slice: Node) -> bool:
	var names := requested()
	if names.size() == 1 and names[0] == INVALID:
		return false
	var sun := slice.find_child("Sun", true, false) as DirectionalLight3D
	for n in names:
		match n:
			# T-1: two cascades; the near one covers 0 to 15.2 m.
			"shadow2":
				sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS
				sun.directional_shadow_split_1 = 0.16
			# T-2: 60 m; the split fractions rescaled by 95/60 keep the absolute ranges
			# 5.2 / 15.2 / 39.9 m (0.055, 0.16, 0.42 x 95/60).
			"shadow60":
				sun.directional_shadow_max_distance = 60.0
				sun.directional_shadow_split_1 = 0.0871
				sun.directional_shadow_split_2 = 0.2533
				sun.directional_shadow_split_3 = 0.6650
			# T-3: mesh LOD threshold 2 px (Godot's default is 1 px).
			"lod2":
				slice.get_viewport().mesh_lod_threshold = 2.0
	print("trial  %s" % label())
	return true

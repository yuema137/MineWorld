## A skinned surface's vertices, bone indices and weights, read back and checked before they are used
## (`GroundContact`, `SeatContact`).
##
## Why it exists. On a rendering device, `Mesh.surface_get_arrays` reads the surface's buffers back from
## the GPU. On the macOS CI runners' paravirtual Metal device that readback waits on a fence that times
## out ("timeout waiting for fence"), and the town figure's bone arrays came back holding values that
## name no bind: 61 984 and 14 528 against its 52 binds (nightly 40412844680). `GroundContact` and
## `SeatContact` indexed past their arrays, so the town figure kept no contact data and every Lakeside seat
## row with it failed by -454 mm.
##
## What it does. A read is accepted only when it is whole: the bone and weight arrays have one entry per
## vertex per influence (4 or 8), every bone index names one of the skin's binds, and every weight lies in
## [0, 1]. A read that is not whole is taken again, up to READS times in all, each after the rendering
## server has synced. A surface that never reads back whole is reported by name and returned as
## `{"unreadable": why}`: the caller treats the body as unmeasured and never uses the bad values.
class_name SkinArrays
extends RefCounted

## How many times a surface is read before it is reported as unreadable.
const READS := 3
## The influences per vertex Godot stores: 4, or 8 with `ARRAY_FLAG_USE_8_BONE_WEIGHTS`.
const INFLUENCES := [4, 8]


## `{"v": PackedVector3Array, "bi": PackedInt32Array, "bw": PackedFloat32Array, "per": int}` for surface
## `surface` of `mesh`, whose skin has `binds` binds; {} when the surface carries no skin weights; and
## `{"unreadable": why}`, with an error naming `what`, when it never reads back whole.
static func read(mesh: Mesh, surface: int, binds: int, what: String) -> Dictionary:
	var why := ""
	for attempt in READS:
		if attempt > 0:
			RenderingServer.force_sync()
		var arr := mesh.surface_get_arrays(surface)
		var v: Variant = arr[Mesh.ARRAY_VERTEX]
		var bi: Variant = arr[Mesh.ARRAY_BONES]
		var bw: Variant = arr[Mesh.ARRAY_WEIGHTS]
		if typeof(v) != TYPE_PACKED_VECTOR3_ARRAY or (v as PackedVector3Array).is_empty():
			return {}
		if bi == null or bw == null:
			return {}
		var bones := PackedInt32Array(bi)
		var weights := PackedFloat32Array(bw)
		if bones.is_empty() and weights.is_empty():
			return {}
		why = _whole(v as PackedVector3Array, bones, weights, binds)
		if why == "":
			return {"v": v, "bi": bones, "bw": weights, "per": bones.size() / (v as PackedVector3Array).size()}
	push_error("SkinArrays: %s surface %d did not read back whole in %d reads (%s); it is not measured"
		% [what, surface, READS, why])
	return {"unreadable": why}


## "" when the arrays are whole, else what is wrong with them.
static func _whole(v: PackedVector3Array, bi: PackedInt32Array, bw: PackedFloat32Array, binds: int) -> String:
	var per := bi.size() / v.size()
	if not per in INFLUENCES or bi.size() != v.size() * per or bw.size() != bi.size():
		return "%d bone and %d weight entries for %d vertices" % [bi.size(), bw.size(), v.size()]
	for i in bi.size():
		if bi[i] < 0 or bi[i] >= binds:
			return "bone index %d at entry %d names no bind of %d" % [bi[i], i, binds]
		var w := bw[i]
		if not (w >= 0.0 and w <= 1.0):
			return "weight %f at entry %d is outside [0, 1]" % [w, i]
	return ""

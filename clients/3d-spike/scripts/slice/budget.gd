## The slice's frame-budget passes, run once after `SliceBatch.merge`
## (RL-b, `.structured-coding/plans/mvp1/pr-rl-b-3d-budget.md` SD-RLb-3, SD-RLb-4).
##
## SCREEN-SIZE CULLING, invisible by construction. Every drawn instance that is
## not a merged cell, the ground, the backdrop or a character gets a
## `visibility_range_end` at the distance where its whole bounding sphere covers
## less than `MIN_PX` pixels on a 1080-line frame. Beyond that it cannot
## contribute a visible pixel, so hiding it changes nothing on screen -- and a
## hidden instance also leaves the depth and shadow passes, which is where most
## of a small prop's cost is. The distance follows from the sphere's projected
## diameter, 2 r / d * (H / 2) / tan(fov / 2) < p, so d > r H / (tan(fov / 2) p).
##
## The field of view is the camera rig's own constant, read here, not copied:
## a wider lens would otherwise cull what it can still see. At a resolution above
## 1080 lines the bound is conservative only by the ratio; RL-i passes a tier's H.
##
## Godot measures a visibility range from the camera to the instance's world
## AABB centre; the radius used is the larger of that AABB's half-diagonal and the
## farthest AABB corner from the instance's origin, so the sphere bounds the
## object whichever point the distance is taken to.
class_name SliceBudget
extends RefCounted

## The Default tier's frame height (step-22 §6.6).
const RENDER_H := 1080.0
## An object is hidden only once its bounding sphere covers less than this.
const MIN_PX := 1.5
## A-5's bound, kept apart from the constant it checks: a range computed with a
## larger `MIN_PX` (X-1) must be reported, not silently agreed with.
const A5_MAX_PX := 1.5
## Never ranged: their own cells are large and always partly near, or they are
## the far field itself.
const EXCLUDED := ["merged", "foliage", "ground", "backdrop"]


## Apply the passes under `root`; returns what they did, for the build log.
static func apply(root: Node3D) -> Dictionary:
	var tan_half := tan(deg_to_rad(CameraRig.FOV) * 0.5)
	var k := RENDER_H / (tan_half * MIN_PX)
	var st := {"ranged": 0, "max_px": 0.0, "violations": 0}
	_range(root, false, k, tan_half, st)
	return st


static func _range(n: Node, excluded: bool, k: float, tan_half: float, st: Dictionary) -> void:
	if n.has_meta("mw_category") and str(n.get_meta("mw_category")) in EXCLUDED:
		excluded = true
	if n is GeometryInstance3D and not excluded:
		var gi := n as GeometryInstance3D
		var r := radius(gi)
		if r > 0.0:
			var end := r * k
			gi.visibility_range_end = end
			gi.visibility_range_end_margin = 0.1 * end
			gi.visibility_range_fade_mode = GeometryInstance3D.VISIBILITY_RANGE_FADE_DISABLED
			# A-5, asserted rather than trusted: the projected diameter at `end`
			var px := 2.0 * r / end * (RENDER_H * 0.5) / tan_half
			st["max_px"] = maxf(st["max_px"], px)
			if px > A5_MAX_PX + 1e-4:
				st["violations"] += 1
				push_error("budget: %s ranged at %.1f m covers %.2f px" % [gi.get_path(), end, px])
			st["ranged"] += 1
	for c in n.get_children():
		_range(c, excluded, k, tan_half, st)


## The bounding radius used for the range: see the header.
static func radius(gi: GeometryInstance3D) -> float:
	var local := gi.get_aabb()
	if local.size == Vector3.ZERO:
		return 0.0
	var xf := gi.global_transform
	var world := xf * local
	var r := world.size.length() * 0.5
	for i in range(8):
		r = maxf(r, (xf * local.get_endpoint(i)).distance_to(xf.origin))
	return r

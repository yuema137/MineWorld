## RL-b2 SD-B2-6: the texel density of every downsized prop in the built slice. For each mesh
## surface under a prop (`mw_category = "gltf"`, slug from `mw_slug`), each triangle gives
## sqrt(uv_area * W * H / world_area) pixels per metre, with W, H the loaded albedo texture.
## A slug's figure is the area-weighted median. A downsized slug under its class floor fails.
## `CLASS_OF` must equal tools/slice_imports.py's SMALL_512 (X-2 checks the pairing).
class_name SliceTexelDensity
extends RefCounted

const CLASS_OF := {
	"croissant": "hand-held", "carrot_cake": "hand-held", "food_apple_01": "hand-held",
	"planter_pot_clay": "small", "calathea_orbifolia_01": "small",
	"industrial_pipe_lamp": "distant", "wall_clock": "distant", "fancy_picture_frame_01": "distant",
}
## hand-held goods: step-22 section 6.4; the other classes: DEP-8 (texel density of a walkable street).
const FLOOR := {"hand-held": 1024.0, "small": 512.0, "distant": 512.0}


## Prints one line per downsized slug and returns the number of slugs under their floor.
static func run(world: Node3D) -> int:
	var samples := {}  # slug -> Array of [density, world area]
	var sizes := {}  # slug -> "WxH" of the loaded albedo texture
	for n in world.find_children("*", "Node3D", true, false):
		if not n.has_meta("mw_category") or n.get_meta("mw_category") != "gltf":
			continue
		var slug: String = n.get_meta("mw_slug", "")
		for mi in n.find_children("*", "MeshInstance3D", true, false):
			_collect(mi as MeshInstance3D, slug, samples, sizes)
	var failures := 0
	var downsized := 0
	for slug in CLASS_OF:
		if not samples.has(slug):
			# a table entry the check cannot measure is a failure, not a pass
			failures += 1
			print("texel %-28s not found in the slice  MISSING" % slug)
			continue
		downsized += 1
		var cls: String = CLASS_OF[slug]
		var floor_px: float = FLOOR[cls]
		var median := _weighted_median(samples[slug])
		var ok: bool = median >= floor_px
		if not ok:
			failures += 1
		print("texel %-28s %-10s loaded %-9s median %7.1f px/m  floor %5.0f  %s"
			% [slug, cls, sizes[slug], median, floor_px, "ok" if ok else "UNDER"])
	print("texel density: %d downsized, %d under floor" % [downsized, failures])
	return failures


static func _collect(mi: MeshInstance3D, slug: String, samples: Dictionary,
		sizes: Dictionary) -> void:
	if mi == null or mi.mesh == null:
		return
	var mesh := mi.mesh
	var xf := mi.global_transform
	for s in range(mesh.get_surface_count()):
		var mat := mi.get_active_material(s)
		if not (mat is BaseMaterial3D):
			continue
		var tex := (mat as BaseMaterial3D).albedo_texture
		if tex == null:
			continue
		var arrays := mesh.surface_get_arrays(s)
		var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
		var uvs: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV]
		if uvs.size() != verts.size():
			continue
		var index: PackedInt32Array = arrays[Mesh.ARRAY_INDEX] if arrays[Mesh.ARRAY_INDEX] != null \
			else PackedInt32Array(range(verts.size()))
		var px := float(tex.get_width()) * float(tex.get_height())
		sizes[slug] = "%dx%d" % [tex.get_width(), tex.get_height()]
		if not samples.has(slug):
			samples[slug] = []
		for t in range(0, index.size() - 2, 3):
			var ia: int = index[t]
			var ib: int = index[t + 1]
			var ic: int = index[t + 2]
			var wa := xf * verts[ia]
			var wb := xf * verts[ib]
			var wc := xf * verts[ic]
			var world_area := 0.5 * (wb - wa).cross(wc - wa).length()
			if world_area < 1e-8:
				continue
			var ub := uvs[ib] - uvs[ia]
			var uc := uvs[ic] - uvs[ia]
			var uv_area := 0.5 * absf(ub.x * uc.y - ub.y * uc.x)
			(samples[slug] as Array).append([sqrt(uv_area * px / world_area), world_area])


## The area-weighted median of [density, area] pairs; 0 when there are none.
static func _weighted_median(pairs: Array) -> float:
	if pairs.is_empty():
		return 0.0
	pairs.sort_custom(func(x: Array, y: Array) -> bool: return x[0] < y[0])
	var total := 0.0
	for p in pairs:
		total += p[1]
	var acc := 0.0
	for p in pairs:
		acc += p[1]
		if acc >= total * 0.5:
			return p[0]
	return pairs[pairs.size() - 1][0]

# SPDX-License-Identifier: MIT
# S32-p R-12 (b): the fit-at-load prototype (pr-s32p-wardrobe-pilot.md §§4.3, 4.4; step-32 §6.2-6.3).
#
# A garment is one neutral-rest mesh (`garment.glb`, one surface, no skin) plus `binding.bin`. On a person P it is
# placed by MakeHuman's proxy formula on P's fitting template T_P (`<subject>_template.bin`):
#     p_k = sum_i w_i * T_P[v_i] + (ox * sx, oy * sy, oz * sz),   s_axis = |T_P[a].axis - T_P[b].axis| / rest
# Its skin weights are the template's (`<subject>_template_weights.bin`), interpolated with the same w_i; the
# weights file's joint table is in the body's glTF skin order, so a joint index IS a bind index of the body's Skin
# (the driver writes it so). Each part reuses the body MeshInstance3D's own Skin resource (D-p-2): no bind name is
# resolved at all, so neither the importer's `mixamorig:` -> `mixamorig_` rename nor a retarget's renaming can bite.
# The body's index buffer is rebuilt without the triangles whose three vertices all map (through `corr.bin`) to
# hidden template vertices (the union of the worn garments' `hides.bin`).
#
# The same formula is `mwcast/wardrobe/garment_format.py` `fit` (Python) and `blender/mpfb/assemble.py` (Blender);
# W-9 compares them. Every file reader checks its magic and its size.
extends RefCounted

const MAGIC_TEMPLATE := "MWGT"
const MAGIC_WEIGHTS := "MWGW"
const MAGIC_BINDING := "MWGB"
const MAGIC_HIDES := "MWGH"
const MAGIC_CORR := "MWGC"


static func _open(path: String, magic: String) -> FileAccess:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		push_error("fit_at_load: cannot open " + path)
		return null
	var got := f.get_buffer(4).get_string_from_ascii()
	if got != magic:
		push_error("fit_at_load: %s has magic %s, expected %s" % [path, got, magic])
		return null
	return f


static func read_template(path: String) -> PackedVector3Array:
	var out := PackedVector3Array()
	var f := _open(path, MAGIC_TEMPLATE)
	if f == null:
		return out
	var n := f.get_32()
	out.resize(n)
	for i in n:
		out[i] = Vector3(f.get_float(), f.get_float(), f.get_float())
	return out


static func read_weights(path: String) -> Dictionary:
	var f := _open(path, MAGIC_WEIGHTS)
	if f == null:
		return {}
	var n := f.get_32()
	var j := f.get_32()
	var names := PackedStringArray()
	for k in j:
		var length := f.get_8()
		names.append(f.get_buffer(length).get_string_from_utf8())
	var joints := PackedInt32Array()
	var weights := PackedFloat32Array()
	joints.resize(4 * n)
	weights.resize(4 * n)
	for i in 4 * n:
		joints[i] = f.get_8()
		weights[i] = f.get_float()
	return {"names": names, "joints": joints, "weights": weights}


static func read_binding(path: String) -> Dictionary:
	var f := _open(path, MAGIC_BINDING)
	if f == null:
		return {}
	var version := f.get_32()
	var n := f.get_32()
	var scale := []
	for axis in 3:
		scale.append([f.get_32(), f.get_32(), f.get_float()])
	var verts := PackedInt32Array()
	var w := PackedFloat32Array()
	var o := PackedFloat32Array()
	verts.resize(3 * n)
	w.resize(3 * n)
	o.resize(3 * n)
	for k in n:
		for i in 3:
			verts[3 * k + i] = f.get_32()
		for i in 3:
			w[3 * k + i] = f.get_float()
		for i in 3:
			o[3 * k + i] = f.get_float()
	return {"version": version, "count": n, "scale": scale, "verts": verts, "w": w, "o": o}


static func read_ids(path: String) -> PackedInt32Array:
	var out := PackedInt32Array()
	var f := _open(path, MAGIC_HIDES)
	if f == null:
		return out
	var n := f.get_32()
	out.resize(n)
	for i in n:
		out[i] = f.get_32()
	return out


static func read_corr(path: String) -> PackedInt32Array:
	# Per body vertex the three template vertex ids (the barycentric pair is not needed by the mask rule).
	var out := PackedInt32Array()
	var f := _open(path, MAGIC_CORR)
	if f == null:
		return out
	var n := f.get_32()
	out.resize(3 * n)
	for k in n:
		for i in 3:
			out[3 * k + i] = f.get_32()
		f.get_float()
		f.get_float()
	return out


static func fit_positions(binding: Dictionary, template: PackedVector3Array) -> PackedVector3Array:
	var s := Vector3.ZERO
	for axis in 3:
		var ref: Array = binding["scale"][axis]
		s[axis] = absf(template[ref[0]][axis] - template[ref[1]][axis]) / float(ref[2])
	var n: int = binding["count"]
	var verts: PackedInt32Array = binding["verts"]
	var w: PackedFloat32Array = binding["w"]
	var o: PackedFloat32Array = binding["o"]
	var out := PackedVector3Array()
	out.resize(n)
	for k in n:
		var p := template[verts[3 * k]] * w[3 * k] + template[verts[3 * k + 1]] * w[3 * k + 1] \
			+ template[verts[3 * k + 2]] * w[3 * k + 2]
		out[k] = p + Vector3(o[3 * k] * s.x, o[3 * k + 1] * s.y, o[3 * k + 2] * s.z)
	return out


## Skin weights of the garment's vertices: the template weights of each record's three vertices, combined with the
## record's weights (negative barycentrics clamped to 0), the four largest kept, normalised. `rigid` (a bind index,
## or -1) gives every vertex weight 1 on that joint (`weights: rigid <bone>`, e.g. the backpack on Spine2).
static func fit_weights(binding: Dictionary, tw: Dictionary, rigid: int) -> Array:
	var n: int = binding["count"]
	var verts: PackedInt32Array = binding["verts"]
	var w: PackedFloat32Array = binding["w"]
	var tj: PackedInt32Array = tw["joints"]
	var twt: PackedFloat32Array = tw["weights"]
	var bones := PackedInt32Array()
	var weights := PackedFloat32Array()
	bones.resize(4 * n)
	weights.resize(4 * n)
	for k in n:
		var acc := {}
		if rigid >= 0:
			acc[rigid] = 1.0
		else:
			for i in 3:
				var bw := maxf(w[3 * k + i], 0.0)
				var v := verts[3 * k + i]
				for m in 4:
					var x := twt[4 * v + m] * bw
					if x > 0.0:
						acc[tj[4 * v + m]] = acc.get(tj[4 * v + m], 0.0) + x
		var keys := acc.keys()
		keys.sort_custom(func(a, b): return acc[a] > acc[b] or (acc[a] == acc[b] and a < b))
		var total := 0.0
		for m in mini(4, keys.size()):
			total += acc[keys[m]]
		for m in 4:
			if m < keys.size() and total > 0.0:
				bones[4 * k + m] = keys[m]
				weights[4 * k + m] = acc[keys[m]] / total
			else:
				bones[4 * k + m] = 0
				weights[4 * k + m] = 0.0
	return [bones, weights]


## The neutral-rest arrays of a garment GLB (its one surface), read at run time without the import pipeline.
static func garment_arrays(path: String) -> Array:
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if doc.append_from_file(path, state) != OK:
		push_error("fit_at_load: cannot read " + path)
		return []
	var scene := doc.generate_scene(state)
	var found: Array = scene.find_children("*", "MeshInstance3D", true, false)
	var arrays: Array = []
	if not found.is_empty():
		var mi: MeshInstance3D = found[0]
		arrays = mi.mesh.surface_get_arrays(0)
		var material: Material = mi.mesh.surface_get_material(0)
		arrays.append(material)
	scene.free()
	return arrays


## One fitted part: an ArrayMesh in the body's mesh space, skinned with the body's own Skin.
static func build_part(arrays_with_material: Array, binding: Dictionary, template: PackedVector3Array,
		tw: Dictionary, rigid: int) -> ArrayMesh:
	var arrays := arrays_with_material.slice(0, Mesh.ARRAY_MAX)
	var material: Material = arrays_with_material[Mesh.ARRAY_MAX]
	var rest: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
	if rest.size() != int(binding["count"]):
		push_error("fit_at_load: garment has %d vertices, binding %d" % [rest.size(), binding["count"]])
		return null
	var bw := fit_weights(binding, tw, rigid)
	var out := []
	out.resize(Mesh.ARRAY_MAX)
	out[Mesh.ARRAY_VERTEX] = fit_positions(binding, template)
	out[Mesh.ARRAY_TEX_UV] = arrays[Mesh.ARRAY_TEX_UV]
	out[Mesh.ARRAY_INDEX] = arrays[Mesh.ARRAY_INDEX]
	out[Mesh.ARRAY_BONES] = bw[0]
	out[Mesh.ARRAY_WEIGHTS] = bw[1]
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, out)
	# Normals and tangents of the fitted shape (SurfaceTool keeps the skin arrays).
	var st := SurfaceTool.new()
	st.create_from(mesh, 0)
	st.generate_normals()
	if out[Mesh.ARRAY_TEX_UV] != null:
		st.generate_tangents()
	var fitted := st.commit()
	if material != null:
		fitted.surface_set_material(0, material)
	return fitted


## The body's indices without the hidden triangles (a triangle is hidden when all three of its vertices map to
## hidden template vertices). `corr` is `read_corr`'s array.
static func masked_indices(indices: PackedInt32Array, corr: PackedInt32Array, hidden: Dictionary) -> PackedInt32Array:
	var out := PackedInt32Array()
	for t in indices.size() / 3:
		var all_hidden := true
		for i in 3:
			var v := indices[3 * t + i]
			if not (hidden.has(corr[3 * v]) and hidden.has(corr[3 * v + 1]) and hidden.has(corr[3 * v + 2])):
				all_hidden = false
				break
		if not all_hidden:
			out.append_array([indices[3 * t], indices[3 * t + 1], indices[3 * t + 2]])
	return out

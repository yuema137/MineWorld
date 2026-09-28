## Merging the slice's primitive geometry, because a mouldings-and-dentils
## facade is thousands of draw calls and a frame is not free.
##
## MEASURED, not assumed. The first complete build of the slice put 5413 nodes
## in the tree, nearly all of them a `BoxMesh` with a `material_override` --
## one cornice course, one dentil, one glazing bar, one shelf, one canister
## each. A frame therefore issued on the order of thirty thousand draw calls
## once the depth pass and four shadow splits are counted, and a windowed
## capture that should take twenty seconds was still running after six minutes.
## Profiling the hung process showed the main thread inside
## `drawIndexedPrimitives`, which is the answer: it was not stuck, it was
## drawing.
##
## Godot 4 does no automatic static batching, so this does it explicitly, after
## the scene is built and before it is first drawn:
##
##   1. walk the slice, collecting every MeshInstance3D whose mesh is a
##      primitive and whose material comes from `material_override`;
##   2. group by (material, shadow setting, 24 m spatial cell);
##   3. bake each group into one ArrayMesh in the slice's own frame;
##   4. free the originals.
##
## Three things this deliberately does **not** touch:
##
##   Poly Haven props, which carry per-surface materials rather than an
##   override, and are already one or two draws each;
##   collision, which lives on separate StaticBody3D nodes and is untouched --
##   merging must not change where a wall is;
##   Label3D signage, which is its own thing and must stay individually
##   transformed.
##
## The spatial cell matters: one mesh for the whole street would defeat frustum
## culling, and a frame inside the cafe would still be paying for the far end
## of the frontage. 24 m is about a block.
class_name SliceBatch
extends RefCounted

const CELL := 24.0

## Every primitive mesh class the slice builds through `build.gd`. A mesh that
## is not one of these is left alone rather than guessed at.
const MERGEABLE := ["BoxMesh", "CylinderMesh", "PrismMesh", "SphereMesh",
	"QuadMesh", "PlaneMesh"]


## Merge `root` in place. Returns what it did, for the record.
static func merge(root: Node3D) -> Dictionary:
	var groups: Dictionary = {}          # key -> {mat, shadow, arrays: Array}
	var victims: Array[MeshInstance3D] = []
	_collect(root, root, groups, victims)

	var before := victims.size()
	for v in victims:
		v.get_parent().remove_child(v)
		v.queue_free()

	var made := 0
	var tris := 0
	for key in groups:
		var g: Dictionary = groups[key]
		var st := SurfaceTool.new()
		st.begin(Mesh.PRIMITIVE_TRIANGLES)
		for item in g["items"]:
			tris += _append(st, item[0], item[1])
		st.index()
		st.generate_tangents()
		var mi := MeshInstance3D.new()
		mi.name = "Merged_%d" % made
		mi.mesh = st.commit()
		mi.material_override = g["mat"]
		mi.cast_shadow = g["shadow"]
		root.add_child(mi)
		made += 1
	return {"merged_from": before, "merged_into": made, "triangles": tris}


static func _collect(node: Node, root: Node3D, groups: Dictionary,
		victims: Array[MeshInstance3D]) -> void:
	for c in node.get_children():
		_collect(c, root, groups, victims)
	if not (node is MeshInstance3D):
		return
	var mi := node as MeshInstance3D
	if mi.mesh == null or mi.material_override == null:
		return
	if not (mi.mesh.get_class() in MERGEABLE):
		return
	# The transform that takes this mesh into the slice's own frame. Computed
	# from global transforms so a node nested three deep inside a building
	# inside the slice lands where it was drawn.
	var xf := root.global_transform.affine_inverse() * mi.global_transform
	var cell := (xf.origin / CELL).floor()
	var key := "%s|%d|%d,%d,%d" % [mi.material_override.get_instance_id(),
		mi.cast_shadow, int(cell.x), int(cell.y), int(cell.z)]
	if not groups.has(key):
		groups[key] = {"mat": mi.material_override, "shadow": mi.cast_shadow,
			"items": []}
	groups[key]["items"].append([mi.mesh, xf])
	victims.append(mi)


## Append one mesh's triangles, transformed. Normals go through the basis
## inverse-transpose; every transform here is a rotation and a translation, so
## the basis itself would do, but the general form costs nothing and does not
## silently break if a scale is ever introduced.
static func _append(st: SurfaceTool, mesh: Mesh, xf: Transform3D) -> int:
	var arrays := mesh.surface_get_arrays(0)
	var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
	var norms: PackedVector3Array = arrays[Mesh.ARRAY_NORMAL]
	var uvs: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV]
	var idx: PackedInt32Array = arrays[Mesh.ARRAY_INDEX]
	var nb := xf.basis.inverse().transposed()
	var has_n := norms != null and norms.size() == verts.size()
	var has_uv := uvs != null and uvs.size() == verts.size()
	var order: PackedInt32Array = idx
	if order == null or order.is_empty():
		order = PackedInt32Array()
		for i in range(verts.size()):
			order.append(i)
	for i in order:
		if has_n:
			st.set_normal((nb * norms[i]).normalized())
		if has_uv:
			st.set_uv(uvs[i])
		st.add_vertex(xf * verts[i])
	return order.size() / 3

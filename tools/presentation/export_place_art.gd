extends SceneTree

## Exports a world's place art to glTF pieces in a 3D Presentation Pack (S14 16f-d1; design
## `.structured-coding/plans/mvp0/pr-16f-presentation-pack-3d.md` §4.5).
##
## The art is built by an AUTHORING SOURCE, a script that builds named pieces with the 3D client's
## builders (`social_cafe/source.gd` is Social Café's street). This tool knows no world: it builds each
## piece the source lists, makes it a glTF scene, and writes, under the pack:
##
##   assets/places/<asset-id>/<asset-id>.glb   the piece: meshes, lights, and the named nodes below
##   assets/places/<asset-id>/asset.yaml       PACKAGE_FORMAT.md §3's contract (YAML's JSON subset)
##   materials/material_rules.yaml             what glTF cannot carry, re-bound by name on load
##   materials/textures/<md5>.png              a texture made at build time (a leaf card), by content
##
## WHAT glTF CANNOT CARRY, AND HOW IT TRAVELS. glTF holds meshes, a metallic-roughness material, lights
## and empty nodes. The slice's art is more: world-triplanar library materials, Label3D signage in a
## system serif, colliders, reflection probes. Nothing is dropped and nothing is approximated here.
## Every material is named `m_<hash>` and every node is named `<kind>_<hash>_<n>`, where `<hash>` is
## the first 12 hex digits of the MD5 of a RECIPE: the object's class and every stored property that
## differs from a fresh instance of that class, sub-resources inline, library files by path. The
## recipe is written once into `material_rules.yaml`; the client re-binds by the name (16f-d2). Kinds:
##
##   mesh   a MeshInstance3D: its geometry and materials are glTF; the recipe holds the instance's
##          settings (shadow casting, the frame-cost category)
##   light  a Light3D: also a KHR_lights_punctual light; the recipe holds what that extension lacks
##   node   anything else (a Label3D, a body, a collision shape, a probe): an empty node carries the
##          transform, the recipe holds the rest
##   g      a plain Node3D group: no recipe
##
## Two named kinds come from the source, not from a recipe: ENTRANCES (one empty node per drawn door,
## at the door's street-side out-point, facing out), and SPACES (one empty node per room volume, its
## scale the volume's size). `asset.yaml` lists them in `semantic_bindings`.
##
## Same input, same bytes (I-16f-7): every name is a counter in tree order or a content hash, and the
## builders draw no unseeded random number. AC-16f-11 runs it twice into scratch and compares with `cmp`.
##
## WHAT IS REFERENCED, NOT COPIED. A library file of the 3D client is referenced by its path, as S33-d's
## files reference texture slugs: a texture (`assets/textures/...`), and a whole library scene, such as
## a Poly Haven prop the client imports with its LODs and shadow meshes (`assets/models/...`). Only
## geometry MineWorld built is in the `.glb`. Every library path must fall under a root the source gives
## a licence for; a path under none refuses the export, so an asset without a licence is never written.
##
##   godot --headless --path clients/3d-spike --script <repo>/tools/presentation/export_place_art.gd -- \
##       --source=<repo>/tools/presentation/social_cafe/source.gd \
##       --pack=<repo>/presentation/mineworld-default/3D [--out=<dir>] [--depictions=<dir>]
##
##   --out=<dir>         write the assets and the rules under <dir> instead of the pack (a scratch run)
##   --depictions=<dir>  also write the source's depiction files there (`<world>.yaml`)
##
## Prints one `EXPORT <asset-id> ...` line per piece and `export_place_art: ok`, or
## `export_place_art: FAIL <why>` and exits 1.

const ASSETS_DIR := "assets/places"
const RULES_FILE := "materials/material_rules.yaml"
const TEXTURES_DIR := "materials/textures"
## The meta that marks an entrance or a space node: named by the source, never renamed.
const SEMANTIC := "mw_semantic"

## Properties a recipe never holds: the transform travels in glTF, the rest is bookkeeping.
const SKIP := ["transform", "position", "rotation", "rotation_degrees", "quaternion", "basis", "scale",
	"rotation_edit_mode", "rotation_order", "top_level", "global_transform", "global_position",
	"global_rotation", "global_rotation_degrees", "global_basis", "name", "owner", "script",
	"unique_name_in_owner", "scene_file_path", "process_mode", "process_priority",
	"process_physics_priority", "process_thread_group", "physics_interpolation_mode",
	"auto_translate_mode", "editor_description", "resource_name", "resource_path",
	"resource_local_to_scene", "resource_scene_unique_id", "visibility_parent", "skeleton", "skin",
	"mesh", "material_override", "material_overlay"]

var _source: Object = null
var _pack := ""
var _out := ""
var _materials := {}     # name -> recipe
var _nodes := {}         # name -> recipe
var _textures := {}      # file -> PackedByteArray (PNG)
var _material_of := {}   # recipe hash -> the one named Material every surface with it shares
var _mesh_of := {}       # source mesh and materials -> the one ArrayMesh exported for them
var _counter := 0


## The builders need the scene tree (global transforms), which is ready after the first frame starts.
func _initialize() -> void:
	_run.call_deferred()


func _run() -> void:
	var source_path := ""
	var depictions := ""
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--source="):
			source_path = arg.substr(9)
		elif arg.begins_with("--pack="):
			_pack = arg.substr(7)
		elif arg.begins_with("--out="):
			_out = arg.substr(6)
		elif arg.begins_with("--depictions="):
			depictions = arg.substr(13)
	if source_path == "" or _pack == "":
		_fail("usage: -- --source=<source.gd> --pack=<3D pack dir> [--out=<dir>] [--depictions=<dir>]")
		return
	if _out == "":
		_out = _pack
	var script := load(source_path) as Script
	if script == null:
		_fail("cannot load the source %s" % source_path)
		return
	_source = script.new()
	var licences: Array = _source.licences()
	var assets := []
	for piece in _source.pieces():
		var why := _export_piece(piece, licences)
		if why != "":
			_fail(why)
			return
		assets.append(piece["id"])
	_write_text(_out.path_join(RULES_FILE), _rules_text())
	for file in _textures:
		_write_bytes(_out.path_join(TEXTURES_DIR).path_join(file), _textures[file])
	if depictions != "":
		for world in _source.worlds():
			var data: Dictionary = _source.depiction(world, _pack)
			_write_text(depictions.path_join("%s.yaml" % world), _json(data))
	print("export_place_art: ok (%d assets, %d materials, %d node recipes, %d textures)"
		% [assets.size(), _materials.size(), _nodes.size(), _textures.size()])
	# the props' scene cache holds nodes outside the tree; freed so the exit reports no leak
	for n in Props._scenes.values():
		if n != null:
			n.free()
	Props._scenes.clear()
	quit(0)


func _fail(why: String) -> void:
	printerr("export_place_art: FAIL %s" % why)
	quit(1)


# --- one piece -------------------------------------------------------------------------------------

## Builds `piece` (the source's {id, build, frame_origin, entrances, spaces}) and writes its asset.
func _export_piece(piece: Dictionary, licences: Array) -> String:
	var id: String = piece["id"]
	# The piece is built, and its doors and rooms are read, in the slice's scene coordinates; then the
	# whole is moved so that the scene point `frame_origin` becomes the asset's (0, 0, 0).
	var top := Node3D.new()
	top.name = "export"
	root.add_child(top)
	var scene := Node3D.new()
	scene.name = "scene"
	top.add_child(scene)
	SliceTerrace.doors.clear()
	(piece["build"] as Callable).call(scene)
	var entrances: Variant = _entrances(scene, piece)
	if entrances is String:
		return "%s: %s" % [id, entrances]
	var spaces := _spaces(scene, piece)
	scene.position = -(piece["frame_origin"] as Vector3)
	var merged := SliceBatch.merge(top)
	var used := {}       # library file -> true
	_counter = 0
	_mesh_of.clear()     # keyed by live meshes: never across pieces, whose meshes are freed
	_normalise(top, used)
	var asset_dir := _out.path_join(ASSETS_DIR).path_join(id)
	DirAccess.make_dir_recursive_absolute(asset_dir)
	var glb := asset_dir.path_join("%s.glb" % id)
	var doc := GLTFDocument.new()
	doc.image_format = "None"
	var state := GLTFState.new()
	var err := doc.append_from_scene(top, state)
	if err == OK:
		err = doc.write_to_filesystem(state, glb)
	root.remove_child(top)
	top.free()
	if err != OK:
		return "%s: glTF export failed (%d)" % [id, err]
	var sources: Variant = _sources(used, licences)
	if sources is String:
		return "%s: %s" % [id, sources]
	var check := _reload(glb, entrances, spaces)
	if check != "":
		return "%s: %s" % [id, check]
	_write_text(asset_dir.path_join("asset.yaml"), _json(_asset_yaml(piece, entrances, spaces, sources)))
	print("EXPORT %s %d bytes, %d merged meshes from %d primitives, %d entrances, %d spaces"
		% [id, FileAccess.get_file_as_bytes(glb).size(), merged["merged_into"], merged["merged_from"],
			entrances.size(), spaces.size()])
	return ""


## One empty node per drawn door the piece registered (`SliceTerrace.doors`, D-16c-4), at the door's
## street-side out-point, facing out; named by the source. Returns [{name, label, space, depth_m,
## floor_m}], or a String when the source names a door that is not there.
func _entrances(top: Node3D, piece: Dictionary) -> Variant:
	var names: Dictionary = piece.get("entrances", {})
	var out := []
	var i := 0
	for door in SliceTerrace.doors:
		var label: String = door[2]
		var name := ""
		for n in names:
			if (names[n] as Callable).call(door):
				name = n
		if name == "":
			name = "door_%02d" % i
		i += 1
		var at := SliceLayout.out_point(door)
		var e := Node3D.new()
		e.name = name
		e.position = at
		e.rotation.y = float(door[1])
		e.set_meta(SEMANTIC, true)
		top.add_child(e)
		# the room behind the door: the asset's space the source names for the door's scene volume
		var space: Variant = piece.get("volume_spaces", {}).get(str(door[3]), null)
		out.append({"name": name, "label": label, "space": space, "depth_m": float(door[4]),
			"floor_m": float(door[5])})
	for n in names:
		if not out.any(func(e: Dictionary) -> bool: return e["name"] == n):
			return "the source names the entrance %s, and no drawn door matches it" % n
	return out


## One empty node per room volume the source names: its centre, its scale the volume's size.
func _spaces(top: Node3D, piece: Dictionary) -> Array:
	var out := []
	var spaces: Dictionary = piece.get("spaces", {})
	for name in spaces:
		var box: AABB = (spaces[name] as Callable).call()
		var s := Node3D.new()
		s.name = name
		s.position = box.get_center()
		s.scale = box.size
		s.set_meta(SEMANTIC, true)
		top.add_child(s)
		out.append({"name": name, "size_m": [box.size.x, box.size.y, box.size.z]})
	return out


# --- what glTF cannot carry --------------------------------------------------------------------------

## Names every node (tree order), bakes every active material into its mesh, and replaces each node glTF
## cannot carry with an empty node of the same transform. `used` collects the library files referenced.
func _normalise(node: Node, used: Dictionary) -> void:
	var top := node.get_parent() == root
	if not top and node is Node3D and node.scene_file_path != "":
		_library_scene(node as Node3D, used)
		return
	for child in node.get_children():
		_normalise(child, used)
	if top or not (node is Node3D):
		return
	var n := node as Node3D
	if n.has_meta(SEMANTIC):
		return
	_counter += 1
	if n is MeshInstance3D:
		_bake_materials(n as MeshInstance3D, used)
		n.name = "mesh_%s_%d" % [_remember(_nodes, _recipe(n, used)), _counter]
	elif n is Light3D:
		n.name = "light_%s_%d" % [_remember(_nodes, _recipe(n, used)), _counter]
	elif n.get_class() == "Node3D":
		n.name = "g_%d" % _counter
	else:
		_stand_in(n, _recipe(n, used), true)


## A library scene (a Poly Haven prop the client imports with its LODs and shadow meshes) is not copied
## into the asset: like a library texture, it is referenced by path, with the material every surface
## of it is drawn with (the slice re-tints props). Its node becomes an empty stand-in.
func _library_scene(n: Node3D, used: Dictionary) -> void:
	_counter += 1
	used[n.scene_file_path] = true
	var surfaces := []
	for mi in n.find_children("*", "MeshInstance3D", true, false):
		var m := mi as MeshInstance3D
		if m.mesh == null:
			continue
		for i in m.mesh.get_surface_count():
			var mat := m.get_active_material(i)
			var name := "" if mat == null else "m_%s" % _remember(_materials, _recipe(mat, used))
			surfaces.append([str(n.get_path_to(m)), i, name])
	var recipe := _recipe(n, used)
	recipe["library"] = n.scene_file_path.trim_prefix("res://")
	recipe["surfaces"] = surfaces
	_stand_in(n, recipe, false)


## Replaces `n` by an empty node of its transform, named by its recipe; keeps or drops its children.
func _stand_in(n: Node3D, recipe: Dictionary, keep_children: bool) -> void:
	var stand_in := Node3D.new()
	stand_in.transform = n.transform
	stand_in.name = "node_%s_%d" % [_remember(_nodes, recipe), _counter]
	var parent := n.get_parent()
	var at := n.get_index()
	if keep_children:
		for c in n.get_children():
			n.remove_child(c)
			stand_in.add_child(c)
	parent.remove_child(n)
	parent.add_child(stand_in)
	parent.move_child(stand_in, at)
	n.free()


## Every surface's active material (override > surface override > the mesh's) becomes the surface
## material of a mesh this instance owns, named by its recipe.
func _bake_materials(mi: MeshInstance3D, used: Dictionary) -> void:
	var mesh := mi.mesh
	if mesh == null:
		return
	var named := []
	for i in mesh.get_surface_count():
		var m := mi.get_active_material(i)
		if m == null:
			named.append(null)
			continue
		var hash := _remember(_materials, _recipe(m, used))
		if not _material_of.has(hash):
			var copy := m.duplicate() as Material
			copy.resource_name = "m_%s" % hash
			_material_of[hash] = copy
		named.append(_material_of[hash])
	# one mesh per (geometry, materials), so a prop drawn twice is one mesh twice
	var key := "%d|%s" % [mesh.get_instance_id(), str(named.map(func(m): return m.resource_name if m else ""))]
	if not _mesh_of.has(key):
		var out := ArrayMesh.new()
		for i in mesh.get_surface_count():
			# a PrimitiveMesh (a box, a torus) is triangles; an ArrayMesh says
			var kind := (mesh as ArrayMesh).surface_get_primitive_type(i) if mesh is ArrayMesh \
				else Mesh.PRIMITIVE_TRIANGLES
			out.add_surface_from_arrays(kind, mesh.surface_get_arrays(i))
			if named[i] != null:
				out.surface_set_material(i, named[i])
		_mesh_of[key] = out
	mi.mesh = _mesh_of[key]
	mi.material_override = null
	for i in mi.get_surface_override_material_count():
		mi.set_surface_override_material(i, null)


func _remember(table: Dictionary, recipe: Dictionary) -> String:
	var hash := _json(recipe).md5_text().substr(0, 12)
	table[hash] = recipe
	return hash


## An object's recipe: its class and every stored property that differs from a fresh instance.
func _recipe(o: Object, used: Dictionary) -> Dictionary:
	var fresh: Object = ClassDB.instantiate(o.get_class())
	var set := {}
	for p in o.get_property_list():
		var key: String = p["name"]
		if not (int(p["usage"]) & PROPERTY_USAGE_STORAGE) or key in SKIP:
			continue
		var v: Variant = o.get(key)
		var d: Variant = fresh.get(key) if fresh != null else null
		if typeof(v) == typeof(d) and v == d:
			continue
		if v == null and d == null:
			continue
		set[key] = _value(v, used)
	if fresh != null and not (fresh is RefCounted):
		fresh.free()
	return {"class": o.get_class(), "set": set}


func _value(v: Variant, used: Dictionary) -> Variant:
	match typeof(v):
		TYPE_NIL, TYPE_BOOL, TYPE_INT, TYPE_FLOAT, TYPE_STRING:
			return v
		TYPE_STRING_NAME, TYPE_NODE_PATH:
			return str(v)
		TYPE_COLOR:
			return {"Color": [v.r, v.g, v.b, v.a]}
		TYPE_VECTOR2:
			return {"Vector2": [v.x, v.y]}
		TYPE_VECTOR3:
			return {"Vector3": [v.x, v.y, v.z]}
		TYPE_PACKED_STRING_ARRAY:
			return Array(v)
		TYPE_ARRAY:
			var out := []
			for x in v:
				out.append(_value(x, used))
			return out
		TYPE_OBJECT:
			return _resource(v as Resource, used)
	# anything else, losslessly in Godot's own text form (str_to_var reads it back)
	return {"variant": var_to_str(v)}


## A resource: a library file by its path; a texture made at build time as a PNG by content; any
## other resource inline, as its recipe.
func _resource(r: Resource, used: Dictionary) -> Variant:
	if r == null:
		return null
	var path := r.resource_path
	if path != "" and not path.contains("::"):
		used[path] = true
		return {"library": path.trim_prefix("res://")}
	if r is Texture2D:
		var png := (r as Texture2D).get_image().save_png_to_buffer()
		var file := "%s.png" % png.hex_encode().md5_text()
		_textures[file] = png
		return {"file": TEXTURES_DIR.path_join(file)}
	return _recipe(r, used)


# --- licences, the reload check, the files -----------------------------------------------------------

## The licence of every library file the piece references, by the source's table of library roots
## ({root, spdx, what, url}). A file under no root is refused: an asset with no licence is not written.
func _sources(used: Dictionary, licences: Array) -> Variant:
	var by_root := {}
	var files := used.keys()
	files.sort()
	for file in files:
		var hit: Variant = null
		for l in licences:
			if (file as String).begins_with(l["root"]):
				hit = l
		if hit == null:
			return "no licence covers %s (add its library root to the source's licences())" % file
		var entry: Dictionary = by_root.get(hit["root"], {"what": hit["what"], "spdx": hit["spdx"],
			"url": hit["url"], "files": []})
		entry["files"].append((file as String).trim_prefix("res://"))
		by_root[hit["root"]] = entry
	var out := []
	var roots := by_root.keys()
	roots.sort()
	for r in roots:
		out.append(by_root[r])
	return out


## Reads the written file back and checks that it holds what was exported: a mesh, and every entrance
## and space by name.
func _reload(glb: String, entrances: Array, spaces: Array) -> String:
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if doc.append_from_file(glb, state) != OK:
		return "the written %s does not read back" % glb
	var scene := doc.generate_scene(state)
	if scene == null:
		return "the written %s makes no scene" % glb
	var meshes := scene.find_children("*", "MeshInstance3D", true, false).size()
	var missing := []
	for e in entrances + spaces:
		if scene.find_child(e["name"], true, false) == null:
			missing.append(e["name"])
	scene.free()
	if meshes == 0:
		return "the written %s holds no mesh" % glb
	if not missing.is_empty():
		return "the written %s lacks the named nodes %s" % [glb, ", ".join(PackedStringArray(missing))]
	return ""


func _asset_yaml(piece: Dictionary, entrances: Array, spaces: Array, sources: Array) -> Dictionary:
	var id: String = piece["id"]
	var names := []
	for e in entrances:
		names.append(e["name"])
	var rooms := []
	for s in spaces:
		rooms.append(s["name"])
	return {
		"id": id,
		"type": "place_visual",
		"format": {"model": "%s.glb" % id, "rules": RULES_FILE},
		"semantic_bindings": {"entrances": names, "spaces": rooms},
		"doors": entrances,
		"presentation": {"style_family": [_source.style_family()]},
		"scale": {"unit": "meter"},
		"license": {"spdx": _source.licence_spdx(), "sources": sources},
		"source": {"tool": "tools/presentation/export_place_art.gd", "authoring": _source.authoring()},
	}


## The rules file: the recipes already in it (another source's assets share the file; a recipe is named
## by its content, so keeping it is always safe) and this run's, one recipe per line, sorted by name.
func _rules_text() -> String:
	var materials := {}
	var nodes := {}
	var existing := _out.path_join(RULES_FILE)
	if FileAccess.file_exists(existing):
		var old: Variant = JSON.parse_string(FileAccess.get_file_as_string(existing))
		if old is Dictionary:
			materials.merge(old.get("materials", {}))
			nodes.merge(old.get("nodes", {}))
	for k in _materials:
		materials["m_" + k] = _materials[k]
	for k in _nodes:
		nodes[k] = _nodes[k]
	return "{\n  \"format\": 1,\n  \"source\": \"tools/presentation/export_place_art.gd\",\n" \
		+ "  \"materials\": %s,\n  \"nodes\": %s\n}\n" % [_table(materials), _table(nodes)]


func _table(table: Dictionary) -> String:
	var keys := table.keys()
	keys.sort()
	var lines := PackedStringArray()
	for k in keys:
		lines.append("    %s: %s" % [JSON.stringify(k), JSON.stringify(table[k], "", false)])
	return "{\n%s\n  }" % ",\n".join(lines)


## YAML's JSON subset (DEP-16), keys in insertion order, a newline at the end.
static func _json(data: Variant) -> String:
	return JSON.stringify(data, "  ", false) + "\n"


func _write_text(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _write_bytes(path: String, bytes: PackedByteArray) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_buffer(bytes)
	f.close()

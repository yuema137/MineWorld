## The `models` section of a depiction file (16f-d2; `DECISIONS.md` ARC-177 as amended by 16f-d1): place
## art the pack carries as glTF assets (`assets/places/<asset>/<asset>.glb` with its `asset.yaml`,
## `PACKAGE_FORMAT.md` §3), loaded at run time with Godot's `GLTFDocument` and placed by each model's
## anchor. Nothing here names a world, a place, a tag or an asset.
##
## RE-BINDING BY NAME. glTF carries meshes, lights and empty nodes; what it cannot carry travels in the
## pack's `materials/material_rules.yaml`, written by `tools/presentation/export_place_art.gd` (finding
## F-d1-3). Every glTF material is named `m_<hash>` and every node `<kind>_<hash>_<n>`; the rules file
## holds the RECIPE under that name -- the Godot class and every stored property that differs from a
## fresh instance. This builder reverses the export:
##
##   mesh_<h>_<n>   the glTF mesh, every surface's material replaced by the recipe its name gives, and
##                  the instance's own settings (shadow casting, the frame-cost category) applied
##   light_<h>_<n>  the glTF light (KHR_lights_punctual), its recipe applied over it
##   node_<h>_<n>   an empty stand-in, replaced by a node of the recipe's class (a Label3D, a body, a
##                  collision shape, a probe), its children kept; with a `library` key, an instance of
##                  that library scene of the 3D client (an imported Poly Haven prop, with its LODs and
##                  shadow meshes), each surface drawn with the material its recipe names
##   g_<n>          a plain group; entrances and spaces (the asset's `semantic_bindings`) stay as named
##
## A library file is the 3D client's (`res://<path>`); a texture written at export time is the pack's
## (`materials/textures/<md5>.png`). A recorded mipmapped texture filter is replaced by the client's one
## filter (`Mats.FILTER`, ARC-291), so `--aniso=off` holds for pack art too.
##
## THE ANCHOR. `at_mm` (+ `yaw_deg`) puts the asset's (0, 0, 0) on that plan point of the file's frame;
## `entrance` with `at_doorway: true` translates the asset, in plan only, so that its named entrance stands
## on the doorway of the place the model `depicts` (`ARC-45`: frames are glued by translation).
class_name DepictionModels
extends RefCounted

## The pack's place assets and its rules file (`PACKAGE_FORMAT.md` §3; `MODULE_SPEC.md` §6.1).
const ASSETS_DIR := "assets/places"
const RULES_FILE := "materials/material_rules.yaml"

## recipe name -> recipe (the rules file, read once per run).
static var _materials_table := {}
static var _nodes_table := {}
static var _rules_from := ""
## recipe name -> the one Material every surface with it shares (one material, one batch).
static var _material_cache := {}
## pack texture file -> its ImageTexture.
static var _texture_cache := {}

## What the last `build` loaded, for the build log.
var loaded := 0
var problems: PackedStringArray = []
## model index -> { asset: String, root: Node3D, entrances: {name: Node3D}, doors: [asset.yaml rows],
## depicts: PackedStringArray }.
var placed := []


## Loads and places every model of `models` under `g`. `depiction` converts plan millimetres and finds a
## depicted place's doorway (`doorway_of(tags)`). Returns the placed models (see `placed`).
func build(g: Node3D, models: Array, depiction: SliceDataDepiction) -> Array:
	var pack := SlicePack.select()
	_read_rules(pack.path(RULES_FILE))
	for i in models.size():
		var m: Dictionary = models[i]
		var asset: String = m["asset"]
		var dir := pack.path(ASSETS_DIR).path_join(asset)
		var meta: Variant = JSON.parse_string(_text(dir.path_join("asset.yaml")))
		if not (meta is Dictionary):
			problems.append("%s: no readable asset.yaml" % asset)
			continue
		var scene := _load_glb(dir.path_join(str(meta["format"]["model"])))
		if scene == null:
			problems.append("%s: its glTF does not load" % asset)
			continue
		scene.name = "Model_%s" % asset
		g.add_child(scene)
		_rebind(scene, pack)
		var entrances := {}
		for e in meta.get("semantic_bindings", {}).get("entrances", []):
			var n := scene.find_child(str(e), true, false) as Node3D
			if n != null:
				entrances[str(e)] = n
		var depicts := PackedStringArray(m["depicts"]) if m.get("depicts") != null else PackedStringArray()
		_anchor(scene, m["anchor"], entrances, depicts, depiction)
		placed.append({"asset": asset, "root": scene, "entrances": entrances,
			"doors": meta.get("doors", []), "depicts": depicts})
		loaded += 1
	for p in problems:
		push_error("models: " + p)
	return placed


## The model placed for a place with `tags` (every tag the model `depicts` among them), or {}.
func for_place(tags: PackedStringArray) -> Dictionary:
	for p in placed:
		var d: PackedStringArray = p["depicts"]
		if d.is_empty():
			continue
		var all := true
		for t in d:
			all = all and t in tags
		if all:
			return p
	return {}


# --- loading -----------------------------------------------------------------------------------------

static func _load_glb(path: String) -> Node3D:
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if doc.append_from_file(path, state) != OK:
		return null
	return doc.generate_scene(state) as Node3D


static func _text(path: String) -> String:
	return FileAccess.get_file_as_string(path) if FileAccess.file_exists(path) else ""


static func _read_rules(path: String) -> void:
	if _rules_from == path:
		return
	_rules_from = path
	_material_cache.clear()
	var parsed: Variant = JSON.parse_string(_text(path))
	if not (parsed is Dictionary):
		push_error("models: cannot read the rules file %s" % path)
		_materials_table = {}
		_nodes_table = {}
		return
	_materials_table = parsed.get("materials", {})
	_nodes_table = parsed.get("nodes", {})


## The anchor (see the header).
func _anchor(scene: Node3D, anchor: Dictionary, entrances: Dictionary, depicts: PackedStringArray,
		depiction: SliceDataDepiction) -> void:
	var origin := depiction.frame_origin()
	if anchor.get("at_doorway", false):
		var e: Node3D = entrances.get(str(anchor["entrance"]))
		var doorway: Variant = depiction.doorway_of(depicts)
		if e == null or doorway == null:
			problems.append("%s: no entrance %s, or no place depicting %s" % [scene.name, anchor["entrance"],
				depicts])
			scene.position = origin
			return
		# the entrance in the asset's own frame (it may sit under a group node), the asset unplaced
		var rel := scene.global_transform.affine_inverse() * e.global_position
		var at := depiction.to_scene(doorway as Vector2)
		scene.position = Vector3(at.x - rel.x, origin.y, at.z - rel.z)
		return
	var p: Array = anchor["at_mm"]
	scene.position = depiction.to_scene(Vector2(p[0], p[1]))
	scene.position.y = origin.y
	scene.rotation.y = deg_to_rad(float(anchor.get("yaw_deg", 0.0)))


# --- re-binding --------------------------------------------------------------------------------------

## Re-binds every node under `n` (children first, so a replaced node keeps re-bound children).
func _rebind(n: Node, pack: SlicePack) -> void:
	for c in n.get_children():
		_rebind(c, pack)
	var name := String(n.name)
	var parts := name.split("_")
	if parts.size() < 3:
		return
	var recipe: Variant = _nodes_table.get(parts[1])
	if parts[0] == "mesh" and n is MeshInstance3D:
		_surfaces(n as MeshInstance3D, pack)
		if recipe is Dictionary:
			_apply(n, recipe, pack)
	elif parts[0] == "light" and n is Light3D and recipe is Dictionary:
		_apply(n, recipe, pack)
	elif parts[0] == "node" and recipe is Dictionary:
		_replace(n as Node3D, recipe, pack)


## Every surface of the mesh drawn with the material its glTF name gives.
func _surfaces(mi: MeshInstance3D, pack: SlicePack) -> void:
	var mesh := mi.mesh
	if mesh == null:
		return
	for i in mesh.get_surface_count():
		var m := mesh.surface_get_material(i)
		if m == null:
			continue
		var made := _material(m.resource_name, pack)
		if made != null:
			mesh.surface_set_material(i, made)


## The shared material named `name` (`m_<hash>`), built from its recipe once.
func _material(name: String, pack: SlicePack) -> Material:
	if _material_cache.has(name):
		return _material_cache[name]
	var recipe: Variant = _materials_table.get(name)
	if not (recipe is Dictionary):
		problems.append("no material recipe %s" % name)
		_material_cache[name] = null
		return null
	var m := _make(recipe, pack) as Material
	if m != null:
		m.resource_name = name
	_material_cache[name] = m
	return m


## The stand-in `n` replaced by the recipe's node, at its place in the tree, children kept.
func _replace(n: Node3D, recipe: Dictionary, pack: SlicePack) -> void:
	var made: Node3D = null
	if recipe.has("library"):
		made = _library(recipe, pack)
	else:
		made = ClassDB.instantiate(str(recipe["class"])) as Node3D
		if made != null:
			_apply(made, recipe, pack)
	if made == null:
		problems.append("cannot make the node %s (%s)" % [n.name, recipe.get("class")])
		return
	made.name = n.name
	made.transform = n.transform
	for c in n.get_children():
		n.remove_child(c)
		made.add_child(c)
	var parent := n.get_parent()
	var at := n.get_index()
	parent.remove_child(n)
	parent.add_child(made)
	parent.move_child(made, at)
	n.free()


## A library scene of the 3D client, each surface drawn with the material its recipe names.
func _library(recipe: Dictionary, pack: SlicePack) -> Node3D:
	var path := "res://" + str(recipe["library"])
	var made: Node3D = null
	if path.begins_with(Props.MODEL_DIR + "/"):
		made = Props.gltf(path.get_file().get_basename())
	elif ResourceLoader.exists(path, "PackedScene"):
		made = (load(path) as PackedScene).instantiate() as Node3D
	if made == null:
		return null
	_apply(made, recipe, pack)
	# Exactly the meshes the recipe lists: a library file may hold a variant set, of which the art drew
	# only some (`SliceProps.keep`); the export recorded every surface it drew, so a mesh it names no
	# surface of was not drawn.
	var drawn := {}
	for s in recipe.get("surfaces", []):
		drawn[str(s[0])] = true
	for mi in made.find_children("*", "MeshInstance3D", true, false):
		if not drawn.has(str(made.get_path_to(mi))):
			mi.get_parent().remove_child(mi)
			mi.free()
	for s in recipe.get("surfaces", []):
		var mi := made.get_node_or_null(NodePath(str(s[0]))) as MeshInstance3D
		if mi == null or str(s[2]) == "":
			continue
		var mat := _material(str(s[2]), pack)
		if mat != null and int(s[1]) < mi.get_surface_override_material_count():
			mi.set_surface_override_material(int(s[1]), mat)
	return made


## An object of the recipe's class with its properties set.
func _make(recipe: Dictionary, pack: SlicePack) -> Object:
	var o: Object = ClassDB.instantiate(str(recipe["class"]))
	if o != null:
		_apply(o, recipe, pack)
	return o


## Sets every recorded property on `o`, decoded and converted to the property's own type.
func _apply(o: Object, recipe: Dictionary, pack: SlicePack) -> void:
	var set: Dictionary = recipe.get("set", {})
	for key in set:
		var v: Variant = _decode(set[key], pack)
		if key == "texture_filter" and int(v) in [BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS,
				BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC]:
			v = Mats.FILTER
		var now: Variant = o.get(key)
		if now != null and typeof(now) != typeof(v) and typeof(now) != TYPE_OBJECT:
			v = type_convert(v, typeof(now))
		o.set(key, v)


## A recorded value (the export's `_value`), back to a Godot value.
func _decode(v: Variant, pack: SlicePack) -> Variant:
	if v is Array:
		var out := []
		for x in v:
			out.append(_decode(x, pack))
		return out
	if not (v is Dictionary):
		return v
	var d: Dictionary = v
	if d.has("Color"):
		var c: Array = d["Color"]
		return Color(c[0], c[1], c[2], c[3])
	if d.has("Vector3"):
		var p: Array = d["Vector3"]
		return Vector3(p[0], p[1], p[2])
	if d.has("Vector2"):
		var q: Array = d["Vector2"]
		return Vector2(q[0], q[1])
	if d.has("variant"):
		return str_to_var(str(d["variant"]))
	if d.has("library"):
		return load("res://" + str(d["library"]))
	if d.has("file"):
		return _texture(pack.path(str(d["file"])))
	if d.has("class"):
		return _make(d, pack)
	return d


## A pack texture, with mipmaps, as every texture the client draws has them (the importer's, or
## `ProcGen`'s for a card made at run time): an alpha-tested card without them keeps its full alpha at a
## distance, and a canopy's shadow turns solid (16f-d2 parity finding F-d2-3).
static func _texture(path: String) -> Texture2D:
	if not _texture_cache.has(path):
		var img := Image.load_from_file(path)
		if img != null and not img.has_mipmaps():
			img.generate_mipmaps()
		_texture_cache[path] = ImageTexture.create_from_image(img) if img != null else null
	return _texture_cache[path]

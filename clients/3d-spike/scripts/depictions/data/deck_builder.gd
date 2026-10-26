## The decks of a depiction file's `decks` section (`DECISIONS.md` ARC-178; S33-e sec.5.1.1): a walked floor
## raised on piles -- a pier, a jetty, a boardwalk, a platform -- with its approach ramp, rails, curb, open
## shelter, lantern and moored craft, all from the file.
##
##   boards     one merged mesh of boards `width_mm` wide with `gap_mm` gaps, laid `across` x or y, over a
##              dark stringer slab `thickness_m` deep; on an approach each board sits at the ramp's height
##   approach   a ramp inside the footprint from its `side` edge at `from_m` to `surface_m` over `length_m`
##   piles      `bents` pairs of round piles from `to_m` to the stringers, a cap beam each, X-bracing
##   rails      round rails `height_m` above the surface (above the ramp along an approach), posts on a curb
##   shelter    posts, an eaves frame, the building roof (`DepictionShells._roof`), a lantern on one post
##   moored     static rowing boats (ARC-171), each tied by a line to its nearest pile
##
## The surface and the approach collide as ground (the group `SliceProbeGeometry.GROUND_GROUP`: where a body
## stands, never a wall); the rails and the shelter's posts collide as drawn; the server's walls still decide
## who passes. Every deck is tagged `mw_category` = `landmark.<name>` when the place it depicts is tagged
## `landmark` (its craft `landmark.<name>.boat`), else `deck.<name>`, so the landmark check can hide it.
## Nothing here names a world, a place or a tag other than the format's word `landmark`; nothing animates.
class_name DepictionDecks
extends RefCounted

## The format's tag that makes a deck a landmark, and the categories (`mw_category`) it takes.
const LANDMARK_TAG := "landmark"
## A texture's repeat on boards and timber, metres (`PRESENTATION.md` sec.6's schema default).
const TEXTURE_M := 2.0
## The stringers' share of the deck's thickness below the boards; the boards' own thickness (38 mm, a
## nominal 2 in board, NIST PS 20) is the rest's top.
const BOARD_T := 0.038
## A cap beam's depth and the X-bracing's reach below it, metres; a mooring line's thickness.
const CAP_M := 0.2
const BRACE_DROP_M := 1.4
const BRACE_W := 0.08
const LINE_R := 0.012
## The eaves frame's depth under the eaves, metres.
const PLATE_M := 0.15


## Draws every deck of `decks` under `parent`, in `d`'s frame.
static func build(parent: Node3D, decks: Array, d: SliceDataDepiction) -> void:
	var g := Node3D.new()
	g.name = "Decks"
	parent.add_child(g)
	for deck in decks:
		_deck(g, deck, d)


## The deck's footprint (plan mm).
static func rect_of(deck: Dictionary) -> Rect2:
	var r: Array = deck["footprint_mm"]
	return Rect2(float(r[0]), float(r[1]), float(r[2]), float(r[3]))


## The walking surface's height (metres above the street) at plan point `p` (mm) on `deck`: the approach's
## ramp within its run of its edge, the surface elsewhere.
static func surface_at(deck: Dictionary, p: Vector2) -> float:
	var surface := float(deck["surface_m"])
	var a: Variant = deck.get("approach")
	if a == null:
		return surface
	var run := float(a["length_m"]) * 1000.0
	var t := _from_edge(rect_of(deck), String(a["side"]), p)
	if t >= run:
		return surface
	return lerpf(float(a["from_m"]), surface, clampf(t / run, 0.0, 1.0))


## How far (mm) plan point `p` lies inside `rect` from its `side` edge.
static func _from_edge(rect: Rect2, side: String, p: Vector2) -> float:
	match side:
		"n": return rect.end.y - p.y
		"s": return p.y - rect.position.y
		"e": return rect.end.x - p.x
		_: return p.x - rect.position.x


## Whether `deck` depicts a place the file tags `landmark`.
static func is_landmark(deck: Dictionary, d: SliceDataDepiction) -> bool:
	for p in d.places():
		var all := true
		for t in deck["depicts"]:
			if not (t in p["tags"]):
				all = false
		if all:
			return LANDMARK_TAG in p["tags"]
	return false


static func _deck(g: Node3D, deck: Dictionary, d: SliceDataDepiction) -> void:
	var node := Node3D.new()
	node.name = String(deck["name"])
	var category := "%s.%s" % [LANDMARK_TAG if is_landmark(deck, d) else "deck", deck["name"]]
	node.set_meta("mw_category", category)
	g.add_child(node)
	var timber := Mats.pbr(deck["piles"]["material"], TEXTURE_M)
	_boards(node, deck, d)
	_ground(node, deck, d)
	var piles := _piles(node, deck, d, timber)
	if deck.get("rails") != null:
		_rails(node, deck, d)
	if deck.get("shelter") != null:
		_shelter(node, deck, d, timber)
	if not (deck.get("moored", []) as Array).is_empty():
		var boats := Node3D.new()
		boats.name = "Moored"
		boats.set_meta("mw_category", category + ".boat")
		g.add_child(boats)
		for m in deck["moored"]:
			_boat(boats, m, d, piles)


# --- boards and ground ---------------------------------------------------------------------------------

## A plan point (mm) at height `h` (metres above the street) in the scene.
static func _at(d: SliceDataDepiction, p: Vector2, h: float) -> Vector3:
	return d.to_scene(p) + Vector3(0.0, h, 0.0)


static func _boards(node: Node3D, deck: Dictionary, d: SliceDataDepiction) -> void:
	var r := rect_of(deck)
	var b: Dictionary = deck["boards"]
	var tint := Color.html(b["tint"]) if b.get("tint") != null else Color.WHITE
	var along_x: bool = b["across"] == "x"
	var pitch := float(b["width_mm"]) + float(b["gap_mm"])
	var span := r.size.y if along_x else r.size.x
	var count := maxi(int(floor(span / pitch)), 1)
	var width := float(b["width_mm"]) / 1000.0
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for k in count:
		var c := (k + 0.5) * span / count
		var centre := Vector2(r.get_center().x, r.position.y + c) if along_x \
			else Vector2(r.position.x + c, r.get_center().y)
		var top := surface_at(deck, centre)
		var a := _at(d, centre, top - BOARD_T * 0.5)
		var size := Vector3(r.size.x / 1000.0, BOARD_T, width) if along_x \
			else Vector3(width, BOARD_T, r.size.y / 1000.0)
		_box_into(st, a, size)
	st.generate_normals()
	st.generate_tangents()
	var mi := MeshInstance3D.new()
	mi.name = "Boards"
	mi.mesh = st.commit()
	mi.material_override = Mats.pbr(b["material"], TEXTURE_M, tint)
	node.add_child(mi)
	# the stringers under the boards, dark in the gaps
	var under := Mats.paint(Color(0.16, 0.13, 0.10), 0.95)
	for part in _parts(deck):
		var lo: float = part[1]
		var hi: float = part[2]
		var rr: Rect2 = part[0]
		var depth := float(deck["thickness_m"]) - BOARD_T
		_slab(node, d, rr, lo - BOARD_T, hi - BOARD_T, depth, under, String(part[3]))


## The deck's two parts: [rect, height at its approach-side end, height at its far end, side], the
## approach (when there is one) and the level rest.
static func _parts(deck: Dictionary) -> Array:
	var r := rect_of(deck)
	var surface := float(deck["surface_m"])
	var a: Variant = deck.get("approach")
	if a == null:
		return [[r, surface, surface, "n"]]
	var run := float(a["length_m"]) * 1000.0
	var side := String(a["side"])
	var ramp := r
	var rest := r
	match side:
		"n":
			ramp = Rect2(r.position.x, r.end.y - run, r.size.x, run)
			rest = Rect2(r.position.x, r.position.y, r.size.x, r.size.y - run)
		"s":
			ramp = Rect2(r.position.x, r.position.y, r.size.x, run)
			rest = Rect2(r.position.x, r.position.y + run, r.size.x, r.size.y - run)
		"e":
			ramp = Rect2(r.end.x - run, r.position.y, run, r.size.y)
			rest = Rect2(r.position.x, r.position.y, r.size.x - run, r.size.y)
		_:
			ramp = Rect2(r.position.x, r.position.y, run, r.size.y)
			rest = Rect2(r.position.x + run, r.position.y, r.size.x - run, r.size.y)
	var out := [[ramp, float(a["from_m"]), surface, side]]
	if rest.size.x > 0.0 and rest.size.y > 0.0:
		out.append([rest, surface, surface, side])
	return out


## A slab over plan rect `rr` whose top runs from `top_from` at its `side` edge to `top_to` at the far edge,
## `depth` deep; returns the basis and centre it used (for its collider).
static func _slab(node: Node3D, d: SliceDataDepiction, rr: Rect2, top_from: float, top_to: float, depth: float,
		mat: Material, side: String) -> Transform3D:
	var xf := _slab_xf(d, rr, top_from, top_to, depth, side)
	var bm := BoxMesh.new()
	bm.size = _slab_size(rr, top_from, top_to, depth, side)
	var mi := MeshInstance3D.new()
	mi.name = "Stringers"
	mi.mesh = bm
	mi.material_override = mat
	mi.transform = xf
	node.add_child(mi)
	return xf


static func _slab_size(rr: Rect2, top_from: float, top_to: float, depth: float, side: String) -> Vector3:
	var along_y := side == "n" or side == "s"
	var run := (rr.size.y if along_y else rr.size.x) / 1000.0
	var length := sqrt(run * run + (top_to - top_from) * (top_to - top_from))
	return Vector3(rr.size.x / 1000.0, depth, length) if along_y else Vector3(length, depth, rr.size.y / 1000.0)


## The slab's transform: centred under the middle of its top, tilted from its `side` edge's height to the
## far edge's.
static func _slab_xf(d: SliceDataDepiction, rr: Rect2, top_from: float, top_to: float, depth: float,
		side: String) -> Transform3D:
	var near := rr.get_center()
	var far := rr.get_center()
	match side:
		"n": near.y = rr.end.y; far.y = rr.position.y
		"s": near.y = rr.position.y; far.y = rr.end.y
		"e": near.x = rr.end.x; far.x = rr.position.x
		_: near.x = rr.position.x; far.x = rr.end.x
	var a := _at(d, near, top_from)
	var b := _at(d, far, top_to)
	var along := (b - a).normalized()
	var up := Vector3.UP - along * along.dot(Vector3.UP)
	up = up.normalized()
	var basis: Basis
	if side == "n" or side == "s":
		basis = Basis(up.cross(along).normalized(), up, along)
	else:
		basis = Basis(along, up, along.cross(up).normalized())
	return Transform3D(basis, (a + b) * 0.5 - up * depth * 0.5)


## The deck's walking surface as ground colliders: one box per part, its top the surface.
static func _ground(node: Node3D, deck: Dictionary, d: SliceDataDepiction) -> void:
	var depth := float(deck["thickness_m"])
	for part in _parts(deck):
		var rr: Rect2 = part[0]
		var body := StaticBody3D.new()
		body.name = "DeckGround"
		body.collision_layer = Build.LAYER_WORLD
		body.add_to_group(SliceProbeGeometry.GROUND_GROUP)
		var cs := CollisionShape3D.new()
		var sh := BoxShape3D.new()
		sh.size = _slab_size(rr, part[1], part[2], depth, String(part[3]))
		cs.shape = sh
		body.add_child(cs)
		body.transform = _slab_xf(d, rr, part[1], part[2], depth, String(part[3]))
		node.add_child(body)


## Adds an axis-aligned box (centre, size) to `st` as six quads.
static func _box_into(st: SurfaceTool, c: Vector3, s: Vector3) -> void:
	var h := s * 0.5
	var faces := [
		[Vector3(-1, 1, -1), Vector3(1, 1, -1), Vector3(1, 1, 1), Vector3(-1, 1, 1)],
		[Vector3(-1, -1, 1), Vector3(1, -1, 1), Vector3(1, -1, -1), Vector3(-1, -1, -1)],
		[Vector3(-1, -1, 1), Vector3(-1, 1, 1), Vector3(1, 1, 1), Vector3(1, -1, 1)],
		[Vector3(1, -1, -1), Vector3(1, 1, -1), Vector3(-1, 1, -1), Vector3(-1, -1, -1)],
		[Vector3(1, -1, 1), Vector3(1, 1, 1), Vector3(1, 1, -1), Vector3(1, -1, -1)],
		[Vector3(-1, -1, -1), Vector3(-1, 1, -1), Vector3(-1, 1, 1), Vector3(-1, -1, 1)],
	]
	for f in faces:
		var q: Array = []
		for v in f:
			q.append(c + (v as Vector3) * h)
		for i in [0, 1, 2, 0, 2, 3]:
			st.set_uv(Vector2((q[i] as Vector3).x + (q[i] as Vector3).y, (q[i] as Vector3).z + (q[i] as Vector3).y) / TEXTURE_M)
			st.add_vertex(q[i])


# --- piles ---------------------------------------------------------------------------------------------

## The piles, their caps and bracing; returns the piles' scene points at the water's surface or the
## deck's underside, for the mooring lines.
static func _piles(node: Node3D, deck: Dictionary, d: SliceDataDepiction, timber: Material) -> Array:
	var r := rect_of(deck)
	var p: Dictionary = deck["piles"]
	var inset := float(p["inset_m"]) * 1000.0
	var radius := float(p["diameter_m"]) * 0.5
	var bents := int(p["bents"])
	var along_y := r.size.y >= r.size.x
	var a: Variant = deck.get("approach")
	if a != null:
		along_y = String(a["side"]) in ["n", "s"]
	var out := []
	var to := float(p["to_m"])
	var thickness := float(deck["thickness_m"])
	var line_y := minf(d.water_level() + 0.3, float(deck["surface_m"]) - thickness)
	for k in bents:
		var t := float(k) / float(maxi(bents - 1, 1))
		var pair := []
		for s in [0, 1]:
			var at := Vector2()
			if along_y:
				at = Vector2(r.position.x + inset if s == 0 else r.end.x - inset,
					lerpf(r.position.y + inset, r.end.y - inset, t))
			else:
				at = Vector2(lerpf(r.position.x + inset, r.end.x - inset, t),
					r.position.y + inset if s == 0 else r.end.y - inset)
			var under := surface_at(deck, at) - thickness
			Build.cyl(node, _at(d, at, to), radius, radius, under - to, timber, 10)
			pair.append([at, under])
			out.append(_at(d, at, line_y))
		# the cap beam across the pair, under the stringers
		var c0: Vector2 = pair[0][0]
		var c1: Vector2 = pair[1][0]
		var top := minf(float(pair[0][1]), float(pair[1][1]))
		_bar(node, _at(d, c0, top - CAP_M * 0.5), _at(d, c1, top - CAP_M * 0.5), radius * 2.0, CAP_M, timber,
			radius * 2.0)
		if bool(p["bracing"]):
			var low := maxf(top - BRACE_DROP_M, to)
			_bar(node, _at(d, c0, top - CAP_M), _at(d, c1, low), BRACE_W, BRACE_W, timber)
			_bar(node, _at(d, c1, top - CAP_M), _at(d, c0, low), BRACE_W, BRACE_W, timber)
	return out


## A box from `a` to `b` (scene), `w` wide and `h` high, its length along a-b, `extra` longer at each end.
static func _bar(node: Node3D, a: Vector3, b: Vector3, w: float, h: float, mat: Material, extra := 0.0) -> void:
	var dir := b - a
	var length := dir.length()
	if length <= 0.0:
		return
	var along := dir / length
	var up := Vector3.UP - along * along.dot(Vector3.UP)
	if up.length() < 0.01:
		up = Vector3.BACK
	up = up.normalized()
	var bm := BoxMesh.new()
	bm.size = Vector3(w, h, length + extra * 2.0)
	var mi := MeshInstance3D.new()
	mi.mesh = bm
	mi.material_override = mat
	mi.transform = Transform3D(Basis(up.cross(along).normalized(), up, along), (a + b) * 0.5)
	node.add_child(mi)


## A round rod from `a` to `b` (scene), radius `r`.
static func _rod(node: Node3D, a: Vector3, b: Vector3, r: float, mat: Material) -> void:
	var dir := b - a
	var length := dir.length()
	if length <= 0.0:
		return
	var y := dir / length
	var x := y.cross(Vector3.BACK if absf(y.dot(Vector3.BACK)) < 0.9 else Vector3.RIGHT).normalized()
	var cm := CylinderMesh.new()
	cm.top_radius = r
	cm.bottom_radius = r
	cm.height = length
	cm.radial_segments = 8
	cm.rings = 1
	var mi := MeshInstance3D.new()
	mi.mesh = cm
	mi.material_override = mat
	mi.transform = Transform3D(Basis(x, y, x.cross(y)), (a + b) * 0.5)
	node.add_child(mi)


# --- rails and curb ------------------------------------------------------------------------------------

## Round rails on each listed side (not across the approach's edge), posts every `posts_m` standing on the
## curb, a top and a middle rail, the curb under them; each span collides as one box.
static func _rails(node: Node3D, deck: Dictionary, d: SliceDataDepiction) -> void:
	var r := rect_of(deck)
	var rails: Dictionary = deck["rails"]
	var mat := Mats.pbr(rails["material"], TEXTURE_M)
	var h := float(rails["height_m"])
	var post := float(rails["post_m"])
	var rod := float(rails["rail_m"]) * 0.5
	var curb: Variant = deck.get("curb")
	var curb_h := float(curb["height_m"]) if curb != null else 0.0
	var curb_w := float(curb["width_m"]) if curb != null else 0.0
	var opening := String(deck["approach"]["side"]) if deck.get("approach") != null else ""
	var inset := maxf(post, curb_w) * 500.0
	for side in rails["sides"]:
		if side == opening:
			continue
		var line := _edge(r, String(side), inset)
		var a: Vector2 = line[0]
		var b: Vector2 = line[1]
		var length := a.distance_to(b) / 1000.0
		var n := maxi(int(ceil(length / float(rails["posts_m"]))), 1)
		var points := []
		for k in n + 1:
			var p := a.lerp(b, float(k) / float(n))
			points.append([p, surface_at(deck, p)])
		for k in points.size():
			var p: Vector2 = points[k][0]
			var base: float = points[k][1]
			Build.box(node, _at(d, p, base + curb_h + (h - curb_h) * 0.5), Vector3(post, h - curb_h, post), mat)
			if k == 0:
				continue
			var q: Vector2 = points[k - 1][0]
			var qb: float = points[k - 1][1]
			for up in [h, h * 0.5]:
				_rod(node, _at(d, q, qb + up), _at(d, p, base + up), rod, mat)
			if curb != null:
				_bar(node, _at(d, q, qb + curb_h * 0.5), _at(d, p, base + curb_h * 0.5), curb_w, curb_h, mat)
			var lo := minf(base, qb)
			var hi := maxf(base, qb) + h
			var mid := (_at(d, p, lo) + _at(d, q, lo)) * 0.5
			var run := _at(d, p, 0.0) - _at(d, q, 0.0)
			Build.box_blocker(node, Vector3(mid.x, lo + (hi - lo) * 0.5, mid.z),
				Vector3(maxf(post, curb_w), hi - lo, run.length()), atan2(run.x, run.z))


## A side's line (plan mm), `inset` mm inside the footprint, from corner to corner.
static func _edge(r: Rect2, side: String, inset: float) -> Array:
	match side:
		"n": return [Vector2(r.position.x + inset, r.end.y - inset), Vector2(r.end.x - inset, r.end.y - inset)]
		"s": return [Vector2(r.position.x + inset, r.position.y + inset), Vector2(r.end.x - inset, r.position.y + inset)]
		"e": return [Vector2(r.end.x - inset, r.position.y + inset), Vector2(r.end.x - inset, r.end.y - inset)]
		_: return [Vector2(r.position.x + inset, r.position.y + inset), Vector2(r.position.x + inset, r.end.y - inset)]


# --- the shelter ---------------------------------------------------------------------------------------

## The shelter's posts (each colliding), the eaves frame over them, the roof, and the lantern.
static func _shelter(node: Node3D, deck: Dictionary, d: SliceDataDepiction, timber: Material) -> void:
	var s: Dictionary = deck["shelter"]
	var g := Node3D.new()
	g.name = "Shelter"
	node.add_child(g)
	var post := float(s["post_m"])
	var eaves := float(s["eaves_m"])
	var box := Rect2()
	var first := true
	var posts := []
	for p in s["posts_mm"]:
		var at := Vector2(float(p[0]), float(p[1]))
		var base := surface_at(deck, at)
		posts.append([at, base])
		Build.box(g, _at(d, at, base + eaves * 0.5), Vector3(post, eaves, post), timber, 0.0, true)
		var half := Vector2(post, post) * 500.0
		var pr := Rect2(at - half, half * 2.0)
		box = pr if first else box.merge(pr)
		first = false
	var top := float(deck["surface_m"]) + eaves
	# the eaves frame: a plate along each side of the posts' box
	var corners := [box.position, Vector2(box.end.x, box.position.y), box.end, Vector2(box.position.x, box.end.y)]
	for k in 4:
		var a: Vector2 = corners[k]
		var b: Vector2 = corners[(k + 1) % 4]
		_bar(g, _at(d, a, top - PLATE_M * 0.5), _at(d, b, top - PLATE_M * 0.5), post, PLATE_M, timber)
	var footprint := PackedVector2Array()
	for c in corners:
		var sc := d.to_scene(c)
		footprint.append(Vector2(sc.x, sc.z))
	DepictionShells._roof(g, {"roof": s["roof"]}, footprint, top, top, timber, d)
	if s.get("lantern") != null:
		_lantern(g, posts[int(s["lantern"]["post"])], float(s["lantern"]["up_m"]), box, d)


## A lantern hung from a short bracket on a post, toward the shelter's middle; unlit (no emission).
static func _lantern(g: Node3D, post: Array, up: float, box: Rect2, d: SliceDataDepiction) -> void:
	var at: Vector2 = post[0]
	var base: float = post[1]
	var toward := (box.get_center() - at).normalized() * 300.0
	var iron := Mats.paint(Color(0.12, 0.12, 0.12), 0.6, 0.6)
	var glass := Mats.paint(Color(0.86, 0.84, 0.74), 0.2)
	var bracket_y := base + up + 0.2
	_bar(g, _at(d, at, bracket_y), _at(d, at + toward, bracket_y), 0.03, 0.03, iron)
	var lamp := _at(d, at + toward, base + up)
	Build.box(g, lamp, Vector3(0.16, 0.24, 0.16), glass)
	Build.box(g, lamp + Vector3(0.0, 0.14, 0.0), Vector3(0.2, 0.04, 0.2), iron)
	_bar(g, lamp + Vector3(0.0, 0.16, 0.0), _at(d, at + toward, bracket_y), 0.015, 0.015, iron)


# --- moored craft --------------------------------------------------------------------------------------

## A static rowing boat on the water, its hull scaled from the spike's 3.6 m boat, tied to the nearest pile.
static func _boat(g: Node3D, m: Dictionary, d: SliceDataDepiction, piles: Array) -> void:
	var boat := Node3D.new()
	boat.name = "Boat"
	var at := d.to_scene(Vector2(float(m["at_mm"][0]), float(m["at_mm"][1])))
	var level := d.water_level()
	boat.position = Vector3(at.x, level, at.z)
	boat.rotation.y = -deg_to_rad(float(m["yaw_deg"]))
	g.add_child(boat)
	var length := float(m["length_m"])
	var beam := float(m["beam_m"])
	var hull := Mats.paint(Color(0.68, 0.30, 0.24), 0.55)
	var planks := Mats.planks()
	# the hull rides with a little freeboard: 0.22 m of its 0.46 m above the water
	Build.slab(boat, 0, 0, beam, length, -0.24, 0.46, hull)
	Build.slab(boat, 0, 0, beam * 0.78, length * 0.89, 0.04, 0.10, planks)
	for seat in [-0.22, 0.22]:
		Build.slab(boat, 0, length * seat, beam * 0.74, 0.18, 0.04, 0.10, planks)
	if piles.is_empty():
		return
	var bow := boat.position + Basis(Vector3.UP, boat.rotation.y) * Vector3(0.0, 0.2, -length * 0.5)
	var nearest: Vector3 = piles[0]
	for p in piles:
		if (p as Vector3).distance_to(bow) < nearest.distance_to(bow):
			nearest = p
	_rod(g, bow, nearest, LINE_R, Mats.paint(Color(0.55, 0.50, 0.40), 0.9))

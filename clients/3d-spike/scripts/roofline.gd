## What a building does above its shopfront.
##
## The shop row used to end in one flat cornice and one flat roof slab per unit,
## at one of two heights. Against `05_main_street_golden_hour` the silhouette is
## what gives that away before any material does: the plate's roofline steps --
## parapets at different heights, a gable end, chimneys, a bay window throwing a
## shadow across the façade below it -- and a row of equal boxes reads as a test
## scene however well it is lit or textured.
##
## Everything here is deterministic in the unit's seed, so the row is varied but
## does not change between runs, and a screenshot is reproducible.
class_name Roofline
extends RefCounted


## A parapet that is not the same height along its length: a raised centre bay
## with lower shoulders, which is the commonest way a real Victorian shopfront
## breaks its own skyline.
static func parapet(g: Node3D, w: float, depth: float, h: float, wall: Material,
		seed_v: int) -> float:
	var r := RandomNumberGenerator.new()
	r.seed = seed_v * 7919 + 13
	var stone := Mats.cutstone()
	var lift := r.randf_range(0.25, 0.85)
	var centre_w := w * r.randf_range(0.34, 0.62)

	# the cornice band, which every unit gets
	Build.slab(g, 0, depth * 0.5, w + 0.45, depth + 0.45, h - 0.05, 0.42, stone)
	var top := h + 0.37
	# shoulders
	Build.slab(g, 0, 0.16, w + 0.30, 0.34, top, lift * 0.55, wall)
	# the raised centre
	Build.slab(g, 0, 0.16, centre_w, 0.40, top, lift, wall)
	Build.slab(g, 0, 0.16, centre_w + 0.30, 0.48, top + lift, 0.18, stone)
	Build.slab(g, 0, 0.16, w + 0.30, 0.40, top + lift * 0.55, 0.16, stone)
	return top


## A gable end facing the street. The plate has one and it is the single
## strongest break in an otherwise horizontal row.
static func gable(g: Node3D, w: float, depth: float, h: float, wall: Material,
		accent: Color, seed_v: int) -> float:
	var r := RandomNumberGenerator.new()
	r.seed = seed_v * 104729 + 7
	var rise: float = w * r.randf_range(0.26, 0.34)
	Build.slab(g, 0, depth * 0.5, w + 0.45, depth + 0.45, h - 0.05, 0.42, Mats.cutstone())
	var base := h + 0.37
	# the gable wall itself, and a roof plane each side sitting just proud of it
	Build.prism(g, Vector3(0, base + rise * 0.5, 0.45), Vector3(w + 0.2, rise, 0.5), wall)
	var slope := atan2(rise, w * 0.5)
	var run := sqrt(rise * rise + (w * 0.5) * (w * 0.5))
	for sx in [-1.0, 1.0]:
		var pitch := MeshInstance3D.new()
		var bm := BoxMesh.new()
		bm.size = Vector3(run, 0.12, depth * 0.62)
		pitch.mesh = bm
		pitch.material_override = Mats.roof()
		pitch.transform = Transform3D(Basis(Vector3(0, 0, sx), -slope),
			Vector3(sx * w * 0.25, base + rise * 0.5 + 0.06, depth * 0.30))
		g.add_child(pitch)
	# bargeboard along the two rake edges
	for sx in [-1.0, 1.0]:
		var bb := MeshInstance3D.new()
		var bm2 := BoxMesh.new()
		bm2.size = Vector3(run, 0.16, 0.10)
		bb.mesh = bm2
		bb.material_override = Mats.paint(accent, 0.5)
		bb.transform = Transform3D(Basis(Vector3(0, 0, sx), -slope),
			Vector3(sx * w * 0.25, base + rise * 0.5, 0.22))
		g.add_child(bb)
	return base + rise


## A brick stack. Real ones sit on a party wall, so this goes near an edge.
static func chimney(g: Node3D, w: float, depth: float, y: float, seed_v: int) -> void:
	var r := RandomNumberGenerator.new()
	r.seed = seed_v * 31337 + 5
	var side: float = -1.0 if r.randf() < 0.5 else 1.0
	var cw := r.randf_range(0.55, 0.85)
	var ch := r.randf_range(1.1, 2.0)
	var cz := depth * r.randf_range(0.18, 0.45)
	Build.slab(g, side * (w * 0.5 - cw * 0.75), cz, cw, cw * 0.78, y, ch, Mats.brick())
	Build.slab(g, side * (w * 0.5 - cw * 0.75), cz, cw + 0.16, cw * 0.78 + 0.16,
		y + ch, 0.14, Mats.cutstone())
	# pots
	for i in 2:
		Build.cyl(g, Vector3(side * (w * 0.5 - cw * 0.75) + (i - 0.5) * cw * 0.42,
			y + ch + 0.14, cz), 0.075, 0.085, 0.26,
			Mats.paint(Color(0.42, 0.26, 0.20), 0.85), 8)


## A projecting bay on an upper storey. It is the cheapest way to stop a façade
## being a plane, and it throws a shadow that the flat version never had.
static func bay(g: Node3D, w: float, y: float, accent: Color, seed_v: int) -> void:
	var bw: float = min(w * 0.42, 3.0)
	var proj := 0.62
	var bh := 2.35
	var joinery := Mats.paint(accent, 0.45)
	var glass := Mats.glass()
	# the box: front face plus two cheeks, glazed on the front
	Build.slab(g, 0, -proj * 0.5, bw, proj, y - 0.18, 0.20, Mats.cutstone())
	Build.slab(g, 0, -proj + 0.06, bw, 0.12, y, bh, joinery)
	Build.slab(g, 0, -proj + 0.14, bw - 0.30, 0.06, y + 0.14, bh - 0.42, glass)
	for sx in [-1.0, 1.0]:
		Build.slab(g, sx * (bw * 0.5 - 0.06), -proj * 0.5, 0.12, proj, y, bh, joinery)
	Build.slab(g, 0, -proj * 0.5, bw + 0.22, proj + 0.12, y + bh, 0.16, Mats.cutstone())
	Build.slab(g, 0, -proj * 0.5, bw + 0.10, proj + 0.06, y + bh + 0.16, 0.22, Mats.roof())


## A cast-iron downpipe from the cornice to the pavement, with a shoe at the
## bottom. Small, but the references are full of them and a façade without one
## reads as a render rather than as a building.
static func downpipe(g: Node3D, w: float, h: float, seed_v: int) -> void:
	var r := RandomNumberGenerator.new()
	r.seed = seed_v * 6151 + 3
	var side: float = -1.0 if r.randf() < 0.5 else 1.0
	var px := side * (w * 0.5 - 0.22)
	var m := Mats.paint(Color(0.17, 0.18, 0.18), 0.55, 0.25)
	Build.cyl(g, Vector3(px, 0.0, -0.16), 0.055, 0.055, h - 0.2, m, 8)
	Build.cyl(g, Vector3(px, h - 0.2, -0.16), 0.10, 0.075, 0.22, m, 8)
	Build.slab(g, px, -0.16, 0.16, 0.16, 0.0, 0.12, m)

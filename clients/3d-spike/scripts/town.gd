## The scene: one lakeside promenade, one shop row, one side street, one plaza,
## one stretch of shore trail. Compact on purpose -- the point is to judge a
## style at three distances, not to ship a town.
##
## Everything is metres. Y is up. The lake is north (-Z). Shopfronts face it.
##
##   far     lake, far shore, treeline, snow-capped ridges   (z < -60)
##   medium  promenade, plaza, shop row, side street         (z -34 .. +34)
##   near    doors, awnings, planters, benches, boards       (arm's length)
class_name Town
extends RefCounted

const WATER_Y := -1.15
const PROM_N := -26.0   # quay edge
const PROM_S := -12.0   # shopfront line
const STREET_W0 := 13.0 # side street, west edge
const STREET_W1 := 21.0 # side street, east edge

## Poly Haven's bench ships a dark reddish stain from its back-alley set.
## Multiplying past 1.0 pushes it back to the honey wood of the references.
const BENCH_TINT := Color(1.34, 1.24, 1.06)

static var rng := RandomNumberGenerator.new()


static func build(root: Node3D) -> void:
	rng.seed = 20260926
	var t0 := Time.get_ticks_msec()
	_terrain(root)
	_lake(root)
	_mountains(root)
	_far_shore(root)
	_paving(root)
	_shop_row(root)
	_side_street(root)
	_plaza(root)
	_promenade_dressing(root)
	_lake_trail(root)
	_people(root)
	print("town built in %d ms, %d nodes" % [Time.get_ticks_msec() - t0, _count(root)])


static func _count(n: Node) -> int:
	var c := 1
	for ch in n.get_children():
		c += _count(ch)
	return c


# --- ground and water ----------------------------------------------------------

const SHORE_Z := -28.0  # where the land ends and the lake begins

static func _terrain(root: Node3D) -> void:
	# Everything the town is not: grass. It must stop at the shoreline -- a lawn
	# plane running north over the water at y=-0.02 hides a lake at y=-1.15, and
	# from the shore trail the whole lake reads as a field.
	var d := 220.0
	var cz := SHORE_Z + d * 0.5
	Build.ground(root, Vector3(0, -0.02, cz), 260, d, Mats.lawn(), 0, true)
	Build.ground(root, Vector3(-230, -0.02, cz), 200, d, Mats.lawn(), 0, true)
	Build.ground(root, Vector3(230, -0.02, cz), 200, d, Mats.lawn(), 0, true)


static func _lake(root: Node3D) -> void:
	var water := MeshInstance3D.new()
	var pm := PlaneMesh.new()
	pm.size = Vector2(1400, 1200)
	pm.subdivide_width = 1
	pm.subdivide_depth = 1
	water.mesh = pm
	var sh := load("res://shaders/water.gdshader")
	var m := ShaderMaterial.new()
	m.shader = sh
	water.material_override = m
	water.position = Vector3(0, WATER_Y, -560)
	root.add_child(water)

	# quay wall, so the promenade ends in a built edge rather than a cliff
	var stone := Mats.cutstone()
	Build.slab(root, -32, PROM_N + 0.25, 40, 0.5, WATER_Y - 0.6, 1.75, stone)
	Build.slab(root, 32, PROM_N + 0.25, 40, 0.5, WATER_Y - 0.6, 1.75, stone)
	Build.slab(root, 0, -34.25, 26, 0.5, WATER_Y - 0.6, 1.75, stone)
	Build.slab(root, -12.25, -30, 0.5, 8.5, WATER_Y - 0.6, 1.75, stone)
	Build.slab(root, 12.25, -30, 0.5, 8.5, WATER_Y - 0.6, 1.75, stone)


## Ridge line. A noise heightfield beats a row of cones: the references put real
## mountains behind the lake and a silhouette that repeats reads as wallpaper.
static func _mountains(root: Node3D) -> void:
	var n := FastNoiseLite.new()
	n.seed = 7
	n.noise_type = FastNoiseLite.TYPE_SIMPLEX
	n.frequency = 0.0022
	n.fractal_octaves = 5

	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var nx := 96
	var nz := 34
	var x0 := -1100.0
	var x1 := 1100.0
	var z0 := -1000.0
	var z1 := -320.0

	var hs := []
	for j in range(nz + 1):
		var row := []
		for i in range(nx + 1):
			var x := lerpf(x0, x1, float(i) / nx)
			var z := lerpf(z0, z1, float(j) / nz)
			# taper the near edge into the water so the base is never visible
			var near := clampf((z1 - z) / 240.0, 0.0, 1.0)
			var v := n.get_noise_2d(x, z) * 0.5 + 0.5
			var h := pow(v, 1.7) * 430.0 * (0.25 + 0.75 * near) - 12.0
			row.append(h)
		hs.append(row)

	for j in range(nz):
		for i in range(nx):
			var ps := []
			for d in [[0, 0], [1, 0], [1, 1], [0, 1]]:
				var ii: int = i + d[0]
				var jj: int = j + d[1]
				ps.append(Vector3(lerpf(x0, x1, float(ii) / nx), hs[jj][ii],
					lerpf(z0, z1, float(jj) / nz)))
			for tri in [[0, 2, 1], [0, 3, 2]]:
				for k in tri:
					var p: Vector3 = ps[k]
					st.set_color(_ridge_colour(p.y))
					st.add_vertex(p)
	st.generate_normals()
	var mi := MeshInstance3D.new()
	mi.mesh = st.commit()
	var m := StandardMaterial3D.new()
	m.vertex_color_use_as_albedo = true
	m.roughness = 0.97
	m.specular_mode = BaseMaterial3D.SPECULAR_DISABLED
	mi.mesh.surface_set_material(0, m)
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	root.add_child(mi)


static func _ridge_colour(h: float) -> Color:
	var forest := Color(0.17, 0.25, 0.17)
	var slope := Color(0.30, 0.31, 0.27)
	var rock := Color(0.45, 0.43, 0.41)
	var snow := Color(0.93, 0.94, 0.96)
	if h < 60.0:
		return forest.lerp(slope, clampf(h / 60.0, 0.0, 1.0))
	if h < 190.0:
		return slope.lerp(rock, (h - 60.0) / 130.0)
	return rock.lerp(snow, clampf((h - 190.0) / 90.0, 0.0, 1.0))


## Far shore: a strip of dark treeline cards, a scatter of pale houses and one
## church spire. In every reference the far bank is legible as a settlement.
static func _far_shore(root: Node3D) -> void:
	var tl := Mats.card(ProcGen.treeline_card(3), 0.0)
	for i in range(9):
		var x := -520.0 + i * 130.0
		Build.card(root, Vector3(x, 12.0, -330.0), 150.0, 40.0, tl, 0.0, 0.0, false)
	var pale := Mats.paint(Color(0.82, 0.78, 0.71), 0.9)
	var terr := Mats.paint(Color(0.48, 0.30, 0.24), 0.9)
	for i in range(34):
		var x := rng.randf_range(-420.0, 420.0)
		var z := rng.randf_range(-350.0, -318.0)
		var w := rng.randf_range(9.0, 18.0)
		var h := rng.randf_range(7.0, 13.0)
		Build.slab(root, x, z, w, w * 0.8, 0.0, h, pale).cast_shadow = 0
		Build.slab(root, x, z, w * 1.1, w * 0.9, h, 1.2, terr).cast_shadow = 0
	Build.slab(root, 96, -334, 9, 9, 0.0, 17.0, pale)
	Build.slab(root, 96, -334, 6, 6, 17.0, 16.0, pale)
	var spire := Build.cyl(root, Vector3(96, 33, -334), 0.2, 4.2, 15.0, terr, 6)
	Build.slab(root, -170, -340, 11, 11, 0.0, 14.0, pale)


# --- paving --------------------------------------------------------------------

static func _paving(root: Node3D) -> void:
	var prom := Mats.promenade()
	var walk := Mats.sidewalk()
	var setts := Mats.setts()
	# promenade
	Build.ground(root, Vector3(0, 0.0, (PROM_N + PROM_S) * 0.5), 100, PROM_S - PROM_N, prom)
	# plaza promontory
	Build.ground(root, Vector3(0, 0.0, -30.0), 24, 8.0, prom)
	# a band of different paving right at the shopfronts, as in 05
	Build.ground(root, Vector3(0, 0.01, PROM_S + 1.2), 100, 2.6, walk)
	# side street
	Build.ground(root, Vector3((STREET_W0 + STREET_W1) * 0.5, 0.0, 12.0),
		STREET_W1 - STREET_W0, 50.0, setts)
	Build.ground(root, Vector3(STREET_W0 - 1.3, 0.01, 12.0), 2.6, 50.0, walk)
	Build.ground(root, Vector3(STREET_W1 + 1.3, 0.01, 12.0), 2.6, 50.0, walk)
	# kerbs
	var kerb := Mats.cutstone()
	Build.slab(root, STREET_W0 - 0.1, 12.0, 0.25, 50, 0.0, 0.14, kerb)
	Build.slab(root, STREET_W1 + 0.1, 12.0, 0.25, 50, 0.0, 0.14, kerb)


# --- buildings -----------------------------------------------------------------

## One shop unit: mass, plinth, recessed glazed front, fascia, awning, upper
## windows with flower boxes, cornice, roof, and a warm interior light.
static func shopfront(root: Node3D, x: float, w: float, front_z: float, depth: float,
		storeys: int, wall: Material, name_txt: String, accent: Color,
		face := 0.0, awning_on := true, seed_v := 0) -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, face), Vector3(x, 0, front_z))
	root.add_child(g)

	var floor_h := 3.75
	var ground_h := 4.30
	var h := ground_h + (storeys - 1) * floor_h
	var zc := depth * 0.5

	# mass (collides)
	Build.slab(g, 0, zc, w, depth, 0, h, wall, 0.0, true)
	# plinth
	Build.slab(g, 0, 0.06, w, 0.4, 0, 0.55, Mats.cutstone())
	# cornice + roof
	Build.slab(g, 0, zc, w + 0.45, depth + 0.45, h - 0.05, 0.42, Mats.cutstone())
	Build.slab(g, 0, zc, w + 0.1, depth + 0.1, h + 0.37, 0.25, Mats.roof())

	var joinery := Mats.paint(accent, 0.42)
	var glass := Mats.glass()

	# --- ground floor shopfront -----------------------------------------------
	# Everything here hangs in FRONT of the wall face (negative local z). Solid
	# box geometry cannot be subtracted, so a shopfront modelled "recessed into"
	# the mass is simply buried inside the brick and renders as a blank wall.
	var gw := w - 0.9
	var top := ground_h - 1.05
	Build.slab(g, 0, -0.10, gw + 0.55, 0.22, 0.55, top - 0.55, joinery)   # surround
	# the lit interior sits between the wall face and the glass, so it reads
	# through the glazing the way a shop does at the end of the afternoon
	var inner := Mats.emissive(Color(1.0, 0.78, 0.48), 2.4)
	Build.slab(g, 0, -0.04, gw - 0.1, 0.05, 0.72, top - 1.0, inner)
	Build.slab(g, 0, -0.20, gw, 0.06, 0.70, top - 0.95, glass)            # glazing
	var lamp := OmniLight3D.new()
	lamp.light_color = Color(1.0, 0.82, 0.58)
	lamp.light_energy = 3.0
	lamp.omni_range = 7.5
	lamp.shadow_enabled = false
	lamp.position = Vector3(0, 2.2, -1.1)
	g.add_child(lamp)
	# mullions
	var bays := maxi(2, int(gw / 1.9))
	for i in range(1, bays):
		var mx := -gw * 0.5 + gw * float(i) / bays
		Build.slab(g, mx, -0.24, 0.10, 0.18, 0.70, top - 0.95, joinery)
	# door, off-centre
	var dx := -gw * 0.5 + 0.95
	Build.slab(g, dx, -0.14, 1.10, 0.16, 0.0, 2.35, joinery)
	Build.slab(g, dx, -0.23, 0.78, 0.05, 0.38, 1.70, glass)
	Build.sphere(g, Vector3(dx + 0.42, 1.05, -0.28), 0.045,
		Mats.paint(Color(0.72, 0.60, 0.33), 0.30, 0.85), 8)
	# step
	Build.slab(g, dx, -0.45, 1.5, 0.45, 0.0, 0.10, Mats.cutstone())

	# --- fascia ---------------------------------------------------------------
	Build.slab(g, 0, -0.16, w - 0.5, 0.30, top, 0.80, joinery)
	Build.label(g, Vector3(0, top + 0.40, -0.33), name_txt, 0.40,
		Color(0.95, 0.91, 0.80), PI, 64)

	if awning_on:
		Props.awning(g, Vector3(0, top - 0.12, -0.1), w - 1.0, PI, accent.lightened(0.05), 1.5)

	# --- upper storeys --------------------------------------------------------
	for s in range(1, storeys):
		var wy := ground_h + (s - 1) * floor_h + 0.55
		var n_win := maxi(2, int(w / 3.1))
		for i in range(n_win):
			var wx := -w * 0.5 + w * (float(i) + 0.5) / n_win
			Build.slab(g, wx, -0.05, 1.15, 0.10, wy, 1.85, glass)
			Build.slab(g, wx, -0.10, 1.38, 0.10, wy - 0.12, 2.09, joinery)
			Build.slab(g, wx, -0.12, 1.44, 0.14, wy - 0.16, 0.10, Mats.cutstone())
			Build.slab(g, wx, -0.12, 1.50, 0.14, wy + 1.97, 0.14, Mats.cutstone())
			# flower box on some sills -- the references are full of them
			if (seed_v + i + s) % 3 == 0:
				Build.slab(g, wx, -0.30, 1.05, 0.30, wy - 0.28, 0.26,
					Mats.paint(Color(0.34, 0.26, 0.19), 0.85))
				Props.flowerbed(g, Vector3(wx, wy + 0.05, -0.30), 0.95, 0.26, seed_v * 31 + i)


static func _shop_row(root: Node3D) -> void:
	# (x_centre, width, storeys, wall, name, accent, awning)
	var units := [
		[-44.0, 11.0, 2, "stucco", "PINE & PAPER", Color(0.26, 0.31, 0.38), false],
		[-33.0, 11.0, 3, "brick", "NORTHSHORE APARTMENTS", Color(0.34, 0.27, 0.22), false],
		[-22.5, 10.0, 2, "brick_tan", "SUNRISE BAKERY", Color(0.44, 0.33, 0.20), true],
		[-12.0, 11.0, 2, "stucco", "RIVERSTONE BOOKS", Color(0.17, 0.26, 0.22), true],
		[-1.0, 11.0, 2, "cutstone", "EVERYDAY MART", Color(0.24, 0.34, 0.42), true],
		[9.0, 9.0, 3, "brick", "LAKESIDE CAFE", Color(0.14, 0.24, 0.20), true],
		[26.0, 10.0, 2, "stucco", "MAPLE & CO.", Color(0.40, 0.24, 0.23), true],
		[36.0, 10.0, 3, "brick_tan", "HARBOUR CHANDLERY", Color(0.22, 0.28, 0.34), false],
		[45.5, 9.0, 2, "brick", "THE DAILY BEAN", Color(0.16, 0.27, 0.23), true],
	]
	for i in range(units.size()):
		var u: Array = units[i]
		shopfront(root, u[0], u[1], PROM_S, 14.0, u[2], _wall(u[3]), u[4], u[5],
			0.0, u[6], i * 17 + 3)


static func _wall(k: String) -> Material:
	match k:
		"brick": return Mats.brick()
		"brick_tan": return Mats.brick_tan()
		"cutstone": return Mats.cutstone()
		_: return Mats.stucco(Color(1.0, 0.96, 0.88) if randi() % 2 == 0 else Color(0.94, 0.93, 0.87))


## The side street gives the mid-distance read the references get from 05:
## a street you can see down, with buildings receding and a tree line closing it.
static func _side_street(root: Node3D) -> void:
	var west := [
		[6.0, 12.0, 2, "stucco", "BLUE DOOR DELI", Color(0.20, 0.34, 0.44)],
		[19.0, 12.0, 3, "brick", "MAPLE AVE FLATS", Color(0.38, 0.28, 0.21)],
		[32.0, 12.0, 2, "brick_tan", "PINECREEK HARDWARE", Color(0.34, 0.30, 0.22)],
	]
	for i in range(west.size()):
		var u: Array = west[i]
		# faces east (+X): rotate so its front normal points +X
		shopfront(root, STREET_W0, 12.0, u[0], 13.0, u[2], _wall(u[3]), u[4], u[5],
			-PI * 0.5, i == 0, i * 23 + 11)
	var east := [
		[7.0, 2, "stucco", "GOOD DAY GROCER", Color(0.24, 0.32, 0.26)],
		[20.0, 3, "brick", "THE LANTERN INN", Color(0.42, 0.26, 0.20)],
		[33.0, 2, "cutstone", "FERN STUDIO", Color(0.28, 0.24, 0.34)],
	]
	for i in range(east.size()):
		var u: Array = east[i]
		shopfront(root, STREET_W1, 12.0, u[0], 13.0, u[1], _wall(u[2]), u[3], u[4],
			PI * 0.5, i == 1, i * 29 + 5)

	# close the far end of the street with trees rather than a wall
	for i in range(6):
		Props.broadleaf(root, Vector3(rng.randf_range(10.0, 26.0), 0,
			rng.randf_range(42.0, 56.0)), rng.randf_range(8.0, 11.0), 700 + i)

	# street dressing
	for z in [2.0, 14.0, 26.0]:
		Props.place(root, "street_lamp_02", Vector3(STREET_W0 - 1.1, 0, z),
			PI * 0.5, Color(0.85, 0.86, 0.88))
		Props.place(root, "street_lamp_02", Vector3(STREET_W1 + 1.1, 0, z + 6.0),
			-PI * 0.5, Color(0.85, 0.86, 0.88))
		Build.blocker(root, Vector3(STREET_W0 - 1.1, 0, z), 0.16, 3.0)
		Build.blocker(root, Vector3(STREET_W1 + 1.1, 0, z + 6.0), 0.16, 3.0)
	for i in range(4):
		Props.broadleaf(root, Vector3(STREET_W1 + 1.4, 0, 4.0 + i * 11.0), 7.2, 300 + i)
	Props.bicycle(root, Vector3(STREET_W0 - 1.5, 0, 9.0), 0.2)
	Props.bin(root, Vector3(STREET_W1 + 1.6, 0, 17.0))
	Props.sandwich_board(root, Vector3(STREET_W0 - 1.6, 0, 5.4), 1.4,
		["FRESH", "TODAY"])


static func _plaza(root: Node3D) -> void:
	# fountain on the promontory, lake behind it -- the composition of 02
	var stone := Mats.cutstone()
	var cx := 0.0
	var cz := -30.0
	Build.cyl(root, Vector3(cx, 0.0, cz), 4.1, 4.2, 0.62, stone, 28)
	Build.cyl(root, Vector3(cx, 0.55, cz), 3.6, 3.7, 0.10, Mats.paint(Color(0.16, 0.30, 0.33), 0.06), 28)
	Build.blocker(root, Vector3(cx, 0, cz), 4.2, 1.0)
	Build.cyl(root, Vector3(cx, 0.6, cz), 0.75, 1.15, 1.35, stone, 18)
	Build.cyl(root, Vector3(cx, 1.95, cz), 1.45, 1.35, 0.22, stone, 20)
	Build.cyl(root, Vector3(cx, 2.17, cz), 0.22, 0.42, 1.0, stone, 14)
	Build.sphere(root, Vector3(cx, 3.35, cz), 0.42, Mats.paint(Color(0.33, 0.30, 0.21), 0.35, 0.75), 14)
	Build.label(root, Vector3(cx, 0.36, cz + 4.16), "Small Town  Brighter Tomorrows",
		0.15, Color(0.45, 0.42, 0.37), 0.0, 48)

	Props.railing(root, Vector3(-12.0, 0, -34.0), Vector3(12.0, 0, -34.0))
	Props.railing(root, Vector3(-12.0, 0, -34.0), Vector3(-12.0, 0, -26.0))
	Props.railing(root, Vector3(12.0, 0, -34.0), Vector3(12.0, 0, -26.0))

	for a in range(6):
		var ang := TAU * float(a) / 6.0 + 0.4
		Props.place(root, "painted_wooden_bench",
			Vector3(cx + cos(ang) * 6.6, 0, cz + sin(ang) * 6.6),
			-ang + PI * 0.5, BENCH_TINT)
		Build.box_blocker(root, Vector3(cx + cos(ang) * 6.6, 0.4, cz + sin(ang) * 6.6),
			Vector3(1.8, 0.8, 0.7), -ang + PI * 0.5)
	Props.wayfinder(root, Vector3(9.5, 0, -26.8), 0.6,
		["<Lakeside Trail", "<Town Square", ">Market", ">Pine Ridge"])


static func _promenade_dressing(root: Node3D) -> void:
	# lamps along the quay
	for i in range(11):
		var x := -46.0 + i * 9.2
		Props.place(root, "street_lamp_01", Vector3(x, 0, PROM_N + 1.6), 0.0,
			Color(0.80, 0.82, 0.85))
		Build.blocker(root, Vector3(x, 0, PROM_N + 1.6), 0.2, 3.2)
	# rail along the quay, broken for the promontory
	Props.railing(root, Vector3(-50.0, 0, PROM_N), Vector3(-12.0, 0, PROM_N))
	Props.railing(root, Vector3(12.0, 0, PROM_N), Vector3(50.0, 0, PROM_N))

	# street trees between promenade and shopfronts
	for i in range(9):
		var x := -45.0 + i * 11.3
		if absf(x - 17.0) < 7.0 or absf(x) < 6.0:
			continue
		Props.broadleaf(root, Vector3(x, 0, PROM_N + 3.4), rng.randf_range(7.0, 9.2), 100 + i)

	# planters, benches, boards, bins along the promenade
	for i in range(8):
		var x := -42.0 + i * 12.0
		Props.planter(root, Vector3(x, 0, PROM_N + 3.0), 2.6, 1.1, 40 + i)
	for i in range(7):
		var x := -38.0 + i * 12.5
		Props.place(root, "painted_wooden_bench", Vector3(x, 0, PROM_N + 7.4), PI,
			BENCH_TINT)
		Build.box_blocker(root, Vector3(x, 0.4, PROM_N + 7.4), Vector3(1.8, 0.8, 0.7))
	for i in range(6):
		Props.bollard(root, Vector3(-30.0 + i * 13.0, 0, PROM_S + 2.9))
	Props.bin(root, Vector3(-16.0, 0, PROM_N + 8.0))
	Props.bin(root, Vector3(22.0, 0, PROM_N + 8.0))

	# cafe frontage at x=9 -- the conversational-scale view of 03
	var cafe_x := 9.0
	Props.sandwich_board(root, Vector3(cafe_x - 4.6, 0, PROM_S - 2.2), 0.25,
		["GOOD COFFEE", "KINDER PEOPLE", "BRIGHTER DAYS"])
	Props.sandwich_board(root, Vector3(-1.0 - 5.0, 0, PROM_S - 2.0), -0.3,
		["SNACKS", "DAILY GOODS", "LOCAL LIFE"])
	Props.hanging_sign(root, Vector3(cafe_x - 4.2, 3.3, PROM_S - 0.15), PI,
		"CAFE", Color(0.14, 0.24, 0.20))
	Props.hanging_sign(root, Vector3(-12.0 - 4.2, 3.3, PROM_S - 0.15), PI,
		"BOOKS", Color(0.17, 0.26, 0.22))
	for i in range(3):
		var tx := cafe_x - 2.6 + i * 2.6
		Props.place(root, "outdoor_table_chair_set_01", Vector3(tx, 0, PROM_S - 2.6),
			rng.randf_range(-0.4, 0.4), Color(1.02, 0.97, 0.90))
		Build.blocker(root, Vector3(tx, 0, PROM_S - 2.6), 0.55, 0.8)
	Props.parasol(root, Vector3(cafe_x + 2.6, 0, PROM_S - 3.4), Color(0.86, 0.84, 0.76))
	for x in [-22.5, -1.0, 26.0, 45.5]:
		Props.place(root, "potted_plant_02", Vector3(x + 4.0, 0, PROM_S - 1.0), 0.0,
			Color(0.95, 1.02, 0.92))
		Props.place(root, "potted_plant_02", Vector3(x - 4.0, 0, PROM_S - 1.0), 2.1,
			Color(0.95, 1.02, 0.92))
	Props.bicycle(root, Vector3(-19.0, 0, PROM_S - 1.6), 1.7, Color(0.22, 0.34, 0.40))


## Nature outside the town: the west end of the shore, per 06.
static func _lake_trail(root: Node3D) -> void:
	var path := Mats.pbr("cobblestone_floor_08", 1.2, Color(1.25, 1.12, 0.92), 1.0)
	for i in range(9):
		var x := -52.0 - i * 9.0
		var z := PROM_N + 4.0 + sin(i * 0.7) * 5.0
		Build.ground(root, Vector3(x, 0.01, z), 10.0, 3.4, path, 0, false)
		Props.grass_patch(root, Vector3(x, 0, z - 3.4), 9.0, 3.0, 800 + i, 2.2)
		Props.grass_patch(root, Vector3(x, 0, z + 3.4), 9.0, 3.4, 900 + i, 2.2)
		if i % 2 == 0:
			Props.flowerbed(root, Vector3(x + 3.0, 0, z + 2.6), 4.0, 1.6, 950 + i)
		Props.boulder(root, Vector3(x - 3.0, -0.1, z - 3.2), rng.randf_range(0.6, 1.4), 60 + i)

	for i in range(16):
		Props.conifer(root, Vector3(rng.randf_range(-140.0, -56.0), 0,
			rng.randf_range(-16.0, 26.0)), rng.randf_range(9.0, 15.0), 500 + i)
	for i in range(5):
		Props.broadleaf(root, Vector3(rng.randf_range(-130.0, -58.0), 0,
			rng.randf_range(-14.0, 20.0)), rng.randf_range(8.0, 11.0), 600 + i)

	# jetty and a rowing boat
	var planks := Mats.planks()
	var jx := -74.0
	for i in range(13):
		Build.slab(root, jx, SHORE_Z - 0.6 - i * 1.6, 3.0, 1.5, 0.05, 0.12, planks)
	for i in range(7):
		var pz := SHORE_Z - 1.1 - i * 3.0
		for px in [jx - 1.3, jx + 1.3]:
			Build.cyl(root, Vector3(px, WATER_Y - 1.2, pz), 0.11, 0.13, 1.6, planks, 8)
	Build.box_blocker(root, Vector3(jx, 0.4, SHORE_Z - 9.6), Vector3(3.2, 0.5, 20.0))
	var boat := Node3D.new()
	boat.position = Vector3(jx + 2.6, WATER_Y + 0.22, SHORE_Z - 13.6)
	boat.rotation.y = 0.35
	root.add_child(boat)
	var hull := Mats.paint(Color(0.68, 0.30, 0.24), 0.55)
	Build.slab(boat, 0, 0, 1.35, 3.6, -0.18, 0.46, hull)
	Build.slab(boat, 0, 0, 1.05, 3.2, 0.10, 0.10, Mats.planks())
	Build.slab(boat, 0, 0.8, 1.0, 0.18, 0.10, 0.10, Mats.planks())
	Build.slab(boat, 0, -0.8, 1.0, 0.18, 0.10, 0.10, Mats.planks())

	Props.wayfinder(root, Vector3(-52.0, 0, PROM_N + 7.0), 1.9,
		["<Lakeside Trail", "<Pine Ridge", ">Town Centre"])
	Props.place(root, "painted_wooden_bench", Vector3(-58.0, 0, PROM_N + 7.6), 2.9,
		BENCH_TINT)
	# grass creeping over the shore edge
	for i in range(14):
		Props.grass_patch(root, Vector3(rng.randf_range(-150.0, -50.0), 0,
			rng.randf_range(-22.0, 28.0)), 4.0, 4.0, 1000 + i, 1.4)


# --- people --------------------------------------------------------------------

static func _people(root: Node3D) -> void:
	var r := RandomNumberGenerator.new()
	r.seed = 4242

	# walkers along the promenade
	var routes := [
		[Vector3(-34, 0, -19.0), Vector3(-6, 0, -19.5)],
		[Vector3(30, 0, -21.0), Vector3(2, 0, -20.0)],
		[Vector3(-16, 0, -16.5), Vector3(16, 0, -16.0)],
		[Vector3(17.0, 0, 4.0), Vector3(17.0, 0, 30.0)],
		[Vector3(19.5, 0, 26.0), Vector3(19.5, 0, 2.0)],
		[Vector3(-46, 0, -21.0), Vector3(-20, 0, -20.0)],
	]
	for i in range(routes.size()):
		var n := NPC.make(r, NPC.Pose.WALK)
		n.path_a = routes[i][0]
		n.path_b = routes[i][1]
		n.speed = r.randf_range(1.05, 1.45)
		n.position = n.path_a
		root.add_child(n)

	# seated at the cafe tables
	for i in range(2):
		var s := NPC.make(r, NPC.Pose.SIT)
		s.position = Vector3(9.0 - 2.6 + i * 5.2, 0, PROM_S - 3.5)
		s.rotation.y = PI + (0.4 if i == 0 else -0.5)
		root.add_child(s)

	# on a promenade bench, looking at the lake
	var b := NPC.make(r, NPC.Pose.SIT)
	b.position = Vector3(-38.0, 0, PROM_N + 7.4)
	b.rotation.y = 0.0
	root.add_child(b)

	# standing pair by the fountain, and one at the mart window
	for p in [[Vector3(4.2, 0, -28.0), 2.4], [Vector3(5.2, 0, -28.7), -0.9],
			[Vector3(-3.0, 0, PROM_S - 1.9), PI]]:
		var st := NPC.make(r, NPC.Pose.STAND)
		st.position = p[0]
		st.rotation.y = p[1]
		root.add_child(st)

	# a dog, because every single reference has one
	_dog(root, Vector3(-36.6, 0, PROM_N + 8.4), 0.4)


static func _dog(root: Node3D, pos: Vector3, yaw: float) -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	root.add_child(g)
	var coat := Mats.paint(Color(0.72, 0.52, 0.27), 0.88)
	var dark := Mats.paint(Color(0.16, 0.13, 0.11), 0.7)
	Build.sphere(g, Vector3(0, 0.44, 0), 0.5, coat, 12).scale = Vector3(0.44, 0.40, 0.90)
	Build.sphere(g, Vector3(0, 0.60, 0.50), 0.5, coat, 12).scale = Vector3(0.32, 0.30, 0.36)
	Build.sphere(g, Vector3(0, 0.55, 0.68), 0.5, coat, 10).scale = Vector3(0.18, 0.16, 0.24)
	Build.sphere(g, Vector3(0, 0.55, 0.80), 0.5, dark, 8).scale = Vector3(0.09, 0.08, 0.08)
	for ex in [-0.09, 0.09]:
		Build.sphere(g, Vector3(ex, 0.65, 0.66), 0.5, dark, 8).scale = Vector3(0.05, 0.05, 0.04)
		Build.sphere(g, Vector3(ex * 1.7, 0.70, 0.46), 0.5, coat, 8).scale = Vector3(0.09, 0.19, 0.06)
	for sx in [-0.16, 0.16]:
		for sz in [-0.28, 0.30]:
			Build.cyl(g, Vector3(sx, 0.0, sz), 0.055, 0.06, 0.34, coat, 7)
	var tail := Build.cyl(g, Vector3(0, 0.52, -0.38), 0.035, 0.05, 0.42, coat, 6)
	tail.rotation.x = -0.9

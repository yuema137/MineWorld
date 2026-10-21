## The light of a depiction file's `light` section (`DECISIONS.md` ARC-177; S33-d sec.5.7): `VoxelGI`
## volumes baked at load over the boxes the file names, and `ReflectionProbe`s updated once -- the café's
## pattern (`SliceMain._voxel_gi`, `_reflections`), placed by data instead of by the café's constants.
## `SliceMain` calls these in place of the café's when the depiction has a `light` section, after the
## scene is built (a bake reads the built scene). The fixed sun and sky stay the slice's.
##
## The bake's tuning is the slice's, shared by every world (`PRESENTATION.md` sec.6's schema defaults).
## Nothing here names a world, a place or a tag.
class_name DepictionLight
extends RefCounted

## The slice's VoxelGI tuning (`SliceMain._voxel_gi`, accepted 2026-10-09).
const GI_ENERGY := 1.15
const GI_PROPAGATION := 0.62
const GI_BIAS := 1.4
const GI_NORMAL_BIAS := 0.6
## A GI volume starts this far below the ground at its centre, so floors and the foot of every wall are
## inside the bake (schema default).
const GI_BELOW := 0.5
## An outdoor probe's intensity and reach (the café's frontage probe).
const PROBE_INTENSITY := 0.85
const PROBE_DISTANCE := 60.0
const SUBDIVS := {64: VoxelGI.SUBDIV_64, 128: VoxelGI.SUBDIV_128, 256: VoxelGI.SUBDIV_256}


## Bakes one VoxelGI per `light.gi` box under `root`.
static func voxel_gi(root: Node3D, light: Dictionary, d: SliceDataDepiction) -> void:
	var k := 0
	for gi in (light["gi"] if light.get("gi") != null else []):
		var box := _box(gi["box_mm"], d)
		var ground := d.ground_at(_centre_mm(gi["box_mm"])).y
		var h := float(gi["height_m"])
		var v := VoxelGI.new()
		v.name = "DepictionGI%d" % k
		v.data = VoxelGIData.new()
		v.subdiv = SUBDIVS.get(int(gi["subdiv"]), VoxelGI.SUBDIV_128)
		v.size = Vector3(box.size.x, h, box.size.y)
		v.position = Vector3(box.get_center().x, ground - GI_BELOW + h * 0.5, box.get_center().y)
		root.add_child(v)
		v.bake(root, false)
		var data := v.data
		data.energy = GI_ENERGY
		data.propagation = GI_PROPAGATION
		data.bias = GI_BIAS
		data.normal_bias = GI_NORMAL_BIAS
		k += 1


## One ReflectionProbe per `light.probes` entry under `root`, updated once.
static func probes(root: Node3D, light: Dictionary, d: SliceDataDepiction) -> void:
	var k := 0
	for p in (light["probes"] if light.get("probes") != null else []):
		var box := _box(p["box_mm"], d)
		var probe := ReflectionProbe.new()
		probe.name = "DepictionProbe%d" % k
		probe.size = Vector3(box.size.x, float(p["height_m"]), box.size.y)
		probe.position = Vector3(box.get_center().x, d.frame_origin().y + float(p["centre_up_m"]),
			box.get_center().y)
		probe.box_projection = true
		probe.interior = false
		probe.intensity = PROBE_INTENSITY
		probe.max_distance = PROBE_DISTANCE
		probe.update_mode = ReflectionProbe.UPDATE_ONCE
		root.add_child(probe)
		k += 1


## A plan rectangle (mm) as a scene rectangle in (x, z).
static func _box(r: Array, d: SliceDataDepiction) -> Rect2:
	var a := d.to_scene(Vector2(float(r[0]), float(r[1])))
	var b := d.to_scene(Vector2(float(r[0]) + float(r[2]), float(r[1]) + float(r[3])))
	var lo := Vector2(minf(a.x, b.x), minf(a.z, b.z))
	return Rect2(lo, Vector2(absf(b.x - a.x), absf(b.z - a.z)))


static func _centre_mm(r: Array) -> Vector2:
	return Vector2(float(r[0]) + float(r[2]) * 0.5, float(r[1]) + float(r[3]) * 0.5)

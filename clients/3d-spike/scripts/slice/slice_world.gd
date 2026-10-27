## The scene's place volumes: where a body is in THIS scene (`VISUAL_SLICE.md` sec.9).
##
## An `Area3D` per drawn place, named by its scene volume id. The client reports which volume the
## player is in; it never decides what that means, what is allowed there, or who else is present --
## that is the server's, and `ADOPTION.md` sec.3.3 makes deciding it here a defect rather than an
## optimisation. Which of the world's places each volume depicts is learned from the server
## (`SliceLayout`), never listed.
##
## Every world is drawn by its depiction (`SliceWorlds`): a depiction file and the pack's place art, a
## stub, or plainly from disclosure. Social Café's street, which this file once assembled, is pack art
## since S14 16f-d2, authored by `tools/presentation/social_cafe/source.gd`.
class_name SliceWorld
extends RefCounted


static func _place(g: Node3D, id: String, centre: Vector3, size: Vector3) -> void:
	var a := Area3D.new()
	a.name = "Place_" + id
	a.monitorable = false
	a.collision_layer = 0
	a.collision_mask = 0
	a.set_meta("place_id", id)
	var cs := CollisionShape3D.new()
	var b := BoxShape3D.new()
	b.size = size
	cs.shape = b
	a.add_child(cs)
	a.position = centre
	g.add_child(a)


## Which place a world-space point is in, innermost first. A room's volume can sit
## inside a street's, so order matters and is stated rather than relied on.
static func place_at(root: Node3D, p: Vector3) -> String:
	var best := ""
	var best_vol := INF
	for c in root.get_children():
		if not (c is Area3D) or not c.has_meta("place_id"):
			continue
		var a := c as Area3D
		var cs := a.get_child(0) as CollisionShape3D
		var box := (cs.shape as BoxShape3D).size
		var local := p - a.position
		if absf(local.x) <= box.x * 0.5 and absf(local.y) <= box.y * 0.5 \
				and absf(local.z) <= box.z * 0.5:
			var vol := box.x * box.y * box.z
			if vol < best_vol:
				best_vol = vol
				best = str(a.get_meta("place_id"))
	return best

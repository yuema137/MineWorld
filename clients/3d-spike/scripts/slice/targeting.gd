## What the player is looking at: one physics ray from the active camera
## through the centre of the screen (step-15 §4.4, D-16a-2).
##
## Acquisition only. A target is whatever the ray meets first, so walls and
## furniture hide what is behind them, exactly as they hide it from the eye. It
## says nothing about whether anything may be done to the target: how far is too
## far, who is busy, what is allowed -- every such question is the server's
## (`ENGINEERING_RULES.md` §§4, 7-8). `RAY_LENGTH` is how far the client looks
## for a thing to name, not a reach: an interaction with something farther than
## a step away is still sent, and the server answers it.
##
## People are hit through a pick collider each perceived figure carries
## (`person_collider`), on `Build.LAYER_BODIES`, which neither the player nor
## the camera masks in PR 16a: it changes what can be targeted and nothing
## about movement.
class_name SliceTargeting
extends RefCounted

const RAY_LENGTH := 30.0
## What the ray can meet: the scene's geometry, which hides, and people.
const RAY_MASK := Build.LAYER_WORLD | Build.LAYER_BODIES


## What the active camera of `viewport` is aimed at.
## `{ "entity": <EntityId string, or "">, "point": Vector3 or null,
##    "collider": <the path of the node met first, or ""> }`
static func aim(viewport: Viewport) -> Dictionary:
	var none := { "entity": "", "point": null, "collider": "" }
	var cam := viewport.get_camera_3d() if viewport != null else null
	if cam == null or cam.get_world_3d() == null:
		return none
	var centre := viewport.get_visible_rect().size * 0.5
	var from := cam.project_ray_origin(centre)
	var to := from + cam.project_ray_normal(centre) * RAY_LENGTH
	var q := PhysicsRayQueryParameters3D.create(from, to, RAY_MASK)
	q.collide_with_areas = false
	var hit := cam.get_world_3d().direct_space_state.intersect_ray(q)
	if hit.is_empty():
		return none
	var collider := hit["collider"] as Node
	return {
		"entity": _entity_of(collider),
		"point": hit["position"],
		"collider": String(collider.get_path()) if collider != null else "",
	}


## The pick collider a perceived person's figure carries: the person capsule,
## centred on the figure's feet, on `LAYER_BODIES`, masking nothing. An
## `AnimatableBody3D` because the figure moves (12e masks the same body for the
## player, step-15 D-16a-3).
static func person_collider() -> AnimatableBody3D:
	var body := AnimatableBody3D.new()
	body.name = "PickBody"
	body.collision_layer = Build.LAYER_BODIES
	body.collision_mask = 0
	# The figure is placed where each observation says, from outside the physics
	# step; with sync on, the body would wait for a physics-frame motion that
	# never comes and stay where it was made (E16a-4).
	body.sync_to_physics = false
	var shape := CollisionShape3D.new()
	var cap := CapsuleShape3D.new()
	cap.radius = Player.CAPSULE_RADIUS
	cap.height = Player.CAPSULE_HEIGHT
	shape.shape = cap
	shape.position = Vector3(0, Player.CAPSULE_HEIGHT * 0.5, 0)
	body.add_child(shape)
	return body


## The entity a collider stands for: the `entity_id` metadata of the collider
## or of the nearest ancestor that carries it (a figure carries it, as a string,
## `ADOPTION.md` §3.1). "" for the scene's own geometry.
static func _entity_of(n: Node) -> String:
	while n != null:
		if n.has_meta("entity_id"):
			return String(n.get_meta("entity_id"))
		n = n.get_parent()
	return ""

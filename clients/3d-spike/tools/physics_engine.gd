class_name PhysicsEngineProbe
extends SceneTree
## Which 3D physics engine this project runs (step-15 §18.3 J-0, DEP-20).
##
##   godot --headless --path clients/3d-spike --script res://tools/physics_engine.gd
##
## The setting alone is not evidence: "DEFAULT" names no engine (step-15
## F-S14-2), and the server singleton reports its class as the abstract
## `PhysicsServer3D` under either engine. So the engine is told by behaviour.
func _init() -> void:
	print("physics/3d/physics_engine setting: '%s'" % setting())
	print("running 3D physics engine: %s" % running())
	quit(0)


static func setting() -> String:
	return String(ProjectSettings.get_setting("physics/3d/physics_engine", ""))


## "Jolt Physics", "Godot Physics" or "unknown", with the evidence.
##
## The discriminator is a new space's solver iterations. Godot Physics answers
## with `physics/3d/solver/solver_iterations` (16 by default); Jolt does not use
## that setting and answers 8. Measured on Godot 4.7.2 with each engine selected,
## 2026-10-08 (step-15 §18.7 E16a-1). Jolt is named only when the behaviour and
## the setting agree, so neither alone can claim it.
static func running() -> String:
	var s := PhysicsServer3D.space_create()
	var iters := int(PhysicsServer3D.space_get_param(s, PhysicsServer3D.SPACE_PARAM_SOLVER_ITERATIONS))
	PhysicsServer3D.free_rid(s)
	var godot_iters := int(ProjectSettings.get_setting("physics/3d/solver/solver_iterations", 16))
	var engine := "unknown"
	if iters == godot_iters:
		engine = "Godot Physics"
	elif setting() == "Jolt Physics":
		engine = "Jolt Physics"
	return "%s (a new space's solver iterations %d; Godot Physics' setting %d)" % [
		engine, iters, godot_iters]

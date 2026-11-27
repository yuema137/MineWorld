extends RefCounted
## Runtime physics diagnostic shared by the slice drive and the tools CLI.
## Kept outside tools/ because exported clients exclude that directory (ARC-80).
## The setting alone is not evidence: DEFAULT names no engine, and both engines
## report the abstract PhysicsServer3D class (step-15 F-S14-2, DEP-20).

static func setting() -> String:
	return String(ProjectSettings.get_setting("physics/3d/physics_engine", ""))


## "Jolt Physics", "Godot Physics" or "unknown", with the evidence.
##
## The discriminator is a new space's solver iterations. Godot Physics answers
## with `physics/3d/solver/solver_iterations` (16 by default); Jolt does not use
## that setting and answers 8. Measured on Godot 4.7.2 with each engine selected,
## 2026-10-08 (step-15 §19.7 E16a-1). Jolt is named only when the behaviour and
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

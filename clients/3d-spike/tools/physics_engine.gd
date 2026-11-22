extends SceneTree
## Print the same physics diagnostic used by the runtime slice drive (DEP-20).
##
##   godot --headless --path clients/3d-spike --script res://tools/physics_engine.gd

const PhysicsEngine := preload("res://scripts/physics_engine.gd")


func _init() -> void:
	print("physics/3d/physics_engine setting: '%s'" % PhysicsEngine.setting())
	print("running 3D physics engine: %s" % PhysicsEngine.running())
	quit(0)

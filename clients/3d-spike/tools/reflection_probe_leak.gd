extends SceneTree
## The slice's exit warning `7 RIDs of type "Texture" were leaked`, reproduced
## without the slice (step-15 §19.7 E16a-6; HUMAN_REVIEW_QUEUE VIS-3D-GODOT-2
## limitation 8).
##
##   godot --path clients/3d-spike --resolution 320x180 --script res://tools/reflection_probe_leak.gd -- probe
##   godot --path clients/3d-spike --resolution 320x180 --script res://tools/reflection_probe_leak.gd -- none
##
## An empty scene with one camera, drawn for 30 frames: with `probe` it also has
## one ReflectionProbe, freed at frame 20, before the quit. On Godot
## 4.7.2.stable (Metal, Forward+) `probe` prints the same 7 leaked Texture RIDs
## on exit and `none` prints nothing; headless prints nothing either way. The
## textures are the renderer's, not a resource the project holds.
var _n := 0
var _probe: ReflectionProbe = null


func _initialize() -> void:
	var world := Node3D.new()
	root.add_child(world)
	world.add_child(Camera3D.new())
	if "probe" in OS.get_cmdline_user_args():
		_probe = ReflectionProbe.new()
		_probe.update_mode = ReflectionProbe.UPDATE_ONCE
		world.add_child(_probe)
	print("reflection probe: %s" % ("yes" if _probe != null else "no"))


func _process(_delta: float) -> bool:
	_n += 1
	if _n == 20 and _probe != null:
		_probe.free()
		_probe = null
	return _n > 30

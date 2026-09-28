extends SceneTree
## Measure the authored ground speed of each locomotion clip.
##
## `Normalize Position Tracks` rescales *position* tracks; stride length lives in
## the leg *rotations*, so it is not normalised and cannot be read off the file.
## It has to be measured from the posed skeleton: sample a foot relative to the
## hips across one cycle, take the peak-to-peak travel along the forward axis --
## that is one stride -- and a cycle contains two of them.
##
##   godot --path clients/3d-spike --script res://tools/measure_stride.gd
func _initialize() -> void:
	var n := Node.new()
	n.set_script(preload("res://tools/measure_stride_runner.gd"))
	root.add_child(n)

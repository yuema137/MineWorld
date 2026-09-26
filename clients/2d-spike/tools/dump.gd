extends SceneTree
func _initialize() -> void:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path("res://shots"))
	for n in ["shop_cafe", "player_front", "tree_a"]:
		var t: Texture2D = load("res://art/svg/%s.svg" % n)
		var img := t.get_image()
		img.save_png("res://shots/sprite_%s.png" % n)
		print(n, " ", img.get_width(), "x", img.get_height())
	quit(0)

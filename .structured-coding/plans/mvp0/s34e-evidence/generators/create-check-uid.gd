extends SceneTree
func _init() -> void:
	var path := "/tmp/mineworld-recover-s34e/clients/shared/checks/sit_pose_check.gd.uid"
	assert(not FileAccess.file_exists(path))
	var file := FileAccess.open(path, FileAccess.WRITE)
	file.store_string(ResourceUID.id_to_text(ResourceUID.create_id()) + "\n")
	quit()

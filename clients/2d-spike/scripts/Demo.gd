extends RefCounted

## Mock scene data for the spike.
##
## Deliberately dumb. A DemoPlace and a DemoPerson answer "what do I draw, and
## where" and nothing else. They hold no rules, no affordances, no permissions
## and no opinion about what may happen in the world; that is the kernel's
## business and none of it is modelled here.

class DemoPlace extends RefCounted:
	var id: StringName
	var sprite: StringName        ## which prop to draw
	var at: Vector2               ## world position of its ground anchor
	var label: String = ""        ## lettering painted on its sign, if any

	func _init(p_id: StringName, p_sprite: StringName, p_at: Vector2, p_label := "") -> void:
		id = p_id
		sprite = p_sprite
		at = p_at
		label = p_label


class DemoPerson extends RefCounted:
	var id: StringName
	var sprite: StringName        ## base sprite name; "_front"/"_back" are appended
	var at: Vector2               ## world position
	var route: PackedVector2Array ## points to amble between, empty for stationary
	var speed: float = 0.9        ## world units per second
	var seated: bool = false

	func _init(p_id: StringName, p_sprite: StringName, p_at: Vector2,
			p_route := PackedVector2Array(), p_speed := 0.9, p_seated := false) -> void:
		id = p_id
		sprite = p_sprite
		at = p_at
		route = p_route
		speed = p_speed
		seated = p_seated

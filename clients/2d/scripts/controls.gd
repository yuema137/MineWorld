## The 2D client's key bindings, in one table (`clients/shared/SETTINGS.md` §7.1; S20 SET-b, SD-SETb-15).
## `app.gd` reads it for walking (its InputMap actions) and for the keys it answers in `_unhandled_input`, and
## the Controls tab's key map is built from it, so the map shows the keys the client really takes. Binding a
## key here changes the game; a binding is never a setting (QSET-7).
class_name Controls2D
extends RefCounted

## Walking: InputMap actions, each with its physical keys. `app.gd` adds them to InputMap at start.
const WALK := {
	"move_left": [KEY_A, KEY_LEFT],
	"move_right": [KEY_D, KEY_RIGHT],
	"move_up": [KEY_W, KEY_UP],
	"move_down": [KEY_S, KEY_DOWN],
}

## The keys `app.gd` answers in `_unhandled_input`, with the action each runs. Esc (`menu`) is checked first
## there, before the others, as it always was.
const ACTIONS := [
	["menu", KEY_ESCAPE, "ui.settings.key.menu"],
	["spot_menu", KEY_E, "ui.settings.key.spot_menu"],
	["person_menu", KEY_Q, "ui.settings.key.person_menu"],
	["things", KEY_I, "ui.settings.key.things"],
	["conversations", KEY_H, "ui.settings.key.conversations"],
]


## The action a key runs, or "" for a key the client does not answer.
static func action_for(keycode: Key) -> String:
	for pair in ACTIONS:
		if pair[1] == keycode:
			return pair[0]
	return ""


## The key map the Controls tab shows: walking first (one row, its eight keys), then each action's key.
static func key_rows() -> Array[MineWorldKeyRow]:
	var rows: Array[MineWorldKeyRow] = []
	var walk := PackedInt32Array()
	for action in ["move_left", "move_right", "move_up", "move_down"]:
		for code in WALK[action]:
			walk.append(code)
	rows.append(MineWorldKeyRow.new("ui.settings.key.walk", walk))
	for pair in ACTIONS:
		rows.append(MineWorldKeyRow.new(pair[2], PackedInt32Array([pair[1]])))
	return rows

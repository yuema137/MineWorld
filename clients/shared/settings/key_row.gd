## One row of a client's key map (`clients/shared/SETTINGS.md` §7.1; SD-SETb-10): an action's catalog key
## and the physical keycodes the client binds to it. The module reads no binding source; each client
## declares its rows from its own bindings, so a binding is never a setting (QSET-7).
class_name MineWorldKeyRow
extends RefCounted

## The catalog key of the action, `ui.settings.key.<action>`.
var action: String
## Physical keycodes (`Key` values), in the order the map shows them.
var keys: PackedInt32Array


func _init(an_action: String, some_keys: PackedInt32Array) -> void:
	action = an_action
	keys = some_keys

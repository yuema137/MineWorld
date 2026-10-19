extends SceneTree

# The 2D half of the cross-client identity check (`clients/shared/checks/identity.sh`; the defect it
# pins: `.structured-coding/plans/mvp0/pr-vf-person-identity.md`).
#
#   godot --headless --path clients/2d --script res://scripts/harness/identity_check.gd -- \
#       --server=<address> --invite=<invite> --seats=alice,bob,...
#
# Seats one connection per seat at once, takes each seat's first observation, and prints, sorted:
#
#   IDENTITY seat <seat> <observer id> name=<disclosed name> look=<disclosed look>
#   IDENTITY person <id> name=<disclosed name> look=<disclosed look>
#
# for the observer of every seat and every person any seat perceives — only what the server
# disclosed. The 3D half prints the same lines, and identity.sh requires them to be equal. It then
# resolves every person through this client's real resolver (`people.gd` `role_for` and the active
# set of the default pack) and fails when a person whose look the pack binds is drawn by any other
# role: a body is chosen from the disclosed look, never from the seat or the observer.

const Client := preload("res://mineworld/world_client.gd")
const Presentation := preload("res://scripts/presentation.gd")
const People := preload("res://scripts/scene/people.gd")
const PACK := "../../presentation/mineworld-default/2D"
const BUDGET := 30.0

var _lines := {}
var _pending := {}
var _failures := 0
var _elapsed := 0.0
var _done := false
var _presentation


func _initialize() -> void:
	var address := ""
	var invite := ""
	var seats := PackedStringArray()
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--server="):
			address = a.substr(9)
		elif a.begins_with("--invite="):
			invite = a.substr(9)
		elif a.begins_with("--seats="):
			seats = a.substr(8).split(",", false)
	_presentation = Presentation.new()
	var dir := ProjectSettings.globalize_path("res://").path_join(PACK).simplify_path()
	if not _presentation.load_pack(dir, ""):
		_fail("the 2D pack loads: %s" % [_presentation.errors])
	for seat in seats:
		var c: Node = Client.new()
		root.add_child(c)
		_pending[seat] = true
		c.observed.connect(func(obs) -> void: _on_observed(seat, obs))
		c.connect_to_world(address, seat, invite, "identity-2d-" + seat)


func _process(delta: float) -> bool:
	_elapsed += delta
	if not _done and (_pending.is_empty() or _elapsed > BUDGET):
		_finish()
	return _done


func _on_observed(seat: String, obs) -> void:
	if not _pending.has(seat):
		return
	_pending.erase(seat)
	var me: String = obs.observer()
	_lines["IDENTITY seat %s %s name=%s look=%s" % [seat, me, obs.display_name(me), obs.look(me)]] = true
	for e in obs.entities():
		if String(e.get("entity_type", "")) != "person":
			continue
		var id := String(e.get("id", ""))
		var look: String = obs.look(id)
		_lines["IDENTITY person %s name=%s look=%s" % [id, obs.display_name(id), look]] = true
		var role: String = People.role_for(_presentation, look, id == me, id)
		if look != "" and role != "look:" + look:
			_fail("seat %s: person %s (look %s) is drawn by role %s, not by its look" % [seat, id, look, role])


func _finish() -> void:
	_done = true
	for seat in _pending:
		_fail("seat %s observed within %d s" % [seat, int(BUDGET)])
	var sorted := _lines.keys()
	sorted.sort()
	for line in sorted:
		print(line)
	print("IDENTITY-2D %s" % ("PASS" if _failures == 0 else "FAIL %d" % _failures))
	quit(0 if _failures == 0 else 1)


func _fail(what: String) -> void:
	_failures += 1
	printerr("FAIL " + what)

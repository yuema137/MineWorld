extends SceneTree

# The far side of `delta` frames (step-12 §17 CA-15, SD-C11; `server/PROTOCOL.md` §5.3).
#
#   bash clients/protocol/run.sh deltas
#
# Two claims, in order:
#
#   golden  MineWorldDelta.apply(base, delta) == next for every reviewed case in
#           server/tests/frames/deltas/ — the files the Rust applier is checked against (CA-9)
#   live    against a hosted market town with the default --keyframe-every 50, the module applies
#           [constant WINDOW] seconds of deltas with no base mismatch and no unknown remove: no
#           resynchronisation, no disconnect, and observations keep arriving whole
#
#   godot --headless --path clients/protocol --script res://checks/delta_check.gd -- \
#       --address 127.0.0.1:7878 --seat wanderer --invite <the server's invite>
#
# Prints `[check] PASS|FAIL <claim>: <observed>` lines and exits 1 on any failure.

const Client := preload("res://mineworld/world_client.gd")

## How long the live half runs, in seconds.
const WINDOW := 60.0

var _world: Node
var _address := "127.0.0.1:7878"
var _seat := "wanderer"
var _invite := ""
var _elapsed := 0.0
var _failures := 0
var _done := false
var _observed := 0


func _initialize() -> void:
	_read_arguments()
	_golden()
	_world = Client.new()
	root.add_child(_world)
	_world.observed.connect(func(_observation): _observed += 1)
	_world.disconnected.connect(_on_disconnected)
	_note("connecting to %s as seat '%s' for %d s" % [_address, _seat, int(WINDOW)])
	_world.connect_to_world(_address, _seat, _invite, "delta-check")


func _process(delta: float) -> bool:
	_elapsed += delta
	if not _done and _elapsed > WINDOW:
		_expect("deltas were applied", _world.deltas_applied > 300, true)
		_expect("none refused (no base mismatch, no unknown remove)", _world.deltas_refused, 0)
		_expect("observations kept arriving whole", _observed > 300, true)
		_note("applied %d deltas, %d observations emitted" % [_world.deltas_applied, _observed])
		_finish()
	return false


## Every reviewed case, through the module's own applier.
func _golden() -> void:
	var directory := ProjectSettings.globalize_path("res://").path_join("../../server/tests/frames/deltas")
	var files := DirAccess.get_files_at(directory)
	_expect("the reviewed cases are there", files.size() >= 7, true)
	for name in files:
		if not name.ends_with(".json"):
			continue
		var case: Variant = JSON.parse_string(FileAccess.get_file_as_string(directory.path_join(name)))
		if typeof(case) != TYPE_DICTIONARY:
			_fail("%s parses" % name, "not a JSON object")
			continue
		var next: Variant = MineWorldDelta.apply(case["base"], case["delta"])
		_expect("%s: apply(base, delta) == next" % name, next == case["next"], true)


func _read_arguments() -> void:
	var arguments := OS.get_cmdline_user_args()
	for index in range(arguments.size() - 1):
		match String(arguments[index]):
			"--address":
				_address = String(arguments[index + 1])
			"--seat":
				_seat = String(arguments[index + 1])
			"--invite":
				_invite = String(arguments[index + 1])


func _on_disconnected(reason: String) -> void:
	if not _done:
		_fail("the connection lasted the window", reason)
		_finish()


func _finish() -> void:
	if _done:
		return
	_done = true
	_world.leave_world()
	print("[check] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
	quit(0 if _failures == 0 else 1)


func _expect(claim: String, observed: Variant, expected: Variant) -> void:
	var held: bool = typeof(observed) == typeof(expected) and observed == expected
	if not held:
		_failures += 1
	print("[check] %s %s: %s%s" % [
		"PASS" if held else "FAIL", claim, JSON.stringify(observed),
		"" if held else " (expected %s)" % JSON.stringify(expected),
	])


func _fail(claim: String, observed: String) -> void:
	_failures += 1
	print("[check] FAIL %s: %s" % [claim, observed])


func _note(line: String) -> void:
	print("[check] %s" % line)

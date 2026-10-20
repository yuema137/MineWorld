extends SceneTree

## The host module, headless, against a scripted transport (`clients/shared/HOST.md` §8; pr-tw-e-set-c
## AC-WC-7, -11, -12 at the module's seam).
##
##   godot --headless --path clients/shared --script res://checks/host_check.gd
##
## `MINEWORLD_ADMIN_TOKEN` is set and cleared by the check itself, to a value used nowhere else.

var _fails := 0
var _sent: Array = []


func _init() -> void:
	for problem in MineWorldText.load_layers("", PackedStringArray()):
		push_warning(problem)
	MineWorldText.set_language("en")
	_no_token()
	_pending_and_confirmed()
	_errors()
	OS.set_environment("MINEWORLD_ADMIN_TOKEN", "")
	print("host_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


func _check(ok: bool, what: String) -> void:
	print("[%s] %s" % ["PASS" if ok else "FAIL", what])
	if not ok:
		_fails += 1


func _host(token: String) -> MineWorldHostClock:
	OS.set_environment("MINEWORLD_ADMIN_TOKEN", token)
	var host := MineWorldHostClock.new()
	host.setup("http://127.0.0.1:1")
	get_root().add_child(host)
	host.on_welcome({"at": 0, "time_scale": 12, "paused": false})
	return host


func _no_token() -> void:
	var host := _host("")
	host.set_paused(true)
	host.set_scale(24)
	host.stop_world()
	var requests := 0
	for child in host.get_children():
		requests += 1 if child is HTTPRequest else 0
	_check(not host.has_token() and requests == 0 and not host.is_pending(),
		"AC-WC-11: no token, no request and no HTTPRequest")
	host.free()


func _pending_and_confirmed() -> void:
	var host := _host("check-token-5a1d")
	_sent.clear()
	host.transport = func(method: String, path: String, headers: PackedStringArray, body: String) -> void:
		_sent.append([method, path, headers, body])
	host.set_paused(true)
	_check(_sent.size() == 1 and _sent[0][1] == "/admin/clock" and _sent[0][3] == "{\"paused\":true}"
		and Array(_sent[0][2]).has("Authorization: Bearer check-token-5a1d"), "the pause request and its bearer header")
	host.set_paused(true)
	_check(_sent.size() == 1, "a second command while one is pending is ignored")
	host.answered(200, "{}")
	_check(not host.paused and host.message == "ui.world.pending", "AC-WC-7: a 200 changes no state; pending")
	host.tick(5.1)
	_check(not host.paused and host.message == "ui.world.unconfirmed", "AC-WC-7: no frame in 5 s: unconfirmed")
	host.set_scale(24)
	host.answered(200, "{}")
	host.on_clock(100, 24, false)
	_check(host.time_scale == 24 and host.message == "" and host.day_length() == "1h",
		"the matching clock frame confirms")
	host.set_scale(24)
	host.answered(200, "{}")
	_check(host.message == "" and not host.is_pending(), "a value already in force is confirmed at once")
	host.set_scale(48)
	_check(_sent.size() == 3, "a scale the tab does not offer sends nothing")
	host.free()


func _errors() -> void:
	for pair in [[401, "unauthorized"], [400, "malformed"], [409, "fast_forward_solo_only"],
			[503, "world_stopped"], [0, "unreachable"]]:
		var host := _host("check-token-5a1d")
		host.transport = func(_m: String, _p: String, _h: PackedStringArray, _b: String) -> void: pass
		host.set_paused(true)
		host.answered(pair[0], "")
		_check(host.message == "ui.world.error." + pair[1] and not host.paused and not host.is_pending(),
			"AC-WC-12: %d is worded %s and reverts" % [pair[0], pair[1]])
		_check(MineWorldText.text(host.message) != host.message, "its words exist in the catalog")
		host.free()
	var host := _host("check-token-5a1d")
	host.transport = func(_m: String, _p: String, _h: PackedStringArray, _b: String) -> void: pass
	host.set_paused(true)
	host.tick(5.1)
	_check(host.message == "ui.world.error.timeout", "AC-WC-12: no answer in 5 s is a timeout")
	host.free()

extends Node

## The scripted checks of PR 13e (many players): `--drive=blink` (the socket dropped under a seated
## client) and the lines every drive prints about seats.
##
## Output, one line each, added to the drive's:
## ```text
## JOIN {seat, take_over, t_ms}       the link sends a join (never the invite)
## REFUSED {code, t_ms}               the server refused a frame
## ENDED {code, t_ms}                 the connection ended for good with this code
## SEAT {took_over, observer, name}   the first observation after a welcome: who I now am
## ```

var drive: Node
var app: Node
var _seat_reported := true


func _ready() -> void:
	app.link.joining.connect(func(seat: String, take_over: bool) -> void:
		print("JOIN ", JSON.stringify({"seat": seat, "take_over": take_over, "t_ms": Time.get_ticks_msec()})))
	app.link.refused.connect(func(code: String, _token: String, _detail: String) -> void:
		print("REFUSED ", JSON.stringify({"code": code, "t_ms": Time.get_ticks_msec()})))
	app.link.ended.connect(func(code: String) -> void:
		print("ENDED ", JSON.stringify({"code": code, "t_ms": Time.get_ticks_msec()})))
	app.link.welcomed.connect(func(_o: String, _w: Dictionary) -> void: _seat_reported = false)
	app.link.observed.connect(_on_observed)


func _on_observed(observation: MineWorldObservation) -> void:
	if _seat_reported:
		return
	_seat_reported = true
	print("SEAT ", JSON.stringify({"took_over": app.link.took_over(), "observer": observation.observer(),
		"name": observation.display_name(observation.observer()), "t_ms": Time.get_ticks_msec()}))


## AC-13e-5: the module's own socket closed under a seated client, as a dropped network looks
## (`reconnect_check.gd`). The client must come back as the same Person, welcomed `held`.
func blink() -> void:
	await drive._seconds(drive.SETTLE_S)
	var before: String = app.latest.observer()
	var at := Time.get_ticks_msec()
	print("EVIDENCE ", JSON.stringify({"blink": {"observer": before, "t_ms": at}}))
	app.link.client._socket.close()
	await drive._until(func() -> bool: return app.link.state != "seated", 10.0)
	var back: bool = await drive._until(func() -> bool:
		return app.link.state == "seated" and app.latest != null and app.link.client.latest != null, 40.0)
	await drive._seconds(drive.SETTLE_S)
	var after: String = app.latest.observer() if app.latest != null else ""
	print("EVIDENCE ", JSON.stringify({"blink_back": {"observer": after, "took_over": app.link.took_over(),
		"after_ms": Time.get_ticks_msec() - at}}))
	drive._check(back, "back after the blink", "state %s" % app.link.state)
	drive._check(after == before, "the same Person", "%s → %s" % [before, after])
	drive._check(app.link.took_over() == "held", "welcomed as the held seat", app.link.took_over())

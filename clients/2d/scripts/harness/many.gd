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
## JOIN_SCREEN {visible, seats, answer, join_enabled, take_over, t_ms}   the join screen changed
## BUBBLE / NOTICE {id, event_type, who, text, t_ms}   a fact shown (scene/speech.gd)
## SKIPPED {id, event_type, why, t_ms}                 a fact not shown: unlisted or undrawn
## EVENTS {shown, skipped}                             totals, at the end of `--drive=listen`
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
	app.join_screen.changed.connect(func() -> void: _report_screen(app.join_screen))
	for pair in [[app.speech.bubbled, "BUBBLE "], [app.speech.noticed, "NOTICE "], [app.speech.skipped, "SKIPPED "]]:
		var tag: String = pair[1]
		(pair[0] as Signal).connect(func(record: Dictionary) -> void:
			var line := record.duplicate()
			line["t_ms"] = Time.get_ticks_msec()
			line["unix_ms"] = unix_ms()
			print(tag, JSON.stringify(line)))
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


## AC-13e-1 … AC-13e-4: the client started without `--seat` operates its own join screen, the path a
## player's clicks take ([method choose]), then stays `--hold=<s>` seconds seated (default 3).
func join() -> void:
	await choose()
	if app.link.is_seated():
		await drive._seconds(float(app.options.get("hold", "3")))
		drive._report_shown()
	print("EVIDENCE ", JSON.stringify({"drawn": {"people": app.people.drawn_ids().size(),
		"places": app.places.drawn.size(), "seated": app.link.is_seated()}}))


## Waits for the seats `/status` lists; `--invite-override=<text>` types another invite;
## `--pick=<index>` (default 0) or `--pick-seat=<key>` chooses; Join is pressed; after `seat_occupied`,
## `--take-over` presses Take over (after `--take-over-after=<s>`). Every other scenario started
## without `--seat` goes through this first, so a scripted run needs no seat named.
func choose() -> void:
	var screen: CanvasLayer = app.join_screen
	var listed: bool = await drive._until(func() -> bool:
		return not screen.seats.is_empty() or screen.answer_code == "unreachable", 30.0)
	if not listed or screen.seats.is_empty():
		drive._check(false, "the join screen lists seats", "answer %s" % screen.answer_code)
		return
	if app.options.has("invite-override"):
		screen.type_invite(String(app.options["invite-override"]))
	var index := int(app.options.get("pick", "0"))
	if app.options.has("pick-seat"):
		index = screen.seats.find(String(app.options["pick-seat"]))
	screen.pick(index)
	screen.press_join()
	await _until_answered(screen)
	if screen.visible and screen.answer_code == screen.OCCUPIED and app.options.has("take-over"):
		await drive._seconds(float(app.options.get("take-over-after", "0")))
		await drive._until(func() -> bool: return screen.take_over_offered(), 10.0)
		screen.press_take_over()
		await _until_answered(screen)


## Until the join was answered: seated and observed, or the screen is back with an answer.
func _until_answered(screen: CanvasLayer) -> void:
	await drive._seconds(0.2)
	await drive._until(func() -> bool:
		return (app.link.is_seated() and app.walker.placed) or (screen.visible and screen.answer_code != ""),
		30.0)


func _report_screen(screen: CanvasLayer) -> void:
	print("JOIN_SCREEN ", JSON.stringify({"visible": screen.visible, "seats": screen.seats,
		"answer": screen.answer_code, "join_enabled": screen.join_enabled(),
		"take_over": screen.take_over_offered(), "t_ms": Time.get_ticks_msec()}))


## AC-13e-6, AC-13e-7: seated, the client listens for `--hold=<s>` seconds (default 20) while the
## BUBBLE, NOTICE and SKIPPED lines are printed, then reports the totals and the span of world time it
## observed, which the test restricts the server's perceived export to.
func listen() -> void:
	var first_at: int = app.latest.at()
	await drive._seconds(float(app.options.get("hold", "20")))
	print("EVIDENCE ", JSON.stringify({"listened": {"observer": app.latest.observer(), "first_at": first_at,
		"last_at": app.latest.at()}}))
	print("EVENTS ", JSON.stringify({"shown": app.speech.shown, "skipped": app.speech.skipped_count}))
	drive._report_shown()


## Wall-clock milliseconds, so two client processes' lines can be compared (their `t_ms` count from
## each process's own start).
static func unix_ms() -> int:
	return int(Time.get_unix_time_from_system() * 1000.0)


## AC-13e-8: a living town. Seated, report what is drawn every 5 s for `--hold=<s>` seconds (default
## 120), so the test sees the town's people move without this player; then AC-W2's check that every
## person is drawn where the server says.
func town() -> void:
	var until: float = drive._now() + float(app.options.get("hold", "120"))
	while drive._now() < until:
		drive._report_shown()
		await drive._seconds(5.0)
	drive._check_seated()

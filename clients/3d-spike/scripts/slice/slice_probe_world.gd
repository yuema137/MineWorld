## The slice's evidence harness for the modes that need a world: the slice
## connected to a real `mineworld server`, exactly as `./mineworld-slice --world`
## starts it (`ENGINEERING_RULES.md` sec.19).
##
##   --slice-link          the round trip: walk out, jog, jump, walk in, talk from
##                         the door and at the counter, watch the street
##   --slice-conversation  the same talk, captured from the player's own HUD
##
## It extends `SliceProbe` for its walking, capture and timing helpers, and adds
## nothing to it: the standalone modes stay there and these stay here, so S14's
## connected checks grow in a file of their own (step-15 F-S14-12, §19.4
## D-16a-5).
class_name SliceProbeWorld
extends SliceProbe

## The connected modes.
const WORLD_MODES := ["link", "conversation"]


static func requested() -> bool:
	return requested_of(WORLD_MODES) != ""


## A scripted mode whose evidence is the player's own HUD, so the slice attaches
## it exactly as for a player (`SliceMain._hud`) rather than the probe making a
## copy of its wiring.
static func with_hud() -> bool:
	return requested_of(["conversation"]) != ""


func _modes() -> Array:
	return WORLD_MODES


func _run(mode: String) -> void:
	match mode:
		"link": await _link_check()
		"conversation": await _conversation_frames()
		_: await super(mode)


# --- the connected modes, moved unchanged from slice_probe.gd -----------------



## C8, with S6's `move`: the slice against a real `mineworld server
## worlds/social-cafe`, walked on the real controller, never teleported.
## Pass: seated; the body placed where the world says; out through the café door
## onto the street, a jog along the pavement, two jumps, and back in -- with the
## server's place changing cafe -> street -> cafe, no move refused, and the
## server's last view of the player equal to the last report, millimetre for
## millimetre; then a `talk` answered by the server.
func _link_check() -> void:
	print("== VISUAL_SLICE.md sec.9 -- the slice as a MineWorld presentation ==\n")
	var link := slice.link
	var fails := 0
	if link == null:
		print("FAIL: no --server= given")
		return
	var waited := 0.0
	while waited < 10.0 and (not link.client.is_seated() or link.here_key == "" or link._reconcile):
		await get_tree().process_frame
		waited += get_process_delta_time()
	if not link.client.is_seated():
		print("FAIL: not seated after 10 s")
		return
	print("seated   observer %s in place %s (%s); places known %s" % [link.client.observer,
		link.client.latest.place(), link.here_key, link.place_ids])
	print("body     at %s -- placed where the world says" % player.global_position)
	await _hold(0.5)

	# perceived people, drawn where the world says. Inside the slice's room or
	# not is reported, not failed: the binding aligns the pack's doorway with the
	# slice's, and the pack's café is authored with its people west of where the
	# slice's room begins (slice_link.gd, design sec.7b).
	for id in link.figures:
		var fp: Vector3 = (link.figures[id] as Node3D).global_position
		var inside := _inside_room(fp + Vector3(0, 0.5, 0))
		print("person   %s at %s  %s" % [id, fp, "inside the room" if inside
			else "outside the slice's room (recorded binding discrepancy)"])
	if link.figures.size() < 2:
		fails += 1
		print("FAIL: expected at least two other people perceived, got %d" % link.figures.size())

	# OUT: to the door's centre line, then south through it onto the pavement
	var door_x := 6.0 + SliceCafe.DOOR_X
	await _walk_to_x(door_x)
	player.rotation.y = PI
	await _walk_dist(3.2, 6.0)
	await _hold(0.8)
	print("out      body at %s, slice place %s, server place %s" % [player.global_position,
		SliceWorld.place_at(slice.world, player.global_position), link.here_key])
	if link.here_key != "street":
		fails += 1
		print("FAIL: the server did not move the player onto the street")

	# JOG east along the pavement, and back; then jump on the spot and running
	Input.action_press("jog")
	player.rotation.y = -PI * 0.5
	await _walk_dist(6.0, 6.0)
	print("jog      body at %s after jogging east" % player.global_position)
	player.rotation.y = PI * 0.5
	await _walk_dist(6.0, 6.0)
	Input.action_release("jog")
	var j0 := player.jumps
	Input.action_press("jump")
	await _hold(0.05)
	Input.action_release("jump")
	await _hold(1.0)
	player.rotation.y = -PI * 0.5
	Input.action_press("move_forward")
	await _hold(0.3)
	Input.action_press("jump")
	await _hold(0.05)
	Input.action_release("jump")
	await _hold(0.8)
	Input.action_release("move_forward")
	await _hold(0.6)
	print("jump     %d jumps made (one standing, one running)" % (player.jumps - j0))
	if player.jumps - j0 < 2:
		fails += 1
		print("FAIL: the jumps did not happen")

	# IN: back to the door's centre line on the pavement, then north through it
	await _walk_to_x(door_x)
	player.rotation.y = 0.0
	await _walk_dist(4.2, 6.0)
	await _hold(0.8)
	var tok := link.report_position()
	var sent: Dictionary = link.last_sent_local.duplicate()
	var t := 0.0
	while t < 3.0 and not _answered(link, tok):
		await get_tree().process_frame
		t += get_process_delta_time()
	await _hold(0.6)   # the next observation after the answer
	print("in       body at %s, server place %s" % [player.global_position, link.here_key])
	if link.here_key != "cafe":
		fails += 1
		print("FAIL: the server did not move the player back into the café")

	print("places   the server's place, as observed: %s" % [link.place_changes])
	var moves := link.answers.filter(func(a): return a["action"] == SliceLink.MOVE_ACTION)
	var accepted := moves.filter(func(a): return a["result"] == "accepted").size()
	var refusals := moves.filter(func(a): return a["result"] != "accepted")
	print("move     %d reports, %d accepted, %d not" % [moves.size(), accepted, refusals.size()])
	for r in refusals:
		print("         NOT ACCEPTED: %s" % JSON.stringify(r))
	# How many facts the server stated per accepted move, from its own answers.
	# The answer names facts by EventId only (protocol revision 1 shows no event
	# bodies), so this counts them: a stride states presence's Arrived; a
	# crossing also states PersonEnteredPlace (`ARC-26`).
	var per_move := {}
	for m in moves:
		var d: Variant = m.get("detail")
		if typeof(d) != TYPE_DICTIONARY or not (d as Dictionary).has("accepted"):
			continue
		var n: int = (d["accepted"].get("events", []) as Array).size()
		per_move[n] = int(per_move.get(n, 0)) + 1
		if n > 1:
			print("facts    %d facts for the move to %s" % [n, JSON.stringify(m.get("sent"))])
	print("facts    moves by facts stated: %s" % [per_move])
	if accepted < 1 or not refusals.is_empty():
		fails += 1
		print("FAIL: every move during normal walking, jogging and jumping must be accepted")
	var seen: Variant = link.client.latest.self_location().get("local")
	print("position sent %s, server now says %s" % [sent, seen])
	if typeof(seen) != TYPE_DICTIONARY or int(seen.get("x", -1)) != int(sent["x"]) \
			or int(seen.get("y", -1)) != int(sent["y"]):
		fails += 1
		print("FAIL: the server's view of the player is not where the player reported")

	# TALK to whoever the world tags as the barista, by turning to face them:
	# first from just inside the door, where the world must refuse it, then from
	# the counter, reached on foot, where it must be accepted and answered.
	var barista := ""
	for id in link.client.latest.tagged("barista"):
		barista = id
	if barista == "" or not link.figures.has(barista):
		fails += 1
		print("FAIL: no barista perceived")
	else:
		var near_door := await _talk_to(link, barista)
		print("talk     from the door, %.2f m from %s -> %s" % [near_door[1], barista, near_door[0]])
		if near_door[0] != "rejected too_far_away" and near_door[0] != "refused too_far_away":
			fails += 1
			print("FAIL: talking from the door must be refused too_far_away")
		# to the counter on foot: east along the room's clear lane, then north to it
		var moves_before := link.answers.filter(
			func(a): return a["action"] == SliceLink.MOVE_ACTION).size()
		await _walk_to(Vector3(8.16, 0.0, -10.20), 6.0)
		var bp: Vector3 = (link.figures[barista] as Node3D).global_position
		await _walk_to(Vector3(bp.x, 0.0, bp.z + 1.85), 6.0)
		await _hold(0.8)
		var walked := link.answers.filter(func(a): return a["action"] == SliceLink.MOVE_ACTION)
		var walked_bad := walked.slice(moves_before).filter(func(a): return a["result"] != "accepted")
		print("counter  body at %s; %d moves on the way, %d not accepted"
			% [player.global_position, walked.size() - moves_before, walked_bad.size()])
		var heard_before := link.heard.size()
		var at_counter := await _talk_to(link, barista)
		print("talk     from the counter, %.2f m from %s -> %s" % [at_counter[1], barista, at_counter[0]])
		if not String(at_counter[0]).begins_with("accepted"):
			fails += 1
			print("FAIL: talking from the counter must be accepted")
		# the reply: what the barista said to this observer, as disclosed to it
		t = 0.0
		var reply := {}
		while t < 15.0 and reply.is_empty():
			for e in link.heard.slice(heard_before):
				if typeof(e.get("speaker")) == TYPE_DICTIONARY \
						and String(e["speaker"].get("entity", "")) == barista:
					reply = e
			await get_tree().process_frame
			t += get_process_delta_time()
		if reply.is_empty():
			fails += 1
			print("FAIL: no reply from %s within 15 s" % barista)
		else:
			print("reply    %s said %s (after %.1f s)" % [barista, JSON.stringify(reply.get("utterance")), t])
	await _street_watch(link)
	print("\n%s" % ("all link checks pass" if fails == 0 else "%d LINK CHECKS FAILED" % fails))
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)


## What the player reads when they talk, captured from the player's own HUD in a
## window: from the door (the world says too far away), then at the counter on
## foot (her answer, as a line of conversation). The probe's `--link` check saw
## the reply in data; this is the screen the operator reads it on.
func _conversation_frames() -> void:
	print("== the conversation, on screen ==\n")
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	var waited := 0.0
	while waited < 10.0 and (not link.client.is_seated() or link.here_key == "" or link._reconcile):
		await get_tree().process_frame
		waited += get_process_delta_time()
	await _hold(0.8)
	var barista := ""
	for id in link.client.latest.tagged("barista"):
		barista = id
	if barista == "" or not link.figures.has(barista):
		print("FAIL: no barista perceived")
		return
	player.set_camera(FP)
	# where everyone is, so a frame that differs from the last run can be traced
	# to who moved (`ARC-23`)
	for id in link.figures:
		print("person   %s (%s) at %s" % [id, link.display_label(id),
			(link.figures[id] as Node3D).global_position])
	print("body     at %s" % player.global_position)
	var near_door := await _talk_to(link, barista)
	await _settle(4)
	await _save("conversation_1_from_door")
	print("door     %.2f m -> %s; on screen: %s" % [near_door[1], near_door[0],
		slice.hud._toast.text])
	await _walk_to(Vector3(8.16, 0.0, -10.20), 6.0)
	var bp: Vector3 = (link.figures[barista] as Node3D).global_position
	await _walk_to(Vector3(bp.x, 0.0, bp.z + 1.85), 6.0)
	await _hold(0.8)
	print("counter  body at %s, %s at %s" % [player.global_position, link.display_label(barista),
		(link.figures[barista] as Node3D).global_position])
	var heard_before := link.heard.size()
	var t0 := Time.get_ticks_msec()
	var at_counter := await _talk_to(link, barista)
	var t := 0.0
	while t < 15.0 and link.heard.size() == heard_before:
		await get_tree().process_frame
		t += get_process_delta_time()
	var t_reply := Time.get_ticks_msec()
	# straight away: an unfocused capture window draws about a frame a second,
	# and the captions' hold is real seconds
	await _save("conversation_2_at_counter")
	var shown := slice.hud._caption.text if slice.hud._caption != null else ""
	print("counter  %.2f m -> %s; reply on screen %.1f s after pressing talk, captured %.1f s "
		% [at_counter[1], at_counter[0], (t_reply - t0) / 1000.0,
		(Time.get_ticks_msec() - t_reply) / 1000.0] + "after it; on screen:\n%s" % shown)
	# The same moment from beside the counter: the player's character and the
	# person she is talking to in one frame, the lines still up. A fixed camera,
	# because the rear boom puts her body between the camera and the barista.
	var ap: Vector3 = (link.figures[barista] as Node3D).global_position
	var mid := (player.global_position + ap) * 0.5 + Vector3(0, 1.15, 0)
	var cam := Camera3D.new()
	cam.fov = 60.0
	slice.add_child(cam)
	# from the east end of the counter: west of it stand Bob and the pastry case
	cam.look_at_from_position(mid + Vector3(1.85, 0.55, 1.45), mid)
	cam.make_current()
	# draw the body without a mode switch, whose toast would cover the caption
	player.body.visible = true
	await _save("conversation_3_at_counter_side")
	player.body.visible = false
	print("side     fixed camera at %s, the caption then: %d lines"
		% [cam.global_position, slice.hud._caption.text.count("\n") + 1])
	cam.queue_free()
	# no entity id on screen: the barista's id must not appear as a word
	var leaked := false
	for w in shown.split(" "):
		if w.strip_edges().trim_suffix(":") == barista or w.strip_edges() == link.client.observer:
			leaked = true
	print("\n%s" % ("conversation on screen, no ids" if not shown.is_empty() and not leaked
		else "CONVERSATION CHECK FAILED (empty caption or an id on screen)"))
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)


## The town's other people walk the street between places this slice does not
## draw. Reported, not failed: where each street doorway the world discloses
## lands in this scene, and where figures appear and are lost while standing on
## the pavement. `--watch=<s>` sets how long (default 60).
func _street_watch(link: SliceLink) -> void:
	var secs := 60.0
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--watch="):
			secs = float(a.substr(8))
	await _walk_to(Vector3(8.16, 0.0, -10.20), 6.0)
	await _walk_to(Vector3(6.0 + SliceCafe.DOOR_X, 0.0, -9.6), 6.0)
	player.rotation.y = PI
	await _walk_dist(3.6, 6.0)
	await _hold(1.0)
	print("\n-- the street's people, %.0f s on the pavement at %s, server place %s --"
		% [secs, player.global_position, link.here_key])
	var obs := link.client.latest
	var leads: Variant = obs.component(obs.place(), "passages").get("leads_to", [])
	if typeof(leads) == TYPE_ARRAY:
		for p in leads:
			var to := String(p.get("to", {}).get("entity", ""))
			var at := link.to_scene("street", p.get("here"))
			var near := ""
			var best := INF
			for d in SliceTerrace.doors:
				var dd := Vector2(at.x - d[0].x, at.z - d[0].z).length()
				if dd < best:
					best = dd
					near = "%s at %s" % [d[2], d[0]]
			print("doorway  to place %s: street frame %s -> scene %s; nearest drawn door %.2f m: %s"
				% [to, JSON.stringify(p.get("here")), at, best, near])
	var first := {}
	for id in link.figures:
		first[id] = (link.figures[id] as Node3D).global_position
	print("start    %d figures on the street: %s" % [first.size(), first])
	var wall0 := Time.get_ticks_msec()
	var t := 0.0
	while t < secs:
		await _hold(1.0)
		t += 1.0
	# simulated seconds against wall seconds: a slow headless loop shows here,
	# not as a silent stall
	print("watch    %.0f s simulated in %.1f s wall" % [secs, (Time.get_ticks_msec() - wall0) / 1000.0])
	var last := {}
	for id in link.figures:
		last[id] = (link.figures[id] as Node3D).global_position
	print("end      %d figures on the street: %s" % [last.size(), last])


## Face a perceived person and press talk, as a player would. Returns
## ["<result> <code>", distance in metres] -- the code is the server's.
func _talk_to(link: SliceLink, id: String) -> Array:
	var to: Vector3 = (link.figures[id] as Node3D).global_position - player.global_position
	player.rotation.y = atan2(-to.x, -to.z)
	player.rig.pitch = 0.0
	await _hold(0.3)
	var tok := link.talk_to_facing("Hello! A coffee, please.")
	var t := 0.0
	while t < 4.0 and tok != "" and not _answered(link, tok):
		await get_tree().process_frame
		t += get_process_delta_time()
	var dist := Vector2(to.x, to.z).length()
	var ans := link.answers.filter(func(a): return a["token"] == tok)
	if ans.is_empty():
		return ["NO ANSWER", dist]
	var a: Dictionary = ans[0]
	var code := ""
	if a.has("code"):
		code = String(a["code"])
	elif typeof(a.get("detail")) == TYPE_DICTIONARY:
		var v: Variant = (a["detail"] as Dictionary).get(a["result"])
		code = String(v) if typeof(v) == TYPE_STRING else ""
	print("         %s" % JSON.stringify(a.get("detail", a)))
	return ["%s %s" % [a["result"], code], dist]


## Turn toward a floor point and walk until within 0.15 m of it, or `max_secs`.
func _walk_to(p: Vector3, max_secs: float) -> void:
	var d := p - player.global_position
	d.y = 0.0
	player.rotation.y = atan2(-d.x, -d.z)
	await _walk_dist(maxf(d.length() - 0.15, 0.0), max_secs)


func _answered(link: SliceLink, tok: String) -> bool:
	for a in link.answers:
		if a["token"] == tok:
			return true
	return false

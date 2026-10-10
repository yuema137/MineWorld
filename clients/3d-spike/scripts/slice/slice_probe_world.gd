## The slice's evidence harness for the modes that need a world: the slice
## connected to a real `mineworld server`, exactly as `./mineworld-slice --world`
## starts it (`ENGINEERING_RULES.md` sec.19).
##
##   --slice-link          the round trip: walk out, jog, jump, walk in, talk from
##                         the door and at the counter, watch the street
##   --slice-conversation  the same talk, captured from the player's own HUD
##   --slice-target        what the targeting ray picks, and what it must not:
##                         nobody through a wall (step-15 §19.3 T-1 ... T-4)
##
## It extends `SliceProbe` for its walking, capture and timing helpers, and adds
## nothing to it: the standalone modes stay there and these stay here, so S14's
## connected checks grow in a file of their own (step-15 F-S14-12, §19.4
## D-16a-5).
class_name SliceProbeWorld
extends SliceProbe

## The connected modes.
const WORLD_MODES := ["link", "conversation", "target", "settings"]
## Where on a perceived figure the probe aims, as a player aims at a face.
const HEAD_Y := 1.55
## The targeting rule before PR 16a (`slice_link.gd` at 47c81d1): the figure
## whose head lies within this cosine of the view's centre. Computed here only
## to show that the wall case would have gone the other way under it.
const CONE_COS := 0.80


static func requested() -> bool:
	return requested_of(WORLD_MODES) != ""


## A scripted mode whose evidence is the player's own HUD, so the slice attaches
## it exactly as for a player (`SliceMain._hud`) rather than the probe making a
## copy of its wiring.
static func with_hud() -> bool:
	return requested_of(["conversation", "settings"]) != ""


func _modes() -> Array:
	return WORLD_MODES


func _run(mode: String) -> void:
	match mode:
		"link": await _link_check()
		"conversation": await _conversation_frames()
		"target": await _target_check()
		"settings": await _settings_check()
		_: await super(mode)


# --- targeting (PR 16a) ---------------------------------------------------------

## Turn the body and pitch the view so the active camera's centre ray passes
## through `point`. The third-person cameras sit off the body, so the aim is
## refined over a few frames from where the camera actually is.
func _aim_at(point: Vector3) -> void:
	for i in range(4):
		var cam := player.get_viewport().get_camera_3d()
		var from := cam.global_position if cam != null else player.global_position
		var d := point - from
		player.rotation.y = atan2(-d.x, -d.z)
		player.rig.pitch = atan2(d.y, Vector2(d.x, d.z).length())
		await _hold(0.1)
	await _hold(0.2)


## What the ray says now, printed; returns the aim.
func _report_aim(link: SliceLink, what: String) -> Dictionary:
	var aim := SliceTargeting.aim(player.get_viewport())
	var who := String(aim["entity"])
	print("aim      %-34s -> %s  [first hit %s at %s]" % [what,
		("%s (%s)" % [link.display_label(who), who]) if who != "" else "nobody",
		aim["collider"] if aim["collider"] != "" else "-", aim["point"]])
	return aim


## The pre-16a cone's choice from the active camera (CONE_COS): the counterfactual.
func _cone_choice(link: SliceLink) -> String:
	var cam := player.get_viewport().get_camera_3d()
	var fwd := -cam.global_transform.basis.z
	var best := ""
	var best_dot := CONE_COS
	for id in link.figures:
		var to: Vector3 = (link.figures[id] as Node3D).global_position + Vector3(0, 1.4, 0) \
			- cam.global_position
		var d := fwd.dot(to.normalized())
		if d > best_dot:
			best_dot = d
			best = id
	return best


## How far each other figure's axis passes from the line of sight to `target`'s
## head, horizontally, and the capsule's half-width at the height the line
## crosses it: under that half-width the figure is in the way (R-16a-2).
func _clearances(link: SliceLink, target: String) -> void:
	var cam := player.get_viewport().get_camera_3d().global_position
	var head: Vector3 = (link.figures[target] as Node3D).global_position + Vector3.UP * HEAD_Y
	var dir := head - cam
	var flat := Vector2(dir.x, dir.z)
	for id in link.figures:
		if id == target:
			continue
		var p: Vector3 = (link.figures[id] as Node3D).global_position
		var rel := Vector2(p.x - cam.x, p.z - cam.z)
		var along := rel.dot(flat.normalized())
		if along <= 0.0 or along >= flat.length():
			continue
		var off := absf(rel.x * flat.normalized().y - rel.y * flat.normalized().x)
		var y := cam.y + dir.y * along / flat.length() - p.y
		var r := Player.CAPSULE_RADIUS
		var half := r
		var top := Player.CAPSULE_HEIGHT - r
		if y > top:
			half = sqrt(maxf(r * r - (y - top) * (y - top), 0.0))
		print("clear    %s (%s) beside the line to %s: axis %.3f m from it, capsule half-width %.3f m at y %.2f -> clearance %+.3f m"
			% [link.display_label(id), id, link.display_label(target), off, half, y, off - half])


## T-1 ... T-4 (step-15 §19.3): the ray targets what is visible and only that.
## Walked on the real controller against the real server, never teleported.
func _target_check() -> void:
	print("== step-15 §19.3 -- targeting is a ray ==\n")
	var link := slice.link
	var fails := 0
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
	var alice: Vector3 = (link.figures[barista] as Node3D).global_position

	# T-1: from the seat by the door, aimed at her head
	await _aim_at(alice + Vector3.UP * HEAD_Y)
	_clearances(link, barista)
	var a := _report_aim(link, "from the door, at her head")
	if a["entity"] != barista:
		fails += 1
		print("FAIL T-1: from the door the ray must meet the barista")
	var door := await _talk_to(link, barista)
	print("talk     from the door, %.2f m -> %s" % [door[1], door[0]])
	if door[0] != "rejected too_far_away" and door[0] != "refused too_far_away":
		fails += 1
		print("FAIL T-1: the talk is sent and the world answers too_far_away")
	# the counter as an occluder: aimed at her knees, the counter is in the way
	await _aim_at(alice + Vector3.UP * 0.5)
	a = _report_aim(link, "from the door, at her knees")
	if a["entity"] == barista:
		fails += 1
		print("FAIL T-1: the counter stands between the door and her knees")
	# at the counter, first person and third person rear
	await _walk_to(Vector3(8.16, 0.0, -10.20), 6.0)
	await _walk_to(Vector3(alice.x, 0.0, alice.z + 1.85), 6.0)
	await _hold(0.6)
	for m in [FP, REAR]:
		player.set_camera(m)
		await _aim_at(alice + Vector3.UP * HEAD_Y)
		a = _report_aim(link, "at the counter, %s" % player.rig.mode_name())
		if a["entity"] != barista:
			fails += 1
			print("FAIL T-1: at the counter the ray must meet the barista")
	player.set_camera(FP)

	# T-4: nothing -- aimed at the ceiling, E sends nothing
	await _aim_at(player.global_position + Vector3(0, 6.0, -0.5))
	a = _report_aim(link, "at the ceiling")
	var sent_before := link.answers.size()
	var tok := link.talk_to_facing(SliceIntents.DEFAULT_UTTERANCE)
	await _hold(1.0)
	print("talk     aimed at nothing -> token '%s', %d answers arrived" % [tok,
		link.answers.size() - sent_before])
	if a["entity"] != "" or tok != "" or link.answers.size() != sent_before:
		fails += 1
		print("FAIL T-4: aimed at nothing, nobody is targeted and nothing is sent")

	# out onto the street
	await _walk_to(Vector3(8.16, 0.0, -10.20), 6.0)
	await _walk_to(Vector3(6.0 + SliceCafe.DOOR_X, 0.0, -9.6), 6.0)
	player.rotation.y = PI
	await _walk_dist(3.6, 6.0)
	await _hold(1.0)
	print("street   body at %s, server place %s, %d people perceived" % [player.global_position,
		link.here_key, link.figures.size()])
	var passer := ""
	for id in link.figures:
		passer = id
	if link.here_key != "street" or passer == "":
		print("FAIL: no person perceived on the street")
		print("\n%d TARGET CHECKS FAILED" % (fails + 1))
		return
	var pp: Vector3 = (link.figures[passer] as Node3D).global_position

	# T-3: the positive control, from the pavement with a clear line
	await _aim_at(pp + Vector3.UP * HEAD_Y)
	a = _report_aim(link, "from the pavement")
	if a["entity"] != passer:
		fails += 1
		print("FAIL T-3: from the pavement the ray must meet %s" % passer)

	# T-2: into The Flower Room (reported to the world as the street, F-S14-9),
	# to the back of its west lane, and aimed at the same person through its wall
	await _walk_to(Vector3(11.84, 0.0, -5.40), 8.0)
	await _walk_to(Vector3(11.84, 0.0, -9.50), 6.0)
	await _walk_to(Vector3(11.84, 0.0, -13.20), 6.0)
	await _hold(1.0)
	print("florist  body at %s, slice place %s, server place %s, %s perceived: %s"
		% [player.global_position, SliceWorld.place_at(slice.world, player.global_position),
		link.here_key, passer, link.figures.has(passer)])
	if not link.figures.has(passer):
		fails += 1
		print("FAIL T-2: %s is no longer perceived, so the case would be vacuous" % passer)
	else:
		pp = (link.figures[passer] as Node3D).global_position
		await _aim_at(pp + Vector3.UP * HEAD_Y)
		a = _report_aim(link, "from the florist, through its wall")
		var cone := _cone_choice(link)
		print("cone     the pre-16a rule would have chosen: %s" % (cone if cone != "" else "nobody"))
		var p: Variant = a["point"]
		var inside := typeof(p) == TYPE_VECTOR3 and (p as Vector3).z < SliceStreet.NORTH_FACE - 0.5
		if a["entity"] != "":
			fails += 1
			print("FAIL T-2: the ray targeted someone through the florist's wall")
		if not inside:
			fails += 1
			print("FAIL T-2: the first hit must be a wall inside the building, not the frontage")
		if cone != passer:
			fails += 1
			print("FAIL T-2: the case does not discriminate -- the cone would not have chosen %s"
				% passer)
	print("\n%s" % ("all target checks pass" if fails == 0 else "%d TARGET CHECKS FAILED" % fails))
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)


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
	# aim at the head, as a player would: targeting is a ray (step-15 §19)
	await _aim_at((link.figures[id] as Node3D).global_position + Vector3.UP * HEAD_Y)
	var tok := link.talk_to_facing(SliceIntents.DEFAULT_UTTERANCE)
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


# --- the settings menu (S20 SET-a) --------------------------------------------------

const MARKER := "qaa"
var _set_fails := 0


func _set_check(ok: bool, what: String, detail := "") -> void:
	if not ok:
		_set_fails += 1
	print("[%s] %s  %s" % ["PASS" if ok else "FAIL", what, detail])


## step-20 §7 in the connected slice: the settings in effect at the first frame (AC-SET-7), the
## render scale (AC-SET-11), two deterministic requests for the transcript (AC-SET-5), every UI text
## turning to the other language at once and back (AC-SET-1), the marker catalog (AC-SET-2, with
## `--marker`), and an open menu taking walking, looking and E (AC-SET-12).
func _settings_check() -> void:
	print("== S20 SET-a -- the settings menu in the connected slice ==\n")
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	link.client.submitted_request.connect(func(token: String, request: Dictionary) -> void:
		print("REQUEST ", JSON.stringify({"token": token, "request": request})))
	var waited := 0.0
	while waited < 10.0 and (not link.client.is_seated() or link.here_key == "" or link._reconcile):
		await get_tree().process_frame
		waited += get_process_delta_time()
	_set_check(link.client.is_seated(), "seated")
	print("EVIDENCE ", JSON.stringify({"first_frame": slice.first_frame}))
	var window := get_window()
	print("EVIDENCE ", JSON.stringify({"display": {"render_scale": window.scaling_3d_scale,
		"window": [window.size.x, window.size.y], "mode": window.mode}}))
	await _hold(0.5)
	# Two requests whose content does not depend on timing: the body where the world placed it, and
	# a line to whoever is perceived first, by id.
	link.report_position()
	var someone := ""
	for id in link.figures:
		someone = id if someone == "" or id < someone else someone
	if someone != "":
		link.intents.talk(link.client, someone, SliceIntents.DEFAULT_UTTERANCE,
			link._location(link.here_key, link.to_world(link.here_key, player.global_position)))
	# Long enough for an answer's toast to fade (ControlsHud: 1.6 s held, 0.6 s fading).
	await _hold(3.0)
	var menu: MineWorldSettingsMenu = slice.settings_menu
	menu.open()
	await _settle(3)
	var start := MineWorldText.language()
	var other := "en" if start == "zh_Hans" else "zh_Hans"
	var first := _ui_texts()
	await _settings_still("settings_%s_general" % start)
	MineWorldText.set_language(other)
	await _settle(3)
	var second := _ui_texts()
	await _settings_still("settings_%s_general" % other)
	menu._tabs.current_tab = 1
	await _settings_still("settings_%s_display" % other)
	menu._tabs.current_tab = 0
	# The HUD alone, in the same language: the menu's layer hidden, not closed (Close drops an
	# unapplied language, by design).
	menu.visible = false
	await _settings_still("hud_%s" % other)
	menu.visible = true
	await _settle(2)
	print("EVIDENCE ", JSON.stringify({"texts": {start: first.size(), other: second.size()}}))
	_compare_texts(first, second, start, other)
	MineWorldText.set_language(start)
	await _settle(3)
	_set_check(_masked(_ui_texts()) == _masked(first), "switching back reproduces every text exactly")
	if "--marker" in OS.get_cmdline_user_args():
		await _marker_texts()
	await _menu_takes_input(link)
	print("\n%s" % ("all settings checks pass" if _set_fails == 0 else "%d SETTINGS CHECKS FAILED" % _set_fails))
	link.client.leave_world()
	await _hold(0.5)


## A still, with `--stills=<dir>` (windowed): the menu or the HUD, for the operator's review (JPEG).
func _settings_still(name: String) -> void:
	var dir := ""
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--stills="):
			dir = arg.substr(9)
	if dir == "":
		return
	await _settle(4)
	await RenderingServer.frame_post_draw
	DirAccess.make_dir_recursive_absolute(dir)
	var path := dir.path_join(name + ".jpg")
	var saved := get_viewport().get_texture().get_image().save_jpg(path, 0.85)
	print("EVIDENCE ", JSON.stringify({"still": path, "saved": saved == OK}))


func _compare_texts(first: PackedStringArray, second: PackedStringArray, a: String, b: String) -> void:
	var by_path := {}
	for line in second:
		by_path[line.get_slice("\t", 0)] = line.get_slice("\t", 1)
	var wrong := PackedStringArray()
	for line in first:
		var path := line.get_slice("\t", 0)
		var before := line.get_slice("\t", 1)
		var after: String = by_path.get(path, "")
		print("EVIDENCE ", JSON.stringify({"text": {"path": path, a: before, b: after}}))
		var chinese := after if b == "zh_Hans" else before
		if after == before or not MineWorldText.has_cjk(chinese):
			wrong.append("%s: \"%s\" → \"%s\"" % [path, before, after])
	_set_check(first.size() >= 30 and first.size() == second.size() and wrong.is_empty(),
		"every UI text turns from %s to %s at once" % [a, b], "%d texts; %s" % [first.size(), " | ".join(wrong)])


func _marker_texts() -> void:
	var english := TranslationServer.get_translation_object("en")
	var marked := Translation.new()
	marked.locale = MARKER
	for key in english.get_message_list():
		if key != "":
			marked.add_message(key, "⟦%s⟧" % String(english.get_message(key)))
	TranslationServer.add_translation(marked)
	var start := MineWorldText.language()
	MineWorldText.set_language(MARKER)
	await _settle(3)
	var unmarked := PackedStringArray()
	for line in _ui_texts():
		if not line.contains("⟦"):
			unmarked.append(line)
	_set_check(unmarked.is_empty(), "under the marker catalog every UI text comes from a catalog", " | ".join(unmarked))
	MineWorldText.set_language(start)
	TranslationServer.remove_translation(marked)
	await _settle(2)


## AC-SET-12 as a player meets it: looking and walking enabled as in play, then the menu opens; two
## seconds of W, mouse motion and E ask for nothing and do not turn the camera.
func _menu_takes_input(link: SliceLink) -> void:
	var menu: MineWorldSettingsMenu = slice.settings_menu
	menu.close()
	player.scripted_look = false
	player.ignore_mouse_look = false
	player.look_enabled = true
	menu.open()
	await _settle(2)
	var asked := [0]
	var count := func(_t: String, _r: Dictionary) -> void: asked[0] += 1
	link.client.submitted_request.connect(count)
	var yaw := player.rotation.y
	var pitch: float = player.rig.pitch
	var at := player.global_position
	Input.action_press("move_forward")
	for i in 20:
		var motion := InputEventMouseMotion.new()
		motion.relative = Vector2(40, 12)
		Input.parse_input_event(motion)
		await _hold(0.1)
	Input.action_release("move_forward")
	var press := InputEventKey.new()
	press.physical_keycode = KEY_E
	press.pressed = true
	Input.parse_input_event(press)
	await _hold(0.5)
	link.client.submitted_request.disconnect(count)
	print("EVIDENCE ", JSON.stringify({"menu_input": {"requests": asked[0], "yaw": [yaw, player.rotation.y],
		"pitch": [pitch, player.rig.pitch], "moved_m": at.distance_to(player.global_position)}}))
	_set_check(asked[0] == 0 and is_equal_approx(yaw, player.rotation.y) and is_equal_approx(pitch, player.rig.pitch)
		and at.distance_to(player.global_position) < 0.05, "an open menu takes gameplay input",
		"%d requests while open" % asked[0])
	menu.close()


## Every UI text of the slice: the HUD and the menu, tab by tab; the language list (each language names
## itself) and the live frame-rate reading are left out.
func _ui_texts() -> PackedStringArray:
	var menu: MineWorldSettingsMenu = slice.settings_menu
	var skip := [str(menu._language.get_path()), str(menu._fps_value.get_path())]
	var out := PackedStringArray()
	var current := menu._tabs.current_tab
	for tab in menu._tabs.get_tab_count():
		menu._tabs.current_tab = tab
		for line in MineWorldText.visible_ui_texts(slice):
			var path := line.get_slice("\t", 0)
			if not skip.any(func(s: String) -> bool: return path.begins_with(s)) and not out.has(line):
				out.append(line)
	menu._tabs.current_tab = current
	return out


static func _masked(lines: PackedStringArray) -> PackedStringArray:
	var out := PackedStringArray()
	var digits := RegEx.create_from_string("[0-9]+")
	for line in lines:
		out.append(digits.sub(line, "#", true))
	return out


func _answered(link: SliceLink, tok: String) -> bool:
	for a in link.answers:
		if a["token"] == tok:
			return true
	return false

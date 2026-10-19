extends SceneTree

# The reader check for `MineWorldNeeds` (`clients/protocol/mineworld/needs.gd`): the shapes the packs
# disclose, read through the module with no server. Synthetic frames only for this file; the recorded
# observations of tests/fixtures/observations/needs/ are checked by the claims added at C2.
#
#   godot --headless --path clients/protocol --script res://checks/needs_check.gd
#
# Every expected value is written by hand from the frame, never computed by the code under test
# (`step-15-demo-3d.md` §18.4 A-1). Prints one `[needs] PASS|FAIL <claim>: <observed>` line per claim
# and exits 1 if any fails.

const Observation := preload("res://mineworld/observation.gd")
const Needs := preload("res://mineworld/needs.gd")

const ME := "17"
const BOB := "18"
const EMPLOYER := "9001"
const WORKPLACE := "7001"

var _failures := 0


func _init() -> void:
	_own_needs()
	_asleep()
	_next_part()
	_lateness()
	_recorded()
	_nothing_disclosed()
	print("[needs] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
	quit(0 if _failures == 0 else 1)


func _claim(claim: String, observed: Variant, expected: Variant) -> void:
	var same := JSON.stringify(observed) == JSON.stringify(expected)
	if not same:
		_failures += 1
	print("[needs] %s %s: %s%s" % ["PASS" if same else "FAIL", claim, JSON.stringify(observed),
		"" if same else "  (expected %s)" % JSON.stringify(expected)])


func _observation(components: Array, events: Array = []) -> Observation:
	return Observation.new({"observer": ME, "at": 0, "entities": [
		{"id": ME, "entity_type": "person", "components": components},
		{"id": BOB, "entity_type": "person", "components": []},
	], "events": events})


func _event(event_type: String, payload: Dictionary, id: String) -> Dictionary:
	return {"id": id, "at": 0, "event_type": event_type, "subjects": [], "participants": [],
		"payload": {"event_type": event_type, "schema_version": 1, "payload": payload}}


func _own_needs() -> void:
	var ana := _observation([{"component_type": "needs", "payload": {"hunger": 640, "tiredness": 120,
		"words": {"hunger": "hungry", "tiredness": "ok"}, "asleep": false}}])
	_claim("own needs in the words map's order", Needs.own_needs(ana), [
		{"need": "hunger", "word": "hungry", "level": 640},
		{"need": "tiredness", "word": "ok", "level": 120}])
	var third := _observation([{"component_type": "needs", "payload": {"hunger": "x", "tiredness": 300,
		"words": {"hunger": "peckish", "tiredness": "sleepy", "thirst": "peckish", "odd": 5}}}])
	_claim("a level that is not a number reads -1; a word that is not text is skipped; a third need reads",
		Needs.own_needs(third), [
		{"need": "hunger", "word": "peckish", "level": -1},
		{"need": "tiredness", "word": "sleepy", "level": 300},
		{"need": "thirst", "word": "peckish", "level": -1}])
	_claim("no needs record: none", Needs.own_needs(_observation([])), [])


func _asleep() -> void:
	var bob := Observation.new({"observer": ME, "at": 0, "entities": [
		{"id": BOB, "entity_type": "person", "components": [{"component_type": "asleep",
			"payload": {"asleep": true, "since": 82800}}]},
	], "events": []})
	_claim("another person's sleep, no alarm", Needs.asleep_of(bob, BOB), {"since": 82800, "alarm": null})
	var mine := _observation([{"component_type": "asleep", "payload": {"asleep": true, "since": 82800,
		"alarm": 108000}}])
	_claim("my own sleep with its alarm", Needs.asleep_of(mine, ME), {"since": 82800, "alarm": 108000})
	_claim("a sleep without since reads nothing", Needs.asleep_of(_observation([{"component_type": "asleep",
		"payload": {"asleep": true}}]), ME), {})
	_claim("awake, nothing disclosed", Needs.asleep_of(_observation([]), ME), {})


func _next_part() -> void:
	var punctual := _observation([{"component_type": "agenda", "payload": {"label": "work", "place": {
		"entity": WORKPLACE, "entity_type": "place"}, "since": 0, "until": 28800,
		"next": {"label": "work", "place": {"entity": WORKPLACE, "entity_type": "place"}, "from": 28800,
			"punctual": true}}}])
	_claim("the next part, punctual", Needs.next_part(punctual), {"label": "work", "place": WORKPLACE,
		"from": 28800, "punctual": true})
	var plain := _observation([{"component_type": "agenda", "payload": {"label": "rest", "place": {
		"entity": WORKPLACE, "entity_type": "place"}, "since": 0, "until": 3600}}])
	_claim("an agenda with no next part", Needs.next_part(plain), {})


func _lateness() -> void:
	var events := [
		_event("late-for-shift", {"employee": {"entity": ME, "entity_type": "person"},
			"employer": {"entity": EMPLOYER, "entity_type": "organization"}, "shift": 28800, "late_by": 900,
			"count": 1, "level": "notice", "absent": false}, "51"),
		_event("late-for-shift", {"employee": {"entity": BOB, "entity_type": "person"},
			"employer": {"entity": EMPLOYER, "entity_type": "organization"}, "shift": 28800, "late_by": 60,
			"count": 1, "level": "notice", "absent": false}, "52"),
		_event("late-for-shift", {"employee": {"entity": ME, "entity_type": "person"},
			"employer": {"entity": EMPLOYER, "entity_type": "organization"}, "shift": 115200,
			"late_by": 28800, "count": 2, "level": "warning", "absent": true}, "53"),
	]
	_claim("my lateness, oldest first, others' skipped", Needs.lateness(_observation([], events)), [
		{"id": "51", "employer": EMPLOYER, "late_by": 900, "count": 1, "level": "notice", "absent": false},
		{"id": "53", "employer": EMPLOYER, "late_by": 28800, "count": 2, "level": "warning", "absent": true}])
	_claim("no lateness fact: none", Needs.lateness(_observation([])), [])


## The recorded observations (tests/fixtures/observations/needs/, `SOURCE.md`): each expected value below
## was read from the file by hand, never from this reader.
func _recorded() -> void:
	var directory := ProjectSettings.globalize_path("res://").path_join("../../tests/fixtures/observations/needs")
	var night := _fixture(directory, "market-town-night")
	_claim("market town at night: my needs, in the pack's words", Needs.own_needs(night), [
		{"need": "hunger", "word": "peckish", "level": 593},
		{"need": "tiredness", "word": "tired", "level": 831}])
	_claim("market town at night: my sleep, with its alarm", Needs.asleep_of(night, "8"),
		{"since": 8700, "alarm": 33060})
	_claim("market town at night: my next part, not punctual", Needs.next_part(night),
		{"label": "coffee", "place": "2", "from": 25200, "punctual": false})
	_claim("market town at night: no lateness", Needs.lateness(night), [])
	var late := _fixture(directory, "commute-late")
	_claim("commute late: my needs, famished and exhausted", Needs.own_needs(late), [
		{"need": "hunger", "word": "famished", "level": 1000},
		{"need": "tiredness", "word": "exhausted", "level": 1000}])
	_claim("commute late: awake, no sleep", Needs.asleep_of(late, "8"), {})
	_claim("commute late: my next part is punctual", Needs.next_part(late),
		{"label": "work", "place": "1", "from": 113400, "punctual": true})
	_claim("commute late: the shift I never came to, told once", Needs.lateness(late), [
		{"id": "51", "employer": "12", "late_by": 16200, "count": 1, "level": "notice", "absent": true}])
	var sleeper := _fixture(directory, "commute-sleeper")
	_claim("commute sleeper: Ben's needs, fine and tired", Needs.own_needs(sleeper), [
		{"need": "hunger", "word": "ok", "level": 329},
		{"need": "tiredness", "word": "tired", "level": 714}])
	_claim("commute sleeper: Ana asleep since 315 s, with no alarm for me", Needs.asleep_of(sleeper, "7"),
		{"since": 315, "alarm": null})
	_claim("commute sleeper: I am awake", Needs.asleep_of(sleeper, "8"), {})


func _fixture(directory: String, name: String) -> Observation:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(directory.path_join(name + ".json")))
	if typeof(parsed) != TYPE_DICTIONARY:
		_failures += 1
		print("[needs] FAIL %s: the fixture does not parse" % name)
		return Observation.new({"observer": ME, "at": 0, "entities": [], "events": []})
	return Observation.new(parsed)


func _nothing_disclosed() -> void:
	var bare := Observation.new({"observer": ME, "at": 0, "entities": [], "events": []})
	_claim("a frame with no pack: needs", Needs.own_needs(bare), [])
	_claim("a frame with no pack: next part", Needs.next_part(bare), {})
	_claim("a frame with no pack: lateness", Needs.lateness(bare), [])

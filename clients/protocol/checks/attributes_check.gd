extends SceneTree

# The composition check for `MineWorldAttributes` (`clients/protocol/mineworld/attributes.gd`; S30-f §5.4):
# every case of the hand-written golden `authoring/attributes/tests/fixtures/composition.json`, which the
# Rust and Python readers also read, merged, composed, listed and sheeted by this module, must equal it.
#
#   godot --headless --path clients/protocol --script res://checks/attributes_check.gd
#
# Prints one `[attributes] PASS|FAIL <case> <claim>` line per claim, then `all attributes checks pass`
# when none fails, and exits 1 if any does.

const Attributes := preload("res://mineworld/attributes.gd")

const GOLDEN := "../../authoring/attributes/tests/fixtures/composition.json"

var _failures := 0


func _init() -> void:
	var path := ProjectSettings.globalize_path("res://").path_join(GOLDEN)
	var text := FileAccess.get_file_as_string(path)
	var golden: Variant = JSON.parse_string(text)
	if typeof(golden) != TYPE_DICTIONARY:
		print("[attributes] FAIL the golden is not readable at %s" % path)
		quit(1)
		return
	for case in golden["cases"]:
		_case(case)
	print("[attributes] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
	if _failures == 0:
		print("all attributes checks pass")
	quit(0 if _failures == 0 else 1)


func _claim(name: String, claim: String, observed: Variant, expected: Variant) -> void:
	var same := JSON.stringify(observed) == JSON.stringify(expected)
	if not same:
		_failures += 1
	print("[attributes] %s %s %s%s" % ["PASS" if same else "FAIL", name, claim,
		"" if same else ": %s (expected %s)" % [JSON.stringify(observed), JSON.stringify(expected)]])


func _case(case: Dictionary) -> void:
	var name := String(case["name"])
	var records: Array = []
	for component in case["components"]:
		records.append({"component_type": component["type"], "payload": component["payload"]})
	var entries := Attributes.blocks({"components": records})
	var merged: Array = []
	for entry in entries:
		merged.append(String(entry["attribute"]))
	_claim(name, "merged", merged, case["merged"])
	for composition in case["compose"]:
		var facet: Variant = composition["facet"]
		var budget: Variant = composition["budget"]
		var text := Attributes.compose(entries, "" if facet == null else String(facet),
			-1 if budget == null else int(budget))
		_claim(name, "compose %s at %s" % [facet, budget], text, composition["text"])
		if budget != null and text.to_utf8_buffer().size() > int(budget):
			_claim(name, "within %s bytes" % budget, text.to_utf8_buffer().size(), int(budget))
	_claim(name, "inspect", Array(Attributes.inspect(entries)), case["inspect"])
	_claim(name, "sheet", Array(Attributes.sheet(entries)), case["sheet"])

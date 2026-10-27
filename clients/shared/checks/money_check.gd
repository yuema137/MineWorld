extends SceneTree

## The money formatter and parser, headless (`clients/shared/TRADE.md` §3; S31-e AC-e-1).
##
##   godot --headless --path clients/shared --script res://checks/money_check.gd
##
## Every row of TRADE.md §3.2's table formats exactly as written; `parse(format(x)) == x` for 200 amounts per
## currency drawn by SplitMix64 from seed 31 over 0 … 10^12; each null case of §3.3 is null; with no currency
## an amount is the bare integer. Prints `[FAIL]` lines and `money_check: PASS|FAIL`; exits non-zero on a
## failure.

const TOP := 1000000000000
const DOLLAR := {"name": "US dollar", "symbol": "$", "exponent": 2, "symbol_at": "before"}
const CROWN := {"name": "crown", "symbol": "cr", "exponent": 0, "symbol_at": "after"}
const DINAR := {"name": "dinar", "symbol": "KD", "exponent": 3, "symbol_at": "before"}
const UNIDAD := {"name": "unidad", "symbol": "UF", "exponent": 4, "symbol_at": "after"}

var _fails := 0
var _state := 31


func _init() -> void:
	var problems := MineWorldText.load_layers("")
	_check(problems.is_empty(), "the shared layer loads", "\n".join(problems))
	MineWorldText.set_language("en")
	_table()
	_round_trips()
	_nulls()
	_currency_of()
	print("money_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


func _table() -> void:
	var rows := [
		[DOLLAR, 200000, "$2,000.00"], [DOLLAR, 300, "$3.00"], [DOLLAR, 5, "$0.05"], [DOLLAR, 0, "$0.00"],
		[CROWN, 3, "3 cr"], [CROWN, 200000, "200,000 cr"], [DINAR, 1500, "KD1.500"],
		[UNIDAD, 12345, "1.2345 UF"], [DOLLAR, TOP, "$10,000,000,000.00"], [{}, 300, "300"],
	]
	for row in rows:
		var said := MineWorldMoney.format(row[1], row[0])
		_check(said == row[2], "format %d in %s" % [row[1], row[0].get("name", "nothing")], said)
	MineWorldText.set_language("zh_Hans")
	_check(MineWorldMoney.format(3, CROWN) == "3 cr", "zh_Hans keeps the world's symbol and side",
		MineWorldMoney.format(3, CROWN))
	MineWorldText.set_language("en")


## SplitMix64 (Steele, Lea, Flood 2014), seed 31: the samples are the same on every machine.
func _next() -> int:
	_state += -7046029254386353131
	var z := _state
	z = (z ^ ((z >> 30) & 0x3FFFFFFFF)) * -4658895280553007687
	z = (z ^ ((z >> 27) & 0x1FFFFFFFFF)) * -7723592293110705685
	return z ^ ((z >> 31) & 0x1FFFFFFFF)


func _round_trips() -> void:
	for currency in [DOLLAR, CROWN, DINAR, UNIDAD]:
		var wrong := 0
		for i in 200:
			var minor := posmod(_next(), TOP + 1)
			var back: Variant = MineWorldMoney.parse(MineWorldMoney.format(minor, currency), currency)
			if not (back is int and back == minor):
				wrong += 1
				if wrong == 1:
					_check(false, "round trip in %s" % currency["name"], "%d -> %s -> %s" % [minor,
						MineWorldMoney.format(minor, currency), str(back)])
		_check(wrong == 0, "200 round trips in %s" % currency["name"], "%d wrong" % wrong)


func _nulls() -> void:
	var cases := [
		[DOLLAR, "2.6", 260], [DOLLAR, "2.60", 260], [DOLLAR, " $2.60 ", 260], [DOLLAR, "1,234.5", 123450],
		[DOLLAR, "0", 0], [DOLLAR, "2.605", null], [DOLLAR, "-1", null], [DOLLAR, "abc", null],
		[DOLLAR, "", null], [DOLLAR, "1.2.3", null], [DOLLAR, "2.", null], [DOLLAR, "1234567890123456789", null],
		[CROWN, "12", 12], [CROWN, "12 cr", 12], [CROWN, "1.5", null], [CROWN, "0", 0],
		[DINAR, "1.5", 1500], [UNIDAD, "0.0001", 1], [{}, "12", null],
	]
	for c in cases:
		var got: Variant = MineWorldMoney.parse(c[1], c[0])
		var same: bool = got == null if c[2] == null else (got is int and got == c[2])
		_check(same, "parse \"%s\" in %s" % [c[1], c[0].get("name", "nothing")], str(got))


## A stand-in observation: the one call `currency_of` makes.
class Seen:
	extends RefCounted
	var record: Dictionary

	func _init(a_record: Dictionary) -> void:
		record = a_record

	func own_component(_type: String) -> Dictionary:
		return record


func _currency_of() -> void:
	var seen := Seen.new({"name": "crown", "symbol": "cr", "exponent": 0.0, "symbol_at": "after",
		"extra": [{"item": "41", "value": 7}]})
	var currency := MineWorldMoney.currency_of(seen)
	_check(currency == CROWN, "currency_of keeps the four fields and nothing else", str(currency))
	_check(MineWorldMoney.currency_of(Seen.new({})).is_empty(), "no record, no currency")


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
		print("[FAIL] %s  %s" % [what, detail])

## Amounts of money, written and read in the currency the world disclosed (`clients/shared/TRADE.md` §3;
## `docs/decisions/ARC-184.md`; S31-e §5.2).
##
## The world says what its money is called, its symbol, which side the symbol goes on and how many digits of
## an amount are minor units (`economy`'s `currency` record, `ARC-111`). This file turns an integer of minor
## units into text with that, and a typed price back into minor units. It assumes no scale: with no currency
## disclosed an amount is shown as the bare integer it is, and nothing can be parsed. It never decides
## anything about an amount: the server says whether a price is in range, affordable or current.
##
## The separators and the symbol's spacing are wording (`format.money.*`); the side is the world's.
class_name MineWorldMoney
extends RefCounted

## The most digits a typed amount may have, so that it fits a GDScript integer (SD-S31e-8).
const DIGITS_MAX := 18
## The exponents a currency may declare (`economy`'s `Exponent`, 0 … 4).
const EXPONENT_TOP := 4


## The observer's own `currency` record, as `{name, symbol, exponent, symbol_at}`, or `{}` when the world
## disclosed none. Only those four fields are kept: what else the record carries is not this file's.
## `observation` is a `MineWorldObservation` (duck-typed, so the module also runs on its own).
static func currency_of(observation: Object) -> Dictionary:
	if observation == null:
		return {}
	var record: Dictionary = observation.own_component("currency")
	if typeof(record.get("exponent")) not in [TYPE_INT, TYPE_FLOAT] or typeof(record.get("symbol")) != TYPE_STRING:
		return {}
	return {"name": str(record.get("name", "")), "symbol": String(record["symbol"]),
		"exponent": clampi(int(record["exponent"]), 0, EXPONENT_TOP),
		"symbol_at": str(record.get("symbol_at", "before"))}


## An amount of minor units as text in `currency` (`currency_of`'s shape): "$2,000.00", "3 cr". With
## `currency` empty the amount is the bare integer, through `format.money.plain`.
static func format(minor: Variant, currency: Dictionary) -> String:
	var units := int(minor)
	if currency.is_empty():
		return MineWorldText.text("format.money.plain", {"amount": str(units)})
	var minus := "-" if units < 0 else ""
	var digits := str(absi(units))
	var places := int(currency.get("exponent", 0))
	var number := minus + _major(digits, places)
	if places != 0:
		number += MineWorldText.text("format.money.decimal_mark") + _minor(digits, places)
	var side := "format.money.after" if String(currency.get("symbol_at", "")) == "after" else "format.money.before"
	return MineWorldText.text(side, {"amount": number, "symbol": String(currency.get("symbol", ""))})


## A typed price as minor units of `currency`, or null when the text is not an amount of it. Spaces, the
## symbol and group marks are dropped; one decimal mark with at most `exponent` digits after it is read; a
## sign, any other character, or more than DIGITS_MAX digits is null. Representation only: 0 and an
## amount above the world's bound parse, and the server answers them (`price-out-of-range`).
static func parse(text: String, currency: Dictionary) -> Variant:
	if currency.is_empty():
		return null
	var said := text.strip_edges()
	var symbol := String(currency.get("symbol", ""))
	if symbol != "":
		said = said.replace(symbol, "")
	said = said.replace(MineWorldText.text("format.money.group_mark"), "").replace(" ", "")
	var mark := MineWorldText.text("format.money.decimal_mark")
	if mark != ".":
		said = said.replace(mark, ".")
	var parts := said.split(".")
	if said == "" or parts.size() > 2:
		return null
	var whole := parts[0]
	var fraction := parts[1] if parts.size() == 2 else ""
	var places := int(currency.get("exponent", 0))
	if parts.size() == 2 and (places == 0 or fraction == "" or fraction.length() > places):
		return null
	fraction = fraction.rpad(places, "0")
	var all := (whole if whole != "" else "0") + fraction
	if not _only_digits(all) or all.length() > DIGITS_MAX:
		return null
	return all.to_int()


## The major part of `digits` (an amount's decimal digits, no sign), grouped in threes.
static func _major(digits: String, places: int) -> String:
	var major := digits.left(digits.length() - places) if digits.length() > places else "0"
	var group := MineWorldText.text("format.money.group_mark")
	var out := ""
	var counted := 0
	for i in range(major.length() - 1, -1, -1):
		if counted == 3:
			out = group + out
			counted = 0
		out = major[i] + out
		counted += 1
	return out


## The minor part of `digits`, zero-padded to `places`.
static func _minor(digits: String, places: int) -> String:
	return digits.right(places) if digits.length() >= places else digits.lpad(places, "0")


static func _only_digits(said: String) -> bool:
	if said == "":
		return false
	for c in said:
		if not "0123456789".contains(c):
			return false
	return true

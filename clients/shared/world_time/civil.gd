## The civil calendar's arithmetic, integers only, mirroring the `calendar` System Pack's `civil.rs`
## (`clients/shared/WORLD_TIME.md` §4, `ARC-266`): what a client needs to roll a disclosed date past
## midnight before the next record arrives. It reads no clock.
class_name MineWorldCivil
extends RefCounted


## Whether `year` is a leap year of the Gregorian calendar.
static func is_leap(year: int) -> bool:
	return (year % 4 == 0 and year % 100 != 0) or year % 400 == 0


## How many days `month` (1 … 12) of `year` has.
static func days_in_month(year: int, month: int) -> int:
	if month == 2:
		return 29 if is_leap(year) else 28
	if month == 4 or month == 6 or month == 9 or month == 11:
		return 30
	return 31


## The day after `date` (`{year, month, day}`), as a new dictionary of the same keys.
static func next_date(date: Dictionary) -> Dictionary:
	var year := int(date.get("year", 1))
	var month := int(date.get("month", 1))
	var day := int(date.get("day", 1)) + 1
	if day > days_in_month(year, month):
		day = 1
		month += 1
		if month > 12:
			month = 1
			year += 1
	return {"year": year, "month": month, "day": day}

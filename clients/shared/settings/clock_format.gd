## A time of day in the chosen 12-hour or 24-hour form, worded by the catalogs
## (`clients/shared/SETTINGS.md` §6.6; step-19 §8.2's templates; QTW-16).
##
## The form is the setting, or for "automatic" the current language's `clock.default`. The words —
## `AM`/`PM`, 上午/下午, and their place — are catalog content, never code. The date (TW-e) is a
## disclosed date the composition root passes in; this file reads no clock and names no other module.
class_name MineWorldClockFormat
extends RefCounted


## A disclosed date `{year, month, day, weekday}` (weekday 0 = Monday) as `hud.date`. Each language
## names its weekdays and months (`hud.weekday.<0…6>`, `hud.month.<1…12>`) as its own template needs:
## "Thu 8 Oct 2026", "2026年10月8日 星期四".
static func date(d: Dictionary) -> String:
	var weekday := posmod(int(d.get("weekday", 0)), 7)
	var month := clampi(int(d.get("month", 1)), 1, 12)
	return MineWorldText.text("hud.date", {"year": int(d.get("year", 0)), "day": int(d.get("day", 1)),
		"month": MineWorldText.text("hud.month." + str(month)),
		"weekday": MineWorldText.text("hud.weekday." + str(weekday))})


## The HUD's clock line: `hud.datetime` with a date, else exactly `day_time(seconds)` (`INV-TW-6`).
static func date_time(seconds: int, d: Dictionary) -> String:
	if d.is_empty():
		return day_time(seconds)
	return MineWorldText.text("hud.datetime", {"date": date(d), "time": time_of_day(posmod(seconds, 86400))})


## World seconds as the HUD's day and time: `hud.day_time` ("day 1  7:42 PM"), the time in the form
## in effect. Both clients' HUDs use it (S20 SD-SET-a-7).
static func day_time(seconds: int) -> String:
	return MineWorldText.text("hud.day_time", {"day": seconds / 86400 + 1,
		"time": time_of_day(posmod(seconds, 86400))})


## `seconds_into_day` (0 … 86399; anything else is taken modulo a day) as `hud.time.24h` or
## `hud.time.12h`. `clock` is H12 or H24; AUTO (the default) is the form in effect.
static func time_of_day(seconds_into_day: int,
		clock: MineWorldSettings.ClockFormat = MineWorldSettings.ClockFormat.AUTO) -> String:
	var form := clock if clock != MineWorldSettings.ClockFormat.AUTO else MineWorldText.clock()
	var in_day := posmod(seconds_into_day, 86400)
	var hour := in_day / 3600
	var minute := (in_day % 3600) / 60
	if form == MineWorldSettings.ClockFormat.H24:
		return MineWorldText.text("hud.time.24h", {"hour24": "%02d" % hour, "minute2": "%02d" % minute})
	var hour12 := hour % 12
	if hour12 == 0:
		hour12 = 12
	return MineWorldText.text("hud.time.12h", {"hour12": str(hour12), "minute2": "%02d" % minute,
		"ampm": MineWorldText.text("hud.ampm.am" if hour < 12 else "hud.ampm.pm")})


## World seconds as a schedule time, 24-hour, worded by `format.clock` ("day 2  07:30"): the rows of the
## needs and sleep (S35 N-e). The same words in both clients; the clock setting does not govern it.
static func clock(seconds: int) -> String:
	var in_day := posmod(seconds, 86400)
	return MineWorldText.text("format.clock", {"day": seconds / 86400 + 1, "hh": "%02d" % (in_day / 3600),
		"mm": "%02d" % ((in_day % 3600) / 60)})


## Whole minutes for a span of `seconds`, rounded up: a unit conversion for a wording argument such as
## "{minutes} min late" (S35 N-e). Not a rule: nothing compares the result with anything.
static func whole_minutes(seconds: int) -> int:
	return (seconds + 59) / 60

## A time of day in the chosen 12-hour or 24-hour form, worded by the catalogs
## (`clients/shared/SETTINGS.md` §6.6; step-19 §8.2's templates; QTW-16).
##
## The form is the setting, or for "automatic" the current language's `clock.default`. The words —
## `AM`/`PM`, 上午/下午, and their place — are catalog content, never code. The date (TW-e) is a
## disclosed date the composition root passes in; this file reads no clock and names no other module.
class_name MineWorldClockFormat
extends RefCounted


## A disclosed date `{year, month, day, weekday}` (weekday 0 = Monday) as `hud.date`.
static func date(d: Dictionary) -> String:
	var weekday := posmod(int(d.get("weekday", 0)), 7)
	var month := clampi(int(d.get("month", 1)), 1, 12)
	return MineWorldText.text("hud.date", {"year": int(d.get("year", 0)), "month": month,
		"day": int(d.get("day", 1)), "month_short": MineWorldText.text("hud.month.short." + str(month)),
		"weekday_short": MineWorldText.text("hud.weekday.short." + str(weekday)),
		"weekday_long": MineWorldText.text("hud.weekday.long." + str(weekday))})


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

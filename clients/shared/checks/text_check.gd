extends SceneTree

## The text layer, headless (step-20 §12.5 C3): layers and discovery, the live switch's round trip, the
## readable fallback, `code`'s families, and the clock in both forms and both languages.
##
##   godot --headless --path clients/shared --script res://checks/text_check.gd

var _fails := 0


func _init() -> void:
	var problems := MineWorldText.load_layers("")
	_check(problems.is_empty(), "the shared layer loads", "\n".join(problems))
	_check(MineWorldText.locales() == PackedStringArray(["en", "zh_Hans"]), "en and zh_Hans are offered, en first",
		str(MineWorldText.locales()))
	MineWorldText.set_language("zh_Hans")
	_check(MineWorldText.text("action.talk", {"target": "Alice"}) == "和Alice说话", "zh_Hans words a shared key",
		MineWorldText.text("action.talk", {"target": "Alice"}))
	MineWorldText.set_language("en")
	_check(MineWorldText.text("action.talk", {"target": "Alice"}) == "Talk to Alice", "back in en, the same key",
		MineWorldText.text("action.talk", {"target": "Alice"}))
	_check(MineWorldText.code("reason", "too_far_away") == "too far away", "code builds a reason key")
	_check(MineWorldText.code("reason", "a_newer_code") == "a newer code", "an unknown code reads, never raw")
	_check(MineWorldText.text("ui.settings.nothing_here") == "nothing here", "an unknown key reads, never raw")
	MineWorldText.set_language("zh_Hans")
	_check(MineWorldText.text("ui.settings.nothing_here") == "nothing here", "an unknown key falls back in zh_Hans too")
	# A key only English has is shown in English, never raw (the fallback locale).
	var only_en := Translation.new()
	only_en.locale = "en"
	only_en.add_message("ui.settings.only-english", "Only English")
	TranslationServer.add_translation(only_en)
	_check(MineWorldText.text("ui.settings.only-english") == "Only English", "a key zh_Hans lacks falls back to English")
	TranslationServer.remove_translation(only_en)
	_clock()
	print("text_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


## 00:00, 12:00, 19:42 and 23:59 in both forms and both languages; AUTO follows each language's default.
func _clock() -> void:
	var H12 := MineWorldSettings.ClockFormat.H12
	var H24 := MineWorldSettings.ClockFormat.H24
	var expected := {
		"en": [["12:00 AM", "00:00"], ["12:00 PM", "12:00"], ["7:42 PM", "19:42"], ["11:59 PM", "23:59"]],
		"zh_Hans": [["上午 12:00", "00:00"], ["下午 12:00", "12:00"], ["下午 7:42", "19:42"], ["下午 11:59", "23:59"]],
	}
	var times := [0, 12 * 3600, 19 * 3600 + 42 * 60, 23 * 3600 + 59 * 60]
	for locale in expected:
		MineWorldText.set_language(locale)
		for i in times.size():
			var twelve := MineWorldClockFormat.time_of_day(times[i], H12)
			var twenty_four := MineWorldClockFormat.time_of_day(times[i], H24)
			_check(twelve == expected[locale][i][0] and twenty_four == expected[locale][i][1],
				"%s %d s" % [locale, times[i]], "%s | %s" % [twelve, twenty_four])
	MineWorldText.set_clock(MineWorldSettings.ClockFormat.AUTO)
	MineWorldText.set_language("en")
	_check(MineWorldClockFormat.time_of_day(times[2]) == "7:42 PM", "automatic is 12h in en")
	MineWorldText.set_language("zh_Hans")
	_check(MineWorldClockFormat.time_of_day(times[2]) == "19:42", "automatic is 24h in zh_Hans")
	MineWorldText.set_clock(H12)
	_check(MineWorldClockFormat.time_of_day(times[2]) == "下午 7:42", "a chosen 12h holds in zh_Hans")
	MineWorldText.set_clock(MineWorldSettings.ClockFormat.AUTO)
	_check(MineWorldClockFormat.time_of_day(86400 + times[2]) == "19:42", "seconds past a day wrap")


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
	print("[%s] %s  %s" % ["PASS" if ok else "FAIL", what, detail])

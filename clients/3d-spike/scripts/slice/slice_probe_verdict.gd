## The probe run's exit status, from what the probe printed (F-RC-7): a scripted mode that reports a
## failed check makes the process exit 1, so CI and the release smoke test can trust the exit status as
## well as the printed verdict.
##
## The rule is the one `scripts/ci_godot.py judge_probe` applies to the same lines, so the two cannot
## disagree: a line that begins `FAIL` or `[FAIL]`, or a summary `<n> <WHAT> CHECKS FAILED` or
## `<n> <WHAT> CHECKS OUT OF RANGE`, is a failed check. Watching the printed lines rather than each
## mode's own counter covers every mode, including the early returns (`FAIL: no --server= given`)
## that end a mode before its summary, and any mode added later that prints the same verdicts.
##
## Registered by `SliceProbe._ready` with `OS.add_logger`; Godot may call it from any thread, and it
## only ever sets a flag.
class_name SliceProbeVerdict
extends Logger

var failed := false
## The first line that failed the run, for the exit line.
var first := ""

static var _summary := RegEx.create_from_string("^\\d+ [A-Z][A-Z ]* CHECKS (FAILED|OUT OF RANGE)$")


func _log_message(message: String, _error: bool) -> void:
	if failed:
		return
	for raw in message.split("\n"):
		var line := raw.strip_edges()
		if is_failure(line):
			first = line
			failed = true
			return


## Whether one printed line reports a failed check.
static func is_failure(line: String) -> bool:
	return line.begins_with("FAIL") or line.begins_with("[FAIL]") or _summary.search(line) != null

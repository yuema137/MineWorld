## Failed-check collection only; a zero exit still needs a terminal semantic verdict from the mode.
## Callbacks may run concurrently. Never log from here. The first failure is the first callback
## observed under the mutex, not a claimed chronological ordering across producer threads.
extends Logger

var _mutex := Mutex.new()
var _first := ""
var _summary := RegEx.create_from_string("^[1-9][0-9]* [A-Z][A-Z ]* CHECKS (FAILED|OUT OF RANGE)$")


func _log_message(message: String, _error: bool) -> void:
	_mutex.lock()
	if _first == "":
		for raw in message.split("\n"):
			var line := raw.strip_edges()
			if line == "FAIL" or line.begins_with("FAIL:") or line.begins_with("FAIL ") \
					or line.begins_with("[FAIL]") or _summary.search(line) != null \
					or line.begins_with("CONVERSATION CHECK FAILED (") \
					or line.begins_with("CAST CHECK FAILED ("):
				_first = line
				break
	_mutex.unlock()


## Unregister first, with probe producers finished, then synchronize with any active callback.
## The owner retains this collector through quit/teardown rather than clearing its last reference.
func first_failure() -> String:
	_mutex.lock()
	var first := _first
	_mutex.unlock()
	return first


func exit_status(mode_status: int) -> int:
	return 1 if mode_status == 0 and first_failure() != "" else mode_status

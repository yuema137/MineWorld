## Focused real-Logger / subprocess regression. Run with --headless --path clients/3d-spike
## --script res://tools/test_probe_verdict.gd. No server, rendering assets, or model calls.
extends SceneTree

const Collector := preload("res://scripts/slice/slice_probe_verdict.gd")
const Probe := preload("res://scripts/slice/slice_probe.gd")
# Hold one real callback across unregister to exercise the retained-reference teardown path.
class InFlightCollector extends Collector:
	var entered := Semaphore.new()
	var release := Semaphore.new()

	func _log_message(message: String, error: bool) -> void:
		if message.begins_with("FAIL: in flight"):
			entered.post()
			release.wait()
		super(message, error)


var failures := 0


func _initialize() -> void:
	call_deferred("_run")


func _check(ok: bool, claim: String) -> void:
	if not ok:
		failures += 1
		printerr("REGRESSION: " + claim)


func _run() -> void:
	var args := OS.get_cmdline_user_args()
	if not args.is_empty():
		var probe := Probe.new()
		OS.add_logger(probe._verdict_log)
		probe._verdict_registered = true
		match args[0]:
			"failure": print("FAIL: intentional child probe failure")
			"row": probe._verdict("intentional row", false, "regression")
			"status":
				probe._exit_code = 3
				print("FAIL: preserve mode status")
			"success": print("all regression checks pass")
		var code := probe._exit_status()
		probe.free()
		quit(code)
		return
	var bad := ["FAIL: no --server= given", "FAIL landmark view", "[FAIL] market",
		"2 LINK CHECKS FAILED", "1 STATION KIT CHECKS OUT OF RANGE",
		"CONVERSATION CHECK FAILED (empty caption or an id on screen)",
		"CAST CHECK FAILED (nobody perceived)", "  \n\tFAIL: multiline\nall link checks pass"]
	for line in bad:
		var log := Collector.new()
		OS.add_logger(log)
		print(line)
		OS.remove_logger(log)
		_check(log.exit_status(0) == 1, "failure vocabulary: " + line)
	var clean := Collector.new()
	OS.add_logger(clean)
	print("all link checks pass\nPASS example\nexpected FAIL in a negative test\n0 LINK CHECKS FAILED")
	OS.remove_logger(clean)
	_check(clean.exit_status(0) == 0, "success/explanatory controls")
	var concurrent := Collector.new()
	OS.add_logger(concurrent)
	print("FAIL: first observed")
	var workers: Array[Thread] = []
	for i in 4:
		var worker := Thread.new()
		worker.start(func():
			for j in 12:
				print("FAIL: concurrent")
				concurrent.first_failure())
		workers.append(worker)
	for worker in workers:
		worker.wait_to_finish()
	OS.remove_logger(concurrent)
	_check(concurrent.first_failure() == "FAIL: first observed", "stable first observed callback")
	_check(concurrent.exit_status(3) == 3, "existing nonzero status")
	var in_flight := InFlightCollector.new()
	OS.add_logger(in_flight)
	var producer := Thread.new()
	producer.start(func(): print("FAIL: in flight during removal"))
	in_flight.entered.wait()
	OS.remove_logger(in_flight)
	in_flight.release.post()
	producer.wait_to_finish()
	_check(in_flight.exit_status(0) == 1, "in-flight callback survives unregister")
	for mode in ["failure", "row", "status", "success", "no-summary"]:
		var output := []
		var code := OS.execute(OS.get_executable_path(), ["--headless", "--path",
			ProjectSettings.globalize_path("res://"), "--script", "res://tools/test_probe_verdict.gd",
			"--", mode], output, true)
		var want := 3 if mode == "status" else (1 if mode in ["failure", "row"] else 0)
		_check(code == want, "child exit " + mode + ": " + str(code))
		_check(not "SCRIPT ERROR" in str(output), "child script health " + mode)
		if mode == "no-summary":
			_check(not "checks pass" in str(output), "zero never fabricates a semantic pass")
	print("all verdict regression checks pass" if failures == 0 else "verdict regression failed")
	quit(0 if failures == 0 else 1)

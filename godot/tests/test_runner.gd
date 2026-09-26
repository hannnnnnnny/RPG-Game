## Headless test runner. Discovers every `tests/unit/test_*.gd`, calls each
## `test_*` method, and exits with a non-zero code on any failure so CI and
## tools/run_tests.sh can gate commits.
##
## Run: godot --headless --path godot res://tests/TestRunner.tscn
extends Node

const UNIT_DIR := "res://tests/unit"

var _failures: Array[String] = []
var _passed: int = 0

func _ready() -> void:
	for path in _discover():
		_run_suite(path)
	_report()
	get_tree().quit(1 if _failures.size() > 0 else 0)

func _discover() -> PackedStringArray:
	var found := PackedStringArray()
	var dir := DirAccess.open(UNIT_DIR)
	if dir == null:
		push_error("Test dir missing: %s" % UNIT_DIR)
		return found
	for file in dir.get_files():
		if file.begins_with("test_") and file.ends_with(".gd"):
			found.push_back("%s/%s" % [UNIT_DIR, file])
	found.sort()
	return found

func _run_suite(path: String) -> void:
	var suite: TestSuite = load(path).new()
	add_child(suite)
	for method in suite.get_method_list():
		var name: String = method.name
		if not name.begins_with("test_"):
			continue
		suite.reset_case()
		suite.before_each()
		suite.call(name)
		# Autoloads are shared by every suite; never let one case's
		# fixtures leak into the next (e.g. into the UI smoke test).
		GameState._reset_to_defaults()
		_record(path.get_file(), name, suite.case_failures())
	suite.queue_free()

func _record(suite_name: String, case_name: String, errors: Array[String]) -> void:
	if errors.is_empty():
		_passed += 1
		return
	for e in errors:
		_failures.push_back("%s::%s — %s" % [suite_name, case_name, e])

func _report() -> void:
	for f in _failures:
		printerr("FAIL ", f)
	print("Tests: %d passed, %d failed" % [_passed, _failures.size()])

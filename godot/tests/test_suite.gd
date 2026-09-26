## Base class for unit suites. Assertions collect failures instead of
## aborting so one run reports every broken expectation.
class_name TestSuite
extends Node

var _case_failures: Array[String] = []

func before_each() -> void:
	pass

func reset_case() -> void:
	_case_failures = []

func case_failures() -> Array[String]:
	return _case_failures

func assert_true(cond: bool, msg: String = "expected true") -> void:
	if not cond:
		_case_failures.push_back(msg)

func assert_false(cond: bool, msg: String = "expected false") -> void:
	assert_true(not cond, msg)

func assert_eq(actual: Variant, expected: Variant, msg: String = "") -> void:
	if actual != expected:
		_case_failures.push_back("%s expected <%s> got <%s>" % [msg, expected, actual])

func assert_between(v: float, lo: float, hi: float, msg: String = "") -> void:
	if v < lo or v > hi:
		_case_failures.push_back("%s %s not in [%s, %s]" % [msg, v, lo, hi])

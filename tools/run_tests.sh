#!/usr/bin/env bash
# Run the Godot headless unit tests. Override the engine path with $GODOT.
set -uo pipefail
cd "$(dirname "$0")/../godot"
GODOT="${GODOT:-$HOME/GodotPortable/Godot_v4.3-stable_win64_console.exe}"
# Import once so new scripts/class_names are registered before running.
"$GODOT" --headless --path . --import >/dev/null 2>&1 || true

log="$(mktemp)"
"$GODOT" --headless --path . res://tests/TestRunner.tscn 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
# Godot 4.3 has no logger hook, so runtime script errors inside scenes
# only reach stderr. Treat any of them as a failed run.
if grep -qE "SCRIPT ERROR|Parse Error" "$log"; then
	echo "FAIL: script errors were logged during the run" >&2
	status=1
fi
rm -f "$log"
exit "$status"

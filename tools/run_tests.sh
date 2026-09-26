#!/usr/bin/env bash
# Run the Godot headless unit tests. Override the engine path with $GODOT.
set -euo pipefail
cd "$(dirname "$0")/../godot"
GODOT="${GODOT:-$HOME/GodotPortable/Godot_v4.3-stable_win64_console.exe}"
# Import once so new scripts/class_names are registered before running.
"$GODOT" --headless --path . --import >/dev/null 2>&1 || true
"$GODOT" --headless --path . res://tests/TestRunner.tscn

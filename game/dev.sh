#!/usr/bin/env bash
# Dev entry point. Usage: ./dev.sh test | run | check | <any cargo args>
# On Windows with the GNU toolchain, Rust's bundled dlltool must be on PATH.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
if command -v rustc >/dev/null; then
	sc="$(rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained"
	[ -d "$sc" ] && export PATH="$sc:$PATH"
fi
case "${1:-run}" in
	test) shift; cargo test --workspace "$@" ;;
	check) shift; cargo clippy --workspace --all-targets "$@" -- -D warnings ;;
	run) shift; cargo run -p tides_game "$@" ;;
	*) cargo "$@" ;;
esac

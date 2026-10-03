#!/usr/bin/env bash
# Dev entry point. Usage: ./dev.sh test | run | check | web | serve | <any cargo args>
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
	web)
		# Release wasm build + JS glue + page + assets into dist/ (gitignored).
		shift
		cargo build -p tides_game --profile web --target wasm32-unknown-unknown "$@"
		rm -rf dist && mkdir -p dist
		wasm-bindgen --target web --no-typescript --out-dir dist --out-name tides_game 			target/wasm32-unknown-unknown/web/tides_game.wasm
		cp web/index.html dist/
		cp -r crates/tides_game/assets dist/assets
		;;
	serve) shift; python -m http.server -d dist "${1:-8080}" ;;
	*) cargo "$@" ;;
esac

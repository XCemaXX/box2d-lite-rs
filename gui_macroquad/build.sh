#!/usr/bin/env bash
# Usage: ./build.sh [--dev]. Output goes to ./pkg.
set -euo pipefail
cd "$(dirname "$0")"

profile=release
cargo_flags=(--release)
if [[ "${1:-}" == "--dev" ]]; then
    profile=debug
    cargo_flags=()
fi

cargo build -p macroquad_gui --target wasm32-unknown-unknown "${cargo_flags[@]}"
cp "../target/wasm32-unknown-unknown/$profile/macroquad_gui.wasm" pkg/macroquad_gui.wasm

# Flags match the wasm features rustc enables by default.
if [[ "$profile" == release ]] && command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
        --enable-mutable-globals --enable-reference-types --enable-multivalue \
        pkg/macroquad_gui.wasm -o pkg/macroquad_gui.wasm
fi

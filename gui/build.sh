#!/usr/bin/env bash
# Usage: ./build.sh [--dev]. Output goes to ./pkg.
# Needs wasm-bindgen-cli with the same version as the wasm-bindgen crate in Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")"

profile=release
cargo_flags=(--release)
if [[ "${1:-}" == "--dev" ]]; then
    profile=debug
    cargo_flags=()
fi

cargo build -p wgpu-gui --target wasm32-unknown-unknown "${cargo_flags[@]}"
wasm-bindgen --target web --out-dir pkg --out-name wgpu_gui \
    "../target/wasm32-unknown-unknown/$profile/wgpu_gui.wasm"

# Flags match the wasm features rustc enables by default.
if [[ "$profile" == release ]] && command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
        --enable-mutable-globals --enable-reference-types --enable-multivalue \
        pkg/wgpu_gui_bg.wasm -o pkg/wgpu_gui_bg.wasm
fi

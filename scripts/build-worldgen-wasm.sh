#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-src/wasm-worldgen}"
WASM_BINDGEN_VERSION="0.2.127"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen CLI ${WASM_BINDGEN_VERSION} is required." >&2
  echo "Install with: cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION} --locked" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"
cargo build -p interlink-worldgen-wasm --release --target wasm32-unknown-unknown
wasm-bindgen \
  --target web \
  --out-dir "${OUT_DIR}" \
  target/wasm32-unknown-unknown/release/interlink_worldgen_wasm.wasm

echo "Packaged Planet Engine Rust/WASM module in ${OUT_DIR}"

#!/bin/bash
set -e

echo "Building WebAssembly..."
cargo build --target wasm32-unknown-unknown --release 2>&1 | grep -v "^warning"

echo "Copying wasm binary to docs..."
cp target/wasm32-unknown-unknown/release/canyon_raider.wasm docs/

echo "✓ Build complete! Wasm at docs/canyon_raider.wasm"
ls -lh docs/canyon_raider.wasm
echo ""
echo "To test locally: python3 -m http.server -d docs 8000"
echo "Then open http://localhost:8000"

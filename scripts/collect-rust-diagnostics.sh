#!/usr/bin/env bash
set -euo pipefail
printf "CarlosK Wallet Rust diagnostics\n"
command -v rustc >/dev/null && rustc --version || echo "rustc not found"
command -v cargo >/dev/null && cargo --version || echo "cargo not found"
cd "$(dirname "$0")/../src-tauri"
echo "\n== cargo check =="
cargo check || true
echo "\n== cargo check --features ldk-node-signet-runtime =="
cargo check --features ldk-node-signet-runtime || true

#!/usr/bin/env bash
set -euo pipefail

echo "== CarlosK Wallet local validation =="
echo "Node: $(node --version)"
echo "npm:  $(npm --version)"
if command -v rustc >/dev/null 2>&1; then rustc --version; else echo "rustc not found"; exit 1; fi
if command -v cargo >/dev/null 2>&1; then cargo --version; else echo "cargo not found"; exit 1; fi

npm install
npm run check:frontend
npm run check:tauri
npm run check:tauri:ldk

echo "All requested v0.15 local validation checks completed."

$ErrorActionPreference = "Stop"
Write-Host "== CarlosK Wallet local validation =="
node --version
npm --version
rustc --version
cargo --version
npm install
npm run check:frontend
npm run check:tauri
npm run check:tauri:ldk
Write-Host "All requested v0.15 local validation checks completed."

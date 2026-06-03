Write-Host "CarlosK Wallet Rust diagnostics"
rustc --version
cargo --version
Set-Location "$PSScriptRoot/../src-tauri"
Write-Host "`n== cargo check =="
cargo check
Write-Host "`n== cargo check --features ldk-node-signet-runtime =="
cargo check --features ldk-node-signet-runtime

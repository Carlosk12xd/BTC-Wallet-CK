# Build Notes v0.90

This version adds `bdk_esplora`, so the first `cargo check` may download and compile additional Rust crates.

Run on your Mac:

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

If Cargo updates `Cargo.lock`, keep that updated in source control.

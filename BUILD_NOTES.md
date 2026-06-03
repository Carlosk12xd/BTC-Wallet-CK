# Build Notes

## v0.99.6

Run locally on macOS:

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

This build adds the `ureq` Rust dependency for HTTP calls to a Core Lightning REST/JSON-RPC endpoint.

The public npm install may not work inside the sandbox, so final validation should be done on the user's Mac where prior versions compiled successfully.

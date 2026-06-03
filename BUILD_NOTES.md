# Build Notes v0.27

Run:

```bash
npm install --no-audit --no-fund
npm run frontend:build
npm run check:tauri
npm run dev
```

If Tauri complains about `frontendDist`, run `npm run frontend:build` first so the `dist/` folder exists.

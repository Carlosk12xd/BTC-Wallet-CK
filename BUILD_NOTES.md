# Build Notes v0.80

Run on your Mac:

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

This version intentionally does not include `package-lock.json` to avoid private registry lockfile problems.

# CarlosK Wallet v0.27

CarlosK Wallet is now scoped down to the core product: a simple self-custody Bitcoin + BOLT12 desktop wallet.

## Core goals

1. Create a new BTC on-chain wallet/address.
2. Receive BTC on-chain by generating fresh addresses.
3. Send BTC on-chain. In v0.27 this is a validated send draft only; real sync, PSBT creation, signing, and broadcast are next.
4. Create a Lightning wallet profile and prepare for in-app BOLT12 receiving.
5. Save/validate external BOLT12 offers while embedded BOLT12 is built safely through LDK signet/testnet first.
6. Sign messages with BIP-322 Simple.

## What was removed from the main UI

The old mining dashboards, OCEAN wizards, payout estimators, runbooks, release gates, and long MVP checklists are no longer shown in the app. The source files can remain temporarily for reference, but the app experience is now only four tabs:

- Wallet
- Send / Receive
- Lightning
- Signatures

## Run locally

```bash
npm install --no-audit --no-fund
npm run frontend:build
npm run check:tauri
npm run dev
```

## Important safety status

v0.27 is a cleaner core wallet prototype. Do not store meaningful funds yet. Real on-chain sending and embedded BOLT12 receiving still need sync, broadcast, LDK signet/testnet receive, and backup/recovery tests.

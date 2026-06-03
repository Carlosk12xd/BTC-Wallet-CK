# CarlosK Wallet v0.80 Wallet MVP

CarlosK Wallet is now focused on the real product goal: a simple self-custody Bitcoin + BOLT12 desktop wallet.

## Core features in this version

- Create a new Bitcoin wallet on mainnet, testnet, or signet.
- Restore a Bitcoin wallet from seed.
- Save and load an encrypted local wallet file.
- Export and verify encrypted backup JSON.
- Generate fresh native SegWit receive addresses.
- Sync the active address with public mempool.space Esplora APIs.
- Show real confirmed, mempool, and total detected balance.
- Show UTXOs for the active address.
- Show recent transaction history for the active address.
- Load fee estimates.
- Create Bitcoin payment URI text.
- Validate send drafts against the current wallet network.
- Broadcast a signed raw transaction hex.
- Save an external BOLT12 offer locally.
- Sign messages with BIP-322 Simple.

## Still locked before 1.0

- Automatic wallet-built sends are not enabled yet.
- The app does not yet build/sign/broadcast PSBTs from wallet UTXOs.
- In-app BOLT12 generation is still locked.
- Embedded Lightning receive is still locked until signet/testnet channel recovery is proven.

## Run locally

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

## Important security note

Use testnet or signet while testing. The raw transaction broadcast tool is real. Only broadcast transaction hex you intentionally created and reviewed.

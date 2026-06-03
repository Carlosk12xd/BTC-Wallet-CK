# CarlosK Wallet v0.90 PSBT Send MVP

CarlosK Wallet is a local-first Bitcoin desktop wallet built with Tauri, React, TypeScript, and Rust.

v0.90 moves the app much closer to the real wallet goal: it can create/restore a wallet, persist encrypted wallet storage, sync the wallet backend with Esplora, build and sign a real wallet transaction locally, and prepare the signed raw transaction for broadcast.

## Current core features

- Create a new BTC wallet on mainnet, testnet, or signet
- Restore wallet from seed
- Save/load encrypted wallet file locally
- Export/verify/restore encrypted backup JSON
- Generate receive addresses
- Frontend address lookup, UTXO list, fee lookup, and tx history using mempool.space APIs
- Backend BDK/Esplora wallet sync
- Build, sign, and extract a real signed wallet transaction after backend sync
- Broadcast signed raw transaction
- Save/validate external BOLT12 offers
- BIP-322 message signing

## Very important safety note

v0.90 creates real signed transactions. Broadcasting a signed transaction spends wallet funds. Test on signet/testnet first. Do not store meaningful mainnet funds until repeated send/backup/restore tests pass.

## Run locally

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

## Send flow

1. Create or restore a wallet.
2. Fund the wallet on signet/testnet first.
3. Click **Backend Sync Wallet**.
4. Enter recipient, amount, and fee rate.
5. Click **Validate Send Draft**.
6. Click **Build & Sign Transaction**.
7. Review txid, recipient, amount, fee, and raw hex.
8. Load the signed transaction into the broadcast box.
9. Broadcast only after review.

## Still missing before 1.0

- In-app transaction confirmation modal with typed confirmation
- Change-output explanation and stronger coin-selection display
- Automatic post-broadcast resync
- QR code rendering for receive/payment URI
- Real in-app BOLT12 Lightning wallet through LDK on signet/testnet first

# Migration to v0.27 Core Wallet Reset

This version removes the cluttered miner/OCEAN prototype UI and resets the app around the product goals:

- Create a BTC wallet/address
- Receive BTC on-chain
- Prepare/send BTC on-chain
- Create a Lightning wallet work area for BOLT12
- Sign messages

## Main source files now

- `src/main.tsx`: clean four-tab React UI
- `src/styles.css`: simplified app styling
- `src-tauri/src/lib.rs`: simplified Tauri command bridge
- `src-tauri/src/wallet.rs`: Bitcoin wallet, receive address generation, send-draft validation, BIP-322 signing
- `src-tauri/src/storage.rs`: encrypted backup/restore
- `src-tauri/src/core_lightning.rs`: Lightning profile and BOLT12 offer validation scaffold

## Removed from the running app

The old miner dashboards, OCEAN setup bundles, payout estimators, release gates, runbooks, and test harnesses were removed from the main UI and invoke handler.

Old prototype Rust modules were moved into:

`archive/legacy-v0.26-modules/`

They are kept for reference but are no longer compiled by the app.

## Current limitations

- Receiving BTC means generating addresses only. Chain sync and balance detection are next.
- Sending BTC is a validated draft only. Real PSBT build/sign/broadcast is next.
- In-app BOLT12 generation is still locked. External BOLT12 offer validation works.
- Mainnet Lightning is not enabled.

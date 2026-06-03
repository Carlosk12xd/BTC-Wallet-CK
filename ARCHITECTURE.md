# Architecture

## Frontend

- `src/main.tsx`: 4-tab wallet UI.
- `src/styles.css`: visual design.

## Backend

- `src-tauri/src/lib.rs`: Tauri command bridge.
- `src-tauri/src/wallet.rs`: BIP39/BDK wallet creation, restore, receive addresses, send draft validation, BIP-322 signing.
- `src-tauri/src/storage.rs`: encrypted backups and encrypted local wallet persistence.
- `src-tauri/src/core_lightning.rs`: BOLT12 placeholder/profile layer.

## v0.28 storage

The saved encrypted wallet file is stored at:

`~/.carlosk-wallet/wallet.encrypted.json`

The passphrase is never saved by the app.

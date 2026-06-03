# Architecture

## Frontend

React + TypeScript with four tabs:

- Wallet
- Send / Receive
- Lightning
- Signatures

## Backend

Tauri + Rust. v0.27 only wires the core modules into the running app:

- `wallet.rs`
- `storage.rs`
- `core_lightning.rs`

Older miner/OCEAN prototype modules may still exist in the source tree temporarily, but they are no longer part of the main UI or invoke handler.

## Security boundary

The app must never send seed phrases to a server. Signing stays local. Lightning mainnet stays locked until testnet/signet recovery is proven.

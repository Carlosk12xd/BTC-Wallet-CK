# Architecture

## Product core

- React/TypeScript UI
- Tauri desktop shell
- Rust backend
- BDK wallet for BTC wallet creation, restore, sync, PSBT signing, and raw transaction broadcast
- BIP-322 message signing for OCEAN
- Core Lightning connector for online BOLT12 offer generation

## v0.99.6 Lightning implementation

`src-tauri/src/core_lightning.rs` now supports:

- validating BOLT12 offer strings,
- testing a Core Lightning REST/JSON-RPC connection,
- generating an OCEAN BOLT12 offer through that node,
- building OCEAN setup instructions for the current BTC wallet address.

The app does not store the node rune/token permanently in v0.99.6.

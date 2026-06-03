# Architecture v0.90

## Frontend

- React + TypeScript
- Wallet, Send/Receive, Lightning, and Signatures tabs
- Frontend mempool.space address lookup for display
- Raw transaction broadcast UI

## Backend

- Rust/Tauri commands
- BDK wallet for descriptors, address generation, signing, and PSBT/final transaction creation
- BDK Esplora for backend wallet sync
- Encrypted wallet persistence with Argon2 + AES-256-GCM
- BIP-322 message signing

## Send flow

Backend sync updates the BDK wallet state. The send command builds a transaction using BDK, signs it locally, finalizes the PSBT, extracts the raw transaction, and returns the hex for user review/broadcast.

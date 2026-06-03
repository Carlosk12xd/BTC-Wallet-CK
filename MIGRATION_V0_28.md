# Migration to v0.28 Real Wallet Foundation

v0.28 keeps the clean 4-tab UI from v0.27 and begins implementing real wallet behavior.

## What changed

- Wallet creation now supports `bitcoin`, `testnet`, and `signet`.
- Wallet restore requires choosing the correct network.
- Receive addresses are generated for the selected network.
- The encrypted backup now stores the network and next receive index.
- The wallet can be saved to disk as an encrypted file.
- The saved file can be loaded after restart with the passphrase.
- Backups can be verified against the currently loaded wallet.
- Send draft validation rejects addresses from the wrong network.

## Local encrypted wallet file

The saved encrypted wallet file is stored at:

```text
~/.carlosk-wallet/wallet.encrypted.json
```

The passphrase is not stored.

## Before using again

Run:

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

## Still not implemented

- Chain sync
- Balance detection
- UTXO listing
- PSBT building
- Transaction signing for broadcast
- Transaction broadcast
- In-app BOLT12 receiving

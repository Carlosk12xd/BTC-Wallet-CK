# CarlosK Wallet Roadmap

## v0.27 current

Core wallet reset. The app now focuses only on:

- BTC wallet creation/restoration
- BTC receive addresses
- BTC send draft validation
- Lightning/BOLT12 work area
- BIP-322 signatures

## v0.28 target

Implement real on-chain wallet sync and balances:

- Add Esplora or Electrum backend configuration.
- Sync wallet UTXOs.
- Show confirmed/unconfirmed balance.
- Show transaction history.

## v0.29 target

Implement real BTC send flow:

- Build PSBT with BDK.
- Show fee and recipient confirmation screen.
- Sign transaction locally.
- Broadcast through Esplora/Electrum.

## Lightning/BOLT12 target

- Start LDK Node on signet/testnet.
- Generate a real signet/testnet BOLT12 offer.
- Receive a test payment.
- Prove backup/restore.
- Only then consider mainnet embedded BOLT12.

# CarlosK Wallet Project Plan

## Product definition

CarlosK Wallet is a desktop self-custody wallet with four jobs:

1. BTC on-chain wallet.
2. On-chain send and receive.
3. Lightning wallet with BOLT12 receiving.
4. Message signatures.

## Removed from product scope

Mining dashboards, miner ROI tools, OCEAN setup bundles, payout estimators, runbooks, and release gates are no longer part of the main app.

## Implementation order

1. Clean wallet UI and local wallet creation.
2. Encrypted backup and restore.
3. On-chain receive addresses.
4. Real balance sync.
5. Real send/broadcast.
6. BIP-322 signatures.
7. LDK signet/testnet BOLT12.
8. Mainnet Lightning only after recovery tests.

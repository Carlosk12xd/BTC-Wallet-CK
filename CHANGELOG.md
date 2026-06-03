# Changelog

## v0.80.0

- Jumped from v0.28 to v0.80 to reflect the Core Wallet MVP direction.
- Added real chain sync through public mempool.space Esplora APIs from the frontend.
- Added real balance detection for the active receive address.
- Added UTXO list.
- Added recent transaction history.
- Added fee estimate lookup.
- Added Bitcoin URI generation.
- Added raw signed transaction broadcast.
- Persisted external BOLT12 offer in local browser storage.
- Kept automatic wallet-built sends locked until PSBT creation/signing is implemented safely.
- Kept in-app BOLT12 locked until LDK signet/testnet receive and recovery work.

## v0.28.0

- Added encrypted local wallet persistence.
- Added backup export/restore verification.
- Added network-aware wallet creation and receive addresses.

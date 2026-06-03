# Changelog

## v0.90.0

- Added backend BDK/Esplora wallet sync command.
- Added local wallet transaction creation and signing.
- Added signed transaction result UI with txid, raw transaction hex, fee, amount, recipient, and finalized status.
- Added button to load signed transaction hex into the broadcast box.
- Added optional custom Esplora API URL field.
- Added `bdk_esplora` dependency for real backend wallet sync.
- Updated wallet warnings to reflect real signed transaction creation.

## v0.80.0

- Added public Esplora address lookup in frontend.
- Added balance detection, UTXOs, transaction history, fee estimates, raw transaction broadcast, and external BOLT12 persistence.

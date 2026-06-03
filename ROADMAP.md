# Roadmap

## v0.90 status

The on-chain wallet is now close to MVP:

- receive address generation: implemented
- encrypted wallet persistence: implemented
- frontend chain lookup: implemented
- backend BDK/Esplora sync: implemented
- local signed transaction creation: implemented
- raw transaction broadcast: implemented

## v0.95 target

- Add safer confirmation modal before broadcast.
- Add automatic post-broadcast resync.
- Add send transaction review screen with full outputs and fee.
- Add QR codes for receiving.
- Add transaction export/import helpers.

## v1.0 target

A simple self-custody Bitcoin desktop wallet with safe create/restore, receive, sync, send, broadcast, backup, and signatures.

## After v1.0

Continue BOLT12/LDK work on signet/testnet before mainnet Lightning.

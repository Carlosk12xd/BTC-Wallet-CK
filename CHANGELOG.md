# Changelog

## v0.27.0

- Reset the app around the actual product scope: Bitcoin wallet, send/receive, Lightning/BOLT12, and signatures.
- Replaced the cluttered prototype UI with four tabs: Wallet, Send / Receive, Lightning, Signatures.
- Removed miner dashboards, OCEAN setup flows, payout estimators, release gates, and long MVP checklists from the main UI.
- Added fresh BTC receive-address generation.
- Added send-draft validation UI. Real transaction build/sign/broadcast is intentionally still disabled.
- Added simplified Lightning wallet profile UI.
- Added external BOLT12 offer validation/saving in the Lightning tab.
- Kept in-app BOLT12 generation locked until LDK signet/testnet receive and backup tests are implemented.
- Kept BIP-322 Simple message signing.

## v0.26.0

- Added LDK signet/testnet runtime acceptance fixture.

## v0.25.0

- Added OCEAN dashboard verifier and real miner evidence checklist.

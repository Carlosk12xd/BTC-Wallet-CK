# Changelog

## v0.99.6

- Added online Core Lightning connector.
- Added backend command `connect_core_lightning_node`.
- Added backend command `create_ocean_bolt12_offer`.
- Added backend command `build_ocean_setup_plan`.
- Added OCEAN-specific BOLT12 generation using amount `any` and description `OCEAN Payouts for <btc-address>`.
- Updated Lightning + OCEAN UI to focus on three steps: connect node, generate offer, sign OCEAN message.
- Kept embedded LDK mainnet Lightning locked until channel state backup/recovery is solved.

## v0.99.5

- Simplified UI to BTC Wallet and Lightning + OCEAN.
- Preserved core BTC wallet, signing, send, receive, and external BOLT12 compatibility.

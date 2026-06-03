# BTC Wallet CK v0.99.6 Online Lightning Core

BTC Wallet CK is a simple self-custody Bitcoin desktop wallet focused on four jobs:

1. Create/import a BTC wallet.
2. Hold, receive, sync, sign, and send BTC.
3. Connect to an online Lightning node that can generate BOLT12 offers.
4. Complete OCEAN Lightning payout setup without Sparrow or Lexe.

## What changed in v0.99.6

This version adds the first real implementation step toward replacing Lexe:

- Core Lightning REST/JSON-RPC connector.
- Node connection test.
- OCEAN-specific BOLT12 offer generation through the connected Lightning node.
- Required OCEAN description builder: `OCEAN Payouts for <btc-address>`.
- OCEAN setup plan with config/stats links.
- OCEAN message signing with the BTC wallet.

The app can now replace Sparrow for signing and can generate the BOLT12 offer if the user connects a real online Core Lightning node.

## Important Lightning requirement

A BOLT12 offer alone is not enough. The Lightning node must remain online and have inbound liquidity. If it cannot receive, OCEAN Lightning payouts can fail and OCEAN will fall back to on-chain payout behavior when eligible.

## Run

```bash
npm install --no-audit --no-fund --registry=https://registry.npmjs.org/
npm run frontend:build
npm run check:tauri
npm run dev
```

## Core Lightning connection

In the Lightning + OCEAN tab, enter:

- Core Lightning REST / JSON-RPC URL, for example `http://127.0.0.1:3010`
- Rune/API token if your node requires one
- Lightning alias

Then click **Test Node Connection** and **Generate OCEAN BOLT12 From Node**.

## Security

Do not paste seed words, private keys, or wallet backup passphrases into OCEAN, a Lightning node dashboard, or GitHub issues. The node token/rune should be treated like a sensitive credential.

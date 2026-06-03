# Contributing to CarlosK Wallet

CarlosK Wallet is intended to become an open-source desktop wallet for Bitcoin home miners.

## Before contributing

Read:

- `README.md`
- `PROJECT_PLAN.md`
- `ARCHITECTURE.md`
- `SECURITY.md`

## Good first contributions

- Improve miner profile descriptions.
- Add unit tests for OCEAN worker label sanitization.
- Add UI validation for BTC addresses.
- Add screenshots to README.
- Improve copywriting for backup warnings.
- Create setup guides for Avalon Mini 3 and FutureBit Apollo III.

## Development setup

```bash
npm install
npm run dev
```

You also need Rust stable and Tauri prerequisites for your operating system.

## Security rules for contributors

Do not ask users to upload or paste seed phrases in GitHub issues.

Do not add telemetry that sends addresses, xpubs, seeds, signatures, or wallet metadata to a server.

Do not add custodial behavior without a separate design and legal/security review.

## Pull request checklist

- Explain what changed.
- Explain security impact.
- Add tests where possible.
- Do not include real seeds, private keys, or personal wallet addresses in tests.
- Keep features local-first.

# Security

## Wallet security

- The BTC wallet seed can be shown for backup, but should not be pasted into websites.
- Encrypted wallet files are protected by the user's passphrase.
- Losing the passphrase means the app cannot decrypt the saved wallet.

## Lightning security

- The Core Lightning rune/API token is sensitive.
- v0.99.6 keeps node credentials in the UI session and does not permanently store them.
- The Lightning node must stay online and have inbound liquidity to receive OCEAN payouts.

## OCEAN security

OCEAN setup should only require:

1. A BOLT12 offer.
2. The unsigned OCEAN configuration message.
3. A signature from the BTC wallet address.

Never give OCEAN or any website seed words, private keys, or backup passphrases.

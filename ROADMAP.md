# Roadmap

## v0.99.6 current

The app now has a real online Lightning path through Core Lightning. This is the practical bridge toward the original goal: replace Lexe for BOLT12 offer generation and replace Sparrow for OCEAN message signing.

## Remaining before 1.0

- Test the Core Lightning connector against a real node.
- Confirm the offer response format for the user's node setup.
- Add safer credential storage for node URL/rune if the user wants persistence.
- Add Lightning receive/payout history once a stable node API is confirmed.
- Add a guided OCEAN setup screen that can verify the offer description if decoding support is added.

## Long-term

Replace the external Core Lightning node requirement with an embedded LDK node only after channel backup, recovery, inbound liquidity, and mainnet safety are proven.

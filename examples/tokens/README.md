# Token Examples

This category contains examples related to fungible tokens, including implementations of Stellar-native standards and common token-related patterns.

## What's Inside?

- **Token Standards**: Implementations of official Stellar token standards like SEP-41.
- **[Mint/Burn Token](./mint-burn/)**: Admin-controlled minting and user burn flows with supply cap handling.
- **[Token Wrapper](./token-wrapper/)**: A 1:1 wrapper around an existing token with deposit, withdraw, backing checks, and invariant tests.
- **[Token Optimization](./optimized-token-ops/)**: Batched transfer and storage-layout optimization patterns with before/after benchmarks.
- **Distribution Patterns**: Examples of vesting schedules and airdrop contracts.

## Examples

- `01-sep41-token`: A minimal SEP-41-compliant fungible token contract.
- `02-vesting-contract`: A contract that releases tokens to a beneficiary over time.
- `03-airdrop-contract`: A contract to efficiently distribute tokens to a list of addresses.
- `04-wrapped-asset`: A contract that creates a Soroban-native representation of a classic Stellar asset.

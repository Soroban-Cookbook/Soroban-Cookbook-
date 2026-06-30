# Token Examples

This category contains examples related to fungible tokens, including implementations of Stellar-native standards and common token-related patterns.

## What's Inside?

- **Token Standards**: Implementations of official Stellar token standards like SEP-41.
- **[Mint/Burn Token](./mint-burn/)**: Admin-controlled minting and user burn flows with supply cap handling.
- **[Allowance Pattern](./allowance-pattern/)**: Delegated spending with `approve`/`transfer_from`, allowance queries, expiration ledgers, and revocation.
- **[Token Wrapper](./token-wrapper/)**: A 1:1 wrapper around an existing token with deposit, withdraw, backing checks, and invariant tests.
- **[Token Optimization](./optimized-token-ops/)**: Batched transfer and storage-layout optimization patterns with before/after benchmarks.
- **Distribution Patterns**: Examples of vesting schedules and airdrop contracts.

## Examples

- `01-sep41-token`: A complete SEP-41 fungible token with metadata, approvals, events, and comprehensive tests.
- `01-sep41-token`: A minimal SEP-41-compliant fungible token contract.
- `02-sep41-extensions`: Optional SEP-41 extensions — permit (EIP-2612 equivalent), batch transfer, and batch approve.
- `02-vesting-contract`: A contract that releases tokens to a beneficiary over time.
- `04-airdrop-contract`: A contract to efficiently distribute tokens to a list of addresses.
- `05-wrapped-asset`: A contract that creates a Soroban-native representation of a classic Stellar asset.

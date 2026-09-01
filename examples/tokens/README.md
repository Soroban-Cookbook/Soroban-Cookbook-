# Token Examples

This category contains examples related to fungible tokens, including implementations of Stellar-native standards and common token-related patterns.

## What's Inside?

- **Token Standards**: Implementations of official Stellar token standards like SEP-41.
- **[Mint/Burn Token](./04-mint-burn/)**: Admin-controlled minting and user burn flows with supply cap handling.
- **[Allowance Pattern](./allowance-pattern/)**: Delegated spending with `approve`/`transfer_from`, allowance queries, expiration ledgers, and revocation.
- **[Token Wrapper](./token-wrapper/)**: A 1:1 wrapper around an existing token with deposit, withdraw, backing checks, and invariant tests.
- **[Snapshot Token](./04-snapshot-token/)**: A fungible token with balance snapshot support for historical/governance voting-power queries.
- **[Token Optimization](./optimized-token-ops/)**: Batched transfer and storage-layout optimization patterns with before/after benchmarks.
- **[Multi-Token Balance Manager](./08-multi-token-balance-manager/)**: A registry for multiple token contracts with batched balance reads and batched transfers.
- **Distribution Patterns**: Examples of vesting schedules (like [Vesting Management](./01-vesting-management/)) and airdrop contracts.

## Examples

- `01-sep41-token`: A minimal SEP-41-compliant fungible token contract.
- `01-vesting-management`: A secure, production-grade token vesting contract with multi-beneficiary support and revocation capabilities.
- `02-minting-strategies`: A token contract showing fixed cap, unlimited, and scheduled issuance patterns.
- `02-vesting-contract`: A contract that releases tokens to a beneficiary over time.
- `04-airdrop-contract`: A contract to efficiently distribute tokens to a list of addresses.
- `04-snapshot-token`: A fungible token contract with balance snapshot support for historical balance queries.
- `05-wrapped-asset`: A contract that creates a Soroban-native representation of a classic Stellar asset.
- `06-reward-token`: A token with multiple independent reward pools; holders earn proportional rewards and claim them on demand.
- `07-token-metadata`: A token with full SEP-41 metadata support (name, symbol, decimals, URI) with admin-governed updates.
- `10-automatic-snapshot-triggers`: Time-based & event-based balance snapshots with pruning.
- [`10-pausable-permissions`](./10-pausable-permissions/): A permission system for pausing — pauser role, multi-sig pause, and time-limited pause.

## Patterns Guide

Before reaching for a specific token example, prefer the consolidated
[Token Patterns](../../docs/token-patterns.md) guide — it synthesizes metadata,
mint/burn, wrapping, storage layout, access control, and event decisions with
concrete do/don't guidance.

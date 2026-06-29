# Multi-Token Balance Manager

This example shows a reusable registry contract for wallets, vaults, and portfolio-style contracts that need to work with many token contracts at once.

The contract stores token addresses with metadata, batches read calls with `batch_balance(user, tokens)`, and batches write calls with `batch_transfer(from, transfers)`.

## What It Demonstrates

- A registry mapping token contract addresses to display metadata.
- Batched balance reads across multiple SEP-41-compatible token contracts.
- Batched token transfers in a single contract call.
- Manual metadata registration for tokens that expose different metadata conventions.
- Guardrails for empty batches, unregistered tokens, and invalid transfer amounts.

## Registry Metadata

Soroban token contracts commonly follow SEP-41, which exposes token behavior through the token interface. In practice, portfolio contracts still need their own display metadata because tokens may come from different sources:

- `Sep41`: token contracts that expose familiar name, symbol, and decimals metadata.
- `StellarAsset`: wrapped Stellar classic assets, where display metadata may come from the asset code and issuer instead of contract methods.
- `Custom`: project-specific tokens whose metadata may be fetched off-chain or normalized before registration.

Register metadata explicitly:

```rust
manager.register_token(
    &admin,
    &token_address,
    &TokenMetadata {
        name,
        symbol,
        decimals: 7,
        standard: MetadataStandard::Sep41,
    },
);
```

The admin can call `register_token` again to update metadata if a display name, decimal precision, or metadata source changes.

## Batched Reads

`batch_balance(user, tokens)` returns a `Vec<TokenBalance>` with the token address, the user's balance, and the registered metadata for each token.

This keeps wallet and vault UIs from needing separate contract calls for every asset in a portfolio.

## Batched Writes

`batch_transfer(from, transfers)` moves several registered tokens from one user to one or more recipients.

The `from` address must authorize the call. Each token must already be registered, and every transfer amount must be positive.

## Testing

Run this example with:

```bash
cargo test -p multi-token-balance-manager
```

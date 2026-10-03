# Cross-Chain Bridge Example

This example demonstrates a cross-chain bridge architecture on Soroban with lock/mint and burn/release patterns, along with a validator system.

## Features

- **Lock/Mint Pattern**: Lock tokens on the source chain, mint wrapped tokens on the destination chain.
- **Burn/Release Pattern**: Burn wrapped tokens on the destination chain, release locked tokens on the source chain.
- **Validator System**: Multi-signature validation for cross-chain transfers.
- **Token Mapping**: Map tokens from different chains to Soroban tokens.

## Usage

### Initialize the Bridge

```rust
let admin = Address::generate(&env);
let validators = Vec::from_array(&env, [validator1, validator2, validator3]);
client.initialize(&admin, &validators, &2u32);
```

### Lock Tokens

```rust
let transfer = client.lock_tokens(
    &sender,
    &symbol_short!("ethereum"),
    &recipient_bytes,
    &token_id,
    &1000i128,
);
```

### Mint Tokens (with Validator Signatures)

```rust
client.mint_tokens(&transfer, &signatures);
```

### Burn Tokens

```rust
let transfer = client.burn_tokens(
    &sender,
    &symbol_short!("soroban"),
    &recipient_bytes,
    &token_id,
    &500i128,
);
```

### Release Tokens (with Validator Signatures)

```rust
client.release_tokens(&transfer, &signatures);
```

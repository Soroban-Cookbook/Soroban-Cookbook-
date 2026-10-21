# AMM Router

A Soroban smart contract implementing an AMM router with multi-hop swap capabilities.

## Features

- Multi-pool routing
- Multi-hop swaps
- Deadline enforcement
- Slippage control
- Pool management

## Usage

### Initialize

```rust
client.initialize();
```

### Add Pool

```rust
client.add_pool(&Pool {
    token_a: token_a_address,
    token_b: token_b_address,
    reserve_a: 1000,
    reserve_b: 1000,
});
```

### Get Pool

```rust
let pool = client.get_pool(&token_a, &token_b);
```

### Swap Exact Tokens for Tokens

```rust
client.swap_exact_tokens_for_tokens(
    &user,
    &amount_in,
    &amount_out_min,
    &path,
    &to,
    &deadline,
);
```

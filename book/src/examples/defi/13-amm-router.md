# 13 · AMM Router

**Source:** [`examples/defi/13-amm-router/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/13-amm-router)

A multi-hop swap router that chains two or more AMM pool contracts to route a swap through intermediate tokens. The caller specifies a `path` of token addresses; the router executes each hop atomically.

## What You'll Learn

- Multi-hop path routing: `token_a → token_b → token_c`
- Computing expected output across multiple pools before executing
- Enforcing a `min_out` slippage guard on the final output only
- Deadline checks to prevent stale transactions from executing

## Key Concepts

| Concept | Detail |
|---------|--------|
| `path: Vec<Address>` | Ordered list of tokens; N tokens = N-1 swaps |
| Output chaining | Output of hop N becomes input of hop N+1 |
| `min_out` | Applied only to the final token received |
| `deadline` | Transaction reverts if `env.ledger().timestamp() > deadline` |

## Quick Code

```rust
// Swap USDC → XLM → WBTC in two hops
let path = vec![usdc, xlm, wbtc];
client.swap_exact_tokens_for_tokens(
    &caller, &100_i128, &min_wbtc, &path, &deadline,
);
```

## Run the Example

```bash
cd examples/defi/13-amm-router
cargo test
```

## See Also

- [02 · Constant-Product AMM](./02-constant-product-amm.md) — the pool contracts this router calls
- [DeFi Overview](../defi.md)

# Constant Product AMM

A Soroban implementation of a Uniswap V2-style constant product automated market maker.

## Features

- `x * y = k` invariant
- Add and remove liquidity in proportion to the current pool reserves
- Swap with price impact and a 0.3% fee
- Internal LP token minting, burning, and balance tracking
- Swap reentrancy mutex held across authorization, reserve reads, and transfers

## Contracts

- `initialize(token_x: Address, token_y: Address)` — configure the pair once
- `add_liquidity(provider: Address, amount_x: i128, amount_y: i128)`
- `remove_liquidity(provider: Address, lp_amount: i128)`
- `swap(trader: Address, sell_token: Address, sell_amount: i128, min_buy_amount: i128)`
- `lp_balance(provider: Address)` — check LP token shares
- `total_supply()` — total LP token supply

## Reentrancy and token assumptions

`swap` acquires an instance-storage mutex (`DataKey::SwapEntered`) before
authorization or any token call, including `balance`. It holds the mutex through
both the input and output transfers. An already-held mutex returns
`AmmError::Reentrancy` without reading reserves or moving tokens. The scoped
`SwapGuard` releases the mutex on successful and error returns; if a token call
traps, Soroban rolls back the invocation, including the mutex and token changes.

Soroban's host currently [disallows contract reentrancy](https://stellar.org/blog/developers/soroban-the-smart-contract-platform-designed-for-developers),
including a token calling back into an active AMM. The mutex makes the intended
critical section explicit as a defensive teaching pattern. Callback tests
exercise the host rejection; a separate active-lock test exercises the AMM's
own `Reentrancy` error.

If the mutex is omitted when adapting this example to an environment that permits
callbacks, a hookable token can re-enter between the reserve reads and transfers.
A nested swap can then price against intermediate reserves and violate the pool's
accounting assumptions. Do not copy that unguarded ordering into such a runtime.
The guard pattern to copy is [Advanced Reentrancy Protection](../../advanced/15-reentrancy-guard/)
(`examples/advanced/15-reentrancy-guard`, formerly referenced as
`examples/advanced/05-reentrancy-guard`). For a port that allows reentrancy, share
the lock across all reserve-changing entry points, including `add_liquidity` and
`remove_liquidity`, and review reads of intermediate state. This example's mutex
protects `swap`; it does not add guards to those liquidity entry points.

This is an educational example, not a production-ready AMM. A mutex does not make
arbitrary token extensions safe: dishonest balances, transfer fees, rebasing,
or other token-side effects can still invalidate the pricing assumptions. Use
reviewed tokens with standard transfer semantics and audit all external-call
boundaries before deployment.

## Build

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release

# From the repository root
cargo build -p constant-product-amm --target wasm32-unknown-unknown --release
```

## Test

```bash
# From this directory
cargo test

# From the repository root
cargo test -p constant-product-amm
```

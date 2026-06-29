# 05 — Batch Transfer

Send tokens to multiple recipients in a single, atomic Soroban transaction.

---

## What it does

The `BatchTransferContract` lets a sender transfer different amounts to up to **100 recipients** in one invocation. Because the sender's balance is read once and written once (rather than once per recipient), batch transfers are significantly cheaper on-chain than looping over individual `transfer` calls.

If any validation check fails — empty batch, invalid amount, or insufficient balance — **the entire batch is rejected** and no state is mutated.

---

## Key concepts

| Concept | Where |
|---------|-------|
| Single-read / single-write gas optimisation | `batch_transfer` in `src/lib.rs` |
| Checked arithmetic on totals and per-recipient credits | `checked_add` calls in `batch_transfer` |
| Atomic all-or-nothing execution | Validation pass before any storage write |
| `#[contracterror]` enum with typed failures | `BatchError` in `src/lib.rs` |
| Event per transfer + batch-complete summary event | `TransferEvent`, `BatchCompleteEvent` |
| `#[contracttype]` for transfer descriptors | `Transfer` struct |

---

## Contract API

### `initialize(admin: Address, initial_supply: i128) → Result<(), BatchError>`

One-time setup. Mints `initial_supply` tokens to `admin`.

Errors: `AlreadyInitialized`, `InvalidAmount`

---

### `batch_transfer(from: Address, transfers: Vec<Transfer>) → Result<u32, BatchError>`

Transfer tokens from `from` to all recipients described in `transfers`.

`from` must authorize the call. Returns the number of recipients credited.

**`Transfer` struct:**

```rust
pub struct Transfer {
    pub to: Address,   // recipient
    pub amount: i128,  // must be > 0
}
```

**Error table:**

| Error | Condition |
|-------|-----------|
| `NotInitialized` | Contract not yet initialized |
| `EmptyBatch` | `transfers` is empty |
| `BatchTooLarge` | More than `MAX_BATCH_SIZE` (100) entries |
| `InvalidAmount` | Any `amount` ≤ 0 |
| `TotalOverflow` | Sum of all amounts overflows `i128` |
| `InsufficientBalance` | Sender balance < total |
| `RecipientOverflow` | A recipient's new balance would overflow `i128` |

---

### `balance(user: Address) → i128`

Returns the current token balance for `user`.

---

## Gas optimisation explained

A naïve implementation of multi-recipient transfers loops and calls a `transfer` function per recipient. Each call reads and writes the sender's balance. For N recipients that is 2N storage operations on the sender's balance alone.

This example reads the sender's balance **once**, validates the full batch, then:
1. Writes the sender's new balance **once** (deducting the total)
2. Reads and writes each recipient's balance exactly once

Total storage ops = `1 read + 1 write` (sender) + `N reads + N writes` (recipients) = `2 + 2N` instead of `2N + 2N = 4N` for naïve looping.

---

## How to build and test

```bash
# Run unit tests
cargo test -p batch-transfer

# Build WASM release binary
cargo build --target wasm32-unknown-unknown --release -p batch-transfer

# Lint
cargo clippy -p batch-transfer --all-targets -- -D warnings
```

---

## Use cases

- **Payroll / airdrop** — distribute stablecoin salaries or token rewards to hundreds of addresses in one transaction
- **DAO reward distribution** — disburse governance participation rewards atomically
- **Revenue splitting** — forward protocol fees to multiple beneficiaries (treasury, team, DAO) in a single call
- **NFT mint proceeds** — split mint revenue across creators and royalty recipients

---

## Related examples

- `examples/advanced/01-multi-party-auth` — authorize an action with multiple signers
- `examples/tokens/01-sep41-token` — full SEP-41 token with allowances
- `examples/tokens/02-sep41-extensions` — permit (gasless approve) and batch operations on a SEP-41 token

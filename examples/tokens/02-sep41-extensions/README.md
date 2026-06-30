# 02 — SEP-41 Extensions

Optional extensions to a standard SEP-41 token contract: **permit** (off-chain
approval), **batch transfer**, and **batch approve**.

---

## What it does

The `Sep41Extensions` contract layers three optional capabilities on top of a
minimal SEP-41 token:

| Extension | Function | Purpose |
|-----------|----------|---------|
| Permit | `permit(owner, spender, amount, expiration_ledger)` | Approve a spender using an off-chain signature instead of an on-chain `approve` call (EIP-2612 equivalent) |
| Batch transfer | `batch_transfer(from, transfers)` | Send different amounts to multiple recipients in one atomic call |
| Batch approve | `batch_approve(owner, approvals)` | Set multiple allowances atomically |

---

## Key concepts

### Permit (EIP-2612 equivalent)

In Ethereum, EIP-2612 allows a token holder to sign an off-chain message that
grants a spender an allowance.  The spender (or a relayer) then submits a
single on-chain transaction that both sets the allowance and executes the
downstream action.

Soroban achieves the same outcome via its built-in auth framework.  The owner
signs an authorisation envelope off-chain (using their Stellar key).  The
spender submits the `permit` invocation with that envelope attached.

`require_auth_for_args` binds the authorisation to the exact
`(spender, amount, expiration_ledger)` arguments, preventing replay with
different parameters.

### Batch transfer gas optimisation

The sender's balance is read **once** and written **once** regardless of the
number of recipients.  This halves the number of sender-side storage operations
compared to looping over individual `transfer` calls.

### Batch approve

Multiple allowances are set in a single contract invocation, saving round-trip
overhead.  Passing `amount = 0` for any spender revokes that allowance.

---

## Contract API

### `initialize(admin: Address, initial_supply: i128) → Result<(), ExtError>`

One-time setup: mints `initial_supply` to `admin`.

---

### `permit(owner, spender, amount, expiration_ledger) → Result<(), ExtError>`

Grant `spender` an allowance of `amount` from `owner` via off-chain signature.
Valid through and including `expiration_ledger`.  Passing `amount = 0` revokes
the allowance.

| Error | Condition |
|-------|-----------|
| `NotInitialized` | Contract not yet set up |
| `InvalidAmount` | `amount < 0` |
| `ExpiredPermit` | `amount > 0` and `expiration_ledger < current_ledger` |

---

### `batch_transfer(from, transfers: Vec<Transfer>) → Result<u32, ExtError>`

Transfer tokens from `from` to all recipients in `transfers` atomically.
Returns the number of recipients credited.

```rust
pub struct Transfer {
    pub to: Address,
    pub amount: i128,  // must be > 0
}
```

| Error | Condition |
|-------|-----------|
| `EmptyBatch` | `transfers` is empty |
| `InvalidAmount` | Any `amount ≤ 0` |
| `InsufficientBalance` | Sender balance < sum of all amounts |
| `ArithmeticOverflow` | Arithmetic overflow detected |

---

### `batch_approve(owner, approvals: Vec<Approval>) → Result<u32, ExtError>`

Set multiple allowances for `owner` in one call. Returns the number of
allowances written.

```rust
pub struct Approval {
    pub spender: Address,
    pub amount: i128,           // 0 to revoke; must be ≥ 0
    pub expiration_ledger: u32, // last ledger the allowance is valid
}
```

---

### `transfer`, `approve`, `transfer_from`, `balance`, `allowance`

Standard SEP-41 functions. `allowance` returns `0` for expired entries.

---

## Error table

| Error | Code | Meaning |
|-------|------|---------|
| `AlreadyInitialized` | 1 | `initialize` called twice |
| `NotInitialized` | 2 | Contract not yet set up |
| `InvalidAmount` | 3 | Amount is zero, negative, or otherwise invalid |
| `InsufficientBalance` | 4 | Sender has fewer tokens than required |
| `InsufficientAllowance` | 5 | Spender's allowance too small |
| `ArithmeticOverflow` | 6 | Overflow detected during arithmetic |
| `EmptyBatch` | 7 | Batch list is empty |
| `ExpiredPermit` | 8 | Permit's expiration ledger is in the past |

---

## How to build and test

```bash
# Run unit tests
cargo test -p sep41-extensions

# Build WASM release binary
cargo build --target wasm32-unknown-unknown --release -p sep41-extensions

# Lint
cargo clippy -p sep41-extensions --all-targets -- -D warnings
```

---

## Use cases

- **Gasless approve flows** — a relayer calls `permit` bundled with
  `transfer_from` in one transaction; the owner never submits a separate
  approve
- **Batch payroll** — fund many addresses in a single call without looping
  on-chain
- **Bulk DeFi allowances** — approve multiple protocols at once before entering
  a complex multi-step position
- **DAO reward distribution** — set allowances for multiple recipients
  atomically, then let each recipient pull their own tokens

---

## Related examples

- `examples/tokens/01-sep41-token` — minimal SEP-41 token with basic approve
  and transfer
- `examples/advanced/05-batch-transfer` — standalone batch transfer contract
  with detailed gas optimisation commentary
- `examples/tokens/allowance-pattern` — allowance lifecycle with expiration

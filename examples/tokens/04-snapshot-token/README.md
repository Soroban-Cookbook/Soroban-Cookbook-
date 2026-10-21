# Snapshot Token

This example contract demonstrates how to implement a fungible token with balance snapshot support in Soroban. A snapshot captures balances (and total supply) at specific points in time, allowing callers to query historical token states.

This pattern is highly popularized by standards like ERC20Snapshot and is critical for decentralized governance (DAOs) and voting systems.

## Why Balance Snapshotting Matters

In decentralized governance, proposal voting weights are typically calculated based on token balances. However, if a voting contract queries balances at the *current* ledger state during a live vote, it is highly vulnerable to:
1. **Flash Loan Attacks**: An attacker borrows a large amount of tokens, votes, and returns the tokens in the same transaction or block.
2. **Double Voting**: A user votes, transfers their tokens to another account, and that account votes again with the same tokens.

By **pinning** voting power to a specific historical `snapshot_id` (created before the proposal is published), the contract ensures that only the tokens held at that specific point in time can be used to vote.

---

## Contract Interface

The `SnapshotToken` contract exposes the following endpoints:

### Base Token Operations
- `initialize(env, admin, name, symbol, decimals)`: Configures the token parameters.
- `mint(env, admin, to, amount)`: Mints new tokens to a recipient (Admin-only).
- `transfer(env, from, to, amount)`: Transfers tokens between accounts.
- `burn(env, from, amount)`: Burns tokens from an account.
- `balance(env, user)`: Returns the current balance of an account.
- `total_supply(env)`: Returns the current total supply.

### Snapshot Operations
- `create_snapshot(env, admin) -> u32`: Increments the snapshot ID counter, records a new snapshot, and returns the new ID. (Admin-only).
- `current_snapshot(env) -> u32`: Returns the current/latest snapshot ID.
- `total_snapshots(env) -> u32`: Returns the total snapshots created.
- `balance_at_snapshot(env, account, snapshot_id) -> i128`: Returns the balance of `account` exactly at the moment the given snapshot was taken.
- `total_supply_at_snapshot(env, snapshot_id) -> i128`: Returns the total supply exactly at the moment the given snapshot was taken.

---

## Sparse-Snapshot Storage Pattern

A naive snapshot implementation might copy the entire ledger of balances whenever `create_snapshot` is called. This is extremely inefficient, scaling as $O(N)$ where $N$ is the number of token holders, and would lead to unsustainable gas costs and storage bloat.

To solve this, `SnapshotToken` implements the **Sparse-Snapshot Pattern**:
- When `create_snapshot` is called, the contract simply increments a global `SnapshotCounter`. No user balances are updated or copied.
- When an account's balance is about to change (via `mint`, `transfer`, or `burn`), the contract checks if a history entry has already been recorded for that account during the *current* snapshot.
- If not, it lazily records the **pre-change** balance and associates it with the current snapshot ID in `DataKey::SnapshotHistory(Account)`.
- Subsequent transfers during the same snapshot period do not trigger further history writes, as the first entry already captured the historical balance at the snapshot boundary.

This ensures that storage growth is strictly bounded by user transaction activity, not the frequency of snapshots.

### Query Resolution Logic

When querying `balance_at_snapshot(account, snapshot_id)`:
1. If the `snapshot_id` is greater than the current snapshot counter or is `0`, the contract returns `SnapshotTokenError::SnapshotNotFound`.
2. The contract retrieves the sorted history array `Vec<(u32, i128)>` for the account.
3. It iterates chronologically to find the *first* entry where `recorded_id >= snapshot_id`. This entry represents the state of the account's balance immediately after the snapshot boundary was crossed.
4. If no such entry exists (i.e. the account had no activity *after* the snapshot was created), the account's balance has not changed since the snapshot time. The contract returns the account's **current** balance.
5. If the account has never held any tokens at all, the contract safely returns `0` rather than throwing an error.

---

## Walkthrough: Governance Flow

The intended governance flow utilizing this token contract works as follows:

1. **Proposal Creation**: A DAO member submits a proposal.
2. **Snapshot Creation**: The DAO admin calls `create_snapshot(admin)` on the token, which returns (for example) snapshot ID `42`.
3. **Voting Phase Begins**: Members cast votes. The voting contract calculates each member's voting power by calling:
   ```rust
   let voting_power = token_client.balance_at_snapshot(&member_address, &42);
   ```
4. **No Double-Voting or Flash-Loans**: Even if a member transfers their tokens to another wallet, or borrows 10,000,000 tokens via a flash loan during the voting period, their historical balance at snapshot `42` remains unchanged, preserving voting integrity.

---

## Design Choices & Storage Growth

- **Access Control on Snapshots**: Creating a snapshot is restricted to the contract `admin` role. In production governance, this admin role is typically owned by a timelock or the DAO contract itself to prevent arbitrary snapshot creation.
- **Storage Limits**: While sparse storage is highly optimized, the history vector of active accounts can grow over time. On Stellar, keeping track of active voters is a standard trade-off for fully on-chain voting. If needed, historical entries can be pruned in more advanced implementations once associated voting proposals are completed.

---

## How to Build and Test

### Prerequisites
Make sure you have Rust and the Soroban CLI toolchain installed.

### Build the Contract
To compile the contract to an optimized WASM file, run:
```bash
cargo build --target wasm32v1-none --release -p snapshot-token
```

### Run Tests
To execute the comprehensive test suite (including sparse logic and boundary edge-cases):
```bash
cargo test -p snapshot-token
```

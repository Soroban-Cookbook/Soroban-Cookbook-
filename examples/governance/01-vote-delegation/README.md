# Vote Delegation Pattern

This example demonstrates how to implement a secure, recursive **Voting Delegation** pattern in Soroban. It is a foundational pattern for Decentralized Autonomous Organizations (DAOs) and on-chain governance systems.

---

## Overview

In governance, users often want to delegate their voting power to a representative (delegatee) who aligns with their views. A robust voting delegation system must support:
1. **Direct Delegation:** User A delegates their voting weight to User B.
2. **Chain Delegation (Liquid Democracy):** User A delegates to B, B delegates to C. C now holds the combined voting power of A, B, and C.
3. **Undelegation:** Users can reclaim their voting power at any time.
4. **Safety Safeguards:**
   - **Self-Delegation Protection:** Users cannot delegate to themselves.
   - **Cycle/Loop Prevention:** If Alice delegates to Bob, Bob cannot delegate to Alice. The contract must reject cyclic delegations that would cause infinite loops.
   - **Depth Limits:** Restricting the maximum length of a delegation chain to prevent out-of-gas errors when walking the tree on-chain.

---

## Project Structure

```text
examples/governance/01-vote-delegation/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs     # Vote delegation contract implementation
    └── test.rs    # Verification tests (cycles, chains, limits)
```

---

## Core Logic & Data Keys

The contract tracks balances and delegations using the following persistent keys:
* `Balance(Address)`: Stores the base voting weight of an address.
* `Delegation(Address)`: Stores who an address delegated their power to.
* `Delegators(Address)`: A list (`Vec<Address>`) of addresses that delegate directly to a given address. This allows the contract to efficiently traverse downward to compute the total voting power of any delegate.

### Dynamic Voting Power Formula

The active voting power of an address is calculated as:
* If the address **has active delegation**, its active voting power is **`0`**.
* If the address **does not have active delegation**, its active voting power is:
  $$\text{Voting Power} = \text{Base Balance} + \sum_{d \in \text{Delegators}} \text{calculate\_power}(d)$$

---

## How to Build

```bash
# Build the contract
cargo build -p vote-delegation --target wasm32-unknown-unknown --release
```

---

## How to Test

```bash
# Run the unit tests
cargo test -p vote-delegation
```

The tests cover:
- Admin setting user base balances.
- Delegating voting power from one user to another.
- Multi-level chain delegation (Liquid Democracy).
- Cycle detection (direct and indirect loops).
- Self-delegation error throwing.
- Reclaiming power via `undelegate`.
- Restricting delegation depth to prevent gas exhaustion (e.g. max depth of 5).

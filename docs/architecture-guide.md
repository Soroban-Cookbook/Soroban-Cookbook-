# Architecture Guide

A practical guide for making sound architectural decisions when building Soroban smart contracts. Covers pattern selection, storage strategies, access-control models, scalability trade-offs, and maintainability tips with real-world examples from this cookbook.

---

## Table of Contents

1. [Pattern Selection Guide](#1-pattern-selection-guide)
2. [Storage Architecture](#2-storage-architecture)
3. [Access-Control Models](#3-access-control-models)
4. [Upgradability Strategies](#4-upgradability-strategies)
5. [Scalability Considerations](#5-scalability-considerations)
6. [Maintainability Tips](#6-maintainability-tips)
7. [Real-World Examples](#7-real-world-examples)

---

## 1. Pattern Selection Guide

Use this decision tree to select the right architectural pattern before you write a single line of code.

### Authorization

| Requirement | Pattern | Example |
|-------------|---------|---------|
| Single admin | `admin.require_auth()` | `examples/tokens/04-mint-burn` |
| M-of-N signers | Multi-party auth with threshold | `examples/advanced/01-multi-party-auth` |
| Role-based (owner/minter/pauser) | RBAC with role bitmap | `examples/advanced/03-rbac-modifiers` |
| Delegated spending | Allowance pattern | `examples/tokens/01-sep41-token` |
| Gasless UX | Permit / trusted forwarder | `examples/advanced/03-permit-pattern` |

### Execution Timing

| Requirement | Pattern | Example |
|-------------|---------|---------|
| Delayed actions | Timelock | `examples/advanced/02-timelock` |
| Rate-limited calls | Rate limiting | `examples/advanced/05-rate-limiting` |
| Emergency stop | Circuit breaker / pausable | `examples/tokens/03-pausable-token` |

### Data Aggregation / Pricing

| Requirement | Pattern | Example |
|-------------|---------|---------|
| On-chain price feeds | Oracle pattern | `examples/advanced/03-oracle-pattern` |
| Off-chain data relay | Gasless relayer | `examples/advanced/03-gasless-relayer` |
| Historical balances | Snapshot token | `examples/tokens/04-snapshot-token` |

### Upgradability

| Requirement | Pattern | Example |
|-------------|---------|---------|
| Minimal proxy | Beacon proxy | `examples/advanced/02-beacon-proxy` |
| Logic separation | Proxy + admin | `examples/advanced/04-upgradeable-proxy` |
| Modular extensions | Diamond pattern | `examples/advanced/06-diamond-pattern` |

---

## 2. Storage Architecture

Soroban offers three storage tiers. Choosing the wrong tier is the most common cause of unexpected ledger-entry expiry and wasted fees.

### Tier Comparison

| Tier | Persists Until | Cost | Use For |
|------|----------------|------|---------|
| `instance` | Contract is live | Cheapest per-call | Admin, config, counters |
| `persistent` | Manually renewed or expired | Moderate | Per-user balances, allowances |
| `temporary` | Fixed TTL, never renewed | Cheapest per-byte | Nonces, session data |

### Key Design Rules

1. **Never store per-user data in instance storage.** Instance storage is loaded on every contract call. Storing `Balance(Address)` there makes every invocation pay for the entire user map.

2. **Use `persistent` for user-keyed data.** Each `Balance(addr)` is an independent ledger entry; only the accessed entry is loaded.

3. **Compound keys must be canonical.** When a key involves multiple addresses (e.g., `Allowance(owner, spender)`), use a deterministic ordering to avoid key-collision bugs.

4. **Avoid large `Vec` in instance storage.** A growing list in instance makes every call more expensive. Prefer a counter + keyed entries (`Pool(id)`) over `Vec<Pool>`.

### Example: Multi-pool Storage Layout

```rust
#[contracttype]
pub enum DataKey {
    Admin,           // instance â€” single entry, always needed
    PoolCount,       // instance â€” cheap counter
    Pool(u32),       // persistent â€” only loaded on demand
    Balance(Address),// persistent â€” per-user, loaded on demand
    Claimed(Address, u32), // persistent â€” per-user-per-pool
}
```

---

## 3. Access-Control Models

### Single Admin

Simplest model. One address can perform privileged operations.

```rust
fn read_admin(env: &Env) -> Result<Address, Error> {
    env.storage().instance().get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

// In any privileged function:
let admin = read_admin(&env)?;
admin.require_auth();
```

**When to use:** Minting, parameter changes, emergency controls where a single trusted party suffices.

### Role-Based Access Control (RBAC)

Assign roles (minter, pauser, upgrader) to separate addresses. Each role is stored as an `Option<Address>`.

**When to use:** DeFi protocols, tokens with separate operator roles, DAOs with specialized committees.

**Example:** `examples/advanced/03-rbac-modifiers`

### Multi-Party Authorization (M-of-N)

A proposal requires `threshold` out of `N` registered signers to approve before it executes.

**When to use:** Treasury management, protocol upgrades, cross-org agreements.

**Example:** `examples/advanced/01-multi-party-auth`

### Comparison

| Model | Complexity | Flexibility | Gas Cost |
|-------|-----------|-------------|----------|
| Single admin | Low | Low | Minimal |
| RBAC | Medium | High | Low |
| M-of-N | High | Very high | O(N) per proposal |

---

## 4. Upgradability Strategies

Soroban contracts are immutable by default. The following patterns allow controlled upgradability.

### Proxy Pattern

Deploy a thin proxy that delegates calls to an implementation contract. Upgrade by pointing the proxy to a new implementation.

- **Pros:** Preserves contract address; storage survives upgrades.
- **Cons:** Proxy overhead on every call; storage layout must remain compatible.

**Example:** `examples/advanced/04-upgradeable-proxy`

### Beacon Proxy

Multiple proxy instances share one beacon that holds the implementation address. Upgrading the beacon upgrades all proxies atomically.

- **Pros:** Single upgrade operation for many instances (e.g., token factory).
- **Cons:** Beacon is a single point of failure; all instances upgrade simultaneously.

**Example:** `examples/advanced/02-beacon-proxy`

### Diamond Pattern (Multi-Facet)

A single contract address routes to many facet contracts based on function selector. Each facet can be upgraded independently.

- **Pros:** Modular; bypass contract size limits; granular upgrades.
- **Cons:** High complexity; careful storage layout management required.

**Examples:** 
- **Canonical:** [`examples/advanced/06-diamond-pattern`](../examples/advanced/06-diamond-pattern/) â€” Full EIP-2535 adaptation with diamond-cut and loupe
- **Security-focused:** [`examples/advanced/05-diamond-security`](../examples/advanced/05-diamond-security/) â€” Hardened with access controls and interface verification
- **Router orchestration:** [`examples/advanced/05-diamond-facets`](../examples/advanced/05-diamond-facets/) â€” Atomic cross-facet operations

### No-Upgrade (Immutable)

Deploy once, never upgrade. Use migration contracts to move state if needed.

- **Pros:** Maximum trustlessness; simplest to audit.
- **Cons:** Bugs cannot be fixed; requires user migration.

**When to prefer immutability:** Escrow contracts, simple token wrappers, any contract where trust matters more than agility.

---

## 5. Scalability Considerations

### Storage Footprint

- Each `persistent` ledger entry has a base fee. Minimize entry count by batching where safe.
- Avoid storing full history on-chain. Use events for indexers and keep only current state.

### Compute Budget

Soroban enforces a per-invocation CPU instruction budget. Profile expensive operations:

```bash
# Use the Soroban environment budget in tests
env.budget().reset_default();
contract.some_operation(...);
let cpu = env.budget().cpu_instruction_cost();
let mem = env.budget().memory_bytes_cost();
```

### Cross-Contract Calls

Each cross-contract call incurs overhead. Batch calls where possible using `batch-operations` patterns:

**Example:** `examples/advanced/39-batch-operations`

### Event-Driven Off-Chain Processing

Heavy aggregation (leaderboards, analytics) belongs off-chain. Emit structured events and let indexers compute:

```rust
env.events().publish((NS, EV_TRANSFER, from, to), amount);
```

**Example:** `examples/intermediate/event-aggregation`

---

## 6. Maintainability Tips

### Use Typed Error Enums

Always use `#[contracterror]` with explicit discriminants. This makes error codes stable across upgrades.

```rust
#[contracterror]
#[repr(u32)]
pub enum MyError {
    AlreadyInitialized = 1, // never renumber
    NotInitialized     = 2,
    Unauthorized       = 3,
}
```

### Separate Storage Keys from Business Logic

Define all storage keys in one `DataKey` enum. This makes it trivial to audit what state a contract touches and prevents key collisions between modules.

### Guard Initialization

Every contract should have an `AlreadyInitialized` guard:

```rust
if env.storage().instance().has(&DataKey::Admin) {
    return Err(Error::AlreadyInitialized);
}
```

### Prefer `checked_` Arithmetic

Use `checked_add` / `checked_sub` and map `None` to an `ArithmeticOverflow` error instead of relying on Rust's debug-mode panics:

```rust
let new_balance = balance
    .checked_add(amount)
    .ok_or(Error::ArithmeticOverflow)?;
```

### Keep Events Structured

Emit events with typed payloads (a `#[contracttype]` struct) rather than raw values. Structured events are easier to decode and index.

### Write Tests First

Use `soroban_sdk::Env::default()` + `mock_all_auths()` for unit tests. Cover:
- Happy path
- Every error variant
- Boundary values (0 amount, max amount, empty strings)

---

## 7. Real-World Examples

### Token with Metadata (SEP-41 + metadata extensions)

Full token lifecycle with name, symbol, decimals, URI, admin-controlled metadata updates, mint, burn, and transfer.

**Location:** `examples/tokens/07-token-metadata`

**Key architectural decisions:**
- `decimals` stored immutably (changing them would silently reinterpret balances)
- `name`, `symbol`, `uri` are mutable by admin with event emission
- All balances in `persistent` storage; metadata in `instance` storage

### Reward Token with Multiple Pools

A token that supports multiple independent reward pools. Users accumulate rewards proportional to their balance at claim time.

**Location:** `examples/tokens/06-reward-token`

**Key architectural decisions:**
- Pool metadata (`rate_per_token`, `total_deposited`) in `persistent` storage keyed by pool ID
- Per-user claim tracking in `persistent` storage keyed by `(user, pool_id)`
- Reward formula: `balance * rate_per_token / 1_000_000` â€” fixed-point, no floating point

### Multi-Party Auth Treasury

A treasury contract that requires M-of-N signers to release funds. Uses proposal-based auth with a configurable threshold.

**Location:** `examples/advanced/01-multi-party-auth`

**Key architectural decisions:**
- Proposals stored with sorted, deduplicated signer lists
- Threshold enforced at execution time
- Expiry on proposals prevents replay of stale approvals

### Timelock Governance

Enforces a mandatory delay between proposal creation and execution, giving stakeholders time to react.

**Location:** `examples/advanced/02-timelock`

**Key architectural decisions:**
- Proposals keyed by hash to prevent replay
- Minimum delay enforced at the contract level, not off-chain
- Cancellation only by proposer or admin

---

*For deeper coverage of individual patterns see [`common-patterns.md`](./common-patterns.md), [`advanced-patterns.md`](./advanced-patterns.md), and [`best-practices.md`](./best-practices.md).*

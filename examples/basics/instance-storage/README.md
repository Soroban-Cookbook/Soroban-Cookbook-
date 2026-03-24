# Instance Storage Example

Demonstrates `env.storage().instance()` — the middle ground between persistent and temporary storage in Soroban.

## What is Instance Storage?

Instance storage is scoped to the **contract instance** (the deployed address). All keys in instance storage share a **single TTL** (Time To Live) that covers the entire instance. This differs from persistent storage, where each key has its own independent TTL.

## Comparison with Other Storage Types

| Property             | Persistent           | Instance          | Temporary     |
| :------------------- | :------------------- | :---------------- | :------------ |
| **Survives Upgrade** | ✅ Yes               | ❌ No             | ❌ No         |
| **TTL Management**   | Per-key              | Per-instance      | Per-key       |
| **Relative Cost**    | Highest              | Medium            | Lowest        |
| **Use Case**         | Critical / long-term | Instance-lifetime | Single-ledger |

## When to Use Instance Storage

Choose **Instance Storage** when:

- The data is important during the life of the instance but does _not_ need to outlive a contract upgrade (e.g., a transaction counter).
- You want cheaper rent than persistent while still keeping data across calls.
- You're managing shared state that should expire with the instance as a whole.

Avoid **Instance Storage** when:

- The data **MUST** survive a `upgrade()` call (use **Persistent** instead).
- The data is only needed for a single invocation (use **Temporary** instead).

## Implementation Details

### Key Pattern

Using a typed enum for storage keys is a best practice to avoid collisions and keep the storage surface explicit.

```rust
#[contracttype]
#[derive(Clone)]
pub enum InstanceKey {
    TxCounter,
    Config(Symbol),
}
```

### TTL Management

Instance storage shares a single TTL for all entries. Calling `extend_ttl` on the instance refreshes the lifetime of _all_ instance keys at once.

```rust
const TTL_THRESHOLD: u32 = 1_000;
const TTL_EXTEND_TO: u32 = 10_000;

env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
```

## Use Cases in this Example

1.  **Transaction Counter**: A classic candidate for instance storage. It's per-instance state, changes often (benefiting from lower costs), and doesn't strictly need to survive upgrades.
2.  **Runtime Configuration**: Operator-tunable parameters (like fee rates or limits) that are shared across all invocations but can be reset if the contract is upgraded.

## Running the Example

### 1. Build the Contract

```bash
cargo build --target wasm32-unknown-unknown --release
```

### 2. Run Tests

```bash
cargo test
```

## Lessons Learned

- Instance storage is ideal for shared "instance-global" state.
- Single TTL management significantly simplifies housekeeping compared to persistent storage.
- Always call `extend_ttl` during both reads and writes to ensure the instance doesn't expire while in use.

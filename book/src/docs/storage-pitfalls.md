# Storage Pitfalls in Soroban Contracts

Soroban has no `mapping` and no `storage` keyword. Every byte a contract keeps
lives in one of **three** storage namespaces with **three different lifetimes**
and **three different costs**. Almost every storage bug in a Soroban contract is
one of three things: the wrong namespace, the wrong key, or the wrong time to
live (TTL) handling.

This page lists the mistakes that ship, the symptom each one produces, and the
fix. Use it together with [Storage Patterns](../examples/storage-patterns.md),
[Common Pitfalls](./common-pitfalls.md), and
[Security Pitfalls](./security-pitfalls.md).

---

## Table of Contents

1. [The three storage types](#1-the-three-storage-types)
2. [Choosing the right type](#2-choosing-the-right-type)
3. [Storage type confusion](#3-storage-type-confusion)
4. [Key design and key collisions](#4-key-design-and-key-collisions)
5. [TTL, rent, and liveness](#5-ttl-rent-and-liveness)
6. [Bugs that only appear at expiration](#6-bugs-that-only-appear-at-expiration)
7. [Migration problems](#7-migration-problems)
8. [Cost and performance traps](#8-cost-and-performance-traps)
9. [Best practices](#9-best-practices)
10. [Review checklist](#10-review-checklist)

---

## 1. The three storage types

| | Persistent | Instance | Temporary |
| --- | --- | --- | --- |
| Namespace | `env.storage().persistent()` | `env.storage().instance()` | `env.storage().temporary()` |
| Lifetime | Until the entry's TTL expires | Until the contract instance's TTL expires | Minutes; restored at ledger close |
| TTL granularity | Per key | Shared across all instance entries | Per key, short |
| Intended use | User balances, positions, allowances | Contract configuration | Locks, caches, nonces |
| Relative rent | Highest | Medium | Lowest |
| Survives contract upgrade | Yes | Yes (instance survives a WASM update) | No |

Key facts that cause most of the bugs below:

* **Persistent** entries each carry their own TTL. When a persistent entry
  expires, the contract is *restored* on the next invocation that touches it, and
  the entry has to pay the rent again.
* **Instance** entries share a single TTL per key that is linked to the contract
  instance. Instance storage is *also* the account of the contract itself, so
  bumping the contract instance TTL and bumping an instance entry are separate
  operations.
* **Temporary** entries are cheap precisely because they are short lived. They
  are restored during ledger close, they have a hard minimum TTL enforced by the
  protocol, and they are always cheaper per byte than persistent entries.

---

## 2. Choosing the right type

```
Does the value need to survive beyond a few ledgers?
├── yes → Does it belong to one user / position?
│         ├── yes → persistent, one entry per key, bump TTL on read and write
│         └── no  → instance (contract-wide config), keep it small
└── no  → temporary (reentrancy guard, per-transaction nonce, scratch value)
```

Rules of thumb:

* Balances, allowances, positions, votes, and metadata URIs: **persistent**.
* Admin address, fee basis points, oracle address, pause flag, token address:
  **instance**.
* Reentrancy guard flag, "already used" nonces, transient computation results:
  **temporary**.

---

## 3. Storage type confusion

### 3.1 Balances in temporary storage

```rust
// BROKEN: the balance disappears at ledger close / when the entry expires
pub fn deposit(env: Env, user: Address, amount: i128) {
    let key = DataKey::Balance(user.clone());
    env.storage().temporary().set(&key, &amount);
}
```

**Symptom:** deposits succeed, then reads return zero. Users lose funds; the
integration tests pass because they never advance a ledger.

**Fix:** use persistent storage and bump the TTL.

### 3.2 Configuration in persistent storage

```rust
// WORKS, but pays persistent rent for a value read on every call
pub fn set_admin(env: Env, admin: Address) {
    env.storage().persistent().set(&DataKey::Admin, &admin);
}
```

**Fix:** instance storage. The admin is contract-wide state, so it belongs in the
instance namespace and is loaded with the rest of the instance configuration.

### 3.3 Instance storage used as a database

```rust
// BROKEN: a map grows inside the instance namespace
env.storage().instance().set(&DataKey::Users, &users_vec);
```

**Fix:** one persistent entry per user.

```rust
env.storage().persistent().set(&DataKey::User(user.clone()), &user_record);
```

Instance storage is intended for configuration that is *small and bounded*.
Growing it increases the cost of every invocation and eventually makes the
contract unusable.

### 3.4 Persistent storage used as a cache

```rust
// EXPENSIVE: persistent rent for data that is only needed right now
let key = DataKey::Price(feed.clone());
env.storage().persistent().set(&key, &price);
```

**Fix:** keep the cache in temporary storage, or keep it in memory for the
duration of the call. Persistent storage should hold state you cannot recompute.

### 3.5 Same key name in two namespaces

```rust
env.storage().persistent().set(&symbol_short!("config"), &config);
env.storage().instance().set(&symbol_short!("config"), &other_config);
```

**Fix:** never rely on the *name* to disambiguate. Use a `DataKey` enum whose
variants are explicit, and never reuse one variant for two different concepts.

### 3.6 Reading from the wrong namespace after an upgrade

An upgrade that moves a value from one namespace to another looks fine in tests
(fresh state) and fails in production (existing state).

**Fix:** when a release changes namespaces, ship a migration that reads from the
old namespace and writes to the new one, and gate it behind a version marker.

---

## 4. Key design and key collisions

### 4.1 Ad-hoc keys instead of an enum

```rust
// FRAGILE: two features can both decide that "fee" is a good key name
env.storage().instance().set(&Symbol::new(&env, "fee"), &fee);
env.storage().instance().set(&Symbol::new(&env, "fee"), &admin);
```

**Fix:** one `#[contracttype]` enum per contract.

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Balance(Address),
    Allowance(Address, Address),
    Admin,
    FeeBps,
    Config,
}
```

### 4.2 Composite keys that flatten into each other

```rust
// COLLISION: ("alice", "bob") can be produced by ("ali", "cebob")
env.storage().persistent().set(&(a.clone(), b.clone()), &value);
```

**Fix:** make the tuple shape unambiguous, or use distinct `DataKey` variants
such as `Allowance(Address, Address)` and `Position(Address, Symbol)`. The types
must match at every call site, which is what prevents the mix-up.

### 4.3 Key length limits

`symbol_short!` panics above 9 characters and `Symbol::new` panics above 32.
Long descriptive key strings panic at runtime, which means the whole
transaction fails.

```rust
// PANICS: longer than 32 characters
let key = Symbol::new(&env, "user_registration_timestamp_for_active_pool");
```

**Fix:** short symbols for `symbol_short!`, an enum for anything structured.

### 4.4 Keys that expose implementation details

A key that embeds a value that can change (`Balance` keyed by a mutable
address alias, or `Config` keyed by a version number that is bumped) breaks every
reader that cached the old key.

**Fix:** keys are permanent identities. If the meaning changes, introduce a new
key and migrate.

### 4.5 Confusing the contract instance with instance *entries*

The contract instance (the account that holds the WASM and the instance entries)
has its own WasmHash and its own TTL, managed with the contract instance API. It
is easy to write `env.storage().instance()` code that accidentally assumes
contract-instance semantics, for example expecting it to be restored the same
way.

**Fix:** be explicit about which one you mean: *instance entries*
(`env.storage().instance()`), or *the contract instance* (the WasmHash and its
lifetime).

---

## 5. TTL, rent, and liveness

Soroban storage is rented. An entry's life is its TTL, expressed in ledgers.

```rust
const LEDGER_THRESHOLD: u32 = 5_184_000; // ~6 months
const LEDGER_BUMP: u32 = 1_209_600;      // ~2 months
```

Typical patterns:

```rust
// Bump on write
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);

// Bump on read, so active users never expire
let value = env.storage().persistent().get(&key);
env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
```

Pitfalls:

* **Only bumping on write.** A user who deposits once and only withdraws months
  later loses their entry to expiration. Bump on read paths that matter.
* **Zero threshold.** `extend_ttl(&key, 0, 0)` only restores an already-expired
  entry and is charged the restoration fee. Use a real threshold.
* **Unbounded threshold.** An enormous threshold can be unaffordable; rent is
  charged for the entire extension.
* **Forgetting temporary storage needs no bump.** Temporary entries have a
  protocol-enforced minimum TTL. Adding manual bump logic there is noise at
  best.
* **Instance TTL is extended with the instance.** When the contract instance is
  restored or its lifetime is extended, its instance entries come along; do not
  assume every instance entry has an independent TTL.

---

## 6. Bugs that only appear at expiration

This is the classic Soroban storage bug class: the code is correct until a TTL
runs out.

```rust
// BUG: an expired entry is indistinguishable from "never existed"
let balance = env.storage().persistent().get(&key).unwrap_or(0);
```

If the entry expired and was restored as empty, `unwrap_or(0)` silently turns a
user's balance into zero. The same pattern turns a "not yet set" admin into a
"no admin" contract, and a nonce into a reusable one.

**Fixes:**

* For balances, decide explicitly whether "missing" means zero (a new user) or
  "corrupt state" (an expired entry), and return a contract error for the latter.
* Test expiration. The SDK lets you advance the ledger sequence in tests so you
  can exercise the expired-entry path deliberately:

  ```rust
  let seq = env.ledger().sequence();
  env.ledger().with_mut(|li| li.sequence = seq + 10_000);
  ```

* Restore the contract before reading: `env.storage().persistent().extend_ttl`
  on an expired entry charges the restoration fee, which is why it is not free.

---

## 7. Migration problems

Storage migrations are the highest-risk change a Soroban contract can make,
because the ledger state is long lived and the contract code is short lived.

### 7.1 The rules

1. **Additive by default.** New keys are safe. Renaming, retyping, or removing a
   key is not.
2. **Never reinterpret an existing key.** If the meaning of a value changes,
   introduce a new key and a migration step; do not write a new type over the old
   value.
3. **Version everything.** Store a version number in instance storage, and make
   `migrate()` idempotent: it should check the current version and do nothing if
   the contract is already migrated.
4. **Test the upgrade path, not just the fresh path.** Write a test that writes
   state with the old layout, runs the migration, and asserts the new layout
   reads the migrated values.

### 7.2 A version marker and idempotent migration

```rust
const VERSION: u32 = 2;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Version,
    Config,
    Balance(Address),
}

fn version(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Version).unwrap_or(1)
}

pub fn migrate(env: Env) -> Result<(), Error> {
    admin(&env).require_auth();

    match version(&env) {
        1 => migrate_v1_to_v2(&env),
        VERSION => return Ok(()), // already migrated: stay idempotent
        v => return Err(Error::UnsupportedVersion),
    }

    env.storage().instance().set(&DataKey::Version, &VERSION);
    Ok(())
}

fn migrate_v1_to_v2(env: &Env) {
    // Move the config from the old flat key to the new namespaced key.
    if let Some(old) = env.storage().persistent().get(&Symbol::new(env, "config")) {
        env.storage().persistent().remove(&Symbol::new(env, "config"));
        env.storage().instance().set(&DataKey::Config, &old);
    }
}
```

### 7.3 Upgrades versus migrations

Updating the contract WASM keeps the storage as it is. That is why:

* `init` must not be re-runnable on an upgraded contract. Use a separate
  `initialize` entry point guarded by a "already initialized" check.
* `migrate` must be callable by an authorized (ideally timelocked) admin only.
* Layout validation before deploying a new WASM is worth automating. See
  `examples/advanced/09-storage-layout-validator` (schema compatibility rules)
  and `examples/advanced/10-contract-migrations` for runnable patterns.

---

## 8. Cost and performance traps

* **One entry per user, not one entry for all users.** Each `persistent()` read
  is billed per entry touched.
* **Avoid `extend_ttl` in hot loops.** Bumping TTL for 100 users in one call is
  100 billed extensions. Bump on the entries that are actually being used.
* **Instance storage is loaded as a unit.** A large instance entry slows down
  every call, including `read-only` calls.
* **Temporary storage for scratch space.** Intermediate values that never need to
  persist belong in temporary storage or in ordinary Rust memory.
* **Prefer `has()` before `get()`.** `has` is cheaper than a full read and makes
  the intent explicit.
* **Measure.** The repo includes gas benchmarks under `gas-benchmarks/` and a
  storage-focused example at `examples/advanced/09-storage-optimization`.

---

## 9. Best practices

1. **One `DataKey` enum per contract.** No ad-hoc `Symbol` or `String` keys.
2. **One helper per stored concept**, so TTL handling lives in exactly one place:

   ```rust
   fn set_balance(env: &Env, user: &Address, amount: i128) {
       let key = DataKey::Balance(user.clone());
       env.storage().persistent().set(&key, &amount);
       env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
   }

   fn get_balance(env: &Env, user: &Address) -> i128 {
       let key = DataKey::Balance(user.clone());
       let value = env.storage().persistent().get(&key).unwrap_or(0);
       env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
       value
   }
   ```

3. **Persistent for durable user state, instance for small bounded config,
   temporary for locks and scratch values.**
4. **Bump TTL on both write and read**, with a documented threshold and bump.
5. **Make `migrate` idempotent** and version-gated; never reinterpret a key.
6. **Treat expiration as a test case.** Advance the ledger sequence in tests and
   assert the behavior you expect.
7. **Prefer `Option`/`Result` over `unwrap()`** so a missing entry is a handled
   branch rather than a panic that reverts the whole transaction.
8. **Keep instance storage small.** Move anything that grows into persistent
   entries.
9. **Document each key** with a comment describing its type, lifetime, and
   meaning. Storage keys are the contract's real schema.
10. **Audit the layout before every upgrade**, not just the code diff.

---

## 10. Review checklist

- [ ] Every key comes from a `DataKey` enum; no inline symbols or strings
- [ ] Balances and positions are persistent, not temporary
- [ ] Instance storage holds only small, bounded configuration
- [ ] Temporary storage holds only locks, nonces, and scratch values
- [ ] TTL is extended on write **and** on the read paths that matter
- [ ] `extend_ttl` uses a non-zero, affordable threshold
- [ ] Missing entries are handled deliberately, not `unwrap()`-ed
- [ ] An expiration test exists (ledger sequence advanced past the TTL)
- [ ] Key tuples cannot collide with each other
- [ ] `migrate` is idempotent, version-gated, and admin-authorized
- [ ] The upgrade test starts from state written by the previous version
- [ ] No key is renamed, retyped, or repurposed without a migration step

---

## Related

- [Storage Patterns](../examples/storage-patterns.md)
- [Security Pitfalls](./security-pitfalls.md)
- [Common Pitfalls](./common-pitfalls.md)
- [Testing Pitfalls](./testing-pitfalls.md)
- [Factory, Proxy, and Registry Patterns](./cross-contract-patterns.md)
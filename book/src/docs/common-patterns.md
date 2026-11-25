# Common Patterns

Reusable building blocks for Soroban contracts.  Every pattern below links
to a runnable example in this repository, so you can copy the snippet, run
the tests, and adapt it to your own contract.

See also [Code Examples →](./code-examples.md) for the topic-by-topic index.

---

## 1. Initialization Guard

A contract must only be initialized once.  Store the admin (or the deployer)
in instance storage and check for it before doing anything else.

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Initialized,
}

pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
    if env.storage().instance().has(&DataKey::Initialized) {
        return Err(ContractError::AlreadyInitialized);
    }
    admin.require_auth();
    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage().instance().set(&DataKey::Initialized, &true);
    Ok(())
}
```

> **Example:** [`examples/intermediate/02-role-based-access-control`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/intermediate/02-role-based-access-control)

---

## 2. Stored-Admin Check

Do not trust an `admin` parameter — read the admin from storage and authorize
*that* address.

```rust
fn read_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_fee(env: Env, fee: u32) -> Result<(), ContractError> {
    read_admin(&env)?.require_auth();
    env.storage().instance().set(&DataKey::Fee, &fee);
    Ok(())
}
```

> **Example:** [`examples/advanced/04-circuit-breaker`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/04-circuit-breaker)

---

## 3. Typed Storage Keys

`#[contracttype]` enums make the storage layout explicit and avoid key
collisions between features.

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Balance(Address),
    Allowance(Address, Address),
    Config(u32),
}

let key = DataKey::Allowance(owner.clone(), spender.clone());
let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
```

> **Example:** [`examples/tokens/05-allowance-pattern`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/05-allowance-pattern)

---

## 4. Reentrancy Guard

Set a flag before an external call and clear it afterwards.

```rust
fn is_entered(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Entered).unwrap_or(false)
}

pub fn withdraw(env: Env, to: Address, amount: i128, target: Address) -> Result<(), ContractError> {
    if is_entered(&env) {
        return Err(ContractError::Reentrancy);
    }
    env.storage().instance().set(&DataKey::Entered, &true);

    // external call — a re-entrant call sees the flag and fails
    env.invoke_contract::<()>(&target, &symbol_short!("receive_funds"), args);

    env.storage().instance().set(&DataKey::Entered, &false);
    Ok(())
}
```

> **Example:** [`examples/advanced/15-reentrancy-guard`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/15-reentrancy-guard)

---

## 5. Circuit Breaker

Trip after N failures and recover automatically after a time window.

```rust
pub fn fail(env: Env, caller: Address) -> Result<(), CircuitError> {
    caller.require_auth();
    ensure_active(&env)?;

    let mut count: u32 = env.storage().instance().get(&DataKey::FailureCount).unwrap_or(0);
    count += 1;
    env.storage().instance().set(&DataKey::FailureCount, &count);

    if count >= threshold(&env) {
        env.storage().instance().set(&DataKey::State, &CircuitState::Paused);
    }
    Ok(())
}
```

> **Example:** [`examples/advanced/04-circuit-breaker`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/04-circuit-breaker)

---

## 6. Pause / Unpause

An emergency brake that halts state-changing calls while reads keep working.

```rust
fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused {
        Err(ContractError::ContractPaused)
    } else {
        Ok(())
    }
}
```

> **Example:** [`examples/intermediate/03-pause-unpause`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/intermediate/03-pause-unpause)

---

## 7. Rate Limiting

Bound how much a user can do inside a time window.

```rust
pub fn consume(env: Env, user: Address, amount: i128) -> Result<(), RateLimitError> {
    user.require_auth();

    let limit = limit_of(&env, &user);
    let mut usage = usage_of(&env, &user);

    if usage.calls >= limit.max_calls {
        return Err(RateLimitError::CallLimitExceeded);
    }
    if usage.amount + amount > limit.max_amount {
        return Err(RateLimitError::AmountLimitExceeded);
    }

    usage.calls += 1;
    usage.amount += amount;
    env.storage().persistent().set(&DataKey::Usage(user), &usage);
    Ok(())
}
```

> **Example:** [`examples/advanced/05-rate-limiting`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/05-rate-limiting)

---

## 8. Merkle Proof Verification

Store one root on-chain and prove membership off-chain.

```rust
pub fn verify(env: Env, leaf: BytesN<32>, index: u32, proof: Vec<BytesN<32>>) -> Result<bool, MerkleError> {
    let root = read_root(&env)?;
    let mut computed = leaf.clone();
    for sibling in proof.iter() {
        computed = hash_pair(&env, &computed, &sibling); // sha256(min || max)
    }
    Ok(computed == root)
}
```

> **Examples:** [`examples/advanced/21-merkle-proofs`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/21-merkle-proofs),
> [`examples/advanced/28-merkle-airdrop`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/28-merkle-airdrop)

---

## 9. Batch Operations

Group many transfers into one transaction to save fees.

```rust
pub fn batch_transfer(env: Env, from: Address, transfers: Vec<Transfer>) -> Result<u32, BatchError> {
    from.require_auth();

    let mut total: i128 = 0;
    for t in transfers.iter() {
        if t.amount <= 0 {
            return Err(BatchError::InvalidAmount);
        }
        total += t.amount;
    }
    if total > balance_of(&env, &from) {
        return Err(BatchError::InsufficientBalance);
    }
    // ... apply every transfer (all-or-nothing)
    Ok(transfers.len() as u32)
}
```

> **Examples:** [`examples/advanced/20-batch-transfer`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/20-batch-transfer),
> [`examples/advanced/42-multicall`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/42-multicall)

---

## 10. Voting Power and Delegation

Let token holders hand their voting power to another address.

```rust
pub fn delegate(env: Env, from: Address, to: Address) -> Result<(), DelegationError> {
    from.require_auth();
    if from == to {
        return Err(DelegationError::SelfDelegation);
    }
    env.storage().persistent().set(&DataKey::Delegation(from), &to);
    Ok(())
}

pub fn get_voting_power(env: Env, user: Address) -> i128 {
    if env.storage().persistent().has(&DataKey::Delegation(user.clone())) {
        return 0;
    }
    balance_of(&env, &user)
}
```

> **Example:** [`examples/governance/06-vote-delegation`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/06-vote-delegation)

---

## 11. Time-Locked Execution

Queue an operation now, execute it once the delay has passed.

```rust
pub fn queue(env: Env, target: Address, function: Symbol, args: Vec<Val>, delay: u64) -> Result<u64, Error> {
    read_admin(&env)?.require_auth();
    if delay < min_delay(&env) {
        return Err(Error::InvalidDelay);
    }
    let eta = env.ledger().timestamp() + delay;
    // ... store the proposal
    Ok(id)
}

pub fn execute(env: Env, id: u64) -> Result<Val, Error> {
    read_admin(&env)?.require_auth();
    if env.ledger().timestamp() < eta(&env, id) {
        return Err(Error::DelayNotMet);
    }
    env.invoke_contract::<Val>(&target(&env, id), &function(&env, id), args(&env, id))
}
```

> **Example:** [`examples/governance/09-timelock-governance`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/09-timelock-governance)

---

## 12. Snapshot Triggers

Keep a cheap history of a value so votes and prices can be looked up "as of"
a past ledger.

```rust
pub fn record_value(env: Env, owner: Address, value: i128) {
    let record = SnapshotRecord {
        ledger: env.ledger().sequence(),
        timestamp: env.ledger().timestamp(),
        value,
    };
    let mut history = history(&env, &owner);
    history.push_back(record);
    env.storage().persistent().set(&DataKey::Snapshots(owner), &history);
}
```

> **Examples:** [`examples/defi/14-automatic-snapshot-triggers`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/14-automatic-snapshot-triggers),
> [`examples/governance/11-automatic-snapshot-triggers`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/11-automatic-snapshot-triggers)

---

## Related Pages

- [Code Examples →](./code-examples.md)
- [Factory, Proxy, and Registry Patterns →](./cross-contract-patterns.md)
- [Common Pitfalls →](./common-pitfalls.md)
- [Best Practices →](./best-practices.md)
- [Testing Best Practices →](./testing-best-practices.md)

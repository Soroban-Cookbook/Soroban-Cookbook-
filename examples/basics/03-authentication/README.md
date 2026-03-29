# Authentication Patterns in Soroban

This example demonstrates how to authenticate callers and build layered
authorization logic in Soroban smart contracts. It covers the core
`require_auth()` primitive, role-based access control, time-based
restrictions, and state-gated operations — all patterns you will encounter
in production contracts.

## Auth Concepts

### What `require_auth()` does

`require_auth()` is called on an `Address` value. It instructs the Soroban
host to verify that the address has cryptographically signed the current
transaction (or, for contract addresses, that the contract has approved the
sub-invocation). If the check fails the host aborts the transaction
immediately — no state is written.

```
User signs tx  ──►  Host verifies signature  ──►  require_auth() passes
                                                   contract logic runs
```

Key properties:

- Works for both user accounts (ed25519 keypairs) and contract addresses.
- Replay protection is handled automatically by the host via nonces.
- A single transaction can carry authorizations for multiple addresses; the
  host checks each one independently when `require_auth()` is called.

### Authentication vs. Authorization

| Term | Question answered | Soroban mechanism |
|---|---|---|
| Authentication | *Is this really Alice?* | `address.require_auth()` |
| Authorization | *Is Alice allowed to do this?* | Custom logic after `require_auth()` |

`require_auth()` only answers the first question. Everything in this example
that follows the `require_auth()` call — role checks, time-lock guards,
state checks — is authorization logic that you write.

### The allowance (delegated auth) pattern

`approve` + `transfer_from` lets an owner delegate spending rights to a
third party without handing over their private key. The spender calls
`transfer_from` and must pass `spender.require_auth()`. The owner's
authorization is captured at `approve` time.

```rust
// Owner grants spender the right to move up to 500 tokens.
pub fn approve(env: Env, from: Address, spender: Address, amount: i128)
    -> Result<(), AuthError>
{
    from.require_auth();                                    // owner signs
    env.storage().persistent()
        .set(&DataKey::Allowance(from, spender), &amount);
    Ok(())
}

// Spender exercises the allowance.
pub fn transfer_from(
    env: Env, spender: Address, from: Address, to: Address, amount: i128,
) -> Result<(), AuthError> {
    spender.require_auth();                                 // spender signs
    let allowance: i128 = env.storage().persistent()
        .get(&DataKey::Allowance(from.clone(), spender.clone()))
        .unwrap_or(0);
    if allowance < amount { return Err(AuthError::Unauthorized); }
    // … update balances and reduce allowance …
    Ok(())
}
```

### N-of-N multi-sig

Iterate the signer list and call `require_auth()` on each. The host
verifies all signatures atomically before the function body executes.

```rust
pub fn multi_sig_action(_env: Env, signers: Vec<Address>, value: u32) -> u32 {
    for signer in signers.iter() {
        signer.require_auth();
    }
    value + signers.len()
}
```

---

## `require_auth()` Usage Guide

### Basic pattern — single caller

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128)
    -> Result<(), AuthError>
{
    from.require_auth();          // 1. authenticate
    // validate …                // 2. validate inputs
    // mutate storage …          // 3. execute
    Ok(())
}
```

Always follow this order: **authenticate → validate → execute**. Placing
`require_auth()` first ensures that unauthenticated callers are rejected
before any computation or storage reads occur.

### Admin-only pattern

Store the admin address at initialization and compare on every privileged
call. The helper `require_admin` encapsulates this check so it is not
duplicated across functions.

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(AuthError::AlreadyInitialized);   // idempotency guard
    }
    admin.require_auth();
    env.storage().instance().set(&DataKey::Admin, &admin);
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), AuthError> {
    let admin: Address = env.storage().instance()
        .get(&DataKey::Admin)
        .ok_or(AuthError::NotAdmin)?;
    if caller != &admin { return Err(AuthError::NotAdmin); }
    Ok(())
}

pub fn admin_action(env: Env, admin: Address, value: u32)
    -> Result<u32, AuthError>
{
    admin.require_auth();
    require_admin(&env, &admin)?;
    Ok(value * 2)
}
```

### Role-based access control

Roles are stored in persistent storage so they survive contract upgrades.
The `require_role` helper accepts a slice of allowed roles, enabling
functions to be accessible by multiple tiers.

```rust
pub fn grant_role(env: Env, admin: Address, account: Address, role: Role)
    -> Result<(), AuthError>
{
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().persistent()
        .set(&DataKey::UserRole(account.clone()), &role);
    Ok(())
}

fn require_role(env: &Env, caller: &Address, allowed: &[Role])
    -> Result<(), AuthError>
{
    let user_role: Role = env.storage().persistent()
        .get(&DataKey::UserRole(caller.clone()))
        .unwrap_or(Role::User);
    for role in allowed {
        if user_role as u32 <= *role as u32 { return Ok(()); }
    }
    Err(AuthError::InsufficientRole)
}

// Admin-only
pub fn admin_role_action(env: Env, caller: Address, value: u64)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    require_role(&env, &caller, &[Role::Admin])?;
    Ok(value * 2)
}

// Admin or Moderator
pub fn moderator_action(env: Env, caller: Address, value: u64)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    require_role(&env, &caller, &[Role::Admin, Role::Moderator])?;
    Ok(value + 10)
}
```

### Time-lock pattern

A global unlock timestamp blocks an action until a future ledger time.
Useful for vesting schedules, governance cool-off periods, and delayed
withdrawals.

```rust
pub fn time_locked_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let unlock: u64 = env.storage().instance()
        .get(&DataKey::TimeLock).unwrap_or(0);
    if env.ledger().timestamp() < unlock {
        return Err(AuthError::TimeLocked);
    }
    Ok(env.ledger().timestamp())
}
```

### Per-address cooldown pattern

Enforces a minimum interval between successive calls from the same address.
Mitigates spam and rate-limits sensitive operations without off-chain
infrastructure.

```rust
pub fn cooldown_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let period: u64 = env.storage().instance()
        .get(&DataKey::CooldownPeriod).unwrap_or(0);
    let last: u64 = env.storage().persistent()
        .get(&DataKey::LastAction(caller.clone())).unwrap_or(0);
    let now = env.ledger().timestamp();
    if last > 0 && now < last + period {
        return Err(AuthError::CooldownActive);
    }
    env.storage().persistent()
        .set(&DataKey::LastAction(caller), &now);
    Ok(now)
}
```

### State-gated pattern

A global `ContractState` enum acts as a circuit-breaker. Admins can pause
or freeze the contract in an emergency; normal operations check the state
before proceeding.

```rust
pub fn active_only_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let state: ContractState = env.storage().instance()
        .get(&DataKey::State).unwrap_or(ContractState::Active);
    if state != ContractState::Active {
        return Err(AuthError::InvalidState);
    }
    Ok(env.ledger().timestamp())
}
```

---

## Security Best Practices

**1. Call `require_auth()` before any other logic.**
Reject unauthenticated callers at the earliest possible point. Do not read
storage, emit events, or perform arithmetic before the auth check.

**2. Never trust the `admin` argument alone.**
Always load the stored admin and compare. An attacker can pass any address
they control as the `admin` argument; only the stored value is authoritative.

```rust
// ✅ correct
admin.require_auth();
let stored: Address = env.storage().instance().get(&DataKey::Admin)...;
if admin != stored { return Err(AuthError::NotAdmin); }

// ❌ wrong — trusts the argument without checking storage
admin.require_auth();
// proceeds as if admin is legitimate
```

**3. Guard `initialize()` against re-entrancy.**
Check for an existing admin before writing. Without this guard a second
caller can overwrite the admin address after deployment.

**4. Store roles in persistent storage.**
Instance storage is wiped on contract upgrade. Persistent storage survives,
so role assignments remain valid across upgrades.

**5. Separate authentication from authorization.**
`require_auth()` proves identity. Role checks, balance checks, and state
checks prove permission. Keep them in distinct, auditable code paths.

**6. Use typed errors, not generic panics.**
`AuthError::NotAdmin` is more informative than `panic!("not admin")` and
allows callers to handle specific failure modes programmatically.

**7. Emit events on privileged state changes.**
Role grants/revocations, state transitions, and time-lock updates should
emit events so off-chain monitors can detect unexpected changes.

---

## Common Pitfalls

### Forgetting `require_auth()` entirely

```rust
// ❌ anyone can drain balances
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    let bal: i128 = env.storage().persistent()
        .get(&DataKey::Balance(from.clone())).unwrap_or(0);
    env.storage().persistent()
        .set(&DataKey::Balance(from), &(bal - amount));
}
```

### Calling `require_auth()` after state changes

```rust
// ❌ state is mutated before auth is verified
pub fn bad_transfer(env: Env, from: Address, to: Address, amount: i128) {
    env.storage().persistent().set(&DataKey::Balance(from.clone()), &0);
    from.require_auth();   // too late — storage already written
}
```

### Checking the wrong address

```rust
// ❌ authenticates `to`, not `from` — the wrong party authorizes the debit
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    to.require_auth();
    // …
}
```

### Skipping the stored-admin comparison

```rust
// ❌ any address that signs can claim admin rights
pub fn admin_action(env: Env, admin: Address, value: u32) -> u32 {
    admin.require_auth();   // proves identity, not that admin == stored admin
    value * 2
}
```

### Using instance storage for roles

```rust
// ❌ roles are lost on contract upgrade
env.storage().instance().set(&DataKey::UserRole(account), &role);

// ✅ roles survive upgrades
env.storage().persistent().set(&DataKey::UserRole(account), &role);
```

---

## Real-World Examples

### Token transfer with allowance (ERC-20 style)

```rust
// 1. Owner approves spender for 500 tokens.
client.approve(&owner, &spender, &500);

// 2. Spender moves 200 tokens to a recipient.
client.transfer_from(&spender, &owner, &recipient, &200);

// Result: owner balance −200, recipient balance +200, allowance 300.
```

### DAO proposal gate

A governance contract can combine role checks with a time-lock so that
proposals can only be executed after a mandatory review period:

```rust
pub fn execute_proposal(env: Env, caller: Address) -> Result<(), AuthError> {
    caller.require_auth();
    require_role(&env, &caller, &[Role::Admin, Role::Moderator])?;

    let unlock: u64 = env.storage().instance()
        .get(&DataKey::TimeLock).unwrap_or(0);
    if env.ledger().timestamp() < unlock {
        return Err(AuthError::TimeLocked);
    }

    let state: ContractState = env.storage().instance()
        .get(&DataKey::State).unwrap_or(ContractState::Active);
    if state != ContractState::Active {
        return Err(AuthError::InvalidState);
    }

    // … execute proposal …
    Ok(())
}
```

### Emergency pause

```rust
// Admin detects an exploit and pauses the contract.
client.set_state(&admin, &ContractState::Paused);

// All active_only_action calls now return AuthError::InvalidState.
// Admin can resume later.
client.set_state(&admin, &ContractState::Active);
```

### Cross-contract proxy auth

When a proxy contract calls a target contract on behalf of a user, the user
must authorize the entire call chain. The proxy requires the user's auth
before making the cross-contract call; the target then calls
`user.require_auth()` again, and the host verifies the chain atomically.

```rust
// Proxy
pub fn proxy_call(env: Env, target: Address, user: Address) -> Address {
    user.require_auth();                          // user authorizes proxy
    let client = TargetContractClient::new(&env, &target);
    client.check_nested_auth(&user);              // target re-checks user
    user
}
```

---

## Running the Tests

Run the comprehensive unit test suite to see these authentication patterns in action:
```bash
cd examples/basics/03-authentication
cargo test
```

The test suite covers:

- Initialization and double-init rejection
- Admin-only actions (authorized and unauthorized)
- Balance set, transfer, and insufficient-balance rejection
- Approve and `transfer_from` with allowance enforcement
- N-of-N multi-sig
- Role grant, revoke, and `has_role` hierarchy
- Admin and moderator role actions
- Time-lock: blocked before unlock, passes after
- Cooldown: blocked within period, passes after expiry
- State gating: Active allows, Paused and Frozen block

## Building for Deployment

Compile the contract to WebAssembly:
```bash
cargo build --target wasm32-unknown-unknown --release
```

The WASM artifact is written to
`target/wasm32-unknown-unknown/release/authentication.wasm`.

## Related Examples

- [05-auth-context](../05-auth-context/) — invoker detection and cross-contract call chains
- [03-custom-errors](../03-custom-errors/) — structured error types used throughout this example
- [02-storage-patterns](../02-storage-patterns/) — persistent vs. instance storage trade-offs

## References

- [Soroban Authorization](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/authorization)
- [Soroban SDK — `Address::require_auth`](https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Address.html#method.require_auth)
- [Custom Account Contracts](https://developers.stellar.org/docs/smart-contracts/guides/custom-accounts)

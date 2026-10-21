# Security Pitfalls in Soroban Contracts

Soroban removes many classic EVM hazards by design, but it introduces its own
sharp edges: host-enforced authorization, three distinct storage types with
independent TTLs, and a host that reverts the whole call tree on panic. This page
collects the mistakes that keep showing up in real Soroban deployments, with a
vulnerable snippet, a fixed snippet, and the prevention rule for each one.

Read this alongside [Security Best Practices](./security-best-practices.md),
[Common Pitfalls](./common-pitfalls.md), and the
[Audit Checklist](../../../SECURITY_AUDIT.md).

---

## Table of Contents

1. [Threat model in 60 seconds](#1-threat-model-in-60-seconds)
2. [Authorization pitfalls](#2-authorization-pitfalls)
3. [Cross-contract call pitfalls](#3-cross-contract-call-pitfalls)
4. [Reentrancy and read-only reentrancy](#4-reentrancy-and-read-only-reentrancy)
5. [Math, overflow, and precision pitfalls](#5-math-overflow-and-precision-pitfalls)
6. [Panics, traps, and lost error information](#6-panics-traps-and-lost-error-information)
7. [Storage and TTL pitfalls](#7-storage-and-ttl-pitfalls)
8. [Token and allowance pitfalls](#8-token-and-allowance-pitfalls)
9. [Randomness, time, and oracle pitfalls](#9-randomness-time-and-oracle-pitfalls)
10. [Upgrade and proxy pitfalls](#10-upgrade-and-proxy-pitfalls)
11. [Prevention strategy](#11-prevention-strategy)
12. [Pre-merge review checklist](#12-pre-merge-review-checklist)

---

## 1. Threat model in 60 seconds

Before listing pitfalls, be precise about who can do what:

* A caller is an `Address`. A contract never "is trusted" because it was
  deployed earlier; every contract is untrusted code that happens to be
  deterministic.
* `require_auth()` is checked by the **host**, not by your contract. A missing
  check is not "weaker validation", it is no validation at all.
* Any contract that calls your contract may be adversarial, including a
  contract you wrote, a token wrapper, or an oracle you integrate with.
* Storage is keyed by contract ID. Instance and temporary entries are separate
  namespaces from persistent entries, and temporary entries are wiped between
  ledger close and the next protocol upgrade rotation.

The three questions to ask about every entry point are:

1. Who is allowed to call this, and how is that enforced?
2. What state is read, and what state is written, in what order?
3. Whose code runs during this call, and can it call me back?

---

## 2. Authorization pitfalls

### 2.1 Missing `require_auth()` on a state-changing function

```rust
// VULNERABLE: anyone can pass another user's address
pub fn withdraw(env: Env, user: Address, amount: i128) {
    let mut balance = get_balance(&env, &user);
    balance -= amount;
    set_balance(&env, &user, balance);
}
```

```rust
// FIXED
pub fn withdraw(env: Env, user: Address, amount: i128) {
    user.require_auth();
    let balance = get_balance(&env, &user);
    let new_balance = balance
        .checked_sub(amount)
        .ok_or(Error::InsufficientBalance)?;
    set_balance(&env, &user, new_balance);
    env.events().publish((symbol_short!("withdraw"), user.clone()), amount);
}
```

**Prevention:** call `require_auth()` as the first statement of every function
that mutates state or moves value, and prefer `require_auth_for_args` when the
signature covers only part of the operation.

### 2.2 Authorizing a contract address you do not control

`require_auth()` on an address that is a *contract* means that contract's
`__check_auth` must approve. If that contract's implementation is an open proxy
or has a permissive fallback, your authorization is effectively public.

```rust
// VULNERABLE: authorization delegated to an arbitrary caller-supplied address
pub fn sweep(env: Env, caller: Address, to: Address, amount: i128) {
    caller.require_auth();
    token_client.transfer(&caller, &to, &amount);
}
```

```rust
// FIXED: an authorized admin is stored in instance storage and may not be changed
// without a timelock
fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn sweep(env: Env, to: Address, amount: i128) {
    admin(&env).require_auth();
    if to == env.current_contract_address() {
        return Err(Error::InvalidRecipient.into());
    }
    let token = token_address(&env);
    token::Client::new(&env, &token).transfer(&admin(&env), &to, &amount);
    Ok(())
}
```

**Prevention:** store the authority that may call an entry point in storage,
load it inside the function, and require auth on the loaded value. Never
authorize a caller-supplied address for an operation the caller should not be
able to route.

### 2.3 Auth inside a loop, or only on the happy path

```rust
// VULNERABLE: early returns and `continue` skip authorization for later items
pub fn settle(env: Env, claims: Vec<(Address, i128)>) {
    for (to, amount) in claims.iter() {
        if amount == &0 {
            continue; // never authorized
        }
        // ...
    }
}
```

**Prevention:** authenticate once at the top of the function, before any
branching or looping, for every address the function can act on behalf of.

### 2.4 Confusing `require_auth` with `require_auth_for_args`

`require_auth()` demands that the **whole** `Address` payload of the invocation
be authorized. `require_auth_for_args(...)` authorizes only the listed values,
which is correct for a function with a non-signing `caller` parameter.

```rust
// VULNERABLE: requires the user to sign the admin address too, which is impossible
pub fn transfer_from_admin(env: Env, user: Address, admin: Address, amount: i128) {
    user.require_auth();
    admin.require_auth();
    move_funds(&env, &admin, &user, amount);
}
```

**Prevention:** if a parameter only identifies *what* is being changed and the
signer is someone else, authorize only the signer and validate the rest against
stored state.

### 2.5 Replay through shared invocations

Soroban supports sub-invocations, so a contract can request your contract to act
inside someone else's transaction. If your entry point relies on "the direct
caller is the user", a malicious contract can call it on the user's behalf.

**Prevention:** use `env.invoker()` for caller checks when the semantics require
the immediate invoker, and use `require_auth()` for identity checks. Pick one
deliberately; do not mix them implicitly.

---

## 3. Cross-contract call pitfalls

### 3.1 Trusting the callee to enforce *your* invariants

A token contract that does not revert on a zero-amount transfer will happily
return `Ok`. If your accounting assumes failure, it silently drifts.

```rust
// VULNERABLE: assumes a failed transfer reverts
pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), Error> {
    token_client.transfer(&user, &env.current_contract_address(), &amount);
    credit(&env, &user, amount); // credited even if the token silently no-ops
    Ok(())
}
```

```rust
// FIXED: observe the balance delta, do not trust the return value alone
pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), Error> {
    let token = token_address(&env);
    let contract = env.current_contract_address();
    let client = token::Client::new(&env, &token);
    let before = client.balance(&contract);

    client.transfer(&user, &contract, &amount);

    let received = client
        .balance(&contract)
        .checked_sub(before)
        .ok_or(Error::AmountMismatch)?;
    if received != amount {
        return Err(Error::AmountMismatch);
    }
    credit(&env, &user, received);
    Ok(())
}
```

**Prevention:** for fee-on-transfer and rebasing-style assets, reconcile against
an observed balance change. Never assume `amount` arrived.

### 3.2 Unchecked `try_` results

`try_invoke_contract` and `try_transfer` return `Result` instead of trapping.
Swallowing it converts a failure into an inconsistent success.

```rust
// VULNERABLE
let _ = token_client.try_transfer(&from, &to, &amount);

// FIXED
token_client.try_transfer(&from, &to, &amount).map_err(|_| Error::TransferFailed)?;
```

**Prevention:** propagate every `Result` with `?` or an explicit `match`. A
`let _ =` on a call that moves value should be treated as a review blocker.

### 3.3 Unbounded loops over caller-controlled input

A `Vec` argument with 10,000 entries costs 10,000 ledger reads and can exceed
the CPU budget, failing for every legitimate user while a griefing caller pays
almost nothing.

**Prevention:** bound loop inputs (`if items.len() > MAX_BATCH { return Err(...) }`),
prefer batching helpers that take fixed-size arrays, and keep external calls out
of loops.

---

## 4. Reentrancy and read-only reentrancy

Soroban does not allow a contract to be re-entered while it is already on the
call stack, which removes *direct* self-reentrancy. It does **not** remove
cross-contract reentrancy: `A -> B -> A` is legal as long as the inner `A`
invocation starts from a fresh frame in the same transaction, and it also does
not stop read-only reentrancy, where `B` calls a view function on `A` while `A`
is mid-update and observes half-updated state.

### 4.1 The vulnerable shape

```rust
// VULNERABLE: state is read *after* an external call
pub fn withdraw(env: Env, user: Address, amount: i128) {
    let balance = get_balance(&env, &user);
    token_client.transfer(&env.current_contract_address(), &user, amount);
    set_balance(&env, &user, balance - amount); // balance may be stale
}
```

### 4.2 Fix 1: checks-effects-interactions

```rust
// FIXED: write state first, interact last
pub fn withdraw(env: Env, user: Address, amount: i128) -> Result<(), Error> {
    user.require_auth();

    let balance = get_balance(&env, &user);
    let remaining = balance
        .checked_sub(amount)
        .ok_or(Error::InsufficientBalance)?;
    set_balance(&env, &user, remaining);

    let token = token_address(&env);
    token::Client::new(&env, &token).transfer(
        &env.current_contract_address(),
        &user,
        &amount,
    );
    Ok(())
}
```

### 4.3 Fix 2: an explicit reentrancy guard

```rust
fn set_entered(env: &Env, entered: bool) {
    let key = DataKey::Entered;
    if entered {
        env.storage().temporary().set(&key, &true);
    } else {
        env.storage().temporary().remove(&key);
    }
}

fn guard(env: &Env) -> Result<(), Error> {
    if env.storage().temporary().has(&DataKey::Entered) {
        return Err(Error::Reentrancy);
    }
    set_entered(env, true);
    Ok(())
}

pub fn guarded_withdraw(env: Env, user: Address, amount: i128) -> Result<(), Error> {
    guard(&env)?;
    // ... effects and interactions ...
    set_entered(&env, false);
    Ok(())
}
```

Use a temporary entry for the flag: it is the cheapest storage and it is reset
at ledger close, so a panic mid-function cannot leave the contract locked for
the next transaction.

### 4.4 Read-only reentrancy

If `get_*` functions read several keys and a callee can call them mid-update,
the callee sees inconsistent state and can price against it.

```rust
// FIXED: a read guard makes views refuse to answer mid-update
pub fn get_position(env: Env, user: Address) -> Result<(i128, i128), Error> {
    if env.storage().temporary().has(&DataKey::Entered) {
        return Err(Error::Reentrancy);
    }
    Ok((get_balance(&env, &user), get_collateral(&env, &user)))
}
```

**Prevention rules**

1. Apply checks-effects-interactions everywhere; treat the guard as defense in
   depth, not as the primary protection.
2. Keep external calls in the last statements of the function.
3. Guard views that aggregate state when the contract can be called back into.
4. Add a regression test that performs the `A -> B -> A` cycle and asserts the
   inner call fails.

See `examples/advanced/15-reentrancy-guard` for a runnable version of all three
defenses.

---

## 5. Math, overflow, and precision pitfalls

### 5.1 Unchecked arithmetic in release builds

```rust
// VULNERABLE: wraps or traps depending on the profile
pub fn accrue(env: Env, user: Address) {
    let balance = get_balance(&env, &user);
    set_balance(&env, &user, balance * 2);
}
```

```rust
// FIXED
pub fn accrue(env: Env, user: Address) -> Result<(), Error> {
    let balance = get_balance(&env, &user);
    let doubled = balance.checked_mul(2).ok_or(Error::Overflow)?;
    set_balance(&env, &user, doubled);
    Ok(())
}
```

**Prevention:** every arithmetic result that touches a balance, a price, or a
share uses `checked_*`. Add a `#![deny(clippy::arithmetic_side_effects)]`-style
lint gate, or enable `clippy::indexing_slicing` and
`clippy::arithmetic_side_effects` in the workspace lint configuration.

### 5.2 Integer division truncation

`(a + b) / 2` rounds toward zero and silently favours one side. In share math and
fee distribution this creates a rounding gap that accumulates.

```rust
// VULNERABLE: the remainder is lost
let half = (a + b) / 2;

// FIXED: multiply by a large scalar and divide last (Soroban recipe)
const SCALE: i128 = 1_000_000;
let half = ((a + b) * SCALE / 2) / SCALE;
```

**Prevention:** follow the Soroban SDK "integer without floating point" recipe -
scale up, divide, scale down - and document the rounding direction in a comment.

### 5.3 Sign confusion between `i128` and `u128`

Amounts that can be negative (signed) and amounts that cannot (unsigned) get
mixed, and casting silently wraps.

**Prevention:** pick one type for amounts (usually `i128`), forbid `as` casts in
review, and use `TryFrom`/`TryInto` where a conversion is genuinely needed.

---

## 6. Panics, traps, and lost error information

A Soroban panic aborts the entire transaction and consumes all gas, so a stray
`unwrap()` is a denial-of-service bug for that user, not just a crash.

```rust
// VULNERABLE
let balance = get_balance(&env, &user).unwrap(); // panics when the key is absent

// FIXED
let balance = get_balance(&env, &user).unwrap_or(0);
```

Other traps to watch for:

* `symbol_short!` panics above 9 characters; `Symbol::new` panics above 32.
* Indexing a `Vec` or `Map` out of bounds panics; use `get()`.
* Arithmetic overflow in a debug test but wraparound in a release WASM - the
  bug reproduces only in the artifact you shipped.
* `env.storage().instance().get(...).unwrap()` after a failed initialization
  path.

**Prevention:** define a `#[contracttype] enum Error`, return `Result` from every
external entry point, and reserve panics for genuine invariants.

---

## 7. Storage and TTL pitfalls

Storage mistakes are security bugs because they become *permission* bugs.

| Pitfall | Consequence | Fix |
| --- | --- | --- |
| Writing a balance to temporary storage | Balance vanishes at ledger close | Persistent + `extend_ttl` |
| Extending TTL only on write | Long-idle keys expire under the user | Extend on read too |
| Same key name in two storage types | Silent split brain | Namespaced enum keys |
| `instance()` used for a growing map | Every call loads the whole map | Persistent per-entry keys |
| Unbounded key growth | State bloat, expensive snapshots | Prune or archive off-chain |

```rust
// FIXED: persistent with a bounded extension, checked on both write and read
pub fn set_balance(env: &Env, user: &Address, amount: i128) {
    let key = DataKey::Balance(user.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_balance(env: &Env, user: &Address) -> i128 {
    let key = DataKey::Balance(user.clone());
    let value = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    value
}
```

**Prevention:** a single `#[contracttype] enum DataKey` per contract, one helper
per stored concept, `LEDGER_THRESHOLD`/`LEDGER_BUMP` constants derived from your
minimum viable TTL, and TTL extension on read paths that matter. The full
treatment lives in [Storage Pitfalls](./storage-pitfalls.md).

---

## 8. Token and allowance pitfalls

* **Approve-then-transfer race:** a user who changes an allowance from 100 to 0
  can be front-run by the spender using the old allowance. Provide a
  `decrease_allowance` helper and document the risk.
* **Infinite approvals** to a contract you later upgrade are permanent
  exposures. Prefer exact allowances or pull-based patterns.
* **Hard-coded token addresses** in code make upgrades impossible. Read the
  token from instance storage.
* **Rebasing or fee-on-transfer tokens** break `amount == received` assumptions;
  reconcile against balance deltas.

---

## 9. Randomness, time, and oracle pitfalls

* There is no secure randomness in a deterministic VM. `env.random()` is seeded
  by the protocol and is predictable to a caller who influences the inputs; never
  use it for lotteries, shuffles, or key generation.
* `env.ledger().timestamp()` is producer-controlled within a window. Never use
  it as a precise deadline or as an entropy source.
* Oracle prices are stale by design. Validate the price, bound it against a
  sanity range, and store the timestamp alongside the value.

---

## 10. Upgrade and proxy pitfalls

* An upgradeable contract keeps its storage. Any layout change must be additive
  or accompanied by a migration function that runs once, guarded by a version
  marker in storage.
* An uninitialized proxy can be initialized by an attacker and take permanent
  ownership. Set `admin` atomically in the initializer and re-initialization must
  revert.
* Upgrading a contract whose keys are unnamespaced can collide with the new
  implementation's keys.

**Prevention:** store a `Version` in instance storage, expose `migrate()` with a
`require_auth` on a timelocked admin, and validate layouts before deploying (see
`examples/advanced/09-storage-layout-validator` and
`examples/advanced/10-contract-migrations`).

---

## 11. Prevention strategy

| Layer | Action |
| --- | --- |
| Design | Threat-model each entry point; write the abuse case next to the happy path |
| Key design | One `DataKey` enum; no ad-hoc `Symbol`/`String` keys |
| Auth | `require_auth()` first, on a stored address where possible |
| Arithmetic | `checked_*` everywhere; no `as` casts |
| External calls | Last statement; results always checked |
| Guards | Temporary reentrancy flag + read guard for aggregators |
| Errors | Contract-defined `Error` enum; no `unwrap()` in entry points |
| Time to live | Extend on read and write; test with `--durability` scenarios |
| Upgrade | Version marker + tested migration path |
| Review | Two reviewers for auth and math changes; [audit checklist](../../../SECURITY_AUDIT.md) |

### Testing checklist

* A test where the caller is **not** the beneficiary must fail with an
  authorization error.
* A test that drives `A -> B -> A` must show the inner call rejected.
* A test that expires TTLs (`test::with_budget` plus a ledger bump simulation)
  must show balances intact.
* A test that passes a fee-on-transfer style mock must show reconciliation.
* Every `try_*` result is asserted, including the failure branch.

---

## 12. Pre-merge review checklist

- [ ] Every state-changing entry point calls `require_auth()` on a trusted address
- [ ] No `unwrap()` in any external entry point
- [ ] No unchecked arithmetic on balances, prices, or shares
- [ ] Every external call result handled; no `let _ =` on value-moving calls
- [ ] Effects written before interactions
- [ ] Reentrancy guard present where the contract can be called back into
- [ ] View functions that aggregate state are guarded
- [ ] Persistent storage for anything that must survive a ledger close
- [ ] TTL extended on read and write paths
- [ ] Storage keys namespaced by a `DataKey` enum
- [ ] Proxy initializer cannot be run twice
- [ ] Tests cover unauthorized, reentrant, and expired-TTL paths

---

## Related

- [Security Best Practices](./security-best-practices.md)
- [Common Pitfalls](./common-pitfalls.md)
- [Storage Pitfalls](./storage-pitfalls.md)
- [Testing Pitfalls](./testing-pitfalls.md)
- [Audit checklist](../../../SECURITY_AUDIT.md)
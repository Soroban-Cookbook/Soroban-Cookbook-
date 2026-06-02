# Token Patterns Guide

A synthesis of the token examples in this cookbook into reusable patterns.
Each section names the pattern, shows the key code, explains the tradeoffs,
and gives concrete do/don't guidance.

---

## Contents

1. [Metadata: name, symbol, decimals, URI](#1-metadata-name-symbol-decimals-uri)
2. [Mint and Burn](#2-mint-and-burn)
3. [Wrapping an Existing Asset](#3-wrapping-an-existing-asset)
4. [Storage Layout for Token State](#4-storage-layout-for-token-state)
5. [Access Control for Token Operations](#5-access-control-for-token-operations)
6. [Events](#6-events)
7. [Decision Guide](#7-decision-guide)

---

## 1. Metadata: name, symbol, decimals, URI

### What to store

| Field | Type | Mutable | Storage tier |
| --- | --- | --- | --- |
| `name` | `String` | yes | instance |
| `symbol` | `String` | yes | instance |
| `decimals` | `u32` | **no** | instance |
| `uri` | `String` | yes | instance |

All four fields live in instance storage because they are read on almost
every external call and instance reads are cheaper than persistent reads.

### Why decimals must be immutable

Changing `decimals` after tokens are in circulation silently reinterprets
every stored balance. A balance of `1_000_0000000` with 7 decimals means
1 000.0 tokens. If decimals were changed to 6, the same integer would mean
10 000.0 — a 10× reinterpretation with no on-chain record of the change.

```rust
// Expose decimals as read-only. There is no update path.
pub fn decimals(env: Env) -> Result<u32, MetadataError> {
    env.storage()
        .instance()
        .get(&DataKey::Decimals)
        .ok_or(MetadataError::NotInitialized)
}
```

### Returning all fields at once

Wallets and indexers typically need all four fields together. A single
`metadata()` call is cheaper than four separate calls.

```rust
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub uri: String,
}

pub fn metadata(env: Env) -> Result<TokenMetadata, MetadataError> {
    Ok(TokenMetadata {
        name:     read_string(&env, &DataKey::Name)?,
        symbol:   read_string(&env, &DataKey::Symbol)?,
        decimals: read_decimals(&env)?,
        uri:      read_string(&env, &DataKey::Uri)?,
    })
}
```

### Governing metadata updates

Only the stored admin may update mutable fields. The admin address is set
once at initialisation and never changed by the metadata contract itself
(add an admin-transfer function if your token requires it).

```rust
pub fn update_metadata(
    env: Env,
    new_name: String,
    new_symbol: String,
    new_uri: String,
) -> Result<(), MetadataError> {
    let admin = read_admin(&env)?;
    admin.require_auth();           // caller must be the stored admin
    require_non_empty(&new_name)?;
    require_non_empty(&new_symbol)?;
    // uri may be empty (signals "not set")
    env.storage().instance().set(&DataKey::Name,   &new_name);
    env.storage().instance().set(&DataKey::Symbol, &new_symbol);
    env.storage().instance().set(&DataKey::Uri,    &new_uri);
    Ok(())
}
```

**Do:**

- Validate that `name` and `symbol` are non-empty before storing.
- Emit an event on every metadata update so indexers can track changes.
- Document which fields are mutable in your README and contract doc comment.

**Don't:**

- Expose a `set_decimals` function. If you need to change precision, deploy
  a new token and migrate balances.
- Store metadata in persistent storage — it is read far more often than it
  is written, and instance storage is cheaper for that access pattern.

---

## 2. Mint and Burn

### Mint: admin-only supply expansion

Minting increases both the recipient's balance and the total supply. Both
writes must succeed atomically — use checked arithmetic to prevent overflow.

```rust
pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
    require_positive(amount)?;
    let admin = read_admin(&env)?;
    admin.require_auth();

    let new_supply = read_total_supply(&env)
        .checked_add(amount)
        .ok_or(Error::ArithmeticOverflow)?;
    let new_balance = read_balance(&env, &to)
        .checked_add(amount)
        .ok_or(Error::ArithmeticOverflow)?;

    env.storage().instance().set(&DataKey::TotalSupply, &new_supply);
    env.storage().persistent().set(&DataKey::Balance(to.clone()), &new_balance);
    env.events().publish((NS, EV_MINT, to), amount);
    Ok(())
}
```

### Burn: self-service supply contraction

Burning is the inverse of minting. The token holder authorises the burn;
no admin approval is required. Subtract from balance first, then from supply.

```rust
pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
    require_positive(amount)?;
    from.require_auth();

    let balance = read_balance(&env, &from);
    if balance < amount {
        return Err(Error::InsufficientBalance);
    }

    let supply = read_total_supply(&env);
    env.storage().instance().set(&DataKey::TotalSupply, &(supply - amount));
    env.storage().persistent().set(&DataKey::Balance(from.clone()), &(balance - amount));
    env.events().publish((NS, EV_BURN, from), amount);
    Ok(())
}
```

### Tradeoffs

| Approach | Pros | Cons |
| --- | --- | --- |
| Admin-only mint | Simple, predictable supply | Single point of failure if admin key is lost |
| Open mint with cap | Decentralised issuance | Requires careful cap enforcement |
| Burn-on-transfer fee | Deflationary without explicit burn call | Complicates accounting; surprises integrators |

**Do:**

- Always update both the per-address balance and the total supply in the
  same transaction. A mismatch between the two is a critical accounting bug.
- Emit distinct `mint` and `burn` events so indexers can reconstruct supply
  history without scanning every transfer.
- Use `checked_add` / `checked_sub` for all supply and balance arithmetic.

**Don't:**

- Allow minting to the zero address or to the contract's own address unless
  you have a specific reason (e.g. a reserve pool).
- Silently cap a mint at the maximum supply — return an error instead so the
  caller knows the full amount was not minted.

---

## 3. Wrapping an Existing Asset

Wrapping creates a Soroban-native representation of an existing token. Users
deposit the underlying token and receive wrapped shares at a 1:1 ratio.

### Core invariant

```text
underlying token balance held by wrapper >= wrapped total supply
```

In normal operation the values are exactly equal. The wrapper must check this
invariant before every unwrap to protect against undercollateralisation caused
by administrative clawback on the underlying asset.

### Deposit (wrap)

```rust
pub fn wrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
    require_positive(amount)?;
    let underlying = read_underlying(&env)?;
    user.require_auth();

    // Pull underlying tokens into the wrapper contract.
    let wrapper = env.current_contract_address();
    TokenClient::new(&env, &underlying).transfer(&user, &wrapper, &amount);

    // Mint exactly `amount` wrapped shares.
    let new_balance = read_balance(&env, &user)
        .checked_add(amount)
        .ok_or(WrapperError::ArithmeticOverflow)?;
    let new_supply = read_total_supply(&env)
        .checked_add(amount)
        .ok_or(WrapperError::ArithmeticOverflow)?;

    env.storage().persistent().set(&DataKey::Balance(user.clone()), &new_balance);
    env.storage().instance().set(&DataKey::TotalSupply, &new_supply);
    env.events().publish((EVENT_WRAP, user), amount);
    Ok(new_balance)
}
```

### Withdrawal (unwrap)

```rust
pub fn unwrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
    require_positive(amount)?;
    let underlying = read_underlying(&env)?;
    let balance = read_balance(&env, &user);
    if balance < amount {
        return Err(WrapperError::InsufficientWrappedBalance);
    }

    // Verify the invariant before releasing collateral.
    let wrapper = env.current_contract_address();
    let collateral = TokenClient::new(&env, &underlying).balance(&wrapper);
    let supply = read_total_supply(&env);
    if collateral < supply {
        return Err(WrapperError::NotFullyBacked);
    }

    user.require_auth();

    env.storage().persistent().set(&DataKey::Balance(user.clone()), &(balance - amount));
    env.storage().instance().set(&DataKey::TotalSupply, &(supply - amount));
    TokenClient::new(&env, &underlying).transfer(&wrapper, &user, &amount);
    env.events().publish((EVENT_UNWRAP, user), amount);
    Ok(balance - amount)
}
```

### Tradeoffs

| Design choice | Pros | Cons |
| --- | --- | --- |
| 1:1 fixed ratio | Simple accounting, no price oracle needed | Cannot represent rebasing or fee-on-transfer tokens |
| Variable ratio (vault shares) | Handles yield-bearing underlyings | More complex; requires price calculation on every operation |
| Clawback guard | Protects users from undercollateralisation | Blocks all unwraps if even 1 unit is clawed back |

**Do:**

- Check the backing invariant before every unwrap, not just at initialisation.
- Emit separate `wrap` and `unwrap` events so indexers can track collateral
  flows independently of internal transfers.
- Expose a `backing()` query so wallets can display the collateralisation
  ratio without calling the underlying token contract directly.

**Don't:**

- Allow the wrapper to hold more than one underlying token type — this
  complicates the invariant and the accounting.
- Assume the underlying balance equals the wrapped supply after deployment.
  A direct transfer to the wrapper address creates a surplus; handle it
  explicitly in `backing()`.

---

## 4. Storage Layout for Token State

### Recommended layout

| Key | Storage tier | Rationale |
| --- | --- | --- |
| `Admin` | instance | Read on every admin call; rarely changes |
| `Name`, `Symbol`, `Decimals`, `Uri` | instance | Read frequently; small and stable |
| `TotalSupply` | instance | Updated on every mint/burn; benefits from instance caching |
| `Balance(Address)` | persistent | Per-user; must survive instance TTL expiry |
| `Allowance(Address, Address)` | persistent | Per-pair; long-lived |

### Why balances must be persistent

Instance storage is tied to the contract instance's TTL. If the instance
entry expires and is restored, instance data is reset. Balances stored in
instance storage would be lost. Persistent storage has its own TTL per key
and is the correct tier for any data that must survive indefinitely.

```rust
// Correct: balance in persistent storage
fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

// Wrong: balance in instance storage — data lost if instance expires
fn read_balance_wrong(env: &Env, user: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}
```

**Do:**

- Use a typed `#[contracttype]` enum for all storage keys. Raw symbol keys
  are error-prone and undocumented.
- Extend persistent entry TTLs when writing balances in high-traffic
  contracts to avoid unexpected expiry.

**Don't:**

- Mix storage tiers for the same logical concept (e.g. storing some balances
  in instance and others in persistent).
- Use temporary storage for balances or allowances — temporary entries expire
  after a short TTL and cannot be relied upon for financial state.

---

## 5. Access Control for Token Operations

### Typical permission matrix

| Operation | Who can call |
| --- | --- |
| `initialize` | Anyone (once) |
| `mint` | Admin only |
| `burn` | Token holder (self-service) |
| `transfer` | Token holder (self-service) |
| `update_metadata` | Admin only |
| `pause` / `unpause` | Admin only |

### Admin authentication pattern

Always load the admin from storage and call `require_auth()` on the stored
address, not on a caller-supplied argument.

```rust
fn read_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
    let admin = read_admin(&env)?;
    admin.require_auth();   // verifies the stored admin signed the transaction
    // ...
}
```

### Emergency pause

A boolean pause flag in instance storage lets the admin halt all non-admin
operations instantly without a timelock.

```rust
pub fn require_unpaused(env: &Env) -> Result<(), Error> {
    let paused: bool = env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        Err(Error::ContractPaused)
    } else {
        Ok(())
    }
}

// Every non-admin entry point calls this first:
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
    require_unpaused(&env)?;
    from.require_auth();
    // ...
}
```

**Do:**

- Protect `initialize` with an already-initialised guard so it cannot be
  called twice.
- Use a multisig or DAO address as the admin in production, not a single
  private key.
- Emit an event when the pause state changes so monitoring systems can react.

**Don't:**

- Accept the admin address as a function argument and call `require_auth()`
  on it without also checking it matches the stored admin. An attacker can
  pass their own address and satisfy `require_auth()`.
- Gate the pause function itself behind the pause check — the admin must
  always be able to unpause.

---

## 6. Events

### Recommended event schema

Use a consistent `(namespace, action, [keys…])` topic layout across all
token events. Put non-indexed payload (amounts, structs) in the data field.

```rust
// Topics:  (namespace, action, primary_address)
// Data:    amount

env.events().publish((NS, EV_MINT,     to.clone()),         amount);
env.events().publish((NS, EV_BURN,     from.clone()),       amount);
env.events().publish((NS, EV_XFER,     from.clone(), to),   amount);
env.events().publish((NS, EV_META,     admin.clone()),      metadata_struct);
env.events().publish((NS, EV_PAUSE,    admin.clone()),      timestamp);
```

### Why structured topics matter

Off-chain indexers filter by topic position. A consistent layout means:

```
All events from this token:   topic[0] == NS
All mints:                    topic[0] == NS  AND  topic[1] == "mint"
All mints to Alice:           topic[0] == NS  AND  topic[1] == "mint"  AND  topic[2] == Alice
```

**Do:**

- Define event name constants at the top of the file with `symbol_short!`.
- Emit an event for every state-changing operation, including metadata
  updates and pause/unpause transitions.
- Keep the namespace symbol unique per contract to avoid collisions when
  multiple token contracts are deployed.

**Don't:**

- Put large payloads (e.g. full metadata structs) in topics — topics are
  indexed and stored more expensively than data.
- Omit events from admin operations. Silent admin actions are a red flag
  for auditors and monitoring systems.

---

## 7. Decision Guide

Use this table to choose the right pattern for your token.

| I need to… | Pattern to use | Example |
| --- | --- | --- |
| Expose name/symbol/decimals | [Metadata](#1-metadata-name-symbol-decimals-uri) | `token-metadata` |
| Issue new tokens | [Mint](#2-mint-and-burn) | `token-metadata` |
| Reduce supply | [Burn](#2-mint-and-burn) | `token-metadata` |
| Wrap a classic Stellar asset | [Wrapping](#3-wrapping-an-existing-asset) | `token-wrapper` |
| Prevent undercollateralisation | [Backing invariant](#3-wrapping-an-existing-asset) | `token-wrapper` |
| Store balances safely | [Persistent storage](#4-storage-layout-for-token-state) | both |
| Gate operations to admin | [Admin auth](#5-access-control-for-token-operations) | both |
| Halt operations in emergency | [Pause guard](#5-access-control-for-token-operations) | `proxy-admin` |
| Track supply changes off-chain | [Events](#6-events) | both |

---

## Related Resources

- [`examples/tokens/token-metadata`](../examples/tokens/token-metadata/) — full metadata + mint/burn example
- [`examples/tokens/token-wrapper`](../examples/tokens/token-wrapper/) — wrapping pattern with backing invariant
- [`examples/advanced/03-proxy-admin`](../examples/advanced/03-proxy-admin/) — pause + upgrade governance
- [`docs/common-patterns.md`](./common-patterns.md) — general Soroban patterns
- [`docs/best-practices.md`](./best-practices.md) — security and efficiency rules

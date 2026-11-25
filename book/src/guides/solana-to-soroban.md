# Migrating from Solana to Soroban

A guide for Solana (native and Anchor) developers moving to Soroban smart
contracts on Stellar. You already write Rust, so the language carries over.
What changes is the execution model: how state is stored, how callers are
authorized, and how contracts talk to each other.

> Coming from Solidity instead? See [Ethereum to Soroban](./ethereum-to-soroban.md).

## Quick Comparison

| Concept | Solana / Anchor | Soroban |
| --- | --- | --- |
| Code unit | Program (BPF/SBF) | Contract (WASM) |
| State | Separate accounts passed into every instruction | Key-value storage owned by the contract |
| Deterministic addresses | PDAs (`find_program_address`) | Storage keys; contract addresses from deployer + salt |
| Caller identity | `is_signer` on an account | `Address::require_auth()` |
| Framework | Anchor (`#[program]`, `#[derive(Accounts)]`) | `soroban-sdk` (`#[contract]`, `#[contractimpl]`) |
| Keeping state alive | Rent / rent-exempt minimum balance | TTL and state archival |
| Calling other programs | CPI (`invoke`, `invoke_signed`) | Typed cross-contract clients |
| Tokens | SPL Token / Token-2022 | Stellar Asset Contract (SAC) and the token interface |
| Errors | `#[error_code]`, `require!` | `#[contracterror]`, `Result<T, E>` |
| Costs | Compute units + priority fees | Resource fees (CPU, memory, ledger I/O, events) |
| Upgrades | Upgrade authority on the program | `update_current_contract_wasm` behind your own auth |

## 1. Accounts vs Contract Storage

On Solana a program is stateless. Every piece of data lives in an account that
the client must find and pass into the instruction, and the program checks
ownership and layout.

**Anchor:**

```rust
#[account]
pub struct Counter {
    pub count: u64,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

pub fn increment(ctx: Context<Increment>) -> Result<()> {
    ctx.accounts.counter.count += 1;
    Ok(())
}
```

On Soroban the contract owns its state. Clients pass plain arguments; the
contract reads and writes its own storage. There is no account list to
validate and no account size to allocate.

**Soroban:**

```rust
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNTER");

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    pub fn increment(env: Env) -> u64 {
        let count: u64 = env.storage().instance().get(&COUNTER).unwrap_or(0) + 1;
        env.storage().instance().set(&COUNTER, &count);
        count
    }
}
```

What this means in practice:

- No `space = 8 + ...` calculations and no `realloc`. Values are serialized for
  you and can grow.
- No owner checks on input accounts. Another contract cannot write your
  storage.
- The transaction footprint (which entries are read or written) is computed
  by simulation, not by the client listing accounts by hand.

## 2. PDAs vs Storage Keys and Contract Addresses

PDAs serve two purposes on Solana: a deterministic address for per-user data,
and a "signer" a program controls. Soroban splits these.

**Per-user data.** Use a typed storage key instead of deriving an address.

**Anchor:**

```rust
#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

**Soroban:**

```rust
use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Vault(Address),
}

pub fn init_vault(env: Env, user: Address) {
    user.require_auth();
    let key = DataKey::Vault(user.clone());
    if env.storage().persistent().has(&key) {
        panic!("vault exists");
    }
    env.storage().persistent().set(&key, &0i128);
}
```

The `DataKey::Vault(user)` key plays the role of the `[b"vault", user]` seeds,
with no bump to store and no canonical-bump check to get wrong.

**Program-controlled authority.** A PDA lets a program sign with
`invoke_signed`. A Soroban contract has its own `Address`
(`env.current_contract_address()`) and is automatically authorized for calls it
makes directly, so it can move tokens it holds without any seeds.

**Deterministic contract addresses.** When you need a predictable address for
a whole contract (for example one pool per pair), deploy it from a factory with
a salt:

```rust
use soroban_sdk::{Address, BytesN, Env};

pub fn deploy_pool(env: Env, wasm_hash: BytesN<32>, salt: BytesN<32>) -> Address {
    env.deployer()
        .with_current_contract(salt)
        .deploy_v2(wasm_hash, ())
}
```

The same `(deployer, salt)` always yields the same address, so clients can
compute it ahead of time.

## 3. Signers vs `require_auth`

**Anchor:**

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = owner)]
    pub vault: Account<'info, Vault>,
    pub owner: Signer<'info>,
}
```

**Soroban:**

```rust
pub fn withdraw(env: Env, owner: Address, amount: i128) {
    owner.require_auth();
    let key = DataKey::Vault(owner.clone());
    let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    if amount <= 0 || amount > balance {
        panic!("invalid amount");
    }
    env.storage().persistent().set(&key, &(balance - amount));
}
```

Differences to keep in mind:

- `require_auth` works for any address type: a classic Stellar account, a
  multisig account, or another contract (including custom smart wallets).
- Authorization covers the specific contract, function and arguments, plus a
  nonce, so a signature cannot be replayed for a different call.
- Use `require_auth_for_args` when the payload a user signs should differ from
  the full argument list.
- There is no `msg.sender` or implicit fee payer identity. Pass the acting
  `Address` explicitly and call `require_auth` on it.

## 4. Anchor Macros vs soroban-sdk Macros

| Anchor | Soroban |
| --- | --- |
| `#[program]` module | `#[contract]` struct + `#[contractimpl]` impl block |
| `#[account]` data struct | `#[contracttype]` struct or enum |
| `#[derive(Accounts)]` + constraints | Explicit checks in the function body |
| `#[error_code]` | `#[contracterror]` |
| `#[event]` + `emit!` | `env.events().publish(...)` or `#[contractevent]` |
| IDL (generated) | Contract spec embedded in the WASM (generates typed clients) |
| `initialize` instruction | `__constructor` or an `initialize` function |

Anchor constraints like `has_one`, `constraint = ...` and `close = ...` become
plain Rust in Soroban. That is more code, but every check is visible in one
place:

```rust
pub fn close_vault(env: Env, owner: Address) -> i128 {
    owner.require_auth();
    let key = DataKey::Vault(owner.clone());
    let balance: i128 = env
        .storage()
        .persistent()
        .get(&key)
        .expect("no vault");
    env.storage().persistent().remove(&key);
    balance
}
```

## 5. Rent vs TTL and State Archival

On Solana an account must hold a rent-exempt balance or it can be purged, and
closing an account refunds the lamports.

On Soroban every ledger entry has a time-to-live (TTL) measured in ledgers.
When it expires, persistent and instance entries are archived (and can be
restored), and temporary entries are deleted. You extend TTL explicitly:

```rust
const DAY_IN_LEDGERS: u32 = 17_280;

pub fn touch_vault(env: Env, owner: Address) {
    let key = DataKey::Vault(owner);
    env.storage()
        .persistent()
        .extend_ttl(&key, 7 * DAY_IN_LEDGERS, 30 * DAY_IN_LEDGERS);
}
```

Pick the storage type by lifetime:

| Use | Solana habit | Soroban |
| --- | --- | --- |
| Config, admin | Singleton PDA | `instance()` storage |
| User balances | One PDA per user | `persistent()` storage |
| Short-lived data (nonces, locks) | Account closed after use | `temporary()` storage |

See [Storage Pitfalls](../docs/storage-pitfalls.md) for common mistakes.

## 6. CPI vs Cross-Contract Calls

**Anchor CPI:**

```rust
let cpi_ctx = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    Transfer {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    },
);
token::transfer(cpi_ctx, amount)?;
```

**Soroban cross-contract call:**

```rust
use soroban_sdk::token;

pub fn deposit(env: Env, from: Address, token_id: Address, amount: i128) {
    from.require_auth();
    let client = token::Client::new(&env, &token_id);
    client.transfer(&from, &env.current_contract_address(), &amount);
}
```

- Calls are typed: `contractimport!` or a shared client trait gives you a
  generated client with real Rust signatures, not raw instruction bytes.
- The user's authorization flows through nested calls. A single signature can
  cover `deposit` and the inner `transfer`.
- Re-entrancy into a contract that is already on the call stack is rejected by
  the host.

## 7. SPL Token vs Stellar Asset Contract

Solana keeps balances in per-owner token accounts (often an Associated Token
Account per mint). On Soroban a token is a contract and balances live inside
it, so there is nothing for the user to create before receiving tokens.

| SPL Token | Soroban |
| --- | --- |
| Mint account | Token contract address |
| Token account / ATA | Balance entry inside the token contract |
| `spl_token::transfer` | `token::Client::transfer(from, to, amount)` |
| `approve` delegate | `approve(from, spender, amount, expiration_ledger)` |
| Native SOL | Native XLM through its own SAC address |

Every Stellar classic asset, including XLM, has a Stellar Asset Contract
(SAC) that implements the same token interface, so your contract can accept
any of them through `token::Client`.

## 8. Errors

**Anchor:**

```rust
#[error_code]
pub enum VaultError {
    #[msg("Insufficient funds")]
    InsufficientFunds,
}

require!(vault.amount >= amount, VaultError::InsufficientFunds);
```

**Soroban:**

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    InsufficientFunds = 1,
}

pub fn spend(env: Env, owner: Address, amount: i128) -> Result<(), VaultError> {
    owner.require_auth();
    let balance: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Vault(owner.clone()))
        .unwrap_or(0);
    if balance < amount {
        return Err(VaultError::InsufficientFunds);
    }
    env.storage()
        .persistent()
        .set(&DataKey::Vault(owner), &(balance - amount));
    Ok(())
}
```

Error codes are stable `u32` values that clients see as `Error(Contract, #1)`.
Do not renumber them once deployed.

## 9. Events

**Anchor:** `emit!(Deposited { user, amount });`

**Soroban:**

```rust
use soroban_sdk::symbol_short;

env.events()
    .publish((symbol_short!("deposit"), user.clone()), amount);
```

Topics (the first tuple) are indexed and filterable by RPC; the data value is
not. Put addresses you want to filter by into the topics.

## 10. Testing

Solana tests usually spin up `solana-program-test`, LiteSVM or a local
validator. Soroban tests run the contract in-process with a full host:

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_increment() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(CounterContract, ());
        let client = CounterContractClient::new(&env, &id);
        assert_eq!(client.increment(), 1);
        assert_eq!(client.increment(), 2);
    }
}
```

`env.ledger().set_sequence_number(...)` and `set_timestamp(...)` replace
warping the clock in a test validator. See the [Testing Guide](./testing.md).

## Migration Checklist

- [ ] Replace each account type with a `#[contracttype]` value and a storage key
- [ ] Replace PDA seeds with `DataKey` variants
- [ ] Replace `Signer` checks and Anchor constraints with `require_auth` and explicit checks
- [ ] Choose instance, persistent or temporary storage for each value and plan TTL extension
- [ ] Replace CPIs with typed cross-contract clients
- [ ] Replace SPL token handling with `token::Client`
- [ ] Convert `#[error_code]` to `#[contracterror]` with fixed numeric codes
- [ ] Port tests to `soroban_sdk::testutils`

## Appendix: CosmWasm at a Glance

CosmWasm developers will find Soroban closer to home: both run WASM and
contracts own their storage.

| CosmWasm | Soroban |
| --- | --- |
| `instantiate` / `execute` / `query` entry points | Public functions on a `#[contractimpl]` block |
| `ExecuteMsg` enum dispatch | One function per action; the client calls it by name |
| `cw-storage-plus` `Item` / `Map` | `instance()` / `persistent()` storage with typed keys |
| `info.sender` | Explicit `Address` argument + `require_auth()` |
| Submessages and replies | Synchronous cross-contract calls that return values |
| CW20 | SAC / token interface |
| `migrate` entry point | `update_current_contract_wasm` |

## Resources

- [Soroban smart contract docs](https://developers.stellar.org/docs/build/smart-contracts)
- [Choosing the right storage](https://developers.stellar.org/docs/build/guides/storage/choosing-the-right-storage)
- [Stellar Asset Contract](https://developers.stellar.org/docs/tokens/stellar-asset-contract)
- [soroban-sdk API reference](https://docs.rs/soroban-sdk/latest/soroban_sdk/)
- [Soroban examples](https://github.com/stellar/soroban-examples)
- [Ethereum to Soroban](./ethereum-to-soroban.md)

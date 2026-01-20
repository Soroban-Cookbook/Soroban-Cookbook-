# Token Examples

Token standards and implementations for fungible and semi-fungible tokens on Soroban.

## Examples

### Standard Tokens

- **Basic Token** - Simple fungible token implementation
- **SEP-41 Token** - Stellar token standard compliance
- **Mintable Token** - Token with minting capabilities
- **Burnable Token** - Token with burning capabilities

### Advanced Features

- **Capped Supply** - Token with maximum supply limit
- **Pausable Token** - Ability to pause transfers
- **Snapshot Token** - Balance snapshots for voting/dividends
- **Taxed Token** - Token with automatic fee on transfer

### Token Utilities

- **Token Wrapper** - Wrap existing tokens with new functionality
- **Multi-Token** - Manage multiple token types
- **Token Vesting** - Lock tokens with release schedule
- **Token Airdrop** - Efficient token distribution

### Specialized Tokens

- **Reward Token** - Staking rewards and incentives
- **Governance Token** - Voting power token
- **Rebasing Token** - Elastic supply token
- **Loyalty Points** - Point-based reward system

## 📋 Token Standards

### SEP-41: Token Interface

Soroban's standard token interface:

```rust
pub trait TokenInterface {
    // Metadata
    fn decimals(env: Env) -> u32;
    fn name(env: Env) -> String;
    fn symbol(env: Env) -> String;

    // Supply
    fn total_supply(env: Env) -> i128;

    // Balance
    fn balance(env: Env, id: Address) -> i128;

    // Transfer
    fn transfer(env: Env, from: Address, to: Address, amount: i128);
    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128);

    // Approval
    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);
    fn allowance(env: Env, from: Address, spender: Address) -> i128;
}
```

## 🏗️ Implementation Guide

### Basic Token Structure

```rust
#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if has_administrator(&env) {
            panic!("Already initialized");
        }

        write_administrator(&env, &admin);
        write_decimal(&env, decimal);
        write_name(&env, name);
        write_symbol(&env, symbol);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        check_admin(&env);
        receive_balance(&env, to.clone(), amount);
        env.events().publish((MINT, to), amount);
    }
}
```

### Balance Management

```rust
fn read_balance(env: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    env.storage().persistent().get(&key).unwrap_or(0)
}

fn write_balance(env: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    env.storage().persistent().set(&key, &amount);
}

fn spend_balance(env: &Env, addr: Address, amount: i128) {
    let balance = read_balance(env, addr.clone());
    if balance < amount {
        panic!("Insufficient balance");
    }
    write_balance(env, addr, balance - amount);
}
```

## 🧪 Testing Tokens

```rust
#[test]
fn test_token_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let token = create_token(&env, &admin);

    // Mint tokens
    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    // Transfer
    token.transfer(&user1, &user2, &100);
    assert_eq!(token.balance(&user1), 900);
    assert_eq!(token.balance(&user2), 100);
}

#[test]
fn test_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &owner);

    // Approve
    token.approve(&owner, &spender, &500, &1000);
    assert_eq!(token.allowance(&owner, &spender), 500);

    // Transfer from
    token.transfer_from(&spender, &owner, &recipient, &100);
    assert_eq!(token.allowance(&owner, &spender), 400);
}
```

## 💡 Best Practices

### 1. Use Standard Interface

Implement SEP-41 for maximum compatibility with wallets and dApps.

### 2. Safe Math

Always check for overflows and underflows:

```rust
let new_balance = current_balance.checked_add(amount)
    .expect("Balance overflow");
```

### 3. Events

Emit events for all state changes:

```rust
env.events().publish((TRANSFER, from, to), amount);
env.events().publish((APPROVAL, owner, spender), amount);
```

### 4. Authorization

Require proper authorization for sensitive operations:

```rust
from.require_auth();
```

### 5. Storage Efficiency

Use appropriate storage types and extend TTL:

```rust
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, 100, 100);
```

## 🔒 Security Considerations

### Integer Overflow

```rust
// Use checked operations
amount.checked_add(value).expect("Overflow");
```

### Reentrancy

Rust's borrow checker helps prevent reentrancy, but still be careful with external calls.

### Authorization

```rust
// Always verify the caller
sender.require_auth();
```

### Zero Address

```rust
// Prevent transfers to invalid addresses
if to == Address::zero() {
    panic!("Invalid recipient");
}
```

## 📊 Token Economics

Consider these factors when designing your token:

- **Total Supply**: Fixed vs. dynamic
- **Distribution**: Fair launch, ICO, airdrop
- **Inflation**: Minting schedule and limits
- **Deflation**: Burning mechanisms
- **Utility**: What can token holders do?
- **Governance**: Voting rights and proposals

## 🎯 Use Cases

### Payment Tokens

- Medium of exchange
- Micropayments
- Remittances

### Utility Tokens

- Access to services
- Platform usage fees
- Reward mechanisms

### Security Tokens

- Asset ownership
- Dividend distribution
- Compliance requirements

### Stablecoins

- Price stability
- Collateralization
- Algorithmic pegs

## 📚 Additional Resources

- [SEP-41 Token Standard](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
- [Soroban Token SDK](https://docs.rs/soroban-token-sdk/)
- [Token Security Best Practices](https://developers.stellar.org/docs/smart-contracts/security)

## 🤝 Contributing

Building a new token pattern? See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

**Build the next generation of tokens on Stellar!**

# Migrating from Ethereum to Soroban

A guide for Ethereum developers transitioning to Soroban smart contract development on Stellar.

## 🔄 Overview

If you're coming from Ethereum/Solidity development, this guide will help you understand the key differences and translate your knowledge to Soroban.

## 🗂️ Key Differences

| Aspect          | Ethereum/Solidity        | Soroban/Rust                             |
| --------------- | ------------------------ | ---------------------------------------- |
| **Language**    | Solidity                 | Rust                                     |
| **Compilation** | EVM bytecode             | WebAssembly (WASM)                       |
| **Gas Model**   | Per-operation gas        | Resource fees (CPU, memory, storage)     |
| **Storage**     | Permanent by default     | 3 types: Persistent, Temporary, Instance |
| **Numbers**     | uint256, etc.            | i128, u64, etc. with explicit types      |
| **Security**    | Reentrancy guards needed | Borrow checker prevents many issues      |

## 📝 Syntax Comparison

### Contract Declaration

**Solidity:**

```solidity
contract MyContract {
    uint256 public value;

    constructor(uint256 _value) {
        value = _value;
    }
}
```

**Soroban:**

```rust
#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn initialize(env: Env, value: u64) {
        env.storage().persistent().set(&symbol_short!("value"), &value);
    }
}
```

### State Variables

**Solidity:**

```solidity
mapping(address => uint256) public balances;
address public owner;
```

**Soroban:**

```rust
// Storage is accessed through env.storage()
// No automatic public getters - must implement explicitly

pub fn get_balance(env: Env, account: Address) -> i128 {
    env.storage().persistent()
        .get(&balance_key(&account))
        .unwrap_or(0)
}

pub fn get_owner(env: Env) -> Address {
    env.storage().instance().get(&symbol_short!("owner")).unwrap()
}
```

### Functions

**Solidity:**

```solidity
function transfer(address to, uint256 amount) public {
    require(balances[msg.sender] >= amount, "Insufficient balance");
    balances[msg.sender] -= amount;
    balances[to] += amount;
    emit Transfer(msg.sender, to, amount);
}
```

**Soroban:**

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth(); // Equivalent to msg.sender

    let from_balance = read_balance(&env, &from);
    if from_balance < amount {
        panic!("Insufficient balance");
    }

    write_balance(&env, &from, from_balance - amount);
    write_balance(&env, &to, read_balance(&env, &to) + amount);

    env.events().publish(
        (symbol_short!("transfer"), from, to),
        amount
    );
}
```

### Modifiers

**Solidity:**

```solidity
modifier onlyOwner() {
    require(msg.sender == owner, "Not owner");
    _;
}

function adminFunction() public onlyOwner {
    // Function logic
}
```

**Soroban:**

```rust
// No modifiers - use regular functions
fn require_owner(env: &Env, caller: &Address) {
    let owner: Address = env.storage().instance()
        .get(&symbol_short!("owner"))
        .unwrap();

    if caller != &owner {
        panic!("Not owner");
    }
}

pub fn admin_function(env: Env, caller: Address) {
    require_owner(&env, &caller);
    caller.require_auth();
    // Function logic
}
```

### Events

**Solidity:**

```solidity
event Transfer(address indexed from, address indexed to, uint256 value);

emit Transfer(msg.sender, recipient, amount);
```

**Soroban:**

```rust
// Events are published through env.events()
env.events().publish(
    (symbol_short!("transfer"), from.clone(), to.clone()),
    amount
);

// Topics can be tuples of up to 4 elements
// The last parameter is the data
```

### Errors

**Solidity:**

```solidity
error InsufficientBalance(uint256 available, uint256 required);

revert InsufficientBalance(balance, amount);
```

**Soroban:**

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InsufficientBalance = 1,
    Unauthorized = 2,
}

// Usage:
if balance < amount {
    return Err(Error::InsufficientBalance);
}
// Or panic for unrecoverable errors
panic!("Insufficient balance");
```

## 🔐 Authorization

### msg.sender vs require_auth()

**Solidity:**

```solidity
function withdraw(uint256 amount) public {
    require(balances[msg.sender] >= amount);
    balances[msg.sender] -= amount;
    payable(msg.sender).transfer(amount);
}
```

**Soroban:**

```rust
pub fn withdraw(env: Env, account: Address, amount: i128) {
    // Explicitly require authorization from the account
    account.require_auth();

    let balance = read_balance(&env, &account);
    if balance < amount {
        panic!("Insufficient balance");
    }

    write_balance(&env, &account, balance - amount);
}
```

Key difference: In Soroban, you explicitly specify which address must authorize the transaction.

## 💾 Storage Patterns

### Solidity Storage

```solidity
// Automatically persistent
mapping(address => uint256) public balances;
uint256 public totalSupply;
```

### Soroban Storage

```rust
// Three storage types with different lifecycles

// Persistent: Like Solidity storage
pub fn set_balance(env: &Env, addr: &Address, amount: i128) {
    env.storage().persistent().set(&balance_key(addr), &amount);
}

// Instance: Contract-lifetime data (config, admin)
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&symbol_short!("admin"), admin);
}

// Temporary: Single-ledger lifetime (for calculations)
pub fn set_temp_flag(env: &Env, flag: bool) {
    env.storage().temporary().set(&symbol_short!("flag"), &flag);
}
```

## 🔢 Number Types

**Solidity:**

- uint256, uint128, uint8, etc.
- int256, int128, etc.
- Fixed sizes, overflow wraps by default (post-0.8.0 reverts)

**Soroban:**

- i128, u64, u32, i32, etc.
- Overflow checking in debug mode
- Explicit conversions required

```rust
let a: u64 = 100;
let b: u64 = 200;
let sum = a + b; // 300

// No implicit conversions
let c: i128 = a as i128; // Explicit cast required
```

## 🔄 Control Flow

Both languages are similar, but Rust requires explicit returns:

**Solidity:**

```solidity
function max(uint a, uint b) public pure returns (uint) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
```

**Soroban:**

```rust
pub fn max(a: u64, b: u64) -> u64 {
    if a > b {
        a  // implicit return (no semicolon)
    } else {
        b
    }
}

// Or use match
pub fn max(a: u64, b: u64) -> u64 {
    match a > b {
        true => a,
        false => b,
    }
}
```

## 🧪 Testing

**Solidity (Hardhat):**

```javascript
describe("MyContract", function () {
  it("Should transfer tokens", async function () {
    const [owner, addr1] = await ethers.getSigners();
    const MyContract = await ethers.deployContract("MyContract");

    await MyContract.transfer(addr1.address, 100);
    expect(await MyContract.balanceOf(addr1.address)).to.equal(100);
  });
});
```

**Soroban:**

```rust
#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    let addr1 = Address::generate(&env);

    client.transfer(&addr1, &100);
    assert_eq!(client.balance(&addr1), 100);
}
```

## 🛡️ Security Considerations

### Reentrancy

**Solidity:** Requires careful guard implementation

```solidity
bool private locked;

modifier noReentrancy() {
    require(!locked, "Reentrant call");
    locked = true;
    _;
    locked = false;
}
```

**Soroban:** Rust's borrow checker prevents many reentrancy issues at compile time. The language design makes classic reentrancy attacks much harder.

### Integer Overflow

**Solidity:** Post 0.8.0, reverts on overflow
**Soroban:** Checked in debug mode, unchecked in release (configurable)

```rust
// Explicitly handle overflow
let result = a.checked_add(b).expect("Overflow occurred");
```

## 🚀 Deployment

**Ethereum:**

```bash
npx hardhat run scripts/deploy.js --network mainnet
```

**Soroban:**

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_contract.wasm \
  --source alice \
  --network mainnet
```

## 📚 Common Patterns Translation

### Token Standard

**ERC-20 (Solidity):**

```solidity
interface IERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address recipient, uint256 amount) external returns (bool);
}
```

**Soroban Token:**

```rust
pub trait TokenTrait {
    fn total_supply(env: Env) -> i128;
    fn balance(env: Env, account: Address) -> i128;
    fn transfer(env: Env, from: Address, to: Address, amount: i128);
}
```

### Access Control

**Ownable (Solidity):**

```solidity
contract Ownable {
    address private _owner;

    constructor() {
        _owner = msg.sender;
    }

    modifier onlyOwner() {
        require(_owner == msg.sender);
        _;
    }
}
```

**Soroban:**

```rust
pub fn initialize(env: Env, admin: Address) {
    env.storage().instance().set(&symbol_short!("admin"), &admin);
}

fn require_admin(env: &Env) -> Address {
    let admin: Address = env.storage().instance()
        .get(&symbol_short!("admin"))
        .unwrap();
    admin.require_auth();
    admin
}
```

## 🎓 Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Essential Rust learning
- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Soroban by Example](https://developers.stellar.org/docs/smart-contracts/example-contracts)
- [Rust vs Solidity](https://arxiv.org/abs/2106.12848)

## 💡 Tips for Ethereum Developers

1. **Embrace Rust's Safety** - The borrow checker is your friend
2. **Think About Storage Costs** - Choose the right storage type
3. **Explicit Authorization** - Use `require_auth()` deliberately
4. **No Inheritance** - Use composition and traits instead
5. **Testing is Built-in** - Use Cargo's integrated testing
6. **Pattern Matching** - Use `match` instead of multiple `if`s
7. **Error Handling** - Use `Result<T, E>` for recoverable errors

## 🤝 Getting Help

- [Stellar Discord](https://discord.gg/stellardev) - Ask questions
- [Stack Exchange](https://stellar.stackexchange.com/) - Q&A
- [GitHub Discussions](https://github.com/stellar/soroban-examples/discussions)

---

**Welcome to Soroban!** The transition from Solidity takes time, but Rust's safety and Soroban's design will help you build more secure smart contracts.

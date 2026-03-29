# Multi-Party Authorization

This example demonstrates patterns for requiring multiple parties to authorize actions in Soroban smart contracts, including threshold signatures, proposal-based approvals, and authorization vectors.

## 📖 What You'll Learn

- **Threshold Signatures**: Require N-of-M signers to approve actions
- **Proposal-Based Approvals**: Sequential approval workflow for multi-party decisions
- **Authorization Vectors**: Require multiple addresses in a single transaction
- **Multi-Sig Patterns**: Common use cases for multi-party authorization

## 🔍 Contract Overview

The contract implements three complementary multi-party authorization patterns:

### 1. Proposal-Based Multi-Sig (Threshold Pattern)

```rust
pub fn initialize(env: Env, threshold: u32, signers: Vec<Address>) -> Result<(), AuthError>
pub fn create_proposal(env: Env, proposer: Address) -> Result<u32, AuthError>
pub fn approve(env: Env, proposal_id: u32, signer: Address) -> Result<(), AuthError>
pub fn execute(env: Env, proposal_id: u32, executor: Address) -> Result<bool, AuthError>
pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, AuthError>
```

This pattern allows signers to approve proposals over multiple transactions. Once the threshold is met, anyone can execute the proposal.

**Use Cases:**
- Multi-sig wallets
- Treasury management
- DAO governance
- High-value transactions

### 2. Single-Transaction Multi-Auth

```rust
pub fn multi_auth_action(env: Env, signers: Vec<Address>) -> bool
```

Requires all specified addresses to authorize within a single transaction. All signers must sign the transaction before it's submitted.

**Use Cases:**
- Atomic multi-party agreements
- Joint account operations
- Simultaneous approvals

### 3. All-Signers Required

```rust
pub fn require_all_signers(env: Env) -> Result<bool, AuthError>
```

Requires authorization from all configured signers in the contract. Useful for critical operations that need unanimous consent.

**Use Cases:**
- Contract upgrades
- Emergency shutdowns
- Critical configuration changes

## 💡 Key Concepts

### Threshold Signatures

A threshold signature scheme requires M-of-N signers to approve an action:

```rust
// Initialize with 2-of-3 threshold
let signers = vec![&env, alice, bob, charlie];
client.initialize(&2, &signers);

// Create proposal
let proposal_id = client.create_proposal(&alice).unwrap();

// Collect approvals (need 2)
client.approve(&proposal_id, &alice).unwrap();
client.approve(&proposal_id, &bob).unwrap();

// Execute once threshold is met
client.execute(&proposal_id, &alice).unwrap();
```

### Authorization Vectors

Soroban allows multiple `require_auth()` calls in a single function. Each address must sign the transaction:

```rust
pub fn multi_auth_action(env: Env, signers: Vec<Address>) -> bool {
    for signer in signers.iter() {
        signer.require_auth();  // Each must authorize
    }
    true
}
```

When calling this function, all signers must sign the transaction before submission.

### Proposal State Management

Proposals track approvals and execution status:

```rust
#[contracttype]
pub struct Proposal {
    pub approvals: Vec<Address>,
    pub executed: bool,
}
```

This prevents:
- Double approvals from the same signer
- Execution before threshold is met
- Re-execution of completed proposals

## 🔒 Security Considerations

### Best Practices

1. **Validate threshold on initialization** — Ensure threshold ≤ number of signers and > 0
2. **Check signer authorization** — Only allow configured signers to create/approve proposals
3. **Prevent double approvals** — Track which signers have already approved
4. **Prevent re-execution** — Mark proposals as executed and check before executing
5. **Use persistent storage** — Proposals must survive across ledgers

### Common Pitfalls

❌ **Not validating threshold**
```rust
// Bad: No validation
client.initialize(&0, &signers);  // Invalid!
client.initialize(&5, &signers);  // More than signers!
```

✅ **Proper validation**
```rust
if threshold == 0 || threshold > signers.len() {
    panic!("Invalid threshold");
}
```

❌ **Allowing double approvals**
```rust
// Bad: No duplicate check
proposal.approvals.push_back(signer);
```

✅ **Check for duplicates**
```rust
if proposal.approvals.contains(&signer) {
    panic!("Already approved");
}
```

## 🎯 Real-World Use Cases

### Multi-Sig Wallet

```rust
// 2-of-3 wallet for shared funds
let owners = vec![&env, alice, bob, charlie];
client.initialize(&2, &owners);

// Propose withdrawal
let proposal_id = client.create_proposal(&alice);

// Collect approvals
client.approve(&proposal_id, &alice);
client.approve(&proposal_id, &bob);

// Execute withdrawal
client.execute(&proposal_id, &alice);
```

### DAO Treasury

```rust
// 5-of-9 council for treasury decisions
let council = vec![&env, /* 9 council members */];
client.initialize(&5, &council);

// Propose funding allocation
let proposal_id = client.create_proposal(&council_member);

// Council members approve over time
// Once 5 approve, execute the allocation
```

### Joint Business Account

```rust
// Both partners must approve
let partners = vec![&env, partner_a, partner_b];
client.initialize(&2, &partners);

// Any major decision requires both signatures
```

## 🧪 Testing

```bash
cargo test
```

Tests cover:
- ✅ Initialization with valid/invalid thresholds
- ✅ Proposal creation by authorized signers
- ✅ Approval workflow and duplicate prevention
- ✅ Execution with/without threshold met
- ✅ Prevention of double execution
- ✅ Multi-auth in single transaction
- ✅ All-signers requirement
- ✅ Unauthorized signer rejection

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/multi_party_auth.wasm \
  --source alice \
  --network testnet

# Initialize with signers
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- initialize \
  --threshold 2 \
  --signers '["<ALICE_ADDRESS>", "<BOB_ADDRESS>", "<CHARLIE_ADDRESS>"]'
```

## 🔄 Comparison with Single-Party Auth

| Aspect | Single-Party | Multi-Party |
|--------|-------------|-------------|
| **Signers** | One address | Multiple addresses |
| **Approval** | Immediate | Sequential or simultaneous |
| **Security** | Single point of failure | Distributed trust |
| **Complexity** | Simple | More complex state management |
| **Use Case** | Personal accounts | Shared resources, governance |

## 🎓 Next Steps

- [Authentication Basics](../../basics/03-authentication/) - Single-party auth patterns
- [Governance Examples](../../governance/) - DAO voting and proposals
- [DeFi Examples](../../defi/) - Multi-sig in financial protocols
- [Advanced Patterns](../../advanced/) - Cross-contract multi-party auth

## 📚 References

- [Soroban Authorization](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/authorization)
- [Multi-Signature Wallets](https://developers.stellar.org/docs/smart-contracts/example-contracts/multi-sig)
- [Soroban SDK Auth](https://docs.rs/soroban-sdk/latest/soroban_sdk/auth/index.html)

---

**Pattern Summary**: Multi-party authorization distributes trust across multiple signers, requiring threshold approval for sensitive operations. Use proposal-based patterns for asynchronous approvals and authorization vectors for atomic multi-party transactions.

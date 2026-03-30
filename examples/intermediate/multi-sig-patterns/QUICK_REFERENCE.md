# Multi-Party Authorization Quick Reference

## Pattern Selection Guide

### Use Proposal-Based Multi-Sig When:
- ✅ Signers approve asynchronously (different times)
- ✅ Need to track approval history
- ✅ Want to allow execution by anyone once threshold is met
- ✅ Building multi-sig wallets or DAO treasuries

### Use Single-Transaction Multi-Auth When:
- ✅ All signers can coordinate to sign together
- ✅ Need atomic approval (all or nothing)
- ✅ Want immediate execution
- ✅ Building joint accounts or simultaneous agreements

### Use All-Signers Pattern When:
- ✅ Critical operations require unanimous consent
- ✅ Contract upgrades or emergency actions
- ✅ Small, trusted signer set
- ✅ Maximum security is priority

## Code Snippets

### Initialize Multi-Sig
```rust
let signers = vec![&env, alice, bob, charlie];
client.initialize(&2, &signers);  // 2-of-3 threshold
```

### Create and Approve Proposal
```rust
// Create
let proposal_id = client.create_proposal(&alice);

// Approve (each signer)
client.approve(&proposal_id, &alice);
client.approve(&proposal_id, &bob);

// Execute once threshold met
client.execute(&proposal_id, &alice);
```

### Single-Transaction Multi-Auth
```rust
let signers = vec![&env, alice, bob];
client.multi_auth_action(&signers);
// All signers must sign the transaction
```

### Require All Signers
```rust
client.require_all_signers();
// All configured signers must authorize
```

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Invalid threshold` | threshold = 0 or > signers | Set 0 < threshold ≤ signers |
| `Not an authorized signer` | Address not in signer list | Only configured signers can act |
| `Already approved` | Signer approved twice | Each signer approves once |
| `Threshold not met` | Not enough approvals | Collect more approvals |
| `Already executed` | Proposal executed twice | Check execution status |

## Security Checklist

- [ ] Validate threshold on initialization
- [ ] Check signer authorization before actions
- [ ] Prevent double approvals
- [ ] Prevent re-execution
- [ ] Use persistent storage for proposals
- [ ] Test all error conditions
- [ ] Document signer responsibilities
- [ ] Plan for signer key rotation

## Testing Template

```rust
#[test]
fn test_your_scenario() {
    let env = Env::default();
    env.mock_all_auths();  // Mock all auth checks
    
    let contract_id = env.register_contract(None, MultiPartyAuth);
    let client = MultiPartyAuthClient::new(&env, &contract_id);
    
    // Setup signers
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let signers = vec![&env, alice.clone(), bob.clone()];
    
    // Initialize
    client.initialize(&2, &signers);
    
    // Your test logic here
}
```

## Deployment Checklist

- [ ] Build contract: `cargo build --target wasm32-unknown-unknown --release`
- [ ] Run tests: `cargo test`
- [ ] Deploy to testnet first
- [ ] Initialize with correct threshold and signers
- [ ] Test proposal workflow on testnet
- [ ] Document signer addresses
- [ ] Plan emergency procedures
- [ ] Deploy to mainnet
- [ ] Verify initialization
- [ ] Monitor first transactions

## Resources

- [Full README](./README.md) - Detailed documentation
- [Contract Source](./src/lib.rs) - Implementation
- [Tests](./src/test.rs) - Test examples
- [Soroban Auth Docs](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/authorization)

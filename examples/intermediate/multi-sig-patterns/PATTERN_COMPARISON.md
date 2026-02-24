# Multi-Party Authorization Pattern Comparison

## Scenario: Treasury Withdrawal

Let's compare how different multi-party authorization patterns handle a treasury withdrawal scenario.

### Pattern 1: Proposal-Based Multi-Sig (Asynchronous)

**Best for**: Signers in different time zones, large signer sets, audit trails

```rust
// Day 1: Alice proposes withdrawal
let proposal_id = client.create_proposal(&alice);

// Day 2: Bob approves
client.approve(&proposal_id, &bob);

// Day 3: Charlie approves (threshold met: 2-of-3)
client.approve(&proposal_id, &charlie);

// Day 4: Anyone can execute
client.execute(&proposal_id, &alice);
```

**Advantages:**
- ✅ Signers don't need to coordinate timing
- ✅ Clear approval history
- ✅ Can collect approvals over days/weeks
- ✅ Anyone can execute once threshold is met

**Disadvantages:**
- ❌ Requires multiple transactions (gas costs)
- ❌ More complex state management
- ❌ Proposals can become stale

---

### Pattern 2: Single-Transaction Multi-Auth (Synchronous)

**Best for**: Small teams, coordinated actions, immediate execution

```rust
// All signers coordinate and sign the same transaction
let signers = vec![&env, alice, bob];
client.multi_auth_action(&signers);
// Executed immediately
```

**Advantages:**
- ✅ Atomic execution (all or nothing)
- ✅ Single transaction (lower gas)
- ✅ Immediate result
- ✅ Simpler state management

**Disadvantages:**
- ❌ All signers must be available simultaneously
- ❌ Coordination overhead
- ❌ No approval history
- ❌ Difficult with large signer sets

---

### Pattern 3: All-Signers Required (Unanimous)

**Best for**: Critical operations, small trusted groups, maximum security

```rust
// Requires ALL configured signers (no threshold)
client.require_all_signers();
// All 3 signers must authorize
```

**Advantages:**
- ✅ Maximum security (unanimous consent)
- ✅ Simple logic (no threshold calculation)
- ✅ Clear requirement (everyone must agree)

**Disadvantages:**
- ❌ Single signer can block action
- ❌ Not practical for large groups
- ❌ No flexibility (all or nothing)

---

## Real-World Example: DAO Treasury

### Scenario Setup
- **Treasury**: 1,000,000 tokens
- **Council**: 9 members
- **Threshold**: 5-of-9 (majority)
- **Action**: Withdraw 100,000 tokens for development

### Using Proposal-Based Multi-Sig

```rust
// Initialize DAO treasury
let council = vec![&env, /* 9 council members */];
client.initialize(&5, &council);

// Week 1: Council member proposes funding
let proposal_id = client.create_proposal(&council_member_1);

// Week 2: Members review and approve
client.approve(&proposal_id, &council_member_1);
client.approve(&proposal_id, &council_member_2);
client.approve(&proposal_id, &council_member_3);

// Week 3: More approvals come in
client.approve(&proposal_id, &council_member_4);
client.approve(&proposal_id, &council_member_5);  // Threshold met!

// Week 4: Execute the withdrawal
client.execute(&proposal_id, &treasurer);
// Funds released to development team
```

**Why this pattern?**
- Council members are distributed globally
- Need time for discussion and review
- Want transparent approval history
- 5-of-9 threshold allows progress even if some members are unavailable

---

## Decision Matrix

| Requirement | Proposal-Based | Single-Transaction | All-Signers |
|-------------|----------------|-------------------|-------------|
| Async approvals | ✅ Best | ❌ No | ❌ No |
| Immediate execution | ❌ No | ✅ Best | ✅ Best |
| Audit trail | ✅ Best | ❌ No | ❌ No |
| Low gas cost | ❌ Multiple txs | ✅ Single tx | ✅ Single tx |
| Large signer sets | ✅ Scales well | ❌ Coordination hard | ❌ Impractical |
| Threshold flexibility | ✅ Configurable | ⚠️ Manual | ❌ All required |
| Emergency actions | ⚠️ Slower | ✅ Fast | ✅ Fast |
| Signer unavailability | ✅ Tolerant | ❌ Blocks | ❌ Blocks |

---

## Hybrid Approach

You can combine patterns for different operations:

```rust
// Normal operations: 3-of-5 threshold
client.initialize(&3, &signers);
let proposal_id = client.create_proposal(&signer1);
// ... collect approvals ...
client.execute(&proposal_id, &executor);

// Emergency pause: require all signers immediately
client.require_all_signers();
```

---

## Pattern Selection Flowchart

```
Start
  |
  ├─ Need immediate execution?
  |    ├─ Yes → Can all signers coordinate?
  |    |         ├─ Yes → Single-Transaction Multi-Auth
  |    |         └─ No → Proposal-Based Multi-Sig
  |    └─ No → Proposal-Based Multi-Sig
  |
  ├─ Critical operation requiring unanimous consent?
  |    └─ Yes → All-Signers Required
  |
  └─ Large signer set (>5)?
       └─ Yes → Proposal-Based Multi-Sig
```

---

## Code Example: Combining Patterns

```rust
#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    // Regular withdrawals: proposal-based
    pub fn propose_withdrawal(env: Env, proposer: Address, amount: i128) -> u32 {
        // Use proposal pattern
    }
    
    // Emergency pause: all signers required immediately
    pub fn emergency_pause(env: Env) {
        let signers: Vec<Address> = env.storage().instance().get(&SIGNERS_KEY).unwrap();
        for signer in signers.iter() {
            signer.require_auth();  // All must authorize NOW
        }
        // Pause contract
    }
    
    // Quick team decision: coordinated multi-auth
    pub fn team_action(env: Env, team_members: Vec<Address>) {
        for member in team_members.iter() {
            member.require_auth();  // All team members sign together
        }
        // Execute team decision
    }
}
```

---

## Summary

- **Proposal-Based**: Best for most multi-sig scenarios, especially with distributed teams
- **Single-Transaction**: Best for small, coordinated teams needing immediate execution
- **All-Signers**: Best for critical operations requiring unanimous consent

Choose based on your specific requirements for timing, coordination, security, and gas costs.

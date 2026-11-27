# Multi-Contract Testing Patterns

This example demonstrates how to test cross-contract interactions using Soroban's mock test environment, including registering multiple contracts, testing cross-contract calls, and verifying state across contract boundaries.

## 📖 What You'll Learn

- **Registering Multiple Contracts**: Set up two or more contracts in the same test environment
- **Cross-Contract Calls**: Test Contract A calling Contract B's methods
- **State Verification**: Verify state changes on both sides of a cross-contract call
- **Mock All Auths**: Use `env.mock_all_auths()` for automatic authorization
- **Targeted Mock Auths**: Use `env.mock_auths()` for precise authorization control
- **Authorization Verification**: Inspect the authorization call tree
- **Error Handling**: Test cross-contract call failures and state consistency

## 🔍 Example Contracts

This example includes two interacting contracts that demonstrate a realistic multi-contract scenario:

### 1. Treasury Contract

A treasury that holds funds and allows authorized withdrawals:

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), TreasuryError>
pub fn deposit(env: Env, depositor: Address, amount: i128) -> Result<(), TreasuryError>
pub fn withdraw(env: Env, admin: Address, recipient: Address, amount: i128) -> Result<(), TreasuryError>
pub fn balance(env: Env) -> i128
pub fn total_withdrawn(env: Env) -> i128
```

**Key Features:**
- Admin-controlled withdrawals
- Balance tracking
- Event emission for deposits and withdrawals
- Authorization checks with `require_auth()`

### 2. Governance Contract

A governance system that can authorize treasury withdrawals through proposals:

```rust
pub fn initialize(env: Env, treasury: Address, voters: Vec<Address>, threshold: u32) -> Result<(), GovernanceError>
pub fn propose(env: Env, proposer: Address, recipient: Address, amount: i128) -> Result<u32, GovernanceError>
pub fn vote(env: Env, proposal_id: u32, voter: Address) -> Result<(), GovernanceError>
pub fn execute(env: Env, proposal_id: u32, executor: Address) -> Result<(), GovernanceError>
pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal>
```

**Key Features:**
- Proposal-based treasury withdrawals
- Vote threshold requirements
- Cross-contract calls to Treasury
- State tracking for proposals

## 💡 Key Testing Patterns

### Pattern 1: Registering Multiple Contracts

```rust
let env = Env::default();

// Register both contracts in the same environment
let treasury_id = env.register_contract(None, Treasury);
let governance_id = env.register_contract(None, Governance);

// Create clients for both contracts
let treasury = TreasuryClient::new(&env, &treasury_id);
let governance = GovernanceClient::new(&env, &governance_id);
```

**What This Demonstrates:**
- Multiple contracts can share the same test environment
- Each contract has its own contract ID and storage
- Clients allow easy interaction with registered contracts

### Pattern 2: Cross-Contract Call with State Verification

```rust
// Initialize governance as treasury admin
treasury.initialize(&governance_id);
governance.initialize(&treasury_id, &voters, &2);

// Fund treasury
treasury.deposit(&depositor, &1000);

// Create and execute proposal (triggers cross-contract call)
let proposal_id = governance.propose(&voter1, &recipient, &300);
governance.vote(&proposal_id, &voter1);
governance.vote(&proposal_id, &voter2);
governance.execute(&proposal_id, &voter1);  // Calls treasury.withdraw()

// Verify state on BOTH contracts
assert_eq!(treasury.balance(), 700);  // Treasury state updated
assert!(governance.is_executed(&proposal_id));  // Governance state updated
```

**What This Demonstrates:**
- Cross-contract calls work seamlessly in test environment
- State changes on both contracts can be verified
- The calling contract (Governance) can invoke the called contract (Treasury)

### Pattern 3: Using `mock_all_auths()`

```rust
let env = Env::default();
env.mock_all_auths();  // Automatically mock all authorization checks

// Now any address can pass require_auth() checks
let depositor = Address::generate(&env);
treasury.deposit(&depositor, &500);  // No explicit auth setup needed

let admin = Address::generate(&env);
treasury.withdraw(&admin, &recipient, &200);  // Works automatically
```

**When to Use `mock_all_auths()`:**
- ✅ Testing happy path workflows
- ✅ Complex multi-step scenarios where auth isn't the focus
- ✅ Rapid prototyping and development
- ❌ Testing authorization logic specifically
- ❌ When you need to verify auth was called correctly

### Pattern 4: Using `mock_auths()` for Targeted Control

```rust
// Mock authorization for ONLY a specific call
env.mock_auths(&[soroban_sdk::MockAuth {
    address: &depositor,
    invoke: &soroban_sdk::MockAuthInvoke {
        contract: &treasury_id,
        fn_name: "deposit",
        args: (depositor.clone(), 1000i128).into_val(&env),
        sub_invokes: &[],  // No sub-invocations
    },
}]);

treasury.deposit(&depositor, &1000);
```

**When to Use `mock_auths()`:**
- ✅ Testing authorization failures
- ✅ Verifying specific auth patterns
- ✅ Testing cross-contract authorization chains
- ✅ Fine-grained control over which calls are authorized

### Pattern 5: Cross-Contract Authorization with Sub-Invocations

```rust
// Mock voter authorization with sub-invocation for cross-contract call
env.mock_auths(&[soroban_sdk::MockAuth {
    address: &voter1,
    invoke: &soroban_sdk::MockAuthInvoke {
        contract: &governance_id,
        fn_name: "execute",
        args: (proposal_id, voter1.clone()).into_val(&env),
        sub_invokes: &[soroban_sdk::MockAuth {
            address: &governance_id,  // Governance authorizes treasury call
            invoke: &soroban_sdk::MockAuthInvoke {
                contract: &treasury_id,
                fn_name: "withdraw",
                args: (governance_id.clone(), recipient.clone(), 800i128).into_val(&env),
                sub_invokes: &[],
            },
        }],
    },
}]);

governance.execute(&proposal_id, &voter1);
```

**What This Demonstrates:**
- Authorization chains across contract boundaries
- The calling contract (Governance) authorizes the treasury call
- Sub-invocations allow modeling complex auth trees

### Pattern 6: Verifying Authorization Calls

```rust
env.mock_all_auths();

// Perform operations
treasury.deposit(&depositor, &1000);
governance.execute(&proposal_id, &voter1);

// Inspect authorization structure
let auth_snapshot = env.auths();

// Verify specific authorizations occurred
let voter_auth = auth_snapshot
    .iter()
    .find(|(addr, _)| addr == &voter1)
    .expect("Should have auth for voter1");

assert_eq!(
    voter_auth.1.first().unwrap().function,
    AuthorizedFunction::Contract((
        governance_id.clone(),
        Symbol::new(&env, "execute"),
        (proposal_id, voter1.clone()).into_val(&env),
    ))
);
```

**What This Demonstrates:**
- Verification that authorization was called correctly
- Inspection of the authorization tree structure
- Validation of function arguments in auth calls

## 🔒 Testing Error Scenarios

### Cross-Contract Call Failures

```rust
// Proposal requests more than treasury has
let proposal_id = governance.propose(&voter1, &recipient, &1000);
governance.vote(&proposal_id, &voter1);

// Execute fails due to insufficient balance
let result = governance.try_execute(&proposal_id, &voter1);
assert!(result.is_err());

// Verify states are consistent - proposal NOT executed
assert!(!governance.is_executed(&proposal_id));
assert_eq!(treasury.balance(), 0);  // Unchanged
```

**What This Tests:**
- Cross-contract call errors are propagated correctly
- Failed calls don't corrupt state
- Both contracts remain in consistent state after failure

### Unauthorized Cross-Contract Calls

```rust
// Initialize treasury with DIFFERENT admin (not governance)
let other_admin = Address::generate(&env);
treasury.initialize(&other_admin);

// Governance can't withdraw because it's not the admin
let result = governance.try_execute(&proposal_id, &voter1);
assert_eq!(result, Err(Ok(GovernanceError::TreasuryCallFailed)));
```

**What This Tests:**
- Authorization is enforced across contract boundaries
- Unauthorized cross-contract calls fail appropriately
- Error types are preserved through contract boundaries

## 🧪 Test Coverage

The example includes 15 comprehensive tests:

### Basic Setup
- ✅ Registering multiple contracts in one environment
- ✅ Contract state isolation

### Cross-Contract Interactions
- ✅ Cross-contract call with state verification on both sides
- ✅ Cross-contract call failure handling
- ✅ Unauthorized cross-contract call attempts

### Authorization Patterns
- ✅ `mock_all_auths()` for simple scenarios
- ✅ `mock_all_auths()` with complex workflows
- ✅ Targeted `mock_auths()` for specific calls
- ✅ Cross-contract authorization with sub-invocations
- ✅ Authorization call tree verification

### Business Logic
- ✅ Multi-voter threshold requirements
- ✅ Double-voting prevention
- ✅ Proposal execution after threshold met

### Edge Cases
- ✅ Insufficient votes for execution
- ✅ State consistency after failures
- ✅ Multiple contract instances with isolated state

## 🚀 Running the Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_cross_contract_call_with_state_verification

# Run tests matching pattern
cargo test cross_contract
```

### Expected Output

```
running 15 tests
test test::test_contract_state_isolation ... ok
test test::test_cross_contract_call_failure_handling ... ok
test test::test_cross_contract_call_with_state_verification ... ok
test test::test_cross_contract_with_targeted_auth ... ok
test test::test_double_voting_prevention ... ok
test test::test_mock_all_auths_complex_workflow ... ok
test test::test_mock_all_auths_pattern ... ok
test test::test_multi_voter_threshold ... ok
test test::test_register_multiple_contracts ... ok
test test::test_targeted_mock_auths ... ok
test test::test_unauthorized_cross_contract_call ... ok
test test::test_verify_authorization_calls ... ok

test result: ok. 15 passed; 0 failed
```

## 💡 Best Practices

### DO ✅

1. **Register all contracts at the start of each test**
   ```rust
   let treasury_id = env.register_contract(None, Treasury);
   let governance_id = env.register_contract(None, Governance);
   ```

2. **Use `mock_all_auths()` for happy path testing**
   ```rust
   env.mock_all_auths();  // Simple and effective for most tests
   ```

3. **Verify state on both sides of cross-contract calls**
   ```rust
   // Check both contracts after cross-contract interaction
   assert_eq!(treasury.balance(), expected_balance);
   assert!(governance.is_executed(&proposal_id));
   ```

4. **Test error propagation across contract boundaries**
   ```rust
   let result = governance.try_execute(&proposal_id, &voter);
   assert!(result.is_err());
   // Verify both contracts are in consistent state
   ```

5. **Use descriptive test names**
   ```rust
   #[test]
   fn test_cross_contract_call_with_state_verification() { ... }
   ```

### DON'T ❌

1. **Don't share test state between tests**
   ```rust
   // Bad: Reusing env across tests
   static ENV: Env = ...;
   
   // Good: Create fresh env per test
   let env = Env::default();
   ```

2. **Don't forget to test both success and failure cases**
   ```rust
   // Test both paths
   governance.execute(&valid_proposal_id, &voter);  // Should succeed
   let result = governance.try_execute(&invalid_id, &voter);  // Should fail
   ```

3. **Don't use `mock_all_auths()` when testing authorization logic**
   ```rust
   // Bad: Can't test auth failures with mock_all_auths
   env.mock_all_auths();
   treasury.withdraw(&unauthorized, &recipient, &100);  // Won't fail!
   
   // Good: Use targeted mocks or no mocks when testing auth
   let result = treasury.try_withdraw(&unauthorized, &recipient, &100);
   assert!(result.is_err());
   ```

4. **Don't forget to initialize contracts before testing**
   ```rust
   // Always initialize before using
   treasury.initialize(&admin);
   governance.initialize(&treasury_id, &voters, &threshold);
   ```

## 🔍 Common Patterns

### Pattern: Testing a Multi-Contract Protocol

```rust
#[test]
fn test_full_protocol_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    // 1. Register all contracts
    let treasury_id = env.register_contract(None, Treasury);
    let governance_id = env.register_contract(None, Governance);
    
    // 2. Initialize with correct relationships
    treasury.initialize(&governance_id);  // Governance is admin
    governance.initialize(&treasury_id, &voters, &2);
    
    // 3. Test the full workflow
    treasury.deposit(&depositor, &1000);
    let proposal_id = governance.propose(&voter1, &recipient, &300);
    governance.vote(&proposal_id, &voter1);
    governance.vote(&proposal_id, &voter2);
    governance.execute(&proposal_id, &voter1);  // Cross-contract call
    
    // 4. Verify end state
    assert_eq!(treasury.balance(), 700);
    assert!(governance.is_executed(&proposal_id));
}
```

### Pattern: Testing Authorization Chains

```rust
#[test]
fn test_authorization_chain() {
    let env = Env::default();
    
    // Use targeted mocks to test exact auth flow
    env.mock_auths(&[soroban_sdk::MockAuth {
        address: &caller,
        invoke: &soroban_sdk::MockAuthInvoke {
            contract: &contract_a_id,
            fn_name: "method_a",
            args: (param1, param2).into_val(&env),
            sub_invokes: &[soroban_sdk::MockAuth {
                address: &contract_a_id,
                invoke: &soroban_sdk::MockAuthInvoke {
                    contract: &contract_b_id,
                    fn_name: "method_b",
                    args: (param3,).into_val(&env),
                    sub_invokes: &[],
                },
            }],
        },
    }]);
    
    contract_a_client.method_a(&param1, &param2);
    
    // Verify auth was called correctly
    let auths = env.auths();
    assert!(!auths.is_empty());
}
```

### Pattern: Testing Multiple Instances

```rust
#[test]
fn test_multiple_instances() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Create two instances of the same contract
    let treasury1_id = env.register_contract(None, Treasury);
    let treasury2_id = env.register_contract(None, Treasury);
    
    let treasury1 = TreasuryClient::new(&env, &treasury1_id);
    let treasury2 = TreasuryClient::new(&env, &treasury2_id);
    
    // Each instance has isolated state
    treasury1.deposit(&depositor, &1000);
    treasury2.deposit(&depositor, &2000);
    
    assert_eq!(treasury1.balance(), 1000);
    assert_eq!(treasury2.balance(), 2000);
}
```

## 📚 Additional Resources

- [Soroban Testing Documentation](https://soroban.stellar.org/docs/getting-started/hello-world#tests)
- [Authorization Guide](https://soroban.stellar.org/docs/learn/authorization)
- [Contract Clients](https://soroban.stellar.org/docs/learn/interacting-with-contracts)
- [Testing Best Practices](../../docs/testing-best-practices.md)

## 🎯 Real-World Applications

This pattern is essential for testing:

- **DeFi Protocols**: DEX calling lending pools, routers calling AMMs
- **DAO Governance**: Governance contracts controlling treasury contracts
- **Multi-Contract Systems**: Any protocol involving multiple interacting contracts
- **Proxy Patterns**: Proxy contracts calling implementation contracts
- **Factory Patterns**: Factory contracts deploying and initializing child contracts

## 🔧 Troubleshooting

### "Contract not found" errors

**Cause**: Trying to call a contract that wasn't registered or using the wrong ID.

**Solution**: Ensure you register the contract and use the returned ID:
```rust
let contract_id = env.register_contract(None, MyContract);
let client = MyContractClient::new(&env, &contract_id);
```

### Authorization failures with `mock_all_auths()`

**Cause**: Authorization setup happens after the contract call.

**Solution**: Call `env.mock_all_auths()` before making contract calls:
```rust
let env = Env::default();
env.mock_all_auths();  // Before any contract calls
contract.some_method();
```

### Cross-contract call fails but should succeed

**Cause**: The called contract might not be initialized or relationship not set up correctly.

**Solution**: Ensure both contracts are initialized with correct parameters:
```rust
treasury.initialize(&governance_id);  // Set governance as admin
governance.initialize(&treasury_id, ...);  // Set treasury address
```

### State changes don't persist across calls

**Cause**: Using instance storage incorrectly or not setting TTL.

**Solution**: Use persistent storage for data that needs to survive:
```rust
env.storage().persistent().set(&key, &value);
```

---

**Need Help?** Check out the [Testing Best Practices](../../docs/testing-best-practices.md) or ask in the [Stellar Discord](https://discord.gg/stellardev).

# Governance Examples

Decentralized governance systems and DAO (Decentralized Autonomous Organization) implementations.

## Examples

### Voting Systems

- **Simple Voting** - Basic proposal and voting system
- **Weighted Voting** - Token-weighted voting power
- **Quadratic Voting** - Quadratic voting mechanism
- **Delegation** - Vote delegation system

### DAO Frameworks

- **Basic DAO** - Simple DAO with treasury
- **Multi-Sig DAO** - Multi-signature governance
- **Token-Gated DAO** - Token-holder governance
- **NFT-Based DAO** - NFT holder governance

### Proposal Systems

- **Timelock** - Time-delayed execution
- **Veto System** - Emergency veto mechanism
- **Proposal Lifecycle** - Draft → Vote → Execute
- **Cancellation** - Cancel pending proposals

### Treasury Management

- **DAO Treasury** - Manage DAO funds
- **Streaming Payments** - Vested payments over time
- **Budget Allocation** - Departmental budgets
- **Grant System** - Community grant management

## 🏛️ Key Concepts

### 1. Proposals

```rust
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub actions: Vec<Action>,
    pub start_time: u64,
    pub end_time: u64,
    pub yes_votes: i128,
    pub no_votes: i128,
    pub executed: bool,
}
```

### 2. Voting Power

Different voting power calculations:

```rust
// Token-based (1 token = 1 vote)
pub fn get_voting_power_token(env: Env, voter: Address) -> i128 {
    token.balance(&voter)
}

// NFT-based (1 NFT = 1 vote)
pub fn get_voting_power_nft(env: Env, voter: Address) -> i128 {
    nft.balance_of(&voter)
}

// Reputation-based
pub fn get_voting_power_reputation(env: Env, voter: Address) -> i128 {
    reputation.get(&voter)
}
```

### 3. Quorum and Thresholds

```rust
pub struct VotingConfig {
    pub quorum_percentage: u32,        // Minimum participation
    pub approval_threshold: u32,       // Minimum yes votes
    pub voting_period: u64,            // Duration in ledgers
}
```

## 🗳️ Voting Patterns

### Simple Majority

```rust
// Requires >50% of votes to pass
pub fn check_passed(proposal: &Proposal) -> bool {
    proposal.yes_votes > proposal.no_votes
}
```

### Supermajority

```rust
// Requires 2/3 approval
pub fn check_passed(proposal: &Proposal) -> bool {
    let total = proposal.yes_votes + proposal.no_votes;
    proposal.yes_votes >= (total * 2 / 3)
}
```

### Quorum + Threshold

```rust
pub fn check_passed(
    proposal: &Proposal,
    total_supply: i128,
    config: &VotingConfig
) -> bool {
    let total_votes = proposal.yes_votes + proposal.no_votes;
    let quorum = total_supply * config.quorum_percentage / 100;

    // Must meet quorum
    if total_votes < quorum {
        return false;
    }

    // Must meet approval threshold
    proposal.yes_votes >= (total_votes * config.approval_threshold / 100)
}
```

## 🔐 Security Considerations

### 1. Proposal Spam Prevention

```rust
// Require minimum token balance to propose
pub fn create_proposal(env: Env, proposer: Address, ...) {
    let balance = token.balance(&proposer);
    if balance < MIN_PROPOSAL_BALANCE {
        panic!("Insufficient balance to propose");
    }
}
```

### 2. Snapshot Voting

```rust
// Capture voting power at proposal creation
// Prevents vote buying during active proposal
pub fn create_proposal(env: Env, proposer: Address, ...) {
    let snapshot_ledger = env.ledger().sequence();
    // Store snapshot for this proposal
}
```

### 3. Timelock for Execution

```rust
// Delay between approval and execution
pub fn execute_proposal(env: Env, proposal_id: u32) {
    let proposal = get_proposal(&proposal_id);
    let current_time = env.ledger().timestamp();

    if current_time < proposal.execution_time {
        panic!("Timelock not expired");
    }

    // Execute proposal
}
```

### 4. Emergency Actions

```rust
// Allow fast-track for critical updates
pub fn emergency_execute(env: Env, proposal_id: u32) {
    // Require higher threshold or specific guardians
    require_emergency_auth(&env);

    // Execute immediately
}
```

## 📊 Voting Strategies

### Token Voting

- **Pros:** Simple, proportional to stake
- **Cons:** Plutocracy risk, whale dominance

### NFT Voting

- **Pros:** One person = one vote (if 1 NFT per person)
- **Cons:** Can still be gamed with multiple wallets

### Quadratic Voting

- **Pros:** Balances minority and majority interests
- **Cons:** More complex, requires sybil resistance

### Delegation

- **Pros:** Increases participation through representatives
- **Cons:** Risk of representative misalignment

## 🧪 Testing Governance Contracts

```rust
#[test]
fn test_proposal_lifecycle() {
    let env = Env::default();
    // Setup contracts...

    // 1. Create proposal
    let proposal_id = dao.create_proposal(&proposer, &"Test", &actions);

    // 2. Vote on proposal
    dao.vote(&voter1, &proposal_id, &true);
    dao.vote(&voter2, &proposal_id, &true);

    // 3. Advance time past voting period
    env.ledger().with_mut(|li| li.timestamp += VOTING_PERIOD);

    // 4. Execute proposal
    dao.execute_proposal(&proposal_id);

    // 5. Verify execution
    assert!(dao.is_executed(&proposal_id));
}

#[test]
fn test_quorum_not_met() {
    // Test proposal fails without quorum
}

#[test]
fn test_voting_power_delegation() {
    // Test vote delegation
}
```

## 🎯 Use Cases

### Protocol Governance

- Parameter updates (fees, limits)
- Protocol upgrades
- Treasury management
- Emergency actions

### Community DAOs

- Grant allocation
- Event planning
- Content curation
- Resource allocation

### Investment DAOs

- Investment decisions
- Portfolio management
- Exit strategies

### Service DAOs

- Service provider selection
- Payment approval
- Quality standards

## 📚 Additional Resources

- [DAO Best Practices](https://developers.stellar.org/docs/smart-contracts/governance)
- [Voting System Design](https://vitalik.ca/general/2021/08/16/voting3.html)
- [DAO Security](https://blog.openzeppelin.com/dao-security)

## 🤝 Contributing

Have a governance pattern to share? See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

**Build the future of decentralized governance on Stellar!**

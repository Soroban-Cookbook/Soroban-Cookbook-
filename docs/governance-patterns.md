# Governance Patterns in Soroban

Governance on Soroban enables decentralized decision-making through voting systems, DAOs, and multi-party controls. This guide covers core governance patterns: simple voting, delegation, multisig governance, proposal lifecycles, and treasury management.

## Why Governance Patterns Matter

On-chain governance requires:
- **Transparent decision-making** with verifiable vote counts
- **Access control** to restrict proposal creation and voting
- **Time-gated execution** for review windows
- **Delegation support** for token holders unable to vote directly
- **Treasury controls** to prevent unauthorized fund movement

Poor governance design can lead to:
- governance capture by a single large token holder
- flash loan attacks manipulating voting power
- rushed execution without community review
- unauthorized fund transfers or protocol changes

## Core Governance Patterns

| Pattern | Focus | Use case |
|---------|-------|----------|
| **Simple Voting** | One-address-one-vote, deadline enforcement | Community polls, basic DAOs |
| **Vote Delegation** | Liquid delegation with cycle prevention | Token-weighted voting, delegated authority |
| **Token Voting** | Governance token balance checks | Large-scale DAOs with token distributions |
| **Time Constraints** | Voting periods, grace periods, quorum | Production governance with formal phases |
| **Proposal Lifecycle** | Full state machine: Draft → Active → Executed | Complex governance flows with multiple gates |
| **Timelock Governance** | Mandatory delay + veto + emergency bypass | Protocol upgrades, large fund movements |
| **DAO Treasury** | Multi-sig approval + timelock + role hierarchy | Decentralized fund management |

## 1. Simple Voting

### What it shows

A foundational voting system with:
- Admin-gated proposal creation
- One-address-one-vote enforcement via persistent storage
- Vote options as typed enums (For, Against, Abstain)
- Time-based deadline checks using `env.ledger().timestamp()`
- Event emission for indexing
- Typed error handling

### Key patterns

**Proposal state stored by ID:**
```rust
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    id: u32,
    title: String,
    deadline: u64,
    votes_for: u32,
    votes_against: u32,
}

#[contracttype]
pub enum DataKey {
    Proposal(u32),
    Vote(u32, Address),  // (proposal_id, voter)
}
```

**Vote storage keyed by (proposal_id, voter):**
```rust
env.storage().persistent()
    .set(&DataKey::Vote(proposal_id, voter), &vote_choice);
```

**Timestamp validation:**
```rust
let deadline: u64 = proposal.deadline;
if env.ledger().timestamp() > deadline {
    return Err(VotingError::VotingClosed);
}
```

### When to use

- Community polls with time-limited voting
- Simple DAO governance with uniform voting weight
- Token airdrops requiring community confirmation
- Basic protocol parameter updates

### Security considerations

- **One-address-one-vote** enforces equal weight; vulnerable to Sybil attacks if addresses are free
- **Deadline enforcement** prevents late votes; clock-dependence risks if validators manipulate timestamps (rare on Stellar)
- **Admin gating** requires trusting the admin key; combine with multisig for higher security

### Example reference

See [`examples/governance/01-simple-voting/`](../examples/governance/01-simple-voting/) for full implementation.

## 2. Vote Delegation

### What it shows

Liquid voting delegation with:
- Addresses delegating their vote to another address
- Chain delegation (A → B → C) supported
- Cycle detection to prevent infinite loops
- Recursion depth limits for safety
- Vote power accumulation in delegated chains

### Key patterns

**Delegation graph stored in persistent storage:**
```rust
#[contracttype]
pub enum DataKey {
    Delegate(Address),        // Maps voter → delegated_to
    VotePower(Address),       // Accumulated vote count at delegation endpoint
}
```

**Vote power calculation with recursion limit:**
```rust
fn get_delegate(env: &Env, voter: &Address, depth: u32) -> Result<Address, Error> {
    if depth > MAX_RECURSION {
        return Err(Error::RecursionLimit);
    }
    
    let delegate: Address = env.storage().persistent()
        .get(&DataKey::Delegate(voter))
        .unwrap_or(voter.clone());
    
    if delegate == *voter {
        Ok(voter.clone())
    } else {
        get_delegate(env, &delegate, depth + 1)
    }
}
```

**Cycle prevention via visited set:**
```rust
fn detect_delegation_cycle(env: &Env, from: &Address, to: &Address) -> Result<(), Error> {
    let mut visited = vec![env, from.clone()];
    let mut current = to.clone();
    
    loop {
        if visited.contains(&current) {
            return Err(Error::CycleDetected);
        }
        visited.push(current.clone());
        
        let delegate: Address = env.storage().persistent()
            .get(&DataKey::Delegate(&current))
            .unwrap_or(current.clone());
        
        if delegate == current {
            return Ok(());
        }
        current = delegate;
    }
}
```

### When to use

- Token-holder governance where not all holders vote directly
- Delegated authority to trusted community members
- Proxy voting systems
- Emergency response with delegation to guardians

### Security considerations

- **Chain delegation** can be slow; depth limits prevent DOS attacks
- **Cycle detection** is critical; untested cycle logic leads to infinite loops
- **Vote power accumulation** can create centralization; monitor delegation patterns off-chain
- **Revocation** is easy but requires active participation; consider lock-in periods

### Example reference

See [`examples/governance/06-vote-delegation/`](../examples/governance/06-vote-delegation/) for full implementation with cycle detection and depth limits.

## 3. Token Voting

### What it shows

Governance voting tied to token balance:
- Token balance snapshots at proposal creation time
- One-token-one-vote enforcement
- Integration with Stellar token standards
- Prevention of vote-then-transfer attacks via snapshot timing
- Withdrawal lock during active voting

### Key patterns

**Proposal includes voting power snapshot:**
```rust
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    id: u32,
    title: String,
    voting_power_snapshot: u64,  // Total eligible votes
    deadline: u64,
}

#[contracttype]
pub enum DataKey {
    TokenBalance(Address, u64),  // Snapshot of balance at block height
    ProposalSnapshot(u32, u64),  // Block height of proposal creation
}
```

**Vote power determined by token balance at snapshot block:**
```rust
fn get_voting_power(env: &Env, voter: &Address, proposal_id: u32) -> Result<u64, Error> {
    let block: u64 = env.storage().persistent()
        .get(&DataKey::ProposalSnapshot(proposal_id))
        .ok_or(Error::ProposalNotFound)?;
    
    let balance: u64 = env.storage().persistent()
        .get(&DataKey::TokenBalance(voter, block))
        .unwrap_or(0);
    
    Ok(balance)
}
```

**Block-height-based snapshots:**
- Proposal created at block N
- Voting power snapshot taken at block N
- Votes cast using balance at block N
- Transfer after block N doesn't affect voting power in this proposal

### When to use

- Token-holder governance (e.g., DAO governance tokens)
- Stake-weighted voting
- Protocol upgrade decisions
- DAO budget allocations

### Security considerations

- **Snapshot timing** is critical; off-by-one errors lead to vote dilution or inflation
- **Balance manipulation** is prevented by snapshotting; confirm snapshot logic is airtight
- **Token wrapping** can introduce voting power inconsistencies; vet token contracts
- **Flash loans** on other blockchains bypass snapshots; Soroban's atomic transactions may have similar risks

### Example reference

See [`examples/governance/02-token-voting/`](../examples/governance/02-token-voting/) for snapshot-based voting implementation.

## 4. Voting Time Constraints

### What it shows

Production-grade voting with formal phases:
- Configurable voting periods (e.g., 7 days open, 3 days grace)
- Quorum thresholds requiring minimum participation
- Early closure when consensus is clear
- Grace period for appeals or reversals
- Proposal state machine: Draft → Active → Grace → Closed

### Key patterns

**Proposal with time phases:**
```rust
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    id: u32,
    created_at: u64,
    voting_start: u64,
    voting_end: u64,
    grace_period_end: u64,
    status: ProposalStatus,
    votes_for: u32,
    votes_against: u32,
    quorum_threshold: u32,
}

#[contracttype]
#[repr(u32)]
pub enum ProposalStatus {
    Draft = 0,
    Active = 1,
    Grace = 2,
    Closed = 3,
}
```

**Early closure with supermajority:**
```rust
fn close_early_if_supermajority(env: &Env, proposal_id: u32) -> Result<(), Error> {
    let proposal = get_proposal(env, proposal_id)?;
    let total_votes = proposal.votes_for + proposal.votes_against;
    let for_percentage = (proposal.votes_for * 100) / total_votes;
    
    if for_percentage >= 80 {  // 80% supermajority
        set_proposal_status(env, proposal_id, ProposalStatus::Closed)?;
    }
    Ok(())
}
```

**Quorum validation at execution:**
```rust
fn execute(env: &Env, proposal_id: u32) -> Result<(), Error> {
    let proposal = get_proposal(env, proposal_id)?;
    let total_votes = proposal.votes_for + proposal.votes_against;
    
    if total_votes < proposal.quorum_threshold {
        return Err(Error::QuorumNotMet);
    }
    if env.ledger().timestamp() <= proposal.grace_period_end {
        return Err(Error::GracePeriodActive);
    }
    
    Ok(())
}
```

### When to use

- DAO governance with required participation minimums
- Protocol parameters requiring broad consensus
- Upgrades with community veto capability
- Governance token holder voting

### Security considerations

- **Quorum games** incentivize low participation; set thresholds to encourage real engagement
- **Grace period abuse** by signers who create proposals; use multisig for proposal creation
- **State machine errors** in phase transitions; test all state paths explicitly
- **Timestamp manipulation** affects voting windows; rare but monitor ledger behavior

### Example reference

See [`examples/governance/03-voting-time-constraints/`](../examples/governance/03-voting-time-constraints/) for configurable voting periods and quorum logic.

## 5. Proposal Lifecycle

### What it shows

Complete proposal state machine with:
- Draft → Active → Voting → Grace → Queued → Executed/Defeated workflow
- Multiple stakeholder roles (Proposer, Voter, Executor)
- Cancellation and veto paths
- Event emission at each state transition
- Detailed state tracking and auditability

### Key patterns

**Full state machine:**
```rust
#[contracttype]
#[repr(u32)]
pub enum ProposalStatus {
    Draft = 0,      // Not yet active
    Active = 1,     // Voting in progress
    Defeated = 2,   // Voting ended, did not pass
    Queued = 3,     // Passed voting, waiting for execution
    Executed = 4,   // Successfully executed
    Canceled = 5,   // Proposal withdrawn
}
```

**State transition validation:**
```rust
fn advance_proposal_state(env: &Env, proposal_id: u32) -> Result<(), Error> {
    let proposal = get_proposal(env, proposal_id)?;
    let now = env.ledger().timestamp();
    
    let new_status = match proposal.status {
        ProposalStatus::Draft if now >= proposal.voting_start => {
            ProposalStatus::Active
        }
        ProposalStatus::Active if now >= proposal.voting_end => {
            check_if_passed(&proposal)
                .map(|_| ProposalStatus::Queued)
                .unwrap_or(ProposalStatus::Defeated)
        }
        _ => return Err(Error::InvalidStateTransition),
    };
    
    set_proposal_status(env, proposal_id, new_status)?;
    env.events().publish((symbol!("governance"), symbol!("state_change")), proposal_id);
    Ok(())
}
```

**Veto and cancellation paths:**
```rust
fn cancel_proposal(env: &Env, proposal_id: u32, guardian: Address) -> Result<(), Error> {
    guardian.require_auth();
    require_guardian(&env, &guardian)?;
    
    let proposal = get_proposal(env, proposal_id)?;
    if proposal.status == ProposalStatus::Executed {
        return Err(Error::CannotCancelExecuted);
    }
    
    set_proposal_status(env, proposal_id, ProposalStatus::Canceled)?;
    env.events().publish((symbol!("governance"), symbol!("canceled")), proposal_id);
    Ok(())
}
```

### When to use

- Complex DAOs requiring formal governance
- Protocol upgrades with multi-stage review
- Large fund movements with veto capability
- Community-driven development decisions

### Security considerations

- **State machine complexity** increases attack surface; test all transitions thoroughly
- **Transition race conditions** occur at deadline boundaries; design idempotent transitions
- **Cancellation authority** must be trustworthy; require multisig for cancellation
- **Event auditing** is critical; emit events at every state change for off-chain monitoring

### Example reference

See [`examples/governance/04-proposal-lifecycle/`](../examples/governance/04-proposal-lifecycle/) for full state machine implementation.

## 6. Timelock Governance

### What it shows

Governance with mandatory execution delays:
- Proposal queue with mandatory delay
- Scheduled execution after delay expires
- Veto capability during delay window
- Emergency bypass for critical situations
- Audit trail via events

### Key patterns

**Queued proposal with execution window:**
```rust
#[contracttype]
#[derive(Clone)]
pub struct QueuedProposal {
    id: u32,
    proposed_action: String,  // Action to execute
    scheduled_execution: u64, // Earliest execution time
    veto_deadline: u64,       // Latest veto time
    status: QueueStatus,
}

#[contracttype]
#[repr(u32)]
pub enum QueueStatus {
    Pending = 0,
    Ready = 1,
    Executed = 2,
    Vetoed = 3,
}
```

**Queue with mandatory delay:**
```rust
fn queue_proposal(env: &Env, proposal_id: u32, delay_seconds: u64) -> Result<(), Error> {
    let proposal = get_proposal(env, proposal_id)?;
    
    let now = env.ledger().timestamp();
    let scheduled_execution = now + delay_seconds;
    let veto_deadline = now + (delay_seconds / 2);  // Veto window is first half of delay
    
    let queued = QueuedProposal {
        id: proposal_id,
        proposed_action: proposal.action,
        scheduled_execution,
        veto_deadline,
        status: QueueStatus::Pending,
    };
    
    env.storage().persistent()
        .set(&DataKey::QueuedProposal(proposal_id), &queued);
    
    env.events().publish(
        (symbol!("governance"), symbol!("queued")),
        scheduled_execution
    );
    Ok(())
}
```

**Veto during window:**
```rust
fn veto(env: &Env, proposal_id: u32, guardian: Address) -> Result<(), Error> {
    guardian.require_auth();
    require_guardian(&env, &guardian)?;
    
    let queued = get_queued_proposal(env, proposal_id)?;
    let now = env.ledger().timestamp();
    
    if now > queued.veto_deadline {
        return Err(Error::VetoWindowClosed);
    }
    if queued.status != QueueStatus::Pending {
        return Err(Error::CannotVeto);
    }
    
    let mut queued = queued;
    queued.status = QueueStatus::Vetoed;
    env.storage().persistent()
        .set(&DataKey::QueuedProposal(proposal_id), &queued);
    
    env.events().publish(
        (symbol!("governance"), symbol!("vetoed")),
        proposal_id
    );
    Ok(())
}
```

**Emergency execution with authorized caller:**
```rust
fn emergency_execute(env: &Env, proposal_id: u32, emergecy_admin: Address) -> Result<(), Error> {
    emergency_admin.require_auth();
    require_emergency_admin(&env, &emergency_admin)?;
    
    let queued = get_queued_proposal(env, proposal_id)?;
    if queued.status == QueueStatus::Executed {
        return Err(Error::AlreadyExecuted);
    }
    
    execute_proposal(env, proposal_id)?;
    
    let mut queued = queued;
    queued.status = QueueStatus::Executed;
    env.storage().persistent()
        .set(&DataKey::QueuedProposal(proposal_id), &queued);
    
    Ok(())
}
```

### When to use

- Protocol upgrades requiring review window
- Large fund transfers with stakeholder notification
- Emergency procedures with staged response
- Production governance with mandatory cooling-off

### Security considerations

- **Delay length** affects governance responsiveness; too short is insecure, too long blocks evolution
- **Veto window** must be long enough for analysis; hidden vetoes erode community trust
- **Emergency bypass** requires trusted guardians; keep count small and rotation frequent
- **Replay prevention** is essential; ensure queued proposals cannot execute twice

### Example reference

See [`examples/governance/09-timelock-governance/`](../examples/governance/09-timelock-governance/) for mandatory delays and veto implementation.

## 7. DAO Treasury

### What it shows

Multi-party fund management contract with:
- Token balance tracking by denomination
- Withdrawal requests with multisig approval
- Role-based access (Treasurer, Guardian, Executor)
- Timelock on large transactions
- Withdrawal cancellation and recovery paths

### Key patterns

**Treasury state with denominations:**
```rust
#[contracttype]
#[repr(u32)]
pub enum TreasuryRole {
    Treasurer = 0,
    Guardian = 1,
    Executor = 2,
}

#[contracttype]
#[derive(Clone)]
pub struct WithdrawalRequest {
    id: u32,
    destination: Address,
    amount: u64,
    token: Address,
    approvals: u32,
    threshold: u32,
    scheduled_time: u64,
    status: WithdrawalStatus,
}

#[contracttype]
#[repr(u32)]
pub enum WithdrawalStatus {
    Pending = 0,
    Approved = 1,
    Executed = 2,
    Canceled = 3,
}
```

**Request and multisig approval:**
```rust
fn request_withdrawal(env: &Env, requester: Address, destination: Address,
                     amount: u64, token: Address) -> Result<u32, Error> {
    requester.require_auth();
    require_role(&env, &requester, TreasuryRole::Treasurer)?;
    
    let request_id = get_next_request_id(env);
    let delay = if amount > LARGE_AMOUNT_THRESHOLD {
        LARGE_WITHDRAWAL_DELAY
    } else {
        NORMAL_WITHDRAWAL_DELAY
    };
    
    let request = WithdrawalRequest {
        id: request_id,
        destination,
        amount,
        token,
        approvals: 0,
        threshold: 2,  // 2-of-3 multisig
        scheduled_time: env.ledger().timestamp() + delay,
        status: WithdrawalStatus::Pending,
    };
    
    env.storage().persistent()
        .set(&DataKey::WithdrawalRequest(request_id), &request);
    
    env.events().publish(
        (symbol!("treasury"), symbol!("withdrawal_requested")),
        (request_id, amount)
    );
    Ok(request_id)
}

fn approve_withdrawal(env: &Env, request_id: u32, approver: Address) -> Result<(), Error> {
    approver.require_auth();
    require_role(&env, &approver, TreasuryRole::Guardian)?;
    
    let mut request = get_withdrawal_request(env, request_id)?;
    if request.status != WithdrawalStatus::Pending {
        return Err(Error::InvalidStatus);
    }
    
    request.approvals += 1;
    if request.approvals >= request.threshold {
        request.status = WithdrawalStatus::Approved;
    }
    
    env.storage().persistent()
        .set(&DataKey::WithdrawalRequest(request_id), &request);
    
    Ok(())
}
```

**Timelock-gated execution:**
```rust
fn execute_withdrawal(env: &Env, request_id: u32, executor: Address) -> Result<(), Error> {
    executor.require_auth();
    require_role(&env, &executor, TreasuryRole::Executor)?;
    
    let mut request = get_withdrawal_request(env, request_id)?;
    
    if request.status != WithdrawalStatus::Approved {
        return Err(Error::NotApproved);
    }
    if env.ledger().timestamp() < request.scheduled_time {
        return Err(Error::TimeLocked);
    }
    
    // Transfer token (assuming token contract interface)
    let client = TokenClient::new(env, &request.token);
    client.transfer(&env.current_contract_address(), &request.destination, &(request.amount as i128));
    
    request.status = WithdrawalStatus::Executed;
    env.storage().persistent()
        .set(&DataKey::WithdrawalRequest(request_id), &request);
    
    env.events().publish(
        (symbol!("treasury"), symbol!("withdrawal_executed")),
        (request_id, request.amount)
    );
    Ok(())
}
```

### When to use

- Decentralized autonomous organizations (DAOs)
- Treasuries managed by multiple councils
- Protocol revenue distribution
- Grant programs and community funds

### Security considerations

- **Role confusion** can lead to privilege escalation; test role checks exhaustively
- **Reentrancy** during token transfer; token interface must be trusted
- **Fund lock** if approvals are misconfigured; set thresholds carefully at initialization
- **Cancellation authority** must be clear; only trustees should cancel pending withdrawals

### Example reference

See [`examples/governance/10-dao-treasury/`](../examples/governance/10-dao-treasury/) for multisig-gated treasury implementation.

## 8. Voting System Comparison

| System | Access | Vote weight | Sybil protection | Scalability | Complexity |
|--------|--------|-------------|------------------|-------------|-----------|
| Simple Voting | Any address | Uniform | None (free accounts) | O(n) | Low |
| Vote Delegation | Any address | Delegated | Delegation-dependent | O(n log n) | Medium |
| Token Voting | Token holders | Balance-based | Liquidity-dependent | O(n log n) | Medium |
| Time Constraints | Gated creation | Threshold-based | Quorum-dependent | O(n) | Medium-High |
| Proposal Lifecycle | Multisig | Multi-stage approval | Multisig-dependent | O(n) | High |
| Timelock Governance | Multisig + Guardian | Threshold-based | Guardian-dependent | O(n) | High |

## 9. Security Considerations for Governance

### Common Vulnerabilities

**Flash Loan Attacks**
- Token balance can be artificially inflated for single transaction
- Mitigation: Use block height snapshots; require historical balance data

**Governance Capture**
- Large token holder can unilaterally pass proposals
- Mitigation: Implement quorum thresholds, delegation, or vote splitting

**Proposal Collision**
- Two simultaneous proposals with same ID
- Mitigation: Use monotonically increasing IDs with overflow checks

**State Machine Races**
- Concurrent calls can transition proposal states unpredictably
- Mitigation: Use semaphore or atomic operations; test all interleaving paths

**Timestamp Dependency**
- Validators can manipulate timestamps within bounds
- Mitigation: Use reasonable deadlines (hours/days, not seconds); assume timestamps are adversarial

### Best Practices

1. **Test all paths**: Draft → Active → Closed → Executed is only one flow; test Draft → Canceled, Draft → Failed, etc.
2. **Emit events**: Every state change should emit an indexed event for off-chain monitoring
3. **Require auth**: Every state-changing call must have `caller.require_auth()`
4. **Validate roles**: Never trust caller-provided role claims; fetch from storage
5. **Use persistent storage**: Governance state must survive contract upgrades
6. **Document thresholds**: Write down quorum%, delay periods, and veto windows clearly
7. **Monitor off-chain**: Keep logs of all governance events for auditability

## 10. Real DAO Examples in This Repository

### Simple DAO Example: 01-simple-voting
Demonstrates the minimum viable voting system: proposal creation, ballot casting, tally, execution.

**When to use as template:**
- Quick prototype of governance concept
- Proof of concept for community polls

### Production DAO Stack: 01-simple-voting → 03-voting-time-constraints → 04-proposal-lifecycle → 09-timelock-governance

**Progression:**
1. Start with simple voting for initial governance
2. Add formal voting periods and quorum for maturity
3. Implement full proposal lifecycle for complex governance
4. Layer timelock + veto for production security

### Token-Holder DAO: 02-token-voting + 10-dao-treasury

**When to use as template:**
- DAO with fungible governance token
- Community treasuries
- Decentralized protocols with voting escrow

## 11. Recommended Deployment Checklist

Before deploying governance to production:

- [ ] All proposal states have event emissions
- [ ] Role checks use `require_role()` not address comparison
- [ ] Timestamps use `env.ledger().timestamp()` consistently
- [ ] Multisig threshold matches security policy (2-of-3, 3-of-5, etc.)
- [ ] Proposal deadlines are in reasonable time frames (hours to weeks)
- [ ] Veto windows exist for large fund movements
- [ ] Quorum threshold encourages real participation (>10% typical)
- [ ] Withdrawal limits prevent accidental fund loss
- [ ] Role assignment is gated by admin multisig
- [ ] Role rotation schedule is documented
- [ ] Off-chain governance process mirrors on-chain flows
- [ ] Emergency procedures are tested with guardians
- [ ] All test paths include failed authorization, timeout, and state race scenarios

## 12. Cross-Pattern References

**RBAC with Governance**
See [`docs/governance-rbac-multisig-timelock.md`](./governance-rbac-multisig-timelock.md) for role hierarchy, multisig threshold patterns, and timelock configuration.

**Events for Auditability**
See [`examples/basics/04-events/`](../examples/basics/04-events/) for event emission patterns used in all governance contracts.

**Authentication Foundations**
See [`examples/basics/03-authentication/`](../examples/basics/03-authentication/) for `require_auth()` and role storage fundamentals.

## 13. Quick Reference: Pattern Selection Decision Tree

```
Start: Need to make a governance decision on-chain?
│
├─ Single decision, limited time?
│  └─ Use: Simple Voting (01-simple-voting)
│
├─ Decisions require input from many token holders?
│  └─ Use: Token Voting (02-token-voting) + Delegation (06-vote-delegation)
│
├─ Need formal phases: voting period → grace → execution?
│  └─ Use: Time Constraints (03-voting-time-constraints) + Lifecycle (04-proposal-lifecycle)
│
├─ Managing shared treasury or protocol upgrades?
│  └─ Use: Timelock (09-timelock-governance) + Treasury (10-dao-treasury)
│
└─ Production DAO with all requirements?
   └─ Use: Stack all patterns: Voting → Delegation → Timelock → Treasury
```

## 14. Next Steps

1. **Try simple-voting** to understand basic voting flow
2. **Add token-voting** to tie governance to economic incentives
3. **Layer timelock** for production governance with review windows
4. **Implement treasury** once governance mechanism is proven

Start with examples in [`examples/governance/`](../examples/governance/), then adapt patterns to your specific governance requirements.

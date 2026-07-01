# 03 — DAO with Treasury

A full-featured Decentralised Autonomous Organisation (DAO) with integrated Treasury management, built entirely on Soroban.

## What this example covers

| Feature | Details |
|---------|---------|
| **Treasury management** | Deposit and withdraw native token balances held by the DAO contract |
| **Proposal types** | `Transfer` (asset disbursement) and `Upgrade` (WASM replacement) |
| **Voting mechanism** | Weighted/token-based: each vote carries a caller-supplied `weight` |
| **Execution queue** | Proposals pass through `Active → Passed → Executed` states; a configurable execution window prevents stale execution |
| **Event emission** | Structured topics + typed payload structs (reuses `04-events` pattern) |
| **Auth pattern** | Every state-mutating call begins with `address.require_auth()` (reuses `03-authentication` pattern) |
| **Tests** | 25+ unit tests covering happy paths, edge cases, and error states |

## Contract API

### Initialisation

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    min_quorum: i128,       // minimum total vote weight needed to pass
    voting_duration: u32,   // ledgers the voting window stays open
    exec_duration: u32,     // ledgers the execution window stays open after voting ends
) -> Result<(), DaoError>
```

### Treasury

```rust
pub fn deposit(env: Env, depositor: Address, amount: i128) -> Result<(), DaoError>
pub fn treasury_balance(env: Env) -> i128
```

### Proposal lifecycle

```rust
// Submit a new proposal – returns the proposal_id
pub fn propose(env: Env, proposer: Address, kind: ProposalKind) -> Result<u32, DaoError>

// Cast a weighted vote
pub fn vote(env: Env, voter: Address, proposal_id: u32, approve: bool, weight: i128) -> Result<(), DaoError>

// Execute a passed proposal (Transfer or Upgrade)
pub fn execute(env: Env, executor: Address, proposal_id: u32) -> Result<(), DaoError>

// Cancel an active proposal (proposer or admin only)
pub fn cancel(env: Env, caller: Address, proposal_id: u32) -> Result<(), DaoError>
```

### Queries

```rust
pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, DaoError>
pub fn proposal_state(env: Env, proposal_id: u32) -> Result<ProposalState, DaoError>
pub fn treasury_balance(env: Env) -> i128
pub fn admin(env: Env) -> Option<Address>
pub fn proposal_count(env: Env) -> u32
pub fn has_voted(env: Env, proposal_id: u32, voter: Address) -> bool
```

## Proposal types

```rust
pub enum ProposalKind {
    Transfer { recipient: Address, amount: i128 },
    Upgrade { new_wasm_hash: Bytes },
}
```

## Proposal state machine

```
          ┌─────────────────────────────────────────────────┐
          │                   Active                        │
          │  (voting window open, votes accumulate)         │
          └─────────────────────────────────────────────────┘
                      │ voting_end_ledger reached
           ┌──────────┴──────────┐
     quorum & yes > no      quorum not met
     & seq ≤ exec_end        or no ≥ yes
           │                      │
        Passed                 Failed
           │
    execute() called
           │
        Executed

 If seq > exec_end_ledger before execute() → Expired
 If cancel() called → Cancelled
```

## Key patterns

### Auth (03-authentication)

Every mutating function authenticates the caller before touching storage:

```rust
pub fn vote(env: Env, voter: Address, ...) -> Result<(), DaoError> {
    voter.require_auth();   // ← host verifies the signature
    // …
}
```

### Events (04-events)

Structured topic layout for efficient off-chain indexing:

```
topic[0]  "dao"           — contract namespace
topic[1]  "propose"       — action name
topic[2]  proposal_id     — primary key (u32)
data      ProposalSubmitted { … }
```

### Lazy state resolution

Proposal state is stored as `Active` and resolved on-the-fly by `resolve_state()`, avoiding the need for a separate cron job to expire proposals:

```rust
fn resolve_state(env: &Env, proposal: &Proposal) -> ProposalState { … }
```

## Running tests

```bash
cargo test -p dao-treasury
```

## Storage layout

| Key | Tier | Type |
|-----|------|------|
| `Admin` | Instance | `Address` |
| `MinQuorum` | Instance | `i128` |
| `VotingDuration` | Instance | `u32` |
| `ExecDuration` | Instance | `u32` |
| `ProposalCount` | Instance | `u32` |
| `TreasuryBalance` | Persistent | `i128` |
| `Proposal(id)` | Persistent | `Proposal` |
| `Voted(id, addr)` | Persistent | `bool` |

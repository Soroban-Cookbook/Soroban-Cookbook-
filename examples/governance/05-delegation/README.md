# 05 — Delegation System

Demonstrates advanced delegation patterns for on-chain governance on Soroban.

## Key Concepts

| Feature | Description |
|---------|-------------|
| **Full delegation** | Delegate 100 % of voting power to another address (`basis_points = 10_000`) |
| **Partial delegation** | Delegate any fraction expressed in basis points (1 bp = 0.01 %) |
| **Topic-specific delegation** | Scope a delegation to a single governance topic (e.g., `"treasury"`) |
| **Delegation registry** | On-chain index of all outgoing / incoming delegations per address |
| **Revocation** | Delegators can revoke any delegation at any time; the record is soft-deleted for auditing |
| **Chain prevention** | A delegate that has already delegated their own power cannot receive further delegations |
| **Effective power** | `effective_power(account, topic)` computes retained + received power for any topic |

## Contract Functions

| Function | Description |
|----------|-------------|
| `initialize(admin)` | Set up the contract |
| `set_voting_power(admin, account, power)` | Assign base voting power (admin only) |
| `get_voting_power(account)` | Read base voting power |
| `delegate(delegator, delegate, basis_points, scope)` | Create a delegation |
| `revoke(delegator, delegate, scope)` | Revoke an existing delegation |
| `get_delegation(delegator, delegate, scope)` | Fetch a single delegation record |
| `get_outgoing(delegator)` | List active delegations from an address |
| `get_incoming(delegate)` | List active delegations to an address |
| `effective_power(account, topic)` | Compute effective voting power for a topic |
| `get_admin()` | Return the current admin |

## Basis-Points Model

Voting power fractions are expressed in basis points:

```
1 bp  = 0.01 %
5000 bp = 50 %
10000 bp = 100 % (full delegation)
```

The sum of all outgoing delegations from a single address cannot exceed
10 000 bp per scope (global or per topic). Global delegations count against
every topic; topic-scoped delegations count only against that specific topic.

## Delegation Scope

```rust
// Delegate all topics
DelegationScope::Global

// Delegate treasury votes only
DelegationScope::Topic(symbol_short!("treasury"))
```

## Effective Power Formula

```
effective_power(account, topic) =
    base_power
    − Σ (base_power × bp / 10_000)  for each outgoing delegation covering topic
    + Σ (delegator_base × bp / 10_000)  for each incoming delegation covering topic
```

A delegation "covers" a topic when it is `Global` or when it is `Topic(t)` with `t == topic`.

## Anti-Chain Rule

To prevent unbounded delegation chains, a delegate cannot accept delegations
if they already have active outgoing delegations of their own.

```
Alice → Bob  ✓  (Bob has no outgoing)
Bob → Carol  ✓  (Bob's own power, Carol has no outgoing)
Alice → Bob  then Bob → Carol  = Bob already has outgoing → Alice cannot delegate to Bob
```

## Event Topics

All events use the namespace `"deleg"` as the first topic:

| Event | Topics | Data |
|-------|--------|------|
| Initialized | `("deleg", "init")` | `admin` |
| Voting power set | `("deleg", "set_vp", account)` | `power` |
| Delegation created | `("deleg", "delegated", delegator, delegate)` | `(basis_points, scope)` |
| Delegation revoked | `("deleg", "revoked", delegator, delegate)` | `scope` |

## Storage

| Key | Tier | Value |
|-----|------|-------|
| `Admin` | Instance | `Address` |
| `VotingPower(addr)` | Persistent | `i128` |
| `Delegation(DelegationId)` | Persistent | `DelegationRecord` |
| `DelegatorOutgoing(addr)` | Persistent | `Vec<DelegationId>` |
| `DelegateIncoming(addr)` | Persistent | `Vec<DelegationId>` |

## How to Run

```bash
# Unit tests (15 tests)
cargo test -p delegation

# WASM release build
cargo build -p delegation --target wasm32-unknown-unknown --release
```

## Tests

| # | Name | What it checks |
|---|------|----------------|
| 1 | `test_initialize_sets_admin` | Admin is stored on init |
| 2 | `test_double_initialize_fails` | Second init returns `AlreadyInitialized` |
| 3 | `test_set_and_get_voting_power` | Admin can set base power |
| 4 | `test_set_voting_power_non_admin_fails` | Non-admin returns `Unauthorized` |
| 5 | `test_get_voting_power_default_zero` | Unregistered address returns 0 |
| 6 | `test_full_delegation_and_effective_power` | 10 000 bp transfers all power |
| 7 | `test_partial_delegation_splits_power` | 30 % + 20 % split verified |
| 8 | `test_topic_delegation_isolated` | Topic delegation doesn't bleed to other topics |
| 9 | `test_delegation_registry_outgoing_and_incoming` | Indexes updated correctly |
| 10 | `test_get_delegation_record` | Record fields match |
| 11 | `test_revoke_restores_power` | Power returns to delegator after revoke |
| 12 | `test_revoke_topic_delegation` | Topic-scoped revoke |
| 13 | `test_revoke_nonexistent_fails` | Returns `DelegationNotFound` |
| 14 | `test_self_delegation_fails` | Returns `SelfDelegation` |
| 15 | `test_zero_basis_points_fails` | Returns `InvalidBasisPoints` |
| 16 | `test_over_10000_basis_points_fails` | Returns `InvalidBasisPoints` |
| 17 | `test_delegation_chain_prevented` | Returns `DelegateeHasOutgoing` |
| 18 | `test_exceeds_total_power_fails` | Returns `ExceedsTotalPower` |
| 19 | `test_duplicate_delegation_fails` | Returns `AlreadyDelegated` |
| 20 | `test_mixed_global_and_topic_power` | Mixed scopes computed correctly |
| 21 | `test_redelegate_after_revoke` | Re-delegation allowed after revoke |
| 22 | `test_empty_registry` | Empty vecs for unregistered addresses |

## Related Examples

- [03-authentication](../../basics/03-authentication/) — `require_auth()` and allowance patterns
- [04-events](../../basics/04-events/) — event topic design conventions
- [01-simple-voting](../01-simple-voting/) — basic proposal + one-address-one-vote
- [04-proposal-lifecycle](../04-proposal-lifecycle/) — weighted voting (`weight: i128`)

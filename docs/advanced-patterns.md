# Advanced Patterns Guide

A comprehensive guide to the advanced Soroban contract patterns in this cookbook, covering architecture decisions, trade-offs, and when to use each pattern.

---

## Table of Contents

1. [Multi-Party Authorization](#1-multi-party-authorization)
2. [Timelock Execution](#2-timelock-execution)
3. [Oracle Patterns](#3-oracle-patterns)
4. [Cross-Chain Bridge](#4-cross-chain-bridge)
5. [Bridge Security](#5-bridge-security)
6. [Upgradeable Proxies](#6-upgradeable-proxies)
7. [Diamond Pattern](#7-diamond-pattern)
8. [Beacon Proxy](#8-beacon-proxy)
9. [Role-Based Access Control](#9-role-based-access-control)
10. [Hierarchical Access Control](#10-hierarchical-access-control)
11. [Batch Operations](#11-batch-operations)
12. [Merkle Proofs](#12-merkle-proofs)
13. [Reentrancy Guard](#13-reentrancy-guard)
14. [Decision Guide](#14-decision-guide)

---

## 1. Multi-Party Authorization

**Location:** `examples/advanced/01-multi-party-auth/`

### What it does
Requires multiple parties to authorize an action before it executes. Supports M-of-N threshold signatures, compact authorization vectors, and sequential approvals.

### Architecture Decisions
- **Authorization vectors** encode signer lists into sorted, deduplicated `Bytes` blobs instead of `Vec<Address>`, reducing storage costs.
- **Canonical sorting** ensures determinism: any input order produces the same blob, enabling use as a storage key.
- **Audit trail events** emit structured payloads for off-chain indexers.

### Trade-offs
| Approach | Storage Cost | Flexibility | Gas Cost |
| --- | --- | --- | --- |
| N-of-N (direct) | None | Low | O(N) auth |
| M-of-N (proposal) | Proposal state | High | O(M+N) storage |
| Auth vectors | Compact Bytes | Medium | O(N log N) encode |

### When to use
- **N-of-N**: Atomic joint agreements, pair-based approvals, high-value transfers requiring unanimity.
- **M-of-N**: DAO governance, corporate treasury, security-critical upgrades.
- **Auth vectors**: Storing signer lists on-chain, cross-contract signer passing.

---

## 2. Timelock Execution

**Location:** `examples/advanced/02-timelock/`

### What it does
Delays action execution by a configurable time window. Users queue operations with a minimum delay, then execute after the delay elapses. Supports cancellation and delay bounds.

### Architecture Decisions
- **Delay bounds** prevent admin from setting delay to zero (bypass) or unreasonably long (lock funds).
- **Pausable** execution allows emergency stop without altering queued operations.
- **Flat storage model** uses a `DataKey` enum instead of hashed keys for clarity.

### Trade-offs
| Feature | Benefit | Cost |
| --- | --- | --- |
| Delay bounds | Safety guarantee | Storage for two values |
| Pause | Emergency control | Centralization risk |
| Queue state tracking | Visibility | Per-operation storage |

### When to use
- Contract upgrades with user opt-out window
- Treasury withdrawals requiring notice period
- Any operation where users need time to react

---

## 3. Oracle Patterns

**Location:** `examples/advanced/03-oracle-pattern/`, `examples/advanced/03-data-aggregation-oracle/`

### Basic Oracle
Single-source oracle with authorized submission and freshness validation. A designated submitter pushes data, and consumers check a timestamp to reject stale values.

### Data Aggregation Oracle
Multi-source oracle that aggregates data from multiple submitters using median, mean, or mode. Includes outlier detection and manipulation resistance.

### Architecture Decisions
- **Freshness threshold** configures how old a value may be before rejection.
- **Submitter whitelist** restricts who can write data.
- **Outlier filtering** drops values more than N standard deviations from the mean.

### When to use
- **Basic oracle**: Single trusted price feed, simple data availability.
- **Aggregation oracle**: DeFi protocols needing manipulation-resistant prices, multi-source feeds.

---

## 4. Cross-Chain Bridge

**Location:** `examples/advanced/03-cross-chain-bridge/`

### What it does
Lock-and-mint bridge pattern where assets are locked on the source chain and minted as wrapped representations on Soroban. Supports validator threshold verification.

### Architecture Decisions
- **Validator set** stored on-chain with M-of-N threshold for release.
- **Nonce-based replay protection** prevents double-processing.
- **Pausable** for emergency scenarios.

### When to use
- Cross-chain asset transfers between Stellar and other ecosystems.
- Wrapped asset creation with decentralized validation.

---

## 5. Bridge Security

**Location:** `examples/advanced/17-bridge-security/`

### What it does
Implements security guards for bridge operations: rate limiting, pause mechanisms, challenge windows, and fraud proofs.

### Architecture Decisions
- **Rate limiting** tracks per-epoch volume and rejects spikes.
- **Challenge window** allows validators to contest a release before finalization.
- **Fraud proofs** enable slashing of malicious validators.

### When to use
Any bridge or cross-chain application requiring:
- Volume caps per time window
- Decentralized dispute resolution
- Economic security via slashing

---

## 6. Upgradeable Proxies

**Location:** `examples/advanced/04-upgradeable-proxy/`

### What it does
Proxy pattern that delegates calls to an implementation contract. Admin can swap the implementation address to upgrade contract logic without changing the contract ID.

### Architecture Decisions
- **EIP-1967-style storage slot** for implementation address to avoid collision with implementation storage.
- **Admin-controlled** upgrade with timelock integration option.

### When to use
- Contracts needing post-deployment logic updates.
- Projects following an iterative development cycle.
- Protocol-owned contracts requiring governance upgrades.

---

## 7. Diamond Pattern

**Location:** `examples/advanced/06-diamond-pattern/` (canonical), `examples/advanced/18-diamond-facets/` (router orchestration), `examples/advanced/19-diamond-security/` (security-focused)

### What it does
Splits contract logic across multiple facet contracts, each responsible for a subset of functions. A diamond proxy routes calls to the appropriate facet, enabling granular upgrades and modular architecture.

**Examples:**
- **[06-diamond-pattern](../examples/advanced/06-diamond-pattern/)** â€” Canonical EIP-2535 adaptation with full diamond-cut operations (Add/Replace/Remove) and diamond-loupe introspection
- **[05-diamond-security](../examples/advanced/05-diamond-security/)** â€” Security-hardened variant demonstrating access control per facet, interface verification, and storage collision prevention
- **[05-diamond-facets](../examples/advanced/05-diamond-facets/)** â€” Router orchestration patterns showing atomic cross-facet operations

### Architecture Decisions
- **Facet-selector mapping** stored in the diamond for O(1) dispatch.
- **Immutable functions** (e.g., `receive`) can be inlined to save gas.
- **Upgrade path** allows adding/replacing facets without touching others.

### Trade-offs
| Approach | Pros | Cons |
| --- | --- | --- |
| Monolithic | Simple, one contract | No upgrade granularity |
| Proxy + 1 impl | Standard upgrade | Single upgrade scope |
| Diamond | Granular upgrades | Dispatch overhead, complexity |

### When to use
- Large contracts exceeding WASM size limits.
- Systems requiring per-function upgrade authority.
- Complex protocols with separable modules.

---

## 8. Beacon Proxy

**Location:** `examples/advanced/02-beacon-proxy/`, `examples/advanced/06-beacon-management/`

### What it does
Beacon pattern where multiple proxy contracts point to a single beacon contract that stores the implementation address. Updating the beacon upgrades all proxies atomically.

### Architecture Decisions
- **Beacon registry** maps implementation versions to deployment timestamps.
- **Rollback support** allows reverting to a previous version.
- **Versioned upgrades** track all implementation changes.

### When to use
- Many proxy instances needing simultaneous upgrade.
- Standardized contract factories (e.g., token factories).
- Systems requiring upgrade audit trail.

---

## 9. Role-Based Access Control

**Canonical Location:** `examples/advanced/03-rbac-modifiers/`  
**Also see:** `examples/intermediate/02-role-based-access-control/` (numeric hierarchy), `examples/advanced/05-hierarchical-access-control/` (permission inheritance), `examples/advanced/03-registry-access-controls/` (registry-specific)

### What it does
Assigns roles to addresses and restricts function access by role. Supports flexible symbol-based roles, composable guards (`only_role`, `any_role`), and role renunciation. The canonical pattern uses symbol-based roles for maximum flexibility.

### Architecture Decisions
- **Symbol-based roles** (not numeric hierarchy) allow custom role names without redeployment.
- **Composable role guards** (`only_role`, `any_role`) protect functions with clear, reusable checks.
- **Default admin role** can manage all other roles.
- **Role renouncement** allows addresses to self-remove.
- **Event-driven audit trail** emits events for every role change.

### When to use
- Multi-user systems with distinct permission levels.
- Admin + operator + user separation.
- Delegated authority patterns.
- Token contracts with minter/pauser/burner roles.

### Alternative patterns
- Use **02-role-based-access-control** for strict numeric hierarchies (Owner > Admin > Moderator > User).
- Use **05-hierarchical-access-control** for dynamic permission inheritance and role hierarchies.
- Use **03-registry-access-controls** for registry-specific whitelist/fee patterns.

---

## 10. Hierarchical Access Control

**Location:** `examples/advanced/16-hierarchical-access-control/`

### What it does
Extends RBAC with hierarchical organization units. Permissions propagate down the hierarchy: department heads inherit team-level access.

### Architecture Decisions
- **Tree-based storage** maps each node to its parent and children.
- **Inheritance resolution** walks the tree upward to check transitive permissions.
- **Separation of duties** enforced at each hierarchy level.

### When to use
- Organizational structures (companies, DAOs with departments).
- Multi-tenant systems.
- Fine-grained permission escalation paths.

---

## 11. Batch Operations

**Location:** `examples/advanced/39-batch-operations/`

### What it does
Executes multiple contract calls in a single transaction with configurable atomicity. Supports all-succeed-or-fail and partial-execution modes.

### Architecture Decisions
- **Atomic mode**: any failure reverts all prior operations.
- **Partial mode**: failures are recorded but successful operations persist.
- **Operation queue** stored as `Vec<Call>` for predictable iteration.

### When to use
- Bulk token transfers.
- Multi-step setup transactions.
- Gas optimization through call consolidation.

---

## 12. Merkle Proofs

**Location:** `examples/advanced/21-merkle-proofs/`

### What it does
Validates data membership in a Merkle tree using on-chain proof verification. Enables off-chain data storage with on-chain verification.

### Architecture Decisions
- **Keccak-256 hashing** for compatibility with off-chain proof generation.
- **Sorted leaf nodes** to prevent second-preimage attacks.
- **Configurable tree depth** to bound gas costs.

### When to use
- Airdrop eligibility verification.
- Off-chain storage of large datasets.
- Cross-chain state verification.

---

## 13. Reentrancy Guard

**Location:** `examples/advanced/15-reentrancy-guard/`

### What it does
Prevents reentrant calls by tracking execution state. Uses a mutex flag that blocks nested invocations.

### Architecture Decisions
- **Storage-based flag** persists across the call lifecycle.
- **Modifier pattern** wraps functions with `lock/unlock`.
- **No reentrancy** for external-facing state-changing functions.

### When to use
- Contracts accepting external calls during execution.
- Cross-contract transfer or callback patterns.
- Any contract holding user funds.

---

## 14. Decision Guide

### How to choose a pattern

```
Need multiple signatures?
â”œâ”€â”€ All signers must approve â†’ N-of-N Multi-Sig
â”œâ”€â”€ Threshold of signers needed â†’ M-of-N with Proposals
â””â”€â”€ Store signer list compactly â†’ Auth Vectors

Need delayed execution?
â”œâ”€â”€ Add time delay to actions â†’ Timelock
â”œâ”€â”€ Emergency stop capability â†’ Add Pause pattern

Need external data?
â”œâ”€â”€ Single trusted source â†’ Basic Oracle
â”œâ”€â”€ Multiple sources â†’ Aggregation Oracle

Need contract upgrades?
â”œâ”€â”€ Single implementation swap â†’ Upgradeable Proxy
â”œâ”€â”€ Many instances upgrade together â†’ Beacon Proxy
â”œâ”€â”€ Per-function upgrade granularity â†’ Diamond
â””â”€â”€ Track all versions â†’ Beacon Management

Need access control?
â”œâ”€â”€ Simple roles â†’ RBAC
â”œâ”€â”€ Organizational hierarchy â†’ Hierarchical Access Control

Need safety?
â”œâ”€â”€ Prevent reentrancy â†’ Reentrancy Guard
â”œâ”€â”€ Cross-chain security â†’ Bridge Security
â”œâ”€â”€ Batch safety â†’ Batch Operations
â”œâ”€â”€ Off-chain verification â†’ Merkle Proofs
â””â”€â”€ Cross-chain transfers â†’ Cross-Chain Bridge
```

# Governance Examples

On-chain governance patterns: voting systems, delegation, multisig control, and DAO treasury management.

> **Note on Numbering:** Each example has a **unique** numeric prefix that defines its position in the learning path. Numbers are not repeated in this directory.

## Pattern Progression (Recommended Order)

**Beginner → Production:**

1. **Start here** — [`01-simple-voting`](./01-simple-voting/) — Learn vote lifecycle and auth patterns
2. **Add governance token** — [`02-token-voting`](./02-token-voting/) — Balance-weighted voting with flash-loan resistance
3. **Formal phases** — [`03-voting-time-constraints`](./03-voting-time-constraints/) — Voting periods and quorum
4. **Complex workflows** — [`04-proposal-lifecycle`](./04-proposal-lifecycle/) — Full state machine
5. **Production security** — [`09-timelock-governance`](./09-timelock-governance/) + [`10-dao-treasury`](./10-dao-treasury/) — Multisig + timelock + treasury

## Quick Start

```bash
cd examples/governance/01-simple-voting
cargo test && cargo build --target wasm32-unknown-unknown --release
```

## Examples by Pattern

### Voting Fundamentals

| # | Example | Focus | Concepts |
|---|---------|-------|----------|
| 01 | [simple-voting](./01-simple-voting/) | 🟢 Beginner | Proposal creation, one-address-one-vote, time-based deadlines, tally, execution |
| 02 | [token-voting](./02-token-voting/) | 🟡 Intermediate | Balance snapshots, token-weighted voting, flash-loan resistance |
| 03 | [voting-time-constraints](./03-voting-time-constraints/) | 🟡 Intermediate | Voting periods, grace periods, quorum thresholds, early closure |

### Delegation & Authority

| # | Example | Focus | Concepts |
|---|---------|-------|----------|
| 05 | [proposal-validation](./05-proposal-validation/) | 🟡 Intermediate | Proposal creation gates, validator patterns, pre-voting checks |
| 06 | [vote-delegation](./06-vote-delegation/) | 🟡 Intermediate | Liquid delegation, chain traversal, cycle detection, recursion limits |
| 07 | [delegation](./07-delegation/) | 🟠 Advanced | Enhanced delegation with revocation, time-bounds, and re-delegation |
| 08 | [delegation-marketplace](./08-delegation-marketplace/) | 🟠 Advanced | Marketplace for listing/renting voting power with incentive mechanisms |

### Advanced Governance

| # | Example | Focus | Concepts |
|---|---------|-------|----------|
| 04 | [proposal-lifecycle](./04-proposal-lifecycle/) | 🟠 Advanced | Full state machine: Draft → Active → Queued → Executed/Defeated, veto paths |
| 09 | [timelock-governance](./09-timelock-governance/) | 🟠 Advanced | Mandatory delays, veto windows, emergency bypass, queue management |
| 10 | [dao-treasury](./10-dao-treasury/) | 🟠 Advanced | Multisig fund management, timelock on withdrawals, role-based access |
| 11 | [automatic-snapshot-triggers](./11-automatic-snapshot-triggers/) | 🟠 Advanced | Time-based & event-based snapshots, snapshot pruning, gas-efficient storage. **Cross-domain pattern:** [`defi/14`](../defi/14-automatic-snapshot-triggers/), [`tokens/10`](../tokens/10-automatic-snapshot-triggers/), [`nfts/05`](../nfts/05-automatic-snapshot-triggers/) |

## Documentation

See [`docs/governance-patterns.md`](../../docs/governance-patterns.md) for:
- Pattern explanations with code examples
- When to use each pattern
- Security considerations and threat models
- Voting system comparison table
- Deployment checklist

See [`docs/governance-rbac-multisig-timelock.md`](../../docs/governance-rbac-multisig-timelock.md) for:
- RBAC, multisig, and timelock foundations
- Combined governance flows
- Role hierarchy design
- Signer and threshold recommendations

## Next Steps

- Read [`docs/governance-patterns.md`](../../docs/governance-patterns.md) for pattern overview
- Start with `01-simple-voting` for basic voting flow
- Combine `02-token-voting` + `09-timelock-governance` for production DAO
- See [`examples/basics/03-authentication/`](../basics/03-authentication/) for RBAC foundation
- See [`examples/basics/04-events/`](../basics/04-events/) for event patterns used in governance contracts

# Governance Examples

DAOs, voting, treasury management.

> **Note on Numbering:** Governance examples use unique sequential prefixes (01–11). Each number appears exactly once, and the ordering reflects the recommended learning path.

## Core Learning Path

1. [`01-simple-voting`](../../examples/governance/01-simple-voting/) — Proposal creation, one-address-one-vote, time-based deadlines
2. [`02-token-voting`](../../examples/governance/02-token-voting/) — Balance snapshots, token-weighted voting, flash-loan resistance
3. [`03-voting-time-constraints`](../../examples/governance/03-voting-time-constraints/) — Voting periods, grace periods, quorum thresholds, early closure
4. [`04-proposal-lifecycle`](../../examples/governance/04-proposal-lifecycle/) — Full state machine: Draft → Active → Queued → Executed

## Delegation & Authority

- [`05-proposal-validation`](../../examples/governance/05-proposal-validation/) — Proposal creation gates and pre-voting checks
- [`06-vote-delegation`](../../examples/governance/06-vote-delegation/) — Liquid delegation with cycle detection and recursion limits
- [`07-delegation`](../../examples/governance/07-delegation/) — Enhanced delegation with revocation and time-bounds
- [`08-delegation-marketplace`](../../examples/governance/08-delegation-marketplace/) — Marketplace for renting voting power

## Advanced Governance

- [`09-timelock-governance`](../../examples/governance/09-timelock-governance/) — Mandatory delays, veto windows, emergency bypass
- [`10-dao-treasury`](../../examples/governance/10-dao-treasury/) — Multisig fund management with role-based access
- [`11-automatic-snapshot-triggers`](../../examples/governance/11-automatic-snapshot-triggers/) — Automated voting-power snapshots

## Playground Use Cases

Try governance flows interactively in the [Playground](../playground.md). The playground use-case examples mirror the contracts in this section, so you can experiment with proposal creation, voting, and delegation before deploying.

Run any governance example locally:

```bash
cd examples/governance/01-simple-voting && cargo test
```

Swap in `02-token-voting`, `06-vote-delegation`, or `09-timelock-governance` to explore token-weighted voting, delegation, and timelocks. See the [Playground use cases](../playground.md#use-cases) for the full set of DeFi, NFT, governance, and token examples.

## Prerequisites
- [Basics](../basics.md), [Tokens](../tokens.md)

## Next: [Tokens](../tokens.md)

# Intermediate Examples

These examples combine Soroban fundamentals into reusable contract patterns. Follow the order below when learning; after the first few examples, choose a branch that matches the problem you are solving.

## Prerequisites

Complete the [basic examples](../basics/) first, especially storage, authentication, events, errors, and collections. You should also have a working Rust and Soroban CLI setup from the [getting started guide](../../guides/getting-started.md) and be comfortable with the [testing guide](../../guides/testing.md).

## Curated Learning Path

Difficulty is relative to the basic examples: **Foundational** introduces the intermediate idea, **Intermediate** combines several patterns, and **Advanced** adds migration, caching, or cross-contract complexity.

| Order | Example | What it teaches | Difficulty | Prerequisites | Related docs |
| --- | --- | --- | --- | --- | --- |
| 1 | [Iterable mappings](./iterable-mappings/) | Enumerable maps with a maintained key index | Foundational | Storage, vectors, maps | [Storage types](../../docs/storage-types.md) |
| 2 | [Priority queue](./03-priority-queue/) | Heap-backed ordering and bounded collection operations | Foundational | Collections, validation | [Testing best practices](../../docs/testing-best-practices.md) |
| 3 | [Event subscriptions](./event-subscriptions/) | Subscriber registration and event-driven contract coordination | Foundational | Authentication, events | [Common patterns](../../docs/common-patterns.md) |
| 4 | [Event aggregation](./event-aggregation/) | Batching related actions into one event | Foundational | Events, collections | [Event filtering](../basics/14-event-filtering/) |
| 5 | [Event history](./event-history/) | Persistent audit history with filtering and pagination | Intermediate | Events, persistent storage | [Security best practices](../../docs/security-best-practices.md) |
| 6 | [Role-based access control](./02-role-based-access-control/) | Role hierarchies, grants, revocation, and protected actions | Intermediate | Authentication, events | [Authentication](../basics/03-authentication/) |
| 7 | [Access control guide](./access-control/) | Combined RBAC, multisig, and timelock with threat models | Intermediate | RBAC, multisig, timelock | [Governance & Auth Patterns](../../docs/governance-rbac-multisig-timelock.md) |
| 8 | [Pause and unpause](./03-pause-unpause/) | Emergency controls for sensitive contract operations | Intermediate | Authentication, RBAC | [Token pause permissions](../tokens/10-pausable-permissions/) |
| 8 | [Multi-sig patterns](./multi-sig-patterns/) | Threshold approvals and multi-party authorization | Intermediate | Authentication, RBAC | [Multi-sig reference](./multi-sig-patterns/QUICK_REFERENCE.md) |
| 9 | [Ajo](./ajo/) | A rotating savings group with member and contribution rules | Intermediate | Authentication, storage, events | [Token examples](../tokens/) |
| 10 | [Ajo factory](./ajo-factory/) | Registering templates and deploying contracts from a factory | Advanced | Cross-contract calls, Wasm deployment | [Factory pattern](./ajo-factory/README.md) |
| 11 | [Lazy loading](./lazy-loading/) | Bounded caching and deferred reads for large state sets | Advanced | Persistent storage, pagination | [Gas benchmarks](../../docs/gas-benchmarks.md) |
| 12 | [Storage pagination](./storage-pagination/) | Opaque cursors for stable, page-sized queries | Advanced | Persistent storage, collections | [Storage types](../../docs/storage-types.md) |
| 13 | [Storage migration](./storage-migration/) | Staged, batched schema upgrades with rollback-friendly state | Advanced | Persistent storage, authorization | [Deployment guide](../../guides/deployment.md) |

## Related Learning Tracks

- [SEP-41 token examples](../tokens/01-sep41-token/) — Start with the standard token interface before exploring wrappers, allowances, or pause permissions.
- [Token examples index](../tokens/) — Token-specific implementations and extensions.
- [Security best practices](../../docs/security-best-practices.md) — Threat modeling and defensive contract design.
- [Cross-contract patterns](../../docs/cross-contract-patterns.md) — Factory and contract-composition concepts.

## Building and Testing

```bash
# Build all workspace contracts
cargo build --target wasm32-unknown-unknown --release

# Run all workspace tests
cargo test
```

To focus on one example, run its package commands from the repository root:

```bash
cargo test -p ajo-factory
cargo build --target wasm32-unknown-unknown --release -p ajo-factory
```

## Next Steps

Once comfortable with these patterns, continue to the [advanced examples](../advanced/), or apply the [SEP-41 token track](../tokens/) before exploring [DeFi](../defi/) and [governance](../governance/).

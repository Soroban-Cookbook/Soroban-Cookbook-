# 7. Version and Migrate Persistent State Explicitly

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Replacing contract code does not transform existing storage values. A change
to key encoding, value shape, or state ownership can therefore make old state
unreadable or leave it stranded. Large migrations may also exceed transaction
resource limits if performed in one call.

## Decision

Treat persistent storage as a versioned schema. Preserve compatibility for
existing keys where practical; otherwise add an explicit migration with a
version marker, authorization, and observable progress. Make migration steps
idempotent and bounded so they can safely resume across transactions. Define
which version each entry uses and how normal entry points behave while
migration is incomplete.

When moving to a different contract address, do not assume the new contract
can read the old contract's storage. Specify and test an explicit export/import,
user-driven migration, or routing plan before deployment.

## Consequences

### Positive

- Upgrades have a defined compatibility and recovery path.
- Bounded, repeatable work is easier to test against transaction limits.

### Negative

- Migration logic increases code size and requires fixtures from prior
  versions.
- Some migrations require user participation or multiple transactions.

### Neutral

- A migration may be unnecessary when a code update preserves the existing
  storage schema; record that compatibility check in the upgrade review.

## References

- Example: `examples/advanced/04-upgradeable-proxy/`
- Example: `examples/advanced/03-proxy-admin/`
- Related: [ADR-003](./003-stable-storage-keys-and-ttl.md),
  [ADR-006](./006-guard-contract-upgrades.md)

## Notes

Test migration from representative old-state fixtures, including repeated
execution and partial completion.
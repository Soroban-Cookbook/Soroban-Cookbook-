# 3. Keep Storage Keys Stable and Manage TTL Explicitly

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Storage keys are part of a contract's persistent data schema. Ad hoc keys and
unplanned TTL renewal make collisions, accidental data loss, and future
upgrades harder to reason about.

## Decision

Represent keys with a typed `#[contracttype]` enum or struct, including the
address or identifier that scopes each record. Treat serialized key shapes as
stable: do not repurpose a key for a different value or silently change its
encoding. Introduce a new key/schema and an explicit migration when the shape
must change.

For persistent and instance entries, set TTL renewal thresholds and target
lifetimes intentionally. Renew before the entry reaches its archival boundary;
test both normal access and expiry/recovery behavior. Temporary entries must
remain safe to lose when their TTL expires.

## Consequences

### Positive

- Key ownership and data shape are easier to audit and test.
- Schema changes become explicit upgrade work instead of accidental breakage.

### Negative

- Typed keys and migration paths add some implementation overhead.
- TTL policy needs maintenance as protocol limits and retention needs evolve.

### Neutral

- Existing key formats remain usable; this decision governs new and changed
  storage schemas.

## References

- Soroban storage guidance: <https://developers.stellar.org/docs/smart-contracts/data/storage-types>
- Example: `examples/basics/02-storage-patterns/`
- Related: [ADR-002](./002-select-storage-tier.md)

## Notes

Include storage key compatibility and TTL renewal in upgrade reviews.
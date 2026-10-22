# 2. Select Storage Tier by Lifetime

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Soroban offers persistent, instance, and temporary storage with different
lifetime, TTL, and rent behavior. A single default tier either increases cost
or risks data expiring sooner than the application expects.

## Decision

Choose the tier from the data's required lifetime and access pattern:

- Use temporary storage only for short-lived, reconstructable data; account
  for its expiration.
- Use instance storage for small, contract-wide configuration and state whose
  lifetime follows the contract instance.
- Use persistent storage for durable, independently keyed application state;
  manage each entry's TTL explicitly.

Do not describe persistent data as permanent: every tier has lifecycle and
archival constraints that the contract must handle.

## Consequences

### Positive

- Storage cost and expiration behavior match the purpose of each value.
- TTL renewal and recovery requirements are visible during design.

### Negative

- Code must select a tier deliberately and handle expiry where applicable.
- Moving data between tiers later may require a migration.

### Neutral

- The storage tier is an application-level choice, not a substitute for access
  control or backups.

## References

- Soroban storage guidance: <https://developers.stellar.org/docs/smart-contracts/data/storage-types>
- Example: `examples/basics/02-storage-patterns/`

## Notes

Review tier choices when retention requirements or protocol limits change.
# 5. Use Least-Privilege Access Control

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Some contracts need only one administrator; others have distinct capabilities
such as minting, pausing, and configuration. A single broad admin role is
simple but grants more authority than many operations require. Fine-grained
roles add state and administration complexity.

## Decision

Use the simplest model that represents the actual trust boundaries:

- Use a stored owner/admin for a small contract with one coherent privileged
  capability.
- Use named, least-privilege roles when capabilities are independently
  granted, revoked, or exercised.
- Authenticate the actor and check the stored owner or role membership on each
  privileged entry point. Protect role grants and revocations with an explicit
  role-administration policy.
- Use multisig or governance authority for high-impact administration where
  operational risk warrants it; add a timelock for changes that need a review
  window.

Do not infer a role hierarchy from enum ordering or naming; encode the
permitted capabilities explicitly.

## Consequences

### Positive

- Routine operators receive only the permissions they need.
- Privileged actions and role changes have clear authorization boundaries.

### Negative

- Role-based systems require careful grant, revoke, and recovery procedures.
- Multisig and timelocks add operational latency and integration work.

### Neutral

- Permissionless entry points remain valid when their behavior is safe for any
  caller.

## References

- Soroban authorization: <https://developers.stellar.org/docs/learn/fundamentals/authorization>
- Examples: `examples/basics/03-authentication/`, `examples/advanced/03-rbac-modifiers/`
- Related: [ADR-004](./004-authenticate-before-state-changes.md)

## Notes

Review administrator recovery and key rotation alongside role design.
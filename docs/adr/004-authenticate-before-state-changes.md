# 4. Authenticate Before Protected State Changes

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Contract entry points often receive an address identifying the actor or owner
of an asset. Checking application permissions alone does not prove that the
address authorized the invocation, and omitting that check can expose state to
unauthorized changes.

## Decision

Call `Address::require_auth()` for the address that must authorize an action
before performing protected state mutations. Then apply separate business
authorization checks, such as ownership, role membership, allowance, or
state-transition rules. A caller-supplied address is not proof of permission
by itself.

For delegated actions, record the grant explicitly and authenticate the
delegate when it exercises that grant. Keep authorization scope limited to
the operation and data the grant is intended to cover.

## Consequences

### Positive

- Authentication and application-level authorization remain distinct and
  auditable.
- Unauthorized state changes fail before contract state is modified.

### Negative

- Clients must provide the required authorization entries.
- Each entry point must identify whose authorization is required.

### Neutral

- Read-only operations may remain public when the data and application policy
  permit it.

## References

- Soroban authorization: <https://developers.stellar.org/docs/learn/fundamentals/authorization>
- Example: `examples/basics/03-authentication/`

## Notes

Test missing authorization, wrong actor, and delegated authorization paths.
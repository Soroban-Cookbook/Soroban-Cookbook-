# 6. Guard Contract Code Upgrades

**Date:** 2026-10-21

## Status

**Status:** Proposed

## Context

Soroban contract code is not replaced merely by deploying a new Wasm artifact.
An in-place update requires an explicit contract upgrade path; systems without
that path need a new contract or explicit routing architecture. An unsafe
upgrade can change critical behavior without giving users time to review it.

## Decision

When a contract supports in-place Wasm replacement, require authenticated
governance or admin approval and use a proposal workflow with a review delay,
cancellation path, and events for proposal and activation. Verify and record
the Wasm hash before activation. Use a multisig or governance address for
production authority when appropriate.

If a contract has no safe in-place upgrade path, deploy a versioned replacement
or an explicit router/adapter and document how clients move to it. Do not
assume EVM-style `delegatecall` or automatic sharing of storage between
contracts.

## Consequences

### Positive

- Upgrade authority, code identity, and review windows are explicit.
- Users and integrators can observe proposed and activated changes.

### Negative

- Timelocks can delay urgent fixes; emergency procedures need separate review.
- New-contract routing requires compatibility and client migration work.

### Neutral

- A pause mechanism can limit incident impact but does not replace a safe
  upgrade process.

## References

- Example: `examples/advanced/03-proxy-admin/`
- Example: `examples/advanced/04-upgradeable-proxy/`
- Soroban upgrade guidance: <https://developers.stellar.org/docs/smart-contracts/contract-upgrades>

## Notes

Test authorization, cancellation, delay enforcement, replay prevention, and
the exact Wasm hash activated.
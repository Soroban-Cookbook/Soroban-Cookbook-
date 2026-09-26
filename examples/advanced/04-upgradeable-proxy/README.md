# Upgradeable Proxy Pattern

A proxy pattern for contract upgrades that separates the proxy and implementation contracts. The proxy owns application state, so replacing the implementation preserves that state.

## Scope In the Upgradeability Sequence

This is step 1 of 6 and the starting point for the
[upgradeability examples](../README.md). It demonstrates a single proxy whose
implementation address can be changed; `implementation-v1` supplies the initial
business logic.

- **In scope:** direct implementation routing and proxy-owned state across upgrades.
- **Out of scope:** timelocked upgrade proposals, shared beacons, and storage
    schema migration. Continue to [Proxy Admin Controls](../03-proxy-admin/) for
    upgrade governance.

## What It Demonstrates

- **Proxy Contract**: Forwards calls to an implementation contract
- **Implementation Contract**: Contains the actual business logic
- **Safe Upgrades**: Seamless migration from one implementation to another
- **Storage Preservation**: Proxy-owned state remains consistent across upgrades
- **Flexible Upgrade Flow**: Admin can set a new implementation address

## Use Cases

- Contract upgrades without redeploying
- Fixing bugs and adding features without losing state
- Testing new implementations alongside existing ones
- Gradual rollout of new contract versions

## Architecture

```
┌─────────────────┐
│ Proxy Contract  │
│                 │
│ - Storage       │
│ - Forwards to   │
│   Implementation│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Implementation  │
│ Contract (v1)   │
│                 │
│ - Business Logic│
└─────────────────┘
```

When upgrading to v2:
1. Deploy new implementation contract
2. Proxy calls `set_implementation(new_address)`
3. All subsequent calls forward to v2
4. Storage is preserved because the counter belongs to the proxy

## Key Concepts

- **Storage Preservation**: The proxy owns the counter; implementations provide behavior
- **Admin Control**: Only the proxy admin can authorize upgrades
- **No Storage Migration**: Because both contracts access the same storage, no migration is needed
- **Clean Interface**: Proxy provides a stable entry point while implementation can be replaced

## Test Flow

The tests deploy v1 and v2 in one environment, increment the proxy-owned counter
before upgrading, then verify that v2 adds `multiply`, changes increment behavior,
and still sees the old counter value. Upgrade authorization and one-time
initialization are also covered.

Run with:

```bash
cargo test -p upgradeable-proxy
```

## Next

Continue with [Proxy Admin Controls](../03-proxy-admin/) to add proposal delays,
cancellation, and emergency pause controls around upgrade operations.

# Upgradeable Proxy Pattern

A simple proxy pattern for contract upgrades that separates the proxy and implementation contracts, preserving storage across upgrades.

## What It Demonstrates

- **Proxy Contract**: Forwards calls to an implementation contract
- **Implementation Contract**: Contains the actual business logic
- **Safe Upgrades**: Seamless migration from one implementation to another
- **Storage Preservation**: Shared storage remains consistent across upgrades
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
4. Storage is preserved (shared between proxy and impl)

## Key Concepts

- **Storage Preservation**: Both proxy and implementation share the same storage context
- **Admin Control**: Only the proxy admin can authorize upgrades
- **No Storage Migration**: Because both contracts access the same storage, no migration is needed
- **Clean Interface**: Proxy provides a stable entry point while implementation can be replaced

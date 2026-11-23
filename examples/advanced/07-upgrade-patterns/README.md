# Upgrade Patterns

Direct WASM upgrade, versioned storage migration, and init guards. Highest-level patterns for safe contract evolution.

## Scope In the Upgradeability Sequence

This is step 6 of 6, following [Beacon Management](../06-beacon-management/).

- **In scope:** direct WASM replacement, versioned storage migration, and
    guarded post-upgrade initialization.
- **Out of scope:** proxy routing, shared-beacon deployment, and a complete
    governance system. For the sequence's starting point, see
    [Upgradeable Proxy](../04-upgradeable-proxy/); for timelocked admin controls,
    see [Proxy Admin Controls](../31-proxy-admin/).

## Role in Learning Path

This is the **final step** in the [upgrade patterns learning path](../README.md#upgrade-patterns--proxy-patterns). After mastering proxy patterns, this example shows:
- Direct WASM contract upgrades
- Versioned storage migrations (v1 to v2)
- Storage compatibility checks

## What It Demonstrates
- Init guard patterns to prevent re-initialization
- Safe state evolution across versions

**Prerequisites:** Understand all proxy and versioning patterns:
- [`02-beacon-proxy`](../02-beacon-proxy/) — Basic beacon concept
- [`23-beacon-proxy-factory`](../23-beacon-proxy-factory/) — Factory patterns
- [`31-proxy-admin`](../31-proxy-admin/) — Governance patterns
- [`04-upgradeable-proxy`](../04-upgradeable-proxy/) — Direct upgrade alternative
- [`06-beacon-management`](../06-beacon-management/) — Versioned implementations

## Key Concepts

- Direct WASM code replacement
- Batched storage migrations
- Version number tracking
- Dual-read patterns for backwards compatibility
- Init guard enforcement
- Safe state schema evolution

## Pattern Progression

**Basic beacon → Beacon factory → Governance → Direct upgrade → Versioning → Full patterns**

## When to Use

- Large-scale deployments where all contracts upgrade atomically
- State migrations that require coordinated changes
- Removing migration code after all instances are upgraded
- Complex storage restructuring

See the [advanced examples README](../README.md) for the full upgrade patterns learning path.

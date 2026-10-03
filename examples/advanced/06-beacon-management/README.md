# Beacon Management

Versioned beacon management with rollback support. Manage multiple implementation versions and roll back if needed.

## Scope In the Upgradeability Sequence

This is step 5 of 6, following the
[beacon proxy factory](../23-beacon-proxy-factory/).

- **In scope:** registering multiple named beacons, tracking each beacon's
	implementation history, and rolling back its latest upgrade.
- **Out of scope:** deploying proxy fleets or forwarding application calls.
	Continue to [Upgrade Patterns](../07-upgrade-patterns/) for direct WASM
	upgrades and storage migration.

## Role in Learning Path

This is the **fifth step** in the [upgrade patterns learning path](../README.md#upgrade-patterns--proxy-patterns). After learning basic and governed proxies, this example adds:
- Version tracking for implementations
- Rollback capabilities to previous versions
- Version history maintenance
- Safe rollback verification
- Production recovery mechanisms

**Prerequisites:** Understand both beacon and direct upgrade patterns:
- [`02-beacon-proxy`](../02-beacon-proxy/) — Basic beacon concept
- [`23-beacon-proxy-factory`](../23-beacon-proxy-factory/) — Factory patterns
- [`31-proxy-admin`](../31-proxy-admin/) — Governance patterns
- [`04-upgradeable-proxy`](../04-upgradeable-proxy/) — Direct upgrade alternative

**Next step:**
- **[`07-upgrade-patterns`](../07-upgrade-patterns/)** — Complete upgrade strategies

## Key Concepts

```bash
cargo test -p beacon-management
```

## Next

Continue with [Upgrade Patterns](../07-upgrade-patterns/) for direct WASM
replacement, versioned storage migration, and post-upgrade initialization.

- Version tracking and history
- Implementation rollback mechanisms
- Previous version storage and retrieval
- Safe downgrade paths
- Emergency recovery procedures

## Pattern Progression

**Basic beacon → Beacon factory → Governance → Direct upgrade → Versioning → Full patterns**

See the [advanced examples README](../README.md) for the full upgrade patterns learning path.

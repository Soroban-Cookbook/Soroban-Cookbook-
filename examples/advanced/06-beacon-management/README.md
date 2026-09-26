# Beacon Management

A Soroban cookbook example for managing upgradeable beacon contracts with per-beacon version history, rollback support, and multi-beacon registration.

## Scope In the Upgradeability Sequence

This is step 5 of 6, following the
[beacon proxy factory](../03-beacon-proxy-factory/).

- **In scope:** registering multiple named beacons, tracking each beacon's
	implementation history, and rolling back its latest upgrade.
- **Out of scope:** deploying proxy fleets or forwarding application calls.
	Continue to [Upgrade Patterns](../07-upgrade-patterns/) for direct WASM
	upgrades and storage migration.

## What it demonstrates

- Versioned beacon implementations
- Rollback to the previous implementation
- Independent state for multiple beacons
- Admin-authenticated lifecycle operations

## Contract API

- `initialize(admin)` — bootstraps the beacon manager
- `register_beacon(admin, name, implementation)` — creates a new beacon with version 1
- `upgrade_beacon(admin, name, implementation)` — appends a new version and activates it
- `rollback_beacon(admin, name)` — reverts the beacon to the immediately previous version
- `get_beacon(name)` — returns the current beacon state and history
- `list_beacons()` — returns all registered beacon names

## Run tests

```bash
cargo test -p beacon-management
```

## Next

Continue with [Upgrade Patterns](../07-upgrade-patterns/) for direct WASM
replacement, versioned storage migration, and post-upgrade initialization.

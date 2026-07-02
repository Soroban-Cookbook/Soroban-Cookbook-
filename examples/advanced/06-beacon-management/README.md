# Beacon Management

A Soroban cookbook example for managing upgradeable beacon contracts with per-beacon version history, rollback support, and multi-beacon registration.

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

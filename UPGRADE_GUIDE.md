# Upgrade Guide

## Preserve initialization guards across WASM upgrades

Initialization guards are part of a contract's persistent storage schema. Every
WASM version must check the same canonical key before accepting initialization.
For the upgradeable proxy example, that key is the persistent storage symbol
`initialized`. Keep both the storage lifetime and the exact key string unchanged
in every implementation, even when storage enums or other key definitions are
refactored. A new WASM that checks a different key can treat an already
initialized contract as fresh and allow an attacker to claim its administrator
role.

Before publishing an upgrade, confirm that its initializer still checks the
original guard key and that the existing key remains present in the new version.

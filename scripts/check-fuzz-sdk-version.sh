#!/usr/bin/env bash
# check-fuzz-sdk-version.sh
#
# CI check for issue #1485: verify that tests/fuzz/Cargo.toml does NOT pin its
# own soroban-sdk version and instead delegates to the workspace root via
# `workspace = true`.
#
# Exit 0 — versions are in sync (fuzz crate uses workspace = true).
# Exit 1 — fuzz crate has a pinned or divergent soroban-sdk declaration.

set -euo pipefail

FUZZ_TOML="tests/fuzz/Cargo.toml"

if [ ! -f "$FUZZ_TOML" ]; then
  echo "ERROR: $FUZZ_TOML not found. Run this script from the workspace root." >&2
  exit 1
fi

# Look for any soroban-sdk line that does NOT use workspace = true.
DIVERGENT=$(grep -n 'soroban-sdk' "$FUZZ_TOML" | grep -v 'workspace\s*=\s*true' || true)

if [ -n "$DIVERGENT" ]; then
  echo "ERROR: $FUZZ_TOML has a soroban-sdk declaration that does not use" >&2
  echo "       'workspace = true'. This risks testing a different SDK version" >&2
  echo "       than the rest of the cookbook (issue #1485)." >&2
  echo "" >&2
  echo "Offending line(s):" >&2
  echo "$DIVERGENT" >&2
  echo "" >&2
  echo "Fix: change the dependency to:" >&2
  echo "  soroban-sdk = { workspace = true, features = [\"testutils\"] }" >&2
  exit 1
fi

# Also cross-check that the workspace root actually declares soroban-sdk.
ROOT_TOML="Cargo.toml"
if ! grep -q 'soroban-sdk' "$ROOT_TOML"; then
  echo "ERROR: $ROOT_TOML does not declare soroban-sdk in [workspace.dependencies]." >&2
  exit 1
fi

echo "OK: tests/fuzz/Cargo.toml soroban-sdk version is in sync with workspace root."
exit 0

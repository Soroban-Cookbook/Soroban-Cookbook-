#!/usr/bin/env bash
# Run a cargo command against the workspace, excluding crates pending SDK 26 migration.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXCLUDE_FILE="${ROOT}/scripts/ci-exclude-packages.txt"

EXCLUDE_ARGS=()
while IFS= read -r pkg || [[ -n "${pkg:-}" ]]; do
  pkg="${pkg%%#*}"
  pkg="$(echo "$pkg" | xargs)"
  [[ -z "$pkg" ]] && continue
  EXCLUDE_ARGS+=(--exclude "$pkg")
done < "$EXCLUDE_FILE"

cd "$ROOT"
SUBCMD=$1
shift
if [ "$SUBCMD" = "clippy" ]; then
  RUSTFLAGS="${RUSTFLAGS:-} -A deprecated" cargo "$SUBCMD" --workspace "${EXCLUDE_ARGS[@]}" "$@"
elif [ "$SUBCMD" = "build" ]; then
  # Host-only test crates, proptest, and the fuzz harness cannot compile for
  # wasm32v1-none.
  cargo "$SUBCMD" --workspace "${EXCLUDE_ARGS[@]}" \
    --exclude integration-tests --exclude security-tests \
    --exclude soroban-cookbook-fuzz "$@"
else
  cargo "$SUBCMD" --workspace "${EXCLUDE_ARGS[@]}" "$@"
fi

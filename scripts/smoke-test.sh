#!/usr/bin/env bash
# Run the smoke test suite and fail if execution exceeds the time budget.
#
# Usage: ./scripts/smoke-test.sh [budget_seconds]   (default: 120)
#
# Compilation is done first and is not counted against the budget; the
# budget applies to test execution only, matching the CI smoke job.

set -euo pipefail

BUDGET="${1:-120}"

echo "==> Building smoke tests"
cargo test -p integration-tests --test smoke_tests --no-run

echo "==> Running smoke tests (budget: ${BUDGET}s)"
START=$(date +%s)
cargo test -p integration-tests --test smoke_tests
END=$(date +%s)
ELAPSED=$((END - START))

echo "==> Smoke tests finished in ${ELAPSED}s"
if [ "$ELAPSED" -gt "$BUDGET" ]; then
  echo "error: smoke tests took ${ELAPSED}s, over the ${BUDGET}s budget" >&2
  exit 1
fi

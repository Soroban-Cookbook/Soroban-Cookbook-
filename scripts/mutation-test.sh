#!/usr/bin/env bash
# Mutation testing for security-critical examples via cargo-mutants.
#
# Tests for the example crates live in the `integration-tests` and
# `security-tests` workspace members, so we pass them via --test-package.
#
# Usage:
#   ./scripts/mutation-test.sh                 # run the default package set
#   ./scripts/mutation-test.sh pkg1 pkg2 ...   # run specific packages
#
# Environment:
#   MIN_MUTATION_SCORE  Minimum acceptable score (0.0-1.0). Unset = report only.
#   MUTANTS_JOBS         Parallel cargo-mutants jobs (default: nproc).
#   MUTANTS_OUTPUT       Output root (default: ./mutants.out).
#
# Requires: cargo, python3 (for parsing outcomes.json).

set -euo pipefail

# Security-critical packages: DeFi, tokens, governance.
DEFAULT_PACKAGES=(
    "constant-product-amm"
    "lending-pool"
    "staking-pool"
    "simple-voting"
    "sep41-token"
)

if [ "$#" -gt 0 ]; then
    PACKAGES=("$@")
else
    PACKAGES=("${DEFAULT_PACKAGES[@]}")
fi

JOBS="${MUTANTS_JOBS:-$(nproc 2>/dev/null || echo 4)}"
OUTPUT_ROOT="${MUTANTS_OUTPUT:-mutants.out}"
TIMEOUT_SECS="${MUTANTS_TIMEOUT:-300}"

# Both the mutated package's own tests and the shared integration/security
# suites exercise these contracts.
TEST_ARGS=(
    --test-package integration-tests
    --test-package security-tests
)

total_caught=0
total_missed=0
total_timeout=0
total_unviable=0

for pkg in "${PACKAGES[@]}"; do
    echo "======================================================================"
    echo "Mutation testing: ${pkg}"
    echo "======================================================================"

    out_dir="${OUTPUT_ROOT}/${pkg}"
    log_file="${out_dir}.log"

    set +e
    cargo mutants \
        --package "${pkg}" \
        --test-package "${pkg}" \
        "${TEST_ARGS[@]}" \
        --timeout "${TIMEOUT_SECS}" \
        --jobs "${JOBS}" \
        --output "${out_dir}" \
        2>&1 | tee "${log_file}"
    status=${PIPESTATUS[0]}
    set -e

    # cargo-mutants exit codes: 0=all caught, 2=missed mutants, 3=timeouts.
    # Anything else (usage error, baseline failure, internal error) aborts.
    if [ "${status}" -ne 0 ] && [ "${status}" -ne 2 ] && [ "${status}" -ne 3 ]; then
        echo "cargo-mutants failed for ${pkg} (exit ${status}); see ${log_file}" >&2
        exit "${status}"
    fi

    counts=$(python3 - "${out_dir}/outcomes.json" <<'PY'
import json, sys
try:
    with open(sys.argv[1]) as f:
        data = json.load(f)
except (OSError, json.JSONDecodeError):
    print("0 0 0 0")
    sys.exit(0)
print(
    data.get("caught", 0),
    data.get("missed", 0),
    data.get("timeout", 0),
    data.get("unviable", 0),
)
PY
)
    read -r c m t u <<< "${counts}"
    total_caught=$((total_caught + c))
    total_missed=$((total_missed + m))
    total_timeout=$((total_timeout + t))
    total_unviable=$((total_unviable + u))

    tested=$((c + m + t + u))
    denom=$((c + m + t))
    if [ "${denom}" -gt 0 ]; then
        score=$(python3 -c "print(f'{${c} / ${denom}:.4f}')")
    else
        score="n/a"
    fi
    echo ">> ${pkg}: caught=${c} missed=${m} timeout=${t} unviable=${u} score=${score}"

    # List missed mutants so they can be turned into tracking issues.
    if [ "${m}" -gt 0 ]; then
        python3 - "${out_dir}/outcomes.json" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    data = json.load(f)
for entry in data.get("outcomes", []):
    if "missed" in str(entry.get("summary", "")).lower():
        mutant = entry.get("scenario", {}).get("Mutant", {})
        print("   MISSED:", mutant.get("name", mutant) if isinstance(mutant, dict) else mutant,
              "|", entry.get("diff_path", ""))
PY
    fi
done

# ---------------------------------------------------------------------------
# Summary + optional threshold check
# ---------------------------------------------------------------------------

denominator=$((total_caught + total_missed + total_timeout))
if [ "${denominator}" -gt 0 ]; then
    overall=$(python3 -c "print(f'{${total_caught} / ${denominator}:.4f}')")
else
    overall="n/a"
fi

echo "======================================================================"
echo "Mutation testing summary"
echo "  caught:    ${total_caught}"
echo "  missed:    ${total_missed}"
echo "  timeout:   ${total_timeout}"
echo "  unviable:  ${total_unviable}"
echo "  score:     ${overall}"
echo "======================================================================"

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    {
        echo "## Mutation Testing"
        echo ""
        echo "| caught | missed | timeout | unviable | score |"
        echo "|---:|---:|---:|---:|---:|"
        echo "| ${total_caught} | ${total_missed} | ${total_timeout} | ${total_unviable} | ${overall} |"
        echo ""
        echo "Per-package results are in \`${OUTPUT_ROOT}/<package>/outcomes.json\`."
        echo "Missed mutants should be addressed with new tests — see CONTRIBUTING.md."
    } >> "${GITHUB_STEP_SUMMARY}"
fi

if [ -n "${MIN_MUTATION_SCORE:-}" ] && [ "${denominator}" -gt 0 ]; then
    python3 - "${overall}" "${MIN_MUTATION_SCORE}" <<'PY'
import sys
score = float(sys.argv[1])
threshold = float(sys.argv[2])
if score < threshold:
    print(f"::error::Mutation score {score:.2%} is below the required {threshold:.0%}")
    sys.exit(1)
print(f"Mutation score {score:.2%} meets threshold {threshold:.0%}")
PY
fi

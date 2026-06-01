#!/bin/bash

# Performance Benchmarking Script for Soroban Contracts
# Usage: ./scripts/benchmark.sh [example-path] [--output-dir <dir>]
# Example: ./scripts/benchmark.sh --output-dir gas-benchmark-results
# Example: ./scripts/benchmark.sh examples/intermediate/multi-sig-patterns --output-dir gas-benchmark-results

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_bench() {
    echo -e "${BLUE}[BENCH]${NC} $1"
}

show_help() {
    cat <<'EOF'
Usage: ./scripts/benchmark.sh [example-path] [--output-dir <dir>]

If no example path is provided, the script benchmarks all example directories
that include a benchmark test.

Options:
  -o, --output-dir <dir>   Write benchmark logs to the given directory.
  -h, --help               Show this help message.
EOF
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install from https://rustup.rs/"
    exit 1
fi

has_benchmark_tests() {
    local contract_path=$1
    if [ -f "$contract_path/src/test.rs" ] && grep -E -q 'benchmark|env.budget\(\)\.print' "$contract_path/src/test.rs"; then
        return 0
    fi
    return 1
}

benchmark_contract() {
    local contract_path=$1
    
    if [ ! -d "$contract_path" ]; then
        print_error "Directory not found: $contract_path"
        return 1
    fi
    
    if [ ! -f "$contract_path/Cargo.toml" ]; then
        print_error "No Cargo.toml found in $contract_path"
        return 1
    fi
    
    if ! has_benchmark_tests "$contract_path"; then
        print_warn "Skipping $contract_path: no benchmark tests found"
        return 0
    fi

    print_bench "Benchmarking contract: $contract_path"
    
    cd "$contract_path"
    local log_file=""
    if [ -n "$OUTPUT_DIR" ]; then
        mkdir -p "$OUTPUT_DIR"
        log_file="$OUTPUT_DIR/$(basename "$contract_path").bench.txt"
        cargo test -- --nocapture benchmark 2>&1 | tee "$log_file"
    else
        cargo test -- --nocapture benchmark
    fi
    local result=$?
    
    cd - > /dev/null
    
    if [ $result -eq 0 ]; then
        print_info "✓ Benchmarking completed"
        return 0
    else
        print_warn "! Benchmarking failed for $contract_path"
        return $result
    fi
}

OUTPUT_DIR=""
TARGETS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        -o|--output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            TARGETS+=("$1")
            shift
            ;;
    esac
done

if [ ${#TARGETS[@]} -eq 0 ]; then
    print_info "No path provided, benchmarking all example contracts with benchmark tests..."
    while IFS= read -r dir; do
        TARGETS+=("$dir")
    done < <(find examples -mindepth 2 -maxdepth 2 -type f -name Cargo.toml -printf '%h\n' | sort -u)
fi

if [ ${#TARGETS[@]} -eq 0 ]; then
    print_error "No example directories found."
    exit 1
fi

failures=0
for dir in "${TARGETS[@]}"; do
    if ! benchmark_contract "$dir"; then
        failures=$((failures + 1))
    fi
done

if [ $failures -ne 0 ]; then
    print_error "Benchmarking completed with $failures error(s)."
    exit 1
fi

print_info "All benchmarks completed successfully."

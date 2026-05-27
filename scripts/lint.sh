#!/bin/bash

# Lint Script for Soroban Contracts
# Usage: ./scripts/lint.sh [OPTIONS] [example-path]
#
# Options:
#   -f, --fmt-only    Run only cargo fmt check (skip clippy)
#   -c, --clippy-only Run only cargo clippy (skip fmt check)
#   -h, --help        Show this help message
#
# Examples:
#   ./scripts/lint.sh                                    # Lint entire workspace
#   ./scripts/lint.sh examples/basics/01-hello-world     # Lint specific example
#   ./scripts/lint.sh -f                                 # Format check only
#   ./scripts/lint.sh -c examples/basics                 # Clippy only on basics

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

print_lint() {
    echo -e "${BLUE}[LINT]${NC} $1"
}

show_help() {
    echo "Lint Script for Soroban Contracts"
    echo ""
    echo "Usage: $0 [OPTIONS] [example-path]"
    echo ""
    echo "Options:"
    echo "  -f, --fmt-only    Run only cargo fmt check (skip clippy)"
    echo "  -c, --clippy-only Run only cargo clippy (skip fmt check)"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Lint entire workspace"
    echo "  $0 examples/basics/01-hello-world     # Lint specific example"
    echo "  $0 -f                                 # Format check only"
    echo "  $0 -c examples/basics                 # Clippy only on basics"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()

    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo (Rust)")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            print_error "  - $dep"
        done
        print_error "Please install missing dependencies and try again."
        exit 1
    fi
}

# Run cargo fmt --check on a path
run_fmt_check() {
    local target=${1:-"."}

    print_lint "Checking formatting: $target"

    if [ "$target" = "." ]; then
        if cargo fmt --all --check; then
            print_info "✓ Formatting check passed"
            return 0
        else
            print_error "✗ Formatting issues found. Run 'cargo fmt --all' to fix."
            return 1
        fi
    else
        cd "$target"
        if cargo fmt --check; then
            print_info "✓ Formatting check passed for $target"
            cd - > /dev/null
            return 0
        else
            print_error "✗ Formatting issues found in $target. Run 'cargo fmt' to fix."
            cd - > /dev/null
            return 1
        fi
    fi
}

# Run cargo clippy on a path
run_clippy() {
    local target=${1:-"."}

    print_lint "Running clippy: $target"

    if [ "$target" = "." ]; then
        if cargo clippy --all-targets --all-features -- -D warnings; then
            print_info "✓ Clippy passed"
            return 0
        else
            print_error "✗ Clippy found issues"
            return 1
        fi
    else
        cd "$target"
        if cargo clippy --all-targets --all-features -- -D warnings; then
            print_info "✓ Clippy passed for $target"
            cd - > /dev/null
            return 0
        else
            print_error "✗ Clippy found issues in $target"
            cd - > /dev/null
            return 1
        fi
    fi
}

# Parse arguments
FMT_ONLY=false
CLIPPY_ONLY=false
TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--fmt-only)
            FMT_ONLY=true
            shift
            ;;
        -c|--clippy-only)
            CLIPPY_ONLY=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        -*)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            TARGET=$1
            shift
            ;;
    esac
done

# Validate conflicting flags
if [ "$FMT_ONLY" = true ] && [ "$CLIPPY_ONLY" = true ]; then
    print_error "--fmt-only and --clippy-only cannot be used together"
    exit 1
fi

check_dependencies

target=${TARGET:-"."}
failed=0

if [ "$CLIPPY_ONLY" = false ]; then
    if ! run_fmt_check "$target"; then
        failed=$((failed + 1))
    fi
fi

if [ "$FMT_ONLY" = false ]; then
    if ! run_clippy "$target"; then
        failed=$((failed + 1))
    fi
fi

echo "================================"
if [ $failed -eq 0 ]; then
    print_info "All lint checks passed! ✓"
    exit 0
else
    print_error "$failed lint check(s) failed."
    exit 1
fi

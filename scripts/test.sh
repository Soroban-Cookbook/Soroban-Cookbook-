#!/bin/bash

# Test Script for Soroban Contracts
# Usage: ./scripts/test.sh [example-path]
# Example: ./scripts/test.sh examples/basics/01-hello-world

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

print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Function to test a single contract
test_contract() {
    local contract_path=$1
    local verbose=${2:-false}
    
    if [ ! -d "$contract_path" ]; then
        print_error "Directory not found: $contract_path"
        return 1
    fi
    
    if [ ! -f "$contract_path/Cargo.toml" ]; then
        print_error "No Cargo.toml found in $contract_path"
        return 1
    fi
    
    print_test "Testing contract: $contract_path"
    
    cd "$contract_path"
    
    # Run tests
    if [ "$verbose" = true ]; then
        cargo test -- --nocapture
    else
        cargo test --quiet
    fi
    
    local result=$?
    
    cd - > /dev/null
    
    if [ $result -eq 0 ]; then
        print_info "✓ All tests passed"
        return 0
    else
        print_error "✗ Tests failed"
        return 1
    fi
}

# Function to run clippy
run_clippy() {
    local contract_path=$1
    
    print_test "Running clippy on: $contract_path"
    
    cd "$contract_path"
    
    if cargo clippy --quiet -- -D warnings 2>&1; then
        print_info "✓ Clippy passed"
        cd - > /dev/null
        return 0
    else
        print_error "✗ Clippy found issues"
        cd - > /dev/null
        return 1
    fi
}

# Function to check formatting
check_format() {
    local contract_path=$1
    
    print_test "Checking format: $contract_path"
    
    cd "$contract_path"
    
    if cargo fmt --check 2>&1; then
        print_info "✓ Format check passed"
        cd - > /dev/null
        return 0
    else
        print_error "✗ Format check failed. Run 'cargo fmt' to fix."
        cd - > /dev/null
        return 1
    fi
}

# Parse arguments
VERBOSE=false
RUN_CLIPPY=false
CHECK_FORMAT=false
CONTRACT_PATH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--clippy)
            RUN_CLIPPY=true
            shift
            ;;
        -f|--format)
            CHECK_FORMAT=true
            shift
            ;;
        -a|--all)
            RUN_CLIPPY=true
            CHECK_FORMAT=true
            shift
            ;;
        *)
            CONTRACT_PATH=$1
            shift
            ;;
    esac
done

# Main execution
if [ -z "$CONTRACT_PATH" ]; then
    # No arguments - test all examples
    print_info "Testing all examples..."
    
    failed=0
    total=0
    
    for example_dir in examples/*/*/; do
        if [ -f "$example_dir/Cargo.toml" ]; then
            total=$((total + 1))
            
            if ! test_contract "$example_dir" "$VERBOSE"; then
                failed=$((failed + 1))
                continue
            fi
            
            if [ "$RUN_CLIPPY" = true ]; then
                if ! run_clippy "$example_dir"; then
                    failed=$((failed + 1))
                    continue
                fi
            fi
            
            if [ "$CHECK_FORMAT" = true ]; then
                if ! check_format "$example_dir"; then
                    failed=$((failed + 1))
                    continue
                fi
            fi
            
            echo ""
        fi
    done
    
    echo "================================"
    print_info "Test Summary:"
    print_info "Total: $total"
    print_info "Success: $((total - failed))"
    
    if [ $failed -gt 0 ]; then
        print_error "Failed: $failed"
        exit 1
    else
        print_info "All tests passed! ✓"
    fi
else
    # Test specific contract
    test_contract "$CONTRACT_PATH" "$VERBOSE"
    
    if [ "$RUN_CLIPPY" = true ]; then
        run_clippy "$CONTRACT_PATH"
    fi
    
    if [ "$CHECK_FORMAT" = true ]; then
        check_format "$CONTRACT_PATH"
    fi
fi

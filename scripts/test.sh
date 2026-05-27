#!/bin/bash

# Test Script for Soroban Contracts
# Usage: ./scripts/test.sh [OPTIONS] [example-path]
# 
# Options:
#   -v, --verbose     Show detailed test output
#   -c, --clippy      Run clippy linting
#   -f, --format      Check code formatting
#   -a, --all         Run all checks (clippy + format)
#   --coverage        Generate coverage report
#   -h, --help        Show this help message
#
# Examples:
#   ./scripts/test.sh                                    # Test all examples
#   ./scripts/test.sh examples/basics/01-hello-world     # Test specific example
#   ./scripts/test.sh --coverage                         # Run with coverage
#   ./scripts/test.sh -a examples/basics                 # Run all checks on basics

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

show_help() {
    echo "Test Script for Soroban Contracts"
    echo ""
    echo "Usage: $0 [OPTIONS] [example-path]"
    echo ""
    echo "Options:"
    echo "  -v, --verbose     Show detailed test output"
    echo "  -c, --clippy      Run clippy linting"
    echo "  -f, --format      Check code formatting"
    echo "  -a, --all         Run all checks (clippy + format)"
    echo "  --coverage        Generate coverage report"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Test all examples"
    echo "  $0 examples/basics/01-hello-world     # Test specific example"
    echo "  $0 --coverage                         # Run with coverage"
    echo "  $0 -a examples/basics                 # Run all checks on basics"
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

# Function to generate coverage report
generate_coverage() {
    local target_path=${1:-"."}
    
    print_test "Generating coverage report..."
    
    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warn "cargo-tarpaulin not found. Installing..."
        if ! cargo install cargo-tarpaulin --locked; then
            print_error "Failed to install cargo-tarpaulin"
            return 1
        fi
    fi
    
    # Create coverage directory
    mkdir -p coverage
    
    # Generate coverage
    if [ "$target_path" = "." ]; then
        # Workspace coverage
        if cargo tarpaulin --workspace --all-features --out xml --output-dir ./coverage --timeout 300; then
            print_info "✓ Coverage report generated: coverage/cobertura.xml"
            
            # Show coverage summary if available
            if [ -f "coverage/cobertura.xml" ]; then
                if command -v grep &> /dev/null && command -v awk &> /dev/null; then
                    local coverage_percent=$(grep -o 'line-rate="[^"]*"' coverage/cobertura.xml | head -1 | awk -F'"' '{printf "%.1f", $2 * 100}')
                    if [ -n "$coverage_percent" ]; then
                        print_info "Coverage: ${coverage_percent}%"
                    fi
                fi
            fi
            return 0
        else
            print_error "✗ Coverage generation failed"
            return 1
        fi
    else
        # Single package coverage
        cd "$target_path"
        if cargo tarpaulin --all-features --out xml --output-dir ../../coverage --timeout 300; then
            print_info "✓ Coverage report generated for $target_path"
            cd - > /dev/null
            return 0
        else
            print_error "✗ Coverage generation failed for $target_path"
            cd - > /dev/null
            return 1
        fi
    fi
}

# Function to find all testable contracts
find_contracts() {
    local base_path=${1:-"examples"}
    local contracts=()
    
    if [ -d "$base_path" ]; then
        while IFS= read -r -d '' contract_dir; do
            contracts+=("$contract_dir")
        done < <(find "$base_path" -name "Cargo.toml" -type f -print0 | xargs -0 dirname | sort)
    fi
    
    printf '%s\n' "${contracts[@]}"
}

# Function to test workspace
test_workspace() {
    local verbose=${1:-false}
    
    print_test "Testing entire workspace..."
    
    if [ "$verbose" = true ]; then
        cargo test --workspace --all-features -- --nocapture
    else
        cargo test --workspace --all-features --quiet
    fi
    
    local result=$?
    
    if [ $result -eq 0 ]; then
        print_info "✓ All workspace tests passed"
        return 0
    else
        print_error "✗ Workspace tests failed"
        return 1
    fi
}
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

# Function to run coverage
run_coverage() {
    local contract_path=$1
    
    print_test "Running coverage on: $contract_path"
    
    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warn "cargo-tarpaulin not found. Installing..."
        if ! cargo install cargo-tarpaulin; then
            print_error "Failed to install cargo-tarpaulin"
            return 1
        fi
    fi
    
    cd "$contract_path"
    
    if cargo tarpaulin --out Html --output-dir coverage 2>&1; then
        print_info "✓ Coverage report generated in $contract_path/coverage/"
        cd - > /dev/null
        return 0
    else
        print_error "✗ Coverage generation failed"
        cd - > /dev/null
        return 1
    fi
}

# Parse arguments
VERBOSE=false
RUN_CLIPPY=false
CHECK_FORMAT=false
GENERATE_COVERAGE=false
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
        --coverage)
            GENERATE_COVERAGE=true
            RUN_COVERAGE=true
            shift
            ;;
        -a|--all)
            RUN_CLIPPY=true
            CHECK_FORMAT=true
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
            CONTRACT_PATH=$1
            shift
            ;;
    esac
done

# Check dependencies
check_dependencies

# Main execution
if [ -z "$CONTRACT_PATH" ]; then
    # No path specified - test all examples or workspace
    print_info "Testing all examples..."
    
    failed=0
    total=0
    
    # Use workspace testing for better performance
    if test_workspace "$VERBOSE"; then
        print_info "✓ Workspace tests passed"
    else
        print_error "✗ Workspace tests failed"
        exit 1
    fi
    
    # Run additional checks if requested
    if [ "$RUN_CLIPPY" = true ] || [ "$CHECK_FORMAT" = true ]; then
        print_info "Running additional checks on individual contracts..."
        
        readarray -t contracts < <(find_contracts "examples")
        total=${#contracts[@]}
        
        for contract_dir in "${contracts[@]}"; do
            if [ "$RUN_CLIPPY" = true ]; then
                if ! run_clippy "$contract_dir"; then
                    failed=$((failed + 1))
                fi
            fi
            
            if [ "$CHECK_FORMAT" = true ]; then
                if ! check_format "$contract_dir"; then
                    failed=$((failed + 1))
                fi
            fi
        done
        
        if [ $failed -gt 0 ]; then
            print_error "Additional checks failed for $failed/$total contracts"
            exit 1
        fi
    fi
    
    # Generate coverage if requested
    if [ "$GENERATE_COVERAGE" = true ]; then
        if ! generate_coverage; then
            print_warn "Coverage generation failed, but tests passed"
        fi
    fi
    
    print_info "All tests and checks passed! ✓"
else
    # Test specific contract or directory
    if [ -d "$CONTRACT_PATH" ]; then
        if [ -f "$CONTRACT_PATH/Cargo.toml" ]; then
            # Single contract
            if ! test_contract "$CONTRACT_PATH" "$VERBOSE"; then
                exit 1
            fi
            
            if [ "$RUN_CLIPPY" = true ]; then
                if ! run_clippy "$CONTRACT_PATH"; then
                    exit 1
                fi
            fi
            
            if [ "$CHECK_FORMAT" = true ]; then
                if ! check_format "$CONTRACT_PATH"; then
                    exit 1
                fi
            fi
            
            if [ "$GENERATE_COVERAGE" = true ]; then
                generate_coverage "$CONTRACT_PATH"
            fi
        else
            # Directory with multiple contracts
            print_info "Testing contracts in directory: $CONTRACT_PATH"
            
            readarray -t contracts < <(find_contracts "$CONTRACT_PATH")
            
            if [ ${#contracts[@]} -eq 0 ]; then
                print_error "No contracts found in $CONTRACT_PATH"
                exit 1
            fi
            
            failed=0
            total=${#contracts[@]}
            
            for contract_dir in "${contracts[@]}"; do
                if ! test_contract "$contract_dir" "$VERBOSE"; then
                    failed=$((failed + 1))
                    continue
                fi
                
                if [ "$RUN_CLIPPY" = true ]; then
                    if ! run_clippy "$contract_dir"; then
                        failed=$((failed + 1))
                        continue
                    fi
                fi
                
                if [ "$CHECK_FORMAT" = true ]; then
                    if ! check_format "$contract_dir"; then
                        failed=$((failed + 1))
                        continue
                    fi
                fi
            done
            
            if [ "$GENERATE_COVERAGE" = true ]; then
                generate_coverage "$CONTRACT_PATH"
            fi
            
            echo "================================"
            print_info "Test Summary for $CONTRACT_PATH:"
            print_info "Total: $total"
            print_info "Success: $((total - failed))"
            
            if [ $failed -gt 0 ]; then
                print_error "Failed: $failed"
                exit 1
            else
                print_info "All tests passed! ✓"
            fi
        fi
    else
        print_error "Path not found: $CONTRACT_PATH"
        exit 1
    fi
    
    if [ "$RUN_COVERAGE" = true ]; then
        run_coverage "$CONTRACT_PATH"
    fi
fi

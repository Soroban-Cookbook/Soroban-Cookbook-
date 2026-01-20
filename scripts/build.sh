#!/bin/bash

# Build Script for Soroban Contracts
# Usage: ./scripts/build.sh [example-path]
# Example: ./scripts/build.sh examples/basics/01-hello-world

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    print_warn "wasm32-unknown-unknown target not found. Installing..."
    rustup target add wasm32-unknown-unknown
fi

# Check if soroban CLI is installed
if ! command -v soroban &> /dev/null; then
    print_warn "Soroban CLI not found. Installing..."
    cargo install --locked soroban-cli
fi

# Function to build a single contract
build_contract() {
    local contract_path=$1
    
    if [ ! -d "$contract_path" ]; then
        print_error "Directory not found: $contract_path"
        return 1
    fi
    
    if [ ! -f "$contract_path/Cargo.toml" ]; then
        print_error "No Cargo.toml found in $contract_path"
        return 1
    fi
    
    print_info "Building contract: $contract_path"
    
    cd "$contract_path"
    
    # Run tests first
    print_info "Running tests..."
    if cargo test --quiet; then
        print_info "✓ Tests passed"
    else
        print_error "✗ Tests failed"
        cd - > /dev/null
        return 1
    fi
    
    # Build the contract
    print_info "Building WASM..."
    if cargo build --target wasm32-unknown-unknown --release; then
        print_info "✓ Build successful"
        
        # Show output location
        local wasm_file=$(find target/wasm32-unknown-unknown/release -name "*.wasm" | grep -v ".d")
        if [ -n "$wasm_file" ]; then
            local size=$(ls -lh "$wasm_file" | awk '{print $5}')
            print_info "Output: $wasm_file ($size)"
        fi
    else
        print_error "✗ Build failed"
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
    return 0
}

# Main execution
if [ $# -eq 0 ]; then
    # No arguments - build all examples
    print_info "Building all examples..."
    
    failed=0
    total=0
    
    for example_dir in examples/*/*/; do
        if [ -f "$example_dir/Cargo.toml" ]; then
            total=$((total + 1))
            if ! build_contract "$example_dir"; then
                failed=$((failed + 1))
            fi
            echo ""
        fi
    done
    
    echo "================================"
    print_info "Build Summary:"
    print_info "Total: $total"
    print_info "Success: $((total - failed))"
    if [ $failed -gt 0 ]; then
        print_error "Failed: $failed"
        exit 1
    else
        print_info "All builds successful! ✓"
    fi
else
    # Build specific contract
    build_contract "$1"
fi

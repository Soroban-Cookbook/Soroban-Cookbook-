#!/bin/bash

# Build Script for Soroban Contracts
# Usage: ./scripts/build.sh [example-path]
# Example: ./scripts/build.sh examples/basics/01-hello-world

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

print_build() {
    echo -e "${BLUE}[BUILD]${NC} $1"
}

# Check dependencies
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install from https://rustup.rs/"
    exit 1
fi

if ! command -v stellar &> /dev/null; then
    print_error "Stellar CLI is not installed. Install with: cargo install --locked stellar-cli --version 22.1.0"
    exit 1
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
    
    print_build "Building contract: $contract_path"
    
    # Build WASM with Stellar CLI
    if ! (cd "$contract_path" && stellar contract build --quiet); then
        print_error "Build failed for $contract_path"
        return 1
    fi
    
    # Find WASM file
    local wasm_file=$(find "$contract_path/target/wasm32-unknown-unknown/release" -name "*.wasm" ! -name "*.d" | head -n 1)
    
    if [ -n "$wasm_file" ]; then
        local size=$(du -h "$wasm_file" | cut -f1)
        print_info "✓ Built: $wasm_file ($size)"
        
        # Optimize WASM
        local optimized_wasm="${wasm_file%.wasm}.optimized.wasm"
        print_info "Optimizing WASM..."
        if stellar contract optimize --wasm "$wasm_file" --wasm-out "$optimized_wasm" &> /dev/null; then
            local opt_size=$(du -h "$optimized_wasm" | cut -f1)
            print_info "✓ Optimized: $optimized_wasm ($opt_size)"
        else
            print_warn "Optimization failed, using unoptimized WASM"
        fi
    fi
    
    return 0
}

# Parse arguments
CONTRACT_PATH=""

if [ $# -gt 0 ]; then
    CONTRACT_PATH=$1
fi

# Main execution
if [ -z "$CONTRACT_PATH" ]; then
    # Build all examples
    print_info "Building all examples..."
    echo ""
    
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
    if build_contract "$CONTRACT_PATH"; then
        print_info "Build completed successfully!"
    else
        exit 1
    fi
fi

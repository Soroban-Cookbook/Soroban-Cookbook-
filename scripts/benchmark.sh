#!/bin/bash

# Performance Benchmarking Script for Soroban Contracts
# Usage: ./scripts/benchmark.sh [example-path]
# Example: ./scripts/benchmark.sh examples/basics/01-hello-world

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

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Function to benchmark a single contract
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
    
    print_bench "Benchmarking contract: $contract_path"
    
    cd "$contract_path"
    
    # Run benchmarking tests
    # Note: We look for tests with 'benchmark' in their name
    cargo test -- --nocapture benchmark
    
    local result=$?
    
    cd - > /dev/null
    
    if [ $result -eq 0 ]; then
        print_info "✓ Benchmarking completed"
        return 0
    else
        print_warn "! No benchmark tests found or benchmarking failed"
        return 1
    fi
}

# If no path provided, benchmark all basic examples
if [ -z "$1" ]; then
    print_info "No path provided, benchmarking all basic examples..."
    for dir in examples/basics/*/; do
        benchmark_contract "$dir"
    done
else
    benchmark_contract "$1"
fi

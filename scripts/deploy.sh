#!/bin/bash

# Deploy Script for Soroban Contracts
# Usage: ./scripts/deploy.sh <contract-path> <network> [identity]
# Example: ./scripts/deploy.sh examples/basics/01-hello-world testnet alice

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract-path> <network> [identity]"
    echo ""
    echo "Examples:"
    echo "  $0 examples/basics/01-hello-world testnet alice"
    echo "  $0 examples/basics/01-hello-world mainnet my-key"
    exit 1
fi

CONTRACT_PATH=$1
NETWORK=$2
IDENTITY=${3:-"default"}

# Check if soroban CLI is installed
if ! command -v soroban &> /dev/null; then
    print_error "Soroban CLI not found. Install with: cargo install soroban-cli"
    exit 1
fi

# Verify contract path exists
if [ ! -d "$CONTRACT_PATH" ]; then
    print_error "Contract path not found: $CONTRACT_PATH"
    exit 1
fi

# Verify network is configured
if ! soroban network ls | grep -q "^$NETWORK"; then
    print_error "Network '$NETWORK' not configured"
    print_info "Configure with:"
    echo "  soroban network add --global $NETWORK \\"
    echo "    --rpc-url <RPC_URL> \\"
    echo "    --network-passphrase <PASSPHRASE>"
    exit 1
fi

# Build the contract
print_info "Building contract..."
cd "$CONTRACT_PATH"

if ! cargo build --target wasm32-unknown-unknown --release --quiet; then
    print_error "Build failed"
    exit 1
fi

# Find the WASM file
WASM_FILE=$(find target/wasm32-unknown-unknown/release -name "*.wasm" | grep -v ".d" | head -n 1)

if [ -z "$WASM_FILE" ]; then
    print_error "WASM file not found"
    exit 1
fi

print_info "WASM file: $WASM_FILE"

# Check if identity exists
if ! soroban keys ls | grep -q "^$IDENTITY"; then
    print_warn "Identity '$IDENTITY' not found"
    print_info "Generating new identity..."
    soroban keys generate "$IDENTITY" --network "$NETWORK"
fi

# Fund account on testnet if needed
if [ "$NETWORK" = "testnet" ]; then
    print_info "Ensuring account is funded..."
    soroban keys fund "$IDENTITY" --network "$NETWORK" || true
fi

# Deploy the contract
print_info "Deploying to $NETWORK..."

CONTRACT_ID=$(soroban contract deploy \
    --wasm "$WASM_FILE" \
    --source "$IDENTITY" \
    --network "$NETWORK" 2>&1)

if [ $? -eq 0 ]; then
    print_info "✓ Deployment successful!"
    echo ""
    echo "Contract ID: $CONTRACT_ID"
    echo ""
    print_info "Save this contract ID for future interactions"
    
    # Save contract ID to file
    echo "$CONTRACT_ID" > .contract-id-$NETWORK
    print_info "Contract ID saved to .contract-id-$NETWORK"
else
    print_error "Deployment failed"
    exit 1
fi

# Return to original directory
cd - > /dev/null

exit 0

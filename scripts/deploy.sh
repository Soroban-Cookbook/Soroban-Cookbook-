#!/bin/bash

# Deploy Script for Soroban Contracts
# Usage: ./scripts/deploy.sh <contract-path> <network> [identity]
# Example: ./scripts/deploy.sh examples/basics/01-hello-world testnet alice

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# --- Argument validation ---

if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract-path> <network> [identity]"
    echo ""
    echo "  network   testnet | mainnet"
    echo "  identity  stellar keys name (default: \"default\")"
    echo ""
    echo "Example:"
    echo "  $0 examples/basics/01-hello-world testnet alice"
    exit 1
fi

CONTRACT_PATH=$1
NETWORK=$2
IDENTITY=${3:-"default"}

# --- Network selection ---

if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
    print_error "Network must be 'testnet' or 'mainnet', got: $NETWORK"
    exit 1
fi

# --- Dependency check ---

if ! command -v stellar &> /dev/null; then
    print_error "stellar CLI not found. Install with: cargo install --locked stellar-cli --version 22.1.0"
    exit 1
fi

if [ ! -d "$CONTRACT_PATH" ]; then
    print_error "Contract path not found: $CONTRACT_PATH"
    exit 1
fi

if [ ! -f "$CONTRACT_PATH/Cargo.toml" ]; then
    print_error "No Cargo.toml found in $CONTRACT_PATH"
    exit 1
fi

# --- Build ---

print_info "Building contract in $CONTRACT_PATH..."
(cd "$CONTRACT_PATH" && stellar contract build --quiet)

WASM_FILE=$(find "$CONTRACT_PATH/target/wasm32-unknown-unknown/release" -name "*.wasm" ! -name "*.d" | head -n 1)

if [ -z "$WASM_FILE" ]; then
    print_error "WASM file not found after build"
    exit 1
fi

print_info "Built: $WASM_FILE"

# --- Optimization ---

print_info "Optimizing WASM..."
OPTIMIZED_WASM="${WASM_FILE%.wasm}.optimized.wasm"
stellar contract optimize --wasm "$WASM_FILE" --wasm-out "$OPTIMIZED_WASM"
print_info "Optimized: $OPTIMIZED_WASM"

# --- Identity ---

if ! stellar keys ls 2>/dev/null | grep -q "^$IDENTITY$"; then
    print_warn "Identity '$IDENTITY' not found — generating..."
    stellar keys generate "$IDENTITY" --network "$NETWORK"
fi

if [ "$NETWORK" = "testnet" ]; then
    print_info "Funding account on testnet..."
    stellar keys fund "$IDENTITY" --network "$NETWORK" || true
fi

# --- Deploy ---

print_info "Deploying to $NETWORK..."

CONTRACT_ID=$(stellar contract deploy \
    --wasm "$OPTIMIZED_WASM" \
    --source "$IDENTITY" \
    --network "$NETWORK")

print_info "✓ Deployment successful!"
echo ""
echo "Contract ID: $CONTRACT_ID"
echo ""

# --- Capture contract ID ---

ID_FILE="$CONTRACT_PATH/.contract-id-$NETWORK"
echo "$CONTRACT_ID" > "$ID_FILE"
print_info "Contract ID saved to $ID_FILE"

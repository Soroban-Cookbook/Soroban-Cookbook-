#!/bin/bash
# Build and Deploy NFT Marketplace Contract to Stellar Testnet

set -e

echo "🎨 Building NFT Marketplace Smart Contract..."
cd "$(dirname "$0")/../contracts/marketplace"
cargo build --target wasm32-unknown-unknown --release

WASM_PATH="../../target/wasm32-unknown-unknown/release/nft_marketplace.wasm"

echo "🚀 Deploying to Stellar Testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source deployer \
  --network testnet)

echo "✅ Contract Deployed Successfully!"
echo "Contract ID: $CONTRACT_ID"

echo "⚙️ Initializing Marketplace..."
DEPLOYER_ADDRESS=$(stellar keys address deployer)

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin "$DEPLOYER_ADDRESS"

echo "📝 Updating Frontend Configuration..."
CONFIG_FILE="../../frontend/src/app.js"
sed -i.bak "s/contractId: \".*\"/contractId: \"$CONTRACT_ID\"/" "$CONFIG_FILE" && rm -f "${CONFIG_FILE}.bak"

echo "🎉 NFT Marketplace Ready! Run 'cd frontend && npm run dev'."

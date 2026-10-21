#!/bin/bash
# Build and Deploy Token Contract to Stellar Testnet

set -e

echo "🔨 Building Token Smart Contract..."
cd "$(dirname "$0")/../contracts/token"
cargo build --target wasm32-unknown-unknown --release

WASM_PATH="../../target/wasm32-unknown-unknown/release/token_contract.wasm"

echo "🚀 Deploying to Stellar Testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source deployer \
  --network testnet)

echo "✅ Contract Deployed Successfully!"
echo "Contract ID: $CONTRACT_ID"

echo "⚙️ Initializing Token Parameters..."
DEPLOYER_ADDRESS=$(stellar keys address deployer)

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin "$DEPLOYER_ADDRESS" \
  --name "Community Token" \
  --symbol "COMM" \
  --decimals 7 \
  --initial_supply 10000000000000

echo "📝 Updating Frontend Configuration..."
CONFIG_FILE="../../frontend/src/app.js"
sed -i.bak "s/contractId: \".*\"/contractId: \"$CONTRACT_ID\"/" "$CONFIG_FILE" && rm -f "${CONFIG_FILE}.bak"

echo "🎉 Setup Complete! Run 'cd frontend && npm run dev' to launch."

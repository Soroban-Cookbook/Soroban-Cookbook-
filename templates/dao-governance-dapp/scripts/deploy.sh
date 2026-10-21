#!/bin/bash
# Build and Deploy DAO Governance Contract to Stellar Testnet

set -e

echo "🏛️ Building DAO Governance Smart Contract..."
cd "$(dirname "$0")/../contracts/governance"
cargo build --target wasm32-unknown-unknown --release

WASM_PATH="../../target/wasm32-unknown-unknown/release/dao_governance.wasm"

echo "🚀 Deploying to Stellar Testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source deployer \
  --network testnet)

echo "✅ Contract Deployed Successfully!"
echo "Contract ID: $CONTRACT_ID"

echo "⚙️ Initializing DAO Parameters..."
DEPLOYER_ADDRESS=$(stellar keys address deployer)

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin "$DEPLOYER_ADDRESS" \
  --quorum_votes 1000 \
  --voting_period_ledgers 17280

echo "📝 Updating Frontend Configuration..."
CONFIG_FILE="../../frontend/src/app.js"
sed -i.bak "s/contractId: \".*\"/contractId: \"$CONTRACT_ID\"/" "$CONFIG_FILE" && rm -f "${CONFIG_FILE}.bak"

echo "🎉 DAO Governance Ready! Run 'cd frontend && npm run dev'."

# Soroban Project Templates

Production-ready, full-stack starter templates for building decentralized applications on Stellar with Soroban. Each template includes a tested smart contract suite, a modern responsive web frontend with Freighter wallet integration, automated deployment scripts, and a developer-ready `README.md`.

---

## 📦 Available Templates

| Template | Primary Use Case | Smart Contract Features | Frontend Features | Difficulty |
| :--- | :--- | :--- | :--- | :---: |
| **[`token-dapp/`](./token-dapp/)** | Custom Tokens, Stablecoins, Points Systems | SEP-41 compliant token, mint/burn, allowances, storage TTL | Balance display, transfer form, mint interface, Freighter connect | Beginner |
| **[`nft-marketplace-dapp/`](./nft-marketplace-dapp/)** | Digital Collectibles, Marketplaces | NFT minting, fixed-price listing, atomic buying, cancellation | Gallery grid, minting modal, listing cards, purchase workflow | Intermediate |
| **[`dao-governance-dapp/`](./dao-governance-dapp/)** | DAOs, Community Treasuries, Voting | Proposal lifecycle, weighted voting, quorum check, timelock execution | Active proposal board, live tally bars, vote casting, treasury stats | Intermediate |

---

## 🚀 Quickstart: How to Fork and Build

### 1. Choose & Copy a Template
You can fork this repository or simply copy the desired template directory into your new project folder:

```bash
# Clone the cookbook
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git

# Copy your chosen template (e.g., token-dapp)
cp -r Soroban-Cookbook-/templates/token-dapp my-stellar-project
cd my-stellar-project
```

### 2. Prerequisites
Ensure you have the following installed:
- [Rust & Cargo](https://rustup.rs/) (`rustc 1.74+` or `stable`)
- Target `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli) (`cargo install --locked stellar-cli`)
- [Node.js](https://nodejs.org/) (v18+) & `npm` / `bun`
- [Freighter Wallet](https://www.freighter.app/) browser extension configured for Testnet

### 3. Test Smart Contracts
Navigate to the contract directory and run the test suite:

```bash
cd contracts/token  # or marketplace / governance
cargo test
```

### 4. Deploy to Stellar Testnet
Use the included automated deployment script to build the WASM binary, deploy to Stellar Testnet, and configure the frontend:

```bash
# Set your deployer identity
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

# Run the deployment script
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

### 5. Launch the Frontend
Start the local development web server:

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:3000` in your browser, connect Freighter, and interact with your live smart contract on Testnet!

---

## 🛡️ Best Practices Baked In

Every template follows official Soroban and Stellar security standards:
- **Explicit Authorization:** All balance changes and privileged calls verify `require_auth()`.
- **Checked Arithmetic:** Safe integer math prevents overflows and precision truncation.
- **Storage Tiering & TTL:** Optimal use of Instance, Persistent, and Temporary storage with automatic TTL renewal.
- **Auditable Events:** Standard SEP topics emitted on all state transitions for indexing.
- **Comprehensive Testing:** Pre-built unit and integration tests covering happy paths, auth errors, and boundary cases.

---

## 🤝 Contributing New Templates

Have you built a novel full-stack pattern? We welcome community template submissions!
1. Follow the template directory structure (`contracts/`, `frontend/`, `scripts/`, `README.md`).
2. Ensure smart contracts have >90% test coverage and zero Clippy warnings.
3. Open a Pull Request or apply for a grant via our [Grants Program](../docs/grants/README.md).

# Soroban NFT Marketplace dApp Template

A full-stack digital collectible and marketplace template on Stellar using Soroban smart contracts and Freighter Wallet.

---

## 🌟 Features

- **NFT Minting & Storage:** Mint NFTs with on-chain Token IDs and IPFS/Arweave URI metadata.
- **Fixed-Price Marketplace:** Secure contract escrow for listed NFTs, atomic payment settlement, and listing cancellation.
- **Modern Responsive Web Client:** Gallery grid, minting modal, price display, and buy flow.
- **Freighter Wallet Integration:** Interactive wallet connection and transaction signing.
- **Automated Deployment:** Testnet deployment script included.

---

## 📁 Project Structure

```
nft-marketplace-dapp/
├── contracts/
│   └── marketplace/
│       └── src/
│           ├── lib.rs       # NFT & Marketplace contract logic
│           └── test.rs      # Unit test suite
├── frontend/
│   ├── index.html           # Marketplace gallery UI
│   ├── package.json         # Web dependencies
│   └── src/
│       ├── app.js           # Client interactions & wallet logic
│       └── styles.css       # Dark-mode marketplace styling
├── scripts/
│   └── deploy.sh            # Automated deployment script
├── Cargo.toml               # Rust package configuration
└── README.md                # Project documentation
```

---

## 🚀 Getting Started

### 1. Test Contracts
This template is a single package that is a member of the cookbook Cargo
workspace, so it is built and tested by `cargo build --workspace` /
`cargo test --workspace`. Run it directly from the repository root:

```bash
cargo test -p nft-marketplace-dapp
```

Or from inside the template directory:

```bash
cd templates/nft-marketplace-dapp
cargo test
```

### 2. Deploy to Testnet
```bash
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

### 3. Start Frontend
```bash
cd frontend
npm install
npm run dev
```

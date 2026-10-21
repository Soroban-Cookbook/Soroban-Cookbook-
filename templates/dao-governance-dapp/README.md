# Soroban DAO Governance & Treasury dApp Template

A full-stack decentralized governance portal and community treasury on Stellar using Soroban smart contracts and Freighter Wallet.

---

## 🌟 Features

- **Decentralized Proposal Lifecycle:** Propose, vote with weighted tokens, track live tallies, and execute passed proposals.
- **Configurable Governance Parameters:** Customize quorum thresholds and voting period durations in ledgers.
- **Modern Responsive Web Client:** Proposal cards, visual vote tally bars, proposal creation modal, and activity log.
- **Freighter Wallet Integration:** Seamless voting and proposal signing.
- **Automated Deployment:** Includes Testnet deployment script.

---

## 📁 Project Structure

```
dao-governance-dapp/
├── contracts/
│   └── governance/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # DAO Governance contract logic
│           └── test.rs      # Unit test suite
├── frontend/
│   ├── index.html           # Governance dashboard UI
│   ├── package.json         # Web dependencies
│   └── src/
│       ├── app.js           # Client interactions & voting logic
│       └── styles.css       # Clean dashboard styling
├── scripts/
│   └── deploy.sh            # Automated deployment script
├── Cargo.toml               # Rust workspace configuration
└── README.md                # Project documentation
```

---

## 🚀 Getting Started

### 1. Test Contracts
```bash
cd contracts/governance
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

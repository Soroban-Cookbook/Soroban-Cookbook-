# Soroban Fungible Token dApp Template

A production-ready full-stack template for building and managing SEP-41 compliant fungible tokens on Stellar using Soroban and Freighter Wallet.

---

## 🌟 Features

- **SEP-41 Compliant Token Contract:** Includes `initialize`, `balance`, `transfer`, `transfer_from`, `approve`, `allowance`, `mint`, and `burn`.
- **Safe Storage & TTL Management:** Automated instance and balance TTL extensions preventing data archival.
- **Modern Responsive Web Client:** Built with modern CSS and Vanilla JS for instant setup without heavy framework overhead.
- **Freighter Wallet Integration:** Seamless connection, signing, and live transaction feedback.
- **Automated Deployment:** One-click deployment script with CLI integration.

---

## 📁 Project Structure

```
token-dapp/
├── contracts/
│   └── token/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # Token smart contract logic
│           └── test.rs      # Unit test suite
├── frontend/
│   ├── index.html           # Main UI dashboard
│   ├── package.json         # Web dependencies
│   └── src/
│       ├── app.js           # Wallet connector & contract client
│       └── styles.css       # Modern dark-mode styling
├── scripts/
│   └── deploy.sh            # Build, deploy & initialization script
├── Cargo.toml               # Rust workspace config
└── README.md                # Project documentation
```

---

## 🧩 Why this is a standalone workspace

This template declares its **own** `[workspace]` in `templates/token-dapp/Cargo.toml` (that is the "Rust workspace config" in the tree above) and resolves `soroban-sdk` from its own `[workspace.dependencies]` table instead of inheriting it from the cookbook root. It is therefore listed in the root `Cargo.toml` under `exclude`, not `members` — a package cannot belong to two workspaces at once.

What that means in practice:

- **Build it from inside this directory.** `cargo build --workspace` run at the repository root does **not** cover this template, because the root cannot own it. Copy this folder out of the repository and it still builds on its own — that is the point of the template.
- **Its `soroban-sdk` pin is local to this workspace**, not the root workspace's `workspace.dependencies`. This is the documented exception to "every crate inherits the workspace SDK".
- **CI still checks it.** Since the root workspace structurally cannot, the dedicated `.github/workflows/templates.yml` job runs `cargo check` and `cargo test` here on every push and pull request, so this template cannot rot while still looking official.

```bash
cd templates/token-dapp
cargo check
cargo test
```

---

## 🚀 Getting Started

### 1. Test the Smart Contract
```bash
cd contracts/token
cargo test
```

### 2. Deploy to Testnet
```bash
# Configure deployer identity
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

# Run automated deployment
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

### 3. Run the Frontend
```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:3000` to view your live token portal!

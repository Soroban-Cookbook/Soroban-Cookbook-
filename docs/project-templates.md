# Project Templates Guide

The Soroban Cookbook provides three full-stack, production-ready project templates to accelerate smart contract development on Stellar. Each template pairs tested Rust smart contracts with modern web frontends and deployment tooling.

---

## 📦 Available Templates

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Soroban Project Templates                             │
├───────────────────────────────┬─────────────────────────────────────────────┤
│ 🪙 Fungible Token dApp        │ 🎨 NFT Marketplace dApp                     │
│ (`templates/token-dapp/`)     │ (`templates/nft-marketplace-dapp/`)         │
├───────────────────────────────┴─────────────────────────────────────────────┤
│ 🏛️ DAO Governance & Treasury dApp                                          │
│ (`templates/dao-governance-dapp/`)                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Fungible Token dApp (`templates/token-dapp/`)

### Overview
A full-stack starter kit for creating, distributing, and managing SEP-41 compliant fungible tokens.

### Key Capabilities
- **Smart Contract:** SEP-41 token operations (`transfer`, `transfer_from`, `approve`, `mint`, `burn`, `balance`, `allowance`, `decimals`, `name`, `symbol`).
- **Storage Strategy:** Instance storage for global metadata and persistent storage with automated TTL extension for user balances.
- **Frontend Dashboard:** Live balance query, token transfer form, admin mint modal, and Freighter wallet integration.

### Quick Start
```bash
cd templates/token-dapp/contracts/token
cargo test
```

---

## 2. NFT Marketplace dApp (`templates/nft-marketplace-dapp/`)

### Overview
A full-stack digital collectible and marketplace dApp for minting NFTs and trading them at fixed prices.

### Key Capabilities
- **Smart Contract:** On-chain NFT minting with URI metadata, fixed-price listing, contract escrow, atomic payment settlement, and listing cancellation.
- **Security:** Verified seller ownership, state mutation atomicity, and safe delisting.
- **Frontend Gallery:** Visual collectible grid, minting modal, listing cards with price in XLM, and 1-click buy button.

### Quick Start
The NFT marketplace template is a single package that is a member of the cookbook
Cargo workspace, so it is built and tested with the workspace `soroban-sdk`.
From the repository root:

```bash
cargo test -p nft-marketplace-dapp
```

Or from inside the template directory:

```bash
cd templates/nft-marketplace-dapp
cargo test
```

---

## 3. DAO Governance & Treasury dApp (`templates/dao-governance-dapp/`)

### Overview
A full-stack community governance portal for managing proposals, voting with token weights, and executing treasury disbursements.

### Key Capabilities
- **Smart Contract:** Proposal registration, weighted voting, quorum and threshold validation, and execution timelocks.
- **Security:** Single-vote per account enforcement, ledger sequence time checks, and status validation.
- **Frontend Portal:** Active proposal dashboard, live vote tally bars (For/Against), proposal submission modal, and execution trigger.

### Quick Start
```bash
cd templates/dao-governance-dapp/contracts/governance
cargo test
```

---

## 🛠️ Common Architecture & Best Practices

All templates implement the following architectural guarantees:
1. **Explicit Authorization:** Critical state mutations require `from.require_auth()` or `admin.require_auth()`.
2. **Checked Arithmetic:** Zero integer overflow or underflow risks using Rust's checked arithmetic.
3. **Storage TTL Management:** Both instance and persistent entries have their TTL refreshed to prevent unexpected state archival.
4. **Auditable Events:** Standard topics emitted for off-chain indexing.
5. **No Placeholders:** Fully functional contracts, complete unit tests, and interactive demo UIs.

---

## 🔗 Related Resources

- [Templates Directory](../templates/)
- [Token Security Checklist](./token-security-checklist.md)
- [Testing Best Practices](./testing-best-practices.md)
- [Security Best Practices](./security-best-practices.md)
- [Grants Program](./grants/README.md)

# Soroban Wallet Ecosystem Survey

This document provides a comprehensive survey of Soroban-compatible wallets, comparing their features, detailing integration requirements, and recommending a developer integration priority list.

---

## Wallet Overview

Smart contracts on Stellar (Soroban) require secure wallets that can parse and sign Soroban contract transactions (invocations, authorization payloads).

### 1. Freighter
- **Publisher:** Stellar Development Foundation (SDF)
- **Format:** Browser Extension (Chrome, Firefox)
- **Developer Focus:** High. De facto standard for local development, Testnet, and Mainnet.
- **Key Feature:** Native support for Soroban transaction parsing, signing, and sandboxed test network switching.

### 2. xBull Wallet
- **Publisher:** Creit Tech
- **Format:** Browser Extension & Mobile App (Android/iOS)
- **Developer Focus:** High. Designed with developers and power users in mind.
- **Key Feature:** Support for custom RPC endpoints, advanced multisig setups, and multiple account management.

### 3. Albedo
- **Publisher:** Independent Community Developer
- **Format:** Browser-based / Web App (no install required)
- **Developer Focus:** Medium. Offers a seamless web-flow for signing without extensions.
- **Key Feature:** Highly mobile-friendly web bridge for signing transactions.

### 4. Lobstr
- **Publisher:** Ultra Stellar
- **Format:** Web & Mobile App
- **Developer Focus:** Low (Consumer focused).
- **Key Feature:** The most popular retail wallet in the Stellar ecosystem. Slowly rolling out support for Soroban token transfers and simple invocations.

---

## Feature Comparison Matrix

| Feature | Freighter | xBull Wallet | Albedo | Lobstr |
|---|---|---|---|---|
| **Type** | Extension | Extension & Mobile | Web App | Web & Mobile |
| **Soroban Support** | Full | Full | Partial | Basic (Tokens) |
| **Testnet/Futurenet** | Yes | Yes | Yes | Mainnet Only |
| **Custom RPC/Nodes** | No | Yes | No | No |
| **Hardware Wallet** | Ledger | Ledger | Ledger | Multisig (Vault) |
| **Open Source** | Yes | Yes | Yes | Proprietary |
| **Developer UX** | Excellent | Advanced | Good | Simple |

---

## Integration Requirements

### 1. Connecting & Fetching Address
For browser extensions (Freighter and xBull), you can leverage their respective client libraries or standard browser event injection.

```javascript
import { requestAccess, getAddress } from "@stellar/freighter-api";

async function connectWallet() {
  const allowed = await requestAccess();
  if (allowed) {
    const address = await getAddress();
    console.log("Connected address:", address);
    return address;
  }
  throw new Error("Access denied");
}
```

### 2. Signing Soroban Transactions
Transactions must be built using the `stellar-sdk` / `soroban-client` and passed to the wallet for signing.

```javascript
import { signTransaction } from "@stellar/freighter-api";

async function signTx(xdrString) {
  const signedXDR = await signTransaction(xdrString, {
    network: "TESTNET"
  });
  return signedXDR;
}
```

### 3. Standards Compliance
- **WalletConnect v2**: Used for connecting web applications to mobile wallets (such as xBull and Lobstr mobile).
- **Stellar Wallet Standard (SWS)**: An emerging standard interface simplifying cross-wallet provider abstraction in JS/TS frontends.

---

## Developer Priority Integration List

When building a decentralized application (dApp) on Soroban, prioritize wallet integrations in this order:

1. **Freighter (Priority 1 - Critical)**: The standard wallet for all developer tools (like the Soroban Dashboard) and the majority of early adopters. Essential for Testnet testing.
2. **xBull Wallet (Priority 2 - High)**: Provides excellent desktop and mobile coverage, custom RPC switching for private devnets, and advanced multisig features.
3. **Albedo (Priority 3 - Medium)**: Essential fallback for mobile users who want to avoid installing mobile apps or extensions.
4. **Lobstr (Priority 4 - Low/Future)**: Integrate once your application is deployed to Mainnet and target retail users holding standard Stellar/Soroban assets.

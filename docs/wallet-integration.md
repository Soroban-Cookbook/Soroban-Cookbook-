# Wallet Integration Guide

This guide helps Soroban developers add wallet support to web applications, dashboards, and frontends that interact with Stellar smart contracts. It focuses on practical integration patterns, expected user flows, and production recommendations for common wallets in the Stellar ecosystem.

## Table of Contents

- [Introduction](#introduction)
- [Prerequisites](#prerequisites)
- [Supported Wallets](#supported-wallets)
- [Integration Examples](#integration-examples)
- [Testing Guide](#testing-guide)
- [Best Practices](#best-practices)
- [Common Problems](#common-problems)
- [External Resources](#external-resources)

## Introduction

Wallet integration is the layer that lets users connect their account, authorize contract calls, and approve transactions from an application. In Soroban, this matters because most contract interactions require a signed transaction or an authorization payload. A smooth wallet experience improves trust, reduces onboarding friction, and helps users understand what they are signing.

For many teams, the wallet experience is part of the product itself. If users cannot connect, cannot switch networks, or receive confusing errors, adoption drops quickly. Good integration should be reliable, secure, and easy to test.

The wallet ecosystem is evolving quickly. In practice, the most common starting points are:

- Freighter for browser-based development and testing
- xBull for desktop and mobile coverage
- Albedo for web-based signing flows
- WalletConnect-compatible wallets for mobile and browser-based DApps

## Prerequisites

Before you integrate wallets into a Soroban app, make sure your development environment is ready.

### Required tools

- A recent Node.js installation for frontend work
- The Soroban / Stellar CLI for local testing and deployment
- The Soroban Rust SDK when building or calling contracts from Rust-based tooling
- A wallet extension or mobile wallet installed for manual testing

### Recommended packages

For a JavaScript or TypeScript frontend, you will typically need:

```bash
npm install @stellar/stellar-sdk
```

If you are using Freighter-style browser APIs, install the wallet SDK that matches your target wallet:

```bash
npm install @stellar/freighter-api
```

### Environment setup

1. Install the Stellar CLI and confirm that it runs.
2. Configure your preferred network, such as Testnet or Futurenet.
3. Create a funded test account for manual testing.
4. Keep the contract ID, network passphrase, and RPC endpoint in a shared config file so your frontend can use them consistently.

> Wallet support changes quickly. Always check the official provider documentation before relying on a specific API or network flow.

## Supported Wallets

### Freighter

- Overview: Freighter is the most commonly used browser extension for Stellar and Soroban development. It is a strong default choice for local development and Testnet testing.
- Installation: Install the official extension from the Freighter website and confirm it is enabled in your browser.
- Configuration: Add your target network, such as Testnet, and allow the app to connect to it.
- Connection flow: Request access to the wallet, read the public address, and store the result in app state.
- Authentication: Use the wallet to approve account access and to sign Soroban transactions.
- Signing transactions: Build a transaction or contract invocation, then pass it to the wallet for signing.
- Sending transactions: Submit the signed XDR to a Soroban RPC endpoint.
- Disconnecting: Clear local application state and remove the connected address from memory.

### xBull

- Overview: xBull is a broader wallet option that supports desktop and mobile use and is useful when you want a more flexible wallet experience.
- Installation: Install the browser extension or mobile app from the official xBull documentation.
- Configuration: Configure networks, RPC endpoints, and account settings for your preferred environment.
- Connection flow: Connect the app to the wallet and request an address or account selection.
- Authentication: Approve the connection request when the app first accesses the account.
- Signing transactions: Sign transactions through the wallet provider interface.
- Sending transactions: Broadcast the signed transaction using the network RPC.
- Disconnecting: Clear the connection from the app and prompt the user to reconnect if needed.

### Albedo

- Overview: Albedo provides a web-based signing experience that can be helpful for users who prefer not to install a browser extension.
- Installation: No extension is required. Open the wallet flow from your app and follow the web-based sign-in experience.
- Configuration: Configure the app redirect and network details according to the official instructions.
- Connection flow: Redirect the user to the Albedo flow, confirm the requested scopes, and receive the response.
- Authentication: Approve the application access request in the Albedo UI.
- Signing transactions: Request a transaction signature and receive the signed output.
- Sending transactions: Submit the signed transaction to the network after the wallet returns the signature.
- Disconnecting: Clear the session state in your app and instruct the user to start a new signing flow if needed.

### WalletConnect-compatible solutions

- Overview: WalletConnect is commonly used for connecting web apps to mobile wallets or other provider ecosystems.
- Installation: Use the official WalletConnect SDK in your frontend and ensure the wallet supports the same protocol version.
- Configuration: Configure a project ID and the required chain metadata for your app.
- Connection flow: Open a connection modal, choose the wallet, and approve the session.
- Authentication: Ask the wallet to approve the session and any required account scopes.
- Signing transactions: Use the WalletConnect signing flow and return the signed transaction payload.
- Sending transactions: Broadcast the signed transaction using your RPC client.
- Disconnecting: End the session in your app and prompt the user to reconnect as needed.

## Integration Examples

The examples below are intentionally simple and follow the same mental model across wallets: connect, inspect the wallet state, sign, submit, and handle errors.

### Connecting a wallet

```ts
import { requestAccess, getAddress } from "@stellar/freighter-api";

async function connectWallet() {
  const allowed = await requestAccess();
  if (!allowed) {
    throw new Error("Wallet access denied by the user");
  }

  const address = await getAddress();
  return address;
}
```

### Checking connection status

```ts
async function getConnectionStatus() {
  // Use the wallet SDK or provider method that exposes connectivity state.
  // The exact API varies by wallet.
  return {
    connected: true,
    network: "TESTNET"
  };
}
```

### Signing a transaction

```ts
import { signTransaction } from "@stellar/freighter-api";

async function signSorobanTx(xdr: string) {
  const signedXdr = await signTransaction(xdr, { network: "TESTNET" });
  return signedXdr;
}
```

### Submitting a transaction

```ts
import { Server, TransactionBuilder, Networks } from "@stellar/stellar-sdk";

async function submitTransaction(signedXdr: string) {
  const server = new Server("https://soroban-testnet.stellar.org");
  const tx = TransactionBuilder.fromXDR(signedXdr, Networks.TESTNET);
  const result = await server.sendTransaction(tx);
  return result;
}
```

### Handling user rejection

```ts
try {
  await connectWallet();
} catch (error) {
  console.warn("User rejected the connection request", error);
}
```

### Handling network errors

```ts
try {
  await submitTransaction("signed-xdr");
} catch (error) {
  console.error("Transaction submission failed", error);
}
```

### Switching networks

If your app supports multiple networks, check the wallet state before sending a transaction. Some wallets let the user switch networks manually, while others require you to prompt the user to switch to the correct network in the wallet UI.

```ts
function ensureNetwork(targetNetwork: string) {
  if (targetNetwork !== "TESTNET") {
    throw new Error("This example only supports TESTNET");
  }
}
```

## Testing Guide

Testing wallet flows is critical because they involve user consent, network configuration, and external providers.

### Local testing

- Use a local development wallet account and a funded test account.
- Prefer Testnet for manual integration checks.
- Keep your RPC endpoints and network passphrases in a config file.
- Test both success and failure flows.

### Testnet testing

- Use Testnet contracts and test accounts rather than mainnet funds.
- Verify that signatures and submitted transactions behave as expected on the network.
- Track the transaction hash and inspect the result through the Stellar explorer or RPC response.

### Mock wallets where applicable

If you are building a frontend test suite, mock the wallet provider interface instead of trying to depend on the real wallet during automated tests. Keep the mock close to the provider boundary and verify that the app responds correctly to connection success, rejection, and network errors.

### Common debugging steps

1. Check whether the wallet is installed and enabled.
2. Confirm that the network selected in the wallet matches your app’s target network.
3. Verify the transaction XDR is built for the same network and passphrase.
4. Inspect RPC errors and wallet error messages for clues.
5. Log the request object and the wallet error before retrying.

### Troubleshooting common integration issues

- Wallet not detected: confirm the extension is installed and the app is running in a supported browser context.
- User rejected signature: treat this as a normal user flow and show a clear message.
- Wrong network selected: prompt the user to switch to the correct network.
- Transaction failed: inspect the RPC response, fee, and submitted operations.
- RPC connection issues: verify the endpoint and retry with a fallback node if available.

## Best Practices

### Security

- Never store private keys in frontend code.
- Treat every transaction as a user-authorized action, even if the request is generated locally.
- Verify the transaction details before asking the user to sign.
- Prefer explicit user confirmation for high-impact operations.

### Private key safety

- Keep signing keys and seed phrases offline whenever possible.
- Use hardware-backed or secure key storage for administrative accounts.
- Avoid exposing secret material through logs, browser devtools, or analytics.

### Transaction verification

- Confirm the source account, network passphrase, contract ID, and fee before signing.
- Validate the transaction preconditions and any parameters passed to the contract.
- Make sure the user understands what they are authorizing.

### Error handling

- Show actionable error messages instead of generic failures.
- Distinguish between user rejection, network issues, and invalid transaction input.
- Retry only when the failure is transient and safe.

### User experience

- Keep the connection flow short and explain why the app needs access.
- Provide a fallback when the wallet extension is missing.
- Show clear loading and success states for signing and submission.

### Network selection

- Default to Testnet for development and staging.
- Make the active network visible in the UI.
- Detect and warn the user when the wallet is on a different network than the app.

### Session management

- Store only the minimal wallet state needed for the current session.
- Clear stale account state when the user disconnects or switches accounts.
- Reconnect gracefully when the tab or page is refreshed.

### Dependency management

- Pin wallet-related packages to versions that have been tested in your environment.
- Revisit wallet SDK compatibility when upgrading dependencies.
- Keep provider integrations behind a small abstraction so wallet-specific code is isolated.

## Common Problems

| Problem | Likely cause | Suggested fix |
| --- | --- | --- |
| Wallet not detected | Extension is not installed or the browser context is unsupported | Verify installation and browser support, then retry |
| User rejected signature | The user canceled the signing request | Show a clear message and let the user retry |
| Wrong network selected | The wallet and app are on different networks | Prompt the user to switch networks |
| Transaction failed | Bad parameters, bad fee, or invalid contract invocation | Inspect the RPC response and rebuild the transaction |
| RPC connection issues | The node is unavailable or the network endpoint is misconfigured | Retry with another RPC provider or update the endpoint |

## External Resources

- [Stellar Developer Documentation](https://developers.stellar.org/)
- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Soroban Rust SDK](https://docs.rs/soroban-sdk/)
- [Freighter Documentation](https://www.freighter.app/)
- [xBull Wallet Documentation](https://xbull.app/)
- [Albedo Documentation](https://albedo.link/)
- [WalletConnect Documentation](https://docs.walletconnect.com/)

## Summary

A good wallet integration experience is a combination of reliable provider support, clear error handling, and careful testing. Start with the wallet that best matches your audience and your development environment, then expand coverage as your app matures.

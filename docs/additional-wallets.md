# Additional Wallet Support

The [Wallet Integration Guide](./wallet-integration.md) covers **Freighter**, **xBull**, **Albedo**, and WalletConnect. This page adds three more wallets, each with a working TypeScript example and unit tests, plus a comparison guide to help you choose which wallets to support.

| Wallet | Kind | Example | Tests |
|---|---|---|---|
| **Hana** | Browser extension + mobile | [`wallet-examples/src/hana.ts`](./wallet-examples/src/hana.ts) | ✅ |
| **LOBSTR** | Signer extension (desktop), mobile via WalletConnect | [`wallet-examples/src/lobstr.ts`](./wallet-examples/src/lobstr.ts) | ✅ |
| **Ledger** | Hardware wallet over WebUSB | [`wallet-examples/src/ledger.ts`](./wallet-examples/src/ledger.ts) | ✅ |
| *Stellar Wallets Kit* | One library covering all of the above and more | [`wallet-examples/src/wallets-kit.ts`](./wallet-examples/src/wallets-kit.ts) | — |

## Contents

1. [Design: One Adapter Interface](#1-design-one-adapter-interface)
2. [Hana](#2-hana)
3. [LOBSTR](#3-lobstr)
4. [Ledger](#4-ledger)
5. [Supporting Many Wallets at Once: Stellar Wallets Kit](#5-supporting-many-wallets-at-once-stellar-wallets-kit)
6. [Comparison Guide](#6-comparison-guide)
7. [Testing](#7-testing)
8. [External Resources](#8-external-resources)

---

## 1. Design: One Adapter Interface

Every example implements the same small interface ([`types.ts`](./wallet-examples/src/types.ts)), so the dApp's connect → build → sign → submit logic does not change from one wallet to the next:

```ts
export interface WalletAdapter {
  readonly id: string;
  readonly name: string;
  isAvailable(): Promise<boolean>;
  connect(): Promise<string>;                     // returns a G... address
  signTransaction(xdr: string, opts: SignOptions): Promise<string>;
  disconnect(): Promise<void>;
}
```

Every adapter throws one of two errors so your UI can handle them the same way:

- `WalletUnavailableError` means the wallet is not installed or the device is not connected. Show an install or connect hint.
- `WalletRejectedError` means the user dismissed or rejected the prompt. Do not retry automatically.

[`invoke.ts`](./wallet-examples/src/invoke.ts) shows the full Soroban flow with any adapter:

```ts
import { hanaAdapter, invokeContract } from "./wallet-examples/src";
import { nativeToScVal } from "@stellar/stellar-sdk";

const address = await hanaAdapter.connect();
const result = await invokeContract(
  hanaAdapter,            // swap for lobstrAdapter, ledgerAdapter, ...
  address,
  "TESTNET",
  "CC...CONTRACT_ID",
  "hello",
  [nativeToScVal("Dev", { type: "symbol" })],
);
console.log("Return value:", result.returnValue);
```

`invokeContract` builds the transaction and calls `prepareTransaction`, which simulates it and adds the footprint, resource fees, and auth entries. It then asks the wallet to sign, submits the transaction, and polls until it succeeds or fails.

---

## 2. Hana

[Hana](https://hana.network) is a multichain wallet with a browser extension and a mobile app, and it supports Stellar and Soroban. It injects a provider at `window.hanaWallet.stellar`, so no npm package is needed.

```ts
// docs/wallet-examples/src/hana.ts (excerpt)
const address = await window.hanaWallet!.stellar!.getPublicKey();

const signedXdr = await window.hanaWallet!.stellar!.signTransaction({
  xdr,
  accountToSign: address,
  networkPassphrase: Networks.TESTNET,
});
```

Notes:

- **Detecting the wallet:** check for `window.hanaWallet?.stellar`. Extensions inject after the page starts loading, so check again on `window.load` before showing "not installed".
- **Network:** pass `networkPassphrase` on every sign request. Hana uses it to show the right network to the user.
- **Sessions:** Hana keeps no dApp session to tear down. Disconnecting means clearing your app state.

---

## 3. LOBSTR

[LOBSTR](https://lobstr.co) is the most widely used retail Stellar wallet. Desktop dApps connect through the **LOBSTR Signer Extension** and its SDK. Mobile users connect over **WalletConnect**, which Stellar Wallets Kit handles (see [§5](#5-supporting-many-wallets-at-once-stellar-wallets-kit)).

```bash
npm install @lobstrco/signer-extension-api
```

```ts
// docs/wallet-examples/src/lobstr.ts (excerpt)
import { isConnected, getPublicKey, signTransaction } from "@lobstrco/signer-extension-api";

if (!(await isConnected())) throw new Error("Install the LOBSTR Signer Extension");
const address = await getPublicKey();
const signedXdr = await signTransaction(xdr);   // empty string = user rejected
```

Notes:

- **Account and network:** the extension signs for the account the user picked and takes no network argument. The passphrase is already part of the transaction hash, so make sure you build the transaction for the network the user's account is on.
- **Rejections:** an empty return value means the user rejected the request. The adapter turns this into `WalletRejectedError`.
- **Soroban support:** LOBSTR targets retail users. Test any contract call with complex auth on testnet before you offer it to LOBSTR users.

---

## 4. Ledger

Ledger hardware wallets keep the key on the device and show the transaction on its screen for the user to approve. The browser talks to the device over **WebUSB**, which works in Chrome, Edge, and Brave but not in Firefox or Safari.

```bash
npm install @ledgerhq/hw-transport-webusb @ledgerhq/hw-app-str @stellar/stellar-sdk
```

```ts
// docs/wallet-examples/src/ledger.ts (excerpt)
const transport = await TransportWebUSB.create();      // must run inside a user click handler
const str = new Str(transport);

const { rawPublicKey } = await str.getPublicKey("44'/148'/0'");
const address = StrKey.encodeEd25519PublicKey(rawPublicKey);

const tx = TransactionBuilder.fromXDR(xdr, networkPassphrase);
const { signature } = await str.signTransaction("44'/148'/0'", tx.signatureBase());
tx.addSignature(address, signature.toString("base64"));
```

Notes:

- **Device requirements:** the **Stellar app** must be open on the device. Soroban `InvokeHostFunction` operations need Stellar app **v5.0 or newer**. Older versions reject the transaction.
- **Derivation path:** `44'/148'/N'` (SEP-0005). Use `createLedgerAdapter("44'/148'/1'")` for the second account.
- **User gesture:** `TransportWebUSB.create()` opens the browser's device picker, so it must be called from a click handler.
- **Rejections:** pressing *Reject* on the device returns status word `0x6985`, which the adapter maps to `WalletRejectedError`.
- **Signature check:** `addSignature` checks the signature against the transaction hash before attaching it, so a wrong network passphrase fails here rather than on chain.

---

## 5. Supporting Many Wallets at Once: Stellar Wallets Kit

If you want to support more than two or three wallets, use [Stellar Wallets Kit](https://github.com/Creit-Tech/Stellar-Wallets-Kit) rather than writing an adapter for each one. It includes modules for Freighter, xBull, Albedo, Rabet, LOBSTR, Hana, Hot Wallet, Ledger, Trezor, and WalletConnect, along with a wallet-selection modal.

```bash
npm install @creit.tech/stellar-wallets-kit
```

```ts
// docs/wallet-examples/src/wallets-kit.ts (excerpt)
const kit = new StellarWalletsKit({
  network: WalletNetwork.TESTNET,
  selectedWalletId: FREIGHTER_ID,
  modules: allowAllModules(),
});

await kit.openModal({
  onWalletSelected: async (option) => {
    kit.setWallet(option.id);
    const { address } = await kit.getAddress();
  },
});

const { signedTxXdr } = await kit.signTransaction(xdr, { address, networkPassphrase });
```

**Use the kit when** you want broad coverage and a ready-made picker. **Write your own adapters when** you need a fully custom UI, want fewer dependencies, or support only one or two wallets. The adapter interface on this page lets you mix the two approaches.

---

## 6. Comparison Guide

### Feature matrix

| | Freighter | xBull | Albedo | **Hana** | **LOBSTR** | **Ledger** |
|---|---|---|---|---|---|---|
| **Form factor** | Extension | Extension, mobile, PWA | Web (no install) | Extension, mobile | Signer extension, mobile (WalletConnect) | Hardware (WebUSB) |
| **Integration** | `@stellar/freighter-api` | `@creit.tech/xbull-wallet-connect` | `@albedo-link/intent` | Injected `window.hanaWallet` | `@lobstrco/signer-extension-api` | `@ledgerhq/hw-app-str` |
| **Soroban invoke signing** | Full | Full | Partial | Yes | Basic (test complex auth first) | Yes (Stellar app ≥ 5) |
| **Auth-entry signing** | Yes | Yes | No | Yes | No | Yes (`signSorobanAuthorization`) |
| **Testnet** | Yes | Yes | Yes | Yes | Limited | Yes |
| **Multichain** | No | No | No | Yes | No | Yes |
| **Key custody** | Browser | Browser / device | Browser (web) | Browser / device | Device | Secure element |
| **Best for** | Developers, default choice | Power users | Users who will not install anything | Users who hold other chains | Retail users | High-value accounts, admins |

### Choosing which wallets to support

| Your app | Support at minimum |
|---|---|
| Developer tool, testnet demo | Freighter |
| Consumer dApp (payments, NFTs) | Freighter + LOBSTR + one mobile option (xBull or Hana), or just use Stellar Wallets Kit |
| DeFi / high-value | Freighter + xBull + **Ledger** |
| DAO / admin console | **Ledger** (hardware keys for admin accounts) + Freighter |
| Multichain product | **Hana** + Freighter |

### UX differences to plan for

- **Connect latency:** injected wallets respond immediately. Ledger needs the device plugged in, unlocked, and the Stellar app open.
- **Prompt content:** hardware wallets show raw operation details on a small screen. Tell users in your UI what they should expect to see on the device.
- **Rejection signals vary:** some wallets throw, LOBSTR returns an empty string, and Ledger returns a status word. The adapters convert all of these to `WalletRejectedError`.
- **Network mismatch** is the most common support issue. Always build and sign with the same `networkPassphrase`, and show which network is active in your UI.

---

## 7. Testing

### Unit tests (no wallet needed)

[`wallet-examples/test/adapters.test.ts`](./wallet-examples/test/adapters.test.ts) tests each adapter against fakes:

- **Hana:** a stub provider on `window.hanaWallet`. Covers missing-extension, connect, sign-argument forwarding, and rejection mapping.
- **LOBSTR:** `vi.mock` of the signer SDK. Covers not-connected, connect + sign, and empty signature treated as a rejection.
- **Ledger:** mocked transport and Stellar app. The fake device signs with a real keypair, so the test checks that the signature attached to the envelope **verifies against the transaction hash**. Also covers the `0x6985` rejection.

```bash
cd docs/wallet-examples
npm install
npm test          # vitest
npm run typecheck # tsc --noEmit
```

### Manual testnet checklist (per wallet)

- [ ] **Not installed:** the UI shows an install or connect hint and does not crash.
- [ ] **Connect:** the address shown matches the wallet.
- [ ] **Reject connect:** you get `WalletRejectedError` and the UI recovers.
- [ ] **Sign and submit** a simple contract call (for example, `hello` on [`examples/basics/01-hello-world`](../examples/basics/01-hello-world/) or a SEP-41 `transfer`). The transaction succeeds and shows on [Stellar Expert (testnet)](https://stellar.expert/explorer/testnet).
- [ ] **Reject signing:** you get `WalletRejectedError`, and nothing is submitted.
- [ ] **Wrong network:** with the wallet on mainnet and the app on testnet, the app detects and reports the mismatch.
- [ ] **Disconnect and reconnect** works without a page reload.
- [ ] **Ledger only:** the device shows the contract ID and function name, and the flow works with the device locked at the start.

For mobile flows, see the mobile section of the [Wallet Ecosystem Survey](../book/src/docs/wallet-ecosystem.md) and the [Mobile Testing Guide](../book/src/guides/mobile-testing.md).

---

## 8. External Resources

- [Stellar Wallets Kit](https://github.com/Creit-Tech/Stellar-Wallets-Kit)
- [Hana Wallet](https://hana.network)
- [LOBSTR Signer Extension](https://lobstr.co/signer-extension/) · [`@lobstrco/signer-extension-api`](https://www.npmjs.com/package/@lobstrco/signer-extension-api)
- [Ledger Stellar app](https://support.ledger.com/hc/en-us/articles/115003797194) · [`@ledgerhq/hw-app-str`](https://www.npmjs.com/package/@ledgerhq/hw-app-str)
- [SEP-0005: Key derivation](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0005.md)
- [Stellar docs: Wallets](https://developers.stellar.org/docs/tools/developer-tools/wallets)
- In this repo: [Wallet Integration Guide](./wallet-integration.md) · [Wallet Ecosystem Survey](../book/src/docs/wallet-ecosystem.md)

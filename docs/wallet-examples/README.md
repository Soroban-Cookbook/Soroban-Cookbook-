# Wallet Adapter Examples

TypeScript examples for connecting Soroban dApps to **Hana**, **LOBSTR**, and **Ledger**, and to many more wallets at once through **Stellar Wallets Kit**. Every adapter implements the same `WalletAdapter` interface, so `invokeContract()` works with any of them.

For the full guide, the comparison matrix, and the testing checklist, see [`../additional-wallets.md`](../additional-wallets.md).

```
src/
  types.ts         WalletAdapter interface + error types
  hana.ts          Hana (injected window.hanaWallet.stellar)
  lobstr.ts        LOBSTR Signer Extension
  ledger.ts        Ledger over WebUSB (Stellar app)
  wallets-kit.ts   Stellar Wallets Kit (multi-wallet modal)
  invoke.ts        build → prepare → sign → submit → poll
test/
  adapters.test.ts vitest unit tests with mocked wallets
```

```bash
npm install
npm test
npm run typecheck
```

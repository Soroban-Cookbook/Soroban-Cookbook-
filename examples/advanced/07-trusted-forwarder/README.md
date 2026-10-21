# Trusted Forwarder

This example demonstrates the Trusted Forwarder pattern for meta-transactions on Soroban, inspired by [ERC-2771](https://eips.ethereum.org/EIPS/eip-2771).

## What it shows

- **Forwarder contract** — validates off-chain signed meta-transactions and forwards calls to target contracts
- **Target validation** — a recipient contract verifies that the caller is the trusted forwarder using `require_auth()`
- **Fee handling** — the forwarder deducts a configurable fee from the sender and credits it to the relayer
- **Nonce tracking** — per-sender monotonic nonces prevent replay attacks
- **Ed25519 signatures** — off-chain keypair authorizes meta-transactions
- **Cross-contract invocation** — `env.invoke_contract()` forwards the original sender and data to the recipient

## Contracts

### TrustedForwarder

The forwarder holds user balances (deposited via `fund`), authorizes off-chain signers via `register_signer`, and processes meta-transactions through `forward`. The forwarder calls the recipient's `forwarded_call(sender, data)` function.

### SimpleRecipient

An example target contract that only accepts forwarded calls from a trusted forwarder. It stores the last sender and data for verification.

## Use cases

- **Gas abstraction** — users sign meta-transactions; relayers pay gas fees and recover costs via fees
- **Batch relaying** — a relayer collects multiple meta-transactions and submits them in one Soroban transaction
- **Application-specific forwarders** — dApps deploy their own forwarder with custom fee models and trusted signers

## Security notes

- Nonces must be strictly monotonic per sender
- The forwarder should be permissioned and monitored
- Recipients must verify the forwarder's identity (via `require_auth()`) before trusting the `sender` parameter
- In production, consider using a dedicated wallet or account abstraction for key management

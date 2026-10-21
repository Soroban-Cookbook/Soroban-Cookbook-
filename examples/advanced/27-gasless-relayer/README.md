# Gasless Relayer

This example demonstrates a simple relayer pattern for Soroban contracts.

## What it shows

- Meta-transaction payloads with a sender, recipient, amount, nonce, and deadline.
- Signature verification using an off-chain ed25519 keypair and a contract-stored public key.
- Nonce tracking to prevent replay and duplicate execution.
- A trusted relayer gate to limit who may submit meta-transactions.

## Security notes

- The relayer should be permissioned and monitored.
- Nonces must be strictly monotonic per sender.
- The contract should avoid trusting raw caller-supplied data without verification.
- In production, prefer a dedicated wallet or account abstraction approach for key management.

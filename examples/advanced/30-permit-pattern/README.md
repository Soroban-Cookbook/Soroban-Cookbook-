# Permit Pattern

This advanced example demonstrates an EIP-2612-style permit flow in Soroban. Instead of requiring the token owner to submit a separate `approve` transaction, the owner signs an authorization envelope off-chain and the spender or a relayer submits it on-chain.

## What it demonstrates

- Off-chain signature-based approval using Soroban auth
- Deadline enforcement with `expiration_ledger`
- Delegated spending via `permit` + `transfer_from`
- Permit expiration and revocation semantics

## Contract workflow

1. The owner calls `permit(owner, spender, amount, expiration_ledger)`.
2. The contract binds the authorization to the exact arguments using `require_auth_for_args`.
3. The spender uses `transfer_from` to move funds up to the approved allowance.
4. If the permit is expired or the allowance is depleted, the transfer fails.

## Example

```rust
client.permit(&owner, &spender, &500, &100_000).unwrap();
client.transfer_from(&spender, &owner, &recipient, &300).unwrap();
```

## Testing

```bash
cargo test -p permit-pattern
```

# 11 · Authorized Token

**Source:** [`examples/tokens/11-authorized-token/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/11-authorized-token)

A SEP-41 token with the compliance-style `authorized` flag: an admin can freeze or unfreeze any account, and transfers, approvals, and burns refuse to move funds for unauthorized parties. This mirrors the authorization model of native Stellar assets.

## What You'll Learn

- Implementing `set_authorized(admin, address, authorized)` — the SEP-41 `authorized` flag used by regulated assets
- Enforcing authorization on `transfer`, `transfer_from`, `approve`, and `burn` with a typed `NotAuthorized` error
- Emitting an `authorize` event on every authorization change so compliance systems can audit holds and releases
- How the contract-level flag maps to the native-asset issuer `authorized` bit (`set_trust_line_flags`)

## Quick Code

```rust
// Freeze an account (admin only)
client.set_authorized(&admin, &suspicious_addr, &false);

// Transfers from the frozen account now fail
assert_eq!(
    client.try_transfer(&suspicious_addr, &recipient, &100),
    Err(Ok(TokenError::NotAuthorized))
);

// Unfreeze restores normal operation
client.set_authorized(&admin, &suspicious_addr, &true);
```

## Key Functions

| Function | Description |
|-----------|-------------|
| `set_authorized(admin, address, authorized)` | Admin-only freeze/unfreeze; emits an `authorize` event |
| `authorized(address)` | Query authorization status; accounts default to `true` |
| `transfer(from, to, amount)` | Rejects a frozen sender |
| `transfer_from(spender, owner, to, amount)` | Rejects a frozen spender or frozen owner |
| `approve(owner, spender, amount)` | Rejects a frozen owner |
| `burn(owner, amount)` | Rejects a frozen owner |

## Design Notes

- Accounts are **authorized by default**, mirroring native assets, so a fresh deployment behaves like a regular token until the admin explicitly freezes an address.
- Freezing prevents movement, not ownership: balances and total supply are untouched, and unfreezing restores full functionality.
- On `transfer_from`, both the spender (the party exercising authorization) and the owner are checked, so a frozen account cannot be drained through a pre-existing allowance.
- Every authorization change emits an `authorize` event — keep your off-chain compliance pipeline subscribed to it.

## Compliance / Native Asset Notes

Native Stellar assets let the issuer clear the `authorized` flag on a trustline (`set_trust_line_flags`) to freeze an account. Contract tokens that carry the same regulatory obligations should implement the SEP-41 equivalent shown here. If your token wraps a native asset, keep both layers aligned and reconcile the Stellar operation and the contract `authorize` event before treating an address as frozen end-to-end.

## Run the Example

```bash
cd examples/tokens/11-authorized-token
cargo test
```

## Frontend Testing

To test this contract from TypeScript, generate bindings with `soroban-spec-typescript` and mock the RPC layer — see [Frontend / TypeScript Testing](../../docs/testing-best-practices.md#frontend--typescript-testing) for a worked example using this token's `authorized` interface.

## See Also

- [01 · SEP-41 Token](./01-sep41-token.md)
- [03 · Optimized Operations](./03-optimized-operations.md)
- [Token Security Checklist](../../docs/token-security-checklist.md)
- [Token Overview](../tokens.md)

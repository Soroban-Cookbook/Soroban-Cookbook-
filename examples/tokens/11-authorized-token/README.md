# Authorized Token (SEP-41 Compliance / Freeze)

A SEP-41 compliant fungible token that adds the compliance-style `authorized` flag, mirroring the authorization model of native Stellar assets: a designated admin (acting as the asset's issuing authority) can freeze or unfreeze any account, and transfers refuse to move funds for unauthorized parties.

## Features

- **SEP-41 Standard**: `initialize`, `transfer`, `transfer_from`, `approve`, `balance`, `allowance`, `total_supply`, `name`, `symbol`, `decimals`, `admin`, `mint`, `burn`
- **Compliance / Freeze**: `set_authorized(admin, address, authorized)` and `authorized(address)` — the SEP-41 `authorized` flag used by regulated assets
- **Authorization Checks**: `transfer`, `transfer_from`, `approve`, and `burn` all reject unauthorized (frozen) parties with a dedicated `NotAuthorized` error
- **Event Emission**: Transfer, approval, and authorization events for off-chain indexing and compliance monitoring

## Why an `authorized` flag?

Native Stellar assets carry an authorization bit that the asset issuer toggles to comply with regulations: an unauthorized account cannot receive or send the asset. Real-world issuers use this to implement freeze/hold functionality required by securities and stablecoin compliance programs.

When you issue a contract-based (Soroban) token that must support the same obligations, the SEP-41 `authorized` flag is the contract-side equivalent:

- **Freeze**: `set_authorized(admin, addr, false)` blocks the address from transacting — the contract-level analogue of the issuer clearing the authorization bit on a native asset trustline.
- **Unfreeze**: `set_authorized(admin, addr, true)` restores normal operation.
- **Auditability**: every authorization change emits an `authorize` event so compliance tooling can track holds and releases, just as Stellar's accounts/operations endpoints expose native asset authorization changes.
- **Defaults**: accounts are authorized by default (mirroring native assets), so an ordinary deployment behaves like a regular token until the admin explicitly freezes someone.

## Key Functions

| Function | Description |
| --- | --- |
| `initialize(admin, name, symbol, decimals, initial_supply)` | Initialize with admin, metadata, and initial supply |
| `set_authorized(admin, address, authorized)` | Admin-only freeze/unfreeze; emits an `authorize` event |
| `authorized(address)` | Query the authorization status (defaults to `true`) |
| `transfer(from, to, amount)` | Standard SEP-41 transfer; requires the sender to be authorized |
| `transfer_from(spender, owner, to, amount)` | SEP-41 transfer_from; requires both spender and owner to be authorized |
| `approve(owner, spender, amount)` | Standard SEP-41 approve; requires the owner to be authorized |
| `mint(admin, to, amount)` | Admin-only minting |
| `burn(owner, amount)` | Burn from the caller's balance; requires the owner to be authorized |

## Design Decisions

- **Who is checked on `transfer_from`?** The spender is the party exercising authorization (it calls `require_auth`), so it must be authorized. The owner is checked too, so a frozen account cannot have its balance drained through a pre-existing allowance.
- **Balances are preserved on freeze.** Freezing prevents movement, not ownership: balances and total supply are untouched, and a later unfreeze restores full functionality.
- **Explicit error, not a panic.** Unauthorized parties surface a typed `NotAuthorized` error so integrators can distinguish compliance blocks from balance failures.
- **Authorization checks live on the hot paths** (`transfer`, `transfer_from`, `approve`, `burn`). Add them to `mint`/`clawback` as well if your regulatory program requires controlling newly issued funds.

## Native Asset Compliance Integration

If your token wraps or mirrors a native Stellar asset, keep both authorization layers aligned:

1. The issuer's native-asset `authorized` flag controls trustline-level movement.
2. The contract's `set_authorized` flag controls contract-level movement.
3. Listen for both the Stellar operation (`set_trust_line_flags`) and the contract's `authorize` event in your compliance pipeline, and reconcile them before treating an address as (un)frozen end-to-end.

For regulated deployments, consider pairing this example with [09-optimized-token-ops](../09-optimized-token-ops/) for gas-efficient checks and the [Token Security Checklist](../../../book/src/docs/token-security-checklist.md).

## Usage

```bash
# Build
cargo build --target wasm32-unknown-unknown --release -p authorized-token

# Test
cargo test -p authorized-token
```

## Quick Code

```rust
// Freeze an account (admin only)
client.set_authorized(&admin, &suspicious_addr, &false);
assert_eq!(client.authorized(&suspicious_addr), false);

// Any transfer from the frozen account now fails
assert_eq!(
    client.try_transfer(&suspicious_addr, &recipient, &100),
    Err(Ok(TokenError::NotAuthorized))
);

// Unfreeze restores normal operation
client.set_authorized(&admin, &suspicious_addr, &true);
client.transfer(&suspicious_addr, &recipient, &100);
```

## Related

- [01-sep41-token](../01-sep41-token/): the minimal SEP-41 base this example extends
- [03-pausable-token](../03-pausable-token/): global pause (freeze everyone at once) versus per-address authorization

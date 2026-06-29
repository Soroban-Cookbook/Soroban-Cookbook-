# 01 · Basic NFT

A complete, minimal Non-Fungible Token contract for Soroban demonstrating the full ownership lifecycle: minting, transfer, operator approval, and enumeration.

This is the recommended starting point for NFT development on Soroban before adding metadata or marketplace logic.

---

## What This Example Covers

| Feature | Implementation |
|---------|---------------|
| Admin-gated minting | `mint` checks caller == stored admin via `require_auth()` |
| Ownership tracking | `DataKey::Owner(token_id)` → `Address` in `Persistent` storage |
| Per-owner token count | `DataKey::Balance(address)` → `u32` |
| Global enumeration | `DataKey::TokenByIndex(index)` for iterating all tokens |
| Per-owner enumeration | `DataKey::OwnedToken(owner, index)` swap-remove pattern |
| Single-token approval | `approve(owner, spender, token_id)` — cleared on transfer |
| Operator approval | `set_approval_for_all(owner, operator, bool)` |
| `transfer_from` | Checks single-token or operator approval before moving |
| Typed errors | `NftError` enum with `#[contracterror]` |
| Structured events | `mint`, `transfer`, `approve` all emit `(action, "nft")` topics |

---

## Key Patterns from Basics

### Authentication (`03-authentication` pattern)

Every state-changing function calls `require_auth()` on the acting address:

```rust
// Minting — only the stored admin may mint
pub fn mint(env: Env, admin: Address, to: Address, token_id: u32) -> Result<(), NftError> {
    admin.require_auth();
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin)
        .ok_or(NftError::NotInitialized)?;
    if stored_admin != admin {
        return Err(NftError::NotAdmin);
    }
    // ...
}

// Transfer — current owner must authenticate
pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) -> Result<(), NftError> {
    from.require_auth();
    // ...
}
```

### Event Emission (`04-events` pattern)

Events use a `(action, namespace)` topic pair so indexers can filter by event type:

```rust
// Mint event
env.events().publish(
    (symbol_short!("mint"), symbol_short!("nft")),
    (to, token_id),
);

// Transfer event
env.events().publish(
    (symbol_short!("transfer"), symbol_short!("nft")),
    (from, to, token_id),
);

// Approval event
env.events().publish(
    (symbol_short!("approve"), symbol_short!("nft")),
    (owner, approved, token_id),
);
```

### Storage Tier Selection

| Key | Tier | Rationale |
|-----|------|-----------|
| `Admin`, `Name`, `Symbol`, `TotalSupply` | `Instance` | Contract-lifetime metadata; cheap to read as a bundle |
| `Owner(token_id)`, `Balance(addr)` | `Persistent` | Must survive ledger entry expiry; TTL-extended on access |
| `Approved(token_id)`, `ApproveAll(owner, op)` | `Persistent` | Approvals must outlive the transaction that set them |

### Enumeration via Swap-Remove

The owner's token list uses a swap-remove strategy to keep the list dense without gaps:

```
Before remove(token B):  [A, B, C]  indices 0,1,2
After swap-remove:        [A, C]     (C moved to index 1)
```

This avoids scanning for holes at query time and keeps every removal O(1).

---

## Contract Interface

```rust
fn initialize(env, admin: Address, name: String, symbol: String) -> Result<(), NftError>
fn mint(env, admin: Address, to: Address, token_id: u32)          -> Result<(), NftError>
fn transfer(env, from: Address, to: Address, token_id: u32)       -> Result<(), NftError>
fn transfer_from(env, spender, from, to, token_id: u32)           -> Result<(), NftError>
fn approve(env, owner, approved: Address, token_id: u32)          -> Result<(), NftError>
fn set_approval_for_all(env, owner, operator: Address, bool)      -> Result<(), NftError>
fn get_approved(env, token_id: u32)                               -> Option<Address>
fn is_approved_for_all(env, owner, operator: Address)             -> bool
fn owner_of(env, token_id: u32)                                   -> Result<Address, NftError>
fn balance_of(env, owner: Address)                                -> u32
fn total_supply(env)                                              -> u32
fn token_by_index(env, index: u32)                                -> Result<u32, NftError>
fn tokens_of_owner(env, owner: Address)                           -> Vec<u32>
fn name(env)                                                      -> Result<String, NftError>
fn symbol(env)                                                    -> Result<String, NftError>
```

---

## How to Run

```bash
cd examples/nfts/01-basic-nft

# Run all tests
cargo test

# Build the WASM contract
cargo build --target wasm32-unknown-unknown --release
```

---

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Contract implementation — types, storage keys, all entry points |
| `src/test.rs` | Unit tests covering mint, transfer, approvals, and error cases |
| `Cargo.toml` | Crate metadata (`basic-nft`) and workspace dependency references |

---

## Next Steps

- **[02-nft-metadata](../02-nft-metadata/)** — Add a `token_uri` / base URI pattern and an on-chain `NFTMetadata` struct
- **[03-nft-metadata-standards](../03-nft-metadata-standards/)** — Full JSON-schema-compliant metadata with attribute system, URI validation, and approval expiry
- **[04-nft-marketplace](../04-nft-marketplace/)** — Fixed-price listings, auctions, bundles, and royalties

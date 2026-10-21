# 03 · NFT Metadata Standards

**Source:** [`examples/nfts/03-nft-metadata-standards/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/03-nft-metadata-standards)

A JSON-schema-compliant metadata standard. Every `TokenMetadata` record is validated on mint and update against a strict ruleset covering required fields, URI schemes, hex color codes, and non-empty attribute values.

## What You'll Learn

- Enforcing metadata schema compliance inside a Soroban contract
- A typed `Vec<Attribute>` system with `trait_type` + `value` fields
- URI scheme validation (`ipfs://`, `https://`, `http://`, `data:`)
- Approval expiry using `expiration_ledger`

## Validation Rules

| Field | Rule |
|-------|------|
| `name`, `description` | Non-empty |
| `image` | Non-empty; must start with `ipfs://`, `https://`, `http://`, or `data:` |
| `background_color` | If set: exactly 6 ASCII hex digits |
| `attributes[*].trait_type` | Non-empty |
| `attributes[*].value` | Non-empty |

## Quick Code

```rust
let meta = TokenMetadata {
    name: String::from_str(&env, "Sword #42"),
    description: String::from_str(&env, "A legendary weapon"),
    image: String::from_str(&env, "ipfs://QmHash/sword.png"),
    // ...
};
client.mint(&admin, &alice, &42u32, &meta);
```

## Run the Example

```bash
cd examples/nfts/03-nft-metadata-standards
cargo test
```

## Next: [04 · NFT Marketplace](./04-nft-marketplace.md)

# NFT Examples

Non-Fungible Token (NFT) implementations and patterns on Soroban.

## Examples

### Basic NFTs

- **Simple NFT** - Basic NFT with minting and transfers
- **NFT with Metadata** - NFTs with rich metadata
- **Enumerable NFT** - NFT with iteration capabilities

### NFT Marketplaces

- **Simple Marketplace** - Buy and sell NFTs
- **Auction Contract** - English and Dutch auctions
- **Offer System** - Peer-to-peer NFT trading

### Advanced Patterns

- **Composable NFTs** - NFTs that contain other NFTs
- **Fractionalized NFTs** - Split NFT ownership
- **Dynamic NFTs** - NFTs that change over time
- **Soulbound Tokens** - Non-transferable NFTs

### Gaming & Collectibles

- **Achievement System** - Mint NFTs for accomplishments
- **Breeding Contract** - Combine NFTs to create new ones
- **Staking NFTs** - Stake NFTs to earn rewards

## 🎨 NFT Standards

While Soroban doesn't have a single NFT standard yet, these examples follow best practices:

### Core Functions

```rust
// Minting
pub fn mint(env: Env, to: Address, token_id: u32) -> Result<(), Error>

// Transfer
pub fn transfer(env: Env, from: Address, to: Address, token_id: u32)

// Ownership
pub fn owner_of(env: Env, token_id: u32) -> Address

// Balance
pub fn balance_of(env: Env, owner: Address) -> u32
```

### Metadata

```rust
pub fn token_uri(env: Env, token_id: u32) -> String
pub fn name(env: Env) -> String
pub fn symbol(env: Env) -> String
```

## 📝 Metadata Best Practices

### On-Chain vs Off-Chain

**On-Chain Metadata** (stored in contract):

- ✅ Immutable and verifiable
- ✅ No external dependencies
- ❌ Higher storage costs
- ❌ Limited data size

**Off-Chain Metadata** (IPFS or other storage):

- ✅ Lower costs
- ✅ Rich media support
- ❌ Requires external service
- ❌ Potential availability issues

### Recommended Metadata Format

```json
{
  "name": "My NFT #1",
  "description": "A unique digital collectible",
  "image": "ipfs://QmHash/image.png",
  "attributes": [
    {
      "trait_type": "Rarity",
      "value": "Legendary"
    },
    {
      "trait_type": "Power",
      "value": 100
    }
  ]
}
```

## 🧪 Testing NFT Contracts

```rust
#[test]
fn test_mint_and_ownership() {
    let env = Env::default();
    let contract_id = env.register_contract(None, NFTContract);
    let client = NFTContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let token_id = 1;

    client.mint(&owner, &token_id);

    assert_eq!(client.owner_of(&token_id), owner);
    assert_eq!(client.balance_of(&owner), 1);
}

#[test]
fn test_transfer() {
    // Test NFT transfers
}

#[test]
fn test_authorization() {
    // Test only owner can transfer
}
```

## 🎯 Use Cases

### Digital Art

- Unique artwork with provenance
- Limited edition collections
- Generative art projects

### Gaming

- In-game items and characters
- Achievement badges
- Virtual land parcels

### Identity

- Digital credentials
- Membership tokens
- Certification badges

### Real-World Assets

- Property deeds
- Event tickets
- Supply chain tracking

## 🔒 Security Considerations

1. **Prevent Unauthorized Minting**

   ```rust
   // Only admin can mint
   admin.require_auth();
   ```

2. **Safe Transfers**

   ```rust
   // Verify ownership before transfer
   let current_owner = get_owner(&token_id);
   current_owner.require_auth();
   ```

3. **Prevent Double Minting**

   ```rust
   // Check token doesn't exist
   if has_token(&token_id) {
       panic!("Token already exists");
   }
   ```

4. **Metadata Immutability**
   - Consider making metadata immutable after minting
   - Or implement clear rules for updates

## 📚 Additional Resources

- [NFT Best Practices](https://developers.stellar.org/docs/smart-contracts/nfts)
- [IPFS Integration](https://docs.ipfs.tech/)
- [Metadata Standards](https://docs.opensea.io/docs/metadata-standards)

## 🤝 Contributing

Have an NFT pattern to share? See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

**Build the next generation of digital collectibles on Stellar!**

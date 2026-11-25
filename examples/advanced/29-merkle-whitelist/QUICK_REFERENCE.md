# Quick Reference: Merkle Whitelist Contract

## 🚀 Quick Start

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Test
cargo test

# Benchmark
cargo test --release -- --nocapture benchmark

# Deploy
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/merkle_whitelist.wasm
```

## 📋 Common Operations

### Initialize Contract
```rust
client.initialize(
    &admin_address,
    &fee_token_address,
    &1000000,  // registration fee
    &merkle_root
);
```

### Verify Whitelist
```rust
client.verify_whitelist(
    &user_address,
    &proof_vector,
    &metadata_bytes
);
```

### Check Status
```rust
let is_listed = client.is_whitelisted(&address);
```

### Propose Root Update
```rust
let proposal_id = client.propose_root_update(
    &governor_address,
    &new_root,
    &description
);
```

### Vote on Proposal
```rust
client.vote_on_proposal(
    &governor_address,
    &proposal_id,
    &true  // support
);
```

### Execute Proposal
```rust
client.execute_proposal(
    &executor_address,
    &proposal_id
);
```

## 🔐 Roles

| Role | Permissions |
|------|-------------|
| **Admin** | Grant/revoke roles, pause contract, manage fees, blacklist |
| **Governor** | Propose root updates, vote on proposals |
| **Validator** | Submit disputes, vote on disputes |

## 💰 Fee Structure

| Operation | Default Fee |
|-----------|-------------|
| Registration | 1,000,000 (configurable) |
| Dispute | 100,000 (10% of registration) |
| Fee Waiver | Admin can grant |

## ⏱️ Governance Timing

| Parameter | Default Value |
|-----------|--------------|
| Proposal Duration | 3 days (259,200 seconds) |
| Timelock Duration | 1 day (86,400 seconds) |
| Dispute Period | 2 days (172,800 seconds) |
| Quorum | 3 votes |

## 🛡️ Security Parameters

| Feature | Configuration |
|---------|--------------|
| Rate Limit | 10 requests per hour per address |
| Max Signers | 20 |
| Proof Depth | Optimal: 16-20 levels |
| Replay Protection | Nonce-based |

## ⚡ Performance

| Tree Size | Depth | Verification Cost |
|-----------|-------|-------------------|
| 16 | 4 | ~3,000 CPU |
| 256 | 8 | ~5,000 CPU |
| 65,536 | 16 | ~9,000 CPU |
| 1,048,576 | 20 | ~11,000 CPU |
| 16,777,216 | 24 | ~13,000 CPU |

## 🔧 Configuration Functions

### Update Registration Fee
```rust
client.set_registration_fee(&admin, &new_fee);
```

### Grant Fee Waiver
```rust
client.grant_fee_waiver(&admin, &user_address);
```

### Add Governor Role
```rust
client.add_role(&admin, &user_address, &Role::Governor);
```

### Add Validator Role
```rust
client.add_role(&admin, &user_address, &Role::Validator);
```

### Update Governance Config
```rust
client.update_governance_config(&admin, &new_config);
```

## 🚨 Emergency Functions

### Pause Contract
```rust
client.pause(&admin);
```

### Unpause Contract
```rust
client.unpause(&admin);
```

### Blacklist Address
```rust
client.add_to_blacklist(&admin, &malicious_address);
```

### Revoke Entry
```rust
client.revoke_entry(&admin, &target_address);
```

## 📊 Query Functions

### Get Merkle Root
```rust
let root = client.get_merkle_root();
```

### Get Root Version
```rust
let version = client.get_root_version();
```

### Get Entry Details
```rust
let entry = client.get_entry(&address);
// Returns: verified, registered_at, metadata, root_version, dispute_count
```

### Get Proposal Details
```rust
let proposal = client.get_proposal(&proposal_id);
```

### Get Dispute Details
```rust
let dispute = client.get_dispute(&dispute_id);
```

### Check Role
```rust
let has_role = client.has_role(&address, &Role::Governor);
```

### Get Accumulated Fees
```rust
let fees = client.get_accumulated_fees();
```

## 🔨 Dispute Management

### Submit Dispute
```rust
let dispute_id = client.submit_dispute(
    &validator_address,
    &target_address,
    &evidence_bytes
);
```

### Vote on Dispute
```rust
client.vote_on_dispute(
    &validator_address,
    &dispute_id,
    &DisputeDecision::Invalid  // or Valid
);
```

### Resolve Dispute
```rust
client.resolve_dispute(&resolver_address, &dispute_id);
```

## 📝 Off-Chain Merkle Tree Generation

### Rust Example
```rust
use sha2::{Sha256, Digest};

fn compute_leaf_hash(address: &str, nonce: u64, metadata: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(address.as_bytes());
    hasher.update(&nonce.to_be_bytes());
    hasher.update(metadata.as_bytes());
    hasher.finalize().into()
}

fn hash_pair(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    if a <= b {
        hasher.update(&a);
        hasher.update(&b);
    } else {
        hasher.update(&b);
        hasher.update(&a);
    }
    hasher.finalize().into()
}
```

### JavaScript Example
```javascript
import { sha256 } from '@noble/hashes/sha256';

function computeLeafHash(address, nonce, metadata) {
  const combined = Buffer.concat([
    Buffer.from(address, 'utf8'),
    Buffer.from(nonce.toString(16).padStart(16, '0'), 'hex'),
    Buffer.from(metadata, 'utf8')
  ]);
  return Buffer.from(sha256(combined));
}

function hashPair(a, b) {
  const sorted = a.compare(b) <= 0 ? [a, b] : [b, a];
  return Buffer.from(sha256(Buffer.concat(sorted)));
}
```

## ⚠️ Error Codes

| Error | Code | Description |
|-------|------|-------------|
| AlreadyInitialized | 1 | Contract already initialized |
| NotInitialized | 2 | Contract not initialized |
| Unauthorized | 3 | Caller lacks required role |
| InvalidProof | 4 | Merkle proof verification failed |
| AlreadyRegistered | 5 | Address already in whitelist |
| NotWhitelisted | 6 | Address not in whitelist |
| InvalidFee | 7 | Fee amount incorrect |
| ProposalNotFound | 8 | Proposal ID doesn't exist |
| ProposalNotPassed | 9 | Proposal didn't meet quorum |
| TimelockNotExpired | 10 | Timelock period not completed |
| AlreadyVoted | 11 | Already voted on this proposal |
| DisputeNotFound | 12 | Dispute ID doesn't exist |
| DisputePeriodActive | 13 | Dispute period still ongoing |
| RateLimitExceeded | 14 | Too many requests |
| Blacklisted | 15 | Address is blacklisted |
| ContractPaused | 16 | Contract in emergency pause |
| InvalidRole | 17 | Invalid role specified |
| InvalidProposal | 18 | Proposal in invalid state |
| DisputeAlreadyResolved | 19 | Dispute already resolved |
| InsufficientStake | 20 | Validator stake too low |

## 🧪 Testing Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_verify_whitelist_valid_proof

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo test --release -- --nocapture benchmark

# Test with coverage
cargo tarpaulin --out Html
```

## 📚 Documentation Links

| Document | Description |
|----------|-------------|
| [README.md](./README.md) | Complete feature overview |
| [SECURITY.md](./SECURITY.md) | Security analysis |
| [INTEGRATION.md](./INTEGRATION.md) | Integration guide |
| [USAGE_EXAMPLES.md](./USAGE_EXAMPLES.md) | Real-world examples |
| [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) | Implementation details |

## 🎯 Use Case Selection

| Use Case | Recommended Config |
|----------|-------------------|
| **Token Airdrop** | High fee, short timelock, no governance |
| **DAO Membership** | No fee, medium timelock, strong governance |
| **NFT Whitelist** | Medium fee, short timelock, admin control |
| **Crowdsale** | Low fee, no timelock, KYC integration |
| **Tiered Access** | Variable fees, long timelock, role-based |

## 🔍 Debugging Checklist

- [ ] Contract initialized?
- [ ] Proof generated for current root?
- [ ] Nonce matches expected value?
- [ ] Metadata format correct?
- [ ] Fee paid or waiver granted?
- [ ] Not blacklisted?
- [ ] Not rate limited?
- [ ] Contract not paused?
- [ ] Authorization signature valid?
- [ ] Proof array in correct order?

## 💡 Best Practices

1. **Store proofs off-chain** - Distribute via IPFS or API
2. **Batch operations** - Group verifications when possible
3. **Monitor TTL** - Extend storage before expiration
4. **Test on testnet** - Extensive testing before mainnet
5. **Multi-sig admin** - Use hardware wallet or multi-sig
6. **Document changes** - Keep governance changelog
7. **Emergency plan** - Have pause/recovery procedures
8. **Audit first** - Security audit before mainnet launch

## 🆘 Common Issues

### Issue: InvalidProof
**Solution**: Regenerate proof for current root version

### Issue: RateLimitExceeded
**Solution**: Wait 1 hour or request fee waiver

### Issue: AlreadyRegistered
**Solution**: Check if already verified, can't register twice

### Issue: TimelockNotExpired
**Solution**: Wait for timelock period to complete

### Issue: Unauthorized
**Solution**: Ensure caller has required role

## 📞 Support

- **GitHub Issues**: Technical problems
- **GitHub Discussions**: Questions and ideas
- **Email**: security@example.com (security issues)
- **Discord**: Community support

## 🔗 Quick Links

```
Contract: examples/advanced/29-merkle-whitelist/
Tests: src/test.rs
Benchmarks: benches/merkle_benchmarks.rs
Examples: examples/generate_merkle_tree.rs
```

---

**Last Updated**: 2026-08-27 | **Version**: 1.0.0 | **License**: Apache-2.0

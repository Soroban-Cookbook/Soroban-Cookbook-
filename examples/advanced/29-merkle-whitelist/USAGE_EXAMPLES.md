# Usage Examples: Merkle Whitelist Contract

This document provides real-world usage examples for common scenarios.

## Table of Contents

1. [Token Airdrop](#token-airdrop)
2. [DAO Membership](#dao-membership)
3. [NFT Whitelist Minting](#nft-whitelist-minting)
4. [Crowdsale Access Control](#crowdsale-access-control)
5. [Tiered Access System](#tiered-access-system)
6. [Dynamic Whitelist Updates](#dynamic-whitelist-updates)

---

## 1. Token Airdrop

Distribute tokens to eligible recipients using Merkle proof verification.

### Scenario
- 10,000 eligible addresses
- Claim window: 30 days
- Anti-sybil protection via fees

### Implementation

```rust
// 1. Generate whitelist with airdrop amounts
let whitelist = vec![
    ("GABC...", 1000_0000000), // 1000 tokens
    ("GDEF...", 500_0000000),  // 500 tokens
    // ... 10,000 entries
];

// 2. Build Merkle tree with amounts in metadata
let entries: Vec<WhitelistEntry> = whitelist
    .iter()
    .map(|(addr, amount)| WhitelistEntry {
        address: addr.to_string(),
        nonce: 0,
        metadata: amount.to_string(),
    })
    .collect();

let tree = build_merkle_tree(entries);

// 3. Initialize contract
client.initialize(
    &admin,
    &token_address,
    &100_0000000, // 100 token registration fee (refunded on claim)
    &tree.root,
);

// 4. User claims airdrop
let proof = tree.proofs.get(&user_address).unwrap();
let metadata = amount.to_string();

client.verify_whitelist(&user, &proof, &metadata);

// 5. Transfer tokens after verification
if client.is_whitelisted(&user) {
    let entry = client.get_entry(&user);
    let amount: i128 = entry.metadata.parse().unwrap();
    token_client.transfer(&airdrop_wallet, &user, &amount);
}
```

### Best Practices

- Set registration fee high enough to deter spam
- Grant fee waivers to known good actors
- Implement deadline for claims
- Redistribute unclaimed tokens after deadline

---

## 2. DAO Membership

Use whitelist to manage DAO membership and voting rights.

### Scenario
- Membership requires application + approval
- Different membership tiers (basic, premium, founder)
- Quarterly membership reviews

### Implementation

```rust
// 1. Define membership tiers
enum MembershipTier {
    Basic,    // Can vote
    Premium,  // Can vote + propose
    Founder,  // Can vote + propose + veto
}

// 2. Build whitelist with tiers
let members = vec![
    ("GABC...", MembershipTier::Founder),
    ("GDEF...", MembershipTier::Premium),
    ("GHIJ...", MembershipTier::Basic),
];

let entries: Vec<WhitelistEntry> = members
    .iter()
    .map(|(addr, tier)| WhitelistEntry {
        address: addr.to_string(),
        nonce: 0,
        metadata: format!("{:?}", tier),
    })
    .collect();

let tree = build_merkle_tree(entries);

// 3. Initialize with zero fees (private membership)
client.initialize(&admin, &token_address, &0, &tree.root);

// 4. Grant governor role to admins
client.add_role(&admin, &membership_admin, &Role::Governor);

// 5. Member verification for voting
fn can_vote(client: &Client, address: &Address) -> bool {
    if !client.is_whitelisted(address) {
        return false;
    }
    
    let entry = client.get_entry(address);
    let tier = entry.metadata.to_string();
    
    matches!(tier.as_str(), "Basic" | "Premium" | "Founder")
}

fn can_propose(client: &Client, address: &Address) -> bool {
    if !client.is_whitelisted(address) {
        return false;
    }
    
    let entry = client.get_entry(address);
    let tier = entry.metadata.to_string();
    
    matches!(tier.as_str(), "Premium" | "Founder")
}

// 6. Quarterly membership review
// Admins propose new root with updated members
let updated_tree = build_merkle_tree(updated_members);
let proposal_id = client.propose_root_update(
    &admin,
    &updated_tree.root,
    &"Q3 2026 membership update",
);

// Governors vote
client.vote_on_proposal(&admin1, &proposal_id, &true);
client.vote_on_proposal(&admin2, &proposal_id, &true);
client.vote_on_proposal(&admin3, &proposal_id, &true);

// Execute after timelock
client.execute_proposal(&admin, &proposal_id);
```

---

## 3. NFT Whitelist Minting

Control access to NFT minting phases using whitelist.

### Scenario
- Phase 1: Founders (100 spots, free mint)
- Phase 2: Whitelist (1000 spots, 0.1 XLM)
- Phase 3: Public (unlimited, 1 XLM)

### Implementation

```rust
// 1. Phase 1: Founder whitelist
let founders = vec![
    "GABC...", "GDEF...", // ... 100 addresses
];

let phase1_entries: Vec<WhitelistEntry> = founders
    .iter()
    .map(|addr| WhitelistEntry {
        address: addr.to_string(),
        nonce: 0,
        metadata: "phase1_free".to_string(),
    })
    .collect();

let phase1_tree = build_merkle_tree(phase1_entries);

// Initialize for phase 1
client.initialize(&admin, &token_address, &0, &phase1_tree.root);

// 2. Founder minting
fn mint_founder_nft(client: &Client, minter: &Address, proof: &Vec<String>) {
    // Verify whitelist
    let metadata = "phase1_free";
    client.verify_whitelist(minter, proof, &metadata);
    
    if client.is_whitelisted(minter) {
        // Mint NFT for free
        nft_contract.mint(minter, &next_token_id());
    }
}

// 3. Transition to Phase 2
let phase2_whitelist = vec![
    "GKLM...", "GNOP...", // ... 1000 addresses
];

let phase2_entries: Vec<WhitelistEntry> = phase2_whitelist
    .iter()
    .map(|addr| WhitelistEntry {
        address: addr.to_string(),
        nonce: 0,
        metadata: "phase2_discount".to_string(),
    })
    .collect();

let phase2_tree = build_merkle_tree(phase2_entries);

// Propose root update
let proposal_id = client.propose_root_update(
    &admin,
    &phase2_tree.root,
    &"Phase 2: Whitelist minting",
);

// Vote and execute
// ... voting logic ...

// Update fee for phase 2
client.set_registration_fee(&admin, &100_000); // 0.01 XLM

// 4. Phase 2 minting with discount
fn mint_whitelist_nft(
    client: &Client,
    minter: &Address,
    proof: &Vec<String>,
    payment: i128,
) {
    require!(payment >= 100_000, "Insufficient payment");
    
    client.verify_whitelist(minter, proof, &"phase2_discount");
    
    if client.is_whitelisted(minter) {
        nft_contract.mint(minter, &next_token_id());
    }
}

// 5. Phase 3: Public minting (no whitelist check)
fn mint_public_nft(minter: &Address, payment: i128) {
    require!(payment >= 1_000_000, "Insufficient payment");
    nft_contract.mint(minter, &next_token_id());
}
```

---

## 4. Crowdsale Access Control

Manage investor whitelist for token sale with KYC requirements.

### Scenario
- KYC verification required
- Investment caps per tier
- Jurisdiction restrictions

### Implementation

```rust
// 1. Build investor whitelist after KYC
struct InvestorProfile {
    address: String,
    tier: String,        // "tier1", "tier2", "tier3"
    max_investment: u64, // In smallest token unit
    jurisdiction: String,
}

let investors = vec![
    InvestorProfile {
        address: "GABC...".to_string(),
        tier: "tier1".to_string(),
        max_investment: 100_000_0000000, // 100k tokens
        jurisdiction: "US".to_string(),
    },
    // ... more investors
];

// 2. Encode profile in metadata
let entries: Vec<WhitelistEntry> = investors
    .iter()
    .map(|profile| WhitelistEntry {
        address: profile.address.clone(),
        nonce: 0,
        metadata: serde_json::to_string(&profile).unwrap(),
    })
    .collect();

let tree = build_merkle_tree(entries);

// 3. Initialize crowdsale contract
client.initialize(&admin, &token_address, &0, &tree.root);

// 4. Investment function with whitelist check
fn invest(
    client: &Client,
    investor: &Address,
    amount: i128,
    proof: &Vec<String>,
) -> Result<(), Error> {
    // Verify investor is whitelisted
    let profile_json = get_investor_profile(investor);
    client.verify_whitelist(investor, proof, &profile_json)?;
    
    if !client.is_whitelisted(investor) {
        return Err(Error::NotWhitelisted);
    }
    
    // Get investment limits
    let entry = client.get_entry(investor);
    let profile: InvestorProfile = serde_json::from_str(&entry.metadata)?;
    
    // Check investment cap
    let total_invested = get_total_invested(investor);
    if total_invested + amount > profile.max_investment {
        return Err(Error::ExceedsInvestmentCap);
    }
    
    // Process investment
    token_client.transfer(investor, &sale_wallet, &amount);
    
    Ok(())
}

// 5. Add new investors dynamically
fn add_kyc_approved_investor(profile: InvestorProfile) {
    // Add to off-chain whitelist
    investors.push(profile);
    
    // Rebuild tree
    let new_tree = build_merkle_tree(/* updated entries */);
    
    // Propose root update
    let proposal_id = client.propose_root_update(
        &admin,
        &new_tree.root,
        &format!("Add investor: {}", profile.address),
    );
    
    // Fast-track approval for time-sensitive sales
    // ... voting logic ...
}
```

---

## 5. Tiered Access System

Implement multiple access tiers with different privileges.

### Scenario
- Free tier: Read-only access
- Basic tier: Read + limited writes (10/day)
- Premium tier: Unlimited access
- Enterprise tier: Unlimited + priority

### Implementation

```rust
#[derive(Serialize, Deserialize)]
enum AccessTier {
    Free { read_only: bool },
    Basic { daily_limit: u32 },
    Premium { unlimited: bool },
    Enterprise { unlimited: bool, priority: bool },
}

// 1. Build tiered whitelist
let users = vec![
    ("GABC...", AccessTier::Enterprise { unlimited: true, priority: true }),
    ("GDEF...", AccessTier::Premium { unlimited: true }),
    ("GHIJ...", AccessTier::Basic { daily_limit: 10 }),
    ("GKLM...", AccessTier::Free { read_only: true }),
];

let entries: Vec<WhitelistEntry> = users
    .iter()
    .map(|(addr, tier)| WhitelistEntry {
        address: addr.to_string(),
        nonce: 0,
        metadata: serde_json::to_string(tier).unwrap(),
    })
    .collect();

let tree = build_merkle_tree(entries);

// 2. Access control functions
fn can_read(client: &Client, user: &Address) -> bool {
    client.is_whitelisted(user)
}

fn can_write(client: &Client, user: &Address) -> Result<bool, Error> {
    if !client.is_whitelisted(user) {
        return Ok(false);
    }
    
    let entry = client.get_entry(user);
    let tier: AccessTier = serde_json::from_str(&entry.metadata)?;
    
    match tier {
        AccessTier::Free { .. } => Ok(false),
        AccessTier::Basic { daily_limit } => {
            let usage = get_daily_usage(user);
            Ok(usage < daily_limit)
        }
        AccessTier::Premium { .. } | AccessTier::Enterprise { .. } => Ok(true),
    }
}

fn has_priority(client: &Client, user: &Address) -> bool {
    if let Ok(entry) = client.get_entry(user) {
        if let Ok(tier) = serde_json::from_str::<AccessTier>(&entry.metadata) {
            return matches!(tier, AccessTier::Enterprise { priority: true, .. });
        }
    }
    false
}

// 3. Rate limiting integration
fn execute_write(
    client: &Client,
    user: &Address,
    data: &Bytes,
) -> Result<(), Error> {
    if !can_write(client, user)? {
        return Err(Error::InsufficientAccess);
    }
    
    // Check priority queue
    if has_priority(client, user) {
        priority_queue.push(user, data);
    } else {
        regular_queue.push(user, data);
    }
    
    // Update usage counter for Basic tier
    increment_daily_usage(user);
    
    Ok(())
}
```

---

## 6. Dynamic Whitelist Updates

Handle frequent whitelist updates efficiently.

### Scenario
- Daily whitelist additions
- Automated KYC approval
- Minimal governance overhead

### Implementation

```rust
// 1. Incremental update strategy
struct WhitelistManager {
    current_root: BytesN<32>,
    pending_additions: Vec<Address>,
    update_frequency: Duration,
}

impl WhitelistManager {
    fn add_pending(&mut self, address: Address, metadata: String) {
        self.pending_additions.push((address, metadata));
    }
    
    fn should_update(&self) -> bool {
        let time_since_update = now() - self.last_update;
        time_since_update >= self.update_frequency || 
        self.pending_additions.len() >= 100
    }
    
    async fn process_updates(&mut self, client: &Client) -> Result<(), Error> {
        if !self.should_update() {
            return Ok(());
        }
        
        // Build new tree with additions
        let mut all_entries = get_current_whitelist();
        all_entries.extend(self.pending_additions.drain(..));
        
        let new_tree = build_merkle_tree(all_entries);
        
        // Propose update
        let proposal_id = client.propose_root_update(
            &admin,
            &new_tree.root,
            &format!("Daily update: {} new entries", self.pending_additions.len()),
        )?;
        
        // Auto-approve for routine updates
        auto_vote_routine_proposals(client, proposal_id).await?;
        
        // Execute after timelock
        sleep(TIMELOCK_DURATION).await;
        client.execute_proposal(&admin, &proposal_id)?;
        
        self.current_root = new_tree.root;
        self.last_update = now();
        
        Ok(())
    }
}

// 2. Batch processing
async fn batch_add_users(
    manager: &mut WhitelistManager,
    kyc_approvals: Vec<KycApproval>,
) {
    for approval in kyc_approvals {
        let metadata = format!(
            "{{\"tier\":\"{}\",\"kyc_date\":\"{}\"}}",
            approval.tier,
            approval.approved_at
        );
        
        manager.add_pending(approval.address, metadata);
    }
    
    // Process if threshold reached
    if manager.should_update() {
        manager.process_updates(&client).await.unwrap();
    }
}

// 3. Emergency individual additions
async fn emergency_add_user(
    client: &Client,
    address: Address,
    justification: String,
) -> Result<(), Error> {
    // Use fee waiver for emergency additions
    client.grant_fee_waiver(&admin, &address)?;
    
    // Admin directly adds (bypasses merkle proof temporarily)
    // User can be included in next batch merkle update
    
    log::warn!(
        "Emergency addition: {} - Reason: {}",
        address,
        justification
    );
    
    Ok(())
}
```

---

## Common Patterns

### Pattern 1: Proof Caching

```rust
// Cache proofs client-side
struct ProofCache {
    proofs: HashMap<Address, Vec<BytesN<32>>>,
    root_version: u64,
}

impl ProofCache {
    fn get_proof(&mut self, address: &Address, current_root_version: u64) -> Option<Vec<BytesN<32>>> {
        // Invalidate cache if root changed
        if self.root_version != current_root_version {
            self.proofs.clear();
            self.root_version = current_root_version;
            return None;
        }
        
        self.proofs.get(address).cloned()
    }
}
```

### Pattern 2: Metadata Encoding

```rust
// Standardized metadata format
#[derive(Serialize, Deserialize)]
struct StandardMetadata {
    tier: String,
    attributes: HashMap<String, String>,
    expires_at: Option<u64>,
}

fn encode_metadata(meta: &StandardMetadata) -> String {
    serde_json::to_string(meta).unwrap()
}
```

### Pattern 3: Dispute Monitoring

```rust
// Monitor for disputes and auto-respond
async fn monitor_disputes(client: &Client) {
    loop {
        let dispute_count = client.get_dispute_count();
        
        for id in 1..=dispute_count {
            let dispute = client.get_dispute(id);
            
            if !dispute.resolved && should_vote_on(&dispute) {
                let decision = analyze_dispute(&dispute);
                client.vote_on_dispute(&validator, &id, &decision).await?;
            }
        }
        
        sleep(Duration::from_secs(3600)).await; // Check hourly
    }
}
```

---

## Testing Strategies

```rust
#[cfg(test)]
mod usage_tests {
    use super::*;
    
    #[test]
    fn test_airdrop_flow() {
        // ... full airdrop scenario
    }
    
    #[test]
    fn test_dao_membership() {
        // ... DAO membership verification
    }
    
    #[test]
    fn test_nft_phases() {
        // ... multi-phase NFT minting
    }
}
```

---

## Additional Resources

- [Integration Guide](./INTEGRATION.md)
- [Security Analysis](./SECURITY.md)
- [API Reference](./README.md#api-reference)
- [Benchmarks](./benches/merkle_benchmarks.rs)

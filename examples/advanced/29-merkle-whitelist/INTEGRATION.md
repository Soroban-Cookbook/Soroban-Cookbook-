# Integration Guide: Merkle Whitelist Contract

This guide walks through integrating the Merkle Whitelist contract into your application.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Contract Deployment](#contract-deployment)
3. [Off-Chain Merkle Tree Generation](#off-chain-merkle-tree-generation)
4. [Client Integration](#client-integration)
5. [Governance Integration](#governance-integration)
6. [Security Considerations](#security-considerations)

## Prerequisites

### Dependencies

```toml
[dependencies]
soroban-sdk = "21.0.0"
sha2 = "0.10"
serde = "1.0"
serde_json = "1.0"
```

### System Requirements

- Rust 1.70+
- Soroban CLI
- Node.js 18+ (for frontend integration)

## Contract Deployment

### 1. Deploy Contract

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to network
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/merkle_whitelist.wasm \
  --network testnet

# Output: CONTRACT_ID
```

### 2. Initialize Contract

```bash
# Initialize with admin, fee token, and initial root
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- initialize \
  --admin $ADMIN_ADDRESS \
  --fee_token $TOKEN_ADDRESS \
  --registration_fee 1000000 \
  --initial_root $MERKLE_ROOT
```

### 3. Set Up Roles

```bash
# Add governors
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- add_role \
  --caller $ADMIN_ADDRESS \
  --target $GOVERNOR_ADDRESS \
  --role Governor

# Add validators
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- add_role \
  --caller $ADMIN_ADDRESS \
  --target $VALIDATOR_ADDRESS \
  --role Validator
```

## Off-Chain Merkle Tree Generation

### Using Rust

```rust
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WhitelistEntry {
    address: String,
    nonce: u64,
    metadata: String,
}

#[derive(Serialize, Deserialize)]
struct MerkleTree {
    root: String,
    entries: Vec<WhitelistEntry>,
    proofs: HashMap<String, Vec<String>>,
}

fn compute_leaf_hash(address: &str, nonce: u64, metadata: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(address.as_bytes());
    hasher.update(&nonce.to_be_bytes());
    hasher.update(metadata.as_bytes());
    hasher.finalize().into()
}

fn build_merkle_tree(entries: Vec<WhitelistEntry>) -> MerkleTree {
    let mut leaves: Vec<[u8; 32]> = entries
        .iter()
        .map(|e| compute_leaf_hash(&e.address, e.nonce, &e.metadata))
        .collect();
    
    let mut current_level = leaves.clone();
    let mut tree_levels = vec![leaves.clone()];
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                left
            };
            
            let parent = hash_pair(left, right);
            next_level.push(parent);
        }
        
        tree_levels.push(next_level.clone());
        current_level = next_level;
    }
    
    let root = current_level[0];
    
    // Generate proofs for each entry
    let mut proofs = HashMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        let proof = generate_proof(&tree_levels, idx);
        proofs.insert(entry.address.clone(), proof);
    }
    
    MerkleTree {
        root: hex::encode(root),
        entries,
        proofs,
    }
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

fn generate_proof(tree_levels: &[Vec<[u8; 32]>], mut index: usize) -> Vec<String> {
    let mut proof = Vec::new();
    
    for level in tree_levels.iter().take(tree_levels.len() - 1) {
        let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        
        if sibling_index < level.len() {
            proof.push(hex::encode(level[sibling_index]));
        } else {
            proof.push(hex::encode(level[index]));
        }
        
        index /= 2;
    }
    
    proof
}

// Example usage
fn main() {
    let entries = vec![
        WhitelistEntry {
            address: "GABC...".to_string(),
            nonce: 0,
            metadata: "tier1".to_string(),
        },
        WhitelistEntry {
            address: "GDEF...".to_string(),
            nonce: 0,
            metadata: "tier2".to_string(),
        },
    ];
    
    let tree = build_merkle_tree(entries);
    
    // Save to file for distribution
    let json = serde_json::to_string_pretty(&tree).unwrap();
    std::fs::write("merkle_tree.json", json).unwrap();
    
    println!("Merkle root: {}", tree.root);
}
```

### Using JavaScript/TypeScript

```typescript
import { sha256 } from '@noble/hashes/sha256';

interface WhitelistEntry {
  address: string;
  nonce: bigint;
  metadata: string;
}

interface MerkleTree {
  root: string;
  entries: WhitelistEntry[];
  proofs: Map<string, string[]>;
}

function computeLeafHash(address: string, nonce: bigint, metadata: string): Buffer {
  const addressBytes = Buffer.from(address, 'utf8');
  const nonceBytes = Buffer.allocUnsafe(8);
  nonceBytes.writeBigUInt64BE(nonce);
  const metadataBytes = Buffer.from(metadata, 'utf8');
  
  const combined = Buffer.concat([addressBytes, nonceBytes, metadataBytes]);
  return Buffer.from(sha256(combined));
}

function hashPair(a: Buffer, b: Buffer): Buffer {
  const sorted = a.compare(b) <= 0 ? [a, b] : [b, a];
  const combined = Buffer.concat(sorted);
  return Buffer.from(sha256(combined));
}

function buildMerkleTree(entries: WhitelistEntry[]): MerkleTree {
  const leaves = entries.map(e => 
    computeLeafHash(e.address, e.nonce, e.metadata)
  );
  
  let currentLevel = [...leaves];
  const treeLevels = [leaves];
  
  while (currentLevel.length > 1) {
    const nextLevel: Buffer[] = [];
    
    for (let i = 0; i < currentLevel.length; i += 2) {
      const left = currentLevel[i];
      const right = i + 1 < currentLevel.length ? currentLevel[i + 1] : left;
      nextLevel.push(hashPair(left, right));
    }
    
    treeLevels.push(nextLevel);
    currentLevel = nextLevel;
  }
  
  const root = currentLevel[0].toString('hex');
  
  // Generate proofs
  const proofs = new Map<string, string[]>();
  entries.forEach((entry, idx) => {
    const proof = generateProof(treeLevels, idx);
    proofs.set(entry.address, proof);
  });
  
  return { root, entries, proofs };
}

function generateProof(treeLevels: Buffer[][], index: number): string[] {
  const proof: string[] = [];
  let idx = index;
  
  for (let level = 0; level < treeLevels.length - 1; level++) {
    const siblingIndex = idx % 2 === 0 ? idx + 1 : idx - 1;
    
    if (siblingIndex < treeLevels[level].length) {
      proof.push(treeLevels[level][siblingIndex].toString('hex'));
    } else {
      proof.push(treeLevels[level][idx].toString('hex'));
    }
    
    idx = Math.floor(idx / 2);
  }
  
  return proof;
}

// Example usage
const entries: WhitelistEntry[] = [
  { address: 'GABC...', nonce: 0n, metadata: 'vip' },
  { address: 'GDEF...', nonce: 0n, metadata: 'standard' },
];

const tree = buildMerkleTree(entries);
console.log('Merkle root:', tree.root);

// Save for distribution
const fs = require('fs');
fs.writeFileSync('merkle_tree.json', JSON.stringify({
  root: tree.root,
  entries: entries,
  proofs: Array.from(tree.proofs.entries()),
}, null, 2));
```

## Client Integration

### Verify Whitelist (Frontend)

```typescript
import { SorobanRpc, Contract, Address } from '@stellar/stellar-sdk';

async function verifyWhitelist(
  contractId: string,
  userAddress: string,
  proof: string[],
  metadata: string
): Promise<boolean> {
  const contract = new Contract(contractId);
  
  // Convert proof to contract format
  const proofBytes = proof.map(p => Buffer.from(p, 'hex'));
  const metadataBytes = Buffer.from(metadata, 'utf8');
  
  try {
    await contract.call(
      'verify_whitelist',
      Address.fromString(userAddress),
      proofBytes,
      metadataBytes
    );
    
    return true;
  } catch (error) {
    console.error('Verification failed:', error);
    return false;
  }
}

// Usage
const isVerified = await verifyWhitelist(
  CONTRACT_ID,
  'GABC...',
  ['proof1', 'proof2', 'proof3'],
  'tier1'
);
```

### Check Whitelist Status

```typescript
async function isWhitelisted(
  contractId: string,
  address: string
): Promise<boolean> {
  const contract = new Contract(contractId);
  
  const result = await contract.call(
    'is_whitelisted',
    Address.fromString(address)
  );
  
  return result as boolean;
}
```

## Governance Integration

### Propose Root Update

```typescript
async function proposeRootUpdate(
  contractId: string,
  governorAddress: string,
  newRoot: string,
  description: string
): Promise<number> {
  const contract = new Contract(contractId);
  
  const rootBytes = Buffer.from(newRoot, 'hex');
  const descriptionBytes = Buffer.from(description, 'utf8');
  
  const proposalId = await contract.call(
    'propose_root_update',
    Address.fromString(governorAddress),
    rootBytes,
    descriptionBytes
  );
  
  return Number(proposalId);
}
```

### Vote on Proposal

```typescript
async function voteOnProposal(
  contractId: string,
  voterAddress: string,
  proposalId: number,
  support: boolean
): Promise<void> {
  const contract = new Contract(contractId);
  
  await contract.call(
    'vote_on_proposal',
    Address.fromString(voterAddress),
    proposalId,
    support
  );
}
```

### Execute Proposal

```typescript
async function executeProposal(
  contractId: string,
  executorAddress: string,
  proposalId: number
): Promise<void> {
  const contract = new Contract(contractId);
  
  await contract.call(
    'execute_proposal',
    Address.fromString(executorAddress),
    proposalId
  );
}
```

## Security Considerations

### 1. Merkle Tree Generation

- **Use secure randomness** for nonces if implementing nonce-based replay protection
- **Validate all inputs** before computing leaf hashes
- **Store proofs securely** - users must retain their proofs to verify
- **Version control** - track which root version was used for each proof

### 2. Root Updates

- **Require multi-sig** for root updates in production
- **Implement timelock** - allow community review before execution
- **Audit new roots** - verify all intended addresses are included
- **Backup old roots** - maintain history for dispute resolution

### 3. Proof Distribution

- **HTTPS only** - serve proofs over encrypted connections
- **Access control** - only provide proofs to entitled users
- **Rate limiting** - prevent proof harvesting attacks
- **Monitoring** - track proof requests for abuse detection

### 4. Frontend Security

- **Input validation** - verify addresses and proofs client-side first
- **Error handling** - don't expose internal contract errors to users
- **Replay protection** - implement nonce tracking
- **Fee estimation** - inform users of costs before transaction

## Example: Full Integration Flow

```typescript
// 1. Generate whitelist
const whitelist = [
  { address: 'GABC...', nonce: 0n, metadata: 'vip' },
  { address: 'GDEF...', nonce: 0n, metadata: 'standard' },
];

const tree = buildMerkleTree(whitelist);

// 2. Deploy and initialize contract
const contractId = await deployContract();
await initializeContract(contractId, tree.root);

// 3. Distribute proofs to users
await distributeProofs(tree.proofs);

// 4. User verifies their whitelist status
const userProof = tree.proofs.get(userAddress);
await verifyWhitelist(contractId, userAddress, userProof, 'vip');

// 5. Check status
const isListed = await isWhitelisted(contractId, userAddress);
console.log('Whitelisted:', isListed);

// 6. Update whitelist via governance (when needed)
const newTree = buildMerkleTree(updatedWhitelist);
const proposalId = await proposeRootUpdate(
  contractId,
  governorAddress,
  newTree.root,
  'Q4 2026 whitelist update'
);

// 7. Governors vote
await voteOnProposal(contractId, gov1Address, proposalId, true);
await voteOnProposal(contractId, gov2Address, proposalId, true);
await voteOnProposal(contractId, gov3Address, proposalId, true);

// 8. Execute after timelock
await sleep(86400000); // Wait 1 day
await executeProposal(contractId, adminAddress, proposalId);
```

## Troubleshooting

### Common Errors

**Error: InvalidProof**
- Verify proof was generated for current root version
- Check leaf hash computation matches contract
- Ensure proof array is in correct order

**Error: AlreadyRegistered**
- User has already verified their whitelist status
- Nonce has been incremented

**Error: Blacklisted**
- Address was flagged via dispute resolution
- Contact admin for review

**Error: RateLimitExceeded**
- Too many verification attempts
- Wait for rate limit window to reset (default: 1 hour)

**Error: ContractPaused**
- Contract is in emergency pause mode
- Wait for admin to unpause

## Best Practices

1. **Test thoroughly** - Use testnet extensively before mainnet
2. **Monitor gas costs** - Proof depth affects verification cost
3. **Plan for updates** - Design governance process before launch
4. **Document everything** - Keep detailed records of root changes
5. **User education** - Teach users about proof storage responsibility
6. **Backup strategy** - Maintain redundant proof storage
7. **Incident response** - Have emergency pause/recovery plan

## Resources

- [Merkle Tree Visualization Tool](https://example.com/merkle-viz)
- [Proof Generator Service](https://example.com/proof-gen)
- [Contract Explorer](https://example.com/explorer)
- [Governance Dashboard](https://example.com/governance)

## Support

For integration support:
- GitHub Issues: [link]
- Discord: [link]
- Documentation: [link]

# Merkle Airdrop

A gas-efficient, secure on-chain token/asset distribution mechanism using Merkle proof verification on Soroban.

## Why a Merkle Airdrop?

Distributing tokens to a large list of eligible addresses (e.g., thousands of accounts) is a common requirement in decentralized networks. However, storing every eligible address and its corresponding allocation directly in on-chain storage is extremely expensive and scales linearly $O(n)$ with the size of the distribution list.

A **Merkle Airdrop** solves this problem by using a Merkle tree to commit to the entire distribution dataset off-chain. Only a single 32-byte hash (the **Merkle Root**) is stored on-chain. Eligible users can then claim their tokens by providing their address, allocation amount, and a **Merkle Proof** (a list of sibling hashes $O(\log n)$) to prove their membership in the committed list.

### Key Benefits

- **Gas Efficiency**: The contract storage requirement is $O(1)$ (independent of the number of recipients), and verification is $O(\log n)$, keeping on-chain transaction fees minimal.
- **Double-Claim Prevention**: The contract uses persistent storage to track which addresses have successfully claimed their tokens, ensuring each user can only claim their allocation once.
- **On-Chain Autonomy**: Users interact directly with the contract to claim their tokens autonomously without relying on central relayers.

---

## How It Works

1. **Off-Chain Generation**: A script hashes every individual claim leaf `(claimer, amount)`. It recursively pairs and hashes sibling nodes in sorted order to construct a Merkle tree and find the unique **Merkle Root**.
2. **On-Chain Initialization**: The contract is initialized with the Merkle root, the token address (SAC / Stellar Asset Contract), and an admin address.
3. **Claim Process**: The claimant requests their specific Merkle proof from an off-chain API or database, then calls the `claim` function. The contract reconstructs the leaf hash using the sender's address and claimed amount, verifies the proof matches the stored root, and executes the token transfer.

---

## Storage Strategy

- **Instance Storage**: Stored configuration details, including `Admin`, `Token`, and the `Root` hash. These are cheap and shared across every invocation.
- **Persistent Storage**: Claim status flags `Claimed(Address)` are recorded in persistent storage. To minimize storage fees, only the presence of a claim is recorded (`claimer_address => true`), and its time-to-live (TTL) is extended dynamically on claim.

---

## Off-Chain Tree and Proof Generation (Rust)

Below is an example snippet showing how to generate the leaf hashes, tree root, and proofs in Rust (matching the canonical hashing rules enforced by the contract):

```rust
use sha2::{Digest, Sha256};

// Define leaf structure matching the contract's XDR serialization
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimLeaf {
    pub claimer: String, // String representation or binary XDR Address
    pub amount: i128,
}

// Order of two sibling hashes is sorted before hashing
fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    if a <= b {
        hasher.update(a);
        hasher.update(b);
    } else {
        hasher.update(b);
        hasher.update(a);
    }
    hasher.finalize().into()
}

// Build the Merkle root and verify proof paths off-chain
pub fn build_merkle_tree(leaves: Vec<[u8; 32]>) -> [u8; 32] {
    let mut current_level = leaves;
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        let mut i = 0;
        while i < current_level.len() {
            if i + 1 < current_level.len() {
                next_level.push(hash_pair(&current_level[i], &current_level[i + 1]));
            } else {
                next_level.push(current_level[i].clone()); // Odd carry up
            }
            i += 2;
        }
        current_level = next_level;
    }
    current_level[0]
}
```

---

## Contract Interface

| Function | Parameters | Description |
| --- | --- | --- |
| `initialize` | `admin: Address`, `token: Address`, `root: BytesN<32>` | Initializes the contract. Only callable once. |
| `claim` | `claimer: Address`, `amount: i128`, `proof: Vec<BytesN<32>>` | Verifies the proof against the stored root, transfers tokens to the claimer, and marks the address as claimed. Enforces `claimer.require_auth()`. |
| `get_admin` | None | Returns the administrator address. |
| `get_token` | None | Returns the token address. |
| `get_root` | None | Returns the active 32-byte Merkle root. |
| `is_claimed` | `claimer: Address` | Returns whether the given address has claimed its airdrop. |
| `hash_leaf` | `claimer: Address`, `amount: i128` | Helper function to compute the exact leaf hash of a claim tuple. |

---

## Unit Testing & Verification

Comprehensive unit tests are provided in `src/test.rs` to guarantee contract correctness:

```bash
# Run tests for this contract
cargo test -p merkle-airdrop

# Run linter checks
cargo clippy -p merkle-airdrop --all-targets -- -D warnings

# Compile to Wasm for deployment
cargo build --target wasm32-unknown-unknown --release -p merkle-airdrop
```

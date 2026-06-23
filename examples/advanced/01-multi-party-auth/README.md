# Multi-Party Authorization Patterns

This example demonstrates advanced patterns for requiring multiple parties to authorize actions in Soroban smart contracts. It covers threshold signatures (M-of-N), authorization vectors for compact storage, and proposal-based approval workflows.

## 📖 What You'll Learn

- **Threshold Signatures**: Implementing M-of-N approval logic
- **Authorization Vectors**: Encoding multiple addresses into a compact `Bytes` blob
- **Canonical Representation**: Sorting and deduplicating signer lists for deterministic storage
- **Multi-Sig Patterns**: N-of-N vs. M-of-N authorization trade-offs
- **Audit Trails**: Emitting events with structured payloads for off-chain tracking

## 🔍 Contract Overview

The contract implements three complementary multi-party authorization patterns:

### 1. N-of-N Multi-Sig (Direct Pattern)

```rust
pub fn multi_sig_transfer(env: Env, signers: Vec<Address>, to: Address, amount: i128)
```

Requires every address in the `signers` list to authorize the transaction. If any signer has not signed, the transaction is rejected by the host.

**Use Cases:**
- Atomic joint agreements
- High-value transfers requiring unanimous consent
- Pair-based approvals

### 2. Authorization Vectors (Compact Pattern)

```rust
pub fn encode_auth_vec(env: Env, signers: Vec<Address>) -> Bytes
pub fn multi_sig_transfer_encoded(env: Env, encoded_signers: Bytes, to: Address, amount: i128)
```

Serializes a list of addresses into a sorted, deduplicated `Bytes` blob. This is significantly cheaper to store in a contract's storage than a `Vec<Address>` and can be passed between contracts as a single value.

**Key Features:**
- **Deterministic**: The same set of addresses always produces the same blob regardless of input order.
- **Validated**: Decoding logic enforces strict ordering and uniqueness constraints.
- **O(1) Existence Check**: Efficiently check if an address is in the vector without full decoding.

### 3. M-of-N Threshold (Proposal Pattern)

```rust
pub fn setup_proposal(env: Env, proposal_id: Symbol, threshold: u32, signers: Vec<Address>)
pub fn proposal_approval(env: Env, proposal_id: Symbol, approvers: Vec<Address>)
```

Allows an action to proceed once at least `M` out of `N` pre-authorized signers have approved. Approvals can be collected across multiple calls or in a single batch.

**Use Cases:**
- DAO governance
- Corporate treasury management
- Security-critical contract upgrades

## 💡 Key Concepts

### Authorization Vector Wire Format

To keep storage costs low and ensure canonical representation, addresses are stored as follows:

1. **Header**: 4-byte big-endian `u32` representing the number of signers.
2. **Payload**: A sequence of 56-byte ASCII strkeys (G... or C...).
3. **Constraint**: Entries must be in strict ascending lexicographical order.

### Canonical Sorting

When encoding, the contract bubble-sorts addresses by their strkey value. This ensures that `[Alice, Bob]` and `[Bob, Alice]` result in the exact same `Bytes` payload, which is critical for using the blob as a storage key or for equality checks.

### Audit Trail Events

Every multi-party action emits a structured event:

```rust
#[contracttype]
pub struct AuditTrailEventData {
    pub details: Symbol,
    pub timestamp: u64,
}
```

This allows off-chain indexers to reconstruct the history of multi-sig actions and provide an audit trail for users.

## 🔒 Security Considerations

### Unbounded Loops

The `multi_sig_transfer` function iterates over the `signers` list. In a production contract, you should enforce a `MAX_SIGNERS` limit (e.g., 20) to prevent an attacker from creating a signer list so large that it exceeds the transaction's CPU/memory limits, effectively "bricking" the contract.

### Replay Protection

The `proposal_approval` pattern should be combined with a mechanism to mark a `proposal_id` as used or expired once the threshold is met, preventing the same set of approvals from being re-played.

## 🧪 Testing

```bash
cargo test -p multi-party-auth
```

Tests cover:
- ✅ Canonical encoding of out-of-order signer lists
- ✅ Validation of malformed or duplicated auth vectors
- ✅ Successful N-of-N authorization
- ✅ M-of-N threshold validation with varying approval sets
- ✅ Event emission and audit trail accuracy

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/multi_party_auth.wasm \
  --source alice \
  --network testnet
```

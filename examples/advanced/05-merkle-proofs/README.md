# Merkle Proof Verification

An on-chain Merkle-root registry that lets a contract certify membership in
a large, off-chain data set (an airdrop allow-list, a voting roll, a batch
of approved transactions, etc.) while storing only a single 32-byte hash.

## Why Merkle Proofs?

Storing thousands of addresses, balances, or approvals directly in contract
storage is expensive and grows linearly with the data set size. A Merkle
tree lets you commit to that entire data set with one hash (the *root*),
and later prove that any single item belongs to the set with a small
*proof* (typically `O(log n)` hashes) — without ever uploading the rest of
the data on-chain.

## How It Works

1. **Off-chain — build the tree.** A script/indexer hashes every leaf
   (e.g. `sha256(address || amount)`), then repeatedly combines sibling
   pairs (sorted, so the same pair always hashes the same way regardless
   of order) into parent hashes until a single root remains. If a level
   has an odd node out, it is carried up unchanged rather than duplicated,
   avoiding a known second-preimage-style pitfall in naive Merkle tree
   implementations.
2. **On-chain — publish the root.** The admin calls `set_root` with the
   computed root and the leaf count. This is the only piece of the data
   set that ever touches contract storage.
3. **On-chain — verify a proof.** Anyone can call `verify` (read-only) or
   `verify_and_claim` (records the claim) with a leaf hash, its original
   index, and the sibling hashes from the off-chain tree. The contract
   recomputes the root from the leaf upward and compares it to the stored
   root.

## Efficient Storage

- The root, leaf count, and a generation counter live in **instance**
  storage — O(1) regardless of how many leaves the tree commits to.
- `verify_and_claim` records consumption in **persistent** storage keyed
  by `(generation, leaf_index)`, so storage grows with the number of
  *claims made*, not the size of the data set. Publishing a new root bumps
  the generation, automatically invalidating old claim records without
  needing to clear them.

## Contract Interface

| Function | Description |
| --- | --- |
| `initialize(admin)` | One-time setup; sets the admin address. |
| `set_root(admin, root, leaf_count)` | Publish/replace the active Merkle root. Admin-only. |
| `get_root_info()` | Returns the active root, leaf count, and generation. |
| `verify(leaf, index, proof)` | Returns `true`/`false` for whether `leaf` is provably in the tree. |
| `verify_and_claim(leaf, index, proof)` | Same check, and records the leaf as claimed (errors if already claimed or invalid). |
| `is_claimed(index)` | Whether `index` has been claimed against the current root generation. |
| `hash_leaf(data)` | Convenience: `sha256(data)`, matching the off-chain leaf-hashing convention. |

## Usage

```rust
// Off-chain: build leaves, compute root + proof for leaf at `index`
// (see src/test.rs for a full reference implementation of the builder).

client.initialize(&admin);
client.set_root(&admin, &root, &leaf_count);

let ok = client.verify(&leaf_hash, &index, &proof);
client.verify_and_claim(&leaf_hash, &index, &proof); // errors on replay or bad proof
```

## Testing

Run the test suite, which includes an off-chain tree-builder reference
implementation used to generate roots/proofs for the on-chain verifier:

```bash
cargo test -p merkle-proofs
```

## Building for Wasm

```bash
cargo build --target wasm32v1-none --release -p merkle-proofs
```

## Security Notes

- This example does not store leaf data on-chain, so it is the *caller's*
  responsibility to publish the leaf data (and the full tree) somewhere
  retrievable (e.g. IPFS, an indexer) so users can fetch their proofs.
- Replacing a root immediately invalidates the previous generation's
  claim records, so plan root rotations carefully (e.g. snapshot claims
  before publishing a new root if continuity is required).
- Always use a domain-separated/pre-hashed leaf scheme
  (`hash_leaf`/`sha256`) rather than raw, unhashed data as a leaf to avoid
  second-preimage attacks against the tree structure.

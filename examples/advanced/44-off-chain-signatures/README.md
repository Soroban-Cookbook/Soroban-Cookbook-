# Off-chain Signature Verification

Verify signatures from keys that are **not** Stellar accounts: an ed25519 key
held by a backend or hardware wallet, or an Ethereum (secp256k1) key. A
trusted signer approves "credit `amount` to `to`" off-chain, and anyone can
submit the approval on-chain.

Use this pattern for cross-chain bridges, Ethereum-wallet logins, signed
vouchers and claims, and oracle or backend attestations. For Stellar accounts,
prefer `require_auth`: the host verifies those signatures and handles their
replay protection for you.

## Functions

| Function | Purpose |
|---|---|
| `__constructor(ed25519_signer: BytesN<32>, eth_signer: BytesN<20>)` | Fix the trusted ed25519 public key and Ethereum address at deploy time |
| `claim_ed25519(to, amount, nonce, signature)` | Verify with `env.crypto().ed25519_verify` over `sha256(payload)` |
| `claim_secp256k1(to, amount, nonce, signature, recovery_id)` | Recover the key with `env.crypto().secp256k1_recover` over `keccak256(payload)` and compare its Ethereum address |
| `domain_separator()` | The 32-byte domain separator every payload starts with |
| `next_nonce(scheme)` | The nonce the next approval for that scheme must use |
| `payload(scheme, to, amount, nonce)` | The exact bytes to sign, so off-chain tools can check their encoding |
| `credited(to)` | Total credited to an address |

## ed25519 vs secp256k1

`ed25519_verify` takes the public key and **traps** if the signature doesn't
match.

The SDK has **no `secp256k1_verify`**. For secp256k1 you call
`secp256k1_recover(digest, signature, recovery_id)`, which returns the public
key that produced the signature, then derive its Ethereum address
(`keccak256(pubkey[1..])[12..]`) and compare it with the trusted one.
Recovery returns *some* key for almost any input, so **the comparison is the
verification**. `signature` is `r || s` (64 bytes) and `recovery_id` is
`v - 27`.

The digest here is `keccak256(payload)`. If you collect signatures with an
Ethereum wallet's `personal_sign`, the wallet first prepends
`"\x19Ethereum Signed Message:\n" || len`, so build the digest the same way on
both sides.

## Signed payload and replay protection

```text
payload = domain_separator || scheme (1 byte) || to.to_xdr() || amount (i128 BE) || nonce (u64 BE)
domain_separator = sha256("soroban-cookbook/off-chain-signatures/v1" || network_id || contract_address.to_xdr())
```

| Attack | Defence |
|---|---|
| Re-submit the same approval | Per-scheme sequential **nonce**: the claim must use `next_nonce` exactly, and the nonce is consumed |
| Replay on another deployment, or testnet vs mainnet | **Domain separator** binds the contract address and network ID |
| Feed an ed25519-path message to the secp256k1 path | **Scheme byte** is part of the payload |
| Change the amount or recipient | Both are signed, so verification fails |

The signature is verified before any state changes. Nonces are sequential, so
approvals must be submitted in order. If you need unordered approvals, store a
set of used nonces (or digests) instead.

## Tests

```bash
cargo test -p off-chain-signatures
```

The tests sign with real keys (`ed25519-dalek`, `k256`) and build the payload
independently of the contract. They cover both happy paths, replaying an
approval, future nonces, approvals signed for another deployment, crossing
schemes, tampered amount or recipient, and untrusted signers.

## Related examples

- [`43-payment-channels`](../43-payment-channels/): ed25519-signed channel states
- [`27-gasless-relayer`](../27-gasless-relayer/): meta-transactions with nonces
- [`30-permit-pattern`](../30-permit-pattern/): EIP-2612-style signed approvals

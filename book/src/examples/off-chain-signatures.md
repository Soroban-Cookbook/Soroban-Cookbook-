# Off-chain Signatures

Verify signatures from keys that are not Stellar accounts, such as a backend's
ed25519 key or an Ethereum wallet's secp256k1 key, so that an approval signed
off-chain can be submitted on-chain by anyone.

See the example: [`examples/advanced/44-off-chain-signatures`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/44-off-chain-signatures).

## ed25519

```rust,ignore
let digest: BytesN<32> = env.crypto().sha256(&payload).into();
env.crypto().ed25519_verify(&trusted_pubkey, &digest.into(), &signature); // traps on mismatch
```

## secp256k1 (Ethereum-compatible)

The SDK has no `secp256k1_verify`. Instead, recover the public key that
signed the digest and compare its Ethereum address with the trusted one. The
comparison *is* the verification, because recovery returns some key for almost
any input.

```rust,ignore
let digest = env.crypto().keccak256(&payload);
let pubkey: BytesN<65> = env.crypto().secp256k1_recover(&digest, &signature, recovery_id); // recovery_id = v - 27
let key: Bytes = pubkey.into();
let hash: Bytes = env.crypto().keccak256(&key.slice(1..)).into();
let address: BytesN<20> = hash.slice(12..).try_into().unwrap();
if address != trusted_eth_address {
    return Err(Error::WrongSigner);
}
```

## Preventing replay

Sign a payload that binds where the approval may be used, and how many times:

```text
payload          = domain_separator || scheme || to.to_xdr() || amount || nonce
domain_separator = sha256("soroban-cookbook/off-chain-signatures/v1" || network_id || contract_address.to_xdr())
```

- **Domain separator**: binds the approval to one contract on one network, so
  it can't be replayed on another deployment or on testnet vs mainnet.
- **Scheme byte**: a message signed for one verification path can't be used on
  the other.
- **Sequential nonce per signer**: the claim must carry exactly the next
  nonce, and the nonce is consumed, so each approval works once.

The example's tests cover replayed approvals, future nonces, approvals signed
for another deployment, crossing schemes, tampered amount or recipient, and
untrusted signers.

For Stellar accounts, use `require_auth` instead. The host verifies those
signatures and handles replay protection.

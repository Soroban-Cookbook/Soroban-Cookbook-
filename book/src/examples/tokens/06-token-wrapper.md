# 06 · Token Wrapper

**Source:** [`examples/tokens/06-token-wrapper/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/06-token-wrapper)

A 1:1 wrapper around an existing SEP-41 token. Users deposit the underlying token and receive wrapped shares; the invariant `wrapped_supply == underlying_balance(contract)` is enforced and tested.

## What You'll Learn

- Cross-contract calls to the underlying token for deposit and withdrawal
- Maintaining the peg invariant after every operation
- Invariant tests that verify balance accounting across operations
- Use cases: wrapping a classic Stellar asset, adding spend limits, or composing multi-token positions

## Quick Code

> ⚠️ **UNAUdITED EXAMPLE — NOT FOR PRODUCTION USE**
>
> The code shown on this page is an educational pattern, not an audited contract. Do **not** deploy it verbatim with real funds. Before any production deployment, obtain a formal security audit, review edge cases for your specific underlying token, and perform your own validation.
>
> **Reentrancy:** This wrapper cross-contract-calls the underlying token on `wrap`, `unwrap`, and `backing`-style reads. The companion example crate uses a reentrancy guard (`DataKey::Entered`) on state-changing entry points. Without a guard, a malicious or hook-bearing underlying token could call back mid-operation and mint unbacked wrapped shares. Reentrancy in Soroban is limited to the same transaction/invocation stack; a cross-contract call cannot outlive the caller's execution frame, but shared-state re-entry within that frame is still possible and must be blocked explicitly.
>
> **Storage TTL / Data Expiry:** Per-user wrapped balances are stored in **persistent** storage, which has a per-key TTL. High-traffic wrappers must explicitly `extend_ttl` on balance writes (or via a periodic maintenance call) or user balances and the `TotalSupply` (instance storage) can expire, breaking the peg. Instance entries (total supply, underlying token address) are tied to the contract instance's TTL and will be reset if the instance expires and is restored; critical accounting must live in persistent storage.

```rust
// Deposit 1000 underlying → receive 1000 wrapped
client.deposit(&user, &1_000_i128);

// Withdraw 500 wrapped → receive 500 underlying
client.withdraw(&user, &500_i128);

// Invariant: wrapped supply == underlying held
assert_eq!(client.total_supply(), underlying.balance(&wrapper));
```

## Run the Example

```bash
cd examples/tokens/06-token-wrapper
cargo test
```

## Next: [07 · Token Metadata](./07-token-metadata.md)

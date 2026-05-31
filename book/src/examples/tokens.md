# Token Examples

Fungible tokens, standards, wrappers.

### Mint/Burn Token [./mint-burn/](../examples/tokens/mint-burn/)
**Controlled issuance and destruction.** Demonstrates admin-only minting, user burn rights, total supply tracking, and optional maximum supply caps.

**Key Concepts:**
- Admin authorization for minting
- Safe burn semantics
- Total supply invariants
- Optional supply cap enforcement
- Event emission for mint/burn operations

### Token Wrapper [./token-wrapper/](../examples/tokens/token-wrapper/)
**Wrap native assets.** Deposit an existing SEP-41 token and mint 1:1 wrapper shares.

**Key Concepts:**
- Cross-contract token transfers
- Backing invariant maintenance
- Mint/burn wrapper shares
- Withdrawal and collateral accounting

## Prerequisites
- [Basics](../basics.md), [Auth](../basics/03-authentication/)

## End of Examples Section 🎉

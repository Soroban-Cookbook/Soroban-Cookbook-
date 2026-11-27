# Token Examples

Soroban fungible token contracts from a minimal SEP-41 implementation to advanced extensions, mint/burn flows, allowances, wrappers, and optimized batch operations.

## Examples

| # | Example | Pattern | Difficulty |
---|--------|-------------|-----------------------|
| 01 | [SEP-41 Token](./tokens/01-sep41-token.md) | Complete SEP-41 fungible token with metadata, approvals, events | Beginner |
| 02 | [SEP-41 Extensions](./tokens/02-sep41-extensions.md) | Permit, batch transfer, batch approve | Intermediate |
| 03 | [Optimized Operations](./tokens/03-optimized-operations.md) | Storage layout and batching with benchmarks | Advanced |
| 04 | [Mint / Burn](./tokens/04-mint-burn.md) | Admin mint + user burn with supply cap | Beginner |
| 05 | [Allowance Pattern](./tokens/05-allowance-pattern.md) | `approve` / `transfer_from` with expiration | Intermediate |
| 06 | [Token Wrapper](./tokens/06-token-wrapper.md) | 1:1 wrapper with peg invariant | Intermediate |
| 07 | [Token Metadata](./tokens/07-token-metadata.md) | Mutable and immutable on-chain metadata | Beginner |
| 08 | [Multi-Token Balance Manager](./tokens/08-multi-token-balance-manager.md) | Portfolio contract across multiple tokens | Intermediate |
| 09 | [Optimized Token Ops](./tokens/09-optimized-token-ops.md) | Micro-optimization patterns for transfers | Intermediate |
| 11 | [Authorized Token](./tokens/11-authorized-token.md) | SEP-41 `authorized` flag for compliance / freeze functionality | Intermediate |

## Gas Cost Benchmarks

See [docs/gas-benchmarks.md](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/gas-benchmarks.md) for gas cost comparisons and CI baselines.

## Prerequisites

- [Basics](./basics.md)

## Testing

Building a frontend for these token contracts? See [Frontend / TypeScript Testing](../docs/testing-best-practices.md#frontend--typescript-testing) for `soroban-spec-typescript` client generation, Jest/Vitest contract invocation tests, and RPC mocking.

## End of Examples Section

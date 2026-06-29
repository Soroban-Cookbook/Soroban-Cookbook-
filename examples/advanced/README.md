# Advanced Examples

This category contains examples of complex systems and advanced architectural patterns for experienced Soroban developers. These examples tackle sophisticated problems and often involve multi-contract interactions and intricate state management.

## What's Inside?

- **Complex Authorization**: Patterns like threshold signatures and multi-party authorization for high-security applications.
- **State Machines**: Contracts that implement complex, multi-step workflows like time-delayed execution.
- **Upgrade Governance**: Admin controls, timelocks, and emergency pauses around contract upgrades.
- **Bridge Defenses**: Inbound bridge release controls such as rate limiting, challenge windows, fraud proofs, and emergency pause.
- **Gas & Ledger Optimization**: Techniques for building highly efficient and scalable contracts.
- **Oracle Patterns**: Single-source oracle with authorized submission and freshness validation.

## Implemented Examples

- [`01-multi-party-auth`](./01-multi-party-auth/) — Multi-party authorization patterns
- [`02-timelock`](./02-timelock/) — Time-delayed execution
- [`03-oracle-pattern`](./03-oracle-pattern/) — Basic oracle with freshness checks
- [`05-bridge-security`](./05-bridge-security/) — Rate limiting, pause, challenge window, and fraud-proof patterns for bridge releases

## Planned Examples

- `04-atomic-swaps`: A trustless, cross-contract asset swap.
- `05-payment-channels`: A basic state channel implementation for off-chain transactions.

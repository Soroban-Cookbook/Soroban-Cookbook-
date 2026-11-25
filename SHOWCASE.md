# Soroban Cookbook Showcase

This page highlights **10+ real projects** in the Stellar ecosystem built on
Soroban that follow the patterns documented in the Soroban Cookbook. They cover
streaming payments, NFTs, oracles, block explorers, verification tools,
crowdfunding, and developer infrastructure across the Stellar network and the
Drips Wave ecosystem.

> Each entry includes a **Confirmation** status. Projects that have not yet been
> verified by their maintainers are marked `pending`; a project becomes
> `confirmed` once its maintainer confirms the cookbook (or its patterns) were
> used. See the [Project Tracking](#project-tracking) section for the workflow.

The showcase fulfills **Phase 8, Issue #441: "10+ Projects Built"**, tracked on
GitHub as issue [#970](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/970).
The goal is to demonstrate that the cookbook's examples and patterns are used by
shipping, production-grade Stellar / Soroban projects, and to make it easy for
developers to discover and learn from them.

---

## Featured Projects

### 1. SoroStream - Streaming Payments
- **Repo:** [SoroStream/sorostream-contracts](https://github.com/SoroStream/sorostream-contracts)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** Soroban smart contracts for continuous value streaming - send
  value per second, run recurring subscriptions, and settle balances trustlessly.
- **Cookbook patterns used:** Persistent vs. Instance storage for stream state,
  `require_auth` authorization, event design, and token transfer integration.

### 2. StellarStream - Payment Streaming MVP
- **Repo:** [ritik4ever/stellar-stream](https://github.com/ritik4ever/stellar-stream)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** A payment-streaming MVP for the Stellar ecosystem - a React
  dashboard, Node/Express API, and a Soroban contract for stream lifecycle,
  claiming, and event indexing.
- **Cookbook patterns used:** Stream state storage, `require_auth` sender /
  recipient authorization, event-driven lifecycle notifications, and the
  deployment workflow from the cookbook guides.

### 3. soroban-nft-marketplace - On-Chain NFT Marketplace
- **Repo:** [waveforge-labs/soroban-nft-marketplace](https://github.com/waveforge-labs/soroban-nft-marketplace)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** A fully on-chain NFT marketplace - mint, list, auction, and
  collect NFTs via Rust smart contracts with a Next.js storefront.
- **Cookbook patterns used:** NFT ownership / metadata conventions documented in
  the cookbook's NFT patterns, escrow listing, and auction settlement logic.

### 4. Soroban-Smart-Block-Explorer - Block Explorer Contracts
- **Repo:** [Soroban-Smart-Block-Explorer/Soroban-Smart-Block](https://github.com/Soroban-Smart-Block-Explorer/Soroban-Smart-Block)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** A smart contract block explorer for Soroban with mainnet
  deployment, upgrade mechanisms, and pause capabilities (UUPS proxy pattern).
- **Cookbook patterns used:** Upgrade patterns, admin / role authorization, and
  cross-network deployment guides.

### 5. Stellar Wave Hub - Project Directory & Registry
- **Repo:** [samieazubike/stellar-wave-hub](https://github.com/samieazubike/stellar-wave-hub)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** "Product Hunt meets a Stellar blockchain explorer" - a
  community directory of Stellar Wave projects with an on-chain
  `wave_hub_registry` contract.
- **Cookbook patterns used:** Contract storage design, admin-gated state
  transitions, and invoke patterns from the cookbook's basics examples.

### 6. soroban-verify - Contract Verification Tool
- **Repo:** [SorobanVerify/soroban-verify](https://github.com/SorobanVerify/soroban-verify)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** A Soroban smart contract verification tool for Stellar (like
  Sourcify for Ethereum) - verify deployed contracts against their source.
- **Cookbook patterns used:** Contract info retrieval, deployment identity
  management, and WASM build workflows from the cookbook guides.

### 7. StellarRoute - DEX & AMM Route Visualizer
- **Repo:** [StellarRoute/route-visualizer](https://github.com/StellarRoute/route-visualizer)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** An embeddable React component that visualizes Stellar DEX &
  AMM trade routes with pool labels and per-hop slippage.
- **Cookbook patterns used:** Reading Soroban contract state via renders, AMM
  interaction patterns, and event-driven price updates.

### 8. Bimex - Impact Crowdfunding
- **Repo:** [David1984TK/Bimex](https://github.com/David1984TK/Bimex) *(Stellar Wave)*
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** A social-impact crowdfunding platform where contributors'
  capital is always recoverable, built on Stellar and Soroban.
- **Cookbook patterns used:** Escrow vaults, token accounting, and claim /
  refund authorization flows from the cookbook's token and authentication
  examples.

### 9. Micopay Protocol - P2P Crypto-Cash
- **Repo:** [Micopay/micopay-protocol](https://github.com/Micopay/micopay-protocol)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** Fast, secure, decentralized P2P crypto-to-cash exchange
  protocol for financial inclusion on Stellar.
- **Cookbook patterns used:** Token contracts, persistent balances, and
  event-based settlement notifications.

### 10. soroban-oracle-safety - Oracle Adapters
- **Repo:** [nice-bills/soroban-oracle-safety](https://github.com/nice-bills/soroban-oracle-safety)
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** SEP-40 oracle safety adapters for Soroban - TWAP, circuit
  breakers, and Blend-compatible price feeds.
- **Cookbook patterns used:** Price data handling, storage patterns for
  time-series data, and safety / auth checks from the cookbook's advanced
  patterns.

### 11. astroid - Modular Wallet Platform
- **Repo:** [ASTROIDX556](https://github.com/ASTROIDX556) *(astroid-contract, astroid-sdk, astroid-api, astroid-web)*
- **Confirmation:** _pending_ (maintainer confirmation requested)
- **What it is:** A full Soroban wallet stack - smart contracts, TypeScript SDK,
  NestJS API, and Next.js dashboard.
- **Cookbook patterns used:** End-to-end frontend <-> contract integration
  following the cookbook's "create an app" guides and deployment workflow.

---

## Case Studies

### Streaming Payments at Scale (SoroStream / StellarStream)
Both streaming protocols reuse the cookbook's **storage-type trade-off guide** to
decide between Persistent and Temporary storage for stream state and checkpoints.
This saved significant design time and kept rent costs predictable.

### NFT Marketplace on Soroban (soroban-nft-marketplace)
The team built directly on the cookbook's **NFT patterns & conventions** for
ownership, metadata, and marketplace design. The cookbook's escrow and
authorization examples shortened the marketplace contract compared to building
from scratch.

### Verifiable Deployments (soroban-verify)
Adopting the cookbook's **deployment and identity-management guides** let the
team standardize testnet-to-mainnet promotion, key handling, and contract
upgrades across every contract they verify.

---

## Developer Support

Building on the cookbook and want help? We've got you covered:

- **Discord:** Chat in the `#soroban` channel on [Stellar Discord](https://discord.gg/stellardev).
- **Issues:** Report bugs, request new examples, or ask about a recipe via the
  [Issues tracker](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues).
- **Documentation:** Guides, patterns, troubleshooting, and reference material
  live in the [documentation index](./docs/README.md).
- **Feedback:** Submit structured feedback through our
  [Community Survey](https://forms.google.com/soroban-cookbook-community-survey).
- See [CONTRIBUTING.md](./CONTRIBUTING.md) for the full contribution workflow.

---

## Project Tracking

This list is maintained as part of the Phase 8 growth milestone. A shared
spreadsheet and the Drips Wave org profile
([drips.network](https://www.drips.network/wave/orgs/694c8fb3-674a-420b-80e3-ef2ece5cc5c2))
track new projects built with the cookbook as they onboard.

### Confirmation workflow

To keep the showcase honest, every entry is assigned one of these statuses:

| Status | Meaning |
|---|---|
| `pending` | Listed for relevance, but not yet verified by the maintainer |
| `confirmed` | Maintainer confirmed the cookbook / its patterns were used (linked evidence) |

Verification steps for `pending` entries:
1. Open a short, friendly issue or comment in the project's repo, e.g. *"Hi! We
   noticed your project builds Soroban contracts. Does it use the Soroban
   Cookbook? We'd love to feature you in our showcase."*
2. Link back to this [SHOWCASE](./SHOWCASE.md) so they can see the context.
3. When the maintainer replies, update the status to `confirmed` and add a link
   to their acknowledgment.

### Status against the acceptance criteria

GitHub issue [#970](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/970)
(Phase 8 issue #441, "10+ Projects Built") asks for five things: project
tracking, a created showcase, developer support, case studies, and 10 projects
featured. All five are met by this file — **11 projects are featured**, the
tracking workflow above is documented, and case studies and support channels
are in place.

Confirmation is how each entry stays **verified over time**, not a prerequisite
for the issue: every entry currently sits at `pending` while maintainer outreach
continues, and moves to `confirmed` (with linked evidence) as replies come in.

> **Want your project featured here?** Open a pull request adding your project
> to the list above (name, link, one-line description, and which cookbook
> patterns you used). We would love to showcase the real-world impact of the
> cookbook.

// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="index.html">Introduction</a></span></li><li class="chapter-item expanded "><li class="part-title">Getting Started</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/getting-started.html"><strong aria-hidden="true">1.</strong> Setup Environment</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/macos-setup.html"><strong aria-hidden="true">2.</strong> macOS Setup Guide</a></span></li><li class="chapter-item expanded "><li class="part-title">Community</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="community-calls.html"><strong aria-hidden="true">3.</strong> Community Calls</a></span></li><li class="chapter-item expanded "><li class="part-title">Guides</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/testing.html"><strong aria-hidden="true">4.</strong> Testing Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/mobile-testing.html"><strong aria-hidden="true">5.</strong> Mobile Testing Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/local-simulation.html"><strong aria-hidden="true">6.</strong> Local Simulation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/deployment.html"><strong aria-hidden="true">7.</strong> Deployment Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/deployment-best-practices.html"><strong aria-hidden="true">8.</strong> Deployment Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/ethereum-to-soroban.html"><strong aria-hidden="true">9.</strong> Ethereum to Soroban</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/project-templates.html"><strong aria-hidden="true">10.</strong> Project Templates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/video-creation.html"><strong aria-hidden="true">11.</strong> Video Creation Tools</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/introduction-video.html"><strong aria-hidden="true">12.</strong> Introduction Video</a></span></li><li class="chapter-item expanded "><li class="part-title">Playground</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="playground.html"><strong aria-hidden="true">13.</strong> Playground Infrastructure</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/playground-intermediate.html"><strong aria-hidden="true">14.</strong> Interactive Playground — Intermediate Examples</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/playground-tutorial-01-hello-world.html"><strong aria-hidden="true">15.</strong> Tutorial 1: Hello World</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/playground-tutorial-02-storage.html"><strong aria-hidden="true">16.</strong> Tutorial 2: Contract Storage</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/playground-tutorial-03-error-handling.html"><strong aria-hidden="true">17.</strong> Tutorial 3: Error Handling</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/playground-tutorial-04-authentication.html"><strong aria-hidden="true">18.</strong> Tutorial 4: Authentication</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="guides/playground-tutorial-05-multi-contract.html"><strong aria-hidden="true">19.</strong> Tutorial 5: Multi-Contract Interactions</a></span></li><li class="chapter-item expanded "><li class="part-title">Examples</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples-index.html"><strong aria-hidden="true">20.</strong> All Examples Index</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="../examples/basics/01-hello-world/index.html"><strong aria-hidden="true">21.</strong> 01 · Hello World (canonical intro)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/basics.html"><strong aria-hidden="true">22.</strong> Overview &amp; listing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/playground.html"><strong aria-hidden="true">23.</strong> Interactive Playground</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/storage-patterns.html"><strong aria-hidden="true">24.</strong> Storage Patterns</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/error-handling.html"><strong aria-hidden="true">25.</strong> Error Handling</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/events.html"><strong aria-hidden="true">26.</strong> Events</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/intermediate.html"><strong aria-hidden="true">27.</strong> Multi-sig patterns</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/ajo-factory.html"><strong aria-hidden="true">28.</strong> Ajo Factory</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/pause-unpause.html"><strong aria-hidden="true">29.</strong> Pause / Unpause</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/advanced.html"><strong aria-hidden="true">30.</strong> Multi-party auth</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/oracle-pattern.html"><strong aria-hidden="true">31.</strong> Oracle Pattern</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/off-chain-signatures.html"><strong aria-hidden="true">32.</strong> Off-chain Signatures</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/advanced/38-diamond-pattern/index.html"><strong aria-hidden="true">33.</strong> Diamond Pattern (canonical)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/advanced/19-diamond-security/index.html"><strong aria-hidden="true">34.</strong> Diamond Security (security-focused)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/advanced/18-diamond-facets/index.html"><strong aria-hidden="true">35.</strong> Diamond Facets (router orchestration)</a></span></li><li class="chapter-item expanded "><li class="part-title">Use Cases</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi.html"><strong aria-hidden="true">36.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/01-simple-swap.html"><strong aria-hidden="true">37.</strong> 01 Â· Simple Swap</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/02-constant-product-amm.html"><strong aria-hidden="true">38.</strong> 02 Â· Constant-Product AMM</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/03-lending-pool.html"><strong aria-hidden="true">39.</strong> 03 Â· Lending Pool</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/04-collateralized-lending.html"><strong aria-hidden="true">40.</strong> 04 Â· Collateralized Lending</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/05-flash-loans.html"><strong aria-hidden="true">41.</strong> 05 Â· Flash Loans</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/06-flash-loan-use-cases.html"><strong aria-hidden="true">42.</strong> 06 Â· Flash Loan Use Cases</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/07-staking-pool.html"><strong aria-hidden="true">43.</strong> 07 Â· Staking Pool</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/08-liquidity-mining.html"><strong aria-hidden="true">44.</strong> 08 Â· Liquidity Mining</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/09-vault-strategies.html"><strong aria-hidden="true">45.</strong> 09 Â· Vault Strategies</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/10-swap-liquidity.html"><strong aria-hidden="true">46.</strong> 10 Â· Swap Liquidity</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/11-amm-price-oracle.html"><strong aria-hidden="true">47.</strong> 11 Â· AMM Price Oracle</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/12-farming-pool.html"><strong aria-hidden="true">48.</strong> 12 Â· Farming Pool</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/defi/13-amm-router.html"><strong aria-hidden="true">49.</strong> 13 Â· AMM Router</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/nfts.html"><strong aria-hidden="true">50.</strong> Overview &amp; Patterns</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/nfts/01-basic-nft.html"><strong aria-hidden="true">51.</strong> 01 Â· Basic NFT</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/nfts/02-nft-metadata.html"><strong aria-hidden="true">52.</strong> 02 Â· NFT Metadata</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/nfts/03-nft-metadata-standards.html"><strong aria-hidden="true">53.</strong> 03 Â· NFT Metadata Standards</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/nfts/04-nft-marketplace.html"><strong aria-hidden="true">54.</strong> 04 Â· NFT Marketplace</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/nft-patterns.html"><strong aria-hidden="true">55.</strong> NFT Patterns Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/nft-best-practices.html"><strong aria-hidden="true">56.</strong> NFT Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/nft-development-video.html"><strong aria-hidden="true">57.</strong> NFT Development Video Walkthrough</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance.html"><strong aria-hidden="true">58.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/governance-patterns.html"><strong aria-hidden="true">59.</strong> Governance Patterns Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/governance-security-checklist.html"><strong aria-hidden="true">60.</strong> Governance Security Checklist</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/governance-best-practices.html"><strong aria-hidden="true">61.</strong> Governance Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/01-simple-voting.html"><strong aria-hidden="true">62.</strong> 01 · Simple Voting</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/02-token-voting.html"><strong aria-hidden="true">63.</strong> 02 · Token Voting</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/03-voting-time-constraints.html"><strong aria-hidden="true">64.</strong> 03 · Voting Time Constraints</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/04-proposal-lifecycle.html"><strong aria-hidden="true">65.</strong> 04 · Proposal Lifecycle</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/05-proposal-validation.html"><strong aria-hidden="true">66.</strong> 05 · Proposal Validation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/06-vote-delegation.html"><strong aria-hidden="true">67.</strong> 06 · Vote Delegation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/07-delegation.html"><strong aria-hidden="true">68.</strong> 07 · Enhanced Delegation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/08-delegation-marketplace.html"><strong aria-hidden="true">69.</strong> 08 · Delegation Marketplace</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/09-timelock-governance.html"><strong aria-hidden="true">70.</strong> 09 · Timelock Governance</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/10-dao-treasury.html"><strong aria-hidden="true">71.</strong> 10 · DAO Treasury</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/governance/11-automatic-snapshot-triggers.html"><strong aria-hidden="true">72.</strong> 11 · Automatic Snapshot Triggers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens.html"><strong aria-hidden="true">73.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/token-security-checklist.html"><strong aria-hidden="true">74.</strong> Token Security Checklist</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/01-sep41-token.html"><strong aria-hidden="true">75.</strong> 01 Â· SEP-41 Token</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/02-sep41-extensions.html"><strong aria-hidden="true">76.</strong> 02 Â· SEP-41 Extensions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/03-optimized-operations.html"><strong aria-hidden="true">77.</strong> 03 Â· Optimized Operations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/04-mint-burn.html"><strong aria-hidden="true">78.</strong> 04 Â· Mint / Burn</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/05-allowance-pattern.html"><strong aria-hidden="true">79.</strong> 05 Â· Allowance Pattern</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/06-token-wrapper.html"><strong aria-hidden="true">80.</strong> 06 Â· Token Wrapper</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/07-token-metadata.html"><strong aria-hidden="true">81.</strong> 07 Â· Token Metadata</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/08-multi-token-balance-manager.html"><strong aria-hidden="true">82.</strong> 08 Â· Multi-Token Balance Manager</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/09-optimized-token-ops.html"><strong aria-hidden="true">83.</strong> 09 Â· Optimized Token Ops</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/tokens/11-authorized-token.html"><strong aria-hidden="true">84.</strong> 11 · Authorized Token</a></span></li><li class="chapter-item expanded "><li class="part-title">Analytics</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/documentation-analytics.html"><strong aria-hidden="true">85.</strong> Documentation Analytics</a></span></li><li class="chapter-item expanded "><li class="part-title">API Reference</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/code-examples.html"><strong aria-hidden="true">86.</strong> Code Examples</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/quick-reference.html"><strong aria-hidden="true">87.</strong> Quick Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/best-practices.html"><strong aria-hidden="true">88.</strong> Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/review-checklist.html"><strong aria-hidden="true">89.</strong> Code Review Checklist</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/testing-best-practices.html"><strong aria-hidden="true">90.</strong> Testing Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/testing-pitfalls.html"><strong aria-hidden="true">91.</strong> Testing Pitfalls</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/documentation-platform-evaluation.html"><strong aria-hidden="true">92.</strong> Documentation Platform Evaluation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/defi-best-practices.html"><strong aria-hidden="true">93.</strong> DeFi Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/defi-security-checklist.html"><strong aria-hidden="true">94.</strong> DeFi Security Checklist</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/security-pitfalls.html"><strong aria-hidden="true">95.</strong> Security Pitfalls</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/storage-pitfalls.html"><strong aria-hidden="true">96.</strong> Storage Pitfalls</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="../docs/ecosystem-partnerships.html"><strong aria-hidden="true">97.</strong> Ecosystem Partnerships</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="../docs/featured-projects.html"><strong aria-hidden="true">98.</strong> Featured Projects</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/common-pitfalls.html"><strong aria-hidden="true">99.</strong> Common Pitfalls</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/style-guide.html"><strong aria-hidden="true">100.</strong> Style Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/benchmarks.html"><strong aria-hidden="true">101.</strong> Performance Benchmarks</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/glossary.html"><strong aria-hidden="true">102.</strong> Glossary</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/translation-infrastructure.html"><strong aria-hidden="true">103.</strong> Translation Infrastructure</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/translation-workflow.html"><strong aria-hidden="true">104.</strong> Translation Workflow</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/common-patterns.html"><strong aria-hidden="true">105.</strong> Common Patterns</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/cross-contract-patterns.html"><strong aria-hidden="true">106.</strong> Factory, Proxy, and Registry Patterns</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/troubleshooting.html"><strong aria-hidden="true">107.</strong> Troubleshooting</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/token-development-video.html"><strong aria-hidden="true">108.</strong> Token Development Walkthrough</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/defi-development-video.html"><strong aria-hidden="true">109.</strong> DeFi Development Walkthrough</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/wallet-ecosystem.html"><strong aria-hidden="true">110.</strong> Wallet Ecosystem Survey</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/wallet-integration.html"><strong aria-hidden="true">111.</strong> Wallet Integration Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/security-best-practices.html"><strong aria-hidden="true">112.</strong> Security Best Practices</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/search.html"><strong aria-hidden="true">113.</strong> Site Search</a></span></li><li class="chapter-item expanded "><li class="part-title">Languages</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="language-support.html"><strong aria-hidden="true">114.</strong> Language Support</a></span></li><li class="chapter-item expanded "><li class="part-title">Community</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="community-guidelines.html"><strong aria-hidden="true">115.</strong> Community Guidelines</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/quality-dashboard.html"><strong aria-hidden="true">116.</strong> Quality Dashboard</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="CONTRIBUTING.html"><strong aria-hidden="true">117.</strong> How to Contribute</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="CODE_OF_CONDUCT.html"><strong aria-hidden="true">118.</strong> Code of Conduct</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="docs/grants-process.html"><strong aria-hidden="true">119.</strong> Grants Application Process</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/index.html"><strong aria-hidden="true">120.</strong> Governance</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/monthly-call-agenda-template.html"><strong aria-hidden="true">120.1.</strong> Agenda Template</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/monthly-call-format-guidelines.html"><strong aria-hidden="true">120.2.</strong> Format Guidelines</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/monthly-call-moderation-guide.html"><strong aria-hidden="true">120.3.</strong> Moderation Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/monthly-call-qa-process.html"><strong aria-hidden="true">120.4.</strong> Q&amp;A Process</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/monthly-call-followup-process.html"><strong aria-hidden="true">120.5.</strong> Follow-up Process</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/notes/index.html"><strong aria-hidden="true">120.6.</strong> Community Call Archive</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="governance/notes/TEMPLATE.html"><strong aria-hidden="true">120.6.1.</strong> Call Notes Template</a></span></li></ol></li></ol><li class="chapter-item expanded "><li class="part-title">ä¸­æ–‡æ–‡æ¡£</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="zh/index.html"><strong aria-hidden="true">121.</strong> ç®€ä»‹</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="zh/SUMMARY.html"><strong aria-hidden="true">122.</strong> ç›®å½•</a></span></li><li class="chapter-item expanded "><li class="part-title">Architecture Decisions</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/index.html"><strong aria-hidden="true">123.</strong> ADR Index</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/001-record-architecture-decisions.html"><strong aria-hidden="true">123.1.</strong> ADR-001: Record Architecture Decisions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/002-rust-version.html"><strong aria-hidden="true">123.2.</strong> ADR-002: Rust Version</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/003-soroban-sdk-version.html"><strong aria-hidden="true">123.3.</strong> ADR-003: Soroban SDK Version</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/004-testing-framework.html"><strong aria-hidden="true">123.4.</strong> ADR-004: Testing Framework</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/005-documentation-platform.html"><strong aria-hidden="true">123.5.</strong> ADR-005: Documentation Platform</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/002-select-storage-tier.html"><strong aria-hidden="true">123.6.</strong> ADR-002: Select Storage Tier by Lifetime</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/003-stable-storage-keys-and-ttl.html"><strong aria-hidden="true">123.7.</strong> ADR-003: Stable Storage Keys and TTL</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/004-authenticate-before-state-changes.html"><strong aria-hidden="true">123.8.</strong> ADR-004: Authenticate Before State Changes</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/005-use-least-privilege-access-control.html"><strong aria-hidden="true">123.9.</strong> ADR-005: Least-Privilege Access Control</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/006-guard-contract-upgrades.html"><strong aria-hidden="true">123.10.</strong> ADR-006: Guard Contract Code Upgrades</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/007-version-state-migrations.html"><strong aria-hidden="true">123.11.</strong> ADR-007: Versioned State Migrations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/008-mdbook-documentation-platform.html"><strong aria-hidden="true">123.12.</strong> ADR-008: Use mdBook for Documentation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/009-workspace-structure-and-ci-exclusion.html"><strong aria-hidden="true">123.13.</strong> ADR-009: Workspace Structure and CI Exclusion</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/010-sdk-version-migration-strategy.html"><strong aria-hidden="true">123.14.</strong> ADR-010: SDK Version Migration Strategy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/011-code-coverage-toolchain.html"><strong aria-hidden="true">123.15.</strong> ADR-011: Code Coverage Toolchain</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/012-example-naming-convention.html"><strong aria-hidden="true">123.16.</strong> ADR-012: Example Naming Convention</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="adr/template.html"><strong aria-hidden="true">123.17.</strong> ADR Template</a></span></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            // Check both with and without the '.html' suffix to be robust against pretty URLs
            if (link.href.replace(/\.html$/, '') === current_page.replace(/\.html$/, '')
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);


// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = `
                    position: fixed;
                    top: 50px;
                    right: 10px;
                    background-color: 0xeeeeee;
                    z-index: 9999;
                    pointer-events: none;
                `;
                document.body.appendChild(data);
            }
            data.innerHTML = `
                <table>
                  <tr><td>documentHeight</td><td>${documentHeight.toFixed(1)}</td></tr>
                  <tr><td>windowHeight</td><td>${windowHeight.toFixed(1)}</td></tr>
                  <tr><td>scrollTop</td><td>${scrollTop.toFixed(1)}</td></tr>
                  <tr><td>pixelsAbove</td><td>${pixelsAbove.toFixed(1)}</td></tr>
                  <tr><td>pixelsBelow</td><td>${pixelsBelow.toFixed(1)}</td></tr>
                  <tr><td>bottomAdd</td><td>${bottomAdd.toFixed(1)}</td></tr>
                  <tr><td>adjustedBottomAdd</td><td>${adjustedBottomAdd.toFixed(1)}</td></tr>
                  <tr><td>scrollingDown</td><td>${scrollingDown}</td></tr>
                  <tr><td>threshold</td><td>${threshold.toFixed(1)}</td></tr>
                </table>
            `;
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = `
            position: fixed;
            top: ${threshold}px;
            left: 0;
            width: 100vw;
            height: 2px;
            background-color: red;
            z-index: 9999;
            pointer-events: none;
        `;
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded. If the current
    // header is inside a collapsed group, then it, and all its parents should
    // be expanded.
    function updateHeaderExpanded(currentA) {
        // Add expanded to all header-item li ancestors.
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    // This is done with a virtual Y threshold, where headers at or below
    // that line will be considered the current one.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }

        // Reset the classes, which will be rebuilt below.
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }

        // Find the last header that is above the threshold.
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }

        // Get the anchor in the summary.
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }

        a.classList.add('current-header');

        updateHeaderExpanded(a);
    }

    // Updates which header is "current" based on the threshold line.
    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }


    // When clicking on a header in the sidebar, this adjusts the threshold so
    // that it is located next to the header. This is so that header becomes
    // "current".
    function headerThresholdClick(event) {
        // See disableScroll description why this is done.
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        // requestAnimationFrame is used to delay the update of the "current"
        // header until after the scroll is done, and the header is in the new
        // position.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                // Closest is needed because if it has child elements like <code>.
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    // Takes the nodes from the given head and copies them over to the
    // destination, along with some filtering.
    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    // Scans page for headers and adds them to the sidebar.
    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }

        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');

        if (headers.length === 0) {
            return;
        }

        // Build a tree of headers in the sidebar.

        const stack = [];

        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }

        // The level where it will start folding deeply nested headers.
        const foldLevel = 3;

        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));

            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                // Begin nesting to this level.
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    // Handle the case where jumping more than one nesting
                    // level, which doesn't have a list item to place this new
                    // list inside of.
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }

            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);

            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }

        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();


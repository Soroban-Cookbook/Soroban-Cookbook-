# Stellar Developer Documentation Feature Request: Soroban Cookbook

This document serves as the formal proposal and outreach package for featuring the **Soroban Cookbook** in the official [Stellar Developer Documentation](https://developers.stellar.org).

---

## 📌 Executive Summary

The **Soroban Cookbook** is a comprehensive, production-grade learning resource and smart contract pattern library for Soroban developers. It is built to complement the official Stellar Developer Portal by providing developers with complete, self-contained, and thoroughly tested Rust examples.

By featuring the Soroban Cookbook in the official developer documentation, the Stellar community will benefit from immediate, practical blueprints for common smart contract requirements, speeding up the time-to-market for dApps, DeFi protocols, and other Web3 services on Stellar.

---

## 💡 Value Proposition

The Soroban Cookbook fills the gap between low-level SDK references and full-fledged production dApp architectures by offering:

1. **Concrete, Real-World Examples:** Over 20+ examples ranging from basic storage patterns and auth context to advanced multi-party authorization and DeFi protocol templates.
2. **Quality & Test-Driven Focus:** Every example features unit and integration tests with >90% coverage, verified on every pull request via rigorous CI.
3. **Interactive Playgrounds:** The repository includes a companion [webapp playground](./webapp/) built with Next.js and TypeScript, enabling developers to interact with and simulate contracts directly.
4. **Comprehensive Style Guides & Best Practices:** Dedicated guidelines on error handling, gas benchmarking, storage optimizations, and overall smart contract design patterns.

---

## 🗺️ Proposed Integration Points

We propose integrating references and links to the Soroban Cookbook in the following areas of the official Stellar developer documentation:

*   **Getting Started with Smart Contracts:**
    *   *Location:* `developers.stellar.org/docs/smart-contracts/getting-started`
    *   *Details:* Suggesting the Cookbook as a secondary practical companion for writing and testing contracts.
*   **Developer Resources / Community Libraries:**
    *   *Location:* `developers.stellar.org/docs/tools/developer-tools` or community resource indexes.
    *   *Details:* Including a direct link to the repository and mdBook output as a featured pattern library.
*   **Advanced Smart Contract Guides:**
    *   *Location:* Alongside SDK references for authorization (`require_auth`) and storage patterns.
    *   *Details:* Cross-linking to our [Storage Patterns Guide](./examples/basics/02-storage-patterns/) and [Authorization Context Guide](./examples/basics/05-auth-context/).

---

## 🔗 Quick Links

For easy review by the Stellar documentation team, these are the main entry points of the Soroban Cookbook:

*   **GitHub Repository:** [Soroban Cookbook Git Repo](https://github.com/Soroban-Cookbook/Soroban-Cookbook-)
*   **Core Documentation (mdBook):** Deployments are automatically built on every commit to `main`.
*   **Getting Started Guide:** [Getting Started Tutorial](./book/src/guides/getting-started.md)
*   **Testing Reference:** [Testing Guide](./book/src/guides/testing.md)
*   **Style Guide:** [Style & Conventions Guide](./docs/style-guide.md)
*   **Roadmap:** [Project Roadmap & Roadmap Phases](./ROADMAP.md)

---

## 🤝 Contributor & Maintenance Guidelines

To guarantee the quality and longevity of this featured resource, the Soroban Cookbook team commits to the following:

*   **SDK Compatibility:** Examples are continuously reviewed and updated against stable releases of the Soroban/Stellar SDK.
*   **Open Contributions:** Anyone in the ecosystem can contribute new templates or improve documentation. Guidelines are outlined in [CONTRIBUTING.md](./CONTRIBUTING.md).
*   **Quality Gates:** No pull request is merged without passing formatting (`cargo fmt`), lint checks (`cargo clippy -- -D warnings`), and a complete test suite check.

---

## 📢 Outreach & Announcement Plan

To maximize the visibility of this integration, the following announcements will be scheduled upon approval:

### 1. Stellar Discord Announcement (`#announcements` or `#soroban`)
```markdown
🚀 We are excited to announce that the **Soroban Cookbook** is now officially featured in the Stellar Developer Documentation! 

The Soroban Cookbook is a community-driven repository filled with tested, ready-to-use smart contract patterns. From basic storage strategies to advanced governance structures, find your next template today:
👉 https://github.com/Soroban-Cookbook/Soroban-Cookbook-

A big thank you to the SDF documentation team and all contributors who helped align this resource with official standards. Check out our contributing guide to add your own pattern!
```

### 2. Social Media Announcement (X / Twitter)
```text
Building on Stellar? 🛠️ 

The Soroban Cookbook is now featured in the official Stellar developer docs! 

Access 20+ fully tested, production-grade Rust smart contract patterns, guides, and benchmarking recipes.

Speed up your Soroban build today:
👉 https://github.com/Soroban-Cookbook/Soroban-Cookbook- 

#Stellar #Soroban #Rust #Web3
```

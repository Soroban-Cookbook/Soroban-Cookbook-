# Community Onboarding Guide

Welcome to the **Soroban Cookbook** community! This guide will help you go from zero to productive contributor as quickly as possible, regardless of your background.

---

## Table of Contents

- [Who This Is For](#who-this-is-for)
- [What Is the Soroban Cookbook?](#what-is-the-soroban-cookbook)
- [Learning Paths](#learning-paths)
  - [Path A: New to Stellar & Soroban](#path-a-new-to-stellar--soroban)
  - [Path B: Rust Developer, New to Soroban](#path-b-rust-developer-new-to-soroban)
  - [Path C: Blockchain Developer, New to Stellar](#path-c-blockchain-developer-new-to-stellar)
  - [Path D: Ready to Contribute](#path-d-ready-to-contribute)
- [First Steps Checklist](#first-steps-checklist)
- [Community Channels](#community-channels)
- [How to Ask for Help](#how-to-ask-for-help)
- [External Resources](#external-resources)
- [Frequently Asked Questions](#frequently-asked-questions)

---

## Who This Is For

This guide is for anyone joining the Soroban Cookbook community:

- Developers discovering Stellar and Soroban for the first time
- Rust engineers exploring smart contract development
- Solidity/EVM developers migrating to Soroban
- Technical writers and documentation contributors
- Reviewers, issue triagers, and community helpers

No prior Soroban or blockchain experience is required to get started.

---

## What Is the Soroban Cookbook?

The Soroban Cookbook is a community-maintained collection of production-ready smart contract recipes, patterns, and guides for building on the [Stellar](https://stellar.org) network with [Soroban](https://developers.stellar.org/docs/smart-contracts).

Think of it as a practical reference alongside the official docs — every example compiles, is fully tested, and is designed to teach a specific concept or pattern. Contributions are welcome at every level, from fixing a typo to adding an advanced DeFi protocol.

---

## Learning Paths

### Path A: New to Stellar & Soroban

If you have never worked with Stellar or Soroban before, start here.

1. **Understand Stellar basics**
   - Read the [Stellar Overview](https://developers.stellar.org/docs/learn/fundamentals) to understand accounts, assets, and transactions.
   - Learn what [Soroban](https://developers.stellar.org/docs/smart-contracts) is and how it differs from other smart contract platforms.

2. **Set up your environment**
   - Follow the [Getting Started guide](../book/src/guides/getting-started.md) in this repo.
   - Install Rust, the WASM target, and the Stellar CLI (see [Installation](../README.md#installation)).

3. **Run your first contract**
   - Open `examples/basics/01-hello-world/` and run `cargo test`.
   - Read the inline comments to understand what each part does.

4. **Work through the basics**
   - Complete examples `01` through `05` in `examples/basics/` in order.
   - Each one builds on the previous; pay attention to the README in each folder.

5. **Check the glossary**
   - [docs/glossary.md](./glossary.md) explains Soroban-specific terms you will encounter.

---

### Path B: Rust Developer, New to Soroban

You know Rust but have not written a Soroban contract before.

1. **Skim the [Soroban SDK docs](https://docs.rs/soroban-sdk)** to get a sense of the APIs.
2. **Read [docs/best-practices.md](./best-practices.md)** — many Soroban patterns differ from standard Rust.
3. **Jump to `examples/basics/02-storage-patterns/`** — storage is the concept that surprises Rust devs most.
4. **Read [docs/storage-types.md](./storage-types.md)** for a concise breakdown of `Persistent`, `Instance`, and `Temporary`.
5. **Look at `examples/intermediate/`** for cross-contract calls and access control patterns.
6. **Review [docs/testing-guide.md](./testing-guide.md)** to understand the Soroban test environment.

---

### Path C: Blockchain Developer, New to Stellar

You are experienced with EVM (Solidity, Foundry, Hardhat) or another chain.

1. **Read the [Ethereum to Soroban guide](../book/src/guides/ethereum-to-soroban.md)** for a direct pattern translation reference.
2. **Key differences to be aware of:**
   - No `msg.sender` — use `env.current_contract_address()` and explicit `require_auth()` patterns.
   - No implicit reentrancy guards — authorization and state changes must be reasoned about explicitly.
   - Storage has three distinct tiers with TTL; see [docs/storage-types.md](./storage-types.md).
   - Gas is called "fees" and metered differently; see [docs/benchmarks.md](./benchmarks.md).
3. **Explore DeFi examples** in `examples/defi/` to see familiar patterns in a Soroban context.
4. **Read [docs/security-best-practices.md](./security-best-practices.md)** before writing any contract you plan to deploy.

---

### Path D: Ready to Contribute

You are comfortable with the codebase and want to submit a contribution.

1. **Read [CONTRIBUTING.md](../CONTRIBUTING.md)** in full — it covers branching, commit style, testing requirements, and the PR process.
2. **Browse open issues** on GitHub. Issues labelled `good first issue` are reserved for new contributors.
3. **Check the [ROADMAP.md](../ROADMAP.md)** to see what the maintainers are planning next.
4. **Follow the validation checklist** before opening a PR:
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace
   cargo build --workspace --target wasm32-unknown-unknown --release
   ```
5. **Fill out the PR template completely.** PRs without a clear description of what changed and why are likely to be held for revision.

---

## First Steps Checklist

Use this as a quick checklist when you first join:

- [ ] Star and fork the repository on GitHub
- [ ] Clone your fork locally and run `cargo test --workspace` to verify your setup
- [ ] Join the [Stellar Community Discord](https://discord.gg/stellardev) and say hello in `#soroban`
- [ ] Read the [Code of Conduct](../CODE_OF_CONDUCT.md)
- [ ] Browse open issues and pick one labelled `good first issue` or `documentation`
- [ ] Introduce yourself in the issue or discussion before starting work to avoid duplication
- [ ] Open your first PR — even a small doc fix counts!

---

## Community Channels

| Channel | Purpose | Link |
|---------|---------|------|
| GitHub Issues | Bug reports, feature requests, task tracking | [Issues](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues) |
| GitHub Discussions | Design questions, RFCs, general Q&A | [Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/discussions) |
| Stellar Discord `#soroban` | Real-time help, announcements, community chat | [Discord](https://discord.gg/stellardev) |
| GitHub Pull Requests | Code review and collaborative development | [Pull Requests](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/pulls) |

**Response times:** Maintainers aim to triage new issues within 3 business days and review pull requests within 7 business days.

---

## How to Ask for Help

Getting a useful answer quickly depends on how the question is framed.

**Good question format:**
1. **What you are trying to do** — describe the goal, not just the error.
2. **What you tried** — paste the relevant code or command.
3. **What happened** — include the full error message or unexpected output.
4. **Environment** — Rust version (`rustc --version`), Stellar CLI version (`stellar --version`), OS.

**Where to ask:**
- For questions about a specific example or pattern → open a GitHub Discussion or comment on the relevant issue.
- For general Soroban questions → the `#soroban` channel on Stellar Discord has both maintainers and a broad community.
- For suspected bugs → open a GitHub Issue and use the bug report template.
- For security issues → **do not open a public issue.** See the [Bug Bounty Program](../CONTRIBUTING.md#bug-bounty-program) for responsible disclosure details.

---

## External Resources

### Official Documentation

| Resource | URL |
|----------|-----|
| Soroban Smart Contract Docs | https://developers.stellar.org/docs/smart-contracts |
| Stellar Developer Portal | https://developers.stellar.org |
| Soroban Rust SDK (docs.rs) | https://docs.rs/soroban-sdk |
| Stellar CLI Reference | https://developers.stellar.org/docs/tools/developer-tools/cli |
| Soroban SDK on GitHub | https://github.com/stellar/rs-soroban-sdk |

### Learning Resources

| Resource | URL |
|----------|-----|
| Stellar Quest (hands-on challenges) | https://quest.stellar.org |
| Stellar Learn (tutorials & courses) | https://developers.stellar.org/docs/learn |
| Soroban Playground (browser IDE) | https://soroban.stellardev.org |

### Tooling

| Tool | URL |
|------|-----|
| Stellar Laboratory (testnet explorer & tool) | https://laboratory.stellar.org |
| StellarExpert (mainnet block explorer) | https://stellar.expert |
| Freighter Wallet (browser extension) | https://freighter.app |

---

## Frequently Asked Questions

**Q: I am not a Rust developer. Can I still contribute?**
Yes. Documentation improvements, example README edits, glossary additions, and guide writing are all valuable contributions that require no Rust experience. Browse issues tagged `documentation` to find starting points.

**Q: Do I need a Stellar account or XLM to contribute?**
No. All development and testing uses the local Soroban environment or testnet. You never need mainnet funds to contribute to this repo.

**Q: How long does PR review take?**
The team aims to provide initial feedback within 7 business days. Complex examples or architectural changes may take longer. Ping the issue or PR if you haven't heard back after 10 business days.

**Q: Can I contribute an example in a new category not listed in the repo?**
Yes — open a GitHub Discussion or issue first to propose the category. Maintainers will give early feedback to help shape the contribution before you invest significant time.

**Q: Is this an official Stellar Foundation project?**
The Soroban Cookbook is a community project. It is featured by the Stellar Development Foundation as a community resource, but it is independently maintained. See [README.md](../README.md#community--integration) for the full relationship description.

**Q: Where is the recognition for contributors documented?**
See [docs/recognition-system.md](./recognition-system.md) and [CONTRIBUTORS.md](../CONTRIBUTORS.md) for the contributor tiers, rewards, and how contributions are credited.

---

*This guide is a living document. If you find anything out of date or missing, please open a PR to improve it — that counts as a contribution too.*

# Contributing to Soroban-Cookbook

> 🎉 **Phase 8 (Community & Ecosystem) Complete!** See our [Phase 8 Completion Report](./PHASE_8_COMPLETION_REPORT.md) and join the celebration event!

Before participating, please read our [Community Guidelines](./COMMUNITY_GUIDELINES.md) and [Code of Conduct](./CODE_OF_CONDUCT.md).

## Built With the Cookbook

We showcase **10+ real production projects** built using the Soroban Cookbook in
our [Showcase](./SHOWCASE.md). It includes featured projects, case studies, and a
developer-support section for anyone building on Soroban.

**Built a project with the cookbook?** Open a pull request that adds your project
to `SHOWCASE.md` (name, repo link, one-line description, and which cookbook
patterns you used). This also helps us keep the Phase 8 "#441: 10+ Projects Built"
milestone current.

## Feedback System

We have implemented a comprehensive feedback system to collect and manage input from our community. The system is located in the `docs/feedback-system/` directory.

### How to Provide Feedback

1. **Use the Feedback Form**: Copy the template from `docs/feedback-system/forms/feedback-form-template.md` and fill it out.
2. **Submit via GitHub**: Create an issue or pull request with your feedback.
3. **Participate in Surveys**: Copy and fill out our [Community Survey Template](./docs/feedback-system/surveys/USER_SURVEY_TEMPLATE.md) or join external survey links.

### Community & Feedback Channels

To make providing feedback as easy and integrated as possible, you can access the following channels:
- **Quarterly Surveys**: We run regular user surveys to gather structured feedback on cookbook clarity, missing examples, and environment setup ease. See our [User Surveys Documentation & Process](./docs/feedback-system/surveys/README.md).
- **Google Forms Survey**: Submit quick structured feedback online via our [Google Forms Survey Link](https://forms.google.com/soroban-cookbook-community-survey).
- **GitHub Discussions**: Share ideas and participate in community polls in the [Discussions Forums](https://github.com/gloriaibrahim2002-blip/Soroban-Cookbook-/discussions).
- **Discord Community**: Chat live with maintainers and other Soroban developers in the `#soroban` channel on [Stellar Discord](https://discord.gg/stellardev).

### Review Process

All feedback and contributions go through our review process:
1. Initial acknowledgment within 2 business days
2. Content and quality review
3. Decision and communication
4. Action tracking and implementation

### Action Tracking

We track all feedback-driven actions using:
- GitHub Issues for individual tasks
- Project boards for status visualization
- Regular status updates in our community channels

### Communication

We commit to:
- Acknowledging all feedback within 2 business days
- Providing regular status updates
- Closing the loop on all submitted feedback

For more details, see the [Feedback System Documentation](docs/feedback-system/README.md).

## Monthly Community Call Governance

The Soroban Cookbook runs monthly community calls to share progress, demo examples, and answer questions. All call templates and governance documents are maintained in the [GOVERNANCE/](./GOVERNANCE/) directory:

- [Agenda Template](./GOVERNANCE/monthly-call-agenda-template.md)
- [Format Guidelines](./GOVERNANCE/monthly-call-format-guidelines.md)
- [Moderation Guide](./GOVERNANCE/monthly-call-moderation-guide.md)
- [Q&A Process](./GOVERNANCE/monthly-call-qa-process.md)
- [Follow-up Process](./GOVERNANCE/monthly-call-followup-process.md)

To propose a topic, moderate a session, or improve the process, open an issue or pull request in this repository.

## Project Templates

We provide three complete full-stack starter templates in the [`templates/`](./templates/) directory to help developers build and launch dApps rapidly:

- [Templates Overview & Guide](./templates/README.md)
- [🪙 Fungible Token dApp](./templates/token-dapp/)
- [🎨 NFT Marketplace dApp](./templates/nft-marketplace-dapp/)
- [🏛️ DAO Governance & Treasury dApp](./templates/dao-governance-dapp/)

To contribute a new template or improve an existing one, check out the [Project Templates Guide](./docs/project-templates.md).

---

## 📍 Table of Contents
- [New Here? Start with Onboarding](#-new-here-start-with-onboarding)
- [Monthly Community Call Governance](#monthly-community-call-governance)
- [Grants Application Process](#grants-application-process)
- [Project Templates](#project-templates)
- [Ways to Contribute](#-ways-to-contribute)
- [Development Environment Setup](#️-development-environment-setup)
- [Code Style Guidelines](#-code-style-guidelines)
- [Project Structure](#️-project-structure)
- [Pull Request Process](#-pull-request-process)
- [Testing Requirements](#-testing-requirements)
- [Example Contribution Template](#-example-contribution-template)
- [Validation Steps](#-validation-steps)
- [Bug Bounty Program](#-bug-bounty-program)
- [Recognition System](#-recognition-system)
- [Community Metrics](#-community-metrics)

## 👋 New Here? Start with Onboarding

If this is your first time in the Soroban Cookbook community, head to **[docs/onboarding.md](./docs/onboarding.md)** before reading the rest of this file.

The onboarding guide covers:

- Learning paths tailored to your background (new to Stellar, Rust dev, EVM dev, or ready to contribute)
- A first-steps checklist to get your environment running and your first PR open
- Community channels and how to ask for help effectively
- External resources — official docs, tools, and learning materials
- Answers to common first-timer questions

Once you have worked through the onboarding guide and run `cargo test --workspace` successfully, come back here for the full contribution process.

---

## 📊 Community Metrics

The Soroban Cookbook tracks community health transparently so contributors can see
the impact of their work and maintainers can identify friction early.

| Resource | Description |
|---|---|
| [Community Metrics](./docs/community-metrics.md) | Full metric definitions, targets, and alert thresholds |
| [Community Dashboard](./docs/community-dashboard.md) | Rolling weekly data table + monthly narrative reports |
| [Automated Workflow](./.github/workflows/community-metrics.yml) | GitHub Actions job that collects data every Monday |

### What We Track

- **Growth** — new contributors, stars, forks, unique clone traffic
- **Engagement** — issue response time, PR review velocity, discussion activity
- **Content Quality** — CI pass rate, test coverage, Clippy warnings
- **Community Health** — bus factor, contributor retention, CoC incidents
- **Documentation** — feedback submissions, satisfaction scores

### How You Contribute to the Numbers

Every merged PR, answered issue, and submitted feedback form flows into these metrics
and feeds the [Recognition System](./docs/recognition-system.md). You can:

- Participate in quarterly satisfaction surveys (announced in GitHub Discussions).
- Flag data anomalies by opening an issue tagged `metrics-anomaly`.
- Propose new metrics in a GitHub Discussion under the `Ideas` category.

---

## 🎯 Ways to Contribute

## 🧪 Testing Requirements

Every change that touches contract logic needs test coverage before it merges. This section covers the commands you need day-to-day; for testing patterns, fixtures, and best practices, see the [Testing Guide](./book/src/guides/testing.md) and [Testing Best Practices](./book/src/docs/testing-best-practices.md).

### Running Tests

```bash
# Run everything in the workspace (what CI runs by default)
cargo test --workspace

# Run tests for a single example you're working on
cd examples/<category>/<example-name>
cargo test

# Run the shared integration test suite
cargo test -p integration-tests

# Run the security-focused test suite
cargo test --package security-tests

# Run the audit-prep documentation consistency checks
cargo test -p docs-audit-tests
```

CI also runs `cargo fmt --all -- --check` and `cargo clippy --tests --lib -- -D warnings` — run both locally before opening a PR to avoid a red pipeline.

### Where to Add a Test

- **Example-level test** (`src/test.rs` or `tests/` inside the example crate): the default for any new or changed example. If the behavior only involves that one contract, it belongs here.
- **Shared integration test** (`tests/integration/`): for scenarios that span multiple contracts — cross-contract calls, multi-step workflows, or state coordination between examples. See `tests/integration/README.md` for existing patterns before adding a new one.
- **Security test** (`tests/security/`): for authorization bypass, reentrancy, and other security-relevant regressions. See `tests/security/README.md` for the threat models already covered.
- **Fuzz target** (`tests/fuzz/`): for property-based testing of parsing/serialization boundaries. Only needed when you're adding a new fuzzable surface, not for typical example changes.
- **Documentation consistency** (`tests/docs-audit/`): for checks that a maintained document (e.g. an audit-scope or checklist file) stays in sync with the filesystem it describes. See `tests/docs-audit/README.md`.

Coverage is measured with `cargo tarpaulin` in CI and uploaded to Codecov; run `cargo tarpaulin` locally if you want to check coverage before pushing.

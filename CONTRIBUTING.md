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

## Ecosystem Partnerships

We collaborate with other Soroban projects so the patterns documented here are
shaped by the teams actually shipping contracts. The full picture — partner
projects, active collaborations, joint content, cross-promotion, and how to join
— lives in [docs/ecosystem-partnerships.md](./docs/ecosystem-partnerships.md).

For information about our outreach to the Stellar Development Foundation, see [docs/stellar-foundation-contact.md](./docs/stellar-foundation-contact.md).

**Current collaboration tracks:**

| Collaboration | What it produces |
|---|---|
| Streaming payments (SoroStream, StellarStream) | Shared storage trade-off case study for stream state |
| NFT marketplace (soroban-nft-marketplace) | Reviewed [NFT patterns](./docs/nft-patterns.md), [NFT best practices](./docs/nft-best-practices.md), and escrow guidance |
| Explorer and verification (soroban-verify, Soroban Smart Block Explorer) | Corrected [deployment](./book/src/guides/deployment.md) and upgrade guidance |
| Oracle safety (soroban-oracle-safety) | Staleness and sanity-bound guidance in the oracle examples |
| Wallet stack (astroid) | Integration notes in the [wallet integration guide](./docs/wallet-integration.md) |
| Monthly community call | Demos, Q&A, and announcements on the [governance cadence](./GOVERNANCE/) |

**Cross-promotion is reciprocal:** we list a project and describe precisely which
cookbook patterns it uses, and we ask for exactly one thing in return — a line in
your README linking to the page you used. Any project can ask to be removed from
all listings and we will remove it in the next pull request.

**Want to collaborate?** Open an issue describing your project, which cookbook
pages you used, and whether you want a case study, a documentation review, or a
demo slot on the monthly call. See
[How to propose a partnership](./docs/ecosystem-partnerships.md#7-how-to-propose-a-partnership).
For institutions, protocols, and developer tool providers seeking formal collaboration, review our
[Formal Partnership Proposal Framework](./docs/formal-partnership-proposal.md), which details qualification criteria, mutual commitments, and an executable MOU template.

## Featured Projects

We feature the first projects built with the cookbook, each with a case study, a
runnable demo, and an interview. The full page lives in
[docs/featured-projects.md](./docs/featured-projects.md).

| # | Project | Focus |
|---|---|---|
| 1 | [SoroStream](https://github.com/SoroStream/sorostream-contracts) | Storage-type trade-offs under a rent budget |
| 2 | [soroban-nft-marketplace](https://github.com/waveforge-labs/soroban-nft-marketplace) | Escrow authorization in marketplace settlement |
| 3 | [soroban-verify](https://github.com/SorobanVerify/soroban-verify) | Reproducible deployments and contract verification |
| 4 | [Micopay Protocol](https://github.com/Micopay/micopay-protocol) | Balances that survive TTL expiry |
| 5 | [astroid](https://github.com/ASTROIDX556) | One error vocabulary across contract, SDK, API, and UI |

**Promotion is scheduled:** a weekly demo clip, a monthly case study or interview
published on the [monthly call](./GOVERNANCE/), and a re-check of each promoted
link every release. Any project can ask to be removed from all listings.

**Want to be featured?** Build something that ships a Soroban contract, tell us
which cookbook pages you used, and we will write the case study and schedule the
demo slot. See
[How a project gets featured](./docs/featured-projects.md#7-how-a-project-gets-featured).

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

- **Schedule:** First Tuesday of every month at 16:00 UTC. Subscribe to the [monthly community call calendar](https://raw.githubusercontent.com/Soroban-Cookbook/Soroban-Cookbook-/main/GOVERNANCE/monthly-community-call.ics); calendar apps convert the UTC time to your local time and include reminders one week and one day before each call.
- **Join details:** The call platform and join link are announced for each event and added to the calendar when confirmed. Announcements include local-time conversions; schedule changes are announced at least 30 days ahead.
- [Agenda Template](./GOVERNANCE/monthly-call-agenda-template.md)
- [Format Guidelines](./GOVERNANCE/monthly-call-format-guidelines.md)
- [Moderation Guide](./GOVERNANCE/monthly-call-moderation-guide.md)
- [Q&A Process](./GOVERNANCE/monthly-call-qa-process.md)
- [Follow-up Process](./GOVERNANCE/monthly-call-followup-process.md)
- [Community Call Archive](./GOVERNANCE/notes/README.md) — searchable notes, recordings, transcripts, and action items

To propose a topic, moderate a session, or improve the process, open an issue or pull request in this repository.

## Project Templates

We provide three complete full-stack starter templates in the [`templates/`](./templates/) directory to help developers build and launch dApps rapidly:

- [Templates Overview & Guide](./templates/README.md)
- [🪙 Fungible Token dApp](./templates/token-dapp/)
- [🎨 NFT Marketplace dApp](./templates/nft-marketplace-dapp/)
- [🏛️ DAO Governance & Treasury dApp](./templates/dao-governance-dapp/)

To contribute a new template or improve an existing one, check out the [Project Templates Guide](./docs/project-templates.md).

## Workshop Materials

We provide a complete, reusable workshop curriculum for teaching Soroban development in [`docs/workshop/`](./docs/workshop/). The materials cover five modules spanning environment setup through advanced patterns and testnet deployment:

| Material | Description |
|---|---|
| [Workshop README](./docs/workshop/README.md) | Overview, agenda, prerequisites, and setup instructions |
| [Slide Decks](./docs/workshop/slides/) | Marp-compatible Markdown slides for all 5 modules |
| [Hands-on Exercises](./docs/workshop/exercises/) | Lab exercises with starter code and solutions, referencing cookbook examples |
| [Facilitator Guide](./docs/workshop/facilitator-guide.md) | Timing, talking points, common pitfalls, and Q&A prep per module |
| [Participant Guide](./docs/workshop/participant-guide.md) | Setup instructions, step-by-step workbook, and reference links |

The workshop is designed for instructor-led delivery (2 days or 4 half-day sessions) but also works for self-paced study. To adapt or contribute improvements, open a pull request modifying files under `docs/workshop/`.

---

## IDE & Tooling Integration

The repo ships a shared VS Code setup in [`.vscode/`](./.vscode/): recommended extensions, rust-analyzer running clippy with CI flags, tasks for tests, smoke tests, coverage, lint, and the book, CodeLLDB debug configs, and Soroban snippets. Setup steps for IntelliJ IDEA and RustRover, the linting tools, and the testing tools are covered in [docs/ide-integration.md](./docs/ide-integration.md).

If you change a CI lint or test command, update the matching task in `.vscode/tasks.json` and the tables in `docs/ide-integration.md` too.

---

## Wallet Support

Besides Freighter, xBull, and Albedo ([Wallet Integration Guide](./docs/wallet-integration.md)), the cookbook has tested adapter examples for **Hana**, **LOBSTR**, and **Ledger**, and shows how to use Stellar Wallets Kit for multi-wallet support. See [docs/additional-wallets.md](./docs/additional-wallets.md) for the examples, comparison guide, and testing checklist. The code is in [`docs/wallet-examples/`](./docs/wallet-examples/), and `npm test` there runs the unit tests.

To add a wallet, implement the `WalletAdapter` interface in `docs/wallet-examples/src/`, add tests to `test/adapters.test.ts`, and add a column to the comparison matrix.

---

## Roadmap Input Process

The Soroban Cookbook has a structured process for community members to propose additions to the project roadmap. See [docs/roadmap-input-process.md](./docs/roadmap-input-process.md) for:

- How to submit roadmap proposals (GitHub Discussions, issues, community calls)
- The review and triage process
- How community votes work
- The quarterly update cadence

---

## 📍 Table of Contents
- [New Here? Start with Onboarding](#-new-here-start-with-onboarding)
- [Featured Projects](#featured-projects)
- [Community Office Hours](#️-community-office-hours)
- [Monthly Community Call Governance](#monthly-community-call-governance)
- [Ecosystem Partnerships](#ecosystem-partnerships)
- [Grants Application Process](#grants-application-process)
- [Project Templates](#project-templates)
- [Workshop Materials](#workshop-materials)
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

## 🛡️ Bug Bounty Program

The Soroban Cookbook bug bounty is open for eligible security findings in
contract examples, shared utilities, tests, and security documentation. Review
the [program scope and rewards](./docs/security-audit/bug-bounty.md) and submit
reports privately through the linked GitHub Security Advisory channel. Never
post vulnerability details in a public issue.

The [launch kit](./docs/security-audit/bug-bounty-launch.md) contains the
announcement draft and outreach checklist; external publication and participant
milestones are recorded there only after verification.

---

## 📊 Community Metrics

The Soroban Cookbook tracks community health transparently so contributors can see
the impact of their work and maintainers can identify friction early.

| Resource | Description |
|---|---|
| [Community Metrics](./docs/community-metrics.md) | Full metric definitions, targets, and alert thresholds |
| [Community Dashboard](./docs/community-dashboard.md) | Rolling weekly data table + monthly narrative reports |
| [Documentation Metrics](./docs/documentation-metrics.md) | Page views, search queries, popular content, user paths |
| [Automated Workflow](./.github/workflows/community-metrics.yml) | GitHub Actions job that collects community data every Monday |
| [Docs Metrics Workflow](./.github/workflows/documentation-metrics.yml) | GitHub Actions job that collects docs usage data every Monday |

### What We Track

- **Growth** — new contributors, stars, forks, unique clone traffic
- **Engagement** — issue response time, PR review velocity, discussion activity
- **Content Quality** — CI pass rate, test coverage, Clippy warnings
- **Community Health** — bus factor, contributor retention, CoC incidents
- **Documentation** — page views, search queries, popular content, user paths, feedback submissions, satisfaction scores

### Documentation Analytics

The project uses privacy-respecting analytics to understand how developers
use the documentation:

- **mdBook** — `book/analytics.js` tracks page views, search queries, and user paths
- **Webapp** — `webapp/src/lib/analytics.ts` tracks page views and search queries

Both scripts are cookie-free, PII-free, respect Do-Not-Track, and use
`navigator.sendBeacon` for non-blocking data transmission. See the
[privacy section](./docs/documentation-metrics.md#privacy--data-handling) of
the documentation metrics dashboard for full details.

### How You Contribute to the Numbers

Every merged PR, answered issue, and submitted feedback form flows into these metrics
and feeds the [Recognition System](./docs/recognition-system.md). You can:

- Participate in quarterly satisfaction surveys (announced in GitHub Discussions).
- Flag data anomalies by opening an issue tagged `metrics-anomaly`.
- Propose new metrics in a GitHub Discussion under the `Ideas` category.

---

## 🎯 Ways to Contribute

## 📋 Submission Guidelines

All contributions must follow these guidelines:

1. **Test Coverage**: Every change touching contract logic requires test coverage (see [Testing Requirements](#-testing-requirements))
2. **Code Style**: Run `cargo fmt --all -- --check` and `cargo clippy --tests --lib -- -D warnings` locally before opening a PR
3. **Documentation**: Update relevant documentation when adding or modifying examples
4. **Commits**: Use conventional commit messages (e.g., `feat:`, `fix:`, `docs:`)
5. **PR Process**: Link related issues using `Closes #<issue-number>` in the PR description

### 🗓️ Community Office Hours

We run **regular office hours** so you can bring questions, get unblocked, and
talk through contract design with maintainers and other contributors. Office
hours complement the [monthly community call](#monthly-community-call-governance):
the call is for demos and announcements, office hours are for open Q&A.

**Schedule**

- **Weekly**, every **Thursday at 16:00 UTC** (60 minutes).
- No session on the week of the monthly community call — join the call instead.
- The next session date is always posted in the [README](./README.md#community-office-hours)
  and in GitHub Discussions under the `Office Hours` category.

**Platform**

- Hosted on **Discord** in the `#soroban` channel on
  [Stellar Discord](https://discord.gg/stellardev), with a fallback video room
  link posted in the Discussion thread before each session.
- No registration required; drop in for any part of the hour.

**Promotion**

- Announced in GitHub Discussions and the `#soroban` Discord channel at least
  48 hours ahead, with the agenda and any pre-submitted questions.
- Included in the monthly call recap and the [Community Dashboard](./docs/community-dashboard.md).

**Q&A format**

1. **Pre-submitted questions** — post in the Discussion thread or comment on the
   tracking issue; these are answered first.
2. **Live questions** — raise your hand in the channel; the moderator queues them.
3. **Deep dives** — anything needing more than ~5 minutes is moved to a follow-up
   issue or a dedicated pairing session.

See the [Q&A Process](./GOVERNANCE/monthly-call-qa-process.md) for the moderation
conventions we reuse.

**Recording and notes**

- Sessions are **not recorded by default** to keep the space open for candid
  questions; if a session is recorded, everyone is told at the start.
- Written notes, answered questions, and links are published after each session
  in the `Office Hours` Discussion category and summarized in the
  [Community Dashboard](./docs/community-dashboard.md).
- Action items become GitHub issues tagged `office-hours`.

**Want to host or propose a topic?** Open an issue describing the topic and
which cookbook pages it relates to, or volunteer as moderator in the Discussion
thread.

## 🧪 Testing Requirements

Every change that touches contract logic needs test coverage before it merges. This section covers the commands you need day-to-day; for testing patterns, fixtures, and best practices, see the [Testing Guide](./book/src/guides/testing.md) and [Testing Best Practices](./book/src/docs/testing-best-practices.md).

### Running Tests

```bash
# Run everything in the workspace (what CI runs by default)
cargo test --workspace

# Run tests for a single example you're working on
cd examples/<category>/<example-name>
cargo test

# Run the fast smoke tests (critical paths, < 2 min; runs first in CI)
cargo test -p integration-tests --test smoke_tests

# Run the shared integration test suite
cargo test -p integration-tests

# Run the security-focused test suite
cargo test --package security-tests

# Run the audit-prep documentation consistency checks
cargo test -p docs-audit-tests
```

CI also runs `cargo fmt --all -- --check` and `cargo clippy --tests --lib -- -D warnings` — run both locally before opening a PR to avoid a red pipeline.

The repository's [pre-commit hooks](./doc/pre-commit.md) run formatting and typo
checks before commits, then Clippy and integration tests before pushes. Install
both hooks during setup so these checks run automatically.

### Where to Add a Test

- **Example-level test** (`src/test.rs` or `tests/` inside the example crate): the default for any new or changed example. If the behavior only involves that one contract, it belongs here.
- **Shared integration test** (`tests/integration/`): for scenarios that span multiple contracts — cross-contract calls, multi-step workflows, or state coordination between examples. See `tests/integration/README.md` for existing patterns before adding a new one.
- **Security test** (`tests/security/`): for authorization bypass, reentrancy, and other security-relevant regressions. See `tests/security/README.md` for the threat models already covered.
- **Fuzz target** (`tests/fuzz/`): for property-based testing of parsing/serialization boundaries. Only needed when you're adding a new fuzzable surface, not for typical example changes.
- **Documentation consistency** (`tests/docs-audit/`): for checks that a maintained document (e.g. an audit-scope or checklist file) stays in sync with the filesystem it describes. See `tests/docs-audit/README.md`.

Coverage is measured with `cargo tarpaulin` in CI and uploaded to Codecov; run `cargo tarpaulin` locally if you want to check coverage before pushing.

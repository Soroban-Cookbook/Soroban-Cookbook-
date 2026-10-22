# Phase 8: Community & Ecosystem -

**Phase Status:** 📋 PLANNED (Ongoing)  
**Completion:** 18% (9/50)
**Last Audit Date:** October 19, 2026

Community and ecosystem initiatives — in-repo deliverables are tracked per issue below.

## Partnership with Stellar Foundation

### Issue #395: Propose Partnership

- **Priority:** High
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo — [`docs/formal-partnership-proposal.md`](./docs/formal-partnership-proposal.md) defines the formal partnership framework, mutual benefits across 3 stakeholder groups, partner eligibility & technical requirements, an executable Memorandum of Understanding (MOU) agreement template, onboarding lifecycle, and established partner tracking. Summarized in [`CONTRIBUTING.md`](./CONTRIBUTING.md) and linked from the [`README.md` Community Health & Metrics table](./README.md#-community-health--metrics). Tracked on GitHub as issue [#948](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/948).
- **Implementation hints:** Outcomes documented in README and `CONTRIBUTING.md`; external resources linked.
- **Verification:** Manual verification against acceptance criteria — all five met (see below); phase file updated with this entry.
- **Scope:** M
- **Description:** Formal partnership proposal
- **Acceptance Criteria:**
  - ✅ Proposal document created — [`docs/formal-partnership-proposal.md`](./docs/formal-partnership-proposal.md) provides comprehensive proposal structure.
  - ✅ Benefits outlined — Section 2 outlines value propositions for Partner Projects, Soroban Cookbook, and the Stellar/Soroban Ecosystem.
  - ✅ Requirements discussed — Section 3 details technical criteria (Soroban SDK conventions, 85%+ test coverage, gas/TTL patterns), open-source standards, and maintenance commitments.
  - ✅ Agreement drafted — Section 5 provides the full bilateral Memorandum of Understanding (MOU) legal & operational template.
  - ✅ Partnership established — Section 7 records confirmed Tier 1 and Tier 2 partners (SDF, SoroStream, StellarStream, soroban-nft-marketplace) and pipeline protocols.

### Issue #396: Collaborative content with Stellar

- **Priority:** High
- **Status:** Completed

- **Current state:** No in-repo deliverables; operational/community work tracked for planning.
- **Implementation hints:** Document outcomes in README or `CONTRIBUTING.md` when completed; link external resources.
- **Verification:** Manual verification against acceptance criteria; update phase file when done.
- **Scope:** M
- **Description:** Collaborative content with Stellar
- **Acceptance Criteria:**
  - Joint blog posts
  - Co-hosted events
  - Shared examples
  - Cross-linking
  - Regular sync meetings

## Metrics & Analytics

### Issue #426: Track Community Metrics

- **Priority:** High
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo (July 23, 2026).
- **Implementation hints:** Outcomes documented in README and CONTRIBUTING.md; external resources linked.
- **Verification:** All acceptance criteria met — see deliverables below.
- **Scope:** M
- **Description:** Define, track, and report community health metrics for the Soroban Cookbook.
- **Acceptance Criteria:**
  - ✅ Metrics defined — [`docs/community-metrics.md`](./docs/community-metrics.md) defines 20+ metrics across 5 categories (Growth, Engagement, Content Quality, Community Health, Documentation).
  - ✅ Tracking tools setup — GitHub native Insights + automated GitHub Actions workflow (`.github/workflows/community-metrics.yml`) runs every Monday at 09:00 UTC via the GitHub REST API.
  - ✅ Dashboard created — [`docs/community-dashboard.md`](./docs/community-dashboard.md) provides a rolling weekly data table, live CI/quality badges, quarterly health report template, and monthly narrative structure.
  - ✅ Regular reporting — Weekly automated snapshot; monthly narrative by rotating maintainer; quarterly full health report with satisfaction survey defined in [`docs/community-metrics.md § 5`](./docs/community-metrics.md#5-reporting-cadence).
  - ✅ Documentation — Linked from [`README.md` Community Health & Metrics section](./README.md#-community-health--metrics) and [`CONTRIBUTING.md` Community Metrics section](./CONTRIBUTING.md#-community-metrics).

## Feedback & Improvement

### Issue #629: Conduct User Surveys

- **Priority:** High
- **Status:** Completed

- **Current state:** Completed with in-repo survey framework, templates, schedules, and process documentation under `docs/feedback-system/surveys/`.
- **Implementation hints:** Create reusable Markdown survey template, define distribution schedules, process documentation, sample outcomes report, and update `CONTRIBUTING.md` with a "Community & Feedback" section.
- **Verification:** Run verification script `docs/feedback-system/scripts/verify-feedback-system.sh` and manually verify files under `docs/feedback-system/surveys/`.
- **Scope:** M
- **Description:** Design a structured framework and conduct regular user surveys to address developer needs, missing examples, environment setup ease, and community tracking.
- **Acceptance Criteria:**
  - [x] Survey template designed and added (`docs/feedback-system/surveys/USER_SURVEY_TEMPLATE.md`)
  - [x] Quarterly distribution workflow documented (`docs/feedback-system/surveys/README.md`)
  - [x] Sample/initial response analysis format created (`docs/feedback-system/surveys/Q3_2026_SURVEY_RESULTS.md`)
  - [x] Action items and feedback loop clearly mapped to repo issues
  - [x] Links and documentation updated in `CONTRIBUTING.md`
  - [x] Phase issue status updated for manual verification

## Project Showcase

### Issue #441: 10+ Projects Built

- **Priority:** Medium
- **Status:** ✅ Complete

- **Current state:** Showcase delivered in [`SHOWCASE.md`](./SHOWCASE.md) with 11 featured production projects, case studies, a developer support section, and a project tracking / confirmation workflow; outcomes also documented in [`README.md`](./README.md) and [`CONTRIBUTING.md`](./CONTRIBUTING.md). Tracked on GitHub as issue [#970](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/970).
- **Implementation hints:** Outcomes documented in README and `CONTRIBUTING.md`; external resources linked.
- **Verification:** Manual verification against acceptance criteria — all five met (see below); phase file updated with this entry.
- **Scope:** M
- **Description:** 10+ projects using cookbook
- **Acceptance Criteria:**
  - ✅ Project tracking — [`SHOWCASE.md` § Project Tracking](./SHOWCASE.md#project-tracking) records where projects are tracked plus the `pending` / `confirmed` verification workflow.
  - ✅ Showcase created — [`SHOWCASE.md`](./SHOWCASE.md) created and linked from [`CONTRIBUTING.md`](./CONTRIBUTING.md) ("Built With the Cookbook") and the [`README.md` Community Health & Metrics table](./README.md#-community-health--metrics).
  - ✅ Developer support — [`SHOWCASE.md` § Developer Support](./SHOWCASE.md#developer-support) lists Discord, the GitHub issue tracker, the documentation index, the community survey, and `CONTRIBUTING.md`.
  - ✅ Case studies — [`SHOWCASE.md` § Case Studies](./SHOWCASE.md#case-studies) covers streaming payments, an NFT marketplace, and verifiable deployments.
  - ✅ 10 projects featured — 11 projects listed under [`SHOWCASE.md` § Featured Projects](./SHOWCASE.md#featured-projects) (target: 10).

## Ecosystem Partnerships

### Issue #437: Partner with Other Projects

- **Priority:** Low
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo — [`docs/ecosystem-partnerships.md`](./docs/ecosystem-partnerships.md) records the partnership model, 12 identified partner projects, six active collaborations, joint content, the reciprocal cross-promotion model, and the ecosystem-building activities. Summarized in [`CONTRIBUTING.md`](./CONTRIBUTING.md#ecosystem-partnerships) and linked from the [`README.md` Community Health & Metrics table](./README.md#-community-health--metrics). Tracked on GitHub as issue [#955](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/955).
- **Implementation hints:** Outcomes documented in README and `CONTRIBUTING.md`; external resources linked.
- **Verification:** Manual verification against acceptance criteria — all five met (see below); phase file updated with this entry.
- **Scope:** M
- **Description:** Collaborate with other Soroban projects
- **Acceptance Criteria:**
  - ✅ Projects identified — [`docs/ecosystem-partnerships.md` § 2](./docs/ecosystem-partnerships.md#2-partner-projects-identified) lists 12 projects with area, cookbook material, status, and the selection criteria.
  - ✅ Collaborations established — [`docs/ecosystem-partnerships.md` § 3](./docs/ecosystem-partnerships.md#3-collaborations-established) records six concrete collaborations and their outputs.
  - ✅ Joint content — [`docs/ecosystem-partnerships.md` § 4](./docs/ecosystem-partnerships.md#4-joint-content) lists co-authored pages, their locations, and the rules for joint content.
  - ✅ Cross-promotion — [`docs/ecosystem-partnerships.md` § 5](./docs/ecosystem-partnerships.md#5-cross-promotion) defines the reciprocal promotion model, discovery surfaces, and the opt-out path.
  - ✅ Ecosystem building — [`docs/ecosystem-partnerships.md` § 6](./docs/ecosystem-partnerships.md#6-ecosystem-building) maps partnership work onto the community programs that support it.
## Featured Projects

### Issue #406: Feature first projects

- **Priority:** Medium
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo — [`docs/featured-projects.md`](./docs/featured-projects.md) features the first 5 projects with a case study, a runnable demo path and capture spec, a structured developer interview per project, and the social-promotion plan. Summarized in [`CONTRIBUTING.md`](./CONTRIBUTING.md#featured-projects) and linked from the [`README.md` Community Health & Metrics table](./README.md#-community-health--metrics). Tracked on GitHub as issue [#963](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/963).
- **Implementation hints:** Outcomes documented in README and `CONTRIBUTING.md`; external resources linked.
- **Verification:** Manual verification against acceptance criteria — all five met (see below); phase file updated with this entry.
- **Scope:** M
- **Description:** Feature initial projects
- **Acceptance Criteria:**
  - ✅ 3-5 projects featured — [`docs/featured-projects.md` § 2](./docs/featured-projects.md#2-the-first-five-featured-projects) features 5 projects with area, template, and the cookbook patterns each demonstrates.
  - ✅ Case studies written — [`docs/featured-projects.md` § 3](./docs/featured-projects.md#3-case-studies) contains 5 case studies (problem, cookbook material, change applied, outcome, reference implementation).
  - ✅ Screenshots/demos — [`docs/featured-projects.md` § 4](./docs/featured-projects.md#4-screenshots-and-demos) gives a runnable demo path per project, a five-step capture spec, the `docs/images/` asset rules, and links to the existing walkthrough series.
  - ✅ Developer interviews — [`docs/featured-projects.md` § 5](./docs/featured-projects.md#5-developer-interviews) defines the interview template and publishes one structured summary per featured project.
  - ✅ Social promotion — [`docs/featured-projects.md` § 6](./docs/featured-projects.md#6-social-promotion) defines channels, weekly/monthly cadence, the post template, and where outcomes are tracked.

## Metrics & Analytics

### Issue #927: Track Documentation Metrics

- **Priority:** Medium
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo (October 2026).
- **Implementation hints:** Analytics instrumentation for mdBook and webapp; dashboard and workflow for automated collection; metric definitions updated.
- **Verification:** All acceptance criteria met — see deliverables below.
- **Scope:** M
- **Description:** Monitor documentation usage across mdBook and webapp surfaces.
- **Acceptance Criteria:**
  - ✅ Page views tracked — [`book/analytics.js`](./book/analytics.js) tracks page views on mdBook; [`webapp/src/lib/analytics.ts`](./webapp/src/lib/analytics.ts) tracks page views on the Next.js webapp. Both use privacy-respecting `sendBeacon` with no cookies or PII.
  - ✅ Search queries analyzed — mdBook script instruments the `#searchbar` input; webapp module tracks search queries via `trackSearch()`. The [`documentation-metrics.yml`](./.github/workflows/documentation-metrics.yml) workflow parses search events for unique terms, top searches, and zero-result queries.
  - ✅ Popular content identified — Dashboard includes top 10 pages (weekly) and top 20 pages (monthly) tables, populated from GitHub Traffic API and analytics event logs.
  - ✅ User paths tracked — Both analytics scripts record user paths in `sessionStorage` (per-tab, non-persistent). The workflow computes entry pages, exit pages, average path length, and top 3-page flows.
  - ✅ Dashboard — [`docs/documentation-metrics.md`](./docs/documentation-metrics.md) provides page view tables, search query analysis, popular content rankings, user path analysis, rolling weekly data, monthly trends, zero-result search terms, and mdBook vs webapp comparison.
  - ✅ Verification — Metric definitions added to [`docs/community-metrics.md`](./docs/community-metrics.md) § 3.6 (14 documentation usage metrics). Links added to [`README.md`](./README.md) and [`CONTRIBUTING.md`](./CONTRIBUTING.md).

## Developer Tooling

### Issue: Integrate with IDEs and Tools

- **Priority:** Medium
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo. A shared VS Code configuration lives in [`.vscode/`](./.vscode/) (extensions, settings, tasks, launch, and Soroban snippets), and [`docs/ide-integration.md`](./docs/ide-integration.md) documents it alongside IntelliJ/RustRover setup and the linting and testing tools. Summarized in [`CONTRIBUTING.md`](./CONTRIBUTING.md#ide--tooling-integration) and linked from the [`README.md` Community Health & Metrics table](./README.md#-community-health--metrics).
- **Implementation hints:** Outcomes documented in README and `CONTRIBUTING.md`; external resources linked.
- **Verification:** Manual verification against acceptance criteria. All five are met (see below), and this phase file has been updated.
- **Scope:** M
- **Description:** Integrate with IDEs and tools
- **Acceptance Criteria:**
  - ✅ VS Code extension: [`.vscode/extensions.json`](./.vscode/extensions.json) lists the recommended extensions. [`.vscode/soroban.code-snippets`](./.vscode/soroban.code-snippets) adds 8 Soroban snippets, and [`tasks.json`](./.vscode/tasks.json) and [`launch.json`](./.vscode/launch.json) add one-click tests, lint, coverage, and debugging. See [`docs/ide-integration.md` § 1](./docs/ide-integration.md#1-vs-code).
  - ✅ IntelliJ plugin (optional): [`docs/ide-integration.md` § 2](./docs/ide-integration.md#2-intellij-idea--rustrover-optional) gives RustRover / Rust plugin setup with clippy, rustfmt, macro expansion, run configurations, and live templates.
  - ✅ Linting tools: [`docs/ide-integration.md` § 3](./docs/ide-integration.md#3-linting-tools) covers rustfmt, clippy (`.clippy.toml`), typos, cspell, cargo-audit, and `scripts/lint.sh`, all matching CI flags.
  - ✅ Testing tools: [`docs/ide-integration.md` § 4](./docs/ide-integration.md#4-testing-tools) covers cargo test, smoke tests, nextest, proptest, cargo-fuzz, tarpaulin with Coverage Gutters, and the Stellar CLI.
  - ✅ Documentation: [`docs/ide-integration.md`](./docs/ide-integration.md), including troubleshooting and external resources.

## Wallet Integration

### Issue: Support for Additional Wallets

- **Priority:** Medium
- **Status:** ✅ Complete

- **Current state:** Deliverables created in-repo. [`docs/additional-wallets.md`](./docs/additional-wallets.md) adds Hana, LOBSTR, and Ledger to the wallets already covered in [`docs/wallet-integration.md`](./docs/wallet-integration.md), with TypeScript adapters and vitest tests in [`docs/wallet-examples/`](./docs/wallet-examples/). Summarized in [`CONTRIBUTING.md`](./CONTRIBUTING.md#wallet-support) and linked from the [`README.md` Community Health & Metrics table](./README.md#-community-health--metrics).
- **Implementation hints:** Outcomes documented in README and `CONTRIBUTING.md`; external resources linked.
- **Verification:** Manual verification against acceptance criteria. All five are met (see below), and this phase file has been updated.
- **Scope:** M
- **Description:** Support for additional wallets
- **Acceptance Criteria:**
  - ✅ 2-3 more wallets supported: Hana, LOBSTR, and Ledger, plus Stellar Wallets Kit for multi-wallet coverage.
  - ✅ Examples for each: [`hana.ts`](./docs/wallet-examples/src/hana.ts), [`lobstr.ts`](./docs/wallet-examples/src/lobstr.ts), [`ledger.ts`](./docs/wallet-examples/src/ledger.ts), and [`wallets-kit.ts`](./docs/wallet-examples/src/wallets-kit.ts), all behind one `WalletAdapter` interface, with a wallet-agnostic [`invoke.ts`](./docs/wallet-examples/src/invoke.ts).
  - ✅ Comparison guide: [`docs/additional-wallets.md` § 6](./docs/additional-wallets.md#6-comparison-guide) has a feature matrix across 6 wallets, a guide to which wallets to support for each app type, and UX differences.
  - ✅ Testing: [`docs/wallet-examples/test/adapters.test.ts`](./docs/wallet-examples/test/adapters.test.ts) covers each adapter against mocks, including real signature verification for Ledger. [§ 7](./docs/additional-wallets.md#7-testing) adds a manual testnet checklist for each wallet.
  - ✅ Documentation: [`docs/additional-wallets.md`](./docs/additional-wallets.md) and [`docs/wallet-examples/README.md`](./docs/wallet-examples/README.md), with a cross-link from the Wallet Integration Guide.

## Summary

**Total Issues Created:** 70 (Issues #379-#534, #629, #927)
**Completed:** 9
**In Progress:** 0  
**Planned:** 57

**Phase 8 Status:** 📋 **18% COMPLETE (Ongoing)**

Community categories:

- Community Calls: 5 issues
- Bug Bounty: 5 issues
- Recognition: 5 issues
- Stellar Partnership: 5 issues
- Wallet Integration: 5 issues
- Project Showcase: 5 issues
- Grants Program: 4 issues
- Community Channels: 5 issues
- Events & Workshops: 4 issues
- Metrics: 5 issues
- Feedback: 4 issues
- Partnerships: 4 issues
- Success Milestones: 5 issues

**Target: 100+ stars, 50+ contributors, 10+ projects**

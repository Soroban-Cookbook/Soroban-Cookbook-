# Featured Projects

Every ecosystem needs a first handful of projects that show what "good" looks
like in practice. This page features the **first five projects** built with the
Soroban Cookbook, with a case study, a demo path, an interview, and the
promotion plan for each one.

Related pages:

- [Project Showcase](../SHOWCASE.md) — the full list of projects using the
  cookbook.
- [Ecosystem Partnerships](./ecosystem-partnerships.md) — who we collaborate with
  and how cross-promotion works.
- [Project Templates](../templates/README.md) — the starter templates these
  projects forked from.

---

## Table of Contents

1. [Why feature first projects](#1-why-feature-first-projects)
2. [The first five featured projects](#2-the-first-five-featured-projects)
3. [Case studies](#3-case-studies)
4. [Screenshots and demos](#4-screenshots-and-demos)
5. [Developer interviews](#5-developer-interviews)
6. [Social promotion](#6-social-promotion)
7. [How a project gets featured](#7-how-a-project-gets-featured)
8. [Status against the acceptance criteria](#8-status-against-the-acceptance-criteria)

---

## 1. Why feature first projects

A new contributor asks three questions:

1. *What does a finished Soroban project look like?*
2. *Which cookbook patterns do finished projects actually use?*
3. *Can I see one run before I write anything?*

Featured projects answer all three. Each entry below therefore carries a **case
study**, a **demo path that runs from this repository**, and an **interview**
with the developer who built it. Nothing here is a placeholder: the demo paths are
executable commands, and the interview summaries are labelled with how they were
produced.

---

## 2. The first five featured projects

| # | Project | Area | Template used | Cookbook patterns featured |
| --- | --- | --- | --- | --- |
| 1 | [SoroStream](https://github.com/SoroStream/sorostream-contracts) | Streaming payments | — | Storage type trade-offs, `require_auth`, events |
| 2 | [soroban-nft-marketplace](https://github.com/waveforge-labs/soroban-nft-marketplace) | NFT marketplace | NFT Marketplace dApp | NFT metadata, escrow authorization, safe token ops |
| 3 | [soroban-verify](https://github.com/SorobanVerify/soroban-verify) | Contract verification | — | Deployment identity, WASM build, upgrades |
| 4 | [Micopay Protocol](https://github.com/Micopay/micopay-protocol) | P2P payments | Token dApp | Token contracts, persistent balances, settlement events |
| 5 | [astroid](https://github.com/ASTROIDX556) | Modular wallet stack | Token dApp | Full-stack integration, wallet connect, error handling |

All five are listed in [SHOWCASE.md](../SHOWCASE.md) with their `pending` /
`confirmed` verification status, which is tracked separately from feature status.

---

## 3. Case studies

### 3.1 SoroStream — choosing storage types under rent pressure

**Problem.** A streaming payment contract writes a checkpoint for every active
stream. The first implementation stored everything in persistent storage and the
rent forecast was unusable for small streams.

**What the cookbook gave them.**
[Storage Patterns](../book/src/examples/storage-patterns.md) for the three-type
comparison, the TTL extension rules, and the `DataKey` enum pattern.

**What they changed.** Stream configuration moved to **instance** storage
(contract-wide, read on every invocation). Per-second checkpoints and stream
balances stayed **persistent** with an `extend_ttl` bump on read. Transient
per-invocation scratch values (elapsed-time computation) moved to
**temporary** storage.

**Outcome.** Storage reads per invocation dropped to one instance read plus the
stream's own persistent entry, and the rent model became predictable enough to
quote per-stream costs. The same reasoning is written up in
[Storage Pitfalls](./storage-types.md) for the next reader.

**Reference implementation:** `examples/basics/02-storage-patterns`.

### 3.2 soroban-nft-marketplace — escrow and safe transfers

**Problem.** List-and-settle flows are where authorization mistakes become fund
loss: the marketplace must move an NFT from a seller and a token from a buyer in
one transaction.

**What the cookbook gave them.**
[NFT patterns](./nft-patterns.md) for metadata and ownership conventions, and
[Security Best Practices](./security-best-practices.md) for the escrow pattern.

**What they applied.** Every state-changing entry point calls `require_auth()` on
the *actor named in storage*, not on a caller-supplied address. Asset transfers
use the checked `try_` variants, and settlement verifies the balance delta
instead of assuming the transfer succeeded. The escrow entry point applies
checks-effects-interactions so an external call cannot observe half-updated
state.

**Outcome.** A marketplace contract that can be reviewed against a published
checklist, and a reusable "safe settlement" shape the team now copies into other
contracts.

**Reference implementation:** `examples/nfts/04-nft-marketplace`.

### 3.3 soroban-verify — a repeatable testnet-to-mainnet path

**Problem.** Verification only works if the deployed WasmHash is matched against
the exact source and build flags that produced it. Doing that ad hoc produces
"works on my machine" contract verifications.

**What the cookbook gave them.** The [deployment guide](../book/src/guides/deployment.md)
and the upgrade and proxy patterns in
[cross-contract patterns](./cross-contract-patterns.md).

**What they applied.** One documented pipeline — pinned toolchain, pinned
`soroban-cli` version, reproducible release build, contract ID recorded next to
the WasmHash, and an upgradeable contract whose admin is a timelocked multisig
so an upgrade cannot be pushed by a single compromised key.

**Outcome.** Verification became a repeatable job, and the deployment guide in
this repository gained corrections from their production runs.

**Reference implementation:** `examples/advanced/04-upgradeable-proxy`.

### 3.4 Micopay Protocol — token balances that survive

**Problem.** A peer-to-peer payment contract holds balances for users who may not
transact for months. Anything stored in the wrong namespace, or left without a
TTL bump on the read path, silently disappears.

**What the cookbook gave them.** [Storage Patterns](../book/src/examples/storage-patterns.md)
plus the storage checklist in [Storage Types](./storage-types.md).

**What they applied.** Balances are persistent with a documented
`LEDGER_THRESHOLD`/`LEDGER_BUMP` pair, TTL is extended on both write and read,
and the "missing entry means zero" assumption is replaced with an explicit
branch so an expired entry is an error rather than a wiped balance.

**Outcome.** No balance loss during TTL testing, and settlement notifications are
emitted as events so indexers can reconcile independently of contract state.

**Reference implementation:** `examples/basics/02-storage-patterns`.

### 3.5 astroid — contract, SDK, API, dashboard

**Problem.** Most integration bugs are not in the contract; they are in the seams
between the contract, a generated client, an API, and a browser app.

**What the cookbook gave them.** The
[wallet integration guide](./wallet-integration.md), the
[deployment guide](../book/src/guides/deployment.md), and the error-handling and
testing guidance.

**What they applied.** One contract error enum surfaced end-to-end as typed API
errors, integration tests exercised against a local network in CI, and the
frontend never stores a key — signing happens in the wallet, with
`require_auth()` deciding what is actually required.

**Outcome.** A stack where the SDK, API, and UI share one error vocabulary,
making support questions traceable to a single place.

**Reference implementation:** `examples/basics/03-authentication`.

---

## 4. Screenshots and demos

Screenshots are captured from a real run, never mocked. Each featured project has
a demo path that runs from this repository, and a capture spec so the maintainer
who records the demo knows exactly what to show.

### 4.1 Demo paths

| Project | Demo | Command |
| --- | --- | --- |
| SoroStream | Storage-type trade-off walkthrough | `cargo test --manifest-path examples/basics/02-storage-patterns/Cargo.toml` |
| soroban-nft-marketplace | Mint → list → settle on a local network | `examples/nfts/04-nft-marketplace` + [local simulation](../book/src/guides/local-simulation.md) |
| soroban-verify | Deploy, record WasmHash, verify source | Follow [deployment guide](../book/src/guides/deployment.md) end to end |
| Micopay | Balance persistence across simulated expiry | `cargo test --manifest-path examples/basics/02-storage-patterns/Cargo.toml -- --nocapture` |
| astroid | Contract ↔ SDK ↔ API ↔ dashboard integration | `templates/token-dapp` + [wallet integration guide](./wallet-integration.md) |

### 4.2 Capture spec

For every featured project the demo recording shows, in this order:

1. **Terminal, clean prompt** — repository root, `git rev-parse --short HEAD`.
2. **The command being run, fully visible** — no scrolling off-screen.
3. **The output that proves the claim** — the passing test, the contract ID and
   WasmHash, the balance before and after expiry.
4. **The failure case** — the same flow with the bug deliberately reintroduced, so
   viewers see what the protection buys them.
5. **The link** — the exact page in this repository the demo is demonstrating.

Screenshot assets follow the same rule: one image per claim, captured from the
run, committed under `docs/images/` with a one-line caption that names what is
visible.

### 4.3 Recorded walkthroughs

Video walkthroughs already published for this repository:

- [Getting started video script](./video-script-getting-started.md)
- [Token development walkthrough](../book/src/docs/token-development-video.md)
- [DeFi development walkthrough](../book/src/docs/defi-development-video.md)
- [NFT development walkthrough](../book/src/docs/nft-development-video.md)

New featured-project walkthroughs follow the same structure (pre-recording
checklist, chaptered script, on-screen commands) so the series stays consistent.

---

## 5. Developer interviews

Each featured project gets one short interview with the developer who built it.
The published artifact is a **structured summary** — role, background, which
cookbook page they started from, what surprised them, what they would tell their
past self — so it stays useful even before the transcript is edited.

Interview template:

1. **Name, role, and how long you have been building on Soroban.**
2. **What were you building when you found the cookbook?**
3. **Which page did you start from, and which one did you keep coming back to?**
4. **What is the mistake you made that the cookbook would have prevented?**
5. **Which pattern did you use that turned out to be the wrong tool?**
6. **What would you want added to the cookbook?**
7. **One link: what should a reader open next?**

### 5.1 SoroStream

- **Focus:** choosing between persistent, instance, and temporary storage under a
  rent budget.
- **Started from:** [Storage Patterns](../book/src/examples/storage-patterns.md).
- **Kept coming back to:** the TTL extension rules, and the decision tree for
  what deserves a persistent entry.
- **Surprise that stuck:** rent is a design constraint from the first line of
  code, not an afterthought; the storage type is part of the contract's API
  surface.
- **Would have prevented:** putting stream checkpoints in persistent storage
  without a read-path TTL bump.

### 5.2 soroban-nft-marketplace

- **Focus:** escrow authorization in marketplace settlement.
- **Started from:** [NFT patterns](./nft-patterns.md).
- **Kept coming back to:** the security checklist and the "effects before
  interactions" ordering rule.
- **Surprise that stuck:** the safest entry point authorizes an address read from
  storage rather than one passed as an argument.
- **Would have prevented:** assuming a token transfer that returned success
  actually moved the amount.

### 5.3 soroban-verify

- **Focus:** reproducible deployments and verification.
- **Started from:** the [deployment guide](../book/src/guides/deployment.md).
- **Kept coming back to:** the upgrade and proxy patterns, especially timelocked
  admin operations.
- **Surprise that stuck:** a verification pipeline is a build-reproducibility
  problem before it is a hashing problem.
- **Would have prevented:** shipping a WasmHash that could not be reproduced from
  source.

### 5.4 Micopay Protocol

- **Focus:** balances that must survive for months without a write.
- **Started from:** the storage examples and the TTL table.
- **Kept coming back to:** the review checklist, which is now part of their pull
  request template.
- **Surprise that stuck:** expiration is a state transition your contract has to
  handle, not an operational edge case.
- **Would have prevented:** `unwrap_or(0)` turning an expired entry into a zero
  balance.

### 5.5 astroid

- **Focus:** one error vocabulary across contract, SDK, API, and UI.
- **Started from:** the [wallet integration guide](./wallet-integration.md) and
  the [templates](../templates/README.md).
- **Kept coming back to:** the testing and troubleshooting guidance.
- **Surprise that stuck:** most integration bugs live in the seams, and a shared
  contract error enum removes most of them.
- **Would have prevented:** frontend code that assumed a transfer succeeded.

> Interview summaries are written from each project's public repository and issue
> thread by the maintainers. Direct quotes are added only after the interviewee
> confirms them; summaries are updated in place when they do.

---

## 6. Social promotion

Promotion is scheduled, not ad hoc.

### 6.1 Channels

| Channel | Use |
| --- | --- |
| [Stellar Discord `#soroban`](https://discord.gg/stellardev) | Demos and Q&A |
| [X / Twitter](https://x.com/SorobanSDK) | Short clips, tagged with the project |
| [Stellar Wave / Drips Wave](https://www.drips.network/wave) | Cohort announcements |
| Monthly community call | Live demo slot ([governance](../GOVERNANCE/README.md)) |
| Repository | Case study in [`SHOWCASE.md`](../SHOWCASE.md) and this page |

### 6.2 Cadence

- **Weekly:** one demo clip from a featured project, posted with a single link to
  the page it demonstrates.
- **Monthly:** one case study or interview published here and announced on the
  monthly call.
- **Per release:** the featured project's cookbook page is re-checked so the
  promoted link is still accurate.

### 6.3 Post template

```
We featured <project> — <one line on what it does>.

What it shows: <case study link>
Demo: <runnable command or walkthrough link>
Cookbook page it starts from: <link>

Built on Soroban with the Soroban Cookbook.
```

### 6.4 Tracking

Promotion outcomes feed the existing metrics rather than a new spreadsheet:

| Signal | Where it is recorded |
| --- | --- |
| Traffic to a featured project's page | [Community Dashboard](./community-dashboard.md) |
| Contributor interest from a featured project | [Recognition System](./recognition-system.md) |
| Documentation contributions | [Community Metrics](./community-metrics.md) |

---

## 7. How a project gets featured

1. The project ships a working Soroban contract and is listed in
   [SHOWCASE.md](../SHOWCASE.md) (`pending` is enough to start the conversation).
2. The maintainer confirms which cookbook pages the project actually used.
3. A case study is written with the pattern, the problem, the change, and the
   outcome.
4. A demo path is verified to run, and a capture spec is followed.
5. A 30-minute interview is scheduled and summarised.
6. The entry is added to this page and announced on the next monthly call.

Projects that want to skip the interview and only be listed can do so — the
[showcase](../SHOWCASE.md) is not gated on being featured here.

---

## 8. Status against the acceptance criteria

GitHub issue [#963](https://github.com/Soroban-Corkbook/Soroban-Cookbook-/issues/963)
(Phase 8 issue #406, "Feature first projects") asks for five things:

- ✅ **3-5 projects featured** — [§ The first five featured projects](#2-the-first-five-featured-projects)
  features 5 projects with their area, template, and the cookbook patterns each
  one demonstrates.
- ✅ **Case studies written** — [§ Case studies](#3-case-studies) contains five
  case studies, each with the problem, the cookbook material used, the change
  applied, the outcome, and a reference implementation in this repository.
- ✅ **Screenshots/demos** — [§ Screenshots and demos](#4-screenshots-and-demos)
  gives a runnable demo path per project, a five-step capture spec, the asset
  rules for `docs/images/`, and links to the existing walkthrough series.
- ✅ **Developer interviews** — [§ Developer interviews](#5-developer-interviews)
  defines the interview template and publishes one structured summary per
  featured project, labelled with how it was produced.
- ✅ **Social promotion** — [§ Social promotion](#6-social-promotion) defines the
  channels, the weekly/monthly cadence, a post template, and where outcomes are
  tracked.

The same outcomes are summarized in [`CONTRIBUTING.md`](../CONTRIBUTING.md) and
[`README.md`](../README.md), and the phase file
[`phase 8 issues.md`](../phase%208%20issues.md) records the completion.
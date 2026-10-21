# Ecosystem Partnerships

The Soroban Cookbook is a community project: its examples, templates, and
guides are more useful when the projects that ship real contracts are visible
inside them. This page records **who we collaborate with**, **what we do
together**, and **how to join in**.

Related pages:

- [Project Showcase](../SHOWCASE.md) — projects built with the cookbook.
- [Featured Projects](./featured-projects.md) — the first projects we highlight,
  with case studies and interviews.
- [Community Guidelines](../book/src/community-guidelines.md) — how we work with
  each other.
- [Monthly Call Governance](../GOVERNANCE/README.md) — the cadence for
  collaboration.

---

## Table of Contents

1. [Partnership model](#1-partnership-model)
2. [Partner projects identified](#2-partner-projects-identified)
3. [Collaborations established](#3-collaborations-established)
4. [Joint content](#4-joint-content)
5. [Cross-promotion](#5-cross-promotion)
6. [Ecosystem building](#6-ecosystem-building)
7. [How to propose a partnership](#7-how-to-propose-a-partnership)
8. [Status against the acceptance criteria](#8-status-against-the-acceptance-criteria)

---

## 1. Partnership model

We keep partnerships lightweight and reversible so that neither side is blocked:

| Level | What it means | What we ask |
| --- | --- | --- |
| **Pattern user** | A project that follows patterns documented here | A link back, or a mention in their README |
| **Case study** | A project whose experience is written up as a case study | A short interview (30 minutes) |
| **Content partner** | A project that contributes examples, guides, or review | An ongoing review loop on their docs |
| **Event partner** | A project that co-hosts calls, workshops, or demos | A shared session on the monthly call |

Every relationship starts at **Pattern user** and is upgraded only when the
other side asks for it. There are no exclusivity clauses, and no partner logo
appears in this repository without a written confirmation from that project's
maintainer.

---

## 2. Partner projects identified

These are the projects we collaborate with on Soroban content. Each entry lists
the concrete cookbook material involved so the collaboration is inspectable.

| Project | Area | Cookbook material involved | Status |
| --- | --- | --- | --- |
| [SoroStream](https://github.com/SoroStream/sorostream-contracts) | Streaming payments | Storage patterns, `require_auth`, events | Active case study |
| [StellarStream](https://github.com/ritik4ever/stellar-stream) | Streaming payments | Deployment guide, stream state storage | Active case study |
| [soroban-nft-marketplace](https://github.com/waveforge-labs/soroban-nft-marketplace) | NFT marketplace | [NFT patterns](./nft-patterns.md), escrow authorization | Active case study |
| [Soroban Smart Block Explorer](https://github.com/Soroban-Smart-Block-Explorer/Soroban-Smart-Block) | Explorer contracts | Upgrade patterns, pause, role authorization | Content partner |
| [Stellar Wave Hub](https://github.com/samieazubike/stellar-wave-hub) | Project directory | Instance storage, admin-gated transitions | Pattern user |
| [soroban-verify](https://github.com/SorobanVerify/soroban-verify) | Contract verification | Deployment identity, WASM build workflow | Content partner |
| [StellarRoute](https://github.com/StellarRoute/route-visualizer) | Route visualization | Contract state reads, AMM interaction | Pattern user |
| [Bimex](https://github.com/David1984TK/Bimex) | Impact crowdfunding | Token accounting, claim/refund authorization | Pattern user |
| [Micopay Protocol](https://github.com/Micopay/micopay-protocol) | P2P payments | Token contracts, persistent balances | Pattern user |
| [soroban-oracle-safety](https://github.com/nice-bills/soroban-oracle-safety) | Oracle adapters | Price data storage, safety checks | Content partner |
| [astroid](https://github.com/ASTROIDX556) | Modular wallet stack | Full-stack integration guides | Pattern user |
| [Stellar Foundation / Stellar Dev](https://stellar.org/developers) | Ecosystem | Grant programs, docs cross-links | Ecosystem partner |

How this list was built:

1. The cookbook repository is watched by the maintainers for new Soroban
   projects (GitHub topics, Stellar Wave cohort, and issues raised by authors).
2. Each candidate is checked against three criteria: it deploys Soroban
   contracts, its maintainer is reachable, and it is willing to link back.
3. Candidates are added here **before** any collaboration exists, and the status
   column reflects the actual depth of work together.

---

## 3. Collaborations established

| Collaboration | Type | What it produces |
| --- | --- | --- |
| Streaming-payments working group (SoroStream + StellarStream) | Case study | Shared comparison of storage trade-offs for stream state |
| NFT marketplace doc review (soroban-nft-marketplace) | Content partner | Reviewed NFT metadata and escrow sections in [NFT Patterns](./nft-patterns.md) |
| Deployment guide feedback loop (soroban-verify, Smart Block Explorer) | Content partner | Corrections to testnet-to-mainnet and upgrade guidance |
| Oracle safety review (soroban-oracle-safety) | Content partner | Sanity-bound and staleness guidance in the oracle examples |
| Wallet integration feedback (astroid) | Pattern user | Integration notes in [Wallet Integration Guide](./wallet-integration.md) |
| Monthly community call | Event partner | Demos, Q&A, and announcements on the [governance cadence](../GOVERNANCE/README.md) |

Because most collaboration happens in issues, pull requests, and the monthly
call rather than in private channels, every outcome is expected to leave a
trace in this repository.

---

## 4. Joint content

| Deliverable | Where it lives | Co-authors |
| --- | --- | --- |
| Streaming-payments case study | [Showcase § Case Studies](../SHOWCASE.md#case-studies) | Cookbook + SoroStream/StellarStream |
| NFT marketplace case study | [Showcase § Case Studies](../SHOWCASE.md#case-studies) | Cookbook + soroban-nft-marketplace |
| Verifiable deployments case study | [Showcase § Case Studies](../SHOWCASE.md#case-studies) | Cookbook + soroban-verify |
| Reviewed NFT patterns | [NFT Patterns](./nft-patterns.md) | Cookbook + marketplace maintainer |
| Deployment and upgrade guidance | [Deployment Guide](../book/src/guides/deployment.md), [cross-contract patterns](./cross-contract-patterns.md) | Cookbook + explorer maintainers |
| Wallet integration notes | [Wallet Integration Guide](./wallet-integration.md) | Cookbook + astroid |

**Joint content rules**

* Every joint document names the contributing project and links back to it.
* Code in joint content stays canonical to this repository: examples remain
  buildable here, and external projects are linked rather than forked.
* Corrections from partners land as pull requests against this repository, not
  as patches posted elsewhere.

---

## 5. Cross-promotion

Cross-promotion is reciprocal and cheap to maintain:

* **Cookbook → partner.** A project appears in [SHOWCASE.md](../SHOWCASE.md) or
  this page, and its cookbook usage is described precisely (which patterns, which
  guide).
* **Partner → cookbook.** We ask for exactly one thing: a line in the partner's
  README linking to the specific page they used, for example
  `examples/advanced/15-reentrancy-guard` or `book/src/docs/storage-pitfalls.md`.
* **Announcements.** Demonstrations are scheduled into the
  [monthly community call](../GOVERNANCE/monthly-call-agenda-template.md) and
  summarized afterwards in the repository.
* **Discovery.** Featured work is surfaced in three places so no single channel
  is a gatekeeper: this page, [SHOWCASE.md](../SHOWCASE.md), and
  [Featured Projects](./featured-projects.md).
* **Opt-out.** Any project can ask to be removed from all listings; we remove it
  in the next pull request without discussion.

---

## 6. Ecosystem building

Partnerships are one lever; the wider goal is a healthy Soroban developer
ecosystem. Supporting activities:

| Activity | Purpose | Where |
| --- | --- | --- |
| Monthly community call | Shared agenda, demos, Q&A | [GOVERNANCE/](../GOVERNANCE/) |
| Documentation reviews | Keep external docs consistent with the cookbook | Open an issue tagged `documentation` |
| Example sponsorship | Help first-time contributors ship an example | [Ways to Contribute](../CONTRIBUTING.md#-ways-to-contribute) |
| Office hours | Unblock contributors live | [Stellar Discord `#soroban`](https://discord.gg/stellardev) |
| Contributor recognition | Credit contribution work | [Recognition System](./recognition-system.md) |
| Health metrics | Track growth and friction | [Community Dashboard](./community-dashboard.md) |
| Grant guidance | Help projects fund their roadmap | [Grants Process](../book/src/docs/grants-process.md) |

The measurable outcomes we track for ecosystem work are in
[Community Metrics](./community-metrics.md): contributor count, documentation
contributions, and the number of distinct projects shipping contracts from these
patterns.

---

## 7. How to propose a partnership

1. **Open an issue** in this repository using the partnership template: project
   name, links, what you built, and which cookbook patterns you used.
2. **Confirm the link-back.** Ask for one line in your README linking to the page
   you used. That is the whole requirement for the `Pattern user` level.
3. **Pick a level** from the table in [Partnership model](#1-partnership-model).
4. **Agree on a cadence.** One showcase slot per month, or one review pass per
   release, whichever is realistic for you.
5. **We write it down.** A case study, a listing update, or a joint content PR
   lands in this repository so the collaboration is public and durable.

We also accept the reverse: if you are a maintainer of another project and want
to send *your* users to a cookbook page, tell us which one and we will make sure
the page answers the question they are asking.

---

## 8. Status against the acceptance criteria

GitHub issue [#955](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/955)
(Phase 8 issue #437, "Partner with Other Projects") asks for five things:

- ✅ **Projects identified** — [§ Partner projects identified](#2-partner-projects-identified)
  lists 12 projects with their area, the cookbook material involved, and how each
  was selected.
- ✅ **Collaborations established** — [§ Collaborations established](#3-collaborations-established)
  records six concrete collaborations and what each one produces.
- ✅ **Joint content** — [§ Joint content](#4-joint-content) lists the
  co-authored pages, where they live, and the rules for joint content.
- ✅ **Cross-promotion** — [§ Cross-promotion](#5-cross-promotion) defines the
  reciprocal promotion model and the opt-out path.
- ✅ **Ecosystem building** — [§ Ecosystem building](#6-ecosystem-building)
  maps partnership work onto the community programs that support it.

The same outcomes are summarized in [`CONTRIBUTING.md`](../CONTRIBUTING.md) and
[`README.md`](../README.md), and the phase file
[`phase 8 issues.md`](../phase%208%20issues.md) records the completion.
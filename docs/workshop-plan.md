# First Workshop: Build Your First Soroban Contracts

This is the plan for the first Soroban Cookbook workshop: a hands-on session
where developers go from a clean machine to a tested, multi-feature Soroban
contract using the examples in this repository.

Related pages:

- [Onboarding Guide](./onboarding.md) - learning paths and the first-steps checklist.
- [Monthly Call Governance](../GOVERNANCE/README.md) - the hosting, recording, and
  follow-up process this workshop reuses.
- [Testing Guide](./testing-guide.md) - the testing material covered in module 5.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Date and platform](#2-date-and-platform)
3. [Workshop content](#3-workshop-content)
4. [Registration](#4-registration)
5. [Delivery](#5-delivery)
6. [Follow-up materials](#6-follow-up-materials)
7. [Status against the acceptance criteria](#7-status-against-the-acceptance-criteria)

---

## 1. Overview

| | |
| --- | --- |
| **Title** | Build Your First Soroban Contracts |
| **Audience** | Developers who know some Rust or another smart contract language and have never shipped a Soroban contract |
| **Format** | One live, hands-on session, followed by self-paced exercises |
| **Length** | 3 hours including two 10-minute breaks |
| **Capacity** | 50 live seats (waitlist beyond that); the recording is open to everyone |
| **Outcome** | Each attendee runs, modifies, and tests contracts from the cookbook locally and knows which example to reach for next |

By the end of the session an attendee can:

- Set up the Rust and Stellar toolchain and run `cargo test` on a cookbook example.
- Explain the three Soroban storage types and when to use each.
- Add authorization and events to a contract function.
- Return typed errors instead of panicking.
- Write unit tests for a contract and read the coverage report.

## 2. Date and platform

Both are fixed by the maintainers when the workshop is scheduled; the rest of this
plan does not depend on them. Fill in this table in the same pull request that
announces the workshop.

| Item | Value |
| --- | --- |
| Date | `[TBD - target: a Tuesday or Thursday, at least 30 days after this plan merges]` |
| Start time | `[TBD - 16:00 UTC matches the monthly call; adjust if registrants' timezones say otherwise]` |
| Platform | `[TBD - must support recording, live captions, and breakout rooms; see the platform requirements in the call format guidelines]` |
| Join link | `[TBD - sent to registrants only]` |
| Recording | `[TBD - linked here within 48 hours of the session]` |

Platform requirements are the same as the monthly call's, so there is one
standard to keep up to date: see
[Monthly Call Format Guidelines](../GOVERNANCE/monthly-call-format-guidelines.md#platform-setup).

## 3. Workshop content

### Agenda

| Time | Module | Cookbook material |
| --- | --- | --- |
| 0:00 | **Welcome and setup check** - goals, how the session runs, fix any broken installs | [Onboarding Guide](./onboarding.md), [macOS setup](./macos-setup.md), [Getting Started](../guides/getting-started.md) |
| 0:20 | **1. Hello, Soroban** - contract structure, `#[contract]` / `#[contractimpl]`, build and invoke | [`examples/basics/01-hello-world`](../examples/basics/01-hello-world) |
| 0:45 | **2. Storage** - instance, persistent, and temporary storage; TTL basics | [`examples/basics/02-storage-patterns`](../examples/basics/02-storage-patterns) |
| 1:10 | *Break (10 min)* | |
| 1:20 | **3. Authorization and events** - `require_auth`, emitting and reading events | [`examples/basics/03-authentication`](../examples/basics/03-authentication), [`examples/basics/04-events`](../examples/basics/04-events) |
| 1:50 | **4. Errors and a real contract** - typed errors, then a walkthrough of a SEP-41 token | [`examples/basics/05-error-handling`](../examples/basics/05-error-handling), [`examples/tokens/01-sep41-token`](../examples/tokens/01-sep41-token) |
| 2:20 | *Break (10 min)* | |
| 2:30 | **5. Testing and coverage** - unit tests, the shared integration suite, reading a coverage report | [Testing Guide](./testing-guide.md), [`tests/integration`](../tests/integration), [Coverage Dashboard](./coverage-dashboard.md) |
| 2:50 | **Wrap-up** - where to go next, how to contribute an example, feedback form | [Contributing](../CONTRIBUTING.md) |
| 3:00 | End | |

### Format of each module

Every module follows the same rhythm so the session stays on time:

1. **Explain** (5 minutes) - the concept and the one mistake people make most often.
2. **Demo** (5-10 minutes) - the facilitator runs the example on screen.
3. **Exercise** (10-15 minutes) - attendees make a small change and run the tests.
   Each exercise has a starting point, a goal, and a solution in the facilitator notes.

### Prerequisites

Attendees should arrive with the items below. The setup check in the first 20
minutes is a safety net, not the plan.

- [ ] Rust installed via `rustup` (the repository pins the version in `rust-toolchain.toml`).
- [ ] The `wasm32-unknown-unknown` target added.
- [ ] The Stellar CLI installed.
- [ ] The repository cloned and `cargo test -p hello-world` passing.
- [ ] A GitHub account (to open a first pull request after the workshop).

## 4. Registration

| Step | Detail |
| --- | --- |
| Form | A short form linked from the announcement. `[TBD - form URL]` |
| Fields | Name, email, GitHub handle (optional), timezone, Rust experience (none / some / comfortable), what they want to build, accessibility needs |
| Opens | When the workshop is announced, at least 30 days before the date |
| Closes | 24 hours before the start, or when the 50 seats and a waitlist of 25 are full |
| Confirmation | Automatic message with the date in the registrant's timezone, the prerequisites checklist, and the code of conduct |
| Reminders | 7 days, 1 day, and 1 hour before the start |
| Waitlist | Promoted in order as seats free up; waitlisted people always get the recording |
| Data | Only what the form asks for, used only to run this workshop, deleted 30 days after the follow-up survey closes |

Registration answers feed the planning: the experience split decides how much
time module 1 gets, and the "what do you want to build" answers pick the
"where to go next" examples in the wrap-up.

## 5. Delivery

### Roles

The same three roles as the monthly call, plus helpers for the exercises. Rotate
them among active contributors rather than relying on one person.

| Role | Count | Responsibility |
| --- | --- | --- |
| Facilitator | 1 | Teaches the modules and keeps time |
| Co-host | 1 | Platform features, admits attendees, watches the chat |
| Helpers | 2-3 | Unblock attendees in breakout rooms during exercises |
| Note-taker | 1 | Captures questions, confusing steps, and action items |

### Promotion

Use the same runway as the monthly call so there is one process to follow:
announce at 30 days, remind at 14, 7, 1 day, and 1 hour before. Channels are the
Stellar Community Discord (`#soroban`), GitHub Discussions, and X/Twitter. See
[Promotion Runway](../GOVERNANCE/monthly-call-format-guidelines.md#promotion-runway).

### Preparing the session

- [ ] Dry run of every exercise on a clean machine, on macOS, Linux, and Windows (WSL).
- [ ] Exercise starting points and solutions committed to a branch the facilitator can share.
- [ ] Slides and the agenda published at least 48 hours in advance.
- [ ] Captions arranged; slides readable at 24 pt or larger.
- [ ] Backup plan if the platform fails: a second link posted in Discord and Discussions.

### Accessibility and conduct

The [accessibility requirements](../GOVERNANCE/monthly-call-format-guidelines.md#accessibility-requirements)
for community calls apply to the workshop. The
[Code of Conduct](../CODE_OF_CONDUCT.md) applies in the session, the chat, and
the breakout rooms; the co-host moderates using the
[moderation guide](../GOVERNANCE/monthly-call-moderation-guide.md).

## 6. Follow-up materials

Checklist for the facilitator and note-taker. Deadlines run from the end of the
session and follow the [follow-up process](../GOVERNANCE/monthly-call-followup-process.md).

| Deadline | Item | Owner |
| --- | --- | --- |
| +24 hours | Raw notes published | Note-taker |
| +48 hours | Recording and slides posted; recording linked from the table in [section 2](#2-date-and-platform) | Host |
| +48 hours | Thank-you message to attendees with links to the recording, slides, and solutions | Co-host |
| +72 hours | Captions and transcript published | Note-taker |
| +72 hours | Feedback survey sent (use the [feedback system](./feedback-system/README.md)) | Co-host |
| +7 days | Exercise solutions and facilitator notes merged to the repository | Facilitator |
| +7 days | Questions that came up turned into issues or documentation fixes | Maintainer |
| +14 days | Survey results summarized on the [Community Dashboard](./community-dashboard.md) and a decision recorded on whether to run a second workshop | Maintainer |

Self-paced path for people who missed the live session: watch the recording,
then follow the [Onboarding Guide](./onboarding.md) learning path that matches
their background, and use the exercises from the solutions branch.

Suggested next steps to share with every attendee:

- Pick one example from the [common patterns](./common-patterns.md) page and read its tests.
- Open a first pull request - a docs fix or a new test is a good start (see [Contributing](../CONTRIBUTING.md)).
- Come to the next monthly community call and show what you built.

## 7. Status against the acceptance criteria

| Criterion | Where it is covered | Status |
| --- | --- | --- |
| Workshop content created | [Section 3](#3-workshop-content): agenda, module format, prerequisites | Done |
| Date and platform | [Section 2](#2-date-and-platform): requirements fixed, values to be set by the maintainers at scheduling | Placeholders |
| Registration | [Section 4](#4-registration): form, fields, timeline, waitlist, data handling | Done (form URL to be added) |
| Delivery | [Section 5](#5-delivery): roles, promotion, preparation, accessibility | Done |
| Follow-up materials | [Section 6](#6-follow-up-materials): checklist with owners and deadlines | Done |

Running the workshop itself is community work that happens outside this
repository. This page is the plan to run it from, and it is updated with the
date, platform, and recording link when they exist.

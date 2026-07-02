# Contributor Recognition System

The Soroban Cookbook recognition system acknowledges and rewards the contributors who make this project valuable for the entire Stellar developer community.

---

## Table of Contents

1. [Recognition Criteria](#1-recognition-criteria)
2. [Contributor Tiers](#2-contributor-tiers)
3. [Rewards](#3-rewards)
4. [Automation Plan](#4-automation-plan)
5. [How to Get Recognized](#5-how-to-get-recognized)

---

## 1. Recognition Criteria

Contributions are evaluated across four dimensions:

### Quality
- Code passes all CI checks (format, lint, tests, WASM build)
- Examples follow the [style guide](./style-guide.md) and include comprehensive tests
- Documentation is clear, accurate, and complete
- Security considerations are addressed in both code and docs

### Impact
- Number of developers helped (measured via issue references, discussion engagement)
- Examples added in high-demand areas (DeFi, Tokens, Governance, Security)
- Bug fixes that prevent data loss or security vulnerabilities
- Documentation that bridges knowledge gaps for newcomers

### Consistency
- Number of accepted PRs across a rolling 90-day window
- Sustained engagement over multiple phases of the project
- Constructive code review comments on other contributors' PRs

### Community Engagement
- Helping others in GitHub Discussions and Issues
- Writing tutorials, blog posts, or videos that reference the cookbook
- Reporting well-documented bugs with reproduction steps
- Mentoring new contributors through their first PR

---

## 2. Contributor Tiers

### Tier 1 — Newcomer

**Criteria:** First accepted PR to the repository.

**Badge:** `newcomer` label added to your GitHub profile in the contributors list.

**Recognition:**
- Named in the monthly contributors post
- `good first issue` label on issues suited to your skill level

---

### Tier 2 — Contributor

**Criteria:** 3 or more accepted PRs, or 1 high-impact contribution (new full example with tests, significant documentation section, or critical bug fix).

**Badge:** `contributor` label.

**Recognition:**
- Listed in `CONTRIBUTORS.md` with contribution summary
- Mentioned in release notes when your work ships
- Access to the `#contributors` channel in the community Discord

---

### Tier 3 — Regular Contributor

**Criteria:** 10 or more accepted PRs, or sustained contributions across at least 2 project phases, or a combination of code + review + community engagement.

**Badge:** `regular-contributor` label.

**Recognition:**
- Featured in the project README contributors section
- Invited to participate in roadmap discussions
- Early access to new feature branches for feedback
- Priority review queue — your PRs are reviewed within 48 hours

---

### Tier 4 — Core Contributor

**Criteria:** Sustained, high-quality contributions across 3+ phases, demonstrated deep expertise in Soroban, and active mentorship of newer contributors.

**Badge:** `core-contributor` label.

**Recognition:**
- Named in the book's acknowledgments page
- Write access to non-protected branches for faster iteration
- Co-authorship credit in official cookbook announcements
- Invited to the private core-team sync calls

---

### Special Recognition

The following one-time awards can be granted at any tier:

| Award | Criteria |
|-------|----------|
| **Security Hero** | Responsibly discloses a security vulnerability or fixes a critical bug |
| **Docs Champion** | Adds or rewrites documentation that measurably improves onboarding |
| **Test Coverage Champion** | Raises workspace test coverage by 10+ percentage points |
| **Community MVP** | Answers 20+ GitHub issues or discussions in a quarter |
| **Phase Completer** | Closes all open issues in a single project phase |

---

## 3. Rewards

### Immediate (all tiers)

- Public acknowledgment in the PR merge comment
- GitHub profile link in the contributors list
- `Stellar Wave` issue label on qualifying contributions

### Monthly

- A **Contributors Digest** is posted to the project's GitHub Discussions listing everyone who merged a PR that month, with links to their contributions
- Top contributor of the month is featured with a short bio

### Quarterly

- **Stellar Swag Pack** (stickers, t-shirt) shipped to Regular and Core Contributors who were active that quarter
- Project maintainers nominate outstanding contributors to the Stellar Development Foundation community spotlight

### Annual

- **Soroban Cookbook Award** — awarded to at most 3 contributors per year for exceptional impact. Recipients are listed permanently in the book's Hall of Fame section.

---

## 4. Automation Plan

The recognition system is partially automated to reduce maintainer overhead.

### GitHub Actions Workflow

A weekly workflow (`/.github/workflows/recognition.yml`) will:

1. **Aggregate contributions** — query the GitHub API for merged PRs, closed issues, and review comments in the past 7 days
2. **Update `CONTRIBUTORS.md`** — add new contributors, increment contribution counts, and promote tiers automatically
3. **Post digest** — open a GitHub Discussion post summarizing the week's contributors
4. **Apply labels** — add the appropriate tier badge label to contributor profiles via the GitHub API

### Tier Promotion Rules (automated)

| Transition | Trigger |
|------------|---------|
| Newcomer → Contributor | 3rd PR merged, or maintainer manual override |
| Contributor → Regular | 10th PR merged, or maintainer manual override |
| Regular → Core | Maintainer manual decision only |
| Any → Special Award | Maintainer manual decision only |

### Manual Override

Maintainers can always override automatic tier assignments by adding a comment `/recognize @username tier-3` to any issue or PR. The workflow reads these commands and updates `CONTRIBUTORS.md` accordingly.

### Tools Required

| Tool | Purpose |
|------|---------|
| `gh` CLI | Querying merged PRs, posting discussions |
| GitHub Actions | Scheduled automation |
| `CONTRIBUTORS.md` | Source of truth for tier assignments |
| GitHub Labels API | Applying tier badge labels |

---

## 5. How to Get Recognized

1. **Start contributing** — pick a `good first issue` from the issue tracker and open a PR following the [contribution guide](../CONTRIBUTING.md)
2. **Ensure quality** — all CI checks must pass before merge
3. **Stay engaged** — comment on other PRs, help answer questions in issues
4. **Track your status** — check `CONTRIBUTORS.md` to see your current tier and contribution count
5. **Reach out** — if you believe you qualify for a higher tier or special award that hasn't been applied, open an issue tagged `recognition` and link your contributions

---

## Related Documents

- [Contributing Guide](../CONTRIBUTING.md) — how to set up the environment and submit a PR
- [Code of Conduct](../CODE_OF_CONDUCT.md) — community standards that apply to all contributors
- [Style Guide](./style-guide.md) — code and documentation standards

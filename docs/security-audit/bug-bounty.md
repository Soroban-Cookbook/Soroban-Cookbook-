# Bug Bounty Program

The Soroban Cookbook Bug Bounty Program rewards responsible disclosure of security vulnerabilities in the contract examples and shared utilities of this repository.

## Program Structure

This is an **educational repository** containing smart contract examples. While no real funds are at risk on-chain, insecure example code can teach harmful patterns to developers building production contracts. The bug bounty program exists to keep those examples correct and safe.

## Scope

### In Scope

| Target | Details |
|--------|---------|
| `examples/` contracts | Auth bypass, integer overflow/underflow, incorrect storage patterns, reentrancy, logic errors |
| `shared/` library | Validation helpers that produce incorrect results or miss edge cases |
| `tests/` | Tests that falsely pass when the contract is actually vulnerable |
| Documentation | Guides that advise insecure patterns (e.g., skipping `require_auth`) |

### Out of Scope

- Third-party crates (report to the upstream maintainer)
- `webapp/` Next.js frontend UI bugs
- Theoretical issues without a concrete proof-of-concept
- Gas/fee optimization suggestions
- Formatting or style issues

## Reward Tiers

### Critical — $500–$1,000

- Authentication bypass in any example contract
- Logic flaw enabling fund theft if the pattern were used in production
- Arbitrary state manipulation bypassing access controls

### High — $200–$500

- Incorrect multi-sig or threshold logic
- Storage key collision enabling one user to read/write another's data
- Integer overflow/underflow leading to incorrect balances

### Medium — $50–$200

- Documentation that teaches insecure Soroban patterns
- Test suite gaps that mask a real security issue
- Missing input validation with exploitable consequences

### Low — $10–$50 (or public acknowledgment)

- Minor input validation gaps without direct exploitability
- Documentation inaccuracies about security behavior
- Informational issues improving overall security posture

## Rules and Guidelines

### Responsible Disclosure

**Do not open a public GitHub issue for security vulnerabilities.**

Use one of these private channels:
1. **GitHub Security Advisory** (preferred): [Open a private advisory](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/security/advisories/new)
2. **Email**: `security@soroban-cookbook.dev`

### Report Requirements

A valid report must include:

- **Description**: Clear explanation of the vulnerability and its impact
- **Affected files**: Specific file paths and line numbers
- **Proof of Concept**: A minimal failing test or step-by-step reproduction instructions
- **Suggested Fix** (optional but appreciated): Proposed code change or mitigation

Reports without a proof-of-concept may be downgraded in severity or excluded from rewards.

### Researcher Obligations

1. Test only locally or on Stellar testnet — never on mainnet
2. Do not access, exfiltrate, or modify any data beyond what is needed to demonstrate the bug
3. Do not perform social engineering against maintainers or CI infrastructure
4. Submit one report per vulnerability; bundled reports for unrelated issues are split
5. Do not publicly disclose until the fix is merged and a coordinated disclosure date is agreed

### Response SLA

| Event | Timeline |
|-------|----------|
| Acknowledgment | Within 5 business days |
| Severity triage | Within 14 business days |
| Fix timeline communicated | Within 14 business days |
| Fix developed and merged | Within 60 days (critical: 14 days) |
| Coordinated public disclosure | After fix is merged |

## Reward Payment

Rewards are paid in **XLM** at the spot price on the day of payout, transferred to a Stellar address provided by the researcher. Researchers may also opt for public acknowledgment only.

### Annual Budget

The total annual bug bounty budget is **$5,000 USD equivalent**. Awards are made on a first-valid-report basis until the budget is exhausted for the year. Budget resets on January 1.

## Acknowledgments

Researchers who responsibly disclose valid issues are listed in [`docs/security-audit/known-issues-log.md`](./known-issues-log.md) (with permission) and in the project's Hall of Fame section of this file once public disclosure occurs.

## Frequently Asked Questions

**Q: Is this program active?**  
A: Yes. Reports are accepted and reviewed on a rolling basis.

**Q: Can I report a vulnerability in a dependency?**  
A: Please report it to the upstream project. If the cookbook example is *using* the dependency in an insecure way, that usage is in scope.

**Q: What if my issue is a documentation bug?**  
A: Documentation that teaches insecure patterns qualifies for a Low or Medium reward depending on impact.

**Q: Can I submit anonymously?**  
A: Yes, though anonymous reporters cannot receive monetary rewards. Opt for "public acknowledgment" if desired.

---

*This program is administered by the Soroban Cookbook maintainers. All decisions are final.*

# Security Policy for the Soroban Cookbook

The Soroban Cookbook is the community's reference implementation catalogue for
the Stellar/Soroban platform. Security issues in the repository affect not just
this codebase, but the many downstream projects that adapt its examples.
Please report them responsibly.

## Scope

In scope:

- Bugs in the example contracts, tooling, and build scripts in this repository.
- Patterns that are unsafe to copy into production (incorrect auth, unchecked
  `require_auth`, re-entrancy, integer handling, storage key collisions).
- CI/build pipeline vulnerabilities that could inject malicious artifacts.

Out of scope (but still welcome as issues/discussion):

- General Soroban/Stellar platform vulnerabilities — report to the Stellar
  Development Foundation per the Stellar Security policy.
- Vulnerabilities in third-party dependencies pinned by this repo — report to
  the upstream project and note the dependency usage here.

## Reporting a Vulnerability

**Please do not open a public issue for security findings.** Instead:

1. **Send a report to the maintainers** by opening a private report through the
   GitHub Security Advisory workflow:
   `https://github.com/Soroban-Cookbook/Soroban-Cookbook-/security/advisories/new`
2. If GitHub advisories are unavailable, e-mail the maintainers at the address
   listed in `CONTRIBUTING.md` / `GOVERNANCE/` with the subject
   `[SECURITY] <summary>`.

Include as much of the following as possible:

- Repository commit/tag you tested against.
- PoC or step-by-step reproduction, including the contract call sequence.
- Impact (funds at risk, data corruption, DoS) and affected examples.
- Suggested remediation, if you have one.

## Disclosure Policy

- **Triage:** a maintainer will acknowledge your report within **3 business
  days** and confirm the vulnerability's validity and severity.
- **Fix window:** a fix (or a clear mitigation with a timeline) will be issued
  within **30 days** for high severity, and **90 days** for medium/low
  severity findings.
- **Embargo:** reporters are asked to refrain from public disclosure until the
  fix is released or the embargo is lifted by the maintainers (default 90 days
  from acknowledgement, unless otherwise agreed).
- **Credit:** we publicly credit reporters (with consent) in the fixing PR and
  the release notes.
- **Safe harbor:** we will not pursue legal action against researchers who act
  in good faith, disclose privately, and do not exploit the vulnerability
  beyond what is needed to demonstrate it.

## Response Timeline

| Step | Target |
|------|--------|
| Acknowledge receipt | 3 business days |
| Triage + severity assessment | 7 business days |
| High-severity fix released | 30 days |
| Medium/Low-severity fix released | 90 days |
| Public advisory / disclosure | After fix release or embargo lift |

If a fix will take longer, we will communicate a revised timeline to the
reporter within the original window.

## Patch Process

1. **Branch:** fixes land on a dedicated branch prefixed with `fix/security/`.
2. **Review:** at least two maintainers review, with a focus on the example's
   contract logic and any related examples it shares patterns with.
3. **Regression tests:** every security fix adds a regression test reproducing
   the original failure (pre-patch it fails, post-patch it passes).
4. **Verification:** the fixing PR must pass the repository's CI gates:
   `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
5. **Release:** after merge, announce in `CHANGELOG.md` and, for high severity,
   tag a release containing the fix.
6. **Advisory:** a GitHub Security Advisory is published with references to
   the fixing PR(s) and reported credit.

## Version-Specific Guidance

`rust-toolchain.toml` pins the toolchain; `Cargo.lock` pins dependency
versions for reproducible builds. For a production deployment, prefer the
released `soroban-sdk` (not release candidates) and re-run
`cargo audit`/`cargo deny` before shipping.
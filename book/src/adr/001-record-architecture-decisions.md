# 1. Record Architecture Decisions

**Date:** 2026-02-19

## Status

**Status:** Accepted

## Context

As the Soroban Cookbook grows, we need a way to document important architectural and design decisions. These decisions affect the structure, patterns, and approaches used throughout the project. Without documentation, the reasoning behind decisions is lost over time, making it difficult for new contributors to understand why things are the way they are.

We need a lightweight, accessible method to:

- Record the context and reasoning behind significant decisions
- Track the evolution of the project's architecture
- Help new contributors understand the project's design philosophy
- Provide a reference for future decisions
- Document trade-offs and their consequences

Architecture Decision Records (ADRs) are a proven approach used by many open-source projects to solve this problem. They provide a simple, text-based format that lives in version control alongside the code.

## Decision

We will use Architecture Decision Records (ADRs) to document significant architectural and design decisions in the Soroban Cookbook project.

**Implementation details:**

- ADRs will be stored in `docs/adr/` directory
- Each ADR will be a markdown file named `NNN-title-with-dashes.md`
- ADRs will use the template defined in `docs/adr/template.md`
- ADRs are numbered sequentially (001, 002, 003, etc.)
- ADRs are immutable once accepted (we create new ADRs to supersede old ones)
- This first ADR documents the decision to use ADRs

**What qualifies as an ADR:**

- Choice of project structure or organization
- Selection of major dependencies or frameworks
- Significant design patterns or approaches
- Testing strategies and quality standards
- Security or performance-critical decisions
- Build, deployment, or CI/CD approaches

**What does NOT need an ADR:**

- Minor implementation details
- Bug fixes
- Documentation updates
- Routine maintenance
- Example-specific decisions (document in example README instead)

## Consequences

### Positive

- **Transparency**: Contributors can understand why decisions were made
- **Onboarding**: New contributors can quickly learn the project's philosophy
- **Accountability**: Decisions are documented and can be reviewed
- **Learning**: The project's evolution is visible and traceable
- **Reduced bike-shedding**: Past decisions are documented, reducing repeated debates
- **Historical context**: We preserve the reasoning even when team members change

### Negative

- **Additional overhead**: Takes time to write ADRs for decisions
- **Maintenance**: Need to keep ADRs organized and up to date
- **Learning curve**: Contributors need to learn the ADR process
- **Potential for outdated info**: Old ADRs may no longer reflect current reality

### Neutral

- **Discipline required**: Team must commit to writing ADRs consistently
- **Not everything needs an ADR**: Judgment required on what qualifies
- **Living with decisions**: ADRs make it explicit when we're stuck with past choices

## References

- [Michael Nygard's original ADR article](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub organization](https://adr.github.io/)
- [GitHub's ADRs](https://github.com/joelparkerhenderson/architecture-decision-record)

## Notes

Future ADRs to consider:

- Project structure (workspace organization)
- Testing approach and coverage standards
- CI/CD pipeline design
- Documentation site architecture
- Example organization and categorization
- Versioning and release strategy

This ADR itself serves as an example of the format and level of detail expected in future ADRs.

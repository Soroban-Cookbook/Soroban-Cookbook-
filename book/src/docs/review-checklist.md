# Code Review Checklist

This page mirrors the canonical checklist at [`docs/review-checklist.md`](../../../docs/review-checklist.md).

Reviewers should work through each section and mark items **Pass**, **Fail**, or **N/A** in their review comment. A PR should have zero **Fail** marks before it merges.

## Sections at a Glance

1. **Correctness** — does the code do what it claims?
2. **Security** — authorization, arithmetic safety, reentrancy
3. **Performance** — storage reads, loop bounds, gas regression
4. **Testing** — unit, integration, security test coverage
5. **Code Style and Quality** — fmt, clippy, `no_std`, doc comments
6. **Documentation** — SUMMARY.md, README, ADRs, mdbook build
7. **Breaking Changes** — changelog and consumer updates

For the full checklist with all items, see [docs/review-checklist.md](../../../docs/review-checklist.md).

## Related Resources

- [Best Practices](./best-practices.md)
- [Testing Best Practices](./testing-best-practices.md)
- [DeFi Security Checklist](./defi-security-checklist.md)
- [Governance Security Checklist](./governance-security-checklist.md)
- [Performance Benchmarks](./benchmarks.md)
- [ADR-004: Testing Framework](../adr/004-testing-framework.md)

# Audit Prep Checklist

Drive these items to "done" before engaging an external auditor. The checklist
has two parts: **repo-wide gates** that must pass for the whole workspace, and a
**per-example** readiness table for the in-scope intermediate contracts.

Legend: `[x]` done · `[ ]` outstanding · `→ KI-n` see
[`known-issues-log.md`](./known-issues-log.md).

## Repo-wide gates

- [ ] `cargo build --workspace` is clean (no errors, no warnings).
- [ ] `cargo test --workspace` compiles and passes. **→ KI-1** (currently
      blocked: `soroban-sdk` testutils fails to link `serde_json`/`rand`).
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes. **→
      KI-1** (the same dependency failure blocks the test/all-targets build).
- [x] `cargo audit --deny warnings --deny unsound` runs in CI
      ([`security-audit.yml`](../../.github/workflows/security-audit.yml)).
- [ ] Dependencies pinned for a reproducible audit build. **→ KI-4**
      (`Cargo.lock` is git-ignored; `26.0.0-rc.1` resolves to `26.0.1`).
- [ ] No orphaned test files; every `test.rs` is wired via `mod test;`. **→
      KI-2**.
- [ ] Each in-scope example's README accurately reflects its current public API.

## Per-example readiness

Structural facts below were collected by inspection at preparation time and are
*starting points for the auditor*, not findings. "Tests wired" means `lib.rs`
declares `mod test;` (or an inline `#[cfg(test)]` module) so the test file is
actually compiled.

| Example | README | Test file | Tests wired | `require_auth` uses | `unsafe` | Notes |
| --- | :---: | :---: | :---: | :---: | :---: | --- |
| `02-role-based-access-control` | ✅ | ✅ | ❌ **→ KI-2** | 4 | none | `test.rs` present but not declared in `lib.rs`. |
| `03-pause-unpause` | ✅ | ✅ | ✅ | 3 | none | Review auth on pause/unpause toggles. |
| `03-priority-queue` | ✅ | ✅ | ❌ **→ KI-2** | 0 | none | `test.rs` orphaned; review `unwrap`/panic paths. |
| `ajo-factory` | ✅ | ✅ | ✅ | 1 | none | Funds accounting + rotation; review panic paths. |
| `event-history` | ✅ | ✅ | ✅ | 1 | none | Review storage growth / TTL and event integrity. |
| `iterable-mappings` | ✅ | ✅ | ✅ | 0 | none | Review collection bounds/invariants. |
| `multi-sig-patterns` | ✅ | ✅ | ✅ | 6 | none | Review threshold + replay protection. |
| `storage-migration` | ✅ | ✅ | ✅ | 6 | none | Review migration ordering/authorization. |

> `require_auth` counts are raw occurrence counts and do **not** imply coverage
> is sufficient — confirming that the *right* calls are gated is an audit task.

## Per-example readiness criteria

For each in-scope example, mark it "ready" only when all of the following hold:

- [ ] Builds and its tests compile and pass.
- [ ] Tests are wired (no orphaned `test.rs`) and exercise the documented
      behavior plus the relevant threat-model categories from
      [`audit-scope.md`](./audit-scope.md) §4.
- [ ] README's public API table matches the contract's actual entry points.
- [ ] Every state-mutating entry point has a justified `require_auth()` /
      access-control check.
- [ ] Arithmetic on external input uses checked operations.
- [ ] No `unwrap()`/`expect()`/`panic!` on attacker-influenced input in
      contract (non-test) code, or each is documented as unreachable.
- [ ] One-time initialization is enforced and uninitialized calls are rejected.

## Documentation completeness

- [ ] Each example README states purpose, public API, and build/test commands.
- [ ] [`audit-scope.md`](./audit-scope.md) boundaries confirmed with the auditor.
- [ ] [`known-issues-log.md`](./known-issues-log.md) is current and handed over
      as the pre-existing-issue baseline.

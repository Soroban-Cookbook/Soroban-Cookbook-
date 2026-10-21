# Audit Prep Checklist

Drive these items to "done" before engaging an external auditor. The checklist
has two parts: **repo-wide gates** that must pass for the whole workspace, and a
**per-example** readiness table for the in-scope intermediate and token
contracts.

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
- [ ] Every in-scope example has a README at all. **→ KI-5** (three examples
      currently have none).
- [ ] `examples/tokens/README.md` category index matches the actual directory
      tree. **→ KI-6** (currently stale: broken links, phantom entries, and
      seven undocumented in-scope examples).

## Per-example readiness

Structural facts below were collected by inspection at preparation time and are
*starting points for the auditor*, not findings. "Tests wired" means `lib.rs`
declares `mod test;` (or an inline `#[cfg(test)]` module) so the test file is
actually compiled.

### Intermediate examples

| Example | README | Test file | Tests wired | `require_auth` uses | `unsafe` | Notes |
| --- | :---: | :---: | :---: | :---: | :---: | --- |
| `02-role-based-access-control` | ✅ | ✅ | ❌ **→ KI-2** | 4 | none | `test.rs` present but not declared in `lib.rs`. |
| `03-pause-unpause` | ✅ | ✅ | ✅ | 3 | none | Review auth on pause/unpause toggles. |
| `03-priority-queue` | ✅ | ✅ | ❌ **→ KI-2** | 0 | none | `test.rs` orphaned; review `unwrap`/panic paths. |
| `ajo` | ❌ **→ KI-5** | ✅ | ✅ | 0 | none | `.expect("Not initialized")` at `src/lib.rs:53,60`; review funds/rotation accounting. |
| `ajo-factory` | ✅ | ✅ | ✅ | 1 | none | Funds accounting + rotation; review panic paths. |
| `event-aggregation` | ✅ | ✅ | ✅ | 1 | none | Review event-batching integrity. |
| `event-history` | ✅ | ✅ | ✅ | 1 | none | Review storage growth / TTL and event integrity. |
| `event-subscriptions` | ✅ | ✅ | ✅ | 2 | none | Review subscriber authorization and dispatch integrity. |
| `iterable-mappings` | ✅ | ✅ | ✅ | 0 | none | Review collection bounds/invariants. |
| `lazy-loading` | ✅ | ✅ | ✅ | 1 | none | `.unwrap()` on oldest cache key at `src/lib.rs:272`; review cache eviction. |
| `multi-sig-patterns` | ✅ | ✅ | ✅ | 6 | none | Review threshold + replay protection. |
| `storage-migration` | ✅ | ✅ | ✅ | 6 | none | Review migration ordering/authorization. |
| `storage-pagination` | ✅ | ✅ | ✅ | 0 | none | `panic!` on out-of-range page index at `src/lib.rs:150`; review cursor bounds. |

### Token examples

| Example | README | Test file | Tests wired | `require_auth` uses | `unsafe` | Notes |
| --- | :---: | :---: | :---: | :---: | :---: | --- |
| `01-sep41-token` | ✅ | ✅ | ❌ **→ KI-2** | 5 | none | Baseline SEP-41; `test.rs` orphaned. |
| `01-vesting-management` | ✅ | ✅ | ✅ | 4 | none | Review revocation auth and cliff/duration arithmetic. |
| `02-minting-strategies` | ✅ | ✅ | ✅ | 3 | none | Review mint-cap enforcement across strategies. |
| `02-sep41-extensions` | ✅ | ✅ | ✅ | 8 | none | Permit extension; review off-chain signature replay/expiration. |
| `03-optimized-operations` | ✅ | ✅ | ✅ | 3 | none | Review optimized paths against baseline semantics. |
| `03-pausable-token` | ❌ **→ KI-5** | ✅ | ✅ | 7 | none | Review pause/unpause gating on transfer paths. |
| `04-mint-burn` | ✅ | ✅ | ✅ | 2 | none | Review mint/burn authorization and supply accounting. |
| `04-snapshot-token` | ✅ | ✅ | ✅ | 4 | none | `.unwrap()` on snapshot history lookup at `src/lib.rs:423,447`. |
| `05-allowance-pattern` | ✅ | ✅ | ✅ | 3 | none | Review allowance expiration and revocation. |
| `05-vesting` | ✅ | ✅ | ✅ | 3 | none | Review release-schedule authorization. |
| `06-reward-token` | ❌ **→ KI-5** | ✅ | ✅ | 6 | none | Review reward accounting across independent pools. |
| `06-token-wrapper` | ✅ | ✅ | ✅ | 3 | none | Review 1:1 backing invariant on deposit/withdraw. |
| `07-token-metadata` | ✅ | ✅ | ✅ | 4 | none | Review admin gating on metadata updates. |
| `08-multi-token-balance-manager` | ✅ | ✅ | ✅ | 3 | none | Review batched cross-contract call safety. |
| `09-optimized-token-ops` | ✅ | ✅ | ❌ **→ KI-2** | 3 | none | `test.rs` orphaned; review batched-transfer correctness. |
| `10-automatic-snapshot-triggers` | ✅ | ✅ | ✅ | 1 | none | Review snapshot pruning for data loss. |
| `10-custom-token` | ✅ | ✅ | ✅ | 8 | none | Review multi-sig + pause integration on top of SEP-41. |
| `10-pausable-permissions` | ✅ | ✅ | ✅ | 8 | none | Review pauser-role / multi-sig-pause model. |
| `token-lock` | ✅ | ✅ | ✅ | 2 | none | `panic!`/`unwrap_or_else(panic!)` at `src/lib.rs:58,62,82,114,126,167` for validation and overflow/underflow guards. |

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
- [ ] Every in-scope example has a README. **→ KI-5**.
- [ ] `examples/tokens/README.md` accurately indexes the token examples. **→
      KI-6**.
- [ ] [`audit-scope.md`](./audit-scope.md) boundaries confirmed with the auditor.
- [ ] [`known-issues-log.md`](./known-issues-log.md) is current and handed over
      as the pre-existing-issue baseline.

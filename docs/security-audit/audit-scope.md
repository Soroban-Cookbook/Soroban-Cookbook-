# Audit Scope & Plan

Finalized scope definition for the external security audit of the Soroban
Cookbook intermediate examples.

## 1. Objective

Provide an external reviewer with a tightly-bounded, well-documented target so
the engagement produces actionable findings rather than scoping churn. Because
the Cookbook is an *educational* resource, a secondary objective is to confirm
that each example teaches a *safe* pattern — an example that compiles but models
an insecure habit is itself a defect.

## 2. In scope

The eight contracts under [`examples/intermediate/`](../../examples/intermediate/):

| Example | Package | Primary concern to review |
| --- | --- | --- |
| `02-role-based-access-control` | `role-based-access-control` | Authorization / privilege escalation |
| `03-pause-unpause` | `pause-unpause` | Emergency-stop correctness, auth on toggles |
| `03-priority-queue` | `priority-queue` | Data-structure invariants, panics/`unwrap` |
| `ajo-factory` | `ajo-factory` | Funds accounting, rotation logic, auth |
| `event-history` | `event-history` | Event integrity, storage growth/TTL |
| `iterable-mappings` | `iterable-mappings` | Index/collection invariants, bounds |
| `multi-sig-patterns` | `multi-sig-patterns` | Threshold logic, signer set, replay |
| `storage-migration` | `storage-migration` | Migration safety, versioning, data loss |

For each example the review covers: the contract source (`src/lib.rs` and
modules), its tests, and its README claims (does the documented behavior match
the code?).

## 3. Out of scope

- All other example categories (`basics`, `advanced`, `defi`, `nfts`,
  `governance`, `tokens`, `hello-world`, `storage`).
- The `webapp/`, `book/`, and `guides/` content.
- `scripts/`, CI workflows, and repository tooling (covered separately by the
  existing `security-audit.yml` advisory scan).
- The `soroban-sdk` dependency itself and the wider Soroban/Stellar platform.
- Economic / game-theoretic modeling of any example protocol.

## 4. Threat model (Soroban-specific)

Reviewers should evaluate each in-scope contract against, at minimum, the
following categories:

1. **Authorization** — Is every state-mutating entry point gated by an
   appropriate `require_auth()` on the correct `Address`? Are admin/role checks
   enforced *in addition to* authentication?
2. **Access control & roles** — Can a role be granted, escalated, or assumed by
   an unauthorized caller? Are role removals and admin transfers safe?
3. **Arithmetic** — Are all `i128`/`u32` arithmetic operations overflow-safe
   (checked math), given that `overflow-checks` are enabled in `test`/`release`
   but should not be relied upon as the only guard?
4. **Storage correctness & TTL** — Are persistent/instance/temporary storage
   choices appropriate? Are entries bumped/expired correctly so data is neither
   prematurely evicted nor unboundedly retained?
5. **Panics & availability** — Do `unwrap()`/`expect()`/`panic!` paths exist on
   attacker-influenced input, turning a recoverable error into a hard trap?
6. **Cross-contract & reentrancy** — For contracts that call out, are
   invariants re-checked after external calls? Is state updated before external
   interaction where appropriate?
7. **Replay & nonce** — For multi-sig / signature flows, are messages bound to a
   nonce, contract, and network so signatures cannot be replayed?
8. **Event integrity** — Do emitted events faithfully reflect state changes
   (no missing or misleading events) without leaking sensitive data?
9. **Initialization** — Is one-time initialization enforced, and are
   uninitialized calls rejected rather than silently using defaults?
10. **Upgrade / migration safety** — For `storage-migration`, can a migration
    corrupt or lose data, or be run by an unauthorized party / out of order?

## 5. Methodology

1. **Reproducible build** — Pin dependencies (see
   [`known-issues-log.md`](./known-issues-log.md), KI-1/KI-4) and record the
   exact toolchain so the auditor builds the same bytes.
2. **Static gates** — `cargo fmt --check`, `cargo clippy --all-targets -- -D
   warnings`, and `cargo audit` must pass (or their failures must be documented
   as known issues).
3. **Test review** — Confirm each example's tests are actually wired and run
   (see KI-2), then assess coverage against the threat model.
4. **Manual review** — Line-by-line review of each in-scope contract against
   section 4.
5. **Targeted dynamic testing** — Where useful, add property-based or fuzz tests
   for arithmetic and collection invariants.

## 6. Severity rubric

| Severity | Definition |
| --- | --- |
| **Critical** | Direct loss/lock of funds or full bypass of authorization. |
| **High** | Privilege escalation, state corruption, or a denial-of-service reachable by an unprivileged caller. |
| **Medium** | Exploitable only under specific preconditions, or with limited impact. |
| **Low** | Defense-in-depth gaps, minor invariant violations, no direct exploit. |
| **Informational** | Style, documentation, or teaching-clarity issues (notably important for an educational repo). |

## 7. Deliverables

- A findings report mapping each issue to an example, severity, and remediation.
- Updates to [`known-issues-log.md`](./known-issues-log.md) for any accepted
  pre-existing issues.
- Confirmation that the [`audit-prep-checklist.md`](./audit-prep-checklist.md)
  repo-wide gates pass at the audited commit.

## 8. Logistics

- **Audited commit:** _to be fixed by maintainers immediately before kickoff._
- **Toolchain:** as declared in [`rust-toolchain.toml`](../../rust-toolchain.toml)
  (`stable`), with the resolved `soroban-sdk` version recorded in the audit's
  reproducible-build notes.
- **Point of contact / timeline:** _to be filled in by maintainers._

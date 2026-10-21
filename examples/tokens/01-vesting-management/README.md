# Vesting Management

This example demonstrates how to implement a secure, production-grade token vesting contract with revocation capabilities, cliff-duration enforcement, and isolated multi-beneficiary support in Soroban.

## Key Concepts

- **Linear Vesting**: Continuous and linear release of tokens over a specified duration after a cliff period.
- **Cliff Period**: A mandatory waiting period during which no tokens are vested or claimable.
- **Revocation**: Admin capability to cancel a schedule mid-vesting, returning unvested tokens to the admin while keeping vested-but-unclaimed tokens claimable by the beneficiary.
- **Multi-Beneficiary Isolation**: Supports managing independent schedules for different beneficiaries, each with their own allocation parameters and token addresses.
- **Arithmetic Safety**: Uses safe, checked arithmetic (`checked_mul`, `checked_add`, `checked_sub`, `checked_div`) to guarantee zero underflow/overflow risk.

---

## Storage Model

The contract uses Soroban's persistent storage to track independent vesting schedules for each beneficiary, and instance storage for global config parameters:

- `Admin`: The privileged admin address allowed to create and revoke schedules (Instance storage).
- `DefaultToken`: An optional default SEP-41 token used if no specific token is provided for a schedule (Instance storage).
- `Schedule(beneficiary)`: The `VestingSchedule` struct for a specific beneficiary (Persistent storage).

---

## Vesting Schedule Schema

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub beneficiary: Address,     // Beneficiary address
    pub total_amount: i128,       // Total tokens allocated
    pub released_amount: i128,    // Total tokens already claimed
    pub start_timestamp: u64,     // Starting timestamp of the schedule
    pub cliff_duration: u64,      // Duration in seconds before any tokens vest
    pub vesting_duration: u64,    // Total duration in seconds over which tokens vest
    pub revoked: bool,            // Whether this schedule has been revoked
    pub token: Address,           // Address of the vested SEP-41 token
}
```

---

## Linear Vesting Calculation

At any block timestamp $T$:
1. If $T < \text{start\_timestamp} + \text{cliff\_duration}$:
   - Vested amount is $0$.
2. If $T \geq \text{start\_timestamp} + \text{vesting\_duration}$:
   - Vested amount is the total allocated $\text{total\_amount}$.
3. Otherwise (mid-vesting):
   - $\text{elapsed\_time} = T - \text{start\_timestamp}$
   - $\text{vested\_amount} = \frac{\text{total\_amount} \times \text{elapsed\_time}}{\text{vesting\_duration}}$

---

## Design Decision: Revocation Flow

When an admin revokes a vesting schedule:
1. The contract calculates the vested amount up to the current timestamp $T_{\text{revoke}}$ using linear interpolation.
2. The `total_amount` is capped exactly at the calculated vested amount ($T_{\text{revoke}}$), and the schedule is marked as `revoked = true`.
3. Any unvested remainder ($\text{total\_amount} - \text{vested\_amount}$) is transferred back to the admin immediately.
4. Beneficiaries retain full rights to any vested-but-unclaimed tokens. They can claim them at any time in the future.
5. No further vesting accrues after revocation.

---

## Contract Interface

> ⚠️ **UNAUdITED EXAMPLE — NOT FOR PRODUCTION USE**
>
> This example code is educational and has **not** been audited. Do **not** deploy it verbatim with real funds. Before any production deployment, obtain a formal security audit, review edge cases for your token, revocation logic, and multi-beneficiary schedule interactions, and perform your own validation.
>
> **Reentrancy:** Both `claim` and `revoke` perform cross-contract `transfer` calls to the vested token after updating schedule state (`released_amount`, `revoked`, `total_amount`). If the token is malicious or hook-bearing, it could call back into `claim`, `revoke`, or another entry point. This example does **not** include an explicit reentrancy guard. In Soroban, reentrancy is limited to the same invocation stack (a callee cannot outlive the caller's frame), but shared-state re-entry within that frame must be protected against for production — add a guard flag in storage set before any token call and cleared after, erroring on re-entry.
>
> **Storage TTL / Data Expiry:** Per-beneficiary `Schedule` entries are stored in **persistent** storage with per-key TTLs. Either explicitly `extend_ttl` on every schedule write (`create_schedule`, `claim`, `revoke`) or run a periodic maintenance call to extend all active schedule keys. Instance entries (`Admin`, `DefaultToken`) are tied to the contract instance TTL and will reset if the instance expires and is restored.

### `initialize`
Initializes the contract with an admin and an optional default token.
```rust
pub fn initialize(env: Env, admin: Address, default_token: Option<Address>) -> Result<(), VestingError>;
```

### `create_schedule`
Creates a vesting schedule for a beneficiary. Only callable by the admin.
```rust
pub fn create_schedule(
    env: Env,
    admin: Address,
    beneficiary: Address,
    total_amount: i128,
    start_timestamp: u64,
    cliff_duration: u64,
    vesting_duration: u64,
    token: Option<Address>,
) -> Result<(), VestingError>;
```

### `claim`
Transfers all currently vested-but-unclaimed tokens to the beneficiary. Requires beneficiary's authorization.
```rust
pub fn claim(env: Env, beneficiary: Address) -> Result<i128, VestingError>;
```

### `revoke`
Revokes an active vesting schedule. Only callable by the admin.
```rust
pub fn revoke(env: Env, admin: Address, beneficiary: Address) -> Result<(), VestingError>;
```

### `vested_amount`
Read-only query to get the total number of tokens vested up to the current ledger timestamp.
```rust
pub fn vested_amount(env: Env, beneficiary: Address) -> Result<i128, VestingError>;
```

### `releasable_amount`
Read-only query to get the remaining claimable amount ($\text{vested\_amount} - \text{released\_amount}$).
```rust
pub fn releasable_amount(env: Env, beneficiary: Address) -> Result<i128, VestingError>;
```

### `transfer_admin`
Allows the current admin to transfer administrative capabilities to a new address.
```rust
pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), VestingError>;
```

---

## Build

To compile the contract into a WASM release target:

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release

# Or from the repository root
cargo build -p vesting-management --target wasm32-unknown-unknown --release
```

---

## Test

To run the unit tests:

```bash
# From this directory
cargo test

# Or from the repository root
cargo test -p vesting-management
```

| Test Name | Verifies |
|-----------|----------|
| `test_initialize` | Admin initialization and double-initialization block |
| `test_create_schedule_admin_only` | Correct administrative authorization required to create schedules |
| `test_duplicate_schedule_error` | Duplicate schedule creation prevention for the same beneficiary |
| `test_invalid_schedule_parameters` | Edge-case input parameters validation (e.g. cliff > duration) |
| `test_cliff_enforcement` | Enforces 0 claims and vested amount before cliff expires |
| `test_linear_vesting_calculation` | Asserts exact linear interpolation mid-vesting |
| `test_multiple_claims` | Updates released amount and transfers tokens across sequential claims |
| `test_full_vesting` | Validates full release after full duration and handles redundant claims |
| `test_claim_auth_failure` | Restricts claims to the authenticated beneficiary |
| `test_multiple_beneficiaries_isolation` | Verifies isolated tracking for multiple concurrent beneficiaries |
| `test_revoke_schedule_mid_vesting` | Mid-vesting revocation flows, unvested transfers, and claim freeze |
| `test_revoke_before_cliff` | Returns 100% of tokens to the admin if revoked before cliff |
| `test_transfer_admin` | Validates admin transfer flow and admin privilege updates |

---

## Project Structure

```
01-vesting-management/
├── Cargo.toml      # Crate manifest
├── README.md       # This documentation
└── src/
    ├── lib.rs      # Vesting Contract logic and struct schemas
    └── test.rs     # Complete unit test suite (13 tests)
```

---

## Next Steps

- [03-optimized-operations](../03-optimized-operations/) — optimized batched operations.
- [06-token-wrapper](../06-token-wrapper/) — 1:1 asset wrapper.

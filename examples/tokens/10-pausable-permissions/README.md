# 10 — Pausable Permissions

A permission system for pausing contracts, demonstrating three complementary
mechanisms that can be composed together.

## Mechanisms

### 1. Pauser Role
A dedicated `pauser` address (separate from the admin) can halt the contract
via `pause_as_pauser`. Only the admin can assign or revoke the role, and only
the admin can unpause. This separates the ability to halt operations from full
admin control.

### 2. Multi-Sig Pause
A pause requires `M` of `N` designated guardians to submit a vote before the
contract halts. Prevents a single compromised key from pausing a production
system unilaterally. Votes reset automatically after triggering a pause.

### 3. Time-Limited Pause
`pause_for(duration)` applies a pause that automatically expires after
`duration` seconds. Guarded operations check the expiry timestamp and resume
without a transaction — removing the risk of an indefinitely locked contract.

## Contract Functions

### Setup
| Function | Description |
|---|---|
| `initialize(admin)` | Deploy with admin; starts unpaused |
| `set_pauser(admin, pauser)` | Assign the pauser role |
| `set_guardians(admin, guardians, threshold)` | Configure multi-sig pause |

### Pausing
| Function | Who | Description |
|---|---|---|
| `pause_as_pauser(pauser)` | Pauser | Indefinite pause via role |
| `guardian_vote_pause(guardian)` | Guardian | Cast one multi-sig vote; auto-pauses at threshold |
| `pause_for(caller, duration)` | Admin or Pauser | Time-limited pause (auto-lifts after `duration` seconds) |

### Unpausing
| Function | Who | Description |
|---|---|---|
| `unpause(admin)` | Admin | Manual unpause; clears votes and expiry |

### Guards & Queries
| Function | Description |
|---|---|
| `assert_not_paused()` | Returns `ContractPaused` if currently paused; use in protected operations |
| `is_paused()` | Read-only; respects time-limited expiry |
| `pause_expires_at()` | Expiry timestamp (0 = no expiry) |
| `pending_votes()` | Current guardian vote count |
| `get_pauser()` | Current pauser address |
| `get_guardians()` | Guardian list |
| `get_threshold()` | Multi-sig threshold |

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Contract not yet initialized |
| 3 | `NotAdmin` | Caller is not the admin |
| 4 | `NotPauser` | Caller is not the designated pauser |
| 5 | `ContractPaused` | Operation blocked — contract is paused |
| 6 | `AlreadyInState` | Already paused / already unpaused |
| 7 | `NotGuardian` | Address not in the guardian list |
| 8 | `AlreadyVoted` | Guardian already voted in this round |
| 9 | `InvalidConfig` | Threshold is 0 or exceeds guardian count |
| 10 | `InvalidDuration` | `pause_for` duration must be > 0 |

## Build

```bash
cargo build --target wasm32v1-none --release -p pausable-permissions
```

## Test

```bash
cargo test -p pausable-permissions
```

## Use Cases

- **DeFi protocols** — pause transfers during an exploit or oracle failure
- **Governance contracts** — guardians halt execution if a malicious proposal passes
- **Maintenance windows** — admin schedules a time-limited pause for an upgrade

## Prerequisites

- [03-authentication](../../basics/03-authentication/) — `require_auth()` patterns
- [04-events](../../basics/04-events/) — structured event emission
- [03-pausable-token](../03-pausable-token/) — single-admin pause on a token
- [Tokens README](../README.md) — category overview

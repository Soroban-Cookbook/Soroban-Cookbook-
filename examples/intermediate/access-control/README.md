# Access Control Guide

This intermediate example demonstrates a complete access control system combining **Role-Based Access Control (RBAC)**, **Multi-Signature (Multisig)** approval workflows, and **Timelock** delays in a single Soroban smart contract.

## What You'll Learn

- **RBAC**: Grant, revoke, and check roles (Admin, Auditor, Operator, User)
- **Multisig**: M-of-N threshold approvals via a shared signers list
- **Timelock**: Mandatory delay between proposal creation and execution
- **Combined flows**: Secure governance with layered authorization

## Architecture

The contract layers three access control patterns:

```
RBAC          → Who can configure the contract (Admin, Operator, Auditor)
Multisig      → Who must approve sensitive actions (M-of-N signers)
Timelock      → How long actions wait before execution (configurable delay)
```

## Role Hierarchy

| Role     | Level | Permissions                              |
|----------|-------|------------------------------------------|
| `Admin`  | 3     | Grant/revoke roles, manage signers, update timelock, pause |
| `Auditor`| 2     | View proposals and roles                 |
| `Operator`| 1    | Create and approve proposals             |
| `User`   | 0     | Read-only access                         |

## Multi-Sig Configuration

The contract uses a signers list stored in instance storage. Proposals require majority approval (more than half of signers) before execution.

## Threat Model

### RBAC protects against:
- Unauthorized privilege escalation
- Role confusion between admin and operator flows
- Accidental exposure of sensitive functions to generic callers

### Multisig protects against:
- Single compromised key authorizing a sensitive action
- Rogue insiders acting alone
- Single administrative account abuse

### Timelock protects against:
- Rushed or impulsive upgrades
- Instant execution of large fund movements
- Hidden admin actions without review time

### Combined protections:
- Even if an Admin key is compromised, multisig + timelock provide reaction time
- Role changes are logged and auditable via events
- Pause mechanism provides emergency circuit-breaker

## Recommended Defaults

- Use **2-of-3** or **3-of-5** signer thresholds for treasury and governance
- Set timelock delay between **1 hour and 24 hours** for standard operations
- Keep the **Admin role** separate from routine **Operator** actions
- Emit events for all role changes, signer updates, and proposal lifecycle events
- Periodically rotate signer keys and maintain an off-chain signer registry

## API Reference

### Initialization

```rust
client.initialize(&admin, &threshold, &signers, &timelock_delay);
```

### RBAC

```rust
client.grant_role(&admin, &account, &Role::Operator);
client.revoke_role(&admin, &account);
client.has_role(&account, &Role::Admin);
```

### Multisig

```rust
client.add_signer(&admin, &signer);
client.remove_signer(&admin, &signer);
client.create_proposal(&proposer, &Symbol::new(&env, "upgrade"));
client.approve(&signer, &proposal_id);
```

### Timelock

```rust
client.set_timelock_delay(&admin, &3600);
client.get_timelock_delay();
client.set_pause(&admin, &true);
```

## Governance Flow Example

1. **Admin** configures the contract with initial signers and timelock delay
2. **Operator** creates a proposal for a sensitive action (e.g., contract upgrade)
3. **Signers** review and approve the proposal over multiple transactions
4. Once the timelock delay expires, any caller can **execute** the proposal
5. **Auditor** monitors events for compliance and anomaly detection

## Run Tests

```bash
cargo test -p access-control
```

## Build

```bash
cargo build --target wasm32v1-none --release -p access-control
```

## Related Examples

- [`02-role-based-access-control`](../02-role-based-access-control/) — Basic RBAC patterns
- [`multi-sig-patterns`](../multi-sig-patterns/) — Multi-party authorization
- [`02-timelock`](../../advanced/02-timelock/) — Time-delayed execution

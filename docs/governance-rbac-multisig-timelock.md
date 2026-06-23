# RBAC, Multisig, and Timelock Patterns

Secure authorization in Soroban often blends three patterns:
- **Role-Based Access Control (RBAC)** for tiered privileges
- **Multi-Signature (Multisig)** for shared control and distributed trust
- **Timelock** for delayed execution and governance cooling-off periods

This page documents those patterns with examples, threat models, and recommended deployment defaults.

## Why these patterns matter

Authorization decisions are the first line of defense for any contract with privileged functionality. Poor access control can lead to:
- lost funds
- unauthorized upgrades or configuration changes
- governance capture by a single compromised key
- fast execution of dangerous actions without time to react

Using RBAC, multisig, and timelock together helps create a layered security model for both operational and governance flows.

## Pattern overview

| Pattern | What it protects | Strongest use cases |
| --- | --- | --- |
| RBAC | Role-specific privileges and separation of duties | Admin dashboards, treasury management, contract configuration |
| Multisig | Shared ownership and threshold authorization | Treasuries, DAO proposals, emergency controls |
| Timelock | Delayed execution and review windows | Protocol upgrades, large withdrawals, governance decisions |

## 1. Role-Based Access Control (RBAC)

### What it is

RBAC assigns roles to addresses and checks role membership before executing privileged functions. A contract can store authorized roles in persistent storage and enforce them at runtime.

### Example

Use `persistent` storage for role assignments and compare the caller's role to an allowed list.

```rust
#[contracttype]
#[repr(u32)]
pub enum Role { Admin = 0, Moderator = 1, User = 2 }

fn require_role(env: &Env, caller: &Address, allowed: &[Role])
    -> Result<(), AuthError>
{
    let role: Role = env.storage().persistent()
        .get(&DataKey::UserRole(caller.clone()))
        .unwrap_or(Role::User);
    for r in allowed {
        if role as u32 <= *r as u32 { return Ok(()); }
    }
    Err(AuthError::InsufficientRole)
}

pub fn admin_action(env: Env, caller: Address, value: u64)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    require_role(&env, &caller, &[Role::Admin])?;
    Ok(value * 2)
}
```

### Threat model

RBAC protects against:
- a compromised low-privilege account performing admin actions
- accidental privilege escalation from generic auth checks
- an attacker using a reused key with broader access than intended

It does not protect against:
- a compromised account that already holds a high privilege role
- insecure role assignment or upgrade paths

### Recommended defaults

- store roles in **persistent** storage so assignments persist across upgrades
- prefer explicit role checks for each privileged action
- keep role hierarchy simple: Admin, Operator, Auditor, Executor
- avoid overly broad roles like `SuperAdmin` unless absolutely necessary
- prevent roles from being assigned by untrusted callers

## 2. Multi-Signature (Multisig)

### What it is

Multisig requires multiple independent signers to approve a sensitive action. Common forms include:
- **N-of-N** where every configured signer must approve
- **M-of-N** where a threshold of signers is required
- **Proposal-based approval** with a separate create/approve/execute workflow

### Example

This pattern is best illustrated by a proposal workflow.

```rust
pub fn create_proposal(env: Env, proposer: Address) -> Result<u32, AuthError> {
    proposer.require_auth();
    // store proposal metadata, approvals list, and execution state
    Ok(proposal_id)
}

pub fn approve(env: Env, proposal_id: u32, signer: Address)
    -> Result<(), AuthError>
{
    signer.require_auth();
    // validate signer is authorized and has not already approved
    Ok(())
}

pub fn execute(env: Env, proposal_id: u32, executor: Address)
    -> Result<bool, AuthError>
{
    executor.require_auth();
    // execute only after threshold approvals are collected
    Ok(true)
}
```

### Threat model

Multisig protects against:
- a single compromised key authorizing a sensitive action
- rogue insiders acting alone
- a single administrative account being abused

It does not protect against:
- coordinated compromise of the threshold number of signers
- a signer that is malicious by design
- bad signer key management practices

### Recommended defaults

- choose a threshold that balances security and availability (e.g. 2-of-3 or 3-of-5)
- include at least one independent guardian or auditor signer when possible
- validate threshold correctness at initialization
- prevent duplicate approvals and replay
- store signer list in **persistent** storage
- document signer roles clearly alongside the multisig contract

## 3. Timelock

### What it is

A timelock introduces a delay between scheduling and executing an action. This gives the community or operators time to review and react before a sensitive change occurs.

### Example

```rust
pub fn set_time_lock(env: Env, admin: Address, unlock_time: u64)
    -> Result<(), AuthError>
{
    admin.require_auth();
    require_admin(&env, &admin)?;
    env.storage().instance().set(&DataKey::TimeLock, &unlock_time);
    Ok(())
}

pub fn time_locked_action(env: Env, caller: Address)
    -> Result<u64, AuthError>
{
    caller.require_auth();
    let unlock: u64 = env.storage().instance()
        .get(&DataKey::TimeLock).unwrap_or(0);
    if env.ledger().timestamp() < unlock {
        return Err(AuthError::TimeLocked);
    }
    Ok(env.ledger().timestamp())
}
```

### Threat model

Timelock protects against:
- rushed or impulsive upgrades
- instant execution of large fund movements
- hidden admin actions without review time

It does not protect against:
- a compromised or malicious admin scheduling unsafe actions early
- threshold collusion in a multisig that also controls execution

### Recommended defaults

- use a delay long enough for review but short enough for operations (e.g. 24-72 hours for treasury actions)
- store timelock configuration in **instance** storage for contract-wide settings
- require explicit scheduling and execution calls
- log queue, cancel, and execute events for auditability
- combine timelocks with multisig or RBAC for stronger governance

## 4. Combined governance flows

A solid governance system often layers RBAC, multisig, and timelock.

### Example flow: secure treasury change

1. **Role assignment**: `Admin` can propose a treasury action; `Treasurer` can execute routine transfers.
2. **Proposal creation**: `Admin` or `Operator` creates a proposal for a large transfer.
3. **Multisig approval**: three council members must approve the proposal.
4. **Timelock delay**: once approved, the action waits for the configured delay.
5. **Execution**: after the delay expires, any authorized executor can finalize the transaction.

### Role table

| Role | Responsibility | Example permission |
| --- | --- | --- |
| Admin | Governance configuration and role assignment | Add/remove roles, set thresholds |
| Operator | Routine protocol operations | Schedule transfers, pause/unpause services |
| Treasurer | Execute approved treasury actions | Withdraw funds after multisig approval |
| Auditor | Review proposals and cancellations | Inspect pending and executed actions |
| Guardian | Emergency pause/cancellation | Cancel a proposal or activate safe mode |

### Recommended structure

- Keep the **power to assign roles** separate from the **power to execute funds**.
- Use a **small, trusted set of admin addresses** for governance configuration.
- Use a **larger threshold** for funds and upgrades than for routine operational changes.
- Use **timelock** only on actions that require external review or community notification.

## 5. Secure defaults for deployment

### Authorization defaults

- require `caller.require_auth()` for every state-changing entrypoint
- validate role membership explicitly for privileged flows
- avoid implicit checks based solely on caller-provided addresses
- prefer `persistent` storage for authorization state

### Signer and threshold defaults

- 2-of-3 is a common minimum for treasury and governance actions
- 3-of-5 or 4-of-7 is better for higher-value systems
- do not use 1-of-N for critical actions
- rotate signer keys periodically and maintain an off-chain signer registry

### Timelock defaults

- schedule a delay that matches the risk profile of the action
- use a distinct delay for governance proposals versus emergency responses
- require a separate `execute` call after the delay expires
- allow cancellation or veto by an independent guardian when needed

### Monitoring and auditability

- emit events for role changes, signer updates, proposal creation, approvals, cancellations, and execution
- maintain off-chain logs for governance actions
- test failed authorization and timelock paths explicitly
- review contract upgrades and role assignments before deployment

## 6. Example references in this repository

- [`examples/basics/03-authentication/`](../examples/basics/03-authentication/) — RBAC and basic authentication patterns
- [`examples/intermediate/multi-sig-patterns/`](../examples/intermediate/multi-sig-patterns/) — proposal-based multisig and authorization vectors
- [`examples/advanced/01-multi-party-auth/`](../examples/advanced/01-multi-party-auth/) — threshold signatures and multi-party approval workflows
- [`examples/advanced/02-timelock/`](../examples/advanced/02-timelock/) — time-delayed execution and timelock controls

## 7. Quick comparison

| Pattern | Best fit | Security benefit | Common limitation |
| --- | --- | --- | --- |
| RBAC | Many privilege tiers | Separation of duties | High privilege roles still risky if compromised |
| Multisig | Shared ownership | Single-key compromise resistance | More complex state and workflow |
| Timelock | Governance review | Reaction window for stakeholders | Adds execution latency |

## 8. Practical guidance

- prefer layered protections: RBAC + multisig + timelock where risk is highest
- document the purpose of each role and threshold clearly in contract docs
- keep the approval process auditable and cancelable
- avoid mixing execution and configuration rights in a single account
- build test cases that cover unauthorized access, duplicate approvals, timelock enforcement, and cancellation paths

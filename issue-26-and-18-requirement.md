
#18 Implement Custom Authorization Logic Patterns
Repo Avatar
Soroban-Cookbook/Soroban-Cookbook-
Overview
Demonstrate how to build custom authorization logic beyond basic require_auth(), including role-based access control, time-based restrictions, and conditional authorization patterns.

Context
While require_auth() handles address verification, many contracts need more sophisticated authorization:

Role-based access control (RBAC)
Time-locked operations
Permission hierarchies
Conditional authorization based on state
Allowlist/blocklist patterns
Objectives
Show custom authorization patterns
Implement role-based access control basics
Demonstrate time-based authorization
Create extensible authorization frameworks
Acceptance Criteria
 Custom authorization functions implemented:
Role-based permissions (admin, moderator, user roles)
Time-based restrictions (time-locks, deadlines)
Conditional auth based on contract state
 At least 3 different custom auth patterns:
Role-based: Different permission levels
Time-based: Operations only valid at certain times
State-based: Auth depends on contract state
 Proper storage of authorization data (roles, permissions)
 Code comments explaining:
When custom auth is needed
How to design auth systems
Security best practices
Extensibility patterns
 Combines require_auth with custom logic
Technical Details
Patterns to demonstrate:

// Role-based authorization
pub fn admin_only(env: Env, caller: Address) {
    caller.require_auth();
    
    let role = get_role(&env, &caller);
    if role != Role::Admin {
        panic!("Admin access required");
    }
    
    // Admin-only operation
}

// Time-based authorization
pub fn time_locked_action(env: Env, caller: Address) {
    caller.require_auth();
    
    let current_time = env.ledger().timestamp();
    let unlock_time = get_unlock_time(&env);
    
    if current_time < unlock_time {
        panic!("Action is time-locked");
    }
    
    // Time-restricted operation
}
Role-based system to implement:

Define Role enum (Admin, Moderator, User)
Store roles in persistent storage
Functions to grant/revoke roles
Permission checking helpers
Time-based examples:

Time-locked withdrawals
Deadline-based voting
Cooldown periods between actions
Security Considerations
Must document:

Always call require_auth first
Custom checks come after auth verification
Store authorization data securely
Test edge cases thoroughly
Consider gas costs of complex auth
Avoid overly complex authorization logic
Implementation Notes
Keep auth logic clear and auditable
Separate auth checks from business logic
Make auth extensible but not overcomplicated
Document why each auth check exists
Priority
🟡 Medium - Advanced but practical pattern




#26 Add Events for State Change Tracking
Repo Avatar
Soroban-Cookbook/Soroban-Cookbook-
Overview
Emit events on important state changes to create audit trail and enable monitoring.

Acceptance Criteria
 Transfer events implemented
 State update events added
 Admin action events included
 Audit trail pattern demonstrated
Priority
🟡 Medium



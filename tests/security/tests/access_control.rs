//! Access-control security tests (Issue #799).
//!
//! Adversarial tests for the three access-control building blocks used across
//! the cookbook:
//!
//! - **RBAC** (`examples/intermediate/02-role-based-access-control`):
//!   privilege escalation, role hijacking, spoofed callers.
//! - **Multisig** (`examples/intermediate/multi-sig-patterns`):
//!   invalid signer sets, missing approvals, signature spoofing and replay.
//! - **Timelock** (`examples/advanced/02-timelock`):
//!   delay skips, re-queue resets, cancelled/replayed operations, admin takeover.
//!
//! Spoofing tests do not use `mock_all_auths`. They use `mock_auths` so that only
//! the attacker's signature, bound to the exact invocation, is present. The host
//! must then reject the call with `Error(Auth, InvalidAction)`.

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, Val, Vec,
};

/// Provide exactly one signature: `signer` authorizing `contract.fn_name(args)`.
fn sign_as(env: &Env, signer: &Address, contract: &Address, fn_name: &str, args: Vec<Val>) {
    env.mock_auths(&[MockAuth {
        address: signer,
        invoke: &MockAuthInvoke {
            contract,
            fn_name,
            args,
            sub_invokes: &[],
        },
    }]);
}

// ---------------------------------------------------------------------------
// RBAC: privilege escalation
// ---------------------------------------------------------------------------

mod rbac {
    use super::*;
    use role_based_access_control::{
        RbacError, Role, RoleBasedAccessControl, RoleBasedAccessControlClient,
    };

    const ALL_ROLES: [Role; 4] = [Role::User, Role::Moderator, Role::Admin, Role::Owner];

    struct Setup<'a> {
        env: Env,
        client: RoleBasedAccessControlClient<'a>,
        owner: Address,
        admin: Address,
        moderator: Address,
    }

    fn setup<'a>() -> Setup<'a> {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(RoleBasedAccessControl, ());
        let client = RoleBasedAccessControlClient::new(&env, &id);
        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let moderator = Address::generate(&env);
        client.initialize(&owner);
        client.grant_role(&owner, &admin, &Role::Admin);
        client.grant_role(&owner, &moderator, &Role::Moderator);
        Setup {
            env,
            client,
            owner,
            admin,
            moderator,
        }
    }

    fn unauthorized<T>(r: Result<T, Result<RbacError, soroban_sdk::InvokeError>>) {
        assert_eq!(r.err(), Some(Ok(RbacError::Unauthorized)));
    }

    #[test]
    fn outsider_cannot_grant_any_role() {
        let s = setup();
        let attacker = Address::generate(&s.env);
        for role in ALL_ROLES {
            unauthorized(s.client.try_grant_role(&attacker, &attacker, &role));
        }
        assert!(!s.client.has_role(&attacker, &Role::Moderator));
    }

    #[test]
    fn moderator_cannot_escalate_self_or_others() {
        let s = setup();
        let target = Address::generate(&s.env);
        for role in [Role::Moderator, Role::Admin, Role::Owner] {
            unauthorized(s.client.try_grant_role(&s.moderator, &s.moderator, &role));
            unauthorized(s.client.try_grant_role(&s.moderator, &target, &role));
        }
        assert!(!s.client.has_role(&s.moderator, &Role::Admin));
    }

    #[test]
    fn admin_cannot_mint_admins_or_owners() {
        let s = setup();
        let target = Address::generate(&s.env);
        unauthorized(s.client.try_grant_role(&s.admin, &target, &Role::Admin));
        unauthorized(s.client.try_grant_role(&s.admin, &target, &Role::Owner));
        unauthorized(s.client.try_grant_role(&s.admin, &s.admin, &Role::Owner));
        assert!(!s.client.has_role(&target, &Role::Admin));
        assert!(!s.client.has_role(&s.admin, &Role::Owner));
    }

    /// Regression: `grant_role` used to check only the *new* role, so an Admin
    /// could "grant" `User` to the Owner and strip ownership.
    #[test]
    fn admin_cannot_demote_owner_via_grant() {
        let s = setup();
        for role in [Role::User, Role::Moderator] {
            unauthorized(s.client.try_grant_role(&s.admin, &s.owner, &role));
        }
        assert!(s.client.has_role(&s.owner, &Role::Owner));
    }

    #[test]
    fn admin_cannot_downgrade_peer_admin_via_grant() {
        let s = setup();
        let peer = Address::generate(&s.env);
        s.client.grant_role(&s.owner, &peer, &Role::Admin);
        unauthorized(s.client.try_grant_role(&s.admin, &peer, &Role::User));
        assert!(s.client.has_role(&peer, &Role::Admin));
    }

    #[test]
    fn admin_cannot_revoke_owner_or_peer_admin() {
        let s = setup();
        let peer = Address::generate(&s.env);
        s.client.grant_role(&s.owner, &peer, &Role::Admin);
        unauthorized(s.client.try_revoke_role(&s.admin, &s.owner));
        unauthorized(s.client.try_revoke_role(&s.admin, &peer));
        assert!(s.client.has_role(&s.owner, &Role::Owner));
        assert!(s.client.has_role(&peer, &Role::Admin));
    }

    #[test]
    fn moderator_cannot_revoke_anyone() {
        let s = setup();
        let user = Address::generate(&s.env);
        for target in [&user, &s.moderator, &s.admin, &s.owner] {
            unauthorized(s.client.try_revoke_role(&s.moderator, target));
        }
    }

    #[test]
    fn revoked_admin_loses_all_privileges() {
        let s = setup();
        assert_eq!(s.client.admin_action(&s.admin, &21), 42);

        s.client.revoke_role(&s.owner, &s.admin);

        unauthorized(s.client.try_admin_action(&s.admin, &21));
        unauthorized(s.client.try_grant_role(
            &s.admin,
            &Address::generate(&s.env),
            &Role::Moderator,
        ));
        unauthorized(s.client.try_revoke_role(&s.admin, &s.moderator));
    }

    #[test]
    fn privileged_action_requires_admin_role() {
        let s = setup();
        let user = Address::generate(&s.env);
        unauthorized(s.client.try_admin_action(&user, &1));
        unauthorized(s.client.try_admin_action(&s.moderator, &1));
    }

    #[test]
    fn reinitialize_cannot_hijack_ownership() {
        let s = setup();
        let attacker = Address::generate(&s.env);
        assert_eq!(
            s.client.try_initialize(&attacker).err(),
            Some(Ok(RbacError::AlreadyInitialized))
        );
        assert!(!s.client.has_role(&attacker, &Role::Owner));
    }

    #[test]
    fn uninitialized_contract_rejects_role_management() {
        let env = Env::default();
        env.mock_all_auths();
        let client =
            RoleBasedAccessControlClient::new(&env, &env.register(RoleBasedAccessControl, ()));
        let a = Address::generate(&env);
        unauthorized(client.try_grant_role(&a, &a, &Role::Owner));
        unauthorized(client.try_revoke_role(&a, &a));
    }

    /// Attacker names the Owner as `caller` but signs with their own key.
    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_owner_caller_rejected() {
        let s = setup();
        let attacker = Address::generate(&s.env);
        let args = (&s.owner, &attacker, Role::Admin).into_val(&s.env);
        sign_as(&s.env, &attacker, &s.client.address, "grant_role", args);
        s.client.grant_role(&s.owner, &attacker, &Role::Admin);
    }

    /// The Owner's signature covers the exact arguments; it cannot be reused to
    /// grant a higher role than the one approved.
    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn owner_signature_bound_to_granted_role() {
        let s = setup();
        let alice = Address::generate(&s.env);
        let args = (&s.owner, &alice, Role::Moderator).into_val(&s.env);
        sign_as(&s.env, &s.owner, &s.client.address, "grant_role", args);
        s.client.grant_role(&s.owner, &alice, &Role::Admin);
    }
}

// ---------------------------------------------------------------------------
// Multisig: invalid signer sets, missing approvals, signature spoofing
// ---------------------------------------------------------------------------

mod multisig {
    use super::*;
    use multi_sig_patterns::{AuthError, MultiPartyAuth, MultiPartyAuthClient};

    struct Setup<'a> {
        env: Env,
        client: MultiPartyAuthClient<'a>,
        s1: Address,
        s2: Address,
        s3: Address,
    }

    /// 2-of-3 multisig.
    fn setup<'a>() -> Setup<'a> {
        let env = Env::default();
        env.mock_all_auths();
        let client = MultiPartyAuthClient::new(&env, &env.register(MultiPartyAuth, ()));
        let (s1, s2, s3) = (
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        );
        client.initialize(
            &2,
            &Vec::from_array(&env, [s1.clone(), s2.clone(), s3.clone()]),
        );
        Setup {
            env,
            client,
            s1,
            s2,
            s3,
        }
    }

    fn approvals(s: &Setup, id: u32) -> u32 {
        s.client.get_proposal(&id).unwrap().approvals.len()
    }

    #[test]
    fn invalid_signer_sets_rejected() {
        let env = Env::default();
        let a = Address::generate(&env);
        let b = Address::generate(&env);
        let cases: [(u32, Vec<Address>); 4] = [
            (0, Vec::from_array(&env, [a.clone(), b.clone()])), // zero threshold
            (3, Vec::from_array(&env, [a.clone(), b.clone()])), // unreachable threshold
            (1, Vec::new(&env)),                                // empty signer set
            (0, Vec::new(&env)),
        ];
        for (threshold, signers) in cases {
            let client = MultiPartyAuthClient::new(&env, &env.register(MultiPartyAuth, ()));
            assert_eq!(
                client.try_initialize(&threshold, &signers).err(),
                Some(Ok(AuthError::InvalidThreshold))
            );
        }
    }

    #[test]
    fn reinitialize_cannot_replace_signer_set() {
        let s = setup();
        let attacker = Address::generate(&s.env);
        let evil = Vec::from_array(&s.env, [attacker.clone()]);
        assert_eq!(
            s.client.try_initialize(&1, &evil).err(),
            Some(Ok(AuthError::AlreadyInitialized))
        );
        assert_eq!(
            s.client.try_create_proposal(&attacker).err(),
            Some(Ok(AuthError::NotAuthorized))
        );
    }

    #[test]
    fn outsider_with_valid_signature_is_not_a_signer() {
        let s = setup();
        let outsider = Address::generate(&s.env);
        let id = s.client.create_proposal(&s.s1);

        let not_authorized = Some(Ok(AuthError::NotAuthorized));
        assert_eq!(
            s.client.try_create_proposal(&outsider).err(),
            not_authorized
        );
        assert_eq!(s.client.try_approve(&id, &outsider).err(), not_authorized);
        assert_eq!(s.client.try_cancel(&id, &outsider).err(), not_authorized);
        assert_eq!(approvals(&s, id), 0);
    }

    #[test]
    fn missing_approvals_block_execution() {
        let s = setup();
        let id = s.client.create_proposal(&s.s1);
        let not_met = Some(Ok(AuthError::ThresholdNotMet));

        assert_eq!(s.client.try_execute(&id, &s.s1).err(), not_met);
        s.client.approve(&id, &s.s1);
        assert_eq!(s.client.try_execute(&id, &s.s1).err(), not_met);

        s.client.approve(&id, &s.s2);
        assert!(s.client.execute(&id, &s.s1));
    }

    #[test]
    fn repeated_approval_cannot_fake_quorum() {
        let s = setup();
        let id = s.client.create_proposal(&s.s1);
        s.client.approve(&id, &s.s1);
        assert_eq!(
            s.client.try_approve(&id, &s.s1).err(),
            Some(Ok(AuthError::AlreadyApproved))
        );
        assert_eq!(approvals(&s, id), 1);
        assert_eq!(
            s.client.try_execute(&id, &s.s1).err(),
            Some(Ok(AuthError::ThresholdNotMet))
        );
    }

    /// A signer listed twice must still count as one vote.
    #[test]
    fn duplicate_signer_entries_do_not_inflate_voting_power() {
        let env = Env::default();
        env.mock_all_auths();
        let client = MultiPartyAuthClient::new(&env, &env.register(MultiPartyAuth, ()));
        let (a, b) = (Address::generate(&env), Address::generate(&env));
        client.initialize(&2, &Vec::from_array(&env, [a.clone(), a.clone(), b]));

        let id = client.create_proposal(&a);
        client.approve(&id, &a);
        assert_eq!(
            client.try_approve(&id, &a).err(),
            Some(Ok(AuthError::AlreadyApproved))
        );
        assert_eq!(
            client.try_execute(&id, &a).err(),
            Some(Ok(AuthError::ThresholdNotMet))
        );
    }

    /// `execute` is permissionless by design (quorum is the gate), so a
    /// non-signer executor must still be unable to skip the threshold.
    #[test]
    fn outsider_executor_cannot_bypass_threshold() {
        let s = setup();
        let outsider = Address::generate(&s.env);
        let id = s.client.create_proposal(&s.s1);
        s.client.approve(&id, &s.s1);
        assert_eq!(
            s.client.try_execute(&id, &outsider).err(),
            Some(Ok(AuthError::ThresholdNotMet))
        );
    }

    #[test]
    fn executed_proposal_cannot_be_replayed_or_reapproved() {
        let s = setup();
        let id = s.client.create_proposal(&s.s1);
        s.client.approve(&id, &s.s1);
        s.client.approve(&id, &s.s2);
        s.client.execute(&id, &s.s1);

        let done = Some(Ok(AuthError::AlreadyExecuted));
        assert_eq!(s.client.try_execute(&id, &s.s2).err(), done);
        assert_eq!(s.client.try_approve(&id, &s.s3).err(), done);
        assert_eq!(s.client.try_cancel(&id, &s.s3).err(), done);
    }

    #[test]
    fn cancelled_proposal_cannot_execute_even_with_quorum() {
        let s = setup();
        let id = s.client.create_proposal(&s.s1);
        s.client.approve(&id, &s.s1);
        s.client.approve(&id, &s.s2);
        s.client.cancel(&id, &s.s3);

        let cancelled = Some(Ok(AuthError::ProposalCancelled));
        assert_eq!(s.client.try_execute(&id, &s.s1).err(), cancelled);
        assert_eq!(s.client.try_approve(&id, &s.s3).err(), cancelled);
        assert!(!s.client.get_proposal(&id).unwrap().executed);
    }

    #[test]
    fn unknown_proposal_rejected() {
        let s = setup();
        let missing = Some(Ok(AuthError::ProposalNotFound));
        assert_eq!(s.client.try_approve(&99, &s.s1).err(), missing);
        assert_eq!(s.client.try_execute(&99, &s.s1).err(), missing);
    }

    /// Attacker submits an approval on behalf of `s1`, signed with their own key.
    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_signer_approval_rejected() {
        let s = setup();
        let id = s.client.create_proposal(&s.s1);
        let attacker = Address::generate(&s.env);
        let args = (id, &s.s1).into_val(&s.env);
        sign_as(&s.env, &attacker, &s.client.address, "approve", args);
        s.client.approve(&id, &s.s1);
    }

    /// `s1`'s approval of proposal 0 cannot be replayed against proposal 1.
    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn approval_signature_bound_to_proposal_id() {
        let s = setup();
        s.client.create_proposal(&s.s1);
        let target = s.client.create_proposal(&s.s1);
        let args = (0u32, &s.s1).into_val(&s.env);
        sign_as(&s.env, &s.s1, &s.client.address, "approve", args);
        s.client.approve(&target, &s.s1);
    }

    /// Only one of two required parties signs.
    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn multi_auth_action_rejects_partial_signatures() {
        let s = setup();
        let signers = Vec::from_array(&s.env, [s.s1.clone(), s.s2.clone()]);
        let args = (signers.clone(),).into_val(&s.env);
        sign_as(&s.env, &s.s1, &s.client.address, "multi_auth_action", args);
        s.client.multi_auth_action(&signers);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn require_all_signers_rejects_missing_signer() {
        let s = setup();
        s.env.mock_auths(&[
            MockAuth {
                address: &s.s1,
                invoke: &MockAuthInvoke {
                    contract: &s.client.address,
                    fn_name: "require_all_signers",
                    args: ().into_val(&s.env),
                    sub_invokes: &[],
                },
            },
            MockAuth {
                address: &s.s2,
                invoke: &MockAuthInvoke {
                    contract: &s.client.address,
                    fn_name: "require_all_signers",
                    args: ().into_val(&s.env),
                    sub_invokes: &[],
                },
            },
        ]);
        s.client.require_all_signers();
    }
}

// ---------------------------------------------------------------------------
// Timelock: delay bypasses and admin takeover
// ---------------------------------------------------------------------------

mod timelock {
    use super::*;
    use ::timelock::{OperationState, TimelockContract, TimelockContractClient};
    use soroban_sdk::{testutils::Ledger as _, Bytes};

    const DELAY: u64 = 3_600;

    struct Setup<'a> {
        env: Env,
        client: TimelockContractClient<'a>,
        admin: Address,
    }

    fn setup<'a>() -> Setup<'a> {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1_000_000);
        let client = TimelockContractClient::new(&env, &env.register(TimelockContract, ()));
        let admin = Address::generate(&env);
        client.initialize(&admin);
        Setup { env, client, admin }
    }

    fn op(env: &Env, tag: &str) -> Bytes {
        Bytes::from_slice(env, tag.as_bytes())
    }

    fn advance(env: &Env, secs: u64) {
        env.ledger().with_mut(|l| l.timestamp += secs);
    }

    #[test]
    fn execute_before_delay_rejected_at_every_boundary() {
        let s = setup();
        let id = op(&s.env, "upgrade");
        s.client.queue(&id, &DELAY);

        assert!(s.client.try_execute(&id).is_err()); // immediately
        advance(&s.env, DELAY - 1);
        assert!(s.client.try_execute(&id).is_err()); // one second early
        assert_eq!(s.client.get_state(&id), OperationState::Pending);

        advance(&s.env, 1);
        s.client.execute(&id); // exactly at execute_at
    }

    #[test]
    fn delay_outside_bounds_rejected() {
        let s = setup();
        let (min, max) = s.client.get_delay_bounds();
        for delay in [0, min - 1, max + 1, u64::MAX] {
            assert!(s.client.try_queue(&op(&s.env, "x"), &delay).is_err());
        }
        assert_eq!(
            s.client.get_state(&op(&s.env, "x")),
            OperationState::Unknown
        );
    }

    /// Re-queuing a pending id must not overwrite its scheduled time.
    #[test]
    fn requeue_cannot_reset_schedule() {
        let s = setup();
        let (min, _) = s.client.get_delay_bounds();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        let eta = s.client.get_execute_at(&id);

        assert!(s.client.try_queue(&id, &min).is_err());
        assert_eq!(s.client.get_execute_at(&id), eta);
    }

    /// Shrinking the delay window must not accelerate operations already queued.
    #[test]
    fn lowering_delay_bounds_does_not_accelerate_queued_ops() {
        let s = setup();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        s.client.update_delay_bounds(&30, &60);

        advance(&s.env, 60);
        assert!(s.client.try_execute(&id).is_err());
        assert_eq!(s.client.get_state(&id), OperationState::Pending);
    }

    /// Admin cannot configure the timelock away (delay below the absolute floor).
    #[test]
    fn delay_bounds_cannot_disable_timelock() {
        let s = setup();
        for (min, max) in [(0, 60), (29, 60), (60, 604_801), (120, 60)] {
            assert!(s.client.try_update_delay_bounds(&min, &max).is_err());
        }
        assert_eq!(s.client.get_delay_bounds(), (60, 86_400));
    }

    #[test]
    fn executed_operation_cannot_be_replayed() {
        let s = setup();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        advance(&s.env, DELAY);
        s.client.execute(&id);

        assert_eq!(s.client.get_state(&id), OperationState::Unknown);
        assert!(s.client.try_execute(&id).is_err());
    }

    #[test]
    fn cancelled_or_unknown_operation_cannot_execute() {
        let s = setup();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        s.client.cancel(&id);
        advance(&s.env, DELAY);

        assert!(s.client.try_execute(&id).is_err());
        assert!(s.client.try_execute(&op(&s.env, "never-queued")).is_err());
    }

    #[test]
    fn pause_blocks_ready_operations_and_new_queues() {
        let s = setup();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        advance(&s.env, DELAY);
        s.client.set_pause(&true);

        assert!(s.client.try_execute(&id).is_err());
        assert!(s.client.try_queue(&op(&s.env, "new"), &DELAY).is_err());

        s.client.set_pause(&false);
        s.client.execute(&id);
    }

    #[test]
    fn reinitialize_cannot_replace_admin() {
        let s = setup();
        let attacker = Address::generate(&s.env);
        assert!(s.client.try_initialize(&attacker).is_err());
        assert_eq!(s.client.admin(), s.admin);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn previous_admin_loses_rights_after_handover() {
        let s = setup();
        let new_admin = Address::generate(&s.env);
        s.client.set_admin(&new_admin);

        let id = op(&s.env, "op");
        let args = (&id, DELAY).into_val(&s.env);
        sign_as(&s.env, &s.admin, &s.client.address, "queue", args);
        s.client.queue(&id, &DELAY);
    }

    /// Each admin-gated entrypoint, signed by an attacker instead of the admin.
    fn attacker_signs(s: &Setup, fn_name: &str, args: Vec<Val>) {
        let attacker = Address::generate(&s.env);
        sign_as(&s.env, &attacker, &s.client.address, fn_name, args);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_queue_rejected() {
        let s = setup();
        let id = op(&s.env, "op");
        attacker_signs(&s, "queue", (&id, DELAY).into_val(&s.env));
        s.client.queue(&id, &DELAY);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_execute_rejected() {
        let s = setup();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        advance(&s.env, DELAY);
        attacker_signs(&s, "execute", (&id,).into_val(&s.env));
        s.client.execute(&id);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_cancel_rejected() {
        let s = setup();
        let id = op(&s.env, "op");
        s.client.queue(&id, &DELAY);
        attacker_signs(&s, "cancel", (&id,).into_val(&s.env));
        s.client.cancel(&id);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_set_admin_rejected() {
        let s = setup();
        let attacker = Address::generate(&s.env);
        sign_as(
            &s.env,
            &attacker,
            &s.client.address,
            "set_admin",
            (&attacker,).into_val(&s.env),
        );
        s.client.set_admin(&attacker);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_delay_bounds_update_rejected() {
        let s = setup();
        attacker_signs(&s, "update_delay_bounds", (30u64, 30u64).into_val(&s.env));
        s.client.update_delay_bounds(&30, &30);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn spoofed_unpause_rejected() {
        let s = setup();
        s.client.set_pause(&true);
        attacker_signs(&s, "set_pause", (false,).into_val(&s.env));
        s.client.set_pause(&false);
    }
}

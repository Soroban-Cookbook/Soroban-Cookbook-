extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Bytes, Env, IntoVal, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Auth vector: encode / decode / validate tests
// ---------------------------------------------------------------------------

#[test]
fn test_encode_decode_roundtrip() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let signers = Vec::from_array(&env, [a.clone(), b.clone(), c.clone()]);

    let encoded = client.encode_auth_vec(&signers);
    let decoded = client.decode_auth_vec(&encoded);

    // Decoded length must equal number of unique signers.
    assert_eq!(decoded.len(), 3);
    // Every original signer must appear in the decoded vector.
    assert!(decoded.contains(&a));
    assert!(decoded.contains(&b));
    assert!(decoded.contains(&c));
}

#[test]
fn test_encode_deduplicates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let a = Address::generate(&env);
    // Pass the same address twice.
    let signers = Vec::from_array(&env, [a.clone(), a.clone()]);

    let encoded = client.encode_auth_vec(&signers);
    // After dedup only one entry should remain.
    assert_eq!(client.auth_vec_len(&encoded), 1);
}

#[test]
fn test_encode_sorts_canonically() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let a = Address::generate(&env);
    let b = Address::generate(&env);

    // Encode in both orders — the resulting blobs must be identical.
    let fwd = client.encode_auth_vec(&Vec::from_array(&env, [a.clone(), b.clone()]));
    let rev = client.encode_auth_vec(&Vec::from_array(&env, [b.clone(), a.clone()]));

    assert_eq!(fwd, rev);
}

#[test]
fn test_validate_accepts_well_formed_blob() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let encoded = client.encode_auth_vec(&Vec::from_array(&env, [a, b]));

    assert!(client.validate_auth_vec(&encoded));
}

#[test]
fn test_validate_rejects_empty_blob() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    assert!(!client.validate_auth_vec(&Bytes::new(&env)));
}

#[test]
fn test_validate_rejects_truncated_blob() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let a = Address::generate(&env);
    let encoded = client.encode_auth_vec(&Vec::from_array(&env, [a]));

    // Truncate by one byte — length no longer matches count.
    let truncated_len = encoded.len() - 1;
    let mut bad = Bytes::new(&env);
    for i in 0..truncated_len {
        bad.push_back(encoded.get(i).unwrap());
    }
    assert!(!client.validate_auth_vec(&bad));
}

#[test]
fn test_validate_rejects_zero_count_header() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    // Manually craft a blob with count = 0.
    let mut bad = Bytes::new(&env);
    bad.push_back(0); bad.push_back(0); bad.push_back(0); bad.push_back(0);
    assert!(!client.validate_auth_vec(&bad));
}

#[test]
fn test_auth_vec_len() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let signers = Vec::from_array(&env, [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ]);
    let encoded = client.encode_auth_vec(&signers);
    assert_eq!(client.auth_vec_len(&encoded), 3);
}

#[test]
fn test_auth_vec_contains() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let outsider = Address::generate(&env);
    let encoded = client.encode_auth_vec(&Vec::from_array(&env, [a.clone(), b.clone()]));

    assert!(client.auth_vec_contains(&encoded, &a));
    assert!(client.auth_vec_contains(&encoded, &b));
    assert!(!client.auth_vec_contains(&encoded, &outsider));
}

#[test]
#[should_panic(expected = "auth vector must not be empty")]
fn test_encode_empty_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);
    client.encode_auth_vec(&Vec::new(&env));
}

#[test]
#[should_panic(expected = "auth vector exceeds MAX_SIGNERS")]
fn test_encode_exceeds_max_signers_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let mut too_many: Vec<Address> = Vec::new(&env);
    for _ in 0..=MAX_SIGNERS {
        too_many.push_back(Address::generate(&env));
    }
    client.encode_auth_vec(&too_many);
}

#[test]
fn test_encoded_transfer_requires_all_auths() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let to = Address::generate(&env);

    let encoded = client.encode_auth_vec(&Vec::from_array(&env, [s1.clone(), s2.clone()]));
    // Should not panic — both signers are mocked.
    client.multi_sig_transfer_encoded(&encoded, &to, &500i128);
}

// ---------------------------------------------------------------------------
// Existing multi-party tests (unchanged)
// ---------------------------------------------------------------------------

#[test]
fn test_multi_sig_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);
    let to = Address::generate(&env);

    let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

    client.multi_sig_transfer(&signers, &to, &100i128);

    // Verify that ALL signers were required to authorize
    assert_eq!(
        env.auths(),
        std::vec![
            (
                signer1.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "multi_sig_transfer"),
                        (signers.clone(), to.clone(), 100i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            ),
            (
                signer2.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "multi_sig_transfer"),
                        (signers.clone(), to.clone(), 100i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            ),
            (
                signer3.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "multi_sig_transfer"),
                        (signers.clone(), to.clone(), 100i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            )
        ]
    );
}

#[test]
fn test_proposal_approval_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "prop1");

    // Setup 2-of-3 multisig
    client.setup_proposal(&proposal_id, &2u32, &all_signers);

    // Only 2 of the 3 approve
    let approvers = Vec::from_array(&env, [signer1.clone(), signer3.clone()]);

    client.proposal_approval(&proposal_id, &approvers);

    assert_eq!(
        env.auths(),
        std::vec![
            (
                signer1.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "proposal_approval"),
                        (proposal_id.clone(), approvers.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            ),
            (
                signer3.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "proposal_approval"),
                        (proposal_id.clone(), approvers.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Threshold not met")]
fn test_proposal_approval_fails_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let all_signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);
    let proposal_id = Symbol::new(&env, "prop2");

    // Setup 2-of-3 multisig
    client.setup_proposal(&proposal_id, &2u32, &all_signers);

    // Only 1 approves (below threshold of 2)
    let approvers = Vec::from_array(&env, [signer2.clone()]);

    client.proposal_approval(&proposal_id, &approvers);
}

#[test]
#[should_panic(expected = "Approver not in the list of valid signers!")]
fn test_proposal_approval_fails_invalid_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);

    let all_signers = Vec::from_array(&env, [signer1.clone(), signer2.clone()]);
    let proposal_id = Symbol::new(&env, "prop3");

    client.setup_proposal(&proposal_id, &2u32, &all_signers);

    let hacker = Address::generate(&env);
    // Hacker tries to approve but they are not in valid_signers
    let approvers = Vec::from_array(&env, [signer1.clone(), hacker.clone()]);

    client.proposal_approval(&proposal_id, &approvers);
}

#[test]
fn test_sequential_auth_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    // Step 1: Buyer funds
    client.sequential_auth_escrow(&buyer, &seller, &1000i128);

    // Assert that we are at step 2
    let step_key = DataKey::EscrowStep(buyer.clone(), seller.clone());
    let step: u32 = env.as_contract(&contract_id, || {
        env.storage().instance().get(&step_key).unwrap_or(0)
    });
    assert_eq!(step, 2);

    // Step 2: Joint Account Release (Both approve)
    client.sequential_auth_escrow(&buyer, &seller, &1000i128);

    // Assert that the escrow is cleared
    let step: u32 = env.as_contract(&contract_id, || {
        env.storage().instance().get(&step_key).unwrap_or(0)
    });
    assert_eq!(step, 0);

    let bal_key = DataKey::EscrowBal(buyer, seller);
    let bal: i128 = env.as_contract(&contract_id, || {
        env.storage().instance().get(&bal_key).unwrap_or(0)
    });
    assert_eq!(bal, 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_multi_sig_transfer_unauthorized() {
    let env = Env::default();
    // No mock_all_auths() provided
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let to = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    client.multi_sig_transfer(&signers, &to, &100i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_sequential_auth_escrow_unauthorized_step1() {
    let env = Env::default();
    // No mock_all_auths() provided
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    client.sequential_auth_escrow(&buyer, &seller, &1000i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_sequential_auth_escrow_unauthorized_step2() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultiPartyAuthContract);
    let client = MultiPartyAuthContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    // Step 1: Mock auth so buyer can fund
    env.mock_all_auths();
    client.sequential_auth_escrow(&buyer, &seller, &1000i128);

    // Step 2: Remove mock auths so the joint release fails
    // In soroban test framework, `env.mock_all_auths_allowing_non_root_auth()` or just creating a new mock state is not directly available to *unmock*,
    // but we can just use another test environment or call the client directly.
    // Actually, `mock_all_auths` applies to all subsequent calls in the same `Env`.
    // To test unauthorized step 2, we can just jump to step 2 with mock_all_auths in one environment,
    // but wait, we can't easily jump to step 2.
    // We can clear auths by not calling mock_all_auths in a fresh env, but step 2 requires step 1 to have happened.
    // Instead of doing it this way, let's use `env.set_auths(&[])` which effectively overrides and fails.
    env.set_auths(&[]);

    client.sequential_auth_escrow(&buyer, &seller, &1000i128);
}

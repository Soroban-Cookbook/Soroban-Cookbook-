#![cfg(test)]

extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol, Vec,
};

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

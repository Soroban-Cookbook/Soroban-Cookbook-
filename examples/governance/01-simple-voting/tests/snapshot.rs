use simple_voting::{VoteChoice, VotingContract, VotingContractClient};
use soroban_sdk::{testutils::Ledger, Address, Env, String};

const ADMIN: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const VOTER_FOR: &str = "GAAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQDZ7H";
const VOTER_AGAINST: &str = "GABAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEJXA";
const VOTER_ABSTAIN: &str = "GABQGAYDAMBQGAYDAMBQGAYDAMBQGAYDAMBQGAYDAMBQGAYDAMBQHGPC";

#[test]
fn snapshot_proposal_state_after_voting() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(100);

    let admin = Address::from_str(&env, ADMIN);
    let contract_id = env.register(VotingContract, ());
    let client = VotingContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.create_prop(&admin, &String::from_str(&env, "Fund development"), &500);

    client.cast_vote(&Address::from_str(&env, VOTER_FOR), &1, &VoteChoice::For);
    client.cast_vote(
        &Address::from_str(&env, VOTER_AGAINST),
        &1,
        &VoteChoice::Against,
    );
    client.cast_vote(
        &Address::from_str(&env, VOTER_ABSTAIN),
        &1,
        &VoteChoice::Abstain,
    );

    insta::assert_snapshot!(
        "proposal_after_voting",
        format!("{:#?}", client.get_prop(&1))
    );
}

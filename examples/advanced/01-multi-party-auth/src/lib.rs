#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contract]
pub struct MultiPartyAuthContract;

/// Example custom types for storage matching.
#[contracttype]
pub enum DataKey {
    // Escrow balance mapped to buyer and seller
    EscrowBal(Address, Address),
    // Step of the escrow process
    EscrowStep(Address, Address),
    // M-of-N parameters (total required)
    Threshold(Symbol),
    // Allowed signers for a specific proposal
    Signers(Symbol),
}

#[contractimpl]
impl MultiPartyAuthContract {
    /// Demonstrates 1-of-N or ALL must authorize.
    /// This function performs a multi-sig transfer that requires `ALL` listed `signers` to approve.
    ///
    /// # Security Considerations
    /// - All parties must authorize before state changes
    /// - Order of auth checks doesn't matter since they are collected and verified by the host environment.
    /// - Be careful with dynamic signer lists: anyone calling the contract could pass a list sizes,
    ///   so `signers` should typically be bounded or verified.
    ///
    /// # Gas cost
    /// Scales linearly with the number of authorizations since each signer verification has a cost.
    pub fn multi_sig_transfer(_env: Env, signers: Vec<Address>, _to: Address, _amount: i128) {
        // Require authorization from all signers
        for signer in signers.iter() {
            signer.require_auth();
        }

        // Proceed with multi-authorized action (e.g., token transfer)
        // TokenClient::new(&env, &token_id).transfer(&signers.get_unchecked(0), &to, &amount);
    }

    /// Demonstrates a Threshold authorization (M-of-N).
    /// Requires that at least `threshold` parties from a known group of `approvers`
    /// authorize this action.
    ///
    /// Real world use-case: DAO voting thresholds or multisig wallets
    pub fn proposal_approval(env: Env, proposal_id: Symbol, approvers: Vec<Address>) {
        // Load the required threshold and the legitimate signatories list
        let required_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold(proposal_id.clone()))
            .unwrap_or(2);
        let valid_signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers(proposal_id))
            .unwrap_or_else(|| {
                // Provide a default empty vector if not configured.
                // In a real app we'd likely panic if the proposal wasn't initialized.
                Vec::new(&env)
            });

        // Ensure we don't have duplicate approvals to cheat the threshold
        // By checking everyone and verifying they are in the valid_signers list.
        let mut valid_approval_count = 0;

        // For each passed approver
        for approver in approvers.iter() {
            // Must be a recognized signer
            if valid_signers.contains(&approver) {
                // Must have actually authorized the call
                approver.require_auth();
                valid_approval_count += 1;
            } else {
                panic!("Approver not in the list of valid signers!");
            }
        }

        // Check if M-of-N was met
        if valid_approval_count < required_threshold {
            panic!("Threshold not met");
        }

        // ... Execute proposal
    }

    /// Demonstrates an Escrow using Sequential logic.
    /// Step 1: Buyer funds the escrow
    /// Step 2: Buyer or Seller approves release
    ///
    /// Use Cases: Escrow services
    pub fn sequential_auth_escrow(env: Env, buyer: Address, seller: Address, amount: i128) {
        let step_key = DataKey::EscrowStep(buyer.clone(), seller.clone());
        let step: u32 = env.storage().instance().get(&step_key).unwrap_or(0);

        if step == 0 {
            // STEP 1: Buyer must authorize funding the escrow
            buyer.require_auth();

            // Simulate funding (e.g. transfer to contract)
            env.storage()
                .instance()
                .set(&DataKey::EscrowBal(buyer.clone(), seller.clone()), &amount);

            // Move to Step 2
            env.storage().instance().set(&step_key, &2u32);
        } else if step == 2 {
            // STEP 2: Wait for release
            // In an escrow, usually the buyer authorizes the release when happy,
            // or maybe the seller (or an admin arbiter) can trigger it.
            // In a joint context both might need to approve.

            // For a 2-of-2 Joint Account logic:
            buyer.require_auth();
            seller.require_auth();

            // Perform release (transfer from contract to seller)
            // Clear escrow
            env.storage().instance().set(&step_key, &0u32);
            env.storage()
                .instance()
                .set(&DataKey::EscrowBal(buyer, seller), &0i128);
        }
    }

    /// Helper for setting threshold and signers to easily test proposal approval
    pub fn setup_proposal(env: Env, proposal_id: Symbol, threshold: u32, signers: Vec<Address>) {
        env.storage()
            .instance()
            .set(&DataKey::Threshold(proposal_id.clone()), &threshold);
        env.storage()
            .instance()
            .set(&DataKey::Signers(proposal_id), &signers);
    }
}

mod test;

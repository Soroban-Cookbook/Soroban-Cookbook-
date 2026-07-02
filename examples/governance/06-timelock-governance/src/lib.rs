#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    ProposalNotFound = 3,
    ProposalNotQueued = 4,
    DelayNotMet = 5,
    ProposalAlreadyExecuted = 6,
    ProposalCanceled = 7,
    InvalidDelay = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Queued = 1,
    Executed = 2,
    Canceled = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<soroban_sdk::Val>,
    pub eta: u64,
    pub status: ProposalStatus,
}

#[contracttype]
pub enum DataKey {
    Admin,
    MinDelay,
    ProposalCount,
    Proposal(u64),
}

#[contract]
pub struct TimelockGovernance;

#[contractimpl]
impl TimelockGovernance {
    pub fn init(env: Env, admin: Address, min_delay: u64) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::MinDelay, &min_delay);
        env.storage().instance().set(&DataKey::ProposalCount, &0u64);
        
        Ok(())
    }

    pub fn queue(
        env: Env,
        target: Address,
        function: Symbol,
        args: Vec<soroban_sdk::Val>,
        delay: u64,
    ) -> Result<u64, Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let min_delay: u64 = env.storage().instance().get(&DataKey::MinDelay).unwrap();
        if delay < min_delay {
            return Err(Error::InvalidDelay);
        }

        let eta = env.ledger().timestamp() + delay;
        let mut count: u64 = env.storage().instance().get(&DataKey::ProposalCount).unwrap();
        count += 1;

        let proposal = Proposal {
            id: count,
            target,
            function,
            args,
            eta,
            status: ProposalStatus::Queued,
        };

        env.storage().persistent().set(&DataKey::Proposal(count), &proposal);
        env.storage().instance().set(&DataKey::ProposalCount, &count);

        // Emit event
        env.events().publish((Symbol::new(&env, "queued"), count), eta);

        Ok(count)
    }

    pub fn execute(env: Env, id: u64) -> Result<soroban_sdk::Val, Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Queued {
            return Err(Error::ProposalNotQueued);
        }

        if env.ledger().timestamp() < proposal.eta {
            return Err(Error::DelayNotMet);
        }

        proposal.status = ProposalStatus::Executed;
        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);

        let result = env.invoke_contract(&proposal.target, &proposal.function, proposal.args);

        // Emit event
        env.events().publish((Symbol::new(&env, "executed"), id), ());

        Ok(result)
    }

    pub fn cancel(env: Env, id: u64) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Queued {
            return Err(Error::ProposalNotQueued);
        }

        proposal.status = ProposalStatus::Canceled;
        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);

        // Emit event
        env.events().publish((Symbol::new(&env, "canceled"), id), ());

        Ok(())
    }

    pub fn emergency_execute(env: Env, id: u64) -> Result<soroban_sdk::Val, Error> {
        // Only admin can execute emergencies, bypassing delay
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Queued {
            return Err(Error::ProposalNotQueued);
        }

        // Bypass ETA check
        proposal.status = ProposalStatus::Executed;
        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);

        let result = env.invoke_contract(&proposal.target, &proposal.function, proposal.args);

        // Emit event
        env.events().publish((Symbol::new(&env, "emergency_executed"), id), ());

        Ok(result)
    }

    pub fn get_proposal(env: Env, id: u64) -> Result<Proposal, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(id))
            .ok_or(Error::ProposalNotFound)
    }
}

mod test;

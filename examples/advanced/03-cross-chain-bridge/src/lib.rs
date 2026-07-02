#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

#[contract]
pub struct CrossChainBridgeContract;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeTransfer {
    pub source_chain: Symbol,
    pub destination_chain: Symbol,
    pub sender: Address,
    pub recipient: Bytes,
    pub token: Address,
    pub amount: i128,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Validator(Address),
    ValidatorThreshold,
    LockedBalance(Address, Address),
    Nonce,
    ProcessedTransfer(Bytes),
    TokenMapping(Symbol, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeEventData {
    pub action: Symbol,
    pub transfer: BridgeTransfer,
    pub timestamp: u64,
}

const CONTRACT_NS: Symbol = symbol_short!("bridge");

// ---------------------------------------------------------------------------
// Contract Implementation
// ---------------------------------------------------------------------------

#[contractimpl]
impl CrossChainBridgeContract {
    /// Initialize the bridge contract with admin and initial validators
    pub fn initialize(env: Env, admin: Address, validators: Vec<Address>, threshold: u32) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ValidatorThreshold, &threshold);

        for validator in validators.iter() {
            env.storage().instance().set(&DataKey::Validator(validator.clone()), &true);
        }
    }

    /// Lock tokens on source chain to initiate a cross-chain transfer
    pub fn lock_tokens(
        env: Env,
        sender: Address,
        destination_chain: Symbol,
        recipient: Bytes,
        token: Address,
        amount: i128,
    ) -> BridgeTransfer {
        sender.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get next nonce
        let nonce: u64 = env.storage().instance().get(&DataKey::Nonce).unwrap_or(0);
        env.storage().instance().set(&DataKey::Nonce, &(nonce + 1));

        // Create transfer
        let transfer = BridgeTransfer {
            source_chain: symbol_short!("soroban"),
            destination_chain,
            sender: sender.clone(),
            recipient,
            token: token.clone(),
            amount,
            nonce,
        };

        // Lock tokens (transfer from sender to bridge)
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        // Update locked balance
        let balance_key = DataKey::LockedBalance(token.clone(), sender.clone());
        let current_balance: i128 = env.storage().instance().get(&balance_key).unwrap_or(0);
        env.storage().instance().set(&balance_key, &(current_balance + amount));

        // Emit event
        env.events().publish(
            (CONTRACT_NS, symbol_short!("lock")),
            BridgeEventData {
                action: symbol_short!("lock"),
                transfer: transfer.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        transfer
    }

    /// Mint tokens on destination chain after validator signatures
    pub fn mint_tokens(env: Env, transfer: BridgeTransfer, signatures: Vec<ValidatorSignature>) {
        // Verify transfer is not already processed
        let transfer_hash = env.crypto().sha256(&env.serialize_value(&transfer)).to_bytes();
        if env.storage().instance().has(&DataKey::ProcessedTransfer(transfer_hash.clone())) {
            panic!("Transfer already processed");
        }

        // Verify validator signatures
        Self::verify_validator_signatures(&env, &transfer, &signatures);

        // Get mapped token or use provided token
        let token = env
            .storage()
            .instance()
            .get(&DataKey::TokenMapping(transfer.source_chain, transfer.token.clone()))
            .unwrap_or(transfer.token.clone());

        // Mint tokens (or transfer from bridge's balance)
        let token_client = TokenClient::new(&env, &token);
        // In a real implementation, this would mint wrapped tokens or use bridge's reserve
        // For this example, we'll assume bridge has a reserve to transfer from
        let recipient_address = Address::from_string_bytes(&transfer.recipient);
        token_client.transfer(&env.current_contract_address(), &recipient_address, &transfer.amount);

        // Mark transfer as processed
        env.storage().instance().set(&DataKey::ProcessedTransfer(transfer_hash), &true);

        // Emit event
        env.events().publish(
            (CONTRACT_NS, symbol_short!("mint")),
            BridgeEventData {
                action: symbol_short!("mint"),
                transfer: transfer.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Burn wrapped tokens to initiate withdrawal back to source chain
    pub fn burn_tokens(
        env: Env,
        sender: Address,
        destination_chain: Symbol,
        recipient: Bytes,
        token: Address,
        amount: i128,
    ) -> BridgeTransfer {
        sender.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get next nonce
        let nonce: u64 = env.storage().instance().get(&DataKey::Nonce).unwrap_or(0);
        env.storage().instance().set(&DataKey::Nonce, &(nonce + 1));

        // Create transfer
        let transfer = BridgeTransfer {
            source_chain: symbol_short!("soroban"),
            destination_chain,
            sender: sender.clone(),
            recipient,
            token: token.clone(),
            amount,
            nonce,
        };

        // Burn tokens (transfer from sender to bridge and hold)
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        // Emit event
        env.events().publish(
            (CONTRACT_NS, symbol_short!("burn")),
            BridgeEventData {
                action: symbol_short!("burn"),
                transfer: transfer.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        transfer
    }

    /// Release locked tokens on source chain after validator signatures
    pub fn release_tokens(env: Env, transfer: BridgeTransfer, signatures: Vec<ValidatorSignature>) {
        // Verify transfer is not already processed
        let transfer_hash = env.crypto().sha256(&env.serialize_value(&transfer)).to_bytes();
        if env.storage().instance().has(&DataKey::ProcessedTransfer(transfer_hash.clone())) {
            panic!("Transfer already processed");
        }

        // Verify validator signatures
        Self::verify_validator_signatures(&env, &transfer, &signatures);

        // Get recipient
        let recipient_address = Address::from_string_bytes(&transfer.recipient);

        // Release tokens
        let token_client = TokenClient::new(&env, &transfer.token);
        token_client.transfer(&env.current_contract_address(), &recipient_address, &transfer.amount);

        // Update locked balance
        let balance_key = DataKey::LockedBalance(transfer.token.clone(), transfer.sender.clone());
        let current_balance: i128 = env.storage().instance().get(&balance_key).unwrap_or(0);
        env.storage().instance().set(&balance_key, &(current_balance - transfer.amount));

        // Mark transfer as processed
        env.storage().instance().set(&DataKey::ProcessedTransfer(transfer_hash), &true);

        // Emit event
        env.events().publish(
            (CONTRACT_NS, symbol_short!("release")),
            BridgeEventData {
                action: symbol_short!("release"),
                transfer: transfer.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Add a validator (admin only)
    pub fn add_validator(env: Env, admin: Address, validator: Address) {
        Self::require_admin(&env, &admin);
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Validator(validator.clone())) {
            panic!("Validator already exists");
        }

        env.storage().instance().set(&DataKey::Validator(validator), &true);
    }

    /// Remove a validator (admin only)
    pub fn remove_validator(env: Env, admin: Address, validator: Address) {
        Self::require_admin(&env, &admin);
        admin.require_auth();

        if !env.storage().instance().has(&DataKey::Validator(validator.clone())) {
            panic!("Validator not found");
        }

        // Check if removing would drop below threshold
        let threshold: u32 = env.storage().instance().get(&DataKey::ValidatorThreshold).unwrap_or(1);
        let current_count = Self::count_validators(&env);
        if current_count - 1 < threshold {
            panic!("Cannot remove: would drop below threshold");
        }

        env.storage().instance().remove(&DataKey::Validator(validator));
    }

    /// Update validator threshold (admin only)
    pub fn set_threshold(env: Env, admin: Address, threshold: u32) {
        Self::require_admin(&env, &admin);
        admin.require_auth();

        if threshold == 0 {
            panic!("Threshold must be at least 1");
        }

        let current_count = Self::count_validators(&env);
        if threshold > current_count {
            panic!("Threshold exceeds validator count");
        }

        env.storage().instance().set(&DataKey::ValidatorThreshold, &threshold);
    }

    /// Map a token from another chain to a Soroban token (admin only)
    pub fn map_token(env: Env, admin: Address, source_chain: Symbol, source_token: Address, soroban_token: Address) {
        Self::require_admin(&env, &admin);
        admin.require_auth();

        env.storage().instance().set(&DataKey::TokenMapping(source_chain, source_token), &soroban_token);
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Check if address is a validator
    pub fn is_validator(env: Env, address: Address) -> bool {
        env.storage().instance().get(&DataKey::Validator(address)).unwrap_or(false)
    }

    /// Get current validator threshold
    pub fn get_threshold(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::ValidatorThreshold).unwrap_or(1)
    }

    /// Get locked balance for a token and sender
    pub fn get_locked_balance(env: Env, token: Address, sender: Address) -> i128 {
        env.storage().instance().get(&DataKey::LockedBalance(token, sender)).unwrap_or(0)
    }

    /// Get nonce
    pub fn get_nonce(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::Nonce).unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal Helpers
    // -----------------------------------------------------------------------

    fn require_admin(env: &Env, address: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if address != &admin {
            panic!("Caller is not admin");
        }
    }

    fn count_validators(env: &Env) -> u32 {
        // In a real implementation, we'd track validators in a list
        // For this example, we'll return a default count
        3
    }

    fn verify_validator_signatures(env: &Env, transfer: &BridgeTransfer, signatures: &Vec<ValidatorSignature>) {
        let threshold: u32 = env.storage().instance().get(&DataKey::ValidatorThreshold).unwrap_or(1);
        let transfer_hash = env.crypto().sha256(&env.serialize_value(transfer)).to_bytes();

        let mut valid_signatures = 0;
        let mut seen_validators = Vec::new(env);

        for sig in signatures.iter() {
            // Check if validator is valid
            if !env.storage().instance().get(&DataKey::Validator(sig.validator.clone())).unwrap_or(false) {
                continue;
            }

            // Check for duplicate validators
            if seen_validators.contains(&sig.validator) {
                continue;
            }
            seen_validators.push_back(sig.validator.clone());

            // In a real implementation, we'd verify the signature here
            // For this example, we'll just count valid validators
            valid_signatures += 1;
        }

        if valid_signatures < threshold {
            panic!("Insufficient validator signatures");
        }
    }
}

// ---------------------------------------------------------------------------
// Token Client (simplified)
// ---------------------------------------------------------------------------

#[soroban_sdk::contractclient(name = "TokenClient")]
pub trait Token {
    fn transfer(e: Env, from: Address, to: Address, amount: i128);
}

#[cfg(test)]
mod test;

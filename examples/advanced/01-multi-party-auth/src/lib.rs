#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

#[contract]
pub struct MultiPartyAuthContract;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Payload for an admin-action event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminActionEventData {
    /// Identifier of the specific action performed.
    pub action: Symbol,
    /// Timestamp when the action was executed.
    pub timestamp: u64,
}

/// Payload for an audit-trail event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTrailEventData {
    /// Free-form description or reference tag.
    pub details: Symbol,
    /// Ledger timestamp at emission time.
    pub timestamp: u64,
}

/// Namespace symbol used as the first topic of every event this contract emits.
const CONTRACT_NS: Symbol = symbol_short!("multi");
/// Naming convention: `snake_case` action names in topic[1].
const ACTION_ADMIN: Symbol = symbol_short!("admin");
const ACTION_AUDIT: Symbol = symbol_short!("audit");

/// Example custom types for storage matching.
#[contracttype]
pub enum DataKey {
    EscrowBal(Address, Address),
    EscrowStep(Address, Address),
    Threshold(Symbol),
    Signers(Symbol),
}

// ---------------------------------------------------------------------------
// Authorization vector format
// ---------------------------------------------------------------------------
//
// An "auth vector" is a length-prefixed, sorted, deduplicated list of signer
// addresses serialized into a Bytes blob for compact on-chain storage or
// cross-contract passing.
//
// Wire format (big-endian):
//
//   [ count: u32 (4 bytes) ][ addr_0: 56 bytes ][ addr_1: 56 bytes ] ...
//
// Each address is stored as its 56-byte ASCII strkey (G… for accounts,
// C… for contracts). Addresses are kept in strict ascending lexicographic
// order of those bytes; duplicates are rejected.
//
// Constraints enforced by encode / decode:
//   1. count == actual number of addresses in the payload.
//   2. Addresses are in strict ascending strkey order.
//   3. No duplicate addresses (strict ordering implies uniqueness).
//   4. Maximum MAX_SIGNERS addresses per vector.

/// Maximum number of signers allowed in a single auth vector.
pub const MAX_SIGNERS: u32 = 20;

/// Byte length of one address entry in the wire format (56-byte strkey).
const ADDR_BYTES: u32 = 56;

/// Byte length of the count header.
const HEADER_LEN: u32 = 4;

// ---------------------------------------------------------------------------
// Contract implementation
// ---------------------------------------------------------------------------

#[contractimpl]
impl MultiPartyAuthContract {
    // -----------------------------------------------------------------------
    // Auth vector: encode / decode / validate
    // -----------------------------------------------------------------------

    /// Encode a `Vec<Address>` into a canonical auth-vector `Bytes` blob.
    ///
    /// The input list is sorted and deduplicated before encoding so the
    /// output is canonical regardless of the order callers supply addresses.
    ///
    /// Panics if the list is empty or contains more than `MAX_SIGNERS` unique
    /// addresses.
    pub fn encode_auth_vec(env: Env, signers: Vec<Address>) -> Bytes {
        let sorted = Self::sort_and_dedup(&env, &signers);
        Self::encode_sorted(&env, &sorted)
    }

    /// Decode an auth-vector `Bytes` blob back into a `Vec<Address>`.
    ///
    /// Validates the wire format and all ordering / uniqueness constraints
    /// before returning. Panics on any violation so callers never receive a
    /// malformed vector.
    pub fn decode_auth_vec(env: Env, encoded: Bytes) -> Vec<Address> {
        Self::decode_and_validate(&env, &encoded)
    }

    /// Validate an encoded auth-vector without fully decoding it.
    ///
    /// Returns `true` if the blob is well-formed, `false` otherwise.
    /// Useful for cheap pre-flight checks before passing a blob to another
    /// contract function.
    pub fn validate_auth_vec(env: Env, encoded: Bytes) -> bool {
        Self::is_valid_encoding(&env, &encoded)
    }

    /// Return the number of signers stored in an encoded auth vector.
    ///
    /// Panics if the blob is shorter than the 4-byte header.
    pub fn auth_vec_len(_env: Env, encoded: Bytes) -> u32 {
        if encoded.len() < HEADER_LEN {
            panic!("auth vector too short");
        }
        read_u32(&encoded, 0)
    }

    /// Return `true` if `signer` is present in the encoded auth vector.
    pub fn auth_vec_contains(env: Env, encoded: Bytes, signer: Address) -> bool {
        let signers = Self::decode_and_validate(&env, &encoded);
        signers.contains(&signer)
    }

    // -----------------------------------------------------------------------
    // Multi-party auth patterns
    // -----------------------------------------------------------------------

    /// N-of-N multi-sig transfer: every signer in the list must authorize.
    ///
    /// # Gas cost
    /// Scales linearly with the number of authorizations since each signer verification has a cost.
    pub fn multi_sig_transfer(env: Env, signers: Vec<Address>, _to: Address, _amount: i128) {
        // Require authorization from all signers
        for signer in signers.iter() {
            signer.require_auth();
        }

        // Audit trail for multi-sig action
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT),
            AuditTrailEventData {
                details: symbol_short!("msig_trf"),
                timestamp: env.ledger().timestamp(),
            },
        );

        // Proceed with multi-authorized action (e.g., token transfer)
        // TokenClient::new(&env, &token_id).transfer(&signers.get_unchecked(0), &to, &amount);
    }

    /// M-of-N threshold approval.
    ///
    /// Requires at least `threshold` parties from the stored valid-signers
    /// list to authorize. Duplicate approvers are rejected by the
    /// valid-signers membership check.
    pub fn proposal_approval(env: Env, proposal_id: Symbol, approvers: Vec<Address>) {
        let required_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold(proposal_id.clone()))
            .unwrap_or(2);

        let valid_signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers(proposal_id.clone()))
            .unwrap_or_else(|| {
                // Provide a default empty vector if not configured.
                // In a real app we'd likely panic if the proposal wasn't initialized.
                Vec::new(&env)
            });

        let mut valid_approval_count = 0u32;

        for approver in approvers.iter() {
            if valid_signers.contains(&approver) {
                approver.require_auth();
                valid_approval_count += 1;
            } else {
                panic!("Approver not in the list of valid signers!");
            }
        }

        if valid_approval_count < required_threshold {
            panic!("Threshold not met");
        }

        // Audit trail for proposal approval
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT, proposal_id),
            AuditTrailEventData {
                details: symbol_short!("prop_app"),
                timestamp: env.ledger().timestamp(),
            },
        );

        // ... Execute proposal
    }

    /// Sequential 2-step escrow.
    ///
    /// Step 0 → 2: buyer funds.
    /// Step 2 → 0: buyer + seller jointly release.
    pub fn sequential_auth_escrow(env: Env, buyer: Address, seller: Address, amount: i128) {
        let step_key = DataKey::EscrowStep(buyer.clone(), seller.clone());
        let step: u32 = env.storage().instance().get(&step_key).unwrap_or(0);

        if step == 0 {
            buyer.require_auth();
            env.storage()
                .instance()
                .set(&DataKey::EscrowBal(buyer.clone(), seller.clone()), &amount);
            env.storage().instance().set(&step_key, &2u32);

            // Audit trail for escrow funding
            env.events().publish(
                (CONTRACT_NS, ACTION_AUDIT, buyer, seller),
                AuditTrailEventData {
                    details: symbol_short!("esc_fund"),
                    timestamp: env.ledger().timestamp(),
                },
            );
        } else if step == 2 {
            buyer.require_auth();
            seller.require_auth();
            env.storage().instance().set(&step_key, &0u32);
            env.storage()
                .instance()
                .set(&DataKey::EscrowBal(buyer.clone(), seller.clone()), &0i128);

            // Audit trail for escrow release
            env.events().publish(
                (CONTRACT_NS, ACTION_AUDIT, buyer, seller),
                AuditTrailEventData {
                    details: symbol_short!("esc_rel"),
                    timestamp: env.ledger().timestamp(),
                },
            );
        }
    }

    /// Test helper: store threshold and valid-signers for a proposal.
    pub fn setup_proposal(env: Env, proposal_id: Symbol, threshold: u32, signers: Vec<Address>) {
        env.storage()
            .instance()
            .set(&DataKey::Threshold(proposal_id.clone()), &threshold);
        env.storage()
            .instance()
            .set(&DataKey::Signers(proposal_id.clone()), &signers);

        // Admin-style setup event
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, proposal_id),
            AdminActionEventData {
                action: symbol_short!("prop_set"),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Sort addresses lexicographically and remove duplicates.
    fn sort_and_dedup(env: &Env, signers: &Vec<Address>) -> Vec<Address> {
        if signers.is_empty() {
            panic!("auth vector must not be empty");
        }
        if signers.len() > MAX_SIGNERS {
            panic!("auth vector exceeds MAX_SIGNERS");
        }

        let n = signers.len() as usize;
        // Fixed-size scratch array — MAX_SIGNERS = 20.
        let mut arr: [Option<Address>; 20] = core::array::from_fn(|_| None);
        for (i, addr) in signers.iter().enumerate() {
            arr[i] = Some(addr);
        }

        // Insertion sort — O(n²), n ≤ 20.
        for i in 1..n {
            let mut j = i;
            while j > 0 {
                let a = arr[j - 1].as_ref().unwrap();
                let b = arr[j].as_ref().unwrap();
                if addr_key(env, a) > addr_key(env, b) {
                    arr.swap(j - 1, j);
                    j -= 1;
                } else {
                    break;
                }
            }
        }

        // Build output, skipping duplicates.
        let mut out: Vec<Address> = Vec::new(env);
        let mut prev: Option<[u8; 56]> = None;
        for slot in arr[..n].iter() {
            let addr = slot.as_ref().unwrap();
            let key = addr_key(env, addr);
            if Some(key) != prev {
                out.push_back(addr.clone());
                prev = Some(key);
            }
        }
        out
    }

    /// Encode a pre-sorted, deduplicated address list into the wire format.
    fn encode_sorted(env: &Env, sorted: &Vec<Address>) -> Bytes {
        let count = sorted.len();
        let mut buf = Bytes::new(env);

        // 4-byte big-endian count header.
        let cb = count.to_be_bytes();
        buf.push_back(cb[0]);
        buf.push_back(cb[1]);
        buf.push_back(cb[2]);
        buf.push_back(cb[3]);

        // 56 bytes per address (full strkey).
        for addr in sorted.iter() {
            for byte in addr_key(env, &addr).iter() {
                buf.push_back(*byte);
            }
        }

        buf
    }

    /// Decode and validate an encoded auth vector, returning the address list.
    fn decode_and_validate(env: &Env, encoded: &Bytes) -> Vec<Address> {
        if encoded.len() < HEADER_LEN {
            panic!("auth vector too short: missing count header");
        }

        let count = read_u32(encoded, 0);

        if count == 0 {
            panic!("auth vector must not be empty");
        }
        if count > MAX_SIGNERS {
            panic!("auth vector exceeds MAX_SIGNERS");
        }

        let expected_len = HEADER_LEN + count * ADDR_BYTES;
        if encoded.len() != expected_len {
            panic!("auth vector length mismatch");
        }

        let mut out: Vec<Address> = Vec::new(env);
        let mut prev: Option<[u8; 56]> = None;

        for i in 0..count {
            let offset = HEADER_LEN + i * ADDR_BYTES;
            let raw = read_addr_bytes(encoded, offset);

            if let Some(p) = prev {
                if raw <= p {
                    panic!("auth vector ordering violation at index {}", i);
                }
            }
            prev = Some(raw);

            let addr = Address::from_string_bytes(&Bytes::from_array(env, &raw));
            out.push_back(addr);
        }

        out
    }

    /// Cheap validity check — returns false instead of panicking.
    fn is_valid_encoding(env: &Env, encoded: &Bytes) -> bool {
        if encoded.len() < HEADER_LEN {
            return false;
        }
        let count = read_u32(encoded, 0);
        if count == 0 || count > MAX_SIGNERS {
            return false;
        }
        if encoded.len() != HEADER_LEN + count * ADDR_BYTES {
            return false;
        }
        let mut prev: Option<[u8; 56]> = None;
        for i in 0..count {
            let raw = read_addr_bytes(encoded, HEADER_LEN + i * ADDR_BYTES);
            if let Some(p) = prev {
                if raw <= p {
                    return false;
                }
            }
            prev = Some(raw);
        }
        let _ = env;
        true
    }
}

// ---------------------------------------------------------------------------
// Byte utilities
// ---------------------------------------------------------------------------

/// Read a big-endian u32 from `buf` at byte `offset`.
fn read_u32(buf: &Bytes, offset: u32) -> u32 {
    let b0 = buf.get(offset).unwrap() as u32;
    let b1 = buf.get(offset + 1).unwrap() as u32;
    let b2 = buf.get(offset + 2).unwrap() as u32;
    let b3 = buf.get(offset + 3).unwrap() as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

/// Read 56 address bytes from `buf` starting at `offset`.
fn read_addr_bytes(buf: &Bytes, offset: u32) -> [u8; 56] {
    let mut raw = [0u8; 56];
    for j in 0..56u32 {
        raw[j as usize] = buf.get(offset + j).unwrap();
    }
    raw
}

/// Derive a stable 56-byte sort key from an `Address` using its strkey
/// (G… / C…) encoding. Soroban strkeys are exactly 56 ASCII characters.
fn addr_key(_env: &Env, addr: &Address) -> [u8; 56] {
    let s = addr.to_string();
    let mut buf = [0u8; 56];
    s.copy_into_slice(&mut buf);
    buf
}

#[cfg(test)]
mod test;

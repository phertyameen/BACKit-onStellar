use soroban_sdk::{contracttype, Address, Env, String};

// TTL Constants
//
// Stellar produces ~1 ledger every 5 seconds.
//   Ledgers per day  = 86_400 / 5 = 17_280
//   Ledgers per year = 17_280 * 365 = 6_307_200
//
// The old value of 31_536_000 was actually the number of *seconds* in a year,
// not ledgers — a ~5× overcount.  Soroban caps persistent-entry TTL at
// MAX_ENTRY_TTL (currently 3_110_400 ledgers ≈ 180 days on Mainnet/Testnet),
// so we must stay at or below that ceiling.

/// Threshold below which we proactively bump a persistent entry's TTL.
/// ~30 days worth of ledgers  (17_280 * 30)
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 518_400;

/// How far into the future we extend a persistent entry's TTL when bumping.
/// ~90 days worth of ledgers  (17_280 * 90)
pub const PERSISTENT_BUMP_AMOUNT: u32 = 1_555_200;

/// Instance-storage TTL bump — how long to keep the contract instance live.
/// ~30 days  (same as threshold so the instance is always kept ≥ BUMP ahead)
pub const INSTANCE_BUMP_AMOUNT: u32 = 518_400;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Individual call entry keyed by call_id
    Call(String),
    /// List of call IDs belonging to a staker
    StakerCalls(Address),
}

#[contracttype]
#[derive(Clone)]
pub struct CallRecord {
    pub call_id: String,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub deadline: u64,
    pub created_at: u64,
    pub is_resolved: bool,
    pub outcome: bool,
}

/// Extend the contract *instance* storage TTL.
/// Uses the corrected ledger-based constant (not seconds).
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Write a call record and immediately extend its persistent TTL.
pub fn set_call(env: &Env, call_id: &String, record: &CallRecord) {
    let key = DataKey::Call(call_id.clone());
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

/// Read a call record (returns None if not found / archived).
pub fn get_call(env: &Env, call_id: &String) -> Option<CallRecord> {
    let key = DataKey::Call(call_id.clone());
    env.storage().persistent().get(&key)
}

/// Bump the TTL of an existing call entry without mutating it.
/// Returns false if the entry doesn't exist.
pub fn extend_call_ttl(env: &Env, call_id: &String) -> bool {
    let key = DataKey::Call(call_id.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        true
    } else {
        false
    }
}

/// Read the list of call IDs for a staker, returning an empty vec if absent.
pub fn get_staker_calls(env: &Env, staker: &Address) -> soroban_sdk::Vec<String> {
    let key = DataKey::StakerCalls(staker.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| soroban_sdk::Vec::new(env))
}

/// Overwrite the staker's call list and extend its TTL.
pub fn set_staker_calls(env: &Env, staker: &Address, calls: &soroban_sdk::Vec<String>) {
    let key = DataKey::StakerCalls(staker.clone());
    env.storage().persistent().set(&key, calls);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

/// Append a call_id to a staker's list, bumping TTL in the same operation.
pub fn add_call_to_staker(env: &Env, staker: &Address, call_id: &String) {
    let mut calls = get_staker_calls(env, staker);
    calls.push_back(call_id.clone());
    set_staker_calls(env, staker, &calls);
}
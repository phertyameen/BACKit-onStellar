use crate::types::{Call, ContractConfig};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Config,
    CallCounter,
    Call(u64),
    StakerCalls(Address),
}

pub fn set_config(env: &Env, config: &ContractConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<ContractConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn next_call_id(env: &Env) -> u64 {
    let counter: u64 = env.storage().instance().get(&DataKey::CallCounter).unwrap_or(0);
    let next_id = counter + 1;
    env.storage().instance().set(&DataKey::CallCounter, &next_id);
    next_id
}

/// Store a call in persistent storage (migrated from instance storage — #147)
pub fn set_call(env: &Env, call: &Call) {
    env.storage().persistent().set(&DataKey::Call(call.id), call);
    env.storage().persistent().extend_ttl(&DataKey::Call(call.id), 17_280, 17_280);
}

/// Retrieve a call by ID from persistent storage (#147)
pub fn get_call(env: &Env, call_id: u64) -> Option<Call> {
    env.storage().persistent().get(&DataKey::Call(call_id))
}

pub fn call_exists(env: &Env, call_id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Call(call_id))
}

pub fn add_staker_call(env: &Env, staker: &Address, call_id: u64) {
    let key = DataKey::StakerCalls(staker.clone());
    let mut call_ids: soroban_sdk::Vec<u64> = env.storage().instance().get(&key).unwrap_or_else(|| soroban_sdk::Vec::new(env));
    if !call_ids.iter().any(|id| id == call_id) {
        call_ids.push_back(call_id);
        env.storage().instance().set(&key, &call_ids);
    }
}

pub fn get_staker_calls(env: &Env, staker: &Address) -> soroban_sdk::Vec<u64> {
    env.storage().instance().get(&DataKey::StakerCalls(staker.clone())).unwrap_or_else(|| soroban_sdk::Vec::new(env))
}

pub fn get_call_counter(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::CallCounter).unwrap_or(0)
}

pub fn extend_storage_ttl(env: &Env) {
    env.storage().instance().extend_ttl(31_536_000, 31_536_000);
}
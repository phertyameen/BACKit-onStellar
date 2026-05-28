#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events as _, Ledger as _, LedgerInfo},
    Vec, Address, Env, IntoVal, Symbol, Bytes, String as SorobanString
};

/// Helper: build a default LedgerInfo for tests
fn ledger_info(sequence: u32) -> LedgerInfo {
    LedgerInfo {
        timestamp: 1_700_000_000,
        protocol_version: 21,
        sequence_number: sequence,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 6_312_000, // ~1 year in ledgers
    }
}
 
/// Convenience: register the contract and return (env, client)
fn setup() -> (Env, CallRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(ledger_info(100));
 
    let contract_id = env.register_contract(None, CallRegistry);
    let client = CallRegistryClient::new(&env, &contract_id);
    (env, client)
}
 
fn mk_str(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
}

mod call_registry {
    use super::*;
    use crate::types::ConditionType;
    use crate::{CallRegistry, CallRegistryClient};
    use crate::storage::DataKey;

    fn setup() -> (Env, CallRegistryClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
 
    let contract_id = env.register_contract(None, CallRegistry);
    let client = CallRegistryClient::new(&env, &contract_id);
 
    let admin = Address::generate(&env);
    let outcome_manager = Address::generate(&env);
 
    client.initialize(&admin, &outcome_manager);
 
    (env, client, admin, outcome_manager)
}
 
// ── set_admin ─────────────────────────────────────────────────────────────────
 
#[test]
fn test_set_admin_updates_config() {
    let (env, client, _admin, _om) = setup();
    let new_admin = Address::generate(&env);
 
    client.set_admin(&new_admin);
 
    assert_eq!(client.get_config().admin, new_admin);
}
 
#[test]
fn test_set_admin_emits_admin_params_changed() {
    let (env, client, old_admin, _om) = setup();
    let new_admin = Address::generate(&env);
 
    client.set_admin(&new_admin);
 
    let events = env.events().all();
    let last = events.last().expect("no events");
 
    // Topic: ("call_registry", "admin_params_changed")
    assert_eq!(
        last.1,
        soroban_sdk::vec![
            &env,
            "call_registry".into_val(&env),
            "admin_params_changed".into_val(&env),
        ]
    );
 
    // First element of the payload tuple is the param discriminant
    let (param, _changed_by, old_val, new_val): (Symbol, Address, Address, Address) =
        last.2.into_val(&env);
 
    assert_eq!(param, Symbol::new(&env, "admin"));
    assert_eq!(old_val, old_admin);
    assert_eq!(new_val, new_admin);
}
 
// ── set_outcome_manager ───────────────────────────────────────────────────────
 
#[test]
fn test_set_outcome_manager_updates_config() {
    let (env, client, _admin, _om) = setup();
    let new_om = Address::generate(&env);
 
    client.set_outcome_manager(&new_om);
 
    assert_eq!(client.get_config().outcome_manager, new_om);
}
 
#[test]
fn test_set_outcome_manager_emits_admin_params_changed() {
    let (env, client, _admin, old_om) = setup();
    let new_om = Address::generate(&env);
 
    client.set_outcome_manager(&new_om);
 
    let events = env.events().all();
    let last = events.last().expect("no events");
 
    let (param, _changed_by, old_val, new_val): (Symbol, Address, Address, Address) =
        last.2.into_val(&env);
 
    assert_eq!(param, Symbol::new(&env, "outcome_manager"));
    assert_eq!(old_val, old_om);
    assert_eq!(new_val, new_om);
}
 
// ── set_fee ───────────────────────────────────────────────────────────────────
 
#[test]
fn test_set_fee_updates_config() {
    let (_env, client, _admin, _om) = setup();
 
    client.set_fee(&250_u32); // 2.5 %
 
    assert_eq!(client.get_config().fee_bps, 250);
}
 
#[test]
fn test_set_fee_emits_admin_params_changed() {
    let (env, client, _admin, _om) = setup();
 
    client.set_fee(&100_u32);
 
    let events = env.events().all();
    let last = events.last().expect("no events");
 
    let (param, _changed_by, old_val, new_val): (Symbol, Address, u32, u32) =
        last.2.into_val(&env);
 
    assert_eq!(param, Symbol::new(&env, "fee_bps"));
    assert_eq!(old_val, 0_u32);   // default set in initialize()
    assert_eq!(new_val, 100_u32);
}
 
#[test]
fn test_set_fee_zero_is_valid() {
    let (_env, client, _admin, _om) = setup();
    client.set_fee(&0_u32);
    assert_eq!(client.get_config().fee_bps, 0);
}
 
#[test]
fn test_set_fee_max_boundary_is_valid() {
    let (_env, client, _admin, _om) = setup();
    client.set_fee(&10_000_u32); // exactly 100 % — allowed
    assert_eq!(client.get_config().fee_bps, 10_000);
}

#[test]
fn test_ttl_constants_are_ledger_based() {
    use crate::storage::{INSTANCE_BUMP_AMOUNT, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
 
    // Ledgers per day ≈ 17_280  (86_400s / 5s per ledger)
    let ledgers_per_day: u32 = 17_280;
 
    // Threshold should be at least 28 days
    assert!(
        PERSISTENT_LIFETIME_THRESHOLD >= ledgers_per_day * 28,
        "PERSISTENT_LIFETIME_THRESHOLD should be at least 28 days ({} ledgers)",
        ledgers_per_day * 28
    );
 
    // Bump should be greater than threshold
    assert!(
        PERSISTENT_BUMP_AMOUNT > PERSISTENT_LIFETIME_THRESHOLD,
        "PERSISTENT_BUMP_AMOUNT must exceed PERSISTENT_LIFETIME_THRESHOLD"
    );
 
    // Neither constant should be anywhere near 31_536_000 (the old incorrect value)
    assert!(
        PERSISTENT_BUMP_AMOUNT < 10_000_000,
        "PERSISTENT_BUMP_AMOUNT looks like it is still in seconds, not ledgers"
    );
    assert!(
        INSTANCE_BUMP_AMOUNT < 10_000_000,
        "INSTANCE_BUMP_AMOUNT looks like it is still in seconds, not ledgers"
    );
}
 
// ─────────────────────────────────────────────────────────────────────────────
// set_call: persistent TTL is bumped on write
// ─────────────────────────────────────────────────────────────────────────────
 
#[test]
fn test_set_call_extends_persistent_ttl() {
    let (env, client) = setup();
    let creator = Address::generate(&env);
    let call_id = mk_str(&env, "call_001");
 
    client.set_call(
        &call_id,
        &creator,
        &mk_str(&env, "BTC hits 100k"),
        &mk_str(&env, "Prediction: BTC > 100k by year end"),
        &(env.ledger().timestamp() + 86_400),
    );
 
    // The persistent entry for this call must have a TTL ≥ PERSISTENT_BUMP_AMOUNT
    // (measured from the current ledger sequence).
    env.as_contract(&client.address, || {
        use crate::storage::{DataKey, PERSISTENT_BUMP_AMOUNT};
        let key = DataKey::Call(mk_str(&env, "call_001"));
        let live_until = env.storage().persistent().get_ttl(&key);
        let current_seq = env.ledger().sequence();
        assert!(
            live_until >= current_seq + PERSISTENT_BUMP_AMOUNT - 1,
            "Call TTL not extended after set_call: live_until={live_until}, expected >= {}",
            current_seq + PERSISTENT_BUMP_AMOUNT - 1
        );
    });
}
 
// ─────────────────────────────────────────────────────────────────────────────
// StakerCalls: persistent TTL is bumped when the list is written
// ─────────────────────────────────────────────────────────────────────────────
 
#[test]
fn test_set_call_extends_staker_calls_ttl() {
    let (env, client) = setup();
    let creator = Address::generate(&env);
    let call_id = mk_str(&env, "call_002");
 
    client.set_call(
        &call_id,
        &creator,
        &mk_str(&env, "ETH flippening"),
        &mk_str(&env, "ETH market cap surpasses BTC"),
        &(env.ledger().timestamp() + 86_400),
    );
 
    env.as_contract(&client.address, || {
        use crate::storage::{DataKey, PERSISTENT_BUMP_AMOUNT};
        let key = DataKey::StakerCalls(creator.clone());
        let live_until = env.storage().persistent().get_ttl(&key);
        let current_seq = env.ledger().sequence();
        assert!(
            live_until >= current_seq + PERSISTENT_BUMP_AMOUNT - 1,
            "StakerCalls TTL not extended after set_call"
        );
    });
}
 
// ─────────────────────────────────────────────────────────────────────────────
// extend_call_ttl: anyone can bump a call's TTL
// ─────────────────────────────────────────────────────────────────────────────
 
#[test]
fn test_extend_call_ttl_bumps_ttl() {
    let (env, client) = setup();
    let creator = Address::generate(&env);
    let call_id = mk_str(&env, "call_003");
 
    // Create the call at ledger 100
    client.set_call(
        &call_id,
        &creator,
        &mk_str(&env, "SOL hits $1000"),
        &mk_str(&env, "Solana reaches 4 figures"),
        &(env.ledger().timestamp() + 172_800),
    );
 
    // Advance ledger sequence to simulate time passing
    env.ledger().set(ledger_info(200_000));
 
    // A different account calls extend_call_ttl (no auth needed)
    let result = client.extend_call_ttl(&call_id);
    assert!(result, "extend_call_ttl should return true for existing call");
 
    env.as_contract(&client.address, || {
        use crate::storage::{DataKey, PERSISTENT_BUMP_AMOUNT};
        let key = DataKey::Call(mk_str(&env, "call_003"));
        let live_until = env.storage().persistent().get_ttl(&key);
        let current_seq = env.ledger().sequence(); // 200_000
        assert!(
            live_until >= current_seq + PERSISTENT_BUMP_AMOUNT - 1,
            "extend_call_ttl did not refresh TTL correctly"
        );
    });
}
 
#[test]
fn test_extend_call_ttl_returns_false_for_missing_call() {
    let (env, client) = setup();
    let nonexistent = mk_str(&env, "does_not_exist");
    let result = client.extend_call_ttl(&nonexistent);
    assert!(!result, "extend_call_ttl should return false for non-existent call");
}
 
// ─────────────────────────────────────────────────────────────────────────────
// resolve_call also re-extends the call's TTL
// ─────────────────────────────────────────────────────────────────────────────
 
#[test]
fn test_resolve_call_extends_ttl() {
    let (env, client) = setup();
    let creator = Address::generate(&env);
    let call_id = mk_str(&env, "call_004");
 
    client.set_call(
        &call_id,
        &creator,
        &mk_str(&env, "DOGE moon"),
        &mk_str(&env, "DOGE reaches $1"),
        &(env.ledger().timestamp() + 86_400),
    );
 
    // Advance time slightly
    env.ledger().set(ledger_info(5_000));
 
    client.resolve_call(&call_id, &creator, &true);
 
    env.as_contract(&client.address, || {
        use crate::storage::{DataKey, PERSISTENT_BUMP_AMOUNT};
        let key = DataKey::Call(mk_str(&env, "call_004"));
        let live_until = env.storage().persistent().get_ttl(&key);
        let current_seq = env.ledger().sequence();
        assert!(
            live_until >= current_seq + PERSISTENT_BUMP_AMOUNT - 1,
            "resolve_call did not extend TTL"
        );
    });
}
 
// ─────────────────────────────────────────────────────────────────────────────
// get_staker_calls: staker list is correct and TTL was bumped
// ─────────────────────────────────────────────────────────────────────────────
 
#[test]
fn test_staker_calls_accumulate_and_ttl_bumped() {
    let (env, client) = setup();
    let creator = Address::generate(&env);
 
    for i in 0u32..3 {
        let call_id = String::from_str(&env, &soroban_sdk::format!("call_{i}"));
        client.set_call(
            &call_id,
            &creator,
            &mk_str(&env, "title"),
            &mk_str(&env, "desc"),
            &(env.ledger().timestamp() + 86_400),
        );
    }
 
    let calls = client.get_staker_calls(&creator);
    assert_eq!(calls.len(), 3, "Expected 3 calls in staker list");
 
    env.as_contract(&client.address, || {
        use crate::storage::{DataKey, PERSISTENT_BUMP_AMOUNT};
        let key = DataKey::StakerCalls(creator.clone());
        let live_until = env.storage().persistent().get_ttl(&key);
        let current_seq = env.ledger().sequence();
        assert!(
            live_until >= current_seq + PERSISTENT_BUMP_AMOUNT - 1,
            "StakerCalls TTL not bumped after third entry"
        );
    });
}
 
#[test]
#[should_panic(expected = "fee_bps cannot exceed 10_000 (100%)")]
fn test_set_fee_above_max_panics() {
    let (_env, client, _admin, _om) = setup();
    client.set_fee(&10_001_u32);
}

    fn create_test_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let outcome_manager = Address::generate(&env);
        let creator = Address::generate(&env);

        (env, admin, outcome_manager, creator)
    }

    fn create_call_with_default_condition(
        client: &CallRegistryClient<'_>,
        creator: &Address,
        stake_token: &Address,
        stake_amount: &i128,
        end_ts: &u64,
        token_address: &Address,
        pair_id: &Bytes,
        ipfs_cid: &Bytes,
    ) -> crate::types::Call {
        client.create_call(
            creator,
            stake_token,
            stake_amount,
            end_ts,
            token_address,
            pair_id,
            ipfs_cid,
            &ConditionType::TargetAbove(100_000_000_i128),
        )
    }

    #[test]
    fn test_initialize() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        // Initialize contract
        client.initialize(&admin, &outcome_manager);

        // Get config and verify
        let config = client.get_config();
        assert_eq!(config.admin, admin);
        assert_eq!(config.outcome_manager, outcome_manager);
    }

    #[test]
    #[should_panic(expected = "Contract already initialized")]
    fn test_initialize_twice_fails() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        client.initialize(&admin, &outcome_manager); // Should panic
    }

    #[test]
    fn test_create_call_success() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        // Setup
        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        // Create call
        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Verify
        assert_eq!(call.id, 1);
        assert_eq!(call.creator, creator);
        assert_eq!(call.stake_amount, 100_000_000);
        assert_eq!(call.total_up_stake, 0);
        assert_eq!(call.total_down_stake, 0);
        assert_eq!(call.outcome, 0);
        assert_eq!(call.condition, ConditionType::TargetAbove(100_000_000_i128));
        assert_eq!(call.settled, false);
        assert_eq!(call.created_at, 1000);
    }

    #[test]
    #[should_panic(expected = "Stake amount must be positive")]
    fn test_create_call_invalid_stake() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &-100_000_000_i128, // Invalid
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
    }

    #[test]
    #[should_panic(expected = "End timestamp must be in the future")]
    fn test_create_call_past_timestamp() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &500u64, // In the past
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
    }

    #[test]
    fn test_stake_on_call_up() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        // Setup
        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Mock token transfer for test
        env.budget().reset_unlimited();

        // Stake UP
        let updated_call = client.stake_on_call(&staker, &call.id, &50_000_000_i128, &1);

        // Verify
        assert_eq!(updated_call.total_up_stake, 50_000_000);
        assert_eq!(updated_call.total_down_stake, 0);
    }

    #[test]
    fn test_stake_on_call_down() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        // Setup
        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        // Stake DOWN
        let updated_call = client.stake_on_call(&staker, &call.id, &30_000_000_i128, &2);

        // Verify
        assert_eq!(updated_call.total_up_stake, 0);
        assert_eq!(updated_call.total_down_stake, 30_000_000);
    }

    #[test]
    #[should_panic(expected = "Call has ended")]
    fn test_stake_on_ended_call() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        // Setup
        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Move time past end
        env.ledger().set_timestamp(3000);

        // Try to stake - should fail
        client.stake_on_call(&staker, &call.id, &50_000_000_i128, &1);
    }

    #[test]
    #[should_panic(expected = "Invalid position")]
    fn test_stake_invalid_position() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Invalid position (3)
        client.stake_on_call(&staker, &call.id, &50_000_000_i128, &3);
    }

    #[test]
    fn test_get_call() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let created_call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Retrieve call
        let retrieved_call = client.get_call(&created_call.id);

        // Verify
        assert_eq!(retrieved_call.id, created_call.id);
        assert_eq!(retrieved_call.creator, creator);
        assert_eq!(retrieved_call.stake_amount, 100_000_000);
    }

    #[test]
    #[should_panic(expected = "Call does not exist")]
    fn test_get_nonexistent_call() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);

        // Try to get non-existent call
        client.get_call(&999);
    }

    #[test]
    fn test_get_call_stats() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker1 = Address::generate(&env);
        let staker2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        // Add stakes
        client.stake_on_call(&staker1, &call.id, &50_000_000_i128, &1);
        client.stake_on_call(&staker2, &call.id, &30_000_000_i128, &2);

        // Get stats
        let stats = client.get_call_stats(&call.id);

        // Verify
        assert_eq!(stats.total_up_stake, 50_000_000);
        assert_eq!(stats.total_down_stake, 30_000_000);
        assert_eq!(stats.up_stake_count, 1);
        assert_eq!(stats.down_stake_count, 1);
    }

    #[test]
    fn test_resolve_call() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Move time past end
        env.ledger().set_timestamp(3000);

        // Resolve call
        let resolved = client.resolve_call(&call.id, &1, &150_000_000_i128);

        // Verify
        assert_eq!(resolved.outcome, 1);
        assert_eq!(resolved.end_price, 150_000_000);
    }

    #[test]
    #[should_panic(expected = "Call has not ended yet")]
    fn test_resolve_call_before_end() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // Try to resolve before end time
        client.resolve_call(&call.id, &1, &150_000_000_i128);
    }

    #[test]
    fn test_set_admin() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let new_admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);

        // Change admin
        client.set_admin(&new_admin);

        // Verify
        let config = client.get_config();
        assert_eq!(config.admin, new_admin);
    }

    #[test]
    fn test_set_outcome_manager() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let new_manager = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);

        // Change outcome manager
        client.set_outcome_manager(&new_manager);

        // Verify
        let config = client.get_config();
        assert_eq!(config.outcome_manager, new_manager);
    }

    #[test]
    fn test_get_call_count() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        assert_eq!(client.get_call_count(), 0);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        // Create multiple calls
        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &3000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        assert_eq!(client.get_call_count(), 2);
    }

    #[test]
    fn test_get_calls_paginated_respects_limit_and_start_id() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &3000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
        create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &4000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        let results = client.get_calls_paginated(&2u64, &2u32);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().id, 2);
        assert_eq!(results.get(1).unwrap().id, 3);
    }

    #[test]
    fn test_get_calls_by_creator_paginated_handles_gaps_and_max_limit() {
        let (env, admin, outcome_manager, creator1) = create_test_env();
        let creator2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        create_call_with_default_condition(&client, 
            &creator1,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
        create_call_with_default_condition(&client, 
            &creator2,
            &stake_token,
            &100_000_000_i128,
            &3000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::CallCounter, &4u64);
        });

        let last_call = create_call_with_default_condition(&client, 
            &creator1,
            &stake_token,
            &100_000_000_i128,
            &4000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        let results = client.get_calls_by_creator_paginated(&creator1, &1u64, &100u32);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().id, 1);
        assert_eq!(results.get(1).unwrap().id, last_call.id);
        assert_eq!(results.get(1).unwrap().id, 5);
        assert!(results.len() <= 20);
    }

    #[test]
    fn test_get_calls_paginated_respects_maximum_limit() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        for _ in 0..25 {
            create_call_with_default_condition(&client, 
                &creator,
                &stake_token,
                &100_000_000_i128,
                &2000u64,
                &token_address,
                &pair_id,
                &ipfs_cid,
            );
        }

        let results = client.get_calls_paginated(&1u64, &100u32);
        assert_eq!(results.len(), 20);
        assert_eq!(results.get(0).unwrap().id, 1);
        assert_eq!(results.get(19).unwrap().id, 20);
    }

    #[test]
    fn test_get_calls_by_creator_paginated_returns_creator_specific_results() {
        let (env, admin, outcome_manager, creator1) = create_test_env();
        let creator2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        create_call_with_default_condition(&client, 
            &creator1,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
        create_call_with_default_condition(&client, 
            &creator2,
            &stake_token,
            &100_000_000_i128,
            &3000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
        create_call_with_default_condition(&client, 
            &creator1,
            &stake_token,
            &100_000_000_i128,
            &4000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        let results = client.get_calls_by_creator_paginated(&creator1, &1u64, &10u32);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().creator, creator1);
        assert_eq!(results.get(1).unwrap().creator, creator1);
        assert_eq!(results.get(0).unwrap().id, 1);
        assert_eq!(results.get(1).unwrap().id, 3);
    }

    #[test]
    fn test_get_staker_stake() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        // Add stake
        client.stake_on_call(&staker, &call.id, &50_000_000_i128, &1);

        // Get stake
        let up_stake = client.get_staker_stake(&call.id, &staker, &1);
        let down_stake = client.get_staker_stake(&call.id, &staker, &2);

        assert_eq!(up_stake, 50_000_000);
        assert_eq!(down_stake, 0);
    }

    #[test]
    fn test_multiple_stakers() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker1 = Address::generate(&env);
        let staker2 = Address::generate(&env);
        let staker3 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = create_call_with_default_condition(&client, 
            &creator,
            &stake_token,
            &100_000_000_i128,
            &5000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        // Multiple stakers
        client.stake_on_call(&staker1, &call.id, &50_000_000_i128, &1);
        client.stake_on_call(&staker2, &call.id, &30_000_000_i128, &1);
        client.stake_on_call(&staker3, &call.id, &40_000_000_i128, &2);

        let call_updated = client.get_call(&call.id);

        // Verify totals
        assert_eq!(call_updated.total_up_stake, 80_000_000);
        assert_eq!(call_updated.total_down_stake, 40_000_000);
    }

    #[test]
    fn test_get_condition_returns_stored_condition() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");
        let condition = ConditionType::Range(90_000_000_i128, 110_000_000_i128);

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
            &condition,
        );

        let stored = client.get_condition(&call.id);
        assert_eq!(stored, condition);
    }

    #[test]
    fn test_evaluate_condition_target_above() {
        let (env, client, _admin, _om) = setup();

        assert!(client.evaluate_condition(
            &ConditionType::TargetAbove(100_i128),
            &100_i128,
            &101_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::TargetAbove(100_i128),
            &100_i128,
            &100_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::TargetAbove(100_i128),
            &100_i128,
            &99_i128,
        ));

        let _ = env;
    }

    #[test]
    fn test_evaluate_condition_target_below() {
        let (_env, client, _admin, _om) = setup();

        assert!(client.evaluate_condition(
            &ConditionType::TargetBelow(100_i128),
            &100_i128,
            &99_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::TargetBelow(100_i128),
            &100_i128,
            &100_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::TargetBelow(100_i128),
            &100_i128,
            &101_i128,
        ));
    }

    #[test]
    fn test_evaluate_condition_percent_up() {
        let (_env, client, _admin, _om) = setup();

        assert!(client.evaluate_condition(
            &ConditionType::PercentUp(10_u32),
            &100_i128,
            &110_i128,
        ));
        assert!(client.evaluate_condition(
            &ConditionType::PercentUp(10_u32),
            &100_i128,
            &111_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::PercentUp(10_u32),
            &100_i128,
            &109_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::PercentUp(10_u32),
            &0_i128,
            &120_i128,
        ));
    }

    #[test]
    fn test_evaluate_condition_percent_down() {
        let (_env, client, _admin, _om) = setup();

        assert!(client.evaluate_condition(
            &ConditionType::PercentDown(10_u32),
            &100_i128,
            &90_i128,
        ));
        assert!(client.evaluate_condition(
            &ConditionType::PercentDown(10_u32),
            &100_i128,
            &89_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::PercentDown(10_u32),
            &100_i128,
            &91_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::PercentDown(10_u32),
            &0_i128,
            &80_i128,
        ));
    }

    #[test]
    fn test_evaluate_condition_range() {
        let (_env, client, _admin, _om) = setup();

        assert!(client.evaluate_condition(
            &ConditionType::Range(90_i128, 110_i128),
            &100_i128,
            &90_i128,
        ));
        assert!(client.evaluate_condition(
            &ConditionType::Range(90_i128, 110_i128),
            &100_i128,
            &100_i128,
        ));
        assert!(client.evaluate_condition(
            &ConditionType::Range(90_i128, 110_i128),
            &100_i128,
            &110_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::Range(90_i128, 110_i128),
            &100_i128,
            &89_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::Range(90_i128, 110_i128),
            &100_i128,
            &111_i128,
        ));
        assert!(!client.evaluate_condition(
            &ConditionType::Range(110_i128, 90_i128),
            &100_i128,
            &100_i128,
        ));
    }
}

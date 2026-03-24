use soroban_sdk::{Address, Env};

use crate::events::{
    emit_admin_params_changed_address, emit_admin_params_changed_u32, PARAM_ADMIN, PARAM_FEE_BPS,
    PARAM_OUTCOME_MANAGER,
};
use crate::storage::{extend_storage_ttl, get_config, set_config};

/// Transfer admin privileges to a new address.
///
/// # Authorization
/// Current admin must sign.
///
/// # Panics
/// * Contract not initialized
pub fn set_admin(env: Env, new_admin: Address) {
    let mut config = get_config(&env).expect("Contract not initialized");

    config.admin.require_auth();

    let old_admin = config.admin.clone();
    config.admin = new_admin.clone();

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_address(
        &env,
        PARAM_ADMIN,
        &new_admin, // changed_by — the incoming admin authorised this via the old admin's sig
        &old_admin,
        &new_admin,
    );
}

/// Replace the outcome manager.
///
/// # Authorization
/// Current admin must sign.
///
/// # Panics
/// * Contract not initialized
pub fn set_outcome_manager(env: Env, new_manager: Address) {
    let mut config = get_config(&env).expect("Contract not initialized");

    config.admin.require_auth();

    let old_manager = config.outcome_manager.clone();
    config.outcome_manager = new_manager.clone();

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_address(
        &env,
        PARAM_OUTCOME_MANAGER,
        &config.admin,
        &old_manager,
        &new_manager,
    );
}

/// Set the protocol fee in basis points (1 bp = 0.01 %).
///
/// # Arguments
/// * `new_fee_bps` — fee in basis points, must be ≤ 10_000 (100 %)
///
/// # Authorization
/// Current admin must sign.
///
/// # Panics
/// * Contract not initialized
/// * `new_fee_bps` > 10_000
pub fn set_fee(env: Env, new_fee_bps: u32) {
    if new_fee_bps > 10_000 {
        panic!("fee_bps cannot exceed 10_000 (100%)");
    }

    let mut config = get_config(&env).expect("Contract not initialized");

    config.admin.require_auth();

    let old_fee_bps = config.fee_bps;
    config.fee_bps = new_fee_bps;

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_u32(&env, PARAM_FEE_BPS, &config.admin, old_fee_bps, new_fee_bps);
}

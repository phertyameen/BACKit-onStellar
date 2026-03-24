use soroban_sdk::{symbol_short, Env};

/// Emitted when a new oracle outcome report is accepted (before quorum)
pub fn emit_outcome_submitted(
    env: &Env,
    call_id: u64,
    oracle: &soroban_sdk::BytesN<32>,
    outcome: u32,
) {
    env.events().publish(
        (symbol_short!("outcome"), symbol_short!("submitted")),
        (call_id, oracle.clone(), outcome),
    );
}

/// Emitted when quorum is reached and the call is finalized
pub fn emit_outcome_finalized(env: &Env, call_id: u64, outcome: u32, price: i128) {
    env.events().publish(
        (symbol_short!("outcome"), symbol_short!("finalized")),
        (call_id, outcome, price),
    );
}

/// Emitted when a winning staker claims their payout
pub fn emit_payout_claimed(env: &Env, call_id: u64, staker: &soroban_sdk::Address, amount: i128) {
    env.events().publish(
        (symbol_short!("payout"), symbol_short!("claimed")),
        (call_id, staker.clone(), amount),
    );
}

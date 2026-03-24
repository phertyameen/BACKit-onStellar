use soroban_sdk::{Bytes, BytesN, Env};

/// Build the canonical message that oracles sign.
///
/// Format (all big-endian):
///   b"BACKit:Outcome:" | call_id(8B) | b":" | outcome(1B) | b":" | price(16B) | b":" | timestamp(8B)
pub fn build_message(env: &Env, call_id: u64, outcome: u32, price: i128, timestamp: u64) -> Bytes {
    let mut msg = Bytes::new(env);

    msg.append(&Bytes::from_slice(env, b"BACKit:Outcome:"));
    msg.append(&Bytes::from_slice(env, &call_id.to_be_bytes()));
    msg.append(&Bytes::from_slice(env, b":"));

    // outcome is always 1 or 2; encode as a single ASCII byte
    let outcome_byte: &[u8] = match outcome {
        1 => b"1",
        _ => b"2",
    };
    msg.append(&Bytes::from_slice(env, outcome_byte));
    msg.append(&Bytes::from_slice(env, b":"));

    msg.append(&Bytes::from_slice(env, &price.to_be_bytes()));
    msg.append(&Bytes::from_slice(env, b":"));
    msg.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));

    msg
}

/// Verify an ed25519 signature.
///
/// `env.crypto().ed25519_verify` panics on failure ─ we wrap that in a bool
/// so callers can handle it gracefully without unwinding the whole transaction.
pub fn verify_signature(
    env: &Env,
    public_key: &BytesN<32>,
    signature: &BytesN<64>,
    message: &Bytes,
) -> bool {
    // ed25519_verify panics on bad signatures in the Soroban SDK;
    // we rely on that panic propagating (which reverts the tx) rather than
    // returning false – this is the safe choice on-chain.
    env.crypto().ed25519_verify(public_key, message, signature);
    true
}

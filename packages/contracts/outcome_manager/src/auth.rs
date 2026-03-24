use crate::storage::InstanceKey;
use soroban_sdk::{Address, Env};

pub fn require_admin(env: &Env) -> Address {
    let admin: Address = env
        .storage()
        .instance()
        .get(&InstanceKey::Admin)
        .expect("admin not set");

    admin.require_auth();
    admin
}

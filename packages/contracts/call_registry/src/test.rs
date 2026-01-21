#![cfg(test)]

use super::*;
use soroban_sdk::Env;

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CallRegistry);
    let client = CallRegistryClient::new(&env, &contract_id);

    let words = client.hello();
    assert_eq!(words, Symbol::new(&env, "world"));
}

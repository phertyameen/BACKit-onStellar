#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct CallRegistry;

#[contractimpl]
impl CallRegistry {
    pub fn hello(env: Env) -> Symbol {
        Symbol::new(&env, "world")
    }
}

#[cfg(test)]
mod test;

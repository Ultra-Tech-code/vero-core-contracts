use soroban_sdk::{Address, Env};

use crate::types::DataKey;

const LEDGER_TTL: u32 = 100_000;

pub fn add_guardian(env: &Env, admin: Address, guardian: Address) {
    admin.require_auth();

    let key = DataKey::Guardian(guardian.clone());
    if env.storage().instance().has(&key) {
        panic!("Guardian already exists");
    }

    env.storage().instance().set(&key, &true);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_TTL, LEDGER_TTL);
}

pub fn remove_guardian(env: &Env, admin: Address, guardian: Address) {
    admin.require_auth();

    let key = DataKey::Guardian(guardian.clone());
    if !env.storage().instance().has(&key) {
        panic!("Guardian not found");
    }

    env.storage().instance().remove(&key);
}

pub fn is_guardian(env: &Env, guardian: &Address) -> bool {
    let key = DataKey::Guardian(guardian.clone());
    env.storage().instance().get(&key).unwrap_or(false)
}

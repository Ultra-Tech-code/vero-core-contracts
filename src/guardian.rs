#![allow(missing_docs)]

use soroban_sdk::{panic_with_error, Address, Env, Vec};

use crate::types::{ContractError, DataKey, Error};
use crate::validation;

const LEDGER_TTL: u32 = 100_000;

/// Adds a new guardian to the contract.
pub fn add_guardian(env: &Env, admin: Address, guardian: Address) -> Result<(), ContractError> {
    validation::validate_guardian_config(env, &admin, &guardian)?;

    let key = DataKey::Guardian(guardian.clone());
    if !env.storage().instance().has(&key) {
        let mut all_guardians: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::AllGuardians)
            .unwrap_or(Vec::new(env));
        all_guardians.push_back(guardian.clone());
        env.storage()
            .instance()
            .set(&DataKey::AllGuardians, &all_guardians);
    }

    env.storage().instance().set(&key, &true);
    env.storage().instance().extend_ttl(LEDGER_TTL, LEDGER_TTL);
    Ok(())
}

/// Removes an existing guardian from the contract.
pub fn remove_guardian(env: &Env, admin: Address, guardian: Address) -> Result<(), ContractError> {
    validation::validate_admin_address(env, &admin)?;
    validation::validate_external_address(env, &guardian)?;

    let key = DataKey::Guardian(guardian.clone());
    if !env.storage().instance().has(&key) {
        panic_with_error!(env, Error::NotGuardian);
    }

    env.storage().instance().remove(&key);
    Ok(())
}

/// Checks if a given address is a registered guardian.
pub fn is_guardian(env: &Env, guardian: &Address) -> bool {
    let key = DataKey::Guardian(guardian.clone());
    env.storage().instance().get(&key).unwrap_or(false)
}

/// Retrieves a list of all registered guardians.
pub fn get_all_guardians(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::AllGuardians)
        .unwrap_or(Vec::new(env))
}

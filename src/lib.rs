#![no_std]

mod circuit_breaker;
mod drips;
mod guardian;
mod reentrancy;
mod reputation;
mod task;
mod types;
mod vault;
pub mod events;

use soroban_sdk::{contract, contractimpl, Address, Env};
use types::{ContractError, DataKey};

pub use guardian::{add_guardian, remove_guardian, is_guardian};
pub use task::{get_task, register_task};

const DEFAULT_WEIGHT_THRESHOLD: u64 = 300;

#[contract]
pub struct VeroCore;

#[contractimpl]
impl VeroCore {
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .extend_ttl(100_000, 100_000);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    pub fn toggle_pause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        let current = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        env.storage().instance().set(&DataKey::Paused, &!current);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn add_guardian(env: Env, admin: Address, guardian: Address) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        guardian::add_guardian(&env, admin, guardian);
        Ok(())
    }

    pub fn remove_guardian(env: Env, admin: Address, guardian: Address) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        guardian::remove_guardian(&env, admin, guardian);
        Ok(())
    }

    pub fn is_guardian(env: Env, guardian: Address) -> bool {
        guardian::is_guardian(&env, &guardian)
    }

    pub fn set_reputation(
        env: Env,
        admin: Address,
        guardian: Address,
        score: u64,
    ) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        reputation::set_reputation(&env, admin, guardian, score);
        Ok(())
    }

    pub fn get_reputation(env: Env, guardian: Address) -> Option<u64> {
        reputation::get_reputation(&env, &guardian)
    }

    pub fn set_weight_threshold(env: Env, admin: Address, threshold: u64) -> Result<(), ContractError> {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::WeightThreshold, &threshold);
        Ok(())
    }

    pub fn get_weight_threshold(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::WeightThreshold)
            .unwrap_or(DEFAULT_WEIGHT_THRESHOLD)
    }
}

use soroban_sdk::{Address, Env};

use crate::types::ContractError;

pub const MAX_TASK_ID: u64 = u64::MAX / 2;
pub const MAX_TOKEN_AMOUNT: i128 = i128::MAX / 2;
pub const MAX_LOCK_THRESHOLD: i128 = MAX_TOKEN_AMOUNT - 1;
pub const MAX_REPUTATION_SCORE: u64 = 1_000_000_000;
pub const MAX_WEIGHT_THRESHOLD: u64 = 1_000_000_000_000;

pub fn validate_external_address(env: &Env, address: &Address) -> Result<(), ContractError> {
    if address == &env.current_contract_address() {
        return Err(ContractError::InvalidAddress);
    }
    Ok(())
}

pub fn validate_distinct_addresses(left: &Address, right: &Address) -> Result<(), ContractError> {
    if left == right {
        return Err(ContractError::InvalidAddress);
    }
    Ok(())
}

pub fn validate_admin_address(env: &Env, admin: &Address) -> Result<(), ContractError> {
    validate_external_address(env, admin)
}

pub fn validate_guardian_config(
    env: &Env,
    admin: &Address,
    guardian: &Address,
) -> Result<(), ContractError> {
    validate_admin_address(env, admin)?;
    validate_external_address(env, guardian)?;
    validate_distinct_addresses(admin, guardian)
}

pub fn validate_reward_stream_config(
    env: &Env,
    drips_address: &Address,
    contributor: &Address,
    task_id: u64,
) -> Result<(), ContractError> {
    validate_external_address(env, drips_address)?;
    validate_external_address(env, contributor)?;
    validate_distinct_addresses(drips_address, contributor)?;
    validate_task_id(task_id)
}

pub fn validate_task_id(task_id: u64) -> Result<(), ContractError> {
    if task_id == 0 || task_id > MAX_TASK_ID {
        return Err(ContractError::InvalidConfig);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_token_amount(amount: i128) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }
    if amount > MAX_TOKEN_AMOUNT {
        return Err(ContractError::InvalidRange);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_lock_threshold(lock_threshold: i128) -> Result<(), ContractError> {
    if lock_threshold <= 0 {
        return Err(ContractError::InvalidAmount);
    }
    if lock_threshold > MAX_LOCK_THRESHOLD {
        return Err(ContractError::InvalidRange);
    }
    Ok(())
}

pub fn validate_reputation_score(score: u64) -> Result<(), ContractError> {
    if score == 0 {
        return Err(ContractError::InvalidAmount);
    }
    if score > MAX_REPUTATION_SCORE {
        return Err(ContractError::InvalidRange);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_weight_threshold(threshold: u64) -> Result<(), ContractError> {
    if threshold == 0 {
        return Err(ContractError::InvalidAmount);
    }
    if threshold > MAX_WEIGHT_THRESHOLD {
        return Err(ContractError::InvalidRange);
    }
    Ok(())
}

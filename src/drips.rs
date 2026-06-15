use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec as SorobanVec};

use crate::types::{ContractError, DataKey, RewardStream};

/// Performs a cross-contract invocation to the Drips protocol address,
/// starting an automated reward stream for the verified contributor.
///
/// # Arguments
/// * `env` - The contract environment.
/// * `drips_address` - The on-chain address of the Drips protocol contract.
/// * `contributor` - The contributor's Stellar address to receive the stream.
/// * `task_id` - The verified task ID that triggered the reward.
///
/// # Errors
/// * `ContractError::TaskNotVerified` — the task is not yet marked `is_done`.
/// * `ContractError::StreamAlreadyActive` — a stream for this task already exists.
/// * `ContractError::DripsCallFailed` — the cross-contract call to Drips reverted (host trap).
pub fn start_drips_stream(
    env: &Env,
    drips_address: Address,
    contributor: Address,
    task_id: u64,
) -> Result<(), ContractError> {
    // 1. Verify the task is resolved
    let task_key = DataKey::Task(task_id);
    let task: crate::types::Task = env
        .storage()
        .instance()
        .get(&task_key)
        .ok_or(ContractError::NotAuthorized)?;

    if !task.is_done {
        return Err(ContractError::TaskNotVerified);
    }

    // 2. Prevent duplicate stream creation
    let stream_key = DataKey::RewardStream(task_id);
    if env.storage().instance().has(&stream_key) {
        return Err(ContractError::StreamAlreadyActive);
    }

    // 3. Perform the cross-contract call to the Drips protocol.
    //    Build invocation args: (contributor_address, task_id, resolution_status).
    //    `invoke_contract` will host-trap on callee failure, which surfaces as
    //    a panic to the caller. This is the standard Soroban error propagation
    //    model for cross-contract calls.
    let resolution_status: u32 = 1; // 1 = verified/completed
    let args: SorobanVec<Val> = SorobanVec::from_array(
        env,
        [
            contributor.clone().into_val(env),
            task_id.into_val(env),
            resolution_status.into_val(env),
        ],
    );

    env.invoke_contract::<Val>(
        &drips_address,
        &Symbol::new(env, "start_stream"),
        args,
    );

    // 4. Record the reward stream in local storage
    let stream = RewardStream {
        task_id,
        contributor: contributor.clone(),
        drips_contract: drips_address,
        active: true,
    };
    env.storage().instance().set(&stream_key, &stream);

    Ok(())
}

/// Retrieves the reward stream record for a given task, if one exists.
pub fn get_reward_stream(env: &Env, task_id: u64) -> Option<RewardStream> {
    env.storage()
        .instance()
        .get(&DataKey::RewardStream(task_id))
}

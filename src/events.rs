// Event emission helpers for contract state transitions.

use soroban_sdk::{symbol_short, Address, Env};

/// Emits an event when a reward stream is started for a contributor.
///
/// Event topic: `"rw_start"` (reward_stream_started)
/// Event data: `(task_id, contributor_address)`
pub fn emit_reward_stream_started(env: &Env, task_id: u64, contributor: &Address) {
    env.events()
        .publish((symbol_short!("rw_start"),), (task_id, contributor.clone()));
}

/// Emits an event when a cross-contract Drips call fails.
///
/// Event topic: `"rw_fail"` (reward_stream_failed)
/// Event data: `(task_id, contributor_address)`
pub fn emit_reward_stream_failed(env: &Env, task_id: u64, contributor: &Address) {
    env.events()
        .publish((symbol_short!("rw_fail"),), (task_id, contributor.clone()));
}

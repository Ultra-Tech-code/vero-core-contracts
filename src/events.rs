#![allow(missing_docs)]

use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_task_resolved(env: &Env, task_id: u64, weight: u64) {
    env.events()
        .publish((symbol_short!("resolved"),), (task_id, weight));
}

pub fn emit_weighted_vote(env: &Env, task_id: u64, guardian: &Address, weight: u64) {
    env.events().publish(
        (symbol_short!("wt_vote"),),
        (task_id, guardian.clone(), weight),
    );
}

pub fn emit_pause_toggled(env: &Env, paused: bool) {
    env.events().publish((symbol_short!("paused"),), paused);
}

pub fn emit_reward_stream_started(env: &Env, task_id: u64, contributor: &Address) {
    env.events()
        .publish((symbol_short!("rw_start"),), (task_id, contributor.clone()));
}

pub fn emit_reward_stream_failed(env: &Env, task_id: u64, contributor: &Address) {
    env.events()
        .publish((symbol_short!("rw_fail"),), (task_id, contributor.clone()));
}

pub fn emit_circuit_breaker_triggered(env: &Env, failure_count: u32) {
    env.events()
        .publish((symbol_short!("cb_trip"),), (failure_count,));
}

pub fn emit_withdrawal_requested(env: &Env, request_id: u64, recipient: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("wd_req"),),
        (request_id, recipient.clone(), amount),
    );
}

pub fn emit_withdrawal_executed(env: &Env, request_id: u64, recipient: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("wd_exec"),),
        (request_id, recipient.clone(), amount),
    );
}

pub fn emit_withdrawal_cancelled(env: &Env, request_id: u64) {
    env.events()
        .publish((symbol_short!("wd_cncl"),), request_id);
}

pub fn emit_task_cancelled(env: &Env, task_id: u64) {
    env.events().publish((symbol_short!("cancelled"),), task_id);
}

pub fn emit_snapshot_recorded(env: &Env, timestamp: u64) {
    env.events()
        .publish((symbol_short!("snapshot"),), timestamp);
}

pub fn emit_task_purged(env: &Env, task_id: u64) {
    env.events()
        .publish((symbol_short!("purged"),), task_id);
}

pub fn emit_contract_initialized(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("inited"),), (admin.clone(),));
}

pub fn emit_guardian_added(env: &Env, admin: &Address, guardian: &Address) {
    env.events().publish(
        (symbol_short!("gd_add"),),
        (admin.clone(), guardian.clone()),
    );
}

pub fn emit_guardian_removed(env: &Env, admin: &Address, guardian: &Address) {
    env.events().publish(
        (symbol_short!("gd_rm"),),
        (admin.clone(), guardian.clone()),
    );
}

pub fn emit_reputation_set(env: &Env, admin: &Address, guardian: &Address, score: u64) {
    env.events().publish(
        (symbol_short!("rep_set"),),
        (admin.clone(), guardian.clone(), score),
    );
}

pub fn emit_tokens_locked(env: &Env, guardian: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("tk_lock"),), (guardian.clone(), amount));
}

pub fn emit_timelock_started(env: &Env, guardian: &Address) {
    env.events()
        .publish((symbol_short!("tm_start"),), (guardian.clone(),));
}

pub fn emit_tokens_unlocked(env: &Env, guardian: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("tk_unlk"),), (guardian.clone(), amount));
}

pub fn emit_guardian_resigned(env: &Env, guardian: &Address) {
    env.events()
        .publish((symbol_short!("gd_res"),), (guardian.clone(),));
}

pub fn emit_threshold_set(env: &Env, admin: &Address, threshold: u64) {
    env.events().publish(
        (symbol_short!("th_set"),),
        (admin.clone(), threshold),
    );
}

pub fn emit_vault_set(env: &Env, admin: &Address, vault: &Address) {
    env.events().publish(
        (symbol_short!("vault"),),
        (admin.clone(), vault.clone()),
    );
}

pub fn emit_task_registered(env: &Env, admin: &Address, task_id: u64) {
    env.events().publish(
        (symbol_short!("reg"),),
        (admin.clone(), task_id),
    );
}

pub fn emit_task_archived(env: &Env, task_id: u64) {
    env.events()
        .publish((symbol_short!("archived"),), task_id);
}

pub fn emit_circuit_breaker_reset(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("cb_rst"),), (admin.clone(),));
}

pub fn emit_contract_upgraded(env: &Env, admin: &Address, wasm_hash: &soroban_sdk::BytesN<32>) {
    env.events().publish(
        (symbol_short!("upgraded"),),
        (admin.clone(), wasm_hash.clone()),
    );
}

pub fn emit_upgrade_signers_set(env: &Env, signer_count: u32, threshold: u32) {
    env.events()
        .publish((symbol_short!("up_sig"),), (signer_count, threshold));
}

pub fn emit_upgrade_proposed(env: &Env, signer: &Address) {
    env.events()
        .publish((symbol_short!("up_prop"),), (signer.clone(),));
}

pub fn emit_upgrade_approved(env: &Env, signer: &Address, count: u32, threshold: u32) {
    env.events().publish(
        (symbol_short!("up_app"),),
        (signer.clone(), count, threshold),
    );
}

pub fn emit_upgrade_executed(env: &Env) {
    env.events().publish((symbol_short!("up_exec"),), ());
}

pub fn emit_upgrade_cancelled(env: &Env) {
    env.events().publish((symbol_short!("up_cncl"),), ());
}

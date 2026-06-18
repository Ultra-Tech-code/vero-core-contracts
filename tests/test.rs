#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, Vec as SorobanVec,
};
use vero_core_contracts::{register_tasks, Operation, VeroContractClient};

const LOCK_THRESHOLD: i128 = 100;
const MAX_TASK_ID: u64 = u64::MAX / 2;
const MAX_TOKEN_AMOUNT: i128 = i128::MAX / 2;
const MAX_LOCK_THRESHOLD: i128 = MAX_TOKEN_AMOUNT - 1;
const MAX_REPUTATION_SCORE: u64 = 1_000_000_000;
const MAX_WEIGHT_THRESHOLD: u64 = 1_000_000_000_000;
const MAX_REGISTER_TASK_BATCH_SIZE: u64 = 32;
const ARCHIVE_AFTER_SECONDS: u64 = 30 * 24 * 60 * 60;

fn setup() -> (Env, Address, Address, Address, VeroContractClient<'static>) {
    setup_with_lock_threshold(LOCK_THRESHOLD)
}

fn setup_with_lock_threshold(
    lock_threshold: i128,
) -> (Env, Address, Address, Address, VeroContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, vero_core_contracts::VeroContract);
    let client = VeroContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin);
    let token_addr = token.address();

    client.initialize(&token_addr, &lock_threshold);

    (env, contract_id, admin, token_addr, client)
}

fn add_guardian_with_rep(
    env: &Env,
    client: &VeroContractClient,
    admin: &Address,
    score: u64,
) -> Address {
    let guardian = Address::generate(env);
    client.add_guardian(admin, &guardian);
    client.set_reputation(admin, &guardian, &score);
    guardian
}

fn mint_and_lock(
    env: &Env,
    token: &Address,
    client: &VeroContractClient,
    guardian: &Address,
    amount: i128,
) {
    let asset_client = soroban_sdk::token::StellarAssetClient::new(env, token);
    asset_client.mint(guardian, &amount);
    client.lock_tokens(guardian, &amount);
}

fn resolved_task(
    env: &Env,
    token: &Address,
    client: &VeroContractClient,
    admin: &Address,
    task_id: u64,
) -> Address {
    let guardian = add_guardian_with_rep(env, client, admin, 300);
    client.register_task(admin, &task_id);
    mint_and_lock(env, token, client, &guardian, LOCK_THRESHOLD + 1);
    client.vote(&guardian, &task_id);
    guardian
}

#[test]
fn valid_admin_config_update_succeeds() {
    let (env, _contract_id, admin, _token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 500);
    let vault = Address::generate(&env);

    client.set_weight_threshold(&admin, &500);
    client.set_vault_address(&admin, &vault);

    assert_eq!(client.get_weight_threshold(), 500);
    assert!(client.is_guardian(&guardian));
    assert_eq!(client.get_reputation(&guardian), Some(500));

    let snapshot = client.get_snapshot();
    assert_eq!(snapshot.vault_address, Some(vault));
}

#[test]
fn initialize_rejects_self_token_and_invalid_thresholds_without_mutation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, vero_core_contracts::VeroContract);
    let client = VeroContractClient::new(&env, &contract_id);

    assert!(client
        .try_initialize(&contract_id, &LOCK_THRESHOLD)
        .is_err());

    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin);
    let token_addr = token.address();

    assert!(client.try_initialize(&token_addr, &0).is_err());
    assert!(client.try_initialize(&token_addr, &-1).is_err());
    assert!(client
        .try_initialize(&token_addr, &MAX_TOKEN_AMOUNT)
        .is_err());
    assert!(client.try_initialize(&token_addr, &i128::MAX).is_err());

    client.initialize(&token_addr, &LOCK_THRESHOLD);
    assert!(client.try_initialize(&token_addr, &LOCK_THRESHOLD).is_err());
}

#[test]
fn numeric_minimum_and_maximum_boundaries_succeed() {
    let (env, _contract_id, admin, token, client) = setup_with_lock_threshold(1);
    let guardian = add_guardian_with_rep(&env, &client, &admin, 1);

    client.set_reputation(&admin, &guardian, &MAX_REPUTATION_SCORE);
    client.set_weight_threshold(&admin, &MAX_WEIGHT_THRESHOLD);
    client.register_task(&admin, &MAX_TASK_ID);
    mint_and_lock(&env, &token, &client, &guardian, MAX_TOKEN_AMOUNT);

    assert_eq!(client.get_reputation(&guardian), Some(MAX_REPUTATION_SCORE));
    assert_eq!(client.get_weight_threshold(), MAX_WEIGHT_THRESHOLD);
    assert!(client.get_task(&MAX_TASK_ID).is_some());
}

#[test]
fn maximum_lock_threshold_still_allows_max_balance_vote() {
    let (env, _contract_id, admin, token, client) = setup_with_lock_threshold(MAX_LOCK_THRESHOLD);
    let guardian = add_guardian_with_rep(&env, &client, &admin, 300);

    client.register_task(&admin, &1);
    mint_and_lock(&env, &token, &client, &guardian, MAX_TOKEN_AMOUNT);
    client.vote(&guardian, &1);

    let task = client.get_task(&1).unwrap();
    assert!(task.is_done);
    assert_eq!(task.total_weight_accrued, 300);
}

#[test]
fn guardian_address_validation_rejects_self_and_duplicate_roles() {
    let (_env, contract_id, admin, _token, client) = setup();

    assert!(client.try_add_guardian(&admin, &contract_id).is_err());
    assert!(client.try_add_guardian(&admin, &admin).is_err());
    assert!(!client.is_guardian(&contract_id));
    assert!(!client.is_guardian(&admin));
}

#[test]
fn reputation_validation_rejects_zero_over_max_and_non_guardian() {
    let (env, _contract_id, admin, _token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 100);
    let non_guardian = Address::generate(&env);

    assert!(client.try_set_reputation(&admin, &guardian, &0).is_err());
    assert!(client
        .try_set_reputation(&admin, &guardian, &u64::MAX)
        .is_err());
    assert!(client
        .try_set_reputation(&admin, &non_guardian, &100)
        .is_err());

    assert_eq!(client.get_reputation(&guardian), Some(100));
    assert_eq!(client.get_reputation(&non_guardian), None);
}

#[test]
fn weight_threshold_validation_rejects_zero_and_over_max_without_mutation() {
    let (_env, contract_id, admin, _token, client) = setup();

    client.set_weight_threshold(&admin, &750);
    assert_eq!(client.get_weight_threshold(), 750);

    assert!(client.try_set_weight_threshold(&admin, &0).is_err());
    assert!(client.try_set_weight_threshold(&admin, &u64::MAX).is_err());
    assert!(client.try_set_weight_threshold(&contract_id, &500).is_err());

    assert_eq!(client.get_weight_threshold(), 750);
}

#[test]
fn task_id_validation_rejects_zero_and_over_max_without_mutation() {
    let (_env, contract_id, admin, _token, client) = setup();

    assert!(client.try_register_task(&admin, &0).is_err());
    assert!(client.try_register_task(&admin, &u64::MAX).is_err());
    assert!(client.try_register_task(&contract_id, &42).is_err());
    assert!(client.get_task(&0).is_none());
    assert!(client.get_task(&u64::MAX).is_none());
    assert!(client.get_task(&42).is_none());

    client.register_task(&admin, &42);
    assert!(client.get_task(&42).is_some());
}

#[test]
fn vault_address_validation_rejects_self_without_mutation() {
    let (env, contract_id, admin, _token, client) = setup();
    let vault = Address::generate(&env);

    client.set_vault_address(&admin, &vault);
    assert_eq!(client.get_snapshot().vault_address, Some(vault.clone()));

    assert!(client.try_set_vault_address(&admin, &contract_id).is_err());

    assert_eq!(client.get_snapshot().vault_address, Some(vault));
}

#[test]
fn reward_stream_validation_rejects_invalid_addresses_and_ids() {
    let (env, contract_id, admin, token, client) = setup();
    let contributor = Address::generate(&env);
    let drips_contract_id = env.register_contract(None, MockDripsContract);

    resolved_task(&env, &token, &client, &admin, 77);

    assert!(client
        .try_start_reward_stream(&admin, &contract_id, &contributor, &77)
        .is_err());
    assert!(client
        .try_start_reward_stream(&admin, &drips_contract_id, &contract_id, &77)
        .is_err());
    assert!(client
        .try_start_reward_stream(&admin, &contributor, &contributor, &77)
        .is_err());
    assert!(client
        .try_start_reward_stream(&admin, &drips_contract_id, &contributor, &0)
        .is_err());

    assert!(client.get_reward_stream(&77).is_none());

    client.start_reward_stream(&admin, &drips_contract_id, &contributor, &77);
    let stream = client.get_reward_stream(&77).unwrap();
    assert_eq!(stream.contributor, contributor);
    assert_eq!(stream.drips_contract, drips_contract_id);
    assert!(stream.active);
}

#[test]
fn token_amount_validation_rejects_zero_negative_and_over_max_without_locking() {
    let (env, _contract_id, admin, token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 300);
    client.register_task(&admin, &88);

    assert!(client.try_lock_tokens(&guardian, &0).is_err());
    assert!(client.try_lock_tokens(&guardian, &-1).is_err());
    assert!(client.try_lock_tokens(&guardian, &i128::MAX).is_err());
    assert!(client.try_vote(&guardian, &88).is_err());

    mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD + 1);
    client.vote(&guardian, &88);
    assert_eq!(client.get_task(&88).unwrap().votes, 1);
}

#[test]
fn aggregate_locked_amount_above_max_is_rejected_without_transfer() {
    let (env, _contract_id, admin, token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 300);
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token);
    let balance_client = soroban_sdk::token::Client::new(&env, &token);

    mint_and_lock(&env, &token, &client, &guardian, MAX_TOKEN_AMOUNT);
    token_client.mint(&guardian, &1);

    assert_eq!(balance_client.balance(&guardian), 1);
    assert!(client.try_lock_tokens(&guardian, &1).is_err());
    assert_eq!(balance_client.balance(&guardian), 1);

    client.register_task(&admin, &89);
    client.vote(&guardian, &89);
    assert_eq!(client.get_task(&89).unwrap().votes, 1);
}

#[test]
fn unauthorized_admin_call_is_still_rejected_and_state_is_unchanged() {
    let env = Env::default();
    let contract_id = env.register_contract(None, vero_core_contracts::VeroContract);
    let client = VeroContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin);
    let token_addr = token.address();
    client.initialize(&token_addr, &LOCK_THRESHOLD);

    assert!(client.try_set_weight_threshold(&admin, &500).is_err());
    assert_eq!(client.get_weight_threshold(), 300);
}

#[test]
fn existing_valid_vote_and_unlock_flows_still_pass() {
    let (env, _contract_id, admin, token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 300);
    client.register_task(&admin, &99);
    mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD + 1);

    client.vote(&guardian, &99);
    let task = client.get_task(&99).unwrap();
    assert!(task.is_done);
    assert_eq!(task.total_weight_accrued, 300);

    assert!(client.try_unlock_tokens(&guardian).is_err());
    client.resign_guardian(&guardian);
    assert!(!client.is_guardian(&guardian));

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&guardian), LOCK_THRESHOLD + 1);
}

#[test]
fn paused_contract_rejects_config_updates() {
    let (env, _contract_id, admin, _token, client) = setup();
    let guardian = Address::generate(&env);

    client.pause(&admin);

    assert!(client.try_add_guardian(&admin, &guardian).is_err());
    assert!(client.try_set_weight_threshold(&admin, &400).is_err());
    assert_eq!(client.get_weight_threshold(), 300);

    client.unpause(&admin);
    client.add_guardian(&admin, &guardian);
    assert!(client.is_guardian(&guardian));
}

#[test]
fn register_task_batch_size_boundaries_are_enforced() {
    let (env, contract_id, admin, _token, client) = setup();
    let mut max_batch = SorobanVec::new(&env);
    for task_id in 1..=MAX_REGISTER_TASK_BATCH_SIZE {
        max_batch.push_back(task_id);
    }

    let max_result = env.as_contract(&contract_id, || {
        register_tasks(&env, admin.clone(), max_batch)
    });
    assert!(max_result.is_ok());
    assert!(client.get_task(&1).is_some());
    assert!(client.get_task(&MAX_REGISTER_TASK_BATCH_SIZE).is_some());

    let mut oversized_batch = SorobanVec::new(&env);
    for task_id in 100..=(100 + MAX_REGISTER_TASK_BATCH_SIZE) {
        oversized_batch.push_back(task_id);
    }

    let oversized_result = env.as_contract(&contract_id, || {
        register_tasks(&env, admin.clone(), oversized_batch)
    });
    assert!(oversized_result.is_err());
    assert!(client.get_task(&100).is_none());
    assert!(client
        .get_task(&(100 + MAX_REGISTER_TASK_BATCH_SIZE))
        .is_none());
}

#[test]
fn archive_timestamp_underflow_is_safely_rejected_without_mutation() {
    let (env, _contract_id, admin, token, client) = setup();

    env.ledger().set_timestamp(1_000);
    resolved_task(&env, &token, &client, &admin, 61);
    assert_eq!(client.get_task(&61).unwrap().resolved_at, 1_000);

    env.ledger().set_timestamp(0);
    assert!(client.try_archive_task(&61).is_err());
    assert!(client.get_task(&61).is_some());
    assert!(client.get_archived_task(&61).is_none());

    env.ledger().set_timestamp(1_000 + ARCHIVE_AFTER_SECONDS);
    assert!(client.try_archive_task(&61).is_err());
    assert!(client.get_task(&61).is_some());
    assert!(client.get_archived_task(&61).is_none());

    env.ledger()
        .set_timestamp(1_000 + ARCHIVE_AFTER_SECONDS + 1);
    client.archive_task(&61);
    assert!(client.get_task(&61).is_none());
    assert!(client.get_archived_task(&61).is_some());
}

#[test]
fn invalid_numeric_inputs_do_not_emit_success_events() {
    let (env, _contract_id, admin, token, client) = setup();
    let contributor = Address::generate(&env);
    let drips_contract_id = env.register_contract(None, MockDripsContract);

    let before_register_events = env.events().all().len();
    assert!(client.try_register_task(&admin, &0).is_err());
    assert_eq!(env.events().all().len(), before_register_events);

    resolved_task(&env, &token, &client, &admin, 62);
    let before_stream_events = env.events().all().len();
    assert!(client
        .try_start_reward_stream(&admin, &drips_contract_id, &contributor, &0)
        .is_err());
    assert_eq!(env.events().all().len(), before_stream_events);
    assert!(client.get_reward_stream(&0).is_none());
}

#[test]
fn legacy_add_guardian_and_register_task_flow_still_passes() {
    let (env, _contract_id, admin, _token, client) = setup();
    let guardian = Address::generate(&env);

    client.add_guardian(&admin, &guardian);
    client.register_task(&admin, &1);

    let task = client.get_task(&1).unwrap();
    assert_eq!(task.id, 1);
    assert_eq!(task.votes, 0);
    assert_eq!(task.total_weight_accrued, 0);
    assert_eq!(task.resolved_at, 0);
    assert!(!task.is_done);
}

#[test]
fn legacy_voting_power_views_still_pass() {
    let (env, _contract_id, admin, _token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 150);
    let stranger = Address::generate(&env);

    assert_eq!(client.calculate_voting_power(&guardian), Some(150));
    assert_eq!(client.calculate_voting_power(&stranger), None);
}

#[test]
fn legacy_multiple_guardian_weight_accumulates() {
    let (env, _contract_id, admin, token, client) = setup();
    client.set_weight_threshold(&admin, &300);

    let g1 = add_guardian_with_rep(&env, &client, &admin, 100);
    let g2 = add_guardian_with_rep(&env, &client, &admin, 100);
    let g3 = add_guardian_with_rep(&env, &client, &admin, 100);
    client.register_task(&admin, &42);

    for guardian in [&g1, &g2, &g3] {
        mint_and_lock(&env, &token, &client, guardian, LOCK_THRESHOLD + 1);
        client.vote(guardian, &42);
    }

    let task = client.get_task(&42).unwrap();
    assert_eq!(task.votes, 3);
    assert_eq!(task.total_weight_accrued, 300);
    assert!(task.is_done);
}

#[test]
fn legacy_low_weight_votes_do_not_resolve_early() {
    let (env, _contract_id, admin, token, client) = setup();
    client.set_weight_threshold(&admin, &300);
    client.register_task(&admin, &30);

    for _ in 0..5 {
        let guardian = add_guardian_with_rep(&env, &client, &admin, 50);
        mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD + 1);
        client.vote(&guardian, &30);
    }

    let task = client.get_task(&30).unwrap();
    assert_eq!(task.votes, 5);
    assert_eq!(task.total_weight_accrued, 250);
    assert!(!task.is_done);
}

#[test]
fn legacy_reputation_can_be_updated() {
    let (env, _contract_id, admin, _token, client) = setup();
    let guardian = Address::generate(&env);

    client.add_guardian(&admin, &guardian);
    client.set_reputation(&admin, &guardian, &100);
    assert_eq!(client.get_reputation(&guardian), Some(100));

    client.set_reputation(&admin, &guardian, &500);
    assert_eq!(client.get_reputation(&guardian), Some(500));
    assert_eq!(client.calculate_voting_power(&guardian), Some(500));
}

#[test]
fn legacy_vote_rejections_still_pass() {
    let (env, _contract_id, admin, token, client) = setup();
    let no_rep = Address::generate(&env);
    let guardian = add_guardian_with_rep(&env, &client, &admin, 100);
    let stranger = Address::generate(&env);

    client.add_guardian(&admin, &no_rep);
    client.register_task(&admin, &7);
    mint_and_lock(&env, &token, &client, &no_rep, LOCK_THRESHOLD + 1);
    mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD + 1);

    assert!(client.try_vote(&no_rep, &7).is_err());
    assert!(client.try_vote(&guardian, &999).is_err());
    assert!(client.try_vote(&stranger, &7).is_err());
}

#[test]
fn legacy_reward_stream_rejects_unverified_and_duplicate_tasks() {
    let (env, _contract_id, admin, token, client) = setup();
    let contributor = Address::generate(&env);
    let drips_contract_id = env.register_contract(None, MockDripsContract);

    client.register_task(&admin, &50);
    assert!(client
        .try_start_reward_stream(&admin, &drips_contract_id, &contributor, &50)
        .is_err());

    resolved_task(&env, &token, &client, &admin, 51);
    client.start_reward_stream(&admin, &drips_contract_id, &contributor, &51);
    assert!(client
        .try_start_reward_stream(&admin, &drips_contract_id, &contributor, &51)
        .is_err());
}

#[test]
fn legacy_token_locking_and_unlocking_flows_still_pass() {
    let (env, _contract_id, admin, token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 100);
    let non_guardian = Address::generate(&env);

    client.register_task(&admin, &100);
    assert!(client.try_vote(&guardian, &100).is_err());

    mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD);
    assert!(client.try_vote(&guardian, &100).is_err());

    mint_and_lock(&env, &token, &client, &guardian, 1);
    client.vote(&guardian, &100);
    assert_eq!(client.get_task(&100).unwrap().votes, 1);

    mint_and_lock(&env, &token, &client, &non_guardian, 150);
    client.unlock_tokens(&non_guardian);
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&non_guardian), 150);
}

#[test]
fn legacy_reentrancy_lock_released_after_failed_vote() {
    let (env, _contract_id, admin, token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 100);
    let stranger = Address::generate(&env);

    client.register_task(&admin, &303);
    let _ = client.try_vote(&stranger, &303);

    mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD + 1);
    client.vote(&guardian, &303);
    assert_eq!(client.get_task(&303).unwrap().votes, 1);
}

#[test]
fn legacy_pause_and_circuit_breaker_flows_still_pass() {
    let (env, _contract_id, admin, token, client) = setup();
    let guardian = add_guardian_with_rep(&env, &client, &admin, 100);

    assert!(!client.is_paused());
    client.toggle_pause(&admin);
    assert!(client.is_paused());
    assert!(client.try_register_task(&admin, &2).is_err());

    client.toggle_pause(&admin);
    client.register_task(&admin, &2);
    mint_and_lock(&env, &token, &client, &guardian, LOCK_THRESHOLD + 1);

    for _ in 0..51 {
        client.record_failure();
    }
    assert!(client.is_paused());
    assert!(client.try_vote(&guardian, &2).is_err());

    client.reset_circuit_breaker(&admin);
    assert!(!client.is_paused());
    client.vote(&guardian, &2);
    assert_eq!(client.get_task(&2).unwrap().votes, 1);
}

#[test]
fn legacy_gas_cost_estimates_still_pass() {
    let (_env, _contract_id, _admin, _token, client) = setup();
    let ops = [
        Operation::RegisterTask,
        Operation::Vote,
        Operation::AddGuardian,
        Operation::SetReputation,
        Operation::LockTokens,
        Operation::UnlockTokens,
        Operation::ResignGuardian,
        Operation::SetWeightThreshold,
        Operation::StartRewardStream,
        Operation::TogglePause,
        Operation::RecordFailure,
        Operation::ResetCircuitBreaker,
        Operation::UpgradeContract,
    ];

    for op in ops {
        assert!(client.get_estimated_cost(&op) > 500_000);
    }

    assert!(
        client.get_estimated_cost(&Operation::Vote)
            >= client.get_estimated_cost(&Operation::RegisterTask)
    );
    assert!(
        client.get_estimated_cost(&Operation::UpgradeContract)
            >= client.get_estimated_cost(&Operation::Vote)
    );
    assert_eq!(
        client.get_estimated_cost(&Operation::SetWeightThreshold),
        650_000
    );
    assert_eq!(
        client.get_estimated_cost(&Operation::UpgradeContract),
        2_500_000
    );
}

#[contract]
pub struct MockDripsContract;

#[contractimpl]
impl MockDripsContract {
    pub fn start_stream(_env: Env, _contributor: Address, _task_id: u64, _resolution_status: u32) {}
}

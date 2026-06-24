use crate::contracts::logic;
use crate::types::{BatchCall, ContractError, DataKey, RewardStream, Snapshot};
use crate::DEFAULT_WEIGHT_THRESHOLD;
use crate::{circuit_breaker, drips, events, guardian, reputation, storage, task};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

#[contract]
pub struct VeroContract;

#[contractimpl]
impl VeroContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        lock_threshold: i128,
    ) -> Result<(), ContractError> {
        if env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Initialized)
            .unwrap_or(false)
        {
            return Err(ContractError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage()
            .instance()
            .set(&DataKey::LockThreshold, &lock_threshold);
        env.storage().instance().set(&DataKey::Paused, &false);
        
        // Grant Admin role to the deployer/initial admin
        let admin_role_key = DataKey::RoleAssignment(admin.clone(), crate::types::Role::Admin);
        env.storage().instance().set(&admin_role_key, &true);
        
        env.storage().instance().extend_ttl(100_000, 100_000);
        events::emit_contract_initialized(&env, &admin);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    pub fn toggle_pause(env: Env, admin: Address) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::EmergencyManager)?;
        let current = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        let new_paused = !current;
        env.storage().instance().set(&DataKey::Paused, &new_paused);
        events::emit_pause_toggled(&env, new_paused);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::EmergencyManager)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        events::emit_pause_toggled(&env, true);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::EmergencyManager)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        events::emit_pause_toggled(&env, false);
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
        guardian::add_guardian(&env, admin.clone(), guardian.clone())?;
        events::emit_guardian_added(&env, &admin, &guardian);
        Ok(())
    }

    pub fn remove_guardian(
        env: Env,
        admin: Address,
        guardian: Address,
    ) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        guardian::remove_guardian(&env, admin.clone(), guardian.clone())?;
        events::emit_guardian_removed(&env, &admin, &guardian);
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
        reputation::set_reputation(&env, admin.clone(), guardian.clone(), score)?;
        events::emit_reputation_set(&env, &admin, &guardian, score);
        Ok(())
    }

    pub fn get_reputation(env: Env, guardian: Address) -> Option<u64> {
        reputation::get_reputation(&env, &guardian)
    }

    pub fn calculate_voting_power(env: Env, guardian: Address) -> Option<u64> {
        reputation::calculate_voting_power(&env, &guardian)
    }

    pub fn lock_tokens(env: Env, guardian: Address, amount: i128) -> Result<(), ContractError> {
        logic::lock_tokens(&env, guardian, amount)
    }

    pub fn request_unlock(env: Env, guardian: Address) -> Result<(), ContractError> {
        logic::request_unlock(&env, guardian)
    }

    pub fn unlock_tokens(env: Env, guardian: Address) -> Result<(), ContractError> {
        logic::unlock_tokens(&env, guardian)
    }

    pub fn resign_guardian(env: Env, guardian: Address) -> Result<(), ContractError> {
        logic::resign_guardian(&env, guardian)
    }

    pub fn set_weight_threshold(
        env: Env,
        admin: Address,
        threshold: u64,
    ) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::ConfigManager)?;
        env.storage()
            .instance()
            .set(&DataKey::WeightThreshold, &threshold);
        events::emit_threshold_set(&env, &admin, threshold);
        Ok(())
    }

    pub fn get_weight_threshold(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::WeightThreshold)
            .unwrap_or(DEFAULT_WEIGHT_THRESHOLD)
    }

    pub fn set_vault_address(env: Env, admin: Address, vault: Address) {
        // Use try-catch pattern via unwrap since this function has no Result return
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::ConfigManager)
            .unwrap();
        env.storage().instance().set(&DataKey::VaultAddress, &vault);
        events::emit_vault_set(&env, &admin, &vault);
    }

    pub fn register_task(env: Env, admin: Address, task_id: u64, min_votes_required: u32) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::TaskManager)?;
        let task_ids = soroban_sdk::vec![&env, task_id];
        task::register_tasks(&env, admin, task_ids, min_votes_required)
    }

    pub fn cancel_task(env: Env, admin: Address, task_id: u64) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        task::cancel_task(&env, admin, task_id)
    }

    /// Purge a terminal task (done or cancelled) from contract storage.
    ///
    /// Removes the task struct, its voter list, each individual `Voted` record,
    /// and the task id from the `AllTasks` index. Reduces on-chain state size
    /// and the cost of future `get_snapshot` calls.
    ///
    /// Reverts with `TaskNotFound` if no task exists, `TaskNotTerminal` if the
    /// task is still active, and `NotAuthorized` if the caller is not the admin.
    pub fn purge_task(env: Env, admin: Address, task_id: u64) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::TaskManager)?;
        task::purge_task(&env, admin, task_id)
    }

    pub fn vote(env: Env, guardian: Address, task_id: u64) -> Result<(), ContractError> {
        logic::process_vote(&env, guardian, task_id)
    }

    pub fn vote_batch(
        env: Env,
        guardian: Address,
        task_ids: Vec<u64>,
    ) -> Result<(), ContractError> {
        logic::process_vote_batch(&env, guardian, task_ids)
    }

    pub fn get_task(env: Env, task_id: u64) -> Option<crate::types::Task> {
        task::get_task(&env, task_id)
    }

    pub fn archive_task(env: Env, task_id: u64) -> Result<(), ContractError> {
        storage::archive_task(&env, task_id)?;
        events::emit_task_archived(&env, task_id);
        Ok(())
    }

    pub fn get_archived_task(env: Env, task_id: u64) -> Option<crate::types::Task> {
        storage::get_archived_task(&env, task_id)
    }

    pub fn start_reward_stream(
        env: Env,
        admin: Address,
        drips_address: Address,
        contributor: Address,
        task_id: u64,
    ) -> Result<(), ContractError> {
        circuit_breaker::require_not_paused(&env)?;
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::TreasuryManager)?;

        let result = drips::start_drips_stream(&env, drips_address, contributor.clone(), task_id);

        match &result {
            Ok(()) => events::emit_reward_stream_started(&env, task_id, &contributor),
            Err(_) => events::emit_reward_stream_failed(&env, task_id, &contributor),
        }

        result
    }

    pub fn get_reward_stream(env: Env, task_id: u64) -> Option<RewardStream> {
        drips::get_reward_stream(&env, task_id)
    }

    pub fn record_failure(env: Env) {
        circuit_breaker::record_failure(&env);
    }

    pub fn reset_circuit_breaker(env: Env, admin: Address) {
        if circuit_breaker::reset(&env, admin.clone()).is_ok() {
            events::emit_circuit_breaker_reset(&env, &admin);
        }
    }

    pub fn get_estimated_cost(_env: Env, op: crate::types::Operation) -> u64 {
        crate::gas::get_estimated_cost(op)
    }

    pub fn upgrade_contract(env: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::Admin).unwrap();
        env.deployer().update_current_contract_wasm(new_wasm_hash.clone());
        events::emit_contract_upgraded(&env, &admin, &new_wasm_hash);
    }

    // ─── Multi-sig upgrade management ────────────────────────────────────────

    /// Configure the list of authorized upgrade signers and the required quorum.
    ///
    /// Only the contract admin may call this function. It overwrites any previous
    /// multi-sig configuration and clears any pending upgrade proposal.
    ///
    /// # Arguments
    /// * `signers`   — List of addresses authorized to propose/approve upgrades.
    /// * `threshold` — Minimum number of approvals required to execute an upgrade.
    ///
    /// # Errors
    /// * `NotAuthorized` — Caller is not the contract admin.
    /// * `InvalidUpgradeConfig` — Threshold is zero or exceeds the number of signers.
    pub fn set_upgrade_signers(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
    ) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::Admin)?;

        if threshold == 0 || threshold > signers.len() {
            return Err(ContractError::InvalidUpgradeConfig);
        }

        // Clear any pending upgrade when reconfiguring
        env.storage().instance().remove(&DataKey::PendingUpgradeWasm);
        env.storage()
            .instance()
            .remove(&DataKey::PendingUpgradeApprovals);

        env.storage()
            .instance()
            .set(&DataKey::UpgradeSigners, &signers);
        env.storage()
            .instance()
            .set(&DataKey::UpgradeThreshold, &threshold);

        events::emit_upgrade_signers_set(&env, signers.len(), threshold);
        Ok(())
    }

    /// Returns the currently configured list of authorized upgrade signers.
    pub fn get_upgrade_signers(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::UpgradeSigners)
            .unwrap_or(Vec::new(&env))
    }

    /// Returns the minimum number of upgrade approvals required (quorum).
    pub fn get_upgrade_threshold(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::UpgradeThreshold)
            .unwrap_or(0u32)
    }

    /// Propose a new upgrade WASM hash as an upgrade signer.
    ///
    /// If no pending upgrade exists, creates one and records the caller's
    /// approval. The caller is added to the approvals list.
    ///
    /// If a pending upgrade exists with a **different** WASM hash, the call
    /// reverts. If the hash matches, the caller is added to the approval list
    /// (same effect as calling `approve_upgrade`).
    ///
    /// # Errors
    /// * `NotUpgradeSigner` — Caller is not in the authorized signers list.
    /// * `NoPendingUpgrade` — (not applicable; propose creates one).
    /// * `AlreadyApproved` — Caller has already approved.
    pub fn propose_upgrade(
        env: Env,
        signer: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        signer.require_auth();

        // Verify signer is authorized
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeSigners)
            .ok_or(ContractError::NotUpgradeSigner)?;
        if !signers.contains(signer.clone()) {
            return Err(ContractError::NotUpgradeSigner);
        }

        // Check if there's an existing pending upgrade
        if let Some(existing_hash) = env
            .storage()
            .instance()
            .get::<_, BytesN<32>>(&DataKey::PendingUpgradeWasm)
        {
            // If hashes differ, reject
            if existing_hash != new_wasm_hash {
                return Err(ContractError::InvalidUpgradeConfig);
            }
            // Hash matches — just add approval (same as approve_upgrade)
            return Self::approve_upgrade(env, signer);
        }

        // No pending upgrade — create one
        env.storage()
            .instance()
            .set(&DataKey::PendingUpgradeWasm, &new_wasm_hash);

        // Record the first approval
        let mut approvals: Vec<Address> = Vec::new(&env);
        approvals.push_back(signer.clone());
        env.storage()
            .instance()
            .set(&DataKey::PendingUpgradeApprovals, &approvals);

        events::emit_upgrade_proposed(&env, &signer);

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeThreshold)
            .unwrap_or(0u32);
        events::emit_upgrade_approved(&env, &signer, approvals.len(), threshold);

        Ok(())
    }

    /// Approve a pending upgrade as an authorized signer.
    ///
    /// A pending upgrade must exist. If the caller has already approved,
    /// the call reverts with `AlreadyApproved`.
    ///
    /// # Errors
    /// * `NotUpgradeSigner` — Caller is not in the authorized signers list.
    /// * `NoPendingUpgrade` — No upgrade has been proposed.
    /// * `AlreadyApproved` — Caller has already approved this proposal.
    pub fn approve_upgrade(env: Env, signer: Address) -> Result<(), ContractError> {
        signer.require_auth();

        // Verify signer is authorized
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeSigners)
            .ok_or(ContractError::NotUpgradeSigner)?;
        if !signers.contains(signer.clone()) {
            return Err(ContractError::NotUpgradeSigner);
        }

        // Verify there is a pending upgrade
        if !env.storage().instance().has(&DataKey::PendingUpgradeWasm) {
            return Err(ContractError::NoPendingUpgrade);
        }

        // Verify caller hasn't already approved
        let mut approvals: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgradeApprovals)
            .unwrap_or(Vec::new(&env));

        if approvals.contains(signer.clone()) {
            return Err(ContractError::AlreadyApproved);
        }

        approvals.push_back(signer.clone());
        env.storage()
            .instance()
            .set(&DataKey::PendingUpgradeApprovals, &approvals);

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeThreshold)
            .unwrap_or(0u32);
        events::emit_upgrade_approved(&env, &signer, approvals.len(), threshold);

        Ok(())
    }

    /// Execute the pending upgrade once the approval quorum is met.
    ///
    /// # Errors
    /// * `NoPendingUpgrade` — No upgrade has been proposed.
    /// * `UpgradeThresholdNotMet` — Not enough approvals yet.
    pub fn execute_upgrade(env: Env) -> Result<(), ContractError> {
        // Check pending proposal exists
        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgradeWasm)
            .ok_or(ContractError::NoPendingUpgrade)?;

        let approvals: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgradeApprovals)
            .ok_or(ContractError::NoPendingUpgrade)?;

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeThreshold)
            .ok_or(ContractError::InvalidUpgradeConfig)?;

        if approvals.len() < threshold {
            return Err(ContractError::UpgradeThresholdNotMet);
        }

        // Clean up pending state BEFORE upgrade (after upgrade the contract
        // code is replaced and further cleanup may not run).
        env.storage().instance().remove(&DataKey::PendingUpgradeWasm);
        env.storage()
            .instance()
            .remove(&DataKey::PendingUpgradeApprovals);

        events::emit_upgrade_executed(&env);

        // Perform the actual WASM upgrade
        env.deployer().update_current_contract_wasm(wasm_hash);

        Ok(())
    }

    /// Cancel a pending upgrade. Only the contract admin may call this.
    ///
    /// # Errors
    /// * `NotAuthorized` — Caller is not the contract admin.
    /// * `NoPendingUpgrade` — No upgrade has been proposed.
    pub fn cancel_upgrade(env: Env, admin: Address) -> Result<(), ContractError> {
        crate::contracts::rbac::require_role(&env, &admin, crate::types::Role::Admin)?;

        if !env.storage().instance().has(&DataKey::PendingUpgradeWasm) {
            return Err(ContractError::NoPendingUpgrade);
        }

        env.storage()
            .instance()
            .remove(&DataKey::PendingUpgradeWasm);
        env.storage()
            .instance()
            .remove(&DataKey::PendingUpgradeApprovals);

        events::emit_upgrade_cancelled(&env);
        Ok(())
    }

    pub fn get_snapshot(env: Env) -> Snapshot {
        logic::get_snapshot(&env)
    }

    pub fn record_snapshot(env: Env) -> Result<(), ContractError> {
        logic::record_snapshot(&env)
    }

    pub fn get_snapshot_history(env: Env) -> soroban_sdk::Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::AllSnapshots)
            .unwrap_or(soroban_sdk::Vec::new(&env))
    }

    pub fn get_snapshot_at(env: Env, timestamp: u64) -> Result<Snapshot, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Snapshot(timestamp))
            .ok_or(ContractError::SnapshotNotFound)
    }

    pub fn get_withdrawal_timelock(env: Env, guardian: Address) -> Option<u64> {
        env.storage()
            .instance()
            .get(&DataKey::WithdrawalTimelock(guardian))
    }

    pub fn batch_execute(
        env: Env,
        calls: soroban_sdk::Vec<BatchCall>,
    ) -> Result<(), ContractError> {
        for call in calls.iter() {
            match call {
                BatchCall::RegisterTask(admin, task_id, min_votes_required) => {
                    Self::register_task(env.clone(), admin, task_id, min_votes_required)?
                }
                BatchCall::CancelTask(admin, task_id) => {
                    Self::cancel_task(env.clone(), admin, task_id)?
                }
                BatchCall::Vote(guardian, task_id) => Self::vote(env.clone(), guardian, task_id)?,
                BatchCall::AddGuardian(admin, guardian) => {
                    Self::add_guardian(env.clone(), admin, guardian)?
                }
                BatchCall::RemoveGuardian(admin, guardian) => {
                    Self::remove_guardian(env.clone(), admin, guardian)?
                }
                BatchCall::SetReputation(admin, guardian, score) => {
                    Self::set_reputation(env.clone(), admin, guardian, score)?
                }
                BatchCall::LockTokens(guardian, amount) => {
                    Self::lock_tokens(env.clone(), guardian, amount)?
                }
                BatchCall::RequestUnlock(guardian) => Self::request_unlock(env.clone(), guardian)?,
                BatchCall::UnlockTokens(guardian) => Self::unlock_tokens(env.clone(), guardian)?,
                BatchCall::ResignGuardian(guardian) => {
                    Self::resign_guardian(env.clone(), guardian)?
                }
                BatchCall::SetWeightThreshold(admin, threshold) => {
                    Self::set_weight_threshold(env.clone(), admin, threshold)?
                }
                BatchCall::SetVaultAddress(admin, vault) => {
                    Self::set_vault_address(env.clone(), admin, vault)
                }
                BatchCall::SetUpgradeSigners(admin, signers, threshold) => {
                    Self::set_upgrade_signers(env.clone(), admin, signers, threshold)?
                }
                BatchCall::ProposeUpgrade(signer, hash) => {
                    Self::propose_upgrade(env.clone(), signer, hash)?
                }
                BatchCall::ApproveUpgrade(signer) => {
                    Self::approve_upgrade(env.clone(), signer)?
                }
                BatchCall::ExecuteUpgrade(_signer) => {
                    Self::execute_upgrade(env.clone())?
                }
                BatchCall::CancelUpgrade(admin) => {
                    Self::cancel_upgrade(env.clone(), admin)?
                }
                BatchCall::StartRewardStream(admin, drips, contributor, task_id) => {
                    Self::start_reward_stream(env.clone(), admin, drips, contributor, task_id)?
                }
                BatchCall::TogglePause(admin) => Self::toggle_pause(env.clone(), admin)?,
                BatchCall::Pause(admin) => Self::pause(env.clone(), admin)?,
                BatchCall::Unpause(admin) => Self::unpause(env.clone(), admin)?,
                BatchCall::RecordFailure(_admin) => Self::record_failure(env.clone()),
                BatchCall::ResetCircuitBreaker(admin) => {
                    Self::reset_circuit_breaker(env.clone(), admin)
                }
            }
        }
        Ok(())
    }

    // ─── Role-based access control ──────────────────────────────────────

    /// Grant a role to a target address. Only callable by Admin role holders.
    ///
    /// # Errors
    /// * `NotAuthorized` — Caller does not hold the Admin role.
    pub fn grant_role(
        env: Env,
        caller: Address,
        target: Address,
        role: crate::types::Role,
    ) -> Result<(), ContractError> {
        crate::contracts::rbac::grant_role_internal(&env, &caller, &target, role)
    }

    /// Revoke a role from a target address. Only callable by Admin role holders.
    ///
    /// # Errors
    /// * `NotAuthorized` — Caller does not hold the Admin role.
    /// * `LastAdminRemovalBlocked` — Cannot revoke the last remaining Admin role.
    pub fn revoke_role(
        env: Env,
        caller: Address,
        target: Address,
        role: crate::types::Role,
    ) -> Result<(), ContractError> {
        crate::contracts::rbac::revoke_role_internal(&env, &caller, &target, role)
    }

    /// Check whether an address holds a specific role.
    pub fn has_role(env: Env, address: Address, role: crate::types::Role) -> bool {
        crate::contracts::rbac::has_role(&env, &address, role)
    }
}

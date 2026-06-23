//! Storage versioning and migration utilities.
//!
//! The contract records a `StorageVersion` (u32) in instance storage so that
//! future schema upgrades can detect the current format and apply the
//! appropriate transformation steps. The migration is designed to be
//! **idempotent** — calling it multiple times is a safe no-op once the
//! latest version is reached.

use crate::types::DataKey;
use soroban_sdk::Env;

/// The current on-chain storage schema version.
/// Increment this constant whenever the storage layout changes.
pub const CURRENT_VERSION: u32 = 1;

/// Returns the storage version currently recorded on-chain.
/// Returns 0 if no version has been set (i.e. a pre-versioning contract).
pub fn get_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StorageVersion)
        .unwrap_or(0)
}

/// Writes the given version into instance storage.
pub fn set_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&DataKey::StorageVersion, &version);
}

/// Returns `true` if the on-chain storage schema is older than the
/// current binary's `CURRENT_VERSION`, meaning a migration is required.
#[allow(dead_code)]
pub fn needs_migration(env: &Env) -> bool {
    get_version(env) < CURRENT_VERSION
}

/// Applies any pending storage migrations to bring the on-chain state
/// up to `CURRENT_VERSION`. The migration is **idempotent**: if the
/// storage is already at the latest version, this function returns
/// immediately without side-effects.
///
/// # Migration steps (v0 → v1)
///
/// This is the initial versioning deployment. No data transformation is
/// needed because the storage layout has not changed — we are simply
/// introducing the version-tracking infrastructure. The only action is
/// to record `CURRENT_VERSION` so that future upgrades can detect the
/// starting point.
pub fn migrate(env: &Env) {
    let current = get_version(env);
    if current >= CURRENT_VERSION {
        return; // already up to date
    }

    // ── v0 → v1 ──────────────────────────────────────────────────
    // No data transformation required.
    // Future migrations (v1 → v2, etc.) will add transformation steps here
    // with explicit version gates, e.g.:
    //
    // if current < 2 {
    //     // transform v1 keys to v2 format
    // }

    set_version(env, CURRENT_VERSION);
}

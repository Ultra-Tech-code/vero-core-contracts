use crate::events;
use crate::types::{ContractError, DataKey, Role};
use soroban_sdk::{Address, Env};

/// Check whether an address holds a specific role.
pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
    let key = DataKey::RoleAssignment(address.clone(), role);
    env.storage().instance().get(&key).unwrap_or(false)
}

/// Require that the caller holds a specific role, or revert with Unauthorized.
/// This is the "modifier" equivalent used at the start of privileged functions.
pub fn require_role(env: &Env, caller: &Address, role: Role) -> Result<(), ContractError> {
    caller.require_auth();
    if !has_role(env, caller, role) {
        return Err(ContractError::NotAuthorized);
    }
    Ok(())
}

/// Count how many addresses currently hold the specified role.
/// Used for admin lockout prevention.
///
/// Note: Soroban storage does not support reverse lookups, so this function
/// checks a known set of addresses that could potentially hold roles.
/// For the Admin role, we maintain a small set of known role holders.
pub fn count_role_holders(env: &Env, role: Role) -> u32 {
    // For now, we scan all guardians + the stored admin address as potential role holders.
    // This is acceptable because:
    // 1. Admin role should have very few holders (typically 1-5)
    // 2. This function is only called during revoke_role, not on hot paths

    let mut count = 0u32;

    // Check the original admin address
    if let Some(admin) = env.storage().instance().get::<_, Address>(&DataKey::Admin) {
        if has_role(env, &admin, role) {
            count += 1;
        }
    }

    // Check all guardians (they could have been granted roles)
    let all_guardians: soroban_sdk::Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::AllGuardians)
        .unwrap_or(soroban_sdk::Vec::new(env));

    for guardian in all_guardians.iter() {
        if has_role(env, &guardian, role) {
            count += 1;
        }
    }

    // For a production system with many role holders, consider maintaining
    // a separate AllRoleHolders(Role) index updated on grant/revoke.

    count
}

/// Grant a role to a target address. Only callable by Admin role holders.
pub fn grant_role_internal(
    env: &Env,
    caller: &Address,
    target: &Address,
    role: Role,
) -> Result<(), ContractError> {
    require_role(env, caller, Role::Admin)?;

    let key = DataKey::RoleAssignment(target.clone(), role);
    env.storage().instance().set(&key, &true);

    events::emit_role_granted(env, caller, target, role as u8);

    Ok(())
}

/// Revoke a role from a target address. Only callable by Admin role holders.
/// Prevents removal of the last Admin role holder to avoid lockout.
pub fn revoke_role_internal(
    env: &Env,
    caller: &Address,
    target: &Address,
    role: Role,
) -> Result<(), ContractError> {
    require_role(env, caller, Role::Admin)?;

    // Admin lockout prevention: if revoking Admin role, ensure at least one remains
    if role == Role::Admin {
        let admin_count = count_role_holders(env, Role::Admin);
        if admin_count <= 1 {
            return Err(ContractError::LastAdminRemovalBlocked);
        }
    }

    let key = DataKey::RoleAssignment(target.clone(), role);
    env.storage().instance().remove(&key);

    events::emit_role_revoked(env, caller, target, role as u8);

    Ok(())
}

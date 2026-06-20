use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Guardian(Address),
    Reputation(Address),
    WeightThreshold,
    Task(u64),
    Voted(u64, Address),
    TaskVoters(u64),
    Admin,
    DripsAddress,
    VaultAddress,
    RewardStream(u64),
    TokenAddress,
    LockThreshold,
    LockedBalance(Address),
    Lock,
    FailureCount,
    Paused,
    AllGuardians,
    AllTasks,
    AllVotes,
    AllRewardStreams,
    Snapshot(u64),
    AllSnapshots,
    ActiveTask(u64),
    ArchivedTask(u64),
    Initialized,
    WithdrawalTimelock(Address),
    /// Stores `Vec<Address>` of authorized multi-sig upgrade signers.
    UpgradeSigners,
    /// Stores `u32` — minimum number of approvals required.
    UpgradeThreshold,
    /// Stores `soroban_sdk::BytesN<32>` — the proposed WASM hash for upgrade.
    PendingUpgradeWasm,
    /// Stores `Vec<Address>` — signers who have approved the pending upgrade.
    PendingUpgradeApprovals,
}

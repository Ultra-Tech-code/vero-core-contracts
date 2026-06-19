use soroban_sdk::{contracterror, contracttype, Address, Map};

#[contracttype]
#[derive(Clone)]
pub struct WithdrawalRequest {
    pub id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub requested_at_ledger: u32,
    pub is_executed: bool,
    pub is_cancelled: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: u64,
    pub votes: u32,
    pub is_done: bool,
    pub resolved_at: u64,
    pub total_weight_accrued: u64,
    pub is_cancelled: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardStream {
    pub task_id: u64,
    pub contributor: Address,
    pub drips_contract: Address,
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Snapshot {
    pub timestamp: u64,
    pub paused: bool,
    pub failure_count: u32,
    pub weight_threshold: u64,
    pub admin: Option<Address>,
    pub vault_address: Option<Address>,
    pub drips_address: Option<Address>,
    pub guardians: Map<Address, bool>,
    pub reputations: Map<Address, u64>,
    pub tasks: Map<u64, Task>,
    pub votes: Map<(u64, Address), bool>,
    pub reward_streams: Map<u64, RewardStream>,
}

pub use crate::contracts::storage_layout::DataKey;

/// A single call within a `batch_execute` transaction.
#[contracttype]
#[derive(Clone)]
pub enum BatchCall {
    RegisterTask(Address, u64),
    CancelTask(Address, u64),
    Vote(Address, u64),
    AddGuardian(Address, Address),
    RemoveGuardian(Address, Address),
    SetReputation(Address, Address, u64),
    LockTokens(Address, i128),
    RequestUnlock(Address),
    UnlockTokens(Address),
    ResignGuardian(Address),
    SetWeightThreshold(Address, u64),
    SetVaultAddress(Address, Address),
    StartRewardStream(Address, Address, Address, u64),
    TogglePause(Address),
    Pause(Address),
    Unpause(Address),
    RecordFailure(Address),
    ResetCircuitBreaker(Address),
}

/// Every public write operation exposed by VeroContract.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    RegisterTask = 0,
    Vote = 1,
    AddGuardian = 2,
    SetReputation = 3,
    LockTokens = 4,
    UnlockTokens = 5,
    ResignGuardian = 6,
    SetWeightThreshold = 7,
    StartRewardStream = 8,
    TogglePause = 9,
    RecordFailure = 10,
    ResetCircuitBreaker = 11,
    UpgradeContract = 12,
    /// `record_snapshot` — records a state snapshot.
    RecordSnapshot = 13,
    /// `purge_task` — removes a terminal task from storage.
    PurgeTask = 14,
    /// `vote_batch` — vote on multiple tasks in one transaction.
    VoteBatch = 15,
}

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractError {
    NotAuthorized = 1,
    DuplicateVote = 2,
    TaskNotVerified = 3,
    StreamAlreadyActive = 4,
    DripsCallFailed = 5,
    AlreadyInitialized = 6,
    NotInitialized = 7,
    NoReputationScore = 8,
    ZeroWeightVote = 9,
    WeightOverflow = 10,
    InsufficientLockedBalance = 11,
    StillGuardian = 12,
    NotGuardian = 13,
    Locked = 14,
    ContractPaused = 15,
    EscrowUnavailable = 16,
    TaskCancelled = 17,
    InvalidAddress = 18,
    InvalidAmount = 19,
    InvalidConfig = 20,
    InvalidRange = 21,
    BatchTooLarge = 22,
    TaskNotFound = 23,
    TaskAlreadyArchived = 24,
    TaskNotStale = 25,
    SnapshotNotFound = 26,
    WithdrawalTimelockActive = 27,
    /// Task is still active (not done and not cancelled) and cannot be purged.
    TaskNotTerminal = 28,
    /// Guardian's reputation score is below the minimum threshold to vote.
    InsufficientReputation = 29,
}

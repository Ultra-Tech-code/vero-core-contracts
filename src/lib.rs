#![no_std]

mod contracts;

mod circuit_breaker;
#[cfg(any(feature = "verification", test))]
pub mod consensus;
mod drips;
pub mod events;
mod gas;
mod guardian;
mod reentrancy;
mod reputation;
mod storage;
mod task;
mod timelock;
mod types;
mod validation;
mod vault;



pub use contracts::proxy_entry::{VeroContract, VeroContractClient};
pub use drips::{get_reward_stream, start_drips_stream};
pub use guardian::{add_guardian, is_guardian, remove_guardian};
pub use task::{get_task, register_tasks};
pub use types::{BatchCall, Operation};

pub const DEFAULT_WEIGHT_THRESHOLD: u64 = 300;

pub type VeroCore = VeroContract;

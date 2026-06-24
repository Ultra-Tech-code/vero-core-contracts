//! # Vero Core Contracts
//!
//! Core contracts for the Vero protocol, providing reputation-weighted voting,
//! task registration and verification, token locking/unlocking for guardians,
//! reward stream management, and multi-sig contract upgrades.

#![no_std]
#![warn(missing_docs)]

mod contracts;
mod circuit_breaker;

/// Pure consensus logic.
#[cfg(any(feature = "verification", test))]
pub mod consensus;

mod drips;

/// Contract event emitters.
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
pub use types::{BatchCall, ContractError, Error, Operation};

/// The default cumulative weight threshold required to resolve a task.
pub const DEFAULT_WEIGHT_THRESHOLD: u64 = 300;

/// Type alias for the main `VeroContract` implementation.
pub type VeroCore = VeroContract;

//! Auto-update checking module

pub mod checker;
pub mod config;
pub mod scheduler;
pub mod types;

pub use checker::check_for_update;
pub use config::UpdateConfig;
pub use scheduler::UpdateScheduler;
pub use types::{UpdateCheckResult, UpdateInfo, UpdateStatus, VersionComparison};

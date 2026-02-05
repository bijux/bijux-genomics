//! Owner: bijux-bench
//! Statistics utilities for bench (robust stats + bootstrap + outliers).

pub mod bootstrap;
pub mod outliers;
pub mod robust;

pub use bootstrap::{bootstrap_ci, seed_from_ids};
pub use outliers::mad_outliers;
pub use robust::robust_stats;

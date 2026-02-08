//! Owner: bijux-dna-benchmark
//! Statistics utilities for bench (robust stats + bootstrap + outliers).

pub mod bootstrap;
pub mod outlier_detection;
pub mod robust_estimators;

pub use bootstrap::{bootstrap_ci, seed_from_ids};
pub use outlier_detection::mad_outliers;
pub use robust_estimators::robust_stats;

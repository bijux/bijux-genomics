//! Owner: bijux-dna-bench
//! Deterministic loaders for persisted benchmark run artifacts.

use std::path::PathBuf;

use anyhow::Result;

mod manifest_loader;
mod metrics_loader;
mod observations_loader;

pub use manifest_loader::load_manifest;
pub use metrics_loader::{load_metrics, load_metrics_map};
pub use observations_loader::load_observations;

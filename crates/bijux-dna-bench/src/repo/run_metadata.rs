//! Owner: bijux-dna-bench
//! Run metadata model for benchmark repositories.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RunMetadata {
    pub manifest_path: PathBuf,
    pub metrics_path: PathBuf,
}

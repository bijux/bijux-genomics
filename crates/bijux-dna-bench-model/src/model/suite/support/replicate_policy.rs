//! Owner: bijux-dna-bench-model
//! Replicate policy contract for benchmark suites.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicatePolicy {
    pub count: u32,
    pub warmup: u32,
    pub seeds: Vec<u64>,
}

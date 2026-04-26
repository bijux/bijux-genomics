mod graph_policy;

use anyhow::{bail, Result};
use bijux_dna_core::contract::RetryPolicy;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    pub step_timeout_s: Option<u64>,
    pub deterministic_scheduler: bool,
    pub retry_policy: Option<RetryPolicy>,
    pub max_parallelism: Option<usize>,
}

impl EngineConfig {
    /// # Errors
    /// Returns an error when the config asks for execution behavior this engine cannot honor.
    pub fn validate(&self) -> Result<()> {
        match self.max_parallelism {
            Some(0) => bail!("engine max_parallelism must be at least 1"),
            Some(value) if value > 1 => {
                bail!("engine max_parallelism > 1 is not supported by the sequential executor")
            }
            _ => Ok(()),
        }
    }
}

pub(crate) use graph_policy::apply_engine_config;

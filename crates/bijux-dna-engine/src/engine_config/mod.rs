mod graph_policy;

use bijux_dna_core::contract::RetryPolicy;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    pub step_timeout_s: Option<u64>,
    pub deterministic_scheduler: bool,
    pub retry_policy: Option<RetryPolicy>,
    pub max_parallelism: Option<usize>,
}

pub(crate) use graph_policy::apply_engine_config;

mod contract;

pub use contract::validate_execution_outputs;

use anyhow::{anyhow, Result};

use crate::core::types::{MetricSet, StageResult};

#[derive(Debug)]
pub struct ValidatedStageResult {
    pub result: StageResult,
    pub metrics: MetricSet,
}

pub fn validate_stage(result: StageResult, metrics: MetricSet) -> Result<ValidatedStageResult> {
    if crate::core::types::trace_enabled() {
        println!(
            "[engine][validator] stage={} tool={}",
            result.invocation.stage_id, result.invocation.tool_id
        );
    }
    if result.exit_code != 0 {
        return Err(anyhow!("stage failed: {}", result.invocation.stage_id));
    }
    Ok(ValidatedStageResult { result, metrics })
}

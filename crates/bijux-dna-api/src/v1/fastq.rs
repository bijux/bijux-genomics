//! FASTQ domain helpers for v1.

use anyhow::{anyhow, Result};
use bijux_dna_core::ids::StageId;

pub use bijux_dna_planner_fastq::stage_api as fastq_banks;
pub use bijux_dna_planner_fastq::stage_api::args as fastq_args;
pub use bijux_dna_planner_fastq::stage_api::banks as fastq_bank_ops;
pub use bijux_dna_planner_fastq::stage_api::*;

pub use crate::internal::public_bridge::handlers::fastq::*;

/// # Errors
/// Returns an error when the stage does not expose the requested benchmark cohort.
pub fn benchmark_tools_for_stage(stage_id: &str, scenario_id: Option<&str>) -> Result<Vec<String>> {
    let stage_id = StageId::new(stage_id.to_string());
    let tool_ids = if let Some(scenario_id) = scenario_id {
        toolset_for_stage_benchmark_scenario(&stage_id, scenario_id)
    } else {
        benchmark_default_scenario_toolset(&stage_id)
    };
    if tool_ids.is_empty() {
        return if let Some(scenario_id) = scenario_id {
            Err(anyhow!(
                "stage `{}` does not expose benchmark cohort `{scenario_id}`",
                stage_id.as_str()
            ))
        } else {
            Err(anyhow!(
                "stage `{}` does not expose a unique default benchmark cohort",
                stage_id.as_str()
            ))
        };
    }
    Ok(tool_ids
        .into_iter()
        .map(|tool_id| tool_id.to_string())
        .collect())
}

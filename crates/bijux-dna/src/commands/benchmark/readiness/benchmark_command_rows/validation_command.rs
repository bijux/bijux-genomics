use super::{local_stage_plans, plan_fastq_stage_tool_command};
use anyhow::{ensure, Result};
use bijux_dna_core::{
    id_catalog::FASTQ_VALIDATE_READS,
    ids::{StageId, ToolId},
};
use std::path::Path;

pub(crate) fn validate_benchmark_validation_command(
    root: &Path,
    stage: &str,
    tool_id: &str,
    argv: &[String],
) -> Result<()> {
    ensure!(stage == FASTQ_VALIDATE_READS, "command admission requires the validation stage");
    let tool = bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_execution_spec(
        root,
        &StageId::new(stage.to_string()),
        &ToolId::new(tool_id.to_string()),
    )?;
    let mut failures = Vec::new();
    for candidate in local_stage_plans(root, stage)? {
        let result = plan_fastq_stage_tool_command(root, stage, tool_id, &candidate)
            .and_then(|plan| bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::validate_command_projection(&tool, &plan, argv));
        match result {
            Ok(()) => return Ok(()),
            Err(error) => failures.push(error.to_string()),
        }
    }
    anyhow::bail!(
        "validation command matches no governed candidate for {tool_id}: {}",
        failures.join("; ")
    )
}

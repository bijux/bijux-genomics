mod hash;
mod seqkit;

pub use hash::hash_file_sha256;
pub use seqkit::{
    input_fastq_stats, length_histogram, output_fastq_stats, parse_fastqvalidator_count,
    SeqkitMetrics,
};

use anyhow::{Context, Result};
use std::path::Path;

use crate::core::types::{ExplainPlan, MetricSet, StageResult};

#[allow(dead_code)]
pub trait Observer {
    fn on_stage_start(&mut self, stage: &StageResult) -> Result<()>;
    fn on_stage_end(&mut self, stage: &StageResult) -> Result<()>;
    fn on_metric(&mut self, stage: &StageResult, metrics: &MetricSet) -> Result<()>;
}

pub fn observe_stage(result: &StageResult) -> Result<MetricSet> {
    if crate::core::types::trace_enabled() {
        println!(
            "[engine][observer] stage={} tool={}",
            result.invocation.stage_id, result.invocation.tool_id
        );
    }
    Ok(MetricSet {
        metrics_schema: "engine.metric.v1".to_string(),
        version: 1,
        metrics: serde_json::json!({}),
    })
}

/// Write `explain_plan.json` for a run.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_explain_plan(path: &Path, plan: &ExplainPlan) -> Result<()> {
    let payload = serde_json::to_vec_pretty(plan)?;
    std::fs::write(path, payload).context("write explain_plan.json")?;
    Ok(())
}

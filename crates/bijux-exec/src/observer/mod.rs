mod hash;
pub use hash::hash_file_sha256;

use anyhow::{Context, Result};
use std::path::Path;

use bijux_core::metrics::MetricSet;
use bijux_core::ExplainPlan;

#[derive(Debug, Clone)]
pub struct ToolInvocation {
    pub stage_id: String,
    pub tool_id: String,
}

#[derive(Debug, Clone)]
pub struct StageResult {
    pub invocation: ToolInvocation,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub outputs: Vec<std::path::PathBuf>,
}

#[allow(dead_code)]
pub trait Observer {
    fn on_stage_start(&mut self, stage: &StageResult) -> Result<()>;
    fn on_stage_end(&mut self, stage: &StageResult) -> Result<()>;
    fn on_metric(
        &mut self,
        stage: &StageResult,
        metrics: &MetricSet<serde_json::Value>,
    ) -> Result<()>;
}

pub fn observe_stage(result: &StageResult) -> Result<MetricSet<serde_json::Value>> {
    if std::env::var("BIJUX_TRACE_ENGINE").is_ok() {
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
    bijux_infra::atomic_write_bytes(path, &payload).context("write explain_plan.json")?;
    Ok(())
}

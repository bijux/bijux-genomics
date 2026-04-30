use anyhow::Result;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1, StagePluginOutputV1};

mod collected_metrics;
mod envelope;

pub(super) fn parse_stage_outputs(
    plan: &StagePlanV1,
    outputs: &[ArtifactRef],
) -> Result<StagePluginOutputV1> {
    let metrics = collected_metrics::collect_output_metrics(plan, outputs);
    let envelope = envelope::build_metrics_envelope(plan, metrics)?;
    Ok(StagePluginOutputV1 {
        metrics: envelope,
        artifacts: outputs.to_vec(),
        operating_mode: plan.operating_mode,
        report_parts: Vec::new(),
        warnings: Vec::new(),
        invariants: Vec::new(),
        verdict: None,
        event_hints: Vec::new(),
    })
}

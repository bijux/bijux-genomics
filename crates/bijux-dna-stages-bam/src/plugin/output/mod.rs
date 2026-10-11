use anyhow::Result;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1, StagePluginOutputV1};

mod collected_metrics;
mod contract;
mod envelope;

pub(super) fn parse_stage_outputs(
    plan: &StagePlanV1,
    outputs: &[ArtifactRef],
) -> Result<StagePluginOutputV1> {
    let (mut metrics, scientific) = collected_metrics::collect_output_metrics(plan, outputs);
    let contract = contract::evaluate_output_contract(plan, outputs);
    let mut invariants = contract.invariants.clone();
    invariants.extend(scientific.results);
    let verdict = contract::stage_verdict(
        plan,
        &invariants,
        &scientific.verdict.key_metrics,
        contract.materialized_count,
        contract.artifacts.len(),
    );
    metrics.stage_verdict = Some(verdict.clone().into());
    let envelope = envelope::build_metrics_envelope(plan, metrics)?;
    let report_parts = vec![contract::report_part(plan, &contract, &invariants, &verdict)];
    let warnings = contract::warnings(&contract);
    Ok(StagePluginOutputV1 {
        metrics: envelope,
        artifacts: contract.artifacts,
        operating_mode: plan.operating_mode,
        report_parts,
        warnings,
        invariants,
        verdict: Some(verdict),
        event_hints: Vec::new(),
    })
}

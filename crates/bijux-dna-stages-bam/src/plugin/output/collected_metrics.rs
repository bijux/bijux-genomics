use bijux_dna_domain_bam::metrics::BamMetricsV1;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::metrics::bam_metrics_from_dir;

pub(super) fn collect_output_metrics(plan: &StagePlanV1, outputs: &[ArtifactRef]) -> BamMetricsV1 {
    let out_dir = metrics_source_dir(plan, outputs);
    let mut metrics = bam_metrics_from_dir(&out_dir);
    let thresholds = bijux_dna_domain_bam::metrics::BamInvariantThresholds::default();
    let evaluation = bijux_dna_domain_bam::metrics::evaluate_bam_invariants(
        &plan.stage_id.0,
        &metrics,
        &thresholds,
    );
    metrics.stage_verdict = Some(evaluation.verdict.into());
    metrics
}

fn metrics_source_dir(plan: &StagePlanV1, outputs: &[ArtifactRef]) -> std::path::PathBuf {
    let mut candidates = vec![plan.out_dir.clone()];
    candidates.extend(
        outputs.iter().filter_map(|output| output.path.parent()).map(std::path::PathBuf::from),
    );
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .find(|candidate| candidate.is_dir())
        .unwrap_or_else(|| plan.out_dir.clone())
}

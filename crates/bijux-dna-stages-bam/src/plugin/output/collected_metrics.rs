use bijux_dna_domain_bam::metrics::BamMetricsV1;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::metrics::bam_metrics_from_dir;

pub(super) fn collect_output_metrics(plan: &StagePlanV1, outputs: &[ArtifactRef]) -> BamMetricsV1 {
    let out_dir = outputs
        .first()
        .and_then(|output| output.path.parent())
        .map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);
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

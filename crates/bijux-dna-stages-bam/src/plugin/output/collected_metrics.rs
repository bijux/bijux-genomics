use bijux_dna_domain_bam::metrics::{BamInvariantEvaluation, BamMetricsV1};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::metrics::{bam_metrics_from_dir, bam_metrics_from_files};

pub(super) fn collect_output_metrics(
    plan: &StagePlanV1,
    outputs: &[ArtifactRef],
) -> (BamMetricsV1, BamInvariantEvaluation) {
    let metrics = if outputs.is_empty() {
        bam_metrics_from_dir(&plan.out_dir)
    } else {
        let paths = outputs.iter().map(|output| output.path.clone()).collect::<Vec<_>>();
        bam_metrics_from_files(&paths)
    };
    let thresholds = bijux_dna_domain_bam::metrics::BamInvariantThresholds::default();
    let evaluation = bijux_dna_domain_bam::metrics::evaluate_bam_invariants(
        &plan.stage_id.0,
        &metrics,
        &thresholds,
    );
    (metrics, evaluation)
}

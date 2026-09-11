use super::checks::check_authenticity_report_contract_ready;
use super::checks::check_composition_ready;
use super::checks::check_evidence_consumption_ready;
use super::checks::check_normalized_metrics_ready;
use super::checks::check_summary_ready;
use super::declarations::REQUIRED_EVIDENCE_METRIC_IDS;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamAuthenticityCompleteRow;
use super::observations::AuthenticityStageProof;
use super::projection::project;
use super::projection::RowProjection;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn build_bam_authenticity_complete_row(
    repo_root: &Path,
    authenticity_row: BamDamageAuthenticityReadyRow,
    proof: &AuthenticityStageProof,
) -> BamAuthenticityCompleteRow {
    let inherited_artifact_set =
        authenticity_row.local_smoke_artifact_paths.iter().cloned().collect::<BTreeSet<_>>();
    let required_artifact_paths = [
        proof.local_smoke_report.authenticity_report.as_str(),
        proof.local_smoke_report.authenticity_summary.as_str(),
        proof.local_smoke_report.authenticity_composite.as_str(),
        proof.local_smoke_report.advisory_boundary.as_str(),
        proof.local_smoke_report.stage_metrics.as_str(),
        proof.local_smoke_report.damage_unified_metrics.as_str(),
        proof.local_smoke_report.contamination_summary.as_str(),
        proof.local_smoke_report.complexity_summary.as_str(),
        proof.local_smoke_report.coverage_regime.as_str(),
        proof.local_smoke_report.mapping_summary.as_str(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    let authenticity_report_contract_ready =
        check_authenticity_report_contract_ready(repo_root, &authenticity_row, proof);

    let summary_ready = check_summary_ready(proof);

    let composition_ready = check_composition_ready(proof);

    let expected_consumed_metric_ids =
        REQUIRED_EVIDENCE_METRIC_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let normalized_metrics_ready =
        check_normalized_metrics_ready(proof, &expected_consumed_metric_ids);

    let evidence_consumption_ready = check_evidence_consumption_ready(
        repo_root,
        proof,
        &inherited_artifact_set,
        &required_artifact_paths,
        &expected_consumed_metric_ids,
    );

    let checks = SurfaceChecks {
        authenticity_report_contract_ready,
        summary_ready,
        composition_ready,
        normalized_metrics_ready,
        evidence_consumption_ready,
    };
    let (missing_surfaces, coverage_status, reason) = explain(&authenticity_row, checks);

    project(RowProjection {
        authenticity_row,
        proof,
        authenticity_report_contract_ready,
        summary_ready,
        composition_ready,
        normalized_metrics_ready,
        evidence_consumption_ready,
        missing_surfaces,
        coverage_status,
        reason,
    })
}

use super::checks::check_damage_metric_consistency_ready;
use super::checks::check_normalized_metrics_ready;
use super::checks::check_parser_output_contract_ready;
use super::checks::check_summary_ready;
use super::checks::check_unified_metrics_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamDamageCompleteRow;
use super::observations::DamageStageProof;
use super::projection::project;
use super::projection::RowProjection;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn build_bam_damage_complete_row(
    repo_root: &Path,
    damage_row: BamDamageAuthenticityReadyRow,
    proof: &DamageStageProof,
) -> BamDamageCompleteRow {
    let parsed_tool_ids = proof
        .parser_output
        .parsed_tools
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect::<Vec<_>>();
    let parsed_tool_id_set = parsed_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    let local_tool_set =
        proof.local_smoke_report.tools_seen.iter().cloned().collect::<BTreeSet<_>>();
    let inherited_artifact_set =
        damage_row.local_smoke_artifact_paths.iter().cloned().collect::<BTreeSet<_>>();
    let required_artifact_paths = [
        proof.local_smoke_report.damage_report.as_str(),
        proof.local_smoke_report.terminal_position_metrics.as_str(),
        proof.local_smoke_report.parser_output.as_str(),
        proof.local_smoke_report.advisory_boundary.as_str(),
        proof.local_smoke_report.udg_regime.as_str(),
        proof.local_smoke_report.stage_metrics.as_str(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    let summary_ready = check_summary_ready(proof);
    let unified_metrics_ready = check_unified_metrics_ready(proof, &local_tool_set);
    let parser_output_contract_ready =
        check_parser_output_contract_ready(proof, &parsed_tool_id_set, &local_tool_set);
    let normalized_metrics_ready = check_normalized_metrics_ready(proof, &local_tool_set);
    let damage_metric_consistency_ready = check_damage_metric_consistency_ready(
        repo_root,
        &damage_row,
        proof,
        &local_tool_set,
        &inherited_artifact_set,
        &required_artifact_paths,
    );

    let checks = SurfaceChecks {
        summary_ready,
        unified_metrics_ready,
        parser_output_contract_ready,
        normalized_metrics_ready,
        damage_metric_consistency_ready,
    };
    let (missing_surfaces, coverage_status, reason) = explain(&damage_row, checks);

    project(RowProjection {
        damage_row,
        proof,
        parsed_tool_ids,
        summary_ready,
        unified_metrics_ready,
        parser_output_contract_ready,
        normalized_metrics_ready,
        damage_metric_consistency_ready,
        missing_surfaces,
        coverage_status,
        reason,
    })
}

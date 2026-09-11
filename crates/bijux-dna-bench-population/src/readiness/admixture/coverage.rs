use super::super::evidence::SurfaceAssessment;
use super::fields::{
    contains_artifact_id, expected_result_id, required_metric_names, retained_result_id,
};
use super::{
    ANCESTRY_SUM_TOLERANCE, COVERAGE_STATUS_COMPLETE, COVERAGE_STATUS_INCOMPLETE,
    GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID, REQUIRED_CLUSTER_HEADERS, REQUIRED_POPULATION_COUNT,
    REQUIRED_SAMPLE_COUNT, REQUIRED_SAMPLE_IDS, REQUIRED_SELECTED_K, REQUIRED_STATUS,
};
use crate::admixture::LocalVcfAdmixtureSmokeReport;
use bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageStatus;
use bijux_dna_bench_vcf_evidence::model::parsers::VcfParserFixtureCoverageStatus;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use std::collections::BTreeSet;
pub(super) fn assess(
    binding: &VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfAdmixtureSmokeReport>,
) -> SurfaceAssessment {
    let result_id = binding
        .expected_row
        .as_ref()
        .map_or_else(|| retained_result_id(binding), expected_result_id);
    let required_metric_names = required_metric_names();
    let mut missing_surfaces = Vec::new();

    let retained_scope_active = binding.retained_row.scope_state == "active";
    if !retained_scope_active {
        missing_surfaces.push("retained_vcf_active_scope".to_string());
    }

    let all_domain_active_row_present = binding.active_row.is_some();
    if !all_domain_active_row_present {
        missing_surfaces.push("all_domain_active_row".to_string());
    }

    let command_ready = command_ready(binding);
    if !command_ready {
        missing_surfaces.push("vcf_rendered_commands".to_string());
    }

    let output_ready = output_ready(binding);
    if !output_ready {
        missing_surfaces.push("vcf_adapter_output_coverage".to_string());
    }

    let parser_ready = parser_ready(binding);
    if !parser_ready {
        missing_surfaces.push("vcf_parser_fixture_coverage".to_string());
    }

    let expected_result_ready = expected_result_ready(binding);
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = report_ready(binding);
    if !report_ready {
        missing_surfaces.push("vcf_report_map".to_string());
    }

    let smoke_ready = smoke_ready(smoke_report);
    if !smoke_ready {
        missing_surfaces.push("local_vcf_admixture_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "`{}` / `{}` stays active with backend-specific adapter, parser, and report contracts plus the canonical `{GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID}` stage-contract smoke; the smoke is not backend execution evidence",
            binding.retained_row.stage_id, binding.retained_row.tool_id
        )
    } else {
        format!(
            "`{}` / `{}` is missing readiness surfaces: {}",
            binding.retained_row.stage_id,
            binding.retained_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    SurfaceAssessment {
        result_id,
        required_metric_names,
        missing_surfaces,
        all_domain_active_row_present,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        smoke_ready,
        coverage_status,
        reason,
    }
}
fn command_ready(binding: &VcfStageReadinessBinding) -> bool {
    binding.command_row.as_ref().is_some_and(|row| {
        row.benchmark_status == "benchmark_ready"
            && !row.command_steps.is_empty()
            && !row.script_commands.is_empty()
            && row.command_steps.iter().all(|step| {
                step.argv.first().is_some_and(|value| !value.trim().is_empty())
                    && !step.command.trim().is_empty()
            })
    })
}
fn output_ready(binding: &VcfStageReadinessBinding) -> bool {
    binding.output_row.as_ref().is_some_and(|row| {
        row.status == VcfAdapterOutputCoverageStatus::Complete
            && row.benchmark_status == "benchmark_ready"
            && contains_artifact_id(&row.normalized_metrics, "admixture_report")
            && !row.raw_outputs.is_empty()
            && !row.manifest.trim().is_empty()
    })
}
fn parser_ready(binding: &VcfStageReadinessBinding) -> bool {
    binding.parser_row.as_ref().is_some_and(|row| {
        row.coverage_status == VcfParserFixtureCoverageStatus::Covered
            && !row.parser_id.trim().is_empty()
            && !row.parser_fixture_root_path.trim().is_empty()
            && !row.expected_normalized_path.trim().is_empty()
            && row.raw_fixture_count > 0
            && !row.schema_id.trim().is_empty()
    })
}
fn expected_result_ready(binding: &VcfStageReadinessBinding) -> bool {
    let required_metric_names = required_metric_names();
    binding.expected_row.as_ref().is_some_and(|row| {
        row.report_section == "population_structure"
            && row.expected_outputs.iter().any(|value| value == "admixture_report")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    })
}
fn report_ready(binding: &VcfStageReadinessBinding) -> bool {
    let required_metric_names = required_metric_names();
    binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    })
}
fn smoke_ready(smoke_report: Option<&LocalVcfAdmixtureSmokeReport>) -> bool {
    smoke_report.is_some_and(|report| {
        report.tool_id == GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID
            && !report.output_root.trim().is_empty()
            && !report.input_vcf_path.trim().is_empty()
            && !report.sample_metadata_path.trim().is_empty()
            && !report.population_metadata_path.trim().is_empty()
            && !report.population_labels_manifest_path.trim().is_empty()
            && !report.admixture_tsv_path.trim().is_empty()
            && !report.admixture_json_path.trim().is_empty()
            && !report.source_q_matrix_path.trim().is_empty()
            && !report.source_k_selection_path.trim().is_empty()
            && !report.source_logs_path.trim().is_empty()
            && !report.stage_result_manifest_path.trim().is_empty()
            && !report.execution_mode.trim().is_empty()
            && report.status == REQUIRED_STATUS
            && report.selected_k == REQUIRED_SELECTED_K
            && report.sample_count == REQUIRED_SAMPLE_COUNT
            && report.population_count == REQUIRED_POPULATION_COUNT
            && report.cluster_headers
                == REQUIRED_CLUSTER_HEADERS
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>()
            && report.rows.len() == REQUIRED_SAMPLE_IDS.len()
            && report.rows.iter().all(|row| {
                row.k == REQUIRED_SELECTED_K
                    && row.status == REQUIRED_STATUS
                    && !row.sample_id.trim().is_empty()
                    && !row.population_id.trim().is_empty()
                    && !row.population_label.trim().is_empty()
                    && row.clusters.len() == REQUIRED_SELECTED_K
                    && (row.clusters.values().sum::<f64>() - 1.0).abs() <= ANCESTRY_SUM_TOLERANCE
            })
            && report.rows.iter().map(|row| row.sample_id.as_str()).collect::<BTreeSet<_>>()
                == REQUIRED_SAMPLE_IDS.into_iter().collect::<BTreeSet<_>>()
    })
}

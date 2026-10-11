use super::super::evidence::SurfaceAssessment;
use super::fields::{
    contains_artifact_id, expected_result_id, required_filter_ids, required_metric_names,
    retained_result_id,
};
use super::{
    COVERAGE_STATUS_COMPLETE, COVERAGE_STATUS_INCOMPLETE, REQUIRED_DEPTH_THRESHOLD,
    REQUIRED_MISSINGNESS_THRESHOLD, REQUIRED_QUALITY_THRESHOLD,
};
use crate::filter::LocalVcfFilterSmokeReport;
use bijux_dna_bench_vcf_evidence::{
    model::{outputs::VcfAdapterOutputCoverageStatus, parsers::VcfParserFixtureCoverageStatus},
    snapshot::VcfStageReadinessBinding,
};
pub(super) fn assess(
    binding: &VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfFilterSmokeReport>,
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
        missing_surfaces.push("local_vcf_filter_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "active retained VCF filter caller `{}` keeps active scope, command, output, parser, expected-result, report, and threshold-normalized smoke proof for `vcf.filter`",
            binding.retained_row.tool_id
        )
    } else {
        format!(
            "active retained VCF filter caller `{}` is missing: {}",
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
            && contains_artifact_id(&row.raw_outputs, "filtered_vcf")
            && !row.normalized_metrics.is_empty()
            && !row.manifest.trim().is_empty()
            && contains_artifact_id(&row.index_outputs, "filtered_vcf_tbi")
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
        row.report_section == "quality_control"
            && row.expected_outputs.iter().any(|value| value == "filtered_vcf")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    })
}
fn report_ready(binding: &VcfStageReadinessBinding) -> bool {
    let required_metric_names = required_metric_names();
    binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "quality_control"
            && row.summary_table == "quality_control_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    })
}
fn smoke_ready(smoke_report: Option<&LocalVcfFilterSmokeReport>) -> bool {
    let required_filter_ids = required_filter_ids();
    smoke_report.is_some_and(|report| {
        report.parseable
            && report.gt_present
            && !report.gl_present
            && !report.input_vcf_path.trim().is_empty()
            && !report.output_vcf_path.trim().is_empty()
            && !report.output_tbi_path.trim().is_empty()
            && !report.metrics_path.trim().is_empty()
            && !report.filter_breakdown_path.trim().is_empty()
            && !report.filter_breakdown_tsv_path.trim().is_empty()
            && !report.filter_explain_path.trim().is_empty()
            && !report.stage_result_manifest_path.trim().is_empty()
            && report.input_variants > 0
            && report.pass_variants > 0
            && report.failed_variants > 0
            && report.input_variants == report.pass_variants + report.failed_variants
            && report.filter_ids == required_filter_ids
            && report.depth_threshold == REQUIRED_DEPTH_THRESHOLD
            && report.quality_threshold == REQUIRED_QUALITY_THRESHOLD
            && report.missingness_threshold == REQUIRED_MISSINGNESS_THRESHOLD
            && report.sample_count == 1
    })
}

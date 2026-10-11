use super::super::evidence::SurfaceAssessment;
use super::fields::{
    contains_artifact_id, expected_result_id, required_metric_names, retained_result_id,
};
use super::{COVERAGE_STATUS_COMPLETE, COVERAGE_STATUS_INCOMPLETE, REQUIRED_DAMAGE_CONTEXT_RULE};
use crate::damage_filter::LocalVcfDamageFilterSmokeReport;
use bijux_dna_bench_vcf_evidence::{
    model::{outputs::VcfAdapterOutputCoverageStatus, parsers::VcfParserFixtureCoverageStatus},
    snapshot::VcfStageReadinessBinding,
};
pub(super) fn assess(
    binding: &VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfDamageFilterSmokeReport>,
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
        missing_surfaces.push("local_vcf_damage_filter_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "active retained VCF damage filter `{}` keeps active scope, command, output, parser, expected-result, report, and damage-context smoke proof for `vcf.damage_filter`",
            binding.retained_row.tool_id
        )
    } else {
        format!(
            "active retained VCF damage filter `{}` is missing: {}",
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
            && contains_artifact_id(&row.raw_outputs, "damage_filtered_vcf")
            && !row.normalized_metrics.is_empty()
            && !row.manifest.trim().is_empty()
            && contains_artifact_id(&row.index_outputs, "damage_filtered_vcf_tbi")
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
        row.report_section == "damage_aware_filtering"
            && row.expected_outputs.iter().any(|value| value == "damage_filtered_vcf")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    })
}
fn report_ready(binding: &VcfStageReadinessBinding) -> bool {
    let required_metric_names = required_metric_names();
    binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "damage_aware_filtering"
            && row.summary_table == "damage_filtering_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    })
}
fn smoke_ready(smoke_report: Option<&LocalVcfDamageFilterSmokeReport>) -> bool {
    smoke_report.is_some_and(|report| {
        report.parseable
            && report.gt_present
            && report.gl_present
            && !report.input_vcf_path.trim().is_empty()
            && !report.output_vcf_path.trim().is_empty()
            && !report.output_tbi_path.trim().is_empty()
            && !report.metrics_path.trim().is_empty()
            && !report.summary_path.trim().is_empty()
            && !report.counts_path.trim().is_empty()
            && !report.warnings_path.trim().is_empty()
            && !report.damage_manifest_path.trim().is_empty()
            && !report.stage_result_manifest_path.trim().is_empty()
            && report.input_variants > 0
            && report.removed_variants > 0
            && report.retained_variants > 0
            && report.input_variants == report.removed_variants + report.retained_variants
            && report.low_quality_filtered_variants > 0
            && report.damage_ratio_filtered_variants > 0
            && report.terminal_damage_filtered_variants > 0
            && report.damage_context_rule == REQUIRED_DAMAGE_CONTEXT_RULE
            && report.terminal_context_count > 0
            && report.sample_count > 0
    })
}

use super::super::evidence::SurfaceAssessment;
use super::fields::{
    contains_artifact_id, expected_result_id, required_metric_names, retained_result_id,
};
use super::science::pca_smoke_matches_governed_contract;
use super::{COVERAGE_STATUS_COMPLETE, COVERAGE_STATUS_INCOMPLETE};
use crate::pca::LocalVcfPcaSmokeReport;
use bijux_dna_bench_vcf_evidence::model::parsers::VcfParserFixtureCoverageStatus;
use bijux_dna_bench_vcf_evidence::model::pca::PcaAdapterOutputProof;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
pub(super) fn assess(
    binding: &VcfStageReadinessBinding,
    output_proof: &PcaAdapterOutputProof,
    smoke_report: Option<&LocalVcfPcaSmokeReport>,
) -> SurfaceAssessment {
    let result_id = binding
        .expected_row
        .as_ref()
        .map_or_else(|| retained_result_id(&binding.retained_row), expected_result_id);
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

    let output_ready = output_ready(binding, output_proof);
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
        missing_surfaces.push("local_vcf_pca_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "active retained VCF PCA caller `{}` keeps active scope, command, output, parser, expected-result, report, and sample-complete PCA smoke proof for `vcf.pca`",
            binding.retained_row.tool_id
        )
    } else {
        format!(
            "active retained VCF PCA caller `{}` is missing: {}",
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
fn output_ready(binding: &VcfStageReadinessBinding, output_proof: &PcaAdapterOutputProof) -> bool {
    output_proof.benchmark_status == "benchmark_ready"
        && contains_artifact_id(&output_proof.normalized_metrics_outputs, "pca_report")
        && !output_proof.manifest_output.trim().is_empty()
        && match binding.retained_row.tool_id.as_str() {
            "eigensoft" => {
                contains_artifact_id(&output_proof.raw_outputs, "eigensoft_geno")
                    && contains_artifact_id(&output_proof.raw_outputs, "eigensoft_snp")
                    && contains_artifact_id(&output_proof.raw_outputs, "eigensoft_ind")
                    && contains_artifact_id(&output_proof.raw_outputs, "smartpca_eigenvec")
                    && contains_artifact_id(&output_proof.raw_outputs, "smartpca_eigenval")
                    && contains_artifact_id(&output_proof.raw_outputs, "smartpca_log")
            }
            "plink2" => {
                contains_artifact_id(&output_proof.raw_outputs, "pca_eigenvec")
                    && contains_artifact_id(&output_proof.raw_outputs, "pca_eigenval")
                    && contains_artifact_id(&output_proof.raw_outputs, "plink2_log")
            }
            _ => false,
        }
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
            && row.expected_outputs.iter().any(|value| value == "pca_report")
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
fn smoke_ready(smoke_report: Option<&LocalVcfPcaSmokeReport>) -> bool {
    smoke_report.is_some_and(pca_smoke_matches_governed_contract)
}

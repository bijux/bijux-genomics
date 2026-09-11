use super::contracts::command_is_admitted;
use super::contracts::required_validated_reads_manifest_fields;
use super::contracts::required_validation_report_fields;
use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::declarations::COVERAGE_STATUS_INCOMPLETE;
use super::declarations::REQUIRED_SHARED_METRIC_FIELDS;
use super::evaluation_inputs::RowAssessment;
use super::evaluation_inputs::RowInputs;
use super::models::FastqValidateReadsReadyRow;
use super::projection::project;
use crate::proofs::inputs::observations::ComparableContract as FastqComparableMetricContractStatus;
use crate::proofs::inputs::ValidationContext;
use crate::readiness::output_contract::FastqAdapterOutputContractStatus;
use crate::readiness::parsers::FastqParserCoverageKind;
pub(super) fn build_fastq_validate_reads_ready_row(
    context: &ValidationContext<'_>,
    inputs: RowInputs<'_>,
) -> FastqValidateReadsReadyRow {
    let RowInputs {
        active_row,
        output_row,
        parser_row,
        expected_row,
        command_row,
        comparable_row,
        report_map_row,
        smoke,
    } = &inputs;

    let validation_report_fields = required_validation_report_fields();
    let validated_reads_manifest_fields = required_validated_reads_manifest_fields();
    let mut missing_surfaces = Vec::new();

    let command_ready = command_is_admitted(context, command_row, &active_row.tool_id);
    if !command_ready {
        missing_surfaces.push("rendered_commands".to_string());
    }

    let output_ready = output_row.output_contract_status
        == FastqAdapterOutputContractStatus::Complete
        && output_row.normalized_metrics_output_id.as_deref() == Some("validation_report")
        && output_row
            .raw_output_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id == "validated_reads_manifest");
    if !output_ready {
        missing_surfaces.push("adapter_output_contract".to_string());
    }

    let parser_ready = parser_row.parser_coverage == FastqParserCoverageKind::Covered
        && parser_row.parser_status == "comparable";
    if !parser_ready {
        missing_surfaces.push("parser_coverage".to_string());
    }

    let comparable_metrics_ready = comparable_row.comparison_contract_status
        == FastqComparableMetricContractStatus::Declared
        && REQUIRED_SHARED_METRIC_FIELDS
            .iter()
            .all(|field| comparable_row.shared_metric_fields.iter().any(|value| value == field));
    if !comparable_metrics_ready {
        missing_surfaces.push("comparable_metrics".to_string());
    }

    let expected_result_ready = expected_row.normalized_metrics_output_id.as_deref()
        == Some("validation_report")
        && expected_row
            .expected_output_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id == "validation_report")
        && expected_row
            .expected_output_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id == "validated_reads_manifest");
    if !expected_result_ready {
        missing_surfaces.push("expected_results".to_string());
    }

    let report_ready = report_map_row.report_section_id == "input_readiness"
        && report_map_row.summary_table_id == "validation_intake";
    if !report_ready {
        missing_surfaces.push("report_map".to_string());
    }

    if !smoke.parseable {
        missing_surfaces.push("smoke_parse".to_string());
    }
    if !smoke.sample_id_normalized {
        missing_surfaces.push("sample_id_normalization".to_string());
    }
    if !smoke.layout_normalized {
        missing_surfaces.push("layout_normalization".to_string());
    }
    if !smoke.read_count_normalized {
        missing_surfaces.push("read_count_normalization".to_string());
    }
    if !smoke.validation_status_normalized {
        missing_surfaces.push("validation_status_normalization".to_string());
    }
    if !smoke.failure_reason_normalized {
        missing_surfaces.push("failure_reason_normalization".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };

    project(
        context,
        inputs,
        RowAssessment {
            coverage_status,
            validation_report_fields,
            validated_reads_manifest_fields,
            command_ready,
            output_ready,
            parser_ready,
            comparable_metrics_ready,
            expected_result_ready,
            report_ready,
            missing_surfaces,
        },
    )
}

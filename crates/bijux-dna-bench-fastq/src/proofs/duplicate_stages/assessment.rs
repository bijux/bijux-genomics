use super::contracts::has_fixture_corpus;
use super::contracts::has_governed_support;
use super::contracts::has_runnable_adapter;
use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::declarations::COVERAGE_STATUS_INCOMPLETE;
use super::evaluation_inputs::RowAssessment;
use super::evaluation_inputs::RowInputs;
use super::models::FastqDuplicateStagesReadyRow;
use super::projection::project;
use crate::proofs::inputs::SchemaContext;
use crate::readiness::adapters::FastqAdapterCoverageKind;
use crate::readiness::adapters::FastqBenchmarkStatus;
use crate::readiness::adapters::FastqReadinessGapKind;
use crate::readiness::output_contract::FastqAdapterOutputContractStatus;
use crate::readiness::parsers::FastqParserCoverageKind;
pub(super) fn build_fastq_duplicate_stages_ready_row(
    context: &SchemaContext<'_>,
    inputs: RowInputs<'_>,
) -> FastqDuplicateStagesReadyRow {
    let RowInputs {
        active_row,
        output_row,
        parser_row,
        expected_row,
        command_row,
        report_map_row,
        schema_contract,
        spec,
        ..
    } = &inputs;

    let active_scope_ready = active_row.benchmark_status == "benchmark_ready"
        && has_governed_support(&active_row.support_status)
        && has_runnable_adapter(&active_row.adapter_status)
        && has_fixture_corpus(&active_row.corpus_status);
    let command_ready = command_row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
        && command_row.adapter_coverage == FastqAdapterCoverageKind::Covered
        && command_row.readiness_gap == FastqReadinessGapKind::None;
    let output_ready = output_row.output_contract_status
        == FastqAdapterOutputContractStatus::Complete
        && output_row.missing_declarations.is_empty()
        && output_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && spec.required_output_ids.iter().all(|output_id| {
            output_row.raw_output_artifact_ids.iter().any(|candidate| candidate == output_id)
                || output_row.stage_output_ids.iter().any(|candidate| candidate == output_id)
        });
    let parser_ready = parser_row.parser_coverage == FastqParserCoverageKind::Covered;
    let expected_result_ready = expected_row.sample_scope == "sample-set"
        && expected_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && spec.required_output_ids.iter().all(|output_id| {
            expected_row.expected_output_artifact_ids.iter().any(|candidate| candidate == output_id)
        })
        && !expected_row.stage_result_manifest_path.is_empty()
        && !expected_row.stdout_path.is_empty()
        && !expected_row.stderr_path.is_empty();
    let report_ready = report_map_row.report_section_id == spec.required_report_section_id
        && report_map_row.summary_table_id == spec.required_summary_table_id;
    let schema_ready = spec
        .required_metric_fields
        .iter()
        .all(|field| schema_contract.required_fields.iter().any(|candidate| candidate == field));

    let mut missing_surfaces = Vec::new();
    if !active_scope_ready {
        missing_surfaces.push("active_scope".to_string());
    }
    if !command_ready {
        missing_surfaces.push("command".to_string());
    }
    if !output_ready {
        missing_surfaces.push("output".to_string());
    }
    if !parser_ready {
        missing_surfaces.push("parser".to_string());
    }
    if !expected_result_ready {
        missing_surfaces.push("expected_result".to_string());
    }
    if !report_ready {
        missing_surfaces.push("report".to_string());
    }
    if !schema_ready {
        missing_surfaces.push("schema".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "duplicate binding `{}` / `{}` keeps active scope, command, output, parser, expected-result, report, and schema proof for `{}`",
            active_row.stage_id, active_row.tool_id, active_row.stage_id
        )
    } else {
        format!(
            "duplicate binding `{}` / `{}` is missing readiness proof for {}",
            active_row.stage_id,
            active_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    project(
        context,
        inputs,
        RowAssessment {
            active_scope_ready,
            command_ready,
            output_ready,
            parser_ready,
            expected_result_ready,
            report_ready,
            schema_ready,
            coverage_status,
            missing_surfaces,
            reason,
        },
    )
}

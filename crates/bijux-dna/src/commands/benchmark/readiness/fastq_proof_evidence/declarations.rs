use super::super::{
    expected_benchmark_results::{
        render_expected_benchmark_results, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
    },
    fastq_active_stage_tool_matrix::{
        render_fastq_active_stage_tool_matrix, DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH,
    },
    fastq_adapter_output_contract::{
        render_fastq_adapter_output_contract, DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH,
    },
    fastq_parser_coverage::{render_fastq_parser_coverage, DEFAULT_FASTQ_PARSER_COVERAGE_PATH},
    fastq_report_map::{render_fastq_report_map, DEFAULT_FASTQ_REPORT_MAP_PATH},
};
use anyhow::Result;
use bijux_dna_bench_fastq::{
    proofs::inputs::{observations::ExpectedResultObservation, ports::DeclarationEvidence},
    readiness::{
        active::FastqActiveStageToolMatrixRow, output_contract::FastqAdapterOutputContractRow,
        parsers::FastqParserCoverageRow, report_map::FastqReportMapRow,
    },
};
use std::path::Path;
pub(super) struct DeclarationAdapter;
impl DeclarationEvidence for DeclarationAdapter {
    fn active(&self, root: &Path) -> Result<Vec<FastqActiveStageToolMatrixRow>> {
        Ok(render_fastq_active_stage_tool_matrix(
            root,
            DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH.into(),
        )?
        .rows)
    }
    fn outputs(&self, root: &Path) -> Result<Vec<FastqAdapterOutputContractRow>> {
        Ok(render_fastq_adapter_output_contract(
            root,
            DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH.into(),
        )?
        .rows)
    }
    fn parsers(&self, root: &Path) -> Result<Vec<FastqParserCoverageRow>> {
        Ok(render_fastq_parser_coverage(root, DEFAULT_FASTQ_PARSER_COVERAGE_PATH.into())?.rows)
    }
    fn expected(&self, root: &Path) -> Result<Vec<ExpectedResultObservation>> {
        Ok(render_expected_benchmark_results(root, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.into())?
            .rows
            .into_iter()
            .map(|row| ExpectedResultObservation {
                domain: row.domain,
                result_row_id: row.result_row_id,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                sample_scope: row.sample_scope,
                normalized_metrics_output_id: row.normalized_metrics_output_id,
                expected_output_artifact_ids: row.expected_output_artifact_ids,
                stage_result_manifest_path: row.stage_result_manifest_path,
                stdout_path: row.stdout_path,
                stderr_path: row.stderr_path,
                readiness_kind: row.readiness_kind,
            })
            .collect())
    }
    fn report_map(&self, root: &Path) -> Result<Vec<FastqReportMapRow>> {
        Ok(render_fastq_report_map(root, DEFAULT_FASTQ_REPORT_MAP_PATH.into())?.rows)
    }
}

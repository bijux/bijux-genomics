use anyhow::Result;
use bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageReport;
use bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractReport;
use bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageReport;
use bijux_dna_bench_bam_evidence::report_map::BamReportMapReport;
use bijux_dna_bench_bam_proofs::retained_tools::observations::BindingObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ExpectedBindingObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ExpectedResultsObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::LocalSmokeObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::RenderedCommandsObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ToolMapObservation;
use bijux_dna_bench_bam_proofs::retained_tools::ports::BindingEvidence;
use std::path::Path;
pub(super) struct BindingAdapter;
impl BindingEvidence for BindingAdapter {
    fn tool_map(&self, root: &Path) -> Result<ToolMapObservation> {
        use super::super::tool_serving_map::{
            render_bam_tool_serving_map, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
        };
        let report = render_bam_tool_serving_map(root, DEFAULT_BAM_TOOL_SERVING_MAP_PATH.into())?;
        Ok(ToolMapObservation {
            row_count: report.row_count,
            rows: report
                .rows
                .into_iter()
                .map(|row| BindingObservation { stage_id: row.stage_id, tool_id: row.tool_id })
                .collect(),
            stage_count: report.stage_count,
            tool_count: report.tool_count,
        })
    }
    fn command_coverage(&self, root: &Path) -> Result<BamCommandAdapterCoverageReport> {
        use super::super::bam_command_adapter_coverage::{
            render_bam_command_adapter_coverage, DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH,
        };
        render_bam_command_adapter_coverage(root, DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH.into())
    }
    fn parser_coverage(&self, root: &Path) -> Result<BamParserFixtureCoverageReport> {
        use super::super::bam_parser_fixture_coverage::{
            render_bam_parser_fixture_coverage, DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
        };
        render_bam_parser_fixture_coverage(root, DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH.into())
    }
    fn local_smoke(&self, root: &Path) -> Result<LocalSmokeObservation> {
        use super::super::bam_local_container_probe::{
            render_bam_local_container_smoke, DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH,
        };
        let report =
            render_bam_local_container_smoke(root, DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH.into())?;
        Ok(LocalSmokeObservation {
            container_smoke_row_count: report.container_smoke_row_count,
            external_artifact_row_count: report.external_artifact_row_count,
            host_stage_smoke_row_count: report.host_stage_smoke_row_count,
            row_count: report.row_count,
            rows: report
                .rows
                .into_iter()
                .map(|row| BindingObservation { stage_id: row.stage_id, tool_id: row.tool_id })
                .collect(),
            stage_count: report.stage_count,
            tool_count: report.tool_count,
        })
    }
    fn output_declarations(&self, root: &Path) -> Result<BamAdapterOutputContractReport> {
        use super::super::bam_adapter_output_contract::{
            render_bam_adapter_output_contract, DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
        };
        render_bam_adapter_output_contract(root, DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH.into())
    }
    fn rendered_commands(&self, root: &Path) -> Result<RenderedCommandsObservation> {
        use super::super::bam_rendered_commands::{
            render_bam_commands, DEFAULT_BAM_RENDERED_COMMANDS_PATH,
        };
        let report = render_bam_commands(root, DEFAULT_BAM_RENDERED_COMMANDS_PATH.into())?;
        Ok(RenderedCommandsObservation {
            bash_syntax_passed: report.bash_syntax_passed,
            row_count: report.row_count,
            rows: report
                .rows
                .into_iter()
                .map(|row| BindingObservation { stage_id: row.stage_id, tool_id: row.tool_id })
                .collect(),
            stage_count: report.stage_count,
            tool_count: report.tool_count,
        })
    }
    fn expected_results(&self, root: &Path) -> Result<ExpectedResultsObservation> {
        use super::super::expected_benchmark_results::{
            render_expected_benchmark_results, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
        };
        let report = render_expected_benchmark_results(
            root,
            DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.into(),
        )?;
        Ok(ExpectedResultsObservation {
            rows: report
                .rows
                .into_iter()
                .map(|row| ExpectedBindingObservation {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    domain: row.domain,
                })
                .collect(),
        })
    }
    fn report_map(&self, root: &Path) -> Result<BamReportMapReport> {
        use super::super::bam_report_map::{render_bam_report_map, DEFAULT_BAM_REPORT_MAP_PATH};
        render_bam_report_map(root, DEFAULT_BAM_REPORT_MAP_PATH.into())
    }
}

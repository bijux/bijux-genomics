use super::super::bam_adapter_output_contract::{self, BamAdapterOutputContractReport};
use super::super::bam_command_adapter_coverage::{self, BamCommandAdapterCoverageReport};
use super::super::bam_parser_fixture_coverage::{self, BamParserFixtureCoverageReport};
use super::super::bam_report_map::{self, BamReportMapReport};
use super::super::expected_benchmark_results::{self, ExpectedBenchmarkResultsReport};
use super::super::tool_serving_map::{self, ToolServingMapReport};
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::ports::DeclarationEvidence;
use std::path::{Path, PathBuf};
pub(super) struct DeclarationAdapter;
impl DeclarationEvidence for DeclarationAdapter {
    fn serving(&self, root: &Path) -> Result<ToolServingMapReport> {
        tool_serving_map::render_bam_tool_serving_map(
            root,
            PathBuf::from(tool_serving_map::DEFAULT_BAM_TOOL_SERVING_MAP_PATH),
        )
    }
    fn commands(&self, root: &Path) -> Result<BamCommandAdapterCoverageReport> {
        bam_command_adapter_coverage::render_bam_command_adapter_coverage(
            root,
            PathBuf::from(bam_command_adapter_coverage::DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH),
        )
    }
    fn outputs(&self, root: &Path) -> Result<BamAdapterOutputContractReport> {
        bam_adapter_output_contract::render_bam_adapter_output_contract(
            root,
            PathBuf::from(bam_adapter_output_contract::DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH),
        )
    }
    fn parsers(&self, root: &Path) -> Result<BamParserFixtureCoverageReport> {
        bam_parser_fixture_coverage::render_bam_parser_fixture_coverage(
            root,
            PathBuf::from(bam_parser_fixture_coverage::DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH),
        )
    }
    fn expected(&self, root: &Path) -> Result<ExpectedBenchmarkResultsReport> {
        expected_benchmark_results::render_expected_benchmark_results(
            root,
            PathBuf::from(expected_benchmark_results::DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
        )
    }
    fn report_map(&self, root: &Path) -> Result<BamReportMapReport> {
        bam_report_map::render_bam_report_map(
            root,
            PathBuf::from(bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH),
        )
    }
    fn schema(&self) -> Result<serde_json::Value> {
        Ok(bijux_dna_api::v1::api::bench::render_bam_normalized_metrics_schema())
    }
}

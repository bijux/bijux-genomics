mod declarations;
mod native;
use super::{
    bam_adapter_output_contract::DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
    bam_command_adapter_coverage::DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH,
    bam_parser_fixture_coverage::DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
    bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH,
    expected_benchmark_results::DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
    tool_serving_map::DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
};
use bijux_dna_bench_bam_evidence::readiness::{ReadinessContext, ReadinessPaths};
use std::path::Path;
pub(super) fn context(repo_root: &Path) -> ReadinessContext<'_> {
    ReadinessContext {
        repo_root,
        declarations: &declarations::DeclarationAdapter,
        native: &native::NativeAdapter,
        paths: ReadinessPaths {
            serving: DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
            commands: DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH,
            outputs: DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
            parsers: DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
            expected: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
            report_map: DEFAULT_BAM_REPORT_MAP_PATH,
            schema:
                crate::commands::benchmark::schema_paths::DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH,
        },
    }
}

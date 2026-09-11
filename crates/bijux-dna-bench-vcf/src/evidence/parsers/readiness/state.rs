use bijux_dna_bench_vcf_evidence::model::{
    expectations::VcfExpectedBenchmarkResultsReport, parsers::VcfParserFixtureCoverageReport,
    reports::VcfReportMapReport,
};
#[derive(Clone, Copy, Default)]
pub(super) struct ReadinessCounts {
    pub parser_fixtures: usize,
    pub active_fixtures: usize,
    pub expected_results: usize,
    pub report_rows: usize,
    pub metrics: usize,
}
#[derive(Default)]
pub(super) struct ReadinessState {
    pub counts: ReadinessCounts,
    pub parser_coverage_report: Option<VcfParserFixtureCoverageReport>,
    pub expected_results_report: Option<VcfExpectedBenchmarkResultsReport>,
    pub report_map_report: Option<VcfReportMapReport>,
}

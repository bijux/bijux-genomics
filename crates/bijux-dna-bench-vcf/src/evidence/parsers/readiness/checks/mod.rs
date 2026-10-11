use crate::evidence::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use crate::evidence::metrics::DEFAULT_VCF_COMPARABLE_METRICS_PATH;
use crate::evidence::parsers::failure::DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH;
use crate::evidence::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
use crate::evidence::reports::DEFAULT_VCF_REPORT_MAP_PATH;
use crate::evidence::results::missing::DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH;
use bijux_dna_bench_artifacts::schemas::validation::DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH;
pub(super) mod completeness;
mod coverage;
mod expectations;
mod fixtures;
mod metrics;
mod record;
mod recovery;
mod reports;
mod schemas;
use super::{
    model::VcfParsersReportReadyGoalCheck, service::ActiveBindingProvider, state::ReadinessState,
    DEFAULT_VCF_PARSERS_REPORT_READY_PATH,
};
use crate::adapters::observations::VcfAdapterObservations;
use record::record_goal_check;
use std::path::Path;
pub(super) fn fixture_checks(repo_root: &Path, checks: &mut Vec<VcfParsersReportReadyGoalCheck>) {
    record_goal_check(
        checks,
        246,
        "vcf normalized metrics schemas",
        Some(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH.to_string()),
        || schemas::normalized_schemas(repo_root),
    );
    record_goal_check(checks, 247, "vcf bcftools parser fixtures", None, || {
        fixtures::bcftools(repo_root)
    });
    record_goal_check(checks, 248, "vcf angsd parser fixtures", None, || {
        fixtures::angsd(repo_root)
    });
    record_goal_check(checks, 249, "vcf plink family parser fixtures", None, || {
        fixtures::plink(repo_root)
    });
    record_goal_check(checks, 250, "vcf eigensoft parser fixtures", None, || {
        fixtures::eigensoft(repo_root)
    });
    record_goal_check(checks, 251, "vcf phasing parser fixtures", None, || {
        fixtures::phasing(repo_root)
    });
    record_goal_check(checks, 252, "vcf imputation parser fixtures", None, || {
        fixtures::imputation(repo_root)
    });
    record_goal_check(checks, 253, "vcf segment parser fixtures", None, || {
        fixtures::segments(repo_root)
    });
}
pub(super) fn evidence_checks(
    observations: &VcfAdapterObservations<'_>,
    active_bindings: ActiveBindingProvider,
    state: &mut ReadinessState,
    checks: &mut Vec<VcfParsersReportReadyGoalCheck>,
) {
    let repo_root = observations.repo_root();
    record_goal_check(
        checks,
        254,
        "vcf parser failure tests",
        Some(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH.to_string()),
        || recovery::parser_failures(repo_root),
    );
    record_goal_check(
        checks,
        255,
        "vcf comparable metrics",
        Some(DEFAULT_VCF_COMPARABLE_METRICS_PATH.to_string()),
        || metrics::comparable_metrics(observations, state),
    );
    record_goal_check(
        checks,
        256,
        "vcf parser fixture coverage",
        Some(DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH.to_string()),
        || coverage::parser_coverage(observations, active_bindings, state),
    );
    record_goal_check(
        checks,
        257,
        "vcf expected benchmark results",
        Some(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH.to_string()),
        || expectations::expected_results(observations, state),
    );
    record_goal_check(
        checks,
        258,
        "vcf missing-result report",
        Some(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH.to_string()),
        || recovery::missing_results(observations, state),
    );
    record_goal_check(
        checks,
        259,
        "vcf report map",
        Some(DEFAULT_VCF_REPORT_MAP_PATH.to_string()),
        || reports::report_map(observations, state),
    );
    record_goal_check(
        checks,
        260,
        "vcf parser/report ready pair completeness",
        Some(DEFAULT_VCF_PARSERS_REPORT_READY_PATH.to_string()),
        || completeness::complete_pairs(state),
    );
}

use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::ports::ToolComparisonObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageCommandManifest as BenchLocalStageCommandManifest;
use bijux_dna_bench_model::BenchmarkLocalStageManifestCompletionReport as BenchLocalStageManifestCompletionReport;
use bijux_dna_bench_model::BenchmarkLocalStageOutputCompletionReport as BenchLocalStageOutputCompletionReport;
use bijux_dna_bench_model::BenchmarkLocalStageRuntimeMetricsReport as BenchLocalStageRuntimeMetricsReport;
use bijux_dna_bench_model::BenchmarkLocalSummaryReport as BenchLocalBenchmarkSummaryReport;

pub(in crate::readiness) fn evaluate_tool_comparison_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &ToolComparisonObservation,
) {
    if report.row_count == 51 && report.observed_row_count == 51 {
        checks.push(ok_check(
            63,
            "benchmark_harness",
            "tool comparison template renders one governed row per benchmark stage",
            Some(report.tsv_output_path.clone()),
            format!("tool comparison template contains {} rows", report.row_count),
        ));
    } else {
        checks.push(fail_check(
            63,
            "benchmark_harness",
            "tool comparison template renders one governed row per benchmark stage",
            Some(report.tsv_output_path.clone()),
            format!(
                "expected 51 rows but found row_count={} rows_len={}",
                report.row_count, report.observed_row_count
            ),
        ));
    }
}

pub(in crate::readiness) fn evaluate_benchmark_summary_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalBenchmarkSummaryReport,
    rendered_stage_commands: Option<&BenchLocalStageCommandManifest>,
    output_completion: Option<&BenchLocalStageOutputCompletionReport>,
    manifest_completion: Option<&BenchLocalStageManifestCompletionReport>,
    runtime_metrics: Option<&BenchLocalStageRuntimeMetricsReport>,
    comparison_template: Option<&ToolComparisonObservation>,
) {
    let upstream_reports_are_aligned = rendered_stage_commands.is_some()
        && output_completion.is_some()
        && manifest_completion.is_some()
        && runtime_metrics.is_some()
        && comparison_template.is_some();
    if report.stage_count == 51
        && report.ready_stage_count == 51
        && report.failed_stage_count == 0
        && upstream_reports_are_aligned
    {
        checks.push(ok_check(
            64,
            "benchmark_harness",
            "benchmark summary renders governed JSON and Markdown surfaces",
            Some(report.report_output_path.clone()),
            format!(
                "benchmark summary renders {} ready stages with governed Markdown companion",
                report.stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            64,
            "benchmark_harness",
            "benchmark summary renders governed JSON and Markdown surfaces",
            Some(report.report_output_path.clone()),
            format!(
                "stage_count={} ready_stage_count={} failed_stage_count={} upstream_reports_aligned={upstream_reports_are_aligned}",
                report.stage_count, report.ready_stage_count, report.failed_stage_count
            ),
        ));
    }
}

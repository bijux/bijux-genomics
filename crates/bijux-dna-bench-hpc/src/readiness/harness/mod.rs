use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_BENCHMARK_SUMMARY_JSON_PATH;
use crate::readiness::declarations::DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH;
use crate::readiness::declarations::DEFAULT_LOCAL_STAGE_FAILURE_ROOT;
use crate::readiness::harness::runs::evaluate_resource_metrics_goal;
use crate::readiness::harness::runs::evaluate_stage_result_manifest_goal;
use crate::readiness::harness::summary::evaluate_benchmark_summary_goal;
use bijux_dna_bench_model::evaluate_benchmark_local_hpc_harness_readiness;
use bijux_dna_bench_model::BenchmarkLocalHpcHarnessObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use std::path::PathBuf;

mod commands;
mod render_commands;
mod runs;
mod summary;
use render_commands::render_commands;
mod observe_fake_runs;
use observe_fake_runs::observe_fake_runs;
mod observe_outputs;
use observe_outputs::observe_outputs;
mod observe_manifests;
use observe_manifests::observe_manifests;
mod observe_runtime;
use observe_runtime::observe_runtime;
mod observe_comparison;
use observe_comparison::observe_comparison;
pub(in crate::readiness) fn evaluate_benchmark_harness_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    fastq_stage_count: Option<usize>,
    bam_stage_count: Option<usize>,
) {
    let repo_root = context.repo_root;
    match (fastq_stage_count, bam_stage_count) {
        (Some(fastq_count), Some(bam_count)) if fastq_count == 27 && bam_count == 24 => {
            checks.push(ok_check(
                53,
                "benchmark_harness",
                "local stage listing covers the governed FASTQ and BAM slices",
                None,
                format!(
                    "list-stages surface resolves {fastq_count} FASTQ stages and {bam_count} BAM stages"
                ),
            ));
        }
        (Some(fastq_count), Some(bam_count)) => {
            checks.push(fail_check(
                53,
                "benchmark_harness",
                "local stage listing covers the governed FASTQ and BAM slices",
                None,
                format!(
                    "expected 27 FASTQ and 24 BAM stages but found {fastq_count} FASTQ and {bam_count} BAM"
                ),
            ));
        }
        _ => {
            checks.push(fail_check(
                53,
                "benchmark_harness",
                "local stage listing covers the governed FASTQ and BAM slices",
                None,
                "stage inventory loading failed earlier".to_string(),
            ));
        }
    }

    let rendered_stage_commands = render_commands(context, checks);

    let fake_run_manifest = observe_fake_runs(context, checks);

    match context.backend.fake_run_local_stage_failures(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAILURE_ROOT),
        &[],
        7,
    ) {
        Ok(manifest) => checks.push(evaluate_benchmark_local_hpc_harness_readiness(
            BenchmarkLocalHpcHarnessObservation::FakeFailures(&manifest),
        )),
        Err(err) => checks.push(fail_check(
            57,
            "benchmark_harness",
            "fake-run failure harness records structured stage failures",
            Some(DEFAULT_LOCAL_STAGE_FAILURE_ROOT.to_string()),
            format!("{err:#}"),
        )),
    }

    let output_completion = observe_outputs(context, checks);

    let manifest_completion = observe_manifests(context, checks);

    evaluate_stage_result_manifest_goal(context, checks, fake_run_manifest.as_ref());

    let runtime_metrics = observe_runtime(context, checks);

    evaluate_resource_metrics_goal(context, checks, fake_run_manifest.as_ref());

    let comparison_template = observe_comparison(context, checks);

    match context.backend.render_local_benchmark_summary(
        repo_root,
        PathBuf::from(context.paths.fake_run_root),
        PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_JSON_PATH),
        PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH),
    ) {
        Ok(report) => evaluate_benchmark_summary_goal(
            checks,
            &report,
            rendered_stage_commands.as_ref(),
            output_completion.as_ref(),
            manifest_completion.as_ref(),
            runtime_metrics.as_ref(),
            comparison_template.as_ref(),
        ),
        Err(err) => checks.push(fail_check(
            64,
            "benchmark_harness",
            "benchmark summary renders governed JSON and Markdown surfaces",
            Some(DEFAULT_BENCHMARK_SUMMARY_JSON_PATH.to_string()),
            format!("{err:#}"),
        )),
    }
}

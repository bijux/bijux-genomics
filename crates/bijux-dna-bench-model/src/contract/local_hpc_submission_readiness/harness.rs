//! Owner: admitted local harness observations and stable HPC readiness decisions.
use crate::{
    BenchmarkLocalHpcSubmissionReadinessGoalCheck as GoalCheck,
    BenchmarkLocalStageFakeFailureManifest, BenchmarkLocalStageManifestCompletionReport,
    BenchmarkLocalStageOutputCompletionReport, BenchmarkLocalStageRuntimeMetricsReport,
};

/// Borrow complete local harness reports. These describe synthetic contract checks, not fresh
/// scientific tool execution. Source acquisition and publication belong to the calling workflow.
#[derive(Debug, Clone, Copy)]
pub enum BenchmarkLocalHpcHarnessObservation<'a> {
    FakeFailures(&'a BenchmarkLocalStageFakeFailureManifest),
    OutputCompletion(&'a BenchmarkLocalStageOutputCompletionReport),
    ManifestCompletion(&'a BenchmarkLocalStageManifestCompletionReport),
    RuntimeMetrics(&'a BenchmarkLocalStageRuntimeMetricsReport),
}

/// Admit the complete report before evaluating the retained governed readiness criterion.
/// Invalid observations retain their goal identity and become failed checks.
#[must_use]
pub fn evaluate_benchmark_local_hpc_harness_readiness(
    observation: BenchmarkLocalHpcHarnessObservation<'_>,
) -> GoalCheck {
    let (admission, mut check) = match observation {
        BenchmarkLocalHpcHarnessObservation::FakeFailures(report) => (
            crate::validate_benchmark_local_stage_fake_failure_manifest(report),
            evaluate_fake_run_failure_goal(report),
        ),
        BenchmarkLocalHpcHarnessObservation::OutputCompletion(report) => (
            crate::validate_benchmark_local_stage_output_completion(report),
            evaluate_output_completion_goal(report),
        ),
        BenchmarkLocalHpcHarnessObservation::ManifestCompletion(report) => (
            crate::validate_benchmark_local_stage_manifest_completion(report),
            evaluate_manifest_completion_goal(report),
        ),
        BenchmarkLocalHpcHarnessObservation::RuntimeMetrics(report) => (
            crate::validate_benchmark_local_runtime_metrics(report),
            evaluate_runtime_metrics_goal(report),
        ),
    };
    if let Err(error) = admission {
        check.ok = false;
        check.detail = format!("invalid local harness observation: {error}");
    }
    check
}

fn evaluate_fake_run_failure_goal(manifest: &BenchmarkLocalStageFakeFailureManifest) -> GoalCheck {
    let fields_are_present = manifest.failures.iter().all(|failure| {
        !failure.stage_id.trim().is_empty()
            && !failure.tool_id.trim().is_empty()
            && !failure.command.trim().is_empty()
            && !failure.stderr_path.trim().is_empty()
            && !failure.failure_record_path.trim().is_empty()
            && failure.exit_code == 7
    });
    if manifest.stage_count == 51 && fields_are_present {
        ok_check(
            57,
            "benchmark_harness",
            "fake-run failure harness records structured stage failures",
            Some(manifest.failure_root.clone()),
            format!("fake-run failure manifest covers {} stages", manifest.stage_count),
        )
    } else {
        fail_check(
            57,
            "benchmark_harness",
            "fake-run failure harness records structured stage failures",
            Some(manifest.failure_root.clone()),
            format!("stage_count={} structured_fields={fields_are_present}", manifest.stage_count),
        )
    }
}

fn evaluate_output_completion_goal(
    report: &BenchmarkLocalStageOutputCompletionReport,
) -> GoalCheck {
    if report.complete && report.stage_count == 51 && report.incomplete_stage_count == 0 {
        ok_check(
            58,
            "benchmark_harness",
            "output completion requires every declared stage output",
            Some(report.report_output_path.clone()),
            format!("output completion marks all {} governed stages complete", report.stage_count),
        )
    } else {
        fail_check(
            58,
            "benchmark_harness",
            "output completion requires every declared stage output",
            Some(report.report_output_path.clone()),
            format!(
                "complete={} stage_count={} incomplete_stage_count={}",
                report.complete, report.stage_count, report.incomplete_stage_count
            ),
        )
    }
}

fn evaluate_manifest_completion_goal(
    report: &BenchmarkLocalStageManifestCompletionReport,
) -> GoalCheck {
    if report.complete && report.stage_count == 51 && report.incomplete_stage_count == 0 {
        ok_check(
            59,
            "benchmark_harness",
            "manifest completion requires every stage-result manifest",
            Some(report.report_output_path.clone()),
            format!(
                "manifest completion marks all {} governed stages complete",
                report.stage_count
            ),
        )
    } else {
        fail_check(
            59,
            "benchmark_harness",
            "manifest completion requires every stage-result manifest",
            Some(report.report_output_path.clone()),
            format!(
                "complete={} stage_count={} incomplete_stage_count={}",
                report.complete, report.stage_count, report.incomplete_stage_count
            ),
        )
    }
}

fn evaluate_runtime_metrics_goal(report: &BenchmarkLocalStageRuntimeMetricsReport) -> GoalCheck {
    let runtime_fields_are_present = report.stages.iter().all(|stage| {
        !stage.runtime_mode.trim().is_empty()
            && !stage.started_at.trim().is_empty()
            && !stage.finished_at.trim().is_empty()
    });
    if report.stage_count == 51 && runtime_fields_are_present {
        ok_check(
            61,
            "benchmark_harness",
            "runtime metrics extract validated start, finish, elapsed, exit, and status fields",
            Some(report.report_output_path.clone()),
            format!("runtime metrics report covers {} stages", report.stage_count),
        )
    } else {
        fail_check(
            61,
            "benchmark_harness",
            "runtime metrics extract validated start, finish, elapsed, exit, and status fields",
            Some(report.report_output_path.clone()),
            format!(
                "stage_count={} runtime_fields_present={runtime_fields_are_present}",
                report.stage_count
            ),
        )
    }
}

fn ok_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> GoalCheck {
    GoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: true,
        detail: detail.into(),
    }
}

fn fail_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> GoalCheck {
    GoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: false,
        detail: detail.into(),
    }
}

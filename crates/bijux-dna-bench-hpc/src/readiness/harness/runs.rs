use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageFakeRunManifest as BenchLocalStageFakeRunManifest;
use bijux_dna_bench_model::BenchmarkStageResultResourceMetricSource as BenchStageResultResourceMetricSource;

pub(in crate::readiness) fn evaluate_fake_run_success_goal(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    manifest: &BenchLocalStageFakeRunManifest,
) {
    let repo_root = context.repo_root;
    let all_stage_outputs_exist = manifest
        .stages
        .iter()
        .all(|stage| stage.created_output_count == stage.declared_output_count);
    let all_manifests_exist =
        manifest.stages.iter().all(|stage| repo_root.join(&stage.stage_manifest_path).is_file());
    if manifest.stage_count == 51 && all_stage_outputs_exist && all_manifests_exist {
        checks.push(ok_check(
            56,
            "benchmark_harness",
            "fake-run stage harness materializes every governed stage output and manifest",
            Some(manifest.fake_run_root.clone()),
            format!("fake-run manifest covers {} stages", manifest.stage_count),
        ));
    } else {
        checks.push(fail_check(
            56,
            "benchmark_harness",
            "fake-run stage harness materializes every governed stage output and manifest",
            Some(manifest.fake_run_root.clone()),
            format!(
                "stage_count={} output_complete={} manifest_complete={}",
                manifest.stage_count, all_stage_outputs_exist, all_manifests_exist
            ),
        ));
    }
}

pub(in crate::readiness) fn evaluate_stage_result_manifest_goal(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    fake_run_manifest: Option<&BenchLocalStageFakeRunManifest>,
) {
    let repo_root = context.repo_root;
    let Some(fake_run_manifest) = fake_run_manifest else {
        checks.push(fail_check(
            60,
            "benchmark_harness",
            "stage-result manifests validate the governed runtime contract",
            Some("artifacts/benchmark-smoke/simulated-runs/stages/*/stage-result.json".to_string()),
            "fake-run stage generation failed earlier".to_string(),
        ));
        return;
    };

    let mut validation_errors = Vec::new();
    for stage in &fake_run_manifest.stages {
        let manifest_path = repo_root.join(&stage.stage_manifest_path);
        if let Err(err) = context.backend.load_validated_stage_result_manifest_path(&manifest_path)
        {
            validation_errors.push(format!("{}: {err:#}", stage.stage_id));
        }
    }
    if validation_errors.is_empty() {
        checks.push(ok_check(
            60,
            "benchmark_harness",
            "stage-result manifests validate the governed runtime contract",
            Some("artifacts/benchmark-smoke/simulated-runs/stages/*/stage-result.json".to_string()),
            format!("validated {} governed stage-result manifests", fake_run_manifest.stage_count),
        ));
    } else {
        checks.push(fail_check(
            60,
            "benchmark_harness",
            "stage-result manifests validate the governed runtime contract",
            Some("artifacts/benchmark-smoke/simulated-runs/stages/*/stage-result.json".to_string()),
            validation_errors.join("; "),
        ));
    }
}

pub(in crate::readiness) fn evaluate_resource_metrics_goal(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    fake_run_manifest: Option<&BenchLocalStageFakeRunManifest>,
) {
    let repo_root = context.repo_root;
    let Some(fake_run_manifest) = fake_run_manifest else {
        checks.push(fail_check(
            62,
            "benchmark_harness",
            "stage-result manifests carry explicit resource metric provenance",
            Some("artifacts/benchmark-smoke/simulated-runs/stages/*/stage-result.json".to_string()),
            "fake-run stage generation failed earlier".to_string(),
        ));
        return;
    };

    let mut invalid_sources = Vec::new();
    for stage in &fake_run_manifest.stages {
        let manifest_path = repo_root.join(&stage.stage_manifest_path);
        match context.backend.load_validated_stage_result_manifest_path(&manifest_path) {
            Ok(manifest) => match manifest.resource_metrics.source {
                BenchStageResultResourceMetricSource::Measured
                | BenchStageResultResourceMetricSource::Estimated
                | BenchStageResultResourceMetricSource::NotAvailable => {}
            },
            Err(err) => invalid_sources.push(format!("{}: {err:#}", stage.stage_id)),
        }
    }
    if invalid_sources.is_empty() {
        checks.push(ok_check(
            62,
            "benchmark_harness",
            "stage-result manifests carry explicit resource metric provenance",
            Some("artifacts/benchmark-smoke/simulated-runs/stages/*/stage-result.json".to_string()),
            format!(
                "resource_metrics.source is explicit for {} governed stage results",
                fake_run_manifest.stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            62,
            "benchmark_harness",
            "stage-result manifests carry explicit resource metric provenance",
            Some("artifacts/benchmark-smoke/simulated-runs/stages/*/stage-result.json".to_string()),
            invalid_sources.join("; "),
        ));
    }
}

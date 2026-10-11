use bijux_dna_bench_model::{
    benchmark_local_hpc_submission_readiness_schema,
    validate_benchmark_local_hpc_submission_readiness,
    BenchmarkLocalHpcSubmissionReadinessGoalCheck, BenchmarkLocalHpcSubmissionReadinessReport,
    BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1,
};

fn category(goal_id: u32) -> &'static str {
    match goal_id {
        1 | 28 => "stage_matrices",
        2..=27 | 29..=52 => "local_smokes",
        53..=64 => "benchmark_harness",
        65..=76 => "corpora",
        77..=90 => "pipeline_dags",
        91..=99 => "slurm_dry_run",
        _ => unreachable!("test goal outside governed inventory"),
    }
}

fn report() -> BenchmarkLocalHpcSubmissionReadinessReport {
    let checks = (1..=99)
        .map(|goal_id| BenchmarkLocalHpcSubmissionReadinessGoalCheck {
            goal_id,
            category: category(goal_id).to_string(),
            surface: format!("governed submission-readiness goal {goal_id}"),
            output_path: Some(format!(
                "artifacts/benchmark-smoke/readiness/local-ready/{goal_id}.json"
            )),
            ok: true,
            detail: "governed submission evidence admitted".to_string(),
        })
        .collect::<Vec<_>>();
    BenchmarkLocalHpcSubmissionReadinessReport {
        schema_version: BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/local-ready/HPC_SUBMISSION_READY.json"
            .to_string(),
        checked_goal_count: 99,
        passed_goal_count: 99,
        failed_goal_count: 0,
        failing_goal_ids: Vec::new(),
        ok: true,
        checks,
    }
}

#[test]
fn local_hpc_submission_readiness_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_submission_readiness(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcSubmissionReadinessReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_submission_readiness_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-submission-readiness.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC submission-readiness schema drifted");
    Ok(())
}

#[test]
fn local_hpc_submission_readiness_admits_coherent_blockers() -> anyhow::Result<()> {
    let mut report = report();
    report.checks[64].ok = false;
    report.checks[65].ok = false;
    report.passed_goal_count = 97;
    report.failed_goal_count = 2;
    report.failing_goal_ids = vec![65, 66];
    report.ok = false;

    validate_benchmark_local_hpc_submission_readiness(&report)?;
    Ok(())
}

#[test]
fn local_hpc_submission_readiness_rejects_inventory_category_path_and_counter_drift() {
    let mut duplicate = report();
    duplicate.checks[1].goal_id = 1;
    assert!(validate_benchmark_local_hpc_submission_readiness(&duplicate).is_err());

    let mut category = report();
    category.checks[64].category = "fixtures".to_string();
    assert!(validate_benchmark_local_hpc_submission_readiness(&category).is_err());

    let mut path = report();
    path.checks[0].output_path = Some("/tmp/unowned.json".to_string());
    assert!(validate_benchmark_local_hpc_submission_readiness(&path).is_err());

    let mut counters = report();
    counters.passed_goal_count = 98;
    assert!(validate_benchmark_local_hpc_submission_readiness(&counters).is_err());
}

#[test]
fn slurm_script_body_owner_preserves_findings_and_rejects_forged_wire_aggregates(
) -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        build_benchmark_local_slurm_script_body_report, inspect_benchmark_local_slurm_script_body,
        validate_benchmark_local_slurm_script_body_report, BenchmarkLocalSlurmScriptBodyReport,
    };
    let root = "artifacts/runs/bench/slurm-dry-run";
    let output = format!("{root}/inspection.json");
    let script = format!("{root}/stage.sbatch");
    let entry = inspect_benchmark_local_slurm_script_body(
        script.clone(),
        "#!/bin/bash\nset -euo pipefail\ncargo run -p bijux-dna -- bench local status\n",
    );
    assert!(entry.ok && entry.has_bijux_dna_command);
    let report = build_benchmark_local_slurm_script_body_report(
        root.to_string(),
        output.clone(),
        vec![entry],
    )?;
    assert!(report.ok);
    let mut wire = serde_json::to_value(&report)?;
    wire["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkLocalSlurmScriptBodyReport>(wire).is_err());
    let mut corrupt = report.clone();
    corrupt.script_count += 1;
    assert!(validate_benchmark_local_slurm_script_body_report(&corrupt).is_err());
    corrupt = report.clone();
    corrupt.schema_version = "unsupported".to_string();
    assert!(validate_benchmark_local_slurm_script_body_report(&corrupt).is_err());
    corrupt = report.clone();
    corrupt.scripts.push(corrupt.scripts[0].clone());
    corrupt.script_count += 1;
    assert!(validate_benchmark_local_slurm_script_body_report(&corrupt).is_err());
    corrupt = report;
    corrupt.scripts[0].script_path = "../outside.sbatch".to_string();
    assert!(validate_benchmark_local_slurm_script_body_report(&corrupt).is_err());
    let entry = inspect_benchmark_local_slurm_script_body(
        script,
        "# TODO: implement\nrc=0\necho execute placeholder\n",
    );
    assert!(!entry.ok && !entry.has_bijux_dna_command);
    assert_eq!(
        entry.findings,
        [
            "contains a to-do marker",
            "contains unconditional `rc=0`",
            "contains `placeholder`",
            "contains fake `echo execute` job body",
            "missing `bijux-dna` command"
        ]
    );
    let failed = build_benchmark_local_slurm_script_body_report(
        root.to_string(),
        output.clone(),
        vec![entry],
    )?;
    assert!(!failed.ok);
    let empty =
        build_benchmark_local_slurm_script_body_report(root.to_string(), output, Vec::new())?;
    assert!(!empty.ok && empty.findings_count == 1);
    Ok(())
}

#[test]
fn hpc_slurm_readiness_admits_complete_observations_before_reporting_success() -> anyhow::Result<()>
{
    use bijux_dna_bench_model::{
        build_benchmark_local_slurm_script_body_report,
        evaluate_benchmark_local_hpc_slurm_readiness as evaluate,
        inspect_benchmark_local_slurm_script_body,
        BenchmarkLocalHpcSlurmObservation as Observation, BenchmarkLocalSlurmSubmitManifest,
        BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    };
    let manifest = BenchmarkLocalSlurmSubmitManifest {
        schema_version: BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1.to_string(),
        root_path: "artifacts/runs/bench/slurm-dry-run".to_string(),
        manifest_path: "artifacts/runs/bench/slurm-dry-run/submit-manifest.json".to_string(),
        run_id: "local-benchmark-dry-run".to_string(),
        job_count: 51,
        dependency_count: 0,
        jobs: Vec::new(),
    };
    let forged = evaluate(Observation::SubmitManifest(&manifest));
    assert_eq!(forged.goal_id, 96);
    assert!(!forged.ok && forged.detail.starts_with("invalid local Slurm observation:"));
    let missing = evaluate(Observation::RunPaths { bam: None, submission: None });
    assert_eq!(missing.goal_id, 98);
    assert!(!missing.ok && missing.detail == "BAM SLURM script rendering failed earlier");
    let root = "artifacts/runs/bench/slurm-dry-run";
    let entry = inspect_benchmark_local_slurm_script_body(
        format!("{root}/stage.sbatch"),
        "bijux-dna --help\n",
    );
    let mut report = build_benchmark_local_slurm_script_body_report(
        root.to_string(),
        format!("{root}/inspection.json"),
        vec![entry],
    )?;
    let admitted = evaluate(Observation::ScriptBodies(&report));
    assert_eq!(admitted.goal_id, 94);
    assert!(admitted.ok);
    report.script_count += 1;
    assert!(!evaluate(Observation::ScriptBodies(&report)).ok);
    Ok(())
}

#[test]
fn hpc_harness_readiness_rejects_fabricated_counts_and_admits_runtime_rows() -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        evaluate_benchmark_local_hpc_harness_readiness as evaluate,
        BenchmarkLocalHpcHarnessObservation as Observation, BenchmarkLocalStageFakeFailureManifest,
        BenchmarkLocalStageManifestCompletionReport, BenchmarkLocalStageOutputCompletionReport,
        BenchmarkLocalStageReadinessKind, BenchmarkLocalStageRuntimeMetric,
        BenchmarkLocalStageRuntimeMetricsReport, BenchmarkStageResultStatus,
        BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1,
        BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1,
        BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
        BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1,
    };
    let failures = BenchmarkLocalStageFakeFailureManifest {
        schema_version: BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1.to_string(),
        failure_root: "artifacts/runs/bench/failures".to_string(),
        source_stage_command_manifest_path: "artifacts/runs/bench/commands.json".to_string(),
        stage_count: 51,
        failures: Vec::new(),
    };
    let mut completion = serde_json::json!({
        "schema_version": BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1,
        "fake_run_root": "artifacts/runs/bench/fake",
        "report_output_path": "artifacts/runs/bench/completion.json",
        "source_stage_command_manifest_path": "artifacts/runs/bench/commands.json",
        "stage_count":51, "complete_stage_count":51, "incomplete_stage_count":0,
        "complete":true, "stages":[],
    });
    let outputs: BenchmarkLocalStageOutputCompletionReport =
        serde_json::from_value(completion.clone())?;
    completion["schema_version"] =
        serde_json::json!(BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1);
    let manifests: BenchmarkLocalStageManifestCompletionReport =
        serde_json::from_value(completion)?;
    let mut runtime = BenchmarkLocalStageRuntimeMetricsReport {
        schema_version: BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1.to_string(),
        fake_run_root: "artifacts/runs/bench/fake".to_string(),
        report_output_path: "artifacts/runs/bench/runtime.json".to_string(),
        source_stage_command_manifest_path: "artifacts/runs/bench/commands.json".to_string(),
        stage_count: 51,
        stages: Vec::new(),
    };
    for (goal, observation) in [
        (57, Observation::FakeFailures(&failures)),
        (58, Observation::OutputCompletion(&outputs)),
        (59, Observation::ManifestCompletion(&manifests)),
        (61, Observation::RuntimeMetrics(&runtime)),
    ] {
        let rejected = evaluate(observation);
        assert_eq!(rejected.goal_id, goal);
        assert_eq!(rejected.category, "benchmark_harness");
        assert!(!rejected.ok);
        assert!(rejected.detail.starts_with("invalid local harness observation:"));
    }
    runtime.stages = (0..51)
        .map(|index| BenchmarkLocalStageRuntimeMetric {
            stage_id: format!("fixture.stage-{index}"),
            readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
            tool_id: "fixture-tool".to_string(),
            manifest_path: format!("artifacts/runs/bench/{index}.json"),
            runtime_mode: "synthetic".to_string(),
            started_at: "2026-09-10T12:00:00Z".to_string(),
            finished_at: "2026-09-10T12:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code: 0,
            status: BenchmarkStageResultStatus::Succeeded,
        })
        .collect();
    let admitted = evaluate(Observation::RuntimeMetrics(&runtime));
    assert!(admitted.ok);
    assert_eq!(admitted.detail, "runtime metrics report covers 51 stages");
    runtime.stages[0].elapsed_seconds = f64::NAN;
    assert!(!evaluate(Observation::RuntimeMetrics(&runtime)).ok);
    runtime.stages[0].elapsed_seconds = 1.0;
    runtime.stages[1] = runtime.stages[0].clone();
    assert!(!evaluate(Observation::RuntimeMetrics(&runtime)).ok);
    Ok(())
}

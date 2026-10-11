use bijux_dna_bench_model::{
    benchmark_local_runtime_metrics_schema, validate_benchmark_local_runtime_metrics,
    BenchmarkLocalStageReadinessKind, BenchmarkLocalStageRuntimeMetric,
    BenchmarkLocalStageRuntimeMetricsReport, BenchmarkStageResultStatus,
    BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1,
};

fn report() -> BenchmarkLocalStageRuntimeMetricsReport {
    BenchmarkLocalStageRuntimeMetricsReport {
        schema_version: BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1.to_string(),
        fake_run_root: "artifacts/benchmark-smoke/simulated-runs/stages".to_string(),
        report_output_path: "artifacts/benchmark-smoke/readiness/runtime-metrics.json".to_string(),
        source_stage_command_manifest_path:
            "artifacts/benchmark-smoke/readiness/rendered-stage-commands.sh".to_string(),
        stage_count: 1,
        stages: vec![BenchmarkLocalStageRuntimeMetric {
            stage_id: "fastq.report_qc".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::DryOrSmoke,
            tool_id: "multiqc".to_string(),
            manifest_path: "artifacts/results/stage-result.json".to_string(),
            runtime_mode: "fake_run".to_string(),
            started_at: "2026-08-12T00:00:00Z".to_string(),
            finished_at: "2026-08-12T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code: 0,
            status: BenchmarkStageResultStatus::Succeeded,
        }],
    }
}

#[test]
fn local_runtime_metrics_round_trip_and_schema_match_model_authority() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_runtime_metrics(&report)?;
    let serialized = serde_json::to_vec(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalStageRuntimeMetricsReport>(&serialized)?,
        report
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_runtime_metrics_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-runtime-metrics.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local runtime-metrics schema drifted from model authority");
    Ok(())
}

#[test]
fn local_runtime_metrics_reject_duplicate_or_non_finite_stages() {
    let mut duplicate = report();
    duplicate.stages.push(duplicate.stages[0].clone());
    duplicate.stage_count = duplicate.stages.len();
    assert!(validate_benchmark_local_runtime_metrics(&duplicate).is_err());

    let mut non_finite = report();
    non_finite.stages[0].elapsed_seconds = f64::NAN;
    assert!(validate_benchmark_local_runtime_metrics(&non_finite).is_err());
}
